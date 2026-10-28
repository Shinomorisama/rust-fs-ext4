"""Reference-only link/linkat(0) observations inside the prescribed guest."""
import ctypes
import errno
import json
import os
import stat
import sys
import time

root, phase = sys.argv[1:]
root = os.fsencode(root)
parent, names, old, peer = [root + b"/" + n for n in [b"policy", b"names", b"old", b"peer"]]
sources = [old + b"/" + n for n in [b"regular", b"fast", b"slow", b"dangling", b"fifo"]]
payload = b"link-reference-data:" + b"p" * 4096
old_times = (1600000000000000000, 1600000001000000000)
libc = ctypes.CDLL(None, use_errno=True)
libc.link.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
libc.link.restype = ctypes.c_int
libc.linkat.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_int, ctypes.c_char_p, ctypes.c_int]
libc.linkat.restype = ctypes.c_int
AT_FDCWD = -100  # Linux userspace syscall ABI in this Linux-only oracle.


def snapshot(path):
    s = os.stat(path, follow_symlinks=False)
    return dict(ino=s.st_ino, kind=stat.S_IFMT(s.st_mode), mode=stat.S_IMODE(s.st_mode),
                uid=s.st_uid, gid=s.st_gid, links=s.st_nlink, size=s.st_size,
                blocks=s.st_blocks, atime=s.st_atime_ns, mtime=s.st_mtime_ns, ctime=s.st_ctime_ns)


def space():
    s = os.statvfs(parent)
    return dict(blocks=s.f_bfree, inodes=s.f_ffree, block_size=s.f_frsize)


def emit(event, **fields):
    print(json.dumps(dict(event=event, **fields), sort_keys=True), flush=True)


def invoke(form, src, dst, src_fd=AT_FDCWD, dst_fd=AT_FDCWD):
    ctypes.set_errno(0)
    rc = libc.link(src, dst) if form == "link" else libc.linkat(src_fd, src, dst_fd, dst, 0)
    if rc != 0:
        number = ctypes.get_errno()
        raise OSError(number, os.strerror(number))


def contents():
    for path in [sources[0], peer + b"/kept"]:
        with os.fdopen(os.open(path, os.O_RDONLY | os.O_NOATIME), "rb") as f:
            assert f.read() == payload
    assert os.getxattr(sources[0], "user.audit") == b"v" * 600
    assert os.getxattr(parent, "user.audit") == b"a" * 512
    assert os.getxattr(peer, "user.audit") == b"b" * 512


def stable_except_ctime_links(a, b):
    assert {k: v for k, v in a.items() if k not in ["ctime", "links"]} == {
        k: v for k, v in b.items() if k not in ["ctime", "links"]}


def perform(index, name, directory, form, via_fd):
    source = sources[index]
    before_source, before_parent = snapshot(source), snapshot(directory)
    before_old, before_peer = snapshot(old), snapshot(peer)
    others = [snapshot(path) for path in sources]
    os.sync()
    before_space = space()
    time.sleep(1.1)  # 128-byte inode profiles have whole-second timestamps.
    started = time.time_ns()
    if via_fd:
        src_fd = os.open(old, os.O_RDONLY | os.O_DIRECTORY | os.O_NOATIME)
        dst_fd = os.open(directory, os.O_RDONLY | os.O_DIRECTORY | os.O_NOATIME)
        try:
            invoke(form, os.path.basename(source), name, src_fd, dst_fd)
        finally:
            os.close(src_fd)
            os.close(dst_fd)
    else:
        # This intermediate directory symlink must be followed; the final
        # source symlink is intentionally preserved by link and linkat(0).
        invoke(form, root + b"/old-link/" + os.path.basename(source), directory + b"/" + name)
    ended = time.time_ns()
    os.sync()
    after_source, after_parent = snapshot(source), snapshot(directory)
    alias = snapshot(directory + b"/" + name)
    assert alias == after_source
    assert after_source["links"] == before_source["links"] + 1
    assert after_source["ctime"] > before_source["ctime"]
    stable_except_ctime_links(before_source, after_source)
    assert after_parent["mtime"] > before_parent["mtime"]
    assert after_parent["ctime"] > before_parent["ctime"]
    assert {k: v for k, v in after_parent.items() if k not in ["mtime", "ctime"]} == {
        k: v for k, v in before_parent.items() if k not in ["mtime", "ctime"]}
    assert snapshot(old) == before_old and snapshot(peer) == before_peer
    for n, path in enumerate(sources):
        if n != index:
            assert snapshot(path) == others[n]
    assert space() == before_space
    contents()
    emit(phase, source_index=index, form=form, flags=0, via_parent_fds=via_fd,
         name_hex=name.hex(), directory=os.path.basename(directory).decode(),
         before_source=before_source, after_source=after_source, alias=alias,
         before_parent=before_parent, after_parent=after_parent,
         old_parent=before_old, peer=before_peer, before_space=before_space,
         after_space=space(), started=started, ended=ended)


assert os.geteuid() == 0 and os.getegid() == 0, "controlled guest credentials required"
os.umask(0)
if phase == "input":
    for path in [parent, names, old, peer, parent + b"/existingdir"]:
        os.mkdir(path, 0o750)
    for path in [sources[0], peer + b"/kept"]:
        with open(path, "wb") as f:
            f.write(payload)
    with open(parent + b"/existing", "wb") as f:
        f.write(b"existing")
    os.chmod(sources[0], 0o640)
    os.chown(sources[0], 1001, 1002)
    os.setxattr(sources[0], "user.audit", b"v" * 600)
    os.setxattr(parent, "user.audit", b"a" * 512)
    os.setxattr(peer, "user.audit", b"b" * 512)
    os.symlink(b"regular", sources[1])
    os.symlink(b"x" * 300, sources[2])
    os.symlink(b"absent", sources[3])
    os.mkfifo(sources[4], 0o640)
    os.symlink(b"old", root + b"/old-link")
    os.symlink(b"policy", root + b"/parent-link")
    os.symlink(b"existingdir", parent + b"/link-dir")
    for path in [parent, names, old, peer, peer + b"/kept", parent + b"/existingdir", parent + b"/existing"] + sources:
        os.utime(path, ns=old_times, follow_symlinks=False)
    os.sync()
    contents()
    emit(phase, parents=[snapshot(p) for p in [parent, names, old, peer]],
         sources=[snapshot(p) for p in sources], space=space(), uid=0, gid=0, umask=0)
elif phase == "names":
    # Record Linux syscall priorities without rewriting the driver's explicit
    # counted-name, pointer or root-walk API contracts from these observations.
    cases = [
        ("empty-source", b"", b"/policy/new"),
        ("empty-destination", b"/old/regular", b""),
        ("missing-source", b"/missing", b"/policy/new"),
        ("existing-file", b"/old/regular", b"/policy/existing"),
        ("existing-directory", b"/old/regular", b"/policy/existingdir"),
        ("same-source-name", b"/old/regular", b"/old/regular"),
        ("name-256", b"/old/regular", b"/policy/" + b"n" * 256),
        ("missing-source-long-destination", b"/missing", b"/policy/" + b"n" * 256),
        ("directory-source", b"/old", b"/policy/new"),
        ("directory-source-missing-parent", b"/old", b"/missing/new"),
        ("directory-source-long-destination", b"/old", b"/policy/" + b"n" * 256),
        ("regular-source-slash", b"/old/regular/", b"/policy/new"),
        ("fast-symlink-source-slash", b"/old/fast/", b"/policy/new"),
        ("dangling-source-slash", b"/old/dangling/", b"/policy/new"),
        ("directory-source-slash", b"/old/", b"/policy/new"),
        ("new-destination-slash", b"/old/regular", b"/policy/new/"),
        ("existing-directory-slash", b"/old/regular", b"/policy/existingdir/"),
        ("existing-file-slash", b"/old/regular", b"/policy/existing/"),
        ("dot", b"/old/regular", b"/policy/."),
        ("dotdot", b"/old/regular", b"/policy/.."),
        ("root", b"/old/regular", b"/"),
        ("source-dot", b"/old/.", b"/policy/new"),
        ("source-dotdot", b"/old/..", b"/policy/new"),
        ("regular-before-new", b"/old/regular", b"/old/regular/new"),
        ("missing-before-new", b"/old/regular", b"/missing/new"),
        ("regular-before-long", b"/old/regular", b"/old/regular/" + b"n" * 256),
        ("missing-before-long", b"/old/regular", b"/missing/" + b"n" * 256),
        ("symlink-before-long", b"/old/regular", b"/policy/link-dir/" + b"n" * 256),
        ("source-name-256", b"/old/" + b"n" * 256, b"/policy/new"),
        ("both-missing", b"/missing", b"/missing/new"),
        ("missing-source-existing-destination", b"/missing", b"/policy/existing"),
        ("missing-source-dot", b"/missing", b"/policy/."),
        ("directory-source-existing-file", b"/old", b"/policy/existing"),
        ("directory-source-dot", b"/old", b"/policy/."),
        ("symlink-directory-source", b"/parent-link", b"/policy/existing"),
    ]
    before_dirs = [snapshot(p) for p in [parent, names, old, peer]]
    before_sources = [snapshot(p) for p in sources]
    before_space = space()
    for form in ["link", "linkat"]:
        for label, src, dst in cases:
            src = root + src if src else src
            dst = root + dst if dst else dst
            try:
                invoke(form, src, dst)
            except OSError as e:
                emit("error", case=label, form=form, flags=0, errno=e.errno,
                     symbol=errno.errorcode[e.errno], source_hex=src.removeprefix(root).hex(),
                     destination_hex=dst.removeprefix(root).hex())
                if label == "name-256":
                    assert e.errno == errno.ENAMETOOLONG
            else:
                raise AssertionError((form, label, "unexpected successful link"))
            assert [snapshot(p) for p in [parent, names, old, peer]] == before_dirs
            for a, path in zip(before_sources, sources):
                b = snapshot(path)
                assert {k: v for k, v in a.items() if k != "atime"} == {
                    k: v for k, v in b.items() if k != "atime"}
    os.sync()
    assert space() == before_space
    contents()
    emit(phase, cases=len(cases) * 2, parents=before_dirs,
         sources=[snapshot(p) for p in sources], space=space())
elif phase in ["first", "second", "boundary", "fast", "slow", "dangling", "special"]:
    profiles = {
        "first": (0, b"alias", parent, "link", False),
        "second": (0, b"raw-\xff", parent, "linkat", True),
        "boundary": (0, b"n" * 255, names, "linkat", True),
        "fast": (1, b"fast-alias", parent, "link", False),
        "slow": (2, b"slow-alias", parent, "linkat", True),
        "dangling": (3, b"dangling-alias", parent, "linkat", True),
        "special": (4, b"fifo-alias", parent, "linkat", True),
    }
    perform(*profiles[phase])
elif phase == "closed":
    before_dirs = [snapshot(p) for p in [parent, names, old, peer]]
    before_space = space()
    rows = []
    for index, directory, name in [(0, parent, b"alias"), (0, parent, b"raw-\xff"), (0, names, b"n" * 255),
                                   (1, parent, b"fast-alias"), (2, parent, b"slow-alias"),
                                   (3, parent, b"dangling-alias"), (4, parent, b"fifo-alias")]:
        source, alias = snapshot(sources[index]), snapshot(directory + b"/" + name)
        assert source == alias and source["links"] == (4 if index == 0 else 2)
        rows.append(dict(source_index=index, name_hex=name.hex(), directory=os.path.basename(directory).decode(), source=source, alias=alias))
    expected_parent = {b"existing", b"existingdir", b"link-dir", b"alias", b"raw-\xff", b"fast-alias", b"slow-alias", b"dangling-alias", b"fifo-alias"}
    namespace = []
    for directory, expected in [(parent, expected_parent), (names, {b"n" * 255})]:
        fd = os.open(directory, os.O_RDONLY | os.O_DIRECTORY | os.O_NOATIME)
        try:
            actual = {os.fsencode(n) for n in os.listdir(fd)}
        finally:
            os.close(fd)
        assert actual == expected
        namespace.append(sorted(n.hex() for n in actual))
    contents()
    assert [snapshot(p) for p in [parent, names, old, peer]] == before_dirs
    assert space() == before_space
    emit(phase, aliases=rows, namespace_hex=namespace, parents=before_dirs,
         sources=[snapshot(p) for p in sources], space=space())
else:
    raise AssertionError(phase)
