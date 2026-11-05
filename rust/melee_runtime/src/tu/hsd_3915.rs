//! Port of src/sysdolphin/baselib/hsd_3915.c: the HSD debug line/vector font and primitive
//! drawing helpers (`DrawRectangle`, `DrawASCII`, `hsd_80391A04` .. `hsd_803922FC`).
//!
//! The TU's globals (`lbl_804D6070/74/78`) live in [`Hsd3915`]; the GX / HSD state calls go
//! through [`Hsd3915Host`], whose default [`ext::Ext`] is the unported-callee stub set (panics,
//! named like the C). `HSD_DebugFontAtlas` (debug_font.inc, generated data) is not ported here.
//!
//! Float fusion follows the shipping build (`-ffp-contract=on`): `f32 * double + f32` sites are
//! true f64 `fma`; `f32 * f32 + f32` sites are `fmadds` (f64 multiply/add, then f32).
#![allow(non_snake_case)]

use melee_core::fp::{fmadds, frsqrte};
use melee_types::{GXColor, GlyphEntry, HSD_CObj, Mtx};

pub const GX_VTXFMT0: u32 = 0;
pub const GX_VA_POS: u32 = 9;
pub const GX_VA_CLR0: u32 = 11;
pub const GX_DIRECT: u32 = 1;
pub const GX_TEX_S: u32 = 0;
pub const GX_TEX_ST: u32 = 1;
pub const GX_RGBA6: u32 = 4;
pub const GX_RGBA8: u32 = 5;

/// Callees of hsd_3915.c.
pub trait Hsd3915Host {
    fn GXBegin(&mut self, prim: u32, fmt: u32, n: u32);
    fn GXPosition2f32(&mut self, x: f32, y: f32);
    fn GXColor4u8(&mut self, r: u8, g: u8, b: u8, a: u8);
    fn HSD_ClearVtxDesc(&mut self);
    fn GXSetCurrentMtx(&mut self, id: u32);
    fn HSD_CObjGetCurrent(&mut self) -> *mut HSD_CObj;
    fn HSD_CObjGetViewingMtx(&mut self, cobj: *mut HSD_CObj, out: &mut Mtx);
    fn GXLoadPosMtxImm(&mut self, m: &Mtx, id: u32);
    fn HSD_StateSetLineWidth(&mut self, width: u8, mode: i32);
    fn HSD_StateSetPointSize(&mut self, size: i32, mode: i32);
    fn HSD_SetupRenderMode(&mut self, mode: u32);
    fn GXSetVtxAttrFmt(&mut self, fmt: u32, attr: u32, cnt: u32, ty: u32, frac: u8);
    fn GXSetVtxDesc(&mut self, attr: u32, ty: u32);
    fn HSD_StateSetCullMode(&mut self, mode: u32);
}

pub mod ext {
    use super::*;
    /// Unported callees (TODO: rewire once their TUs are ported).
    pub struct Ext;
    impl Hsd3915Host for Ext {
        fn GXBegin(&mut self, _: u32, _: u32, _: u32) { if melee_core::ct::render_stub() { return; } todo!("GXBegin") }
        fn GXPosition2f32(&mut self, _: f32, _: f32) { if melee_core::ct::render_stub() { return; } todo!("GXPosition2f32") }
        fn GXColor4u8(&mut self, _: u8, _: u8, _: u8, _: u8) { if melee_core::ct::render_stub() { return; } todo!("GXColor4u8") }
        fn HSD_ClearVtxDesc(&mut self) { todo!("HSD_ClearVtxDesc (baselib/state.c)") }
        fn GXSetCurrentMtx(&mut self, _: u32) { if melee_core::ct::render_stub() { return; } todo!("GXSetCurrentMtx") }
        fn HSD_CObjGetCurrent(&mut self) -> *mut HSD_CObj { todo!("HSD_CObjGetCurrent (baselib/cobj.c)") }
        fn HSD_CObjGetViewingMtx(&mut self, _: *mut HSD_CObj, _: &mut Mtx) { todo!("HSD_CObjGetViewingMtx (baselib/cobj.c)") }
        fn GXLoadPosMtxImm(&mut self, _: &Mtx, _: u32) { if melee_core::ct::render_stub() { return; } todo!("GXLoadPosMtxImm") }
        fn HSD_StateSetLineWidth(&mut self, _: u8, _: i32) { if melee_core::ct::render_stub() { return; } todo!("HSD_StateSetLineWidth") }
        fn HSD_StateSetPointSize(&mut self, _: i32, _: i32) { if melee_core::ct::render_stub() { return; } todo!("HSD_StateSetPointSize") }
        fn HSD_SetupRenderMode(&mut self, _: u32) { if melee_core::ct::render_stub() { return; } todo!("HSD_SetupRenderMode") }
        fn GXSetVtxAttrFmt(&mut self, _: u32, _: u32, _: u32, _: u32, _: u8) { if melee_core::ct::render_stub() { return; } todo!("GXSetVtxAttrFmt") }
        fn GXSetVtxDesc(&mut self, _: u32, _: u32) { if melee_core::ct::render_stub() { return; } todo!("GXSetVtxDesc") }
        fn HSD_StateSetCullMode(&mut self, _: u32) { if melee_core::ct::render_stub() { return; } todo!("HSD_StateSetCullMode") }
    }
}

/// The engine's host: hsd_3915.c's callees as the native TUs and the GX bridge provide them.
pub struct NativeHost;
impl Hsd3915Host for NativeHost {
    fn GXBegin(&mut self, prim: u32, fmt: u32, n: u32) { crate::tu::lbbgflash::ext::GXBegin(prim, fmt, n as u16) }
    fn GXPosition2f32(&mut self, x: f32, y: f32) { crate::tu::lbbgflash::ext::GXPosition2f32(x, y) }
    fn GXColor4u8(&mut self, r: u8, g: u8, b: u8, a: u8) { crate::tu::lbbgflash::ext::GXColor4u8(r, g, b, a) }
    fn HSD_ClearVtxDesc(&mut self) { unsafe { crate::tu::pobj::HSD_ClearVtxDesc() } }
    fn GXSetCurrentMtx(&mut self, id: u32) { crate::tu::pobj::ext::GXSetCurrentMtx(id) }
    fn HSD_CObjGetCurrent(&mut self) -> *mut HSD_CObj { unsafe { crate::tu::cobj::HSD_CObjGetCurrent() } }
    fn HSD_CObjGetViewingMtx(&mut self, cobj: *mut HSD_CObj, out: &mut Mtx) { unsafe { crate::tu::cobj::HSD_CObjGetViewingMtx(cobj, out.as_mut_ptr()) } }
    fn GXLoadPosMtxImm(&mut self, m: &Mtx, id: u32) { crate::tu::pobj::ext::GXLoadPosMtxImm(m.as_ptr() as *mut core::ffi::c_void, id) }
    fn HSD_StateSetLineWidth(&mut self, width: u8, mode: i32) { unsafe { crate::tu::state_native::HSD_StateSetLineWidth(width, mode) } }
    fn HSD_StateSetPointSize(&mut self, size: i32, mode: i32) { unsafe { crate::tu::state_native::HSD_StateSetPointSize(size as u8, mode) } }
    fn HSD_SetupRenderMode(&mut self, mode: u32) { unsafe { crate::tu::state_native::HSD_SetupRenderMode(mode) } }
    fn GXSetVtxAttrFmt(&mut self, fmt: u32, attr: u32, cnt: u32, ty: u32, frac: u8) { crate::tu::pobj::ext::GXSetVtxAttrFmt(fmt, attr, cnt, ty, frac) }
    fn GXSetVtxDesc(&mut self, attr: u32, ty: u32) { crate::tu::pobj::ext::GXSetVtxDesc(attr, ty) }
    fn HSD_StateSetCullMode(&mut self, mode: u32) { unsafe { crate::tu::state_native::HSD_StateSetCullMode(mode as i32) } }
}

/// The TU's globals for the engine (`lbl_804D6070/74/78`).
pub static mut NATIVE: Hsd3915 = Hsd3915 { lbl_804D6070: 12.0, lbl_804D6074: 12.0, lbl_804D6078: 12 };

/// `hsd_80391A04` on the engine's GX: the 2D line / quad state the bg flash and the text drawer set before drawing.
/// (Left out, the next immediate-mode quad streams with the previous draw's vertex format and the FIFO desynchronises.)
pub unsafe fn native_80391A04(scale_x: f32, scale_y: f32, line_width: i32) {
    (*core::ptr::addr_of_mut!(NATIVE)).hsd_80391A04(&mut NativeHost, scale_x, scale_y, line_width)
}

/// `lbl_80408630`: 47 glyphs of 13 bytes (nibble-packed x/y vertex pairs, 0xFF terminated).
pub static LBL_80408630: [u8; 0x268] = [
    0x00, 0x60, 0x60, 0x66, 0x66, 0x06, 0x06, 0x00, 0xFF, 0x00, 0x00, 0x00,
    0x00, 0x30, 0x36, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x60, 0x60, 0x63, 0x63, 0x03, 0x03, 0x06, 0x06, 0x66,
    0xFF, 0x00, 0x00, 0x00, 0x60, 0x60, 0x66, 0x66, 0x06, 0x13, 0x63, 0xFF,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x03, 0x63, 0x40, 0x46, 0xFF, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x60, 0x00, 0x00, 0x03, 0x03, 0x63, 0x63,
    0x66, 0x66, 0x06, 0xFF, 0x00, 0x00, 0x00, 0x06, 0x06, 0x66, 0x66, 0x63,
    0x63, 0x03, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x60, 0x60, 0x62, 0x62,
    0x43, 0x43, 0x46, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x60, 0x60, 0x66,
    0x66, 0x06, 0x06, 0x00, 0x03, 0x63, 0xFF, 0x00, 0x00, 0x63, 0x03, 0x03,
    0x00, 0x00, 0x60, 0x60, 0x66, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x06, 0x30,
    0x30, 0x66, 0x14, 0x54, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x40, 0x40, 0x61, 0x61, 0x66, 0x66, 0x06, 0x06, 0x00, 0x03, 0x63, 0xFF,
    0x60, 0x20, 0x20, 0x03, 0x03, 0x26, 0x26, 0x66, 0xFF, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x30, 0x30, 0x63, 0x63, 0x36, 0x36, 0x06, 0x10, 0x16, 0xFF,
    0x00, 0x00, 0x60, 0x00, 0x00, 0x06, 0x06, 0x66, 0x03, 0x53, 0xFF, 0x00,
    0x00, 0x00, 0x00, 0x60, 0x00, 0x00, 0x06, 0x02, 0x42, 0xFF, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x60, 0x30, 0x30, 0x02, 0x02, 0x06, 0x06, 0x66,
    0x66, 0x63, 0x63, 0x33, 0xFF, 0x00, 0x06, 0x60, 0x66, 0x03, 0x63, 0xFF,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x50, 0x16, 0x56, 0x30, 0x36,
    0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x50, 0x56, 0x56, 0x16, 0x16,
    0x14, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x60, 0x03,
    0x03, 0x66, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x06,
    0x66, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00,
    0x00, 0x33, 0x33, 0x60, 0x60, 0x66, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x06,
    0x00, 0x00, 0x66, 0x66, 0x60, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x30, 0x63, 0x63, 0x36, 0x36, 0x03, 0x03, 0x30, 0xFF, 0x00, 0x00, 0x00,
    0x00, 0x06, 0x00, 0x00, 0x50, 0x50, 0x61, 0x61, 0x62, 0x62, 0x53, 0x53,
    0x03, 0xFF, 0x06, 0x02, 0x02, 0x60, 0x60, 0x64, 0x64, 0x06, 0x24, 0x46,
    0x46, 0x66, 0xFF, 0x06, 0x00, 0x00, 0x60, 0x60, 0x62, 0x62, 0x02, 0x32,
    0x66, 0xFF, 0x00, 0x00, 0x60, 0x00, 0x00, 0x66, 0x66, 0x06, 0xFF, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x60, 0x30, 0x36, 0xFF, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x06, 0x66, 0x66, 0x60,
    0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x02, 0x36, 0x36,
    0x62, 0x62, 0x60, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x16, 0x16, 0x56,
    0x56, 0x60, 0x32, 0x36, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x66, 0x60,
    0x06, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x32,
    0x32, 0x60, 0x32, 0x36, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x60, 0x60, 0x06, 0x06, 0x66, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x03, 0x63, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x06, 0x66, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x03, 0x22, 0x22, 0x44, 0x44, 0x63, 0xFF, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x63, 0x63, 0x06, 0xFF, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x50, 0x16, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x22, 0x22, 0x24, 0x24, 0x46, 0xFF,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x42, 0x42, 0x44, 0x44, 0x26,
    0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x44, 0x26, 0xFF, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x60, 0x03, 0x03, 0x66,
    0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0x36, 0x03,
    0x63, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x61, 0x01,
    0x01, 0x65, 0x65, 0x05, 0x30, 0x36, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
];

/// `hsd_80392194`: the 2-byte glyph-cell callback of `lbl_80408898`. `src` is the table entry,
/// read as native bytes like the C does.
///
/// # Safety
/// `dst` must be writable for 2 bytes and `src` readable for 3.
pub unsafe extern "C" fn hsd_80392194(dst: *mut u8, flags: i32, _unused1: i32, _unused2: i32, src: *const u8) {
    melee_core::ct!("hsd_80392194");
    *dst = *src;
    let b = if flags & 1 != 0 { *src.add(2) } else { *src.add(1) };
    *dst.add(1) = b;
}

/// `lbl_80408898`: default glyph colour table.
pub fn lbl_80408898() -> [GlyphEntry; 4] {
    let e = |x0: u32| GlyphEntry { x0: x0 as i32, callback: Some(hsd_80392194) };
    [e(0x10808000), e(0x46808000), e(0x7C808000), e(0xB3808000)]
}

/// The globals of hsd_3915.c (`lbl_804D6070/74/78`; `numPeakParticles` is never referenced).
#[derive(Clone, Copy, Debug)]
pub struct Hsd3915 {
    pub lbl_804D6070: f32,
    pub lbl_804D6074: f32,
    pub lbl_804D6078: u8,
}

impl Default for Hsd3915 {
    fn default() -> Self {
        Hsd3915 { lbl_804D6070: 12.0, lbl_804D6074: 12.0, lbl_804D6078: 12 }
    }
}

/// `DrawRectangle`.
pub fn DrawRectangle<H: Hsd3915Host>(h: &mut H, x_min: f32, y_min: f32, w: f32, hgt: f32, color: &GXColor) {
    melee_core::ct!("DrawRectangle");
    h.GXBegin(0x80, 0, 4);
    let x_max = x_min + w;
    let y_max = y_min + hgt;
    h.GXPosition2f32(x_min, y_min);
    h.GXColor4u8(color.r, color.g, color.b, color.a);
    h.GXPosition2f32(x_max, y_min);
    h.GXColor4u8(color.r, color.g, color.b, color.a);
    h.GXPosition2f32(x_max, y_max);
    h.GXColor4u8(color.r, color.g, color.b, color.a);
    h.GXPosition2f32(x_min, y_max);
    h.GXColor4u8(color.r, color.g, color.b, color.a);
}

/// `f32 * double-literal + f32` contracted to a double `fma`, narrowed to f32.
#[inline(always)]
fn fma_lit(l: f32, lit: f64, x: f32) -> f32 {
    (l as f64).mul_add(lit, x as f64) as f32
}

impl Hsd3915 {
    /// `DrawASCII`: draws one character at (x, y); returns the advance.
    pub fn DrawASCII<H: Hsd3915Host>(&mut self, h: &mut H, chr: i32, x: f32, y: f32, color: &GXColor) -> f32 {
        let index: i32;
        if chr >= '0' as i32 && chr <= '9' as i32 {
            index = chr - '0' as i32;
        } else if chr >= 'A' as i32 && chr <= 'Z' as i32 {
            index = chr - 'A' as i32 + 10;
        } else if chr >= 'a' as i32 && chr <= 'z' as i32 {
            index = chr - 'a' as i32 + 10;
        } else {
            match u8::try_from(chr).ok().map(|c| c as char) {
                Some('.') => {
                    h.GXBegin(0xB8, 0, 1);
                    let px = fma_lit(self.lbl_804D6070, 0.3, x);
                    let py = fma_lit(self.lbl_804D6074, 0.9, y);
                    h.GXPosition2f32(px, py);
                    h.GXColor4u8(color.r, color.g, color.b, color.a);
                    return self.lbl_804D6070;
                }
                Some(':') => {
                    h.GXBegin(0xB8, 0, 2);
                    h.GXPosition2f32(fma_lit(self.lbl_804D6070, 0.3, x), fma_lit(self.lbl_804D6074, 0.3, y));
                    h.GXColor4u8(color.r, color.g, color.b, color.a);
                    h.GXPosition2f32(fma_lit(self.lbl_804D6070, 0.3, x), fma_lit(self.lbl_804D6074, 0.7, y));
                    h.GXColor4u8(color.r, color.g, color.b, color.a);
                    return self.lbl_804D6070;
                }
                Some('-') => index = 0x24,
                Some('_') => index = 0x25,
                Some('~') => index = 0x26,
                Some('>') => index = 0x27,
                Some('/') => index = 0x28,
                Some('(') => index = 0x29,
                Some(')') => index = 0x2A,
                Some(',') => index = 0x2B,
                Some('<') => index = 0x2C,
                Some('+') => index = 0x2D,
                Some('$') => index = 0x2E,
                _ => return self.lbl_804D6070,
            }
        }

        let glyph = &LBL_80408630[(index * 13) as usize..];
        let mut i = 0usize;
        while i < 0x29 && glyph[i] != 0xFF {
            let p0 = glyph[i];
            i += 1;
            let p1 = glyph[i];
            i += 1;
            h.GXBegin(0xA8, 0, 2);
            h.GXPosition2f32(
                fmadds(self.lbl_804D6070, 0.11f32 * (p0 >> 4) as f32, x),
                fmadds(self.lbl_804D6074, 0.11f32 * (p0 & 0xF) as f32, y),
            );
            h.GXColor4u8(color.r, color.g, color.b, color.a);
            h.GXPosition2f32(
                fmadds(self.lbl_804D6070, 0.11f32 * (p1 >> 4) as f32, x),
                fmadds(self.lbl_804D6074, 0.11f32 * (p1 & 0xF) as f32, y),
            );
            h.GXColor4u8(color.r, color.g, color.b, color.a);
        }
        self.lbl_804D6070
    }

    /// `hsd_80391A04`: set the font scale / line width and the GX state for line drawing.
    pub fn hsd_80391A04<H: Hsd3915Host>(&mut self, h: &mut H, scale_x: f32, scale_y: f32, line_width: i32) {
        let mut view_mtx: Mtx = [[0.0; 4]; 3];
        self.lbl_804D6070 = scale_x;
        self.lbl_804D6074 = scale_y;
        self.lbl_804D6078 = line_width as u8;
        h.HSD_ClearVtxDesc();
        h.GXSetCurrentMtx(0);
        let cobj = h.HSD_CObjGetCurrent();
        h.HSD_CObjGetViewingMtx(cobj, &mut view_mtx);
        h.GXLoadPosMtxImm(&view_mtx, 0);
        h.HSD_StateSetLineWidth(self.lbl_804D6078, 5);
        h.HSD_StateSetPointSize(self.lbl_804D6078 as i32 * 2, 5);
        h.HSD_SetupRenderMode(0x6800_0002);
        h.GXSetVtxAttrFmt(GX_VTXFMT0, GX_VA_POS, GX_TEX_S, GX_RGBA6, 0);
        h.GXSetVtxAttrFmt(GX_VTXFMT0, GX_VA_CLR0, GX_TEX_ST, GX_RGBA8, 0);
        h.GXSetVtxDesc(GX_VA_POS, GX_DIRECT);
        h.GXSetVtxDesc(GX_VA_CLR0, GX_DIRECT);
        h.HSD_StateSetCullMode(0);
    }

    /// `hsd_80391AC8`: draws `str` (NUL terminated; `\cRRGGBB` switches colour) and returns the
    /// final x as an integer. A NULL `str` returns `(s32) x`.
    ///
    /// # Safety
    /// `str` is null or NUL terminated; `color` is writable.
    pub unsafe fn hsd_80391AC8<H: Hsd3915Host>(&mut self, h: &mut H, str: *const i8, color: *mut GXColor, mut x: f32, y: f32) -> i32 {
        if str.is_null() {
            return melee_core::ppc::fctiwz_f32(x);
        }
        let mut i = 0isize;
        while *str.offset(i) != 0 {
            if *str.offset(i) != b'\\' as i8 {
                let col = *color;
                x += self.DrawASCII(h, *str.offset(i) as i32, x, y, &col);
            } else {
                i += 1;
                match *str.offset(i) as u8 {
                    b'c' | b'C' => {
                        let p = str.offset(i) as *const u8;
                        (*color).r = hexval(*p.add(2) as i32).wrapping_add(hexval(*p.add(1) as i32).wrapping_mul(16)) as u8;
                        (*color).g = hexval(*p.add(4) as i32).wrapping_add(hexval(*p.add(3) as i32).wrapping_mul(16)) as u8;
                        (*color).b = hexval(*p.add(6) as i32).wrapping_add(hexval(*p.add(5) as i32).wrapping_mul(16)) as u8;
                        i += 6;
                        (*color).a = 0xFF;
                    }
                    _ => {}
                }
            }
            i += 1;
        }
        melee_core::ppc::fctiwz_f32(x)
    }
}

/// `hexval` (static inline): `0..9`, `A..F`, `a..f` (as C's range tests allow), else 0.
fn hexval(ch: i32) -> i32 {
    if ch < 0x47 {
        if ch < 0x0A {
            if ch >= 0 { ch - 0x30 } else { 0 }
        } else if ch >= 0x41 {
            ch - 0x37
        } else {
            0
        }
    } else if ch < 0x67 {
        if ch >= 0x61 { ch - 0x57 } else { 0 }
    } else {
        0
    }
}

/// `hsd_80391E18`: draws a coloured polyline from (x1, y1) to (x2, y2) described by a list of
/// `{ f32 t; u32 color }` records, terminated by `t < 0`.
///
/// # Safety
/// `list` is null or a terminated record list.
pub unsafe fn hsd_80391E18<H: Hsd3915Host>(h: &mut H, mut list: *const u8, x1: f32, y1: f32, x2: f32, y2: f32) {
    melee_core::ct!("hsd_80391E18");
    if list.is_null() {
        return;
    }
    let mut prev_x = x1;
    let mut prev_y = y1;
    let dx = x2 - x1;
    let dy = y2 - y1;
    loop {
        let t: f64 = (list as *const f32).read_unaligned() as f64;
        let color: u32 = (list.add(4) as *const u32).read_unaligned();
        list = list.add(8);
        if t < 0.0 {
            break;
        }
        h.GXBegin(0xA8, 0, 2);
        h.GXPosition2f32(prev_x, prev_y);
        prev_x = t.mul_add(dx as f64, x1 as f64) as f32;
        prev_y = t.mul_add(dy as f64, y1 as f64) as f32;
        let b = color.to_ne_bytes();
        h.GXColor4u8(b[0], b[1], b[2], b[3]);
        h.GXPosition2f32(prev_x, prev_y);
        h.GXColor4u8(b[0], b[1], b[2], b[3]);
    }
}

/// `hsd_80391F28_len` (static inline): Newton-Raphson `sqrtf`, pinned to f32 by a volatile store.
fn hsd_80391F28_len(dy: f32, dx: f32) -> f32 {
    // `(dx*dx) + (dy*dy)`: LHS multiply fused (fmadds), RHS multiply rounded to f32 first.
    let mut len = fmadds(dx, dx, dy * dy);
    if len > 0.0 {
        let l = len as f64;
        let mut e = frsqrte(l);
        // `e = 0.5 * e * -(len * (e * e) - 3.0)`: inner `len * (e*e) - 3.0` is a double fma.
        for _ in 0..3 {
            e = (0.5 * e) * (-l.mul_add(e * e, -3.0));
        }
        e *= l;
        len = core::hint::black_box(e as f32);
    }
    len
}

/// `hsd_80391F28`: line from (x1, y1) to (x2, y2) with a tick every `1/count`; every fifth longer.
pub fn hsd_80391F28<H: Hsd3915Host>(h: &mut H, color: &GXColor, x1: f32, y1: f32, x2: f32, y2: f32, count: f32) {
    melee_core::ct!("hsd_80391F28");
    let dy = y2 - y1;
    let dx = x2 - x1;
    let len = hsd_80391F28_len(dy, dx);
    let perp_x = -dy / len;
    let perp_y = -dx / len;

    h.GXBegin(0xA8, 0, 2);

    let tick6_y = 6.0f32 * perp_y;
    let tick6_x = 6.0f32 * perp_x;
    let tick4_x = 4.0f32 * perp_x;
    let tick4_y = 4.0f32 * perp_y;

    h.GXPosition2f32(x1, y1);
    h.GXColor4u8(color.r, color.g, color.b, color.a);
    h.GXPosition2f32(x2, y2);
    h.GXColor4u8(color.r, color.g, color.b, color.a);

    let mut x = x1;
    let mut y = y1;
    let mut i: i32 = 0;
    while (i as f32) <= count {
        h.GXBegin(0xA8, 0, 2);
        h.GXPosition2f32(x, y);
        h.GXColor4u8(color.r, color.g, color.b, color.a);
        if i % 5 == 0 {
            h.GXPosition2f32(x + tick6_x, y + tick6_y);
        } else {
            h.GXPosition2f32(x + tick4_x, y + tick4_y);
        }
        h.GXColor4u8(color.r, color.g, color.b, color.a);
        x += dx / count;
        y += dy / count;
        i = i.wrapping_add(1);
    }
}

/// Shared body of `hsd_803921B8` / `hsd_803922FC`'s inner pixel loop.
///
/// # Safety
/// See [`hsd_803921B8`].
#[allow(clippy::too_many_arguments)]
unsafe fn glyph_row(
    bitmap: *const u8, data_off: i32, off_x: i32, x: i32, y: i32, max_x: u32, start_dst: i32, table: *const GlyphEntry,
) {
    let mut col = x;
    let mut bit_x = off_x;
    let mut cur_dst = start_dst;
    while (col as u32) < max_x {
        let word = (bitmap
            .offset(data_off as isize)
            .offset((((bit_x as u32) >> 2) & 0x3FFF_FFFC) as isize) as *const u32)
            .read_unaligned();
        let mut shift = cur_dst;
        let mut bit_off = bit_x & 0xF;
        while bit_off < 16 && (col as u32) < max_x {
            let val = ((<u32 as melee_core::ppc::Ppc>::pshr(word, ((((15 - bit_off) * 2) as u32 & 31)) as i64)) & 3) as i32;
            let entry = table.offset(val as isize);
            ((*entry).callback.expect("GlyphEntry.callback"))(shift as isize as *mut u8, col, y, val, entry as *const u8);
            bit_off += 1;
            bit_x = bit_x.wrapping_add(1);
            shift = shift.wrapping_add(2);
            cur_dst = cur_dst.wrapping_add(2);
            col = col.wrapping_add(1);
        }
    }
}

/// `hsd_803921B8`: blits a 14-row, 2-bit-per-pixel glyph bitmap into a framebuffer. `dst` is the
/// C `s32` address (sign-extended to a pointer for the per-pixel callback, as a 64-bit C build
/// does); `tbl` NULL selects [`lbl_80408898`].
///
/// # Safety
/// `bitmap` has 14 rows of 4-byte words readable; `dst`/`stride`/`w`/`h` describe a writable
/// framebuffer; `tbl` is null or 4 [`GlyphEntry`].
pub unsafe fn hsd_803921B8(bitmap: *const u8, mut x: i32, mut y: i32, dst: i32, w: i32, h: i32, stride: i32, tbl: *const GlyphEntry) {
    melee_core::ct!("hsd_803921B8");
    let default_tbl = lbl_80408898();
    let table: *const GlyphEntry = if tbl.is_null() { default_tbl.as_ptr() } else { tbl };
    let mut off_y: i32 = 0;
    let mut off_x: i32 = 0;
    if y < 0 {
        off_y = y.wrapping_neg();
        y = 0;
    }
    let max_x: u32 = if (x.wrapping_add(11) as u32) < w as u32 { x.wrapping_add(11) as u32 } else { w as u32 };
    if x < 0 {
        off_x = x.wrapping_neg();
        x = 0;
    }
    let x2 = x.wrapping_mul(2);
    let mut data_off = off_y.wrapping_mul(4);
    while off_y < 14 && (y as u32) < h as u32 {
        let start = dst.wrapping_add((y as u32).wrapping_mul(stride as u32) as i32).wrapping_add(x2);
        glyph_row(bitmap, data_off, off_x, x, y, max_x, start, table);
        off_y += 1;
        data_off = data_off.wrapping_add(4);
        y += 1;
    }
}

/// `hsd_803922FC`: as [`hsd_803921B8`] but writes only every other row (`parity` picks which).
///
/// # Safety
/// As [`hsd_803921B8`].
pub unsafe fn hsd_803922FC(bitmap: *const u8, mut x: i32, mut y: i32, parity: i32, dst: i32, w: i32, h: i32, stride: i32, tbl: *const GlyphEntry) {
    melee_core::ct!("hsd_803922FC");
    let default_tbl = lbl_80408898();
    let table: *const GlyphEntry = if tbl.is_null() { default_tbl.as_ptr() } else { tbl };
    let mut off_y: i32 = 0;
    let mut off_x: i32 = 0;
    if y < 0 {
        off_y = y.wrapping_neg();
        y = 0;
    }
    if y.wrapping_rem(2) != parity {
        y += 1;
        off_y += 1;
    }
    let max_x: u32 = if (x.wrapping_add(11) as u32) < w as u32 { x.wrapping_add(11) as u32 } else { w as u32 };
    if x < 0 {
        off_x = x.wrapping_neg();
        x = 0;
    }
    let mut row_idx = y / 2;
    let x2 = x.wrapping_mul(2);
    let mut data_off = off_y.wrapping_mul(4);
    while off_y < 14 && (y as u32) < h as u32 {
        let start = dst.wrapping_add((row_idx as u32).wrapping_mul(stride as u32) as i32).wrapping_add(x2);
        glyph_row(bitmap, data_off, off_x, x, y, max_x, start, table);
        off_y += 2;
        data_off = data_off.wrapping_add(8);
        row_idx += 1;
        y += 2;
    }
}
