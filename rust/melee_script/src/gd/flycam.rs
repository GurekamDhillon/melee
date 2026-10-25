//! Fly cursor attack and stage camera parameters (`gw_script_fly_attack.inc`, `gw_script_camera_params.inc`):
//! fly_target, fly_attack, fly_clear, fly_state, camera_params. The cursor/attack state lives in the game half
//! (`ScriptGame::geno_fly_*`); this keeps the per-slot cursor owner.
use super::camera::{bitsf, fbits};
use super::caps::{key_str, neg1, one, pairs};
use super::{lua_err, reg, slot_arg, TableExt};
use crate::args::Args;
use crate::engine::{Engine, Shared, State, GS_MAX_SCRIPTS};
use mlua::{Table, Value};

type LuaResult<T> = mlua::Result<T>;

#[derive(Default)]
pub struct FlyCursor {
    owner: [i32; 6],
}

impl FlyCursor {
    /// `gs_fly_cursor_reset` also resets the game's cursor state; the owners are cleared here
    pub fn reset(&mut self) {
        self.owner = [0; 6];
    }
}

impl State {
    pub fn fly_cursor_owner(&self, slot: usize) -> i32 {
        self.fly_cursor.owner[slot]
    }
    pub fn fly_cursor_clear(&mut self, slot: usize) {
        self.fly_cursor.owner[slot] = 0;
    }
}

fn caller(s: &State) -> i32 {
    match s.cur {
        Some(c) if c == s.console => GS_MAX_SCRIPTS as i32 + 1,
        Some(c) => c as i32 + 1,
        None => 1,
    }
}
fn available(s: &mut State, slot: usize, owner: i32) -> bool {
    if s.game.geno_fly_state(slot as i32, 0) == 0 && s.game.geno_fly_state(slot as i32, 1) == 0 {
        s.fly_cursor.owner[slot] = 0;
    }
    s.fly_cursor.owner[slot] == 0 || s.fly_cursor.owner[slot] == owner
}
fn cursor_error(rc: i32) -> &'static str {
    match rc {
        -4 => "cursor belongs to another script",
        -3 => "invalid cursor parameters",
        -1 => "no fighter on that port",
        -2 => "that fighter's state cannot fly (dead, held, respawning)",
        _ => "",
    }
}
fn target_set(s: &mut State, slot: usize, owner: i32, x: f32, y: f32) -> i32 {
    if !available(s, slot, owner) {
        return -4;
    }
    let rc = s.fly_set(slot as i32, 1);
    if rc < 0 {
        return rc;
    }
    let rc = s.game.geno_fly_target(slot as i32, fbits(x), fbits(y));
    if rc >= 0 {
        s.fly_cursor.owner[slot] = owner;
    }
    rc
}

#[derive(Default, Clone, Copy)]
struct Burst {
    active: i32,
    gap: i32,
    kb: i32,
    kbg: i32,
    angle: i32,
    every_frame: bool,
}
fn attack_cfg(s: &mut State, slot: usize, owner: i32, on: bool, damage: i32, radius: f32, b: Burst) -> i32 {
    if !available(s, slot, owner) {
        return -4;
    }
    if on {
        let rc = s.fly_set(slot as i32, 1);
        if rc < 0 {
            return rc;
        }
    }
    s.rw_branch();
    let rc = s.game.geno_fly_attack_config(
        slot as i32,
        on as i32 | (b.every_frame as i32) << 1,
        damage,
        fbits(radius),
        b.active | (b.gap << 16),
        b.kb | (b.kbg << 16),
        b.angle,
    );
    if rc >= 0 {
        s.fly_cursor.owner[slot] = owner;
    }
    rc
}

fn fly_target(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<bool> {
    let slot = slot_arg(&a, 1)?;
    let (x, y) = (a.check_number(2)? as f32, a.check_number(3)? as f32);
    if !x.is_finite() || !y.is_finite() || x.abs() > 49000.0 || y.abs() > 49000.0 {
        return Err(lua_err("fly_target requires finite coordinates within +/-49000"));
    }
    let mut s = st.borrow_mut();
    s.require_offline("fly_target")?;
    let owner = caller(&s);
    let rc = target_set(&mut s, slot, owner, x, y);
    if rc < 0 {
        return Err(lua_err(format!("fly_target: {}", cursor_error(rc))));
    }
    Ok(true)
}

fn opt_int(t: &Table, key: &str, lo: i64, hi: i64, def: i64) -> LuaResult<i32> {
    let v: Value = t.get(key)?;
    match v {
        Value::Nil => Ok(def as i32),
        Value::Integer(n) if (lo..=hi).contains(&n) => Ok(n as i32),
        _ => Err(lua_err(format!("fly_attack: {key} must be an integer {lo}..{hi}"))),
    }
}
fn fly_attack(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<bool> {
    let slot = slot_arg(&a, 1)?;
    let on = match a.get(2) {
        Value::Boolean(b) => *b,
        o => return Err(a.err(2, &format!("boolean expected, got {}", crate::args::typename(Some(o))))),
    };
    let (mut damage, mut radius) = (3i64, 6f32);
    let mut b = Burst::default();
    if let Value::Table(t) = a.get(3) {
        let r: Value = t.get("radius")?;
        match r {
            Value::Nil => {}
            Value::Integer(_) | Value::Number(_) => radius = neg1(one("fly_attack", &r).check_number(1))? as f32,
            _ => return Err(lua_err("fly_attack: radius must be a number 1..30")),
        }
        damage = opt_int(t, "damage", 1, 30, 3)? as i64;
        b.active = opt_int(t, "active", 1, 600, 0)?;
        b.gap = opt_int(t, "gap", 0, 3000, 0)?;
        b.kb = opt_int(t, "kb", 1, 2000, 0)?;
        b.kbg = opt_int(t, "kbg", 1, 2000, 0)?;
        b.angle = opt_int(t, "angle", 1, 361, 0)?;
        let e: Value = t.get("every_frame")?;
        match e {
            Value::Nil => {}
            Value::Boolean(v) => b.every_frame = v,
            _ => return Err(lua_err("fly_attack: every_frame must be a boolean")),
        }
    } else {
        damage = a.opt_integer(3, 3)?;
        radius = a.opt_number(4, 6.0)? as f32;
    }
    if !(1..=30).contains(&damage) || !radius.is_finite() || !(1.0..=30.0).contains(&radius) {
        return Err(lua_err("fly_attack requires damage 1..30 and radius 1..30"));
    }
    let mut s = st.borrow_mut();
    s.require_offline("fly_attack")?;
    let owner = caller(&s);
    let rc = attack_cfg(&mut s, slot, owner, on, damage as i32, radius, b);
    if rc < 0 {
        return Err(lua_err(format!("fly_attack: {}", cursor_error(rc))));
    }
    Ok(s.game.geno_fly_state(slot as i32, 0) != 0)
}
fn fly_clear(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<()> {
    let slot = slot_arg(&a, 1)?;
    let mut s = st.borrow_mut();
    s.require_offline("fly_clear")?;
    let owner = caller(&s);
    if !available(&mut s, slot, owner) {
        return Err(lua_err("fly_clear: cursor belongs to another script"));
    }
    s.rw_branch();
    s.game.geno_fly_clear(slot as i32);
    s.fly_cursor.owner[slot] = 0;
    Ok(())
}
fn fly_state(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Table> {
    let slot = slot_arg(&a, 1)? as i32;
    let s = st.borrow();
    let g = &*s.game;
    let t = lua.create_table()?;
    t.boolean("flying", g.geno_fly_get(slot) > 0)?;
    t.boolean("attacking", g.geno_fly_state(slot, 0) != 0)?;
    t.boolean("targeting", g.geno_fly_state(slot, 1) != 0)?;
    t.int("damage", g.geno_fly_state(slot, 2) as i64)?;
    t.num("radius", bitsf(g.geno_fly_state(slot, 3)) as f64)?;
    t.num("x", bitsf(g.geno_fly_state(slot, 4)) as f64)?;
    t.num("y", bitsf(g.geno_fly_state(slot, 5)) as f64)?;
    t.int("pulses", g.geno_fly_state(slot, 6) as i64)?;
    t.boolean("every_frame", g.geno_fly_state(slot, 7) != 0)?;
    for (k, f) in [("active", 8), ("gap", 9), ("kb", 10), ("kbg", 11), ("angle", 12), ("phase", 13)] {
        t.int(k, g.geno_fly_state(slot, f) as i64)?;
    }
    Ok(t)
}

fn camera_params(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Table> {
    const NAMES: [&str; 10] =
        ["min_dist", "max_depth", "fov", "fixed_zoom", "track_smooth", "track_ratio", "tilt", "pan", "yaw_gain", "pitch_gain"];
    const LO: [f32; 10] = [1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, -180.0, 0.0, 0.0];
    const HI: [f32; 10] = [49000.0, 49000.0, 89.0, 10.0, 10.0, 10.0, 89.0, 180.0, 10.0, 10.0];
    let mut s = st.borrow_mut();
    s.require_offline("camera_params")?;
    if !s.match_active {
        return Err(lua_err("camera_params requires an active match"));
    }
    let owner = s.game.camera_param_owner();
    let me = s.cur.map_or(0, |c| c as i32 + 1);
    if owner != 0 && owner != me {
        return Err(lua_err("camera params owned by another script"));
    }
    let mut previous = [0f32; 10];
    for i in 0..10 {
        previous[i] = bitsf(s.game.camera_param_read(i as i32));
    }
    let mut v = previous;
    let mut mask = 0;
    if !a.none_or_nil(1) {
        let t = a.check_table(1)?;
        for (k, val) in pairs(&t)? {
            let Some(i) = key_str(&k).and_then(|k| NAMES.iter().position(|n| *n == k)) else {
                return Err(lua_err("unknown camera parameter"));
            };
            v[i] = neg1(one("camera_params", &val).check_number(1))? as f32;
            if !(v[i] >= LO[i] && v[i] <= HI[i]) {
                return Err(lua_err(format!("{} must be in {:.6}..{:.6}", NAMES[i], LO[i] as f64, HI[i] as f64)));
            }
            mask |= 1 << i;
        }
        if v[0] > v[1] {
            return Err(lua_err("min_dist must not exceed max_depth"));
        }
        if mask & (1 << 2) != 0 && mask & (1 << 6) != 0 && v[2] != v[6] {
            return Err(lua_err("tilt is the stage's FOV alias; fov and tilt must agree"));
        }
    }
    // an empty table is a no-op; nil restores. Everything is validated before the rewind branch.
    if mask != 0 || a.none_or_nil(1) {
        s.rw_branch();
        let bits: Vec<i32> = v.iter().map(|f| fbits(*f)).collect();
        if s.game.camera_params(me, mask, &bits) == 0 {
            return Err(lua_err("camera params rejected"));
        }
    }
    let out = lua.create_table()?;
    for i in 0..10 {
        out.num(NAMES[i], previous[i] as f64)?;
    }
    Ok(out)
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    reg(e, gd, "fly_target", fly_target)?;
    reg(e, gd, "fly_attack", fly_attack)?;
    reg(e, gd, "fly_clear", fly_clear)?;
    reg(e, gd, "fly_state", fly_state)?;
    reg(e, gd, "camera_params", camera_params)?;
    Ok(())
}
