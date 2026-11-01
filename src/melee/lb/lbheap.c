#include "lbheap.h"

#include "lbmemory.h"
#include <sysdolphin/baselib/debug.h>
#include <sysdolphin/baselib/initialize.h>
#include <sysdolphin/baselib/memory.h>

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

struct lbHeap_HeapDesc {
    u32 idx;
    u32 type;
    u32 prev_idx;
    u32 size;
};

/// LbHeapKind_Count marks an absent predecessor or the table end.
#if defined(TARGET_PC)
#define LBHEAP_DESC_COUNT 6
#else
#define LBHEAP_DESC_COUNT 5
#endif

struct lbHeap_HeapDesc lbHeap_803BA380[LBHEAP_DESC_COUNT] = {
#if defined(TARGET_PC)
    /* Ported from m-ex (https://github.com/akaneia/m-ex): asm/m-ex/Persistent Heap Expansion/
     * "Relocate Heap Def.asm". Adds heap 6 (m-ex's "CUSTOM scene file heap", 0x20 bytes) and
     * moves the terminator to LBHEAP_HEAP_COUNT. m-ex raises heap 3 from 0x4F8800 to 0x4FA690.
     *
     * The Akaneia disc's IfAll.usd (0xE9B1C, the "all items" archive) is 0x55B8 larger than the
     * vanilla disc's, which overflows m-ex's heap 3 during match setup (the item/effect preload
     * in lbdvd.c inline2 -- EfMnData, EfCoData, ItCo, IfAll -- all land in heap 3). The measured
     * shortfall was 0xB6B0; the first PC expansion added 0x20000 (128 KB) to reach 0x51A690.
     * Six full Geno Falcons on ACE ext:121 then left only 9,520 bytes. Give heap 3 another
     * 8 MiB (0xD1A690 total), leaving >8 MiB for that load and a 4 MiB regression floor.
     * GW_MEM1_SIZE in gw_runtime.c grows by the same 8 MiB (40 -> 48 MiB):
     * lbHeap_80015900's remaining main heap and heaps 4/5 keep their capacities. */
    /* Heap 4 (AllM, the fighter files) keeps the first 16 MiB of the PC port's extra MEM1:
       0x64B400 + 0x1000000. Two Ultimate Soras in different costumes (3.6 + 3.9 MB costume files
       on top of 0.9 MB of fighter data) failed ALLOC_FAIL heap 4 at 0x64B400; four need ~15 MB.
       The main heap (Hsd) is what is left of the arena, so it keeps its size. */
    { 2, 1, 6, 0x800 },    { 3, 1, 2, 0xD1A690 }, { 4, 2, 6, 0x164B400 },
    /* Heap 5 (AllA) holds each loaded fighter's complete animation archive. Sora's PlUsAJ is
     * about 4.6 MB; Classic's multi-opponent preload filled 8.8 MB of the old 9.4 MB slice and
     * failed its next 0x14D940-byte allocation. The extra 16 MB of port ARAM goes to AllA. */
    { 5, 4, 6, 0x196C800 }, { 6, 1, 3, 0x20 },     { LBHEAP_HEAP_COUNT, 0, 0, 0 },
#else
    { LbHeapKind_Seq, 1, LbHeapKind_Count, 0x800 },
    { LbHeapKind_Stay, 1, LbHeapKind_Seq, 0x4F8800 },
    { LbHeapKind_AllM, 2, LbHeapKind_Count, 0x64B400 },
    { LbHeapKind_AllA, 4, LbHeapKind_Count, 0x96C800 },
    { LbHeapKind_Count, 0, 0, 0 },
#endif
};

static inline void resetHeap(LbHeapKind kind)
{
    struct Heap* heap = &lbHeap_80431FA0.heap_array[kind];
    heap->id = -1;
    heap->handle = (Handle*) -1;
    heap->start = 0;
    heap->size = 0;
    heap->type = 1;
    heap->transient = 1;
    heap->status = LbHeapStatus_Destroy;
}

static inline void destroyHeap(LbHeapKind kind)
{
    struct Heap* heap = &lbHeap_80431FA0.heap_array[kind];
    if (heap->status == LbHeapStatus_Create) {
        if (heap->type == 0) {
            OSDestroyHeap(heap->id);
            heap->id = -1;
        } else {
            lbMemory_80014EEC(heap->handle);
            heap->handle = (Handle*) -1;
        }
        heap->status = LbHeapStatus_Destroy;
    }
}

static inline void createHeap(LbHeapKind kind)
{
    struct Heap* heap = &lbHeap_80431FA0.heap_array[kind];
    if (heap->status == LbHeapStatus_Destroy) {
        if (heap->type == 0) {
            heap->id = OSCreateHeap((void*) heap->start,
                                    (void*) (heap->start + heap->size));
        } else {
            heap->handle = lbMemory_80014E24(
                (void*) heap->start, (void*) (heap->start + heap->size));
        }
        heap->status = LbHeapStatus_Create;
    }
}

void lbHeap_800158D0(int arg0, int arg1)
{
    lbHeap_80431FA0.heap_array[arg0].transient = arg1;
}

int lbHeap_800158E8(int arg0)
{
    return lbHeap_80431FA0.heap_array[arg0].transient;
}

void lbHeap_80015900(void)
{
    uintptr_t arena_hi;
    uintptr_t arena_lo;
    s32 create_i;
    struct Heap* main_heap;
    s32 bounds_i;
    uintptr_t end;
    struct Heap* bounds_heap;
    uintptr_t aram_lo;
    uintptr_t aram_hi;
    s32 destroy_i;
    struct Heap* aram_heap;

    for (destroy_i = LbHeapKind_Seq; destroy_i < LBHEAP_HEAP_COUNT; destroy_i++)
    {
        if (lbHeap_80431FA0.heap_array[destroy_i].transient == 1) {
            destroyHeap(destroy_i);
        }
    }

    arena_lo = (uintptr_t) lbHeap_80431FA0.arena_lo;
    arena_hi = (uintptr_t) lbHeap_80431FA0.arena_hi;
    aram_lo = lbHeap_80431FA0.aram_lo;
    aram_hi = lbHeap_80431FA0.aram_hi;

    for (bounds_i = LbHeapKind_Seq; bounds_i < LBHEAP_HEAP_COUNT; bounds_i++) {
        bounds_heap = &lbHeap_80431FA0.heap_array[bounds_i];
        if (lbHeap_80431FA0.heap_array[bounds_i].transient == 0) {
            switch (bounds_heap->type) {
            case 1:
                end = bounds_heap->start + bounds_heap->size;
                if (arena_lo < end) {
                    arena_lo = end;
                }
                break;

            case 2:
                if (arena_hi > bounds_heap->start) {
                    arena_hi = bounds_heap->start;
                }
                break;

            case 4:
                end = bounds_heap->start + bounds_heap->size;
                if (aram_lo < end) {
                    aram_lo = end;
                }
                break;
            }
        }
    }

    main_heap = &lbHeap_80431FA0.heap_array[LbHeapKind_Hsd];
    main_heap->id = HSD_CreateMainHeap((void*) arena_lo, (void*) arena_hi);
    main_heap->start = arena_lo;
    aram_heap = &lbHeap_80431FA0.heap_array[LbHeapKind_ARAM];
    main_heap->size = arena_hi - arena_lo;
    main_heap->status = LbHeapStatus_Create;
    main_heap->type = 0;

    lbMemory_800155A4();

    aram_heap->handle = lbMemory_800154D4((void*) aram_lo, (void*) aram_hi);
    aram_heap->start = aram_lo;
    aram_heap->size = aram_hi - aram_lo;
    aram_heap->status = LbHeapStatus_Create;
    aram_heap->type = 3;

    for (create_i = LbHeapKind_Seq; create_i < LBHEAP_HEAP_COUNT; create_i++) {
        if (lbHeap_80431FA0.heap_array[create_i].transient == 0) {
            createHeap(create_i);
        }
    }
}

LbHeapStatus lbHeap_80015BB8(int arg0)
{
    return lbHeap_80431FA0.heap_array[arg0].status;
}

void* lbHeap_80015BD0(int heap_id, size_t size)
{
    void* result;
    int enabled = OSDisableInterrupts();
    struct Heap* p = &lbHeap_80431FA0.heap_array[heap_id];

    if (p->status == LbHeapStatus_Create) {
        if (p->type == 0) {
            int cur_heap = HSD_GetHeap();
            HSD_SetHeap(p->id);
            result = HSD_MemAlloc(size);
            HSD_SetHeap(cur_heap);
        } else {
#if defined(TARGET_PC)
            /* MELEE_HEAP_TRACE=1: log each allocation from the handle-managed (ARAM / fixed)
             * heaps - to see what fills one when an allocation fails. */
            static int trace = -1;
            if (trace < 0) {
                extern int Env1(const char* name);
                trace = Env1("MELEE_HEAP_TRACE");
            }
            if (trace) {
                OSReport("lbHeap: heap %d type %d alloc 0x%X\n", heap_id, p->type,
                         (unsigned int) size);
            }
#endif
            result = lbMemory_80014FC8(p->handle, size);
            if (p->type == 3) {
                result = ((HSD_AllocEntry*) result)->addr;
            }
        }
    } else {
        result = NULL;
    }
    OSRestoreInterrupts(enabled);
    return result;
}

void lbHeap_80015CA8(int arg0, void* addr)
{
    int enabled = OSDisableInterrupts();
    struct Heap* p = &lbHeap_80431FA0.heap_array[arg0];

    HSD_ASSERT(0x143, p->status == LbHeapStatus_Create);
    if (p->type == 0) {
        int cur_heap = HSD_GetHeap();
        HSD_SetHeap(p->id);
        HSD_Free(addr);
        HSD_SetHeap(cur_heap);
    } else {
        lbMemFreeToHeap(p->handle, addr);
    }
    OSRestoreInterrupts(enabled);
}

int lbHeap_80015D6C(u32 heap0, void (*cb)(u32), u32 heap1)
{
    int enabled = OSDisableInterrupts();
    struct Heap* p = &lbHeap_80431FA0.heap_array[heap0];
    int var_r30;

    if (heap0 <= 1) {
        var_r30 = 0;
    } else {
        var_r30 = lbMemory_8001529C(p->handle, cb, heap1);
    }
    OSRestoreInterrupts(enabled);
    return var_r30;
}

char* lbHeap_803BA448[] = {
    "     Hsd", "    ARAM", "     Seq", "    Stay", "    AllM", "    AllA",
#if defined(TARGET_PC)
    /* Ported from m-ex (https://github.com/akaneia/m-ex): asm/m-ex/Persistent Heap Expansion/
     * adds heap 6 but never extends this parallel name table, so the widened report loop in
     * lbHeap_80015DF8 would read one entry past the end. The name is added here to keep that
     * loop in bounds; it is the "CUSTOM scene file heap" in m-ex's Relocate Heap Def.asm. */
    "   Scene",
#endif
};

#if defined(TARGET_PC)
/* Which lbHeap owns a handle-managed arena (lbMemory's ALLOC_FAIL line names it), -1 if none.
 * Writes the heap's trimmed report name ("Stay", "AllM", ...) into name. */
int lbHeap_IdOfHandle(void* handle, char* name, int cap)
{
    int i;
    for (i = 0; i < LBHEAP_HEAP_COUNT; i++) {
        if (lbHeap_80431FA0.heap_array[i].status == LbHeapStatus_Create &&
            lbHeap_80431FA0.heap_array[i].type != 0 &&
            (void*) lbHeap_80431FA0.heap_array[i].handle == handle)
        {
            const char* s = lbHeap_803BA448[i];
            int k = 0;
            while (*s == ' ') {
                s++;
            }
            while (s[k] != '\0' && k < cap - 1) {
                name[k] = s[k];
                k++;
            }
            name[k] = '\0';
            return i;
        }
    }
    if (cap > 0) {
        name[0] = '\0';
    }
    return -1;
}
#endif


void lbHeap_80015DF8(void)
{
    ssize_t bytes;
    struct Heap* p;
    int i;
    int var_r25;

    OSReport("[lbHeap] -- Report --\n");

    for (i = 0; i < LBHEAP_HEAP_COUNT; i++) {
        OSReport("%s :", lbHeap_803BA448[i]);
        p = &lbHeap_80431FA0.heap_array[i];
        if (p->status == LbHeapStatus_Create) {
            if (p->type == 0) {
                var_r25 = OSCheckHeap(p->id);
            } else {
#if defined(TARGET_PC)
                /* m-ex's 0x20-byte "Scene" heap (6) is too small for lbMemory_80014E24 to give it a
                   handle, and lbMemory_80014F7C on that NULL read address 4: the ALLOC_FAIL report
                   this prints turned a clean out-of-memory assert into an ACCESS_VIOLATION. */
                if (p->handle == NULL || p->handle == (Handle*) -1) {
                    OSReport("                    no arena");
                    OSReport(" / %5d KB\n", p->size / 1024, p->size);
                    continue;
                }
#endif
                var_r25 = lbMemory_80014F7C(p->handle);
            }
            OSReport(" %5d KB + ", (p->size - var_r25) / 1024);
            OSReport(" %5d KB( %8d)", var_r25 / 1024, var_r25);
        } else {
            OSReport("                         destroy");
        }
        OSReport(" / %5d KB\n", p->size / 1024, p->size);
    }

    bytes = (uintptr_t) lbHeap_80431FA0.arena_hi -
            (uintptr_t) lbHeap_80431FA0.arena_lo;
    OSReport("MainRAM Total : %5d KB( %8d)\n", bytes / 1024, bytes);
    bytes = lbHeap_80431FA0.aram_hi - lbHeap_80431FA0.aram_lo;
    OSReport("   ARAM Total : %5d KB( %8d)\n", bytes / 1024, bytes);
}

void lbHeap_80015F3C(void)
{
    int curr_idx;
    int prev_idx;
    struct Heap* curr_heap;
    struct Heap* prev_heap;
    struct lbHeap_HeapDesc* desc;
    LbHeapKind kind;

    HSD_GetNextArena(&lbHeap_80431FA0.arena_lo, &lbHeap_80431FA0.arena_hi);
    lbMemory_800154BC(&lbHeap_80431FA0.aram_lo, &lbHeap_80431FA0.aram_hi);

    for (kind = 0; kind < LBHEAP_HEAP_COUNT; kind++) {
        resetHeap(kind);
    }

    desc = lbHeap_803BA380;
    while ((curr_idx = desc->idx) != LBHEAP_HEAP_COUNT) {
        curr_heap = &lbHeap_80431FA0.heap_array[curr_idx];

        curr_heap->type = desc->type;
        curr_heap->size = desc->size;
        prev_idx = desc->prev_idx;
        if (prev_idx == LbHeapKind_Count) {
            switch (curr_heap->type) {
            case 3:
                break;
            case 1:
                curr_heap->start = (uintptr_t) lbHeap_80431FA0.arena_lo;
                break;
            case 2:
                curr_heap->start =
                    (uintptr_t) lbHeap_80431FA0.arena_hi - curr_heap->size;
                break;
            case 4:
                curr_heap->start = lbHeap_80431FA0.aram_lo;
                break;
            }
        } else {
            prev_heap = &lbHeap_80431FA0.heap_array[prev_idx];
            switch (curr_heap->type) {
            case 3:
                break;
            case 1:
                curr_heap->start = prev_heap->start + prev_heap->size;
                break;
            case 2:
                curr_heap->start = prev_heap->start - curr_heap->size;
                break;
            case 4:
                curr_heap->start = prev_heap->start + prev_heap->size;
                break;
            }
        }
        desc++;
    }
#if defined(TARGET_PC)
    OSReport("lbHeap: PC stage heap 3 expanded capacity=%u (+8 MiB MEM1)\n",
             lbHeap_80431FA0.heap_array[3].size);
#endif
}

#if defined(TARGET_PC)
unsigned lbHeap_Capacity(int heap)
{
    return heap >= 0 && heap < LBHEAP_HEAP_COUNT ? lbHeap_80431FA0.heap_array[heap].size : 0;
}
#endif

#if defined(TARGET_PC)
unsigned lbHeap_Free(int heap)
{
    struct Heap* p;
    int free_bytes;
    if (heap<0 || heap>=LBHEAP_HEAP_COUNT) return 0;
    p=&lbHeap_80431FA0.heap_array[heap];
    if (p->status!=LbHeapStatus_Create) return 0;
    if (p->type==0) {
        free_bytes=OSCheckHeap(p->id);
        return free_bytes>0 ? free_bytes : 0;
    }
    if (!p->handle || p->handle==(Handle*) -1) return 0;
    return lbMemory_80014F7C(p->handle);
}
#endif
#include "lbheap_stage_slots.inc"
