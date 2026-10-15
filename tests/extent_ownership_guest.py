"""Linux-written ownership controls and FIEMAP observations, inside the VM.

Published interface: https://docs.kernel.org/filesystems/fiemap.html
The fixed Linux ioctl ABI has a 32-byte request followed by 56-byte records;
reserved fields occupy the gaps in the published logical/physical/length/flags.
"""

import ctypes
import fcntl
import os
import struct
import sys

block_size = int(sys.argv[1])
root = os.environ["MNT"]
libc = ctypes.CDLL(None, use_errno=True)
libc.fallocate.argtypes = [ctypes.c_int, ctypes.c_int, ctypes.c_int64, ctypes.c_int64]
libc.fallocate.restype = ctypes.c_int

for name in ["inline", "external"]:
    fd = os.open(os.path.join(root, name), os.O_RDWR | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        positions = [0] if name == "inline" else [16 + i * 32 for i in range(16)]
        for logical in positions:
            assert os.pwrite(fd, b"x" * block_size, logical * block_size) == block_size
        if name == "external":
            # KEEP_SIZE must preserve allocated storage beyond visible EOF.
            result = libc.fallocate(fd, 1, 1024 * block_size, 3 * block_size)
            assert result == 0, ctypes.get_errno()
        os.fsync(fd)
        capacity = 128
        request = bytearray(32 + 56 * capacity)
        struct.pack_into("=QQIIII", request, 0, 0, (1 << 64) - 1, 1, 0, capacity, 0)
        # FS_IOC_FIEMAP; SYNC=1, LAST=1, UNWRITTEN=0x800.
        fcntl.ioctl(fd, 0xC020660B, request, True)
        count = struct.unpack_from("=I", request, 20)[0]
        assert 0 < count < capacity
        rows = []
        for number in range(count):
            offset = 32 + number * 56
            logical, physical, length = struct.unpack_from("=QQQ", request, offset)
            flags = struct.unpack_from("=I", request, offset + 40)[0]
            assert flags & ~(1 | 0x800) == 0, hex(flags)
            assert logical % block_size == physical % block_size == length % block_size == 0
            assert length > 0
            rows.append((logical // block_size, physical // block_size,
                         length // block_size, int(bool(flags & 0x800))))
        assert flags & 1
        stat = os.fstat(fd)
        assert stat.st_size == (positions[-1] + 1) * block_size
        if name == "external":
            assert len(rows) > 4
            assert any(logical >= 1024 and unwritten for logical, _, _, unwritten in rows)
        print("INODE", name, stat.st_ino, stat.st_blocks, stat.st_size)
        for row in rows:
            print("EXTENT", name, *row)
    finally:
        os.close(fd)
