#![allow(non_snake_case, unused_unsafe)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_Pass.c (TARGET_PC build: every function of
//! the TU, including the UCF 0.84 shield-drop copy of the platform check inlined into
//! ftCo_8009A080). Calls out of it go through `ext`. Checked against the C by tests/tu_ftCo_Pass.rs.
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
        mpColl_IsOnPlatform(coll: &mut CollData) -> bool = |coll: &mut CollData| -> bool { unsafe { (melee_world::tu::mpcoll::mpColl_IsOnPlatform(coll as *mut _ as *mut _) != 0 as _) } };
        // rp3 mpUpdateFloorSkip
        mpUpdateFloorSkip(coll: &mut CollData) = |coll: &mut CollData| unsafe { melee_world::tu::mpcoll::mpUpdateFloorSkip(coll as _); };
        // rp3-end
        ftCo_800C5240(g: G) -> bool = crate::tu::ftCo_HammerWait::ftCo_800C5240;
        ftCo_800C60C8(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftco_0c60::ftco_800c60c8(g as _) as _ } };
        Replay_UcfVersion(port: i32) -> i32 = |_| todo!("TODO(port) Replay_UcfVersion (pc replay)");
        ftUcf_FlickCounter(port: i32) -> i32 = |_| todo!("TODO(port) ftUcf_FlickCounter (fighter.c)");
        ftCommon_8007D5D4(fp: &mut Fighter) = |fp: &mut Fighter| { unsafe { let _ = crate::ftcommon2::ftcommon_8007d5d4(fp as _); } };
        ftCommon_ClampAirDrift(fp: &mut Fighter) = |_| todo!("TODO(port) ftCommon_ClampAirDrift (ftcommon.c)");
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCo_Fall_Enter(g: G) = crate::tu::ftCo_Fall::ftCo_Fall_Enter;
        ftCo_SpecialAir_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_SpecialAir::ftCo_SpecialAir_CheckInput(g as _) as _ } };
        ftCo_80095328(g: G, b: *mut bool) -> bool = |g: G, b: *mut bool| -> bool { unsafe { crate::tu::ftCo_ItemThrow::ftCo_80095328(g as _, b as _) as _ } };
        ftCo_800D7100(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_800D7100;
        ftCo_800C3B10(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftco_aircatch::ftco_800c3b10(g as _) as _ } };
        ftCo_80099A58(g: G) -> bool = crate::tu::ftCo_EscapeAir::ftCo_80099A58;
        ftCo_AttackAir_CheckItemThrowInput(g: G) -> bool = crate::tu::ftCo_AttackAir::ftCo_AttackAir_CheckItemThrowInput;
        ftCo_800D705C(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_800D705C;
        ftCo_800CB870(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_JumpAerial::ftCo_800CB870(g as _) as _ } };
        ft_80084DB0(g: G) = crate::ft084e::ft_80084db0;
        ft_80082F28(g: G) = crate::tu::ft_081b::ft_80082f28;
    }
}

pub fn ftCo_80099F1C(g: G) -> bool {
    let f = fp(g);
    f.input.lstick[0].y <= -cd().x464
        && (f.active_timer.lstick.y as f32) < cd().x468
        && ext::mpColl_IsOnPlatform(&mut fp(g).coll_data)
}

pub fn ftCo_80099F9C(g: G) -> bool {
    if ext::ftCo_800C5240(g) {
        return ext::ftCo_800C60C8(g);
    }
    if unsafe { fp(g).mv.co.pass.x0 } == 0 && ftCo_80099F1C(g) {
        let f = fp(g);
        unsafe {
            f.mv.co.pass.x0 = 1;
            f.mv.co.pass.x4 = cd().x470;
        }
        return true;
    }
    false
}

/// TARGET_PC `ftCo_8009A080_Check`: the inlined copy of ftCo_80099F1C with UCF 0.84 Shield
/// Drop Extended.
fn ftCo_8009A080_Check(g: G) -> bool {
    let f = fp(g);
    let mut down = f.input.lstick[0].y <= -cd().x464;
    if !down && ext::Replay_UcfVersion(f.x618_player_id as i32) == 84 {
        down = ext::ftUcf_FlickCounter(fp(g).x618_player_id as i32) > 1;
    }
    down && (fp(g).active_timer.lstick.y as f32) < cd().x468 && ext::mpColl_IsOnPlatform(&mut fp(g).coll_data)
}

pub fn ftCo_8009A080(g: G) -> bool {
    if fp(g).input.held_buttons[0] & HSD_PAD_LR != 0 && ftCo_8009A080_Check(g) {
        ftCo_8009A228(g);
        return true;
    }
    false
}

pub fn ftCo_8009A134(g: G) -> bool {
    if ext::mpColl_IsOnPlatform(&mut fp(g).coll_data) {
        ext::mpUpdateFloorSkip(&mut fp(g).coll_data);
        return true;
    }
    false
}

pub fn ftCo_8009A184(g: G, msid: i32, mf: u32, anim_start: f32) {
    ext::ftCommon_8007D5D4(fp(g));
    ext::ftCommon_ClampAirDrift(fp(g));
    fp(g).self_vel.y = cd().x46C;
    ext::Fighter_ChangeMotionState(g, msid, mf, anim_start, 1.0, 0.0, core::ptr::null_mut());
    ext::mpUpdateFloorSkip(&mut fp(g).coll_data);
    fp(g).active_timer.lstick.y = 0xFE;
}

pub fn ftCo_8009A228(g: G) {
    ftCo_8009A184(g, ftCo_MS_Pass as i32, Ft_MF_None as u32, 0.0);
}

pub fn ftCo_Pass_Anim(g: G) {
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCo_Fall_Enter(g);
    }
}

pub fn ftCo_Pass_IASA(g: G) {
    let _ = ext::ftCo_SpecialAir_CheckInput(g)
        || ext::ftCo_80095328(g, core::ptr::null_mut())
        || ext::ftCo_800D7100(g)
        || ext::ftCo_800C3B10(g)
        || ext::ftCo_80099A58(g)
        || ext::ftCo_AttackAir_CheckItemThrowInput(g)
        || ext::ftCo_800D705C(g)
        || ext::ftCo_800CB870(g);
}

pub fn ftCo_Pass_Phys(g: G) {
    ext::ft_80084DB0(g);
}

pub fn ftCo_Pass_Coll(g: G) {
    ext::ft_80082F28(g);
}
