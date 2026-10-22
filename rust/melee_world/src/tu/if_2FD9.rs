//! Port of src/melee/if/if_2FD9.c: the four "Stc_rarwmdls" arrow models of the results screen
//! (left / right arrows that mark which side each player's score belongs to). `un_802FD928`
//! registers a human player in one of four arrow rows, `un_802FDA78` is the per-arrow proc that
//! tints the model and places it (x = -25 / +25, y stacked 2.6 apart by rank, z hidden when
//! unused) and plays the cue when a side first gets an arrow. Callees of unported TUs go through
//! `ext`. Checked against the C (whole TU, shipping contraction) by tests/tu_if_2FD9.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_world/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            } )*
        };
    }
    ext_fns! {
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj;
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32);
    fn HSD_GObjFree(a0: *mut HSD_GObj);
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void);
    fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32);
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc;
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj;
    fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn Player_GetPlayerSlotType(a0: i32) -> Gm_PKind;
    fn gm_8016895C(a0: *mut HSD_JObj, a1: *mut DynamicModelDesc, a2: i32);
    fn gm_8016B124() -> i32;
    fn ifAll_GetArchive() -> *mut *mut HSD_Archive;
    fn lbArchive_LoadSections(a0: *mut HSD_Archive, a1: *mut *mut c_void, a2: *const u8, a3: i32);
    fn lbAudioAx_80024030(a0: i32);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut HSD_GObj_JObjKind: u8 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

/// `static struct un_804A1F10_t` (4A1F10)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct un_804A1F10_t {
    pub x0: *mut *mut DynamicModelDesc,
    pub x4: [*mut HSD_GObj; 4],
    pub x14: [_GXColor; 4],
    pub x24: [u8; 4],
    pub x28: [u8; 4],
    pub x2C: [u8; 4],
    pub x30: u8,
    pub x31: i8,
    pub x32: u8,
}

/// `static struct un_804A1F10_t un_804A1F10`
pub static mut un_804A1F10: un_804A1F10_t = unsafe { core::mem::zeroed() };
/// `struct un_804A1F48_t if_Scene_Prize_EnterData` (4A1F48)
pub static mut if_Scene_Prize_EnterData: un_804A1F48_t = unsafe { core::mem::zeroed() };
/// `static char const un_804DDBA8[4]` (4DDBA8): the arrows' slot numbers (their user data points here).
pub static un_804DDBA8: [u8; 4] = [0, 1, 2, 3];
/// `static int un_804D6D90` (4D6D90): nonzero hides the arrows.
pub static mut un_804D6D90: i32 = 0;

/// `HSD_ASSERT(line, cond)`: `__assert` does not return.
#[inline]
fn hsd_assert(cond: bool, what: &str) {
    if !cond {
        panic!("if_2FD9.c: HSD_ASSERT {what}");
    }
}

/// `HSD_JObjMtxIsDirty` (baselib/jobj.h, static inline).
#[inline]
pub unsafe fn HSD_JObjMtxIsDirty(jobj: *mut HSD_JObj) -> bool {
    hsd_assert(!jobj.is_null(), "(jobj)");
    (*jobj).flags & JOBJ_USER_DEF_MTX as u32 == 0 && (*jobj).flags & JOBJ_MTX_DIRTY as u32 != 0
}

/// `HSD_JObjSetMtxDirty` (macro)
#[inline]
unsafe fn HSD_JObjSetMtxDirty(jobj: *mut HSD_JObj) {
    if !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}

/// `HSD_JObjSetRotationZ` (baselib/jobj.h, static inline)
#[inline]
pub unsafe fn HSD_JObjSetRotationZ(jobj: *mut HSD_JObj, z: f32) {
    hsd_assert(!jobj.is_null(), "(jobj)");
    hsd_assert((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "!(jobj->flags & JOBJ_USE_QUATERNION)");
    (*jobj).rotate.z = z;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

/// `HSD_JObjSetTranslateX` (baselib/jobj.h, static inline)
#[inline]
pub unsafe fn HSD_JObjSetTranslateX(jobj: *mut HSD_JObj, x: f32) {
    hsd_assert(!jobj.is_null(), "(jobj)");
    (*jobj).translate.x = x;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

/// `HSD_JObjSetTranslateY` (baselib/jobj.h, static inline)
#[inline]
pub unsafe fn HSD_JObjSetTranslateY(jobj: *mut HSD_JObj, y: f32) {
    hsd_assert(!jobj.is_null(), "(jobj)");
    (*jobj).translate.y = y;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

/// `HSD_JObjSetTranslateZ` (baselib/jobj.h, static inline)
#[inline]
pub unsafe fn HSD_JObjSetTranslateZ(jobj: *mut HSD_JObj, z: f32) {
    hsd_assert(!jobj.is_null(), "(jobj)");
    (*jobj).translate.z = z;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

/// `HSD_GObjGetHSDObj` (baselib/gobj.h, static inline)
#[inline]
pub unsafe fn HSD_GObjGetHSDObj(gobj: *mut HSD_GObj) -> *mut c_void {
    (*gobj).hsd_obj
}

/// `void un_802FD910(void)`: hide the arrows.
pub unsafe extern "C" fn un_802FD910() {
    un_804D6D90 = 1;
}

/// `void un_802FD91C(void)`: show the arrows.
pub unsafe extern "C" fn un_802FD91C() {
    un_804D6D90 = 0;
}

/// `void un_802FD928(u8 slot, u8 arg1, GXColor* arg2)`: puts a human player's arrow into the first
/// free of four rows (side `arg1`, tint `*arg2`).
pub unsafe extern "C" fn un_802FD928(slot: u8, arg1: u8, arg2: *mut _GXColor) {
    if slot >= 4 {
        return;
    }
    for i in 0..4usize {
        let kind = ext::Player_GetPlayerSlotType(slot as i32);
        if kind == Gm_PKind_Cpu {
            continue;
        }
        if un_804A1F10.x28[i] != 0 {
            continue;
        }
        if un_804A1F10.x24[slot as usize] != 0 {
            continue;
        }
        un_804A1F10.x24[slot as usize] = 1;
        un_804A1F10.x2C[i] = slot;
        un_804A1F10.x28[i] = arg1;
        un_804A1F10.x14[i] = *arg2;
        return;
    }
}

/// `void un_802FD9D8(u8 slot)`: drops the player's arrow rows.
pub unsafe extern "C" fn un_802FD9D8(slot: u8) {
    for i in 0..4usize {
        if un_804A1F10.x2C[i] == slot {
            un_804A1F10.x24[slot as usize] = 0;
            un_804A1F10.x28[i] = 0;
        }
    }
}

/// `void fn_802FDA4C(HSD_GObj*, int pass)`: GX callback; draws the arrow unless they are hidden.
pub unsafe extern "C" fn fn_802FDA4C(gobj: *mut HSD_GObj, pass: i32) {
    if un_804D6D90 == 0 {
        ext::HSD_GObj_JObjCallback(gobj, pass);
    }
}

/// Tints the three materials under the arrow's joint with the slot's colour.
unsafe fn tint(jobj: *mut HSD_JObj, c: _GXColor, depth: usize) {
    let mut d = (*(*jobj).child).u.dobj;
    for _ in 0..depth {
        d = (*d).next;
    }
    let mat = (*(*d).mobj).mat;
    (*mat).diffuse.r = c.r;
    (*mat).diffuse.g = c.g;
    (*mat).diffuse.b = c.b;
    (*mat).diffuse.a = c.a;
}

/// `void fn_802FDA78(HSD_GObj*)`: per-arrow proc (user data points at its slot number). The
/// `2.6f * j + v[..]` is an f32 fused multiply-add (shipping contraction).
pub unsafe extern "C" fn fn_802FDA78(gobj: *mut HSD_GObj) {
    let p = &raw mut un_804A1F10;
    let v: [f32; 4] = [0.0, -1.3, -2.6, -3.9];
    let jobj = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;
    let pslot = (*gobj).user_data as *mut u8;
    let slot = *pslot;
    let mut j: u8 = 0;
    let mut k: u8 = 0;
    HSD_JObjSetTranslateZ(jobj, 0.0);
    let c = (*p).x14[slot as usize];
    tint(jobj, c, 0);
    tint(jobj, c, 1);
    tint(jobj, c, 2);
    if (*p).x28[0] == 0 && (*p).x28[1] == 0 && (*p).x28[2] == 0 && (*p).x28[3] == 0 {
        (*p).x31 = -1;
        (*p).x32 = 0;
    }
    match (*p).x28[slot as usize] {
        0 => HSD_JObjSetTranslateZ(jobj, 10000.0),
        side @ (1 | 2) => {
            if (*p).x31 == -1 {
                (*p).x31 = slot as i8;
            } else if (*p).x28[(*p).x31 as usize] == side {
                if (*p).x32 == 0 && (*p).x30 == 0 {
                    ext::lbAudioAx_80024030(4);
                }
                (*p).x32 = ((*p).x32 as i32 + 1).rem_euclid(51) as u8;
            }
            for i in 0..slot as usize {
                if (*p).x28[i] == side {
                    j += 1;
                }
            }
            for i in 0..4usize {
                if (*p).x28[i] == side {
                    k += 1;
                }
            }
            HSD_JObjSetRotationZ(jobj, if side == 1 { 0.0 } else { 3.1415927f64 as f32 });
            let idx = if k != 0 { k as usize - 1 } else { 0 };
            HSD_JObjSetTranslateY(jobj, fmadds(2.6f32, j as f32, v[idx]));
            HSD_JObjSetTranslateX(jobj, if side == 1 { -25.0 } else { 25.0 });
        }
        _ => {}
    }
    ext::HSD_JObjAnimAll(jobj);
}

/// `void un_802FE260(void)`: loads "Stc_rarwmdls" and builds the four arrow gobjs.
pub unsafe extern "C" fn un_802FE260() {
    let p = &raw mut un_804A1F10;
    un_804D6D90 = 0;
    ext::lbArchive_LoadSections(
        *ext::ifAll_GetArchive(),
        &raw mut (*p).x0 as *mut *mut c_void,
        b"Stc_rarwmdls\0".as_ptr(),
        0,
    );
    for i in 0..4usize {
        let gobj = ext::GObj_Create(HSD_GOBJ_CLASS_UI as u16, 15, 0);
        let jobj = ext::HSD_JObjLoadJoint((**(*p).x0).joint);
        (*gobj).user_data = &un_804DDBA8[i] as *const u8 as *mut c_void;
        ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
        ext::GObj_SetupGXLink(gobj, Some(fn_802FDA4C), 11, 0);
        ext::gm_8016895C(jobj, *(*p).x0, 0);
        ext::HSD_GObj_SetupProc(gobj, Some(fn_802FDA78), 17);
        ext::HSD_JObjReqAnimAll(jobj, 0.0);
        ext::HSD_JObjAnimAll(jobj);
        (*p).x24[i] = 0;
        (*p).x28[i] = 0;
        (*p).x4[i] = gobj;
    }
    (*p).x31 = -1;
    (*p).x32 = 0;
    (*p).x30 = ext::gm_8016B124() as u8;
}

/// `void un_802FE390(void)`: frees the arrow gobjs.
pub unsafe extern "C" fn un_802FE390() {
    for i in 0..4usize {
        if !un_804A1F10.x4[i].is_null() {
            ext::HSD_GObjFree(un_804A1F10.x4[i]);
            un_804A1F10.x4[i] = null_mut();
        }
    }
}
