//! The PC platform layer as the fighter engine sees it (`pc/platform/*.c`, `pc/geno/*.c`): replay,
//! rollback/netplay, SyncTest snapshots, the m-ex feature registry and its per-kind hook tables,
//! the Geno define registry, Slippi codes, settings and the UCF raw-stick history.
//!
//! Every function here is the C function with the platform in its *default configuration*: no
//! replay playing or recording, no rollback session, no netplay, no SyncTest, no m-ex fighter and
//! no registered Geno definition. That is what the C returns in that configuration (each branch is
//! read off the C named in its doc comment), so a default build of the port behaves as the C does
//! when started without any of those features. The binary installs `hook::<name>` to attach the
//! real subsystem; an unset hook runs the default.
//!
//! State that the C keeps in static storage (the m-ex per-(event, kind) hook tables, the UCF
//! raw-stick ring) is native state here too, with the C's registration and dispatch rules.
#![allow(non_snake_case, static_mut_refs, clippy::missing_safety_doc)]

use crate::types::*;
use core::ffi::c_void;

type G = *mut HSD_GObj;
type F = *mut Fighter;

macro_rules! plat {
    ($($(#[$m:meta])* $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? = $def:expr;)*) => {
        /// Overrides installed by the platform (`None` = the default configuration below).
        pub mod hook {
            #[allow(unused_imports)] use super::*;
            $( #[allow(non_upper_case_globals)] pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
        }
        $(
            $(#[$m])*
            #[allow(unused_variables)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } { Some(f) => f($($a),*), None => $def }
            }
        )*
    };
}

// ---- settings (pc/platform/gw_settings.c) -------------------------------------------------------

/// `st_env_value`: `MELEE_<KEY>` (upper-cased), never for `fc_*` keys.
fn settings_env(key: &str) -> Option<String> {
    if key.starts_with("fc_") || key.is_empty() || key.len() >= 48 {
        return None;
    }
    let v = std::env::var(format!("MELEE_{}", key.to_ascii_uppercase())).ok()?;
    if v.is_empty() { None } else { Some(v) }
}

/// C `atoi`: optional whitespace and sign, then digits; anything else stops it.
pub fn atoi(s: &str) -> i32 {
    let t = s.trim_start();
    let (neg, t) = match t.as_bytes().first() {
        Some(b'-') => (true, &t[1..]),
        Some(b'+') => (false, &t[1..]),
        _ => (false, t),
    };
    let mut n: i32 = 0;
    for c in t.bytes() {
        if !c.is_ascii_digit() { break; }
        n = n.wrapping_mul(10).wrapping_add((c - b'0') as i32);
    }
    if neg { n.wrapping_neg() } else { n }
}

// ---- UCF raw-stick history (pc/platform/gw_replay.c: gw_Ucf_PadLatch, ucf_pad_back) ----------------

static mut UCF_HIST: [[[i8; 4]; 8]; 4] = [[[0; 4]; 8]; 4];
static mut UCF_HEAD: [u32; 4] = [0; 4];

/// `gw_Ucf_PadLatch`: one poll's raw stick bytes for a port (called by the pad reader).
pub fn Ucf_PadLatch(port: i32, sx: i32, sy: i32, cx: i32, cy: i32) {
    if !(0..=3).contains(&port) { return; }
    unsafe {
        let p = port as usize;
        UCF_HEAD[p] = UCF_HEAD[p].wrapping_add(1);
        UCF_HIST[p][(UCF_HEAD[p] & 7) as usize] = [sx as i8, sy as i8, cx as i8, cy as i8];
    }
}

/// `ucf_pad_back`.
fn ucf_pad_back(port: i32, which: i32, back: i32) -> i32 {
    if !(0..=3).contains(&port) || !(0..=3).contains(&which) || !(0..=7).contains(&back) { return 0; }
    unsafe {
        let p = port as usize;
        UCF_HIST[p][(UCF_HEAD[p].wrapping_sub(back as u32) & 7) as usize][which as usize] as i32
    }
}

static mut UCF_OFF_MODE: i32 = -1;
/// `gw_Ucf_Reload`.
pub fn Ucf_Reload() { unsafe { UCF_OFF_MODE = -1 } }
/// `gw_Ucf_OfflineMode`: 84 (UCF 0.84) unless `MELEE_UCF` says off or the `ucf` setting is 0.
pub fn Ucf_OfflineMode() -> i32 {
    unsafe {
        if UCF_OFF_MODE < 0 {
            // `--codes` given: the UCF code bit decides (notes/gameplay-codes.md); else the C build's setting
            UCF_OFF_MODE = if melee_core::codes::configured() { if melee_core::codes::on(melee_core::codes::UCF) { 84 } else { 0 } } else { match std::env::var("MELEE_UCF") {
                Ok(v) if !v.is_empty() => if matches!(v.as_str(), "0" | "off" | "OFF" | "false" | "no") { 0 } else { 84 },
                _ => if Settings_Int("ucf", 84) != 0 { 84 } else { 0 },
            } };
        }
        UCF_OFF_MODE
    }
}

// ---- m-ex feature registry and hook tables (pc/platform/gw_runtime.c) ------------------------------

pub const MEX_EVENT_ON_LOAD: usize = 0;
pub const MEX_EVENT_ON_DEATH: usize = 1;
pub const MEX_EVENT_ON_DESTROY: usize = 2;
pub const MEX_EVENT_ON_FRAME: usize = 3;
pub const MEX_EVENT_ON_ABSORB: usize = 4;
pub const MEX_EVENT_SPECIAL_N: usize = 13;
pub const MEX_EVENT_SPECIAL_N_AIR: usize = 14;
pub const MEX_EVENT_SPECIAL_S: usize = 15;
pub const MEX_EVENT_SPECIAL_S_AIR: usize = 16;
pub const MEX_EVENT_SPECIAL_HI: usize = 17;
pub const MEX_EVENT_SPECIAL_HI_AIR: usize = 18;
pub const MEX_EVENT_SPECIAL_LW: usize = 19;
pub const MEX_EVENT_SPECIAL_LW_AIR: usize = 20;
pub const MEX_EVENT_ON_DOUBLE_JUMP: usize = 22;
pub const MEX_EVENT_ON_USMASH: usize = 23;
pub const MEX_EVENT_ON_ITEM_PICKUP: usize = 24;
pub const MEX_EVENT_ON_LANDING: usize = 33;
pub const MEX_EVENT_ON_FSMASH: usize = 34;
pub const MEX_EVENT_ON_TAUNT: usize = 36;
pub const MEX_EVENT_ON_CATCH: usize = 37;
pub const MEX_EVENT_ON_DSMASH: usize = 38;
pub const MEX_EVENT_COUNT: usize = 40;
/// `GW_MEX_KIND_MAX` = `0x21 + GW_MEX_SLOT_COUNT` (retail kinds + m-ex slots); the tables are sized to
/// the engine's `Ft_Kind_Max`.
pub const MEX_KIND_MAX: usize = Ft_Kind_Max as usize;

type GFn = unsafe extern "C" fn(G);
type GFn2 = unsafe extern "C" fn(G, *mut c_void);

static mut MEX_HOOKS: [[Option<GFn>; MEX_KIND_MAX]; MEX_EVENT_COUNT] = [[None; MEX_KIND_MAX]; MEX_EVENT_COUNT];
static mut MEX_HOOKS2: [[Option<GFn2>; MEX_KIND_MAX]; MEX_EVENT_COUNT] = [[None; MEX_KIND_MAX]; MEX_EVENT_COUNT];
static mut MEX_HOOK_ACTIVE: [[bool; MEX_KIND_MAX]; MEX_EVENT_COUNT] = [[false; MEX_KIND_MAX]; MEX_EVENT_COUNT];

/// Registers (or with `None` clears) the one-argument override of `event` for `kind`; last
/// registration wins, as in m-ex.
pub fn Mex_SetHook(event: usize, kind: usize, f: Option<GFn>) {
    if event < MEX_EVENT_COUNT && kind < MEX_KIND_MAX { unsafe { MEX_HOOKS[event][kind] = f } }
}
/// Two-argument override (`OnItemPickup`).
pub fn Mex_SetHook2(event: usize, kind: usize, f: Option<GFn2>) {
    if event < MEX_EVENT_COUNT && kind < MEX_KIND_MAX { unsafe { MEX_HOOKS2[event][kind] = f } }
}

/// `gw_Mex_HasHook`.
pub fn Mex_HasHook(event: i32, kind: i32) -> bool {
    if event < 0 || event as usize >= MEX_EVENT_COUNT || kind < 0 || kind as usize >= MEX_KIND_MAX { return false; }
    let (e, k) = (event as usize, kind as usize);
    unsafe { MEX_HOOKS[e][k].is_some() || MEX_HOOKS2[e][k].is_some() }
}

/// `gw_Mex_GObjDispatch`: the registered override (not re-entered while it runs), else `vanilla`.
pub unsafe fn Mex_GObjDispatch(event: usize, kind: i32, g: G, vanilla: Option<GFn>) {
    let in_range = event < MEX_EVENT_COUNT && kind >= 0 && (kind as usize) < MEX_KIND_MAX;
    let mut f = None;
    if in_range && !MEX_HOOK_ACTIVE[event][kind as usize] {
        f = MEX_HOOKS[event][kind as usize];
    }
    match f {
        Some(f) => {
            MEX_HOOK_ACTIVE[event][kind as usize] = true;
            f(g);
            MEX_HOOK_ACTIVE[event][kind as usize] = false;
        }
        None => {
            if let Some(v) = vanilla { v(g) }
        }
    }
}

/// `gw_Mex_GObjDispatch2`.
pub unsafe fn Mex_GObjDispatch2(event: usize, kind: i32, g: G, arg1: *mut c_void, vanilla: Option<GFn2>) {
    let in_range = event < MEX_EVENT_COUNT && kind >= 0 && (kind as usize) < MEX_KIND_MAX;
    let mut f = None;
    if in_range && !MEX_HOOK_ACTIVE[event][kind as usize] {
        f = MEX_HOOKS2[event][kind as usize];
    }
    match f {
        Some(f) => {
            MEX_HOOK_ACTIVE[event][kind as usize] = true;
            f(g, arg1);
            MEX_HOOK_ACTIVE[event][kind as usize] = false;
        }
        None => {
            if let Some(v) = vanilla { v(g, arg1) }
        }
    }
}

/// A `void*` vanilla pointer from the C call sites (`(void*) ftData_X[kind]`) as the callback.
#[inline]
pub unsafe fn as_gfn(p: *mut c_void) -> Option<GFn> {
    if p.is_null() { None } else { Some(core::mem::transmute::<*mut c_void, GFn>(p)) }
}
#[inline]
pub unsafe fn as_gfn2(p: *mut c_void) -> Option<GFn2> {
    if p.is_null() { None } else { Some(core::mem::transmute::<*mut c_void, GFn2>(p)) }
}

/// `Mex_On*Dispatch(kind, gobj, vanilla)` / `Mex_Special*Dispatch(...)` with a `void*` vanilla.
pub fn mex_dispatch(event: usize, kind: i32, g: G, vanilla: *mut c_void) {
    unsafe { Mex_GObjDispatch(event, kind, g, as_gfn(vanilla)) }
}
/// Same with a typed `HSD_GObjEvent` vanilla.
pub fn mex_dispatch_ev(event: usize, kind: i32, g: G, vanilla: HSD_GObjEvent) {
    unsafe { Mex_GObjDispatch(event, kind, g, vanilla) }
}
/// `gw_Mex_OnItemPickupDispatch(kind, gobj, arg1, vanilla)`.
pub fn mex_dispatch2(event: usize, kind: i32, g: G, arg1: *mut c_void, vanilla: *mut c_void) {
    unsafe { Mex_GObjDispatch2(event, kind, g, arg1, as_gfn2(vanilla)) }
}

/// The engine's per-kind vanilla callback tables (`ftData_OnLoad[kind]` ...), for dispatchers whose
/// Rust call site folds the vanilla table in (`Mex_OnLoadDispatch(kind, gobj)`).
fn vanilla_of(table: &[HSD_GObjEvent; MEX_KIND_MAX], kind: i32) -> HSD_GObjEvent {
    if (0..MEX_KIND_MAX as i32).contains(&kind) { table[kind as usize] } else { None }
}
pub fn Mex_OnLoadDispatch(kind: i32, g: G) {
    unsafe { Mex_GObjDispatch(MEX_EVENT_ON_LOAD, kind, g, vanilla_of(&crate::tu::ftdata::ftData_OnLoad, kind)) }
}
pub fn Mex_OnDeathDispatch(kind: i32, g: G) {
    unsafe { Mex_GObjDispatch(MEX_EVENT_ON_DEATH, kind, g, vanilla_of(&crate::tu::ftdata::ftData_OnDeath, kind)) }
}
pub fn Mex_OnLandingDispatch(kind: i32, g: G) {
    unsafe { Mex_GObjDispatch(MEX_EVENT_ON_LANDING, kind, g, None) }
}
pub fn Mex_OnFrameDispatch(kind: i32, g: G) {
    unsafe { Mex_GObjDispatch(MEX_EVENT_ON_FRAME, kind, g, vanilla_of(&crate::tu::ftdata::ftData_UnkMotionStates3, kind)) }
}
pub fn Mex_OnAbsorbDispatch(kind: i32, g: G) {
    unsafe { Mex_GObjDispatch(MEX_EVENT_ON_ABSORB, kind, g, vanilla_of(&crate::tu::ftdata::ftData_OnAbsorb, kind)) }
}
pub fn Mex_OnDestroyDispatch(kind: i32, g: G) {
    unsafe { Mex_GObjDispatch(MEX_EVENT_ON_DESTROY, kind, g, vanilla_of(&crate::tu::ftdata::ftData_OnUserDataRemove, kind)) }
}
/// `gw_Mex_FighterCallbackDispatch(gobj, cb)`: a native callback is called directly (guest-address
/// callbacks are the m-ex interpreter's, which installs `hook::Mex_FighterCallbackDispatch`).
pub fn Mex_FighterCallbackDispatch_default(g: G, cb: HSD_GObjEvent) {
    if let Some(f) = cb { unsafe { f(g) } }
}

/// `gw_Mex_Enabled`: `MELEE_MEX` (comma or space separated, case-insensitive). The `mods\mex.txt`
/// file the C also reads is the platform's; it installs `hook::Mex_Enabled`.
pub fn Mex_Enabled_default(name: &str) -> bool {
    static mut FEATURES: Option<std::vec::Vec<String>> = None;
    unsafe {
        if FEATURES.is_none() {
            let v = std::env::var("MELEE_MEX").unwrap_or_default();
            FEATURES = Some(v.split(|c: char| c == ',' || c.is_whitespace()).filter(|s| !s.is_empty()).map(|s| s.to_string()).collect());
        }
        FEATURES.as_ref().unwrap().iter().any(|f| f.eq_ignore_ascii_case(name))
    }
}

// ---- DbLevel (db/dbinit.c) ----------------------------------------------------------------------------

pub const DBLKIND_MASTER: i32 = 0;
pub const DBLKIND_NO_DEBUG_ROM: i32 = 1;
pub const DBLKIND_DEBUG_DEVELOP: i32 = 2;
pub const DBLKIND_DEBUG_ROM: i32 = 3;
pub const DBLKIND_DEVELOP: i32 = 4;
/// `DbLevel` (0x804D4A08); `dbinit.c` initialises it to `DbLKind_NoDebugRom`.
pub static mut DB_LEVEL: i32 = DBLKIND_NO_DEBUG_ROM;

// ---- the platform functions ---------------------------------------------------------------------------

plat! {
    // ---- gw_replay.c ----
    Replay_Active() -> i32 = 0;
    Replay_HasInput(port: i32, fol: i32) -> i32 = 0;
    Replay_StickX(port: i32, fol: i32) -> f32 = 0.0;
    Replay_StickY(port: i32, fol: i32) -> f32 = 0.0;
    Replay_CStickX(port: i32, fol: i32) -> f32 = 0.0;
    Replay_CStickY(port: i32, fol: i32) -> f32 = 0.0;
    Replay_Trigger(port: i32, fol: i32) -> f32 = 0.0;
    Replay_Buttons(port: i32, fol: i32) -> u32 = 0;
    Replay_RecordInput(port: i32, fol: i32, lx: f32, ly: f32, cx: f32, cy: f32, t: f32, b: u32, action: i32, x: f32, y: f32, facing: f32, pct: f32) = ();
    Replay_Tracing() -> i32 = 0;
    Replay_TraceFighter(port: i32, fol: i32, ckind: i32, action: i32, x: f32, y: f32, facing: f32, pct: f32, stocks: i32, ax: f32, ay: f32, kx: f32, ky: f32, gx: f32) = ();
    Replay_TraceAtProcMap() -> i32 = 0;
    /// `rp.live || gw_rb_slippi_local_port() >= 0` is false: the arrived seed stands.
    Replay_ChecksumSeed(arrived: u32, frame: i32) -> u32 = arrived;
    Replay_UcfCardinals(port: i32) -> i32 = ((0..=3).contains(&port) && Ucf_OfflineMode() == 84) as i32;
    Replay_UcfVersion(port: i32) -> i32 = if (0..=3).contains(&port) { Ucf_OfflineMode() } else { 0 };
    Replay_RawStick(port: i32, which: i32) -> i32 = ucf_pad_back(port, which, 0);
    Replay_RawStickBack(port: i32, which: i32, back: i32) -> i32 = ucf_pad_back(port, which, back);
    /// `rp.live` is false and `Replay_Active()` is false: `MELEE_SLIPPI_CODES` decides (`o` online = 2,
    /// `t`/`T`/`c` tournament = 1, anything else 0), else 0.
    Slippi_Codes() -> i32 = match std::env::var("MELEE_SLIPPI_CODES") {
        Ok(v) if !v.is_empty() => { let c = v.as_bytes()[0]; if c == b'o' || c == b'O' { 2 } else if c == b't' || c == b'T' || c == b'c' { 1 } else { 0 } }
        _ => 0,
    };
    // ---- gw_snap.c (SyncTest) ----
    Snap_Curated() -> i32 = 0;
    Snap_CuratedMix(w: *const u32, n: i32) = ();
    Snap_CuratedRegion(tag: i32, va: usize, len: u32) = ();
    Snap_InputHook(port: i32, fol: i32, v: *mut f32, b: *mut u32) -> i32 = 0;
    /// `gw_Snap_XRegion` records hash regions for SyncTest diagnostics only.
    Snap_XRegion(tag: i32, va: usize, len: u32) = ();
    // ---- gw_rollback.c / gw_net.c ----
    /// `rb_hash_capture == NULL`: nothing is capturing.
    RbHashWord(value: u32, symbol: &str) = ();
    RbHashRegion(region: i32, slot: i32) = ();
    RbHashSeed(arrived: u32, effective: u32) = ();
    RbHashLegacyEnv() -> i32 = std::env::var("MELEE_RB_HASH_LEGACY").map(|e| e.starts_with('1') as i32).unwrap_or(0);
    RB_Enabled() -> i32 = 0;
    Netplay_Enabled() -> i32 = 0;
    // ---- gw_runtime.c / gw_script.c / gw_settings.c ----
    PcTraceMotionEnabled() -> i32 = std::env::var("MELEE_LOG_MOTION").map(|v| (v.as_bytes().first().copied().unwrap_or(0) != b'0') as i32).unwrap_or(0);
    /// No script runtime attached: events have no listener.
    Script_GameEvent(what: i32, a: i32, b: i32, c: i32, d: i32) = ();
    /// `gw_Settings_Int`: the `MELEE_<KEY>` override, else the stored value (none loaded), else `d`.
    Settings_Int(key: &str, d: i32) -> i32 = match settings_env(key) { Some(v) => atoi(&v), None => d };
    FighterControls_TapJumpOff(port: i32) -> i32 = 0;
    diag_geno_stun(port: i32, base: i32, bonus: i32) = ();
    Camera_ScriptState() -> i32 = 0;
    // ---- DbLevel ----
    DbLevel() -> i32 = unsafe { DB_LEVEL };
    DbLevel_below_DebugRom() -> bool = unsafe { DB_LEVEL } < DBLKIND_DEBUG_ROM;
    DbLevel_at_least_DebugRom() -> bool = unsafe { DB_LEVEL } >= DBLKIND_DEBUG_ROM;
    // ---- m-ex ----
    Mex_Enabled(name: &str) -> bool = Mex_Enabled_default(name);
    Mex_HasFSmashHook(kind: i32) -> bool = Mex_HasHook(MEX_EVENT_ON_FSMASH as i32, kind);
    Mex_FighterCallbackDispatch(g: G, cb: HSD_GObjEvent) = Mex_FighterCallbackDispatch_default(g, cb);
    /// `gw_Mex_InternalForPortKind` with no m-ex slot installed: retail kinds 0..=26 map to
    /// themselves, the six bosses 27..=32 to the last six internal ids (`n - 6 + (fk - 27)`, with
    /// the retail count `n = 33` that is the identity); other kinds have no slot (-1).
    Mex_InternalForPortKind(k: i32) -> i32 = if (0..=32).contains(&k) { k } else { -1 };
    Mex_FtSynth() -> i32 = 0;
    /// `!gw_mex_ft_on()` -> 0.
    Mex_FtBaseKind(k: i32) -> i32 = 0;
    Mex_FtPartsTable(k: i32) -> usize = 0;
    Mex_FtTrailData(g: G) -> *mut c_void = core::ptr::null_mut();
    /// `gw_Mex_MoveLogicTable(kind, vanilla)` with no installed MoveLogic table: the vanilla one.
    Mex_MoveLogicTable(kind: i32) -> *mut MotionState = if (0..MEX_KIND_MAX as i32).contains(&kind) { unsafe { crate::tu::ftdata::ftData_CharacterStateTables[kind as usize] } } else { core::ptr::null_mut() };
    // ---- Geno define registry with no registered definition ----
    Geno_DefineBaseKind(kind: i32) -> i32 = -1;
    Geno_DefineIsNone(kind: i32) -> i32 = 0;
    Geno_DefineThrownAs(kind: i32) -> i32 = -1;
    Geno_DefineThrownAsRow(kind: i32, row: i32) -> i32 = -1;
    Geno_DefineAiLike(kind: i32) -> i32 = -1;
    Geno_DefineTemplateSkeleton(kind: i32) -> i32 = 0;
    Geno_DefineKirbyCopy(kind: i32) -> i32 = -2;
    Geno_ToonUnlit(kind: i32) -> i32 = 0;
    GenoDefine_PartsTable(kind: i32) -> *mut FighterPartsTable = crate::tu::ftdata::GenoDefine_PartsTable(kind);
    GenoDefine_RestPoseTree(fp: &mut Fighter) -> *mut HSD_Joint = crate::tu::ftdata::GenoDefine_RestPoseTree(fp);
    GenoDefine_BindFighter(fp: F) = unsafe { crate::tu::ftdata::GenoDefine_BindFighter(&mut *fp) };
    GenoDefine_InitKinds() = crate::tu::ftdata::GenoDefine_InitKinds();
    Geno_AnimBankBeforeSpawn() = ();
    Geno_FighterScale(fp: F) -> f32 = 1.0;
}
