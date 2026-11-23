"""Run the C checked-recovery probe against the pinned Linux VM reference."""

import argparse
import hashlib
import re
import subprocess
from pathlib import Path
from zipfile import ZipFile

ROOT = Path(__file__).resolve().parents[2]
REFERENCE_SHA = "8a06dd44e92ca256e1e5eb18d190edecc8c14093"
ARCHIVE_SHA = "b484c04c149d6604a8ee674ffb678ca85f4b5fb0df7ad5c0f7ea9d38e897b4e1"
BLOCK_SIZE = 4096


def build_probe(output):
    target = "x86_64-pc-windows-msvc"
    build = ROOT / "target" / "checked-windows"
    library = build / target / "release" / "fs_ext4.lib"
    completed = subprocess.run(
        ["cargo", "rustc", "--locked", "--release", "--lib",
         "--target", target, "--target-dir", str(build),
         "--", "--print", "native-static-libs"],
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    (output / "native-static-libs.log").write_text(completed.stdout, encoding="utf-8")
    print(completed.stdout, end="", flush=True)
    completed.check_returncode()
    libs, = re.findall(r"^note: native-static-libs: (.*)$", completed.stdout, re.M)
    executable = output / "checked-recovery.exe"
    subprocess.run(
        ["cl", "/nologo", "/TC", "/std:c11", "/W4", "/WX", "/MD", "/D_CRT_SECURE_NO_WARNINGS",
         f"/I{ROOT / 'include'}", str(Path(__file__).with_suffix(".c").resolve()),
         f"/Fe:{executable}", "/link", str(library), *libs.split()],
        cwd=output,
        check=True,
    )
    return executable


def exercise(archive, output, executable):
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    if digest != ARCHIVE_SHA:
        raise ValueError(f"Linux reference archive digest mismatch: {digest}")
    with ZipFile(archive) as fixture:
        source = fixture.read("checked-recovery-reference/source-commit.txt").decode().strip()
        if source != REFERENCE_SHA:
            raise ValueError(f"Linux reference source mismatch: {source}")
        base, = (name for name in fixture.namelist() if re.search(r"_input_\d+\.img$", name))
        working = output / "checked.img"
        refused = output / "refused.img"
        payload = output / "payload.bin"
        callback = output / "callback-checked.img"
        failed = output / "callback-failed.img"
        original = fixture.read(base)
        for image in (working, callback, failed):
            image.write_bytes(original)
        del original
        original_refusal = fixture.read(base + ".refused.img")
        refused.write_bytes(original_refusal)
        payload.write_bytes(fixture.read(base + ".payload.bin"))
        result = subprocess.run(
            [str(executable), working.name, refused.name, payload.name,
             callback.name, failed.name],
            cwd=output,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            timeout=45,
        )
        (output / "probe.log").write_text(result.stdout, encoding="utf-8")
        print(result.stdout, end="", flush=True)
        result.check_returncode()
        if refused.read_bytes() != original_refusal:
            raise ValueError("Checksum refusal changed the image")
        linux = fixture.read(base + ".linux.img")
        checked = working.read_bytes()
        if memoryview(checked)[BLOCK_SIZE:] != memoryview(linux)[BLOCK_SIZE:]:
            raise ValueError("Checked finish differs from Linux outside block zero")
        release_failed = failed.read_bytes()
        callback_checked = callback.read_bytes()
        if memoryview(callback_checked)[BLOCK_SIZE:] != memoryview(linux)[BLOCK_SIZE:]:
            raise ValueError("Callback finish differs from Linux outside block zero")
        if memoryview(release_failed)[BLOCK_SIZE:] != memoryview(linux)[BLOCK_SIZE:]:
            raise ValueError("Failed callback finish changed replayed data or journal state")
        if callback_checked[1024 + 0x60] & 4 or not release_failed[1024 + 0x60] & 4:
            raise ValueError("Callback finish did not record the expected recovery markers")
        verdict = (
            f"Reference {source} archive {digest}\n"
            "Damaged image unchanged and checked image matches Linux outside block zero\n"
            f"Finished image SHA-256 {hashlib.sha256(checked).hexdigest()}\n"
            f"Callback finish SHA-256 {hashlib.sha256(callback_checked).hexdigest()}\n"
            f"Failed callback finish SHA-256 {hashlib.sha256(release_failed).hexdigest()}\n"
        )
        (output / "verdict.txt").write_text(verdict, encoding="utf-8")
        print(verdict, end="", flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--probe", type=Path, help="use an already linked executable")
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    if args.probe is None:
        source = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True)
        (output / "source-commit.txt").write_text(source, encoding="utf-8")
    probe = args.probe.resolve(strict=True) if args.probe else build_probe(output)
    exercise(args.archive, output, probe)


if __name__ == "__main__":
    main()
