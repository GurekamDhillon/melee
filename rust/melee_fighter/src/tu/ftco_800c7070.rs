//! src/melee/ft/ftCo_800C7070.c: `ftCo_800C7070`, the default "rebirth wait" entry shared by the
//! per-kind wait selectors of ftCo_800C70D0..800C7434.

use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
pub(crate) fn fp<'a>(g: G) -> &'a mut Fighter {
    melee_core::ct!("fp");
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// `ftCo_800C7070(gobj)`: enter RebirthWait, set x2219_b2 and x2219_b1.
pub fn ftco_800c7070(g: G) {
    melee_core::ct!("ftco_800c7070");
    ext::Fighter_ChangeMotionState(g, ftCo_MS_RebirthWait, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    f.set_x2219_b2(1);
    f.set_x2219_b1(1);
}

/// Calls into TUs that are not ported yet. Each is a settable hook; unset, it panics.
pub mod ext {
    #![allow(non_snake_case, static_mut_refs)]
    use crate::types::*;
    pub type ChangeMotionStateFn = fn(*mut HSD_GObj, FtMotionId, MotionFlags, f32, f32, f32, *mut HSD_GObj);
    pub static mut FIGHTER_CHANGE_MOTION_STATE: Option<ChangeMotionStateFn> = None;
    /// TODO(group3 fighter.c) `Fighter_ChangeMotionState`.
    pub fn Fighter_ChangeMotionState(g: *mut HSD_GObj, msid: FtMotionId, flags: MotionFlags, start: f32, speed: f32, blend: f32, o: *mut HSD_GObj) {
        match unsafe { FIGHTER_CHANGE_MOTION_STATE } {
            Some(f) => f(g, msid, flags, start, speed, blend, o),
            None => { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, o as _); } return; },
        }
    }
}
