"""Observe byte-name behavior using only filesystem calls in the pinned guest.

No host Unicode normalization or folding is used to predict Linux's answers.
Each image is unmounted between measurement and verification.
"""
import hashlib
import json
import os
import stat
import subprocess
import sys


def flags(path):
    return subprocess.check_output(["lsattr", "-d", path]).split()[0].decode()


def create(path):
    with open(path, "xb") as stream:
        stream.write(b"casefold behavior payload\n")


def outcome(action):
    try:
        action()
        return 0
    except OSError as error:
        assert error.errno is not None
        return error.errno


def identity(first, second):
    try:
        return os.stat(first).st_ino == os.stat(second).st_ino
    except OSError as error:
        # The oracle may reject a name with an error outside the usual missing
        # or invalid-name classes. Preserve that answer instead of predicting it.
        assert error.errno is not None
        return {"errno": error.errno}


def pairs():
    # Explicit bytes/code points are inputs, never host-generated casefold keys.
    text = [
        ("ascii", "ReadMe", "README"),
        ("sharp_s", "Straße", "STRASSE"),
        ("sigma", "Σ", "ς"),
        ("dotted_i", "İ", "i\u0307"),
        ("dotless_i", "ı", "I"),
        ("canonical", "Café", "CAFE\u0301"),
        ("combining_order", "A\u0301\u0323", "a\u0323\u0301"),
        ("hangul", "가", "\u1100\u1161"),
        ("supplementary", "\U00010400", "\U00010428"),
        ("ligature", "ﬁ", "fi"),
        ("width", "Ａ", "a"),
        ("soft_hyphen", "A\u00adB", "ab"),
        ("joiner", "A\u200dB", "ab"),
        ("variation_selector", "A\ufe0fB", "ab"),
        ("post_12_1", "A\U0001fae0", "a\U0001fae0"),
        ("unassigned", "A\u0378", "a\u0378"),
        ("noncharacter", "A\ufdd0", "a\ufdd0"),
        ("private_use", "A\ue000", "a\ue000"),
    ]
    result = [(label, first.encode(), second.encode()) for label, first, second in text]
    for label, invalid in [
        ("invalid_lead", b"\xff"),
        ("invalid_continuation", b"\x80"),
        ("overlong", b"\xc0\x80"),
        ("surrogate", b"\xed\xa0\x80"),
        ("truncated", b"\xe2\x82"),
    ]:
        result.append((label, b"A" + invalid, b"a" + invalid))
    return result


def pair_probe(parent, label, stored, alias, prefix=True):
    head = ("probe_" + label + "_").encode() if prefix else b""
    first, second = parent + b"/" + head + stored, parent + b"/" + head + alias
    first_result = outcome(lambda: create(first))
    lookup_result = outcome(lambda: os.stat(second))
    same_before = identity(first, second)
    alias_result = outcome(lambda: create(second))
    return {
        "label": label, "stored_hex": (head + stored).hex(),
        "alias_hex": (head + alias).hex(), "create_stored": first_result,
        "lookup_alias_before": lookup_result, "same_inode_before": same_before,
        "create_alias": alias_result,
        "lookup_stored_after": outcome(lambda: os.stat(first)),
        "lookup_alias_after": outcome(lambda: os.stat(second)),
        "same_inode_after": identity(first, second),
        "names_hex": [name[len(head):].hex() for name in sorted(os.listdir(parent))
                      if name.startswith(head)],
    }


def namespace_probe(parent):
    observations = {}
    for label, old, new in [
        ("case_rename", b"ReadMe", b"README"),
        ("canonical_rename", "Café".encode(), "CAFE\u0301".encode()),
    ]:
        head = ("probe_" + label + "_").encode()
        first, second = parent + b"/" + head + old, parent + b"/" + head + new
        create(first)
        inode = os.stat(first).st_ino
        result = outcome(lambda: os.rename(first, second))
        observations[label] = {
            "errno": result,
            "destination_is_original_inode": os.stat(second).st_ino == inode,
            "names_hex": [name[len(head):].hex() for name in sorted(os.listdir(parent))
                          if name.startswith(head)],
        }
    first, second = parent + b"/probe_link_one", parent + b"/probe_link_two"
    create(first)
    os.link(first, second)
    observations["rename_distinct_hardlinks"] = {
        "errno": outcome(lambda: os.rename(first, second)),
        "both_names_remain": os.path.exists(first) and os.path.exists(second),
        "same_inode": identity(first, second), "nlink": os.stat(second).st_nlink,
    }
    first, second = parent + b"/probe_unlink_Mixed", parent + b"/probe_unlink_MIXED"
    create(first)
    observations["unlink_alias"] = {
        "errno": outcome(lambda: os.unlink(second)),
        "stored_name_remains": os.path.exists(first),
    }
    child = parent + b"/probe_child"
    os.mkdir(child)
    observations["mkdir_inherits_casefold"] = "F" in flags(child)
    observations["file_has_casefold_flag"] = "F" in flags(parent + b"/probe_link_two")
    return observations


def inventory(root):
    records = []
    for parent, directories, files in os.walk(root):
        for name in sorted(directories + files):
            path = parent + b"/" + name
            relative = os.path.relpath(path, root)
            if relative == b"behavior-evidence.json":
                continue
            info = os.lstat(path)
            record = {"path_hex": relative.hex(), "inode": info.st_ino,
                      "mode": info.st_mode, "nlink": info.st_nlink}
            if stat.S_ISREG(info.st_mode):
                with open(path, "rb") as stream:
                    record["sha256"] = hashlib.file_digest(stream, "sha256").hexdigest()
            elif stat.S_ISDIR(info.st_mode):
                record["flags"] = flags(path)
            else:
                raise AssertionError(("unexpected entry type", record))
            records.append(record)
    return sorted(records, key=lambda record: record["path_hex"])


def measure(root):
    observations = {}
    for shape in ["ordinary", "fold_small", "fold_indexed"]:
        parent = root + b"/" + shape.encode()
        namespace = namespace_probe(parent)
        if shape == "fold_small":
            assert "I" not in flags(parent), "namespace probes outgrew the linear directory"
        probes = []
        for case in pairs():
            destination = parent
            if shape == "fold_small":
                # Each comparison stays in a genuinely non-indexed directory,
                # including with 1 KiB blocks; its parent may grow an index.
                destination += b"/probe_pair_" + case[0].encode()
                os.mkdir(destination)
            probe = pair_probe(destination, *case)
            probe["parent_hex"] = os.path.relpath(destination, root).hex()
            actual_flags = flags(destination)
            if shape == "fold_small":
                assert "F" in actual_flags and "I" not in actual_flags, actual_flags
            elif shape == "fold_indexed":
                assert "F" in actual_flags and "I" in actual_flags, actual_flags
            probes.append(probe)
        observations[shape] = {"pairs": probes, "namespace": namespace}
    # Isolated, small +F directories allow truly empty/dot-like folded keys
    # and exact byte-length boundaries without a case-label prefix.
    special = root + b"/special"
    os.mkdir(special)
    subprocess.check_call(["chattr", "+F", special])
    observations["special"] = []
    for label, first, second in [
        ("all_ignorable", "\u00ad".encode(), "\ufe0f".encode()),
        ("dot_ignorable", "\u00ad.".encode(), b"."),
        ("dotdot_ignorable", "\u00ad..".encode(), b".."),
        ("length_255", b"A" * 255, b"a" * 255),
        ("length_256", b"A" * 256, b"a" * 256),
        ("folded_expansion", "ΐ".encode() * 80, "ι\u0308\u0301".encode() * 80),
    ]:
        parent = special + b"/" + label.encode()
        os.mkdir(parent)
        probe = pair_probe(parent, label, first, second, False)
        probe["parent_hex"] = os.path.relpath(parent, root).hex()
        observations["special"].append(probe)
    # Moving an existing directory must be measured separately from mkdir.
    ordinary = root + b"/ordinary/moved_from_folded"
    folded = root + b"/fold_small/moved_from_ordinary"
    os.rename(root + b"/fold_small/probe_child", ordinary)
    os.rename(root + b"/ordinary/probe_child", folded)
    observations["moved_directory_flags"] = {
        "folded_into_ordinary": "F" in flags(ordinary),
        "ordinary_into_folded": "F" in flags(folded),
    }
    return observations


def cold_aliases(root, observations):
    results = {}
    for shape in ["ordinary", "fold_small", "fold_indexed", "special"]:
        probes = observations[shape] if shape == "special" else observations[shape]["pairs"]
        results[shape] = []
        for probe in probes:
            parent = root + b"/" + bytes.fromhex(probe["parent_hex"])
            first = parent + b"/" + bytes.fromhex(probe["stored_hex"])
            second = parent + b"/" + bytes.fromhex(probe["alias_hex"])
            # Query the alternate spelling before stat/open of the stored name.
            # Directory enumeration plus open would otherwise prime dentries.
            alias = outcome(lambda: os.stat(second))
            stored = outcome(lambda: os.stat(first))
            results[shape].append({"label": probe["label"], "lookup_alias": alias,
                                   "lookup_stored": stored, "same_inode": identity(first, second)})
    return results


def verify_aliases(root, observations):
    for shape in ["ordinary", "fold_small", "fold_indexed", "special"]:
        probes = observations[shape] if shape == "special" else observations[shape]["pairs"]
        for probe in probes:
            parent = root + b"/" + bytes.fromhex(probe["parent_hex"])
            first = parent + b"/" + bytes.fromhex(probe["stored_hex"])
            second = parent + b"/" + bytes.fromhex(probe["alias_hex"])
            assert outcome(lambda: os.stat(first)) == probe["lookup_stored_after"], probe
            assert outcome(lambda: os.stat(second)) == probe["lookup_alias_after"], probe
            assert identity(first, second) == probe["same_inode_after"], probe


def main():
    assert os.environ.get("FLTH_GUEST") == "1", "requires the harness guest"
    assert len(sys.argv) == 2 and sys.argv[1] in ["measure", "verify"]
    root = os.fsencode(os.environ["MNT"])
    evidence = root + b"/behavior-evidence.json"
    if sys.argv[1] == "measure":
        report = {"observations": measure(root), "inventory": inventory(root)}
        with open(evidence, "x") as stream:
            json.dump(report, stream, sort_keys=True)
    else:
        with open(evidence) as stream:
            report = json.load(stream)
        report["cold_lookups"] = cold_aliases(root, report["observations"])
        assert inventory(root) == report["inventory"], "namespace changed across remount"
        verify_aliases(root, report["observations"])
        print(json.dumps(report, sort_keys=True))


if __name__ == "__main__":
    main()
