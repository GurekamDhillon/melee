//! The byte-order-neutral gameplay hash, format NHV1 (docs/rust-port/neutral-hash.md).
//!
//! Port of `nh_Mix`, `nh_CpuFlags` and `nh_Fighter` (src/melee/ft/fighter.c, `TARGET_PC`). The
//! word order is the format; it must match the C build's `MELEE_NEUTRAL_HASHLOG` goldens exactly.
//! The frame-level word (seed, players, items) needs the player table, the RNG and the item list;
//! it is [`game_word`], fed by the caller.

use crate::types::{CpuFighter, Fighter};
use melee_core::hash::{StateHash, StateHasher};

/// "NHV1": format version, the first word mixed into the game hash.
pub const NHV1: u32 = 0x4E48_5631;
pub const FNV_START: u32 = 0x811C_9DC5;

/// `nh_Mix` / `ftRb_Mix`.
#[inline]
pub const fn mix(mut h: u32, v: u32) -> u32 {
    h ^= v;
    h = h.wrapping_mul(0x0100_0193);
    h ^= h >> 15;
    h
}

/// `ftRb_Bits`: an f32's IEEE bit pattern.
#[inline]
pub fn bits(f: f32) -> u32 {
    f.to_bits()
}

/// `nh_CpuFlags`: the CPU flag bitfields at CpuFighter+0xF8, packed first-declared-highest.
pub fn cpu_flags(c: &CpuFighter) -> u32 {
    let mut w = 0u32;
    let mut put = |v: u8, width: u32| w = (w << width) | (v as u32 & ((1u32 << width) - 1));
    put(c.xF8_b0(), 1);
    put(c.xF8_b12(), 2);
    put(c.xF8_b34(), 2);
    put(c.xF8_b5(), 1);
    put(c.xF8_b6(), 1);
    put(c.xF8_b7(), 1);
    put(c.xF9_b0(), 1);
    put(c.xF9_b1(), 1);
    put(c.xF9_b2(), 1);
    put(c.xF9_b3(), 1);
    put(c.xF9_b4(), 1);
    put(c.xF9_b5(), 1);
    put(c.xF9_b6(), 1);
    put(c.xF9_b7(), 1);
    put(c.xFA_b0(), 1);
    put(c.xFA_b1(), 1);
    put(c.xFA_b2(), 1);
    put(c.xFA_b34(), 2);
    put(c.xFA_b5(), 1);
    put(c.xFA_b6(), 1);
    put(c.xFA_b7(), 1);
    put(c.xFB_b0(), 1);
    put(c.xFB_b1(), 1);
    put(c.xFB_b2(), 1);
    put(c.xFB_b3(), 1);
    put(c.xFB_b4(), 1);
    put(c.xFB_b5(), 1);
    put(c.xFB_b6(), 1);
    put(c.xFB_b7(), 1);
    w
}

/// Script read offset: `csP - buffer` when csP points into (or one past) `buffer`, else 0.
pub fn cpu_script_offset(c: &CpuFighter) -> u32 {
    let base = c.buffer.as_ptr() as usize;
    let p = c.csP as usize;
    if p >= base && p <= base + c.buffer.len() {
        (p - base) as u32
    } else {
        0
    }
}

/// The NHV1 input words of one fighter, in mixing order. `stocks` is
/// `Player_GetStocks(fp->player_id)`; `cpu` is `ftCo_IsCpuControlled(fp)`.
pub fn fighter_words(fp: &Fighter, slot: u32, stocks: i32, cpu: bool, out: &mut Vec<u32>) {
    out.push(slot.wrapping_add(0x100));
    out.push(fp.motion_id as u32);
    out.push(bits(fp.cur_pos.x));
    out.push(bits(fp.cur_pos.y));
    out.push(bits(fp.cur_pos.z));
    out.push(bits(fp.self_vel.x));
    out.push(bits(fp.self_vel.y));
    out.push(bits(fp.dmg.x1830_percent));
    out.push(bits(fp.facing_dir));
    out.push(fp.ground_or_air as u32);
    out.push(bits(fp.x8c_kb_vel.x));
    out.push(bits(fp.x8c_kb_vel.y));
    out.push(bits(fp.gr_vel));
    out.push(bits(fp.dmg.x195c_hitlag_frames));
    out.push(fp.x2219_b5() as u32);
    out.push(bits(fp.shield_health));
    out.push(bits(fp.x3E4_fighterCmdScript.frame_count));
    out.push(fp.x1968_jumpsUsed as u32);
    out.push(fp.x67F as u32);
    out.push(fp.x680 as u32);
    out.push(stocks as u32);
    if cpu {
        let c = &fp.cpu;
        out.push(0x4350_5541); // "CPUA"
        out.push(c.buttons);
        out.push(
            ((c.lstick.x as u8 as u32) << 24)
                | ((c.lstick.y as u8 as u32) << 16)
                | ((c.cstick.x as u8 as u32) << 8)
                | (c.cstick.y as u8 as u32),
        );
        out.push(((c.ltrigger as u32) << 8) | c.rtrigger as u32);
        out.push(c.kind as u32);
        out.push(c.level as u32);
        for v in [c.x14, c.x18, c.x1C, c.x20, c.x24, c.x28, c.x2C, c.x30, c.x34] {
            out.push(v as u32);
        }
        for v in [c.x38, c.x3C, c.x40] {
            out.push(bits(v));
        }
        for v in [c.x7C, c.x80, c.x84] {
            out.push(v as u32);
        }
        out.push(c.xC8 as u32);
        out.push(c.xEC as u32);
        out.push(c.command_duration);
        out.push(cpu_script_offset(c));
        out.push(cpu_flags(c));
    }
}

/// `nh_Fighter`: one fighter's word (`s0`..`s11` columns).
pub fn fighter_word(fp: &Fighter, slot: u32, stocks: i32, cpu: bool) -> u32 {
    let mut w = Vec::with_capacity(48);
    fighter_words(fp, slot, stocks, cpu, &mut w);
    w.into_iter().fold(FNV_START, mix)
}

/// One item's contribution (`ih`), summed (wrapping) into `itemsum`.
pub fn item_word(kind: i32, msid: i32, pos: [f32; 3], vel: [f32; 3]) -> u32 {
    let mut ih = mix(0x4954_454D, kind as u32);
    ih = mix(ih, msid as u32);
    for v in pos.into_iter().chain(vel) {
        ih = mix(ih, bits(v));
    }
    ih
}

/// `RB_NeutralHash`'s game word: `fighters` are (slot, fighter word) for present slots in
/// player-major order, `items` the item words.
pub fn game_word(seed: u32, fighters: &[(u32, u32)], items: &[u32]) -> u32 {
    let mut h = mix(FNV_START, NHV1);
    h = mix(h, seed);
    for &(_, fw) in fighters {
        h = mix(h, fw);
    }
    h = mix(h, 0x4954_434E); // "ITCN"
    h = mix(h, items.len() as u32);
    h = mix(h, items.iter().fold(0u32, |a, &w| a.wrapping_add(w)));
    h
}

/// The fighter's neutral-hash field set for [`StateHash`] (big-endian u32 words, NHV1 order;
/// stocks and CPU control come from the player table, which is not part of `Fighter`, so this
/// form hashes them as absent: use [`fighter_words`] when they are known).
impl StateHash for Fighter {
    fn state_hash(&self, h: &mut StateHasher) {
        let mut w = Vec::with_capacity(21);
        fighter_words(self, 0, 0, false, &mut w);
        for v in &w[1..w.len() - 1] {
            v.state_hash(h);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mix_known_values() {
        // h = 0x811C9DC5 ^ 0 -> * 0x01000193 -> ^ >> 15 (worked by hand from the C definition)
        let h = 0x811C_9DC5u32.wrapping_mul(0x0100_0193);
        assert_eq!(mix(FNV_START, 0), h ^ (h >> 15));
    }
}
