extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/sysdolphin/baselib/tev.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static HSD_Chan CH[3];
static HSD_TevDesc TD[3];
static struct { float x, y, z; } VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static void* PPA[4];
static char STRS[4][64] = { "a.dat", "sym", "LbAd.dat", "x" };

static HSD_Chan* pick_HSD_Chan(void) { u32 r = rnd(); return &CH[(r >> 8) % 3]; }
static HSD_Chan* picko_HSD_Chan(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &CH[(r >> 8) % 3]; }
static HSD_TevDesc* pick_HSD_TevDesc(void) { u32 r = rnd(); return &TD[(r >> 8) % 3]; }
static HSD_TevDesc* picko_HSD_TevDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &TD[(r >> 8) % 3]; }
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

#include "tev_native_gen.h"



static void preg_world(void)
{
    preg(CH, sizeof CH);
    preg(TD, sizeof TD);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS); preg(PPA, sizeof PPA);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(CH, sizeof CH);
    zero(TD, sizeof TD);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    for (i = 0; i < 4; i++) PPA[i] = 0;
    for (i = 0; i < 3; i++) fill_HSD_Chan(&CH[i]);
    for (i = 0; i < 3; i++) fill_HSD_TevDesc(&TD[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 3; i++) hash_HSD_Chan(&CH[i]);
    for (i = 0; i < 3; i++) hash_HSD_TevDesc(&TD[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    for (i = 0; i < 4; i++) HM(PI(PPA[i]));
    hashg_all();
    hasho_all();
}

enum { OP_HSD_RenderInitAllocData = 0, OP_HSD_RenderGetAllocData = 1, OP_HSD_TevRegGetAllocData = 2, OP_HSD_ChanGetAllocData = 3, OP_HSD_SetupChannel = 4, OP_HSD_StateSetNumChans = 5, OP_HSD_SetupChannelAll = 6, OP_HSD_StateRegisterTexGen = 7, OP_HSD_StateSetNumTexGens = 8, OP_HSD_StateInitTev = 9, OP_HSD_StateGetCommittedTevStages = 10, OP_HSD_StateGetNumTevStages = 11, OP_HSD_StateAssignTev = 12, OP_HSD_StateSetNumTevStages = 13, OP_HSD_SetupTevStage = 14, OP_HSD_SetupTevStageAll = 15, OP_HSD_Channel2Num = 16, OP_HSD_Index2TevStage = 17, OP_HSD_TevStage2Index = 18, OP_HSD_TevStage2Num = 19, OP_HSD_SetTevRegAll = 20, OP_HSD_TexCoordID2Num = 21, OP_ChanUpdateFunc = 22, OP__HSD_StateInvalidateColorChannel = 23, OP__HSD_StateInvalidateTevStage = 24, OP__HSD_StateInvalidateTevRegister = 25, OP__HSD_StateInvalidateTexCoordGen = 26 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();

    for (op = 0; op < 27; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 400; break;  /* HSD_RenderInitAllocData */
        case 1: n = 400; break;  /* HSD_RenderGetAllocData */
        case 2: n = 400; break;  /* HSD_TevRegGetAllocData */
        case 3: n = 400; break;  /* HSD_ChanGetAllocData */
        case 4: n = 400; break;  /* HSD_SetupChannel */
        case 5: n = 400; break;  /* HSD_StateSetNumChans */
        case 6: n = 400; break;  /* HSD_SetupChannelAll */
        case 7: n = 400; break;  /* HSD_StateRegisterTexGen */
        case 8: n = 400; break;  /* HSD_StateSetNumTexGens */
        case 9: n = 400; break;  /* HSD_StateInitTev */
        case 10: n = 400; break;  /* HSD_StateGetCommittedTevStages */
        case 11: n = 400; break;  /* HSD_StateGetNumTevStages */
        case 12: n = 400; break;  /* HSD_StateAssignTev */
        case 13: n = 400; break;  /* HSD_StateSetNumTevStages */
        case 14: n = 400; break;  /* HSD_SetupTevStage */
        case 15: n = 400; break;  /* HSD_SetupTevStageAll */
        case 16: n = 400; break;  /* HSD_Channel2Num */
        case 17: n = 400; break;  /* HSD_Index2TevStage */
        case 18: n = 400; break;  /* HSD_TevStage2Index */
        case 19: n = 400; break;  /* HSD_TevStage2Num */
        case 20: n = 400; break;  /* HSD_SetTevRegAll */
        case 21: n = 400; break;  /* HSD_TexCoordID2Num */
        case 22: n = 400; break;  /* ChanUpdateFunc */
        case 23: n = 400; break;  /* _HSD_StateInvalidateColorChannel */
        case 24: n = 400; break;  /* _HSD_StateInvalidateTevStage */
        case 25: n = 400; break;  /* _HSD_StateInvalidateTevRegister */
        case 26: n = 400; break;  /* _HSD_StateInvalidateTexCoordGen */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ int q; for (q = 0; q < 3; q++) { CH[q].chan = ((fv() % 8) == 7) ? 0xFF : fv() % 6; CH[q].next = (q < 2 && (fv() & 1)) ? &CH[q + 1] : 0; CH[q].enable = fv() & 1; CH[q].amb_src = fv() & 1; CH[q].mat_src = fv() & 1; TD[q].stage = fv() % 16; TD[q].next = (q < 2 && (fv() & 1)) ? &TD[q + 1] : 0; TD[q].flags = fv() & 1; TD[q].coord = fv() % 8; TD[q].map = fv() % 8; TD[q].color = fv() % 8; } current_tev = fv() % 16; for (q = 0; q < 2; q++) { prev_mat_invalid[q] = fv() & 1; prev_amb_invalid[q] = fv() & 1; } for (q = 0; q < 4; q++) TevReg[q].c = fv() & 1; }
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            ARMED = 1;
            if (MTU_SETJMP()) { ARMED = 0; printf("%d %d ABORT\n", op, k); continue; }
            switch (op) {
            case 0: {   HSD_RenderInitAllocData();  } break;
            case 1: {   ret = PI(HSD_RenderGetAllocData());  } break;
            case 2: {   ret = PI(HSD_TevRegGetAllocData());  } break;
            case 3: {   ret = PI(HSD_ChanGetAllocData());  } break;
            case 4: {  HSD_Chan * a0 = pick_HSD_Chan(); HSD_SetupChannel(a0);  } break;
            case 5: {  int a0 = fv(); HSD_StateSetNumChans(a0);  } break;
            case 6: {  HSD_Chan * a0 = pick_HSD_Chan(); HSD_SetupChannelAll(a0);  } break;
            case 7: {  int a0 = fv(); HSD_StateRegisterTexGen(a0);  } break;
            case 8: {   HSD_StateSetNumTexGens();  } break;
            case 9: {   HSD_StateInitTev();  } break;
            case 10: {   ret = (u32) HSD_StateGetCommittedTevStages();  } break;
            case 11: {   ret = (u32) HSD_StateGetNumTevStages();  } break;
            case 12: {   ret = (u32) HSD_StateAssignTev();  } break;
            case 13: {   HSD_StateSetNumTevStages();  } break;
            case 14: {  HSD_TevDesc * a0 = pick_HSD_TevDesc(); HSD_SetupTevStage(a0);  } break;
            case 15: {  HSD_TevDesc * a0 = pick_HSD_TevDesc(); HSD_SetupTevStageAll(a0);  } break;
            case 16: {  int a0 = fv(); ret = (u32) HSD_Channel2Num(a0);  } break;
            case 17: {  int a0 = fv(); ret = (u32) HSD_Index2TevStage(a0);  } break;
            case 18: {  int a0 = fv(); ret = (u32) HSD_TevStage2Index(a0);  } break;
            case 19: {  int a0 = fv(); ret = (u32) HSD_TevStage2Num(a0);  } break;
            case 20: {   HSD_SetTevRegAll();  } break;
            case 21: {  int a0 = fv(); ret = (u32) HSD_TexCoordID2Num(a0);  } break;
            case 22: {  HSD_Chan * a0 = pick_HSD_Chan(); int a1 = fv(); const f32 * a2 = &FA[rnd() % 4]; ChanUpdateFunc(a0, a1, a2);  } break;
            case 23: {   _HSD_StateInvalidateColorChannel();  } break;
            case 24: {   _HSD_StateInvalidateTevStage();  } break;
            case 25: {   _HSD_StateInvalidateTevRegister();  } break;
            case 26: {   _HSD_StateInvalidateTexCoordGen();  } break;
            }
            ARMED = 0;
            hash_world();

            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
