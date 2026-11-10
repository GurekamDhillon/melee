//! src/melee/ft/kinds/ftKirby/ftkirbyspecialluigi.c: Kirby's Luigi copy ability (fireball).
//! No fused sites (tools/rust-port/fma_sites.py: 0 fmuladd in every function). Smoke-tested by
//! tests/tu_ftkirbyspecialluigi.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::stubs as st;
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use melee_types::*;

/// Already ported (rounds 2/3/5/7): re-exported under this TU.





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
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")),
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
                    None => { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, off as _, out as _); } return; },
                }
            }
    pub static mut IT_802C01AC: Option<fn(*mut HSD_GObj, *mut Vec3, i32, f32) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C01AC(gobj: *mut HSD_GObj, pos: *mut Vec3, kind: i32, facing_dir: f32)  {
                match unsafe { IT_802C01AC } {
                    Some(f) => f(gobj, pos, kind, facing_dir),
                    None => { unsafe { melee_items::tu::itluigifireball::it_802C01AC(gobj as _, pos as _, kind as _, facing_dir as _); } return; },
                }
            }
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

/// Accessory callback entry for `fn_800F98F4`.
pub unsafe extern "C" fn cb_fn_800F98F4(g: G) {
    melee_core::ct!("cb_fn_800F98F4");
    fn_800F98F4(g);
}

pub fn ftKb_LgSpecialAirN_IASA(gobj: G) {
    melee_core::ct!("ftKb_LgSpecialAirN_IASA");
    unsafe {
        let ft = fpp(gobj);
        if (*ft).cmd_vars[0] != 0 {
            ext::ftCo_Fall_IASA_Inner(gobj);
        }
    }
}

pub fn fn_800F98F4(gobj: G) {
    melee_core::ct!("fn_800F98F4");
    unsafe {
        let fp = fpp(gobj);
        // `ftCheckThrowB0`: test and clear.
        if (*fp).x2210.x0.throw_flags_b0() != 0 {
            (*fp).x2210.x0.set_throw_flags_b0(0);
            let mut pos: Vec3 = core::mem::zeroed();
            let bone = melee_fighter::hooks::ftparts_get_bone_index(&*fp, FtPart_LHandN as i32);
            ext::lb_8000B1CC((*(*fp).parts.add(bone as usize)).joint, core::ptr::null(), &mut pos);
            ext::it_802C01AC(gobj, &mut pos, It_Kind_Kirby_LuigiFire as i32, (*fp).facing_dir);
            let bone2 = melee_fighter::hooks::ftparts_get_bone_index(&*fp, FtPart_LHandN as i32);
            let joint = (*(*fp).parts.add(bone2 as usize)).joint;
            ext::efSync_Spawn(0x4B1, gobj, &[joint as usize, &raw mut (*fp).facing_dir as usize]);
        }
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftKb_LgSpecialN_Anim` (src/melee/ft/kinds/ftKirby/ftkirbyspecialluigi.c).
pub fn ftKb_LgSpecialN_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_LgSpecialN_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ft_8008A2BC(gobj);
    }
}

/// `ftKb_LgSpecialAirN_Anim` (src/melee/ft/kinds/ftKirby/ftkirbyspecialluigi.c).
pub fn ftKb_LgSpecialAirN_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_LgSpecialAirN_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ftCo_Fall_Enter(gobj);
    }
}

/// `ftKb_LgSpecialN_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialluigi.c).
pub fn ftKb_LgSpecialN_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_LgSpecialN_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftKb_LgSpecialAirN_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialluigi.c).
pub fn ftKb_LgSpecialAirN_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_LgSpecialAirN_Phys");
    st::ft_80084DB0(gobj);
}

/// `ftKb_LgSpecialN_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialluigi.c).
pub fn ftKb_LgSpecialN_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_LgSpecialN_Coll");
    unsafe {
        if st::ft_80082708(gobj) == 0 /* GA_Ground */ {
            let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
            st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 432 /* ftKb_MS_LgSpecialAirN */, 0x5000u32 /* ftLg_MF_SpecialN_Coll */);
            (*fp).accessory4_cb = Some(crate::chars::cb::ftkirbyspecialluigi__fn_800F98F4);
        }
    }
}

/// `ftKb_LgSpecialAirN_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialluigi.c).
pub fn ftKb_LgSpecialAirN_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_LgSpecialAirN_Coll");
    unsafe {
        if st::ft_80081D0C(gobj) != 0 /* GA_Ground */ {
            let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
            st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 431 /* ftKb_MS_LgSpecialN */, 0x5000u32 /* ftLg_MF_SpecialN_Coll */);
            (*fp).accessory4_cb = Some(crate::chars::cb::ftkirbyspecialluigi__fn_800F98F4);
        }
    }
}

/// `ftKb_LgSpecialN_IASA` (src/melee/ft/kinds/ftKirby/ftkirbyspecialluigi.c).
pub fn ftKb_LgSpecialN_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_LgSpecialN_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (((*fp).cmd_vars[0] as u32) != 0u32 /* 0 */) {
            st::ftCo_Wait_IASA(gobj);
        }
    }
}

/// `ftKb_SpecialNLg_800F9614` (src/melee/ft/kinds/ftKirby/ftkirbyspecialluigi.c).
pub fn ftKb_SpecialNLg_800F9614(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNLg_800F9614");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).cmd_vars[0] = 0_u32;
        (*fp).x2210.throw_flags = 0_u32;
        st::Fighter_ChangeMotionState(gobj, 431 /* ftKb_MS_LgSpecialN */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).accessory4_cb = Some(crate::chars::cb::ftkirbyspecialluigi__fn_800F98F4);
    }
}

/// `ftKb_SpecialNLg_800F9684` (src/melee/ft/kinds/ftKirby/ftkirbyspecialluigi.c).
pub fn ftKb_SpecialNLg_800F9684(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNLg_800F9684");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).cmd_vars[0] = 0_u32;
        (*fp).x2210.throw_flags = 0_u32;
        st::Fighter_ChangeMotionState(gobj, 432 /* ftKb_MS_LgSpecialAirN */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).accessory4_cb = Some(crate::chars::cb::ftkirbyspecialluigi__fn_800F98F4);
    }
}
