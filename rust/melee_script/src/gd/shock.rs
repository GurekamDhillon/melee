//! Shock (`gw_script_shock.inc`): gd.shock. The status is game-side; events ride the skill ring.
use super::arena::cur_owner;
use super::camera::fbits;
use super::caps::{entity, neg1, number, one, write};
use super::crit::item_keys;
use super::{lua_err, reg, TableExt};
use crate::args::Args;
use crate::engine::{Engine, Shared};
use mlua::{Table, Value};

type LuaResult<T> = mlua::Result<T>;

fn range(name: &'static str, v: &Value, lo: i64, hi: i64, msg: &str) -> LuaResult<i32> {
    let n = neg1(one(name, v).check_integer(1))?;
    if n < lo || n > hi {
        return Err(lua_err(msg));
    }
    Ok(n as i32)
}

fn shock(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Value> {
    const KEYS: [&str; 5] = ["frames", "charges", "hitstun", "bonus", "stack"];
    let e = entity(&a, 1)?;
    if a.len() == 1 {
        let s = st.borrow();
        let g = &*s.game;
        let ow = g.shock_read(e, 0);
        if ow == 0 {
            return Ok(Value::Nil);
        }
        let t = lua.create_table()?;
        t.int("owner", ow as i64)?;
        t.int("frames", g.shock_read(e, 1) as i64)?;
        t.int("charges", g.shock_read(e, 2) as i64)?;
        t.int("bonus", g.shock_read(e, 3) as i64)?;
        t.num("hitstun_multiplier", f32::from_bits(g.shock_read(e, 4) as u32) as f64)?;
        t.int("hits", g.shock_read(e, 5) as i64)?;
        t.int("applied", g.shock_read(e, 6) as i64)?;
        return Ok(Value::Table(t));
    }
    let mut s = st.borrow_mut();
    write(&mut s, "shock")?;
    let owner = cur_owner(&s);
    let ow = s.game.shock_read(e, 0);
    if ow != 0 && ow != owner {
        return Ok(Value::Boolean(false));
    }
    if a.none_or_nil(2) {
        s.rw_branch();
        return Ok(Value::Boolean(s.game.shock_set(e, owner, 0, 0, 0, 0, 4) != 0));
    }
    let t = a.check_table(2)?;
    item_keys(&t, &KEYS)?;
    let (mut frames, mut charges, mut bonus, mut flags, mut mult) = (300, 1, 0, 1, 1.5f32);
    let v: Value = t.get("frames")?;
    if !v.is_nil() {
        frames = range("shock", &v, 1, 3600, "shock frames must be 1-3600")?;
    }
    let v: Value = t.get("charges")?;
    if !v.is_nil() {
        charges = range("shock", &v, 1, 8, "shock charges must be 1-8")?;
    }
    let v: Value = t.get("hitstun")?;
    if !v.is_nil() {
        mult = number(&one("shock", &v), 1, 1.0, 4.0)?;
    }
    let v: Value = t.get("bonus")?;
    if !v.is_nil() {
        bonus = range("shock", &v, 0, 120, "shock bonus must be 0-120")?;
    }
    let v: Value = t.get("stack")?;
    if !v.is_nil() && super::caps::check_boolean("shock", &v)? {
        flags = 2;
    }
    s.rw_branch();
    Ok(Value::Boolean(s.game.shock_set(e, owner, frames, charges, fbits(mult), bonus, flags) != 0))
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    reg(e, gd, "shock", shock)
}

#[allow(dead_code)]
fn _t(t: &Table) -> LuaResult<()> {
    t.int("x", 0)
}
