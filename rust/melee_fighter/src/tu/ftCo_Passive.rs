#![allow(non_snake_case, unused_unsafe)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_Passive.c (every function of the TU). Calls out of it
//! go through `ext`. Checked against the C (whole TU, shipping contraction) by tests/tu_ftCo_Passive.rs.
use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
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
        ftCommon_8007D7FC(fp: &mut Fighter) = |_| todo!("ftCommon_8007D7FC: installed by melee_link");
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ft_800881D8(fp: &mut Fighter, sfx: i32, vol: u8, pan: u8) = |_, _, _, _| todo!("ft_800881D8: installed by melee_link");
        ft_PlaySFX(fp: &mut Fighter, id: i32, vol: u8, pan: u8) = |_, _, _, _| todo!("ft_PlaySFX: installed by melee_link");
        ftCommon_SetGroundedKnockbackIfLanded(fp: &mut Fighter) = |a0| crate::tu::ftcommon::ftCommon_SetGroundedKnockbackIfLanded(a0);
        efAsync_Spawn(g: G, queue: *mut core::ffi::c_void, kind: u32, gfx: u32, jobj: *mut HSD_JObj) = |a0, a1, a2, a3, a4| unsafe { let _ = melee_world::tu::efn::efasync::efAsync_Spawn(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, &[]); };
        ftCo_800BFFD0(fp: &mut Fighter, a: i32, b: bool) -> bool = |fp: &mut Fighter, a: i32, b: bool| -> bool { unsafe { crate::tu::ftcolanim::ftco_800bffd0(fp as _, a as _, b as _) as _ } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ft_8008A2BC(g: G) = crate::hooks::ft_8008a2bc;
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ft_80083F88(g: G) = crate::tu::ft_081b::ft_80083f88;
    }
}

pub fn ftCo_800987D0(g: G) {
    melee_core::ct!("ftCo_800987D0");
    let f = fp(g);
    if f.ground_or_air == GA_Air {
        ext::ftCommon_8007D7FC(f);
    }
    ext::Fighter_ChangeMotionState(g, 199, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    let sfx = unsafe { (*(*f.ft_data).x4C_sfx).x24 };
    ext::ft_800881D8(f, sfx, 127, 64);
    ext::ft_PlaySFX(fp(g), 3, 127, 64);
    ext::ftCommon_SetGroundedKnockbackIfLanded(fp(g));
    let f = fp(g);
    let joint = unsafe { (*f.parts.add(FtPart_TopN as usize)).joint };
    // ftCo_SpawnEf (kinds/ftCommon/inlines.h)
    ext::efAsync_Spawn(g, &mut f.x60C as *mut _ as *mut core::ffi::c_void, 1, 1053, joint);
    ext::ftCo_800BFFD0(fp(g), 120, false);
}

pub fn ftCo_Passive_Anim(g: G) {
    melee_core::ct!("ftCo_Passive_Anim");
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ft_8008A2BC(g);
    }
}

pub fn ftCo_Passive_IASA(_g: G) { melee_core::ct!("ftCo_Passive_IASA");}

pub fn ftCo_Passive_Phys(g: G) {
    melee_core::ct!("ftCo_Passive_Phys");
    ext::ft_80084F3C(g);
}

pub fn ftCo_Passive_Coll(g: G) {
    melee_core::ct!("ftCo_Passive_Coll");
    ext::ft_80083F88(g);
}
