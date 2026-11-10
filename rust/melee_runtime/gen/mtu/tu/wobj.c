extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/sysdolphin/baselib/wobj.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static HSD_WObj WO[3];
static HSD_AObj AO[3];
static HSD_RObj RO[2];
static HSD_JObj JO[2];
static HSD_WObjInfo WI[2];
static HSD_ClassInfo CI[2];
static HSD_Spline SPL[1];
static HSD_ObjData OBD[2];
static HSD_WObjDesc WD[2];
static HSD_WObjAnim WA[2];
static HSD_AObjDesc AD[2];
static HSD_RObjDesc RD[2];
static HSD_RObjAnimJoint RJ[2];
static Vec3 VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static void* PPA[4];
static char STRS[4][64] = { "a.dat", "sym", "LbAd.dat", "x" };

static HSD_WObj* pick_HSD_WObj(void) { u32 r = rnd(); return &WO[(r >> 8) % 3]; }
static HSD_WObj* picko_HSD_WObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &WO[(r >> 8) % 3]; }
static HSD_AObj* pick_HSD_AObj(void) { u32 r = rnd(); return &AO[(r >> 8) % 3]; }
static HSD_AObj* picko_HSD_AObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &AO[(r >> 8) % 3]; }
static HSD_RObj* pick_HSD_RObj(void) { u32 r = rnd(); return &RO[(r >> 8) % 2]; }
static HSD_RObj* picko_HSD_RObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &RO[(r >> 8) % 2]; }
static HSD_JObj* pick_HSD_JObj(void) { u32 r = rnd(); return &JO[(r >> 8) % 2]; }
static HSD_JObj* picko_HSD_JObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &JO[(r >> 8) % 2]; }
static HSD_WObjInfo* pick_HSD_WObjInfo(void) { u32 r = rnd(); return &WI[(r >> 8) % 2]; }
static HSD_WObjInfo* picko_HSD_WObjInfo(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &WI[(r >> 8) % 2]; }
static HSD_ClassInfo* pick_HSD_ClassInfo(void) { u32 r = rnd(); return &CI[(r >> 8) % 2]; }
static HSD_ClassInfo* picko_HSD_ClassInfo(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &CI[(r >> 8) % 2]; }
static HSD_Spline* pick_HSD_Spline(void) { u32 r = rnd(); return &SPL[(r >> 8) % 1]; }
static HSD_Spline* picko_HSD_Spline(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &SPL[(r >> 8) % 1]; }
static HSD_ObjData* pick_HSD_ObjData(void) { u32 r = rnd(); return &OBD[(r >> 8) % 2]; }
static HSD_ObjData* picko_HSD_ObjData(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &OBD[(r >> 8) % 2]; }
static HSD_WObjDesc* pick_HSD_WObjDesc(void) { u32 r = rnd(); return &WD[(r >> 8) % 2]; }
static HSD_WObjDesc* picko_HSD_WObjDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &WD[(r >> 8) % 2]; }
static HSD_WObjAnim* pick_HSD_WObjAnim(void) { u32 r = rnd(); return &WA[(r >> 8) % 2]; }
static HSD_WObjAnim* picko_HSD_WObjAnim(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &WA[(r >> 8) % 2]; }
static HSD_AObjDesc* pick_HSD_AObjDesc(void) { u32 r = rnd(); return &AD[(r >> 8) % 2]; }
static HSD_AObjDesc* picko_HSD_AObjDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &AD[(r >> 8) % 2]; }
static HSD_RObjDesc* pick_HSD_RObjDesc(void) { u32 r = rnd(); return &RD[(r >> 8) % 2]; }
static HSD_RObjDesc* picko_HSD_RObjDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &RD[(r >> 8) % 2]; }
static HSD_RObjAnimJoint* pick_HSD_RObjAnimJoint(void) { u32 r = rnd(); return &RJ[(r >> 8) % 2]; }
static HSD_RObjAnimJoint* picko_HSD_RObjAnimJoint(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &RJ[(r >> 8) % 2]; }
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

#include "wobj_gen.h"



static void preg_world(void)
{
    preg(WO, sizeof WO);
    preg(AO, sizeof AO);
    preg(RO, sizeof RO);
    preg(JO, sizeof JO);
    preg(WI, sizeof WI);
    preg(CI, sizeof CI);
    preg(SPL, sizeof SPL);
    preg(OBD, sizeof OBD);
    preg(WD, sizeof WD);
    preg(WA, sizeof WA);
    preg(AD, sizeof AD);
    preg(RD, sizeof RD);
    preg(RJ, sizeof RJ);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS); preg(PPA, sizeof PPA);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(WO, sizeof WO);
    zero(AO, sizeof AO);
    zero(RO, sizeof RO);
    zero(JO, sizeof JO);
    zero(WI, sizeof WI);
    zero(CI, sizeof CI);
    zero(SPL, sizeof SPL);
    zero(OBD, sizeof OBD);
    zero(WD, sizeof WD);
    zero(WA, sizeof WA);
    zero(AD, sizeof AD);
    zero(RD, sizeof RD);
    zero(RJ, sizeof RJ);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    for (i = 0; i < 4; i++) PPA[i] = 0;
    for (i = 0; i < 3; i++) fill_HSD_WObj(&WO[i]);
    for (i = 0; i < 3; i++) fill_HSD_AObj(&AO[i]);
    for (i = 0; i < 2; i++) fill_HSD_RObj(&RO[i]);
    for (i = 0; i < 2; i++) fill_HSD_JObj(&JO[i]);
    for (i = 0; i < 2; i++) fill_HSD_WObjInfo(&WI[i]);
    for (i = 0; i < 2; i++) fill_HSD_ClassInfo(&CI[i]);
    for (i = 0; i < 1; i++) fill_HSD_Spline(&SPL[i]);
    for (i = 0; i < 2; i++) fill_HSD_ObjData(&OBD[i]);
    for (i = 0; i < 2; i++) fill_HSD_WObjDesc(&WD[i]);
    for (i = 0; i < 2; i++) fill_HSD_WObjAnim(&WA[i]);
    for (i = 0; i < 2; i++) fill_HSD_AObjDesc(&AD[i]);
    for (i = 0; i < 2; i++) fill_HSD_RObjDesc(&RD[i]);
    for (i = 0; i < 2; i++) fill_HSD_RObjAnimJoint(&RJ[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 3; i++) hash_HSD_WObj(&WO[i]);
    for (i = 0; i < 3; i++) hash_HSD_AObj(&AO[i]);
    for (i = 0; i < 2; i++) hash_HSD_RObj(&RO[i]);
    for (i = 0; i < 2; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 2; i++) hash_HSD_WObjInfo(&WI[i]);
    for (i = 0; i < 2; i++) hash_HSD_ClassInfo(&CI[i]);
    for (i = 0; i < 1; i++) hash_HSD_Spline(&SPL[i]);
    for (i = 0; i < 2; i++) hash_HSD_ObjData(&OBD[i]);
    for (i = 0; i < 2; i++) hash_HSD_WObjDesc(&WD[i]);
    for (i = 0; i < 2; i++) hash_HSD_WObjAnim(&WA[i]);
    for (i = 0; i < 2; i++) hash_HSD_AObjDesc(&AD[i]);
    for (i = 0; i < 2; i++) hash_HSD_RObjDesc(&RD[i]);
    for (i = 0; i < 2; i++) hash_HSD_RObjAnimJoint(&RJ[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    for (i = 0; i < 4; i++) HM(PI(PPA[i]));
    hashg_all();
    hasho_all();
}

enum { OP_HSD_WObjRemoveAnim = 0, OP_HSD_WObjReqAnim = 1, OP_HSD_WObjAddAnim = 2, OP_WObjUpdateFunc = 3, OP_HSD_WObjInterpretAnim = 4, OP_WObjLoad = 5, OP_HSD_WObjInit = 6, OP_HSD_WObjSetDefaultClass = 7, OP_HSD_WObjLoadDesc = 8, OP_HSD_WObjSetPosition = 9, OP_HSD_WObjSetPositionX = 10, OP_HSD_WObjSetPositionY = 11, OP_HSD_WObjSetPositionZ = 12, OP_HSD_WObjGetPosition = 13, OP_HSD_WObjAlloc = 14, OP_WObjRelease = 15, OP_WObjAmnesia = 16, OP_WObjInfoInit = 17 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();

    for (op = 0; op < 18; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; break;  /* HSD_WObjRemoveAnim */
        case 1: n = 300; break;  /* HSD_WObjReqAnim */
        case 2: n = 300; break;  /* HSD_WObjAddAnim */
        case 3: n = 300; break;  /* WObjUpdateFunc */
        case 4: n = 300; break;  /* HSD_WObjInterpretAnim */
        case 5: n = 300; break;  /* WObjLoad */
        case 6: n = 300; break;  /* HSD_WObjInit */
        case 7: n = 300; break;  /* HSD_WObjSetDefaultClass */
        case 8: n = 300; break;  /* HSD_WObjLoadDesc */
        case 9: n = 300; break;  /* HSD_WObjSetPosition */
        case 10: n = 300; break;  /* HSD_WObjSetPositionX */
        case 11: n = 300; break;  /* HSD_WObjSetPositionY */
        case 12: n = 300; break;  /* HSD_WObjSetPositionZ */
        case 13: n = 300; break;  /* HSD_WObjGetPosition */
        case 14: n = 300; break;  /* HSD_WObjAlloc */
        case 15: n = 300; break;  /* WObjRelease */
        case 16: n = 300; break;  /* WObjAmnesia */
        case 17: n = 300; break;  /* WObjInfoInit */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ int q; for (q = 0; q < 2; q++) WI[q].load = WObjLoad; for (q = 0; q < 3; q++) { WO[q].parent.parent.class_info = (HSD_ClassInfo*) &WI[q & 1]; WO[q].aobj = (fv() & 3) ? &AO[fv() % 3] : 0; WO[q].robj = (fv() & 3) ? &RO[fv() & 1] : 0; } for (q = 0; q < 3; q++) AO[q].jobj = &JO[q & 1]; for (q = 0; q < 2; q++) JO[q].u.spline = (fv() & 3) ? &SPL[0] : 0; hsdWObj.parent.parent.head.parent = &CI[0]; CI[0].release = (void*) cbev1; CI[0].amnesia = (void*) cbev1; default_class = (fv() & 1) ? &CI[1] : 0; }
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            ARMED = 1;
            if (MTU_SETJMP()) { ARMED = 0; printf("%d %d ABORT\n", op, k); continue; }
            switch (op) {
            case 0: {  HSD_WObj * a0 = pick_HSD_WObj(); HSD_WObjRemoveAnim(a0);  } break;
            case 1: {  HSD_WObj * a0 = pick_HSD_WObj(); f32 a1 = rw(); HSD_WObjReqAnim(a0, a1);  } break;
            case 2: {  HSD_WObj * a0 = pick_HSD_WObj(); HSD_WObjAnim * a1 = pick_HSD_WObjAnim(); HSD_WObjAddAnim(a0, a1);  } break;
            case 3: {  void * a0 = (void*) picko_HSD_WObj(); enum_t a1 = (int) (fv() % 9); HSD_ObjData * a2 = pick_HSD_ObjData(); WObjUpdateFunc(a0, a1, a2);  } break;
            case 4: {  HSD_WObj * a0 = pick_HSD_WObj(); HSD_WObjInterpretAnim(a0);  } break;
            case 5: {  HSD_WObj * a0 = pick_HSD_WObj(); HSD_WObjDesc * a1 = pick_HSD_WObjDesc(); ret = (u32) WObjLoad(a0, a1);  } break;
            case 6: {  HSD_WObj * a0 = pick_HSD_WObj(); HSD_WObjDesc * a1 = pick_HSD_WObjDesc(); HSD_WObjInit(a0, a1);  } break;
            case 7: {  HSD_ClassInfo * a0 = pick_HSD_ClassInfo(); HSD_WObjSetDefaultClass(a0);  } break;
            case 8: {  HSD_WObjDesc * a0 = pick_HSD_WObjDesc(); ret = PI(HSD_WObjLoadDesc(a0));  } break;
            case 9: {  HSD_WObj * a0 = pick_HSD_WObj(); Vec3 * a1 = &VA[rnd() % 4]; HSD_WObjSetPosition(a0, a1);  } break;
            case 10: {  HSD_WObj * a0 = pick_HSD_WObj(); f32 a1 = rw(); HSD_WObjSetPositionX(a0, a1);  } break;
            case 11: {  HSD_WObj * a0 = pick_HSD_WObj(); f32 a1 = rw(); HSD_WObjSetPositionY(a0, a1);  } break;
            case 12: {  HSD_WObj * a0 = pick_HSD_WObj(); f32 a1 = rw(); HSD_WObjSetPositionZ(a0, a1);  } break;
            case 13: {  HSD_WObj * a0 = pick_HSD_WObj(); Vec3 * a1 = &VA[rnd() % 4]; HSD_WObjGetPosition(a0, a1);  } break;
            case 14: {   ret = PI(HSD_WObjAlloc());  } break;
            case 15: {  HSD_Class * a0 = (HSD_Class*) pick_HSD_WObj(); WObjRelease(a0);  } break;
            case 16: {  HSD_ClassInfo * a0 = pick_HSD_ClassInfo(); WObjAmnesia(a0);  } break;
            case 17: {   WObjInfoInit();  } break;
            }
            ARMED = 0;
            hash_world();

            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
