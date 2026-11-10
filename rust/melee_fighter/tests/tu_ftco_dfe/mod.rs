//! Rust half of gen/cref/tu/ftco_dfe_fill.h: same fill (field for field, draw for draw) and the
//! same state hash, for the tu_ftCo_DownDamage/DownSpot/DownStand/Escape/EscapeAir/Fall/
//! FallAerial/FallSpecial tests. Use with `tu_ftco_harness`.
#![allow(dead_code, static_mut_refs)]
use super::tu_ftco_harness::*;
use melee_fighter::types::*;

pub static mut DAT: *mut [u8; 256] = core::ptr::null_mut();
pub static mut SKEL: *mut [u8; 4] = core::ptr::null_mut();
const MS: [i32; 7] = [
    ftCo_MS_DownBoundU, ftCo_MS_DownWaitU, ftCo_MS_DownDamageU,
    ftCo_MS_DownBoundD, ftCo_MS_DownWaitD, ftCo_MS_DownDamageD, 0,
];

pub fn df_init() {
    unsafe {
        DAT = Box::leak(Box::new([0u8; 256]));
        SKEL = Box::leak(Box::new([0u8; 4]));
    }
}
pub fn dat() -> *mut core::ffi::c_void { unsafe { DAT as *mut core::ffi::c_void } }
pub fn skel() -> *mut HSD_JObj { unsafe { SKEL as *mut HSD_JObj } }
fn mvw(i: usize) -> *mut u32 { unsafe { (core::ptr::addr_of_mut!(f().mv) as *mut u32).add(i) } }

pub fn df_fill() {
    let c = cd();
    c.x428 = (rnd() % 300) as i32; c.x244 = rg(); c.x20_radians = rg(); c.x31C = rg();
    c.x320 = (rnd() % 8) as i32; c.x324 = (rnd() % 3) as i32; c.x314 = rg(); c.x318 = (rnd() % 8) as i32;
    c.walk_fast_stick_threshold = rg(); c.x334 = (rnd() % 40) as i32;
    c.escapeair_deadzone.x = rg(); c.escapeair_deadzone.y = rg(); c.escapeair_force = rw();
    c.x340 = rg(); c.x344 = rw(); c.escapeair_decay = rg(); c.x444 = rg(); c.x448 = rg();
    c.x25C = rg();
    for i in 0..7 {
        let v = if rnd() & 1 != 0 { b(rw()) } else { rnd() % 64 };
        unsafe { *mvw(i) = v };
    }
    let fr = f();
    fr.motion_id = MS[(rnd() % 7) as usize] as _;
    fr.set_stamina_dead((rnd() & 1) as _);
    fr.dmg.x1838_percentTemp = rw();
    fr.facing_dir = if rnd() % 3 == 0 { rg() } else if rnd() & 1 != 0 { 1.0 } else { -1.0 };
    fr.ground_or_air = rnd() & 1;
    fr.coll_data.env_flags = rnd() as i32;
    fr.coll_data.floor.index = if rnd() & 1 != 0 { -1 } else { (rnd() % 5) as i32 };
    fr.coll_data.floor.flags = rnd() & 0x100;
    fr.input.lstick[0].x = rg(); fr.input.lstick[0].y = rg();
    fr.input.cstick[0].x = rg(); fr.input.cstick[0].y = rg();
    let p = rnd() & 0x6F;
    fr.input.pressed_buttons = p | if rnd() & 1 != 0 { HSD_PAD_LR } else { 0 };
    fr.input.held_buttons[0] = if rnd() & 1 != 0 { HSD_PAD_LR } else { 0 };
    fr.active_timer.lstick.x = (rnd() % 8) as u8; fr.active_timer.lstick.y = (rnd() % 8) as u8;
    fr.kind = match rnd() % 3 {
        0 => Ft_Kind_Samus,
        1 => Ft_Kind_Yoshi,
        _ => rnd() % 27,
    };
    fr.x2210.throw_flags = (rnd() & 0xFF) << 8;
    unsafe { fr.x2210.x0.set_throw_flags_b3((rnd() & 1) as u8) };
    fr.cmd_vars[0] = rnd() & 1;
    fr.x5F4_arr[0].idx = (rnd() % 3) as i8;
    fr.dat_attrs = dat();
    fr.self_vel.x = rw(); fr.self_vel.y = rw(); fr.self_vel.z = rw();
    fr.co_attrs.air_drift_max = rw(); fr.co_attrs.gravity = rg(); fr.co_attrs.terminal_velocity = rg();
    fr.co_attrs.fast_fall_velocity = rg(); fr.co_attrs.air_drift_stick_mul = rg();
    fr.co_attrs.aerial_drift_base = rg(); fr.co_attrs.aerial_friction = rg();
    fr.set_fall_fast((rnd() & 1) as _); fr.set_is_sandbag((rnd() & 1) as _);
    fr.cur_anim_frame = rg();
    fr.x8AC_animSkeleton = skel();
    fr.x8_spawnNum = (rnd() % 6) as i32;
    fr.cur_pos.y = rw(); fr.x2EC = rw();
    fr.pad_port = (rnd() % 4) as u8;
    fr.gr_vel = rw();
    fr.gobj = go();
}

pub fn df_hash() -> Hs {
    let mut h = Hs::new();
    for i in 0..7 { h.m(unsafe { *mvw(i) }); }
    let fr = f();
    h.m(b(fr.facing_dir));
    unsafe {
        h.m(fr.x2210.throw_flags >> 8);
        let t = &fr.x2210.x0;
        h.m(t.throw_flags_b0() as u32); h.m(t.throw_flags_b1() as u32); h.m(t.throw_flags_b2() as u32); h.m(t.throw_flags_b3() as u32);
        h.m(t.throw_flags_b4() as u32); h.m(t.throw_flags_b5() as u32); h.m(t.throw_flags_b6() as u32); h.m(t.throw_flags_b7() as u32);
    }
    h.m(fr.cmd_vars[0]);
    h.m(b(fr.self_vel.x)); h.m(b(fr.self_vel.y)); h.m(b(fr.self_vel.z));
    h.m(b(fr.gr_vel)); h.m(fr.x221D_b5() as u32);
    h
}

/// `BN` of ftco_dfe_fill.h: any NaN as 0x7FC00000.
pub fn bn(x: f32) -> u32 { if x.is_nan() { 0x7FC0_0000 } else { x.to_bits() } }
/// `chk(id)` of ftco_dfe_fill.h.
pub fn chk(id: u32) -> bool { unsafe { (SRV[(10 + id % 6) as usize] >> (id % 7)) & 1 != 0 } }
pub fn is_f(p: &Fighter) -> u32 { (p as *const Fighter == f() as *const Fighter) as u32 }
/// `DS_V(name, id)`.
macro_rules! dv { ($id:expr) => { Some(|g| { t($id); t(is_go(g)); }) }; }
/// `DS_CHK(name, id)`.
macro_rules! dchk { ($id:expr) => { Some(|g| { t($id); t(is_go(g)); chk($id) }) }; }
/// `DS_FV(name, id)`.
macro_rules! dfv { ($id:expr) => { Some(|p| { t($id); t(is_f(p)); }) }; }
