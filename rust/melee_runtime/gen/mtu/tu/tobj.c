extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/sysdolphin/baselib/tobj.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static HSD_TObj TOB[6];
static HSD_TexAnim TA[2];
static HSD_AObj AOB[2];
static HSD_AObjDesc AD[2];
static HSD_ImageDesc IMG[3];
static HSD_Tlut TL[3];
static HSD_TlutDesc TLD[3];
static HSD_TObjTev TEV[2];
static HSD_TObjTevDesc TEVD[2];
static HSD_TObjDesc TDS[2];
static HSD_TexLODDesc LOD[1];
static HSD_TObjInfo TINF[1];
static HSD_ClassInfo CIN[2];
static HSD_LObj LOB[1];
static HSD_CObj COB[1];
static HSD_ObjData OBD[2];
static Vec3 VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static void* PPA[4];
static char STRS[4][64] = { "a.dat", "sym", "LbAd.dat", "x" };

static HSD_TObj* pick_HSD_TObj(void) { u32 r = rnd(); return &TOB[(r >> 8) % 6]; }
static HSD_TObj* picko_HSD_TObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &TOB[(r >> 8) % 6]; }
static HSD_TexAnim* pick_HSD_TexAnim(void) { u32 r = rnd(); return &TA[(r >> 8) % 2]; }
static HSD_TexAnim* picko_HSD_TexAnim(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &TA[(r >> 8) % 2]; }
static HSD_AObj* pick_HSD_AObj(void) { u32 r = rnd(); return &AOB[(r >> 8) % 2]; }
static HSD_AObj* picko_HSD_AObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &AOB[(r >> 8) % 2]; }
static HSD_AObjDesc* pick_HSD_AObjDesc(void) { u32 r = rnd(); return &AD[(r >> 8) % 2]; }
static HSD_AObjDesc* picko_HSD_AObjDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &AD[(r >> 8) % 2]; }
static HSD_ImageDesc* pick_HSD_ImageDesc(void) { u32 r = rnd(); return &IMG[(r >> 8) % 3]; }
static HSD_ImageDesc* picko_HSD_ImageDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &IMG[(r >> 8) % 3]; }
static HSD_Tlut* pick_HSD_Tlut(void) { u32 r = rnd(); return &TL[(r >> 8) % 3]; }
static HSD_Tlut* picko_HSD_Tlut(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &TL[(r >> 8) % 3]; }
static HSD_TlutDesc* pick_HSD_TlutDesc(void) { u32 r = rnd(); return &TLD[(r >> 8) % 3]; }
static HSD_TlutDesc* picko_HSD_TlutDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &TLD[(r >> 8) % 3]; }
static HSD_TObjTev* pick_HSD_TObjTev(void) { u32 r = rnd(); return &TEV[(r >> 8) % 2]; }
static HSD_TObjTev* picko_HSD_TObjTev(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &TEV[(r >> 8) % 2]; }
static HSD_TObjTevDesc* pick_HSD_TObjTevDesc(void) { u32 r = rnd(); return &TEVD[(r >> 8) % 2]; }
static HSD_TObjTevDesc* picko_HSD_TObjTevDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &TEVD[(r >> 8) % 2]; }
static HSD_TObjDesc* pick_HSD_TObjDesc(void) { u32 r = rnd(); return &TDS[(r >> 8) % 2]; }
static HSD_TObjDesc* picko_HSD_TObjDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &TDS[(r >> 8) % 2]; }
static HSD_TexLODDesc* pick_HSD_TexLODDesc(void) { u32 r = rnd(); return &LOD[(r >> 8) % 1]; }
static HSD_TexLODDesc* picko_HSD_TexLODDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &LOD[(r >> 8) % 1]; }
static HSD_TObjInfo* pick_HSD_TObjInfo(void) { u32 r = rnd(); return &TINF[(r >> 8) % 1]; }
static HSD_TObjInfo* picko_HSD_TObjInfo(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &TINF[(r >> 8) % 1]; }
static HSD_ClassInfo* pick_HSD_ClassInfo(void) { u32 r = rnd(); return &CIN[(r >> 8) % 2]; }
static HSD_ClassInfo* picko_HSD_ClassInfo(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &CIN[(r >> 8) % 2]; }
static HSD_LObj* pick_HSD_LObj(void) { u32 r = rnd(); return &LOB[(r >> 8) % 1]; }
static HSD_LObj* picko_HSD_LObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &LOB[(r >> 8) % 1]; }
static HSD_CObj* pick_HSD_CObj(void) { u32 r = rnd(); return &COB[(r >> 8) % 1]; }
static HSD_CObj* picko_HSD_CObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &COB[(r >> 8) % 1]; }
static HSD_ObjData* pick_HSD_ObjData(void) { u32 r = rnd(); return &OBD[(r >> 8) % 2]; }
static HSD_ObjData* picko_HSD_ObjData(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &OBD[(r >> 8) % 2]; }
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

static HSD_ImageDesc* IMT[3][4];
static HSD_Tlut* TLT[3][4];
static HSD_TlutDesc* TLDT[3][4];
static unsigned char HP[4096];
static unsigned char MA[1024];
static unsigned char TXA[4][64];
static int HPI, MAI; static float VF[4] = { 0.0f, 1.0f, 2.0f, 0.5f };
static void cb_rel(HSD_Class* c) { T(8203); T(PI(c)); }
static void cb_dest(HSD_Class* c) { T(8204); T(PI(c)); }
static void cb_amn(HSD_ClassInfo* c) { T(8205); T(PI(c)); }
static const unsigned char FMT[12] = { 0, 1, 2, 3, 4, 5, 6, 8, 9, 10, 14, 7 };

#include "tobj_gen.h"
float fabsf(float x) { union { float f; u32 u; } c; c.f = x; c.u &= 0x7FFFFFFFu; return c.f; }


static void preg_world(void)
{
    preg(TOB, sizeof TOB);
    preg(TA, sizeof TA);
    preg(AOB, sizeof AOB);
    preg(AD, sizeof AD);
    preg(IMG, sizeof IMG);
    preg(TL, sizeof TL);
    preg(TLD, sizeof TLD);
    preg(TEV, sizeof TEV);
    preg(TEVD, sizeof TEVD);
    preg(TDS, sizeof TDS);
    preg(LOD, sizeof LOD);
    preg(TINF, sizeof TINF);
    preg(CIN, sizeof CIN);
    preg(LOB, sizeof LOB);
    preg(COB, sizeof COB);
    preg(OBD, sizeof OBD);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS); preg(PPA, sizeof PPA);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(TOB, sizeof TOB);
    zero(TA, sizeof TA);
    zero(AOB, sizeof AOB);
    zero(AD, sizeof AD);
    zero(IMG, sizeof IMG);
    zero(TL, sizeof TL);
    zero(TLD, sizeof TLD);
    zero(TEV, sizeof TEV);
    zero(TEVD, sizeof TEVD);
    zero(TDS, sizeof TDS);
    zero(LOD, sizeof LOD);
    zero(TINF, sizeof TINF);
    zero(CIN, sizeof CIN);
    zero(LOB, sizeof LOB);
    zero(COB, sizeof COB);
    zero(OBD, sizeof OBD);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    for (i = 0; i < 4; i++) PPA[i] = 0;
    for (i = 0; i < 6; i++) fill_HSD_TObj(&TOB[i]);
    for (i = 0; i < 2; i++) fill_HSD_TexAnim(&TA[i]);
    for (i = 0; i < 2; i++) fill_HSD_AObj(&AOB[i]);
    for (i = 0; i < 2; i++) fill_HSD_AObjDesc(&AD[i]);
    for (i = 0; i < 3; i++) fill_HSD_ImageDesc(&IMG[i]);
    for (i = 0; i < 3; i++) fill_HSD_Tlut(&TL[i]);
    for (i = 0; i < 3; i++) fill_HSD_TlutDesc(&TLD[i]);
    for (i = 0; i < 2; i++) fill_HSD_TObjTev(&TEV[i]);
    for (i = 0; i < 2; i++) fill_HSD_TObjTevDesc(&TEVD[i]);
    for (i = 0; i < 2; i++) fill_HSD_TObjDesc(&TDS[i]);
    for (i = 0; i < 1; i++) fill_HSD_TexLODDesc(&LOD[i]);
    for (i = 0; i < 1; i++) fill_HSD_TObjInfo(&TINF[i]);
    for (i = 0; i < 2; i++) fill_HSD_ClassInfo(&CIN[i]);
    for (i = 0; i < 1; i++) fill_HSD_LObj(&LOB[i]);
    for (i = 0; i < 1; i++) fill_HSD_CObj(&COB[i]);
    for (i = 0; i < 2; i++) fill_HSD_ObjData(&OBD[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 6; i++) hash_HSD_TObj(&TOB[i]);
    for (i = 0; i < 2; i++) hash_HSD_TexAnim(&TA[i]);
    for (i = 0; i < 2; i++) hash_HSD_AObj(&AOB[i]);
    for (i = 0; i < 2; i++) hash_HSD_AObjDesc(&AD[i]);
    for (i = 0; i < 3; i++) hash_HSD_ImageDesc(&IMG[i]);
    for (i = 0; i < 3; i++) hash_HSD_Tlut(&TL[i]);
    for (i = 0; i < 3; i++) hash_HSD_TlutDesc(&TLD[i]);
    for (i = 0; i < 2; i++) hash_HSD_TObjTev(&TEV[i]);
    for (i = 0; i < 2; i++) hash_HSD_TObjTevDesc(&TEVD[i]);
    for (i = 0; i < 2; i++) hash_HSD_TObjDesc(&TDS[i]);
    for (i = 0; i < 1; i++) hash_HSD_TexLODDesc(&LOD[i]);
    for (i = 0; i < 1; i++) hash_HSD_TObjInfo(&TINF[i]);
    for (i = 0; i < 2; i++) hash_HSD_ClassInfo(&CIN[i]);
    for (i = 0; i < 1; i++) hash_HSD_LObj(&LOB[i]);
    for (i = 0; i < 1; i++) hash_HSD_CObj(&COB[i]);
    for (i = 0; i < 2; i++) hash_HSD_ObjData(&OBD[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    for (i = 0; i < 4; i++) HM(PI(PPA[i]));
    hashg_all();
    hasho_all();
}

enum { OP_HSD_TObjRemoveAnim = 0, OP_HSD_TObjRemoveAnimAll = 1, OP_lookupTextureAnim = 2, OP_HSD_TObjAddAnim = 3, OP_HSD_TObjAddAnimAll = 4, OP_HSD_TObjReqAnimByFlags = 5, OP_HSD_TObjReqAnimAllByFlags = 6, OP_HSD_TObjReqAnim = 7, OP_HSD_TObjReqAnimAll = 8, OP_TObjUpdateFunc = 9, OP_HSD_TObjAnim = 10, OP_HSD_TObjAnimAll = 11, OP_TObjLoad = 12, OP_HSD_TObjLoadDesc = 13, OP_HSD_TlutLoadDesc = 14, OP_HSD_TObjTevLoadDesc = 15, OP__HSD_TObjGetCurrentByType = 16, OP_MakeTextureMtx = 17, OP_TObjSetupMtx = 18, OP_setupTextureCoordGen = 19, OP_setupTextureCoordGenBump = 20, OP_setupTextureCoordGenToon = 21, OP_HSD_TObjSetupTextureCoordGen = 22, OP_TObjSetupTevModulateShadow = 23, OP_SetupEmbossBumpTev = 24, OP_HSD_TObjSetupVolatileTev = 25, OP_MakeColorGenTExp = 26, OP_TObjMakeTExp = 27, OP_HSD_TObjAssignResources = 28, OP_DifferentTluts = 29, OP_HSD_TObjSetup = 30, OP_HSD_Index2TexMtx = 31, OP_HSD_TObjRemove = 32, OP_HSD_TObjRemoveAll = 33, OP_HSD_TObjGetNext = 34, OP_HSD_TObjSetDefaultClass = 35, OP_HSD_TObjGetDefaultClass = 36, OP_HSD_TObjAlloc = 37, OP_HSD_TObjFree = 38, OP_HSD_TlutAlloc = 39, OP_HSD_TlutFree = 40, OP_HSD_TlutRemove = 41, OP_HSD_TObjTevAlloc = 42, OP_HSD_TObjTevFree = 43, OP_HSD_TObjTevRemove = 44, OP_HSD_ImageDescAlloc = 45, OP_HSD_ImageDescFree = 46, OP_HSD_ImageDescCopyFromEFB = 47, OP_TObjRelease = 48, OP_TObjAmnesia = 49, OP_TObjInfoInit = 50 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();
preg(IMT, sizeof IMT); preg(TLT, sizeof TLT); preg(TLDT, sizeof TLDT); preg(HP, sizeof HP); preg(MA, sizeof MA); preg(TXA, sizeof TXA); freg((const void*) cb_rel); freg((const void*) cb_dest); freg((const void*) cb_amn);
    for (op = 0; op < 51; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 250; break;  /* HSD_TObjRemoveAnim */
        case 1: n = 250; break;  /* HSD_TObjRemoveAnimAll */
        case 2: n = 250; break;  /* lookupTextureAnim */
        case 3: n = 250; break;  /* HSD_TObjAddAnim */
        case 4: n = 250; break;  /* HSD_TObjAddAnimAll */
        case 5: n = 250; break;  /* HSD_TObjReqAnimByFlags */
        case 6: n = 250; break;  /* HSD_TObjReqAnimAllByFlags */
        case 7: n = 250; break;  /* HSD_TObjReqAnim */
        case 8: n = 250; break;  /* HSD_TObjReqAnimAll */
        case 9: n = 250; break;  /* TObjUpdateFunc */
        case 10: n = 250; break;  /* HSD_TObjAnim */
        case 11: n = 250; break;  /* HSD_TObjAnimAll */
        case 12: n = 250; break;  /* TObjLoad */
        case 13: n = 250; break;  /* HSD_TObjLoadDesc */
        case 14: n = 250; break;  /* HSD_TlutLoadDesc */
        case 15: n = 250; break;  /* HSD_TObjTevLoadDesc */
        case 16: n = 250; break;  /* _HSD_TObjGetCurrentByType */
        case 17: n = 250; break;  /* MakeTextureMtx */
        case 18: n = 250; break;  /* TObjSetupMtx */
        case 19: n = 250; break;  /* setupTextureCoordGen */
        case 20: n = 250; break;  /* setupTextureCoordGenBump */
        case 21: n = 250; break;  /* setupTextureCoordGenToon */
        case 22: n = 250; break;  /* HSD_TObjSetupTextureCoordGen */
        case 23: n = 250; break;  /* TObjSetupTevModulateShadow */
        case 24: n = 250; break;  /* SetupEmbossBumpTev */
        case 25: n = 250; break;  /* HSD_TObjSetupVolatileTev */
        case 26: n = 250; break;  /* MakeColorGenTExp */
        case 27: n = 250; break;  /* TObjMakeTExp */
        case 28: n = 250; break;  /* HSD_TObjAssignResources */
        case 29: n = 250; break;  /* DifferentTluts */
        case 30: n = 250; break;  /* HSD_TObjSetup */
        case 31: n = 250; break;  /* HSD_Index2TexMtx */
        case 32: n = 250; break;  /* HSD_TObjRemove */
        case 33: n = 250; break;  /* HSD_TObjRemoveAll */
        case 34: n = 250; break;  /* HSD_TObjGetNext */
        case 35: n = 250; break;  /* HSD_TObjSetDefaultClass */
        case 36: n = 250; break;  /* HSD_TObjGetDefaultClass */
        case 37: n = 250; break;  /* HSD_TObjAlloc */
        case 38: n = 250; break;  /* HSD_TObjFree */
        case 39: n = 250; break;  /* HSD_TlutAlloc */
        case 40: n = 250; break;  /* HSD_TlutFree */
        case 41: n = 250; break;  /* HSD_TlutRemove */
        case 42: n = 250; break;  /* HSD_TObjTevAlloc */
        case 43: n = 250; break;  /* HSD_TObjTevFree */
        case 44: n = 250; break;  /* HSD_TObjTevRemove */
        case 45: n = 250; break;  /* HSD_ImageDescAlloc */
        case 46: n = 250; break;  /* HSD_ImageDescFree */
        case 47: n = 250; break;  /* HSD_ImageDescCopyFromEFB */
        case 48: n = 250; break;  /* TObjRelease */
        case 49: n = 250; break;  /* TObjAmnesia */
        case 50: n = 250; break;  /* TObjInfoInit */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ int q, j; HPI = 0; MAI = 0; for (q = 0; q < 3; q++) for (j = 0; j < 4; j++) { IMT[q][j] = (j < 3) ? &IMG[(j + q) % 3] : 0; TLT[q][j] = (j < 1 + (int) (fv() % 3)) ? &TL[(j + q) % 3] : 0; TLDT[q][j] = (j < 3) ? &TLD[(j + q) % 3] : 0; } for (q = 0; q < 3; q++) TLT[q][3] = 0; for (q = 0; q < 6; q++) { TOB[q].next = (q < 3) ? &TOB[q + 1] : 0; if (q == 3) TOB[q].next = 0; TOB[q].parent.parent.class_info = (HSD_ClassInfo*) &TINF[0]; TOB[q].id = (GXTexMapID) (fv() % 9); TOB[q].coord = (GXTexCoordID) (fv() % 9); TOB[q].repeat_s = (fv() % 16) ? 1 + fv() % 4 : 0; TOB[q].repeat_t = (fv() % 16) ? 1 + fv() % 4 : 0; TOB[q].wrap_s = (GXTexWrapMode) (fv() % 3); TOB[q].wrap_t = (GXTexWrapMode) (fv() % 3); TOB[q].imagedesc = &IMG[fv() % 3]; TOB[q].lod = (q < 2 || (fv() & 1)) ? &LOD[0] : 0; TOB[q].tlut = (fv() & 3) ? &TL[fv() % 3] : 0; TOB[q].tluttbl = TLT[fv() % 3]; TOB[q].imagetbl = IMT[fv() % 3]; TOB[q].tlut_no = (fv() & 1) ? -1 : (s32) (fv() % 4); TOB[q].tev = (q < 2 || (fv() & 3)) ? &TEV[fv() & 1] : 0; TOB[q].aobj = (fv() & 3) ? &AOB[fv() & 1] : 0; } for (q = 0; q < 3; q++) { IMG[q].format = (GXTexFmt) FMT[fv() % 12]; IMG[q].image_ptr = (fv() & 7) ? (void*) BUF : 0; IMG[q].mipmap = fv() & 1; IMG[q].width = 1 + fv() % 64; IMG[q].height = 1 + fv() % 64; TL[q].lut = BUF; TL[q].n_entries = fv() % 300; TL[q].fmt = (GXTlutFmt) (fv() % 3); } for (q = 0; q < 2; q++) { TA[q].next = q < 1 ? &TA[1] : 0; TA[q].id = (GXTexMapID) (fv() % 9); TA[q].aobjdesc = (fv() & 3) ? &AD[fv() & 1] : 0; TA[q].imagetbl = IMT[fv() % 3]; TA[q].tluttbl = TLDT[fv() % 3]; TA[q].n_tluttbl = fv() % 3; } for (q = 0; q < 2; q++) { TDS[q].next = q < 1 ? &TDS[1] : 0; TDS[q].class_name = (fv() & 1) ? (char*) STRS[0] : 0; TDS[q].imagedesc = &IMG[fv() % 3]; TDS[q].tlutdesc = (fv() & 1) ? &TLD[fv() % 3] : 0; TDS[q].tev = (fv() & 1) ? &TEVD[fv() & 1] : 0; TDS[q].lod = (fv() & 1) ? &LOD[0] : 0; } TINF[0].make_mtx = MakeTextureMtx; TINF[0].load = TObjLoad; TINF[0].make_texp = TObjMakeTExp; TINF[0].parent.release = (void*) cb_rel; TINF[0].parent.destroy = (void*) cb_dest; hsdTObj.parent.head.parent = &CIN[0]; CIN[0].release = (void*) cb_rel; CIN[0].amnesia = (void*) cb_amn; default_class = (fv() & 1) ? &TINF[0] : 0; tobj_head = (fv() & 1) ? &TOB[fv() % 4] : 0; }
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            ARMED = 1;
            if (MTU_SETJMP()) { ARMED = 0; printf("%d %d ABORT\n", op, k); continue; }
            switch (op) {
            case 0: {  HSD_TObj * a0 = pick_HSD_TObj(); HSD_TObjRemoveAnim(a0);  } break;
            case 1: {  HSD_TObj * a0 = pick_HSD_TObj(); HSD_TObjRemoveAnimAll(a0);  } break;
            case 2: {  s32 a0 = fv(); HSD_TexAnim * a1 = pick_HSD_TexAnim(); ret = PI(lookupTextureAnim(a0, a1));  } break;
            case 3: {  HSD_TObj * a0 = pick_HSD_TObj(); HSD_TexAnim * a1 = pick_HSD_TexAnim(); HSD_TObjAddAnim(a0, a1);  } break;
            case 4: {  HSD_TObj * a0 = pick_HSD_TObj(); HSD_TexAnim * a1 = pick_HSD_TexAnim(); HSD_TObjAddAnimAll(a0, a1);  } break;
            case 5: {  HSD_TObj * a0 = pick_HSD_TObj(); f32 a1 = rw(); u32 a2 = fv() | 1; HSD_TObjReqAnimByFlags(a0, a1, a2);  } break;
            case 6: {  HSD_TObj * a0 = pick_HSD_TObj(); f32 a1 = rw(); u32 a2 = fv(); HSD_TObjReqAnimAllByFlags(a0, a1, a2);  } break;
            case 7: {  HSD_TObj * a0 = pick_HSD_TObj(); f32 a1 = rw(); HSD_TObjReqAnim(a0, a1);  } break;
            case 8: {  HSD_TObj * a0 = pick_HSD_TObj(); f32 a1 = rw(); HSD_TObjReqAnimAll(a0, a1);  } break;
            case 9: {  void * a0 = (void*) &TOB[fv() % 2]; enum_t a1 = (int) (fv() % 24); HSD_ObjData * a2 = (HSD_ObjData*) &VF[fv() % 4]; TObjUpdateFunc(a0, a1, a2);  } break;
            case 10: {  HSD_TObj * a0 = pick_HSD_TObj(); HSD_TObjAnim(a0);  } break;
            case 11: {  HSD_TObj * a0 = pick_HSD_TObj(); HSD_TObjAnimAll(a0);  } break;
            case 12: {  HSD_TObj * a0 = pick_HSD_TObj(); HSD_TObjDesc * a1 = pick_HSD_TObjDesc(); ret = (u32) TObjLoad(a0, a1);  } break;
            case 13: {  HSD_TObjDesc * a0 = pick_HSD_TObjDesc(); ret = PI(HSD_TObjLoadDesc(a0));  } break;
            case 14: {  HSD_TlutDesc * a0 = pick_HSD_TlutDesc(); ret = PI(HSD_TlutLoadDesc(a0));  } break;
            case 15: {  HSD_TObjTevDesc * a0 = pick_HSD_TObjTevDesc(); ret = PI(HSD_TObjTevLoadDesc(a0));  } break;
            case 16: {  HSD_TObj * a0 = pick_HSD_TObj(); u32 a1 = fv(); ret = PI(_HSD_TObjGetCurrentByType(a0, a1));  } break;
            case 17: {  HSD_TObj * a0 = pick_HSD_TObj(); MakeTextureMtx(a0);  } break;
            case 18: {  HSD_TObj * a0 = pick_HSD_TObj(); TObjSetupMtx(a0);  } break;
            case 19: {  HSD_TObj * a0 = pick_HSD_TObj(); setupTextureCoordGen(a0);  } break;
            case 20: {  HSD_TObj * a0 = pick_HSD_TObj(); setupTextureCoordGenBump(a0);  } break;
            case 21: {  HSD_TObj * a0 = pick_HSD_TObj(); setupTextureCoordGenToon(a0);  } break;
            case 22: {  HSD_TObj * a0 = pick_HSD_TObj(); HSD_TObjSetupTextureCoordGen(a0);  } break;
            case 23: {  HSD_TObj * a0 = pick_HSD_TObj(); TObjSetupTevModulateShadow(a0);  } break;
            case 24: {  HSD_TObj * a0 = pick_HSD_TObj(); SetupEmbossBumpTev(a0);  } break;
            case 25: {  HSD_TObj * a0 = pick_HSD_TObj(); u32 a1 = fv(); HSD_TObjSetupVolatileTev(a0, a1);  } break;
            case 26: {  u32 a0 = fv(); HSD_TObj * a1 = &TOB[fv() % 2]; HSD_TExp ** a2 = (void*) &PPA[fv() % 4]; HSD_TExp ** a3 = (void*) &PPA[fv() % 4]; HSD_TExp ** a4 = (void*) &PPA[fv() % 4]; int a5 = fv(); MakeColorGenTExp(a0, a1, a2, a3, a4, a5);  } break;
            case 27: {  HSD_TObj * a0 = pick_HSD_TObj(); u32 a1 = fv(); u32 a2 = fv(); HSD_TExp ** a3 = (void*) &PPA[fv() % 4]; HSD_TExp ** a4 = (void*) &PPA[fv() % 4]; HSD_TExp ** a5 = (void*) &PPA[fv() % 4]; TObjMakeTExp(a0, a1, a2, a3, a4, a5);  } break;
            case 28: {  HSD_TObj * a0 = pick_HSD_TObj(); ret = (u32) HSD_TObjAssignResources(a0);  } break;
            case 29: {  HSD_Tlut * a0 = pick_HSD_Tlut(); HSD_Tlut * a1 = pick_HSD_Tlut(); ret = (u32) DifferentTluts(a0, a1);  } break;
            case 30: {  HSD_TObj * a0 = pick_HSD_TObj(); HSD_TObjSetup(a0);  } break;
            case 31: {  u32 a0 = fv() % 12; ret = (u32) HSD_Index2TexMtx(a0);  } break;
            case 32: {  HSD_TObj * a0 = pick_HSD_TObj(); HSD_TObjRemove(a0);  } break;
            case 33: {  HSD_TObj * a0 = pick_HSD_TObj(); HSD_TObjRemoveAll(a0);  } break;
            case 34: {  HSD_TObj * a0 = pick_HSD_TObj(); ret = PI(HSD_TObjGetNext(a0));  } break;
            case 35: {  HSD_TObjInfo * a0 = pick_HSD_TObjInfo(); HSD_TObjSetDefaultClass(a0);  } break;
            case 36: {   ret = PI(HSD_TObjGetDefaultClass());  } break;
            case 37: {   ret = PI(HSD_TObjAlloc());  } break;
            case 38: {  HSD_TObj * a0 = pick_HSD_TObj(); HSD_TObjFree(a0);  } break;
            case 39: {   ret = PI(HSD_TlutAlloc());  } break;
            case 40: {  HSD_Tlut * a0 = pick_HSD_Tlut(); HSD_TlutFree(a0);  } break;
            case 41: {  HSD_Tlut * a0 = pick_HSD_Tlut(); HSD_TlutRemove(a0);  } break;
            case 42: {   ret = PI(HSD_TObjTevAlloc());  } break;
            case 43: {  HSD_TObjTev * a0 = pick_HSD_TObjTev(); HSD_TObjTevFree(a0);  } break;
            case 44: {  HSD_TObjTev * a0 = pick_HSD_TObjTev(); HSD_TObjTevRemove(a0);  } break;
            case 45: {   ret = PI(HSD_ImageDescAlloc());  } break;
            case 46: {  HSD_ImageDesc * a0 = pick_HSD_ImageDesc(); HSD_ImageDescFree(a0);  } break;
            case 47: {  HSD_ImageDesc * a0 = pick_HSD_ImageDesc(); u16 a1 = fv(); u16 a2 = fv(); GXBool a3 = fv(); bool a4 = fv(); HSD_ImageDescCopyFromEFB(a0, a1, a2, a3, a4);  } break;
            case 48: {  HSD_Class * a0 = (HSD_Class*) pick_HSD_TObj(); TObjRelease(a0);  } break;
            case 49: {  HSD_ClassInfo * a0 = pick_HSD_ClassInfo(); TObjAmnesia(a0);  } break;
            case 50: {   TObjInfoInit();  } break;
            }
            ARMED = 0;
            hash_world();

            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
