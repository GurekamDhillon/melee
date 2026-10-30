//! src/melee/ft/kinds/ftKirby/ftkirbyspecialsamus.c: Kirby's Charge Shot (Samus copy ability).
//! No fused sites (tools/rust-port/fma_sites.py). The charge-level sfx index and the recoil are
//! checked by tests/tu_ftkirbyspecialsamus.rs, which also smoke-tests the states.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::missing_safety_doc)]

use crate::stubs as st;
use crate::tu::ftkirbyspecialn::{change_ms, fpp, ftKb_SpecialN_set_cbs, G};
use melee_fighter::hooks as fh;
use melee_types::*;

/// Already ported (rounds 2, 4, 11): re-exported under this TU.
pub use crate::chars::r11::ftKirby::ftkirbyspecialsamus::{
    ftKb_SpecialNSs_800FCF74, ftKb_SpecialNSs_800FD020, ftKb_SsSpecialAirNStart_Coll, ftKb_SsSpecialAirN_Coll,
    ftKb_SsSpecialNCancel_Coll, ftKb_SsSpecialNStart_Coll, ftKb_SsSpecialN_Coll,
};
pub use crate::chars::r2::ftKirby::ftkirbyspecialsamus::{
    ftKb_SsSpecialAirNStart_Phys, ftKb_SsSpecialAirN_Phys, ftKb_SsSpecialNCancel_Phys,
    ftKb_SsSpecialNHold_Phys, ftKb_SsSpecialNStart_Phys, ftKb_SsSpecialN_Phys,
};
pub use crate::chars::r4::ftKirby::ftkirbyspecialsamus::ftKb_SpecialNSs_UnsetChargeShot;

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
    type G = *mut HSD_GObj;
    pub use crate::tu::ftcaptainspecialn::ext::{efSync_Spawn, EFSYNC_SPAWN};
    pub use crate::tu::ftkirbyspecialmewtwo::ext::{ftCo_800BFFD0, ftCo_8009917C, ft_80088510, FTCO_800BFFD0, FTCO_8009917C, FT_80088510};
    pub use crate::tu::ftkirbyspecialn::ext::{lb_8000B1CC, LB_8000B1CC};
    pub static mut IT_802B5974: Option<fn(*mut c_void) > = None;
            #[allow(non_snake_case)]
            pub fn it_802B5974(item: *mut c_void)  {
                match unsafe { IT_802B5974 } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::itsamuschargeshot::it_802B5974(item as _); } return; },
                }
            }
    pub static mut IT_802B56E4: Option<fn(*mut c_void, *const Vec3, f32, f32, f32) > = None;
            #[allow(non_snake_case)]
            pub fn it_802B56E4(item: *mut c_void, pos: *const Vec3, angle: f32, charge: f32, max: f32)  {
                match unsafe { IT_802B56E4 } {
                    Some(f) => f(item, pos, angle, charge, max),
                    None => { unsafe { let _ = melee_items::tu::itsamuschargeshot::it_802B56E4(item as _, pos as _, angle as _, charge as _, max as _); } return; },
                }
            }
    pub static mut IT_802B55C8: Option<fn(G, *const Vec3, i32, i32, f32) -> *mut c_void> = None;
            #[allow(non_snake_case)]
            pub fn it_802B55C8(g: G, pos: *const Vec3, part: i32, kind: i32, dir: f32) -> *mut c_void {
                match unsafe { IT_802B55C8 } {
                    Some(f) => f(g, pos, part, kind, dir),
                    None => { return unsafe { melee_items::tu::itsamuschargeshot::it_802B55C8(g as _, pos as _, part as _, kind as _, dir as _) as _ }; },
                }
            }
}

/// `ftKb_Init_803CB6C0`: charge-level sfx ids (the last entry is the sentinel the index never reaches
/// except at exactly full charge).
pub static ftKb_Init_803CB6C0: [u32; 6] = [260006, 260009, 260012, 260015, 260018, 0];

/// Charge Shot status readout for the HUD. Returns -1 when no shot is charging.
pub fn ftKb_SpecialNSs_800FCC14(gobj: G, out1: &mut i32, out2: &mut i32) -> i32 {
    melee_core::ct!("ftKb_SpecialNSs_800FCC14");
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            let da = &*((*fp).dat_attrs as *const ftKb_DatAttrs);
            if (*fp).u.kb.xA4.is_null() {
                return -1;
            }
            *out1 = (*fp).u.kb.xA8;
            *out2 = da.specialn_ss_charge_time as i32;
            return 0;
        }
        -1
    }
}

pub fn ftKb_SpecialNSs_800FCC6C(gobj: G) -> bool {
    melee_core::ct!("ftKb_SpecialNSs_800FCC6C");
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            return match (*fp).motion_id as i32 {
                0x197..=0x19C => (*fp).x2070.anon_0.x2071_b6() != 0,
                _ => true,
            };
        }
        true
    }
}

pub fn ftKb_SpecialNSs_800FCCBC(gobj: G) -> bool {
    melee_core::ct!("ftKb_SpecialNSs_800FCCBC");
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            return !matches!((*fp).motion_id as i32, 0x197 | 0x198 | 0x19A | 0x19B | 0x19C);
        }
        true
    }
}

/// `ftKb_SpecialNSs_it_802B5974` (static inline): release the charge-shot item.
pub fn ftKb_SpecialNSs_it_802B5974(gobj: G) {
    melee_core::ct!("ftKb_SpecialNSs_it_802B5974");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.kb.xA4.is_null() {
            ext::it_802B5974((*fp).u.kb.xA4);
            (*fp).u.kb.xA4 = core::ptr::null_mut();
        }
    }
}

/// `ftKb_SpecialNSs_DestroyChargeShot` (static inline).
pub fn ftKb_SpecialNSs_DestroyChargeShot(gobj: G) {
    melee_core::ct!("ftKb_SpecialNSs_DestroyChargeShot");
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            st::efLib_DestroyAll(gobj);
            (*fp).u.kb.xAC = core::ptr::null_mut();
        }
    }
}

/// `ftKb_ChargeShot_inline` (static inline).
pub fn ftKb_ChargeShot_inline(gobj: G) {
    melee_core::ct!("ftKb_ChargeShot_inline");
    if !gobj.is_null() {
        ftKb_SpecialNSs_it_802B5974(gobj);
        ftKb_SpecialNSs_DestroyChargeShot(gobj);
    }
}

pub fn ftKb_SpecialNSs_800FCD04(gobj: G) {
    melee_core::ct!("ftKb_SpecialNSs_800FCD04");
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            if !(*fp).u.kb.xA4.is_null() {
                (*fp).u.kb.xA4 = core::ptr::null_mut();
            }
            ftKb_SpecialNSs_DestroyChargeShot(gobj);
        }
    }
}

pub fn ftKb_SpecialNSs_800FCD60(gobj: G) {
    melee_core::ct!("ftKb_SpecialNSs_800FCD60");
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            ftKb_ChargeShot_inline(gobj);
            (*fp).u.kb.xA8 = 0;
        }
    }
}

/// `ftKb_SpecialNSs_800FCDE0_inline` (static inline): aerial shot recoil.
pub fn ftKb_SpecialNSs_800FCDE0_inline(gobj: G) {
    melee_core::ct!("ftKb_SpecialNSs_800FCDE0_inline");
    unsafe {
        let fp = fpp(gobj);
        let da = &*((*fp).dat_attrs as *const ftKb_DatAttrs);
        (*fp).self_vel.x = (*fp).facing_dir * (da.specialn_ss_aerial_shot_recoil * (*fp).u.kb.xA8 as f32);
    }
}

/// `facing_to_angle` (static inline).
pub fn facing_to_angle(fp: &Fighter) -> f64 {
    melee_core::ct!("facing_to_angle");
    if fp.facing_dir == 1.0 {
        0.0
    } else {
        core::f64::consts::PI
    }
}

/// Fires the charged shot (cmd_vars[1] == 1).
pub fn ftKb_SpecialNSs_800FCDE0(gobj: G) {
    melee_core::ct!("ftKb_SpecialNSs_800FCDE0");
    unsafe {
        let fp = fpp(gobj);
        let da = &*((*fp).dat_attrs as *const ftKb_DatAttrs);
        if (*fp).cmd_vars[1] == 1 && !(*fp).u.kb.xA4.is_null() {
            (*fp).cmd_vars[1] = 2;
            let offset = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            ext::lb_8000B1CC((*(*fp).parts.add(FtPart_R3rdNa as usize)).joint as *mut _, &offset, &mut pos);
            pos.z = 0.0;
            let saved_item = (*fp).item_gobj;
            ext::it_802B56E4((*fp).u.kb.xA4, &pos, facing_to_angle(&*fp) as f32, (*fp).u.kb.xA8 as u32 as f32, da.specialn_ss_charge_time);
            if (*fp).motion_id == ftKb_MS_SsSpecialAirN as FtMotionId || (*fp).ground_or_air == GA_Air {
                ftKb_SpecialNSs_800FCDE0_inline(gobj);
            }
            (*fp).u.kb.xA8 = 0;
            if !gobj.is_null() {
                ftKb_SpecialNSs_UnsetChargeShot(gobj);
                ftKb_SpecialNSs_DestroyChargeShot(gobj);
            }
            let facing = &mut (*fp).facing_dir as *mut f32 as *mut core::ffi::c_void;
            ext::efSync_Spawn(0x4A4, gobj, &[&mut pos as *mut Vec3 as *mut core::ffi::c_void, facing]);
            (*fp).item_gobj = saved_item;
        }
    }
}

/// Spawn the charge item at the hand when the cmd says so (shared by the Start states).
unsafe fn spawn_charge(gobj: G, fp: *mut Fighter) {
    if (*fp).cmd_vars[0] == 1 && (*fp).u.kb.xA4.is_null() {
        (*fp).cmd_vars[0] = 0;
        let offset = Vec3 { x: 0.0, y: 0.0, z: 4.0 };
        let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ext::lb_8000B1CC((*(*fp).parts.add(FtPart_R3rdNa as usize)).joint as *mut _, &offset, &mut pos);
        pos.z = 0.0;
        (*fp).u.kb.xA4 = ext::it_802B55C8(gobj, &pos, FtPart_R3rdNa as i32, It_Kind_Kirby_SamusCharge as i32, (*fp).facing_dir);
        if !(*fp).u.kb.xA4.is_null() {
            ftKb_SpecialN_set_cbs(gobj);
        } else {
            (*fp).u.kb.xA4 = core::ptr::null_mut();
        }
    }
}

pub fn ftKb_SsSpecialNStart_Anim(gobj: G) {
    melee_core::ct!("ftKb_SsSpecialNStart_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = &*((*fp).dat_attrs as *const ftKb_DatAttrs);
        spawn_charge(gobj, fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            if (*fp).mv.kb.specialhi.x0 == 1 || (*fp).u.kb.xA8 as f32 == da.specialn_ss_charge_time {
                change_ms(gobj, ftKb_MS_SsSpecialN as i32, 0);
            } else {
                change_ms(gobj, ftKb_MS_SsSpecialNHold as i32, 0);
                (*fpp(gobj)).u.kb.xAC = core::ptr::null_mut();
            }
            ftKb_SpecialN_set_cbs(gobj);
        }
    }
}

/// The charge-level index into `ftKb_Init_803CB6C0`: `(int)(5.0f * ratio)`.
pub fn charge_sfx_index(charge: i32, charge_time: f32) -> usize {
    melee_core::ct!("charge_sfx_index");
    let ratio = if charge != 0 { charge as f32 / charge_time } else { 0.0 };
    ((5.0f32 * ratio) as i32) as usize
}

pub fn ftKb_SsSpecialNHold_Anim(gobj: G) {
    melee_core::ct!("ftKb_SsSpecialNHold_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = &*((*fp).dat_attrs as *const ftKb_DatAttrs);
        if (*fp).cmd_vars[2] != 0 {
            (*fp).cmd_vars[2] = 0;
            let i = charge_sfx_index((*fp).u.kb.xA8, da.specialn_ss_charge_time);
            crate::tu::ftkirbyspecialmewtwo::ext::ft_80088510(fp, ftKb_Init_803CB6C0[i], 0x7F, 0x40);
        }
        (*fp).mv.kb.specialhi.x4 += 1;
        if (*fp).mv.kb.specialhi.x4 > da.specialn_ss_frames_per_charge_level {
            (*fp).mv.kb.specialhi.x4 = 0;
            (*fp).u.kb.xA8 += 1;
            if (*fp).u.kb.xA8 as f32 >= da.specialn_ss_charge_time {
                ext::ftCo_800BFFD0(fp, 0x36, 0);
                (*fp).u.kb.xA8 = da.specialn_ss_charge_time as i32;
                change_ms(gobj, ftKb_MS_SsSpecialNCancel as i32, 0);
                ftKb_ChargeShot_inline(gobj);
                ftKb_SpecialN_set_cbs(gobj);
            }
        }
    }
}

pub fn ftKb_SsSpecialNCancel_Anim(gobj: G) {
    melee_core::ct!("ftKb_SsSpecialNCancel_Anim");
    ftKb_ChargeShot_inline(gobj);
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ft_8008A2BC(gobj);
    }
}

pub fn ftKb_SsSpecialN_Anim(gobj: G) {
    melee_core::ct!("ftKb_SsSpecialN_Anim");
    ftKb_SpecialNSs_800FCDE0(gobj);
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ft_8008A2BC(gobj);
    }
}

pub fn ftKb_SsSpecialAirNStart_Anim(gobj: G) {
    melee_core::ct!("ftKb_SsSpecialAirNStart_Anim");
    unsafe {
        let fp = fpp(gobj);
        spawn_charge(gobj, fp);
        (*fp).mv.kb.specialhi.x0 = 1;
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            change_ms(gobj, ftKb_MS_SsSpecialAirN as i32, 0);
            ftKb_SpecialN_set_cbs(gobj);
        }
    }
}

pub fn ftKb_SsSpecialAirN_Anim(gobj: G) {
    melee_core::ct!("ftKb_SsSpecialAirN_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = &*((*fp).dat_attrs as *const ftKb_DatAttrs);
        ftKb_SpecialNSs_800FCDE0(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            if da.specialn_ss_freefall_toggle == 0.0 {
                st::ftCo_Fall_Enter(gobj);
                return;
            }
            st::ftCo_80096900(gobj, 1, 0, 1, 1.0, da.specialn_ss_freefall_toggle);
        }
    }
}

pub fn ftKb_SsSpecialNStart_IASA(_gobj: G) { melee_core::ct!("ftKb_SsSpecialNStart_IASA");}

pub fn ftKb_SsSpecialNHold_IASA(gobj: G) {
    melee_core::ct!("ftKb_SsSpecialNHold_IASA");
    unsafe {
        let fp = fpp(gobj);
        if ext::ftCo_8009917C(gobj) {
            ftKb_ChargeShot_inline(gobj);
        } else {
            if ((*fp).input.pressed_buttons & 0x200) != 0 {
                change_ms(gobj, 0x19A, 0);
                ftKb_SpecialN_set_cbs(gobj);
                return;
            }
            if ((*fp).input.pressed_buttons & 0x8000_0000) != 0 {
                change_ms(gobj, 0x199, 0);
                ftKb_ChargeShot_inline(gobj);
                ftKb_SpecialN_set_cbs(gobj);
            }
        }
    }
}

pub fn ftKb_SsSpecialNCancel_IASA(_gobj: G) { melee_core::ct!("ftKb_SsSpecialNCancel_IASA");}
pub fn ftKb_SsSpecialN_IASA(_gobj: G) { melee_core::ct!("ftKb_SsSpecialN_IASA");}
pub fn ftKb_SsSpecialAirNStart_IASA(_gobj: G) { melee_core::ct!("ftKb_SsSpecialAirNStart_IASA");}
pub fn ftKb_SsSpecialAirN_IASA(_gobj: G) { melee_core::ct!("ftKb_SsSpecialAirN_IASA");}

pub fn ftKb_SsSpecialNHold_Coll(gobj: G) {
    melee_core::ct!("ftKb_SsSpecialNHold_Coll");
    unsafe {
        let fp = fpp(gobj);
        if st::ft_80082708(gobj) == GA_Ground as i32 {
            st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, ftKb_MS_SsSpecialAirN as i32, crate::tu::ftkirbyspecialn::ftCommon_GroundAirColl_MF);
            ftKb_SpecialN_set_cbs(gobj);
            fh::play_sfx(&mut *fp, 0x3F7B5, 0x7F, 0x40);
            (*fp).cmd_vars[1] = 1;
        }
    }
}
