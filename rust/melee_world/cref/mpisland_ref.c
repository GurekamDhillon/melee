/* C reference for melee_world::mp::island and mp::stitch. Built by gen_mpisland_ref.sh, which
 * cuts the real bodies out of src/melee/mp/mplib.c (lines, geom, joint, check, stitch) and
 * src/melee/mp/mpisland.c. check_prelude.inc is mplib_geom_ref.c up to and including its
 * `#include "mplib_lines_body.inc"` (types, PSVECNormalize, sqrtf, StageLineActive hook).
 * mpScriptIslandRefresh is mpIsland_8005B334, as on the console. */
#include "check_prelude.inc"

#define LINE_FLAG_EMPTY (1 << 7)
#define LINE_FLAG_PLATFORM (1 << 8)
#define LINE_FLAG_LEDGE (1 << 9)
#define F32_MAX 3.4028235e38f
#define UNUSED
typedef int ssize_t;
typedef uint8_t u8;
typedef void Fighter_GObj;
typedef void HSD_GObj;
enum { MapLineGroup_Floor, MapLineGroup_Ceiling, MapLineGroup_RightWall, MapLineGroup_LeftWall,
       MapLineGroup_Dynamic, MapLineGroup_Count };
enum {
    CollJoint_B8 = 1 << 8, CollJoint_B9 = 1 << 9, CollJoint_B10 = 1 << 10, CollJoint_B11 = 1 << 11,
    CollJoint_TooFar = 1 << 12, CollJoint_Enabled = 1 << 16, CollJoint_Hidden = 1 << 18,
};
typedef struct MapLineRange { s16 start, count; } MapLineRange;
typedef struct MapJoint {
    MapLineRange ranges[MapLineGroup_Count];
    f32 left_bound, bottom_bound, right_bound, top_bound;
    s16 vtx_start, vtx_count;
} MapJoint;
typedef struct CollJoint {
    struct CollJoint* next;
    MapJoint* inner;
    u32 flags;
    s16 xC;
    u8 xE;
    Vec2 bounding_min, bounding_max;
} CollJoint;
typedef struct { f32 top, bottom, left, right; } mpCollisionBox;
static CollJoint* groundCollJoint;
static CollJoint* jointListStart;
static CollJoint* jointListEnd;
static bool didCheckBounding;
static mpCollisionBox mpLib_80458868[2];
#define joint_count pad[0]
static unsigned char joint_active_tab[16];
static unsigned char stage_joint_tab[16];
int ScriptGame_StageJointActive(int joint) { return joint_active_tab[joint]; }
int ScriptGame_StageJoint(int joint) { return stage_joint_tab[joint]; }
bool mpCheckedBounding(void);
void mpBoundingCheck(float left, float bottom, float right, float top);
void mpBoundingCheck2(float x1, float y1, float x2, float y2);
void mpBoundingCheck3(float x0, float y0, float x1, float y1, float x2, float y2, float x3, float y3);
void mpUncheckBounding(void);
void mpIsland_8005B334(int arg0, int arg1, int arg2, bool arg3);
static void mpScriptIslandRefresh(int joint, int start, int count, bool enabled)
{
    mpIsland_8005B334(joint, start, count, enabled);
}
static void grDynamicAttr_801CA224(void) {}
static int gobj_dummy;
static HSD_GObj* GObj_Create(int a, int b, int c) { (void) a; (void) b; (void) c; return &gobj_dummy; }
#define HSD_GObj_SetupProc(g, f, p) ((void) (g), (void) (f))
#include "mplib_geom_body.inc"
#include "mplib_joint_body.inc"
#include "mplib_check_body.inc"
#include "mplib_stitch_body.inc"

/* --- mpisland.c prelude --- */
typedef struct mp_UnkStruct0 mp_UnkStruct0;
struct mpIsland_80458E88_t {
    mp_UnkStruct0 *next, *x4, *x8, *xC, *x10, *x14, *x18, *x1C, *x20;
};
struct mp_UnkStruct0 {
    mp_UnkStruct0* next;
    u16 x4, x6;
    Vec3 x8, x14;
    int x20;
    s16 x24, x26, x28, x2A;
    void* ptr;
};
#define POOL 40000
static mp_UnkStruct0 pool[POOL];
static unsigned char pool_freed[POOL];
static int npool;
static void* HSD_MemAlloc(int size)
{
    (void) size;
    if (npool >= POOL) { printf("POOL\n"); exit(4); }
    memset(&pool[npool], 0, sizeof pool[npool]);
    return &pool[npool++];
}
static void HSD_Free(void* p) { pool_freed[(mp_UnkStruct0*) p - pool] = 1; }
#define memzero(p, n) memset((p), 0, (n))
static struct IslandMap { MapLineRange ranges[MapLineGroup_Count]; } island_map;
#define mpLib_8004D164() (&island_map)
#define MapCollData struct IslandMap
static CollLine* mpGetGroundCollLine(void) { return groundCollLine; }
static CollVtx* mpGetGroundCollVtx(void) { return groundCollVtx; }
static CollJoint* mpGetGroundCollJoint(void) { return groundCollJoint; }
#include "mpisland_body.inc"
#undef MapCollData

static u32 rng = 0x1517a9d3u;
static u32 rnd(u32 n)
{
    rng = rng * 1103515245u + 12345u;
    return (rng >> 8) % n;
}
static u32 fbits(f32 f) { u32 u; memcpy(&u, &f, 4); return u; }
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

static CollVtx vtx[64];
static MapLine map[48];
static CollLine lines[48];
static MapJoint mj[8];
static CollJoint joints[8];
static int nv, nl, nj;

static int six(mp_UnkStruct0* s) { return s ? (int) (s - pool) : -1; }
static void dump_list(const char* name, mp_UnkStruct0* s)
{
    int n = 0;
    printf(" %s", name);
    for (; s && n < 5000; s = s->next, n++)
        printf(" [%d %u %u %08x %08x %08x %08x %08x %08x %d %d %d %d]", six(s), s->x4, s->x6,
               fbits(s->x8.x), fbits(s->x8.y), fbits(s->x8.z), fbits(s->x14.x), fbits(s->x14.y),
               fbits(s->x14.z), s->x20, s->x24, s->x26, s->x28);
    if (s) printf(" CYC");
}
static void dump(void)
{
    int i;
    printf("S %d %d |", six(mpIsland_80458E88.x8), six(mpIsland_80458E88.xC));
    dump_list("n", mpIsland_80458E88.next);
    dump_list("c", mpIsland_80458E88.x4);
    dump_list("d", mpIsland_80458E88.x10);
    dump_list("e", mpIsland_80458E88.x14);
    dump_list("h", mpIsland_80458E88.x18);
    dump_list("i", mpIsland_80458E88.x1C);
    dump_list("f", mpIsland_80458E88.x20);
    printf(" |");
    for (i = 0; i < nl; i++) printf(" %d,%d", map[i].prev_id1, map[i].next_id1);
    printf(" |");
    for (i = 0; i < nj; i++) printf(" %x,%d", joints[i].flags, joints[i].xE);
    printf(" |");
    for (i = 0; i < 2; i++)
        printf(" %08x %08x %08x %08x", fbits(mpLib_80458868[i].top), fbits(mpLib_80458868[i].bottom),
               fbits(mpLib_80458868[i].left), fbits(mpLib_80458868[i].right));
    printf("\n");
}

/* a random live segment (from the surface, hidden and free lists) with a valid joint, or -1 */
static int pick_seg(void)
{
    mp_UnkStruct0* lists[4];
    int c[512], n = 0, k;
    lists[0] = mpIsland_80458E88.next;
    lists[1] = mpIsland_80458E88.x4;
    lists[2] = mpIsland_80458E88.x18;
    lists[3] = mpIsland_80458E88.x20;
    for (k = 0; k < 4; k++) {
        mp_UnkStruct0* s;
        int m = 0;
        for (s = lists[k]; s && n < 512 && m < 5000; s = s->next, m++)
            if (s->x28 >= 0 && s->x28 < nj) c[n++] = six(s);
    }
    return n ? c[rnd(n)] : -1;
}

int main(void)
{
    static MapCollData mcd;
    int g, i, j, k;
    mpLib_804D64B4 = &mcd;
    groundCollVtx = vtx;
    groundCollLine = lines;
    groundCollJoint = joints;
    for (g = 0; g < 128; g++) {
        int perm[48];
        int ring = rnd(3) == 0;
        nv = 8 + rnd(56);
        nl = 4 + rnd(44);
        nj = 1 + rnd(7);
        mcd.line_count = nl;
        mcd.joint_count = nj;
        for (i = 0; i < nv; i++) {
            memset(&vtx[i], 0, sizeof vtx[i]);
            vtx[i].pos.x = (f32) (int) rnd(12) * 0.75f + (rnd(4) == 0 ? 0.3f : 0.0f) - 4.0f;
            vtx[i].pos.y = (f32) (int) rnd(12) * 1.25f - (rnd(3) == 0 ? 0.6f : 0.0f);
            if (rnd(2)) { vtx[i].pos.x += rf() * 0.01f; vtx[i].pos.y += rf() * 0.01f; }
            if (rnd(5) == 0) { vtx[i].pos.x = rf() * 10.0f; vtx[i].pos.y = rf() * 10.0f; }
        }
        for (i = 0; i < nl; i++) perm[i] = i;
        for (i = nl - 1; i > 0; i--) { j = rnd(i + 1); int t = perm[i]; perm[i] = perm[j]; perm[j] = t; }
        /* MapLine map[perm[i]] is CollLine i's; chains (or rings) over runs of line ids */
        for (i = 0; i < nl; i++) {
            MapLine* m = &map[perm[i]];
            static const u32 lowtab[6] = { 0x11, 0x12, 0x01, 0x02, 0x11, 0x12 };
            m->v0_idx = rnd(nv);
            m->v1_idx = rnd(nv);
            m->next_id1 = rnd(2) ? (s16) rnd(nl) : -1;
            m->prev_id1 = rnd(2) ? (s16) rnd(nl) : -1;
            m->hi_flags = (rnd(3) ? (1u << rnd(2)) : rnd(16)) | (rnd(0x1000) << 4);
            m->lo_flags = rnd(0x10000);
            lines[i].x0 = m;
            lines[i].flags = (rnd(5) ? lowtab[rnd(6)] : rnd(32)) | (rnd(6) ? LINE_FLAG_ENABLED : 0) |
                             (rnd(10) == 0 ? LINE_FLAG_HIDDEN : 0) | (rnd(10) == 0 ? LINE_FLAG_EMPTY : 0) |
                             (rnd(0x100) << 20);
            line_active_tab[i] = rnd(10) != 0;
        }
        for (i = 0; i < nl;) {
            int len = 1 + rnd(8), s = i;
            if (s + len > nl) len = nl - s;
            for (k = 0; k < len; k++) {
                MapLine* m = lines[s + k].x0;
                m->next_id0 = k + 1 < len ? s + k + 1 : (ring ? s : -1);
                m->prev_id0 = k > 0 ? s + k - 1 : (ring ? s + len - 1 : -1);
                /* chains mostly share their joint vertices */
                if (k > 0 && rnd(4)) m->v0_idx = lines[s + k - 1].x0->v1_idx;
            }
            i += len;
        }
        for (j = 0; j < nj; j++) {
            int s;
            for (k = 0; k < MapLineGroup_Count; k++) {
                s = rnd(nl);
                mj[j].ranges[k].start = s;
                mj[j].ranges[k].count = (k == MapLineGroup_Dynamic && rnd(3) == 0) ? 0 : rnd(nl - s + 1);
            }
            s = rnd(nv);
            mj[j].vtx_start = s;
            mj[j].vtx_count = rnd(nv - s + 1);
            joints[j].inner = &mj[j];
            joints[j].next = NULL;
            joints[j].flags = (rnd(4) == 0 ? CollJoint_B8 : 0) | (rnd(6) == 0 ? CollJoint_B9 : 0) |
                              (rnd(5) == 0 ? CollJoint_B10 : 0) | (rnd(6) == 0 ? CollJoint_B11 : 0) |
                              (rnd(8) == 0 ? CollJoint_Hidden : 0) | (rnd(3) == 0 ? CollJoint_Enabled : 0) |
                              rnd(0x100);
            joints[j].xE = 0;
            joints[j].bounding_min.x = rf();
            joints[j].bounding_min.y = rf();
            joints[j].bounding_max.x = rf();
            joints[j].bounding_max.y = rf();
            joint_active_tab[j] = rnd(6) != 0;
            stage_joint_tab[j] = rnd(6) == 0;
        }
        for (k = 0; k < 2; k++) {
            int s = rnd(nl);
            island_map.ranges[k].start = s;
            island_map.ranges[k].count = rnd(nl - s + 1);
        }
        for (k = 0; k < 2; k++) {
            mpLib_80458868[k].top = rf();
            mpLib_80458868[k].bottom = rf();
            mpLib_80458868[k].left = rf();
            mpLib_80458868[k].right = rf();
        }
        jointListStart = jointListEnd = NULL;
        didCheckBounding = false;
        mpIsland_8005A6F8();
        npool = 0; /* the Rust arena is per graph */
        memset(pool_freed, 0, sizeof pool_freed);
        printf("G %d %d %d\n", nv, nl, nj);
        for (i = 0; i < nv; i++)
            printf("V %08x %08x\n", fbits(vtx[i].pos.x), fbits(vtx[i].pos.y));
        for (i = 0; i < nl; i++) {
            MapLine* m = lines[i].x0;
            printf("L %d %d %d %d %d %d %d %d %d %u %d\n", (int) (m - map), m->v0_idx, m->v1_idx,
                   m->prev_id0, m->next_id0, m->prev_id1, m->next_id1, m->hi_flags, m->lo_flags,
                   lines[i].flags, line_active_tab[i]);
        }
        for (j = 0; j < nj; j++) {
            printf("J %u %08x %08x %08x %08x %d %d %d %d", joints[j].flags, fbits(joints[j].bounding_min.x),
                   fbits(joints[j].bounding_min.y), fbits(joints[j].bounding_max.x),
                   fbits(joints[j].bounding_max.y), mj[j].vtx_start, mj[j].vtx_count,
                   joint_active_tab[j], stage_joint_tab[j]);
            for (k = 0; k < MapLineGroup_Count; k++)
                printf(" %d %d", mj[j].ranges[k].start, mj[j].ranges[k].count);
            printf("\n");
        }
        printf("M %d %d %d %d", island_map.ranges[0].start, island_map.ranges[0].count,
               island_map.ranges[1].start, island_map.ranges[1].count);
        for (k = 0; k < 2; k++)
            printf(" %08x %08x %08x %08x", fbits(mpLib_80458868[k].top), fbits(mpLib_80458868[k].bottom),
                   fbits(mpLib_80458868[k].left), fbits(mpLib_80458868[k].right));
        printf("\n");
        for (i = 0; i < 70; i++) {
            int op = rnd(23), a = rnd(nj), b = rnd(nj), l = rnd(nl);
            active_budget = 100000;
            if (setjmp(loop_jmp)) {
                printf("R LOOP\n");
                dump();
                continue;
            }
            switch (op) {
            case 0: case 1: printf("O a728\n"); mpIsland_8005A728(); break;
            case 2: {
                int s = rnd(nv), c = rnd(nv - s + 1), e = rnd(2);
                printf("O b334 %d %d %d %d\n", a, s, c, e);
                mpIsland_8005B334(a, s, c, e);
                break;
            }
            case 3: {
                u8 dirty[8];
                int cnt = rnd(nj + 1);
                printf("O batch %d", cnt);
                for (k = 0; k < cnt; k++) { dirty[k] = rnd(3) == 0; printf(" %d", dirty[k]); }
                printf("\n");
                mpIsland_ScriptBatch(dirty, cnt);
                break;
            }
            case 4: if (rnd(3)) continue; printf("O free\n"); mpIsland_StageSlotFree(); break;
            case 5: printf("O 81dc %d %d\n", a, b); mpLib_800581DC(a, b); break;
            case 6: printf("O 8560\n"); mpLib_80058560(); break;
            case 7: printf("O 8614\n"); mpLib_80058614_Floor(); break;
            case 8: { int v = rnd(2); printf("O xe %d %d\n", a, v); joints[a].xE = v; break; }
            case 9: printf("O 87fc\n"); mpLib_800587FC(NULL); break;
            case 10: if (rnd(4)) continue; printf("O 8820\n"); mpLib_80058820(); break;
            case 11: {
                u32 f = joints[a].flags ^ (1u << rnd(19));
                printf("O fl %d %u\n", a, f);
                joints[a].flags = f;
                break;
            }
            case 12: case 13: printf("O add %d\n", a); mpJointListAdd(a); break;
            case 21: case 22: printf("O bound %d\n", a); mpJointUpdateBounding(a); break;
            case 14: case 20:
                if (rnd(2)) { printf("O hide %d\n", a); mpJointHide(a); break; }
                {
                    u32 f;
                    if (rnd(4) && mj[a].ranges[MapLineGroup_Dynamic].count > 0)
                        l = mj[a].ranges[MapLineGroup_Dynamic].start + rnd(mj[a].ranges[MapLineGroup_Dynamic].count);
                    f = lines[l].flags ^ (rnd(2) ? (1u << rnd(5)) : LINE_FLAG_ENABLED);
                    printf("O lf %d %u\n", l, f);
                    lines[l].flags = f;
                }
                break;
            case 15: {
                if (rnd(4)) {
                    mp_UnkStruct0* sp = mpIsland_80458E88.next;
                    int n = 0, pickn = rnd(16), m = 0;
                    for (; sp && m < 5000; sp = sp->next, m++, n++)
                        if (n == pickn) break;
                    if (sp) {
                        l = sp->x24;
                        for (k = rnd(4); k > 0 && lines[l].x0->next_id0 != -1; k--) l = lines[l].x0->next_id0;
                    }
                }
                printf("Q ab54 %d\n", l);
                printf("R %d\n", six(mpIsland_8005AB54(l)));
                break;
            }
            case 16: {
                Vec3 p;
                f32 dy = rnd(4) ? -(1.0f + (f32) rnd(3) + rf() * 0.1f) : rf() * 4.0f;
                MapLine* m = lines[l].x0;
                p.x = vtx[m->v0_idx].pos.x + (vtx[m->v1_idx].pos.x - vtx[m->v0_idx].pos.x) * 0.5f;
                p.y = vtx[m->v0_idx].pos.y + (vtx[m->v1_idx].pos.y - vtx[m->v0_idx].pos.y) * 0.5f +
                      0.5f + rf() * 0.1f;
                p.z = 0.0f;
                printf("Q ac14 %08x %08x %08x\n", fbits(p.x), fbits(p.y), fbits(dy));
                printf("R %d\n", six(mpIsland_8005AC14(&p, dy)));
                break;
            }
            case 17: {
                int s = pick_seg();
                if (s < 0) continue;
                printf("Q ac8c %d\n", s);
                printf("R %d\n", mpIsland_8005AC8C(&pool[s]));
                break;
            }
            default: {
                int s = pick_seg(), w1 = rnd(4) != 0, w2 = rnd(4) != 0;
                Vec3 o1 = { -9, -9, -9 }, o2 = { -9, -9, -9 };
                if (s < 0) continue;
                printf("Q ace8 %d %d %d\n", s, w1, w2);
                mpIsland_8005ACE8(&pool[s], w1 ? &o1 : NULL, w2 ? &o2 : NULL);
                printf("R %08x %08x %08x %08x %08x %08x\n", fbits(o1.x), fbits(o1.y), fbits(o1.z),
                       fbits(o2.x), fbits(o2.y), fbits(o2.z));
                break;
            }
            }
            dump();
        }
    }
    return 0;
}
