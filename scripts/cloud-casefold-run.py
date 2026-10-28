#!/usr/bin/env python3
"""Run against the verified core pin without PRoot or changing sibling sources.

The setup script verifies the core archive. A serialized, temporary manifest
selection uses those same bytes; normal completion and exceptions restore the
original manifest and check the lockfile. Do not edit Cargo.toml during this run.
"""
import fcntl
import pathlib
import subprocess
import sys
import tomllib


def main():
    if len(sys.argv) < 2:
        raise SystemExit("usage: cloud-casefold-run.py COMMAND [ARG...]")
    repo = pathlib.Path(__file__).resolve().parent.parent
    setup = repo.parent / ".cloud-setup"
    core = setup / "pinned-core-0.3.0"
    if not (core / "Cargo.toml").is_file():
        raise SystemExit("missing verified core; run scripts/cloud-casefold-setup.sh first")
    with (core / "Cargo.toml").open("rb") as stream:
        if tomllib.load(stream)["package"]["version"] != "0.3.0":
            raise SystemExit("unexpected core version; preserve the casefold pin")
    with (setup / "native-run.lock").open("a") as guard:
        fcntl.flock(guard, fcntl.LOCK_EX)
        manifest, lock = repo / "Cargo.toml", repo / "Cargo.lock"
        original, locked = manifest.read_bytes(), lock.read_bytes()
        needle = b'path = "../rust-fs-core"'
        if original.count(needle) != 1:
            raise SystemExit("unexpected core path; inspect the manifest before continuing")
        selected = original.replace(needle, b'path = "../.cloud-setup/pinned-core-0.3.0"')
        try:
            manifest.write_bytes(selected)
            status = subprocess.run(sys.argv[1:], cwd=repo).returncode
        finally:
            manifest.write_bytes(original)
            if lock.read_bytes() != locked:
                raise RuntimeError("command changed Cargo.lock; inspect it before continuing")
        return status


if __name__ == "__main__":
    sys.exit(main())
