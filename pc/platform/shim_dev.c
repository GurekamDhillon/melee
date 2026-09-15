/* Stubs for SDK subsystems the port does not support.
 *
 * MCC is the GBA link cable controller and FIO is a developer file-I/O channel: neither exists on
 * the PC and nothing on the boot path uses them, so every entry point fails immediately. (THP, the
 * FMV decoder, used to be stubbed here too; it is now built for real - see the note at the end of
 * this file.) */
#include "gw.h"

/* ---- MCC (GBA link) ----------------------------------------------------------------------- */

int gw_MCCInit(int exi_channel, uint8_t timeout, void *callback) {
  (void)exi_channel;
  (void)timeout;
  (void)callback;
  return -1;
}

void gw_MCCExit(void) {}

int gw_MCCEnumDevices(void *callback) {
  (void)callback;
  return 0; /* no devices */
}

int gw_MCCGetConnectionStatus(int ch, void *connect) {
  (void)ch;
  if (connect != NULL) {
    *(int *)connect = 0;
  }
  return -1;
}

uint8_t gw_MCCGetFreeBlocks(int mode) {
  (void)mode;
  return 0;
}

uint8_t gw_MCCGetLastError(void) { return 0; }

int gw_MCCNotify(int ch, uint32_t notify) {
  (void)ch;
  (void)notify;
  return -1;
}

int gw_MCCOpen(int ch, uint8_t block_size, void *callback) {
  (void)ch;
  (void)block_size;
  (void)callback;
  return -1;
}

int gw_MCCRead(int ch, uint32_t offset, void *data, long size, int async) {
  (void)ch;
  (void)offset;
  (void)data;
  (void)size;
  (void)async;
  return -1;
}

int gw_MCCStreamOpen(int ch, uint8_t block_size) {
  (void)ch;
  (void)block_size;
  return -1;
}

int gw_MCCWrite(int ch, uint32_t offset, void *data, long size, int async) {
  (void)ch;
  (void)offset;
  (void)data;
  (void)size;
  (void)async;
  return -1;
}

int gw_MCCClose(int ch) {
  (void)ch;
  return -1;
}

/* ---- FIO (developer file I/O) ------------------------------------------------------------- */

int gw_FIOInit(int exi_channel, int ch_id, uint8_t block_size) {
  (void)exi_channel;
  (void)ch_id;
  (void)block_size;
  return -1;
}

void gw_FIOExit(void) {}

int gw_FIOQuery(void) { return -1; }

int gw_FIOFopen(const char *filename, uint32_t mode) {
  (void)filename;
  (void)mode;
  return -1;
}

int gw_FIOFclose(int handle) {
  (void)handle;
  return -1;
}

uint32_t gw_FIOFwrite(int handle, void *data, uint32_t size) {
  (void)handle;
  (void)data;
  (void)size;
  return 0;
}

/* ---- THP (FMV decoder) --------------------------------------------------------------------
 * Not stubbed any more: extern/dolphin/src/dolphin/thp/THPDec.c is built as a game TU and
 * supplies the real decoder, with its Gekko paired-single IDCT and assembly Huffman decoders
 * reimplemented in C behind TARGET_PC. The locked cache it decodes into is backed by
 * gw_LCStoreData/gw_LCQueueWait in shim_misc.c. */
