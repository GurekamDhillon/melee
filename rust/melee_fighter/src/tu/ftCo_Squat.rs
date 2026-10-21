#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_Squat.c (crouch entry and the Squat state).
//! No fused sites. Calls out of the TU go through `ext` (unported ones panic until rewired;
//! ported ones default to the port). Checked against the C (whole TU, shipping contraction) by
//! tests/tu_ftCo_Squat.rs.
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
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |_, _, _, _, _, _, _| todo!("TODO(port) Fighter_ChangeMotionState (fighter.c)");
        ftAnim_8006EBA4(g: G) = |_| todo!("TODO(port) ftAnim_8006EBA4 (ftanim.c)");
        ftCommon_8007EFC0(f: &mut Fighter, val: u32) = |_, _| todo!("TODO(port) ftCommon_8007EFC0 (ftcommon.c)");
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCo_800D638C(g: G) = crate::tu::ftCo_SquatWait::ftCo_800D638C;
        mpColl_IsOnPlatform(coll: &mut CollData) -> bool = |_| todo!("TODO(port) mpColl_IsOnPlatform (mpcoll.c)");
        ftCo_8009A228(g: G) = |_| todo!("TODO(port) ftCo_8009A228 (ftCo_Pass.c)");
        ftCo_SpecialS_CheckInput(g: G) -> bool = crate::tu::ftCo_SpecialS::ftCo_SpecialS_CheckInput;
        ftCo_Attack100_CheckInput(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_Attack100_CheckInput;
        ftCo_800D6824(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_800D6824;
        ftCo_800D68C0(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_800D68C0;
        ftCo_Catch_CheckInput(g: G) -> bool = crate::tu::ftCo_Catch::ftCo_Catch_CheckInput;
        ftCo_AttackS4_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackS4::ftCo_AttackS4_CheckInput;
        ftCo_AttackHi4_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackHi4::ftCo_AttackHi4_CheckInput;
        ftCo_AttackLw4_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackLw4::ftCo_AttackLw4_CheckInput;
        ftCo_AttackS3_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackS3::ftCo_AttackS3_CheckInput;
        ftCo_AttackHi3_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackHi3::ftCo_AttackHi3_CheckInput;
        ftCo_AttackLw3_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackLw3::ftCo_AttackLw3_CheckInput;
        ftCo_Attack1_CheckInput(g: G) -> bool = crate::tu::ftCo_Attack1::ftCo_Attack1_CheckInput;
        ftCo_80091A4C(g: G) -> bool = |_| todo!("TODO(port) ftCo_80091A4C (ftCo_Guard.c)");
        ftCo_800DE9D8(g: G) -> bool = crate::tu::ftCo_AppealS::ftCo_800DE9D8;
        ftCo_Jump_CheckInput(g: G) -> bool = crate::tu::ftCo_Jump::ftCo_Jump_CheckInput;
        ftCo_80099F9C(g: G) -> bool = |_| todo!("TODO(port) ftCo_80099F9C (ftCo_Pass.c)");
        ft_80084F3C(g: G) = |_| todo!("TODO(port) ft_80084F3C (ft_084E.c)");
        ft_80083F88(g: G) = |_| todo!("TODO(port) ft_80083F88 (ft_081B.c)");
    }
}

pub fn ftCo_Squat_CheckInput(g: G) -> bool {
    fp(g).input.lstick[0].y < -cd().x90
}

pub fn fn_800D5F84(g: G) -> bool {
    ftCo_Squat_CheckInput(g)
}

pub fn ftCo_800D5FB0(g: G) -> bool {
    if ftCo_Squat_CheckInput(g) {
        ftCo_Squat_Enter(g);
        return true;
    }
    false
}

/// static `ftCo_Squat_Enter`.
pub fn ftCo_Squat_Enter(g: G) {
    ext::Fighter_ChangeMotionState(g, ftCo_MS_Squat as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    let f = fp(g);
    f.mv.co.squat.x0 = 0;
    ext::ftCommon_8007EFC0(f, cd().x5F0);
}

pub fn ftCo_Squat_Anim(g: G) {
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCo_800D638C(g);
    }
}

/// static inline `ftCo_Squat_IASA_inline`.
fn ftCo_Squat_IASA_inline(g: G) -> bool {
    let f = fp(g);
    unsafe {
        if f.mv.co.squat.x0 != 0 && f.mv.co.squat.x4 != 0.0 {
            f.mv.co.squat.x4 -= 1.0;
            if f.mv.co.squat.x4 == 0.0 && ext::mpColl_IsOnPlatform(&mut f.coll_data) {
                ext::ftCo_8009A228(g);
                return true;
            }
        }
    }
    false
}

pub fn ftCo_Squat_IASA(g: G) {
    let _ = ext::ftCo_SpecialS_CheckInput(g)
        || ext::ftCo_Attack100_CheckInput(g)
        || ext::ftCo_800D6824(g)
        || ext::ftCo_800D68C0(g)
        || ext::ftCo_Catch_CheckInput(g)
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
        || ftCo_Squat_IASA_inline(g);
}

pub fn ftCo_Squat_Phys(g: G) {
    ext::ft_80084F3C(g);
}

pub fn ftCo_Squat_Coll(g: G) {
    ext::ft_80083F88(g);
}
