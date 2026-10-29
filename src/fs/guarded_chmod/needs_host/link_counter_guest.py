"""Kernel-only counter reference and cold readback, inside the harness guest."""
import errno
import json
import os
import stat
import sys

root = os.fsencode(sys.argv[1])
mode = sys.argv[2]
payload = b"counter-reference-data:" + b"d" * 257


def snapshot(name):
    s = os.stat(root + name, follow_symlinks=False)
    return dict(ino=s.st_ino, kind=stat.S_IFMT(s.st_mode), mode=stat.S_IMODE(s.st_mode),
                uid=s.st_uid, gid=s.st_gid, links=s.st_nlink, size=s.st_size,
                blocks=s.st_blocks, atime=s.st_atime_ns, mtime=s.st_mtime_ns, ctime=s.st_ctime_ns)


if mode == "input":
    os.mkdir(root + b"/policy")
    for name in [b"/policy/source", b"/peer"]:
        with open(root + name, "wb") as f:
            f.write(payload)
        os.chmod(root + name, 0o640)
    os.chmod(root + b"/policy", 0o750)
    os.chown(root + b"/policy/source", 1001, 1002)
    for name, value in [(b"/policy", b"a" * 512), (b"/policy/source", b"v" * 600), (b"/peer", b"b" * 512)]:
        os.setxattr(root + name, "user.audit", value)
        os.utime(root + name, ns=(1600000000000000000, 1600000001000000000))
    baseline = snapshot(b"/policy/source")
    os.link(root + b"/policy/source", root + b"/policy/control")
    linked = snapshot(b"/policy/control")
    assert linked["ino"] == baseline["ino"] and linked["links"] == baseline["links"] + 1
    os.unlink(root + b"/policy/control")
    assert snapshot(b"/policy/source")["links"] == baseline["links"]
    os.utime(root + b"/policy", ns=(1600000000000000000, 1600000001000000000))
    print(json.dumps(dict(event="counter-input", baseline_link=True, baseline_source=baseline, baseline_linked=linked,
                          records=[snapshot(n) for n in [b"/policy", b"/policy/source", b"/peer"]]), sort_keys=True))
elif mode == "refuse":
    before = [snapshot(n) for n in [b"/policy", b"/policy/source", b"/peer"]]
    cases = [(b"/policy/source", b"/policy/source", errno.EEXIST),
             (b"/policy/source", b"/policy/.", errno.EEXIST),
             (b"/policy/source", b"/policy/" + b"x" * 256, errno.ENAMETOOLONG),
             (b"/policy", b"/policy/no-directory-link", errno.EPERM)]
    for source, destination, expected in cases:
        try:
            os.link(root + source, root + destination)
        except OSError as e:
            assert e.errno == expected, (source, destination, e.errno, expected)
            print(json.dumps(dict(event="counter-refusal", errno=e.errno, source_hex=source.hex(), destination_hex=destination.hex()), sort_keys=True))
        else:
            raise AssertionError("a reference refusal unexpectedly created a link")
        assert [snapshot(n) for n in [b"/policy", b"/policy/source", b"/peer"]] == before
elif mode == "link":
    before = [snapshot(n) for n in [b"/policy", b"/policy/source", b"/peer"]]
    try:
        os.link(root + b"/policy/source", root + b"/policy/alias")
    except OSError as e:
        assert not os.path.lexists(root + b"/policy/alias")
        assert [snapshot(n) for n in [b"/policy", b"/policy/source", b"/peer"]] == before
        print("COUNTER_LINK_ERR", e.errno)
        print(json.dumps(dict(event="counter-link", created=False, errno=e.errno), sort_keys=True))
    else:
        assert snapshot(b"/policy/alias")["ino"] == before[1]["ino"]
        assert snapshot(b"/policy/source")["links"] == before[1]["links"] + 1
        print("COUNTER_LINK_OK")
        print(json.dumps(dict(event="counter-link", created=True, source=snapshot(b"/policy/source"), parent=snapshot(b"/policy")), sort_keys=True))
elif mode == "cold":
    for name, expected in EXPECTED:
        found = snapshot(name)
        assert found == expected, (name, found, expected)
        print(json.dumps(dict(event="counter-inode", name_hex=name.hex(), **found), sort_keys=True))
    fd = os.open(root + b"/policy", os.O_RDONLY | os.O_DIRECTORY | os.O_NOATIME)
    try:
        assert set(os.listdir(fd)) == ({"source", "alias"} if CREATED else {"source"})
    finally:
        os.close(fd)
    for name in [b"/policy/source", b"/peer"] + ([b"/policy/alias"] if CREATED else []):
        with os.fdopen(os.open(root + name, os.O_RDONLY | os.O_NOATIME), "rb") as f:
            assert f.read() == payload
    for name, value in [(b"/policy", b"a" * 512), (b"/policy/source", b"v" * 600), (b"/peer", b"b" * 512)]:
        assert os.getxattr(root + name, "user.audit") == value
    space = os.statvfs(root)
    assert space.f_frsize == BS and (space.f_bfree, space.f_ffree) == FREE
    print(json.dumps(dict(event="counter-readback", created=CREATED, blocks=space.f_bfree, inodes=space.f_ffree), sort_keys=True))
else:
    raise AssertionError(mode)
