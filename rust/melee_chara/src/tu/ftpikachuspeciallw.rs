//! src/melee/ft/kinds/ftPikachu/ftpikachuspeciallw.c: Pikachu / Pichu Thunder (down special).
//! No fused sites (tools/rust-port/fma_sites.py: 0 fmuladd in every function); the only float
//! math is the `ABS` distance tests in `ftPk_SpecialLw_8012765C`. The Coll / Phys / ChangeMotion
//! functions are thin; their ports live in `chars` (rounds 2/4/5/6/7) and are re-exported here.
//! Smoke-tested through recording hooks by tests/tu_ftpikachuspeciallw.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_fighter::ftcommon::abs_c;
use melee_types::*;

/// Already ported (rounds 2/4/5/6/7 and the original pass): re-exported under this TU.
pub use crate::chars::r2::ftPikachu::ftpikachuspeciallw::{
    ftPk_SpecialAirLwEnd_Anim, ftPk_SpecialAirLwEnd_Phys, ftPk_SpecialAirLwLoop0_Phys,
    ftPk_SpecialAirLwStart_Phys, ftPk_SpecialLwEnd_Anim, ftPk_SpecialLwEnd_Phys,
    ftPk_SpecialLwLoop0_Phys, ftPk_SpecialLwLoop1_Phys, ftPk_SpecialLwStart_Phys,
    ftPk_SpecialLw_ChangeMotion_Unk00, ftPk_SpecialLw_ChangeMotion_Unk04,
    ftPk_SpecialLw_ChangeMotion_Unk06,
};
pub use crate::chars::r4::ftPikachu::ftpikachuspeciallw::{
    ftPk_SpecialAirLwLoop1_Anim, ftPk_SpecialLwLoop1_Anim, ftPk_SpecialLw_ChangeMotion_Unk02,
};
pub use crate::chars::r5::ftPikachu::ftpikachuspeciallw::{
    ftPk_SpecialAirLwEnd_Coll, ftPk_SpecialAirLwLoop0_Coll, ftPk_SpecialAirLwLoop1_Coll,
    ftPk_SpecialAirLwStart_Coll, ftPk_SpecialLwEnd_Coll, ftPk_SpecialLwLoop0_Coll,
    ftPk_SpecialLwLoop1_Coll, ftPk_SpecialLwStart_Coll,
};
pub use crate::chars::r6::ftPikachu::ftpikachuspeciallw::{
    ftPk_SpecialLw_ChangeMotion_Unk01, ftPk_SpecialLw_ChangeMotion_Unk03,
    ftPk_SpecialLw_ChangeMotion_Unk05, ftPk_SpecialLw_ChangeMotion_Unk07,
};
pub use crate::chars::r7::ftPikachu::ftpikachuspeciallw::{ftPk_SpecialAirLw_Enter, ftPk_SpecialLw_Enter};

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
    crate::port_hook!(CAMERA_REQUESTQUAKE, Camera_RequestQuake, crate::adapt::Camera_RequestQuake, (kind: CmQuakeKind, pos: *mut Vec3));
    // `efSync_Spawn(gfx_id, gobj, ...)` (ef/efsync.c, variadic): the trailing pointers as a slice.
    crate::port_hook!(EFSYNC_SPAWN, efSync_Spawn, crate::adapt::efSync_Spawn, (gfx_id: i32, gobj: *mut HSD_GObj, args: &[*mut c_void]) -> *mut c_void);
    crate::port_hook!(EFASYNC_SPAWN, efAsync_Spawn, crate::adapt::efAsync_Spawn_plain,
        (gobj: *mut HSD_GObj, queue: *mut c_void, kind: u32, gfx: u32, jobj: *mut HSD_JObj));
    pub static mut IT_802B1DEC: Option<fn(*mut Item_GObj) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn it_802B1DEC(thunder: *mut Item_GObj) -> i32 {
                match unsafe { IT_802B1DEC } {
                    Some(f) => f(thunder),
                    None => { return unsafe { melee_items::tu::itpikachuthunder::it_802B1DEC(thunder as _) as _ }; },
                }
            }
    pub static mut IT_802B1DF8: Option<fn(*mut HSD_GObj, *mut Vec3, *mut Vec3, i32, i32, i32) -> *mut Item_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_802B1DF8(owner: *mut HSD_GObj, pos: *mut Vec3, vel: *mut Vec3, count: i32, delay: i32, kind: i32) -> *mut Item_GObj {
                match unsafe { IT_802B1DF8 } {
                    Some(f) => f(owner, pos, vel, count, delay, kind),
                    None => { return unsafe { melee_items::tu::itpikachuthunder::it_802B1DF8(owner as _, pos as _, vel as _, count as _, delay as _, kind as _) as _ }; },
                }
            }
    pub static mut IT_802B1FC8: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802B1FC8(thunder: *mut Item_GObj)  {
                match unsafe { IT_802B1FC8 } {
                    Some(f) => f(thunder),
                    None => { unsafe { let _ = melee_items::tu::itpikachuthunder::it_802B1FC8(thunder as _); } return; },
                }
            }
    pub static mut IT_802B1FE8: Option<fn(*mut Item_GObj, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn it_802B1FE8(thunder: *mut Item_GObj, out: *mut Vec3)  {
                match unsafe { IT_802B1FE8 } {
                    Some(f) => f(thunder, out),
                    None => { unsafe { let _ = melee_items::tu::itpikachuthunder::it_802B1FE8(thunder as _, out as _); } return; },
                }
            }
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

/// `ftCheckThrowB0` (ft/inlines.h): test and clear `throw_flags_b0`.
#[inline(always)]
unsafe fn ftCheckThrowB0(fp: *mut Fighter) -> bool {
    if (*fp).anon_2210.anon_0.throw_flags_b0() != 0 {
        (*fp).anon_2210.anon_0.set_throw_flags_b0(0);
        true
    } else {
        false
    }
}

/// Gobj entries for the ported `&mut Fighter` callbacks.
pub fn ftPk_SpecialAirLwLoop1_Phys(gobj: G) {
    unsafe { crate::chars::ftPikachu::ftpikachuspeciallw::ftPk_SpecialAirLwLoop1_Phys(&mut *fpp(gobj)) }
}
pub fn ftPk_SpecialLw_SetState_Unk0(gobj: G) {
    unsafe { crate::chars::ftPikachu::ftpikachuspeciallw::ftPk_SpecialLw_SetState_Unk0(&mut *fpp(gobj)) }
}
pub fn ftPk_SpecialLw_SetState_Unk1(gobj: G) {
    unsafe { crate::chars::ftPikachu::ftpikachuspeciallw::ftPk_SpecialLw_SetState_Unk1(&mut *fpp(gobj)) }
}

/// `ftPk_SpecialLw_CheckProperty`: true for the Thunder property values 1, 2 and 4..=13.
pub fn ftPk_SpecialLw_CheckProperty(gobj: G) -> bool {
    unsafe {
        let fp = fpp(gobj);
        let value = (*fp).x2070.anon_0.x2071_b0_3() as i32;
        matches!(value, 1 | 2 | 4..=13)
    }
}

pub fn ftPk_SpecialLw_80127608(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        ext::Camera_RequestQuake(QuakeKind_Small, &mut (*fp).cur_pos);
        st::ftCommon_8007EBAC(&mut *fp, 11, 0);
    }
}

/// True when the Thunder cloud's bolt is close enough to strike (and starts it).
pub fn ftPk_SpecialLw_8012765C(gobj: G) -> bool {
    unsafe {
        let fp = fpp(gobj);
        let pika_attr = da(fp);
        let thunder_gobj = (*fp).mv.pk.speciallw.x0;
        let mut vec: Vec3 = core::mem::zeroed();

        if (*fp).mv.pk.speciallw.x4 == 0 {
            return false;
        }
        if !thunder_gobj.is_null() {
            ext::it_802B1FE8(thunder_gobj, &mut vec);
        } else {
            return false;
        }
        if abs_c((*fp).cur_pos.x - vec.x) < abs_c((*pika_attr).xC4) {
            let final_y_pos = abs_c((*fp).cur_pos.y + abs_c((*pika_attr).xBC) - vec.y);
            if final_y_pos < (*pika_attr).xC8
                && !(*fp).mv.pk.speciallw.x0.is_null()
                && ext::it_802B1DEC((*fp).mv.pk.speciallw.x0) == 0
            {
                ext::it_802B1FC8((*fp).mv.pk.speciallw.x0);
                return true;
            }
        }
        false
    }
}

/// `ftPk_SpecialLw_SpawnEffect`: the accessory4 callback that spawns the cloud.
pub fn ftPk_SpecialLw_SpawnEffect(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let pika_attr = da(fp);
        if ftCheckThrowB0(fp) && (*fp).mv.pk.speciallw.x0.is_null() {
            let mut pos: Vec3 = (*fp).cur_pos;
            pos.y += (*pika_attr).xD0;
            let mut ef_pos: Vec3 = pos;
            ef_pos.y += (*pika_attr).xCC;
            ext::efSync_Spawn(1219, gobj, &[&mut ef_pos as *mut Vec3 as *mut core::ffi::c_void]);
            let mut vec: Vec3 = core::mem::zeroed();
            vec.z = 0.0;
            vec.x = 0.0;
            vec.y = (*pika_attr).xC0;
            (*fp).mv.pk.speciallw.x0 = ext::it_802B1DF8(
                gobj,
                &mut pos,
                &mut vec,
                (*pika_attr).xD4,
                (*pika_attr).xD8,
                (*pika_attr).xDC as i32,
            );
        }
    }
}
/// `ftPk_SpecialLw_SpawnEffect` as the `accessory4_cb` entry.
pub unsafe extern "C" fn cb_ftPk_SpecialLw_SpawnEffect(g: G) {
    ftPk_SpecialLw_SpawnEffect(g);
}

/// The Start_Anim body shared by ground (`msid` 360) and air (364).
unsafe fn start_anim(gobj: G, msid: i32) {
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        let fp = fpp(gobj);
        st::Fighter_ChangeMotionState(gobj, msid, 2048, 0.0, 1.0, 0.0, core::ptr::null_mut());
        let fighter_copy = fpp(gobj);
        (*fighter_copy).anon_2210.throw_flags = 0;
        (*fighter_copy).take_dmg_cb = Some(crate::chars::cb::ftpikachuspeciallw__ftPk_SpecialLw_SetState_Unk1);
        (*fp).accessory4_cb = Some(cb_ftPk_SpecialLw_SpawnEffect);
    }
}

pub fn ftPk_SpecialLwStart_Anim(gobj: G) {
    unsafe { start_anim(gobj, 360) }
}

pub fn ftPk_SpecialAirLwStart_Anim(gobj: G) {
    unsafe { start_anim(gobj, 364) }
}

pub fn ftPk_SpecialLwLoop0_Anim(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).mv.pk.specialhi.x4 == 3 || (*fp).cmd_vars[0] != 0 {
            (*fp).take_dmg_cb = None;
            st::Fighter_ChangeMotionState(gobj, 362, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            return;
        }
        if ftPk_SpecialLw_8012765C(gobj) {
            let fighter_copy = fpp(gobj);
            st::Fighter_ChangeMotionState(gobj, 361, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            (*fighter_copy).cmd_vars[0] = 0;
            (*fighter_copy).take_dmg_cb = None;
            let fp = fpp(gobj);
            ext::efAsync_Spawn(
                gobj,
                &mut (*fp).x60C as *mut _ as *mut core::ffi::c_void,
                0,
                1216,
                (*(*fighter_copy).parts.add(FtPart_TopN as usize)).joint,
            );
        }
    }
}

pub fn ftPk_SpecialAirLwLoop0_Anim(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).mv.pk.specialhi.x4 == 3 || (*fp).cmd_vars[0] != 0 {
            (*fp).take_dmg_cb = None;
            st::Fighter_ChangeMotionState(gobj, 366, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            return;
        }
        if ftPk_SpecialLw_8012765C(gobj) {
            let fighter_copy = fpp(gobj);
            let pika_attr = da(fighter_copy);
            st::Fighter_ChangeMotionState(gobj, 365, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            (*fighter_copy).cmd_vars[0] = 0;
            (*fighter_copy).take_dmg_cb = None;
            (*fighter_copy).self_vel.y = (*pika_attr).xB4;
            let fp = fpp(gobj);
            ext::efAsync_Spawn(
                gobj,
                &mut (*fp).x60C as *mut _ as *mut core::ffi::c_void,
                0,
                1216,
                (*(*fighter_copy).parts.add(FtPart_TopN as usize)).joint,
            );
        }
    }
}
