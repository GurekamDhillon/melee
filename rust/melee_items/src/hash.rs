//! Items' part of the byte-order-neutral gameplay hash (docs/rust-port/neutral-hash.md, NHV1).
//!
//! Port of the item loop of `RB_NeutralHash` (src/melee/ft/fighter.c): per item
//! `ih = mix(0x4954454D, kind)`, then msid, pos.xyz, x40_vel.xyz (f32 bit patterns); the
//! frame's `items` column is the count and `itemsum` the wrapping sum, so list order does not
//! matter. `mix` is `nh_Mix` / `ftRb_Mix`.
//! TODO(move-to-core): `mix` is shared with the fighter word (group 3); it belongs in melee_core.

use crate::layout::Item;
use melee_core::hash::{StateHash, StateHasher};

/// `nh_Mix`: `h ^= v; h *= 0x01000193; h ^= h >> 15` (u32, wrapping).
#[inline]
pub const fn mix(h: u32, v: u32) -> u32 {
    let h = (h ^ v).wrapping_mul(0x0100_0193);
    h ^ (h >> 15)
}

/// "ITEM": per-item seed.
pub const ITEM_TAG: u32 = 0x4954_454D;
/// "ITCN": mixed into the game word before the count and sum.
pub const ITEM_COUNT_TAG: u32 = 0x4954_434E;

/// The neutral-hash word of one item.
pub fn item_word(ip: &Item) -> u32 {
    let mut ih = mix(ITEM_TAG, ip.kind as u32);
    ih = mix(ih, ip.msid as u32);
    ih = mix(ih, ip.pos.x.to_bits());
    ih = mix(ih, ip.pos.y.to_bits());
    ih = mix(ih, ip.pos.z.to_bits());
    ih = mix(ih, ip.x40_vel.x.to_bits());
    ih = mix(ih, ip.x40_vel.y.to_bits());
    ih = mix(ih, ip.x40_vel.z.to_bits());
    ih
}

/// The `items` / `itemsum` columns for one frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ItemSum {
    pub count: u32,
    pub sum: u32,
}

impl ItemSum {
    pub fn add(&mut self, ip: &Item) {
        self.sum = self.sum.wrapping_add(item_word(ip));
        self.count = self.count.wrapping_add(1);
    }
    /// Sum over every live item (the caller walks HSD_GOBJ_PLINK_ITEM and skips gobjs whose
    /// user_data is null).
    pub fn of<'a>(items: impl IntoIterator<Item = &'a Item>) -> Self {
        let mut s = Self::default();
        for ip in items {
            s.add(ip);
        }
        s
    }
    /// Continue the game word: `mix(h, "ITCN")`, count, sum.
    pub fn mix_into(&self, h: u32) -> u32 {
        mix(mix(mix(h, ITEM_COUNT_TAG), self.count), self.sum)
    }
}

/// The neutral-hash field set of an item, in NHV1 order, as canonical bytes (melee_core's
/// FNV stream; the NHV1 word itself is [`item_word`]).
impl StateHash for Item {
    fn state_hash(&self, h: &mut StateHasher) {
        (self.kind as u32).state_hash(h);
        (self.msid as u32).state_hash(h);
        [self.pos.x, self.pos.y, self.pos.z].state_hash(h);
        [self.x40_vel.x, self.x40_vel.y, self.x40_vel.z].state_hash(h);
    }
}
