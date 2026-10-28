"""Create inline inputs and check disposable cold copies, only in the harness guest."""
import json
import os
import stat
import sys

root = os.fsencode(sys.argv[1])
mode = sys.argv[2]
payload = b"inline-link-data:" + b"p" * 4096


def snapshot(name):
    s = os.stat(root + name, follow_symlinks=False)
    return dict(ino=s.st_ino, kind=stat.S_IFMT(s.st_mode), mode=stat.S_IMODE(s.st_mode),
                uid=s.st_uid, gid=s.st_gid, links=s.st_nlink, size=s.st_size,
                blocks=s.st_blocks, atime=s.st_atime_ns, mtime=s.st_mtime_ns, ctime=s.st_ctime_ns)


def listing(name):
    fd = os.open(root + name, os.O_DIRECTORY | os.O_RDONLY | os.O_NOATIME)
    try:
        return {os.fsencode(n) for n in os.listdir(fd)}
    finally:
        os.close(fd)


if mode == "input":
    bs, ipg, continued = map(int, sys.argv[3:6])
    for n in range(4):
        name = f"/shape-{n}".encode()
        os.mkdir(root + name)
        with open(root + name + b"/source", "wb") as f:
            f.write(payload)
        a, b = snapshot(name)["ino"] - 1, snapshot(name + b"/source")["ino"] - 1
        if a // ipg == b // ipg and (a % ipg) * 256 // bs == (b % ipg) * 256 // bs:
            os.rename(root + name, root + b"/policy")
            break
    else:
        raise AssertionError("no same-table source/parent fixture; no fallback")
    os.mkdir(root + b"/peer")
    with open(root + b"/peer/kept", "wb") as f:
        f.write(payload)
    os.chmod(root + b"/policy/source", 0o640)
    os.chown(root + b"/policy/source", 1001, 1002)
    os.setxattr(root + b"/policy", "user.keep", b"k")
    os.setxattr(root + b"/policy", "user.audit", b"a" * 512)
    os.setxattr(root + b"/policy/source", "user.audit", b"v" * 600)
    os.setxattr(root + b"/peer", "user.audit", b"b" * 512)
    assert snapshot(b"/policy")["size"] == 60, "EA setup already changed inline shape"
    fillers = []
    if continued:
        for n in range(32):
            name = f"f{n}".encode()
            os.link(root + b"/policy/source", root + b"/policy/" + name)
            fillers.append(name.hex())
            size = snapshot(b"/policy")["size"]
            assert size < bs, "Linux converted before the intended operation"
            if size > 60:
                break
        else:
            raise AssertionError("Linux supplied no populated continuation")
    # Deterministic input atime/mtime; ctime is measured, not invented.
    for parent, dirs, files in os.walk(root):
        for name in dirs + files:
            os.utime(os.path.join(parent, name), ns=(1600000000000000000, 1600000001000000000), follow_symlinks=False)
    os.sync()
    print(json.dumps(dict(event="inline-input", block_size=bs, continuation=bool(continued),
                          parent=snapshot(b"/policy"), source=snapshot(b"/policy/source"), fillers=fillers), sort_keys=True))
elif mode == "cold":
    # EXPECTED/ATTRS/DIRECTORIES/FILES/FREE/BS/COUNT are retained test literals.
    for name, expected in EXPECTED:
        found = snapshot(name)
        assert found == expected, (name, found, expected)
        print(json.dumps(dict(event="inode", name_hex=name.hex(), **found), sort_keys=True))
    for name, expected in ATTRS:
        found = snapshot(name)
        assert found == expected
        print(json.dumps(dict(event="c-attrs", name_hex=name.hex(), **found), sort_keys=True))
    for name, names in DIRECTORIES:
        assert listing(name) == set(names), (name, listing(name), names)
    for name in FILES:
        with os.fdopen(os.open(root + name, os.O_RDONLY | os.O_NOATIME), "rb") as f:
            assert f.read() == payload, name
    for name, key, value in [(b"/policy", "user.keep", b"k"), (b"/policy", "user.audit", b"a" * 512),
                             (b"/policy/source", "user.audit", b"v" * 600), (b"/peer", "user.audit", b"b" * 512)]:
        assert os.getxattr(root + name, key) == value
    for name in [b"fit", b"n" * 255, b"post"][:COUNT]:
        assert snapshot(b"/policy/" + name) == snapshot(b"/policy/source")
    space = os.statvfs(root)
    assert space.f_frsize == BS and (space.f_bfree, space.f_ffree) == FREE
    print(json.dumps(dict(event="inline-readback", count=COUNT, blocks=space.f_bfree, inodes=space.f_ffree), sort_keys=True))
else:
    raise AssertionError(mode)
