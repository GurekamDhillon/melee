#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_ShieldBreakFly.c (TARGET_PC build). Every function of the TU;
//! calls out of it go through `ext` (unported ones panic until rewired). Checked against the C
//! (whole TU, shipping contraction) by tests/tu_ftCo_ShieldBreakFly.rs.
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

/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    use super::*;
    ext_fns! {
        ftCommon_8007D5D4(f: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { let _ = crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftGetTemplateKind(f: &Fighter) -> i32 = crate::ft0892::template_kind;
        efAsync_Spawn(g: G, queue_head: *mut core::ffi::c_void, kind: u32, gfx: u32, jobj: *mut HSD_JObj) = |_, _, _, _, _| todo!("TODO(port) efAsync_Spawn (efasync.c)");
        ft_PlaySFX(fp: &mut Fighter, id: i32, vol: u8, pan: u8) = |fp: &mut Fighter, id: i32, vol: u8, pan: u8| { unsafe { let _ = crate::ft0877::play_sfx(fp, id as _, vol as _, pan as _); } };
        ftCommon_8007EBAC(f: &mut Fighter, a: u32, b: u32) = crate::ftcommon2::ftcommon_8007ebac;
        ftColl_8007B62C(g: G, n: i32) = |g: G, n: i32| { unsafe { let _ = crate::tu::ftcoll::ftcoll_8007b62c(g as _, n as _); } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCo_80098D90(g: G) = crate::tu::ftCo_ShieldBreakFall::ftCo_80098D90;
        ft_80084EEC(g: G) = crate::ft084e::ft_80084eec;
        ft_80082C74(g: G, cb: fn(G)) = crate::tu::ft_081b::ft_80082c74;
    }
}

/// `ftCo_SpawnEf` (ftCommon/inlines.h).
fn spawn_ef(g: G, joint: *mut HSD_JObj, arg2: u32, arg3: u32) {
    let f = fp(g);
    ext::efAsync_Spawn(g, &mut f.x60C as *mut *mut core::ffi::c_void as *mut core::ffi::c_void, arg2, arg3, joint);
}

pub fn ftCo_80098B20(g: G) {
    melee_core::ct!("ftCo_80098B20");
    ext::ftCommon_8007D5D4(fp(g));
    ext::Fighter_ChangeMotionState(g, 205, 64, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    let f = fp(g);
    f.self_vel.x = 0.0;
    f.self_vel.y = f.co_attrs.shield_break_initial_velocity;
    if ext::ftGetTemplateKind(f) == Ft_Kind_Purin as i32 {
        f.set_x2222_b3(1);
    }
    let joint = unsafe { (*f.parts.add((*(*f.ft_data).x8).x11 as usize)).joint };
    spawn_ef(g, joint, 0, 1051);
    ext::ft_PlaySFX(fp(g), 130, 127, 64);
    ext::ftCommon_8007EBAC(fp(g), 24, 0);
    ext::ftColl_8007B62C(g, 2);
}

pub fn ftCo_ShieldBreakFly_Anim(g: G) {
    melee_core::ct!("ftCo_ShieldBreakFly_Anim");
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCo_80098D90(g);
    }
}

pub fn ftCo_ShieldBreakFly_IASA(_g: G) { melee_core::ct!("ftCo_ShieldBreakFly_IASA");}

pub fn ftCo_ShieldBreakFly_Phys(g: G) {
    melee_core::ct!("ftCo_ShieldBreakFly_Phys");
    ext::ft_80084EEC(g);
}

pub fn ftCo_ShieldBreakFly_Coll(g: G) {
    melee_core::ct!("ftCo_ShieldBreakFly_Coll");
    ext::ft_80082C74(g, crate::tu::ftCo_ShieldBreakDown::ftCo_80098E3C);
}

pub fn ftCo_80098C9C(g: G) {
    melee_core::ct!("ftCo_80098C9C");
    ftCo_80098B20(g);
}
