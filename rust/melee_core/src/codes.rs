//! Opt-in Slippi / community gameplay codes as match rules (docs/rust-port/notes/gameplay-codes.md).
//!
//! One word for the whole process, set before the match boots (`--codes LIST` or `MELEE_CODES`) and constant during
//! it, so a rollback (which rewrites every static) and a replay see the same value. Every bit is off by default: with
//! the word 0 each hook site runs the game's own code. The word enters the netplay handshake digest (both peers must
//! run the same codes) and the `.slp` metadata (`gameplayCodes`).
//!
//! Each code reimplements the documented behaviour of the Slippi / community code it is named after; the citation is
//! at the hook site and in the notes file.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// UCF 0.84 (dashback, shield drop, 1.0 cardinals, SDI / tumble second chances): the existing ftUcf sites.
pub const UCF: u32 = 1 << 0;
/// Slippi "Frozen Pokemon Stadium": the stage never transforms.
pub const FROZEN_PS: u32 = 1 << 1;
/// Slippi "Frozen Stages" (all): Frozen PS, Yoshi's Story without Shy Guys, Dream Land 64 without wind,
/// Final Destination's backdrop holds still.
pub const FROZEN_STAGES: u32 = 1 << 2;
/// Slippi "Disable Wobbling" (PreventWobbling): the 4th distinct Ice Climbers hit on the grabbed fighter breaks the grab.
pub const NO_WOBBLE: u32 = 1 << 3;
/// Slippi "Ledge Grab Limit": on a time-out, a fighter with more than [`lgl_limit`] ledge grabs loses.
pub const LGL: u32 = 1 << 4;
/// Slippi Neutral Spawn (part of the tournament / stealth / friendlies sets).
pub const NEUTRAL_SPAWN: u32 = 1 << 5;
/// "Skip results screen": a VS match goes back to character select.
pub const SKIP_RESULTS: u32 = 1 << 6;
/// 20XX TE hand-warmer mode: in a 1-minute timed match fighters are intangible, do not push, and share ledges.
pub const HANDWARMERS: u32 = 1 << 7;
/// Slippi FreezeGlitchFix (part of the tournament sets): the Ice Climbers freeze glitch.
pub const FREEZE_FIX: u32 = 1 << 8;
/// Stealth: the same gameplay set as tournament with no on-screen code indicator (we draw none; recorded only).
pub const STEALTH: u32 = 1 << 9;

/// Slippi's "Mods: Tournament" gameplay part (the CSS / nametag codes of that set are cosmetic and not here).
pub const PRESET_TOURNAMENT: u32 = NEUTRAL_SPAWN | FREEZE_FIX;
/// Slippi's "Mods: Stealth": the tournament gameplay set, nothing visible.
pub const PRESET_STEALTH: u32 = PRESET_TOURNAMENT | STEALTH;

/// (name, bit) in the order `describe` prints them.
pub const NAMES: &[(&str, u32)] = &[
    ("ucf", UCF),
    ("frozen-ps", FROZEN_PS),
    ("frozen-stages", FROZEN_STAGES),
    ("no-wobble", NO_WOBBLE),
    ("lgl", LGL),
    ("neutral-spawn", NEUTRAL_SPAWN),
    ("skip-results", SKIP_RESULTS),
    ("handwarmers", HANDWARMERS),
    ("freeze-fix", FREEZE_FIX),
    ("stealth", STEALTH),
];

/// The default ledge-grab limit (Slippi's LGL code, see the notes for the source of the number).
pub const LGL_DEFAULT: u32 = 45;

static WORD: AtomicU32 = AtomicU32::new(0);
static LGL_N: AtomicU32 = AtomicU32::new(LGL_DEFAULT);
static CONFIGURED: AtomicBool = AtomicBool::new(false);

/// The active codes word.
#[inline]
pub fn word() -> u32 {
    WORD.load(Ordering::Relaxed)
}

/// Is `bit` on?
#[inline]
pub fn on(bit: u32) -> bool {
    word() & bit != 0
}

/// Were the codes given explicitly (`--codes`, even `none`)? Only then does the UCF bit override the `ucf` setting.
pub fn configured() -> bool {
    CONFIGURED.load(Ordering::Relaxed)
}

/// The ledge-grab limit of [`LGL`].
pub fn lgl_limit() -> u32 {
    LGL_N.load(Ordering::Relaxed)
}

/// Set the word (and the LGL limit); marks the codes as configured.
pub fn set(w: u32, lgl: u32) {
    WORD.store(w, Ordering::Relaxed);
    LGL_N.store(lgl, Ordering::Relaxed);
    CONFIGURED.store(true, Ordering::Relaxed);
}

/// Back to the unconfigured vanilla state (tests).
pub fn reset() {
    WORD.store(0, Ordering::Relaxed);
    LGL_N.store(LGL_DEFAULT, Ordering::Relaxed);
    CONFIGURED.store(false, Ordering::Relaxed);
}

/// Parse `a,b,c`: code names, `tournament`, `stealth` (presets), `none`, `lgl:N` (limit N), `0x..` (a raw word).
pub fn parse(s: &str) -> Result<(u32, u32), String> {
    let (mut w, mut lgl) = (0u32, LGL_DEFAULT);
    for t in s.split([',', ' ', '+']).map(str::trim).filter(|t| !t.is_empty()) {
        let t = t.to_ascii_lowercase();
        if let Some(h) = t.strip_prefix("0x") {
            w |= u32::from_str_radix(h, 16).map_err(|_| format!("codes: bad word {t}"))?;
            continue;
        }
        if let Some(n) = t.strip_prefix("lgl:").or_else(|| t.strip_prefix("lgl=")) {
            lgl = n.parse().map_err(|_| format!("codes: bad ledge-grab limit {n}"))?;
            w |= LGL;
            continue;
        }
        w |= match t.as_str() {
            "none" | "off" | "vanilla" => 0,
            "tournament" => PRESET_TOURNAMENT,
            "stealth" | "stealth-tournament" => PRESET_STEALTH,
            "dashback" | "shield-drop" | "ucf084" => UCF,
            "frozen" | "frozen-stadium" => FROZEN_PS,
            "frozen-all" => FROZEN_STAGES,
            "wobbling" | "disable-wobbling" => NO_WOBBLE,
            "ledge-grab-limit" => LGL,
            "hand-warmers" | "handwarmer" => HANDWARMERS,
            "freeze-glitch-fix" => FREEZE_FIX,
            other => match NAMES.iter().find(|(n, _)| *n == other) {
                Some((_, b)) => *b,
                None => return Err(format!("codes: unknown code '{other}' (known: {}, tournament, stealth, none, lgl:N)", NAMES.iter().map(|n| n.0).collect::<Vec<_>>().join(", "))),
            },
        };
    }
    if w & !all() != 0 {
        return Err(format!("codes: unknown bits 0x{:x}", w & !all()));
    }
    Ok((w, lgl))
}

/// Every defined bit.
pub fn all() -> u32 {
    NAMES.iter().fold(0, |a, n| a | n.1)
}

/// `ucf, lgl(45), ...` or `none`.
pub fn describe(w: u32, lgl: u32) -> String {
    let v: Vec<String> = NAMES
        .iter()
        .filter(|(_, b)| w & b != 0)
        .map(|(n, b)| if *b == LGL { format!("{n}({lgl})") } else { n.to_string() })
        .collect();
    if v.is_empty() {
        "none".into()
    } else {
        v.join(", ")
    }
}

/// The handshake / metadata form: `0` for none, else `<hex word>` plus `:<limit>` when LGL is on.
pub fn encode(w: u32, lgl: u32) -> String {
    if w & LGL != 0 {
        format!("{w:x}:{lgl}")
    } else {
        format!("{w:x}")
    }
}

/// Inverse of [`encode`].
pub fn decode(s: &str) -> Option<(u32, u32)> {
    let (a, b) = s.split_once(':').unwrap_or((s, ""));
    let w = u32::from_str_radix(a, 16).ok()?;
    let l = if b.is_empty() { LGL_DEFAULT } else { b.parse().ok()? };
    Some((w, l))
}

/// `--codes LIST` in `args`, else `MELEE_CODES`; sets the word when either is present. Returns the description.
pub fn init_from_args(args: &[String]) -> Result<Option<String>, String> {
    let v = args.iter().position(|a| a == "--codes").and_then(|i| args.get(i + 1)).cloned().or_else(|| std::env::var("MELEE_CODES").ok().filter(|v| !v.is_empty()));
    let Some(v) = v else { return Ok(None) };
    let (w, l) = parse(&v)?;
    set(w, l);
    Ok(Some(describe(w, l)))
}

// ---- per-match state (statics: a rollback load rewrites them with the rest of the executable's data) ----

static mut HW_ACTIVE: bool = false;
/// Ledge grabs per player slot this match ([`LGL`]).
pub static mut LEDGE_GRABS: [u32; 6] = [0; 6];

/// At match start (`fn_8016E730`): reset the counters; hand-warmers apply to a timed (not stock) 1-minute match, as
/// 20XX TE's hand-warmer mode decides from the match's own rules.
pub fn match_begin(is_stock: bool, timer_on: bool, time_limit_secs: u32) {
    unsafe {
        HW_ACTIVE = on(HANDWARMERS) && !is_stock && timer_on && time_limit_secs == 60;
        LEDGE_GRABS = [0; 6];
    }
}

/// Hand-warmer mode is in effect for this match.
#[inline]
pub fn handwarmers_active() -> bool {
    unsafe { HW_ACTIVE }
}

/// A successful ledge grab by `slot` (`ftCliffCommon_80081298` entering CliffCatch).
pub fn note_ledge_grab(slot: i32) {
    if on(LGL) && (0..6).contains(&slot) {
        unsafe { LEDGE_GRABS[slot as usize] = LEDGE_GRABS[slot as usize].wrapping_add(1) };
    }
}

/// [`LGL`] on and `slot` over the limit.
pub fn over_ledge_limit(slot: usize) -> bool {
    on(LGL) && slot < 6 && unsafe { LEDGE_GRABS[slot] } > lgl_limit()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_round_trip() {
        assert_eq!(parse("none").unwrap().0, 0);
        assert_eq!(parse("ucf,frozen-ps").unwrap().0, UCF | FROZEN_PS);
        assert_eq!(parse("tournament").unwrap().0, NEUTRAL_SPAWN | FREEZE_FIX);
        assert_eq!(parse("stealth").unwrap().0, NEUTRAL_SPAWN | FREEZE_FIX | STEALTH);
        assert_eq!(parse("lgl:60").unwrap(), (LGL, 60));
        assert!(parse("bogus").is_err());
        for w in [0, UCF, all()] {
            assert_eq!(decode(&encode(w, 33)).unwrap(), (w, if w & LGL != 0 { 33 } else { LGL_DEFAULT }));
        }
        assert_eq!(describe(LGL | UCF, 45), "ucf, lgl(45)");
    }
}
