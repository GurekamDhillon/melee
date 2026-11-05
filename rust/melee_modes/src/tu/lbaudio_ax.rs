//! lbaudio_ax.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types as mt;
#[allow(unused_imports)]
use mt::HSD_GObj as HSD_GObj_lb_r4;
#[allow(unused_imports)]
use crate::lb_r4::{get_sound, pan_of};
#[allow(unused_imports)]
use melee_types::HSD_GObj;
#[allow(unused_imports)]
use crate::lb_r4::lbAudioAx_UserData;
#[allow(unused_imports)]
use melee_runtime::cabi;

// ---- folded from lb.rs (consolidation steps 21/22) ----
/// lbaudio_ax.c `calcPan_fp`: linear pan from left to right over `end` steps, clamped to 0..127.
/// All f32 operations (no fused site); `end == 0` gives NaN -> 0 via gwtool's saturation.
pub fn calcPan_fp(mut current: i32, end: i32, left: i32, right: i32) -> i32 {
    melee_core::ct!("calcPan_fp");
    if current > end {
        current = end;
    }
    if left != right {
        let mut result = if left <= right {
            left as f32 + current as f32 * (right as f32 - left as f32) / end as f32
        } else {
            right as f32 + current as f32 * (left as f32 - right as f32) / end as f32
        };
        if result < 0.0 {
            result = 0.0;
        }
        if result > 127.0 {
            result = 127.0;
        }
        return cabi::f2i32(result as f64);
    }
    64
}

// ---- folded from lb_r10.rs (consolidation steps 21/22) ----
/// lbaudio_ax.c `fn_80025E38_fp` (static): the volume fade from `start_vol` to `end_vol` over
/// `end_frame` frames, `VOL_MAX` (0x7F) once past the end. No fused site (lower.py: 0 fmuladd);
/// the float -> int conversion saturates (NaN -> 0), as gwtool's lowering does.
pub fn fn_80025E38_fp(gobj: Option<&HSD_GObj>) -> bool {
    melee_core::ct!("fn_80025E38_fp");
    if let Some(g) = gobj {
        // SAFETY: a sound gobj's user_data is its lbAudioAx_UserData (or NULL).
        let ud = unsafe { (melee_types::inline::HSD_GObjGetUserData(g) as *mut lbAudioAx_UserData).as_mut() };
        if let Some(ud) = ud {
            if ud.current_frame <= ud.end_frame {
                let t = ud.current_frame as f32 / ud.end_frame as f32;
                let d = ud.end_vol as f32 - ud.start_vol as f32;
                let a = if d < 0.0 { -d } else { d };
                let k = melee_core::ppc::fctiwz_f32(t * a);
                ud.vol = if ud.start_vol < ud.end_vol { ud.start_vol.wrapping_add(k) } else { ud.end_vol.wrapping_sub(k) };
            } else {
                ud.vol = 0x7F;
            }
        }
    }
    false
}

// ---- folded from lb_r4.rs (consolidation steps 21/22) ----
/// lbaudio_ax.c `fn_800253D8_fp` (static): pan left->right when x3C == 1, else mirrored.
pub fn fn_800253D8_fp(gobj: Option<&HSD_GObj_lb_r4>) -> bool {
    melee_core::ct!("fn_800253D8_fp");
    if let Some(ud) = gobj.and_then(get_sound) {
        ud.pan = if ud.x3C == 1.0 { pan_of(ud) } else { 0x7F - pan_of(ud) };
    }
    false
}

/// lbaudio_ax.c `fn_800256BC_fp` (static): the same body as `fn_800253D8_fp`.
pub fn fn_800256BC_fp(gobj: Option<&HSD_GObj_lb_r4>) -> bool {
    melee_core::ct!("fn_800256BC_fp");
    if let Some(ud) = gobj.and_then(get_sound) {
        ud.pan = if ud.x3C == 1.0 { pan_of(ud) } else { 0x7F - pan_of(ud) };
    }
    false
}

/// lbaudio_ax.c `fn_80025B44_fp` (static).
pub fn fn_80025B44_fp(gobj: Option<&HSD_GObj_lb_r4>) -> bool {
    melee_core::ct!("fn_80025B44_fp");
    if let Some(ud) = gobj.and_then(get_sound) {
        ud.pan = pan_of(ud);
    }
    false
}

/// lbaudio_ax.c `fn_80025CBC_fp` (static).
pub fn fn_80025CBC_fp(gobj: Option<&HSD_GObj_lb_r4>) -> bool {
    melee_core::ct!("fn_80025CBC_fp");
    if let Some(ud) = gobj.and_then(get_sound) {
        ud.pan = 0x7F - pan_of(ud);
    }
    false
}

/// lbaudio_ax.c `fn_800259EC_fp` (static): the first time only, `fn_800253D8_fp`.
pub fn fn_800259EC_fp(gobj: Option<&HSD_GObj_lb_r4>) -> bool {
    melee_core::ct!("fn_800259EC_fp");
    if let Some(g) = gobj {
        if let Some(ud) = get_sound(g) {
            if ud.x44 == 0 {
                ud.x44 = 1;
                fn_800253D8_fp(Some(g));
            }
        }
    }
    false
}

/// lbaudio_ax.c `fn_80025A98_fp` (static): the same body as `fn_800259EC_fp`.
pub fn fn_80025A98_fp(gobj: Option<&HSD_GObj_lb_r4>) -> bool {
    melee_core::ct!("fn_80025A98_fp");
    if let Some(g) = gobj {
        if let Some(ud) = get_sound(g) {
            if ud.x44 == 0 {
                ud.x44 = 1;
                fn_800253D8_fp(Some(g));
            }
        }
    }
    false
}

/// lbaudio_ax.c `lbAudioAx_800264E4_fp`: the sound's voice id, -1 without one.
pub fn lbAudioAx_800264E4_fp(gobj: Option<&HSD_GObj_lb_r4>) -> i32 {
    melee_core::ct!("lbAudioAx_800264E4_fp");
    match gobj {
        Some(g) => get_sound(g).map_or(-1, |ud| ud.voice_id),
        None => -1,
    }
}
