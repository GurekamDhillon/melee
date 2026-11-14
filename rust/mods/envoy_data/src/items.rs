//! Envoy items (docs/rust-port/notes/envoy-items-design.md): the hand-made item table that replaces the drive rewards.
//! One record per item: one trigger tied to a Melee action, one effect function in envoy_sim (`item_<name>`), one
//! visual, one stack rule with its cap. A const table (slice 1's 12, slice 2's 16); `lint` runs at compile time (a broken record
//! fails the build of every Envoy mod) and `ITEMS_DIGEST` goes into the sim hash and the run record.

/// Pools an item can come from (bits).
pub const PL_ITEM: u8 = 1;
pub const PL_SHOP: u8 = 2;
pub const PL_BOSS: u8 = 4;
/// Slice 2 rooms: the devil deal (costs stock slots), the angel room (free, strong), the secret room.
pub const PL_DEVIL: u8 = 8;
pub const PL_ANGEL: u8 = 16;
pub const PL_SECRET: u8 = 32;

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
/// Slice 2 triggers: a jab that hits, taking a hit, a Fox / Falco laser that hits, a crouch cancel, a powershield, and
/// the pickup itself (Spare Heart).
pub const TR_JAB_HIT: u8 = 11;
pub const TR_TAKE_HIT: u8 = 12;
pub const TR_LASER: u8 = 13;
pub const TR_CC: u8 = 14;
pub const TR_POWERSHIELD: u8 = 15;
pub const TR_PICKUP: u8 = 16;
/// Stance orbs (slice 2): an orb that exists only WHILE the fighter is in this state.
pub const TR_WHILE_AIRDODGE: u8 = 17;
pub const TR_WHILE_NAIR: u8 = 18;
pub const TR_WHILE_DASH: u8 = 19;
pub const TR_WHILE_SHIELD: u8 = 20;
/// Synergy lane: an item that fires off another item's hit (a proc; envoy-synergy.md), and a co-op item (2P only).
pub const TR_ITEM_HIT: u8 = 21;
pub const TR_COOP: u8 = 22;
pub const TR_LAST: u8 = TR_COOP;
pub const TRIGGER_TEXT: [&str; 23] = [
    "always on", "nair hit", "wavedash", "shine", "short hop", "ledge grab", "whiff", "roll", "smash hit", "KO", "modifies", "jab hit", "taking a hit",
    "laser hit", "crouch cancel", "powershield", "on pickup", "while airdodging", "while in nair", "while dashing", "while shielding",
    "item hit", "co-op",
];

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
// slice 2 (envoy-items-design.md 7 and 11)
pub const IT_LASER_PRISM: i32 = 8;
pub const IT_CROUCH_THORNS: i32 = 21;
pub const IT_PERFECT_GUARD: i32 = 26;
pub const IT_BRIMSTONE_JAB: i32 = 27;
pub const IT_DEMON_WINGS: i32 = 28;
pub const IT_CRIMSON_COUNT: i32 = 29;
pub const IT_BLOODLUST: i32 = 30;
pub const IT_HALO: i32 = 31;
pub const IT_GUARDIAN_FEATHER: i32 = 32;
pub const IT_ICE_GLASS: i32 = 37;
pub const IT_CHAIN_SPARK: i32 = 38;
pub const IT_TWIN_SHOT: i32 = 39;
pub const IT_GRAVITY_WELL: i32 = 40;
pub const IT_BIG_BANG: i32 = 41;
pub const IT_BOUNCY: i32 = 42;
pub const IT_SPARE_HEART: i32 = 43;
// stance orbs (orbitals that live only in one movement state)
pub const IT_DODGE_ORB: i32 = 44;
pub const IT_NAIR_RING: i32 = 45;
pub const IT_DASH_ORB: i32 = 46;
pub const IT_SHIELD_ORBS: i32 = 47;
// synergy lane (envoy-synergy.md): proc items (ON <KIND> HIT) and co-op items (2P only). Ids 48-55 are stored two bits
// a copy in the record (cap 3, never repeatable).
pub const IT_EMBER_SEED: i32 = 48;
pub const IT_STATIC_COIL: i32 = 49;
pub const IT_FROST_BUD: i32 = 50;
pub const IT_KIN_TETHER: i32 = 51;
pub const IT_RALLY_BANNER: i32 = 52;
pub const IT_KINDRED_SPARK: i32 = 53;

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
    /// A devil deal's price in stock slots (1-2; 0 for an item outside the devil pool).
    pub slots: u8,
}

pub const ITEMS: [Item; 38] = [
    Item { id: IT_SPARK_RING, key: "spark_ring", name: "Spark Ring", line: "Nair hits pop a ring of sparks", pools: PL_ITEM, quality: 1, trigger: TR_NAIR_HIT, visual: "sparks", tags: TG_SPARK | TG_PROJECTILE, mods: 0, stack: ST_MORE, cap: 3, repeatable: true, rgba: 0xFFF0_70FF, slots: 0 },
    Item { id: IT_GLIDE_WAKE, key: "glide_wake", name: "Glide Wake", line: "Wavedashes leave a burning streak", pools: PL_ITEM, quality: 1, trigger: TR_WAVEDASH, visual: "wake", tags: TG_TRAIL, mods: 0, stack: ST_BIGGER, cap: 3, repeatable: false, rgba: 0xFF8A_2AFF, slots: 0 },
    Item { id: IT_SHINE_BELL, key: "shine_bell", name: "Shine Bell", line: "Each shine rings a shock ring", pools: PL_ITEM, quality: 2, trigger: TR_SHINE, visual: "bell", tags: TG_BURST | TG_LAUNCH, mods: 0, stack: ST_BIGGER, cap: 3, repeatable: false, rgba: 0x7AD8_FFFF, slots: 0 },
    Item { id: IT_HOP_SHOT, key: "hop_shot", name: "Hop Shot", line: "Short hops spit a star at a foe", pools: PL_ITEM, quality: 1, trigger: TR_SHORT_HOP, visual: "star", tags: TG_PROJECTILE, mods: 0, stack: ST_MORE, cap: 3, repeatable: true, rgba: 0xFFD8_4AFF, slots: 0 },
    Item { id: IT_LEDGE_SPIKES, key: "ledge_spikes", name: "Ledge Spikes", line: "Ledge grabs stab ice spikes up", pools: PL_ITEM, quality: 1, trigger: TR_LEDGE_GRAB, visual: "ice_spikes", tags: TG_BURST | TG_LAUNCH, mods: 0, stack: ST_STRONGER, cap: 3, repeatable: false, rgba: 0x9CF0_FFFF, slots: 0 },
    Item { id: IT_WHIFF_ECHO, key: "whiff_echo", name: "Whiff Echo", line: "A whiff leaves a ghost that swings again", pools: PL_ITEM, quality: 2, trigger: TR_WHIFF, visual: "ghost", tags: TG_BURST, mods: 0, stack: ST_MORE, cap: 3, repeatable: false, rgba: 0x6AF0_E0FF, slots: 0 },
    Item { id: IT_ROLL_MINES, key: "roll_mines", name: "Roll Mines", line: "Rolls drop a blinking mine", pools: PL_ITEM, quality: 1, trigger: TR_ROLL, visual: "mine", tags: TG_BURST | TG_LAUNCH, mods: 0, stack: ST_MORE, cap: 3, repeatable: true, rgba: 0xFF4A_4AFF, slots: 0 },
    Item { id: IT_SMASH_QUAKE, key: "smash_quake", name: "Smash Quake", line: "Smashes send shockwaves along the floor", pools: PL_ITEM, quality: 2, trigger: TR_SMASH_HIT, visual: "quake", tags: TG_BURST | TG_LAUNCH, mods: 0, stack: ST_BIGGER, cap: 3, repeatable: false, rgba: 0xE0B0_60FF, slots: 0 },
    Item { id: IT_MOON_DROPS, key: "moon_drops", name: "Moon Drops", line: "Two moons orbit you and nick foes", pools: PL_SHOP, quality: 2, trigger: TR_PASSIVE, visual: "moons", tags: TG_ORBITAL, mods: 0, stack: ST_MORE, cap: 4, repeatable: true, rgba: 0xE8EC_FFFF, slots: 0 },
    Item { id: IT_COIN_MAGNET, key: "coin_magnet", name: "Coin Magnet", line: "KO'd foes burst into coins for you", pools: PL_SHOP, quality: 1, trigger: TR_KO, visual: "coins", tags: 0, mods: 0, stack: ST_STRONGER, cap: 3, repeatable: true, rgba: 0xFFC8_30FF, slots: 0 },
    Item { id: IT_HOMING_INK, key: "homing_ink", name: "Homing Ink", line: "Your projectiles curve to foes", pools: PL_ITEM | PL_SHOP, quality: 2, trigger: TR_MODIFIER, visual: "ink", tags: 0, mods: TG_PROJECTILE, stack: ST_STRONGER, cap: 3, repeatable: false, rgba: 0xC85A_FFFF, slots: 0 },
    Item { id: IT_FIRE_LACE, key: "fire_lace", name: "Fire Lace", line: "Everything you make leaves embers and burns", pools: PL_ITEM | PL_DEVIL, quality: 3, trigger: TR_MODIFIER, visual: "embers", tags: 0, mods: TG_BURST | TG_TRAIL | TG_PROJECTILE | TG_SPARK, stack: ST_STRONGER, cap: 3, repeatable: false, rgba: 0xFF6A_20FF, slots: 1 },
    // ---- slice 2: the new triggers, the devil / angel / secret pools, the modifiers, Spare Heart
    Item { id: IT_LASER_PRISM, key: "laser_prism", name: "Laser Prism", line: "A laser that hits splits into three", pools: PL_ITEM, quality: 2, trigger: TR_LASER, visual: "prism", tags: TG_PROJECTILE, mods: 0, stack: ST_MORE, cap: 3, repeatable: false, rgba: 0xFF5A_8AFF, slots: 0 },
    Item { id: IT_CROUCH_THORNS, key: "crouch_thorns", name: "Crouch Thorns", line: "Crouch-cancel a hit: thorns strike back", pools: PL_SHOP, quality: 1, trigger: TR_CC, visual: "thorns", tags: TG_BURST | TG_LAUNCH, mods: 0, stack: ST_STRONGER, cap: 3, repeatable: false, rgba: 0x5AD0_6AFF, slots: 0 },
    Item { id: IT_PERFECT_GUARD, key: "perfect_guard", name: "Perfect Guard", line: "Powershields fire a gold beam back", pools: PL_BOSS, quality: 3, trigger: TR_POWERSHIELD, visual: "gold_beam", tags: TG_PROJECTILE | TG_LAUNCH, mods: 0, stack: ST_STRONGER, cap: 3, repeatable: false, rgba: 0xFFD0_40FF, slots: 0 },
    Item { id: IT_BRIMSTONE_JAB, key: "brimstone_jab", name: "Brimstone Jab", line: "Jab hits fire a blood-red beam", pools: PL_DEVIL, quality: 4, trigger: TR_JAB_HIT, visual: "brimstone", tags: TG_BURST | TG_LAUNCH, mods: 0, stack: ST_BIGGER, cap: 3, repeatable: false, rgba: 0xD01A_2AFF, slots: 2 },
    Item { id: IT_DEMON_WINGS, key: "demon_wings", name: "Demon Wings", line: "One more air jump; it rains feathers", pools: PL_DEVIL, quality: 3, trigger: TR_PASSIVE, visual: "wings", tags: TG_PROJECTILE, mods: 0, stack: ST_MORE, cap: 2, repeatable: false, rgba: 0x5A3A_7AFF, slots: 1 },
    Item { id: IT_CRIMSON_COUNT, key: "crimson_count", name: "Crimson Count", line: "Each KO adds a sigil that burns foes", pools: PL_DEVIL, quality: 3, trigger: TR_KO, visual: "sigils", tags: TG_ORBITAL, mods: 0, stack: ST_MORE, cap: 3, repeatable: false, rgba: 0xE028_48FF, slots: 1 },
    Item { id: IT_BLOODLUST, key: "bloodlust", name: "Bloodlust", line: "When hit, red tendrils lash back", pools: PL_DEVIL, quality: 2, trigger: TR_TAKE_HIT, visual: "tendrils", tags: TG_BURST | TG_LAUNCH, mods: 0, stack: ST_STRONGER, cap: 3, repeatable: false, rgba: 0xB018_38FF, slots: 1 },
    Item { id: IT_HALO, key: "halo", name: "Halo", line: "Your shots turn holy and pierce", pools: PL_ANGEL, quality: 4, trigger: TR_MODIFIER, visual: "halo", tags: 0, mods: TG_PROJECTILE | TG_SPARK, stack: ST_BIGGER, cap: 3, repeatable: false, rgba: 0xFFF4_B8FF, slots: 0 },
    Item { id: IT_GUARDIAN_FEATHER, key: "guardian_feather", name: "Guardian Feather", line: "Once a stock, eats a KO-strength hit", pools: PL_ANGEL, quality: 4, trigger: TR_PASSIVE, visual: "feather", tags: TG_ORBITAL, mods: 0, stack: ST_MORE, cap: 3, repeatable: false, rgba: 0xE8F4_FFFF, slots: 0 },
    Item { id: IT_ICE_GLASS, key: "ice_glass", name: "Ice Glass", line: "Shots turn to ice and shatter in three", pools: PL_ITEM | PL_SHOP, quality: 2, trigger: TR_MODIFIER, visual: "ice", tags: 0, mods: TG_PROJECTILE | TG_SPARK, stack: ST_STRONGER, cap: 3, repeatable: false, rgba: 0xA8E8_FFFF, slots: 0 },
    Item { id: IT_CHAIN_SPARK, key: "chain_spark", name: "Chain Spark", line: "Each item hit arcs on to the next foe", pools: PL_BOSS | PL_SECRET, quality: 3, trigger: TR_MODIFIER, visual: "chain", tags: 0, mods: TG_SPARK | TG_PROJECTILE, stack: ST_MORE, cap: 3, repeatable: false, rgba: 0x80D0_FFFF, slots: 0 },
    Item { id: IT_TWIN_SHOT, key: "twin_shot", name: "Twin Shot", line: "Doubles shots and orbitals, each at 60%", pools: PL_SHOP | PL_ANGEL, quality: 3, trigger: TR_MODIFIER, visual: "twin", tags: 0, mods: TG_PROJECTILE | TG_ORBITAL, stack: ST_MORE, cap: 2, repeatable: false, rgba: 0xFF9A_E0FF, slots: 0 },
    Item { id: IT_GRAVITY_WELL, key: "gravity_well", name: "Gravity Well", line: "Bursts pull foes in, then blow out", pools: PL_SECRET, quality: 3, trigger: TR_MODIFIER, visual: "well", tags: 0, mods: TG_BURST | TG_LAUNCH, stack: ST_BIGGER, cap: 3, repeatable: false, rgba: 0x8A5A_F0FF, slots: 0 },
    Item { id: IT_BIG_BANG, key: "big_bang", name: "Big Bang", line: "Bursts twice as big, a quake; -20% damage", pools: PL_DEVIL, quality: 3, trigger: TR_MODIFIER, visual: "big_bang", tags: 0, mods: TG_BURST | TG_LAUNCH, stack: ST_BIGGER, cap: 2, repeatable: false, rgba: 0xFF8A_30FF, slots: 1 },
    Item { id: IT_BOUNCY, key: "bouncy", name: "Bouncy", line: "Shots bounce off floors and walls", pools: PL_SECRET, quality: 2, trigger: TR_MODIFIER, visual: "bounce", tags: 0, mods: TG_PROJECTILE, stack: ST_MORE, cap: 3, repeatable: false, rgba: 0x6AE0_7AFF, slots: 0 },
    Item { id: IT_SPARE_HEART, key: "spare_heart", name: "Spare Heart", line: "+1 stock limit and +1 stock (to 8)", pools: PL_ANGEL | PL_SECRET, quality: 3, trigger: TR_PICKUP, visual: "heart", tags: 0, mods: 0, stack: ST_MORE, cap: 8, repeatable: true, rgba: 0xFF5A_7AFF, slots: 0 },
    Item { id: IT_DODGE_ORB, key: "dodge_orb", name: "Dodge Orb", line: "An orb leads every airdodge and wavedash", pools: PL_ITEM | PL_SHOP, quality: 2, trigger: TR_WHILE_AIRDODGE, visual: "orb_dodge", tags: TG_ORBITAL, mods: 0, stack: ST_MORE, cap: 3, repeatable: false, rgba: 0x7AF0_FFFF, slots: 0 },
    Item { id: IT_NAIR_RING, key: "nair_ring", name: "Nair Ring", line: "A ring of orbs circles your nair", pools: PL_ITEM, quality: 2, trigger: TR_WHILE_NAIR, visual: "orb_ring", tags: TG_ORBITAL, mods: 0, stack: ST_MORE, cap: 3, repeatable: false, rgba: 0xFFB0_4AFF, slots: 0 },
    Item { id: IT_DASH_ORB, key: "dash_orb", name: "Dash Orb", line: "An orb leads your dash and run", pools: PL_ITEM | PL_SHOP, quality: 1, trigger: TR_WHILE_DASH, visual: "orb_dash", tags: TG_ORBITAL, mods: 0, stack: ST_MORE, cap: 3, repeatable: false, rgba: 0x9AFF_7AFF, slots: 0 },
    Item { id: IT_SHIELD_ORBS, key: "shield_orbs", name: "Shield Orbs", line: "Orbs circle your shield and break on blocks", pools: PL_SHOP | PL_SECRET, quality: 2, trigger: TR_WHILE_SHIELD, visual: "orb_shield", tags: TG_ORBITAL, mods: 0, stack: ST_MORE, cap: 3, repeatable: false, rgba: 0xC0A0_FFFF, slots: 0 },
    // ---- synergy lane: proc items (their keyword text is procs::keyword) and co-op items
    Item { id: IT_EMBER_SEED, key: "ember_seed", name: "Ember Seed", line: "ON BURST HIT: may bloom a fire seed", pools: PL_ITEM | PL_SHOP, quality: 2, trigger: TR_ITEM_HIT, visual: "seed", tags: TG_BURST, mods: 0, stack: ST_STRONGER, cap: 3, repeatable: false, rgba: 0xFF7A_3AFF, slots: 0 },
    Item { id: IT_STATIC_COIL, key: "static_coil", name: "Static Coil", line: "ON SHOT HIT: may arc to another foe", pools: PL_ITEM | PL_SHOP, quality: 2, trigger: TR_ITEM_HIT, visual: "coil", tags: TG_SPARK, mods: 0, stack: ST_STRONGER, cap: 3, repeatable: false, rgba: 0x7AE8_FFFF, slots: 0 },
    Item { id: IT_FROST_BUD, key: "frost_bud", name: "Frost Bud", line: "ON SPARK HIT: may burst into frost", pools: PL_ITEM | PL_BOSS, quality: 2, trigger: TR_ITEM_HIT, visual: "frost", tags: TG_BURST, mods: 0, stack: ST_STRONGER, cap: 3, repeatable: false, rgba: 0xB8D8_FFFF, slots: 0 },
    Item { id: IT_KIN_TETHER, key: "kin_tether", name: "Kin Tether", line: "2P: a tether to your partner nicks foes", pools: PL_ITEM | PL_SHOP, quality: 2, trigger: TR_COOP, visual: "tether", tags: TG_TRAIL, mods: 0, stack: ST_STRONGER, cap: 3, repeatable: false, rgba: 0x8AFF_C8FF, slots: 0 },
    Item { id: IT_RALLY_BANNER, key: "rally_banner", name: "Rally Banner", line: "2P: your KOs rally your partner for 5 s", pools: PL_ITEM | PL_SHOP, quality: 2, trigger: TR_COOP, visual: "banner", tags: 0, mods: 0, stack: ST_STRONGER, cap: 3, repeatable: false, rgba: 0xFFB8_3AFF, slots: 0 },
    Item { id: IT_KINDRED_SPARK, key: "kindred_spark", name: "Kindred Spark", line: "2P: your hits near your partner proc theirs", pools: PL_ITEM | PL_BOSS, quality: 3, trigger: TR_COOP, visual: "kindred", tags: TG_SPARK, mods: 0, stack: ST_STRONGER, cap: 3, repeatable: false, rgba: 0xF0A0_FFFF, slots: 0 },
];
pub const ITEMS_LEN: usize = ITEMS.len();
/// Highest item id + 1 (the inventory arrays are indexed by id).
pub const ITEM_IDS: usize = 56;
/// The view's and the record's first block holds ids 0-47 (V_INV's stride); ids 48-55 are in V_INV2 and the record's
/// spare high halves (two bits a copy).
pub const INV_STRIDE: usize = 48;

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
        assert!(it.trigger <= TR_LAST, "item trigger unknown");
        assert!((it.slots > 0) == (it.pools & PL_DEVIL != 0), "a devil item has a slot price; only a devil item");
        assert!(it.slots <= 2, "a deal costs 1-2 stock slots");
        assert!((it.trigger == TR_MODIFIER) == (it.mods != 0), "a modifier must change tags; only a modifier may");
        assert!(it.id < INV_STRIDE as i32 || (it.cap <= 3 && !it.repeatable), "ids 48+ are stored two bits a copy");
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
        h = fnv_bytes(h, &[it.pools, it.quality, it.trigger, it.tags, it.mods, it.stack, it.cap, it.repeatable as u8, it.slots]);
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

/// Body groups (the host's melee_modes::tu::envoy_tint G_* bits): which part of the fighter an item lights.
pub const BG_HEAD: u32 = 1 << 0;
pub const BG_TORSO: u32 = 1 << 1;
pub const BG_ARMS: u32 = 1 << 2 | 1 << 3; // arms and hands
pub const BG_LEGS: u32 = 1 << 4 | 1 << 5; // legs and feet

/// The body group an item lights (notes/envoy-tints.md): hits and shots from the hands on the arms, movement on
/// the legs, defence and the rest on the torso; a devil deal also marks the head.
pub fn body_groups(id: i32) -> u32 {
    let Some(it) = item(id) else { return 0 };
    let base = match it.trigger {
        _ if id == IT_HALO => BG_TORSO, // the angel's halo: a guard look
        TR_NAIR_HIT | TR_JAB_HIT | TR_LASER | TR_SMASH_HIT | TR_WHIFF | TR_WHILE_NAIR => BG_ARMS,
        TR_WAVEDASH | TR_SHORT_HOP | TR_LEDGE_GRAB | TR_ROLL | TR_WHILE_AIRDODGE | TR_WHILE_DASH => BG_LEGS,
        TR_SHINE | TR_POWERSHIELD | TR_WHILE_SHIELD | TR_TAKE_HIT | TR_CC | TR_PICKUP | TR_KO => BG_TORSO,
        // modifiers: shot modifiers ride the hands, burst / trail modifiers the body
        TR_MODIFIER if it.mods & (TG_PROJECTILE | TG_SPARK) != 0 && it.mods & (TG_BURST | TG_TRAIL) == 0 => BG_ARMS,
        _ => match id {
            IT_DEMON_WINGS => BG_LEGS,
            _ => BG_TORSO,
        },
    };
    base | if it.pools & PL_DEVIL != 0 { BG_HEAD } else { 0 }
}
