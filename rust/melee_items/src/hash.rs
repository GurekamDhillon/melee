//! Items' part of the byte-order-neutral gameplay hash (docs/rust-port/neutral-hash.md, NHV1).
//!
//! The per-item word is `melee_types::neutral_hash::nh_item` (the one NHV1 implementation); this
//! module keeps group 5's frame-level helpers (count + wrapping sum, "ITCN" combine) on top of it.

use melee_types::Item;
pub use melee_types::neutral_hash::nh_mix as mix;

/// "ITEM": per-item seed.
pub const ITEM_TAG: u32 = 0x4954_454D;
/// "ITCN": mixed into the game word before the count and sum.
pub const ITEM_COUNT_TAG: u32 = 0x4954_434E;

/// The neutral-hash word of one item.
pub fn item_word(ip: &Item) -> u32 {
    melee_types::neutral_hash::nh_item(ip)
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
