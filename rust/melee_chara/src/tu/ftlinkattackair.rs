//! src/melee/ft/kinds/ftLink/ftlinkattackair.c: Link's down-air (the multi-hit drill) and the
//! Hylian-shield setup. Fused sites (tools/rust-port/fma_sites.py): none (one subtraction each in
//! `lwOnHit` / `lwOnAnim`). Smoke-tested by tests/tu_ftlinkattackair.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use crate::tu::ftlink::cb_ftLk_800EB334;
use melee_types::*;

/// Already ported (round 5): re-exported under this TU.
pub use crate::chars::r5::ftLink::ftlinkattackair::ftLk_AttackAir_SetupParts;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    /// Shared with ftclink (same C function).
    pub use crate::tu::ftclink::ext::{ftColl_8007B1B8, FTCOLL_8007B1B8};
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
    // ft/ftcoll.c
    crate::port_hook!(FTCOLL_8007AFF8, ftColl_8007AFF8, crate::adapt::ftColl_8007AFF8, (gobj: *mut HSD_GObj));
    crate::port_hook!(FTCOLL_8007B064, ftColl_8007B064, crate::adapt::ftColl_8007B064, (gobj: *mut HSD_GObj, idx: i32));
    crate::port_hook!(FTCOLL_8007ABD0, ftColl_8007ABD0, crate::adapt::ftColl_8007ABD0, (hit: *mut HitCapsule, flags: u32, gobj: *mut HSD_GObj));
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

#[inline(always)]
unsafe fn da<'a>(fp: *mut Fighter) -> &'a ftLk_DatAttrs {
    &*((*fp).dat_attrs as *const ftLk_DatAttrs)
}

/// Absorb the Hylian-shield hit (pushes Link back through `ftLk_800EB334`).
pub fn ftLk_AttackAir_800EB3BC(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_AttackAir_800EB3BC");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).x5F4_arr[2].prev == 0 {
            let a = (*fp).dat_attrs as *mut ftLk_DatAttrs;
            ext::ftColl_8007B1B8(gobj, &mut (*a).xC4 as *mut AbsorbDesc as *mut _, Some(cb_ftLk_800EB334));
            (*fp).anon_221b.set_x221B_b3(1);
            (*fp).anon_221b.set_x221B_b4(1);
            (*fp).anon_221b.set_x221B_b2(1);
        }
    }
}

pub fn ftLk_AttackAir_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_AttackAir_Enter");
    unsafe {
        let fp = fpp(gobj);
        melee_fighter::tu::ftCo_AttackAir::ftCo_AttackAir_EnterFromCStick(gobj);
        if (*fp).motion_id as u32 == ftCo_MS_AttackAirLw as u32 {
            (*fp).deal_dmg_cb = Some(cb_lwOnHit);
            (*fp).anim_cb = Some(cb_lwOnAnim);
            (*fp).mv.lk.attackair.lw_frame_start = 0.0;
        }
    }
}

/// `lwOnHit` (static): the first hit bounces Link up and restarts the drill animation.
pub fn lwOnHit(gobj: *mut HSD_GObj) {
    melee_core::ct!("lwOnHit");
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        let frame_len = a.attackairlw_hit_anim_frame_end - a.attackairlw_hit_anim_frame_start;
        ext::ftColl_8007AFF8(gobj);
        (*fp).self_vel.y = a.attackairlw_hit_vel_y;
        (*fp).set_fall_fast(0);
        (*fp).mv.lk.attackair.lw_frame_start = a.attackairlw_hit_anim_frame_start;
        if (*fp).cur_anim_frame > frame_len {
            st::Fighter_ChangeMotionState(gobj, ftCo_MS_AttackAirLw as i32, Ft_MF_None as u32, frame_len, 1.0, 0.0, core::ptr::null_mut());
        }
    }
}

/// `lwOnHit` as a Fighter callback entry.
pub unsafe extern "C" fn cb_lwOnHit(g: *mut HSD_GObj) {
    melee_core::ct!("cb_lwOnHit");
    lwOnHit(g);
}

/// `lwDoAnimHitCapsule` (static).
fn lwDoAnimHitCapsule(gobj: *mut HSD_GObj, idx: usize) {
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        ext::ftColl_8007B064(gobj, idx as i32);
        ext::ftColl_8007ABD0(&mut (*fp).x914[idx], a.attackairlw_anim_flags[idx], gobj);
    }
}

/// `lwOnAnim` (static): re-enable the three hit capsules when the delay counter runs out.
pub fn lwOnAnim(gobj: *mut HSD_GObj) {
    melee_core::ct!("lwOnAnim");
    unsafe {
        let fp = fpp(gobj);
        let frame_start = (*fp).mv.lk.attackair.lw_frame_start;
        let a = da(fp);
        if frame_start > 0.0 {
            (*fp).mv.lk.attackair.lw_frame_start = frame_start - (*fp).frame_speed_mul;
            if (*fp).mv.lk.attackair.lw_frame_start <= 0.0 && (*fp).cur_anim_frame < a.attackairlw_hit_anim_frame_end {
                lwDoAnimHitCapsule(gobj, 0);
                lwDoAnimHitCapsule(gobj, 1);
                lwDoAnimHitCapsule(gobj, 2);
            }
        }
        melee_fighter::tu::ftCo_AttackAir::ftCo_AttackAir_Anim(gobj);
    }
}

/// `lwOnAnim` as a Fighter callback entry.
pub unsafe extern "C" fn cb_lwOnAnim(g: *mut HSD_GObj) {
    melee_core::ct!("cb_lwOnAnim");
    lwOnAnim(g);
}
