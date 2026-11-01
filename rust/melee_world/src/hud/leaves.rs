
//! Night 3 round 2: self-contained leaves from src/melee/if (no callees). TARGET_PC bodies.
//! Checked bit for bit against the C by `tests/leaves_r2_vs_c.rs` (`cref/gen_leaves_r2_ref.sh`).
//! No fused sites. Integer `/` and `%` use the C build's semantics (`melee_runtime::cabi`),
//! `int` arithmetic wraps (`-fwrapv`), `(int)f` saturates (gwtool).
use melee_runtime::cabi::{f2i32, sdiv, srem};

/// DevText_NumToStr (if/textdraw.c): writes the decimal digits of `num` and a NUL into `out`
/// (at most 12 bytes). For `i32::MIN` the C's `-num` wraps and the digits come from negative
/// remainders; this writes the same bytes.
pub fn devtext_num_to_str(mut num: i32, out: &mut [u8]) {
    let mut s = 0usize;
    if num < 0 {
        out[s] = b'-';
        num = num.wrapping_neg();
        s += 1;
    }
    let mut e = s;
    let mut length = 0i32;
    loop {
        out[e] = (srem(num, 10) + b'0' as i32) as u8;
        num = sdiv(num, 10);
        e += 1;
        length += 1;
        if num == 0 {
            break;
        }
    }
    out[e] = 0;
    e -= 1;
    length >>= 1;
    for _ in 0..length {
        out.swap(s, e);
        e -= 1;
        s += 1;
    }
}

/// inlineA1 (if/ifnametag.c, static inline).
pub fn ifnametag_inline_a1(f: f32) -> f32 {
    let q = f2i32(f as f64);
    if q >= 16 {
        28.0
    } else {
        (srem(q, 4) + 20) as f32
    }
}

/// `foo` (if/ifstatus.c, static inline) with its toggle `ifStatus_804D6D61` (s8) passed in.
pub fn ifstatus_foo(flag: &mut i8, a: f32, b: f32) -> f32 {
    if *flag != 0 {
        *flag = 0;
        -a - b
    } else {
        *flag = 1;
        a + b
    }
}

pub use crate::tu::textlib::devtext_clamp;
pub use crate::tu::textdraw::devtext_strlen;
pub use crate::tu::ifstock::ifstock_802f89f8_count;
pub use crate::tu::ifstock::ifstock_802f89f8_digit;
pub use crate::tu::ifstock::ifstock_802f89f8_divisor;
