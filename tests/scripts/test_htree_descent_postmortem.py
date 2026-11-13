"""Admission/retention controls; child filesystem execution is always mocked."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock

sys.dont_write_bytecode = True
SPEC = importlib.util.spec_from_file_location('postmortem', Path(__file__).resolve().parents[2] / 'scripts/htree-descent-postmortem.py')
subject = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(subject)


def manifest(directory):
    retained = directory / 'retained'
    retained.mkdir()
    (retained / 'indexed.img').write_bytes(b'fixed')
    (directory / 'witness.zip').write_bytes(b'capsule')
    return {'source': 'source', 'zip_sha256': hashlib.sha256(b'capsule').hexdigest(), 'input_sha256': {'indexed.img': hashlib.sha256(b'fixed').hexdigest()}}


class PostmortemControls(unittest.TestCase):
    def test_wrong_archive_is_refused_before_opening_or_extracting_it(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / 'wrong.zip'
            path.write_bytes(b'not the reviewed artifact')
            with mock.patch.object(subject.zipfile, 'ZipFile') as opened:
                with self.assertRaisesRegex(RuntimeError, 'fixed witness archive digest'):
                    subject.validate_archive(path, Path(temp) / 'extract')
                opened.assert_not_called()
            self.assertFalse((Path(temp) / 'extract').exists())

    def test_existing_preparation_is_preserved(self):
        with tempfile.TemporaryDirectory() as temp, mock.patch.object(subject, 'DATA', Path(temp)), mock.patch.object(subject.targeted, 'source', return_value='source'), mock.patch.object(subject.common, 'download_artifact') as download:
            (Path(temp) / 'prior').write_text('retained')
            with self.assertRaises(FileExistsError):
                subject.prepare()
            self.assertEqual((Path(temp) / 'prior').read_text(), 'retained')
            download.assert_not_called()

    def test_failed_preparation_keeps_the_error_without_a_ready_manifest(self):
        with tempfile.TemporaryDirectory() as temp, mock.patch.object(subject, 'DATA', Path(temp) / 'new'), mock.patch.object(subject.targeted, 'source', return_value='source'):
            path = Path(temp) / 'wrong.zip'
            path.write_bytes(b'wrong')
            with self.assertRaises(RuntimeError):
                subject.prepare(path)
            self.assertEqual(json.loads((subject.DATA / 'prepare-error.json').read_text())['status'], 'incomplete')
            self.assertFalse((subject.DATA / 'prepared.json').exists())

    def test_manifest_digest_is_checked_before_any_child(self):
        with tempfile.TemporaryDirectory() as temp, mock.patch.object(subject, 'DATA', Path(temp)), mock.patch.object(subject.targeted, 'execute') as execute:
            (Path(temp) / 'prepared.json').write_text('{}')
            (Path(temp) / 'prepared.sha256').write_text('0' * 64)
            with self.assertRaisesRegex(RuntimeError, 'manifest changed'):
                subject.run()
            execute.assert_not_called()

    def test_source_run_capsule_and_input_changes_are_refused(self):
        for change in ('none', 'source', 'sources', 'run', 'identity', 'capsule', 'input'):
            with self.subTest(change=change), tempfile.TemporaryDirectory() as temp, mock.patch.object(subject, 'DATA', Path(temp)), mock.patch.object(subject.targeted, 'source', return_value='source'), mock.patch.object(subject, 'source_inventory', return_value={'file': 'sha'}), mock.patch.dict(os.environ, {}, clear=True):
                directory = Path(temp)
                row = manifest(directory)
                (directory / 'witness.zip').write_bytes(b'capsule')
                capsule_sha = subject.common.digest(directory / 'witness.zip')
                row.update(workflow_run_id=None, source_sha256={'file': 'sha'}, original_source=subject.SOURCE, original_run=subject.RUN, original_artifact=subject.ARTIFACT, original_exit=101, zip_sha256=capsule_sha)
                if change == 'source': row['source'] = 'other'
                if change == 'sources': row['source_sha256'] = {'file': 'changed'}
                if change == 'run': row['workflow_run_id'] = 'other'
                if change == 'identity': row['original_exit'] = 0
                if change == 'capsule': (directory / 'witness.zip').write_bytes(b'changed')
                if change == 'input': (directory / 'retained/indexed.img').write_bytes(b'changed')
                (directory / 'prepared.json').write_text(json.dumps(row))
                (directory / 'prepared.sha256').write_text(subject.common.digest(directory / 'prepared.json'))
                with mock.patch.object(subject, 'ZIP_SHA', capsule_sha):
                    if change == 'none': self.assertEqual(subject.plan(), row)
                    else:
                        with self.assertRaises(RuntimeError): subject.plan()

    def test_missing_extra_or_changed_retained_files_and_capsule_fail_the_guard(self):
        for change in ('missing', 'extra', 'changed', 'capsule'):
            with self.subTest(change=change), tempfile.TemporaryDirectory() as temp, mock.patch.object(subject, 'DATA', Path(temp)):
                row = manifest(Path(temp))
                if change == 'missing': (Path(temp) / 'retained/indexed.img').unlink()
                if change == 'extra': (Path(temp) / 'retained/extra').write_text('extra')
                if change == 'changed': (Path(temp) / 'retained/indexed.img').write_text('changed')
                if change == 'capsule': (Path(temp) / 'witness.zip').write_text('changed')
                with self.assertRaisesRegex(RuntimeError, 'changed'): subject.verify_inputs(row)

    def test_completion_marker_cannot_replace_missing_raw_observations(self):
        with tempfile.TemporaryDirectory() as temp, mock.patch.object(subject, 'DATA', Path(temp)):
            (Path(temp) / 'checks').mkdir()
            (Path(temp) / 'checks/completion.txt').write_text('independent checks complete\n')
            with self.assertRaises(FileNotFoundError): subject.verify_outputs()

    def test_original_child_failure_is_retained_and_never_retried(self):
        for code in (101, 124, -9, 0):
            with self.subTest(exit=code), tempfile.TemporaryDirectory() as temp, mock.patch.object(subject, 'DATA', Path(temp)), mock.patch.object(subject.shutil, 'disk_usage', return_value=type('Space', (), {'free': 5 * 1024**3})()), mock.patch.object(subject.targeted, 'execute', return_value=code) as execute:
                row = manifest(Path(temp))
                with mock.patch.object(subject, 'plan', return_value=row), self.assertRaises(SystemExit): subject.run()
                record = json.loads((Path(temp) / 'result.json').read_text())
                self.assertEqual((record['exit'], record['original_exit'], record['status'], record['immutable_inputs']), (code, 101, 'stopped', 'unchanged'))
                execute.assert_called_once()
                self.assertEqual(execute.call_args.args[2], 1200)

    def test_exception_keeps_the_partial_record_and_immutable_guard(self):
        with tempfile.TemporaryDirectory() as temp, mock.patch.object(subject, 'DATA', Path(temp)), mock.patch.object(subject.shutil, 'disk_usage', return_value=type('Space', (), {'free': 5 * 1024**3})()), mock.patch.object(subject.targeted, 'execute', side_effect=OSError('interrupted')) as execute:
            row = manifest(Path(temp))
            with mock.patch.object(subject, 'plan', return_value=row), self.assertRaises(SystemExit): subject.run()
            record = json.loads((Path(temp) / 'result.json').read_text())
            self.assertEqual((record['status'], record['reason'], record['immutable_inputs']), ('stopped', 'interrupted', 'unchanged'))
            execute.assert_called_once()

    def test_success_cannot_hide_changed_original_inputs(self):
        with tempfile.TemporaryDirectory() as temp, mock.patch.object(subject, 'DATA', Path(temp)), mock.patch.object(subject.shutil, 'disk_usage', return_value=type('Space', (), {'free': 5 * 1024**3})()):
            row = manifest(Path(temp))
            def child(*args):
                (subject.DATA / 'checks').mkdir()
                (subject.DATA / 'checks/completion.txt').write_text('independent checks complete\n')
                (subject.DATA / 'retained/indexed.img').write_text('changed')
                return 0
            with mock.patch.object(subject, 'plan', return_value=row), mock.patch.object(subject.targeted, 'execute', side_effect=child), self.assertRaises(SystemExit): subject.run()
            record = json.loads((Path(temp) / 'result.json').read_text())
            self.assertEqual((record['status'], record['immutable_inputs']), ('stopped', 'FAILED'))

    def test_success_preserves_the_original_red_and_requires_later_classification(self):
        with tempfile.TemporaryDirectory() as temp, mock.patch.object(subject, 'DATA', Path(temp)), mock.patch.object(subject.shutil, 'disk_usage', return_value=type('Space', (), {'free': 5 * 1024**3})()):
            row = manifest(Path(temp))
            def child(command, environment, timeout):
                self.assertIn('session', command)
                self.assertIn('htree_descent_postmortem', command)
                self.assertNotIn('htree_index_descent_oracle', command)
                self.assertEqual(timeout, 1200)
                (subject.DATA / 'checks').mkdir()
                (subject.DATA / 'checks/completion.txt').write_text('independent checks complete\n')
                return 0
            with mock.patch.object(subject, 'plan', return_value=row), mock.patch.object(subject.targeted, 'execute', side_effect=child) as execute, mock.patch.object(subject, 'verify_outputs', return_value={'tool_calls': 9, 'cold_read_only_mounts': 2}) as checked: subject.run()
            record = json.loads((Path(temp) / 'result.json').read_text())
            self.assertEqual((record['exit'], record['original_exit'], record['immutable_inputs']), (0, 101, 'unchanged'))
            self.assertIn('classification requires review', record['status'])
            execute.assert_called_once()
            checked.assert_called_once()

    def test_low_space_or_existing_attempt_prevents_any_child(self):
        for prior in (False, True):
            with self.subTest(prior=prior), tempfile.TemporaryDirectory() as temp, mock.patch.object(subject, 'DATA', Path(temp)), mock.patch.object(subject.shutil, 'disk_usage', return_value=type('Space', (), {'free': 0})()), mock.patch.object(subject.targeted, 'execute') as execute:
                row = manifest(Path(temp))
                if prior: (Path(temp) / 'checks').mkdir()
                with mock.patch.object(subject, 'plan', return_value=row), self.assertRaises(RuntimeError): subject.run()
                execute.assert_not_called()


if __name__ == '__main__':
    unittest.main()
