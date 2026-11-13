#!/usr/bin/env python3
"""Frozen collision-candidate trials; all filesystem tools stay in the prescribed VM."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
import sys
import time
import zipfile

sys.dont_write_bytecode = True
SPEC = importlib.util.spec_from_file_location("targeted_common", Path(__file__).with_name("htree-descent-experiment.py"))
common = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(common)
ROOT = common.ROOT
DATA = ROOT / "tmp/htree-descent-targeted"
FROZEN = ROOT / "docs/evidence/htree-descent-targeted-2026-11-12"
FROZEN_SHA = "45e6d0c5b6a808e00ecc2269397bf8278b5adbbca1515194091516109cb98558"
FILES = ("manifest.json", "manifest.sha256", "names.hex", "candidates.jsonl", "screen.tsv", "special.jsonl")


def require(condition, reason):
    if not condition:
        raise RuntimeError(reason)


def frozen(directory=None):
    directory = directory or FROZEN
    require(common.digest(directory / "manifest.json") == FROZEN_SHA, "frozen manifest identity differs")
    manifest = json.loads((directory / "manifest.json").read_text())
    require((directory / "manifest.sha256").read_text().strip() == FROZEN_SHA, "frozen sidecar differs")
    for name, sha in manifest["files_sha256"].items():
        require(name in FILES and common.digest(directory / name) == sha, "frozen file identity differs")
    require(manifest["status"] == "complete" and manifest["outcome"] == "target_reached", "screen incomplete")
    rows = common.read_jsonl(directory / "candidates.jsonl")
    require(len(rows) == 16 and [r["id"] for r in rows] == manifest["candidate_ids"], "frozen candidate ordering differs")
    names = (directory / "names.hex").read_text().splitlines()
    require(len(names) == len(set(names)) == 1500 and common.digest(directory / "names.hex") == common.CORPUS_SHA, "corpus identity differs")
    for row in rows:
        ident = row["id"]
        require(re.fullmatch(r"[0-9]{5}", ident) and row["index"] == int(ident), "ambiguous candidate identity")
        for field in ("seed", "uuid"):
            value = hashlib.sha256(f"casefold-descent-targeted-2026-11-12/{field}/{ident}".encode()).digest()[:16].hex()
            require(row[field + "_hex"] == value, "candidate identity formula differs")
        require(bool(row["groups"]), "candidate has no collision group")
        for group in row["groups"]:
            require(len(group) >= 2 and len({m["name_index"] for m in group}) == len(group), "candidate group is incomplete")
            require(len({m["major"] for m in group}) == 1 and group[0]["major"] != 0xfffffffc, "candidate is not an ordinary collision")
            for member in group:
                require(names[member["name_index"]] == member["name_hex"], "candidate name differs")
    return rows


def trial_ids(batch, rows=None):
    rows = rows if rows is not None else frozen()
    if batch not in (0, 1):
        raise ValueError("targeted batch must be 0 or 1")
    return (["replay"] if batch == 0 else []) + [r["id"] for r in rows[batch * 8:batch * 8 + 8]]


def source():
    require(not subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT), "source must be committed and clean")
    return subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()


def check_replay():
    require(common.digest(DATA / "replay.zip") == common.REPLAY_ZIP_SHA, "replay ZIP identity differs")
    replay = DATA / "replay"
    require(common.digest(replay / "configured.img") == common.CONFIG_SHA and common.digest(replay / "names.hex") == common.CORPUS_SHA, "replay input differs")
    for phase in ("formatted", "configured", "indexed"):
        image = replay / (phase + ".img")
        require(image.stat().st_size == 64 * 1024**2 and common.digest(image) == (replay / (phase + ".sha256")).read_text().strip(), "replay phase identity differs")


def tracked_files():
    return sorted(set(subprocess.check_output(["git", "ls-files", "-z"], cwd=ROOT).split(b"\0")) - {b""})


def prepare(replay_zip=None):
    rows = frozen()
    commit = source()
    DATA.mkdir(parents=True, exist_ok=False)
    try:
        snapshot = DATA / "frozen"
        snapshot.mkdir()
        for name in FILES:
            shutil.copyfile(FROZEN / name, snapshot / name)
        archive_path = DATA / "replay.zip"
        if replay_zip is None:
            common.download_artifact(common.REPLAY_ARTIFACT, archive_path, common.REPLAY_SOURCE, common.REPLAY_ZIP_SHA)
        else:
            require(common.digest(replay_zip) == common.REPLAY_ZIP_SHA, "replay ZIP identity differs")
            shutil.copyfile(replay_zip, archive_path)
        require(common.digest(archive_path) == common.REPLAY_ZIP_SHA, "replay ZIP identity differs")
        with zipfile.ZipFile(archive_path) as archive:
            require(archive.testzip() is None, "replay CRC differs")
            profiles = [n for n in archive.namelist() if n.startswith("half_md4-1/") and n.endswith("/profile.json")]
            require(len(profiles) == 1, "expected one reviewed replay profile")
            prefix = profiles[0].removesuffix("profile.json")
            for entry in archive.infolist():
                if entry.filename.startswith(prefix) and not entry.is_dir():
                    target = common.member_path(DATA / "replay", entry.filename[len(prefix):])
                    target.parent.mkdir(parents=True, exist_ok=True)
                    with archive.open(entry) as stream, target.open("wb") as output:
                        shutil.copyfileobj(stream, output)
        check_replay()
        records = [f"{common.digest(ROOT / os.fsdecode(f))}  {os.fsdecode(f)}\n" for f in tracked_files()]
        (DATA / "source-sha256.txt").write_text("".join(records))
        (DATA / "source-commit.txt").write_text(commit + "\n")
        manifest = {"protocol": "targeted-collision-v1", "source": commit, "workflow_run_id": os.getenv("GITHUB_RUN_ID"),
                    "candidates": rows, "frozen_sha256": FROZEN_SHA, "corpus_sha256": common.CORPUS_SHA,
                    "replay_artifact": common.REPLAY_ARTIFACT, "replay_zip_sha256": common.REPLAY_ZIP_SHA}
        path = DATA / "manifest.json"
        path.write_text(json.dumps(manifest, indent=2) + "\n")
        (DATA / "manifest.sha256").write_text(common.digest(path) + "\n")
    except Exception as error:
        (DATA / "prepare-error.json").write_text(json.dumps({"status": "incomplete", "reason": str(error)}) + "\n")
        raise
    print("targeted preparation complete: 16 frozen candidates plus one exact replay; no trial executed")


def plan():
    path = DATA / "manifest.json"
    require(common.digest(path) == (DATA / "manifest.sha256").read_text().strip(), "prepared manifest changed")
    manifest = json.loads(path.read_text())
    require(manifest["protocol"] == "targeted-collision-v1" and manifest["frozen_sha256"] == FROZEN_SHA and manifest["corpus_sha256"] == common.CORPUS_SHA, "prepared protocol differs")
    require(manifest["candidates"] == frozen() == frozen(DATA / "frozen"), "prepared candidates differ")
    require(manifest["source"] == source(), "source changed since preparation")
    require(manifest["workflow_run_id"] == os.getenv("GITHUB_RUN_ID"), "workflow run changed")
    require(manifest["replay_artifact"] == common.REPLAY_ARTIFACT and manifest["replay_zip_sha256"] == common.REPLAY_ZIP_SHA, "prepared replay identity differs")
    check_replay()
    records = [line.split("  ", 1) for line in (DATA / "source-sha256.txt").read_text().splitlines()]
    require([name for _, name in records] == [os.fsdecode(f) for f in tracked_files()], "prepared source inventory differs")
    for sha, name in records:
        require(common.digest(common.member_path(ROOT, name)) == sha, "prepared source bytes changed")
    return manifest


def layout(profile, trial):
    queries = {q["name_hex"]: q for q in common.read_jsonl(profile / "queries.jsonl")}
    routes = common.read_jsonl(profile / "routes.jsonl")
    observations = []
    for group in trial.get("groups", []):
        major = group[0]["major"]
        leaves = set()
        for member in group:
            query = queries[member["name_hex"]]
            require((query["major"], query["minor"]) == (member["major"], member["minor"]), "predicted collision not independently confirmed")
            require(len(query["actual"]) == 1, "collision name has ambiguous leaf membership")
            leaves.update(query["actual"])
        observations.append({"major": major, "members": group, "actual_leaves": sorted(leaves), "cross_leaf": len(leaves) > 1,
                             "continuation_routes": [r for r in routes if r["hash"] & 1 and r["hash"] & ~1 == major]})
    (profile / "collision-layout.json").write_text(json.dumps({"groups": observations, "note": "observed layout; same-leaf pairs are inconclusive"}, indent=2) + "\n")


def execute(command, environment, seconds):
    child = subprocess.Popen(command, cwd=ROOT, env=environment, start_new_session=True)
    try:
        return child.wait(timeout=seconds)
    except subprocess.TimeoutExpired:
        try:
            os.killpg(child.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        try:
            child.wait(timeout=5)
        except subprocess.TimeoutExpired:
            os.killpg(child.pid, signal.SIGKILL)
            child.wait()
        return 124


def run_batch(batch):
    manifest = plan()
    ids = trial_ids(batch, manifest["candidates"])
    output = DATA / f"batch-{batch}"
    output.mkdir()
    require(shutil.disk_usage(ROOT).free >= len(ids) * 3 * 64 * 1024**2 + 3 * 1024**3, "insufficient space for retained phases and VM/build reserve")
    deadline = time.monotonic() + 40 * 60
    evidence = ROOT / "tmp/htree-descent-evidence/half_md4-1"
    for ident in ids:
        trial = next((r for r in manifest["candidates"] if r["id"] == ident), None)
        if ident == "replay":
            saved = json.loads((DATA / "replay/filesystem.json").read_text())
            trial = {"id": ident, "seed_hex": saved["seed_hex"], "uuid_hex": saved["uuid_hex"]}
        label = "targeted-descent-" + ident
        record = {"trial": trial, "source": manifest["source"], "status": "started"}
        record_file = output / (ident + ".json")
        record_file.write_text(json.dumps(record, indent=2) + "\n")
        before = set(evidence.glob("*"))
        environment = os.environ.copy()
        environment.pop("FS_EXT4_DESCENT_TRIAL", None)
        environment.pop("FS_EXT4_DESCENT_REPLAY_DIR", None)
        environment["FS_EXT4_DESCENT_TARGETED_TRIAL"] = ident
        environment["FS_EXT4_DESCENT_TARGETED_MANIFEST"] = str(DATA / "frozen/manifest.json")
        if ident == "replay":
            environment["FS_EXT4_DESCENT_REPLAY_DIR"] = str(DATA / "replay")
        command = ["scripts/tier.sh", "experiment:" + label, label, "4200", "340000", "--", "scripts/test.sh", "--locked", "--release", "--test", "htree_index_descent_oracle", "half_md4_signed", "--", "--exact", "--show-output"]
        try:
            remaining = deadline - time.monotonic()
            require(remaining > 0, "targeted batch deadline reached")
            record["exit"] = execute(command, environment, remaining)
            profiles = sorted(set(evidence.glob("*")) - before)
            record["captured_profiles"] = []
            for index, profile in enumerate(profiles):
                target = output / "trials" / ident
                if len(profiles) != 1:
                    target = target / str(index)
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.move(str(profile), target)
                record["captured_profiles"].append(str(target.relative_to(DATA)))
            require(record["exit"] == 0, "original test failed or deadline expired; retain exit and witness")
            require(len(profiles) == 1, "expected exactly one newly retained profile")
            subprocess.run(["bash", "scripts/core.sh", "test-floor", label, "1"], cwd=ROOT, check=True)
            record["floor"] = "passed: 1"
            previous = common.DATA
            try:
                common.DATA = DATA
                common.audit_profile(target, trial)
            finally:
                common.DATA = previous
            layout(target, trial)
            record["status"] = "passed"
        except Exception as error:
            record["status"] = "stopped"
            record["reason"] = str(error)
        finally:
            if "captured_profiles" not in record:
                record["captured_profiles"] = []
                for index, profile in enumerate(sorted(set(evidence.glob("*")) - before)):
                    destination = output / "controller-error-profiles" / ident / str(index)
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    shutil.move(str(profile), destination)
                    record["captured_profiles"].append(str(destination.relative_to(DATA)))
            log = ROOT / "tmp/logs" / (label + ".log")
            if log.exists():
                logs = output / "logs"
                logs.mkdir(exist_ok=True)
                shutil.copyfile(log, logs / log.name)
            record_file.write_text(json.dumps(record, indent=2) + "\n")
        if record["status"] != "passed":
            raise SystemExit(record.get("exit", 1) if record.get("exit", 1) > 0 else 1)
    (output / "result.json").write_text(json.dumps({"completed": ids, "status": "failure not reproduced in this finite batch"}) + "\n")
    print(f"targeted batch {batch}: {len(ids)} trials complete; no failing witness reproduced")


def verify_upload(artifact):
    commit = source()
    path = ROOT / "tmp" / f"verified-targeted-artifact-{artifact}.zip"
    metadata = common.download_artifact(artifact, path, commit, newly_uploaded=True)
    if os.getenv("GITHUB_RUN_ID"):
        require(metadata["workflow_run"]["id"] == int(os.environ["GITHUB_RUN_ID"]), "uploaded artifact belongs to another workflow run")
    with zipfile.ZipFile(path) as archive:
        members = [entry for entry in archive.infolist() if not entry.is_dir()]
        require(len({entry.filename for entry in members}) == len(members), "duplicate uploaded archive members")
        local = {str(p.relative_to(DATA)) for p in DATA.rglob("*") if p.is_file()}
        require({entry.filename for entry in members} == local, "uploaded inventory is incomplete or unexpected")
        for entry in members:
            target = common.member_path(DATA, entry.filename)
            with archive.open(entry) as stream:
                actual = hashlib.file_digest(stream, "sha256").hexdigest()
            require(common.digest(target) == actual, "uploaded bytes differ from local evidence")
    print(f"artifact {artifact}: actual SHA-256, CRC, complete inventory and retained bytes verified")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("prepare").add_argument("--replay-zip", type=Path)
    sub.add_parser("batch").add_argument("number", type=int, choices=(0, 1))
    sub.add_parser("verify-upload").add_argument("artifact", type=int)
    args = parser.parse_args()
    if args.command == "prepare":
        prepare(args.replay_zip)
    elif args.command == "batch":
        run_batch(args.number)
    else:
        verify_upload(args.artifact)


if __name__ == "__main__":
    main()
