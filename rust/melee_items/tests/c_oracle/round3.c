/* Round 3 (night 3) part of the melee_items C oracle; #included by oracle.c after round2.c and
 * reusing its LCG, Item fill/snap, fill_ext and helpers. Checks src/itlib3.rs: the bodies in
 * round3_bodies.h are verbatim copies of the decomp C.
 *
 * Each case seeds the LCG from (function id, case), fills the Item and the external data like
 * round 2, then draws this function's own inputs, runs it, and hashes snap(Item) + `ret` + the
 * output Vec3. `ret` additionally folds in every field the function reads or writes that the
 * round-2 field list does not cover. The globals the bodies use are redirected to statics here
 * (the Rust ports take them as parameters). Built like the rest: contraction on + gwtool
 * lowering (gen_expected.sh). */
#include "melee/it/it_3F14.h"
#include "melee/it/item.h"
#define it_804D6D28 it_804D6D28_o
#define HSD_GObjPLinkHead r3_plink
#define it_804D6D40 r3_d40
#define Item_804A0E24 r3_sel
#define it_804D6D14 r3_ctr
static HSD_GObj** r3_plink;
static it_804D6D40_t* r3_d40;
static PokemonSelectionState r3_sel;
static u32 r3_ctr;

#include "round3_bodies.h"

static HSD_GObj* r3_heads[16];
static HSD_GObj gL[3];
static Item r3_li[2];
static it_2728_DatAttrs r3_2728;
static itPokemonSpawn_DatAttrs r3_pk;
static Item r3_it1;
static Article r3_art1;
static it_804D6D40_t r3_d40v;
static void r3_pick(HSD_GObj* g) { (void) g; }

/* A list of n (0..3) GObjs: the case Item first, then r3_li[0], r3_li[1]. */
static void r3_list(Item* ip)
{
    u32 n = R() % 4, k;
    memset(gL, 0, sizeof gL);
    for (k = 0; k < 2; k++) {
        memset(&r3_li[k], 0, sizeof r3_li[k]);
        r3_li[k].kind = (R() & 1) ? ip->kind : KINDS[R() % NKINDS];
        flag32_set(&r3_li[k].xDC8_word, R());
    }
    gL[0].user_data = ip; gL[1].user_data = &r3_li[0]; gL[2].user_data = &r3_li[1];
    for (k = 0; k + 1 < n; k++) gL[k].next = &gL[k + 1];
    memset(r3_heads, 0, sizeof r3_heads);
    r3_heads[HSD_GOBJ_PLINK_ITEM] = n ? &gL[0] : NULL;
    r3_plink = r3_heads;
}
static u32 r3_list_h(void)
{
    return H(H(H(H(0, flag32_get(&r3_li[0].xDC8_word)), flag32_get(&r3_li[1].xDC8_word)),
             (u32) r3_li[0].kind), (u32) r3_li[1].kind);
}
static u32 r3_gidx(HSD_GObj* p) { return p == NULL ? 0 : (u32) (p - gL) + 1; }
static u32 r3_pk_h(itPokemonSpawn_ItemVars* v)
{
    u32 h = H(H(H(0, (u32) v->x0), bits(v->x4.x)), bits(v->x4.y)), k;
    h = H(H(H(H(h, bits(v->x4.z)), bits(v->x10.x)), bits(v->x10.y)), bits(v->x10.z));
    h = H(H(H(H(h, bits(v->x1C)), bits(v->x20)), bits(v->x24)), bits(v->x28));
    for (k = 0; k < 6; k++) h = H(h, bits(v->x2C[k]));
    h = H(H(H(H(h, bits(v->x44)), bits(v->x48)), bits(v->x4C)), bits(v->x50));
    return H(H(H(h, (u32) v->x54), (u32) v->x58), (u32) v->x5C);
}
static void r3_pk_fill(itPokemonSpawn_ItemVars* v)
{
    int k;
    v->x0 = (s32) R(); v->x4.x = RF(); v->x4.y = RF(); v->x4.z = RF();
    v->x10.x = RF(); v->x10.y = RF(); v->x10.z = RF();
    v->x1C = RF(); v->x20 = RF(); v->x24 = RF(); v->x28 = RF();
    for (k = 0; k < 6; k++) v->x2C[k] = RF();
    v->x44 = RF(); v->x48 = RF(); v->x4C = RF(); v->x50 = RF();
    v->x54 = (s32) R(); v->x58 = (s32) R(); v->x5C = (s32) R();
}
static void r3_pk_attr(void)
{
    int k;
    memset(&r3_pk, 0, sizeof r3_pk);
    r3_pk.xC = RF(); r3_pk.x10 = RF(); r3_pk.x14 = RF(); r3_pk.x18 = RF();
    for (k = 0; k < 6; k++) r3_pk.x1C[k] = RF();
    r3_pk.x34 = RF(); r3_pk.x38 = (s32) (R() % 7);
    for (k = 0; k < 30; k++) r3_pk.pokemon_spawn_weights[k] = (R() & 3) ? (s32) (R() % 1000) : (s32) R();
}
static s32 r3_pkind(void) { return (R() & 3) ? It_PKind_Start + (s32) (R() % 30) : (s32) (R() % 300); }
static u32 r3_hdr_extra(Item* ip)
{
    u32 h = H(H(H(0, bits(ip->xCC8_knockback)), bits(ip->xCCC_incDamageDirection)),
              bits(ip->xCB8_outDamageDirection));
    h = H(H(H(H(h, bits(ip->xC68)), bits(ip->xCD0)), bits(ip->xCC0)), ufs_get(&ip->xDCF_flag));
    h = H(H(H(H(h, (u32) ip->xCC4), PIDX(ip->xCF0_itemGObj)), PIDX(ip->xCFC)), (u32) ip->xCB4);
    h = H(H(H(H(h, (u32) ip->xC38), bits(ip->xCE0.x)), bits(ip->xCE0.y)), bits(ip->xCE0.z));
    h = H(H(H(h, bits(ip->xCD4.x)), bits(ip->xCD4.y)), bits(ip->xCD4.z));
    return H(H(h, bits(ip->xC6C)), (u32) ip->xCA8);
}
static void r3_fill_extra(Item* ip)
{
    ip->xCC8_knockback = RF(); ip->xCCC_incDamageDirection = RF(); ip->xCB8_outDamageDirection = RF();
    ip->xC68 = RF(); ip->xCD0 = RF(); ip->xCC0 = RF(); ufs_set(&ip->xDCF_flag, (u8) R());
    ip->xCC4 = (s32) R(); ip->xCF0_itemGObj = PTR(R()); ip->xCFC = PTR(R()); ip->xCB4 = (s32) R();
    ip->xC38 = (s32) R(); ip->xCE0.x = RF(); ip->xCE0.y = RF(); ip->xCE0.z = RF();
    ip->xCD4.x = RF(); ip->xCD4.y = RF(); ip->xCD4.z = RF(); ip->xC6C = RF(); ip->xCA8 = (s32) R();
}

static void round3(void)
{
    /* it_2725.c */
    T(201, "R3_80272818", { r3_2728.fighter = (Fighter*) PTR(R()); r2_art.x4_specialAttributes = &r3_2728;
                            ret = PIDX(it_80272818(ip)); });
    T(202, "R3_80272828", { ItemKind k; r3_list(ip); k = (R() & 1) ? ip->kind : KINDS[R() % NKINDS];
                            ret = H(r3_gidx(it_80272828(k)), r3_list_h()); });
    T(203, "R3_80272860", { f32 a1 = RF(), a2 = RF(); it_80272860(g, a1, a2); });
    T(204, "R3_80272D1C", { HSD_GObj* p = (R() & 3) ? g : NULL;
                            g->classifier = (R() & 1) ? HSD_GOBJ_CLASS_ITEM : (u16) R(); ret = it_80272D1C(p); });
    T(205, "R3_BOUNCEVICTIM", { r2_common.x58_float = RM(); r2_common.x5C_float = RM(); r2_common.x60_float = RM();
                                if (R() & 1) ip->x40_vel.y = RM(); itColl_BounceOffVictim(g); });
    T(206, "R3_80273500", { Vec3 v; v.x = RF(); v.y = RF(); v.z = RF(); it_80273500(g, &v); });
    T(207, "R3_80274658", { f32 a; ip->spin_spd = (R() & 3) ? RF() : 0.0f; ip->xD3C_spinSpeed = RF(); a = RF();
                            ret = (u32) it_80274658(g, a); ret = H(H(ret, bits(ip->xD3C_spinSpeed)), bits(ip->spin_spd)); });
    T(208, "R3_80274D04", { struct lb_80014638_arg1_t o; memset(&o, 0, sizeof o); o.unk_x = RF(); o.unk_y = RF();
                            it_80274D04(g, &o); ret = H(bits(o.unk_x), bits(o.unk_y)); });
    T(209, "R3_80275174", { f32 lt; r2_common.x4C_float = RF(); lt = RF(); it_80275174(g, lt); });
    T(210, "R3_80275328", { ItemLink lk; HSD_GObj* own; u8 b = (u8) R();
                            memset(&lk, 0, sizeof lk);
                            lk.prev = (ItemLink*) PTR(R()); lk.next = (ItemLink*) PTR(R());
                            lk.vel.x = RF(); lk.vel.y = RF(); lk.vel.z = RF(); lk.pos.x = RF(); lk.pos.y = RF();
                            lk.pos.z = RF(); lk.x20.x = RF(); lk.x20.y = RF(); lk.x20.z = RF();
                            lk.x2C_b0 = b >> 7; lk.x2C_b1 = b >> 6; lk.x2C_b2 = b >> 5; lk.x2C_b3 = b >> 4;
                            lk.x2C_b4 = b >> 3; lk.x2C_b5 = b >> 2; lk.x2C_b6 = b >> 1; lk.x2C_b7 = b;
                            lk.x1CC = (s32) R(); lk.gobj = PTR(R()); lk.jobj = (HSD_JObj*) PTR(R());
                            own = PTR(R());
                            it_80275328(own, &lk);
                            ret = H(H(PIDX(lk.prev), PIDX(lk.next)), bits(lk.vel.x));
                            ret = H(H(H(H(ret, bits(lk.vel.y)), bits(lk.vel.z)), bits(lk.pos.x)), bits(lk.pos.y));
                            ret = H(H(H(H(ret, bits(lk.pos.z)), bits(lk.x20.x)), bits(lk.x20.y)), bits(lk.x20.z));
                            ret = H(ret, (lk.x2C_b0 << 7) | (lk.x2C_b1 << 6) | (lk.x2C_b2 << 5) | (lk.x2C_b3 << 4) |
                                         (lk.x2C_b4 << 3) | (lk.x2C_b5 << 2) | (lk.x2C_b6 << 1) | lk.x2C_b7);
                            ret = H(H(H(ret, (u32) lk.x1CC), PIDX(lk.gobj)), PIDX(lk.jobj)); });
    T(211, "R3_80275390", it_80275390(g));
    /* it_26B1.c */
    T(212, "R3_8026B3C0", { ItemKind k; r3_list(ip); k = (R() & 1) ? ip->kind : KINDS[R() % NKINDS];
                            ret = H((u32) it_8026B3C0(k), r3_list_h()); });
    T(213, "R3_8026C3FC", { r3_list(ip); it_8026C3FC(); ret = r3_list_h(); });
    T(214, "R3_8026C42C", { r3_list(ip); it_8026C42C(); ret = r3_list_h(); });
    /* it_279C.c */
    T(215, "R3_8027A13C", { r3_pk_fill(&ip->xDD4_itemVar.pokemon_spawn); it_8027A13C(g);
                            ret = r3_pk_h(&ip->xDD4_itemVar.pokemon_spawn); });
    T(216, "R3_8027A344", { r3_pk_fill(&ip->xDD4_itemVar.pokemon_spawn); it_8027A344(g);
                            ret = r3_pk_h(&ip->xDD4_itemVar.pokemon_spawn); });
    T(217, "R3_8027A364", { r3_pk_attr(); r2_art.x4_specialAttributes = &r3_pk;
                            r3_sel.last_kind = r3_pkind(); r3_sel.previous_kind = r3_pkind();
                            ret = (u32) it_8027A364(ip); });
    T(218, "R3_8027AAA0", { s32 a2; r3_pk_attr(); memset(&r3_it1, 0, sizeof r3_it1); memset(&r3_art1, 0, sizeof r3_art1);
                            r3_art1.x4_specialAttributes = &r3_pk; r3_it1.xC4_article_data = &r3_art1;
                            r2_attr.x60_scale = RF(); r3_pk_fill(&ip->xDD4_itemVar.pokemon_spawn);
                            a2 = (R() & 1) ? It_PKind_Marumine : (s32) R();
                            { HSD_GObj g1; memset(&g1, 0, sizeof g1); g1.user_data = &r3_it1; it_8027AAA0(&g1, ip, a2); }
                            ret = r3_pk_h(&ip->xDD4_itemVar.pokemon_spawn); });
    /* itanimlist.c */
    T(219, "R3_802795EC", { CmdUnion cb[2]; CommandInfo ci; memset(cb, 0, sizeof cb); memset(&ci, 0, sizeof ci);
                            cb[0].set_hitbox_scale.opcode = R(); cb[0].set_hitbox_scale.idx = R() % 4;
                            cb[0].set_hitbox_scale.value = R(); ci.u = cb;
                            it_802795EC(g, &ci); ret = H((u32) (ci.u - cb), bits(ip->x3C)); });
    T(220, "R3_802796FC", { CmdUnion cb[2]; CommandInfo ci; memset(cb, 0, sizeof cb); memset(&ci, 0, sizeof ci);
                            cb[0].set_throw_flags.opcode = R(); cb[0].set_throw_flags.hit_idx = R(); ci.u = cb;
                            it_802796FC(g, &ci); ret = H((u32) (ci.u - cb), ip->xDAC_itcmd_var0); });
    /* itcoll.c */
    T(221, "R3_80270CD8", { HitCapsule h; int k; memset(&h, 0, sizeof h);
                            h.x24 = R() % 200; h.x28 = (R() & 1) ? 0 : R() % 200; h.x2C = R() % 100; h.damage = RM();
                            for (k = 7; k < 13; k++) r2_common.x80_float[k] = RM();
                            if (R() & 1) r2_common.x80_float[7] = 1e30f;
                            ip->xC9C = (s32) (R() % 300); ip->xCA0 = (s32) (R() % 50); r2_attr.x1C_damage_mul = RM();
                            ret = bits(it_80270CD8(ip, &h)); });
    /* item.c */
    T(222, "R3_80268E40", { struct ItemStateDesc d; memset(&d, 0, sizeof d); d.xC_script = PTR(R());
                            ip->x524_cmd.loop_count = R(); ip->x524_cmd.timer = RF();
                            Item_80268E40(ip, &d);
                            ret = H(H(PIDX(ip->x524_cmd.u), ip->x524_cmd.loop_count), bits(ip->x524_cmd.timer)); });
    T(223, "R3_80269CC4", { r3_fill_extra(ip); Item_80269CC4(g); ret = r3_hdr_extra(ip); });
    T(224, "R3_8026AE60", { r3_ctr = (R() % 3 == 0) ? 0xFFFFFFFFu : R(); ret = Item_8026AE60(); ret = H(ret, r3_ctr); });
    T(225, "R3_ISGRABBABLE", { ItemLogicTable lt; memset(&lt, 0, sizeof lt);
                               lt.picked_up = (R() & 1) ? r3_pick : NULL; ip->xB8_itemLogicTable = &lt;
                               ret = Item_IsGrabbable(g); });
    /* ithitbox.c */
    T(226, "R3_802756E0", it_802756E0(g));
    /* itmaplib.c */
    T(227, "R3_80276CEC", { ip->x378_itemColl.env_flags = (R() % 4 == 0) ? 0 : (s32) (R() & R());
                            ip->xC2C = (s32) R(); it_80276CEC(g); ret = (u32) ip->xC2C; });
    /* itspawn.c */
    T(228, "R3_8026CA4C", { s32 w[64]; u64 m; s32 st; f32 a4; ItemPickTable t; int k;
                            for (k = 0; k < 64; k++) w[k] = (R() & 7) ? (s32) (R() % 100) : (s32) (R() % 100000);
                            m = (u64) R() << 32; m |= R(); st = (s32) (R() % 40);
                            a4 = (R() & 1) ? RM() : (f32) (R() % 1000) / 997.0f;
                            memset(&t, 0, sizeof t); it_8026CA4C(&t, w, m, st, a4); ret = t.x8; });
    /* targeted inputs for fused sites the tables above do not reach: */
    /* it_80270CD8's no-x28 branch, with x80_float[9] small so the first fma's rounding shows */
    T(231, "R3_80270CD8_NOX28", { HitCapsule h; int k; memset(&h, 0, sizeof h);
                                  h.x24 = R() % 200; h.x2C = R() % 100; h.damage = RM();
                                  for (k = 7; k < 13; k++) r2_common.x80_float[k] = RM();
                                  r2_common.x80_float[9] = RM() / 4096.0f; r2_common.x80_float[7] = 1e30f;
                                  ip->xC9C = (s32) (R() % 300); ip->xCA0 = (s32) (R() % 50); r2_attr.x1C_damage_mul = RM();
                                  ret = bits(it_80270CD8(ip, &h)); });
    /* it_8026CA4C with arg4 = (n - 0.99) / w for one weight w, so arg4 * w + 0.99 lands on an
     * integer and the fused vs unfused rounding decides the truncation */
    T(232, "R3_8026CA4C_EDGE", { s32 w[64]; s32 st, wv, n; f32 a4; ItemPickTable t; int k;
                                 wv = (s32) (R() % 200) + 1; n = (s32) (R() % 50) + 1;
                                 for (k = 0; k < 64; k++) w[k] = wv;
                                 st = (s32) (R() % 35); a4 = ((f32) n - 0.99f) / (f32) wv;
                                 if (R() & 1) a4 = fbits(bits(a4) + 1); else if (R() & 1) a4 = fbits(bits(a4) - 1);
                                 memset(&t, 0, sizeof t); it_8026CA4C(&t, w, ~(u64) 0, st, a4); ret = t.x8; });
    /* itzako.c */
    T(229, "R3_8027B730", { itZako_ItemVars* z = &ip->xDD4_itemVar.zako; it_8027B730(g);
                            ret = H(H(H(H(PIDX(z->jobj), (u32) z->idx), bits(z->x8.x)), bits(z->x8.y)), bits(z->x8.z));
                            ret = H(H(H(ret, (u32) z->x14.x), (u32) z->x14.y), (u32) z->x14.z); });
    T(230, "R3_8027CBFC", { r3_d40v.xC = RF(); r3_d40v.x10 = RF(); r3_d40 = &r3_d40v; ip->xCC4 = (s32) (R() % 18);
                            ret = bits(it_8027CBFC(g)); });
}
#undef it_804D6D28
#undef HSD_GObjPLinkHead
#undef it_804D6D40
#undef Item_804A0E24
#undef it_804D6D14
