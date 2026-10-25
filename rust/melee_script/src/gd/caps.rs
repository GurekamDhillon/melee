//! Scalar fighter capabilities (`gw_script_fighter_caps.inc`, `gw_script_fighter_caps_armor.inc`): fighter_caps, fighter_effect,
//! fighter_armour, fighter_armor, give_item, nearest_opponent, opponents_in_radius, fighter_timed_status, fighter_interrupt.
//! Authoritative records live in the game half (`ScriptGame::caps_*`, `armor_*`, `intr_win_*`); floats cross as bit patterns.
use super::arena::cur_owner;
use super::camera::{bitsf, fbits};
use super::{lua_err, reg, TableExt};
use crate::args::{typename, Args};
use crate::engine::{Engine, Shared, State};
use mlua::{MultiValue, Table, Value};

type LuaResult<T> = mlua::Result<T>;

pub(crate) fn entity(a: &Args, i: usize) -> LuaResult<i32> {
    let n = a.check_integer(i)?;
    if !(1..=12).contains(&n) {
        return Err(lua_err("fighter entity must be 1-12 (1-6 primary, 7-12 secondary)"));
    }
    Ok(n as i32)
}
fn frames(a: &Args, i: usize) -> LuaResult<i32> {
    let n = a.check_integer(i)?;
    if !(0..=3600).contains(&n) {
        return Err(lua_err("fighter duration must be 0-3600 logic frames"));
    }
    Ok(n as i32)
}
pub(crate) fn number(a: &Args, i: usize, low: f64, high: f64) -> LuaResult<f32> {
    let n = a.check_number(i)?;
    if !n.is_finite() || n < low || n > high {
        return Err(lua_err("fighter value must be finite and in range"));
    }
    Ok(n as f32)
}
pub(crate) fn write(s: &mut State, name: &str) -> LuaResult<()> {
    s.require_offline(name)?;
    if !s.match_active {
        return Err(lua_err("fighter capability requires an active match"));
    }
    Ok(())
}

/// A table value checked as the C code does at stack index -1 (`luaL_checkinteger(L,-1)` and friends report "#-1").
pub(crate) fn one(name: &'static str, v: &Value) -> Args {
    Args::new(name, MultiValue::from_iter([v.clone()]))
}
pub(crate) fn neg1<T>(r: LuaResult<T>) -> LuaResult<T> {
    r.map_err(|e| match e {
        mlua::Error::RuntimeError(m) => mlua::Error::RuntimeError(m.replacen("bad argument #1", "bad argument #-1", 1)),
        e => e,
    })
}
pub(crate) fn check_boolean(name: &'static str, v: &Value) -> LuaResult<bool> {
    match v {
        Value::Boolean(b) => Ok(*b),
        o => Err(lua_err(format!("bad argument #-1 to '{name}' (boolean expected, got {})", typename(Some(o))))),
    }
}
pub(crate) fn key_str(k: &Value) -> Option<String> {
    match k {
        Value::String(s) => Some(s.to_string_lossy().to_string()),
        _ => None,
    }
}
/// `luaL_checkoption(L, at, NULL, opts)` over an argument
fn option(a: &Args, i: usize, opts: &[&str]) -> LuaResult<usize> {
    let s = a.check_string(i)?;
    opts.iter().position(|o| *o == s).ok_or_else(|| a.err(i, &format!("invalid option '{s}'")))
}
pub(crate) fn pairs(t: &Table) -> LuaResult<Vec<(Value, Value)>> {
    t.clone().pairs::<Value, Value>().collect()
}

const CAPS_KEYS: [&str; 5] = ["shield", "air_dodge", "run", "grab", "specials"];
const EFFECT_KINDS: [&str; 4] = ["intangible", "invincible", "metal", "size"];
const IW_EXITS: [&str; 15] = [
    "jab", "tilt", "smash", "aerial", "special", "grab", "jump", "dash", "crouch", "turn", "walk", "escape", "shield", "air_dodge",
    "air_jump",
];
const ARMOR_TYPES: [&str; 6] = ["knockback", "damage_threshold", "knockback_threshold", "super", "hit_count", "damage_pool"];

fn fighter_caps(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<MultiValue> {
    let e = entity(&a, 1)?;
    if a.len() == 1 {
        let s = st.borrow();
        let first = if s.game.caps_movement_read(e, 0) == 0 {
            Value::Nil
        } else {
            let t = lua.create_table()?;
            t.int("air_jumps", s.game.caps_movement_read(e, 1) as i64)?;
            let flags = s.game.caps_movement_read(e, 2);
            for (i, k) in CAPS_KEYS.iter().enumerate() {
                t.boolean(k, flags & (1 << i) != 0)?;
            }
            Value::Table(t)
        };
        let retail = s.game.caps_movement_read(e, 3);
        let second = if retail < 0 { Value::Nil } else { Value::Integer(retail as i64) };
        return Ok(MultiValue::from_iter([first, second]));
    }
    let mut s = st.borrow_mut();
    write(&mut s, "fighter_caps")?;
    let (mut jumps, mut flags) = (-1i32, 0i32);
    if !a.none_or_nil(2) {
        let t = a.check_table(2)?;
        for (k, v) in pairs(&t)? {
            let key = key_str(&k);
            if key.as_deref() == Some("air_jumps") {
                let n = neg1(one("fighter_caps", &v).check_integer(1))?;
                if !(0..=8).contains(&n) {
                    return Err(lua_err("air_jumps must be 0-8"));
                }
                jumps = n as i32;
            } else {
                let Some(i) = key.as_deref().and_then(|k| CAPS_KEYS.iter().position(|c| *c == k)) else {
                    return Err(lua_err("unknown fighter capability"));
                };
                if check_boolean("fighter_caps", &v)? {
                    flags |= 1 << i;
                }
            }
        }
    }
    s.rw_branch();
    let owner = cur_owner(&s);
    Ok(MultiValue::from_iter([Value::Boolean(s.game.caps_movement(e, owner, jumps, flags) != 0)]))
}

fn fighter_effect(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Value> {
    let e = entity(&a, 1)?;
    let kind = option(&a, 2, &EFFECT_KINDS)? as i32;
    if a.len() == 2 {
        let s = st.borrow();
        let fr = s.game.caps_effect_read(e, kind, 0);
        if fr == 0 {
            return Ok(Value::Nil);
        }
        let t = lua.create_table()?;
        t.int("frames", fr as i64)?;
        t.num("value", bitsf(s.game.caps_effect_read(e, kind, 1)) as f64)?;
        return Ok(Value::Table(t));
    }
    let mut s = st.borrow_mut();
    write(&mut s, "fighter_effect")?;
    let (lo, hi) = if kind == 3 { (0.25, 4.0) } else { (0.0, 1.0) };
    let value = number(&a, 3, lo, hi)?;
    let fr = frames(&a, 4)?;
    s.rw_branch();
    let owner = cur_owner(&s);
    Ok(Value::Boolean(s.game.caps_effect(e, owner, kind, fbits(value), fr) != 0))
}

fn fighter_armour(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Value> {
    let e = entity(&a, 1)?;
    if a.len() == 1 {
        let s = st.borrow();
        if s.game.caps_armour_read(e, 0) == 0 {
            return Ok(Value::Nil);
        }
        let t = lua.create_table()?;
        t.num("damage", bitsf(s.game.caps_armour_read(e, 1)) as f64)?;
        t.num("knockback", bitsf(s.game.caps_armour_read(e, 2)) as f64)?;
        return Ok(Value::Table(t));
    }
    let mut s = st.borrow_mut();
    write(&mut s, "fighter_armour")?;
    let (mut damage, mut kb) = (0f32, 0f32);
    if !a.none_or_nil(2) {
        let t = a.check_table(2)?;
        for (k, v) in pairs(&t)? {
            let key = key_str(&k);
            let n = neg1(one("fighter_armour", &v).check_number(1))?;
            if !n.is_finite() || !(0.0..=1000.0).contains(&n) {
                return Err(lua_err("fighter value must be finite and in range"));
            }
            match key.as_deref() {
                Some("damage") => damage = n as f32,
                Some("knockback") => kb = n as f32,
                _ => return Err(lua_err("unknown armour threshold")),
            }
        }
    }
    s.rw_branch();
    let owner = cur_owner(&s);
    Ok(Value::Boolean(s.game.caps_armour(e, owner, fbits(damage), fbits(kb)) != 0))
}

fn give_item(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<MultiValue> {
    let e = entity(&a, 1)?;
    let mut s = st.borrow_mut();
    write(&mut s, "give_item")?;
    let kind = if let Value::String(x) = a.get(2) {
        if x.as_bytes() != b"random" {
            return Err(lua_err("item must be a retail kind ID or 'random'"));
        }
        -1
    } else {
        let n = a.check_integer(2)?;
        if !(0..=255).contains(&n) {
            return Err(lua_err("item kind outside retail ID range"));
        }
        n as i32
    };
    s.rw_branch();
    let owner = cur_owner(&s);
    let r = s.game.caps_give_item(e, owner, kind);
    let mut out = vec![Value::Boolean(r > 0)];
    if r <= 0 {
        out.push(Value::String(lua.create_string(s.game.caps_give_item_reason(r))?));
    }
    Ok(MultiValue::from_iter(out))
}

fn nearest_opponent(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Value> {
    let e = entity(&a, 1)?;
    let exclude = if a.none_or_nil(2) { 0 } else { entity(&a, 2)? };
    let s = st.borrow();
    let t = if s.match_active { s.game.caps_opponent(e, fbits(100000.0), 0, exclude) } else { 0 };
    Ok(if t != 0 { Value::Integer(t as i64) } else { Value::Nil })
}

fn opponents_in_radius(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Table> {
    let e = entity(&a, 1)?;
    let exclude = if a.none_or_nil(3) { 0 } else { entity(&a, 3)? };
    let radius = number(&a, 2, 0.0, 100000.0)?;
    let t = lua.create_table()?;
    let s = st.borrow();
    if s.match_active {
        for n in 1..=12 {
            let id = s.game.caps_opponent(e, fbits(radius), n, exclude);
            if id == 0 {
                break;
            }
            t.raw_set(n as i64, id as i64)?;
        }
    }
    Ok(t)
}

fn fighter_timed_status(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Value> {
    let e = entity(&a, 1)?;
    let c = a.check_integer(2)?;
    if !(1..=4).contains(&c) {
        return Err(lua_err("fighter status channel must be 1-4"));
    }
    let ch = c as i32;
    if a.len() == 2 {
        let s = st.borrow();
        let fr = s.game.caps_status_read(e, ch, 0);
        if fr == 0 {
            return Ok(Value::Nil);
        }
        let t = lua.create_table()?;
        t.int("frames", fr as i64)?;
        t.num("value", bitsf(s.game.caps_status_read(e, ch, 1)) as f64)?;
        return Ok(Value::Table(t));
    }
    let mut s = st.borrow_mut();
    write(&mut s, "fighter_timed_status")?;
    let value = number(&a, 3, -100000.0, 100000.0)?;
    let fr = frames(&a, 4)?;
    s.rw_branch();
    let owner = cur_owner(&s);
    Ok(Value::Boolean(s.game.caps_status(e, ch, owner, fbits(value), fr) != 0))
}

fn fighter_interrupt(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Value> {
    let e = entity(&a, 1)?;
    if a.len() == 1 {
        let s = st.borrow();
        if s.game.intr_win_read(e, 0) <= 0 {
            return Ok(Value::Nil);
        }
        let t = lua.create_table()?;
        t.int("frames", s.game.intr_win_read(e, 0) as i64)?;
        t.int("owner", s.game.intr_win_read(e, 1) as i64)?;
        t.int("key", s.game.intr_win_read(e, 3) as i64)?;
        t.int("opened", s.game.intr_win_read(e, 4) as i64)?;
        t.int("cancels", s.game.intr_win_read(e, 5) as i64)?;
        let m = s.game.intr_win_read(e, 2);
        let x = lua.create_table()?;
        for (i, n) in IW_EXITS.iter().enumerate() {
            if m & (1 << i) != 0 {
                x.set(*n, true)?;
            }
        }
        t.set("exits", x)?;
        return Ok(Value::Table(t));
    }
    let mut s = st.borrow_mut();
    write(&mut s, "fighter_interrupt")?;
    let t = a.check_table(2)?;
    let (mut fr, mut exits, mut flags) = (0i32, 0x7FFFi32, 0i32);
    for (k, v) in pairs(&t)? {
        match key_str(&k).as_deref() {
            Some("frames") => {
                let n = neg1(one("fighter_interrupt", &v).check_integer(1))?;
                if !(0..=3600).contains(&n) {
                    return Err(lua_err("interrupt window must be 0-3600 logic frames"));
                }
                fr = n as i32;
            }
            Some("exits") => {
                let Value::Table(xt) = &v else {
                    return Err(lua_err(format!(
                        "bad argument #-1 to 'fighter_interrupt' (table expected, got {})",
                        typename(Some(&v))
                    )));
                };
                exits = 0;
                for (_, x) in pairs(xt)? {
                    let name = match &x {
                        Value::String(s) => Some(s.to_string_lossy().to_string()),
                        _ => None,
                    };
                    let Some(i) = name.and_then(|n| IW_EXITS.iter().position(|c| *c == n)) else {
                        return Err(lua_err("unknown interrupt exit"));
                    };
                    exits |= 1 << i;
                }
            }
            Some("guard") => {
                if check_boolean("fighter_interrupt", &v)? {
                    flags |= 1;
                }
            }
            Some("restore_jumps") => {
                if check_boolean("fighter_interrupt", &v)? {
                    flags |= 2;
                }
            }
            _ => return Err(lua_err("unknown interrupt option")),
        }
    }
    s.rw_branch();
    let owner = cur_owner(&s);
    Ok(Value::Boolean(s.game.intr_win_set(e, owner, fr, exits, flags) != 0))
}

fn armor_num(g: &dyn crate::game::ScriptGame, e: i32, ty: i32, f: i32) -> f64 {
    bitsf(g.armor_read(e, ty, f)) as f64
}
fn armor_int(v: &Value, lo: i64, hi: i64) -> LuaResult<i32> {
    let n = neg1(one("fighter_armor", v).check_integer(1))?;
    if n < lo || n > hi {
        return Err(lua_err("armour integer outside range"));
    }
    Ok(n as i32)
}
fn armor_bound(v: &Value) -> LuaResult<f32> {
    let n = neg1(one("fighter_armor", v).check_number(1))?;
    if !n.is_finite() || !(-1.0..=100000.0).contains(&n) {
        return Err(lua_err("armour animation bound outside range"));
    }
    if n < 0.0 && n != -1.0 {
        return Err(lua_err("armour animation bound must be -1 or nonnegative"));
    }
    Ok(n as f32)
}

fn fighter_armor(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Value> {
    let e = entity(&a, 1)?;
    if a.len() == 1 {
        let s = st.borrow();
        let out = lua.create_table()?;
        let mut i = 1;
        for ty in 1..=6 {
            if s.game.armor_read(e, ty, 0) == 0 {
                continue;
            }
            let g = &*s.game;
            let t = lua.create_table()?;
            t.string("type", ARMOR_TYPES[ty as usize - 1])?;
            t.num("value", armor_num(g, e, ty, 1))?;
            t.num("remaining", armor_num(g, e, ty, 2))?;
            t.int("frames", g.armor_read(e, ty, 3) as i64)?;
            t.int("state", g.armor_read(e, ty, 4) as i64)?;
            t.num("from", armor_num(g, e, ty, 5))?;
            t.num("to", armor_num(g, e, ty, 6))?;
            let d = g.armor_read(e, ty, 7);
            t.string("direction", if d > 0 { "front" } else if d < 0 { "back" } else { "any" })?;
            t.boolean("enabled", g.armor_read(e, ty, 8) != 0)?;
            out.raw_set(i, t)?;
            i += 1;
        }
        return Ok(Value::Table(out));
    }
    let mut s = st.borrow_mut();
    write(&mut s, "fighter_armor")?;
    if a.none_or_nil(2) {
        s.rw_branch();
        let owner = cur_owner(&s);
        return Ok(Value::Boolean(s.game.armor_clear(e, owner, 0) != 0));
    }
    let t = a.check_table(2)?;
    let tv: Value = t.get("type")?;
    let ty = {
        let aa = one("fighter_armor", &tv);
        neg1(option(&aa, 1, &ARMOR_TYPES))? as i32 + 1
    };
    let (mut clear, mut frames_, mut state, mut direction) = (false, 0i32, -1i32, 0i32);
    let (mut has_value, mut value) = (false, 1.0f64);
    let (mut from, mut to) = (-1f32, -1f32);
    for (k, v) in pairs(&t)? {
        let Some(key) = key_str(&k) else {
            return Err(lua_err("armour option names must be strings"));
        };
        match key.as_str() {
            "type" => {}
            "value" => {
                value = neg1(one("fighter_armor", &v).check_number(1))?;
                if !value.is_finite() || !(0.0..=1000.0).contains(&value) {
                    return Err(lua_err("armour value outside range"));
                }
                has_value = true;
            }
            "frames" => frames_ = armor_int(&v, 0, 36000)?,
            "state" => state = armor_int(&v, -1, 65535)?,
            "from" => from = armor_bound(&v)?,
            "to" => to = armor_bound(&v)?,
            "direction" => {
                let aa = one("fighter_armor", &v);
                direction = match neg1(option(&aa, 1, &["any", "front", "back"]))? {
                    1 => 1,
                    2 => -1,
                    _ => 0,
                };
            }
            "clear" => clear = check_boolean("fighter_armor", &v)?,
            _ => return Err(lua_err("unknown armour option")),
        }
    }
    if from >= 0.0 && to >= 0.0 && from > to {
        return Err(lua_err("armour animation range is reversed"));
    }
    if !clear {
        if !has_value && ty != 4 {
            return Err(lua_err("armour value required"));
        }
        if ty != 4 && value <= 0.0 {
            return Err(lua_err("armour value must be positive"));
        }
        if ty == 5 && (value > 255.0 || value != (value as i32) as f64) {
            return Err(lua_err("hit_count value must be integer 1-255"));
        }
    }
    s.rw_branch();
    let owner = cur_owner(&s);
    let r = if clear {
        s.game.armor_clear(e, owner, ty)
    } else {
        s.game.armor_set(e, owner, ty, fbits(value as f32), frames_, state, fbits(from), fbits(to), direction)
    };
    Ok(Value::Boolean(r != 0))
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    reg(e, gd, "fighter_caps", fighter_caps)?;
    reg(e, gd, "fighter_effect", fighter_effect)?;
    reg(e, gd, "fighter_armour", fighter_armour)?;
    reg(e, gd, "fighter_armor", fighter_armor)?;
    reg(e, gd, "give_item", give_item)?;
    reg(e, gd, "nearest_opponent", nearest_opponent)?;
    reg(e, gd, "opponents_in_radius", opponents_in_radius)?;
    reg(e, gd, "fighter_timed_status", fighter_timed_status)?;
    reg(e, gd, "fighter_interrupt", fighter_interrupt)?;
    Ok(())
}
