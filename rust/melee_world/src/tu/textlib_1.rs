//! Port of src/melee/if/textlib_1.c: the DevText menu widget (the sound test's rows): per-row
//! callbacks (`x4` of an entry, `xC` of the menu), drawing a row table into a text box, stepping
//! values with the pad (stick auto-repeat in `un_804D6E4C`), and the menu stack (`prev` / `next`).
//! Callees of unported TUs go through `ext`. Checked against the C (whole TU, shipping contraction)
//! by tests/tu_textlib_1.rs.
//!
//! `DevText_Printf` is C-variadic; its callees here pass [`PrintfArg`](super::textlib::PrintfArg)
//! slices. Float to integer conversions are Rust `as` (saturating, NaN to 0), which is what the
//! shipping build's gwtool defines; `sfxMove()` is the header inline `lbAudioAx_80024030(2)`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_types::*;

use super::textlib::PrintfArg;

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
    fn DevText_Create(a0: i8, a1: i32, a2: i32, a3: i32, a4: i32, a5: *mut i8) -> *mut DevText = |a0: i8, a1: i32, a2: i32, a3: i32, a4: i32, a5: *mut i8| unsafe { crate::tu::textlib::DevText_Create(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _) as _ };
    fn DevText_Erase(a0: *mut DevText) = |a0: *mut DevText| { unsafe { crate::tu::textlib::DevText_Erase(a0 as _); } };
    fn DevText_GetGObj() -> *mut HSD_GObj = || unsafe { crate::tu::textdraw::DevText_GetGObj() as _ };
    fn DevText_HideBackground(a0: *mut DevText) = |a0: *mut DevText| { unsafe { crate::tu::textlib::DevText_HideBackground(a0 as _); } };
    fn DevText_HideCursor(a0: *mut DevText) = |a0: *mut DevText| { unsafe { crate::tu::textlib::DevText_HideCursor(a0 as _); } };
    fn DevText_HideText(a0: *mut DevText) = |a0: *mut DevText| { unsafe { crate::tu::textlib::DevText_HideText(a0 as _); } };
    fn DevText_Print(a0: *mut DevText, a1: *mut i8) = |a0: *mut DevText, a1: *mut i8| { unsafe { crate::tu::textlib::DevText_Print(a0 as _, a1 as _); } };
    fn DevText_PrintInt(a0: *mut DevText, a1: i32) = |a0: *mut DevText, a1: i32| { unsafe { crate::tu::textlib::DevText_PrintInt(a0 as _, a1 as _); } };
    fn DevText_Printf(a0: *mut DevText, a1: *const i8, a2: &[crate::tu::textlib::PrintfArg]) = |a0: *mut DevText, a1: *const i8, a2: &[crate::tu::textlib::PrintfArg]| { unsafe { crate::tu::textlib::DevText_Printf(a0, a1, a2); } };
    fn DevText_Remove(a0: *mut *mut DevText) = |a0: *mut *mut DevText| { unsafe { crate::tu::textdraw::DevText_Remove(a0 as _); } };
    fn DevText_SetBGColor(a0: *mut DevText, a1: _GXColor) -> _GXColor = |a0: *mut DevText, a1: _GXColor| unsafe { crate::tu::textlib::DevText_SetBGColor(a0 as _, a1 as _) as _ };
    fn DevText_SetCursorXY(a0: *mut DevText, a1: i32, a2: i32) = |a0: *mut DevText, a1: i32, a2: i32| { unsafe { crate::tu::textlib::DevText_SetCursorXY(a0 as _, a1 as _, a2 as _); } };
    fn DevText_SetScale(a0: *mut DevText, a1: f32, a2: f32) = |a0: *mut DevText, a1: f32, a2: f32| { unsafe { crate::tu::textlib::DevText_SetScale(a0 as _, a1 as _, a2 as _); } };
    fn DevText_SetTextColor(a0: *mut DevText, a1: _GXColor) -> _GXColor = |a0: *mut DevText, a1: _GXColor| unsafe { crate::tu::textlib::DevText_SetTextColor(a0 as _, a1 as _) as _ };
    fn DevText_Show(a0: *mut HSD_GObj, a1: *mut DevText) = |a0: *mut HSD_GObj, a1: *mut DevText| { unsafe { crate::tu::textdraw::DevText_Show(a0 as _, a1 as _); } };
    fn DevText_ShowBackground(a0: *mut DevText) = |a0: *mut DevText| { unsafe { crate::tu::textlib::DevText_ShowBackground(a0 as _); } };
    fn DevText_ShowText(a0: *mut DevText) = |a0: *mut DevText| { unsafe { crate::tu::textlib::DevText_ShowText(a0 as _); } };
    fn DevText_StoreColorIndex(a0: *mut DevText, a1: u8) -> u8 = |a0: *mut DevText, a1: u8| unsafe { crate::tu::textlib::DevText_StoreColorIndex(a0 as _, a1 as _) as _ };
    fn DevText_StrLen(a0: *mut i8) -> i32 = |a0: *mut i8| unsafe { crate::tu::textdraw::DevText_StrLen(a0 as _) as _ };
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj = |a0: u16, a1: u8, a2: u8| unsafe { melee_runtime::tu::gobj_native::GObj_Create(a0 as _, a1 as _, a2 as _) as _ };
    fn HSD_Free(a0: *mut c_void) = |a0: *mut c_void| { unsafe { melee_runtime::tu::memory::HSD_Free(a0 as _); } };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(a0 as _, a1 as _, a2 as _) as _ };
    fn HSD_MemAlloc(a0: i32) -> *mut c_void = |a0: i32| unsafe { melee_runtime::tu::memory::HSD_MemAlloc(a0 as _) as _ };
    fn HSD_ObjAlloc(a0: *mut _HSD_ObjAllocData) -> *mut c_void = |a0: *mut _HSD_ObjAllocData| unsafe { melee_runtime::tu::objalloc::HSD_ObjAlloc(a0 as _) as _ };
    fn HSD_ObjAllocInit(a0: *mut _HSD_ObjAllocData, a1: u32, a2: u32) = |a0: *mut _HSD_ObjAllocData, a1: u32, a2: u32| { unsafe { melee_runtime::tu::objalloc::HSD_ObjAllocInit(a0 as _, a1 as _, a2 as _); } };
    fn HSD_ObjFree(a0: *mut _HSD_ObjAllocData, a1: *mut c_void) = |a0: *mut _HSD_ObjAllocData, a1: *mut c_void| { unsafe { melee_runtime::tu::objalloc::HSD_ObjFree(a0 as _, a1 as _); } };
    fn lbAudioAx_80024030(a0: i32) = |a0: i32| { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024030(a0 as _); } };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub use melee_runtime::tu::controller::HSD_PadCopyStatus;
}
// END GENERATED EXT

/// `unsigned char un_804D6E4C` (4D6E4C): stick auto-repeat counter.
pub static mut un_804D6E4C: u8 = 0;
/// `struct un_80304138_objalloc_t_x8* un_804D6E48` (4D6E48): the entry whose callback runs.
pub static mut un_804D6E48: *mut un_80304138_objalloc_t_x8 = null_mut();
/// `struct un_80304138_objalloc_t* un_804D6E44` (4D6E44): the current menu.
pub static mut un_804D6E44: *mut un_80304138_objalloc_t = null_mut();
/// `struct un_80304138_objalloc_t* un_804D6E40` (4D6E40): the newest menu.
pub static mut un_804D6E40: *mut un_80304138_objalloc_t = null_mut();
/// `static HSD_ObjAllocData un_804A2688` (4A2688)
pub static mut un_804A2688: HSD_ObjAllocData = unsafe { core::mem::zeroed() };

/// `GXColor un_804D5A08` (4D5A08)
pub static mut un_804D5A08: _GXColor = _GXColor { r: 0x40, g: 0x50, b: 0x80, a: 0x80 };
/// `GXColor un_804D5A0C` (4D5A0C)
pub static mut un_804D5A0C: _GXColor = _GXColor { r: 0xE2, g: 0xE2, b: 0xE2, a: 0xFF };
/// `GXColor un_804D5A10` (4D5A10)
pub static mut un_804D5A10: _GXColor = _GXColor { r: 0xFF, g: 0x80, b: 0x20, a: 0xFF };
/// `GXColor un_804D5A14` (4D5A14)
pub static mut un_804D5A14: _GXColor = _GXColor { r: 0xA0, g: 0xA0, b: 0xFF, a: 0xFF };

/// `static inline void sfxMove(void)`
#[inline]
fn sfxMove() {
    ext::lbAudioAx_80024030(2);
}

/// Runs an entry's callback and, when it returns 0 (or the entry has none), the menu's `xC`.
/// This is the sequence the C repeats at every dispatch site; the return value is the callback's
/// (only `un_80302E00` uses it).
#[inline]
unsafe fn dispatch(x8: *mut un_80304138_objalloc_t_x8, code: u32) -> i32 {
    let mut ret = 0;
    if let Some(cb) = (*x8).x4 {
        un_804D6E48 = x8;
        ret = cb(code);
        if ret == 0 {
            if !un_804D6E44.is_null() {
                if let Some(q) = (*un_804D6E44).xC {
                    return q(code);
                }
            }
        }
    } else if !un_804D6E44.is_null() {
        if let Some(q) = (*un_804D6E44).xC {
            return q(code);
        }
    }
    ret
}

/// `int un_80302E00(x8*, int)`: runs the entry's callback, then the menu's.
pub unsafe extern "C" fn un_80302E00(arg0: *mut un_80304138_objalloc_t_x8, arg1: i32) -> i32 {
    melee_core::ct!("un_80302E00");
    dispatch(arg0, arg1 as u32)
}

/// `int un_80302EA4(x8*)`: the width in characters a menu needs (entries end at `x0 == 9`).
pub unsafe extern "C" fn un_80302EA4(mut arg0: *mut un_80304138_objalloc_t_x8) -> i32 {
    melee_core::ct!("un_80302EA4");
    let mut x = 1i32;
    let mut y = 1i32;
    let mut z = 1i32;
    while (*arg0).x0 != 9 {
        if (*arg0).x0 as u32 <= 1 {
            let len = ext::DevText_StrLen((*arg0).x8) + 1;
            if len > z {
                z = len;
            }
        } else {
            let len = ext::DevText_StrLen((*arg0).x8) + 1;
            if len > x {
                x = len;
            }
        }
        if ((*arg0).x0 == 3 || (*arg0).x0 == 8) && y < 3 {
            y = 3;
        }
        if (*arg0).x0 == 4 && y < 10 {
            y = 10;
        }
        if (*arg0).x0 == 5 && y < 2 {
            y = 2;
        }
        if (*arg0).x0 == 6 && y < 4 {
            y = 4;
        }
        if (*arg0).x0 == 7 && y < 8 {
            y = 8;
        }
        if (*arg0).x0 == 2 {
            let n = melee_core::ppc::fctiwz_f32((*arg0).x18);
            let mut i = 0;
            while i < n {
                let len = ext::DevText_StrLen(*(*arg0).xC.offset(i as isize));
                if len > y {
                    y = len;
                }
                i += 1;
            }
        }
        arg0 = arg0.add(1);
    }
    x += y;
    if x > z {
        x + 1
    } else {
        z + 1
    }
}

/// `static inline GXColor adjust(GXColor)`: halves r, g and b.
#[inline]
pub fn adjust(mut c: _GXColor) -> _GXColor {
    melee_core::ct!("adjust");
    c.r = (c.r as i32 * 50 / 100) as u8;
    c.g = (c.g as i32 * 50 / 100) as u8;
    c.b = (c.b as i32 * 50 / 100) as u8;
    c
}

/// `static inline int un_80302FFC_maxlen(x8*)`
pub unsafe fn un_80302FFC_maxlen(mut thing: *mut un_80304138_objalloc_t_x8) -> i32 {
    melee_core::ct!("un_80302FFC_maxlen");
    let mut cursor_x = 1;
    while (*thing).x0 != 9 {
        if (*thing).x0 != 0 && (*thing).x0 != 1 {
            let len = ext::DevText_StrLen((*thing).x8);
            if len + 1 > cursor_x {
                cursor_x = len + 1;
            }
        }
        thing = thing.add(1);
    }
    cursor_x
}

/// `void un_80302FFC(objalloc_t*)`: draws the rows of a menu into its text box.
pub unsafe extern "C" fn un_80302FFC(arg0: *mut un_80304138_objalloc_t) {
    melee_core::ct!("un_80302FFC");
    let mut x8 = (*arg0).x8;
    let cursor_x = un_80302FFC_maxlen((*arg0).x8);
    let t = (*arg0).x4;
    if (*arg0).x1 & 0x10 != 0 {
        ext::DevText_StoreColorIndex(t, 0);
        ext::DevText_SetTextColor(t, adjust(un_804D5A0C));
        ext::DevText_StoreColorIndex(t, 1);
        ext::DevText_SetTextColor(t, adjust(un_804D5A10));
        ext::DevText_StoreColorIndex(t, 2);
        ext::DevText_SetTextColor(t, adjust(un_804D5A14));
        ext::DevText_SetBGColor(t, adjust(un_804D5A08));
    } else {
        ext::DevText_StoreColorIndex(t, 0);
        ext::DevText_SetTextColor(t, un_804D5A0C);
        ext::DevText_StoreColorIndex(t, 1);
        ext::DevText_SetTextColor(t, un_804D5A10);
        ext::DevText_StoreColorIndex(t, 2);
        ext::DevText_SetTextColor(t, un_804D5A14);
        ext::DevText_SetBGColor(t, un_804D5A08);
    }
    let mut cursor_y = 0i32;
    while cursor_y < (*(*arg0).x4).h as i32 {
        if (*x8).x0 == 0 {
            ext::DevText_StoreColorIndex(t, 2);
        } else if (*arg0).x0 as i32 == cursor_y {
            ext::DevText_StoreColorIndex(t, 1);
        } else {
            ext::DevText_StoreColorIndex(t, 0);
        }
        ext::DevText_SetCursorXY(t, 0, cursor_y);
        ext::DevText_Print(t, (*x8).x8);
        ext::DevText_SetCursorXY(t, cursor_x, cursor_y);
        match (*x8).x0 {
            2 => ext::DevText_Print(t, *(*x8).xC.offset(*((*x8).x10 as *mut i32) as isize)),
            3 => ext::DevText_PrintInt(t, *((*x8).x10 as *mut i32)),
            4 => ext::DevText_Printf(t, b"%d\0".as_ptr() as *const i8, &[PrintfArg::Int(*((*x8).x10 as *mut i32))]),
            5 => ext::DevText_Printf(t, b"%02x\0".as_ptr() as *const i8, &[PrintfArg::Int(*((*x8).x10 as *mut u8) as i32)]),
            6 => ext::DevText_Printf(t, b"%04x\0".as_ptr() as *const i8, &[PrintfArg::Int(*((*x8).x10 as *mut u16) as i32)]),
            7 => ext::DevText_Printf(t, b"%08x\0".as_ptr() as *const i8, &[PrintfArg::Int(*((*x8).x10 as *mut i32))]),
            8 => ext::DevText_Printf(t, b"%3.2f\0".as_ptr() as *const i8, &[PrintfArg::Dbl(*((*x8).x10 as *mut f32) as f64)]),
            _ => {}
        }
        x8 = x8.add(1);
        cursor_y += 1;
    }
}

/// `bool un_80303444(objalloc_t*)`: steps the selected row's value up; true when it changed.
pub unsafe extern "C" fn un_80303444(arg0: *mut un_80304138_objalloc_t) -> i32 {
    melee_core::ct!("un_80303444");
    let mut ret = false;
    let e = (*arg0).x8.add((*arg0).x0 as usize);
    match (*e).x0 {
        2 => {
            let q = (*e).x10 as *mut i32;
            if (*q as f32) < (*e).x18 - 1.0 {
                *q = (*q).wrapping_add(1);
                ret = true;
                (*arg0).x1 |= 1;
                sfxMove();
            }
        }
        3 => {
            let q = (*e).x10 as *mut i32;
            if (*q as f32) + (*e).x1C <= (*e).x18 {
                ret = true;
                *q = melee_core::ppc::fctiwz_f32((*q as f32) + (*e).x1C);
                (*arg0).x1 |= 1;
                sfxMove();
            }
        }
        5 => {
            let q = (*e).x10 as *mut u8;
            let idk = melee_core::ppc::fctiwz_f32((*e).x1C);
            if (*q as i32).wrapping_add(idk & 0xFF) <= 0xFF {
                *q = (*q as i32).wrapping_add(idk) as u8;
            } else {
                *q = (*q as i32).wrapping_sub(0x100i32.wrapping_sub(idk)) as u8;
            }
            ret = true;
            (*arg0).x1 |= 1;
            sfxMove();
        }
        6 => {
            let q = (*e).x10 as *mut u16;
            let idk = melee_core::ppc::fctiwz_f32((*e).x1C);
            if (*q as i32).wrapping_add(idk & 0xFFFF) <= 0xFFFF {
                *q = (*q as i32).wrapping_add(idk) as u16;
            } else {
                *q = (*q as i32).wrapping_sub(0x10000i32.wrapping_sub(idk)) as u16;
            }
            ret = true;
            (*arg0).x1 |= 1;
            sfxMove();
        }
        4 | 7 => {
            // `*q + (idk & 0xFFFFFFFF) <= 0xFFFFFFFF` holds for every unsigned int
            let q = (*e).x10 as *mut u32;
            let idk = melee_core::ppc::fcvt_u32((*e).x1C);
            *q = (*q).wrapping_add(idk);
            ret = true;
            (*arg0).x1 |= 1;
            sfxMove();
        }
        8 => {
            let q = (*e).x10 as *mut f32;
            if *q + (*e).x1C <= (*e).x18 {
                *q += (*e).x1C;
                ret = true;
                (*arg0).x1 |= 1;
                sfxMove();
            }
        }
        _ => {}
    }
    ret as i32
}

/// `bool un_80303720(objalloc_t*)`: steps the selected row's value down; true when it changed.
pub unsafe extern "C" fn un_80303720(arg0: *mut un_80304138_objalloc_t) -> i32 {
    melee_core::ct!("un_80303720");
    let mut ret = false;
    let e = (*arg0).x8.add((*arg0).x0 as usize);
    match (*e).x0 {
        2 => {
            let q = (*e).x10 as *mut i32;
            if (*q as f32) > (*e).x14 {
                *q = (*q).wrapping_sub(1);
                ret = true;
                (*arg0).x1 |= 1;
                sfxMove();
            }
        }
        3 => {
            let q = (*e).x10 as *mut i32;
            if (*q as f32) - (*e).x1C >= (*e).x14 {
                ret = true;
                *q = melee_core::ppc::fctiwz_f32((*q as f32) - (*e).x1C);
                (*arg0).x1 |= 1;
                sfxMove();
            }
        }
        5 => {
            let q = (*e).x10 as *mut u8;
            let idk = melee_core::ppc::fctiwz_f32((*e).x1C);
            if (*q as i32).wrapping_sub(idk & 0xFF) >= 0 {
                *q = (*q as i32).wrapping_sub(idk) as u8;
            } else {
                *q = (*q as i32).wrapping_add(0x100i32.wrapping_sub(idk)) as u8;
            }
            ret = true;
            (*arg0).x1 |= 1;
            sfxMove();
        }
        6 => {
            let q = (*e).x10 as *mut u16;
            let idk = melee_core::ppc::fctiwz_f32((*e).x1C);
            if (*q as i32).wrapping_sub(idk & 0xFFFF) >= 0 {
                *q = (*q as i32).wrapping_sub(idk) as u16;
            } else {
                *q = (*q as i32).wrapping_add(0x10000i32.wrapping_sub(idk)) as u16;
            }
            ret = true;
            (*arg0).x1 |= 1;
            sfxMove();
        }
        4 | 7 => {
            // `*q - (idk & 0xFFFFFFFF) >= 0` holds for every unsigned int
            let q = (*e).x10 as *mut u32;
            let idk = melee_core::ppc::fcvt_u32((*e).x1C);
            *q = (*q).wrapping_sub(idk);
            ret = true;
            (*arg0).x1 |= 1;
            sfxMove();
        }
        8 => {
            let q = (*e).x10 as *mut f32;
            if *q - (*e).x1C >= (*e).x14 {
                *q -= (*e).x1C;
                ret = true;
                (*arg0).x1 |= 1;
                sfxMove();
            }
        }
        _ => {}
    }
    ret as i32
}

/// `int un_803039A4(u8 pad)`: stick direction bits (0x40000000 left, 0x80000000 right, 0x20000000
/// down, 0x10000000 up) with a repeat delay held in `un_804D6E4C`.
pub unsafe extern "C" fn un_803039A4(arg0: u8) -> i32 {
    melee_core::ct!("un_803039A4");
    let pad = &HSD_PadCopyStatus_ref()[arg0 as usize];
    let mut ret: u32 = 0;
    let button: u8 = if HSD_PadCopyStatus_ref()[0].button as i32 & HSD_PAD_R != 0 { 1 } else { 8 };
    if (pad.stickX as i32) < -60 && un_804D6E4C == 0 {
        ret |= 0x40000000;
    }
    if (pad.stickX as i32) > 60 && un_804D6E4C == 0 {
        ret |= 0x80000000;
    }
    if (pad.stickY as i32) < -60 && un_804D6E4C == 0 {
        ret |= 0x20000000;
    }
    if (pad.stickY as i32) > 60 && un_804D6E4C == 0 {
        ret |= 0x10000000;
    }
    if ret != 0 && un_804D6E4C == 0 {
        un_804D6E4C = button;
    }
    let ax = if (pad.stickX as i32) < 0 { -(pad.stickX as i32) } else { pad.stickX as i32 };
    let ay = if (pad.stickY as i32) < 0 { -(pad.stickY as i32) } else { pad.stickY as i32 };
    if ax <= 60 && ay <= 60 {
        un_804D6E4C = 0;
    }
    if un_804D6E4C != 0 {
        un_804D6E4C -= 1;
    }
    ret as i32
}

#[inline]
unsafe fn HSD_PadCopyStatus_ref() -> &'static mut [HSD_PadStatus; 4] {
    &mut ext::HSD_PadCopyStatus
}

/// `void un_80303AC4(objalloc_t*)`: reads the pad and moves the cursor / steps the value / fires
/// the entry callbacks (6 start, 3 right, 2 left, 1 A, 0 B).
pub unsafe extern "C" fn un_80303AC4(arg0: *mut un_80304138_objalloc_t) {
    melee_core::ct!("un_80303AC4");
    let trigger = HSD_PadCopyStatus_ref()[0].trigger as i32;
    let stick = un_803039A4(0);
    let buttons = stick | trigger;
    if buttons & HSD_PAD_START != 0 {
        dispatch((*arg0).x8.add((*arg0).x0 as usize), 6);
    } else if buttons & (0x10000000 | HSD_PAD_Y) != 0 {
        // up
        let mut i = (*arg0).x0 as i32;
        i -= 1;
        let mut found = false;
        while i >= 0 {
            if (*(*arg0).x8.add(i as usize)).x0 != 0 {
                found = true;
                break;
            }
            i -= 1;
        }
        if !found {
            i = -1;
        }
        if i != -1 {
            (*arg0).x0 = i as u8;
            (*arg0).x1 |= 1;
            sfxMove();
        }
    } else if buttons & (0x20000000 | HSD_PAD_X) != 0 {
        // down
        let mut i = (*arg0).x0 as i32;
        i += 1;
        let mut found = false;
        while i < (*(*arg0).x4).h as i32 {
            if (*(*arg0).x8.add(i as usize)).x0 != 0 {
                found = true;
                break;
            }
            i += 1;
        }
        if !found {
            i = -1;
        }
        if i != -1 {
            (*arg0).x0 = i as u8;
            (*arg0).x1 |= 1;
            sfxMove();
        }
    } else if buttons & (0x80000000u32 as i32 | HSD_PAD_R) != 0 {
        // right
        if un_80303444(arg0) != 0 {
            dispatch((*arg0).x8.add((*arg0).x0 as usize), 3);
        }
    } else if buttons & (0x40000000 | HSD_PAD_L) != 0 {
        // left
        if un_80303720(arg0) != 0 {
            dispatch((*arg0).x8.add((*arg0).x0 as usize), 2);
        }
    } else if buttons & HSD_PAD_A != 0 {
        dispatch((*arg0).x8.add((*arg0).x0 as usize), 1);
    } else if buttons & HSD_PAD_B != 0 {
        dispatch((*arg0).x8.add((*arg0).x0 as usize), 0);
    }
}

/// `void fn_80303EF4(HSD_GObj*)`: the menu proc, newest menu first.
pub unsafe extern "C" fn fn_80303EF4(_gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80303EF4");
    let mut q = un_804D6E40;
    while !q.is_null() {
        if (*q).x1 & 0x20 != 0 {
            ext::DevText_HideText((*q).x4);
            ext::DevText_HideBackground((*q).x4);
        } else {
            ext::DevText_ShowText((*q).x4);
            ext::DevText_ShowBackground((*q).x4);
            if (*q).x1 & 1 != 0 {
                ext::DevText_Erase((*q).x4);
                un_80302FFC(q);
                (*q).x1 &= !1;
            } else {
                un_80302FFC(q);
            }
            if (*q).x1 & 2 != 0 {
                (*q).x1 &= !2;
            } else if (*q).x1 & 0x10 == 0 {
                un_80303AC4(q);
            }
        }
        if (*q).x1 & 0x80 != 0 {
            un_80304344(q);
            q = null_mut();
        } else {
            q = (*q).prev;
        }
    }
}

/// `static inline int un_80303FD4_first(x8*)`: index of the first selectable row.
pub unsafe fn un_80303FD4_first(mut p: *mut un_80304138_objalloc_t_x8) -> i32 {
    melee_core::ct!("un_80303FD4_first");
    let mut i = 0;
    loop {
        let v = (*p).x0;
        if v == 9 {
            break;
        }
        if v != 0 {
            return i;
        }
        p = p.add(1);
        i += 1;
    }
    0
}

/// `static inline int un_80303FD4_count(x8*)`: number of rows.
pub unsafe fn un_80303FD4_count(mut p: *mut un_80304138_objalloc_t_x8) -> i32 {
    melee_core::ct!("un_80303FD4_count");
    let mut n = 0;
    while (*p).x0 != 9 {
        n += 1;
        p = p.add(1);
    }
    n
}

/// `void un_80303FD4(HSD_GObj*, objalloc_t*, x8*, int, int, int)`: builds a menu at (arg4, arg5).
pub unsafe extern "C" fn un_80303FD4(
    arg0: *mut HSD_GObj,
    arg1: *mut un_80304138_objalloc_t,
    arg2: *mut un_80304138_objalloc_t_x8,
    _arg3: i32,
    arg4: i32,
    arg5: i32,
) {
    melee_core::ct!("un_80303FD4");
    let mut count2 = 0i32;
    (*arg1).x8 = arg2;
    (*arg1).x1 = 0;
    (*arg1).prev = null_mut();
    (*arg1).next = null_mut();
    (*arg1).x10 = arg0;

    let count = un_80303FD4_count((*arg1).x8);
    let size = un_80302EA4((*arg1).x8);
    un_804D6E44 = arg1;
    let buf = ext::HSD_MemAlloc(size.wrapping_mul(count).wrapping_mul(2));
    if !buf.is_null() {
        let mut un = un_804D6E40;
        while !un.is_null() && un != arg1 {
            un = (*un).prev;
            count2 += 1;
        }
        (*arg1).x4 = ext::DevText_Create((count2 + 0x78) as i8, arg4, arg5, size, count, buf as *mut i8);
        if !(*arg1).x4.is_null() {
            ext::DevText_Show(arg0, (*arg1).x4);
            (*arg1).x0 = un_80303FD4_first((*arg1).x8) as u8;
            (*arg1).xC = None;
            ext::DevText_HideCursor((*arg1).x4);
            ext::DevText_SetScale((*arg1).x4, 10.0, 17.0);
            un_804D6E48 = null_mut();
            let mut p2 = (*arg1).x8;
            while (*p2).x0 != 9 {
                un_80302E00(p2, 4);
                p2 = p2.add(1);
            }
        }
    }
}

/// `void un_80304138(void)`: initialises the menu allocator.
pub unsafe extern "C" fn un_80304138() {
    melee_core::ct!("un_80304138");
    ext::HSD_ObjAllocInit(&raw mut un_804A2688, core::mem::size_of::<un_80304138_objalloc_t>() as u32, 4);
}

/// `HSD_GObj* un_80304168(void*, int, int, int)`: a menu gobj under the DevText gobj's links.
pub unsafe extern "C" fn un_80304168(arg0: *mut c_void, arg1: i32, arg2: i32, arg3: i32) -> *mut HSD_GObj {
    melee_core::ct!("un_80304168");
    let gobj = ext::DevText_GetGObj();
    let mut gobj2: *mut HSD_GObj = null_mut();
    if !gobj.is_null() {
        gobj2 = ext::GObj_Create((*gobj).classifier, (*gobj).p_link, (*gobj).p_priority);
        if !gobj2.is_null() {
            let userdata = ext::HSD_ObjAlloc(&raw mut un_804A2688) as *mut un_80304138_objalloc_t;
            un_804D6E40 = userdata;
            un_80303FD4(gobj2, userdata, arg0 as *mut un_80304138_objalloc_t_x8, arg1, arg2, arg3);
            (*userdata).x14 = ext::HSD_GObj_SetupProc(gobj2, Some(fn_80303EF4), 0);
        }
    }
    gobj2
}

/// `objalloc_t* un_80304210(objalloc_t*, void*, int, int, int)`: a submenu to the right of a row.
/// The position uses `scale_y * x0 + arg4` as an f32 fused multiply-add.
pub unsafe extern "C" fn un_80304210(
    arg0: *mut un_80304138_objalloc_t,
    arg1: *mut c_void,
    arg2: i32,
    arg3: i32,
    arg4: i32,
) -> *mut un_80304138_objalloc_t {
    melee_core::ct!("un_80304210");
    let obj = ext::HSD_ObjAlloc(&raw mut un_804A2688) as *mut un_80304138_objalloc_t;
    if !obj.is_null() {
        let text = (*arg0).x4;
        let x = (*text).scale_x * ((*text).w as f32);
        let x_pos = melee_core::ppc::fctiwz_f32((arg3 as f32) + x);
        let y_pos = melee_core::ppc::fctiwz_f32(fmadds((*text).scale_y, (*arg0).x0 as f32, arg4 as f32));
        un_80303FD4(
            (*arg0).x10,
            obj,
            arg1 as *mut un_80304138_objalloc_t_x8,
            arg2,
            (*text).x as i32 + x_pos,
            (*text).y as i32 + y_pos,
        );
        (*arg0).x1 |= 0x10;
        (*arg0).prev = obj;
        (*obj).next = arg0;
        (*obj).x1 |= 2;
    }
    obj
}

/// `void un_80304334(objalloc_t*)`: marks the menu for removal.
pub unsafe extern "C" fn un_80304334(arg0: *mut un_80304138_objalloc_t) {
    melee_core::ct!("un_80304334");
    (*arg0).x1 |= 1 << 7;
}

/// `void un_80304344(objalloc_t*)`: closes the menu and every menu opened from it (callback 5 each).
pub unsafe extern "C" fn un_80304344(mut arg0: *mut un_80304138_objalloc_t) {
    melee_core::ct!("un_80304344");
    let next = (*arg0).next;
    if !next.is_null() {
        (*next).x1 &= !0x10;
        (*next).prev = null_mut();
    }
    while !arg0.is_null() {
        let mut r4 = (*arg0).x8;
        un_804D6E44 = arg0;
        while (*r4).x0 != 9 {
            r4 = r4.add(1);
        }
        if let Some(cb) = (*r4).x4 {
            un_804D6E48 = r4;
            if cb(5) == 0 && !un_804D6E44.is_null() {
                if let Some(q) = (*un_804D6E44).xC {
                    q(5);
                }
            }
        } else if !arg0.is_null() {
            if let Some(q) = (*arg0).xC {
                q(5);
            }
        }
        ext::HSD_Free((*(*arg0).x4).buf as *mut c_void);
        ext::DevText_Remove(&raw mut (*arg0).x4);
        let w = (*arg0).prev;
        ext::HSD_ObjFree(&raw mut un_804A2688, arg0 as *mut c_void);
        arg0 = w;
    }
    un_804D6E44 = next;
}
