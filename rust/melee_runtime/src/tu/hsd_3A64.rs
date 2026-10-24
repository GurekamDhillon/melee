//! Port of src/sysdolphin/baselib/hsd_3A64.c: SIS text-stream editing. A SIS stream is a byte
//! string of 2-byte glyph codes (>= 0x20) and control opcodes (< 0x20) with fixed operand sizes;
//! this TU copies/appends streams, converts ASCII/Shift-JIS to glyph codes (`HSD_SisLib_803A67EC`),
//! and edits the position / colour / scale operands of numbered entries in place.
//!
//! Stream operands are big-endian bytes written one at a time, so this port keeps the C's byte
//! stores. `printf`-style entry points take the already-formatted string (`vsnprintf` into the
//! C's 128-byte stack buffer is reproduced by truncating to 127 bytes). Callees in unported TUs
//! (sislib.c allocation and text creation) are TODO stubs in [`ext`], named like the C.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use super::hsd_3A76::{lbl_8040C8C0, HSD_SisLib_8040C680, HSD_SisLib_FontAtlas_COUNT};
use core::ffi::c_void;
use melee_types::*;

pub mod ext {
    #![allow(non_snake_case, unused_variables)]
    use super::*;
    /// `HSD_SisLib_804D1124[5]` (sislib.c): the loaded font SIS tables.
    pub static mut HSD_SisLib_804D1124: [*mut SIS; 5] = [core::ptr::null_mut(); 5];
    /// `HSD_SisLib_Alloc(size)` (sislib.c).
    pub fn HSD_SisLib_Alloc(size: i32) -> *mut c_void { todo!("HSD_SisLib_Alloc (baselib/sislib.c)") }
    /// `HSD_SisLib_Free(ptr)` (sislib.c).
    pub fn HSD_SisLib_Free(ptr: *mut c_void) { unsafe { crate::tu::sislib::HSD_SisLib_Free(ptr as _) } }
    pub fn HSD_SisLib_803A5ACC(font_idx: i32, context_id: i32, x: f32, y: f32, z: f32, w: f32, h: f32) -> *mut HSD_Text {
        todo!("HSD_SisLib_803A5ACC (baselib/sislib.c)")
    }
    pub fn HSD_SisLib_803A6368(text: *mut HSD_Text, a: i32) { unsafe { crate::tu::sislib::HSD_SisLib_803A6368(text, a as _) } }
}

/// Reads the NUL-terminated SIS byte at `p`.
#[inline]
unsafe fn rd(p: *const u8) -> u8 {
    *p
}

/// `HSD_SisLib_803A6478` (0x803A6478): copy one SIS stream (up to, not including, the NUL).
/// Returns the pointer to the terminating NUL written to `dst`.
pub unsafe fn HSD_SisLib_803A6478(mut dst: *mut u8, mut src: *const u8) -> *mut u8 {
    while rd(src) != 0 {
        if rd(src) >= 0x20 {
            *dst = rd(src);
            dst = dst.add(1);
            src = src.add(1);
            *dst = rd(src);
            dst = dst.add(1);
            src = src.add(1);
        } else {
            let op = rd(src);
            *dst = op;
            dst = dst.add(1);
            src = src.add(1);
            // switch fallthrough: 6/7/8/9/10/14 -> 12 -> 5
            let extra = match op {
                6 | 7 | 8 | 9 | 10 | 14 => 4,
                12 => 3,
                5 => 2,
                _ => 0,
            };
            for _ in 0..extra {
                *dst = rd(src);
                dst = dst.add(1);
                src = src.add(1);
            }
        }
    }
    *dst = 0;
    dst
}

/// `HSD_SisLib_803A6530` (0x803A6530): copy entry `src_idx` of font `font_idx`'s table into `dst_idx`.
pub unsafe fn HSD_SisLib_803A6530(font_idx: i32, dst_idx: i32, src_idx: i32) -> *mut u8 {
    let sis_table = ext::HSD_SisLib_804D1124[font_idx as usize] as *mut *mut u8;
    HSD_SisLib_803A6478(*sis_table.offset(dst_idx as isize), *sis_table.offset(src_idx as isize))
}

/// `HSD_SisLib_803A660C` (0x803A660C): append entry `src_idx` to the end of entry `dst_idx`.
pub unsafe fn HSD_SisLib_803A660C(font_idx: i32, dst_idx: i32, src_idx: i32) {
    let sis_table = ext::HSD_SisLib_804D1124[font_idx as usize] as *mut *mut u8;
    let mut dst = *sis_table.offset(dst_idx as isize);
    let src = *sis_table.offset(src_idx as isize);

    while *dst != 0 {
        if *dst >= 0x20 {
            dst = dst.add(2);
        } else {
            match *dst {
                6 | 7 | 8 | 9 | 10 | 14 => dst = dst.add(5),
                12 => dst = dst.add(4),
                5 => dst = dst.add(3),
                _ => dst = dst.add(1),
            }
        }
    }
    HSD_SisLib_803A6478(dst, src);
}

/// `HSD_SisLib_803A6754` (0x803A6754): a text with its own growable SIS buffer.
///
/// Divergence: the C allocates 0x10 bytes for the `SisBlock` (GC `sizeof` 0xC plus the counter
/// word); the native `SisBlock` is 24 bytes, so that is what is requested.
pub unsafe fn HSD_SisLib_803A6754(font_idx: i32, context_id: i32) -> *mut HSD_Text {
    let text = ext::HSD_SisLib_803A5ACC(font_idx, context_id, 0.0, 0.0, 0.0, 640.0, 480.0);
    let alloc = ext::HSD_SisLib_Alloc(core::mem::size_of::<SisBlock>() as i32) as *mut SisBlock;
    (*text).alloc_data = alloc;
    let buffer = ext::HSD_SisLib_Alloc(0x80) as *mut HSD_Text;
    (*alloc).data = buffer;
    (*alloc).next = buffer as *mut SisBlock;
    (*alloc).size = 0x80;
    // `*(u8*)&alloc->next->next = 0`: the first byte of the new buffer.
    *(buffer as *mut u8) = 0;
    // `*(&alloc->size + 1) = 0`: the counter word after `size` (`sisLib_803A7664_t.xC`).
    (*(alloc as *mut sisLib_803A7664_t)).xC = 0;
    ext::HSD_SisLib_803A6368(text, 0);
    (*text).sis_buffer = (*alloc).data as *mut SIS;
    text
}

/// `sisBeginLine` (static).
unsafe fn sisBeginLine(data: *mut u8, out_idx: &mut i32, has_kerning: &mut i32) {
    let mut state = *has_kerning;
    let mut idx = *out_idx;

    if state == 0 {
        for b in [0xAu8, 0xF4, 0x0] {
            *data.offset(idx as isize) = b;
            idx += 1;
        }
        state += 1;
        *data.offset(idx as isize) = 0x0;
        idx += 1;
        *data.offset(idx as isize) = 0x0;
        idx += 1;
    }
    *has_kerning = state;
    *out_idx = idx;
}

/// `sisEndKerning` (static).
unsafe fn sisEndKerning(data: *mut u8, out_idx: &mut i32, has_kerning: &mut i32) {
    let idx = *out_idx;
    *data.offset(idx as isize) = 0xB;
    *has_kerning = 0;
    *out_idx = idx + 1;
}

/// `HSD_SisLib_803A67EC` (0x803A67EC): ASCII / Shift-JIS to SIS glyph codes. Returns the encoded
/// length. `string` must be NUL-terminated or at least 0x80 bytes (the C reads up to 0x80).
pub unsafe fn HSD_SisLib_803A67EC(data: *mut u8, string: *const u8) -> i32 {
    let mut has_kerning: i32 = 0;
    let mut out_idx: i32 = 0;
    let mut in_idx: i32 = 0;

    while in_idx < 0x80 {
        let cur_char = *string.offset(in_idx as isize);
        if cur_char == 0 {
            if has_kerning != 0 {
                *data.offset(out_idx as isize) = 0xB;
                out_idx += 1;
            }
            break;
        }
        let (sjis_hi, sjis_lo): (u8, u8);
        if cur_char == b' ' {
            sisBeginLine(data, &mut out_idx, &mut has_kerning);
            sjis_hi = 0x81;
            sjis_lo = 0x40;
        } else if cur_char == b'"' {
            sisEndKerning(data, &mut out_idx, &mut has_kerning);
            sjis_hi = 0x81;
            sjis_lo = 0x68;
        } else if cur_char == b'\'' {
            sisEndKerning(data, &mut out_idx, &mut has_kerning);
            sjis_hi = 0x81;
            sjis_lo = 0x66;
        } else if cur_char == b',' {
            sisEndKerning(data, &mut out_idx, &mut has_kerning);
            sjis_hi = 0x81;
            sjis_lo = 0x43;
        } else if cur_char == b'-' {
            sisEndKerning(data, &mut out_idx, &mut has_kerning);
            sjis_hi = 0x81;
            sjis_lo = 0x7C;
        } else if cur_char == b'.' {
            sisBeginLine(data, &mut out_idx, &mut has_kerning);
            sjis_hi = 0x81;
            sjis_lo = 0x44;
        } else if (b'0'..=b'9').contains(&cur_char) {
            sisBeginLine(data, &mut out_idx, &mut has_kerning);
            sjis_hi = 0x82;
            sjis_lo = cur_char.wrapping_add(0x1F);
        } else if cur_char == b':' {
            sisBeginLine(data, &mut out_idx, &mut has_kerning);
            sjis_hi = 0x81;
            sjis_lo = 0x46;
        } else if (b'A'..=b'Z').contains(&cur_char) {
            sisEndKerning(data, &mut out_idx, &mut has_kerning);
            sjis_hi = 0x82;
            sjis_lo = cur_char.wrapping_add(0x1F);
        } else if (b'a'..=b'z').contains(&cur_char) {
            sisEndKerning(data, &mut out_idx, &mut has_kerning);
            sjis_hi = 0x82;
            sjis_lo = cur_char.wrapping_add(0x20);
        } else {
            sisEndKerning(data, &mut out_idx, &mut has_kerning);
            // two-byte Shift-JIS: the C reads the trail byte through its second cursor.
            sjis_hi = cur_char;
            sjis_lo = *string.offset(in_idx as isize + 1);
            in_idx += 1;
        }
        for lut_idx in 0..HSD_SisLib_FontAtlas_COUNT {
            if sjis_hi == lbl_8040C8C0[lut_idx * 2] && sjis_lo == lbl_8040C8C0[lut_idx * 2 + 1] {
                *data.offset(out_idx as isize) = HSD_SisLib_8040C680[lut_idx * 2];
                out_idx += 1;
                *data.offset(out_idx as isize) = HSD_SisLib_8040C680[lut_idx * 2 + 1];
                out_idx += 1;
                break;
            }
        }
        in_idx += 1;
    }
    *data.offset(out_idx as isize) = 0;
    out_idx
}

/// `vsnprintf((char*)buffer, sizeof buffer, ...)` result: the formatted text in the C's 128-byte
/// buffer (127 characters at most, NUL-terminated).
fn format_buffer(s: &str) -> [u8; 128] {
    let mut buffer = [0u8; 128];
    for (d, b) in buffer.iter_mut().take(127).zip(s.bytes()) {
        *d = b;
    }
    buffer
}

#[inline]
fn f32_to_u8(v: f32) -> u8 {
    (v as i32) as u8
}

/// Grow `alloc`'s buffer to `new_size` bytes, copying the used part; shared tail of the two
/// text-append paths. Returns the old `data` pointer.
unsafe fn sis_grow(text: *mut HSD_Text, alloc: *mut SisBlock, old_buf: *mut u8) {
    let new_buf = ext::HSD_SisLib_Alloc((*alloc).size as i32) as *mut u8;
    let copy_src = old_buf;
    let used = ((*alloc).next as *mut u8).offset_from((*alloc).data as *mut u8) as i32 + 1;
    let mut copy_idx = 0;
    while copy_idx < used {
        *new_buf.offset(copy_idx as isize) = *copy_src.offset(copy_idx as isize);
        copy_idx += 1;
    }
    (*alloc).data = new_buf as *mut HSD_Text;
    (*text).sis_buffer = new_buf as *mut SIS;
    (*alloc).next = new_buf.offset(((*alloc).next as *mut u8).offset_from(old_buf)) as *mut SisBlock;
    ext::HSD_SisLib_Free(old_buf as *mut c_void);
}

/// `HSD_SisLib_803A6B98` (0x803A6B98): append a positioned, coloured, scaled string entry.
/// `fmt` is the formatted text (`None` for the C's NULL format). Returns the new entry index.
pub unsafe fn HSD_SisLib_803A6B98(text: *mut HSD_Text, x: f32, y: f32, fmt: Option<&str>) -> i32 {
    let mut encoded = [0u8; 128];
    let mut encoded_len: i32 = 0;
    let alloc = (*text).alloc_data;
    encoded[0] = 0;
    if let Some(f) = fmt {
        let buffer = format_buffer(f);
        encoded_len = HSD_SisLib_803A67EC(encoded.as_mut_ptr(), buffer.as_ptr());
    }

    let old_size = (*alloc).size;
    let old_buf = (*alloc).data as *mut u8;
    let mut required_size = (((*alloc).next as *mut u8).offset_from(old_buf) as i32 as u32).wrapping_add(0x11);
    required_size = (encoded_len as u32).wrapping_add(required_size);
    if old_size < required_size {
        (*alloc).size = old_size.wrapping_add(((required_size - old_size) >> 7).wrapping_add(1) << 7);
        sis_grow(text, alloc, old_buf);
    }

    let mut cur = (*alloc).next as *mut u8;
    macro_rules! put {
        ($v:expr) => {{
            *cur = $v;
            cur = cur.add(1);
        }};
    }
    put!(7);
    let x_coord = x as i32 as i16 as i32;
    put!((x_coord >> 8) as u8);
    put!(x_coord as u8);
    let y_coord = y as i32 as i16 as i32;
    put!((y_coord >> 8) as u8);
    put!(y_coord as u8);
    put!(0xC);
    put!((*text).text_color.r);
    put!((*text).text_color.g);
    put!((*text).text_color.b);
    put!(0xE);
    put!((*text).x34.x as i32 as u8);
    put!(f32_to_u8(256.0f32 * (*text).x34.x));
    put!((*text).x34.y as i32 as u8);
    put!(f32_to_u8(256.0f32 * (*text).x34.y));
    for i in 0..encoded_len {
        put!(encoded[i as usize]);
    }
    put!(0xF);
    put!(0xD);
    *cur = 0;
    (*alloc).next = cur as *mut SisBlock;
    let counter = &mut (*(alloc as *mut sisLib_803A7664_t)).xC;
    let old = *counter;
    *counter = old.wrapping_add(1);
    old as i32
}

/// `fn_803A6FEC` (0x803A6FEC): find entry `entry_idx` (the Nth opcode-7 record) in a SIS stream.
/// Returns NULL at the end of the stream; `out_size` receives the length of the entry's text.
pub unsafe fn fn_803A6FEC(mut sis_data: *mut u8, mut entry_idx: i32, out_size: Option<&mut i32>) -> *mut u8 {
    loop {
        match *sis_data {
            0 => return core::ptr::null_mut(),
            12 => sis_data = sis_data.add(3),
            11 | 13 | 15 => {}
            7 => {
                entry_idx -= 1;
                if entry_idx < 0 {
                    break;
                }
                sis_data = sis_data.add(4);
            }
            10 | 14 => sis_data = sis_data.add(4),
            _ => sis_data = sis_data.add(1),
        }
        sis_data = sis_data.add(1);
    }
    if let Some(out) = out_size {
        *out = 0;
        let mut scan_ptr = sis_data.add(0xE);
        loop {
            let mut char_size: u8 = 0;
            if *scan_ptr >= 32 {
                char_size = 2;
            } else if *scan_ptr == 10 {
                char_size = 5;
            } else if *scan_ptr == 11 {
                char_size = 1;
            }
            *out += char_size as i32;
            scan_ptr = scan_ptr.add(char_size as usize);
            if char_size == 0 {
                break;
            }
        }
    }
    sis_data
}

/// `HSD_SisLib_803A70A0` (0x803A70A0): replace the text of entry `entry_idx` (`fmt` already
/// formatted; `None` clears it). Returns 1 if the entry exists, else 0.
pub unsafe fn HSD_SisLib_803A70A0(text: *mut HSD_Text, entry_idx: i32, fmt: Option<&str>) -> i32 {
    let mut encoded = [0u8; 128];
    let mut old_size: i32 = 0;
    let mut result: i32 = 0;

    let entry = fn_803A6FEC((*text).sis_buffer as *mut u8, entry_idx, Some(&mut old_size));
    if !entry.is_null() {
        let alloc = (*text).alloc_data;
        let mut playhead = entry.add(0xE);
        let new_size: i32;
        if let Some(f) = fmt {
            let buffer = format_buffer(f);
            new_size = HSD_SisLib_803A67EC(encoded.as_mut_ptr(), buffer.as_ptr());
        } else {
            new_size = 0;
        }
        if old_size < new_size {
            result = new_size - old_size;
            let tail_len = ((*alloc).next as *mut u8).offset_from(playhead) as i32;
            let old_buf = (*alloc).data as *mut u8;
            let required_size = (new_size.wrapping_add(((*alloc).next as *mut u8).offset_from(old_buf) as i32).wrapping_add(1)) as u32;
            if (*alloc).size < required_size {
                let sz = (*alloc).size;
                (*alloc).size = sz.wrapping_add(((required_size - sz) >> 7).wrapping_add(1) << 7);
                sis_grow(text, alloc, old_buf);
                playhead = ((*alloc).next as *mut u8).offset(-(tail_len as isize));
            }
            let mut i = tail_len;
            while i > 0 {
                *playhead.offset((result + i) as isize) = *playhead.offset(i as isize);
                i -= 1;
            }
            (*alloc).next = ((*alloc).next as *mut u8).offset(result as isize) as *mut SisBlock;
        } else if old_size > new_size {
            let shrink_size = old_size - new_size;
            let tail_len = ((*alloc).next as *mut u8).offset_from(playhead) as i32;
            let mut i = 0;
            while i < tail_len {
                *playhead.offset(i as isize) = *playhead.offset((shrink_size + i) as isize);
                i += 1;
            }
            (*alloc).next = ((*alloc).next as *mut u8).offset(-(shrink_size as isize)) as *mut SisBlock;
        }
        for i in 0..new_size {
            *playhead = encoded[i as usize];
            playhead = playhead.add(1);
        }
        *playhead = 0xF;
        result = 1;
    }
    result
}

/// `HSD_SisLib_803A746C` (0x803A746C): set the position of entry `entry_idx`.
pub unsafe fn HSD_SisLib_803A746C(text: *mut HSD_Text, entry_idx: i32, new_x: f32, new_y: f32) {
    let entry = fn_803A6FEC((*text).sis_buffer as *mut u8, entry_idx, None);
    if !entry.is_null() {
        let p = entry.add(1);
        let x = new_x as i32 as i16 as i32;
        *p = (x >> 8) as u8;
        let y = new_y as i32 as i16 as i32;
        *p.add(1) = x as u8;
        *p.add(2) = (y >> 8) as u8;
        *p.add(3) = y as u8;
    }
}

/// `HSD_SisLib_803A74F0` (0x803A74F0): set the colour of entry `entry_idx`.
pub unsafe fn HSD_SisLib_803A74F0(text: *mut HSD_Text, entry_idx: i32, color_rgb: *const GXColor) {
    let entry = fn_803A6FEC((*text).sis_buffer as *mut u8, entry_idx, None);
    if !entry.is_null() {
        let color_ptr = entry.add(5);
        *color_ptr.add(1) = (*color_rgb).r;
        *color_ptr.add(2) = (*color_rgb).g;
        *color_ptr.add(3) = (*color_rgb).b;
    }
}

/// `HSD_SisLib_803A7548` (0x803A7548): set the scale of entry `entry_idx`.
pub unsafe fn HSD_SisLib_803A7548(text: *mut HSD_Text, entry_idx: i32, scale_x: f32, scale_y: f32) {
    let entry = fn_803A6FEC((*text).sis_buffer as *mut u8, entry_idx, None);
    if !entry.is_null() {
        // `*++scale_ptr = ...` starts from entry + 9.
        let scale_ptr = entry.add(10);
        *scale_ptr = f32_to_u8(scale_x);
        *scale_ptr.add(1) = f32_to_u8(256.0f32 * scale_x);
        *scale_ptr.add(2) = f32_to_u8(scale_y);
        *scale_ptr.add(3) = f32_to_u8(256.0f32 * scale_y);
    }
}

/// `HSD_SisLib_803A75E0` (0x803A75E0): clear entry `entry_idx` and reset its colour to the text's.
pub unsafe fn HSD_SisLib_803A75E0(text: *mut HSD_Text, entry_idx: i32) {
    if HSD_SisLib_803A70A0(text, entry_idx, None) != 0 {
        let color = (*text).text_color;
        let entry = fn_803A6FEC((*text).sis_buffer as *mut u8, entry_idx, None);
        if !entry.is_null() {
            let p = entry.add(5);
            *p.add(1) = color.r;
            *p.add(2) = color.g;
            *p.add(3) = color.b;
        }
    }
}

/// `HSD_SisLib_803A7664` (0x803A7664): empty a text's entry buffer.
pub unsafe fn HSD_SisLib_803A7664(text: *mut HSD_Text) {
    let data = (*text).alloc_data as *mut sisLib_803A7664_t;
    (*data).x0 = (*data).x4;
    *((*data).x0 as *mut u8) = 0;
    (*data).xC = 0;
}
