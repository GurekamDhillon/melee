/* Round 10 part of the melee_items C oracle; #included by oracle.c after round9.c and reusing
 * round 2's LCG / Item fill+snap, round 4's r4_pre / r4_h, round 7's vector draws and round 9's
 * seeded RNG. Checks src/itkinds10.rs (23 hand ports) against round10_bodies.h, verbatim copies
 * of the decomp C extracted by gen_round10.py. The call is always sequenced before r10_h (the
 * MSVC-ABI target evaluates call arguments right to left). Built like the rest: contraction on + gwtool
 * lowering (gen_expected.sh).
 *
 * The callees are the game's own: src/melee/lb/lbtrigf.c is compiled into this TU, the lbvector.c
 * functions reached and mtx_pc.c's PSVECCrossProduct are verbatim copies (round10_callees.h, also
 * written by gen_round10.py), and sinf / cosf are pc/gameworld/gekko_fp.c's (round7.c's r7_gk_*
 * copies). The trig names are renamed so they do not collide with the CRT's declarations. */
#define sinf r7_gk_sinf
#define cosf r7_gk_cosf
#define atanf r10_atanf
#define atan2f r10_atan2f
#define acosf r10_acosf
#define asinf r10_asinf
#include "melee/lb/lbtrigf.c"
float MSL_TrigF_80400774[] = { INFINITY }; /* MSL float.c: 0x7F800000 */
#include "melee/lb/lbvector.h"
#include "round10_callees.h"

/* itcoll.c */
const Quaternion it_803B8560 = { 0.0f, 0.0f, 1.0f, 0.0f };
/* ithammer.c (file-local type) */
typedef struct itHammerData {
    u32 x0;
    u32 x4;
    f32 x8;
} itHammerData;

static ftCommonData r10_cd;
static ftCommonData* r10_cd_p = &r10_cd;
#define p_ftCommonData r10_cd_p
#define it_804D6D28 it_804D6D28_o
#include "round10_bodies.h"
#undef it_804D6D28
#undef p_ftCommonData

static u32 r10_hammer[3];
/* extra Item fields the round-10 functions write (snap() does not cover all of them) */
static u32 r10_h(Item* ip)
{
    u32 h = H(H(H(0, r7_v(&ip->x7C)), r7_v(&ip->x88)), r7_v(&ip->x94));
    h = H(H(h, r7_v(&ip->xA0)), r7_v(&ip->x40_vel));
    h = H(H(H(H(h, bits(ip->xBEC.top)), bits(ip->xBEC.bottom)), bits(ip->xBEC.right)), bits(ip->xBEC.left));
    h = H(H(H(H(h, (u32) ip->xC30), ip->xD58), ip->xD5C), bits(ip->facing_dir));
    return H(h, r4_h(ip));
}
/* collision state: random env flags, the four surface normals and wall indices */
static void r10_coll(Item* ip)
{
    CollData* c = &ip->x378_itemColl;
    c->env_flags = (s32) (R() >> 8);
    r7_vin(&c->floor.normal); r7_vin(&c->ceiling.normal);
    r7_vin(&c->left_facing_wall.normal); r7_vin(&c->right_facing_wall.normal);
    c->left_facing_wall.index = (s32) ((R() >> 16) % 8);
    c->right_facing_wall.index = (s32) ((R() >> 16) % 8);
}
/* a moderate angle (|a| < 16) */
static f32 r10_a(void) { return RM() / 8.0f; }

/* it_8027770C's dot sign: vel / wall normal nearly orthogonal (n.z solves the dot to ~0), kept
 * only when the shipping (fused) sign differs from un-fusing the outer or the inner site. Volatile
 * temporaries keep this test itself uncontracted. */
static void r10_dot_pick(Vec3* v, Vec3* n)
{
    for (;;) {
        volatile f32 px, py, pz, inner, inner2, a, b, nz;
        volatile f64 d;
        f32 fused;
        v->x = RM(); v->y = RM(); v->z = RM(); n->x = RM(); n->y = RM();
        if (v->z == 0.0f) continue;
        px = v->x * n->x; py = v->y * n->y;
        nz = px + py; nz = -nz; nz = nz / v->z; n->z = nz;
        d = (f64) v->x * (f64) n->x; d = d + (f64) py; inner = (f32) d;
        d = (f64) v->z * (f64) n->z; d = d + (f64) inner; fused = (f32) d;
        pz = v->z * n->z; a = pz + inner;
        inner2 = px + py; d = (f64) v->z * (f64) n->z; d = d + (f64) inner2; b = (f32) d;
        if ((fused < 0.0f) != (a < 0.0f) || (fused < 0.0f) != (b < 0.0f)) return;
    }
}

static void round10(void)
{
    T(3001, "R10_80271830", { f32 a; r4_pre(ip); ip->xBEC.top = r7_c(); ip->xBEC.bottom = r7_c(); ip->xBEC.right = r7_c(); ip->xBEC.left = r7_c(); a = r10_a(); it_80271830(ip, a); ret = r10_h(ip); });
    T(3002, "R10_8027649C", { r4_pre(ip); r10_coll(ip); ip->facing_dir = r7_c(); ret = bits(it_8027649C(g)); ret = H(ret, r10_h(ip)); });
    T(3003, "R10_8027649C_REF", { Vec3 a, b; f32 r; r4_pre(ip); r10_coll(ip); ip->facing_dir = r7_c(); r7_vin(&a); r7_vin(&b); r = it_8027649C_ref(g, &a, &b); ret = H(H(H(bits(r), r7_v(&a)), r7_v(&b)), r10_h(ip)); });
    T(3004, "R10_80277040", { r4_pre(ip); r10_coll(ip); r2_attr.x50 = ((R() >> 16) % 4) ? r7_c() : RM() / 16777216.0f; r7_vin(&ip->x88); r7_vin(&ip->xAC_unk); ip->xC2C = (s32) ((R() >> 16) % 3); ip->xC30 = (s32) ((R() >> 16) % 3); ip->xD58 = (R() >> 16) % 4; ip->xD5C = 0; r2_common.xC0 = RM() / 64.0f; r2_common.xC4 = RM() / 64.0f; r2_common.x38_float = (R() >> 16) % 4; ret = it_80277040(g); ret = H(ret, r10_h(ip)); });
    T(3005, "R10_8027770C", { r4_pre(ip); r10_coll(ip); r7_vin(&ip->x40_vel); r2_attr.x58 = r7_c(); ret = it_8027770C(g); ret = H(ret, r10_h(ip)); });
    T(3006, "R10_8027B798", { u32 s; r4_pre(ip); r10_coll(ip); memset(&r10_cd, 0, sizeof r10_cd); r10_cd.x100 = RM() / 64.0f; r10_cd.x144_radians = r10_a(); r10_cd.x148 = RM(); r10_cd.x14C = RM(); r10_cd.x150 = RM(); s = (R() >> 16) % 4; ip->xCAC_angle = s == 0 ? 0x169 : s == 1 ? (s32) ((R() >> 16) % 400) : 0x169; ip->xCC8_knockback = r7_c(); ret = it_8027B798(g, &ov); ret = H(ret, r10_h(ip)); });
    T(3007, "R10_8027BB1C", { Vec3 v, a; r7_vin(&v); r7_vin(&a); it_8027BB1C(&v, &a); ret = r7_v(&v); });
    T(3024, "R10_8027770C_DOT_SIGN", { Vec3 n; r4_pre(ip); r10_coll(ip); ip->x378_itemColl.env_flags = Collide_LeftWallMask; r10_dot_pick(&ip->x40_vel, &n); ip->x378_itemColl.left_facing_wall.normal = n; r2_attr.x58 = 1.0f; ret = it_8027770C(g); ret = H(ret, r10_h(ip)); });
    T(3025, "R10_802E628C_UNIT_FACING", { f32 a8, a9; r4_pre(ip); ip->facing_dir = (R() & 1) ? 1.0f : -1.0f; memset(&r9_2e5a, 0, sizeof r9_2e5a); r9_2e5a.xC = RM(); r9_2e5a.x10 = RM(); r9_2e5a.x18 = RM(); r9_2e5a.x20 = RM(); r2_art.x4_specialAttributes = &r9_2e5a; a8 = ((R() >> 16) % 4) ? (f32) ((R() >> 16) % 7000) / 1000.0f : r10_a(); a9 = r7_c(); SEED(); it_802E628C(g, a8, a9); ret = SEED_H(r10_h(ip)); });
    T(3008, "R10_802E628C", { f32 a8, a9; r4_pre(ip); memset(&r9_2e5a, 0, sizeof r9_2e5a); r9_2e5a.xC = RM(); r9_2e5a.x10 = RM(); r9_2e5a.x18 = RM(); r9_2e5a.x20 = RM(); r2_art.x4_specialAttributes = &r9_2e5a; a8 = ((R() >> 16) % 4) ? (f32) ((R() >> 16) % 7000) / 1000.0f : r10_a(); a9 = r7_c(); SEED(); it_802E628C(g, a8, a9); ret = SEED_H(r10_h(ip)); });
    T(3009, "R10_802863BC", { itBoxAttributes* ba; r4_pre(ip); r10_coll(ip); ba = (itBoxAttributes*) r4_sa; ba->bounce_angle_threshold = RM() / 32.0f; r7_vin(&ip->x7C); r7_vin(&ip->x88); r7_vin(&ip->x94); r7_vin(&ip->xA0); ret = it_802863BC(g); ret = H(ret, r10_h(ip)); });
    T(3010, "R10_FLIPPER_REPEL_INLINE", { Vec3 p; s32 k; r4_pre(ip); k = (R() & 1) ? It_Kind_Flipper : (s32) ((R() >> 16) % 40); r7_vin(&p); itFlipper_Repel_inline(g, k, &p, &ov); ret = r10_h(ip); });
    T(3011, "R10_FOXBLASTER_SETSHOTANGLE", { Vec3 a, b; r4_pre(ip); r7_vin(&a); r7_vin(&b); itFoxBlaster_SetShotAngle(ip, &a, &b); ret = r10_h(ip); });
    T(3012, "R10_80293DCC", { itHammerData* hd = (itHammerData*) r10_hammer; r4_pre(ip); hd->x0 = (u32) r9_i(); hd->x4 = (u32) r9_i(); hd->x8 = RM(); r2_art.x4_specialAttributes = hd; SEED(); ret = SEED_H(it_80293DCC(g)); });
    T(3013, "R10_802D61C8", { s32 c; r4_pre(ip); r10_coll(ip); ip->xDD4_itemVar.hinoarashi.x78 = r10_a(); c = (s32) ((R() >> 16) % 16); it_802D61C8(g, c); ret = r10_h(ip); });
    T(3014, "R10_802D4494", { itHitodemanAttributes* ha; r4_pre(ip); ha = (itHitodemanAttributes*) r4_sa; ha->x14 = RM(); ha->x18 = RM(); SEED(); it_802D4494(g); ret = SEED_H(r10_h(ip)); });
    T(3015, "R10_802D4510", { itHitodemanAttributes* ha; r4_pre(ip); ha = (itHitodemanAttributes*) r4_sa; ha->x40 = r9_i(); ha->x44 = r9_i(); SEED(); it_802D4510(g); ret = SEED_H(r10_h(ip)); });
    T(3016, "R10_KOOPAFLAME_UNKMOTION0_PHYS", { r4_pre(ip); itKoopaFlame_UnkMotion0_Phys(g); ret = r10_h(ip); });
    T(3017, "R10_LGUNBEAM_UNKMOTION0_PHYS", { r4_pre(ip); itLgunbeam_UnkMotion0_Phys(g); ret = r10_h(ip); });
    T(3018, "R10_802A2320", { r4_pre(ip); r7_vin(&ip->x40_vel); r7_vin(&ip->xC58); ip->facing_dir = (R() & 1) ? 1.0f : r7_c(); ret = it_802A2320(g); ret = H(ret, r10_h(ip)); });
    T(3019, "R10_SHADOWBALL_SHIELDBOUNCED", { r4_pre(ip); r7_vin(&ip->x40_vel); r7_vin(&ip->xC58); ret = it_2725_Logic101_ShieldBounced(g); ret = H(ret, r10_h(ip)); });
    T(3020, "R10_802D6740", { r4_pre(ip); it_802D6740(g); ret = r10_h(ip); });
    T(3021, "R10_PIKACHUTJOLTGROUND_UNKMOTION0_PHYS", { r4_pre(ip); itPikachutjoltground_UnkMotion0_Phys(g); ret = r10_h(ip); });
    T(3022, "R10_SAMUSCHARGESHOT_UNKMOTION8_PHYS", { r4_pre(ip); itSamuschargeshot_UnkMotion8_Phys(g); ret = r10_h(ip); });
    T(3023, "R10_802EC870", { itTincleAttributes* ta; s32 a1; r4_pre(ip); ta = (itTincleAttributes*) r4_sa; ta->x3C = r9_i(); a1 = ((R() >> 16) % 4) ? 0 : 1; SEED(); it_802EC870(g, a1); ret = SEED_H(r10_h(ip)); });
}
