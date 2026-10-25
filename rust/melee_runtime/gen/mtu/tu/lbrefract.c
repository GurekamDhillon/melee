extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/melee/lb/lbrefract.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static lbRefract_CallbackData CB[2];
static HSD_PObj P1[2];
static HSD_PObjDesc P2[2];
static HSD_VtxDescList VD[6];
static HSD_PObjInfo PI_[1];
static Vec3 VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static void* PPA[4];
static char STRS[4][64] = { "a.dat", "sym", "LbAd.dat", "x" };

static lbRefract_CallbackData* pick_lbRefract_CallbackData(void) { u32 r = rnd(); return &CB[(r >> 8) % 2]; }
static lbRefract_CallbackData* picko_lbRefract_CallbackData(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &CB[(r >> 8) % 2]; }
static HSD_PObj* pick_HSD_PObj(void) { u32 r = rnd(); return &P1[(r >> 8) % 2]; }
static HSD_PObj* picko_HSD_PObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &P1[(r >> 8) % 2]; }
static HSD_PObjDesc* pick_HSD_PObjDesc(void) { u32 r = rnd(); return &P2[(r >> 8) % 2]; }
static HSD_PObjDesc* picko_HSD_PObjDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &P2[(r >> 8) % 2]; }
static HSD_VtxDescList* pick_HSD_VtxDescList(void) { u32 r = rnd(); return &VD[(r >> 8) % 6]; }
static HSD_VtxDescList* picko_HSD_VtxDescList(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &VD[(r >> 8) % 6]; }
static HSD_PObjInfo* pick_HSD_PObjInfo(void) { u32 r = rnd(); return &PI_[(r >> 8) % 1]; }
static HSD_PObjInfo* picko_HSD_PObjInfo(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &PI_[(r >> 8) % 1]; }
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

static unsigned char RFB[4096];
HSD_Archive* lbArchive_LoadSymbols(const char* f, void* out, ...) { (void) f; (void) out; return 0; }
float fabsf(float x) { union { float f; u32 u; } c; c.f = x; c.u &= 0x7FFFFFFFu; return c.f; }
static struct { u8 x0; f32* x4; } RD[1];
static int cb_load(HSD_PObj* p, HSD_PObjDesc* d) { T(8208); T(PI(p)); T(PI(d)); return (fv() & 3) ? 0 : (int) (fv() % 3) - 1; }

#include "lbrefract_gen.h"



static void preg_world(void)
{
    preg(CB, sizeof CB);
    preg(P1, sizeof P1);
    preg(P2, sizeof P2);
    preg(VD, sizeof VD);
    preg(PI_, sizeof PI_);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS); preg(PPA, sizeof PPA);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(CB, sizeof CB);
    zero(P1, sizeof P1);
    zero(P2, sizeof P2);
    zero(VD, sizeof VD);
    zero(PI_, sizeof PI_);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    for (i = 0; i < 4; i++) PPA[i] = 0;
    for (i = 0; i < 2; i++) fill_lbRefract_CallbackData(&CB[i]);
    for (i = 0; i < 2; i++) fill_HSD_PObj(&P1[i]);
    for (i = 0; i < 2; i++) fill_HSD_PObjDesc(&P2[i]);
    for (i = 0; i < 6; i++) fill_HSD_VtxDescList(&VD[i]);
    for (i = 0; i < 1; i++) fill_HSD_PObjInfo(&PI_[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 2; i++) hash_lbRefract_CallbackData(&CB[i]);
    for (i = 0; i < 2; i++) hash_HSD_PObj(&P1[i]);
    for (i = 0; i < 2; i++) hash_HSD_PObjDesc(&P2[i]);
    for (i = 0; i < 6; i++) hash_HSD_VtxDescList(&VD[i]);
    for (i = 0; i < 1; i++) hash_HSD_PObjInfo(&PI_[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    for (i = 0; i < 4; i++) HM(PI(PPA[i]));
    hashg_all();
    hasho_all();
}

enum { OP_lbRefract_80021CE8 = 0, OP_lbRefract_WriteTexCoordIA4 = 1, OP_lbRefract_PObjLoad = 2 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();
preg(RFB, sizeof RFB); preg(RD, sizeof RD); freg((const void*) cb_load); freg((const void*) lbRefract_WriteTexCoordIA4);
    for (op = 0; op < 3; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 400; break;  /* lbRefract_80021CE8 */
        case 1: n = 400; break;  /* lbRefract_WriteTexCoordIA4 */
        case 2: n = 400; break;  /* lbRefract_PObjLoad */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ int q; for (q = 0; q < 2; q++) { CB[q].buffer = RFB; CB[q].width = 1 + fv() % 32; CB[q].height = 1 + fv() % 32; CB[q].row_stride = 256; CB[q].buffer_size = 2048; CB[q].format = 3; CB[q].callback0 = (void*) lbRefract_WriteTexCoordIA4; CB[q].callback1 = 0; } for (q = 0; q < 4096; q++) RFB[q] = (unsigned char) rnd(); RD[0].x4 = FA; refract_data = (void*) &RD[0]; for (q = 0; q < 6; q++) { VD[q].attr = (fv() & 3) ? (fv() % 26) : 255; VD[q].attr_type = fv() % 4; VD[q].comp_type = fv() % 6; VD[q].comp_cnt = fv() % 2; } VD[5].attr = 255; for (q = 0; q < 2; q++) { P1[q].verts = (fv() & 3) ? &VD[fv() % 6] : 0; P1[q].display = RFB; P1[q].n_display = fv() % 4; P1[q].flags = (fv() & 1) ? 0x2000 : (fv() & 0xFFF); P1[q].parent.class_info = (HSD_ClassInfo*) &PI_[0]; } PI_[0].load = cb_load; hsdPObj.load = cb_load; }
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            ARMED = 1;
            if (MTU_SETJMP()) { ARMED = 0; printf("%d %d ABORT\n", op, k); continue; }
            switch (op) {
            case 0: {  lbRefract_CallbackData * a0 = &CB[fv() & 1]; s32 a1 = (s32) (fv() & 1); lbRefract_80021CE8(a0, a1);  } break;
            case 1: {  lbRefract_CallbackData * a0 = &CB[fv() & 1]; u32 a1 = rnd() % 32; u32 a2 = rnd() % 32; u32 a3 = fv(); u8 a4 = fv(); u8 a5 = fv(); u8 a6 = fv(); lbRefract_WriteTexCoordIA4(a0, a1, a2, a3, a4, a5, a6);  } break;
            case 2: {  HSD_PObj * a0 = pick_HSD_PObj(); HSD_PObjDesc * a1 = pick_HSD_PObjDesc(); ret = (u32) lbRefract_PObjLoad(a0, a1);  } break;
            }
            ARMED = 0;
            hash_world();
{ int i; for (i = 0; i < 4096; i++) HM(RFB[i]); }
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
