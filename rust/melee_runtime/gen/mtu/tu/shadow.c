extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/sysdolphin/baselib/shadow.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static HSD_Shadow P0[2];
static HSD_JObj P1[2];
static HSD_ViewingRect P2[2];
static HSD_CObj CO[2];
static HSD_TObj TO[2];
static HSD_ImageDesc IM[2];
static HSD_SList SL[3];
static HSD_ClassInfo CI[1];
static Vec3 VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static void* PPA[4];
static char STRS[4][64] = { "a.dat", "sym", "LbAd.dat", "x" };

static HSD_Shadow* pick_HSD_Shadow(void) { u32 r = rnd(); return &P0[(r >> 8) % 2]; }
static HSD_Shadow* picko_HSD_Shadow(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &P0[(r >> 8) % 2]; }
static HSD_JObj* pick_HSD_JObj(void) { u32 r = rnd(); return &P1[(r >> 8) % 2]; }
static HSD_JObj* picko_HSD_JObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &P1[(r >> 8) % 2]; }
static HSD_ViewingRect* pick_HSD_ViewingRect(void) { u32 r = rnd(); return &P2[(r >> 8) % 2]; }
static HSD_ViewingRect* picko_HSD_ViewingRect(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &P2[(r >> 8) % 2]; }
static HSD_CObj* pick_HSD_CObj(void) { u32 r = rnd(); return &CO[(r >> 8) % 2]; }
static HSD_CObj* picko_HSD_CObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &CO[(r >> 8) % 2]; }
static HSD_TObj* pick_HSD_TObj(void) { u32 r = rnd(); return &TO[(r >> 8) % 2]; }
static HSD_TObj* picko_HSD_TObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &TO[(r >> 8) % 2]; }
static HSD_ImageDesc* pick_HSD_ImageDesc(void) { u32 r = rnd(); return &IM[(r >> 8) % 2]; }
static HSD_ImageDesc* picko_HSD_ImageDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &IM[(r >> 8) % 2]; }
static HSD_SList* pick_HSD_SList(void) { u32 r = rnd(); return &SL[(r >> 8) % 3]; }
static HSD_SList* picko_HSD_SList(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &SL[(r >> 8) % 3]; }
static HSD_ClassInfo* pick_HSD_ClassInfo(void) { u32 r = rnd(); return &CI[(r >> 8) % 1]; }
static HSD_ClassInfo* picko_HSD_ClassInfo(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &CI[(r >> 8) % 1]; }
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

static void cb_rel(HSD_Class* c) { T(8203); T(PI(c)); }
static void cb_dest(HSD_Class* c) { T(8204); T(PI(c)); }

#include "shadow_gen.h"



static void preg_world(void)
{
    preg(P0, sizeof P0);
    preg(P1, sizeof P1);
    preg(P2, sizeof P2);
    preg(CO, sizeof CO);
    preg(TO, sizeof TO);
    preg(IM, sizeof IM);
    preg(SL, sizeof SL);
    preg(CI, sizeof CI);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS); preg(PPA, sizeof PPA);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(P0, sizeof P0);
    zero(P1, sizeof P1);
    zero(P2, sizeof P2);
    zero(CO, sizeof CO);
    zero(TO, sizeof TO);
    zero(IM, sizeof IM);
    zero(SL, sizeof SL);
    zero(CI, sizeof CI);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    for (i = 0; i < 4; i++) PPA[i] = 0;
    for (i = 0; i < 2; i++) fill_HSD_Shadow(&P0[i]);
    for (i = 0; i < 2; i++) fill_HSD_JObj(&P1[i]);
    for (i = 0; i < 2; i++) fill_HSD_ViewingRect(&P2[i]);
    for (i = 0; i < 2; i++) fill_HSD_CObj(&CO[i]);
    for (i = 0; i < 2; i++) fill_HSD_TObj(&TO[i]);
    for (i = 0; i < 2; i++) fill_HSD_ImageDesc(&IM[i]);
    for (i = 0; i < 3; i++) fill_HSD_SList(&SL[i]);
    for (i = 0; i < 1; i++) fill_HSD_ClassInfo(&CI[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 2; i++) hash_HSD_Shadow(&P0[i]);
    for (i = 0; i < 2; i++) hash_HSD_JObj(&P1[i]);
    for (i = 0; i < 2; i++) hash_HSD_ViewingRect(&P2[i]);
    for (i = 0; i < 2; i++) hash_HSD_CObj(&CO[i]);
    for (i = 0; i < 2; i++) hash_HSD_TObj(&TO[i]);
    for (i = 0; i < 2; i++) hash_HSD_ImageDesc(&IM[i]);
    for (i = 0; i < 3; i++) hash_HSD_SList(&SL[i]);
    for (i = 0; i < 1; i++) hash_HSD_ClassInfo(&CI[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    for (i = 0; i < 4; i++) HM(PI(PPA[i]));
    hashg_all();
    hasho_all();
}

enum { OP_HSD_ShadowGetAllocData = 0, OP_HSD_ShadowInitAllocData = 1, OP_makeShadowTObj = 2, OP_HSD_ShadowAlloc = 3, OP_HSD_ShadowRemove = 4, OP_HSD_ShadowInit = 5, OP_HSD_ShadowSetSize = 6, OP_drawBackgroundRect = 7, OP_HSD_ShadowStartRender = 8, OP_HSD_ShadowEndRender = 9, OP_HSD_ShadowSetActive = 10, OP_HSD_ShadowAddObject = 11, OP_HSD_ShadowDeleteObject = 12, OP_makeMatrix = 13, OP_HSD_ShadowSetViewingRect = 14, OP_HSD_ViewingRectInit = 15, OP_HSD_ViewingRectCheck = 16, OP_HSD_ViewingRectAddRect = 17 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();
freg((const void*) cb_rel); freg((const void*) cb_dest);
    for (op = 0; op < 18; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; break;  /* HSD_ShadowGetAllocData */
        case 1: n = 300; break;  /* HSD_ShadowInitAllocData */
        case 2: n = 300; break;  /* makeShadowTObj */
        case 3: n = 300; break;  /* HSD_ShadowAlloc */
        case 4: n = 300; break;  /* HSD_ShadowRemove */
        case 5: n = 300; break;  /* HSD_ShadowInit */
        case 6: n = 300; break;  /* HSD_ShadowSetSize */
        case 7: n = 300; break;  /* drawBackgroundRect */
        case 8: n = 300; break;  /* HSD_ShadowStartRender */
        case 9: n = 300; break;  /* HSD_ShadowEndRender */
        case 10: n = 300; break;  /* HSD_ShadowSetActive */
        case 11: n = 300; break;  /* HSD_ShadowAddObject */
        case 12: n = 300; break;  /* HSD_ShadowDeleteObject */
        case 13: n = 300; break;  /* makeMatrix */
        case 14: n = 300; break;  /* HSD_ShadowSetViewingRect */
        case 15: n = 300; break;  /* HSD_ViewingRectInit */
        case 16: n = 300; break;  /* HSD_ViewingRectCheck */
        case 17: n = 300; break;  /* HSD_ViewingRectAddRect */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ int q; CI[0].release = (void*) cb_rel; CI[0].destroy = (void*) cb_dest; for (q = 0; q < 2; q++) CO[q].parent.parent.class_info = &CI[0]; for (q = 0; q < 3; q++) { SL[q].next = (q < 2 && (fv() & 1)) ? &SL[q + 1] : 0; SL[q].data = &P1[fv() & 1]; } for (q = 0; q < 2; q++) { P0[q].camera = &CO[q]; P0[q].texture = &TO[q]; TO[q].imagedesc = &IM[q]; P0[q].objects = (fv() & 1) ? &SL[fv() % 3] : 0; IM[q].image_ptr = (fv() & 1) ? BUF : 0; IM[q].width = (u16) (fv() % 300); IM[q].height = (u16) (fv() % 300); } }
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            ARMED = 1;
            if (MTU_SETJMP()) { ARMED = 0; printf("%d %d ABORT\n", op, k); continue; }
            switch (op) {
            case 0: {   ret = PI(HSD_ShadowGetAllocData());  } break;
            case 1: {   HSD_ShadowInitAllocData();  } break;
            case 2: {   ret = PI(makeShadowTObj());  } break;
            case 3: {   ret = PI(HSD_ShadowAlloc());  } break;
            case 4: {  HSD_Shadow * a0 = pick_HSD_Shadow(); HSD_ShadowRemove(a0);  } break;
            case 5: {  HSD_Shadow * a0 = pick_HSD_Shadow(); HSD_ShadowInit(a0);  } break;
            case 6: {  HSD_Shadow * a0 = pick_HSD_Shadow(); u16 a1 = fv(); u16 a2 = fv(); HSD_ShadowSetSize(a0, a1, a2);  } break;
            case 7: {  HSD_Shadow * a0 = pick_HSD_Shadow(); drawBackgroundRect(a0);  } break;
            case 8: {  HSD_Shadow * a0 = pick_HSD_Shadow(); HSD_ShadowStartRender(a0);  } break;
            case 9: {  HSD_Shadow * a0 = pick_HSD_Shadow(); HSD_ShadowEndRender(a0);  } break;
            case 10: {  HSD_Shadow * a0 = pick_HSD_Shadow(); int a1 = fv(); HSD_ShadowSetActive(a0, a1);  } break;
            case 11: {  HSD_Shadow * a0 = pick_HSD_Shadow(); HSD_JObj * a1 = pick_HSD_JObj(); HSD_ShadowAddObject(a0, a1);  } break;
            case 12: {  HSD_Shadow * a0 = pick_HSD_Shadow(); HSD_JObj * a1 = pick_HSD_JObj(); HSD_ShadowDeleteObject(a0, a1);  } break;
            case 13: {  HSD_Shadow * a0 = pick_HSD_Shadow(); makeMatrix(a0);  } break;
            case 14: {  HSD_Shadow * a0 = pick_HSD_Shadow(); float a1 = rw(); float a2 = rw(); float a3 = rw(); float a4 = rw(); HSD_ShadowSetViewingRect(a0, a1, a2, a3, a4);  } break;
            case 15: {  HSD_ViewingRect * a0 = pick_HSD_ViewingRect(); Vec3 * a1 = &VA[rnd() % 4]; Vec3 * a2 = &VA[rnd() % 4]; Vec3 * a3 = &VA[rnd() % 4]; int a4 = fv(); HSD_ViewingRectInit(a0, a1, a2, a3, a4);  } break;
            case 16: {  HSD_ViewingRect * a0 = pick_HSD_ViewingRect(); ret = (u32) HSD_ViewingRectCheck(a0);  } break;
            case 17: {  HSD_ViewingRect * a0 = pick_HSD_ViewingRect(); Vec3 * a1 = &VA[rnd() % 4]; float a2 = rw(); float a3 = rw(); float a4 = rw(); float a5 = rw(); HSD_ViewingRectAddRect(a0, a1, a2, a3, a4, a5);  } break;
            }
            ARMED = 0;
            hash_world();

            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
