//! src/melee/ft/ft_0BEF.c: `ftCo_800BEFD4`, the DeadUpFallIce pose. No float math.
#![allow(non_snake_case)]

use crate::types::*;

pub type G = *mut HSD_GObj;

/// Callees outside this TU with no port yet: settable, panic while unset.
pub mod ext {
    use crate::types::*;
    type G = *mut HSD_GObj;
    pub static mut FIGHTER_CHANGE_MOTION_STATE: Option<fn(G, i32, u32, f32, f32, f32, G)> = None;
    pub fn Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, frame: f32, rate: f32, blend: f32, other: G) {
        match unsafe { FIGHTER_CHANGE_MOTION_STATE } {
            Some(f) => f(g, msid, flags, frame, rate, blend, other),
            None => { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, frame as _, rate as _, blend as _, other as _); } return; },
        }
    }
}

/// `ftCo_800BEFD4`.
pub fn ftCo_800BEFD4(g: G) {
    melee_core::ct!("ftCo_800BEFD4");
    ext::Fighter_ChangeMotionState(g, ftCo_MS_DeadUpFallIce as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = unsafe { &mut *((*g).user_data as *mut Fighter) };
    f.set_x2219_b2(1);
    f.set_x2219_b1(1);
}
