extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/melee/lb/lbdvd.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static HSD_AllocEntry AE[8];
static Vec3 VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static char STRS[4][64] = { "a.dat", "sym", "LbAd.dat", "x" };

static HSD_AllocEntry* pick_HSD_AllocEntry(void) { u32 r = rnd(); return &AE[(r >> 8) % 8]; }
static HSD_AllocEntry* picko_HSD_AllocEntry(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &AE[(r >> 8) % 8]; }
static void cbev1(HSD_GObj* g) { T(7001); T(PI(g)); }
static void cbev2(HSD_GObj* g) { T(7002); T(PI(g)); }
static int cbp1(HSD_GObj* g) { T(7003); T(PI(g)); return (int) (fv() & 1); }
static int cbp2(HSD_GObj* g) { T(7004); T(PI(g)); return (int) (fv() & 1); }
static void cbev3(HSD_GObj* g, HSD_GObj* h) { T(7005); T(PI(g)); T(PI(h)); }
static void cbi1(int a) { T(7006); T((u32) a); }
static void cbi2(int a) { T(7007); T((u32) a); }
static void* pick_fn(int kind)
{
    u32 r = rnd();
    if ((r & 3) == 0) return 0;
    switch (kind) {
    case 1: return (r & 4) ? (void*) cbev1 : (void*) cbev2;
    case 2: return (r & 4) ? (void*) cbp1 : (void*) cbp2;
    case 3: return (void*) cbev3;
    case 4: return (r & 4) ? (void*) cbi1 : (void*) cbi2;
    }
    return 0;
}
static void* pick_cb(int kind) { void* f = pick_fn(kind); return f ? f : (kind == 2 ? (void*) cbp1 : kind == 3 ? (void*) cbev3 : kind == 4 ? (void*) cbi1 : (void*) cbev1); }
static void freg_world(void) { freg((const void*) cbev1); freg((const void*) cbev2); freg((const void*) cbp1); freg((const void*) cbp2); freg((const void*) cbev3); freg((const void*) cbi1); freg((const void*) cbi2); }

#include "lbdvd_gen.h"



static void preg_world(void)
{
    preg(AE, sizeof AE);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(AE, sizeof AE);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    for (i = 0; i < 8; i++) fill_HSD_AllocEntry(&AE[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 8; i++) hash_HSD_AllocEntry(&AE[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    hashg_all();
    hasho_all();
}

enum { OP_lbDvd_IsPlanningSixSlot = 0, OP_lbDvd_TrySetupSixSlotCache = 1, OP_lbDvd_SixBudgetTest = 2, OP_lbDvd_SetupVsPreloadCache = 3, OP_lbDvd_800174E8 = 4, OP_lbDvd_80017598 = 5, OP_lbDvd_80017644 = 6, OP_lbDvd_80017700 = 7, OP_lbDvd_80017740 = 8, OP_lbDvd_800178E8 = 9, OP_lbDvd_80017960 = 10, OP_lbDvd_CachePreloadedFile = 11, OP_lbDvd_80017CC4 = 12, OP_lbDvd_80017E64 = 13, OP_lbDvd_GetPreloadedArchive = 14, OP_lbDvd_GetPreloadCacheScene = 15, OP_lbDvd_8001823C = 16, OP_lbDvd_80018254 = 17, OP_lbDvd_800187F4 = 18, OP_lbDvd_80018A2C = 19, OP_lbDvd_80018C2C = 20, OP_lbDvd_80018C6C = 21, OP_lbDvd_80018CF4 = 22, OP_lbDvd_80018F58 = 23, OP_lbDvd_80018F68 = 24 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();

    for (op = 0; op < 25; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; break;  /* lbDvd_IsPlanningSixSlot */
        case 1: n = 300; break;  /* lbDvd_TrySetupSixSlotCache */
        case 2: n = 300; break;  /* lbDvd_SixBudgetTest */
        case 3: n = 300; break;  /* lbDvd_SetupVsPreloadCache */
        case 4: n = 300; break;  /* lbDvd_800174E8 */
        case 5: n = 300; break;  /* lbDvd_80017598 */
        case 6: n = 300; break;  /* lbDvd_80017644 */
        case 7: n = 300; break;  /* lbDvd_80017700 */
        case 8: n = 300; break;  /* lbDvd_80017740 */
        case 9: n = 300; break;  /* lbDvd_800178E8 */
        case 10: n = 300; break;  /* lbDvd_80017960 */
        case 11: n = 300; break;  /* lbDvd_CachePreloadedFile */
        case 12: n = 300; break;  /* lbDvd_80017CC4 */
        case 13: n = 300; break;  /* lbDvd_80017E64 */
        case 14: n = 300; break;  /* lbDvd_GetPreloadedArchive */
        case 15: n = 300; break;  /* lbDvd_GetPreloadCacheScene */
        case 16: n = 300; break;  /* lbDvd_8001823C */
        case 17: n = 300; break;  /* lbDvd_80018254 */
        case 18: n = 300; break;  /* lbDvd_800187F4 */
        case 19: n = 300; break;  /* lbDvd_80018A2C */
        case 20: n = 300; break;  /* lbDvd_80018C2C */
        case 21: n = 300; break;  /* lbDvd_80018C6C */
        case 22: n = 300; break;  /* lbDvd_80018CF4 */
        case 23: n = 300; break;  /* lbDvd_80018F58 */
        case 24: n = 300; break;  /* lbDvd_80018F68 */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();

            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            ARMED = 1;
            if (MTU_SETJMP()) { ARMED = 0; printf("%d %d ABORT\n", op, k); continue; }
            switch (op) {
            case 0: {   ret = (u32) lbDvd_IsPlanningSixSlot();  } break;
            case 1: {   ret = (u32) lbDvd_TrySetupSixSlotCache();  } break;
            case 2: {   ret = (u32) lbDvd_SixBudgetTest();  } break;
            case 3: {   lbDvd_SetupVsPreloadCache();  } break;
            case 4: {  int a0 = (int) (fv() % 80); lbDvd_800174E8(a0);  } break;
            case 5: {  int a0 = (int) (fv() % 8); ret = (u32) lbDvd_80017598(a0);  } break;
            case 6: {  int a0 = (int) (fv() % 8); ret = (u32) lbDvd_80017644(a0);  } break;
            case 7: {  int a0 = (int) (fv() % 8); lbDvd_80017700(a0);  } break;
            case 8: {  int a0 = fv(); int a1 = fv(); int a2 = fv(); int a3 = fv(); u32 a4 = fv(); int a5 = fv(); int a6 = fv(); u8 a7 = fv(); int a8 = fv(); { PreloadEntry* p = (PreloadEntry*) lbDvd_80017740(a0, a1, a2, a3, a4, a5, a6, a7, a8); ret = p ? (u32) (p - preloadCache.entries) + 1 : 0; }  } break;
            case 9: {  int a0 = fv(); const char * a1 = STRS[rnd() % 4]; int a2 = fv(); int a3 = fv(); int a4 = fv(); int a5 = fv(); int a6 = fv(); u8 a7 = fv(); int a8 = fv(); lbDvd_800178E8(a0, a1, a2, a3, a4, a5, a6, a7, a8);  } break;
            case 10: {   lbDvd_80017960();  } break;
            case 11: {  s32 a0 = (int) (fv() % 80); lbDvd_CachePreloadedFile(a0);  } break;
            case 12: {   lbDvd_80017CC4();  } break;
            case 13: {  int a0 = fv(); uintptr_t a1 = (uintptr_t) (fv() % 80); void * a2 = (void*) BUF; bool a3 = (fv() & 1); lbDvd_80017E64(a0, a1, a2, a3);  } break;
            case 14: {  ssize_t a0 = (ssize_t) (fv() % 90) - 5; ret = PI(lbDvd_GetPreloadedArchive(a0));  } break;
            case 15: {   ret = PI(lbDvd_GetPreloadCacheScene());  } break;
            case 16: {   lbDvd_8001823C();  } break;
            case 17: {   lbDvd_80018254();  } break;
            case 18: {  int a0 = (int) (fv() % 90) - 5; ret = (u32) lbDvd_800187F4(a0);  } break;
            case 19: {  u8 a0 = fv(); ret = (u32) lbDvd_80018A2C(a0);  } break;
            case 20: {  u8 a0 = fv(); lbDvd_80018C2C(a0);  } break;
            case 21: {   lbDvd_80018C6C();  } break;
            case 22: {  int a0 = fv(); lbDvd_80018CF4(a0);  } break;
            case 23: {  bool a0 = (fv() & 1); lbDvd_80018F58(a0);  } break;
            case 24: {   lbDvd_80018F68();  } break;
            }
            ARMED = 0;
            hash_world();

            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
