/* Round 12 (night 3, round 4) part of the melee_items C oracle; #included by oracle.c after
 * round11.c and reusing round 2's LCG / Item fill+snap, round 4's r4_pre / r4_h, round 7's vector
 * draws, round 10's r10_coll / r10_h (and its it_80277040) and round 11's JObj chain / model desc.
 * Checks src/itkinds12.rs (15 hand ports; itmaplib.c's sqrtf_accurate_sp18 is round 9's copy) against
 * round12_bodies.h, verbatim copies of the decomp C extracted by gen_round12.py. The PC build has
 * no MUST_MATCH, so it_26B1.c's my_sqrtf is MSL sqrtf (it_26B1.c:59). Built like the rest:
 * contraction on + gwtool lowering (gen_expected.sh). */
#ifndef GET_FIGHTER
#define GET_FIGHTER(gobj) ((Fighter*) HSD_GObjGetUserData(gobj)) /* ft/inlines.h */
#endif
#define my_sqrtf(x) sqrtf(x)
#define it_804D6D28 it_804D6D28_o
#define HSD_GObjPLinkHead r3_plink
#define Item_804A0C64 r12_unk
#define it_804D6D24 r12_tbl
#define it_804D6D08 r12_c08
#define it_804D6D0C r12_c0c
#define it_804D6D18 r12_c18
static HSD_ObjAllocUnk r12_unk;
static Article** r12_tbl;
static s32 r12_c08, r12_c0c;
static u32 r12_c18;
/* pc/gameworld/mtx_pc.c PSVECAdd, verbatim (the Rust side writes the three adds out) */
void PSVECAdd(Vec* a, Vec* b, Vec* c)
{
    c->x = a->x + b->x;
    c->y = a->y + b->y;
    c->z = a->z + b->z;
}
#include "round12_bodies.h"
#undef it_804D6D18
#undef it_804D6D0C
#undef it_804D6D08
#undef it_804D6D24
#undef Item_804A0C64
#undef HSD_GObjPLinkHead
#undef it_804D6D28
#undef my_sqrtf

/* three items on the p-link list for it_8026C258 */
static HSD_GObj r12_gl[3];
static Item r12_li[3];
static HSD_GObj* r12_heads[16];
static void r12_list(void)
{
    u32 k;
    memset(r12_gl, 0, sizeof r12_gl);
    for (k = 0; k < 3; k++) {
        Item* li = &r12_li[k];
        memset(li, 0, sizeof *li);
        li->hold_kind = (enum_t) (3 + (R() >> 16) % 6); /* 3..8: 4..7 pass */
        li->grab_victim = (R() & 3) ? NULL : &r12_gl[0];
        flag32_set(&li->xDC8_word, R());
        li->owner = (R() & 1) ? NULL : &r12_gl[1];
        li->pos.x = RM() / 4.0f; li->pos.y = RM() / 4.0f; li->pos.z = RM();
        r12_gl[k].user_data = li;
        r12_gl[k].next = k < 2 ? &r12_gl[k + 1] : NULL;
    }
    memset(r12_heads, 0, sizeof r12_heads);
    r12_heads[HSD_GOBJ_PLINK_ITEM] = (R() % 5) ? &r12_gl[0] : NULL;
    r3_plink = r12_heads;
}
static u32 r12_gidx(HSD_GObj* g)
{
    u32 k;
    if (g == NULL) return 3;
    for (k = 0; k < 3; k++) if (g == &r12_gl[k]) return k;
    return 4;
}
static f32 r12_face(void)
{
    u32 s = (R() >> 16) % 4;
    return s == 0 ? -1.0f : (s == 1 ? 1.0f : (s == 2 ? RF() : 0.5f));
}
/* Item_804A0C64: small counters so the compares go both ways */
static u32 r12_unk_fill(void)
{
    s32* w = (s32*) &r12_unk;
    u32 k, n = sizeof r12_unk / 4;
    for (k = 0; k < n; k++) w[k] = (s32) ((R() >> 16) % 4) - 1;
    return n;
}
static u32 r12_unk_h(void)
{
    u32* w = (u32*) &r12_unk;
    u32 k, h = 0, n = sizeof r12_unk / 4;
    for (k = 0; k < n; k++) h = H(h, w[k]);
    return h;
}
/* the L-Gun article table */
static Article* r12_arts[20];
static Article r12_lgun_art;
static itLGun_ItemVars r12_lgun;
static void r12_ev(HSD_GObj* g) { (void) g; }
static void r12_ev2(HSD_GObj* g) { (void) g; }
static void r12_in(HSD_GObj* a, HSD_GObj* b) { (void) a; (void) b; }
static void r12_in2(HSD_GObj* a, HSD_GObj* b) { (void) a; (void) b; }
static u32 r12_evi(HSD_GObjEvent e) { return e == NULL ? 0 : (e == r12_ev ? 1 : 2); }
static u32 r12_ini(HSD_GObjInteraction e) { return e == NULL ? 0 : (e == r12_in ? 1 : 2); }
static u32 r12_dd0(Item* ip)
{
    return (ip->xDD0_flag.b0 << 7) | (ip->xDD0_flag.b1 << 6) | (ip->xDD0_flag.b2 << 5) | (ip->xDD0_flag.b3 << 4) |
           (ip->xDD0_flag.b4 << 3) | (ip->xDD0_flag.b5 << 2) | (ip->xDD0_flag.b6 << 1) | ip->xDD0_flag.b7;
}
static Fighter r12_fp;
static HSD_GObj r12_fg;
static u32 r12_atk(Item* ip)
{
    u32 h = H(H(0, (u32) ip->xD88_attackID), (u32) ip->xD8C_attack_instance);
    u32 w; memcpy(&w, &ip->xD90, 4);
    h = H(H(H(h, w), bits(ip->xD94.x)), bits(ip->xD94.y));
    h = H(H(H(h, (u32) ip->xD9C.x), (u32) ip->xD9C.y), ip->xDA4_word);
    return H(h, ip->xDA8_short);
}
/* selectors for the fused-site tables: one f32 rounding of an exact f64 product + addend (the
 * f32 fma, up to rare double rounding; both sides use the same emulation), and plain f32 ops kept
 * apart by volatile so this TU's contraction cannot fuse them */
static f32 r12_fm(f32 a, f32 b, f32 c) { volatile f64 p = (f64) a * (f64) b; volatile f64 q = p + (f64) c; return (f32) q; }
static f32 r12_mul(f32 a, f32 b) { volatile f32 r = a * b; return r; }
static f32 r12_add(f32 a, f32 b) { volatile f32 r = a + b; return r; }
/* it_8026B1D4's value: var 0 as the C contracts it, 1 the outer add unfused, 2 the inner swapped */
static f32 r12_b1d4(Vec3* v, f32 k5, f32 k6, f32 dmg, int var)
{
    f32 xx = r12_mul(v->x, v->x), yy = r12_mul(v->y, v->y), zz = r12_mul(v->z, v->z);
    f32 in = var == 2 ? r12_fm(v->y, v->y, xx) : r12_fm(v->x, v->x, yy);
    f32 sq = var == 1 ? r12_add(zz, in) : r12_fm(v->z, v->z, in);
    f32 r = r12_add(r12_fm(sqrtf(sq), k5, dmg), k6);
    return r <= 1.0 ? 1.0f : r;
}
static void r12_b1d4_pick(Item* ip, HitCapsule* hc, int var, s32* kk)
{
    s32 k;
    for (k = 0; k < 4000; k++) {
        r7_vin(&ip->x40_vel); r2_common.x80_float[5] = RM() / 64.0f; r2_common.x80_float[6] = RM() / 64.0f; hc->damage = RM() / 8.0f;
        if (bits(r12_b1d4(&ip->x40_vel, r2_common.x80_float[5], r2_common.x80_float[6], hc->damage, 0)) !=
            bits(r12_b1d4(&ip->x40_vel, r2_common.x80_float[5], r2_common.x80_float[6], hc->damage, var)))
            break;
    }
    *kk = k;
}
/* it_80277544's draws (as case 3213) */
static Item r12_tmp;
static HSD_GObj r12_tg;
static void r12_544_draw(Item* ip)
{
    r4_pre(ip); r10_coll(ip); r2_attr.x50 = ((R() >> 16) % 4) ? r7_c() : RM() / 16777216.0f; r7_vin(&ip->x88); r7_vin(&ip->xAC_unk); ip->xC2C = (s32) ((R() >> 16) % 3); ip->xC30 = (s32) ((R() >> 16) % 3); ip->xD58 = (R() >> 16) % 4; ip->xD5C = 1 + (R() >> 16) % 2; r2_common.xC0 = RM() / 64.0f; r2_common.xC4 = RM() / 64.0f; r2_common.x38_float = (R() >> 16) % 4; r7_vin(&ip->x40_vel); r7_vin(&ip->x70_nudge);
}
/* mode 0: it_80277040 true and |vel + nudge|^2 lands exactly on xCC fused but below it unfused;
 * mode 1: it_80277040 true and xCC = FLT_MAX (so only xD5C == 2 decides) */
static s32 r12_544_pick(Item* ip, int mode)
{
    s32 k;
    for (k = 0; k < 2000; k++) {
        r12_544_draw(ip);
        r12_tmp = *ip; memset(&r12_tg, 0, sizeof r12_tg); r12_tg.user_data = &r12_tmp;
        if (!it_80277040(&r12_tg)) continue;
        if (mode == 1) { r2_common.xCC = fbits(0x7F7FFFFFu); break; } else {
            f32 sx = r12_add(r12_tmp.x40_vel.x, r12_tmp.x70_nudge.x), sy = r12_add(r12_tmp.x40_vel.y, r12_tmp.x70_nudge.y);
            r2_common.xCC = r12_fm(sx, sx, r12_mul(sy, sy));
            if (r12_add(r12_mul(sx, sx), r12_mul(sy, sy)) < r2_common.xCC) break;
        }
    }
    return k;
}

static void round12(void)
{
    T(3201, "R12_8026B1D4", { HitCapsule hc; memset(&hc, 0, sizeof hc); hc.damage = (R() & 3) ? RM() / 8.0f : RF(); r7_vin(&ip->x40_vel); r2_common.x80_float[5] = RM() / 64.0f; r2_common.x80_float[6] = RM() / 64.0f; ret = bits(it_8026B1D4(g, &hc)); });
    T(3202, "R12_8026C258", { Vec3 p; f32 fd; r12_list(); p.x = RM() / 4.0f; p.y = RM() / 4.0f; p.z = RM(); if ((R() & 3) == 0) { p.x = r12_li[R() % 3].pos.x; } fd = r12_face(); ret = r12_gidx(it_8026C258(&p, fd)); });
    T(3203, "R12_ITEM_8026784C", { u32 d = (R() >> 16) % 14; r12_unk_fill(); ret = (u32) Item_8026784C((enum_t) d, (int) R()); ret = H(ret, r12_unk_h()); });
    T(3204, "R12_8026C1D4", { u32 k; for (k = 0; k < 20; k++) r12_arts[k] = NULL; memset(&r12_lgun_art, 0, sizeof r12_lgun_art); r12_lgun.timer = (s32) R(); r12_lgun_art.x4_specialAttributes = &r12_lgun; r12_arts[It_Kind_L_Gun] = &r12_lgun_art; r12_tbl = r12_arts; ret = it_8026C1D4(); });
    T(3205, "R12_8026B7D8", { r12_c0c = (s32) R(); ret = (u32) it_8026B7D8(); });
    T(3206, "R12_8026B7E0", { r12_c08 = (s32) R(); ret = (u32) it_8026B7E0(); });
    T(3207, "R12_80274ED8", { r12_c0c = (R() & 1) ? 0x7FFFFFFF : (s32) R(); it_80274ED8(); ret = (u32) r12_c0c; });
    T(3208, "R12_80274EE8", { r12_c08 = (R() & 1) ? 0x7FFFFFFF : (s32) R(); it_80274EE8(); ret = (u32) r12_c08; });
    T(3209, "R12_8026F9A0", { r12_c18 = R(); it_8026F9A0(); ret = r12_c18; });
    T(3210, "R12_80272C90", { r11_jobjs(g); r11_model(ip); r11_md.x8_bone_attach_id = (s32) ((R() >> 16) % 6); ret = r11_jidx(it_80272C90(g)); });
    T(3211, "R12_802746F8", { u32 k = R() % 4; r11_jobjs(g); r11_model(ip); if (k == 0) g->hsd_obj = NULL; else if (k == 1) r11_j[1].child = NULL; ret = r11_jidx(it_802746F8(g)); });
    T(3212, "R12_80274F28", { HSD_GObjEvent e; HSD_GObjInteraction in; u32 a = R() % 3, b = R() % 3; e = a == 0 ? NULL : (a == 1 ? r12_ev : r12_ev2); in = b == 0 ? NULL : (b == 1 ? r12_in : r12_in2); ip->xD08 = (u8) R(); it_80274F28(ip, (s8) R(), e, in); ret = H(H(H(H(0, r12_dd0(ip)), ip->xD08), r12_evi(ip->grab_dealt)), r12_ini(ip->grabbed_for_victim)); });
    T(3213, "R12_80277544", { r4_pre(ip); r10_coll(ip); r2_attr.x50 = ((R() >> 16) % 4) ? r7_c() : RM() / 16777216.0f; r7_vin(&ip->x88); r7_vin(&ip->xAC_unk); ip->xC2C = (s32) ((R() >> 16) % 3); ip->xC30 = (s32) ((R() >> 16) % 3); ip->xD58 = (R() >> 16) % 4; ip->xD5C = (R() >> 16) % 3; r2_common.xC0 = RM() / 64.0f; r2_common.xC4 = RM() / 64.0f; r2_common.x38_float = (R() >> 16) % 4; r7_vin(&ip->x40_vel); r7_vin(&ip->x70_nudge); r2_common.xCC = (R() & 1) ? RM() : r7_c(); ret = it_80277544(g); ret = H(ret, r10_h(ip)); });
    T(3214, "R12_802775F0", { Vec3 a; ip->spin_spd = (R() & 3) ? RM() : 0.0f; r7_vin(&a); ip->xC1C.bottom = (R() & 7) ? RM() / 8.0f : 0.0f; ip->facing_dir = r12_face(); it_802775F0(g, &a); ret = bits(ip->xD3C_spinSpeed); });
    T(3215, "R12_8027B070", { u32 w; memset(&r12_fp, 0, sizeof r12_fp); memset(&r12_fg, 0, sizeof r12_fg); r12_fp.x2068_attackID = (s32) R(); r12_fp.x206C_attack_instance = (u16) R(); w = R(); memcpy(&r12_fp.x2070, &w, 4); r12_fp.x2074.x2074_vec.x = RF(); r12_fp.x2074.x2074_vec.y = RF(); r12_fp.x2074.x207C.x = (s32) R(); r12_fp.x2074.x207C.y = (s32) R(); r12_fp.x2074.x2084 = R(); r12_fp.x2074.x2088 = (u16) R(); r12_fg.user_data = &r12_fp; it_8027B070(g, &r12_fg); ret = r12_atk(ip); });
    T(3216, "R12_8026B1D4_FUSED_OUTER", { HitCapsule hc; s32 k; memset(&hc, 0, sizeof hc); ip->xDC8_word.flags.x14 = 1; r12_b1d4_pick(ip, &hc, 1, &k); ret = H((u32) k, bits(it_8026B1D4(g, &hc))); });
    T(3217, "R12_8026B1D4_FUSED_INNER", { HitCapsule hc; s32 k; memset(&hc, 0, sizeof hc); ip->xDC8_word.flags.x14 = 1; r12_b1d4_pick(ip, &hc, 2, &k); ret = H((u32) k, bits(it_8026B1D4(g, &hc))); });
    /* items 0 and 1 eligible at the same squared distance from the origin: exactly (mode 0) or as
     * (a, b) / (b, a), where the fused sum puts item 1 strictly nearer (mode 1); item 2 not eligible */
    T(3218, "R12_8026C258_TIE", { Vec3 p; s32 k = 0; u32 m, i; r12_list(); for (i = 0; i < 3; i++) { r12_li[i].hold_kind = i < 2 ? 4 : 3; r12_li[i].grab_victim = NULL; r12_li[i].owner = NULL; } r3_plink[HSD_GOBJ_PLINK_ITEM] = &r12_gl[0]; p.x = 0.0f; p.y = 0.0f; p.z = RM(); m = (R() >> 16) & 1; /* the LCG's low bit alternates */
        if (m == 0) { r12_li[1].pos = r12_li[0].pos; } else { for (k = 0; k < 4000; k++) { f32 a = RM() / 4.0f, b = RM() / 4.0f; r12_li[0].pos.x = a; r12_li[0].pos.y = b; r12_li[1].pos.x = b; r12_li[1].pos.y = a; if (r12_fm(b, b, r12_mul(a, a)) < r12_fm(a, a, r12_mul(b, b))) break; } }
        ret = H(H(m, (u32) k), r12_gidx(it_8026C258(&p, 0.5f))); });
    T(3220, "R12_80277544_FUSED", { s32 k = r12_544_pick(ip, 0); ret = it_80277544(g); ret = H(H((u32) k, ret), r10_h(ip)); });
    T(3221, "R12_80277544_D5C", { s32 k = r12_544_pick(ip, 1); ret = it_80277544(g); ret = H(H((u32) k, ret), r10_h(ip)); });
    T(3222, "R12_ITEM_8026784C_ALL", { u32 d, rep; for (d = 0; d < 14; d++) for (rep = 0; rep < 4; rep++) { u32 r; r12_unk_fill(); r = (u32) Item_8026784C((enum_t) d, 0); ret = H(H(ret, r), r12_unk_h()); } });
}
