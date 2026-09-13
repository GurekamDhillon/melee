/* CARD shims.
 *
 * These forward to Aurora's kabufuda-based memory card implementation (extern/aurora/lib/card),
 * which provides the full Dolphin CARD API against either a raw card image or a folder of GCI
 * files. The card lives next to the executable under "card/" unless MELEE_CARD_PATH says
 * otherwise; set MELEE_CARD_RAW=1 to use a single raw image instead of the GCI folder.
 *
 * Two boundaries need care:
 *
 *   - Endianness. Aurora fills CARDFileInfo and CARDStat natively, but the game reads them
 *     big-endian, so every multi-byte field is marshalled here rather than handed across
 *     directly. Character arrays (file names, game/company codes) are bytes and cross unchanged.
 *
 *   - Completion. Aurora's async entry points run the work and then call the callback before
 *     returning. The game does not expect that: lbcardnew starts a task, returns to its own
 *     state machine, and spins until the completion arrives, so a callback delivered inside the
 *     call is overwritten by the state the game sets immediately afterwards and the spin never
 *     ends. So the work is done synchronously here and the game's callback is queued with
 *     gw_defer, which pumps it from gw_wait_idle / gw_frame_tick -- the same treatment ARQ and
 *     DVD completions already get, and the same order the console produces.
 *
 * If the card cannot be initialised, every entry reports CARD_RESULT_NOCARD (-3) exactly as the
 * previous stub did, which is a path the game already handles: lbCardGame_DecideGameMode takes
 * the no-card branch and the card scene is never entered. CARDProbeEx must never return
 * CARD_RESULT_BUSY (-1) in that case, because db_GetGameLaunchButtonState spins on it.
 */
#include "gw.h"

#include "shim_vi.h"

#include <dolphin/card.h>

#include <stdlib.h>
#include <string.h>

#define WIN32_LEAN_AND_MEAN
#include <windows.h>

#define CARD_RESULT_READY (0)
#define CARD_RESULT_NOCARD (-3)

/* Melee's disc id and maker code; Aurora uses these to name and validate the card files. */
#define GW_CARD_GAME "GALE"
#define GW_CARD_MAKER "01"

static int gw_card_ready;

/* Card traffic is low-frequency and the card paths are new, so log every call and its result
 * until they are trusted. Off unless MELEE_CARD_DIAG=1. */
static int gw_card_diag(void) {
  static int cached = -1;
  if (cached < 0) {
    const char *v = getenv("MELEE_CARD_DIAG");
    cached = (v != NULL && v[0] == '1') ? 1 : 0;
  }
  return cached;
}

#define GW_CARD_LOG(...)                                                                             do {                                                                                                 if (gw_card_diag()) {                                                                                gw_log(__VA_ARGS__);                                                                             }                                                                                                } while (0)

/* ---- endianness marshalling -------------------------------------------------------------- */

/* CARDFileInfo: four s32 and a u16, all game-visible. */
static void gw_card_info_to_game(void *dst, const CARDFileInfo *src) {
  unsigned char *p = (unsigned char *)dst;
  if (p == NULL) {
    return;
  }
  gw_w32(p + 0x0, (uint32_t)src->chan);
  gw_w32(p + 0x4, (uint32_t)src->fileNo);
  gw_w32(p + 0x8, (uint32_t)src->offset);
  gw_w32(p + 0xC, (uint32_t)src->length);
  gw_w16(p + 0x10, (uint16_t)src->iBlock);
}

static void gw_card_info_from_game(CARDFileInfo *dst, const void *src) {
  const unsigned char *p = (const unsigned char *)src;
  memset(dst, 0, sizeof(*dst));
  if (p == NULL) {
    return;
  }
  dst->chan = (s32)gw_r32(p + 0x0);
  dst->fileNo = (s32)gw_r32(p + 0x4);
  dst->offset = (s32)gw_r32(p + 0x8);
  dst->length = (s32)gw_r32(p + 0xC);
  dst->iBlock = (u16)gw_r16(p + 0x10);
}

/* CARDStat: leading char arrays cross as bytes; everything from length onward is marshalled. */
static void gw_card_stat_to_game(void *dst, const CARDStat *src) {
  unsigned char *p = (unsigned char *)dst;
  int i;
  if (p == NULL) {
    return;
  }
  memcpy(p + 0x00, src->fileName, CARD_FILENAME_MAX);
  gw_w32(p + 0x20, src->length);
  gw_w32(p + 0x24, src->time);
  memcpy(p + 0x28, src->gameName, 4);
  memcpy(p + 0x2C, src->company, 2);
  p[0x2E] = src->bannerFormat;
  gw_w32(p + 0x30, src->iconAddr);
  gw_w16(p + 0x34, src->iconFormat);
  gw_w16(p + 0x36, src->iconSpeed);
  gw_w32(p + 0x38, src->commentAddr);
  gw_w32(p + 0x3C, src->offsetBanner);
  gw_w32(p + 0x40, src->offsetBannerTlut);
  for (i = 0; i < CARD_ICON_MAX; ++i) {
    gw_w32(p + 0x44 + i * 4, src->offsetIcon[i]);
  }
  gw_w32(p + 0x64, src->offsetIconTlut);
  gw_w32(p + 0x68, src->offsetData);
}

static void gw_card_stat_from_game(CARDStat *dst, const void *src) {
  const unsigned char *p = (const unsigned char *)src;
  int i;
  memset(dst, 0, sizeof(*dst));
  if (p == NULL) {
    return;
  }
  memcpy(dst->fileName, p + 0x00, CARD_FILENAME_MAX);
  dst->length = gw_r32(p + 0x20);
  dst->time = gw_r32(p + 0x24);
  memcpy(dst->gameName, p + 0x28, 4);
  memcpy(dst->company, p + 0x2C, 2);
  dst->bannerFormat = p[0x2E];
  dst->iconAddr = gw_r32(p + 0x30);
  dst->iconFormat = (u16)gw_r16(p + 0x34);
  dst->iconSpeed = (u16)gw_r16(p + 0x36);
  dst->commentAddr = gw_r32(p + 0x38);
  dst->offsetBanner = gw_r32(p + 0x3C);
  dst->offsetBannerTlut = gw_r32(p + 0x40);
  for (i = 0; i < CARD_ICON_MAX; ++i) {
    dst->offsetIcon[i] = gw_r32(p + 0x44 + i * 4);
  }
  dst->offsetIconTlut = gw_r32(p + 0x64);
  dst->offsetData = gw_r32(p + 0x68);
}


/* ---- deferred completion ------------------------------------------------------------------ */

/* Hand the game's callback back through the port's deferred queue rather than calling it inside
 * the CARD entry point. See the note at the top of this file. */
static void gw_card_complete(void *cb, void *chan, uint32_t result) {
  if (cb != NULL) {
    ((CARDCallback)cb)((s32)(intptr_t)chan, (s32)result);
  }
}

static void gw_card_defer(void *callback, int chan, s32 result) {
  if (callback == NULL) {
    return;
  }
  gw_defer(gw_card_complete, callback, (void *)(intptr_t)chan, (uint32_t)result);
}

/* ---- lifecycle ---------------------------------------------------------------------------- */


void gw_CARDInit(void) {
  const char *base = getenv("MELEE_CARD_PATH");
  const char *raw = getenv("MELEE_CARD_RAW");
  const char *enable = getenv("MELEE_CARD");
  static char path[1024];

  if (gw_card_ready) {
    return;
  }

  /* Opt-in while the card paths are still being brought up. They are new code on both sides and
   * currently break the boot flow, so the default stays on the long-standing no-card behaviour
   * that the rest of the game is known to handle. Set MELEE_CARD=1 to exercise them. */
  if (enable == NULL || enable[0] != '1') {
    gw_log("gw: card: disabled (set MELEE_CARD=1 to enable); reporting no card");
    return;
  }

  if (base == NULL || base[0] == '\0') {
    /* Default to a "card" directory beside the executable, created on demand so a first run does
     * not have to be told where to put the card. */
    DWORD n = GetModuleFileNameA(NULL, path, (DWORD)sizeof(path));
    if (n > 0 && n < sizeof(path)) {
      char *slash = strrchr(path, '\\');
      if (slash != NULL) {
        slash[1] = '\0';
        strncat(path, "card", sizeof(path) - strlen(path) - 1);
      }
    } else {
      strcpy(path, "card");
    }
    CreateDirectoryA(path, NULL);
    base = path;
  }

  CARDSetLoadType((raw != NULL && raw[0] == '1') ? CARD_RAWIMAGE : CARD_GCIFOLDER);
  CARDSetBasePath(base, 0);
  CARDSetBasePath(base, 1);
  CARDInit(GW_CARD_GAME, GW_CARD_MAKER);

  gw_card_ready = 1;
  gw_log("gw: card: initialised (%s) at %s",
         (raw != NULL && raw[0] == '1') ? "raw image" : "GCI folder", base);
}

/* 0 = no card present. */
int gw_CARDProbe(int chan) {
  GW_CARD_LOG("gw: card: Probe(chan=%d)", chan);
  if (!gw_card_ready) {
    return 0;
  }
  return CARDProbe(chan);
}

int gw_CARDProbeEx(int chan, void *mem_size, void *sector_size) {
  s32 mem = 0;
  s32 sector = 0;
  s32 res;

  if (!gw_card_ready) {
    if (mem_size != NULL) {
      gw_w32(mem_size, 0);
    }
    if (sector_size != NULL) {
      gw_w32(sector_size, 0);
    }
    return CARD_RESULT_NOCARD;
  }

  res = CARDProbeEx(chan, &mem, &sector);
  if (mem_size != NULL) {
    gw_w32(mem_size, (uint32_t)mem);
  }
  if (sector_size != NULL) {
    gw_w32(sector_size, (uint32_t)sector);
  }
  /* Never report BUSY: db_GetGameLaunchButtonState spins until this is anything else. */
  if (res == -1) {
    res = CARD_RESULT_NOCARD;
  }
  return res;
}

int gw_CARDMountAsync(int chan, void *work_area, void *detach_callback, void *attach_callback) {
  GW_CARD_LOG("gw: card: MountAsync(chan=%d)", chan);
  if (!gw_card_ready) {
    return CARD_RESULT_NOCARD;
  }
  {
    const s32 res = CARDMount(chan, work_area, (CARDCallback)detach_callback);
    gw_card_defer(attach_callback, chan, res);
    return res;
  }
}

int gw_CARDUnmount(int chan) {
  GW_CARD_LOG("gw: card: Unmount(chan=%d)", chan);
  if (!gw_card_ready) {
    return CARD_RESULT_NOCARD;
  }
  return CARDUnmount(chan);
}

int gw_CARDCheckAsync(int chan, void *callback) {
  GW_CARD_LOG("gw: card: CheckAsync(chan=%d)", chan);
  if (!gw_card_ready) {
    return CARD_RESULT_NOCARD;
  }
  {
    const s32 res = CARDCheck(chan);
    gw_card_defer(callback, chan, res);
    return res;
  }
}

/* Aurora provides only the synchronous CARDFormat, so the async form is built here. Its other
 * async entry points call the callback before returning, so doing the same keeps the game's
 * completion counter behaving identically across all of them. */
int gw_CARDFormatAsync(int chan, void *callback) {
  GW_CARD_LOG("gw: card: FormatAsync(chan=%d)", chan);
  s32 res;

  if (!gw_card_ready) {
    return CARD_RESULT_NOCARD;
  }
  res = CARDFormat(chan);
  gw_card_defer(callback, chan, res);
  return res;
}

/* ---- files -------------------------------------------------------------------------------- */

int gw_CARDOpen(int chan, void *file_name, void *file_info) {
  GW_CARD_LOG("gw: card: Open(chan=%d, name=%s)", chan, (const char *)file_name);
  CARDFileInfo info;
  s32 res;

  if (!gw_card_ready) {
    return CARD_RESULT_NOCARD;
  }
  memset(&info, 0, sizeof(info));
  res = CARDOpen(chan, (const char *)file_name, &info);
  /* Aurora leaves fileInfo untouched when the open fails. Write the zeroed struct back anyway:
   * the game goes on to call CARDClose with it, and letting it keep whatever happened to be in
   * that memory means closing a handle built from stale bytes. */
  gw_card_info_to_game(file_info, &info);
  GW_CARD_LOG("gw: card: Open -> %d (fileNo=%d)", (int)res, (int)info.fileNo);
  return res;
}

int gw_CARDFastOpen(int chan, int file_no, void *file_info) {
  GW_CARD_LOG("gw: card: FastOpen(chan=%d, fileNo=%d)", chan, file_no);
  CARDFileInfo info;
  s32 res;

  if (!gw_card_ready) {
    return CARD_RESULT_NOCARD;
  }
  memset(&info, 0, sizeof(info));
  res = CARDFastOpen(chan, file_no, &info);
  gw_card_info_to_game(file_info, &info);
  GW_CARD_LOG("gw: card: FastOpen -> %d", (int)res);
  return res;
}

int gw_CARDClose(void *file_info) {
  GW_CARD_LOG("gw: card: Close()");
  CARDFileInfo info;
  s32 res;

  if (!gw_card_ready) {
    return CARD_RESULT_NOCARD;
  }
  gw_card_info_from_game(&info, file_info);
  res = CARDClose(&info);
  gw_card_info_to_game(file_info, &info);
  return res;
}

int gw_CARDRead(void *file_info, void *buf, int length, int offset) {
  GW_CARD_LOG("gw: card: Read(len=%d, off=%d)", length, offset);
  CARDFileInfo info;
  s32 res;

  if (!gw_card_ready) {
    return CARD_RESULT_NOCARD;
  }
  gw_card_info_from_game(&info, file_info);
  /* The payload is opaque bytes: the game interprets it, so it must not be swapped here. */
  res = CARDRead(&info, buf, length, offset);
  gw_card_info_to_game(file_info, &info);
  return res;
}

int gw_CARDReadAsync(void *file_info, void *buf, int length, int offset, void *callback) {
  GW_CARD_LOG("gw: card: ReadAsync(len=%d, off=%d)", length, offset);
  CARDFileInfo info;
  s32 res;

  if (!gw_card_ready) {
    return CARD_RESULT_NOCARD;
  }
  gw_card_info_from_game(&info, file_info);
  res = CARDRead(&info, buf, length, offset);
  gw_card_info_to_game(file_info, &info);
  gw_card_defer(callback, info.chan, res);
  return res;
}

int gw_CARDWrite(void *file_info, void *buf, int length, int offset) {
  GW_CARD_LOG("gw: card: Write(len=%d, off=%d)", length, offset);
  CARDFileInfo info;
  s32 res;

  if (!gw_card_ready) {
    return CARD_RESULT_NOCARD;
  }
  gw_card_info_from_game(&info, file_info);
  res = CARDWrite(&info, buf, length, offset);
  gw_card_info_to_game(file_info, &info);
  return res;
}

int gw_CARDWriteAsync(void *file_info, void *buf, int length, int offset, void *callback) {
  GW_CARD_LOG("gw: card: WriteAsync(len=%d, off=%d)", length, offset);
  CARDFileInfo info;
  s32 res;

  if (!gw_card_ready) {
    return CARD_RESULT_NOCARD;
  }
  gw_card_info_from_game(&info, file_info);
  res = CARDWrite(&info, buf, length, offset);
  gw_card_info_to_game(file_info, &info);
  gw_card_defer(callback, info.chan, res);
  return res;
}

int gw_CARDCreateAsync(int chan, void *file_name, uint32_t size, void *file_info, void *callback) {
  GW_CARD_LOG("gw: card: CreateAsync(chan=%d, name=%s, size=%u)", chan, (const char *)file_name, size);
  CARDFileInfo info;
  s32 res;

  if (!gw_card_ready) {
    return CARD_RESULT_NOCARD;
  }
  memset(&info, 0, sizeof(info));
  res = CARDCreate(chan, (const char *)file_name, size, &info);
  gw_card_info_to_game(file_info, &info);
  GW_CARD_LOG("gw: card: Create -> %d (fileNo=%d)", (int)res, (int)info.fileNo);
  gw_card_defer(callback, chan, res);
  return res;
}

int gw_CARDDeleteAsync(int chan, void *file_name, void *callback) {
  GW_CARD_LOG("gw: card: DeleteAsync(chan=%d, name=%s)", chan, (const char *)file_name);
  if (!gw_card_ready) {
    return CARD_RESULT_NOCARD;
  }
  {
    const s32 res = CARDDelete(chan, (const char *)file_name);
    gw_card_defer(callback, chan, res);
    return res;
  }
}

int gw_CARDRenameAsync(int chan, void *old_name, void *new_name, void *callback) {
  if (!gw_card_ready) {
    return CARD_RESULT_NOCARD;
  }
  {
    const s32 res = CARDRename(chan, (const char *)old_name, (const char *)new_name);
    gw_card_defer(callback, chan, res);
    return res;
  }
}

/* ---- status ------------------------------------------------------------------------------- */

int gw_CARDGetStatus(int chan, int file_no, void *stat) {
  GW_CARD_LOG("gw: card: GetStatus(chan=%d, fileNo=%d)", chan, file_no);
  CARDStat st;
  s32 res;

  if (!gw_card_ready) {
    return CARD_RESULT_NOCARD;
  }
  memset(&st, 0, sizeof(st));
  res = CARDGetStatus(chan, file_no, &st);
  if (res == CARD_RESULT_READY) {
    gw_card_stat_to_game(stat, &st);
  }
  return res;
}

int gw_CARDSetStatusAsync(int chan, int file_no, void *stat, void *callback) {
  GW_CARD_LOG("gw: card: SetStatusAsync(chan=%d, fileNo=%d)", chan, file_no);
  CARDStat st;

  if (!gw_card_ready) {
    return CARD_RESULT_NOCARD;
  }
  gw_card_stat_from_game(&st, stat);
  {
    const s32 res = CARDSetStatus(chan, file_no, &st);
    gw_card_defer(callback, chan, res);
    return res;
  }
}

int gw_CARDFreeBlocks(int chan, void *bytes_not_used, void *files_not_used) {
  GW_CARD_LOG("gw: card: FreeBlocks(chan=%d)", chan);
  s32 bytes = 0;
  s32 files = 0;
  s32 res;

  if (!gw_card_ready) {
    if (bytes_not_used != NULL) {
      gw_w32(bytes_not_used, 0);
    }
    if (files_not_used != NULL) {
      gw_w32(files_not_used, 0);
    }
    return CARD_RESULT_NOCARD;
  }

  res = CARDFreeBlocks(chan, &bytes, &files);
  if (bytes_not_used != NULL) {
    gw_w32(bytes_not_used, (uint32_t)bytes);
  }
  if (files_not_used != NULL) {
    gw_w32(files_not_used, (uint32_t)files);
  }
  return res;
}

int gw_CARDGetXferredBytes(int chan) {
  if (!gw_card_ready) {
    return 0;
  }
  return (int)CARDGetXferredBytes(chan);
}
