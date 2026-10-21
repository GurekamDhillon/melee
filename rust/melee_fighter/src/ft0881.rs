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

const STOP: i32 = 0x83D61;
const NOP: i32 = 0x83D60;

#[inline(always)]
fn chan(f: &Fighter, base: i32) -> i32 {
    base + f.player_id as i32 * 2 + f.is_sub_fighter() as i32
}
#[inline(always)]
fn geno_voice(f: &mut Fighter, id: i32) -> bool {
    f.kind >= Ft_Kind_Mex0 && h::geno_define_voice_play_owner(f.kind as i32, id, f as *mut Fighter) != 0
}
#[inline(always)]
fn stop(f: &mut Fighter, base: i32) {
    h::lbaudioax_80023870(STOP, 0, 0x40, chan(f, base));
}

/// Shared body of 800881D8 / 80088328: `stop_first` is what 0x83D61 calls.
fn voice(f: &mut Fighter, sfx_id: i32, vol: u8, pan: u8, second: bool) {
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

pub fn ft_800881d8(f: &mut Fighter, sfx_id: i32, vol: u8, pan: u8) {
    if geno_voice(f, sfx_id) {
        return;
    }
    voice(f, sfx_id, vol, pan, false)
}
pub fn ft_80088328(f: &mut Fighter, sfx_id: i32, vol: u8, pan: u8) {
    if geno_voice(f, sfx_id) {
        return;
    }
    voice(f, sfx_id, vol, pan, true)
}

/// Shared body of 80088478 .. ftCo_800886D8: play on one channel, or stop it.
fn simple(f: &mut Fighter, sfx_id: i32, vol: u8, pan: u8, base: i32, slot: fn(&mut Fighter) -> &mut i32) {
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

pub fn ft_80088478(f: &mut Fighter, s: i32, v: u8, p: u8) {
    if !geno_voice(f, s) { simple(f, s, v, p, 0x36, |f| &mut f.x214C) }
}
pub fn ft_80088510(f: &mut Fighter, s: i32, v: u8, p: u8) {
    if !geno_voice(f, s) { simple(f, s, v, p, 0x42, |f| &mut f.x2150) }
}
pub fn ft_800885a8(f: &mut Fighter, s: i32, v: u8, p: u8) {
    if !geno_voice(f, s) { simple(f, s, v, p, 0x4E, |f| &mut f.x2154) }
}
pub fn ft_80088640(f: &mut Fighter, s: i32, v: u8, p: u8) {
    if !geno_voice(f, s) { simple(f, s, v, p, 0x5A, |f| &mut f.x2158) }
}
/// No Geno voice check in the C.
pub fn ftco_800886d8(f: &mut Fighter, s: i32, v: u8, p: u8) {
    simple(f, s, v, p, 0x72, |f| &mut f.x215C)
}

pub fn ft_80088770(f: &mut Fighter) { stop(f, 0x1E); f.x2144 = -1; }
pub fn ft_800887cc(f: &mut Fighter) { stop(f, 0x2A); f.x2148 = -1; }
pub fn ft_80088828(f: &mut Fighter) { stop(f, 0x36); f.x214C = -1; }
pub fn ft_80088884(f: &mut Fighter) { stop(f, 0x42); f.x2150 = -1; }
pub fn ft_800888e0(f: &mut Fighter) { stop(f, 0x4E); f.x2154 = -1; }
pub fn ft_8008893c(f: &mut Fighter) { stop(f, 0x5A); f.x2158 = -1; }
pub fn ft_80088998(f: &mut Fighter) { stop(f, 0x72); f.x215C = -1; }

/// Random voice line from `sfx` (may be null).
pub fn ft_800889f4(f: &mut Fighter, sfx: *const FtSFXArr) {
    if sfx.is_null() {
        return;
    }
    let a = unsafe { &*sfx };
    let i = h::hsd_randi(a.num);
    let id = unsafe { *a.sfx_ids.offset(i as isize) };
    ft_800881d8(f, id, 0x7F, 0x40);
}

/// Stops both (main + sub) channels of every bank, then the lbAudioAx counters.
pub fn ft_80088a50(f: &mut Fighter) {
    let p = f.player_id as i32 * 2;
    for (base, slot) in [(0x36, 2usize), (0x1E, 0), (0x2A, 1), (0x42, 3), (0x4E, 4), (0x5A, 5), (0x72, 6)] {
        h::lbaudioax_80023870(STOP, 0, 0x40, p + base);
        h::lbaudioax_80023870(STOP, 0, 0x40, p + base + 1);
        *chan_slot(f, slot) = -1;
    }
    f.x2160 = -1;
    h::lbaudioax_8002500c(f.x2164);
    f.x2164 = 0;
    h::lbaudioax_80025038(f.x2168);
    f.x2168 = 0;
}

fn chan_slot(f: &mut Fighter, i: usize) -> &mut i32 {
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

pub fn ft_80088c5c(g: G) {
    let f = unsafe { &mut *((*g).user_data as *mut Fighter) };
    h::lbaudioax_80026510(g);
    if f.x2160 != -1 {
        h::lbaudioax_800265c4(g, f.x2160);
        f.x2160 = -1;
    }
    let order: [(usize, fn(&mut Fighter)); 7] = [
        (2, ft_80088828), (0, ft_80088770), (1, ft_800887cc), (3, ft_80088884),
        (4, ft_800888e0), (5, ft_8008893c), (6, ft_80088998),
    ];
    for (slot, stopf) in order {
        let v = *chan_slot(f, slot);
        if v != -1 && h::lbaudioax_800265c4(g, v) {
            stopf(f);
        }
    }
    ft_80088a50(f);
}
