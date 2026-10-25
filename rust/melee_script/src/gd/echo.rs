//! Echoes and the fighter history (`gw_script_echo.inc`): echo_add, echo_remove, echoes, fighter_history,
//! fighter_history_depth. Authoritative history, handles and target memory stay in the game half
//! (`ScriptGame::echo_*`, `fighter_history_*`); a rule is nine ints, floats as bit patterns.
use super::camera::{bitsf, fbits};
use super::caps::{neg1, one};
use super::hit_rules::{element, element_name, int_field, keys, MOVES};
use super::{lua_err, reg, slot_arg, TableExt};
use crate::args::Args;
use crate::engine::{Engine, Shared, State};
use mlua::{Table, Value};

type LuaResult<T> = mlua::Result<T>;

pub const GS_ECHO_MAX: usize = 8;
pub const GS_ECHO_FIELDS: usize = 9;

pub type EchoRule = [i32; GS_ECHO_FIELDS];
#[derive(Clone, Default)]
pub struct EchoRules {
    pub count: usize,
    pub rules: [EchoRule; GS_ECHO_MAX],
}

fn move_tag(name: &'static str, v: &Value) -> LuaResult<i32> {
    let s = neg1(one(name, v).check_string(1))?;
    if s == "any" {
        return Ok(-1);
    }
    if let Some(i) = MOVES[..9].iter().position(|m| *m == s) {
        return Ok(i as i32);
    }
    // neutral-air is the precise action selector, not all aerial attacks
    match s.as_str() {
        "nair" => Ok(10),
        "fair" => Ok(11),
        "bair" => Ok(12),
        "uair" => Ok(13),
        "dair" => Ok(14),
        _ => Err(lua_err("unknown echo move tag")),
    }
}
/// `gs_echo_rule`
pub fn echo_rule(name: &'static str, at: &Table) -> LuaResult<EchoRule> {
    const KEYS: [&str; 8] = ["handle", "delay", "match", "damage", "knockback", "element", "once_per_move", "sub"];
    const MATCHES: [&str; 3] = ["move", "element", "airborne"];
    keys(at, &KEYS)?;
    let mut b: EchoRule = [0; GS_ECHO_FIELDS];
    b[2] = -1;
    b[3] = -1;
    b[4] = -1;
    b[7] = -1;
    b[5] = fbits(1.0);
    b[6] = fbits(1.0);
    b[8] = 1;
    b[0] = int_field(name, at, "handle", 0, 0, i32::MAX as i64)?;
    b[1] = int_field(name, at, "delay", 1, 1, 60)?;
    let m: Value = at.get("match")?;
    if !m.is_nil() {
        let m = neg1(one(name, &m).check_table(1))?;
        keys(&m, &MATCHES)?;
        let v: Value = m.get("move")?;
        if !v.is_nil() {
            b[2] = move_tag(name, &v)?;
        }
        let v: Value = m.get("element")?;
        if !v.is_nil() {
            b[3] = element(name, &v)?;
        }
        let v: Value = m.get("airborne")?;
        if !v.is_nil() {
            b[4] = super::caps::check_boolean(name, &v)? as i32;
        }
    }
    for (j, k) in ["damage", "knockback"].iter().enumerate() {
        let v: Value = at.get(*k)?;
        let n = if v.is_nil() { 1.0 } else { neg1(one(name, &v).check_number(1))? };
        if !n.is_finite() || !(0.1..=4.0).contains(&n) {
            return Err(lua_err("echo scale must be finite and 0.1..4"));
        }
        b[5 + j] = fbits(n as f32);
    }
    let v: Value = at.get("element")?;
    if !v.is_nil() {
        b[7] = element(name, &v)?;
    }
    let v: Value = at.get("once_per_move")?;
    if !v.is_nil() {
        b[8] = super::caps::check_boolean(name, &v)? as i32;
    }
    Ok(b)
}
fn echo_owner(s: &State, entity: i32) -> LuaResult<()> {
    let owner = s.game.echo_owner(entity);
    if owner != 0 && owner != s.cur.map_or(0, |c| c as i32 + 1) {
        return Err(lua_err("echoes owned by another script"));
    }
    Ok(())
}
fn echo_sub(a: &Args, i: usize) -> LuaResult<bool> {
    if a.none_or_nil(i) {
        return Ok(false);
    }
    match a.get(i) {
        Value::Boolean(b) => Ok(*b),
        o => Err(a.err(i, &format!("boolean expected, got {}", crate::args::typename(Some(o))))),
    }
}
/// `gs_echo_parse`: `{rules = {...}}`
pub fn echo_parse(name: &'static str, at: &Table) -> LuaResult<EchoRules> {
    let v: Value = at.get("rules")?;
    let t = neg1(one(name, &v).check_table(1))?;
    let n = t.raw_len();
    if n > GS_ECHO_MAX {
        return Err(lua_err("echo capacity is 8 per fighter entity"));
    }
    let mut r = EchoRules { count: n, ..Default::default() };
    for i in 0..n {
        let e: Value = t.raw_get(i + 1)?;
        let et = neg1(one(name, &e).check_table(1))?;
        r.rules[i] = echo_rule(name, &et)?;
    }
    Ok(r)
}
/// `gs_echo_prepare`: canonicalise zero handles before the journal is stored; one caller cursor covers every echo operation
pub fn echo_prepare(s: &State, entity: i32, r: &mut EchoRules, next: &mut i32) -> LuaResult<()> {
    echo_owner(s, entity)?;
    for i in 0..r.count {
        let mut b = r.rules[i];
        if b[0] == 0 {
            for j in 0..GS_ECHO_MAX as i32 {
                let h = s.game.echo_read(entity, j, 0);
                if h == 0 {
                    continue;
                }
                if r.rules[..i].iter().any(|x| x[0] == h) {
                    continue;
                }
                if (1..GS_ECHO_FIELDS).all(|k| b[k] == s.game.echo_read(entity, j, k as i32)) {
                    b[0] = h;
                    break;
                }
            }
        }
        if b[0] == 0 {
            if *next < 1 || *next == i32::MAX {
                return Err(lua_err("echo handle space exhausted"));
            }
            b[0] = *next;
            *next += 1;
        }
        if r.rules[..i].iter().any(|x| x[0] == b[0]) {
            return Err(lua_err("duplicate echo handle"));
        }
        for e in 0..12 {
            for j in 0..GS_ECHO_MAX as i32 {
                if s.game.echo_read(e, j, 0) == b[0] && e != entity {
                    return Err(lua_err("echo handle belongs to another entity"));
                }
            }
        }
        if b[0] >= *next {
            if b[0] == i32::MAX {
                return Err(lua_err("echo handle space exhausted"));
            }
            *next = b[0] + 1;
        }
        r.rules[i] = b;
    }
    Ok(())
}
/// `gs_echo_pair`
pub fn echo_pair(entity_a: i32, a: &EchoRules, entity_b: i32, b: &EchoRules) -> LuaResult<()> {
    if entity_a == entity_b {
        return Err(lua_err("sim_commit permits one echo replacement per entity"));
    }
    for x in &a.rules[..a.count] {
        for y in &b.rules[..b.count] {
            if x[0] > 0 && x[0] == y[0] {
                return Err(lua_err("sim_commit duplicate echo handle across entities"));
            }
        }
    }
    Ok(())
}
/// `gs_echo_apply`
pub fn echo_apply(s: &mut State, entity: i32, owner: i32, r: &EchoRules) -> bool {
    let flat: Vec<i32> = r.rules.iter().flat_map(|x| x.iter().copied()).collect();
    s.game.echo_replace(entity, owner, r.count as i32, &flat) != 0
}

fn echo_add(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<i64> {
    let port = slot_arg(&a, 1)?;
    let mut s = st.borrow_mut();
    s.require_gameplay("echo_add")?;
    let t = a.check_table(2)?;
    let b = echo_rule("echo_add", &t)?;
    let sv: Value = t.get("sub")?;
    let sub = match sv {
        Value::Nil => false,
        Value::Boolean(b) => b,
        o => return Err(lua_err(format!("bad argument #-1 to 'echo_add' (boolean expected, got {})", crate::args::typename(Some(&o))))),
    };
    let entity = port as i32 * 2 + sub as i32;
    echo_owner(&s, entity)?;
    if b[0] != 0 {
        return Err(lua_err("echo_add does not accept a preassigned handle"));
    }
    s.rw_branch();
    let owner = s.cur.map_or(0, |c| c as i32 + 1);
    let handle = s.game.echo_add_with_handle(entity, owner, &b[1..], 0);
    if handle == 0 {
        return Err(lua_err("echo add refused (capacity or fighter unavailable)"));
    }
    Ok(handle as i64)
}
fn echo_remove(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<bool> {
    let h = a.check_integer(1)?;
    let mut s = st.borrow_mut();
    s.require_gameplay("echo_remove")?;
    if h < 1 || h > i32::MAX as i64 {
        return Err(lua_err("invalid echo handle"));
    }
    s.rw_branch();
    let owner = s.cur.map_or(0, |c| c as i32 + 1);
    Ok(s.game.echo_remove(h as i32, owner) != 0)
}
fn history_depth(_: &mlua::Lua, st: &Shared, _a: Args) -> LuaResult<i64> {
    Ok(st.borrow().game.fighter_history_depth() as i64)
}
fn fighter_history(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Value> {
    const FLOATS: [&str; 5] = ["action_frame", "x", "y", "z", "facing"];
    const HF: [&str; 7] = ["ax", "ay", "az", "bx", "by", "bz", "radius"];
    let raw_age = a.check_integer(2)?;
    let port = slot_arg(&a, 1)? as i32;
    let entity = port * 2 + echo_sub(&a, 3)? as i32;
    let s = st.borrow();
    let g = &*s.game;
    if raw_age < 0 || raw_age >= g.fighter_history_depth() as i64 {
        return Err(a.err(2, "history age 0..60"));
    }
    let age = raw_age as i32;
    let rd = |i: i32, f: i32| g.fighter_history_read(entity, age, i, f);
    if rd(-1, 0) == 0 {
        return Ok(Value::Nil);
    }
    let t = lua.create_table()?;
    t.int("age", age as i64)?;
    t.int("generation", rd(-1, 1) as i64)?;
    t.int("attack_id", rd(-1, 11) as i64)?;
    t.int("action_state", rd(-1, 2) as i64)?;
    t.int("move", rd(-1, 3) as i64)?;
    t.boolean("airborne", rd(-1, 4) != 0)?;
    t.boolean("grounded", rd(-1, 4) == 0)?;
    for (j, k) in FLOATS.iter().enumerate() {
        t.num(k, bitsf(rd(-1, 5 + j as i32)) as f64)?;
    }
    let hb = lua.create_table()?;
    let mut dense = 0;
    for i in 0..5 {
        if rd(i, 0) == 0 {
            continue;
        }
        let h = lua.create_table()?;
        let source = rd(i, 18);
        h.int("source_entity", source as i64)?;
        h.int("owner_port", (source / 2 + 1) as i64)?;
        h.boolean("owner_sub", source % 2 != 0)?;
        h.int("source_index", rd(i, 19) as i64)?;
        h.int("identity", rd(i, 1) as i64)?;
        h.num("damage", bitsf(rd(i, 2)) as f64)?;
        for (k, f) in [("angle", 3), ("growth", 4), ("base", 5), ("fixed", 6), ("element", 7), ("original_element", 15), ("move_tag", 16)] {
            h.int(k, rd(i, f) as i64)?;
        }
        h.boolean("grounded", rd(i, 17) != 0)?;
        for (j, k) in HF.iter().enumerate() {
            h.num(k, bitsf(rd(i, 8 + j as i32)) as f64)?;
        }
        dense += 1;
        hb.raw_set(dense, h)?;
    }
    t.set("hitboxes", hb)?;
    Ok(Value::Table(t))
}
fn echoes(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Table> {
    const AERIAL: [&str; 5] = ["nair", "fair", "bair", "uair", "dair"];
    let e = slot_arg(&a, 1)? as i32 * 2 + echo_sub(&a, 2)? as i32;
    let s = st.borrow();
    let g = &*s.game;
    let out = lua.create_table()?;
    out.boolean("journal", true)?;
    out.int("capacity", GS_ECHO_MAX as i64)?;
    out.int("owner", g.echo_owner(e) as i64)?;
    let mut dense = 0;
    for i in 0..GS_ECHO_MAX as i32 {
        let b: Vec<i32> = (0..GS_ECHO_FIELDS as i32).map(|j| g.echo_read(e, i, j)).collect();
        if b[0] == 0 {
            continue;
        }
        let t = lua.create_table()?;
        t.int("handle", b[0] as i64)?;
        t.int("delay", b[1] as i64)?;
        t.num("damage", bitsf(b[5]) as f64)?;
        t.num("knockback", bitsf(b[6]) as f64)?;
        t.boolean("once_per_move", b[8] != 0)?;
        if b[7] >= 0 {
            t.string("element", element_name(b[7]))?;
        }
        let m = lua.create_table()?;
        let mv = if b[2] < 0 {
            "any"
        } else if (10..=14).contains(&b[2]) {
            AERIAL[(b[2] - 10) as usize]
        } else if b[2] < 9 {
            MOVES[b[2] as usize]
        } else {
            "unknown"
        };
        m.string("move", mv)?;
        m.string("element", element_name(b[3]))?;
        if b[4] >= 0 {
            m.boolean("airborne", b[4] != 0)?;
        }
        t.set("match", m)?;
        dense += 1;
        out.raw_set(dense, t)?;
    }
    Ok(out)
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    reg(e, gd, "echo_add", echo_add)?;
    reg(e, gd, "echo_remove", echo_remove)?;
    reg(e, gd, "fighter_history_depth", history_depth)?;
    reg(e, gd, "fighter_history", fighter_history)?;
    reg(e, gd, "echoes", echoes)?;
    Ok(())
}
