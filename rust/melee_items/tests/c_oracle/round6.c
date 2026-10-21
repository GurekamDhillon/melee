/* Round 6 (night 4) part of the melee_items C oracle; #included by oracle.c after round5.c and
 * reusing round 2's LCG / Item fill+snap and round 4's r4_pre / r4_h. Checks src/itfwd6.rs
 * (generated forwarders and `return true;`/`return;` callbacks) and src/itkinds6.rs (hand
 * ports): round6_bodies.h holds verbatim copies of the decomp C, extracted by gen_round6.py.
 * Built like the rest: contraction on + gwtool lowering (gen_expected.sh). The case draws of
 * this round use the LCG's high bits (R() >> 16): its low bits cycle (round 5's trap).
 * Item-var bitfield bytes stay in x86 order here; the Rust side bit-reverses them around the
 * call (as rounds 4 and 5). */
#include "melee/it/kinds/itfoxblaster.h"
#undef it_804D6D28
#define it_804D6D28 it_804D6D28_o

#include "round6_bodies.h"

static u32 r6_sa_h(int n)
{
    u32 h = 0;
    int k;
    for (k = 0; k < n; k++) h = H(h, r4_sa[k]);
    return h;
}
static u32 r6_v(const Vec3* v) { return H(H(H(0, bits(v->x)), bits(v->y)), bits(v->z)); }
static ItemLink r6_link;
static void r6_link_in(void)
{
    memset(&r6_link, 0, sizeof r6_link);
    r6_link.pos.x = RF(); r6_link.pos.y = RF(); r6_link.pos.z = RF();
    r6_link.vel.x = RF(); r6_link.vel.y = RF(); r6_link.vel.z = RF();
    r6_link.coll_data.cur_pos.x = RF(); r6_link.coll_data.cur_pos.y = RF(); r6_link.coll_data.cur_pos.z = RF();
    r6_link.coll_data.last_pos.x = RF(); r6_link.coll_data.last_pos.y = RF(); r6_link.coll_data.last_pos.z = RF();
}
static u32 r6_link_h(void)
{
    return H(H(H(r6_v(&r6_link.pos), r6_v(&r6_link.vel)), r6_v(&r6_link.coll_data.cur_pos)),
             r6_v(&r6_link.coll_data.last_pos));
}
static u32 r6_hurt0(Item* ip) { return HURT_H(ip->xACC_itemHurtbox[0]); }
/* moderate inputs (|x| < 128) for the fused sites: the random tables clamp most of them */
#define SAF(k) ((f32*) r4_sa)[k]

static void round6(void)
{
#include "round6_fwd.inc"
    /* ---- hand ports (src/itkinds6.rs) */
    T(2001, "R6H_8027574C", { r4_pre(ip); it_8027574C(g); ret = H(H(ret, bits(ip->xB54.unk_x)), bits(ip->xB54.unk_y)); ret = H(ret, r4_h(ip)); });
    T(2002, "R6H_802762B0", { r4_pre(ip); it_802762B0(ip); ret = H(ret, r4_h(ip)); });
    T(2003, "R6H_CHECK_KIND", { r4_pre(ip); { s32 k = (s32) ((R() >> 16) % 300) - 10; ret = (u32) it_8026F3D4_check_kind(ip, (ItemKind) k); } ret = H(ret, r4_h(ip)); });
    T(2004, "R6H_2725_LOGIC70_EVTUNK", { r4_pre(ip); { HSD_GObj* r = PTR(R()); if ((R() >> 16) & 1) ip->xDD4_itemVar.climbersstring.xC = r; it_2725_Logic70_EvtUnk(g, r); } ret = H(ret, r4_h(ip)); });
    T(2005, "R6H_2725_LOGIC59_EVTUNK", { r4_pre(ip); it_2725_Logic59_EvtUnk(g, PTR(R())); ret = H(ret, r4_h(ip)); });
    T(2006, "R6H_CLINKMILK_DESTROY", { r4_pre(ip); ret = (u32) itCLinkMilk_Destroy(g); ret = H(ret, r4_h(ip)); });
    T(2007, "R6H_NESSBAT_REMOVEITEM", { r4_pre(ip); ret = (u32) itNessbat_RemoveItem(g); ret = H(ret, r4_h(ip)); });
    T(2008, "R6H_802DB358", { r4_pre(ip); ip->grab_victim = PTR(R()); it_802DB358(g); ret = PIDX(ip->grab_victim); ip->xDD4_itemVar.likelike.x50 = (HSD_GObj*) (uintptr_t) PIDX(ip->xDD4_itemVar.likelike.x50); /* pointer -> index before hashing */ ret = H(ret, r4_h(ip)); });
    T(2009, "R6H_802AE608", { r4_pre(ip); it_802AE608(g); ret = H(ret, r4_h(ip)); });
    T(2010, "R6H_FOXBLASTER_SETSHOTOFFSET", { r4_pre(ip); { Vec3 p; p.x = RF(); p.y = RF(); p.z = RF(); itFoxBlaster_SetShotOffset(ip, &p, &ov); } ret = H(ret, r4_h(ip)); });
    T(2011, "R6H_FOXBLASTER_SETCOMMANDVARS", { r4_pre(ip); itFoxBlaster_SetCommandVars(ip, (s32) R()); ret = H(ret, r4_h(ip)); });
    T(2012, "R6H_FOXBLASTER_CLEARSHOT", { r4_pre(ip); itFoxBlaster_ClearShot(g, (s32) ((R() >> 16) % 6)); ret = H(ret, r4_h(ip)); });
    T(2013, "R6H_PKTHUNDER_SHIFTANGLES", { r4_pre(ip); itNesspkthunderball_ShiftAngles(ip); ret = H(ret, r4_h(ip)); });
    T(2014, "R6H_PKTHUNDER_SHIFTPOSITIONS", { r4_pre(ip); itNesspkthunderball_ShiftPositions(ip); ret = H(ret, r4_h(ip)); });
    T(2015, "R6H_DRMARIOPILL_M0_PHYS", { r4_pre(ip); r2_common.x68_float = RM(); itDrMarioPill_UnkMotion0_Phys(g); ret = H(ret, r4_h(ip)); });
    T(2016, "R6H_TOGEPY_M6_PHYS", { r4_pre(ip); itTogepy_UnkMotion6_Phys(g); ret = H(ret, r4_h(ip)); });
    T(2017, "R6H_TOOLS_M4_PHYS", { r4_pre(ip); ip->xDD4_itemVar.tools.x0 = (s32) ((R() >> 16) % 4); itTools_UnkMotion4_Phys(g); ret = H(ret, r4_h(ip)); });
    T(2018, "R6H_HASSAM_M1_PHYS", { r4_pre(ip); itHassam_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(2019, "R6H_8028EDBC", { r4_pre(ip); it_8028EDBC(g); ret = H(ret, r4_h(ip)); });
    T(2020, "R6H_802874F0", { r4_pre(ip); it_802874F0(g); ret = H(ret, r4_h(ip)); });
    T(2021, "R6H_802962E0", { r4_pre(ip); it_802962E0(g); ret = H(ret, r4_h(ip)); });
    T(2022, "R6H_2725_LOGIC101_REFLECTED", { r4_pre(ip); ip->xDD4_itemVar.mewtwoshadowball.x4.x = RM(); ret = (u32) it_2725_Logic101_Reflected(g); ret = H(ret, r4_h(ip)); });
    T(2023, "R6H_MARIL_M1_COLL_INLINE", { r4_pre(ip); itMaril_UnkMotion1_Coll_inline(g); ret = H(ret, r4_h(ip)); });
    T(2024, "R6H_80291DAC_LEVEL", { r4_pre(ip); r4_sa[1] = (R() >> 16) % 40; r4_sa[2] = (R() >> 16) % 12; ret = (u32) it_80291DAC_level(g, (s32) ((R() >> 16) % 110) - 10); ret = H(ret, r4_h(ip)); });
    T(2025, "R6H_802944AC", { r4_pre(ip); { ftCollisionBox b; memset(&b, 0, sizeof b); it_802944AC(g, &b); ret = H(H(H(H(H(H(ret, bits(b.top)), bits(b.bottom)), bits(b.left.x)), bits(b.left.y)), bits(b.right.x)), bits(b.right.y)); } ret = H(ret, r4_h(ip)); });
    T(2026, "R6H_SWORD_INLINEA1", { r4_pre(ip); inlineA1(g); ret = H(ret, r4_h(ip)); });
    T(2027, "R6H_SWORD_INLINEA2", { r4_pre(ip); inlineA2(g); ret = H(ret, r4_h(ip)); });
    T(2028, "R6H_SWORD_INLINEC1", { r4_pre(ip); inlineC1(g); ret = H(ret, r4_h(ip)); });
    T(2029, "R6H_SWORD_INLINEA3", { r4_pre(ip); ((itSword_UnkArticle1*) r4_sa)->x18 = (s32) ((R() >> 16) % 9); inlineA3(g); ret = H(ret, r4_h(ip)); });
    T(2030, "R6H_SWORD_INLINED1", { r4_pre(ip); ((itSword_UnkArticle1*) r4_sa)->x18 = (s32) ((R() >> 16) % 9); inlineD1(g); ret = H(ret, r4_h(ip)); });
    T(2031, "R6H_802DDBE8", { r4_pre(ip); it_802DDBE8(g); ret = H(ret, r4_h(ip)); });
    T(2032, "R6H_ZRSHELL_M3_ANIM", { r4_pre(ip); ret = (u32) itZrshell_UnkMotion3_Anim(g); ret = H(ret, r4_h(ip)); });
    T(2033, "R6H_802DFF14", { r4_pre(ip); it_802DFF14(g, (s32) ((R() >> 16) & 1)); ret = H(ret, r6_hurt0(ip)); ret = H(ret, r4_h(ip)); });
    T(2034, "R6H_802EA988", { r4_pre(ip); { s32 e = (s32) (R() >> 16); ip->x378_itemColl.env_flags = (R() >> 16) & 1 ? e : e * 64; } { f32 a = RF(); f32 b = RF(); ret = (u32) it_802EA988(g, a, b); } ret = H(ret, r4_h(ip)); });
    T(2035, "R6H_802A43B8", { r6_link_in(); it_802A43B8(&r6_link); ret = r6_link_h(); });
    T(2036, "R6H_802A43EC", { r6_link_in(); it_802A43EC(&r6_link); ret = r6_link_h(); });
    T(2037, "R6H_802A4420", { r6_link_in(); it_802A4420(&r6_link); ret = r6_link_h(); });
    T(2038, "R6H_TEST_COMP", { Vec3 a, b; f32 k; a.x = RF(); a.y = RF(); a.z = RF(); b.x = RF(); b.y = RF(); b.z = RF(); k = RF(); test_comp(&ov, &a, &b, &k); });
    T(2039, "R6H_LINK_ATTR_MATH", { r4_pre(ip); r4_sa[3] = (R() >> 16) % 20; { s32 a = (s32) ((R() >> 16) & 1); f32 m = (R() >> 16) & 1 ? RM() / 64.0f : RF(); it_link_attr_math((itLinkHookshotAttributes*) r4_sa, a, m); } ret = r6_sa_h(24); });
    T(2040, "R6H_SAMUSGRAPPLE_COMPARE", { static const s32 ids[] = { 0x165, 0x166, 0xD4, 0xD6, 0x164, 0x167, 0xD5, 0 }; s32 id = (R() >> 16) & 1 ? ids[(R() >> 16) % 8] : (s32) R(); ret = (u32) samus_grapple_fighter_compare((FtMotionId) id); });
    /* ---- targeted moderate-input tables for the fused sites */
    T(2119, "R6M_8028EDBC", { r4_pre(ip); ip->ground_or_air = 0; ip->xDD4_itemVar.freeze.x4.x = RM(); ip->xDD4_itemVar.freeze.x0 = RM(); SAF(3) = RM(); SAF(2) = 1.0e30f; it_8028EDBC(g); ret = H(ret, r4_h(ip)); });
    T(2120, "R6M_802874F0", { r4_pre(ip); ip->ground_or_air = (R() >> 16) % 2; ip->xDD4_itemVar.taru.xDE8.x = RM(); ip->xDD4_itemVar.taru.xDE4 = RM(); ip->xDD4_itemVar.taru.xDE0 = RM(); ip->facing_dir = RM(); SAF(9) = RM(); SAF(11) = 1.0e30f; SAF(8) = 1.0e30f; it_802874F0(g); ret = H(ret, r4_h(ip)); });
    T(2121, "R6M_802962E0", { r4_pre(ip); ip->ground_or_air = (R() >> 16) % 2; ip->xDD4_itemVar.tarucann.x14.x = RM(); ip->xDD4_itemVar.tarucann.x10 = RM(); ip->xDD4_itemVar.tarucann.xC = RM(); ip->xDD4_itemVar.tarucann.x28 = RM(); ip->facing_dir = RM(); SAF(4) = RM(); SAF(6) = 1.0e30f; SAF(3) = 1.0e30f; SAF(20) = RM(); it_802962E0(g); ret = H(ret, r4_h(ip)); });
    /* r4_pre's low-bit kinds never pair a set xE08_b0/b1 byte with a due timer, nor arg1 == x4 */
    T(2124, "R6M_80291DAC_LEVEL", { r4_pre(ip); r4_sa[1] = (R() >> 16) % 40; r4_sa[2] = (R() >> 16) % 12; { s32 a = (R() >> 16) & 1 ? (s32) r4_sa[1] + (s32) ((R() >> 16) % 3) - 1 : (s32) (r4_sa[2] * 8) + (s32) ((R() >> 16) % 3) - 1; ret = (u32) it_80291DAC_level(g, a); } ret = H(ret, r4_h(ip)); });
    T(2131, "R6M_802DDBE8", { r4_pre(ip); ((u8*) &ip->xDD4_itemVar)[0x34] = (u8) ((R() >> 16) & 3); ip->xDD4_itemVar.zgshell.xDFC = (R() >> 16) & 1 ? 0.5f : 2.0f; it_802DDBE8(g); ret = H(ret, r4_h(ip)); });
    T(2138, "R6M_TEST_COMP", { Vec3 a, b; f32 k; a.x = RM(); a.y = RM(); a.z = RM(); b.x = RM(); b.y = RM(); b.z = RM(); k = RM(); test_comp(&ov, &a, &b, &k); });
}
#undef it_804D6D28
