//! src/melee/ft/kinds/ftKirby/ftkirbyspecialmars.c: Kirby's Marth / Roy copy ability (Shield
//! Breaker charge). No fused sites (tools/rust-port/fma_sites.py: 0 fmuladd in every function);
//! the hit damage is integer math then (u32)(f32)(s32). Smoke-tested by
//! tests/tu_ftkirbyspecialmars.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_runtime::cabi::sdiv;
use melee_types::*;

/// Already ported (round 2): re-exported under this TU.


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
    /// `efSync_Spawn(gfx_id, gobj, ...)` (variadic in C; `va` are the extra word arguments).
    pub static mut EFSYNC_SPAWN: Option<fn(i32, *mut HSD_GObj, &[usize]) -> *mut core::ffi::c_void> = None;
    pub fn efSync_Spawn(id: i32, gobj: *mut HSD_GObj, va: &[usize]) -> *mut core::ffi::c_void {
        match unsafe { EFSYNC_SPAWN } {
            Some(f) => f(id, gobj, va),
            None => unsafe { crate::adapt::efSync_Spawn_usize(id, gobj, va) },
        }
    }
    // `ft_80459B88.hats[index]`, or NULL when `hats` is NULL.
    crate::port_hook!(FT_80459B88_HAT, ft_80459B88_hat, crate::adapt::ft_80459B88_hat, (index: i32) -> *mut KirbyHatStruct);
    crate::port_hook!(FTCOMMON_SETACCESSORY, ftCommon_SetAccessory, crate::adapt::ftCommon_SetAccessory, (fp: *mut Fighter, joint: *mut HSD_Joint));
    hook_or!(FTCOMMON_GETMODELSCALE, ftCommon_GetModelScale, ftcommon_get_model_scale_port, (fp: *mut Fighter) -> f32);
    fn ftcommon_get_model_scale_port(fp: *mut Fighter) -> f32 {
        melee_fighter::ftcommon2::get_model_scale(unsafe { &*fp })
    }
    pub static mut HSD_JOBJSETSCALE: Option<fn(*mut HSD_JObj, *const Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_JObjSetScale(jobj: *mut HSD_JObj, scale: *const Vec3)  {
                match unsafe { HSD_JOBJSETSCALE } {
                    Some(f) => f(jobj, scale),
                    None => { unsafe { crate::tu::ftinl::HSD_JObjSetScale(jobj as _, scale as _); } return; },
                }
            }
    pub static mut LB_8000C2F8: Option<fn(*mut HSD_JObj, *mut HSD_JObj) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000C2F8(jobj: *mut HSD_JObj, parent: *mut HSD_JObj)  {
                match unsafe { LB_8000C2F8 } {
                    Some(f) => f(jobj, parent),
                    None => { unsafe { melee_runtime::tu::lb_00B0::lb_8000C2F8(jobj as _, parent as _); } return; },
                }
            }
    pub static mut FTCO_800BFFD0: Option<fn(*mut Fighter, i32, i32) -> bool> = None;
    #[allow(non_snake_case)]
    pub fn ftCo_800BFFD0(fp: *mut Fighter, a: i32, b: i32) -> bool {
        match unsafe { FTCO_800BFFD0 } {
            Some(f) => f(fp, a, b),
            None => { unsafe { melee_fighter::tu::ftcolanim::ftco_800bffd0(&mut *(fp as *mut _), a as _, b as _) as _ } },
        }
    }
    crate::port_hook!(FTCOLL_8007ABD0, ftColl_8007ABD0, crate::adapt::ftColl_8007ABD0, (hit: *mut HitCapsule, dmg: u32, gobj: *mut HSD_GObj));
}

type G = *mut HSD_GObj;

#[inline(always)]
unsafe fn fpp(gobj: G) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *const ftKb_DatAttrs {
    (*fp).dat_attrs as *const ftKb_DatAttrs
}
/// `hat.kind == Ft_Kind_Mars ? &da->ms : &da->fe`.
#[inline(always)]
unsafe fn ms_da(fp: *mut Fighter) -> *const ftKb_SpecialNMs_DatAttrs {
    let da = da(fp);
    if (*fp).u.kb.hat.kind as u32 == Ft_Kind_Mars as u32 {
        &(*da).ms
    } else {
        &(*da).fe
    }
}
/// Marth id when the hat is Marth, else the Roy (Fe) id.
#[inline(always)]
unsafe fn ms_or_fe(fp: *mut Fighter, ms: u32, fe: u32) -> i32 {
    if (*fp).u.kb.hat.kind as u32 == Ft_Kind_Mars as u32 {
        ms as i32
    } else {
        fe as i32
    }
}
/// `Fighter_SetEffectHitlagCallbacks` (ft/inlines.h).
#[inline(always)]
unsafe fn set_hitlag_cbs(fp: *mut Fighter) {
    (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
    (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
}

/// Accessory callback entry for `fn_8010B1F4`.
pub unsafe extern "C" fn cb_fn_8010B1F4(g: G) {
    melee_core::ct!("cb_fn_8010B1F4");
    fn_8010B1F4(g);
}
/// In-state-change callback entry for `fn_8010B2E8`.
pub unsafe extern "C" fn cb_fn_8010B2E8(g: G) {
    melee_core::ct!("cb_fn_8010B2E8");
    fn_8010B2E8(g);
}

pub fn fn_8010B1F4(gobj: G) {
    melee_core::ct!("fn_8010B1F4");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).x2219_b0() == 0 {
            let top = (*(*fp).parts.add(FtPart_TopN as usize)).joint as usize;
            if (*fp).u.kb.hat.kind as u32 == Ft_Kind_Mars as u32 {
                if (*fp).ground_or_air as u32 == GA_Ground as u32 {
                    ext::efSync_Spawn(0x4B4, gobj, &[top]);
                } else {
                    ext::efSync_Spawn(0x4B5, gobj, &[top]);
                }
            } else if (*fp).ground_or_air as u32 == GA_Ground as u32 {
                ext::efSync_Spawn(0x4B8, gobj, &[top]);
            } else {
                ext::efSync_Spawn(0x4B9, gobj, &[top]);
            }
            (*fp).set_x2219_b0(1);
        }
        set_hitlag_cbs(fp);
        (*fp).accessory4_cb = None;
    }
}

pub fn fn_8010B2E8(gobj: G) {
    melee_core::ct!("fn_8010B2E8");
    unsafe {
        let fp = fpp(gobj);
        (*fp).cmd_vars[0] = 0;
        (*fp).mv.ms.specialn.cur_frame = 0;
    }
}

/// `setupStartAccessory` (static inline).
unsafe fn setup_start_accessory(gobj: G, scale: *mut Vec3) {
    let fp = fpp(gobj);
    let mars_hat = ext::ft_80459B88_hat(Ft_Kind_Mars as i32 - 1);
    let fe_hat = ext::ft_80459B88_hat(Ft_Kind_Emblem as i32 - 1);

    if (*fp).u.kb.hat.kind as u32 == Ft_Kind_Mars as u32 {
        if mars_hat.is_null() {
            return;
        }
        ext::ftCommon_SetAccessory(fp, (*mars_hat).hat_dynamics[0] as *mut HSD_Joint);
    } else {
        if fe_hat.is_null() {
            return;
        }
        ext::ftCommon_SetAccessory(fp, (*fe_hat).hat_dynamics[0] as *mut HSD_Joint);
    }

    let s = ext::ftCommon_GetModelScale(fp);
    (*scale).z = s;
    (*scale).y = s;
    (*scale).x = s;
    let jobj = (*fp).x20A0_accessory;
    ext::HSD_JObjSetScale(jobj, scale);
    let bone = melee_fighter::hooks::ftparts_get_bone_index(&*fp, FtPart_RThumbNb as i32);
    ext::lb_8000C2F8((*fp).x20A0_accessory, (*(*fp).parts.add(bone as usize)).joint);
}

pub fn ftKb_SpecialNMs_8010B2FC(gobj: G) {
    melee_core::ct!("ftKb_SpecialNMs_8010B2FC");
    unsafe {
        let mut scale: Vec3 = core::mem::zeroed();
        {
            let fp = fpp(gobj);
            let ms_da = ms_da(fp);
            // InStateChange callback
            (*fp).x21EC = Some(cb_fn_8010B2E8);
            (*fp).gr_vel = (*fp).gr_vel / (*ms_da).air_horizontal_momentum_preservation;
        }
        {
            let msid = ms_or_fe(fpp(gobj), ftKb_MS_MsSpecialNStart as u32, ftKb_MS_FeSpecialNStart as u32);
            st::Fighter_ChangeMotionState(gobj, msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        }
        st::ftAnim_8006EBA4(gobj);
        setup_start_accessory(gobj, &mut scale);
    }
}

pub fn ftKb_SpecialNMs_8010B4A0(gobj: G) {
    melee_core::ct!("ftKb_SpecialNMs_8010B4A0");
    unsafe {
        {
            let fp = fpp(gobj);
            let ms_da = ms_da(fp);
            // InStateChange callback
            (*fp).x21EC = Some(cb_fn_8010B2E8);
            (*fp).self_vel.x = (*fp).self_vel.x / (*ms_da).air_horizontal_momentum_preservation;
            if (*fp).self_vel.y <= 0.0 {
                (*fp).self_vel.y = 0.0;
            }
        }
        {
            let msid = ms_or_fe(fpp(gobj), ftKb_MS_MsSpecialAirNStart as u32, ftKb_MS_FeSpecialAirNStart as u32);
            st::Fighter_ChangeMotionState(gobj, msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        }
        st::ftAnim_8006EBA4(gobj);
        {
            let mut scale: Vec3 = core::mem::zeroed();
            setup_start_accessory(gobj, &mut scale);
        }
    }
}

unsafe fn start_anim_tail(gobj: G) {
    let fp = fpp(gobj);
    if (*fp).u.kb.hat.kind as u32 == Ft_Kind_Mars as u32 {
        ext::ftCo_800BFFD0(fp, 99, 0);
    } else {
        ext::ftCo_800BFFD0(fp, 100, 0);
    }
}

pub fn ftKb_MsSpecialNStart_Anim(gobj: G) {
    melee_core::ct!("ftKb_MsSpecialNStart_Anim");
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            ftKb_SpecialNMs_8010BC40(gobj);
            start_anim_tail(gobj);
        }
    }
}

pub fn ftKb_MsSpecialAirNStart_Anim(gobj: G) {
    melee_core::ct!("ftKb_MsSpecialAirNStart_Anim");
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            ftKb_SpecialNMs_8010BC90(gobj);
            start_anim_tail(gobj);
        }
    }
}

pub fn ftKb_MsSpecialNStart_IASA(_gobj: G) { melee_core::ct!("ftKb_MsSpecialNStart_IASA");}
pub fn ftKb_MsSpecialAirNStart_IASA(_gobj: G) { melee_core::ct!("ftKb_MsSpecialAirNStart_IASA");}

pub fn ftKb_MsSpecialNStart_Phys(gobj: G) {
    melee_core::ct!("ftKb_MsSpecialNStart_Phys");
    unsafe {
        let fp = fpp(gobj);
        let ms_da = ms_da(fp);
        melee_fighter::ftcommon::calc_ground_accel_deaccel(&mut *fp, (*ms_da).air_horizontal_deceleration_rate);
        melee_fighter::ftcommon::set_self_movement_from_grounded_movement(&mut *fp);
    }
}

pub fn ftKb_MsSpecialAirNStart_Phys(gobj: G) {
    melee_core::ct!("ftKb_MsSpecialAirNStart_Phys");
    unsafe {
        let fp = fpp(gobj);
        let ms_da = ms_da(fp);
        melee_fighter::ftcommon::fall_basic(&mut *fp);
        melee_fighter::ftcommon::calc_self_accel_deaccel(&mut *fp, (*ms_da).air_horizontal_deceleration_rate);
    }
}

pub fn ftKb_MsSpecialNStart_Coll(gobj: G) {
    melee_core::ct!("ftKb_MsSpecialNStart_Coll");
    unsafe {
        if st::ft_80082708(gobj) == GA_Ground as i32 {
            ftKb_SpecialNMs_8010B868(gobj);
        }
    }
}

pub fn ftKb_MsSpecialAirNStart_Coll(gobj: G) {
    melee_core::ct!("ftKb_MsSpecialAirNStart_Coll");
    unsafe {
        if st::ft_80081D0C(gobj) != GA_Ground as i32 {
            ftKb_SpecialNMs_8010B8E0(gobj);
        }
    }
}

pub fn ftKb_SpecialNMs_8010B868(gobj: G) {
    melee_core::ct!("ftKb_SpecialNMs_8010B868");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_8007D5D4(&mut *fp);
        let msid = ms_or_fe(fpp(gobj), ftKb_MS_MsSpecialAirNStart as u32, ftKb_MS_FeSpecialAirNStart as u32);
        st::Fighter_ChangeMotionState(gobj, msid, 0x0C4C7084, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
    }
}

pub fn ftKb_SpecialNMs_8010B8E0(gobj: G) {
    melee_core::ct!("ftKb_SpecialNMs_8010B8E0");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_8007D7FC(&mut *fp);
        let msid = ms_or_fe(fpp(gobj), ftKb_MS_MsSpecialNStart as u32, ftKb_MS_FeSpecialNStart as u32);
        st::Fighter_ChangeMotionState(gobj, msid, 0x0C4C7084, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
    }
}

/// Shared by the two Loop Anim callbacks: count the charge frame; past `charge_iterations * 30`
/// release through `release`.
unsafe fn loop_anim(gobj: G, release: fn(G)) {
    let fp = fpp(gobj);
    let ms_da = ms_da(fp);
    let charge = (*ms_da).charge_iterations as i32;
    (*fp).mv.ms.specialn.cur_frame += 1;
    if (*fp).mv.ms.specialn.cur_frame > charge.wrapping_mul(30) {
        (*fp).cmd_vars[0] = 1;
        release(gobj);
    }
}

pub fn ftKb_MsSpecialNLoop_Anim(gobj: G) {
    melee_core::ct!("ftKb_MsSpecialNLoop_Anim");
    unsafe { loop_anim(gobj, ftKb_SpecialNPe_8010C148) }
}

pub fn ftKb_MsSpecialAirNLoop_Anim(gobj: G) {
    melee_core::ct!("ftKb_MsSpecialAirNLoop_Anim");
    unsafe { loop_anim(gobj, ftKb_SpecialNPe_8010C1E8) }
}

pub fn ftKb_MsSpecialNLoop_IASA(gobj: G) {
    melee_core::ct!("ftKb_MsSpecialNLoop_IASA");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).input.held_buttons[0] & (HSD_PAD_B as u32) == 0 {
            (*fp).cmd_vars[0] = 0;
            ftKb_SpecialNPe_8010C148(gobj);
        }
    }
}

pub fn ftKb_MsSpecialAirNLoop_IASA(gobj: G) {
    melee_core::ct!("ftKb_MsSpecialAirNLoop_IASA");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).input.held_buttons[0] & (HSD_PAD_B as u32) == 0 {
            (*fp).cmd_vars[0] = 0;
            ftKb_SpecialNPe_8010C1E8(gobj);
        }
    }
}

pub fn ftKb_MsSpecialNLoop_Coll(gobj: G) {
    melee_core::ct!("ftKb_MsSpecialNLoop_Coll");
    unsafe {
        if st::ft_80082708(gobj) == GA_Ground as i32 {
            ftKb_SpecialNMs_8010BB50(gobj);
        }
    }
}

pub fn ftKb_MsSpecialAirNLoop_Coll(gobj: G) {
    melee_core::ct!("ftKb_MsSpecialAirNLoop_Coll");
    unsafe {
        if st::ft_80081D0C(gobj) != GA_Ground as i32 {
            ftKb_SpecialNMs_8010BBC8(gobj);
        }
    }
}

pub fn ftKb_SpecialNMs_8010BB50(gobj: G) {
    melee_core::ct!("ftKb_SpecialNMs_8010BB50");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_8007D5D4(&mut *fp);
        let msid = ms_or_fe(fpp(gobj), ftKb_MS_MsSpecialAirNLoop as u32, ftKb_MS_FeSpecialAirNLoop as u32);
        st::Fighter_ChangeMotionState(gobj, msid, 0x0C4C7A86, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
    }
}

pub fn ftKb_SpecialNMs_8010BBC8(gobj: G) {
    melee_core::ct!("ftKb_SpecialNMs_8010BBC8");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_8007D7FC(&mut *fp);
        let msid = ms_or_fe(fpp(gobj), ftKb_MS_MsSpecialNLoop as u32, ftKb_MS_FeSpecialNLoop as u32);
        st::Fighter_ChangeMotionState(gobj, msid, 0x0C4C7A86, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
    }
}

pub fn ftKb_SpecialNMs_8010BC40(gobj: G) {
    melee_core::ct!("ftKb_SpecialNMs_8010BC40");
    unsafe {
        let fp = fpp(gobj);
        let i = if (*fp).u.kb.hat.kind as u32 == Ft_Kind_Mars as u32 { // retail: the word at fp+0x2238 (u.gw.x2238_panicCharge in the source) = Kirby's hat.kind on GC
            ftKb_MS_MsSpecialNLoop as i32
        } else {
            ftKb_MS_FeSpecialNLoop as i32
        };
        st::Fighter_ChangeMotionState(gobj, i, 0x3200, 0.0, 1.0, 0.0, core::ptr::null_mut());
    }
}

pub fn ftKb_SpecialNMs_8010BC90(gobj: G) {
    melee_core::ct!("ftKb_SpecialNMs_8010BC90");
    unsafe {
        let fp = fpp(gobj);
        let i = if (*fp).u.kb.hat.kind as u32 == Ft_Kind_Mars as u32 { // retail: the word at fp+0x2238 (u.gw.x2238_panicCharge in the source) = Kirby's hat.kind on GC
            ftKb_MS_MsSpecialAirNLoop as i32
        } else {
            ftKb_MS_FeSpecialAirNLoop as i32
        };
        st::Fighter_ChangeMotionState(gobj, i, 0x3200, 0.0, 1.0, 0.0, core::ptr::null_mut());
    }
}

/// Shared hit-damage scaling of the two End Anim callbacks.
unsafe fn end_anim_hits(gobj: G) {
    let fp = fpp(gobj);
    let ms_da = ms_da(fp);
    if (*fp).cmd_vars[0] == 0 {
        let mut i = 0usize;
        while i < 4 {
            let hit = (*fp).x914.as_mut_ptr().add(i);
            if (*hit).state as u32 == HitCapsule_Enabled as u32 {
                let per = sdiv((*fp).mv.ms.specialn.cur_frame, 30) as u32;
                let raw = (*ms_da)
                    .base_damage
                    .wrapping_add(per.wrapping_mul((*ms_da).additional_damage_per_iteration));
                // `(u32)(f32)(s32)raw`
                let dmg = melee_core::ppc::fcvt_u32((raw as i32 as f32));
                ext::ftColl_8007ABD0(hit, dmg, gobj);
            }
            i += 1;
        }
    }
}

pub fn ftKb_MsSpecialNEnd_Anim(gobj: G) {
    melee_core::ct!("ftKb_MsSpecialNEnd_Anim");
    unsafe {
        end_anim_hits(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftKb_MsSpecialAirNEnd_Anim(gobj: G) {
    melee_core::ct!("ftKb_MsSpecialAirNEnd_Anim");
    unsafe {
        end_anim_hits(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCo_Fall_Enter(gobj);
        }
    }
}

pub fn ftKb_MsSpecialNEnd_IASA(_gobj: G) { melee_core::ct!("ftKb_MsSpecialNEnd_IASA");}
pub fn ftKb_MsSpecialAirNEnd_IASA(_gobj: G) { melee_core::ct!("ftKb_MsSpecialAirNEnd_IASA");}

pub fn ftKb_MsSpecialNEnd_Coll(gobj: G) {
    melee_core::ct!("ftKb_MsSpecialNEnd_Coll");
    unsafe {
        if st::ft_80082708(gobj) == GA_Ground as i32 {
            ftKb_SpecialNPe_8010BF90(gobj);
        }
    }
}

pub fn ftKb_MsSpecialAirNEnd_Coll(gobj: G) {
    melee_core::ct!("ftKb_MsSpecialAirNEnd_Coll");
    unsafe {
        if st::ft_80081D0C(gobj) != GA_Ground as i32 {
            ftKb_SpecialNPe_8010C06C(gobj);
        }
    }
}

/// Pick the `End0`/`End1` state (End1 is End0 + 1) by `cmd_vars[0]` and the hat.
unsafe fn end_msid(fp: *mut Fighter, ms0: u32, fe0: u32) -> i32 {
    let add = if (*fp).cmd_vars[0] == 0 { 0 } else { 1 };
    ms_or_fe(fp, ms0, fe0) + add
}

pub fn ftKb_SpecialNPe_8010BF90(gobj: G) {
    melee_core::ct!("ftKb_SpecialNPe_8010BF90");
    unsafe {
        let fp = fpp(gobj);
        let msid = end_msid(fp, ftKb_MS_MsSpecialAirNEnd0 as u32, ftKb_MS_FeSpecialAirNEnd0 as u32);
        st::ftCommon_8007D5D4(&mut *fp);
        st::Fighter_ChangeMotionState(gobj, msid, 0x0C4C708E, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
        if (*fp).x2219_b0() == 1 {
            set_hitlag_cbs(fp);
        }
    }
}

pub fn ftKb_SpecialNPe_8010C06C(gobj: G) {
    melee_core::ct!("ftKb_SpecialNPe_8010C06C");
    unsafe {
        let fp = fpp(gobj);
        let msid = end_msid(fp, ftKb_MS_MsSpecialNEnd0 as u32, ftKb_MS_FeSpecialNEnd0 as u32);
        st::ftCommon_8007D7FC(&mut *fp);
        st::Fighter_ChangeMotionState(gobj, msid, 0x0C4C708E, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
        if (*fp).x2219_b0() == 1 {
            set_hitlag_cbs(fp);
        }
    }
}

pub fn ftKb_SpecialNPe_8010C148(gobj: G) {
    melee_core::ct!("ftKb_SpecialNPe_8010C148");
    unsafe {
        let fp = fpp(gobj);
        let msid = end_msid(fp, ftKb_MS_MsSpecialNEnd0 as u32, ftKb_MS_FeSpecialNEnd0 as u32);
        st::Fighter_ChangeMotionState(gobj, msid, 0x2000, 1.0, 1.0, 0.0, core::ptr::null_mut());
        (*fpp(gobj)).accessory4_cb = Some(cb_fn_8010B1F4);
    }
}

pub fn ftKb_SpecialNPe_8010C1E8(gobj: G) {
    melee_core::ct!("ftKb_SpecialNPe_8010C1E8");
    unsafe {
        let fp = fpp(gobj);
        let msid = end_msid(fp, ftKb_MS_MsSpecialAirNEnd0 as u32, ftKb_MS_FeSpecialAirNEnd0 as u32);
        st::Fighter_ChangeMotionState(gobj, msid, 0x2000, 1.0, 1.0, 0.0, core::ptr::null_mut());
        (*fpp(gobj)).accessory4_cb = Some(cb_fn_8010B1F4);
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftKb_MsSpecialNLoop_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialmars.c).
pub fn ftKb_MsSpecialNLoop_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_MsSpecialNLoop_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftKb_MsSpecialAirNLoop_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialmars.c).
pub fn ftKb_MsSpecialAirNLoop_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_MsSpecialAirNLoop_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftKb_MsSpecialNEnd_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialmars.c).
pub fn ftKb_MsSpecialNEnd_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_MsSpecialNEnd_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftKb_MsSpecialAirNEnd_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialmars.c).
pub fn ftKb_MsSpecialAirNEnd_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_MsSpecialAirNEnd_Phys");
    st::ft_80084EEC(gobj);
}

// ---- folded from chars blocks (consolidation step 20) ----
pub fn fn_8010B2E8_fp(fp: &mut Fighter) {
    melee_core::ct!("fn_8010B2E8_fp");
    fp.cmd_vars[0] = 0;
    unsafe { fp.mv.ms.specialn.cur_frame = 0 as _ };
}
