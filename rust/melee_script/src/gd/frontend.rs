//! Frontend and presentation services: scene_launch, scene_clear, training_select, screenshot, hud_visible, launch_request,
//! launch_cancel (gw_script.c, gw_script_launch.inc). The scene-launch text, the Training select choice, the screenshot writer and
//! the status HUD are platform/frontend services behind [`crate::Host`]; the game starts the scene (`ScriptGame::launch_scene`).
use super::core::data_path;
use super::{lua_err, reg, TableExt};
use crate::args::{typename, Args};
use crate::engine::{Engine, Shared};
use mlua::{MultiValue, Table, Value};

type LuaResult<T> = mlua::Result<T>;

/// `gs_pending_launch`, the launch request state (`gs_launch_*`)
pub struct Frontend {
    /// game mode to start at the next tick, outside any frame (-1 none)
    pub pending_launch: i32,
    pub launch_consumed: bool,
    pub launch_pending: bool,
    pub launch_owner: i32,
    pub hud_owner: i32,
}
impl Default for Frontend {
    fn default() -> Self {
        Frontend { pending_launch: -1, launch_consumed: false, launch_pending: false, launch_owner: 0, hud_owner: 0 }
    }
}

/// `gd.scene_launch("mode=training;p1=fox")` or `gd.scene_launch{mode="training", p1="fox"}`
fn scene_launch(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Value> {
    let mut s = st.borrow_mut();
    s.require_gameplay("scene_launch")?;
    let mut text = String::new();
    if let Value::Table(t) = a.get(1) {
        // mode first, the rest in any order
        let m: Value = t.get("mode")?;
        if let Value::String(ms) = &m {
            text.push_str(&format!("mode={}", ms.to_string_lossy()));
        }
        for kv in t.clone().pairs::<Value, Value>() {
            if text.len() >= 511 {
                break;
            }
            let (k, v) = kv?;
            if let Value::String(k) = &k {
                let k = k.to_string_lossy();
                if k != "mode" {
                    let tostring: mlua::Function = lua.globals().get("tostring")?;
                    let vs: mlua::String = tostring.call(v)?;
                    text.push_str(&format!("{}{}={}", if text.is_empty() { "" } else { ";" }, k, vs.to_string_lossy()));
                }
            }
        }
        text.truncate(511);
    } else {
        text = a.check_string(1)?.chars().take(511).collect();
    }
    let host = s.host.clone();
    host.borrow_mut().scene_launch_set_text(Some(&text));
    let mode = host.borrow_mut().scene_launch_boot_game_mode();
    if mode < 0 {
        return Err(lua_err(format!(
            "gd.scene_launch: \"{text}\" was refused or names no scene; see log (six slots require direct VS/LAB and CPU slots 5-6)"
        )));
    }
    s.frontend.pending_launch = mode; // acted on at the next tick, outside any frame
    Ok(Value::String(lua.create_string(&text)?))
}
fn scene_clear(_: &mlua::Lua, st: &Shared, _a: Args) -> LuaResult<()> {
    st.borrow().host.borrow_mut().scene_launch_set_text(None);
    Ok(())
}
fn training_select(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Value> {
    let host = st.borrow().host.clone();
    if !a.none_or_nil(1) {
        let v = a.check_string(1)?;
        match v.as_str() {
            "kit" => host.borrow_mut().set_training_select(true),
            "native" => host.borrow_mut().set_training_select(false),
            _ => return Err(lua_err(format!("gd.training_select: \"kit\" or \"native\" (got \"{v}\")"))),
        }
    }
    let kit = host.borrow().training_select();
    Ok(Value::String(lua.create_string(if kit { "kit" } else { "native" })?))
}
/// `gd.screenshot(name)`: into the script's data folder (a bare file name, .png added)
fn screenshot(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<MultiValue> {
    let n = a.opt_string(1, "shot")?;
    let name = if n.len() > 4 && n[n.len() - 4..].eq_ignore_ascii_case(".png") { n.clone() } else { format!("{n}.png") };
    let path = data_path(st, &name)?;
    let ok = st.borrow().host.borrow_mut().screenshot(&path);
    Ok(MultiValue::from_iter([Value::Boolean(ok), Value::String(lua.create_string(path.to_string_lossy().as_bytes())?)]))
}
fn hud_visible(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<bool> {
    let mut s = st.borrow_mut();
    s.require_offline("hud_visible")?;
    let Some(cur) = s.cur.filter(|c| *c != s.console && s.scripts.get(*c).is_some_and(|x| x.used)) else {
        return Err(lua_err("hud_visible requires a gameplay mod script"));
    };
    if !a.none_or_nil(1) {
        let visible = match a.get(1) {
            Value::Boolean(b) => *b,
            o => return Err(a.err(1, &format!("boolean expected, got {}", typename(Some(o))))),
        };
        if s.frontend.hud_owner != 0 && s.frontend.hud_owner != cur as i32 + 1 {
            return Err(lua_err("HUD visibility belongs to another script"));
        }
        if !visible {
            if !s.match_active {
                return Err(lua_err("hiding HUD requires an active match"));
            }
            s.frontend.hud_owner = cur as i32 + 1;
        } else {
            s.frontend.hud_owner = 0;
        }
        let hidden = s.frontend.hud_owner != 0;
        s.host.borrow_mut().set_status_hud_hidden(hidden);
    }
    let v = s.host.borrow().status_hud_visible();
    Ok(v)
}
/// `gd.launch_request()`: the requested mission/maze/mod of the scene launch
fn launch_request(lua: &mlua::Lua, st: &Shared, _a: Args) -> LuaResult<Table> {
    let s = st.borrow();
    let info = s.host.borrow().scene_launch_info();
    let t = lua.create_table()?;
    t.string("mission", &info.mission)?;
    t.int("seed", info.maze_seed as i64)?;
    t.int("size", info.maze_size as i64)?;
    t.string("mod", &info.module)?;
    t.boolean("pending", s.frontend.launch_pending)?;
    Ok(t)
}
fn launch_cancel(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<()> {
    let mut s = st.borrow_mut();
    s.require_offline("launch_cancel")?;
    let me = s.scripts.get(s.cur.unwrap_or(0)).map_or(0, |x| x.stage_owner);
    if s.frontend.launch_owner != 0 && s.frontend.launch_owner != me {
        return Err(lua_err("launch_cancel requires the launch owner"));
    }
    s.frontend.launch_pending = false;
    s.frontend.launch_consumed = true;
    let why = a.opt_string(1, "script requested cancellation")?;
    s.log(&format!("launch: cancelled: {why}"));
    Ok(())
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    reg(e, gd, "scene_launch", scene_launch)?;
    reg(e, gd, "scene_clear", scene_clear)?;
    reg(e, gd, "training_select", training_select)?;
    reg(e, gd, "screenshot", screenshot)?;
    reg(e, gd, "hud_visible", hud_visible)?;
    reg(e, gd, "launch_request", launch_request)?;
    reg(e, gd, "launch_cancel", launch_cancel)?;
    Ok(())
}

impl Engine {
    /// the tick's launch: a `gd.scene_launch` is acted on outside any frame
    pub fn launch_tick(&self) {
        let mode = {
            let mut s = self.st.borrow_mut();
            let m = s.frontend.pending_launch;
            if m < 0 {
                return;
            }
            s.frontend.pending_launch = -1;
            s.log(&format!("script: launching scene (game mode {m})"));
            m
        };
        let mut s = self.st.borrow_mut();
        s.host.borrow_mut().hang_transition(true);
        s.game.launch_scene(mode);
    }
}

#[allow(dead_code)]
fn _x() {
    let _ = Table::clone;
}
