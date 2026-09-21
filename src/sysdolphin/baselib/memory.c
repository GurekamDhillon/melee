#include "memory.h"

#include <Runtime/platform.h>

#include "debug.h"
#include "initialize.h"
#include <dolphin/os/OSAlloc.h>

#if defined(TARGET_PC)
/* MELEE_HEAP_TRACE=1: census the HSD heap. Akaneia's ext:302 renders for ~275 frames and then
 * HSD_MemAlloc returns NULL, so something allocates on a repeating path that should be
 * once-only. Counting allocations and frees, and remembering the most-repeated large size,
 * names it. Off by default - this sits on every allocation the game makes. */
#include <dolphin/os.h>
extern int PcTraceHeapEnabled(void);
static int hsd_trace = -1;
static int hsd_allocs;
static int hsd_frees;
static int hsd_live;
static int hsd_big_size;
static int hsd_big_count;
#define HSD_HIST 12
static int hist_size[HSD_HIST];
static int hist_hits[HSD_HIST];

/* Exact-size histogram. The struct a repeating path allocates is identified by its SIZE, so the
 * few most-requested sizes name the leak without needing a call stack. Smallest-hit slot is
 * evicted, which is enough when one size dominates by orders of magnitude. */
#define HSD_CALLERS 8
static unsigned caller_pc[HSD_CALLERS];
static int caller_hits[HSD_CALLERS];

/* Who is allocating. The size histogram says a 48-byte object is made 31,795 times but not
 * WHAT makes it - it is not an HSD class (nothing reaches hsdNew) and not m-ex's calloc. The
 * return address answers it outright, and melee-pc.map turns it into a name. */
static void caller_note(unsigned pc)
{
    int i, worst = 0;
    for (i = 0; i < HSD_CALLERS; i++) {
        if (caller_pc[i] == pc) {
            caller_hits[i]++;
            return;
        }
        if (caller_hits[i] < caller_hits[worst]) {
            worst = i;
        }
    }
    caller_pc[worst] = pc;
    caller_hits[worst] = 1;
}

static void caller_dump(void)
{
    int i;
    for (i = 0; i < HSD_CALLERS; i++) {
        if (caller_hits[i] > 64) {
            OSReport("heap:   from 0x%08X x%d\n", caller_pc[i], caller_hits[i]);
        }
    }
}

static void hist_note(int size)
{
    int i, worst = 0;
    for (i = 0; i < HSD_HIST; i++) {
        if (hist_size[i] == size) {
            hist_hits[i]++;
            return;
        }
        if (hist_hits[i] < hist_hits[worst]) {
            worst = i;
        }
    }
    hist_size[worst] = size;
    hist_hits[worst] = 1;
}

static void hist_dump(void)
{
    int i;
    for (i = 0; i < HSD_HIST; i++) {
        if (hist_hits[i] > 64) {
            OSReport("heap:   size %5d x %d\n", hist_size[i], hist_hits[i]);
        }
    }
}

static void hsd_note(int size, int freeing)
{
    if (hsd_trace < 0) {
        hsd_trace = PcTraceHeapEnabled();
    }
    if (hsd_trace == 0) {
        return;
    }
    if (freeing) {
        hsd_frees++;
        return;
    }
    hsd_allocs++;
    hist_note(size);
    hsd_live += size;
    if (size >= 2048) {
        if (size == hsd_big_size) {
            hsd_big_count++;
        } else {
            hsd_big_size = size;
            hsd_big_count = 1;
        }
    }
    if ((hsd_allocs % 512) == 0) {
        OSReport("heap: %d allocs %d frees, live %d KB, top repeat %d bytes x%d\n",
                 hsd_allocs, hsd_frees, hsd_live / 1024, hsd_big_size, hsd_big_count);
    }
}
#endif

void HSD_Free(void* ptr)
{
#if defined(TARGET_PC)
    {
        extern void Snap_NoteMem(int size, void* ptr, unsigned caller, int freeing);
        Snap_NoteMem(0, ptr, (unsigned) (uintptr_t) __builtin_return_address(0), 1);
    }
    hsd_note(0, 1);
#endif
    OSFreeToHeap(HSD_GetHeap(), ptr);
}

void* HSD_MemAlloc(ssize_t size)
{
    void* adr;

    if (size <= 0) {
        return NULL;
    }

    adr = OSAllocFromHeap(HSD_GetHeap(), size);
#if defined(TARGET_PC)
    {
        extern void Snap_NoteMem(int size, void* ptr, unsigned caller, int freeing);
        Snap_NoteMem((int) size, adr,
                     (unsigned) (uintptr_t) __builtin_return_address(0), 0);
    }
    hsd_note((int) size, 0);
    /* Frame 0 HERE is HSD_MemAlloc's own caller, which is the thing that wants memory.
     * Taking it inside hsd_note() gave HSD_MemAlloc itself, and asking for frame 1 there
     * walked the frame chain and took the process down. */
    caller_note((unsigned) (uintptr_t) __builtin_return_address(0));
    if (adr == NULL) {
        OSReport("heap: EXHAUSTED on %d bytes after %d allocs %d frees, live %d KB, top repeat %d bytes x%d\n",
                 (int) size, hsd_allocs, hsd_frees, hsd_live / 1024, hsd_big_size,
                 hsd_big_count);
        hist_dump();
        caller_dump();
    }
#endif
    HSD_ASSERT(52, adr);

    return adr;
}
