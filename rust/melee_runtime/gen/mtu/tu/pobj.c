extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/sysdolphin/baselib/pobj.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static HSD_PObj P0[3];
static HSD_ShapeAnim P1[3];
static HSD_ShapeSet P2[3];
static HSD_ObjData P3[3];
static HSD_Envelope P4[3];
static HSD_SList P5[4];
static HSD_ShapeSetDesc P6[2];
static HSD_PObjDesc P7[3];
static HSD_PObjInfo P8[3];
static HSD_VtxDescList P9[9];
static HSD_JObj P10[3];
static HSD_ClassInfo P12[3];
static HSD_EnvelopeDesc P13[3];
static HSD_AObj P14[2];
static HSD_AObjDesc P15[2];
static HSD_Joint P16[2];
static HSD_Class P11[2];
static Vec3 VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static void* PPA[4];
static char STRS[4][64] = { "a.dat", "sym", "LbAd.dat", "x" };

static HSD_PObj* pick_HSD_PObj(void) { u32 r = rnd(); return &P0[(r >> 8) % 3]; }
static HSD_PObj* picko_HSD_PObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &P0[(r >> 8) % 3]; }
static HSD_ShapeAnim* pick_HSD_ShapeAnim(void) { u32 r = rnd(); return &P1[(r >> 8) % 3]; }
static HSD_ShapeAnim* picko_HSD_ShapeAnim(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &P1[(r >> 8) % 3]; }
static HSD_ShapeSet* pick_HSD_ShapeSet(void) { u32 r = rnd(); return &P2[(r >> 8) % 3]; }
static HSD_ShapeSet* picko_HSD_ShapeSet(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &P2[(r >> 8) % 3]; }
static HSD_ObjData* pick_HSD_ObjData(void) { u32 r = rnd(); return &P3[(r >> 8) % 3]; }
static HSD_ObjData* picko_HSD_ObjData(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &P3[(r >> 8) % 3]; }
static HSD_Envelope* pick_HSD_Envelope(void) { u32 r = rnd(); return &P4[(r >> 8) % 3]; }
static HSD_Envelope* picko_HSD_Envelope(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &P4[(r >> 8) % 3]; }
static HSD_SList* pick_HSD_SList(void) { u32 r = rnd(); return &P5[(r >> 8) % 4]; }
static HSD_SList* picko_HSD_SList(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &P5[(r >> 8) % 4]; }
static HSD_ShapeSetDesc* pick_HSD_ShapeSetDesc(void) { u32 r = rnd(); return &P6[(r >> 8) % 2]; }
static HSD_ShapeSetDesc* picko_HSD_ShapeSetDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &P6[(r >> 8) % 2]; }
static HSD_PObjDesc* pick_HSD_PObjDesc(void) { u32 r = rnd(); return &P7[(r >> 8) % 3]; }
static HSD_PObjDesc* picko_HSD_PObjDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &P7[(r >> 8) % 3]; }
static HSD_PObjInfo* pick_HSD_PObjInfo(void) { u32 r = rnd(); return &P8[(r >> 8) % 3]; }
static HSD_PObjInfo* picko_HSD_PObjInfo(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &P8[(r >> 8) % 3]; }
static HSD_VtxDescList* pick_HSD_VtxDescList(void) { u32 r = rnd(); return &P9[(r >> 8) % 9]; }
static HSD_VtxDescList* picko_HSD_VtxDescList(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &P9[(r >> 8) % 9]; }
static HSD_JObj* pick_HSD_JObj(void) { u32 r = rnd(); return &P10[(r >> 8) % 3]; }
static HSD_JObj* picko_HSD_JObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &P10[(r >> 8) % 3]; }
static HSD_ClassInfo* pick_HSD_ClassInfo(void) { u32 r = rnd(); return &P12[(r >> 8) % 3]; }
static HSD_ClassInfo* picko_HSD_ClassInfo(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &P12[(r >> 8) % 3]; }
static HSD_EnvelopeDesc* pick_HSD_EnvelopeDesc(void) { u32 r = rnd(); return &P13[(r >> 8) % 3]; }
static HSD_EnvelopeDesc* picko_HSD_EnvelopeDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &P13[(r >> 8) % 3]; }
static HSD_AObj* pick_HSD_AObj(void) { u32 r = rnd(); return &P14[(r >> 8) % 2]; }
static HSD_AObj* picko_HSD_AObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &P14[(r >> 8) % 2]; }
static HSD_AObjDesc* pick_HSD_AObjDesc(void) { u32 r = rnd(); return &P15[(r >> 8) % 2]; }
static HSD_AObjDesc* picko_HSD_AObjDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &P15[(r >> 8) % 2]; }
static HSD_Joint* pick_HSD_Joint(void) { u32 r = rnd(); return &P16[(r >> 8) % 2]; }
static HSD_Joint* picko_HSD_Joint(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &P16[(r >> 8) % 2]; }
static HSD_Class* pick_HSD_Class(void) { u32 r = rnd(); return &P11[(r >> 8) % 2]; }
static HSD_Class* picko_HSD_Class(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &P11[(r >> 8) % 2]; }
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

static HSD_EnvelopeDesc* EPL[3][4]; static unsigned RK;
static void cb_rel(HSD_Class* c) { T(8203); T(PI(c)); }
static void cb_dest(HSD_Class* c) { T(8204); T(PI(c)); }
static void cb_amn(HSD_ClassInfo* c) { T(8205); T(PI(c)); }
static void cb_disp(HSD_PObj* p, Mtx v, Mtx m, u32 r) { T(8206); T(PI(p)); T(PI(v)); T(PI(m)); T(r); }
static void cb_setup(HSD_PObj* p, Mtx v, Mtx m, u32 r) { T(8207); T(PI(p)); T(PI(v)); T(PI(m)); T(r); }
static int cb_load(HSD_PObj* p, HSD_PObjDesc* d) { T(8208); T(PI(p)); T(PI(d)); return (int) (fv() % 3) - 1; }
static unsigned char MT[4][48];

#include "pobj_gen.h"



static void preg_world(void)
{
    preg(P0, sizeof P0);
    preg(P1, sizeof P1);
    preg(P2, sizeof P2);
    preg(P3, sizeof P3);
    preg(P4, sizeof P4);
    preg(P5, sizeof P5);
    preg(P6, sizeof P6);
    preg(P7, sizeof P7);
    preg(P8, sizeof P8);
    preg(P9, sizeof P9);
    preg(P10, sizeof P10);
    preg(P12, sizeof P12);
    preg(P13, sizeof P13);
    preg(P14, sizeof P14);
    preg(P15, sizeof P15);
    preg(P16, sizeof P16);
    preg(P11, sizeof P11);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS); preg(PPA, sizeof PPA);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(P0, sizeof P0);
    zero(P1, sizeof P1);
    zero(P2, sizeof P2);
    zero(P3, sizeof P3);
    zero(P4, sizeof P4);
    zero(P5, sizeof P5);
    zero(P6, sizeof P6);
    zero(P7, sizeof P7);
    zero(P8, sizeof P8);
    zero(P9, sizeof P9);
    zero(P10, sizeof P10);
    zero(P12, sizeof P12);
    zero(P13, sizeof P13);
    zero(P14, sizeof P14);
    zero(P15, sizeof P15);
    zero(P16, sizeof P16);
    zero(P11, sizeof P11);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    for (i = 0; i < 4; i++) PPA[i] = 0;
    for (i = 0; i < 3; i++) fill_HSD_PObj(&P0[i]);
    for (i = 0; i < 3; i++) fill_HSD_ShapeAnim(&P1[i]);
    for (i = 0; i < 3; i++) fill_HSD_ShapeSet(&P2[i]);
    for (i = 0; i < 3; i++) fill_HSD_ObjData(&P3[i]);
    for (i = 0; i < 3; i++) fill_HSD_Envelope(&P4[i]);
    for (i = 0; i < 4; i++) fill_HSD_SList(&P5[i]);
    for (i = 0; i < 2; i++) fill_HSD_ShapeSetDesc(&P6[i]);
    for (i = 0; i < 3; i++) fill_HSD_PObjDesc(&P7[i]);
    for (i = 0; i < 3; i++) fill_HSD_PObjInfo(&P8[i]);
    for (i = 0; i < 9; i++) fill_HSD_VtxDescList(&P9[i]);
    for (i = 0; i < 3; i++) fill_HSD_JObj(&P10[i]);
    for (i = 0; i < 3; i++) fill_HSD_ClassInfo(&P12[i]);
    for (i = 0; i < 3; i++) fill_HSD_EnvelopeDesc(&P13[i]);
    for (i = 0; i < 2; i++) fill_HSD_AObj(&P14[i]);
    for (i = 0; i < 2; i++) fill_HSD_AObjDesc(&P15[i]);
    for (i = 0; i < 2; i++) fill_HSD_Joint(&P16[i]);
    for (i = 0; i < 2; i++) fill_HSD_Class(&P11[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 3; i++) hash_HSD_PObj(&P0[i]);
    for (i = 0; i < 3; i++) hash_HSD_ShapeAnim(&P1[i]);
    for (i = 0; i < 3; i++) hash_HSD_ShapeSet(&P2[i]);
    for (i = 0; i < 3; i++) hash_HSD_ObjData(&P3[i]);
    for (i = 0; i < 3; i++) hash_HSD_Envelope(&P4[i]);
    for (i = 0; i < 4; i++) hash_HSD_SList(&P5[i]);
    for (i = 0; i < 2; i++) hash_HSD_ShapeSetDesc(&P6[i]);
    for (i = 0; i < 3; i++) hash_HSD_PObjDesc(&P7[i]);
    for (i = 0; i < 3; i++) hash_HSD_PObjInfo(&P8[i]);
    for (i = 0; i < 9; i++) hash_HSD_VtxDescList(&P9[i]);
    for (i = 0; i < 3; i++) hash_HSD_JObj(&P10[i]);
    for (i = 0; i < 3; i++) hash_HSD_ClassInfo(&P12[i]);
    for (i = 0; i < 3; i++) hash_HSD_EnvelopeDesc(&P13[i]);
    for (i = 0; i < 2; i++) hash_HSD_AObj(&P14[i]);
    for (i = 0; i < 2; i++) hash_HSD_AObjDesc(&P15[i]);
    for (i = 0; i < 2; i++) hash_HSD_Joint(&P16[i]);
    for (i = 0; i < 2; i++) hash_HSD_Class(&P11[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    for (i = 0; i < 4; i++) HM(PI(PPA[i]));
    hashg_all();
    hasho_all();
}

enum { OP_HSD_PObjGetFlags = 0, OP_HSD_PObjRemoveAnimByFlags = 1, OP_HSD_PObjRemoveAnimAllByFlags = 2, OP_HSD_PObjAddAnim = 3, OP_HSD_PObjAddAnimAll = 4, OP_HSD_PObjReqAnimByFlags = 5, OP_HSD_PObjReqAnimAllByFlags = 6, OP_ShapeSetSetAnimResult = 7, OP_PObjUpdateFunc = 8, OP_HSD_PObjAnim = 9, OP_HSD_PObjAnimAll = 10, OP_HSD_EnvelopeAlloc = 11, OP_HSD_EnvelopeFree = 12, OP_HSD_EnvelopeListFree = 13, OP_loadEnvelopeDesc = 14, OP_HSD_ShapeSetFree = 15, OP_HSD_ShapeSetRemove = 16, OP_loadShapeSetDesc = 17, OP_PObjLoad = 18, OP_HSD_PObjLoadDesc = 19, OP_HSD_PObjRemove = 20, OP_HSD_PObjRemoveAll = 21, OP_HSD_PObjGetDefaultClass = 22, OP_HSD_PObjSetDefaultClass = 23, OP_HSD_PObjAlloc = 24, OP_HSD_PObjFree = 25, OP_resolveEnvelope = 26, OP_HSD_PObjResolveRefs = 27, OP_HSD_PObjResolveRefsAll = 28, OP_HSD_ClearVtxDesc = 29, OP_setupArrayDesc = 30, OP_setupVtxDesc = 31, OP_HSD_PObjClearMtxMark = 32, OP_HSD_PObjSetMtxMark = 33, OP_HSD_PObjGetMtxMark = 34, OP_GetSetupFlags = 35, OP_SetupRigidModelMtx = 36, OP_SetupSharedVtxModelMtx = 37, OP_SetupEnvelopeModelMtx = 38, OP_PObjSetupMtx = 39, OP_PObjDispSimplePrimitive = 40, OP_HSD_PObjDisp = 41, OP_PObjRelease = 42, OP_PObjAmnesia = 43, OP_PObjInfoInit = 44, OP_GenoPalInfoInit = 45, OP_geno_pal_desc = 46 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();
preg(MT, sizeof MT); preg(EPL, sizeof EPL); freg((const void*) cb_rel); freg((const void*) cb_dest); freg((const void*) cb_amn); freg((const void*) cb_disp); freg((const void*) cb_setup); freg((const void*) cb_load);
    for (op = 0; op < 47; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; break;  /* HSD_PObjGetFlags */
        case 1: n = 300; break;  /* HSD_PObjRemoveAnimByFlags */
        case 2: n = 300; break;  /* HSD_PObjRemoveAnimAllByFlags */
        case 3: n = 300; break;  /* HSD_PObjAddAnim */
        case 4: n = 300; break;  /* HSD_PObjAddAnimAll */
        case 5: n = 300; break;  /* HSD_PObjReqAnimByFlags */
        case 6: n = 300; break;  /* HSD_PObjReqAnimAllByFlags */
        case 7: n = 300; break;  /* ShapeSetSetAnimResult */
        case 8: n = 300; break;  /* PObjUpdateFunc */
        case 9: n = 300; break;  /* HSD_PObjAnim */
        case 10: n = 300; break;  /* HSD_PObjAnimAll */
        case 11: n = 300; break;  /* HSD_EnvelopeAlloc */
        case 12: n = 300; break;  /* HSD_EnvelopeFree */
        case 13: n = 300; break;  /* HSD_EnvelopeListFree */
        case 14: n = 300; break;  /* loadEnvelopeDesc */
        case 15: n = 300; break;  /* HSD_ShapeSetFree */
        case 16: n = 300; break;  /* HSD_ShapeSetRemove */
        case 17: n = 300; break;  /* loadShapeSetDesc */
        case 18: n = 300; break;  /* PObjLoad */
        case 19: n = 300; break;  /* HSD_PObjLoadDesc */
        case 20: n = 300; break;  /* HSD_PObjRemove */
        case 21: n = 300; break;  /* HSD_PObjRemoveAll */
        case 22: n = 300; break;  /* HSD_PObjGetDefaultClass */
        case 23: n = 300; break;  /* HSD_PObjSetDefaultClass */
        case 24: n = 300; break;  /* HSD_PObjAlloc */
        case 25: n = 300; break;  /* HSD_PObjFree */
        case 26: n = 300; break;  /* resolveEnvelope */
        case 27: n = 300; break;  /* HSD_PObjResolveRefs */
        case 28: n = 300; break;  /* HSD_PObjResolveRefsAll */
        case 29: n = 300; break;  /* HSD_ClearVtxDesc */
        case 30: n = 300; break;  /* setupArrayDesc */
        case 31: n = 300; break;  /* setupVtxDesc */
        case 32: n = 300; break;  /* HSD_PObjClearMtxMark */
        case 33: n = 300; break;  /* HSD_PObjSetMtxMark */
        case 34: n = 300; break;  /* HSD_PObjGetMtxMark */
        case 35: n = 300; break;  /* GetSetupFlags */
        case 36: n = 300; break;  /* SetupRigidModelMtx */
        case 37: n = 300; break;  /* SetupSharedVtxModelMtx */
        case 38: n = 300; break;  /* SetupEnvelopeModelMtx */
        case 39: n = 300; break;  /* PObjSetupMtx */
        case 40: n = 300; break;  /* PObjDispSimplePrimitive */
        case 41: n = 300; break;  /* HSD_PObjDisp */
        case 42: n = 300; break;  /* PObjRelease */
        case 43: n = 300; break;  /* PObjAmnesia */
        case 44: n = 300; break;  /* PObjInfoInit */
        case 45: n = 300; break;  /* GenoPalInfoInit */
        case 46: n = 300; break;  /* geno_pal_desc */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ int q, l;  for (q = 0; q < 3; q++) { HSD_ClassInfo* ci = (HSD_ClassInfo*) &P8[q];    ci->release = (void*) cb_rel; ci->destroy = (void*) cb_dest; ci->amnesia = cb_amn; P8[q].disp = cb_disp; P8[q].setup_mtx = (q == 0) ? PObjSetupMtx : cb_setup; P8[q].load = cb_load; }  for (l = 0; l < 3; l++) { P9[3 * l].attr = fv() % 26; P9[3 * l + 1].attr = fv() % 26; P9[3 * l + 2].attr = 255; P9[3 * l].vertex = (fv() & 1) ? BUF : 0; P9[3 * l + 1].vertex = (fv() & 1) ? BUF + 64 : 0; P9[3 * l].attr_type = fv() % 4; P9[3 * l + 1].attr_type = fv() % 4; }  for (q = 0; q < 4; q++) { P5[q].next = (q < 3 && (fv() & 1)) ? &P5[q + 1] : 0; P5[q].data = &P4[fv() % 3]; }  for (q = 0; q < 3; q++) { P4[q].next = (q < 2 && (fv() & 1)) ? &P4[q + 1] : 0; P4[q].jobj = &P10[fv() % 3]; }  for (q = 0; q < 3; q++) { int k = 0; int n = fv() % 4; for (; k < n; k++) EPL[q][k] = &P13[fv() % 3]; EPL[q][k] = 0; }  for (q = 0; q < 3; q++) P13[q].joint = (fv() & 3) ? &P16[fv() & 1] : 0; P13[2].joint = 0;  for (q = 0; q < 3; q++) { P2[q].aobj = (fv() & 1) ? &P14[fv() & 1] : 0; P2[q].flags &= (u16) ~2; }  for (q = 0; q < 3; q++) { int kind = (q == 0) ? 0 : (q == 1) ? 0x2000 : (fv() % 3) * 0x1000;    P0[q].parent.class_info = (HSD_ClassInfo*) &P8[q]; P0[q].next = (q < 2 && (fv() & 1)) ? &P0[q + 1] : 0; P0[q].verts = &P9[3 * (fv() % 3)];    P0[q].flags = (u16) ((fv() & 0xCFFF) | kind); P0[q].n_display = (u16) (fv() % 4); P0[q].display = BUF;    if (kind == 0) P0[q].u.jobj = (q == 0 || (fv() & 1)) ? &P10[fv() % 3] : 0; else if (kind == 0x1000) P0[q].u.shape_set = &P2[fv() % 3]; else P0[q].u.envelope_list = (fv() & 3) ? &P5[fv() % 4] : 0; }  for (q = 0; q < 3; q++) { int kind = P0[q].flags & 0x3000; P7[q].n_display = (u16) (fv() % 4); P7[q].class_name = (fv() & 3) ? 0 : STRS[fv() % 4]; P7[q].next = P0[q].next ? &P7[q + 1] : 0; P7[q].verts = &P9[3 * (fv() % 3)];    P7[q].flags = (u16) ((fv() & 0xCFFF) | kind); P7[q].display = BUF; if (kind == 0) P7[q].u.joint = (fv() & 3) ? &P16[fv() & 1] : 0; else if (kind == 0x1000) P7[q].u.shape_set = &P6[fv() & 1]; else P7[q].u.envelope_p = (fv() & 3) ? EPL[fv() % 3] : 0; }  for (q = 0; q < 2; q++) { P6[q].vertex_desc = &P9[0]; P6[q].normal_desc = &P9[3]; P6[q].flags &= (u16) ~2; }  for (q = 0; q < 4; q++) { int i; for (i = 0; i < 48; i++) MT[q][i] = (unsigned char) rnd(); }  hsdPObj.parent.release = (void*) cb_rel; hsdPObj.parent.destroy = (void*) cb_dest; hsdPObj.parent.amnesia = cb_amn; hsdPObj.parent.head.parent = &P12[0]; P12[0].release = (void*) cb_rel; P12[0].amnesia = cb_amn; P12[0].destroy = (void*) cb_dest;  default_class = (fv() & 1) ? &P8[fv() % 3] : 0; mtx_mark[0].obj = (fv() & 1) ? (void*) &P10[fv() % 3] : 0; mtx_mark[1].obj = (fv() & 1) ? (void*) &P10[fv() % 3] : 0; mtx_mark[0].mark = fv() % 4; mtx_mark[1].mark = fv() % 4;  prev_vtxdesc = (fv() & 1) ? &P9[3 * (fv() % 3)] : 0; prev_vtxdesclist_array = (fv() & 1) ? &P9[3 * (fv() % 3)] : 0; }
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            ARMED = 1;
            if (MTU_SETJMP()) { ARMED = 0; printf("%d %d ABORT\n", op, k); continue; }
            switch (op) {
            case 0: {  HSD_PObj * a0 = pick_HSD_PObj(); ret = (u32) HSD_PObjGetFlags(a0);  } break;
            case 1: {  HSD_PObj * a0 = pick_HSD_PObj(); u32 a1 = fv(); HSD_PObjRemoveAnimByFlags(a0, a1);  } break;
            case 2: {  HSD_PObj * a0 = pick_HSD_PObj(); u32 a1 = fv(); HSD_PObjRemoveAnimAllByFlags(a0, a1);  } break;
            case 3: {  HSD_PObj * a0 = pick_HSD_PObj(); HSD_ShapeAnim * a1 = pick_HSD_ShapeAnim(); HSD_PObjAddAnim(a0, a1);  } break;
            case 4: {  HSD_PObj * a0 = pick_HSD_PObj(); HSD_ShapeAnim * a1 = pick_HSD_ShapeAnim(); HSD_PObjAddAnimAll(a0, a1);  } break;
            case 5: {  HSD_PObj * a0 = pick_HSD_PObj(); f32 a1 = rw(); u32 a2 = fv(); HSD_PObjReqAnimByFlags(a0, a1, a2);  } break;
            case 6: {  HSD_PObj * a0 = pick_HSD_PObj(); f32 a1 = rw(); u32 a2 = fv(); HSD_PObjReqAnimAllByFlags(a0, a1, a2);  } break;
            case 7: {  HSD_ShapeSet * a0 = pick_HSD_ShapeSet(); u32 a1 = fv(); HSD_ObjData * a2 = pick_HSD_ObjData(); ShapeSetSetAnimResult(a0, a1, a2);  } break;
            case 8: {  void * a0 = (void*) picko_HSD_PObj(); int a1 = fv(); HSD_ObjData * a2 = pick_HSD_ObjData(); PObjUpdateFunc(a0, a1, a2);  } break;
            case 9: {  HSD_PObj * a0 = pick_HSD_PObj(); HSD_PObjAnim(a0);  } break;
            case 10: {  HSD_PObj * a0 = pick_HSD_PObj(); HSD_PObjAnimAll(a0);  } break;
            case 11: {   ret = PI(HSD_EnvelopeAlloc());  } break;
            case 12: {  HSD_Envelope * a0 = pick_HSD_Envelope(); HSD_EnvelopeFree(a0);  } break;
            case 13: {  HSD_SList * a0 = pick_HSD_SList(); HSD_EnvelopeListFree(a0);  } break;
            case 14: {  HSD_EnvelopeDesc ** a0 = (void*) &PPA[fv() % 4]; ret = PI(loadEnvelopeDesc(a0));  } break;
            case 15: {  HSD_ShapeSet * a0 = pick_HSD_ShapeSet(); HSD_ShapeSetFree(a0);  } break;
            case 16: {  HSD_ShapeSet * a0 = pick_HSD_ShapeSet(); HSD_ShapeSetRemove(a0);  } break;
            case 17: {  HSD_ShapeSetDesc * a0 = pick_HSD_ShapeSetDesc(); ret = PI(loadShapeSetDesc(a0));  } break;
            case 18: {  HSD_PObj * a0 = pick_HSD_PObj(); HSD_PObjDesc * a1 = pick_HSD_PObjDesc(); ret = (u32) PObjLoad(a0, a1);  } break;
            case 19: {  HSD_PObjDesc * a0 = pick_HSD_PObjDesc(); ret = PI(HSD_PObjLoadDesc(a0));  } break;
            case 20: {  HSD_PObj * a0 = pick_HSD_PObj(); HSD_PObjRemove(a0);  } break;
            case 21: {  HSD_PObj * a0 = pick_HSD_PObj(); HSD_PObjRemoveAll(a0);  } break;
            case 22: {   ret = PI(HSD_PObjGetDefaultClass());  } break;
            case 23: {  HSD_PObjInfo * a0 = pick_HSD_PObjInfo(); HSD_PObjSetDefaultClass(a0);  } break;
            case 24: {   ret = PI(HSD_PObjAlloc());  } break;
            case 25: {  HSD_PObj * a0 = pick_HSD_PObj(); HSD_PObjFree(a0);  } break;
            case 26: {  HSD_SList * a0 = pick_HSD_SList(); HSD_EnvelopeDesc ** a1 = (void*) &PPA[fv() % 4]; resolveEnvelope(a0, a1);  } break;
            case 27: {  HSD_PObj * a0 = (RK = fv() % 3, &P0[RK]); HSD_PObjDesc * a1 = &P7[RK]; HSD_PObjResolveRefs(a0, a1);  } break;
            case 28: {  HSD_PObj * a0 = &P0[0]; HSD_PObjDesc * a1 = &P7[0]; HSD_PObjResolveRefsAll(a0, a1);  } break;
            case 29: {   HSD_ClearVtxDesc();  } break;
            case 30: {  HSD_VtxDescList * a0 = pick_HSD_VtxDescList(); setupArrayDesc(a0);  } break;
            case 31: {  HSD_PObj * a0 = pick_HSD_PObj(); setupVtxDesc(a0);  } break;
            case 32: {  void * a0 = (void*) BUF; u32 a1 = fv(); HSD_PObjClearMtxMark(a0, a1);  } break;
            case 33: {  int a0 = (int) (fv() % 4); void * a1 = (void*) BUF; u32 a2 = fv(); HSD_PObjSetMtxMark(a0, a1, a2);  } break;
            case 34: {  int a0 = fv(); void ** a1 = (void*) &PPA[fv() % 4]; u32 * a2 = UA; HSD_PObjGetMtxMark(a0, a1, a2);  } break;
            case 35: {  HSD_JObj * a0 = pick_HSD_JObj(); u32 a1 = fv(); ret = (u32) GetSetupFlags(a0, a1);  } break;
            case 36: {  HSD_PObj * a0 = pick_HSD_PObj(); void* a1 = (void*) BUF; void* a2 = (void*) BUF; u32 a3 = fv(); SetupRigidModelMtx(a0, a1, a2, a3);  } break;
            case 37: {  HSD_PObj * a0 = &P0[0]; void* a1 = (void*) BUF; void* a2 = (void*) BUF; u32 a3 = fv(); SetupSharedVtxModelMtx(a0, a1, a2, a3);  } break;
            case 38: {  HSD_PObj * a0 = &P0[1]; void* a1 = (void*) BUF; void* a2 = (void*) BUF; u32 a3 = fv(); SetupEnvelopeModelMtx(a0, a1, a2, a3);  } break;
            case 39: {  HSD_PObj * a0 = pick_HSD_PObj(); void* a1 = (void*) BUF; void* a2 = (void*) BUF; u32 a3 = fv(); PObjSetupMtx(a0, a1, a2, a3);  } break;
            case 40: {  HSD_PObj * a0 = pick_HSD_PObj(); u32 a1 = fv(); PObjDispSimplePrimitive(a0, a1);  } break;
            case 41: {  HSD_PObj * a0 = (fv() & 1) ? &P0[1] : &P0[0]; void* a1 = (Mtx*) MT[rnd() % 4]; void* a2 = (Mtx*) MT[rnd() % 4]; u32 a3 = fv(); HSD_PObjDisp(a0, a1, a2, a3);  } break;
            case 42: {  HSD_Class * a0 = (HSD_Class*) pick_HSD_PObj(); PObjRelease(a0);  } break;
            case 43: {  HSD_ClassInfo * a0 = pick_HSD_ClassInfo(); PObjAmnesia(a0);  } break;
            case 44: {   PObjInfoInit();  } break;
            case 45: {   GenoPalInfoInit();  } break;
            case 46: {  const HSD_PObjDesc * a0 = pick_HSD_PObjDesc(); ret = (u32) geno_pal_desc(a0);  } break;
            }
            ARMED = 0;
            hash_world();

            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
