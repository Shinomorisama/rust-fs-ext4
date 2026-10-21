"""Read back driver-emitted unlink images only inside the Linux harness VM."""
import json
import os
import sys

root = sys.argv[1]
stamp, final_link, parent, peer = map(int, sys.argv[2:])
p = root + "/p"
info = os.stat(p)
assert info.st_ino == parent
assert info.st_mtime_ns == info.st_ctime_ns == stamp * 10**9
assert info.st_atime_ns == 1600000000000000000
for name in ["exclusive", "shared", "shared-peer"]:
    if name != "shared-peer":
        assert not os.path.lexists(p + "/" + name)
        if final_link:
            assert not os.path.lexists(p + "/" + name + "-alias")
            continue
    path = p + "/" + name + ("-alias" if name != "shared-peer" else "")
    s = os.stat(path)
    assert s.st_nlink == 1
    assert s.st_atime_ns == 1600000000000000000
    assert s.st_mtime_ns == 1600000001000000000
    if name != "shared-peer":
        assert s.st_ctime_ns == stamp * 10**9
    else:
        assert s.st_ino == peer
    with os.fdopen(os.open(path, os.O_RDONLY | os.O_NOATIME), "rb") as f:
        assert f.read() == name.encode() + b":" + b"x" * 16384
    assert os.getxattr(path, "user.audit") == (b"a" if name == "exclusive" else b"b") * 512
    print(json.dumps(dict(name=name, ino=s.st_ino, links=s.st_nlink,
                         atime=s.st_atime_ns, mtime=s.st_mtime_ns,
                         ctime=s.st_ctime_ns, blocks=s.st_blocks)))
v = os.statvfs(p)
print(json.dumps(dict(parent=parent, stamp=stamp, final_link=final_link,
                     free_blocks=v.f_bfree, free_inodes=v.f_ffree,
                     block_size=v.f_frsize)))
