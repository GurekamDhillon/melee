//! Hand port, round 9: leaf functions of ft/fighter.c: the spawn-number counter, two hitlag
//! flag helpers, the position getters, and the TARGET_PC UCF 0.84 pad ring + flick counter.
//! The C statics (`Fighter_804D64F8`, `ftUcf_Pad`) are module statics here (single-threaded
//! game state, like the C). Checked against the C (cut verbatim) by tests/ftleaf_ref.rs.
#![allow(static_mut_refs)]

use melee_core::fp::Fmadd as _;
use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// `Fighter_804D64F8` (`g_spawnNumCounter`); Fighter_FirstInitialize sets it to 1.
pub static mut SPAWN_NUM_COUNTER: u32 = 0;

/// `Fighter_NewSpawn_80068E40`: post-increment, skipping 0 on wrap.
pub fn fighter_new_spawn_80068e40() -> u32 {
    unsafe {
        let n = SPAWN_NUM_COUNTER;
        SPAWN_NUM_COUNTER = SPAWN_NUM_COUNTER.wrapping_add(1);
        if SPAWN_NUM_COUNTER == 0 {
            SPAWN_NUM_COUNTER = 1;
        }
        n
    }
}

/// `Fighter_UnkSetFlag_8006CFBC`.
pub fn fighter_unk_set_flag_8006cfbc(g: G) {
    let f = fp(g);
    if f.x2219_b7() != 0 {
        f.set_x221A_b1(1);
    }
}

/// `setBit` (static inline): `x2219_b7 = 1`.
pub fn set_bit(g: G) {
    fp(g).set_x2219_b7(1);
}

/// `Fighter_GetPosX` (static inline).
pub fn fighter_get_pos_x(f: &Fighter) -> f32 {
    f.cur_pos.x
}
/// `Fighter_GetPosY` (static inline).
pub fn fighter_get_pos_y(f: &Fighter) -> f32 {
    f.cur_pos.y
}

/// One port of `ftUcf_Pad` (TARGET_PC): 4-deep raw stick ring, newest index, flick counter.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct UcfPad {
    pub x: [i8; 4],
    pub y: [i8; 4],
    pub idx: u8,
    pub counter: u8,
}
/// `ftUcf_Pad[4]`.
pub static mut UCF_PAD: [UcfPad; 4] = [UcfPad { x: [0; 4], y: [0; 4], idx: 0, counter: 0 }; 4];

/// `ftUcf_PadBufferPush`.
pub fn ftucf_pad_buffer_push(port: i32, x: i32, y: i32) {
    let p = unsafe { &mut UCF_PAD[port as usize] };
    let i = p.idx.wrapping_add(1) & 3;
    p.idx = i;
    p.x[i as usize] = x as i8;
    p.y[i as usize] = y as i8;
}

/// `ftUcf_PadRaw`.
pub fn ftucf_pad_raw(port: i32, which_y: i32, back: i32) -> i32 {
    let p = unsafe { &UCF_PAD[port as usize] };
    let i = ((p.idx as i32).wrapping_sub(back) & 3) as usize;
    if which_y != 0 { p.y[i] as i32 } else { p.x[i] as i32 }
}

/// `ftUcf_FlickCounter`.
pub fn ftucf_flick_counter(port: i32) -> i32 {
    unsafe { UCF_PAD[port as usize].counter as i32 }
}

/// `ftUcf_RimUnits`: `(f32)((double)|v| * 80.0 - (double)1e-4f)`, truncated, + 2. The C mul-sub
/// contracts to an f64 fma (shipping build); written as that fma here.
pub fn ftucf_rim_units(v: f32) -> i32 {
    let a = if v < 0.0 { -v } else { v };
    let t = (a as f64).fmadd(80.0, -(9.99999974738e-05f32 as f64)) as f32;
    (melee_core::ppc::fctiwz_f32(t)).wrapping_add(2)
}

/// `ftUcf_FlickCount` (static).
pub fn ftucf_flick_count(f: &Fighter, port: i32) {
    let mut c: i32 = 0;
    if !(f.input.lstick[0].y > -0.609375) {
        let rx = ftucf_rim_units(f.input.lstick[0].x);
        let ry = ftucf_rim_units(f.input.lstick[0].y);
        if rx.wrapping_mul(rx).wrapping_add(ry.wrapping_mul(ry)) > 6400 {
            let cur = unsafe { UCF_PAD[port as usize].counter };
            if cur != 0 {
                c = cur.wrapping_add(1) as i32;
            } else if f.active_timer.lstick.y <= 1 {
                let dy = ftucf_pad_raw(port, 1, 0) - ftucf_pad_raw(port, 1, 2);
                c = if dy * dy > 1936 { 1 } else { 0 };
            }
        }
    }
    unsafe { UCF_PAD[port as usize].counter = c as u8 };
}
