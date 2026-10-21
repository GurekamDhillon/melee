//! Rust half of gen/cref/tu/s217_fill.h (shared by the SpecialS/Squat*/Stop*/Throw/Thrown TU
//! tests): same fill order, same hash.
#![allow(dead_code, static_mut_refs)]
use crate::tu_ftco_harness::*;
use melee_fighter::types::*;

pub static mut FD: *mut ftData = core::ptr::null_mut();
pub static mut FD2: *mut ftData = core::ptr::null_mut();
pub static mut DA: *mut ftCo_DatAttrs = core::ptr::null_mut();
pub static mut DA2: *mut ftCo_DatAttrs = core::ptr::null_mut();
pub static mut J: *mut [HSD_JObj; 8] = core::ptr::null_mut();
pub static mut PARTS: *mut [FighterBone; 8] = core::ptr::null_mut();
pub static mut JO: *mut HSD_JObj = core::ptr::null_mut();
pub static mut JO2: *mut HSD_JObj = core::ptr::null_mut();
pub const WS: *mut WaitStruct = 0x1234 as *mut WaitStruct;
const KINDS: [u32; 16] = [4, 13, 14, 5, 31, 9, 19, 6, 20, 1, 0, 33, 34, 40, 2, 18];

pub fn jo() -> &'static mut HSD_JObj { unsafe { &mut *JO } }
pub fn jo2() -> &'static mut HSD_JObj { unsafe { &mut *JO2 } }

pub fn pi<T>(p: *const T) -> u32 {
    let p = p as usize;
    unsafe {
        let fr = F;
        let fr2 = F2;
        if p == 0 { return 0; }
        if p == go() as usize { return 1; }
        if p == go2() as usize { return 2; }
        if p == fr as usize { return 3; }
        if p == fr2 as usize { return 4; }
        if p == JO as usize { return 5; }
        if p == JO2 as usize { return 6; }
        if p == &(*fr).coll_data as *const _ as usize { return 7; }
        if p == &(*fr2).coll_data as *const _ as usize { return 8; }
        for i in 0..8 {
            if p == &(*J)[i] as *const _ as usize { return 10 + i as u32; }
        }
        if p == &(*fr).xDF4[0] as *const _ as usize { return 20; }
        if p == &(*fr).xDF4[1] as *const _ as usize { return 21; }
        if p == &(*fr2).xDF4[0] as *const _ as usize { return 22; }
        if p == &(*fr2).xDF4[1] as *const _ as usize { return 23; }
        if p == &(*fr2).dmg.facing_dir_1 as *const _ as usize { return 24; }
        if p == &(*fr).dmg.facing_dir_1 as *const _ as usize { return 25; }
        if p == &(*CD).x380 as *const _ as usize { return 26; }
        if p == WS as usize { return 27; }
    }
    99
}
pub fn chk(id: u32) -> bool { unsafe { (SRV[(10 + id % 6) as usize] >> (id % 7)) & 1 != 0 } }

pub fn s_init() {
    init();
    unsafe {
        FD = Box::leak(Box::new(ftData::zeroed()));
        FD2 = Box::leak(Box::new(ftData::zeroed()));
        DA = Box::leak(Box::new(ftCo_DatAttrs::zeroed()));
        DA2 = Box::leak(Box::new(ftCo_DatAttrs::zeroed()));
        J = Box::leak(Box::new([HSD_JObj::zeroed(); 8]));
        PARTS = Box::leak(Box::new([FighterBone::zeroed(); 8]));
        JO = Box::leak(Box::new(HSD_JObj::zeroed()));
        JO2 = Box::leak(Box::new(HSD_JObj::zeroed()));
        for i in 0..8 {
            (*PARTS)[i].joint = &mut (*J)[i];
        }
    }
}

fn fill_one(f: &mut Fighter, me: *mut HSD_GObj, other: *mut HSD_GObj) {
    f.kind = KINDS[(rnd() % 16) as usize];
    f.x688 = (rnd() & 1) as u8;
    if rnd() % 3 == 0 {
        f.facing_dir = rg();
    } else if rnd() & 1 != 0 {
        f.facing_dir = 1.0;
    } else {
        f.facing_dir = -1.0;
    }
    f.gr_vel = rw();
    f.co_attrs.specials_ground_speed_retention = rg();
    f.co_attrs.walk_max_vel = rw();
    f.co_attrs.weight = rw();
    f.input.pressed_buttons = (rnd() & 0x3FF) as _;
    f.input.lstick[0].x = rg();
    f.input.lstick[0].y = rg();
    f.input.lstick[1].x = rg();
    f.input.lstick[1].y = rg();
    f.input.cstick[0].x = rg();
    f.input.cstick[0].y = rg();
    f.active_timer.lstick.x = (rnd() % 3) as u8;
    f.coll_data.env_flags = rnd() as _;
    f.coll_data.ecb.top.y = rw();
    f.coll_data.ecb.bottom.y = rw();
    f.coll_data.ecb.left.x = rw();
    f.coll_data.ecb.left.y = rw();
    f.coll_data.ecb.right.x = rw();
    f.coll_data.ecb.right.y = rw();
    f.cur_pos.x = rw();
    f.cur_pos.y = rw();
    f.cur_pos.z = rw();
    f.x68C_transNPos.x = rw();
    f.x68C_transNPos.y = rw();
    f.x68C_transNPos.z = rw();
    f.x6A4_transNOffset.x = rw();
    f.x6A4_transNOffset.y = rw();
    f.x6A4_transNOffset.z = rw();
    f.self_vel.x = rw();
    f.self_vel.y = rw();
    f.self_vel.z = rw();
    f.motion_id = if rnd() & 1 != 0 { 219 + (rnd() % 4) as i32 } else { (rnd() % 400) as i32 };
    f.cmd_vars[0] = rnd() & 1;
    f.cur_anim_frame = rg();
    f.grab_timer = if rnd() & 1 != 0 { 0.0 } else { rw() };
    f.player_id = (rnd() % 4) as u8;
    f.set_is_sub_fighter((rnd() & 1) as u8);
    unsafe {
        f.anon_2210.throw_flags = (rnd() & 0xFF) << 8;
        let t = &mut f.anon_2210.anon_0;
        t.set_throw_flags_b0((rnd() & 1) as u8);
        t.set_throw_flags_b1((rnd() & 1) as u8);
        t.set_throw_flags_b2((rnd() & 1) as u8);
        t.set_throw_flags_b3((rnd() & 1) as u8);
        t.set_throw_flags_b4((rnd() & 1) as u8);
        t.set_throw_flags_b5((rnd() & 1) as u8);
        t.set_throw_flags_b6((rnd() & 1) as u8);
        t.set_throw_flags_b7((rnd() & 1) as u8);
    }
    f.anon_221b.set_x221B_b5((rnd() & 1) as u8);
    f.anon_221b.set_x221B_b7((rnd() & 1) as u8);
    f.set_x2219_b2((rnd() & 1) as u8);
    f.set_x2222_b0((rnd() & 1) as u8);
    f.set_x2226_b2((rnd() & 1) as u8);
    f.victim_gobj = if rnd() % 4 != 0 { other } else { core::ptr::null_mut() };
    f.x1A5C = if rnd() & 1 != 0 { other } else { core::ptr::null_mut() };
    f.ground_or_air = rnd() & 1;
    for i in 0..2 {
        f.xDF4[i].damage = rw();
        f.xDF4[i].unk_count = rnd() % 5;
        f.xDF4[i].kb_angle = (rnd() % 361) as i32;
        f.xDF4[i].element = rnd() % 5;
    }
    f.x1988 = (rnd() % 3) as _;
    f.x1A70.x = rw();
    f.x1A70.y = rw();
    f.x1A70.z = rw();
    f.x34_scale.y = rg();
    f.x2174.x = rw();
    f.x2174.y = rw();
    f.x2174.z = rw();
    f.dmg.x1848_kb_angle = (rnd() % 361) as i32;
    f.dmg.facing_dir_1 = rg();
    let w = &mut f.mv as *mut _ as *mut u32;
    for i in 0..8 {
        let v = if rnd() & 1 != 0 { b(rg()) } else { rnd() % 3 };
        unsafe { *w.add(i) = v };
    }
    f.gobj = me;
    f.parts = unsafe { PARTS as *mut FighterBone };
}

pub fn s_fill() {
    unsafe {
        *FD = ftData::zeroed();
        *FD2 = ftData::zeroed();
        *DA = ftCo_DatAttrs::zeroed();
        *DA2 = ftCo_DatAttrs::zeroed();
        *J = [HSD_JObj::zeroed(); 8];
        *JO = HSD_JObj::zeroed();
        *JO2 = HSD_JObj::zeroed();
        (*go()).hsd_obj = JO as *mut core::ffi::c_void;
        (*go2()).hsd_obj = JO2 as *mut core::ffi::c_void;
        (*JO).flags = rnd();
        (*JO2).flags = rnd();
        for i in 0..8 {
            (*J)[i].flags = rnd();
        }
        (*FD).x0 = DA;
        (*FD2).x0 = DA2;
        (*FD).x28 = WS;
        f().ft_data = FD;
        f2().ft_data = FD2;
        (*DA).weight = rw();
        (*DA2).weight = rw();
        (*DA).weight_independent_throws_mask = (rnd() & 15) as u8;
    }
    fill_one(f(), go(), go2());
    fill_one(f2(), go2(), go());
    let c = cd();
    c.x218 = rg();
    c.x220 = rg();
    c.x90 = rg();
    c.x94 = rg();
    c.x5F0 = rnd();
    c.x98 = rg();
    c.attackhi3_stick_threshold_y = rg();
    c.xB0 = rg();
    c.x348 = (rnd() % 9) as i32;
    c.x37C = rw();
    c.x10C = rw();
    c.grab_timer_decrement = rw();
    c.x3C8 = rw();
}

fn hash_one(h: &mut Hs, f: &Fighter) {
    h.m(b(f.facing_dir));
    h.m(b(f.gr_vel));
    h.m(b(f.self_vel.x));
    h.m(b(f.self_vel.y));
    h.m(b(f.self_vel.z));
    h.m(b(f.cur_pos.x));
    h.m(b(f.cur_pos.y));
    h.m(b(f.cur_pos.z));
    unsafe {
        h.m(f.anon_2210.throw_flags >> 8);
        let t = &f.anon_2210.anon_0;
        h.m(t.throw_flags_b0() as u32);
        h.m(t.throw_flags_b1() as u32);
        h.m(t.throw_flags_b2() as u32);
        h.m(t.throw_flags_b3() as u32);
        h.m(t.throw_flags_b4() as u32);
        h.m(t.throw_flags_b5() as u32);
        h.m(t.throw_flags_b6() as u32);
        h.m(t.throw_flags_b7() as u32);
    }
    h.m(f.anon_221b.x221B_b5() as u32);
    h.m(f.anon_221b.x221B_b7() as u32);
    h.m(f.x2219_b2() as u32);
    h.m(f.x2226_b2() as u32);
    h.m(f.cmd_vars[0]);
    h.m(b(f.grab_timer));
    h.m(pi(f.victim_gobj));
    h.m(pi(f.x1A5C));
    h.m(f.accessory1_cb.is_some() as u32);
    h.m(f.x21EC.is_some() as u32);
    let w = &f.mv as *const _ as *const u32;
    unsafe {
        for i in 0..4 {
            h.m(*w.add(i));
        }
        h.m(*w.add(6));
        h.m(*w.add(7));
        h.m(pi(f.mv.co.fighterthrow.victim));
    }
    h.m(b(f.dmg.kb_applied));
    h.m(f.dmg.x1848_kb_angle as u32);
    h.m(b(f.dmg.facing_dir_1));
    h.m(f.dmg.x184c_damaged_hurtbox as u32);
    h.m(b(f.dmg.x1854_collpos.x));
    h.m(b(f.dmg.x1854_collpos.y));
    h.m(b(f.dmg.x1854_collpos.z));
    h.m(f.dmg.x1860_element);
    h.m(b(f.coll_data.last_pos.x));
    h.m(b(f.coll_data.last_pos.y));
    h.m(b(f.coll_data.last_pos.z));
    h.m(b(f.coll_data.cur_pos.x));
    h.m(b(f.coll_data.cur_pos.y));
    h.m(b(f.coll_data.cur_pos.z));
}

pub fn state_hash() -> u32 {
    let mut h = Hs::new();
    hash_one(&mut h, f());
    hash_one(&mut h, f2());
    for j in [jo(), jo2()] {
        h.m(b(j.translate.x));
        h.m(b(j.translate.y));
        h.m(b(j.translate.z));
    }
    for i in 0..8 {
        let j = unsafe { &(*J)[i] };
        h.m(b(j.translate.x));
        h.m(b(j.translate.y));
        h.m(b(j.translate.z));
    }
    h.0
}

/// `T(id); T(PI(g))` recording stubs, as hook values.
#[macro_export]
macro_rules! sv {
    ($hk:ident; $($n:ident = $id:expr),* $(,)?) => { unsafe { $( $hk::$n = Some(|g| { t($id); t(pi(g)) }); )* } };
}
#[macro_export]
macro_rules! sc {
    ($hk:ident; $($n:ident = $id:expr),* $(,)?) => { unsafe { $( $hk::$n = Some(|g| { t($id); t(pi(g)); chk($id) }); )* } };
}
pub fn cms(g: *mut HSD_GObj, m: i32, fl: u32, a: f32, bb: f32, c: f32, o: *mut HSD_GObj) {
    t(9); t(pi(g)); t(m as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(pi(o));
}
