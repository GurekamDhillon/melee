//! src/tu/ftCo_DamageIce.rs vs the C (gen/cref/tu/ftCo_DamageIce.c, shipping contraction;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_DamageIce`).
#![allow(static_mut_refs, non_snake_case)]
#[path = "../../melee_core/test_support/fnsym.rs"]
mod fnsym;
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_DamageIce as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

static mut PARTS: *mut FighterBone = core::ptr::null_mut();
static mut JO: *mut HSD_JObj = core::ptr::null_mut();
static mut ATAN: f32 = 0.0;
static mut RANDF: f32 = 0.0;
fn isf(fr: &Fighter) -> u32 { (fr as *const Fighter == unsafe { F } as *const Fighter) as u32 }
fn jo(i: usize) -> *mut HSD_JObj { unsafe { JO.add(i) } }
fn jid(j: *mut HSD_JObj) -> u32 { if j.is_null() { 0 } else if j == jo(0) { 1 } else if j == jo(1) { 2 } else { 3 } }
fn tv(v: &Vec3) { t(b(v.x)); t(b(v.y)); t(b(v.z)); }
fn tm(mm: &Mtx) { for r in 0..3 { for c in 0..4 { t(b(mm[r][c])); } } }
fn icoll() -> *mut ftCollisionBox { unsafe { &mut f().mv.co.damageice.ice_coll as *mut _ } }
fn cbid(cb: HSD_GObjEvent) -> u32 {
    match cb {
        None => 0,
        Some(p) if fnsym::same(p as usize, m::ftCo_DamageIce_OnHit as usize) => 1,
        Some(p) if fnsym::same(p as usize, m::ftCo_DamageIce_OnHit2 as usize) => 2,
        _ => 3,
    }
}

#[test]
fn ftCo_DamageIce_matches_c() {
    init();
    unsafe {
        PARTS = Box::leak(Box::new([FighterBone::zeroed(); 8])).as_mut_ptr();
        JO = Box::leak(Box::new([HSD_JObj::zeroed(), HSD_JObj::zeroed()])).as_mut_ptr();
        hk::ftCo_80098928 = Some(|_| { t(1); sr(1) });
        hk::ftCo_8009872C = Some(|_| { t(2); sr(2) });
        hk::ftCo_80097D40 = Some(|g| { t(3); t(is_go(g)) });
        hk::ftParts_GetBoneIndex = Some(|fr, p| { t(4); t(isf(fr)); t(p as u32); (SRV[5] % 2 + 3) as i32 });
        hk::lb_8000B1CC = Some(|j, off, out| {
            t(5); t(jid(j)); t(off.is_null() as u32);
            out.x = (*j).translate.x; out.y = (*j).translate.y; out.z = (*j).rotate.x;
        });
        hk::lbVector_Sub = Some(|a, bb| { t(6); tv(a); tv(bb); a.x -= bb.x; a.y -= bb.y; a.z -= bb.z; });
        hk::ftCommon_UnlockECB = Some(|fr| { t(7); t(isf(fr)) });
        hk::ft_80082638 = Some(|g, bx| { t(8); t(is_go(g)); t((bx == icoll()) as u32); sr(3) });
        hk::ft_80082888 = Some(|g, bx| { t(9); t(is_go(g)); t((bx == icoll()) as u32); sr(4) });
        hk::efAsync_Spawn = Some(|g, q, kind, id, j, a1, a2| {
            t(10); t(is_go(g)); t((q == &mut f().x60C as *mut _ as *mut core::ffi::c_void) as u32); t(kind); t(id); t(jid(j));
            if kind == 3 {
                t(b(*(a1 as *mut f32)));
            } else {
                tv(&*(a1 as *mut Vec3)); t(b(*(a2 as *mut f32)));
            }
        });
        hk::ftCo_8009750C = Some(|g| { t(11); t(is_go(g)) });
        hk::ftCo_800DD168 = Some(|g| { t(12); t(is_go(g)) });
        hk::ftCommon_8007DB58 = Some(|g| { t(13); t(is_go(g)) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(14); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftCommon_InitGrab = Some(|fr, a, tm_| { t(15); t(isf(fr)); t(a as u32); t(b(tm_)) });
        hk::PSMTXTrans = Some(|mm, x, y, z| {
            t(16); t(b(x)); t(b(y)); t(b(z));
            for r in 0..3 { for c in 0..4 { mm[r][c] = (r * 4 + c) as f32 + y; } }
            mm[2][3] = z;
        });
        hk::HSD_MkRotationMtx = Some(|mm, v| {
            t(17); tv(v);
            for r in 0..3 { for c in 0..4 { mm[r][c] = v.x - (r * 4 + c) as f32; } }
            mm[1][3] = v.z;
        });
        hk::HSD_MtxQuat = Some(|mm, q| {
            t(18); t(b(q.x)); t(b(q.y)); t(b(q.z)); t(b(q.w));
            for r in 0..3 { for c in 0..4 { mm[r][c] = q.w + (r as i32 - c as i32) as f32; } }
        });
        hk::PSMTXTranspose = Some(|s, d| { t(19); tm(s); for r in 0..3 { for c in 0..4 { d[r][c] = s[r][c] * 0.5; } } });
        hk::PSMTXConcat = Some(|a, bb, ab| { t(20); tm(a); tm(bb); for r in 0..3 { for c in 0..4 { ab[r][c] = a[r][c] - bb[r][c]; } } });
        hk::HSD_MtxGetTranslate = Some(|mm, v| { t(21); tm(mm); v.x = mm[0][3]; v.y = mm[1][3]; v.z = mm[2][3]; });
        hk::HSD_JObjSetMtxDirtySub = Some(|j| { t(22); t(jid(j)) });
        hk::ftCo_8009E140 = Some(|fr, bb| { t(23); t(isf(fr)); t(bb as u32) });
        hk::ftCommon_8007F824 = Some(|g| { t(24); t(is_go(g)) });
        hk::ftCommon_8007E2F4 = Some(|fr, v| { t(25); t(isf(fr)); t(v as i32 as u32) });
        hk::HSD_Randf = Some(|| { t(26); RANDF });
        hk::ftColl_8007B0C0 = Some(|g, s| { t(27); t(is_go(g)); t(s as u32) });
        hk::ftColl_HurtboxInit = Some(|fr, h, i| {
            t(28); t(isf(fr)); t((h == fr.hurt_capsules.as_mut_ptr()) as u32);
            t(i.bone_idx as u32); t(i.height as u32); t(i.is_grabbable); tv(&i.a_offset); tv(&i.b_offset); t(b(i.scale));
        });
        hk::ftCommon_8007EBAC = Some(|fr, a, bb| { t(29); t(isf(fr)); t(a); t(bb) });
        hk::ftCo_800886D8 = Some(|fr, id, v, p| { t(30); t(isf(fr)); t(id as u32); t(v as u32); t(p as u32) });
        hk::ftCommon_GrabMash = Some(|fr, v| { t(31); t(isf(fr)); t(b(v)); sr(5) });
        hk::ftCommon_CalcSelfAccel_DeaccelAir = Some(|fr| { t(32); t(isf(fr)) });
        hk::ftCommon_Fall = Some(|fr, a, bb| { t(33); t(isf(fr)); t(b(a)); t(b(bb)) });
        hk::ft_80084F3C = Some(|g| { t(34); t(is_go(g)) });
        hk::ftCommon_8007D5D4 = Some(|fr| { t(35); t(isf(fr)) });
        hk::ft_800824A0 = Some(|g, bx| { t(36); t(is_go(g)); t((bx == icoll()) as u32); sr(6) });
        hk::ftKb_SpecialN_800F1F1C = Some(|g, v| { t(37); t(is_go(g)); tv(v); v.z = (SRV[7] % 5) as f32; });
        hk::ftCommon_8007D7FC = Some(|fr| { t(38); t(isf(fr)) });
        hk::atan2f = Some(|y, x| { t(39); t(b(y)); t(b(x)); ATAN });
        hk::Camera_RequestQuake = Some(|k, v| { t(40); t(k as u32); tv(&*v) });
        hk::ft_PlaySFX = Some(|fr, id, a, bb| { t(41); t(isf(fr)); t(id as u32); t(a as u32); t(bb as u32) });
        hk::ftCo_80090780 = Some(|g| { t(42); t(is_go(g)) });
        hk::lbVector_Mirror = Some(|a, n| { t(43); tv(a); tv(n); a.x = -a.x; a.y = a.y + n.y; });
        hk::ft_80081D0C = Some(|_| { t(44); sr(7) as i32 });
        hk::ftCo_800C5240 = Some(|_| { t(45); sr(8) });
        hk::ftCo_800C5A98 = Some(|g| { t(46); t(is_go(g)) });
        hk::ftCo_Fall_Enter = Some(|g| { t(47); t(is_go(g)) });
        hk::ft_80084EEC = Some(|g| { t(48); t(is_go(g)) });
        hk::ftCo_80096CC8 = Some(|g, l| { t(49); t(is_go(g)); t(l as u32); sr(9) });
        hk::ft_80082B1C = Some(|g| { t(50); t(is_go(g)) });
        hk::ft_800831CC = Some(|g, a, bb| { t(51); t(is_go(g)); let r = a(g, 7); t(r as u32); bb(g) });
    }
    let jf = [0u32, JOBJ_MTX_DIRTY as u32, JOBJ_USE_QUATERNION as u32, JOBJ_MTX_INDEP_SRT as u32, JOBJ_USER_DEF_MTX as u32];
    let mut c = Check::new(include_str!("data/tu_ftCo_DamageIce.txt"));
    for op in 0..20u32 {
        for k in 0..3000u32 {
            case(op, k);
            unsafe {
                for i in 0..8 { *PARTS.add(i) = FighterBone::zeroed(); (*PARTS.add(i)).joint = jo(i & 1); }
                for i in 0..2 {
                    *jo(i) = HSD_JObj::zeroed();
                }
                for i in 0..2 {
                    let a = jf[(rnd() % 5) as usize];
                    let bb = jf[(rnd() % 5) as usize];
                    let j = &mut *jo(i);
                    j.flags = a | bb;
                    j.translate.x = rw(); j.translate.y = rw(); j.translate.z = rw();
                    j.rotate.x = rw(); j.rotate.y = rw(); j.rotate.z = rw(); j.rotate.w = rw();
                }
                ATAN = rg();
                RANDF = rg();
            }
            let d = cd();
            d.x790_damageice_unk = rw(); d.x794_damageice_unk = rw(); d.x798_damageice_unk = rg();
            d.damageice_rot_speed_min = rw(); d.damageice_rot_speed_max = rw();
            d.damageice_dmg_time_reduction_mult = rw(); d.damageice_ice_size = rw();
            d.damageice_gravity_mult = rw(); d.damageice_min_speed = rw();
            d.damageice_speed_mult_on_break = rw(); d.damageicejump_escape_time = rw();
            let fr = f();
            fr.parts = unsafe { PARTS };
            fr.gobj = go();
            fr.x34_scale.y = rw();
            fr.co_attrs.damageice_ice_size = rw(); fr.co_attrs.x150_damageice_unk = rw();
            fr.co_attrs.x154_damageice_unk = rw(); fr.co_attrs.damageicejump_vel_y = rw();
            fr.co_attrs.damageicejump_vel_x_mult = rw(); fr.co_attrs.gravity = rw();
            fr.co_attrs.terminal_velocity = rw();
            fr.ground_or_air = (rnd() & 1) as _;
            fr.self_vel.x = rw(); fr.self_vel.y = rw(); fr.self_vel.z = rw();
            fr.x8c_kb_vel.x = rw(); fr.x8c_kb_vel.y = rw(); fr.x8c_kb_vel.z = rw();
            fr.gr_vel = rw(); fr.xF0_ground_kb_vel = rw();
            fr.dmg.x1838_percentTemp = rw();
            fr.dmg.x1860_element = if rnd() & 1 != 0 { HitElement_Fire as u32 } else { rnd() % 16 };
            fr.grab_timer = rw();
            fr.set_x2224_b2((rnd() & 1) as u8);
            unsafe {
                fr.mv.co.damageice.rot_speed = rw();
                fr.mv.co.damageice.wall_hit_dir = (rnd() % 4) as u8;
            }
            fr.cur_anim_frame = (rnd() % 7) as f32;
            fr.coll_data.env_flags = (rnd() & (Collide_RightWallHug | Collide_LeftWallHug | Collide_CeilingHug) as u32) as _;
            fr.coll_data.ecb.left.x = rw(); fr.coll_data.ecb.left.y = rw();
            fr.coll_data.ecb.right.x = rw(); fr.coll_data.ecb.right.y = rw();
            fr.coll_data.ecb.top.y = rw();
            fr.coll_data.right_facing_wall.normal.x = rg(); fr.coll_data.right_facing_wall.normal.y = rg();
            fr.coll_data.left_facing_wall.normal.x = rg(); fr.coll_data.left_facing_wall.normal.y = rg();
            fr.coll_data.ceiling.normal.x = rg(); fr.coll_data.ceiling.normal.y = rg();
            fr.cur_pos.x = rw(); fr.cur_pos.y = rw(); fr.cur_pos.z = rw();
            fr.x68C_transNPos.y = rw(); fr.x68C_transNPos.z = rw();
            fr.facing_dir = rg();
            fr.input.lstick[0].x = rg();
            if op == 16 {
                unsafe { fr.mv.co.damageicejump.escape_timer = ((rnd() % 5) as i32 - 1) as f32 };
            }
            let g = go();
            let ret = match op {
                0 => { m::ftCo_80090984(g); 0 }
                1 => { m::ftCo_800909D0(f()); 0 }
                2 => { m::ftCo_80090AC0(f()); 0 }
                3 => { unsafe { m::ftCo_DamageIce_OnHit(g) }; 0 }
                4 => { m::ftCo_DamageIce_Init(g); 0 }
                5 => { m::ftCo_DamageIce_HitWhileFrozen(g); 0 }
                6 => { unsafe { m::ftCo_DamageIce_OnHit2(g) }; 0 }
                7 => { m::ftCo_DamageIce_Anim(g); 0 }
                8 => { m::ftCo_DamageIce_IASA(g); 0 }
                9 => { m::ftCo_DamageIce_Phys(g); 0 }
                10 => { m::ftCo_DamageIce_Coll(g); 0 }
                11 => { m::ftCo_DamageIce_InAirUpdate(g); 0 }
                12 => {
                    let n = Vec3 { x: rg(), y: rg(), z: rg() };
                    let v = Vec3 { x: rw(), y: rw(), z: rw() };
                    m::ftCo_DamageIce_Collide(g, &n, &v);
                    0
                }
                13 => { m::ftCo_80091854(g); 0 }
                14 => { m::ftCo_DamageIceJump_IASA(g); 0 }
                15 => { m::ftCo_DamageIceJump_Phys(g); 0 }
                16 => { m::ftCo_DamageIceJump_Anim(g); 0 }
                17 => { m::ftCo_DamageIceJump_Coll(g); 0 }
                18 => b(m::ftCo_Speed(f())),
                _ => { let mut p = rw(); m::ftCo_SpawnEffect_x415(g, jo(1), &mut p); 0 }
            };
            let f = f();
            let mut h = Hs::new();
            h.m(f.x2219_b0() as u32); h.m(f.x2227_b6() as u32); h.m(f.x2222_b3() as u32);
            h.m(b(f.self_vel.x)); h.m(b(f.self_vel.y)); h.m(b(f.self_vel.z));
            h.m(b(f.x8c_kb_vel.x)); h.m(b(f.x8c_kb_vel.y)); h.m(b(f.x8c_kb_vel.z));
            h.m(b(f.gr_vel)); h.m(b(f.xF0_ground_kb_vel)); h.m(b(f.grab_timer));
            h.m(b(f.cur_pos.x)); h.m(b(f.cur_pos.y)); h.m(b(f.cur_pos.z));
            unsafe {
                let di = &f.mv.co.damageice;
                h.m(di.wall_hit_dir as u32); h.m(b(di.rot_speed));
                h.m(b(di.ice_coll.top)); h.m(b(di.ice_coll.bottom));
                h.m(b(di.ice_coll.left.x)); h.m(b(di.ice_coll.left.y));
                h.m(b(di.ice_coll.right.x)); h.m(b(di.ice_coll.right.y));
            }
            h.m(cbid(f.take_dmg_cb)); h.m(cbid(f.take_dmg_2_cb));
            for i in 0..2 {
                let j = unsafe { &*jo(i) };
                h.m(b(j.translate.y)); h.m(b(j.translate.z)); h.m(b(j.rotate.x));
            }
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 60000);
}
