"""Exercise checked WinFsp release or refusal on a pinned Linux image."""

import argparse
from contextlib import contextmanager
import ctypes
import hashlib
import os
import re
import shutil
import subprocess
import time
from pathlib import Path
from zipfile import ZipFile

from checked_recovery import ARCHIVE_SHA, REFERENCE_SHA

DRIVE = "X:"
DRIVE_MASK = 1 << (ord("X") - ord("A"))
COPY_NAME = "winfsp-checked.bin"
COPY_PAYLOAD = bytes((index * 29 + 7) % 256 for index in range(5123))
CASEFOLD_ARCHIVE_SHA = "23e943bf9b34b026196fa2aeecb4ea6c9e4f2a528b02d424bf39de1dabff9e18"
CASEFOLD_IMAGE_SHA = "a0dabe3454a67b1ba1fe7bb4f880974a518501cb23f419f34886a4d29a5f8b3e"
CASEFOLD_FILE_SHA = "7beb5db28751ba43973ea7d20f08dffa3f2bc7d8015c5b9095236f3ca8820562"


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def sha256(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def fixture(archive, output):
    digest = sha256(archive)
    require(digest == ARCHIVE_SHA, f"Linux archive digest mismatch: {digest}")
    image = output / "checked.img"
    with ZipFile(archive) as saved:
        source = saved.read("checked-recovery-reference/source-commit.txt").decode().strip()
        require(source == REFERENCE_SHA, f"Linux reference source mismatch: {source}")
        member, = (name for name in saved.namelist() if re.search(r"_input_\d+\.img$", name))
        with image.open("xb") as working:
            working.write(saved.read(member))
        payload = saved.read(member + ".payload.bin")
    return image, payload


def casefold_fixture(archive, output):
    require(sha256(archive) == CASEFOLD_ARCHIVE_SHA, "Casefold fixture archive digest mismatch")
    image = output / "casefold-4096-strict.img"
    member = f"casefold-stage0/{image.name}"
    with ZipFile(archive) as saved:
        require(saved.namelist().count(member) == 1, "Expected one pinned casefold fixture")
        with saved.open(member) as source, image.open("xb") as destination:
            shutil.copyfileobj(source, destination)
    require(sha256(image) == CASEFOLD_IMAGE_SHA, "Casefold fixture image digest mismatch")
    return image


def interrupt(kernel, process):
    if not kernel.FreeConsole() or not kernel.AttachConsole(process.pid):
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        if not kernel.SetConsoleCtrlHandler(None, True):
            raise ctypes.WinError(ctypes.get_last_error())
        if not kernel.GenerateConsoleCtrlEvent(0, 0):
            raise ctypes.WinError(ctypes.get_last_error())
        require(process.wait(timeout=20) == 0, "WinFsp release returned an error")
    finally:
        kernel.FreeConsole()


@contextmanager
def mounted(kernel, command, access, transcript):
    root = Path(DRIVE + "/")
    require(kernel.GetLogicalDrives() & DRIVE_MASK == 0, f"Drive {DRIVE} is already in use")
    with transcript.open("wb") as log:
        process = subprocess.Popen(
            command,
            stdin=subprocess.DEVNULL,
            stdout=log,
            stderr=subprocess.STDOUT,
            creationflags=subprocess.CREATE_NEW_CONSOLE,
        )
    ready = False
    try:
        deadline = time.monotonic() + 45
        while time.monotonic() < deadline:
            require(process.poll() is None, f"WinFsp exited before mount ({process.returncode})")
            log = transcript.read_text(encoding="utf-8", errors="replace")
            if f"ext4 mounted at {DRIVE} ({access})" in log and root.is_dir():
                ready = True
                break
            time.sleep(0.25)
        require(ready, f"WinFsp {access} mount did not become ready")
        yield root
    finally:
        try:
            if ready and process.poll() is None:
                interrupt(kernel, process)
        finally:
            if process.poll() is None:
                process.kill()
                process.wait(timeout=10)
            print(transcript.read_text(encoding="utf-8", errors="replace"), end="", flush=True)

    require(process.returncode == 0, f"WinFsp exited with status {process.returncode}")
    require(kernel.GetLogicalDrives() & DRIVE_MASK == 0, "WinFsp drive is still mounted")


def exercise(executable, image, expected, output):
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    command = [str(executable), "mount", str(image), "--drive", DRIVE, "--checked"]
    with mounted(kernel, command, "RW", output / "mount.log") as root:
        try:
            with image.open("rb"):
                raise RuntimeError("A second handle opened the image during the checked mount")
        except PermissionError:
            pass
        require((root / "oracle").read_bytes() == expected, "Replayed payload differs")
        copy = root / COPY_NAME
        with copy.open("xb") as handle:
            require(handle.write(COPY_PAYLOAD) == len(COPY_PAYLOAD), "Short WinFsp write")
            handle.flush()
            os.fsync(handle.fileno())
        require(copy.read_bytes() == COPY_PAYLOAD, "WinFsp read-back differs")
        (output / COPY_NAME).write_bytes(COPY_PAYLOAD)


def exercise_casefold(executable, image, output):
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    require(kernel.GetLogicalDrives() & DRIVE_MASK == 0, f"Drive {DRIVE} is already in use")
    command = [str(executable), "mount", str(image), "--drive", DRIVE]
    refused = subprocess.run(
        [*command, "--checked"],
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        errors="replace",
        timeout=15,
    )
    (output / "refusal.log").write_text(refused.stdout, encoding="utf-8")
    print(refused.stdout, end="", flush=True)
    require(refused.returncode == 1, f"Unexpected checked mount exit status {refused.returncode}")
    require("ext4 mounted at" not in refused.stdout, "Checked casefold mount reported ready")
    expected_error = "unsupported INCOMPAT features: 0x00020000"
    require(
        any(line.startswith("Error: checked mount ") and line.endswith(expected_error)
            for line in refused.stdout.splitlines()),
        "Checked mount did not report the casefold admission error",
    )
    require(kernel.GetLogicalDrives() & DRIVE_MASK == 0, "Refusal left a drive mounted")
    after_refusal = sha256(image)
    require(after_refusal == CASEFOLD_IMAGE_SHA, "Checked refusal changed the Linux fixture")
    print(f"Checked mount refused casefold (exit 1). Image SHA-256 {after_refusal}", flush=True)

    with mounted(kernel, [*command, "--ro"], "RO", output / "mount.log") as root:
        directory = root / "fold_small"
        names = sorted(entry.name for entry in directory.iterdir())
        (output / "names.txt").write_text("\n".join(names) + "\n", encoding="utf-8")
        require("ReadMe" in names and "README" not in names, f"Unexpected stored spelling: {names}")
        payload = (directory / "README").read_bytes()
        require(hashlib.sha256(payload).hexdigest() == CASEFOLD_FILE_SHA, "Alias payload differs")
        (output / "alias.bin").write_bytes(payload)

    after_release = sha256(image)
    require(after_release == CASEFOLD_IMAGE_SHA, "Read-only mount changed the Linux fixture")
    return (
        f"Input image SHA-256 {CASEFOLD_IMAGE_SHA}\n"
        f"Checked refusal exit {refused.returncode}; drive absent\n"
        f"Refused image SHA-256 {after_refusal}\n"
        f"Listed ReadMe and read README with SHA-256 {CASEFOLD_FILE_SHA}\n"
        f"Read-only release exit 0; drive absent\n"
        f"Released image SHA-256 {after_release}\n"
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("executable", type=Path)
    parser.add_argument("archive", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--casefold", action="store_true")
    args = parser.parse_args()
    executable = args.executable.resolve(strict=True)
    if args.casefold:
        image = casefold_fixture(args.archive, args.output)
        verdict = exercise_casefold(executable, image, args.output)
    else:
        verdict = checked_release(executable, args.archive, args.output)
    (args.output / "verdict.txt").write_text(verdict, encoding="utf-8")
    print(verdict, end="", flush=True)


def checked_release(executable, archive, output):
    image, expected = fixture(archive, output)
    exercise(executable, image, expected, output)
    with image.open("rb") as finished:
        finished.seek(1024)
        superblock = finished.read(1024)
    require(len(superblock) == 1024 and superblock[0x60] & 4 == 0 and superblock[0x3A] & 1,
            "Checked release did not leave a clean filesystem superblock")
    return (
        "WinFsp recovered the archived file, wrote and flushed the new file, "
        "and completed checked release.\n"
        f"Image SHA-256 {sha256(image)}\n"
        f"File SHA-256 {hashlib.sha256(COPY_PAYLOAD).hexdigest()}\n"
    )


if __name__ == "__main__":
    main()
