"""Populate a mounted fixture inside the harness; no driver implementation code."""
import hashlib
import json
import os
import subprocess


def command(*args):
    return subprocess.check_output(args, stderr=subprocess.STDOUT).decode().strip()


def flags(path):
    return command("lsattr", "-d", os.fsdecode(path)).split()[0]


def create(path, content):
    with open(path, "xb") as stream:
        stream.write(content)


def main():
    assert os.environ.get("FLTH_GUEST") == "1", "requires the harness guest"
    root = os.fsencode(os.environ["MNT"])
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
        assert os.stat(parent + b"/ReadMe").st_ino == os.stat(parent + b"/README").st_ino
        assert os.stat(parent + b"/Caf\xc3\xa9").st_ino == os.stat(parent + b"/CAFE\xcc\x81").st_ino
    for number in range(1000):
        name = f"File_{number:04d}_casefold_payload.txt".encode()
        create(root + b"/fold_indexed/" + name, name + b"\n")
    assert os.stat(root + b"/ordinary/ReadMe").st_ino != os.stat(root + b"/ordinary/README").st_ino
    ordinary = flags(root + b"/ordinary")
    small = flags(root + b"/fold_small")
    indexed = flags(root + b"/fold_indexed")
    assert "F" not in ordinary
    assert "F" in small and "I" not in small, small
    assert "F" in indexed and "I" in indexed, indexed
    entries = []
    for directory in [b"ordinary", b"fold_small", b"fold_indexed"]:
        parent = root + b"/" + directory
        for name in sorted(os.listdir(parent)):
            path = parent + b"/" + name
            with open(path, "rb") as stream:
                digest = hashlib.file_digest(stream, "sha256").hexdigest()
            entries.append({"directory": directory.decode(), "name_hex": name.hex(),
                            "inode": os.stat(path).st_ino, "sha256": digest})
    release = os.uname().release
    profile = {
        "kernel_release": release,
        "kernel_machine": os.uname().machine,
        "packages": command("dpkg-query", "-W", "-f=${binary:Package}=${Version}\n",
                            "e2fsprogs", "libext2fs2", "linux-image-" + release),
        "mke2fs_version": command("mke2fs", "-V"),
        "python_version": command("python3", "--version"),
    }
    assert profile["mke2fs_version"].startswith("mke2fs 1.47.0 "), profile
    print(json.dumps({"profile": profile, "directory_flags": {
        "ordinary": ordinary, "fold_small": small, "fold_indexed": indexed},
        "entries": entries}, sort_keys=True))


if __name__ == "__main__":
    main()
