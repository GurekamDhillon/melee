// The drive vocabulary (envoy-core-v2.md 2): kinds, families, charge sources, effects, words, prices. Shared by
// build.rs (names -> codes, the lint) and the no_std library (the codes the sim matches on). The name tables and the
// constants are in the same order; the `codes_agree` test checks it.

/// A pool record is a drive or a keystone.
pub const KINDS: [&str; 2] = ["drive", "keystone"];
pub const K_DRIVE: u8 = 0;
pub const K_KEYSTONE: u8 = 1;

/// The six families: the six model bodies (envoy_drives), shape and colour.
pub const FAMILIES: [&str; 6] = ["RED", "GREEN", "BLUE", "YELLOW", "PURPLE", "WHITE"];
pub const F_RED: u8 = 0;
pub const F_GREEN: u8 = 1;
pub const F_BLUE: u8 = 2;
pub const F_YELLOW: u8 = 3;
pub const F_PURPLE: u8 = 4;
pub const F_WHITE: u8 = 5;
/// What each family means (cards).
pub const FAMILY_MEANING: [&str; 6] = ["damage", "speed", "defence", "air", "status", "wild"];

/// What fills a drive's charge (one source per drive). `need` counts these, or milliseconds for `time`.
pub const CHARGES: [&str; 6] = ["hit", "taken", "landing", "jump", "time", "tag_in"];
pub const C_HIT: u8 = 0;
pub const C_TAKEN: u8 = 1;
pub const C_LANDING: u8 = 2;
pub const C_JUMP: u8 = 3;
pub const C_TIME: u8 = 4;
pub const C_TAG_IN: u8 = 5;

/// The sim's effects (envoy-core-v2.md 2.4). Drive effects first, then keystone rules.
pub const EFFECTS: [&str; 12] =
    ["burst", "nova", "bolt", "haste", "shield", "shockwave", "chill", "shatter", "relay", "plague", "echo", "overclock"];
pub const E_BURST: u8 = 0;
pub const E_NOVA: u8 = 1;
pub const E_BOLT: u8 = 2;
pub const E_HASTE: u8 = 3;
pub const E_SHIELD: u8 = 4;
pub const E_SHOCKWAVE: u8 = 5;
pub const E_CHILL: u8 = 6;
pub const E_SHATTER: u8 = 7;
pub const E_RELAY: u8 = 8;
pub const E_PLAGUE: u8 = 9;
pub const E_ECHO: u8 = 10;
pub const E_OVERCLOCK: u8 = 11;
/// The first keystone effect: effects below it are drive effects.
pub const E_KEYSTONE0: u8 = E_PLAGUE;

/// The four words (index 0: none).
pub const WORDS: [&str; 5] = ["", "Burning", "Chilled", "Marked", "Haste"];
pub const W_NONE: u8 = 0;
pub const W_BURNING: u8 = 1;
pub const W_CHILLED: u8 = 2;
pub const W_MARKED: u8 = 3;
pub const W_HASTE: u8 = 4;

/// What each word does (the glossary; numbers live here, not in rule text).
pub const GLOSSARY: [&str; 5] = [
    "",
    "Burning: the foe takes 2% a second.",
    "Chilled: the foe walks and dashes 30% slower.",
    "Marked: the foe takes 30% more.",
    "Haste: you walk and dash 30% faster.",
];

/// A keystone's price: one of the seven, or `own` (the rule is its own price).
pub const PRICES: [&str; 9] = ["", "Slow", "Low", "Fragile", "Weak", "Vulnerable", "Lock", "Bleed", "own"];
pub const P_NONE: u8 = 0;
pub const P_SLOW: u8 = 1;
pub const P_LOW: u8 = 2;
pub const P_FRAGILE: u8 = 3;
pub const P_WEAK: u8 = 4;
pub const P_VULNERABLE: u8 = 5;
pub const P_LOCK: u8 = 6;
pub const P_BLEED: u8 = 7;
pub const P_OWN: u8 = 8;
pub const PRICE_TEXT: [&str; 9] = [
    "",
    "Slow: walk and dash -15%.",
    "Low: jump -15%.",
    "Fragile: you take 25% more.",
    "Weak: you deal 20% less.",
    "Vulnerable: statuses on you last twice as long.",
    "Lock: tag cooldown 10 s.",
    "Bleed: lose 1% whenever a drive fires.",
    "The rule is its own price.",
];

/// Burning: percent x1000 per tick and stack, frames between ticks, frames a Burning lasts, the most stacks.
pub const BURN_TICK_PCT: i32 = 2000;
pub const BURN_TICK: i32 = 60;
pub const BURN_FRAMES: i32 = 240;
pub const BURN_STACKS: i32 = 3;
/// Chilled / Haste: the walk and dash ratio x1000.
pub const CHILL_SPEED: i32 = 700;
pub const HASTE_SPEED: i32 = 1300;
/// Level scale of a drive's number x1000: level 1, 2, 3.
pub const LEVEL_SCALE: [i32; 3] = [1000, 1600, 2400];
pub const MAX_LEVEL: i32 = 3;
/// The fire queue: at most one fire per fighter in this many frames; Relay's volley spacing.
pub const FIRE_GAP: i32 = 20;
pub const RELAY_GAP: i32 = 6;
