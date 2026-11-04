//! pc/platform/gw_uigen.c's text art: `gw_UI_TextI4` (a name drawn into an I4 texture, white on black, centred:
//! the results name strips and a Geno define's stock-icon initials) over Windows GDI, and png2gx.py's I4 tiler
//! (`gw_UI_RasterI4`). Render-only: the bytes go to a texture, never into simulation state.
#![allow(non_snake_case, clippy::upper_case_acronyms)]

/// png2gx.py _mask_level (alpha is always 255 here: the luma).
fn mask_level(r: i32, g: i32, b: i32, a: i32) -> i32 {
    if a != 255 {
        return a;
    }
    let l = (r * 299 + g * 587 + b * 114) / 1000;
    if a < l { a } else { l }
}
fn i4_level(p: &[u8]) -> u8 {
    ((mask_level(p[0] as i32, p[1] as i32, p[2] as i32, p[3] as i32) * 15 + 127) / 255) as u8
}

/// gw_UI_I4Size.
pub fn i4_size(w: i32, h: i32) -> i32 {
    if w > 0 && h > 0 && w % 8 == 0 && h % 8 == 0 { (w / 8) * (h / 8) * 32 } else { 0 }
}

/// gw_UI_RasterI4: 8x8 tiles (ty then tx), y-outer, a byte per texel pair (high nibble first).
pub fn raster_i4(w: i32, h: i32, rgba: &[u8], dst: &mut [u8]) {
    if i4_size(w, h) == 0 {
        return;
    }
    let mut o = 0;
    for ty in (0..h).step_by(8) {
        for tx in (0..w).step_by(8) {
            for y in ty..ty + 8 {
                for x in (tx..tx + 8).step_by(2) {
                    let hi = ((y * w + x) * 4) as usize;
                    let lo = ((y * w + x + 1) * 4) as usize;
                    dst[o] = (i4_level(&rgba[hi..hi + 4]) << 4) | i4_level(&rgba[lo..lo + 4]);
                    o += 1;
                }
            }
        }
    }
}

#[cfg(windows)]
mod gdi {
    use core::ffi::c_void;
    pub type H = *mut c_void;
    #[repr(C)]
    pub struct BITMAPINFOHEADER {
        pub biSize: u32,
        pub biWidth: i32,
        pub biHeight: i32,
        pub biPlanes: u16,
        pub biBitCount: u16,
        pub biCompression: u32,
        pub biSizeImage: u32,
        pub biXPelsPerMeter: i32,
        pub biYPelsPerMeter: i32,
        pub biClrUsed: u32,
        pub biClrImportant: u32,
    }
    #[repr(C)]
    pub struct BITMAPINFO {
        pub bmiHeader: BITMAPINFOHEADER,
        pub bmiColors: [u32; 1],
    }
    #[repr(C)]
    #[derive(Default)]
    pub struct SIZE {
        pub cx: i32,
        pub cy: i32,
    }
    /// TEXTMETRICA: only tmAveCharWidth (the sixth i32) is read.
    #[repr(C)]
    pub struct TEXTMETRICA {
        pub l: [i32; 11],
        pub b: [u8; 9],
    }
    #[link(name = "gdi32")]
    extern "system" {
        pub fn CreateCompatibleDC(dc: H) -> H;
        pub fn CreateDIBSection(dc: H, bi: *const BITMAPINFO, usage: u32, bits: *mut *mut c_void, sec: H, off: u32) -> H;
        pub fn SelectObject(dc: H, o: H) -> H;
        pub fn DeleteObject(o: H) -> i32;
        pub fn DeleteDC(dc: H) -> i32;
        pub fn CreateFontA(
            h: i32, w: i32, esc: i32, ori: i32, weight: i32, it: u32, ul: u32, so: u32, cs: u32, op: u32, cp: u32, q: u32,
            pf: u32, face: *const u8,
        ) -> H;
        pub fn SetTextCharacterExtra(dc: H, e: i32) -> i32;
        pub fn GetTextExtentPoint32A(dc: H, s: *const u8, n: i32, sz: *mut SIZE) -> i32;
        pub fn GetTextMetricsA(dc: H, tm: *mut TEXTMETRICA) -> i32;
        pub fn SetBkMode(dc: H, m: i32) -> i32;
        pub fn SetTextColor(dc: H, c: u32) -> u32;
        pub fn TextOutA(dc: H, x: i32, y: i32, s: *const u8, n: i32) -> i32;
        pub fn GdiFlush() -> i32;
    }
}

/// gw_UI_TextI4: `text` into an I4 texture of w x round8(h) texels at `dst` (zeroed first). `style` 0 the
/// per-player name (heavy Arial), 1 the winner banner (bold Times, letter-spaced, a grey outline). A name too wide is
/// squeezed. 1 when drawn, 0 on any failure (dst left blank).
///
/// # Safety
/// `text` is NUL-terminated; `dst` holds i4_size(w, round8(h)) bytes.
pub unsafe fn UI_TextI4(text: *const i8, w: i32, h: i32, style: i32, dst: *mut core::ffi::c_void) -> i32 {
    let hp = (h + 7) & !7;
    let size = i4_size(w, hp);
    if text.is_null() || dst.is_null() || size == 0 || w > 1024 || h > 256 {
        return 0;
    }
    let out = core::slice::from_raw_parts_mut(dst as *mut u8, size as usize);
    out.fill(0);
    let t = core::ffi::CStr::from_ptr(text).to_bytes();
    if t.is_empty() {
        return 0;
    }
    #[cfg(windows)]
    {
        use gdi::*;
        let len = t.len() as i32;
        let null = core::ptr::null_mut();
        let dc = CreateCompatibleDC(null);
        let bi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: core::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -hp,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: 0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [0],
        };
        let mut bits: *mut core::ffi::c_void = null;
        let bmp = if dc.is_null() { null } else { CreateDIBSection(dc, &bi, 0, &mut bits, null, 0) };
        let mut ok = 0;
        let (mut old_bmp, mut font, mut old_font) = (null, null, null);
        'done: {
            if bmp.is_null() || bits.is_null() {
                break 'done;
            }
            old_bmp = SelectObject(dc, bmp);
            let font_h = if style == 1 { h - 2 } else { h - 3 };
            let mut spacing = if style == 1 { 3 } else { 0 };
            let face: &[u8] = if style == 1 { b"Times New Roman\0" } else { b"Arial\0" };
            let weight = if style == 1 { 700 } else { 900 };
            // ANSI_CHARSET 0, OUT_TT_PRECIS 4, CLIP_DEFAULT_PRECIS 0, ANTIALIASED_QUALITY 4, VARIABLE_PITCH 2
            font = CreateFontA(-font_h, 0, 0, 0, weight, 0, 0, 0, 0, 4, 0, 4, 2, face.as_ptr());
            if font.is_null() {
                break 'done;
            }
            old_font = SelectObject(dc, font);
            SetTextCharacterExtra(dc, spacing);
            let mut ext = SIZE::default();
            GetTextExtentPoint32A(dc, t.as_ptr(), len, &mut ext);
            if ext.cx > w - 4 {
                spacing = 0;
                SetTextCharacterExtra(dc, 0);
                let mut tm = TEXTMETRICA { l: [0; 11], b: [0; 9] };
                GetTextMetricsA(dc, &mut tm);
                GetTextExtentPoint32A(dc, t.as_ptr(), len, &mut ext);
                let ave = tm.l[5];
                let avg = if ext.cx > 0 { (ave as i64 * (w - 4) as i64 / ext.cx as i64) as i32 } else { ave };
                SelectObject(dc, old_font);
                DeleteObject(font);
                font = CreateFontA(-font_h, if avg > 1 { avg } else { 1 }, 0, 0, weight, 0, 0, 0, 0, 4, 0, 4, 2, face.as_ptr());
                if font.is_null() {
                    old_font = null;
                    break 'done;
                }
                old_font = SelectObject(dc, font);
                GetTextExtentPoint32A(dc, t.as_ptr(), len, &mut ext);
            }
            SetBkMode(dc, 1);
            let px = (w - ext.cx + spacing) / 2;
            let py = (h - ext.cy) / 2;
            if style == 1 {
                const OFF: [[i32; 2]; 8] = [[-1, -1], [0, -1], [1, -1], [-1, 0], [1, 0], [-1, 1], [0, 1], [1, 1]];
                SetTextColor(dc, 110 | 110 << 8 | 110 << 16);
                for o in OFF {
                    TextOutA(dc, px + o[0], py + o[1], t.as_ptr(), len);
                }
            }
            SetTextColor(dc, 0xFF_FFFF);
            TextOutA(dc, px, py, t.as_ptr(), len);
            GdiFlush();
            let src = core::slice::from_raw_parts(bits as *const u8, (w * hp * 4) as usize);
            let mut rgba = vec![0u8; (w * hp * 4) as usize];
            for y in 0..h {
                for x in 0..w {
                    let i = ((y * w + x) * 4) as usize;
                    let v = src[i].max(src[i + 1]).max(src[i + 2]);
                    rgba[i..i + 4].copy_from_slice(&[v, v, v, 255]);
                }
            }
            raster_i4(w, hp, &rgba, out);
            ok = 1;
        }
        if !old_font.is_null() {
            SelectObject(dc, old_font);
        }
        if !font.is_null() {
            DeleteObject(font);
        }
        if !old_bmp.is_null() {
            SelectObject(dc, old_bmp);
        }
        if !bmp.is_null() {
            DeleteObject(bmp);
        }
        if !dc.is_null() {
            DeleteDC(dc);
        }
        ok
    }
    #[cfg(not(windows))]
    {
        0
    }
}
