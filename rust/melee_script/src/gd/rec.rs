//! `gd.rec_*`: the LAB recorder's Lua half (`pc/platform/gw_script_rec.inc`, docs/geno.md "Keys and commands"). The
//! gating, name rules and argument shapes are here; the recorder state machine and its `.gdlp` files are behind
//! [`crate::ScriptGame`]'s `rec_*` methods.
use super::{reg, TableExt};
use crate::engine::{Engine, State};
use mlua::{Lua, MultiValue, Result as LuaResult, Table, Value};

fn fail(lua: &Lua, why: &str) -> LuaResult<MultiValue> {
    Ok(MultiValue::from_iter([Value::Boolean(false), Value::String(lua.create_string(why)?)]))
}
fn one(v: Value) -> LuaResult<MultiValue> {
    Ok(MultiValue::from_iter([v]))
}
fn text(lua: &Lua, s: &str) -> LuaResult<Value> {
    Ok(Value::String(lua.create_string(s)?))
}

/// `gr_name_ok`
pub fn name_ok(n: &str) -> bool {
    !n.is_empty() && n.len() <= 40 && n.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
}

/// `gr_gate`: the offline check (raises), then the reasons a recording cannot run here.
fn gate(s: &mut State, f: &str) -> LuaResult<Option<&'static str>> {
    s.require_offline(f)?;
    Ok(if s.game.onep_state_active() {
        Some("retail 1P state active")
    } else if s.game.stage_slots_active() {
        Some("stage slot session active")
    } else if !s.match_active {
        Some("not in a match")
    } else if s.host.borrow().session() {
        Some("not during a netplay/rollback session")
    } else {
        None
    })
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    // gd.rec_start([name]) -> name | false, why
    reg(e, gd, "rec_start", |lua, st, a| {
        let mut s = st.borrow_mut();
        if let Some(w) = gate(&mut s, "rec_start")? {
            return fail(lua, w);
        }
        let name = if a.none_or_nil(1) {
            let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
            format!("rec_{t}")
        } else {
            a.check_string(1)?
        };
        if !name_ok(&name) {
            return fail(lua, "name: letters, digits, _ and - only (40 max)");
        }
        match s.game.rec_start(&name) {
            Ok(n) => one(text(lua, &n)?),
            Err(w) => fail(lua, &w),
        }
    })?;
    // gd.rec_stop() -> true, frames | false, why
    reg(e, gd, "rec_stop", |lua, st, _| {
        let mut s = st.borrow_mut();
        s.require_offline("rec_stop")?;
        match s.game.rec_stop() {
            Ok(n) => Ok(MultiValue::from_iter([Value::Boolean(true), Value::Integer(n as i64)])),
            Err(w) => fail(lua, &w),
        }
    })?;
    // gd.rec_mark([label]) -> frame offset | false, why
    reg(e, gd, "rec_mark", |lua, st, a| {
        let label = a.opt_string(1, "")?;
        let mut s = st.borrow_mut();
        match s.game.rec_mark(&label) {
            Ok(n) => one(Value::Integer(n as i64)),
            Err(w) => fail(lua, &w),
        }
    })?;
    // gd.rec_play(name [, map [, loop]]) -> true | false, why
    reg(e, gd, "rec_play", |lua, st, a| {
        let name = a.check_string(1)?;
        let map = a.opt_string(2, "")?;
        let looping = a.truthy(3);
        let mut s = st.borrow_mut();
        if let Some(w) = gate(&mut s, "rec_play")? {
            return fail(lua, w);
        }
        if !name_ok(&name) {
            return fail(lua, "bad recording name");
        }
        match s.game.rec_play(&name, &map, looping) {
            Ok(()) => one(Value::Boolean(true)),
            Err(w) => fail(lua, &w),
        }
    })?;
    reg(e, gd, "rec_play_stop", |lua, st, _| {
        let mut s = st.borrow_mut();
        s.require_offline("rec_play_stop")?;
        match s.game.rec_play_stop() {
            Ok(()) => one(Value::Boolean(true)),
            Err(w) => fail(lua, &w),
        }
    })?;
    // gd.rec_trim(name, from, to [, newname]) -> newname | false, why
    reg(e, gd, "rec_trim", |lua, st, a| {
        let name = a.check_string(1)?;
        let from = a.check_integer(2)? as i32;
        let to = a.check_integer(3)? as i32;
        let dst = if a.none_or_nil(4) { format!("{name}-cut") } else { a.check_string(4)? };
        let mut s = st.borrow_mut();
        if let Some(w) = gate(&mut s, "rec_trim")? {
            return fail(lua, w);
        }
        if !name_ok(&name) {
            return fail(lua, "bad recording name");
        }
        if !name_ok(&dst) {
            return fail(lua, "bad new name");
        }
        match s.game.rec_trim(&name, from.max(0), to, &dst) {
            Ok(n) => one(text(lua, &n)?),
            Err(w) => fail(lua, &w),
        }
    })?;
    // gd.rec_keep_last(seconds [, name]) -> newname | false, why
    reg(e, gd, "rec_keep_last", |lua, st, a| {
        let sec = a.check_number(1)?;
        let name = if a.none_or_nil(2) { None } else { Some(a.check_string(2)?) };
        let mut s = st.borrow_mut();
        // the C function ends in l_rec_trim, so the same gate applies
        if let Some(w) = gate(&mut s, "rec_trim")? {
            return fail(lua, w);
        }
        match s.game.rec_keep_last(sec, name.as_deref()) {
            Ok(n) => one(text(lua, &n)?),
            Err(w) => fail(lua, &w),
        }
    })?;
    // gd.rec_save_history(name [, seconds]) -> name | false, why
    reg(e, gd, "rec_save_history", |lua, st, a| {
        let name = a.check_string(1)?;
        let sec = a.opt_number(2, 3600.0)?;
        let mut s = st.borrow_mut();
        if let Some(w) = gate(&mut s, "rec_save_history")? {
            return fail(lua, w);
        }
        if !name_ok(&name) {
            return fail(lua, "bad recording name");
        }
        match s.game.rec_save_history(&name, sec) {
            Ok(n) => one(text(lua, &n)?),
            Err(w) => fail(lua, &w),
        }
    })?;
    // gd.rec_export_slp(name [, outname]) -> path, frames | false, why
    reg(e, gd, "rec_export_slp", |lua, st, a| {
        let name = a.check_string(1)?;
        if !name_ok(&name) {
            return fail(lua, "bad recording name");
        }
        let out = if a.none_or_nil(2) { name.clone() } else { a.check_string(2)? };
        if !name_ok(&out) {
            return fail(lua, "bad output name");
        }
        let mut s = st.borrow_mut();
        match s.game.rec_export_slp(&name, &out) {
            Ok((p, n)) => Ok(MultiValue::from_iter([text(lua, &p)?, Value::Integer(n as i64)])),
            Err(w) => fail(lua, &w),
        }
    })?;
    reg(e, gd, "rec_status", |lua, st, _| {
        let r = st.borrow().game.rec_status();
        let t = lua.create_table()?;
        t.string("mode", r.mode)?;
        t.string("name", &r.name)?;
        t.string("last", &r.last)?;
        t.int("frames", r.frames as i64)?;
        t.int("frame", r.frame as i64)?;
        t.boolean("loop", r.looping)?;
        t.int("base", r.base as i64)?;
        Ok(t)
    })?;
    // gd.rec_list() -> {{name, frames, seconds, marks, complete, retail, bytes, format, whole, embedded, exportable, fighters, ok, why}, ...}
    reg(e, gd, "rec_list", |lua, st, _| {
        let rows = st.borrow().game.rec_list();
        let out = lua.create_table()?;
        for (i, r) in rows.iter().enumerate() {
            let t = lua.create_table()?;
            t.string("name", &r.name)?;
            t.int("bytes", r.bytes)?;
            if let Some(g) = &r.read {
                t.int("frames", g.frames as i64)?;
                t.num("seconds", g.frames as f64 / 60.0)?;
                t.int("marks", g.marks as i64)?;
                t.boolean("complete", g.complete)?;
                t.boolean("retail", g.retail)?;
                t.int("format", g.format as i64)?;
                t.boolean("whole", g.whole)?;
                t.boolean("embedded", g.embedded)?;
                t.boolean("exportable", g.exportable)?;
                t.string("fighters", &g.fighters)?;
            }
            t.boolean("ok", r.ok)?;
            t.string("why", &r.why)?;
            out.raw_set(i + 1, t)?;
        }
        Ok(out)
    })?;
    Ok(())
}
