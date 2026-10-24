//! Port of src/melee/lb/lb_01F8.c (the THP movie player's texture side: the SObj that shows the
//! decoded frame, the three Y/U/V texture objects, and the first-frame decode setup).
//! Checked against the C (whole TU, shipping contraction) by tests/tu_lb_01F8.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use core::ffi::c_void;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_modes/gen/cref/gen_tu.py; the callees of the C TU)
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
    fn DCInvalidateRange(a0: *mut c_void, a1: u32);
    fn GXInitTexObj(a0: *mut _GXTexObj, a1: *mut c_void, a2: u16, a3: u16, a4: u32, a5: u32, a6: u32, a7: u8);
    fn GXInitTexObjLOD(a0: *mut _GXTexObj, a1: u32, a2: u32, a3: f32, a4: f32, a5: f32, a6: u8, a7: u8, a8: u32);
    fn GXLoadTexObj(a0: *mut _GXTexObj, a1: u32);
    fn HSD_Free(a0: *mut c_void);
    fn HSD_MemAlloc(a0: i32) -> *mut c_void;
    fn HSD_SObjLib_803A477C(a0: *mut HSD_GObj, a1: *mut HSD_SObjDesc, a2: u32, a3: u32, a4: u8, a5: u8) -> *mut HSD_SObj;
    fn HSD_SObjLib_803A49E0(a0: *mut HSD_GObj, a1: i32);
    fn THPDec_8032F8D4(a0: *mut u8, a1: *mut c_void /* THPDec_8032FD40_Data */) -> i32;
    fn THPDec_8032FD40(a0: *mut c_void /* THPDec_8032FD40_Data */, a1: u16) -> i32;
    fn THPDec_80331340(a0: *mut c_void /* _THPFileInfo */, a1: *mut c_void, a2: *mut c_void, a3: *mut c_void);
    fn THPDec_803313D0(a0: *mut c_void /* _THPFileInfo */, a1: *mut c_void, a2: *mut c_void, a3: *mut c_void, a4: u32);
    fn THPInit();
    fn THPVideoDecode(a0: *mut c_void, a1: *mut c_void, a2: *mut c_void /* _THPFileInfo */, a3: *mut c_void, a4: *mut c_void /* THPDec_8032FD40_Data */) -> *mut c_void /* _THPFileInfo */;
    fn lbFile_80016760(a0: *const i8, a1: *mut *mut c_void, a2: *mut u32);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// `GX_TF_RGBA8`
const GX_TF_RGBA8: u32 = 6;

/// `struct lbl_804335B8_t` (0xA0 on the GC; the three GXTexObj are 0x20 there, 0x40 natively).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct lbl_804335B8_t {
    pub tex0: GXTexObj,
    pub x20: *mut c_void,
    pub tex1: GXTexObj,
    pub x44: *mut c_void,
    pub tex2: GXTexObj,
    pub x68: *mut c_void,
    pub x6C: u16,
    pub x6E: u16,
    pub x70: HSD_ImageDesc,
    pub x88: HSD_SObjDesc,
    pub x90: *mut HSD_SObj,
    pub unk94: *mut c_void,
    pub unk98: u32,
    pub pad_9C: [i8; 4],
}
melee_core::gc_layout!(lbl_804335B8_t, "lbl_804335B8_t", size 0xA0, align 4, [
    tex0 @ 0x0: 0x20 "GXTexObj",
    x20 @ 0x20: 4 "void*",
    tex1 @ 0x24: 0x20 "GXTexObj",
    x44 @ 0x44: 4 "void*",
    tex2 @ 0x48: 0x20 "GXTexObj",
    x68 @ 0x68: 4 "void*",
    x6C @ 0x6C: 2 "u16",
    x6E @ 0x6E: 2 "u16",
    x70 @ 0x70: 0x18 "HSD_ImageDesc",
    x88 @ 0x88: 8 "HSD_SObjDesc",
    x90 @ 0x90: 4 "HSD_SObj*",
    unk94 @ 0x94: 4 "void*",
    unk98 @ 0x98: 4 "size_t",
    pad_9C @ 0x9C: 4 "char[4]",
]);

/// 4335B8 (static)
pub static mut lbl_804335B8: lbl_804335B8_t = unsafe { core::mem::zeroed() };

/// 01F890: creates the movie SObj over the (not yet allocated) image.
pub unsafe fn lbMthp8001F890(gobj: *mut HSD_GObj) -> *mut HSD_SObj {
    let l = &raw mut lbl_804335B8;
    (*l).x70.image_ptr = core::ptr::null_mut();
    (*l).x70.width = (*l).x6C;
    (*l).x70.height = (*l).x6E;
    (*l).x70.format = GX_TF_RGBA8;
    (*l).x70.mipmap = 0;
    (*l).x70.maxLOD = 0.0;
    (*l).x70.minLOD = (*l).x70.maxLOD;
    (*l).x88.image = &raw mut (*l).x70;
    (*l).x88.tlut = core::ptr::null_mut();
    (*l).x90 = ext::HSD_SObjLib_803A477C(gobj, &raw mut (*l).x88, 0, 0, 0x80, 0);
    (*(*l).x90).x40 |= 0x10;
    (*l).x90
}

/// 01F928: loads the Y (full size) and U / V (half size) textures and draws the SObj.
pub unsafe fn lbMthp8001F928(gobj: *mut HSD_GObj, arg1: i32) {
    let l = &raw mut lbl_804335B8;
    let p_width = &raw mut (*l).x6C;
    let p_height = &raw mut (*l).x6E;

    // First texture - full size
    ext::GXInitTexObj(&raw mut (*l).tex0, (*l).x20, (*l).x6C, (*l).x6E, 1, 0, 0, 0);
    ext::GXInitTexObjLOD(&raw mut (*l).tex0, 0, 0, 0.0, 0.0, 0.0, 0, 0, 0);
    ext::GXLoadTexObj(&raw mut (*l).tex0, 0);

    // Second texture - half size
    ext::GXInitTexObj(&raw mut (*l).tex1, (*l).x44, *p_width >> 1, *p_height >> 1, 1, 0, 0, 0);
    ext::GXInitTexObjLOD(&raw mut (*l).tex1, 0, 0, 0.0, 0.0, 0.0, 0, 0, 0);
    ext::GXLoadTexObj(&raw mut (*l).tex1, 1);

    // Third texture - half size
    ext::GXInitTexObj(&raw mut (*l).tex2, (*l).x68, *p_width >> 1, *p_height >> 1, 1, 0, 0, 0);
    ext::GXInitTexObjLOD(&raw mut (*l).tex2, 0, 0, 0.0, 0.0, 0.0, 0, 0, 0);
    ext::GXLoadTexObj(&raw mut (*l).tex2, 2);

    ext::HSD_SObjLib_803A49E0(gobj, arg1);
}

/// `header` of lbMthp8001FAA0: the THP decoder's frame header (0x1C bytes).
#[repr(C)]
#[derive(Clone, Copy)]
struct Header {
    w: u16,
    h: u16,
    pad: [u8; 0x18],
}

/// 01FAA0: allocates the YUV planes and decodes the first frame of `filename`.
pub unsafe fn lbMthp8001FAA0(filename: *const i8, width: i32, height: i32) {
    let l = &raw mut lbl_804335B8;
    let mut header: Header = core::mem::zeroed();
    let mut output: i32 = 0;

    (*l).x6C = width as u16;
    (*l).x6E = height as u16;
    ext::THPInit();
    ext::lbFile_80016760(filename, &raw mut (*l).unk94, &raw mut (*l).unk98);
    let yuv_size: i32 = ((*l).x6C as i32).wrapping_mul((*l).x6E as i32);
    (*l).x20 = ext::HSD_MemAlloc(yuv_size);
    ext::DCInvalidateRange((*l).x20, yuv_size as u32);
    let uv_size: i32 = (((*l).x6C as i32).wrapping_mul((*l).x6E as i32)) >> 2;
    (*l).x44 = ext::HSD_MemAlloc(uv_size);
    ext::DCInvalidateRange((*l).x44, uv_size as u32);
    (*l).x68 = ext::HSD_MemAlloc(uv_size);
    ext::DCInvalidateRange((*l).x68, uv_size as u32);
    let context = ext::HSD_MemAlloc(0xC);
    core::ptr::write_bytes(&raw mut header, 0, 1); // memset(&header, 0, sizeof header)
    header.w = width as u16;
    header.h = height as u16;
    ext::THPDec_8032F8D4((*l).unk94 as *mut u8, context);
    let decode_buf = ext::HSD_MemAlloc(ext::THPDec_8032FD40(context, header.h));
    let decoded = ext::THPVideoDecode(
        (&raw mut header) as *mut c_void,
        (&raw mut output) as *mut c_void,
        decode_buf,
        (*l).unk94,
        context,
    );
    if (*l).x6C == 0x280 {
        ext::THPDec_80331340(decoded, (*l).x20, (*l).x44, (*l).x68);
    } else {
        ext::THPDec_803313D0(decoded, (*l).x20, (*l).x44, (*l).x68, (*l).x6C as u32);
    }
    ext::HSD_Free(context);
    ext::HSD_Free(decode_buf);
}
