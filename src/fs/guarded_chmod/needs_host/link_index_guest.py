"""Ordinary indexed-parent reference, executed only in the harness guest.

FS_IOC_GETFLAGS observes the documented userspace inode flags. Every name is
looked up through a fresh directory descriptor; no driver namespace writes occur.
"""
import errno
import fcntl
import json
import os
import stat
import struct
import sys

root = os.fsencode(sys.argv[1])
mode = sys.argv[2]
parent = root + b"/policy"
payload = b"indexed-link-reference:" + b"d" * 257


def flags(path):
    fd = os.open(path, os.O_RDONLY | os.O_NOATIME)
    try:
        return struct.unpack_from("I", fcntl.ioctl(fd, 0x80086601, bytes(8)))[0]
    finally:
        os.close(fd)


def snapshot(path):
    s = os.stat(path, follow_symlinks=False)
    return dict(ino=s.st_ino, mode=s.st_mode, uid=s.st_uid, gid=s.st_gid,
                links=s.st_nlink, size=s.st_size, blocks=s.st_blocks,
                atime=s.st_atime_ns, mtime=s.st_mtime_ns, ctime=s.st_ctime_ns,
                flags=flags(path))


def inventory():
    fd = os.open(parent, os.O_RDONLY | os.O_DIRECTORY | os.O_NOATIME)
    try:
        names = sorted(os.fsencode(n) for n in os.listdir(fd))
        return [[n.hex(), os.stat(n, dir_fd=fd, follow_symlinks=False).st_ino] for n in names]
    finally:
        os.close(fd)


def name(n):
    result = f"alias-{n:04d}-".encode() + b"n" * 244
    assert len(result) == 255
    return result


def audit():
    rows = inventory()
    source = snapshot(root + b"/source")
    assert rows and all(ino == source["ino"] for _, ino in rows)
    assert source["links"] == len(rows) + 1
    for path in [root + b"/source", root + b"/peer"]:
        with os.fdopen(os.open(path, os.O_RDONLY | os.O_NOATIME), "rb") as f:
            assert f.read() == payload
    for path, value in [(parent, b"p" * 512), (root + b"/source", b"s" * 600),
                        (root + b"/peer", b"e" * 512)]:
        assert os.getxattr(path, "user.audit") == value
    space = os.statvfs(parent)
    return dict(parent=snapshot(parent), source=source, peer=snapshot(root + b"/peer"),
                inventory=rows, free_blocks=space.f_bfree, free_inodes=space.f_ffree)


if mode == "input":
    # Offset the first group's reserved-inode boundary, then allocate the
    # source in its parent's group before moving its initial name to root.
    with open(root + b"/anchor", "wb"):
        pass
    os.mkdir(parent, 0o750)
    for path in [parent + b"/seed", root + b"/peer"]:
        with open(path, "wb") as f:
            f.write(payload)
        os.chmod(path, 0o640)
    os.rename(parent + b"/seed", root + b"/source")
    for path, value in [(parent, b"p" * 512), (root + b"/source", b"s" * 600),
                        (root + b"/peer", b"e" * 512)]:
        os.setxattr(path, "user.audit", value)
    for n in range(128):
        os.link(root + b"/source", parent + b"/" + name(n))
        if flags(parent) & 0x1000:
            break
    else:
        raise AssertionError("bounded Linux input did not become indexed")
    result = audit()
    assert result["parent"]["flags"] & 0x1000
    print(json.dumps(dict(event="indexed-input", **result), sort_keys=True))
elif mode in ["inplace", "step"]:
    before = audit()
    destination = b"inplace" if mode == "inplace" else b"growth-" + name(int(sys.argv[3]))[7:]
    assert len(destination) == (7 if mode == "inplace" else 255)
    os.link(root + b"/source", parent + b"/" + destination)
    after = audit()
    assert after["parent"]["flags"] & 0x1000
    assert after["source"]["links"] == before["source"]["links"] + 1
    assert after["peer"] == before["peer"]
    expected = sorted(before["inventory"] + [[destination.hex(), before["source"]["ino"]]])
    assert after["inventory"] == expected
    assert after["free_inodes"] == before["free_inodes"]
    if mode == "inplace":
        assert after["parent"]["size"] == before["parent"]["size"]
        assert after["free_blocks"] == before["free_blocks"]
    print(json.dumps(dict(event=mode, name_hex=destination.hex(), before=before, after=after), sort_keys=True))
elif mode == "refuse":
    before = audit()
    existing = bytes.fromhex(before["inventory"][0][0])
    for src, dst, expected in [(root + b"/source", parent + b"/" + existing, errno.EEXIST),
                               (root + b"/source", parent + b"/" + b"x" * 256, errno.ENAMETOOLONG),
                               (parent, parent + b"/directory-link", errno.EPERM)]:
        try:
            os.link(src, dst)
        except OSError as e:
            assert e.errno == expected, (e.errno, expected)
        else:
            raise AssertionError("reference refusal unexpectedly succeeded")
        assert audit() == before
    print(json.dumps(dict(event="refuse", unchanged=True, errnos=[errno.EEXIST, errno.ENAMETOOLONG, errno.EPERM], **before), sort_keys=True))
elif mode == "cold":
    result = audit()
    assert result == EXPECTED, (result, EXPECTED)
    print(json.dumps(dict(event="cold", **result), sort_keys=True))
else:
    raise AssertionError(mode)
