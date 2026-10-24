//! Port of src/sysdolphin/baselib/hsd_3A76.c: the SIS text renderer. `HSD_SisLib_803A84BC` walks a
//! text's SIS stream (see hsd_3A64.rs) and draws one GX quad per glyph; `HSD_SisLib_803A8134`
//! measures a line, `HSD_SisLib_803A7684` / `HSD_SisLib_803A7F0C` push and pop the text's
//! style stack (`string_buffer`, `x6C` entries), and the three data tables of the TU
//! (`HSD_SisLib_8040C680`, `lbl_8040C8C0`, `HSD_SisLib_8040CB00`) map ASCII/Shift-JIS to glyph
//! codes and give default kerning.
//!
//! Notes on the native port:
//! * Stream operands are big-endian bytes (the GC reads them through `u16*`/`s16*`/`s32*`);
//!   they are read with `from_be_bytes` here.
//! * The stream embeds 32-bit GameCube pointers (opcodes 8/9, the style-stack entry type 5). A
//!   native pointer does not fit, so the conversions are the TODO stubs `ext::gc_ptr_to_u32` /
//!   `ext::gc_u32_to_ptr` (the host's memory model decides).
//! * `HSD_SisLib_803A84BC(NULL, text_as_int)` passes a pointer in the `int pass`; `pass` is
//!   converted with `gc_u32_to_ptr`.
//! * Float: every `a*b + c` the shipping build contracts is a `fmadds` here (19 f32 sites and one
//!   f64 fma in `803A84BC`, 4 in `803A8134`; `tools/rust-port/fma_sites.py` counts).
//!
//! Callees in unported TUs are TODO stubs in [`ext`], named like the C.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use super::hsd_3A64::ext::{HSD_SisLib_804D1124, HSD_SisLib_Alloc, HSD_SisLib_Free};
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::{fmadds, fmsubs};
use melee_types::*;

pub mod ext {
    #![allow(non_snake_case, unused_variables)]
    use super::*;
    /// `HSD_SisLib_FontAtlas` base (sislib_font.c): 287 glyph textures of 0x200 bytes.
    pub static mut HSD_SisLib_FontAtlas: *mut u8 = core::ptr::null_mut();
    /// A GameCube pointer stored in a SIS stream (host memory model).
    pub fn gc_ptr_to_u32(p: *const u8) -> u32 { todo!("gc_ptr_to_u32 (guest address of a SIS stream position)") }
    pub fn gc_u32_to_ptr(a: u32) -> *mut u8 { todo!("gc_u32_to_ptr (SIS stream position of a guest address)") }
    pub fn HSD_GObjGetUserData(gobj: *mut HSD_GObj) -> *mut c_void { unsafe { melee_types::inline::HSD_GObjGetUserData(&*(gobj as *const _)) as _ } }
    pub fn Mex_Enabled(name: &str) -> bool { todo!("Mex_Enabled (pc/platform mex)") }
    pub fn HSD_StateSetZMode(enable: u32, func: u32, update: u32) { todo!("HSD_StateSetZMode (baselib/state.c)") }
    pub fn HSD_CObjGetCurrent() -> *mut HSD_CObj { unsafe { crate::tu::cobj::HSD_CObjGetCurrent() } }
    pub fn HSD_CObjGetViewingMtx(cobj: *mut HSD_CObj, mtx: *mut [f32; 4]) { todo!("HSD_CObjGetViewingMtx (baselib/cobj.c)") }
    pub fn GXSetZMode(a: u32, b: u32, c: u32) { todo!("GXSetZMode (dolphin/gx)") }
    pub fn GXSetViewport(l: f32, t: f32, w: f32, h: f32, n: f32, f: f32) { todo!("GXSetViewport (dolphin/gx)") }
    pub fn GXSetScissor(l: u32, t: u32, w: u32, h: u32) { todo!("GXSetScissor (dolphin/gx)") }
    pub fn MTXOrtho(mtx: *mut [f32; 4], t: f32, b: f32, l: f32, r: f32, n: f32, f: f32) { todo!("MTXOrtho (dolphin/mtx)") }
    pub fn GXSetProjection(mtx: *mut [f32; 4], ty: u32) { todo!("GXSetProjection (dolphin/gx)") }
    pub fn GXSetCurrentMtx(id: u32) { todo!("GXSetCurrentMtx (dolphin/gx)") }
    pub fn GXLoadPosMtxImm(mtx: *mut [f32; 4], id: u32) { todo!("GXLoadPosMtxImm (dolphin/gx)") }
    pub fn GXSetTexCoordGen2(coord: u32, ty: u32, src: u32, mtx: u32, normalize: u32, post_mtx: u32) { todo!("GXSetTexCoordGen2 (dolphin/gx)") }
    pub fn HSD_StateSetNumChans(n: u32) { todo!("HSD_StateSetNumChans (baselib/state.c)") }
    pub fn GXSetNumChans(n: u32) { todo!("GXSetNumChans (dolphin/gx)") }
    pub fn GXClearVtxDesc() { todo!("GXClearVtxDesc (dolphin/gx)") }
    pub fn GXSetCullMode(mode: u32) { todo!("GXSetCullMode (dolphin/gx)") }
    pub fn GXSetNumTexGens(n: u32) { todo!("GXSetNumTexGens (dolphin/gx)") }
    pub fn GXSetNumTevStages(n: u32) { todo!("GXSetNumTevStages (dolphin/gx)") }
    pub fn GXSetVtxAttrFmt(fmt: u32, attr: u32, cnt: u32, ty: u32, frac: u32) { todo!("GXSetVtxAttrFmt (dolphin/gx)") }
    pub fn GXSetVtxDesc(attr: u32, ty: u32) { todo!("GXSetVtxDesc (dolphin/gx)") }
    pub fn GXSetTevOrder(stage: u32, coord: u32, map: u32, color: u32) { todo!("GXSetTevOrder (dolphin/gx)") }
    pub fn GXSetBlendMode(ty: u32, src: u32, dst: u32, op: u32) { todo!("GXSetBlendMode (dolphin/gx)") }
    pub fn GXSetAlphaCompare(c0: u32, r0: u32, op: u32, c1: u32, r1: u32) { todo!("GXSetAlphaCompare (dolphin/gx)") }
    pub fn GXSetColorUpdate(on: u32) { todo!("GXSetColorUpdate (dolphin/gx)") }
    pub fn GXSetAlphaUpdate(on: u32) { todo!("GXSetAlphaUpdate (dolphin/gx)") }
    pub fn GXSetTevColorIn(stage: u32, a: u32, b: u32, c: u32, d: u32) { todo!("GXSetTevColorIn (dolphin/gx)") }
    pub fn GXSetTevColorOp(stage: u32, op: u32, bias: u32, scale: u32, clamp: u32, out: u32) { todo!("GXSetTevColorOp (dolphin/gx)") }
    pub fn GXSetTevAlphaOp(stage: u32, op: u32, bias: u32, scale: u32, clamp: u32, out: u32) { todo!("GXSetTevAlphaOp (dolphin/gx)") }
    pub fn GXSetTevAlphaIn(stage: u32, a: u32, b: u32, c: u32, d: u32) { todo!("GXSetTevAlphaIn (dolphin/gx)") }
    pub fn GXSetTevColor(id: u32, color: GXColor) { todo!("GXSetTevColor (dolphin/gx)") }
    pub fn GXSetTevKAlphaSel(stage: u32, sel: u32) { todo!("GXSetTevKAlphaSel (dolphin/gx)") }
    pub fn GXSetTevKColor(id: u32, color: GXColor) { todo!("GXSetTevKColor (dolphin/gx)") }
    pub fn GXBegin(prim: u32, fmt: u32, n: u16) { todo!("GXBegin (dolphin/gx)") }
    pub fn GXPosition3f32(x: f32, y: f32, z: f32) { todo!("GXPosition3f32 (dolphin/gx)") }
    pub fn GXTexCoord2f32(s: f32, t: f32) { todo!("GXTexCoord2f32 (dolphin/gx)") }
    pub fn GXEnd() { unsafe { crate::tu::sobjlib::GXEnd() } }
    pub fn GXInitTexObj(obj: *mut GXTexObj, data: *mut u8, w: u16, h: u16, fmt: u32, wrap_s: u32, wrap_t: u32, mipmap: u8) { todo!("GXInitTexObj (dolphin/gx)") }
    pub fn GXLoadTexObj(obj: *mut GXTexObj, map: u32) { todo!("GXLoadTexObj (dolphin/gx)") }
    pub fn HSD_StateInvalidate(mask: i32) { todo!("HSD_StateInvalidate (baselib/state.c)") }
    pub fn lbArchive_LoadArchive(path: *mut i8) -> *mut HSD_Archive { unsafe { crate::tu::lbarchive::lbArchive_LoadArchive(path as _) } }
    pub fn lbArchive_80016EFC(archive: *mut HSD_Archive) { unsafe { crate::tu::lbarchive::lbArchive_80016EFC(archive) } }
}

/// `ARRAY_SIZE(HSD_SisLib_FontAtlas)` (sislib_font.h).
pub const HSD_SisLib_FontAtlas_COUNT: usize = 287;

#[inline]
unsafe fn be16(p: *const u8) -> u16 {
    u16::from_be_bytes([*p, *p.add(1)])
}

#[inline]
unsafe fn be32(p: *const u8) -> u32 {
    u32::from_be_bytes([*p, *p.add(1), *p.add(2), *p.add(3)])
}

#[inline]
fn f32_to_u8(v: f32) -> u8 {
    (v as i32) as u8
}

/// `HSD_SisLib_GlyphWidth` (static inline).
#[inline]
fn HSD_SisLib_GlyphWidth(text: &HSD_Text, scale_x: f32) -> f32 {
    32.0f32 * text.x80.x * scale_x
}

/// Grow `text->string_buffer` by 0x10 if fewer than `need` bytes remain after `x6C` (the shared
/// prologue of every `HSD_SisLib_803A7684` case).
unsafe fn ensure_style_room(text: *mut HSD_Text, need: i32) {
    let old_x6e = (*text).x6E as i32;
    if old_x6e < (*text).x6C as i32 + need {
        let new_x6e = old_x6e + 0x10;
        let old_buf = (*text).string_buffer as *mut u8;
        (*text).string_buffer = HSD_SisLib_Alloc(new_x6e) as *mut i8;
        (*text).x6E = new_x6e as u16;
        let count = old_x6e as u32;
        let mut idx: i32 = 0;
        while (idx as u32) < count {
            *(*text).string_buffer.offset(idx as isize) = *old_buf.offset(idx as isize) as i8;
            idx += 1;
        }
        while idx < (*text).x6E as i32 {
            *(*text).string_buffer.offset(idx as isize) = 0;
            idx += 1;
        }
        HSD_SisLib_Free(old_buf as *mut c_void);
    }
}

/// `text->string_buffer[text->x6C++] = b`.
#[inline]
unsafe fn push_style_byte(text: *mut HSD_Text, b: u8) {
    *(*text).string_buffer.offset((*text).x6C as isize) = b as i8;
    (*text).x6C = (*text).x6C.wrapping_add(1);
}

/// `HSD_SisLib_803A7684` (0x803A7684): push a style entry of type `flags & 0x7F` on the text's
/// style stack (type 5 stores the stream position `cursor`).
pub unsafe fn HSD_SisLib_803A7684(text: *mut HSD_Text, cursor: *const u8, flags: u8) {
    match flags & 0x7F {
        1 => {
            ensure_style_room(text, 5);
            let x = (*text).x78.x;
            let y = (*text).x78.y;
            push_style_byte(text, ((256.0f32 * x) as i32 >> 8) as u8);
            push_style_byte(text, f32_to_u8(256.0f32 * x));
            push_style_byte(text, ((256.0f32 * y) as i32 >> 8) as u8);
            push_style_byte(text, f32_to_u8(256.0f32 * y));
            push_style_byte(text, flags);
        }
        2 => {
            ensure_style_room(text, 4);
            push_style_byte(text, (*text).active_color.r);
            push_style_byte(text, (*text).active_color.g);
            push_style_byte(text, (*text).active_color.b);
            push_style_byte(text, flags);
        }
        3 => {
            ensure_style_room(text, 5);
            let x = (*text).x80.x;
            let y = (*text).x80.y;
            push_style_byte(text, ((256.0f32 * x) as i32 >> 8) as u8);
            push_style_byte(text, f32_to_u8(256.0f32 * x));
            push_style_byte(text, ((256.0f32 * y) as i32 >> 8) as u8);
            push_style_byte(text, f32_to_u8(256.0f32 * y));
            push_style_byte(text, flags);
        }
        4 => {
            ensure_style_room(text, 2);
            push_style_byte(text, (*text).alignment);
            push_style_byte(text, flags);
        }
        5 => {
            ensure_style_room(text, 5);
            let a = ext::gc_ptr_to_u32(cursor);
            push_style_byte(text, (a >> 0x18) as u8);
            push_style_byte(text, ((a >> 0x10) & 0xFF) as u8);
            push_style_byte(text, ((a >> 8) & 0xFF) as u8);
            push_style_byte(text, a as u8);
            push_style_byte(text, flags);
        }
        _ => {}
    }
}

/// `HSD_SisLib_803A7F0C` (0x803A7F0C): pop the newest style entry of type `flags & 0x7F`,
/// restoring the text's state from it (and removing it if `flags & 0x80` matches the entry's).
/// Returns the saved stream position for type 5, else 0.
pub unsafe fn HSD_SisLib_803A7F0C(text: *mut HSD_Text, flags: i32) -> i32 {
    let flag_hi = flags & 0x80;
    let target_type = flags & 0x7F;
    let mut pos: i32 = (*text).x6C as i32;
    let mut result: i32 = 0;
    let mut remove_size: i32 = 0;
    let buf = (*text).string_buffer as *mut u8;
    while pos >= 0 {
        let entry = *buf.offset(pos as isize) as i8;
        let entry_flags = (entry as i32) & 0x80;
        match (entry as u8) & 0x7F {
            1 => {
                pos -= 4;
                if target_type == 1 {
                    (*text).x78.x = (be16(buf.offset(pos as isize)) as i16) as f32 / 256.0f32;
                    (*text).x78.y = (be16(buf.offset(pos as isize + 2)) as i16) as f32 / 256.0f32;
                    if flag_hi == entry_flags {
                        remove_size = 5;
                    }
                    break;
                }
            }
            2 => {
                pos -= 3;
                if target_type == 2 {
                    (*text).active_color.r = *buf.offset(pos as isize);
                    (*text).active_color.g = *buf.offset(pos as isize + 1);
                    (*text).active_color.b = *buf.offset(pos as isize + 2);
                    if flag_hi == entry_flags {
                        remove_size = 4;
                    }
                    break;
                }
            }
            3 => {
                pos -= 4;
                if target_type == 3 {
                    (*text).x80.x = be16(buf.offset(pos as isize)) as f32 / 256.0f32;
                    (*text).x80.y = be16(buf.offset(pos as isize + 2)) as f32 / 256.0f32;
                    if flag_hi == entry_flags {
                        remove_size = 5;
                    }
                    break;
                }
            }
            4 => {
                pos -= 1;
                if target_type == 4 {
                    (*text).alignment = *buf.offset(pos as isize);
                    if flag_hi == entry_flags {
                        remove_size = 2;
                    }
                    break;
                }
            }
            5 => {
                pos -= 4;
                if target_type == 5 {
                    result = be32(buf.offset(pos as isize)) as i32;
                    if flag_hi == entry_flags {
                        remove_size = 5;
                    }
                    break;
                }
            }
            _ => {}
        }
        pos -= 1;
    }
    if remove_size != 0 {
        while pos + remove_size < (*text).x6C as i32 {
            *buf.offset(pos as isize) = *buf.offset((pos + remove_size) as isize);
            pos += 1;
        }
        while pos < (*text).x6C as i32 {
            *buf.offset(pos as isize) = 0;
            pos += 1;
        }
        (*text).x6C = (*text).x6C.wrapping_sub(remove_size as u16);
    }
    result
}

/// Kerning pair `(left, right)` for glyph code `glyph_code` (< 0x4000: the default table;
/// else the font's `textures` array read as a flat byte array, as the C does).
#[inline]
unsafe fn kern_pair(glyph_code: u32, glyph_tex: *const u8) -> (i32, i32) {
    if glyph_code < 0x4000 {
        let idx = (((glyph_code as i32 - 0x2000) * 2) & 0x1FFFE) as usize;
        (HSD_SisLib_8040CB00[idx] as i32, HSD_SisLib_8040CB00[idx + 1] as i32)
    } else {
        let idx = (((glyph_code as i32 - 0x4000) * 2) & 0x1FFFE) as usize;
        (*glyph_tex.add(idx) as i32, *glyph_tex.add(idx + 1) as i32)
    }
}

/// `HSD_SisLib_803A8134` (0x803A8134): measure the line starting at `cursor`. Writes the width
/// and height to `out_width` / `out_height`; style-stack changes made while measuring are undone.
pub unsafe fn HSD_SisLib_803A8134(mut cursor: *mut u8, text: *mut HSD_Text, out_width: &mut f32, out_height: &mut f32) {
    let saved_scale_x = (*text).x80.x;
    let saved_scale_y = (*text).x80.y;
    let saved_spacing = (*text).x78.x;
    let mut kern_enabled: u16 = (*text).kerning as u16;
    let sis = HSD_SisLib_804D1124[(*text).font_idx as usize];
    let saved_x6c = (*text).x6C;
    let mut glyph_tex: *const u8 = core::ptr::null();
    if !sis.is_null() {
        glyph_tex = (*sis).textures as *const u8;
    }
    *out_width = 0.0f32;
    *out_height = 32.0f32 * (*text).x80.y;
    loop {
        let opcode = *cursor;
        // Each arm either `break 'done` (end of line) or leaves the cursor on the last byte of
        // the item (block_33 then steps over it).
        match opcode {
            0 => {
                let pop_result = HSD_SisLib_803A7F0C(text, 0x85) as u32;
                if pop_result != 0 {
                    cursor = ext::gc_u32_to_ptr(pop_result.wrapping_add(4));
                } else {
                    break;
                }
            }
            1 | 2 | 3 | 7 => break,
            9 => {
                HSD_SisLib_803A7684(text, cursor, 0x85);
                cursor = ext::gc_u32_to_ptr(be32(cursor.add(1))).offset(-1);
            }
            8 => {
                cursor = ext::gc_u32_to_ptr(be32(cursor.add(1))).offset(-1);
            }
            14 => {
                HSD_SisLib_803A7684(text, cursor, 0x83);
                (*text).x80.x = be16(cursor.add(1)) as f32 / 256.0f32;
                let scale_val = be16(cursor.add(3));
                cursor = cursor.add(4);
                (*text).x80.y = scale_val as f32 / 256.0f32;
            }
            15 => {
                HSD_SisLib_803A7F0C(text, 0x83);
            }
            6 => cursor = cursor.add(4),
            10 => {
                if (*text).alloc_data.is_null() || kern_enabled == 0 {
                    HSD_SisLib_803A7684(text, cursor, 0x81);
                    (*text).x78.x = (be16(cursor.add(1)) as i16) as f32 / 256.0f32;
                }
                cursor = cursor.add(4);
            }
            11 => {
                if (*text).alloc_data.is_null() || kern_enabled == 0 {
                    HSD_SisLib_803A7F0C(text, 0x81);
                }
            }
            12 => cursor = cursor.add(3),
            5 => cursor = cursor.add(2),
            22 => kern_enabled = 1,
            23 => kern_enabled = 0,
            26 => {
                *out_width = fmadds((*text).x80.x, 16.0f32 + (*text).x78.x, *out_width);
                if *out_height < 32.0f32 * (*text).x80.y {
                    *out_height = 32.0f32 * (*text).x80.y;
                }
            }
            _ => {
                if opcode >= 0x20 {
                    *out_width = fmadds((*text).x80.x, 32.0f32 + (*text).x78.x, *out_width);
                    if kern_enabled != 0 {
                        let glyph_code = be16(cursor) as u32;
                        let (left, right) = kern_pair(glyph_code, glyph_tex);
                        let kern_width = left + (right - 2);
                        *out_width = -fmsubs((*text).x80.x, kern_width as f32, *out_width);
                    }
                    if *out_height < 32.0f32 * (*text).x80.y {
                        *out_height = 32.0f32 * (*text).x80.y;
                    }
                    cursor = cursor.add(1);
                }
            }
        }
        // block_33
        cursor = cursor.add(1);
    }
    // block_done
    *out_width -= (*text).x78.x;
    (*text).x80.x = saved_scale_x;
    (*text).x80.y = saved_scale_y;
    (*text).x78.x = saved_spacing;
    (*text).x6C = saved_x6c;
    let mut clear_idx = (*text).x6C as i32;
    while clear_idx < (*text).x6E as i32 {
        *(*text).string_buffer.offset(clear_idx as isize) = 0;
        clear_idx += 1;
    }
}

/// `sisFitLineToBox` (static).
unsafe fn sisFitLineToBox(text: *mut HSD_Text, measured_width: f32) {
    if (*text).fitting == 1 && (*text).box_size_x < measured_width {
        (*text).current_width = 0.0f32;
        (*text).x88 = (*text).box_size_x / measured_width;
    } else {
        (*text).x88 = 1.0f32;
        match (*text).alignment as i32 {
            1 => (*text).current_width = 0.5f32 * ((*text).box_size_x - measured_width),
            2 => (*text).current_width = (*text).box_size_x - measured_width,
            _ => (*text).current_width = 0.0f32,
        }
    }
}

/// `HSD_SisLib_803A84BC` (0x803A84BC): GObj render callback (`pass == 2` draws) or, with a NULL
/// `gobj`, a direct draw of the text whose guest address is `pass`.
pub unsafe fn HSD_SisLib_803A84BC(gobj: *mut HSD_GObj, pass: i32) {
    let text: *mut HSD_Text;
    if !gobj.is_null() {
        if pass as u32 != 2 {
            return;
        }
        text = ext::HSD_GObjGetUserData(gobj) as *mut HSD_Text;
    } else {
        text = ext::gc_u32_to_ptr(pass as u32) as *mut HSD_Text;
    }
    sis_render_text(gobj, text);
}

/// The body of `HSD_SisLib_803A84BC` once `text` is known.
unsafe fn sis_render_text(gobj: *mut HSD_GObj, text: *mut HSD_Text) {
    let mut tex_obj: GXTexObj = core::mem::zeroed();
    let mut m: Mtx = [[0.0; 4]; 3];
    let mut line_width_out: f32 = 0.0;
    let mut line_height_out: f32 = 0.0;
    let mut kerning: *mut TextKerning = null_mut();
    let mut textures: *mut TextGlyphTexture = null_mut();

    let mut min_x: f32 = 0.0;
    let mut min_y: f32 = 0.0;
    let mut max_x: f32 = 0.0;
    let mut max_y: f32 = 0.0;

    let data: *mut u8 = ext::HSD_SisLib_FontAtlas;
    let default_kerning: *const u8 = HSD_SisLib_8040CB00.as_ptr();

    if (*text).hidden == 0 && !(*text).sis_buffer.is_null() {
        let mut sis_cursor = (*text).sis_buffer as *mut u8;
        if !gobj.is_null() {
            let sis = HSD_SisLib_804D1124[(*text).font_idx as usize];
            if !sis.is_null() {
                kerning = (*sis).kerning;
                textures = (*sis).textures;
            }
        }
        if !gobj.is_null() {
            if (*text).x4C != 0 {
                ext::HSD_StateSetZMode(1, GX_LEQUAL, 0);
            } else {
                ext::HSD_StateSetZMode(0, GX_NEVER, 0);
            }
            ext::HSD_CObjGetViewingMtx(ext::HSD_CObjGetCurrent(), m.as_mut_ptr());
        } else {
            // MTXOrtho writes four rows; GXSetProjection reads them on PC.
            let mut projection_m: Mtx44 = [[0.0; 4]; 4];
            ext::GXSetZMode(0, 0, 0);
            ext::GXSetViewport(0.0, 0.0, 640.0, 480.0, 0.0, 1.0);
            ext::GXSetScissor(0, 0, 0x280, 0x1E0);
            ext::MTXOrtho(projection_m.as_mut_ptr(), 0.0, -480.0, 0.0, 640.0, 0.0, 2.0);
            ext::GXSetProjection(projection_m.as_mut_ptr(), 0);
            m[0] = [1.0, 0.0, 0.0, 0.0];
            m[1] = [0.0, 1.0, 0.0, 0.0];
            m[2] = [0.0, 0.0, 1.0, -1.0];
        }
        ext::GXSetCurrentMtx(0);
        ext::GXLoadPosMtxImm(m.as_mut_ptr(), 0);
        ext::GXSetTexCoordGen2(GX_TEXCOORD0, GX_TG_MTX2x4, GX_TG_TEX0, GX_IDENTITY, 0, GX_PTIDENTITY);
        if !gobj.is_null() {
            ext::HSD_StateSetNumChans(0);
        } else {
            ext::GXSetNumChans(0);
        }
        ext::GXClearVtxDesc();
        ext::GXSetCullMode(GX_CULL_NONE);
        ext::GXSetNumTexGens(1);
        ext::GXSetNumTevStages(1);
        ext::GXSetVtxAttrFmt(GX_VTXFMT0, GX_VA_POS, GX_POS_XYZ, GX_RGBA6, 0);
        ext::GXSetVtxDesc(GX_VA_POS, GX_DIRECT);
        ext::GXSetVtxAttrFmt(GX_VTXFMT0, GX_VA_TEX0, GX_POS_XYZ, GX_RGBA6, 0);
        ext::GXSetVtxDesc(GX_VA_TEX0, GX_DIRECT);
        ext::GXSetTevOrder(GX_TEVSTAGE0, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR_NULL);
        ext::GXSetBlendMode(GX_BM_BLEND, GX_BL_SRCALPHA, GX_BL_INVSRCALPHA, GX_LO_CLEAR);
        ext::GXSetAlphaCompare(GX_GREATER, 0, GX_AOP_OR, GX_GREATER, GX_NEVER);
        ext::GXSetColorUpdate(1);
        ext::GXSetAlphaUpdate(0);
        if !gobj.is_null() {
            if let Some(cb) = (*text).render_callback {
                cb(gobj as *mut c_void);
            }
        }
        ext::GXSetTevColorIn(GX_TEVSTAGE0, GX_CC_ZERO, GX_CC_ZERO, GX_CC_ZERO, GX_CC_C0);
        ext::GXSetTevColorOp(GX_TEVSTAGE0, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, 0, GX_TEVPREV);
        ext::GXSetTevAlphaOp(GX_TEVSTAGE0, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, 0, GX_TEVPREV);
        // first quad is the background
        if (*text).bg_color.a != 0 {
            min_x = (*text).pos_x;
            min_y = (*text).pos_y;
            max_x = fmadds((*text).box_size_x, (*text).font_size.x, (*text).pos_x);
            max_y = fmadds((*text).box_size_y, (*text).font_size.y, (*text).pos_y);
            // Ported from m-ex "align background" (opt-in: MELEE_MEX=text_align_background).
            if ext::Mex_Enabled("text_align_background") && (*text).default_alignment != 0 {
                let width = (*text).box_size_x * (*text).font_size.x;
                let factor: f32 = if (*text).default_alignment == 1 { -0.5 } else { -1.0 };
                min_x = fmadds(width, factor, (*text).pos_x);
                max_x = min_x + width;
            }
            ext::GXSetTevAlphaIn(GX_TEVSTAGE0, GX_CA_ZERO, GX_CA_ZERO, GX_CA_ZERO, GX_CA_A0);
            ext::GXSetTevColor(GX_TEVREG0, (*text).bg_color);
            ext::GXBegin(GX_QUADS, GX_VTXFMT0, 4);
            {
                let depth = (*text).pos_z;
                let neg_min_y = -min_y;
                let neg_max_y = -max_y;
                ext::GXPosition3f32(min_x, neg_min_y, depth);
                ext::GXTexCoord2f32(0.0, 0.0);
                ext::GXPosition3f32(max_x, neg_min_y, depth);
                ext::GXTexCoord2f32(1.0, 0.0);
                ext::GXPosition3f32(max_x, neg_max_y, depth);
                ext::GXTexCoord2f32(1.0, 1.0);
                ext::GXPosition3f32(min_x, neg_max_y, depth);
                ext::GXTexCoord2f32(0.0, 1.0);
            }
            ext::GXEnd();
        }
        ext::GXSetTevAlphaIn(GX_TEVSTAGE0, GX_CA_ZERO, GX_CA_TEXA, GX_CA_A0, GX_CA_ZERO);
        if (*text).x4E != 0 {
            min_x = fmadds((*text).x14.z, (*text).font_size.x, (*text).pos_x);
            max_x = fmadds((*text).font_size.x, (*text).box_size_x - (*text).x14.w, (*text).pos_x);
            min_y = fmadds((*text).x14.x, (*text).font_size.y, (*text).pos_y);
            max_y = fmadds((*text).font_size.y, (*text).box_size_y - (*text).x14.y, (*text).pos_y);
        }
        if !(*text).alloc_data.is_null() {
            (*text).x78.x = (*text).x3C.x;
            (*text).x78.y = (*text).x3C.y;
            (*text).alignment = (*text).default_alignment;
            (*text).kerning = (*text).default_kerning;
            (*text).fitting = (*text).default_fitting;
        }
        let half_glyph: f64 = 16.0;

        'restart: loop {
            (*text).current_height = 0.0f32;
            let mut line_started: u32 = 0;
            let saved_font_size_x = (*text).x80.x;
            let saved_font_size_y = (*text).x80.y;
            let saved_color_r = (*text).active_color.r;
            let saved_color_g = (*text).active_color.g;
            let saved_color_b = (*text).active_color.b;
            let saved_scale_x = (*text).x78.x;
            let saved_scale_y = (*text).x78.y;
            let saved_alignment = (*text).alignment;
            let saved_fitting = (*text).fitting;
            let saved_kerning = (*text).kerning;
            let mut line_delay = (*text).x90;
            let mut char_delay = (*text).x92;
            let mut skip_count = (*text).x98;
            let saved_x6c = (*text).x6C;
            'scan: loop {
                if sis_cursor as *mut c_void == (*text).x60 && (*text).x4B != 0 {
                    break 'scan;
                }
                if sis_cursor as *mut c_void == (*text).x60 && (*text).x94 != 0 {
                    (*text).x94 -= 1;
                    break 'scan;
                }
                let opcode = *sis_cursor;
                match opcode {
                    0 => {
                        let pop_result = HSD_SisLib_803A7F0C(text, 5) as u32;
                        if pop_result != 0 {
                            sis_cursor = ext::gc_u32_to_ptr(pop_result.wrapping_add(4));
                        } else {
                            break 'scan;
                        }
                    }
                    1 | 2 => {
                        if opcode == 1 {
                            (*text).x60 = null_mut();
                            (*text).active_color = (*text).text_color;
                            (*text).x80.x = (*text).x34.x;
                            (*text).x80.y = (*text).x34.y;
                            (*text).x78.x = (*text).x3C.x;
                            (*text).x78.y = (*text).x3C.y;
                            (*text).x90 = (*text).x44;
                            (*text).x92 = (*text).x46;
                            (*text).alignment = (*text).default_alignment;
                            (*text).kerning = (*text).default_kerning;
                            (*text).fitting = (*text).default_fitting;
                            (*text).x94 = 0;
                            (*text).x4B = 0;
                            let mut clear_idx: i32 = 0;
                            while clear_idx < (*text).x6E as i32 {
                                *(*text).string_buffer.offset(clear_idx as isize) = 0;
                                clear_idx += 1;
                            }
                            (*text).x6C = 0;
                        }
                        (*text).x98 = 0;
                        sis_cursor = sis_cursor.add(1);
                        (*text).sis_buffer = sis_cursor as *mut SIS;
                        continue 'restart;
                    }
                    3 => {
                        if line_started == 0 {
                            line_height_out = 32.0f32 * (*text).x80.y;
                        }
                        line_started = 0;
                        let inner = fmadds((*text).x80.y, (*text).x78.y, line_height_out);
                        (*text).current_height = fmadds((*text).font_size.y, inner, (*text).current_height);
                        if skip_count != 0 {
                            skip_count -= 1;
                        } else {
                            (*text).x98 = (*text).x98.wrapping_add(1);
                            (*text).x94 = char_delay as u32;
                            (*text).x60 = sis_cursor.add(1) as *mut c_void;
                        }
                    }
                    4 => {
                        if skip_count != 0 {
                            skip_count -= 1;
                        } else {
                            (*text).x98 = (*text).x98.wrapping_add(1);
                            (*text).x4B = 1;
                            (*text).x60 = sis_cursor.add(1) as *mut c_void;
                        }
                    }
                    5 => {
                        if skip_count != 0 {
                            skip_count -= 1;
                        } else {
                            (*text).x98 = (*text).x98.wrapping_add(1);
                            (*text).x94 = be16(sis_cursor.add(1)) as u32;
                            (*text).x60 = sis_cursor.add(3) as *mut c_void;
                        }
                        sis_cursor = sis_cursor.add(2);
                    }
                    6 => {
                        line_delay = be16(sis_cursor.add(1));
                        char_delay = be16(sis_cursor.add(3));
                        sis_cursor = sis_cursor.add(4);
                    }
                    7 => {
                        line_started = 1;
                        HSD_SisLib_803A8134(sis_cursor.add(5), text, &mut line_width_out, &mut line_height_out);
                        let x_origin = (be16(sis_cursor.add(1)) as i16) as f32;
                        if (*text).fitting == 1 && (*text).box_size_x < line_width_out {
                            (*text).x88 = (*text).box_size_x / line_width_out;
                        } else {
                            (*text).x88 = 1.0f32;
                        }
                        match (*text).alignment as i32 {
                            1 => {
                                (*text).current_width =
                                    -fmsubs(0.5f32, line_width_out * (*text).x88, x_origin);
                            }
                            2 => {
                                (*text).current_width = -fmsubs(line_width_out, (*text).x88, x_origin);
                            }
                            _ => (*text).current_width = x_origin,
                        }
                        let y_offset = be16(sis_cursor.add(3)) as i16;
                        sis_cursor = sis_cursor.add(4);
                        (*text).current_height = (y_offset as f32) * (*text).font_size.y;
                    }
                    8 | 9 => {
                        if opcode == 9 {
                            HSD_SisLib_803A7684(text, sis_cursor, 5);
                        }
                        sis_cursor = ext::gc_u32_to_ptr(be32(sis_cursor.add(1))).offset(-1);
                    }
                    10 => {
                        if (*text).alloc_data.is_null() || saved_kerning == 0 {
                            HSD_SisLib_803A7684(text, sis_cursor, 1);
                            (*text).x78.x = (be16(sis_cursor.add(1)) as i16) as f32 / 256.0f32;
                            (*text).x78.y = (be16(sis_cursor.add(3)) as i16) as f32 / 256.0f32;
                        }
                        sis_cursor = sis_cursor.add(4);
                    }
                    11 => {
                        if (*text).alloc_data.is_null() || saved_kerning == 0 {
                            HSD_SisLib_803A7F0C(text, 1);
                        }
                    }
                    12 => {
                        HSD_SisLib_803A7684(text, sis_cursor, 2);
                        (*text).active_color.r = *sis_cursor.add(1);
                        (*text).active_color.g = *sis_cursor.add(2);
                        (*text).active_color.b = *sis_cursor.add(3);
                        // Ported from m-ex "copy alpha to 0x8c" (opt-in: MELEE_MEX=text_copy_alpha).
                        if ext::Mex_Enabled("text_copy_alpha") {
                            (*text).active_color.a = (*text).text_color.a;
                        }
                        sis_cursor = sis_cursor.add(3);
                    }
                    13 => {
                        HSD_SisLib_803A7F0C(text, 2);
                    }
                    14 => {
                        HSD_SisLib_803A7684(text, sis_cursor, 3);
                        (*text).x80.x = be16(sis_cursor.add(1)) as f32 / 256.0f32;
                        (*text).x80.y = be16(sis_cursor.add(3)) as f32 / 256.0f32;
                        sis_cursor = sis_cursor.add(4);
                    }
                    15 => {
                        HSD_SisLib_803A7F0C(text, 3);
                    }
                    16 => {
                        HSD_SisLib_803A7684(text, sis_cursor, 4);
                        (*text).alignment = 1;
                    }
                    18 => {
                        HSD_SisLib_803A7684(text, sis_cursor, 4);
                        (*text).alignment = 0;
                    }
                    20 => {
                        HSD_SisLib_803A7684(text, sis_cursor, 4);
                        (*text).alignment = 2;
                    }
                    17 | 19 | 21 => {
                        HSD_SisLib_803A7F0C(text, 4);
                    }
                    22 => (*text).kerning = 1,
                    23 => (*text).kerning = 0,
                    24 => (*text).fitting = 1,
                    25 => (*text).fitting = 0,
                    26 => {
                        if line_started == 0 {
                            line_started += 1;
                            HSD_SisLib_803A8134(sis_cursor, text, &mut line_width_out, &mut line_height_out);
                            let measured_width = line_width_out;
                            sisFitLineToBox(text, measured_width);
                        }
                        // f64 fma: (f64)x88 * ((f64)x80.x * (16.0 + (f64)x78.x)) + (f64)current_width
                        let inner = (*text).x80.x as f64 * (half_glyph + (*text).x78.x as f64);
                        (*text).current_width =
                            ((*text).x88 as f64).mul_add(inner, (*text).current_width as f64) as f32;
                        if skip_count != 0 {
                            skip_count -= 1;
                        } else {
                            (*text).x98 = (*text).x98.wrapping_add(1);
                            (*text).x94 = line_delay as u32;
                            (*text).x60 = sis_cursor.add(1) as *mut c_void;
                        }
                    }
                    _ => {
                        if opcode >= 0x20 {
                            if line_started == 0 {
                                line_started += 1;
                                HSD_SisLib_803A8134(sis_cursor, text, &mut line_width_out, &mut line_height_out);
                                let measured_width = line_width_out;
                                sisFitLineToBox(text, measured_width);
                            }
                            let glyph_idx = be16(sis_cursor);
                            let mut tex_offset: i32 = if glyph_idx < 0x4000 {
                                glyph_idx as i32 - 0x2000
                            } else {
                                glyph_idx as i32 - 0x4000
                            };
                            let mut glyph_x =
                                fmadds((*text).current_width, (*text).font_size.x, (*text).pos_x);
                            let scale_x = (*text).font_size.x;
                            if (*text).kerning != 0 {
                                let idx = ((tex_offset * 2) & 0x1FFFE) as usize;
                                let k = if glyph_idx < 0x4000 {
                                    default_kerning.add(idx).read()
                                } else {
                                    (textures as *const u8).add(idx).read()
                                };
                                glyph_x = -fmsubs(scale_x, (*text).x80.x * ((k as i32 - 1) as f32), glyph_x);
                            }
                            {
                                let scale_y = (*text).font_size.y;
                                let glyph_size = 32.0f32 * (*text).x80.y;
                                let mut uv_top: f32 = 0.0;
                                let glyph_w = HSD_SisLib_GlyphWidth(&*text, scale_x);
                                let mut quad_right = fmadds((*text).x88, glyph_w, glyph_x);
                                let glyph_y = fmadds(
                                    scale_y,
                                    line_height_out - glyph_size,
                                    (*text).pos_y + (*text).current_height,
                                );
                                let glyph_h = glyph_size * scale_y;
                                let mut uv_bottom: f32 = 1.0;
                                let mut uv_left: f32 = 0.0;
                                let mut quad_top = glyph_y;
                                let mut uv_right: f32 = 1.0;
                                let mut quad_bottom = glyph_y + glyph_h;
                                'draw: {
                                    if (*text).x4E != 0 {
                                        if min_x > quad_right || max_x < glyph_x || min_y > quad_bottom || max_y < quad_top {
                                            break 'draw;
                                        }
                                        if min_x > glyph_x {
                                            let clip_left = min_x - glyph_x;
                                            uv_left = clip_left / glyph_w;
                                            glyph_x += clip_left;
                                        }
                                        if max_x < quad_right {
                                            let clip_right = quad_right - max_x;
                                            uv_right = 1.0f32 - clip_right / glyph_w;
                                            quad_right -= clip_right;
                                        }
                                        if min_y > quad_top {
                                            let clip_top = min_y - quad_top;
                                            uv_top = clip_top / glyph_h;
                                            quad_top += clip_top;
                                        }
                                        if max_y < quad_bottom {
                                            let clip_bottom = quad_bottom - max_y;
                                            uv_bottom = 1.0f32 - clip_bottom / glyph_h;
                                            quad_bottom -= clip_bottom;
                                        }
                                    }
                                    let tex_ofs = ((tex_offset << 9) & 0x01FFFE00) as usize;
                                    if glyph_idx < 0x4000 {
                                        ext::GXInitTexObj(&mut tex_obj, data.add(tex_ofs), 0x20, 0x20, GX_TF_I4, GX_CLAMP, GX_CLAMP, 0);
                                    } else {
                                        ext::GXInitTexObj(&mut tex_obj, (kerning as *mut u8).add(tex_ofs), 0x20, 0x20, GX_TF_I4, GX_CLAMP, GX_CLAMP, 0);
                                    }
                                    ext::GXLoadTexObj(&mut tex_obj, GX_TEXMAP0);
                                    ext::GXSetTevColor(GX_TEVREG0, (*text).active_color);
                                    // Ported from m-ex "add alpha tev" (opt-in: MELEE_MEX=text_alpha_tev).
                                    if ext::Mex_Enabled("text_alpha_tev") {
                                        ext::GXSetTevKAlphaSel(GX_TEVSTAGE0, GX_TEV_KASEL_K0_A);
                                        ext::GXSetTevKColor(GX_KCOLOR0, (*text).active_color);
                                    }
                                    ext::GXBegin(GX_QUADS, GX_VTXFMT0, 4);
                                    {
                                        let glyph_depth = (*text).pos_z;
                                        let neg_quad_top = -quad_top;
                                        let neg_quad_bottom = -quad_bottom;
                                        ext::GXPosition3f32(glyph_x, neg_quad_top, glyph_depth);
                                        ext::GXTexCoord2f32(uv_left, uv_top);
                                        ext::GXPosition3f32(quad_right, neg_quad_top, glyph_depth);
                                        ext::GXTexCoord2f32(uv_right, uv_top);
                                        ext::GXPosition3f32(quad_right, neg_quad_bottom, glyph_depth);
                                        ext::GXTexCoord2f32(uv_right, uv_bottom);
                                        ext::GXPosition3f32(glyph_x, neg_quad_bottom, glyph_depth);
                                        ext::GXTexCoord2f32(uv_left, uv_bottom);
                                    }
                                    ext::GXEnd();
                                }
                                // glyph_draw_done:
                                (*text).current_width = fmadds(
                                    (*text).x88,
                                    (*text).x80.x * (32.0f32 + (*text).x78.x),
                                    (*text).current_width,
                                );
                                if (*text).kerning != 0 {
                                    let idx = ((tex_offset * 2) & 0x1FFFE) as usize;
                                    let (k0, k1) = if glyph_idx < 0x4000 {
                                        (default_kerning.add(idx).read(), default_kerning.add(idx + 1).read())
                                    } else {
                                        let base = (textures as *const u8).add(idx);
                                        (base.read(), base.add(1).read())
                                    };
                                    tex_offset = k1 as i32 - 2;
                                    (*text).current_width = -fmsubs(
                                        (*text).x88,
                                        (*text).x80.x * ((k0 as i32 + tex_offset) as f32),
                                        (*text).current_width,
                                    );
                                }
                                if skip_count != 0 {
                                    skip_count -= 1;
                                } else {
                                    (*text).x98 = (*text).x98.wrapping_add(1);
                                    (*text).x94 = line_delay as u32;
                                    (*text).x60 = sis_cursor.add(2) as *mut c_void;
                                }
                                sis_cursor = sis_cursor.add(1);
                            }
                        }
                    }
                }
                sis_cursor = sis_cursor.add(1);
            }
            // render_done:
            ext::HSD_StateInvalidate(-1);
            (*text).active_color.r = saved_color_r;
            (*text).active_color.g = saved_color_g;
            (*text).active_color.b = saved_color_b;
            (*text).x80.x = saved_font_size_x;
            (*text).x80.y = saved_font_size_y;
            (*text).x78.x = saved_scale_x;
            (*text).x78.y = saved_scale_y;
            (*text).alignment = saved_alignment;
            (*text).kerning = saved_kerning;
            (*text).fitting = saved_fitting;
            (*text).x6C = saved_x6c;
            let mut clear_i = (*text).x6C as i32;
            while clear_i < (*text).x6E as i32 {
                *(*text).string_buffer.offset(clear_i as isize) = 0;
                clear_i += 1;
            }
            break 'restart;
        }
    }
}

/// `HSD_SisLib_803A945C` (0x803A945C)
pub unsafe fn HSD_SisLib_803A945C(path: *mut i8) -> *mut HSD_Archive {
    ext::lbArchive_LoadArchive(path)
}

/// `HSD_SisLib_803A947C` (0x803A947C)
pub unsafe fn HSD_SisLib_803A947C(archive: *mut HSD_Archive) {
    ext::lbArchive_80016EFC(archive);
}

// ---- data tables of the TU (from the C source) ----

/// `HSD_SisLib_8040C680` (glyph codes by table index).
pub static HSD_SisLib_8040C680: [u8; 0x240] = [
    0x20, 0xE3, 0x20, 0xEC, 0x20, 0xF4, 0x21, 0x04, 0x21, 0x02, 0x21, 0x03,
    0x21, 0x05, 0x20, 0xF3, 0x20, 0xF5, 0x20, 0xF6, 0x21, 0x06, 0x20, 0xFB,
    0x20, 0xE6, 0x20, 0xFC, 0x20, 0xE7, 0x20, 0xF0, 0x20, 0x00, 0x20, 0x01,
    0x20, 0x02, 0x20, 0x03, 0x20, 0x04, 0x20, 0x05, 0x20, 0x06, 0x20, 0x07,
    0x20, 0x08, 0x20, 0x09, 0x20, 0xE9, 0x20, 0xEA, 0x20, 0xFF, 0x20, 0xFE,
    0x21, 0x00, 0x20, 0xEB, 0x21, 0x07, 0x20, 0x0A, 0x20, 0x0B, 0x20, 0x0C,
    0x20, 0x0D, 0x20, 0x0E, 0x20, 0x0F, 0x20, 0x10, 0x20, 0x11, 0x20, 0x12,
    0x20, 0x13, 0x20, 0x14, 0x20, 0x15, 0x20, 0x16, 0x20, 0x17, 0x20, 0x18,
    0x20, 0x19, 0x20, 0x1A, 0x20, 0x1B, 0x20, 0x1C, 0x20, 0x1D, 0x20, 0x1E,
    0x20, 0x1F, 0x20, 0x20, 0x20, 0x21, 0x20, 0x22, 0x20, 0x23, 0x20, 0xF7,
    0x21, 0x01, 0x20, 0xF8, 0x20, 0xED, 0x20, 0xEE, 0x20, 0x24, 0x20, 0x25,
    0x20, 0x26, 0x20, 0x27, 0x20, 0x28, 0x20, 0x29, 0x20, 0x2A, 0x20, 0x2B,
    0x20, 0x2C, 0x20, 0x2D, 0x20, 0x2E, 0x20, 0x2F, 0x20, 0x30, 0x20, 0x31,
    0x20, 0x32, 0x20, 0x33, 0x20, 0x34, 0x20, 0x35, 0x20, 0x36, 0x20, 0x37,
    0x20, 0x38, 0x20, 0x39, 0x20, 0x3A, 0x20, 0x3B, 0x20, 0x3C, 0x20, 0x3D,
    0x20, 0xF9, 0x20, 0xF2, 0x20, 0xFA, 0x20, 0xF1, 0x20, 0x3E, 0x20, 0x3F,
    0x20, 0x40, 0x20, 0x41, 0x20, 0x42, 0x20, 0x43, 0x20, 0x44, 0x20, 0x45,
    0x20, 0x46, 0x20, 0x47, 0x20, 0x48, 0x20, 0x49, 0x20, 0x4A, 0x20, 0x4B,
    0x20, 0x4C, 0x20, 0x4D, 0x20, 0x4E, 0x20, 0x4F, 0x20, 0x50, 0x20, 0x51,
    0x20, 0x52, 0x20, 0x53, 0x20, 0x54, 0x20, 0x55, 0x20, 0x56, 0x20, 0x57,
    0x20, 0x58, 0x20, 0x59, 0x20, 0x5A, 0x20, 0x5B, 0x20, 0x5C, 0x20, 0x5D,
    0x20, 0x5E, 0x20, 0x5F, 0x20, 0x60, 0x20, 0x61, 0x20, 0x62, 0x20, 0x63,
    0x20, 0x64, 0x20, 0x65, 0x20, 0x66, 0x20, 0x67, 0x20, 0x68, 0x20, 0x69,
    0x20, 0x6A, 0x20, 0x6B, 0x20, 0x6C, 0x20, 0x6D, 0x20, 0x6E, 0x20, 0x6F,
    0x20, 0x70, 0x20, 0x71, 0x20, 0x72, 0x20, 0x73, 0x20, 0x74, 0x20, 0x75,
    0x20, 0x76, 0x20, 0x77, 0x20, 0x78, 0x20, 0x79, 0x20, 0x7A, 0x20, 0x7B,
    0x20, 0x7C, 0x20, 0x7D, 0x20, 0x7E, 0x20, 0x7F, 0x20, 0x80, 0x20, 0x81,
    0x20, 0x82, 0x20, 0x83, 0x20, 0x84, 0x20, 0x85, 0x20, 0x86, 0x20, 0x87,
    0x20, 0x88, 0x20, 0x89, 0x20, 0x8A, 0x20, 0x8B, 0x20, 0x8C, 0x20, 0x8D,
    0x20, 0x8E, 0x20, 0x8F, 0x20, 0x90, 0x20, 0x91, 0x20, 0x92, 0x20, 0x93,
    0x20, 0x94, 0x20, 0x95, 0x20, 0x96, 0x20, 0x97, 0x20, 0x98, 0x20, 0x99,
    0x20, 0x9A, 0x20, 0x9B, 0x20, 0x9C, 0x20, 0x9D, 0x20, 0x9E, 0x20, 0x9F,
    0x20, 0xA0, 0x20, 0xA1, 0x20, 0xA2, 0x20, 0xA3, 0x20, 0xA4, 0x20, 0xA5,
    0x20, 0xA6, 0x20, 0xA7, 0x20, 0xA8, 0x20, 0xA9, 0x20, 0xAA, 0x20, 0xAB,
    0x20, 0xAC, 0x20, 0xAD, 0x20, 0xAE, 0x20, 0xAF, 0x20, 0xB0, 0x20, 0xB1,
    0x20, 0xB2, 0x20, 0xB3, 0x20, 0xB4, 0x20, 0xB5, 0x20, 0xB6, 0x20, 0xB7,
    0x20, 0xB8, 0x20, 0xB9, 0x20, 0xBA, 0x20, 0xBB, 0x20, 0xBC, 0x20, 0xBD,
    0x20, 0xBE, 0x20, 0xBF, 0x20, 0xC0, 0x20, 0xC1, 0x20, 0xC2, 0x20, 0xC3,
    0x20, 0xC4, 0x20, 0xC5, 0x20, 0xC6, 0x20, 0xC7, 0x20, 0xC8, 0x20, 0xC9,
    0x20, 0xCA, 0x20, 0xCB, 0x20, 0xCC, 0x20, 0xCD, 0x20, 0xCE, 0x20, 0xCF,
    0x20, 0xD0, 0x20, 0xD1, 0x20, 0xD2, 0x20, 0xD3, 0x20, 0xD4, 0x20, 0xD5,
    0x20, 0xD6, 0x20, 0xD7, 0x20, 0xD8, 0x20, 0xD9, 0x20, 0xDA, 0x20, 0xDB,
    0x20, 0xDC, 0x20, 0xDD, 0x20, 0xDE, 0x20, 0xDF, 0x20, 0xE0, 0x20, 0xE1,
    0x20, 0xE2, 0x20, 0xE4, 0x20, 0xE5, 0x20, 0xEF, 0x20, 0xE8, 0x20, 0xFD,
    0x21, 0x0A, 0x21, 0x17, 0x21, 0x13, 0x21, 0x1B, 0x21, 0x0C, 0x21, 0x1C,
    0x21, 0x19, 0x21, 0x1D, 0x21, 0x16, 0x21, 0x10, 0x21, 0x08, 0x21, 0x15,
    0x21, 0x1E, 0x21, 0x11, 0x21, 0x12, 0x21, 0x09, 0x21, 0x18, 0x21, 0x0B,
    0x21, 0x0E, 0x21, 0x0F, 0x21, 0x1A, 0x21, 0x14, 0x21, 0x0D, 0x00, 0x00,
];

/// `lbl_8040C8C0` (Shift-JIS code per glyph).
pub static lbl_8040C8C0: [u8; 0x240] = [
    0x81, 0x40, 0x81, 0x49, 0x81, 0x68, 0x81, 0x94, 0x81, 0x90, 0x81, 0x93,
    0x81, 0x95, 0x81, 0x66, 0x81, 0x69, 0x81, 0x6A, 0x81, 0x96, 0x81, 0x7B,
    0x81, 0x43, 0x81, 0x7C, 0x81, 0x44, 0x81, 0x5E, 0x82, 0x4F, 0x82, 0x50,
    0x82, 0x51, 0x82, 0x52, 0x82, 0x53, 0x82, 0x54, 0x82, 0x55, 0x82, 0x56,
    0x82, 0x57, 0x82, 0x58, 0x81, 0x46, 0x81, 0x47, 0x81, 0x83, 0x81, 0x81,
    0x81, 0x84, 0x81, 0x48, 0x81, 0x97, 0x82, 0x60, 0x82, 0x61, 0x82, 0x62,
    0x82, 0x63, 0x82, 0x64, 0x82, 0x65, 0x82, 0x66, 0x82, 0x67, 0x82, 0x68,
    0x82, 0x69, 0x82, 0x6A, 0x82, 0x6B, 0x82, 0x6C, 0x82, 0x6D, 0x82, 0x6E,
    0x82, 0x6F, 0x82, 0x70, 0x82, 0x71, 0x82, 0x72, 0x82, 0x73, 0x82, 0x74,
    0x82, 0x75, 0x82, 0x76, 0x82, 0x77, 0x82, 0x78, 0x82, 0x79, 0x81, 0x6D,
    0x81, 0x8F, 0x81, 0x6E, 0x81, 0x4F, 0x81, 0x51, 0x82, 0x81, 0x82, 0x82,
    0x82, 0x83, 0x82, 0x84, 0x82, 0x85, 0x82, 0x86, 0x82, 0x87, 0x82, 0x88,
    0x82, 0x89, 0x82, 0x8A, 0x82, 0x8B, 0x82, 0x8C, 0x82, 0x8D, 0x82, 0x8E,
    0x82, 0x8F, 0x82, 0x90, 0x82, 0x91, 0x82, 0x92, 0x82, 0x93, 0x82, 0x94,
    0x82, 0x95, 0x82, 0x96, 0x82, 0x97, 0x82, 0x98, 0x82, 0x99, 0x82, 0x9A,
    0x81, 0x6F, 0x81, 0x62, 0x81, 0x70, 0x81, 0x60, 0x82, 0x9F, 0x82, 0xA0,
    0x82, 0xA1, 0x82, 0xA2, 0x82, 0xA3, 0x82, 0xA4, 0x82, 0xA5, 0x82, 0xA6,
    0x82, 0xA7, 0x82, 0xA8, 0x82, 0xA9, 0x82, 0xAA, 0x82, 0xAB, 0x82, 0xAC,
    0x82, 0xAD, 0x82, 0xAE, 0x82, 0xAF, 0x82, 0xB0, 0x82, 0xB1, 0x82, 0xB2,
    0x82, 0xB3, 0x82, 0xB4, 0x82, 0xB5, 0x82, 0xB6, 0x82, 0xB7, 0x82, 0xB8,
    0x82, 0xB9, 0x82, 0xBA, 0x82, 0xBB, 0x82, 0xBC, 0x82, 0xBD, 0x82, 0xBE,
    0x82, 0xBF, 0x82, 0xC0, 0x82, 0xC1, 0x82, 0xC2, 0x82, 0xC3, 0x82, 0xC4,
    0x82, 0xC5, 0x82, 0xC6, 0x82, 0xC7, 0x82, 0xC8, 0x82, 0xC9, 0x82, 0xCA,
    0x82, 0xCB, 0x82, 0xCC, 0x82, 0xCD, 0x82, 0xCE, 0x82, 0xCF, 0x82, 0xD0,
    0x82, 0xD1, 0x82, 0xD2, 0x82, 0xD3, 0x82, 0xD4, 0x82, 0xD5, 0x82, 0xD6,
    0x82, 0xD7, 0x82, 0xD8, 0x82, 0xD9, 0x82, 0xDA, 0x82, 0xDB, 0x82, 0xDC,
    0x82, 0xDD, 0x82, 0xDE, 0x82, 0xDF, 0x82, 0xE0, 0x82, 0xE1, 0x82, 0xE2,
    0x82, 0xE3, 0x82, 0xE4, 0x82, 0xE5, 0x82, 0xE6, 0x82, 0xE7, 0x82, 0xE8,
    0x82, 0xE9, 0x82, 0xEA, 0x82, 0xEB, 0x82, 0xEC, 0x82, 0xED, 0x82, 0xF0,
    0x82, 0xF1, 0x83, 0x40, 0x83, 0x41, 0x83, 0x42, 0x83, 0x43, 0x83, 0x44,
    0x83, 0x45, 0x83, 0x46, 0x83, 0x47, 0x83, 0x48, 0x83, 0x49, 0x83, 0x4A,
    0x83, 0x4B, 0x83, 0x4C, 0x83, 0x4D, 0x83, 0x4E, 0x83, 0x4F, 0x83, 0x50,
    0x83, 0x51, 0x83, 0x52, 0x83, 0x53, 0x83, 0x54, 0x83, 0x55, 0x83, 0x56,
    0x83, 0x57, 0x83, 0x58, 0x83, 0x59, 0x83, 0x5A, 0x83, 0x5B, 0x83, 0x5C,
    0x83, 0x5D, 0x83, 0x5E, 0x83, 0x5F, 0x83, 0x60, 0x83, 0x61, 0x83, 0x62,
    0x83, 0x63, 0x83, 0x64, 0x83, 0x65, 0x83, 0x66, 0x83, 0x67, 0x83, 0x68,
    0x83, 0x69, 0x83, 0x6A, 0x83, 0x6B, 0x83, 0x6C, 0x83, 0x6D, 0x83, 0x6E,
    0x83, 0x6F, 0x83, 0x70, 0x83, 0x71, 0x83, 0x72, 0x83, 0x73, 0x83, 0x74,
    0x83, 0x75, 0x83, 0x76, 0x83, 0x77, 0x83, 0x78, 0x83, 0x79, 0x83, 0x7A,
    0x83, 0x7B, 0x83, 0x7C, 0x83, 0x7D, 0x83, 0x7E, 0x83, 0x80, 0x83, 0x81,
    0x83, 0x82, 0x83, 0x83, 0x83, 0x84, 0x83, 0x85, 0x83, 0x86, 0x83, 0x87,
    0x83, 0x88, 0x83, 0x89, 0x83, 0x8A, 0x83, 0x8B, 0x83, 0x8C, 0x83, 0x8D,
    0x83, 0x8E, 0x83, 0x8F, 0x83, 0x92, 0x83, 0x93, 0x83, 0x94, 0x83, 0x95,
    0x83, 0x96, 0x81, 0x41, 0x81, 0x42, 0x81, 0x5B, 0x81, 0x45, 0x81, 0x7E,
    0x8C, 0x52, 0x92, 0x63, 0x90, 0x6C, 0x94, 0xB2, 0x8C, 0xC2, 0x95, 0xC2,
    0x93, 0xC7, 0x96, 0x7B, 0x91, 0xCC, 0x8E, 0xE6, 0x88, 0xB5, 0x90, 0xE0,
    0x96, 0xBE, 0x8F, 0x91, 0x8F, 0xDA, 0x89, 0x9F, 0x93, 0x64, 0x8C, 0xB9,
    0x8E, 0x77, 0x8E, 0xA6, 0x94, 0xAD, 0x90, 0xB6, 0x8D, 0x9E, 0x00, 0x00,
];

/// `HSD_SisLib_8040CB00` (default kerning pairs).
pub static HSD_SisLib_8040CB00: [u8; 0x240] = [
    0x09, 0x08, 0x09, 0x0C, 0x09, 0x08, 0x08, 0x08, 0x09, 0x08, 0x09, 0x08,
    0x09, 0x08, 0x09, 0x08, 0x09, 0x08, 0x09, 0x08, 0x04, 0x03, 0x06, 0x05,
    0x04, 0x04, 0x05, 0x03, 0x08, 0x06, 0x08, 0x06, 0x04, 0x03, 0x05, 0x03,
    0x0D, 0x0B, 0x07, 0x06, 0x06, 0x04, 0x08, 0x06, 0x03, 0x01, 0x04, 0x03,
    0x04, 0x03, 0x06, 0x04, 0x04, 0x02, 0x06, 0x04, 0x05, 0x04, 0x06, 0x05,
    0x05, 0x03, 0x04, 0x03, 0x01, 0x00, 0x05, 0x04, 0x04, 0x03, 0x05, 0x04,
    0x07, 0x06, 0x07, 0x06, 0x07, 0x06, 0x07, 0x06, 0x07, 0x06, 0x08, 0x09,
    0x07, 0x06, 0x08, 0x07, 0x0A, 0x0A, 0x09, 0x0C, 0x08, 0x06, 0x0C, 0x0C,
    0x01, 0x00, 0x07, 0x06, 0x07, 0x06, 0x07, 0x06, 0x07, 0x06, 0x0A, 0x09,
    0x08, 0x07, 0x08, 0x09, 0x07, 0x05, 0x07, 0x06, 0x02, 0x01, 0x07, 0x06,
    0x06, 0x06, 0x07, 0x06, 0x05, 0x05, 0x03, 0x02, 0x05, 0x05, 0x03, 0x02,
    0x06, 0x06, 0x04, 0x04, 0x06, 0x05, 0x03, 0x02, 0x05, 0x03, 0x02, 0x00,
    0x01, 0x00, 0x00, 0x00, 0x04, 0x04, 0x04, 0x00, 0x04, 0x05, 0x04, 0x03,
    0x02, 0x02, 0x02, 0x00, 0x04, 0x04, 0x03, 0x00, 0x04, 0x03, 0x03, 0x00,
    0x05, 0x03, 0x05, 0x03, 0x02, 0x02, 0x02, 0x00, 0x02, 0x02, 0x01, 0x00,
    0x02, 0x02, 0x01, 0x00, 0x02, 0x02, 0x02, 0x00, 0x03, 0x03, 0x03, 0x00,
    0x04, 0x05, 0x01, 0x02, 0x01, 0x00, 0x02, 0x02, 0x02, 0x00, 0x04, 0x04,
    0x03, 0x00, 0x01, 0x01, 0x03, 0x02, 0x01, 0x00, 0x00, 0x00, 0x01, 0x01,
    0x02, 0x01, 0x02, 0x00, 0x02, 0x00, 0x02, 0x01, 0x02, 0x00, 0x02, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x01,
    0x02, 0x01, 0x01, 0x00, 0x01, 0x00, 0x03, 0x03, 0x01, 0x01, 0x02, 0x01,
    0x02, 0x02, 0x03, 0x02, 0x04, 0x04, 0x01, 0x01, 0x05, 0x05, 0x02, 0x02,
    0x05, 0x05, 0x02, 0x02, 0x03, 0x03, 0x05, 0x05, 0x02, 0x02, 0x01, 0x00,
    0x02, 0x03, 0x04, 0x04, 0x01, 0x01, 0x02, 0x03, 0x01, 0x01, 0x05, 0x04,
    0x03, 0x01, 0x04, 0x06, 0x01, 0x03, 0x06, 0x05, 0x03, 0x02, 0x05, 0x04,
    0x02, 0x02, 0x04, 0x05, 0x02, 0x01, 0x02, 0x02, 0x02, 0x00, 0x02, 0x02,
    0x01, 0x00, 0x03, 0x02, 0x02, 0x00, 0x01, 0x01, 0x01, 0x00, 0x03, 0x04,
    0x02, 0x00, 0x02, 0x02, 0x01, 0x00, 0x02, 0x01, 0x02, 0x01, 0x02, 0x01,
    0x02, 0x00, 0x02, 0x02, 0x02, 0x00, 0x02, 0x03, 0x01, 0x00, 0x03, 0x02,
    0x02, 0x00, 0x02, 0x02, 0x02, 0x00, 0x05, 0x05, 0x02, 0x02, 0x01, 0x00,
    0x02, 0x02, 0x01, 0x00, 0x09, 0x03, 0x09, 0x02, 0x02, 0x02, 0x01, 0x01,
    0x02, 0x02, 0x03, 0x02, 0x03, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x04, 0x04, 0x00, 0x04, 0x02, 0x04, 0x03, 0x03, 0x00, 0x03, 0x00,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
    0x02, 0x02, 0x04, 0x04, 0x01, 0x01, 0x02, 0x03, 0x02, 0x02, 0x04, 0x04,
    0x02, 0x01, 0x05, 0x05, 0x02, 0x02, 0x06, 0x06, 0x03, 0x04, 0x03, 0x03,
    0x05, 0x05, 0x01, 0x01, 0x05, 0x03, 0x03, 0x03, 0x06, 0x06, 0x03, 0x03,
    0x04, 0x03, 0x02, 0x02, 0x03, 0x00, 0x04, 0x05, 0x04, 0x00, 0x08, 0x08,
    0x02, 0x12, 0x02, 0x13, 0x0C, 0x0D, 0x0C, 0x0D, 0x0B, 0x0A, 0x0D, 0x0C,
    0x0D, 0x0C, 0x07, 0x06, 0x0D, 0x0C, 0x0A, 0x08, 0x00, 0x00, 0x02, 0x01,
    0x01, 0x00, 0x01, 0x01, 0x0F, 0x0F, 0x02, 0x17, 0x01, 0x10, 0x15, 0x00,
    0x01, 0x14, 0x15, 0x00, 0x01, 0x13, 0x13, 0x00, 0x01, 0x12, 0x04, 0x03,
    0x04, 0x03, 0x05, 0x04, 0x04, 0x03, 0x01, 0x01, 0x01, 0x00, 0x04, 0x02,
    0x05, 0x05, 0x01, 0x00, 0x04, 0x03, 0x03, 0x02, 0x08, 0x06, 0x03, 0x02,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x01, 0x01, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00,
];
