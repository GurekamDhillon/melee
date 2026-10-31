#![allow(non_snake_case, unused_unsafe)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_DownStand.c (getup). Every function of the
//! TU; calls out of it go through `ext`. Checked against the C by tests/tu_ftCo_DownStand.rs.
use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[allow(dead_code)]
#[inline(always)]
fn cd() -> &'static ftCommonData {
    crate::hooks::common_data()
}
#[allow(dead_code)]
#[inline(always)]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}
/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    #[allow(unused_imports)]
    use super::*;
    macro_rules! ext_fns {
        ($($name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? = $def:expr;)*) => {
            pub mod hook {
                #[allow(unused_imports)] use super::*;
                $( #[allow(non_upper_case_globals)] pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( #[allow(non_snake_case)] pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } { Some(f) => f($($a),*), None => ($def)($($a),*) }
            } )*
        };
    }
    ext_fns! {
        ftCo_GetLStickAngle(fp: &Fighter) -> f32 = crate::ftcommon3::ftco_get_lstick_angle;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ft_8008A2BC(g: G) = crate::tu::ft_08a1::ft_8008A2BC;
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ft_80083F88(g: G) = |g: G| { unsafe { crate::tu::ft_081b::ft_80083f88(g as _); } };
    }
}

pub fn ftCo_800980BC(g: G) -> bool {
    melee_core::ct!("ftCo_800980BC");
    let f = fp(g);
    let c = cd();
    if (f.input.lstick[0].y >= c.x244 && ext::ftCo_GetLStickAngle(f) >= c.x20_radians)
        || f.input.pressed_buttons & HSD_PAD_LR != 0
    {
        let msid = if f.motion_id as i32 == ftCo_MS_DownWaitU { ftCo_MS_DownStandU } else { ftCo_MS_DownStandD };
        ftCo_80098160(g, msid);
        return true;
    }
    false
}

pub fn ftCo_80098160(g: G, msid: i32) {
    melee_core::ct!("ftCo_80098160");
    ext::Fighter_ChangeMotionState(g, msid, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
}

pub fn ftCo_DownStand_Anim(g: G) {
    melee_core::ct!("ftCo_DownStand_Anim");
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ft_8008A2BC(g);
    }
}

pub fn ftCo_DownStand_IASA(_g: G) { melee_core::ct!("ftCo_DownStand_IASA");}

pub fn ftCo_DownStand_Phys(g: G) {
    melee_core::ct!("ftCo_DownStand_Phys");
    ext::ft_80084F3C(g);
}

pub fn ftCo_DownStand_Coll(g: G) {
    melee_core::ct!("ftCo_DownStand_Coll");
    ext::ft_80083F88(g);
}
