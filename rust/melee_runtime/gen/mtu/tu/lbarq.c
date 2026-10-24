extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/melee/lb/lbarq.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */
#include "types.h"

#include TU_SRC
#include "mtu_common.h"
static Vec3 VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static char STRS[4][64] = { "a.dat", "sym", "LbAd.dat", "x" };

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
static void freg_world(void) { freg((const void*) cbev1); freg((const void*) cbev2); freg((const void*) cbp1); freg((const void*) cbp2); freg((const void*) cbev3); freg((const void*) cbi1); freg((const void*) cbi2); }

static ARQRequest* PEND;
static void cbv1(void* a) { T(7101); T(PI(a)); }
static void cbv2(void* a) { T(7102); T(PI(a)); }

#include "lbarq_gen.h"



static void preg_world(void)
{
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS);
}

static void setup_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    hashg_all();
    hasho_all();
}

enum { OP_lbArq_80014ABC = 0, OP_lbArq_80014AC4 = 1, OP_lbArq_80014BD0 = 2, OP_lbArq_80014D2C = 3 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();
freg((const void*) cbv1); freg((const void*) cbv2);
    for (op = 0; op < 4; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; break;  /* lbArq_80014ABC */
        case 1: n = 300; break;  /* lbArq_80014AC4 */
        case 2: n = 300; break;  /* lbArq_80014BD0 */
        case 3: n = 300; break;  /* lbArq_80014D2C */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ int q; lbArqNode* last[3] = { 0, 0, 0 }; PEND = 0;
  lbArq_804316C0.list[0] = lbArq_804316C0.list[1] = lbArq_804316C0.list[2] = 0;
  for (q = 0; q < 10; q++) { lbArqNode* n = &lbArq_804316C0.nodes[q]; u32 s = q == 0 ? 0 : fv() % 3;
    n->state = s; n->next = 0; if (last[s]) last[s]->next = n; else lbArq_804316C0.list[s] = n; last[s] = n;
    n->callback = (fv() & 1) ? ((fv() & 1) ? cbv1 : cbv2) : 0; n->callback_arg = (fv() & 1) ? BUF + 8 * (fv() % 8) : 0; } }
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            switch (op) {
            case 0: {  lbArqNode * a0 = &lbArq_804316C0.nodes[fv() % 10]; ret = (u32) lbArq_80014ABC(a0);  } break;
            case 1: {  lbArqHandle * a0 = (lbArqHandle*) &lbArq_804316C0.nodes[fv() % 10].arq; lbArq_80014AC4(a0);  } break;
            case 2: {  unsigned int a0 = fv(); void * a1 = (void*) BUF; gc_size_t a2 = fv(); lbArqCallback a3 = (lbArqCallback) ((fv() & 1) ? ((fv() & 1) ? (void*) cbv1 : (void*) cbv2) : 0); void * a4 = (void*) BUF; lbArq_80014BD0(a0, a1, a2, a3, a4);  } break;
            case 3: {   lbArq_80014D2C();  } break;
            }
            hash_world();

            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
