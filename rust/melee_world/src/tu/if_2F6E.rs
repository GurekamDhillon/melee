//! Port of src/melee/if/if_2F6E.c: the VS result banner table (GAME! / Time Up / Sudden Death /
//! ...) and its loader. Every call out of the TU goes through `ext`.
//! Checked against the C (whole TU, shipping contraction) by tests/tu_if_2F6E.rs.
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
    fn HSD_GObjFree(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { melee_runtime::tu::gobj_native::HSD_GObjFree(a0 as _); } };
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void) = |a0: *mut HSD_GObj, a1: u8, a2: *mut c_void| { unsafe { melee_runtime::tu::gobj_native::HSD_GObjObject_80390A70(a0 as _, a1 as _, a2 as _); } };
    fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: isize) = |a0: *mut HSD_GObj, a1: isize| { unsafe { melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0 as _, a1 as _); } };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(a0 as _, a1 as _, a2 as _) as _ };
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj = |a0: *mut HSD_Joint| unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0 as _) as _ };
    fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
    fn gmVs_GetSceneController() -> *mut VsSceneController = || unsafe { crate::gm::gmVs_GetSceneController() as _ };
    fn ifAll_GetArchive() -> *mut *mut HSD_Archive = || unsafe { crate::tu::ifall::ifAll_GetArchive() as _ };
    fn if_802F73C4(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { crate::tu::if_2F72::if_802F73C4(a0 as _); } };
    fn if_802F74D0(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { crate::tu::if_2F72::if_802F74D0(a0 as _); } };
    fn lbArchive_LoadSections(a0: *mut HSD_Archive, a1: *mut c_void, a2: *const u8, rest: &[(*mut c_void, *const u8)]) = |a0: *mut HSD_Archive, a1: *mut c_void, a2: *const u8, rest: &[(*mut c_void, *const u8)]| { unsafe { let mut v: std::vec::Vec<*mut core::ffi::c_void> = std::vec![a1 as *mut core::ffi::c_void, a2 as *mut core::ffi::c_void]; for (s, n) in rest.iter() { v.push(*s as *mut core::ffi::c_void); v.push(*n as *mut core::ffi::c_void); } v.push(core::ptr::null_mut()); melee_runtime::tu::lbarchive::lbArchive_LoadSections(a0 as _, &v); } };
    fn lbAudioAx_800237A8(a0: i32, a1: i32, a2: i32) -> i32 = |a0: i32, a1: i32, a2: i32| unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800237A8(a0 as _, a1 as _, a2 as _) as _ };
    fn lb_8000C0E8(a0: *mut HSD_JObj, a1: i32, a2: *mut DynamicModelDesc) = |a0: *mut HSD_JObj, a1: i32, a2: *mut DynamicModelDesc| { unsafe { melee_runtime::tu::lb_00B0::lb_8000C0E8(a0 as _, a1 as _, a2 as _); } };
    fn lb_80011C18(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { melee_runtime::tu::lbspdisplay::lb_80011C18(a0 as _, a1 as _); } };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: isize) { super::HSD_GObj_JObjCallback(a0, a1) }
        pub unsafe extern "C" fn if_802F73C4(a0: *mut HSD_GObj) { super::if_802F73C4(a0) }
        pub unsafe extern "C" fn if_802F74D0(a0: *mut HSD_GObj) { super::if_802F74D0(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub use melee_runtime::tu::gobj_native::HSD_GObj_JObjKind;
}
// END GENERATED EXT

/// 3F9628: the result banner table (initial values from the C initializer; `x14` is filled by
/// `ifStatus_802F7134`).
pub static mut ifStatus_803F9628: [Element_803F9628; 8] = [
    elem(ext::cb::if_802F74D0, 0x7C860, 8),
    elem(ext::cb::if_802F73C4, 0xC351, 0),
    elem(ext::cb::if_802F74D0, 0x7C85E, 0),
    elem(ext::cb::if_802F73C4, 0x7C85D, 0),
    elem(ext::cb::if_802F73C4, 0x7C857, 10),
    elem(ext::cb::if_802F74D0, 0x7C855, 8),
    elem(ext::cb::if_802F74D0, 0x9C48, 0),
    elem(ext::cb::if_802F74D0, 0x9C46, 0),
];

const fn elem(x8: unsafe extern "C" fn(*mut HSD_GObj), xc: u32, x10: u8) -> Element_803F9628 {
    Element_803F9628 {
        x0: null_mut(),
        x4: 0,
        x8: Some(x8),
        xC: xc as i32,
        x10,
        x11: 0,
        x12: Element_803F9628_x12 { bf_0: [0] },
        x13: 0,
        x14: null_mut(),
        x18: None,
        x1C: None,
        x20: 0,
        x24: 0,
    }
}

pub unsafe extern "C" fn ifStatus_802F6EA4(arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: Event, arg5: Event) {
    melee_core::ct!("ifStatus_802F6EA4");
    if arg0 == 8 {
        if arg4.is_some() {
            (core::mem::transmute::<Event, IfStatusCb>(arg4).unwrap())(-1);
        }
        if arg5.is_some() {
            (core::mem::transmute::<Event, IfStatusCb>(arg5).unwrap())(-1);
        }
        if arg1 >= 0 {
            ext::lbAudioAx_800237A8(arg1, 0x7F, 0x40);
        }
        // the C tests arg1 again for the second sound
        if arg1 >= 0 {
            ext::lbAudioAx_800237A8(arg2, 0x7F, 0x40);
        }
    } else {
        let e: *mut Element_803F9628 = &raw mut ifStatus_803F9628[arg0 as usize];
        (*e).x20 = arg1 as i32;
        (*e).x24 = arg2 as i32;
        (*e).x11 = arg3 as u8;
        if !(*e).x0.is_null() {
            ext::HSD_GObjFree((*e).x0);
        }
        let gobj = ext::GObj_Create(0xE, 0xE, 0);
        let jobj = ext::HSD_JObjLoadJoint((*(*e).x14).joint);
        ext::lb_80011C18(jobj, 0x0800_0000);
        ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj.cast());
        ext::GObj_SetupGXLink(gobj, Some(ext::cb::HSD_GObj_JObjCallback), 0xB, 0);
        if (*e).x8.is_some() {
            ext::HSD_GObj_SetupProc(gobj, (*e).x8, 0);
        }
        ext::lb_8000C0E8(jobj, 0, (*e).x14);
        ext::HSD_JObjReqAnimAll(jobj, 0.0);
        ext::HSD_JObjAnimAll(jobj);
        (*e).x12.set_x0(0);
        (*e).x12.set_x1(0);
        (*e).x12.set_x2(0);
        (*e).x0 = gobj;
        (*e).x18 = core::mem::transmute::<Event, IfStatusCb>(arg4);
        (*e).x1C = core::mem::transmute::<Event, IfStatusCb>(arg5);
    }
}

pub unsafe extern "C" fn ifStatus_802F7034(arg0: *mut c_void) {
    melee_core::ct!("ifStatus_802F7034");
    let big_thing = ext::gmVs_GetSceneController();
    let a: i32 = if (*big_thing).state.unk_10 != 0 { (*big_thing).state.unk_10 - 1 } else { -1 };
    let b: i32 = if (*big_thing).state.unk_14 != 0 { (*big_thing).state.unk_14 - 1 } else { -1 };
    let c: i32 = if (*big_thing).state.unk_18 != 0 { (*big_thing).state.unk_18 as i32 } else { 0 };

    let arg0: Event = core::mem::transmute::<*mut c_void, Event>(arg0);
    let unk_b = (*big_thing).state.unk_B;
    if unk_b != 0 {
        ifStatus_802F6EA4(unk_b as i32 - 1, a, b, c, arg0, None);
        return;
    }
    let mr = (*big_thing).state.match_result;
    if mr as i32 == OUTCOME_TIMEOUT as i32 {
        ifStatus_802F6EA4(0, a, b, c, arg0, None);
        return;
    }
    if (*big_thing).start.x5_1() != 0 {
        if mr as i32 == OUTCOME_UNK_1P_BONUS_STAGE_END as i32 {
            ifStatus_802F6EA4(7, a, b, c, arg0, None);
            return;
        }
        ifStatus_802F6EA4(6, a, b, c, arg0, None);
        return;
    }
    ifStatus_802F6EA4(5, a, b, c, arg0, None);
}

pub unsafe extern "C" fn ifStatus_802F7134() {
    melee_core::ct!("ifStatus_802F7134");
    for i in 0..8usize {
        ifStatus_803F9628[i].x0 = null_mut();
        ifStatus_803F9628[i].x4 = 0;
    }

    let archive = ext::ifAll_GetArchive();
    let mut models: *mut *mut DynamicModelDesc = null_mut();
    ext::lbArchive_LoadSections(
        *archive,
        (&raw mut models).cast(),
        c"ScInfCnt_scene_models".as_ptr().cast(),
        &[],
    );

    for i in 0..8usize {
        ifStatus_803F9628[i].x14 = *models.add(i);
    }
}

/// free
pub unsafe extern "C" fn ifStatus_802F7220() {
    melee_core::ct!("ifStatus_802F7220");
    for i in 0..8usize {
        if !ifStatus_803F9628[i].x0.is_null() {
            ext::HSD_GObjFree(ifStatus_803F9628[i].x0);
            ifStatus_803F9628[i].x0 = null_mut();
        }
    }
}
