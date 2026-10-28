"""Cold Linux checks of copied driver-emitted links, within the prescribed guest."""
import json
import os
import stat
import sys

root = os.fsencode(sys.argv[1])
# EXPECTED / ATTRS / ALIASES / FREE / BS / COUNT / GROW are saved test literals.

def snapshot(name):
    s = os.stat(root + name, follow_symlinks=False)
    return dict(ino=s.st_ino, kind=stat.S_IFMT(s.st_mode), mode=stat.S_IMODE(s.st_mode),
                uid=s.st_uid, gid=s.st_gid, links=s.st_nlink, size=s.st_size,
                blocks=s.st_blocks, atime=s.st_atime_ns, mtime=s.st_mtime_ns, ctime=s.st_ctime_ns)

for name, expected in EXPECTED:
    found = snapshot(name)
    assert found == expected, (name, found, expected)
    print(json.dumps(dict(event="inode", name_hex=name.hex(), **found), sort_keys=True))
for name, expected in ATTRS:
    found = snapshot(name)
    assert found == expected, ("C target attrs", name, found, expected)
    print(json.dumps(dict(event="c-attrs", name_hex=name.hex(), **found), sort_keys=True))
for source, alias in ALIASES:
    assert snapshot(source) == snapshot(alias), (source, alias)

def listing(name):
    fd = os.open(root + name, os.O_RDONLY | os.O_DIRECTORY | os.O_NOATIME)
    try:
        return {os.fsencode(n) for n in os.listdir(fd)}
    finally:
        os.close(fd)

policy = {b"existing", b"existingdir", b"link-dir"}
fillers = [f"fill-{n:03d}-".encode() + b"f" * 246 for n in range(3 if BS == 1024 else 15)] if GROW else []
policy.update(fillers)
for _, alias in ALIASES:
    if alias.startswith(b"/policy/"):
        policy.add(alias.rsplit(b"/", 1)[1])
assert listing(b"/policy") == policy
assert listing(b"/names") == ({b"n" * 255} if COUNT >= 3 else set())
assert listing(b"/old") == {b"regular", b"fast", b"slow", b"dangling", b"fifo"}
assert listing(b"/peer") == {b"kept"}
for name in fillers:
    assert snapshot(b"/policy/" + name) == snapshot(b"/peer/kept")
payload = b"link-reference-data:" + b"p" * 4096
for name in [b"/old/regular", b"/peer/kept"] + [alias for source, alias in ALIASES if source == b"/old/regular"]:
    with os.fdopen(os.open(root + name, os.O_RDONLY | os.O_NOATIME), "rb") as f:
        assert f.read() == payload
    if name != b"/peer/kept":
        assert os.getxattr(root + name, "user.audit") == b"v" * 600
assert os.getxattr(root + b"/policy", "user.audit") == b"a" * 512
assert os.getxattr(root + b"/peer", "user.audit") == b"b" * 512
# Read targets only after all metadata comparisons; reading a symlink can
# update its atime on this disposable Linux copy.
for source, alias in ALIASES:
    if stat.S_ISLNK(snapshot(source)["kind"]):
        wanted = {b"/old/fast": b"regular", b"/old/slow": b"x" * 300,
                  b"/old/dangling": b"absent"}[source]
        assert os.readlink(root + source) == wanted
        assert os.readlink(root + alias) == wanted
space = os.statvfs(root + b"/policy")
assert space.f_frsize == BS
assert (space.f_bfree, space.f_ffree) == FREE
print(json.dumps(dict(event="link-readback", count=COUNT, blocks=space.f_bfree,
                      inodes=space.f_ffree), sort_keys=True))
