//! Crits (`gw_script_crit.inc`) and the CPU technique overlays (`gw_script_cpu.inc`): crit, crit_force, crit_seed,
//! cpu_technical, cpu_assist. The crit decision and the CPU assist live in the game half; the on_crit event is not ported.
use super::arena::cur_owner;
use super::camera::{bitsf, fbits};
use super::caps::{entity, key_str, neg1, number, one, pairs, write};
use super::{lua_err, reg, slot_arg, TableExt};
use crate::args::Args;
use crate::engine::{Engine, Shared};
use mlua::{Table, Value};

type LuaResult<T> = mlua::Result<T>;

const TAGS: [&str; 10] = ["default", "jab", "dash_attack", "tilt", "smash", "aerial", "grab", "throw", "special", "projectile"];

#[derive(Clone, Copy)]
struct Slot {
    chance: f32,
    mult: f32,
    mult_max: f32,
    launch: f32,
}

/// `gs_item_keys`: every key must be a string in `allowed`
pub(crate) fn item_keys(t: &Table, allowed: &[&str]) -> LuaResult<()> {
    for (k, _) in pairs(t)? {
        match key_str(&k) {
            Some(s) if allowed.contains(&s.as_str()) => {}
            _ => return Err(lua_err("unknown item option/payload key")),
        }
    }
    Ok(())
}

fn field_number(t: &Table, key: &str, lo: f64, hi: f64) -> LuaResult<Option<f32>> {
    let v: Value = t.get(key)?;
    if v.is_nil() {
        return Ok(None);
    }
    let n = neg1(one("crit", &v).check_number(1))?;
    if !n.is_finite() || n < lo || n > hi {
        return Err(lua_err("fighter value must be finite and in range"));
    }
    Ok(Some(n as f32))
}

fn slot_option(t: &Table, s: &mut Slot, is_default: bool) -> LuaResult<()> {
    const KEYS: [&str; 7] = ["chance", "multiplier", "multiplier_max", "launch", "min_percent", "force", "tags"];
    item_keys(t, if is_default { &KEYS } else { &KEYS[..4] })?;
    if let Some(v) = field_number(t, "chance", 0.0, 1.0)? {
        s.chance = v;
    }
    let (mut has_mult, mut has_max) = (false, false);
    if let Some(v) = field_number(t, "multiplier", 1.0, 16.0)? {
        s.mult = v;
        has_mult = true;
    }
    if let Some(v) = field_number(t, "multiplier_max", 1.0, 16.0)? {
        s.mult_max = v;
        has_max = true;
    }
    if let Some(v) = field_number(t, "launch", 1.0, 4.0)? {
        s.launch = v;
    }
    if has_mult && !has_max {
        s.mult_max = s.mult;
    }
    if s.mult_max < s.mult {
        return Err(lua_err("crit multiplier_max is below multiplier"));
    }
    Ok(())
}

fn crit(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Value> {
    let e = entity(&a, 1)?;
    if a.len() == 1 {
        let s = st.borrow();
        let g = &*s.game;
        let ow = g.crit_read(e, 0);
        if ow == 0 {
            return Ok(Value::Nil);
        }
        let f = |field: i32| bitsf(g.crit_read(e, field)) as f64;
        let t = lua.create_table()?;
        t.int("owner", ow as i64)?;
        t.num("chance", f(10))?;
        t.num("multiplier", f(11))?;
        t.num("multiplier_max", f(12))?;
        t.num("launch", f(13))?;
        t.num("min_percent", f(2))?;
        t.int("force", g.crit_read(e, 3) as i64)?;
        t.int("hits", g.crit_read(e, 4) as i64)?;
        t.int("crits", g.crit_read(e, 5) as i64)?;
        t.num("max_multiplier", f(6))?;
        let tags = lua.create_table()?;
        let has = g.crit_read(e, 1) as u32;
        for i in 1..10 {
            if has & (1 << i) != 0 {
                let r = lua.create_table()?;
                r.num("chance", f(10 + i * 4))?;
                r.num("multiplier", f(11 + i * 4))?;
                r.num("multiplier_max", f(12 + i * 4))?;
                r.num("launch", f(13 + i * 4))?;
                tags.set(TAGS[i as usize], r)?;
            }
        }
        t.set("tags", tags)?;
        return Ok(Value::Table(t));
    }
    let mut s = st.borrow_mut();
    write(&mut s, "crit")?;
    let owner = cur_owner(&s);
    let ow = s.game.crit_read(e, 0);
    if ow != 0 && ow != owner {
        return Ok(Value::Boolean(false));
    }
    if a.get(2).is_nil() {
        s.rw_branch();
        return Ok(Value::Boolean(s.game.crit_set(e, owner, 0, 0, 0, 0, 0, 0, 0, 2) != 0));
    }
    let t = a.check_table(2)?;
    let mut slots = [Slot { chance: 0.0, mult: 1.5, mult_max: 1.5, launch: 1.0 }; 10];
    let mut has = 1u32;
    slot_option(&t, &mut slots[0], true)?;
    let min_percent = field_number(&t, "min_percent", 0.0, 999.0)?.unwrap_or(0.0);
    let fv: Value = t.get("force")?;
    let mut force = 0;
    if !fv.is_nil() {
        let n = neg1(one("crit", &fv).check_integer(1))?;
        if !(0..=255).contains(&n) {
            return Err(lua_err("crit force must be 0-255"));
        }
        force = n as i32;
    }
    let tv: Value = t.get("tags")?;
    if !tv.is_nil() {
        let Value::Table(tt) = &tv else {
            return Err(lua_err(format!(
                "bad argument #{} to 'crit' (table expected, got {})",
                -1,
                crate::args::typename(Some(&tv))
            )));
        };
        for (k, v) in pairs(tt)? {
            let name = key_str(&k);
            let Some(i) = name.and_then(|n| TAGS.iter().skip(1).position(|c| *c == n)).map(|i| i + 1) else {
                return Err(lua_err("unknown crit move tag"));
            };
            slots[i] = slots[0];
            let Value::Table(vt) = &v else {
                return Err(lua_err(format!("bad argument #{} to 'crit' (table expected, got {})", 3, crate::args::typename(Some(&v)))));
            };
            slot_option(vt, &mut slots[i], false)?;
            has |= 1 << i;
        }
    }
    s.rw_branch();
    if s.game.crit_set(e, owner, 0, 0, 0, 0, 0, fbits(min_percent), force, 1 | 8) == 0 {
        return Ok(Value::Boolean(false));
    }
    for i in 0..10 {
        if has & (1 << i) != 0 {
            let sl = slots[i];
            if s.game.crit_set(e, owner, i as i32, fbits(sl.chance), fbits(sl.mult), fbits(sl.mult_max), fbits(sl.launch), 0, 0, 4) == 0 {
                return Ok(Value::Boolean(false));
            }
        }
    }
    Ok(Value::Boolean(true))
}

fn crit_force(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<bool> {
    let e = entity(&a, 1)?;
    let n = a.check_integer(2)?;
    if !(0..=255).contains(&n) {
        return Err(lua_err("crit force must be 0-255"));
    }
    let mut s = st.borrow_mut();
    write(&mut s, "crit_force")?;
    let owner = cur_owner(&s);
    if s.game.crit_read(e, 0) != owner {
        return Ok(false);
    }
    s.rw_branch();
    let min = s.game.crit_read(e, 2);
    Ok(s.game.crit_set(e, owner, 0, 0, 0, 0, 0, min, n as i32, 8) != 0)
}

fn crit_seed(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<()> {
    let n = a.check_integer(1)?;
    if !(1..=2147483647).contains(&n) {
        return Err(lua_err("crit seed must be 1-2147483647"));
    }
    let mut s = st.borrow_mut();
    write(&mut s, "crit_seed")?;
    s.rw_branch();
    s.game.crit_seed(n as i32);
    Ok(())
}

fn cpu_technical(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Value> {
    let slot = slot_arg(&a, 1)? as i32;
    if a.none_or_nil(2) {
        const FIELDS: [&str; 7] =
            ["skill", "opportunities", "lcancel_inputs", "tech_inputs", "missed_decisions", "last_event", "reaction_frames"];
        let s = st.borrow();
        let owner = cur_owner(&s);
        let t = lua.create_table()?;
        for (i, f) in FIELDS.iter().enumerate() {
            t.int(f, s.game.cpu_technical_state(slot, owner, i as i32) as i64)?;
        }
        t.boolean("enabled", s.game.cpu_technical_state(slot, owner, 0) != 0)?;
        t.string("policy", "original landing-cancel / hitstun ground-tech assist")?;
        return Ok(Value::Table(t));
    }
    let skill = a.check_integer(2)?;
    let seed = a.opt_integer(3, 1)?;
    if !(0..=3).contains(&skill) || !(1..=2147483647).contains(&seed) {
        return Err(lua_err("cpu_technical: skill 0..3 and seed 1..2147483647 required"));
    }
    let mut s = st.borrow_mut();
    s.require_offline("cpu_technical")?;
    s.rw_branch();
    let owner = cur_owner(&s);
    Ok(Value::Boolean(s.game.cpu_technical(slot, owner, skill as i32, seed as i32) != 0))
}

const ASSIST_NAMES: [&str; 5] = ["lcancel", "perfect_shield", "tech", "wavedash", "fast_fall"];
const ASSIST_DIRS: [&str; 4] = ["in", "away", "toward", "random"];

fn assist_prob(t: &Table, key: &str) -> LuaResult<i32> {
    let v: Value = t.get(key)?;
    if v.is_nil() {
        return Ok(0);
    }
    let aa = one("cpu_assist", &v);
    let msg = format!("cpu_assist: {key} must be a number 0..1");
    if !aa.is_number(1) {
        return Err(lua_err(msg));
    }
    let n = aa.check_number(1)?;
    if !(0.0..=1.0).contains(&n) {
        return Err(lua_err(msg));
    }
    Ok((n * 1000.0 + 0.5) as i32)
}

fn cpu_assist(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Value> {
    let slot = slot_arg(&a, 1)? as i32;
    if a.len() < 2 {
        let s = st.borrow();
        let owner = cur_owner(&s);
        let g = &*s.game;
        let rd = |f: i32| g.cpu_assist_state(slot, owner, f);
        let t = lua.create_table()?;
        t.boolean("enabled", rd(0) != 0)?;
        if rd(0) != 0 {
            t.int("seed", rd(7) as i64)?;
            t.string("tech_dir", ASSIST_DIRS[(rd(6) & 3) as usize])?;
        }
        let p = lua.create_table()?;
        for (i, n) in ASSIST_NAMES.iter().enumerate() {
            p.num(n, rd(1 + i as i32) as f64 / 1000.0)?;
        }
        t.set("probability", p)?;
        let c = lua.create_table()?;
        for (i, n) in ASSIST_NAMES.iter().enumerate() {
            let r = lua.create_table()?;
            r.int("opportunities", rd(8 + i as i32) as i64)?;
            r.int("performed", rd(13 + i as i32) as i64)?;
            c.set(*n, r)?;
        }
        t.set("counters", c)?;
        return Ok(Value::Table(t));
    }
    let mut s = st.borrow_mut();
    s.require_offline("cpu_assist")?;
    s.rw_branch();
    let owner = cur_owner(&s);
    if a.get(2).is_nil() {
        return Ok(Value::Boolean(s.game.cpu_assist_clear_slot(slot, owner) != 0));
    }
    let t = a.check_table(2)?;
    let lc = assist_prob(&t, "lcancel")?;
    let ps = assist_prob(&t, "perfect_shield")?;
    let tech = assist_prob(&t, "tech")?;
    let wd = assist_prob(&t, "wavedash")?;
    let ff = assist_prob(&t, "fast_fall")?;
    let mut tdir = 1;
    let dv: Value = t.get("tech_dir")?;
    if !dv.is_nil() {
        let d = key_str(&dv);
        match d.and_then(|d| ASSIST_DIRS.iter().position(|x| *x == d)) {
            Some(i) => tdir = i as i32,
            None => return Err(lua_err("cpu_assist: tech_dir is \"in\", \"away\", \"toward\" or \"random\"")),
        }
    }
    let mut seed = 1i64;
    let sv: Value = t.get("seed")?;
    if !sv.is_nil() {
        let msg = "cpu_assist: seed must be an integer 1..2147483647";
        let Value::Integer(n) = sv else { return Err(lua_err(msg)) };
        if !(1..=2147483647).contains(&n) {
            return Err(lua_err(msg));
        }
        seed = n;
    }
    Ok(Value::Boolean(s.game.cpu_assist(slot, owner, lc, ps, tech, tdir, wd, ff, seed as i32) != 0))
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    reg(e, gd, "crit", crit)?;
    reg(e, gd, "crit_force", crit_force)?;
    reg(e, gd, "crit_seed", crit_seed)?;
    reg(e, gd, "cpu_technical", cpu_technical)?;
    reg(e, gd, "cpu_assist", cpu_assist)?;
    let _ = number;
    Ok(())
}
