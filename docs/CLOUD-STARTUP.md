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
2. Immediately identify and request required missing access using the available
   permission/environment tools, grouped where possible. Inspect current runtime
   capabilities and existing grants; distinguish already granted from requested
   and blocked access. Reuse existing approvals and injected authentication.
   The Git/API, Actions storage, package/reference destinations and repository
   permissions are listed in `AGENTS.md`. Contents/Actions read and authorized
   casefold checkpoint push access are normally enough; Actions write is needed
   only for an authorized dispatch/rerun. Do not request repository admin or a
   personal token merely because a token variable is absent or `gh` asks to log in.
3. Once access permits it, fetch the latest `casefold-rw`, read that branch's
   `AGENTS.md`, then `docs/extprogress.md` and the next-step references. Refine
   access requests if that task reveals another actual requirement. If fetch is
   blocked, report it; do not call an old handoff the latest remote state.
4. Prepare the documented tools as needed, after their required access works.
   Report the access/handoff status and pause for the user's second message
   requesting the double-check. Do not start the project implementation.

## Second message: verify, report the next step, then wait

When the user asks to double-check access, run fresh checks in this chat:

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

Report each required capability as verified or blocked. Say ready only when all
requirements for the proposed next step have been verified. Give the existing
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
release checksum, and uses signed Debian package metadata for unprivileged
PRoot tools. It targets the existing x86_64 Debian cloud workspace; inspect the
platform before using it elsewhere. It does not change branches, tracked
sources, sibling checkout contents, pins or lockfiles. An absent sibling mount
directory can be created without cloning or modifying another project.

Use `/workspace/.cloud-setup/run-casefold.sh <command>` for subsequent locked
Cargo/chore checks from this checkout. The setup script builds the CLI and runs
the existing unit tier; a zero-test run or merely installing tools is not
validation. It also retains the older main/core-0.3.6 workflow when that dependency
is selected and available, but the casefold handoff remains authoritative.

No local KVM means Linux/e2fsprogs/kernel qualification uses the existing CI VM.
Never mount images or run host e2fsprogs here. Do not alter Windows driver locks
or siblings during cloud setup. Successful setup is distinct from independent
qualification, publication, or access in a fresh regular chat.
