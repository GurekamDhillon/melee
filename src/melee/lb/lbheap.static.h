#ifndef __GALE01_0158D0
#define __GALE01_0158D0

#include <melee/lb/lbheap.h> // IWYU pragma: export
#include <melee/lb/lbmemory.h>

#if defined(TARGET_PC)
/* Ported from m-ex (https://github.com/akaneia/m-ex): asm/m-ex/Persistent Heap Expansion/
 * adds a seventh heap slot (id 6) and moves the descriptor terminator to 7. This one is a
 * build-level change rather than a runtime option, because the heap array's size must be
 * compile-time; it is therefore always on for TARGET_PC and never on for the matcher. */
#define LBHEAP_HEAP_COUNT 7
#else
#define LBHEAP_HEAP_COUNT 6
#endif

struct Heap {
    /* 10 */ s32 id;
    /* 14 */ Handle* handle;
    /// The heap's base *address*. `s32` would sign-extend when it is cast
    /// back to a pointer; the same four bytes on GameCube.
    /* 18 */ uintptr_t start;
    /* 1C */ u32 size;
    /* 20 */ s32 type;
    /* 24 */ s32 transient;
    /* 28 */ LbHeapStatus status;
};
ASSERT_SIZE(struct Heap, 0x1C);

struct lbHeap_HeapState {
    /* 0x00 */ void* arena_lo;    /* inferred */
    /* 0x04 */ void* arena_hi;    /* inferred */
    /* 0x08 */ uintptr_t aram_lo; /* inferred */
    /* 0x0C */ uintptr_t aram_hi; /* inferred */
    /* 0x10 */ struct Heap heap_array[LBHEAP_HEAP_COUNT];
}; /* size = 0xB8 vanilla; grows 0x1C per extra heap */
ASSERT_SIZE(struct lbHeap_HeapState, 0x10 + LBHEAP_HEAP_COUNT * 0x1C);

/* 431FA0 */ static struct lbHeap_HeapState lbHeap_80431FA0;

#endif
