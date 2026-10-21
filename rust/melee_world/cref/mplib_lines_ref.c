/* C reference for melee_world::mp::lines. Build with gen_mplib_lines_ref.sh (it extracts the
 * real function bodies from src/melee/mp/mplib.c into mplib_lines_body.inc). Prints random line
 * graphs followed by every query's result; the Rust test rebuilds each graph and compares. */
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

#include "mplib_lines_body.inc"

static u32 rng = 0x1234567u;
static u32 rnd(u32 n)
{
    rng = rng * 1103515245u + 12345u;
    return (rng >> 8) % n;
}
static u32 fbits(f32 f) { u32 u; memcpy(&u, &f, 4); return u; }

typedef int (*IdFn)(int);
typedef void (*PosFn)(int, Vec3*);
#define ID(f) { #f, f }
static const struct { const char* name; IdFn fn; } id_fns[] = {
    ID(mpLineGetNext), ID(mpLineGetPrev),
    ID(mpLineNextNonFloor), ID(mpLinePrevNonFloor), ID(mpLinePrevNonCeiling),
    ID(mpLineNextNonCeiling), ID(mpLineNextNonLeftWall), ID(mpLinePrevNonLeftWall),
    ID(mpLinePrevNonRightWall), ID(mpLineNextNonRightWall),
    ID(mpLib_80053394_Floor), ID(mpLib_80053448_Floor), ID(mpLib_800534FC_Floor),
    ID(mpLib_800536CC_Floor), ID(mpLib_8005389C_Ceiling), ID(mpLib_80053950_Ceiling),
    ID(mpLib_80053A04_Ceiling), ID(mpLib_80053BD4_Ceiling),
    ID(mpLineGetKind), { "mpLineGetFlags", (IdFn) mpLineGetFlags }, ID(mpLib_80054ED8),
};
static const struct { const char* name; PosFn fn; } pos_fns[] = {
    ID(mpLib_80053DA4_Floor), ID(mpLib_80053ECC_Floor),
    ID(mpFloorGetRight), ID(mpFloorGetLeft), ID(mpCeilingGetRight), ID(mpCeilingGetLeft),
    ID(mpLeftWallGetTop), ID(mpLeftWallGetBottom), ID(mpRightWallGetTop),
    ID(mpRightWallGetBottom), ID(mpLineGetV1Pos), ID(mpLineGetV0Pos),
};

int main(void)
{
    static CollVtx vtx[64];
    static MapLine map[48];
    static CollLine lines[48];
    static MapCollData mcd;
    int g, i, j;
    mpLib_804D64B4 = &mcd;
    groundCollVtx = vtx;
    groundCollLine = lines;
    for (g = 0; g < 24; g++) {
        int nv = 8 + rnd(56), nl = 4 + rnd(44);
        int perm[48];
        mcd.line_count = nl;
        for (i = 0; i < nv; i++) {
            memset(&vtx[i], 0, sizeof vtx[i]);
            vtx[i].pos.x = (f32) (int) rnd(12) * 0.75f + (rnd(4) == 0 ? 0.3f : 0.0f) - 4.0f;
            vtx[i].pos.y = (f32) (int) rnd(12) * 1.25f - (rnd(3) == 0 ? 0.6f : 0.0f);
        }
        for (i = 0; i < nl; i++) perm[i] = i;
        for (i = nl - 1; i > 0; i--) { j = rnd(i + 1); int t = perm[i]; perm[i] = perm[j]; perm[j] = t; }
        for (i = 0; i < nl; i++) {
            MapLine* m = &map[perm[i]];
            bool chain_end = rnd(5) == 0;
            m->v0_idx = rnd(nv);
            m->v1_idx = rnd(nv);
            /* id0 chains are acyclic (index order) so the unbounded id0 walks terminate. */
            m->next_id0 = (i + 1 < nl && !chain_end) ? i + 1 : -1;
            m->prev_id0 = -1;
            m->next_id1 = rnd(2) ? (s16) rnd(nl) : -1;
            m->prev_id1 = rnd(2) ? (s16) rnd(nl) : -1;
            m->hi_flags = rnd(0x10000);
            m->lo_flags = rnd(0x10000);
            lines[i].x0 = m;
            lines[i].flags = (1u << rnd(4)) | (rnd(6) == 0 ? (1u << rnd(4)) : 0) |
                             (rnd(5) ? LINE_FLAG_ENABLED : 0) | (rnd(10) == 0 ? LINE_FLAG_HIDDEN : 0) |
                             (rnd(3) == 0 ? 0x100 : 0);
            line_active_tab[i] = rnd(8) != 0;
        }
        for (i = 0; i < nl; i++)
            if (lines[i].x0->next_id0 != -1) lines[lines[i].x0->next_id0].x0->prev_id0 = i;

        printf("G %d %d\n", nv, nl);
        for (i = 0; i < nv; i++) printf("V %08x %08x\n", fbits(vtx[i].pos.x), fbits(vtx[i].pos.y));
        for (i = 0; i < nl; i++) {
            MapLine* m = lines[i].x0;
            printf("L %d %d %d %d %d %d %d %d %d %u %d\n", (int) (m - map), m->v0_idx, m->v1_idx,
                   m->prev_id0, m->next_id0, m->prev_id1, m->next_id1, m->hi_flags, m->lo_flags,
                   lines[i].flags, line_active_tab[i]);
        }
        for (i = 0; i < nl; i++) {
            for (j = 0; j < (int) (sizeof id_fns / sizeof id_fns[0]); j++) {
                active_budget = 100000;
                if (setjmp(loop_jmp)) { printf("LOOP %s %d\n", id_fns[j].name, i); continue; }
                printf("R %s %d %d\n", id_fns[j].name, i, id_fns[j].fn(i));
            }
            for (j = 0; j < (int) (sizeof pos_fns / sizeof pos_fns[0]); j++) {
                Vec3 p;
                active_budget = 100000;
                if (setjmp(loop_jmp)) { printf("LOOP %s %d\n", pos_fns[j].name, i); continue; }
                pos_fns[j].fn(i, &p);
                printf("P %s %d %08x %08x %08x\n", pos_fns[j].name, i, fbits(p.x), fbits(p.y), fbits(p.z));
            }
            {
                Vec3 n;
                active_budget = 100000;
                mpLineGetNormal(i, &n);
                printf("P mpLineGetNormal %d %08x %08x %08x\n", i, fbits(n.x), fbits(n.y), fbits(n.z));
            }
            {
                int t = rnd(nl);
                active_budget = 100000;
                if (setjmp(loop_jmp)) printf("LOOP mpLinesConnected %d %d\n", i, t);
                else printf("C %d %d %d\n", i, t, mpLinesConnected(i, t));
            }
            {
                u16 save = lines[i].x0->lo_flags;
                u32 f = rnd(0x400);
                mpLib_80054D68(i, f);
                printf("F %d %u %u\n", i, f, lines[i].x0->lo_flags);
                lines[i].x0->lo_flags = save;
            }
        }
    }
    {
        static const float dirs[][2] = { {1, 0}, {0, 1}, {3, 4}, {-1, 0.5f}, {1e-3f, 2e3f}, {0.1f, 0.2f} };
        for (i = 0; i < (int) (sizeof dirs / sizeof dirs[0]); i++) {
            Vec3 v = { dirs[i][0], dirs[i][1], 0.5f * (f32) i }, o;
            PSVECNormalize(&v, &o);
            printf("N %08x %08x %08x %08x %08x %08x\n", fbits(v.x), fbits(v.y), fbits(v.z),
                   fbits(o.x), fbits(o.y), fbits(o.z));
        }
    }
    return 0;
}
