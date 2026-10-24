extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/sysdolphin/baselib/mobj.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static HSD_MObj MOB[3];
static HSD_TObj TOB[6];
static HSD_Material MAT[2];
static HSD_PEDesc PED[2];
static HSD_AObj AOB[2];
static HSD_MatAnim MAN[2];
static HSD_MObjDesc MDS[2];
static HSD_ClassInfo CIN[2];
static HSD_MObjInfo MINF[2];
static HSD_TObjInfo TINF[1];
static HSD_ImageDesc IMD[2];
static HSD_TObjDesc TDS[2];
static Vec3 VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static char STRS[4][64] = { "a.dat", "sym", "LbAd.dat", "x" };

static HSD_MObj* pick_HSD_MObj(void) { u32 r = rnd(); return &MOB[(r >> 8) % 3]; }
static HSD_MObj* picko_HSD_MObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &MOB[(r >> 8) % 3]; }
static HSD_TObj* pick_HSD_TObj(void) { u32 r = rnd(); return &TOB[(r >> 8) % 6]; }
static HSD_TObj* picko_HSD_TObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &TOB[(r >> 8) % 6]; }
static HSD_Material* pick_HSD_Material(void) { u32 r = rnd(); return &MAT[(r >> 8) % 2]; }
static HSD_Material* picko_HSD_Material(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &MAT[(r >> 8) % 2]; }
static HSD_PEDesc* pick_HSD_PEDesc(void) { u32 r = rnd(); return &PED[(r >> 8) % 2]; }
static HSD_PEDesc* picko_HSD_PEDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &PED[(r >> 8) % 2]; }
static HSD_AObj* pick_HSD_AObj(void) { u32 r = rnd(); return &AOB[(r >> 8) % 2]; }
static HSD_AObj* picko_HSD_AObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &AOB[(r >> 8) % 2]; }
static HSD_MatAnim* pick_HSD_MatAnim(void) { u32 r = rnd(); return &MAN[(r >> 8) % 2]; }
static HSD_MatAnim* picko_HSD_MatAnim(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &MAN[(r >> 8) % 2]; }
static HSD_MObjDesc* pick_HSD_MObjDesc(void) { u32 r = rnd(); return &MDS[(r >> 8) % 2]; }
static HSD_MObjDesc* picko_HSD_MObjDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &MDS[(r >> 8) % 2]; }
static HSD_ClassInfo* pick_HSD_ClassInfo(void) { u32 r = rnd(); return &CIN[(r >> 8) % 2]; }
static HSD_ClassInfo* picko_HSD_ClassInfo(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &CIN[(r >> 8) % 2]; }
static HSD_MObjInfo* pick_HSD_MObjInfo(void) { u32 r = rnd(); return &MINF[(r >> 8) % 2]; }
static HSD_MObjInfo* picko_HSD_MObjInfo(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &MINF[(r >> 8) % 2]; }
static HSD_TObjInfo* pick_HSD_TObjInfo(void) { u32 r = rnd(); return &TINF[(r >> 8) % 1]; }
static HSD_TObjInfo* picko_HSD_TObjInfo(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &TINF[(r >> 8) % 1]; }
static HSD_ImageDesc* pick_HSD_ImageDesc(void) { u32 r = rnd(); return &IMD[(r >> 8) % 2]; }
static HSD_ImageDesc* picko_HSD_ImageDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &IMD[(r >> 8) % 2]; }
static HSD_TObjDesc* pick_HSD_TObjDesc(void) { u32 r = rnd(); return &TDS[(r >> 8) % 2]; }
static HSD_TObjDesc* picko_HSD_TObjDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &TDS[(r >> 8) % 2]; }
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

static unsigned char TXA[4][64];
static HSD_TExp* TXP[3];
static unsigned char HP[2048];
static int HPI;
static void cb_setup(HSD_MObj* m, u32 r) { T(8200); T(PI(m)); T(r); }
static void cb_unset(HSD_MObj* m, u32 r) { T(8201); T(PI(m)); T(r); }
static void cb_rel(HSD_Class* c) { T(8203); T(PI(c)); }
static void cb_dest(HSD_Class* c) { T(8204); T(PI(c)); }
static void cb_amn(HSD_ClassInfo* c) { T(8205); T(PI(c)); }
static void cb_tmake(HSD_TObj* t, u32 a, u32 b, HSD_TExp** x, HSD_TExp** y, HSD_TExp** l) { T(8300); T(PI(t)); T(a); T(b); T(PI(x)); T(PI(y)); T(PI(l)); }

#include "mobj_gen.h"



static void preg_world(void)
{
    preg(MOB, sizeof MOB);
    preg(TOB, sizeof TOB);
    preg(MAT, sizeof MAT);
    preg(PED, sizeof PED);
    preg(AOB, sizeof AOB);
    preg(MAN, sizeof MAN);
    preg(MDS, sizeof MDS);
    preg(CIN, sizeof CIN);
    preg(MINF, sizeof MINF);
    preg(TINF, sizeof TINF);
    preg(IMD, sizeof IMD);
    preg(TDS, sizeof TDS);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(MOB, sizeof MOB);
    zero(TOB, sizeof TOB);
    zero(MAT, sizeof MAT);
    zero(PED, sizeof PED);
    zero(AOB, sizeof AOB);
    zero(MAN, sizeof MAN);
    zero(MDS, sizeof MDS);
    zero(CIN, sizeof CIN);
    zero(MINF, sizeof MINF);
    zero(TINF, sizeof TINF);
    zero(IMD, sizeof IMD);
    zero(TDS, sizeof TDS);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    for (i = 0; i < 3; i++) fill_HSD_MObj(&MOB[i]);
    for (i = 0; i < 6; i++) fill_HSD_TObj(&TOB[i]);
    for (i = 0; i < 2; i++) fill_HSD_Material(&MAT[i]);
    for (i = 0; i < 2; i++) fill_HSD_PEDesc(&PED[i]);
    for (i = 0; i < 2; i++) fill_HSD_AObj(&AOB[i]);
    for (i = 0; i < 2; i++) fill_HSD_MatAnim(&MAN[i]);
    for (i = 0; i < 2; i++) fill_HSD_MObjDesc(&MDS[i]);
    for (i = 0; i < 2; i++) fill_HSD_ClassInfo(&CIN[i]);
    for (i = 0; i < 2; i++) fill_HSD_MObjInfo(&MINF[i]);
    for (i = 0; i < 1; i++) fill_HSD_TObjInfo(&TINF[i]);
    for (i = 0; i < 2; i++) fill_HSD_ImageDesc(&IMD[i]);
    for (i = 0; i < 2; i++) fill_HSD_TObjDesc(&TDS[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 3; i++) hash_HSD_MObj(&MOB[i]);
    for (i = 0; i < 6; i++) hash_HSD_TObj(&TOB[i]);
    for (i = 0; i < 2; i++) hash_HSD_Material(&MAT[i]);
    for (i = 0; i < 2; i++) hash_HSD_PEDesc(&PED[i]);
    for (i = 0; i < 2; i++) hash_HSD_AObj(&AOB[i]);
    for (i = 0; i < 2; i++) hash_HSD_MatAnim(&MAN[i]);
    for (i = 0; i < 2; i++) hash_HSD_MObjDesc(&MDS[i]);
    for (i = 0; i < 2; i++) hash_HSD_ClassInfo(&CIN[i]);
    for (i = 0; i < 2; i++) hash_HSD_MObjInfo(&MINF[i]);
    for (i = 0; i < 1; i++) hash_HSD_TObjInfo(&TINF[i]);
    for (i = 0; i < 2; i++) hash_HSD_ImageDesc(&IMD[i]);
    for (i = 0; i < 2; i++) hash_HSD_TObjDesc(&TDS[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    hashg_all();
    hasho_all();
}

enum { OP_HSD_MObjSetCurrent = 0, OP_HSD_MObjSetFlags = 1, OP_HSD_MObjClearFlags = 2, OP_HSD_MObjRemoveAnimByFlags = 3, OP_HSD_MObjAddAnim = 4, OP_HSD_MObjReqAnimByFlags = 5, OP_HSD_MObjReqAnim = 6, OP_MObjUpdateFunc = 7, OP_HSD_MObjAnim = 8, OP_MObjLoad = 9, OP_HSD_MObjLoadDesc = 10, OP_MObjMakeTExp = 11, OP_HSD_MObjCompileTev = 12, OP_MObjSetupTev = 13, OP_HSD_MObjSetup = 14, OP_HSD_MObjUnset = 15, OP_HSD_MObjSetToonTextureImage = 16, OP_HSD_MObjSetDiffuseColor = 17, OP_HSD_MObjSetAlpha = 18, OP_HSD_MObjGetTObj = 19, OP_HSD_MObjRemove = 20, OP_HSD_MObjAlloc = 21, OP_HSD_MaterialAlloc = 22, OP_HSD_MObjAddShadowTexture = 23, OP_HSD_MObjDeleteShadowTexture = 24, OP_MObjRelease = 25, OP_MObjAmnesia = 26, OP_MObjInfoInit = 27 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();
preg(TXA, sizeof TXA); preg(TXP, sizeof TXP); preg(HP, sizeof HP); freg((const void*) cb_setup); freg((const void*) cb_unset); freg((const void*) cb_rel); freg((const void*) cb_dest); freg((const void*) cb_amn); freg((const void*) cb_tmake);
    for (op = 0; op < 28; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; break;  /* HSD_MObjSetCurrent */
        case 1: n = 300; break;  /* HSD_MObjSetFlags */
        case 2: n = 300; break;  /* HSD_MObjClearFlags */
        case 3: n = 300; break;  /* HSD_MObjRemoveAnimByFlags */
        case 4: n = 300; break;  /* HSD_MObjAddAnim */
        case 5: n = 300; break;  /* HSD_MObjReqAnimByFlags */
        case 6: n = 300; break;  /* HSD_MObjReqAnim */
        case 7: n = 300; break;  /* MObjUpdateFunc */
        case 8: n = 300; break;  /* HSD_MObjAnim */
        case 9: n = 300; break;  /* MObjLoad */
        case 10: n = 300; break;  /* HSD_MObjLoadDesc */
        case 11: n = 300; break;  /* MObjMakeTExp */
        case 12: n = 300; break;  /* HSD_MObjCompileTev */
        case 13: n = 300; break;  /* MObjSetupTev */
        case 14: n = 300; break;  /* HSD_MObjSetup */
        case 15: n = 300; break;  /* HSD_MObjUnset */
        case 16: n = 300; break;  /* HSD_MObjSetToonTextureImage */
        case 17: n = 300; break;  /* HSD_MObjSetDiffuseColor */
        case 18: n = 300; break;  /* HSD_MObjSetAlpha */
        case 19: n = 300; break;  /* HSD_MObjGetTObj */
        case 20: n = 300; break;  /* HSD_MObjRemove */
        case 21: n = 300; break;  /* HSD_MObjAlloc */
        case 22: n = 300; break;  /* HSD_MaterialAlloc */
        case 23: n = 300; break;  /* HSD_MObjAddShadowTexture */
        case 24: n = 300; break;  /* HSD_MObjDeleteShadowTexture */
        case 25: n = 300; break;  /* MObjRelease */
        case 26: n = 300; break;  /* MObjAmnesia */
        case 27: n = 300; break;  /* MObjInfoInit */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ int q; HPI = 0; for (q = 0; q < 3; q++) { MOB[q].parent.class_info = (HSD_ClassInfo*) &MINF[q & 1]; MOB[q].tobj = (fv() & 3) ? &TOB[fv() & 1] : 0; MOB[q].mat = &MAT[fv() & 1]; MOB[q].pe = (fv() & 1) ? &PED[fv() & 1] : 0; MOB[q].aobj = (fv() & 3) ? &AOB[fv() & 1] : 0; MOB[q].tevdesc = (fv() & 1) ? (struct _HSD_TExpTevDesc*) TXA[3] : 0; MOB[q].texp = (fv() & 1) ? (HSD_TExp*) TXA[2] : 0; } for (q = 0; q < 6; q++) { TOB[q].next = (q == 0 || q == 1 || q == 3) ? &TOB[q + 1] : 0; TOB[q].parent.parent.class_info = (HSD_ClassInfo*) &TINF[0]; } for (q = 0; q < 2; q++) { MINF[q].load = MObjLoad; MINF[q].make_texp = MObjMakeTExp; MINF[q].setup_tev = MObjSetupTev; MINF[q].setup = (void*) cb_setup; MINF[q].unset = (void*) cb_unset; MINF[q].parent.release = (void*) cb_rel; MINF[q].parent.destroy = (void*) cb_dest; } TINF[0].make_texp = (void*) cb_tmake; for (q = 0; q < 2; q++) { MDS[q].class_name = (fv() & 1) ? (char*) STRS[0] : 0; MDS[q].mat = &MAT[0]; MDS[q].pedesc = (fv() & 1) ? &PED[0] : 0; } hsdMObj.parent.head.parent = &CIN[0]; CIN[0].release = (void*) cb_rel; CIN[0].amnesia = (void*) cb_amn; default_class = (fv() & 1) ? &CIN[1] : 0; current_mobj = 0; tobj_toon = (fv() & 1) ? &TOB[5] : 0; tobj_shadows = (fv() & 1) ? &TOB[3] : 0; for (q = 0; q < 3; q++) TXP[q] = 0; }
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            ARMED = 1;
            if (MTU_SETJMP()) { ARMED = 0; printf("%d %d ABORT\n", op, k); continue; }
            switch (op) {
            case 0: {  HSD_MObj * a0 = pick_HSD_MObj(); HSD_MObjSetCurrent(a0);  } break;
            case 1: {  HSD_MObj * a0 = pick_HSD_MObj(); u32 a1 = fv(); HSD_MObjSetFlags(a0, a1);  } break;
            case 2: {  HSD_MObj * a0 = pick_HSD_MObj(); u32 a1 = fv(); HSD_MObjClearFlags(a0, a1);  } break;
            case 3: {  HSD_MObj * a0 = pick_HSD_MObj(); u32 a1 = fv(); HSD_MObjRemoveAnimByFlags(a0, a1);  } break;
            case 4: {  HSD_MObj * a0 = pick_HSD_MObj(); HSD_MatAnim * a1 = pick_HSD_MatAnim(); HSD_MObjAddAnim(a0, a1);  } break;
            case 5: {  HSD_MObj * a0 = pick_HSD_MObj(); f32 a1 = rw(); u32 a2 = fv(); HSD_MObjReqAnimByFlags(a0, a1, a2);  } break;
            case 6: {  HSD_MObj * a0 = pick_HSD_MObj(); f32 a1 = rw(); HSD_MObjReqAnim(a0, a1);  } break;
            case 7: {  void * a0 = (void*) picko_HSD_MObj(); enum_t a1 = (int) (fv() % 20); HSD_ObjData * a2 = (HSD_ObjData *) BUF; MObjUpdateFunc(a0, a1, a2);  } break;
            case 8: {  HSD_MObj * a0 = pick_HSD_MObj(); HSD_MObjAnim(a0);  } break;
            case 9: {  HSD_MObj * a0 = pick_HSD_MObj(); HSD_MObjDesc * a1 = pick_HSD_MObjDesc(); ret = (u32) MObjLoad(a0, a1);  } break;
            case 10: {  HSD_MObjDesc * a0 = pick_HSD_MObjDesc(); ret = PI(HSD_MObjLoadDesc(a0));  } break;
            case 11: {  HSD_MObj * a0 = pick_HSD_MObj(); HSD_TObj * a1 = pick_HSD_TObj(); HSD_TExp ** a2 = &TXP[fv() % 3]; ret = PI(MObjMakeTExp(a0, a1, a2));  } break;
            case 12: {  HSD_MObj * a0 = pick_HSD_MObj(); HSD_MObjCompileTev(a0);  } break;
            case 13: {  HSD_MObj * a0 = pick_HSD_MObj(); HSD_TObj * a1 = pick_HSD_TObj(); u32 a2 = fv(); MObjSetupTev(a0, a1, a2);  } break;
            case 14: {  HSD_MObj * a0 = pick_HSD_MObj(); u32 a1 = fv(); HSD_MObjSetup(a0, a1);  } break;
            case 15: {  HSD_MObj * a0 = pick_HSD_MObj(); u32 a1 = fv(); HSD_MObjUnset(a0, a1);  } break;
            case 16: {  HSD_ImageDesc * a0 = &IMD[fv() & 1]; HSD_MObjSetToonTextureImage(a0);  } break;
            case 17: {  HSD_MObj * a0 = pick_HSD_MObj(); u8 a1 = fv(); u8 a2 = fv(); u8 a3 = fv(); HSD_MObjSetDiffuseColor(a0, a1, a2, a3);  } break;
            case 18: {  HSD_MObj * a0 = pick_HSD_MObj(); f32 a1 = rw(); HSD_MObjSetAlpha(a0, a1);  } break;
            case 19: {  HSD_MObj * a0 = pick_HSD_MObj(); ret = PI(HSD_MObjGetTObj(a0));  } break;
            case 20: {  HSD_MObj * a0 = pick_HSD_MObj(); HSD_MObjRemove(a0);  } break;
            case 21: {   ret = PI(HSD_MObjAlloc());  } break;
            case 22: {   ret = PI(HSD_MaterialAlloc());  } break;
            case 23: {  HSD_TObj * a0 = pick_HSD_TObj(); HSD_MObjAddShadowTexture(a0);  } break;
            case 24: {  HSD_TObj * a0 = pick_HSD_TObj(); HSD_MObjDeleteShadowTexture(a0);  } break;
            case 25: {  HSD_Class * a0 = (HSD_Class *) BUF; MObjRelease(a0);  } break;
            case 26: {  HSD_ClassInfo * a0 = pick_HSD_ClassInfo(); MObjAmnesia(a0);  } break;
            case 27: {   MObjInfoInit();  } break;
            }
            ARMED = 0;
            hash_world();

            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
