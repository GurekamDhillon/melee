#![allow(non_snake_case, unused_unsafe)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_PassiveStand.c (every function of the TU). Calls out of it
//! go through `ext`. Checked against the C (whole TU, shipping contraction) by tests/tu_ftCo_PassiveStand.rs.
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
        ftCo_800986B0(g: G) -> bool = crate::tu::ftCo_DownAttack::ftCo_800986B0;
        ftCommon_8007D7FC(fp: &mut Fighter) = |fp: &mut Fighter| { unsafe { let _ = crate::ftcommon3::ftcommon_8007d7fc(fp as _); } };
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ft_800881D8(fp: &mut Fighter, sfx: i32, vol: u8, pan: u8) = |fp: &mut Fighter, sfx: i32, vol: u8, pan: u8| { unsafe { let _ = crate::ft0881::ft_800881d8(fp as _, sfx as _, vol as _, pan as _); } };
        ft_PlaySFX(fp: &mut Fighter, id: i32, vol: u8, pan: u8) = |fp: &mut Fighter, id: i32, vol: u8, pan: u8| { unsafe { let _ = crate::ft0877::play_sfx(fp, id as _, vol as _, pan as _); } };
        efAsync_Spawn(g: G, queue: *mut core::ffi::c_void, kind: u32, gfx: u32, jobj: *mut HSD_JObj) = |_, _, _, _, _| todo!("TODO(port) efAsync_Spawn (efasync.c)");
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ft_8008A2BC(g: G) = crate::hooks::ft_8008a2bc;
        ft_80084FA8(g: G) = |g: G| { unsafe { let _ = crate::ft084e::ft_80084fa8(g as _); } };
        ft_80084104(g: G) = crate::tu::ft_081b::ft_80084104;
    }
}

pub fn ftCo_80098928(g: G) -> bool {
    melee_core::ct!("ftCo_80098928");
    if ext::ftCo_800986B0(g) {
        let f = fp(g);
        let x = f.input.lstick[0].x;
        let ax = if x < 0.0 { -x } else { x };
        if ax >= cd().x254 {
            let msid = if x * f.facing_dir >= 0.0 { ftCo_MS_PassiveStandF } else { ftCo_MS_PassiveStandB };
            ftCo_800989D4(g, msid as i32);
            return true;
        }
    }
    false
}

pub fn ftCo_800989D4(g: G, msid: i32) {
    melee_core::ct!("ftCo_800989D4");
    let f = fp(g);
    if f.ground_or_air == GA_Air {
        ext::ftCommon_8007D7FC(f);
    }
    ext::Fighter_ChangeMotionState(g, msid, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    let sfx = unsafe { (*(*f.ft_data).x4C_sfx).x24 };
    ext::ft_800881D8(f, sfx, 127, 64);
    ext::ft_PlaySFX(fp(g), 3, 127, 64);
    let f = fp(g);
    let joint = unsafe { (*f.parts.add(FtPart_TopN as usize)).joint };
    // ftCo_SpawnEf (kinds/ftCommon/inlines.h)
    ext::efAsync_Spawn(g, &mut f.x60C as *mut _ as *mut core::ffi::c_void, 1, 1053, joint);
}

pub fn ftCo_PassiveStand_Anim(g: G) {
    melee_core::ct!("ftCo_PassiveStand_Anim");
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ft_8008A2BC(g);
    }
}

pub fn ftCo_PassiveStand_IASA(_g: G) { melee_core::ct!("ftCo_PassiveStand_IASA");}

pub fn ftCo_PassiveStand_Phys(g: G) {
    melee_core::ct!("ftCo_PassiveStand_Phys");
    ext::ft_80084FA8(g);
}

pub fn ftCo_PassiveStand_Coll(g: G) {
    melee_core::ct!("ftCo_PassiveStand_Coll");
    ext::ft_80084104(g);
}
