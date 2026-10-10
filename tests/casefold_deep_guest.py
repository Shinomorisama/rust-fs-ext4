"""Linux creates and independently resolves a directory with a deeper index."""
import os
import subprocess
import sys
from pathlib import Path

assert os.environ.get("FLTH_GUEST") == "1", "requires the harness guest"
phase, count = sys.argv[1], int(sys.argv[2])
assert phase in ["populate", "verify"]
parent = Path(os.environ["MNT"]) / "fold_deep"


def name(number):
    return f"File_{number:05d}_" + "Ab" * 100


if phase == "populate":
    parent.mkdir()
    subprocess.run(["chattr", "+F", str(parent)], check=True)
    (parent / "ReadMe").write_bytes(b"deep ASCII payload\n")
    (parent / "Café").write_bytes(b"deep Unicode payload\n")
    # Hard links grow the directory index without depending on inode capacity.
    for number in range(count):
        os.link(parent / "ReadMe", parent / name(number))
else:
    # The harness remounted between phases, discarding population's dentries.
    assert len(list(parent.iterdir())) == count + 2
    assert os.stat(str(parent) + "/.").st_ino == parent.stat().st_ino
    assert os.stat(str(parent) + "/..").st_ino == parent.parent.stat().st_ino
    for path in [str(parent) + "/./README", str(parent) + "/../fold_deep/README"]:
        assert os.stat(path).st_ino == (parent / "ReadMe").stat().st_ino
    probes = [("ReadMe", "README"), ("Café", "CAFE\u0301")]
    probes += [(name(n), name(n).upper()) for n in [0, 1, count // 3, count // 2, count - 2, count - 1]]
    for stored, alias in probes:
        first, second = parent / stored, parent / alias
        assert first.stat().st_ino == second.stat().st_ino
        expected = b"deep Unicode payload\n" if stored == "Café" else b"deep ASCII payload\n"
        assert first.read_bytes() == second.read_bytes() == expected
        print("\t".join([str(first.stat().st_ino), stored.encode().hex(),
                         alias.encode().hex(), expected.hex()]))
    assert not (parent / "missing-name").exists()
