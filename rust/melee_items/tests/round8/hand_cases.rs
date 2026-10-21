// Round 8 hand-port cases; included by tests/oracle_round8.rs. Each line mirrors the T(...)
// line with the same id in tests/c_oracle/round8.c (same RNG draws in the same order).
cases! { hand_match_c, 7;
    2801 R8H_GET_MIN_LIFE |ip, g, ov, x| { get_min_life().to_bits() };
    2802 R8H_802A13EC_INLINE |ip, g, ov, x| { r4_pre(ip, g, x); let ret = it_802a13ec_inline(ip).to_bits(); hmix(ret, r4_h(ip)) };
    2803 R8H_802A20E8_INLINE |ip, g, ov, x| { r4_pre(ip, g, x); let ret = it_802a20e8_inline(ip).to_bits(); hmix(ret, r4_h(ip)) };
    2804 R8H_802EAAEC_INLINE |ip, g, ov, x| { r4_pre(ip, g, x); let ret = it_802eaaec_inline(ip).to_bits(); hmix(ret, r4_h(ip)) };
    2805 R8H_80292394 |ip, g, ov, x| { r4_pre(ip, g, x); it_80292394(ip, ov); hmix(0, r4_h(ip)) };
    2806 R8H_DOSEI_M4_ANIMSPEED |ip, g, ov, x| { let mut a = itDoseiAttributes::zeroed(); a.unk0 = g.rf(); a.unk4 = g.r() as i32; it_dosei_unk_motion4_get_anim_speed(&a).to_bits() };
    2807 R8H_LINKBOOMERANG_X18 |ip, g, ov, x| { let mut a = itLinkBoomerangAttributes::zeroed(); a.x18 = g.rf(); it_linkboomerang_get_x18(&a).to_bits() };
}
