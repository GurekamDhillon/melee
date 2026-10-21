//! src/tu/ftCo_ThrownKirby.rs vs the C (gen/cref/tu/ftCo_ThrownKirby.c: the whole TU, TARGET_PC,
//! shipping contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_ThrownKirby`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_ThrownKirby as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

static mut JO: *mut HSD_JObj = core::ptr::null_mut();
static mut JO2: *mut HSD_JObj = core::ptr::null_mut();
static mut JB: *mut [HSD_JObj; 4] = core::ptr::null_mut();
static mut BONES: *mut [FighterBone; 4] = core::ptr::null_mut();
static mut GENO: i32 = 0;

fn pi<T>(p: *const T) -> u32 {
    let p = p as usize;
    unsafe {
        let fr = f();
        if p == 0 { return 0; }
        if p == go() as usize { return 1; }
        if p == go2() as usize { return 2; }
        if p == F as usize { return 3; }
        if p == F2 as usize { return 4; }
        if p == JO as usize { return 5; }
        if p == JO2 as usize { return 6; }
        for i in 0..4 { if p == &(*JB)[i] as *const HSD_JObj as usize { return 7 + i as u32; } }
        if p == fr.x914.as_ptr() as usize { return 11; }
        if p == &fr.mv.co.thrownkirby.coll_box as *const _ as usize { return 12; }
        if p == &fr.self_vel as *const _ as usize { return 13; }
    }
    99
}

#[test]
fn ftCo_ThrownKirby_matches_c() {
    init();
    unsafe {
        JO = Box::leak(Box::new(HSD_JObj::zeroed()));
        JO2 = Box::leak(Box::new(HSD_JObj::zeroed()));
        JB = Box::leak(Box::new([HSD_JObj::zeroed(), HSD_JObj::zeroed(), HSD_JObj::zeroed(), HSD_JObj::zeroed()]));
        BONES = Box::leak(Box::new(core::mem::zeroed::<[FighterBone; 4]>()));
        hk::Geno_DefineKirbyCopy = Some(|k| { t(1); t(k as u32); GENO });
        hk::ftKb_MexKirbyLike = Some(|k| { t(2); t(k as u32); sr(2) as i32 });
        hk::ft_800824A0 = Some(|g, bx| { t(3); t(pi(g)); t(pi(bx)); sr(3) });
        hk::Fighter_UpdateModelScale = Some(|g| { t(4); t(pi(g)) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(5); t(pi(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(pi(o))
        });
        hk::ftCommon_8007E2F4 = Some(|fr, v| { t(6); t(pi(fr as *const Fighter)); t(v as i32 as u32) });
        hk::ftColl_8007B62C = Some(|g, v| { t(7); t(pi(g)); t(v as u32) });
        hk::ftCommon_8007EFC0 = Some(|fr, v| { t(8); t(pi(fr as *const Fighter)); t(v) });
        hk::ftCommon_InitGrab = Some(|fr, a, v| { t(9); t(pi(fr as *const Fighter)); t(a as u32); t(b(v)) });
        hk::ftAnim_8006EBA4 = Some(|g| { t(10); t(pi(g)) });
        hk::ftCommon_8007D5D4 = Some(|fr| { t(11); t(pi(fr as *const Fighter)) });
        hk::ftCommon_SetAccessory = Some(|fr, j| { t(12); t(pi(fr as *const Fighter)); t((j as usize == igo() as usize) as u32) });
        hk::ftKb_SpecialN_800F5898 = Some(|g| { t(13); t(pi(g)); igo() as *mut HSD_Joint });
        hk::HSD_JObjSetMtxDirtySub = Some(|j| { t(14); t(pi(j)) });
        hk::lb_8000C2F8 = Some(|a, bb| { t(15); t(pi(a)); t(pi(bb)) });
        hk::ftParts_GetBoneIndex = Some(|fr, p| { t(16); t(pi(fr as *const Fighter)); t(p as u32); (rnd() % 4) as i32 });
        hk::ftColl_8007ABD0 = Some(|h, d, g| { t(17); t(pi(h)); t(d); t(pi(g)) });
        hk::ftColl_8007AFF8 = Some(|g| { t(18); t(pi(g)) });
        hk::ftKb_SpecialN_800F5820 = Some(|g, bx, s| { t(19); t(pi(g)); t(pi(bx)); t(b(s)) });
        hk::ftKb_SpecialN_800F58AC = Some(|g, v, fa| { t(20); t(pi(g)); t(pi(v)); t(b(fa)); rw() });
        hk::ftKb_SpecialN_800F58D8 = Some(|g, v, fa| { t(21); t(pi(g)); t(pi(v)); t(b(fa)); rw() });
        hk::ftKb_SpecialN_800F5A88 = Some(|g| { t(22); t(pi(g)); rw() });
        hk::ftKb_SpecialN_800F5AB0 = Some(|g| { t(23); t(pi(g)); rw() });
        hk::ftKb_SpecialN_800F5AC0 = Some(|| { t(24); rg() });
        hk::ftKb_SpecialN_800F5AD8 = Some(|| { t(25); rg() });
        hk::ftKb_SpecialN_800F5874 = Some(|v| { t(26); (*v).x = rw(); (*v).y = rw(); });
        hk::ftKb_SpecialN_800F190C = Some(|g, k| { t(27); t(pi(g)); t(k as u32) });
        hk::ftKb_SpecialN_800EEEC4 = Some(|g, k| { t(28); t(pi(g)); t(k as u32) });
        hk::ftKb_SpecialN_800F5A98 = Some(|| { t(29); rw() });
        hk::ftKb_SpecialN_800F5A70 = Some(|| { t(30); rg() });
        hk::ftCommon_GrabMash = Some(|fr, v| { t(31); t(pi(fr as *const Fighter)); t(b(v)); sr(31) });
        hk::lbVector_Mirror = Some(|a, bb| { t(32); t(pi(a)); t(b((*bb).x)); t(b((*bb).y)); t(b((*bb).z)); (*a).x = rw(); (*a).y = rw(); });
        hk::ftCo_Fall_Enter = Some(|g| { t(33); t(pi(g)) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_ThrownKirby.txt"));
    for op in 0..13u32 {
        for k in 0..2000u32 {
            case(op, k);
            let (jo, jo2) = unsafe {
                *JO = HSD_JObj::zeroed();
                *JO2 = HSD_JObj::zeroed();
                for i in 0..4 { (*BONES)[i].joint = &mut (*JB)[i]; }
                (*go()).hsd_obj = JO as *mut _;
                (&mut *JO, &mut *JO2)
            };
            let fr = f();
            fr.parts = unsafe { (*BONES).as_mut_ptr() };
            fr.x20A0_accessory = jo2;
            jo.flags = rnd() & 0x0200_0000;
            jo2.flags = rnd() & 0x0200_0000;
            if rnd() & 1 != 0 { jo.flags |= 0x40; }
            if rnd() & 1 != 0 { jo2.flags |= 0x40; }
            jo.scale.x = rw(); jo.scale.y = rw(); jo.scale.z = rw();
            unsafe { GENO = if rnd() & 3 != 0 { -2 } else { (rnd() % 40) as i32 - 1 } };
            fr.kind = if rnd() & 1 != 0 { Ft_Kind_Kirby } else { (rnd() % 60) as _ };
            f2().kind = (rnd() % 60) as _;
            unsafe {
                fr.u.kb.hat.kind = (rnd() % 34) as _;
                f2().u.kb.hat.kind = (rnd() % 34) as _;
                fr.u.kb.hat.set_x8_b0((rnd() & 1) as u8);
            }
            fr.facing_dir = rg(); f2().facing_dir = rg();
            fr.self_vel.x = rw(); fr.self_vel.y = rw(); fr.self_vel.z = rw();
            fr.grab_timer = rw();
            fr.co_attrs.xDC = rw();
            fr.co_attrs.kirby_b_star_damage = if rnd() & 1 != 0 { (rnd() % 30) as f32 } else { rw() };
            unsafe {
                let tk = &mut fr.mv.co.thrownkirby;
                tk.x4 = rw(); tk.xC = rg();
                tk.x10 = if rnd() & 7 != 0 { rw() } else { 0.0 };
                tk.anon_18.anon_0.set_x18_b0((rnd() & 1) as u8);
                tk.anon_18.anon_0.set_x18_b1((rnd() & 1) as u8);
                tk.scale.x = rw(); tk.scale.y = rw(); tk.scale.z = rw();
                fr.mv.ca.specialhi.vel.x = rw();
                fr.mv.ca.specialhi.vel.y = if rnd() & 1 != 0 { rw() } else { 0.0 };
            }
            fr.coll_data.env_flags = (rnd() & (0x3F | 0xFC0 | 0x6000)) as i32;
            fr.coll_data.floor.normal.x = rg(); fr.coll_data.floor.normal.y = rg(); fr.coll_data.floor.normal.z = rg();
            fr.coll_data.right_facing_wall.normal.x = rg(); fr.coll_data.right_facing_wall.normal.y = rg();
            fr.coll_data.left_facing_wall.normal.x = rg(); fr.coll_data.left_facing_wall.normal.z = rg();
            fr.coll_data.ceiling.normal.y = rg(); fr.coll_data.ceiling.normal.z = rg();
            let g = go();
            let ret: u32 = match op {
                0 => m::ftCo_800BD9E0(g, go2()) as u32,
                1 => { m::ftCo_800BDA50(g); 0 }
                2 => {
                    let mut n = Vec3 { x: 7.0, y: 7.0, z: 7.0 };
                    m::ftCo_800BDA74(g, &mut n);
                    b(n.x) ^ b(n.y).wrapping_mul(3) ^ b(n.z).wrapping_mul(7)
                }
                3 => { m::ftCo_800BDB58(g, go2()); 0 }
                4 => { m::ftCo_ThrownKirbyStar_Phys(g); 0 }
                5 => { m::ftCo_ThrownKirbyStar_Coll(g); 0 }
                6 => { m::ftCo_800BE000(g, go2()); 0 }
                7 => { m::ftCo_ThrownCopyStar_Phys(g); 0 }
                8 => { m::ftCo_ThrownCopyStar_Coll(g); 0 }
                9 => { m::ftCo_800BE494(g); 0 }
                10 => { m::ftCo_ThrownKirby_Phys(g); 0 }
                11 => { m::ftCo_800BE6AC(g); 0 }
                _ => {
                    m::ftCo_800BE7C0(g);
                    m::ftCo_ThrownKirbyStar_Anim(g); m::ftCo_ThrownKirbyStar_IASA(g); m::ftCo_ThrownCopyStar_Anim(g);
                    m::ftCo_ThrownCopyStar_IASA(g); m::ftCo_ThrownKirby_Anim(g); m::ftCo_ThrownKirby_IASA(g);
                    m::ftCo_ThrownKirby_Coll(g);
                    0
                }
            };
            let fr = f();
            let mut h = Hs::new();
            h.m(b(fr.facing_dir)); h.m(b(fr.self_vel.x)); h.m(b(fr.self_vel.y)); h.m(b(fr.self_vel.z)); h.m(b(fr.grab_timer));
            unsafe {
                let tk = &fr.mv.co.thrownkirby;
                h.m(pi(tk.thrower_gobj)); h.m(b(tk.x4)); h.m(b(tk.x8));
                h.m(b(tk.xC)); h.m(b(tk.x10)); h.m(tk.x14 as u32);
                h.m(tk.anon_18.anon_0.x18_b0() as u32); h.m(tk.anon_18.anon_0.x18_b1() as u32);
                h.m(b(tk.scale.x)); h.m(b(tk.scale.y)); h.m(b(tk.scale.z));
            }
            h.m(fr.take_dmg_cb.is_some() as u32); h.m(fr.take_dmg_2_cb.is_some() as u32);
            h.m(fr.accessory4_cb.is_some() as u32); h.m(fr.invisible() as u32);
            unsafe {
                h.m(fr.u.kb.hat.kind as u32); h.m(f2().u.kb.hat.kind as u32);
                h.m(f2().mv.co.thrownkirby.anon_18.anon_0.x18_b1() as u32);
                h.m(b((*JO).scale.x)); h.m(b((*JO).scale.y)); h.m(b((*JO).scale.z));
                h.m(b((*JO2).scale.x)); h.m(b((*JO2).scale.y)); h.m(b((*JO2).scale.z));
            }
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 26000);
}
