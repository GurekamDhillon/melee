//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandgrabunk1b174.c: Crazy Hand tag-fail ending
//! cutscene (camera script in `fn_8015B2C0`). The camera angles are `M_PI` fractions computed in
//! double and narrowed to float, as the C does. Smoke-tested by tests/tu_ftcrazyhandgrabunk1b174.rs.
#![allow(non_snake_case, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use crate::tu::ftcrazyhandfingerbeam0::ext::{ftCh_GrabUnk1_8015BC88, lb_8000B1CC};
use melee_types::*;

/// Already ported (round 8): re-exported under this TU.


/// Calls into TUs that are not ported yet (cm/camera.c). Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    crate::port_hook!(CAMERA_8002E6FC, Camera_8002E6FC, crate::adapt::Camera_8002E6FC, (a: i32));
    crate::port_hook!(CAMERA_8002E948, Camera_8002E948, crate::adapt::Camera_8002E948_rs, (cb: fn(*mut Vec3) -> bool));
    crate::port_hook!(CAMERA_8002EB5C, Camera_8002EB5C, crate::adapt::Camera_8002EB5C, (a: f32));
    crate::port_hook!(CAMERA_8002EC7C, Camera_8002EC7C, crate::adapt::Camera_8002EC7C, (a: f32));
    crate::port_hook!(CAMERA_8002ED9C, Camera_8002ED9C, crate::adapt::Camera_8002ED9C, (a: f32));
    crate::port_hook!(CAMERA_8002EEC8, Camera_8002EEC8, crate::adapt::Camera_8002EEC8, (a: f32));
    crate::port_hook!(CAMERA_8002EF14, Camera_8002EF14, crate::adapt::Camera_8002EF14, ());
    crate::port_hook!(CAMERA_8002F0E4, Camera_8002F0E4, crate::adapt::Camera_8002F0E4, (a: i32));
    crate::port_hook!(CAMERA_8002F260, Camera_8002F260, crate::adapt::Camera_8002F260, () -> bool);
    crate::port_hook!(CAMERA_8002F274, Camera_8002F274, crate::adapt::Camera_8002F274, ());
    crate::port_hook!(CAMERA_SETMODETOSTANDARD, Camera_SetModeToStandard, crate::adapt::Camera_SetModeToStandard, ());
}

const M_PI: f64 = 3.14159265358979323846;

pub fn ftCh_GrabUnk1_8015B174(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_GrabUnk1_8015B174");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
        st::Fighter_ChangeMotionState(gobj, 0x183, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).cur_pos.x = (*da).x18;
        (*fp).cur_pos.y = (*da).x1C;
        (*fp).cur_pos.z = 0.0;
        (*fp).accessory4_cb = Some(fn_8015B2C0);
        (*fp).mv.ch.unk4.x0 = 0;
        melee_fighter::ft0881::ft_800881d8(&mut *fp, 320002, 127, 64);
        melee_fighter::tu::ftbosslib::ftBossLib_8015C09C(gobj, 1.0);
    }
}

pub fn ftCh_TagFail_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_TagFail_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        unsafe {
            let fp = (*gobj).user_data as *mut Fighter;
            (*fp).u.mh.x2258 = ftMh_MS_Wait1_0 as i32;
        }
        ftCh_GrabUnk1_8015BC88(gobj);
    }
}

pub fn ftCh_TagFail_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_TagFail_Phys");}

pub fn ftCh_TagFail_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_TagFail_Coll");}

/// Accessory-4 callback: the camera script, one step per frame, keyed by `mv.ch.unk4.x0`.
pub unsafe extern "C" fn fn_8015B2C0(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8015B2C0");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        match (*fp).mv.ch.unk4.x0 {
            0 => {
                let enemy = melee_fighter::ftlib::ftlib_findnearestopponent(&(*fp).cur_pos, gobj);
                if !enemy.is_null() {
                    ext::Camera_8002E6FC(melee_fighter::ftlib::ftlib_getplayerindex(enemy) as i32);
                } else {
                    ext::Camera_8002E6FC(0);
                }
                ext::Camera_8002ED9C(40.0);
                ext::Camera_8002EEC8(45.0);
                ext::Camera_8002EC7C((-M_PI) as f32);
                ext::Camera_8002EF14();
                ext::Camera_8002EC7C(0.0);
                ext::Camera_8002F0E4(120);
                (*fp).mv.ch.unk4.x0 = 1;
            }
            1 => {
                if ext::Camera_8002F260() {
                    ext::Camera_8002E948(fn_8015B4EC);
                    ext::Camera_8002ED9C(120.0);
                    ext::Camera_8002F0E4(20);
                    (*fp).mv.ch.unk4.x0 = 2;
                }
            }
            2 => {
                if ext::Camera_8002F260() {
                    ext::Camera_8002EC7C((M_PI / 2.0) as f32);
                    ext::Camera_8002F0E4(60);
                    (*fp).mv.ch.unk4.x0 = 3;
                }
            }
            3 => {
                if ext::Camera_8002F260() {
                    ext::Camera_8002ED9C(90.0);
                    ext::Camera_8002EC7C((-M_PI / 9.0) as f32);
                    ext::Camera_8002EB5C((-M_PI / 12.0) as f32);
                    ext::Camera_8002EF14();
                    (*fp).mv.ch.unk4.x0 = 4;
                }
            }
            4 => {
                ext::Camera_8002F274();
                (*fp).mv.ch.unk4.x8 = 120;
                (*fp).mv.ch.unk4.x0 = 5;
            }
            5 => {
                (*fp).mv.ch.unk4.x8 -= 1;
                if (*fp).mv.ch.unk4.x8 == 0 {
                    ext::Camera_8002EC7C((-M_PI / 9.0) as f32);
                    ext::Camera_8002EF14();
                    ext::Camera_8002ED9C(120.0);
                    (*fp).mv.ch.unk4.x8 = 30;
                    (*fp).mv.ch.unk4.x0 = 6;
                }
            }
            6 => {
                (*fp).mv.ch.unk4.x8 -= 1;
                if (*fp).mv.ch.unk4.x8 == 0 {
                    ext::Camera_8002EC7C((M_PI / 9.0) as f32);
                    ext::Camera_8002EF14();
                    (*fp).mv.ch.unk4.x8 = 30;
                    (*fp).mv.ch.unk4.x0 = 7;
                }
            }
            7 => {
                (*fp).mv.ch.unk4.x8 -= 1;
                if (*fp).mv.ch.unk4.x8 == 0 {
                    ext::Camera_8002ED9C(180.0);
                    ext::Camera_8002EC7C((-M_PI / 2.0) as f32);
                    ext::Camera_8002EB5C((-M_PI / 9.0) as f32);
                    ext::Camera_8002EF14();
                    (*fp).mv.ch.unk4.x8 = 50;
                    (*fp).mv.ch.unk4.x0 = 8;
                }
            }
            8 => {
                ext::Camera_8002F274();
                (*fp).mv.ch.unk4.x0 = 9;
            }
            9 => {
                (*fp).mv.ch.unk4.x8 -= 1;
                if (*fp).mv.ch.unk4.x8 == 0 {
                    ext::Camera_SetModeToStandard();
                    (*fp).mv.ch.unk4.x0 = 10;
                }
            }
            _ => {}
        }
    }
}

/// Camera follow callback: the Crazy Hand waist position.
pub fn fn_8015B4EC(vec: *mut Vec3) -> bool {
    melee_core::ct!("fn_8015B4EC");
    unsafe {
        let gobj = melee_fighter::tu::ftbosslib::ftBossLib_GetFighterGObj(Ft_Kind_CrezyH);
        if !gobj.is_null() {
            let fp = (*gobj).user_data as *mut Fighter;
            lb_8000B1CC((*(*fp).parts.add(FtPart_WaistN as usize)).joint, core::ptr::null(), vec);
            return true;
        }
        false
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftCh_TagFail_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandgrabunk1b174.c).
pub fn ftCh_TagFail_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_TagFail_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}
