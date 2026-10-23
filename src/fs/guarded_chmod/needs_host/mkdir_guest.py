"""Read driver-emitted mkdir metadata inside the existing Linux harness guest."""
import json
import os
import stat
import sys

root = os.fsencode(sys.argv[1])
# EXPECTED, FREE, BS and COUNT are injected as literals by the Rust test.
for name, expected in EXPECTED:
    s = os.stat(root + name)
    found = dict(ino=s.st_ino, mode=stat.S_IMODE(s.st_mode), uid=s.st_uid,
                 gid=s.st_gid, links=s.st_nlink, size=s.st_size, blocks=s.st_blocks,
                 atime=s.st_atime_ns, mtime=s.st_mtime_ns, ctime=s.st_ctime_ns)
    assert found == expected, (name, found, expected)
    print(json.dumps(dict(name_hex=name.hex(), **found), sort_keys=True))

def listing(path):
    fd = os.open(root + path, os.O_RDONLY | os.O_DIRECTORY | os.O_NOATIME)
    try:
        return {os.fsencode(n) for n in os.listdir(fd)}
    finally:
        os.close(fd)

policy = {b'existingdir', b'regular', b'link-dir', b'link-file', b'dangling', b'e'*255}
if COUNT >= 1:
    policy.add(b'child')
if COUNT >= 2:
    policy.add(b'raw-\xff')
assert listing(b'/policy') == policy
assert listing(b'/names') == ({b'n'*255} if COUNT == 3 else set())
assert listing(b'/peer') == {b'kept'}
for name, _ in EXPECTED:
    if name in [b'/policy/child', b'/policy/raw-\xff', b'/names/' + b'n'*255]:
        assert listing(name) == set()
        s = os.stat(root + name)
        assert stat.S_ISDIR(s.st_mode)
        assert s.st_size == BS and s.st_blocks*512 == BS and s.st_nlink == 2
        parent = b'/names' if name.startswith(b'/names/') else b'/policy'
        assert os.stat(root + name + b'/..').st_ino == os.stat(root + parent).st_ino
with os.fdopen(os.open(root+b'/peer/kept', os.O_RDONLY|os.O_NOATIME),'rb') as f:
    assert f.read() == b'mkdir-reference-peer:' + b'p'*4096
with os.fdopen(os.open(root+b'/policy/regular', os.O_RDONLY|os.O_NOATIME),'rb') as f:
    assert f.read() == b'regular'
assert os.getxattr(root+b'/policy','user.audit') == b'a'*512
assert os.getxattr(root+b'/peer','user.audit') == b'p'*512
space = os.statvfs(root+b'/policy')
assert space.f_frsize == BS
assert (space.f_bfree, space.f_ffree) == FREE
print(json.dumps(dict(event='mkdir-readback', count=COUNT, blocks=space.f_bfree,
                      inodes=space.f_ffree),sort_keys=True))
