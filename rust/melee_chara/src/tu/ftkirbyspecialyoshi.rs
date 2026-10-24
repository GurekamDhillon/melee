//! src/melee/ft/kinds/ftKirby/ftkirbyspecialyoshi.c: Kirby's Yoshi copy ability (egg lay /
//! swallow) and its Yoshi-hat helpers. No fused sites (tools/rust-port/fma_sites.py: 0 fmuladd
//! in every function). Smoke-tested through recording hooks by tests/tu_ftkirbyspecialyoshi.rs.
//!
//! The getters that read the attributes of the fighter (`ftKb_SpecialNYs_80109260` ..
//! `801092CC`) exist in `chars::ftKirby::ftkirbyspecialyoshi` as pure helpers on `&Fighter`;
//! the functions below keep the C signature (a `gobj`) and delegate to them.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::chars::ftKirby::ftkirbyspecialyoshi as pure;
use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2, 3, 5): re-exported under this TU.
pub use crate::chars::r2::ftKirby::ftkirbyspecialyoshi::{
    ftKb_YsSpecialAirN2_0_Phys, ftKb_YsSpecialAirN2_1_Phys, ftKb_YsSpecialAirNCapture1_Phys,
    ftKb_YsSpecialAirNCapture2_Phys, ftKb_YsSpecialAirCapture1_Phys, ftKb_YsSpecialAirCapture2_Phys,
    ftKb_YsSpecialN1_Phys, ftKb_YsSpecialNCapture1_Phys, ftKb_YsSpecialNCapture2_0_Phys,
    ftKb_YsSpecialNCapture2_1_Phys,
};
pub use crate::chars::r3::ftKirby::ftkirbyspecialyoshi::{fn_80109A08, fn_80109B00, fn_80109C74};
pub use crate::chars::r5::ftKirby::ftkirbyspecialyoshi::{
    ftKb_YsSpecialAirN2_0_Coll, ftKb_YsSpecialAirN2_1_Coll, ftKb_YsSpecialAirNCapture1_Coll,
    ftKb_YsSpecialAirNCapture2_Coll, ftKb_YsSpecialAirCapture1_Coll, ftKb_YsSpecialAirCapture2_Coll,
    ftKb_YsSpecialN1_Coll, ftKb_YsSpecialNCapture1_Coll, ftKb_YsSpecialNCapture2_0_Coll,
    ftKb_YsSpecialNCapture2_1_Coll,
};

/// `ftYs_MF_SpecialN_Coll` (ftYoshi/forward.h): `ftCommon_GroundAirColl_MF | KeepGfx | SkipModel`.
pub const ftYs_MF_SpecialN_Coll: u32 = 0x0C4C5080 | Ft_MF_KeepGfx as u32 | Ft_MF_SkipModel as u32;
/// `ftYs_MF_SpecialN_CollHit`: [`ftYs_MF_SpecialN_Coll`] `| SkipHit`.
pub const ftYs_MF_SpecialN_CollHit: u32 = ftYs_MF_SpecialN_Coll | Ft_MF_SkipHit as u32;

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
    // `ft_80459B88.hats[kind]` (ftdata.c global): null when the hat table is not loaded.
    crate::port_hook!(FT_80459B88_HAT, ft_80459B88_hat, crate::adapt::ft_80459B88_hat, (kind: i32) -> *mut KirbyHatStruct);
    // `ftKb_KirbyFtData()->ext_attr` (ftdata.c): Kirby's ftData extension attributes.
    pub static mut FTKB_KIRBYFTDATA: Option<fn() -> *mut ftData> = None;
            #[allow(non_snake_case)]
            pub fn ftKb_KirbyFtData() -> *mut ftData {
                match unsafe { FTKB_KIRBYFTDATA } {
                    Some(f) => f(),
                    None => { return unsafe { crate::tu::ftkirbyspecialn::ftKb_KirbyFtData() as _ }; },
                }
            }
    crate::port_hook!(HSD_JOBJADDANIMALL, HSD_JObjAddAnimAll, crate::adapt::HSD_JObjAddAnimAll, (jobj: *mut HSD_JObj, anim: *mut HSD_AnimJoint, mat: *mut HSD_MatAnimJoint, shape: *mut HSD_ShapeAnimJoint));
    crate::port_hook!(HSD_JOBJREQANIMALL, HSD_JObjReqAnimAll, crate::adapt::HSD_JObjReqAnimAll, (jobj: *mut HSD_JObj, frame: f32));
    crate::port_hook!(HSD_JOBJANIMALL, HSD_JObjAnimAll, crate::adapt::HSD_JObjAnimAll, (jobj: *mut HSD_JObj));
    pub static mut LB_8000BA0C: Option<fn(*mut HSD_JObj, f32) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000BA0C(jobj: *mut HSD_JObj, frame: f32)  {
                match unsafe { LB_8000BA0C } {
                    Some(f) => f(jobj, frame),
                    None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000BA0C(jobj as _, frame as _); } return; },
                }
            }
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(jobj: *mut HSD_JObj, ofs: *const Vec3, out: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(jobj, ofs, out),
                    None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, ofs as _, out as _); } return; },
                }
            }
    pub static mut IT_802F2D98: Option<fn(*mut HSD_GObj, *mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802F2D98(item: *mut HSD_GObj, gobj: *mut HSD_GObj)  {
                match unsafe { IT_802F2D98 } {
                    Some(f) => f(item, gobj),
                    None => { unsafe { let _ = melee_items::tu::itkirbyyoshispecialn::it_802F2D98(item as _, gobj as _); } return; },
                }
            }
    pub static mut IT_802F2E7C: Option<fn(*mut HSD_GObj, i32) > = None;
            #[allow(non_snake_case)]
            pub fn it_802F2E7C(item: *mut HSD_GObj, destroy_type: i32)  {
                match unsafe { IT_802F2E7C } {
                    Some(f) => f(item, destroy_type),
                    None => { unsafe { let _ = melee_items::tu::itkirbyyoshispecialn::it_802F2E7C(item as _, destroy_type as _); } return; },
                }
            }
    pub static mut IT_802F2F34: Option<fn(*mut HSD_GObj, *mut itYoshiEggLay_DatAttrs) > = None;
            #[allow(non_snake_case)]
            pub fn it_802F2F34(gobj: *mut HSD_GObj, attrs: *mut itYoshiEggLay_DatAttrs)  {
                match unsafe { IT_802F2F34 } {
                    Some(f) => f(gobj, attrs),
                    None => { unsafe { let _ = melee_items::tu::ityoshiegglay::it_802F2F34(gobj as _, attrs as _); } return; },
                }
            }
    /// `fn_8010A930` (ftkirbycaptureyoshi.c): the grabbed (interaction) callback.
    pub static mut FN_8010A930: Option<fn(*mut HSD_GObj, *mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn fn_8010A930(gobj: *mut HSD_GObj, attacker: *mut HSD_GObj)  {
                match unsafe { FN_8010A930 } {
                    Some(f) => f(gobj, attacker),
                    None => { unsafe { let _ = crate::tu::ftkirbycaptureyoshi::fn_8010A930(gobj as _, attacker as _); } return; },
                }
            }
    /// `ftKb_SpecialNYs_8010AA2C` (ftkirbycaptureyoshi.c).
    pub static mut FTKB_SPECIALNYS_8010AA2C: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftKb_SpecialNYs_8010AA2C(gobj: *mut HSD_GObj)  {
                match unsafe { FTKB_SPECIALNYS_8010AA2C } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = crate::tu::ftkirbycaptureyoshi::ftKb_SpecialNYs_8010AA2C(gobj as _); } return; },
                }
            }
    /// Recording hook; unset, the native `melee_fighter::tu::ftCo_Throw` port runs.
    pub static mut FTCO_800DE2CC: Option<fn(*mut HSD_GObj, *mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn ftCo_800DE2CC(gobj: *mut HSD_GObj, victim: *mut HSD_GObj) {
        match unsafe { FTCO_800DE2CC } {
            Some(f) => f(gobj, victim),
            None => melee_fighter::tu::ftCo_Throw::ftCo_800DE2CC(gobj, victim),
        }
    }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *mut ftKb_DatAttrs {
    (*fp).dat_attrs as *mut ftKb_DatAttrs
}
/// `ftKb_KirbyFtData()->ext_attr`.
#[inline(always)]
unsafe fn ea() -> *mut ftKb_DatAttrs {
    (*ext::ftKb_KirbyFtData()).ext_attr as *mut ftKb_DatAttrs
}

/// `ftKb_SpecialN_set_cbs` (ftKirby/inlines.h): Kirby's damage callbacks.
use crate::tu::ftkirbyspecialseak::ftKb_SpecialN_set_cbs;

pub fn ftKb_SpecialNYs_80109260(gobj: *mut HSD_GObj, vec: *mut Vec3) {
    unsafe { pure::ftKb_SpecialNYs_80109260(&*fpp(gobj), &mut *vec) }
}
pub fn ftKb_SpecialNYs_80109290(gobj: *mut HSD_GObj) -> f32 {
    unsafe { pure::ftKb_SpecialNYs_80109290(&*fpp(gobj)) }
}
pub fn ftKb_SpecialNYs_8010929C(gobj: *mut HSD_GObj) -> f32 {
    unsafe { pure::ftKb_SpecialNYs_8010929C(&*fpp(gobj)) }
}
pub fn ftKb_SpecialNYs_801092AC(gobj: *mut HSD_GObj) -> f32 {
    unsafe { pure::ftKb_SpecialNYs_801092AC(&*fpp(gobj)) }
}
pub fn ftKb_SpecialNYs_801092BC(gobj: *mut HSD_GObj) -> f32 {
    unsafe { pure::ftKb_SpecialNYs_801092BC(&*fpp(gobj)) }
}
pub fn ftKb_SpecialNYs_801092CC(gobj: *mut HSD_GObj) -> f32 {
    unsafe { pure::ftKb_SpecialNYs_801092CC(&*fpp(gobj)) }
}

pub fn ftKb_SpecialNYs_801092DC() -> f32 {
    unsafe { (*ea()).specialn_ys_egg_breakout_resistance }
}
pub fn ftKb_SpecialNYs_801092F4() -> f32 {
    unsafe { (*ea()).specialn_ys_frames_reduced_per_input }
}
pub fn ftKb_SpecialNYs_8010930C() -> f32 {
    unsafe { (*ea()).specialn_ys_unk1 }
}
pub fn ftKb_SpecialNYs_80109324() -> f32 {
    unsafe { (*ea()).specialn_ys_unk2 }
}
pub fn ftKb_SpecialNYs_8010933C() -> i32 {
    unsafe { (*ea()).specialn_ys_iframes_on_release as i32 }
}
pub fn ftKb_SpecialNYs_80109354(v: *mut Vec3) {
    unsafe {
        let cd = ea();
        (*v).x = (*cd).specialn_ys_horizontal_velocity_on_breakout;
        (*v).y = (*cd).specialn_ys_vertical_velocity_on_breakout;
        (*v).z = 0.0;
    }
}
pub fn ftKb_SpecialNYs_80109380() -> f32 {
    unsafe {
        let cd = ea();
        (*cd).specialn_ys_unk3 / (*cd).specialn_ys_damage_multiplier
    }
}

/// The Samus-indexed hat slot (the C uses `hats[Ft_Kind_Samus]`), or null.
unsafe fn samus_hat() -> *mut KirbyHatStruct {
    ext::ft_80459B88_hat(Ft_Kind_Samus as i32)
}

pub fn ftKb_SpecialNYs_801093A0(_gobj: *mut HSD_GObj) -> *mut ftDynamics {
    unsafe {
        let hat = samus_hat();
        if hat.is_null() {
            return core::ptr::null_mut();
        }
        (*hat).hat_dynamics[0]
    }
}

/// Attach the hat animation `dyn_idx` to Kirby's hat jobj and rewind it.
unsafe fn start_hat_anim(fp: *mut Fighter, hat: *mut KirbyHatStruct, dyn_idx: usize) {
    ext::HSD_JObjAddAnimAll((*fp).u.kb.hat.jobj, (*hat).hat_dynamics[dyn_idx] as *mut HSD_AnimJoint, core::ptr::null_mut(), core::ptr::null_mut());
    ext::HSD_JObjReqAnimAll((*fp).u.kb.hat.jobj, 0.0);
}

pub fn ftKb_SpecialNYs_801093B4(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let hat = samus_hat();
        if hat.is_null() {
            return;
        }
        start_hat_anim(fp, hat, 1);
        ext::HSD_JObjAnimAll((*fp).u.kb.hat.jobj);
        ext::lb_8000BA0C((*fp).u.kb.hat.jobj, 0.0);
    }
}

/// `fn_801095DC`, `fn_80109680`, `fn_80109714`, `fn_801097B8` as Fighter callback entries.
pub unsafe extern "C" fn cb_fn_801095DC(g: *mut HSD_GObj) {
    fn_801095DC(g);
}
pub unsafe extern "C" fn cb_fn_80109680(g: *mut HSD_GObj) {
    fn_80109680(g);
}
pub unsafe extern "C" fn cb_fn_80109714(g: *mut HSD_GObj) {
    fn_80109714(g);
}
pub unsafe extern "C" fn cb_fn_801097B8(g: *mut HSD_GObj) {
    fn_801097B8(g);
}
/// `fn_8010A930` as a Fighter interaction callback entry.
pub unsafe extern "C" fn cb_fn_8010A930(g: *mut HSD_GObj, attacker: *mut HSD_GObj) {
    ext::fn_8010A930(g, attacker);
}

/// `ftCommon_8007E2D0(GET_FIGHTER(gobj), 4, grab, unk, fn_8010A930)`.
unsafe fn set_grab_cbs(gobj: *mut HSD_GObj, grab: unsafe extern "C" fn(*mut HSD_GObj), unk: unsafe extern "C" fn(*mut HSD_GObj)) {
    melee_fighter::ftcommon2::ftcommon_8007e2d0(&mut *fpp(gobj), 4, Some(grab), Some(unk), Some(cb_fn_8010A930));
}

pub fn ftKb_SpecialNYs_8010941C(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let ys_hat = samus_hat();
        if ys_hat.is_null() {
            return;
        }
        (*fp).cmd_vars[0] = 0;
        st::Fighter_ChangeMotionState(gobj, ftKb_MS_YsSpecialN1 as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        start_hat_anim(fp, ys_hat, 1);
        ext::HSD_JObjAnimAll((*fp).u.kb.hat.jobj);
        set_grab_cbs(gobj, cb_fn_80109680, cb_fn_801095DC);
        ftKb_SpecialN_set_cbs(gobj);
    }
}

pub fn ftKb_SpecialNYs_801094FC(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let ys_hat = samus_hat();
        if ys_hat.is_null() {
            return;
        }
        (*fp).cmd_vars[0] = 0;
        st::Fighter_ChangeMotionState(gobj, ftKb_MS_YsSpecialAirNCapture2 as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        start_hat_anim(fp, ys_hat, 3);
        ext::HSD_JObjAnimAll((*fp).u.kb.hat.jobj);
        set_grab_cbs(gobj, cb_fn_801097B8, cb_fn_80109714);
        ftKb_SpecialN_set_cbs(gobj);
    }
}

/// Shared body of the four capture-latch callbacks: swap to `msid` with the inherited anim frame.
unsafe fn latch_to(gobj: *mut HSD_GObj, msid: i32, flags: u32, with_item: bool) {
    let fp = fpp(gobj);
    if with_item {
        ext::it_802F2D98((*fp).target_item_gobj, gobj);
    }
    st::Fighter_ChangeMotionState(gobj, msid, flags, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
    (*fp).set_x2222_b2(1);
    st::ftCommon_8007E2F4(&mut *fp, 0x1FF);
    st::ftCommon_8007E2FC(gobj);
    ftKb_SpecialN_set_cbs(gobj);
}

pub fn fn_801095DC(gobj: *mut HSD_GObj) {
    unsafe { latch_to(gobj, ftKb_MS_YsSpecialNCapture1_1 as i32, 0x81092, true) }
}

pub fn fn_80109680(gobj: *mut HSD_GObj) {
    unsafe { latch_to(gobj, ftKb_MS_YsSpecialNCapture1_0 as i32, 0x81092, false) }
}

pub fn fn_80109714(gobj: *mut HSD_GObj) {
    unsafe { latch_to(gobj, ftKb_MS_YsSpecialAirNCapture1_1 as i32, 0x81092, true) }
}

pub fn fn_801097B8(gobj: *mut HSD_GObj) {
    unsafe { latch_to(gobj, ftKb_MS_YsSpecialAirNCapture1_0 as i32, 0x81092, false) }
}

pub fn fn_8010984C(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, ftKb_MS_YsSpecialAirNCapture2 as i32, ftYs_MF_SpecialN_CollHit);
        set_grab_cbs(gobj, cb_fn_801097B8, cb_fn_80109714);
        ftKb_SpecialN_set_cbs(gobj);
    }
}

pub fn fn_801098EC(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, ftKb_MS_YsSpecialN1 as i32, ftYs_MF_SpecialN_CollHit);
        set_grab_cbs(gobj, cb_fn_80109680, cb_fn_801095DC);
        ftKb_SpecialN_set_cbs(gobj);
    }
}

pub fn fn_8010998C(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, ftKb_MS_YsSpecialAirNCapture1_1 as i32, ftYs_MF_SpecialN_Coll);
        ftKb_SpecialN_set_cbs(gobj);
    }
}

pub fn fn_80109A84(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_8007D7FC(&mut *fp);
        st::Fighter_ChangeMotionState(gobj, ftKb_MS_YsSpecialNCapture1_1 as i32, 0x0C4C5092, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
        ftKb_SpecialN_set_cbs(gobj);
    }
}

pub fn fn_80109B7C(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_8007D5D4(&mut *fp);
        st::Fighter_ChangeMotionState(gobj, ftKb_MS_YsSpecialAirN2_1 as i32, 0x0C4C5092, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
        ftKb_SpecialN_set_cbs(gobj);
    }
}

pub fn fn_80109BF8(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, ftKb_MS_YsSpecialAirN2_0 as i32, ftYs_MF_SpecialN_Coll);
        ftKb_SpecialN_set_cbs(gobj);
    }
}

pub fn fn_80109CF0(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, ftKb_MS_YsSpecialNCapture2_0 as i32, ftYs_MF_SpecialN_Coll);
        ftKb_SpecialN_set_cbs(gobj);
    }
}

/// `fn_8010984C`, `fn_801098EC`, `fn_8010998C`, `fn_80109A84`, `fn_80109B7C`, `fn_80109BF8`,
/// `fn_80109CF0` as Fighter callback entries (the Coll ports hand them to `ft_8008403C` /
/// `ft_80082C74`).
pub unsafe extern "C" fn cb_fn_8010984C(g: *mut HSD_GObj) {
    fn_8010984C(g);
}
pub unsafe extern "C" fn cb_fn_801098EC(g: *mut HSD_GObj) {
    fn_801098EC(g);
}
pub unsafe extern "C" fn cb_fn_8010998C(g: *mut HSD_GObj) {
    fn_8010998C(g);
}
pub unsafe extern "C" fn cb_fn_80109A84(g: *mut HSD_GObj) {
    fn_80109A84(g);
}
pub unsafe extern "C" fn cb_fn_80109B7C(g: *mut HSD_GObj) {
    fn_80109B7C(g);
}
pub unsafe extern "C" fn cb_fn_80109BF8(g: *mut HSD_GObj) {
    fn_80109BF8(g);
}
pub unsafe extern "C" fn cb_fn_80109CF0(g: *mut HSD_GObj) {
    fn_80109CF0(g);
}

pub fn ftKb_YsSpecialN1_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ext::HSD_JObjAnimAll((*fp).u.kb.hat.jobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftKb_YsSpecialAirNCapture2_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ext::HSD_JObjAnimAll((*fp).u.kb.hat.jobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCo_Fall_Enter(gobj);
        }
    }
}

/// Once the script sets `cmd_vars[0]` and the `latch` object exists, swap to the capture-2 state
/// `msid` and its hat animation `dyn_idx` (the four `*Capture*_Anim` callbacks). Returns whether
/// the swap ran.
unsafe fn capture_swap(gobj: *mut HSD_GObj, latched: bool, msid: i32, dyn_idx: usize) {
    let fp = fpp(gobj);
    if (*fp).cmd_vars[0] != 0 && latched {
        (*fp).cmd_vars[0] = 0;
        (*fp).cmd_vars[1] = 0;
        let hat = ext::ft_80459B88_hat(0xD);
        if hat.is_null() {
            return;
        }
        st::Fighter_ChangeMotionState(gobj, msid, 0x80012, 0.0, 1.0, 0.0, core::ptr::null_mut());
        start_hat_anim(fp, hat, dyn_idx);
        ftKb_SpecialN_set_cbs(gobj);
        st::ftCommon_8007E2F4(&mut *fp, 0x1FF);
    }
}

pub fn ftKb_YsSpecialAirNCapture1_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let latched = !(*fp).target_item_gobj.is_null();
        capture_swap(gobj, latched, ftKb_MS_YsSpecialNCapture2_1 as i32, 2);
    }
}

pub fn ftKb_YsSpecialNCapture1_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ext::HSD_JObjAnimAll((*fp).u.kb.hat.jobj);
        let latched = !(*fp).victim_gobj.is_null();
        capture_swap(gobj, latched, ftKb_MS_YsSpecialNCapture2_0 as i32, 2);
    }
}

pub fn ftKb_YsSpecialAirCapture2_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let latched = !(*fp).target_item_gobj.is_null();
        capture_swap(gobj, latched, ftKb_MS_YsSpecialAirN2_1 as i32, 4);
    }
}

pub fn ftKb_YsSpecialAirCapture1_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ext::HSD_JObjAnimAll((*fp).u.kb.hat.jobj);
        let latched = !(*fp).victim_gobj.is_null();
        capture_swap(gobj, latched, ftKb_MS_YsSpecialAirN2_0 as i32, 4);
    }
}

/// Release the swallowed item (`it_802F2E7C(.., false)`) once the script sets `cmd_vars[1]`, then
/// lay the egg on `cmd_vars[0]` (the two `*N2_1`/`*Capture2_0` Anims).
unsafe fn lay_egg_anim(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    ext::HSD_JObjAnimAll((*fp).u.kb.hat.jobj);
    if (*fp).cmd_vars[1] != 0 && !(*fp).target_item_gobj.is_null() {
        ext::it_802F2E7C((*fp).target_item_gobj, 0);
        (*fp).x1A64 = core::ptr::null_mut();
        (*fp).target_item_gobj = core::ptr::null_mut();
        (*fp).cmd_vars[1] = 0;
    }
    if (*fp).cmd_vars[0] != 0 {
        let mut item_attrs = itYoshiEggLay_DatAttrs::zeroed();
        st::ftCommon_8007E2F4(&mut *fp, 0);
        {
            let fp2 = fpp(gobj);
            let bone = melee_fighter::ftdmgfall::ftparts_get_bone_index(&*fp2, FtPart_TransN2 as i32);
            ext::lb_8000B1CC((*(*fp2).parts.add(bone as usize)).joint, core::ptr::null(), &mut item_attrs.pos);
        }
        {
            let fp2 = fpp(gobj);
            let da = da(fp2);
            item_attrs.vel.x = -(*fp2).facing_dir * (*da).specialn_ys_initial_horizontal_momentum;
            item_attrs.vel.y = (*da).specialn_ys_initial_vertical_momentum;
            item_attrs.vel.z = 0.0;
        }
        item_attrs.float2 = (*fpp(gobj)).facing_dir;
        item_attrs.lifetime = (*da(fpp(gobj))).specialn_ys_base_duration;
        item_attrs.x24 = (*da(fpp(gobj))).specialn_ys_damage_multiplier;
        {
            let ea = ea();
            item_attrs.float3 = (*ea).specialn_ys_unk3 / (*ea).specialn_ys_damage_multiplier;
        }
        item_attrs.kind = It_Kind_Kirby_YoshiEggLay as _;
        ext::it_802F2F34(gobj, &mut item_attrs);
        (*fp).cmd_vars[0] = 0;
    }
}

pub fn ftKb_YsSpecialNCapture2_0_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        lay_egg_anim(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftKb_YsSpecialAirN2_1_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        lay_egg_anim(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCo_Fall_Enter(gobj);
        }
    }
}

/// The swallow path of the `*Capture2_1` / `*N2_0` Anims: drop the held victim, then spit the egg.
unsafe fn spit_anim(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    ext::HSD_JObjAnimAll((*fp).u.kb.hat.jobj);
    if (*fp).cmd_vars[1] != 0 && !(*fp).victim_gobj.is_null() {
        ext::ftKb_SpecialNYs_8010AA2C((*fp).victim_gobj);
        (*fp).cmd_vars[1] = 0;
    }
    if (*fp).cmd_vars[0] != 0 {
        let victim = (*fp).victim_gobj;
        if !victim.is_null() {
            st::ftCommon_8007E2F4(&mut *fp, 0);
            ext::ftCo_800DE2CC(gobj, victim);
            crate::tu::ftkirbyyoshiegg::ftKb_SpecialNYs_8010AC78(victim, gobj);
            (*fp).cmd_vars[0] = 0;
        }
    }
}

pub fn ftKb_YsSpecialNCapture2_1_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        spit_anim(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftKb_YsSpecialAirN2_0_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        spit_anim(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCo_Fall_Enter(gobj);
        }
    }
}

pub fn ftKb_SpecialNYs_8010A8BC(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).target_item_gobj.is_null() {
            let mut has_destroy_type = false;
            let m = (*fp).motion_id;
            if m == ftKb_MS_YsSpecialNCapture1_1 as _
                || m == ftKb_MS_YsSpecialNCapture2_1 as _
                || m == ftKb_MS_YsSpecialAirNCapture1_1 as _
                || m == ftKb_MS_YsSpecialAirN2_1 as _
            {
                has_destroy_type = true;
            }
            ext::it_802F2E7C((*fp).target_item_gobj, has_destroy_type as i32);
            (*fp).x1A64 = core::ptr::null_mut();
            (*fp).target_item_gobj = core::ptr::null_mut();
        }
    }
}
