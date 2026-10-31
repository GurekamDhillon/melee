//! Port of src/melee/if/if_2FF2.c: the results/"score" HUD pieces: one `HSD_Text` per player slot
//! showing the match-end score (`fn_802FF218` is the gobj proc that refreshes the number or blanks
//! it, `un_802FF364` rebuilds a slot's text), the HUD's point light and perspective camera
//! (`un_802FF6A0`, `un_802FF710`, from the static descriptor tables below) and the DevText overlay
//! setup (`un_802FF78C`). Callees of unported TUs go through `ext`; the two variadic text calls take
//! their `int` arguments as slices. Checked against the C (whole TU, shipping contraction) by
//! tests/tu_if_2FF2.rs, which also traces the descriptor tables' values through the load stubs.
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
    fn DevText_Setup(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: u8) -> *mut HSD_GObj = |a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: u8| unsafe { crate::tu::textdraw::DevText_Setup(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _) as _ };
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj = |a0: u16, a1: u8, a2: u8| unsafe { melee_runtime::tu::gobj_native::GObj_Create(a0 as _, a1 as _, a2 as _) as _ };
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32| { unsafe { melee_runtime::tu::gobj_native::GObj_SetupGXLink(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn GObj_SetupGXLinkMax(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32| { unsafe { melee_runtime::tu::gobj_native::GObj_SetupGXLinkMax(a0 as _, a1 as _, a2 as _); } };
    fn HSD_CObjLoadDesc(a0: *mut HSD_CObjDesc) -> *mut HSD_CObj = |a0: *mut HSD_CObjDesc| unsafe { melee_runtime::tu::cobj::HSD_CObjLoadDesc(a0 as _) as _ };
    fn HSD_GObjFree(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { melee_runtime::tu::gobj_native::HSD_GObjFree(a0 as _); } };
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void) = |a0: *mut HSD_GObj, a1: u8, a2: *mut c_void| { unsafe { melee_runtime::tu::gobj_native::HSD_GObjObject_80390A70(a0 as _, a1 as _, a2 as _); } };
    fn HSD_GObj_803910D8(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { melee_runtime::tu::gobj_native::HSD_GObj_803910D8(a0 as _, a1 as _); } };
    fn HSD_GObj_LObjCallback(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { melee_runtime::tu::gobj::HSD_GObj_LObjCallback(a0 as _, a1 as _); } };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(a0 as _, a1 as _, a2 as _) as _ };
    fn HSD_LObjLoadDesc(a0: *mut HSD_LightDesc) -> *mut HSD_LObj = |a0: *mut HSD_LightDesc| unsafe { melee_runtime::tu::lobj::HSD_LObjLoadDesc(a0 as _) as _ };
    fn HSD_SisLib_803A5CC4(a0: *mut HSD_Text) = |a0: *mut HSD_Text| { unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A5CC4(a0 as _); } };
    fn HSD_SisLib_803A611C(a0: i32, a1: *mut HSD_GObj, a2: u16, a3: u8, a4: u8, a5: u8, a6: u8, a7: u32) -> i32 = |a0: i32, a1: *mut HSD_GObj, a2: u16, a3: u8, a4: u8, a5: u8, a6: u8, a7: u32| unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A611C(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _, a7 as _) as _ };
    fn HSD_SisLib_803A6754(a0: i32, a1: i32) -> *mut HSD_Text = |a0: i32, a1: i32| unsafe { melee_runtime::tu::hsd_3A64::HSD_SisLib_803A6754(a0 as _, a1 as _) as _ };
    fn HSD_SisLib_803A6B98(a0: *mut HSD_Text, a1: f32, a2: f32, a3: *const i8, a4: &[i32]) -> i32 = |a0: *mut HSD_Text, a1: f32, a2: f32, a3: *const i8, a4: &[i32]| { unsafe { let args: std::vec::Vec<melee_runtime::cformat::PrintfArg> = a4.iter().map(|x| melee_runtime::cformat::PrintfArg::Int(*x as i32)).collect(); let s = String::from_utf8_lossy(&melee_runtime::cformat::c_format(a3, &args)).into_owned(); melee_runtime::tu::hsd_3A64::HSD_SisLib_803A6B98(a0, a1, a2, if a3.is_null() { None } else { Some(&s) }) } };
    fn HSD_SisLib_803A70A0(a0: *mut HSD_Text, a1: i32, a2: *const i8, a3: &[i32]) -> i32 = |a0: *mut HSD_Text, a1: i32, a2: *const i8, a3: &[i32]| { unsafe { let args: std::vec::Vec<melee_runtime::cformat::PrintfArg> = a3.iter().map(|x| melee_runtime::cformat::PrintfArg::Int(*x as i32)).collect(); let s = String::from_utf8_lossy(&melee_runtime::cformat::c_format(a2, &args)).into_owned(); melee_runtime::tu::hsd_3A64::HSD_SisLib_803A70A0(a0, a1, if a2.is_null() { None } else { Some(&s) }) } };
    fn HSD_SisLib_803A7548(a0: *mut HSD_Text, a1: i32, a2: f32, a3: f32) = |a0: *mut HSD_Text, a1: i32, a2: f32, a3: f32| { unsafe { melee_runtime::tu::hsd_3A64::HSD_SisLib_803A7548(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn gm_8016B774() -> *mut MatchEnd = || unsafe { crate::gm::gm_8016B774() as _ };
    fn gm_GetMatchEndPlayerScore(a0: i32) -> i32;
    fn ifAll_GetHUDGObj() -> *mut HSD_GObj = || unsafe { crate::tu::ifall::ifAll_GetHUDGObj() as _ };
    fn ifAll_GetPlayerHUDPosition(a0: i32) -> *mut Vec = |a0: i32| unsafe { crate::tu::ifall::ifAll_GetPlayerHUDPosition(a0 as _) as _ };
    fn un_80304138() = || { unsafe { crate::tu::textlib_1::un_80304138(); } };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_GObj_803910D8(a0: *mut HSD_GObj, a1: i32) { super::HSD_GObj_803910D8(a0, a1) }
        pub unsafe extern "C" fn HSD_GObj_LObjCallback(a0: *mut HSD_GObj, a1: i32) { super::HSD_GObj_LObjCallback(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
    pub use melee_runtime::tu::gobj_native::HSD_GObj_CameraKind;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_LightKind;
}
// END GENERATED EXT

/// `static HSD_WObjDesc eyepos` (3F9E38)
pub static mut eyepos: HSD_WObjDesc = HSD_WObjDesc {
    class_name: null_mut(),
    pos: Vec { x: 0.0, y: 40.241425, z: 100.24099 },
    robjdesc: null_mut(),
};
/// `static HSD_WObjDesc interest`
pub static mut interest: HSD_WObjDesc = HSD_WObjDesc {
    class_name: null_mut(),
    pos: Vec { x: 0.0, y: 10.0, z: 0.0 },
    robjdesc: null_mut(),
};
/// `static HSD_CameraDescPerspective un_803F9E60` (3F9E60)
pub static mut un_803F9E60: HSD_CameraDescPerspective = HSD_CameraDescPerspective {
    class_name: null_mut(),
    flags: 0,
    projection_type: 1,
    viewport: _HSD_RectS16 { xmin: 0, xmax: 640, ymin: 0, ymax: 480 },
    scissor: _Scissor { left: 0, right: 640, top: 0, bottom: 480 },
    eyepos: &raw mut eyepos,
    interest: &raw mut interest,
    roll: 0.0,
    up_vector: null_mut(),
    nnear: 0.1,
    ffar: 32.0 * 1024.0,
    fov: 30.0,
    aspect: 1.18,
};
/// `static HSD_WObjDesc light_position` (3F9E98)
pub static mut light_position: HSD_WObjDesc = HSD_WObjDesc {
    class_name: null_mut(),
    pos: Vec { x: 6.360198974609375, y: 14.943239, z: 59.982383728027344 },
    robjdesc: null_mut(),
};
/// `static HSD_LightPointDesc light_point` (3F9EAC)
pub static mut light_point: HSD_LightPointDesc = HSD_LightPointDesc { ref_br: 0.99, ref_dist: 10000.0, dist_func: GX_DA_STEEP as u32 };
/// `static HSD_LightDesc light0` (3F9EB8)
pub static mut light0: HSD_LightDesc = HSD_LightDesc {
    class_name: null_mut(),
    next: null_mut(),
    flags: 1 << 3,
    attnflags: 0,
    color: _GXColor { r: 0xFF, g: 0xFF, b: 0xFF, a: 0xFF },
    position: &raw mut light_position,
    interest: null_mut(),
    u: HSD_LightDesc_u { point: &raw mut light_point },
};
/// `static HSD_LightDesc light1` (3F9ED4)
pub static mut light1: HSD_LightDesc = HSD_LightDesc {
    class_name: null_mut(),
    next: &raw mut light0,
    flags: 1 << 0,
    attnflags: 0,
    color: _GXColor { r: 0xFF, g: 0xFF, b: 0xFF, a: 0xFF },
    position: null_mut(),
    interest: null_mut(),
    u: HSD_LightDesc_u { p: null_mut() },
};

/// `struct un_804A1F58_x8_t`: one player's text.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct un_804A1F58_x8_t {
    pub x0: *mut HSD_GObj,
    pub x4: *mut HSD_Text,
    pub x8: u32,
    pub xC: u32,
    pub x10: u8,
}

/// `static struct un_804A1F58_t un_804A1F58` (4A1F58)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct un_804A1F58_t {
    pub x0: u32,
    pub pad_x4: [u8; 4],
    pub x8: [un_804A1F58_x8_t; 6],
}

pub static mut un_804A1F58: un_804A1F58_t = unsafe { core::mem::zeroed() };

/// `static inline int fn_802FF218_inline(HSD_GObj*)`: the player index whose gobj this is, or -1.
pub unsafe fn fn_802FF218_inline(arg0: *mut HSD_GObj) -> i32 {
    melee_core::ct!("fn_802FF218_inline");
    for x in 0..6usize {
        if un_804A1F58.x8[x].x0 == arg0 {
            return x as i32;
        }
    }
    -1
}

/// `void fn_802FF218(HSD_GObj*)`: refreshes the player's number (blank while `x10 == 1`).
pub unsafe extern "C" fn fn_802FF218(arg0: *mut HSD_GObj) {
    melee_core::ct!("fn_802FF218");
    let y = fn_802FF218_inline(arg0);
    if y >= 0 {
        let thing = &raw mut un_804A1F58.x8[y as usize];
        if (*thing).x10 == 1 {
            ext::HSD_SisLib_803A70A0((*thing).x4, (*thing).x8 as i32, b"  \0".as_ptr() as *const i8, &[]);
        } else {
            ext::gm_8016B774();
            let mut s = ext::gm_GetMatchEndPlayerScore(y);
            if s > 9999 {
                s = 9999;
            }
            let tmp = s;
            if (*thing).xC != s as u32 {
                ext::HSD_SisLib_803A70A0((*thing).x4, (*thing).x8 as i32, b"%d\0".as_ptr() as *const i8, &[tmp]);
                (*thing).xC = s as u32;
            }
        }
    }
}

/// `void fn_802FF360(void*)`: the texts' render callback (does nothing).
pub unsafe extern "C" fn fn_802FF360(_arg0: *mut c_void) { melee_core::ct!("fn_802FF360");}

/// `void un_802FF364(int slot)`: (re)builds the slot's score text at its HUD position.
pub unsafe extern "C" fn un_802FF364(slot: i32) {
    melee_core::ct!("un_802FF364");
    let base = &raw mut un_804A1F58;
    let thing = &raw mut (*base).x8[slot as usize];
    let if_all = ext::ifAll_GetPlayerHUDPosition(slot);
    let gobj = (*thing).x0;
    if !gobj.is_null() {
        ext::HSD_GObjFree(gobj);
    }
    if !(*thing).x4.is_null() {
        ext::HSD_SisLib_803A5CC4((*thing).x4);
    }
    (*thing).x4 = ext::HSD_SisLib_803A6754(2, (*base).x0 as i32);
    (*(*thing).x4).default_alignment = 1;
    (*(*thing).x4).default_kerning = 1;
    ext::gm_8016B774();
    let mut s = ext::gm_GetMatchEndPlayerScore(slot);
    if s > 9999 {
        s = 9999;
    }
    (*thing).x8 = ext::HSD_SisLib_803A6B98((*thing).x4, (*if_all).x, 3.2f32 + (*if_all).y, b"%d\0".as_ptr() as *const i8, &[s]) as u32;
    ext::HSD_SisLib_803A7548((*thing).x4, (*thing).x8 as i32, 0.06, 0.06);
    (*(*thing).x4).render_callback = Some(fn_802FF360);
    (*thing).x0 = ext::GObj_Create(HSD_GOBJ_CLASS_UI as u16, 15, 0);
    ext::HSD_GObj_SetupProc((*thing).x0, Some(fn_802FF218), 17);
}

/// `void un_802FF498(void)`: clears the state and creates the SisLib text list.
pub unsafe extern "C" fn un_802FF498() {
    melee_core::ct!("un_802FF498");
    core::ptr::write_bytes(&raw mut un_804A1F58 as *mut u8, 0, core::mem::size_of::<un_804A1F58_t>());
    un_804A1F58.x0 = ext::HSD_SisLib_803A611C(2, ext::ifAll_GetHUDGObj(), 14, 15, 0, 11, 0, 19) as u32;
}

/// `void un_802FF4FC(void)`: frees every player's gobj and text.
pub unsafe extern "C" fn un_802FF4FC() {
    melee_core::ct!("un_802FF4FC");
    for i in 0..6usize {
        let thing = &raw mut un_804A1F58.x8[i];
        if !(*thing).x0.is_null() {
            ext::HSD_GObjFree((*thing).x0);
        }
        if !(*thing).x4.is_null() {
            ext::HSD_SisLib_803A5CC4((*thing).x4);
        }
    }
}

/// `void un_802FF570(void)`: hides every text (and blanks the numbers).
pub unsafe extern "C" fn un_802FF570() {
    melee_core::ct!("un_802FF570");
    for i in 0..6usize {
        let thing = &raw mut un_804A1F58.x8[i];
        (*thing).x10 = 1;
        let text = (*thing).x4;
        if !text.is_null() {
            (*text).hidden = 1;
        }
    }
}

/// `void un_802FF620(void)`: rebuilds and shows the texts that exist.
pub unsafe extern "C" fn un_802FF620() {
    melee_core::ct!("un_802FF620");
    for i in 0..6usize {
        let thing = &raw mut un_804A1F58.x8[i];
        (*thing).x10 = 0;
        if !(*thing).x4.is_null() {
            un_802FF364(i as i32);
            (*(*thing).x4).hidden = 0;
        }
    }
}

/// `void un_802FF6A0(void)`: the HUD light gobj.
pub unsafe extern "C" fn un_802FF6A0() {
    melee_core::ct!("un_802FF6A0");
    let gobj = ext::GObj_Create(HSD_GOBJ_CLASS_LIGHT as u16, 3, 0);
    let new_var = ext::HSD_LObjLoadDesc(&raw mut light1);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_LightKind as u8, new_var as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::cb::HSD_GObj_LObjCallback), 0, 0);
}

/// `void un_802FF710(void)`: the HUD camera gobj.
pub unsafe extern "C" fn un_802FF710() {
    melee_core::ct!("un_802FF710");
    let gobj = ext::GObj_Create(0x13, 20, 0);
    if !gobj.is_null() {
        let new_var = ext::HSD_CObjLoadDesc(&raw mut un_803F9E60 as *mut HSD_CObjDesc);
        ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_CameraKind, new_var as *mut c_void);
        ext::GObj_SetupGXLinkMax(gobj, Some(ext::cb::HSD_GObj_803910D8), 11);
        (*gobj).gxlink_prios = 0x20000;
    }
}

/// `HSD_GObj* un_802FF78C(void)`: sets the DevText overlay up (and its menu allocator).
pub unsafe extern "C" fn un_802FF78C() -> *mut HSD_GObj {
    melee_core::ct!("un_802FF78C");
    let gobj = ext::DevText_Setup(21, 24, 0, 17, 0, 11);
    if !gobj.is_null() {
        ext::un_80304138();
    }
    gobj
}
