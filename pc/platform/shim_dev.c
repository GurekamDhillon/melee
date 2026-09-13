/* Stubs for SDK subsystems the port does not support.
 *
 * MCC is the GBA link cable controller and FIO is a developer file-I/O channel: neither exists on
 * the PC and nothing on the boot path uses them, so every entry point fails immediately. THP is the
 * FMV decoder; the port skips the opening movie (OSGetResetCode()==0x80000000), so its entry points
 * are inert too. */
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

/* ---- THP (FMV decoder) -------------------------------------------------------------------- */

void gw_THPInit(void) {}

int gw_THPVideoDecode(void *file, void *tile_y, void *tile_u, void *tile_v, void *work) {
  (void)file;
  (void)tile_y;
  (void)tile_u;
  (void)tile_v;
  (void)work;
  return -1;
}

int gw_THPDec_8032F8D4(void *data, void *out) {
  (void)data;
  (void)out;
  return -1;
}

int gw_THPDec_8032FD40(void *data, uint16_t arg) {
  (void)data;
  (void)arg;
  return -1;
}

void gw_THPDec_80331340(int a, void *b, void *c, void *d) {
  (void)a;
  (void)b;
  (void)c;
  (void)d;
}

void gw_THPDec_803313D0(int a, void *b, void *c, void *d, uint32_t e) {
  (void)a;
  (void)b;
  (void)c;
  (void)d;
  (void)e;
}
