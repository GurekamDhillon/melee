extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/sysdolphin/baselib/jobj.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static HSD_JObj J[4];
static HSD_Joint JT[3];
static HSD_JObjInfo JI[3];
static HSD_ClassInfo CI[3];
static HSD_DObj DO[2];
static HSD_DObjDesc DD[2];
static HSD_RObj RO[2];
static HSD_RObjDesc RD[2];
static HSD_AObj AO[2];
static HSD_SList SL[3];
static HSD_Spline SPL[1];
static HSD_Class CL[2];
static Vec3 VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static void* PPA[4];
static char STRS[4][64] = { "a.dat", "sym", "LbAd.dat", "x" };

static HSD_JObj* pick_HSD_JObj(void) { u32 r = rnd(); return &J[(r >> 8) % 4]; }
static HSD_JObj* picko_HSD_JObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &J[(r >> 8) % 4]; }
static HSD_Joint* pick_HSD_Joint(void) { u32 r = rnd(); return &JT[(r >> 8) % 3]; }
static HSD_Joint* picko_HSD_Joint(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &JT[(r >> 8) % 3]; }
static HSD_JObjInfo* pick_HSD_JObjInfo(void) { u32 r = rnd(); return &JI[(r >> 8) % 3]; }
static HSD_JObjInfo* picko_HSD_JObjInfo(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &JI[(r >> 8) % 3]; }
static HSD_ClassInfo* pick_HSD_ClassInfo(void) { u32 r = rnd(); return &CI[(r >> 8) % 3]; }
static HSD_ClassInfo* picko_HSD_ClassInfo(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &CI[(r >> 8) % 3]; }
static HSD_DObj* pick_HSD_DObj(void) { u32 r = rnd(); return &DO[(r >> 8) % 2]; }
static HSD_DObj* picko_HSD_DObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &DO[(r >> 8) % 2]; }
static HSD_DObjDesc* pick_HSD_DObjDesc(void) { u32 r = rnd(); return &DD[(r >> 8) % 2]; }
static HSD_DObjDesc* picko_HSD_DObjDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &DD[(r >> 8) % 2]; }
static HSD_RObj* pick_HSD_RObj(void) { u32 r = rnd(); return &RO[(r >> 8) % 2]; }
static HSD_RObj* picko_HSD_RObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &RO[(r >> 8) % 2]; }
static HSD_RObjDesc* pick_HSD_RObjDesc(void) { u32 r = rnd(); return &RD[(r >> 8) % 2]; }
static HSD_RObjDesc* picko_HSD_RObjDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &RD[(r >> 8) % 2]; }
static HSD_AObj* pick_HSD_AObj(void) { u32 r = rnd(); return &AO[(r >> 8) % 2]; }
static HSD_AObj* picko_HSD_AObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &AO[(r >> 8) % 2]; }
static HSD_SList* pick_HSD_SList(void) { u32 r = rnd(); return &SL[(r >> 8) % 3]; }
static HSD_SList* picko_HSD_SList(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &SL[(r >> 8) % 3]; }
static HSD_Spline* pick_HSD_Spline(void) { u32 r = rnd(); return &SPL[(r >> 8) % 1]; }
static HSD_Spline* picko_HSD_Spline(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &SPL[(r >> 8) % 1]; }
static HSD_Class* pick_HSD_Class(void) { u32 r = rnd(); return &CL[(r >> 8) % 2]; }
static HSD_Class* picko_HSD_Class(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &CL[(r >> 8) % 2]; }
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
static int cb_load(HSD_JObj* j, HSD_Joint* d, HSD_JObj* p) { T(8207); T(PI(j)); T(PI(d)); T(PI(p)); return (int) (fv() % 3) - 1; }
static void cb_relchild(HSD_JObj* j) { T(8208); T(PI(j)); }
static unsigned char MT[4][48];

#include "jobj_gen.h"



static void preg_world(void)
{
    preg(J, sizeof J);
    preg(JT, sizeof JT);
    preg(JI, sizeof JI);
    preg(CI, sizeof CI);
    preg(DO, sizeof DO);
    preg(DD, sizeof DD);
    preg(RO, sizeof RO);
    preg(RD, sizeof RD);
    preg(AO, sizeof AO);
    preg(SL, sizeof SL);
    preg(SPL, sizeof SPL);
    preg(CL, sizeof CL);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS); preg(PPA, sizeof PPA);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(J, sizeof J);
    zero(JT, sizeof JT);
    zero(JI, sizeof JI);
    zero(CI, sizeof CI);
    zero(DO, sizeof DO);
    zero(DD, sizeof DD);
    zero(RO, sizeof RO);
    zero(RD, sizeof RD);
    zero(AO, sizeof AO);
    zero(SL, sizeof SL);
    zero(SPL, sizeof SPL);
    zero(CL, sizeof CL);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    for (i = 0; i < 4; i++) PPA[i] = 0;
    for (i = 0; i < 4; i++) fill_HSD_JObj(&J[i]);
    for (i = 0; i < 3; i++) fill_HSD_Joint(&JT[i]);
    for (i = 0; i < 3; i++) fill_HSD_JObjInfo(&JI[i]);
    for (i = 0; i < 3; i++) fill_HSD_ClassInfo(&CI[i]);
    for (i = 0; i < 2; i++) fill_HSD_DObj(&DO[i]);
    for (i = 0; i < 2; i++) fill_HSD_DObjDesc(&DD[i]);
    for (i = 0; i < 2; i++) fill_HSD_RObj(&RO[i]);
    for (i = 0; i < 2; i++) fill_HSD_RObjDesc(&RD[i]);
    for (i = 0; i < 2; i++) fill_HSD_AObj(&AO[i]);
    for (i = 0; i < 3; i++) fill_HSD_SList(&SL[i]);
    for (i = 0; i < 1; i++) fill_HSD_Spline(&SPL[i]);
    for (i = 0; i < 2; i++) fill_HSD_Class(&CL[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 4; i++) hash_HSD_JObj(&J[i]);
    for (i = 0; i < 3; i++) hash_HSD_Joint(&JT[i]);
    for (i = 0; i < 3; i++) hash_HSD_JObjInfo(&JI[i]);
    for (i = 0; i < 3; i++) hash_HSD_ClassInfo(&CI[i]);
    for (i = 0; i < 2; i++) hash_HSD_DObj(&DO[i]);
    for (i = 0; i < 2; i++) hash_HSD_DObjDesc(&DD[i]);
    for (i = 0; i < 2; i++) hash_HSD_RObj(&RO[i]);
    for (i = 0; i < 2; i++) hash_HSD_RObjDesc(&RD[i]);
    for (i = 0; i < 2; i++) hash_HSD_AObj(&AO[i]);
    for (i = 0; i < 3; i++) hash_HSD_SList(&SL[i]);
    for (i = 0; i < 1; i++) hash_HSD_Spline(&SPL[i]);
    for (i = 0; i < 2; i++) hash_HSD_Class(&CL[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    for (i = 0; i < 4; i++) HM(PI(PPA[i]));
    hashg_all();
    hasho_all();
}

enum { OP_HSD_JObjSetDefaultClass = 0, OP_JObjLoadJointSub = 1, OP_JObjLoad = 2, OP_HSD_JObjLoadJoint = 3, OP_HSD_JObjResolveRefs = 4, OP_HSD_JObjResolveRefsAll = 5, OP_HSD_JObjUnref = 6, OP_HSD_JObjUnrefThis = 7, OP_HSD_JObjAlloc = 8, OP_HSD_JObjSetCurrent = 9, OP_HSD_JObjGetCurrent = 10, OP_JObjInit = 11, OP_JObjRelease = 12, OP_JObjAmnesia = 13 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();
preg(MT, sizeof MT); freg((const void*) cb_rel); freg((const void*) cb_dest); freg((const void*) cb_amn); freg((const void*) cb_init); freg((const void*) cb_load); freg((const void*) cb_relchild);
    for (op = 0; op < 14; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; break;  /* HSD_JObjSetDefaultClass */
        case 1: n = 300; break;  /* JObjLoadJointSub */
        case 2: n = 300; break;  /* JObjLoad */
        case 3: n = 300; break;  /* HSD_JObjLoadJoint */
        case 4: n = 300; break;  /* HSD_JObjResolveRefs */
        case 5: n = 300; break;  /* HSD_JObjResolveRefsAll */
        case 6: n = 300; break;  /* HSD_JObjUnref */
        case 7: n = 300; break;  /* HSD_JObjUnrefThis */
        case 8: n = 300; break;  /* HSD_JObjAlloc */
        case 9: n = 300; break;  /* HSD_JObjSetCurrent */
        case 10: n = 300; break;  /* HSD_JObjGetCurrent */
        case 11: n = 300; break;  /* JObjInit */
        case 12: n = 300; break;  /* JObjRelease */
        case 13: n = 300; break;  /* JObjAmnesia */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ int q;  for (q = 0; q < 3; q++) { HSD_ClassInfo* ci = (HSD_ClassInfo*) &JI[q]; ci->release = (void*) cb_rel; ci->destroy = (void*) cb_dest; ci->amnesia = cb_amn; ci->init = cb_init; JI[q].load = cb_load; JI[q].release_child = cb_relchild; }  for (q = 0; q < 3; q++) { CI[q].release = (void*) cb_rel; CI[q].destroy = (void*) cb_dest; CI[q].amnesia = cb_amn; CI[q].init = cb_init; }  for (q = 0; q < 4; q++) { J[q].object.parent.class_info = (HSD_ClassInfo*) &JI[q % 3]; J[q].next = (q < 3 && (fv() & 1)) ? &J[q + 1] : 0; J[q].child = (q < 3 && (fv() & 1)) ? &J[q + 1 + (fv() % (3 - q))] : 0;    J[q].parent = 0; J[q].flags = fv() & 0xFFFFF; J[q].robj = (fv() & 1) ? &RO[fv() & 1] : 0; J[q].aobj = 0; J[q].scl = (fv() & 1) ? (Vec*) &VA[fv() % 4] : 0; J[q].envelopemtx = (fv() & 1) ? (MtxPtr) MT[q] : 0;    J[q].object.ref_count = (fv() & 3) ? (u16) (fv() % 3) : (u16) 65535; J[q].object.ref_count_individual = (u16) (fv() % 3);    if (J[q].flags & 0x4020) { J[q].u.ptcl = (fv() & 1) ? &SL[fv() % 3] : 0; } else { J[q].u.dobj = (fv() & 1) ? &DO[fv() & 1] : 0; } }  for (q = 0; q < 3; q++) SL[q].next = (q < 2 && (fv() & 1)) ? &SL[q + 1] : 0;  for (q = 0; q < 3; q++) { JT[q].class_name = (fv() & 3) ? 0 : STRS[fv() % 4]; JT[q].flags = fv() & 0xFFFFF; JT[q].child = (q < 2 && (fv() & 1)) ? &JT[q + 1 + (fv() % (2 - q))] : 0; JT[q].next = (q < 2 && (fv() & 1)) ? &JT[q + 1] : 0;    JT[q].mtx = (fv() & 1) ? (MtxPtr) MT[q] : 0; JT[q].robjdesc = (fv() & 1) ? &RD[fv() & 1] : 0;    if (JT[q].flags & 0x4020) { if (JT[q].flags & 0x4000) JT[q].u.spline = &SPL[0]; else JT[q].u.ptcl = (fv() & 1) ? &SL[fv() % 3] : 0; } else JT[q].u.dobjdesc = (fv() & 1) ? &DD[fv() & 1] : 0; }  for (q = 0; q < 4; q++) { int i; for (i = 0; i < 48; i++) MT[q][i] = (unsigned char) rnd(); }  hsdJObj.parent.parent.head.parent = &CI[0]; hsdJObj.parent.parent.init = cb_init; hsdJObj.parent.parent.release = (void*) cb_rel; hsdJObj.parent.parent.amnesia = cb_amn;  default_class = (fv() & 1) ? &CI[fv() % 3] : 0; ufc_callbacks = (fv() & 1) ? &SL[fv() % 3] : 0; current_jobj = (fv() & 1) ? &J[fv() % 4] : 0; }
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            ARMED = 1;
            if (MTU_SETJMP()) { ARMED = 0; printf("%d %d ABORT\n", op, k); continue; }
            switch (op) {
            case 0: {  HSD_ClassInfo * a0 = pick_HSD_ClassInfo(); HSD_JObjSetDefaultClass(a0);  } break;
            case 1: {  HSD_Joint * a0 = pick_HSD_Joint(); HSD_JObj * a1 = pick_HSD_JObj(); ret = PI(JObjLoadJointSub(a0, a1));  } break;
            case 2: {  HSD_JObj * a0 = pick_HSD_JObj(); HSD_Joint * a1 = pick_HSD_Joint(); HSD_JObj * a2 = pick_HSD_JObj(); ret = (u32) JObjLoad(a0, a1, a2);  } break;
            case 3: {  HSD_Joint * a0 = pick_HSD_Joint(); ret = PI(HSD_JObjLoadJoint(a0));  } break;
            case 4: {  HSD_JObj * a0 = pick_HSD_JObj(); HSD_Joint * a1 = pick_HSD_Joint(); HSD_JObjResolveRefs(a0, a1);  } break;
            case 5: {  HSD_JObj * a0 = pick_HSD_JObj(); HSD_Joint * a1 = pick_HSD_Joint(); HSD_JObjResolveRefsAll(a0, a1);  } break;
            case 6: {  HSD_JObj * a0 = pick_HSD_JObj(); HSD_JObjUnref(a0);  } break;
            case 7: {  HSD_JObj * a0 = pick_HSD_JObj(); HSD_JObjUnrefThis(a0);  } break;
            case 8: {   ret = PI(HSD_JObjAlloc());  } break;
            case 9: {  HSD_JObj * a0 = pick_HSD_JObj(); HSD_JObjSetCurrent(a0);  } break;
            case 10: {   ret = PI(HSD_JObjGetCurrent());  } break;
            case 11: {  HSD_Class * a0 = (HSD_Class*) pick_HSD_JObj(); ret = (u32) JObjInit(a0);  } break;
            case 12: {  HSD_Class * a0 = (HSD_Class*) pick_HSD_JObj(); JObjRelease(a0);  } break;
            case 13: {  HSD_ClassInfo * a0 = pick_HSD_ClassInfo(); JObjAmnesia(a0);  } break;
            }
            ARMED = 0;
            hash_world();

            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
