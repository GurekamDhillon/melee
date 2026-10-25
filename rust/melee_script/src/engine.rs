//! The script engine core: `gs_init`, the sandbox, script slots, hooks, tasks, console
//! (`pc/platform/gw_script.c`, "Lua plumbing" through "init" and the console commands).
//!
//! One Lua state is shared by every script. A script is a slot with a private environment table (copies
//! of the library tables and `gd`), a manifest, up to 16 coroutine tasks, and an error count that switches
//! it off at 20. Every call into Lua goes through [`Engine::pcall`], which arms the per-call instruction
//! budget and wall-clock backstop, names the running script for `gd.*` (the C `gs.cur`), and reports
//! errors to the console and the log.
use crate::args::fmt_g;
use crate::game::ScriptGame;
use crate::host::Host;
use crate::manifest;
use mlua::{
    Function, HookTriggers, Lua, LuaOptions, MultiValue, RegistryKey, Result as LuaResult, StdLib, Table, Thread,
    Value, VmState,
};
use mlua::thread::ThreadStatus;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::rc::Rc;

pub const GW_SCRIPT_API_VERSION: i32 = 2;
pub const GS_MAX_SCRIPTS: usize = 128;
pub const GS_MAX_TASKS: usize = 16;
pub const GS_CLIENTS: usize = 4;
pub const GS_MAX_COMMANDS: usize = 64;
pub const GS_MEM_CAP: usize = 64 << 20;
pub const GS_MAX_ERRORS: i32 = 20;
pub const GS_LINES: usize = 400;
pub const GS_MAX_DRAW: usize = 2048;
/// `gw_script_budget.h`
pub const BUDGET_INSTR: i64 = 2_000_000;
pub const BUDGET_MS: f64 = 500.0;
pub const HOOK_STEP: i64 = 1000;

pub const GS_WHITE: u32 = 0xE8E8E8FF;
pub const GS_GREY: u32 = 0x9A9AA2FF;
pub const GS_RED: u32 = 0xFF6B6BFF;
pub const GS_YELLOW: u32 = 0xFFD166FF;
pub const GS_GREEN: u32 = 0x7BE495FF;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DrawKind {
    Text = 0,
    Box = 1,
    Fill = 2,
    Line = 3,
    Kit = 4,
}

/// `GwScriptDraw`
#[derive(Clone, Debug)]
pub struct Draw {
    pub kind: DrawKind,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub rgba: u32,
    pub size: f32,
    pub text: String,
}

pub struct Task {
    pub thread: Thread,
    pub wait: i32,
    pub pad_owner: i32,
    pub client_owner: i32,
}

#[derive(Default)]
pub struct Script {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub entry: String,
    pub ui_dir: String,
    pub origin: String,
    pub api_version: i32,
    pub gameplay: bool,
    pub rollback_safe: bool,
    pub env: Option<Table>,
    pub tasks: Vec<Option<Task>>,
    pub errors: i32,
    pub disabled: bool,
    pub used: bool,
    pub stage_owner: i32,
    pub src_hash: u64,
}

pub struct Command {
    pub name: String,
    pub help: String,
    pub func: Function,
    pub script: usize,
}

pub struct State {
    pub host: Rc<RefCell<dyn Host>>,
    pub game: Box<dyn ScriptGame>,
    pub scripts: Vec<Script>,
    pub cur: Option<usize>,
    pub console: usize,
    pub commands: Vec<Command>,
    pub lines: VecDeque<(u32, String)>,
    pub capture: Option<String>,
    pub base: Option<Table>,
    pub budget: i64,
    pub budget_per_call: i64,
    pub deadline: f64,
    pub ms_per_call: f64,
    pub callback_refused: bool,
    pub stage_owner_serial: i32,
    pub input_owner: i32,
    pub client_owner: i32,
    // frame / scene
    pub frame: i32,
    pub scene_kind: i32,
    pub scene_epoch: i32,
    pub match_active: bool,
    pub match_frame: i32,
    pub ev: crate::events::EventState,
    pub zone: crate::gd::zones::ZoneAdapter,
    pub deadlines: crate::gd::deadline::Deadlines,
    pub contacts: crate::gd::contacts::Contacts,
    pub fly_cursor: crate::gd::flycam::FlyCursor,
    pub last_action: [i32; 6],
    pub state_frame: [i32; 6],
    pub paused: bool,
    pub step: i32,
    /// `gs.stage_zones`: LAB_STAGE_ZONES has no getter, so the last write is remembered
    pub stage_zones: bool,
    pub pads: crate::pad::ScriptPads,
    pub pending_save: i32,
    pub pending_load: i32,
    pub snap_ready: bool,
    pub rw: crate::rewind::Rewind,
    pub slot: [crate::gd::savestate::SaveSlot; crate::gd::savestate::GS_SAVE_SLOTS],
    /// `gd.set_motion` / `gd.anim_preview` requests: motion + 1 to enter at the next frame boundary, per slot
    pub set_motion: [i32; 6],
    pub set_rate: [f32; 6],
    pub set_lift: [f32; 6],
    /// `gs.lab_request`: the frontend's LAB entry or MELEE_LAB=1 asked for the Lab
    pub lab_request: bool,
    /// `gs.camera_owner` (script slot + 1), the task that took it, and the completion counter last seen
    pub camera_owner: i32,
    pub camera_task_owner: i32,
    pub camera_completion: i32,
    /// error sentinel + wrapper factory (see Engine::native)
    pub err_sentinel: Option<Table>,
    pub wrapper: Option<Function>,
    // drawing
    pub draw: [Vec<Draw>; 2],
    pub build: usize,
    // keys
    pub key_now: [bool; 256],
    pub key_prev: [bool; 256],
    pub console_open: bool,
    // directories
    pub exe_dir: PathBuf,
    pub scripts_dir: PathBuf,
    pub data_dir: PathBuf,
    pub mods_dir: PathBuf,
    pub active_mods: Vec<String>,
}

pub struct Config {
    pub exe_dir: PathBuf,
    pub scripts_dir: PathBuf,
    pub data_dir: PathBuf,
    pub mods_dir: PathBuf,
    pub active_mods: Vec<String>,
    pub budget_instr: i64,
    pub budget_ms: f64,
}

impl Config {
    /// Defaults as `gs_init` computes them, with the same environment overrides
    /// (`MELEE_SCRIPTS_DIR`, `MELEE_SCRIPT_DATA_DIR`, `MELEE_SCRIPT_BUDGET`, `MELEE_SCRIPT_MS`).
    pub fn from_env(exe_dir: PathBuf) -> Config {
        let get = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
        let scripts_dir = get("MELEE_SCRIPTS_DIR").map(PathBuf::from).unwrap_or_else(|| exe_dir.join("scripts"));
        let data_dir = get("MELEE_SCRIPT_DATA_DIR").map(PathBuf::from).unwrap_or_else(|| exe_dir.join("scripts-data"));
        let budget = get("MELEE_SCRIPT_BUDGET").map(|v| manifest::atoi(&v)).filter(|&v| v > 0);
        let ms = get("MELEE_SCRIPT_MS").map(|v| manifest::atoi(&v)).filter(|&v| v > 0);
        Config {
            mods_dir: exe_dir.join("mods"),
            exe_dir,
            scripts_dir,
            data_dir,
            active_mods: Vec::new(),
            budget_instr: budget.map(|v| v as i64).unwrap_or(BUDGET_INSTR),
            budget_ms: ms.map(|v| v as f64).unwrap_or(BUDGET_MS),
        }
    }
}

pub type Shared = Rc<RefCell<State>>;

pub struct Engine {
    pub lua: Lua,
    pub st: Shared,
}

/// Lua-side helpers compiled into the shared base (`gs_prelude`): they only use the public API.
const PRELUDE: &str = "local gd, coroutine = ...\n\
function gd.wait(n) return coroutine.yield(math.max(1, math.floor(n or 1))) end\n\
local native_wait_until = gd.wait_until\n\
function gd.wait_until(fn, timeout)\n\
  if type(fn) == 'table' then return native_wait_until(fn) end\n\
  local t = 0\n\
  while not fn() do\n\
    if timeout and t >= timeout then return false end\n\
    coroutine.yield(1); t = t + 1\n\
  end\n\
  return true\n\
end\n\
function gd.press(port, buttons, frames, extra)\n\
  frames = frames or 1\n\
  local spec = extra or {}\n\
  spec.buttons = buttons\n\
  gd.input(port, spec, frames)\n\
  gd.wait(frames)\n\
end\n\
function gd.tilt(port, x, y, frames) gd.input(port, {x = x, y = y}, frames or 1) gd.wait(frames or 1) end\n";

const COMM_LUA: &str = include_str!("../../../pc/platform/gw_comm.lua");

fn lua_err(e: impl std::fmt::Display) -> mlua::Error {
    mlua::Error::runtime(e.to_string())
}

/// `gs_fnv`
pub fn fnv(mut h: u64, bytes: &[u8]) -> u64 {
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    h
}

impl State {
    pub fn arm_budget(&mut self) {
        self.budget = self.budget_per_call;
        let now = self.host.borrow().now_ms();
        self.deadline = now + self.ms_per_call;
    }

    pub fn log(&self, msg: &str) {
        self.host.borrow_mut().log(msg);
    }

    pub fn script_id(&self, i: Option<usize>) -> String {
        match i {
            Some(i) if i < self.scripts.len() && self.scripts[i].used => self.scripts[i].id.clone(),
            _ => "?".to_string(),
        }
    }

    /// `gw_Console_Print`: text is split on newlines; lines are cut to 199 bytes like `gs_line[..][200]`.
    pub fn print(&mut self, rgba: u32, text: &str) {
        for part in text.split('\n') {
            let mut end = part.len().min(199);
            while !part.is_char_boundary(end) {
                end -= 1;
            }
            let line = part[..end].to_string();
            if let Some(c) = self.capture.as_mut() {
                c.push_str(&line);
                c.push('\n');
            }
            if self.lines.len() == GS_LINES {
                self.lines.pop_front();
            }
            self.lines.push_back((rgba, line));
        }
    }

    pub fn cur_script(&self) -> Option<&Script> {
        self.cur.and_then(|i| self.scripts.get(i))
    }

    pub fn find(&self, id: &str) -> Option<usize> {
        self.scripts.iter().position(|s| s.used && s.id.eq_ignore_ascii_case(id))
    }

    /// `gs_pad_owner`: the identity a pad claim is made under.
    pub fn pad_owner(&self) -> i32 {
        let cur = self.cur.map_or(0, |c| c as i32 + 1);
        if self.input_owner > (GS_MAX_SCRIPTS + GS_CLIENTS) as i32 || (self.cur == Some(self.console) && self.input_owner > 0) {
            return self.input_owner;
        }
        cur
    }

    /// `gw_script_pad_release` for any of the six slots (5-6 are the game's virtual pads), then the pad log.
    pub fn pad_release(&mut self, ch: i32, owner: i32) {
        if (4..6).contains(&ch) && owner > 0 {
            self.game.virtual_pad_release(ch, owner);
            return;
        }
        self.pads.release(ch, owner);
        let log: Vec<String> = self.pads.log.drain(..).collect();
        for l in log {
            self.log(&l);
        }
    }

    /// `gs_cpu_log_slot`: "cpu: P2 idle (global)" - which CPUs could interfere is visible from the log alone.
    pub fn cpu_log_slot(&mut self, slot: i32) {
        if let Some((w, src)) = crate::gd::fighter::cpu_mode_words(self.game.cpu_mode_info(slot)) {
            self.log(&format!("cpu: P{} {} ({})", slot + 1, w, src));
        }
    }

    /// `gs_fly_set`: mode 0 off, 1 on, 2 place, -1 toggle. Returns the game's code.
    pub fn fly_set(&mut self, slot: i32, mode: i32) -> i32 {
        let mode = if mode < 0 { if self.game.geno_fly_get(slot) > 0 { 0 } else { 1 } } else { mode };
        self.rw_branch();
        let rc = self.game.geno_fly_set(slot, mode);
        if rc >= 0 {
            self.log(&format!(
                "fly: P{} {}{}",
                slot + 1,
                match mode { 1 => "on", 2 => "off (placed)", _ => "off" },
                if rc == 1 { " - no floor below, dropped where it was" } else { "" }
            ));
        }
        rc
    }

    /// `gs_set_motion_core`: shared scheduling; callers validate the fighter and motion first.
    pub fn set_motion_core(&mut self, slot: usize, motion: i32, frame: i32, rate: f32, lift: f32) {
        let frame = frame.clamp(1, 600);
        self.step = 0;
        self.set_motion[slot] = motion + 1;
        self.set_rate[slot] = rate;
        self.set_lift[slot] = lift;
        self.paused = true;
        self.step = frame - 1;
    }

    /// `gs_apply_set_motion`: at a frame boundary the requested fighters enter their motion.
    pub fn apply_set_motion(&mut self) {
        let session = self.host.borrow().session();
        let mut any = false;
        for slot in 0..6 {
            if self.set_motion[slot] > 0 && !session {
                let rate = if self.set_rate[slot] > 0.0 { self.set_rate[slot] } else { 1.0 };
                if !any {
                    self.rw_branch(); // a write: the timeline forks here
                    any = true;
                }
                let m = self.set_motion[slot] - 1;
                self.game.lab_set_motion(slot as i32, m, rate.to_bits() as i32, self.set_lift[slot].to_bits() as i32);
            }
            self.set_motion[slot] = 0;
        }
    }

    /// `gs.nev = 0`: queued engine events are not ported yet
    pub fn nev_reset(&mut self) {}

    /// `gs_require_gameplay`
    pub fn require_gameplay(&mut self, fname: &str) -> LuaResult<()> {
        let session = self.host.borrow().session();
        let Some(i) = self.cur.filter(|&i| i < self.scripts.len()) else {
            self.callback_refused = true;
            return Err(lua_err(format!("gd.{fname}: no current script")));
        };
        if i != self.console && !self.scripts[i].gameplay {
            self.callback_refused = true;
            return Err(lua_err(format!(
                "gd.{fname} changes gameplay: the script's mod.json must say \"gameplay\": true"
            )));
        }
        if session {
            self.callback_refused = true;
            return Err(lua_err(format!(
                "gd.{fname} is offline-only: Lua gameplay writes cannot be replayed during rollback"
            )));
        }
        Ok(())
    }

    /// `gs_require_offline`
    pub fn require_offline(&mut self, fname: &str) -> LuaResult<()> {
        self.require_gameplay(fname)?;
        if self.host.borrow().session() {
            self.callback_refused = true;
            return Err(lua_err(format!("gd.{fname} is offline-only (refused during a netplay/rollback session)")));
        }
        Ok(())
    }
}

impl Engine {
    pub fn new(host: Rc<RefCell<dyn Host>>, game: Box<dyn ScriptGame>, cfg: Config) -> LuaResult<Engine> {
        let libs = StdLib::COROUTINE | StdLib::TABLE | StdLib::STRING | StdLib::UTF8 | StdLib::MATH;
        let lua = Lua::new_with(libs, LuaOptions::new())?;
        lua.set_memory_limit(GS_MEM_CAP)?;
        let st = Rc::new(RefCell::new(State {
            host,
            game,
            scripts: Vec::new(),
            cur: None,
            console: 0,
            commands: Vec::new(),
            lines: VecDeque::new(),
            capture: None,
            base: None,
            budget: cfg.budget_instr,
            budget_per_call: cfg.budget_instr,
            deadline: 0.0,
            ms_per_call: cfg.budget_ms,
            callback_refused: false,
            stage_owner_serial: 0,
            input_owner: 0,
            client_owner: 0,
            frame: 0,
            scene_kind: -1,
            scene_epoch: 0,
            match_active: false,
            match_frame: 0,
            ev: Default::default(),
            zone: Default::default(),
            deadlines: Default::default(),
            contacts: Default::default(),
            fly_cursor: Default::default(),
            last_action: [0; 6],
            state_frame: [0; 6],
            paused: false,
            step: 0,
            stage_zones: false,
            pads: Default::default(),
            pending_save: 0,
            pending_load: 0,
            snap_ready: false,
            rw: Default::default(),
            slot: Default::default(),
            set_motion: [0; 6],
            set_rate: [0.0; 6],
            set_lift: [0.0; 6],
            lab_request: false,
            camera_owner: 0,
            camera_task_owner: 0,
            camera_completion: 0,
            err_sentinel: None,
            wrapper: None,
            draw: [Vec::new(), Vec::new()],
            build: 0,
            key_now: [false; 256],
            key_prev: [false; 256],
            console_open: false,
            exe_dir: cfg.exe_dir,
            scripts_dir: cfg.scripts_dir,
            data_dir: cfg.data_dir,
            mods_dir: cfg.mods_dir,
            active_mods: cfg.active_mods,
        }));
        let eng = Engine { lua, st };
        eng.install_budget_hook()?;
        eng.build_base()?;
        // the console: a pseudo-script with its own persistent environment
        {
            let console = eng.alloc_script().expect("script table");
            let env = eng.new_env()?;
            let mut s = eng.st.borrow_mut();
            let sc = &mut s.scripts[console];
            sc.id = "console".into();
            sc.name = "console".into();
            sc.origin = "console".into();
            sc.used = true;
            sc.gameplay = true;
            sc.env = Some(env);
            s.console = console;
            s.print(GS_GREY, &format!("GD's Melee console - scripting API {GW_SCRIPT_API_VERSION}. \"help\" lists the commands."));
            let msg = format!(
                "script: Lua 5.4 engine up (API {GW_SCRIPT_API_VERSION}), scripts from {}",
                s.scripts_dir.display()
            );
            s.log(&msg);
        }
        {
            let mut s = eng.st.borrow_mut();
            let v = s.host.borrow().getenv("MELEE_LAB");
            if v.is_some_and(|v| !v.is_empty() && !v.starts_with('0')) {
                s.lab_request = true;
                s.log("script: MELEE_LAB: the Geno Lab is requested for this session");
            }
        }
        Ok(eng)
    }

    /// The load half of `gs_init`: scripts from the scripts folder, then from active mods, then the `MELEE_SCRIPT`
    /// list (`;`-separated names or paths, origin "env"). `MELEE_SCRIPTS=0` skips the first two.
    pub fn boot(&self) {
        let getenv = |k: &str| self.st.borrow().host.borrow().getenv(k);
        if !getenv("MELEE_SCRIPTS").is_some_and(|v| v.starts_with('0')) {
            self.scan_scripts_dir();
            self.scan_mods();
        }
        if let Some(v) = getenv("MELEE_SCRIPT").filter(|v| !v.is_empty()) {
            for name in v.split(';').filter(|n| !n.is_empty()) {
                if let Some(i) = self.load_named(name) {
                    self.st.borrow_mut().scripts[i].origin = "env".into();
                }
            }
        }
    }

    /// `lua_sethook(L, gs_count_hook, LUA_MASKCOUNT, GW_SCRIPT_HOOK_STEP)` as a raw C hook: `luaL_error` raises a plain
    /// string with the caller's position, like the C build (mlua's hook callbacks raise error userdata instead), and new
    /// coroutines inherit it from the main state.
    fn install_budget_hook(&self) -> LuaResult<()> {
        let ptr = Rc::as_ptr(&self.st) as *mut std::ffi::c_void;
        unsafe {
            self.lua.exec_raw::<()>((), |state| {
                mlua::ffi::lua_pushlightuserdata(state, ptr);
                mlua::ffi::lua_rawsetp(state, mlua::ffi::LUA_REGISTRYINDEX, HOOK_KEY.as_ptr() as *const std::ffi::c_void);
                mlua::ffi::lua_sethook(state, Some(count_hook), mlua::ffi::LUA_MASKCOUNT, HOOK_STEP as std::ffi::c_int);
            })
        }
    }

    /// Wrap a native `gd` function. Errors from an mlua callback reach `pcall` as userdata; the C build's `luaL_error`
    /// raises a string positioned at the Lua caller. So the native never raises: it returns `(sentinel, message)`
    /// and a small Lua trampoline turns that into `error(message, 2)`.
    pub fn native<R, F>(&self, name: &'static str, f: F) -> LuaResult<Function>
    where
        R: mlua::IntoLuaMulti,
        F: Fn(&Lua, &Shared, crate::args::Args) -> LuaResult<R> + 'static,
    {
        let st = self.st.clone();
        let sentinel = st.borrow().err_sentinel.clone().expect("sentinel");
        let inner = self.lua.create_function(move |lua, args: MultiValue| match f(lua, &st, crate::args::Args::new(name, args)) {
            Ok(r) => r.into_lua_multi(lua),
            Err(e) => Ok(MultiValue::from_iter([Value::Table(sentinel.clone()), Value::String(lua.create_string(error_text(&e))?)])),
        })?;
        let factory = self.st.borrow().wrapper.clone().expect("wrapper");
        factory.call(inner)
    }

    /// `gs_build_base`: the template environment every script's environment is copied from.
    fn build_base(&self) -> LuaResult<()> {
        let lua = &self.lua;
        let g = lua.globals();
        {
            let sentinel = lua.create_table()?;
            let factory: Function = lua
                .load(NATIVE_WRAPPER)
                .set_name("=gd.native")
                .call::<Function>(sentinel.clone())?;
            let mut s = self.st.borrow_mut();
            s.err_sentinel = Some(sentinel);
            s.wrapper = Some(factory);
        }
        // string.dump would hand out bytecode; lock the string metatable; a fixed seed keeps math.random reproducible
        lua.load(
            "string.dump = nil\n\
             getmetatable('').__metatable = 'locked'\n\
             math.randomseed(0)",
        )
        .set_name("=sandbox")
        .exec()?;
        let base = lua.create_table()?;
        for k in [
            "assert", "error", "ipairs", "next", "pairs", "pcall", "select", "tonumber", "tostring", "type",
            "xpcall", "rawequal", "rawget", "rawset", "rawlen", "setmetatable", "getmetatable", "_VERSION",
        ] {
            base.set(k, g.get::<Value>(k)?)?;
        }
        for k in ["string", "table", "math", "utf8", "coroutine"] {
            base.set(k, g.get::<Value>(k)?)?;
        }
        base.set("print", self.make_log("print")?)?;
        base.set("load", self.make_load()?)?;
        base.set("collectgarbage", self.make_collectgarbage()?)?;
        let table: Table = g.get("table")?;
        base.set("unpack", table.get::<Value>("unpack")?)?;

        let gd = lua.create_table()?;
        crate::gd::register_all(self, &gd)?;
        gd.set("api_version", GW_SCRIPT_API_VERSION)?;
        gd.set("clank_event", true)?;
        gd.set("sim_supported", true)?;
        gd.set("echo_supported", true)?;
        gd.set("api_name", "GD's Melee scripting API 2")?;
        gd.set("lab_api", 1)?; // the Geno Lab API (private build); nil elsewhere
        let dep = lua.create_table()?;
        dep.set("tbd_request", "use mod.json menus and on_entry (docs/scripting.md, Atlas entries)")?;
        gd.set("deprecated", dep)?;
        let buttons = lua.create_table()?;
        for (name, bit) in crate::gd::BUTTONS {
            buttons.set(*name, *bit)?;
        }
        gd.set("buttons", buttons)?;
        let draw = lua.create_table()?;
        for (n, b) in [
            ("MODEL", 0x01), ("HIT", 0x02), ("HURT", 0x02), ("COLL", 0x02), ("DYNAMICS", 0x04), ("STOMP", 0x08),
            ("CPU", 0x10), ("ITEM_PICKUP", 0x20), ("THROWN", 0x40), ("COIN", 0x80), ("DEFAULT", 0x01),
        ] {
            draw.set(n, b)?;
        }
        gd.set("draw", draw)?;
        let stage = lua.create_table()?;
        for (n, b) in [("COLL", 1), ("ECB", 1), ("TERRAIN", 2), ("LEDGES", 4), ("POINTS", 8), ("ZONES", 16)] {
            stage.set(n, b)?;
        }
        gd.set("stage_draw", stage)?;
        gd.set("kit", crate::gd::push_kit(self)?)?;
        gd.set("ui", crate::gd::push_ui(lua)?)?;
        // the prelude adds wait / wait_until / press / tilt; gd.comm is the comm callout window
        let co: Value = g.get("coroutine")?;
        for (src, name) in [(PRELUDE, "=gd.prelude"), (COMM_LUA, "=gd.comm")] {
            let r = lua.load(src).set_name(name).set_mode(mlua::chunk::ChunkMode::Text).into_function();
            match r {
                Ok(f) => {
                    if let Err(e) = f.call::<()>((gd.clone(), co.clone())) {
                        self.st.borrow().log(&format!("script: {name} failed: {e}"));
                    }
                }
                Err(e) => self.st.borrow().log(&format!("script: {name} did not compile: {e}")),
            }
        }
        base.set("gd", gd)?;
        self.st.borrow_mut().base = Some(base);
        Ok(())
    }

    /// `l_log` (also the sandbox's `print`)
    pub(crate) fn make_log(&self, _what: &'static str) -> LuaResult<Function> {
        let tostring: Function = self.lua.globals().get("tostring")?;
        self.native(_what, move |_, st, args| {
            let mut out = String::new();
            for (i, v) in args.get_all().into_iter().enumerate() {
                if i > 0 {
                    out.push(' ');
                }
                let s: mlua::LuaString = tostring.call(v)?;
                out.push_str(&s.to_string_lossy());
            }
            let mut s = st.borrow_mut();
            let id = s.script_id(s.cur);
            s.print(GS_WHITE, &format!("[{id}] {out}"));
            s.log(&format!("script [{id}] {out}"));
            Ok(())
        })
    }

    /// `l_load`: text chunks only, run in the calling script's environment unless one is given.
    fn make_load(&self) -> LuaResult<Function> {
        self.native("load", move |lua, st, a| {
            let chunk = a.check_bytes(1)?;
            let name = a.opt_string(2, "=(load)")?;
            let has_env = !a.none_or_nil(4);
            let f = match lua.load(&chunk[..]).set_name(name).set_mode(mlua::chunk::ChunkMode::Text).into_function() {
                Ok(f) => f,
                Err(e) => {
                    let msg = match &e {
                        mlua::Error::SyntaxError { message, .. } => message.clone(),
                        other => other.to_string(),
                    };
                    return Ok(MultiValue::from_iter([Value::Nil, Value::String(lua.create_string(&msg)?)]));
                }
            };
            let env: Value = if has_env {
                a.get(4).clone()
            } else {
                let s = st.borrow();
                match s.cur_script().and_then(|c| c.env.clone()) {
                    Some(t) => Value::Table(t),
                    None => Value::Nil,
                }
            };
            match env {
                Value::Table(t) => {
                    f.set_environment(t)?;
                }
                _ => {
                    // lua_setupvalue(_ENV = non-table): the chunk runs with no globals; mirror with an empty table
                    f.set_environment(lua.create_table()?)?;
                }
            }
            Ok(MultiValue::from_iter([Value::Function(f)]))
        })
    }

    fn make_collectgarbage(&self) -> LuaResult<Function> {
        self.native("collectgarbage", |lua, _, a| {
            let opt = a.opt_string(1, "count")?;
            if opt == "count" {
                return Ok(lua.used_memory() as f64 / 1024.0);
            }
            Err(lua_err("collectgarbage: only \"count\" is available to scripts"))
        })
    }

    /// `gs_copy_table` / `gs_copy_deep`
    fn copy_table(&self, src: &Table, depth: u32) -> LuaResult<Table> {
        let t = self.lua.create_table()?;
        for pair in src.pairs::<Value, Value>() {
            let (k, v) = pair?;
            let v = match v {
                Value::Table(inner) if depth > 1 => Value::Table(self.copy_table(&inner, depth - 1)?),
                other => other,
            };
            t.raw_set(k, v)?;
        }
        Ok(t)
    }

    /// `gs_new_env`: the base's entries with private copies of every library table and gd.
    pub fn new_env(&self) -> LuaResult<Table> {
        let base = self.st.borrow().base.clone().expect("base");
        let env = self.copy_table(&base, 1)?;
        for k in ["string", "table", "math", "utf8", "coroutine", "gd", "wb"] {
            let Value::Table(t) = base.get::<Value>(k)? else { continue };
            let c = self.copy_table(&t, 1)?;
            if k == "gd" {
                // gd.kit and its tables (colors and its sections, roles, row): private copies too
                if let Value::Table(kit) = c.get::<Value>("kit")? {
                    c.set("kit", self.copy_table(&kit, 3)?)?;
                }
                if let Value::Table(ui) = c.get::<Value>("ui")? {
                    c.set("ui", self.copy_table(&ui, 2)?)?;
                }
            }
            env.set(k, c)?;
        }
        env.set("_G", env.clone())?;
        Ok(env)
    }

    pub fn alloc_script(&self) -> Option<usize> {
        let mut s = self.st.borrow_mut();
        if s.stage_owner_serial == i32::MAX {
            return None;
        }
        let i = match s.scripts.iter().position(|x| !x.used) {
            Some(i) => i,
            None => {
                if s.scripts.len() >= GS_MAX_SCRIPTS {
                    return None;
                }
                s.scripts.push(Script::default());
                s.scripts.len() - 1
            }
        };
        s.stage_owner_serial += 1;
        let owner = s.stage_owner_serial;
        let mut sc = Script::default();
        sc.stage_owner = owner;
        sc.tasks = (0..GS_MAX_TASKS).map(|_| None).collect();
        s.scripts[i] = sc;
        Some(i)
    }

    // ---------------------------------------------------------------- calls into Lua

    /// `gs_report`
    pub fn report(&self, script: usize, what: &str, err: &str) {
        let mut s = self.st.borrow_mut();
        let id = s.script_id(Some(script));
        s.print(GS_RED, &format!("[{id}] {what}: {err}"));
        s.log(&format!("script [{id}] {what}: {err}"));
        let console = s.console;
        if script != console && script < s.scripts.len() {
            s.scripts[script].errors += 1;
            if s.scripts[script].errors >= GS_MAX_ERRORS && !s.scripts[script].disabled {
                s.scripts[script].disabled = true;
                let (id, n) = (s.scripts[script].id.clone(), s.scripts[script].errors);
                s.print(GS_RED, &format!("[{id}] switched off after {n} errors (\"reload\" to try again)"));
            }
        }
    }

    /// `gs_pcall`: run `f` as script `script`. `None` when it raised (already reported).
    pub fn pcall(&self, script: usize, f: &Function, args: MultiValue, what: &str) -> Option<MultiValue> {
        let prev = {
            let mut s = self.st.borrow_mut();
            let p = s.cur;
            s.cur = Some(script);
            s.arm_budget();
            s.callback_refused = false;
            p
        };
        let r = f.call::<MultiValue>(args);
        self.st.borrow_mut().cur = prev;
        match r {
            Ok(v) => Some(v),
            Err(e) => {
                self.report(script, what, &clean_error(&e));
                None
            }
        }
    }

    /// `gs_get_hook`: script i's global function `name`.
    pub fn get_hook(&self, i: usize, name: &str) -> Option<Function> {
        let env = {
            let s = self.st.borrow();
            let sc = s.scripts.get(i)?;
            if !sc.used || sc.disabled {
                return None;
            }
            sc.env.clone()?
        };
        match env.get::<Value>(name) {
            Ok(Value::Function(f)) => Some(f),
            _ => None,
        }
    }

    /// `gs_hook_all`
    pub fn hook_all(&self, name: &str, args: &[Value]) {
        let n = self.st.borrow().scripts.len();
        for i in 0..n {
            if let Some(f) = self.get_hook(i, name) {
                self.pcall(i, &f, args.iter().cloned().collect(), name);
            }
        }
    }

    /// `gs_run_tasks`: resume every script's coroutine tasks that are not waiting.
    pub fn run_tasks(&self) {
        let n = self.st.borrow().scripts.len();
        for i in 0..n {
            let (usable, ntasks) = {
                let s = self.st.borrow();
                (s.scripts[i].used && !s.scripts[i].disabled, s.scripts[i].tasks.len())
            };
            if !usable {
                continue;
            }
            for k in 0..ntasks {
                let thread = {
                    let mut s = self.st.borrow_mut();
                    let Some(t) = s.scripts[i].tasks[k].as_mut() else { continue };
                    if t.wait > 0 {
                        t.wait -= 1;
                        continue;
                    }
                    let (po, co) = (t.pad_owner, t.client_owner);
                    let th = t.thread.clone();
                    s.cur = Some(i);
                    s.input_owner = po;
                    s.client_owner = co;
                    s.arm_budget();
                    s.callback_refused = false;
                    th
                };
                let r = thread.resume::<MultiValue>(());
                {
                    let mut s = self.st.borrow_mut();
                    s.cur = None;
                    s.input_owner = 0;
                    s.client_owner = 0;
                }
                match r {
                    Ok(vals) if thread.status() == ThreadStatus::Resumable => {
                        // gd.wait(n) yields n: skip n-1 more frames
                        let n = match vals.front() {
                            Some(Value::Integer(n)) => *n as i32,
                            _ => 1,
                        };
                        let mut s = self.st.borrow_mut();
                        if let Some(t) = s.scripts[i].tasks[k].as_mut() {
                            t.wait = if n > 1 { n - 1 } else { 0 };
                        }
                        continue;
                    }
                    Ok(_) => {}
                    Err(e) => self.report(i, "task", &clean_error(&e)),
                }
                self.st.borrow_mut().scripts[i].tasks[k] = None;
                if self.st.borrow().scripts[i].disabled {
                    break;
                }
            }
        }
    }

    // ---------------------------------------------------------------- loading

    /// `gs_unload`
    pub fn unload(&self, i: usize) {
        if !self.st.borrow().scripts.get(i).is_some_and(|s| s.used) {
            return;
        }
        if let Some(f) = self.get_hook(i, "on_unload") {
            self.pcall(i, &f, MultiValue::new(), "on_unload");
        }
        let stage_owner = self.st.borrow().scripts[i].stage_owner;
        self.zones_release(stage_owner);
        self.contact_release(i);
        {
            let mut s = self.st.borrow_mut();
            let (p, e, l) = s.deadlines.release(i);
            s.host.borrow_mut().hang_script_state(p, e, &l);
        }
        let mut s = self.st.borrow_mut();
        s.commands.retain(|c| c.script != i);
        if s.match_active {
            // arena-hooks: ownership is read from the restored game snapshot
            s.rw_branch();
            s.game.arena_release(i as i32 + 1);
        }
        s.game.caps_release(i as i32 + 1);
        s.game.owner_release(i as i32 + 1);
        for slot in 0..6 {
            // gs_fly_clear_owner
            if s.fly_cursor_owner(slot) == i as i32 + 1 {
                s.game.geno_fly_clear(slot as i32);
                s.fly_cursor_clear(slot);
            }
        }
        let sc = &mut s.scripts[i];
        for t in sc.tasks.iter_mut() {
            *t = None;
        }
        sc.env = None;
        sc.used = false;
    }

    /// `gs_load_text`: the common path for a file, a built-in or a console load. `manifest` is the mod.json text.
    pub fn load_text(&self, id: &str, entry: &str, src: &[u8], manifest: Option<&str>, origin: &str) -> Option<usize> {
        if let Some(old) = self.st.borrow().find(id) {
            self.unload(old);
        }
        let Some(i) = self.alloc_script() else {
            let mut s = self.st.borrow_mut();
            s.print(GS_RED, &format!("too many scripts (max {GS_MAX_SCRIPTS})"));
            let n = s.scripts.len();
            s.log(&format!(
                "script: REFUSED {id} from {entry}: the script table is full ({n} of {GS_MAX_SCRIPTS} slots in use)"
            ));
            return None;
        };
        let src_text = String::from_utf8_lossy(src).into_owned();
        {
            let mut s = self.st.borrow_mut();
            let ui_dir = find_ui_dir(entry);
            let sc = &mut s.scripts[i];
            sc.used = true;
            sc.id = id.to_string();
            sc.name = id.to_string();
            sc.entry = entry.to_string();
            sc.ui_dir = ui_dir;
            sc.origin = origin.to_string();
            sc.api_version = GW_SCRIPT_API_VERSION;
            match manifest {
                None => {
                    // single file: "-- @key: value" header lines
                    let h = |k: &str, cap: usize| manifest::header_field(&src_text, k, cap);
                    if let Some(v) = h("name", 96) { sc.name = v }
                    if let Some(v) = h("version", 96) { sc.version = v }
                    if let Some(v) = h("author", 96) { sc.author = v }
                    if let Some(v) = h("api_version", 96) { sc.api_version = manifest::atoi(&v) }
                    if let Some(v) = h("gameplay", 96) { sc.gameplay = manifest::truthy(&v) }
                    if let Some(v) = h("rollback_safe", 96) { sc.rollback_safe = manifest::truthy(&v) }
                }
                Some(m) => {
                    let j = |k: &str| manifest::json_field(m, k, 96);
                    if let Some(v) = j("name") { sc.name = v }
                    if let Some(v) = j("version") { sc.version = v }
                    if let Some(v) = j("author") { sc.author = v }
                    if let Some(v) = j("api_version") { sc.api_version = manifest::atoi(&v) }
                    if let Some(v) = j("gameplay") { sc.gameplay = manifest::truthy(&v) }
                    if let Some(v) = j("rollback_safe") { sc.rollback_safe = manifest::truthy(&v) }
                }
            }
            if sc.api_version > GW_SCRIPT_API_VERSION {
                let want = sc.api_version;
                sc.used = false;
                s.print(
                    GS_RED,
                    &format!("[{id}] needs scripting API {want}; this build has {GW_SCRIPT_API_VERSION} - not loaded"),
                );
                s.log(&format!(
                    "script [{id}] refused: it needs scripting API {want}, this build has {GW_SCRIPT_API_VERSION}"
                ));
                return None;
            }
            let sc = &mut s.scripts[i];
            sc.src_hash = fnv(fnv(14695981039346656037, id.as_bytes()), src);
        }
        let env = match self.new_env() {
            Ok(e) => e,
            Err(e) => {
                self.report(i, "load", &e.to_string());
                self.unload(i);
                return None;
            }
        };
        self.st.borrow_mut().scripts[i].env = Some(env.clone());
        let chunk = self.lua.load(src).set_name(format!("@{id}")).set_mode(mlua::chunk::ChunkMode::Text).set_environment(env);
        let f = match chunk.into_function() {
            Ok(f) => f,
            Err(e) => {
                let msg = match &e {
                    mlua::Error::SyntaxError { message, .. } => message.clone(),
                    o => o.to_string(),
                };
                self.report(i, "load", &msg);
                self.unload(i);
                return None;
            }
        };
        if self.pcall(i, &f, MultiValue::new(), "load").is_none() {
            self.unload(i);
            return None;
        }
        let (line, log, scene_kind) = {
            let s = self.st.borrow();
            let sc = &s.scripts[i];
            (
                format!(
                    "loaded {}{}{} ({}){}",
                    sc.id,
                    if sc.version.is_empty() { "" } else { " " },
                    sc.version,
                    sc.origin,
                    if sc.gameplay { " [gameplay]" } else { "" }
                ),
                format!("script: loaded {} from {}{}", sc.id, entry, if sc.gameplay { " (gameplay)" } else { "" }),
                s.scene_kind,
            )
        };
        {
            let mut s = self.st.borrow_mut();
            s.print(GS_GREEN, &line);
            s.log(&log);
        }
        if let Some(f) = self.get_hook(i, "on_scene") {
            // catch it up with the scene it arrived in
            let name = self.st.borrow().host.borrow().scene_name(scene_kind);
            let args: MultiValue =
                [Value::Integer(scene_kind as i64), Value::String(self.lua.create_string(&name).ok()?)].into_iter().collect();
            self.pcall(i, &f, args, "on_scene");
        }
        Some(i)
    }

    /// `gs_load_script`: read the entry file, then [`Engine::load_text`].
    pub fn load_script(&self, id: &str, entry: &str, manifest: Option<&str>, origin: &str) -> Option<usize> {
        match read_limited(std::path::Path::new(entry), 4 << 20) {
            Some(src) => self.load_text(id, entry, &src, manifest, origin),
            None => {
                self.st.borrow_mut().print(GS_RED, &format!("[{id}] cannot read {entry}"));
                None
            }
        }
    }

    /// `gs_load_named`: a path relative to the scripts folder, "examples/foo", or an absolute path; a folder
    /// loads its mod.json + entry (default main.lua).
    pub fn load_named(&self, name: &str) -> Option<usize> {
        let scripts_dir = self.st.borrow().scripts_dir.clone();
        let is_abs = name.contains(':') || name.starts_with('\\') || name.starts_with('/');
        let path = if is_abs { PathBuf::from(name) } else { scripts_dir.join(name) };
        if path.is_dir() {
            let manifest = std::fs::read(path.join("mod.json")).ok().map(|b| String::from_utf8_lossy(&b).into_owned());
            let mut entry = "main.lua".to_string();
            if let Some(m) = &manifest {
                if let Some(e) = manifest::json_field(m, "entry", 96) {
                    entry = e;
                }
            }
            let full = path.join(&entry);
            let mut id = path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
            if let Some(m) = &manifest {
                if let Some(v) = manifest::json_field(m, "id", 1024) {
                    id = v;
                }
            }
            return self.load_script(&id, &full.to_string_lossy(), manifest.as_deref(), "scripts");
        }
        let mut p = path.to_string_lossy().into_owned();
        if p.len() < 4 || !p[p.len() - 4..].eq_ignore_ascii_case(".lua") {
            p.push_str(".lua");
        }
        let base = std::path::Path::new(&p).file_name()?.to_string_lossy().into_owned();
        let id = if base.len() > 4 { base[..base.len() - 4].to_string() } else { base };
        self.load_script(&id, &p, None, "scripts")
    }

    /// `gs_scan_scripts_dir`: every `*.lua` and every folder holding a mod.json (not `examples/`).
    pub fn scan_scripts_dir(&self) {
        let dir = self.st.borrow().scripts_dir.clone();
        let Ok(rd) = std::fs::read_dir(&dir) else { return };
        let mut names: Vec<(String, bool)> =
            rd.flatten().map(|e| (e.file_name().to_string_lossy().into_owned(), e.path().is_dir())).collect();
        names.sort();
        for (name, is_dir) in names {
            if name.starts_with('.') || name.eq_ignore_ascii_case("examples") {
                continue;
            }
            if is_dir {
                if dir.join(&name).join("mod.json").exists() {
                    self.load_named(&name);
                }
            } else if name.len() > 4 && name[name.len() - 4..].eq_ignore_ascii_case(".lua") {
                self.load_named(&name);
            }
        }
    }

    /// `gs_scan_mods`: `<mods>/<id>/scripts/*.lua` of every active mod; the mod's mod.json is the manifest and each
    /// script's id is `<mod id>/<file stem>`.
    pub fn scan_mods(&self) {
        let (mods_dir, active) = {
            let s = self.st.borrow();
            (s.mods_dir.clone(), s.active_mods.clone())
        };
        for id in active {
            let sp = mods_dir.join(&id).join("scripts");
            let Ok(rd) = std::fs::read_dir(&sp) else { continue };
            let manifest = std::fs::read(mods_dir.join(&id).join("mod.json"))
                .ok()
                .map(|b| String::from_utf8_lossy(&b).into_owned());
            let mut files: Vec<String> = rd
                .flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|n| n.len() > 4 && n[n.len() - 4..].eq_ignore_ascii_case(".lua"))
                .collect();
            files.sort();
            for f in files {
                let sid = format!("{id}/{}", &f[..f.len() - 4]);
                let entry = sp.join(&f);
                self.load_script(&sid, &entry.to_string_lossy(), manifest.as_deref(), "mods");
            }
        }
    }

    // ---------------------------------------------------------------- the loop

    /// `gw_Script_SceneBegin` (the parts that belong to the script core; the engine-owned releases are the game's).
    pub fn scene_begin(&self, scene_kind: i32) {
        self.zones_release(0);
        self.contacts_reset("scene changed; watcher cancelled");
        self.st.borrow_mut().contacts.labels.clear();
        let (prev, name) = {
            let mut s = self.st.borrow_mut();
            let prev = s.scene_kind;
            if s.match_active {
                s.match_active = false;
                drop(s);
                self.hook_all("on_match_end", &[]);
                s = self.st.borrow_mut();
            }
            s.scene_kind = scene_kind;
            s.scene_epoch += 1;
            s.paused = false; // a scene change always resumes
            s.step = 0;
            let name = s.host.borrow().scene_name(scene_kind);
            (prev, name)
        };
        let n = self.st.borrow().scripts.len();
        for i in 0..n {
            if let Some(f) = self.get_hook(i, "on_scene") {
                let args: MultiValue = [
                    Value::Integer(scene_kind as i64),
                    Value::String(self.lua.create_string(&name).unwrap()),
                    Value::Integer(prev as i64),
                ]
                .into_iter()
                .collect();
                self.pcall(i, &f, args, "on_scene");
            }
        }
    }

    /// `gw_Script_Tick` (script side): poll keys, swap the draw lists, run `on_tick`.
    pub fn tick(&self) {
        {
            let mut s = self.st.borrow_mut();
            let focused = !s.console_open;
            s.key_prev = s.key_now;
            let host = s.host.clone();
            for vk in 1..256 {
                s.key_now[vk] = focused && host.borrow().key_down(vk);
            }
        }
        self.finish_draw();
        {
            let mut s = self.st.borrow_mut();
            let b = s.build;
            s.draw[b].clear();
        }
        self.rw_tick_top();
        self.update_want_events();
        self.hook_all("on_tick", &[]);
        // (gs_rw_tick_top comes before on_tick in the C build's loop; see below)
        // paused: no logic frame will run this tick, so there is no frame boundary for a pending savestate to wait for
        let apply = {
            let s = self.st.borrow();
            s.paused && s.step == 0 && !s.host.borrow().session()
        };
        if apply {
            self.apply_pending(true);
            self.st.borrow_mut().apply_set_motion();
        }
    }

    fn finish_draw(&self) {
        // on_draw completes the list; the overlay then shows it (gs_finish_draw)
        self.hook_all("on_draw", &[]);
        self.contact_draw();
        let mut s = self.st.borrow_mut();
        s.build = 1 - s.build;
    }

    /// The camera block of `gw_Script_FramePost`: tell the owner a move or path finished, drop ownership once the
    /// camera is attached again.
    fn camera_tick(&self) {
        let (completed, kind, owner, state, last) = {
            let s = self.st.borrow();
            (
                s.game.camera_script_completion(),
                s.game.camera_script_completion_kind(),
                s.camera_owner,
                s.game.camera_script_state(),
                s.camera_completion,
            )
        };
        if completed > last && owner > 0 {
            let o = (owner - 1) as usize;
            let kind_name = if kind == 2 { "path" } else { "move" };
            let id = self.st.borrow().script_id(Some(o));
            self.st.borrow().log(&format!("script [{id}] camera {kind_name} complete"));
            if let Some(f) = self.get_hook(o, "on_camera_complete") {
                let a: MultiValue = [Value::String(self.lua.create_string(kind_name).unwrap())].into_iter().collect();
                self.pcall(o, &f, a, "on_camera_complete");
            }
        }
        let mut s = self.st.borrow_mut();
        s.camera_completion = completed; // a rewind may move the counter backward
        if s.camera_owner != 0 && state == 0 {
            let id = s.script_id(Some((s.camera_owner - 1) as usize));
            s.log(&format!("script [{id}] camera restored (attach complete)"));
            s.camera_owner = 0;
            s.camera_task_owner = 0;
        }
    }

    /// `gw_Script_PadApply`: the script layer over the four pad samples just read; returns the driven-port mask.
    pub fn pad_apply(&self, st: &mut [crate::pad::PadSample; 4], driven: u32) -> u32 {
        let mut s = self.st.borrow_mut();
        let session = s.host.borrow().session();
        let d = s.pads.apply(st, driven, session);
        let log: Vec<String> = s.pads.log.drain(..).collect();
        for l in log {
            s.log(&l);
        }
        d
    }

    /// `gw_Script_Iterations`: logic frames this tick runs (rewind bursts, pause, step)
    pub fn iterations(&self, count: i32) -> i32 {
        self.st.borrow_mut().iterations(count)
    }

    /// `gw_Script_FramePre`
    pub fn frame_pre(&self) {
        let resim = {
            let mut s = self.st.borrow_mut();
            s.rw.frame_kind = 0;
            s.rw.sfx_claimed = 0;
            s.rw.q_pos = 0;
            s.rw.in_frame = true;
            let mut resim = false;
            if s.rw.resim_run > 0 {
                resim = s.rw.resim_run > 1;
                s.rw.frame_kind = if resim { 1 } else { 2 };
                s.game.snap_lab_resim(resim);
                s.rw.resim_run -= 1;
            }
            resim
        };
        if !resim {
            self.st.borrow_mut().rwt_frame();
            self.apply_pending(false);
            self.st.borrow_mut().apply_set_motion();
        }
        self.st.borrow_mut().rw_frame();
        if !resim {
            self.hook_all("on_frame_pre", &[]);
        }
    }

    /// `gw_Script_FramePost` (the script-visible bookkeeping): per-slot action tracking, match start, `on_frame`, tasks.
    pub fn frame_post(&self) {
        let (start, kind) = {
            let mut s = self.st.borrow_mut();
            let kind = s.rw.frame_kind;
            s.ev.post_pending = true;
            if s.match_active {
                s.game.zones_frame(0);
            }
            s.rw.in_frame = false;
            if s.rw.began {
                // the frame's sounds are all in the log now
                let tag = s.ring_now();
                let e = s.log_entry_mut(tag);
                if e.tag == tag {
                    e.sfx_open = false;
                }
            }
            if kind != 1 {
                s.pads.frame_consumed();
            }
            s.rw.frame_kind = 0;
            s.ev.post_pending = false;
            s.frame += 1;
            let mut any = false;
            for slot in 0..6 {
                if crate::game::players_present(&*s.game, slot) {
                    let a = s.game.fighter_i(slot as i32, crate::game::SI_ACTION);
                    any = true;
                    if a != s.last_action[slot] {
                        s.last_action[slot] = a;
                        s.state_frame[slot] = 0;
                    } else {
                        s.state_frame[slot] += 1;
                    }
                } else {
                    s.last_action[slot] = -1;
                    s.state_frame[slot] = 0;
                }
            }
            if kind == 1 {
                // a frame the Lab re-simulates: its bookkeeping, none of its hooks
                if s.match_active {
                    s.match_frame += 1;
                }
                return;
            }
            let start = any && !s.match_active;
            if start {
                s.match_active = true;
                s.match_frame = 0;
            } else if s.match_active {
                s.match_frame += 1;
            }
            if kind == 2 {
                let now_ms = s.host.borrow().now_ms();
                s.rw.last_ms = now_ms - s.rw.t0;
                let (now, lm, pages, lf, ms) = (s.ring_now(), s.rw.last_load_ms, s.game.rw_last_load_pages(), s.rw.last_frames, s.rw.last_ms);
                s.log(&format!(
                    "rewind: to frame {now}: keyframe load {lm:.2} ms ({pages} pages), {lf} frame(s) re-simulated, {ms:.1} ms in all"
                ));
            }
            (start, kind)
        };
        let _ = kind;
        if kind != 1 {
            self.zones_dispatch();
        }
        if kind != 1 {
            self.deadline_frame();
        }
        self.camera_tick();
        self.contact_tick();
        self.dispatch_events();
        if start {
            self.hook_all("on_match_start", &[]);
        }
        self.hook_all("on_frame", &[]);
        self.run_tasks();
    }

    // ---------------------------------------------------------------- console

    /// `gw_Script_Exec`: one console line; returns (status, captured output).
    pub fn exec_capture(&self, line: &str) -> (i32, String) {
        {
            let mut s = self.st.borrow_mut();
            s.print(GS_GREY, &format!("> {line}"));
            s.capture = Some(String::new());
        }
        let rc = self.exec(line);
        let out = self.st.borrow_mut().capture.take().unwrap_or_default();
        (rc, out)
    }

    /// `gs_exec`: a built-in command, a script command, or Lua ("= expr" prints a value).
    pub fn exec(&self, line_in: &str) -> i32 {
        let trimmed = line_in.trim_start_matches([' ', '\t']);
        let line = trimmed.trim_end_matches(['\r', '\n', ' ']);
        if line.is_empty() {
            return 0;
        }
        if let Some(code) = line.strip_prefix('=') {
            return self.exec_lua(code);
        }
        let (cmd, arg) = match line.split_once(' ') {
            Some((c, a)) => (c, a.trim_start_matches(' ')),
            None => (line, ""),
        };
        let is = |c: &str| cmd.eq_ignore_ascii_case(c);
        let print = |rgba: u32, t: &str| self.st.borrow_mut().print(rgba, t);
        if is("help") || is("?") {
            self.cmd_help();
            return 0;
        }
        if is("api") {
            print(GS_WHITE, &format!("scripting API {GW_SCRIPT_API_VERSION} (docs/scripting.md), Lua 5.4.7"));
            return 0;
        }
        if is("label") {
            self.st.borrow().host.borrow_mut().set_run_label(arg);
            if arg.is_empty() {
                print(GS_GREEN, "label cleared");
            } else {
                print(GS_GREEN, &format!("label: {arg}"));
            }
            return 0;
        }
        if is("echo") {
            print(GS_WHITE, arg);
            return 0;
        }
        if is("clear") {
            self.st.borrow_mut().lines.clear();
            return 0;
        }
        if is("quit") {
            let s = self.st.borrow();
            s.log("console: quit");
            s.host.borrow_mut().request_quit();
            return 0;
        }
        if is("scripts") {
            let rows: Vec<(bool, String)> = {
                let s = self.st.borrow();
                s.scripts
                    .iter()
                    .enumerate()
                    .filter(|(i, sc)| sc.used && *i != s.console)
                    .map(|(_, sc)| {
                        (
                            sc.disabled,
                            format!(
                                "  {:<24} {:<8} {:<8} {}{}{}",
                                sc.id,
                                if sc.version.is_empty() { "-" } else { &sc.version },
                                sc.origin,
                                if sc.gameplay { "gameplay " } else { "" },
                                if sc.disabled { "OFF " } else { "" },
                                sc.entry
                            ),
                        )
                    })
                    .collect()
            };
            for (off, r) in rows {
                print(if off { GS_RED } else { GS_WHITE }, &r);
            }
            return 0;
        }
        if is("load") {
            return if self.load_named(arg).is_some() { 0 } else { -1 };
        }
        if is("unload") {
            let found = self.st.borrow().find(arg);
            let console = self.st.borrow().console;
            match found {
                Some(i) if i != console => {
                    self.unload(i);
                    print(GS_GREEN, &format!("unloaded {arg}"));
                    return 0;
                }
                _ => {
                    print(GS_RED, &format!("no script \"{arg}\""));
                    return -1;
                }
            }
        }
        if is("reload") {
            let mut rc = 0;
            let n = self.st.borrow().scripts.len();
            for i in 0..n {
                let snap = {
                    let s = self.st.borrow();
                    let sc = &s.scripts[i];
                    if !sc.used || i == s.console || (!arg.is_empty() && !sc.id.eq_ignore_ascii_case(arg)) {
                        continue;
                    }
                    (sc.id.clone(), sc.entry.clone(), sc.origin.clone(), sc.gameplay, sc.rollback_safe)
                };
                let (id, entry, origin, gameplay, rb) = snap;
                let ok = if origin.starts_with('m') || origin.starts_with('e') {
                    // keep the manifest flags
                    let m = format!("{{\"gameplay\": {gameplay}, \"rollback_safe\": {rb}}}");
                    self.load_script(&id, &entry, Some(&m), &origin).is_some()
                } else {
                    self.load_named(&entry).is_some()
                };
                if !ok {
                    rc = -1;
                }
            }
            return rc;
        }
        if is("contacts") || is("trace") || is("wait") {
            return self.contact_console(cmd, arg);
        }
        if is("frame") {
            let f = self.st.borrow().frame;
            print(GS_WHITE, &f.to_string());
            return 0;
        }
        if is("pause") || is("resume") || is("step") || is("savestate") || is("loadstate") {
            return self.exec_lua(&format!("gd.{cmd}({arg})"));
        }
        let hit = {
            let s = self.st.borrow();
            s.commands.iter().find(|c| c.name.eq_ignore_ascii_case(cmd)).map(|c| (c.func.clone(), c.script, c.name.clone()))
        };
        if let Some((f, script, name)) = hit {
            let a: MultiValue = [Value::String(self.lua.create_string(arg).unwrap())].into_iter().collect();
            return if self.pcall(script, &f, a, &name).is_some() { 0 } else { -1 };
        }
        self.exec_lua(line_in)
    }

    fn cmd_help(&self) {
        let mut s = self.st.borrow_mut();
        s.print(GS_YELLOW, "commands (anything else runs as Lua; \"= expr\" prints a value):");
        for l in [
            "  help | scripts | load <name> | unload <id> | reload [id]",
            "  state | frame | items | fx | pause | resume | step [n]",
            "  savestate [1-4] | loadstate [1-4]",
            "  shot [path] | label <text> | echo <text> | clear | api | quit",
        ] {
            s.print(GS_WHITE, l);
        }
        let rows: Vec<String> = s
            .commands
            .iter()
            .map(|c| format!("  {}  {}  [{}]", c.name, c.help, s.script_id(Some(c.script))))
            .collect();
        for r in rows {
            s.print(GS_WHITE, &r);
        }
    }

    /// `gs_exec_lua`: an expression first ("return <code>"), then a statement; results are printed.
    pub fn exec_lua(&self, code: &str) -> i32 {
        let (console, env) = {
            let s = self.st.borrow();
            (s.console, s.scripts[s.console].env.clone().unwrap())
        };
        let load = |src: &str| {
            self.lua.load(src).set_name("=console").set_mode(mlua::chunk::ChunkMode::Text).set_environment(env.clone()).into_function()
        };
        let f = match load(&format!("return {code}")) {
            Ok(f) => f,
            Err(_) => match load(code) {
                Ok(f) => f,
                Err(e) => {
                    let msg = match &e {
                        mlua::Error::SyntaxError { message, .. } => message.clone(),
                        o => o.to_string(),
                    };
                    self.st.borrow_mut().print(GS_RED, &msg);
                    return -1;
                }
            },
        };
        let Some(vals) = self.pcall(console, &f, MultiValue::new(), "console") else { return -1 };
        for v in vals.iter() {
            let text = self.print_value(v, 0);
            self.st.borrow_mut().print(GS_WHITE, &text);
        }
        0
    }

    /// `gs_print_value`: tables as `{k=v, ...}` to depth 2, non-integral numbers as `%.4g`.
    fn print_value(&self, v: &Value, depth: u32) -> String {
        let tostring: Function = self.lua.globals().get("tostring").unwrap();
        let ts = |v: &Value| -> String {
            tostring.call::<mlua::LuaString>(v.clone()).map(|s| s.to_string_lossy().to_string()).unwrap_or_default()
        };
        match v {
            Value::Table(t) if depth < 2 => {
                let mut out = String::from("{");
                let mut count = 0;
                for pair in t.pairs::<Value, Value>().flatten() {
                    if out.len() >= 1800 - 8 {
                        break;
                    }
                    if count > 0 {
                        out.push_str(", ");
                    }
                    count += 1;
                    let val = self.print_value(&pair.1, depth + 1);
                    match &pair.0 {
                        Value::String(s) => out.push_str(&format!("{}={}", s.to_string_lossy(), val)),
                        k => out.push_str(&format!("[{}]={}", ts(k), val)),
                    }
                }
                out.push('}');
                out
            }
            Value::Number(n) => fmt_g(*n, 4),
            other => ts(other),
        }
    }

    // ---------------------------------------------------------------- inspection helpers for hosts and tests

    pub fn console_lines(&self) -> Vec<String> {
        self.st.borrow().lines.iter().map(|(_, l)| l.clone()).collect()
    }

    /// Evaluate Lua in the console environment and return the value (a test/host convenience, not a C entry point).
    pub fn eval<T: mlua::FromLuaMulti>(&self, code: &str) -> LuaResult<T> {
        let env = {
            let s = self.st.borrow();
            s.scripts[s.console].env.clone().unwrap()
        };
        let f = self.lua.load(code).set_name("=eval").set_environment(env).into_function()?;
        let prev = self.st.borrow().cur;
        {
            let mut s = self.st.borrow_mut();
            s.cur = Some(s.console);
            s.arm_budget();
        }
        let r = f.call::<T>(());
        self.st.borrow_mut().cur = prev;
        r
    }
}

/// mlua errors carry their traceback; the console line is the first line (the C report shows the whole message,
/// which for luaL_traceback is multi-line too, so keep everything).
fn clean_error(e: &mlua::Error) -> String {
    match e {
        mlua::Error::RuntimeError(m) => m.clone(),
        mlua::Error::CallbackError { cause, traceback } => format!("{}\n{}", clean_error(cause), traceback),
        other => other.to_string(),
    }
}

fn read_limited(path: &std::path::Path, max: u64) -> Option<Vec<u8>> {
    let md = std::fs::metadata(path).ok()?;
    if md.len() > max {
        return None;
    }
    std::fs::read(path).ok()
}

/// `gs_find_ui_dir`: `ui/` beside the script, else one level up.
fn find_ui_dir(entry: &str) -> String {
    let mut dir = PathBuf::from(entry);
    for _ in 0..2 {
        if !dir.pop() || dir.as_os_str().is_empty() {
            return String::new();
        }
        let cand = dir.join("ui");
        if cand.is_dir() {
            return cand.to_string_lossy().into_owned();
        }
    }
    String::new()
}

#[allow(dead_code)]
fn _unused(_: RegistryKey) {}

/// Registry key of the pointer to the shared state, for the raw count hook.
static HOOK_KEY: [u8; 1] = [0];

/// `gs_count_hook` (raw): spend HOOK_STEP of the call's budget; when it is gone, or the wall-clock backstop is past (not
/// in turbo), `luaL_error` "ran too long". Runs inside the Lua VM, so it holds no Rust borrow when it raises.
unsafe extern "C-unwind" fn count_hook(l: *mut mlua::ffi::lua_State, _ar: *mut mlua::ffi::lua_Debug) {
    use mlua::ffi;
    ffi::lua_rawgetp(l, ffi::LUA_REGISTRYINDEX, HOOK_KEY.as_ptr() as *const std::ffi::c_void);
    let p = ffi::lua_touserdata(l, -1) as *const RefCell<State>;
    ffi::lua_pop(l, 1);
    if p.is_null() {
        return;
    }
    let msg = {
        let Ok(mut s) = (*p).try_borrow_mut() else { return };
        s.budget -= HOOK_STEP;
        let (now, turbo) = {
            let h = s.host.borrow();
            (h.now_ms(), h.turbo())
        };
        if s.budget < 0 || (!turbo && now > s.deadline) {
            s.budget = 0; // keep failing until control is back with the engine
            Some(format!(
                "ran too long (limit: {} instructions or {} ms per call)\0",
                s.budget_per_call, s.ms_per_call as i64
            ))
        } else {
            None
        }
    };
    if let Some(m) = msg {
        ffi::luaL_error(l, b"%s\0".as_ptr() as *const std::ffi::c_char, m.as_ptr() as *const std::ffi::c_char);
    }
}

/// The trampoline every native `gd` function runs under (see [`Engine::native`]).
const NATIVE_WRAPPER: &str = "local ERR = ...
local function ret(...)
  if (...) == ERR then error((select(2, ...)), 2) end
  return ...
end
return function(native)
  return function(...) return ret(native(...)) end
end
";

/// The message a script sees for a native's error.
pub fn error_text(e: &mlua::Error) -> String {
    match e {
        mlua::Error::RuntimeError(m) => m.clone(),
        mlua::Error::CallbackError { cause, .. } => error_text(cause),
        mlua::Error::SyntaxError { message, .. } => message.clone(),
        other => other.to_string(),
    }
}
