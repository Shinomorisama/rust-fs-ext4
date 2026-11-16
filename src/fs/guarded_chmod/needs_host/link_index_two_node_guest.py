"""Explicit namespace protocol for two-node support; filesystem calls stay in the guest."""
import fcntl
import json
import os
import stat
import struct
import sys

LIMIT = 16384
PAYLOAD = b"indexed-link-reference:" + b"d" * 257


def component(value):
    assert isinstance(value, str) and len(value) % 2 == 0
    name = bytes.fromhex(value)
    assert name.hex() == value
    assert 0 < len(name) <= 255 and b"/" not in name and b"\0" not in name
    assert name not in [b".", b".."]
    return name


def namespace(values):
    assert isinstance(values, list) and 0 < len(values) <= LIMIT
    names = [component(value) for value in values]
    assert names == sorted(set(names)), "explicit namespace must be sorted and unique"
    return names


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


def audit(root, names, source):
    parent = root + b"/policy"
    paths = [b"/policy", b"/source", b"/peer"]
    space = os.statvfs(parent)
    records = {p.decode(): snapshot(root + p) for p in paths}
    result = dict(records=records, flags=[flags(root + p) for p in paths],
                  free_blocks=space.f_bfree, free_inodes=space.f_ffree)
    assert result["flags"][0] & 0x1000
    assert result["flags"][0] & (0x40000000 | 0x800) == 0
    assert records["/source"]["ino"] == source
    assert records["/source"]["kind"] == stat.S_IFREG
    assert records["/source"]["links"] == len(names) + 1
    fd = os.open(parent, os.O_RDONLY | os.O_DIRECTORY | os.O_NOATIME)
    try:
        actual = sorted(os.fsencode(n) for n in os.listdir(fd))
        assert actual == names, "explicit namespace differs"
        rows = []
        for name in actual:
            s = os.stat(name, dir_fd=fd, follow_symlinks=False)
            assert s.st_ino == source and stat.S_ISREG(s.st_mode)
            rows.append([name.hex(), s.st_ino])
    finally:
        os.close(fd)
    for path, value in [(b"/policy", b"p" * 512), (b"/source", b"s" * 600),
                        (b"/peer", b"e" * 512)]:
        assert os.getxattr(root + path, "user.audit") == value
    for path in [b"/source", b"/peer"]:
        with os.fdopen(os.open(root + path, os.O_RDONLY | os.O_NOATIME), "rb") as f:
            assert f.read() == PAYLOAD
    result["inventory"] = rows
    return result


def main(root, mode, values, query_hex, source):
    assert mode in ["audit", "link", "cold"]
    names = namespace(values)
    query = component(query_hex)
    assert len(query) == 255 and type(source) is int and source > 0
    if mode == "link":
        assert query not in names and len(names) < LIMIT
        before = audit(root, names, source)
        try:
            os.link(root + b"/source", root + b"/policy/" + query)
        except OSError as error:
            print(json.dumps(dict(event="link-error", query_hex=query_hex,
                                  errno=error.errno, before=before), sort_keys=True))
            raise
        after = audit(root, sorted(names + [query]), source)
        print(json.dumps(dict(event=mode, query_hex=query_hex, before=before,
                              after=after), sort_keys=True))
    else:
        result = audit(root, names, source)
        if mode == "cold":
            for path, wanted in EXPECTED:
                assert snapshot(root + path) == wanted
            assert result["flags"] == FLAGS
            assert (result["free_blocks"], result["free_inodes"]) == FREE
        print(json.dumps(dict(event=mode, query_hex=query_hex, **result), sort_keys=True))


if __name__ == "__main__":
    main(os.fsencode(sys.argv[1]), sys.argv[2], NAMES, QUERY_HEX, SOURCE)
