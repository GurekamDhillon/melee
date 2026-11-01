//! src/melee/gr/grizumi.c: Fountain of Dreams (Gr_Kind_Izumi): the two rising / falling platforms
//! (gobj kind 4), the water mirror (a reflection camera gobj whose render callback copies the
//! scene's mirrored view into an image and a texture matrix), the star-field joint and the
//! stage-slot (TARGET_PC) hooks. State is the `yakumono_param` static, the per-gobj `Ground`
//! (`u.izumi` / `u.izumi2` / `u.izumi3`), `stage_info`, the reflection descriptors and the callback /
//! stage-data tables; every other callee is a TODO hook in [`ext`].
//!
//! TARGET_PC code (this is the PC port): `grIzumi_StageSlotParamSwap`, the `ScriptGame_StageSlot*`
//! checks (no stage-slot context in the native build yet: the hooks default to 0 / no-op) and
//! `grIzumi_StageSlotReflection`.
//! Float code: `grIzumi_801CC358` has two fused sites (`ff * fff + x18`, `0.5f * f + 0.5f`); int
//! parameters taking float arguments convert with `as i32` (saturating, like gwtool).
//! State-oracle checked against the C (tests/gro_grizumi.rs, gen/oracle/gr_oracle.py).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::missing_safety_doc)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::gr::trivial::*;
#[allow(unused_imports)]
use crate::gr::forward::*;
use super::grinl::{
    assert_, GET_GROUND, GET_JOBJ, HSD_JObjGetChild, HSD_JObjGetNext, HSD_JObjGetParent, HSD_JObjGetTranslation,
    HSD_JObjGetTranslationY, HSD_JObjSetScaleX, HSD_JObjSetScaleY, HSD_JObjSetTranslate, HSD_JObjSetTranslateY,
};
use super::grinl2::rand_range;
use core::ffi::c_void;
use core::ptr::{addr_of_mut, null_mut};
use melee_core::fp::fmadds;
use melee_types::*;


/// Callees in TUs that are not ported yet. An unset hook panics (the `ScriptGame_*` ones default).
pub mod ext {
    use crate::tu::hookmac::{hook, hook_or};
    use core::ffi::c_void;
    use melee_types::*;
    pub use crate::tu::grinl::ext::{stage_info, HSD_JObjSetMtxDirtySub};
    pub use crate::tu::grinl2::ext::{
        grAnime_801C8138, grDisplay_801C5DB0, GObj_SetupGXLink, Ground_InitMapColl, HSD_GObj_SetupProc, lb_800115F4,
    };
    hook_or!(CAMERA_8002A4AC, Camera_8002A4AC, (a0: *mut HSD_GObj), |a0: *mut HSD_GObj| { unsafe { crate::tu::camera::Camera_8002A4AC(a0 as _); } });
    hook_or!(CAMERA_80030A50, Camera_80030A50, () -> *mut HSD_GObj, || unsafe { crate::tu::camera::Camera_80030A50() as _ });
    hook_or!(CAMERA_80031074, Camera_80031074, (a0: u8), |a0: u8| unsafe { crate::tu::camera::Camera_80031074(a0) });
    hook_or!(CAMERA_800310A0, Camera_800310A0, (a0: u8), |a0: u8| unsafe { crate::tu::camera::Camera_800310A0(a0) });
    hook_or!(GOBJ_CREATE, GObj_Create, (classifier: u16, p_link: u8, priority: u8) -> *mut HSD_GObj, |classifier: u16, p_link: u8, priority: u8| unsafe { melee_runtime::tu::gobj_native::GObj_Create(classifier as _, p_link as _, priority as _) as _ });
    hook_or!(GOBJ_INITUSERDATA, GObj_InitUserData, (gobj: *mut HSD_GObj, kind: u8, remove_func: Option<unsafe extern "C" fn(*mut c_void)>, data: *mut c_void), |gobj: *mut HSD_GObj, kind: u8, remove_func: Option<unsafe extern "C" fn(*mut c_void)>, data: *mut c_void| { unsafe { melee_runtime::tu::gobj_native::GObj_InitUserData(gobj as _, kind as _, remove_func as _, data as _); } });
    hook_or!(GOBJ_SETUPGXLINKMAXSORTED, GObj_SetupGXLinkMaxSorted, (gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, priority: u32), |gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, priority: u32| { unsafe { melee_runtime::tu::gobj_native::GObj_SetupGXLinkMaxSorted(gobj as _, render_cb as _, priority as _); } });
    hook!(GXGETTEXBUFFERSIZE, GXGetTexBufferSize, (width: u16, height: u16, format: u32, mipmap: u8, max_lod: u8) -> u32);
    pub static mut GROUND_801C0498: Option<fn() -> f32> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C0498() -> f32 {
        match unsafe { GROUND_801C0498 } { Some(f) => f(), None => { return unsafe { crate::tu::ground::Ground_801C0498() as _ }; } }
    }
    pub static mut GROUND_801C1A20: Option<fn(*mut HSD_Joint, s32) -> *mut HSD_GObj> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C1A20(a0: *mut HSD_Joint, a1: s32) -> *mut HSD_GObj {
        match unsafe { GROUND_801C1A20 } { Some(f) => f(a0, a1), None => { return unsafe { crate::tu::ground::Ground_801C1A20(a0 as _, a1 as _) as _ }; } }
    }
    pub static mut GROUND_801C39C0: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C39C0() {
        match unsafe { GROUND_801C39C0 } { Some(f) => f(), None => { unsafe { crate::tu::ground::Ground_801C39C0(); } return; } }
    }
    pub static mut GROUND_801C3BB4: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C3BB4() {
        match unsafe { GROUND_801C3BB4 } { Some(f) => f(), None => { unsafe { crate::tu::ground::Ground_801C3BB4(); } return; } }
    }
    pub static mut GROUND_801C3FA4: Option<fn(*mut HSD_GObj, i32) -> *mut HSD_JObj> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C3FA4(gobj: *mut HSD_GObj, depth: i32) -> *mut HSD_JObj {
        match unsafe { GROUND_801C3FA4 } { Some(f) => f(gobj, depth), None => { return unsafe { crate::tu::ground::Ground_801C3FA4(gobj as _, depth as _) as _ }; } }
    }
    pub static mut GROUND_GETSTAGEGOBJ: Option<fn(i32) -> *mut HSD_GObj> = None;
    #[allow(non_snake_case)]
    pub fn Ground_GetStageGObj(a0: i32) -> *mut HSD_GObj {
        match unsafe { GROUND_GETSTAGEGOBJ } { Some(f) => f(a0), None => { return unsafe { crate::tu::ground::Ground_GetStageGObj(a0 as _) as _ }; } }
    }
    pub static mut GROUND_GETYAKUMONOPARAM: Option<fn() -> *mut c_void> = None;
    #[allow(non_snake_case)]
    pub fn Ground_GetYakumonoParam() -> *mut c_void {
        match unsafe { GROUND_GETYAKUMONOPARAM } { Some(f) => f(), None => { return unsafe { crate::tu::ground::Ground_GetYakumonoParam() as _ }; } }
    }
    pub static mut HSD_AOBJSETFLAGS: Option<fn(*mut HSD_AObj, u32)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_AObjSetFlags(aobj: *mut HSD_AObj, flags: u32) {
        match unsafe { HSD_AOBJSETFLAGS } { Some(f) => f(aobj, flags), None => { unsafe { melee_runtime::tu::aobj::HSD_AObjSetFlags(aobj as _, flags as _); } return; } }
    }
    pub static mut HSD_ARCHIVEGETPUBLICADDRESS: Option<fn(*mut HSD_Archive, *const i8) -> *mut c_void> = None;
    #[allow(non_snake_case)]
    pub fn HSD_ArchiveGetPublicAddress(a0: *mut HSD_Archive, a1: *const i8) -> *mut c_void {
        match unsafe { HSD_ARCHIVEGETPUBLICADDRESS } { Some(f) => f(a0, a1), None => { return unsafe { melee_runtime::tu::archive::HSD_ArchiveGetPublicAddress(a0 as _, a1 as _) as _ }; } }
    }
    pub static mut HSD_COBJENDCURRENT: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn HSD_CObjEndCurrent() {
        match unsafe { HSD_COBJENDCURRENT } { Some(f) => f(), None => { unsafe { melee_runtime::tu::cobj::HSD_CObjEndCurrent(); } return; } }
    }
    pub static mut HSD_COBJERASESCREEN: Option<fn(*mut HSD_CObj, s32, s32, s32)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_CObjEraseScreen(cobj: *mut HSD_CObj, enable_color: s32, enable_alpha: s32, enable_depth: s32) {
        match unsafe { HSD_COBJERASESCREEN } { Some(f) => f(cobj, enable_color, enable_alpha, enable_depth), None => { unsafe { melee_runtime::tu::cobj::HSD_CObjEraseScreen(cobj as _, enable_color as _, enable_alpha as _, enable_depth as _); } return; } }
    }
    hook_or!(HSD_COBJGETAUTHOREDASPECT, HSD_CObjGetAuthoredAspect, (cobj: *mut HSD_CObj) -> f32, |cobj: *mut HSD_CObj| unsafe { melee_runtime::tu::cobj_world::HSD_CObjGetAuthoredAspect(cobj as _) as _ });
    hook_or!(HSD_COBJGETEYEPOSITION, HSD_CObjGetEyePosition, (cobj: *mut HSD_CObj, cam_pos: *mut Vec3), |cobj: *mut HSD_CObj, cam_pos: *mut Vec3| { unsafe { melee_runtime::tu::cobj_world::HSD_CObjGetEyePosition(cobj as _, cam_pos as _); } });
    pub static mut HSD_COBJGETFAR: Option<fn(*mut HSD_CObj) -> f32> = None;
    #[allow(non_snake_case)]
    pub fn HSD_CObjGetFar(a0: *mut HSD_CObj) -> f32 {
        match unsafe { HSD_COBJGETFAR } { Some(f) => f(a0), None => { return unsafe { melee_runtime::tu::cobj::HSD_CObjGetFar(a0 as _) as _ }; } }
    }
    hook_or!(HSD_COBJGETFOV, HSD_CObjGetFov, (cobj: *mut HSD_CObj) -> f32, |cobj: *mut HSD_CObj| unsafe { melee_runtime::tu::cobj_world::HSD_CObjGetFov(cobj as _) as _ });
    hook_or!(HSD_COBJGETINTEREST, HSD_CObjGetInterest, (cobj: *mut HSD_CObj, interest: *mut Vec3), |cobj: *mut HSD_CObj, interest: *mut Vec3| { unsafe { melee_runtime::tu::cobj_world::HSD_CObjGetInterest(cobj as _, interest as _); } });
    pub static mut HSD_COBJGETNEAR: Option<fn(*mut HSD_CObj) -> f32> = None;
    #[allow(non_snake_case)]
    pub fn HSD_CObjGetNear(a0: *mut HSD_CObj) -> f32 {
        match unsafe { HSD_COBJGETNEAR } { Some(f) => f(a0), None => { return unsafe { melee_runtime::tu::cobj::HSD_CObjGetNear(a0 as _) as _ }; } }
    }
    hook!(HSD_COBJGETVIEWINGMTXPTR, HSD_CObjGetViewingMtxPtr, (cobj: *mut HSD_CObj) -> MtxPtr);
    hook_or!(HSD_COBJSETASPECT, HSD_CObjSetAspect, (cobj: *mut HSD_CObj, aspect: f32), |cobj: *mut HSD_CObj, aspect: f32| { unsafe { melee_runtime::tu::cobj_world::HSD_CObjSetAspect(cobj as _, aspect as _); } });
    pub static mut HSD_COBJSETCURRENT: Option<fn(*mut HSD_CObj) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn HSD_CObjSetCurrent(a0: *mut HSD_CObj) -> i32 {
        match unsafe { HSD_COBJSETCURRENT } { Some(f) => f(a0), None => { return unsafe { melee_runtime::tu::cobj::HSD_CObjSetCurrent(a0 as _) as _ }; } }
    }
    hook_or!(HSD_COBJSETEYEPOSITION, HSD_CObjSetEyePosition, (cobj: *mut HSD_CObj, a1: *mut Vec3), |cobj: *mut HSD_CObj, a1: *mut Vec3| { unsafe { melee_runtime::tu::cobj_world::HSD_CObjSetEyePosition(cobj as _, a1 as _); } });
    pub static mut HSD_COBJSETFAR: Option<fn(*mut HSD_CObj, f32)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_CObjSetFar(cobj: *mut HSD_CObj, far: f32) {
        match unsafe { HSD_COBJSETFAR } { Some(f) => f(cobj, far), None => { unsafe { melee_runtime::tu::cobj::HSD_CObjSetFar(cobj as _, far as _); } return; } }
    }
    hook!(HSD_COBJSETFOV, HSD_CObjSetFov, (a0: *mut HSD_CObj, a1: f32));
    hook_or!(HSD_COBJSETINTEREST, HSD_CObjSetInterest, (cobj: *mut HSD_CObj, a1: *mut Vec3), |cobj: *mut HSD_CObj, a1: *mut Vec3| { unsafe { melee_runtime::tu::cobj_world::HSD_CObjSetInterest(cobj as _, a1 as _); } });
    pub static mut HSD_COBJSETNEAR: Option<fn(*mut HSD_CObj, f32)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_CObjSetNear(cobj: *mut HSD_CObj, near: f32) {
        match unsafe { HSD_COBJSETNEAR } { Some(f) => f(cobj, near), None => { unsafe { melee_runtime::tu::cobj::HSD_CObjSetNear(cobj as _, near as _); } return; } }
    }
    hook!(HSD_FOGSET, HSD_FogSet, (a0: *mut HSD_Fog));
    hook!(HSD_FOREACHANIM, HSD_ForeachAnim, (obj: *mut c_void, ty: HSD_Type, mask: HSD_TypeMask, func: *mut c_void, arg_type: AObj_Arg_Type));
    hook_or!(HSD_GOBJGXLINK_8039084C, HSD_GObjGXLink_8039084C, (a0: *mut HSD_GObj), |a0: *mut HSD_GObj| { unsafe { melee_runtime::tu::gobj_native::HSD_GObjGXLink_8039084C(a0 as _); } });
    hook_or!(HSD_GOBJOBJECT_80390A70, HSD_GObjObject_80390A70, (gobj: *mut HSD_GObj, kind: u8, obj: *mut c_void), |gobj: *mut HSD_GObj, kind: u8, obj: *mut c_void| { unsafe { melee_runtime::tu::gobj_native::HSD_GObjObject_80390A70(gobj as _, kind as _, obj as _); } });
    hook_or!(HSD_GOBJ_80390ED0, HSD_GObj_80390ED0, (gobj: *mut HSD_GObj, mask: u32), |gobj: *mut HSD_GObj, mask: u32| { unsafe { melee_runtime::tu::gobj_native::HSD_GObj_80390ED0(gobj as _, mask as _); } });
    hook_or!(HSD_JOBJCLEARFLAGSALL, HSD_JObjClearFlagsAll, (a0: *mut HSD_JObj, flags: u32), |a0: *mut HSD_JObj, flags: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjClearFlagsAll(a0 as _, flags as _); } });
    hook_or!(HSD_JOBJGETFLAGS, HSD_JObjGetFlags, (jobj: *mut HSD_JObj) -> u32, |jobj: *mut HSD_JObj| unsafe { melee_runtime::tu::jobj_world::HSD_JObjGetFlags(jobj as _) as _ });
    hook!(HSD_JOBJREMOVEANIMALL, HSD_JObjRemoveAnimAll, (jobj: *mut HSD_JObj));
    hook_or!(HSD_JOBJSETFLAGSALL, HSD_JObjSetFlagsAll, (a0: *mut HSD_JObj, flags: u32), |a0: *mut HSD_JObj, flags: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetFlagsAll(a0 as _, flags as _); } });
    // rp3 HSD_LObjDeleteCurrentAll
    hook_or!(HSD_LOBJDELETECURRENTALL, HSD_LObjDeleteCurrentAll, (lobj: *mut HSD_LObj), |lobj: *mut HSD_LObj| unsafe { melee_runtime::tu::lobj::HSD_LObjDeleteCurrentAll(lobj); });
    // rp3-end

    pub static mut HSD_MEMALLOC: Option<fn(ssize_t) -> *mut c_void> = None;
    #[allow(non_snake_case)]
    pub fn HSD_MemAlloc(size: ssize_t) -> *mut c_void {
        match unsafe { HSD_MEMALLOC } { Some(f) => f(size), None => { return unsafe { melee_runtime::tu::memory::HSD_MemAlloc(size as _) as _ }; } }
    }
    hook_or!(HSD_RANDF, HSD_Randf, () -> f32, || melee_runtime::tu::random::HSD_Randf(melee_runtime::tu::globals::rand()));
    hook_or!(HSD_RANDI, HSD_Randi, (max_val: s32) -> s32, |max_val: s32| melee_runtime::tu::random::HSD_Randi(melee_runtime::tu::globals::rand(), max_val));
    hook!(HSD_SETERASECOLOR, HSD_SetEraseColor, (r: u8, g: u8, b: u8, a: u8));
    hook!(HSD_STATESETPOINTSIZE, HSD_StateSetPointSize, (a0: u8, a1: i32));
    hook_or!(MTXLIGHTPERSPECTIVE, MTXLightPerspective, (m: MtxPtr, fov_y: f32, aspect: f32, scale_s: f32, scale_t: f32, trans_s: f32, trans_t: f32), |m: MtxPtr, fov_y: f32, aspect: f32, scale_s: f32, scale_t: f32, trans_s: f32, trans_t: f32| { unsafe { melee_runtime::tu::mtx_pc::raw::MTXLightPerspective(m as _, fov_y as _, aspect as _, scale_s as _, scale_t as _, trans_s as _, trans_t as _); } });
    hook_or!(OSREPORT, OSReport, (a0: *mut i8), |a0: *mut i8| unsafe { melee_runtime::tu::os_error::OSReport(a0 as _, &[]) });
    hook_or!(PSMTXCONCAT, PSMTXConcat, (m_a: MtxPtr, m_b: MtxPtr, m_ab: MtxPtr), |m_a: MtxPtr, m_b: MtxPtr, m_ab: MtxPtr| { unsafe { melee_runtime::tu::mtx_pc::raw::PSMTXConcat(m_a as _, m_b as _, m_ab as _); } });
    hook_or!(PSMTXCOPY, PSMTXCopy, (src: MtxPtr, dst: MtxPtr), |src: MtxPtr, dst: MtxPtr| { unsafe { melee_runtime::tu::mtx_pc::raw::PSMTXCopy(src as _, dst as _); } });
    // The stage-slot (m-ex stage mod) context: the native build has none yet, so these answer 0 / do nothing.
    hook_or!(SCRIPTGAME_STAGESLOTNATIVECONTEXT, ScriptGame_StageSlotNativeContext, () -> i32, || 0);
    hook_or!(SCRIPTGAME_STAGESLOTRETAILBUFFER, ScriptGame_StageSlotRetailBuffer, (a0: *mut c_void), |_a0: *mut c_void| ());
    hook_or!(SCRIPTGAME_STAGESLOTRETAILLOADING, ScriptGame_StageSlotRetailLoading, () -> i32, || 0);
    hook!(FTDRAWCOMMON_80081118, ftDrawCommon_80081118, ());
    hook!(FTDRAWCOMMON_80081140, ftDrawCommon_80081140, ());
    pub static mut GRANIME_801C7FF8: Option<fn(*mut HSD_GObj, i32, i32, i32, f32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C7FF8(a0: *mut HSD_GObj, a1: i32, a2: i32, a3: i32, a4: f32, a5: f32) {
        match unsafe { GRANIME_801C7FF8 } { Some(f) => f(a0, a1, a2, a3, a4, a5), None => { unsafe { crate::tu::granime::grAnime_801C7FF8(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _); } return; } }
    }
    pub static mut GRANIME_801C8780: Option<fn(*mut HSD_GObj, u32, u32, f32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C8780(a0: *mut HSD_GObj, a1: u32, a2: u32, a3: f32, a4: f32) {
        match unsafe { GRANIME_801C8780 } { Some(f) => f(a0, a1, a2, a3, a4), None => { unsafe { crate::tu::granime::grAnime_801C8780(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _); } return; } }
    }
    pub static mut GRDATFILES_801C6330: Option<fn(s32) -> *mut UnkArchiveStruct> = None;
    #[allow(non_snake_case)]
    pub fn grDatFiles_801C6330(a0: s32) -> *mut UnkArchiveStruct {
        match unsafe { GRDATFILES_801C6330 } { Some(f) => f(a0), None => { return unsafe { crate::tu::grdatfiles::grDatFiles_801C6330(a0 as _) as _ }; } }
    }
    pub static mut GRDATFILES_GETARCHIVE: Option<fn() -> *mut UnkArchiveStruct> = None;
    #[allow(non_snake_case)]
    pub fn grDatFiles_GetArchive() -> *mut UnkArchiveStruct {
        match unsafe { GRDATFILES_GETARCHIVE } { Some(f) => f(), None => { return unsafe { crate::tu::grdatfiles::grDatFiles_GetArchive() as _ }; } }
    }
    pub static mut GRLIB_801C96F8: Option<fn(s32, s32, *mut Vec3) -> *mut HSD_Generator> = None;
    #[allow(non_snake_case)]
    pub fn grLib_801C96F8(a0: s32, a1: s32, a2: *mut Vec3) -> *mut HSD_Generator {
        match unsafe { GRLIB_801C96F8 } { Some(f) => f(a0, a1, a2), None => { return unsafe { crate::tu::grlib::grLib_801C96F8(a0 as _, a1 as _, a2 as _) as _ }; } }
    }
    pub static mut GRZAKOGENERATOR_801CAE04: Option<fn(*mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj> = None;
    #[allow(non_snake_case)]
    pub fn grZakoGenerator_801CAE04(a0: *mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj {
        match unsafe { GRZAKOGENERATOR_801CAE04 } { Some(f) => f(a0), None => { return unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CAE04(a0 as _) as _ }; } }
    }
    pub static mut LBDVD_80017740: Option<fn(i32, i32, i32, i32, u32, i32, i32, u8, i32) -> *mut c_void> = None;
    #[allow(non_snake_case)]
    pub fn lbDvd_80017740(ty: i32, entry_num: i32, transient_heap: i32, heap: i32, size: u32, load_state: i32, load_score: i32, a7: u8, effect_index: i32) -> *mut c_void {
        match unsafe { LBDVD_80017740 } { Some(f) => f(ty, entry_num, transient_heap, heap, size, load_state, load_score, a7, effect_index), None => { return unsafe { melee_runtime::tu::lbdvd::lbDvd_80017740(ty as _, entry_num as _, transient_heap as _, heap as _, size as _, load_state as _, load_score as _, a7 as _, effect_index as _) as _ }; } }
    }
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *mut Vec3, *mut Vec3)> = None;
    #[allow(non_snake_case)]
    pub fn lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *mut Vec3, pos1: *mut Vec3) {
        match unsafe { LB_8000B1CC } { Some(f) => f(jobj, pos0, pos1), None => { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, pos0 as _, pos1 as _); } return; } }
    }
    pub static mut LB_800121FC: Option<fn(*mut HSD_ImageDesc, i32, i32, GXTexFmt, s16) -> *mut HSD_ImageDesc> = None;
    #[allow(non_snake_case)]
    pub fn lb_800121FC(image_desc: *mut HSD_ImageDesc, width: i32, height: i32, format: GXTexFmt, entry_num: s16) -> *mut HSD_ImageDesc {
        match unsafe { LB_800121FC } { Some(f) => f(image_desc, width, height, format, entry_num), None => { return unsafe { melee_runtime::tu::lbspdisplay::lb_800121FC(image_desc as _, width as _, height as _, format as _, entry_num as _) as _ }; } }
    }
    pub static mut LB_800122C8: Option<fn(*mut HSD_ImageDesc, u16, u16, i32)> = None;
    #[allow(non_snake_case)]
    pub fn lb_800122C8(image_desc: *mut HSD_ImageDesc, origx: u16, origy: u16, clear: i32) {
        match unsafe { LB_800122C8 } {
            Some(f) => f(image_desc, origx, origy, clear),
            None => { unsafe { melee_runtime::tu::lbspdisplay::lb_800122C8(image_desc as _, origx as _, origy as _, (clear != 0)); } },
        }
    }
    pub static mut LB_80013B14: Option<fn(*mut HSD_CameraDescPerspective) -> *mut HSD_CObj> = None;
    #[allow(non_snake_case)]
    pub fn lb_80013B14(a0: *mut HSD_CameraDescPerspective) -> *mut HSD_CObj {
        match unsafe { LB_80013B14 } { Some(f) => f(a0), None => { return unsafe { melee_runtime::tu::lbspdisplay::lb_80013B14(a0 as _) as _ }; } }
    }
    pub static mut MEMZERO: Option<fn(*mut c_void, ssize_t)> = None;
    #[allow(non_snake_case)]
    pub fn memzero(mem: *mut c_void, size: ssize_t) {
        match unsafe { MEMZERO } { Some(f) => f(mem, size), None => { unsafe { melee_runtime::tu::lb_00B0::memzero(mem as _, size as _); } return; } }
    }
    pub static mut MPLIB_80055E9C: Option<fn(i32)> = None;
    #[allow(non_snake_case)]
    pub fn mpLib_80055E9C(joint_id: i32) {
        match unsafe { MPLIB_80055E9C } { Some(f) => f(joint_id), None => { unsafe { crate::tu::mplib::mpLib_80055E9C(joint_id as _); } return; } }
    }
    /// `HSD_Free` is stored as a `GObj_InitUserData` remove function: an `extern "C"` fn over a hook.
    pub static mut HSD_FREE: Option<fn(*mut c_void)> = None;
    #[allow(non_snake_case)]
    pub unsafe extern "C" fn HSD_Free(ptr: *mut c_void) {
        match HSD_FREE {
            Some(f) => f(ptr),
            None => { unsafe { melee_runtime::tu::memory::HSD_Free(ptr as _); } return; },
        }
    }
    /// `HSD_GObj** HSD_GObjPLinkHead`, `u8 HSD_GObj_CameraKind`.
    pub use melee_runtime::tu::gobj_native::HSD_GObjPLinkHead;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_CameraKind;
}

/// `struct grIzumi_YakumonoParam` (file-local in the C).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grIzumi_YakumonoParam {
    pub x0: f32,
    pub x4: i32,
    pub x8: f32,
    pub xc: f32,
    pub x10: f32,
    pub x14: f32,
    pub x18: f32,
    pub x1c: f32,
    pub x20: f32,
    pub x24: f32,
    pub x28: f32,
    pub x2c: f32,
    pub x30: f32,
    pub x34: f32,
    pub x38: f32,
    pub x3c: f32,
    pub x40: f32,
    pub x44: f32,
    pub x48: f32,
    pub x4c: f32,
    pub x50: f32,
}

/// `typedef struct IzumiReflection`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct IzumiReflection {
    pub texture_matrix: Mtx,
    pub image: *mut HSD_ImageDesc,
}

/// `typedef struct IzumiUnkCC` is a view of the `Ground` at `user_data` (`x18` at GC +0x18).
const _: () = ();

/// `static struct grIzumi_YakumonoParam* yakumono_param`.
pub static mut yakumono_param: *mut grIzumi_YakumonoParam = null_mut();

/// `void* grIzumi_StageSlotParamSwap(void*)` (TARGET_PC): swap the stage-slot parameter block.
pub unsafe fn grIzumi_StageSlotParamSwap(param: *mut c_void) -> *mut c_void {
    melee_core::ct!("grIzumi_StageSlotParamSwap");
    let previous = yakumono_param as *mut c_void;
    yakumono_param = param as *mut grIzumi_YakumonoParam;
    previous
}

/// `GrJoint grIz_803E0D60[]`.
pub static mut grIz_803E0D60: [GrJoint; 3] = [GrJoint { x: 0, y: 3, z: 1 }, GrJoint { x: 1, y: 3, z: 2 }, GrJoint { x: 2, y: 3, z: 3 }];

macro_rules! cbs {
    ($a:expr, $b:expr, $c:expr, $d:expr, $flags:expr) => {
        StageCallbacks {
            on_init: Some($a),
            callback1: Some($b),
            gobj_proc: Some($c),
            callback3: Some($d),
            anon_10: StageCallbacks_anon_135 { flags: u32::to_be($flags) },
        }
    };
}

macro_rules! cbs_filler {
    () => {
        cbs!(grIzumi_801CCA14, grIzumi_801CCA54, grIzumi_801CCA5C, grIzumi_801CCA60, 0)
    };
}

/// `StageCallbacks grIz_StageCallbacks[]`.
pub static mut grIz_StageCallbacks: [StageCallbacks; 11] = [
    cbs!(grIzumi_801CBDD4, grIzumi_801CBE00, grIzumi_801CBE08, grIzumi_801CBE0C, 0),
    cbs!(grIzumi_801CBE10, grIzumi_801CBE54, grIzumi_801CBE5C, grIzumi_801CBE60, 0),
    cbs!(grIzumi_801CCA64, grIzumi_801CCB08, grIzumi_801CCB10, grIzumi_801CCB14, 0),
    cbs!(grIzumi_801CBE64, grIzumi_801CC0CC, grIzumi_801CC0D4, grIzumi_801CC338, 0xC000_0000),
    cbs!(grIzumi_801CC33C, grIzumi_801CC350, grIzumi_801CC358, grIzumi_801CCA10, 0),
    cbs_filler!(),
    cbs_filler!(),
    cbs_filler!(),
    cbs_filler!(),
    cbs_filler!(),
    cbs_filler!(),
];

/// `StageData grIz_StageData`.
pub static mut grIz_StageData: StageData = StageData {
    grkind: Gr_Kind_Izumi,
    callbacks: unsafe { core::ptr::addr_of!(grIz_StageCallbacks) as *mut StageCallbacks },
    data1: b"/GrIz.dat\0".as_ptr() as *mut i8,
    on_init: Some(grIzumi_801CBB88),
    on_demo_init: Some(grIzumi_801CBB84),
    on_load: Some(grIzumi_OnLoad),
    on_start: Some(grIzumi_OnStart),
    callback4: Some(grIzumi_801CBCE0),
    on_touch_line: Some(grIzumi_801CD278),
    on_check_shadow_render: Some(grIzumi_801CD280),
    flags2: 1,
    joints: unsafe { core::ptr::addr_of!(grIz_803E0D60) as *mut GrJoint },
    joint_count: 3,
};

/// `HSD_WObjDesc EyeDesc` / `InterestDesc`.
pub static mut EyeDesc: HSD_WObjDesc = HSD_WObjDesc { class_name: null_mut(), pos: Vec3 { x: 0.0, y: 0.0, z: 1.0 }, robjdesc: null_mut() };
pub static mut InterestDesc: HSD_WObjDesc = HSD_WObjDesc { class_name: null_mut(), pos: Vec3 { x: 0.0, y: 0.0, z: 0.0 }, robjdesc: null_mut() };
/// `HSD_CameraDescPerspective ReflectCObjDesc`.
pub static mut ReflectCObjDesc: HSD_CameraDescPerspective = HSD_CameraDescPerspective {
    class_name: null_mut(),
    flags: 0,
    projection_type: 1,
    viewport: _HSD_RectS16 { xmin: 0, xmax: 80, ymin: 0, ymax: 60 },
    scissor: _Scissor { left: 0, right: 80, top: 0, bottom: 60 },
    eyepos: unsafe { addr_of_mut!(EyeDesc) },
    interest: unsafe { addr_of_mut!(InterestDesc) },
    roll: 0.0,
    up_vector: null_mut(),
    nnear: 0.1,
    ffar: 32768.0,
    fov: 30.0,
    aspect: 1.2173333168029785,
};

pub unsafe extern "C" fn grIzumi_801CBB84(_x: i32) { melee_core::ct!("grIzumi_801CBB84");}

pub unsafe extern "C" fn grIzumi_801CBB88() {
    melee_core::ct!("grIzumi_801CBB88");
    yakumono_param = ext::Ground_GetYakumonoParam() as *mut grIzumi_YakumonoParam;
    let si = ext::stage_info();
    (*si).unk8C.set_b4(0);
    (*si).unk8C.set_b5(1);
    grIzumi_801CBCE8(0);
    grIzumi_801CBCE8(1);
    let r3 = grIzumi_801CBCE8(3);
    ext::grAnime_801C8780(r3, 3, 0, 0.0, 1.0);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
}

pub unsafe extern "C" fn grIzumi_OnLoad() {
    melee_core::ct!("grIzumi_OnLoad");
    let mut gobj = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_LIGHT as usize);
    while !gobj.is_null() {
        if (*gobj).classifier == 0xC {
            let mut lobj = (*gobj).hsd_obj as *mut HSD_LObj;
            while !lobj.is_null() {
                // HSD_ForeachAnim(lobj, LOBJ_TYPE, ALL_TYPE_MASK, HSD_AObjSetFlags, AOBJ_ARG_AU, AOBJ_LOOP):
                // the variadic argument (AOBJ_LOOP) is the constant 0x20000000.
                ext::HSD_ForeachAnim(lobj as *mut c_void, LOBJ_TYPE, ALL_TYPE_MASK, ext::HSD_AObjSetFlags as *mut c_void, AOBJ_ARG_AU);
                lobj = lobj_next(lobj);
            }
            return;
        }
        gobj = (*gobj).next;
    }
}

/// `HSD_LObjGetNext` (jobj.h inline).
#[inline]
unsafe fn lobj_next(lobj: *mut HSD_LObj) -> *mut HSD_LObj {
    if lobj.is_null() { null_mut() } else { (*lobj).next }
}

pub unsafe extern "C" fn grIzumi_OnStart() {
    melee_core::ct!("grIzumi_OnStart");
    ext::grZakoGenerator_801CAE04(null_mut());
}

pub unsafe fn grIzumi_801CBCE8(gobj_id: i32) -> *mut HSD_GObj {
    melee_core::ct!("grIzumi_801CBCE8");
    let callbacks: *mut StageCallbacks = &mut grIz_StageCallbacks[gobj_id as usize];
    let gobj = ext::Ground_GetStageGObj(gobj_id);

    if !gobj.is_null() {
        let gp = (*gobj).user_data as *mut Ground;
        (*gp).x8_callback = None;
        (*gp).xC_callback = None;
        ext::GObj_SetupGXLink(gobj, Some(ext::grDisplay_801C5DB0), 3, 0);

        if (*callbacks).callback3.is_some() {
            (*gp).x1C_callback = (*callbacks).callback3;
        }

        if let Some(f) = (*callbacks).on_init {
            f(gobj);
        }

        if (*callbacks).gobj_proc.is_some() {
            ext::HSD_GObj_SetupProc(gobj, (*callbacks).gobj_proc, 4);
        }
    } else {
        let mut msg = *b"%s:%d: couldn t get gobj(id=%d)\n\0";
        ext::OSReport(msg.as_mut_ptr() as *mut i8);
    }

    gobj
}

pub unsafe extern "C" fn grIzumi_801CBDD4(gobj: *mut HSD_GObj) {
    melee_core::ct!("grIzumi_801CBDD4");
    ext::grAnime_801C8138(gobj, (*GET_GROUND(gobj)).map_id as enum_t, 0);
}

pub unsafe extern "C" fn grIzumi_801CBE10(gobj: *mut HSD_GObj) {
    melee_core::ct!("grIzumi_801CBE10");
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
    (*gp).x11_flags.set_b012(1);
}

pub unsafe extern "C" fn grIzumi_801CBE64(gobj: *mut HSD_GObj) {
    melee_core::ct!("grIzumi_801CBE64");
    let gp = GET_GROUND(gobj);
    let mut jobj = GET_JOBJ(gobj);
    ext::Ground_InitMapColl(jobj, (*gp).map_id as s32);
    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
    (*gp).x11_flags.set_b012(1);
    (*gp).x10_flags.set_b5(1);
    (*gp).u.izumi.xD0 = ext::Ground_801C3FA4(gobj, 1);
    (*gp).u.izumi.xD4 = ext::Ground_801C3FA4(gobj, 2);
    // TARGET_PC: a stage-slot (m-ex stage) visit builds the platforms only
    if ext::ScriptGame_StageSlotNativeContext() != 0 {
        let mut p = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ext::lb_8000B1CC(ext::Ground_801C3FA4(gobj, 4), null_mut(), &mut p);
        let mut plat = grIzumi_801CCBDC((*yakumono_param).x0, &mut p, 0, (*gp).u.izumi.xD0);
        (*GET_GROUND(plat)).u.izumi2.xDC = (*yakumono_param).xc;
        ext::lb_8000B1CC(ext::Ground_801C3FA4(gobj, 6), null_mut(), &mut p);
        plat = grIzumi_801CCBDC((*yakumono_param).x8, &mut p, 1, (*gp).u.izumi.xD4);
        (*GET_GROUND(plat)).u.izumi2.xDC = (*yakumono_param).xc;
        return;
    }
    (*gp).u.izumi.xC8 = grIzumi_801CCD98();
    let tobj = grIzumi_801CD090(gobj, (*((*(*gp).u.izumi.xC8).user_data as *mut IzumiReflection)).image);
    if !tobj.is_null() {
        (*tobj).src = 0;
        (*tobj).wrap_s = 0;
        (*tobj).wrap_t = 0;
        (*tobj).flags = ((*tobj).flags & !0x1FF) | 0x103;
    } else {
        let mut msg = *b"%s:%d:Warning: not found tobj !\n\0";
        ext::OSReport(msg.as_mut_ptr() as *mut i8);
    }
    (*gp).u.izumi.xC4 = tobj;
    (*gobj).render_cb = Some(grIzumi_801CD220);
    grIzumi_801CCB18(gobj);
    {
        let mut x = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ext::grLib_801C96F8(0x7534, 0x1E, &mut x);
    }
    {
        let mut y = Vec3 { x: 0.0, y: 1.0, z: -27.0 };
        y.x *= ext::Ground_801C0498();
        y.y *= ext::Ground_801C0498();
        y.z *= ext::Ground_801C0498();
        ext::grLib_801C96F8(0x7536, 0x1E, &mut y);
    }
    (*gp).u.izumi.xCC = grIzumi_801CBCE8(2);
    // ((IzumiUnkCC*) user_data)->x18 = xC8: the `Ground.x18` slot (GC +0x18)
    (*((*(*gp).u.izumi.xCC).user_data as *mut Ground)).x18 = (*gp).u.izumi.xC8;
    jobj = ext::Ground_801C3FA4(gobj, 4);
    {
        let mut x38 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ext::lb_8000B1CC(jobj, null_mut(), &mut x38);
        {
            let plat = grIzumi_801CCBDC((*yakumono_param).x0, &mut x38, 0, (*gp).u.izumi.xD0);
            let platground = GET_GROUND(plat);
            (*platground).u.izumi2.xDC = (*yakumono_param).xc;
            (*platground).x10_flags.set_b3(1);
            (*platground).x18 = (*gp).u.izumi.xC8;
        }
        jobj = ext::Ground_801C3FA4(gobj, 6);
        ext::lb_8000B1CC(jobj, null_mut(), &mut x38);
        {
            let plat = grIzumi_801CCBDC((*yakumono_param).x8, &mut x38, 1, (*gp).u.izumi.xD4);
            let platground = GET_GROUND(plat);
            (*platground).u.izumi2.xDC = (*yakumono_param).xc;
            (*platground).x10_flags.set_b3(1);
            (*platground).x18 = (*gp).u.izumi.xC8;
        }
    }
}

pub unsafe extern "C" fn grIzumi_801CC0D4(gobj: *mut HSD_GObj) {
    melee_core::ct!("grIzumi_801CC0D4");
    let gp = GET_GROUND(gobj);
    if !(*gp).u.izumi.xCC.is_null() {
        let gp2 = GET_GROUND((*gp).u.izumi.xCC);
        let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        if !(*gp).u.izumi.xD0.is_null() {
            HSD_JObjGetTranslation((*gp).u.izumi.xD0, &mut vec);
            HSD_JObjSetTranslate((*gp2).u.izumi2.xC4, &vec);
            if vec.y < 0.0 {
                let flags = ext::HSD_JObjGetFlags((*gp2).u.izumi2.xC4);
                if flags & 0x10 == 0 {
                    ext::HSD_JObjSetFlagsAll((*gp2).u.izumi2.xC4, JOBJ_HIDDEN as u32);
                }
            } else {
                let flags = ext::HSD_JObjGetFlags((*gp2).u.izumi2.xC4);
                if flags & 0x10 != 0 {
                    ext::HSD_JObjClearFlagsAll((*gp2).u.izumi2.xC4, JOBJ_HIDDEN as u32);
                }
            }
        }
        if !(*gp).u.izumi.xD4.is_null() {
            HSD_JObjGetTranslation((*gp).u.izumi.xD4, &mut vec);
            HSD_JObjSetTranslate((*gp2).u.izumi2.xC8, &vec);
            if vec.y < 0.0 {
                let flags = ext::HSD_JObjGetFlags((*gp2).u.izumi2.xC8);
                if flags & 0x10 == 0 {
                    ext::HSD_JObjSetFlagsAll((*gp2).u.izumi2.xC8, JOBJ_HIDDEN as u32);
                }
            } else {
                let flags = ext::HSD_JObjGetFlags((*gp2).u.izumi2.xC8);
                if flags & 0x10 != 0 {
                    ext::HSD_JObjClearFlagsAll((*gp2).u.izumi2.xC8, JOBJ_HIDDEN as u32);
                }
            }
        }
    }
    ext::lb_800115F4();
}

pub unsafe extern "C" fn grIzumi_801CC33C(gobj: *mut HSD_GObj) {
    melee_core::ct!("grIzumi_801CC33C");
    let gp = GET_GROUND(gobj);
    (*gp).x8_callback = None;
    (*gp).xC_callback = None;
}

pub unsafe extern "C" fn grIzumi_801CC358(gobj: *mut HSD_GObj) {
    melee_core::ct!("grIzumi_801CC358");
    let mut r29 = false;
    let jobj = GET_JOBJ(gobj);
    let gp = GET_GROUND(gobj);
    let yp = yakumono_param;
    match (*gp).u.izumi3.xC4 {
        0 => {
            (*gp).u.izumi3.xC4 = 1;
            (*gp).u.izumi3.xC6 = rand_range((*yp).x3c as i32, (*yp).x38 as i32) as i16;
            r29 = true;
        }
        1 => {
            let c = (*gp).u.izumi3.xC6;
            (*gp).u.izumi3.xC6 = c.wrapping_sub(1);
            if c < 0 {
                let mut f = ext::HSD_Randf() * ((*yp).x40 + (*yp).x44 + (*yp).x48);
                f -= (*yp).x40;
                if (f as f64) < 0.0 {
                    (*gp).u.izumi3.xC4 = 2;
                    (*gp).u.izumi3.xD4 = -1.0;
                } else {
                    f -= (*yp).x48;
                    if (f as f64) < 0.0 {
                        let fff = ext::HSD_Randf();
                        let mut ff = (*yp).x1c - (*yp).x18;
                        ff = fmadds(ff, fff, (*yp).x18);
                        (*gp).u.izumi3.xC4 = 2;
                        if (*gp).u.izumi3.xD0 < (*gp).u.izumi3.xDC {
                            if ext::HSD_Randf() < (*yp).x30 {
                                (*gp).u.izumi3.xD4 -= ff;
                            } else {
                                (*gp).u.izumi3.xD4 += ff;
                            }
                        } else if (*gp).u.izumi3.xD0 > (*gp).u.izumi3.xDC {
                            if ext::HSD_Randf() < (*yp).x34 {
                                (*gp).u.izumi3.xD4 += ff;
                            } else {
                                (*gp).u.izumi3.xD4 -= ff;
                            }
                        } else if (ext::HSD_Randf() as f64) < 0.5 {
                            (*gp).u.izumi3.xD4 += ff;
                        } else {
                            (*gp).u.izumi3.xD4 -= ff;
                        }
                        if (*gp).u.izumi3.xD4 > (*yp).x20 {
                            (*gp).u.izumi3.xD4 = (*yp).x20;
                        } else if (*gp).u.izumi3.xD4 < (*yp).x24 {
                            (*gp).u.izumi3.xD4 = (*yp).x24;
                        }
                    } else {
                        (*gp).u.izumi3.xC6 = rand_range((*yp).x3c as i32, (*yp).x38 as i32) as i16;
                    }
                }
            }
        }
        2 => {
            let f = (*gp).u.izumi3.xD4 - (*gp).u.izumi3.xD0;
            if f > 0.0 {
                if f < (*yp).x28 {
                    (*gp).u.izumi3.xD0 = (*gp).u.izumi3.xD4;
                    (*gp).u.izumi3.xC4 = 0;
                } else {
                    (*gp).u.izumi3.xD0 += (*yp).x28;
                }
            } else if f < 0.0 {
                if -f < (*yp).x2c {
                    (*gp).u.izumi3.xD0 = (*gp).u.izumi3.xD4;
                    if (*gp).u.izumi3.xD0 < (*yp).x24 {
                        (*gp).u.izumi3.xC4 = 3;
                    } else {
                        (*gp).u.izumi3.xC4 = 0;
                    }
                } else {
                    (*gp).u.izumi3.xD0 -= (*yp).x2c;
                }
            } else {
                (*gp).u.izumi3.xC4 = 0;
            }
            r29 = true;
        }
        3 => {
            (*gp).u.izumi3.xC4 = 4;
            (*gp).u.izumi3.xC6 = rand_range((*yp).x50 as i32, (*yp).x4c as i32) as i16;
            ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
            ext::HSD_JObjRemoveAnimAll(jobj);
            let mut f = HSD_JObjGetTranslationY(jobj);
            // f += -1.0 (a double constant): add in double, round to float
            f = (f as f64 + -1.0) as f32;
            HSD_JObjSetTranslateY((*gp).u.izumi3.xCC, f);
        }
        4 => {
            let c = (*gp).u.izumi3.xC6;
            (*gp).u.izumi3.xC6 = c.wrapping_sub(1);
            if c < 0 {
                (*gp).u.izumi3.xC4 = 2;
                (*gp).u.izumi3.xD4 = (*gp).u.izumi3.xDC;
                ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
                ext::grAnime_801C7FF8(gobj, 0, 7, 0, 0.0, 1.0);
                r29 = true;
            }
        }
        _ => {}
    }
    if r29 {
        let jobj2 = HSD_JObjGetChild(jobj);
        if !jobj2.is_null() {
            // gp->u.izumi3.xD0 / (double) gp->u.izumi3.xD8, rounded to float
            let mut f = ((*gp).u.izumi3.xD0 as f64 / (*gp).u.izumi3.xD8 as f64) as f32;
            if f < 0.01_f32 {
                f = 0.01_f32;
            }
            HSD_JObjSetScaleX(jobj2, fmadds(0.5, f, 0.5));
            HSD_JObjSetScaleY(jobj2, f);
            HSD_JObjSetTranslateY((*gp).u.izumi3.xCC, (*gp).u.izumi3.xD0);
        }
    }
    ext::mpLib_80055E9C((*gp).u.izumi3.xC8 as i32);
}

pub unsafe extern "C" fn grIzumi_801CCA14(gobj: *mut HSD_GObj) {
    melee_core::ct!("grIzumi_801CCA14");
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
    (*gp).x8_callback = None;
    (*gp).xC_callback = None;
}

pub unsafe extern "C" fn grIzumi_801CCA64(gobj: *mut HSD_GObj) {
    melee_core::ct!("grIzumi_801CCA64");
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
    (*gp).x8_callback = None;
    (*gp).xC_callback = None;
    ext::HSD_GObjGXLink_8039084C(gobj);
    ext::GObj_SetupGXLink(gobj, Some(ext::grDisplay_801C5DB0), 2, 0);
    (*gp).u.izumi2.xC4 = ext::Ground_801C3FA4(gobj, 2);
    ext::HSD_JObjSetFlagsAll((*gp).u.izumi2.xC4, JOBJ_HIDDEN as u32);
    (*gp).u.izumi2.xC8 = ext::Ground_801C3FA4(gobj, 3);
    ext::HSD_JObjSetFlagsAll((*gp).u.izumi2.xC8, JOBJ_HIDDEN as u32);
}

pub unsafe fn grIzumi_801CCB18(_gobj: *mut HSD_GObj) {
    melee_core::ct!("grIzumi_801CCB18");
    let j = ext::HSD_ArchiveGetPublicAddress((*ext::grDatFiles_GetArchive()).unk0, b"GrdIzumiStar_TopN_joint\0".as_ptr() as *const i8)
        as *mut HSD_Joint;
    if !j.is_null() {
        let sub = ext::Ground_801C1A20(j, -1);
        if !sub.is_null() {
            let gp = GET_GROUND(sub);
            ext::GObj_SetupGXLink(sub, Some(grIzumi_801CCB90), 3, 0);
            (*gp).x11_flags.set_b012(2);
        }
    }
}

pub unsafe extern "C" fn grIzumi_801CCB90(gobj: *mut HSD_GObj, renderpass: i32) {
    melee_core::ct!("grIzumi_801CCB90");
    ext::HSD_StateSetPointSize(18, 0);
    ext::grDisplay_801C5DB0(gobj, renderpass);
}

pub unsafe fn grIzumi_801CCBDC(height: f32, a: *mut Vec3, b: i32, jobj: *mut HSD_JObj) -> *mut HSD_GObj {
    melee_core::ct!("grIzumi_801CCBDC");
    let gobj = grIzumi_801CBCE8(4);
    if !gobj.is_null() {
        let gp = GET_GROUND(gobj);
        if !gp.is_null() {
            let mut jobj2 = (*gobj).hsd_obj as *mut HSD_JObj;
            if !jobj2.is_null() {
                let aa = *a;
                HSD_JObjSetTranslate(jobj2, &aa);
                (*gp).u.izumi3.xD4 = height;
                (*gp).u.izumi3.xD0 = height;
                (*gp).u.izumi3.xC8 = b as i16;
                (*gp).u.izumi3.xCC = jobj;
                if height < 0.0 {
                    (*gp).u.izumi3.xC4 = 3;
                } else {
                    (*gp).u.izumi3.xC4 = 0;
                    ext::grAnime_801C7FF8(gobj, 0, 7, 0, 0.0, 1.0);
                }
                jobj2 = ext::Ground_801C3FA4(gobj, 2);
                if !jobj2.is_null() {
                    let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                    ext::lb_8000B1CC(jobj2, null_mut(), &mut vec);
                    (*gp).u.izumi3.xD8 = (vec.y - aa.y) / ext::Ground_801C0498();
                } else {
                    (*gp).u.izumi3.xD8 = 45.0;
                }
                grIzumi_801CC358(gobj);
                return gobj;
            }
        }
    }

    let mut msg = *b"%s:%d:oioi...\n\0";
    ext::OSReport(msg.as_mut_ptr() as *mut i8);
    // while (true) {}: the C spins forever; here that is a hang the game would also have
    panic!("grIzumi_801CCBDC: oioi... (stage gobj / Ground / jobj missing)");
}

/// TARGET_PC `grIzumi_StageSlotReflection` (grizumi_stage_slot_retail.inc).
unsafe fn grIzumi_StageSlotReflection(image: *mut HSD_ImageDesc) {
    let private_visit = ext::ScriptGame_StageSlotRetailLoading();
    ext::lb_800121FC(image, 80, 60, 4, if private_visit != 0 { 0 } else { 2001 });
    if private_visit != 0 {
        ext::ScriptGame_StageSlotRetailBuffer((*image).image_ptr);
    }
}

pub unsafe fn grIzumi_801CCD98() -> *mut HSD_GObj {
    melee_core::ct!("grIzumi_801CCD98");
    let gobj = ext::GObj_Create(0x11, 0x12, 0);
    let cobj = ext::lb_80013B14(addr_of_mut!(ReflectCObjDesc));
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_CameraKind, cobj as *mut c_void);
    ext::GObj_SetupGXLinkMaxSorted(gobj, Some(grIzumi_801CCEA0), 2);
    let refl = ext::HSD_MemAlloc(core::mem::size_of::<IzumiReflection>() as ssize_t) as *mut IzumiReflection;
    ext::GObj_InitUserData(gobj, 3, Some(ext::HSD_Free), refl as *mut c_void);
    let dat = ext::grDatFiles_801C6330(3);
    (*refl).image = ext::HSD_ArchiveGetPublicAddress((*dat).unk0, b"GrdIzumi_cd_wt_GrdIzumiDummy1_1_image_desc\0".as_ptr() as *const i8)
        as *mut HSD_ImageDesc;
    if !(*refl).image.is_null() {
        ext::memzero((*refl).image as *mut c_void, core::mem::size_of::<HSD_ImageDesc>() as ssize_t);
        grIzumi_StageSlotReflection((*refl).image);
    } else {
        let mut msg = *b"not found mirror image desc! (GrdIzumi_cd_wt_GrdIzumiDummy1_1_image_desc)\n\0";
        ext::OSReport(msg.as_mut_ptr() as *mut i8);
        assert_(false, 968, "0");
    }
    gobj
}

pub unsafe extern "C" fn grIzumi_801CCEA0(gobj: *mut HSD_GObj, _renderpass: i32) {
    melee_core::ct!("grIzumi_801CCEA0");
    let mut mtx: Mtx = [[0.0; 4]; 3];
    let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let refl = (*gobj).user_data as *mut IzumiReflection;

    if !(*refl).image.is_null() {
        let cobj = (*gobj).hsd_obj as *mut HSD_CObj;
        ext::ftDrawCommon_80081140();
        ext::Camera_8002A4AC(ext::Camera_80030A50());

        let dst = (*gobj).hsd_obj as *mut HSD_CObj;
        let src_gobj = ext::Camera_80030A50();
        if !src_gobj.is_null() {
            let src = (*src_gobj).hsd_obj as *mut HSD_CObj;
            if !src.is_null() {
                ext::HSD_CObjSetNear(dst, ext::HSD_CObjGetNear(src));
                ext::HSD_CObjSetFar(dst, ext::HSD_CObjGetFar(src));
                ext::HSD_CObjSetFov(dst, ext::HSD_CObjGetFov(src));
                ext::HSD_CObjSetAspect(dst, ext::HSD_CObjGetAuthoredAspect(src));
                ext::HSD_CObjGetEyePosition(src, &mut vec);
                vec.y = -vec.y;
                ext::HSD_CObjSetEyePosition(dst, &mut vec);
                ext::HSD_CObjGetInterest(src, &mut vec);
                vec.y = -vec.y;
                ext::HSD_CObjSetInterest(dst, &mut vec);
            }
        }
        if ext::HSD_CObjSetCurrent(cobj) != 0 {
            ext::HSD_SetEraseColor(0xFF, 0xFF, 0xFF, 1);
            ext::HSD_CObjEraseScreen(cobj, 1, 0, 0);
            ext::HSD_LObjDeleteCurrentAll(null_mut());
            ext::Camera_800310A0(0);
            ext::Camera_80031074(1);
            (*gobj).gxlink_prios = 0x25;
            ext::HSD_GObj_80390ED0(gobj, 3);
            ext::Camera_80031074(0);
            (*gobj).gxlink_prios = 0x70;
            ext::HSD_GObj_80390ED0(gobj, 7);
            ext::HSD_FogSet(null_mut());
            (*gobj).gxlink_prios = 0x80;
            ext::HSD_GObj_80390ED0(gobj, 7);
            ext::HSD_CObjEndCurrent();
        }
        ext::lb_800122C8((*refl).image, 0, 0, 1);
        ext::MTXLightPerspective(
            mtx.as_mut_ptr(),
            (*cobj).projection_param.perspective.fov,
            (*cobj).projection_param.perspective.aspect,
            0.49_f32,
            -0.49_f32,
            0.5_f32,
            0.5_f32,
        );
        ext::PSMTXConcat(mtx.as_mut_ptr(), ext::HSD_CObjGetViewingMtxPtr(cobj), (*refl).texture_matrix.as_mut_ptr());
        ext::ftDrawCommon_80081118();
    }
}

/// `static inline bool check_flag_4020(HSD_JObj*)`.
#[inline]
unsafe fn check_flag_4020(jobj: *mut HSD_JObj) -> bool {
    (*jobj).flags & 0x4020 == 0
}

pub unsafe fn grIzumi_801CD090(gobj: *mut HSD_GObj, image: *mut HSD_ImageDesc) -> *mut HSD_TObj {
    melee_core::ct!("grIzumi_801CD090");
    let mut jobj = GET_JOBJ(gobj);
    while !jobj.is_null() {
        if check_flag_4020(jobj) {
            let mut dobj = (*jobj).u.dobj;
            while !dobj.is_null() {
                if !(*dobj).mobj.is_null() {
                    let mut tobj = (*(*dobj).mobj).tobj;
                    while !tobj.is_null() {
                        if (*tobj).imagedesc == image {
                            return tobj;
                        }
                        tobj = (*tobj).next;
                    }
                }
                dobj = (*dobj).next;
            }
        }
        if (*jobj).flags & 0x1000 == 0 && !HSD_JObjGetChild(jobj).is_null() {
            jobj = HSD_JObjGetChild(jobj);
            continue;
        }
        if !HSD_JObjGetNext(jobj).is_null() {
            jobj = HSD_JObjGetNext(jobj);
        } else {
            loop {
                if HSD_JObjGetParent(jobj).is_null() {
                    jobj = null_mut();
                    break;
                } else if !HSD_JObjGetNext(HSD_JObjGetParent(jobj)).is_null() {
                    jobj = HSD_JObjGetNext(HSD_JObjGetParent(jobj));
                    break;
                } else {
                    jobj = HSD_JObjGetParent(jobj);
                }
            }
        }
    }
    null_mut()
}

pub unsafe extern "C" fn grIzumi_801CD220(gobj: *mut HSD_GObj, renderpass: i32) {
    melee_core::ct!("grIzumi_801CD220");
    let gp = GET_GROUND(gobj);
    let refl = (*(*gp).u.izumi.xC8).user_data as *mut IzumiReflection;
    let tobj = (*gp).u.izumi.xC4;
    // TARGET_PC: a skin without the shared image desc leaves tobj NULL
    if !tobj.is_null() {
        ext::PSMTXCopy((*refl).texture_matrix.as_mut_ptr(), (*tobj).mtx.as_mut_ptr());
    }
    ext::grDisplay_801C5DB0(gobj, renderpass);
}

pub unsafe extern "C" fn grIzumi_801CD280(a: *mut Vec3, _b: i32, jobj: *mut HSD_JObj) -> i32 {
    melee_core::ct!("grIzumi_801CD280");
    let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::lb_8000B1CC(jobj, null_mut(), &mut vec);
    if (*a).y > vec.y {
        1
    } else {
        0
    }
}

pub unsafe fn grIzumi_801CD2D4() {
    melee_core::ct!("grIzumi_801CD2D4");
    let size = ext::GXGetTexBufferSize(80, 60, 4, 0, 0);
    ext::lbDvd_80017740(0, 2001, 4, 4, (size.wrapping_add(31)) & !31, 0, 7, 16, 0);
}

// ---- folded from grizumi.rs (consolidation steps 21/22) ----
/// grIzumi_OnStart_fp (gr/grizumi.c:226)
    pub unsafe extern "C" fn grIzumi_OnStart_fp() { let c = c(); c.grzakogenerator_801cae04(core::ptr::null_mut()); }

// ---- folded from grizumi.rs (consolidation steps 21/22) ----
/// grIzumi_801CBB84_fp (gr/grizumi.c:186)
    pub extern "C" fn grIzumi_801CBB84_fp(a0: i32) {  }

/// grIzumi_801CBCE0 (gr/grizumi.c:231)
    pub extern "C" fn grIzumi_801CBCE0() -> i32 { 0 }

/// grIzumi_801CBE00 (gr/grizumi.c:273)
    pub extern "C" fn grIzumi_801CBE00(a0: *mut HSD_GObj) -> i32 { 0 }

/// grIzumi_801CBE08 (gr/grizumi.c:278)
    pub extern "C" fn grIzumi_801CBE08(a0: *mut HSD_GObj) {  }

/// grIzumi_801CBE0C (gr/grizumi.c:283)
    pub extern "C" fn grIzumi_801CBE0C(a0: *mut HSD_GObj) {  }

/// grIzumi_801CBE54 (gr/grizumi.c:295)
    pub extern "C" fn grIzumi_801CBE54(a0: *mut HSD_GObj) -> i32 { 0 }

/// grIzumi_801CBE5C (gr/grizumi.c:300)
    pub extern "C" fn grIzumi_801CBE5C(a0: *mut HSD_GObj) {  }

/// grIzumi_801CBE60 (gr/grizumi.c:305)
    pub extern "C" fn grIzumi_801CBE60(a0: *mut HSD_GObj) {  }

/// grIzumi_801CC0CC (gr/grizumi.c:395)
    pub extern "C" fn grIzumi_801CC0CC(a0: *mut HSD_GObj) -> i32 { 0 }

/// grIzumi_801CC338 (gr/grizumi.c:440)
    pub extern "C" fn grIzumi_801CC338(a0: *mut HSD_GObj) {  }

/// grIzumi_801CC350 (gr/grizumi.c:452)
    pub extern "C" fn grIzumi_801CC350(a0: *mut HSD_GObj) -> i32 { 0 }

/// grIzumi_801CCA10 (gr/grizumi.c:588)
    pub extern "C" fn grIzumi_801CCA10(a0: *mut HSD_GObj) {  }

/// grIzumi_801CCA54 (gr/grizumi.c:601)
    pub extern "C" fn grIzumi_801CCA54(a0: *mut HSD_GObj) -> i32 { 0 }

/// grIzumi_801CCA5C (gr/grizumi.c:606)
    pub extern "C" fn grIzumi_801CCA5C(a0: *mut HSD_GObj) {  }

/// grIzumi_801CCA60 (gr/grizumi.c:611)
    pub extern "C" fn grIzumi_801CCA60(a0: *mut HSD_GObj) {  }

/// grIzumi_801CCB08 (gr/grizumi.c:630)
    pub extern "C" fn grIzumi_801CCB08(a0: *mut HSD_GObj) -> i32 { 0 }

/// grIzumi_801CCB10 (gr/grizumi.c:635)
    pub extern "C" fn grIzumi_801CCB10(a0: *mut HSD_GObj) {  }

/// grIzumi_801CCB14 (gr/grizumi.c:640)
    pub extern "C" fn grIzumi_801CCB14(a0: *mut HSD_GObj) {  }

/// grIzumi_801CD278 (gr/grizumi.c:906)
    pub extern "C" fn grIzumi_801CD278(a0: i32) -> *mut DynamicsDesc { core::ptr::null_mut() }
