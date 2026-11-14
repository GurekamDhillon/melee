//! The WebAssembly mod host (mod-c-v1 ABI 1.0, hook table hooks-v1): sandboxed code mods for the native engine.
//!
//! Notes: docs/rust-port/notes/modhost.md. Spec: docs/specs/mod-c-v1.md (declarations: docs/specs/gdmod.h).
//!
//! * Off by default. `MELEE_WASM_MODS=<dir>` loads every `<dir>/*.wasm` (chains ordered by mod id, so the order on
//!   disk never matters). Unset, empty or `0`: nothing is loaded and no hook pointer is installed, so the game runs
//!   exactly as before (identical hashes by construction: every hook site is a `None` check).
//! * Runtime: wasmi (an interpreter in pure Rust) with its `deterministic` profile (NaN canonicalisation), fuel
//!   metering (a deterministic watchdog), no SIMD, no threads, fixed memory, no start function, imports only from `gd`.
//! * Rollback: the host (wasmi engine, store, instances, linear memories) is created inside the engine's
//!   `GameScope`, so every byte of it lives in the game arena / statics that `melee_engine::snapshot` saves and loads.
//! * Hooks: `frame.run_procs` and `damage.fighter_take_damage` have their own sites; every other row in [`WIRED`] is a
//!   generic `melee_core::mod_hook!` site dispatched by [`dispatch`]. A mod that declares an unwired row is refused.
#![allow(static_mut_refs)]

use core::ffi::c_void;
use melee_core::modhook::{Arg, Ret};
use melee_types::{Fighter, HSD_GObj, Item, SpawnItem};
use std::collections::BTreeMap;
use wasmi::{AsContextMut, Caller, Config, Engine, Extern, Func, FuncType, Linker, Memory, Module, Store, StoreContextMut, Val, ValType};

pub mod lab;
pub mod pres;
pub mod ui;

pub const ABI_VERSION: u32 = 1 << 16; // 1.0
pub const HOOK_TABLE_VERSION: u32 = 1;
pub const MANIFEST_MAGIC: u32 = 0x444D_4447; // 'GDMD'
pub const MANIFEST_SIZE: u32 = 124; // gd_manifest on wasm32
pub const MAX_MODS: usize = 16;
pub const MAX_HOOKS: u32 = 256;
pub const MAX_PAGES: u64 = 16; // 1 MiB
/// Fuel per hook invocation (wasmi fuel units, about one per instruction). Running out traps and disables the mod.
pub const HOOK_FUEL: u64 = 1_000_000;
/// Activations of one address deeper than this run the original only (spec section 5, re-entrancy).
pub const MAX_DEPTH: u32 = 8;
/// Hooks per address across all mods (spec limits): pre and post each.
pub const MAX_PER_ADDR: usize = 8;

pub const FEATURE_ITEM_WRITE: u32 = 1 << 0;
pub const FEATURE_ACTION_REQUEST: u32 = 1 << 1;
pub const FEATURE_REPLACE: u32 = 1 << 2;
pub const FEATURE_LOG: u32 = 1 << 3;
const FEATURES_HOST: u32 = FEATURE_ITEM_WRITE | FEATURE_ACTION_REQUEST | FEATURE_REPLACE | FEATURE_LOG;

pub const PRE: u32 = 1;
pub const POST: u32 = 2;
pub const REPLACE: u32 = 3;

const ADDR_FRAME: u32 = 0x8039_0CFC;
const ADDR_DAMAGE: u32 = 0x8006_CC7C;

/// hooks-v1 rows this host implements: (name, address). Each generic row has a `mod_hook!` site in the game crates.
pub const WIRED: &[(&str, u32)] = &[
    ("frame.run_procs", ADDR_FRAME),
    ("damage.fighter_take_damage", ADDR_DAMAGE),
    ("fighter.proc00", 0x8006_A1BC),
    ("fighter.proc01", 0x8006_A360),
    ("fighter.proc04", 0x8006_B82C),
    ("fighter.proc06", 0x8006_C27C),
    ("action.HammerLanding.enter", 0x800C_5F88),
    ("action.DownSpot.enter", 0x800C_8B74),
    ("action.Barrel.enter", 0x800C_9078),
    ("action.Walk.enter", 0x800C_9528),
    ("action.Turn.enter", 0x800C_9840),
    ("action.TurnRun.enter", 0x800C_9D94),
    ("action.Dash.enter", 0x800C_A120),
    ("action.Run.enter", 0x800C_A6F4),
    ("action.RunBrake.enter", 0x800C_AC18),
    ("action.Jump.enter", 0x800C_B250),
    ("action.KneeBend.enter", 0x800C_B4E0),
    ("action.Fall.enter", 0x800C_C730),
    ("action.FallAerial.enter", 0x800C_CDA8),
    ("action.ItemScrew.enter", 0x800D_2D50),
    ("action.Landing.enter", 0x800D_5AEC),
    ("action.LandingFallSpecial.enter", 0x800D_5CB0),
    ("action.Squat.enter", 0x800D_600C),
    ("action.SquatRv.enter", 0x800D_6620),
    ("action.CaptureCut.enter", 0x800D_C750),
    ("item.spawn", 0x8026_862C),
    ("item.spawn_gobj", 0x8026_8B18),
    ("item.spawn_gobj2", 0x8026_8B5C),
    ("item.spawn_void", 0x8026_8B9C),
    ("item.destroy", 0x8026_80CC),
    ("item.change_state", 0x8026_8E5C),
    ("item.update0", 0x8026_93E4),
    ("item.update5", 0x8026_9978),
    ("stage.update", 0x801C_0C2C),
    ("stage.on_start", 0x801C_0FB8),
];

const HOOKS_V1: &str = include_str!("../../../docs/specs/hooks-v1.tsv");

/// hooks-v1 row `addr` -> (name, signature, sig_src), None when the address is not in the table.
pub fn hook_row(addr: u32) -> Option<(&'static str, &'static str, &'static str)> {
    HOOKS_V1.lines().filter(|l| !l.starts_with('#') && !l.starts_with("name\t")).find_map(|l| {
        let c: Vec<&str> = l.split('\t').collect();
        let a = u32::from_str_radix(c.get(1)?.trim_start_matches("0x"), 16).ok()?;
        (a == addr).then(|| (c[0], c.get(2).copied().unwrap_or(""), c.get(7).copied().unwrap_or("")))
    })
}

/// An ABI type of a row's signature.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Ty {
    I32,
    F32,
    /// gd_fighter / gd_item / gd_gobj / gd_spawn / gd_ptr: an i32 handle
    Handle,
}

#[derive(Clone, Debug)]
struct Sig {
    args: Vec<Ty>,
    ret: Option<Ty>,
}

fn parse_sig(s: &str) -> Option<Sig> {
    let (r, rest) = s.split_once('(')?;
    let ty = |t: &str| match t.trim() {
        "f32" => Some(Ty::F32),
        "i32" | "u32" | "gd_bool" => Some(Ty::I32),
        t if t.starts_with("gd_") => Some(Ty::Handle),
        _ => None,
    };
    let args = rest.trim_end_matches(')');
    let args = if args.trim().is_empty() { Vec::new() } else { args.split(',').map(ty).collect::<Option<Vec<_>>>()? };
    let ret = if r.trim() == "void" { None } else { Some(ty(r)?) };
    Some(Sig { args, ret })
}

fn vt(t: Ty) -> ValType {
    if t == Ty::F32 { ValType::F32 } else { ValType::I32 }
}

/// The wasm function type a hook of `mode` must have for `sig` (spec section 5).
fn func_type(sig: &Sig, mode: u32) -> FuncType {
    let mut p: Vec<ValType> = sig.args.iter().map(|t| vt(*t)).collect();
    let r: Vec<ValType> = match mode {
        PRE => vec![ValType::I32],
        REPLACE => sig.ret.map(vt).into_iter().collect(),
        _ => {
            if let Some(t) = sig.ret { p.push(vt(t)); }
            sig.ret.map(vt).into_iter().collect()
        }
    };
    FuncType::new(p, r)
}

/// One running replace hook's original (for `gd_orig_call_*`): erased-lifetime pointer, valid while the hook runs.
struct OrigFrame {
    f: *mut (dyn FnMut() -> Ret + 'static),
    used: bool,
}

/// Per-store data the accessors see.
#[derive(Default)]
pub struct HostData {
    /// `gd_match_frame`: frame.begin calls since the host was loaded (rolls back with the arena).
    pub frame: i32,
    /// Messages from `gd_log` (drained to stderr after each hook).
    pub log: Vec<String>,
    /// Host accessor calls with a stale or foreign handle (spec section 3.3: counted, not an error).
    pub bad_handles: u32,
    /// Activations past [`MAX_DEPTH`] that ran the original only.
    pub deep_calls: u32,
    /// The module whose hook is running (feature checks), and each module's declared features.
    cur_module: usize,
    features: Vec<u32>,
    /// Handle tables for items and other gobjs: valid for the logic frame they were issued in (cleared at frame begin).
    items: Vec<usize>,
    gobjs: Vec<usize>,
    /// `SpawnItem*` of the item.spawn* activations in progress, with whether a pre hook is running.
    spawns: Vec<(usize, bool)>,
    /// `gd_set_return_*` of the current pre chain.
    set_return: Option<Val>,
    /// Queued `gd_fighter_request_action_state` (handle, state), applied after the chain.
    requests: Vec<(u32, i32)>,
    /// Queued `gd_stage_switch` ground kind, applied after the `frame.run_procs` post chain (outside the procs).
    stage_switch: Option<i32>,
    /// The result of the last applied switch (`gd_stage_switch_result`): 0 done, < 0 refused, 1 pending.
    stage_switch_rc: i32,
    /// `gd_hash_word`: this logic frame's mod words folded together (cleared at frame begin; in the store, so it rolls
    /// back with the arena), and whether any mod gave one.
    hash_acc: u32,
    hash_any: bool,
    origs: Vec<OrigFrame>,
    depth: BTreeMap<u32, u32>,
}

struct Hook {
    addr: u32,
    mode: u32,
    priority: i32,
    index: u32,
    func: Func,
}

pub struct LoadedMod {
    pub id: String,
    pub name: String,
    pub version: u32,
    /// FNV-1a 64 of the module bytes (the session identity until OPEN 10 settles on sha-256).
    pub digest: u64,
    pub disabled: bool,
    pub features: u32,
    hooks: Vec<Hook>,
    /// The frame profiler's slot for this mod's time (melee_core::frameprof).
    prof: u8,
}

#[derive(Clone, Copy)]
struct Entry {
    module: usize,
    func: Func,
}

#[derive(Clone)]
struct Chain {
    sig: Sig,
    pre: Vec<Entry>,
    replace: Option<Entry>,
    post: Vec<Entry>,
}

pub struct Host {
    store: Store<HostData>,
    pub mods: Vec<LoadedMod>,
    chains: BTreeMap<u32, Chain>,
}

static mut HOST: Option<Box<Host>> = None;
/// Callers of the host functions currently running game code (`gd_orig_call_*`): a hook reached from there runs in
/// that context (wasmi re-entrancy through `Caller`), never through a second borrow of the store.
static mut CTX: Vec<*mut c_void> = Vec::new();

pub fn engine_config() -> Config {
    let mut c = Config::default();
    c.consume_fuel(true);
    c.wasm_multi_memory(false);
    // translate every function at load, not on its first call (wasmi's default, LazyTranslation): the first Envoy
    // match's run setup translated ~6 ms of envoy_sim code inside the fight's first tick (notes/zero-hitch.md)
    c.compilation_mode(wasmi::CompilationMode::Eager);
    c
}

fn fnv64(b: &[u8]) -> u64 {
    b.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &x| (h ^ x as u64).wrapping_mul(0x100_0000_01b3))
}

/// True when the binary has a start section (id 8): mod-c-v1 refuses it.
fn has_start_section(b: &[u8]) -> bool {
    if b.len() < 8 {
        return false;
    }
    let mut p = 8;
    while p < b.len() {
        let id = b[p];
        p += 1;
        let (mut len, mut shift) = (0usize, 0);
        loop {
            let Some(&x) = b.get(p) else { return false };
            p += 1;
            len |= ((x & 0x7f) as usize) << shift;
            shift += 7;
            if x & 0x80 == 0 || shift > 28 {
                break;
            }
        }
        if id == 8 {
            return true;
        }
        p = p.saturating_add(len);
    }
    false
}

/// A short UTF-8 name from mod memory (at most 64 bytes).
fn mod_str(c: &Caller<'_, HostData>, ptr: u32, n: u32) -> Option<String> {
    if n == 0 || n > 64 {
        return None;
    }
    let m = mem_of(c)?;
    let b = m.data(c).get(ptr as usize..ptr as usize + n as usize)?;
    std::str::from_utf8(b).ok().map(str::to_string)
}

fn mem_of(caller: &Caller<'_, HostData>) -> Option<Memory> {
    caller.get_export("memory").and_then(Extern::into_memory)
}

/// Floats a mod writes or reads are NaN-canonicalised at the boundary (spec section 4).
fn canon(x: f32) -> f32 {
    if x.is_nan() { f32::from_bits(0x7FC0_0000) } else { x }
}

// ---- handles ------------------------------------------------------------------------------------------------
// (kind << 16) | (index + 1). Fighters: kind 1, index = slot, re-resolved through Player_GetEntity on every call.
// Items: kind 2, gobjs: kind 3, index into a per-frame table (an item handle is valid for the logic frame it was
// issued in and while the item is alive). Spawn requests: kind 4, the innermost item.spawn* activation.
const H_FIGHTER: u32 = 1;
const H_ITEM: u32 = 2;
const H_GOBJ: u32 = 3;
const H_SPAWN: u32 = 4;

fn handle_of(slot: i32) -> u32 {
    (H_FIGHTER << 16) | (slot as u32 + 1)
}

fn fighter_ptr(h: u32) -> *mut Fighter {
    if h >> 16 != H_FIGHTER {
        return core::ptr::null_mut();
    }
    let slot = (h & 0xFFFF) as i32 - 1;
    if !(0..4).contains(&slot) {
        return core::ptr::null_mut();
    }
    let g = melee_fighter::player::get_entity(slot);
    if g.is_null() {
        return core::ptr::null_mut();
    }
    unsafe { (*g).user_data as *mut Fighter }
}

/// The handle of a fighter GObj; 0 for a sub-fighter (Nana is not addressable in v1) or a non-fighter.
fn fighter_handle_of_gobj(g: *mut HSD_GObj) -> u32 {
    if g.is_null() {
        return 0;
    }
    unsafe {
        if (*g).classifier as i32 != melee_types::HSD_GOBJ_CLASS_FIGHTER || (*g).user_data.is_null() {
            return 0;
        }
        let fp = &*((*g).user_data as *const Fighter);
        if fp.is_sub_fighter() != 0 || !(0..4).contains(&(fp.player_idx as i32)) {
            return 0;
        }
        handle_of(fp.player_idx as i32)
    }
}

fn item_list() -> impl Iterator<Item = *mut HSD_GObj> {
    let mut g = unsafe { *melee_runtime::tu::gobj_native::HSD_GObjPLinkHead.add(melee_types::HSD_GOBJ_PLINK_ITEM as usize) };
    core::iter::from_fn(move || {
        if g.is_null() {
            return None;
        }
        let r = g;
        g = unsafe { (*g).next };
        Some(r)
    })
}

fn table_handle(t: &mut Vec<usize>, kind: u32, p: *mut HSD_GObj) -> u32 {
    if p.is_null() {
        return 0;
    }
    let i = t.iter().position(|&x| x == p as usize).unwrap_or_else(|| {
        t.push(p as usize);
        t.len() - 1
    });
    (kind << 16) | (i as u32 + 1)
}

fn item_handle(d: &mut HostData, g: *mut HSD_GObj) -> u32 {
    table_handle(&mut d.items, H_ITEM, g)
}

fn item_gobj(d: &HostData, h: u32) -> *mut HSD_GObj {
    if h >> 16 != H_ITEM {
        return core::ptr::null_mut();
    }
    let Some(&p) = d.items.get((h & 0xFFFF) as usize - 1) else { return core::ptr::null_mut() };
    let p = p as *mut HSD_GObj;
    if item_list().any(|g| g == p) && unsafe { !(*p).user_data.is_null() } { p } else { core::ptr::null_mut() }
}

fn gobj_handle(d: &mut HostData, g: *mut HSD_GObj) -> u32 {
    table_handle(&mut d.gobjs, H_GOBJ, g)
}

fn gobj_ptr(d: &HostData, h: u32) -> *mut HSD_GObj {
    match h >> 16 {
        H_FIGHTER => {
            let f = fighter_ptr(h);
            if f.is_null() { core::ptr::null_mut() } else { unsafe { (*f).gobj } }
        }
        H_ITEM => item_gobj(d, h),
        H_GOBJ => d.gobjs.get(((h & 0xFFFF) as usize).wrapping_sub(1)).map_or(core::ptr::null_mut(), |&p| p as *mut HSD_GObj),
        _ => core::ptr::null_mut(),
    }
}

fn spawn_ptr(d: &HostData, h: u32) -> Option<(*mut SpawnItem, bool)> {
    (h == (H_SPAWN << 16) | 1).then(|| d.spawns.last().map(|&(p, pre)| (p as *mut SpawnItem, pre))).flatten()
}

fn with_fighter<R>(c: &mut Caller<'_, HostData>, h: u32, dflt: R, f: impl FnOnce(&mut Fighter) -> R) -> R {
    let p = fighter_ptr(h);
    if p.is_null() {
        c.data_mut().bad_handles += 1;
        return dflt;
    }
    f(unsafe { &mut *p })
}

fn with_item<R>(c: &mut Caller<'_, HostData>, h: u32, dflt: R, f: impl FnOnce(&mut Item) -> R) -> R {
    let g = item_gobj(c.data(), h);
    if g.is_null() {
        c.data_mut().bad_handles += 1;
        return dflt;
    }
    f(unsafe { &mut *((*g).user_data as *mut Item) })
}

fn read_f32s<const N: usize>(c: &Caller<'_, HostData>, at: u32) -> Option<[f32; N]> {
    let m = mem_of(c)?;
    let src = m.data(c).get(at as usize..at as usize + N * 4)?;
    let mut v = [0f32; N];
    for (i, x) in v.iter_mut().enumerate() {
        *x = canon(f32::from_le_bytes(src[i * 4..i * 4 + 4].try_into().unwrap()));
    }
    Some(v)
}

fn write_words(c: &mut Caller<'_, HostData>, at: u32, w: &[u32]) {
    if let Some(m) = mem_of(c) {
        let d = m.data_mut(c);
        if let Some(dst) = d.get_mut(at as usize..at as usize + w.len() * 4) {
            for (i, x) in w.iter().enumerate() {
                dst[i * 4..i * 4 + 4].copy_from_slice(&x.to_le_bytes());
            }
        }
    }
}

fn has_feature(c: &Caller<'_, HostData>, f: u32) -> bool {
    let d = c.data();
    d.features.get(d.cur_module).is_some_and(|x| x & f != 0)
}

/// `gd_orig_call_*`: run the replaced original once, with `c` as the store context for hooks it reaches.
fn orig_call(c: &mut Caller<'_, HostData>) -> Ret {
    let Some(top) = c.data_mut().origs.last_mut() else { return Ret::Void };
    if top.used {
        return Ret::Void; // once per replace activation (spec section 5)
    }
    top.used = true;
    let f = top.f;
    let fuel = c.get_fuel().unwrap_or(0);
    unsafe {
        CTX.push(c as *mut Caller<'_, HostData> as *mut c_void);
        let r = (*f)();
        CTX.pop();
        let _ = c.set_fuel(fuel); // nested hooks had their own budgets
        r
    }
}

/// Module `gd`: the implemented subset of gdmod.h. An import outside it refuses the load (wasmi reports the name).
fn linker(engine: &Engine) -> Result<Linker<HostData>, String> {
    let mut l = Linker::<HostData>::new(engine);
    macro_rules! w {
        ($name:literal, $f:expr) => {
            l.func_wrap("gd", $name, $f).map_err(|e| format!("linker {}: {e}", $name))?;
        };
    }
    w!("gd_abi_host_version", || -> u32 { ABI_VERSION });
    w!("gd_match_frame", |c: Caller<'_, HostData>| -> i32 { c.data().frame });
    // gobjs
    w!("gd_gobj_kind", |c: Caller<'_, HostData>, h: u32| -> u32 {
        let g = gobj_ptr(c.data(), h);
        if g.is_null() { return 0; }
        match unsafe { (*g).classifier } as i32 {
            melee_types::HSD_GOBJ_CLASS_FIGHTER => 1,
            melee_types::HSD_GOBJ_CLASS_ITEM => 2,
            melee_types::HSD_GOBJ_CLASS_STAGE | melee_types::HSD_GOBJ_CLASS_GROUND => 3,
            _ => 0,
        }
    });
    w!("gd_gobj_to_fighter", |c: Caller<'_, HostData>, h: u32| -> u32 { fighter_handle_of_gobj(gobj_ptr(c.data(), h)) });
    w!("gd_gobj_to_item", |mut c: Caller<'_, HostData>, h: u32| -> u32 {
        let g = gobj_ptr(c.data(), h);
        if g.is_null() || unsafe { (*g).classifier } as i32 != melee_types::HSD_GOBJ_CLASS_ITEM { return 0; }
        item_handle(c.data_mut(), g)
    });
    // fighters
    w!("gd_fighter_from_slot", |slot: i32| -> u32 {
        if (0..4).contains(&slot) && !fighter_ptr(handle_of(slot)).is_null() { handle_of(slot) } else { 0 }
    });
    w!("gd_player_present", |slot: i32| -> i32 { ((0..4).contains(&slot) && !fighter_ptr(handle_of(slot)).is_null()) as i32 });
    w!("gd_player_stocks", |slot: i32| -> i32 {
        if (0..4).contains(&slot) { unsafe { melee_fighter::stale::PLAYER_SLOTS[slot as usize].stocks as i32 } } else { 0 }
    });
    w!("gd_player_character", |slot: i32| -> i32 {
        if (0..4).contains(&slot) { melee_fighter::player::get_player_character(slot) as i32 } else { -1 }
    });
    w!("gd_fighter_gobj", |mut c: Caller<'_, HostData>, h: u32| -> u32 {
        let g = with_fighter(&mut c, h, core::ptr::null_mut(), |f| f.gobj);
        gobj_handle(c.data_mut(), g)
    });
    w!("gd_fighter_slot", |mut c: Caller<'_, HostData>, h: u32| -> i32 { with_fighter(&mut c, h, -1, |f| f.player_idx as i32) });
    w!("gd_fighter_kind", |mut c: Caller<'_, HostData>, h: u32| -> i32 { with_fighter(&mut c, h, -1, |f| f.kind as i32) });
    w!("gd_fighter_get_action_state", |mut c: Caller<'_, HostData>, h: u32| -> i32 { with_fighter(&mut c, h, -1, |f| f.motion_id as i32) });
    w!("gd_fighter_request_action_state", |mut c: Caller<'_, HostData>, h: u32, state: i32| -> i32 {
        if !has_feature(&c, FEATURE_ACTION_REQUEST) || fighter_ptr(h).is_null() {
            c.data_mut().bad_handles += fighter_ptr(h).is_null() as u32;
            return 0;
        }
        c.data_mut().requests.push((h, state));
        1
    });
    w!("gd_fighter_get_percent", |mut c: Caller<'_, HostData>, h: u32| -> f32 { with_fighter(&mut c, h, 0.0, |f| canon(f.dmg.x1830_percent)) });
    w!("gd_fighter_set_percent", |mut c: Caller<'_, HostData>, h: u32, v: f32| { with_fighter(&mut c, h, (), |f| f.dmg.x1830_percent = canon(v)) });
    w!("gd_fighter_get_facing", |mut c: Caller<'_, HostData>, h: u32| -> f32 { with_fighter(&mut c, h, 0.0, |f| canon(f.facing_dir)) });
    w!("gd_fighter_set_facing", |mut c: Caller<'_, HostData>, h: u32, v: f32| {
        if v == 1.0 || v == -1.0 { with_fighter(&mut c, h, (), |f| f.facing_dir = v) }
    });
    w!("gd_fighter_get_pos_x", |mut c: Caller<'_, HostData>, h: u32| -> f32 { with_fighter(&mut c, h, 0.0, |f| canon(f.cur_pos.x)) });
    w!("gd_fighter_get_pos_y", |mut c: Caller<'_, HostData>, h: u32| -> f32 { with_fighter(&mut c, h, 0.0, |f| canon(f.cur_pos.y)) });
    w!("gd_fighter_get_pos_z", |mut c: Caller<'_, HostData>, h: u32| -> f32 { with_fighter(&mut c, h, 0.0, |f| canon(f.cur_pos.z)) });
    w!("gd_fighter_set_pos", |mut c: Caller<'_, HostData>, h: u32, x: f32, y: f32, z: f32| {
        with_fighter(&mut c, h, (), |f| { f.cur_pos.x = canon(x); f.cur_pos.y = canon(y); f.cur_pos.z = canon(z); })
    });
    w!("gd_fighter_get_vel_x", |mut c: Caller<'_, HostData>, h: u32| -> f32 { with_fighter(&mut c, h, 0.0, |f| canon(f.self_vel.x)) });
    w!("gd_fighter_get_vel_y", |mut c: Caller<'_, HostData>, h: u32| -> f32 { with_fighter(&mut c, h, 0.0, |f| canon(f.self_vel.y)) });
    w!("gd_fighter_get_vel_z", |mut c: Caller<'_, HostData>, h: u32| -> f32 { with_fighter(&mut c, h, 0.0, |f| canon(f.self_vel.z)) });
    w!("gd_fighter_set_vel", |mut c: Caller<'_, HostData>, h: u32, x: f32, y: f32, z: f32| {
        with_fighter(&mut c, h, (), |f| { f.self_vel.x = canon(x); f.self_vel.y = canon(y); f.self_vel.z = canon(z); })
    });
    // gd_fighter_get_input(h, gd_input* out): 32 bytes, Fighter.input slot 0
    w!("gd_fighter_get_input", |mut c: Caller<'_, HostData>, h: u32, out: u32| {
        let w = with_fighter(&mut c, h, [0u32; 8], |f| {
            let i = &f.input;
            [canon(i.lstick[0].x).to_bits(), canon(i.lstick[0].y).to_bits(), canon(i.cstick[0].x).to_bits(), canon(i.cstick[0].y).to_bits(),
             canon(i.triggers[0]).to_bits(), i.held_buttons[0], i.pressed_buttons, i.released_buttons]
        });
        write_words(&mut c, out, &w);
    });
    // gd_fighter_read(h, gd_fighter_view* out): 80 bytes, little-endian
    w!("gd_fighter_read", |mut c: Caller<'_, HostData>, h: u32, out: u32| {
        let mut v = [0u32; 20];
        with_fighter(&mut c, h, (), |f| {
            let fl = [f.cur_pos.x, f.cur_pos.y, f.cur_pos.z, f.self_vel.x, f.self_vel.y, f.self_vel.z, f.x8c_kb_vel.x, f.x8c_kb_vel.y, f.x8c_kb_vel.z, f.facing_dir, f.dmg.x1830_percent];
            for (i, x) in fl.iter().enumerate() { v[i] = canon(*x).to_bits(); }
            v[11] = f.motion_id as u32;
            v[12] = f.kind as u32;
            v[13] = f.player_idx as u32;
        });
        write_words(&mut c, out, &v);
    });
    // gd_fighter_write(h, const gd_fighter_view* in, mask): POS 1, VEL 2, KB_VEL 4, FACING 8, PERCENT 16
    w!("gd_fighter_write", |mut c: Caller<'_, HostData>, h: u32, inp: u32, mask: u32| {
        let Some(v) = read_f32s::<11>(&c, inp) else { return };
        with_fighter(&mut c, h, (), |f| {
            if mask & 1 != 0 { f.cur_pos.x = v[0]; f.cur_pos.y = v[1]; f.cur_pos.z = v[2]; }
            if mask & 2 != 0 { f.self_vel.x = v[3]; f.self_vel.y = v[4]; f.self_vel.z = v[5]; }
            if mask & 4 != 0 { f.x8c_kb_vel.x = v[6]; f.x8c_kb_vel.y = v[7]; f.x8c_kb_vel.z = v[8]; }
            if mask & 8 != 0 && (v[9] == 1.0 || v[9] == -1.0) { f.facing_dir = v[9]; }
            if mask & 16 != 0 { f.dmg.x1830_percent = v[10]; }
        });
    });
    // items
    w!("gd_item_count", || -> i32 { item_list().count() as i32 });
    w!("gd_item_at", |mut c: Caller<'_, HostData>, i: i32| -> u32 {
        let g = if i < 0 { None } else { item_list().nth(i as usize) };
        g.map_or(0, |g| item_handle(c.data_mut(), g))
    });
    w!("gd_item_kind", |mut c: Caller<'_, HostData>, h: u32| -> i32 { with_item(&mut c, h, -1, |ip| ip.kind as i32) });
    w!("gd_item_gobj", |mut c: Caller<'_, HostData>, h: u32| -> u32 {
        let g = item_gobj(c.data(), h);
        if g.is_null() { c.data_mut().bad_handles += 1; 0 } else { h } // an item's gobj handle is its item handle
    });
    // gd_item_read(h, gd_item_view* out): 52 bytes
    w!("gd_item_read", |mut c: Caller<'_, HostData>, h: u32, out: u32| {
        let mut v = [0u32; 13];
        with_item(&mut c, h, (), |ip| {
            let fl = [ip.pos.x, ip.pos.y, ip.pos.z, ip.x40_vel.x, ip.x40_vel.y, ip.x40_vel.z, ip.facing_dir];
            for (i, x) in fl.iter().enumerate() { v[i] = canon(*x).to_bits(); }
            v[7] = ip.kind as u32;
            let o = fighter_handle_of_gobj(ip.owner);
            v[8] = if o == 0 { u32::MAX } else { (o & 0xFFFF) - 1 };
        });
        write_words(&mut c, out, &v);
    });
    w!("gd_item_write", |mut c: Caller<'_, HostData>, h: u32, inp: u32, mask: u32| {
        if !has_feature(&c, FEATURE_ITEM_WRITE) { return; }
        let Some(v) = read_f32s::<7>(&c, inp) else { return };
        with_item(&mut c, h, (), |ip| {
            if mask & 1 != 0 { ip.pos.x = v[0]; ip.pos.y = v[1]; ip.pos.z = v[2]; }
            if mask & 2 != 0 { ip.x40_vel.x = v[3]; ip.x40_vel.y = v[4]; ip.x40_vel.z = v[5]; }
            if mask & 8 != 0 && (v[6] == 1.0 || v[6] == -1.0) { ip.facing_dir = v[6]; }
        });
    });
    // spawn requests (inside item.spawn* only; setters in pre hooks only)
    w!("gd_spawn_get_kind", |mut c: Caller<'_, HostData>, h: u32| -> i32 {
        match spawn_ptr(c.data(), h) { Some((p, _)) => unsafe { (*p).kind as i32 }, None => { c.data_mut().bad_handles += 1; -1 } }
    });
    w!("gd_spawn_set_kind", |mut c: Caller<'_, HostData>, h: u32, kind: i32| {
        match spawn_ptr(c.data(), h) {
            // only vanilla common item kinds (0..It_Kind_Kuriboh): a kind whose article is not loaded spawns nothing
            Some((p, true)) if has_feature(&c, FEATURE_ITEM_WRITE) && (0..melee_types::It_Kind_Kuriboh as i32).contains(&kind) => unsafe { (*p).kind = kind as _ },
            _ => c.data_mut().bad_handles += 1,
        }
    });
    w!("gd_spawn_get_pos", |mut c: Caller<'_, HostData>, h: u32, out: u32| {
        let w = match spawn_ptr(c.data(), h) {
            Some((p, _)) => unsafe { [canon((*p).pos.x).to_bits(), canon((*p).pos.y).to_bits(), canon((*p).pos.z).to_bits()] },
            None => { c.data_mut().bad_handles += 1; [0; 3] }
        };
        write_words(&mut c, out, &w);
    });
    w!("gd_spawn_set_pos", |mut c: Caller<'_, HostData>, h: u32, inp: u32| {
        let Some(v) = read_f32s::<3>(&c, inp) else { return };
        match spawn_ptr(c.data(), h) {
            Some((p, true)) if has_feature(&c, FEATURE_ITEM_WRITE) => unsafe { (*p).pos.x = v[0]; (*p).pos.y = v[1]; (*p).pos.z = v[2]; },
            _ => c.data_mut().bad_handles += 1,
        }
    });
    // stage
    // tag pairs and the sim transition (melee_modes::tu::tag_swap; rust/mods/tag_fighter): all game state
    w!("gd_tag_partner", |slot: i32| -> i32 { unsafe { melee_modes::tu::tag_swap::partner_kind(slot) } });
    w!("gd_tag_check", |slot: i32| -> i32 { if !(0..4).contains(&slot) { return -1 } unsafe { melee_modes::tu::tag_swap::check(slot) } });
    w!("gd_tag_begin", |slot: i32, len: i32| -> i32 { if !(0..4).contains(&slot) { return -1 } unsafe { melee_modes::tu::tag_swap::begin(slot, len) } });
    w!("gd_tag_swap", |skip: i32| -> i32 { unsafe { melee_modes::tu::tag_swap::request_swap(skip != 0) } });
    w!("gd_transition_phase", || -> i32 { unsafe { melee_modes::tu::tag_swap::TRANSITION.phase as i32 } });
    w!("gd_transition_frame", || -> i32 { unsafe { melee_modes::tu::tag_swap::TRANSITION.frame as i32 } });
    w!("gd_transition_slot", || -> i32 { unsafe { melee_modes::tu::tag_swap::TRANSITION.slot as i32 } });
    w!("gd_transition_kind", || -> i32 { unsafe { melee_modes::tu::tag_swap::TRANSITION.kind as i32 } });
    // tag with air tags allowed (flags bit 0); a stage switch under the transition freeze; a hold freeze (Envoy)
    w!("gd_tag_begin_ex", |slot: i32, len: i32, flags: i32| -> i32 {
        if !(0..4).contains(&slot) { return -1 }
        unsafe { melee_modes::tu::tag_swap::begin_ex(slot, len, flags & 1 != 0) }
    });
    // Envoy Quick Tag: a swap with no freeze (`any`: the window after dealing damage), the newcomer paused `pause`
    // frames; the tag button (0 Y, 1 X, 2 D-pad down classic)
    w!("gd_tag_quick", |slot: i32, any: i32, pause: i32| -> i32 {
        if !(0..4).contains(&slot) { return -1 }
        unsafe { melee_modes::tu::tag_swap::quick(slot, any != 0, pause) }
    });
    w!("gd_envoy_tag_button", || -> i32 { melee_modes::tu::envoy::tag_button() });
    // Envoy Assist Strike: the partner strikes beside the player (dir 0 forward, 1 up, 2 down; a smash on the
    // ground, an aerial in the air): the move (0-5) or a refusal; the frames the running strike has run + 1 (0 none)
    w!("gd_tag_assist", |slot: i32, dir: i32| -> i32 { unsafe { melee_modes::tu::tag_swap::assist(slot, dir) } });
    // Envoy rooms: park a foe slot for the next wave; the parked-and-asleep slots (bits); start the next wave
    // profiling marks (no game effect): the time since the previous mark as a frame-profile phase "mod mark N"
    w!("gd_prof_mark", |id: i32| -> i32 {
        thread_local! { static LAST: core::cell::Cell<Option<std::time::Instant>> = const { core::cell::Cell::new(None) }; }
        if melee_core::frameprof::on() {
            let now = std::time::Instant::now();
            let prev = LAST.with(|l| l.replace(Some(now)));
            if let Some(p) = prev {
                let us = (now - p).as_micros() as u32;
                if us >= 500 {
                    melee_core::frameprof::event(melee_core::frameprof::Kind::Note, melee_core::frameprof::Text::new().s("mod mark ").n(id as i64), us, 0);
                }
            }
        }
        0
    });
    w!("gd_envoy_wave_park", |slot: i32| -> i32 { unsafe { melee_modes::tu::envoy::wave_park(slot) } });
    w!("gd_envoy_wave_pending", || -> i32 { unsafe { melee_modes::tu::envoy::wave_pending() } });
    w!("gd_envoy_wave_start", || -> i32 { unsafe { melee_modes::tu::envoy::wave_start() } });
    w!("gd_tag_assist_frames", |slot: i32| -> i32 { unsafe { melee_modes::tu::tag_swap::assist_frames(slot) } });
    w!("gd_stage_transition", |kind: i32, len: i32| -> i32 { unsafe { melee_modes::tu::tag_swap::stage_begin(kind, len) } });
    // Envoy "Fights 2": a live stage transition (no freeze; the hand-off at frame `at`) and its record: phase, done,
    // from, to, frame, len, at, finished count
    w!("gd_stage_transition_live", |kind: i32, len: i32, at: i32| -> i32 { unsafe { melee_modes::tu::tag_swap::stage_live_begin(kind, len, at) } });
    w!("gd_stage_live", |mut c: Caller<'_, HostData>, out: u32| -> i32 {
        let l = unsafe { melee_modes::tu::tag_swap::LIVE };
        write_words(&mut c, out, &[l.phase as u32, l.done as u32, l.from as i32 as u32, l.to as i32 as u32, l.frame as u32, l.len as u32, l.at as u32, l.count]);
        l.phase as i32
    });
    w!("gd_hold_begin", |len: i32| -> i32 { unsafe { melee_modes::tu::tag_swap::hold_begin(len) } });
    w!("gd_hold_release", || -> i32 { unsafe { melee_modes::tu::tag_swap::hold_release() } });
    // Envoy run (melee_modes::tu::envoy): the next match's plan, the presentation view, live upgrades
    w!("gd_envoy_plan", |c: Caller<'_, HostData>, ptr: u32, n: u32| -> i32 {
        let n = (n as usize).min(melee_modes::tu::envoy::PLAN_WORDS);
        let Some(m) = mem_of(&c) else { return -1 };
        let Some(b) = m.data(&c).get(ptr as usize..ptr as usize + n * 4) else { return -1 };
        unsafe { for i in 0..n { melee_modes::tu::envoy::PLAN[i] = i32::from_le_bytes(b[i * 4..i * 4 + 4].try_into().unwrap()); } }
        unsafe {
            melee_modes::tu::petey_file::prefetch();
            if let Some(h) = melee_modes::tu::envoy::PLAN_HINT {
                h(&*core::ptr::addr_of!(melee_modes::tu::envoy::PLAN));
            }
        }
        0
    });
    w!("gd_envoy_publish", |c: Caller<'_, HostData>, ptr: u32, n: u32| -> i32 {
        let n = (n as usize).min(melee_modes::tu::envoy::VIEW_WORDS);
        let Some(m) = mem_of(&c) else { return -1 };
        let Some(b) = m.data(&c).get(ptr as usize..ptr as usize + n * 4) else { return -1 };
        unsafe { for i in 0..n { melee_modes::tu::envoy::VIEW[i] = i32::from_le_bytes(b[i * 4..i * 4 + 4].try_into().unwrap()); } }
        0
    });
    // test knob: MELEE_ENVOY_START_FIGHT=N (1-5) starts each new run at fight N (fixed for the process: deterministic)
    w!("gd_envoy_start_fight", || -> i32 {
        static V: std::sync::OnceLock<i32> = std::sync::OnceLock::new();
        *V.get_or_init(|| std::env::var("MELEE_ENVOY_START_FIGHT").ok().and_then(|v| v.parse::<i32>().ok()).map_or(0, |n| (n - 1).clamp(0, 4)))
    });
    // the salt of a new run's seed (menu-time, set by the camp's New run; 0 under MELEE_ENVOY_ARM and in scripted runs)
    w!("gd_envoy_salt", || -> i32 { unsafe { melee_modes::tu::envoy::RUN_SALT } });
    // test knob: MELEE_ENVOY_TEST_DRIVES="key[:level],key/key|keystone" (main / partner | keystones), fixed per process
    w!("gd_envoy_test_drives", |mut c: Caller<'_, HostData>, out: u32, cap: u32| -> i32 {
        static V: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        let v = V.get_or_init(|| std::env::var("MELEE_ENVOY_TEST_DRIVES").unwrap_or_default());
        let n = v.len().min(cap as usize);
        let Some(m) = mem_of(&c) else { return -1 };
        let Some(d) = m.data_mut(&mut c).get_mut(out as usize..out as usize + n) else { return -1 };
        d.copy_from_slice(&v.as_bytes()[..n]);
        n as i32
    });
    // MELEE_ENVOY_SHARED_DRIVES=1: one drive set for the pair (envoy-core-v2.md Q1's alternative), fixed per process
    w!("gd_envoy_shared_drives", || -> i32 {
        static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *V.get_or_init(|| std::env::var("MELEE_ENVOY_SHARED_DRIVES").is_ok_and(|v| v == "1")) as i32
    });
    // MELEE_ENVOY_COOP=1: local co-op (envoy-core-v2.md V2), fixed per process; the run reads it when it starts
    w!("gd_envoy_coop", || -> i32 {
        static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        // V3: or player 2 joined at the camp (melee_modes::tu::envoy::COOP_JOIN, set by envoy_ui before arming)
        (*V.get_or_init(|| std::env::var("MELEE_ENVOY_COOP").is_ok_and(|v| v == "1")) || unsafe { melee_modes::tu::envoy::COOP_JOIN } != 0) as i32
    });
    // V2 drive effects (melee_modes::tu::envoy): speed from a stored base, the orb's absorb, hit attribution, air
    // jumps, launching blasts on the engine's hit path; all game state, rolled back with the game
    w!("gd_envoy_speed", |slot: i32, active: i32, bench: i32| -> i32 { unsafe { melee_modes::tu::envoy::speed(slot, active, bench) } });
    w!("gd_envoy_absorb", |slot: i32| -> i32 { unsafe { melee_modes::tu::envoy::absorb(slot) } });
    w!("gd_envoy_attacker", |slot: i32| -> i32 { unsafe { melee_modes::tu::envoy::attacker(slot) } });
    w!("gd_envoy_hit_item", |slot: i32| -> i32 { unsafe { melee_modes::tu::envoy::hit_item(slot) } });
    // the combo breaker: the slot's fighter bursts out of the combo (no launch, out of hitstun, intangible `frames`;
    // the shield-break sound, a quake, every fighter frozen `freeze` frames)
    w!("gd_envoy_combo_break", |slot: i32, frames: i32, freeze: i32| -> i32 { unsafe { melee_modes::tu::envoy::combo_break(slot, frames, freeze) } });
    w!("gd_envoy_jumps", |slot: i32| -> i32 { unsafe { melee_modes::tu::envoy::jumps(slot) } });
    w!("gd_envoy_air_jump", |slot: i32| -> i32 { unsafe { melee_modes::tu::envoy::air_jump(slot) } });
    w!("gd_envoy_blast", |slot: i32, x: f32, y: f32, r: f32, dmg: i32, angle: i32, kbg: i32, bkb: i32| -> i32 {
        unsafe { melee_modes::tu::envoy::blast(slot, canon(x), canon(y), canon(r), dmg, angle, kbg, bkb) }
    });
    // a blast's landing cap (x1000, 0 none) and log tag, on the blast `slot` armed this frame; what blasts landed on
    // a victim slot this frame (9 words: envoy::landed)
    w!("gd_envoy_blast_cap", |slot: i32, cap_milli: i32, tag: i32| -> i32 { unsafe { melee_modes::tu::envoy::blast_cap(slot, cap_milli, tag) } });
    w!("gd_envoy_landed", |mut c: Caller<'_, HostData>, slot: i32, out: u32| -> i32 {
        let l = unsafe { melee_modes::tu::envoy::landed(slot) };
        write_words(&mut c, out, &l.map(|x| x as u32));
        l[0]
    });
    // the boss carrier slot of a Petey fight (-1 none) and Petey's body position (x, y as f32 bits in out[0..2]), the hits each slot landed on him (out[2..6])
    w!("gd_envoy_boss", |mut c: Caller<'_, HostData>, out: u32| -> i32 {
        let s = melee_modes::tu::petey::carrier();
        let (x, y) = melee_modes::tu::petey::pos();
        let h = unsafe { melee_modes::tu::petey::HITS };
        write_words(&mut c, out, &[canon(x).to_bits(), canon(y).to_bits(), h[0], h[1], h[2], h[3]]);
        s
    });
    // V3 (envoy-synergy-v3.md 12.5): the engine's event bits (bit 0 L-cancel), read and cleared; the boss's damage
    // per slot (x1000, running) with its HP and full HP (x1000); Grand Finale's knockback multiplier on the hit now
    w!("gd_envoy_events", |slot: i32| -> u32 { unsafe { melee_modes::tu::envoy::events(slot) } });
    w!("gd_envoy_boss_dmg", |mut c: Caller<'_, HostData>, out: u32| -> i32 {
        let s = melee_modes::tu::petey::carrier();
        let d = unsafe { melee_modes::tu::petey::DMG };
        let v = melee_modes::tu::petey::view();
        let (hp, max) = if v[0] != 0 { (v[1].max(0) as u32 * 1000, v[2].max(0) as u32 * 1000) } else { (0, 0) };
        write_words(&mut c, out, &[d[0], d[1], d[2], d[3], hp, max]);
        s
    });
    w!("gd_envoy_kb_scale", |slot: i32, mul: i32| -> i32 { unsafe { melee_modes::tu::envoy::kb_scale(slot, mul) } });
    w!("gd_envoy_seq", || -> i32 { unsafe { melee_modes::tu::envoy::SEQ } });
    // Envoy enemy waves (first step): spawn a script-layer enemy (which: script_game's kind list, 0 Goomba, 2 ReDead,
    // 4 Octorok), read its registry fields (ScriptGame_EnemyI) and status (0 gone, 1 alive, 2 defeated). Gameplay only.
    w!("gd_envoy_enemy_spawn", |which: i32, x: f32, y: f32, facing: i32, handle: i32| -> i32 {
        unsafe {
            if melee_modes::tu::envoy::armed() == 0 || melee_modes::tu::stage_hop::check(-1) == melee_modes::tu::stage_hop::HOP_NOT_GAMEPLAY { return -1 }
            melee_modes::tu::script_game::ScriptGame_SpawnEnemy(which, canon(x).to_bits() as i32, canon(y).to_bits() as i32, facing, handle)
        }
    });
    w!("gd_envoy_enemy_i", |handle: i32, field: i32| -> i32 { unsafe { melee_modes::tu::script_game::ScriptGame_EnemyI(handle, field) } });
    w!("gd_envoy_enemy_status", |handle: i32| -> i32 { unsafe { melee_modes::tu::script_game::ScriptGame_EnemyStatus(handle) } });
    // Envoy sessions: hold the retail match end (the stop runs after the win), and end the held match
    w!("gd_envoy_end_hold", |on: i32| -> i32 { unsafe { if melee_modes::tu::envoy::armed() == 0 { return -1 } melee_modes::tu::gmvs::gmVs_SetEndHold(on) } 0 });
    w!("gd_envoy_end_match", || -> i32 {
        unsafe {
            if melee_modes::tu::envoy::armed() == 0 { return -1 }
            let held = melee_modes::tu::gmvs::gmVs_GetHeldOutcome();
            melee_modes::tu::gmvs::gmVs_SetEndHold(0);
            melee_modes::tu::gmvs::gmVs_EndMatch(if held != 0 { held } else { melee_types::OUTCOME_ELIMINATION as i32 });
        }
        0
    });
    // the armed run's match was left from the pause (No Contest): the sim keeps the run instead of losing it
    w!("gd_envoy_quit", || -> i32 { unsafe { melee_modes::tu::envoy::quit_pending() } });
    // the door (envoy-redesign.md 1.3): whether the run is armed (the camp's Continue / New run); the sim publishes a
    // plan only while armed, so VS stays retail with the mods loaded
    w!("gd_envoy_armed", || -> i32 { melee_modes::tu::envoy::armed() });
    // the saved run record (host-side file, read at arm time): the words copied (<= cap), 0 = no saved run
    w!("gd_envoy_record", |mut c: Caller<'_, HostData>, out: u32, cap: u32| -> i32 {
        let (rec, n) = unsafe { (melee_modes::tu::envoy::SAVED, melee_modes::tu::envoy::SAVED_LEN) };
        let n = (n.max(0) as usize).min(cap as usize).min(rec.len());
        let Some(m) = mem_of(&c) else { return -1 };
        let Some(d) = m.data_mut(&mut c).get_mut(out as usize..out as usize + n * 4) else { return -1 };
        for i in 0..n {
            d[i * 4..i * 4 + 4].copy_from_slice(&rec[i].to_le_bytes());
        }
        n as i32
    });
    // the pairs the last rewrite used: 0 / 1 player 1's main / partner, 2 / 3 player 2's (co-op; -1 solo)
    w!("gd_envoy_roster", |i: i32| -> i32 {
        match i { 0 | 1 => unsafe { melee_modes::tu::envoy::ROSTER[i as usize] }, 2 | 3 => unsafe { melee_modes::tu::envoy::ROSTER2[i as usize - 2] }, _ => -1 }
    });
    // Brawl audio from the player's own disc (melee_audio::brawl, docs/rust-port/notes/brawl-audio.md). Presentation
    // only: the play calls always answer 0 (fire and forget, silent without the disc) so a mod's sim cannot branch on
    // whether the disc is present; gd_brawl_present is for UI only.
    w!("gd_brawl_present", || -> i32 { melee_audio::brawl::present() as i32 });
    w!("gd_brawl_music", |c: Caller<'_, HostData>, ptr: u32, n: u32| -> i32 {
        if let Some(name) = mod_str(&c, ptr, n) { melee_audio::brawl::play_music(&name); }
        0
    });
    w!("gd_brawl_music_stop", |frames: i32| -> i32 { melee_audio::brawl::stop_music(frames.clamp(0, 600) as u32); 0 });
    w!("gd_brawl_sfx", |c: Caller<'_, HostData>, ptr: u32, n: u32| -> i32 {
        if let Some(name) = mod_str(&c, ptr, n) { melee_audio::brawl::play_sfx(&name); }
        0
    });
    w!("gd_envoy_apply", |slot: i32, which: i32, v: i32| -> i32 { unsafe { melee_modes::tu::envoy::apply(slot, which, v) } });
    // a drive's burst, heal or Burning tick: percent delta (x1000) on the slot's active fighter (bench 0) or the bench
    w!("gd_envoy_percent", |slot: i32, bench: i32, delta: i32| -> i32 { unsafe { melee_modes::tu::envoy::percent(slot, bench, delta) } });
    // the pad a port pressed this logic frame (HSD_PadMasterStatus.trigger: the synced sim input, valid while frozen)
    w!("gd_pad_pressed", |port: i32| -> u32 {
        if !(0..4).contains(&port) { return 0 }
        unsafe { melee_runtime::tu::controller::HSD_PadMasterStatus[port as usize].trigger }
    });
    // the buttons a port holds this logic frame, and its main stick (x | y << 8, signed bytes): synced sim input
    w!("gd_pad_held", |port: i32| -> u32 {
        if !(0..4).contains(&port) { return 0 }
        unsafe { melee_runtime::tu::controller::HSD_PadMasterStatus[port as usize].button }
    });
    w!("gd_pad_stick", |port: i32| -> i32 {
        if !(0..4).contains(&port) { return 0 }
        let p = unsafe { &melee_runtime::tu::controller::HSD_PadMasterStatus[port as usize] };
        (p.stickX as u8 as i32) | (p.stickY as u8 as i32) << 8
    });
    w!("gd_stage_kind", || -> i32 { unsafe { melee_world::tu::stage::selected_stage.stkind as i32 } });
    // in-match stage switching (melee_modes::tu::stage_hop): checked now, applied between logic frames
    w!("gd_stage_grkind", || -> i32 { melee_modes::tu::stage_hop::current() });
    w!("gd_stage_switch", |mut c: Caller<'_, HostData>, kind: i32| -> i32 {
        let rc = unsafe { melee_modes::tu::stage_hop::check(kind) };
        if rc == 0 { c.data_mut().stage_switch = Some(kind); c.data_mut().stage_switch_rc = 1; }
        rc
    });
    w!("gd_stage_switch_result", |c: Caller<'_, HostData>| -> i32 { c.data().stage_switch_rc });
    // a sim mod's state digest: folded into this frame's word, which the netplay session mixes into its compared hash
    w!("gd_hash_word", |mut c: Caller<'_, HostData>, word: u32| {
        let d = c.data_mut();
        d.hash_acc = hash_mix(if d.hash_any { d.hash_acc } else { 0x811C_9DC5 }, word);
        d.hash_any = true;
    });
    // the game's RNG (HSD_Randi / HSD_Randf on the logic seed): the draws are game state and roll back with it
    w!("gd_rand_u32", |bound: u32| -> u32 {
        if bound == 0 { 0 } else { melee_runtime::tu::random::global::HSD_Randi(bound.min(i32::MAX as u32) as i32) as u32 }
    });
    w!("gd_rand_f32", || -> f32 { melee_runtime::tu::random::global::HSD_Randf() });
    // trig: the game's own implementations (MSL sinf/cosf as retail computes them, lbtrigf atan2f, lb powf)
    w!("gd_sinf", |x: f32| -> f32 { canon(melee_runtime::tu::gekko_fp::sinf(canon(x))) });
    w!("gd_cosf", |x: f32| -> f32 { canon(melee_runtime::tu::gekko_fp::cosf(canon(x))) });
    w!("gd_atan2f", |y: f32, x: f32| -> f32 { canon(melee_runtime::lbtrigf::atan2f(canon(y), canon(x))) });
    w!("gd_powf", |x: f32, y: f32| -> f32 { canon(melee_runtime::tu::lb_00CE::powf(canon(x), canon(y))) });
    // original call / return value
    w!("gd_orig_call_void", |mut c: Caller<'_, HostData>| { orig_call(&mut c); });
    w!("gd_orig_call_i32", |mut c: Caller<'_, HostData>| -> i32 { if let Ret::I32(v) = orig_call(&mut c) { v } else { 0 } });
    w!("gd_orig_call_f32", |mut c: Caller<'_, HostData>| -> f32 { if let Ret::F32(v) = orig_call(&mut c) { canon(v) } else { 0.0 } });
    w!("gd_set_return_i32", |mut c: Caller<'_, HostData>, v: i32| { c.data_mut().set_return = Some(Val::I32(v)); });
    w!("gd_set_return_f32", |mut c: Caller<'_, HostData>, v: f32| { c.data_mut().set_return = Some(Val::F32(canon(v).into())); });
    w!("gd_log", |mut c: Caller<'_, HostData>, msg: u32, value: i32| {
        let s = mem_of(&c).map(|m| {
            let d = m.data(&c);
            let o = (msg as usize).min(d.len());
            let n = d[o..].iter().take(256).position(|&b| b == 0).unwrap_or(0);
            String::from_utf8_lossy(&d[o..o + n]).into_owned()
        }).unwrap_or_default();
        c.data_mut().log.push(format!("{s} {value}"));
    });
    Ok(l)
}

fn rd_u32(d: &[u8], o: usize) -> Result<u32, String> {
    d.get(o..o + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap())).ok_or_else(|| format!("read {o:#x} outside memory"))
}

fn cstr(d: &[u8], o: usize, max: usize) -> Result<String, String> {
    let b = d.get(o..o + max).ok_or("manifest string outside memory")?;
    let n = b.iter().position(|&x| x == 0).ok_or("manifest string not NUL-terminated")?;
    String::from_utf8(b[..n].to_vec()).map_err(|_| "manifest string not UTF-8".into())
}

/// The address of an exported data object (`gd_mod_manifest`, `gdmod_abi_version`): an exported i32 global.
fn data_export(inst: &wasmi::Instance, store: &Store<HostData>, name: &str) -> Result<usize, String> {
    let g = inst.get_global(store, name).ok_or_else(|| format!("export {name}: missing (refuse)"))?;
    g.get(store).i32().map(|v| v as u32 as usize).ok_or_else(|| format!("export {name}: not an i32 global (refuse)"))
}

/// The row signature as the host dispatches it; None when the row has no usable signature.
fn row_sig(addr: u32) -> Option<Sig> {
    parse_sig(hook_row(addr)?.1)
}

impl Host {
    pub fn new() -> Host {
        let engine = Engine::new(&engine_config());
        Host { store: Store::new(&engine, HostData::default()), mods: Vec::new(), chains: BTreeMap::new() }
    }

    /// Validate and instantiate one module. Any failure refuses the whole mod with a message naming the check.
    pub fn load(&mut self, bytes: &[u8]) -> Result<usize, String> {
        if self.mods.len() >= MAX_MODS {
            return Err(format!("mods loaded: limit {MAX_MODS}"));
        }
        if has_start_section(bytes) {
            return Err("start function: present (refuse)".into());
        }
        let engine = self.store.engine().clone();
        let module = Module::new(&engine, bytes).map_err(|e| format!("valid wasm: {e}"))?;
        for imp in module.imports() {
            if imp.module() != "gd" {
                return Err(format!("import {}::{}: module must be gd", imp.module(), imp.name()));
            }
            if imp.ty().func().is_none() {
                return Err(format!("import gd::{}: only functions may be imported", imp.name()));
            }
        }
        let inst = linker(&engine)?
            .instantiate_and_start(&mut self.store, &module)
            .map_err(|e| format!("imports / instantiate: {e} (an import gdmod.h declares but this host lacks is refused)"))?;
        let mem = inst.get_memory(&self.store, "memory").ok_or("memory: no exported `memory`")?;
        let mt = mem.ty(&self.store);
        if mt.maximum() != Some(mt.minimum()) || mt.minimum() > MAX_PAGES {
            return Err(format!("memory: must be fixed (min = max) and at most {MAX_PAGES} pages, got min {} max {:?}", mt.minimum(), mt.maximum()));
        }
        let table = inst.get_table(&self.store, "__indirect_function_table").ok_or("table: no exported function table (link with --export-table)")?;
        let abi_at = data_export(&inst, &self.store, "gdmod_abi_version")?;
        let man = data_export(&inst, &self.store, "gd_mod_manifest")?;
        let d = mem.data(&self.store);
        let abi = rd_u32(d, abi_at)?;
        if abi >> 16 != ABI_VERSION >> 16 {
            return Err(format!("gdmod_abi_version: major {} != host {}", abi >> 16, ABI_VERSION >> 16));
        }
        let f = |i: usize| rd_u32(d, man + i * 4);
        if f(0)? != MANIFEST_MAGIC {
            return Err("manifest magic: not GDMD".into());
        }
        if f(1)? != MANIFEST_SIZE {
            return Err(format!("manifest struct_size: {} != {MANIFEST_SIZE}", f(1)?));
        }
        let (abi_min, abi_max, table_v, version, req, opt, state_bytes, count, hooks_at) = (f(2)?, f(3)?, f(4)?, f(5)?, f(6)?, f(7)?, f(8)?, f(9)?, f(10)? as usize);
        if abi_min >> 16 != ABI_VERSION >> 16 || abi_min > ABI_VERSION {
            return Err(format!("manifest abi_min: {abi_min:#x} not satisfied by host {ABI_VERSION:#x}"));
        }
        let id = cstr(d, man + 44, 32)?;
        let name = cstr(d, man + 76, 48)?;
        if id.is_empty() || !id.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(&b)) {
            return Err(format!("manifest mod_id: {id:?} not [a-z0-9._-]+"));
        }
        if self.mods.iter().any(|m| m.id == id) {
            return Err(format!("manifest mod_id: {id} already loaded"));
        }
        if req & !FEATURES_HOST != 0 {
            return Err(format!("manifest required_features: {:#x} not provided by host {FEATURES_HOST:#x}", req));
        }
        if table_v != HOOK_TABLE_VERSION {
            return Err(format!("manifest hook_table_version: {table_v} (host has hooks-v{HOOK_TABLE_VERSION})"));
        }
        if state_bytes as u64 > 256 << 10 || state_bytes as u64 > mt.minimum() * 65536 {
            return Err(format!("manifest state_bytes: {state_bytes} beyond 256 KiB or the memory"));
        }
        if count > MAX_HOOKS {
            return Err(format!("manifest hook_count: {count} > {MAX_HOOKS}"));
        }
        if abi_max < ABI_VERSION {
            eprintln!("modhost: {id}: abi_max {abi_max:#x} below host {ABI_VERSION:#x} (loaded)");
        }
        // gd-ui-v1 / gd-lab-v1: a declared UI or LAB bit needs its ABI global, checked like gdmod_abi_version.
        for (bits, name, host) in [(pres::FEATURES_UI, "gdmod_ui_abi", pres::UI_ABI_VERSION), (pres::FEATURES_LAB, "gdmod_lab_abi", pres::LAB_ABI_VERSION)] {
            if (req | opt) & bits != 0 {
                let g = inst.get_global(&self.store, name).ok_or_else(|| format!("export {name}: missing (refuse)"))?;
                let v = g.get(&self.store).i32().ok_or_else(|| format!("export {name}: not an i32 global (refuse)"))? as u32;
                if v >> 16 != host >> 16 || v > host {
                    return Err(format!("{name}: {v:#x} not satisfied by host {host:#x}"));
                }
            }
        }
        let features = (req | opt) & FEATURES_HOST;
        let mut hooks = Vec::new();
        for i in 0..count as usize {
            let at = hooks_at + i * 20;
            let (addr, mode, prio, flags, idx) = (rd_u32(d, at)?, rd_u32(d, at + 4)?, rd_u32(d, at + 8)? as i32, rd_u32(d, at + 12)?, rd_u32(d, at + 16)?);
            let Some((row, sig_s, sig_src)) = hook_row(addr) else {
                return Err(format!("hook {i}: address {addr:#010x} is not in hooks-v1"));
            };
            if !WIRED.iter().any(|w| w.1 == addr) {
                return Err(format!("hook {i}: row {row} ({addr:#010x}) is not wired by this host yet (wired: {:?})", WIRED.iter().map(|w| w.0).collect::<Vec<_>>()));
            }
            if mode != PRE && mode != POST && mode != REPLACE {
                return Err(format!("hook {i}: mode {mode} invalid"));
            }
            let sig = parse_sig(sig_s).ok_or_else(|| format!("hook {i}: row {row} signature {sig_s:?} not understood"))?;
            if mode == REPLACE {
                if req & FEATURE_REPLACE == 0 {
                    return Err(format!("hook {i}: replace on {row}: GD_FEATURE_REPLACE not in required_features"));
                }
                if sig_src != "decomp" || addr == ADDR_FRAME || sig.ret == Some(Ty::Handle) {
                    return Err(format!("hook {i}: replace on {row}: not allowed on this row (sig_src {sig_src}, frame row or handle result)"));
                }
                if let Some((m, _)) = self.mods.iter().find_map(|m| m.hooks.iter().find(|h| h.addr == addr && h.mode == REPLACE).map(|h| (m, h))) {
                    return Err(format!("hook {i}: replace on {row}: {} already replaces it (one replace per address)", m.id));
                }
            }
            if !(-1000..=1000).contains(&prio) || flags != 0 {
                return Err(format!("hook {i}: priority {prio} / flags {flags} out of range"));
            }
            let func = table.get(&self.store, idx as u64).and_then(|r| r.as_func().and_then(|n| n.val().map(|f| (*f).clone()))).ok_or_else(|| format!("hook {i}: fn {idx} not a function in the table"))?;
            // the function's wasm signature against the row, checked now so a dispatch can never fail on a type
            let want = func_type(&sig, mode);
            if func.ty(&self.store) != want {
                return Err(format!("hook {i}: signature of fn {idx} does not match row {row} {sig_s} mode {mode} (want {want:?})"));
            }
            let n = self.mods.iter().flat_map(|m| m.hooks.iter()).chain(hooks.iter()).filter(|h: &&Hook| h.addr == addr && h.mode == mode).count();
            if mode != REPLACE && n >= MAX_PER_ADDR {
                return Err(format!("hook {i}: {row}: more than {MAX_PER_ADDR} hooks of mode {mode} on one address"));
            }
            hooks.push(Hook { addr, mode, priority: prio, index: i as u32, func });
        }
        let prof = if melee_core::frameprof::on() { melee_core::frameprof::mod_slot(&id) } else { 0 };
        self.mods.push(LoadedMod { id, name, version, digest: fnv64(bytes), disabled: false, features, hooks, prof });
        self.store.data_mut().features.push(features);
        self.rebuild();
        // an optional `gd_mod_loaded() -> ()` export runs once now (setup a mod wants before the first match, e.g.
        // the Envoy run's first plan); same fuel budget as a hook, a trap is logged and the mod stays loaded
        if let Ok(f) = inst.get_typed_func::<(), ()>(&self.store, "gd_mod_loaded") {
            self.store.data_mut().cur_module = self.mods.len() - 1;
            let _ = self.store.set_fuel(HOOK_FUEL);
            if let Err(e) = f.call(&mut self.store, ()) {
                eprintln!("modhost: {}: gd_mod_loaded trapped: {e}", self.mods.last().unwrap().id);
            }
        }
        Ok(self.mods.len() - 1)
    }

    /// Per-address chains in the spec's total order (priority, mod_id bytewise, declaration index); post is the reverse.
    fn rebuild(&mut self) {
        let mut all: Vec<(u32, u32, i32, String, u32, usize, Func)> = Vec::new();
        for (mi, m) in self.mods.iter().enumerate() {
            for h in &m.hooks {
                all.push((h.addr, h.mode, h.priority, m.id.clone(), h.index, mi, h.func));
            }
        }
        all.sort_by(|a, b| (a.2, a.3.as_bytes(), a.4).cmp(&(b.2, b.3.as_bytes(), b.4)));
        self.chains.clear();
        for h in &all {
            let c = self.chains.entry(h.0).or_insert_with(|| Chain { sig: row_sig(h.0).unwrap(), pre: Vec::new(), replace: None, post: Vec::new() });
            let e = Entry { module: h.5, func: h.6 };
            match h.1 {
                PRE => c.pre.push(e),
                POST => c.post.push(e),
                _ => c.replace = Some(e),
            }
        }
        for c in self.chains.values_mut() {
            c.post.reverse();
        }
    }

    pub fn host_data(&self) -> &HostData {
        self.store.data()
    }

    /// Addresses with at least one hook, sorted.
    pub fn hooked(&self) -> Vec<u32> {
        self.chains.keys().copied().collect()
    }
}

impl Default for Host {
    fn default() -> Self {
        Host::new()
    }
}

/// The store context hooks run in: the innermost `gd_orig_call_*` caller, else the host's store.
unsafe fn ctx(host: *mut Host) -> StoreContextMut<'static, HostData> {
    match CTX.last() {
        Some(&c) => (*(c as *mut Caller<'static, HostData>)).as_context_mut(),
        None => (*host).store.as_context_mut(),
    }
}

/// Run one hook with a fresh fuel budget; a trap disables that mod for the rest of the session (deterministic: fuel
/// and traps do not depend on the machine). None when the mod is disabled or trapped.
unsafe fn call_hook(host: *mut Host, e: Entry, args: &[Val], nres: usize) -> Option<Option<Val>> {
    if (&(*host).mods)[e.module].disabled {
        return None;
    }
    let mut c = ctx(host);
    let prev = c.data().cur_module;
    c.data_mut().cur_module = e.module;
    c.set_fuel(HOOK_FUEL).ok()?;
    let mut out = [Val::I32(0)];
    let prof = melee_core::frameprof::on().then(std::time::Instant::now);
    let r = e.func.call(&mut c, args, &mut out[..nres]);
    if let Some(t) = prof {
        let ns = t.elapsed().as_nanos() as u64;
        melee_core::frameprof::mod_ns((&(*host).mods)[e.module].prof, ns);
        if ns >= 1_000_000 {
            // a long hook call as a named phase (its host calls show inside it: fighter create, file loads)
            let id = (&(*host).mods)[e.module].id.clone();
            melee_core::frameprof::event(melee_core::frameprof::Kind::Note, melee_core::frameprof::Text::new().s("wasm hook ").s(&id), (ns / 1000) as u32, 0);
        }
    }
    let mut c = ctx(host);
    c.data_mut().cur_module = prev;
    let id = (&(*host).mods)[e.module].id.clone();
    // the fuel this hook used: a new peak is logged when it enters another eighth of the budget (a mod that runs out
    // traps and is disabled for the session, so the margin is worth knowing; MELEE_MOD_FUEL_LOG=1 logs every new peak)
    {
        static PEAK: [std::sync::atomic::AtomicU64; MAX_MODS] = [const { std::sync::atomic::AtomicU64::new(0) }; MAX_MODS];
        let used = HOOK_FUEL.saturating_sub(c.get_fuel().unwrap_or(HOOK_FUEL));
        let slot = &PEAK[e.module.min(MAX_MODS - 1)];
        let prev_peak = slot.load(std::sync::atomic::Ordering::Relaxed);
        if used > prev_peak {
            slot.store(used, std::sync::atomic::Ordering::Relaxed);
            static ALL: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
            if used * 8 / HOOK_FUEL != prev_peak * 8 / HOOK_FUEL || *ALL.get_or_init(|| std::env::var_os("MELEE_MOD_FUEL_LOG").is_some()) {
                eprintln!("modhost: {id}: hook fuel peak {used} of {HOOK_FUEL}");
            }
        }
    }
    for l in c.data_mut().log.drain(..) {
        eprintln!("modhost: {id}: {l}");
    }
    match r {
        Ok(()) => Some((nres == 1).then(|| out[0].clone())),
        Err(err) => {
            eprintln!("modhost: {id}: trapped ({err}); the mod is disabled for this session");
            (&mut (*host).mods)[e.module].disabled = true;
            None
        }
    }
}

/// Apply the action-state requests queued by the chain just run (game defaults: no flags, frame 0, rate 1, no blend).
/// Only a small set of common states that need no context is accepted, each only in its own ground / air situation.
unsafe fn apply_requests(host: *mut Host) {
    let reqs: Vec<(u32, i32)> = ctx(host).data_mut().requests.drain(..).collect();
    for (h, state) in reqs {
        let fp = fighter_ptr(h);
        if fp.is_null() {
            continue;
        }
        let grounded = (*fp).ground_or_air == melee_types::GA_Ground;
        let ok = match state {
            s if s == melee_types::ftCo_MS_Wait as i32 || s == melee_types::ftCo_MS_Squat as i32 || s == melee_types::ftCo_MS_SquatWait as i32 => grounded,
            s if s == melee_types::ftCo_MS_Fall as i32 || s == melee_types::ftCo_MS_FallSpecial as i32 => !grounded,
            _ => false,
        };
        if ok {
            melee_fighter::tu::fighter::Fighter_ChangeMotionState((*fp).gobj as _, state, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        } else {
            eprintln!("modhost: action request {state} for slot {} refused (allowed: Wait/Squat/SquatWait grounded, Fall/FallSpecial airborne)", (h & 0xFFFF) - 1);
        }
    }
}

fn ret_val(t: Ty, r: Ret, d: &mut HostData) -> Val {
    match (t, r) {
        (Ty::F32, Ret::F32(v)) => Val::F32(canon(v).into()),
        (_, Ret::I32(v)) => Val::I32(v),
        (Ty::Handle, Ret::Ptr(p)) => {
            let g = p as *mut HSD_GObj;
            let h = fighter_handle_of_gobj(g);
            Val::I32(if h != 0 { h } else if !g.is_null() && unsafe { (*g).classifier } as i32 == melee_types::HSD_GOBJ_CLASS_ITEM { item_handle(d, g) } else { gobj_handle(d, g) } as i32)
        }
        (Ty::F32, _) => Val::F32(0f32.into()),
        _ => Val::I32(0),
    }
}

fn val_ret(t: Option<Ty>, v: Option<Val>) -> Ret {
    match (t, v) {
        (None, _) => Ret::Void,
        (Some(Ty::F32), Some(Val::F32(x))) => Ret::F32(canon(x.into())),
        (Some(Ty::F32), _) => Ret::F32(0.0),
        (Some(_), Some(Val::I32(x))) => Ret::I32(x),
        (Some(_), _) => Ret::I32(0),
    }
}

/// The generic dispatcher (`melee_core::modhook::DISPATCH`): pre chain, then replace or the original, then post.
fn dispatch(addr: u32, args: &[Arg], orig: &mut dyn FnMut() -> Ret) -> Ret {
    unsafe {
        let Some(host) = HOST.as_deref_mut().map(|h| h as *mut Host) else { return orig() };
        let Some(chain) = (*host).chains.get(&addr).cloned() else { return orig() };
        // sub-fighters (Nana) are not addressable in v1: their activations run the original only
        if let Some(Arg::Fighter(g)) = args.first() {
            if fighter_handle_of_gobj(*g as *mut HSD_GObj) == 0 {
                return orig();
            }
        }
        {
            let mut c = ctx(host);
            let d = c.data_mut().depth.entry(addr).or_insert(0);
            if *d >= MAX_DEPTH {
                c.data_mut().deep_calls += 1;
                return orig();
            }
            *d += 1;
        }
        let mut spawn = false;
        let vals: Vec<Val> = {
            let mut c = ctx(host);
            let d = c.data_mut();
            args.iter().map(|a| match *a {
                Arg::Fighter(g) => Val::I32(fighter_handle_of_gobj(g as _) as i32),
                Arg::Item(g) => Val::I32(item_handle(d, g as _) as i32),
                Arg::Gobj(g) => Val::I32(gobj_handle(d, g as _) as i32),
                Arg::Spawn(p) => {
                    spawn = true;
                    d.spawns.push((p as usize, true));
                    Val::I32(((H_SPAWN << 16) | 1) as i32)
                }
                Arg::Ptr(_) => Val::I32(0),
                Arg::I32(v) => Val::I32(v),
                Arg::F32(v) => Val::F32(canon(v).into()),
            }).collect()
        };
        let set_spawn_pre = |pre: bool| if spawn { if let Some(s) = ctx(host).data_mut().spawns.last_mut() { s.1 = pre; } };
        // pre
        ctx(host).data_mut().set_return = None;
        let mut skipped = false;
        for e in &chain.pre {
            if let Some(Some(Val::I32(1))) = call_hook(host, *e, &vals, 1) {
                skipped = true; // GD_SKIP: the later pre hooks, the replace and the original do not run
                break;
            }
        }
        set_spawn_pre(false);
        let mut ret = if skipped {
            match chain.sig.ret {
                Some(Ty::Handle) | None => if chain.sig.ret.is_some() { Ret::Ptr(core::ptr::null_mut()) } else { Ret::Void },
                t => val_ret(t, ctx(host).data_mut().set_return.take()),
            }
        } else if let Some(e) = chain.replace {
            let f: *mut (dyn FnMut() -> Ret + '_) = orig;
            ctx(host).data_mut().origs.push(OrigFrame { f: core::mem::transmute(f), used: false });
            let r = call_hook(host, e, &vals, chain.sig.ret.is_some() as usize);
            let used = ctx(host).data_mut().origs.pop().is_some_and(|o| o.used);
            match r {
                Some(v) => val_ret(chain.sig.ret, v),
                // the replacing mod trapped: run the original, unless it already did
                None if !used => orig(),
                None => val_ret(chain.sig.ret, None),
            }
        } else {
            orig()
        };
        apply_requests(host);
        // post: void(args) or R(args, ret)
        for e in &chain.post {
            match chain.sig.ret {
                None => { call_hook(host, *e, &vals, 0); }
                Some(t) => {
                    let mut a = vals.clone();
                    a.push(ret_val(t, ret, ctx(host).data_mut()));
                    let r = call_hook(host, *e, &a, 1);
                    // a handle result (gd_gobj / gd_fighter rows) is passed through unchanged in this host
                    if t != Ty::Handle { if let Some(v) = r { ret = val_ret(Some(t), v); } }
                }
            }
        }
        apply_requests(host);
        let mut c = ctx(host);
        if spawn {
            c.data_mut().spawns.pop();
        }
        if let Some(d) = c.data_mut().depth.get_mut(&addr) {
            *d -= 1;
        }
        ret
    }
}

fn frame_hook(post: bool) {
    unsafe {
        let Some(host) = HOST.as_deref_mut().map(|h| h as *mut Host) else { return };
        if !post {
            let d = (*host).store.data_mut();
            d.frame += 1;
            d.items.clear();
            d.gobjs.clear();
            d.begin_hash();
        }
        let Some(chain) = (*host).chains.get(&ADDR_FRAME).cloned() else { return };
        if !post {
            for e in &chain.pre {
                if let Some(Some(Val::I32(1))) = call_hook(host, *e, &[], 1) {
                    break; // GD_SKIP: the later pre hooks do not run (the procs still run: frame skipping is not offered)
                }
            }
        } else {
            for e in &chain.post {
                call_hook(host, *e, &[], 0);
            }
        }
        apply_requests(host);
        if post {
            if let Some(kind) = ctx(host).data_mut().stage_switch.take() {
                let rc = melee_modes::tu::stage_hop::switch_to(kind);
                if rc != 0 { eprintln!("modhost: stage switch to {kind} refused ({rc})"); }
                ctx(host).data_mut().stage_switch_rc = rc;
            }
            // a queued tag swap and the transition clock: between logic frames, outside the procs
            melee_modes::tu::tag_swap::tick();
            melee_modes::tu::petey::tick();
            melee_modes::tu::envoy::tick();
        }
    }
}

/// The damage row through the generic dispatcher (the site keeps its own pointer: it hands a `&mut Fighter`).
fn damage_hook(fp: &mut Fighter, d: f32, orig: &mut dyn FnMut() -> Ret) {
    if std::env::var_os("MELEE_MODHOST_TRACE").is_some() { eprintln!("modhost: damage slot {} {d}", fp.player_idx); }
    dispatch(ADDR_DAMAGE, &[Arg::Fighter(fp.gobj as _), Arg::F32(d)], orig);
}

/// fighter.c's `nh_Mix` (the NHV1 step)
fn hash_mix(mut h: u32, v: u32) -> u32 {
    h ^= v;
    h = h.wrapping_mul(0x0100_0193);
    h ^ (h >> 15)
}

impl HostData {
    /// Frame begin: no mod word yet.
    fn begin_hash(&mut self) {
        self.hash_acc = 0;
        self.hash_any = false;
    }
    /// This frame's mod word, 0 when no mod gave one (never 0 otherwise).
    pub fn hash_word(&self) -> u32 {
        if self.hash_any { self.hash_acc | 1 } else { 0 }
    }
}

/// NHV1 word 0 with a mod word mixed in (unchanged for 0).
fn mix_session(nhv1: u32, word: u32) -> u32 {
    if word == 0 { nhv1 } else { hash_mix(nhv1, word) }
}

/// The mods' word for the current logic frame (`gd_hash_word`), 0 when no host or no mod gave one this frame.
pub fn hash_word() -> u32 {
    unsafe { HOST.as_deref() }.map_or(0, |h| h.store.data().hash_word())
}

/// The hash a netplay session compares: the engine's NHV1 word with the mods' word mixed in when there is one (as the C
/// mixes `ScriptGame_BuildHashWord` only when it is nonzero), so a vanilla match and every golden hash are unchanged and
/// NHV1 itself stays the pure cross-engine format.
pub fn session_hash(nhv1: u32) -> u32 {
    mix_session(nhv1, hash_word())
}

/// [`session_hash`] with a given mod word (a netplay test perturbs the word of one peer).
pub fn session_hash_with(nhv1: u32, word: u32) -> u32 {
    mix_session(nhv1, word)
}

/// The loaded host, if any.
pub fn host() -> Option<&'static Host> {
    unsafe { HOST.as_deref() }
}

/// Test entry: run the generic dispatcher for `addr` (the game sites call it through `melee_core::modhook`).
pub fn dispatch_for_test(addr: u32, args: &[Arg], orig: &mut dyn FnMut() -> Ret) -> Ret {
    dispatch(addr, args, orig)
}

/// `MELEE_WASM_MODS`: the mods directory, None when unset / empty / `0` (off by default).
/// A relative dir is made absolute against the working directory, else (when it is not there) against the exe's
/// folder, so a game started from another folder still finds its mods and their art.
pub fn dir_from_env() -> Option<std::path::PathBuf> {
    let v = std::env::var("MELEE_WASM_MODS").ok()?;
    if v.is_empty() || v == "0" {
        return None;
    }
    let p = std::path::PathBuf::from(v);
    if p.is_absolute() {
        return Some(p);
    }
    let from_exe = std::env::current_exe().ok().and_then(|e| e.parent().map(|d| d.join(&p)));
    Some(match (std::path::absolute(&p).ok(), from_exe) {
        (Some(a), _) if a.is_dir() => a,
        (_, Some(e)) if e.is_dir() => e,
        (a, _) => a.unwrap_or(p),
    })
}

/// Load every `*.wasm` of `dir` and install the hook pointers the loaded set needs. Call on the game thread inside the
/// engine's `GameScope` (the host must live in the arena to roll back). A refused mod fails the whole call: the match
/// must not start without a mod it was asked for.
pub fn install_dir(dir: &std::path::Path) -> Result<(), String> {
    let mut files: Vec<_> = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("wasm")))
        .collect();
    files.sort();
    let mut h = Box::new(Host::new());
    for f in &files {
        let b = std::fs::read(f).map_err(|e| format!("{}: {e}", f.display()))?;
        // a gd_ui presentation mod is sim-inert: the play flow's presentation host loads it (ui::PresHost), not this one
        if ui::is_presentation(&b) {
            continue;
        }
        h.load(&b).map_err(|e| format!("{}: refused: {e}", f.display()))?;
    }
    for m in &h.mods {
        eprintln!("modhost: loaded {} \"{}\" v{:#x} fnv64 {:016x} ({} hooks, features {:#x})", m.id, m.name, m.version, m.digest, m.hooks.len(), m.features);
    }
    install(h);
    Ok(())
}

/// Make `h` the active host and install exactly the hook pointers its chains need.
pub fn install(h: Box<Host>) {
    unsafe {
        let frame = !h.mods.is_empty(); // gd_match_frame and the per-frame handle tables run whenever a mod is loaded
        let hooked: Vec<u32> = h.hooked().into_iter().filter(|&a| a != ADDR_FRAME && a != ADDR_DAMAGE).collect();
        let dmg = h.chains.contains_key(&ADDR_DAMAGE);
        HOST = Some(h);
        melee_modes::tu::gmscene::MOD_FRAME = frame.then_some(frame_hook as fn(bool));
        melee_fighter::hooks::MOD_TAKE_DAMAGE = dmg.then_some(damage_hook as fn(&mut Fighter, f32, &mut dyn FnMut() -> Ret));
        melee_core::modhook::DISPATCH = (!hooked.is_empty()).then_some(dispatch as melee_core::modhook::Dispatch);
        melee_core::modhook::HOOKED = hooked;
    }
}

/// `MELEE_WASM_MODS` -> [`install_dir`]; nothing at all when unset.
pub fn install_from_env() -> Result<(), String> {
    match dir_from_env() {
        Some(d) => install_dir(&d),
        None => Ok(()),
    }
}

/// Loaded mods as (id, version, fnv64), in load order: the netplay handshake's mod list.
pub fn loaded_ids() -> Vec<(String, u32, u64)> {
    host().map(|h| h.mods.iter().map(|m| (m.id.clone(), m.version, m.digest)).collect()).unwrap_or_default()
}

#[cfg(test)]
mod tests;
