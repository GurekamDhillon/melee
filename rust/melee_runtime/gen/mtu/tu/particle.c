extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/sysdolphin/baselib/particle.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static HSD_Particle P0[4];
static HSD_Generator GEN[2];
static HSD_psAppSRT SRT[2];
static struct _PSUserFunc UF[2];
static HSD_JObj J[2];
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
static HSD_psAppSRT* pick_HSD_psAppSRT(void) { u32 r = rnd(); return &SRT[(r >> 8) % 2]; }
static HSD_psAppSRT* picko_HSD_psAppSRT(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &SRT[(r >> 8) % 2]; }
static struct _PSUserFunc* pick__PSUserFunc(void) { u32 r = rnd(); return &UF[(r >> 8) % 2]; }
static struct _PSUserFunc* picko__PSUserFunc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &UF[(r >> 8) % 2]; }
static HSD_JObj* pick_HSD_JObj(void) { u32 r = rnd(); return &J[(r >> 8) % 2]; }
static HSD_JObj* picko_HSD_JObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &J[(r >> 8) % 2]; }
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

static int cb_hook(HSD_Particle* p) { T(8401); T(PI(p)); return (int) (fv() % 3) - 1; }
static int cb_part(HSD_Particle* p) { T(8402); T(PI(p)); return (fv() & 1) ? -1 : 0; }

#include "particle_gen.h"



static void preg_world(void)
{
    preg(P0, sizeof P0);
    preg(GEN, sizeof GEN);
    preg(SRT, sizeof SRT);
    preg(UF, sizeof UF);
    preg(J, sizeof J);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS); preg(PPA, sizeof PPA);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(P0, sizeof P0);
    zero(GEN, sizeof GEN);
    zero(SRT, sizeof SRT);
    zero(UF, sizeof UF);
    zero(J, sizeof J);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    for (i = 0; i < 4; i++) PPA[i] = 0;
    for (i = 0; i < 4; i++) fill_HSD_Particle(&P0[i]);
    for (i = 0; i < 2; i++) fill_HSD_Generator(&GEN[i]);
    for (i = 0; i < 2; i++) fill_HSD_psAppSRT(&SRT[i]);
    for (i = 0; i < 2; i++) fill__PSUserFunc(&UF[i]);
    for (i = 0; i < 2; i++) fill_HSD_JObj(&J[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 4; i++) hash_HSD_Particle(&P0[i]);
    for (i = 0; i < 2; i++) hash_HSD_Generator(&GEN[i]);
    for (i = 0; i < 2; i++) hash_HSD_psAppSRT(&SRT[i]);
    for (i = 0; i < 2; i++) hash__PSUserFunc(&UF[i]);
    for (i = 0; i < 2; i++) hash_HSD_JObj(&J[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    for (i = 0; i < 4; i++) HM(PI(PPA[i]));
    hashg_all();
    hasho_all();
}

enum { OP_psGenerateParticle0 = 0, OP_hsd_80398F0C = 1, OP_hsd_80398F8C = 2, OP_hsd_8039930C = 3, OP_hsd_8039D0A0 = 4 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();
freg((const void*) cb_hook); freg((const void*) cb_part);
    for (op = 0; op < 5; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 400; break;  /* psGenerateParticle0 */
        case 1: n = 400; break;  /* hsd_80398F0C */
        case 2: n = 400; break;  /* hsd_80398F8C */
        case 3: n = 400; break;  /* hsd_8039930C */
        case 4: n = 400; break;  /* hsd_8039D0A0 */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ int q;  for (q = 0; q < 2; q++) { UF[q].hookCreate = (fv() & 1) ? cb_hook : 0; UF[q].hookDelete = (fv() & 1) ? cb_hook : 0; }  for (q = 0; q < 2; q++) { GEN[q].userfunc = (fv() & 1) ? &UF[fv() & 1] : 0; GEN[q].appsrt = (fv() & 1) ? &SRT[fv() & 1] : 0; GEN[q].linkNo = (u8) (fv() % 16); GEN[q].idnum = (u16) (fv() % 3); GEN[q].numChild = fv() % 8; }  for (q = 0; q < 4; q++) { P0[q].next = (q < 3 && (fv() & 1)) ? &P0[q + 1] : 0; P0[q].gen = &GEN[fv() & 1]; P0[q].appsrt = (fv() & 1) ? &SRT[fv() & 1] : 0; P0[q].idnum = (u16) (fv() % 3); P0[q].linkNo = (u8) (fv() % 16);    P0[q].cmdList = 0; P0[q].cmdWait = 0; P0[q].life = (u16) (1 + fv() % 3); P0[q].kind = (fv() ^ (fv() << 9)) & ~0x8800u; P0[q].callback = (fv() & 1) ? cb_part : 0;    if (P0[q].sizeCount > 100) P0[q].sizeCount = 0; if (P0[q].rotateCount > 100) P0[q].rotateCount = 0; }  for (q = 0; q < 16; q++) hsd_804D0908[q] = (fv() & 1) ? &P0[fv() & 3] : 0; for (q = 0; q < 8; q++) hsd_804D08E8[q] = (fv() & 1) ? &J[fv() & 1] : 0; }
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            ARMED = 1;
            if (MTU_SETJMP()) { ARMED = 0; printf("%d %d ABORT\n", op, k); continue; }
            switch (op) {
            case 0: {  HSD_Particle ** a0 = (fv() & 1) ? (HSD_Particle**) &PPA[0] : 0; int a1 = (int) (fv() % 16); int a2 = (int) (fv() % 4); u32 a3 = fv() & ~0x8804u; u16 a4 = fv(); u8 * a5 = (u8*) 0; int a6 = fv(); int a7 = fv(); f32 a8 = rw(); f32 a9 = rw(); f32 a10 = rw(); f32 a11 = rw(); f32 a12 = rw(); f32 a13 = rw(); f32 a14 = rw(); f32 a15 = rw(); f32 a16 = rw(); HSD_Generator * a17 = picko_HSD_Generator(); int a18 = (int) (fv() & 1); ret = PI(psGenerateParticle0(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18));  } break;
            case 1: {  s32 a0 = (s32) (fv() % 16); s32 a1 = (s32) (fv() % 4); s32 a2 = (s32) (fv() & ~0x8804u); u16 a3 = fv(); s32 a4 = (s32) 0; s32 a5 = fv(); s32 a6 = fv(); s32 a7 = (s32) 0; f32 a8 = rw(); f32 a9 = rw(); f32 a10 = rw(); f32 a11 = rw(); f32 a12 = rw(); f32 a13 = rw(); f32 a14 = rw(); f32 a15 = rw(); f32 a16 = rw(); hsd_80398F0C(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16);  } break;
            case 2: {  HSD_Particle * a0 = pick_HSD_Particle(); f32 a1 = rw(); hsd_80398F8C(a0, a1);  } break;
            case 3: {  HSD_Particle * a0 = &P0[fv() & 3]; HSD_Particle * a1 = (fv() & 1) ? &P0[fv() & 3] : 0; ret = PI(hsd_8039930C(a0, a1));  } break;
            case 4: {  HSD_Generator * a0 = &GEN[fv() & 1]; hsd_8039D0A0(a0);  } break;
            }
            ARMED = 0;
            hash_world();

            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
