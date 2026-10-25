extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/sysdolphin/baselib/dobj.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static HSD_DObj DOB[3];
static HSD_MObj MOB[2];
static HSD_PObj POB[2];
static HSD_AObj AOB[2];
static HSD_MatAnim MA[2];
static HSD_ShapeAnimDObj SAD[2];
static HSD_ShapeAnim SHA[2];
static HSD_DObjDesc DD[2];
static HSD_MObjDesc MD[2];
static HSD_PObjDesc PD[2];
static HSD_ClassInfo CIN[2];
static HSD_DObjInfo DINF[2];
static HSD_MObjInfo MINF[1];
static HSD_PObjInfo PINF[1];
static Vec3 VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static char STRS[4][64] = { "a.dat", "sym", "LbAd.dat", "x" };

static HSD_DObj* pick_HSD_DObj(void) { u32 r = rnd(); return &DOB[(r >> 8) % 3]; }
static HSD_DObj* picko_HSD_DObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &DOB[(r >> 8) % 3]; }
static HSD_MObj* pick_HSD_MObj(void) { u32 r = rnd(); return &MOB[(r >> 8) % 2]; }
static HSD_MObj* picko_HSD_MObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &MOB[(r >> 8) % 2]; }
static HSD_PObj* pick_HSD_PObj(void) { u32 r = rnd(); return &POB[(r >> 8) % 2]; }
static HSD_PObj* picko_HSD_PObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &POB[(r >> 8) % 2]; }
static HSD_AObj* pick_HSD_AObj(void) { u32 r = rnd(); return &AOB[(r >> 8) % 2]; }
static HSD_AObj* picko_HSD_AObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &AOB[(r >> 8) % 2]; }
static HSD_MatAnim* pick_HSD_MatAnim(void) { u32 r = rnd(); return &MA[(r >> 8) % 2]; }
static HSD_MatAnim* picko_HSD_MatAnim(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &MA[(r >> 8) % 2]; }
static HSD_ShapeAnimDObj* pick_HSD_ShapeAnimDObj(void) { u32 r = rnd(); return &SAD[(r >> 8) % 2]; }
static HSD_ShapeAnimDObj* picko_HSD_ShapeAnimDObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &SAD[(r >> 8) % 2]; }
static HSD_ShapeAnim* pick_HSD_ShapeAnim(void) { u32 r = rnd(); return &SHA[(r >> 8) % 2]; }
static HSD_ShapeAnim* picko_HSD_ShapeAnim(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &SHA[(r >> 8) % 2]; }
static HSD_DObjDesc* pick_HSD_DObjDesc(void) { u32 r = rnd(); return &DD[(r >> 8) % 2]; }
static HSD_DObjDesc* picko_HSD_DObjDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &DD[(r >> 8) % 2]; }
static HSD_MObjDesc* pick_HSD_MObjDesc(void) { u32 r = rnd(); return &MD[(r >> 8) % 2]; }
static HSD_MObjDesc* picko_HSD_MObjDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &MD[(r >> 8) % 2]; }
static HSD_PObjDesc* pick_HSD_PObjDesc(void) { u32 r = rnd(); return &PD[(r >> 8) % 2]; }
static HSD_PObjDesc* picko_HSD_PObjDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &PD[(r >> 8) % 2]; }
static HSD_ClassInfo* pick_HSD_ClassInfo(void) { u32 r = rnd(); return &CIN[(r >> 8) % 2]; }
static HSD_ClassInfo* picko_HSD_ClassInfo(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &CIN[(r >> 8) % 2]; }
static HSD_DObjInfo* pick_HSD_DObjInfo(void) { u32 r = rnd(); return &DINF[(r >> 8) % 2]; }
static HSD_DObjInfo* picko_HSD_DObjInfo(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &DINF[(r >> 8) % 2]; }
static HSD_MObjInfo* pick_HSD_MObjInfo(void) { u32 r = rnd(); return &MINF[(r >> 8) % 1]; }
static HSD_MObjInfo* picko_HSD_MObjInfo(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &MINF[(r >> 8) % 1]; }
static HSD_PObjInfo* pick_HSD_PObjInfo(void) { u32 r = rnd(); return &PINF[(r >> 8) % 1]; }
static HSD_PObjInfo* picko_HSD_PObjInfo(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &PINF[(r >> 8) % 1]; }
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

static void cb_setup(HSD_MObj* m, u32 r) { T(8200); T(PI(m)); T(r); }
static void cb_unset(HSD_MObj* m, u32 r) { T(8201); T(PI(m)); T(r); }
static void cb_disp(HSD_PObj* p, void* v, void* pm, u32 r) { T(8202); T(PI(p)); T(PI(v)); T(PI(pm)); T(r); }
static void cb_rel(HSD_Class* c) { T(8203); T(PI(c)); }
static void cb_dest(HSD_Class* c) { T(8204); T(PI(c)); }
static void cb_amn(HSD_ClassInfo* c) { T(8205); T(PI(c)); }

#include "dobj_gen.h"



static void preg_world(void)
{
    preg(DOB, sizeof DOB);
    preg(MOB, sizeof MOB);
    preg(POB, sizeof POB);
    preg(AOB, sizeof AOB);
    preg(MA, sizeof MA);
    preg(SAD, sizeof SAD);
    preg(SHA, sizeof SHA);
    preg(DD, sizeof DD);
    preg(MD, sizeof MD);
    preg(PD, sizeof PD);
    preg(CIN, sizeof CIN);
    preg(DINF, sizeof DINF);
    preg(MINF, sizeof MINF);
    preg(PINF, sizeof PINF);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(DOB, sizeof DOB);
    zero(MOB, sizeof MOB);
    zero(POB, sizeof POB);
    zero(AOB, sizeof AOB);
    zero(MA, sizeof MA);
    zero(SAD, sizeof SAD);
    zero(SHA, sizeof SHA);
    zero(DD, sizeof DD);
    zero(MD, sizeof MD);
    zero(PD, sizeof PD);
    zero(CIN, sizeof CIN);
    zero(DINF, sizeof DINF);
    zero(MINF, sizeof MINF);
    zero(PINF, sizeof PINF);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    for (i = 0; i < 3; i++) fill_HSD_DObj(&DOB[i]);
    for (i = 0; i < 2; i++) fill_HSD_MObj(&MOB[i]);
    for (i = 0; i < 2; i++) fill_HSD_PObj(&POB[i]);
    for (i = 0; i < 2; i++) fill_HSD_AObj(&AOB[i]);
    for (i = 0; i < 2; i++) fill_HSD_MatAnim(&MA[i]);
    for (i = 0; i < 2; i++) fill_HSD_ShapeAnimDObj(&SAD[i]);
    for (i = 0; i < 2; i++) fill_HSD_ShapeAnim(&SHA[i]);
    for (i = 0; i < 2; i++) fill_HSD_DObjDesc(&DD[i]);
    for (i = 0; i < 2; i++) fill_HSD_MObjDesc(&MD[i]);
    for (i = 0; i < 2; i++) fill_HSD_PObjDesc(&PD[i]);
    for (i = 0; i < 2; i++) fill_HSD_ClassInfo(&CIN[i]);
    for (i = 0; i < 2; i++) fill_HSD_DObjInfo(&DINF[i]);
    for (i = 0; i < 1; i++) fill_HSD_MObjInfo(&MINF[i]);
    for (i = 0; i < 1; i++) fill_HSD_PObjInfo(&PINF[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 3; i++) hash_HSD_DObj(&DOB[i]);
    for (i = 0; i < 2; i++) hash_HSD_MObj(&MOB[i]);
    for (i = 0; i < 2; i++) hash_HSD_PObj(&POB[i]);
    for (i = 0; i < 2; i++) hash_HSD_AObj(&AOB[i]);
    for (i = 0; i < 2; i++) hash_HSD_MatAnim(&MA[i]);
    for (i = 0; i < 2; i++) hash_HSD_ShapeAnimDObj(&SAD[i]);
    for (i = 0; i < 2; i++) hash_HSD_ShapeAnim(&SHA[i]);
    for (i = 0; i < 2; i++) hash_HSD_DObjDesc(&DD[i]);
    for (i = 0; i < 2; i++) hash_HSD_MObjDesc(&MD[i]);
    for (i = 0; i < 2; i++) hash_HSD_PObjDesc(&PD[i]);
    for (i = 0; i < 2; i++) hash_HSD_ClassInfo(&CIN[i]);
    for (i = 0; i < 2; i++) hash_HSD_DObjInfo(&DINF[i]);
    for (i = 0; i < 1; i++) hash_HSD_MObjInfo(&MINF[i]);
    for (i = 0; i < 1; i++) hash_HSD_PObjInfo(&PINF[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    hashg_all();
    hasho_all();
}

enum { OP_HSD_DObjSetCurrent = 0, OP_HSD_DObjGetFlags = 1, OP_HSD_DObjSetFlags = 2, OP_HSD_DObjClearFlags = 3, OP_HSD_DObjModifyFlags = 4, OP_HSD_DObjRemoveAnimByFlags = 5, OP_HSD_DObjRemoveAnimAllByFlags = 6, OP_HSD_DObjAddAnim = 7, OP_HSD_DObjAddAnimAll = 8, OP_HSD_DObjReqAnimByFlags = 9, OP_HSD_DObjReqAnimAllByFlags = 10, OP_HSD_DObjReqAnimAll = 11, OP_HSD_DObjAnim = 12, OP_HSD_DObjAnimAll = 13, OP_DObjLoad = 14, OP_HSD_DObjLoadDesc = 15, OP_HSD_DObjRemove = 16, OP_HSD_DObjRemoveAll = 17, OP_HSD_DObjSetDefaultClass = 18, OP_HSD_DObjAlloc = 19, OP_HSD_DObjResolveRefs = 20, OP_HSD_DObjResolveRefsAll = 21, OP_forceStringAllocation = 22, OP_HSD_DObjDisp = 23, OP_DObjRelease = 24, OP_DObjAmnesia = 25, OP_DObjInfoInit = 26 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();
freg((const void*) cb_setup); freg((const void*) cb_unset); freg((const void*) cb_disp); freg((const void*) cb_rel); freg((const void*) cb_dest); freg((const void*) cb_amn);
    for (op = 0; op < 27; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; break;  /* HSD_DObjSetCurrent */
        case 1: n = 300; break;  /* HSD_DObjGetFlags */
        case 2: n = 300; break;  /* HSD_DObjSetFlags */
        case 3: n = 300; break;  /* HSD_DObjClearFlags */
        case 4: n = 300; break;  /* HSD_DObjModifyFlags */
        case 5: n = 300; break;  /* HSD_DObjRemoveAnimByFlags */
        case 6: n = 300; break;  /* HSD_DObjRemoveAnimAllByFlags */
        case 7: n = 300; break;  /* HSD_DObjAddAnim */
        case 8: n = 300; break;  /* HSD_DObjAddAnimAll */
        case 9: n = 300; break;  /* HSD_DObjReqAnimByFlags */
        case 10: n = 300; break;  /* HSD_DObjReqAnimAllByFlags */
        case 11: n = 300; break;  /* HSD_DObjReqAnimAll */
        case 12: n = 300; break;  /* HSD_DObjAnim */
        case 13: n = 300; break;  /* HSD_DObjAnimAll */
        case 14: n = 300; break;  /* DObjLoad */
        case 15: n = 300; break;  /* HSD_DObjLoadDesc */
        case 16: n = 300; break;  /* HSD_DObjRemove */
        case 17: n = 300; break;  /* HSD_DObjRemoveAll */
        case 18: n = 300; break;  /* HSD_DObjSetDefaultClass */
        case 19: n = 300; break;  /* HSD_DObjAlloc */
        case 20: n = 300; break;  /* HSD_DObjResolveRefs */
        case 21: n = 300; break;  /* HSD_DObjResolveRefsAll */
        case 22: n = 300; break;  /* forceStringAllocation */
        case 23: n = 300; break;  /* HSD_DObjDisp */
        case 24: n = 300; break;  /* DObjRelease */
        case 25: n = 300; break;  /* DObjAmnesia */
        case 26: n = 300; break;  /* DObjInfoInit */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ int q; for (q = 0; q < 3; q++) { DOB[q].next = q < 2 ? &DOB[q + 1] : 0; DOB[q].mobj = &MOB[fv() & 1]; DOB[q].pobj = (fv() & 7) ? &POB[0] : 0; DOB[q].aobj = (fv() & 3) ? &AOB[fv() & 1] : 0; DOB[q].parent.class_info = (HSD_ClassInfo*) &DINF[q & 1]; } POB[0].next = &POB[1]; POB[1].next = 0; for (q = 0; q < 2; q++) { POB[q].parent.class_info = (HSD_ClassInfo*) &PINF[0]; MOB[q].parent.class_info = (HSD_ClassInfo*) &MINF[0]; } for (q = 0; q < 2; q++) { DINF[q].load = DObjLoad; DINF[q].parent.release = (void*) cb_rel; DINF[q].parent.destroy = (void*) cb_dest; } MINF[0].setup = (void*) cb_setup; MINF[0].unset = (void*) cb_unset; PINF[0].disp = (void*) cb_disp; DD[0].next = &DD[1]; DD[1].next = 0; hsdDObj.parent.head.parent = &CIN[0]; CIN[0].release = (void*) cb_rel; CIN[0].amnesia = (void*) cb_amn; default_class = (fv() & 1) ? &CIN[1] : 0; current_dobj = 0; }
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            ARMED = 1;
            if (MTU_SETJMP()) { ARMED = 0; printf("%d %d ABORT\n", op, k); continue; }
            switch (op) {
            case 0: {  HSD_DObj * a0 = pick_HSD_DObj(); HSD_DObjSetCurrent(a0);  } break;
            case 1: {  HSD_DObj * a0 = pick_HSD_DObj(); ret = (u32) HSD_DObjGetFlags(a0);  } break;
            case 2: {  HSD_DObj * a0 = pick_HSD_DObj(); u32 a1 = fv(); HSD_DObjSetFlags(a0, a1);  } break;
            case 3: {  HSD_DObj * a0 = pick_HSD_DObj(); u32 a1 = fv(); HSD_DObjClearFlags(a0, a1);  } break;
            case 4: {  HSD_DObj * a0 = pick_HSD_DObj(); u32 a1 = fv(); u32 a2 = fv(); HSD_DObjModifyFlags(a0, a1, a2);  } break;
            case 5: {  HSD_DObj * a0 = pick_HSD_DObj(); u32 a1 = fv(); HSD_DObjRemoveAnimByFlags(a0, a1);  } break;
            case 6: {  HSD_DObj * a0 = pick_HSD_DObj(); u32 a1 = fv(); HSD_DObjRemoveAnimAllByFlags(a0, a1);  } break;
            case 7: {  HSD_DObj * a0 = pick_HSD_DObj(); HSD_MatAnim * a1 = pick_HSD_MatAnim(); HSD_ShapeAnimDObj * a2 = pick_HSD_ShapeAnimDObj(); HSD_DObjAddAnim(a0, a1, a2);  } break;
            case 8: {  HSD_DObj * a0 = pick_HSD_DObj(); HSD_MatAnim * a1 = pick_HSD_MatAnim(); HSD_ShapeAnimDObj * a2 = pick_HSD_ShapeAnimDObj(); HSD_DObjAddAnimAll(a0, a1, a2);  } break;
            case 9: {  HSD_DObj * a0 = pick_HSD_DObj(); f32 a1 = rw(); u32 a2 = fv(); HSD_DObjReqAnimByFlags(a0, a1, a2);  } break;
            case 10: {  HSD_DObj * a0 = pick_HSD_DObj(); f32 a1 = rw(); u32 a2 = fv(); HSD_DObjReqAnimAllByFlags(a0, a1, a2);  } break;
            case 11: {  HSD_DObj * a0 = pick_HSD_DObj(); f32 a1 = rw(); HSD_DObjReqAnimAll(a0, a1);  } break;
            case 12: {  HSD_DObj * a0 = pick_HSD_DObj(); HSD_DObjAnim(a0);  } break;
            case 13: {  HSD_DObj * a0 = pick_HSD_DObj(); HSD_DObjAnimAll(a0);  } break;
            case 14: {  HSD_DObj * a0 = pick_HSD_DObj(); HSD_DObjDesc * a1 = pick_HSD_DObjDesc(); ret = (u32) DObjLoad(a0, a1);  } break;
            case 15: {  HSD_DObjDesc * a0 = pick_HSD_DObjDesc(); ret = PI(HSD_DObjLoadDesc(a0));  } break;
            case 16: {  HSD_DObj * a0 = pick_HSD_DObj(); HSD_DObjRemove(a0);  } break;
            case 17: {  HSD_DObj * a0 = pick_HSD_DObj(); HSD_DObjRemoveAll(a0);  } break;
            case 18: {  HSD_ClassInfo * a0 = pick_HSD_ClassInfo(); HSD_DObjSetDefaultClass(a0);  } break;
            case 19: {   ret = PI(HSD_DObjAlloc());  } break;
            case 20: {  HSD_DObj * a0 = pick_HSD_DObj(); HSD_DObjDesc * a1 = pick_HSD_DObjDesc(); HSD_DObjResolveRefs(a0, a1);  } break;
            case 21: {  HSD_DObj * a0 = pick_HSD_DObj(); HSD_DObjDesc * a1 = pick_HSD_DObjDesc(); HSD_DObjResolveRefsAll(a0, a1);  } break;
            case 22: {  HSD_DObj * a0 = pick_HSD_DObj(); HSD_MObj * a1 = pick_HSD_MObj(); forceStringAllocation(a0, a1);  } break;
            case 23: {  HSD_DObj * a0 = pick_HSD_DObj(); void* a1 = (void*) BUF; void* a2 = (void*) (BUF + 64); u32 a3 = fv(); HSD_DObjDisp(a0, a1, a2, a3);  } break;
            case 24: {  HSD_Class * a0 = (HSD_Class*) pick_HSD_DObj(); DObjRelease(a0);  } break;
            case 25: {  HSD_ClassInfo * a0 = pick_HSD_ClassInfo(); DObjAmnesia(a0);  } break;
            case 26: {   DObjInfoInit();  } break;
            }
            ARMED = 0;
            hash_world();

            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
