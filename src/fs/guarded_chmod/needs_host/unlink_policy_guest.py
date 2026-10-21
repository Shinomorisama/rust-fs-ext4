"""Black-box unlink observations, executed only by guest_kernel_write."""
import errno
import json
import os
import sys
import time

root, phase = sys.argv[1:]
parent = root + "/policy"
target, peer = parent + "/target", parent + "/peer"
payload = b"unlink-reference:" + b"x" * 16368
attribute = b"a" * 512
old_times = (1600000000000000000, 1600000001000000000)


def snapshot(path):
    s = os.stat(path)
    return dict(ino=s.st_ino, links=s.st_nlink, size=s.st_size,
                blocks=s.st_blocks, atime=s.st_atime_ns,
                mtime=s.st_mtime_ns, ctime=s.st_ctime_ns)


def space():
    s = os.statvfs(parent)
    return dict(blocks=s.f_bfree, inodes=s.f_ffree, block_size=s.f_frsize)


def contents(path):
    # Reading the control must not introduce an atime change of its own.
    with os.fdopen(os.open(path, os.O_RDONLY | os.O_NOATIME), "rb") as f:
        assert f.read() == payload
    assert os.getxattr(path, "user.audit") == attribute


def emit(event, **fields):
    print(json.dumps(dict(event=event, **fields), sort_keys=True), flush=True)


if phase == "input":
    os.mkdir(parent)
    os.mkdir(parent + "/directory")
    with open(target, "wb") as f:
        f.write(payload)
    os.setxattr(target, "user.audit", attribute)
    os.link(target, peer)
    os.mkdir(root + "/names")
    with open(root + "/names/" + "n" * 255, "wb") as f:
        f.write(b"boundary")
    os.utime(target, ns=old_times)
    os.utime(parent, ns=old_times)
    os.sync()
    contents(target)
    assert snapshot(target) == snapshot(peer)
    assert snapshot(target)["links"] == 2
    emit("input", parent=snapshot(parent), target=snapshot(target), space=space())
elif phase == "surviving":
    # Linux accepts a 255-byte final component, independently of the errors below.
    boundary = root + "/names/" + "n" * 255
    with open(boundary, "rb") as f:
        assert f.read() == b"boundary"
    os.unlink(boundary)
    assert not os.path.lexists(boundary)
    emit("name-255", removed=True)
    before_parent, before_target = snapshot(parent), snapshot(peer)
elif phase != "final":
    raise AssertionError(phase)

if phase == "surviving":
    fd = os.open(parent, os.O_RDONLY | os.O_DIRECTORY)
    try:
        for label, name, expected in [
            ("name-256", b"n" * 256, errno.ENAMETOOLONG),
            ("missing", b"missing", errno.ENOENT),
            ("empty", b"", errno.ENOENT),
            ("directory", b"directory", errno.EISDIR),
            ("regular-slash", b"peer/", errno.ENOTDIR),
        ]:
            try:
                os.unlink(name, dir_fd=fd)
            except OSError as e:
                emit("error", case=label, errno=e.errno, name_length=len(name))
                assert e.errno == expected, (label, e)
            else:
                raise AssertionError((label, "unexpected success"))
            assert snapshot(parent) == before_parent
            assert snapshot(peer) == before_target
    finally:
        os.close(fd)
    contents(peer)
    os.sync()
    before_space = space()
    # Legacy inodes have whole-second timestamps. Cross that resolution even
    # on fast machines, rather than masking a missing update with a fixed clock.
    time.sleep(1.1)
    os.unlink(target)
    os.sync()
    after_parent, after_target, after_space = snapshot(parent), snapshot(peer), space()
    assert not os.path.lexists(target)
    assert after_target["ino"] == before_target["ino"]
    assert (before_target["links"], after_target["links"]) == (2, 1)
    assert after_target["ctime"] > before_target["ctime"]
    for key in ["atime", "mtime", "size", "blocks"]:
        assert after_target[key] == before_target[key], key
    assert after_parent["mtime"] > before_parent["mtime"]
    assert after_parent["ctime"] > before_parent["ctime"]
    assert after_parent["atime"] == before_parent["atime"]
    assert after_space == before_space
    contents(peer)
    emit("surviving", before_parent=before_parent, after_parent=after_parent,
         before_target=before_target, after_target=after_target,
         before_space=before_space, after_space=after_space)
elif phase == "final":
    contents(peer)
    os.sync()
    before_parent, before_target, before_space = snapshot(parent), snapshot(peer), space()
    assert before_target["links"] == 1
    time.sleep(1.1)
    # All regular-file descriptors are closed, so this is final deletion rather
    # than the separate policy for an open, unlinked file.
    os.unlink(peer)
    os.sync()
    after_parent, after_space = snapshot(parent), space()
    assert not os.path.lexists(target) and not os.path.lexists(peer)
    assert after_parent["mtime"] > before_parent["mtime"]
    assert after_parent["ctime"] > before_parent["ctime"]
    assert after_parent["atime"] == before_parent["atime"]
    assert after_space["inodes"] == before_space["inodes"] + 1
    assert before_target["blocks"] * 512 % before_space["block_size"] == 0
    released = before_target["blocks"] * 512 // before_space["block_size"]
    assert after_space["blocks"] == before_space["blocks"] + released
    emit("final", before_parent=before_parent, after_parent=after_parent,
         before_target=before_target, before_space=before_space,
         after_space=after_space, released_blocks=released)
