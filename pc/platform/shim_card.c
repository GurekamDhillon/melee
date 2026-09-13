/* CARD shims.
 *
 * The port has no memory card. Every operation reports "no card" (CARD_RESULT_NOCARD, -3)
 * synchronously and never calls back. Two details matter for boot:
 *   - CARDProbeEx must never return CARD_RESULT_BUSY (-1): db_GetGameLaunchButtonState spins on it
 *     until it returns something else.
 *   - Async calls must fail immediately rather than queue, or lbcardnew's completion counter
 *     (and the lb_8001B760(11) spin behind it) never settles.
 * With cards always absent, lbCardGame_DecideGameMode takes the no-card path and the card scene is
 * never entered. */
#include "gw.h"

#define CARD_RESULT_NOCARD (-3)

void gw_CARDInit(void) {}

/* 0 = no card present; the game's card-existence flag stays 0. */
int gw_CARDProbe(int chan) {
  (void)chan;
  return 0;
}

int gw_CARDProbeEx(int chan, void *mem_size, void *sector_size) {
  (void)chan;
  if (mem_size != NULL) {
    *(int *)mem_size = 0;
  }
  if (sector_size != NULL) {
    *(int *)sector_size = 0;
  }
  return CARD_RESULT_NOCARD;
}

int gw_CARDOpen(int chan, void *file_name, void *file_info) {
  (void)chan;
  (void)file_name;
  (void)file_info;
  return CARD_RESULT_NOCARD;
}

int gw_CARDFastOpen(int chan, int file_no, void *file_info) {
  (void)chan;
  (void)file_no;
  (void)file_info;
  return CARD_RESULT_NOCARD;
}

int gw_CARDClose(void *file_info) {
  (void)file_info;
  return CARD_RESULT_NOCARD;
}

int gw_CARDRead(void *file_info, void *buf, int length, int offset) {
  (void)file_info;
  (void)buf;
  (void)length;
  (void)offset;
  return CARD_RESULT_NOCARD;
}

int gw_CARDReadAsync(void *file_info, void *buf, int length, int offset, void *callback) {
  (void)file_info;
  (void)buf;
  (void)length;
  (void)offset;
  (void)callback;
  return CARD_RESULT_NOCARD;
}

int gw_CARDWrite(void *file_info, void *buf, int length, int offset) {
  (void)file_info;
  (void)buf;
  (void)length;
  (void)offset;
  return CARD_RESULT_NOCARD;
}

int gw_CARDWriteAsync(void *file_info, void *buf, int length, int offset, void *callback) {
  (void)file_info;
  (void)buf;
  (void)length;
  (void)offset;
  (void)callback;
  return CARD_RESULT_NOCARD;
}

int gw_CARDGetStatus(int chan, int file_no, void *stat) {
  (void)chan;
  (void)file_no;
  (void)stat;
  return CARD_RESULT_NOCARD;
}

int gw_CARDSetStatusAsync(int chan, int file_no, void *stat, void *callback) {
  (void)chan;
  (void)file_no;
  (void)stat;
  (void)callback;
  return CARD_RESULT_NOCARD;
}

int gw_CARDCreateAsync(int chan, void *file_name, uint32_t size, void *file_info,
                       void *callback) {
  (void)chan;
  (void)file_name;
  (void)size;
  (void)file_info;
  (void)callback;
  return CARD_RESULT_NOCARD;
}

int gw_CARDDeleteAsync(int chan, void *file_name, void *callback) {
  (void)chan;
  (void)file_name;
  (void)callback;
  return CARD_RESULT_NOCARD;
}

int gw_CARDRenameAsync(int chan, void *old_name, void *new_name, void *callback) {
  (void)chan;
  (void)old_name;
  (void)new_name;
  (void)callback;
  return CARD_RESULT_NOCARD;
}

int gw_CARDFormatAsync(int chan, void *callback) {
  (void)chan;
  (void)callback;
  return CARD_RESULT_NOCARD;
}

int gw_CARDMountAsync(int chan, void *work_area, void *detach_callback, void *attach_callback) {
  (void)chan;
  (void)work_area;
  (void)detach_callback;
  (void)attach_callback;
  return CARD_RESULT_NOCARD;
}

int gw_CARDUnmount(int chan) {
  (void)chan;
  return CARD_RESULT_NOCARD;
}

int gw_CARDCheckAsync(int chan, void *callback) {
  (void)chan;
  (void)callback;
  return CARD_RESULT_NOCARD;
}

int gw_CARDFreeBlocks(int chan, void *bytes_not_used, void *files_not_used) {
  (void)chan;
  if (bytes_not_used != NULL) {
    *(int *)bytes_not_used = 0;
  }
  if (files_not_used != NULL) {
    *(int *)files_not_used = 0;
  }
  return CARD_RESULT_NOCARD;
}

int gw_CARDGetXferredBytes(int chan) {
  (void)chan;
  return 0;
}
