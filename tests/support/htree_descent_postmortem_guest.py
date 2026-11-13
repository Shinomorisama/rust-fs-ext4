"""Cold observations only; invoked through the read-only kernel VM helper."""
import errno
import os
import stat
import sys

root = os.fsencode(sys.argv[1])
order = int(sys.argv[2])
assert order in (0, 1)
assert os.statvfs(root).f_flag & os.ST_RDONLY, 'mount is not read-only'
print('readonly|1')
pair = [(b'a_name_of_twenty_00121', 337), ('café_863'.encode(), 882)]
if order:
    pair.reverse()
# No readdir or lookup of either name precedes these observations. Each
# invocation receives a new guest copy, loop device and read-only mount.
for name, expected in pair:
    path = root + b'/bigdir/' + name
    found = os.lstat(path)
    assert found.st_ino == expected and found.st_size == 0
    assert stat.S_ISREG(found.st_mode)
    descriptor = os.open(path, os.O_RDONLY | os.O_NOFOLLOW)
    try:
        opened = os.fstat(descriptor)
        assert opened.st_ino == found.st_ino and opened.st_size == 0
        assert os.read(descriptor, 1) == b''
    finally:
        os.close(descriptor)
    print(f'{name.hex()}|{found.st_ino}|{found.st_size}')
try:
    os.lstat(root + b'/bigdir/postmortem-missing-control')
except OSError as error:
    assert error.errno == errno.ENOENT
    print('missing|ENOENT')
else:
    raise AssertionError('absent-name control unexpectedly exists')
print('kernel|' + os.uname().release)
