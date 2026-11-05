#ifndef MELEE_LB_HEAP_H
#define MELEE_LB_HEAP_H

#include <Runtime/platform.h>

/// @remarks @c LbHeapStatus_Create is named by an assert in #lbHeap_80015CA8.
typedef enum LbHeapStatus {
    /* 0x00 */ LbHeapStatus_Create,
    /* 0x01 */ LbHeapStatus_Destroy,
} LbHeapStatus;

typedef enum LbHeapKind {
    /* 0x00 */ LbHeapKind_Hsd,
    /* 0x01 */ LbHeapKind_ARAM,
    /* 0x02 */ LbHeapKind_Seq,
    /* 0x03 */ LbHeapKind_Stay,
    /* 0x04 */ LbHeapKind_AllM,
    /* 0x05 */ LbHeapKind_AllA,
    /* 0x06 */ LbHeapKind_Count,
} LbHeapKind;

#if defined(TARGET_PC)
/* Ported from m-ex (https://github.com/akaneia/m-ex): asm/m-ex/Persistent Heap Expansion/
 * adds a seventh heap slot (id 6) and moves the descriptor terminator to 7. This is a
 * build-level change rather than a runtime option, because the heap array's size must be
 * compile-time; it is therefore always on for TARGET_PC and never on for the matcher. */
#define LBHEAP_HEAP_COUNT 7
#else
#define LBHEAP_HEAP_COUNT LbHeapKind_Count
#endif

/* 0158D0 */ void lbHeap_800158D0(int, int);
/* 0158E8 */ int lbHeap_800158E8(int);
/* 015900 */ void lbHeap_80015900(void);
/* 015BB8 */ LbHeapStatus lbHeap_80015BB8(int);
/** Returns a RAM pointer for Hsd, an ARAM address for ARAM, or an
 * HSD_AllocEntry pointer for the other heaps.
 */
/* 015BD0 */ void* lbHeap_80015BD0(int heap_id, size_t size);
/// Takes the payload address, not the allocation record.
/* 015CA8 */ void lbHeap_80015CA8(int, void*);
/* 015D6C */ int lbHeap_80015D6C(u32 heap0, void (*cb)(u32), u32 heap1);
/* 015DF8 */ void lbHeap_80015DF8(void);
/* 015F3C */ void lbHeap_80015F3C(void);

#if defined(TARGET_PC)
unsigned lbHeap_Capacity(int heap);
unsigned lbHeap_Free(int heap);
#endif
#endif
