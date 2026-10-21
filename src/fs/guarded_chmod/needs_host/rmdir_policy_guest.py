"""Reference-only rmdir observations, executed by guest_kernel_write."""
import errno
import json
import os
import sys
import time

root, phase = sys.argv[1:]
parent, names = root + "/policy", root + "/names"
exclusive, shared, peer = [parent + "/" + n for n in ["exclusive", "shared", "peer"]]
boundary = names + "/" + "n" * 255
old_times = (1600000000000000000, 1600000001000000000)
payload = b"rmdir-reference-peer:" + b"p" * 4096


def snapshot(path):
    s = os.stat(path)
    return dict(ino=s.st_ino, links=s.st_nlink, size=s.st_size,
                blocks=s.st_blocks, atime=s.st_atime_ns,
                mtime=s.st_mtime_ns, ctime=s.st_ctime_ns)


def space():
    s = os.statvfs(parent)
    return dict(blocks=s.f_bfree, inodes=s.f_ffree, block_size=s.f_frsize)


def emit(event, **fields):
    print(json.dumps(dict(event=event, **fields), sort_keys=True), flush=True)


def empty(path):
    fd = os.open(path, os.O_RDONLY | os.O_DIRECTORY | os.O_NOATIME)
    try:
        assert os.listdir(fd) == [], path
    finally:
        os.close(fd)


def peer_contents():
    with os.fdopen(os.open(peer + "/kept", os.O_RDONLY | os.O_NOATIME), "rb") as f:
        assert f.read() == payload
    assert os.getxattr(peer, "user.audit") == b"b" * 512


def removed(before_parent, before_target, before_space, path, shared_attribute):
    # All target directory handles from empty() are closed before removal.
    # Legacy inode timestamps have whole-second resolution.
    time.sleep(1.1)
    os.rmdir(path)
    os.sync()
    after_parent, after_space = snapshot(os.path.dirname(path)), space()
    assert not os.path.lexists(path)
    assert after_parent["links"] == before_parent["links"] - 1
    assert after_parent["mtime"] > before_parent["mtime"]
    assert after_parent["ctime"] > before_parent["ctime"]
    assert after_parent["atime"] == before_parent["atime"]
    assert after_space["inodes"] == before_space["inodes"] + 1
    assert before_target["blocks"] * 512 % before_space["block_size"] == 0
    released = before_target["blocks"] * 512 // before_space["block_size"]
    released -= int(shared_attribute)
    assert after_space["blocks"] == before_space["blocks"] + released
    return dict(before_parent=before_parent, after_parent=after_parent,
                before_target=before_target, before_space=before_space,
                after_space=after_space, released_blocks=released)


if phase == "input":
    os.mkdir(parent)
    for path in [exclusive, shared, peer, parent + "/nonempty", names]:
        os.mkdir(path)
    with open(peer + "/kept", "wb") as f:
        f.write(payload)
    with open(parent + "/nonempty/child", "wb") as f:
        f.write(b"keep nonempty")
    with open(parent + "/regular", "wb") as f:
        f.write(b"regular")
    os.symlink("exclusive", parent + "/link")
    os.setxattr(exclusive, "user.audit", b"a" * 512)
    for path in [shared, peer]:
        os.setxattr(path, "user.audit", b"b" * 512)
    os.mkdir(boundary)
    for path in [parent, names, exclusive, shared, peer, boundary]:
        os.utime(path, ns=old_times)
    os.sync()
    for path in [exclusive, shared, boundary]:
        empty(path)
        assert snapshot(path)["links"] == 2
    peer_contents()
    emit("input", parent=snapshot(parent), names=snapshot(names),
         exclusive=snapshot(exclusive), shared=snapshot(shared),
         peer=snapshot(peer), boundary=snapshot(boundary), space=space())
elif phase == "names":
    before_parent, before_peer = snapshot(parent), snapshot(peer)
    fd = os.open(parent, os.O_RDONLY | os.O_DIRECTORY)
    try:
        for label, name in [
            ("name-256", b"n" * 256),
            ("missing", b"missing"),
            ("empty", b""),
            ("regular", b"regular"),
            ("regular-slash", b"regular/"),
            ("symlink", b"link"),
            ("symlink-slash", b"link/"),
            ("nonempty", b"nonempty"),
            ("nonempty-slash", b"nonempty/"),
            ("dot", b"."),
            ("dot-slash", b"./"),
            ("dotdot", b".."),
            ("dotdot-slash", b"../"),
            ("regular-before-long", b"regular/" + b"n" * 256),
            ("missing-before-long", b"missing/" + b"n" * 256),
            ("symlink-before-long", b"link/" + b"n" * 256),
            ("directory-before-long", b"nonempty/" + b"n" * 256),
            ("missing-name-255", b"m" * 255),
            ("regular-dot", b"regular/."),
            ("symlink-dot", b"link/."),
            ("missing-dot", b"missing/."),
            ("dot-before-long", b"./" + b"n" * 256),
            ("dotdot-before-long", b"../" + b"n" * 256),
        ]:
            try:
                os.rmdir(name, dir_fd=fd)
            except OSError as e:
                emit("error", case=label, errno=e.errno,
                     symbol=errno.errorcode[e.errno], name_length=len(name))
                if label == "name-256":
                    assert e.errno == errno.ENAMETOOLONG
            else:
                raise AssertionError((label, "unexpected success"))
            assert snapshot(parent) == before_parent
            assert snapshot(peer) == before_peer
    finally:
        os.close(fd)
    empty(boundary)
    os.sync()
    observation = removed(snapshot(names), snapshot(boundary), space(), boundary, False)
    assert snapshot(parent) == before_parent
    assert snapshot(peer) == before_peer
    peer_contents()
    emit("name-255", parent=snapshot(parent), peer=snapshot(peer), **observation)
elif phase in ["exclusive", "shared"]:
    path = exclusive if phase == "exclusive" else shared
    empty(path)
    before_peer = snapshot(peer)
    peer_contents()
    os.sync()
    observation = removed(snapshot(parent), snapshot(path), space(), path, phase == "shared")
    assert snapshot(peer) == before_peer
    peer_contents()
    if phase == "exclusive":
        empty(shared)
        assert os.getxattr(shared, "user.audit") == b"b" * 512
    else:
        assert not os.path.lexists(exclusive)
    emit(phase, peer=snapshot(peer), **observation)
else:
    raise AssertionError(phase)
