"""Exercise checked WinFsp release on the pinned disposable Linux image."""

import argparse
import ctypes
import hashlib
import os
import re
import subprocess
import time
from pathlib import Path
from zipfile import ZipFile

from checked_recovery import ARCHIVE_SHA, REFERENCE_SHA

DRIVE = "X:"
DRIVE_MASK = 1 << (ord("X") - ord("A"))
COPY_NAME = "winfsp-checked.bin"
COPY_PAYLOAD = bytes((index * 29 + 7) % 256 for index in range(5123))


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def fixture(archive, output):
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
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


def interrupt(kernel, process):
    if not kernel.FreeConsole() or not kernel.AttachConsole(process.pid):
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        if not kernel.SetConsoleCtrlHandler(None, True):
            raise ctypes.WinError(ctypes.get_last_error())
        if not kernel.GenerateConsoleCtrlEvent(0, 0):
            raise ctypes.WinError(ctypes.get_last_error())
        require(process.wait(timeout=20) == 0, "WinFsp checked release returned an error")
    finally:
        kernel.FreeConsole()


def exercise(executable, image, expected, output):
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    root = Path(DRIVE + "/")
    require(kernel.GetLogicalDrives() & DRIVE_MASK == 0, f"Drive {DRIVE} is already in use")
    transcript = output / "mount.log"
    with transcript.open("wb") as log:
        process = subprocess.Popen(
            [str(executable), "mount", str(image), "--drive", DRIVE, "--checked"],
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
            if f"ext4 mounted at {DRIVE} (RW)" in log and root.is_dir():
                ready = True
                break
            time.sleep(0.25)
        require(ready, "Checked WinFsp mount did not become ready")
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


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("executable", type=Path)
    parser.add_argument("archive", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    image, expected = fixture(args.archive, args.output)
    exercise(args.executable.resolve(strict=True), image, expected, args.output)
    with image.open("rb") as finished:
        finished.seek(1024)
        superblock = finished.read(1024)
    require(len(superblock) == 1024 and superblock[0x60] & 4 == 0 and superblock[0x3A] & 1,
            "Checked release did not leave a clean filesystem superblock")
    verdict = (
        "WinFsp recovered the archived file, wrote and flushed the new file, "
        "and completed checked release.\n"
        f"Image SHA-256 {hashlib.sha256(image.read_bytes()).hexdigest()}\n"
        f"File SHA-256 {hashlib.sha256(COPY_PAYLOAD).hexdigest()}\n"
    )
    (args.output / "verdict.txt").write_text(verdict, encoding="utf-8")
    print(verdict, end="", flush=True)


if __name__ == "__main__":
    main()
