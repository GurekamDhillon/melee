//! Port of src/melee/if/iftime.c (built `TARGET_PC`): the VS match timer and the final-seconds
//! countdown HUD. Every call out of the TU goes through `ext`.
//! Checked against the C (whole TU, shipping contraction) by tests/tu_iftime.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_world/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_none {
        ($name:ident, ($($a:ident),*)) => { melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) };
        ($name:ident, ($($a:ident),*), $def:expr) => { ($def)($($a),*) };
    }
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => ext_none!($name, ($($a),*) $(, $def)?),
                }
            } )*
        };
    }
    ext_fns! {
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj = |a0: u16, a1: u8, a2: u8| unsafe { melee_runtime::tu::gobj_native::GObj_Create(a0 as _, a1 as _, a2 as _) as _ };
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, isize)>, a2: u8, a3: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, isize)>, a2: u8, a3: u32| { unsafe { melee_runtime::tu::gobj_native::GObj_SetupGXLink(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn HSD_AObjReqAnim(a0: *mut HSD_AObj, a1: f32) = |a0: *mut HSD_AObj, a1: f32| { unsafe { melee_runtime::tu::aobj::HSD_AObjReqAnim(a0 as _, a1 as _); } };
    fn HSD_AObjSetRate(a0: *mut HSD_AObj, a1: f32) = |a0: *mut HSD_AObj, a1: f32| { unsafe { melee_runtime::tu::aobj::HSD_AObjSetRate(a0 as _, a1 as _); } };
    fn HSD_GObjFree(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { melee_runtime::tu::gobj_native::HSD_GObjFree(a0 as _); } };
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void) = |a0: *mut HSD_GObj, a1: u8, a2: *mut c_void| { unsafe { melee_runtime::tu::gobj_native::HSD_GObjObject_80390A70(a0 as _, a1 as _, a2 as _); } };
    fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: isize) = |a0: *mut HSD_GObj, a1: isize| { unsafe { melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0 as _, a1 as _); } };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(a0 as _, a1 as _, a2 as _) as _ };
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjClearFlags(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjClearFlags(a0 as _, a1 as _); } };
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjClearFlagsAll(a0 as _, a1 as _); } };
    fn HSD_JObjGetFlags(a0: *mut HSD_JObj) -> u32 = |a0: *mut HSD_JObj| unsafe { melee_runtime::tu::jobj_world::HSD_JObjGetFlags(a0 as _) as _ };
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj = |a0: *mut HSD_Joint| unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0 as _) as _ };
    fn HSD_JObjRemoveAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
    fn HSD_JObjSetFlags(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetFlags(a0 as _, a1 as _); } };
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetFlagsAll(a0 as _, a1 as _); } };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) = |a0: *mut HSD_JObj| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetMtxDirtySub(a0 as _); } };
    fn Ui_RetailHidden(a0: i32) -> i32;
    fn gm_8016AEEC() -> u32 = || unsafe { crate::gm::gm_8016AEEC() as _ };
    fn gm_8016AF0C() -> u16 = || unsafe { crate::gm::gm_8016AF0C() as _ };
    fn gm_8016B110() -> i32 = || unsafe { crate::gm::gm_8016B110() as _ };
    fn gm_GetStartMeleeRules() -> *mut StartMeleeRules = || unsafe { crate::gm::gm_GetStartMeleeRules() as _ };
    fn ifAll_GetArchive() -> *mut *mut HSD_Archive = || unsafe { crate::tu::ifall::ifAll_GetArchive() as _ };
    fn ifAll_GetTimerPosition() -> *mut Vec = || unsafe { crate::tu::ifall::ifAll_GetTimerPosition() as _ };
    fn lbArchive_LoadSections(a0: *mut HSD_Archive, a1: *mut c_void, a2: *const u8, rest: &[(*mut c_void, *const u8)]) = |a0: *mut HSD_Archive, a1: *mut c_void, a2: *const u8, rest: &[(*mut c_void, *const u8)]| { unsafe { let mut v: std::vec::Vec<*mut core::ffi::c_void> = std::vec![a1 as *mut core::ffi::c_void, a2 as *mut core::ffi::c_void]; for (s, n) in rest.iter() { v.push(*s as *mut core::ffi::c_void); v.push(*n as *mut core::ffi::c_void); } v.push(core::ptr::null_mut()); melee_runtime::tu::lbarchive::lbArchive_LoadSections(a0 as _, &v); } };
    fn lb_8000C07C(a0: *mut HSD_JObj, a1: i32, a2: *mut *mut HSD_AnimJoint, a3: *mut *mut HSD_MatAnimJoint, a4: *mut *mut HSD_ShapeAnimJoint) = |a0: *mut HSD_JObj, a1: i32, a2: *mut *mut HSD_AnimJoint, a3: *mut *mut HSD_MatAnimJoint, a4: *mut *mut HSD_ShapeAnimJoint| { unsafe { melee_runtime::tu::lb_00B0::lb_8000C07C(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _); } };
    fn lb_8000C0E8(a0: *mut HSD_JObj, a1: i32, a2: *mut DynamicModelDesc) = |a0: *mut HSD_JObj, a1: i32, a2: *mut DynamicModelDesc| { unsafe { melee_runtime::tu::lb_00B0::lb_8000C0E8(a0 as _, a1 as _, a2 as _); } };
    fn memzero(a0: *mut c_void, a1: i32) = super::memzero_default;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub use melee_runtime::tu::gobj_native::HSD_GObj_JObjKind;
}
// END GENERATED EXT

crate::jobj_inlines!();

/// `AT_RE_HUD_TIMER` (pc/platform/gw_ui_retail_ids.h).
const AT_RE_HUD_TIMER: i32 = 2;

const IFTIME_HOUR_SEP: usize = 0;
const IFTIME_HOUR_TEN: usize = 2;
const IFTIME_HOUR_ONE: usize = 3;
const IFTIME_MINUTE_TEN: usize = 4;
const IFTIME_MINUTE_ONE: usize = 5;
const IFTIME_CENTISECOND_TEN: usize = 6;
const IFTIME_CENTISECOND_ONE: usize = 7;
const IFTIME_SECOND_TEN: usize = 8;
const IFTIME_SECOND_ONE: usize = 9;

/// `struct ifTime_data` (the C defines it in the .c): the two timer GObjs, the ten digit joints, the
/// countdown's second and the countdown model table. GameCube size 0x3C.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ifTime_data {
    pub match_timer: *mut HSD_GObj,
    pub countdown_timer: *mut HSD_GObj,
    pub digits: [*mut HSD_JObj; 10],
    pub countdown_seconds: u8,
    pub pad31: [i8; 7],
    pub countdown_timer_models: *mut *mut DynamicModelDesc,
}

melee_core::gc_layout!(ifTime_data, "ifTime_data", size 0x3C, align 4, [
    match_timer @ 0x0: 4 "HSD_GObj*",
    countdown_timer @ 0x4: 4 "HSD_GObj*",
    digits @ 0x8: 0x28 "HSD_JObj*[10]",
    countdown_seconds @ 0x30: 1 "u8",
    pad31 @ 0x31: 7 "u8[7]",
    countdown_timer_models @ 0x38: 4 "DynamicModelDesc**",
]);

pub static mut ifTime_data: ifTime_data = ifTime_data {
    match_timer: null_mut(),
    countdown_timer: null_mut(),
    digits: [null_mut(); 10],
    countdown_seconds: 0,
    pad31: [0; 7],
    countdown_timer_models: null_mut(),
};
pub static mut ifTime_match_timer_models: DynamicModelDesc = unsafe { core::mem::zeroed() };

/// `memzero` (lb_00B0.c): the real behaviour, used unless a test installs a hook.
pub fn memzero_default(mem: *mut c_void, size: i32) {
    melee_core::ct!("memzero_default");
    unsafe { core::ptr::write_bytes(mem as *mut u8, 0, size as usize) }
}

pub unsafe extern "C" fn ifTime_LoadModels() -> bool {
    melee_core::ct!("ifTime_LoadModels");
    let mut scinftim_scene_models: *mut *mut DynamicModelDesc = null_mut();
    let archive = ext::ifAll_GetArchive();
    ext::lbArchive_LoadSections(
        *archive,
        (&raw mut scinftim_scene_models).cast(),
        c"ScInfTim_scene_models".as_ptr().cast(),
        &[((&raw mut ifTime_data.countdown_timer_models).cast(), c"tdsce".as_ptr().cast())],
    );
    if !(*scinftim_scene_models).is_null() {
        ifTime_match_timer_models.joint = (**scinftim_scene_models).joint;
        ifTime_match_timer_models.anims = (**scinftim_scene_models).anims;
        ifTime_match_timer_models.matanims = (**scinftim_scene_models).matanims;
        ifTime_match_timer_models.shapeanims = (**scinftim_scene_models).shapeanims;
    }
    !(*scinftim_scene_models).is_null()
}

pub unsafe extern "C" fn ifTime_SetDigit(jobj: *mut HSD_JObj, frame: u32) {
    melee_core::ct!("ifTime_SetDigit");
    let aobj = (*(*(*(*jobj).u.dobj).mobj).tobj).aobj;
    ext::HSD_AObjReqAnim(aobj, (frame & 0xFF) as f32);
    let aobj = (*(*(*(*jobj).u.dobj).mobj).tobj).aobj;
    ext::HSD_AObjSetRate(aobj, 0.0);
}

#[inline]
fn ones_digit(x: i32) -> u32 {
    (x % 10) as u32
}
#[inline]
fn tens_digit(x: i32) -> u32 {
    ((x % 100) / 10) as u32
}

pub unsafe extern "C" fn ifTime_SetTime(jobj: *mut HSD_JObj, seconds: i32, centiseconds: i32) {
    melee_core::ct!("ifTime_SetTime");
    let rules = ext::gm_GetStartMeleeRules();
    let hours: i32;

    // minutes
    if (*rules).timer_shows_hours() != 0 {
        ifTime_SetDigit(ifTime_data.digits[IFTIME_MINUTE_TEN], tens_digit((seconds / 60) % 60));
        hours = (seconds / 60) / 60;
    } else {
        ifTime_SetDigit(ifTime_data.digits[IFTIME_MINUTE_TEN], tens_digit(seconds / 60));
        hours = 0;
    }
    ifTime_SetDigit(ifTime_data.digits[IFTIME_MINUTE_ONE], ones_digit((seconds / 60) % 60));

    // seconds
    ifTime_SetDigit(ifTime_data.digits[IFTIME_SECOND_TEN], tens_digit(seconds % 60));
    ifTime_SetDigit(ifTime_data.digits[IFTIME_SECOND_ONE], ones_digit(seconds % 60));

    // hours
    if hours != 0 {
        let flags: u32 = !ext::HSD_JObjGetFlags(jobj) & JOBJ_HIDDEN as u32;
        ext::HSD_JObjClearFlags(ifTime_data.digits[IFTIME_HOUR_SEP], flags);
        ext::HSD_JObjClearFlags(ifTime_data.digits[IFTIME_HOUR_ONE], flags);
        ifTime_SetDigit(ifTime_data.digits[IFTIME_HOUR_ONE], ones_digit(hours));
        let hours_ten = tens_digit(hours) as i32;
        if hours_ten > 0 {
            ext::HSD_JObjClearFlags(ifTime_data.digits[IFTIME_HOUR_TEN], flags);
            ifTime_SetDigit(ifTime_data.digits[IFTIME_HOUR_TEN], hours_ten as u32);
        } else {
            ext::HSD_JObjSetFlags(ifTime_data.digits[IFTIME_HOUR_TEN], flags);
        }
    } else {
        ext::HSD_JObjSetFlags(ifTime_data.digits[IFTIME_HOUR_ONE], JOBJ_HIDDEN as u32);
        ext::HSD_JObjSetFlags(ifTime_data.digits[IFTIME_HOUR_TEN], JOBJ_HIDDEN as u32);
        ext::HSD_JObjSetFlags(ifTime_data.digits[IFTIME_HOUR_SEP], JOBJ_HIDDEN as u32);
    }

    // centiseconds
    ifTime_SetDigit(ifTime_data.digits[IFTIME_CENTISECOND_TEN], tens_digit(centiseconds));
    ifTime_SetDigit(ifTime_data.digits[IFTIME_CENTISECOND_ONE], ones_digit(centiseconds));
}

pub unsafe extern "C" fn ifTime_HideTimers() {
    melee_core::ct!("ifTime_HideTimers");
    let x = &raw mut ifTime_data;
    if !(*x).match_timer.is_null() {
        ext::HSD_JObjSetFlagsAll((*(*x).match_timer).hsd_obj.cast(), JOBJ_HIDDEN as u32);
    }
    if !(*x).countdown_timer.is_null() {
        ext::HSD_JObjSetFlagsAll((*(*x).countdown_timer).hsd_obj.cast(), JOBJ_HIDDEN as u32);
    }
}

pub unsafe extern "C" fn ifTime_ShowTimers() {
    melee_core::ct!("ifTime_ShowTimers");
    let x = &raw mut ifTime_data;
    if !(*x).match_timer.is_null() {
        let jobj: *mut HSD_JObj = (*(*x).match_timer).hsd_obj.cast();
        ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
        let seconds = ext::gm_8016AEEC() as i32;
        ifTime_SetTime(jobj, seconds, ext::gm_8016AF0C() as i32);
    }
    if !(*x).countdown_timer.is_null() {
        ext::HSD_JObjClearFlagsAll((*(*x).countdown_timer).hsd_obj.cast(), JOBJ_HIDDEN as u32);
    }
}

pub unsafe extern "C" fn ifTime_GetCountdownSeconds() -> i32 {
    melee_core::ct!("ifTime_GetCountdownSeconds");
    let centiseconds = ext::gm_8016AF0C() as i32;
    let mut seconds = ext::gm_8016AEEC() as i32;
    if centiseconds == 0 {
        seconds = 5 - seconds;
    } else {
        seconds = 4 - seconds;
    }
    if seconds < 0 {
        seconds = 0;
    }
    seconds
}

#[inline]
pub unsafe extern "C" fn ifTime_GetCountdownSeconds_dontinline() -> i32 {
    melee_core::ct!("ifTime_GetCountdownSeconds_dontinline");
    ifTime_GetCountdownSeconds()
}

/// The match timer and the countdown (the two `GObj_SetupGXLink` sites below): the Atlas retail
/// mask hides this element's render only (never its proc); empty by default.
pub unsafe extern "C" fn ifTime_RenderGuarded(gobj: *mut HSD_GObj, pass: isize) {
    melee_core::ct!("ifTime_RenderGuarded");
    if ext::Ui_RetailHidden(AT_RE_HUD_TIMER) != 0 {
        return;
    }
    ext::HSD_GObj_JObjCallback(gobj, pass);
}

pub unsafe extern "C" fn ifTime_UpdateCountdown(arg0: *mut HSD_GObj) {
    melee_core::ct!("ifTime_UpdateCountdown");
    let x = &raw mut ifTime_data;
    let jobj: *mut HSD_JObj = (*arg0).hsd_obj.cast();
    let f = ifTime_GetCountdownSeconds();
    if f != (*x).countdown_seconds as i32 {
        (*x).countdown_seconds = f as u8;
        ext::HSD_JObjRemoveAnimAll(jobj);
        ext::lb_8000C0E8(jobj, (*x).countdown_seconds as i32, *(*x).countdown_timer_models);
        ext::HSD_JObjReqAnimAll(jobj, 0.0);
        ext::HSD_JObjAnimAll(jobj);
    }
    ext::HSD_JObjAnimAll(jobj);
}

pub unsafe extern "C" fn ifTime_FreeCountdown() {
    melee_core::ct!("ifTime_FreeCountdown");
    let x = &raw mut ifTime_data;
    if !(*x).countdown_timer.is_null() {
        ext::HSD_GObjFree((*x).countdown_timer);
        (*x).countdown_timer = null_mut();
    }
}

pub unsafe extern "C" fn ifTime_UpdateTimers(arg0: *mut HSD_GObj) {
    melee_core::ct!("ifTime_UpdateTimers");
    let x = &raw mut ifTime_data;
    let jobj: *mut HSD_JObj = (*arg0).hsd_obj.cast();

    ext::gm_GetStartMeleeRules();
    let seconds = ext::gm_8016AEEC() as i32;
    let centiseconds = ext::gm_8016AF0C() as i32;
    ifTime_SetTime(jobj, seconds, centiseconds);
    if ext::gm_8016B110() == 0 && centiseconds == 0 && seconds == 5 {
        let jobj2 = ext::HSD_JObjLoadJoint((**(*x).countdown_timer_models).joint);
        if jobj2.is_null() {
            // OSReport("Error : jobj dont't get (ifAddTimeDownModel)"); OSPanic("iftime.c", 300, "")
            panic!("iftime.c:300 jobj get failed (ifAddTimeDownModel)");
        }
        let tmp = ext::HSD_GObj_JObjKind;
        ext::HSD_GObjObject_80390A70((*x).countdown_timer, tmp, jobj2.cast());
        ext::GObj_SetupGXLink((*x).countdown_timer, Some(ifTime_RenderGuarded), 11, 0);
        (*x).countdown_seconds = ifTime_GetCountdownSeconds_dontinline() as u8;
        ext::lb_8000C0E8(jobj2, (*x).countdown_seconds as i32, *(*x).countdown_timer_models);
        ext::HSD_JObjReqAnimAll(jobj2, 0.0);
        ext::HSD_JObjAnimAll(jobj2);
        HSD_JObjSetTranslate(jobj2, ext::ifAll_GetTimerPosition());
        ext::HSD_GObj_SetupProc((*x).countdown_timer, Some(ifTime_UpdateCountdown), 17);
        if !(*x).match_timer.is_null() {
            ext::HSD_GObjFree((*x).match_timer);
            (*x).match_timer = null_mut();
        }
    }
    ext::HSD_JObjAnimAll(jobj);
}

pub unsafe extern "C" fn ifTime_CreateTimers() {
    melee_core::ct!("ifTime_CreateTimers");
    let rules = ext::gm_GetStartMeleeRules();
    if (*rules).x1_0() == 0 && (*rules).timer_enabled() == 0 {
        ifTime_data.match_timer = null_mut();
        return;
    }
    if ifTime_LoadModels() {
        ifTime_data.countdown_timer = ext::GObj_Create(HSD_GOBJ_CLASS_UI as u16, 15, 0);
        if ifTime_data.countdown_timer.is_null() {
            // OSReport("Error : gobj dont't get (ifAddTime)"); OSPanic("iftime.c", 379, "")
            panic!("iftime.c:379 gobj get failed (ifAddTime)");
        }
        let gobj = ext::GObj_Create(HSD_GOBJ_CLASS_UI as u16, 16, 0);
        if gobj.is_null() {
            panic!("iftime.c:383 gobj get failed (ifAddTime)");
        }
        let jobj = ext::HSD_JObjLoadJoint(ifTime_match_timer_models.joint);
        if jobj.is_null() {
            panic!("iftime.c:389 jobj get failed (ifAddTime)");
        }
        ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj.cast());
        ext::GObj_SetupGXLink(gobj, Some(ifTime_RenderGuarded), 11, 0);
        let anims = ifTime_match_timer_models.anims;
        let matanims = ifTime_match_timer_models.matanims;
        let shapeanims = ifTime_match_timer_models.shapeanims;
        ext::lb_8000C07C(jobj, 0, anims, matanims, shapeanims);
        ext::HSD_JObjReqAnimAll(jobj, 0.0);
        ext::HSD_GObj_SetupProc(gobj, Some(ifTime_UpdateTimers), 17);
        HSD_JObjSetTranslate(jobj, ext::ifAll_GetTimerPosition());
        let mut digit = HSD_JObjGetChild(jobj);
        ifTime_data.digits[0] = digit;
        for i in 1..10usize {
            digit = HSD_JObjGetNext(digit);
            ifTime_data.digits[i] = digit;
        }
        ifTime_data.match_timer = gobj;
    }
}

pub unsafe extern "C" fn ifTime_Reset() {
    melee_core::ct!("ifTime_Reset");
    let x = &raw mut ifTime_data;
    ext::memzero(x.cast(), core::mem::size_of::<ifTime_data>() as i32);
}

pub unsafe extern "C" fn ifTime_FreeTimers() {
    melee_core::ct!("ifTime_FreeTimers");
    let x = &raw mut ifTime_data;
    if !(*x).match_timer.is_null() {
        ext::HSD_GObjFree((*x).match_timer);
        (*x).match_timer = null_mut();
    }
    if !(*x).countdown_timer.is_null() {
        ext::HSD_GObjFree((*x).countdown_timer);
        (*x).countdown_timer = null_mut();
    }
}

pub unsafe extern "C" fn ifTime_IsTimerHidden() -> bool {
    melee_core::ct!("ifTime_IsTimerHidden");
    let x = &raw mut ifTime_data;
    if !(*x).match_timer.is_null() && (ext::HSD_JObjGetFlags((*(*x).match_timer).hsd_obj.cast()) & JOBJ_HIDDEN as u32) != 0 {
        return true;
    }
    if !(*x).countdown_timer.is_null() && (ext::HSD_JObjGetFlags((*(*x).countdown_timer).hsd_obj.cast()) & JOBJ_HIDDEN as u32) != 0 {
        return true;
    }
    false
}
