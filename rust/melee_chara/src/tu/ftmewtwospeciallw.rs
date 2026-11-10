//! src/melee/ft/kinds/ftMewtwo/ftmewtwospeciallw.c: Disable (down B). One fused site
//! (tools/rust-port/fma_sites.py): `ftMt_SpecialLw_CreateDisable`'s `sp18.x += offset_x *
//! facing_dir` = fmadds(offset_x, facing_dir, sp18.x). Smoke-tested through recording hooks by
//! tests/tu_ftmewtwospeciallw.rs, which checks the position bit for bit against an independent
//! double-precision fused multiply-add (not against compiled C).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_types::*;

/// Already ported (rounds 2/4/5/6/7/12): re-exported under this TU.







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
    pub static mut ITMEWTWODISABLE_LOGIC67_DESTROY: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn itMewtwoDisable_Logic67_Destroy(item: *mut HSD_GObj)  {
                match unsafe { ITMEWTWODISABLE_LOGIC67_DESTROY } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::itmewtwodisable::itMewtwoDisable_Logic67_Destroy(item as _); } return; },
                }
            }
    pub static mut ITMEWTWODISABLE_LOGIC67_SPAWNMEWTWODISABLE: Option<fn(*mut HSD_GObj, *const Vec3, f32) -> *mut HSD_GObj> = None;
            #[allow(non_snake_case)]
            pub fn itMewtwoDisable_Logic67_SpawnMewtwoDisable(owner: *mut HSD_GObj, pos: *const Vec3, facing_dir: f32) -> *mut HSD_GObj {
                match unsafe { ITMEWTWODISABLE_LOGIC67_SPAWNMEWTWODISABLE } {
                    Some(f) => f(owner, pos, facing_dir),
                    None => { return unsafe { melee_items::tu::itmewtwodisable::itMewtwoDisable_Logic67_SpawnMewtwoDisable(owner as _, pos as _, facing_dir as _) as _ }; },
                }
            }
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(joint: *mut HSD_JObj, off: *const Vec3, out: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(joint, off, out),
                    None => { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(joint as _, off as _, out as _); } return; },
                }
            }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

pub fn ftMt_SpecialLw_ClearDisableGObj(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMt_SpecialLw_ClearDisableGObj");
    unsafe { (*fpp(gobj)).u.mt.x222C_disableGObj = core::ptr::null_mut() }
}

pub fn ftMt_SpecialLw_RemoveDisable(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMt_SpecialLw_RemoveDisable");
    unsafe {
        if !(*fpp(gobj)).u.mt.x222C_disableGObj.is_null() {
            ext::itMewtwoDisable_Logic67_Destroy((*fpp(gobj)).u.mt.x222C_disableGObj);
            (*fpp(gobj)).u.mt.x222C_disableGObj = core::ptr::null_mut();
        }
    }
}

pub fn ftMt_SpecialLw_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMt_SpecialLw_Anim");
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            if !(*fpp(gobj)).u.mt.x222C_disableGObj.is_null() {
                ext::itMewtwoDisable_Logic67_Destroy((*fpp(gobj)).u.mt.x222C_disableGObj);
                (*fpp(gobj)).u.mt.x222C_disableGObj = core::ptr::null_mut();
            }
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftMt_SpecialAirLw_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMt_SpecialAirLw_Anim");
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            if !(*fpp(gobj)).u.mt.x222C_disableGObj.is_null() {
                ext::itMewtwoDisable_Logic67_Destroy((*fpp(gobj)).u.mt.x222C_disableGObj);
                (*fpp(gobj)).u.mt.x222C_disableGObj = core::ptr::null_mut();
            }
            st::ftCo_Fall_Enter(gobj);
        }
    }
}

pub fn ftMt_SpecialAirLw_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMt_SpecialAirLw_Phys");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftMewtwoAttributes;
        st::ftCommon_Fall(
            &mut *fp,
            (*da).x78_MEWTWO_DISABLE_GRAVITY,
            (*da).x7C_MEWTWO_DISABLE_TERMINAL_VELOCITY,
        );
        melee_fighter::ftcommon::calc_self_accel_deaccel(&mut *fp, (*fp).co_attrs.aerial_friction);
    }
}

/// `ftMt_SpecialLw_CreateDisable` (the `accessory4_cb`): spawn the Disable projectile on the
/// frame the script sets cmd_vars[0].
pub fn ftMt_SpecialLw_CreateDisable(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMt_SpecialLw_CreateDisable");
    unsafe {
        let fp = fpp(gobj);
        let mut sp18 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

        if (*fp).cmd_vars[0] != 0 {
            let da = (*fp).dat_attrs as *const ftMewtwoAttributes;

            ext::lb_8000B1CC((*(*fp).parts.add(FtPart_L3rdNb as usize)).joint, core::ptr::null(), &mut sp18);

            sp18.x = fmadds((*da).x80_MEWTWO_DISABLE_OFFSET_X, (*fp).facing_dir, sp18.x);
            sp18.y += (*da).x84_MEWTWO_DISABLE_OFFSET_Y;

            (*fp).u.mt.x222C_disableGObj =
                ext::itMewtwoDisable_Logic67_SpawnMewtwoDisable(gobj, &sp18, (*fp).facing_dir);

            ftMewtwo_SpecialLw_SetCall(gobj);

            (*fp).cmd_vars[0] = 0;
        }
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftMt_SpecialLw_IASA` (src/melee/ft/kinds/ftMewtwo/ftmewtwospeciallw.c).
pub fn ftMt_SpecialLw_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMt_SpecialLw_IASA");
    unsafe {
        return;
    }
}

/// `ftMt_SpecialAirLw_IASA` (src/melee/ft/kinds/ftMewtwo/ftmewtwospeciallw.c).
pub fn ftMt_SpecialAirLw_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMt_SpecialAirLw_IASA");
    unsafe {
        return;
    }
}

/// `ftMt_SpecialLw_Phys` (src/melee/ft/kinds/ftMewtwo/ftmewtwospeciallw.c).
pub fn ftMt_SpecialLw_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMt_SpecialLw_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftMewtwo_SpecialLw_SetCall` (static) (src/melee/ft/kinds/ftMewtwo/ftmewtwospeciallw.c).
pub fn ftMewtwo_SpecialLw_SetCall(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMewtwo_SpecialLw_SetCall");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (!(*fp).u.mt.x222C_disableGObj.is_null()) {
            (*fp).death2_cb = Some(crate::chars::cb::ftmewtwo__ftMt_Init_OnDeath2);
            (*fp).take_dmg_cb = Some(crate::chars::cb::ftmewtwo__ftMt_Init_OnTakeDamage);
        }
    }
}

/// `ftMt_SpecialAirLw_AirToGround` (src/melee/ft/kinds/ftMewtwo/ftmewtwospeciallw.c).
pub fn ftMt_SpecialAirLw_AirToGround(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMt_SpecialAirLw_AirToGround");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 359 /* ftMt_MS_SpecialLw */, 0xC4C5082u32 /* FTMEWTWO_SPECIALLW_COLL_FLAG */);
        (*fp).accessory4_cb = Some(crate::chars::cb::ftmewtwospeciallw__ftMt_SpecialLw_CreateDisable);
        crate::chars::r4::ftMewtwo::ftmewtwospeciallw::ftMewtwo_SpecialLw_SetCall(gobj);
    }
}

/// `ftMt_SpecialLw_Coll` (src/melee/ft/kinds/ftMewtwo/ftmewtwospeciallw.c).
pub fn ftMt_SpecialLw_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMt_SpecialLw_Coll");
    unsafe {
        st::ft_8008403C(gobj, Some(crate::chars::cb::ftmewtwospeciallw__ftMt_SpecialLw_GroundToAir));
    }
}

/// `ftMt_SpecialAirLw_Coll` (src/melee/ft/kinds/ftMewtwo/ftmewtwospeciallw.c).
pub fn ftMt_SpecialAirLw_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMt_SpecialAirLw_Coll");
    unsafe {
        st::ft_80082C74(gobj, Some(crate::chars::cb::ftmewtwospeciallw__ftMt_SpecialAirLw_AirToGround));
    }
}

/// `ftMt_SpecialLw_GroundToAir` (src/melee/ft/kinds/ftMewtwo/ftmewtwospeciallw.c).
pub fn ftMt_SpecialLw_GroundToAir(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMt_SpecialLw_GroundToAir");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_8007D5D4(&mut *fp);
        (*fp).self_vel.y = f32::from_bits(0x00000000) /* 0.0f */;
        st::Fighter_ChangeMotionState(gobj, 360 /* ftMt_MS_SpecialAirLw */, 0xC4C5082u32 /* FTMEWTWO_SPECIALLW_COLL_FLAG */, (*fp).cur_anim_frame, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        (*fp).accessory4_cb = Some(crate::chars::cb::ftmewtwospeciallw__ftMt_SpecialLw_CreateDisable);
        crate::chars::r4::ftMewtwo::ftmewtwospeciallw::ftMewtwo_SpecialLw_SetCall(gobj);
        st::ftCommon_ClampAirDrift(&mut *fp);
    }
}

/// `ftMt_SpecialLw_Enter` (src/melee/ft/kinds/ftMewtwo/ftmewtwospeciallw.c).
pub fn ftMt_SpecialLw_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMt_SpecialLw_Enter");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).x2210.throw_flags = 0_u32;
        (*fp).cmd_vars[0] = 0_u32;
        (*fp).u.mt.x222C_disableGObj = core::mem::zeroed(); /* NULL */
        st::Fighter_ChangeMotionState(gobj, 359 /* ftMt_MS_SpecialLw */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).accessory4_cb = Some(crate::chars::cb::ftmewtwospeciallw__ftMt_SpecialLw_CreateDisable);
    }
}

/// `ftMt_SpecialAirLw_Enter` (src/melee/ft/kinds/ftMewtwo/ftmewtwospeciallw.c).
pub fn ftMt_SpecialAirLw_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMt_SpecialAirLw_Enter");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).x2210.throw_flags = 0_u32;
        (*fp).cmd_vars[0] = 0_u32;
        (*fp).u.mt.x222C_disableGObj = core::mem::zeroed(); /* NULL */
        (*fp).self_vel.y = f32::from_bits(0x00000000) /* 0.0f */;
        st::Fighter_ChangeMotionState(gobj, 360 /* ftMt_MS_SpecialAirLw */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).accessory4_cb = Some(crate::chars::cb::ftmewtwospeciallw__ftMt_SpecialLw_CreateDisable);
    }
}
