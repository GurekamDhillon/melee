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
/// V3 (envoy-synergy-v3.md 12.7): the pool may only use the damage sources (`dealt` and its filters, `taken` as damage
/// taken), `time`, `tag_in`, `shift` and `cashout`; the V2 count sources stay as codes only (the lint rejects them).
pub const CHARGES: [&str; 17] = [
    "hit", "taken", "landing", "jump", "time", "tag_in", "aerial_hit", "hit_100", "ko", "shift", "fire", "passive",
    "dealt", "aerial_dealt", "chilled_dealt", "offstage_dealt", "cashout",
];
pub const C_HIT: u8 = 0;
pub const C_TAKEN: u8 = 1;
pub const C_LANDING: u8 = 2;
pub const C_JUMP: u8 = 3;
pub const C_TIME: u8 = 4;
pub const C_TAG_IN: u8 = 5;
/// A hit landed with an aerial attack.
pub const C_AERIAL_HIT: u8 = 6;
/// A hit landed on a foe at 100% or more.
pub const C_HIT_100: u8 = 7;
/// A KO dealt (a foe lost a stock); with `reads`, only a foe carrying that word.
pub const C_KO: u8 = 8;
/// A stage shift.
pub const C_SHIFT: u8 = 9;
/// Another drive of the same set fired.
pub const C_FIRE: u8 = 10;
/// No charge: the rule is always on (it shows on the orbiter when it applies).
pub const C_PASSIVE: u8 = 11;
/// V3: percent dealt to a foe (any move, and fires to the drive's left).
pub const C_DEALT: u8 = 12;
/// V3: percent dealt with an aerial.
pub const C_AERIAL_DEALT: u8 = 13;
/// V3: percent dealt to a Chilled foe.
pub const C_CHILLED_DEALT: u8 = 14;
/// V3: percent dealt to a foe offstage.
pub const C_OFFSTAGE_DEALT: u8 = 15;
/// V3: a cash-out (a KO with 10% on the foe); with `when = burning_foe`, only on a Burning foe.
pub const C_CASHOUT: u8 = 16;
/// The sources that count hits, landings or jumps (V2): the V3 lint rejects them in the pool.
pub const COUNT_SOURCES: [u8; 7] = [C_HIT, C_LANDING, C_JUMP, C_AERIAL_HIT, C_HIT_100, C_FIRE, C_KO];
/// The damage sources (charge in percent x CHARGE_UNIT; `taken` is damage taken).
pub const DAMAGE_SOURCES: [u8; 5] = [C_DEALT, C_AERIAL_DEALT, C_CHILLED_DEALT, C_OFFSTAGE_DEALT, C_TAKEN];

/// The sim's effects (envoy-core-v2.md 2.4). Drive effects first, then keystone rules.
pub const EFFECTS: [&str; 30] = [
    "burst", "nova", "bolt", "haste", "shield", "shockwave", "chill", "shatter", "relay", "spire", "thorns", "heal",
    "airjump", "feathers", "mark", "wildfire", "repeat", "surge", "tailwind", "plague", "echo", "overclock", "twin",
    "winter", "glass", "finale", "heat", "gain", "insulate", "neighbour",
];
pub const E_BURST: u8 = 0;
pub const E_NOVA: u8 = 1;
pub const E_BOLT: u8 = 2;
pub const E_HASTE: u8 = 3;
pub const E_SHIELD: u8 = 4;
pub const E_SHOCKWAVE: u8 = 5;
pub const E_CHILL: u8 = 6;
pub const E_SHATTER: u8 = 7;
pub const E_RELAY: u8 = 8;
pub const E_SPIRE: u8 = 9;
pub const E_THORNS: u8 = 10;
pub const E_HEAL: u8 = 11;
pub const E_AIRJUMP: u8 = 12;
pub const E_FEATHERS: u8 = 13;
pub const E_MARK: u8 = 14;
pub const E_WILDFIRE: u8 = 15;
pub const E_REPEAT: u8 = 16;
pub const E_SURGE: u8 = 17;
pub const E_TAILWIND: u8 = 18;
pub const E_PLAGUE: u8 = 19;
pub const E_ECHO: u8 = 20;
pub const E_OVERCLOCK: u8 = 21;
pub const E_TWIN: u8 = 22;
pub const E_WINTER: u8 = 23;
pub const E_GLASS: u8 = 24;
/// V3 keystone: Grand Finale.
pub const E_FINALE: u8 = 25;
/// V3 drive effects with no fire: a +Heat drive, an xGain drive, Insulate, a neighbour (Echo).
pub const E_HEAT: u8 = 26;
pub const E_GAIN: u8 = 27;
pub const E_INSULATE: u8 = 28;
pub const E_NEIGHBOUR: u8 = 29;
/// The first keystone effect (V2's). V3: the keystone rules are KEY_EFFECTS; every other effect is a drive's.
pub const E_KEYSTONE0: u8 = E_PLAGUE;
pub const KEY_EFFECTS: [u8; 7] = [E_PLAGUE, E_ECHO, E_OVERCLOCK, E_TWIN, E_WINTER, E_GLASS, E_FINALE];
pub const fn is_key_effect(e: u8) -> bool {
    let mut i = 0;
    while i < KEY_EFFECTS.len() {
        if KEY_EFFECTS[i] == e {
            return true;
        }
        i += 1;
    }
    false
}

/// V3 (12.7): what a drive is. A trigger fires when charged; the others act without firing.
pub const ROLES: [&str; 8] = ["trigger", "shaper", "heat_add", "gain_mul", "heat_mul", "neighbour", "scaler", "economy"];
pub const R_TRIGGER: u8 = 0;
pub const R_SHAPER: u8 = 1;
pub const R_HEAT_ADD: u8 = 2;
pub const R_GAIN_MUL: u8 = 3;
pub const R_HEAT_MUL: u8 = 4;
pub const R_NEIGHBOUR: u8 = 5;
pub const R_SCALER: u8 = 6;
pub const R_ECONOMY: u8 = 7;
/// V3 tiers: the beam colour, the shop price.
pub const TIERS: [&str; 3] = ["common", "rare", "epic"];
pub const T_COMMON: u8 = 0;
pub const T_RARE: u8 = 1;
pub const T_EPIC: u8 = 2;
/// V3: the condition of a heat_add / gain_mul drive, or the filter on a cashout trigger (index 0: none).
pub const WHENS: [&str; 9] = ["", "haste", "aerial", "offstage", "lcancel", "lcancel_streak", "tech", "new_move", "burning_foe"];
pub const WH_NONE: u8 = 0;
pub const WH_HASTE: u8 = 1;
pub const WH_AERIAL: u8 = 2;
pub const WH_OFFSTAGE: u8 = 3;
pub const WH_LCANCEL: u8 = 4;
pub const WH_LCANCEL_STREAK: u8 = 5;
pub const WH_TECH: u8 = 6;
pub const WH_NEW_MOVE: u8 = 7;
pub const WH_BURNING_FOE: u8 = 8;
/// The technique conditions (the lint: such a drive has a same-family drive with no technique condition).
pub const TECH_WHENS: [u8; 3] = [WH_LCANCEL, WH_LCANCEL_STREAK, WH_TECH];
/// The effects that launch (Q6: only the drives you time yourself), through `gd_envoy_blast`.
pub const LAUNCHES: [u8; 3] = [E_NOVA, E_SHOCKWAVE, E_SPIRE];

/// The presentation's effect shapes (`look`): envoy_ui draws each fire with its drive's look in its family colour.
pub const LOOKS: [&str; 10] = ["burst", "ring", "bolt", "spikes", "beam", "orb", "line", "rain", "wave", "glow"];

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
    "Burning: the foe takes 1.5% a second per stack.",
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
pub const BURN_TICK_PCT: i32 = 1500;
pub const BURN_TICK: i32 = 60;
pub const BURN_FRAMES: i32 = 240;
pub const BURN_STACKS: i32 = 3;
/// Inferno (the RED set): Burning stacks to this.
pub const INFERNO_STACKS: i32 = 6;
/// Chilled / Haste: the walk and dash ratio x1000; Slow (a price): x0.85.
pub const CHILL_SPEED: i32 = 700;
pub const HASTE_SPEED: i32 = 1300;
pub const SLOW_SPEED: i32 = 850;
/// Marked: the hits it takes deal this much more (x1000).
pub const MARK_MORE: i32 = 300;
/// On a boss, Chilled and Marked act at half strength (envoy-core-v2.md 2.6): x0.85 speed, +15%.
pub const BOSS_CHILL_SPEED: i32 = 850;
pub const BOSS_MARK_MORE: i32 = 150;
/// Lock (a price): the tag cooldown in frames.
pub const LOCK_TAG_COOLDOWN: i32 = 600;
/// Level scale of a drive's number x1000: level 1, 2, 3.
pub const LEVEL_SCALE: [i32; 3] = [1000, 1600, 2400];
pub const MAX_LEVEL: i32 = 3;
/// The fire queue: at most one fire per fighter in this many frames; Relay's volley spacing.
pub const FIRE_GAP: i32 = 20;
pub const RELAY_GAP: i32 = 6;

// ---- V3: Heat (envoy-synergy-v3.md 3, 12.2). Integers only; folded into POOL_DIGEST (spec::digest).
/// 1 Heat in ticks: 2 (the overkill half) x 1000 (dmg_m) x 1000 (coef_m) x 1000 (gain_m).
pub const HEAT_TICK: i64 = 2_000_000_000;
/// The marks (Warm, Hot, Blazing) and the cap, in Heat.
pub const HEAT_MARKS: [i64; 3] = [40, 90, 165];
pub const HEAT_CAP: i64 = 225;
/// What each state does to a fire (x1000): Cold, Warm, Hot, Blazing.
pub const STATE_SCALE: [i32; 4] = [1000, 1250, 1500, 2000];
/// The overkill line: damage that lands on a foe above this percent (x1000) counts half.
pub const OVERKILL_M: i32 = 150_000;
/// Gain multiplier cap (x1000).
pub const GAIN_CAP: i32 = 4000;
/// Charge: 1% = 1e9 units (dmg_m x coef_m x rate_m); a need of N (percent x1000) is N x CHARGE_UNIT.
pub const CHARGE_UNIT: i64 = 1_000_000;
/// Word damage (Burning ticks) as a record: its coefficient (Inferno: the second).
pub const WORD_COEF: [i32; 2] = [200, 500];
/// A cash-out needs this much dealt to the foe this stock (x1000).
pub const CASHOUT_MIN: i32 = 10_000;
/// Coins: 1 coin = 15 Heat at pay 1000 (15 x 2e9 x 1000 ticks). Pips: 150 Heat.
pub const COIN_UNIT: i64 = 30_000_000_000_000;
pub const PIP_UNIT: i64 = 300_000_000_000_000;
/// The flare: Heat ticks x pay / FLARE_DIV is percent x1000, at most FLARE_MAX.
pub const FLARE_DIV: i64 = 30_000_000_000;
pub const FLARE_MAX: i32 = 15_000;
/// What a foe pays on a cash-out (x1000): a pack minion, a fighter (duel), an elite, a boss break, the last break.
pub const PAY_MINION: i32 = 500;
pub const PAY_FIGHTER: i32 = 1000;
pub const PAY_ELITE: i32 = 1500;
pub const PAY_BREAK: i32 = 1000;
pub const PAY_LAST: i32 = 2000;
/// One fire's damage cap (x1000), a launching timed fire's.
pub const FIRE_CAP: i32 = 30_000;
pub const LAUNCH_CAP: i32 = 40_000;
/// Chains: links, frames between links, frames between chains.
pub const CHAIN_LINKS: i32 = 4;
pub const LINK_GAP: i32 = 6;
pub const CHAIN_GAP: i32 = 20;
/// Technique marks (frames): L-cancel, wavedash, tech.
pub const MARK_FRAMES: [i32; 3] = [20, 60, 30];
/// Insulate's allowance (x1000 percent).
pub const INSULATE_FREE: i32 = 20_000;
/// The active (Volley): pips one use costs and the pips held at the run's start.
pub const VOLLEY_PIPS: i64 = 3;
/// Coins: a fight cleared, + no stock lost, an edgeguard KO, interest per 10 held (to 5).
pub const COINS_CLEAR: i32 = 20;
pub const COINS_CLEAN: i32 = 10;
pub const COINS_EDGEGUARD: i32 = 10;
pub const INTEREST_MAX: i32 = 5;
/// Shop prices (coins): a drive by tier, a level (to 2, to 3), a stock, the first reroll (+10 each).
pub const PRICE_TIER: [i32; 3] = [30, 80, 150];
pub const PRICE_LEVEL: [i32; 2] = [60, 120];
pub const PRICE_STOCK: i32 = 100;
pub const PRICE_REROLL: i32 = 10;
/// Grand Finale: +1% knockback per this many ticks (4.5 Heat), to +50%.
pub const FINALE_TICKS: i64 = 9_000_000_000;
/// The numbers above, folded into the pool digest (a peer with other Heat rules cannot sync).
pub const HEAT_CONSTS: [i64; 16] = [
    HEAT_TICK, HEAT_MARKS[0], HEAT_MARKS[1], HEAT_MARKS[2], HEAT_CAP, OVERKILL_M as i64, GAIN_CAP as i64, CHARGE_UNIT,
    CASHOUT_MIN as i64, COIN_UNIT, PIP_UNIT, FLARE_DIV, FIRE_CAP as i64, INSULATE_FREE as i64, VOLLEY_PIPS,
    BURN_TICK_PCT as i64,
];
