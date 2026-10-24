//! Port of src/melee/lb/lb_0219.c (data-driven animated screen flashes: the LbBf.dat colour
//! animations played over the background flash overlay).
//! Checked against the C (whole TU, shipping contraction) by tests/tu_lb_0219.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use core::ffi::c_void;
use melee_runtime::cabi::f2u8;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_modes/gen/cref/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:path)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => { $crate::ext_dflt!([$($def)?] ($($a),*) $name) }
                }
            } )*
        };
    }
    ext_fns! {
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj;
    fn GObj_InitUserData(a0: *mut HSD_GObj, a1: u8, a2: Option<unsafe extern "C" fn(*mut c_void)>, a3: *mut c_void);
    fn HSD_GObjFree(a0: *mut HSD_GObj);
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc;
    fn HSD_ObjAlloc(a0: *mut _HSD_ObjAllocData) -> *mut c_void = crate::tu::adapt::HSD_ObjAlloc__394117;
    fn HSD_ObjAllocInit(a0: *mut _HSD_ObjAllocData, a1: u32, a2: u32) = crate::tu::adapt::HSD_ObjAllocInit__d50812;
    fn HSD_ObjFree(a0: *mut _HSD_ObjAllocData, a1: *mut c_void) = crate::tu::adapt::HSD_ObjFree__102255;
    fn fn_800208B0(a0: u8) = crate::tu::adapt::fn_800208B0__5b119b;
    fn lbArchive_LoadSymbols(a0: *const u8, a1: *mut c_void, a2: *const u8, a3: i32) -> *mut c_void;
    fn lbBgFlash_800208EC(a0: i32) = crate::tu::adapt::lbBgFlash_800208EC__c1a64f;
    fn lbBgFlash_InitState(a0: *mut _GXColor) = crate::tu::adapt::lbBgFlash_InitState__bb0850;
    fn lb_80014258(a0: *mut HSD_GObj, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut CommandInfo, i32)>) -> i32;
    fn lb_80014498(a0: *mut ColorOverlay) = crate::tu::adapt::lb_80014498__63b487;
    fn lb_800144C8(a0: *mut ColorOverlay, a1: *mut Fighter_804D653C_t, a2: i32, a3: i32) -> i32 = crate::tu::adapt::lb_800144C8__0c3a12;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// `struct BgFlashUserData` (0x84 on the GC)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BgFlashUserData {
    pub x0: u8,
    pub pad_01: [u8; 3],
    pub x4: ColorOverlay,
}
melee_core::gc_layout!(BgFlashUserData, "BgFlashUserData", size 0x84, align 4, [
    x0 @ 0x0: 1 "u8",
    pad_01 @ 0x1: 3 "u8[3]",
    x4 @ 0x4: 0x80 "ColorOverlay",
]);

pub static mut bgflash_alloc_data: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
pub static mut flash_gobj: *mut HSD_GObj = core::ptr::null_mut();
pub static mut lbl_804D63DC: *mut Fighter_804D653C_t = core::ptr::null_mut();
pub static mut flash_scale: f32 = 0.0;

/// `lbBgFlash_Free`: the user-data destructor.
pub unsafe extern "C" fn lbBgFlash_Free(arg0: *mut c_void) {
    ext::HSD_ObjFree(&raw mut bgflash_alloc_data, arg0);
}

pub unsafe fn lbBgFlash_SetFlashScale(arg8: f32) {
    flash_scale = arg8;
}

pub unsafe fn lbBgFlash_Init(arg0: i32) {
    ext::HSD_ObjAllocInit(&raw mut bgflash_alloc_data, core::mem::size_of::<BgFlashUserData>() as u32, 4);
    let gobj = ext::GObj_Create(0xE, 0xE, 0);
    if !gobj.is_null() {
        let user_data = ext::HSD_ObjAlloc(&raw mut bgflash_alloc_data) as *mut BgFlashUserData;
        if !user_data.is_null() {
            ext::GObj_InitUserData(gobj, 0xE, Some(lbBgFlash_Free), user_data as *mut c_void);
            flash_gobj = gobj;
            flash_scale = 1.0;
            (*user_data).x0 = arg0 as u8;
            ext::lbArchive_LoadSymbols(
                c"LbBf.dat".as_ptr() as *const u8,
                (&raw mut lbl_804D63DC) as *mut c_void,
                c"lbBgFlashColAnimData".as_ptr() as *const u8,
                0,
            );
            ext::lbBgFlash_800208EC(6);
            fn_80021C1C();
            ext::HSD_GObj_SetupProc(gobj, Some(lbBgFlash_Proc), 1);
            return;
        }
        ext::HSD_GObjFree(gobj);
    }
}

pub unsafe extern "C" fn lbBgFlash_Proc(gobj: *mut HSD_GObj) {
    let data = (*gobj).user_data as *mut BgFlashUserData;
    let was_active = (*data).x4.x7C_color_enable() as i32;
    let mut color: GXColor = core::mem::zeroed();

    fn_80021C80(gobj);
    if (*data).x4.x7C_color_enable() != 0 {
        let scale = flash_scale;
        color.r = f2u8(((*data).x4.x2C_hex.r as f32 * scale) as f64);
        color.g = f2u8(((*data).x4.x2C_hex.g as f32 * scale) as f64);
        color.b = f2u8(((*data).x4.x2C_hex.b as f32 * scale) as f64);
        color.a = (((*data).x4.x2C_hex.a as i32 * (*data).x0 as i32) / 255) as u8;
        ext::lbBgFlash_InitState(&raw mut color);
        return;
    }
    if was_active != 0 {
        ext::fn_800208B0(0);
    }
}

pub unsafe extern "C" fn fn_80021C18(_gobj: *mut HSD_GObj, _cmd: *mut CommandInfo, _arg2: i32) {}

pub unsafe fn fn_80021C1C() {
    let gobj = flash_gobj;
    let user_data = (*gobj).user_data as *mut BgFlashUserData;
    ext::lb_80014498(&raw mut (*user_data).x4);
}

pub unsafe fn lbBgFlash_80021C48(arg0: u32, arg1: u32) {
    let data = (*flash_gobj).user_data as *mut BgFlashUserData;
    ext::lb_800144C8(&raw mut (*data).x4, lbl_804D63DC, arg0 as i32, arg1 as i32);
}

pub unsafe fn fn_80021C80(gobj: *mut HSD_GObj) {
    let user_data = (*gobj).user_data as *mut BgFlashUserData;
    while ext::lb_80014258(gobj, &raw mut (*user_data).x4 as *mut c_void, Some(fn_80021C18)) != 0 {
        ext::lb_80014498(&raw mut (*user_data).x4);
    }
}
