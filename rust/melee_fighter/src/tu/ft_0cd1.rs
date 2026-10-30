//! src/melee/ft/ft_0CD1.c: the shared item-swing state (sword / bat / parasol / fan) helpers.
//! Only float op: ftCo_800CD278 `x420 * ground_friction` (a plain product, no fused site).
#![allow(non_snake_case)]

use crate::hooks as h;
use crate::types::*;

pub type G = *mut HSD_GObj;

const HSD_PAD_A: u32 = 0x100;

#[inline(always)]
pub(crate) fn fp<'a>(g: G) -> &'a mut Fighter {
    melee_core::ct!("fp");
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// inlines.h `ftCheckThrowB3`: test and clear `throw_flags_b3`.
pub fn ft_check_throw_b3(f: &mut Fighter) -> bool {
    melee_core::ct!("ft_check_throw_b3");
    unsafe {
        if f.anon_2210.anon_0.throw_flags_b3() != 0 {
            f.anon_2210.anon_0.set_throw_flags_b3(0);
            true
        } else {
            false
        }
    }
}
/// inlines.h `ftCheckThrowB0`.
pub fn ft_check_throw_b0(f: &mut Fighter) -> bool {
    melee_core::ct!("ft_check_throw_b0");
    unsafe {
        if f.anon_2210.anon_0.throw_flags_b0() != 0 {
            f.anon_2210.anon_0.set_throw_flags_b0(0);
            true
        } else {
            false
        }
    }
}
/// inlines.h `ftCheckThrowB4`.
pub fn ft_check_throw_b4(f: &mut Fighter) -> bool {
    melee_core::ct!("ft_check_throw_b4");
    unsafe {
        if f.anon_2210.anon_0.throw_flags_b4() != 0 {
            f.anon_2210.anon_0.set_throw_flags_b4(0);
            true
        } else {
            false
        }
    }
}

/// Callees outside this TU that have no port yet. Each panics until rewired. Other TUs of this
/// shard (ft_0CD3 / 0CDD / 0CDF / 0CE3) share these.
pub mod ext {
    use crate::types::*;
    type G = *mut HSD_GObj;
    pub fn ftAnim_8006EBA4(_g: G) { { unsafe { let _ = crate::tu::ftanim::ftAnim_8006EBA4(_g as _); } return; } }
    pub fn ftAnim_IsFramesRemaining(_g: G) -> bool { { return unsafe { crate::tu::ftanim::ftAnim_IsFramesRemaining(_g as _) as _ }; } }
    pub fn ftAnim_80070734(_j: *mut HSD_JObj, _f: f32) { { unsafe { let _ = crate::tu::ftanim::ftAnim_80070734(_j as _, _f as _); } return; } }
    pub fn Fighter_ChangeMotionState(_g: G, _msid: i32, _flags: u32, _start: f32, _speed: f32, _blend: f32, _arg: G) {
        { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(_g as _, _msid as _, _flags as _, _start as _, _speed as _, _blend as _, _arg as _); } return; }
    }
    pub fn ft_800827A0(_g: G) -> bool { { return unsafe { crate::tu::ft_081b::ft_800827a0(_g as _) as _ }; } }
    pub fn ft_8008A2BC(_g: G) { { unsafe { let _ = crate::tu::ft_08a1::ft_8008A2BC(_g as _); } return; } }
    pub fn ftCo_Attack_800CCF58(_g: G, _i: i32) { { unsafe { let _ = crate::tu::ftswing::ftCo_Attack_800CCF58(_g as _, _i as _); } return; } }
    pub extern "C" fn ftCo_Fall_Enter(_g: G) { { unsafe { let _ = crate::tu::ftCo_Fall::ftCo_Fall_Enter(_g as _); } return; } }
}

/// `ftCo_800CD140(gobj, msid, flags, kind, anim_speed)`: enter a swing state.
pub fn ftco_800cd140(g: G, arg1: i32, arg2: i32, arg3: i32, arg4: f32) {
    melee_core::ct!("ftco_800cd140");
    unsafe { fp(g).anon_2210.throw_flags = 0 };
    ext::Fighter_ChangeMotionState(g, arg1, arg2 as u32, 0.0, arg4, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    let f = fp(g);
    unsafe {
        f.mv.co.swing.x0 = 1;
        f.mv.co.swing.x4 = arg3;
    }
    crate::ftcommon3::ftcommon_8007e79c(g, 1);
}

/// `ftCo_800CD1BC` (Anim).
pub fn ftco_800cd1bc(g: G) {
    melee_core::ct!("ftco_800cd1bc");
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ft_8008A2BC(g);
        crate::ftcommon3::ftcommon_8007e7e4(g, 1);
    }
}

/// `ftCo_800CD204` (IASA): A released cancels the repeat; throw flag b3 + still held = smash.
pub fn ftco_800cd204(g: G) {
    melee_core::ct!("ftco_800cd204");
    let f = fp(g);
    if f.input.held_buttons[0] as u32 & HSD_PAD_A == 0 {
        unsafe { f.mv.co.swing.x0 = 0 };
    }
    if ft_check_throw_b3(f) && unsafe { f.mv.co.swing.x0 } != 0 {
        ext::ftCo_Attack_800CCF58(g, 3);
    }
}

/// Test/rewire overrides for the ported callees of ftCo_800CD278 (unset = the crate's port).
pub static mut FT_80084FA8: Option<fn(G)> = None;
pub static mut FT_80085030: Option<fn(G, f32, f32)> = None;

/// `ftCo_800CD278` (Phys).
pub fn ftco_800cd278(g: G) {
    melee_core::ct!("ftco_800cd278");
    let f = fp(g);
    if unsafe { f.mv.co.swing.x4 } != 4 {
        match unsafe { FT_80084FA8 } { Some(h) => h(g), None => crate::ft084e::ft_80084fa8(g) }
    } else {
        let fr = h::common_data().x420 * f.co_attrs.ground_friction;
        let dir = f.facing_dir;
        match unsafe { FT_80085030 } { Some(h) => h(g, fr, dir), None => crate::ft084e::ft_80085030(g, fr, dir) }
    }
}

/// `ftCo_800CD2C4(gobj, cb)` (Coll).
pub fn ftco_800cd2c4(g: G, cb: unsafe extern "C" fn(G)) {
    melee_core::ct!("ftco_800cd2c4");
    if !ext::ft_800827A0(g) {
        unsafe { cb(g) };
        ext::ftCo_Fall_Enter(g);
    }
}

/// `ft_800CD31C`: take-damage callback, drop the item.
pub extern "C" fn ft_800cd31c(g: G) {
    melee_core::ct!("ft_800cd31c");
    if !fp(g).item_gobj.is_null() {
        crate::ftcommon3::ftcommon_8007e7e4(g, 1);
    }
}
