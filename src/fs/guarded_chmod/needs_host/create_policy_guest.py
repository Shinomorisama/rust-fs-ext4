"""Reference-only create observations, executed by guest_kernel_write."""
import errno
import json
import os
import stat
import sys
import time

root, phase = sys.argv[1:]
root = os.fsencode(root)
parent, names, peer = [root + b"/" + n for n in [b"policy", b"names", b"peer"]]
boundary = names + b"/" + b"n" * 255
payload = b"create-reference-peer:" + b"p" * 4096
old_times = (1600000000000000000, 1600000001000000000)
create_flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOATIME


def snapshot(path):
    s = os.stat(path)
    return dict(ino=s.st_ino, kind=stat.S_IFMT(s.st_mode), mode=stat.S_IMODE(s.st_mode), uid=s.st_uid,
                gid=s.st_gid, links=s.st_nlink, size=s.st_size,
                blocks=s.st_blocks, atime=s.st_atime_ns,
                mtime=s.st_mtime_ns, ctime=s.st_ctime_ns)


def space():
    s = os.statvfs(parent)
    return dict(blocks=s.f_bfree, inodes=s.f_ffree, block_size=s.f_frsize)


def emit(event, **fields):
    print(json.dumps(dict(event=event, **fields), sort_keys=True), flush=True)


def peer_contents():
    with os.fdopen(os.open(peer + b"/kept", os.O_RDONLY | os.O_NOATIME), "rb") as f:
        assert f.read() == payload
    assert os.getxattr(peer, "user.audit") == b"p" * 512
    assert os.getxattr(parent, "user.audit") == b"a" * 512


def empty(path):
    fd = os.open(path, os.O_RDONLY | os.O_NOATIME)
    try:
        assert stat.S_ISREG(os.fstat(fd).st_mode)
        assert os.read(fd, 1) == b"", path
    finally:
        os.close(fd)


def exclusive(name, mode, dir_fd=None):
    fd = os.open(name, create_flags, mode, dir_fd=dir_fd)
    os.close(fd)


def created(path, requested_mode, via_parent_fd):
    directory = os.path.dirname(path)
    before_parent, before_peer = snapshot(directory), snapshot(peer)
    os.sync()
    before_space = space()
    # Legacy inodes have whole-second timestamps. Do not mistake same-second
    # values for unchanged parent ctime on these profiles.
    time.sleep(1.1)
    started = time.time_ns()
    if via_parent_fd:
        fd = os.open(directory, os.O_RDONLY | os.O_DIRECTORY | os.O_NOATIME)
        try:
            exclusive(os.path.basename(path), requested_mode, dir_fd=fd)
        finally:
            os.close(fd)
    else:
        # Successful creation follows an intermediate parent symlink; trailing
        # slash refusal is measured separately, never assumed from mkdir.
        exclusive(root + b"/parent-link/child", requested_mode)
    ended = time.time_ns()
    os.sync()
    child, after_parent, after_space = snapshot(path), snapshot(directory), space()
    empty(path)
    assert stat.S_ISREG(os.stat(path).st_mode)
    assert child["links"] == 1
    assert child["size"] == 0
    assert child["blocks"] == 0
    assert child["mode"] & 0o777 == requested_mode & 0o777
    # Record special mode bits rather than assuming syscall/API equivalence.
    assert (child["uid"], child["gid"]) == (0, 0)
    # The kernel's coarse wall clock can round across a second boundary.
    # Keep exact nanoseconds in the report; this is only a sanity bound.
    assert started // 1000000000 - 1 <= child["ctime"] // 1000000000 <= ended // 1000000000 + 1
    assert after_parent["links"] == before_parent["links"]
    assert after_parent["mtime"] > before_parent["mtime"]
    assert after_parent["ctime"] > before_parent["ctime"]
    assert after_parent["atime"] == before_parent["atime"]
    assert after_parent["size"] == before_parent["size"]
    assert after_parent["blocks"] == before_parent["blocks"]
    assert after_space["inodes"] == before_space["inodes"] - 1
    assert after_space["blocks"] == before_space["blocks"]
    assert snapshot(peer) == before_peer
    peer_contents()
    return dict(requested_mode=requested_mode, create_flags=create_flags, name_hex=os.path.basename(path).hex(),
                via_parent_fd=via_parent_fd, started=started, ended=ended,
                before_parent=before_parent, after_parent=after_parent,
                child=child, peer=before_peer, before_space=before_space,
                after_space=after_space)


assert os.geteuid() == 0 and os.getegid() == 0, "controlled root credentials required"
os.umask(0)
if phase == "input":
    for path in [parent, names, peer, parent + b"/existingdir"]:
        os.mkdir(path, 0o750)
    exclusive(parent + b"/" + b"e" * 255, 0o750)
    for path, data in [(peer + b"/kept", payload), (parent + b"/regular", b"regular")]:
        with open(path, "wb") as f:
            f.write(data)
    os.symlink(b"existingdir", parent + b"/link-dir")
    os.symlink(b"regular", parent + b"/link-file")
    os.symlink(b"absent", parent + b"/dangling")
    os.symlink(b"policy", root + b"/parent-link")
    os.setxattr(parent, "user.audit", b"a" * 512)
    os.setxattr(peer, "user.audit", b"p" * 512)
    for path in [parent, names]:
        assert not snapshot(path)["mode"] & stat.S_ISGID
        assert "system.posix_acl_default" not in os.listxattr(path)
    for path in [parent, names, peer, peer + b"/kept", parent + b"/existingdir",
                 parent + b"/regular", parent + b"/" + b"e" * 255]:
        os.utime(path, ns=old_times)
    os.sync()
    peer_contents()
    emit("input", parent=snapshot(parent), names=snapshot(names),
         peer=snapshot(peer), space=space(), uid=0, gid=0, umask=0,
         create_flags=create_flags)
elif phase == "names":
    before_parent, before_names, before_peer, before_space = snapshot(parent), snapshot(names), snapshot(peer), space()
    cases = [
        ("name-256", b"n" * 256), ("empty", b""),
        ("existing-directory", b"existingdir"), ("directory-slash", b"existingdir/"),
        ("existing-file", b"regular"), ("file-slash", b"regular/"),
        ("symlink-directory", b"link-dir"), ("symlink-directory-slash", b"link-dir/"),
        ("symlink-file", b"link-file"), ("dangling", b"dangling"),
        ("dot", b"."), ("dot-slash", b"./"),
        ("dotdot", b".."), ("dotdot-slash", b"../"),
        ("regular-before-new", b"regular/new"), ("missing-before-new", b"missing/new"),
        ("regular-before-long", b"regular/" + b"n" * 256),
        ("missing-before-long", b"missing/" + b"n" * 256),
        ("symlink-before-long", b"link-dir/" + b"n" * 256),
        ("directory-before-long", b"existingdir/" + b"n" * 256),
        ("dot-before-long", b"./" + b"n" * 256),
        ("dotdot-before-long", b"../" + b"n" * 256),
        ("regular-dot", b"regular/."), ("missing-dot", b"missing/."),
        ("symlink-dot", b"link-dir/."), ("existing-name-255", b"e" * 255),
        ("symlink-file-slash", b"link-file/"),
    ]
    assert len(cases) == 27
    fd = os.open(parent, os.O_RDONLY | os.O_DIRECTORY | os.O_NOATIME)
    try:
        for label, name in cases:
            try:
                exclusive(name, 0o750, dir_fd=fd)
            except OSError as e:
                emit("error", case=label, errno=e.errno, symbol=errno.errorcode[e.errno],
                     name_hex=name.hex(), name_length=len(name))
                if label == "name-256":
                    assert e.errno == errno.ENAMETOOLONG
            else:
                raise AssertionError((label, "unexpected creation"))
            assert snapshot(parent) == before_parent
            assert snapshot(names) == before_names
            assert snapshot(peer) == before_peer
    finally:
        os.close(fd)
    os.sync()
    assert space() == before_space
    peer_contents()
    emit("names", cases=len(cases), parent=before_parent, names=before_names,
         peer=before_peer, space=before_space)
elif phase == "first":
    emit(phase, **created(parent + b"/child", 0o2750, False))
elif phase == "second":
    emit(phase, **created(parent + b"/raw-\xff", 0o1770, True))
elif phase == "boundary":
    emit(phase, **created(boundary, 0o750, True))
elif phase == "closed":
    # A separate invocation means a fresh cold kernel mount after all three
    # creation phases have closed their handles and unmounted successfully.
    before_parent, before_names, before_peer, before_space = snapshot(parent), snapshot(names), snapshot(peer), space()
    children = []
    for path in [parent + b"/child", parent + b"/raw-\xff", boundary]:
        empty(path)
        child = snapshot(path)
        assert child["links"] == 1 and child["size"] == 0 and child["blocks"] == 0
        children.append(dict(name_hex=os.path.basename(path).hex(), child=child))
    listed = []
    for directory, expected in [
            (parent, {b"existingdir", b"regular", b"e" * 255, b"link-dir",
                      b"link-file", b"dangling", b"child", b"raw-\xff"}),
            (names, {b"n" * 255})]:
        fd = os.open(directory, os.O_RDONLY | os.O_DIRECTORY | os.O_NOATIME)
        try:
            actual = {os.fsencode(n) for n in os.listdir(fd)}
        finally:
            os.close(fd)
        assert actual == expected
        listed.append(sorted(n.hex() for n in actual))
    peer_contents()
    assert snapshot(parent) == before_parent and snapshot(names) == before_names
    assert snapshot(peer) == before_peer and space() == before_space
    emit(phase, children=children, namespace_hex=listed, parent=before_parent, names=before_names,
         peer=before_peer, space=before_space)
else:
    raise AssertionError(phase)
