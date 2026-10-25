extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/sysdolphin/baselib/cobj.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static HSD_CObj COB[3];
static HSD_WObj WOB[4];
static HSD_AObj AOB[2];
static HSD_CObjDesc CDS[2];
static HSD_CObjInfo CINF[2];
static HSD_ClassInfo CIN[2];
static HSD_CameraAnim CAN[2];
static HSD_WObjDesc WDS[2];
static HSD_WObjAnim WAN[2];
static HSD_AObjDesc ADS[2];
static GXRenderModeObj RMO[1];
static HSD_ObjData OBD[2];
static Scissor SCS[1];
static HSD_RectS16 RS16[1];
static Vec3 VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static void* PPA[4];
static char STRS[4][64] = { "a.dat", "sym", "LbAd.dat", "x" };

static HSD_CObj* pick_HSD_CObj(void) { u32 r = rnd(); return &COB[(r >> 8) % 3]; }
static HSD_CObj* picko_HSD_CObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &COB[(r >> 8) % 3]; }
static HSD_WObj* pick_HSD_WObj(void) { u32 r = rnd(); return &WOB[(r >> 8) % 4]; }
static HSD_WObj* picko_HSD_WObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &WOB[(r >> 8) % 4]; }
static HSD_AObj* pick_HSD_AObj(void) { u32 r = rnd(); return &AOB[(r >> 8) % 2]; }
static HSD_AObj* picko_HSD_AObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &AOB[(r >> 8) % 2]; }
static HSD_CObjDesc* pick_HSD_CObjDesc(void) { u32 r = rnd(); return &CDS[(r >> 8) % 2]; }
static HSD_CObjDesc* picko_HSD_CObjDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &CDS[(r >> 8) % 2]; }
static HSD_CObjInfo* pick_HSD_CObjInfo(void) { u32 r = rnd(); return &CINF[(r >> 8) % 2]; }
static HSD_CObjInfo* picko_HSD_CObjInfo(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &CINF[(r >> 8) % 2]; }
static HSD_ClassInfo* pick_HSD_ClassInfo(void) { u32 r = rnd(); return &CIN[(r >> 8) % 2]; }
static HSD_ClassInfo* picko_HSD_ClassInfo(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &CIN[(r >> 8) % 2]; }
static HSD_CameraAnim* pick_HSD_CameraAnim(void) { u32 r = rnd(); return &CAN[(r >> 8) % 2]; }
static HSD_CameraAnim* picko_HSD_CameraAnim(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &CAN[(r >> 8) % 2]; }
static HSD_WObjDesc* pick_HSD_WObjDesc(void) { u32 r = rnd(); return &WDS[(r >> 8) % 2]; }
static HSD_WObjDesc* picko_HSD_WObjDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &WDS[(r >> 8) % 2]; }
static HSD_WObjAnim* pick_HSD_WObjAnim(void) { u32 r = rnd(); return &WAN[(r >> 8) % 2]; }
static HSD_WObjAnim* picko_HSD_WObjAnim(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &WAN[(r >> 8) % 2]; }
static HSD_AObjDesc* pick_HSD_AObjDesc(void) { u32 r = rnd(); return &ADS[(r >> 8) % 2]; }
static HSD_AObjDesc* picko_HSD_AObjDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &ADS[(r >> 8) % 2]; }
static GXRenderModeObj* pick_GXRenderModeObj(void) { u32 r = rnd(); return &RMO[(r >> 8) % 1]; }
static GXRenderModeObj* picko_GXRenderModeObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &RMO[(r >> 8) % 1]; }
static HSD_ObjData* pick_HSD_ObjData(void) { u32 r = rnd(); return &OBD[(r >> 8) % 2]; }
static HSD_ObjData* picko_HSD_ObjData(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &OBD[(r >> 8) % 2]; }
static Scissor* pick_Scissor(void) { u32 r = rnd(); return &SCS[(r >> 8) % 1]; }
static Scissor* picko_Scissor(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &SCS[(r >> 8) % 1]; }
static HSD_RectS16* pick_HSD_RectS16(void) { u32 r = rnd(); return &RS16[(r >> 8) % 1]; }
static HSD_RectS16* picko_HSD_RectS16(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &RS16[(r >> 8) % 1]; }
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
static void cb_amn(HSD_ClassInfo* c) { T(8205); T(PI(c)); }
static int cb_init(HSD_Class* c) { T(8206); T(PI(c)); return (int) (fv() % 3) - 1; }
static unsigned char MT[2][64];

#include "cobj_gen.h"



static void preg_world(void)
{
    preg(COB, sizeof COB);
    preg(WOB, sizeof WOB);
    preg(AOB, sizeof AOB);
    preg(CDS, sizeof CDS);
    preg(CINF, sizeof CINF);
    preg(CIN, sizeof CIN);
    preg(CAN, sizeof CAN);
    preg(WDS, sizeof WDS);
    preg(WAN, sizeof WAN);
    preg(ADS, sizeof ADS);
    preg(RMO, sizeof RMO);
    preg(OBD, sizeof OBD);
    preg(SCS, sizeof SCS);
    preg(RS16, sizeof RS16);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS); preg(PPA, sizeof PPA);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(COB, sizeof COB);
    zero(WOB, sizeof WOB);
    zero(AOB, sizeof AOB);
    zero(CDS, sizeof CDS);
    zero(CINF, sizeof CINF);
    zero(CIN, sizeof CIN);
    zero(CAN, sizeof CAN);
    zero(WDS, sizeof WDS);
    zero(WAN, sizeof WAN);
    zero(ADS, sizeof ADS);
    zero(RMO, sizeof RMO);
    zero(OBD, sizeof OBD);
    zero(SCS, sizeof SCS);
    zero(RS16, sizeof RS16);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    for (i = 0; i < 4; i++) PPA[i] = 0;
    for (i = 0; i < 3; i++) fill_HSD_CObj(&COB[i]);
    for (i = 0; i < 4; i++) fill_HSD_WObj(&WOB[i]);
    for (i = 0; i < 2; i++) fill_HSD_AObj(&AOB[i]);
    for (i = 0; i < 2; i++) fill_HSD_CObjDesc(&CDS[i]);
    for (i = 0; i < 2; i++) fill_HSD_CObjInfo(&CINF[i]);
    for (i = 0; i < 2; i++) fill_HSD_ClassInfo(&CIN[i]);
    for (i = 0; i < 2; i++) fill_HSD_CameraAnim(&CAN[i]);
    for (i = 0; i < 2; i++) fill_HSD_WObjDesc(&WDS[i]);
    for (i = 0; i < 2; i++) fill_HSD_WObjAnim(&WAN[i]);
    for (i = 0; i < 2; i++) fill_HSD_AObjDesc(&ADS[i]);
    for (i = 0; i < 1; i++) fill_GXRenderModeObj(&RMO[i]);
    for (i = 0; i < 2; i++) fill_HSD_ObjData(&OBD[i]);
    for (i = 0; i < 1; i++) fill_Scissor(&SCS[i]);
    for (i = 0; i < 1; i++) fill_HSD_RectS16(&RS16[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 3; i++) hash_HSD_CObj(&COB[i]);
    for (i = 0; i < 4; i++) hash_HSD_WObj(&WOB[i]);
    for (i = 0; i < 2; i++) hash_HSD_AObj(&AOB[i]);
    for (i = 0; i < 2; i++) hash_HSD_CObjDesc(&CDS[i]);
    for (i = 0; i < 2; i++) hash_HSD_CObjInfo(&CINF[i]);
    for (i = 0; i < 2; i++) hash_HSD_ClassInfo(&CIN[i]);
    for (i = 0; i < 2; i++) hash_HSD_CameraAnim(&CAN[i]);
    for (i = 0; i < 2; i++) hash_HSD_WObjDesc(&WDS[i]);
    for (i = 0; i < 2; i++) hash_HSD_WObjAnim(&WAN[i]);
    for (i = 0; i < 2; i++) hash_HSD_AObjDesc(&ADS[i]);
    for (i = 0; i < 1; i++) hash_GXRenderModeObj(&RMO[i]);
    for (i = 0; i < 2; i++) hash_HSD_ObjData(&OBD[i]);
    for (i = 0; i < 1; i++) hash_Scissor(&SCS[i]);
    for (i = 0; i < 1; i++) hash_HSD_RectS16(&RS16[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    for (i = 0; i < 4; i++) HM(PI(PPA[i]));
    hashg_all();
    hasho_all();
}

enum { OP_HSD_CObjEraseScreen = 0, OP_HSD_CObjRemoveAnimByFlags = 1, OP_HSD_CObjRemoveAnim = 2, OP_HSD_CObjAddAnim = 3, OP_CObjUpdateFunc = 4, OP_HSD_CObjAnim = 5, OP_HSD_CObjReqAnim = 6, OP_makeProjectionMtx = 7, OP_HSD_CObjSetupViewingMtx = 8, OP_HSD_CObjUpdateView = 9, OP_HSD_CObjSetCurrent = 10, OP_HSD_CObjEndCurrent = 11, OP_HSD_CObjGetInterestWObj = 12, OP_HSD_CObjGetEyePositionWObj = 13, OP_HSD_CObjSetMtxDirty = 14, OP_HSD_CObjMtxIsDirty = 15, OP_HSD_CObjSetFov = 16, OP_HSD_CObjGetNear = 17, OP_HSD_CObjSetNear = 18, OP_HSD_CObjGetFar = 19, OP_HSD_CObjSetFar = 20, OP_HSD_CObjSetScissor = 21, OP_HSD_CObjSetViewport = 22, OP_HSD_CObjGetProjectionType = 23, OP_HSD_CObjSetPerspective = 24, OP_HSD_CObjSetFrustum = 25, OP_HSD_CObjSetOrtho = 26, OP_HSD_CObjSetFlags = 27, OP_HSD_CObjClearFlags = 28, OP_HSD_CObjGetCurrent = 29, OP_HSD_CObjSetDefaultClass = 30, OP_HSD_CObjAlloc = 31, OP_CObjLoad = 32, OP_HSD_CObjInit = 33, OP_HSD_CObjLoadDesc = 34, OP_CObjInit = 35, OP_CObjRelease = 36, OP_CObjAmnesia = 37, OP_CObjInfoInit = 38 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();
preg(MT, sizeof MT); freg((const void*) cb_rel); freg((const void*) cb_dest); freg((const void*) cb_amn); freg((const void*) cb_init);
    for (op = 0; op < 39; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; break;  /* HSD_CObjEraseScreen */
        case 1: n = 300; break;  /* HSD_CObjRemoveAnimByFlags */
        case 2: n = 300; break;  /* HSD_CObjRemoveAnim */
        case 3: n = 300; break;  /* HSD_CObjAddAnim */
        case 4: n = 300; break;  /* CObjUpdateFunc */
        case 5: n = 300; break;  /* HSD_CObjAnim */
        case 6: n = 300; break;  /* HSD_CObjReqAnim */
        case 7: n = 300; break;  /* makeProjectionMtx */
        case 8: n = 300; break;  /* HSD_CObjSetupViewingMtx */
        case 9: n = 300; break;  /* HSD_CObjUpdateView */
        case 10: n = 300; break;  /* HSD_CObjSetCurrent */
        case 11: n = 300; break;  /* HSD_CObjEndCurrent */
        case 12: n = 300; break;  /* HSD_CObjGetInterestWObj */
        case 13: n = 300; break;  /* HSD_CObjGetEyePositionWObj */
        case 14: n = 300; break;  /* HSD_CObjSetMtxDirty */
        case 15: n = 300; break;  /* HSD_CObjMtxIsDirty */
        case 16: n = 300; break;  /* HSD_CObjSetFov */
        case 17: n = 300; break;  /* HSD_CObjGetNear */
        case 18: n = 300; break;  /* HSD_CObjSetNear */
        case 19: n = 300; break;  /* HSD_CObjGetFar */
        case 20: n = 300; break;  /* HSD_CObjSetFar */
        case 21: n = 300; break;  /* HSD_CObjSetScissor */
        case 22: n = 300; break;  /* HSD_CObjSetViewport */
        case 23: n = 300; break;  /* HSD_CObjGetProjectionType */
        case 24: n = 300; break;  /* HSD_CObjSetPerspective */
        case 25: n = 300; break;  /* HSD_CObjSetFrustum */
        case 26: n = 300; break;  /* HSD_CObjSetOrtho */
        case 27: n = 300; break;  /* HSD_CObjSetFlags */
        case 28: n = 300; break;  /* HSD_CObjClearFlags */
        case 29: n = 300; break;  /* HSD_CObjGetCurrent */
        case 30: n = 300; break;  /* HSD_CObjSetDefaultClass */
        case 31: n = 300; break;  /* HSD_CObjAlloc */
        case 32: n = 300; break;  /* CObjLoad */
        case 33: n = 300; break;  /* HSD_CObjInit */
        case 34: n = 300; break;  /* HSD_CObjLoadDesc */
        case 35: n = 300; break;  /* CObjInit */
        case 36: n = 300; break;  /* CObjRelease */
        case 37: n = 300; break;  /* CObjAmnesia */
        case 38: n = 300; break;  /* CObjInfoInit */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ int q; for (q = 0; q < 3; q++) { COB[q].parent.parent.class_info = (HSD_ClassInfo*) &CINF[q & 1]; COB[q].eyepos = &WOB[fv() % 4]; COB[q].interest = &WOB[fv() % 4]; COB[q].aobj = (fv() & 3) ? &AOB[fv() & 1] : 0; COB[q].proj_mtx = (fv() & 3) ? (Mtx*) MT[q & 1] : 0; COB[q].projection_type = (u8) (1 + fv() % 3); } for (q = 0; q < 2; q++) { CINF[q].load = CObjLoad; CINF[q].parent.parent.init = cb_init; CINF[q].parent.parent.release = (void*) cb_rel; CINF[q].parent.parent.destroy = (void*) cb_dest; } for (q = 0; q < 2; q++) { CDS[q].class_name = (fv() & 1) ? (char*) STRS[0] : 0; CDS[q].common.eyepos = &WDS[0]; CDS[q].common.interest = &WDS[1]; CDS[q].common.up_vector = (fv() & 1) ? (Vec3*) &VA[fv() % 4] : 0; } hsdCObj.parent.parent.head.parent = &CIN[0]; CIN[0].init = cb_init; CIN[0].release = (void*) cb_rel; CIN[0].destroy = (void*) cb_dest; CIN[0].amnesia = (void*) cb_amn; for (q = 0; q < 4; q++) WOB[q].parent.parent.class_info = &CIN[0]; default_class = (fv() & 1) ? &CIN[1] : 0; current = (fv() & 1) ? &COB[fv() % 3] : 0; }
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            ARMED = 1;
            if (MTU_SETJMP()) { ARMED = 0; printf("%d %d ABORT\n", op, k); continue; }
            switch (op) {
            case 0: {  HSD_CObj * a0 = pick_HSD_CObj(); s32 a1 = fv(); s32 a2 = fv(); s32 a3 = fv(); HSD_CObjEraseScreen(a0, a1, a2, a3);  } break;
            case 1: {  HSD_CObj * a0 = pick_HSD_CObj(); u32 a1 = fv(); HSD_CObjRemoveAnimByFlags(a0, a1);  } break;
            case 2: {  HSD_CObj * a0 = pick_HSD_CObj(); HSD_CObjRemoveAnim(a0);  } break;
            case 3: {  HSD_CObj * a0 = pick_HSD_CObj(); HSD_CameraAnim * a1 = pick_HSD_CameraAnim(); HSD_CObjAddAnim(a0, a1);  } break;
            case 4: {  void * a0 = (void*) BUF; int a1 = fv(); HSD_ObjData * a2 = pick_HSD_ObjData(); CObjUpdateFunc(a0, a1, a2);  } break;
            case 5: {  HSD_CObj * a0 = pick_HSD_CObj(); HSD_CObjAnim(a0);  } break;
            case 6: {  HSD_CObj * a0 = pick_HSD_CObj(); float a1 = rw(); HSD_CObjReqAnim(a0, a1);  } break;
            case 7: {  HSD_CObj * a0 = pick_HSD_CObj(); void* a1 = (void*) BUF; ret = (u32) makeProjectionMtx(a0, a1);  } break;
            case 8: {  HSD_CObj * a0 = pick_HSD_CObj(); HSD_CObjSetupViewingMtx(a0);  } break;
            case 9: {  HSD_CObj * a0 = pick_HSD_CObj(); HSD_CObjUpdateView(a0);  } break;
            case 10: {  HSD_CObj * a0 = pick_HSD_CObj(); ret = (u32) HSD_CObjSetCurrent(a0);  } break;
            case 11: {   HSD_CObjEndCurrent();  } break;
            case 12: {  HSD_CObj * a0 = pick_HSD_CObj(); ret = PI(HSD_CObjGetInterestWObj(a0));  } break;
            case 13: {  HSD_CObj * a0 = pick_HSD_CObj(); ret = PI(HSD_CObjGetEyePositionWObj(a0));  } break;
            case 14: {  HSD_CObj * a0 = pick_HSD_CObj(); HSD_CObjSetMtxDirty(a0);  } break;
            case 15: {  HSD_CObj * a0 = pick_HSD_CObj(); ret = (u32) HSD_CObjMtxIsDirty(a0);  } break;
            case 16: {  HSD_CObj * a0 = pick_HSD_CObj(); float a1 = rw(); HSD_CObjSetFov(a0, a1);  } break;
            case 17: {  HSD_CObj * a0 = pick_HSD_CObj(); ret = B(HSD_CObjGetNear(a0));  } break;
            case 18: {  HSD_CObj * a0 = pick_HSD_CObj(); float a1 = rw(); HSD_CObjSetNear(a0, a1);  } break;
            case 19: {  HSD_CObj * a0 = pick_HSD_CObj(); ret = B(HSD_CObjGetFar(a0));  } break;
            case 20: {  HSD_CObj * a0 = pick_HSD_CObj(); float a1 = rw(); HSD_CObjSetFar(a0, a1);  } break;
            case 21: {  HSD_CObj * a0 = pick_HSD_CObj(); Scissor * a1 = pick_Scissor(); HSD_CObjSetScissor(a0, a1);  } break;
            case 22: {  HSD_CObj * a0 = pick_HSD_CObj(); HSD_RectS16 * a1 = pick_HSD_RectS16(); HSD_CObjSetViewport(a0, a1);  } break;
            case 23: {  HSD_CObj * a0 = pick_HSD_CObj(); ret = (u32) HSD_CObjGetProjectionType(a0);  } break;
            case 24: {  HSD_CObj * a0 = pick_HSD_CObj(); float a1 = rw(); float a2 = rw(); HSD_CObjSetPerspective(a0, a1, a2);  } break;
            case 25: {  HSD_CObj * a0 = pick_HSD_CObj(); float a1 = rw(); float a2 = rw(); float a3 = rw(); float a4 = rw(); HSD_CObjSetFrustum(a0, a1, a2, a3, a4);  } break;
            case 26: {  HSD_CObj * a0 = pick_HSD_CObj(); float a1 = rw(); float a2 = rw(); float a3 = rw(); float a4 = rw(); HSD_CObjSetOrtho(a0, a1, a2, a3, a4);  } break;
            case 27: {  HSD_CObj * a0 = pick_HSD_CObj(); u32 a1 = fv(); HSD_CObjSetFlags(a0, a1);  } break;
            case 28: {  HSD_CObj * a0 = pick_HSD_CObj(); u32 a1 = fv(); HSD_CObjClearFlags(a0, a1);  } break;
            case 29: {   ret = PI(HSD_CObjGetCurrent());  } break;
            case 30: {  HSD_ClassInfo * a0 = pick_HSD_ClassInfo(); HSD_CObjSetDefaultClass(a0);  } break;
            case 31: {   ret = PI(HSD_CObjAlloc());  } break;
            case 32: {  HSD_CObj * a0 = pick_HSD_CObj(); HSD_CObjDesc * a1 = pick_HSD_CObjDesc(); ret = (u32) CObjLoad(a0, a1);  } break;
            case 33: {  HSD_CObj * a0 = pick_HSD_CObj(); HSD_CObjDesc * a1 = pick_HSD_CObjDesc(); HSD_CObjInit(a0, a1);  } break;
            case 34: {  HSD_CObjDesc * a0 = pick_HSD_CObjDesc(); ret = PI(HSD_CObjLoadDesc(a0));  } break;
            case 35: {  HSD_Class * a0 = (HSD_Class*) pick_HSD_CObj(); ret = (u32) CObjInit(a0);  } break;
            case 36: {  HSD_Class * a0 = (HSD_Class*) pick_HSD_CObj(); CObjRelease(a0);  } break;
            case 37: {  HSD_ClassInfo * a0 = pick_HSD_ClassInfo(); CObjAmnesia(a0);  } break;
            case 38: {   CObjInfoInit();  } break;
            }
            ARMED = 0;
            hash_world();

            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
