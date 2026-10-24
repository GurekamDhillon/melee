//! src/melee/ft/kinds/ftKirby/ftkirbyyoshiegg.c: the egg a Yoshi-Kirby swallow wraps its victim in.
//! Fused sites (tools/rust-port/fma_sites.py): `fn_8010B16C` 1 (`grab_timer -= percentTemp *
//! f()`, the IR negates the left factor: `fmadds(-percentTemp, f(), grab_timer)`). Smoke-tested
//! through recording hooks by tests/tu_ftkirbyyoshiegg.rs (the egg-shrink scale math and the
//! fused grab timer are checked against hand-computed values).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use crate::tu::ftkirbyspecialyoshi as ys;
use melee_core::fp::fmadds;
use melee_types::*;

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
    pub static mut HSD_JOBJSETSCALE: Option<fn(*mut HSD_JObj, *const Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_JObjSetScale(jobj: *mut HSD_JObj, scale: *const Vec3)  {
                match unsafe { HSD_JOBJSETSCALE } {
                    Some(f) => f(jobj, scale),
                    None => { unsafe { let _ = crate::tu::ftinl::HSD_JObjSetScale(jobj as _, scale as _); } return; },
                }
            }
    pub static mut HSD_JOBJGETSCALE: Option<fn(*mut HSD_JObj, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_JObjGetScale(jobj: *mut HSD_JObj, out: *mut Vec3)  {
                match unsafe { HSD_JOBJGETSCALE } {
                    Some(f) => f(jobj, out),
                    None => { unsafe { let _ = crate::tu::ftinl::HSD_JObjGetScale(jobj as _, out as _); } return; },
                }
            }
    pub static mut HSD_JOBJGETCHILD: Option<fn(*mut HSD_JObj) -> *mut HSD_JObj> = None;
            #[allow(non_snake_case)]
            pub fn HSD_JObjGetChild(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
                match unsafe { HSD_JOBJGETCHILD } {
                    Some(f) => f(jobj),
                    None => { return unsafe { melee_items::tu::kinds_inl::HSD_JObjGetChild(jobj as _) as _ }; },
                }
            }
    pub static mut LB_8000C2F8: Option<fn(*mut HSD_JObj, *mut HSD_JObj) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000C2F8(jobj: *mut HSD_JObj, parent: *mut HSD_JObj)  {
                match unsafe { LB_8000C2F8 } {
                    Some(f) => f(jobj, parent),
                    None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000C2F8(jobj as _, parent as _); } return; },
                }
            }
    hook!(FTCOLL_8007B0C0, ftColl_8007B0C0, (gobj: *mut HSD_GObj, state: i32));
    hook!(FTCOLL_HURTBOXINIT, ftColl_HurtboxInit, (fp: *mut Fighter, hurt_capsules: *mut FighterHurtCapsule, init: *mut ftHurtboxInit));
    hook!(FTCOLL_8007B760, ftColl_8007B760, (gobj: *mut HSD_GObj, v: i32));
    hook!(FTCOMMON_SETACCESSORY, ftCommon_SetAccessory, (fp: *mut Fighter, joint: *mut HSD_Joint));
    hook!(FTCOMMON_INITGRAB, ftCommon_InitGrab, (fp: *mut Fighter, a: i32, b: f32));
    hook!(FTCOMMON_GRABMASH, ftCommon_GrabMash, (fp: *mut Fighter, decrement: f32) -> i32);
    // `efAsync_Spawn(gobj, queue, kind, gfx_id, ...)` (ef/efasync.c, variadic): trailing args as a slice.
    hook!(EFASYNC_SPAWN, efAsync_Spawn, (gobj: *mut HSD_GObj, queue: *mut *mut c_void, kind: u32, gfx_id: u32, args: &[*mut c_void]));
    /// Recording hook; unset, the native `melee_fighter::tu::fighter` port runs.
    pub static mut FIGHTER_UPDATEMODELSCALE: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn Fighter_UpdateModelScale(gobj: *mut HSD_GObj) {
        match unsafe { FIGHTER_UPDATEMODELSCALE } {
            Some(f) => f(gobj),
            None => unsafe { melee_fighter::tu::fighter::Fighter_UpdateModelScale(gobj) },
        }
    }
    /// Recording hook; unset, the native `melee_fighter::tu::ftCo_Fall` port runs.
    pub static mut FTCO_FALL_ENTER_YOSHIEGG_KIRBY: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn ftCo_Fall_Enter_YoshiEgg_Kirby(gobj: *mut HSD_GObj) {
        match unsafe { FTCO_FALL_ENTER_YOSHIEGG_KIRBY } {
            Some(f) => f(gobj),
            None => melee_fighter::tu::ftCo_Fall::ftCo_Fall_Enter_YoshiEgg_Kirby(gobj),
        }
    }
    /// Recording hook; unset, the native `melee_fighter::tu::ftCo_Bury` port runs.
    pub static mut FTCO_800C0C88: Option<fn(i32) -> bool> = None;
    #[allow(non_snake_case)]
    pub fn ftCo_800C0C88(arg0: i32) -> bool {
        match unsafe { FTCO_800C0C88 } {
            Some(f) => f(arg0),
            None => melee_fighter::tu::ftCo_Bury::ftCo_800C0C88(arg0),
        }
    }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `fn_8010AA64` as a Fighter callback entry (`accessory4_cb`).
pub unsafe extern "C" fn cb_fn_8010AA64(g: *mut HSD_GObj) {
    fn_8010AA64(g);
}
/// `fn_8010B16C` as a Fighter callback entry (`take_dmg_2_cb`).
pub unsafe extern "C" fn cb_fn_8010B16C(g: *mut HSD_GObj) {
    fn_8010B16C(g);
}
/// `fn_8010B1D4` as a Fighter callback entry (`take_dmg_cb`).
pub unsafe extern "C" fn cb_fn_8010B1D4(g: *mut HSD_GObj) {
    fn_8010B1D4(g);
}
/// `fn_8010B124` / `fn_8010B148` as Fighter callback entries (landing / leave-ground).
pub unsafe extern "C" fn cb_fn_8010B124(g: *mut HSD_GObj) {
    fn_8010B124(g);
}
pub unsafe extern "C" fn cb_fn_8010B148(g: *mut HSD_GObj) {
    fn_8010B148(g);
}

/// `fn_8010AA64` (static): the egg shrinks back to Kirby's scale over `x10` frames.
pub fn fn_8010AA64(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
        if (*fp).mv.co.yoshiegg.x14 <= 0.0 {
            ext::Fighter_UpdateModelScale(gobj);
            (*fp).accessory4_cb = None;
            return;
        }
        (*fp).mv.co.yoshiegg.x14 -= 1.0;
        let mut temp0 = ((*fp).mv.co.yoshiegg.x10 - (*fp).mv.co.yoshiegg.x14) / (*fp).mv.co.yoshiegg.x10;
        temp0 *= (*fp).mv.co.yoshiegg.xC;
        let temp1 = (1.0_f32 - (*fp).mv.co.yoshiegg.xC) + temp0;
        let mut vec1 = Vec3 { x: temp1, y: temp1, z: temp1 };
        let mut vec0 = vec1;
        vec1.x = vec1.x * (*fp).mv.co.yoshiegg.x18.x;
        vec1.y *= (*fp).mv.co.yoshiegg.x18.y;
        vec1.z *= (*fp).mv.co.yoshiegg.x18.z;
        ext::HSD_JObjSetScale(jobj, &vec1);
        vec0.x = vec0.x * (*fp).mv.co.yoshiegg.scale.x;
        vec0.y *= (*fp).mv.co.yoshiegg.scale.y;
        vec0.z *= (*fp).mv.co.yoshiegg.scale.z;
        ext::HSD_JObjSetScale((*fp).x20A0_accessory, &vec0);
    }
}

/// `inlineB0` (static inline): rebuild the victim's single hurtbox around the egg.
pub fn inlineB0(gobj: *mut HSD_GObj, hurt: &mut ftHurtboxInit) {
    unsafe {
        let fp = fpp(gobj);
        ext::ftColl_8007B0C0(gobj, HurtCapsule_Intangible as i32);
        let xbc = &(*fp).co_attrs.xBC;
        hurt.bone_idx = melee_fighter::ftdmgfall::ftparts_get_bone_index(&*fp, FtPart_TransN as i32) as _;
        hurt.height = HurtHeight_Mid as _;
        hurt.is_grabbable = 0;
        hurt.a_offset = xbc.x4;
        hurt.b_offset = xbc.x10;
        hurt.scale = xbc.x1C;
        ext::ftColl_HurtboxInit(fp, (*fp).hurt_capsules.as_mut_ptr(), hurt);
    }
}

pub fn ftKb_SpecialNYs_8010AC78(victim: *mut HSD_GObj, gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(victim);
        let jobj = (*victim).hsd_obj as *mut HSD_JObj;
        let mut hurt = ftHurtboxInit::zeroed();
        if (*fp).ground_or_air == GA_Ground {
            st::ftCommon_8007D5D4(&mut *fp);
        }
        st::Fighter_ChangeMotionState(victim, ftCo_MS_KirbyYoshiEgg as i32, Ft_MF_Unk06 as u32, 0.0, 1.0, 0.0, gobj);
        (*fp).take_dmg_cb = Some(cb_fn_8010B1D4);
        st::ftCommon_8007E2F4(&mut *fp, 0x1FF);
        (*fp).set_invisible(1);
        let size = (*fp).co_attrs.xBC.size;
        let scale = Vec3 { x: size, y: size, z: size };
        ext::ftCommon_SetAccessory(fp, ys::ftKb_SpecialNYs_801093A0(gobj) as *mut HSD_Joint);
        ext::HSD_JObjSetScale((*fp).x20A0_accessory, &scale);
        let bone = melee_fighter::ftdmgfall::ftparts_get_bone_index(&*fp, FtPart_TransN as i32);
        ext::lb_8000C2F8(ext::HSD_JObjGetChild((*fp).x20A0_accessory), (*(*fp).parts.add(bone as usize)).joint);
        (*fp).mv.co.yoshiegg.scale = scale;
        inlineB0(victim, &mut hurt);
        ys::ftKb_SpecialNYs_80109260(gobj, &mut (*fp).self_vel);
        (*fp).facing_dir = ys::ftKb_SpecialNYs_80109290(gobj);
        (*fp).dmg.x182c_behavior = ys::ftKb_SpecialNYs_8010929C(gobj);
        (*fp).mv.ca.specialhi.vel.y = 0.0;
        (*fp).mv.co.yoshiegg.x0 = gobj;
        (*fp).take_dmg_2_cb = Some(cb_fn_8010B16C);
        melee_fighter::ftcommon2::ftcommon_8007efc0(&mut *fp, 1);
        (*fp).mv.co.walk.fast_anim_frame = ys::ftKb_SpecialNYs_801092BC(gobj);
        (*fp).mv.co.common.x14 = (*fp).mv.co.walk.fast_anim_frame as _;
        (*fp).mv.co.common.x4.z = ys::ftKb_SpecialNYs_801092AC(gobj);
        ext::ftCommon_InitGrab(fp, 0, ys::ftKb_SpecialNYs_801092CC(gobj));
        ext::HSD_JObjGetScale(jobj, &mut (*fp).mv.co.yoshiegg.x18);
        (*fp).accessory4_cb = Some(cb_fn_8010AA64);
    }
}

pub fn ftCo_KirbyYoshiEgg_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let temp_ret = ys::ftKb_SpecialNYs_801092DC();
        (*fp).grab_timer -= temp_ret;
        (*fp).mv.kb.specialhi.x4 = ext::ftCommon_GrabMash(fp, ys::ftKb_SpecialNYs_801092F4());
        if (*fp).grab_timer <= 0.0 {
            melee_fighter::hooks::play_sfx(&mut *fp, 0x44618, 0x7F, 0x40);
            let fp2 = fpp(gobj);
            let xbc = &mut (*fp).co_attrs.xBC as *mut ftCo_DatAttrs_xBC_t;
            ext::efAsync_Spawn(
                gobj,
                &mut (*fp2).x60C as *mut *mut core::ffi::c_void,
                4,
                0x4CF,
                &[(*(*fp).parts).joint as *mut _, xbc as *mut _],
            );
            ys::ftKb_SpecialNYs_80109354(&mut (*fp).self_vel);
            st::ftCommon_8007D5D4(&mut *fp);
            ext::ftColl_8007B760(gobj, ys::ftKb_SpecialNYs_8010933C());
            ext::ftCo_Fall_Enter_YoshiEgg_Kirby(gobj);
            return;
        }
        if (*fp).mv.kb.specialhi.x8.f != 0.0 {
            (*fp).mv.kb.specialhi.x8.f -= 1.0;
            if (*fp).mv.kb.specialhi.x8.f <= 0.0 && (*fp).mv.kb.specialhi.x4 == 0 {
                st::ftAnim_SetAnimRate(gobj, 1.0);
                (*fp).mv.kb.specialhi.x8.f = 0.0;
            }
        }
        if (*fp).mv.kb.specialhi.x8.f <= 0.0 && (*fp).mv.kb.specialhi.x4 != 0 {
            (*fp).mv.kb.specialhi.x8.f = ys::ftKb_SpecialNYs_8010930C();
            st::ftAnim_SetAnimRate(gobj, ys::ftKb_SpecialNYs_80109324());
        }
    }
}

pub fn ftCo_KirbyYoshiEgg_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftCo_KirbyYoshiEgg_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).ground_or_air == GA_Ground {
            st::ft_80084F3C(gobj);
            return;
        }
        st::ft_80084DB0(gobj);
    }
}

pub fn ftCo_KirbyYoshiEgg_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).ground_or_air == GA_Ground {
            st::ft_8008403C(gobj, Some(cb_fn_8010B148));
            return;
        }
        st::ft_80082C74(gobj, Some(cb_fn_8010B124));
    }
}

pub fn fn_8010B124(gobj: *mut HSD_GObj) {
    unsafe { st::ftCommon_8007D7FC(&mut *fpp(gobj)) }
}

pub fn fn_8010B148(gobj: *mut HSD_GObj) {
    unsafe { st::ftCommon_8007D5D4(&mut *fpp(gobj)) }
}

pub fn fn_8010B16C(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        // `grab_timer -= x1838_percentTemp * f()`: the IR is fmuladd(-x1838, f(), grab_timer).
        let f = ys::ftKb_SpecialNYs_80109380();
        (*fp).grab_timer = fmadds(-(*fp).dmg.x1838_percentTemp, f, (*fp).grab_timer);
        if (*fp).dmg.x18CC == 3 && ext::ftCo_800C0C88((*fp).dmg.x18D0) {
            (*fp).grab_timer = 0.0;
        }
        (*fp).x1828 = 4;
    }
}

pub fn fn_8010B1D4(gobj: *mut HSD_GObj) {
    ext::Fighter_UpdateModelScale(gobj);
}
