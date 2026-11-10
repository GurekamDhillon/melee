// The crest vocabulary: families, moments, effects, words. Shared by build.rs (names -> codes, the lint) and the
// no_std library (the codes the sim matches on). The name tables and the constants are in the same order; the
// `codes_agree` test checks it.

pub const FAMILIES: [&str; 7] = ["ALWAYS", "RELAY", "BENCH", "SHIFT", "POOL", "OPENER", "HEAVY"];
pub const F_ALWAYS: u8 = 0;
pub const F_RELAY: u8 = 1;
pub const F_BENCH: u8 = 2;
pub const F_SHIFT: u8 = 3;
pub const F_POOL: u8 = 4;
pub const F_OPENER: u8 = 5;
pub const F_HEAVY: u8 = 6;

/// The moments (trigger events). Window moments (`fresh`, `benched`, `last_stock`, `always`) are states the rule
/// holds during; the others are edges the sim sees on one frame.
pub const MOMENTS: [&str; 11] =
    ["always", "tag_in", "tag_out", "fresh", "benched", "shift", "stock_lost", "last_stock", "ko_dealt", "opener", "boss"];
pub const M_ALWAYS: u8 = 0;
pub const M_TAG_IN: u8 = 1;
pub const M_TAG_OUT: u8 = 2;
pub const M_FRESH: u8 = 3;
pub const M_BENCHED: u8 = 4;
pub const M_SHIFT: u8 = 5;
pub const M_STOCK_LOST: u8 = 6;
pub const M_LAST_STOCK: u8 = 7;
pub const M_KO_DEALT: u8 = 8;
pub const M_OPENER: u8 = 9;
pub const M_BOSS: u8 = 10;

/// Which moments each family may key on (index: family).
pub const FAMILY_MOMENTS: [&[u8]; 7] = [
    &[M_ALWAYS],
    &[M_TAG_IN, M_FRESH],
    &[M_BENCHED, M_TAG_OUT, M_TAG_IN],
    &[M_SHIFT],
    &[M_STOCK_LOST, M_LAST_STOCK, M_KO_DEALT],
    &[M_OPENER],
    &[M_ALWAYS, M_BOSS, M_TAG_IN, M_SHIFT],
];

pub const EFFECTS: [&str; 17] = [
    "attack", "defense", "weight", "speed", "jump", "heal", "heal_both", "mark", "mark_all", "burn", "burn_all", "fresh",
    "fresh_len", "tag_cooldown", "partner_attack", "foe_percent", "first_shift",
];
pub const E_ATTACK: u8 = 0;
pub const E_DEFENSE: u8 = 1;
pub const E_WEIGHT: u8 = 2;
pub const E_SPEED: u8 = 3;
pub const E_JUMP: u8 = 4;
pub const E_HEAL: u8 = 5;
pub const E_HEAL_BOTH: u8 = 6;
pub const E_MARK: u8 = 7;
pub const E_MARK_ALL: u8 = 8;
pub const E_BURN: u8 = 9;
pub const E_BURN_ALL: u8 = 10;
pub const E_FRESH: u8 = 11;
pub const E_FRESH_LEN: u8 = 12;
pub const E_TAG_COOLDOWN: u8 = 13;
pub const E_PARTNER_ATTACK: u8 = 14;
pub const E_FOE_PERCENT: u8 = 15;
pub const E_FIRST_SHIFT: u8 = 16;

/// The four words (index 0: none).
pub const WORDS: [&str; 5] = ["", "Fresh", "Benched", "Marked", "Burning"];
pub const W_NONE: u8 = 0;
pub const W_FRESH: u8 = 1;
pub const W_BENCHED: u8 = 2;
pub const W_MARKED: u8 = 3;
pub const W_BURNING: u8 = 4;
/// Words every tag gives (the incoming fighter is Fresh, the outgoing one Benched): a reader needs no crest giver.
pub const INHERENT: [u8; 2] = [W_FRESH, W_BENCHED];

/// What each word does (the glossary; numbers live here, not in rule text).
pub const GLOSSARY: [&str; 5] = [
    "",
    "Fresh: just tagged in; 3 s in which RELAY crests act.",
    "Benched: the fighter waiting on the bench.",
    "Marked: a foe that takes +20% damage for a while.",
    "Burning: a foe that takes 1% per second for a while.",
];
/// Marked: the foe's defense ratio x1000 while marked.
pub const MARKED_DEFENSE: i32 = 1200;
/// Fresh: the base window on a tag-in (frames).
pub const FRESH_FRAMES: i32 = 180;
/// Burning: percent per tick and frames between ticks.
pub const BURN_TICK: i32 = 60;
