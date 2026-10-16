//! Private operation exclusion and invalidation, disconnected from Filesystem.
//! Contract: docs/CASEFOLD-INDEX-REVIEW.md, concrete serialization audit.
//! No ownership proof, buffer seal, readiness or durable publication is provided.

use crate::error::{Error, Result};
use std::sync::{Mutex, MutexGuard, TryLockError};

const ACTIVE: Error = Error::Unsupported("filesystem operation already active");
const FAULT: Error = Error::Corrupt("operation control faulted");
const LIMIT: Error = Error::Unsupported("operation version exhausted");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Validity {
    Unbuilt,
    Invalid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Health {
    Healthy,
    Uncertain,
    Faulted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Status {
    pub epoch: u64,
    /// Control-local change counter, not a seal or a BlockBuffer revision yet.
    pub revision: u64,
    pub validity: Validity,
    pub health: Health,
}

/// One control per future owned mount/session. No explicit heap reservation or
/// filesystem I/O; synchronization internals depend on the target platform.
/// Callers may independently own it in an Arc during mutable maintenance; merely
/// constructing/sharing it establishes neither device exclusivity nor readiness.
pub(crate) struct Control {
    state: Mutex<Status>,
}

/// A freshness marker only. Borrowing its owner prevents address reuse while it
/// exists; matching counters from another control never establish identity.
/// It carries no ownership claims, admission rights or device/buffer evidence.
#[derive(Clone, Copy)]
pub(crate) struct Stamp<'a> {
    owner: &'a Control,
    epoch: u64,
    revision: u64,
}

/// One non-Send, exclusive context; nested helpers borrow this instead of entry.
pub(crate) struct Operation<'a> {
    owner: &'a Control,
    state: MutexGuard<'a, Status>,
}

impl Control {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(Status {
                epoch: 0,
                revision: 0,
                validity: Validity::Unbuilt,
                health: Health::Healthy,
            }),
        }
    }

    /// Never wait: this includes same-thread and callback-worker reentry.
    /// Poison/fault has no reset API; recovery/remount needs a different control.
    pub fn try_enter(&self) -> Result<Operation<'_>> {
        let state = match self.state.try_lock() {
            Ok(state) => state,
            Err(TryLockError::WouldBlock) => return Err(ACTIVE),
            Err(TryLockError::Poisoned(_)) => {
                return Err(Error::Corrupt("operation gate poisoned"));
            }
        };
        let op = Operation { owner: self, state };
        op.healthy()?;
        Ok(op)
    }
}

impl<'a> Operation<'a> {
    fn healthy(&self) -> Result<()> {
        if self.state.health != Health::Healthy {
            return Err(FAULT);
        }
        Ok(())
    }

    pub fn status(&self) -> Status {
        *self.state
    }

    pub fn stamp(&self) -> Result<Stamp<'a>> {
        self.healthy()?;
        Ok(Stamp {
            owner: self.owner,
            epoch: self.state.epoch,
            revision: self.state.revision,
        })
    }

    pub fn check(&self, stamp: Stamp<'_>) -> Result<()> {
        self.healthy()?;
        if !std::ptr::eq(self.owner, stamp.owner)
            || (self.state.epoch, self.state.revision) != (stamp.epoch, stamp.revision)
        {
            return Err(Error::Stale);
        }
        Ok(())
    }

    /// Before future mutable staging exposure. No actual buffer is bound here.
    pub fn revise(&mut self) -> Result<()> {
        self.advance(false)
    }

    /// Revoke view freshness before any raw/logical-view change. No rebuild.
    pub fn invalidate(&mut self) -> Result<()> {
        self.advance(true)
    }

    fn advance(&mut self, view: bool) -> Result<()> {
        self.healthy()?;
        self.state.validity = Validity::Invalid;
        let revision = self.state.revision.checked_add(1);
        let epoch = self.state.epoch.checked_add(u64::from(view));
        match (epoch, revision) {
            (Some(epoch), Some(revision)) => {
                self.state.epoch = epoch;
                self.state.revision = revision;
                Ok(())
            }
            _ => {
                // Never wrap/partly advance into a version an old marker held.
                self.state.health = Health::Faulted;
                Err(LIMIT)
            }
        }
    }

    /// Invalidate before external side effects, including a write/flush attempt.
    /// The closure must encompass the whole effect and propagate its failures.
    /// Ok only ends uncertainty; it does not prove durability or publish owners.
    /// Any Err, even before a known physical write, conservatively faults reuse.
    pub fn effect<T>(&mut self, action: impl FnOnce(Status) -> Result<T>) -> Result<T> {
        self.invalidate()?;
        self.state.health = Health::Uncertain;
        let result = action(self.status());
        self.state.health = if result.is_ok() {
            Health::Healthy
        } else {
            Health::Faulted
        };
        result
    }
}

impl Drop for Operation<'_> {
    fn drop(&mut self) {
        // Also cover a panic caught while this guard remains alive: the mutex
        // need not be poisoned then, but Uncertain must never become usable.
        if std::thread::panicking() || self.state.health == Health::Uncertain {
            self.state.validity = Validity::Invalid;
            self.state.health = Health::Faulted;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{mpsc, Arc, Barrier};
    use std::time::Duration;

    const WAIT: Duration = Duration::from_secs(5);

    #[test]
    fn untouched_abort_releases_entry_without_creating_readiness() {
        let control = Control::new();
        let first = control.try_enter().unwrap();
        let stamp = first.stamp().unwrap();
        let initial = first.status();
        assert_eq!(initial.validity, Validity::Unbuilt);
        drop(first);
        let next = control.try_enter().unwrap();
        next.check(stamp).unwrap();
        assert_eq!(next.status(), initial);
    }

    #[test]
    fn same_thread_nested_entry_refuses_promptly() {
        let (tx, rx) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            let control = Control::new();
            let _held = control.try_enter().unwrap();
            tx.send(matches!(control.try_enter(), Err(Error::Unsupported(_))))
                .unwrap();
        });
        assert!(rx.recv_timeout(WAIT).expect("nested entry blocked"));
        worker.join().unwrap();
    }

    #[test]
    fn callback_can_wait_for_a_worker_that_refuses_same_control() {
        let control = Arc::new(Control::new());
        let mut outer = control.try_enter().unwrap();
        outer
            .effect(|_| {
                let shared = control.clone();
                let (tx, rx) = mpsc::channel();
                let worker = std::thread::spawn(move || {
                    tx.send(matches!(shared.try_enter(), Err(Error::Unsupported(_))))
                        .unwrap();
                });
                assert!(rx.recv_timeout(WAIT).expect("callback worker blocked"));
                worker.join().unwrap();
                Ok(())
            })
            .unwrap();
        drop(outer);
        assert!(control.try_enter().is_ok());
    }

    #[test]
    fn simultaneous_contenders_execute_exactly_one_effect() {
        let control = Arc::new(Control::new());
        let barrier = Arc::new(Barrier::new(9));
        let effects = Arc::new(AtomicUsize::new(0));
        let (tx, rx) = mpsc::channel();
        let mut release = Vec::new();
        let mut workers = Vec::new();
        for _ in 0..8 {
            let (permit, wait) = mpsc::channel();
            release.push(permit);
            let (control, barrier, effects, tx) = (
                control.clone(),
                barrier.clone(),
                effects.clone(),
                tx.clone(),
            );
            workers.push(std::thread::spawn(move || {
                barrier.wait();
                match control.try_enter() {
                    Ok(mut op) => {
                        op.effect(|_| {
                            assert_eq!(effects.fetch_add(1, Ordering::SeqCst), 0);
                            Ok(())
                        })
                        .unwrap();
                        tx.send(true).unwrap();
                        wait.recv_timeout(WAIT).expect("winner not released");
                    }
                    Err(Error::Unsupported(_)) => tx.send(false).unwrap(),
                    Err(e) => panic!("unexpected contender error: {e}"),
                }
            }));
        }
        barrier.wait();
        let winners = (0..8)
            .map(|_| usize::from(rx.recv_timeout(WAIT).expect("contender blocked")))
            .sum::<usize>();
        assert_eq!((winners, effects.load(Ordering::SeqCst)), (1, 1));
        for permit in release {
            let _ = permit.send(());
        }
        for worker in workers {
            worker.join().unwrap();
        }
        assert!(control.try_enter().is_ok());
    }

    #[test]
    fn nested_helpers_reuse_context_without_reentry() {
        fn inner(op: &mut Operation<'_>) -> Result<()> {
            op.revise()
        }
        let control = Control::new();
        let mut op = control.try_enter().unwrap();
        let stamp = op.stamp().unwrap();
        inner(&mut op).unwrap();
        assert!(matches!(op.check(stamp), Err(Error::Stale)));
        op.effect(|_| Ok(())).unwrap();
        assert_eq!((op.status().epoch, op.status().revision), (1, 2));
    }

    #[test]
    fn identity_is_control_bound_even_with_matching_versions() {
        let first = Arc::new(Control::new());
        let alias = first.clone();
        let other = Control::new();
        let stamp = first.try_enter().unwrap().stamp().unwrap();
        alias.try_enter().unwrap().check(stamp).unwrap();
        assert!(matches!(
            other.try_enter().unwrap().check(stamp),
            Err(Error::Stale)
        ));
    }

    #[test]
    fn stale_view_stamp_cannot_survive_invalidation_and_reentry() {
        let control = Control::new();
        let mut op = control.try_enter().unwrap();
        let old = op.stamp().unwrap();
        op.invalidate().unwrap();
        drop(op);
        let next = control.try_enter().unwrap();
        assert!(matches!(next.check(old), Err(Error::Stale)));
        assert_eq!(next.status().validity, Validity::Invalid);
    }

    #[test]
    fn revision_alone_revokes_old_stamps_without_advancing_epoch() {
        let control = Control::new();
        let mut op = control.try_enter().unwrap();
        let old = op.stamp().unwrap();
        op.revise().unwrap();
        assert_eq!((op.status().epoch, op.status().revision), (0, 1));
        assert!(matches!(op.check(old), Err(Error::Stale)));
    }

    #[test]
    fn invalidation_and_uncertainty_precede_the_effect() {
        let control = Control::new();
        let mut op = control.try_enter().unwrap();
        let changed = AtomicUsize::new(0);
        let value = op
            .effect(|before| {
                assert_eq!((before.epoch, before.revision), (1, 1));
                assert_eq!(before.validity, Validity::Invalid);
                assert_eq!(before.health, Health::Uncertain);
                assert!(matches!(control.try_enter(), Err(Error::Unsupported(_))));
                changed.store(1, Ordering::SeqCst);
                Ok(42)
            })
            .unwrap();
        assert_eq!((value, changed.load(Ordering::SeqCst)), (42, 1));
        assert_eq!(op.status().health, Health::Healthy);
        assert_eq!(op.status().validity, Validity::Invalid);
    }

    #[test]
    fn uncertain_failure_prevents_further_effects_and_entry() {
        let control = Control::new();
        let mut op = control.try_enter().unwrap();
        let before = op.stamp().unwrap();
        let effects = AtomicUsize::new(0);
        let result: Result<()> = op.effect(|_| {
            effects.fetch_add(1, Ordering::SeqCst);
            Err(std::io::Error::other("injected write/flush failure").into())
        });
        assert!(matches!(result, Err(Error::Io(_))));
        assert_eq!(op.status().health, Health::Faulted);
        assert!(matches!(op.check(before), Err(Error::Corrupt(_))));
        assert!(op.stamp().is_err());
        assert!(op.revise().is_err());
        assert!(op.invalidate().is_err());
        assert!(op
            .effect(|_| {
                effects.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
            .is_err());
        assert_eq!(effects.load(Ordering::SeqCst), 1);
        drop(op);
        assert!(matches!(control.try_enter(), Err(Error::Corrupt(_))));
    }

    #[test]
    fn caught_effect_panic_cannot_be_cleared_by_normal_guard_drop() {
        let control = Control::new();
        let mut op = control.try_enter().unwrap();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _: Result<()> = op.effect(|_| panic!("injected effect panic"));
        }));
        assert!(result.is_err());
        assert!(!control.state.is_poisoned());
        assert!(op.stamp().is_err());
        assert!(op.effect(|_| Ok(())).is_err());
        drop(op);
        assert!(matches!(control.try_enter(), Err(Error::Corrupt(_))));
    }

    #[test]
    fn unwind_with_guard_poison_is_permanent() {
        let control = Arc::new(Control::new());
        let shared = control.clone();
        assert!(std::thread::spawn(move || {
            let _op = shared.try_enter().unwrap();
            panic!("injected planning panic");
        })
        .join()
        .is_err());
        assert!(control.state.is_poisoned());
        for _ in 0..2 {
            assert!(matches!(control.try_enter(), Err(Error::Corrupt(_))));
        }
    }

    #[test]
    fn exhausted_versions_fault_before_any_side_effect_without_partial_advance() {
        for (epoch, revision, view) in [
            (u64::MAX, 7, true),
            (7, u64::MAX, true),
            (7, u64::MAX, false),
        ] {
            let control = Control::new();
            {
                let mut state = control.state.lock().unwrap();
                state.epoch = epoch;
                state.revision = revision;
            }
            let mut op = control.try_enter().unwrap();
            let effects = AtomicUsize::new(0);
            let result = if view {
                op.effect(|_| {
                    effects.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                })
            } else {
                op.revise()
            };
            assert!(matches!(result, Err(Error::Unsupported(_))));
            assert_eq!(effects.load(Ordering::SeqCst), 0);
            assert_eq!((op.status().epoch, op.status().revision), (epoch, revision));
            assert_eq!(op.status().health, Health::Faulted);
            drop(op);
            assert!(control.try_enter().is_err());
        }
    }

    #[test]
    fn later_planning_refusal_does_not_restore_an_earlier_view() {
        let control = Control::new();
        let mut op = control.try_enter().unwrap();
        let original = op.stamp().unwrap();
        let effects = AtomicUsize::new(0);
        let refused: Result<()> = (|| {
            op.effect(|_| {
                effects.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })?;
            // A later chunk refuses during planning, outside the effect API.
            Err(Error::NoSpaceLeftOnDevice)
        })();
        assert!(matches!(refused, Err(Error::NoSpaceLeftOnDevice)));
        assert_eq!(effects.load(Ordering::SeqCst), 1);
        assert!(matches!(op.check(original), Err(Error::Stale)));
        let after = op.status();
        drop(op);
        let next = control.try_enter().unwrap();
        assert_eq!(next.status(), after);
        assert_eq!(after.validity, Validity::Invalid);
    }

    #[test]
    fn last_versions_succeed_once_and_then_refuse_before_another_effect() {
        let control = Control::new();
        {
            let mut state = control.state.lock().unwrap();
            state.epoch = u64::MAX - 1;
            state.revision = u64::MAX - 1;
        }
        let mut op = control.try_enter().unwrap();
        let before = op.stamp().unwrap();
        let effects = AtomicUsize::new(0);
        op.effect(|status| {
            assert_eq!((status.epoch, status.revision), (u64::MAX, u64::MAX));
            effects.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .unwrap();
        assert!(matches!(op.check(before), Err(Error::Stale)));
        assert!(op
            .effect(|_| {
                effects.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
            .is_err());
        assert_eq!(effects.load(Ordering::SeqCst), 1);
    }
}
