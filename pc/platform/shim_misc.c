/* Shims for hardware that is simply absent on the PC: data cache maintenance, the debug port, the
 * PowerPC MSR, and the audio interface. */
#include "gw.h"

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

/* No AI hardware. Aurora owns the audio path (currently unimplemented), so these are inert until
 * the AX/AI work happens. */
void gw_AIInit(uint8_t *stack) { (void)stack; }

void gw_AISetDSPSampleRate(uint32_t rate) { (void)rate; }

void gw_AISetStreamVolLeft(uint8_t vol) { (void)vol; }

void gw_AISetStreamVolRight(uint8_t vol) { (void)vol; }
