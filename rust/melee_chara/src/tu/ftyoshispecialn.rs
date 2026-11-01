//! src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c: Yoshi's Egg Lay (neutral B) and the tongue grab
//! state machine. No fused sites (tools/rust-port/fma_sites.py). State-oracle checked against the C
//! (tests/oracle_ftyoshispecialn.rs, gen/oracle); the earlier rounds' ports are re-exported and
//! checked through the legacy `crate::stubs` hooks. The attribute getters of round 1 take the
//! `&Fighter` instead of the gobj (documented in docs/rust-port/group-chara.md).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::chars::da;
#[allow(unused_imports)]
use melee_types::{ftYoshiAttributes, ftYs_DatAttrs, Fighter, Vec3};
#[allow(unused_imports)]
use crate::stubs as st;
#[allow(unused_imports)]
use melee_types::{HSD_GObj};
use crate::tu::ftinl;
use melee_types::*;

/// Already ported (rounds 1/2/3/5): re-exported under this TU.





/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use crate::tu::hookmac::{hook, hook_or};
    use melee_types::*;
    pub use crate::tu::ftinl::ext::{
        ftAnim_8006EBA4, ftAnim_IsFramesRemaining, ftCo_Fall_Enter, ftCommon_8007D5D4, ftCommon_8007D7FC,
        ft_80084EEC, ft_80084F3C, ft_8008A2BC, Fighter_ChangeMotionState,
    };
    pub use crate::tu::ftinl::ext::{ft_80082C74, ft_8008403C};
    hook_or!(FTPARTS_GETBONEINDEX, ftParts_GetBoneIndex, (fp: *mut Fighter, part: i32) -> i32,
        |fp: *mut Fighter, part: i32| unsafe { melee_fighter::hooks::ftparts_get_bone_index(&*fp, part) });
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *mut Vec3, *mut Vec3)> = None;
    #[allow(non_snake_case)]
    pub fn lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *mut Vec3, pos1: *mut Vec3) {
        match unsafe { LB_8000B1CC } { Some(f) => f(jobj, pos0, pos1), None => { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, pos0 as _, pos1 as _); } return; } }
    }
    hook_or!(FTCOMMON_8007E2D0, ftCommon_8007E2D0,
        (fp: *mut Fighter, arg1: i16, grab_cb: HSD_GObjEvent, unk_cb: HSD_GObjEvent, grabbed_cb: HSD_GObjInteraction),
        |fp: *mut Fighter, a: i16, g: HSD_GObjEvent, u: HSD_GObjEvent, gd: HSD_GObjInteraction| unsafe {
            melee_fighter::ftcommon2::ftcommon_8007e2d0(&mut *fp, a, g, u, gd)
        });
    hook_or!(FTCOMMON_8007E2F4, ftCommon_8007E2F4, (fp: *mut Fighter, val: i16),
        |fp: *mut Fighter, v: i16| unsafe { melee_fighter::ftcommon2::ftcommon_8007e2f4(&mut *fp, v) });
    hook_or!(FTCOMMON_8007E2FC, ftCommon_8007E2FC, (gobj: *mut HSD_GObj),
        |g: *mut HSD_GObj| unsafe { melee_fighter::ftcommon2::ftcommon_8007e2fc(&mut *((*g).user_data as *mut Fighter)) });
    pub static mut IT_802F2BFC: Option<fn(*mut Item_GObj, *mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn it_802F2BFC(item: *mut Item_GObj, owner: *mut HSD_GObj) {
        match unsafe { IT_802F2BFC } { Some(f) => f(item, owner), None => { unsafe { melee_items::tu::ityoshitongue::it_802F2BFC(item as _, owner as _); } return; } }
    }
    pub static mut IT_802F2CE0: Option<fn(*mut Item_GObj, i32)> = None;
    #[allow(non_snake_case)]
    pub fn it_802F2CE0(item: *mut Item_GObj, clear_destroy_type: i32) {
        match unsafe { IT_802F2CE0 } {
            Some(f) => f(item, clear_destroy_type),
            None => { unsafe { melee_items::tu::ityoshitongue::it_802F2CE0(item as _, (clear_destroy_type != 0)); } },
        }
    }
    pub static mut IT_802F2F34: Option<fn(*mut HSD_GObj, *mut itYoshiEggLay_DatAttrs)> = None;
    #[allow(non_snake_case)]
    pub fn it_802F2F34(gobj: *mut HSD_GObj, attrs: *mut itYoshiEggLay_DatAttrs) {
        match unsafe { IT_802F2F34 } { Some(f) => f(gobj, attrs), None => { unsafe { melee_items::tu::ityoshiegglay::it_802F2F34(gobj as _, attrs as _); } return; } }
    }
    pub static mut FTCO_800BBB8C: Option<fn(*mut HSD_GObj, *mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn ftCo_800BBB8C(gobj: *mut HSD_GObj, attacker_gobj: *mut HSD_GObj) {
        match unsafe { FTCO_800BBB8C } { Some(f) => f(gobj, attacker_gobj), None => { unsafe { melee_fighter::tu::ftCo_CaptureYoshi::ftCo_800BBB8C(gobj as _, attacker_gobj as _); } return; } }
    }
    pub static mut FTCO_800BBC88: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn ftCo_800BBC88(gobj: *mut HSD_GObj) {
        match unsafe { FTCO_800BBC88 } { Some(f) => f(gobj), None => { unsafe { melee_fighter::tu::ftCo_YoshiEgg::ftCo_800BBC88(gobj as _); } return; } }
    }
    pub static mut FTCO_800DE2CC: Option<fn(*mut HSD_GObj, *mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn ftCo_800DE2CC(a: *mut HSD_GObj, b: *mut HSD_GObj) {
        match unsafe { FTCO_800DE2CC } { Some(f) => f(a, b), None => { unsafe { melee_fighter::tu::ftCo_Throw::ftCo_800DE2CC(a as _, b as _); } return; } }
    }
    pub static mut FTCO_800BBED4: Option<fn(*mut HSD_GObj, *mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn ftCo_800BBED4(arg0: *mut HSD_GObj, arg1: *mut HSD_GObj) {
        match unsafe { FTCO_800BBED4 } { Some(f) => f(arg0, arg1), None => { unsafe { melee_fighter::tu::ftCo_YoshiEgg::ftCo_800BBED4(arg0 as _, arg1 as _); } return; } }
    }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ftCo_800BBB8C` as the grabbed callback (a `void (*)(HSD_GObj*, HSD_GObj*)`).
pub unsafe extern "C" fn cb_ftCo_800BBB8C(a: *mut HSD_GObj, b: *mut HSD_GObj) {
    melee_core::ct!("cb_ftCo_800BBB8C");
    ext::ftCo_800BBB8C(a, b);
}
/// The statics `fn_8012CEE0` / `fn_8012CF7C` / `fn_8012D004` / `fn_8012D0A0` as callback entries.
pub unsafe extern "C" fn cb_fn_8012CEE0(g: *mut HSD_GObj) {
    melee_core::ct!("cb_fn_8012CEE0");
    fn_8012CEE0(g);
}
pub unsafe extern "C" fn cb_fn_8012CF7C(g: *mut HSD_GObj) {
    melee_core::ct!("cb_fn_8012CF7C");
    fn_8012CF7C(g);
}
pub unsafe extern "C" fn cb_fn_8012D004(g: *mut HSD_GObj) {
    melee_core::ct!("cb_fn_8012D004");
    fn_8012D004(g);
}
pub unsafe extern "C" fn cb_fn_8012D0A0(g: *mut HSD_GObj) {
    melee_core::ct!("cb_fn_8012D0A0");
    fn_8012D0A0(g);
}
/// `fn_8012D128` / `fn_8012D1AC` as callback entries: the shared `chars::cb` ones, which call them.
pub use crate::chars::cb::{ftyoshispecialn__fn_8012D128 as cb_fn_8012D128, ftyoshispecialn__fn_8012D1AC as cb_fn_8012D1AC};

/// `motion_flags0` (static const).
pub const motion_flags0: u32 = Ft_MF_KeepGfx | Ft_MF_SkipModel | Ft_MF_Unk19;
/// `motion_flags1` (static const).
pub const motion_flags1: u32 = motion_flags0 | Ft_MF_SkipMatAnim | Ft_MF_SkipColAnim;
/// `motion_flags2` (static const).
pub const motion_flags2: u32 =
    motion_flags1 | Ft_MF_KeepGfx | Ft_MF_SkipModel | Ft_MF_SkipMatAnim | Ft_MF_SkipColAnim | Ft_MF_Unk19;
/// `ftYs_MF_SpecialN_Coll` / `ftYs_MF_SpecialN_CollHit` (ftYoshi/forward.h): the ground/air collision
/// flags plus KeepGfx / SkipModel, and with SkipHit.
pub const ftYs_MF_SpecialN_Coll: u32 = 0xC4C5092;
pub const ftYs_MF_SpecialN_CollHit: u32 = ftYs_MF_SpecialN_Coll | Ft_MF_SkipHit;

pub fn ftYs_SpecialN_GetBoneIndex(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftYs_SpecialN_GetBoneIndex");
    unsafe { ext::ftParts_GetBoneIndex(fpp(gobj), FtPart_TransN2 as i32) }
}

/// `gFtDataList[Ft_Kind_Yoshi]->ext_attr`.
#[inline(always)]
unsafe fn ext_attr() -> &'static ftYoshiAttributes {
    &*((*melee_fighter::tu::ftdata::gFtDataList[Ft_Kind_Yoshi as usize]).ext_attr as *const ftYoshiAttributes)
}

pub fn ftYs_SpecialN_GetExtAttr28() -> f32 {
    melee_core::ct!("ftYs_SpecialN_GetExtAttr28");
    unsafe { ext_attr().x28 }
}

pub fn ftYs_SpecialN_GetExtAttr2C() -> f32 {
    melee_core::ct!("ftYs_SpecialN_GetExtAttr2C");
    unsafe { ext_attr().x2C }
}

pub fn ftYs_SpecialN_8012CD40() -> f32 {
    melee_core::ct!("ftYs_SpecialN_8012CD40");
    unsafe { ext_attr().x30 }
}

pub fn ftYs_SpecialN_GetExtAttr34() -> f32 {
    melee_core::ct!("ftYs_SpecialN_GetExtAttr34");
    unsafe { ext_attr().x34 }
}

pub fn ftYs_SpecialN_GetExtAttr38() -> i32 {
    melee_core::ct!("ftYs_SpecialN_GetExtAttr38");
    unsafe { ext_attr().x38 as i32 }
}

pub fn ftYs_SpecialN_8012CD88(arg0: *mut Vec3) {
    melee_core::ct!("ftYs_SpecialN_8012CD88");
    unsafe {
        let ea = ext_attr();
        (*arg0).x = ea.x3C.x;
        (*arg0).y = ea.x3C.y;
        (*arg0).z = 0.0;
    }
}

pub fn ftYs_SpecialN_8012CDB4() -> f32 {
    melee_core::ct!("ftYs_SpecialN_8012CDB4");
    unsafe {
        let ea = ext_attr();
        ea.x44 / ea.x18
    }
}

pub fn ftYs_SpecialN_8012CDD4(gobj: *mut HSD_GObj) -> *mut HSD_Joint {
    melee_core::ct!("ftYs_SpecialN_8012CDD4");
    unsafe {
        let fp = fpp(gobj);
        let ca = (*fp).ft_data;
        *(*ca).x48_items.add(3) as *mut HSD_Joint
    }
}

pub fn setupCallbacks(gobj: *mut HSD_GObj, grab_cb: HSD_GObjEvent, arg1: HSD_GObjEvent, grabbed_cb: HSD_GObjInteraction) {
    melee_core::ct!("setupCallbacks");
    unsafe {
        let fp = fpp(gobj);
        ext::ftCommon_8007E2D0(fp, 4, grab_cb, arg1, grabbed_cb);
    }
}

pub fn ftYs_SpecialN_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialN_Enter");
    unsafe {
        let fp = fpp(gobj);
        (*fp).cmd_vars[0] = 0;
        ext::Fighter_ChangeMotionState(gobj, ftYs_MS_SpecialN1 as i32, Ft_MF_None, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ext::ftAnim_8006EBA4(gobj);
        setupCallbacks(gobj, Some(cb_fn_8012CF7C), Some(cb_fn_8012CEE0), Some(cb_ftCo_800BBB8C));
    }
}

pub fn ftYs_SpecialAirN_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialAirN_Enter");
    unsafe {
        let fp = fpp(gobj);
        (*fp).cmd_vars[0] = 0;
        ext::Fighter_ChangeMotionState(gobj, ftYs_MS_SpecialAirN1_0 as i32, Ft_MF_None, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ext::ftAnim_8006EBA4(gobj);
        setupCallbacks(gobj, Some(cb_fn_8012D0A0), Some(cb_fn_8012D004), Some(cb_ftCo_800BBB8C));
    }
}

pub fn fn_8012CEE0(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8012CEE0");
    unsafe {
        let fp = fpp(gobj);
        ext::it_802F2BFC((*fp).target_item_gobj, gobj);
        ext::Fighter_ChangeMotionState(gobj, ftYs_MS_SpecialN1_1 as i32, motion_flags1, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
        (*fp).set_x2222_b2(1);
        (*fp).mv.ys.specialn.set_x0_b0(1);
        ftYs_SpecialS_8012DF00(gobj);
        ext::ftCommon_8007E2F4(fp, 0x1FF);
        ext::ftCommon_8007E2FC(gobj);
    }
}

pub fn fn_8012CF7C(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8012CF7C");
    unsafe {
        let fp = fpp(gobj);
        ext::Fighter_ChangeMotionState(gobj, ftYs_MS_SpecialN1_0 as i32, motion_flags1, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
        (*fp).set_x2222_b2(1);
        (*fp).mv.ys.specialn.set_x0_b0(0);
        ext::ftCommon_8007E2F4(fp, 0x1FF);
        ext::ftCommon_8007E2FC(gobj);
    }
}

pub fn fn_8012D004(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8012D004");
    unsafe {
        let fp = fpp(gobj);
        ext::it_802F2BFC((*fp).target_item_gobj, gobj);
        ext::Fighter_ChangeMotionState(gobj, ftYs_MS_SpecialAirN1_2 as i32, motion_flags2, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
        (*fp).set_x2222_b2(1);
        (*fp).mv.ys.specialn.set_x0_b0(1);
        ftYs_SpecialS_8012DF00(gobj);
        ext::ftCommon_8007E2F4(fp, 0x1FF);
        ext::ftCommon_8007E2FC(gobj);
    }
}

pub fn fn_8012D0A0(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8012D0A0");
    unsafe {
        let fp = fpp(gobj);
        ext::Fighter_ChangeMotionState(gobj, ftYs_MS_SpecialAirN1_1 as i32, motion_flags2, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
        (*fp).set_x2222_b2(1);
        (*fp).mv.ys.specialn.set_x0_b0(0);
        ext::ftCommon_8007E2F4(fp, 0x1FF);
        ext::ftCommon_8007E2FC(gobj);
    }
}

pub fn fn_8012D128(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8012D128");
    unsafe {
        let fp = fpp(gobj);
        ftinl::ftCommon_GroundToAirStateChange(gobj, fp, ftYs_MS_SpecialAirN1_0 as i32, ftYs_MF_SpecialN_CollHit);
        setupCallbacks(gobj, Some(cb_fn_8012D0A0), Some(cb_fn_8012D004), Some(cb_ftCo_800BBB8C));
    }
}

pub fn fn_8012D1AC(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8012D1AC");
    unsafe {
        let fp = fpp(gobj);
        ftinl::ftCommon_AirToGroundStateChange(gobj, fp, ftYs_MS_SpecialN1 as i32, ftYs_MF_SpecialN_CollHit);
        setupCallbacks(gobj, Some(cb_fn_8012CF7C), Some(cb_fn_8012CEE0), Some(cb_ftCo_800BBB8C));
    }
}

/// `checkAnimEnd(gobj, cb)` (static).
pub fn checkAnimEnd(gobj: *mut HSD_GObj, cb: fn(*mut HSD_GObj)) {
    melee_core::ct!("checkAnimEnd");
    if !ext::ftAnim_IsFramesRemaining(gobj) {
        cb(gobj);
    }
}

pub fn ftYs_SpecialN1_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialN1_Anim");
    checkAnimEnd(gobj, ext::ft_8008A2BC);
}

pub fn ftYs_SpecialAirN1_0_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialAirN1_0_Anim");
    checkAnimEnd(gobj, ext::ftCo_Fall_Enter);
}

/// `getFtTargetItem` / `getFtVictim` (ft/inlines.h).
pub fn getFtTargetItem(fp: *mut Fighter) -> *mut HSD_GObj {
    melee_core::ct!("getFtTargetItem");
    unsafe { (*fp).target_item_gobj }
}
pub fn getFtVictim(fp: *mut Fighter) -> *mut HSD_GObj {
    melee_core::ct!("getFtVictim");
    unsafe { (*fp).victim_gobj }
}

pub fn changeMotionState(gobj: *mut HSD_GObj, msid: i32, target_getter: fn(*mut Fighter) -> *mut HSD_GObj, arg3: bool) {
    melee_core::ct!("changeMotionState");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).cmd_vars[0] != 0 && !target_getter(fp).is_null() {
            (*fp).cmd_vars[0] = 0;
            (*fp).cmd_vars[1] = 0;
            ext::Fighter_ChangeMotionState(gobj, msid, motion_flags0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            if arg3 {
                ftYs_SpecialS_8012DF00(gobj);
            }
            ext::ftCommon_8007E2F4(fp, 0x1FF);
        }
    }
}

pub fn ftYs_SpecialN1_1_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialN1_1_Anim");
    changeMotionState(gobj, ftYs_MS_SpecialN2_1 as i32, getFtTargetItem, true);
}

pub fn ftYs_SpecialN1_0_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialN1_0_Anim");
    changeMotionState(gobj, ftYs_MS_SpecialN2_0 as i32, getFtVictim, false);
}

pub fn ftYs_SpecialAirN1_2_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialAirN1_2_Anim");
    changeMotionState(gobj, ftYs_MS_SpecialAirN2_1 as i32, getFtTargetItem, true);
}

pub fn ftYs_SpecialAirN1_1_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialAirN1_1_Anim");
    changeMotionState(gobj, ftYs_MS_SpecialAirN2_0 as i32, getFtVictim, false);
}

/// `inlineB0(gobj, on_anim_end)` (static inline): lays the egg.
pub fn inlineB0(gobj: *mut HSD_GObj, on_anim_end: fn(*mut HSD_GObj)) {
    melee_core::ct!("inlineB0");
    unsafe {
        let fp = fpp(gobj);
        let mut item_attrs: itYoshiEggLay_DatAttrs = core::mem::zeroed();
        if (*fp).cmd_vars[1] != 0 && !(*fp).target_item_gobj.is_null() {
            ext::it_802F2CE0((*fp).target_item_gobj, 0);
            (*fp).x1A64 = core::ptr::null_mut();
            (*fp).target_item_gobj = core::ptr::null_mut();
            (*fp).cmd_vars[1] = 0;
        }
        if (*fp).cmd_vars[0] != 0 && (*fp).mv.ys.specialn.x0_b0() != 0 {
            ext::ftCommon_8007E2F4(fp, 0);
            {
                let fp = fpp(gobj);
                ext::lb_8000B1CC(
                    (*(*fp).parts.add(ftYs_SpecialN_GetBoneIndex(gobj) as usize)).joint,
                    core::ptr::null_mut(),
                    &mut item_attrs.pos,
                );
            }
            ftYs_SpecialN_SetupItemVel(gobj, &mut item_attrs.vel);
            item_attrs.float2 = ftYs_SpecialN_GetFacingDir(gobj);
            item_attrs.lifetime = ftYs_SpecialN_GetDatAttr24(gobj);
            item_attrs.x24 = ftYs_SpecialN_GetDatAttr18(gobj);
            item_attrs.float3 = ftYs_SpecialN_8012CDB4();
            item_attrs.kind = It_Kind_Yoshi_EggLay;
            ext::it_802F2F34(gobj, &mut item_attrs);
            (*fp).mv.ys.specialn.set_x0_b0(0);
            (*fp).cmd_vars[0] = 0;
        }
        if !ext::ftAnim_IsFramesRemaining(gobj) {
            on_anim_end(gobj);
        }
    }
}

pub fn ftYs_SpecialN2_1_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialN2_1_Anim");
    inlineB0(gobj, ext::ft_8008A2BC);
}

/// `inlineA0(fp)` (static): release the grabbed victim.
pub fn inlineA0(fp: *mut Fighter) {
    melee_core::ct!("inlineA0");
    unsafe {
        let victim_gobj = getFtVictim(fp);
        if !victim_gobj.is_null() {
            ext::ftCo_800BBC88(victim_gobj);
            (*fp).cmd_vars[1] = 0;
        }
    }
}

/// `inlineA1(gobj, anim_end_cb)` (static): the tongue's victim handling.
pub fn inlineA1(gobj: *mut HSD_GObj, anim_end_cb: fn(*mut HSD_GObj)) {
    melee_core::ct!("inlineA1");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).cmd_vars[1] != 0 {
            inlineA0(fp);
        }
        if (*fp).cmd_vars[0] != 0 {
            let victim_gobj = (*fp).victim_gobj;
            if !victim_gobj.is_null() {
                ext::ftCommon_8007E2F4(fp, 0);
                ext::ftCo_800DE2CC(gobj, victim_gobj);
                ext::ftCo_800BBED4(victim_gobj, gobj);
                (*fp).cmd_vars[0] = 0;
            }
        }
        if !ext::ftAnim_IsFramesRemaining(gobj) {
            anim_end_cb(gobj);
        }
    }
}

pub fn ftYs_SpecialN2_0_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialN2_0_Anim");
    inlineA1(gobj, ext::ft_8008A2BC);
}

pub fn ftYs_SpecialAirN2_1_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialAirN2_1_Anim");
    inlineB0(gobj, ext::ftCo_Fall_Enter);
}

pub fn ftYs_SpecialAirN2_0_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialAirN2_0_Anim");
    inlineA1(gobj, ext::ftCo_Fall_Enter);
}

// ---- folded from chars (consolidation step 20/21) ----
pub fn ftYs_SpecialN_GetDatAttr18(gobj: *mut HSD_GObj) -> f32 {
    melee_core::ct!("ftYs_SpecialN_GetDatAttr18");
    let fp = unsafe { &*((*gobj).user_data as *const Fighter) };
    da::<ftYoshiAttributes>(fp).x18
}

pub fn ftYs_SpecialN_GetDatAttr1C(gobj: *mut HSD_GObj) -> f32 {
    melee_core::ct!("ftYs_SpecialN_GetDatAttr1C");
    let fp = unsafe { &*((*gobj).user_data as *const Fighter) };
    da::<ftYoshiAttributes>(fp).x1C
}

pub fn ftYs_SpecialN_GetDatAttr20(gobj: *mut HSD_GObj) -> f32 {
    melee_core::ct!("ftYs_SpecialN_GetDatAttr20");
    let fp = unsafe { &*((*gobj).user_data as *const Fighter) };
    da::<ftYoshiAttributes>(fp).x20
}

pub fn ftYs_SpecialN_GetDatAttr24(gobj: *mut HSD_GObj) -> f32 {
    melee_core::ct!("ftYs_SpecialN_GetDatAttr24");
    let fp = unsafe { &*((*gobj).user_data as *const Fighter) };
    da::<ftYs_DatAttrs>(fp).x24
}

pub fn ftYs_SpecialN_GetFacingDir(gobj: *mut HSD_GObj) -> f32 {
    melee_core::ct!("ftYs_SpecialN_GetFacingDir");
    let fp = unsafe { &*((*gobj).user_data as *const Fighter) };
    fp.facing_dir
}

pub fn ftYs_SpecialN_SetupItemVel(gobj: *mut HSD_GObj, out: &mut Vec3) {
    melee_core::ct!("ftYs_SpecialN_SetupItemVel");
    let fp = unsafe { &*((*gobj).user_data as *const Fighter) };
    let a: &ftYoshiAttributes = da(fp);
    out.x = -fp.facing_dir * a.x10;
    out.y = a.x14;
    out.z = 0.0;
}

/// `fn_8012D298` (src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c).
pub fn fn_8012D298(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8012D298");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_GroundToAirStateChange(gobj, fp, 352 /* ftYs_MS_SpecialAirN1_1 */, 0xC4C5092u32 /* ftYs_MF_SpecialN_Coll */);
}

/// `fn_8012D360` (src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c).
pub fn fn_8012D360(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8012D360");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_AirToGroundStateChange(gobj, fp, 347 /* ftYs_MS_SpecialN1_0 */, 0xC4C5092u32 /* ftYs_MF_SpecialN_Coll */);
}

/// `fn_8012D428` (src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c).
pub fn fn_8012D428(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8012D428");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_GroundToAirStateChange(gobj, fp, 354 /* ftYs_MS_SpecialAirN2_0 */, 0xC4C5092u32 /* ftYs_MF_SpecialN_Coll */);
}

/// `fn_8012D4F0` (src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c).
pub fn fn_8012D4F0(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8012D4F0");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_AirToGroundStateChange(gobj, fp, 349 /* ftYs_MS_SpecialN2_0 */, 0xC4C5092u32 /* ftYs_MF_SpecialN_Coll */);
}

/// `ftYs_SpecialN1_Phys` (src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c).
pub fn ftYs_SpecialN1_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialN1_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftYs_SpecialAirN1_0_Phys` (src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c).
pub fn ftYs_SpecialAirN1_0_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialAirN1_0_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftYs_SpecialN1_1_Phys` (src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c).
pub fn ftYs_SpecialN1_1_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialN1_1_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftYs_SpecialN1_0_Phys` (src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c).
pub fn ftYs_SpecialN1_0_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialN1_0_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftYs_SpecialAirN1_2_Phys` (src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c).
pub fn ftYs_SpecialAirN1_2_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialAirN1_2_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftYs_SpecialAirN1_1_Phys` (src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c).
pub fn ftYs_SpecialAirN1_1_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialAirN1_1_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftYs_SpecialN2_1_Phys` (src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c).
pub fn ftYs_SpecialN2_1_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialN2_1_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftYs_SpecialN2_0_Phys` (src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c).
pub fn ftYs_SpecialN2_0_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialN2_0_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftYs_SpecialAirN2_1_Phys` (src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c).
pub fn ftYs_SpecialAirN2_1_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialAirN2_1_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftYs_SpecialAirN2_0_Phys` (src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c).
pub fn ftYs_SpecialAirN2_0_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialAirN2_0_Phys");
    st::ft_80084EEC(gobj);
}

/// `fn_8012D230` (src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c).
pub fn fn_8012D230(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8012D230");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 353 /* ftYs_MS_SpecialAirN1_2 */, 0xC4C5092u32 /* ftYs_MF_SpecialN_Coll */);
        crate::chars::r3::ftYoshi::ftyoshispecialn::ftYs_SpecialS_8012DF00(gobj);
    }
}

/// `fn_8012D2F8` (src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c).
pub fn fn_8012D2F8(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8012D2F8");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 348 /* ftYs_MS_SpecialN1_1 */, 0xC4C5092u32 /* ftYs_MF_SpecialN_Coll */);
        crate::chars::r3::ftYoshi::ftyoshispecialn::ftYs_SpecialS_8012DF00(gobj);
    }
}

/// `fn_8012D3C0` (src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c).
pub fn fn_8012D3C0(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8012D3C0");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 355 /* ftYs_MS_SpecialAirN2_1 */, 0xC4C5092u32 /* ftYs_MF_SpecialN_Coll */);
        crate::chars::r3::ftYoshi::ftyoshispecialn::ftYs_SpecialS_8012DF00(gobj);
    }
}

/// `fn_8012D488` (src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c).
pub fn fn_8012D488(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8012D488");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 350 /* ftYs_MS_SpecialN2_1 */, 0xC4C5092u32 /* ftYs_MF_SpecialN_Coll */);
        crate::chars::r3::ftYoshi::ftyoshispecialn::ftYs_SpecialS_8012DF00(gobj);
    }
}

/// `ftYs_SpecialS_8012DF00` (src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c).
pub fn ftYs_SpecialS_8012DF00(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialS_8012DF00");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).take_dmg_cb = Some(crate::chars::cb::ftyoshi__ftYs_Init_8012BA8C);
        (*fp).death2_cb = Some(crate::chars::cb::ftyoshi__ftYs_Init_8012BA8C);
    }
}

/// `ftYs_SpecialN1_Coll` (src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c).
pub fn ftYs_SpecialN1_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialN1_Coll");
    unsafe {
        st::ft_8008403C(gobj, Some(crate::chars::cb::ftyoshispecialn__fn_8012D128));
    }
}

/// `ftYs_SpecialAirN1_0_Coll` (src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c).
pub fn ftYs_SpecialAirN1_0_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialAirN1_0_Coll");
    unsafe {
        st::ft_80082C74(gobj, Some(crate::chars::cb::ftyoshispecialn__fn_8012D1AC));
    }
}

/// `ftYs_SpecialN1_1_Coll` (src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c).
pub fn ftYs_SpecialN1_1_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialN1_1_Coll");
    unsafe {
        st::ft_8008403C(gobj, Some(crate::chars::cb::ftyoshispecialn__fn_8012D230));
    }
}

/// `ftYs_SpecialN1_0_Coll` (src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c).
pub fn ftYs_SpecialN1_0_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialN1_0_Coll");
    unsafe {
        st::ft_8008403C(gobj, Some(crate::chars::cb::ftyoshispecialn__fn_8012D298));
    }
}

/// `ftYs_SpecialAirN1_2_Coll` (src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c).
pub fn ftYs_SpecialAirN1_2_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialAirN1_2_Coll");
    unsafe {
        st::ft_80082C74(gobj, Some(crate::chars::cb::ftyoshispecialn__fn_8012D2F8));
    }
}

/// `ftYs_SpecialAirN1_1_Coll` (src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c).
pub fn ftYs_SpecialAirN1_1_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialAirN1_1_Coll");
    unsafe {
        st::ft_80082C74(gobj, Some(crate::chars::cb::ftyoshispecialn__fn_8012D360));
    }
}

/// `ftYs_SpecialN2_1_Coll` (src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c).
pub fn ftYs_SpecialN2_1_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialN2_1_Coll");
    unsafe {
        st::ft_8008403C(gobj, Some(crate::chars::cb::ftyoshispecialn__fn_8012D3C0));
    }
}

/// `ftYs_SpecialN2_0_Coll` (src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c).
pub fn ftYs_SpecialN2_0_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialN2_0_Coll");
    unsafe {
        st::ft_8008403C(gobj, Some(crate::chars::cb::ftyoshispecialn__fn_8012D428));
    }
}

/// `ftYs_SpecialAirN2_1_Coll` (src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c).
pub fn ftYs_SpecialAirN2_1_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialAirN2_1_Coll");
    unsafe {
        st::ft_80082C74(gobj, Some(crate::chars::cb::ftyoshispecialn__fn_8012D488));
    }
}

/// `ftYs_SpecialAirN2_0_Coll` (src/melee/ft/kinds/ftYoshi/ftyoshispecialn.c).
pub fn ftYs_SpecialAirN2_0_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialAirN2_0_Coll");
    unsafe {
        st::ft_80082C74(gobj, Some(crate::chars::cb::ftyoshispecialn__fn_8012D4F0));
    }
}
