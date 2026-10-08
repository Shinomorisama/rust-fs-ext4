"""Measure creation order without warming the existing filename first."""
import json
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, "/repo/tests")
import casefold_behavior_guest as base


def create(path, payload):
    with open(path, "xb") as stream:
        stream.write(payload.encode())


def read(path):
    try:
        info = os.stat(path)
        with open(path, "rb") as stream:
            payload = stream.read().decode()
        return {"inode": info.st_ino, "payload": payload, "nlink": info.st_nlink}
    except OSError as error:
        return {"errno": error.errno}


def observe(root, probes, reverse):
    results = []
    for probe in probes:
        parent = root + b"/" + bytes.fromhex(probe["parent_hex"])
        paths = [parent + b"/" + bytes.fromhex(probe[key]) for key in ["first_hex", "second_hex"]]
        # Each pair is independent. Neither member has been accessed on this
        # mount before the requested order of lookups starts.
        values = [None, None]
        for index in ([1, 0] if reverse else [0, 1]):
            values[index] = read(paths[index])
        names = [name.hex() for name in sorted(os.listdir(parent))
                 if name.startswith(bytes.fromhex(probe["prefix_hex"]))]
        results.append({"id": probe["id"], "first": values[0], "second": values[1],
                        "names_hex": names})
    return results


def seed(root):
    probes = []
    pairs = [("ascii", b"A", b"a"),
             ("continuation_tail", b"A\x80", b"a\x80"),
             ("continuation_before_letter", b"\x80A", b"\x80a"),
             ("continuation_vs_plain", b"A\x80", b"A"),
             ("invalid_lead", b"A\xff", b"a\xff")]
    for shape in ["ordinary", "fold_small", "fold_indexed"]:
        for label, left, right in pairs:
            for reverse in [False, True]:
                identity = f"{shape}_{label}_{int(reverse)}"
                parent = root + b"/" + shape.encode()
                prefix = f"cold_{label}_{int(reverse)}_".encode()
                if shape == "fold_small":
                    parent += b"/" + prefix
                    os.mkdir(parent)
                    assert "F" in base.flags(parent) and "I" not in base.flags(parent)
                elif shape == "fold_indexed":
                    assert "F" in base.flags(parent) and "I" in base.flags(parent)
                first, second = (right, left) if reverse else (left, right)
                first, second = prefix + first, prefix + second
                result = base.outcome(lambda: create(parent + b"/" + first, identity + ":first"))
                probes.append({"id": identity, "parent_hex": os.path.relpath(parent, root).hex(),
                               "prefix_hex": prefix.hex(), "first_hex": first.hex(),
                               "second_hex": second.hex(), "create_first": result})
    assert len(probes) == 30
    return {"probes": probes}


def main():
    assert os.environ.get("FLTH_GUEST") == "1", "requires the harness guest"
    assert len(sys.argv) == 2 and sys.argv[1] in ["seed", "create", "first", "second"]
    root = os.fsencode(os.environ["MNT"])
    evidence = root + b"/cold-create-evidence.json"
    phase = sys.argv[1]
    if phase == "seed":
        report = seed(root)
    else:
        with open(evidence) as stream:
            report = json.load(stream)
        if phase == "create":
            for probe in report["probes"]:
                # No stat, enumeration or read of either member before O_EXCL.
                path = root + b"/" + bytes.fromhex(probe["parent_hex"]) + b"/" + bytes.fromhex(probe["second_hex"])
                probe["create_second"] = base.outcome(lambda: create(path, probe["id"] + ":second"))
            report["after_create"] = observe(root, report["probes"], False)
        else:
            report[phase + "_lookup_first"] = observe(root, report["probes"], phase == "second")
    with open(evidence, "w") as stream:
        json.dump(report, stream, sort_keys=True)
    if phase == "second":
        print(json.dumps(report, sort_keys=True))


if __name__ == "__main__":
    main()
