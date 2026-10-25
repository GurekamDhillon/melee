//! Contact observer and waits (`gw_script_contacts.inc`, `gw_script_contacts_core.inc`): contacts, contact_events,
//! contact_trace, contact_overlay, wait_until{...}, wait_status, and the `contacts` / `trace` / `wait` console commands.
//! Observer-only state: never read by game simulation or saved in snapshots.
use super::core::data_path;
use super::lab::project;
use super::players::{anim_name, motion_name_at};
use super::zones::{zone_has_name, zone_names};
use super::{lua_err, reg, slot_arg, TableExt};
use crate::args::{fmt_g, Args};
use crate::engine::{Draw, DrawKind, Engine, Shared, State, GS_GREEN, GS_RED, GS_WHITE};
use crate::game::lab::{I_JUMPS_USED, I_MAX_JUMPS, I_NAME_KIND};
use crate::game::{players_present, ScriptGame, SF_FACING, SF_X, SF_Y, SI_ACTION, SI_AIRBORNE, SI_SLOT_TYPE};
use crate::zones_core::SZ_CAP;
use mlua::{Table, Value};
use std::io::Write;

type LuaResult<T> = mlua::Result<T>;

pub const GS_CONTACT_CAP: usize = 512;
pub const GS_CONTACT_WAITS: usize = 64;
pub const GS_CONTACT_TRACE_MAX: usize = 1024 * 1024;
pub const GS_CONTACT_LABELS: usize = 1024;
// script_contacts.h
const SC_FLOOR: i32 = 0;
const SC_LEDGE: i32 = 4;
const SC_LINE: i32 = 0;
const SC_OWNER: i32 = 1;
const SC_PART: i32 = 2;
const SC_PASS: i32 = 4;
const SC_X0: i32 = 5;
const KINDS: [&str; 5] = ["floor", "left_wall", "right_wall", "ceiling", "ledge"];

#[derive(Clone, Debug, Default)]
pub struct Surface {
    pub present: bool,
    pub owner: i32,
    pub part: i32,
    pub line: i32,
    pub kind: i32,
    pub passthrough: bool,
    pub label: String,
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    pub offset: f32,
    pub t: f32,
    pub nx: f32,
    pub ny: f32,
}
#[derive(Clone, Debug, Default)]
pub struct Picture {
    pub present: bool,
    pub state: i32,
    pub state_frame: i32,
    pub jumps_left: i32,
    pub airborne: bool,
    pub jump: bool,
    pub pass: bool,
    pub knocked: bool,
    pub dead: bool,
    pub rebirth: bool,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub facing: f32,
    pub state_name: String,
    pub zones: String,
    pub zone_handle: Vec<i32>,
    pub zone_frames: Vec<i32>,
    pub surface: [Surface; 5],
}
#[derive(Clone, Debug)]
pub struct ContactEvent {
    pub seq: u64,
    pub frame: i32,
    pub port: i32,
    pub kind: String,
    pub reason: String,
    pub surface: Surface,
    pub fighter: Picture,
}
#[derive(Clone, Debug, Default)]
pub struct Wait {
    pub used: bool,
    pub handle: i32,
    pub owner: usize,
    pub port: usize,
    pub frames: i32,
    pub timeout: i32,
    pub done: bool,
    pub ok: bool,
    pub console: bool,
    pub has_x: bool,
    pub has_y: bool,
    pub has_state: bool,
    pub has_on: bool,
    pub has_air: bool,
    pub airborne: bool,
    pub on: i32,
    pub state_id: i32,
    pub saw_air: bool,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub last_on: Surface,
    pub state: String,
    pub label: String,
    pub drop_label: String,
    pub reason: String,
    pub zone: String,
}

pub struct Contacts {
    ring: std::collections::VecDeque<ContactEvent>,
    previous: [Picture; 6],
    waits: Vec<Wait>,
    next: u64,
    dropped: u64,
    pub enabled: bool,
    pub overlay: bool,
    state_changes: bool,
    wait_serial: i32,
    file: Option<std::fs::File>,
    path: std::path::PathBuf,
    bytes: usize,
    trace_owner: Option<usize>,
    /// handles and resource-owner tokens never repeat within a scene: retired labels stay (handle, owner, label)
    pub labels: Vec<(i32, i32, String)>,
}
impl Default for Contacts {
    fn default() -> Self {
        Contacts {
            ring: Default::default(),
            previous: Default::default(),
            waits: vec![Wait::default(); GS_CONTACT_WAITS],
            next: 1,
            dropped: 0,
            enabled: false,
            overlay: false,
            state_changes: false,
            wait_serial: 0,
            file: None,
            path: Default::default(),
            bytes: 0,
            trace_owner: None,
            labels: Vec::new(),
        }
    }
}

fn fl(b: i32) -> f32 {
    f32::from_bits(b as u32)
}
fn project_surface(s: &mut Surface, x: f32, y: f32) {
    if s.x0 > s.x1 || (s.x0 == s.x1 && s.y0 > s.y1) {
        std::mem::swap(&mut s.x0, &mut s.x1);
        std::mem::swap(&mut s.y0, &mut s.y1);
    }
    let (dx, dy) = (s.x1 - s.x0, s.y1 - s.y0);
    let len2 = dx * dx + dy * dy;
    s.t = if len2 > 0.0 { ((x - s.x0) * dx + (y - s.y0) * dy) / len2 } else { 0.0 };
    if s.t < 0.0 {
        s.t = 0.0;
    }
    if s.t > 1.0 {
        s.t = 1.0;
    }
    s.offset = s.t * len2.sqrt();
}
fn surface_name(s: &Surface) -> &str {
    if !s.label.is_empty() {
        &s.label
    } else if s.owner != 0 {
        "unlabelled object"
    } else {
        "stage"
    }
}

impl Contacts {
    pub fn label(&self, handle: i32) -> &str {
        if handle > 0 {
            if let Some((_, _, l)) = self.labels.iter().find(|(h, _, _)| *h == handle) {
                return l;
            }
        }
        ""
    }
    /// `gs_contact_label_put`
    pub fn label_put(&mut self, handle: i32, owner: i32, label: &str) -> bool {
        if let Some(e) = self.labels.iter_mut().find(|(h, _, _)| *h == handle) {
            if e.1 != owner {
                return false;
            }
            e.2 = label.chars().take(80).collect();
            return true;
        }
        if self.labels.len() >= GS_CONTACT_LABELS {
            return false;
        }
        self.labels.push((handle, owner, label.chars().take(80).collect()));
        true
    }
    fn emit(&mut self, kind: &str, reason: &str, port: usize, p: &Picture, s: Option<&Surface>, frame: i32) {
        let mut dropped = false;
        if self.ring.len() == GS_CONTACT_CAP {
            self.ring.pop_front();
            dropped = true;
        }
        if dropped {
            self.dropped += 1;
        }
        let e = ContactEvent {
            seq: self.next,
            frame,
            port: port as i32 + 1,
            kind: kind.into(),
            reason: reason.into(),
            surface: s.cloned().unwrap_or_default(),
            fighter: p.clone(),
        };
        self.next += 1;
        if self.file.is_some() {
            self.trace_event(&e);
        }
        self.ring.push_back(e);
        if kind == "land" {
            for w in self.waits.iter_mut().filter(|w| w.used && !w.done && w.port == port) {
                w.drop_label.clear(); // a later landing supersedes an earlier drop
            }
        }
        if kind == "drop_through" {
            let label = match s {
                Some(s) if !s.label.is_empty() => s.label.clone(),
                _ => "stage".into(),
            };
            for w in self.waits.iter_mut().filter(|w| w.used && !w.done && w.port == port) {
                w.drop_label = label.clone();
            }
        }
    }
    fn trace_event(&mut self, e: &ContactEvent) {
        // a record is bounded below 4096 bytes by the fixed string fields: rotate BEFORE it
        if self.bytes + 4096 > GS_CONTACT_TRACE_MAX {
            self.file = None;
            let mut rotated = self.path.clone().into_os_string();
            rotated.push(".1");
            let rotated = std::path::PathBuf::from(rotated);
            let _ = std::fs::remove_file(&rotated);
            if std::fs::rename(&self.path, &rotated).is_err() {
                self.trace_owner = None;
                return;
            }
            match std::fs::File::create(&self.path) {
                Ok(f) => self.file = Some(f),
                Err(_) => {
                    self.trace_owner = None;
                    return;
                }
            }
            self.bytes = 0;
        }
        let line = contact_json(e);
        if let Some(f) = self.file.as_mut() {
            if f.write_all(line.as_bytes()).and_then(|_| f.flush()).is_err() {
                self.trace_close();
                return;
            }
            self.bytes += line.len();
        }
    }
    fn trace_close(&mut self) {
        self.file = None;
        self.trace_owner = None;
    }
    fn leave(a: &Picture, b: &Picture) -> &'static str {
        if b.pass {
            "dropped_through"
        } else if b.knocked || b.dead {
            "knocked"
        } else if b.jump {
            "jumped"
        } else if b.x < a.surface[0].x0 {
            "ran_off_left"
        } else if b.x > a.surface[0].x1 {
            "ran_off_right"
        } else {
            "lost_contact"
        }
    }
    fn same(a: &Surface, b: &Surface) -> bool {
        a.present == b.present && (!a.present || (a.line == b.line && a.owner == b.owner))
    }
    /// `gs_contact_observe`
    fn observe(&mut self, port: usize, b: &Picture, frame: i32) {
        let a = self.previous[port].clone();
        if a.present && !b.present && !a.dead {
            self.emit("ko", "fighter_removed", port, &a, None, frame);
        }
        if b.present {
            for i in 0..5 {
                if !Self::same(&a.surface[i], &b.surface[i]) {
                    if a.surface[i].present {
                        let release = if i == 0 { "leave" } else if i == 4 { "ledge_release" } else { "wall_release" };
                        let reason = if i == 0 { Self::leave(&a, b) } else { "" };
                        if i != 3 {
                            self.emit(release, reason, port, b, Some(&a.surface[i]), frame);
                        }
                        if i == 0 && b.pass && a.surface[i].passthrough {
                            self.emit("drop_through", "dropped_through", port, b, Some(&a.surface[i]), frame);
                        }
                    }
                    if b.surface[i].present {
                        let kind = match i {
                            0 => "land",
                            4 => "ledge_grab",
                            3 => "ceiling_hit",
                            _ => "wall_touch",
                        };
                        self.emit(kind, "", port, b, Some(&b.surface[i]), frame);
                    }
                }
            }
            if b.dead && !a.dead {
                self.emit("ko", "", port, b, None, frame);
            }
            if b.rebirth && !a.rebirth {
                self.emit("respawn", "", port, b, None, frame);
            }
            if self.state_changes && a.present && a.state != b.state {
                self.emit("state_change", "", port, b, None, frame);
            }
        }
        self.previous[port] = b.clone();
    }
    /// `gs_contacts_reset`
    pub fn reset(&mut self, reason: &str, console: &mut Vec<(bool, String)>, log: &mut Vec<String>) {
        self.ring.clear();
        self.dropped = 0;
        self.next += 1;
        self.previous = Default::default();
        for w in self.waits.iter_mut().filter(|w| w.used && !w.done) {
            resolve(w, false, reason, console, log);
        }
    }
}

fn matches(w: &Wait, p: &Picture, s: &State) -> bool {
    let dx = if w.has_x { p.x - w.x } else { 0.0 };
    let dy = if w.has_y { p.y - w.y } else { 0.0 };
    if !p.present || dx * dx + dy * dy > w.radius * w.radius {
        return false;
    }
    if !w.zone.is_empty() && !zone_has_name(s, w.port as i32, &w.zone) {
        return false;
    }
    if w.has_state && (if !w.state.is_empty() { w.state != p.state_name } else { w.state_id != p.state }) {
        return false;
    }
    if w.has_air && w.airborne != p.airborne {
        return false;
    }
    if w.has_on
        && (!p.surface[0].present
            || (if !w.label.is_empty() { w.label != p.surface[0].label } else { w.on != p.surface[0].owner }))
    {
        return false;
    }
    true
}
fn failure(w: &Wait, p: &Picture) -> String {
    if !w.zone.is_empty() {
        format!(
            "timeout {} frames waiting for zone \"{}\"; in {}",
            w.frames,
            w.zone,
            if p.zones.is_empty() { "outside every zone" } else { &p.zones }
        )
    } else if !p.present {
        format!("fighter was absent after {} frames", w.frames)
    } else if !w.drop_label.is_empty() {
        format!("fell through \"{}\"", w.drop_label)
    } else if w.has_air && w.airborne && !w.saw_air {
        "never left the ground".into()
    } else if p.surface[1].present || p.surface[2].present {
        let side = if p.surface[1].present { 1 } else { 2 };
        format!(
            "stopped at wall \"{}\" ({} side) at x={:.1} after {} frames",
            surface_name(&p.surface[side]),
            if side == 1 { "left" } else { "right" },
            p.x,
            w.frames
        )
    } else if p.surface[0].present {
        format!("timeout {} frames; last on \"{}\" offset {:.1}", w.frames, surface_name(&p.surface[0]), p.surface[0].offset)
    } else if w.last_on.present {
        format!("timeout {} frames; last on \"{}\" offset {:.1}", w.frames, surface_name(&w.last_on), w.last_on.offset)
    } else {
        format!("timeout {} frames; last at x={:.1} y={:.1} in {}", w.frames, p.x, p.y, p.state_name)
    }
}
fn resolve(w: &mut Wait, ok: bool, reason: &str, console: &mut Vec<(bool, String)>, log: &mut Vec<String>) {
    w.done = true;
    w.ok = ok;
    w.reason = reason.chars().take(255).collect();
    if w.console {
        console.push((ok, format!("wait {}: {} ({} frames)", w.handle, w.reason, w.frames)));
        log.push(format!("script wait {}: {}: {} ({} frames)", w.handle, if ok { "ok" } else { "failed" }, w.reason, w.frames));
    }
}
fn wait_seed(w: &mut Wait, p: &Picture) {
    if p.present && p.airborne {
        w.saw_air = true;
    }
    if p.surface[0].present {
        w.last_on = p.surface[0].clone();
    }
}

fn json_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' | '\\' => {
                out.push('\\');
                out.push(c);
            }
            c if (c as u32) < 32 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}
fn g9(f: f32) -> String {
    fmt_g(f as f64, 9)
}
fn contact_json(e: &ContactEvent) -> String {
    let (p, s) = (&e.fighter, &e.surface);
    let mut o = format!("{{\"seq\":{},\"frame\":{},\"port\":{},\"event\":", e.seq, e.frame, e.port);
    json_string(&mut o, &e.kind);
    o.push_str(",\"reason\":");
    json_string(&mut o, &e.reason);
    o.push_str(&format!(",\"x\":{},\"y\":{},\"vx\":{},\"vy\":{},\"facing\":{},\"state\":", g9(p.x), g9(p.y), g9(p.vx), g9(p.vy), g9(p.facing)));
    json_string(&mut o, &p.state_name);
    o.push_str(",\"zone_names\":");
    json_string(&mut o, &p.zones);
    o.push_str(",\"zones\":[");
    for (zi, (h, f)) in p.zone_handle.iter().zip(&p.zone_frames).enumerate() {
        o.push_str(&format!("{}{{\"handle\":{},\"frames\":{}}}", if zi > 0 { "," } else { "" }, h, f));
    }
    o.push(']');
    o.push_str(&format!(
        ",\"state_id\":{},\"state_frame\":{},\"jumps_left\":{},\"airborne\":{},\"contact\":",
        p.state, p.state_frame, p.jumps_left, p.airborne
    ));
    if !s.present {
        o.push_str("null");
    } else {
        o.push_str("{\"owner\":");
        if s.owner != 0 {
            o.push_str(&s.owner.to_string());
        } else {
            o.push_str("\"stage\"");
        }
        o.push_str(",\"label\":");
        json_string(&mut o, &s.label);
        o.push_str(&format!(",\"part\":{},\"line\":{},\"kind\":", s.part, s.line));
        json_string(&mut o, KINDS[s.kind as usize]);
        o.push_str(&format!(
            ",\"passthrough\":{},\"x0\":{},\"y0\":{},\"x1\":{},\"y1\":{},\"offset\":{},\"t\":{},\"normal\":{{\"x\":{},\"y\":{}}}}}",
            s.passthrough,
            g9(s.x0),
            g9(s.y0),
            g9(s.x1),
            g9(s.y1),
            g9(s.offset),
            g9(s.t),
            g9(s.nx),
            g9(s.ny)
        ));
    }
    o.push_str("}\n");
    o
}

fn read_surface(s: &State, slot: i32, kind: i32, line: i32) -> Surface {
    let g = &*s.game;
    let mut r = Surface { line: g.contact_field(slot, kind, SC_LINE, line), ..Default::default() };
    if r.line < 0 {
        return r;
    }
    r.present = true;
    r.kind = kind;
    r.owner = g.contact_field(slot, kind, SC_OWNER, line);
    r.part = g.contact_field(slot, kind, SC_PART, line);
    r.passthrough = g.contact_field(slot, kind, SC_PASS, line) != 0;
    r.label = s.contacts.label(r.owner).to_string();
    let f: Vec<f32> = (0..8).map(|k| fl(g.contact_field(slot, kind, SC_X0 + k, line))).collect();
    r.x0 = f[0];
    r.y0 = f[1];
    r.x1 = f[2];
    r.y1 = f[3];
    r.nx = f[6];
    r.ny = f[7];
    project_surface(&mut r, f[4], f[5]);
    // ledges use the facing-selected endpoint, not the fighter's hanging ECB
    if kind == SC_LEDGE {
        r.t = if g.fighter_f(slot, SF_FACING) > 0.0 { 0.0 } else { 1.0 };
        r.offset = r.t * (r.x1 - r.x0).hypot(r.y1 - r.y0);
    }
    if line >= 0 || kind == SC_LEDGE {
        let (dx, dy) = (r.x1 - r.x0, r.y1 - r.y0);
        let n = dx.hypot(dy);
        r.nx = if n > 0.0 { -dy / n } else { 0.0 };
        r.ny = if n > 0.0 { dx / n } else { 1.0 };
    }
    r
}
/// `gs_contact_picture`
fn picture(s: &State, slot: usize) -> Picture {
    let mut p = Picture::default();
    if !s.match_active || !players_present(&*s.game, slot) {
        return p;
    }
    let g = &*s.game;
    let sl = slot as i32;
    p.present = true;
    p.state = g.fighter_i(sl, SI_ACTION);
    p.state_frame = s.state_frame[slot];
    p.airborne = g.fighter_i(sl, SI_AIRBORNE) != 0;
    p.jumps_left = (g.lab_i(sl, I_MAX_JUMPS) - g.lab_i(sl, I_JUMPS_USED)).max(0);
    p.x = g.fighter_f(sl, SF_X);
    p.y = g.fighter_f(sl, SF_Y);
    p.vx = g.fighter_f(sl, crate::game::SF_VX);
    p.vy = g.fighter_f(sl, crate::game::SF_VY);
    p.facing = g.fighter_f(sl, SF_FACING);
    let sym = g.lab_anim_symbol(sl);
    let anim = anim_name(sym.as_deref());
    p.state_name = motion_name_at(g, sl, g.lab_i(sl, I_NAME_KIND), p.state, Some(&anim)).chars().take(127).collect();
    let flags = g.contact_flags(sl);
    p.jump = flags & 1 != 0;
    p.pass = flags & 2 != 0;
    p.knocked = flags & 4 != 0;
    p.dead = flags & 8 != 0;
    p.rebirth = flags & 16 != 0;
    for i in 0..5 {
        p.surface[i] = read_surface(s, sl, i as i32, -1);
    }
    p.zones = zone_names(s, sl);
    for i in 0..SZ_CAP as i32 {
        let h = g.zone_entity(sl * 2, 3, i);
        if h != 0 {
            p.zone_handle.push(h);
            p.zone_frames.push(g.zone_entity(sl * 2, 4, i));
        }
    }
    p
}

fn push_surface(lua: &mlua::Lua, s: &Surface) -> LuaResult<Value> {
    if !s.present {
        return Ok(Value::Nil);
    }
    let t = lua.create_table()?;
    if s.owner != 0 {
        t.int("owner", s.owner as i64)?;
    } else {
        t.string("owner", "stage")?;
    }
    t.string("label", &s.label)?;
    t.int("part", s.part as i64)?;
    t.int("line", s.line as i64)?;
    t.string("kind", KINDS[s.kind as usize])?;
    t.boolean("passthrough", s.passthrough)?;
    for (k, v) in [("x0", s.x0), ("y0", s.y0), ("x1", s.x1), ("y1", s.y1), ("offset", s.offset), ("t", s.t)] {
        t.num(k, v as f64)?;
    }
    push_xy(lua, &t, "normal", s.nx, s.ny)?;
    Ok(Value::Table(t))
}
fn push_xy(lua: &mlua::Lua, t: &Table, k: &str, x: f32, y: f32) -> LuaResult<()> {
    let n = lua.create_table()?;
    n.num("x", x as f64)?;
    n.num("y", y as f64)?;
    t.set(k, n)
}
fn push_fighter(lua: &mlua::Lua, t: &Table, p: &Picture) -> LuaResult<()> {
    for (k, v) in [("x", p.x), ("y", p.y), ("vx", p.vx), ("vy", p.vy), ("facing", p.facing)] {
        t.num(k, v as f64)?;
    }
    t.string("state", &p.state_name)?;
    t.int("state_id", p.state as i64)?;
    t.int("state_frame", p.state_frame as i64)?;
    t.int("jumps_left", p.jumps_left as i64)?;
    t.boolean("airborne", p.airborne)?;
    t.string("zone_names", &p.zones)?;
    let zs = lua.create_table()?;
    for (zi, (h, f)) in p.zone_handle.iter().zip(&p.zone_frames).enumerate() {
        let z = lua.create_table()?;
        z.int("handle", *h as i64)?;
        z.int("frames", *f as i64)?;
        zs.raw_set(zi as i64 + 1, z)?;
    }
    t.set("zones", zs)
}

fn contacts(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Value> {
    let port = a.check_integer(1)?;
    if !(1..=6).contains(&port) {
        return Err(a.err(1, "port must be 1..6"));
    }
    let mut s = st.borrow_mut();
    s.contacts.enabled = true;
    let p = picture(&s, port as usize - 1);
    if !p.present {
        return Ok(Value::Nil);
    }
    let t = lua.create_table()?;
    push_fighter(lua, &t, &p)?;
    for i in 0..5 {
        t.set(KINDS[i], push_surface(lua, &p.surface[i])?)?;
    }
    Ok(Value::Table(t))
}
fn contact_events(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<mlua::MultiValue> {
    let since = a.opt_integer(1, 0)?;
    if since < 0 {
        return Err(a.err(1, "sequence must be nonnegative"));
    }
    let mut s = st.borrow_mut();
    s.contacts.enabled = true;
    let out = lua.create_table()?;
    let mut n = 0;
    for e in s.contacts.ring.iter().filter(|e| e.seq >= since as u64) {
        let t = lua.create_table()?;
        push_fighter(lua, &t, &e.fighter)?;
        t.int("seq", e.seq as i64)?;
        t.int("frame", e.frame as i64)?;
        t.int("port", e.port as i64)?;
        t.string("event", &e.kind)?;
        t.string("reason", &e.reason)?;
        t.set("contact", push_surface(lua, &e.surface)?)?;
        let sf = &e.surface;
        if sf.present {
            if sf.owner != 0 {
                t.int("owner", sf.owner as i64)?;
            } else {
                t.string("owner", "stage")?;
            }
            t.string("label", &sf.label)?;
            t.int("part", sf.part as i64)?;
            t.int("line", sf.line as i64)?;
            t.num("offset", sf.offset as f64)?;
            t.num("t", sf.t as f64)?;
            t.string("kind", KINDS[sf.kind as usize])?;
            t.boolean("passthrough", sf.passthrough)?;
            for (k, v) in [("x0", sf.x0), ("y0", sf.y0), ("x1", sf.x1), ("y1", sf.y1)] {
                t.num(k, v as f64)?;
            }
            push_xy(lua, &t, "normal", sf.nx, sf.ny)?;
        }
        n += 1;
        out.raw_set(n, t)?;
    }
    Ok(mlua::MultiValue::from_iter([Value::Table(out), Value::Integer(s.contacts.next as i64), Value::Integer(s.contacts.dropped as i64)]))
}
fn contact_trace(st: &Shared, a: &Args) -> LuaResult<bool> {
    let cur = st.borrow().cur.unwrap_or(0);
    {
        let s = st.borrow();
        if s.contacts.trace_owner.is_some_and(|o| o != cur) {
            return Err(lua_err("contact trace belongs to another script"));
        }
    }
    let on = a.truthy(1);
    let mut file = "contacts.jsonl".to_string();
    let mut changes = false;
    if let Value::Table(t) = a.get(1) {
        let v: Value = t.get("file")?;
        if !v.is_nil() {
            let b = super::caps::neg1(super::caps::one("contact_trace", &v).check_bytes(1))?;
            if b.contains(&0) {
                return Err(lua_err("trace filename contains NUL"));
            }
            file = String::from_utf8_lossy(&b).to_string();
        }
        let v: Value = t.get("state_changes")?;
        if !v.is_nil() {
            changes = super::caps::check_boolean("contact_trace", &v)?;
        }
    } else if !matches!(a.get(1), Value::Boolean(_)) {
        return Err(a.err(1, &format!("boolean expected, got {}", crate::args::typename(a.get(1).into()))));
    }
    if !on {
        let mut s = st.borrow_mut();
        s.contacts.trace_close();
        s.contacts.state_changes = false;
        return Ok(true);
    }
    let path = data_path(st, &file)?;
    let f = std::fs::OpenOptions::new().append(true).create(true).open(&path).map_err(|_| lua_err("could not open contact trace"))?;
    let bytes = f.metadata().map(|m| m.len() as usize).map_err(|_| lua_err("could not measure contact trace"))?;
    let mut s = st.borrow_mut();
    s.contacts.trace_close();
    s.contacts.file = Some(f);
    s.contacts.trace_owner = Some(cur);
    s.contacts.bytes = bytes;
    s.contacts.path = path;
    s.contacts.enabled = true;
    s.contacts.state_changes = changes;
    Ok(true)
}

fn opt(t: &Table, k: &str) -> LuaResult<Option<Value>> {
    let v: Value = t.get(k)?;
    Ok(if v.is_nil() { None } else { Some(v) })
}
fn wait_until(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<i64> {
    let t = a.check_table(1)?;
    let cur = st.borrow().cur.unwrap_or(0);
    let mut w = Wait { owner: cur, timeout: 600, radius: 1.0, ..Default::default() };
    const KEYS: [&str; 9] = ["port", "x", "y", "radius", "state", "on", "airborne", "timeout", "zone"];
    for kv in t.clone().pairs::<Value, Value>() {
        let (k, _) = kv?;
        if !matches!(&k, Value::String(s) if KEYS.contains(&&*s.to_string_lossy())) {
            return Err(lua_err("unknown wait condition"));
        }
    }
    let cn = |v: &Value| super::caps::neg1(super::caps::one("wait_until", v).check_integer(1));
    let cf = |v: &Value| super::caps::neg1(super::caps::one("wait_until", v).check_number(1));
    if let Some(v) = opt(&t, "port")? {
        let p = cn(&v)?;
        if !(1..=6).contains(&p) {
            return Err(lua_err("wait port must be 1..6"));
        }
        w.port = p as usize - 1;
    }
    if let Some(v) = opt(&t, "x")? {
        w.has_x = true;
        w.x = cf(&v)? as f32;
    }
    if let Some(v) = opt(&t, "y")? {
        w.has_y = true;
        w.y = cf(&v)? as f32;
    }
    if !w.x.is_finite() || !w.y.is_finite() || w.x.abs() > 49000.0 || w.y.abs() > 49000.0 {
        return Err(lua_err("wait coordinates must be finite within +/-49000"));
    }
    if let Some(v) = opt(&t, "radius")? {
        w.radius = cf(&v)? as f32;
        if !w.radius.is_finite() || w.radius.abs() > 49000.0 {
            return Err(lua_err("wait radius must be finite within +/-49000"));
        }
    }
    if w.radius < 0.0 {
        return Err(lua_err("wait radius must be nonnegative"));
    }
    if let Some(v) = opt(&t, "timeout")? {
        let n = cn(&v)?;
        if !(1..=360000).contains(&n) {
            return Err(lua_err("wait timeout must be 1..360000 frames"));
        }
        w.timeout = n as i32;
    }
    if let Some(v) = opt(&t, "airborne")? {
        w.has_air = true;
        w.airborne = super::caps::check_boolean("wait_until", &v)?;
    }
    if let Some(v) = opt(&t, "state")? {
        w.has_state = true;
        if let Value::String(s) = &v {
            let b = s.as_bytes();
            if b.is_empty() || b.len() > 127 || b.contains(&0) {
                return Err(lua_err("wait state must be 1..127 bytes with no NUL"));
            }
            w.state = String::from_utf8_lossy(&b).to_string();
        } else {
            let n = cn(&v)?;
            if n < 0 || n > i32::MAX as i64 {
                return Err(lua_err("wait state id must be nonnegative"));
            }
            w.state_id = n as i32;
        }
    }
    if let Some(v) = opt(&t, "on")? {
        w.has_on = true;
        if let Value::String(s) = &v {
            let b = s.as_bytes();
            if b.is_empty() || b.len() > 80 || b.contains(&0) {
                return Err(lua_err("wait on label must be 1..80 bytes with no NUL"));
            }
            w.label = String::from_utf8_lossy(&b).to_string();
        } else {
            let n = cn(&v)?;
            if n < 1 || n > i32::MAX as i64 {
                return Err(lua_err("wait on handle must be positive"));
            }
            w.on = n as i32;
        }
    }
    if let Some(v) = opt(&t, "zone")? {
        match v {
            Value::String(s) => {
                let b = s.as_bytes();
                if b.is_empty() || b.len() >= 81 || b.contains(&0) {
                    return Err(lua_err("zone zone needs 1..80 bytes without NUL"));
                }
                w.zone = String::from_utf8_lossy(&b).to_string();
            }
            _ => return Err(lua_err("zone zone must be text")),
        }
    }
    if !(w.has_x || w.has_y || w.has_state || w.has_on || w.has_air || !w.zone.is_empty()) {
        return Err(lua_err("wait needs at least one condition"));
    }
    let mut s = st.borrow_mut();
    // keep completed results until their owner reuses capacity; active waits are never evicted
    let mut at = s.contacts.waits.iter().position(|x| !x.used);
    if at.is_none() {
        at = s.contacts.waits.iter().position(|x| x.done && x.owner == cur);
    }
    let Some(at) = at.filter(|_| s.contacts.wait_serial != i32::MAX) else {
        return Err(lua_err("wait capacity exhausted"));
    };
    s.contacts.wait_serial += 1;
    w.used = true;
    w.handle = s.contacts.wait_serial;
    s.contacts.enabled = true;
    let p = picture(&s, w.port);
    wait_seed(&mut w, &p);
    let (mut console, mut log) = (Vec::new(), Vec::new());
    if matches(&w, &p, &s) {
        resolve(&mut w, true, "conditions reached", &mut console, &mut log);
    }
    let h = w.handle;
    s.contacts.waits[at] = w;
    Ok(h as i64)
}
fn wait_status(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<mlua::MultiValue> {
    let h = a.check_integer(1)?;
    let s = st.borrow();
    let cur = s.cur.unwrap_or(0);
    if let Some(w) = s.contacts.waits.iter().find(|w| w.used && w.handle as i64 == h && w.owner == cur) {
        let t = lua.create_table()?;
        t.boolean("done", w.done)?;
        t.boolean("ok", w.ok)?;
        t.int("frames", w.frames as i64)?;
        t.string("reason", &w.reason)?;
        return Ok(mlua::MultiValue::from_iter([Value::Table(t)]));
    }
    Ok(mlua::MultiValue::from_iter([Value::Nil, Value::String(lua.create_string("stale wait handle or another owner")?)]))
}
fn contact_overlay(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<bool> {
    match a.get(1) {
        Value::Boolean(b) => {
            let mut s = st.borrow_mut();
            s.contacts.overlay = *b;
            if *b {
                s.contacts.enabled = true;
            }
            Ok(true)
        }
        o => Err(a.err(1, &format!("boolean expected, got {}", crate::args::typename(Some(o))))),
    }
}

fn overlay_text(port: usize, p: &Picture) -> String {
    let s = &p.surface[0];
    let wall = if p.surface[1].present {
        surface_name(&p.surface[1]).to_string()
    } else if p.surface[2].present {
        surface_name(&p.surface[2]).to_string()
    } else {
        "-".into()
    };
    let ceil = if p.surface[3].present { surface_name(&p.surface[3]).to_string() } else { "-".into() };
    let mut text = if s.present {
        format!(
            "P{} {}  on {}  {:.1} from {} ({:.2})  wall: {}  ceil: {}",
            port + 1,
            p.state_name,
            surface_name(s),
            s.offset,
            if s.x0 == s.x1 { "bottom" } else { "left" },
            s.t,
            wall,
            ceil
        )
    } else {
        format!("P{} {}  on -  wall: {}  ceil: {}", port + 1, p.state_name, wall, ceil)
    };
    text.push_str(&format!("  zones: {}", p.zones));
    text
}
fn event_text(e: &ContactEvent) -> String {
    let verb = match e.kind.as_str() {
        "land" => "landed on",
        "leave" => "left",
        "wall_touch" => {
            if e.surface.kind == 1 { "touched the left wall" } else { "touched the right wall" }
        }
        "wall_release" => {
            if e.surface.kind == 1 { "released the left wall" } else { "released the right wall" }
        }
        "ceiling_hit" => "hit the ceiling",
        "pass_up_through" => "passed upward through",
        "drop_through" => "dropped through",
        "ledge_grab" => "grabbed the ledge",
        "ledge_release" => "released the ledge",
        "state_change" => "changed action",
        "ko" => "was KO'd",
        "respawn" => "respawned",
        k => k,
    };
    let reason = e.reason.replace('_', " ");
    if e.surface.present {
        format!(
            "frame {}: P{} {} \"{}\" (line {}, offset {:.1}), at x={:.1} y={:.1} in {}{}{}.",
            e.frame,
            e.port,
            verb,
            surface_name(&e.surface),
            e.surface.line,
            e.surface.offset,
            e.fighter.x,
            e.fighter.y,
            e.fighter.state_name,
            if reason.is_empty() { "" } else { "; " },
            reason
        )
    } else {
        format!("frame {}: P{} {} at x={:.1} y={:.1} in {}.", e.frame, e.port, verb, e.fighter.x, e.fighter.y, e.fighter.state_name)
    }
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    reg(e, gd, "contacts", contacts)?;
    reg(e, gd, "contact_events", contact_events)?;
    reg(e, gd, "contact_trace", |_, st, a| contact_trace(st, &a))?;
    reg(e, gd, "contact_overlay", contact_overlay)?;
    reg(e, gd, "wait_until", wait_until)?;
    reg(e, gd, "wait_status", wait_status)?;
    Ok(())
}

impl State {
    /// `gs_contacts_reset` for callers that hold the state (a savestate load, a rewind)
    pub fn contacts_reset(&mut self, reason: &str) {
        let (mut console, mut log) = (Vec::new(), Vec::new());
        self.contacts.reset(reason, &mut console, &mut log);
        for (ok, t) in console {
            self.print(if ok { GS_GREEN } else { GS_RED }, &t);
        }
        for l in log {
            self.log(&l);
        }
    }
}

impl Engine {
    fn flush_contact_output(&self, console: Vec<(bool, String)>, log: Vec<String>) {
        let mut s = self.st.borrow_mut();
        for (ok, t) in console {
            s.print(if ok { GS_GREEN } else { GS_RED }, &t);
        }
        for l in log {
            s.log(&l);
        }
    }
    /// `gs_contacts_reset`
    pub fn contacts_reset(&self, reason: &str) {
        self.st.borrow_mut().contacts_reset(reason);
    }
    /// `gs_contact_tick` (FramePost)
    pub fn contact_tick(&self) {
        let (mut console, mut log) = (Vec::new(), Vec::new());
        {
            let mut s = self.st.borrow_mut();
            if !s.contacts.enabled {
                return;
            }
            let frame = s.match_frame;
            for port in 0..6 {
                let p = picture(&s, port);
                s.contacts.observe(port, &p, frame);
                if p.present && p.airborne {
                    let mut line = -1;
                    loop {
                        line = s.game.contact_cross(port as i32, line);
                        if line < 0 {
                            break;
                        }
                        let sf = read_surface(&s, port as i32, SC_FLOOR, line);
                        s.contacts.emit("pass_up_through", "", port, &p, Some(&sf), frame);
                    }
                }
                // wait tick
                for i in 0..GS_CONTACT_WAITS {
                    let mut w = s.contacts.waits[i].clone();
                    if !w.used || w.done || w.port != port {
                        continue;
                    }
                    w.frames += 1;
                    wait_seed(&mut w, &p);
                    if matches(&w, &p, &s) {
                        resolve(&mut w, true, "conditions reached", &mut console, &mut log);
                    } else if w.frames >= w.timeout {
                        let r = failure(&w, &p);
                        resolve(&mut w, false, &r, &mut console, &mut log);
                    }
                    s.contacts.waits[i] = w;
                }
            }
        }
        self.flush_contact_output(console, log);
    }
    /// `gs_contact_release`: a script unloaded
    pub fn contact_release(&self, owner: usize) {
        let mut s = self.st.borrow_mut();
        if s.contacts.trace_owner == Some(owner) {
            s.contacts.trace_close();
            s.contacts.state_changes = false;
        }
        for w in s.contacts.waits.iter_mut().filter(|w| w.used && w.owner == owner) {
            *w = Wait::default();
        }
    }
    /// `gs_contact_draw`: the overlay text and stage strokes
    pub fn contact_draw(&self) {
        let mut s = self.st.borrow_mut();
        if !s.contacts.overlay || !s.match_active {
            return;
        }
        let width = s.host.borrow().canvas_width();
        let (bottom, left) = (480.0f32, 0.0f32);
        let mut row = 0;
        for port in 0..6 {
            if s.game.fighter_i(port as i32, SI_SLOT_TYPE) != 0 {
                continue; // humans only
            }
            let p = picture(&s, port);
            if !p.present {
                continue;
            }
            let text = overlay_text(port, &p);
            row += 1;
            let b = s.build;
            let size = 0.85f32.min((width - 24.0) / (7.0 * (text.len() as f32).max(1.0)));
            s.draw[b].push(Draw {
                kind: DrawKind::Text,
                x: left + 12.0,
                y: bottom - 18.0 * row as f32 - 10.0,
                w: 0.0,
                h: 0.0,
                rgba: 0xF7E7CDFF,
                size,
                text: text.chars().take(159).collect(),
            });
            let cw = width;
            for sf in p.surface.iter().filter(|x| x.present) {
                let a = project(&*s.game, cw, sf.x0, sf.y0, 0.0);
                let bb = project(&*s.game, cw, sf.x1, sf.y1, 0.0);
                if let (Ok(a), Ok(bb)) = (a, bb) {
                    s.draw[b].push(Draw { kind: DrawKind::Line, x: a.0, y: a.1, w: bb.0, h: bb.1, rgba: 0x62DDD5FF, size: 1.0, text: String::new() });
                }
                if let Ok(c) = project(&*s.game, cw, sf.x0 + (sf.x1 - sf.x0) * sf.t, sf.y0 + (sf.y1 - sf.y0) * sf.t, 0.0) {
                    s.draw[b].push(Draw { kind: DrawKind::Line, x: c.0 - 3.0, y: c.1, w: c.0 + 3.0, h: c.1, rgba: 0xFFE095FF, size: 1.0, text: String::new() });
                    s.draw[b].push(Draw { kind: DrawKind::Line, x: c.0, y: c.1 - 3.0, w: c.0, h: c.1 + 3.0, rgba: 0xFFE095FF, size: 1.0, text: String::new() });
                }
            }
        }
    }

    /// `gs_contact_console`: `contacts`, `trace`, `wait`. Arguments are data; nothing is interpolated into Lua.
    pub fn contact_console(&self, command: &str, arg: &str) -> i32 {
        let print = |ok: u32, t: &str| self.st.borrow_mut().print(ok, t);
        if command.eq_ignore_ascii_case("contacts") {
            if arg == "overlay on" || arg == "overlay off" {
                let mut s = self.st.borrow_mut();
                s.contacts.overlay = arg == "overlay on";
                if s.contacts.overlay {
                    s.contacts.enabled = true;
                }
                return 0;
            }
            match arg.parse::<i64>() {
                Ok(p) if (1..=6).contains(&p) && arg == p.to_string() => {
                    let mut s = self.st.borrow_mut();
                    s.contacts.enabled = true;
                    let pic = picture(&s, p as usize - 1);
                    let text = overlay_text(p as usize - 1, &pic);
                    if pic.present {
                        s.print(
                            GS_WHITE,
                            &format!(
                                "{text}  x={:.1} y={:.1} vx={:.1} vy={:.1} state={} frame={} jumps={} air={}",
                                pic.x, pic.y, pic.vx, pic.vy, pic.state, pic.state_frame, pic.jumps_left, pic.airborne
                            ),
                        );
                    } else {
                        s.print(GS_WHITE, "fighter absent");
                    }
                    return 0;
                }
                _ => {
                    print(GS_RED, "usage: contacts <1..6> | contacts overlay on|off");
                    return -1;
                }
            }
        }
        if command.eq_ignore_ascii_case("trace") {
            if arg == "on" || arg == "off" {
                let a = Args::new("contact_trace", mlua::MultiValue::from_iter([Value::Boolean(arg == "on")]));
                let console = self.st.borrow().console;
                self.st.borrow_mut().cur = Some(console);
                let r = contact_trace(&self.st, &a);
                self.st.borrow_mut().cur = None;
                return match r {
                    Ok(_) => 0,
                    Err(e) => {
                        print(GS_RED, &crate::engine::error_text(&e));
                        -1
                    }
                };
            }
            if arg == "last" || arg.starts_with("last ") {
                let mut n = 10usize;
                if arg.len() > 4 {
                    match arg[5..].parse::<usize>() {
                        Ok(v) if (1..=GS_CONTACT_CAP).contains(&v) => n = v,
                        _ => {
                            print(GS_RED, "trace last count must be 1..512");
                            return -1;
                        }
                    }
                }
                let mut s = self.st.borrow_mut();
                let total = s.contacts.ring.len();
                let n = n.min(total);
                let lines: Vec<String> = s.contacts.ring.iter().skip(total - n).map(event_text).collect();
                for l in lines {
                    s.print(GS_WHITE, &l);
                }
                s.contacts.enabled = true;
                return 0;
            }
            print(GS_RED, "usage: trace on|off|last [n]");
            return -1;
        }
        // wait port=1 x=41 y=8 radius=1 state=Wait on=Floor_4m.003 airborne=false timeout=600
        let t = match self.lua.create_table() {
            Ok(t) => t,
            Err(_) => return -1,
        };
        let mut valid = true;
        for token in arg.split([' ', '\t']).filter(|t| !t.is_empty()) {
            let Some((k, v)) = token.split_once('=') else {
                valid = false;
                break;
            };
            let num = v.parse::<f64>().ok();
            let val = match k {
                "zone" => Value::String(self.lua.create_string(v).unwrap()),
                "state" | "on" => match num {
                    Some(n) => Value::Number(n),
                    None => Value::String(self.lua.create_string(v).unwrap()),
                },
                "airborne" => {
                    if v != "true" && v != "false" {
                        valid = false;
                        break;
                    }
                    Value::Boolean(v == "true")
                }
                "port" | "x" | "y" | "radius" | "timeout" => match num {
                    Some(n) => Value::Number(n),
                    None => {
                        valid = false;
                        break;
                    }
                },
                _ => {
                    valid = false;
                    break;
                }
            };
            let _ = t.set(k, val);
        }
        if !valid {
            print(GS_RED, "usage: wait port=1 x=41 y=8 radius=1 state=Wait on=label airborne=false timeout=600");
            return -1;
        }
        let console = self.st.borrow().console;
        self.st.borrow_mut().cur = Some(console);
        let a = Args::new("wait_until", mlua::MultiValue::from_iter([Value::Table(t)]));
        let r = wait_until(&self.lua, &self.st, a);
        self.st.borrow_mut().cur = None;
        match r {
            Ok(h) => {
                let mut s = self.st.borrow_mut();
                if let Some(w) = s.contacts.waits.iter_mut().find(|w| w.handle as i64 == h) {
                    w.console = true;
                    if w.done {
                        let m = format!("wait {h}: {} (0 frames)", w.reason);
                        s.print(GS_GREEN, &m);
                    }
                }
                0
            }
            Err(e) => {
                print(GS_RED, &crate::engine::error_text(&e));
                -1
            }
        }
    }
}

#[allow(dead_code)]
fn _s(_: usize) -> LuaResult<usize> {
    slot_arg(&Args::new("x", Default::default()), 1)
}
