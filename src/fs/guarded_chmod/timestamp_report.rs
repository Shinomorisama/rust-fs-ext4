//! Observed debugfs timestamp displays; full values are checked by Linux.
type Time = (i64, u32);

pub(super) fn debugfs_time(report: &str, label: &str) -> Time {
    let prefix = format!("{label}:");
    let line = report
        .lines()
        .find(|line| line.trim_start().starts_with(&prefix))
        .unwrap();
    let encoded = line.split_whitespace().nth(1).unwrap();
    let (base, extra) = encoded
        .split_once(':')
        .map_or((encoded, "0"), |(base, extra)| (base, extra));
    let base = u32::from_str_radix(base.trim_start_matches("0x"), 16).unwrap();
    let extra = u32::from_str_radix(extra, 16).unwrap();
    (
        i64::from(base as i32) + (i64::from(extra & 3) << 32),
        extra >> 2,
    )
}
pub(super) fn displayed_time(time: Time, extra: u16) -> Time {
    // The captured extra-isize-12 stat report omits every timestamp extra
    // word, including ctime/mtime words present on disk. Only this observed
    // profile uses the legacy display; Linux must still check full values.
    if extra == 12 {
        (i64::from(time.0 as i32), 0)
    } else {
        time
    }
}

#[test]
fn short_extra_display_matches_captured_vm_report() {
    // Actual first short-extra profile from focused CI run 39078002510.
    let report = "ctime: 0x8000000a -- Fri Dec 13 20:46:02 1901\natime: 0xffffff85 -- Wed Dec 31 23:57:57 1969\nmtime: 0x8000006e -- Fri Dec 13 20:47:42 1901\n";
    for (label, wanted) in [
        ("atime", (-123, 0)),
        ("mtime", (2_147_483_758, 987_654_321)),
        ("ctime", (2_147_483_658, 0)),
    ] {
        assert_eq!(debugfs_time(report, label), displayed_time(wanted, 12));
    }
}

#[test]
fn normal_extra_display_retains_epoch_and_nanoseconds() {
    let report = "atime: 0xffffff85:1d6f3454 -- 1969\nmtime: 0x8000006e:eb79a2c5 -- 2038\nctime: 0x8000000a:00000001 -- 2038\n";
    for (label, wanted) in [
        ("atime", (-123, 123_456_789)),
        ("mtime", (2_147_483_758, 987_654_321)),
        ("ctime", (2_147_483_658, 0)),
    ] {
        assert_eq!(debugfs_time(report, label), displayed_time(wanted, 32));
    }
    // A missing normal extra word cannot pass as a truncated display.
    assert_ne!(
        debugfs_time("mtime: 0x8000006e -- 1901", "mtime"),
        displayed_time((2_147_483_758, 987_654_321), 32)
    );
}

#[test]
fn legacy_and_short_displays_still_check_stored_seconds() {
    let report = "atime: 0xffffff85 -- 1969\nmtime: 0x7fffffff -- 2038\n";
    assert_eq!(debugfs_time(report, "atime"), displayed_time((-123, 0), 0));
    assert_eq!(
        debugfs_time(report, "mtime"),
        displayed_time((2_147_483_647, 0), 0)
    );
    assert_ne!(debugfs_time(report, "atime"), displayed_time((-124, 0), 12));
    assert_ne!(
        debugfs_time(report, "mtime"),
        displayed_time((2_147_483_646, 0), 0)
    );
}
