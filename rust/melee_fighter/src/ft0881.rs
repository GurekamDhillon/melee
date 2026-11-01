
//! Hand port, round 6: the voice / sfx-channel half of src/melee/ft/ft_0881.c (17 functions + `inline0`,
//! ft_800881D8 .. ft_80088C5C; the staling tail ft_800890BC .. ft_800892A0 is in `stale.rs`).
//! Each fighter owns seven sfx channels (`x2144`..`x215C`), channel base `0x1E/0x2A/0x36/0x42/
//! 0x4E/0x5A/0x72 + player_id*2 + is_sub_fighter`; `0x83D61` stops a channel.
//!
//! Fused sites (`tools/rust-port/fma_sites.py src/melee/ft/ft_0881.c`): none. Calls out of the
//! group go through `crate::hooks` (lbAudioAx: TODO(group1 lb); HSD_Randi: TODO(group1 HSD)).
//! TARGET_PC path ported: `FT_GENO_VOICE` (Geno voice owner returns early, sets nothing).
//! Checked against the C (cut verbatim, shipping contraction) by tests/ft0881_ref.rs.

use crate::ft0877::ft_80087d0c;

use crate::hooks as h;

use crate::types::*;

pub type G = *mut HSD_GObj;

pub(crate) const STOP: i32 = 0x83D61;

pub(crate) const NOP: i32 = 0x83D60;

#[inline(always)]
pub(crate) fn chan(f: &Fighter, base: i32) -> i32 {
    base + f.player_id as i32 * 2 + f.is_sub_fighter() as i32
}

#[inline(always)]
pub(crate) fn geno_voice(f: &mut Fighter, id: i32) -> bool {
    f.kind >= Ft_Kind_Mex0 && h::geno_define_voice_play_owner(f.kind as i32, id, f as *mut Fighter) != 0
}

#[inline(always)]
pub(crate) fn stop(f: &mut Fighter, base: i32) {
    h::lbaudioax_80023870(STOP, 0, 0x40, chan(f, base));
}

/// Shared body of 800881D8 / 80088328: `stop_first` is what 0x83D61 calls.
pub(crate) fn voice(f: &mut Fighter, sfx_id: i32, vol: u8, pan: u8, second: bool) {
    let allowed = f.x2225_b6() == 0 || f.kind == Ft_Kind_GameWatch || f.kind == Ft_Kind_Samus;
    if !allowed || sfx_id == NOP {
        return;
    }
    if sfx_id == STOP {
        if second { ft_800887cc(f) } else { ft_80088770(f) }
        return;
    }
    let id = ft_80087d0c(f, sfx_id);
    ft_80088770(f);
    ft_800887cc(f);
    if second {
        f.x2148 = h::lbaudioax_80023870(id, vol as i32, pan as i32, chan(f, 42));
    } else {
        f.x2144 = h::lbaudioax_80023870(id, vol as i32, pan as i32, chan(f, 0x1E));
    }
}

/// Shared body of 80088478 .. ftCo_800886D8: play on one channel, or stop it.
pub(crate) fn simple(f: &mut Fighter, sfx_id: i32, vol: u8, pan: u8, base: i32, slot: fn(&mut Fighter) -> &mut i32) {
    if sfx_id == NOP {
        return;
    }
    if sfx_id == STOP {
        stop(f, base);
        *slot(f) = -1;
        return;
    }
    let id = ft_80087d0c(f, sfx_id);
    let v = h::lbaudioax_80023870(id, vol as i32, pan as i32, chan(f, base));
    *slot(f) = v;
}

pub(crate) fn chan_slot(f: &mut Fighter, i: usize) -> &mut i32 {
    match i {
        0 => &mut f.x2144,
        1 => &mut f.x2148,
        2 => &mut f.x214C,
        3 => &mut f.x2150,
        4 => &mut f.x2154,
        5 => &mut f.x2158,
        _ => &mut f.x215C,
    }
}

pub use crate::tu::ft_0881::ft_800881d8;
pub use crate::tu::ft_0881::ft_80088328_fp as ft_80088328;
pub use crate::tu::ft_0881::ft_80088478_fp as ft_80088478;
pub use crate::tu::ft_0881::ft_80088510_fp as ft_80088510;
pub use crate::tu::ft_0881::ft_800885a8;
pub use crate::tu::ft_0881::ft_80088640_fp as ft_80088640;
pub use crate::tu::ft_0881::ft_80088770_fp as ft_80088770;
pub use crate::tu::ft_0881::ft_800887cc;
pub use crate::tu::ft_0881::ft_80088828_fp as ft_80088828;
pub use crate::tu::ft_0881::ft_80088884_fp as ft_80088884;
pub use crate::tu::ft_0881::ft_800888e0;
pub use crate::tu::ft_0881::ft_8008893c;
pub use crate::tu::ft_0881::ft_80088998;
pub use crate::tu::ft_0881::ft_800889f4;
pub use crate::tu::ft_0881::ft_80088a50;
pub use crate::tu::ft_0881::ft_80088c5c;
pub use crate::tu::ft_0881::ftco_800886d8;
