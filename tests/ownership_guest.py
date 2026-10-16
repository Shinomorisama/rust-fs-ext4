"""Populate complete-owner profiles only inside the harness VM."""

import ctypes
import os

root = os.environ["MNT"]
bs = os.statvfs(root).f_frsize
# Cross both 8 MiB allocation groups, including the journal profiles, without
# filling the 16 MiB image. Keep unwritten allocated storage in the file map.
with open(os.path.join(root, "large"), "wb") as stream:
    os.posix_fallocate(stream.fileno(), 0, 10 * 1024 * 1024)
    stream.write(b"complete ownership\n")
    os.fsync(stream.fileno())
for number in range(40):
    with open(os.path.join(root, f"record-{number}"), "wb"):
        pass
for number in range(0, 40, 2):
    os.unlink(os.path.join(root, f"record-{number}"))
os.mkdir(os.path.join(root, "directory"))
os.symlink("large", os.path.join(root, "short-link"))
os.symlink("l" * 100, os.path.join(root, "long-link"))
os.link(os.path.join(root, "large"), os.path.join(root, "hardlink"))
os.mkfifo(os.path.join(root, "fifo"), 0o600)
libc = ctypes.CDLL(None, use_errno=True)
libc.fallocate.argtypes = [ctypes.c_int, ctypes.c_int, ctypes.c_int64, ctypes.c_int64]
libc.fallocate.restype = ctypes.c_int
fd = os.open(os.path.join(root, "external"), os.O_CREAT | os.O_RDWR, 0o600)
try:
    for number in range(16):
        assert os.pwrite(fd, b"e" * bs, number * 32 * bs) == bs
        os.fsync(fd)
    # KEEP_SIZE adds unwritten storage beyond EOF to the complete ownership map.
    assert libc.fallocate(fd, 1, 1024 * bs, 3 * bs) == 0, ctypes.get_errno()
    os.fsync(fd)
finally:
    os.close(fd)
os.sync()
