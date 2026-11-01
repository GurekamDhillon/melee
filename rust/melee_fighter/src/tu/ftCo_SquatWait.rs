#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_SquatWait.c (the crouch idle state). No fused
//! sites. Calls out of the TU go through `ext` (unported ones panic until rewired; ported ones
//! default to the port). Checked against the C (whole TU, shipping contraction) by
//! tests/tu_ftCo_SquatWait.rs.
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
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCo_8008A698(f: &Fighter) -> bool = crate::tu::ftwaitanim::ftCo_8008A698;
        // `ftData_80085FD4(fp, msid)->x8` (only that word is read here).
        ftData_80085FD4(f: &mut Fighter, msid: i32) -> usize = crate::tu::ftdata::ftData_80085FD4_x8;
        ftCo_8008A6D8(g: G, anim_id: i32) = crate::tu::ftwaitanim::ftCo_8008A6D8;
        ftCommon_8007EFC0(f: &mut Fighter, val: u32) = |_, _| todo!("ftCommon_8007EFC0: installed by melee_link");
        ftLk_AttackAir_800EB3BC(g: G) = |_| todo!("TODO(port) ftLk_AttackAir_800EB3BC (ftlinkattackair.c)");
        ftCl_Init_8014919C(g: G) = |_| todo!("TODO(port) ftCl_Init_8014919C (ftclink.c)");
        ftCo_8008A7A8(g: G, w: *mut WaitStruct) = |g, w| crate::tu::ftwaitanim::ftCo_8008A7A8(g, w);
        ftCo_800D68C0(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_800D68C0;
        ftCo_Attack100_CheckInput(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_Attack100_CheckInput;
        ftCo_AttackS4_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackS4::ftCo_AttackS4_CheckInput;
        ftCo_AttackHi4_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackHi4::ftCo_AttackHi4_CheckInput;
        ftCo_AttackLw4_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackLw4::ftCo_AttackLw4_CheckInput;
        ftCo_AttackS3_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackS3::ftCo_AttackS3_CheckInput;
        ftCo_AttackHi3_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackHi3::ftCo_AttackHi3_CheckInput;
        ftCo_AttackLw3_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackLw3::ftCo_AttackLw3_CheckInput;
        ftCo_Attack1_CheckInput(g: G) -> bool = crate::tu::ftCo_Attack1::ftCo_Attack1_CheckInput;
        ftCo_80091A4C(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Guard::ftCo_80091A4C(g as _) as _ } };
        ftCo_800DE9D8(g: G) -> bool = crate::tu::ftCo_AppealS::ftCo_800DE9D8;
        ftCo_Jump_CheckInput(g: G) -> bool = crate::tu::ftCo_Jump::ftCo_Jump_CheckInput;
        ftCo_80099F9C(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Pass::ftCo_80099F9C(g as _) as _ } };
        ftCo_Dash_CheckInput(g: G) -> bool = crate::tu::ftCo_Dash::ftCo_Dash_CheckInput;
        ftCo_SquatRv_CheckInput(g: G) -> bool = crate::tu::ftCo_SquatRv::ftCo_SquatRv_CheckInput;
        ft_80084F3C(g: G) = |_| todo!("ft_80084F3C: installed by melee_link");
        ftColl_8007AEE0(g: G) = crate::ft0df::ftcoll_8007aee0;
        ft_80083F88(g: G) = |g: G| { unsafe { crate::tu::ft_081b::ft_80083f88(g as _); } };
    }
}

pub fn ftCo_SquatWait_CheckInput(g: G) -> bool {
    melee_core::ct!("ftCo_SquatWait_CheckInput");
    if fp(g).input.lstick[0].y < -cd().x90 {
        fn_800D62C4(g);
        return true;
    }
    false
}

/// static inline `ftCo_SquatWait_Enter_inline`.
fn ftCo_SquatWait_Enter_inline(g: G, flags: u32, arg2: bool) {
    ext::Fighter_ChangeMotionState(g, ftCo_MS_SquatWait as i32, flags, 0.0, 1.0, 0.0, core::ptr::null_mut());
    if ext::ftCo_8008A698(fp(g)) && ext::ftData_80085FD4(fp(g), ftCo_SM_SquatWaitItem as i32) != 0 {
        ext::ftCo_8008A6D8(g, ftCo_SM_SquatWaitItem as i32);
    }
    if arg2 {
        ext::ftCommon_8007EFC0(fp(g), cd().x5F0);
    }
    let k = fp(g).kind as i32;
    if k == Ft_Kind_Link as i32 {
        ext::ftLk_AttackAir_800EB3BC(g);
    } else if k == Ft_Kind_CLink as i32 {
        ext::ftCl_Init_8014919C(g);
    }
}

/// static `fn_800D62C4`.
pub fn fn_800D62C4(g: G) {
    melee_core::ct!("fn_800D62C4");
    ftCo_SquatWait_Enter_inline(g, Ft_MF_None as u32, true);
}

pub fn ftCo_800D638C(g: G) {
    melee_core::ct!("ftCo_800D638C");
    ftCo_SquatWait_Enter_inline(g, Ft_MF_SkipNametagVis as u32, false);
}

pub fn ftCo_SquatWait_Anim(g: G) {
    melee_core::ct!("ftCo_SquatWait_Anim");
    let x28 = unsafe { (*fp(g).ft_data).x28 };
    ext::ftCo_8008A7A8(g, x28);
}

pub fn ftCo_SquatWait_IASA(g: G) {
    melee_core::ct!("ftCo_SquatWait_IASA");
    let _ = ext::ftCo_800D68C0(g)
        || ext::ftCo_Attack100_CheckInput(g)
        || ext::ftCo_AttackS4_CheckInput(g)
        || ext::ftCo_AttackHi4_CheckInput(g)
        || ext::ftCo_AttackLw4_CheckInput(g)
        || ext::ftCo_AttackS3_CheckInput(g)
        || ext::ftCo_AttackHi3_CheckInput(g)
        || ext::ftCo_AttackLw3_CheckInput(g)
        || ext::ftCo_Attack1_CheckInput(g)
        || ext::ftCo_80091A4C(g)
        || ext::ftCo_800DE9D8(g)
        || ext::ftCo_Jump_CheckInput(g)
        || ext::ftCo_80099F9C(g)
        || ext::ftCo_Dash_CheckInput(g)
        || ext::ftCo_SquatRv_CheckInput(g);
}

pub fn ftCo_SquatWait_Phys(g: G) {
    melee_core::ct!("ftCo_SquatWait_Phys");
    ext::ft_80084F3C(g);
    ext::ftColl_8007AEE0(g);
}

pub fn ftCo_SquatWait_Coll(g: G) {
    melee_core::ct!("ftCo_SquatWait_Coll");
    ext::ft_80083F88(g);
}
