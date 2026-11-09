//! C-named entry points of the staling tail of src/melee/ft/ft_0881.c. The bodies live in
//! `crate::stale` (`clear_attack_id`, `new_attack`, `apply_staling`, `renew_attack_instance`);
//! tests/tu_stale_vs_c.rs checks these names against the C.
#![allow(non_snake_case)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::ft0877::ft_80087d0c;
#[allow(unused_imports)]
use crate::hooks as h;
#[allow(unused_imports)]
use crate::ft0881::{G, STOP, chan_slot, geno_voice, simple, stop, voice};
use crate::types::*;

/// `ft_800890BC`: clear the attack id / instance.
pub fn ft_800890BC(fp: &mut Fighter) { melee_core::ct!("ft_800890BC"); crate::stale::clear_attack_id(fp) }
/// `ft_800890D0`: a new attack instance unless the same (non-1) move is still running.
pub fn ft_800890D0(fp: &mut Fighter, attack_id: i32) { melee_core::ct!("ft_800890D0"); crate::stale::new_attack(fp, attack_id as u32) }
/// `ft_80089228`: the staled damage of `attack_id`.
pub fn ft_80089228(fp: &Fighter, attack_id: i32, arg2: i32, damage: f32) -> f32 { melee_core::ct!("ft_80089228"); crate::stale::apply_staling(fp, attack_id, arg2, damage) }
/// `ft_800892A0`: restart the current attack's instance.
pub fn ft_800892A0(gobj: *mut HSD_GObj) { melee_core::ct!("ft_800892A0"); unsafe { crate::stale::renew_attack_instance(&mut *((*gobj).user_data as *mut Fighter)) } }

// ---- folded from ft0881.rs (consolidation steps 21/22) ----
pub fn ft_800881d8(f: &mut Fighter, sfx_id: i32, vol: u8, pan: u8) {
    melee_core::ct!("ft_800881d8");
    if geno_voice(f, sfx_id) {
        return;
    }
    voice(f, sfx_id, vol, pan, false)
}

pub fn ft_80088328_fp(f: &mut Fighter, sfx_id: i32, vol: u8, pan: u8) {
    melee_core::ct!("ft_80088328_fp");
    if geno_voice(f, sfx_id) {
        return;
    }
    voice(f, sfx_id, vol, pan, true)
}

pub fn ft_80088478_fp(f: &mut Fighter, s: i32, v: u8, p: u8) {
    melee_core::ct!("ft_80088478_fp");
    if !geno_voice(f, s) { simple(f, s, v, p, 0x36, |f| &mut f.x214C) }
}

pub fn ft_80088510_fp(f: &mut Fighter, s: i32, v: u8, p: u8) {
    melee_core::ct!("ft_80088510_fp");
    if !geno_voice(f, s) { simple(f, s, v, p, 0x42, |f| &mut f.x2150) }
}

pub fn ft_800885a8(f: &mut Fighter, s: i32, v: u8, p: u8) {
    melee_core::ct!("ft_800885a8");
    if !geno_voice(f, s) { simple(f, s, v, p, 0x4E, |f| &mut f.x2154) }
}

pub fn ft_80088640_fp(f: &mut Fighter, s: i32, v: u8, p: u8) {
    melee_core::ct!("ft_80088640_fp");
    if !geno_voice(f, s) { simple(f, s, v, p, 0x5A, |f| &mut f.x2158) }
}

/// No Geno voice check in the C.
pub fn ftco_800886d8(f: &mut Fighter, s: i32, v: u8, p: u8) {
    melee_core::ct!("ftco_800886d8");
    simple(f, s, v, p, 0x72, |f| &mut f.x215C)
}

pub fn ft_80088770_fp(f: &mut Fighter) { melee_core::ct!("ft_80088770_fp"); stop(f, 0x1E); f.x2144 = -1; }

pub fn ft_800887cc(f: &mut Fighter) { melee_core::ct!("ft_800887cc"); stop(f, 0x2A); f.x2148 = -1; }

pub fn ft_80088828_fp(f: &mut Fighter) { melee_core::ct!("ft_80088828_fp"); stop(f, 0x36); f.x214C = -1; }

pub fn ft_80088884_fp(f: &mut Fighter) { melee_core::ct!("ft_80088884_fp"); stop(f, 0x42); f.x2150 = -1; }

pub fn ft_800888e0(f: &mut Fighter) { melee_core::ct!("ft_800888e0"); stop(f, 0x4E); f.x2154 = -1; }

pub fn ft_8008893c(f: &mut Fighter) { melee_core::ct!("ft_8008893c"); stop(f, 0x5A); f.x2158 = -1; }

pub fn ft_80088998(f: &mut Fighter) { melee_core::ct!("ft_80088998"); stop(f, 0x72); f.x215C = -1; }

/// Random voice line from `sfx` (may be null).
pub fn ft_800889f4(f: &mut Fighter, sfx: *const FtSFXArr) {
    melee_core::ct!("ft_800889f4");
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
    melee_core::ct!("ft_80088a50");
    let p = f.player_idx as i32 * 2;
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

pub fn ft_80088c5c(g: G) {
    melee_core::ct!("ft_80088c5c");
    let f = unsafe { &mut *((*g).user_data as *mut Fighter) };
    h::lbaudioax_80026510(g);
    if f.x2160 != -1 {
        h::lbaudioax_800265c4(g, f.x2160);
        f.x2160 = -1;
    }
    let order: [(usize, fn(&mut Fighter)); 7] = [
        (2, ft_80088828_fp), (0, ft_80088770_fp), (1, ft_800887cc), (3, ft_80088884_fp),
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
