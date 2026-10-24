extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/sysdolphin/baselib/aobj.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static HSD_AObj AO[4];
static HSD_FObj FO[2];
static HSD_AObjDesc AD[2];
static HSD_JObj JO[3];
static HSD_DObj DO[3];
static HSD_MObj MO[2];
static HSD_TObj TO[3];
static HSD_PObj PO[2];
static HSD_RObj RO[2];
static HSD_WObj WO[2];
static HSD_CObj CO[2];
static HSD_LObj LO[3];
static HSD_Fog FG[2];
static HSD_ShapeSet SS[2];
static HSD_SList SL[3];
static HSD_FObjDesc FD[2];
static Vec3 VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static char STRS[4][64] = { "a.dat", "sym", "LbAd.dat", "x" };

static HSD_AObj* pick_HSD_AObj(void) { u32 r = rnd(); return &AO[(r >> 8) % 4]; }
static HSD_AObj* picko_HSD_AObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &AO[(r >> 8) % 4]; }
static HSD_FObj* pick_HSD_FObj(void) { u32 r = rnd(); return &FO[(r >> 8) % 2]; }
static HSD_FObj* picko_HSD_FObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &FO[(r >> 8) % 2]; }
static HSD_AObjDesc* pick_HSD_AObjDesc(void) { u32 r = rnd(); return &AD[(r >> 8) % 2]; }
static HSD_AObjDesc* picko_HSD_AObjDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &AD[(r >> 8) % 2]; }
static HSD_JObj* pick_HSD_JObj(void) { u32 r = rnd(); return &JO[(r >> 8) % 3]; }
static HSD_JObj* picko_HSD_JObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &JO[(r >> 8) % 3]; }
static HSD_DObj* pick_HSD_DObj(void) { u32 r = rnd(); return &DO[(r >> 8) % 3]; }
static HSD_DObj* picko_HSD_DObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &DO[(r >> 8) % 3]; }
static HSD_MObj* pick_HSD_MObj(void) { u32 r = rnd(); return &MO[(r >> 8) % 2]; }
static HSD_MObj* picko_HSD_MObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &MO[(r >> 8) % 2]; }
static HSD_TObj* pick_HSD_TObj(void) { u32 r = rnd(); return &TO[(r >> 8) % 3]; }
static HSD_TObj* picko_HSD_TObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &TO[(r >> 8) % 3]; }
static HSD_PObj* pick_HSD_PObj(void) { u32 r = rnd(); return &PO[(r >> 8) % 2]; }
static HSD_PObj* picko_HSD_PObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &PO[(r >> 8) % 2]; }
static HSD_RObj* pick_HSD_RObj(void) { u32 r = rnd(); return &RO[(r >> 8) % 2]; }
static HSD_RObj* picko_HSD_RObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &RO[(r >> 8) % 2]; }
static HSD_WObj* pick_HSD_WObj(void) { u32 r = rnd(); return &WO[(r >> 8) % 2]; }
static HSD_WObj* picko_HSD_WObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &WO[(r >> 8) % 2]; }
static HSD_CObj* pick_HSD_CObj(void) { u32 r = rnd(); return &CO[(r >> 8) % 2]; }
static HSD_CObj* picko_HSD_CObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &CO[(r >> 8) % 2]; }
static HSD_LObj* pick_HSD_LObj(void) { u32 r = rnd(); return &LO[(r >> 8) % 3]; }
static HSD_LObj* picko_HSD_LObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &LO[(r >> 8) % 3]; }
static HSD_Fog* pick_HSD_Fog(void) { u32 r = rnd(); return &FG[(r >> 8) % 2]; }
static HSD_Fog* picko_HSD_Fog(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &FG[(r >> 8) % 2]; }
static HSD_ShapeSet* pick_HSD_ShapeSet(void) { u32 r = rnd(); return &SS[(r >> 8) % 2]; }
static HSD_ShapeSet* picko_HSD_ShapeSet(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &SS[(r >> 8) % 2]; }
static HSD_SList* pick_HSD_SList(void) { u32 r = rnd(); return &SL[(r >> 8) % 3]; }
static HSD_SList* picko_HSD_SList(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &SL[(r >> 8) % 3]; }
static HSD_FObjDesc* pick_HSD_FObjDesc(void) { u32 r = rnd(); return &FD[(r >> 8) % 2]; }
static HSD_FObjDesc* picko_HSD_FObjDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &FD[(r >> 8) % 2]; }
struct HSD_GObj; typedef struct HSD_GObj HSD_GObj;
static void cbev1(HSD_GObj* g) { T(7001); T(PI(g)); }
static void cbev2(HSD_GObj* g) { T(7002); T(PI(g)); }
static int cbp1(HSD_GObj* g) { T(7003); T(PI(g)); return (int) (fv() & 1); }
static int cbp2(HSD_GObj* g) { T(7004); T(PI(g)); return (int) (fv() & 1); }
static void cbev3(HSD_GObj* g, HSD_GObj* h) { T(7005); T(PI(g)); T(PI(h)); }
static void cbi1(int a) { T(7006); T((u32) a); }
static void cbi2(int a) { T(7007); T((u32) a); }
static void autocb_0(void * a0, int a1, union HSD_ObjData * a2) { T(9000); T(PI((const void*) a0)); T((u32) a1); T(PI((const void*) a2)); }
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
static void freg_world(void) { freg((const void*) cbev1); freg((const void*) cbev2); freg((const void*) cbp1); freg((const void*) cbp2); freg((const void*) cbev3); freg((const void*) cbi1); freg((const void*) cbi2); freg((const void*) autocb_0); }

static int AOI;
static void cbv0(void) { T(8100); }
static void cbv1(void) { T(8101); }
static void cb_A(HSD_AObj* a) { T(8000); T(PI(a)); }
static void cb_AF(HSD_AObj* a, float f) { T(8001); T(PI(a)); T(B(f)); }
static void cb_AV(HSD_AObj* a, void* v) { T(8002); T(PI(a)); T(PI(v)); }
static void cb_AU(HSD_AObj* a, u32 d) { T(8003); T(PI(a)); T(d); }
static void cb_AO(HSD_AObj* a, void* o) { T(8004); T(PI(a)); T(PI(o)); }
static void cb_AOF(HSD_AObj* a, void* o, float f) { T(8005); T(PI(a)); T(PI(o)); T(B(f)); }
static void cb_AOV(HSD_AObj* a, void* o, void* v) { T(8006); T(PI(a)); T(PI(o)); T(PI(v)); }
static void cb_AOU(HSD_AObj* a, void* o, u32 d) { T(8007); T(PI(a)); T(PI(o)); T(d); }
static void cb_AOT(HSD_AObj* a, void* o, int t) { T(8008); T(PI(a)); T(PI(o)); T(t); }
static void cb_AOTF(HSD_AObj* a, void* o, int t, float f) { T(8009); T(PI(a)); T(PI(o)); T(t); T(B(f)); }
static void cb_AOTV(HSD_AObj* a, void* o, int t, void* v) { T(8010); T(PI(a)); T(PI(o)); T(t); T(PI(v)); }
static void cb_AOTU(HSD_AObj* a, void* o, int t, u32 d) { T(8011); T(PI(a)); T(PI(o)); T(t); T(d); }
static void* FN[12] = { cb_A, cb_AF, cb_AV, cb_AU, cb_AO, cb_AOF, cb_AOV, cb_AOU, cb_AOT, cb_AOTF, cb_AOTV, cb_AOTU };
static const int TY[10] = { 6, 3, 8, 9, 11, 7, 2, 10, 12, 5 };
static void* objfor(int type)
{
    if ((fv() & 7) == 0) return 0;
    switch (type) {
    case 6: return &JO[0]; case 3: return &DO[0]; case 8: return &MO[0]; case 9: return &PO[0]; case 11: return &TO[0];
    case 7: return &LO[0]; case 2: return &CO[0]; case 10: return &RO[0]; case 12: return &WO[0]; default: return &FG[0];
    }
}

#include "aobj_gen.h"



static void preg_world(void)
{
    preg(AO, sizeof AO);
    preg(FO, sizeof FO);
    preg(AD, sizeof AD);
    preg(JO, sizeof JO);
    preg(DO, sizeof DO);
    preg(MO, sizeof MO);
    preg(TO, sizeof TO);
    preg(PO, sizeof PO);
    preg(RO, sizeof RO);
    preg(WO, sizeof WO);
    preg(CO, sizeof CO);
    preg(LO, sizeof LO);
    preg(FG, sizeof FG);
    preg(SS, sizeof SS);
    preg(SL, sizeof SL);
    preg(FD, sizeof FD);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(AO, sizeof AO);
    zero(FO, sizeof FO);
    zero(AD, sizeof AD);
    zero(JO, sizeof JO);
    zero(DO, sizeof DO);
    zero(MO, sizeof MO);
    zero(TO, sizeof TO);
    zero(PO, sizeof PO);
    zero(RO, sizeof RO);
    zero(WO, sizeof WO);
    zero(CO, sizeof CO);
    zero(LO, sizeof LO);
    zero(FG, sizeof FG);
    zero(SS, sizeof SS);
    zero(SL, sizeof SL);
    zero(FD, sizeof FD);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    for (i = 0; i < 4; i++) fill_HSD_AObj(&AO[i]);
    for (i = 0; i < 2; i++) fill_HSD_FObj(&FO[i]);
    for (i = 0; i < 2; i++) fill_HSD_AObjDesc(&AD[i]);
    for (i = 0; i < 3; i++) fill_HSD_JObj(&JO[i]);
    for (i = 0; i < 3; i++) fill_HSD_DObj(&DO[i]);
    for (i = 0; i < 2; i++) fill_HSD_MObj(&MO[i]);
    for (i = 0; i < 3; i++) fill_HSD_TObj(&TO[i]);
    for (i = 0; i < 2; i++) fill_HSD_PObj(&PO[i]);
    for (i = 0; i < 2; i++) fill_HSD_RObj(&RO[i]);
    for (i = 0; i < 2; i++) fill_HSD_WObj(&WO[i]);
    for (i = 0; i < 2; i++) fill_HSD_CObj(&CO[i]);
    for (i = 0; i < 3; i++) fill_HSD_LObj(&LO[i]);
    for (i = 0; i < 2; i++) fill_HSD_Fog(&FG[i]);
    for (i = 0; i < 2; i++) fill_HSD_ShapeSet(&SS[i]);
    for (i = 0; i < 3; i++) fill_HSD_SList(&SL[i]);
    for (i = 0; i < 2; i++) fill_HSD_FObjDesc(&FD[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 4; i++) hash_HSD_AObj(&AO[i]);
    for (i = 0; i < 2; i++) hash_HSD_FObj(&FO[i]);
    for (i = 0; i < 2; i++) hash_HSD_AObjDesc(&AD[i]);
    for (i = 0; i < 3; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 3; i++) hash_HSD_DObj(&DO[i]);
    for (i = 0; i < 2; i++) hash_HSD_MObj(&MO[i]);
    for (i = 0; i < 3; i++) hash_HSD_TObj(&TO[i]);
    for (i = 0; i < 2; i++) hash_HSD_PObj(&PO[i]);
    for (i = 0; i < 2; i++) hash_HSD_RObj(&RO[i]);
    for (i = 0; i < 2; i++) hash_HSD_WObj(&WO[i]);
    for (i = 0; i < 2; i++) hash_HSD_CObj(&CO[i]);
    for (i = 0; i < 3; i++) hash_HSD_LObj(&LO[i]);
    for (i = 0; i < 2; i++) hash_HSD_Fog(&FG[i]);
    for (i = 0; i < 2; i++) hash_HSD_ShapeSet(&SS[i]);
    for (i = 0; i < 3; i++) hash_HSD_SList(&SL[i]);
    for (i = 0; i < 2; i++) hash_HSD_FObjDesc(&FD[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    hashg_all();
    hasho_all();
}

enum { OP_HSD_AObjInitAllocData = 0, OP_HSD_AObjGetAllocData = 1, OP_HSD_AObjGetFlags = 2, OP_HSD_AObjSetFlags = 3, OP_HSD_AObjClearFlags = 4, OP_HSD_AObjSetFObj = 5, OP_HSD_AObjInitEndCallBack = 6, OP_HSD_AObjInvokeCallBacks = 7, OP_HSD_AObjReqAnim = 8, OP_HSD_AObjStopAnim = 9, OP_HSD_AObjInterpretAnim = 10, OP_HSD_AObjLoadDesc = 11, OP_HSD_AObjRemove = 12, OP_HSD_AObjAlloc = 13, OP_HSD_AObjFree = 14, OP_HSD_ForeachAnim = 15, OP_HSD_AObjSetRate = 16, OP_HSD_AObjSetRewindFrame = 17, OP_HSD_AObjSetEndFrame = 18, OP_HSD_AObjSetCurrentFrame = 19, OP__HSD_AObjForgetMemory = 20 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();
freg((const void*) cbv0); freg((const void*) cbv1); { int i; for (i = 0; i < 12; i++) freg(FN[i]); } preg(&AOI, 4);
    for (op = 0; op < 21; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; break;  /* HSD_AObjInitAllocData */
        case 1: n = 300; break;  /* HSD_AObjGetAllocData */
        case 2: n = 300; break;  /* HSD_AObjGetFlags */
        case 3: n = 300; break;  /* HSD_AObjSetFlags */
        case 4: n = 300; break;  /* HSD_AObjClearFlags */
        case 5: n = 300; break;  /* HSD_AObjSetFObj */
        case 6: n = 300; break;  /* HSD_AObjInitEndCallBack */
        case 7: n = 300; break;  /* HSD_AObjInvokeCallBacks */
        case 8: n = 300; break;  /* HSD_AObjReqAnim */
        case 9: n = 300; break;  /* HSD_AObjStopAnim */
        case 10: n = 300; break;  /* HSD_AObjInterpretAnim */
        case 11: n = 300; break;  /* HSD_AObjLoadDesc */
        case 12: n = 300; break;  /* HSD_AObjRemove */
        case 13: n = 300; break;  /* HSD_AObjAlloc */
        case 14: n = 300; break;  /* HSD_AObjFree */
        case 15: n = 300; break;  /* HSD_ForeachAnim */
        case 16: n = 300; break;  /* HSD_AObjSetRate */
        case 17: n = 300; break;  /* HSD_AObjSetRewindFrame */
        case 18: n = 300; break;  /* HSD_AObjSetEndFrame */
        case 19: n = 300; break;  /* HSD_AObjSetCurrentFrame */
        case 20: n = 300; break;  /* _HSD_AObjForgetMemory */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ int q; AOI = 0; for (q = 0; q < 3; q++) { JO[q].aobj = (fv() & 3) ? &AO[fv() % 4] : 0; } for (q = 0; q < 3; q++) { DO[q].aobj = (fv() & 3) ? &AO[fv() % 4] : 0; } for (q = 0; q < 2; q++) { MO[q].aobj = (fv() & 3) ? &AO[fv() % 4] : 0; } for (q = 0; q < 3; q++) { TO[q].aobj = (fv() & 3) ? &AO[fv() % 4] : 0; } for (q = 0; q < 2; q++) { RO[q].aobj = (fv() & 3) ? &AO[fv() % 4] : 0; } for (q = 0; q < 2; q++) { WO[q].aobj = (fv() & 3) ? &AO[fv() % 4] : 0; } for (q = 0; q < 2; q++) { CO[q].aobj = (fv() & 3) ? &AO[fv() % 4] : 0; } for (q = 0; q < 3; q++) { LO[q].aobj = (fv() & 3) ? &AO[fv() % 4] : 0; } for (q = 0; q < 2; q++) { FG[q].aobj = (fv() & 3) ? &AO[fv() % 4] : 0; } JO[0].child = &JO[1]; JO[0].next = 0; JO[1].child = 0; JO[1].next = &JO[2]; JO[2].child = 0; JO[2].next = 0; for (q = 0; q < 3; q++) { JO[q].u.dobj = (fv() & 3) ? &DO[fv() % 3] : 0; JO[q].robj = (fv() & 3) ? &RO[fv() & 1] : 0; } for (q = 0; q < 3; q++) { DO[q].next = q < 2 ? &DO[q + 1] : 0; DO[q].mobj = (fv() & 3) ? &MO[fv() & 1] : 0; DO[q].pobj = (fv() & 3) ? &PO[0] : 0; } for (q = 0; q < 2; q++) MO[q].tobj = (fv() & 3) ? &TO[0] : 0; for (q = 0; q < 3; q++) TO[q].next = q < 2 ? &TO[q + 1] : 0; PO[0].next = &PO[1]; PO[1].next = 0; for (q = 0; q < 2; q++) PO[q].u.shape_set = (fv() & 3) ? &SS[fv() & 1] : 0; RO[0].next = &RO[1]; RO[1].next = 0; for (q = 0; q < 2; q++) WO[q].robj = (fv() & 3) ? &RO[0] : 0; for (q = 0; q < 3; q++) LO[q].next = q < 2 ? &LO[q + 1] : 0; SL[0].next = (fv() & 1) ? &SL[1] : 0; SL[1].next = 0; SL[2].next = 0; SL[0].data = (void*) cbv0; SL[1].data = (void*) cbv1; endcallback_list = (fv() & 7) ? &SL[0] : 0; HSD_AObj_804D762C = fv() & 1; HSD_AObj_804D7630 = (fv() % 3) == 0; }
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            ARMED = 1;
            if (MTU_SETJMP()) { ARMED = 0; printf("%d %d ABORT\n", op, k); continue; }
            switch (op) {
            case 0: {   HSD_AObjInitAllocData();  } break;
            case 1: {   ret = PI(HSD_AObjGetAllocData());  } break;
            case 2: {  HSD_AObj * a0 = pick_HSD_AObj(); ret = (u32) HSD_AObjGetFlags(a0);  } break;
            case 3: {  HSD_AObj * a0 = pick_HSD_AObj(); u32 a1 = fv(); HSD_AObjSetFlags(a0, a1);  } break;
            case 4: {  HSD_AObj * a0 = pick_HSD_AObj(); u32 a1 = fv(); HSD_AObjClearFlags(a0, a1);  } break;
            case 5: {  HSD_AObj * a0 = pick_HSD_AObj(); HSD_FObj * a1 = pick_HSD_FObj(); HSD_AObjSetFObj(a0, a1);  } break;
            case 6: {   HSD_AObjInitEndCallBack();  } break;
            case 7: {   HSD_AObjInvokeCallBacks();  } break;
            case 8: {  HSD_AObj * a0 = pick_HSD_AObj(); f32 a1 = rw(); HSD_AObjReqAnim(a0, a1);  } break;
            case 9: {  HSD_AObj * a0 = pick_HSD_AObj(); void * a1 = (void*) BUF; HSD_ObjUpdateFunc a2 = (HSD_ObjUpdateFunc) autocb_0; HSD_AObjStopAnim(a0, a1, a2);  } break;
            case 10: {  HSD_AObj * a0 = pick_HSD_AObj(); void * a1 = (void*) BUF; HSD_ObjUpdateFunc a2 = (HSD_ObjUpdateFunc) autocb_0; HSD_AObjInterpretAnim(a0, a1, a2);  } break;
            case 11: {  HSD_AObjDesc * a0 = pick_HSD_AObjDesc(); ret = PI(HSD_AObjLoadDesc(a0));  } break;
            case 12: {  HSD_AObj * a0 = pick_HSD_AObj(); HSD_AObjRemove(a0);  } break;
            case 13: {   ret = PI(HSD_AObjAlloc());  } break;
            case 14: {  HSD_AObj * a0 = pick_HSD_AObj(); HSD_AObjFree(a0);  } break;
            case 15: {  void * a0 = 0; HSD_Type a1 = 0; HSD_TypeMask a2 = 0; void * a3 = 0; AObj_Arg_Type a4 = 0; { int ty = TY[fv() % 10]; int at = (int) (fv() % 14); void* ob = objfor(ty); u32 mk = (fv() & 1) ? 0x7FFFu : (fv() & 0x7FFFu); void* fn = FN[at > 11 ? 0 : at]; switch (at) { case 1: case 5: case 9: HSD_ForeachAnim(ob, ty, mk, fn, at, (double) rw()); break; case 2: case 6: case 10: HSD_ForeachAnim(ob, ty, mk, fn, at, (void*) BUF); break; case 3: case 7: case 11: HSD_ForeachAnim(ob, ty, mk, fn, at, (u32) fv()); break; default: HSD_ForeachAnim(ob, ty, mk, fn, at); } }  } break;
            case 16: {  HSD_AObj * a0 = pick_HSD_AObj(); f32 a1 = rw(); HSD_AObjSetRate(a0, a1);  } break;
            case 17: {  HSD_AObj * a0 = pick_HSD_AObj(); f32 a1 = rw(); HSD_AObjSetRewindFrame(a0, a1);  } break;
            case 18: {  HSD_AObj * a0 = pick_HSD_AObj(); f32 a1 = rw(); HSD_AObjSetEndFrame(a0, a1);  } break;
            case 19: {  HSD_AObj * a0 = pick_HSD_AObj(); f32 a1 = rw(); HSD_AObjSetCurrentFrame(a0, a1);  } break;
            case 20: {  void * a0 = (void*) BUF; void * a1 = (void*) BUF; _HSD_AObjForgetMemory(a0, a1);  } break;
            }
            ARMED = 0;
            hash_world();

            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
