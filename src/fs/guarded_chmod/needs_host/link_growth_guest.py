"""Fill an ordinary parent using Linux only, within the pinned guest."""
import json
import os
import sys

root = os.fsencode(sys.argv[1])
bs = int(sys.argv[2])
parent = root + b"/policy"
before = os.stat(parent)
assert before.st_size == bs
names = [f"fill-{n:03d}-".encode() + b"f" * 246 for n in range(3 if bs == 1024 else 15)]
for name in names:
    assert len(name) == 255
    os.link(root + b"/peer/kept", parent + b"/" + name)
os.sync()
after = os.stat(parent)
assert after.st_size == bs, (before, after)
assert after.st_blocks == before.st_blocks
assert os.stat(root + b"/peer/kept").st_nlink == 1 + len(names)
print(json.dumps(dict(event="growth-input", block_size=bs, size=after.st_size,
                      blocks=after.st_blocks, names=[n.hex() for n in names]), sort_keys=True))
