/* C reference for rust/melee_runtime/src/tu/lbfile.rs: the game's lbfile.c verbatim; DevCom, the disc pump, DVD,
 * the heap and the language getters are stubbed and logged (the Rust hooks log the same text). The pump
 * stub completes the pending request after a seeded number of calls, running the TU's own callback.
 * Run: python rust/melee_runtime/gen/turef/tu_shard833.py tu_lbfile */
#include "../../../../src/melee/lb/lbfile.c"
#include "tu_lb_common.h"
extern char* getenv(const char*);
extern int atoi(const char*);

static struct {
    int on;
    HSD_DevComCallback cb;
    uintptr_t args;
    int countdown;
} PEND;
static int g_pumps, g_cb2, g_dvd_hit, g_open_len;
static void cb2(int a, uintptr_t b, void* c, bool d) { (void) a; (void) c; (void) d; g_cb2++; LG("cb2(%u);", (unsigned) b); }

void lb_800195D0(void)
{
    LG("lb_800195D0();");
    if (++g_pumps > 100000) {
        printf("pump loop\n");
        for (;;) {}
    }
    if (PEND.on) {
        if (PEND.countdown-- == 0) {
            PEND.on = 0;
            PEND.cb(0, PEND.args, NULL, 0);
        }
    }
}
int HSD_DevComRequest(int file, uintptr_t src, uintptr_t dest, gc_size_t size, int type, int pri, HSD_DevComCallback callback, uintptr_t args)
{
    LG("HSD_DevComRequest(%d,%s,%s,%u,%d,%d,%s,%s);", file, pidu(src), pidu(dest), (unsigned) size, type, pri, pid((const void*) callback),
       pidu(args));
    PEND.on = 1;
    PEND.cb = callback;
    PEND.args = args;
    PEND.countdown = (int) (rnd() % 3);
    return 0;
}
BOOL DVDFastOpen(s32 entry, DVDFileInfo* info)
{
    BOOL ok = (BOOL) (rnd() % 5 != 0);
    LG("DVDFastOpen(%d,%s);", (int) entry, pid(info));
    if (ok) {
        info->length = rnd() % 100000;
    }
    return ok;
}
HSD_Archive* lbDvd_8001819C(const char* name)
{
    LG("lbDvd_8001819C(%s);", pids(name));
    return (HSD_Archive*) (rnd() % 3 == 0 ? NULL : ARENA_P + 512);
}
void* lbHeap_80015BD0(int heap_id, gc_size_t size)
{
    LG("lbHeap_80015BD0(%d,%u);", heap_id, (unsigned) size);
    return rnd() % 5 == 0 ? NULL : ARENA_P + (rnd() % 8) * 64;
}
#include "tu_lbfile_stubs.inc"

static const char* NAMES[] = {"PlFx", "IfAll.usd", "GmTitle.", "a.b", "x.", "", "noext", "abcdefghijklmnopqrstuvwxyz0", "abcdefghijklmnopqrstuvwxy.", "abcdefghijklmnopqrstuvwxyz01234",
                              "...", ".dat", "Pl.Fx", "ab.c", "q.."};
#define NNAMES ((int) (sizeof NAMES / sizeof NAMES[0]))
#define NF 12

static char* g_res;
static u32 sumres(void)
{
    u32 h = 2166136261u;
    int i;
    for (i = 0; i < 32; i++) {
        h = (h ^ (u8) g_res[i]) * 16777619u;
    }
    return h;
}

int main(void)
{
    int i;
    const char* name;
    arena_init();
    preg("buf", ARENA_P, 1u << 20);
    pfreg("cb", lbFile_8001615C);
    pfreg("cb2", cb2);
    g_res = lbFileGetFullName("");
    g_loglen = 0;
    for (i = 0; i < (getenv("N") ? atoi(getenv("N")) : 700); i++) {
        int f = i % NF;
        size_t sz = 0xABCD;
        void* dst = NULL;
        int ret = 0;
        uintptr_t dstv;
        u32 args = rnd();
        name = NAMES[rnd() % NNAMES];
        dstv = (rnd() % 2) ? (uintptr_t) (ARENA_P + (rnd() % 8) * 64) : (uintptr_t) 0x80000000u + (rnd() % 4) * 32;
        g_loglen = 0;
        g_log[0] = 0;
        PEND.on = 0;
        g_pumps = 0;
        cancel = (rnd() & 1) != 0;
        switch (f) {
        case 0: TRY(ret = lbFileGetFullName(name) == g_res); break;
        case 1: {
            int a1 = (int) (rnd() % 50);
            u32 b1 = rnd() % 1000;
            u32 c1 = rnd() % 100000;
            int d1 = (int) (rnd() % 4);
            int e1 = (int) (rnd() % 3);
            TRY(lbFile_800161C4(a1, b1, dstv, c1, d1, e1));
            break;
        }
        case 2: {
            int a1 = (int) (rnd() % 50);
            TRY(ret = (int) lbFile_8001634C(a1));
            break;
        }
        case 3: TRY(ret = (int) lbFileGetSize(name)); break;
        case 4: {
            int a1 = (int) (rnd() % 50);
            int b1 = (int) (rnd() % 3);
            HSD_DevComCallback cb = (rnd() & 1) ? cb2 : lbFile_8001615C;
            TRY(lbFile_800164A4(a1, dstv, &sz, b1, cb, args));
            break;
        }
        case 5: {
            HSD_DevComCallback cb = (rnd() & 1) ? cb2 : lbFile_8001615C;
            TRY(lbFile_80016580(name, (void*) dstv, &sz, cb, args));
            break;
        }
        case 6: TRY(lbFile_8001668C(name, (void*) dstv, &sz)); break;
        case 7: TRY(lbFile_80016760(name, &dst, &sz)); break;
        case 8: {
            int h = (int) (rnd() % 7);
            TRY(dst = lbFile_LoadToHeap(h, name, &sz));
            break;
        }
        case 9: {
            size_t cap = rnd() % 100000;
            void* d9 = (rnd() % 6 == 0) ? NULL : (void*) dstv;
            TRY(ret = lbFile_LoadInto(name, d9, cap, &sz));
            break;
        }
        case 10: {
            int h = (int) (rnd() % 7);
            TRY(ret = lbFile_800168A0(h, name, &dst, &sz));
            break;
        }
        case 11: {
            int cf = (int) (rnd() % 3 == 0);
            TRY(lbFile_8001615C(0, 0, NULL, cf));
            break;
        }
        }
        printf("%d f=%d name=%s asserted=%d ret=%d cancel=%d sz=%u dst=%s res=%s sum=%08x pend=%d g_cb2=%d log=%s\n", i, f, name, g_asserted, ret,
               (int) cancel, (unsigned) sz, pid(dst), g_res, (unsigned) sumres(), PEND.on, g_cb2, g_log);
    }
    return 0;
}
