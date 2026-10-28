# Start a casefold cloud chat

Read this before implementation or long tests. These checks run in the **current
chat**, even when its selected environment was previously tested or published.
Environment-editor access does not establish regular-chat access. After three
failed regular chats, the user requests a separate environment for each new chat
with its own setup/access editing available. Use the product's supported workflow
when available. If this chat cannot create or edit its environment, report that
limitation immediately; repository instructions cannot grant that capability.

## First message: request access, then load the handoff

1. Read the initial `AGENTS.md` and follow its branch pointer to the applicable
   agent guide. Preserve the existing checkout and local work.
2. Immediately identify and request **the entire initial access bundle in
   `AGENTS.md`**, using available permission/environment tools and grouping all
   missing items where possible. Inspect current runtime capabilities and
   existing grants; distinguish already granted from requested
   and blocked access. Reuse existing approvals and injected authentication.
   Include the preset/custom hosts, all listed repository reads, authorized fork
   workflow capabilities, workspace/temp writes and setup-tool execution. Include
   Actions dispatch/rerun and shared claim/PR capabilities in this initial request
   rather than postponing known workflow access until a later stage. Use supported
   session-scoped approvals where available. Do not request repository admin or a
   personal token merely because a token variable is absent or `gh` asks to log in.
3. Once access permits it, fetch the latest `casefold-rw`, read that branch's
   `AGENTS.md`, then `docs/extprogress.md` and the next-step references. Resolve
   any initial inventory discrepancy before marking this first setup complete;
   do not defer a known permission to the second message or implementation.
   If fetch is blocked, report it; do not call an old handoff the latest remote state.
4. Prepare the documented tools and verify **every item in the bundle**, after
   the required access works. Do not label the setup complete with pending grants.
   Report the access/handoff status and pause for the user's second message
   requesting the double-check. Do not start the project implementation.

## Second message: verify, report the next step, then wait

When the user asks to double-check access, freshly recheck the **whole initial
bundle** in this chat. This is verification, not a planned second access request:

- Inspect current runtime status/currentness, connectivity, applied policy and
  configuration/spec versions when the tools expose them. Preserve proxy/CA trust.
- Verify native HTTPS Git read, repository API and reported push permission,
  the actual branch/commit, pinned tools/dependencies and writable build paths.
- Download the relevant exact-commit Actions logs and artifacts, not just their
  metadata or an editor-side URL. Verify complete artifact bytes against GitHub's
  SHA-256 digest before crediting evidence. Keep API authorization off redirected
  storage requests. Never print tokens or signed URLs.
- If an important network command fails under the sandbox, follow the session's
  escalation instructions and retry that specific operation through the supported
  permission flow. Keep proxy inspection and TLS verification. Report any remaining
  failure as blocked; do not claim readiness from the editor's results.

Use non-mutating checks and grant metadata for write authorization; do not create
test issues/PRs, dispatch workflows or push dummy content during verification.
Report each required capability as verified or blocked, distinguishing reported
write authorization from an actual write. Say ready only when the entire initial
bundle has been checked with no unresolved requirement. Give the existing
plain-language project/stage report, identify the next bounded task and its
checks, and **wait for the user's explicit confirmation before implementing it**.
A repository link, an access grant, or a request to double-check is not permission
to implement. A subsequent explicit instruction such as `do next` authorizes the
reported bounded step; do not ask for that same confirmation again.

Keep the `package_managers` preset and the custom destinations already listed
in `AGENTS.md`. Change only a demonstrated missing configuration requirement.
An allowed hostname is a route permission, not a GitHub authorization grant.
The older handoff's request for individual blob host additions is historical:
the editor now succeeds with the configured wildcard. Recheck the affected
chat's actual policy before assuming another allowlist entry is required.

## Sandbox approvals and what the environment can save

Do this inventory during initial setup, not repeatedly at each `do next`.
User authorization, the environment's network destinations, GitHub credentials
and the execution sandbox are different controls. A saved environment or an
`AGENTS.md` instruction does not itself grant sandbox privileges.

| Capability | Upfront handling and observed evidence |
|---|---|
| Session network | Request `network.enabled=true` with `request_permissions` when missing and supported. Verify the returned grant and scope. Afterward use ordinary commands for Git, API requests, downloads and authorized pushes/reruns. Do not request command elevation merely because the operation uses the network. |
| Workspace and temporary files | Read/write/create under `/workspace` and `/tmp` covers this workflow, including Rustup, Cargo, build, log, pinned-tool and cache paths. Verify actual writes and create missing directories under those roots. A nonexistent cache directory is not a denied permission. Do not request unrelated home/system paths. |
| GitHub repository and CI access | Reuse platform HTTPS authentication. Source/API, redirected artifact/log reads, authorized `casefold-rw` pushes and an authorized Actions job rerun have actually worked here. Repository API metadata also reports pull/push access. Preserve the full agent-guide inventory; do not request a new PAT, admin access or duplicate secret for these working operations. A future claim/PR workflow must still verify its own required scope. |
| Native build execution | The committed `scripts/cloud-casefold-run.py` uses the verified immutable core with ordinary unprivileged processes, serializes runner invocations and restores its temporary manifest selection on exit. The setup script generates a native `run-casefold.sh`. PRoot/ptrace is no longer required; do not request that extra capability or reuse an old generated PRoot wrapper. Never change sibling checkouts or dependency pins. |
| Later platforms | Current Linux core work needs no host image mounts/e2fsprogs, local KVM, Docker daemon, Windows administrator or real-card/device access. Independent Linux checks use GitHub's pinned harness. Request separately scoped Windows/card capabilities only when that work is selected. |

On 2026-10-27 the saved configuration and policy file already selected
**unrestricted Internet**. The runtime reported current observations but an
unknown enforcement state, so actual requests were tested. Ordinary commands
still failed to connect to the configured `proxy:8080` until the supported
session network permission was granted. The tool returned session scope; in a
later execution context the same connection failure recurred, and a renewed
grant restored access again. **Grant persistence across contexts is therefore
not established.** This is separate from the historical inspection 403 below.

Reuse grants and try normal required operations first. If a genuine sandbox
failure recurs, follow the governing permission/retry flow once for the affected
capability, then return to normal execution when it works. Record the error,
scope and affected context; do not repeatedly run unchanged escalation requests,
add already-covered hosts, replace credentials or promise that publication fixes
the sandbox. Do not restart the two-message setup because the user said `do next`.

The available environment draft API can save installation/startup instructions,
network destinations and credential/variable requirements. It has **no field
for persistent sandbox auto-approval, session-grant lifetime or ptrace policy**.
Do not invent environment variables or scripts that claim to grant those powers.
Saving startup instructions records what the next chat must request and verify;
it does not itself approve it. If the client cannot retain a needed grant, report
that precise platform limitation rather than claiming all future prompts are solved.

Keep existing network policy, repositories, pins and credential bindings unless
a demonstrated missing requirement calls for a change. Updating startup notes
is a real draft change: review/save/publish it once to apply it to future restored
environments when the product requires publication. Ordinary code checkpoints
do not require republishing the environment. Publication is not evidence that a
sandbox grant will survive another task or execution context.

## Recognize the reported cloud inspection failure

Three regular chats using this selected environment were reported to fail with:

```text
HTTP/1.1 403 Forbidden
server: envoy
Cloudflare egress requires HTTP inspection
```

Affected hosts were `productionresultssa4.blob.core.windows.net`,
`productionresultssa8.blob.core.windows.net` and
`productionresultssa10.blob.core.windows.net`. On 2026-10-20 UTC the environment
editor downloaded SHA-256-verified artifacts from all three, and individual
Actions logs also worked. **Regular-chat recovery was not verified.**

This response identifies an egress inspection refusal. It does not establish
GitHub rate limiting, a missing token, or a repository defect. If the current
chat still receives it after its supported permission retry and with a current
matching allowlist, record the safe error, hostname, timestamp and observed
runtime versions for Codex support. Editor success cannot close that blocker.
Do not cycle secrets, broaden access or ask for another publication without
evidence that the change addresses the failing route.

## Prepare tools after access is established

On the active branch, from `/workspace/rust-fs-ext4`:

```sh
bash scripts/cloud-casefold-setup.sh
```

The script prepares Rust 1.95.0, chore 0.11.0, and the immutable core v0.3.0
source view used by casefold. It verifies the core archive SHA-256 and chore
release checksum and uses the committed native runner. It targets the existing
x86_64 Linux cloud workspace; inspect the platform before using it elsewhere.
It checks basic setup/compiler tools upfront. No PRoot, ptrace, host mount or
sibling checkout is needed for this pinned-core view. The runner temporarily
selects that core in the manifest, restores the original bytes in `finally`
and checks that the lockfile stayed unchanged. Do not edit the manifest or run
Cargo outside the runner concurrently. Branches, source code, sibling contents
and dependency pins remain unchanged.

Use `/workspace/.cloud-setup/run-casefold.sh <command>` for subsequent locked
Cargo/chore checks from this checkout. The setup script builds the CLI and runs
the existing unit tier; a zero-test run or merely installing tools is not
validation. It also retains the older main/core-0.3.6 workflow when that dependency
is selected and available, but the casefold handoff remains authoritative.

The native runner is tracked; a fresh chat must not rely on ignored
`tmp/upstream-forks-oct27/run-direct.py` or cached audit scripts. After setup:

```sh
source /workspace/.cloud-setup/env.sh
python3 scripts/cloud-casefold-run.py chore test:unit
```

Historical logs and audit helpers under `tmp/` are conveniences. Retrieve
source-matched CI artifacts and reconstruct any audit from committed tests,
format documentation and the evidence ledger when those helpers are absent.
Resume the latest `docs/extprogress.md` checkpoint; inspect that exact commit's
focused/full CI before interpreting an old failure or starting a later stage.

On 2026-10-28 the chat UI showed **Full access** while environment status
reported **unrestricted/enforced**, but normal Git/API commands still failed
to connect to `proxy:8080`; approved retries worked and prompted again. This
is an observed runtime/approval mismatch, not evidence of a new missing host.
Request and verify the full known bundle upfront, reuse actual grants, and
report a recurrence accurately. Do not promise that another session approval,
environment publication, repository instruction or fresh environment guarantees
approval persistence. Follow the governing runtime retry rules when blocked.

No local KVM means Linux/e2fsprogs/kernel qualification uses the existing CI VM.
Never mount images or run host e2fsprogs here. Do not alter Windows driver locks
or siblings during cloud setup. Successful setup is distinct from independent
qualification, publication, or access in a fresh regular chat.
