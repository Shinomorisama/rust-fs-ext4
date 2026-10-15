"""Populate explicit inode-role profiles only inside the harness VM."""

import os

root = os.environ["MNT"]
# Force both 32-slot inode groups to become initialized before freeing slots.
for number in range(40):
    with open(os.path.join(root, f"record-{number}"), "wb") as stream:
        stream.write(b"role payload\n")
for number in range(0, 40, 2):
    os.unlink(os.path.join(root, f"record-{number}"))
os.mkdir(os.path.join(root, "directory"))
os.symlink("record-1", os.path.join(root, "short-link"))
os.symlink("l" * 100, os.path.join(root, "long-link"))
os.link(os.path.join(root, "record-1"), os.path.join(root, "hardlink"))
os.mkfifo(os.path.join(root, "fifo"), 0o600)
with open(os.path.join(root, "empty"), "wb"):
    pass
bs = os.statvfs(root).f_frsize
fd = os.open(os.path.join(root, "external-map"), os.O_CREAT | os.O_RDWR, 0o600)
try:
    for number in range(16):
        os.pwrite(fd, b"e" * bs, number * 32 * bs)
        os.fsync(fd)
finally:
    os.close(fd)
os.sync()
