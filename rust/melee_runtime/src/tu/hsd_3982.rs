//! Port of src/sysdolphin/baselib/hsd_3982.c: the debug overlay's own camera GObj (an ortho
//! camera typed in the C as the frustum arm of `HSD_CObjDesc`), whose render callback runs the
//! hsd_392A sampler and the hsd_3924 overlay.
//!
//! Callees in unported TUs are TODO stubs in [`ext`], named like the C functions.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

use core::ffi::c_void;
use core::ptr::{addr_of_mut, null_mut};
use melee_types::*;

pub mod ext {
    #![allow(non_snake_case, unused_variables)]
    use super::*;
    /// `HSD_GObj_CameraKind` (gobj.c: assigned at `HSD_GObjInit`; the test harness sets it).
    pub static mut HSD_GObj_CameraKind: u8 = 0;
    pub fn HSD_CObjSetCurrent(cobj: *mut HSD_CObj) { unsafe { { let _ = crate::tu::cobj::HSD_CObjSetCurrent(cobj); } } }
    pub fn fn_80392934() { todo!("fn_80392934 (baselib/hsd_392A.c)") }
    pub fn hsd_8039254C() { todo!("hsd_8039254C (baselib/hsd_3924.c)") }
    pub fn GObj_Create(_class_id: u16, _p_link: u8, _obj_kind: u8) -> *mut HSD_GObj { todo!("GObj_Create (baselib/gobjplink.c)") }
    pub fn HSD_CObjLoadDesc(desc: *mut HSD_CObjDesc) -> *mut HSD_CObj { unsafe { crate::tu::cobj::HSD_CObjLoadDesc(desc) } }
    pub fn HSD_GObjObject_80390A70(_gobj: *mut HSD_GObj, _kind: u8, _obj: *mut c_void) { todo!("HSD_GObjObject_80390A70 (baselib/gobjobject.c)") }
    pub fn GObj_SetupGXLinkMax(_gobj: *mut HSD_GObj, _cb: GObj_RenderFunc, _gx_link: u32) { todo!("GObj_SetupGXLinkMax (baselib/gobjgxlink.c)") }
    pub fn hsd_80392528(_event: Event) { todo!("hsd_80392528 (baselib/hsd_3924.c)") }
    pub fn fn_80392A3C() { todo!("fn_80392A3C (baselib/hsd_392A.c)") }
    pub fn fn_80392A08(_a: i32, _b: i32, _c: i32) { todo!("fn_80392A08 (baselib/hsd_392A.c)") }
    /// Address stored as the `Event` callback of the overlay: C-ABI shim over the hook.
    pub unsafe extern "C" fn fn_80392A3C_cb() { fn_80392A3C() }
}

/// `lbl_8040BF48`
static mut lbl_8040BF48: HSD_WObjDesc =
    HSD_WObjDesc { class_name: null_mut(), pos: Vec { x: 0.0, y: 0.0, z: 1.0 }, robjdesc: null_mut() };
/// `lbl_8040BF5C`
static mut lbl_8040BF5C: HSD_WObjDesc =
    HSD_WObjDesc { class_name: null_mut(), pos: Vec { x: 0.0, y: 0.0, z: 0.0 }, robjdesc: null_mut() };
/// `lbl_8040BF70`: ortho camera, typed as the frustum arm of the `HSD_CObjDesc` union.
static mut lbl_8040BF70: HSD_CameraDescFrustum = HSD_CameraDescFrustum {
    class_name: null_mut(),
    flags: 0,
    projection_type: 3,
    viewport: _HSD_RectS16 { xmin: 0, xmax: 640, ymin: 0, ymax: 480 },
    scissor: _Scissor { left: 0, right: 640, top: 0, bottom: 480 },
    eyepos: null_mut(),
    interest: null_mut(),
    roll: 0.0,
    up_vector: null_mut(),
    nnear: 0.0,
    ffar: 32768.0,
    top: -445.0,
    bottom: 35.0,
    left: -20.0,
    right: 620.0,
};

/// `fn_803982E4` (0x803982E4): render callback of the overlay camera GObj.
pub unsafe extern "C" fn fn_803982E4(gobj: *mut HSD_GObj, _unused: i32) {
    melee_core::ct!("fn_803982E4");
    ext::HSD_CObjSetCurrent((*gobj).hsd_obj as *mut HSD_CObj);
    ext::fn_80392934();
    ext::hsd_8039254C();
}

/// `hsd_80398310` (0x80398310): create the overlay camera GObj and register the overlay event.
pub unsafe fn hsd_80398310(class_id: u16, p_link: u8, obj_kind: u8, gx_link: u32) -> *mut HSD_GObj {
    melee_core::ct!("hsd_80398310");
    let gobj = ext::GObj_Create(class_id, p_link, obj_kind);
    if gobj.is_null() {
        return null_mut();
    }
    // The statics point at each other, which a Rust initializer cannot express.
    lbl_8040BF70.eyepos = addr_of_mut!(lbl_8040BF48);
    lbl_8040BF70.interest = addr_of_mut!(lbl_8040BF5C);
    let cobj = ext::HSD_CObjLoadDesc(addr_of_mut!(lbl_8040BF70) as *mut HSD_CObjDesc);
    ext::HSD_GObjObject_80390A70(gobj, unsafe { ext::HSD_GObj_CameraKind }, cobj as *mut c_void);
    ext::GObj_SetupGXLinkMax(gobj, Some(fn_803982E4), gx_link);
    ext::hsd_80392528(Some(ext::fn_80392A3C_cb));
    ext::fn_80392A08(4, 1, 0);
    gobj
}
