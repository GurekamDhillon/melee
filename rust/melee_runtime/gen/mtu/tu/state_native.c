extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/sysdolphin/baselib/state.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static HSD_LObj L[3];
static HSD_PEDesc PE[2];
static Vec3 VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static void* PPA[4];
static char STRS[4][64] = { "a.dat", "sym", "LbAd.dat", "x" };

static HSD_LObj* pick_HSD_LObj(void) { u32 r = rnd(); return &L[(r >> 8) % 3]; }
static HSD_LObj* picko_HSD_LObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &L[(r >> 8) % 3]; }
static HSD_PEDesc* pick_HSD_PEDesc(void) { u32 r = rnd(); return &PE[(r >> 8) % 2]; }
static HSD_PEDesc* picko_HSD_PEDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &PE[(r >> 8) % 2]; }
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

static GXColor pick_col(void) { GXColor c; c.r = (u8) fv(); c.g = (u8) fv(); c.b = (u8) fv(); c.a = (u8) fv(); return c; }

#include "state_native_gen.h"



static void preg_world(void)
{
    preg(L, sizeof L);
    preg(PE, sizeof PE);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS); preg(PPA, sizeof PPA);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(L, sizeof L);
    zero(PE, sizeof PE);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    for (i = 0; i < 4; i++) PPA[i] = 0;
    for (i = 0; i < 3; i++) fill_HSD_LObj(&L[i]);
    for (i = 0; i < 2; i++) fill_HSD_PEDesc(&PE[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 3; i++) hash_HSD_LObj(&L[i]);
    for (i = 0; i < 2; i++) hash_HSD_PEDesc(&PE[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    for (i = 0; i < 4; i++) HM(PI(PPA[i]));
    hashg_all();
    hasho_all();
}

enum { OP_HSD_SetupChannelMode = 0, OP_HSD_SetupPEMode = 1, OP_HSD_SetupRenderModeWithCustomPE = 2, OP_HSD_SetupRenderMode = 3, OP_HSD_SetMaterialColor = 4, OP_HSD_SetMaterialShininess = 5, OP_HSD_StateSetLineWidth = 6, OP_HSD_StateSetCullMode = 7, OP_HSD_StateSetBlendMode = 8, OP_HSD_StateSetZMode = 9, OP_HSD_StateSetPointSize = 10, OP_HSD_StateSetAlphaCompare = 11, OP_HSD_StateSetColorUpdate = 12, OP_HSD_StateSetAlphaUpdate = 13, OP_HSD_StateSetDstAlpha = 14, OP_HSD_StateSetZCompLoc = 15, OP_HSD_StateSetDither = 16, OP__HSD_StateInvalidatePrimitive = 17, OP__HSD_StateInvalidateVtxAttr = 18, OP__HSD_StateInvalidateRenderMode = 19, OP_HSD_StateInvalidate = 20 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();

    for (op = 0; op < 21; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; break;  /* HSD_SetupChannelMode */
        case 1: n = 300; break;  /* HSD_SetupPEMode */
        case 2: n = 300; break;  /* HSD_SetupRenderModeWithCustomPE */
        case 3: n = 300; break;  /* HSD_SetupRenderMode */
        case 4: n = 300; break;  /* HSD_SetMaterialColor */
        case 5: n = 300; break;  /* HSD_SetMaterialShininess */
        case 6: n = 300; break;  /* HSD_StateSetLineWidth */
        case 7: n = 300; break;  /* HSD_StateSetCullMode */
        case 8: n = 300; break;  /* HSD_StateSetBlendMode */
        case 9: n = 300; break;  /* HSD_StateSetZMode */
        case 10: n = 300; break;  /* HSD_StateSetPointSize */
        case 11: n = 300; break;  /* HSD_StateSetAlphaCompare */
        case 12: n = 300; break;  /* HSD_StateSetColorUpdate */
        case 13: n = 300; break;  /* HSD_StateSetAlphaUpdate */
        case 14: n = 300; break;  /* HSD_StateSetDstAlpha */
        case 15: n = 300; break;  /* HSD_StateSetZCompLoc */
        case 16: n = 300; break;  /* HSD_StateSetDither */
        case 17: n = 300; break;  /* _HSD_StateInvalidatePrimitive */
        case 18: n = 300; break;  /* _HSD_StateInvalidateVtxAttr */
        case 19: n = 300; break;  /* _HSD_StateInvalidateRenderMode */
        case 20: n = 300; break;  /* HSD_StateInvalidate */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();

            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            ARMED = 1;
            if (MTU_SETJMP()) { ARMED = 0; printf("%d %d ABORT\n", op, k); continue; }
            switch (op) {
            case 0: {  u32 a0 = fv(); HSD_SetupChannelMode(a0);  } break;
            case 1: {  u32 a0 = fv(); HSD_PEDesc * a1 = pick_HSD_PEDesc(); HSD_SetupPEMode(a0, a1);  } break;
            case 2: {  u32 a0 = fv(); HSD_PEDesc * a1 = pick_HSD_PEDesc(); HSD_SetupRenderModeWithCustomPE(a0, a1);  } break;
            case 3: {  u32 a0 = fv(); HSD_SetupRenderMode(a0);  } break;
            case 4: {  GXColor a0 = pick_col(); GXColor a1 = pick_col(); GXColor a2 = pick_col(); f32 a3 = rw(); HSD_SetMaterialColor(a0, a1, a2, a3);  } break;
            case 5: {  f32 a0 = rw(); HSD_SetMaterialShininess(a0);  } break;
            case 6: {  u8 a0 = fv(); int a1 = fv(); HSD_StateSetLineWidth(a0, a1);  } break;
            case 7: {  int a0 = fv(); HSD_StateSetCullMode(a0);  } break;
            case 8: {  int a0 = fv(); int a1 = fv(); int a2 = fv(); int a3 = fv(); HSD_StateSetBlendMode(a0, a1, a2, a3);  } break;
            case 9: {  int a0 = fv(); int a1 = fv(); int a2 = fv(); HSD_StateSetZMode(a0, a1, a2);  } break;
            case 10: {  u8 a0 = fv(); int a1 = fv(); HSD_StateSetPointSize(a0, a1);  } break;
            case 11: {  int a0 = fv(); u8 a1 = fv(); int a2 = fv(); int a3 = fv(); u8 a4 = fv(); HSD_StateSetAlphaCompare(a0, a1, a2, a3, a4);  } break;
            case 12: {  int a0 = fv(); HSD_StateSetColorUpdate(a0);  } break;
            case 13: {  int a0 = fv(); HSD_StateSetAlphaUpdate(a0);  } break;
            case 14: {  int a0 = fv(); u8 a1 = fv(); HSD_StateSetDstAlpha(a0, a1);  } break;
            case 15: {  int a0 = fv(); HSD_StateSetZCompLoc(a0);  } break;
            case 16: {  int a0 = fv(); HSD_StateSetDither(a0);  } break;
            case 17: {   _HSD_StateInvalidatePrimitive();  } break;
            case 18: {   _HSD_StateInvalidateVtxAttr();  } break;
            case 19: {   _HSD_StateInvalidateRenderMode();  } break;
            case 20: {  int a0 = fv(); HSD_StateInvalidate(a0);  } break;
            }
            ARMED = 0;
            hash_world();

            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
