#![allow(non_snake_case, unused_unsafe)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_Ottotto.c (every function of the TU). Calls
//! out of it go through `ext`. Checked against the C (whole TU, shipping contraction) by
//! tests/tu_ftCo_Ottotto.rs.
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
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCo_SpecialS_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_SpecialS::ftCo_SpecialS_CheckInput(g as _) as _ } };
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
        ftCo_80091A4C(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Guard::ftCo_80091A4C(g as _) as _ } };
        ftCo_800DE9D8(g: G) -> bool = crate::tu::ftCo_AppealS::ftCo_800DE9D8;
        ftCo_Jump_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Jump::ftCo_Jump_CheckInput(g as _) as _ } };
        ftCo_Dash_CheckInput(g: G) -> bool = crate::tu::ftCo_Dash::ftCo_Dash_CheckInput;
        ftCo_800D5FB0(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Squat::ftCo_800D5FB0(g as _) as _ } };
        ftCo_Turn_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Turn::ftCo_Turn_CheckInput(g as _) as _ } };
        ftCo_Walk_CheckInput_Ottotto(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Walk::ftCo_Walk_CheckInput_Ottotto(g as _) as _ } };
        ft_800827A0(g: G) -> bool = crate::tu::ft_081b::ft_800827a0;
        ftCo_Fall_Enter(g: G) = crate::tu::ftCo_Fall::ftCo_Fall_Enter;
        // rp3 mpFloorGetRight
        mpFloorGetRight(line_id: i32, pos: &mut Vec3) = |line_id: i32, pos: &mut Vec3| unsafe { melee_world::tu::mplib::mpFloorGetRight(line_id, pos as _); };
        // rp3-end
        // rp3 mpFloorGetLeft
        mpFloorGetLeft(line_id: i32, pos: &mut Vec3) = |line_id: i32, pos: &mut Vec3| unsafe { melee_world::tu::mplib::mpFloorGetLeft(line_id, pos as _); };
        // rp3-end
        ft_8008A2BC(g: G) = crate::hooks::ft_8008a2bc;
        ft_80088770(fp: &mut Fighter) = |fp: &mut Fighter| { unsafe { let _ = crate::ft0881::ft_80088770(fp as _); } };
        ft_800887CC(fp: &mut Fighter) = |fp: &mut Fighter| { unsafe { let _ = crate::ft0881::ft_800887cc(fp as _); } };
        ft_80088328(fp: &mut Fighter, sfx: i32, vol: u8, pan: u8) = |fp: &mut Fighter, sfx: i32, vol: u8, pan: u8| { unsafe { let _ = crate::ft0881::ft_80088328(fp as _, sfx as _, vol as _, pan as _); } };
    }
}

pub fn ftCo_8009A3C8(g: G) -> bool {
    melee_core::ct!("ftCo_8009A3C8");
    let f = fp(g);
    if f.coll_data.env_flags & Collide_Edge != 0 && f.is_sandbag() == 0 {
        ftCo_8009A410(g);
        return true;
    }
    false
}

pub fn ftCo_8009A410(g: G) {
    melee_core::ct!("ftCo_8009A410");
    ext::Fighter_ChangeMotionState(g, ftCo_MS_Ottotto as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    f.self_vel.z = 0.0;
    f.self_vel.y = 0.0;
    f.self_vel.x = 0.0;
    f.gr_vel = 0.0;
}

pub fn ftCo_Ottotto_Anim(g: G) {
    melee_core::ct!("ftCo_Ottotto_Anim");
    if !ext::ftAnim_IsFramesRemaining(g) {
        ftCo_8009A6B8(g);
    }
}

pub fn ftCo_Ottotto_IASA(g: G) {
    melee_core::ct!("ftCo_Ottotto_IASA");
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
        || ext::ftCo_Dash_CheckInput(g)
        || ext::ftCo_800D5FB0(g)
        || ext::ftCo_Turn_CheckInput(g)
        || ext::ftCo_Walk_CheckInput_Ottotto(g);
}

pub fn ftCo_Ottotto_Phys(_g: G) { melee_core::ct!("ftCo_Ottotto_Phys");}

/// Shared body of `ftCo_Ottotto_Coll` / `ftCo_OttottoWait_Coll` (identical math).
fn coll(g: G) {
    if !ext::ft_800827A0(g) {
        ext::ftCo_Fall_Enter(g);
        return;
    }
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let f = fp(g);
    let idx = f.coll_data.floor.index;
    if f.facing_dir > 0.0 {
        ext::mpFloorGetRight(idx, &mut pos);
    } else {
        ext::mpFloorGetLeft(idx, &mut pos);
    }
    let f = fp(g);
    let mut x = f.cur_pos.x - pos.x;
    x = if x < 0.0 { -x } else { x };
    if x > cd().x478 + cd().x47C {
        ext::ft_8008A2BC(g);
    }
}

pub fn ftCo_Ottotto_Coll(g: G) {
    melee_core::ct!("ftCo_Ottotto_Coll");
    coll(g);
}

pub fn ftCo_8009A6B8(g: G) {
    melee_core::ct!("ftCo_8009A6B8");
    ext::Fighter_ChangeMotionState(g, ftCo_MS_OttottoWait as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ft_80088770(fp(g));
    ext::ft_800887CC(fp(g));
    let f = fp(g);
    let sfx = unsafe { (*(*f.ft_data).x4C_sfx).x18 };
    ext::ft_80088328(f, sfx, 127, 64);
}

pub fn ftCo_OttottoWait_Anim(_g: G) { melee_core::ct!("ftCo_OttottoWait_Anim");}

pub fn ftCo_OttottoWait_IASA(g: G) {
    melee_core::ct!("ftCo_OttottoWait_IASA");
    ftCo_Ottotto_IASA(g);
}

pub fn ftCo_OttottoWait_Phys(_g: G) { melee_core::ct!("ftCo_OttottoWait_Phys");}

pub fn ftCo_OttottoWait_Coll(g: G) {
    melee_core::ct!("ftCo_OttottoWait_Coll");
    coll(g);
}
