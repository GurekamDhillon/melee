//! The byte-order-neutral gameplay hash (format NHV1, docs/rust-port/neutral-hash.md), computed
//! from the native structs. A port of `nh_Mix` / `nh_CpuFlags` / `nh_Fighter` and the item-loop
//! body of `RB_NeutralHash` (src/melee/ft/fighter.c). Field order is part of the format.
//!
//! The two inputs that live in other groups (`Player_GetStocks`, `ftCo_IsCpuControlled`) are
//! parameters here; the frame driver that owns the player table and the GObj lists supplies them.
//! TODO(move-to-core): the frame-level combine belongs next to the frame driver.

use crate::gen::{CpuFighter, Fighter, Item};
use melee_core::hash::{StateHash, StateHasher};

/// `nh_Mix`: `h ^= v; h *= 0x01000193; h ^= h >> 15` (wrapping u32).
#[inline]
pub const fn nh_mix(mut h: u32, v: u32) -> u32 {
    h ^= v;
    h = h.wrapping_mul(0x0100_0193);
    h ^= h >> 15;
    h
}

#[inline]
fn bits(f: f32) -> u32 {
    f.to_bits()
}

/// `nh_CpuFlags`: the +0xF8..+0xFB bitfields packed in declaration order, first in the MSBs.
pub fn nh_cpu_flags(c: &CpuFighter) -> u32 {
    let mut w: u32 = 0;
    let mut put = |v: u8, width: u32| w = (w << width) | (v as u32 & ((1u32 << width) - 1));
    put(c.xF8_b0(), 1); put(c.xF8_b12(), 2); put(c.xF8_b34(), 2); put(c.xF8_b5(), 1); put(c.xF8_b6(), 1); put(c.xF8_b7(), 1);
    put(c.xF9_b0(), 1); put(c.xF9_b1(), 1); put(c.xF9_b2(), 1); put(c.xF9_b3(), 1);
    put(c.xF9_b4(), 1); put(c.xF9_b5(), 1); put(c.xF9_b6(), 1); put(c.xF9_b7(), 1);
    put(c.xFA_b0(), 1); put(c.xFA_b1(), 1); put(c.xFA_b2(), 1); put(c.xFA_b34(), 2); put(c.xFA_b5(), 1); put(c.xFA_b6(), 1); put(c.xFA_b7(), 1);
    put(c.xFB_b0(), 1); put(c.xFB_b1(), 1); put(c.xFB_b2(), 1); put(c.xFB_b3(), 1);
    put(c.xFB_b4(), 1); put(c.xFB_b5(), 1); put(c.xFB_b6(), 1); put(c.xFB_b7(), 1);
    w
}

/// `csP - buffer` when csP points inside (or one past) `buffer`, else 0.
pub fn cs_offset(c: &CpuFighter) -> u32 {
    let base = c.buffer.as_ptr() as usize;
    let p = c.csP as usize;
    if p >= base && p <= base + c.buffer.len() {
        (p - base) as u32
    } else {
        0
    }
}

/// The CPU half of `nh_Fighter`, continuing `h`.
pub fn nh_cpu(mut h: u32, c: &CpuFighter) -> u32 {
    h = nh_mix(h, 0x4350_5541);
    h = nh_mix(h, c.buttons as u32);
    h = nh_mix(h, ((c.lstick.x as u8 as u32) << 24) | ((c.lstick.y as u8 as u32) << 16)
        | ((c.cstick.x as u8 as u32) << 8) | (c.cstick.y as u8 as u32));
    h = nh_mix(h, ((c.ltrigger as u32) << 8) | c.rtrigger as u32);
    h = nh_mix(h, c.kind as u32);
    h = nh_mix(h, c.level as u32);
    for v in [c.x14, c.x18, c.x1C, c.x20, c.x24, c.x28, c.x2C, c.x30, c.x34] {
        h = nh_mix(h, v as u32);
    }
    for v in [c.x38, c.x3C, c.x40] {
        h = nh_mix(h, bits(v));
    }
    for v in [c.x7C, c.x80, c.x84] {
        h = nh_mix(h, v as u32);
    }
    h = nh_mix(h, c.xC8 as u32);
    h = nh_mix(h, c.xEC as u32);
    h = nh_mix(h, c.command_duration);
    h = nh_mix(h, cs_offset(c));
    nh_mix(h, nh_cpu_flags(c))
}

/// `nh_Fighter(fp, slot)`. `stocks` = `Player_GetStocks(fp->player_id)`,
/// `is_cpu` = `ftCo_IsCpuControlled(fp)`.
pub fn nh_fighter(fp: &Fighter, slot: i32, stocks: i32, is_cpu: bool) -> u32 {
    let mut h: u32 = 0x811C_9DC5;
    h = nh_mix(h, (slot as u32).wrapping_add(0x100));
    h = nh_mix(h, fp.motion_id as u32);
    h = nh_mix(h, bits(fp.cur_pos.x));
    h = nh_mix(h, bits(fp.cur_pos.y));
    h = nh_mix(h, bits(fp.cur_pos.z));
    h = nh_mix(h, bits(fp.self_vel.x));
    h = nh_mix(h, bits(fp.self_vel.y));
    h = nh_mix(h, bits(fp.dmg.x1830_percent));
    h = nh_mix(h, bits(fp.facing_dir));
    h = nh_mix(h, fp.ground_or_air as u32);
    h = nh_mix(h, bits(fp.x8c_kb_vel.x));
    h = nh_mix(h, bits(fp.x8c_kb_vel.y));
    h = nh_mix(h, bits(fp.gr_vel));
    h = nh_mix(h, bits(fp.dmg.x195c_hitlag_frames));
    h = nh_mix(h, fp.x2219_b5() as u32);
    h = nh_mix(h, bits(fp.shield_health));
    h = nh_mix(h, bits(fp.x3E4_fighterCmdScript.frame_count));
    h = nh_mix(h, fp.x1968_jumpsUsed as u32);
    h = nh_mix(h, fp.x67F as u32);
    h = nh_mix(h, fp.x680 as u32);
    h = nh_mix(h, stocks as u32);
    if is_cpu {
        h = nh_cpu(h, &fp.cpu);
    }
    h
}

/// One item's word (the body of RB_NeutralHash's item loop); the frame sums these, wrapping.
pub fn nh_item(ip: &Item) -> u32 {
    let mut ih = nh_mix(0x4954_454D, ip.kind as u32);
    ih = nh_mix(ih, ip.msid as u32);
    for v in [ip.pos.x, ip.pos.y, ip.pos.z, ip.x40_vel.x, ip.x40_vel.y, ip.x40_vel.z] {
        ih = nh_mix(ih, bits(v));
    }
    ih
}

/// The `hash` column: seed, then each present fighter word in slot order (player*2+follower),
/// then "ITCN", item count and item sum (docs/rust-port/neutral-hash.md, "Game word").
pub fn nh_game(seed: u32, fighter_words: &[Option<u32>; 12], item_count: u32, item_sum: u32) -> u32 {
    let mut h = nh_mix(0x811C_9DC5, 0x4E48_5631);
    h = nh_mix(h, seed);
    for w in fighter_words.iter().flatten() {
        h = nh_mix(h, *w);
    }
    h = nh_mix(h, 0x4954_434E);
    h = nh_mix(h, item_count);
    nh_mix(h, item_sum)
}

/// StateHash streams exactly the neutral-hash fields (canonical big-endian words), so a struct in
/// the neutral field list can be folded into any larger state hash.
impl StateHash for CpuFighter {
    fn state_hash(&self, h: &mut StateHasher) {
        nh_cpu(0, self).state_hash(h)
    }
}
/// Excludes stocks and the CPU check (owned by other groups): hashes fields 2..20 + the CPU block.
impl StateHash for Fighter {
    fn state_hash(&self, h: &mut StateHasher) {
        nh_fighter(self, 0, 0, false).state_hash(h);
        self.cpu.state_hash(h);
    }
}
impl StateHash for Item {
    fn state_hash(&self, h: &mut StateHasher) {
        nh_item(self).state_hash(h)
    }
}
