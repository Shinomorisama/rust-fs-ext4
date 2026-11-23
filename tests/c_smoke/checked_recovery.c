#ifndef _WIN32
#define _POSIX_C_SOURCE 200809L
#endif

#include "fs_ext4.h"

#include <assert.h>
#include <errno.h>
#include <stdio.h>
#include <string.h>
#ifdef _WIN32
#include <io.h>
#include <share.h>
#else
#include <unistd.h>
#endif

enum { SECTOR_SIZE = 512 };

typedef struct {
    FILE *file;
    uint64_t size;
    size_t reads;
    size_t writes;
    size_t flushes;
    bool fail_flush;
} image_device;

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

static int seek_image(image_device *image, uint64_t offset, uint64_t length)
{
    if (offset > image->size || length > image->size - offset ||
        offset % SECTOR_SIZE != 0 || length % SECTOR_SIZE != 0) {
        return -1;
    }
    return fseek(image->file, (long)offset, SEEK_SET);
}

static int image_read(void *context, void *buf, uint64_t offset, uint64_t length)
{
    image_device *image = context;
    image->reads++;
    return seek_image(image, offset, length) != 0 ||
        fread(buf, 1, (size_t)length, image->file) != (size_t)length;
}

static int image_write(void *context, const void *buf, uint64_t offset, uint64_t length)
{
    image_device *image = context;
    image->writes++;
    return seek_image(image, offset, length) != 0 ||
        fwrite(buf, 1, (size_t)length, image->file) != (size_t)length;
}

static int image_flush(void *context)
{
    image_device *image = context;
    image->flushes++;
    if (image->fail_flush || fflush(image->file) != 0) {
        return 1;
    }
#ifdef _WIN32
    return _commit(_fileno(image->file)) != 0;
#else
    return fsync(fileno(image->file)) != 0;
#endif
}

static void check_payload(fs_ext4_fs_t *fs, const char *expected_path)
{
    unsigned char payload[4096], expected[4096];
    read_bytes(expected_path, 0, expected, sizeof expected);
    assert(fs_ext4_read_file(fs, "/oracle", payload, 0, sizeof payload) == (int64_t)sizeof payload);
    assert(memcmp(payload, expected, sizeof payload) == 0);
}

static void check_callbacks(const char *path, const char *payload, bool fail_finish)
{
    image_device image = {0};
#ifdef _WIN32
    image.file = _fsopen(path, "r+b", _SH_DENYRW);
#else
    image.file = fopen(path, "r+b");
#endif
    assert(image.file != NULL);
    assert(setvbuf(image.file, NULL, _IONBF, 0) == 0);
    assert(fseek(image.file, 0, SEEK_END) == 0);
    long size = ftell(image.file);
    assert(size > 0);
    image.size = (uint64_t)size;
    fs_ext4_blockdev_cfg_t cfg = {
        .read = image_read,
        .context = &image,
        .size_bytes = image.size,
        .block_size = SECTOR_SIZE,
        .write = image_write,
        .flush = NULL,
    };
    assert(fs_ext4_mount_recovering_with_callbacks(&cfg) == NULL);
    assert(fs_ext4_last_errno() == EINVAL);
    assert(image.reads == 0 && image.writes == 0 && image.flushes == 0);

    cfg.flush = image_flush;
    fs_ext4_fs_t *fs = fs_ext4_mount_recovering_with_callbacks(&cfg);
    if (fs == NULL) {
        fprintf(stderr, "%s\n", fs_ext4_last_error());
    }
    assert(fs != NULL);
    assert(image.reads > 0 && image.writes > 0 && image.flushes > 0);
    check_payload(fs, payload);
    size_t flushes = image.flushes;
    assert(fs_ext4_flush(fs) == 0);
    assert(image.flushes > flushes);

    unsigned char incompat;
    assert(fseek(image.file, 1024 + 0x60, SEEK_SET) == 0);
    assert(fread(&incompat, 1, 1, image.file) == 1 && (incompat & 4) != 0);
    image.fail_flush = fail_finish;
    if (fail_finish) {
        assert(fs_ext4_finish(fs) == -1);
        assert(fs_ext4_last_errno() == EIO);
        assert(strstr(fs_ext4_last_error(), "flush callback returned 1") != NULL);
        assert(fseek(image.file, 1024 + 0x60, SEEK_SET) == 0);
        assert(fread(&incompat, 1, 1, image.file) == 1 && (incompat & 4) != 0);
        puts("Callback finish reported the flush failure and retained the recovery marker");
    } else {
        assert(fs_ext4_finish(fs) == 0);
    }
    assert(fclose(image.file) == 0);
    if (!fail_finish) {
        check_state(path, 0, 1);
        puts("Callback mount, live flush and checked finish passed");
    }
}

int main(int argc, char **argv)
{
    assert(argc == 6);
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
    check_payload(fs, argv[3]);
    assert(fs_ext4_flush(fs) == 0);
    check_state(argv[1], 1, 0);
    puts("Committed payload recovered and recovery marker retained through live flush");

    assert(fs_ext4_finish(fs) == 0);
    check_state(argv[1], 0, 1);
    puts("Checked finish cleared the recovery marker");
    check_callbacks(argv[4], argv[3], false);
    check_callbacks(argv[5], argv[3], true);
    return 0;
}
