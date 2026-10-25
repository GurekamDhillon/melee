//! Entity and rule-source references (`gw_script_spine_refs.inc`): entity_ref, entity_resolve, entity_valid, source_ref,
//! source_resolve, entity_call. The identity codec is `e1:epoch:kind:id:generation:definition`; the game half answers
//! `ScriptGame::entity_field` (0 live, 1 epoch, 2 generation, 3 definition, 4 owner entity + 1, 5 role bits).
use super::{lua_err, reg, TableExt};
use crate::args::{typename, Args};
use crate::engine::{Engine, Shared};
use crate::game::ScriptGame;
use mlua::{MultiValue, Table, Value};

type LuaResult<T> = mlua::Result<T>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EntityRef {
    pub epoch: u32,
    pub kind: u32,
    pub id: u32,
    pub generation: u32,
    pub definition: u32,
}

const SOURCE_KINDS: [&str; 6] = ["", "drive", "opponent", "fighter", "item", "stage"];

/// `gs_ref_uint`: decimal, no leading zero, ends at `delimiter` (0 = end of text)
fn ref_uint(s: &[u8], pos: &mut usize, delim: u8) -> Option<u32> {
    let mut p = *pos;
    if s.get(p) == Some(&b'0') && s.get(p + 1).is_some_and(|c| c.is_ascii_digit()) {
        return None;
    }
    let (mut n, mut digits) = (0u32, 0);
    while let Some(c) = s.get(p).filter(|c| c.is_ascii_digit()) {
        let d = (c - b'0') as u32;
        if n > (u32::MAX - d) / 10 {
            return None;
        }
        n = n * 10 + d;
        digits += 1;
        p += 1;
    }
    let at = s.get(p).copied().unwrap_or(0);
    if digits == 0 || at != delim {
        return None;
    }
    *pos = p + (delim != 0) as usize;
    Some(n)
}

/// `gs_ref_parse`
pub fn ref_parse(text: &[u8]) -> Option<EntityRef> {
    if text.len() < 3 || text.contains(&0) || &text[..3] != b"e1:" {
        return None;
    }
    let mut p = 3;
    let epoch = ref_uint(text, &mut p, b':')?;
    let kind = ref_uint(text, &mut p, b':')?;
    let id = ref_uint(text, &mut p, b':')?;
    let generation = ref_uint(text, &mut p, b':')?;
    let definition = ref_uint(text, &mut p, 0)?;
    if epoch == 0 || !(1..=5).contains(&kind) {
        return None;
    }
    Some(EntityRef { epoch, kind, id, generation, definition })
}
/// `gs_ref_current`
pub fn ref_current(g: &dyn ScriptGame, kind: u32, id: u32) -> Option<EntityRef> {
    if g.entity_field(kind as i32, id as i32, 0) == 0 {
        return None;
    }
    let epoch = g.entity_field(kind as i32, id as i32, 1) as u32;
    let r = EntityRef {
        kind,
        id,
        epoch,
        generation: g.entity_field(kind as i32, id as i32, 2) as u32,
        definition: g.entity_field(kind as i32, id as i32, 3) as u32,
    };
    (epoch != 0).then_some(r)
}
/// `gs_ref_text`
pub fn ref_text(r: &EntityRef) -> String {
    format!("e1:{}:{}:{}:{}:{}", r.epoch, r.kind, r.id, r.generation, r.definition)
}
/// `gs_ref_live`
pub fn ref_live(g: &dyn ScriptGame, r: &EntityRef) -> bool {
    ref_current(g, r.kind, r.id).is_some_and(|n| n.epoch == r.epoch && n.generation == r.generation && n.definition == r.definition)
}
/// `gs_spine_actor`: the current reference text of the fighter entity port*2+sub (empty when it is not live)
pub fn actor_text(g: &dyn ScriptGame, port: i32, sub: bool) -> String {
    if (0..6).contains(&port) {
        if let Some(r) = ref_current(g, 1, (port * 2 + sub as i32) as u32) {
            return ref_text(&r);
        }
    }
    String::new()
}

/// `gs_ref_arg`
fn ref_arg(g: &dyn ScriptGame, a: &Args, i: usize, required: bool) -> LuaResult<Option<EntityRef>> {
    let Value::String(s) = a.get(i) else {
        return if required { Err(a.err(i, "versioned entity reference required")) } else { Ok(None) };
    };
    let bytes = s.as_bytes();
    match ref_parse(&bytes).filter(|r| ref_live(g, r)) {
        Some(r) => Ok(Some(r)),
        None if required => Err(a.err(i, "malformed or stale entity reference")),
        None => Ok(None),
    }
}
fn ref_integer(a: &Args, i: usize, max: u64) -> LuaResult<u32> {
    let n = a.check_integer(i)?;
    if n < 0 || n as u64 > max {
        return Err(a.err(i, "reference integer out of range"));
    }
    Ok(n as u32)
}
fn push_current(g: &dyn ScriptGame, t: &Table, kind: u32, id: u32, field: &str) -> LuaResult<()> {
    match ref_current(g, kind, id) {
        Some(r) => t.set(field, ref_text(&r)),
        None => t.set(field, Value::Nil),
    }
}
fn ref_push(lua: &mlua::Lua, g: &dyn ScriptGame, r: &EntityRef) -> LuaResult<Table> {
    const NAMES: [&str; 6] = ["", "fighter", "item", "echo", "stage_object", "stage_actor"];
    let owner = g.entity_field(r.kind as i32, r.id as i32, 4);
    let t = lua.create_table()?;
    t.int("version", 1)?;
    t.string("kind", NAMES[r.kind as usize])?;
    t.int("epoch", r.epoch as i64)?;
    t.int("id", r.id as i64)?;
    t.int("generation", r.generation as i64)?;
    t.int("definition", r.definition as i64)?;
    t.int("roles", g.entity_field(r.kind as i32, r.id as i32, 5) as i64)?;
    if owner != 0 {
        t.int("owner_port", ((owner - 1) / 2 + 1) as i64)?;
        t.boolean("owner_sub", (owner - 1) % 2 != 0)?;
        push_current(g, &t, 1, (owner - 1) as u32, "owner_ref")?;
    }
    if r.kind == 1 {
        t.int("port", (r.id / 2 + 1) as i64)?;
        t.boolean("sub", r.id % 2 != 0)?;
        t.int("collision_entity", r.id as i64)?;
        t.int("capability_entity", (r.id / 2 + 1 + (r.id % 2) * 6) as i64)?;
    }
    Ok(t)
}

fn entity_ref(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Value> {
    let s = st.borrow();
    let g = &*s.game;
    if let Value::String(_) = a.get(1) {
        ref_arg(g, &a, 1, true)?;
        return Ok(a.get(1).clone());
    }
    let (kind, id);
    if a.is_number(1) && !matches!(a.get(1), Value::String(_)) {
        let port = ref_integer(&a, 1, 6)?;
        if port == 0 {
            return Err(a.err(1, "port1..6"));
        }
        kind = 1;
        id = (port - 1) * 2;
    } else {
        let t = a.check_table(1)?;
        let k: Value = t.get("kind")?;
        let name = if matches!(k, Value::Nil) {
            "fighter".to_string()
        } else {
            super::caps::neg1(super::caps::one("entity_ref", &k).check_string(1))?
        };
        kind = match name.as_str() {
            "fighter" => 1,
            "item" | "article" | "projectile" => 2,
            "echo" => 3,
            "stage_object" => 4,
            "stage_actor" => 5,
            _ => return Err(lua_err("unknown entity class")),
        };
        if kind == 1 {
            let pv: Value = t.get("port")?;
            let port = super::caps::neg1(super::caps::one("entity_ref", &pv).check_integer(1))?;
            if !(0..=6).contains(&port) {
                return Err(lua_err("bad argument #-1 to 'entity_ref' (reference integer out of range)"));
            }
            if port == 0 {
                return Err(lua_err("port1..6"));
            }
            let sv: Value = t.get("sub")?;
            let sub = if matches!(sv, Value::Nil) { false } else { super::caps::check_boolean("entity_ref", &sv)? };
            id = (port as u32 - 1) * 2 + sub as u32;
        } else {
            let iv: Value = t.get("id")?;
            let n = super::caps::neg1(super::caps::one("entity_ref", &iv).check_integer(1))?;
            if n < 0 || n as u64 > u32::MAX as u64 {
                return Err(lua_err("bad argument #-1 to 'entity_ref' (reference integer out of range)"));
            }
            id = n as u32;
        }
    }
    Ok(match ref_current(g, kind, id) {
        Some(r) => Value::String(lua.create_string(ref_text(&r))?),
        None => Value::Nil,
    })
}
fn entity_resolve(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Value> {
    let s = st.borrow();
    match ref_arg(&*s.game, &a, 1, false)? {
        Some(r) => Ok(Value::Table(ref_push(lua, &*s.game, &r)?)),
        None => Ok(Value::Nil),
    }
}
fn entity_valid(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<bool> {
    let s = st.borrow();
    Ok(ref_arg(&*s.game, &a, 1, false)?.is_some())
}

fn source_key_ok(s: &[u8]) -> bool {
    !s.is_empty() && s.len() <= 96 && s.iter().all(|c| c.is_ascii_alphanumeric() || *c == b'_' || *c == b'-')
}
fn source_ref(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Value> {
    let kind = a.check_bytes(1)?;
    let key = a.check_bytes(2)?;
    if kind.contains(&0) {
        return Err(lua_err("malformed rule source class"));
    }
    let Some(k) = (1..=5).find(|k| SOURCE_KINDS[*k].as_bytes() == kind.as_slice()) else {
        return Err(lua_err("unknown rule source class"));
    };
    if !source_key_ok(&key) {
        return Err(lua_err("source key must be1..96 letters/digits/underscore/hyphen"));
    }
    let revision = if a.none_or_nil(3) { 1 } else { ref_integer(&a, 3, i32::MAX as u64)? };
    if revision == 0 {
        return Err(lua_err("source revision must be positive"));
    }
    let mut binding = "-".to_string();
    if !a.none_or_nil(4) {
        let s = st.borrow();
        ref_arg(&*s.game, &a, 4, true)?;
        binding = a.check_string(4)?;
    }
    Ok(Value::String(lua.create_string(format!("s1:{k}:{revision}/{binding}/{}", String::from_utf8_lossy(&key)))?))
}
fn source_resolve(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Value> {
    let s = a.check_bytes(1)?;
    if s.contains(&0) || !s.starts_with(b"s1:") {
        return Ok(Value::Nil);
    }
    let mut p = 3;
    let Some(kind) = ref_uint(&s, &mut p, b':').filter(|k| (1..=5).contains(k)) else { return Ok(Value::Nil) };
    let Some(revision) = ref_uint(&s, &mut p, b'/').filter(|r| *r != 0 && *r <= i32::MAX as u32) else { return Ok(Value::Nil) };
    let rest = &s[p..];
    let Some(slash) = rest.iter().position(|c| *c == b'/') else { return Ok(Value::Nil) };
    if slash >= 80 || !source_key_ok(&rest[slash + 1..]) {
        return Ok(Value::Nil);
    }
    let binding = &rest[..slash];
    if binding != b"-" {
        let live = ref_parse(binding).is_some_and(|r| ref_live(&*st.borrow().game, &r));
        if !live {
            return Ok(Value::Nil);
        }
    }
    let t = lua.create_table()?;
    t.int("version", 1)?;
    t.string("kind", SOURCE_KINDS[kind as usize])?;
    t.int("revision", revision as i64)?;
    t.string("key", &String::from_utf8_lossy(&rest[slash + 1..]))?;
    if binding != b"-" {
        t.string("entity_ref", &String::from_utf8_lossy(binding))?;
    }
    Ok(Value::Table(t))
}

/// `gs_ref_selector`: an explicit selector must agree with the reference; the slot is then filled
fn selector(args: &mut Vec<Value>, at: usize, sub: bool, numeric: bool) -> LuaResult<()> {
    let cur = args.get(at - 1).cloned().unwrap_or(Value::Nil);
    if !matches!(cur, Value::Nil) {
        let supplied = if numeric {
            let n = match &cur {
                Value::Integer(n) => *n,
                Value::Number(f) if f.fract() == 0.0 => *f as i64,
                o => return Err(lua_err(format!("bad argument #{at} (number expected, got {})", typename(Some(o))))),
            };
            if !(0..=1).contains(&n) {
                return Err(lua_err("sub selector0..1"));
            }
            n != 0
        } else {
            match cur {
                Value::Boolean(b) => b,
                o => return Err(lua_err(format!("bad argument #{at} (boolean expected, got {})", typename(Some(&o))))),
            }
        };
        if supplied != sub {
            return Err(lua_err("selector conflicts with entity reference"));
        }
    }
    while args.len() < at {
        args.push(Value::Nil);
    }
    args[at - 1] = if numeric { Value::Integer(sub as i64) } else { Value::Boolean(sub) };
    Ok(())
}

fn entity_call(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<MultiValue> {
    let r = ref_arg(&*st.borrow().game, &a, 1, true)?.unwrap();
    if r.kind != 1 {
        return Err(lua_err("this compatibility operation requires a fighter reference"));
    }
    let op = a.check_bytes(2)?;
    if op.contains(&0) {
        return Err(lua_err("malformed compatibility operation"));
    }
    let op = String::from_utf8_lossy(&op).to_string();
    const TARGETS: [&str; 16] = [
        "fighter_caps", "fighter_effect", "fighter_armour", "fighter_armor", "fighter_timed_status", "give_item", "nearest_opponent",
        "opponents_in_radius", "fighter_status", "hit_rule_add", "hit_rules", "hit_rules_clear", "fighter_history", "echo_add",
        "echoes", "zones_at",
    ];
    let target = if TARGETS.contains(&op.as_str()) {
        let base = st.borrow().base.clone();
        let gd: Option<Table> = match base {
            Some(b) => b.get("gd").ok(),
            None => None,
        };
        let _ = lua;
        match gd.map_or(Ok(Value::Nil), |g| g.get::<Value>(op.as_str()))? {
            Value::Function(f) => Some(f),
            _ => None,
        }
    } else {
        None
    };
    let Some(f) = target else {
        return Err(lua_err("unsupported entity compatibility operation"));
    };
    let (sub, port) = (r.id % 2 != 0, (r.id / 2 + 1) as i64);
    let mut args: Vec<Value> = a.get_all().into_iter().skip(2).collect(); // operation name removed
    args.insert(0, Value::Integer(port));
    args.remove(1); // the reference slot is replaced by the port number
    match op.as_str() {
        "fighter_caps" | "fighter_effect" | "fighter_armour" | "fighter_armor" | "fighter_timed_status" | "give_item"
        | "nearest_opponent" | "opponents_in_radius" => {
            args[0] = Value::Integer(port + sub as i64 * 6);
        }
        "echo_add" => {
            let Some(Value::Table(src)) = args.get(1).cloned() else {
                return Err(lua_err(format!(
                    "bad argument #2 (table expected, got {})",
                    typename(args.get(1))
                )));
            };
            let sv: Value = src.get("sub")?;
            if !matches!(sv, Value::Nil) {
                match sv {
                    Value::Boolean(b) if b == sub => {}
                    Value::Boolean(_) => return Err(lua_err("selector conflicts with entity reference")),
                    o => return Err(lua_err(format!("bad argument #-1 (boolean expected, got {})", typename(Some(&o))))),
                }
            }
            let dest = lua.create_table()?;
            for pair in src.pairs::<Value, Value>() {
                let (k, v) = pair?;
                dest.raw_set(k, v)?;
            }
            dest.set("sub", sub)?;
            args[1] = Value::Table(dest);
        }
        "zones_at" => selector(&mut args, 2, sub, true)?,
        "hit_rules" | "hit_rules_clear" | "echoes" => selector(&mut args, 2, sub, false)?,
        _ => selector(&mut args, 3, sub, false)?,
    }
    f.call::<MultiValue>(MultiValue::from_iter(args))
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    reg(e, gd, "entity_ref", entity_ref)?;
    reg(e, gd, "entity_resolve", entity_resolve)?;
    reg(e, gd, "entity_valid", entity_valid)?;
    reg(e, gd, "source_ref", source_ref)?;
    reg(e, gd, "source_resolve", source_resolve)?;
    reg(e, gd, "entity_call", entity_call)?;
    Ok(())
}
