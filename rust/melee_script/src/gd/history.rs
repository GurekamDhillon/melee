//! The Geno Lab's history and state library: `gd.history`, `step_back`, `rewind_to`, `rewind_live`, `rewind_test(_result)`,
//! `hot_reload(_status)`, `state_save/list/gen/load/delete/rename` and `kb_preview` (gw_script.c).
use super::{lua_err, reg, slot_arg, TableExt};
use crate::args::Args;
use crate::engine::{Engine, Shared, State};
use crate::game::*;
use crate::gd::labmisc::c as lc;
use crate::gd::savestate::SaveSlot;
use crate::rewind::{slot_from_bytes, slot_to_bytes, GS_RW_INTERVAL, GS_RW_MAX_FRAMES, SLOT_BYTES};
use mlua::{Lua, MultiValue, Result as LuaResult, Table, Value};
use std::path::{Path, PathBuf};

pub const GS_ST_MAGIC: &[u8; 8] = b"GDLABHD1";
pub const GENO_VERSION: u32 = 11;
pub const HDR_BYTES: usize = 8 + 4 + 4 + 32 + 8 + 96 + 8 + 64 + 96 + SLOT_BYTES;

/// `GsStateHdr`
#[derive(Clone, Debug, Default)]
pub struct StateHdr {
    pub geno_version: u32,
    pub ident: [u64; 4],
    pub match_frame: i32,
    pub stage: i32,
    pub present: [i32; 6],
    pub chr: [i32; 6],
    pub costume: [i32; 6],
    pub cpu: [i32; 6],
    pub unix_time: i64,
    pub name: String,
    pub desc: String,
    pub slot: SaveSlot,
}

fn put_str(v: &mut Vec<u8>, s: &str, cap: usize) {
    let mut b = s.as_bytes().to_vec();
    b.truncate(cap - 1);
    b.resize(cap, 0);
    v.extend_from_slice(&b);
}
fn get_str(b: &[u8]) -> String {
    let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
    String::from_utf8_lossy(&b[..end]).into_owned()
}

impl StateHdr {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(HDR_BYTES);
        v.extend_from_slice(GS_ST_MAGIC);
        v.extend_from_slice(&(HDR_BYTES as u32).to_le_bytes());
        v.extend_from_slice(&self.geno_version.to_le_bytes());
        for h in self.ident {
            v.extend_from_slice(&h.to_le_bytes());
        }
        v.extend_from_slice(&self.match_frame.to_le_bytes());
        v.extend_from_slice(&self.stage.to_le_bytes());
        for arr in [&self.present, &self.chr, &self.costume, &self.cpu] {
            for x in arr.iter() {
                v.extend_from_slice(&x.to_le_bytes());
            }
        }
        v.extend_from_slice(&self.unix_time.to_le_bytes());
        put_str(&mut v, &self.name, 64);
        put_str(&mut v, &self.desc, 96);
        v.extend_from_slice(&slot_to_bytes(&self.slot));
        debug_assert_eq!(v.len(), HDR_BYTES);
        v
    }
    /// None when the bytes are not a header this build can read (`gs_state_check`'s first test)
    pub fn from_bytes(b: &[u8]) -> Option<StateHdr> {
        if b.len() != HDR_BYTES || &b[..8] != GS_ST_MAGIC || u32::from_le_bytes(b[8..12].try_into().unwrap()) as usize != HDR_BYTES {
            return None;
        }
        let u32_at = |o: usize| u32::from_le_bytes(b[o..o + 4].try_into().unwrap());
        let i32_at = |o: usize| i32::from_le_bytes(b[o..o + 4].try_into().unwrap());
        let u64_at = |o: usize| u64::from_le_bytes(b[o..o + 8].try_into().unwrap());
        let mut h = StateHdr { geno_version: u32_at(12), ..Default::default() };
        for k in 0..4 {
            h.ident[k] = u64_at(16 + 8 * k);
        }
        h.match_frame = i32_at(48);
        h.stage = i32_at(52);
        let mut o = 56;
        for arr in [&mut h.present, &mut h.chr, &mut h.costume, &mut h.cpu] {
            for x in arr.iter_mut() {
                *x = i32_at(o);
                o += 4;
            }
        }
        h.unix_time = i64::from_le_bytes(b[o..o + 8].try_into().unwrap());
        o += 8;
        h.name = get_str(&b[o..o + 64]);
        o += 64;
        h.desc = get_str(&b[o..o + 96]);
        o += 96;
        h.slot = slot_from_bytes(&b[o..])?;
        Some(h)
    }
}

pub fn state_file_ok(f: &str) -> bool {
    let n = f.len();
    if !(6..=64).contains(&n) || !f.to_ascii_lowercase().ends_with(".gdst") {
        return false;
    }
    f.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-' || c == b'.')
}

impl State {
    /// `gs_states_dir`: `<data>/<script id with / as _>/states`, created
    pub fn states_dir(&self) -> PathBuf {
        let id = match self.cur {
            Some(i) if i != self.console && i < self.scripts.len() => self.scripts[i].id.clone(),
            _ => "geno-lab/lab".to_string(),
        };
        let dir = self.data_dir.join(id.replace('/', "_")).join("states");
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    /// `gs_state_fill`
    pub fn state_fill(&self, name: &str, desc: &str) -> StateHdr {
        let mut h = StateHdr { geno_version: GENO_VERSION, ident: self.game.identity(), match_frame: self.match_frame, stage: self.game.stage_kind(), ..Default::default() };
        for s in 0..6 {
            let present = players_present(&*self.game, s);
            h.present[s] = present as i32;
            h.chr[s] = if present { self.game.fighter_i(s as i32, SI_CHAR) } else { -1 };
            h.costume[s] = if present { self.game.fighter_i(s as i32, SI_COSTUME) } else { -1 };
            h.cpu[s] = if present { self.game.fighter_i(s as i32, SI_SLOT_TYPE) } else { -1 };
        }
        h.unix_time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
        h.name = name.to_string();
        h.desc = desc.to_string();
        h.slot = SaveSlot {
            used: true,
            scene_epoch: self.scene_epoch,
            frame: self.frame,
            match_frame: self.match_frame,
            last_action: self.last_action,
            state_frame: self.state_frame,
        };
        h
    }

    /// `gs_state_check`: Err(why) when this state would be refused now
    pub fn state_check(&self, hdr: Option<&StateHdr>) -> Result<(), String> {
        let Some(h) = hdr else { return Err("not a state this Lab can read".into()) };
        if self.game.stage_slots_active() {
            return Err("state crosses a stage slot/cache session; loading is refused".into());
        }
        let id = self.game.identity();
        if h.ident[0] != id[0] {
            return Err("saved by another build of the game".into());
        }
        if h.ident[1] != id[1] {
            return Err("saved with another disc".into());
        }
        if h.ident[2] != id[2] {
            return Err("saved with other mods".into());
        }
        if h.ident[3] != id[3] || h.geno_version != GENO_VERSION {
            return Err("saved with other Geno fighter data".into());
        }
        if !self.match_active {
            return Err("load it during a match".into());
        }
        for s in 0..6 {
            let present = players_present(&*self.game, s) as i32;
            if present != h.present[s]
                || (present != 0
                    && (self.game.fighter_i(s as i32, SI_CHAR) != h.chr[s] || self.game.fighter_i(s as i32, SI_COSTUME) != h.costume[s]))
            {
                let w = if h.desc.is_empty() { "another match" } else { &h.desc };
                return Err(format!("this state is {w}: start that match first"));
            }
        }
        if self.game.stage_kind() != h.stage {
            let w = if h.desc.is_empty() { "another stage" } else { &h.desc };
            return Err(format!("this state is {w}: start that match first"));
        }
        Ok(())
    }

    pub fn read_state_hdr(&self, path: &Path) -> Option<StateHdr> {
        let b = self.game.snap_file_header(&path.to_string_lossy(), HDR_BYTES)?;
        StateHdr::from_bytes(&b)
    }

    /// `gs_state_index`: index.txt beside the states (file | frame | saved | name | what, tab separated)
    pub fn state_index(&self, dir: &Path) {
        let mut out = String::from("# the Geno Lab's saved states (the .gdst files are the truth; this list is rewritten)\n");
        if let Ok(rd) = std::fs::read_dir(dir) {
            let mut names: Vec<String> = rd.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().into_owned()).collect();
            names.sort();
            for n in names {
                if !n.to_ascii_lowercase().ends_with(".gdst") {
                    continue;
                }
                if let Some(h) = self.read_state_hdr(&dir.join(&n)) {
                    let when = self.host.borrow().format_time(h.unix_time);
                    out.push_str(&format!("{n}\tf{}\t{when}\t{}\t{}\n", h.match_frame, h.name, h.desc));
                }
            }
        }
        let _ = std::fs::write(dir.join("index.txt"), out);
    }
}

fn fail(lua: &Lua, why: &str) -> LuaResult<MultiValue> {
    Ok(MultiValue::from_iter([Value::Boolean(false), Value::String(lua.create_string(why)?)]))
}

fn present_arg(st: &Shared, a: &Args, i: usize) -> LuaResult<i32> {
    let slot = slot_arg(a, i)?;
    Ok(if players_present(&*st.borrow().game, slot) { slot as i32 } else { -1 })
}

fn opt_field(t: &Table, k: &str, def: f64) -> f64 {
    match t.get::<Value>(k) {
        Ok(Value::Number(n)) => n,
        Ok(Value::Integer(i)) => i as f64,
        Ok(Value::String(s)) => s.to_str().ok().and_then(|s| s.trim().parse::<f64>().ok()).unwrap_or(def),
        _ => def,
    }
}

/// the trajectory DI of ftCo_8008E5A4 for a stick (sx, sy) on a knockback velocity
fn apply_di(vx: &mut f32, vy: &mut f32, sx: f32, sy: f32, di_deg: f32) {
    let (kx, ky) = (*vx, *vy);
    let mag2 = kx * kx + ky * ky;
    if (sx == 0.0 && sy == 0.0) || mag2 < 0.00001 {
        return;
    }
    let f3 = ky * sx + (-kx) * sy;
    let mut f30 = f3 * f3 / mag2;
    if kx * sy - ky * sx < 0.0 {
        f30 = -f30;
    }
    let a = ky.atan2(kx) + di_deg * 3.14159265f32 / 180.0 * f30;
    let mag = mag2.sqrt();
    *vx = mag * a.cos();
    *vy = mag * a.sin();
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    // gd.history([frames [, interval]])
    reg(e, gd, "history", |lua, st, a| {
        let mut s = st.borrow_mut();
        if !a.none_or_nil(1) {
            let mut want = a.check_integer(1)? as i32;
            let def = if s.rw.interval > 0 { s.rw.interval } else { GS_RW_INTERVAL };
            let mut iv = a.opt_integer(2, def as i64)? as i32;
            s.require_offline("history")?;
            if want > 0 && s.game.onep_state_active() {
                return Err(lua_err("retail 1P state active: snapshot/rewind unavailable"));
            }
            if want > 0 && s.game.mode_active() {
                return Err(lua_err("mode state active: Lua director cannot resimulate"));
            }
            if want > 0 && s.game.stage_slots_active() {
                return Err(lua_err("stage slot session active: queue/cache cannot resimulate"));
            }
            want = want.clamp(0, GS_RW_MAX_FRAMES);
            iv = iv.clamp(1, 600);
            s.rw.interval = iv;
            if want == 0 {
                s.rw_stop();
            }
            s.rw.frames = want;
        }
        let now = s.ring_now();
        let g = &s.game;
        let rw = &s.rw;
        let t = lua.create_table()?;
        t.int("depth", rw.frames as i64)?;
        t.num("seconds", rw.frames as f64 / 60.0)?;
        t.int("interval", (if rw.interval > 0 { rw.interval } else { GS_RW_INTERVAL }) as i64)?;
        t.int("now", now as i64)?;
        let base = g.rw_base_tag();
        t.int("back", if rw.began && now > base { (now - base) as i64 } else { 0 })?;
        t.int("fwd", if rw.began && rw.head > now { (rw.head - now) as i64 } else { 0 })?;
        t.int("head", (if rw.began { rw.head } else { now }) as i64)?;
        t.int("oldest", (if rw.began { base } else { now }) as i64)?;
        t.int("keys", g.rw_count() as i64)?;
        t.num("mb", g.rw_bytes() / 1048576.0)?;
        t.num("delta_mb", g.rw_delta_bytes() / 1048576.0)?;
        t.num("slot_mb", g.snap_slot_bytes() as f64 / 1048576.0)?;
        t.num("key_ms", g.rw_ms_save_avg())?;
        t.num("last_ms", rw.last_ms)?;
        t.num("last_load_ms", rw.last_load_ms)?;
        t.int("last_frames", rw.last_frames as i64)?;
        t.int("last_pages", g.rw_last_load_pages() as i64)?;
        t.boolean("replaying", rw.began && now < rw.head)?;
        t.boolean("busy", rw.resim > 0 || rw.resim_run > 0 || rw.pending_on)?;
        Ok(t)
    })?;
    reg(e, gd, "step_back", |lua, st, a| {
        let mut n = a.opt_integer(1, 1)? as i32;
        let mut s = st.borrow_mut();
        s.require_offline("step_back")?;
        if n < 1 {
            n = 1;
        }
        let target = s.ring_now() - n;
        match s.rw_request(target) {
            Err(why) => fail(lua, why),
            Ok(()) => Ok(MultiValue::from_iter([Value::Boolean(true)])),
        }
    })?;
    reg(e, gd, "rewind_to", |lua, st, a| {
        let mut s = st.borrow_mut();
        if s.game.onep_state_active() {
            return Err(lua_err("retail 1P state active: snapshot/rewind unavailable"));
        }
        let f = a.check_integer(1)? as i32;
        s.require_offline("rewind_to")?;
        match s.rw_request(f) {
            Err(why) => fail(lua, why),
            Ok(()) => Ok(MultiValue::from_iter([Value::Boolean(true)])),
        }
    })?;
    reg(e, gd, "rewind_live", |_, st, _| {
        let mut s = st.borrow_mut();
        if s.game.onep_state_active() {
            return Err(lua_err("retail 1P state active: snapshot/rewind unavailable"));
        }
        s.require_offline("rewind_live")?;
        s.rw_branch();
        Ok(true)
    })?;
    reg(e, gd, "rewind_test", |lua, st, a| {
        let mut s = st.borrow_mut();
        if s.game.onep_state_active() {
            return Err(lua_err("retail 1P state active: snapshot/rewind unavailable"));
        }
        let n = a.opt_integer(1, 90)? as i32;
        s.require_offline("rewind_test")?;
        if s.game.stage_slots_active() {
            return s.stage_state_refuse(lua, "rewind_test");
        }
        if s.game.mode_active() {
            return fail(lua, "mode state active: Lua director cannot resimulate");
        }
        if !s.rw.began || s.rw.frames <= 0 {
            return fail(lua, "history is off (gd.history(n) turns it on)");
        }
        if n < 1 || n > s.rw.frames - s.rw.interval {
            return fail(lua, "frames: 1 .. the history window minus one keyframe interval");
        }
        s.rw.rwt_phase = 1;
        s.rw.rwt_back = n;
        s.rw.rwt_keep = a.truthy(2);
        s.rw.rwt_arm = 0;
        s.rw.rwt_pass = -1;
        s.rw.rwt_text = "running".into();
        s.paused = false;
        s.step = 0;
        Ok(MultiValue::from_iter([Value::Boolean(true)]))
    })?;
    reg(e, gd, "rewind_test_result", |lua, st, _| {
        let s = st.borrow();
        let t = lua.create_table()?;
        t.int("phase", s.rw.rwt_phase as i64)?;
        if s.rw.rwt_pass >= 0 {
            t.boolean("pass", s.rw.rwt_pass != 0)?;
        }
        t.num("diff", s.rw.rwt_diff[0])?;
        t.num("diff_compared", s.rw.rwt_diff[1])?;
        t.string("text", &s.rw.rwt_text)?;
        Ok(t)
    })?;
    reg(e, gd, "hot_reload", |lua, st, a| {
        let sec = a.opt_number(1, 2.0)?;
        let mut s = st.borrow_mut();
        let now = s.ring_now();
        s.require_offline("hot_reload")?;
        if s.game.stage_slots_active() {
            return s.stage_state_refuse(lua, "hot_reload");
        }
        if !s.game.geno_lab_mode_active() {
            return fail(lua, "hot reload requires a LAB match");
        }
        if s.game.mode_active() {
            return fail(lua, "mode state active: Lua director cannot resimulate");
        }
        if s.rw.hot_phase != 0 {
            return fail(lua, "a reload is already running");
        }
        if !s.match_active {
            return fail(lua, "not in a match");
        }
        let mut start = now - (sec * 60.0 + 0.5) as i32;
        if !s.rw.began || sec <= 0.0 {
            start = now; // no history: reload in place
            s.rw.hot_start = now;
            s.rw.hot_phase = 2;
            s.paused = true;
            s.step = 0;
        } else {
            start = start.max(s.game.rw_base_tag()).max(s.rw.log_start);
            if let Err(why) = s.rw_request(start) {
                return fail(lua, why);
            }
            s.rw.hot_start = start;
            s.rw.hot_phase = 1;
        }
        s.rw.hot_text = "reloading".into();
        Ok(MultiValue::from_iter([Value::Boolean(true), Value::Integer(start as i64)]))
    })?;
    reg(e, gd, "hot_reload_status", |lua, st, _| {
        let s = st.borrow();
        let t = lua.create_table()?;
        t.int("phase", s.rw.hot_phase as i64)?;
        t.boolean("ok", s.rw.hot_ok)?;
        t.string("text", &s.rw.hot_text)?;
        Ok(t)
    })?;

    // ---- the persistent state library
    reg(e, gd, "state_save", |lua, st, a| {
        let mut s = st.borrow_mut();
        if s.game.onep_state_active() {
            return Err(lua_err("retail 1P state active: snapshot/rewind unavailable"));
        }
        let name = a.opt_string(1, "")?;
        let desc = a.opt_string(2, "")?;
        s.require_offline("state_save")?;
        if s.game.stage_slots_active() {
            return s.stage_state_refuse(lua, "state_save");
        }
        if !s.match_active {
            return fail(lua, "not in a match");
        }
        if !s.rw.st_pending_save.is_empty() {
            return fail(lua, "a save is already waiting");
        }
        let dir = s.states_dir();
        s.rw.st_counter += 1;
        let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let file = format!("st_{secs}_{:03}.gdst", s.rw.st_counter % 1000);
        s.rw.st_pending_save = dir.join(&file).to_string_lossy().into_owned();
        s.rw.st_save_name = name;
        s.rw.st_save_desc = desc;
        Ok(MultiValue::from_iter([Value::String(lua.create_string(&file)?)]))
    })?;
    reg(e, gd, "state_gen", |_, st, _| Ok(st.borrow().rw.st_gen as i64))?;
    reg(e, gd, "state_list", |lua, st, _| {
        let s = st.borrow();
        let dir = s.states_dir();
        let mut rows: Vec<(String, Option<StateHdr>, Result<(), String>)> = Vec::new();
        if let Ok(rd) = std::fs::read_dir(&dir) {
            let mut names: Vec<String> = rd.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().into_owned()).collect();
            names.sort();
            for n in names {
                if !n.to_ascii_lowercase().ends_with(".gdst") || !state_file_ok(&n) {
                    continue;
                }
                let h = s.read_state_hdr(&dir.join(&n));
                let ok = s.state_check(h.as_ref());
                rows.push((n, h, ok));
                if rows.len() >= 512 {
                    break;
                }
            }
        }
        rows.sort_by(|x, y| {
            let (tx, ty) = (x.1.as_ref().map(|h| h.unix_time).unwrap_or(0), y.1.as_ref().map(|h| h.unix_time).unwrap_or(0));
            ty.cmp(&tx).then_with(|| y.0.cmp(&x.0))
        });
        let out = lua.create_table()?;
        for (i, (file, h, ok)) in rows.iter().enumerate() {
            let h = h.clone().unwrap_or_default();
            let r = lua.create_table()?;
            r.string("file", file)?;
            r.string("name", &h.name)?;
            r.string("what", &h.desc)?;
            r.int("frame", h.match_frame as i64)?;
            r.int("time", h.unix_time)?;
            r.string("saved", &s.host.borrow().format_time(h.unix_time))?;
            r.int("stage", h.stage as i64)?;
            r.boolean("ok", ok.is_ok())?;
            r.string("why", ok.as_ref().err().map(|s| s.as_str()).unwrap_or(""))?;
            let fighters = lua.create_table()?;
            let mut k = 0;
            for sl in 0..6 {
                if h.present[sl] == 0 {
                    continue;
                }
                let f = lua.create_table()?;
                f.int("port", sl as i64 + 1)?;
                f.int("char", h.chr[sl] as i64)?;
                f.string("char_name", &crate::gd::players::char_name(&*s.game, h.chr[sl]))?;
                f.int("costume", h.costume[sl] as i64)?;
                f.boolean("cpu", h.cpu[sl] == 1)?;
                k += 1;
                fighters.raw_set(k, f)?;
            }
            r.set("fighters", fighters)?;
            out.raw_set(i + 1, r)?;
        }
        Ok(out)
    })?;
    reg(e, gd, "state_load", |lua, st, a| {
        let mut s = st.borrow_mut();
        if s.game.onep_state_active() {
            return Err(lua_err("retail 1P state active: snapshot/rewind unavailable"));
        }
        let file = a.check_string(1)?;
        s.require_offline("state_load")?;
        if s.game.stage_slots_active() {
            return s.stage_state_refuse(lua, "state_load");
        }
        if !state_file_ok(&file) {
            return fail(lua, "not a state file name");
        }
        let path = s.states_dir().join(&file);
        if s.game.snap_file_header(&path.to_string_lossy(), HDR_BYTES).is_none() {
            return fail(lua, "no such state");
        }
        let h = s.read_state_hdr(&path);
        if let Err(why) = s.state_check(h.as_ref()) {
            return fail(lua, &why);
        }
        s.rw.st_pending_load = path.to_string_lossy().into_owned();
        Ok(MultiValue::from_iter([Value::Boolean(true)]))
    })?;
    reg(e, gd, "state_delete", |lua, st, a| {
        let file = a.check_string(1)?;
        let mut s = st.borrow_mut();
        s.require_offline("state_delete")?;
        if !state_file_ok(&file) {
            return fail(lua, "not a state file name");
        }
        let dir = s.states_dir();
        if std::fs::remove_file(dir.join(&file)).is_err() {
            return fail(lua, "could not delete it");
        }
        s.rw.st_gen += 1;
        s.state_index(&dir);
        Ok(MultiValue::from_iter([Value::Boolean(true)]))
    })?;
    reg(e, gd, "state_rename", |lua, st, a| {
        let file = a.check_string(1)?;
        let name = a.check_string(2)?;
        let mut s = st.borrow_mut();
        s.require_offline("state_rename")?;
        if !state_file_ok(&file) {
            return fail(lua, "not a state file name");
        }
        let dir = s.states_dir();
        let path = dir.join(&file);
        let Some(mut h) = s.read_state_hdr(&path) else { return fail(lua, "not a state this Lab can read") };
        h.name = name;
        if s.game.snap_file_rewrite_header(&path.to_string_lossy(), &h.to_bytes()) != 0 {
            return fail(lua, "could not rename it");
        }
        s.rw.st_gen += 1;
        s.state_index(&dir);
        Ok(MultiValue::from_iter([Value::Boolean(true)]))
    })?;

    // gd.kb_preview(victim_port, {...}) -> the knockback trajectory
    reg(e, gd, "kb_preview", |lua, st, a| {
        let slot = present_arg(st, &a, 1)?;
        let opts = a.check_table(2)?;
        st.borrow_mut().require_offline("kb_preview")?;
        if slot < 0 {
            return Ok(MultiValue::from_iter([Value::Nil, Value::String(lua.create_string("no fighter on that port")?)]));
        }
        let s = st.borrow();
        let g = &*s.game;
        let mut att = -1;
        if let Ok(Value::Integer(_)) = opts.get::<Value>("attacker") {
            let at = opts.get::<i64>("attacker")?;
            if !(1..=6).contains(&at) {
                return Err(lua_err(format!("player/port numbers are 1-6 (got {})", at as i32)));
            }
            if players_present(g, (at - 1) as usize) {
                att = (at - 1) as i32;
            }
        }
        let dmg = opt_field(&opts, "damage", 10.0) as f32;
        let angle = opt_field(&opts, "angle", 45.0) as i32;
        let kbg = opt_field(&opts, "kbg", 100.0) as i32;
        let bkb = opt_field(&opts, "bkb", 0.0) as i32;
        let wbk = opt_field(&opts, "wbk", 0.0) as i32;
        let pct = opt_field(&opts, "percent", -1.0) as f32;
        let extra = opt_field(&opts, "extra", 90.0) as i32;
        let mut px = opt_field(&opts, "x", g.fighter_f(slot, SF_X) as f64) as f32;
        let mut py = opt_field(&opts, "y", g.fighter_f(slot, SF_Y) as f64) as f32;
        let dir = {
            let mut def = -g.lab_flight_f(slot, 3);
            if att >= 0 {
                def = if g.fighter_f(slot, SF_X) >= g.fighter_f(att, SF_X) { 1.0 } else { -1.0 };
            }
            if opt_field(&opts, "dir", def as f64) as f32 >= 0.0 { 1.0f32 } else { -1.0f32 }
        };
        let kb = g.lab_knockback(slot, att, dmg.to_bits() as i32, kbg, wbk, bkb, pct.to_bits() as i32, 1);
        let level = g.lab_kb_level(kb.to_bits() as i32);
        let mut hitstun = (kb * g.lab_common_f(lc::HITSTUN_MUL)) as i32;
        if hitstun < 1 {
            hitstun = 1;
        }
        let pi = 3.14159265f32;
        let a_ang = if angle != 361 {
            angle as f32 * pi / 180.0
        } else if g.lab_flight_f(slot, 4) == 0.0 {
            g.lab_common_f(lc::ANGLE_AIR_361)
        } else if kb < g.lab_common_f(lc::ANGLE_GROUND_KB0) {
            0.0
        } else {
            let k0 = g.lab_common_f(lc::ANGLE_GROUND_KB0);
            let k1 = g.lab_common_f(lc::ANGLE_GROUND_KB1);
            let mx = g.lab_common_f(lc::ANGLE_GROUND_MAX);
            let mut a = (mx * ((kb - k0) / (k1 - k0)) + 1.0) * pi / 180.0;
            if a > mx * pi / 180.0 {
                a = mx * pi / 180.0;
            }
            a
        };
        let mut vx = kb * g.lab_common_f(lc::KB_SPEED) * a_ang.cos() * dir;
        let mut vy = kb * g.lab_common_f(lc::KB_SPEED) * a_ang.sin();
        let (mut sx, mut sy) = (0.0f32, 0.0f32);
        let mut di = "none".to_string();
        match opts.get::<Value>("di")? {
            Value::Table(t) => {
                di = "stick".into();
                sx = opt_field(&t, "x", 0.0) as f32;
                sy = opt_field(&t, "y", 0.0) as f32;
            }
            Value::String(v) => di = v.to_str()?.to_string(),
            _ => {}
        }
        {
            let t = vy.atan2(vx);
            let (ccx, ccy) = (-t.sin(), t.cos());
            let di_deg = g.lab_common_f(lc::DI_DEGREES);
            if di == "in" || di == "out" {
                let toward = (ccx * dir < 0.0) == (di == "in");
                sx = if toward { ccx } else { -ccx };
                sy = if toward { ccy } else { -ccy };
            } else if di == "survival" {
                let (mut ax, mut ay, mut bx, mut by) = (vx, vy, vx, vy);
                let goal = if dir > 0.0 { 0.785398f32 } else { 2.356194f32 };
                apply_di(&mut ax, &mut ay, ccx, ccy, di_deg);
                apply_di(&mut bx, &mut by, -ccx, -ccy, di_deg);
                if (ay.atan2(ax) - goal).abs() <= (by.atan2(bx) - goal).abs() {
                    sx = ccx;
                    sy = ccy;
                } else {
                    sx = -ccx;
                    sy = -ccy;
                }
            }
            apply_di(&mut vx, &mut vy, sx, sy, di_deg);
        }
        let a_di = vy.atan2(vx);
        let (grav, term, fric) = (g.lab_flight_f(slot, 0), g.lab_flight_f(slot, 1), g.lab_flight_f(slot, 2));
        let decay = g.lab_common_f(lc::KB_DECAY);
        let (left, right, top, bottom) =
            (g.lab_common_f(lc::BLAST_LEFT), g.lab_common_f(lc::BLAST_RIGHT), g.lab_common_f(lc::BLAST_TOP), g.lab_common_f(lc::BLAST_BOTTOM));
        let t = lua.create_table()?;
        t.num("kb", kb as f64)?;
        t.int("level", level as i64)?;
        t.boolean("tumble", level >= 3)?;
        t.int("hitstun", hitstun as i64)?;
        t.num("angle", a_ang as f64 * 180.0 / 3.14159265)?;
        t.num("angle_di", a_di as f64 * 180.0 / 3.14159265)?;
        t.num("vx", vx as f64)?;
        t.num("vy", vy as f64)?;
        t.string("di", &di)?;
        t.num("percent", (if pct >= 0.0 { pct } else { g.lab_flight_f(slot, 6) }) as f64)?;
        t.num("weight", g.lab_flight_f(slot, 5) as f64)?;
        let z = lua.create_table()?;
        z.num("left", left as f64)?;
        z.num("right", right as f64)?;
        z.num("top", top as f64)?;
        z.num("bottom", bottom as f64)?;
        t.set("zones", z)?;
        let points = lua.create_table()?;
        let (mut svx, mut svy) = (0.0f32, 0.0f32);
        let mut npts = 0;
        let mut f = 1;
        while f <= hitstun + extra && f <= 1200 {
            svy -= grav;
            if svy < -term {
                svy = -term;
            }
            if svx > 0.0 {
                svx -= fric;
                if svx < 0.0 {
                    svx = 0.0;
                }
            } else if svx < 0.0 {
                svx += fric;
                if svx > 0.0 {
                    svx = 0.0;
                }
            }
            let m = (vx * vx + vy * vy).sqrt();
            if m < decay {
                vx = 0.0;
                vy = 0.0;
            } else {
                let th = vy.atan2(vx);
                vx -= decay * th.cos();
                vy -= decay * th.sin();
            }
            px += svx + vx;
            py += svy + vy;
            if f <= hitstun {
                let p = lua.create_table()?;
                p.raw_set(1, px as f64)?;
                p.raw_set(2, py as f64)?;
                npts += 1;
                points.raw_set(npts, p)?;
            }
            if px < left || px > right || py > top || py < bottom {
                t.set("points", points)?;
                let b = lua.create_table()?;
                b.num("x", px as f64)?;
                b.num("y", py as f64)?;
                b.int("frame", f as i64)?;
                b.string("side", if px < left { "left" } else if px > right { "right" } else if py > top { "top" } else { "bottom" })?;
                b.boolean("after_hitstun", f > hitstun)?;
                t.set("blast", b)?;
                return Ok(MultiValue::from_iter([Value::Table(t)]));
            }
            if f >= hitstun && vx == 0.0 && vy == 0.0 {
                break;
            }
            f += 1;
        }
        t.set("points", points)?;
        Ok(MultiValue::from_iter([Value::Table(t)]))
    })?;
    Ok(())
}
