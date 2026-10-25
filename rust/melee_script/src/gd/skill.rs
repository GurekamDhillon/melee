//! Skill events and the skill ring (`gw_script_skill.inc`): skill_history, skill_state, skill_kinds, skill_thresholds, and the
//! on_<technique> / on_skill tables. The producer is game-side; rows arrive through `State::skill_event`.
use super::caps::entity;
use super::{lua_err, reg, TableExt};
use crate::args::Args;
use crate::engine::{Engine, Shared};
use crate::events::{GsEvent, SKILL_KINDS};
use mlua::Table;

type LuaResult<T> = mlua::Result<T>;

pub const SKILL_NAMES: [&str; SKILL_KINDS] = [
    "", "lcancel", "lcancel_miss", "auto_cancel", "lcancel_hit", "wavedash", "waveland", "ledge_dash", "air_dodge", "perfect_shield",
    "tech", "tech_miss", "dash_dance", "short_hop", "full_hop", "fast_fall", "shield_drop", "jump_cancel_grab", "jump_cancel_usmash",
    "sdi", "combo", "combo_end", "shock", "shock_hit", "shock_end",
];
pub const SKILL_HOOKS: [&str; SKILL_KINDS] = [
    "", "on_lcancel", "on_lcancel_miss", "on_auto_cancel", "on_lcancel_hit", "on_wavedash", "on_waveland", "on_ledge_dash",
    "on_air_dodge", "on_perfect_shield_skill", "on_tech", "on_tech_miss", "on_dash_dance", "on_short_hop", "on_full_hop",
    "on_fast_fall", "on_shield_drop", "on_jump_cancel_grab", "on_jump_cancel_usmash", "on_sdi", "on_combo", "on_combo_end",
    "on_shock", "on_shock_hit", "on_shock_end",
];

fn f(bits: i32) -> f64 {
    f32::from_bits(bits as u32) as f64
}

/// `gs_skill_fill`
#[allow(clippy::too_many_arguments)]
fn fill(lua: &mlua::Lua, kind: i32, entity: i32, frame: i32, i1: i32, i2: i32, i3: i32, f1b: i32, f2b: i32) -> LuaResult<Table> {
    const AERIALS: [&str; 5] = ["nair", "fair", "bair", "uair", "dair"];
    const TECH_DIR: [&str; 5] = ["in_place", "toward", "away", "wall", "ceiling"];
    const WHY: [&str; 5] = ["?", "expired", "spent", "cleared", "replaced"];
    let (f1, f2) = (f(f1b), f(f2b));
    let t = lua.create_table()?;
    t.string("kind", SKILL_NAMES[kind as usize])?;
    t.int("port", ((entity - 1) % 6 + 1) as i64)?;
    t.int("entity", entity as i64)?;
    t.boolean("subfighter", entity > 6)?;
    t.int("frame", frame as i64)?;
    let i = |k: &str, v: i32| t.int(k, v as i64);
    match kind {
        1..=4 => {
            t.string("aerial", if (0..5).contains(&i1) { AERIALS[i1 as usize] } else { "?" })?;
            t.boolean("hit", i2 != 0)?;
            if kind != 3 {
                i("trigger_age", i3)?;
                t.num("lag", f1)?;
                t.num("lag_cancelled", f2)?;
            }
        }
        5 | 6 => {
            i("frames_after_jump", i1)?;
            i("dodge_frames", i2)?;
            t.num("speed", f1)?;
            t.num("speed_abs", f2)?;
        }
        7 => {
            i("frames_since_ledge", i1)?;
            i("dodge_frames", i2)?;
            t.num("speed", f1)?;
            t.num("speed_abs", f2)?;
        }
        8 => {
            i("frames_after_jump", i1)?;
            t.num("stick_x", f1)?;
            t.num("stick_y", f2)?;
        }
        9 => {
            t.boolean("projectile", i1 != 0)?;
            if i2 > 0 {
                i("attacker", i2)?;
            }
            i("window_frame", i3)?;
            t.num("damage", f1)?;
        }
        10 => {
            t.string("direction", if (0..5).contains(&i1) { TECH_DIR[i1 as usize] } else { "?" })?;
            t.boolean("roll", i2 != 0)?;
            t.boolean("wall_jump", i3 != 0)?;
        }
        11 => t.boolean("facing_down", i1 != 0)?,
        12 => {
            i("reversals", i1)?;
            i("frames_since_dash", i2)?;
        }
        13 | 14 => {
            i("jumpsquat", i1)?;
            t.num("speed", f1)?;
            t.num("expected", f2)?;
        }
        15 => {
            i("air_frames", i1)?;
            t.num("speed", f1)?;
        }
        16 => i("shield_frames", i1)?,
        17 | 18 => i("jumpsquat", i1)?,
        19 => {
            i("count", i1)?;
            i("hitlag_left", i2)?;
            t.num("dx", f1)?;
            t.num("dy", f2)?;
        }
        20 => {
            i("count", i1)?;
            if i2 > 0 {
                i("attacker", i2)?;
            }
            t.num("damage", f1)?;
            t.num("last_damage", f2)?;
        }
        21 => {
            i("count", i1)?;
            if i2 > 0 {
                i("attacker", i2)?;
            }
            t.boolean("ko", i3 != 0)?;
            t.num("damage", f1)?;
        }
        22 => {
            i("frames", i1)?;
            i("charges", i2)?;
            i("bonus", i3)?;
            t.num("hitstun_multiplier", f1)?;
        }
        23 => {
            i("base_hitstun", i1)?;
            i("extra_hitstun", i2)?;
            i("charges_left", i3)?;
            t.num("hitstun_multiplier", f1)?;
        }
        24 => {
            t.string("reason", if (1..5).contains(&i1) { WHY[i1 as usize] } else { "?" })?;
            i("hits", i2)?;
        }
        _ => {}
    }
    Ok(t)
}
/// `gs_skill_push`
pub fn skill_push(lua: &mlua::Lua, e: &GsEvent) -> LuaResult<Table> {
    fill(lua, e.b, e.a, e.item_reason, e.c, e.d, e.item_amount, e.item_y, e.item_colour)
}

fn skill_history(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Table> {
    let e = entity(&a, 1)?;
    let n = if a.none_or_nil(2) { 16 } else { a.check_integer(2)? as i32 };
    let mut want = 0;
    if !(0..=256).contains(&n) {
        return Err(lua_err("skill history length must be 0-256"));
    }
    if !a.none_or_nil(3) {
        let k = a.check_string(3)?;
        if let Some(i) = (1..SKILL_KINDS).find(|&i| SKILL_NAMES[i] == k) {
            want = i as i32;
        }
        if want == 0 {
            return Err(lua_err("unknown skill kind"));
        }
    }
    let s = st.borrow();
    let g = &*s.game;
    let out = lua.create_table()?;
    let (total, mut n_out) = (g.skill_row_count(), 0);
    let mut i = 0;
    while i < total && n_out < n {
        let kind = g.skill_row(i, 1);
        if g.skill_row(i, 2) == e && (want == 0 || kind == want) && (1..SKILL_KINDS as i32).contains(&kind) {
            n_out += 1;
            out.raw_set(
                n_out as i64,
                fill(lua, kind, e, g.skill_row(i, 0), g.skill_row(i, 3), g.skill_row(i, 4), g.skill_row(i, 5), g.skill_row(i, 6), g.skill_row(i, 7))?,
            )?;
        }
        i += 1;
    }
    Ok(out)
}
fn skill_state(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Table> {
    let e = entity(&a, 1)?;
    let s = st.borrow();
    let g = &*s.game;
    let t = lua.create_table()?;
    t.int("combo_count", g.skill_state(e, 0) as i64)?;
    t.num("combo_damage", f(g.skill_state(e, 1)))?;
    t.int("combo_attacker", g.skill_state(e, 2) as i64)?;
    t.int("air_frames", g.skill_state(e, 3) as i64)?;
    t.int("frames_in_state", g.skill_state(e, 4) as i64)?;
    t.boolean("aerial_hit", g.skill_state(e, 5) != 0)?;
    Ok(t)
}
fn skill_kinds(lua: &mlua::Lua, _: &Shared, _a: Args) -> LuaResult<Table> {
    let t = lua.create_table()?;
    for i in 1..SKILL_KINDS {
        t.raw_set(i as i64, SKILL_NAMES[i])?;
    }
    Ok(t)
}
fn skill_thresholds(lua: &mlua::Lua, st: &Shared, _a: Args) -> LuaResult<Table> {
    let s = st.borrow();
    let g = &*s.game;
    let t = lua.create_table()?;
    t.num("wave_min_speed", f(g.skill_const(0)))?;
    t.int("wavedash_max_airborne_frames", g.skill_const(1) as i64)?;
    t.int("ledge_dash_frames", g.skill_const(2) as i64)?;
    t.int("dash_dance_frames", g.skill_const(3) as i64)?;
    t.int("lcancel_window_frames", g.skill_const(4) as i64)?;
    t.int("powershield_window_frames", g.skill_const(5) as i64)?;
    Ok(t)
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    reg(e, gd, "skill_history", skill_history)?;
    reg(e, gd, "skill_state", skill_state)?;
    reg(e, gd, "skill_kinds", skill_kinds)?;
    reg(e, gd, "skill_thresholds", skill_thresholds)?;
    Ok(())
}
