/* DVD shims: read the user's disc image directly.
 *
 * Aurora's DVD support is compiled out of this build (AURORA_ENABLE_DVD=OFF), and the port needs
 * the GameCube's own path/file semantics anyway, so this implements the small surface melee uses on
 * top of a plain ISO image.
 *
 * Disc layout used here: the boot block holds the DOL offset at 0x420 and the FST offset/size at
 * 0x424/0x428 (all big-endian). The FST begins with the root node -- { u8 kind; u24 name offset;
 * u32 offset; u32 length }, 12 bytes, kind 0 = file, 1 = directory. For a directory, `offset` is
 * the parent index and `length` is the index one past its children; for a file they are the
 * partition offset and byte size. The name table follows the last node, i.e. at root.length()*12,
 * and node name offsets are relative to it.
 *
 * Read completions are queued through gw_defer rather than called inline: devcom.c sets its
 * in-flight flag *after* the DVDReadAsyncPrio call, so a synchronous callback would observe a
 * half-updated request. gw_wait_idle (called from DVDGetDriveStatus) and the frame tick both drain
 * the queue. */
#define _CRT_SECURE_NO_WARNINGS
#include "gw.h"
#include "shim_vi.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define GW_DVD_FST_OFFSET_FIELD 0x424
#define GW_DVD_FST_SIZE_FIELD 0x428
#define GW_DVD_DISK_ID_SIZE 32
#define GW_DVD_STATE_END 0
#define GW_DVD_NODE_SIZE 12

static FILE *gw_iso;
static unsigned char *gw_fst;
static uint32_t gw_fst_size;
static uint32_t gw_fst_nodes;
static uint32_t gw_fst_names;
static unsigned char gw_disk_id[GW_DVD_DISK_ID_SIZE];
static bool gw_iso_attempted;

static uint32_t gw_be32(const unsigned char *p) {
  return ((uint32_t)p[0] << 24) | ((uint32_t)p[1] << 16) | ((uint32_t)p[2] << 8) | (uint32_t)p[3];
}

static bool gw_iso_open(void) {
  if (gw_iso_attempted) {
    return gw_iso != NULL;
  }
  gw_iso_attempted = true;

  const char *path = gw_iso_path();
  if (path == NULL) {
    gw_log("gw: DVD used before a disc image was found");
    return false;
  }
  gw_iso = fopen(path, "rb");
  if (gw_iso == NULL) {
    gw_log("gw: cannot open disc image %s", path);
    return false;
  }

  unsigned char boot[GW_DVD_FST_SIZE_FIELD + 4];
  if (fread(boot, 1, sizeof boot, gw_iso) != sizeof boot) {
    gw_log("gw: disc image %s is too short", path);
    fclose(gw_iso);
    gw_iso = NULL;
    return false;
  }
  memcpy(gw_disk_id, boot, sizeof gw_disk_id);

  const uint32_t fst_offset = gw_be32(boot + GW_DVD_FST_OFFSET_FIELD);
  const uint32_t fst_size = gw_be32(boot + GW_DVD_FST_SIZE_FIELD);
  gw_fst = (unsigned char *)malloc(fst_size);
  if (gw_fst == NULL || fst_size < GW_DVD_NODE_SIZE) {
    gw_log("gw: cannot allocate %u bytes for the disc FST", fst_size);
    fclose(gw_iso);
    gw_iso = NULL;
    return false;
  }
  if (fseek(gw_iso, (long)fst_offset, SEEK_SET) != 0 ||
      fread(gw_fst, 1, fst_size, gw_iso) != fst_size) {
    gw_log("gw: cannot read the disc FST (%u bytes at %u)", fst_size, fst_offset);
    free(gw_fst);
    gw_fst = NULL;
    fclose(gw_iso);
    gw_iso = NULL;
    return false;
  }

  gw_fst_nodes = gw_be32(gw_fst + 8);
  gw_fst_names = gw_fst_nodes * GW_DVD_NODE_SIZE;
  gw_fst_size = fst_size;
  if (gw_fst_nodes == 0 || gw_fst_names + GW_DVD_NODE_SIZE > fst_size) {
    gw_log("gw: disc FST looks corrupt (root length %u, size %u)", gw_fst_nodes, fst_size);
    gw_fst_nodes = 0;
  }
  gw_log("gw: disc image %s, disk id %.6s, FST %u nodes", path, gw_disk_id, gw_fst_nodes);
  return true;
}

static int gw_fst_kind(uint32_t index) { return gw_fst[index * GW_DVD_NODE_SIZE]; }

static uint32_t gw_fst_name_offset(uint32_t index) {
  const unsigned char *p = gw_fst + index * GW_DVD_NODE_SIZE + 1;
  return ((uint32_t)p[0] << 16) | ((uint32_t)p[1] << 8) | (uint32_t)p[2];
}

static uint32_t gw_fst_offset(uint32_t index) {
  return gw_be32(gw_fst + index * GW_DVD_NODE_SIZE + 4);
}

static uint32_t gw_fst_length(uint32_t index) {
  return gw_be32(gw_fst + index * GW_DVD_NODE_SIZE + 8);
}

static const char *gw_fst_name(uint32_t index) {
  const uint32_t offset = gw_fst_name_offset(index);
  if (index >= gw_fst_nodes || gw_fst_names + offset >= gw_fst_size) {
    return "";
  }
  return (const char *)(gw_fst + gw_fst_names + offset);
}

static bool gw_name_matches(const char *name, const char *component, size_t length) {
  return strlen(name) == length && strncmp(name, component, length) == 0;
}

void gw_DVDInit(void) { (void)gw_iso_open(); }

int gw_DVDGetDriveStatus(void) {
  gw_wait_idle();
  return GW_DVD_STATE_END;
}

int gw_DVDCheckDisk(void) { return 1; }

void *gw_DVDGetCurrentDiskID(void) { return gw_disk_id; }

int gw_DVDConvertPathToEntrynum(const char *path) {
  if (!gw_iso_open() || gw_fst_nodes == 0 || path == NULL) {
    return -1;
  }

  const char *component = path;
  if (*component == '/') {
    ++component;
  }
  if (*component == '\0') {
    return 0;
  }

  uint32_t end = gw_fst_length(0);
  uint32_t index = 1;
  for (;;) {
    const char *slash = strchr(component, '/');
    const size_t length = slash != NULL ? (size_t)(slash - component) : strlen(component);
    bool found = false;

    while (index < end && index < gw_fst_nodes) {
      if (gw_name_matches(gw_fst_name(index), component, length)) {
        if (slash == NULL) {
          return (int)index;
        }
        if (gw_fst_kind(index) != 1) {
          return -1;
        }
        end = gw_fst_length(index);
        index = index + 1;
        found = true;
        break;
      }
      index = gw_fst_kind(index) == 1 ? gw_fst_length(index) : index + 1;
    }

    if (!found || slash == NULL) {
      return -1;
    }
    component = slash + 1;
  }
}

int gw_DVDFastOpen(int entrynum, void *file_info) {
  if (!gw_iso_open() || gw_fst_nodes == 0 || entrynum < 0 || (uint32_t)entrynum >= gw_fst_nodes) {
    return 0;
  }
  if (gw_fst_kind((uint32_t)entrynum) != 0) {
    return 0;
  }

  const uint32_t offset = gw_fst_offset((uint32_t)entrynum);
  const uint32_t length = gw_fst_length((uint32_t)entrynum);
  unsigned char *info = (unsigned char *)file_info;
  gw_w32(info + 0x0C, GW_DVD_STATE_END);
  gw_w32(info + 0x10, 0);
  gw_w32(info + 0x14, length);
  gw_w32(info + 0x30, offset);
  gw_w32(info + 0x34, length);
  return 1;
}

int gw_DVDClose(void *file_info) {
  (void)file_info;
  return 1;
}

static void gw_dvd_complete(void *file_info, void *callback, uint32_t result) {
  if (callback != NULL) {
    ((void (*)(int, void *))callback)((int)(int32_t)result, file_info);
  }
}

int gw_DVDReadAsyncPrio(void *file_info, void *addr, int length, int offset, void *callback,
                        int prio) {
  (void)prio;

  uint32_t result = 0;
  if (gw_iso_open()) {
    const unsigned char *info = (const unsigned char *)file_info;
    const uint32_t file_offset = gw_r32(info + 0x30);
    const uint32_t file_length = gw_r32(info + 0x34);
    uint32_t want = (uint32_t)length;

    if (offset < 0 || (uint32_t)offset >= file_length) {
      want = 0;
    } else if (want > file_length - (uint32_t)offset) {
      want = file_length - (uint32_t)offset;
    }

    /* fread rejects a NULL buffer through the CRT's invalid-parameter path, which calls
     * __fastfail and takes the process down with STATUS_STACK_BUFFER_OVERRUN -- bypassing SEH, so
     * no crash handler runs and nothing is logged. A NULL destination here means a game
     * allocation failed upstream, which is worth reporting as itself rather than as a silent
     * disappearance. */
    if (want != 0 && addr == NULL) {
      gw_log("gw: DVDReadAsyncPrio with a NULL destination (%u bytes at offset %d) - the caller's "
             "allocation failed",
             want, offset);
      want = 0;
      result = (uint32_t)-1;
    }

    if (want != 0 &&
        (fseek(gw_iso, (long)(file_offset + (uint32_t)offset), SEEK_SET) != 0 ||
         fread(addr, 1, want, gw_iso) != want)) {
      result = (uint32_t)-1;
    }
  } else {
    result = (uint32_t)-1;
  }

  gw_defer(gw_dvd_complete, file_info, callback, result);
  return 1;
}
