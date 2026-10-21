/* Round 7 (night 5) part of the melee_items C oracle; #included by oracle.c after round6.c and
 * reusing round 2's LCG / Item fill+snap and round 4's r4_pre / r4_h. Checks src/itfwd7.rs (one
 * generated forwarder) and src/itkinds7.rs (hand ports): round7_bodies.h holds verbatim copies of
 * the decomp C, extracted by gen_round7.py. Built like the rest: contraction on + gwtool lowering
 * (gen_expected.sh). Case draws use the LCG's high bits (R() >> 16).
 *
 * sqrtf: the item TUs get MSL's math_ppc.h `extern inline sqrtf` (frsqrte + 3 Newton steps), not
 * the CRT's. This file includes math_ppc.h and pc/gameworld/gekko_fp.c's bit-exact __frsqrte (its
 * sinf/cosf/tanf/__fres renamed away so no other round changes). */
#define sinf r7_gk_sinf
#define cosf r7_gk_cosf
#define tanf r7_gk_tanf
#define __fres r7_gk_fres
#include "gameworld/gekko_fp.c"
#undef sinf
#undef cosf
#undef tanf
#undef __fres
#include "MSL/math_ppc.h"
#include "melee/ft/types.h"
#include "melee/mp/types.h"
#include <dolphin/mtx.h>

#include "round7_bodies.h"

static u32 r7_v(const Vec3* v) { return H(H(H(0, bits(v->x)), bits(v->y)), bits(v->z)); }
static u32 r7_bitsd(f64 d) { u64 u; memcpy(&u, &d, 8); return H((u32) u, (u32) (u >> 32)); }
/* a vector component: mostly moderate (|x| < 128), sometimes any RF() value, sometimes 0 */
static f32 r7_c(void)
{
    u32 s = (R() >> 16) % 8;
    if (s == 0) return 0.0f;
    return s < 3 ? RF() : RM();
}
static void r7_vin(Vec3* v) { v->x = r7_c(); v->y = r7_c(); v->z = r7_c(); }
static Fighter r7_fp;
static CollData r7_coll;
static u16 r7_tab[33];
static u32 r7_spawn_h(const SpawnItem* s)
{
    u32 h = H(H(0, (u32) s->kind), r7_v(&s->pos));
    h = H(H(h, r7_v(&s->prev_pos)), r7_v(&s->vel));
    h = H(H(h, (u32) (s32) s->x3C_damage), (u32) s->x40);
    h = H(H(h, PIDX(s->x0_parent_gobj)), PIDX(s->x4_parent_gobj2));
    return H(h, (u32) s->x44_flag.byte); /* x86 bit order; the Rust side reverses the byte */
}

static void round7(void)
{
#include "round7_fwd.inc"
    /* ---- hand ports (src/itkinds7.rs) */
    T(2401, "R7H_802A3C98", { Vec3 a, b; f32 l; r7_vin(&a); r7_vin(&b); if (((R() >> 16) & 7) == 0) b = a; l = it_802A3C98(&a, &b, &ov); ret = bits(l); });
    T(2402, "R7H_NORMALIZE_DIFF", { Vec3 a, b; f64 l; r7_vin(&a); r7_vin(&b); if (((R() >> 16) & 7) == 0) b = a; l = it_802A6A78_normalize_diff(&a, &b, &ov); ret = r7_bitsd(l); });
    T(2403, "R7H_NORMALIZE_DIFF_REV", { Vec3 a, b; f64 l; r7_vin(&a); r7_vin(&b); if (((R() >> 16) & 7) == 0) b = a; l = it_802A6A78_normalize_diff_rev(&a, &b, &ov); ret = r7_bitsd(l); });
    T(2404, "R7H_VEC3_EQ_MTX", { Mtx m; int i, j; for (i = 0; i < 3; i++) for (j = 0; j < 4; j++) m[i][j] = RF(); vec3_eq_mtx(&ov, m); });
    T(2405, "R7H_LINK_FIGHTER_COMPARE", { static const s32 ids[] = { 0x168, 0x169, 0xD4, 0xD6, 0x167, 0x16A, 0xD5, 0xD7 }; memset(&r7_fp, 0, sizeof r7_fp); r7_fp.motion_id = (R() >> 16) & 1 ? ids[(R() >> 16) % 8] : (s32) R(); ret = (u32) link_fighter_compare(&r7_fp); });
    T(2406, "R7H_2F28_M0_PHYS", { r4_pre(ip); if ((R() >> 16) & 1) { ip->x40_vel.x = RM(); ip->x40_vel.y = RM(); ip->xDD4_itemVar.unk2.x0 = RM() / 16.0f; } it_2F28_UnkMotion0_Phys(g); ret = H(ret, r4_h(ip)); });
    T(2407, "R7H_BISECTVALUE", { int n = 1 + (int) ((R() >> 16) % 32); int k, acc = 0; s32 v; for (k = 0; k <= n; k++) { r7_tab[k] = (u16) acc; acc += (int) ((R() >> 16) % 50); } v = (s32) ((R() >> 16) % (u32) (acc + 1)); { ItemPickTable t; memset(&t, 0, sizeof t); t.xC = r7_tab; ret = (u32) bisectValue(v, &t, 0, n); } ret = H(ret, (u32) n); });
    T(2408, "R7H_802E614C", { SpawnItem s; Vec3 p, v; HSD_GObj* a = PTR(R()); HSD_GObj* b = PTR(R()); memset(&s, 0, sizeof s); s.kind = (ItemKind) R(); s.x3C_damage = (s16) R(); s.x40 = (s32) R(); s.x44_flag.byte = (u8) (R() >> 16); p.x = RF(); p.y = RF(); p.z = RF(); v.x = RF(); v.y = RF(); v.z = RF(); it_802E614C(a, b, &s, &p, &v); ret = r7_spawn_h(&s); });
    T(2409, "R7H_TARUCANN_M9_ANIM_INLINE", { r4_pre(ip); if ((R() >> 16) & 1) ip->xDD4_itemVar.tarucann.x20 = 0; itTarucann_UnkMotion9_Anim_inline(g); ret = H(ret, r4_h(ip)); });
    T(2410, "R7H_KYASARIN_SETFALLCOLLPOS", { memset(&r7_coll, 0, sizeof r7_coll); r7_coll.cur_pos.x = RF(); r7_coll.cur_pos.y = RF(); r7_coll.cur_pos.z = RF(); itKyasarin_SetFallCollPos(ip, &r7_coll, &ov); ret = r7_v(&r7_coll.cur_pos); });
    T(2411, "R7H_ARWINGLASER_GETCOLLARG", { r4_pre(ip); ret = (u32) itArwinglaser_GetCollArg(ip); ret = H(ret, r4_h(ip)); });
    T(2412, "R7H_GETRANDMAX", { r4_pre(ip); ret = getRandMax(ip->xC4_article_data); ret = H(ret, r4_h(ip)); });
}
