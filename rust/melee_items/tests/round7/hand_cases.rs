// Round 7 hand-port cases; included by tests/oracle_round7.rs. Each line mirrors the T(...)
// line with the same id in tests/c_oracle/round7.c (same RNG draws in the same order).
cases! { hand_match_c, 12;
    2401 R7H_802A3C98 |ip, g, ov, x| { let a = r7_vin(g); let mut b = r7_vin(g); if (g.r() >> 16) & 7 == 0 { b = a; } let l = it_802a3c98(&a, &b, ov); l.to_bits() };
    2402 R7H_NORMALIZE_DIFF |ip, g, ov, x| { let a = r7_vin(g); let mut b = r7_vin(g); if (g.r() >> 16) & 7 == 0 { b = a; } let l = it_802a6a78_normalize_diff(&a, &b, ov); r7_bitsd(l) };
    2403 R7H_NORMALIZE_DIFF_REV |ip, g, ov, x| { let a = r7_vin(g); let mut b = r7_vin(g); if (g.r() >> 16) & 7 == 0 { b = a; } let l = it_802a6a78_normalize_diff_rev(&a, &b, ov); r7_bitsd(l) };
    2404 R7H_VEC3_EQ_MTX |ip, g, ov, x| { let mut m: Mtx = [[0.0; 4]; 3]; for i in 0..3 { for j in 0..4 { m[i][j] = g.rf(); } } vec3_eq_mtx(ov, &m); 0 };
    2405 R7H_LINK_FIGHTER_COMPARE |ip, g, ov, x| { const IDS: [i32; 8] = [0x168, 0x169, 0xD4, 0xD6, 0x167, 0x16A, 0xD5, 0xD7]; let mut fp = Box::new(Fighter::zeroed()); fp.motion_id = if (g.r() >> 16) & 1 != 0 { IDS[((g.r() >> 16) % 8) as usize] } else { g.r() as i32 } as _; link_fighter_compare(&fp) as u32 };
    2406 R7H_2F28_M0_PHYS |ip, g, ov, x| { r4_pre(ip, g, x); if (g.r() >> 16) & 1 != 0 { ip.x40_vel.x = g.rm(); ip.x40_vel.y = g.rm(); ip.xDD4_itemVar.unk2.x0 = g.rm() / 16.0; } it_2f28_unk_motion0_phys(ip); hmix(0, r4_h(ip)) };
    2407 R7H_BISECTVALUE |ip, g, ov, x| { let n = 1 + ((g.r() >> 16) % 32) as i32; let mut tab = [0u16; 33]; let mut acc = 0i32; for k in 0..=n as usize { tab[k] = acc as u16; acc += ((g.r() >> 16) % 50) as i32; } let v = ((g.r() >> 16) % (acc as u32 + 1)) as i32; let mut t = ItemPickTable::zeroed(); t.xC = tab.as_mut_ptr(); let ret = bisect_value(v, &t, 0, n) as u32; hmix(ret, n as u32) };
    2408 R7H_802E614C |ip, g, ov, x| { let a = g.ptr(); let b = g.ptr(); let mut s = SpawnItem::zeroed(); s.kind = g.r() as _; s.x3C_damage = g.r() as i16; s.x40 = g.r() as i32; s.x44_flag.byte = ((g.r() >> 16) as u8).reverse_bits(); let p = Vec { x: g.rf(), y: g.rf(), z: g.rf() }; let v = Vec { x: g.rf(), y: g.rf(), z: g.rf() }; it_802e614c(a, b, &mut s, &p, &v); r7_spawn_h(&s, g) };
    2409 R7H_TARUCANN_M9_ANIM_INLINE |ip, g, ov, x| { r4_pre(ip, g, x); if (g.r() >> 16) & 1 != 0 { ip.xDD4_itemVar.tarucann.x20 = null_mut(); } it_tarucann_unk_motion9_anim_inline(ip); hmix(0, r4_h(ip)) };
    2410 R7H_KYASARIN_SETFALLCOLLPOS |ip, g, ov, x| { let mut c = Box::new(CollData::zeroed()); c.cur_pos = Vec { x: g.rf(), y: g.rf(), z: g.rf() }; it_kyasarin_set_fall_coll_pos(ip, &mut c, ov); r7_v(&c.cur_pos) };
    2411 R7H_ARWINGLASER_GETCOLLARG |ip, g, ov, x| { r4_pre(ip, g, x); let ret = it_arwinglaser_get_coll_arg(ip) as u32; hmix(ret, r4_h(ip)) };
    2412 R7H_GETRANDMAX |ip, g, ov, x| { r4_pre(ip, g, x); let ret = get_rand_max(&*ip.xC4_article_data); hmix(ret, r4_h(ip)) };
}
