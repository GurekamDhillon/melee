//! Envoy proc chains, named synergies, evolutions (docs/rust-port/notes/envoy-synergy.md).
//!
//! **Proc chains** (Risk of Rain 2): an item hit (a burst, a shot or a spark landing on a foe) may trigger a proc item
//! (`ON BURST HIT`, `ON SHOT HIT`, `ON SPARK HIT`); the proc's own hit is an item hit of its kind, so it may trigger
//! more procs. Every source has a proc coefficient (x1000) that scales the chance of what it triggers, and the chain
//! carries the product down. `resolve` works the whole chain out at once, breadth first, from integers only: the run
//! seed, the play frame, a per-run nonce and the player. No floats, no host state, no order hazards: the same inputs
//! give the same chain on every peer and every rollback replay. Depth is capped at `PROC_DEPTH`, the chain at
//! `PROC_MAX` procs, and a branch whose coefficient falls under `COEF_FLOOR` stops.
//!
//! **Synergies** (Gungeon / Hades duos): a named pair of items unlocks a combined effect. **Evolutions** (Vampire
//! Survivors): an item at its stack cap plus its partner item becomes an evolved form. Both are data here; the sim
//! applies each one at one place in its item code and shows a banner the first time it fires.
use crate::items::*;

/// Hit kinds (bits): what an item hit counts as for the procs listening.
pub const HK_BURST: u8 = 1;
pub const HK_SHOT: u8 = 2;
pub const HK_SPARK: u8 = 4;
pub const HK_TEXT: [&str; 3] = ["BURST", "SHOT", "SPARK"];

/// The chain's caps.
pub const PROC_DEPTH: u8 = 4;
pub const PROC_MAX: usize = 12;
/// A branch whose coefficient is under this (x1000) triggers nothing more.
pub const COEF_FLOOR: u32 = 60;

/// A proc item: what it listens for, its base chance (x1000), the kind of hit it makes, its own coefficient.
#[derive(Clone, Copy)]
pub struct ProcItem {
    pub id: i32,
    pub on: u8,
    pub chance: u32,
    pub makes: u8,
    pub coef: u32,
}

/// The proc items in their fixed evaluation order (index = `ProcIn::copies` slot).
pub const PROC_ITEMS: [ProcItem; 3] = [
    ProcItem { id: IT_EMBER_SEED, on: HK_BURST, chance: 250, makes: HK_BURST, coef: 500 },
    ProcItem { id: IT_STATIC_COIL, on: HK_SHOT, chance: 250, makes: HK_SPARK, coef: 600 },
    ProcItem { id: IT_FROST_BUD, on: HK_SPARK, chance: 200, makes: HK_BURST, coef: 400 },
];
pub const NPROC: usize = PROC_ITEMS.len();

/// The proc slot of item `id` (None: not a proc item).
pub fn proc_slot(id: i32) -> Option<usize> {
    PROC_ITEMS.iter().position(|q| q.id == id)
}

/// The kind of hit item `id` makes when it lands (0: none, a hit that procs nothing: coins, hearts).
pub const fn hit_kind(id: i32) -> u8 {
    match id {
        IT_SPARK_RING | IT_CHAIN_SPARK | IT_MOON_DROPS | IT_KINDRED_SPARK | IT_STATIC_COIL => HK_SPARK,
        IT_HOP_SHOT | IT_LASER_PRISM | IT_DEMON_WINGS | IT_ICE_GLASS | IT_PERFECT_GUARD => HK_SHOT,
        IT_DODGE_ORB | IT_NAIR_RING | IT_DASH_ORB | IT_SHIELD_ORBS | IT_KIN_TETHER => HK_SPARK,
        IT_GLIDE_WAKE | IT_SHINE_BELL | IT_LEDGE_SPIKES | IT_WHIFF_ECHO | IT_ROLL_MINES | IT_SMASH_QUAKE | IT_CROUCH_THORNS
        | IT_BRIMSTONE_JAB | IT_BLOODLUST | IT_GRAVITY_WELL | IT_EMBER_SEED | IT_FROST_BUD => HK_BURST,
        _ => 0,
    }
}

/// The proc coefficient (x1000) of a hit made by item `id`: big single hits 1.0, fast or many-hit sources less
/// (RoR2's rule: what hits often procs rarely), the chain's own procs their table value.
pub const fn proc_coef(id: i32) -> u32 {
    match id {
        IT_SPARK_RING | IT_LASER_PRISM => 500,
        IT_GLIDE_WAKE | IT_MOON_DROPS | IT_DEMON_WINGS | IT_ICE_GLASS => 300,
        IT_DODGE_ORB | IT_NAIR_RING | IT_DASH_ORB | IT_SHIELD_ORBS => 300,
        IT_CHAIN_SPARK | IT_KIN_TETHER => 200,
        IT_EMBER_SEED => 500,
        IT_STATIC_COIL => 600,
        IT_FROST_BUD => 400,
        IT_KINDRED_SPARK => 500,
        _ => 1000,
    }
}

/// What the resolver needs (all integers).
#[derive(Clone, Copy, Default)]
pub struct ProcIn {
    pub seed: u32,
    pub frame: i32,
    /// A per-run counter the sim bumps per root hit (two hits on one frame roll apart).
    pub nonce: u32,
    pub player: u8,
    /// Copies held per proc slot (0: not held).
    pub copies: [u8; NPROC],
    /// Extra chance per proc slot, x1000 of the base (synergies, evolutions, a partner's Rally Banner).
    pub bonus: [u32; NPROC],
}

/// One proc of a chain.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Proc {
    pub item: i32,
    /// 1 for a proc off the root hit, 2 for a proc off that proc ...
    pub depth: u8,
    /// The coefficient its own hit carries on (x1000).
    pub coef: u32,
    /// The index of the proc it came off (255: the root hit).
    pub parent: u8,
}

pub fn mix(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^ x >> 16
}

/// The chance (x1000, capped at 1000) a proc slot fires off a hit of coefficient `coef`.
pub fn chance(inp: &ProcIn, k: usize, coef: u32) -> u32 {
    let c = inp.copies[k] as u32;
    if c == 0 {
        return 0;
    }
    let stack = (1000 + 500 * (c.min(3) - 1)).min(3000);
    let base = PROC_ITEMS[k].chance * stack / 1000;
    (base * coef / 1000 * (1000 + inp.bonus[k]) / 1000).min(1000)
}

/// The chain off one root hit made by item `root` (its kind and coefficient from the tables): procs in breadth-first
/// order into `out`; returns how many. Never more than PROC_MAX, never deeper than PROC_DEPTH.
pub fn resolve(inp: &ProcIn, root: i32, out: &mut [Proc; PROC_MAX]) -> usize {
    resolve_from(inp, hit_kind(root), proc_coef(root), out)
}

pub fn resolve_from(inp: &ProcIn, kind: u8, coef: u32, out: &mut [Proc; PROC_MAX]) -> usize {
    let mut n = 0usize;
    // the event being expanded: the root (index -1), then each proc in turn
    let mut at: isize = -1;
    let base = inp.seed ^ (inp.frame as u32).wrapping_mul(0x9E37_79B9) ^ inp.nonce.wrapping_mul(0x85EB_CA6B) ^ (inp.player as u32) << 28;
    loop {
        let (k_hit, c_hit, depth) = if at < 0 { (kind, coef, 0u8) } else { let q = out[at as usize]; (hit_kind(q.item), q.coef, q.depth) };
        if k_hit != 0 && depth < PROC_DEPTH && c_hit >= COEF_FLOOR {
            for (k, q) in PROC_ITEMS.iter().enumerate() {
                if n >= PROC_MAX {
                    return n;
                }
                if q.on & k_hit == 0 {
                    continue;
                }
                let ch = chance(inp, k, c_hit);
                if ch == 0 {
                    continue;
                }
                let r = mix(base ^ ((at + 1) as u32).wrapping_mul(0xC2B2_AE35) ^ (k as u32 + 1).wrapping_mul(0x27D4_EB2F)) % 1000;
                if r < ch {
                    out[n] = Proc { item: q.id, depth: depth + 1, coef: c_hit * q.coef / 1000, parent: if at < 0 { 255 } else { at as u8 } };
                    n += 1;
                }
            }
        }
        at += 1;
        if at as usize >= n {
            return n;
        }
    }
}

// ---- synergies (named pairs)

#[derive(Clone, Copy)]
pub struct Synergy {
    pub key: &'static str,
    pub name: &'static str,
    pub a: i32,
    pub b: i32,
    /// The keyword text (the item language: TRIGGER: effect).
    pub line: &'static str,
    pub rgba: u32,
}

pub const SY_THUNDER_BELL: usize = 0;
pub const SY_SPECTRUM: usize = 1;
pub const SY_WILDFIRE: usize = 2;
pub const SY_BLOOD_MOON: usize = 3;
pub const SY_IRON_MAIDEN: usize = 4;
pub const SY_TECTONIC: usize = 5;
pub const SY_SUPERCONDUCTOR: usize = 6;
pub const SY_SEEKER_STARS: usize = 7;

pub const SYNERGIES: [Synergy; 8] = [
    Synergy { key: "thunder_bell", name: "Thunder Bell", a: IT_SHINE_BELL, b: IT_CHAIN_SPARK, line: "SHINE: the bell's ring arcs to every foe in reach", rgba: 0x9AE0_FFFF },
    Synergy { key: "spectrum", name: "Spectrum", a: IT_LASER_PRISM, b: IT_HALO, line: "LASER HIT: splits into five holy bolts, not three", rgba: 0xFFF0_C8FF },
    Synergy { key: "wildfire", name: "Wildfire", a: IT_EMBER_SEED, b: IT_FIRE_LACE, line: "ON BURST HIT: Ember Seed procs twice as often", rgba: 0xFF5A_1AFF },
    Synergy { key: "blood_moon", name: "Blood Moon", a: IT_MOON_DROPS, b: IT_CRIMSON_COUNT, line: "ORBIT: moons turn red and BURN what they nick", rgba: 0xE030_3AFF },
    Synergy { key: "iron_maiden", name: "Iron Maiden", a: IT_BLOODLUST, b: IT_CROUCH_THORNS, line: "TAKING A HIT: thorns grow as the tendrils lash", rgba: 0x7A2A_3AFF },
    Synergy { key: "tectonic", name: "Tectonic", a: IT_SMASH_QUAKE, b: IT_BIG_BANG, line: "SMASH HIT: the quake runs twice as far and tall", rgba: 0xC88A_40FF },
    Synergy { key: "superconductor", name: "Superconductor", a: IT_STATIC_COIL, b: IT_FROST_BUD, line: "ON SHOT HIT: Static arcs CHILL; frost bursts arc", rgba: 0xA0F0_FFFF },
    Synergy { key: "seeker_stars", name: "Seeker Stars", a: IT_HOP_SHOT, b: IT_HOMING_INK, line: "SHORT HOP: one more star, and stars home harder", rgba: 0xE8A0_FFFF },
];

// ---- evolutions (an item at its stack cap + a partner item)

#[derive(Clone, Copy)]
pub struct Evolution {
    pub key: &'static str,
    pub name: &'static str,
    /// Held at its cap.
    pub base: i32,
    pub partner: i32,
    pub line: &'static str,
    pub rgba: u32,
}

pub const EVO_STORM_CROWN: usize = 0;
pub const EVO_ECLIPSE: usize = 1;
pub const EVO_INFERNO_SEED: usize = 2;

pub const EVOLUTIONS: [Evolution; 3] = [
    Evolution { key: "storm_crown", name: "Storm Crown", base: IT_SPARK_RING, partner: IT_CHAIN_SPARK, line: "NAIR HIT: a crown of lightning strikes every foe near", rgba: 0xC8F0_FFFF },
    Evolution { key: "eclipse", name: "Eclipse", base: IT_MOON_DROPS, partner: IT_HALO, line: "ORBIT: black moons with gold rims hit twice as hard", rgba: 0xFFD8_60FF },
    Evolution { key: "inferno_seed", name: "Inferno Seed", base: IT_EMBER_SEED, partner: IT_FIRE_LACE, line: "ON BURST HIT: seeds LAUNCH, and proc +50%", rgba: 0xFF3A_10FF },
];

/// Whether copies `c_base` of an evolution's base (at its cap) and `c_partner` > 0 form it.
pub fn evolves(e: &Evolution, c_base: i32, c_partner: i32) -> bool {
    item(e.base).is_some_and(|it| c_base >= it.cap as i32) && c_partner > 0
}

/// The glossary of the item language's keywords (the help pages and the pause Items page).
pub const KEYWORDS: [(&str, &str); 8] = [
    ("ON BURST HIT", "when one of your bursts (rings, quakes, mines, beams) hits a foe"),
    ("ON SHOT HIT", "when one of your shots (stars, bolts, feathers, shards) hits a foe"),
    ("ON SPARK HIT", "when one of your sparks, arcs or orbs hits a foe"),
    ("PROC", "an item firing off another item's hit; a proc's hit can proc again, 4 deep at most"),
    ("COEFFICIENT", "how hard a hit procs: big hits 1.0, sparks 0.5, orbs and ticks 0.3; a chain multiplies them"),
    ("SYNERGY", "two named items held together: a combined effect (a banner the first time)"),
    ("EVOLUTION", "an item at its cap with its partner item: an evolved form"),
    ("2P", "a co-op item: it works through your partner, and nothing in a solo run"),
];

/// Keyword text for an item: the synergies and evolutions it takes part in, for the item cards.
pub fn partners(id: i32, out: &mut [&'static str; 6]) -> usize {
    let mut n = 0;
    for s in SYNERGIES.iter() {
        if (s.a == id || s.b == id) && n < out.len() {
            out[n] = s.name;
            n += 1;
        }
    }
    for e in EVOLUTIONS.iter() {
        if (e.base == id || e.partner == id) && n < out.len() {
            out[n] = e.name;
            n += 1;
        }
    }
    n
}

const fn lint() -> bool {
    let mut i = 0;
    while i < SYNERGIES.len() {
        assert!(SYNERGIES[i].a != SYNERGIES[i].b, "a synergy needs two items");
        i += 1;
    }
    true
}
const _: bool = lint();

#[cfg(test)]
mod tests {
    use super::*;

    fn all(seed: u32, c: u8, bonus: u32) -> ProcIn {
        ProcIn { seed, frame: 1234, nonce: 7, player: 0, copies: [c; NPROC], bonus: [bonus; NPROC] }
    }

    #[test]
    fn same_seed_same_chain() {
        for seed in 0..500u32 {
            let inp = all(seed, 3, 2000);
            let (mut a, mut b) = ([Proc::default(); PROC_MAX], [Proc::default(); PROC_MAX]);
            let na = resolve(&inp, IT_ROLL_MINES, &mut a);
            let nb = resolve(&inp, IT_ROLL_MINES, &mut b);
            assert_eq!(na, nb);
            assert_eq!(a[..na], b[..nb]);
        }
    }

    #[test]
    fn seeds_differ() {
        let mut seen = std::collections::BTreeSet::new();
        for seed in 0..200u32 {
            let mut o = [Proc::default(); PROC_MAX];
            let n = resolve(&all(seed, 1, 0), IT_ROLL_MINES, &mut o);
            seen.insert(o[..n].iter().map(|q| q.item).collect::<Vec<_>>());
        }
        assert!(seen.len() > 2, "the chain must depend on the seed");
    }

    #[test]
    fn depth_and_length_capped_even_at_certain_chance() {
        // every proc certain (bonus huge): the chain still stops at the depth and length caps
        for seed in 0..300u32 {
            let inp = all(seed, 3, 1_000_000);
            let mut o = [Proc::default(); PROC_MAX];
            let n = resolve_from(&inp, HK_BURST | HK_SHOT | HK_SPARK, 1000, &mut o);
            assert!(n <= PROC_MAX);
            for q in &o[..n] {
                assert!(q.depth >= 1 && q.depth <= PROC_DEPTH);
                assert!(q.parent == 255 || (q.parent as usize) < n);
                if q.parent != 255 {
                    assert_eq!(o[q.parent as usize].depth + 1, q.depth);
                }
            }
        }
    }

    #[test]
    fn coefficient_decays_and_floor_stops() {
        let inp = all(9, 3, 1_000_000);
        let mut o = [Proc::default(); PROC_MAX];
        let n = resolve(&inp, IT_ROLL_MINES, &mut o);
        assert!(n > 0);
        for q in &o[..n] {
            let parent = if q.parent == 255 { 1000 } else { o[q.parent as usize].coef };
            assert!(q.coef < parent, "a proc carries less than its source");
            assert!(parent >= COEF_FLOOR);
        }
        // a hit under the floor procs nothing
        assert_eq!(resolve_from(&inp, HK_BURST, COEF_FLOOR - 1, &mut o), 0);
    }

    #[test]
    fn nothing_held_nothing_procs() {
        let mut o = [Proc::default(); PROC_MAX];
        assert_eq!(resolve(&all(5, 0, 1_000_000), IT_SHINE_BELL, &mut o), 0);
        // a coin (no hit kind) procs nothing
        assert_eq!(resolve(&all(5, 3, 1_000_000), IT_COIN_MAGNET, &mut o), 0);
    }

    #[test]
    fn chance_scales_with_coefficient() {
        let inp = all(1, 1, 0);
        assert_eq!(chance(&inp, 0, 1000), 250);
        assert_eq!(chance(&inp, 0, 500), 125);
        assert!(chance(&all(1, 3, 0), 0, 1000) == 500);
        assert!(chance(&all(1, 3, 100_000), 0, 1000) == 1000);
    }

    #[test]
    fn rate_matches_chance() {
        // over many frames Ember Seed off a 1.0 burst procs near 25% at depth 1
        let mut hits = 0;
        for f in 0..20_000 {
            let inp = ProcIn { seed: 77, frame: f, nonce: 0, player: 1, copies: [1, 0, 0], bonus: [0; NPROC] };
            let mut o = [Proc::default(); PROC_MAX];
            let n = resolve(&inp, IT_SMASH_QUAKE, &mut o);
            hits += o[..n].iter().filter(|q| q.depth == 1).count();
        }
        assert!((4500..5500).contains(&hits), "{hits}");
    }

    #[test]
    fn tables_agree() {
        for q in PROC_ITEMS.iter() {
            assert!(item(q.id).is_some());
            assert_eq!(hit_kind(q.id), q.makes);
            assert_eq!(proc_coef(q.id), q.coef);
        }
        for s in SYNERGIES.iter() {
            assert!(item(s.a).is_some() && item(s.b).is_some(), "{}", s.key);
        }
        for e in EVOLUTIONS.iter() {
            assert!(item(e.base).is_some() && item(e.partner).is_some(), "{}", e.key);
        }
        assert!(evolves(&EVOLUTIONS[0], 3, 1) && !evolves(&EVOLUTIONS[0], 2, 1) && !evolves(&EVOLUTIONS[0], 3, 0));
    }
}
