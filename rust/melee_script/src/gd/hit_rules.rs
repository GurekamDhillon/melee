//! Hit rules and fighter status bits (`gw_script_hit_rules.inc`): hit_rule_add, hit_rule_remove, hit_rules_clear, fighter_status,
//! hit_rules. Authoritative rules live in the game half (`ScriptGame::hit_rule*`); a rule is 16 scalar ints, floats as bit patterns.
use super::camera::{bitsf, fbits};
use super::caps::{check_boolean, key_str, neg1, one, pairs};
use super::{lua_err, reg, slot_arg, TableExt};
use crate::args::Args;
use crate::engine::{Engine, Shared, State};
use mlua::{MultiValue, Table, Value};

type LuaResult<T> = mlua::Result<T>;

pub const GS_HR_MAX: usize = 32;
pub const GS_HR_FIELDS: usize = 16;
pub(crate) const MOVES: [&str; 10] = ["unknown", "jab", "dash_attack", "tilt", "smash", "aerial", "grab", "throw", "special", "projectile"];

type Rule = [i32; GS_HR_FIELDS];

#[derive(Clone)]
struct Rules {
    count: usize,
    status: i32,
    rules: Vec<Rule>,
}

pub(crate) fn element_name(e: i32) -> &'static str {
    match e {
        0 => "normal",
        1 => "fire",
        2 => "electric",
        5 => "ice",
        13 => "darkness",
        _ => "any",
    }
}
pub(crate) fn element(name: &'static str, v: &Value) -> LuaResult<i32> {
    let n = neg1(one(name, v).check_string(1))?;
    if n == "any" {
        return Ok(-1);
    }
    for e in [0, 1, 2, 5, 13] {
        if n == element_name(e) {
            return Ok(e);
        }
    }
    Err(lua_err("hit rules allow only normal/fire/electric/ice/darkness elements"))
}
pub(crate) fn keys(t: &Table, allowed: &[&str]) -> LuaResult<()> {
    for (k, _) in pairs(t)? {
        let ks = key_str(&k);
        if !ks.as_deref().is_some_and(|k| allowed.contains(&k)) {
            return Err(lua_err(format!(
                "hit rule unknown field '{}' (fraction/split elements unsupported)",
                ks.as_deref().unwrap_or("non-string")
            )));
        }
    }
    Ok(())
}
pub(crate) fn int_field(name: &'static str, t: &Table, key: &str, fallback: i64, lo: i64, hi: i64) -> LuaResult<i32> {
    let v: Value = t.get(key)?;
    let n = if matches!(v, Value::Nil) { fallback } else { neg1(one(name, &v).check_integer(1))? };
    if n < lo || n > hi {
        return Err(lua_err(format!("hit rule {key} out of range")));
    }
    Ok(n as i32)
}
fn table_field(name: &'static str, v: &Value) -> LuaResult<Table> {
    neg1(one(name, v).check_table(1))
}

fn parse_rule(name: &'static str, at: &Table) -> LuaResult<Rule> {
    const TOP: [&str; 3] = ["id", "match", "change"];
    const MATCH: [&str; 5] = ["move", "grounded", "element", "status_bits", "incoming"];
    const CHANGE: [&str; 9] = [
        "element", "damage", "knockback_growth", "knockback_base", "shield_damage", "hitstun", "knockback_taken", "percent_damage",
        "launch",
    ];
    const MULT: [&str; 6] = ["damage", "knockback_growth", "knockback_base", "knockback_taken", "percent_damage", "launch"];
    keys(at, &TOP)?;
    let one_bits = fbits(1.0);
    let mut b: Rule = [0; GS_HR_FIELDS];
    b[0] = int_field(name, at, "id", 0, 0, 0x7fffffff)?;
    b[1] = -1;
    b[2] = -1;
    b[3] = -1;
    b[5] = -1;
    for i in [6, 7, 8, 11, 14, 15] {
        b[i] = one_bits;
    }
    let m: Value = at.get("match")?;
    if !matches!(m, Value::Nil) {
        let m = table_field(name, &m)?;
        keys(&m, &MATCH)?;
        let mv: Value = m.get("move")?;
        if !matches!(mv, Value::Nil) {
            let s = neg1(one(name, &mv).check_string(1))?;
            if s != "any" {
                let Some(j) = MOVES.iter().position(|x| *x == s) else {
                    return Err(lua_err("unknown hit rule move tag"));
                };
                b[1] = j as i32;
            }
        }
        let g: Value = m.get("grounded")?;
        if !matches!(g, Value::Nil) {
            b[2] = check_boolean(name, &g)? as i32;
        }
        let el: Value = m.get("element")?;
        if !matches!(el, Value::Nil) {
            b[3] = element(name, &el)?;
        }
        b[4] = int_field(name, &m, "status_bits", 0, 0, 0x7fffffff)?;
        let inc: Value = m.get("incoming")?;
        if !matches!(inc, Value::Nil) {
            b[13] = check_boolean(name, &inc)? as i32;
        }
    }
    let c: Value = at.get("change")?;
    let c = table_field(name, &c)?;
    keys(&c, &CHANGE)?;
    let el: Value = c.get("element")?;
    if !matches!(el, Value::Nil) {
        b[5] = element(name, &el)?;
    }
    for (j, key) in MULT.iter().enumerate() {
        let (lo, hi) = if j >= 4 { (-1e9f64, 1e9f64) } else { (0.1f64, 4f64) };
        let v: Value = c.get(*key)?;
        let v = if matches!(v, Value::Nil) { 1.0 } else { neg1(one(name, &v).check_number(1))? };
        if !v.is_finite() || v < lo || v > hi {
            return Err(lua_err(format!("hit rule multiplier {key} must be {lo:.6}..{hi:.6}")));
        }
        let idx = if j >= 4 { 10 + j } else if j == 3 { 11 } else { 6 + j };
        b[idx] = fbits(v as f32);
    }
    b[9] = int_field(name, &c, "shield_damage", 0, -100, 100)?;
    b[10] = int_field(name, &c, "hitstun", 0, 0, 120)?;
    if (b[4] != 0 || b[13] != 0) && (b[5] >= 0 || b[7] != one_bits || b[8] != one_bits || b[9] != 0 || b[10] != 0) {
        return Err(lua_err("contact rule permits only damage, knockback_taken, percent_damage and launch changes"));
    }
    if b[4] == 0 && b[13] == 0 && b[11] != one_bits {
        return Err(lua_err("knockback_taken requires status_bits or incoming"));
    }
    Ok(b)
}

fn read(s: &State, port: i32) -> Rules {
    let count = s.game.hit_rules_count(port).clamp(0, GS_HR_MAX as i32) as usize;
    let status = s.game.fighter_status(port);
    let mut rules = vec![[0; GS_HR_FIELDS]; GS_HR_MAX];
    for (i, r) in rules.iter_mut().enumerate().take(count) {
        for (j, f) in r.iter_mut().enumerate() {
            *f = s.game.hit_rule_read(port, i as i32, j as i32);
        }
    }
    Rules { count, status, rules }
}
fn owner_check(s: &State, port: i32) -> LuaResult<()> {
    let owner = s.game.hit_rules_owner(port);
    if owner != 0 && owner != s.cur.map_or(0, |c| c as i32 + 1) {
        return Err(lua_err("hit rules/status owned by another script"));
    }
    Ok(())
}
fn cur(s: &State) -> i32 {
    s.cur.map_or(0, |c| c as i32 + 1)
}
fn apply(s: &mut State, port: i32, owner: i32, r: &Rules) -> bool {
    let mut next = r.clone();
    for i in 0..next.count {
        if next.rules[i][12] == 0 && (i as i32) < s.game.hit_rules_count(port) {
            let same = (0..GS_HR_FIELDS).all(|j| j == 12 || next.rules[i][j] == s.game.hit_rule_read(port, i as i32, j as i32));
            if same {
                next.rules[i][12] = s.game.hit_rule_read(port, i as i32, 12);
            }
        }
    }
    let flat: Vec<i32> = next.rules.iter().flat_map(|r| r.iter().copied()).collect();
    s.game.hit_rules_replace(port, owner, next.count as i32, next.status, &flat) != 0
}

/// Omitted write selector targets both entities; omitted reads show primary. false = primary, true = the partner.
fn selector(a: &Args, i: usize) -> LuaResult<i32> {
    if a.none_or_nil(i) {
        return Ok(-1);
    }
    match a.get(i) {
        Value::Boolean(b) => Ok(*b as i32),
        _ => Err(a.err(i, &format!("boolean expected, got {}", crate::args::typename(Some(a.get(i)))))),
    }
}
fn targets_owner(s: &State, port: usize, sub: i32) -> LuaResult<()> {
    if sub <= 0 {
        owner_check(s, port as i32 * 2)?;
    }
    if sub != 0 {
        owner_check(s, port as i32 * 2 + 1)?;
    }
    Ok(())
}

fn hit_rule_add(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<i64> {
    let port = slot_arg(&a, 1)?;
    let sub = selector(&a, 3)?;
    let mut s = st.borrow_mut();
    s.require_gameplay("hit_rule_add")?;
    targets_owner(&s, port, sub)?;
    let t = a.check_table(2)?;
    let parsed = parse_rule("hit_rule_add", &t)?;
    let first = port as i32 * 2 + (sub > 0) as i32;
    let last = port as i32 * 2 + (sub != 0) as i32;
    let mut draft = Vec::new();
    for entity in first..=last {
        let mut r = read(&s, entity);
        if r.count == GS_HR_MAX {
            return Err(lua_err("hit rules capacity is 32 per fighter entity"));
        }
        r.rules[r.count] = parsed;
        r.count += 1;
        draft.push(r);
    }
    s.rw_branch();
    let mut handle = 0;
    let owner = cur(&s);
    for (k, entity) in (first..=last).enumerate() {
        let r = &mut draft[k];
        if handle != 0 {
            r.rules[r.count - 1][12] = handle;
        }
        if !apply(&mut s, entity, owner, r) {
            return Err(lua_err("hit rule add refused"));
        }
        if handle == 0 {
            handle = s.game.hit_rule_read(entity, r.count as i32 - 1, 12);
        }
    }
    Ok(handle as i64)
}

fn hit_rule_remove(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<bool> {
    let raw = a.check_integer(1)?;
    let mut s = st.borrow_mut();
    s.require_gameplay("hit_rule_remove")?;
    if !(1..=0x7fffffff).contains(&raw) {
        return Err(lua_err("invalid hit rule handle"));
    }
    let handle = raw as i32;
    let mut found = false;
    for p in 0..12 {
        let r = read(&s, p);
        if r.rules[..r.count].iter().any(|x| x[12] == handle) {
            owner_check(&s, p)?;
            found = true;
        }
    }
    if found {
        s.rw_branch();
    }
    let owner = cur(&s);
    for p in 0..12 {
        let mut r = read(&s, p);
        if let Some(i) = r.rules[..r.count].iter().position(|x| x[12] == handle) {
            r.rules.remove(i);
            r.rules.push([0; GS_HR_FIELDS]);
            r.count -= 1;
            apply(&mut s, p, owner, &r);
        }
    }
    Ok(found)
}

fn hit_rules_clear(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<bool> {
    let port = slot_arg(&a, 1)?;
    let sub = selector(&a, 2)?;
    let mut s = st.borrow_mut();
    s.require_gameplay("hit_rules_clear")?;
    targets_owner(&s, port, sub)?;
    s.rw_branch();
    let owner = cur(&s);
    for e in (port as i32 * 2 + (sub > 0) as i32)..=(port as i32 * 2 + (sub != 0) as i32) {
        let mut r = read(&s, e);
        r.count = 0;
        r.rules = vec![[0; GS_HR_FIELDS]; GS_HR_MAX];
        apply(&mut s, e, owner, &r);
    }
    Ok(true)
}

fn fighter_status(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<i64> {
    let port = slot_arg(&a, 1)?;
    let sub = selector(&a, 3)?;
    let mut s = st.borrow_mut();
    if !a.none_or_nil(2) {
        s.require_gameplay("fighter_status")?;
        targets_owner(&s, port, sub)?;
        let n = a.check_integer(2)?;
        if !(0..=0x7fffffff).contains(&n) {
            return Err(lua_err("fighter_status bits must be 0..2147483647"));
        }
        s.rw_branch();
        let owner = cur(&s);
        for e in (port as i32 * 2 + (sub > 0) as i32)..=(port as i32 * 2 + (sub != 0) as i32) {
            let mut r = read(&s, e);
            r.status = n as i32;
            if !apply(&mut s, e, owner, &r) {
                return Err(lua_err("fighter_status refused"));
            }
        }
    }
    Ok(s.game.fighter_status(port as i32 * 2 + (sub > 0) as i32) as i64)
}

fn hit_rules(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Table> {
    const KEYS: [&str; 6] = ["damage", "knockback_growth", "knockback_base", "knockback_taken", "percent_damage", "launch"];
    let port = slot_arg(&a, 1)?;
    let sub = selector(&a, 2)?;
    let s = st.borrow();
    let p = port as i32 * 2 + (sub > 0) as i32;
    let r = read(&s, p);
    let out = lua.create_table()?;
    out.boolean("percent_only", true)?;
    out.boolean("progression", true)?;
    out.int("status_bits", r.status as i64)?;
    out.int("owner", s.game.hit_rules_owner(p) as i64)?;
    out.boolean("sub", sub > 0)?;
    for (i, b) in r.rules[..r.count].iter().enumerate() {
        let t = lua.create_table()?;
        t.int("id", b[0] as i64)?;
        t.int("handle", b[12] as i64)?;
        let m = lua.create_table()?;
        m.string("move", if b[1] < 0 { "any" } else { MOVES[(b[1] as usize).min(9)] })?;
        if b[2] >= 0 {
            m.boolean("grounded", b[2] != 0)?;
        }
        m.string("element", element_name(b[3]))?;
        m.int("status_bits", b[4] as i64)?;
        m.boolean("incoming", b[13] != 0)?;
        t.set("match", m)?;
        let c = lua.create_table()?;
        if b[5] >= 0 {
            c.string("element", element_name(b[5]))?;
        }
        for (j, k) in KEYS.iter().enumerate() {
            let idx = if j >= 4 { 10 + j } else if j == 3 { 11 } else { 6 + j };
            c.num(k, bitsf(b[idx]) as f64)?;
        }
        c.int("shield_damage", b[9] as i64)?;
        c.int("hitstun", b[10] as i64)?;
        t.set("change", c)?;
        out.raw_set(i as i64 + 1, t)?;
    }
    Ok(out)
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    reg(e, gd, "hit_rule_add", hit_rule_add)?;
    reg(e, gd, "hit_rule_remove", hit_rule_remove)?;
    reg(e, gd, "hit_rules_clear", hit_rules_clear)?;
    reg(e, gd, "fighter_status", fighter_status)?;
    reg(e, gd, "hit_rules", hit_rules)?;
    Ok(())
}

#[allow(dead_code)]
fn _unused(_: MultiValue) {}
