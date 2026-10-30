#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_HammerJump.c. Every function of the TU; calls out go
//! through `ext`. Checked against the C by tests/tu_ftCo_HammerJump.rs.

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

/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
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
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCo_800C54C4(fp: &mut Fighter) -> u32 = |fp: &mut Fighter| -> u32 { unsafe { crate::tu::ftCo_HammerWait::ftCo_800C54C4(fp as _) as _ } };
        ftCo_800C548C(fp: &mut Fighter) -> f32 = |fp: &mut Fighter| -> f32 { unsafe { crate::tu::ftCo_HammerWait::ftCo_800C548C(fp as _) as _ } };
        ftCo_800C4E94(fp: &mut Fighter) = |fp: &mut Fighter| { unsafe { let _ = crate::tu::ftCo_HammerWait::ftCo_800C4E94(fp as _); } };
        ftCo_800C4F64(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_HammerWait::ftCo_800C4F64(g as _); } };
        ftCo_Jump_GetInput(g: G) -> u32 = |g: G| -> u32 { unsafe { crate::tu::ftCo_Jump::ftCo_Jump_GetInput(g as _) as _ } };
        ftCo_800C5B88(g: G, e: i32) = crate::tu::ftCo_HammerKneeBend::ftCo_800C5B88;
        ftCommon_8007D5D4(fp: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        ftCo_800CB110(g: G, b: bool, v: f32) = |g: G, b: bool, v: f32| { unsafe { let _ = crate::tu::ftCo_Jump::ftCo_800CB110(g as _, b as _, v as _); } };
        ftCo_Jump_Phys_Inner(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_Jump::ftCo_Jump_Phys_Inner(g as _); } };
        ftCo_HammerFall_Coll(g: G) = crate::tu::ftCo_HammerFall::ftCo_HammerFall_Coll;
    }
}

pub fn ftCo_800C5A50(g: G) -> bool {
    melee_core::ct!("ftCo_800C5A50");
    let e = ext::ftCo_Jump_GetInput(g);
    if e != 0 {
        ext::ftCo_800C5B88(g, e as i32);
        return true;
    }
    false
}

pub fn ftCo_800C5A98(g: G) {
    melee_core::ct!("ftCo_800C5A98");
    ext::ftCommon_8007D5D4(fp(g));
    let flags = ext::ftCo_800C54C4(fp(g));
    let start = ext::ftCo_800C548C(fp(g));
    ext::Fighter_ChangeMotionState(g, ftCo_MS_HammerJump as i32, flags, start, 1.0, 0.0, core::ptr::null_mut());
    ext::ftCo_800CB110(g, false, 1.0);
    ext::ftCo_800C4E94(fp(g));
}

pub fn ftCo_HammerJump_Anim(g: G) {
    melee_core::ct!("ftCo_HammerJump_Anim");
    ext::ftCo_800C4F64(g);
}

pub fn ftCo_HammerJump_IASA(_g: G) { melee_core::ct!("ftCo_HammerJump_IASA");}

pub fn ftCo_HammerJump_Phys(g: G) {
    melee_core::ct!("ftCo_HammerJump_Phys");
    ext::ftCo_Jump_Phys_Inner(g);
}

pub fn ftCo_HammerJump_Coll(g: G) {
    melee_core::ct!("ftCo_HammerJump_Coll");
    ext::ftCo_HammerFall_Coll(g);
}
