//! Fighter reserve/bench (`gw_script_fighter_bench.inc`): fighter_benched, fighter_bench, fighter_call.
use super::arena::{cur_owner, stage_num};
use super::camera::{bitsf, fbits};
use super::{lua_err, reg, slot_arg, TableExt};
use crate::args::{typename, Args};
use crate::engine::{Engine, Shared, State};
use mlua::{MultiValue, Table, Value};

type LuaResult<T> = mlua::Result<T>;

const REASONS: [&str; 12] = [
    "ok",
    "missing fighter or camera box",
    "boss fighter",
    "dead, asleep or respawning",
    "grabbed, grabbing, carried or thrown",
    "hitlag or externally frozen",
    "transformation in progress",
    "unsafe action or held item (requires wait, locomotion, fall or teeter)",
    "owned by another script",
    "fighter is not benched",
    "fighter entity changed",
    "invalid call parameters",
];

/// `gs_reserve_result`
fn result(lua: &mlua::Lua, s: &State, rc: i32, op: &str, slot: usize) -> LuaResult<MultiValue> {
    let mut out = vec![Value::Boolean(rc == 0)];
    if rc != 0 {
        let rc = if !(0..=11).contains(&rc) { 1 } else { rc };
        s.log(&format!("script reserve: {op} refused port={} reason={}", slot + 1, REASONS[rc as usize]));
        out.push(Value::String(lua.create_string(REASONS[rc as usize])?));
    }
    Ok(MultiValue::from_iter(out))
}

fn benched(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<MultiValue> {
    const NAMES: [&str; 14] = [
        "present", "kind", "action", "airborne", "x", "y", "cpu_kind", "frozen", "invisible", "intangible", "camera_excluded",
        "dormant", "benched", "refusal_code",
    ];
    let slot = slot_arg(&a, 1)? as i32;
    let s = st.borrow();
    let first = s.match_active && s.game.fighter_benched(slot) != 0;
    let list = lua.create_table()?;
    for entity in 0..2 {
        let t = lua.create_table()?;
        t.int("entity_index", entity as i64)?;
        let present = s.match_active && s.game.fighter_reserve_read(slot, entity, 0) != 0;
        t.boolean("present", present)?;
        if present {
            for field in 1..14 {
                let v = s.game.fighter_reserve_read(slot, entity, field);
                let n = NAMES[field as usize];
                match field {
                    4 | 5 => t.num(n, bitsf(v) as f64)?,
                    1 | 2 | 6 | 13 => t.int(n, v as i64)?,
                    _ => t.boolean(n, v != 0)?,
                }
            }
        }
        list.raw_set(entity as i64 + 1, t)?;
    }
    Ok(MultiValue::from_iter([Value::Boolean(first), Value::Table(list)]))
}

fn bench(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<MultiValue> {
    let slot = slot_arg(&a, 1)?;
    let mut s = st.borrow_mut();
    s.require_offline("fighter_bench")?;
    if !s.match_active {
        return result(lua, &s, 1, "bench", slot);
    }
    let owner = cur_owner(&s);
    let mut rc = s.game.fighter_bench_check(slot as i32, owner, 0);
    if rc == 0 {
        s.rw_branch();
        rc = s.game.fighter_bench(slot as i32, owner);
    }
    result(lua, &s, rc, "bench", slot)
}

fn opt_field(v: Value, int: bool) -> LuaResult<f64> {
    let a = Args::new("fighter_call", MultiValue::from_iter([v]));
    let r = if int { a.opt_integer(1, 0).map(|n| n as f64) } else { a.opt_number(1, 1.0) };
    r.map_err(|e| match e {
        mlua::Error::RuntimeError(m) => mlua::Error::RuntimeError(m.replacen("bad argument #1", "bad argument #-1", 1)),
        e => e,
    })
}

fn call(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<MultiValue> {
    let slot = slot_arg(&a, 1)?;
    let (x, y) = (stage_num(&a, 2)?, stage_num(&a, 3)?);
    let mut s = st.borrow_mut();
    s.require_offline("fighter_call")?;
    if x.abs() > 49000.0 || y.abs() > 49000.0 {
        return Err(lua_err("reserve coordinates must be within +/-49000"));
    }
    let (mut facing, mut frames) = (1f32, 0i32);
    if !a.none_or_nil(4) {
        let t = match a.get(4) {
            Value::Table(t) => t.clone(),
            o => return Err(a.err(4, &format!("table expected, got {}", typename(Some(o))))),
        };
        facing = opt_field(t.get("facing")?, false)? as f32;
        let n = opt_field(t.get("intangible_frames")?, true)? as i64;
        if (facing != 1.0 && facing != -1.0) || !(0..=600).contains(&n) {
            return Err(lua_err("facing must be -1 or 1; intangible_frames must be 0..600"));
        }
        frames = n as i32;
    }
    if !s.match_active {
        return result(lua, &s, 1, "call", slot);
    }
    let owner = cur_owner(&s);
    let mut rc = s.game.fighter_bench_check(slot as i32, owner, 1);
    if rc == 0 {
        s.rw_branch();
        rc = s.game.fighter_call(slot as i32, owner, fbits(x), fbits(y), fbits(facing), frames);
    }
    result(lua, &s, rc, "call", slot)
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    reg(e, gd, "fighter_benched", benched)?;
    reg(e, gd, "fighter_bench", bench)?;
    reg(e, gd, "fighter_call", call)?;
    Ok(())
}
