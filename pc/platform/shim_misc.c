/* Shims for hardware that is simply absent on the PC: data cache maintenance, the debug port, the
 * PowerPC MSR, and the audio interface. */
#include "gw.h"
#include "shim_ax.h"

#include <string.h>

/* DMA-coherency maintenance only; x86 has no such caches, and every game DMA is a memcpy here. */
void gw_DCFlushRange(void *addr, uint32_t n) {
  (void)addr;
  (void)n;
}

void gw_DCStoreRange(void *addr, uint32_t n) {
  (void)addr;
  (void)n;
}

void gw_DCInvalidateRange(void *addr, uint32_t n) {
  (void)addr;
  (void)n;
}

/* DCZeroRange is the one cache op with an architectural side effect rather than just coherency
 * bookkeeping: dcbz establishes a cache line as all zeroes without reading memory first, so the
 * range really does read back as zero afterwards. THPVideoDecode relies on it to clear its
 * 0x920-byte decoder state. */
void gw_DCZeroRange(void *addr, uint32_t n) { memset(addr, 0, n); }

/* ---- locked cache ---------------------------------------------------------------------------
 * The Gekko can lock a 16 KB chunk of L1 as directly addressed scratch at 0xE0000000 and DMA
 * between it and main memory. The THP decoder is the only user here: it builds each MCU row of
 * Y/U/V tiles in locked cache and queues it out with LCStoreData. With no such memory on the PC,
 * THPInit points its work pointers at an ordinary static buffer (see the TARGET_PC branch of
 * THPInit in extern/dolphin/src/dolphin/thp/THPDec.c), which makes the store a plain copy and the
 * queue wait a no-op. Both sides are raw tile bytes, so there is no endianness to handle. */
uint32_t gw_LCStoreData(void *dst, void *src, uint32_t n) {
  memcpy(dst, src, n);
  return 0;
}

void gw_LCQueueWait(uint32_t len) { (void)len; }

/* No debugger attached to the PC build. */
int gw_DBIsDebuggerPresent(void) { return 0; }

/* MSR access is only used to mask/unmask interrupts; the port is not interrupt-driven (the frame
 * driver is the single async source and runs on the main thread). */
int gw_PPCMfmsr(void) { return 0; }

void gw_PPCMtmsr(int msr) { (void)msr; }

/* The AI has no hardware here; the AX mixer in shim_ax.c owns the output path. These track the
 * master volume and sample rate the game sets, which the mixer applies each sub-frame. */
void gw_AIInit(uint8_t *stack) { (void)stack; }

void gw_AISetDSPSampleRate(uint32_t rate) { gw_ai_dsp_sample_rate = rate; }

void gw_AISetStreamVolLeft(uint8_t vol) { gw_ai_stream_vol_left = vol; }

void gw_AISetStreamVolRight(uint8_t vol) { gw_ai_stream_vol_right = vol; }
