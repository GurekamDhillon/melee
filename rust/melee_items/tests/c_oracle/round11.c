/* Round 11 (night 3, round 3) part of the melee_items C oracle; #included by oracle.c after
 * round10.c and reusing round 2's LCG / Item fill+snap, round 7's vector draws and round 9's
 * seeded RNG (the game's random.c). Checks src/itkinds11.rs (6 hand ports) against
 * round11_bodies.h, verbatim copies of the decomp C extracted by gen_round11.py. jobj.h's inline
 * getters and MSL math_ppc.h's sqrtf_accurate come from the game headers. Built like the rest:
 * contraction on + gwtool lowering (gen_expected.sh). */
#include <sysdolphin/baselib/mtx.h> /* VEC2_SQ_LEN */
#define it_804D6D28 it_804D6D28_o
#define HSD_GObjPLinkHead r3_plink
#define Item_804A0CCC r11_track
static Item_FtTrack r11_track;
#include "round11_bodies.h"
#undef Item_804A0CCC
#undef HSD_GObjPLinkHead
#undef it_804D6D28

/* A chain of 4 JObjs (r11_j[0] the root, each the next one's parent via `child`). */
static HSD_JObj r11_j[4];
static ItemModelDesc r11_md;
static DynamicBoneTable r11_bt;
static void r11_jobjs(HSD_GObj* g)
{
    int k;
    memset(r11_j, 0, sizeof r11_j);
    for (k = 0; k < 4; k++) {
        r11_j[k].child = k < 3 ? &r11_j[k + 1] : NULL;
        r11_j[k].rotate.x = r7_c(); r11_j[k].rotate.y = r7_c(); r11_j[k].rotate.z = r7_c(); r11_j[k].rotate.w = r7_c();
        r11_j[k].translate.x = RM(); r11_j[k].translate.y = RM(); r11_j[k].translate.z = RM();
    }
    g->hsd_obj = &r11_j[0];
}
static u32 r11_jidx(HSD_JObj* j)
{
    int k;
    if (j == NULL) return 4;
    for (k = 0; k < 4; k++) if (j == &r11_j[k]) return (u32) k;
    return 5;
}
/* the item model description (bone count 0 half the time) and a bone table of chain JObjs */
static void r11_model(Item* ip)
{
    int k;
    memset(&r11_md, 0, sizeof r11_md);
    r11_md.x4_bone_count = (R() & 1) ? 0 : (R() >> 16) % 8 + 1;
    r11_md.xC_bit_field = (u8) (R() >> 8);
    r2_art.x10_modelDesc = &r11_md;
    for (k = 0; k < 8; k++) r11_bt.bones[k] = (R() % 5) ? &r11_j[R() % 4] : NULL;
    ip->xBBC_dynamicBoneTable = &r11_bt;
}
/* a small ECB (|x| < 16) */
/* ECBs with top/left >= 0 and bottom/right <= 0, so overlaps happen often */
static f32 r11_abs(f32 x) { return x < 0 ? -x : x; }
static void r11_ecb(itECB* e) { e->top = r11_abs(RM()) / 8.0f; e->bottom = -(r11_abs(RM()) / 8.0f); e->right = -(r11_abs(RM()) / 8.0f); e->left = r11_abs(RM()) / 8.0f; }
/* the sign bit of x70_nudge.x also goes in as bit 0: H's xor-multiply only carries bit 31
 * into bit 31, so a sign flip here cancelled against the same flip in the field snapshot */
static u32 r11_h(Item* ip) { return H(H(H(bits(ip->x70_nudge.x), ip->xDC0), bits(ip->x70_nudge.y)), bits(ip->x70_nudge.x) >> 31); }

/* the other two items of the p-link list for it_80271F78 */
static HSD_GObj r11_gl[3];
static Item r11_li[2];
static ItemAttr r11_la[2];
static HSD_JObj r11_lj[2];
static HSD_GObj* r11_heads[16];
static void r11_list(Item* ip, HSD_GObj* g)
{
    u32 k, n = R() % 4;
    memset(r11_gl, 0, sizeof r11_gl);
    for (k = 0; k < 2; k++) {
        Item* li = &r11_li[k];
        memset(li, 0, sizeof *li); memset(&r11_la[k], 0, sizeof r11_la[k]); memset(&r11_lj[k], 0, sizeof r11_lj[k]);
        flag32_set(&li->xDC8_word, R());
        li->ground_or_air = (R() & 3) ? GA_Ground : GA_Air;
        li->xDD1_flag.b0 = (R() & 3) == 0;
        r11_la[k].x0_is_heavy = (R() & 3) != 0;
        li->xCC_item_attr = &r11_la[k];
        li->x70_nudge.x = r7_c();
        r11_ecb(&li->xBEC);
        r11_lj[k].translate.x = RM() / 4.0f; r11_lj[k].translate.y = RM() / 4.0f; r11_lj[k].translate.z = RM();
        r11_gl[k + 1].user_data = li; r11_gl[k + 1].hsd_obj = &r11_lj[k];
    }
    r11_ecb(&ip->xBEC);
    r11_j[0].translate.x = RM() / 4.0f; r11_j[0].translate.y = RM() / 4.0f;
    if ((R() & 1) == 0) {
        /* item 0 at the same spot and eligible (grounded heavy, x13 and xDD1.b0 clear) */
        r11_lj[0].translate.x = r11_j[0].translate.x; r11_lj[0].translate.y = r11_j[0].translate.y;
        while (r11_li[0].xDC8_word.flags.x13) flag32_set(&r11_li[0].xDC8_word, R());
        r11_li[0].ground_or_air = GA_Ground; r11_li[0].xDD1_flag.b0 = 0; r11_la[0].x0_is_heavy = 1;
    }
    /* the case's own GObj is first on the list (or absent), then the two others */
    r11_gl[0] = *g;
    for (k = 0; k + 1 < 3; k++) r11_gl[k].next = &r11_gl[k + 1];
    memset(r11_heads, 0, sizeof r11_heads);
    r11_heads[HSD_GOBJ_PLINK_ITEM] = n == 0 ? NULL : (n == 1 ? &r11_gl[1] : (n == 2 ? g : &r11_gl[0]));
    if (n == 2) g->next = &r11_gl[1];
    r3_plink = r11_heads;
}
static void r11_trk(Item* ip)
{
    u32 k;
    memset(&r11_track, 0, sizeof r11_track);
    r11_track.count = (R() >> 16) % 12;
    r11_track.x154.b0 = (R() & 3) == 0;
    for (k = 0; k < 11; k++) {
        r11_ecb(&r11_track.ecb_offset_arr[k]);
        r11_track.ft_pos_arr[k].x = RM() / 4.0f; r11_track.ft_pos_arr[k].y = RM() / 4.0f; r11_track.ft_pos_arr[k].z = RM();
    }
    r11_ecb(&ip->xBEC);
    r11_j[0].translate.x = RM() / 4.0f; r11_j[0].translate.y = RM() / 4.0f;
    if ((R() & 1) == 0) { u32 i = (R() & 1) && r11_track.count ? (u32) r11_track.count - 1u : 0u; /* often the last one, so not overwritten */ r11_track.ft_pos_arr[i].x = r11_j[0].translate.x; r11_track.ft_pos_arr[i].y = r11_j[0].translate.y; }
}

static void round11(void)
{
    T(3101, "R11_80272CC0", { s32 idx; r11_jobjs(g); r11_model(ip); idx = (s32) ((R() >> 16) % 6); ret = r11_jidx(it_80272CC0(g, idx)); });
    T(3102, "R11_80274990", { r11_jobjs(g); r11_model(ip); ret = bits(it_80274990(g)); });
    T(3103, "R11_80271B60", { r11_jobjs(g); r2_common.x7C_float = RM(); r11_trk(ip); SEED(); it_80271B60(g); ret = SEED_H(r11_h(ip)); });
    T(3104, "R11_80271F78", { r11_jobjs(g); r2_common.x7C_float = RM(); r11_list(ip, g); SEED(); it_80271F78(g); ret = SEED_H(r11_h(ip)); });
    T(3105, "R11_LINKARROW_802A850C_INLINE_2", { Quaternion q; r4_pre(ip); r11_jobjs(g); r7_vin(&ip->xDD4_itemVar.linkarrow.x18); itLinkArrow_802A850C_inline_2(g, &q); ret = H(H(H(H(r4_h(ip), bits(q.x)), bits(q.y)), bits(q.z)), bits(q.w)); });
    T(3106, "R11_CALC_DIST_2D_ACCURATE", { Vec3 v; f64 d; u64 u; r7_vin(&v); d = calc_dist_2d_accurate(&v); memcpy(&u, &d, 8); ret = H(H(0, (u32) u), (u32) (u >> 32)); });
    /* 3107: only inputs where the fused SQ(x) + SQ(y) differs from two f32 roundings */
    T(3107, "R11_CALC_DIST_2D_ACCURATE_FUSED", { Vec3 v; f64 d; u64 u; s32 k; for (k = 0; k < 4000; k++) { volatile f32 yy, xx; f64 dx; r7_vin(&v); yy = v.y * v.y; xx = v.x * v.x; dx = (f64) v.x * (f64) v.x; if ((f32) (dx + (f64) yy) != (f32) (xx + yy)) break; } d = calc_dist_2d_accurate(&v); memcpy(&u, &d, 8); ret = H(H((u32) k, (u32) u), (u32) (u >> 32)); });
}

/* baselib debug.c's __assert (jobj.h's getters assert a non-null JObj; never hit by these cases) */
void __assert(const char* file, u32 line, const char* cond)
{
    fprintf(stderr, "assert %s:%u %s\n", file, (unsigned) line, cond);
    abort();
}
