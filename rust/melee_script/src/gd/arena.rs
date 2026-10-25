//! arena-hooks and the merged `gd.stage_bounds` (`gw_script_arena.inc`, `gw_script_largemap.inc`, `gw_script_model_api.inc`):
//! stage_set_spawn/stage_spawn, stage_set_origin, stage_set_camera_bounds/blast_bounds, stage_restore_bounds, stage_bounds,
//! stage_collision_groups/stage_collision_group. State and ownership live in the game half (snapshotted).
use super::camera::{bitsf, fbits};
use super::{lua_err, reg, TableExt};
use crate::args::{fmt_g, Args};
use crate::engine::{Engine, Shared, State};
use mlua::{Table, Value};

type LuaResult<T> = mlua::Result<T>;

/// `gs_require_stage`
pub(crate) fn require_stage(s: &mut State, fname: &str) -> LuaResult<()> {
    s.require_offline(fname)?;
    let ok = s.cur.is_some_and(|c| c != s.console && s.scripts.get(c).is_some_and(|x| x.gameplay));
    if !ok {
        return Err(lua_err(format!("gd.{fname} requires a gameplay mod script")));
    }
    if !s.match_active {
        return Err(lua_err(format!("gd.{fname} requires an active match")));
    }
    Ok(())
}

/// `gs_stage_num`
pub(crate) fn stage_num(a: &Args, i: usize) -> LuaResult<f32> {
    let v = a.check_number(i)?;
    if !v.is_finite() || !(-100000.0..=100000.0).contains(&v) {
        return Err(lua_err("stage coordinates must be finite and within +/-100000"));
    }
    Ok(v as f32)
}

pub(crate) fn cur_owner(s: &State) -> i32 {
    s.cur.map_or(0, |c| c as i32 + 1)
}

/// `gs_arena_require`
fn arena_require(s: &mut State, fname: &str) -> LuaResult<()> {
    require_stage(s, fname)?;
    let owner = s.game.arena_owner();
    if owner != 0 && owner != cur_owner(s) {
        return Err(lua_err(format!("gd.{fname}: bounds are owned by another script")));
    }
    Ok(())
}

fn spawn_slot_ok(slot: i64) -> bool {
    (0..=7).contains(&slot) || (127..=146).contains(&slot)
}
const SPAWN_ERR: &str = "spawn slot must be 0-7 (starts/respawns) or 127-146 (item spawns)";

/// `l_stage_bounds_arena`: nil outside a match
fn bounds_arena(lua: &mlua::Lua, s: &State) -> LuaResult<Option<Table>> {
    if !s.match_active {
        return Ok(None);
    }
    let keys = ["left", "right", "top", "bottom"];
    let t = lua.create_table()?;
    for (b, name) in ["camera", "blast"].iter().enumerate() {
        let r = lua.create_table()?;
        for (k, key) in keys.iter().enumerate() {
            r.num(key, bitsf(s.game.arena_read((b * 4 + k) as i32)) as f64)?;
        }
        t.set(*name, r)?;
    }
    let o = lua.create_table()?;
    o.num("x", bitsf(s.game.arena_read(8)) as f64)?;
    o.num("y", bitsf(s.game.arena_read(9)) as f64)?;
    t.set("origin", o)?;
    t.num("frames", s.game.arena_read(10) as f64)?;
    Ok(Some(t))
}

/// `l_stage_bounds_base` (only main_floor and surface_top survive the merge)
fn bounds_base(lua: &mlua::Lua, s: &State) -> LuaResult<Option<Table>> {
    if !s.match_active {
        return Ok(None);
    }
    let fields = ["left", "right", "top", "bottom"];
    let t = lua.create_table()?;
    let has_floor = s.game.stage_bounds_i(12) != 0;
    if has_floor {
        let r = lua.create_table()?;
        for (f, key) in fields.iter().enumerate() {
            r.num(key, bitsf(s.game.stage_bounds_i((2 * 4 + f) as i32)) as f64)?;
        }
        t.set("main_floor", r)?;
        t.num("surface_top", bitsf(s.game.stage_bounds_i(13)) as f64)?;
    }
    Ok(Some(t))
}

/// `l_stage_bounds_largemap`: optional set (`false` restores), then the camera/blast rectangles
fn bounds_largemap(lua: &mlua::Lua, s: &mut State, a: &Args) -> LuaResult<Table> {
    let rects = ["camera", "blast"];
    let fields = ["left", "right", "bottom", "top"];
    require_stage(s, "stage_bounds")?;
    if matches!(a.get(1), Value::Boolean(false)) {
        s.rw_branch();
        s.game.map_bounds(-1, 0, 0, 0, 0);
    } else if !a.none_or_nil(1) {
        let arg = a.check_table(1)?;
        let mut value = [[0f32; 4]; 2];
        let mut present = [false; 2];
        for i in 0..2 {
            let v: Value = arg.raw_get(rects[i])?;
            if matches!(v, Value::Nil) {
                continue;
            }
            let Value::Table(rt) = v else {
                return Err(lua_err(format!("bad argument #1 to 'stage_bounds' (table expected, got {})", crate::args::typename(Some(&v)))));
            };
            present[i] = true;
            for (j, f) in fields.iter().enumerate() {
                let n: Value = rt.raw_get(*f)?;
                let n = match n {
                    Value::Integer(n) => n as f64,
                    Value::Number(n) => n,
                    other => {
                        return Err(lua_err(format!("number expected, got {}", crate::args::typename(Some(&other)))))
                    }
                };
                if !n.is_finite() || !(-100000.0..=100000.0).contains(&n) {
                    return Err(lua_err("stage coordinates must be finite and within +/-100000"));
                }
                value[i][j] = n as f32;
            }
            if value[i][0] >= value[i][1] || value[i][2] >= value[i][3] {
                return Err(lua_err("stage bounds require left < right and bottom < top"));
            }
        }
        s.rw_branch();
        for i in 0..2 {
            if present[i] {
                let v = value[i];
                s.game.map_bounds(i as i32, fbits(v[0]), fbits(v[1]), fbits(v[2]), fbits(v[3]));
            }
        }
    }
    let t = lua.create_table()?;
    for (i, name) in rects.iter().enumerate() {
        let r = lua.create_table()?;
        for (j, f) in fields.iter().enumerate() {
            r.num(f, bitsf(s.game.map_bound(i as i32, j as i32)) as f64)?;
        }
        t.set(*name, r)?;
    }
    Ok(t)
}

fn arena_bounds(a: &Args, st: &Shared, blast: bool) -> LuaResult<bool> {
    let (l, r, t, b) = (stage_num(a, 1)?, stage_num(a, 2)?, stage_num(a, 3)?, stage_num(a, 4)?);
    let mut s = st.borrow_mut();
    arena_require(&mut s, if blast { "stage_set_blast_bounds" } else { "stage_set_camera_bounds" })?;
    if l >= r || b >= t {
        return Err(lua_err("bounds require left < right and bottom < top"));
    }
    if blast {
        s.rw_branch();
    }
    let owner = cur_owner(&s);
    s.game.arena_bounds(owner, blast, fbits(l), fbits(r), fbits(t), fbits(b));
    if blast {
        let g = |v: f32| fmt_g(v as f64, 6);
        s.log(&format!("script arena-hooks: blast bounds {} {} {} {}", g(l), g(r), g(t), g(b)));
    }
    Ok(true)
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    reg(e, gd, "stage_set_spawn", |_, st, a| {
        let slot = a.check_integer(1)?;
        let (x, y) = (stage_num(&a, 2)?, stage_num(&a, 3)?);
        let mut s = st.borrow_mut();
        arena_require(&mut s, "stage_set_spawn")?;
        if !spawn_slot_ok(slot) {
            return Err(lua_err(SPAWN_ERR));
        }
        s.rw_branch();
        let owner = cur_owner(&s);
        let applied = s.game.arena_spawn(owner, slot as i32, fbits(x), fbits(y)) != 0;
        s.log(&format!("script arena-hooks: spawn {} -> {} {}", slot as i32, fmt_g(x as f64, 6), fmt_g(y as f64, 6)));
        Ok(applied)
    })?;
    reg(e, gd, "stage_spawn", |_, st, a| {
        let slot = a.check_integer(1)?;
        if !spawn_slot_ok(slot) {
            return Err(lua_err(SPAWN_ERR));
        }
        let s = st.borrow();
        let p = |axis| s.game.arena_spawn_pos(slot as i32, axis, 0.0) as f64;
        Ok((p(0), p(1), p(2)))
    })?;
    reg(e, gd, "stage_set_origin", |_, st, a| {
        let (x, y) = (stage_num(&a, 1)?, stage_num(&a, 2)?);
        let mut s = st.borrow_mut();
        arena_require(&mut s, "stage_set_origin")?;
        let mut frames = 0i64;
        if !a.none_or_nil(3) {
            let t = a.check_table(3)?;
            let f: Value = t.get("frames")?;
            frames = match f {
                Value::Nil => 0,
                Value::Integer(n) => n,
                Value::Number(n) if n.floor() == n => n as i64,
                other => {
                    return Err(lua_err(format!("number expected, got {}", crate::args::typename(Some(&other)))))
                }
            };
        }
        if !(0..=i32::MAX as i64).contains(&frames) {
            return Err(lua_err("frames must be in 0..INT_MAX"));
        }
        let owner = cur_owner(&s);
        s.game.arena_origin(owner, fbits(x), fbits(y), frames as i32);
        Ok(true)
    })?;
    reg(e, gd, "stage_set_camera_bounds", |_, st, a| arena_bounds(&a, st, false))?;
    reg(e, gd, "stage_set_blast_bounds", |_, st, a| arena_bounds(&a, st, true))?;
    reg(e, gd, "stage_restore_bounds", |_, st, _| {
        let mut s = st.borrow_mut();
        arena_require(&mut s, "stage_restore_bounds")?;
        s.rw_branch();
        s.game.arena_restore();
        s.log("script arena-hooks: bounds restored");
        Ok(true)
    })?;
    reg(e, gd, "stage_collision_groups", |lua, st, _| {
        let s = st.borrow();
        let count = if s.match_active { s.game.arena_group(-1, 0) } else { 0 };
        let t = lua.create_table()?;
        for id in 0..count {
            let r = lua.create_table()?;
            r.num("id", id as f64)?;
            r.boolean("enabled", s.game.arena_group(id, 0) != 0)?;
            t.raw_set(id as i64 + 1, r)?;
        }
        Ok(t)
    })?;
    reg(e, gd, "stage_collision_group", |_, st, a| {
        let id = a.check_integer(1)?;
        let mut s = st.borrow_mut();
        require_stage(&mut s, "stage_collision_group")?;
        if !matches!(a.get(2), Value::Boolean(_)) {
            return Err(a.err(2, &format!("boolean expected, got {}", crate::args::typename(Some(a.get(2))))));
        }
        if id < 0 || id >= s.game.arena_group(-1, 0) as i64 {
            return Err(lua_err("collision group id is out of range"));
        }
        let owner = s.game.arena_group(id as i32, 1);
        if owner != 0 && owner != cur_owner(&s) {
            return Err(lua_err("collision group is owned by another script"));
        }
        let enabled = a.truthy(2);
        s.rw_branch();
        let me = cur_owner(&s);
        s.game.arena_group_set(me, id as i32, enabled);
        s.log(&format!("script arena-hooks: collision group {} enabled={}", id as i32, enabled as i32));
        Ok(true)
    })?;
    // the merge of the largemap, arena-hooks and model-gaps readers
    reg(e, gd, "stage_bounds", |lua, st, a| {
        let mut s = st.borrow_mut();
        let main = if a.is_none(1) { bounds_arena(lua, &s)? } else { Some(bounds_largemap(lua, &mut s, &a)?) };
        let Some(main) = main else { return Ok(Value::Nil) };
        if let Some(t) = bounds_arena(lua, &s)? {
            for k in ["origin", "frames"] {
                let v: Value = t.get(k)?;
                if !matches!(v, Value::Nil) {
                    main.set(k, v)?;
                }
            }
        }
        if let Some(t) = bounds_base(lua, &s)? {
            for k in ["main_floor", "surface_top"] {
                let v: Value = t.get(k)?;
                if !matches!(v, Value::Nil) {
                    main.set(k, v)?;
                }
            }
        }
        Ok(Value::Table(main))
    })?;
    Ok(())
}
