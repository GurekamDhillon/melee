//! src/melee/ft/kinds/ftCommon/ftCo_0C60.c: hammer fall entry. No fused sites.
//! Callees outside the ported set are `ext` stubs (panic until their TU is ported).
#![allow(non_snake_case)]

use crate::types::*;

pub type G = *mut Fighter_GObj;

pub mod ext {
    use crate::types::*;
    /// TODO(ftCo_Pass.c) `ftCo_80099F1C`.
    pub fn ftCo_80099F1C(_g: *mut Fighter_GObj) -> bool { { return unsafe { crate::tu::ftCo_Pass::ftCo_80099F1C(_g as _) as _ }; } }
    /// TODO(ftCo_Pass.c) `ftCo_8009A184(gobj, msid, flags, anim_start)`.
    pub fn ftCo_8009A184(_g: *mut Fighter_GObj, _msid: i32, _flags: MotionFlags, _f: f32) { { unsafe { crate::tu::ftCo_Pass::ftCo_8009A184(_g as _, _msid as _, _flags as _, _f as _); } return; } }
    /// TODO(ftCo_HammerWait.c) `ftCo_800C4E94(fp)`.
    pub fn ftCo_800C4E94(_fp: &mut Fighter) { { unsafe { crate::tu::ftCo_HammerWait::ftCo_800C4E94(_fp as _); } return; } }
}

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// `ftCo_800C60C8`.
pub fn ftco_800c60c8(g: G) -> bool {
    melee_core::ct!("ftco_800c60c8");
    if ext::ftCo_80099F1C(g) {
        ftco_800c6110(g);
        return true;
    }
    false
}

/// `ftCo_800C6110`.
pub fn ftco_800c6110(g: G) {
    melee_core::ct!("ftco_800c6110");
    ext::ftCo_8009A184(g, ftCo_MS_HammerFall as i32, Ft_MF_None, 0.0);
    ext::ftCo_800C4E94(fp(g));
}
