//! src/melee/ft/kinds/ftMewtwo/ftmewtwospecials.c: Confusion (side B). No fused sites
//! (tools/rust-port/fma_sites.py) and no float math. Smoke-tested through recording hooks by
//! tests/tu_ftmewtwospecials.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::chars::cb;
use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 5/6/7/12): re-exported under this TU.
pub use crate::chars::r12::ftMewtwo::ftmewtwospecials::{
    ftMt_SpecialAirS_IASA, ftMt_SpecialS_IASA, ftMt_SpecialS_OnReflect,
};
pub use crate::chars::r5::ftMewtwo::ftmewtwospecials::{ftMt_SpecialAirS_Coll, ftMt_SpecialS_Coll};
pub use crate::chars::r6::ftMewtwo::ftmewtwospecials::{ftMt_SpecialAirS_Phys, ftMt_SpecialS_Phys};
pub use crate::chars::r7::ftMewtwo::ftmewtwospecials::ftMt_SpecialS_SetFlags;

/// `CONFUSION_REFLECT_NONE` / `_ON` / `_OFF`: the values the script writes to cmd_vars[1].
pub const CONFUSION_REFLECT_NONE: u32 = 0;
pub const CONFUSION_REFLECT_ON: u32 = 1;
pub const CONFUSION_REFLECT_OFF: u32 = 2;

/// `FTMEWTWO_SPECIALS_COLL_FLAG`: KeepGfx | SkipMatAnim | SkipColAnim | UpdateCmd | SkipItemVis |
/// Unk19 | SkipModelPartVis | SkipModelFlags | Unk27.
pub const FTMEWTWO_SPECIALS_COLL_FLAG: u32 = (Ft_MF_KeepGfx
    | Ft_MF_SkipMatAnim
    | Ft_MF_SkipColAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27) as u32;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics, except the
/// ones with a verified port in melee_fighter, which the unset hook calls.
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
    macro_rules! hook_or {
        ($hook:ident, $name:ident, $port:path, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => $port($($a),*),
                }
            }
        };
    }
    crate::port_hook!(FTCOLL_CREATEREFLECTHIT, ftColl_CreateReflectHit, crate::adapt::ftColl_CreateReflectHit,
          (gobj: *mut HSD_GObj, desc: *const ReflectDesc, cb: HSD_GObjEvent));
    hook_or!(FTCO_800DE2A8, ftCo_800DE2A8, melee_fighter::tu::ftCo_Throw::ftCo_800DE2A8,
             (gobj: *mut HSD_GObj, victim: *mut HSD_GObj));
    hook_or!(FTCO_80090780, ftCo_80090780, melee_fighter::tu::ftCo_DamageFall::ftCo_80090780,
             (gobj: *mut HSD_GObj));
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ftMt_SpecialS_SetFlags` as an `HSD_GObjEvent` entry (the `ftCommon_8007E2D0` grab callback).
pub unsafe extern "C" fn cb_ftMt_SpecialS_SetFlags(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftMt_SpecialS_SetFlags");
    ftMt_SpecialS_SetFlags(g)
}

/// `ftMt_SpecialS_ReflectThink` as an `HSD_GObjEvent` entry (`accessory4_cb`).
pub unsafe extern "C" fn cb_ftMt_SpecialS_ReflectThink(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftMt_SpecialS_ReflectThink");
    ftMt_SpecialS_ReflectThink(g)
}

/// `ftMt_SpecialS_OnReflect` as an `HSD_GObjEvent` entry (`reflect_hit_cb`).
pub unsafe extern "C" fn cb_ftMt_SpecialS_OnReflect(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftMt_SpecialS_OnReflect");
    ftMt_SpecialS_OnReflect(g)
}

/// `ftCo_800BCF18` as an `HSD_GObjInteraction` entry (`grabbed_cb`, ground).
pub unsafe extern "C" fn cb_ftCo_800BCF18(g: *mut HSD_GObj, victim: *mut HSD_GObj) {
    melee_core::ct!("cb_ftCo_800BCF18");
    melee_fighter::tu::ftCo_CaptureMewtwo::ftCo_800BCF18(g, victim)
}

/// `ftCo_800BD000` as an `HSD_GObjInteraction` entry (`grabbed_cb`, air).
pub unsafe extern "C" fn cb_ftCo_800BD000(g: *mut HSD_GObj, victim: *mut HSD_GObj) {
    melee_core::ct!("cb_ftCo_800BD000");
    melee_fighter::tu::ftCo_CaptureMewtwo::ftCo_800BD000(g, victim)
}

/// `ftMewtwo_SpecialS_SetGrab` (static inline): arm the grab, or keep the held victim.
pub fn ftMewtwo_SpecialS_SetGrab(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMewtwo_SpecialS_SetGrab");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).victim_gobj.is_null() {
            melee_fighter::ftcommon2::ftcommon_8007e2d0(
                &mut *fp,
                0x40,
                Some(cb_ftMt_SpecialS_SetFlags),
                None,
                Some(cb_ftCo_800BCF18),
            );
        } else {
            st::ftCommon_8007E2F4(&mut *fp, 0x1FF);
        }
    }
}

/// `ftMewtwo_SpecialAirS_SetGrab` (static inline): the aerial version of the above.
pub fn ftMewtwo_SpecialAirS_SetGrab(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMewtwo_SpecialAirS_SetGrab");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).victim_gobj.is_null() {
            melee_fighter::ftcommon2::ftcommon_8007e2d0(
                &mut *fp,
                0x40,
                Some(cb_ftMt_SpecialS_SetFlags),
                None,
                Some(cb_ftCo_800BD000),
            );
        } else {
            st::ftCommon_8007E2F4(&mut *fp, 0x1FF);
        }
    }
}

/// Grounded Confusion Motion State handler.
pub fn ftMt_SpecialS_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMt_SpecialS_Enter");
    unsafe {
        let fp = fpp(gobj);
        (*fp).anon_2210.throw_flags = 0;
        (*fp).cmd_vars[0] = 0;
        (*fp).cmd_vars[1] = 0;
        (*fp).mv.mt.SpecialS.set_isConfusionReflect(0);
        st::Fighter_ChangeMotionState(gobj, ftMt_MS_SpecialS as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        ftMewtwo_SpecialS_SetGrab(gobj);
        (*fp).accessory4_cb = Some(cb_ftMt_SpecialS_ReflectThink);
    }
}

/// Aerial Confusion Motion State handler.
pub fn ftMt_SpecialAirS_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMt_SpecialAirS_Enter");
    unsafe {
        let fp = fpp(gobj);
        let a = (*fp).dat_attrs as *const ftMewtwoAttributes;
        (*fp).anon_2210.throw_flags = 0;
        (*fp).cmd_vars[0] = 0;
        (*fp).cmd_vars[1] = 0;
        (*fp).mv.mt.SpecialS.set_isConfusionReflect(0);
        if (*fp).u.mt.x223C_isConfusionBoost == 0 {
            (*fp).self_vel.y = (*a).x18_MEWTWO_CONFUSION_AIR_BOOST;
            (*fp).u.mt.x223C_isConfusionBoost = 1;
        }
        st::Fighter_ChangeMotionState(gobj, ftMt_MS_SpecialAirS as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        ftMewtwo_SpecialAirS_SetGrab(gobj);
        (*fp).accessory4_cb = Some(cb_ftMt_SpecialS_ReflectThink);
    }
}

/// `ftMewtwo_SetGrabVictim` (static inline): on the script's cmd_vars[0] pulse, hand the grabbed
/// victim to the throw.
pub fn ftMewtwo_SetGrabVictim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMewtwo_SetGrabVictim");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).cmd_vars[0] != 0 {
            let victim = (*fp).victim_gobj;
            if !victim.is_null() {
                st::ftCommon_8007E2F4(&mut *fp, 0);
                ext::ftCo_800DE2A8(gobj, victim);
                ext::ftCo_80090780(victim);
                (*fp).cmd_vars[0] = 0;
            }
        }
    }
}

pub fn ftMt_SpecialS_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMt_SpecialS_Anim");
    unsafe {
        ftMewtwo_SetGrabVictim(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftMt_SpecialAirS_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMt_SpecialAirS_Anim");
    unsafe {
        ftMewtwo_SetGrabVictim(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCo_Fall_Enter(gobj);
        }
    }
}

/// `ftMewtwo_SpecialS_SetReflect` (static inline): re-arm the bubble across a ground/air switch.
pub fn ftMewtwo_SpecialS_SetReflect(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMewtwo_SpecialS_SetReflect");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).mv.mt.SpecialS.isConfusionReflect() != 0 {
            (*fp).set_reflecting(1);
            (*fp).set_x2218_b4(1);
            (*fp).reflect_hit_cb = Some(cb_ftMt_SpecialS_OnReflect);
        }
    }
}

pub fn ftMt_SpecialS_GroundToAir(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMt_SpecialS_GroundToAir");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, ftMt_MS_SpecialAirS as i32, FTMEWTWO_SPECIALS_COLL_FLAG);
        st::ftCommon_ClampAirDrift(&mut *fp);
        ftMewtwo_SpecialAirS_SetGrab(gobj);
        (*fp).accessory4_cb = Some(cb_ftMt_SpecialS_ReflectThink);
        ftMewtwo_SpecialS_SetReflect(gobj);
    }
}

pub fn ftMt_SpecialAirS_AirToGround(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMt_SpecialAirS_AirToGround");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, ftMt_MS_SpecialS as i32, FTMEWTWO_SPECIALS_COLL_FLAG);
        ftMewtwo_SpecialS_SetGrab(gobj);
        (*fp).accessory4_cb = Some(cb_ftMt_SpecialS_ReflectThink);
        (*fp).u.mt.x223C_isConfusionBoost = 0;
        ftMewtwo_SpecialS_SetReflect(gobj);
    }
}

/// Confusion reflect think function (`accessory4_cb`).
pub fn ftMt_SpecialS_ReflectThink(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMt_SpecialS_ReflectThink");
    unsafe {
        let fp = fpp(gobj);
        let a = (*fp).dat_attrs as *const ftMewtwoAttributes;
        let reflect_flag = (*fp).cmd_vars[1];
        match reflect_flag {
            CONFUSION_REFLECT_ON => {
                ext::ftColl_CreateReflectHit(gobj, &(*a).x1C_MEWTWO_CONFUSION_REFLECTION, Some(cb_ftMt_SpecialS_OnReflect));
                (*fp).set_x2218_b4(1);
                (*fp).mv.mt.SpecialS.set_isConfusionReflect(1);
                (*fp).cmd_vars[1] = CONFUSION_REFLECT_NONE;
            }
            CONFUSION_REFLECT_OFF => {
                if (*fp).mv.mt.SpecialS.isConfusionReflect() != 0 {
                    (*fp).set_reflecting(0);
                    (*fp).set_x2218_b4(0);
                    (*fp).reflect_hit_cb = None;
                    (*fp).mv.mt.SpecialS.set_isConfusionReflect(0);
                }
                (*fp).cmd_vars[1] = CONFUSION_REFLECT_NONE;
            }
            _ => {}
        }
    }
}
