/* Round 13 (night 3, round 5) part of the melee_items C oracle; #included by oracle.c after
 * round12.c and reusing round 2's LCG / Item fill+snap, round 4's r4_pre / r4_h, round 7's vector
 * draws, round 10's r10_h and round 12's counters / article table / r12_face. Checks
 * src/itkinds13.rs (29 hand ports) against round13_bodies.h, verbatim copies of the decomp C
 * extracted by gen_round13.py. Built like the rest: contraction on + gwtool lowering. */
#define Item_804A0C64 r12_unk
#define it_804D6D24 r12_tbl
#define it_804A0F60 r13_stage
#define gm_80473A18 r13_gm
#define it_803F74F8 r13_st
static Article* r13_stage[30];
static UnkAllstarData r13_gm;
static ItemStateTable r13_st[4];
#include "round13_bodies.h"
#undef it_803F74F8
#undef gm_80473A18
#undef it_804A0F60
#undef it_804D6D24
#undef Item_804A0C64

static Item r13_li2;
static HSD_GObj r13_lg;
static Vec3 r13_v;
static KinokoAnim r13_ka[8];
static HSD_AnimJoint r13_aj[8];
static Article r13_art;
static Article* r13_arts[40];
static Article r13_dummy[30];
static u32 r13_gm_h(void)
{
    u8* p = (u8*) &r13_gm + offsetof(UnkAllstarData, x90);
    u32 k, h = 0;
    for (k = 0; k < 12; k++) h = H(h, p[k]);
    return h;
}
static void r13_gm_fill(void)
{
    u8* p = (u8*) &r13_gm + offsetof(UnkAllstarData, x90);
    u32 k;
    for (k = 0; k < 12; k++) p[k] = (u8) (1 + (R() >> 16) % 255);
}
static void r13_link(Item* ip)
{
    memset(&r13_li2, 0, sizeof r13_li2); memset(&r13_lg, 0, sizeof r13_lg);
    r13_li2.xDD4_itemVar.freeze.x20 = &r13_lg; r13_lg.user_data = &r13_li2;
    ip->xDD4_itemVar.freeze.unk_1C = (R() >> 16) & 1 ? &r13_lg : NULL;
}
static u32 r13_link_h(Item* ip)
{
    return H(H(0, ip->xDD4_itemVar.freeze.unk_1C == NULL), r13_li2.xDD4_itemVar.freeze.x20 == NULL);
}
static void r13_ll(Item* ip)
{
    r4_pre(ip); r13_v.x = RF(); r13_v.y = RF(); r13_v.z = RF();
    ((itLikelikeAttributes*) ip->xC4_article_data->x4_specialAttributes)->x0.x0_f32 = &r13_v;
    ip->facing_dir = r12_face();
}
/* r10_h plus the sign bits of facing_dir and vel.x as bit 0 (H only carries bit 31 into bit 31,
 * so flipping both signs would cancel; the round 11 trap) */
static u32 r13_sh(Item* ip) { return H(H(r10_h(ip), bits(ip->facing_dir) >> 31), bits(ip->x40_vel.x) >> 31); }
static s32 r13_sc(void)
{
    u32 s = (R() >> 16) % 4;
    return s == 0 ? (s32) R() : (s32) ((R() >> 16) % 100) - 10;
}

static void round13(void)
{
    T(3301, "R13_CHECKHITLAG", { f32 m = (R() & 3) ? RM() : RF(); ip->xCBC_hitlagFrames = (R() & 3) ? RM() : (R() & 1 ? m : RF()); checkHitLag(m, ip); ret = H(bits(ip->xCBC_hitlagFrames), ip->xDC8_word.flags.xA); });
    T(3302, "R13_RESETSTAGEART", { u32 k; for (k = 0; k < 30; k++) r13_stage[k] = (R() & 1) ? &r13_dummy[k] : NULL; it_ResetStageArticleTable(); for (k = 0; k < 30; k++) ret = H(ret, r13_stage[k] == NULL); });
    T(3303, "R13_BLIZZARD_M0_PHYS", { r4_pre(ip); itClimbersBlizzard_UnkMotion0_Phys(g); ret = r10_h(ip); });
    T(3304, "R13_DOSEI_GETATTRS", { r4_pre(ip); ret = (u32) ((u8*) itDosei_UnkMotion4_GetAttrs(ip) - (u8*) ip->xC4_article_data->x4_specialAttributes); });
    T(3305, "R13_FLIPPER_REPEL", { Vec3 p; s32 k; r4_pre(ip); k = (R() & 1) ? It_Kind_Flipper : (s32) ((R() >> 16) % 40); r7_vin(&p); itFlipper_Repel(g, k, &p); ret = r10_h(ip); });
    T(3306, "R13_FREEZE_CLEARLINKED", { r13_link(ip); itFreeze_ClearLinkedItem(g); ret = r13_link_h(ip); });
    T(3307, "R13_FREEZE_L17_DESTROYED", { r13_link(ip); itFreeze_Logic17_Destroyed(g); ret = r13_link_h(ip); });
    T(3308, "R13_80299FB4_GET", { ret = it_80299FB4_get(g) == ip; });
    T(3309, "R13_HEART_L8_DESTROYED", { r13_gm_fill(); ip->xDD4_itemVar.heart.xDD8.b0 = (R() >> 16) & 1; ip->xDD4_itemVar.heart.xDDC = (s32) ((R() >> 16) % 12); itHeart_Logic8_Destroyed(g); ret = r13_gm_h(); });
    T(3310, "R13_80293660", { u32 k; s32 idx = (s32) ((R() >> 16) % 6); for (k = 0; k < 40; k++) r13_arts[k] = NULL; memset(&r13_art, 0, sizeof r13_art); for (k = 0; k < 8; k++) r13_ka[k].joint = &r13_aj[(k * 5 + R()) % 8]; r13_art.x4_specialAttributes = r13_ka; r13_arts[It_Kind_Kinoko] = &r13_art; r12_tbl = r13_arts; ret = (u32) (it_80293660(idx) - r13_aj); });
    T(3311, "R13_KYASARIN_GETCOLL", { ret = (u32) ((u8*) itKyasarin_GetCollData(ip) - (u8*) ip); });
    T(3312, "R13_KYASARIN_M9_COLL", { ret = itKyasarin_UnkMotion9_Coll(g); });
    T(3313, "R13_LEADEAD_M16_COLL", { ret = itLeadead_UnkMotion16_Coll(g); });
    T(3314, "R13_LIKELIKE_M21_COLL", { ret = itLikelike_UnkMotion21_Coll(g); });
    T(3315, "R13_OCTAROCK_M8_COLL", { ret = itOctarock_UnkMotion8_Coll(g); });
    T(3316, "R13_OLDKURI_M11_COLL", { ret = itOldkuri_UnkMotion11_Coll(g); });
    T(3317, "R13_OLDKURI_M7_COLL", { ret = itOldkuri_UnkMotion7_Coll(g); });
    T(3318, "R13_WHITEBEA_M11_COLL", { ret = itWhitebea_UnkMotion11_Coll(g); });
    T(3319, "R13_ITSWAPVELOCITY", { r13_ll(ip); itSwapVelocity(ip); ret = r13_sh(ip); });
    T(3320, "R13_SWAPVELOCITY", { r13_ll(ip); swapVelocity(g); ret = r13_sh(ip); });
    T(3321, "R13_802D9B78", { r13_ll(ip); it_802D9B78(g); ret = r13_sh(ip); });
    T(3322, "R13_80295748", { r4_pre(ip); it_80295748(g, &ov); });
    T(3323, "R13_MBALL_DESTROYED", { r12_unk_fill(); ip->xDD4_itemVar.mball.b1 = (R() >> 16) & 1; itMball_Destroyed(g); ret = r12_unk_h(); });
    T(3324, "R13_802BDA40", { u32 k; s32 i = (s32) ((R() >> 16) % 4); for (k = 0; k < 4; k++) { memset(&r13_st[k], 0, sizeof r13_st[k]); r13_st[k].anim_id = (R() >> 16) & 1; } ret = (u32) it_802BDA40(g, i); });
    T(3325, "R13_80291CCC", { r4_pre(ip); it_80291CCC(g, &ov); });
    T(3326, "R13_80291CF4", { itSScopeAttributes* a; s32 n; r4_pre(ip); a = ip->xC4_article_data->x4_specialAttributes; a->x4 = r13_sc(); a->x8 = r13_sc() / 4; n = r13_sc(); if ((R() >> 16) % 4 == 0) n = a->x4 + (s32) ((R() >> 16) % 3) - 1; else if ((R() >> 16) % 4 == 0) n = a->x8 * 8 + (s32) ((R() >> 16) % 3) - 1; ret = (u32) it_80291CF4(g, n); });
    T(3327, "R13_80285300", { r4_pre(ip); ret = (u32) ((u8*) it_80285300(g) - (u8*) ip->xC4_article_data->x4_specialAttributes); });
    T(3328, "R13_TOMATO_L9_DESTROYED", { r13_gm_fill(); ip->xDD4_itemVar.tomato.x4_b0 = (R() >> 16) & 1; ip->xDD4_itemVar.tomato.x8 = (R() >> 16) % 12; itTomato_Logic9_Destroyed(g); ret = r13_gm_h(); });
    T(3329, "R13_ZRSHELL_M4_ANIM", { r4_pre(ip); ((u8*) &ip->xDD4_itemVar)[0x34] = (u8) ((R() >> 16) & 3); ip->xDD4_itemVar.zgshell.xDFC = (R() >> 16) & 1 ? 0.5f : 2.0f; ret = (u32) itZrshell_UnkMotion4_Anim(g); ret = H(ret, r4_h(ip)); });
}
