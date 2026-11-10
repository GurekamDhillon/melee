//! Rust half of gen/cref/tu/ftco_attack_fill.h: same fill (field for field, draw for draw) and
//! the same state hash, for the tu_ftCo_AppealS/Attack1/Attack100/AttackAir/AttackDash/
//! AttackHi3/AttackHi4/AttackLw3 tests. Use with `tu_ftco_harness`.
#![allow(dead_code, static_mut_refs)]
use super::tu_ftco_harness::*;
use melee_fighter::types::*;

pub static mut MS1: *mut MotionState = core::ptr::null_mut();
pub static mut MS2: *mut MotionState = core::ptr::null_mut();
pub static mut FD: *mut ftData = core::ptr::null_mut();
pub static mut FDX8: *mut ftData_x8 = core::ptr::null_mut();
pub static mut PARTS: *mut FighterBone = core::ptr::null_mut();
pub static mut JOBJ: *mut [u8; 32] = core::ptr::null_mut();
pub static mut IGO2: *mut HSD_GObj = core::ptr::null_mut();
const KINDS: [u32; 16] = [0, 4, 6, 8, 9, 12, 18, 19, 20, 21, 23, 24, 25, 33, 34, 2];

pub fn igo2() -> *mut HSD_GObj { unsafe { IGO2 } }

pub fn af_init() {
    unsafe {
        let ms1: &mut [MotionState] = Box::leak((0..400).map(|_| MotionState::zeroed()).collect::<std::vec::Vec<_>>().into_boxed_slice());
        let ms2: &mut [MotionState] = Box::leak((0..400).map(|_| MotionState::zeroed()).collect::<std::vec::Vec<_>>().into_boxed_slice());
        for i in 0..400 {
            ms1[i].anim_id = i as i32;
            ms2[i].anim_id = 1000 + i as i32;
        }
        MS1 = ms1.as_mut_ptr();
        MS2 = ms2.as_mut_ptr();
        JOBJ = Box::leak(Box::new([0u8; 32]));
        let parts: &mut [FighterBone] = Box::leak((0..8).map(|_| FighterBone::zeroed()).collect::<std::vec::Vec<_>>().into_boxed_slice());
        for i in 0..8 {
            parts[i].joint = (JOBJ as *mut u8).add(i * 4) as *mut HSD_JObj;
        }
        PARTS = parts.as_mut_ptr();
        FD = Box::leak(Box::new(ftData::zeroed()));
        FDX8 = Box::leak(Box::new(ftData_x8::zeroed()));
        (*FD).x8 = FDX8;
        IGO2 = Box::leak(Box::new(HSD_GObj::zeroed()));
    }
}
pub fn af_jobj(j: *mut HSD_JObj) -> u32 {
    for i in 0..8 {
        if j == unsafe { (JOBJ as *mut u8).add(i * 4) } as *mut HSD_JObj {
            return i as u32 + 1;
        }
    }
    if j.is_null() { 0 } else { 99 }
}
pub fn af_item<T>(p: *mut T) -> u32 {
    let p = p as *mut HSD_GObj;
    if p.is_null() { 0 } else if p == igo() { 1 } else if p == igo2() { 2 } else { 99 }
}

pub fn af_fill() {
    let c = cd();
    c.attackhi3_stick_threshold_y = rg(); c.x20_radians = rg(); c.xCC = rg(); c.xD0 = rw();
    c.x21C = rg(); c.x1C = (rnd() % 8) as i32; c.xB0 = rg(); c.xDC = rg(); c.xE0 = rg();
    c.x50 = rw(); c.x68 = rw(); c.x414 = (rnd() % 5) as i32 - 1;
    let fr = f();
    fr.input.pressed_buttons = rnd() & 0x30F;
    fr.input.released_buttons = rnd() & 0x100;
    fr.input.held_buttons[0] = if rnd() & 1 != 0 { HSD_PAD_LR } else { 0 };
    fr.input.lstick[0].x = rg(); fr.input.lstick[0].y = rg();
    fr.input.cstick[0].x = rg(); fr.input.cstick[0].y = rg();
    fr.active_timer.lstick.y = (rnd() % 8) as u8;
    fr.item_gobj = if rnd() & 1 != 0 { igo() as *mut Item_GObj } else { core::ptr::null_mut() };
    fr.kind = KINDS[(rnd() % 16) as usize];
    fr.set_allow_interrupt((rnd() & 1) as u8); fr.set_x2218_b1((rnd() & 1) as u8);
    fr.set_x2218_b2((rnd() & 1) as u8); fr.set_x2224_b1((rnd() & 1) as u8);
    let a = rg();
    fr.hitlag_mul = a + (rnd() % 3) as f32;
    fr.unk_msid = 43 + (rnd() % 4) as i32;
    fr.cmd_vars[0] = rnd() & 1;
    fr.mv.co.attack100.x0 = (rnd() & 1) as i32; fr.mv.co.attack100.x4 = (rnd() & 1) as i32;
    fr.x2210.throw_flags = (rnd() & 0xFF) << 8;
    unsafe {
        fr.x2210.x0.set_throw_flags_b3((rnd() & 1) as u8);
        fr.x2210.x0.set_throw_flags_b0((rnd() & 1) as u8);
    }
    fr.x1A54 = (rnd() % 6) as i32; fr.co_attrs.rapid_jab_window = (rnd() % 6) as i32;
    fr.co_attrs.jab_2_input_window = rw(); fr.co_attrs.jab_3_input_window = rw();
    fr.x209C = (rnd() % 3) as u16; fr.x683 = (rnd() % 8) as u8; fr.x686 = (rnd() & 1) as u8;
    fr.x687 = (rnd() & 1) as u8; fr.x689 = (rnd() & 1) as u8;
    fr.x68B = (rnd() % 8) as u8;
    fr.cur_anim_frame = rg(); fr.frame_speed_mul = rg();
    fr.facing_dir = if rnd() % 3 == 0 { rg() } else if rnd() & 1 != 0 { 1.0 } else { -1.0 };
    fr.co_attrs.ground_friction = rw();
    fr.player_idx = (rnd() % 4) as u8; fr.set_is_sub_fighter((rnd() & 1) as u8);
    fr.x18 = if rnd() & 1 != 0 { 200 } else { 300 };
    unsafe {
        fr.x1C_actionStateList = MS1; fr.x20_actionStateList = MS2;
        fr.ft_data = FD; (*FDX8).x10 = (rnd() % 4) as u8;
        fr.parts = PARTS;
    }
    fr.gobj = go();
}

pub fn af_hash() -> u32 {
    let fr = f();
    let mut h = Hs::new();
    h.m(fr.allow_interrupt() as u32); h.m(fr.x2218_b1() as u32); h.m(fr.x2218_b2() as u32); h.m(fr.x2224_b1() as u32);
    h.m(b(fr.hitlag_mul)); h.m(fr.unk_msid as u32); h.m(fr.cmd_vars[0]);
    unsafe {
        h.m(fr.mv.co.attack100.x0 as u32); h.m(fr.mv.co.attack100.x4 as u32);
        h.m(fr.x2210.throw_flags >> 8);
        let t = &fr.x2210.x0;
        h.m(t.throw_flags_b0() as u32); h.m(t.throw_flags_b1() as u32); h.m(t.throw_flags_b2() as u32); h.m(t.throw_flags_b3() as u32);
        h.m(t.throw_flags_b4() as u32); h.m(t.throw_flags_b5() as u32); h.m(t.throw_flags_b6() as u32); h.m(t.throw_flags_b7() as u32);
    }
    h.m(fr.x1A54 as u32); h.m(fr.x209C as u32); h.m(b(fr.facing_dir));
    h.m(af_item(fr.item_gobj)); h.m(af_item(fr.x1978));
    h.0
}

/// `chk(id)` of ftco_attack_fill.h.
pub fn chk(id: u32) -> bool { unsafe { (SRV[(10 + id % 6) as usize] >> (id % 7)) & 1 != 0 } }
pub fn angle_of(v: u32) -> f32 { ((v % 41) as i32 - 20) as f32 / 20.0 }
pub fn srv(i: usize) -> u32 { unsafe { SRV[i] } }
/// `STUB_V(name, id)`.
macro_rules! vrec { ($id:expr) => { Some(|g| { t($id); t(is_go(g)); }) }; }
/// `STUB_CHK(name, id)`.
macro_rules! chkrec { ($id:expr) => { Some(|g| { t($id); t(is_go(g)); chk($id) }) }; }
