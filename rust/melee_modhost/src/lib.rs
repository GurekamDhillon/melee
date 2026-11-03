//! The WebAssembly mod host (mod-c-v1 ABI 1.0, hook table hooks-v1): sandboxed code mods for the native engine.
//!
//! Notes: docs/rust-port/notes/modhost.md. Spec: docs/specs/mod-c-v1.md (declarations: docs/specs/gdmod.h).
//!
//! * Off by default. `MELEE_WASM_MODS=<dir>` loads every `<dir>/*.wasm` (sorted by mod id, so the order on disk never
//!   matters). Unset, empty or `0`: nothing is loaded and no hook pointer is installed, so the game runs exactly as
//!   before (identical hashes by construction: every hook site is a `None` check).
//! * Runtime: wasmi (an interpreter in pure Rust) with its `deterministic` profile (NaN canonicalisation), fuel
//!   metering (a deterministic watchdog: a hook that runs out of fuel traps at the same instruction on every machine),
//!   no SIMD, no threads, fixed memory, no start function, imports only from module `gd`.
//! * Rollback: the host (wasmi engine, store, instances, linear memories) is created inside the engine's
//!   `GameScope`, so every byte of it lives in the game arena / statics that `melee_engine::snapshot` saves and loads.
//!   A rollback restores the mods' memories, globals and fuel with the rest of the game; mods need no save/load code.
//! * Hooks: every row of hooks-v1 is accepted by the address check, but this host wires the rows listed in
//!   [`WIRED`]; a mod that declares any other row is refused (never a silent no-op).
#![allow(static_mut_refs)]

use melee_types::Fighter;
use wasmi::{Caller, Config, Engine, Extern, Func, Linker, Memory, Module, Store, TypedFunc};

pub const ABI_VERSION: u32 = 1 << 16; // 1.0
pub const HOOK_TABLE_VERSION: u32 = 1;
pub const MANIFEST_MAGIC: u32 = 0x444D_4447; // 'GDMD'
pub const MANIFEST_SIZE: u32 = 124; // gd_manifest on wasm32
pub const MAX_MODS: usize = 16;
pub const MAX_HOOKS: u32 = 256;
pub const MAX_PAGES: u64 = 16; // 1 MiB
/// Fuel per hook invocation (wasmi fuel units, about one per instruction). Running out traps and disables the mod.
pub const HOOK_FUEL: u64 = 1_000_000;
const FEATURES_HOST: u32 = 1 << 3; // GD_FEATURE_LOG only

pub const PRE: u32 = 1;
pub const POST: u32 = 2;
pub const REPLACE: u32 = 3;

/// hooks-v1 rows this host implements: (name, address).
pub const WIRED: &[(&str, u32)] = &[("frame.run_procs", 0x8039_0CFC), ("damage.fighter_take_damage", 0x8006_CC7C)];
const ADDR_FRAME: u32 = 0x8039_0CFC;
const ADDR_DAMAGE: u32 = 0x8006_CC7C;

const HOOKS_V1: &str = include_str!("../../../docs/specs/hooks-v1.tsv");

/// The address of hooks-v1 row `addr` -> (name, sig_src), None when the address is not in the table.
pub fn hook_row(addr: u32) -> Option<(&'static str, &'static str)> {
    HOOKS_V1.lines().filter(|l| !l.starts_with('#') && !l.starts_with("name\t")).find_map(|l| {
        let c: Vec<&str> = l.split('\t').collect();
        let a = u32::from_str_radix(c.get(1)?.trim_start_matches("0x"), 16).ok()?;
        (a == addr).then(|| (c[0], c.get(7).copied().unwrap_or("")))
    })
}

/// Per-store data the accessors see.
#[derive(Default)]
pub struct HostData {
    /// `gd_match_frame`: frame.begin calls since the host was loaded (rolls back with the arena).
    pub frame: i32,
    /// Messages from `gd_log` (drained to stderr after each hook chain).
    pub log: Vec<String>,
    /// Host accessor calls with a stale or foreign handle (spec section 3.3: counted, not an error).
    pub bad_handles: u32,
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
    hooks: Vec<Hook>,
}

/// One entry of a per-address chain.
#[derive(Clone)]
struct Entry {
    module: usize,
    func: Func,
}

pub struct Host {
    store: Store<HostData>,
    pub mods: Vec<LoadedMod>,
    frame_pre: Vec<(Entry, TypedFunc<(), i32>)>,
    frame_post: Vec<(Entry, TypedFunc<(), ()>)>,
    dmg_pre: Vec<(Entry, TypedFunc<(u32, f32), i32>)>,
    dmg_post: Vec<(Entry, TypedFunc<(u32, f32), ()>)>,
}

static mut HOST: Option<Box<Host>> = None;

pub fn engine_config() -> Config {
    let mut c = Config::default();
    c.consume_fuel(true);
    c.wasm_multi_memory(false);
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

fn mem_of(caller: &Caller<'_, HostData>) -> Option<Memory> {
    caller.get_export("memory").and_then(Extern::into_memory)
}

/// Floats a mod writes or reads are NaN-canonicalised at the boundary (spec section 4).
fn canon(x: f32) -> f32 {
    if x.is_nan() { f32::from_bits(0x7FC0_0000) } else { x }
}

/// Fighter handle encoding: (generation 1 << 16) + slot + 1. Resolved each call; stale -> null.
fn handle_of(slot: i32) -> u32 {
    (1 << 16) | (slot as u32 + 1)
}

fn fighter_ptr(h: u32) -> *mut Fighter {
    if h >> 16 != 1 {
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

fn with_fighter<R>(c: &mut Caller<'_, HostData>, h: u32, dflt: R, f: impl FnOnce(&mut Fighter) -> R) -> R {
    let p = fighter_ptr(h);
    if p.is_null() {
        c.data_mut().bad_handles += 1;
        return dflt;
    }
    f(unsafe { &mut *p })
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
    w!("gd_fighter_from_slot", |slot: i32| -> u32 {
        if (0..4).contains(&slot) && !fighter_ptr(handle_of(slot)).is_null() { handle_of(slot) } else { 0 }
    });
    w!("gd_player_present", |slot: i32| -> i32 { ((0..4).contains(&slot) && !fighter_ptr(handle_of(slot)).is_null()) as i32 });
    w!("gd_fighter_slot", |mut c: Caller<'_, HostData>, h: u32| -> i32 { with_fighter(&mut c, h, -1, |f| f.player_id as i32) });
    w!("gd_fighter_kind", |mut c: Caller<'_, HostData>, h: u32| -> i32 { with_fighter(&mut c, h, -1, |f| f.kind as i32) });
    w!("gd_fighter_get_action_state", |mut c: Caller<'_, HostData>, h: u32| -> i32 { with_fighter(&mut c, h, -1, |f| f.motion_id as i32) });
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
    // gd_fighter_read(h, gd_fighter_view* out): 80 bytes, little-endian
    w!("gd_fighter_read", |mut c: Caller<'_, HostData>, h: u32, out: u32| {
        let mut v = [0u32; 20];
        let ok = with_fighter(&mut c, h, false, |f| {
            let fl = [f.cur_pos.x, f.cur_pos.y, f.cur_pos.z, f.self_vel.x, f.self_vel.y, f.self_vel.z, f.x8c_kb_vel.x, f.x8c_kb_vel.y, f.x8c_kb_vel.z, f.facing_dir, f.dmg.x1830_percent];
            for (i, x) in fl.iter().enumerate() { v[i] = canon(*x).to_bits(); }
            v[11] = f.motion_id as u32;
            v[12] = f.kind as u32;
            v[13] = f.player_id as u32;
            true
        });
        let _ = ok;
        if let Some(m) = mem_of(&c) {
            let d = m.data_mut(&mut c);
            let o = out as usize;
            if let Some(dst) = d.get_mut(o..o + 80) {
                for (i, w) in v.iter().enumerate() { dst[i * 4..i * 4 + 4].copy_from_slice(&w.to_le_bytes()); }
            }
        }
    });
    // gd_fighter_write(h, const gd_fighter_view* in, mask): POS 1, VEL 2, KB_VEL 4, FACING 8, PERCENT 16
    w!("gd_fighter_write", |mut c: Caller<'_, HostData>, h: u32, inp: u32, mask: u32| {
        let Some(m) = mem_of(&c) else { return };
        let o = inp as usize;
        let Some(src) = m.data(&c).get(o..o + 80) else { return };
        let mut v = [0f32; 11];
        for (i, x) in v.iter_mut().enumerate() { *x = canon(f32::from_le_bytes(src[i * 4..i * 4 + 4].try_into().unwrap())); }
        with_fighter(&mut c, h, (), |f| {
            if mask & 1 != 0 { f.cur_pos.x = v[0]; f.cur_pos.y = v[1]; f.cur_pos.z = v[2]; }
            if mask & 2 != 0 { f.self_vel.x = v[3]; f.self_vel.y = v[4]; f.self_vel.z = v[5]; }
            if mask & 4 != 0 { f.x8c_kb_vel.x = v[6]; f.x8c_kb_vel.y = v[7]; f.x8c_kb_vel.z = v[8]; }
            if mask & 8 != 0 && (v[9] == 1.0 || v[9] == -1.0) { f.facing_dir = v[9]; }
            if mask & 16 != 0 { f.dmg.x1830_percent = v[10]; }
        });
    });
    // the game's RNG (HSD_Randi / HSD_Randf on the logic seed): the draws are game state and roll back with it
    w!("gd_rand_u32", |bound: u32| -> u32 {
        if bound == 0 { 0 } else { melee_runtime::tu::random::global::HSD_Randi(bound.min(i32::MAX as u32) as i32) as u32 }
    });
    w!("gd_rand_f32", || -> f32 { melee_runtime::tu::random::global::HSD_Randf() });
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

impl Host {
    pub fn new() -> Host {
        let engine = Engine::new(&engine_config());
        Host { store: Store::new(&engine, HostData::default()), mods: Vec::new(), frame_pre: Vec::new(), frame_post: Vec::new(), dmg_pre: Vec::new(), dmg_post: Vec::new() }
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
        let (abi_min, abi_max, table_v, version, req, _opt, state_bytes, count, hooks_at) = (f(2)?, f(3)?, f(4)?, f(5)?, f(6)?, f(7)?, f(8)?, f(9)?, f(10)? as usize);
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
        let mut hooks = Vec::new();
        for i in 0..count as usize {
            let at = hooks_at + i * 20;
            let (addr, mode, prio, flags, idx) = (rd_u32(d, at)?, rd_u32(d, at + 4)?, rd_u32(d, at + 8)? as i32, rd_u32(d, at + 12)?, rd_u32(d, at + 16)?);
            let Some((row, _sig)) = hook_row(addr) else {
                return Err(format!("hook {i}: address {addr:#010x} is not in hooks-v1"));
            };
            if !WIRED.iter().any(|w| w.1 == addr) {
                return Err(format!("hook {i}: row {row} ({addr:#010x}) is not wired by this host yet (wired: {:?})", WIRED.iter().map(|w| w.0).collect::<Vec<_>>()));
            }
            if mode == REPLACE {
                return Err(format!("hook {i}: replace on {row}: GD_FEATURE_REPLACE not provided by this host"));
            }
            if mode != PRE && mode != POST {
                return Err(format!("hook {i}: mode {mode} invalid"));
            }
            if !(-1000..=1000).contains(&prio) || flags != 0 {
                return Err(format!("hook {i}: priority {prio} / flags {flags} out of range"));
            }
            let func = table.get(&self.store, idx as u64).and_then(|r| r.as_func().and_then(|n| n.val().map(|f| (*f).clone()))).ok_or_else(|| format!("hook {i}: fn {idx} not a function in the table"))?;
            hooks.push(Hook { addr, mode, priority: prio, index: i as u32, func });
        }
        // signatures per row, checked now so a dispatch can never fail on a type
        for h in &hooks {
            let s = &self.store;
            let ok = match (h.addr, h.mode) {
                (ADDR_FRAME, PRE) => h.func.typed::<(), i32>(s).is_ok(),
                (ADDR_FRAME, POST) => h.func.typed::<(), ()>(s).is_ok(),
                (ADDR_DAMAGE, PRE) => h.func.typed::<(u32, f32), i32>(s).is_ok(),
                (ADDR_DAMAGE, POST) => h.func.typed::<(u32, f32), ()>(s).is_ok(),
                _ => false,
            };
            if !ok {
                return Err(format!("hook {}: signature does not match row {:#010x} mode {}", h.index, h.addr, h.mode));
            }
        }
        self.mods.push(LoadedMod { id, name, version, digest: fnv64(bytes), disabled: false, hooks });
        self.rebuild();
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
        let s = &self.store;
        let pick = |addr: u32, mode: u32| all.iter().filter(move |h| h.0 == addr && h.1 == mode).map(|h| Entry { module: h.5, func: h.6 });
        self.frame_pre = pick(ADDR_FRAME, PRE).map(|e| { let t = e.func.typed(s).unwrap(); (e, t) }).collect();
        self.frame_post = pick(ADDR_FRAME, POST).map(|e| { let t = e.func.typed(s).unwrap(); (e, t) }).collect();
        self.dmg_pre = pick(ADDR_DAMAGE, PRE).map(|e| { let t = e.func.typed(s).unwrap(); (e, t) }).collect();
        self.dmg_post = pick(ADDR_DAMAGE, POST).map(|e| { let t = e.func.typed(s).unwrap(); (e, t) }).collect();
        self.frame_post.reverse();
        self.dmg_post.reverse();
    }

    /// Run one hook body with a fresh fuel budget; a trap disables that mod for the rest of the session (deterministic:
    /// fuel and traps do not depend on the machine). Returns the hook's result, None when skipped or trapped.
    fn guard<R>(&mut self, module: usize, f: impl FnOnce(&mut Store<HostData>) -> Result<R, wasmi::Error>) -> Option<R> {
        if self.mods[module].disabled {
            return None;
        }
        self.store.set_fuel(HOOK_FUEL).ok()?;
        let r = f(&mut self.store);
        for l in self.store.data_mut().log.drain(..) {
            eprintln!("modhost: {}: {l}", self.mods[module].id);
        }
        match r {
            Ok(v) => Some(v),
            Err(e) => {
                eprintln!("modhost: {}: trapped ({e}); the mod is disabled for this session", self.mods[module].id);
                self.mods[module].disabled = true;
                None
            }
        }
    }

    pub fn run_frame(&mut self, post: bool) {
        if !post {
            self.store.data_mut().frame += 1;
            for (e, t) in self.frame_pre.clone() {
                if self.guard(e.module, |s| t.call(s, ())) == Some(1) {
                    break; // GD_SKIP: the later pre hooks do not run (the procs still run: frame skipping is not offered)
                }
            }
        } else {
            for (e, t) in self.frame_post.clone() {
                self.guard(e.module, |s| t.call(s, ()));
            }
        }
    }

    /// The damage row; for pre returns true when a hook returned GD_SKIP.
    pub fn damage(&mut self, slot: i32, d: f32, post: bool) -> bool {
        let h = handle_of(slot);
        let d = canon(d);
        if !post {
            for (e, t) in self.dmg_pre.clone() {
                if self.guard(e.module, |s| t.call(s, (h, d))) == Some(1) {
                    return true;
                }
            }
        } else {
            for (e, t) in self.dmg_post.clone() {
                self.guard(e.module, |s| t.call(s, (h, d)));
            }
        }
        false
    }

    pub fn host_data(&self) -> &HostData {
        self.store.data()
    }

}

impl Default for Host {
    fn default() -> Self {
        Host::new()
    }
}

fn frame_hook(post: bool) {
    unsafe {
        if let Some(h) = HOST.as_mut() {
            h.run_frame(post)
        }
    }
}

fn damage_hook(fp: &mut Fighter, d: f32, post: bool) -> bool {
    let slot = fp.player_id as i32;
    if std::env::var_os("MELEE_MODHOST_TRACE").is_some() { eprintln!("modhost: damage slot {slot} {d} post {post}"); }
    // sub-fighters (Nana) are not addressable in v1: their damage runs no hook
    if fp.is_sub_fighter() != 0 {
        return false;
    }
    unsafe { HOST.as_mut().map_or(false, |h| h.damage(slot, d, post)) }
}

/// The loaded host, if any.
pub fn host() -> Option<&'static Host> {
    unsafe { HOST.as_deref() }
}

/// `MELEE_WASM_MODS`: the mods directory, None when unset / empty / `0` (off by default).
pub fn dir_from_env() -> Option<std::path::PathBuf> {
    let v = std::env::var("MELEE_WASM_MODS").ok()?;
    (!v.is_empty() && v != "0").then(|| v.into())
}

/// Load every `*.wasm` of `dir` (sorted by mod id after loading, so load order never matters) and install the hook
/// pointers the loaded set needs. Call on the game thread inside the engine's `GameScope` (the host must live in the
/// arena to roll back). A refused mod fails the whole call: the match must not start without a mod it was asked for.
pub fn install_dir(dir: &std::path::Path) -> Result<(), String> {
    let mut files: Vec<_> = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("wasm")))
        .collect();
    files.sort();
    let mut h = Box::new(Host::new());
    for f in &files {
        let b = std::fs::read(f).map_err(|e| format!("{}: {e}", f.display()))?;
        h.load(&b).map_err(|e| format!("{}: refused: {e}", f.display()))?;
    }
    for m in &h.mods {
        eprintln!("modhost: loaded {} \"{}\" v{:#x} fnv64 {:016x} ({} hooks)", m.id, m.name, m.version, m.digest, m.hooks.len());
    }
    install(h);
    Ok(())
}

/// Make `h` the active host and install exactly the hook pointers its chains need.
pub fn install(h: Box<Host>) {
    unsafe {
        let frame = !h.mods.is_empty(); // gd_match_frame counts frames whenever a mod is loaded
        let dmg = !h.dmg_pre.is_empty() || !h.dmg_post.is_empty();
        HOST = Some(h);
        melee_modes::tu::gmscene::MOD_FRAME = frame.then_some(frame_hook as fn(bool));
        melee_fighter::hooks::MOD_TAKE_DAMAGE = dmg.then_some(damage_hook as fn(&mut Fighter, f32, bool) -> bool);
    }
}

/// `MELEE_WASM_MODS` -> [`install_dir`]; nothing at all when unset.
pub fn install_from_env() -> Result<(), String> {
    match dir_from_env() {
        Some(d) => install_dir(&d),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests;
