//! src/melee/ft/kinds/ftMasterHand/ftmasterhandentry.c: Master Hand entry state and its camera
//! cut-scene callback (`ftMh_MS_343_801511FC`, a state machine over `mv.mh.unk4.x0`).
//! No fused sites (fma_sites.py). The Camera_ angle arguments are the C's double expressions
//! (`-M_PI / 9`, ...) evaluated in f64 and converted to the f32 parameter once.
//! Smoke-tested by tests/tu_ftmasterhandentry.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use core::f64::consts::PI as M_PI;
use melee_types::*;

/// Already ported (round 8): re-exported under this TU.


/// Calls into TUs that are not ported yet. Each is a settable hook.
pub mod ext {
    use melee_types::*;
    pub use crate::tu::ftmasterhanddamage0::ext::{ft_800881D8, ftMh_MS_389_80151018};
    crate::port_hook!(CAMERA_8002E6FC, Camera_8002E6FC, crate::adapt::Camera_8002E6FC, (arg0: i32));
    crate::port_hook!(CAMERA_8002ED9C, Camera_8002ED9C, crate::adapt::Camera_8002ED9C, (arg0: f32));
    crate::port_hook!(CAMERA_8002EEC8, Camera_8002EEC8, crate::adapt::Camera_8002EEC8, (arg0: f32));
    crate::port_hook!(CAMERA_8002EC7C, Camera_8002EC7C, crate::adapt::Camera_8002EC7C, (arg0: f32));
    crate::port_hook!(CAMERA_8002EB5C, Camera_8002EB5C, crate::adapt::Camera_8002EB5C, (arg0: f32));
    crate::port_hook!(CAMERA_8002EF14, Camera_8002EF14, crate::adapt::Camera_8002EF14, ());
    crate::port_hook!(CAMERA_8002F0E4, Camera_8002F0E4, crate::adapt::Camera_8002F0E4, (arg0: i32));
    crate::port_hook!(CAMERA_8002F260, Camera_8002F260, crate::adapt::Camera_8002F260, () -> bool);
    crate::port_hook!(CAMERA_8002F274, Camera_8002F274, crate::adapt::Camera_8002F274, ());
    crate::port_hook!(CAMERA_8002E948, Camera_8002E948, crate::adapt::Camera_8002E948, (cb: unsafe extern "C" fn(*mut Vec3) -> bool));
    crate::port_hook!(CAMERA_SETMODETOSTANDARD, Camera_SetModeToStandard, crate::adapt::Camera_SetModeToStandard, ());
    fn ftLib_8008627C_port(pos: *mut Vec3, gobj: *mut HSD_GObj) -> *mut HSD_GObj {
        melee_fighter::ftlib::ftlib_8008627c(unsafe { &*pos }, gobj)
    }
    fn ftLib_80086BE0_port(gobj: *mut HSD_GObj) -> u8 {
        melee_fighter::ftlib::ftlib_80086be0(gobj)
    }
    // Falls back to melee_fighter's port.
    crate::mh_hook_or!(FTLIB_8008627C, ftLib_8008627C, ftLib_8008627C_port, (pos: *mut Vec3, gobj: *mut HSD_GObj) -> *mut HSD_GObj);
    // Falls back to melee_fighter's port.
    crate::mh_hook_or!(FTLIB_80086BE0, ftLib_80086BE0, ftLib_80086BE0_port, (gobj: *mut HSD_GObj) -> u8);
    // Falls back to melee_fighter's port.
    crate::mh_hook_or!(FTBOSSLIB_8015C09C, ftBossLib_8015C09C, melee_fighter::tu::ftbosslib::ftBossLib_8015C09C, (gobj: *mut HSD_GObj, facing_dir: f32));
}

pub fn ftMh_MS_343_801510B0(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_343_801510B0");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        st::Fighter_ChangeMotionState(gobj, ftMh_MS_Entry as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).cur_pos.x = (*da).x30_pos2.x;
        (*fp).cur_pos.y = (*da).x30_pos2.y;
        (*fp).cur_pos.z = 0.0;
        (*fp).accessory4_cb = Some(ftMh_MS_343_801511FC);
        (*fp).mv.mh.unk4.x0 = ftMh_UnkEnum0_Unk00;
        ext::ft_800881D8(fp, 320001, 127, 64);
        ext::ftBossLib_8015C09C(gobj, -1.0);
    }
}

pub fn ftMh_Entry_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Entry_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        unsafe {
            let fp = (*gobj).user_data as *mut Fighter;
            (*fp).u.mh.x2258 = ftMh_MS_Wait1_0 as i32;
        }
        ext::ftMh_MS_389_80151018(gobj);
    }
}

pub fn ftMh_Entry_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_Entry_Phys");}

pub fn ftMh_Entry_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_Entry_Coll");}

/// Accessory-4 callback of the entry: the intro camera sequence (static in the C).
pub unsafe extern "C" fn ftMh_MS_343_801511FC(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_343_801511FC");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        match (*fp).mv.mh.unk4.x0 {
            ftMh_UnkEnum0_Unk00 => {
                let enemy_gobj = ext::ftLib_8008627C(&mut (*fp).cur_pos, gobj);
                if !enemy_gobj.is_null() {
                    ext::Camera_8002E6FC(ext::ftLib_80086BE0(enemy_gobj) as i32);
                } else {
                    ext::Camera_8002E6FC(0);
                }
                ext::Camera_8002ED9C(40.0);
                ext::Camera_8002EEC8(45.0);
                ext::Camera_8002EC7C((-M_PI) as f32);
                ext::Camera_8002EF14();
                ext::Camera_8002EC7C(0.0);
                ext::Camera_8002F0E4(120);
                (*fp).mv.mh.unk4.x0 = ftMh_UnkEnum0_Unk01;
            }
            ftMh_UnkEnum0_Unk01 => {
                if ext::Camera_8002F260() {
                    ext::Camera_8002E948(crate::tu::ftmasterhanddamage0::ftMh_MS_343_80151428);
                    ext::Camera_8002ED9C(120.0);
                    ext::Camera_8002F0E4(20);
                    (*fp).mv.mh.unk4.x0 = ftMh_UnkEnum0_Unk02;
                }
            }
            ftMh_UnkEnum0_Unk02 => {
                if ext::Camera_8002F260() {
                    ext::Camera_8002EC7C((M_PI / 2.0) as f32);
                    ext::Camera_8002F0E4(60);
                    (*fp).mv.mh.unk4.x0 = 3;
                }
            }
            ftMh_UnkEnum0_Unk03 => {
                if ext::Camera_8002F260() {
                    ext::Camera_8002ED9C(90.0);
                    ext::Camera_8002EC7C((-M_PI / 9.0) as f32);
                    ext::Camera_8002EB5C((-M_PI / 12.0) as f32);
                    ext::Camera_8002EF14();
                    (*fp).mv.mh.unk4.x0 = ftMh_UnkEnum0_Unk04;
                }
            }
            ftMh_UnkEnum0_Unk04 => {
                ext::Camera_8002F274();
                (*fp).mv.mh.unk4.x8 = 120;
                (*fp).mv.mh.unk4.x0 = ftMh_UnkEnum0_Unk05;
            }
            ftMh_UnkEnum0_Unk05 => {
                (*fp).mv.mh.unk4.x8 = (*fp).mv.mh.unk4.x8.wrapping_sub(1);
                if (*fp).mv.mh.unk4.x8 == 0 {
                    ext::Camera_8002EC7C((-M_PI / 9.0) as f32);
                    ext::Camera_8002EF14();
                    ext::Camera_8002ED9C(120.0);
                    (*fp).mv.mh.unk4.x8 = 30;
                    (*fp).mv.mh.unk4.x0 = ftMh_UnkEnum0_Unk06;
                }
            }
            ftMh_UnkEnum0_Unk06 => {
                (*fp).mv.mh.unk4.x8 = (*fp).mv.mh.unk4.x8.wrapping_sub(1);
                if (*fp).mv.mh.unk4.x8 == 0 {
                    ext::Camera_8002EC7C((M_PI / 9.0) as f32);
                    ext::Camera_8002EF14();
                    (*fp).mv.mh.unk4.x8 = 30;
                    (*fp).mv.mh.unk4.x0 = ftMh_UnkEnum0_Unk07;
                }
            }
            ftMh_UnkEnum0_Unk07 => {
                (*fp).mv.mh.unk4.x8 = (*fp).mv.mh.unk4.x8.wrapping_sub(1);
                if (*fp).mv.mh.unk4.x8 == 0 {
                    ext::Camera_8002ED9C(180.0);
                    ext::Camera_8002EC7C((-M_PI / 2.0) as f32);
                    ext::Camera_8002EB5C((-M_PI / 9.0) as f32);
                    ext::Camera_8002EF14();
                    (*fp).mv.mh.unk4.x8 = 50;
                    (*fp).mv.mh.unk4.x0 = ftMh_UnkEnum0_Unk08;
                }
            }
            ftMh_UnkEnum0_Unk08 => {
                ext::Camera_8002F274();
                (*fp).mv.mh.unk4.x0 = ftMh_UnkEnum0_Unk09;
            }
            ftMh_UnkEnum0_Unk09 => {
                (*fp).mv.mh.unk4.x8 = (*fp).mv.mh.unk4.x8.wrapping_sub(1);
                if (*fp).mv.mh.unk4.x8 == 0 {
                    ext::Camera_SetModeToStandard();
                    (*fp).mv.mh.unk4.x0 = ftMh_UnkEnum0_Unk10;
                }
            }
            _ => {}
        }
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftMh_Entry_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandentry.c).
pub fn ftMh_Entry_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Entry_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}
