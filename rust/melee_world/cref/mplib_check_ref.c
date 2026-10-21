/* C reference for melee_world::mp::check. Built by gen_mplib_check_ref.sh, which cuts the real
 * bodies out of src/melee/mp/mplib.c (mplib_lines_body.inc, mplib_geom_body.inc,
 * mplib_check_body.inc). check_prelude.inc is mplib_geom_ref.c up to and including its
 * `#include "mplib_lines_body.inc"` (types, PSVECNormalize, sqrtf, StageLineActive hook). */
#include "check_prelude.inc"

/* --- joint-list prelude for the mpCheck* bodies --- */
#define LINE_FLAG_EMPTY (1 << 7)
#define LINE_FLAG_LEDGE (1 << 9)
#define F32_MAX 3.4028235e38f
typedef int ssize_t;
typedef uint8_t u8;
typedef void Fighter_GObj;
enum { MapLineGroup_Floor, MapLineGroup_Ceiling, MapLineGroup_RightWall, MapLineGroup_LeftWall,
       MapLineGroup_Dynamic, MapLineGroup_Count };
enum {
    CollJoint_B8 = 1 << 8, CollJoint_B9 = 1 << 9, CollJoint_B10 = 1 << 10, CollJoint_B11 = 1 << 11,
    CollJoint_TooFar = 1 << 12, CollJoint_Enabled = 1 << 16, CollJoint_Hidden = 1 << 18,
};
typedef struct { s16 start, count; } MapLineRange;
typedef struct MapJoint { MapLineRange ranges[MapLineGroup_Count]; } MapJoint;
typedef struct CollJoint {
    struct CollJoint* next;
    MapJoint* inner;
    u32 flags;
    Vec2 bounding_min, bounding_max;
} CollJoint;
static CollJoint* groundCollJoint;
static CollJoint* jointListStart;
static bool didCheckBounding;
static unsigned char joint_active_tab[16];
int ScriptGame_StageJointActive(int joint) { return joint_active_tab[joint]; }
bool mpCheckedBounding(void);
void mpBoundingCheck(float left, float bottom, float right, float top);
void mpBoundingCheck2(float x1, float y1, float x2, float y2);
void mpBoundingCheck3(float x0, float y0, float x1, float y1, float x2, float y2, float x3, float y3);
void mpUncheckBounding(void);
#include "mplib_geom_body.inc"
#include "mplib_check_body.inc"

static u32 rng = 0x13572468u;
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
static f32 rx(void) { return (f32) (int) rnd(12) * 0.75f - 4.0f + (rnd(2) ? rf() * 0.3f : 0.0f); }
static f32 ry(void) { return (f32) (int) rnd(12) * 1.25f + (rnd(2) ? rf() * 0.3f : 0.0f); }

static unsigned char cb_tab[64];
static bool test_cb(Fighter_GObj* gobj, int line) { (void) gobj; return cb_tab[line]; }

static CollJoint joints[8];
static int nj;
static int pre;
static f32 Q[8];

/* pre: run each query with the bounding box already checked (mpBoundingCheck3 over Q), so the
 * query neither culls nor unchecks; the flags are printed, then unchecked here. */
static void before(void)
{
    active_budget = 1000000000;
    if (pre) mpBoundingCheck3(Q[0], Q[1], Q[2], Q[3], Q[4], Q[5], Q[6], Q[7]);
}
static void after(void)
{
    int j;
    printf(" %d", didCheckBounding);
    for (j = 0; j < nj; j++) printf(" %x", joints[j].flags);
    printf("\n");
    if (pre) mpUncheckBounding();
}
static void hit(const char* name, int r, Vec3 p, int lid, u32 fl, Vec3 n)
{
    printf("R %s %d %08x %08x %08x %d %u %08x %08x %08x", name, r, fbits(p.x), fbits(p.y),
           fbits(p.z), lid, fl, fbits(n.x), fbits(n.y), fbits(n.z));
    after();
}
#define S4(name, call)                                                                    \
    do {                                                                                  \
        Vec3 p = { -9, -9, -9 }, n = { -9, -9, -9 };                                      \
        int lid = -5;                                                                     \
        u32 fl = 0xdead;                                                                  \
        int r;                                                                            \
        before();                                                                         \
        r = call;                                                                         \
        hit(name, r, p, lid, fl, n);                                                      \
    } while (0)

int main(void)
{
    static CollVtx vtx[64];
    static MapLine map[48];
    static CollLine lines[48];
    static MapJoint mj[8];
    static MapCollData mcd;
    int g, i, j, k;
    mpLib_804D64B4 = &mcd;
    groundCollVtx = vtx;
    groundCollLine = lines;
    groundCollJoint = joints;
    for (g = 0; g < 48; g++) {
        int nv = 8 + rnd(56), nl = 4 + rnd(44);
        int perm[48];
        mcd.line_count = nl;
        nj = 1 + rnd(6);
        for (i = 0; i < nv; i++) {
            memset(&vtx[i], 0, sizeof vtx[i]);
            vtx[i].pos.x = (f32) (int) rnd(12) * 0.75f + (rnd(4) == 0 ? 0.3f : 0.0f) - 4.0f;
            vtx[i].pos.y = (f32) (int) rnd(12) * 1.25f - (rnd(3) == 0 ? 0.6f : 0.0f);
            if (rnd(2)) { vtx[i].pos.x += rf() * 0.01f; vtx[i].pos.y += rf() * 0.01f; }
            vtx[i].x0 = rf();
            vtx[i].x4 = rf();
            vtx[i].x10 = vtx[i].pos.x + (rnd(3) ? rf() * 0.3f : 0.0f);
            vtx[i].x14 = vtx[i].pos.y + (rnd(3) ? rf() * 0.3f : 0.0f);
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
                             (rnd(6) ? LINE_FLAG_ENABLED : 0) | (rnd(10) == 0 ? LINE_FLAG_HIDDEN : 0) |
                             (rnd(10) == 0 ? LINE_FLAG_EMPTY : 0);
            line_active_tab[i] = rnd(8) != 0;
            cb_tab[i] = rnd(4) != 0;
            /* flatten some lines exactly (horizontal or vertical) for the flat-line paths */
            if (rnd(3) == 0) vtx[m->v1_idx].pos.y = vtx[m->v0_idx].pos.y;
            else if (rnd(4) == 0) vtx[m->v1_idx].pos.x = vtx[m->v0_idx].pos.x;
        }
        for (i = 0; i < nl; i++)
            if (lines[i].x0->next_id0 != -1) lines[lines[i].x0->next_id0].x0->prev_id0 = i;
        for (j = 0; j < nj; j++) {
            f32 x0 = rx(), x1 = rx(), y0 = ry(), y1 = ry();
            for (k = 0; k < MapLineGroup_Count; k++) {
                int s = rnd(nl);
                mj[j].ranges[k].start = s;
                mj[j].ranges[k].count = (k == MapLineGroup_Dynamic && rnd(2)) ? 0 : rnd(nl - s + 1);
            }
            joints[j].inner = &mj[j];
            joints[j].next = NULL;
            joints[j].flags = (rnd(4) == 0 ? CollJoint_B8 : 0) | (rnd(4) == 0 ? CollJoint_B9 : 0) |
                              (rnd(5) == 0 ? CollJoint_B10 : 0) | (rnd(5) ? CollJoint_Enabled : 0) |
                              (rnd(8) == 0 ? CollJoint_Hidden : 0) | (rnd(8) == 0 ? CollJoint_TooFar : 0) |
                              rnd(0x100);
            joints[j].bounding_min.x = x0 < x1 ? x0 : x1;
            joints[j].bounding_max.x = x0 < x1 ? x1 : x0;
            joints[j].bounding_min.y = y0 < y1 ? y0 : y1;
            joints[j].bounding_max.y = y0 < y1 ? y1 : y0;
            joint_active_tab[j] = rnd(6) != 0;
        }
        {
            int order[8], cnt;
            for (j = 0; j < nj; j++) order[j] = j;
            for (j = nj - 1; j > 0; j--) { k = rnd(j + 1); int t = order[j]; order[j] = order[k]; order[k] = t; }
            cnt = nj - (rnd(4) == 0 ? 1 : 0);
            jointListStart = NULL;
            for (j = cnt - 1; j >= 0; j--) {
                joints[order[j]].next = jointListStart;
                jointListStart = &joints[order[j]];
            }
        }
        didCheckBounding = false;

        printf("G %d %d %d %d\n", nv, nl, nj, jointListStart ? (int) (jointListStart - joints) : -1);
        for (i = 0; i < nv; i++)
            printf("V %08x %08x %08x %08x %08x %08x\n", fbits(vtx[i].x0), fbits(vtx[i].x4),
                   fbits(vtx[i].pos.x), fbits(vtx[i].pos.y), fbits(vtx[i].x10), fbits(vtx[i].x14));
        for (i = 0; i < nl; i++) {
            MapLine* m = lines[i].x0;
            printf("L %d %d %d %d %d %d %d %d %d %u %d %d\n", (int) (m - map), m->v0_idx, m->v1_idx,
                   m->prev_id0, m->next_id0, m->prev_id1, m->next_id1, m->hi_flags, m->lo_flags,
                   lines[i].flags, line_active_tab[i], cb_tab[i]);
        }
        for (j = 0; j < nj; j++) {
            printf("J %d %u %08x %08x %08x %08x %d", joints[j].next ? (int) (joints[j].next - joints) : -1,
                   joints[j].flags, fbits(joints[j].bounding_min.x), fbits(joints[j].bounding_min.y),
                   fbits(joints[j].bounding_max.x), fbits(joints[j].bounding_max.y), joint_active_tab[j]);
            for (k = 0; k < MapLineGroup_Count; k++)
                printf(" %d %d", mj[j].ranges[k].start, mj[j].ranges[k].count);
            printf("\n");
        }
        for (i = 0; i < 72; i++) {
            int lskip = rnd(3) ? -1 : (int) rnd(nl);
            int jskip = rnd(3) ? -1 : (int) rnd(nj);
            int jonly = rnd(3) ? -1 : (int) rnd(nj);
            int usecb = rnd(2);
            int dir = rnd(2) ? 1 : -1;
            u32 checks = rnd(32);
            f32 yoff = rnd(2) ? 0.0f : rf() * 0.5f;
            pre = rnd(4) == 0;
            Q[0] = rx(); Q[1] = ry(); Q[2] = rx(); Q[3] = ry();
            if (rnd(5) == 0) Q[3] = Q[1];
            if (rnd(5) == 0) Q[2] = Q[0];
            Q[4] = rx(); Q[5] = ry(); Q[6] = rx(); Q[7] = ry();
            if (rnd(2)) {
                /* aim the segment through a random line's midpoint (vertical or horizontal,
                 * either direction) so hits, flat lines and ties are common */
                MapLine* m = lines[rnd(nl)].x0;
                f32 mx = (vtx[m->v0_idx].pos.x + vtx[m->v1_idx].pos.x) * 0.5f;
                f32 my = (vtx[m->v0_idx].pos.y + vtx[m->v1_idx].pos.y) * 0.5f;
                f32 d0 = 0.5f + (f32) rnd(4), d1 = 0.5f + (f32) rnd(4);
                f32 e = rnd(2) ? 0.0f : rf() * 0.05f;
                if (rnd(2)) { Q[0] = mx + e; Q[2] = mx; Q[1] = my + d0; Q[3] = my - d1; }
                else { Q[1] = my + e; Q[3] = my; Q[0] = mx + d0; Q[2] = mx - d1; }
                if (rnd(2)) { f32 t = Q[0]; Q[0] = Q[2]; Q[2] = t; t = Q[1]; Q[1] = Q[3]; Q[3] = t; }
                if (rnd(2)) { Q[4] = Q[0] + rf() * 0.2f; Q[5] = Q[1]; Q[6] = Q[2]; Q[7] = Q[3] + rf() * 0.2f; }
            }
            printf("C");
            for (k = 0; k < 8; k++) printf(" %08x", fbits(Q[k]));
            printf(" %08x %d %d %d %d %d %u %d\n", fbits(yoff), lskip, jskip, jonly, usecb, dir, checks, pre);
#define A4 Q[0], Q[1], Q[2], Q[3]
            S4("mpCheckFloor", mpCheckFloor(A4, yoff, &p, &lid, &fl, &n, lskip, jskip, jonly, usecb ? test_cb : NULL, NULL));
            S4("mpCheckFloorRemap", mpCheckFloorRemap(A4, yoff, &p, &lid, &fl, &n, lskip, jskip, jonly, usecb ? test_cb : NULL, NULL));
            S4("mpCheckCeiling", mpCheckCeiling(A4, &p, &lid, &fl, &n, jskip, jonly));
            S4("mpCheckCeilingRemap", mpCheckCeilingRemap(A4, &p, &lid, &fl, &n, jskip, jonly));
            S4("mpCheckLeftWall", mpCheckLeftWall(A4, &p, &lid, &fl, &n, jskip, jonly));
            S4("mpCheckLeftWallRemap", mpCheckLeftWallRemap(A4, &p, &lid, &fl, &n, jskip, jonly));
            S4("mpCheckRightWall", mpCheckRightWall(A4, &p, &lid, &fl, &n, jskip, jonly));
            S4("mpCheckRightWallRemap", mpCheckRightWallRemap(A4, &p, &lid, &fl, &n, jskip, jonly));
            S4("mpLib_800511A4_RightWall", mpLib_800511A4_RightWall(A4, Q[4], Q[5], Q[6], Q[7], &lid, jskip, jonly));
            S4("mpLib_800515A0_LeftWall", mpLib_800515A0_LeftWall(A4, Q[4], Q[5], Q[6], Q[7], &lid, jskip, jonly));
            {
                Vec3 v = { Q[0], Q[1], 0.0f };
                S4("mpLib_8005199C_Floor", mpLib_8005199C_Floor(&v, jskip, jonly));
            }
            S4("mpLib_80051BA8_Floor", mpLib_80051BA8_Floor(&p, lskip, jskip, jonly, dir, Q[4], Q[5], Q[6], Q[7]));
            S4("mpCheckMultiple", mpCheckMultiple(A4, &p, &lid, &fl, &n, checks, jskip, jonly));
            S4("mpCheckAll", mpCheckAll(&p, &lid, &fl, &n, jskip, jonly, A4));
            S4("mpCheckAllRemap", mpCheckAllRemap(&p, &lid, &fl, &n, jskip, jonly, A4));
        }
    }
    return 0;
}
