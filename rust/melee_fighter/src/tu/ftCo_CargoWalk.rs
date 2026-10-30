#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_CargoWalk.c (DK cargo walk). Every function
//! of the TU; calls out go through `ext`. Checked by tests/tu_ftCo_CargoWalk.rs.

use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
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
        ftWalkCommon_800DFC70(g: G) -> bool = crate::tu::ftwalkcommon::ftwalkcommon_800dfc70;
        ftWalkCommon_800DFCA4(g: G, msid: i32, flags: u32, anim_start: f32, a: f32, b: f32, c: f32, d: f32, e: f32, h: f32, accel_mul: f32) = crate::tu::ftwalkcommon::ftwalkcommon_800dfca4;
        ftWalkCommon_GetWalkType(g: G) -> u32 = crate::tu::ftwalkcommon::ftwalkcommon_get_walk_type;
        ftWalkCommon_800DFDDC(g: G) = crate::tu::ftwalkcommon::ftwalkcommon_800dfddc;
        ftWalkCommon_800DFEC8(g: G, cb: fn(G, f32)) = crate::tu::ftwalkcommon::ftwalkcommon_800dfec8;
        ftWalkCommon_800E0060(g: G) = crate::tu::ftwalkcommon::ftwalkcommon_800e0060;
        ftCo_8009C5A4(g: G, msid: i32) = |g: G, msid: i32| { unsafe { let _ = crate::tu::ftCo_Shouldered::ftCo_8009C5A4(g as _, msid as _); } };
        ftCo_8009BF3C(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_CargoThrow::ftCo_8009BF3C(g as _) as _ } };
        ftCo_8009BB1C(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_CargoJump::ftCo_8009BB1C(g as _) as _ } };
        ftCo_8009C4F8(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftco_09c4::ftCo_8009C4F8(g as _) as _ } };
        ftCo_8009B4D0(g: G) -> bool = crate::tu::ftCo_CargoWait::ftCo_8009B4D0;
        ftCo_CargoWait_Coll(g: G) = crate::tu::ftCo_CargoWait::ftCo_CargoWait_Coll;
    }
}

pub fn ftCo_8009B67C(g: G) -> bool {
    melee_core::ct!("ftCo_8009B67C");
    if ext::ftWalkCommon_800DFC70(g) {
        ftCo_8009B6C8(g, 0.0);
        return true;
    }
    false
}

/// `ftCo_8009B6C8` (static in the C).
pub fn ftCo_8009B6C8(g: G, anim_start: f32) {
    melee_core::ct!("ftCo_8009B6C8");
    let f = fp(g);
    let a = unsafe { &*f.x2CC };
    ext::ftWalkCommon_800DFCA4(g, a.x4_motion_state + 1, Ft_MF_None, anim_start, a.x8, a.xC, a.x10, a.x14, a.x18, a.x1C, 1.0);
    let victim = fp(g).victim_gobj;
    let wt = ext::ftWalkCommon_GetWalkType(g);
    ext::ftCo_8009C5A4(victim, (wt as i32).wrapping_add(ftCo_MS_ShoulderedWait as i32));
}

pub fn ftCo_CargoWalk_Anim(g: G) {
    melee_core::ct!("ftCo_CargoWalk_Anim");
    ext::ftWalkCommon_800DFDDC(g);
}

pub fn ftCo_CargoWalk_IASA(g: G) {
    melee_core::ct!("ftCo_CargoWalk_IASA");
    if ext::ftCo_8009BF3C(g) { return; }
    if ext::ftCo_8009BB1C(g) { return; }
    if ext::ftCo_8009C4F8(g) { return; }
    if ext::ftCo_8009B4D0(g) { return; }
    ext::ftWalkCommon_800DFEC8(g, ftCo_8009B6C8);
}

pub fn ftCo_CargoWalk_Phys(g: G) {
    melee_core::ct!("ftCo_CargoWalk_Phys");
    ext::ftWalkCommon_800E0060(g);
}

pub fn ftCo_CargoWalk_Coll(g: G) {
    melee_core::ct!("ftCo_CargoWalk_Coll");
    ext::ftCo_CargoWait_Coll(g);
}
