/* Round 14 (night 3, round 6) part of the melee_items C oracle; #included by oracle.c after
 * round13.c and reusing round 2's LCG / Item fill+snap, round 4's r4_pre / r4_h, round 7's vector
 * draws, round 10's r10_h, round 11's JObj chain / model, round 12's plink heads and round 13's
 * Like-Like setup / r13_sh. Checks src/itkinds14.rs (26 hand ports, 28 tables) against round14_bodies.h,
 * verbatim copies of the decomp C extracted by gen_round14.py. Built like the rest: contraction
 * on + gwtool lowering. */

/* file-local types the bodies name: ityoshistar.c:19-22 (verbatim) and itzgshell.c's attrs
 * (only ever returned as a pointer here, so left incomplete) */
typedef struct {
    f32 speed;
    f32 accel;
} StarAttrs;
typedef struct itGShell_Attrs itGShell_Attrs;

/* itsamusbomb.c's own my_sqrtf is the 4-step one (itsamusbomb.c:149-167), the same body as
 * itlugia.c's my_sqrtf_accurate (round9_bodies.h) */
#define my_sqrtf my_sqrtf_accurate
#define it_803F5AB0 r14_pst
#define HSD_GObjPLinkHead r3_plink
static ItemStateTable r14_pst[11];
#include "round14_bodies.h"
#undef HSD_GObjPLinkHead
#undef it_803F5AB0
#undef my_sqrtf

static Item r14_li[3];
static HSD_GObj r14_lg[3];
static CmdUnion r14_cu[2];
static HSD_GObj* r14_heads[16];

static void r14_lists(void)
{
    u32 k;
    memset(r14_li, 0, sizeof r14_li); memset(r14_lg, 0, sizeof r14_lg);
    for (k = 0; k < 3; k++) r14_lg[k].user_data = &r14_li[k];
}
static u32 r14_gidx(void* p)
{
    u32 k;
    if (p == NULL) return 3;
    for (k = 0; k < 3; k++) if (p == (void*) &r14_lg[k] || p == (void*) &r14_li[k]) return k;
    return 4;
}
/* pointers stored in the item-var union are cleared before the hash: fields.h's snap reads
 * heal amounts etc. that alias them, and pointer values differ between the C and Rust sides */
/* the item p-link list: 3 items of drawn kinds, head NULL a fifth of the time */
static void r14_plist(void)
{
    u32 k;
    r14_lists();
    for (k = 0; k < 3; k++) {
        r14_li[k].kind = (ItemKind) ((R() >> 16) % 240);
        r14_lg[k].next = k < 2 ? &r14_lg[k + 1] : NULL;
    }
    memset(r14_heads, 0, sizeof r14_heads);
    r14_heads[HSD_GOBJ_PLINK_ITEM] = (R() % 5) ? &r14_lg[0] : NULL;
    r3_plink = r14_heads;
}
static u32 r14_ptr(const void* p) { u64 v = (u64) (uintptr_t) p; return H((u32) v, (u32) (v >> 32)); }

static void round14(void)
{
    T(3401, "R14_80285314", { r11_jobjs(g); r11_model(ip); ret = r11_jidx(it_80285314(g)); });
    T(3402, "R14_GET_BONE_BY_ID", { s32 idx; r11_jobjs(g); r11_model(ip); idx = (s32) ((R() >> 16) % 6); ret = r11_jidx(get_bone_by_id(g, idx)); });
    T(3403, "R14_80279768", { CommandInfo cmd; memset(&cmd, 0, sizeof cmd); memset(r14_cu, 0, sizeof r14_cu); cmd.u = &r14_cu[0]; ip->xDBC_itcmd_var4.flags.x0 = (R() >> 16) & 1; ip->xDBC_itcmd_var4.flags.x1 = (R() >> 16) & 1; it_80279768(g, &cmd); ret = H(H(ip->xDBC_itcmd_var4.flags.x0, ip->xDBC_itcmd_var4.flags.x1), (u32) (cmd.u - r14_cu)); });
    T(3404, "R14_80279720", { CommandInfo cmd; memset(&cmd, 0, sizeof cmd); memset(r14_cu, 0, sizeof r14_cu); cmd.u = &r14_cu[0]; r14_cu[0].set_throw_flags.opcode = R() & 63; r14_cu[0].set_throw_flags.hit_idx = R() & 0x3FFFFFF; it_80279720(g, &cmd); ret = H(ip->xDB0_itcmd_var1, (u32) (cmd.u - r14_cu)); });
    T(3405, "R14_80279744", { CommandInfo cmd; memset(&cmd, 0, sizeof cmd); memset(r14_cu, 0, sizeof r14_cu); cmd.u = &r14_cu[0]; r14_cu[0].set_throw_flags.opcode = R() & 63; r14_cu[0].set_throw_flags.hit_idx = R() & 0x3FFFFFF; it_80279744(g, &cmd); ret = H(ip->xDB4_itcmd_var2, (u32) (cmd.u - r14_cu)); });
    T(3406, "R14_8028B08C", { u32 k; s32 s; for (k = 0; k < 11; k++) { memset(&r14_pst[k], 0, sizeof r14_pst[k]); r14_pst[k].anim_id = (enum_t) ((R() >> 16) % 8); } s = (s32) ((R() >> 16) % 11); ret = (u32) it_8028B08C(g, s); });
    T(3407, "R14_YOSHISTAR_M0_PHYS", { r4_pre(ip); ip->facing_dir = r12_face(); itYoshistar_UnkMotion0_Phys(g); ret = r10_h(ip); });
    T(3408, "R14_80291D38", { s32 n; r4_pre(ip); n = (s32) ((R() >> 16) % 10); ret = (u32) it_80291D38(g, n); });
    T(3409, "R14_802AB9C0", { u32 k; r14_lists(); for (k = 0; k < 3; k++) { r14_li[k].owner = &r14_lg[0]; r14_li[k].xDC8_word.flags.x13 = 1; } for (k = 0; k < 6; k++) ip->xDD4_itemVar.pkthunder.xDD4[k] = ((R() >> 16) % 3) ? &r14_lg[(R() >> 16) % 3] : NULL; ip->xDD4_itemVar.pkthunder.xF00 = ((R() >> 16) & 1) ? 0 : (s32) R(); ip->owner = &r14_lg[1]; it_802AB9C0(((R() >> 16) & 7) ? g : NULL); memset(ip->xDD4_itemVar.pkthunder.xDD4, 0, sizeof ip->xDD4_itemVar.pkthunder.xDD4); ret = H(0, ip->owner == NULL); for (k = 0; k < 3; k++) ret = H(H(ret, r14_li[k].owner == NULL), r14_li[k].xDC8_word.flags.x13); });
    T(3410, "R14_LIKELIKEVELOCITY", { f32 m; itLikelikeAttributes* a; r13_ll(ip); a = ip->xC4_article_data->x4_specialAttributes; ip->xDD4_itemVar.likelike.x20.x = ip->pos.x + RM() / 64.0f; ip->xDD4_itemVar.likelike.x20.y = ((R() >> 16) & 3) ? ip->pos.y + RM() / 64.0f : RF(); ip->xDD4_itemVar.likelike.x48 = (s32) ((R() >> 16) % 7); m = ((R() >> 16) & 3) ? RM() / 16.0f : RF(); if ((R() >> 16) & 1) { r13_v.y = 4.0f; m = 1.0f; } likelikeVelocity(g, ip, a, m); ret = H(r13_sh(ip), (u32) ip->xDD4_itemVar.likelike.x48); });
    T(3411, "R14_SAMUSBOMB_M2_PHYS", { itSamusBombAttributes* a; r4_pre(ip); r7_vin(&ip->x7C); a = ip->xC4_article_data->x4_specialAttributes; if ((R() >> 16) & 3) { f32 t = RM(); a->xC = ABS(t) / 8.0f; } else { a->xC = RF(); } if (((R() >> 16) & 3) == 0) a->xC = my_sqrtf_accurate(VEC2_SQ_LEN(ip->x7C)); itSamusbomb_UnkMotion2_Phys(g); ret = r10_h(ip); });
    T(3412, "R14_PKFIREPILLAR_L24", { r4_pre(ip); ip->xC9C = (s32) ((R() >> 16) % 300) - 20; ip->xD44_lifeTimer = ((R() >> 16) & 3) ? RM() : RF(); if (((R() >> 16) & 7) == 0) { ip->xC9C = 0; ip->xD44_lifeTimer = 0.0f; } ret = (u32) itNesspkfirepillar_Logic24_DmgReceived(g); ret = H(ret, bits(ip->xD44_lifeTimer)); });
    T(3413, "R14_802B22B8_INLINE", { r14_lists(); ip->xDD4_itemVar.pikachuthunder.x34 = ((R() >> 16) % 3) ? &r14_lg[0] : NULL; r14_lg[0].user_data = ((R() >> 16) & 3) ? &r14_li[0] : NULL; r14_li[0].xDD4_itemVar.pikachuthunder.x4 = (s32) R(); it_802B22B8_inline(g); ip->xDD4_itemVar.pikachuthunder.x34 = NULL; ret = H(H(H(H(0, bits(r14_li[0].xDD4_itemVar.pikachuthunder.x28.x)), bits(r14_li[0].xDD4_itemVar.pikachuthunder.x28.y)), bits(r14_li[0].xDD4_itemVar.pikachuthunder.x28.z)), (u32) r14_li[0].xDD4_itemVar.pikachuthunder.x4); });
    T(3414, "R14_JOLTAIR_M0_ANIM_INLINE", { u32 s; r14_lists(); ip->xDD4_itemVar.pikachujoltair.xDD8 = ((R() >> 16) % 3) ? &r14_lg[0] : NULL; r14_lg[0].user_data = ((R() >> 16) & 3) ? &r14_li[0] : NULL; s = (R() >> 16) % 3; r14_li[0].xDD4_itemVar.pikachujoltground.xDDC = s == 0 ? g : s == 1 ? NULL : &r14_lg[1]; ret = (u32) itPikachutjoltair_UnkMotion0_Anim_inline(g); ip->xDD4_itemVar.pikachujoltair.xDD8 = NULL; });
    T(3415, "R14_PIKATHUNDER_M2_SCALECALL", { r4_pre(ip); ip->x5D4_hitboxes[0].hit.state = ((R() >> 16) & 1) ? HitCapsule_Disabled : (HitCapsuleState) 1; ip->x5D4_hitboxes[0].hit.scale = RF(); ip->xDD4_itemVar.pikachuthunder.x10 = RM(); ip->xDD4_itemVar.pikachuthunder.xC = ((R() >> 16) & 3) ? RM() : RF(); itPikachuthunder_UnkMotion2_ScaleCall(g); ret = H(bits(ip->x5D4_hitboxes[0].hit.scale), (u32) ip->x5D4_hitboxes[0].hit.state); });
    T(3416, "R14_BOMBHEI_80280974_INLINE", { r4_pre(ip); ip->xDCD_flag.b5 = 0; ip->xDCD_flag.b6 = 0; ip->xDCD_flag.b7 = 0; ip->xD5C = (s32) R(); fn_80280974_inline(g); ret = H(H(H(r10_h(ip), ip->xDCD_flag.b5), ip->xDCD_flag.b6), ip->xDCD_flag.b7); });
    T(3417, "R14_8026C47C", { struct it_8026C47C_arg0_t a; u32 k; u32* w = (u32*) &a; r14_plist(); for (k = 0; k < 8; k++) w[k] = R(); it_8026C47C(&a); for (k = 0; k < 8; k++) ret = H(ret, w[k]); });
    T(3418, "R14_802D14D0", { it_802D14D0(); });
    T(3419, "R14_802F1340", { it_802F1340(); });
    T(3420, "R14_802D472C_INLINE", { ret = it_802D472C_inline(ip) == ip->xC4_article_data; });
    T(3421, "R14_GETFIGHTER", { u32 k = (R() >> 16) % 3; r14_lists(); ip->owner = &r14_lg[k]; ret = r14_gidx(fn_802A33A0_GetFighter(ip)); });
    T(3422, "R14_ZGSHELL_GET_ATTRS", { r4_pre(ip); ret = (void*) get_attrs(ip) == ip->xC4_article_data->x4_specialAttributes; });
    T(3423, "R14_DOSEI_GETJOBJ", { r11_jobjs(g); ret = r11_jidx(itDosei_GetJObj(g)); });
    T(3424, "R14_JOBJ_CHILD", { u32 k; r11_jobjs(g); k = (R() >> 16) % 5; ret = r11_jidx(jobj_child(k < 4 ? &r11_j[k] : NULL)); });
    T(3425, "R14_CLIMBERSSTRING_JOINT24", { r4_pre(ip); ret = r14_ptr(it_802C248C_joint24(ip)); });
    T(3426, "R14_CLIMBERSSTRING_JOINT28", { r4_pre(ip); ret = r14_ptr(it_802C248C_joint28(ip)); });
    /* boundary cases for the fused length (xC = the contracted length, or one ulp under it) and
     * for the strict `<` of likelikeVelocity (|dx|, |dy| exactly the limit 4 * 1) */
    T(3427, "R14_SAMUSBOMB_M2_PHYS_EDGE", { itSamusBombAttributes* a; f32 l; r4_pre(ip); ip->x7C.x = RM(); ip->x7C.y = RM(); ip->x7C.z = RM(); a = ip->xC4_article_data->x4_specialAttributes; l = my_sqrtf_accurate(VEC2_SQ_LEN(ip->x7C)); a->xC = ((R() >> 16) & 1) ? l : fbits(bits(l) - 1); itSamusbomb_UnkMotion2_Phys(g); ret = r10_h(ip); });
    T(3428, "R14_LIKELIKEVELOCITY_EDGE", { static const f32 d[4] = { 4.0f, -4.0f, 3.5f, -4.5f }; itLikelikeAttributes* a; r13_ll(ip); a = ip->xC4_article_data->x4_specialAttributes; r13_v.y = 4.0f; ip->pos.x = RM() / 64.0f; ip->pos.y = RM() / 64.0f; ip->xDD4_itemVar.likelike.x20.x = ip->pos.x + d[(R() >> 16) % 4]; ip->xDD4_itemVar.likelike.x20.y = ip->pos.y + d[(R() >> 16) % 4]; ip->xDD4_itemVar.likelike.x48 = (s32) ((R() >> 16) % 7); likelikeVelocity(g, ip, a, 1.0f); ret = H(r13_sh(ip), (u32) ip->xDD4_itemVar.likelike.x48); });
}
