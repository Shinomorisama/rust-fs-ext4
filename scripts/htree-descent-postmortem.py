#!/usr/bin/env python3
"""One fixed retained-witness postmortem; all filesystem oracles stay in the VM."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import sys
import zipfile

sys.dont_write_bytecode = True
SPEC = importlib.util.spec_from_file_location('targeted', Path(__file__).with_name('htree-descent-targeted.py'))
targeted = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(targeted)
common = targeted.common
ROOT = targeted.ROOT
DATA = ROOT / 'tmp/htree-descent-postmortem'
ARTIFACT = 14031054992
RUN = 43415472096
SOURCE = 'dea278cf6cd69bfca9e9f23ecbae331a4c6c6e14'
ZIP_SHA = '4490cf7307b2c018878aa33f5cdc31a197a0d3268e7372f9981f040658792a01'
PREFIX = 'batch-0/trials/13835/'
PHASES = {
    'formatted': 'f672b237be05b59d80d0d9f49b4be0604167c80aac240834535fbe946455d98b',
    'configured': 'a31a80e4de44a239b32110036daee41abc2fcbaf28ef24520cacf26824d4631a',
    'indexed': '9d60cf8562f3ef70d8f040701a1e549a9e1f1bb2ff747570707da5074f40992d',
}


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def validate_archive(path, destination=None):
    require(common.digest(path) == ZIP_SHA, 'fixed witness archive digest differs')
    with zipfile.ZipFile(path) as archive:
        members = [m for m in archive.infolist() if not m.is_dir()]
        names = [m.filename for m in members]
        require(len(names) == len(set(names)) == 353, 'witness archive inventory differs')
        require(sum(m.file_size for m in members) < 2 * 1024**3, 'witness archive exceeds extraction bound')
        for member in members:
            common.member_path(Path('safe'), member.filename)
            require((member.external_attr >> 16) & 0o170000 != 0o120000, 'archive symlink is refused')
        require(archive.testzip() is None, 'witness archive CRC differs')
        read_json = lambda name: json.loads(archive.read(name))
        manifest = read_json('manifest.json')
        require(manifest['source'] == SOURCE and manifest['workflow_run_id'] == str(RUN), 'original source/run differs')
        require(archive.read('source-commit.txt').decode().strip() == SOURCE, 'original source sidecar differs')
        record = read_json('batch-0/13835.json')
        require((record['source'], record['trial']['id'], record['status'], record['exit']) == (SOURCE, '13835', 'stopped', 101), 'original failure identity/status differs')
        require(record['captured_profiles'] == [PREFIX.rstrip('/')], 'original witness location differs')
        require(read_json(PREFIX + 'result.json') == {'wrong': 1}, 'original mismatch count differs')
        require(archive.read(PREFIX + 'check.verdict').strip() == b'Clean', 'original judged consistency differs')
        settings = read_json(PREFIX + 'filesystem.json')
        require(settings == {'uuid_hex': '80fcf3c1fbdbfa27abe389e479b7ecef', 'seed_hex': '0fca7f62ba785c0649a14ab6381e9270', 'seed_uuid': '0fca7f62-ba78-5c06-49a1-4ab6381e9270', 'version': 1, 'unsigned': False, 'depth': 0, 'block_size': 1024, 'inode': 12}, 'original filesystem settings differ')
        for phase, expected in PHASES.items():
            member = archive.getinfo(PREFIX + phase + '.img')
            require(member.file_size == 64 * 1024**2, 'witness phase size differs')
            with archive.open(member) as stream:
                require(hashlib.file_digest(stream, 'sha256').hexdigest() == expected, 'witness phase bytes differ')
            require(archive.read(PREFIX + phase + '.sha256').decode().strip() == expected, 'witness phase sidecar differs')
        with archive.open(PREFIX + 'working-at-failure.img') as stream:
            require(hashlib.file_digest(stream, 'sha256').hexdigest() == PHASES['indexed'], 'failure copy differs from indexed witness')
        names_hex = archive.read(PREFIX + 'names.hex').decode().splitlines()
        queries = [json.loads(line) for line in archive.read(PREFIX + 'queries.jsonl').splitlines()]
        requests = archive.read(PREFIX + 'hash-requests.bin').splitlines()
        answers = re.findall(rb'Hash of .*? is 0x([0-9a-fA-F]+) \(minor 0x([0-9a-fA-F]+)\)', archive.read(PREFIX + 'hashes.stdout'))
        require(len(names_hex) == len(queries) == len(requests) == len(answers) == 1500, 'original independent evidence is incomplete')
        wrong = []
        for index, (name, query, request, answer) in enumerate(zip(names_hex, queries, requests, answers)):
            require(query['name_hex'] == name and request.rsplit(b' -- "', 1)[1][:-1].hex() == name, 'original query/request name differs')
            require((query['major'], query['minor']) == tuple(int(h, 16) for h in answer), 'original independent hash differs')
            if not query['found'] or query['selected'] not in query['actual']:
                wrong.append(index)
        require(wrong == [863] and queries[863] == {'name_hex': '636166c3a95f383633', 'major': 4292002400, 'minor': 3853483151, 'selected': 49, 'found': False, 'actual': [50]}, 'original failed query differs')
        selected = [m for m in members if m.filename.startswith(PREFIX)]
        inventory = {}
        for member in selected:
            name = member.filename.removeprefix(PREFIX)
            with archive.open(member) as stream:
                inventory[name] = hashlib.file_digest(stream, 'sha256').hexdigest()
            if destination is not None:
                output = common.member_path(destination, name)
                output.parent.mkdir(parents=True, exist_ok=True)
                with archive.open(member) as stream, output.open('xb') as target:
                    shutil.copyfileobj(stream, target)
                output.chmod(0o444)
        return inventory


def source_inventory():
    return {os.fsdecode(name): common.digest(ROOT / os.fsdecode(name)) for name in targeted.tracked_files()}


def prepare(archive=None):
    commit = targeted.source()
    DATA.mkdir(parents=True, exist_ok=False)
    try:
        retained = DATA / 'witness.zip'
        if archive is None:
            metadata = common.download_artifact(ARTIFACT, retained, SOURCE, ZIP_SHA)
            require(metadata['workflow_run']['id'] == RUN, 'artifact belongs to another run')
        else:
            require(common.digest(archive) == ZIP_SHA, 'fixed witness archive digest differs')
            shutil.copyfile(archive, retained)
        inputs = validate_archive(retained, DATA / 'retained')
        # Preserve the original log and status alongside the immutable archive.
        with zipfile.ZipFile(retained) as original:
            for name in ('batch-0/13835.json', 'batch-0/logs/targeted-descent-13835.log'):
                (DATA / Path(name).name).write_bytes(original.read(name))
        prepared = {'source': commit, 'workflow_run_id': os.getenv('GITHUB_RUN_ID'),
                    'original_source': SOURCE, 'original_run': RUN, 'original_artifact': ARTIFACT,
                    'original_exit': 101, 'zip_sha256': ZIP_SHA,
                    'source_sha256': source_inventory(), 'input_sha256': inputs}
        path = DATA / 'prepared.json'
        path.write_text(json.dumps(prepared, indent=2) + '\n')
        (DATA / 'prepared.sha256').write_text(common.digest(path) + '\n')
    except Exception as error:
        (DATA / 'prepare-error.json').write_text(json.dumps({'status': 'incomplete', 'reason': str(error)}) + '\n')
        raise
    print('postmortem preparation: fixed failed witness verified; no filesystem execution')


def plan():
    path = DATA / 'prepared.json'
    require(common.digest(path) == (DATA / 'prepared.sha256').read_text().strip(), 'postmortem manifest changed')
    manifest = json.loads(path.read_text())
    require(manifest['source'] == targeted.source() and manifest['source_sha256'] == source_inventory(), 'postmortem source changed')
    require(manifest['workflow_run_id'] == os.getenv('GITHUB_RUN_ID'), 'postmortem workflow run changed')
    require((manifest['original_source'], manifest['original_run'], manifest['original_artifact'], manifest['original_exit'], manifest['zip_sha256']) == (SOURCE, RUN, ARTIFACT, 101, ZIP_SHA), 'prepared witness identity differs')
    require(common.digest(DATA / 'witness.zip') == ZIP_SHA, 'immutable witness archive changed')
    verify_inputs(manifest)
    return manifest


def verify_inputs(manifest):
    require(common.digest(DATA / 'witness.zip') == manifest['zip_sha256'], 'immutable witness archive changed')
    actual = {str(p.relative_to(DATA / 'retained')): common.digest(p) for p in (DATA / 'retained').rglob('*') if p.is_file()}
    require(actual == manifest['input_sha256'], 'immutable retained input changed')


def verify_outputs():
    output = DATA / 'checks'
    labels = ['check-before', 'hashes', 'inode-0', 'inode-1', 'directory-inode', 'htree', 'map-49', 'map-50', 'cold-lookup-0', 'cold-lookup-1', 'check-after']
    for label in labels:
        require((output / (label + '.status')).read_text() == 'Some(0)\n', 'fresh observation status differs')
        require((output / (label + '.stdout')).is_file() and (output / (label + '.stderr')).is_file(), 'fresh observation streams missing')
    for label in ('check-before', 'check-after'):
        require((output / (label + '.verdict')).read_text() == 'Clean\n', 'fresh judged consistency differs')
    require((output / 'final-integrity.txt').read_text() == f"input {PHASES['indexed']}\nprivate {PHASES['indexed']}\n", 'final image integrity report differs')
    require(common.digest(output / 'private-check.img') == PHASES['indexed'], 'private image bytes changed')
    require((output / 'driver-observations.txt').read_text() == '615f6e616d655f6f665f7477656e74795f3030313231|ffd2c260|b71eb1ce|Some(49)|[49]|337\n636166c3a95f383633|ffd2c260|e5af7c8f|Some(49)|[50]|882\n', 'current-driver observation differs')
    return {'tool_calls': 9, 'cold_read_only_mounts': 2, 'independent_hash_answers': 2, 'independent_named_inode_reports': 2}


def run():
    manifest = plan()
    result_file = DATA / 'result.json'
    require(not result_file.exists() and not (DATA / 'checks').exists(), 'existing postmortem attempt is preserved')
    require(shutil.disk_usage(ROOT).free >= 3 * 1024**3, 'insufficient postmortem disk reserve')
    record = {'source': manifest['source'], 'original_run': RUN, 'original_exit': 101, 'status': 'started'}
    result_file.write_text(json.dumps(record, indent=2) + '\n')
    command = ['scripts/tier.sh', 'witness:postmortem', 'witness-postmortem', '4200', '340000', '--',
               str(ROOT / '../fs-linux-test-harness/scripts/vm.sh'), 'session',
               'cargo', 'run', '--locked', '--release', '--example', 'htree_descent_postmortem', '--',
               str(DATA / 'retained/indexed.img'), str(DATA / 'checks')]
    try:
        record['exit'] = targeted.execute(command, os.environ.copy(), 20 * 60)
        verify_inputs(manifest)
        require(record['exit'] == 0, 'postmortem failed; retain original postmortem exit and reports')
        require((DATA / 'checks/completion.txt').read_text() == 'independent checks complete\n', 'postmortem completion missing')
        record['observations'] = verify_outputs()
        record['status'] = 'independent checks complete; classification requires review'
    except Exception as error:
        record['status'] = 'stopped'
        record['reason'] = str(error)
    finally:
        try:
            verify_inputs(manifest)
            record['immutable_inputs'] = 'unchanged'
        except Exception:
            record['immutable_inputs'] = 'FAILED'
            record['status'] = 'stopped'
        log = ROOT / 'tmp/logs/witness-postmortem.log'
        if log.exists():
            shutil.copyfile(log, DATA / 'witness-postmortem.log')
        result_file.write_text(json.dumps(record, indent=2) + '\n')
    if record['status'] == 'stopped':
        raise SystemExit(max(1, record.get('exit', 1)))
    print('postmortem: one fixed witness examined; original trial remains exit 101; no correction')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    sub.add_parser('prepare').add_argument('--artifact-zip', type=Path)
    sub.add_parser('validate-input').add_argument('artifact_zip', type=Path)
    sub.add_parser('run')
    sub.add_parser('verify-upload').add_argument('artifact', type=int)
    args = parser.parse_args()
    if args.command == 'prepare':
        prepare(args.artifact_zip)
    elif args.command == 'validate-input':
        inventory = validate_archive(args.artifact_zip)
        print(f'fixed witness archive and {len(inventory)} retained files verified; no filesystem execution')
    elif args.command == 'run':
        run()
    else:
        previous = targeted.DATA
        try:
            targeted.DATA = DATA
            targeted.verify_upload(args.artifact)
        finally:
            targeted.DATA = previous


if __name__ == '__main__':
    main()
