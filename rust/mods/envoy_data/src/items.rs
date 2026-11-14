//! Envoy items (docs/rust-port/notes/envoy-items-design.md): the hand-made item table that replaces the drive rewards.
//! One record per item: one trigger tied to a Melee action, one effect function in envoy_sim (`item_<name>`), one
//! visual, one stack rule with its cap. A const table (the slice's 12); `lint` runs at compile time (a broken record
//! fails the build of every Envoy mod) and `ITEMS_DIGEST` goes into the sim hash and the run record.

/// Pools an item can come from (bits).
pub const PL_ITEM: u8 = 1;
pub const PL_SHOP: u8 = 2;
pub const PL_BOSS: u8 = 4;

/// Triggers (design section 3).
pub const TR_PASSIVE: u8 = 0;
pub const TR_NAIR_HIT: u8 = 1;
pub const TR_WAVEDASH: u8 = 2;
pub const TR_SHINE: u8 = 3;
pub const TR_SHORT_HOP: u8 = 4;
pub const TR_LEDGE_GRAB: u8 = 5;
pub const TR_WHIFF: u8 = 6;
pub const TR_ROLL: u8 = 7;
pub const TR_SMASH_HIT: u8 = 8;
pub const TR_KO: u8 = 9;
pub const TR_MODIFIER: u8 = 10;
pub const TRIGGER_TEXT: [&str; 11] = ["always on", "nair hit", "wavedash", "shine", "short hop", "ledge grab", "whiff", "roll", "smash hit", "KO", "modifies"];

/// Stack rules: what a second copy does.
pub const ST_MORE: u8 = 1;
pub const ST_BIGGER: u8 = 2;
pub const ST_STRONGER: u8 = 3;
pub const ST_FASTER: u8 = 4;
pub const STACK_TEXT: [&str; 5] = ["", "more", "bigger", "stronger", "faster"];

/// Tags: what an item makes (modifiers read them).
pub const TG_PROJECTILE: u8 = 1;
pub const TG_BURST: u8 = 2;
pub const TG_TRAIL: u8 = 4;
pub const TG_ORBITAL: u8 = 8;
pub const TG_SPARK: u8 = 16;
pub const TG_LAUNCH: u8 = 32;

/// Item ids (stable; saves and replays store them; never reused).
pub const IT_SPARK_RING: i32 = 1;
pub const IT_GLIDE_WAKE: i32 = 2;
pub const IT_SHINE_BELL: i32 = 3;
pub const IT_HOP_SHOT: i32 = 4;
pub const IT_LEDGE_SPIKES: i32 = 5;
pub const IT_WHIFF_ECHO: i32 = 6;
pub const IT_ROLL_MINES: i32 = 7;
pub const IT_SMASH_QUAKE: i32 = 12;
pub const IT_MOON_DROPS: i32 = 15;
pub const IT_COIN_MAGNET: i32 = 18;
pub const IT_HOMING_INK: i32 = 35;
pub const IT_FIRE_LACE: i32 = 36;

#[derive(Clone, Copy)]
pub struct Item {
    pub id: i32,
    pub key: &'static str,
    pub name: &'static str,
    /// One line: what you see and feel.
    pub line: &'static str,
    pub pools: u8,
    /// 0-4 (Isaac's quality): the shop price.
    pub quality: u8,
    pub trigger: u8,
    /// The signature look (presentation key; the sim never reads it).
    pub visual: &'static str,
    pub tags: u8,
    /// Modifiers: the tags they change (0 for a trigger item).
    pub mods: u8,
    pub stack: u8,
    /// The stack cap: copies past this add nothing more.
    pub cap: u8,
    /// Can come again from a pool after it is taken (copies).
    pub repeatable: bool,
    /// RGBA: the item's colour (icon, effects).
    pub rgba: u32,
}

pub const ITEMS: [Item; 12] = [
    Item { id: IT_SPARK_RING, key: "spark_ring", name: "Spark Ring", line: "Nair hits pop a ring of sparks", pools: PL_ITEM, quality: 1, trigger: TR_NAIR_HIT, visual: "sparks", tags: TG_SPARK | TG_PROJECTILE, mods: 0, stack: ST_MORE, cap: 3, repeatable: true, rgba: 0xFFF0_70FF },
    Item { id: IT_GLIDE_WAKE, key: "glide_wake", name: "Glide Wake", line: "Wavedashes leave a burning streak", pools: PL_ITEM, quality: 1, trigger: TR_WAVEDASH, visual: "wake", tags: TG_TRAIL, mods: 0, stack: ST_BIGGER, cap: 3, repeatable: false, rgba: 0xFF8A_2AFF },
    Item { id: IT_SHINE_BELL, key: "shine_bell", name: "Shine Bell", line: "Each shine rings a shock ring", pools: PL_ITEM, quality: 2, trigger: TR_SHINE, visual: "bell", tags: TG_BURST | TG_LAUNCH, mods: 0, stack: ST_BIGGER, cap: 3, repeatable: false, rgba: 0x7AD8_FFFF },
    Item { id: IT_HOP_SHOT, key: "hop_shot", name: "Hop Shot", line: "Short hops spit a star at a foe", pools: PL_ITEM, quality: 1, trigger: TR_SHORT_HOP, visual: "star", tags: TG_PROJECTILE, mods: 0, stack: ST_MORE, cap: 3, repeatable: true, rgba: 0xFFD8_4AFF },
    Item { id: IT_LEDGE_SPIKES, key: "ledge_spikes", name: "Ledge Spikes", line: "Ledge grabs stab ice spikes up", pools: PL_ITEM, quality: 1, trigger: TR_LEDGE_GRAB, visual: "ice_spikes", tags: TG_BURST | TG_LAUNCH, mods: 0, stack: ST_STRONGER, cap: 3, repeatable: false, rgba: 0x9CF0_FFFF },
    Item { id: IT_WHIFF_ECHO, key: "whiff_echo", name: "Whiff Echo", line: "A whiff leaves a ghost that swings again", pools: PL_ITEM, quality: 2, trigger: TR_WHIFF, visual: "ghost", tags: TG_BURST, mods: 0, stack: ST_MORE, cap: 3, repeatable: false, rgba: 0x6AF0_E0FF },
    Item { id: IT_ROLL_MINES, key: "roll_mines", name: "Roll Mines", line: "Rolls drop a blinking mine", pools: PL_ITEM, quality: 1, trigger: TR_ROLL, visual: "mine", tags: TG_BURST | TG_LAUNCH, mods: 0, stack: ST_MORE, cap: 3, repeatable: true, rgba: 0xFF4A_4AFF },
    Item { id: IT_SMASH_QUAKE, key: "smash_quake", name: "Smash Quake", line: "Smashes send shockwaves along the floor", pools: PL_ITEM, quality: 2, trigger: TR_SMASH_HIT, visual: "quake", tags: TG_BURST | TG_LAUNCH, mods: 0, stack: ST_BIGGER, cap: 3, repeatable: false, rgba: 0xE0B0_60FF },
    Item { id: IT_MOON_DROPS, key: "moon_drops", name: "Moon Drops", line: "Two moons orbit you and nick foes", pools: PL_SHOP, quality: 2, trigger: TR_PASSIVE, visual: "moons", tags: TG_ORBITAL, mods: 0, stack: ST_MORE, cap: 4, repeatable: true, rgba: 0xE8EC_FFFF },
    Item { id: IT_COIN_MAGNET, key: "coin_magnet", name: "Coin Magnet", line: "KO'd foes burst into coins for you", pools: PL_SHOP, quality: 1, trigger: TR_KO, visual: "coins", tags: 0, mods: 0, stack: ST_STRONGER, cap: 3, repeatable: true, rgba: 0xFFC8_30FF },
    Item { id: IT_HOMING_INK, key: "homing_ink", name: "Homing Ink", line: "Your projectiles curve to foes", pools: PL_ITEM | PL_SHOP, quality: 2, trigger: TR_MODIFIER, visual: "ink", tags: 0, mods: TG_PROJECTILE, stack: ST_STRONGER, cap: 3, repeatable: false, rgba: 0xC85A_FFFF },
    Item { id: IT_FIRE_LACE, key: "fire_lace", name: "Fire Lace", line: "Everything you make leaves embers and burns", pools: PL_ITEM, quality: 3, trigger: TR_MODIFIER, visual: "embers", tags: 0, mods: TG_BURST | TG_TRAIL | TG_PROJECTILE | TG_SPARK, stack: ST_STRONGER, cap: 3, repeatable: false, rgba: 0xFF6A_20FF },
];
pub const ITEMS_LEN: usize = ITEMS.len();
/// Highest item id + 1 (the inventory arrays are indexed by id).
pub const ITEM_IDS: usize = 40;

/// The table's lint (design section 9): every item has a trigger or is a modifier with tags to change, a visual, a
/// stack rule and a cap; ids are unique, in range and never 0; a modifier makes nothing; quality 0-4.
const fn lint() -> bool {
    let mut i = 0;
    while i < ITEMS.len() {
        let it = &ITEMS[i];
        assert!(it.id > 0 && (it.id as usize) < ITEM_IDS, "item id out of range");
        assert!(it.stack >= ST_MORE && it.stack <= ST_FASTER, "item has no stack rule");
        assert!(it.cap >= 1, "item has no cap");
        assert!(!it.visual.is_empty(), "item has no visual");
        assert!(it.quality <= 4, "item quality past 4");
        assert!(it.pools != 0, "item is in no pool");
        assert!(it.trigger <= TR_MODIFIER, "item trigger unknown");
        assert!((it.trigger == TR_MODIFIER) == (it.mods != 0), "a modifier must change tags; only a modifier may");
        let mut j = i + 1;
        while j < ITEMS.len() {
            assert!(ITEMS[j].id != it.id, "duplicate item id");
            j += 1;
        }
        i += 1;
    }
    true
}
const _: bool = lint();

const fn fnv_bytes(mut h: u32, b: &[u8]) -> u32 {
    let mut i = 0;
    while i < b.len() {
        h ^= b[i] as u32;
        h = h.wrapping_mul(0x0100_0193);
        i += 1;
    }
    h
}

const fn digest() -> u32 {
    let mut h = 0x811C_9DC5u32;
    let mut i = 0;
    while i < ITEMS.len() {
        let it = &ITEMS[i];
        h = fnv_bytes(h, &it.id.to_le_bytes());
        h = fnv_bytes(h, it.key.as_bytes());
        h = fnv_bytes(h, &[it.pools, it.quality, it.trigger, it.tags, it.mods, it.stack, it.cap, it.repeatable as u8]);
        i += 1;
    }
    h
}
/// FNV-1a over every sim-relevant field: the sim hash and the run record (a save from another table is dropped).
pub const ITEMS_DIGEST: u32 = digest();

/// The record of item `id` (None: no such item).
pub fn item(id: i32) -> Option<&'static Item> {
    ITEMS.iter().find(|it| it.id == id)
}

pub fn item_by_key(key: &str) -> Option<&'static Item> {
    ITEMS.iter().find(|it| it.key == key)
}

/// The shop price of an item before the per-loop markup.
pub fn item_price(id: i32) -> i32 {
    const Q: [i32; 5] = [25, 35, 50, 70, 95];
    item(id).map_or(0, |it| Q[it.quality.min(4) as usize])
}

/// A stack's strength for `copies` (x1000): `stronger` x(1 + 0.5 per copy past the first), capped at x3 and by the
/// item's cap; `bigger` the same curve (radius / length); `more` and `faster` return the copies counted (capped).
pub fn stack_mul(id: i32, copies: i32) -> i32 {
    let Some(it) = item(id) else { return 1000 };
    let c = copies.clamp(1, it.cap as i32);
    match it.stack {
        ST_STRONGER | ST_BIGGER => (1000 + 500 * (c - 1)).min(3000),
        _ => c * 1000,
    }
}
