//! Written by gen/mtu/if_drv.py from the same description as gen/mtu/tu/ifstatus.c.
//! src/tu/ifstatus.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_world/gen/mtu/run_if.py ifstatus`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_world::tu::ifstatus as m;
use mtu_harness::*;
#[path = "mtu/ifstatus_gen.rs"]
mod g;

use melee_world::tu::ifstatus::UnkX;
pub static mut GCOL: [_GXColor; 3] = unsafe { core::mem::zeroed() };
pub static mut GOP: [*mut HSD_GObj; 1] = [null_mut(); 1];
pub static mut CHR: [u32; 1] = [0; 1];
unsafe extern "C" fn cbA(a: i32) { t(7001); t(a as u32); }
unsafe extern "C" fn cbB(a: i32) { t(7002); t(a as u32); }
fn cbsel() -> usize { let q = rnd() % 3; if q == 0 { 0 } else if q == 1 { cbA as usize } else { cbB as usize } }
pub static mut MAJP: [*mut HSD_MatAnimJoint; 2] = [null_mut(); 2];
pub static mut NMOD: [*mut DynamicModelDesc; 2] = [null_mut(); 2];
pub static mut ARCP: *mut HSD_Archive = null_mut();
pub static mut V3A: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
fn rcol() -> _GXColor { let r = fv() as u8; let g = fv() as u8; let b = fv() as u8; let a = fv() as u8; _GXColor { r, g, b, a } }




unsafe fn setup_extra() {
    let mut n: u32;
    let mut i: usize;
for k in 0..6usize { g::P_HSD_GObj[k].hsd_obj = &raw mut g::P_HSD_JObj[k] as *mut _; }
}

#[test]
fn ifstatus_matches_c() {
    unsafe {
        g::install();
        g::preg_state();
        g::freg_all();
        freg(cbA as *const () as usize);
        freg(cbB as *const () as usize);
        let mut c = Check::new(include_str!("data/mtu_ifstatus.txt"));
        let nc: [u32; 33] = [400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400];
        case(99, 0); g::hash_init_state(); c.line(99, 0, 0);
        for op in 0..33u32 {
            for k in 0..nc[op as usize] {
                case(op, k);
                g::reset_state();
                setup_extra();
                let mut ret = 0u32;
                match op {
                    0 => {   ret = pi(m::ifStatus_GetHUDInfo()); }
                    1 => {  let a0: *mut IfDamageState = (&raw mut m::ifStatus_HudInfo.players[(rnd() % 6) as usize]) as *mut IfDamageState; let a1: i32 = (fv()) as i32; let a2: i32 = (fv()) as i32; m::ifStatus_PercentOnDeathAnimationThink(a0, a1, a2); }
                    2 => {  let a0: f32 = (rg()) as f32; let a1: f32 = (rg()) as f32; ret = m::foo(a0, a1).to_bits(); }
                    3 => {   ret = m::offset_rand().to_bits(); }
                    4 => {  let a0: *mut HSD_JObj = g::pick_HSD_JObj(0) as *mut HSD_JObj; m::jobj_flagCheckSetMtxDirtySub(a0); }
                    5 => {  let a0: *mut IfDamageState = (&raw mut m::ifStatus_HudInfo.players[(rnd() % 6) as usize]) as *mut IfDamageState; let a1: i32 = (fv()) as i32; m::ifStatus_802F4B84(a0, a1); }
                    6 => {  let a0: *mut IfDamageState = (&raw mut m::ifStatus_HudInfo.players[(rnd() % 6) as usize]) as *mut IfDamageState; let a1: *mut *mut *mut HSD_MatAnimJoint = (&raw mut m::ifStatus_HudInfo.damage_num_matanims) as *mut *mut *mut HSD_MatAnimJoint; m::ifStatus_InitDamageDigits(a0, a1); }
                    7 => {  let a0: *mut IfDamageState = (&raw mut m::ifStatus_HudInfo.players[(rnd() % 6) as usize]) as *mut IfDamageState; let a1: i32 = (rnd() % 4) as i32; ret = m::ifStatus_GetStoredTranslationX(a0, a1).to_bits(); }
                    8 => {  let a0: *mut *mut HSD_GObj = (&raw mut GOP[0]) as *mut *mut HSD_GObj; m::ifStatus_CreateMarkGObj(a0); }
                    9 => {  let a0: i32 = (fv()) as i32; let a1: *mut u32 = (&raw mut CHR[0]) as *mut u32; m::ifStatus_GetPlayerCharacter(a0, a1); }
                    10 => {  let a0: *mut IfDamageState = (&raw mut m::ifStatus_HudInfo.players[(rnd() % 6) as usize]) as *mut IfDamageState; let a1: *mut HSD_JObj = g::pick_HSD_JObj(0) as *mut HSD_JObj; let a2: *mut _GXColor = (&raw mut GCOL[0]) as *mut _GXColor; let a3: *mut _GXColor = (&raw mut GCOL[1]) as *mut _GXColor; let a4: *mut _GXColor = (&raw mut GCOL[2]) as *mut _GXColor; m::ifStatus_UpdateDamageDisplay(a0, a1, a2, a3, a4); }
                    11 => { m::ifStatus_HudInfo.players[(rnd() % 6) as usize].HUD_parent_entity = &raw mut g::P_HSD_GObj[0]; let a0: *mut HSD_GObj = (&raw mut g::P_HSD_GObj[0]) as *mut HSD_GObj; m::ifStatus_802F4EDC(a0); }
                    12 => { m::ifStatus_HudInfo.players[(rnd() % 6) as usize].HUD_parent_entity = &raw mut g::P_HSD_GObj[0]; for q in 0..6usize { *(&raw mut g::P_VsSceneController[0].state.fighters[q] as *mut u8).add(4) = if rnd() & 1 != 0 { 0x24 } else { 0 }; } let a0: *mut HSD_GObj = (&raw mut g::P_HSD_GObj[0]) as *mut HSD_GObj; m::ifStatus_802F5B48(a0); }
                    13 => { m::ifStatus_HudInfo.players[(rnd() % 6) as usize].HUD_parent_entity = &raw mut g::P_HSD_GObj[0]; let a0: *mut HSD_GObj = (&raw mut g::P_HSD_GObj[0]) as *mut HSD_GObj; let a1: i32 = (fv()) as i32; m::ifStatus_802F5DE0(a0, a1); }
                    14 => { m::ifStatus_HudInfo.players[(rnd() % 6) as usize].next = &raw mut g::P_HSD_GObj[0]; let a0: *mut HSD_GObj = (&raw mut g::P_HSD_GObj[0]) as *mut HSD_GObj; let a1: i32 = (fv()) as i32; m::ifStatus_802F5E50(a0, a1); }
                    15 => { for q in 0..6usize { m::ifStatus_HudInfo.players[q].unk9 = 0; } for q in 0..6usize { *(&raw mut g::P_VsSceneController[0].state.fighters[q] as *mut u8).add(4) = if rnd() & 1 != 0 { 0x24 } else { 0 }; } let a0: *mut IfDamageState = (&raw mut m::ifStatus_HudInfo.players[(rnd() % 6) as usize]) as *mut IfDamageState; let a1: i32 = (rnd() % 6) as i32; ret = pi(m::ifStatus_802F5EC0(a0, a1)); }
                    16 => { if rnd() & 1 != 0 { g::P_HSD_JObj[(rnd() % 10) as usize].next = null_mut(); } let a0: *mut HSD_JObj = g::pick_HSD_JObj(1) as *mut HSD_JObj; let a1: i32 = ((-2 + (rnd() % 8) as i32)) as i32; ret = pi(m::ifStatus_802F6194(a0, a1)); }
                    17 => {  let a0: *mut IfDamageState = (&raw mut m::ifStatus_HudInfo.players[(rnd() % 6) as usize]) as *mut IfDamageState; let a1: i32 = (rnd() % 6) as i32; ret = pi(m::ifStatus_802F61FC(a0, a1)); }
                    18 => { for q in 0..6usize { m::ifStatus_HudInfo.players[q].unk9 = 0; } for q in 0..6usize { *(&raw mut g::P_VsSceneController[0].state.fighters[q] as *mut u8).add(4) = if rnd() & 1 != 0 { 0x24 } else { 0 }; } let a0: i32 = (rnd() % 6) as i32; m::ifStatus_802F6508(a0); }
                    19 => { for q in 0..6usize { m::ifStatus_HudInfo.players[q].unk9 = 0; } for q in 0..6usize { *(&raw mut g::P_VsSceneController[0].state.fighters[q] as *mut u8).add(4) = if rnd() & 1 != 0 { 0x24 } else { 0 }; } let a0: i32 = ((0 + (rnd() % 8) as i32)) as i32; m::ifStatus_802F665C(a0); }
                    20 => {   m::ifStatus_802F66A4(); }
                    21 => {  let a0: u8 = (rnd() % 6) as u8; m::ifStatus_802F6788(a0); }
                    22 => {   m::ifStatus_802F6804(); }
                    23 => {   m::ifStatus_802F6898(); }
                    24 => {   m::ifStatus_802F68F0(); }
                    25 => {  let a0: i32 = (rnd() % 6) as i32; m::ifStatus_802F6948(a0); }
                    26 => {  let a0: i32 = (rnd() % 6) as i32; let a1: Option<unsafe extern "C" fn(i32)> = (core::mem::transmute::<usize, Option<unsafe extern "C" fn(i32)>>(cbsel())) as Option<unsafe extern "C" fn(i32)>; m::ifStatus_TriggerStockLoss(a0, a1); }
                    27 => {  let a0: i32 = (rnd() % 6) as i32; let a1: i32 = (fv()) as i32; m::ifStatus_802F69C0(a0, a1); }
                    28 => {  let a0: i32 = (rnd() % 6) as i32; m::ifStatus_802F6AF8(a0); }
                    29 => {  let a0: i32 = (rnd() % 6) as i32; m::ifStatus_802F6C04(a0); }
                    30 => {  let a0: i32 = (rnd() % 6) as i32; m::ifStatus_802F6D10(a0); }
                    31 => { for q in 0..6usize { m::ifStatus_HudInfo.players[q].unk9 = 0; } for q in 0..6usize { *(&raw mut g::P_VsSceneController[0].state.fighters[q] as *mut u8).add(4) = if rnd() & 1 != 0 { 0x24 } else { 0 }; } let a0: i32 = (rnd() % 6) as i32; m::ifStatus_802F6E1C(a0); }
                    32 => {  let a0: i32 = (rnd() % 6) as i32; m::ifStatus_802F6E3C(a0); }
                    _ => unreachable!(),
                }
                g::hash_state();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 13201);
    }
}
