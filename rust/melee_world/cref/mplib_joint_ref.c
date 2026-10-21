/* C reference for melee_world::mp::joint. Built by gen_mplib_joint_ref.sh, which cuts the real
 * bodies out of src/melee/mp/mplib.c (mplib_lines_body.inc, mplib_geom_body.inc,
 * mplib_joint_body.inc). check_prelude.inc is mplib_geom_ref.c up to and including its
 * `#include "mplib_lines_body.inc"` (types, PSVECNormalize, sqrtf, StageLineActive hook). */
#include "check_prelude.inc"

#define LINE_FLAG_EMPTY (1 << 7)
#define LINE_FLAG_PLATFORM (1 << 8)
#define F32_MAX 3.4028235e38f
typedef uint8_t u8;
enum { MapLineGroup_Floor, MapLineGroup_Ceiling, MapLineGroup_RightWall, MapLineGroup_LeftWall,
       MapLineGroup_Dynamic, MapLineGroup_Count };
enum {
    CollJoint_B8 = 1 << 8, CollJoint_B9 = 1 << 9, CollJoint_B10 = 1 << 10, CollJoint_B11 = 1 << 11,
    CollJoint_TooFar = 1 << 12, CollJoint_Enabled = 1 << 16, CollJoint_Hidden = 1 << 18,
};
typedef struct { s16 start, count; } MapLineRange;
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
static CollJoint* groundCollJoint;
static CollJoint* jointListStart;
static CollJoint* jointListEnd;
/* mpLib_804D64B4->joint_count (the prelude's MapCollData only names line_count) */
#define joint_count pad[0]
/* mpScriptIslandRefresh: logged in call order */
static char island_log[8192];
static int island_len;
static void mpScriptIslandRefresh(int joint, int start, int count, bool enabled)
{
    island_len += sprintf(island_log + island_len, " %d,%d,%d,%d", joint, start, count, enabled ? 1 : 0);
}
#include "mplib_geom_body.inc"
#include "mplib_joint_body.inc"

static u32 rng = 0x5eed1234u;
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

static int jix(CollJoint* j) { return j ? (int) (j - joints) : -1; }
static void dump(void)
{
    int i;
    printf("S %d %d |", jix(jointListStart), jix(jointListEnd));
    for (i = 0; i < nj; i++)
        printf(" %d %x %d %08x %08x %08x %08x", jix(joints[i].next), joints[i].flags, joints[i].xE,
               fbits(joints[i].bounding_min.x), fbits(joints[i].bounding_min.y),
               fbits(joints[i].bounding_max.x), fbits(joints[i].bounding_max.y));
    printf(" |");
    for (i = 0; i < nl; i++) printf(" %x", lines[i].flags);
    printf(" |");
    for (i = 0; i < nl; i++) printf(" %x", map[i].lo_flags);
    printf(" |");
    for (i = 0; i < nv; i++) printf(" %08x %08x", fbits(vtx[i].x10), fbits(vtx[i].x14));
    printf(" |%s\n", island_log);
    island_len = 0;
    island_log[0] = 0;
}

/* mpJointUpdateDynamics HSD_ASSERTs on a zero-length Dynamic line: skip those ops. */
static bool dyn_ok(int j)
{
    int i;
    for (i = 0; i < mj[j].ranges[MapLineGroup_Dynamic].count; i++) {
        MapLine* m = lines[mj[j].ranges[MapLineGroup_Dynamic].start + i].x0;
        if (vtx[m->v0_idx].pos.x == vtx[m->v1_idx].pos.x &&
            vtx[m->v0_idx].pos.y == vtx[m->v1_idx].pos.y)
            return false;
    }
    return true;
}

int main(void)
{
    static MapCollData mcd;
    int g, i, j, k;
    mpLib_804D64B4 = &mcd;
    groundCollVtx = vtx;
    groundCollLine = lines;
    groundCollJoint = joints;
    for (g = 0; g < 64; g++) {
        int perm[48];
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
            if (rnd(4) == 0) { vtx[i].pos.x = rf() * 10.0f; vtx[i].pos.y = rf() * 10.0f; }
            vtx[i].x0 = rf();
            vtx[i].x4 = rf();
            vtx[i].x10 = rf();
            vtx[i].x14 = rf();
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
                             (rnd(10) == 0 ? LINE_FLAG_EMPTY : 0) | (rnd(0x100) << 20);
            line_active_tab[i] = rnd(8) != 0;
            if (rnd(3) == 0) vtx[m->v1_idx].pos.y = vtx[m->v0_idx].pos.y;
            else if (rnd(4) == 0) vtx[m->v1_idx].pos.x = vtx[m->v0_idx].pos.x;
        }
        for (i = 0; i < nl; i++)
            if (lines[i].x0->next_id0 != -1) lines[lines[i].x0->next_id0].x0->prev_id0 = i;
        for (j = 0; j < nj; j++) {
            int s;
            for (k = 0; k < MapLineGroup_Count; k++) {
                s = rnd(nl);
                mj[j].ranges[k].start = s;
                mj[j].ranges[k].count = (k == MapLineGroup_Dynamic && rnd(2)) ? 0 : rnd(nl - s + 1);
            }
            s = rnd(nv);
            mj[j].vtx_start = s;
            mj[j].vtx_count = rnd(nv - s + 1);
            joints[j].inner = &mj[j];
            joints[j].next = NULL;
            joints[j].flags = (rnd(4) == 0 ? CollJoint_B8 : 0) | (rnd(5) == 0 ? CollJoint_B10 : 0) |
                              (rnd(4) == 0 ? CollJoint_B11 : 0) | (rnd(8) == 0 ? CollJoint_Hidden : 0) |
                              rnd(0x100);
            joints[j].xE = 0;
            joints[j].bounding_min.x = rf();
            joints[j].bounding_min.y = rf();
            joints[j].bounding_max.x = rf();
            joints[j].bounding_max.y = rf();
        }
        jointListStart = jointListEnd = NULL;
        printf("G %d %d %d\n", nv, nl, nj);
        for (i = 0; i < nv; i++)
            printf("V %08x %08x %08x %08x %08x %08x\n", fbits(vtx[i].x0), fbits(vtx[i].x4),
                   fbits(vtx[i].pos.x), fbits(vtx[i].pos.y), fbits(vtx[i].x10), fbits(vtx[i].x14));
        for (i = 0; i < nl; i++) {
            MapLine* m = lines[i].x0;
            printf("L %d %d %d %d %d %d %d %d %d %u %d\n", (int) (m - map), m->v0_idx, m->v1_idx,
                   m->prev_id0, m->next_id0, m->prev_id1, m->next_id1, m->hi_flags, m->lo_flags,
                   lines[i].flags, line_active_tab[i]);
        }
        for (j = 0; j < nj; j++) {
            printf("J %u %08x %08x %08x %08x %d %d", joints[j].flags, fbits(joints[j].bounding_min.x),
                   fbits(joints[j].bounding_min.y), fbits(joints[j].bounding_max.x),
                   fbits(joints[j].bounding_max.y), mj[j].vtx_start, mj[j].vtx_count);
            for (k = 0; k < MapLineGroup_Count; k++)
                printf(" %d %d", mj[j].ranges[k].start, mj[j].ranges[k].count);
            printf("\n");
        }
        for (i = 0; i < 80; i++) {
            int op = rnd(15), a = rnd(nj), l = rnd(nl);
            switch (op) {
            case 0: printf("O hide %d\n", a); mpJointHide(a); break;
            case 1: printf("O unhide %d\n", a); mpJointUnhide(a); break;
            case 2:
                if (!dyn_ok(a)) continue;
                printf("O dyn %d\n", a);
                mpJointUpdateDynamics(a);
                break;
            case 3:
                if (!dyn_ok(a)) continue;
                printf("O e24 %d\n", a);
                mpLib_80055E24(a);
                break;
            case 4: printf("O bound %d\n", a); mpJointUpdateBounding(a); break;
            case 5: printf("O 67c %d\n", a); mpLib_8005667C(a); break;
            case 6: printf("O 424 %d\n", a); mpLib_80057424(a); break;
            case 7: printf("O 528 %d\n", l); mpLib_80057528(l); break;
            case 8: printf("O 5b0 %d\n", l); mpLib_800575B0(l); break;
            case 9:
            case 10: printf("O add %d\n", a); mpJointListAdd(a); break;
            case 11: printf("O bc0 %d\n", a); mpLib_80057BC0(a); break;
            case 12: printf("O fdc %d\n", a); mpLib_80057FDC(a); break;
            case 13: printf("O 044 %d\n", a); mpLib_80058044(a); break;
            default: printf("O b10 %d\n", a); mpJointSetB10(a); break;
            }
            dump();
        }
        for (i = 0; i < nl; i++) printf("F %d %d\n", i, mpJointFromLine(i));
        for (i = 0; i < 160; i++) {
            int lid = rnd(nl);
            Vec3 pos, vo = { -9, -9, -9 }, n = { -9, -9, -9 };
            int lo = -5;
            u32 fl = 0xdead;
            f32 dist = rf() * (rnd(2) ? 4.0f : 0.5f), wmax = rnd(3) ? rf() * 3.0f : 1000.0f;
            int r;
            MapLine* m = lines[lid].x0;
            f32 t = (f32) rnd(1001) / 1000.0f;
            pos.x = vtx[m->v0_idx].pos.x + (vtx[m->v1_idx].pos.x - vtx[m->v0_idx].pos.x) * t +
                    (rnd(3) ? 0.0f : rf() * 0.1f);
            pos.y = vtx[m->v0_idx].pos.y + rf() * 0.2f;
            pos.z = rnd(2) ? 0.0f : rf();
            printf("W %d %08x %08x %08x %08x %08x\n", lid, fbits(pos.x), fbits(pos.y), fbits(pos.z),
                   fbits(dist), fbits(wmax));
            active_budget = 100000;
            if (setjmp(loop_jmp)) {
                printf("R LOOP\n");
                continue;
            }
            r = mpLib_80056C54(lid, &pos, &lo, &vo, &fl, &n, dist, wmax);
            printf("R %d %d %u %08x %08x %08x %08x %08x %08x\n", r, lo, fl, fbits(n.x), fbits(n.y),
                   fbits(n.z), fbits(vo.x), fbits(vo.y), fbits(vo.z));
        }
    }
    return 0;
}
