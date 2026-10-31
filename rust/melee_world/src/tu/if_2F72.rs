//! Port of src/melee/if/if_2F72.c: the "ScInfStc_scene_models" HUD animations. Two things live
//! here: the eight `ifStatus_803F9628` element gobjs (`if_802F73C4` / `if_802F74D0` run an element's
//! start / end callbacks around its joint animation, with `fn_802F7288` firing its two sound
//! cues), and the per-player KO-star models spawned from the archive's scene models
//! (`fn_802F77F8` builds one, `fn_802F75D4` / `fn_802F7670` / `fn_802F7994` are their procs, the
//! `if_802F7*` entry points spawn them by player slot). Callees of unported TUs go through `ext`.
//! Checked against the C (whole TU, shipping contraction) by tests/tu_if_2F72.rs.
//!
//! The C's `&slots[idx].gobj - 1` / `*++gobjp` pointer walks are plain `slots[idx].gobj` accesses
//! here (the element address is the same).
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
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32| { unsafe { melee_runtime::tu::gobj_native::GObj_SetupGXLink(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn HSD_GObjFree(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { melee_runtime::tu::gobj_native::HSD_GObjFree(a0 as _); } };
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void) = |a0: *mut HSD_GObj, a1: u8, a2: *mut c_void| { unsafe { melee_runtime::tu::gobj_native::HSD_GObjObject_80390A70(a0 as _, a1 as _, a2 as _); } };
    fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0 as _, a1 as _); } };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(a0 as _, a1 as _, a2 as _) as _ };
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj = |a0: *mut HSD_Joint| unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0 as _) as _ };
    fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) = |a0: *mut HSD_JObj| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetMtxDirtySub(a0 as _); } };
    fn Player_80036428(a0: i32) -> i32;
    fn gm_8016895C(a0: *mut HSD_JObj, a1: *mut DynamicModelDesc, a2: i32);
    fn gm_8016AEC8() -> i32 = || unsafe { crate::gm::gm_8016AEC8() as _ };
    fn ifAll_GetArchive() -> *mut *mut HSD_Archive = || unsafe { crate::tu::ifall::ifAll_GetArchive() as _ };
    fn ifAll_GetPlayerHUDPosition(a0: i32) -> *mut Vec = |a0: i32| unsafe { crate::tu::ifall::ifAll_GetPlayerHUDPosition(a0 as _) as _ };
    fn ifStatus_GetHUDInfo() -> *mut HudIndex = || unsafe { crate::tu::ifstatus::ifStatus_GetHUDInfo() as _ };
    fn lbArchive_LoadSections(a0: *mut HSD_Archive, a1: *mut *mut c_void, a2: *const u8, a3: i32) = |a0: *mut HSD_Archive, a1: *mut *mut c_void, a2: *const u8, a3: i32| { unsafe { let v: std::vec::Vec<*mut core::ffi::c_void> = std::vec![a1 as *mut core::ffi::c_void, a2 as *mut core::ffi::c_void, core::ptr::null_mut()]; melee_runtime::tu::lbarchive::lbArchive_LoadSections(a0 as _, &v); } };
    fn lbAudioAx_800237A8(a0: i32, a1: i32, a2: i32) -> i32 = |a0: i32, a1: i32, a2: i32| unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800237A8(a0 as _, a1 as _, a2 as _) as _ };
    fn lbGetJObjCurrFrame(a0: *mut HSD_JObj) -> f32 = |a0: *mut HSD_JObj| unsafe { melee_runtime::tu::lb_00B0::lbGetJObjCurrFrame(a0 as _) as _ };
    fn lb_8000B09C(a0: *mut HSD_JObj) -> i32 = |a0: *mut HSD_JObj| unsafe { melee_runtime::tu::lb_00B0::lb_8000B09C(a0 as _) as _ };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub use melee_runtime::tu::gobj_native::HSD_GObj_JObjKind;
    pub use crate::tu::if_2F6E::ifStatus_803F9628;
}
// END GENERATED EXT

/// `static char lbl_803F9780[]` (orphaned data string)
static lbl_803F9780: &[u8; 22] = b"ScInfStc_scene_models\0";

/// `struct ScInfStcSlot`: the gobjs one player slot has spawned.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ScInfStcSlot {
    /// spawned when a KO is recorded
    pub gobj: *mut HSD_GObj,
    /// spawned by `gobj`'s proc once it passes frame 12
    pub gobj2: *mut HSD_GObj,
}

/// `struct ScInfStcModels`: the archive's scene models plus the gobjs spawned from them.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ScInfStcModels {
    pub scene_models: *mut *mut DynamicModelDesc,
    pub slots: [ScInfStcSlot; 6],
}

/// `static struct ScInfStcModels scinfstc_models`
pub static mut scinfstc_models: ScInfStcModels = ScInfStcModels {
    scene_models: null_mut(),
    slots: [ScInfStcSlot { gobj: null_mut(), gobj2: null_mut() }; 6],
};

/// `HSD_ASSERT(line, cond)`: `__assert` does not return.
#[inline]
fn hsd_assert(cond: bool, what: &str) {
    if !cond {
        panic!("if_2F72.c: HSD_ASSERT {what}");
    }
}

/// `HSD_JObjMtxIsDirty` (baselib/jobj.h, static inline).
#[inline]
pub unsafe fn HSD_JObjMtxIsDirty(jobj: *mut HSD_JObj) -> bool {
    melee_core::ct!("HSD_JObjMtxIsDirty");
    hsd_assert(!jobj.is_null(), "(jobj)");
    (*jobj).flags & JOBJ_USER_DEF_MTX as u32 == 0 && (*jobj).flags & JOBJ_MTX_DIRTY as u32 != 0
}

/// `HSD_JObjSetTranslate` (baselib/jobj.h, static inline)
#[inline]
pub unsafe fn HSD_JObjSetTranslate(jobj: *mut HSD_JObj, translate: *mut Vec) {
    melee_core::ct!("HSD_JObjSetTranslate");
    hsd_assert(!jobj.is_null(), "(jobj)");
    hsd_assert(!translate.is_null(), "(translate)");
    (*jobj).translate = *translate;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 && !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}

/// `s32 fn_802F7288(HSD_GObj*, Element_803F9628*)`: animates the element's joint and fires its two
/// sound cues (`x10` / `x11` are the frames); returns whether the animation is still running.
pub unsafe extern "C" fn fn_802F7288(gobj: *mut HSD_GObj, entry: *mut Element_803F9628) -> i32 {
    melee_core::ct!("fn_802F7288");
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let frame = ext::lbGetJObjCurrFrame(jobj);

    ext::HSD_JObjAnimAll(jobj);

    if (*entry).x12.x0() == 0 {
        if frame >= (*entry).x10 as f32 {
            if (*entry).x20 as i32 >= 0 {
                ext::lbAudioAx_800237A8((*entry).x20 as i32, 127, 64);
                (*entry).x12.set_x0(1);
            } else if (*entry).xC as i32 >= 0 {
                ext::lbAudioAx_800237A8((*entry).xC as i32, 127, 64);
                (*entry).x12.set_x0(1);
            }
        }
    }

    if (*entry).x12.x1() == 0 {
        if frame >= (*entry).x11 as f32 {
            if (*entry).x24 as i32 >= 0 {
                ext::lbAudioAx_800237A8((*entry).x24 as i32, 127, 64);
                (*entry).x12.set_x1(1);
            }
        }
    }

    ext::lb_8000B09C(jobj)
}

/// The element of `ifStatus_803F9628` whose gobj is `gobj`, or NULL (the C's search loop).
unsafe fn find_entry(gobj: *mut HSD_GObj) -> *mut Element_803F9628 {
    let entries = (&raw mut ext::ifStatus_803F9628) as *mut Element_803F9628;
    for i in 0..8usize {
        if (*entries.add(i)).x0 == gobj {
            return entries.add(i);
        }
    }
    null_mut()
}

/// `void if_802F73C4(HSD_GObj*)`: element proc that frees the gobj when the animation ends.
pub unsafe extern "C" fn if_802F73C4(gobj: *mut HSD_GObj) {
    melee_core::ct!("if_802F73C4");
    let entry = find_entry(gobj);
    let idx = if entry.is_null() {
        0
    } else {
        (entry as usize - (&raw mut ext::ifStatus_803F9628) as usize) / core::mem::size_of::<Element_803F9628>()
    } as i32;
    if !entry.is_null() && (*entry).x12.x2() == 0 {
        if let Some(cb) = (*entry).x18 {
            cb(idx);
        }
        (*entry).x12.set_x2(1);
    }
    if !entry.is_null() {
        if fn_802F7288(gobj, entry) == 0 {
            if let Some(cb) = (*entry).x1C {
                cb(idx);
            }
            (*entry).x0 = null_mut();
            ext::HSD_GObjFree(gobj);
        }
    }
}

/// `void if_802F74D0(HSD_GObj*)`: element proc that keeps the gobj (it only clears the end callback).
pub unsafe extern "C" fn if_802F74D0(gobj: *mut HSD_GObj) {
    melee_core::ct!("if_802F74D0");
    let entry = find_entry(gobj);
    let idx = if entry.is_null() {
        0
    } else {
        (entry as usize - (&raw mut ext::ifStatus_803F9628) as usize) / core::mem::size_of::<Element_803F9628>()
    } as i32;
    if !entry.is_null() && (*entry).x12.x2() == 0 {
        if let Some(cb) = (*entry).x18 {
            cb(idx);
        }
        (*entry).x12.set_x2(1);
    }
    if !entry.is_null() {
        if fn_802F7288(gobj, entry) == 0 {
            if let Some(cb) = (*entry).x1C {
                cb(idx);
            }
            (*entry).x1C = None;
        }
    }
}

/// `void fn_802F75D4(HSD_GObj*)`: proc of a slot's first model; frees it when its animation ends.
pub unsafe extern "C" fn fn_802F75D4(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_802F75D4");
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    if ext::lb_8000B09C(jobj) == 0 {
        for i in 0..6 {
            if scinfstc_models.slots[i].gobj == gobj {
                scinfstc_models.slots[i].gobj = null_mut();
                break;
            }
        }
        ext::HSD_GObjFree(gobj);
    } else {
        ext::HSD_JObjAnimAll(jobj);
    }
}

/// `void fn_802F7670(HSD_GObj*)`: proc of a slot's second model.
pub unsafe extern "C" fn fn_802F7670(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_802F7670");
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    if ext::lb_8000B09C(jobj) == 0 {
        for i in 0..6 {
            if scinfstc_models.slots[i].gobj2 == gobj {
                scinfstc_models.slots[i].gobj2 = null_mut();
                break;
            }
        }
        ext::HSD_GObjFree(gobj);
    } else {
        ext::HSD_JObjAnimAll(jobj);
    }
}

/// `static inline s32 GetSlot(HSD_GObj*)`: the slot whose first gobj is `gobj`, or -1.
pub unsafe fn GetSlot(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("GetSlot");
    for i in 0..6 {
        if scinfstc_models.slots[i].gobj == gobj {
            return i as i32;
        }
    }
    -1
}

/// `void fn_802F770C(HSD_GObj*, int)`: GX callback; draws the model unless the slot's digits are hidden.
pub unsafe extern "C" fn fn_802F770C(gobj: *mut HSD_GObj, callback: i32) {
    melee_core::ct!("fn_802F770C");
    let status = ext::ifStatus_GetHUDInfo();
    let slot = GetSlot(gobj);
    if (*status).players[slot as usize].flags.hide_all_digits() == 0 {
        ext::HSD_GObj_JObjCallback(gobj, callback);
    }
}

/// `HSD_GObj* fn_802F77F8(HSD_GObj* gobj, u8 slot, u16 arg2)`: frees `gobj` and builds a model
/// (scene model 0, animation `arg2`) at the slot's HUD position; NULL when it cannot.
pub unsafe extern "C" fn fn_802F77F8(mut gobj: *mut HSD_GObj, slot: u8, arg2: u16) -> *mut HSD_GObj {
    melee_core::ct!("fn_802F77F8");
    if scinfstc_models.scene_models.is_null() {
        return null_mut();
    }
    if !gobj.is_null() {
        ext::HSD_GObjFree(gobj);
    }
    gobj = ext::GObj_Create(14, 15, 0);
    if !gobj.is_null() {
        let jobj = ext::HSD_JObjLoadJoint((**scinfstc_models.scene_models).joint);
        if !jobj.is_null() {
            ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
            ext::GObj_SetupGXLink(gobj, Some(fn_802F770C), 11, 0);
            ext::gm_8016895C(jobj, *scinfstc_models.scene_models, arg2 as u8 as i32);
            ext::HSD_JObjReqAnimAll(jobj, 0.0);
            ext::HSD_JObjAnimAll(jobj);

            let pos = ext::ifAll_GetPlayerHUDPosition(slot as i32);
            HSD_JObjSetTranslate(jobj, pos);
        } else {
            ext::HSD_GObjFree(gobj);
            gobj = null_mut();
        }
    }
    gobj
}

/// `void fn_802F7994(HSD_GObj*)`: proc of a slot's first model that also spawns the second one
/// once it passes frame 12.
pub unsafe extern "C" fn fn_802F7994(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_802F7994");
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let frame = ext::lbGetJObjCurrFrame(jobj);
    let slot = GetSlot(gobj);

    if slot >= 0 {
        if frame > 12.0 && scinfstc_models.slots[slot as usize].gobj2.is_null() {
            let idx = slot as u8 as usize;
            // the C's `*(gobjp += 2)` is the slot's second gobj (NULL here), and so is `*gobjp` after
            scinfstc_models.slots[idx].gobj2 = fn_802F77F8(scinfstc_models.slots[idx].gobj2, slot as u8, 1);
            if !scinfstc_models.slots[idx].gobj2.is_null() {
                ext::HSD_GObj_SetupProc(scinfstc_models.slots[idx].gobj2, Some(fn_802F7670), 0x11);
            }
        }
        if ext::lb_8000B09C(jobj) == 0 {
            scinfstc_models.slots[slot as usize].gobj = null_mut();
            ext::HSD_GObjFree(gobj);
        } else {
            ext::HSD_JObjAnimAll(jobj);
        }
    }
}

/// `static inline void SpawnGObj(s32 slot, u16 anim, HSD_GObjEvent proc)`
pub unsafe fn SpawnGObj(slot: i32, anim: u16, proc_: unsafe extern "C" fn(*mut HSD_GObj)) {
    melee_core::ct!("SpawnGObj");
    let idx = slot as u8 as usize;
    scinfstc_models.slots[idx].gobj = fn_802F77F8(scinfstc_models.slots[idx].gobj, idx as u8, anim);
    if !scinfstc_models.slots[idx].gobj.is_null() {
        ext::HSD_GObj_SetupProc(scinfstc_models.slots[idx].gobj, Some(proc_), 0x11);
    }
}

/// `void if_802F7AF8(s32 slot)`: spawns the model for `slot` (animation 1) and for the slot
/// `Player_80036428` names (animation 2).
pub unsafe extern "C" fn if_802F7AF8(slot: i32) {
    melee_core::ct!("if_802F7AF8");
    let slot2 = ext::Player_80036428(slot);

    let mut idx = slot as u8 as usize;
    let result = fn_802F77F8(scinfstc_models.slots[idx].gobj, slot as u8, 1);
    scinfstc_models.slots[idx].gobj = result;
    if !scinfstc_models.slots[idx].gobj.is_null() {
        ext::HSD_GObj_SetupProc(scinfstc_models.slots[idx].gobj, Some(fn_802F75D4), 0x11);
    }

    idx = slot2 as u8 as usize;
    let result = fn_802F77F8(scinfstc_models.slots[idx].gobj, slot2 as u8, 2);
    scinfstc_models.slots[idx].gobj = result;
    if !scinfstc_models.slots[idx].gobj.is_null() {
        ext::HSD_GObj_SetupProc(scinfstc_models.slots[idx].gobj, Some(fn_802F75D4), 0x11);
    }
}

/// `void if_802F7BB4(s32 player_idx)`
pub unsafe extern "C" fn if_802F7BB4(player_idx: i32) {
    melee_core::ct!("if_802F7BB4");
    let idx = player_idx as u8 as usize;
    scinfstc_models.slots[idx].gobj = fn_802F77F8(scinfstc_models.slots[idx].gobj, idx as u8, 1);
    if !scinfstc_models.slots[idx].gobj.is_null() {
        ext::HSD_GObj_SetupProc(scinfstc_models.slots[idx].gobj, Some(fn_802F75D4), 0x11);
    }
}

/// `void if_802F7C30(s32 slot)`: KO star for the slot, by `gm_8016AEC8()` (-2 first, -1 second).
pub unsafe extern "C" fn if_802F7C30(slot: i32) {
    melee_core::ct!("if_802F7C30");
    let ret = ext::gm_8016AEC8();
    if ret == -2 {
        let idx = slot as u8 as usize;
        scinfstc_models.slots[idx].gobj = fn_802F77F8(scinfstc_models.slots[idx].gobj, idx as u8, 0);
        if !scinfstc_models.slots[idx].gobj.is_null() {
            ext::HSD_GObj_SetupProc(scinfstc_models.slots[idx].gobj, Some(fn_802F75D4), 0x11);
        }
    } else if ret == -1 {
        SpawnGObj(slot, 1, fn_802F75D4);
    }
}

/// `void if_802F7D08(s32 slot)`: as `if_802F7C30`, with the animating proc `fn_802F7994`.
pub unsafe extern "C" fn if_802F7D08(slot: i32) {
    melee_core::ct!("if_802F7D08");
    let ret = ext::gm_8016AEC8();
    if ret == -2 {
        let idx = slot as u8 as usize;
        scinfstc_models.slots[idx].gobj = fn_802F77F8(scinfstc_models.slots[idx].gobj, idx as u8, 0);
        if !scinfstc_models.slots[idx].gobj.is_null() {
            ext::HSD_GObj_SetupProc(scinfstc_models.slots[idx].gobj, Some(fn_802F7994), 0x11);
        }
    } else if ret == -1 {
        SpawnGObj(slot, 1, fn_802F7994);
    } else {
        SpawnGObj(slot, 1, fn_802F75D4);
    }
}

/// `void if_802F7E24(void)`: clears the state and loads the archive's `ScInfStc_scene_models`.
pub unsafe extern "C" fn if_802F7E24() {
    melee_core::ct!("if_802F7E24");
    core::ptr::write_bytes(&raw mut scinfstc_models as *mut u8, 0, core::mem::size_of::<ScInfStcModels>());
    ext::lbArchive_LoadSections(
        *ext::ifAll_GetArchive(),
        &raw mut scinfstc_models.scene_models as *mut *mut c_void,
        lbl_803F9780.as_ptr(),
        0,
    );
}

/// `void if_802F7E7C(void)`: frees every spawned gobj and clears the state.
pub unsafe extern "C" fn if_802F7E7C() {
    melee_core::ct!("if_802F7E7C");
    for i in 0..6 {
        if !scinfstc_models.slots[i].gobj.is_null() {
            ext::HSD_GObjFree(scinfstc_models.slots[i].gobj);
        }
        if !scinfstc_models.slots[i].gobj2.is_null() {
            ext::HSD_GObjFree(scinfstc_models.slots[i].gobj2);
        }
    }
    core::ptr::write_bytes(&raw mut scinfstc_models as *mut u8, 0, core::mem::size_of::<ScInfStcModels>());
}
