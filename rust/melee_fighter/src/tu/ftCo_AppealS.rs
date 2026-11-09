#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_AppealS.c (side taunt). Every function of the
//! TU, TARGET_PC path (the m-ex onTaunt dispatch after the motion state is entered); calls out
//! of it go through `ext` (unported ones panic until rewired; ported ones default to the port).
//! Checked against the C (whole TU, shipping contraction) by tests/tu_ftCo_AppealS.rs.

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
        ftCl_Init_80149318(g: G) = |_| todo!("TODO(port) ftCl_Init_80149318 (ftclink)");
        ftDr_Init_80149910(g: G) = |_| todo!("TODO(port) ftDr_Init_80149910 (ftdrmario)");
        lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *const Vec3, out: &mut Vec3) = |jobj: *mut HSD_JObj, pos0: *const Vec3, out: &mut Vec3| { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, pos0 as _, out as _); } };
        lb_800119DC(pos: &mut Vec3, n: i32, a: f32, b: f32, c: f32) = |pos: &mut Vec3, n: i32, a: f32, b: f32, c: f32| { unsafe { melee_runtime::tu::lb_00F9::lb_800119DC(pos as _, n as _, a as _, b as _, c as _); } };
        pl_80040120(slot: i32, sub: i32) = crate::plbonus::pl_80040120;
        // `ftData_80085FD4(fp, anim_id)->x8`.
        ftData_80085FD4(fp: &mut Fighter, anim_id: i32) -> usize = crate::hooks::ftdata_80085fd4_x8;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        // `DbLevel` (db.c global).
        DbLevel() -> i32 = || -> i32 { unsafe { crate::platform::DbLevel() } };
        ftPe_Init_8011B93C(g: G) = |_| todo!("TODO(port) ftPe_Init_8011B93C (ftpeach)");
        ftZd_Init_801395C8(g: G) = |_| todo!("TODO(port) ftZd_Init_801395C8 (ftzelda)");
        ftKb_MexKirbyLike(kind: i32) -> i32 = |_| todo!("TODO(port) ftKb_MexKirbyLike (ftkirby.c)");
        ftKb_SpecialN_800F5D04(g: G, b: bool) = |_, _| todo!("TODO(port) ftKb_SpecialN_800F5D04 (ftkirbyspecialn.c)");
        Mex_OnTauntDispatch(kind: i32, g: G, vanilla: *mut core::ffi::c_void) = |kind: i32, g: G, vanilla: *mut core::ffi::c_void| { unsafe { let _ = crate::platform::mex_dispatch(crate::platform::MEX_EVENT_ON_TAUNT, kind, g, vanilla); } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ft_8008A2BC(g: G) = crate::hooks::ft_8008a2bc;
        ftCo_SpecialS_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_SpecialS::ftCo_SpecialS_CheckInput(g as _) as _ } };
        ftCo_Attack100_CheckInput(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_Attack100_CheckInput;
        ftCo_800D6824(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_800D6824;
        ftCo_800D68C0(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_800D68C0;
        ftCo_Catch_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Catch::ftCo_Catch_CheckInput(g as _) as _ } };
        ftCo_AttackS4_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackS4::ftCo_AttackS4_CheckInput;
        ftCo_AttackHi4_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackHi4::ftCo_AttackHi4_CheckInput;
        ftCo_AttackLw4_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackLw4::ftCo_AttackLw4_CheckInput;
        ftCo_AttackS3_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackS3::ftCo_AttackS3_CheckInput;
        ftCo_AttackHi3_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackHi3::ftCo_AttackHi3_CheckInput;
        ftCo_AttackLw3_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackLw3::ftCo_AttackLw3_CheckInput;
        ftCo_Attack1_CheckInput(g: G) -> bool = crate::tu::ftCo_Attack1::ftCo_Attack1_CheckInput;
        ftCo_80099794(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Escape::ftCo_80099794(g as _) as _ } };
        ftCo_80091A4C(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Guard::ftCo_80091A4C(g as _) as _ } };
        ft_80084FA8(g: G) = crate::ft084e::ft_80084fa8;
        ft_80084104(g: G) = |g: G| { unsafe { crate::tu::ft_081b::ft_80084104(g as _); } };
    }
}

/// `FTKB_IS_KIRBY(kind)` (ft/forward.h, TARGET_PC: m-ex Kirby clones count).
fn is_kirby(kind: FighterKind) -> bool {
    kind as i32 == Ft_Kind_Kirby as i32
        || ((kind as i32) >= Ft_Kind_Mex0 as i32 && (kind as i32) < Ft_Kind_Max as i32 && ext::ftKb_MexKirbyLike(kind as i32) != 0)
}

/// `ftCo_800DE9B8`: D-pad up pressed.
pub fn ftCo_800DE9B8(g: G) -> bool {
    melee_core::ct!("ftCo_800DE9B8");
    fp(g).input.pressed_buttons & HSD_PAD_DPADUP as u32 != 0
}

/// `ftCo_800DE9D8`: taunt on D-pad up.
pub fn ftCo_800DE9D8(g: G) -> bool {
    melee_core::ct!("ftCo_800DE9D8");
    if ftCo_800DE9B8(g) {
        ftCo_800DEA28(g);
        return true;
    }
    false
}

/// `ftCo_800DEA28`: the per-kind taunt entry. Ganondorf's case falls through into the default,
/// so his taunt is entered twice, as in the C.
pub fn ftCo_800DEA28(g: G) {
    melee_core::ct!("ftCo_800DEA28");
    let f = fp(g);
    match f.kind {
        Ft_Kind_CLink => ext::ftCl_Init_80149318(g),
        Ft_Kind_DrMario => ext::ftDr_Init_80149910(g),
        Ft_Kind_Ganon => {
            let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            let joint = unsafe { (*f.parts).joint };
            ext::lb_8000B1CC(joint, core::ptr::null(), &mut pos);
            ext::lb_800119DC(&mut pos, 80, 1.0, 0.003, 1.0471976);
            ftCo_800DEBD0(g);
            ftCo_800DEBD0(g);
        }
        _ => ftCo_800DEBD0(g),
    }
    let f = fp(g);
    ext::pl_80040120(f.player_idx as i32, f.is_sub_fighter() as i32);
}

/// `ftCo_800DEAE8(gobj, msid0, msid1)`: enter msid1 (the left-facing variant) when facing
/// left and its animation exists, else msid0.
pub fn ftCo_800DEAE8(g: G, msid0: i32, msid1: i32) {
    melee_core::ct!("ftCo_800DEAE8");
    let f = fp(g);
    let ms = unsafe {
        if msid1 >= f.x18 {
            &*f.x20_actionStateList.offset((msid1 - f.x18) as isize)
        } else {
            &*f.x1C_actionStateList.offset(msid1 as isize)
        }
    };
    let anim_id = ms.anim_id;
    f.set_allow_interrupt(0);
    if f.facing_dir == -1.0 && ext::ftData_80085FD4(f, anim_id) != 0 {
        ext::Fighter_ChangeMotionState(g, msid1, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    } else {
        ext::Fighter_ChangeMotionState(g, msid0, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    }
}

/// `ftCo_800DEBD0`: side taunt (debug-ROM Peach/Zelda inits, Kirby drops his copy ability;
/// TARGET_PC: m-ex onTaunt after the motion state).
pub fn ftCo_800DEBD0(g: G) {
    melee_core::ct!("ftCo_800DEBD0");
    let kind = fp(g).kind;
    if ext::DbLevel() >= DbLKind_DebugRom as i32 {
        if kind == Ft_Kind_Peach {
            ext::ftPe_Init_8011B93C(g);
        }
        if fp(g).kind == Ft_Kind_Zelda {
            ext::ftZd_Init_801395C8(g);
        }
    }
    if is_kirby(fp(g).kind) {
        ext::ftKb_SpecialN_800F5D04(g, true);
    }
    ftCo_800DEAE8(g, ftCo_MS_AppealSR as i32, ftCo_MS_AppealSL as i32);
    ext::Mex_OnTauntDispatch(fp(g).kind as i32, g, core::ptr::null_mut());
}

pub fn ftCo_AppealS_Anim(g: G) {
    melee_core::ct!("ftCo_AppealS_Anim");
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ft_8008A2BC(g);
    }
}

pub fn ftCo_AppealS_IASA(g: G) {
    melee_core::ct!("ftCo_AppealS_IASA");
    if fp(g).allow_interrupt() == 0 {
        return;
    }
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
        || ext::ftCo_80099794(g)
        || ext::ftCo_80091A4C(g);
}

pub fn ftCo_AppealS_Phys(g: G) {
    melee_core::ct!("ftCo_AppealS_Phys");
    ext::ft_80084FA8(g);
}

pub fn ftCo_AppealS_Coll(g: G) {
    melee_core::ct!("ftCo_AppealS_Coll");
    ext::ft_80084104(g);
}
