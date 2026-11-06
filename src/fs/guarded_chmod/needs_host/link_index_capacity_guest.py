"""Bounded ordinary index-capacity observations, only in the harness guest."""
import fcntl
import json
import os
import stat
import struct
import sys

root = os.fsencode(sys.argv[1])
mode = sys.argv[2]
start = int(sys.argv[3])
count = int(sys.argv[4])
parent = root + b"/policy"
payload = b"indexed-link-reference:" + b"d" * 257
LIMIT = 16384


def name(n):
    assert 0 <= n < LIMIT
    value = f"cap-{n:05d}-".encode() + b"n" * 245
    assert len(value) == 255
    return value


def flags(path):
    fd = os.open(path, os.O_RDONLY | os.O_NOATIME)
    try:
        return struct.unpack_from("I", fcntl.ioctl(fd, 0x80086601, bytes(8)))[0]
    finally:
        os.close(fd)


def snapshot(path):
    s = os.stat(path, follow_symlinks=False)
    return dict(ino=s.st_ino, kind=stat.S_IFMT(s.st_mode), mode=s.st_mode & 0xfff,
                uid=s.st_uid, gid=s.st_gid, links=s.st_nlink, size=s.st_size,
                blocks=s.st_blocks, atime=s.st_atime_ns, mtime=s.st_mtime_ns,
                ctime=s.st_ctime_ns)


def metadata():
    paths = [b"/policy", b"/source", b"/peer"]
    space = os.statvfs(parent)
    return dict(records={p.decode(): snapshot(root + p) for p in paths},
                flags=[flags(root + p) for p in paths],
                free_blocks=space.f_bfree, free_inodes=space.f_ffree)


def audit(total):
    assert 0 < total <= LIMIT
    result = metadata()
    assert bool(result["flags"][0] & 0x1000) == globals().get("EXPECTED_INDEX", True)
    source = result["records"]["/source"]
    assert source["links"] == total + 1
    fd = os.open(parent, os.O_RDONLY | os.O_DIRECTORY | os.O_NOATIME)
    try:
        names = sorted(os.fsencode(n) for n in os.listdir(fd))
        assert names == [name(n) for n in range(total)]
        rows = []
        for entry in names:
            s = os.stat(entry, dir_fd=fd, follow_symlinks=False)
            assert s.st_ino == source["ino"] and stat.S_ISREG(s.st_mode)
            rows.append([entry.hex(), s.st_ino])
    finally:
        os.close(fd)
    for path, value in [(b"/policy", b"p" * 512), (b"/source", b"s" * 600),
                        (b"/peer", b"e" * 512)]:
        assert os.getxattr(root + path, "user.audit") == value
    for path in [b"/source", b"/peer"]:
        with os.fdopen(os.open(root + path, os.O_RDONLY | os.O_NOATIME), "rb") as f:
            assert f.read() == payload
    result["inventory"] = rows
    return result


if mode == "input":
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
        raise AssertionError("bounded input did not become indexed")
    print(json.dumps(dict(event="input", total=n + 1, **audit(n + 1)), sort_keys=True))
elif mode == "fill":
    assert 0 < count <= 256 and 0 < start < LIMIT and start + count <= LIMIT
    before = metadata()
    assert before["records"]["/source"]["links"] == start + 1
    completed = 0
    try:
        for n in range(start, start + count):
            os.link(root + b"/source", parent + b"/" + name(n))
            completed += 1
    except OSError as e:
        print(json.dumps(dict(event="fill-error", start=start, requested=count,
                              completed=completed, errno=e.errno,
                              before=before, after=metadata()), sort_keys=True))
        raise
    after = metadata()
    assert after["records"]["/source"]["links"] == start + count + 1
    print(json.dumps(dict(event="fill", start=start, count=count,
                          before=before, after=after), sort_keys=True))
elif mode in ["audit", "cold"]:
    result = audit(start)
    if mode == "cold":
        for path, wanted in EXPECTED:
            assert snapshot(root + path) == wanted, (path, snapshot(root + path), wanted)
        assert result["flags"] == FLAGS
        assert (result["free_blocks"], result["free_inodes"]) == FREE
    print(json.dumps(dict(event=mode, total=start, **result), sort_keys=True))
else:
    raise AssertionError(mode)
