//! src/melee/ft/kinds/ftKirby/ftkirbyspecialmario.c: Kirby's Mario / Dr. Mario copy ability
//! (fireball, vitamin). No fused sites (tools/rust-port/fma_sites.py: 0 fmuladd in every
//! function). Smoke-tested by tests/tu_ftkirbyspecialmario.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2/5): re-exported under this TU.
pub use crate::chars::r2::ftKirby::ftkirbyspecialmario::{
    ftKb_MrSpecialAirN_Anim, ftKb_MrSpecialAirN_Phys, ftKb_MrSpecialN_Anim, ftKb_MrSpecialN_Phys,
};
pub use crate::chars::r5::ftKirby::ftkirbyspecialmario::ftKb_MrSpecialN_IASA;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
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
    /// Like `hook!`, but an unset hook falls back to an existing native port.
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
    hook_or!(FTCO_FALL_IASA_INNER, ftCo_Fall_IASA_Inner, melee_fighter::tu::ftCo_Fall::ftCo_Fall_IASA_Inner,
        (gobj: *mut HSD_GObj) -> bool);
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(jobj: *mut HSD_JObj, off: *const Vec3, out: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(jobj, off, out),
                    None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, off as _, out as _); } return; },
                }
            }
    crate::port_hook!(HSD_RANDI, HSD_Randi, crate::adapt::HSD_Randi, (max: i32) -> i32);
    pub static mut IT_8029B6F8: Option<fn(*mut HSD_GObj, *mut Vec3, i32, f32) > = None;
            #[allow(non_snake_case)]
            pub fn it_8029B6F8(gobj: *mut HSD_GObj, pos: *mut Vec3, kind: i32, facing_dir: f32)  {
                match unsafe { IT_8029B6F8 } {
                    Some(f) => f(gobj, pos, kind, facing_dir),
                    None => { unsafe { let _ = melee_items::tu::itmariofireball::it_8029B6F8(gobj as _, pos as _, kind as _, facing_dir as _); } return; },
                }
            }
    crate::port_hook!(ITDRMARIOPILL_SPAWN, itDrMarioPill_Spawn, crate::adapt::itDrMarioPill_Spawn,
        (gobj: *mut HSD_GObj, pos: *mut Vec3, pick: i32, kind: i32, facing_dir: f32));
    /// `efSync_Spawn(gfx_id, gobj, ...)` (variadic in C; `va` are the extra word arguments).
    pub static mut EFSYNC_SPAWN: Option<fn(i32, *mut HSD_GObj, &[usize]) -> *mut core::ffi::c_void> = None;
    pub fn efSync_Spawn(id: i32, gobj: *mut HSD_GObj, va: &[usize]) -> *mut core::ffi::c_void {
        match unsafe { EFSYNC_SPAWN } {
            Some(f) => f(id, gobj, va),
            None => unsafe { crate::adapt::efSync_Spawn_usize(id, gobj, va) },
        }
    }
}

type G = *mut HSD_GObj;

#[inline(always)]
unsafe fn fpp(gobj: G) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// Accessory callback entry for `fn_800F9260`.
pub unsafe extern "C" fn cb_fn_800F9260(g: G) {
    fn_800F9260(g);
}

pub fn ftKb_SpecialN_800F9110(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let mut msid = ftKb_MS_MrSpecialN as i32;
        (*fp).cmd_vars[0] = 0;
        (*fp).anon_2210.throw_flags = 0;
        if (*fpp(gobj)).u.kb.hat.kind as u32 != Ft_Kind_Mario as u32 {
            msid = 0x200;
        }
        st::Fighter_ChangeMotionState(gobj, msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).accessory4_cb = Some(cb_fn_800F9260);
    }
}

pub fn ftKb_MrSpecialN_Coll(gobj: G) {
    unsafe {
        if st::ft_80082708(gobj) == GA_Ground as i32 {
            ftKb_SpecialNLg_800F951C(gobj);
        }
    }
}

/// `fn_800F9260_inline`: pick a pill colour that is neither of the last two.
unsafe fn pick_pill(gobj: G) -> i32 {
    let fp2 = fpp(gobj);
    let mut candidates = [0i32; 9];
    let mut count = 0i32;
    let mut i = 0i32;
    while i < 9 {
        if i != (*fp2).u.kb.x68 && i != (*fp2).u.kb.x6C {
            candidates[count as usize] = i;
            count += 1;
        }
        i += 1;
    }
    let pick = candidates[ext::HSD_Randi(count) as usize];
    (*fp2).u.kb.x6C = (*fp2).u.kb.x68;
    (*fp2).u.kb.x68 = pick;
    pick
}

pub fn fn_800F9260(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).u.kb.hat.kind as u32 != Ft_Kind_Kirby as u32 {
            // `ftCheckThrowB0`: test and clear.
            if (*fp).anon_2210.anon_0.throw_flags_b0() != 0 {
                (*fp).anon_2210.anon_0.set_throw_flags_b0(0);
                let mut sp44: Vec3 = core::mem::zeroed();
                let bone0 = melee_fighter::hooks::ftparts_get_bone_index(&*fp, FtPart_LHandN as i32);
                ext::lb_8000B1CC((*(*fp).parts.add(bone0 as usize)).joint, core::ptr::null(), &mut sp44);
                if (*fp).u.kb.hat.kind as u32 == Ft_Kind_Mario as u32 {
                    ext::it_8029B6F8(gobj, &mut sp44, It_Kind_Kirby_MarioFire as i32, (*fp).facing_dir);
                    let bone = melee_fighter::hooks::ftparts_get_bone_index(&*fp, FtPart_LHandN as i32);
                    let joint = (*(*fp).parts.add(bone as usize)).joint;
                    ext::efSync_Spawn(0x49F, gobj, &[joint as usize, &raw mut (*fp).facing_dir as usize]);
                    return;
                }
                let pick = pick_pill(gobj);
                ext::itDrMarioPill_Spawn(gobj, &mut sp44, pick, It_Kind_Kirby_DrMarioVitamin as i32, (*fp).facing_dir);
            }
        }
    }
}

pub fn ftKb_SpecialNMr_800F93CC(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let mut msid = ftKb_MS_MrSpecialAirN as i32;
        (*fp).cmd_vars[0] = 0;
        (*fp).anon_2210.throw_flags = 0;
        if (*fpp(gobj)).u.kb.hat.kind as u32 != Ft_Kind_Mario as u32 {
            msid = ftKb_MS_DrSpecialAirN as i32;
        }
        st::Fighter_ChangeMotionState(gobj, msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).accessory4_cb = Some(cb_fn_800F9260);
    }
}

pub fn ftKb_MrSpecialAirN_IASA(gobj: G) {
    unsafe {
        let ft = fpp(gobj);
        if (*ft).cmd_vars[0] != 0 {
            ext::ftCo_Fall_IASA_Inner(gobj);
        }
    }
}

pub fn ftKb_MrSpecialAirN_Coll(gobj: G) {
    unsafe {
        if st::ft_80081D0C(gobj) != GA_Ground as i32 {
            ftKb_SpecialNLg_800F9598(gobj);
        }
    }
}

pub fn ftKb_SpecialNLg_800F951C(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_8007D5D4(&mut *fp);
        let fp2 = fpp(gobj);
        let mut motion_id = 0x190;
        let anim_frame = (*fp).cur_anim_frame;
        if (*fp2).u.kb.hat.kind != 0 {
            motion_id = 0x201;
        }
        st::Fighter_ChangeMotionState(gobj, motion_id, 0x5000, anim_frame, 1.0, 0.0, core::ptr::null_mut());
        (*fp).accessory4_cb = Some(cb_fn_800F9260);
    }
}

pub fn ftKb_SpecialNLg_800F9598(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_8007D7FC(&mut *fp);
        let fp2 = fpp(gobj);
        let mut motion_id = 0x18F;
        let anim_frame = (*fp).cur_anim_frame;
        if (*fp2).u.kb.hat.kind != 0 {
            motion_id = 0x200;
        }
        st::Fighter_ChangeMotionState(gobj, motion_id, 0x5000, anim_frame, 1.0, 0.0, core::ptr::null_mut());
        (*fp).accessory4_cb = Some(cb_fn_800F9260);
    }
}
