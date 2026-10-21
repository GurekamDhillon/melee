//! Round 10 (night 3 round 6): the lbrefract texel callbacks and lbaudio_ax `fn_80025E38`,
//! verified bit for bit against the C by `tests/oracle_r10.rs`.
use melee_types::{lbRefract_CallbackData, HSD_GObj};

use crate::lb_r4::lbAudioAx_UserData;

/// `(u8*) data->buffer + c * data->row_stride + r * tile` in the C's u32 arithmetic.
unsafe fn tile_base(data: &lbRefract_CallbackData, c: u32, r_off: u32) -> *mut u8 {
    (data.buffer as *mut u8).add(c.wrapping_mul(data.row_stride).wrapping_add(r_off) as usize)
}

/// lbrefract.c `fn_80021F70` (static): writes one RGB565 texel of a 4x4-tiled texture.
///
/// # Safety
/// `data.buffer` must cover the addressed tile.
pub unsafe fn fn_80021F70(data: &lbRefract_CallbackData, row: u32, col: u32, r: u8, g: u8, b: u32) {
    let base = tile_base(data, col >> 2, (row >> 2).wrapping_mul(32)) as *mut u16;
    let offset = (row & 3) + (col & 3) * 4;
    let v = (b / 8) | (((g as i32 * 8) & 0x7E0 & !0xF800) | (((r as i32) << 8) & 0xF800)) as u32;
    base.add(offset as usize).write_unaligned(v as u16);
}

/// lbrefract.c `fn_80021FB4` (static): writes one RGBA8 texel (AR and GB halves 32 bytes apart).
///
/// # Safety
/// `data.buffer` must cover the addressed tile.
pub unsafe fn fn_80021FB4(data: &lbRefract_CallbackData, row: u32, col: u32, arg6: u8, arg7: u8, arg8: u8, arg9: u8) {
    let base = tile_base(data, col >> 2, (row >> 2).wrapping_mul(64));
    let i = ((row & 3) + (col & 3) * 4) as usize;
    *base.add(2 * i) = arg9;
    *base.add(2 * i + 1) = arg6;
    *base.add(2 * (i + 16)) = arg7;
    *base.add(2 * (i + 16) + 1) = arg8;
}

/// lbrefract.c `fn_80021FF8` (static): reads one IA8 texel (alpha, intensity) into the outputs
/// that are non-NULL; the first two get 0xFF.
///
/// # Safety
/// `data.buffer` must cover the addressed tile.
pub unsafe fn fn_80021FF8(
    data: &lbRefract_CallbackData,
    row: u32,
    col: u32,
    arg3: Option<&mut i32>,
    arg4: Option<&mut i32>,
    arg5: Option<&mut i32>,
    arg6: Option<&mut i32>,
) {
    let base = tile_base(data, col >> 2, (row >> 2).wrapping_mul(32));
    let offset = (((row & 3) + (col & 3) * 4) * 2) as usize;
    if let Some(p) = arg3 {
        *p = 0xFF;
    }
    if let Some(p) = arg4 {
        *p = 0xFF;
    }
    if let Some(p) = arg5 {
        *p = *base.add(offset + 1) as i32;
    }
    if let Some(p) = arg6 {
        *p = *base.add(offset) as i32;
    }
}

/// lbrefract.c `fn_8002206C` (static): reads one RGB565 texel back as 3/2/3-bit presence flags.
///
/// # Safety
/// `data.buffer` must cover the addressed tile.
pub unsafe fn fn_8002206C(
    data: &lbRefract_CallbackData,
    row: i32,
    col: u32,
    arg3: Option<&mut i32>,
    arg4: Option<&mut i32>,
    arg5: Option<&mut i32>,
    arg6: Option<&mut i32>,
) {
    let r = (row as u32) & !3;
    let c = col & !3;
    let base = tile_base(data, c / 4, r.wrapping_mul(8)) as *mut u16;
    let idx = ((row & 3) as u32).wrapping_add((col & 3) * 4);
    let pixel = base.add(idx as usize).read_unaligned() as u32;
    if let Some(p) = arg6 {
        *p = 0xFF;
    }
    if let Some(p) = arg3 {
        *p = if ((pixel >> 8) & 0xF8) | (pixel & 0x8000) != 0 { 7 } else { 0 };
    }
    if let Some(p) = arg4 {
        *p = if ((pixel >> 3) & 0xFC) | (pixel & 0x400) != 0 { 3 } else { 0 };
    }
    if let Some(p) = arg5 {
        *p = if pixel & 0xF8 != 0 { 7 } else { 0 };
    }
}

/// lbaudio_ax.c `fn_80025E38` (static): the volume fade from `start_vol` to `end_vol` over
/// `end_frame` frames, `VOL_MAX` (0x7F) once past the end. No fused site (lower.py: 0 fmuladd);
/// the float -> int conversion saturates (NaN -> 0), as gwtool's lowering does.
pub fn fn_80025E38(gobj: Option<&HSD_GObj>) -> bool {
    if let Some(g) = gobj {
        // SAFETY: a sound gobj's user_data is its lbAudioAx_UserData (or NULL).
        let ud = unsafe { (melee_types::inline::HSD_GObjGetUserData(g) as *mut lbAudioAx_UserData).as_mut() };
        if let Some(ud) = ud {
            if ud.current_frame <= ud.end_frame {
                let t = ud.current_frame as f32 / ud.end_frame as f32;
                let d = ud.end_vol as f32 - ud.start_vol as f32;
                let a = if d < 0.0 { -d } else { d };
                let k = (t * a) as i32;
                ud.vol = if ud.start_vol < ud.end_vol { ud.start_vol.wrapping_add(k) } else { ud.end_vol.wrapping_sub(k) };
            } else {
                ud.vol = 0x7F;
            }
        }
    }
    false
}
