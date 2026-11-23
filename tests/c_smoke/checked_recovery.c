#include "fs_ext4.h"

#include <assert.h>
#include <errno.h>
#include <stdio.h>
#include <string.h>

static void read_bytes(const char *path, long offset, void *bytes, size_t length)
{
    FILE *file = fopen(path, "rb");
    assert(file != NULL);
    assert(fseek(file, offset, SEEK_SET) == 0);
    assert(fread(bytes, 1, length, file) == length);
    assert(fclose(file) == 0);
}

static void check_state(const char *path, int recover, int clean)
{
    unsigned char superblock[1024];
    read_bytes(path, 1024, superblock, sizeof superblock);
    assert(((superblock[0x60] & 4) != 0) == recover);
    assert(((superblock[0x3a] & 1) != 0) == clean);
}

int main(int argc, char **argv)
{
    assert(argc == 4);
    assert(fs_ext4_mount_recovering(argv[2]) == NULL);
    assert(fs_ext4_last_errno() == EIO);
    assert(strstr(fs_ext4_last_error(), "journal superblock checksum mismatch") != NULL);
    puts("Damaged checksum refused");

    check_state(argv[1], 1, 1);
    fs_ext4_fs_t *fs = fs_ext4_mount_recovering(argv[1]);
    if (fs == NULL) {
        fprintf(stderr, "%s\n", fs_ext4_last_error());
        return 1;
    }
    check_state(argv[1], 1, 0);
    unsigned char payload[4096], expected[4096];
    read_bytes(argv[3], 0, expected, sizeof expected);
    assert(fs_ext4_read_file(fs, "/oracle", payload, 0, sizeof payload) == (int64_t)sizeof payload);
    assert(memcmp(payload, expected, sizeof payload) == 0);
    assert(fs_ext4_flush(fs) == 0);
    check_state(argv[1], 1, 0);
    puts("Committed payload recovered and recovery marker retained through live flush");

    assert(fs_ext4_finish(fs) == 0);
    check_state(argv[1], 0, 1);
    puts("Checked finish cleared the recovery marker");
    return 0;
}
