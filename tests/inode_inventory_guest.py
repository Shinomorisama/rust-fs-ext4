"""Create and remove names in the harness VM for allocation inventory checks."""

import os

root = os.environ["MNT"]
# Fill the first group's 32 inode slots so both groups are initialized by Linux.
for number in range(40):
    with open(os.path.join(root, f"record-{number}"), "wb") as stream:
        stream.write(b"inventory payload\n")
for number in range(0, 40, 2):
    os.unlink(os.path.join(root, f"record-{number}"))
os.mkdir(os.path.join(root, "directory"))
os.symlink("record-1", os.path.join(root, "symlink"))
os.link(os.path.join(root, "record-1"), os.path.join(root, "hardlink"))
os.sync()
