"""Populate a mounted fixture inside the harness; no driver implementation code."""
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path


def command(*args):
    return subprocess.check_output(args, stderr=subprocess.STDOUT).decode().strip()


def flags(path):
    return command("lsattr", "-d", os.fsdecode(path)).split()[0]


def create(path, content):
    with open(path, "xb") as stream:
        stream.write(content)


def populate(root):
    for directory in [b"ordinary", b"fold_small", b"fold_indexed"]:
        os.mkdir(root + b"/" + directory)
    for directory in [b"fold_small", b"fold_indexed"]:
        command("chattr", "+F", os.fsdecode(root + b"/" + directory))
    create(root + b"/ordinary/ReadMe", b"ordinary mixed case\n")
    create(root + b"/ordinary/README", b"ordinary uppercase\n")
    for directory in [b"fold_small", b"fold_indexed"]:
        parent = root + b"/" + directory
        create(parent + b"/ReadMe", b"casefold ASCII payload\n")
        create(parent + b"/Caf\xc3\xa9", b"casefold composed Unicode payload\n")
    for number in range(1000):
        name = f"File_{number:04d}_casefold_payload.txt".encode()
        create(root + b"/fold_indexed/" + name, name + b"\n")


def main():
    assert os.environ.get("FLTH_GUEST") == "1", "requires the harness guest"
    release = os.uname().release
    profile = {
        "kernel_release": release,
        "kernel_machine": os.uname().machine,
        "packages": command("dpkg-query", "-W", "-f=${binary:Package}=${Version}\n",
                            "e2fsprogs", "libext2fs2", "linux-image-" + release),
        "mke2fs_version": command("mke2fs", "-V"),
        "python_version": command("python3", "--version"),
    }
    expected = json.loads(Path("/repo/test-disks/casefold-oracle-profile.json").read_text())
    assert profile == expected, (
        "casefold oracle profile changed; qualify a new profile before updating the pin",
        {"expected": expected, "actual": profile},
    )
    root = os.fsencode(os.environ["MNT"])
    assert len(sys.argv) == 2 and sys.argv[1] in ["populate", "verify"]
    if sys.argv[1] == "populate":
        populate(root)
        return
    assert os.stat(root + b"/ordinary/ReadMe").st_ino != os.stat(root + b"/ordinary/README").st_ino
    ordinary = flags(root + b"/ordinary")
    small = flags(root + b"/fold_small")
    indexed = flags(root + b"/fold_indexed")
    assert "F" not in ordinary
    assert "F" in small and "I" not in small, small
    assert "F" in indexed and "I" in indexed, indexed
    # The harness unmounted and remounted the image between these phases.
    # Check aliases after growth and without the population mount's dentries.
    for directory in [b"fold_small", b"fold_indexed"]:
        parent = root + b"/" + directory
        assert os.stat(parent + b"/.").st_ino == os.stat(parent).st_ino
        assert os.stat(parent + b"/..").st_ino == os.stat(root).st_ino
        for path in [parent + b"/./README", parent + b"/../" + directory + b"/README"]:
            assert os.stat(path).st_ino == os.stat(parent + b"/ReadMe").st_ino
        assert os.stat(parent + b"/ReadMe").st_ino == os.stat(parent + b"/README").st_ino
        assert os.stat(parent + b"/Caf\xc3\xa9").st_ino == os.stat(parent + b"/CAFE\xcc\x81").st_ino
    parent = root + b"/fold_indexed/"
    assert os.stat(parent + b"File_0999_casefold_payload.txt").st_ino == os.stat(
        parent + b"FILE_0999_CASEFOLD_PAYLOAD.TXT").st_ino
    entries = []
    for directory in [b"ordinary", b"fold_small", b"fold_indexed"]:
        parent = root + b"/" + directory
        for name in sorted(os.listdir(parent)):
            path = parent + b"/" + name
            with open(path, "rb") as stream:
                digest = hashlib.file_digest(stream, "sha256").hexdigest()
            entries.append({"directory": directory.decode(), "name_hex": name.hex(),
                            "inode": os.stat(path).st_ino, "sha256": digest})
    print(json.dumps({"profile": profile, "directory_flags": {
        "ordinary": ordinary, "fold_small": small, "fold_indexed": indexed},
        "entries": entries}, sort_keys=True))


if __name__ == "__main__":
    main()
