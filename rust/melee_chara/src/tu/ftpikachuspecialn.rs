//! src/melee/ft/kinds/ftPikachu/ftpikachuspecialn.c: Pikachu / Pichu Thunder Jolt (neutral B).
//! Fused sites (tools/rust-port/fma_sites.py), in `ftPk_SpecialN_Anim` and `ftPk_SpecialAirN_Anim`
//! (two each): `x = scale.y * (offset.x * facing_dir) + cur_pos.x` = fmadds(scale.y,
//! offset.x * facing_dir, cur_pos.x) and `y = offset.y * scale.y + cur_pos.y`. The int field
//! `mv.pk.unk2.x0` takes a float attribute, so that store truncates (`as i32`). Smoke-tested by
//! tests/tu_ftpikachuspecialn.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_types::*;

/// Already ported (rounds 2/4/12): re-exported under this TU.
pub use crate::chars::r12::ftPikachu::ftpikachuspecialn::ftPk_SpecialN_Coll;
pub use crate::chars::r2::ftPikachu::ftpikachuspecialn::{ftPk_SpecialAirN_Phys, ftPk_SpecialN_Phys};
pub use crate::chars::r4::ftPikachu::ftpikachuspecialn::ftPk_SpecialAirN_Coll;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use core::ffi::c_void;
    use melee_types::*;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    pub static mut ITPIKACHUTHUNDERJOLT_SPAWN: Option<fn(*mut HSD_GObj, *mut Vec3, f32, i32) > = None;
            #[allow(non_snake_case)]
            pub fn itPikachuThunderJolt_Spawn(gobj: *mut HSD_GObj, pos: *mut Vec3, facing_dir: f32, kind: i32)  {
                match unsafe { ITPIKACHUTHUNDERJOLT_SPAWN } {
                    Some(f) => f(gobj, pos, facing_dir, kind),
                    None => { unsafe { let _ = melee_items::tu::itpikachutjoltground::itPikachuThunderJolt_Spawn(gobj as _, pos as _, facing_dir as _, kind as _); } return; },
                }
            }
    // `efSync_Spawn(gfx_id, gobj, ...)` (ef/efsync.c, variadic): the trailing pointers as a slice.
    crate::port_hook!(EFSYNC_SPAWN, efSync_Spawn, crate::adapt::efSync_Spawn, (gfx_id: i32, gobj: *mut HSD_GObj, args: &[*mut c_void]) -> *mut c_void);
}

type G = *mut HSD_GObj;

#[inline(always)]
unsafe fn fpp(gobj: G) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *mut ftPikachuAttributes {
    (*fp).dat_attrs as *mut ftPikachuAttributes
}

const ftPk_SpecialN_Cmd0: usize = 0;
const ftPk_SpecialN_Cmd1: usize = 1;
const ftPk_SpecialN_Cmd2: usize = 2;
const ftPk_SpecialN_Cmd3: usize = 3;

/// `doEnter` (static).
pub fn doEnter(gobj: G, msid: ftPikachu_MotionState) {
    unsafe {
        let fp = fpp(gobj);
        st::Fighter_ChangeMotionState(gobj, msid as i32, Ft_MF_None, 0.0, 1.0, 0.0, core::ptr::null_mut());
        (*fp).cmd_vars[ftPk_SpecialN_Cmd0] = 0;
        (*fp).cmd_vars[ftPk_SpecialN_Cmd1] = 0;
        (*fp).cmd_vars[ftPk_SpecialN_Cmd2] = 0;
        (*fp).cmd_vars[ftPk_SpecialN_Cmd3] = 0;
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftPk_SpecialN_Enter(gobj: G) {
    doEnter(gobj, ftPk_MS_SpecialN);
}

pub fn ftPk_SpecialAirN_Enter(gobj: G) {
    doEnter(gobj, ftPk_MS_SpecialAirN);
}

/// The jolt spawn shared by the ground and air Anim: `offset` is the attribute's spawn offset.
unsafe fn spawn_jolt(gobj: G, fp: *mut Fighter, pika_attr: *mut ftPikachuAttributes, offset: Vec2) {
    let mut pos: Vec3 = core::mem::zeroed();
    pos.x = fmadds((*fp).x34_scale.y, offset.x * (*fp).facing_dir, (*fp).cur_pos.x);
    pos.y = fmadds(offset.y, (*fp).x34_scale.y, (*fp).cur_pos.y);
    pos.z = 0.0;
    ext::itPikachuThunderJolt_Spawn(gobj, &mut pos, (*fp).facing_dir, (*pika_attr).specialn_itkind as i32);
    let kind = melee_fighter::hooks::ftlib_get_kind(gobj);
    match kind {
        Ft_Kind_Pikachu => melee_fighter::hooks::play_sfx(&mut *fp, 240076, 127, 64),
        Ft_Kind_Pichu => melee_fighter::hooks::play_sfx(&mut *fp, 230067, 127, 64),
        _ => {}
    }
}

pub fn ftPk_SpecialN_Anim(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let pika_attr = da(fp);
        if (*fp).cmd_vars[ftPk_SpecialN_Cmd0] == 1 {
            (*fp).cmd_vars[ftPk_SpecialN_Cmd0] = 0;
            if (*fp).cmd_vars[ftPk_SpecialN_Cmd1] == 0 {
                (*fp).cmd_vars[ftPk_SpecialN_Cmd1] = 1;
                spawn_jolt(gobj, fp, pika_attr, (*pika_attr).specialn_spawn_offset);
            }
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftPk_SpecialAirN_Anim(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let pika_attr = da(fp);
        if (*fp).cmd_vars[ftPk_SpecialN_Cmd0] == 1 {
            (*fp).cmd_vars[ftPk_SpecialN_Cmd0] = 0;
            if (*fp).cmd_vars[ftPk_SpecialN_Cmd1] == 0 {
                (*fp).cmd_vars[ftPk_SpecialN_Cmd1] = 1;
                spawn_jolt(gobj, fp, pika_attr, (*pika_attr).specialairn_spawn_offset);
            }
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            if (*pika_attr).specialairn_landing_lag == 0.0 {
                st::ftCo_Fall_Enter(gobj);
            } else {
                st::ftCo_80096900(gobj, 1, 0, 1, 1.0, (*pika_attr).specialairn_landing_lag);
            }
        }
    }
}

pub fn ftPk_SpecialN_IASA(_gobj: G) {}

pub fn ftPk_SpecialAirN_IASA(_gobj: G) {}

/// The effect spawn shared by `ftPk_SpecialN_SpawnEffect0/1`.
unsafe fn spawn_effect(gobj: G, gfx_id: i32) {
    let fp = fpp(gobj);
    if (*fp).x2219_b0() == 0 {
        let part = melee_fighter::hooks::ftparts_get_bone_index(&*fp, FtPart_HipN as i32);
        ext::efSync_Spawn(gfx_id, gobj, &[(*(*fp).parts.add(part as usize)).joint as *mut core::ffi::c_void]);
        (*fp).set_x2219_b0(1);
    }
    // `Fighter_SetEffectHitlagCallbacks(fp)` (ft/inlines.h).
    (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
    (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
    (*fp).accessory4_cb = None;
}

pub fn ftPk_SpecialN_SpawnEffect0(gobj: G) {
    unsafe { spawn_effect(gobj, 1214) }
}

pub fn ftPk_SpecialN_SpawnEffect1(gobj: G) {
    unsafe { spawn_effect(gobj, 1215) }
}

pub fn ftPk_SpecialN_80124DC8(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let pika_attr = da(fp);
        (*fp).cmd_vars[ftPk_SpecialN_Cmd0] = 0;
        let fp_x673 = (*fp).active_sticky.lstick.x;
        let pika_attr_1c = (*pika_attr).x1C;
        if (fp_x673 as f32) < pika_attr_1c {
            (*fp).mv.pk.unk2.x0 = (*pika_attr).x20 as i32;
            (*fp).x2070.anon_0.set_count_thrown_items(1);
        } else {
            (*fp).mv.pk.unk2.x0 = 0;
        }
    }
}

/// `ftPk_SpecialN_80124DC8` / `ftPk_SpecialN_SpawnEffect0/1` as Fighter callback entries
/// (`x21EC` and `accessory4_cb`, stored by ftpikachuspecials.c).
pub unsafe extern "C" fn cb_ftPk_SpecialN_80124DC8(g: G) {
    ftPk_SpecialN_80124DC8(g);
}
pub unsafe extern "C" fn cb_ftPk_SpecialN_SpawnEffect0(g: G) {
    ftPk_SpecialN_SpawnEffect0(g);
}
pub unsafe extern "C" fn cb_ftPk_SpecialN_SpawnEffect1(g: G) {
    ftPk_SpecialN_SpawnEffect1(g);
}
