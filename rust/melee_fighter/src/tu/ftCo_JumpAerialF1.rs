#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_JumpAerialF1.c (TARGET_PC build): the
//! multi-jump (Kirby / Jigglypuff / Meta Knight style) air jumps, with the port's Geno bounds.
//! Every function of the TU; calls out go through `ext`. Checked by tests/tu_ftCo_JumpAerialF1.rs.

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
        FighterControls_TapJumpOff(port: i32) -> i32 = |p| crate::tu::ftCo_Jump::ext::FighterControls_TapJumpOff(p);
        ScriptGame_ModJump(slot: i32, field: i32, base: f32) -> f32 = |s, f, b| crate::tu::ftCo_Jump::ext::ScriptGame_ModJump(s, f, b);
        ScriptGame_CapsMaxJumps(fp: &Fighter) -> i32 = |f| crate::tu::ftCo_JumpAerial::ext::ScriptGame_CapsMaxJumps(f);
        Geno_MultiJump(fp: &mut Fighter, first_state: i32, msid: &mut i32, vy: &mut f32) = |_, _, _, _| todo!("TODO(port) Geno_MultiJump (pc/geno)");
        ft_did_jump(fp: &mut Fighter, arg1: bool) -> bool = crate::tu::ftCo_JumpAerial::ft_did_jump;
        ft_800D2D0C(g: G) -> bool = |g| crate::tu::ftCo_JumpAerial::ext::ft_800D2D0C(g);
        ft_800D2E7C(g: G, v: &mut Vec3) = crate::tu::ftCo_ItemScrew::ft_800D2E7C;
        ftCommon_8007D5D4(fp: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        ftCo_800D72A0(fp: &mut Fighter) -> bool = |_| todo!("TODO(port) ftCo_800D72A0 (ftCo_0D72.c)");
        ftCo_800D7268(fp: &mut Fighter) -> i32 = |f| crate::tu::ftCo_Attack100::ftCo_800D7268(f);
        ftCo_800CBAC4(g: G, msid: i32, vel: &Vec3, arg3: bool) = crate::tu::ftCo_JumpAerial::ftCo_800CBAC4;
        ft_800CB6EC(fp: &mut Fighter, arg1: i32) = crate::tu::ftCo_JumpAerial::ft_800CB6EC;
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCo_FallAerial_Enter(g: G) = |g| crate::tu::ftCo_JumpAerial::ext::ftCo_FallAerial_Enter(g);
        ftCo_Fall_Enter(g: G) = |g| crate::tu::ftCo_Jump::ext::ftCo_Fall_Enter(g);
        ftCo_JumpAerial_IASA(g: G) = crate::tu::ftCo_JumpAerial::ftCo_JumpAerial_IASA;
        ft_80084E1C(g: G, threshold: f32, drift: f32, max: f32) = crate::ft084e::ft_80084e1c;
        ft_80082F28(g: G) = crate::tu::ft_081b::ft_80082f28;
    }
}

#[inline(always)]
fn tap_jump_ok(f: &Fighter) -> bool {
    ext::FighterControls_TapJumpOff(f.player_id as i32) == 0
}

/// static inline `ftCo_800D74A4_inline`.
#[inline(always)]
pub fn ftCo_800D74A4_inline(f: &Fighter) -> *mut Fighter_x2D0_t {
    f.x2D0
}

/// `((s32*) fp->x2D0 + idx)->x2C`: the start state, read `idx` words past `x2C`.
#[inline(always)]
unsafe fn start_state(p: *mut Fighter_x2D0_t, idx: i32) -> i32 {
    *((p as *const i32).offset(idx as isize).byte_add(0x2C))
}

pub fn ftCo_800D730C(g: G, arg1: bool) -> bool {
    let f = fp(g);
    let x2d0 = f.x2D0;
    if f.x1968_jumpsUsed as i32 >= ext::ScriptGame_CapsMaxJumps(f) {
        return false;
    }
    if f.motion_id == 0x9B {
        if ext::ft_did_jump(fp(g), arg1) {
            ftCo_800D74A4(g);
            return true;
        }
        return false;
    }
    if f.x1968_jumpsUsed == 1 {
        if ext::ft_did_jump(fp(g), arg1) {
            if ext::ft_800D2D0C(g) {
                let f = fp(g);
                let x = f.input.lstick[0].x * unsafe { (*x2d0).x8 };
                let y = ext::ScriptGame_ModJump(f.player_id as i32, 6, unsafe { (*x2d0).x14[0] });
                let mut v = Vec3 { x, y, z: 0.0 };
                ext::ft_800D2E7C(g, &mut v);
            } else {
                ext::ftCommon_8007D5D4(fp(g));
                ftCo_800D74A4(g);
            }
            return true;
        }
    } else {
        let mut r29 = true;
        let can_jump = (f.x1968_jumpsUsed as i32) < ext::ScriptGame_CapsMaxJumps(f);
        if ext::ftCo_800D72A0(fp(g)) && fp(g).cmd_vars[0] == 0 {
            r29 = false;
        }
        let f = fp(g);
        let mut r4 = true;
        if !(tap_jump_ok(f) && f.input.lstick[0].y >= cd().tap_jump_threshold)
            && f.input.held_buttons[0] & 0xC00 == 0
        {
            r4 = false;
        }
        if can_jump && r29 && r4 {
            ftCo_800D74A4(g);
            return true;
        }
    }
    false
}

pub fn ftCo_800D74A4(g: G) {
    let f = fp(g);
    let p = ftCo_800D74A4_inline(f);
    f.cmd_vars[0] = 0;
    unsafe {
        let mut msid = f.x1968_jumpsUsed as i32 + start_state(f.x2D0, ext::ftCo_800D7268(fp(g)));
        let f = fp(g);
        msid = msid.wrapping_sub(1);
        let mut msid2 = msid;
        let vx = f.input.lstick[0].x * (*p).x8;
        let first = start_state(f.x2D0, ext::ftCo_800D7268(fp(g)));
        let f = fp(g);
        let mut vy;
        {
            let mut jump_row = msid.wrapping_sub(first);
            let mut rows = (*p).x28;
            let state_row = msid2.wrapping_sub(first);
            if rows < 1 || rows > 5 {
                rows = 5;
            }
            if jump_row < 0 {
                jump_row = 0;
            }
            if jump_row >= rows {
                jump_row = rows - 1;
            }
            vy = (*p).x14[jump_row as usize];
            if state_row >= rows {
                msid2 = first + rows - 1;
            }
        }
        {
            let mut geno_msid = msid2;
            ext::Geno_MultiJump(f, first, &mut geno_msid, &mut vy);
            let f = fp(g);
            let mut rows = (*p).x28;
            let limit = ext::ScriptGame_CapsMaxJumps(f);
            let row = f.x1968_jumpsUsed as i32 - 1;
            if rows < 1 || rows > 5 {
                rows = 5;
            }
            if limit != f.co_attrs.max_jumps && row >= rows - 1 {
                geno_msid = first + rows - 1;
                if rows > 1 && f.x1968_jumpsUsed as i32 + 1 < limit {
                    geno_msid = first + rows - 2;
                }
            }
            msid2 = geno_msid;
        }
        let f = fp(g);
        vy = ext::ScriptGame_ModJump(f.player_id as i32, 6, vy);
        let v = Vec3 { x: vx, y: vy, z: 0.0 };
        ext::ftCo_800CBAC4(g, msid2, &v, false);
        let f = fp(g);
        let gp = &mut f.mv.ca.specials.grav as *mut _ as *mut i32;
        if f.input.lstick[0].x * f.facing_dir < -(*p).x4 {
            *gp = (*p).x0;
        } else {
            *gp = 0;
        }
        ext::ft_800CB6EC(f, (*p).x0);
    }
}

pub fn ftCo_JumpAerialF1_Anim(g: G) {
    let f = fp(g);
    let tmp = unsafe { (*f.x2D0).x0 };
    ext::ft_800CB6EC(f, tmp);
    if !ext::ftAnim_IsFramesRemaining(g) {
        let f = fp(g);
        if f.x1968_jumpsUsed as i32 >= ext::ScriptGame_CapsMaxJumps(f) {
            ext::ftCo_FallAerial_Enter(g);
        } else {
            ext::ftCo_Fall_Enter(g);
        }
    }
}

pub fn ftCo_JumpAerialF1_IASA(g: G) {
    ext::ftCo_JumpAerial_IASA(g);
}

pub fn ftCo_JumpAerialF1_Phys(g: G) {
    let f = fp(g);
    let p = f.x2D0;
    let (a, b) = unsafe { (f.co_attrs.air_drift_stick_mul * (*p).xC, f.co_attrs.air_drift_max * (*p).x10) };
    ext::ft_80084E1C(g, cd().x258, a, b);
}

pub fn ftCo_JumpAerialF1_Coll(g: G) {
    ext::ft_80082F28(g);
}
