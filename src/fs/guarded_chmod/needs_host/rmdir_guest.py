"""Create reference inputs or read driver rmdir output inside the Linux VM."""
import json
import os
import sys

root, phase = sys.argv[1:3]
p = root + "/p"
boundary = "n" * 255
payload = b"peer:" + b"x" * 16384


def snapshot(path):
    s = os.stat(path, follow_symlinks=False)
    return dict(ino=s.st_ino, links=s.st_nlink, atime=s.st_atime_ns,
                mtime=s.st_mtime_ns, ctime=s.st_ctime_ns, blocks=s.st_blocks)


def entries(path):
    fd = os.open(path, os.O_RDONLY | os.O_DIRECTORY | os.O_NOATIME)
    try:
        return sorted(os.listdir(fd))
    finally:
        os.close(fd)


if phase == "input":
    os.mkdir(p)
    for name in ["exclusive", "shared", "peer", "nonempty", boundary]:
        os.mkdir(p + "/" + name)
    with open(p + "/peer/kept", "wb") as f:
        f.write(payload)
    with open(p + "/nonempty/child", "wb") as f:
        f.write(b"keep nonempty")
    with open(p + "/regular", "wb") as f:
        f.write(b"regular")
    os.symlink("exclusive", p + "/link")
    os.setxattr(p + "/exclusive", "user.audit", b"a" * 512)
    for name in ["shared", "peer"]:
        os.setxattr(p + "/" + name, "user.audit", b"b" * 512)
    for name in ["", "/exclusive", "/shared", "/peer", "/" + boundary]:
        os.utime(p + name, ns=(1600000000000000000, 1600000001000000000))
    os.sync()
    v = os.statvfs(p)
    print(json.dumps(dict(phase=phase, parent=snapshot(p), peer=snapshot(p + "/peer"),
                         free_blocks=v.f_bfree, free_inodes=v.f_ffree,
                         block_size=v.f_frsize)))
    print("LEDGER", v.f_bfree, v.f_ffree)
else:
    step, stamp, parent, peer, links, blocks, inodes, bs, peer_ctime = map(int, sys.argv[3:])
    assert phase == ["exclusive", "shared", "boundary"][step - 1]
    s = snapshot(p)
    assert s["ino"] == parent
    assert s["links"] == links - step
    assert s["atime"] == 1600000000000000000
    assert s["mtime"] == s["ctime"] == stamp * 10**9
    deleted = ["exclusive", "shared", boundary][:step]
    expected = sorted(set(["exclusive", "shared", "peer", "nonempty", boundary,
                           "regular", "link"]) - set(deleted))
    assert entries(p) == expected
    for name in deleted:
        assert not os.path.lexists(p + "/" + name)
    for name in ["exclusive", "shared", boundary][step:]:
        assert entries(p + "/" + name) == []
    info = snapshot(p + "/peer")
    assert info["ino"] == peer
    assert info["links"] == 2
    assert info["atime"] == 1600000000000000000
    assert info["mtime"] == 1600000001000000000
    assert info["ctime"] == peer_ctime
    assert entries(p + "/peer") == ["kept"]
    with os.fdopen(os.open(p + "/peer/kept", os.O_RDONLY | os.O_NOATIME), "rb") as f:
        assert f.read() == payload
    assert os.getxattr(p + "/peer", "user.audit") == b"b" * 512
    assert entries(p + "/nonempty") == ["child"]
    assert os.readlink(p + "/link") == "exclusive"
    v = os.statvfs(p)
    assert (v.f_bfree, v.f_ffree, v.f_frsize) == (blocks, inodes, bs)
    print(json.dumps(dict(phase=phase, step=step, parent=s, peer=info,
                         names=expected, free_blocks=v.f_bfree,
                         free_inodes=v.f_ffree, block_size=v.f_frsize)))
