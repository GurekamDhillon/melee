extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/melee/lb/lbcollision.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static HitCapsule P0[2];
static HitResult P2[2];
static HSD_JObj P8[2];
static Vec3 VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static void* PPA[4];
static char STRS[4][64] = { "a.dat", "sym", "LbAd.dat", "x" };

static HitCapsule* pick_HitCapsule(void) { u32 r = rnd(); return &P0[(r >> 8) % 2]; }
static HitCapsule* picko_HitCapsule(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &P0[(r >> 8) % 2]; }
static HitResult* pick_HitResult(void) { u32 r = rnd(); return &P2[(r >> 8) % 2]; }
static HitResult* picko_HitResult(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &P2[(r >> 8) % 2]; }
static HSD_JObj* pick_HSD_JObj(void) { u32 r = rnd(); return &P8[(r >> 8) % 2]; }
static HSD_JObj* picko_HSD_JObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &P8[(r >> 8) % 2]; }
struct HSD_GObj; typedef struct HSD_GObj HSD_GObj;
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
static void freg_world(void) { freg((const void*) cbev1); freg((const void*) cbev2); freg((const void*) cbp1); freg((const void*) cbp2); freg((const void*) cbev3); freg((const void*) cbi1); freg((const void*) cbi2);  }

#include "lbcollision_gen.h"



static void preg_world(void)
{
    preg(P0, sizeof P0);
    preg(P2, sizeof P2);
    preg(P8, sizeof P8);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS); preg(PPA, sizeof PPA);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(P0, sizeof P0);
    zero(P2, sizeof P2);
    zero(P8, sizeof P8);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    for (i = 0; i < 4; i++) PPA[i] = 0;
    for (i = 0; i < 2; i++) fill_HitCapsule(&P0[i]);
    for (i = 0; i < 2; i++) fill_HitResult(&P2[i]);
    for (i = 0; i < 2; i++) fill_HSD_JObj(&P8[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 2; i++) hash_HitCapsule(&P0[i]);
    for (i = 0; i < 2; i++) hash_HitResult(&P2[i]);
    for (i = 0; i < 2; i++) hash_HSD_JObj(&P8[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    for (i = 0; i < 4; i++) HM(PI(PPA[i]));
    hashg_all();
    hasho_all();
}

enum { OP_lbColl_80006E58 = 0, OP_lbColl_80007BCC = 1 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();

    for (op = 0; op < 2; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 1500; break;  /* lbColl_80006E58 */
        case 1: n = 1500; break;  /* lbColl_80007BCC */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ int q; for (q = 0; q < 2; q++) { P2[q].bone = &P8[q]; P2[q].skip_update_pos = fv() & 1; } }
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            ARMED = 1;
            if (MTU_SETJMP()) { ARMED = 0; printf("%d %d ABORT\n", op, k); continue; }
            switch (op) {
            case 0: {  Vec3 * a0 = &VA[rnd() % 4]; Vec3 * a1 = &VA[rnd() % 4]; Vec3 * a2 = &VA[rnd() % 4]; Vec3 * a3 = &VA[rnd() % 4]; Vec3 * a4 = &VA[rnd() % 4]; Vec3 * a5 = &VA[rnd() % 4]; MtxPtr a6 = (MtxPtr) BUF; Vec3 * a7 = &VA[rnd() % 4]; float * a8 = &FA[rnd() % 4]; float a9 = rw(); float a10 = rw(); float a11 = rw(); ret = (u32) lbColl_80006E58(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11);  } break;
            case 1: {  HitCapsule * a0 = pick_HitCapsule(); HitResult * a1 = pick_HitResult(); void * a2 = ((fv() & 1) ? (void*) BUF : 0); s32 a3 = fv(); float a4 = rw(); float a5 = rw(); float a6 = rw(); ret = (u32) lbColl_80007BCC(a0, a1, a2, a3, a4, a5, a6);  } break;
            }
            ARMED = 0;
            hash_world();

            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
