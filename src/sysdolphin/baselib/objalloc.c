#include "objalloc.h"

#include <string.h>

#include "initialize.h"
#include "memory.h"
#include <dolphin/os/OSAlloc.h>

static objheap obj_heap = { 0, 0, -1, -1 };

static HSD_ObjAllocData* alloc_datas;

void HSD_ObjSetHeap(u32 size, void* ptr)
{
    obj_heap.curr = (u32) ptr;
    obj_heap.top = (u32) ptr;
    obj_heap.remain = size;
    obj_heap.size = size;
}

s32 HSD_ObjAllocAddFree(HSD_ObjAllocData* data, u32 num)
{
    u32 computed_start;
    u32 pool_end;
    u32 pool_size;
    u8* pool_start;

    u8 _[4];

    HSD_ASSERT(0xEE, data);
#if defined(TARGET_PC)
    /* MELEE_HEAP_TRACE=1: which object allocator is refilling. AddFree(data, 1) means one
     * HSD_MemAlloc per object, so a runaway count here IS a runaway spawn - 62,547 of them on
     * Akaneia's ext:302, about 227 a frame. A PER-ALLOCATOR TABLE, not a consecutive-run
     * counter: allocators interleave, so a run counter resets constantly and never reports.
     * The data pointer identifies the owner; the map resolves it to gobj_alloc_data,
     * aobj_alloc_data and so on. */
    {
        extern int PcTraceHeapEnabled(void);
        static int oa_trace = -1;
        static void *oa_who[8];
        static int oa_hits[8];
        static int oa_size[8];
        static int oa_total;
        int k, worst = 0;
        if (oa_trace < 0) {
            oa_trace = PcTraceHeapEnabled();
        }
        if (oa_trace != 0) {
            for (k = 0; k < 8; k++) {
                if (oa_who[k] == (void *) data) { oa_hits[k]++; break; }
                if (oa_hits[k] < oa_hits[worst]) { worst = k; }
            }
            if (k == 8) {
                oa_who[worst] = (void *) data;
                oa_hits[worst] = 1;
                oa_size[worst] = (int) data->size;
            }
            if ((++oa_total % 8192) == 0) {
                for (k = 0; k < 8; k++) {
                    if (oa_hits[k] > 256) {
                        OSReport("objalloc: %p size %d x%d\n", oa_who[k],
                                 oa_size[k], oa_hits[k]);
                    }
                }
            }
        }
    }
#endif
    pool_size = data->size * num;
    if (obj_heap.top != 0) {
        pool_end = obj_heap.top + obj_heap.size;
        computed_start = (obj_heap.curr + data->align) & ~data->align;
        pool_start = (void*) computed_start;
        if (computed_start > pool_end) {
            return 0;
        }
        if (pool_end - (u32) pool_start < pool_size) {
            pool_size = pool_end - (u32) pool_start -
                        (pool_end - (u32) pool_start) % data->size;
        }
        num = pool_size / data->size;
        if (num == 0) {
            return 0;
        }
        obj_heap.curr = (u32) pool_start + pool_size;
        obj_heap.remain = pool_end - obj_heap.curr;
    } else {
        pool_start = HSD_MemAlloc(pool_size);
        if (pool_start == 0) {
            return 0;
        }
        obj_heap.remain -= pool_size;
    }

    {
        int i;
        for (i = 0; (unsigned) i < num - 1; i++) {
            *(void**) (pool_start + data->size * i) =
                (void*) (pool_start + data->size * (i + 1));
        }
        *(void**) (pool_start + data->size * i) = data->freehead;
    }

    data->freehead = (HSD_ObjAllocLink*) pool_start;
    data->free += num;
    return num;
}

void* HSD_ObjAlloc(HSD_ObjAllocData* data)
{
    HSD_ObjAllocLink* cur;
    u32 size;

    if (data->num_limit_flag && data->used >= data->num_limit) {
        return NULL;
    }
    if (data->heap_limit_flag) {
        if (data->heap_limit_num == (unsigned) -1) {
            if (obj_heap.top != 0) {
                size = obj_heap.remain;
            } else {
                size = OSCheckHeap(HSD_GetHeap());
            }
            if (size <= data->heap_limit_size) {
                data->heap_limit_num = data->used + data->free;
            }
        } else {
            if (obj_heap.top != 0) {
                size = obj_heap.remain;
            } else {
                size = OSCheckHeap(HSD_GetHeap());
            }
            if (size > data->heap_limit_size) {
                data->heap_limit_num = -1;
            }
        }
        if (data->used >= data->heap_limit_num) {
            return NULL;
        }
    }
    if (data->free == 0) {
        HSD_ObjAllocAddFree(data, 1);
        if (data->free == 0) {
            return NULL;
        }
    }
    cur = data->freehead;
    data->freehead = cur->next;
    data->used += 1;
    data->free -= 1;
    if (data->used > data->peak) {
        data->peak = data->used;
    }
    return cur;
}

void HSD_ObjFree(HSD_ObjAllocData* data, void* obj)
{
    HSD_ObjAllocLink* link = obj;
    link->next = data->freehead;
    data->freehead = link;
    data->free += 1;
    data->used -= 1;
}

static inline void removeAll(HSD_ObjAllocData* data)
{
    HSD_ObjAllocData** cur = &alloc_datas;
    while (*cur != NULL) {
        if (*cur == data) {
            *cur = (*cur)->next;
        } else {
            cur = &(*cur)->next;
        }
    }
}

void HSD_ObjAllocInit(HSD_ObjAllocData* data, size_t size, u32 align)
{
    HSD_ASSERT(0x185, data);
    if (data != NULL) {
        removeAll(data);
    } else {
        alloc_datas = NULL;
    }
    memset(data, 0, sizeof(HSD_ObjAllocData));
    data->num_limit = -1;
    data->heap_limit_size = 0;
    data->heap_limit_num = -1;
    data->align = align - 1;
    data->size = (size + data->align) & ~data->align;
    data->next = alloc_datas;
    alloc_datas = data;
}

void _HSD_ObjAllocForgetMemory(void* low, void* high)
{
    alloc_datas = NULL;
}
