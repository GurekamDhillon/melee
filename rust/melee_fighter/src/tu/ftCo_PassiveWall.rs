#![allow(non_snake_case, unused_unsafe)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_PassiveWall.c (every function of the TU). Calls out of it
//! go through `ext`. Checked against the C (whole TU, shipping contraction) by tests/tu_ftCo_PassiveWall.rs.
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
        ftCommon_8007E2FC(g: G) = |_| todo!("ftCommon_8007E2FC: installed by melee_link");
        ftKb_SpecialN_800F1F1C(g: G, ofs: &mut Vec3) = |_, _| todo!("TODO(port) ftKb_SpecialN_800F1F1C (ftKb_SpecialN.c)");
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        efAsync_Spawn(g: G, queue: *mut core::ffi::c_void, kind: u32, gfx: u32, jobj: *mut HSD_JObj, pos: *mut Vec3) = |a0, a1, a2, a3, a4, a5| unsafe { let _ = melee_world::tu::efn::efasync::efAsync_Spawn(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, &[a5 as *const core::ffi::c_void]); };
        ft_80081F2C(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ft_081b::ft_80081f2c(g as _) as _ } };
        ft_800881D8(fp: &mut Fighter, sfx: i32, vol: u8, pan: u8) = |_, _, _, _| todo!("ft_800881D8: installed by melee_link");
        ftCommon_8007EBAC(fp: &mut Fighter, a: u32, b: u32) = |_, _, _| todo!("ftCommon_8007EBAC: installed by melee_link");
        ft_PlaySFX(fp: &mut Fighter, id: i32, vol: u8, pan: u8) = |_, _, _, _| todo!("ft_PlaySFX: installed by melee_link");
        ftCo_800BFFD0(fp: &mut Fighter, a: i32, b: bool) -> bool = |fp: &mut Fighter, a: i32, b: bool| -> bool { unsafe { crate::tu::ftcolanim::ftco_800bffd0(fp as _, a as _, b as _) as _ } };
        ftColl_8007B760(g: G, v: i32) = |g: G, v: i32| { unsafe { crate::tu::ftcoll::ftcoll_8007b760(g as _, v as _); } };
        ftAnim_SetAnimRate(g: G, r: f32) = |g: G, r: f32| { unsafe { crate::tu::ftanim::ftAnim_SetAnimRate(g as _, r as _); } };
        powf(x: f32, y: f32) -> f32 = |a0, a1| melee_runtime::tu::lb_00CE::powf(a0, a1);
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
        ftCo_800CEE70(g: G) -> bool = crate::tu::ftCo_ItemParasolOpen::ftCo_800CEE70;
        ftCommon_CheckFallFast(fp: &mut Fighter) -> bool = crate::ftcommon::check_fall_fast;
        ftCommon_FallFast(fp: &mut Fighter) = crate::ftcommon::fall_fast;
        ftCommon_Fall(fp: &mut Fighter, grav: f32, term: f32) = |_, _, _| todo!("ftCommon_Fall: installed by melee_link");
        ftCommon_CalcSelfAccel_AccelToVelClamped(fp: &mut Fighter, a: f32, b: f32, c: f32) = |a0, a1, a2, a3| crate::tu::ftcommon::ftCommon_CalcSelfAccel_AccelToVelClamped(a0, a1, a2, a3);
        ft_80083318(g: G, a: fn(G, i32) -> bool, b: fn(G)) = crate::tu::ft_081b::ft_80083318;
        ft_800831CC(g: G, a: fn(G, i32) -> bool, b: fn(G)) = crate::tu::ft_081b::ft_800831cc;
        ftCo_80096CC8(g: G, line_id: i32) -> bool = |g: G, line_id: i32| -> bool { unsafe { crate::tu::ftCo_FallSpecial::ftCo_80096CC8(g as _, line_id as _) as _ } };
        ft_80082B1C(g: G) = crate::tu::ft_081b::ft_80082b1c;
    }
}

fn cb_80096CC8(g: G, line_id: i32) -> bool {
    ext::ftCo_80096CC8(g, line_id)
}
fn cb_80082B1C(g: G) {
    ext::ft_80082B1C(g)
}

pub fn ftCo_800C1D38(g: G) -> bool {
    melee_core::ct!("ftCo_800C1D38");
    let ef = fp(g).coll_data.env_flags;
    if (ef & Collide_RightWallHug != 0 || ef & Collide_LeftWallHug != 0) && ext::ftCo_800986B0(g) {
        let msid = if ftCo_800C1E0C(fp(g)) { ftCo_MS_PassiveWallJump } else { ftCo_MS_PassiveWall } as i32;
        let timer = cd().x760;
        if fp(g).coll_data.env_flags & Collide_RightWallHug != 0 {
            ftCo_800C1E64(g, msid, timer, 0, -1.0);
        } else {
            ftCo_800C1E64(g, msid, timer, 0, 1.0);
        }
        return true;
    }
    false
}

pub fn ftCo_800C1E0C(f: &Fighter) -> bool {
    melee_core::ct!("ftCo_800C1E0C");
    (f.x67E as f32) < cd().x250 || f.input.lstick[0].y >= cd().tap_jump_threshold
}

pub fn ftCo_800C1E64(g: G, msid: i32, timer: i32, vel_y_exponent: i32, facing_dir: f32) {
    melee_core::ct!("ftCo_800C1E64");
    let f = fp(g);
    f.cmd_vars[0] = 1;
    f.facing_dir = -facing_dir;
    ext::ftCommon_8007E2FC(g);
    let f = fp(g);
    let c = &f.coll_data;
    let mut ef_offset = if c.env_flags & Collide_RightWallHug != 0 {
        Vec3 { x: c.ecb.left.x, y: c.ecb.left.y, z: 0.0 }
    } else {
        Vec3 { x: c.ecb.right.x, y: c.ecb.right.y, z: 0.0 }
    };
    ext::ftKb_SpecialN_800F1F1C(g, &mut ef_offset);
    ext::Fighter_ChangeMotionState(g, msid, Ft_MF_None as u32, 0.0, if timer != 0 { 0.0 } else { 1.0 }, 0.0, core::ptr::null_mut());
    let f = fp(g);
    f.active_timer.lstick.x = 0xFE;
    f.active_timer.lstick.y = 0xFE;
    unsafe {
        f.mv.co.passivewall.timer = timer;
        f.mv.co.passivewall.x4 = 0;
        f.mv.co.passivewall.x8 = 0;
        f.mv.co.passivewall.vel_y_exponent = vel_y_exponent;
    }
    let pos_x_offset = f.cur_pos.x + ef_offset.x;
    let mut ef_pos = Vec3 { x: pos_x_offset, y: f.cur_pos.y + ef_offset.y, z: f.cur_pos.z };
    ext::efAsync_Spawn(g, &mut f.x60C as *mut _ as *mut core::ffi::c_void, 2, 0x41D, core::ptr::null_mut(), &mut ef_pos);
    let f = fp(g);
    // `-(z * -facing - pos_x_offset)`: clang contracts the inner a*b-c to fmuladd(a, b, -c) (the
    // negated operand matters for NaN sign), lowered as gk_fmadds.
    f.cur_pos.x = -melee_core::fp::fmadds(f.x68C_transNPos.z, -f.facing_dir, -pos_x_offset);
    ext::ft_80081F2C(g);
    let f = fp(g);
    let sfx = unsafe { (*(*f.ft_data).x4C_sfx).x24 };
    ext::ft_800881D8(f, sfx, 0x7F, 0x40);
    ext::ftCommon_8007EBAC(fp(g), 12, 0);
    if timer == 0 {
        ext::ft_PlaySFX(fp(g), 3, 0x7F, 0x40);
    }
    ext::ftCo_800BFFD0(fp(g), 120, false);
    ext::ftColl_8007B760(g, cd().x764);
}

pub fn ftCo_PassiveWall_Anim(g: G) {
    melee_core::ct!("ftCo_PassiveWall_Anim");
    let f = fp(g);
    if unsafe { f.mv.co.passivewall.timer } != 0 {
        let t = unsafe {
            f.mv.co.passivewall.timer -= 1;
            f.mv.co.passivewall.timer
        };
        if t == 0 {
            ext::ft_PlaySFX(f, 8, 127, 64);
            if unsafe { fp(g).mv.co.passivewall.x8 } != 0 {
                // inlineA0
                let frame = fp(g).cur_anim_frame;
                ext::Fighter_ChangeMotionState(g, ftCo_MS_PassiveWallJump as i32, Ft_MF_None as u32, frame, 1.0, 0.0, core::ptr::null_mut());
                let f = fp(g);
                f.active_timer.lstick.y = 0xFE;
                unsafe { f.mv.co.passivewall.timer = 0 };
            } else {
                ext::ftAnim_SetAnimRate(g, 1.0);
            }
            let f = fp(g);
            if f.motion_id == ftCo_MS_PassiveWall as _ {
                f.self_vel.x = f.facing_dir * f.co_attrs.passivewall_vel_x;
            } else {
                f.self_vel.x = f.facing_dir * f.co_attrs.wall_jump_horizontal_velocity;
                f.self_vel.y = f.co_attrs.wall_jump_vertical_velocity;
                if f.x1969_walljumpUsed != 0 {
                    let e = unsafe { f.mv.co.passivewall.vel_y_exponent } as f32;
                    let p = ext::powf(cd().passive_wall_vel_y_base, e);
                    fp(g).self_vel.y *= p;
                }
            }
        }
    }
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCo_Fall_Enter(g);
    }
}

pub fn ftCo_PassiveWall_IASA(g: G) {
    melee_core::ct!("ftCo_PassiveWall_IASA");
    if unsafe { fp(g).mv.co.passivewall.timer } == 0 {
        if ext::ftCo_SpecialAir_CheckInput(g)
            || ext::ftCo_80095328(g, core::ptr::null_mut())
            || ext::ftCo_800D7100(g)
            || ext::ftCo_800C3B10(g)
            || ext::ftCo_80099A58(g)
            || ext::ftCo_AttackAir_CheckItemThrowInput(g)
            || ext::ftCo_800D705C(g)
            || ext::ftCo_800CB870(g)
            || fp(g).cmd_vars[0] == 0
        {
            return;
        }
        ext::ftCo_800CEE70(g);
    } else {
        if !ftCo_800C1E0C(fp(g)) {
            return;
        }
        unsafe { fp(g).mv.co.passivewall.x8 = 1 };
    }
}

pub fn ftCo_PassiveWall_Phys(g: G) {
    melee_core::ct!("ftCo_PassiveWall_Phys");
    if unsafe { fp(g).mv.co.passivewall.timer } == 0 {
        ext::ftCommon_CheckFallFast(fp(g));
        let f = fp(g);
        if f.fall_fast() != 0 {
            ext::ftCommon_FallFast(f);
        } else {
            let (gr, tv) = (f.co_attrs.gravity, f.co_attrs.terminal_velocity);
            ext::ftCommon_Fall(f, gr, tv);
        }
        let f = fp(g);
        let af = f.co_attrs.aerial_friction;
        ext::ftCommon_CalcSelfAccel_AccelToVelClamped(f, 0.0, 0.0, af);
    }
}

pub fn ftCo_PassiveWall_Coll(g: G) {
    melee_core::ct!("ftCo_PassiveWall_Coll");
    if unsafe { fp(g).mv.co.passivewall.timer } != 0 {
        ext::ft_80083318(g, cb_80096CC8, cb_80082B1C);
    } else {
        ext::ft_800831CC(g, cb_80096CC8, cb_80082B1C);
    }
}
