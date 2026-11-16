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
/// A Lab mod runs the match (gd-lab-v1, lab-ui-spec.md 5.2): lab matches are untimed. Set once before the match boots
/// (offline or `--np-lab`, unless the command line asks for a time limit) and constant during it, like [`WORD`], so a
/// rollback or lab load rewrites it with the same value. Off: the game's own rules.
static LAB_UNTIMED: AtomicBool = AtomicBool::new(false);

/// A replay that says which codes its match ran (our own `.slp` export's `gameplayCodes` metadata) without Neutral
/// Spawn: the spawn stays on the stage's joints instead of the Neutral Spawn every other replay is played with
/// (Slippi's codesets all carry it). Set before the match boots.
static REPLAY_NO_NEUTRAL_SPAWN: AtomicBool = AtomicBool::new(false);

/// See [`REPLAY_NO_NEUTRAL_SPAWN`].
pub fn set_replay_no_neutral_spawn(on: bool) {
    REPLAY_NO_NEUTRAL_SPAWN.store(on, Ordering::Relaxed);
}

/// See [`REPLAY_NO_NEUTRAL_SPAWN`].
#[inline]
pub fn replay_no_neutral_spawn() -> bool {
    REPLAY_NO_NEUTRAL_SPAWN.load(Ordering::Relaxed)
}

/// Make lab matches untimed (see [`LAB_UNTIMED`]).
pub fn set_lab_untimed(on: bool) {
    LAB_UNTIMED.store(on, Ordering::Relaxed);
}

/// Lab matches are untimed: `fn_8016E730` clears the match's timer before it starts.
#[inline]
pub fn lab_untimed() -> bool {
    LAB_UNTIMED.load(Ordering::Relaxed)
}

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

/// Turn `bit` on without marking the codes configured (slpcheck: a code found in the replay's own Gecko list).
pub fn enable(bit: u32) {
    WORD.fetch_or(bit, Ordering::Relaxed);
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

/// Slippi's PSCameraIndependentMonitor (C2 0x801D24FC, slippi-ssbm-asm `Online/Core/Hacks/Stadium`): the Pokemon Stadium
/// monitor's "player still shown" test (the `grStadium_801D32D0` call in grStadium_801D2344's case 8) becomes "the
/// player's position is in x [-120, 120], y [-20, 80]" instead of "on screen". Armed from a replay's own Gecko list
/// (slpcheck, `golden --slp`) and for Slippi online play.
static PS_MONITOR: AtomicU32 = AtomicU32::new(0);
pub fn set_ps_camera_independent_monitor(on: bool) {
    PS_MONITOR.store(on as u32, Ordering::Relaxed);
}
#[inline]
pub fn ps_camera_independent_monitor() -> bool {
    PS_MONITOR.load(Ordering::Relaxed) != 0
}

/// Slippi's "Preload Stadium Transformations" (C2 0x801D45EC and five more inserts, slippi-ssbm-asm `Common/Preload
/// Stadium Transformations`, in the Slippi Recording core set): Pokemon Stadium decides its next transformation (one
/// HSD_Randi draw loop) when the normal-stage countdown starts instead of when it ends (grpstadium.rs). Armed from a
/// replay's own Gecko list (slpcheck, `golden --slp`).
static PS_PRELOAD: AtomicU32 = AtomicU32::new(0);
pub fn set_ps_preload_transformations(on: bool) {
    if on {
        PS_PRELOAD.fetch_or(1, Ordering::Relaxed);
    } else {
        PS_PRELOAD.fetch_and(!1, Ordering::Relaxed);
    }
}
#[inline]
pub fn ps_preload_transformations() -> bool {
    PS_PRELOAD.load(Ordering::Relaxed) & 1 != 0
}
/// The replay freezes Pokemon Stadium with IngameCheckIfFrozen (C2 0x801D457C), which returns from grStadium_801D4548
/// before its state machine; the older Frozen PS (04 0x801D45FC) only stops the countdown.
#[inline]
pub fn ps_freeze_at_entry() -> bool {
    PS_PRELOAD.load(Ordering::Relaxed) & 2 != 0
}
pub fn set_ps_freeze_at_entry(on: bool) {
    if on {
        PS_PRELOAD.fetch_or(2, Ordering::Relaxed);
    } else {
        PS_PRELOAD.fetch_and(!2, Ordering::Relaxed);
    }
}

/// A replay's widescreen camera code (melee_data::slp::widescreen_code), active only while that replay plays: set before
/// the match boots from the replay's own Gecko list (slpcheck, `golden --slp`), cleared (all 0) otherwise. Packed as
/// left | right << 16 in [`WS_BOUNDS`], the aspect ratio's numerator/denominator as f32 bits.
static WS_BOUNDS: AtomicU32 = AtomicU32::new(0);
static WS_NUM: AtomicU32 = AtomicU32::new(0);
static WS_DEN: AtomicU32 = AtomicU32::new(0);

/// Arm (Some((left, right, num, den))) or clear the replay's widescreen code.
pub fn set_replay_widescreen(ws: Option<(u16, u16, f32, f32)>) {
    let (b, n, d) = ws.map_or((0, 0, 0), |(l, r, n, d)| (l as u32 | (r as u32) << 16, n.to_bits(), d.to_bits()));
    WS_BOUNDS.store(b, Ordering::Relaxed);
    WS_NUM.store(n, Ordering::Relaxed);
    WS_DEN.store(d, Ordering::Relaxed);
}

/// The replay's widescreen code: (left, right, num, den), or None.
#[inline]
pub fn replay_widescreen() -> Option<(u16, u16, f32, f32)> {
    let b = WS_BOUNDS.load(Ordering::Relaxed);
    if b == 0 {
        return None;
    }
    Some((b as u16, (b >> 16) as u16, f32::from_bits(WS_NUM.load(Ordering::Relaxed)), f32::from_bits(WS_DEN.load(Ordering::Relaxed))))
}

/// The mounted disc's own 32-bit writes (codes.gct type 04) to retail game code, for the few a ported function checks
/// at the instruction they patch (an m-ex disc's codes.gct; ACE v2.0.1 nops two branches). Set once at mount from
/// the disc ([`set_disc_writes`]), constant afterwards, so rollback and replays see the same value.
static DISC_WRITES: std::sync::RwLock<Vec<(u32, u32)>> = std::sync::RwLock::new(Vec::new());

/// The 04 writes of a codes.gct image (`00D0C0DE00D0C0DE` header; 04 / C2 / 00 / 02 / 06 skipped as
/// tools/mex_port/dump_gct.py does). Stops at the terminator or the first unknown type.
pub fn gct_writes(gct: &[u8]) -> Vec<(u32, u32)> {
    let be = |o: usize| gct.get(o..o + 4).map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]));
    let mut v = Vec::new();
    if gct.get(..8) != Some(&[0x00, 0xD0, 0xC0, 0xDE, 0x00, 0xD0, 0xC0, 0xDE][..]) {
        return v;
    }
    let mut off = 8;
    while let (Some(w0), Some(w1)) = (be(off), be(off + 4)) {
        if w0 == 0xF000_0000 {
            break;
        }
        let addr = 0x8000_0000 | (w0 & 0x01FF_FFFF);
        match (w0 >> 24) & 0xFE {
            0xC2 => off += 8 + w1 as usize * 8,
            0x04 => {
                v.push((addr, w1));
                off += 8;
            }
            0x00 | 0x02 => off += 8,
            0x06 => off += 8 + ((w1 as usize + 7) & !7),
            _ => break,
        }
    }
    v
}

/// codes.gct 04 writes (ACE 2.0.1 / Akaneia) with a sim or data effect the port does not mirror yet;
/// each one present on the disc is logged at load. The ported ones are gated at their site through
/// `disc_writes` (ftswing, ftmewtwospecials, stale, ground Ground_801C24F8, gmMainLib defaults);
/// the m-ex table redirects are read natively by mex_host; the rest are UI or platform-only.
pub const UNHANDLED_WRITES: &[(u32, &str)] = &[
    (0x8002_A124, "Camera_ApplyQuake: 1P-mode check forced false"),
    (0x8002_B0F8, "Camera_8002B0E0: 1P-mode check forced false"),
    (0x8002_C6A8, "Camera_8002C5B4: stack stores removed"),
    (0x8006_AE00, "Fighter_procInput: debug-level branch removed"),
    (0x8006_AE90, "Fighter_procInput: 1P-mode check forced false"),
    (0x800C_09D0, "ftCo_800C09B4: +0x2324 < 30 check skipped"),
    (0x800C_0AB4, "ftCo_800C0A98: +0x2324 < 30 check skipped"),
    (0x8018_2FBC, "fn_80182F40: (r3&0xff)==3 path skipped"),
    (0x804D_C49C, "@2315: float 26.0 -> 29.0"),
    (0x8002_6EF0, "lbAudioAx_80026EBC: stage sound-bank mask table from MxDt"),
    (0x801A_3F24, "gm_801A3EF4: major-scene stride 0x18 (m-ex scene table)"),
    (0x801A_4560, "gm_801A4510: major-scene stride 0x18 (m-ex scene table)"),
    (0x8017_A890, "1P per-fighter table 0x803D7058 from MxDt"),
    (0x800C_2798, "per-fighter switch bounds for added kinds"),
    (0x800C_30A4, "per-fighter switch bounds for added kinds"),
    (0x8017_9DCC, "function entry overwritten (li r4,0)"),
];

/// Install the disc's 04 writes (empty for a vanilla disc).
pub fn set_disc_writes(w: Vec<(u32, u32)>) {
    for &(a, why) in UNHANDLED_WRITES {
        if let Some(&(_, v)) = w.iter().find(|&&(x, _)| x == a) {
            eprintln!("codes.gct: unhandled write {a:08X} = {v:08X} ({why})");
        }
    }
    if let Ok(mut d) = DISC_WRITES.write() {
        *d = w;
    }
}

/// The disc's codes.gct writes `value` to retail address `addr`.
pub fn disc_writes(addr: u32, value: u32) -> bool {
    DISC_WRITES.read().map_or(false, |d| d.iter().any(|&(a, v)| a == addr && v == value))
}

/// PowerPC `nop`, the value of a disc write that removes an instruction.
pub const PPC_NOP: u32 = 0x6000_0000;
