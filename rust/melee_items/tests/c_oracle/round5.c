/* Round 5 (night 3) part of the melee_items C oracle; #included by oracle.c after round4.c and
 * reusing round 2's LCG / Item fill+snap and round 4's r4_pre / r4_h. Checks src/itfwd.rs
 * (generated forwarders) and src/itkinds5.rs (hand ports): round5_bodies.h holds verbatim
 * copies of the decomp C, extracted by gen_round5.py. Built like the rest: contraction on +
 * gwtool lowering (gen_expected.sh). */
#include <stdint.h>
#include "melee/it/kinds/itcapsule.h"
#define it_804D6D28 it_804D6D28_o
/* round 4's GObj-taking copy of it_80275158 */
#define it_80275158 it_80275158_g
/* round1.c's copy of decrease_lifetimer takes the Item (its GET_ITEM is a cast); the round-5
 * callers pass the GObj, so they get this verbatim copy (src/melee/it/kinds/itpeachturnip.c). */
#define decrease_lifetimer decrease_lifetimer_g
static bool decrease_lifetimer(Item_GObj* item_gobj)
{
    Item* item;

    item = GET_ITEM(item_gobj);
    item->xD44_lifeTimer--;
    if (item->xD44_lifeTimer <= 0.0F) {
        return true;
    }
    return false;
}

#include "round5_bodies.h"

static u32 bits64(f64 d) { u64 b; memcpy(&b, &d, 8); return H((u32) b, (u32) (b >> 32)); }
static u32 praw(void* p) { u64 b = (u64) (uintptr_t) p; return H((u32) b, (u32) (b >> 32)); }

/* Sword blend state with small values from the LCG's high bits: r4_pre draws each item-var word
 * kind from the LCG's low bits, which cycle, so some int/int/step combinations never occur. */
static void r5_sword(Item* ip)
{
    static const int isint[19] = { 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0 };
    u32* w = (u32*) &ip->xDD4_itemVar;
    int k;
    for (k = 1; k < 19; k++) {
        s32 v = (s32) ((R() >> 16) % 5) - 2;
        w[k] = isint[k] ? (u32) v : bits((f32) v * 0.5f);
    }
}

static void round5(void)
{
#include "round5_fwd.inc"
    /* ---- hand ports (src/itkinds5.rs) */
    T(1001, "R5H_SEAKNEEDLE_DESTROYED", { r4_pre(ip); itSeakNeedleThrown_Logic109_Destroyed(g); ret = H(ret, r4_h(ip)); });
    T(1002, "R5H_8028ECE0", { r4_pre(ip); it_8028ECE0(g); ret = H(ret, r4_h(ip)); });
    T(1003, "R5H_802B1DEC", { r4_pre(ip); ret = (u32) it_802B1DEC(g); ret = H(ret, r4_h(ip)); });
    T(1004, "R5H_802B3544", { r4_pre(ip); it_802B3544(g); ret = H(ret, r4_h(ip)); });
    T(1005, "R5H_802E5EF4", { r4_pre(ip); ret = (u32) it_802E5EF4(g); ret = H(ret, r4_h(ip)); });
    T(1006, "R5H_802ECC8C", { r4_pre(ip); ret = (u32) it_802ECC8C(g); ret = H(ret, r4_h(ip)); });
    T(1007, "R5H_802ECC98", { r4_pre(ip); { f32 a = RF(); it_802ECC98(g, a); } ret = H(ret, r4_h(ip)); });
    T(1008, "R5H_802ECD1C", { r4_pre(ip); it_802ECD1C(g, &ov); ret = H(ret, r4_h(ip)); });
    T(1009, "R5H_802F2014", { r4_pre(ip); { s32 a = (s32) R(); it_802F2014(g, a); } ret = (u32) ip->x378_itemColl.joint_id_only; ret = H(ret, r4_h(ip)); });
    T(1010, "R5H_CAPSULE_M3_PHYS", { r4_pre(ip); r2_common.x68_float = RM(); itCapsule_UnkMotion3_Phys(g); ret = H(ret, r4_h(ip)); });
    T(1011, "R5H_GSHELL_M4_ANIM", { r4_pre(ip); ret = (u32) itGshell_UnkMotion4_Anim(g); ret = H(ret, r4_h(ip)); });
    T(1012, "R5H_SAMUSGRAPPLE_SPAWNED", { r4_pre(ip); itSamusGrapple_Logic53_Spawned(g); ret = H(ret, r4_h(ip)); });
    T(1013, "R5H_STARROD_M1_PHYS", { r4_pre(ip); itStarrod_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(1014, "R5H_ZELDADINFIREEXPLODE_M0", { r4_pre(ip); itZeldadinfireexplode_UnkMotion0_Phys(g); ret = H(ret, r4_h(ip)); });
    T(1015, "R5H_802A2418", { r4_pre(ip); it_802A2418(g); ret = H(ret, r4_h(ip)); });
    T(1016, "R5H_802EAF28", { r4_pre(ip); ret = (u32) it_802EAF28(g); ret = H(ret, r4_h(ip)); });
    T(1017, "R5H_SAMUSBOMB_EVTUNK", { r4_pre(ip); itSamusBomb_Logic50_EvtUnk(g, PTR(R())); ret = H(ret, r4_h(ip)); });
    T(1018, "R5H_TOSAKINTO_M2_PHYS", { r4_pre(ip); itTosakinto_UnkMotion2_Phys(g); ret = H(ret, r4_h(ip)); });
    T(1019, "R5H_2E5A_M1_ANIM", { r4_pre(ip); ret = (u32) it_2E5A_UnkMotion1_Anim(g); ret = H(ret, r4_h(ip)); });
    T(1020, "R5H_8028428C", { r4_pre(ip); it_8028428C(g); ret = H(ret, r4_h(ip)); });
    T(1021, "R5H_8028EC98", { r4_pre(ip); { f32 v = RF(); r2_common.x30_lifetime = R() % 4096; it_8028EC98(g, v); } ret = H(ret, r4_h(ip)); });
    T(1022, "R5H_80292EF8", { r4_pre(ip); it_80292EF8(g, &ov); ret = H(ret, r4_h(ip)); });
    T(1023, "R5H_802960B8", { r4_pre(ip); ret = (u32) it_802960B8(g); ret = H(ret, r4_h(ip)); });
    T(1024, "R5H_802960CC", { r4_pre(ip); ret = (u32) it_802960CC(g); ret = H(ret, r4_h(ip)); });
    T(1025, "R5H_802E37A4", { r4_pre(ip); it_802E37A4(g); ret = H(ret, r4_h(ip)); });
    T(1026, "R5H_802F046C", { r4_pre(ip); it_802F046C(g); ret = H(ret, r4_h(ip)); });
    T(1027, "R5H_802F23AC", { r4_pre(ip); ret = bits64(it_802F23AC(g, &ov)); ret = H(ret, r4_h(ip)); });
    T(1028, "R5H_BAT_M3_PHYS", { r4_pre(ip); r2_common.x68_float = RM(); itBat_UnkMotion3_Phys(g); ret = H(ret, r4_h(ip)); });
    T(1029, "R5H_HAMMERHEAD_M2_PHYS", { r4_pre(ip); r2_common.x68_float = RM(); itHammerhead_UnkMotion2_Phys(g); ret = H(ret, r4_h(ip)); });
    T(1030, "R5H_UNKNOWN_M0_PHYS", { r4_pre(ip); itUnknown_UnkMotion0_Phys(g); ret = H(ret, r4_h(ip)); });
    T(1031, "R5H_ZRSHELL_M11_PHYS", { r4_pre(ip); itZrshell_UnkMotion11_Phys(g); ret = H(ret, r4_h(ip)); });
    T(1032, "R5H_27CF_M1_PHYS", { r4_pre(ip); it_27CF_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(1033, "R5H_8028D56C", { r4_pre(ip); { f32 a = RF(); f32 b = RF(); ret = bits(it_8028D56C(g, a, b)); } ret = H(ret, r4_h(ip)); });
    T(1034, "R5H_8028E774", { r4_pre(ip); it_8028E774(g, &ov); ret = H(ret, r4_h(ip)); });
    T(1035, "R5H_802D43B0", { r4_pre(ip); it_802D43B0(g, PTR(R())); ret = H(ret, r4_h(ip)); });
    T(1036, "R5H_802DFFA0", { r4_pre(ip); it_802DFFA0(g); ret = H(ret, r4_h(ip)); });
    T(1037, "R5H_802ECCA4", { r4_pre(ip); { s32 k = (s32) R(); it_802ECCA4(g, &k, &ov); ret = (u32) k; } ret = H(ret, r4_h(ip)); });
#define EVT(ID, NAME, FIELD, FN) \
    T(ID, NAME, { r4_pre(ip); { HSD_GObj* r = PTR(R()); if ((R() >> 16) & 1) ip->xDD4_itemVar.FIELD = (void*) r; FN(g, r); } ret = H(ret, r4_h(ip)); })
    EVT(1038, "R5H_CLIMBERSICE_EVTUNK", climbersice.x0, itClimbersIce_Logic90_EvtUnk);
    EVT(1039, "R5H_PIKACHUTHUNDER_EVTUNK", pikachuthunder.x38, itPikachuThunder_Logic39_EvtUnk);
    EVT(1040, "R5H_SAMUSGRAPPLE_EVTUNK", samusgrapple.x8, itSamusGrapple_Logic53_EvtUnk);
    EVT(1041, "R5H_SEAKCHAIN_EVTUNK", seakchain.parent_gobj, itSeakChain_Logic54_EvtUnk);
    EVT(1042, "R5H_2725_EVTUNK", samusmissile.owner, it_2725_Logic52_EvtUnk);
    EVT(1043, "R5H_802A7D40", linkhookshot.x8, it_802A7D40);
    EVT(1044, "R5H_8029FD84", linkbomb.x10, it_8029FD84);
    EVT(1045, "R5H_MEWTWODISABLE_EVTUNK", mdisable.owner, itMewtwoDisable_Logic67_EvtUnk);
    EVT(1046, "R5H_PEACHTURNIP_EVTUNK", peachturnip.xDE4_owner, itPeachTurnip_Logic56_EvtUnk);
    EVT(1047, "R5H_802A23CC", linkboomerang.xF98, it_802A23CC);
#undef EVT
    T(1048, "R5H_SWORD_DMGDEALT", { r4_pre(ip); if ((R() >> 16) & 1) ip->msid = 3; ret = (u32) itSword_Logic12_DmgDealt(g); ret = H(ret, r4_h(ip)); });
    T(1049, "R5H_TOMATO_M0_PHYS", { r4_pre(ip); itTomato_UnkMotion0_Phys(g); ret = H(ret, r4_h(ip)); });
    T(1050, "R5H_FOODS_M0_PHYS", { r4_pre(ip); itFoods_UnkMotion0_Phys(g); ret = H(ret, r4_h(ip)); });
    T(1051, "R5H_27CF_DMGRECEIVED", { r4_pre(ip); ret = (u32) it_27CF_Logic114_DmgReceived(g); ret = H(ret, r4_h(ip)); });
    T(1052, "R5H_8028ECF0", { r4_pre(ip); ret = PIDX(it_8028ECF0(g, &ov)); ret = H(ret, r4_h(ip)); });
    T(1053, "R5H_TARU_M7_ANIM", { r4_pre(ip); ret = (u32) itTaru_UnkMotion7_Anim(g); ret = H(ret, r4_h(ip)); });
    T(1054, "R5H_TARUCANN_M2_ANIM", { r4_pre(ip); if ((R() >> 16) & 1) ip->xDD4_itemVar.tarucann.x20 = NULL; ret = (u32) itTarucann_UnkMotion2_Anim(g); ret = H(ret, r4_h(ip)); });
    T(1055, "R5H_WSTAR_M5_ANIM", { r4_pre(ip); ret = (u32) itWstar_UnkMotion5_Anim(g); ret = H(ret, r4_h(ip)); });
    T(1056, "R5H_802B43B0", { r4_pre(ip); it_802B43B0(g); ret = H(ret, r4_h(ip)); });
    T(1057, "R5H_802AC5D8", { r4_pre(ip); it_802AC5D8(g); ret = H(ret, r4_h(ip)); });
    T(1058, "R5H_802B3EFC", { r4_pre(ip); ret = praw(it_802B3EFC(g)); ret = H(ret, r4_h(ip)); });
    T(1059, "R5H_802C3D44", { r4_pre(ip); it_802C3D44(g); ret = H(ret, r4_h(ip)); });
    T(1060, "R5H_802AB568", { r4_pre(ip); ret = PIDX(it_802AB568(g)); ret = H(ret, r4_h(ip)); });
    T(1061, "R5H_802B3368", { r4_pre(ip); ret = praw(it_802B3368(g)); ret = H(ret, r4_h(ip)); });
    T(1062, "R5H_802AAA50", { r4_pre(ip); it_802AAA50(g); ret = H(ret, r4_h(ip)); });
    T(1063, "R5H_WHISPYAPPLE_M0_PHYS", { r4_pre(ip); ip->xDD4_itemVar.whispyapple.xDD8 = (s32) (R() % 3); itWhispyapple_UnkMotion0_Phys(g); ret = H(ret, r4_h(ip)); });
    T(1064, "R5H_27CF_M2_ANIM", { r4_pre(ip); ret = (u32) it_27CF_UnkMotion2_Anim(g); ret = H(ret, r4_h(ip)); });
    T(1065, "R5H_80284FC4", { r4_pre(ip); { s32 a = (s32) (R() % 9) - 4; f32 b = RF(); it_80284FC4(g, a, b); } ret = H(ret, r4_h(ip)); });
    T(1066, "R5H_80285024", { r4_pre(ip); { s32 a = (s32) (R() % 9) - 4; it_80285024(g, a); } ret = H(ret, r4_h(ip)); });
    T(1067, "R5H_802A8398", { r4_pre(ip); { Vec3 p, q; p.x = RF(); p.y = RF(); p.z = RF(); q.x = RF(); q.y = RF(); q.z = RF(); it_802A8398(g, &p, &q); } ret = H(ret, r4_h(ip)); });
    T(1068, "R5H_802B1FE8", { r4_pre(ip); it_802B1FE8(g, &ov); ret = H(ret, r4_h(ip)); });
    T(1069, "R5H_FN_802C5E18", { r4_pre(ip); fn_802C5E18(g); ret = H(ret, r4_h(ip)); });
    T(1070, "R5H_FLIPPER_INFLIGHT_PHYS", { r4_pre(ip); ip->xDD4_itemVar.flipper.xDD4_flightTimer = (s32) ((R() >> 16) % 16) - 2; r2_common.x68_float = RM(); itFlipper_Inflight_Phys(g); ret = H(ret, r4_h(ip)); });
    T(1071, "R5H_802852B8", { r4_pre(ip); { f32 a = RF(); f32 b = RF(); u32 m = (R() >> 16) % 4; it_802852B8(g, (m & 1) ? &a : NULL, (m & 2) ? &b : NULL); ret = H(bits(a), bits(b)); } ret = H(ret, r4_h(ip)); });
    T(1072, "R5H_802AB3F0", { r4_pre(ip); { s32 i = (s32) ((R() >> 16) % 8); u32 m = (R() >> 16) % 2; it_802AB3F0(g, m ? &ov : NULL, i); } ret = H(ret, r4_h(ip)); });
    T(1073, "R5H_THUNDER_M2_PHYS", { r4_pre(ip); itThunder_UnkMotion2_Phys(g); ret = H(ret, r4_h(ip)); });
    T(1074, "R5H_SSCOPEBEAM_M9_ANIM", { r4_pre(ip); { s32 i = (s32) (R() % 9); ip->xDD4_itemVar.scopebeam.x0 = i;
        if ((R() >> 16) & 1) ip->xD44_lifeTimer = fbits(r4_sa[i * 3 + 2]); ip->x5D4_hitboxes[0].hit.state = (HitCapsuleState) (R() % 3);
        ret = (u32) itSscopebeam_UnkMotion9_Anim(g); ret = H(ret, bits(ip->x5D4_hitboxes[0].hit.scale)); } ret = H(ret, r4_h(ip)); });
    T(1075, "R5H_802AB468", { r4_pre(ip); { f32 o = RF(); u32 m = (R() >> 16) % 2; it_802AB468(g, m ? &o : NULL, 0); ret = bits(o); } ret = H(ret, r4_h(ip)); });
    T(1076, "R5H_80285084", { r4_pre(ip); it_80285084(g); ret = H(ret, r4_h(ip)); });
    T(1077, "R5H_80285140", { r4_pre(ip); it_80285140(g); ret = H(ret, r4_h(ip)); });
    T(1078, "R5H_802851FC", { r4_pre(ip); it_802851FC(g); ret = H(ret, r4_h(ip)); });
    T(1079, "R5H_802D48B0", { { f32 v = RF(); f32 t = RF(); f32 mx = RF(); f32 ac = RF(); f32 dc = RF(); f32 r = it_802D48B0(&v, t, mx, ac, dc); ret = H(bits(v), bits(r)); } });
    T(1176, "R5H_80285084_SMALL", { r4_pre(ip); r5_sword(ip); it_80285084(g); ret = H(ret, r4_h(ip)); });
    T(1177, "R5H_80285140_SMALL", { r4_pre(ip); r5_sword(ip); it_80285140(g); ret = H(ret, r4_h(ip)); });
    T(1178, "R5H_802851FC_SMALL", { r4_pre(ip); r5_sword(ip); it_802851FC(g); ret = H(ret, r4_h(ip)); });
    /* targeted: moderate inputs where the fused sites round differently */
    T(1119, "R5H_2E5A_M1_ANIM_FMA", { r4_pre(ip); ip->x40_vel.y = RM(); ret = (u32) it_2E5A_UnkMotion1_Anim(g); ret = H(ret, r4_h(ip)); });
    T(1127, "R5H_802F23AC_FMA", { r4_pre(ip); ip->pos.x = RM(); ip->pos.y = RM(); ip->pos.z = RM(); ov.x = RM(); ov.y = RM(); ov.z = RM();
        ret = bits64(it_802F23AC(g, &ov)); ret = H(ret, r4_h(ip)); });
    T(1130, "R5H_UNKNOWN_M0_PHYS_FMA", { r4_pre(ip); ip->xDD4_itemVar.unknown.x60 = RM(); r4_sa[1] = bits(RM()); r4_sa[3] = bits(1e30f); r4_sa[4] = bits(1e30f); ip->x40_vel.y = 0.0f;
        itUnknown_UnkMotion0_Phys(g); ret = H(ret, r4_h(ip)); });
    T(1133, "R5H_8028D56C_FMA", { r4_pre(ip); { f32 a = RM(); f32 b = RM(); ret = bits(it_8028D56C(g, a, b)); } ret = H(ret, r4_h(ip)); });
    T(1151, "R5H_27CF_DMGRECEIVED_FMA", { r4_pre(ip); ip->xCA0 = (s32) (R() % 200); ip->xD44_lifeTimer = RM(); ip->xDD4_itemVar.yoshiegglay.x0 = RM();
        ret = (u32) it_27CF_Logic114_DmgReceived(g); ret = H(ret, r4_h(ip)); });
}
#undef it_804D6D28
#undef it_80275158
#undef decrease_lifetimer
