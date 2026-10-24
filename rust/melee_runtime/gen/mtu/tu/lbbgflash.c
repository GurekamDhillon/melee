extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/melee/lb/lbbgflash.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static HSD_GObj GO[4];
static GXColor COL[4];
static Vec3 VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static char STRS[4][64] = { "a.dat", "sym", "LbAd.dat", "x" };

static HSD_GObj* pick_HSD_GObj(void) { u32 r = rnd(); return &GO[(r >> 8) % 4]; }
static HSD_GObj* picko_HSD_GObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &GO[(r >> 8) % 4]; }
static GXColor* pick_GXColor(void) { u32 r = rnd(); return &COL[(r >> 8) % 4]; }
static GXColor* picko_GXColor(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &COL[(r >> 8) % 4]; }
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

#include "lbbgflash_gen.h"



static void preg_world(void)
{
    preg(GO, sizeof GO);
    preg(COL, sizeof COL);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(GO, sizeof GO);
    zero(COL, sizeof COL);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    for (i = 0; i < 4; i++) fill_HSD_GObj(&GO[i]);
    for (i = 0; i < 4; i++) fill_GXColor(&COL[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 4; i++) hash_HSD_GObj(&GO[i]);
    for (i = 0; i < 4; i++) hash_GXColor(&COL[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    hashg_all();
    hasho_all();
}

enum { OP_bgflash_position = 0, OP_fn_8001FC08 = 1, OP_fn_8001FEC4 = 2, OP_fn_800204C8 = 3, OP_lbBgFlash_800205F0 = 4, OP_lbBgFlash_8002063C = 5, OP_lbBgFlash_80020688 = 6, OP_lbBgFlash_800206D4 = 7, OP_lbBgFlash_InitState = 8, OP_fn_800208B0 = 9, OP_lbBgFlash_800208EC = 10, OP_lbBgFlash_800209F4 = 11 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();

    for (op = 0; op < 12; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; break;  /* bgflash_position */
        case 1: n = 300; break;  /* fn_8001FC08 */
        case 2: n = 300; break;  /* fn_8001FEC4 */
        case 3: n = 300; break;  /* fn_800204C8 */
        case 4: n = 300; break;  /* lbBgFlash_800205F0 */
        case 5: n = 300; break;  /* lbBgFlash_8002063C */
        case 6: n = 300; break;  /* lbBgFlash_80020688 */
        case 7: n = 300; break;  /* lbBgFlash_800206D4 */
        case 8: n = 300; break;  /* lbBgFlash_InitState */
        case 9: n = 300; break;  /* fn_800208B0 */
        case 10: n = 300; break;  /* lbBgFlash_800208EC */
        case 11: n = 300; break;  /* lbBgFlash_800209F4 */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ lbl_80433658.x32 = (u8) (1 + fv() % 60); lbl_80433658.x38 = (int) (fv() % 700) - 100; lbl_80433658.x3C = (int) (fv() % 40); lbl_80433658.x34 = (int) (fv() % 900) - 50; lbl_80433658.x30 = (fv() & 3) ? 0 : (u8) fv(); lbl_80433658.x33 = (fv() & 3) ? 0 : 1; }
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            switch (op) {
            case 0: {  f32 a0 = rw(); f32 a1 = rw(); bgflash_position(a0, a1);  } break;
            case 1: {   fn_8001FC08();  } break;
            case 2: {  HSD_GObj * a0 = pick_HSD_GObj(); s32 a1 = fv(); fn_8001FEC4(a0, a1);  } break;
            case 3: {   fn_800204C8();  } break;
            case 4: {  s32 a0 = fv(); lbBgFlash_800205F0(a0);  } break;
            case 5: {  int a0 = fv(); lbBgFlash_8002063C(a0);  } break;
            case 6: {  int a0 = fv(); lbBgFlash_80020688(a0);  } break;
            case 7: {  GXColor * a0 = pick_GXColor(); GXColor * a1 = pick_GXColor(); int a2 = fv(); lbBgFlash_800206D4(a0, a1, a2);  } break;
            case 8: {  GXColor * a0 = pick_GXColor(); lbBgFlash_InitState(a0);  } break;
            case 9: {  u8 a0 = fv(); fn_800208B0(a0);  } break;
            case 10: {  int a0 = fv(); lbBgFlash_800208EC(a0);  } break;
            case 11: {   lbBgFlash_800209F4();  } break;
            }
            hash_world();

            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
