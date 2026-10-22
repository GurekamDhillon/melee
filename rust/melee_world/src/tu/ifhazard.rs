//! Port of src/melee/if/ifhazard.c (built `TARGET_PC`): the stage-hazard HUD icon (Onett car /
//! Mute City / Big Blue warning). Every call out of the TU goes through `ext`.
//! Checked against the C (whole TU, shipping contraction) by tests/tu_ifhazard.rs.
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
        ($name:ident, ($($a:ident),*)) => { panic!(concat!("TODO: ", stringify!($name), " is not ported")) };
        ($name:ident, ($($a:ident),*), $def:path) => { $def($($a),*) };
    }
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:path)?;)*) => {
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
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj;
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32);
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void);
    fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32);
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc;
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj;
    fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn Ui_RetailHidden(a0: i32) -> i32;
    fn gm_8016895C(a0: *mut HSD_JObj, a1: *mut DynamicModelDesc, a2: i32);
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

crate::jobj_inlines!();

/// `AT_RE_HUD_HAZARD` (pc/platform/gw_ui_retail_ids.h).
const AT_RE_HUD_HAZARD: i32 = 7;

/// 4D6D80 .sbss
pub static mut un_804D6D80: *mut HSD_GObj = null_mut();
/// 4D6D84 .sbss
pub static mut un_804D6D84: i32 = 0;
/// 4D6D88 .sbss
pub static mut un_804D6D88: i32 = 0;
/// 4D6D8C .sbss
pub static mut un_804D6D8C: i32 = 0;

/// `HSD_GObjGetHSDObj` (baselib/gobj.h, static inline).
#[inline]
unsafe fn HSD_GObjGetHSDObj(gobj: *mut HSD_GObj) -> *mut c_void {
    (*gobj).hsd_obj
}

pub unsafe extern "C" fn un_802FD604(arg: i32) {
    let jobj = HSD_GObjGetHSDObj(un_804D6D80) as *mut HSD_JObj;
    ext::HSD_JObjReqAnimAll(jobj, 0.0);
    ext::HSD_JObjAnimAll(jobj);
    un_804D6D84 = arg;
    un_804D6D88 = 1;
}

pub unsafe extern "C" fn un_802FD65C() {
    un_804D6D88 = 0;
}

pub unsafe extern "C" fn un_802FD668() {
    un_804D6D8C = 1;
}

pub unsafe extern "C" fn un_802FD674() {
    un_804D6D8C = 0;
}

pub unsafe extern "C" fn fn_802FD680(gobj: *mut HSD_GObj) {
    let jobj = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;
    if un_804D6D84 != 0 {
        un_804D6D84 -= 1;
        if un_804D6D84 == 0 {
            un_804D6D88 = 0;
        }
    }
    ext::HSD_JObjAnimAll(jobj);
}

pub unsafe extern "C" fn fn_802FD6CC(gobj: *mut HSD_GObj, pass: i32) {
    // TARGET_PC: the Atlas retail mask (empty by default).
    if ext::Ui_RetailHidden(AT_RE_HUD_HAZARD) != 0 {
        return;
    }
    if un_804D6D8C == 0 && un_804D6D88 != 0 {
        ext::HSD_GObj_JObjCallback(gobj, pass);
    }
}

pub unsafe extern "C" fn un_802FD704() {}

pub unsafe extern "C" fn un_802FD708(desc: *mut DynamicModelDesc, arg1: i32) {
    let gobj = ext::GObj_Create(HSD_GOBJ_CLASS_UI as u16, 15, 0);
    let jobj = ext::HSD_JObjLoadJoint((*desc).joint);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj.cast());
    ext::GObj_SetupGXLink(gobj, Some(fn_802FD6CC), 11, 0);
    ext::gm_8016895C(jobj, desc, 0);
    ext::HSD_GObj_SetupProc(gobj, Some(fn_802FD680), 17);
    un_804D6D88 = 0;
    un_804D6D8C = 0;
    un_804D6D80 = gobj;
    match arg1 {
        0 => {
            // Onett
            let mut v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            HSD_JObjGetTranslation(jobj, &mut v);
            v.x += 18.0;
            HSD_JObjSetTranslate(jobj, &mut v);
        }
        // 1: Mute City, 2: Big Blue
        _ => {}
    }
}

pub unsafe extern "C" fn un_802FD8A0(desc: *mut DynamicModelDesc) {
    un_802FD708(desc, 0);
}

pub unsafe extern "C" fn un_802FD8C4(desc: *mut DynamicModelDesc) {
    un_802FD708(desc, 1);
}

pub unsafe extern "C" fn un_802FD8E8(desc: *mut DynamicModelDesc) {
    un_802FD708(desc, 2);
}

pub unsafe extern "C" fn un_802FD90C() {}
