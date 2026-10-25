extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/sysdolphin/baselib/archive.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static HSD_Archive ARC[1];
static struct { float x, y, z; } VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static char STRS[4][64] = { "a.dat", "sym", "LbAd.dat", "x" };

static HSD_Archive* pick_HSD_Archive(void) { u32 r = rnd(); return &ARC[(r >> 8) % 1]; }
static HSD_Archive* picko_HSD_Archive(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &ARC[(r >> 8) % 1]; }
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
static void freg_world(void) { freg((const void*) cbev1); freg((const void*) cbev2); freg((const void*) cbp1); freg((const void*) cbp2); freg((const void*) cbev3); freg((const void*) cbi1); freg((const void*) cbi2); }
static unsigned char AB[1024] __attribute__((aligned(8))); static u32 TOT; static const char SYMS[4][8] = { "ab", "abc", "x", "sym" };
#include "archive_gen.h"



static void preg_world(void)
{
    preg(ARC, sizeof ARC);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(ARC, sizeof ARC);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    for (i = 0; i < 1; i++) fill_HSD_Archive(&ARC[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 1; i++) hash_HSD_Archive(&ARC[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    hashg_all();
    hasho_all();
}

enum { OP_HSD_ArchiveParse = 0, OP_HSD_ArchiveParse_profile_body = 1, OP_HSD_ArchiveGetPublicAddress = 2, OP_HSD_ArchiveGetExtern = 3, OP_HSD_ArchiveLocateExtern = 4 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();
preg(AB, sizeof AB);
    for (op = 0; op < 5; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 400; break;  /* HSD_ArchiveParse */
        case 1: n = 400; break;  /* HSD_ArchiveParse_profile_body */
        case 2: n = 400; break;  /* HSD_ArchiveGetPublicAddress */
        case 3: n = 400; break;  /* HSD_ArchiveGetExtern */
        case 4: n = 400; break;  /* HSD_ArchiveLocateExtern */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ u32* w = (u32*) AB; int i, D, R, P, E, pw, ew, rw, sb; static const unsigned char SY[24] = { 'a', 'b', 0, 'a', 'b', 'c', 0, 'x', 0, 's', 'y', 'm', 0 }; static const int SO[4] = { 0, 3, 7, 9 }; for (i = 0; i < 1024; i++) AB[i] = 0; D = 4 * (2 + fv() % 6); R = fv() % 3; P = fv() % 3; E = fv() % 3; rw = (0x20 + D) / 4; pw = rw + R; ew = pw + 2 * P; sb = 0x20 + D + 4 * R + 8 * P + 8 * E; TOT = sb + 24; w[0] = TOT; w[1] = D; w[2] = R; w[3] = P; w[4] = E; for (i = 0; i < D / 4; i++) { u32 c = fv(); w[8 + i] = (c & 3) ? 4 * (fv() % (D / 4)) : 0xFFFFFFFFu; } { int x = fv() % (D / 4); for (i = 0; i < R; i++) w[rw + i] = 4 * ((x + i) % (D / 4)); } for (i = 0; i < P; i++) { w[pw + 2 * i] = 4 * (fv() % (D / 4)); w[pw + 2 * i + 1] = SO[fv() % 4]; } for (i = 0; i < E; i++) { w[ew + 2 * i] = 4 * (fv() % (D / 4)); w[ew + 2 * i + 1] = SO[fv() % 4]; } for (i = 0; i < 24; i++) AB[sb + i] = SY[i]; if (op != OP_HSD_ArchiveParse && op != OP_HSD_ArchiveParse_profile_body) HSD_ArchiveParse(&ARC[0], AB, TOT); }
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            ARMED = 1;
            if (MTU_SETJMP()) { ARMED = 0; printf("%d %d ABORT\n", op, k); continue; }
            switch (op) {
            case 0: {  HSD_Archive * a0 = pick_HSD_Archive(); u8 * a1 = AB; gc_size_t a2 = (size_t) ((fv() & 7) ? TOT : TOT + 1); ret = (u32) HSD_ArchiveParse(a0, a1, a2);  } break;
            case 1: {  HSD_Archive * a0 = pick_HSD_Archive(); u8 * a1 = AB; gc_size_t a2 = (size_t) ((fv() & 7) ? TOT : TOT + 1); ret = (u32) HSD_ArchiveParse_profile_body(a0, a1, a2);  } break;
            case 2: {  HSD_Archive * a0 = pick_HSD_Archive(); const char * a1 = SYMS[fv() % 4]; ret = PI(HSD_ArchiveGetPublicAddress(a0, a1));  } break;
            case 3: {  HSD_Archive * a0 = pick_HSD_Archive(); int a1 = (int) (fv() % 5) - 1; ret = PI(HSD_ArchiveGetExtern(a0, a1));  } break;
            case 4: {  HSD_Archive * a0 = pick_HSD_Archive(); const char * a1 = SYMS[fv() % 4]; void * a2 = (void*) (AB + 4 * (fv() % 64)); HSD_ArchiveLocateExtern(a0, a1, a2);  } break;
            }
            ARMED = 0;
            hash_world();
{ int i; u32 bs = (u32) (unsigned long long) AB; for (i = 0; i < 1024; i += 4) { u32 v = *(u32*) (AB + i); u32 d = v - bs; HM((d < 4096u || d > 0xFFFFFF00u) ? (0xF0000000u ^ d) : v); } }
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
