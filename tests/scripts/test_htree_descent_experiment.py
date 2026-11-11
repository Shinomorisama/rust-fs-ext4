"""Controls for finite scheduling, frozen inputs and independent-report failures."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock
import sys
from types import SimpleNamespace

sys.dont_write_bytecode = True

SPEC = importlib.util.spec_from_file_location(
    "experiment", Path(__file__).resolve().parents[2] / "scripts/htree-descent-experiment.py")
experiment = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(experiment)


class ExperimentControls(unittest.TestCase):
    def test_fixed_manifest_has_exact_unique_identity_and_endpoints(self):
        candidates = experiment.candidates()
        self.assertEqual([row["id"] for row in candidates], [f"{i:02d}" for i in range(32)])
        self.assertEqual(candidates[0]["seed"], "ef274b78-3e6f-9950-4f60-7095197e643e")
        self.assertEqual(candidates[31]["uuid"], "61261780-8762-408e-6ade-2e77190aae8d")
        self.assertEqual(len({row["seed_hex"] for row in candidates}), 32)

    def test_schedule_has_one_replay_and_all_candidates_once_in_order(self):
        schedule = sum((experiment.trial_ids(batch) for batch in range(4)), [])
        self.assertEqual(schedule, ["replay"] + [f"{i:02d}" for i in range(32)])
        self.assertEqual([len(experiment.trial_ids(i)) for i in range(4)], [9, 8, 8, 8])
        for invalid in (-1, 4, 33):
            with self.assertRaises(ValueError):
                experiment.trial_ids(invalid)

    def test_zip_members_cannot_escape_or_use_platform_separators(self):
        root = Path("root")
        self.assertEqual(experiment.member_path(root, "batch-0/trials/replay/result.json"),
                         root / "batch-0/trials/replay/result.json")
        for name in ("/outside", "../outside", "nested/../../outside", "nested\\outside"):
            with self.assertRaises(ValueError):
                experiment.member_path(root, name)

    def test_digest_hashes_actual_bytes(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "bytes"
            path.write_bytes(b"abc")
            self.assertEqual(experiment.digest(path), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")

    def test_missing_manifest_and_edited_candidate_list_stop_before_execution(self):
        with tempfile.TemporaryDirectory() as temporary, mock.patch.object(experiment, "DATA", Path(temporary)), mock.patch.object(experiment.subprocess, "run") as run:
            with self.assertRaises(FileNotFoundError):
                experiment.run_batch(0)
            manifest = Path(temporary) / "manifest.json"
            manifest.write_text(json.dumps({"candidates": [], "corpus_sha256": experiment.CORPUS_SHA}))
            (Path(temporary) / "manifest.sha256").write_text(experiment.digest(manifest))
            with self.assertRaisesRegex(RuntimeError, "candidate list"):
                experiment.run_batch(0)
            run.assert_not_called()

    def test_changed_manifest_digest_stops_before_execution(self):
        with tempfile.TemporaryDirectory() as temporary, mock.patch.object(experiment, "DATA", Path(temporary)), mock.patch.object(experiment.subprocess, "run") as run:
            (Path(temporary) / "manifest.json").write_text("{}")
            (Path(temporary) / "manifest.sha256").write_text("0" * 64)
            with self.assertRaisesRegex(RuntimeError, "manifest changed"):
                experiment.run_batch(0)
            run.assert_not_called()

    def test_missing_authentication_does_not_attempt_download(self):
        with mock.patch.dict(experiment.os.environ, {}, clear=True):
            with self.assertRaisesRegex(RuntimeError, "GH_TOKEN"):
                experiment.api("actions/artifacts/1")

    def test_existing_attempt_is_preserved(self):
        with tempfile.TemporaryDirectory() as temporary, mock.patch.object(experiment, "DATA", Path(temporary)):
            path = Path(temporary) / "manifest.json"
            path.write_text("earlier attempt")
            with self.assertRaisesRegex(RuntimeError, "already exists"):
                experiment.prepare()
            self.assertEqual(path.read_text(), "earlier attempt")

    def test_incomplete_profile_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "names.hex").write_text("61\n")
            (root / "queries.jsonl").write_text('{"name_hex":"61"}\n')
            (root / "filesystem.json").write_text("{}")
            with self.assertRaisesRegex(RuntimeError, "completeness"):
                experiment.audit_profile(root, {})

    def test_first_child_failure_or_missing_capture_stops_and_preserves_exit(self):
        for child_status in (0, 101):
            with self.subTest(exit=child_status), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                data = root / "data"
                data.mkdir()
                (data / "replay").mkdir()
                (data / "replay/filesystem.json").write_text(json.dumps({"seed_hex": "seed", "uuid_hex": "uuid"}))
                manifest = data / "manifest.json"
                manifest.write_text(json.dumps({"source": "source", "candidates": experiment.candidates(), "corpus_sha256": experiment.CORPUS_SHA}))
                (data / "manifest.sha256").write_text(experiment.digest(manifest))
                with mock.patch.object(experiment, "ROOT", root), mock.patch.object(experiment, "DATA", data), mock.patch.object(experiment.subprocess, "check_output", return_value="source\n"), mock.patch.object(experiment.subprocess, "run", return_value=SimpleNamespace(returncode=child_status)) as run, mock.patch.object(experiment.shutil, "disk_usage", return_value=SimpleNamespace(free=100 * 1024**3)):
                    with self.assertRaises(SystemExit) as stopped:
                        experiment.run_batch(0)
                    self.assertEqual(stopped.exception.code, child_status or 1)
                    self.assertEqual(run.call_count, 1)
                    record = json.loads((data / "batch-0/replay.json").read_text())
                    self.assertEqual((record["status"], record["exit"]), ("stopped", child_status))
                    self.assertFalse((data / "batch-0/00.json").exists())

    def test_independent_answers_reject_changed_hash_name_and_membership(self):
        query = {"name_hex": "ff80", "found": True, "selected": 1, "actual": [1], "major": 0x1234, "minor": 0xabcd}
        requests = [b'dx_hash -h 1 -s seed -- "\xff\x80"']
        experiment.check_answers(["ff80"], [query], requests, [(b"1234", b"abcd")])
        for change in ({"major": 0}, {"name_hex": "ff81"}, {"found": False}, {"actual": [2]}):
            with self.assertRaises(RuntimeError):
                experiment.check_answers(["ff80"], [query | change], requests, [(b"1234", b"abcd")])
        with self.assertRaisesRegex(RuntimeError, "incomplete"):
            experiment.check_answers(["ff80"], [query], [], [])


if __name__ == "__main__":
    unittest.main(verbosity=2)
