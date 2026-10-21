/* C reference for melee_world::mp::geom. Built by gen_mplib_geom_ref.sh, which cuts the real
 * function bodies out of src/melee/mp/mplib.c (mplib_lines_body.inc + mplib_geom_body.inc).
 * Prelude copied from mplib_lines_ref.c. */
#include <setjmp.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define TARGET_PC 1
typedef int16_t s16;
typedef uint16_t u16;
typedef int32_t s32;
typedef uint32_t u32;
typedef float f32;
typedef int bool;
typedef int enum_t;
#define true 1
#define false 0
#define SQ(x) ((x) * (x))
#define PAD_STACK(n)
#define OSReport(...) (printf("ASSERT\n"), exit(3))
#define HSD_ASSERTREPORT(line, c, ...) (printf("ASSERT\n"), exit(3))

typedef struct { f32 x, y; } Vec2;
typedef struct { f32 x, y, z; } Vec3;
typedef Vec3 Vec;
typedef struct MapLine {
    u16 v0_idx, v1_idx;
    s16 prev_id0, next_id0, prev_id1, next_id1;
    u16 hi_flags, lo_flags;
} MapLine;
typedef struct CollLine { MapLine* x0; u32 flags; } CollLine;
typedef struct CollVtx { f32 x0, x4; Vec2 pos; float x10, x14; } CollVtx;
typedef struct MapCollData { int pad[3]; int line_count; } MapCollData;
_Static_assert(sizeof(CollLine) == 8, "build for i686");

enum { CollLine_Floor = 1, CollLine_Ceiling = 2, CollLine_RightWall = 4, CollLine_LeftWall = 8 };
#define LINE_FLAG_KIND (0xF)
#define LINE_FLAG_ENABLED (1 << 16)
#define LINE_FLAG_HIDDEN (1 << 18)
#define LINEID_CHECK(line, line_id)                                         \
    do {                                                                    \
        if ((line_id) == -1 || (line_id) >= mpLib_804D64B4->line_count)     \
            HSD_ASSERTREPORT(line, 0, "", 0);                               \
    } while (0)

static MapCollData* mpLib_804D64B4;
static CollVtx* groundCollVtx;
static CollLine* groundCollLine;

/* PSVECNormalize as in pc/gameworld/mtx_pc.c */
double __frsqrte(double);
static double gekko_force25(double d)
{
    union { double d; unsigned long long u; } v;
    v.d = d;
    v.u = (v.u & 0xFFFFFFFFF8000000ULL) + (v.u & 0x8000000ULL);
    return v.d;
}
static void PSVECNormalize(Vec* vec1, Vec* dst)
{
    f32 x = vec1->x, y = vec1->y, z = vec1->z;
    f32 xx = x * x;
    f32 yy = y * y;
    f32 xz = (f32) ((double) z * (double) z + (double) xx);
    f32 sqsum = xz + yy;
    double est = __frsqrte((double) sqsum);
    f32 n0 = (f32) (est * gekko_force25(est));
    f32 n1 = (f32) (est * 0.5);
    f32 n2 = (f32) (-((double) n0 * (double) sqsum - 3.0));
    f32 rsqrt = n2 * n1;
    dst->x = x * rsqrt;
    dst->y = y * rsqrt;
    dst->z = z * rsqrt;
}

static unsigned char line_active_tab[256];
static long active_budget;
static jmp_buf loop_jmp;
int ScriptGame_StageLineActive(int line)
{
    if (--active_budget < 0) longjmp(loop_jmp, 1);
    return line_active_tab[line];
}

#define ABS(x) ((x) < 0 ? -(x) : (x))
#define HSD_ASSERT(line, c) ((c) ? (void) 0 : (void) (printf("ASSERT\n"), exit(3)))
static int DbLevel = 0;
enum { DbLKind_DebugRom = 3 };
/* sqrtf exactly as src/MSL/math_ppc.h (the shipping build -includes it). */
static float sqrtf(float x)
{
    volatile float y;
    if (x > 0.0f) {
        double guess = __frsqrte((double) x);
        guess = 0.5 * guess * (3.0 - guess * guess * x);
        guess = 0.5 * guess * (3.0 - guess * guess * x);
        guess = 0.5 * guess * (3.0 - guess * guess * x);
        y = (float) (x * guess);
        return y;
    }
    return x;
}

#include "mplib_lines_body.inc"
#include "mplib_geom_body.inc"

static u32 rng = 0x2468ace1u;
static u32 rnd(u32 n)
{
    rng = rng * 1103515245u + 12345u;
    return (rng >> 8) % n;
}
static u32 fbits(f32 f) { u32 u; memcpy(&u, &f, 4); return u; }
/* Mix of grid values (degenerate/collinear cases) and arbitrary floats (FMA-sensitive cases). */
static f32 rf(void)
{
    u32 k = rnd(4);
    if (k == 0) return (f32) ((int) rnd(9) - 4) * 0.5f;
    if (k == 1) return (f32) ((int) rnd(2000001) - 1000000) * 1.0e-5f;
    if (k == 2) return (f32) ((int) rnd(20001) - 10000) * 0.0137f;
    {
        u32 a = rnd(1u << 23), e = rnd(12);
        return (f32) a / 8388608.0f * (f32) (1 << e) - (f32) (1 << e) * 0.5f;
    }
}

#define DOQ(name)                                                                          \
    do {                                                                                   \
        float d = -7.0f;                                                                   \
        u32 fl = 0xdead;                                                                   \
        Vec3 n = { -9, -9, -9 };                                                           \
        int r;                                                                             \
        active_budget = 100000;                                                            \
        if (setjmp(loop_jmp)) {                                                            \
            printf("LOOP %s %d %08x %08x\n", #name, i, fbits(q.x), fbits(q.y));            \
            break;                                                                         \
        }                                                                                  \
        r = name(i, &q, &d, &fl, &n);                                                      \
        printf("Q %s %d %08x %08x %d %08x %u %08x %08x %08x\n", #name, i, fbits(q.x),      \
               fbits(q.y), r, fbits(d), fl, fbits(n.x), fbits(n.y), fbits(n.z));           \
    } while (0)

int main(void)
{
    static CollVtx vtx[64];
    static MapLine map[48];
    static CollLine lines[48];
    static MapCollData mcd;
    int g, i, j, k;
    mpLib_804D64B4 = &mcd;
    groundCollVtx = vtx;
    groundCollLine = lines;
    for (g = 0; g < 32; g++) {
        int nv = 8 + rnd(56), nl = 4 + rnd(44);
        int perm[48];
        mcd.line_count = nl;
        for (i = 0; i < nv; i++) {
            memset(&vtx[i], 0, sizeof vtx[i]);
            vtx[i].pos.x = (f32) (int) rnd(12) * 0.75f + (rnd(4) == 0 ? 0.3f : 0.0f) - 4.0f;
            vtx[i].pos.y = (f32) (int) rnd(12) * 1.25f - (rnd(3) == 0 ? 0.6f : 0.0f);
            if (rnd(2)) { vtx[i].pos.x += rf() * 0.01f; vtx[i].pos.y += rf() * 0.01f; }
            vtx[i].x0 = rf();
            vtx[i].x4 = rf();
            vtx[i].x10 = vtx[i].pos.x + (rnd(3) ? rf() * 0.1f : 0.0f);
            vtx[i].x14 = vtx[i].pos.y + (rnd(3) ? rf() * 0.1f : 0.0f);
        }
        for (i = 0; i < nl; i++) perm[i] = i;
        for (i = nl - 1; i > 0; i--) { j = rnd(i + 1); int t = perm[i]; perm[i] = perm[j]; perm[j] = t; }
        for (i = 0; i < nl; i++) {
            MapLine* m = &map[perm[i]];
            bool chain_end = rnd(5) == 0;
            m->v0_idx = rnd(nv);
            m->v1_idx = rnd(nv);
            m->next_id0 = (i + 1 < nl && !chain_end) ? i + 1 : -1;
            m->prev_id0 = -1;
            m->next_id1 = rnd(2) ? (s16) rnd(nl) : -1;
            m->prev_id1 = rnd(2) ? (s16) rnd(nl) : -1;
            m->hi_flags = rnd(0x10000);
            m->lo_flags = rnd(0x10000);
            lines[i].x0 = m;
            lines[i].flags = (1u << rnd(4)) | (rnd(6) == 0 ? (1u << rnd(4)) : 0) |
                             (rnd(5) ? LINE_FLAG_ENABLED : 0) | (rnd(10) == 0 ? LINE_FLAG_HIDDEN : 0);
            line_active_tab[i] = rnd(8) != 0;
        }
        for (i = 0; i < nl; i++)
            if (lines[i].x0->next_id0 != -1) lines[lines[i].x0->next_id0].x0->prev_id0 = i;

        printf("G %d %d\n", nv, nl);
        for (i = 0; i < nv; i++)
            printf("V %08x %08x %08x %08x %08x %08x\n", fbits(vtx[i].x0), fbits(vtx[i].x4),
                   fbits(vtx[i].pos.x), fbits(vtx[i].pos.y), fbits(vtx[i].x10), fbits(vtx[i].x14));
        for (i = 0; i < nl; i++) {
            MapLine* m = lines[i].x0;
            printf("L %d %d %d %d %d %d %d %d %d %u %d\n", (int) (m - map), m->v0_idx, m->v1_idx,
                   m->prev_id0, m->next_id0, m->prev_id1, m->next_id1, m->hi_flags, m->lo_flags,
                   lines[i].flags, line_active_tab[i]);
        }
        for (i = 0; i < nl; i++) {
            for (k = 0; k < 4; k++) {
                Vec3 q;
                q.x = (f32) (int) rnd(12) * 0.75f - 4.0f + (rnd(2) ? rf() * 0.3f : 0.0f);
                q.y = (f32) (int) rnd(12) * 1.25f + (rnd(2) ? rf() * 0.3f : 0.0f);
                q.z = 0.0f;
                DOQ(mpLib_8004DD90_Floor);
                DOQ(mpLib_8004E090_Ceiling);
                DOQ(mpLib_8004E398_LeftWall);
                DOQ(mpLib_8004E684_RightWall);
                {
                    Vec3 sp = { -9, -9, -9 };
                    int r;
                    active_budget = 100000;
                    r = mpGetSpeed(i, &q, &sp);
                    printf("S %d %08x %08x %d %08x %08x %08x\n", i, fbits(q.x), fbits(q.y), r,
                           fbits(sp.x), fbits(sp.y), fbits(sp.z));
                }
            }
            {
                float a, b, c, d;
                active_budget = 100000;
                if (setjmp(loop_jmp)) {
                    printf("LOOP mpLib_8004ED5C %d\n", i);
                } else {
                    mpLib_8004ED5C(i, &a, &b, &c, &d);
                    printf("E %d %08x %08x %08x %08x\n", i, fbits(a), fbits(b), fbits(c), fbits(d));
                }
            }
        }
        /* Setters, applied in order; the Rust side replays them and checks the vertex table. */
        for (k = 0; k < 6; k++) {
            int l = rnd(nl);
            f32 a = rf(), b = rf(), c = rf(), d = rf();
            int which = rnd(3);
            if (which == 0) mpLineSetPos(l, a, b, c, d);
            else if (which == 1) mpLib_80056758(l, a, b, c, d);
            else mpVtxSetPos(l % nv, a, b);
            printf("W %d %d %08x %08x %08x %08x\n", which, l, fbits(a), fbits(b), fbits(c), fbits(d));
        }
        for (i = 0; i < nv; i++) {
            float x, y;
            mpVtxGetPos(i, &x, &y);
            printf("X %d %08x %08x\n", i, fbits(x), fbits(y));
        }
    }
    for (i = 0; i < 6000; i++) {
        f32 v[10];
        float ox = -7.0f, oy = -7.0f;
        int r;
        for (j = 0; j < 10; j++) v[j] = rf();
        if (rnd(4) == 0) v[3] = v[1]; /* horizontal a */
        if (rnd(6) == 0) { v[6] = v[4]; v[7] = v[5]; } /* point b */
        mpRemap2d(&ox, &oy, v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], v[8], v[9]);
        printf("M");
        for (j = 0; j < 10; j++) printf(" %08x", fbits(v[j]));
        printf(" %08x %08x\n", fbits(ox), fbits(oy));
        ox = oy = -7.0f;
        r = mpLineIntersection(v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], &ox, &oy);
        printf("I %d %08x %08x\n", r, fbits(ox), fbits(oy));
        ox = oy = -7.0f;
        r = mpLineIntersectionH(&ox, &oy, v[0], v[1], v[2], v[4], v[5], v[6], v[7]);
        printf("H %d %08x %08x\n", r, fbits(ox), fbits(oy));
        ox = oy = -7.0f;
        r = mpLineIntersectionV(&ox, &oy, v[0], v[1], v[3], v[4], v[5], v[6], v[7]);
        printf("Y %d %08x %08x\n", r, fbits(ox), fbits(oy));
        {
            f32 s = ABS(v[8]) * (f32) (1 + rnd(1000));
            printf("Z %08x %08x\n", fbits(s), fbits(sqrtf(s)));
        }
    }
    return 0;
}
