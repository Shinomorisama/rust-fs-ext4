#!/usr/bin/env python3
"""Finite retained-witness experiment; filesystem tools run only through Rust's VM oracle."""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import sys
import urllib.error
import urllib.parse
import urllib.request
import uuid
import zipfile

ROOT = Path(__file__).resolve().parent.parent
DATA = ROOT / "tmp/htree-descent-experiment"
REPO = "Shinomorisama/rust-fs-ext4"
REPLAY_ARTIFACT = 13598975326
REPLAY_SOURCE = "4981c7b67d63a3e8501733f3bdb507f249ebdd59"
REPLAY_ZIP_SHA = "f06cd9efed4eab578d9c276e39b42580fb87afc66bd8734ab227c44287a635c9"
CORPUS_SHA = "8fa9e9ecaaeebae3085ab85d2fc21f4f2946638db6af70c2b3ee0e6ee4b3e30b"
CONFIG_SHA = "7aabbbb9c73d4b7d400d7b74e9d69387c30c87152bb1209194f2bc4952ff19c1"


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def candidates():
    result = []
    for index in range(32):
        row = {"id": f"{index:02d}"}
        for field in ("seed", "uuid"):
            raw = hashlib.sha256(f"casefold-descent-2026-11-11/{field}/{index:02d}".encode("ascii")).digest()[:16]
            row[field + "_hex"] = raw.hex()
            row[field] = str(uuid.UUID(bytes=raw))
        result.append(row)
    for field, replay in (("seed_hex", "6d2c9f2f80bf492db620a3c18010d6c9"), ("uuid_hex", "c0ecd51e9d34484a87bed3c1ac73065d")):
        values = {row[field] for row in result}
        assert len(values) == 32 and "0" * 32 not in values and replay not in values
    return result


def trial_ids(batch):
    if batch not in range(4):
        raise ValueError("batch must be 0..3")
    return (["replay"] if batch == 0 else []) + [f"{i:02d}" for i in range(8 * batch, 8 * batch + 8)]


def member_path(root, name):
    parts = PurePosixPath(name)
    if parts.is_absolute() or ".." in parts.parts or "\\" in name:
        raise ValueError("unsafe ZIP member")
    return root.joinpath(*parts.parts)


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, msg, headers, newurl):
        return None


def api(path, redirect=False):
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    if not token:
        raise RuntimeError("GH_TOKEN is required for the CI artifact request")
    request = urllib.request.Request("https://api.github.com/repos/" + REPO + "/" + path,
                                     headers={"Authorization": "Bearer " + token,
                                              "Accept": "application/vnd.github+json"})
    opener = urllib.request.build_opener(NoRedirect())
    try:
        with opener.open(request, timeout=30) as response:
            if redirect:
                raise RuntimeError("artifact API did not return a storage redirect")
            return json.load(response)
    except urllib.error.HTTPError as error:
        if redirect and error.code == 302:
            location = error.headers["Location"]
            if urllib.parse.urlparse(location).scheme != "https":
                raise RuntimeError("artifact storage redirect must use HTTPS") from None
            return location
        raise RuntimeError(f"GitHub artifact API returned HTTP {error.code}") from None


def download_artifact(artifact, destination, source, expected=None):
    metadata = api(f"actions/artifacts/{artifact}")
    if metadata["expired"] or metadata["workflow_run"]["head_sha"] != source:
        raise RuntimeError("artifact expired or source identity differs")
    advertised = metadata["digest"].removeprefix("sha256:")
    if expected is not None and advertised != expected:
        raise RuntimeError("artifact API digest differs from the fixed replay identity")
    location = api(f"actions/artifacts/{artifact}/zip", redirect=True)
    try:
        # The storage request has no GitHub Authorization header.
        with urllib.request.urlopen(location, timeout=60) as response, destination.open("wb") as output:
            shutil.copyfileobj(response, output)
    except Exception as error:
        raise RuntimeError(f"artifact storage download failed ({type(error).__name__})") from None
    if digest(destination) != advertised:
        raise RuntimeError("downloaded artifact SHA-256 mismatch")
    with zipfile.ZipFile(destination) as archive:
        if archive.testzip() is not None:
            raise RuntimeError("downloaded artifact ZIP CRC mismatch")
    return metadata


def prepare(replay_zip=None):
    DATA.mkdir(parents=True, exist_ok=True)
    manifest_path = DATA / "manifest.json"
    if manifest_path.exists():
        raise RuntimeError("experiment manifest already exists; preserve the earlier attempt")
    source = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    manifest = {"source": source, "candidates": candidates(), "corpus_sha256": CORPUS_SHA,
                "replay_artifact": REPLAY_ARTIFACT, "replay_zip_sha256": REPLAY_ZIP_SHA}
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
    (DATA / "manifest.sha256").write_text(digest(manifest_path) + "\n")
    files = subprocess.check_output(["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"], cwd=ROOT)
    records = [f"{digest(ROOT / os.fsdecode(path))}  {os.fsdecode(path)}\n" for path in sorted(set(files.split(b"\0")) - {b""})]
    (DATA / "source-sha256.txt").write_text("".join(records))
    (DATA / "source-commit.txt").write_text(source + "\n")
    if replay_zip is None:
        replay_zip = DATA / "replay.zip"
        download_artifact(REPLAY_ARTIFACT, replay_zip, REPLAY_SOURCE, REPLAY_ZIP_SHA)
    if digest(replay_zip) != REPLAY_ZIP_SHA:
        raise RuntimeError("replay ZIP is not the reviewed artifact")
    with zipfile.ZipFile(replay_zip) as archive:
        if archive.testzip() is not None:
            raise RuntimeError("replay archive CRC mismatch")
        profiles = [x for x in archive.namelist() if x.startswith("half_md4-1/") and x.endswith("/profile.json")]
        if len(profiles) != 1:
            raise RuntimeError("expected one exact signed half-MD4 replay profile")
        prefix = profiles[0].removesuffix("profile.json")
        for info in archive.infolist():
            if info.filename.startswith(prefix) and not info.is_dir():
                target = member_path(DATA / "replay", info.filename[len(prefix):])
                target.parent.mkdir(parents=True, exist_ok=True)
                with archive.open(info) as stream, target.open("wb") as output:
                    shutil.copyfileobj(stream, output)
    replay = DATA / "replay"
    if digest(replay / "configured.img") != CONFIG_SHA or digest(replay / "names.hex") != CORPUS_SHA:
        raise RuntimeError("replay image or corpus identity differs")
    for phase in ("formatted", "configured", "indexed"):
        if digest(replay / (phase + ".img")) != (replay / (phase + ".sha256")).read_text().strip():
            raise RuntimeError("replay phase digest mismatch")
    print("experiment prepared: fixed replay and all 32 candidate identities retained")


def read_jsonl(path):
    return [json.loads(line) for line in path.read_text().splitlines()]


def check_answers(names, queries, requests, answers):
    if not (len(names) == len(queries) == len(requests) == len(answers)):
        raise RuntimeError("independent hash report is incomplete")
    for name, query, request, (major, minor) in zip(names, queries, requests, answers):
        if query["name_hex"] != name or request.rsplit(b' -- "', 1)[1][:-1].hex() != name:
            raise RuntimeError("query or independent request name differs")
        if not query["found"] or query["selected"] not in query["actual"]:
            raise RuntimeError("retained selected-leaf mismatch")
        if (query["major"], query["minor"]) != (int(major, 16), int(minor, 16)):
            raise RuntimeError("retained independent hash disagreement")


def audit_profile(profile, trial):
    names = (profile / "names.hex").read_text().splitlines()
    queries = read_jsonl(profile / "queries.jsonl")
    settings = json.loads((profile / "filesystem.json").read_text())
    if len(names) != 1500 or len(queries) != 1500 or digest(profile / "names.hex") != CORPUS_SHA:
        raise RuntimeError("trial corpus/query completeness differs")
    if settings["seed_hex"] != trial["seed_hex"] or settings["uuid_hex"] != trial["uuid_hex"] or settings["version"] != 1 or settings["unsigned"]:
        raise RuntimeError("trial identity or signed algorithm differs")
    answers = re.findall(rb"Hash of .*? is 0x([0-9a-fA-F]+) \(minor 0x([0-9a-fA-F]+)\)", (profile / "hashes.stdout").read_bytes())
    requests = (profile / "hash-requests.bin").read_bytes().splitlines()
    if len(answers) != 1500 or len(requests) != 1500:
        raise RuntimeError("independent hash report is incomplete")
    check_answers(names, queries, requests, answers)
    if json.loads((profile / "result.json").read_text())["wrong"] != 0 or (profile / "check.verdict").read_text().strip() != "Clean":
        raise RuntimeError("trial membership or judged consistency verdict differs")
    for phase in ("formatted", "configured", "indexed"):
        if digest(profile / (phase + ".img")) != (profile / (phase + ".sha256")).read_text().strip():
            raise RuntimeError("trial phase digest differs")
    if trial["id"] == "replay":
        # Rebuild the exact saved configured input, then compare semantic records.
        for name in ("routes.jsonl", "mapping.jsonl", "entries.jsonl", "queries.jsonl"):
            if read_jsonl(profile / name) != read_jsonl(DATA / "replay" / name):
                raise RuntimeError("exact-input replay semantic records differ: " + name)


def run_batch(batch):
    manifest_file = DATA / "manifest.json"
    if digest(manifest_file) != (DATA / "manifest.sha256").read_text().strip():
        raise RuntimeError("experiment manifest changed")
    manifest = json.loads(manifest_file.read_text())
    if manifest["candidates"] != candidates() or manifest["corpus_sha256"] != CORPUS_SHA:
        raise RuntimeError("candidate list or corpus differs from the fixed plan")
    source = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    if manifest["source"] != source:
        raise RuntimeError("source changed since experiment preparation")
    ids = trial_ids(batch)
    output = DATA / f"batch-{batch}"
    output.mkdir()  # An earlier attempt must never be overwritten.
    if shutil.disk_usage(ROOT).free < len(ids) * 3 * 64 * 1024**2 + 3 * 1024**3:
        raise RuntimeError("insufficient space for phase images, work copies and VM/build reserve")
    evidence = ROOT / "tmp/htree-descent-evidence/half_md4-1"
    for ident in ids:
        before = set(evidence.glob("*"))
        environment = os.environ.copy()
        environment["FS_EXT4_DESCENT_TRIAL"] = ident
        environment.pop("FS_EXT4_DESCENT_REPLAY_DIR", None)
        if ident == "replay":
            environment["FS_EXT4_DESCENT_REPLAY_DIR"] = str(DATA / "replay")
            saved = json.loads((DATA / "replay/filesystem.json").read_text())
            trial = {"id": ident, "seed_hex": saved["seed_hex"], "uuid_hex": saved["uuid_hex"]}
        else:
            trial = manifest["candidates"][int(ident)]
        label = "descent-" + ident
        record = {"trial": trial, "source": source, "status": "started"}
        record_file = output / (ident + ".json")
        record_file.write_text(json.dumps(record, indent=2) + "\n")
        command = ["scripts/tier.sh", "experiment:" + label, label, "4200", "340000", "--",
                   "scripts/test.sh", "--locked", "--release", "--test", "htree_index_descent_oracle",
                   "half_md4_signed", "--", "--exact", "--show-output"]
        status = subprocess.run(command, cwd=ROOT, env=environment).returncode
        record["exit"] = status
        new_profiles = set(evidence.glob("*")) - before
        if len(new_profiles) == 1:
            target = output / "trials" / ident
            target.parent.mkdir(exist_ok=True)
            shutil.move(str(new_profiles.pop()), target)
            record["evidence"] = str(target.relative_to(DATA))
        try:
            if status != 0:
                raise RuntimeError("original test failed; preserve its exit and witness")
            if len(new_profiles) != 0 or "evidence" not in record:
                raise RuntimeError("expected exactly one newly retained profile")
            subprocess.run(["bash", "scripts/core.sh", "test-floor", label, "1"], cwd=ROOT, check=True)
            record["floor"] = "passed: 1"
            audit_profile(target, trial)
            record["status"] = "passed"
        except Exception as error:
            record["status"] = "stopped"
            record["reason"] = str(error)
        finally:
            logs = output / "logs"
            logs.mkdir(exist_ok=True)
            log = ROOT / "tmp/logs" / (label + ".log")
            if log.exists():
                shutil.copyfile(log, logs / log.name)
            record_file.write_text(json.dumps(record, indent=2) + "\n")
        if record["status"] != "passed":
            raise SystemExit(status if status > 0 else 1)
    (output / "result.json").write_text(json.dumps({"completed": ids, "status": "failure not reproduced in this batch"}) + "\n")
    print(f"batch {batch}: {len(ids)} trials passed; no failing witness reproduced")


def verify_upload(artifact):
    source = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    path = ROOT / "tmp" / f"verified-descent-artifact-{artifact}.zip"
    download_artifact(artifact, path, source)
    with zipfile.ZipFile(path) as archive:
        for info in archive.infolist():
            if not info.is_dir():
                target = member_path(DATA, info.filename)
                with archive.open(info) as stream:
                    actual = hashlib.file_digest(stream, "sha256").hexdigest()
                if digest(target) != actual:
                    raise RuntimeError("uploaded/extracted artifact bytes differ from retained local bytes")
    print(f"artifact {artifact}: actual downloaded SHA-256, CRC and retained bytes verified")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    prepare_parser = sub.add_parser("prepare")
    prepare_parser.add_argument("--replay-zip", type=Path)
    sub.add_parser("batch").add_argument("number", type=int, choices=range(4))
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
