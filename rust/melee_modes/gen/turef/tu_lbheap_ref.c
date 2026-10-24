/* C reference for rust/melee_modes/src/tu/lbheap.rs and lbmemory.rs: the game's lbheap.c and lbmemory.c verbatim
 * (one program, so the calls between them are the real ones); dolphin ar/os, the baselib heap and DevCom are
 * stubbed and logged (the Rust hooks log the same text). DevCom requests and OSAlarms are queued and completed
 * by pump(). Every step prints a hash of both modules' whole state; a hash of main RAM is printed every 25.
 * The image is linked below 4 GB because lbmemory.c keeps Handle* in u32 (`cb_arg`).
 * Run: python rust/melee_modes/gen/turef/tu_shard833.py tu_lbheap */
#include <Runtime/platform.h>
#include <dolphin/os.h>
/* the macro reads 0x800000F8 through a sign extended (u32*) on this 64-bit host: use the GameCube value */
#undef OS_BUS_CLOCK
#define OS_BUS_CLOCK 162000000u
#include "../../../../src/melee/lb/lbheap.c"
#include "../../../../src/melee/lb/lbmemory.c"
#include "tu_lb_common.h"
extern char* getenv(const char*);
extern int atoi(const char*);

static struct {
    int on;
    HSD_DevComCallback cb;
    uintptr_t args;
} DC;
static struct {
    int on;
    OSAlarm* a;
    OSAlarmHandler h;
} AL;
static int g_done;
static void done_cb(u32 v) { LG("done(%u);", (unsigned) v); g_done++; }

int HSD_DevComRequest(int file, uintptr_t src, uintptr_t dest, gc_size_t size, int type, int pri, HSD_DevComCallback callback, uintptr_t args)
{
    LG("HSD_DevComRequest(%d,%s,%s,%u,%d,%d,%s,%s);", file, pidu(src), pidu(dest), (unsigned) size, type, pri, pid((const void*) callback),
       pidu(args));
    DC.on = 1;
    DC.cb = callback;
    DC.args = args;
    return 0;
}
void OSSetAlarm(OSAlarm* alarm, OSTime tick, OSAlarmHandler handler)
{
    LG("OSSetAlarm(%s,%d,%s);", pid((const void*) alarm), (int) tick, pid((const void*) handler));
    AL.on = 1;
    AL.a = alarm;
    AL.h = handler;
}
void* HSD_MemAlloc(ssize_t size)
{
    LG("HSD_MemAlloc(%d);", (int) size);
    return rnd() % 6 == 0 ? NULL : ARENA_P + (rnd() % 64) * 16;
}
void HSD_GetNextArena(void** lo, void** hi)
{
    unsigned long long l;
    unsigned long long h;
    LG("HSD_GetNextArena(%s,%s);", pid((const void*) lo), pid((const void*) hi));
    l = 0x80100000ull + (rnd() % 4) * 0x1000;
    h = l + 0x164B400ull + 0x40000ull + (rnd() % 4) * 0x1000;
    *lo = (void*) (size_t) l;
    *hi = (void*) (size_t) h;
}
void* OSAllocFromHeap(int heap, unsigned long size)
{
    LG("OSAllocFromHeap(%d,%u);", heap, (unsigned) size);
    return rnd() % 4 == 0 ? NULL : ARENA_P + (rnd() % 64) * 16;
}
#include "tu_lbheap_stubs.inc"

static void pump(void)
{
    int n = 0;
    while ((DC.on || AL.on) && n++ < 4000) {
        if (DC.on) {
            DC.on = 0;
            DC.cb(0, DC.args, NULL, 0);
        }
        if (AL.on) {
            AL.on = 0;
            AL.h(AL.a, NULL);
        }
    }
}

static u32 H;
static void hx(u32 v) { H = (H ^ v) * 16777619u; }
static u32 hv(const void* p)
{
    unsigned long long a = (unsigned long long) (size_t) p;
    unsigned long long b = (unsigned long long) (size_t) &lbMemory_804318B0;
    if (a >= b && a < b + sizeof lbMemory_804318B0) return 0x40000000u + (u32) (a - b);
    return (u32) a;
}
static u32 hfn(const void* f)
{
    if (f == NULL) return 0;
    if (f == (const void*) done_cb) return 1;
    if (f == (const void*) lbMemory_80015320) return 2;
    return 3;
}
static u32 state_hash(void)
{
    int i;
    H = 2166136261u;
    hx(hv(lbMemory_804318B0.a_arenaLo));
    hx(hv(lbMemory_804318B0.a_arenaHi));
    for (i = 0; i < 0x83; i++) {
        hx(hv(lbMemory_804318B0.x8_mem[i].x0_next));
        hx(hv(lbMemory_804318B0.x8_mem[i].x4_lo));
        hx(hv(lbMemory_804318B0.x8_mem[i].x8_hi));
    }
    hx(hv(lbMemory_804318B0.free_mem));
    hx((u32) lbMemory_804318B0.x630_num_allocs);
    hx((u32) lbMemory_804318B0.x634_max_num_allocs);
    for (i = 0; i < 6; i++) {
        hx(hv(lbMemory_804318B0.x638_heap[i].x0_next));
        hx(hv(lbMemory_804318B0.x638_heap[i].x4_lo));
        hx(hv(lbMemory_804318B0.x638_heap[i].x8_hi));
        hx(hv(lbMemory_804318B0.x638_heap[i].xC_prev));
    }
    hx(hv(lbMemory_804318B0.free_heap));
    hx(hv(lbMemory_804318B0.x69C));
    hx((u32) (size_t) lbMemory_804318B0.x6A0_mgr.src);
    hx((u32) (size_t) lbMemory_804318B0.x6A0_mgr.dst);
    hx(lbMemory_804318B0.x6A0_mgr.size);
    hx(lbMemory_804318B0.x6A0_mgr.offset);
    hx(hv((const void*) (size_t) lbMemory_804318B0.x6A0_mgr.cb_arg));
    hx(hfn((const void*) lbMemory_804318B0.x6A0_mgr.cb));
    hx(lbMemory_804318B0.x6E0);
    hx(hv(lbMemory_804318B0.x6E4));
    hx(hfn((const void*) lbMemory_804318B0.x6E8));
    hx(hv(lbHeap_80431FA0.arena_lo));
    hx(hv(lbHeap_80431FA0.arena_hi));
    hx((u32) lbHeap_80431FA0.aram_lo);
    hx((u32) lbHeap_80431FA0.aram_hi);
    for (i = 0; i < LBHEAP_HEAP_COUNT; i++) {
        struct Heap* p = &lbHeap_80431FA0.heap_array[i];
        hx((u32) p->id);
        hx(hv(p->handle));
        hx((u32) p->start);
        hx((u32) ((unsigned long long) p->start >> 32));
        hx(p->size);
        hx((u32) p->type);
        hx((u32) p->transient);
        hx((u32) p->status);
    }
    hx((u32) g_done);
    hx((u32) DC.on);
    hx((u32) AL.on);
    return H;
}
static u32 mem_hash(void)
{
    u32 h = 2166136261u;
    unsigned char* m = MEM2_P + 0x100000;
    int i;
    for (i = 0; i < 0x100000; i++) h = (h ^ m[i]) * 16777619u;
    return h;
}

/* heap `a` can be used by lbHeap_80015BD0/CA8: not created, or created with a real handle (a heap that an
 * assert interrupted can be left created without one, which the game would dereference) */
static int heap_ok(int a)
{
    struct Heap* p = &lbHeap_80431FA0.heap_array[a];
    return p->status != LbHeapStatus_Create || p->type == 0 || (p->handle != NULL && p->handle != (Handle*) -1);
}
typedef struct { int heap; void* val; } Slot;
static Slot SL[64];
static int NSL;

static int g_free_heap = -1;
static void do_free(int* pa, u32* pb)
{
    int a = *pa;
    u32 b = *pb;
    int mode = (int) (rnd() % 8);
    mode = mode == 7 ? 2 : mode < 4 ? 0 : 1;
            if (NSL > 0) {
                int s = (int) (rnd() % NSL);
                if (g_free_heap >= 0) {
                    int q, m = 0, pick;
                    for (q = 0; q < NSL; q++) m += SL[q].heap == g_free_heap;
                    if (m == 0) return;
                    pick = (int) (rnd() % m);
                    for (q = 0; q < NSL; q++) {
                        if (SL[q].heap == g_free_heap && pick-- == 0) { s = q; break; }
                    }
                }
                void* v = SL[s].val;
                a = SL[s].heap;
                if ((unsigned long long) (size_t) v >= (unsigned long long) (size_t) &lbMemory_804318B0 &&
                    (unsigned long long) (size_t) v < (unsigned long long) (size_t) &lbMemory_804318B0 + sizeof lbMemory_804318B0 && mode == 0) {
            mode = 1;
                }
                if (mode == 1 && (unsigned long long) (size_t) v >= (unsigned long long) (size_t) &lbMemory_804318B0 &&
                    (unsigned long long) (size_t) v < (unsigned long long) (size_t) &lbMemory_804318B0 + sizeof lbMemory_804318B0) {
                    v = ((Handle*) v)->x4_lo;
                }
                if (mode == 2) {
                    v = (void*) (size_t) (rnd() % 4096);
                    a = (int) (rnd() % 7);
                }
                b = hv(v);
                if (heap_ok(a)) TRY({ lbHeap_80015CA8(a, v); });
                if (!g_asserted && mode != 2) {
                    SL[s] = SL[--NSL];
                }
            }
    *pa = a;
    *pb = b;
}

extern void* GetStdHandle(unsigned);
extern int WriteFile(void*, const void*, unsigned, unsigned*, void*);
static void dbgw(const char* s, unsigned n) { unsigned w; WriteFile(GetStdHandle(0xFFFFFFF4u), s, n, &w, 0); }
#define DBG(s) dbgw(s "\n", sizeof(s))
int main(void)
{
    int i, n = getenv("N") ? atoi(getenv("N")) : 6000;
    if ((unsigned long long) (size_t) &lbMemory_804318B0 > 0xFFFFFFFFull) {
        printf("allocator above 4 GB\n");
        return 1;
    }
    arena_init();
    mem2_init();
    preg("buf", ARENA_P, 1u << 20);
    preg("alloc", &lbMemory_804318B0, sizeof lbMemory_804318B0);
    preg("hs", &lbHeap_80431FA0, sizeof lbHeap_80431FA0);
    pfreg("done", done_cb);
    pfreg("cb", lbMemory_80015320);
    pfreg("alarm", fn_80015184);
    for (i = 0; i < 0x180000; i++) MEM2_P[0x100000 + i] = (unsigned char) rnd();
    for (i = 0; i < n; i++) {
        int op = (int) (rnd() % 100);
        int a = 0, asserted = 0;
        g_asserted = 0;
        u32 b = 0;
        unsigned long long ret = 0;
        const char* rk = "-";
        g_loglen = 0;
        g_log[0] = 0;
        if (i == 0 || op < 4) {
            u32 t[5];
            int k;
            for (k = 0; k < 5; k++) t[k] = (rnd() % 4 == 0) ? 1 : 0;
            t[4] = 0;
            memset(&lbMemory_804318B0, 0, sizeof lbMemory_804318B0);
            memset(&lbHeap_80431FA0, 0, sizeof lbHeap_80431FA0);
            DC.on = AL.on = 0;
            NSL = 0;
            TRY({
                lbMemory_8001564C();
                lbHeap_80015F3C();
                for (k = 0; k < 5; k++) lbHeap_800158D0(2 + k, (int) t[k]);
                lbHeap_80015900();
            });
            a = -1;
        } else if (op < 7) {
            a = (int) (2 + rnd() % 5);
            b = rnd() % 2;
            TRY({
                lbHeap_800158D0(a, (int) b);
                lbHeap_80015900();
            });
            NSL = 0;
        } else if (op < 35 || op >= 89) {
            int r = (int) (rnd() % 25);
            size_t sz;
            void* res = NULL;
            a = (int) (rnd() % 7);
            if (rnd() % 2) a = 3 + (int) (rnd() % 3);
            sz = r == 0 ? 0 : r == 1 ? 0x10000000u : (size_t) (rnd() % 0x3000);
            b = (u32) sz;
            if (heap_ok(a)) TRY({ res = lbHeap_80015BD0(a, sz); });
            ret = (unsigned long long) (size_t) res;
            rk = "p";
            if (!g_asserted && res != NULL) {
                int s = NSL < 64 ? NSL++ : (int) (rnd() % 64);
                SL[s].heap = a;
                SL[s].val = res;
            }
        } else if (op < 65) {
            if (NSL > 0) do_free(&a, &b);
        } else if (op < 80) {
            int ok = 1;
            a = (int) (rnd() % 7);
            if (rnd() % 2) a = 3 + (int) (rnd() % 3);
            b = rnd() % 100;
            if (a >= 2) {
                struct Heap* p = &lbHeap_80431FA0.heap_array[a];
                ok = p->status == LbHeapStatus_Create && p->type != 0 && p->handle != NULL && p->handle != (Handle*) -1;
            }
            if (ok) {
                int r = 0, k;
                g_done = 0;
                g_free_heap = a >= 1 ? a : -1;
                for (k = 0; k < 3; k++) {
                    if (NSL > 0 && rnd() % 3 != 0) {
                        int fa = 0;
                        u32 fb = 0;
                        do_free(&fa, &fb);
                    }
                }
                TRY({
                    r = lbHeap_80015D6C((u32) a, done_cb, b);
                    pump();
                });
                g_free_heap = -1;
                ret = (unsigned long long) r;
                rk = "i";
            } else {
                rk = "skip";
            }
        } else if (op < 83) {
            u32 q1, q2, q3, q4;
            int id;
            char name[16];
            a = (int) (rnd() % 9) - 1;
            q1 = a >= 0 && a < 7 ? (u32) lbHeap_800158E8(a) : 99;
            q2 = a >= 0 && a < 7 ? (u32) lbHeap_80015BB8(a) : 99;
            q3 = lbHeap_Capacity(a);
            q4 = 0;
            TRY({ q4 = lbHeap_Free(a); });
            id = lbHeap_IdOfHandle(a >= 0 && a < 7 ? (void*) lbHeap_80431FA0.heap_array[a].handle : NULL, name, 16);
            printf("q %u %u %u %u id=%d name=%s\n", q1, q2, q3, q4, id, name);
        } else if (op < 86) {
            void* res = NULL;
            a = (int) (rnd() % 3);
            b = rnd() % 3000;
            TRY({
                res = lbHeap_StageSlotTryAlloc(b);
                lbHeap_StageSlotDump(a);
            });
            ret = (unsigned long long) (size_t) res;
            rk = "p";
        } else {
            TRY({ lbHeap_80015DF8(); });
        }
        printf("%d op=%d a=%d b=%u asserted=%d ret=%s:%s h=%08x log=%s\n", i, op, a, (unsigned) b, g_asserted, rk, rk[0] == 'p' ? pidu(ret) : rk[0] == 'i' ? (ret ? "1" : "0") : "-", (unsigned) state_hash(),
               g_log);
        if (i % 25 == 24) printf("mem %08x\n", (unsigned) mem_hash());
    }
    return 0;
}
