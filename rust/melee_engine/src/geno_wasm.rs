//! geno 12: a define's fighter code as a wasm module (the `wasm` block), the compiled replacement of the geno 10
//! `lua` block. Same contract as melee_script::geno::code (geno_lua_core): the game half hands a call its io block of
//! GENO_LUA_IO_WORDS words and gets a fault code back; the only state is the declared slots in that block, so a callback
//! cannot keep anything between calls (each call runs in a fresh instance of the module) and a rollback needs nothing
//! beyond the game state. The ctx surface is module `gd_geno`, one import per ctx value / command, each with the Lua
//! version's checks and the same command encoding, so a module that computes what the Lua did (in f64, as Lua
//! numbers are) writes the same io words.
//!
//! Module rules: imports only `gd_geno` functions; no start section; an exported `memory` with a maximum of at most
//! 16 pages; each callback an exported `() -> ()` function. Fuel per call: FUEL (running out is
//! GENO_LUA_FAULT_INSNS, the instruction budget; units differ from Lua's count, no shipped module comes near either).
use melee_script::geno::consts::*;
use melee_script::geno::code::{FighterCode, GLUA_BOOL, GLUA_FLOAT, GLUA_INT};
use wasmi::{Caller, Config, Engine, Extern, Linker, Module, Store};

/// Fuel for one callback.
pub const FUEL: u64 = 200_000;
const MAX_PAGES: u32 = 16;

// ---- the ctx keys of `gd_geno.ctx(key) -> f64` (gd_geno.h): ctx.self.* and ctx.input.* of the Lua version
pub const K_AIR: i32 = 0;
pub const K_ANIM_ENDED: i32 = 1;
pub const K_FACING: i32 = 2;
pub const K_PERCENT: i32 = 3;
pub const K_X: i32 = 4;
pub const K_Y: i32 = 5;
pub const K_VEL_X: i32 = 6;
pub const K_VEL_Y: i32 = 7;
pub const K_ACTION_FRAME: i32 = 8;
pub const K_MOTION: i32 = 9;
pub const K_HIT_DAMAGE: i32 = 10;
pub const K_HIT_FROM: i32 = 11;
pub const K_COUNTERED: i32 = 12;
pub const K_KI: i32 = 13;
pub const K_COMBO_LINKS: i32 = 14;
pub const K_CONNECTED: i32 = 15;
pub const K_STICK_X: i32 = 16;
pub const K_STICK_Y: i32 = 17;
pub const K_STICK_FWD: i32 = 18;
/// 20 + 2 * button: held; 21 + 2 * button: pressed (button 0 attack, 1 special, 2 jump, 3 shield, 4 grab, 5 blast)
pub const K_BUTTONS: i32 = 20;
const BTN: [u32; 6] = [GENO_BTN_ATTACK, GENO_BTN_SPECIAL, GENO_BTN_JUMP, GENO_BTN_SHIELD, GENO_BTN_GRAB, GENO_BTN_BLAST];

struct Ctx {
    io: [u32; GENO_LUA_IO_WORDS as usize],
    ncmd: usize,
    fault: u32,
    msg: &'static str,
    slot_name: Vec<String>,
    slot_type: Vec<i32>,
    targets: Vec<String>,
}

type Res<T> = Result<T, wasmi::Error>;

fn fault<T>(c: &mut Caller<'_, Ctx>, code: u32, msg: &'static str) -> Res<T> {
    let d = c.data_mut();
    if d.fault == 0 {
        d.fault = code;
        d.msg = msg;
    }
    Err(wasmi::Error::new(msg))
}

/// glua_checkfinite
fn finite(c: &mut Caller<'_, Ctx>, v: f64, lim: f64, what: &'static str) -> Res<f32> {
    if v.is_nan() || v.abs() > lim {
        return fault(c, GENO_LUA_FAULT_COMMAND, what);
    }
    Ok(v as f32)
}

/// glua_cmd
fn cmd(c: &mut Caller<'_, Ctx>, op: u32, a: u32, b: u32) -> Res<()> {
    if c.data().ncmd >= GENO_LUA_MAX_CMDS as usize {
        return fault(c, GENO_LUA_FAULT_CMDS, "more commands than one call may queue");
    }
    let d = c.data_mut();
    let at = GENO_LUA_IO_CMDS as usize + 3 * d.ncmd;
    d.io[at] = op;
    d.io[at + 1] = a;
    d.io[at + 2] = b;
    d.ncmd += 1;
    d.io[GENO_LUA_IO_NCMDS as usize] = d.ncmd as u32;
    Ok(())
}

/// The bytes [ptr, ptr + len) of the module's memory (a name), up to the first NUL as the Lua side's strcmp sees it.
fn name(c: &mut Caller<'_, Ctx>, ptr: i32, len: i32) -> Res<Vec<u8>> {
    let Some(Extern::Memory(m)) = c.get_export("memory") else { return fault(c, GENO_LUA_FAULT_ERROR, "no exported memory") };
    let d = m.data(&*c);
    let (p, n) = (ptr as u32 as usize, len as u32 as usize);
    match d.get(p..p.saturating_add(n)) {
        Some(b) if n <= 64 => Ok(b.split(|&x| x == 0).next().unwrap_or(&[]).to_vec()),
        _ => fault(c, GENO_LUA_FAULT_COMMAND, "a name outside the module's memory"),
    }
}

fn linker(engine: &Engine) -> Res<Linker<Ctx>> {
    let mut l = Linker::<Ctx>::new(engine);
    l.func_wrap("gd_geno", "ctx", |mut c: Caller<'_, Ctx>, key: i32| -> Res<f64> {
        let io = &c.data().io;
        let r = |i: u32| io[i as usize];
        let (fl, held, pressed) = (r(GENO_LUA_IO_FLAGS), r(GENO_LUA_IO_HELD), r(GENO_LUA_IO_PRESSED));
        let f = |i: u32| f32::from_bits(r(i)) as f64;
        let b = |v: bool| v as i32 as f64;
        Ok(match key {
            K_AIR => b(fl & 1 != 0),
            K_ANIM_ENDED => b(fl & 4 != 0),
            K_FACING => if fl & 2 != 0 { -1.0 } else { 1.0 },
            K_PERCENT => f(GENO_LUA_IO_PERCENT),
            K_X => f(GENO_LUA_IO_POS_X),
            K_Y => f(GENO_LUA_IO_POS_Y),
            K_VEL_X => f(GENO_LUA_IO_VEL_X),
            K_VEL_Y => f(GENO_LUA_IO_VEL_Y),
            K_ACTION_FRAME => r(GENO_LUA_IO_ACTION_FRAME) as i32 as f64,
            K_MOTION => r(GENO_LUA_IO_MOTION) as i32 as f64,
            K_HIT_DAMAGE => f(GENO_LUA_IO_HIT_DAMAGE),
            K_HIT_FROM => match r(GENO_LUA_IO_HIT_FROM) {
                1 => 1.0,
                2 => -1.0,
                _ => 0.0,
            },
            K_COUNTERED => r(GENO_LUA_IO_COUNTERED) as i32 as f64,
            K_KI => (r(GENO_LUA_IO_KI) as i32) as f64 / 100.0,
            K_COMBO_LINKS => r(GENO_LUA_IO_LINKS) as i32 as f64,
            K_CONNECTED => r(GENO_LUA_IO_CONNECTED) as i32 as f64,
            K_STICK_X => f(GENO_LUA_IO_STICK_X),
            K_STICK_Y => f(GENO_LUA_IO_STICK_Y),
            K_STICK_FWD => {
                let sx = f32::from_bits(r(GENO_LUA_IO_STICK_X));
                (if fl & 2 != 0 { -sx } else { sx }) as f64
            }
            k if (K_BUTTONS..K_BUTTONS + 12).contains(&k) => {
                let bit = BTN[((k - K_BUTTONS) / 2) as usize];
                b(if (k - K_BUTTONS) % 2 == 0 { held } else { pressed } & bit != 0)
            }
            _ => return fault(&mut c, GENO_LUA_FAULT_ERROR, "gd_geno.ctx: no such key"),
        })
    })?;
    l.func_wrap("gd_geno", "slot", |mut c: Caller<'_, Ctx>, ptr: i32, len: i32| -> Res<i32> {
        let k = name(&mut c, ptr, len)?;
        match c.data().slot_name.iter().position(|n| n.as_bytes() == k.as_slice()) {
            Some(i) => Ok(i as i32),
            None => fault(&mut c, GENO_LUA_FAULT_STATE, "ctx.state: no such slot is declared"),
        }
    })?;
    // glua_state_index: an int slot as its integer, a float slot widened, a bool 0 / 1
    l.func_wrap("gd_geno", "get", |mut c: Caller<'_, Ctx>, slot: i32| -> Res<f64> {
        let d = c.data();
        if slot < 0 || slot as usize >= d.slot_type.len() {
            return fault(&mut c, GENO_LUA_FAULT_STATE, "ctx.state: no such slot is declared");
        }
        let w = d.io[GENO_LUA_IO_STATE as usize + slot as usize];
        Ok(match d.slot_type[slot as usize] {
            GLUA_INT => w as i32 as f64,
            GLUA_FLOAT => f32::from_bits(w) as f64,
            _ => (w != 0) as i32 as f64,
        })
    })?;
    // glua_state_newindex: an int slot takes an integral value that fits 32 bits (lua_tointegerx), a float slot a finite
    // value (|v| <= 3e38, stored as f32), a bool slot 0 or 1
    l.func_wrap("gd_geno", "set", |mut c: Caller<'_, Ctx>, slot: i32, v: f64| -> Res<()> {
        let n = c.data().slot_type.len();
        if slot < 0 || slot as usize >= n {
            return fault(&mut c, GENO_LUA_FAULT_STATE, "ctx.state: no such slot is declared");
        }
        let w = match c.data().slot_type[slot as usize] {
            GLUA_INT => {
                if v.fract() != 0.0 || !v.is_finite() || v < i32::MIN as f64 || v > i32::MAX as f64 {
                    return fault(&mut c, GENO_LUA_FAULT_STATE, "ctx.state: an int slot takes an integer that fits 32 bits");
                }
                v as i32 as u32
            }
            GLUA_FLOAT => {
                if v.is_nan() || v.abs() > 3.0e38 {
                    return fault(&mut c, GENO_LUA_FAULT_STATE, "ctx.state: a float slot takes a finite value");
                }
                (v as f32).to_bits()
            }
            GLUA_BOOL => {
                if v != 0.0 && v != 1.0 {
                    return fault(&mut c, GENO_LUA_FAULT_STATE, "ctx.state: a bool slot takes true or false");
                }
                (v != 0.0) as u32
            }
            _ => unreachable!(),
        };
        c.data_mut().io[GENO_LUA_IO_STATE as usize + slot as usize] = w;
        Ok(())
    })?;
    // glua_go: "auto", "roll", "helpless" or a Geno state of the define
    l.func_wrap("gd_geno", "go", |mut c: Caller<'_, Ctx>, ptr: i32, len: i32| -> Res<()> {
        let k = name(&mut c, ptr, len)?;
        let t = match k.as_slice() {
            b"auto" => GENO_TGT_AUTO,
            b"roll" => GENO_TGT_ROLL,
            b"helpless" => GENO_TGT_HELPLESS,
            _ => match c.data().targets.iter().position(|t| t.as_bytes() == k.as_slice()) {
                Some(i) => ((GENO_TGT_GENO & 15) << 28) | (i as u32 & 0xFFFF),
                None => return fault(&mut c, GENO_LUA_FAULT_COMMAND, "ctx.go: no such state"),
            },
        };
        cmd(&mut c, GENO_LUA_CMD_GO, t, 0)
    })?;
    l.func_wrap("gd_geno", "velocity", |mut c: Caller<'_, Ctx>, fwd: f64, up: f64| -> Res<()> {
        let f = finite(&mut c, fwd, 1000.0, "ctx.velocity takes finite numbers (forward, up)")?;
        let u = finite(&mut c, up, 1000.0, "ctx.velocity takes finite numbers (forward, up)")?;
        cmd(&mut c, GENO_LUA_CMD_VELOCITY, f.to_bits(), u.to_bits())
    })?;
    l.func_wrap("gd_geno", "hitbox_damage", |mut c: Caller<'_, Ctx>, mask: i32, dmg: f64| -> Res<()> {
        if !(1..=15).contains(&mask) {
            return fault(&mut c, GENO_LUA_FAULT_COMMAND, "ctx.hitbox_damage: the mask is 1..15 (bit n = hitbox slot n)");
        }
        let d = finite(&mut c, dmg, 1000.0, "ctx.hitbox_damage takes a finite damage")?;
        if d < 0.0 {
            return fault(&mut c, GENO_LUA_FAULT_COMMAND, "ctx.hitbox_damage: damage is not negative");
        }
        cmd(&mut c, GENO_LUA_CMD_HITBOX_DAMAGE, mask as u32, d.to_bits())
    })?;
    l.func_wrap("gd_geno", "loop", |mut c: Caller<'_, Ctx>| -> Res<()> { cmd(&mut c, GENO_LUA_CMD_LOOP, 0, 0) })?;
    l.func_wrap("gd_geno", "turn", |mut c: Caller<'_, Ctx>| -> Res<()> { cmd(&mut c, GENO_LUA_CMD_TURN, 0, 0) })?;
    // glua_ki: meter units, two decimals, rounded half away from zero
    l.func_wrap("gd_geno", "ki", |mut c: Caller<'_, Ctx>, delta: f64| -> Res<()> {
        let d = finite(&mut c, delta, 1000.0, "ctx.ki takes a finite number (meter units)")?;
        let h = (d * 100.0 + if d < 0.0 { -0.5 } else { 0.5 }) as i32;
        cmd(&mut c, GENO_LUA_CMD_KI, h as u32, 0)
    })?;
    l.func_wrap("gd_geno", "ledge", |mut c: Caller<'_, Ctx>, mode: i32| -> Res<()> {
        if !(0..=3).contains(&mode) {
            return fault(
                &mut c,
                GENO_LUA_FAULT_COMMAND,
                "ctx.ledge takes 0 (none), 1 (front), 2 (front and back) or 3 (2, and the special fall after the move)",
            );
        }
        cmd(&mut c, GENO_LUA_CMD_LEDGE, mode as u32, 0)
    })?;
    Ok(l)
}

/// A loaded module: compiled once, instantiated per call.
pub struct GenoWasm {
    engine: Engine,
    module: Module,
    fns: Vec<String>,
    slot_name: Vec<String>,
    slot_type: Vec<i32>,
    targets: Vec<String>,
    err: String,
    pub calls: u32,
    faults: u32,
}

fn has_start_section(b: &[u8]) -> bool {
    let mut p = 8;
    while p < b.len() {
        let id = b[p];
        p += 1;
        let (mut n, mut shift) = (0usize, 0);
        while p < b.len() {
            let x = b[p];
            p += 1;
            n |= ((x & 0x7f) as usize) << shift;
            shift += 7;
            if x & 0x80 == 0 {
                break;
            }
        }
        if id == 8 {
            return true;
        }
        p = p.saturating_add(n);
    }
    false
}

/// Validates and compiles a module (`Env::fighter_wasm`). Err names the rule it broke.
pub fn load(bytes: &[u8], slot_name: &[String], slot_type: &[i32], targets: &[String]) -> Result<GenoWasm, String> {
    if bytes.len() < 8 || has_start_section(bytes) {
        return Err("not a wasm module, or it has a start function".into());
    }
    let mut cfg = Config::default();
    cfg.consume_fuel(true);
    let engine = Engine::new(&cfg);
    let module = Module::new(&engine, bytes).map_err(|e| format!("invalid wasm: {e}"))?;
    for imp in module.imports() {
        if imp.module() != "gd_geno" || imp.ty().func().is_none() {
            return Err(format!("import {}::{}: only gd_geno functions may be imported", imp.module(), imp.name()));
        }
    }
    let mut fns = Vec::new();
    let mut mem = false;
    for ex in module.exports() {
        match ex.ty() {
            wasmi::ExternType::Func(f) if f.params().is_empty() && f.results().is_empty() => fns.push(ex.name().to_string()),
            wasmi::ExternType::Memory(m) if ex.name() == "memory" => {
                if m.maximum().is_none_or(|x| x > MAX_PAGES as u64) {
                    return Err(format!("memory: needs a maximum of at most {MAX_PAGES} pages"));
                }
                mem = true;
            }
            _ => {}
        }
    }
    if !mem {
        return Err("no exported memory".into());
    }
    let g = GenoWasm {
        engine,
        module,
        fns,
        slot_name: slot_name.to_vec(),
        slot_type: slot_type.to_vec(),
        targets: targets.to_vec(),
        err: String::new(),
        calls: 0,
        faults: 0,
    };
    // every import must be one this host provides: a trial instantiation
    let mut store = g.store([0; GENO_LUA_IO_WORDS as usize]);
    linker(&g.engine).and_then(|l| l.instantiate_and_start(&mut store, &g.module).map(|_| ())).map_err(|e| format!("imports: {e}"))?;
    Ok(g)
}

impl GenoWasm {
    fn store(&self, io: [u32; GENO_LUA_IO_WORDS as usize]) -> Store<Ctx> {
        let ctx = Ctx {
            io,
            ncmd: 0,
            fault: 0,
            msg: "",
            slot_name: self.slot_name.clone(),
            slot_type: self.slot_type.clone(),
            targets: self.targets.clone(),
        };
        let mut s = Store::new(&self.engine, ctx);
        let _ = s.set_fuel(FUEL);
        s
    }
}

impl FighterCode for GenoWasm {
    fn nslot(&self) -> usize {
        self.slot_name.len()
    }
    fn slot_name(&self, i: usize) -> &str {
        &self.slot_name[i]
    }
    fn slot_type(&self, i: usize) -> i32 {
        self.slot_type[i]
    }
    fn find(&self, name: &str) -> i32 {
        self.fns.iter().position(|n| n == name).map_or(-1, |i| i as i32)
    }
    /// The geno_lua_core.h call contract: the commands and fault word reset, the callback run, on a fault the command count
    /// cleared (the state words it already wrote are the caller's to discard) and the fault word set.
    unsafe fn call(&mut self, f: i32, io: *mut u32) -> i32 {
        self.calls += 1;
        self.err.clear();
        let mut words = [0u32; GENO_LUA_IO_WORDS as usize];
        core::ptr::copy_nonoverlapping(io, words.as_mut_ptr(), words.len());
        words[GENO_LUA_IO_NCMDS as usize] = 0;
        words[GENO_LUA_IO_FAULT as usize] = 0;
        let mut store = self.store(words);
        let run = linker(&self.engine)
            .and_then(|l| l.instantiate_and_start(&mut store, &self.module))
            .and_then(|inst| inst.get_typed_func::<(), ()>(&store, &self.fns[f as usize]))
            .and_then(|func| func.call(&mut store, ()));
        let fuel_left = store.get_fuel().unwrap_or(1);
        let d = store.data_mut();
        let mut st = 0u32;
        if let Err(e) = run {
            st = if d.fault != 0 {
                d.fault
            } else if fuel_left == 0 {
                GENO_LUA_FAULT_INSNS
            } else {
                GENO_LUA_FAULT_ERROR
            };
            self.err = if d.msg.is_empty() { e.to_string() } else { d.msg.to_string() };
            self.faults += 1;
            d.io[GENO_LUA_IO_NCMDS as usize] = 0;
        }
        d.io[GENO_LUA_IO_FAULT as usize] = st;
        core::ptr::copy_nonoverlapping(d.io.as_ptr(), io, d.io.len());
        st as i32
    }
    fn err(&self) -> &str {
        &self.err
    }
    fn faults(&self) -> u32 {
        self.faults
    }
}
