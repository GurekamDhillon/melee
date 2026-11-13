//! Envoy exploration rooms (notes/envoy-exploration.md): a Brawl Subspace Emissary map section, read at run time from
//! the owner's own files, played in place of Final Destination's geometry, with Brawl-style doors (up on the stick)
//! that move the party between map sections.
//!
//! Off unless `MELEE_ENVOY_EXPLORE=1` (and the owner's Brawl files are present). With it on, any Final Destination
//! match becomes the exploration room; vanilla play, the goldens and the gate never reach this code.
//!
//! * Sim side (deterministic, rollback safe): the collision is built once, off the game thread, from the archives'
//!   collision files into a template; at each stage load (`ground::EXPLORE_COLL`) it is copied into the game heap
//!   (the arena the snapshots cover) as a retail `MapCollData`, so `mpLibLoad` and every collision routine run unchanged.
//!   The door and transition state is plain statics (snapshot covered). Doors read only the fighters' own stick input.
//! * Presentation: the section models (MDL0 + TEX0, bind pose) are posed once and drawn in the fighter pass
//!   (`ftdrawcommon::MAP_DRAW`); Final Destination's own model is hidden (`grdisplay::HIDE_GROUND`).
//!
//! Nothing read from the owner's files is ever written anywhere or committed; the files' paths are never printed.
#![allow(static_mut_refs, clippy::missing_safety_doc)]

use crate::tu::petey_model as pm;
use crate::tu::script_game::ext as sg;
use melee_types::*;
use std::vec::Vec; // melee_types has its own `Vec` (the 3D vector): `melee_types::Vec` below

/// One map section: an adventure archive placed at `off` in the shared world (sections never overlap).
struct SecDef {
    file: &'static str,
    off: [f32; 2],
    model: &'static str,
}

/// The map (notes: why this one): The Jungle, step 3 (`080103`): a short, mostly flat corridor with a raised slab and
/// a ledge, its door into the small hut room `080103a`, and an arrow door (the section's exit).
const SECS: [SecDef; 2] = [
    SecDef { file: "080103.pac", off: [0.0, 0.0], model: "081mAdvJungle03" },
    SecDef { file: "080103a.pac", off: [1500.0, 0.0], model: "081mAdvJungle03a" },
];
/// Door links by (section, GDOR index) -> (section, GDOR index). The arrow door (type 2) is the exit: it returns the
/// party to the start of section 0 (in an Envoy run, the room's end).
const LINKS: [((usize, usize), (usize, usize)); 2] = [((0, 1), (1, 0)), ((1, 0), (0, 1))];
/// Script enemy kind used for every Brawl spawn point (the Subspace enemy lane is not merged: Goomba, see notes).
const STUB_ENEMY: i32 = 0;

#[derive(Clone, Copy, Debug)]
pub struct Door {
    pub sec: usize,
    pub x: f32,
    pub y: f32,
    /// GDOR door type (0 normal, 1 yellow, 2 arrow, ...)
    pub kind: u8,
    /// index into `Map::doors` of the destination; -1: the exit
    pub to: i32,
}

pub struct Map {
    verts: Vec<Vec2>,
    lines: Vec<MapLine>,
    ranges: [MapLineRange; 5],
    bounds: [f32; 4],
    /// per section: collision extent (left, right, top, bottom), world space
    pub ext: Vec<[f32; 4]>,
    pub doors: Vec<Door>,
    /// enemy spawn points (section, x, y, facing)
    pub spawns: Vec<(usize, f32, f32, i32)>,
    /// per section: (rig, posed triangles, offset); presentation only
    models: Vec<(pm::Rig, Vec<pm::Vert>, [f32; 2])>,
    door_models: Vec<(pm::Rig, Vec<pm::Vert>)>,
}

fn be16(d: &[u8], o: usize) -> u16 {
    d.get(o..o + 2).map(|s| u16::from_be_bytes([s[0], s[1]])).unwrap_or(0)
}
fn be32(d: &[u8], o: usize) -> u32 {
    d.get(o..o + 4).map(|s| u32::from_be_bytes([s[0], s[1], s[2], s[3]])).unwrap_or(0)
}
fn bef(d: &[u8], o: usize) -> f32 {
    f32::from_bits(be32(d, o))
}

/// A Brawl collision file: header (points, planes, objects, offsets), 8-byte points, 16-byte planes.
fn is_coll(d: &[u8]) -> bool {
    d.len() >= 0x28 && be32(d, 8) == 0x28 && (be16(d, 0) as i16) > 0 && (be16(d, 2) as i16) > 0 && be32(d, 12) as usize == 0x28 + 8 * be16(d, 0) as usize
}

/// The entries of one BLOC sub-block with tag `tag` (each entry's absolute offset in `d`).
fn bloc_entries(d: &[u8], tag: &[u8; 4]) -> Vec<usize> {
    let mut v = vec![];
    if d.get(..4) != Some(b"BLOC") {
        return v;
    }
    for i in 0..be32(d, 4).min(256) as usize {
        let o = be32(d, 0x10 + 4 * i) as usize;
        if d.get(o..o + 4) == Some(tag) {
            for k in 0..be32(d, o + 4).min(256) as usize {
                v.push(o + be32(d, o + 8 + 4 * k) as usize);
            }
        }
    }
    v
}

struct RawLine {
    a: usize,
    b: usize,
    kind: u16,
    mat_flags: u8,
    mat: u8,
    prev: i32,
    next: i32,
}

fn load() -> Result<Map, String> {
    let mut verts: Vec<[f32; 2]> = vec![];
    let mut raw: Vec<RawLine> = vec![];
    let mut ext = vec![];
    let mut doors = vec![];
    let mut door_at: Vec<(usize, usize)> = vec![]; // (section, GDOR index) per door
    let mut spawns = vec![];
    let mut models = vec![];
    let mut door_models = vec![];
    for (si, s) in SECS.iter().enumerate() {
        let leaves = pm::pac_leaves(s.file).ok_or("map archive not readable")?;
        // the section's main collision: the largest collision file (others are props bound to moving bones)
        let c = leaves.iter().filter(|d| is_coll(d)).max_by_key(|d| be16(d, 2)).ok_or("no collision")?;
        let (np, npl) = (be16(c, 0) as usize, be16(c, 2) as usize);
        let (po, pl) = (be32(c, 8) as usize, be32(c, 12) as usize);
        let v0 = verts.len();
        let l0 = raw.len() as i32;
        let mut e = [f32::MAX, f32::MIN, f32::MIN, f32::MAX];
        for i in 0..np {
            let p = [bef(c, po + 8 * i) + s.off[0], bef(c, po + 8 * i + 4) + s.off[1]];
            e = [e[0].min(p[0]), e[1].max(p[0]), e[2].max(p[1]), e[3].min(p[1])];
            verts.push(p);
        }
        ext.push(e);
        for i in 0..npl {
            let o = pl + 16 * i;
            let link = |k: usize| -> i32 {
                let v = be16(c, o + k) as i16 as i32;
                if v < 0 || v as usize >= npl { -1 } else { l0 + v }
            };
            raw.push(RawLine {
                a: v0 + be16(c, o) as usize,
                b: v0 + be16(c, o + 2) as usize,
                kind: be16(c, o + 12),
                mat_flags: c[o + 14],
                mat: c[o + 15],
                prev: link(4),
                next: link(6),
            });
        }
        // doors (GDOR) and enemy spawn points (GEG1)
        for b in leaves.iter().filter(|d| d.get(..4) == Some(b"BLOC")) {
            for (gi, e) in bloc_entries(b, b"GDOR").into_iter().enumerate() {
                doors.push(Door { sec: si, x: bef(b, e + 0x38) + s.off[0], y: bef(b, e + 0x3C) + s.off[1], kind: b[e + 0x44], to: -1 });
                door_at.push((si, gi));
            }
            for e in bloc_entries(b, b"GEG1") {
                spawns.push((si, bef(b, e + 0x28) + s.off[0], bef(b, e + 0x2C) + s.off[1], if b[e + 0x67] != 0 { 1 } else { -1 }));
            }
        }
        for (_, r, t) in pm::static_models(&leaves, &[s.model]) {
            models.push((r, t, s.off));
        }
        if si == 0 {
            for (_, r, t) in pm::static_models(&leaves, &["AdvGdtDoor01"]) {
                door_models.push((r, t));
            }
        }
    }
    for ((fs, fi), (ts, ti)) in LINKS {
        let from = door_at.iter().position(|&d| d == (fs, fi));
        let to = door_at.iter().position(|&d| d == (ts, ti));
        if let (Some(f), Some(t)) = (from, to) {
            doors[f].to = t as i32;
        }
    }
    // group the lines the way the retail format wants them (floor, ceiling, right wall, left wall), links remapped
    let group = |k: u16| match k & 0xF {
        1 => Some(0usize),
        2 => Some(1),
        4 => Some(2),
        8 => Some(3),
        _ => None,
    };
    let mut order: Vec<usize> = vec![];
    let mut ranges = [MapLineRange { start: 0, count: 0 }; 5];
    for g in 0..4 {
        ranges[g].start = order.len() as i16;
        for (i, l) in raw.iter().enumerate() {
            if group(l.kind) == Some(g) {
                order.push(i);
            }
        }
        ranges[g].count = order.len() as i16 - ranges[g].start;
    }
    ranges[4].start = order.len() as i16;
    let mut new_of = vec![-1i32; raw.len()];
    for (n, &i) in order.iter().enumerate() {
        new_of[i] = n as i32;
    }
    let remap = |i: i32| if i < 0 { -1 } else { new_of[i as usize] } as i16;
    let lines: Vec<MapLine> = order
        .iter()
        .map(|&i| {
            let l = &raw[i];
            let drop = l.mat_flags & 0x01 != 0;
            let ledge = l.mat_flags & 0x60 != 0;
            let kind = 1u16 << group(l.kind).unwrap_or(0);
            let terrain: u16 = match l.mat {
                3 => 2,  // grass (the jungle's ground)
                1 | 2 => 4, // wood
                _ => 0,
            };
            let mut m: MapLine = unsafe { core::mem::zeroed() };
            m.v0_idx = l.a as u16;
            m.v1_idx = l.b as u16;
            m.prev_id0 = remap(l.prev);
            m.next_id0 = remap(l.next);
            m.prev_id1 = remap(l.prev);
            m.next_id1 = remap(l.next);
            m.hi_flags = kind | if drop { 0x100 } else { 0 };
            m.lo_flags = terrain | if drop { 0x100 } else { 0 } | if ledge { 0x200 } else { 0 };
            m
        })
        .collect();
    let bounds = verts.iter().fold([f32::MAX, f32::MAX, f32::MIN, f32::MIN], |b, p| [b[0].min(p[0]), b[1].min(p[1]), b[2].max(p[0]), b[3].max(p[1])]);
    Ok(Map {
        verts: verts.iter().map(|p| Vec2 { x: p[0], y: p[1] }).collect(),
        lines,
        ranges,
        bounds,
        ext,
        doors,
        spawns,
        models,
        door_models,
    })
}

pub fn enabled() -> bool {
    static E: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *E.get_or_init(|| std::env::var("MELEE_ENVOY_EXPLORE").is_ok_and(|v| v == "1"))
}

/// The map, loaded once on a helper thread (system heap, outside the arena rollback rewrites). `None` when off or
/// without the owner's files.
pub fn map() -> Option<&'static Map> {
    static M: std::sync::OnceLock<Option<Map>> = std::sync::OnceLock::new();
    M.get_or_init(|| {
        if !enabled() {
            return None;
        }
        let t = std::time::Instant::now();
        let r = std::thread::spawn(load).join().unwrap_or_else(|_| Err("loader panicked".into()));
        match &r {
            Ok(m) => eprintln!(
                "explore: map loaded ({} sections, {} verts, {} lines, {} doors, {} spawns, {} tris) in {} ms",
                m.ext.len(), m.verts.len(), m.lines.len(), m.doors.len(), m.spawns.len(),
                m.models.iter().map(|(_, t, _)| t.len() / 3).sum::<usize>(), t.elapsed().as_millis()
            ),
            Err(e) => eprintln!("explore: no map ({e}); Final Destination as usual"),
        }
        r.ok()
    })
    .as_ref()
}

// ---------------------------------------------------------------- sim state (statics: snapshot covered)

/// Envoy's plan for the coming fight wants the room (set by `envoy::match_begin`; outside an Envoy run any Final
/// Destination match is the room).
pub static mut WANT: bool = false;
/// Whether the current stage is the exploration room (set at each stage load).
pub static mut ACTIVE: bool = false;
/// The party's section.
pub static mut SEC: i32 = 0;
/// Door transition clock (0 idle); the move happens at `FADE / 2`.
pub static mut FADE: i32 = 0;
/// Destination door of the running transition (-1 none).
pub static mut DEST: i32 = -1;
/// Per player: stick was up last frame (doors take a fresh tilt, as Brawl).
pub static mut UP_PREV: [bool; 6] = [false; 6];
/// Frames since the room started (spawns happen on frame 2).
pub static mut CLOCK: i32 = 0;
/// Doors used (logs, checks).
pub static mut DOORS_USED: i32 = 0;

const FADE_LEN: i32 = 24;

/// `ground::EXPLORE_COLL`: at every stage load, before the script layer and `mpLibLoad`. Returns the exploration
/// collision (a fresh copy in the game heap) for Final Destination with the mode on, else null (the stage's own).
pub unsafe fn coll_override() -> *mut MapCollData {
    ACTIVE = false;
    SEC = 0;
    FADE = 0;
    DEST = -1;
    UP_PREV = [false; 6];
    CLOCK = 0;
    DOORS_USED = 0;
    if sg::stage_info.grkind as i32 != 37 || (crate::tu::envoy::armed() != 0 && !WANT) {
        return core::ptr::null_mut();
    }
    let Some(m) = map() else { return core::ptr::null_mut() };
    let alloc = |n: usize| unsafe { melee_runtime::tu::memory::HSD_MemAlloc(n as _) };
    let cd = alloc(core::mem::size_of::<MapCollData>()) as *mut MapCollData;
    let vs = alloc(core::mem::size_of::<Vec2>() * m.verts.len()) as *mut Vec2;
    let ls = alloc(core::mem::size_of::<MapLine>() * m.lines.len()) as *mut MapLine;
    let js = alloc(core::mem::size_of::<MapJoint>()) as *mut MapJoint;
    if cd.is_null() || vs.is_null() || ls.is_null() || js.is_null() {
        eprintln!("explore: no heap for the collision; Final Destination as usual");
        return core::ptr::null_mut();
    }
    core::ptr::copy_nonoverlapping(m.verts.as_ptr(), vs, m.verts.len());
    core::ptr::copy_nonoverlapping(m.lines.as_ptr(), ls, m.lines.len());
    let mut j: MapJoint = core::mem::zeroed();
    j.ranges = m.ranges;
    j.left_bound = m.bounds[0];
    j.bottom_bound = m.bounds[1];
    j.right_bound = m.bounds[2];
    j.top_bound = m.bounds[3];
    j.vtx_start = 0;
    j.vtx_count = m.verts.len() as i16;
    *js = j;
    let mut c: MapCollData = core::mem::zeroed();
    c.verts = vs;
    c.vert_count = m.verts.len() as i32;
    c.lines = ls;
    c.line_count = m.lines.len() as i32;
    c.ranges = m.ranges;
    c.joints = js;
    c.joint_count = 1;
    *cd = c;
    ACTIVE = true;
    set_bounds(0);
    eprintln!("explore: room collision in place ({} lines)", m.lines.len());
    cd
}

/// The camera and blast bounds of section `s` (camera: the section's extent; blast: 120 units outside it).
unsafe fn set_bounds(s: usize) {
    let Some(m) = map() else { return };
    let Some(e) = m.ext.get(s) else { return };
    let ci = &mut sg::stage_info.cam_info;
    let (ox, oy) = (ci.cam_x_offset, ci.cam_y_offset);
    // the camera may look a little past the walls and well below the floor (Melee's camera aims above the
    // fighters; with the floor as its bottom bound the party would sit on the screen's lower edge)
    ci.cam_bounds.left = e[0] - 20.0 - ox;
    ci.cam_bounds.right = e[1] + 20.0 - ox;
    ci.cam_bounds.top = e[2] + 10.0 - oy;
    ci.cam_bounds.bottom = e[3] - 70.0 - oy;
    let bz = &mut sg::stage_info.blast_zone;
    bz.left = e[0] - 120.0 - ox;
    bz.right = e[1] + 120.0 - ox;
    bz.top = e[2] + 120.0 - oy;
    bz.bottom = e[3] - 120.0 - oy;
}

unsafe fn each_fighter(mut f: impl FnMut(*mut HSD_GObj, *mut Fighter)) {
    let mut g = *sg::HSD_GObjPLinkHead.offset(HSD_GOBJ_PLINK_FIGHTER as isize);
    while !g.is_null() {
        let next = (*g).next as *mut HSD_GObj;
        let fp = sg::HSD_GObjGetUserData(g) as *mut Fighter;
        if !fp.is_null() && (*fp).is_sleeping() == 0 && (*fp).motion_id > 13 {
            f(g, fp);
        }
        g = next;
    }
}

/// Move one fighter to a door (on its floor, falling the last few units, as the stage switch places fighters).
unsafe fn place(g: *mut HSD_GObj, fp: *mut Fighter, x: f32, y: f32) {
    (*fp).cur_pos = melee_types::Vec { x, y: y + 3.0, z: 0.0 };
    (*fp).prev_pos = (*fp).cur_pos;
    sg::ftCo_Fall_Enter(g as *mut Fighter_GObj);
    (*fp).self_vel = core::mem::zeroed();
    (*fp).x8c_kb_vel = core::mem::zeroed();
    (*fp).x98_atk_shield_kb = core::mem::zeroed();
    (*fp).gr_vel = 0.0;
    let cpu = (*fp).cpu;
    sg::script_switch_contacts(fp);
    (*fp).cpu = cpu;
    sg::ftCamera_80076064(fp);
}

/// `grlast::EXPLORE_FRAME`: once per logic frame of the room (Final Destination's stage proc).
pub unsafe fn frame() {
    if !ACTIVE {
        return;
    }
    let Some(m) = map() else { return };
    CLOCK += 1;
    if CLOCK == 1 {
        set_bounds(SEC as usize); // after the stage's own camera data has been read
    }
    if CLOCK == 2 {
        let mut n = 0;
        for (si, x, y, face) in m.spawns.iter().copied() {
            if si == 0 && n < 4 {
                // the room's ground spawn points only (the enemies drop onto the floor below their point)
                let h = sg::ScriptGame_SpawnEnemy(STUB_ENEMY, x.to_bits() as i32, (y + 5.0).to_bits() as i32, face, 0x4558_0000 + n);
                eprintln!("explore: enemy spawn at ({x:.0},{y:.0}) -> {h}");
                n += 1;
            }
        }
    }
    if CLOCK % 10 == 0 && std::env::var_os("MELEE_EXPLORE_LOG").is_some() {
        each_fighter(|_, fp| {
            let p = (*fp).cur_pos;
            eprintln!("explore: frame {} player {} at ({:.1},{:.1}) ground {} section {}", CLOCK, (*fp).player_idx, p.x, p.y, ((*fp).ground_or_air as i32 == GA_Ground as i32) as i32, SEC);
        });
    }
    if FADE > 0 {
        FADE += 1;
        if FADE == FADE_LEN / 2 && DEST >= 0 {
            let d = m.doors[DEST as usize];
            let mut k = 0;
            each_fighter(|g, fp| {
                // the party stands side by side at the destination door
                place(g, fp, d.x + if k % 2 == 0 { -6.0 } else { 6.0 } * ((k / 2 + 1) as f32), d.y);
                k += 1;
            });
            SEC = d.sec as i32;
            set_bounds(d.sec);
            DOORS_USED += 1;
            eprintln!("explore: through a door to section {} ({:.0},{:.0}) at room frame {}", d.sec, d.x, d.y, CLOCK);
        }
        if FADE >= FADE_LEN {
            FADE = 0;
            DEST = -1;
        }
        return;
    }
    // a fresh up tilt on the ground inside a door's area opens it (Brawl: 15 x 20 around the door's foot; air doors hang
    // up to ~50 above their floor, so the area reaches 55 below the door)
    let mut want = -1;
    let mut idx = 0usize;
    each_fighter(|_, fp| {
        let up = (*fp).input.lstick[0].y > 0.7;
        let fresh = up && !UP_PREV[idx.min(5)];
        UP_PREV[idx.min(5)] = up;
        idx += 1;
        if !fresh || (*fp).ground_or_air as i32 != GA_Ground as i32 || want >= 0 {
            return;
        }
        let p = (*fp).cur_pos;
        for (di, d) in m.doors.iter().enumerate() {
            if d.sec as i32 == SEC && (p.x - d.x).abs() <= 15.0 && p.y >= d.y - 55.0 && p.y <= d.y + 20.0 {
                want = di as i32;
            }
        }
    });
    if want >= 0 {
        let d = m.doors[want as usize];
        DEST = if d.to >= 0 {
            d.to
        } else {
            // the exit (arrow door): back to the first door of section 0 here; an Envoy run ends the room instead
            m.doors.iter().position(|o| o.sec == 0 && o.to >= 0).map(|i| i as i32).unwrap_or(-1)
        };
        if DEST >= 0 {
            FADE = 1;
            eprintln!("explore: door {want} opened (type {}) at room frame {}", d.kind, CLOCK);
        }
    }
}

/// `grdisplay::HIDE_GROUND`: Final Destination's own model stays hidden in the room.
pub fn hide_ground() -> bool {
    unsafe { ACTIVE }
}

/// `ftdrawcommon::MAP_DRAW`: the section models, the doors and the transition fade, once per frame (player 1's
/// opaque pass). Presentation only.
pub unsafe fn draw(g: *mut HSD_GObj, pass: i32) {
    if !ACTIVE || pass != 0 {
        return;
    }
    // once per frame: in the first in-play fighter's pass
    let mut first: *mut HSD_GObj = core::ptr::null_mut();
    each_fighter(|fg, _| {
        if first.is_null() {
            first = fg;
        }
    });
    if first != g {
        return;
    }
    let Some(m) = map() else { return };
    let _prof = melee_core::frameprof::Span::new(melee_core::frameprof::Kind::Fighter, || melee_core::frameprof::Text::new().s("explore map draw"));
    for (r, t, off) in &m.models {
        pm::draw_tris(r, t, [0.0, 0.0], [off[0], off[1], 0.0], 1.0, 0.0, false);
    }
    if let Some((r, t)) = m.door_models.first() {
        for d in &m.doors {
            pm::draw_tris(r, t, [0.0, 0.0], [d.x, d.y, 0.0], 1.0, 0.0, false);
        }
    }
    if FADE > 0 {
        let a = 1.0 - ((FADE - FADE_LEN / 2) as f32).abs() / (FADE_LEN / 2) as f32;
        pm::draw_fade((a.clamp(0.0, 1.0) * 255.0) as u8);
    }
}

/// Install the hooks (the engine start). Nothing is hooked unless the mode is on.
pub fn install() {
    if !enabled() {
        return;
    }
    unsafe {
        melee_world::tu::ground::EXPLORE_COLL = Some(coll_override);
        melee_world::tu::grlast::EXPLORE_FRAME = Some(frame);
        melee_world::tu::grdisplay::HIDE_GROUND = Some(hide_ground);
        melee_fighter::tu::ftdrawcommon::MAP_DRAW = Some(draw);
    }
    // load now, before the first match (the loader is a helper thread; the first stage load must not wait on it)
    let _ = map();
}
