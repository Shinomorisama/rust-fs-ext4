#!/usr/bin/env python3
"""Verify the frozen Unicode inputs offline: chore check:casefold-unicode.

The manifest records the official archive and individual-file URLs. Reproduce
the inputs by downloading that archive, verifying its SHA-256, and copying only
the listed UCD members verbatim. Keep the separately licensed notice with them.
This checks input identity, not filesystem name-comparison semantics.
"""

import argparse
import hashlib
import json
from pathlib import Path
import sys


VERSION = "12.1.0"
UCD_URL = f"https://www.unicode.org/Public/{VERSION}/ucd/"
FILES = {
    "UnicodeData.txt",
    "CaseFolding.txt",
    "DerivedCoreProperties.txt",
    "DerivedAge.txt",
    "NormalizationTest.txt",
    "ReadMe.txt",
    "LICENSE.txt",
}


def verify(directory):
    manifest = json.loads((directory / "manifest.json").read_text(encoding="utf-8"))
    if manifest["unicode_version"] != VERSION or manifest["license"] != "Unicode-3.0":
        raise ValueError("unexpected Unicode version or license")
    if set(manifest["files"]) != FILES:
        raise ValueError("the manifest does not contain exactly the required inputs")
    if manifest["archive"]["url"] != f"https://www.unicode.org/Public/zipped/{VERSION}/UCD.zip":
        raise ValueError("unexpected archive URL")
    archive_hash = manifest["archive"]["sha256"]
    if len(archive_hash) != 64 or any(c not in "0123456789abcdef" for c in archive_hash):
        raise ValueError("invalid archive SHA-256")

    total = 0
    for name in sorted(FILES):
        entry = manifest["files"][name]
        url = "https://www.unicode.org/license.txt" if name == "LICENSE.txt" else UCD_URL + name
        if entry["url"] != url:
            raise ValueError(f"{name}: unexpected source URL")
        data = (directory / name).read_bytes()
        if len(data) != entry["bytes"] or hashlib.sha256(data).hexdigest() != entry["sha256"]:
            raise ValueError(f"{name}: size or SHA-256 mismatch")
        text = data.decode("utf-8")
        if name == "LICENSE.txt":
            if not text.startswith("UNICODE LICENSE V3\n"):
                raise ValueError("missing Unicode license heading")
        elif name == "ReadMe.txt":
            if f"Version {VERSION} of the Unicode Standard" not in text:
                raise ValueError("ReadMe does not identify the pinned Unicode release")
        elif name != "UnicodeData.txt":
            if text.splitlines()[0] != f"# {name[:-4]}-{VERSION}.txt":
                raise ValueError(f"{name}: wrong Unicode version heading")
        total += len(data)
    return total


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--data-dir",
        type=Path,
        default=Path(__file__).resolve().parents[1] / "data" / "unicode" / VERSION,
    )
    args = parser.parse_args()
    try:
        total = verify(args.data_dir)
    except (OSError, ValueError, KeyError, TypeError, IndexError) as error:
        print(f"casefold-unicode: FAILED: {error}", file=sys.stderr)
        return 1
    print(f"casefold-unicode: {len(FILES)} files verified, Unicode {VERSION}, {total} bytes")
    return 0


if __name__ == "__main__":
    sys.exit(main())
