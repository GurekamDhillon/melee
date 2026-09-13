/* AR/ARQ shims: the audio RAM and its DMA queue.
 *
 * The port's ARAM is a real 16 MB buffer at low addresses (gw_aram), so the game's
 * "< 0x80000000 means ARAM" rule keeps working and an ARQ transfer is a plain memcpy between the
 * two address spaces.
 *
 * The transfer is done inline but the callback is queued through gw_defer so it runs outside the
 * caller's stack. That matters for HSD's ARAM state machine: its completion callbacks unlink the
 * node and recurse into HSD_DevComARAMWakeUp, and running them inline leaves the outer frame
 * dereferencing a node the callback already freed (observed as an access violation at
 * HSD_DevComARAMWakeUp+0x1E1 reading aramDC). The deferred queue is drained by the frame tick and
 * by gw_wait_idle, which the loading waits reach. lbarq.c's callback-less path spins on its node
 * state without pumping; if that is ever hit at boot, that spin needs fixing rather than moving
 * this callback back inline. */
#include "gw.h"
#include "shim_vi.h"

/* Stack allocator over ARAM, mirroring extern/dolphin/src/dolphin/ar/ar.c.
 *
 * Two details of the real allocator are load-bearing and were easy to miss:
 *   - the base is 0x4000, not 0: ar.c:117 sets __AR_StackPointer to 0x4000 and ARInit returns it,
 *     reserving the low 16 KB. lbmemory.c:342 probes the base with an ARAlloc/ARFree pair and
 *     stores it as the ARAM arena lo, so a base of 0 installs a NULL arena there;
 *   - ARFree's argument is an *out* parameter (ar.c:88): it pops the block-length stack and
 *     writes the popped size back. lbmemory.c:343 passes an uninitialised local, so reading it
 *     instead consumes stack garbage.
 *
 * The block-length stack is the array the game hands to ARInit (lbaudio_ax.c:2092, u32[0x10]).
 * It lives in game memory, so entries go through gw_r32/gw_w32. */
#define GW_AR_BASE 0x4000u

static uint32_t gw_ar_top = GW_AR_BASE;
static uint32_t *gw_ar_stack_base;  /* game memory: one entry per live allocation */
static uint32_t gw_ar_blocks_used;  /* live allocations */
static uint32_t gw_ar_blocks_total; /* capacity of that array */

uint32_t gw_ARInit(uint32_t *stack_index_addr, uint32_t num_entries) {
  gw_ar_top = GW_AR_BASE;
  gw_ar_stack_base = stack_index_addr;
  gw_ar_blocks_used = 0;
  gw_ar_blocks_total = num_entries;
  return GW_AR_BASE;
}

uint32_t gw_ARGetSize(void) { return gw_aram_size; }

uint32_t gw_ARAlloc(uint32_t length) {
  /* ar.c asserts 32-byte alignment rather than rounding; round instead so a stray caller cannot
   * desync the stack, which keeps every returned offset 32-aligned either way. */
  const uint32_t aligned = (length + 31u) & ~31u;
  if (gw_ar_top + aligned > gw_aram_size) {
    gw_log("gw: ARAlloc(%u) does not fit in ARAM (%u bytes free)", length,
           gw_aram_size - gw_ar_top);
    return 0;
  }
  if (gw_ar_stack_base == NULL || gw_ar_blocks_used >= gw_ar_blocks_total) {
    gw_log("gw: ARAlloc(%u) has no free block-length slot (ARInit not called?)", length);
    return 0;
  }
  const uint32_t offset = gw_ar_top;
  gw_ar_top += aligned;
  gw_w32(&gw_ar_stack_base[gw_ar_blocks_used], aligned);
  ++gw_ar_blocks_used;
  gw_log("gw: ARAlloc(%u) -> 0x%X", length, offset);
  return offset;
}

uint32_t gw_ARFree(uint32_t *length) {
  if (gw_ar_stack_base == NULL || gw_ar_blocks_used == 0) {
    gw_log("gw: ARFree with no live ARAM allocation");
    return gw_ar_top;
  }
  --gw_ar_blocks_used;
  const uint32_t popped = gw_r32(&gw_ar_stack_base[gw_ar_blocks_used]);
  if (length != NULL) {
    gw_w32(length, popped);
  }
  gw_ar_top = gw_ar_top >= GW_AR_BASE + popped ? gw_ar_top - popped : GW_AR_BASE;
  return gw_ar_top;
}

void gw_ARQInit(void) {}

/* ARAM offsets are 0..gw_aram_size; anything else is a native pointer.
 *
 * The obvious test -- "below 0x80000000 means ARAM" -- is what the game itself uses, and it is
 * wrong here. It holds for memory carved out of MEM1, which the port maps at 0x80000000, but the
 * game's *statically linked* globals live in the exe image instead, far below that line. DevCom
 * hands exactly such an address to ARQ (devcom.c:151 passes &HSD_DevCom_804C6330_bufs[i], a game
 * BSS array), so the old test read it as a wild ARAM offset and memcpy walked off the end of the
 * buffer. Bounding by gw_aram_size is unambiguous as long as no real pointer can be smaller,
 * which is why the port is linked at a fixed base above ARAM (/BASE:0x10000000 /DYNAMICBASE:NO in
 * build_melee_pc.bat). */
static void *gw_ar_addr(uint32_t addr) {
  if (addr < gw_aram_size) {
    return gw_aram + addr;
  }
  if (addr < 0x80000000u && (gw_mem1 == NULL || addr < (uint32_t)(uintptr_t)gw_mem1)) {
    /* Neither an ARAM offset nor a MEM1 address: a game global or host allocation. Fine to use
     * directly, but worth noticing if it ever looks like a small integer mistaken for a pointer. */
    if (addr < 0x10000u) {
      gw_log("gw: ARQ address 0x%08X is neither an ARAM offset nor a plausible pointer", addr);
    }
  }
  return (void *)(uintptr_t)addr;
}

static void gw_arq_complete(void *request, void *callback, uint32_t unused) {
  (void)unused;
  ((void (*)(void *))callback)(request);
}

void gw_ARQPostRequest(void *request, uint32_t owner, uint32_t type, uint32_t priority,
                       uint32_t source, uint32_t dest, uint32_t length, void *callback) {
  (void)owner;
  (void)type;
  (void)priority;
  if (length != 0 && source != dest) {
    memcpy(gw_ar_addr(dest), gw_ar_addr(source), length);
  }
  if (callback != NULL) {
    gw_defer(gw_arq_complete, request, callback, 0);
  }
}
