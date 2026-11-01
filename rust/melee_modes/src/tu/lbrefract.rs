//! lbrefract.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_core::fp::fmadds;
#[allow(unused_imports)]
use melee_types::lbRefract_CallbackData;
#[allow(unused_imports)]
use crate::lb_r10::{tile_base};

// ---- folded from lb_r10.rs (consolidation steps 21/22) ----
/// lbrefract.c `fn_80021F70_fp` (static): writes one RGB565 texel of a 4x4-tiled texture.
///
/// # Safety
/// `data.buffer` must cover the addressed tile.
pub unsafe fn fn_80021F70_fp(data: &lbRefract_CallbackData, row: u32, col: u32, r: u8, g: u8, b: u32) {
    melee_core::ct!("fn_80021F70_fp");
    let base = tile_base(data, col >> 2, (row >> 2).wrapping_mul(32)) as *mut u16;
    let offset = (row & 3) + (col & 3) * 4;
    let v = (b / 8) | (((g as i32 * 8) & 0x7E0 & !0xF800) | (((r as i32) << 8) & 0xF800)) as u32;
    base.add(offset as usize).write_unaligned(v as u16);
}

/// lbrefract.c `fn_80021FB4_fp` (static): writes one RGBA8 texel (AR and GB halves 32 bytes apart).
///
/// # Safety
/// `data.buffer` must cover the addressed tile.
pub unsafe fn fn_80021FB4_fp(data: &lbRefract_CallbackData, row: u32, col: u32, arg6: u8, arg7: u8, arg8: u8, arg9: u8) {
    melee_core::ct!("fn_80021FB4_fp");
    let base = tile_base(data, col >> 2, (row >> 2).wrapping_mul(64));
    let i = ((row & 3) + (col & 3) * 4) as usize;
    *base.add(2 * i) = arg9;
    *base.add(2 * i + 1) = arg6;
    *base.add(2 * (i + 16)) = arg7;
    *base.add(2 * (i + 16) + 1) = arg8;
}

/// lbrefract.c `fn_80021FF8_fp` (static): reads one IA8 texel (alpha, intensity) into the outputs
/// that are non-NULL; the first two get 0xFF.
///
/// # Safety
/// `data.buffer` must cover the addressed tile.
pub unsafe fn fn_80021FF8_fp(
    data: &lbRefract_CallbackData,
    row: u32,
    col: u32,
    arg3: Option<&mut i32>,
    arg4: Option<&mut i32>,
    arg5: Option<&mut i32>,
    arg6: Option<&mut i32>,
) {
    melee_core::ct!("fn_80021FF8_fp");
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

/// lbrefract.c `fn_8002206C_fp` (static): reads one RGB565 texel back as 3/2/3-bit presence flags.
///
/// # Safety
/// `data.buffer` must cover the addressed tile.
pub unsafe fn fn_8002206C_fp(
    data: &lbRefract_CallbackData,
    row: i32,
    col: u32,
    arg3: Option<&mut i32>,
    arg4: Option<&mut i32>,
    arg5: Option<&mut i32>,
    arg6: Option<&mut i32>,
) {
    melee_core::ct!("fn_8002206C_fp");
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

// ---- folded from lb_r6.rs (consolidation steps 21/22) ----
/// lbrefract.c `my_fmodf_fp` (static inline): `a - b * (s64)(a / b)`, or `a` when |b| > |a|.
/// The shipping build contracts the last step into one fused site: fmuladd(-b, (float) q, a).
/// `(s64)` of a non-finite or out-of-range quotient saturates (gwtool's lowering, Rust `as`).
pub fn my_fmodf_fp(a: f32, b: f32) -> f32 {
    melee_core::ct!("my_fmodf_fp");
    let fa = a.abs();
    let fb = b.abs();
    if fb > fa {
        return a;
    }
    let quotient = (a / b) as i64;
    fmadds(-b, quotient as f32, a)
}
