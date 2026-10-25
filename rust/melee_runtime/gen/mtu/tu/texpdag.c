extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/sysdolphin/baselib/texpdag.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static HSD_TExp TE[6];
static HSD_TExpDag DG[6];
static Vec3 VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static void* PPA[4];
static char STRS[4][64] = { "a.dat", "sym", "LbAd.dat", "x" };

static HSD_TExp* pick_HSD_TExp(void) { u32 r = rnd(); return &TE[(r >> 8) % 6]; }
static HSD_TExp* picko_HSD_TExp(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &TE[(r >> 8) % 6]; }
static HSD_TExpDag* pick_HSD_TExpDag(void) { u32 r = rnd(); return &DG[(r >> 8) % 6]; }
static HSD_TExpDag* picko_HSD_TExpDag(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &DG[(r >> 8) % 6]; }
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

static HSD_TExp* TEP[6];
static int DIST[6];
static int ORD[8];
static int MINO[8];
static int MINV;
static u32 DEP[8];
static u32 FDEP[8];
static u8 pick_type(int q) { u8 t; if (q < 5 && (fv() & 1)) return 1; t = (u8) (fv() % 6); return (q == 5 && t == 1) ? 0 : t; }

#include "texpdag_gen.h"



static void preg_world(void)
{
    preg(TE, sizeof TE);
    preg(DG, sizeof DG);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS); preg(PPA, sizeof PPA);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(TE, sizeof TE);
    zero(DG, sizeof DG);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    for (i = 0; i < 4; i++) PPA[i] = 0;
    for (i = 0; i < 6; i++) fill_HSD_TExp(&TE[i]);
    for (i = 0; i < 6; i++) fill_HSD_TExpDag(&DG[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 6; i++) hash_HSD_TExp(&TE[i]);
    for (i = 0; i < 6; i++) hash_HSD_TExpDag(&DG[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    for (i = 0; i < 4; i++) HM(PI(PPA[i]));
    hashg_all();
    hasho_all();
}

enum { OP_assign_reg = 0, OP_order_dag = 1, OP_CalcDistance = 2, OP_SimplifySrc = 3, OP_SimplifyThis = 4, OP_SimplifyByMerge = 5, OP_HSD_TExpSimplify = 6 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();
preg(TEP, sizeof TEP); preg(DIST, sizeof DIST); preg(ORD, sizeof ORD); preg(MINO, sizeof MINO); preg(&MINV, sizeof MINV); preg(DEP, sizeof DEP); preg(FDEP, sizeof FDEP);
    for (op = 0; op < 7; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; break;  /* assign_reg */
        case 1: n = 300; break;  /* order_dag */
        case 2: n = 300; break;  /* CalcDistance */
        case 3: n = 300; break;  /* SimplifySrc */
        case 4: n = 300; break;  /* SimplifyThis */
        case 5: n = 300; break;  /* SimplifyByMerge */
        case 6: n = 300; break;  /* HSD_TExpSimplify */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ int q, i; LG = 0;  for (q = 0; q < 6; q++) { TEP[q] = &TE[q]; DIST[q] = (int) (fv() % 4); ORD[q] = (int) (fv() % 6); MINO[q] = 0; DEP[q] = fv() & 0x3F; FDEP[q] = fv() & 0x3F; DG[q].tev = &TE[q].tev; DG[q].idx = (u8) q; DG[q].nb_dep = (u8) (fv() % 3); DG[q].nb_ref = (u8) (fv() % 3); DG[q].dist = 0;    for (i = 0; i < 8; i++) DG[q].depend[i] = &DG[fv() % 6];    TE[q].tev.c_ref = (int) (fv() % 3); TE[q].tev.a_ref = (int) (fv() % 3); TE[q].tev.c_dst = (u8) (fv() % 4); TE[q].tev.a_dst = (u8) (fv() % 4); TE[q].tev.tex = 0; TE[q].tev.chan = (fv() & 1) ? 0xFF : (u8) (fv() % 3);    TE[q].tev.tex_swap = (fv() & 1) ? 0xFF : (u8) (fv() % 2); TE[q].tev.ras_swap = (fv() & 1) ? 0xFF : (u8) (fv() % 2); TE[q].tev.kcsel = (fv() & 1) ? 0xFF : (u8) (fv() % 4); TE[q].tev.kasel = (fv() & 1) ? 0xFF : (u8) (fv() % 4);    TE[q].tev.c_op = (fv() % 5 == 0) ? 0xFF : (u8) (fv() % 2); TE[q].tev.a_op = (fv() % 5 == 0) ? 0xFF : (u8) (fv() % 2); TE[q].tev.c_bias = (u8) (fv() % 3); TE[q].tev.c_scale = (u8) (fv() % 3); TE[q].tev.a_bias = (u8) (fv() % 3); TE[q].tev.a_scale = (u8) (fv() % 3);    TE[q].tev.c_clamp = (u8) (fv() & 1); TE[q].tev.a_clamp = (u8) (fv() & 1);    for (i = 0; i < 4; i++) {      TE[q].tev.c_in[i].type = pick_type(q); TE[q].tev.c_in[i].sel = (u8) (fv() % 3); TE[q].tev.c_in[i].exp = &TE[q < 5 ? q + 1 + fv() % (5 - q) : q];      TE[q].tev.a_in[i].type = pick_type(q); TE[q].tev.a_in[i].sel = (u8) (fv() % 3); TE[q].tev.a_in[i].exp = &TE[q < 5 ? q + 1 + fv() % (5 - q) : q]; } }  MINV = 1000; }
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            ARMED = 1;
            if (MTU_SETJMP()) { ARMED = 0; printf("%d %d ABORT\n", op, k); continue; }
            switch (op) {
            case 0: {  int a0 = (int) (1 + fv() % 6); u32 * a1 = (u32*) DEP; HSD_TExpDag * a2 = DG; const int * a3 = ORD; ret = (u32) assign_reg(a0, a1, a2, a3);  } break;
            case 1: {  int a0 = (int) (1 + fv() % 5); u32 * a1 = DEP; u32 * a2 = FDEP; HSD_TExpDag * a3 = DG; int a4 = (int) 0; int a5 = (int) (fv() % 5); u32 a6 = fv(); u32 a7 = fv(); int * a8 = ORD; int * a9 = &MINV; int * a10 = MINO; order_dag(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10);  } break;
            case 2: {  HSD_TExp ** a0 = TEP; int * a1 = DIST; HSD_TExp * a2 = &TE[fv() % 6]; int a3 = (int) (1 + fv() % 6); int a4 = (int) (fv() % 3); CalcDistance(a0, a1, a2, a3, a4);  } break;
            case 3: {  HSD_TExp * a0 = &TE[fv() % 5]; ret = (u32) SimplifySrc(a0);  } break;
            case 4: {  HSD_TExp * a0 = &TE[fv() % 5]; ret = (u32) SimplifyThis(a0);  } break;
            case 5: {  HSD_TExp * a0 = &TE[fv() % 5]; ret = (u32) SimplifyByMerge(a0);  } break;
            case 6: {  HSD_TExp * a0 = &TE[fv() % 5]; ret = (u32) HSD_TExpSimplify(a0);  } break;
            }
            ARMED = 0;
            hash_world();
{ int i; for (i = 0; i < 6; i++) { HM((u32) DIST[i]); } for (i = 0; i < 8; i++) { HM((u32) ORD[i]); HM((u32) MINO[i]); } HM((u32) MINV); }
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
