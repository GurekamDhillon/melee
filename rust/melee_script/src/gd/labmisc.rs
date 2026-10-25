//! Small Geno Lab calls: `lab_request`, `lab_mode`, `lab_leave`, `lab_now`, `lab_common`, `lab_env`, `floor_below`,
//! `set_shield`, `set_scale`, `rollbacks`, `rollbacks_clear`, `tbd_request` (gw_script.c "Stage D / E").
use super::{lua_err, reg, slot_arg, TableExt};
use crate::engine::Engine;
use crate::game::*;
use mlua::{MultiValue, Result as LuaResult, Table, Value};

/// `LAB_C_*` (script_lab.h `ScriptGame_LabCommonF`)
pub mod c {
    pub const KB_SPEED: i32 = 0;
    pub const KB_MAX: i32 = 1;
    pub const ANGLE_AIR_361: i32 = 2;
    pub const ANGLE_GROUND_MAX: i32 = 3;
    pub const ANGLE_GROUND_KB0: i32 = 4;
    pub const ANGLE_GROUND_KB1: i32 = 5;
    pub const HITSTUN_MUL: i32 = 6;
    pub const DI_DEGREES: i32 = 7;
    pub const KB_DECAY: i32 = 8;
    pub const SQUAT_MUL: i32 = 9;
    pub const BLAST_LEFT: i32 = 10;
    pub const BLAST_RIGHT: i32 = 11;
    pub const BLAST_TOP: i32 = 12;
    pub const BLAST_BOTTOM: i32 = 13;
    pub const LCANCEL_WINDOW: i32 = 14;
    pub const LCANCEL_DIV: i32 = 15;
    pub const STICK_SMASH_DZ: i32 = 16;
    pub const TUMBLE_WIGGLE: i32 = 17;
    pub const TUMBLE_WINDOW: i32 = 18;
    pub const TECH_ROLL_STICK: i32 = 19;
    pub const SDI_MIN: i32 = 20;
    pub const SDI_WINDOW: i32 = 21;
    pub const ASDI_SCALE: i32 = 22;
    pub const STICK_DZ_X: i32 = 23;
    pub const STICK_DZ_Y: i32 = 24;
    pub const COUNT: i32 = 25;
}

/// One row of the rollback visualiser ring (`gw_RbViz_Get`).
#[derive(Clone, Copy, Default, Debug)]
pub struct RbRow {
    pub frame: i32,
    pub first: i32,
    pub depth: i32,
    pub cause: i32,
    pub kind: i32,
    pub mismatch: bool,
    pub ms: f32,
}

/// `gw_RbViz_FirstMismatch`
#[derive(Clone, Debug)]
pub struct RbMismatch {
    pub frame: i32,
    pub count: i32,
    pub place: String,
    pub va: u32,
    pub was: i32,
    pub now: i32,
}

/// `SYSTEMTIME` local time as the Lab stamps versions with it (UTC offset is the platform's business, so the host asks).
fn fmt_now(t: (u32, u32, u32, u32, u32, u32), long: bool) -> String {
    let (y, mo, d, h, mi, s) = t;
    if long {
        format!("{y:04}-{mo:02}-{d:02} {h:02}:{mi:02}:{s:02}")
    } else {
        format!("{y:04}{mo:02}{d:02}-{h:02}{mi:02}{s:02}")
    }
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    // gd.lab_request([clear]) -> true when the frontend's LAB entry (or MELEE_LAB=1) asked for the Lab this session
    reg(e, gd, "lab_request", |_, st, a| {
        let clear = a.truthy(1);
        let mut s = st.borrow_mut();
        let was = s.lab_request;
        if clear {
            s.lab_request = false;
        }
        Ok(was)
    })?;
    // deprecated in API 2: the main-menu TBD tile is gone, so there is never a request
    reg(e, gd, "tbd_request", |_, _, _| Ok(false))?;
    reg(e, gd, "lab_mode", |_, st, _| Ok(st.borrow().game.geno_lab_mode_active()))?;
    reg(e, gd, "lab_leave", |lua, st, a| {
        let to = a.opt_string(1, "css")?;
        let mut s = st.borrow_mut();
        s.require_gameplay("lab_leave")?;
        let where_ = match to.as_str() {
            "css" => 0,
            "sss" => 1,
            "menu" => 2,
            "restart" => 3, // the same match again
            _ => {
                return Err(lua_err(format!(
                    "gd.lab_leave: \"css\", \"sss\", \"menu\" or \"restart\" (got \"{to}\")"
                )))
            }
        };
        let extra = s.game.fighter_i(4, SI_SLOT_TYPE) != 3 || s.game.fighter_i(5, SI_SLOT_TYPE) != 3;
        let why: Option<&str> = if where_ < 2 && extra {
            Some("six-fighter LAB cannot enter four-slot CSS/SSS; use menu or restart")
        } else if !s.game.geno_lab_leave(where_) {
            Some("LAB leave refused: no active LAB match or unsupported destination")
        } else {
            None
        };
        let reply = |why: Option<&str>| -> LuaResult<MultiValue> {
            let mut v = vec![Value::Boolean(why.is_none())];
            if let Some(w) = why {
                v.push(Value::String(lua.create_string(w)?));
            }
            Ok(MultiValue::from_iter(v))
        };
        if let Some(w) = why {
            s.log(&format!("gd.lab_leave: refused: {w}"));
            return reply(Some(w));
        }
        s.paused = false; // the match has to run a frame to end
        reply(None)
    })?;
    // gd.lab_now([long]): the local time as a version tag; the sandbox has no os.date
    reg(e, gd, "lab_now", |_, st, a| {
        let t = st.borrow().host.borrow().local_time();
        Ok(fmt_now(t, a.truthy(1)))
    })?;
    reg(e, gd, "lab_common", |lua, st, _| {
        let s = st.borrow();
        let g = &*s.game;
        let t = lua.create_table()?;
        for (k, f) in [
            ("lcancel_window", c::LCANCEL_WINDOW), ("lcancel_div", c::LCANCEL_DIV), ("hitstun_mul", c::HITSTUN_MUL),
            ("kb_speed", c::KB_SPEED), ("kb_decay", c::KB_DECAY), ("stick_smash_dz", c::STICK_SMASH_DZ),
            ("tumble_wiggle", c::TUMBLE_WIGGLE), ("tumble_window", c::TUMBLE_WINDOW),
            ("tech_roll_stick", c::TECH_ROLL_STICK), ("sdi_min", c::SDI_MIN), ("sdi_window", c::SDI_WINDOW),
            ("asdi_scale", c::ASDI_SCALE), ("stick_dz_x", c::STICK_DZ_X), ("stick_dz_y", c::STICK_DZ_Y),
        ] {
            t.num(k, g.lab_common_f(f) as f64)?;
        }
        Ok(t)
    })?;
    // gd.lab_env(name): MELEE_LAB_<name> from the environment (only that family), or nil
    reg(e, gd, "lab_env", |lua, st, a| {
        let key = format!("MELEE_LAB_{}", a.check_string(1)?);
        let v = st.borrow().host.borrow().getenv(&key);
        Ok(match v {
            Some(v) if !v.is_empty() => Value::String(lua.create_string(&v)?),
            _ => Value::Nil,
        })
    })?;
    // gd.floor_below(x, y [, depth]): the y of the first floor line under (x, y), or nil; in a match only
    reg(e, gd, "floor_below", |_, st, a| {
        let x = a.check_number(1)? as f32;
        let y = a.check_number(2)? as f32;
        let depth = a.opt_number(3, 200.0)? as f32;
        let s = st.borrow();
        if !s.match_active {
            return Ok(Value::Nil);
        }
        let fy = s.game.lab_floor_y(x, y, depth);
        Ok(if fy <= -999999.0 { Value::Nil } else { Value::Number(fy as f64) })
    })?;
    reg(e, gd, "set_shield", |_, st, a| {
        let slot = slot_arg(&a, 1)? as i32;
        let h = a.check_number(2)? as f32;
        let mut s = st.borrow_mut();
        s.require_offline("set_shield")?;
        s.rw_branch();
        s.game.lab_set_shield(slot, if h < 0.0 { 0.0 } else { h });
        Ok(())
    })?;
    reg(e, gd, "set_scale", |_, st, a| {
        let slot = slot_arg(&a, 1)? as i32;
        let v = a.check_number(2)? as f32;
        let mut s = st.borrow_mut();
        s.require_offline("set_scale")?;
        if !(v == v) || v <= 0.0 {
            return Err(lua_err("gd.set_scale: scale must be a positive number"));
        }
        s.rw_branch();
        s.game.lab_set_scale(slot, v.clamp(0.5, 2.0));
        Ok(())
    })?;
    reg(e, gd, "rollbacks", |lua, st, a| {
        const KINDS: [&str; 4] = ["?", "synctest", "fake", "netplay"];
        let n = (a.opt_integer(1, 120)? as i32).clamp(0, 600);
        let s = st.borrow();
        let g = &*s.game;
        let t = lua.create_table()?;
        t.int("total", g.rbviz_total() as i64)?;
        let list = lua.create_table()?;
        let mut i = 0;
        while i < n {
            let Some(r) = g.rbviz_get(i) else { break };
            let row = lua.create_table()?;
            row.int("frame", r.frame as i64)?;
            row.int("first", r.first as i64)?;
            row.int("depth", r.depth as i64)?;
            row.int("cause", r.cause as i64 + 1)?;
            row.string("kind", if (0..=3).contains(&r.kind) { KINDS[r.kind as usize] } else { "?" })?;
            row.boolean("mismatch", r.mismatch)?;
            row.num("ms", r.ms as f64)?;
            list.raw_set(i + 1, row)?;
            i += 1;
        }
        t.set("list", list)?;
        if let Some(m) = g.rbviz_first_mismatch().filter(|m| m.frame >= 0) {
            let mm = lua.create_table()?;
            mm.int("frame", m.frame as i64)?;
            mm.int("count", m.count as i64)?;
            mm.string("where", &m.place)?;
            mm.int("va", m.va as i64)?;
            mm.int("was", m.was as i64)?;
            mm.int("now", m.now as i64)?;
            t.set("mismatch", mm)?;
        }
        Ok(t)
    })?;
    reg(e, gd, "rollbacks_clear", |_, st, _| {
        st.borrow_mut().game.rbviz_reset();
        Ok(())
    })?;
    Ok(())
}
