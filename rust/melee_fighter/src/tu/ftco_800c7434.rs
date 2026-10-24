//! src/melee/ft/ftCo_800C7434.c: Giga Bowser selector, the leadead capture trigger
//! (`ftCo_800C74F4`) and its damage callbacks. Static inlines inlineB0..B2 are `ftCo_800C7070`.

use super::ftco_800c7070::{ext as e70, fp, ftco_800c7070, G};
use super::ftco_800c70d0::ext as e0d0;
use crate::types::*;

/// `ftCo_800C7434(gobj)`: Giga Bowser -> motion state 0xE (Wait).
pub fn ftco_800c7434(g: G) {
    let ms = if fp(g).kind == Ft_Kind_GKoops {
        0xE
    } else {
        ftco_800c7070(g);
        return;
    };
    e70::Fighter_ChangeMotionState(g, ms, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    f.set_x2219_b2(1);
    f.set_x2219_b1(1);
}

/// `ftCo_800C74AC(gobj)`: `ft_8008521C(gobj)`.
pub fn ftco_800c74ac(g: G) {
    e0d0::ft_8008521C(g)
}

/// `fn_800C74CC(gobj)`: take_dmg_cb of CaptureLeadead.
pub unsafe extern "C" fn fn_800c74cc(g: G) {
    ext::it_802EADD8(unsafe { fp(g).mv.co.captureleadead.x0 })
}

/// `ftCo_800C74F4(gobj)`: hit by a ReDead (leadead) grab -> CaptureLeadead.
pub fn ftco_800c74f4(g: G) -> bool {
    let f = fp(g);
    if f.dmg.x1860_element == HitElement_Leadead
        && f.capture_timer == 0
        && f.motion_id != ftCo_MS_CaptureLeadead
        && ext::it_802EAF28(f.dmg.x1868_source) == 0
    {
        super::ftco_800c7590::ftco_800c7590(g);
        return true;
    }
    false
}

/// `fn_800C7568(gobj)`: death2_cb of CaptureLeadead.
pub unsafe extern "C" fn fn_800c7568(g: G) {
    ext::it_802EADD8(unsafe { fp(g).mv.co.captureleadead.x0 })
}

pub mod ext {
    #![allow(non_snake_case, static_mut_refs)]
    use crate::types::*;
    pub static mut IT_802EADD8: Option<fn(*mut Item_GObj)> = None;
    pub static mut IT_802EAF28: Option<fn(*mut Item_GObj) -> i32> = None;
    /// TODO(group5 itleadead.c) `it_802EADD8`.
    pub fn it_802EADD8(i: *mut Item_GObj) {
        match unsafe { IT_802EADD8 } {
            Some(f) => f(i),
            None => { unsafe { let _ = melee_items::tu::itleadead::it_802EADD8(i as _); } return; },
        }
    }
    /// TODO(group5 itleadead.c) `it_802EAF28`.
    pub fn it_802EAF28(i: *mut Item_GObj) -> i32 {
        match unsafe { IT_802EAF28 } {
            Some(f) => f(i),
            None => { return unsafe { melee_items::tu::itgreatfoxlaser::it_802EAF28(i as _) as _ }; },
        }
    }
}
