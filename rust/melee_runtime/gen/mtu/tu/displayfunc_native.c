extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/sysdolphin/baselib/displayfunc.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static HSD_JObj J[4];
static HSD_JObjInfo JI[3];
static HSD_DObj DO[3];
static HSD_DObjInfo DI[2];
static HSD_CObj CO[2];
static HSD_SList SL[3];
static HSD_ZList ZL[4];
static HSD_ClassInfo CI[2];
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
static HSD_JObjInfo* pick_HSD_JObjInfo(void) { u32 r = rnd(); return &JI[(r >> 8) % 3]; }
static HSD_JObjInfo* picko_HSD_JObjInfo(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &JI[(r >> 8) % 3]; }
static HSD_DObj* pick_HSD_DObj(void) { u32 r = rnd(); return &DO[(r >> 8) % 3]; }
static HSD_DObj* picko_HSD_DObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &DO[(r >> 8) % 3]; }
static HSD_DObjInfo* pick_HSD_DObjInfo(void) { u32 r = rnd(); return &DI[(r >> 8) % 2]; }
static HSD_DObjInfo* picko_HSD_DObjInfo(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &DI[(r >> 8) % 2]; }
static HSD_CObj* pick_HSD_CObj(void) { u32 r = rnd(); return &CO[(r >> 8) % 2]; }
static HSD_CObj* picko_HSD_CObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &CO[(r >> 8) % 2]; }
static HSD_SList* pick_HSD_SList(void) { u32 r = rnd(); return &SL[(r >> 8) % 3]; }
static HSD_SList* picko_HSD_SList(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &SL[(r >> 8) % 3]; }
static HSD_ZList* pick_HSD_ZList(void) { u32 r = rnd(); return &ZL[(r >> 8) % 4]; }
static HSD_ZList* picko_HSD_ZList(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &ZL[(r >> 8) % 4]; }
static HSD_ClassInfo* pick_HSD_ClassInfo(void) { u32 r = rnd(); return &CI[(r >> 8) % 2]; }
static HSD_ClassInfo* picko_HSD_ClassInfo(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &CI[(r >> 8) % 2]; }
struct HSD_GObj; typedef struct HSD_GObj HSD_GObj;
static void cbev1(HSD_GObj* g) { T(7001); T(PI(g)); }
static void cbev2(HSD_GObj* g) { T(7002); T(PI(g)); }
static int cbp1(HSD_GObj* g) { T(7003); T(PI(g)); return (int) (fv() & 1); }
static int cbp2(HSD_GObj* g) { T(7004); T(PI(g)); return (int) (fv() & 1); }
static void cbev3(HSD_GObj* g, HSD_GObj* h) { T(7005); T(PI(g)); T(PI(h)); }
static void cbi1(int a) { T(7006); T((u32) a); }
static void cbi2(int a) { T(7007); T((u32) a); }
static void autocb_0(int a0, int a1, int a2, struct HSD_JObj * a3) { T(9000); T((u32) a0); T((u32) a1); T((u32) a2); T(PI((const void*) a3)); }
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

static unsigned char MT[4][48];
static void cb_pmtx(HSD_JObj* j, Mtx v, Mtx m) { T(8210); T(PI(j)); T(PI(v)); T(PI(m)); }
static void cb_jdisp(HSD_JObj* j, Mtx v, Mtx m, u32 trsp, u32 r) { T(8211); T(PI(j)); T(PI(v)); T(PI(m)); T(trsp); T(r); }
static void cb_ddisp(HSD_DObj* d, Mtx v, Mtx m, u32 r) { T(8212); T(PI(d)); T(PI(v)); T(PI(m)); T(r); }
static void cb_sp(s32 a, s32 b, s32 c, HSD_JObj* j) { T(8213); T((u32) a); T((u32) b); T((u32) c); T(PI(j)); }
static unsigned ZK;

#include "displayfunc_native_gen.h"



static void preg_world(void)
{
    preg(J, sizeof J);
    preg(JI, sizeof JI);
    preg(DO, sizeof DO);
    preg(DI, sizeof DI);
    preg(CO, sizeof CO);
    preg(SL, sizeof SL);
    preg(ZL, sizeof ZL);
    preg(CI, sizeof CI);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS); preg(PPA, sizeof PPA);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(J, sizeof J);
    zero(JI, sizeof JI);
    zero(DO, sizeof DO);
    zero(DI, sizeof DI);
    zero(CO, sizeof CO);
    zero(SL, sizeof SL);
    zero(ZL, sizeof ZL);
    zero(CI, sizeof CI);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    for (i = 0; i < 4; i++) PPA[i] = 0;
    for (i = 0; i < 4; i++) fill_HSD_JObj(&J[i]);
    for (i = 0; i < 3; i++) fill_HSD_JObjInfo(&JI[i]);
    for (i = 0; i < 3; i++) fill_HSD_DObj(&DO[i]);
    for (i = 0; i < 2; i++) fill_HSD_DObjInfo(&DI[i]);
    for (i = 0; i < 2; i++) fill_HSD_CObj(&CO[i]);
    for (i = 0; i < 3; i++) fill_HSD_SList(&SL[i]);
    for (i = 0; i < 4; i++) fill_HSD_ZList(&ZL[i]);
    for (i = 0; i < 2; i++) fill_HSD_ClassInfo(&CI[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 4; i++) hash_HSD_JObj(&J[i]);
    for (i = 0; i < 3; i++) hash_HSD_JObjInfo(&JI[i]);
    for (i = 0; i < 3; i++) hash_HSD_DObj(&DO[i]);
    for (i = 0; i < 2; i++) hash_HSD_DObjInfo(&DI[i]);
    for (i = 0; i < 2; i++) hash_HSD_CObj(&CO[i]);
    for (i = 0; i < 3; i++) hash_HSD_SList(&SL[i]);
    for (i = 0; i < 4; i++) hash_HSD_ZList(&ZL[i]);
    for (i = 0; i < 2; i++) hash_HSD_ClassInfo(&CI[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    for (i = 0; i < 4; i++) HM(PI(PPA[i]));
    hashg_all();
    hasho_all();
}

enum { OP_HSD_ZListInitAllocData = 0, OP_HSD_StateInitDirect = 1, OP_HSD_JObjMakePositionMtx = 2, OP_HSD_JObjFindSkeleton = 3, OP__HSD_mkEnvelopeModelNodeMtx = 4, OP_HSD_JObjDispSub = 5, OP_HSD_JObjDispDObj = 6, OP__HSD_ZListSort = 7, OP__HSD_ZListDisp = 8, OP__HSD_ZListClear = 9, OP_HSD_JObjDisp = 10, OP_HSD_JObjSetSPtclCallback = 11, OP_HSD_SetEraseColor = 12, OP_HSD_EraseRect = 13, OP__HSD_DispForgetMemory = 14 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();
preg(MT, sizeof MT); freg((const void*) cb_pmtx); freg((const void*) cb_jdisp); freg((const void*) cb_ddisp); freg((const void*) cb_sp);
    for (op = 0; op < 15; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; break;  /* HSD_ZListInitAllocData */
        case 1: n = 300; break;  /* HSD_StateInitDirect */
        case 2: n = 300; break;  /* HSD_JObjMakePositionMtx */
        case 3: n = 300; break;  /* HSD_JObjFindSkeleton */
        case 4: n = 300; break;  /* _HSD_mkEnvelopeModelNodeMtx */
        case 5: n = 300; break;  /* HSD_JObjDispSub */
        case 6: n = 300; break;  /* HSD_JObjDispDObj */
        case 7: n = 300; break;  /* _HSD_ZListSort */
        case 8: n = 300; break;  /* _HSD_ZListDisp */
        case 9: n = 300; break;  /* _HSD_ZListClear */
        case 10: n = 300; break;  /* HSD_JObjDisp */
        case 11: n = 300; break;  /* HSD_JObjSetSPtclCallback */
        case 12: n = 300; break;  /* HSD_SetEraseColor */
        case 13: n = 300; break;  /* HSD_EraseRect */
        case 14: n = 300; break;  /* _HSD_DispForgetMemory */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ int q, n, k, ord[4];  for (q = 0; q < 3; q++) { JI[q].make_pmtx = cb_pmtx; JI[q].disp = cb_jdisp; }  for (q = 0; q < 2; q++) DI[q].disp = cb_ddisp;  for (q = 0; q < 3; q++) SL[q].next = (q < 2 && (fv() & 1)) ? &SL[q + 1] : 0;  for (q = 0; q < 3; q++) SL[q].data = (void*) (unsigned long long) ((fv() & 1) ? (fv() | 0x80000000u) : fv());  for (q = 0; q < 3; q++) { DO[q].next = (q < 2 && (fv() & 1)) ? &DO[q + 1] : 0; DO[q].parent.class_info = (HSD_ClassInfo*) &DI[q & 1]; DO[q].flags = fv() & 0xFF; }  for (q = 0; q < 4; q++) { J[q].object.parent.class_info = (HSD_ClassInfo*) &JI[q % 3]; J[q].parent = (q > 0 && (fv() & 3)) ? &J[fv() % q] : 0; J[q].flags = fv() & 0x1FFFFFF; if (q < 2) J[q].flags &= ~0x4020u;    J[q].envelopemtx = (fv() & 1) ? (MtxPtr) MT[q] : 0;    if (J[q].flags & 0x20) J[q].u.ptcl = (fv() & 1) ? &SL[fv() % 3] : 0; else J[q].u.dobj = (fv() & 1) ? &DO[fv() % 3] : 0; }  for (q = 0; q < 2; q++) { int i; (void) i; }  for (q = 0; q < 4; q++) { ZL[q].jobj = &J[fv() % 4]; ZL[q].vmtx = (fv() & 1) ? (MtxPtr) MT[q] : 0; ZL[q].sort.texedge = 0; ZL[q].sort.xlu = 0; ZL[q].next = 0; }  n = fv() % 5; if (n > 4) n = 4; zlist_texedge_nb = 0; zlist_texedge_top = 0; zlist_xlu_nb = 0; zlist_xlu_top = 0;  for (q = 0; q < 4; q++) ord[q] = q; for (q = 3; q > 0; q--) { k = fv() % (q + 1); { int tq = ord[q]; ord[q] = ord[k]; ord[k] = tq; } }  k = fv() % 5; { HSD_ZList** pp = &zlist_texedge_top; for (q = 0; q < k && q < 4; q++) { *pp = &ZL[ord[q]]; pp = &ZL[ord[q]].sort.texedge; zlist_texedge_nb++; } }  for (q = 3; q > 0; q--) { k = fv() % (q + 1); { int tq = ord[q]; ord[q] = ord[k]; ord[k] = tq; } }  k = fv() % 5; { HSD_ZList** pp = &zlist_xlu_top; for (q = 0; q < k && q < 4; q++) { *pp = &ZL[ord[q]]; pp = &ZL[ord[q]].sort.xlu; zlist_xlu_nb++; } }  zlist_top = (fv() & 1) ? &ZL[0] : 0; for (q = 0; q < 3; q++) ZL[q].next = (zlist_top && (fv() & 1)) ? &ZL[q + 1] : 0;  zlist_bottom = (fv() & 1) ? &zlist_top : &ZL[fv() % 4].next; zlist_texedge_bottom = (fv() & 1) ? &zlist_texedge_top : &ZL[fv() % 4].sort.texedge; zlist_xlu_bottom = (fv() & 1) ? &zlist_xlu_top : &ZL[fv() % 4].sort.xlu;  sptcl_callback = (fv() & 1) ? cb_sp : 0;  for (q = 0; q < 4; q++) { int i; for (i = 0; i < 48; i++) MT[q][i] = (unsigned char) rnd(); } }
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            ARMED = 1;
            if (MTU_SETJMP()) { ARMED = 0; printf("%d %d ABORT\n", op, k); continue; }
            switch (op) {
            case 0: {   HSD_ZListInitAllocData();  } break;
            case 1: {  int a0 = fv(); u32 a1 = fv(); HSD_StateInitDirect(a0, a1);  } break;
            case 2: {  HSD_JObj * a0 = pick_HSD_JObj(); void* a1 = (MtxPtr) MT[rnd() % 4]; void* a2 = (MtxPtr) MT[rnd() % 4]; HSD_JObjMakePositionMtx(a0, a1, a2);  } break;
            case 3: {  HSD_JObj * a0 = pick_HSD_JObj(); ret = PI(HSD_JObjFindSkeleton(a0));  } break;
            case 4: {  HSD_JObj * a0 = pick_HSD_JObj(); MtxPtr a1 = (MtxPtr) MT[rnd() % 4]; ret = PI(_HSD_mkEnvelopeModelNodeMtx(a0, a1));  } break;
            case 5: {  HSD_JObj * a0 = &J[fv() & 1]; MtxPtr a1 = (MtxPtr) MT[rnd() % 4]; MtxPtr a2 = (MtxPtr) MT[rnd() % 4]; HSD_TrspMask a3 = fv(); u32 a4 = fv(); HSD_JObjDispSub(a0, a1, a2, a3, a4);  } break;
            case 6: {  HSD_JObj * a0 = pick_HSD_JObj(); MtxPtr a1 = ((fv() & 3) ? (MtxPtr) MT[rnd() % 4] : 0); HSD_TrspMask a2 = fv(); u32 a3 = fv(); HSD_JObjDispDObj(a0, a1, a2, a3);  } break;
            case 7: {   _HSD_ZListSort();  } break;
            case 8: {   _HSD_ZListDisp();  } break;
            case 9: {   _HSD_ZListClear();  } break;
            case 10: {  HSD_JObj * a0 = pick_HSD_JObj(); MtxPtr a1 = ((fv() & 3) ? (MtxPtr) MT[rnd() % 4] : 0); HSD_TrspMask a2 = fv(); u32 a3 = fv(); HSD_JObjDisp(a0, a1, a2, a3);  } break;
            case 11: {  void* a0 = (void (*)(s32, s32, s32, HSD_JObj *)) autocb_0; HSD_JObjSetSPtclCallback(a0);  } break;
            case 12: {  u8 a0 = fv(); u8 a1 = fv(); u8 a2 = fv(); u8 a3 = fv(); HSD_SetEraseColor(a0, a1, a2, a3);  } break;
            case 13: {  f32 a0 = rw(); f32 a1 = rw(); f32 a2 = rw(); f32 a3 = rw(); f32 a4 = rw(); int a5 = fv(); int a6 = fv(); int a7 = fv(); HSD_EraseRect(a0, a1, a2, a3, a4, a5, a6, a7);  } break;
            case 14: {  void * a0 = (void*) BUF; void * a1 = (void*) BUF; _HSD_DispForgetMemory(a0, a1);  } break;
            }
            ARMED = 0;
            hash_world();
{ int i; for (i = 0; i < 3; i++) HM((u32) (unsigned long long) SL[i].data); }
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
