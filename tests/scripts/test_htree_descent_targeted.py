"""Offline controls; filesystem execution is mocked, never run on this host."""
import importlib.util
import json
import os
import re
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest import mock
import zipfile

sys.dont_write_bytecode = True
SPEC = importlib.util.spec_from_file_location("targeted", Path(__file__).resolve().parents[2] / "scripts/htree-descent-targeted.py")
targeted = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(targeted)


class TargetedControls(unittest.TestCase):
    def test_exact_frozen_schedule_has_nine_then_eight_and_no_substitution(self):
        rows = targeted.frozen()
        self.assertEqual([len(targeted.trial_ids(b, rows)) for b in (0, 1)], [9, 8])
        self.assertEqual(targeted.trial_ids(0, rows) + targeted.trial_ids(1, rows), ["replay"] + [r['id'] for r in rows])
        self.assertEqual((rows[0]['id'], rows[-1]['id']), ('05103', '35297'))
        for invalid in (-1, 2, 32):
            with self.assertRaises(ValueError):
                targeted.trial_ids(invalid, rows)

    def test_each_frozen_input_is_digest_checked_before_use(self):
        for name in targeted.FILES:
            with self.subTest(file=name), tempfile.TemporaryDirectory() as temp:
                p = Path(temp)
                for member in targeted.FILES:
                    shutil.copyfile(targeted.FROZEN / member, p / member)
                with (p / name).open('ab') as stream:
                    stream.write(b'changed')
                with self.assertRaises(RuntimeError):
                    targeted.frozen(p)

    def test_existing_attempt_is_preserved_and_dirty_source_is_refused(self):
        with tempfile.TemporaryDirectory() as temp, mock.patch.object(targeted, 'DATA', Path(temp)), mock.patch.object(targeted, 'source', return_value='source'):
            with self.assertRaises(FileExistsError):
                targeted.prepare()
        with mock.patch.object(targeted.subprocess, 'check_output', return_value=b' M file'):
            with self.assertRaisesRegex(RuntimeError, 'committed and clean'):
                targeted.source()

    def test_bad_replay_preserves_incomplete_preparation(self):
        with tempfile.TemporaryDirectory() as temp, mock.patch.object(targeted, 'DATA', Path(temp) / 'attempt'), mock.patch.object(targeted, 'source', return_value='source'):
            bad = Path(temp) / 'bad.zip';bad.write_bytes(b'bad')
            with self.assertRaisesRegex(RuntimeError, 'replay ZIP identity'):
                targeted.prepare(bad)
            self.assertTrue((targeted.DATA / 'prepare-error.json').exists())
            self.assertFalse((targeted.DATA / 'manifest.json').exists())

    def test_changed_prepared_digest_stops_before_any_child(self):
        with tempfile.TemporaryDirectory() as temp, mock.patch.object(targeted, 'DATA', Path(temp)), mock.patch.object(targeted, 'execute') as execute:
            (Path(temp) / 'manifest.json').write_text('{}')
            (Path(temp) / 'manifest.sha256').write_text('0' * 64)
            with self.assertRaisesRegex(RuntimeError, 'prepared manifest changed'):
                targeted.run_batch(0)
            execute.assert_not_called()

    def test_original_failure_and_missing_capture_stop_after_first_child(self):
        for code in (0, 101, 124, -9):
            with self.subTest(exit=code), tempfile.TemporaryDirectory() as temp, mock.patch.object(targeted, 'DATA', Path(temp)), mock.patch.object(targeted, 'ROOT', Path(temp)), mock.patch.object(targeted, 'plan', return_value={'source': 'source', 'candidates': targeted.frozen()}), mock.patch.object(targeted, 'execute', return_value=code) as execute, mock.patch.object(targeted.shutil, 'disk_usage', return_value=type('Space', (), {'free': 20 * 1024**3})()):
                replay = Path(temp) / 'replay';replay.mkdir();(replay / 'filesystem.json').write_text(json.dumps({'seed_hex': 'seed', 'uuid_hex': 'uuid'}))
                with self.assertRaises(SystemExit):
                    targeted.run_batch(0)
                self.assertEqual(execute.call_count, 1)
                record = json.loads((Path(temp) / 'batch-0/replay.json').read_text())
                self.assertEqual((record['status'], record['exit']), ('stopped', code))
                self.assertFalse((Path(temp) / 'batch-0/result.json').exists())
                env = execute.call_args.args[1]
                self.assertNotIn('FS_EXT4_DESCENT_TRIAL', env)
                self.assertEqual(env['FS_EXT4_DESCENT_TARGETED_TRIAL'], 'replay')

    def test_failed_trial_moves_original_witness_before_stopping(self):
        with tempfile.TemporaryDirectory() as temp, mock.patch.object(targeted, 'ROOT', Path(temp)), mock.patch.object(targeted, 'DATA', Path(temp) / 'data'), mock.patch.object(targeted, 'plan', return_value={'source':'source','candidates':targeted.frozen()}), mock.patch.object(targeted.shutil, 'disk_usage', return_value=type('Space', (), {'free': 20 * 1024**3})()):
            targeted.DATA.mkdir()
            def child(*args):
                p=targeted.ROOT/'tmp/htree-descent-evidence/half_md4-1/profile';p.mkdir(parents=True);(p/'failure.img').write_bytes(b'witness');return 101
            with mock.patch.object(targeted, 'execute', side_effect=child) as execute, self.assertRaises(SystemExit):
                targeted.run_batch(1)
            self.assertEqual(execute.call_count,1)
            self.assertEqual((targeted.DATA/'batch-1/trials/20908/failure.img').read_bytes(),b'witness')

    def test_low_space_stops_before_launching_a_trial(self):
        with tempfile.TemporaryDirectory() as temp, mock.patch.object(targeted,'DATA',Path(temp)), mock.patch.object(targeted,'plan',return_value={'source':'source','candidates':targeted.frozen()}), mock.patch.object(targeted.shutil,'disk_usage',return_value=type('Space',(),{'free':0})()), mock.patch.object(targeted,'execute') as execute:
            with self.assertRaisesRegex(RuntimeError,'insufficient space'):
                targeted.run_batch(1)
            execute.assert_not_called()

    def test_deadline_kills_the_owned_process_group_without_retry(self):
        child=mock.Mock(pid=123)
        child.wait.side_effect=[subprocess.TimeoutExpired('trial',1),0]
        with mock.patch.object(targeted.subprocess,'Popen',return_value=child) as popen, mock.patch.object(targeted.os,'killpg') as kill:
            self.assertEqual(targeted.execute(['trial'],{},1),124)
            self.assertTrue(popen.call_args.kwargs['start_new_session'])
            kill.assert_called_once_with(123,targeted.signal.SIGTERM)

    def test_layout_retains_same_leaf_and_cross_leaf_observations(self):
        trial=targeted.frozen()[0]
        for cross in (False,True):
            with tempfile.TemporaryDirectory() as temp:
                p=Path(temp);queries=[]
                for i,member in enumerate(trial['groups'][0]):
                    queries.append(dict(member,actual=[i+1 if cross else 1]))
                (p/'queries.jsonl').write_text(''.join(json.dumps(q)+'\n' for q in queries))
                (p/'routes.jsonl').write_text(json.dumps({'parent':0,'hash':trial['groups'][0][0]['major']|1,'logical':2})+'\n')
                targeted.layout(p,trial)
                result=json.loads((p/'collision-layout.json').read_text())
                self.assertEqual(result['groups'][0]['cross_leaf'],cross)
                self.assertEqual(len(result['groups'][0]['continuation_routes']),1)
                queries[0]['minor']^=1
                (p/'queries.jsonl').write_text(''.join(json.dumps(q)+'\n' for q in queries))
                with self.assertRaisesRegex(RuntimeError,'independently confirmed'):
                    targeted.layout(p,trial)

    def test_uploaded_inventory_and_bytes_must_be_complete(self):
        with tempfile.TemporaryDirectory() as temp, mock.patch.object(targeted,'ROOT',Path(temp)), mock.patch.object(targeted,'DATA',Path(temp)/'data'), mock.patch.object(targeted,'source',return_value='source'):
            targeted.DATA.mkdir();(targeted.DATA/'one').write_bytes(b'one');(targeted.DATA/'two').write_bytes(b'two')
            (Path(temp)/'tmp').mkdir()
            def downloaded(artifact,path,*args,**kwargs):
                with zipfile.ZipFile(path,'w') as z:z.writestr('one',b'one')
                return {'workflow_run':{'id':1}}
            with mock.patch.object(targeted.common,'download_artifact',side_effect=downloaded), mock.patch.dict(os.environ,{},clear=True):
                with self.assertRaisesRegex(RuntimeError,'inventory'):
                    targeted.verify_upload(1)

    def test_successful_batch_observes_each_fixed_candidate_once(self):
        rows=targeted.frozen()
        with tempfile.TemporaryDirectory() as temp, mock.patch.object(targeted,'ROOT',Path(temp)), mock.patch.object(targeted,'DATA',Path(temp)/'data'), mock.patch.object(targeted,'plan',return_value={'source':'source','candidates':rows}), mock.patch.object(targeted.shutil,'disk_usage',return_value=type('Space',(),{'free':20*1024**3})()), mock.patch.object(targeted.common,'audit_profile') as audit, mock.patch.object(targeted.subprocess,'run') as floor:
            targeted.DATA.mkdir()
            def child(command,env,remaining):
                ident=env['FS_EXT4_DESCENT_TARGETED_TRIAL'];trial=next(r for r in rows if r['id']==ident)
                profile=targeted.ROOT/'tmp/htree-descent-evidence/half_md4-1'/ident;profile.mkdir(parents=True)
                queries=[dict(m,actual=[1]) for group in trial['groups'] for m in group]
                (profile/'queries.jsonl').write_text(''.join(json.dumps(q)+'\n' for q in queries));(profile/'routes.jsonl').write_text('')
                self.assertGreater(remaining,0);self.assertLessEqual(remaining,2400)
                self.assertEqual(command[-4:],['half_md4_signed','--','--exact','--show-output'])
                return 0
            with mock.patch.object(targeted,'execute',side_effect=child) as execute:
                targeted.run_batch(1)
            self.assertEqual(execute.call_count,8);self.assertEqual(audit.call_count,8);self.assertEqual(floor.call_count,8)
            self.assertEqual(json.loads((targeted.DATA/'batch-1/result.json').read_text())['completed'],[r['id'] for r in rows[8:]])
            self.assertTrue(all((targeted.DATA/'batch-1/trials'/r['id']/'collision-layout.json').exists() for r in rows[8:]))

    def test_controller_exception_still_captures_available_profiles(self):
        with tempfile.TemporaryDirectory() as temp, mock.patch.object(targeted,'ROOT',Path(temp)), mock.patch.object(targeted,'DATA',Path(temp)/'data'), mock.patch.object(targeted,'plan',return_value={'source':'source','candidates':targeted.frozen()}), mock.patch.object(targeted.shutil,'disk_usage',return_value=type('Space',(),{'free':20*1024**3})()):
            targeted.DATA.mkdir()
            def child(*args):
                p=targeted.ROOT/'tmp/htree-descent-evidence/half_md4-1/profile';p.mkdir(parents=True);(p/'partial.img').write_bytes(b'partial');raise OSError('controller interrupted')
            with mock.patch.object(targeted,'execute',side_effect=child) as execute, self.assertRaises(SystemExit):
                targeted.run_batch(1)
            self.assertEqual(execute.call_count,1)
            self.assertEqual((targeted.DATA/'batch-1/controller-error-profiles/20908/0/partial.img').read_bytes(),b'partial')

    def test_changed_workflow_identity_stops_before_execution(self):
        rows=targeted.frozen()
        with tempfile.TemporaryDirectory() as temp, mock.patch.object(targeted,'DATA',Path(temp)), mock.patch.object(targeted,'source',return_value='source'), mock.patch.dict(os.environ,{'GITHUB_RUN_ID':'current'}):
            p=Path(temp);f=p/'frozen';f.mkdir()
            for name in targeted.FILES:shutil.copyfile(targeted.FROZEN/name,f/name)
            value={'protocol':'targeted-collision-v1','frozen_sha256':targeted.FROZEN_SHA,'corpus_sha256':targeted.common.CORPUS_SHA,'candidates':rows,'source':'source','workflow_run_id':'different'}
            (p/'manifest.json').write_text(json.dumps(value));(p/'manifest.sha256').write_text(targeted.common.digest(p/'manifest.json'))
            with self.assertRaisesRegex(RuntimeError,'workflow run changed'):targeted.plan()

    def test_source_inventory_cannot_be_truncated(self):
        rows=targeted.frozen()
        with tempfile.TemporaryDirectory() as temp, mock.patch.object(targeted,'DATA',Path(temp)), mock.patch.object(targeted,'source',return_value='source'), mock.patch.object(targeted,'check_replay'), mock.patch.object(targeted,'tracked_files',return_value=[b'expected']), mock.patch.dict(os.environ,{},clear=True):
            p=Path(temp);f=p/'frozen';f.mkdir()
            for name in targeted.FILES:shutil.copyfile(targeted.FROZEN/name,f/name)
            value={'protocol':'targeted-collision-v1','frozen_sha256':targeted.FROZEN_SHA,'corpus_sha256':targeted.common.CORPUS_SHA,'candidates':rows,'source':'source','workflow_run_id':None,'replay_artifact':targeted.common.REPLAY_ARTIFACT,'replay_zip_sha256':targeted.common.REPLAY_ZIP_SHA}
            (p/'manifest.json').write_text(json.dumps(value));(p/'manifest.sha256').write_text(targeted.common.digest(p/'manifest.json'));(p/'source-sha256.txt').write_text('')
            with self.assertRaisesRegex(RuntimeError,'source inventory'):targeted.plan()

    def test_complete_upload_checks_each_retained_byte_and_new_metadata_mode(self):
        with tempfile.TemporaryDirectory() as temp, mock.patch.object(targeted,'ROOT',Path(temp)), mock.patch.object(targeted,'DATA',Path(temp)/'data'), mock.patch.object(targeted,'source',return_value='source'), mock.patch.dict(os.environ,{},clear=True):
            targeted.DATA.mkdir();(targeted.DATA/'one').write_bytes(b'one');(Path(temp)/'tmp').mkdir()
            for value in (b'wrong',b'one'):
                def downloaded(artifact,path,source,**kwargs):
                    self.assertEqual(source,'source');self.assertTrue(kwargs['newly_uploaded'])
                    with zipfile.ZipFile(path,'w') as z:z.writestr('one',value)
                    return {'workflow_run':{'id':1}}
                with mock.patch.object(targeted.common,'download_artifact',side_effect=downloaded):
                    if value==b'wrong':
                        with self.assertRaisesRegex(RuntimeError,'uploaded bytes'):targeted.verify_upload(1)
                    else:targeted.verify_upload(1)

    def test_upload_from_another_workflow_run_is_rejected(self):
        with tempfile.TemporaryDirectory() as temp, mock.patch.object(targeted,'ROOT',Path(temp)), mock.patch.object(targeted,'source',return_value='source'), mock.patch.dict(os.environ,{'GITHUB_RUN_ID':'1'}), mock.patch.object(targeted.common,'download_artifact',return_value={'workflow_run':{'id':2}}):
            with self.assertRaisesRegex(RuntimeError,'another workflow run'):targeted.verify_upload(1)

    def test_audit_failure_stops_with_original_zero_exit_and_witness(self):
        with tempfile.TemporaryDirectory() as temp, mock.patch.object(targeted,'ROOT',Path(temp)), mock.patch.object(targeted,'DATA',Path(temp)/'data'), mock.patch.object(targeted,'plan',return_value={'source':'source','candidates':targeted.frozen()}), mock.patch.object(targeted.shutil,'disk_usage',return_value=type('Space',(),{'free':20*1024**3})()), mock.patch.object(targeted.subprocess,'run'), mock.patch.object(targeted.common,'audit_profile',side_effect=RuntimeError('independent hash disagreement')):
            targeted.DATA.mkdir()
            def child(*args):
                p=targeted.ROOT/'tmp/htree-descent-evidence/half_md4-1/profile';p.mkdir(parents=True);(p/'indexed.img').write_bytes(b'witness');return 0
            with mock.patch.object(targeted,'execute',side_effect=child) as execute,self.assertRaises(SystemExit):targeted.run_batch(1)
            self.assertEqual(execute.call_count,1)
            record=json.loads((targeted.DATA/'batch-1/20908.json').read_text())
            self.assertEqual((record['exit'],record['status']),(0,'stopped'))
            self.assertIn('independent hash disagreement',record['reason'])
            self.assertEqual((targeted.DATA/'batch-1/trials/20908/indexed.img').read_bytes(),b'witness')

    def test_deadline_escalates_only_for_the_same_owned_group(self):
        child=mock.Mock(pid=123);child.wait.side_effect=[subprocess.TimeoutExpired('trial',1),subprocess.TimeoutExpired('trial',5),0]
        with mock.patch.object(targeted.subprocess,'Popen',return_value=child),mock.patch.object(targeted.os,'killpg') as kill:
            self.assertEqual(targeted.execute(['trial'],{},1),124)
            self.assertEqual(kill.call_args_list,[mock.call(123,targeted.signal.SIGTERM),mock.call(123,targeted.signal.SIGKILL)])

    def test_workflow_is_manual_two_sequential_batches_with_always_retention(self):
        text=(targeted.ROOT/'.github/workflows/htree-descent-targeted.yml').read_text()
        self.assertEqual(len(re.findall(r'^  targeted-batch-[01]:$',text,re.M)),2)
        self.assertIn('needs: [targeted-batch-0]',text)
        self.assertEqual(text.count('timeout-minutes: 40'),2)
        self.assertEqual(len(re.findall(r'^\s+if: always\(\)$',text,re.M)),3)
        self.assertEqual(text.count("if: always() && steps.evidence.outcome == 'success'"),2)
        self.assertEqual(text.count('ref: ${{ github.sha }}'),2)
        self.assertNotIn('  push:',text)
        self.assertNotIn('  pull_request:',text)


if __name__=='__main__':
    unittest.main()
