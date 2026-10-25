//! Zones (`gw_script_zones.inc`): zone_add, zone_set, zone_remove, zones, point_zones, zones_at, zone_members and the
//! enter/exit/none/some hook dispatch. The zone records live in the game half (`zones_core::SzState` behind the scalar
//! `ScriptGame::zone_*` accessors); this is the Lua adapter. Reads never branch the rewind timeline.
use super::caps::{neg1, one};
use super::{lua_err, reg, TableExt};
use crate::args::Args;
use crate::engine::{Engine, Shared, State};
use crate::events::{spine_push, GsEvent};
use crate::game::ScriptGame;
use crate::zones_core::*;
use mlua::{MultiValue, Table, Value};

type LuaResult<T> = mlua::Result<T>;

#[derive(Default)]
pub struct ZoneAdapter {
    pub dispatching: bool,
    /// monotonic allocation watermark; not simulation input
    pub serial: i32,
    pub cleanup: bool,
    pub pending_release: i32,
}

pub const ZONE_HOOKS: [&str; 4] = ["on_zone_enter", "on_zone_exit", "on_zone_none", "on_zone_some"];

fn fl(b: i32) -> f32 {
    f32::from_bits(b as u32)
}
fn bits(f: f32) -> i32 {
    f.to_bits() as i32
}
/// `gs_zone_read_text`
fn read_text(g: &dyn ScriptGame, slot: i32, text: i32, cap: usize, event: bool) -> String {
    let mut out = Vec::new();
    for i in 0..cap - 1 {
        let c = if event { g.zone_event_text(slot, text, i as i32) } else { g.zone_text(slot, text, i as i32) } as u8;
        if c == 0 {
            break;
        }
        out.push(c);
    }
    String::from_utf8_lossy(&out).to_string()
}
/// `gs_zone_read`
pub fn zone_read(g: &dyn ScriptGame, slot: i32) -> SzDef {
    let mut z = SzDef {
        handle: g.zone_field(slot, SZF_HANDLE),
        owner: g.zone_field(slot, SZF_OWNER),
        live: g.zone_field(slot, SZF_LIVE),
        model: g.zone_field(slot, SZF_MODEL),
        n: g.zone_field(slot, SZF_N).clamp(0, SZ_VERTICES as i32) as usize,
        ntags: g.zone_field(slot, SZF_TAGS).clamp(0, 8) as usize,
        area: fl(g.zone_field(slot, SZF_AREA)),
        dx: fl(g.zone_field(slot, SZF_DX)),
        dy: fl(g.zone_field(slot, SZF_DY)),
        ..Default::default()
    };
    z.name = read_text(g, slot, SZT_NAME, 81, false);
    z.label = read_text(g, slot, SZT_LABEL, 81, false);
    z.kind = read_text(g, slot, SZT_KIND, 32, false);
    for i in 0..z.ntags {
        z.tags[i] = read_text(g, slot, SZT_TAG + i as i32, 32, false);
    }
    for i in 0..z.n {
        z.x[i] = fl(g.zone_field(slot, SZF_X + i as i32));
        z.y[i] = fl(g.zone_field(slot, SZF_Y + i as i32));
    }
    z
}
/// `gs_zone_slot`
fn zone_slot(g: &dyn ScriptGame, handle: i32) -> i32 {
    (0..SZ_CAP as i32).find(|&i| g.zone_field(i, SZF_LIVE) == 1 && g.zone_field(i, SZF_HANDLE) == handle).unwrap_or(-1)
}
fn zone_push(lua: &mlua::Lua, z: &SzDef, frames: i32) -> LuaResult<Table> {
    let t = lua.create_table()?;
    t.int("handle", z.handle as i64)?;
    t.string("name", &z.name)?;
    t.string("label", &z.label)?;
    t.string("kind", &z.kind)?;
    t.int("owner", z.owner as i64)?;
    t.num("area", z.area as f64)?;
    if z.model != 0 {
        t.int("model", z.model as i64)?;
    }
    if frames >= 0 {
        t.int("frames", frames as i64)?;
        t.num("seconds", frames as f64 / 60.0)?;
    }
    let tags = lua.create_table()?;
    for i in 0..z.ntags {
        tags.raw_set(i as i64 + 1, z.tags[i].as_str())?;
    }
    t.set("tags", tags)?;
    let pts = lua.create_table()?;
    for i in 0..z.n {
        let p = lua.create_table()?;
        p.raw_set(1, z.x[i] as f64)?;
        p.raw_set(2, z.y[i] as f64)?;
        pts.raw_set(i as i64 + 1, p)?;
    }
    t.set("points", pts)?;
    t.num("offset_x", z.dx as f64)?;
    t.num("offset_y", z.dy as f64)?;
    Ok(t)
}
/// `gs_zone_string`
fn zone_string(t: &Table, field: &str, dst: &mut String, cap: usize) -> LuaResult<()> {
    let v: Value = t.get(field)?;
    match v {
        Value::Nil => {}
        Value::String(s) => {
            let b = s.as_bytes();
            if b.is_empty() || b.len() >= cap || b.contains(&0) {
                return Err(lua_err(format!("zone {field} needs 1..{} bytes without NUL", cap - 1)));
            }
            *dst = String::from_utf8_lossy(&b).to_string();
        }
        _ => return Err(lua_err(format!("zone {field} must be text"))),
    }
    Ok(())
}
fn zone_handle(a: &Args, i: usize) -> LuaResult<i32> {
    let h = a.check_integer(i)?;
    if h < 1 || h > i32::MAX as i64 {
        return Err(a.err(i, "positive zone handle required"));
    }
    Ok(h as i32)
}
/// `gs_zone_array`: only consecutive integer keys 1..n
fn zone_array(t: &Table, n: usize) -> LuaResult<()> {
    for kv in t.clone().pairs::<Value, Value>() {
        let (k, _) = kv?;
        match k {
            Value::Integer(k) if k >= 1 && k as usize <= n => {}
            _ => return Err(lua_err("zone arrays require consecutive integer keys")),
        }
    }
    Ok(())
}
fn num_at(name: &'static str, v: &Value) -> LuaResult<f64> {
    neg1(one(name, v).check_number(1))
}

fn zone_write(st: &Shared, name: &'static str, handle: i32, at: &Args, ti: usize) -> LuaResult<Value> {
    let mut s = st.borrow_mut();
    s.require_offline(name)?;
    let t = at.check_table(ti)?;
    if s.zone.cleanup {
        return Err(lua_err("zone definitions cannot change during resource teardown"));
    }
    if !s.match_active {
        return Err(lua_err("zones require an active offline match"));
    }
    let owner = s.cur.and_then(|c| s.scripts.get(c)).map_or(0, |x| x.stage_owner);
    if owner <= 0 {
        return Err(lua_err("zone owner unavailable"));
    }
    let slot = if handle != 0 { zone_slot(&*s.game, handle) } else { -1 };
    let mut z;
    if handle != 0 {
        if slot < 0 {
            return Err(lua_err("zone does not exist"));
        }
        z = zone_read(&*s.game, slot);
        if z.owner != owner {
            return Err(lua_err("zone belongs to another script"));
        }
    } else {
        z = SzDef { kind: "trigger".into(), ..Default::default() };
    }
    const KEYS: [&str; 10] = ["name", "label", "kind", "tags", "model", "points", "x0", "y0", "x1", "y1"];
    for kv in t.clone().pairs::<Value, Value>() {
        let (k, _) = kv?;
        let ok = matches!(&k, Value::String(s) if KEYS.contains(&&*s.to_string_lossy()));
        if !ok {
            return Err(lua_err("unknown zone definition field"));
        }
    }
    zone_string(&t, "name", &mut z.name, 81)?;
    zone_string(&t, "label", &mut z.label, 81)?;
    zone_string(&t, "kind", &mut z.kind, 32)?;
    if handle == 0 && z.label.is_empty() {
        z.label = z.name.clone();
    }
    let mv: Value = t.get("model")?;
    if !matches!(mv, Value::Nil) {
        let v = neg1(one(name, &mv).check_integer(1))?;
        if v < 0 || v > i32::MAX as i64 {
            return Err(lua_err("model must be zero or a positive handle"));
        }
        z.model = v as i32;
    }
    let tv: Value = t.get("tags")?;
    if !matches!(tv, Value::Nil) {
        let tt = neg1(one(name, &tv).check_table(1))?;
        z.ntags = tt.raw_len();
        if z.ntags > 8 {
            return Err(lua_err("zone permits eight tags"));
        }
        zone_array(&tt, z.ntags)?;
        z.tags = Default::default();
        for i in 0..z.ntags {
            match tt.raw_get::<Value>(i + 1)? {
                Value::String(s) => {
                    let b = s.as_bytes();
                    if b.is_empty() || b.len() > 31 || b.contains(&0) {
                        return Err(lua_err("zone tags need 1..31 bytes without NUL"));
                    }
                    z.tags[i] = String::from_utf8_lossy(&b).to_string();
                }
                _ => return Err(lua_err("zone tags must be text")),
            }
        }
    }
    let mut has_rect = 0;
    let mut bounds = [0f32; 4];
    for (i, c) in ["x0", "y0", "x1", "y1"].iter().enumerate() {
        let v: Value = t.get(*c)?;
        if !matches!(v, Value::Nil) {
            has_rect += 1;
            bounds[i] = num_at(name, &v)? as f32;
        }
    }
    let pv: Value = t.get("points")?;
    let has_points = !matches!(pv, Value::Nil);
    if has_points && has_rect > 0 {
        return Err(lua_err("choose rectangle or polygon"));
    }
    if has_rect > 0 {
        if has_rect != 4 || !(bounds[0] < bounds[2] && bounds[1] < bounds[3]) {
            return Err(lua_err("rectangle requires x0<x1 and y0<y1 with all four coordinates"));
        }
        z.n = 4;
        z.x[0] = bounds[0];
        z.x[3] = bounds[0];
        z.x[1] = bounds[2];
        z.x[2] = bounds[2];
        z.y[0] = bounds[1];
        z.y[1] = bounds[1];
        z.y[2] = bounds[3];
        z.y[3] = bounds[3];
    }
    if has_points {
        let pt = neg1(one(name, &pv).check_table(1))?;
        z.n = pt.raw_len();
        if z.n < 3 || z.n > SZ_VERTICES {
            return Err(lua_err("convex polygon requires 3..16 vertices"));
        }
        zone_array(&pt, z.n)?;
        for i in 0..z.n {
            let p = neg1(one(name, &pt.raw_get::<Value>(i + 1)?).check_table(1))?;
            zone_array(&p, 2)?;
            z.x[i] = num_at(name, &p.raw_get::<Value>(1)?)? as f32;
            z.y[i] = num_at(name, &p.raw_get::<Value>(2)?)? as f32;
        }
    }
    z.owner = owner;
    if !validate(&mut z) {
        return Err(lua_err("zone requires a name and finite, strictly convex, nonzero geometry within +/-49000"));
    }
    // all Lua validation precedes the first game write; commit validates capacity, name uniqueness, model and ownership
    s.rw_branch();
    {
        let g = &mut s.game;
        g.zone_draft(-1, 0);
        g.zone_draft(SZF_N, z.n as i32);
        g.zone_draft(SZF_MODEL, z.model);
        g.zone_draft(SZF_TAGS, z.ntags as i32);
        for i in 0..z.n {
            g.zone_draft(SZF_X + i as i32, bits(z.x[i]));
            g.zone_draft(SZF_Y + i as i32, bits(z.y[i]));
        }
        for i in 0..(SZT_TAG as usize + z.ntags) {
            let p: &str = match i {
                0 => &z.name,
                1 => &z.label,
                2 => &z.kind,
                _ => &z.tags[i - SZT_TAG as usize],
            };
            for (j, c) in p.bytes().enumerate() {
                g.zone_draft_text(i as i32, j as i32, c as i32);
            }
        }
    }
    let floor = s.zone.serial;
    let i = s.game.zone_commit(handle, owner, floor);
    if i < 0 {
        return Err(lua_err(format!("zone refused ({i}): invalid definition, owner, duplicate name, capacity 64 or missing model")));
    }
    if i > s.zone.serial {
        s.zone.serial = i;
    }
    s.log(&format!("script: zone {} {} ({})", if handle != 0 { "updated" } else { "added" }, i, z.name));
    Ok(Value::Integer(i as i64))
}

fn zone_list(lua: &mlua::Lua, g: &dyn ScriptGame, entity: i32, point: bool, x: f32, y: f32) -> LuaResult<Table> {
    let mut list: Vec<SzDef> = Vec::new();
    for i in 0..SZ_CAP as i32 {
        if g.zone_field(i, SZF_LIVE) == 1 {
            if entity >= 0 && g.zone_entity(entity, 3, i) == 0 {
                continue;
            }
            let z = zone_read(g, i);
            if point && !contains(&z, x, y) {
                continue;
            }
            list.push(z);
        }
    }
    list.sort_by(|a, b| {
        a.kind
            .as_bytes()
            .cmp(b.kind.as_bytes())
            .then(a.area.partial_cmp(&b.area).unwrap_or(std::cmp::Ordering::Equal))
            .then(a.handle.cmp(&b.handle))
    });
    let out = lua.create_table()?;
    for (i, z) in list.iter().enumerate() {
        let slot = zone_slot(g, z.handle);
        let frames = if entity >= 0 { g.zone_entity(entity, 4, slot) } else { -1 };
        out.raw_set(i as i64 + 1, zone_push(lua, z, frames)?)?;
    }
    Ok(out)
}

fn zone_add(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Value> {
    zone_write(st, "zone_add", 0, &a, 1)
}
fn zone_set(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Value> {
    let h = zone_handle(&a, 1)?;
    zone_write(st, "zone_set", h, &a, 2)
}
fn zone_remove(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<bool> {
    let h = zone_handle(&a, 1)?;
    let mut s = st.borrow_mut();
    s.require_offline("zone_remove")?;
    s.rw_branch();
    let owner = s.cur.and_then(|c| s.scripts.get(c)).map_or(0, |x| x.stage_owner);
    if s.game.zone_remove(h, owner) == 0 {
        return Err(lua_err("zone missing or owned by another script"));
    }
    s.log(&format!("script: zone removed {h}"));
    Ok(true)
}
fn zones(lua: &mlua::Lua, st: &Shared, _a: Args) -> LuaResult<Table> {
    zone_list(lua, &*st.borrow().game, -1, false, 0.0, 0.0)
}
fn point_zones(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Table> {
    let (x, y) = (a.check_number(1)? as f32, a.check_number(2)? as f32);
    if !x.is_finite() || !y.is_finite() {
        return Err(lua_err("point must be finite"));
    }
    zone_list(lua, &*st.borrow().game, -1, true, x, y)
}
fn zones_at(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Table> {
    let port = a.check_integer(1)?;
    let sub = a.opt_integer(2, 0)?;
    if !(1..=6).contains(&port) || !(0..=1).contains(&sub) {
        return Err(lua_err("zones_at requires port 1..6 and optional sub 0..1"));
    }
    zone_list(lua, &*st.borrow().game, (port as i32 - 1) * 2 + sub as i32, false, 0.0, 0.0)
}
fn zone_members(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Table> {
    let slot = zone_slot(&*st.borrow().game, zone_handle(&a, 1)?);
    if slot < 0 {
        return Err(lua_err("zone does not exist"));
    }
    let s = st.borrow();
    let g = &*s.game;
    let out = lua.create_table()?;
    let mut n = 0;
    for e in 0..SZ_ENTITIES as i32 {
        if g.zone_entity(e, 3, slot) != 0 {
            let t = lua.create_table()?;
            t.int("port", (e / 2 + 1) as i64)?;
            t.int("sub", (e % 2) as i64)?;
            t.int("entity", g.zone_entity(e, 0, 0) as i64)?;
            t.num("x", fl(g.zone_entity(e, 1, 0)) as f64)?;
            t.num("y", fl(g.zone_entity(e, 2, 0)) as f64)?;
            t.int("frames", g.zone_entity(e, 4, slot) as i64)?;
            n += 1;
            out.raw_set(n, t)?;
        }
    }
    Ok(out)
}

/// `gs_zone_names`: the names of the zones a port's primary fighter is in
pub fn zone_names(s: &State, port: i32) -> String {
    let g = &*s.game;
    let names: Vec<String> =
        (0..SZ_CAP as i32).filter(|&i| g.zone_entity(port * 2, 3, i) != 0).map(|i| read_text(g, i, SZT_NAME, 81, false)).collect();
    if names.is_empty() { "outside every zone".into() } else { names.join(", ") }
}
/// `gs_zone_has_name`
pub fn zone_has_name(s: &State, port: i32, name: &str) -> bool {
    let g = &*s.game;
    (0..SZ_CAP as i32).any(|i| g.zone_entity(port * 2, 3, i) != 0 && read_text(g, i, SZT_NAME, 81, false) == name)
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    reg(e, gd, "zone_add", zone_add)?;
    reg(e, gd, "zone_set", zone_set)?;
    reg(e, gd, "zone_remove", zone_remove)?;
    reg(e, gd, "zones", zones)?;
    reg(e, gd, "point_zones", point_zones)?;
    reg(e, gd, "zones_at", zones_at)?;
    reg(e, gd, "zone_members", zone_members)?;
    Ok(())
}

impl Engine {
    /// `gs_zones_arm`: tell the game which of enter/exit/none/some anyone listens to
    pub fn zones_arm(&self) {
        if self.st.borrow().game.resimulating() {
            return;
        }
        let n = self.st.borrow().scripts.len();
        let mut mask = 0;
        for i in 0..n {
            for (k, h) in ZONE_HOOKS.iter().enumerate() {
                if self.get_hook(i, h).is_some() {
                    mask |= 1 << k;
                }
            }
        }
        // a script listening on the common envelope hears every zone signal
        if (0..n).any(|i| self.get_hook(i, "on_event").is_some()) {
            mask |= 15;
        }
        self.st.borrow_mut().game.zones_arm(mask);
    }

    /// `gs_zones_dispatch`
    pub fn zones_dispatch(&self) {
        let batch: Vec<(i32, i32, i32, i32, f32, f32, i32, String, String, String, Vec<i32>)> = {
            let s = self.st.borrow();
            let n = s.game.zone_event_count();
            let net = s.host.borrow().session();
            if n == 0 || s.zone.dispatching || s.game.resimulating() || net {
                return;
            }
            let g = &*s.game;
            (0..n)
                .map(|e| {
                    (
                        g.zone_event(e, SZE_WHAT),
                        g.zone_event(e, SZE_ENTITY),
                        g.zone_event(e, SZE_PORT),
                        g.zone_event(e, SZE_SUB),
                        fl(g.zone_event(e, SZE_X)),
                        fl(g.zone_event(e, SZE_Y)),
                        g.zone_event(e, SZE_ZONE),
                        read_text(g, e, SZT_NAME, 81, true),
                        read_text(g, e, SZT_LABEL, 81, true),
                        read_text(g, e, SZT_KIND, 32, true),
                        (0..SZ_CAP as i32).map(|k| g.zone_event(e, SZE_FROM + k)).collect(),
                    )
                })
                .collect()
        };
        self.st.borrow_mut().zone.dispatching = true;
        let n_scripts = self.st.borrow().scripts.len();
        for v in &batch {
            let hook = ZONE_HOOKS[(v.0 - 1).clamp(0, 3) as usize];
            // the common envelope row is built once per signal: every listener sees the same id
            let mut env: Option<GsEvent> = None;
            for i in 0..n_scripts {
                if self.st.borrow().game.resimulating() {
                    continue;
                }
                if env.is_none() && self.get_hook(i, "on_event").is_some() {
                    let mut e = GsEvent::new(0, v.2, v.6, 0, 0);
                    let mut s = self.st.borrow_mut();
                    s.spine_capture(&mut e);
                    e.spine.phase = 5;
                    env = Some(e);
                }
                if let (Some(e), false) = (&env, self.st.borrow().game.resimulating()) {
                    if let Some(f) = self.get_hook(i, "on_event") {
                        if let Ok(t) = spine_push(&self.lua, e, hook) {
                            self.pcall(i, &f, MultiValue::from_iter([Value::Table(t)]), "on_event");
                        }
                    }
                }
                if self.st.borrow().game.resimulating() {
                    continue;
                }
                let Some(f) = self.get_hook(i, hook) else { continue };
                let build = || -> LuaResult<Table> {
                    let t = self.lua.create_table()?;
                    t.int("zone", v.6 as i64)?;
                    t.string("name", &v.7)?;
                    t.string("label", &v.8)?;
                    t.string("kind", &v.9)?;
                    t.int("port", v.2 as i64)?;
                    t.int("entity", v.1 as i64)?;
                    t.int("sub", v.3 as i64)?;
                    t.num("x", v.4 as f64)?;
                    t.num("y", v.5 as f64)?;
                    let from = self.lua.create_table()?;
                    for h in v.10.iter().filter(|h| **h != 0) {
                        from.raw_push(*h as i64)?;
                    }
                    t.set("from", from)?;
                    Ok(t)
                };
                if let Ok(t) = build() {
                    self.pcall(i, &f, MultiValue::from_iter([Value::Table(t)]), hook);
                }
            }
        }
        let pending = {
            let mut s = self.st.borrow_mut();
            s.zone.dispatching = false;
            std::mem::take(&mut s.zone.pending_release)
        };
        if pending != 0 {
            self.st.borrow_mut().game.zones_frame(pending);
            self.zones_dispatch();
            self.st.borrow_mut().zone.cleanup = false;
        }
    }

    /// `gs_zones_release`: owner 0 is every script's zones
    pub fn zones_release(&self, owner: i32) {
        let terminal = if owner != 0 { 2 } else { 1 };
        {
            let mut s = self.st.borrow_mut();
            s.game.zones_release(owner);
            s.zone.cleanup = true;
            if s.zone.dispatching {
                if s.zone.pending_release == 0 || terminal == 1 {
                    s.zone.pending_release = terminal;
                }
                return;
            }
            s.game.zones_frame(terminal);
        }
        self.zones_dispatch();
        self.st.borrow_mut().zone.cleanup = false;
    }
}
