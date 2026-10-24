#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_Attack100.c (rapid jab, the special-move
//! entry checks and the L/R+A item catch). Every function of the TU, TARGET_PC paths (the
//! script caps gate, Geno special overrides, m-ex special dispatchers, the Geno "press" item
//! collect); calls out of it go through `ext` (unported ones panic until rewired; ported ones
//! default to the port). Checked against the C (whole TU, shipping contraction) by
//! tests/tu_ftCo_Attack100.rs.

use crate::types::*;
use core::ffi::c_void;

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
    fn pl_8003e854(slot: i32, sub: i32, item: *mut Item_GObj) {
        crate::plbonus2::pl_8003e854(slot, sub, item)
    }
    ext_fns! {
        // TARGET_PC `FT_CAPS_ACTION_ALLOWED(fp, flag)` = `ScriptGame_CapsActionAllowed`.
        ScriptGame_CapsActionAllowed(fp: &Fighter, flag: i32) -> i32 = |_, _| todo!("TODO(port) ScriptGame_CapsActionAllowed (pc/gameworld/script_game.c)");
        // `ftData_SpecialN[kind]` (ftdata.c table).
        ftData_SpecialN(kind: u32) -> HSD_GObjEvent = |kind: u32| -> HSD_GObjEvent { unsafe { crate::tu::ftdata::ftData_SpecialN[kind as usize] } };
        ftData_SpecialLw(kind: u32) -> HSD_GObjEvent = |kind: u32| -> HSD_GObjEvent { unsafe { crate::tu::ftdata::ftData_SpecialLw[kind as usize] } };
        ftData_SpecialHi(kind: u32) -> HSD_GObjEvent = |kind: u32| -> HSD_GObjEvent { unsafe { crate::tu::ftdata::ftData_SpecialHi[kind as usize] } };
        ftData_SpecialAirHi(kind: u32) -> HSD_GObjEvent = |kind: u32| -> HSD_GObjEvent { unsafe { crate::tu::ftdata::ftData_SpecialAirHi[kind as usize] } };
        Geno_SpecialEnter(g: G, which: i32) -> i32 = |_, _| todo!("TODO(pc geno) Geno_SpecialEnter");
        Mex_SpecialNDispatch(kind: i32, g: G, vanilla: HSD_GObjEvent) = |kind: i32, g: G, vanilla: HSD_GObjEvent| { unsafe { let _ = crate::platform::mex_dispatch_ev(crate::platform::MEX_EVENT_SPECIAL_N, kind, g, vanilla); } };
        Mex_SpecialLwDispatch(kind: i32, g: G, vanilla: HSD_GObjEvent) = |kind: i32, g: G, vanilla: HSD_GObjEvent| { unsafe { let _ = crate::platform::mex_dispatch_ev(crate::platform::MEX_EVENT_SPECIAL_LW, kind, g, vanilla); } };
        Mex_SpecialHiDispatch(kind: i32, g: G, vanilla: HSD_GObjEvent) = |kind: i32, g: G, vanilla: HSD_GObjEvent| { unsafe { let _ = crate::platform::mex_dispatch_ev(crate::platform::MEX_EVENT_SPECIAL_HI, kind, g, vanilla); } };
        Mex_SpecialHiAirDispatch(kind: i32, g: G, vanilla: HSD_GObjEvent) = |kind: i32, g: G, vanilla: HSD_GObjEvent| { unsafe { let _ = crate::platform::mex_dispatch_ev(crate::platform::MEX_EVENT_SPECIAL_HI_AIR, kind, g, vanilla); } };
        ftGw_Attack100Start_Enter(g: G) = |_| todo!("TODO(port) ftGw_Attack100Start_Enter (ftgamewatch)");
        ftpickupitem_80094790(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftpickupitem::ftpickupitem_80094790(g as _) as _ } };
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { let _ = crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ft_800892A0(g: G) = |_| todo!("TODO(port) ft_800892A0 (ft_0881.c)");
        ft_80089824(g: G) = crate::ft0892::ft_80089824;
        ft_8008A2BC(g: G) = crate::hooks::ft_8008a2bc;
        ft_80084FA8(g: G) = crate::ft084e::ft_80084fa8;
        ft_80084104(g: G) = |g: G| { unsafe { let _ = crate::tu::ft_081b::ft_80084104(g as _); } };
        Geno_ItemPressDist(item: *mut Item_GObj, fp: &Fighter) -> f32 = |_, _| todo!("TODO(pc geno) Geno_ItemPressDist");
        Geno_ItemPressCollect(item: *mut Item_GObj, player: i32) = |_, _| todo!("TODO(pc geno) Geno_ItemPressCollect");
        ftpickupitem_80094818(g: G, b: bool) = |g: G, b: bool| { unsafe { let _ = crate::tu::ftpickupitem::ftpickupitem_80094818(g as _, b as _); } };
        itGetKind(item: *mut Item_GObj) -> i32 = crate::hooks::it_get_kind;
        pl_8003E854(slot: i32, sub: i32, item: *mut Item_GObj) = pl_8003e854;
        Item_8026AB54(item: *mut Item_GObj, owner: G, part: i32) = crate::hooks::item_8026ab54;
        ftpickupitem_8009447C(g: G, item: *mut Item_GObj) -> bool = |g: G, item: *mut Item_GObj| -> bool { unsafe { crate::tu::ftpickupitem::ftpickupitem_8009447C(g as _, item as _) as _ } };
        efAsync_Spawn(g: G, queue_head: *mut c_void, kind: u32, gfx: u32, jobj: *mut HSD_JObj) = |_, _, _, _, _| todo!("TODO(port) efAsync_Spawn (efasync.c)");
        ftpickupitem_800942A0(g: G, flags: u32) -> *mut Item_GObj = |g: G, flags: u32| -> *mut Item_GObj { unsafe { crate::tu::ftpickupitem::ftpickupitem_800942A0(g as _, flags as _) as _ } };
        ftKb_MexKirbyLike(kind: i32) -> i32 = |_| todo!("TODO(port) ftKb_MexKirbyLike (ftkirby.c)");
        ftKb_SpecialN_800F1CD8(g: G) -> i32 = |_| todo!("TODO(port) ftKb_SpecialN_800F1CD8 (ftkirbyspecialn.c)");
    }
}

/// `FT_CAPS_ACTION_ALLOWED(fp, 16)` (TARGET_PC).
#[inline(always)]
fn caps(f: &Fighter) -> bool {
    ext::ScriptGame_CapsActionAllowed(f, 16) != 0
}

/// `FTKB_IS_KIRBY(kind)` (ft/forward.h, TARGET_PC: m-ex Kirby clones count).
fn is_kirby(kind: FighterKind) -> bool {
    kind as i32 == Ft_Kind_Kirby as i32
        || ((kind as i32) >= Ft_Kind_Mex0 as i32 && (kind as i32) < Ft_Kind_Max as i32 && ext::ftKb_MexKirbyLike(kind as i32) != 0)
}

/// `ftCo_800D6824`: neutral special (Geno override, else the m-ex dispatcher).
pub fn ftCo_800D6824(g: G) -> bool {
    let f = fp(g);
    if !caps(f) {
        return false;
    }
    let vanilla = ext::ftData_SpecialN(f.kind);
    if vanilla.is_none() {
        return false;
    }
    if f.x689 == 0 {
        if ext::Geno_SpecialEnter(g, 0) == 0 {
            ext::Mex_SpecialNDispatch(f.kind as i32, g, vanilla);
        }
        return true;
    }
    false
}

/// `ftCo_800D688C`: B with the stick down.
pub fn ftCo_800D688C(f: &Fighter) -> bool {
    if !caps(f) {
        return false;
    }
    f.input.pressed_buttons & HSD_PAD_B as u32 != 0 && f.input.lstick[0].y < -cd().x21C
}

/// `ftCo_800D68C0`: down special.
pub fn ftCo_800D68C0(g: G) -> bool {
    let f = fp(g);
    if !caps(f) {
        return false;
    }
    let vanilla = ext::ftData_SpecialLw(f.kind);
    if vanilla.is_none() {
        return false;
    }
    if f.x687 == 0 {
        if ext::Geno_SpecialEnter(g, 3) == 0 {
            ext::Mex_SpecialLwDispatch(f.kind as i32, g, vanilla);
        }
        return true;
    }
    false
}

/// `ftCo_800D6928`: B (0x200) with the stick up.
pub fn ftCo_800D6928(f: &Fighter) -> bool {
    if !caps(f) {
        return false;
    }
    f.input.pressed_buttons & 0x200 != 0 && f.input.lstick[0].y >= cd().x21C
}

/// `ftCo_Attack100_CheckInput`: up special (the name is the decomp's).
pub fn ftCo_Attack100_CheckInput(g: G) -> bool {
    let f = fp(g);
    if !caps(f) {
        return false;
    }
    let vanilla = ext::ftData_SpecialHi(f.kind);
    if vanilla.is_none() {
        return false;
    }
    if f.x686 == 0 {
        if ext::Geno_SpecialEnter(g, 2) == 0 {
            ext::Mex_SpecialHiDispatch(f.kind as i32, g, vanilla);
        }
        return true;
    }
    false
}

/// `ftCo_800D69C4`: aerial up special. The NULL test reads ftData_SpecialHi, the dispatch
/// passes ftData_SpecialAirHi, as in the C.
pub fn ftCo_800D69C4(g: G) -> bool {
    let f = fp(g);
    if !caps(f) {
        return false;
    }
    if ext::ftData_SpecialHi(f.kind).is_none() {
        return false;
    }
    if f.x686 == 0 && (f.x68B as i32) >= cd().x1C {
        if ext::Geno_SpecialEnter(g, 6) == 0 {
            let air = ext::ftData_SpecialAirHi(f.kind);
            ext::Mex_SpecialHiAirDispatch(f.kind as i32, g, air);
        }
        return true;
    }
    false
}

/// `ftCo_Attack_800D6A50`: count A presses/releases; enough of them (and the window open)
/// starts the rapid jab.
pub fn ftCo_Attack_800D6A50(g: G) -> bool {
    let f = fp(g);
    if f.input.pressed_buttons & HSD_PAD_A as u32 != 0 || f.input.released_buttons & HSD_PAD_A as u32 != 0 {
        f.x1A54 = f.x1A54.wrapping_add(1);
    }
    if f.x1A54 >= f.co_attrs.rapid_jab_window && f.x2218_b2() != 0 {
        fn_800D6AC4(g);
        return true;
    }
    false
}

pub fn fn_800D6AC4(g: G) {
    if fp(g).kind == Ft_Kind_GameWatch {
        ext::ftGw_Attack100Start_Enter(g);
    } else {
        fn_800D6B8C(g);
    }
}

/// `ftCo_800D6B00(gobj, msid)`.
pub fn ftCo_800D6B00(g: G, msid: i32) {
    let f = fp(g);
    if !ext::ftpickupitem_80094790(g) {
        f.anon_2210.throw_flags = 0;
        ext::Fighter_ChangeMotionState(g, msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ext::ftAnim_8006EBA4(g);
        let f = fp(g);
        f.mv.co.attack100.x0 = 0;
        f.mv.co.attack100.x4 = 0;
    }
}

pub fn fn_800D6B8C(g: G) {
    ftCo_800D6B00(g, ftCo_MS_Attack100Start as i32);
}

pub fn ftCo_Attack100Start_Anim(g: G) {
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::Fighter_ChangeMotionState(g, ftCo_MS_Attack100Loop as i32, Ft_MF_SkipAttackCount as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    }
}

/// The loop body shared by ftCo_800D6C60 and ftCo_Attack100Loop_Anim; `end` runs when the loop
/// may end.
fn loop_anim(g: G, end: &dyn Fn(G)) {
    let f = fp(g);
    if f.cur_anim_frame >= 0.0 && f.cur_anim_frame < f.frame_speed_mul {
        f.mv.co.attack100.x0 = 1;
        ext::ft_800892A0(g);
        ext::ft_80089824(g);
    }
    let f = fp(g);
    let b3 = unsafe {
        if f.anon_2210.anon_0.throw_flags_b3() != 0 {
            f.anon_2210.anon_0.set_throw_flags_b3(0);
            true
        } else {
            false
        }
    };
    if b3 {
        if unsafe { f.mv.co.attack100.x0 != 0 && f.mv.co.attack100.x4 == 0 } {
            end(g);
        } else if !ext::ftpickupitem_80094790(g) {
            fp(g).mv.co.attack100.x4 = 0;
        }
    }
}

/// `ftCo_800D6C60(gobj, callback)`.
pub fn ftCo_800D6C60(g: G, callback: fn(G)) {
    loop_anim(g, &|g| callback(g));
}

pub fn ftCo_Attack100Loop_Anim(g: G) {
    loop_anim(g, &|g| {
        ext::Fighter_ChangeMotionState(g, ftCo_MS_Attack100End as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut())
    });
}

pub fn ftCo_Attack100End_Anim(g: G) {
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ft_8008A2BC(g);
    }
}

pub fn ftCo_Attack100Loop_IASA(g: G) {
    let f = fp(g);
    if f.input.pressed_buttons & HSD_PAD_A as u32 != 0 || f.input.released_buttons & HSD_PAD_A as u32 != 0 {
        f.mv.co.attack100.x4 = 1;
    }
}

pub fn ftCo_Attack100Start_Phys(g: G) {
    ext::ft_80084FA8(g);
}
pub fn ftCo_Attack100Loop_Phys(g: G) {
    ext::ft_80084FA8(g);
}
pub fn ftCo_Attack100End_Phys(g: G) {
    ext::ft_80084FA8(g);
}
pub fn ftCo_Attack100Start_Coll(g: G) {
    ext::ft_80084104(g);
}
pub fn ftCo_Attack100Loop_Coll(g: G) {
    ext::ft_80084104(g);
}
pub fn ftCo_Attack100End_Coll(g: G) {
    ext::ft_80084104(g);
}

/// `fn_800D6F58(gobj, item_gobj)`: catch an item (TARGET_PC: a Geno "press" item is collected).
pub fn fn_800D6F58(g: G, item: *mut Item_GObj) {
    let f = fp(g);
    if ext::Geno_ItemPressDist(item, f) >= 0.0 {
        ext::Geno_ItemPressCollect(item, f.player_id as i32);
        fp(g).set_x2224_b1(1);
        return;
    }
    if !f.item_gobj.is_null() {
        f.x1978 = item;
    } else {
        f.item_gobj = item;
    }
    ext::ftpickupitem_80094818(g, true);
    let f = fp(g);
    let part = if ext::itGetKind(f.item_gobj) == It_Kind_WStar as i32 {
        FtPart_TopN as i32
    } else {
        unsafe { (*(*f.ft_data).x8).x10 as i32 }
    };
    ext::pl_8003E854(f.player_id as i32, f.is_sub_fighter() as i32, item);
    ext::Item_8026AB54(item, g, part);
    ext::ftpickupitem_8009447C(g, item);
    let f = fp(g);
    let jobj = unsafe { (*f.parts.add((*(*f.ft_data).x8).x10 as usize)).joint };
    ext::efAsync_Spawn(g, &mut f.x60C as *mut *mut c_void as *mut c_void, 0, 0x422, jobj);
    fp(g).set_x2224_b1(1);
}

fn catch_ready(f: &Fighter) -> bool {
    f.item_gobj.is_null()
        && f.input.held_buttons[0] & HSD_PAD_LR != 0
        && f.input.pressed_buttons & HSD_PAD_A as u32 != 0
        && f.x2224_b1() == 0
        && (f.x683 as i32) >= cd().x1C
}

/// `ftCo_800D705C`: arm the L/R+A catch window.
pub fn ftCo_800D705C(g: G) -> bool {
    let f = fp(g);
    if catch_ready(f) {
        f.x209C = cd().x414 as u16;
        return true;
    }
    false
}

/// `ftCo_800D7100`: L/R+A catch now.
pub fn ftCo_800D7100(g: G) -> bool {
    if catch_ready(fp(g)) {
        let item = ext::ftpickupitem_800942A0(g, 1);
        if !item.is_null() {
            fn_800D6F58(g, item);
            return true;
        }
    }
    false
}

/// `ftCo_800D71D8`: run the armed catch window.
pub fn ftCo_800D71D8(g: G) {
    let f = fp(g);
    if f.x209C != 0 {
        let item = ext::ftpickupitem_800942A0(g, 1);
        let f = fp(g);
        f.x209C = f.x209C.wrapping_sub(1);
        if f.x209C == 0 {
            f.set_x2224_b1(1);
        }
        if !item.is_null() {
            fn_800D6F58(g, item);
            fp(g).x209C = 0;
        }
    }
}

/// `ftCo_800D7268(fp)`.
pub fn ftCo_800D7268(f: &Fighter) -> i32 {
    if is_kirby(f.kind) {
        return ext::ftKb_SpecialN_800F1CD8(f.gobj);
    }
    0
}
