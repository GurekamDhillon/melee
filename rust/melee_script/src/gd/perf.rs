//! `gd.perf([frames [, profiler]])` and `gd.prof(args)`: host diagnostics (gw_script.c `l_perf`, `l_prof`). No game
//! memory or rollback state is read or changed.
use super::{reg, TableExt};
use crate::engine::Engine;
use crate::host::{ProfStats, GW_PERF_HISTORY};
use mlua::{MultiValue, Result as LuaResult, Table, Value};

fn stats_row(lua: &mlua::Lua, s: &ProfStats) -> LuaResult<Table> {
    let t = lua.create_table()?;
    t.int("count", s.count as i64)?;
    t.num("mean", s.mean)?;
    t.num("p50", s.p50)?;
    t.num("p95", s.p95)?;
    t.num("p99", s.p99)?;
    t.num("max", s.max)?;
    Ok(t)
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    reg(e, gd, "perf", |lua, st, a| {
        let requested = (a.opt_integer(1, 120)?).clamp(1, GW_PERF_HISTORY as i64) as usize;
        let s = st.borrow();
        let snap = s.host.borrow_mut().perf_snapshot(requested);
        let t = lua.create_table()?;
        t.num("fps", snap.fps as f64)?;
        t.int("surface_load_calls", snap.surface[0] as i64)?;
        t.int("surface_fifo_commands", snap.surface[1] as i64)?;
        t.int("surface_selections", snap.surface[2] as i64)?;
        t.int("target", snap.target as i64)?;
        let frames = lua.create_table()?;
        for (i, f) in snap.frames.iter().enumerate() {
            let r = lua.create_table()?;
            r.num("total_ms", f.total_ms as f64)?;
            r.num("logic_ms", f.logic_ms as f64)?;
            r.num("gx_ms", f.gx_ms as f64)?;
            r.num("submit_ms", f.submit_ms as f64)?;
            r.num("worker_ms", f.worker_ms as f64)?;
            r.int("draw_calls", f.draw_calls as i64)?;
            r.int("vertices", f.vertices as i64)?;
            r.int("fx_particles", f.fx_particles as i64)?;
            frames.raw_set(i + 1, r)?;
        }
        t.set("frames", frames)?;
        let sh = lua.create_table()?;
        let p = &snap.shaders;
        sh.int("compiles", p.compiles)?;
        sh.int("cache_hits", p.cache_hits)?;
        sh.int("errors", p.errors)?;
        sh.int("reloads", p.reloads)?;
        sh.int("draws", p.draws)?;
        sh.int("passes", p.passes)?;
        sh.int("resolves", p.resolves)?;
        sh.int("skipped", p.skipped)?;
        sh.num("pixels", p.pixels)?;
        sh.num("uniform_bytes", p.uniform_bytes)?;
        sh.num("vertex_bytes", p.vertex_bytes)?;
        sh.num("record_ms", p.record_ms)?;
        t.set("shaders", sh)?;
        // the explicit cheap path: no profiler block
        if matches!(a.get(2), Value::Boolean(false)) {
            return Ok(t);
        }
        let prof = s.host.borrow_mut().prof_snapshot();
        let pt = lua.create_table()?;
        pt.boolean("enabled", prof.enabled)?;
        pt.int("frames", prof.frames as i64)?;
        pt.int("sample_window", 4096)?;
        pt.string("percentile_scope", "deterministic_run_reservoir")?;
        let zones = lua.create_table()?;
        for (n, v) in &prof.zones {
            zones.set(n.as_str(), stats_row(lua, v)?)?;
        }
        pt.set("zones", zones)?;
        let counters = lua.create_table()?;
        for (n, v) in &prof.counters {
            counters.set(n.as_str(), stats_row(lua, v)?)?;
        }
        pt.set("counters", counters)?;
        let details = lua.create_table()?;
        for (j, d) in prof.details.iter().enumerate() {
            let r = lua.create_table()?;
            r.string("zone", &d.zone)?;
            r.string("name", &d.name)?;
            r.int("detail", d.detail as i64)?;
            r.int("count", d.stats.count as i64)?;
            r.num("mean", d.stats.mean)?;
            r.num("max", d.stats.max)?;
            r.num("p50", d.stats.p50)?;
            r.num("p95", d.stats.p95)?;
            r.num("p99", d.stats.p99)?;
            if d.port_kind {
                r.int("port", (d.detail >> 16) as i64)?;
                r.int("kind", (d.detail & 65535) as i64)?;
            }
            details.raw_set(j + 1, r)?;
        }
        pt.set("details", details)?;
        t.set("profiler", pt)?;
        Ok(t)
    })?;
    reg(e, gd, "prof", |lua, st, a| {
        let cmd = a.check_string(1)?;
        let s = st.borrow();
        let (ok, msg) = s.host.borrow_mut().prof_command(&cmd);
        Ok(MultiValue::from_iter([Value::Boolean(ok), Value::String(lua.create_string(&msg)?)]))
    })?;
    Ok(())
}
