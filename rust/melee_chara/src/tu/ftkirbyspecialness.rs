//! src/melee/ft/kinds/ftKirby/ftkirbyspecialness.c: Kirby's PK Flash (Ness copy ability).
//! One fused site (tools/rust-port/fma_sites.py): the flash height `y += scale.y * 3.0` in
//! `ftKb_NsSpecialNStart_Anim_inline`. Smoke-tested by tests/tu_ftkirbyspecialness.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::missing_safety_doc)]

use crate::stubs as st;
use crate::tu::ftkirbyspecialn::{change_ms, fpp, ftKb_SpecialN_set_cbs, G};
use melee_core::fp::fmadds;
use melee_types::*;

/// Already ported (rounds 2, 4, 6, root module): re-exported under this TU.
pub use crate::chars::r2::ftKirby::ftkirbyspecialness::{
    ftKb_NsSpecialAirNEnd_Coll, ftKb_NsSpecialAirNHold_Coll, ftKb_NsSpecialAirNStart_Coll,
    ftKb_NsSpecialNEnd_Coll, ftKb_NsSpecialNEnd_Phys, ftKb_NsSpecialNHold_Coll, ftKb_NsSpecialNHold_Phys,
    ftKb_NsSpecialNStart_Coll,
};
pub use crate::chars::r4::ftKirby::ftkirbyspecialness::ftKb_SetNsFlashAttr;
pub use crate::chars::r6::ftKirby::ftkirbyspecialness::ftKb_NsSpecialNStart_Phys;

/// Calls into TUs that are not ported yet (or ported elsewhere: the default then calls the port).
pub mod ext {
    use melee_types::*;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    type G = *mut HSD_GObj;
    pub use crate::tu::ftkirbyspecialn::ext::{lb_8000B1CC, LB_8000B1CC};
    pub static mut IT_802AA8C0: Option<fn(G, *const Vec3, i32, f32) -> *mut Item_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_802AA8C0(g: G, pos: *const Vec3, kind: i32, dir: f32) -> *mut Item_GObj {
                match unsafe { IT_802AA8C0 } {
                    Some(f) => f(g, pos, kind, dir),
                    None => { return unsafe { melee_items::tu::itnesspkflash::it_802AA8C0(g as _, pos as _, kind as _, dir as _) as _ }; },
                }
            }
    pub static mut IT_802AA7E4: Option<fn(*mut Item_GObj) -> G> = None;
            #[allow(non_snake_case)]
            pub fn it_802AA7E4(item: *mut Item_GObj) -> G {
                match unsafe { IT_802AA7E4 } {
                    Some(f) => f(item),
                    None => { return unsafe { melee_items::tu::itnesspkflash::it_802AA7E4(item as _) as _ }; },
                }
            }
    pub static mut IT_802AA7F0: Option<fn(*mut Item_GObj) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn it_802AA7F0(item: *mut Item_GObj) -> i32 {
                match unsafe { IT_802AA7F0 } {
                    Some(f) => f(item),
                    None => { return unsafe { melee_items::tu::itnesspkflash::it_802AA7F0(item as _) as _ }; },
                }
            }
    pub static mut FTPARTSETROTX: Option<fn(*mut Fighter, i32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn ftPartSetRotX(fp: *mut Fighter, part_idx: i32, rot: f32) {
        match unsafe { FTPARTSETROTX } {
            Some(f) => f(fp, part_idx, rot),
            None => { unsafe { melee_fighter::tu::ftparts::ftPartSetRotX(&*(fp as *const _), part_idx as _, rot as _); } },
        }
    }
}

const HSD_PAD_B_U: u32 = HSD_PAD_B as u32;

/// Reset the PK Flash timers from the attrs (the Ground entry; `da` can't move below `fp` in the C).
unsafe fn init_flash_attrs(fp: *mut Fighter) {
    let da = &*((*fp).dat_attrs as *const ftKb_DatAttrs);
    (*fp).mv.ns.specialn.frames_to_loop_charge_ground = da.specialn_ns_frames_to_loop_charge_ground;
    (*fp).mv.ns.specialn.frames_to_loop_charge_air = da.specialn_ns_frames_to_loop_charge_air;
    (*fp).mv.ns.specialn.falling_acceleration_delay = da.specialn_ns_falling_acceleration_delay;
    (*fp).u.kb.ns_flash_gobj = core::ptr::null_mut();
    (*fp).mv.ns.specialn.charge_release_delay = da.specialn_ns_charge_release_delay;
}

/// `ftKb_NsSpecialNStart_Anim_inline` (static): spawn the PK Flash item at the waist.
pub fn ftKb_NsSpecialNStart_Anim_inline(gobj: G, flash_pos: &mut Vec3) {
    melee_core::ct!("ftKb_NsSpecialNStart_Anim_inline");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).u.kb.ns_flash_gobj.is_null() {
            ext::lb_8000B1CC((*(*fp).parts.add(FtPart_WaistN as usize)).joint as *mut _, core::ptr::null(), flash_pos);
            flash_pos.z = 0.0;
            flash_pos.y = fmadds((*fp).x34_scale.y, 3.0, flash_pos.y);
            let flash_gobj = ext::it_802AA8C0(gobj, flash_pos, It_Kind_Kirby_NessPKFlush as i32, (*fp).facing_dir);
            (*fp).u.kb.ns_flash_gobj = flash_gobj;
            if !flash_gobj.is_null() {
                ftKb_SpecialN_set_cbs(gobj);
            }
        }
    }
}

pub fn ftKb_SpecialNNs_800FED38(gobj: G) {
    melee_core::ct!("ftKb_SpecialNNs_800FED38");
    unsafe {
        let fp = fpp(gobj);
        change_ms(gobj, ftKb_MS_NsSpecialNStart as i32, Ft_MF_None as u32);
        st::Fighter_ClearCmdVars(&mut *fp);
        let fp = fpp(gobj);
        init_flash_attrs(fp);
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftKb_SpecialNNs_800FEDD0(gobj: G) {
    melee_core::ct!("ftKb_SpecialNNs_800FEDD0");
    unsafe {
        let fp = fpp(gobj);
        change_ms(gobj, ftKb_MS_NsSpecialAirNStart as i32, Ft_MF_None as u32);
        st::Fighter_ClearCmdVars(&mut *fp);
        (*fp).self_vel.y = 0.0;
        let fp = fpp(gobj);
        init_flash_attrs(fp);
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftKb_NsSpecialNStart_Anim(gobj: G) {
    melee_core::ct!("ftKb_NsSpecialNStart_Anim");
    unsafe {
        let fp = fpp(gobj);
        let mut flash_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            change_ms(gobj, 436, 0);
            ftKb_NsSpecialNStart_Anim_inline(gobj, &mut flash_pos);
            (*fp).x1968_jumpsUsed = (*fp).co_attrs.max_jumps as u8;
        }
    }
}

pub fn ftKb_NsSpecialAirNStart_Anim(gobj: G) {
    melee_core::ct!("ftKb_NsSpecialAirNStart_Anim");
    unsafe {
        let fp = fpp(gobj);
        let mut flash_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            change_ms(gobj, ftKb_MS_NsSpecialAirNHold0 as i32, 0);
            ftKb_NsSpecialNStart_Anim_inline(gobj, &mut flash_pos);
            (*fp).x1968_jumpsUsed = (*fp).co_attrs.max_jumps as u8;
        }
    }
}

/// Hold timers shared by the ground and air Hold states: `end_ms` when the charge ran out,
/// `hold1_ms` once the flash is hit.
unsafe fn hold_anim(gobj: G, end_ms: i32, hold1_ms: i32) {
    let fp = fpp(gobj);
    let flash_timer_min = (*fp).mv.ns.specialn.frames_to_loop_charge_ground;
    if flash_timer_min != 0 {
        (*fp).mv.ns.specialn.frames_to_loop_charge_ground = flash_timer_min - 1;
    }
    if (*fp).u.kb.ns_flash_gobj.is_null() {
        let flash_timer_delay = (*fp).mv.ns.specialn.frames_to_loop_charge_air;
        if flash_timer_delay != 0 {
            (*fp).mv.ns.specialn.frames_to_loop_charge_air = flash_timer_delay - 1;
        }
    }
    if (*fp).u.kb.ns_flash_gobj.is_null() {
        if (*fp).mv.ns.specialn.frames_to_loop_charge_ground <= 0 && (*fp).mv.ns.specialn.frames_to_loop_charge_air <= 0 {
            st::Fighter_ChangeMotionState(gobj, end_ms, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            return;
        }
        if (*fp).motion_id != hold1_ms as FtMotionId {
            st::Fighter_ChangeMotionState(gobj, hold1_ms, 0, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
        }
    } else {
        if ext::it_802AA7E4((*fp).u.kb.ns_flash_gobj) != gobj {
            (*fp).u.kb.ns_flash_gobj = core::ptr::null_mut();
            return;
        }
        if ext::it_802AA7F0((*fp).u.kb.ns_flash_gobj) == 1 && (*fp).motion_id != hold1_ms as FtMotionId {
            st::Fighter_ChangeMotionState(gobj, hold1_ms, 0, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
        }
    }
}

pub fn ftKb_NsSpecialNHold_Anim(gobj: G) {
    melee_core::ct!("ftKb_NsSpecialNHold_Anim");
    unsafe { hold_anim(gobj, ftKb_MS_NsSpecialNEnd as i32, ftKb_MS_NsSpecialNHold1 as i32) }
}

pub fn ftKb_NsSpecialAirNHold_Anim(gobj: G) {
    melee_core::ct!("ftKb_NsSpecialAirNHold_Anim");
    unsafe { hold_anim(gobj, ftKb_MS_NsSpecialAirNEnd as i32, ftKb_MS_NsSpecialAirNHold1 as i32) }
}

pub fn ftKb_NsSpecialNEnd_Anim(gobj: G) {
    melee_core::ct!("ftKb_NsSpecialNEnd_Anim");
    unsafe {
        let fp = fpp(gobj);
        ftKb_SetNsFlashAttr(gobj);
        ext::ftPartSetRotX(fp, 0, 0.0);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftKb_NsSpecialAirNEnd_Anim(gobj: G) {
    melee_core::ct!("ftKb_NsSpecialAirNEnd_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = &*((*fp).dat_attrs as *const ftKb_DatAttrs);
        ftKb_SetNsFlashAttr(gobj);
        ext::ftPartSetRotX(fp, 0, 0.0);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            if da.specialn_ns_freefall_toggle == 0.0 {
                st::ftCo_Fall_Enter(gobj);
            } else {
                st::ftCo_80096900(gobj, 1, 0, 1, 1.0, da.specialn_ns_freefall_toggle);
            }
        }
    }
}

pub fn ftKb_NsSpecialNStart_IASA(_gobj: G) { melee_core::ct!("ftKb_NsSpecialNStart_IASA");}
pub fn ftKb_NsSpecialNEnd_IASA(_gobj: G) { melee_core::ct!("ftKb_NsSpecialNEnd_IASA");}
pub fn ftKb_NsSpecialAirNStart_IASA(_gobj: G) { melee_core::ct!("ftKb_NsSpecialAirNStart_IASA");}
pub fn ftKb_NsSpecialAirNEnd_IASA(_gobj: G) { melee_core::ct!("ftKb_NsSpecialAirNEnd_IASA");}

/// Shared Hold IASA: count the release delay down; once it hit 0 and B is let go, drop the flash.
unsafe fn hold_iasa(gobj: G) {
    {
        let fp = fpp(gobj);
        (*fp).mv.ns.specialn.charge_release_delay -= 1;
        let result;
        if (*fp).mv.ns.specialn.charge_release_delay <= 0 {
            (*fp).mv.ns.specialn.charge_release_delay = 0;
            result = true;
        } else {
            result = false;
        }
        if !result {
            return;
        }
        if ((*fp).input.held_buttons[0] & HSD_PAD_B_U) != 0 {
            return;
        }
    }
    if gobj.is_null() {
        return;
    }
    let fp = fpp(gobj);
    if fp.is_null() {
        return;
    }
    if !(*fp).u.kb.ns_flash_gobj.is_null() {
        (*fp).u.kb.ns_flash_gobj = core::ptr::null_mut();
    }
}

pub fn ftKb_NsSpecialNHold_IASA(gobj: G) {
    melee_core::ct!("ftKb_NsSpecialNHold_IASA");
    unsafe { hold_iasa(gobj) }
}

pub fn ftKb_NsSpecialAirNHold_IASA(gobj: G) {
    melee_core::ct!("ftKb_NsSpecialAirNHold_IASA");
    unsafe { hold_iasa(gobj) }
}

/// The root module's Fighter-only half, as the C-signature entry.
pub fn ftKb_NsSpecialAirNStart_Phys(gobj: G) {
    melee_core::ct!("ftKb_NsSpecialAirNStart_Phys");
    unsafe { crate::chars::ftKirby::ftkirbyspecialness::ftKb_NsSpecialAirNStart_Phys(&mut *fpp(gobj)) }
}

pub fn ftKb_NsSpecialAirNHold_Phys(gobj: G) {
    melee_core::ct!("ftKb_NsSpecialAirNHold_Phys");
    ftKb_NsSpecialAirNStart_Phys(gobj);
}

pub fn ftKb_NsSpecialAirNEnd_Phys(gobj: G) {
    melee_core::ct!("ftKb_NsSpecialAirNEnd_Phys");
    ftKb_NsSpecialAirNStart_Phys(gobj);
}
