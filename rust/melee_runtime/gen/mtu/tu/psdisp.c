extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/sysdolphin/baselib/psdisp.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static HSD_Particle P0[4];
static HSD_Generator GEN[2];
static GXColor P1[2];
static HSD_psAppSRT SRT[2];
static Vec3 VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static void* PPA[4];
static char STRS[4][64] = { "a.dat", "sym", "LbAd.dat", "x" };

static HSD_Particle* pick_HSD_Particle(void) { u32 r = rnd(); return &P0[(r >> 8) % 4]; }
static HSD_Particle* picko_HSD_Particle(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &P0[(r >> 8) % 4]; }
static HSD_Generator* pick_HSD_Generator(void) { u32 r = rnd(); return &GEN[(r >> 8) % 2]; }
static HSD_Generator* picko_HSD_Generator(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &GEN[(r >> 8) % 2]; }
static GXColor* pick_GXColor(void) { u32 r = rnd(); return &P1[(r >> 8) % 2]; }
static GXColor* picko_GXColor(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &P1[(r >> 8) % 2]; }
static HSD_psAppSRT* pick_HSD_psAppSRT(void) { u32 r = rnd(); return &SRT[(r >> 8) % 2]; }
static HSD_psAppSRT* picko_HSD_psAppSRT(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &SRT[(r >> 8) % 2]; }
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

#include "psdisp_gen.h"



static void preg_world(void)
{
    preg(P0, sizeof P0);
    preg(GEN, sizeof GEN);
    preg(P1, sizeof P1);
    preg(SRT, sizeof SRT);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS); preg(PPA, sizeof PPA);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(P0, sizeof P0);
    zero(GEN, sizeof GEN);
    zero(P1, sizeof P1);
    zero(SRT, sizeof SRT);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    for (i = 0; i < 4; i++) PPA[i] = 0;
    for (i = 0; i < 4; i++) fill_HSD_Particle(&P0[i]);
    for (i = 0; i < 2; i++) fill_HSD_Generator(&GEN[i]);
    for (i = 0; i < 2; i++) fill_GXColor(&P1[i]);
    for (i = 0; i < 2; i++) fill_HSD_psAppSRT(&SRT[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 4; i++) hash_HSD_Particle(&P0[i]);
    for (i = 0; i < 2; i++) hash_HSD_Generator(&GEN[i]);
    for (i = 0; i < 2; i++) hash_GXColor(&P1[i]);
    for (i = 0; i < 2; i++) hash_HSD_psAppSRT(&SRT[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    for (i = 0; i < 4; i++) HM(PI(PPA[i]));
    hashg_all();
    hasho_all();
}

enum { OP_calcTornadoLastPos = 0, OP_getColorPrimEnv = 1, OP_setupChanCtrl = 2, OP_setupChanReg = 3, OP_setupTevReg = 4, OP_particleSort = 5 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();

    for (op = 0; op < 6; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 400; break;  /* calcTornadoLastPos */
        case 1: n = 400; break;  /* getColorPrimEnv */
        case 2: n = 400; break;  /* setupChanCtrl */
        case 3: n = 400; break;  /* setupChanReg */
        case 4: n = 400; break;  /* setupTevReg */
        case 5: n = 400; break;  /* particleSort */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ int q;  for (q = 0; q < 4; q++) { P0[q].next = (q < 3 && (fv() & 1)) ? &P0[q + 1] : 0; P0[q].gen = &GEN[fv() & 1]; P0[q].appsrt = (fv() & 1) ? &SRT[fv() & 1] : 0;    P0[q].kind = fv() ^ (fv() << 13) ^ (fv() << 26); if (fv() & 1) { P0[q].primColCount = (u16) (1 + fv() % 100); P0[q].primColRemain = (u16) (fv() % 101); } else P0[q].primColCount = 0;    if (fv() & 1) { P0[q].envColCount = (u16) (1 + fv() % 100); P0[q].envColRemain = (u16) (fv() % 101); } else P0[q].envColCount = 0;    if (fv() & 1) { P0[q].matColCount = (u16) (1 + fv() % 100); P0[q].matColRemain = (u16) (fv() % 101); } else P0[q].matColCount = 0;    if (fv() & 1) { P0[q].ambColCount = (u16) (1 + fv() % 100); P0[q].ambColRemain = (u16) (fv() % 101); } else P0[q].ambColCount = 0; }  for (q = 0; q < 16; q++) hsd_804D0908[q] = (fv() & 1) ? &P0[fv() & 3] : 0; for (q = 0; q < 17; q++) particle_list[q] = (fv() & 1) ? &P0[fv() & 3] : 0; }
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            ARMED = 1;
            if (MTU_SETJMP()) { ARMED = 0; printf("%d %d ABORT\n", op, k); continue; }
            switch (op) {
            case 0: {  HSD_Particle * a0 = pick_HSD_Particle(); f32 * a1 = &FA[rnd() % 4]; f32 * a2 = &FA[rnd() % 4]; f32 * a3 = &FA[rnd() % 4]; calcTornadoLastPos(a0, a1, a2, a3);  } break;
            case 1: {  HSD_Particle * a0 = pick_HSD_Particle(); GXColor * a1 = pick_GXColor(); GXColor * a2 = pick_GXColor(); getColorPrimEnv(a0, a1, a2);  } break;
            case 2: {  HSD_Particle * a0 = pick_HSD_Particle(); setupChanCtrl(a0);  } break;
            case 3: {  HSD_Particle * a0 = pick_HSD_Particle(); setupChanReg(a0);  } break;
            case 4: {  HSD_Particle * a0 = pick_HSD_Particle(); setupTevReg(a0);  } break;
            case 5: {  s32 a0 = (s32) (fv() & 15); u8 a1 = fv(); HSD_Particle ** a2 = (HSD_Particle**) &PPA[0]; HSD_Particle ** a3 = (HSD_Particle**) &PPA[1]; ret = PI(particleSort(a0, a1, a2, a3));  } break;
            }
            ARMED = 0;
            hash_world();

            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
