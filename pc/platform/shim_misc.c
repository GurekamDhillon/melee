/* Shims for hardware that is simply absent on the PC: data cache maintenance, the debug port, the
 * PowerPC MSR, and the audio interface. */
#include "gw.h"
#include "shim_ax.h"

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
