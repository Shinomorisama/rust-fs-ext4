"""Initialize both allocation groups through Linux, retaining free blocks."""

import os

root = os.environ["MNT"]
# More than one small group's capacity forces initialized block bitmaps in both.
with open(os.path.join(root, "allocated-ranges"), "wb") as stream:
    os.posix_fallocate(stream.fileno(), 0, 1536 * 1024)
    stream.write(b"allocation interval oracle\n")
    os.fsync(stream.fileno())
# Also initialize both inode groups for later component-profile comparisons.
for number in range(40):
    with open(os.path.join(root, f"record-{number}"), "wb"):
        pass
for number in range(0, 40, 2):
    os.unlink(os.path.join(root, f"record-{number}"))
os.sync()
