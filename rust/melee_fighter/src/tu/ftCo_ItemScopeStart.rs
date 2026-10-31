#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_ItemScopeStart.c (Super Scope charge start).
//! Every function of the TU; calls out go through `ext`. Checked against the C by
//! tests/tu_ftCo_ItemScopeStart.rs.
//!
//! The C reads `fp + 0x2340` / `+ 0x2344` through overlay structs: GameCube offsets, i.e. the
//! first two words of `fp->mv` (the charge timer and the released flag).

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
/// GC `fp + 0x2340` as f32 (`mv` word 0).
#[inline(always)]
pub(crate) fn mv_f32<'a>(f: &'a mut Fighter, word: usize) -> &'a mut f32 {
    melee_core::ct!("mv_f32");
    unsafe { &mut *((&mut f.mv as *mut Fighter_MotionVars as *mut f32).add(word)) }
}
/// GC `fp + 0x2340 + 4*word` as s32.
#[inline(always)]
pub(crate) fn mv_i32<'a>(f: &'a mut Fighter, word: usize) -> &'a mut i32 {
    melee_core::ct!("mv_i32");
    unsafe { &mut *((&mut f.mv as *mut Fighter_MotionVars as *mut i32).add(word)) }
}

const HSD_PAD_A: u32 = 0x100;

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
        it_8026B594(item: *mut HSD_GObj) -> bool = |item: *mut HSD_GObj| -> bool { unsafe { crate::tu::ft_0cdf::it_8026b594(item as _) as _ } };
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftCommon_ClampAirDrift(fp: &mut Fighter) = crate::ftcommon::clamp_air_drift;
        ftCommon_8007E79C(g: G, a: i32) = crate::ftcommon3::ftcommon_8007e79c;
        ftCommon_8007E7E4(g: G, a: i32) = crate::ftcommon3::ftcommon_8007e7e4;
        ftCommon_8007D7FC(fp: &mut Fighter) = crate::ftcommon3::ftcommon_8007d7fc;
        ftCommon_8007D5D4(fp: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        it_80291DAC(item: *mut HSD_GObj, n: i32) -> i32 = |item: *mut HSD_GObj, n: i32| -> i32 { unsafe { melee_items::tu::itsscope::it_80291DAC(item as _, n as _) as _ } };
        it_80291F14(item: *mut HSD_GObj, n: i32) = |item: *mut HSD_GObj, n: i32| { unsafe { melee_items::tu::itsscope::it_80291F14(item as _, n as _); } };
        it_80291CF4(item: *mut HSD_GObj, n: i32) -> i32 = |item: *mut HSD_GObj, n: i32| -> i32 { unsafe { melee_items::tu::itsscope::it_80291CF4(item as _, n as _) as _ } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        fn_800D7BDC(g: G) = crate::tu::ftCo_ItemScopeRapid::fn_800D7BDC;
        fn_800D7C60(g: G) = crate::tu::ftCo_ItemScopeRapid::fn_800D7C60;
        fn_800D8140(g: G, n: i32) = |g: G, n: i32| { unsafe { crate::tu::ftCo_ItemScopeFire::fn_800D8140(g as _, n as _); } };
        fn_800D81D0(g: G, n: i32) = |g: G, n: i32| { unsafe { crate::tu::ftCo_ItemScopeFire::fn_800D81D0(g as _, n as _); } };
        ftCo_800CDE54(g: G) = crate::tu::ft_0cdd::ftco_800cde54;
        ftCo_800CDE74(g: G) = |g: G| { unsafe { crate::tu::ft_0cdd::ftco_800cde74(g as _); } };
        ft_800841B8(g: G, cb: fn(G)) = crate::tu::ft_081b::ft_800841b8;
        ft_80082C74(g: G, cb: fn(G)) = crate::tu::ft_081b::ft_80082c74;
    }
}

pub unsafe extern "C" fn fn_800D7938_c(g: G) {
    melee_core::ct!("fn_800D7938_c");
    fn_800D7938(g)
}

pub fn fn_800D769C(f: &Fighter, msid: i32) -> i32 {
    melee_core::ct!("fn_800D769C");
    if f.motion_id >= ftCo_MS_ItemScopeStartEmpty as i32 {
        return msid + 8;
    }
    msid
}

fn start_common(g: G, full: i32, empty: i32, air: bool) {
    let m = if !ext::it_8026B594(fp(g).item_gobj) { full } else { empty };
    ext::Fighter_ChangeMotionState(g, m, Ft_MF_None, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    if air {
        ext::ftCommon_ClampAirDrift(fp(g));
    }
    let f = fp(g);
    *mv_f32(f, 0) = 0.0;
    *mv_i32(f, 1) = 0;
    ext::ftCommon_8007E79C(f.gobj, 1);
    fp(g).take_dmg_cb = Some(fn_800D7938_c);
}

pub fn ft_800D76B8(g: G) {
    melee_core::ct!("ft_800D76B8");
    start_common(g, ftCo_MS_ItemScopeStart as i32, ftCo_MS_ItemScopeStartEmpty as i32, false);
}

pub fn ft_800D7770(g: G) {
    melee_core::ct!("ft_800D7770");
    start_common(g, ftCo_MS_ItemScopeAirStart as i32, ftCo_MS_ItemScopeAirStartEmpty as i32, true);
}

pub fn fn_800D7830(g: G) {
    melee_core::ct!("fn_800D7830");
    ext::ftCommon_8007D7FC(fp(g));
    let f = fp(g);
    let msid = if f.motion_id >= ftCo_MS_ItemScopeStartEmpty as i32 { ftCo_MS_ItemScopeStartEmpty } else { ftCo_MS_ItemScopeStart };
    ext::Fighter_ChangeMotionState(g, msid as i32, 0x0C4C5280, f.cur_anim_frame, f.frame_speed_mul, 0.0, core::ptr::null_mut());
    fp(g).take_dmg_cb = Some(fn_800D7938_c);
}

pub fn fn_800D78B0(g: G) {
    melee_core::ct!("fn_800D78B0");
    ext::ftCommon_8007D5D4(fp(g));
    let f = fp(g);
    let msid = if f.motion_id >= ftCo_MS_ItemScopeStartEmpty as i32 { ftCo_MS_ItemScopeAirStartEmpty } else { ftCo_MS_ItemScopeAirStart };
    ext::Fighter_ChangeMotionState(g, msid as i32, 0x0C4C5280, f.cur_anim_frame, f.frame_speed_mul, 0.0, core::ptr::null_mut());
    ext::ftCommon_ClampAirDrift(fp(g));
    fp(g).take_dmg_cb = Some(fn_800D7938_c);
}

pub fn fn_800D7938(g: G) {
    melee_core::ct!("fn_800D7938");
    let f = fp(g);
    if !f.item_gobj.is_null() {
        let n = melee_runtime::cabi::f2i32(*mv_f32(f, 0) as f64);
        let r = ext::it_80291DAC(fp(g).item_gobj, n);
        if r != -1 {
            ext::it_80291F14(fp(g).item_gobj, r);
        }
        ext::ftCommon_8007E7E4(g, 1);
    }
}

pub fn ftCo_ItemScopeStart_Anim(_g: G) { melee_core::ct!("ftCo_ItemScopeStart_Anim");}

pub fn ftCo_ItemScopeAirStart_Anim(_g: G) { melee_core::ct!("ftCo_ItemScopeAirStart_Anim");}

pub fn fn_800D79B4(g: G, cb_ground: fn(G), cb_air: fn(G, i32)) {
    melee_core::ct!("fn_800D79B4");
    let f = fp(g);
    if f.item_gobj.is_null() {
        return;
    }
    if *mv_i32(f, 1) == 0 && f.input.held_buttons[0] & HSD_PAD_A != 0 {
        *mv_f32(f, 0) += 1.0;
    }
    if f.input.held_buttons[0] & HSD_PAD_A == 0 {
        *mv_i32(f, 1) = 1;
    }
    let n = melee_runtime::cabi::f2i32(*mv_f32(f, 0) as f64);
    let item_val = ext::it_80291DAC(f.item_gobj, n);
    if *mv_f32(fp(g), 0) >= cd().x5B8 && !ext::ftAnim_IsFramesRemaining(g) {
        cb_air(g, item_val);
    }
    if *mv_i32(fp(g), 1) == 0 {
        return;
    }
    if ext::ftAnim_IsFramesRemaining(g) {
        return;
    }
    let f = fp(g);
    let n = melee_runtime::cabi::f2i32(*mv_f32(f, 0) as f64);
    if ext::it_80291CF4(f.item_gobj, n) == 0 {
        cb_ground(g);
    } else {
        cb_air(g, item_val);
    }
}

pub fn ftCo_ItemScopeStart_IASA(g: G) {
    melee_core::ct!("ftCo_ItemScopeStart_IASA");
    fn_800D79B4(g, ext::fn_800D7BDC, ext::fn_800D8140);
}

pub fn ftCo_ItemScopeAirStart_IASA(g: G) {
    melee_core::ct!("ftCo_ItemScopeAirStart_IASA");
    fn_800D79B4(g, ext::fn_800D7C60, ext::fn_800D81D0);
}

pub fn ftCo_ItemScopeStart_Phys(g: G) {
    melee_core::ct!("ftCo_ItemScopeStart_Phys");
    ext::ftCo_800CDE54(g);
}

pub fn ftCo_ItemScopeAirStart_Phys(g: G) {
    melee_core::ct!("ftCo_ItemScopeAirStart_Phys");
    ext::ftCo_800CDE74(g);
}

pub fn ftCo_ItemScopeStart_Coll(g: G) {
    melee_core::ct!("ftCo_ItemScopeStart_Coll");
    ext::ft_800841B8(g, fn_800D78B0);
}

pub fn ftCo_ItemScopeAirStart_Coll(g: G) {
    melee_core::ct!("ftCo_ItemScopeAirStart_Coll");
    ext::ft_80082C74(g, fn_800D7830);
}
