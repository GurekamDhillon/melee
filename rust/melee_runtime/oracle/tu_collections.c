/* Include the original definitions to exercise static IDEntryAlloc/Free/hash too. */
#include <sysdolphin/baselib/id.c>
#include <sysdolphin/baselib/list.c>
#include <stdio.h>
#include <stdlib.h>

extern void* malloc(unsigned long long);
extern void free(void*);
extern void abort(void);
void __assert(const char* f, u32 line, const char* what) { abort(); }
void HSD_ObjAllocInit(HSD_ObjAllocData* d, size_t size, u32 align)
{
    memset(d, 0, sizeof(*d)); d->size = size; d->align = align;
}
void* HSD_ObjAlloc(HSD_ObjAllocData* d)
{
    void* p = malloc(d->size);
    memset(p, 0xA5, d->size);
    d->used++; if (d->used > d->peak) d->peak = d->used;
    return p;
}
void HSD_ObjFree(HSD_ObjAllocData* d, void* p)
{ d->used--; d->free++; free(p); }

static u32 rng = 0x945952;
static u32 next(void) { rng = rng * 1664525 + 1013904223; return rng; }
static void pools(void)
{
    HSD_ObjAllocData* pools[] = { HSD_IDGetAllocData(), HSD_SListGetAllocData(), HSD_DListGetAllocData() };
    int k;
    for (k = 0; k < 3; ++k) {
        HSD_ObjAllocData* d = pools[k];
        printf(" %u %u %u %u %u %u %u %u %u %u %u %u", d->num_limit_flag, d->heap_limit_flag,
            d->freehead != NULL, d->used, d->free, d->peak, d->num_limit, d->heap_limit_size,
            d->heap_limit_num, d->size, d->align, d->next != NULL);
    }
}
static void tables(HSD_IDTable* a)
{
    int t, b;
    for (t = 0; t < 2; ++t) {
        HSD_IDTable* tab = t ? &default_table : a;
        for (b = 0; b < 101; ++b) {
            IDEntry* e;
            for (e = tab->table[b]; e; e = e->next)
                printf(" %d:%d:%llu:%llu", t, b, (unsigned long long)e->id, (unsigned long long)(uintptr_t)e->data);
        }
    }
}
int main(void)
{
    HSD_IDTable a = { 0 };
    HSD_SList* lists[2] = { NULL, NULL };
    int i, j, counts[10] = { 0 };
    HSD_IDInitAllocData(); HSD_ListInitAllocData(); HSD_IDSetup();
    printf("EMPTY %d\n", HSD_SListRemove(NULL) == NULL);
    for (i = 0; i < 1200; ++i) {
        u32 r = next(), op = (r >> 5) % 10, which = (r >> 8) & 1;
        HSD_IDKey key = (HSD_IDKey)((r >> 16) % 16) * 101 + ((r >> 12) & 3);
        void* data = (void*)(uintptr_t)(next() & 0xFFFF);
        HSD_IDTable* tab = which ? NULL : &a;
        s32 success = -9;
        void* value = NULL;
        counts[op]++;
        if (i % 97 == 0) key |= (HSD_IDKey)1 << 40;
        if (op < 3) HSD_IDInsertToTable(tab, key, data);
        else if (op == 3) HSD_IDRemoveByIDFromTable(tab, key);
        else if (op == 4) value = HSD_IDGetDataFromTable(tab, key, &success);
        else if (op == 5) lists[which] = HSD_SListAllocAndAppend(lists[which], data);
        else if (op == 6) lists[which] = HSD_SListAllocAndPrepend(lists[which], data);
        else if (op == 7) lists[which] = HSD_SListRemove(lists[which]);
        else if (op == 8) {
            HSD_SList* node = HSD_SListAlloc();
            node->data = data;
            lists[which] = (r & 0x400) ? HSD_SListAppendList(lists[which], node) : HSD_SListPrependList(lists[which], node);
        } else {
            IDEntry* e = IDEntryAlloc();
            printf("ZERO %u %u %u\n", e->next == NULL, e->id == 0, e->data == NULL);
            IDEntryFree(e);
            if ((r & 0x400) == 0) _HSD_IDForgetMemory(NULL, NULL); else HSD_IDSetup();
        }
        printf("S %d %u %d %llu", i, hash(key), success, (unsigned long long)(uintptr_t)value);
        pools(); tables(&a);
        printf(" L");
        for (j = 0; j < 2; ++j) {
            HSD_SList* n;
            printf(" %d", j);
            for (n = lists[j]; n; n = n->next) printf(":%llu", (unsigned long long)(uintptr_t)n->data);
        }
        printf("\n");
    }
    /* Present NULL data and success==NULL are distinct from absent keys. */
    HSD_IDInsertToTable(&a, 0xFFFFFFFFFFFFFFFFull, NULL);
    i = -9;
    HSD_IDGetDataFromTable(&a, 0xFFFFFFFFFFFFFFFFull, &i);
    printf("NULL %d %u\n", i, HSD_IDGetDataFromTable(&a, 0xFFFFFFFFFFFFFFFFull, NULL) == NULL);
    printf("OPS"); for (i = 0; i < 10; i++) printf(" %d", counts[i]); printf("\n");
    return 0;
}
