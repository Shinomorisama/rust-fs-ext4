#!/usr/bin/env python3
"""Check emitted Rust data against the pinned UCD and its NFD conformance rows.

The small NFD interpreter is test-only. Its Hangul arithmetic and canonical
ordering are from Unicode 12, sections 3.11 and 3.12, not an OS implementation:
https://www.unicode.org/versions/Unicode12.0.0/ch03.pdf
It makes no claim about filesystem comparison or malformed UTF-8 semantics.
"""

import ast
from functools import lru_cache
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


sys.dont_write_bytecode = True
SCRIPT = Path(__file__).resolve().with_name("generate-casefold-unicode.py")
spec = importlib.util.spec_from_file_location("unicode_generator", SCRIPT)
generator = importlib.util.module_from_spec(spec)
spec.loader.exec_module(generator)


def emitted_tables(data):
    """Read the emitted literals, so assertions exercise the serialization too."""
    tables = {}
    current = None
    for line in data.decode("utf-8").splitlines():
        if line.startswith("pub static "):
            current = line.split()[2].rstrip(":")
            tables[current] = []
        elif line.startswith("    ("):
            tables[current].append(ast.literal_eval(line.strip().rstrip(",").replace("&[", "[")))
    return tables


class UnicodeTables(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.generated = generator.render(generator.DATA)
        cls.tables = emitted_tables(cls.generated)
        cls.decomposition = dict(cls.tables["CANONICAL_DECOMPOSITIONS"])
        cls.combining = dict(cls.tables["COMBINING_CLASSES"])
        cls.folding = dict(cls.tables["FULL_CASE_FOLDING"])

    def test_reproducible_checked_in_output(self):
        self.assertEqual(self.generated, generator.OUTPUT.read_bytes())
        self.assertEqual(self.generated, generator.render(generator.DATA))
        self.assertNotIn(b"\r", self.generated)

    def test_emitted_tables_are_sorted_unique_and_bounded(self):
        self.assertEqual(set(self.tables), {
            "CANONICAL_DECOMPOSITIONS", "COMBINING_CLASSES", "FULL_CASE_FOLDING",
            "DEFAULT_IGNORABLE_RANGES", "AGE_RANGES",
        })
        for name, rows in self.tables.items():
            keys = [row[0] for row in rows]
            self.assertEqual(keys, sorted(set(keys)), name)
            for row in rows:
                self.assertTrue(0 <= row[0] <= 0x10FFFF)
            if name.endswith("RANGES"):
                for first, last, *_ in rows:
                    self.assertTrue(first <= last <= 0x10FFFF)
                self.assertTrue(all(a[1] < b[0] for a, b in zip(rows, rows[1:])))

    def test_every_explicit_decomposition_and_combining_class(self):
        seen_dm, seen_ccc = set(), set()
        for line in (generator.DATA / "UnicodeData.txt").read_text(encoding="utf-8").splitlines():
            fields = line.split(";")
            cp, value = int(fields[0], 16), fields[5]
            self.assertEqual(self.combining.get(cp, 0), int(fields[3]), fields[0])
            if int(fields[3]):
                seen_ccc.add(cp)
            if value and value[0] != "<":
                self.assertEqual(self.decomposition[cp], [int(x, 16) for x in value.split()])
                seen_dm.add(cp)
            else:
                self.assertNotIn(cp, self.decomposition)
        self.assertEqual(seen_dm, set(self.decomposition))
        self.assertEqual(seen_ccc, set(self.combining))

    def test_every_full_case_mapping(self):
        expected = {}
        for line in (generator.DATA / "CaseFolding.txt").read_text(encoding="utf-8").splitlines():
            content = line.split("#", 1)[0].strip()
            if content:
                cp, status, values, _ = (field.strip() for field in content.split(";"))
                if status in ("C", "F"):
                    expected[int(cp, 16)] = [int(x, 16) for x in values.split()]
        self.assertEqual(self.folding, expected)
        self.assertEqual(self.folding[0x00DF], [0x73, 0x73])
        self.assertEqual(self.folding[0x0130], [0x69, 0x0307])
        self.assertEqual(self.folding[0x0049], [0x69])
        self.assertEqual(self.folding[0x1E9E], [0x73, 0x73])  # Full, not simple ß.
        self.assertNotIn(0x0131, self.folding)  # Dotless i stays distinct.

    def test_properties_and_version_boundaries(self):
        for filename, table, property_name in [
            ("DerivedCoreProperties.txt", "DEFAULT_IGNORABLE_RANGES", "Default_Ignorable_Code_Point"),
            ("DerivedAge.txt", "AGE_RANGES", None),
        ]:
            expected = []
            for line in (generator.DATA / filename).read_text(encoding="utf-8").splitlines():
                content = line.split("#", 1)[0].strip()
                if not content:
                    continue
                span, value = (part.strip() for part in content.split(";"))
                if property_name is not None and value != property_name:
                    continue
                endpoints = span.split("..")
                row = (int(endpoints[0], 16), int(endpoints[-1], 16))
                if property_name is None:
                    row += tuple(int(x) for x in value.split("."))
                expected.append(row)
            self.assertEqual(self.tables[table], sorted(expected))
        def age(cp):
            return next((tuple(row[2:]) for row in self.tables["AGE_RANGES"] if row[0] <= cp <= row[1]), None)
        self.assertEqual(age(0x32FF), (12, 1))  # Reiwa era square.
        self.assertIsNone(age(0x1FAE0))  # Melting face, added after 12.1.
        self.assertNotIn(0x1FAE0, self.decomposition)
        self.assertNotIn(0x1FAE0, self.folding)

    def test_official_nfd_conformance_rows(self):
        @lru_cache(maxsize=4096)
        def expand(cp):
            if 0xAC00 <= cp <= 0xD7A3:
                index = cp - 0xAC00
                parts = (0x1100 + index // 588, 0x1161 + (index % 588) // 28)
                return parts + ((0x11A7 + index % 28,) if index % 28 else ())
            if cp in self.decomposition:
                return tuple(y for x in self.decomposition[cp] for y in expand(x))
            return (cp,)

        def nfd(sequence):
            result = []
            for cp in sequence:
                for value in expand(cp):
                    result.append(value)
                    ccc = self.combining.get(value, 0)
                    if ccc:
                        pos = len(result) - 1
                        while pos and self.combining.get(result[pos - 1], 0) > ccc:
                            result[pos], result[pos - 1] = result[pos - 1], result[pos]
                            pos -= 1
            return result

        count, part, part_one = 0, None, set()
        for line in (generator.DATA / "NormalizationTest.txt").read_text(encoding="utf-8").splitlines():
            content = line.split("#", 1)[0].strip()
            if content.startswith("@"):
                part = content
            elif content:
                columns = [[int(x, 16) for x in field.split()] for field in content.split(";")[:5]]
                for i, sequence in enumerate(columns):
                    self.assertEqual(nfd(sequence), columns[2 if i < 3 else 4], f"row {count}, column {i + 1}")
                if part == "@Part1":
                    part_one.update(columns[0])
                count += 1
        # The pinned suite must actually be exercised, including Hangul.
        self.assertEqual(count, 18820)
        self.assertTrue(set(range(0xAC00, 0xD7A4)) <= part_one)
        # Every scalar with a nonidentity decomposition is explicitly covered.
        self.assertTrue(set(self.decomposition) <= part_one)

    def test_cli_refuses_stale_output_and_corrupted_inputs_without_writing(self):
        with tempfile.TemporaryDirectory() as scratch:
            directory = Path(scratch)
            data, output = directory / "data", directory / "tables.rs"
            shutil.copytree(generator.DATA, data)
            command = [sys.executable, str(SCRIPT), "--data-dir", str(data), "--output", str(output)]
            def run(extra=()):
                return subprocess.run(command + list(extra), capture_output=True, text=True)
            self.assertEqual(run().returncode, 0)
            self.assertEqual(output.read_bytes(), self.generated)
            self.assertEqual(run(["--check"]).returncode, 0)
            # Manifest formatting and platform line endings cannot change tables.
            manifest = data / "manifest.json"
            value = json.loads(manifest.read_text(encoding="utf-8"))
            manifest.write_bytes(json.dumps(value, indent=4).replace("\n", "\r\n").encode())
            self.assertEqual(run().returncode, 0)
            self.assertEqual(output.read_bytes(), self.generated)
            output.unlink()
            self.assertNotEqual(run(["--check"]).returncode, 0)
            self.assertFalse(output.exists())
            output.write_bytes(b"stale\n")
            result = run(["--check"])
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("stale tables", result.stderr)
            self.assertEqual(output.read_bytes(), b"stale\n")
            (data / "CaseFolding.txt").write_bytes(b"corrupted")
            result = run()
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("SHA-256 mismatch", result.stderr)
            self.assertEqual(output.read_bytes(), b"stale\n")

    def test_generated_rust_compiles(self):
        with tempfile.TemporaryDirectory() as scratch:
            result = subprocess.run([
                "rustc", "--edition=2021", "--crate-type=lib", "--deny=warnings",
                str(generator.OUTPUT), "-o", str(Path(scratch) / "tables.rlib"),
            ], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)


if __name__ == "__main__":
    result = unittest.main(exit=False).result
    if not result.wasSuccessful() or result.skipped or result.expectedFailures or not result.testsRun:
        sys.exit(1)
    # Use the existing test-floor protocol, reporting actual unittest results.
    print(f"test result: ok. {result.testsRun} passed; 0 failed; 0 ignored;")
