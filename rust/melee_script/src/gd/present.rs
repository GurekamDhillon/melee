//! Presentation controls: part overrides and measurements (parts, parts_id, parts_id_off, parts_clear, dobj_solid, dobj_solid_off,
//! dobj_tint), the effects lab (fx_play, fx_attach, fx_world, fx_move, fx_control, fx_end, fx_instance, fx_stop, fx_shake, fx_flash)
//! and lab_peek (gw_script.c). The part table and effect instances are game/host services (`ScriptGame::parts_*`, `Host::fx_*`).
use super::core::color_arg;
use super::{lua_err, reg, slot_arg, TableExt};
use crate::args::Args;
use crate::engine::{Engine, Shared, State};
use crate::game::{players_present, SF_FACING};
use mlua::{Table, Value};

type LuaResult<T> = mlua::Result<T>;

// script_parts.h
pub const PART_I_REFRESH: i32 = 0;
pub const PART_I_COUNT: i32 = 1;
pub const PART_I_OWNER_JOINT: i32 = 2;
pub const PART_I_HASH_LO: i32 = 12;
pub const PART_I_HASH_HI: i32 = 13;
pub const PART_I_MODELS: i32 = 15;
pub const PART_I_BODY_JOINT: i32 = 18;
pub const PART_F_AREA: i32 = 0;
pub const PART_F_MIN_X: i32 = 1;
pub const PART_F_REGION: i32 = 16;
pub const PART_REGIONS: i32 = 12;
pub const PART_CTL_COLOR: i32 = 1;
pub const PART_CTL_CLEAR: i32 = 2;
pub const PART_CTL_IDS: i32 = 3;
pub const PART_CTL_IDS_OFF: i32 = 4;
pub const PART_CTL_TINT: i32 = 5;

/// `gs_present_arg`: the slot, or -1 when no fighter is there
fn present_arg(st: &Shared, a: &Args, i: usize) -> LuaResult<i32> {
    let slot = slot_arg(a, i)?;
    Ok(if players_present(&*st.borrow().game, slot) { slot as i32 } else { -1 })
}
fn owner(s: &State) -> i32 {
    s.cur.map_or(0, |c| c as i32 + 1)
}

fn dobj_solid(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<bool> {
    let slot = present_arg(st, &a, 1)?;
    let d = a.check_integer(2)? as i32;
    if slot < 0 {
        return Ok(false);
    }
    let rgba = color_arg(&a, 3, 0xFFFFFFFF)?;
    let mut s = st.borrow_mut();
    s.require_offline("dobj_solid")?;
    s.rw_branch();
    let ow = owner(&s);
    let r = s.game.parts_control(slot, d, PART_CTL_COLOR, (rgba >> 8) as i32, ow);
    if r < 0 {
        return Err(lua_err(format!("gd.dobj_solid: invalid part or override capacity exhausted ({r})")));
    }
    Ok(r != 0)
}
fn dobj_solid_off(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<bool> {
    let slot = present_arg(st, &a, 1)?;
    let d = a.check_integer(2)? as i32;
    if slot < 0 {
        return Ok(false);
    }
    let mut s = st.borrow_mut();
    s.require_offline("dobj_solid_off")?;
    s.rw_branch();
    let ow = owner(&s);
    let r = s.game.parts_control(slot, d, PART_CTL_CLEAR, 0, ow);
    if r < 0 {
        return Err(lua_err("gd.dobj_solid_off: invalid part"));
    }
    Ok(r != 0)
}
fn dobj_tint(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<bool> {
    let slot = present_arg(st, &a, 1)?;
    let d = a.check_integer(2)? as i32;
    if slot < 0 {
        return Ok(false);
    }
    let rgba = color_arg(&a, 3, 0xFFFFFFFF)?;
    let mut s = st.borrow_mut();
    s.require_offline("dobj_tint")?;
    s.rw_branch();
    let ow = owner(&s);
    let r = s.game.parts_control(slot, d, PART_CTL_TINT, (rgba >> 8) as i32, ow);
    if r < 0 {
        return Err(lua_err(format!("dobj_tint: invalid part or owned by another script ({r})")));
    }
    Ok(r != 0)
}

fn parts(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Value> {
    const REGIONS: [&str; 12] = [
        "unclassified", "head", "torso", "left_arm", "right_arm", "left_hand", "right_hand", "left_leg", "right_leg", "left_foot",
        "right_foot", "equipment",
    ];
    const FIELDS: [&str; 17] = [
        "joint", "material", "material_data", "triangles", "status", "hidden", "source", "item_kind", "item_ordinal", "path",
        "hash_lo", "hash_hi", "joint_flags", "models", "visibility_model", "visibility_state", "body_joint",
    ];
    let slot = present_arg(st, &a, 1)?;
    st.borrow_mut().require_offline("parts")?;
    if slot < 0 {
        return Ok(Value::Nil);
    }
    let s = st.borrow();
    let g = &*s.game;
    let n = if a.truthy(2) || g.parts_i(slot, 0, PART_I_COUNT) < 0 { g.parts_i(slot, 0, PART_I_REFRESH) } else { g.parts_i(slot, 0, PART_I_COUNT) };
    if !(0..=512).contains(&n) {
        return Err(lua_err(format!("gd.parts: invalid draw-list mapping ({n})")));
    }
    let t = lua.create_table()?;
    t.int("schema", 1)?;
    t.string(
        "geometry_signature",
        &format!("{:08x}{:08x}", g.parts_i(slot, 0, PART_I_HASH_HI) as u32, g.parts_i(slot, 0, PART_I_HASH_LO) as u32),
    )?;
    t.string("measurement_space", "current_pose_world")?;
    t.string("coverage_mode", "opaque_geometry_paired_ids")?;
    for i in 0..n {
        let p = lua.create_table()?;
        p.int("index", i as i64)?;
        for k in PART_I_OWNER_JOINT..=PART_I_BODY_JOINT {
            if k == PART_I_HASH_LO || k == PART_I_HASH_HI || k == PART_I_MODELS {
                continue;
            }
            p.int(FIELDS[(k - PART_I_OWNER_JOINT) as usize], g.parts_i(slot, i, k) as i64)?;
        }
        p.num("area", g.parts_f(slot, i, PART_F_AREA) as f64)?;
        let b = lua.create_table()?;
        for k in 0..6 {
            b.raw_set(k as i64 + 1, g.parts_f(slot, i, PART_F_MIN_X + k) as f64)?;
        }
        p.set("bounds", b)?;
        let r = lua.create_table()?;
        for k in 0..PART_REGIONS {
            r.num(REGIONS[k as usize], g.parts_f(slot, i, PART_F_REGION + k) as f64)?;
        }
        p.set("regions", r)?;
        p.string("recolor_capability", "independent_draw_override")?;
        t.raw_set(i as i64 + 1, p)?;
    }
    Ok(Value::Table(t))
}
fn parts_id(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<bool> {
    let slot = present_arg(st, &a, 1)?;
    let mut s = st.borrow_mut();
    s.require_offline("parts_id")?;
    s.rw_branch();
    let ow = owner(&s);
    let r = s.game.parts_control(slot, 0, PART_CTL_IDS, a.truthy(2) as i32, ow);
    if r < 0 {
        return Err(lua_err(format!("gd.parts_id: unavailable snapshot or another owner ({r})")));
    }
    Ok(true)
}
fn parts_id_off(_: &mlua::Lua, st: &Shared, _a: Args) -> LuaResult<()> {
    let mut s = st.borrow_mut();
    s.require_offline("parts_id_off")?;
    let ow = owner(&s);
    if s.game.parts_control(0, 0, PART_CTL_IDS_OFF, 0, ow) < 0 {
        return Err(lua_err("gd.parts_id_off: owned by another script"));
    }
    Ok(())
}
fn parts_clear(_: &mlua::Lua, st: &Shared, _a: Args) -> LuaResult<()> {
    let mut s = st.borrow_mut();
    s.require_offline("parts_clear")?;
    let ow = owner(&s);
    s.game.parts_reset(ow);
    Ok(())
}

fn finite_pos(name: &str, v: [f32; 4]) -> LuaResult<()> {
    let _ = name;
    if !v.iter().all(|x| x.is_finite()) || v[3] <= 0.0 || v[3] > 100.0 {
        return Err(lua_err(format!("{name} requires finite offsets and scale in (0,100]")));
    }
    Ok(())
}
fn fx_play(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<i64> {
    let package = a.check_string(1)?;
    let slot = present_arg(st, &a, 2)?;
    let joint = a.opt_integer(3, 0)? as i32;
    let (x, y, z) = (a.opt_number(4, 0.0)? as f32, a.opt_number(5, 9.0)? as f32, a.opt_number(6, 0.0)? as f32);
    let scale = a.opt_number(7, 1.0)? as f32;
    let seed = a.opt_integer(8, 1)?;
    let mut s = st.borrow_mut();
    s.require_offline("fx_play")?;
    s.rw_branch();
    if !x.is_finite() || !y.is_finite() || !z.is_finite() || !scale.is_finite() || scale <= 0.0 || scale > 100.0 || !(0..=2147483647).contains(&seed) {
        return Err(lua_err("fx_play requires finite offsets, scale in (0,100], seed in [0,2147483647]"));
    }
    let address = s.game.parts_joint(slot, joint);
    let facing = if s.game.fighter_f(slot, SF_FACING) < 0.0 { -1 } else { 1 };
    let (ow, frame) = (owner(&s), s.frame);
    let h = s.host.borrow_mut().fx_lab_play(&package, address, ow, slot + 1, joint, frame, facing, [x, y, z], scale, seed as u32);
    Ok(h as i64)
}
fn fx_attach(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<bool> {
    let package = a.check_string(1)?;
    let slot = present_arg(st, &a, 2)?;
    let joint = a.opt_integer(3, 0)? as i32;
    let (x, y, z) = (a.opt_number(4, 0.0)? as f32, a.opt_number(5, 9.0)? as f32, a.opt_number(6, 0.0)? as f32);
    let scale = a.opt_number(7, 1.0)? as f32;
    let mut s = st.borrow_mut();
    s.require_offline("fx_attach")?;
    s.rw_branch();
    finite_pos("fx_attach", [x, y, z, scale])?;
    let address = s.game.parts_joint(slot, joint);
    let facing = if s.game.fighter_f(slot, SF_FACING) < 0.0 { -1 } else { 1 };
    let (ow, frame) = (owner(&s), s.frame);
    let h = s.host.borrow_mut().fx_lab_attach(&package, address, ow, slot + 1, joint, frame, facing, [x, y, z], scale);
    Ok(h > 0)
}
fn fx_handle(a: &Args, i: usize) -> LuaResult<i32> {
    let h = a.check_integer(i)?;
    if h < 1 || h > 0x3FFFFFFF {
        return Err(lua_err("invalid effect instance handle"));
    }
    Ok(h as i32)
}
fn fx_world_position(a: &Args, first: usize) -> LuaResult<[f32; 3]> {
    let mut v = [0f32; 3];
    for (i, o) in v.iter_mut().enumerate() {
        let n = a.check_number(first + i)?;
        if !n.is_finite() || n.abs() > 100000.0 {
            return Err(lua_err("effect world position must be finite and within +/-100000"));
        }
        *o = n as f32;
    }
    Ok(v)
}
fn fx_world(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<i64> {
    let package = a.check_bytes(1)?;
    let scale = a.opt_number(5, 1.0)?;
    let seed = a.opt_integer(6, 1)?;
    let mut s = st.borrow_mut();
    s.require_offline("fx_world")?;
    if package.contains(&0) {
        return Err(lua_err("effect package contains NUL"));
    }
    let pos = fx_world_position(&a, 2)?;
    if !scale.is_finite() || scale <= 0.0 || scale > 100.0 || !(0..=2147483647).contains(&seed) {
        return Err(lua_err("fx_world requires scale in (0,100], seed in [0,2147483647]"));
    }
    s.rw_branch();
    let (ow, frame) = (owner(&s), s.frame);
    let h = s.host.borrow_mut().fx_lab_world(&String::from_utf8_lossy(&package), ow, frame, pos, scale as f32, seed as u32);
    Ok(h as i64)
}
fn fx_move(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<bool> {
    let h = fx_handle(&a, 1)?;
    let mut s = st.borrow_mut();
    s.require_offline("fx_move")?;
    let pos = fx_world_position(&a, 2)?;
    s.rw_branch();
    let ow = owner(&s);
    let r = s.host.borrow_mut().fx_lab_move(h, ow, pos);
    Ok(r > 0)
}
fn fx_control(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<bool> {
    const KEYS: [&str; 7] = ["opacity", "rate", "speed", "life", "size", "brightness", "turbulence"];
    const LO: [f32; 7] = [0.0, 0.0, 0.1, 0.1, 0.1, 0.0, 0.0];
    const HI: [f32; 7] = [1.0, 2.0, 3.0, 3.0, 3.0, 3.0, 3.0];
    let h = fx_handle(&a, 1)?;
    let em = a.check_integer(2)? as i32;
    let frames = a.opt_integer(4, 12)? as i32;
    let mut s = st.borrow_mut();
    s.require_offline("fx_control")?;
    s.rw_branch();
    let t = a.check_table(3)?;
    if !(-1..32).contains(&em) || !(0..=600).contains(&frames) {
        return Err(lua_err("emitter -1..31, tween 0..600 game frames"));
    }
    let mut values = [0f32; 7];
    for k in 0..7 {
        let v: Value = t.get(KEYS[k])?;
        values[k] = if v.is_nil() { 1.0 } else { super::caps::neg1(super::caps::one("fx_control", &v).check_number(1))? as f32 };
        if !values[k].is_finite() || values[k] < LO[k] || values[k] > HI[k] {
            return Err(lua_err(format!("effect control {} out of range", KEYS[k])));
        }
    }
    let ow = owner(&s);
    let r = s.host.borrow_mut().fx_lab_control(h, ow, em, values, frames);
    Ok(r > 0)
}
fn fx_end(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<bool> {
    let h = fx_handle(&a, 1)?;
    let fade = a.opt_integer(2, 12)? as i32;
    let mut s = st.borrow_mut();
    s.require_offline("fx_end")?;
    s.rw_branch();
    if !(0..=600).contains(&fade) {
        return Err(lua_err("effect fade 0..600 game frames"));
    }
    let ow = owner(&s);
    let r = s.host.borrow_mut().fx_lab_end(h, ow, fade);
    Ok(r > 0)
}
fn fx_instance(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Table> {
    let h = a.opt_integer(1, 0)? as i32;
    let s = st.borrow();
    let ow = owner(&s);
    let (alive, q) = s.host.borrow_mut().fx_lab_query(h, ow);
    let t = lua.create_table()?;
    t.boolean("alive", alive)?;
    t.int("age", q.age as i64)?;
    t.int("particles", q.particles as i64)?;
    t.int("emitters", q.emitters as i64)?;
    t.int("emitting", q.emitting as i64)?;
    t.int("refused", q.refused as i64)?;
    t.int("refused_emitters", q.refused_emitters as i64)?;
    Ok(t)
}
fn fx_stop(_: &mlua::Lua, st: &Shared, _a: Args) -> LuaResult<()> {
    let mut s = st.borrow_mut();
    s.require_offline("fx_stop")?;
    let ow = owner(&s);
    s.host.borrow_mut().fx_lab_stop(ow);
    Ok(())
}
fn fx_shake(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<()> {
    let amp = a.check_number(1)? as f32;
    let frames = a.check_integer(2)? as i32;
    let seed = a.opt_integer(3, 1)?;
    if !amp.is_finite() || amp < 0.0 || frames < 0 {
        return Err(lua_err("fx_shake requires amp >= 0 and frames >= 0"));
    }
    st.borrow().host.borrow_mut().fx_shake(amp, frames, seed as u32);
    Ok(())
}
fn fx_flash(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<()> {
    let (r, g, b, al) = (a.check_number(1)? as f32, a.check_number(2)? as f32, a.check_number(3)? as f32, a.check_number(4)? as f32);
    let frames = a.check_integer(5)? as i32;
    if !r.is_finite() || !g.is_finite() || !b.is_finite() || !al.is_finite() || frames < 0 {
        return Err(lua_err("fx_flash requires finite colour and frames >= 0"));
    }
    st.borrow().host.borrow_mut().fx_flash(r, g, b, al, frames);
    Ok(())
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    reg(e, gd, "dobj_solid", dobj_solid)?;
    reg(e, gd, "dobj_solid_off", dobj_solid_off)?;
    reg(e, gd, "dobj_tint", dobj_tint)?;
    reg(e, gd, "parts", parts)?;
    reg(e, gd, "parts_id", parts_id)?;
    reg(e, gd, "parts_id_off", parts_id_off)?;
    reg(e, gd, "parts_clear", parts_clear)?;
    reg(e, gd, "fx_play", fx_play)?;
    reg(e, gd, "fx_attach", fx_attach)?;
    reg(e, gd, "fx_world", fx_world)?;
    reg(e, gd, "fx_move", fx_move)?;
    reg(e, gd, "fx_control", fx_control)?;
    reg(e, gd, "fx_end", fx_end)?;
    reg(e, gd, "fx_instance", fx_instance)?;
    reg(e, gd, "fx_stop", fx_stop)?;
    reg(e, gd, "fx_shake", fx_shake)?;
    reg(e, gd, "fx_flash", fx_flash)?;
    reg(e, gd, "lab_peek", |lua, st, a| {
        let addr = a.check_integer(1)?;
        let n = (a.opt_integer(2, 16)? as i32).clamp(1, 64);
        let s = st.borrow();
        let Some(bytes) = u32::try_from(addr).ok().and_then(|ad| s.game.mem1_read(ad, n as usize)) else {
            return Ok(Value::Nil);
        };
        let hex: String = bytes.iter().map(|b| format!("{b:02X}")).collect();
        Ok(Value::String(lua.create_string(hex)?))
    })?;
    Ok(())
}
