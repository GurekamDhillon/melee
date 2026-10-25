//! The `gd.*` API (`gs_gd_funcs` in `pc/platform/gw_script.c`). Each submodule ports one area; `register_all`
//! installs them on the shared template `gd` table.
use crate::args::Args;
use crate::engine::{Engine, Shared};
use mlua::{IntoLuaMulti, Lua, Result as LuaResult, Table, Value};

pub mod arena;
pub mod bench;
pub mod camera;
pub mod caps;
pub mod contacts;
pub mod cpu_ctl;
pub mod crit;
pub mod deadline;
pub mod echo;
pub mod entity;
pub mod combo;
pub mod core;
pub mod fighter;
pub mod flycam;
pub mod hit_rules;
pub mod history;
pub mod lab;
pub mod labmisc;
pub mod labstate;
pub mod mods;
pub mod perf;
pub mod pad;
pub mod players;
pub mod rec;
pub mod savestate;
pub mod shock;
pub mod skill;
pub mod zones;

/// `gs_buttons` (only these twelve appear in `gd.buttons`).
pub const BUTTONS: &[(&str, u32)] = &[
    ("A", 0x0100), ("B", 0x0200), ("X", 0x0400), ("Y", 0x0800), ("START", 0x1000), ("L", 0x0040),
    ("R", 0x0020), ("Z", 0x0010), ("UP", 0x0008), ("DOWN", 0x0004), ("LEFT", 0x0001), ("RIGHT", 0x0002),
];

pub(crate) fn lua_err(msg: impl Into<String>) -> mlua::Error {
    mlua::Error::runtime(msg.into())
}

/// Install one native function on `gd`: the closure gets the Lua state, the shared engine state and the checked-argument reader.
pub(crate) fn reg<R, F>(e: &Engine, gd: &Table, name: &'static str, f: F) -> LuaResult<()>
where
    R: IntoLuaMulti,
    F: Fn(&Lua, &Shared, Args) -> LuaResult<R> + 'static,
{
    gd.set(name, e.native(name, f)?)
}

pub fn register_all(e: &Engine, gd: &Table) -> LuaResult<()> {
    core::register(e, gd)?;
    fighter::register(e, gd)?;
    camera::register(e, gd)?;
    arena::register(e, gd)?;
    caps::register(e, gd)?;
    hit_rules::register(e, gd)?;
    entity::register(e, gd)?;
    echo::register(e, gd)?;
    zones::register(e, gd)?;
    deadline::register(e, gd)?;
    cpu_ctl::register(e, gd)?;
    flycam::register(e, gd)?;
    contacts::register(e, gd)?;
    shock::register(e, gd)?;
    skill::register(e, gd)?;
    bench::register(e, gd)?;
    crit::register(e, gd)?;
    mods::register(e, gd)?;
    pad::register(e, gd)?;
    players::register(e, gd)?;
    lab::register(e, gd)?;
    labmisc::register(e, gd)?;
    labstate::register(e, gd)?;
    savestate::register(e, gd)?;
    history::register(e, gd)?;
    combo::register(e, gd)?;
    perf::register(e, gd)?;
    rec::register(e, gd)?;
    Ok(())
}

/// `gs_push_kit`: the kit art table. The kit (gw_kit.c) is not ported yet; scripts get the table with no entries.
pub fn push_kit(lua: &Lua) -> LuaResult<Table> {
    lua.create_table()
}

/// `gs_push_ui`: the Atlas UI table (gw_script_ui*.inc), not ported yet.
pub fn push_ui(lua: &Lua) -> LuaResult<Table> {
    lua.create_table()
}

// ---- table builders (gs_setnum / gs_setint / gs_setstr / gs_setbool)
pub(crate) trait TableExt {
    fn num(&self, k: &str, v: f64) -> LuaResult<()>;
    fn int(&self, k: &str, v: i64) -> LuaResult<()>;
    fn string(&self, k: &str, v: &str) -> LuaResult<()>;
    fn boolean(&self, k: &str, v: bool) -> LuaResult<()>;
}
impl TableExt for Table {
    fn num(&self, k: &str, v: f64) -> LuaResult<()> {
        self.set(k, Value::Number(v))
    }
    fn int(&self, k: &str, v: i64) -> LuaResult<()> {
        self.set(k, Value::Integer(v))
    }
    fn string(&self, k: &str, v: &str) -> LuaResult<()> {
        self.set(k, v)
    }
    fn boolean(&self, k: &str, v: bool) -> LuaResult<()> {
        self.set(k, v)
    }
}

/// `gs_slot_arg`: player/port numbers are 1-6.
pub(crate) fn slot_arg(a: &Args, i: usize) -> LuaResult<usize> {
    let p = a.check_integer(i)?;
    if !(1..=6).contains(&p) {
        return Err(lua_err(format!("player/port numbers are 1-6 (got {})", p as i32)));
    }
    Ok((p - 1) as usize)
}
