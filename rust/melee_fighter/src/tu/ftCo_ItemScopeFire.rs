#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_ItemScopeFire.c (Super Scope fire motions and
//! the shot spawn `fn_800D84D4`). Every function of the TU; calls out go through `ext`. Checked
//! against the C by tests/tu_ftCo_ItemScopeFire.rs. No float arithmetic (the shot position comes
//! from ftCo_800CDE94 and is only passed through; z is zeroed).

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
        fn_800D769C(fp: &Fighter, msid: i32) -> i32 = crate::tu::ftCo_ItemScopeStart::fn_800D769C;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCommon_ClampAirDrift(fp: &mut Fighter) = crate::ftcommon::clamp_air_drift;
        ftCommon_8007D7FC(fp: &mut Fighter) = crate::ftcommon3::ftcommon_8007d7fc;
        ftCommon_8007D5D4(fp: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        ftCommon_8007D92C(g: G) = crate::ftcommon3::ftcommon_8007d92c;
        ftCommon_8007E7E4(g: G, a: i32) = crate::ftcommon3::ftcommon_8007e7e4;
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCo_800CDE54(g: G) = crate::tu::ft_0cdd::ftco_800cde54;
        ftCo_800CDE74(g: G) = crate::tu::ft_0cdd::ftco_800cde74;
        ftCo_800CDE94(fp: &mut Fighter, inp: &Vec3, out: &mut Vec3) = crate::tu::ft_0cdd::ftco_800cde94;
        ft_800841B8(g: G, cb: fn(G)) = crate::tu::ft_081b::ft_800841b8;
        ft_80082C74(g: G, cb: fn(G)) = crate::tu::ft_081b::ft_80082c74;
        ft_PlaySFX(fp: &mut Fighter, sfx: i32, vol: i32, pan: i32) = crate::hooks::play_sfx;
        it_80291CCC(item: *mut HSD_GObj, out: &mut Vec3) = |item: *mut HSD_GObj, out: &mut Vec3| { unsafe { melee_items::tu::itsscope::it_80291CCC(item as _, out as _); } };
        it_80291FA8(item: *mut HSD_GObj, pos: &mut Vec3, n: i32, facing: f32) = |item: *mut HSD_GObj, pos: &mut Vec3, n: i32, facing: f32| { unsafe { melee_items::tu::itsscope::it_80291FA8(item as _, pos as _, n as _, facing as _); } };
        efSync_Spawn(id: i32, g: G, pos: &mut Vec3, facing: Option<&mut f32>) = |_, _, _, _| todo!("TODO(port) efSync_Spawn (efsync.c; variadic, facing = optional &fp->facing_dir)");
    }
}

pub unsafe extern "C" fn fn_800D8378_c(g: G) {
    melee_core::ct!("fn_800D8378_c");
    fn_800D8378(g)
}
pub unsafe extern "C" fn fn_800D86B8_c(g: G) {
    melee_core::ct!("fn_800D86B8_c");
    fn_800D86B8(g)
}

#[inline(always)]
fn set_cbs(f: &mut Fighter) {
    f.accessory4_cb = Some(fn_800D86B8_c);
    f.take_dmg_cb = Some(fn_800D8378_c);
}

pub fn fn_800D8140(g: G, arg1: i32) {
    melee_core::ct!("fn_800D8140");
    unsafe { fp(g).anon_2210.throw_flags = 0 };
    let msid = ext::fn_800D769C(fp(g), ftCo_MS_ItemScopeFire as i32);
    ext::Fighter_ChangeMotionState(g, msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    unsafe { f.mv.co.common.x0 = arg1 };
    set_cbs(f);
}

pub fn fn_800D81D0(g: G, arg1: i32) {
    melee_core::ct!("fn_800D81D0");
    unsafe { fp(g).anon_2210.throw_flags = 0 };
    let msid = ext::fn_800D769C(fp(g), ftCo_MS_ItemScopeAirFire as i32);
    ext::Fighter_ChangeMotionState(g, msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftCommon_ClampAirDrift(fp(g));
    let f = fp(g);
    unsafe { f.mv.co.common.x0 = arg1 };
    set_cbs(f);
}

pub fn fn_800D8268(g: G) {
    melee_core::ct!("fn_800D8268");
    ext::ftCommon_8007D7FC(fp(g));
    let msid = ext::fn_800D769C(fp(g), ftCo_MS_ItemScopeFire as i32);
    let f = fp(g);
    ext::Fighter_ChangeMotionState(g, msid, 0x0C4C5080, f.cur_anim_frame, f.frame_speed_mul, 0.0, core::ptr::null_mut());
    set_cbs(fp(g));
}

pub fn fn_800D82EC(g: G) {
    melee_core::ct!("fn_800D82EC");
    ext::ftCommon_8007D5D4(fp(g));
    let msid = ext::fn_800D769C(fp(g), ftCo_MS_ItemScopeAirFire as i32);
    let f = fp(g);
    ext::Fighter_ChangeMotionState(g, msid, 0x0C4C5080, f.cur_anim_frame, f.frame_speed_mul, 0.0, core::ptr::null_mut());
    ext::ftCommon_ClampAirDrift(fp(g));
    set_cbs(fp(g));
}

pub fn fn_800D8378(g: G) {
    melee_core::ct!("fn_800D8378");
    if !fp(g).item_gobj.is_null() {
        ext::ftCommon_8007E7E4(g, 1);
    }
}

fn fire_anim(g: G) {
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCommon_8007D92C(g);
        ext::ftCommon_8007E7E4(g, 1);
    }
}

pub fn ftCo_ItemScopeFire_Anim(g: G) {
    melee_core::ct!("ftCo_ItemScopeFire_Anim");
    fire_anim(g);
}

pub fn ftCo_ItemScopeAirFire_Anim(g: G) {
    melee_core::ct!("ftCo_ItemScopeAirFire_Anim");
    fire_anim(g);
}

pub fn ftCo_ItemScopeFire_IASA(_g: G) { melee_core::ct!("ftCo_ItemScopeFire_IASA");}

pub fn ftCo_ItemScopeAirFire_IASA(_g: G) { melee_core::ct!("ftCo_ItemScopeAirFire_IASA");}

pub fn ftCo_ItemScopeFire_Phys(g: G) {
    melee_core::ct!("ftCo_ItemScopeFire_Phys");
    ext::ftCo_800CDE54(g);
}

pub fn ftCo_ItemScopeAirFire_Phys(g: G) {
    melee_core::ct!("ftCo_ItemScopeAirFire_Phys");
    ext::ftCo_800CDE74(g);
}

pub fn ftCo_ItemScopeFire_Coll(g: G) {
    melee_core::ct!("ftCo_ItemScopeFire_Coll");
    ext::ft_800841B8(g, fn_800D82EC);
}

pub fn ftCo_ItemScopeAirFire_Coll(g: G) {
    melee_core::ct!("ftCo_ItemScopeAirFire_Coll");
    ext::ft_80082C74(g, fn_800D8268);
}

pub fn fn_800D84D4(g: G, arg1: i32) {
    melee_core::ct!("fn_800D84D4");
    let f = fp(g);
    if f.item_gobj.is_null() {
        return;
    }
    // ftCheckThrowB0: test and clear.
    if unsafe { f.anon_2210.anon_0.throw_flags_b0() } == 0 {
        return;
    }
    unsafe { f.anon_2210.anon_0.set_throw_flags_b0(0) };
    let mut sp14 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp20 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::it_80291CCC(f.item_gobj, &mut sp14);
    ext::ftCo_800CDE94(fp(g), &sp14, &mut sp20);
    let f = fp(g);
    let item = f.item_gobj;
    let facing = f.facing_dir;
    let fgobj = f.gobj;
    if arg1 == 0 {
        ext::it_80291FA8(item, &mut sp20, arg1, facing);
        sp20.z = 0.0;
        ext::efSync_Spawn(0x430, fgobj, &mut sp20, Some(&mut fp(g).facing_dir));
        ext::ft_PlaySFX(fp(g), 0xFC, 0x7F, 0x40);
    } else if (1..=8).contains(&arg1) {
        ext::it_80291FA8(item, &mut sp20, arg1, facing);
        sp20.z = 0.0;
        ext::efSync_Spawn(0x430, fgobj, &mut sp20, Some(&mut fp(g).facing_dir));
        if arg1 <= 4 {
            ext::ft_PlaySFX(fp(g), 0xFF, 0x7F, 0x40);
        } else {
            ext::ft_PlaySFX(fp(g), 0x100, 0x7F, 0x40);
        }
    } else if arg1 == 9 {
        ext::it_80291FA8(item, &mut sp20, arg1, facing);
        sp20.z = 0.0;
        ext::efSync_Spawn(0x430, fgobj, &mut sp20, Some(&mut fp(g).facing_dir));
        ext::ft_PlaySFX(fp(g), 0x100, 0x7F, 0x40);
    } else {
        ext::efSync_Spawn(0x405, g, &mut sp20, None);
        ext::ft_PlaySFX(fp(g), 0x101, 0x7F, 0x40);
    }
}

pub fn fn_800D86B8(g: G) {
    melee_core::ct!("fn_800D86B8");
    let x = unsafe { fp(g).mv.co.common.x0 };
    fn_800D84D4(g, x);
}
