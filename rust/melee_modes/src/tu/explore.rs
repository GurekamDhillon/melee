//! Envoy exploration rooms (notes/envoy-exploration.md): a Brawl Subspace Emissary map section, read at run time from
//! the owner's own files, played in place of Final Destination's geometry, with Brawl-style doors (up on the stick)
//! that move the party between map sections.
//!
//! Its own stage slot (notes/sse-stage.md): the GD stage id `GD_STAGE_SSE_LAKE3` (the GD range this loader owns,
//! outside the retail and m-ex external ids). The stage select shows it as an extra icon when the owner's Brawl files
//! are present (hidden otherwise); picking it plays the room on a Final Destination carrier (`carrier`), so the rest
//! of the game only ever sees a retail stage id. Final Destination itself is untouched. Envoy rooms ask for the same id
//! (`envoy::match_begin`, opt-in with `MELEE_ENVOY_EXPLORE=1`). Vanilla play, the goldens and the gate never reach this.
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
use melee_core::ppc::Ppc;
use melee_types::*;
use std::vec::Vec; // melee_types has its own `Vec` (the 3D vector): `melee_types::Vec` below

/// One map section: an adventure archive placed at `off` in the shared world (sections never overlap).
struct SecDef {
    file: &'static str,
    off: [f32; 2],
    model: &'static str,
    /// 0: the step's main section; 1, 2: its side rooms `a`, `b` (Brawl's room files)
    room: u8,
}

/// The map (notes/sse-stage.md: The Lake, step 3; Brawl's own model names call the Diddy arc "Jungle"): `080103`, a
/// short, mostly flat corridor with a raised slab and a ledge, its doors into the side rooms `080103a` (the hut) and
/// `080103b`, and an arrow door (the section's exit). Door targets come from the doors' own data (`link_doors`).
const SECS: [SecDef; 3] = [
    SecDef { file: "080103.pac", off: [0.0, 0.0], model: "081mAdvJungle03", room: 0 },
    SecDef { file: "080103a.pac", off: [1500.0, 0.0], model: "081mAdvJungle03a", room: 1 },
    SecDef { file: "080103b.pac", off: [3000.0, 0.0], model: "081mAdvJungle03b", room: 2 },
];
/// Petey's arena (the boss fight, `ARENA_WANT`): the adventure stage of his fight, one section at the origin.
const ARENA_SECS: [SecDef; 1] = [SecDef { file: "030101.pac", off: [0.0, 0.0], model: "030vAdvStadium", room: 0 }];

/// Brawl's enemy kind (GEG1 +0x1C; names: BrawlCrate's public `EnemyList`, ids matched to the per-file counts in
/// notes/sse-audit.md) -> the nearest of our four Subspace kinds (`subspace::K_*`), or None for a boss (not spawned).
/// Unknown ids fall back to the Goomba (logged).
fn enemy_kind(id: u16) -> (Option<usize>, &'static str) {
    use crate::tu::subspace::{K_BILL, K_GIANT, K_GOOMBA, K_PRIMID};
    match id {
        0 => (Some(K_GOOMBA), "Goomba"),
        1 => (Some(K_GOOMBA), "Poppant"),
        2 => (Some(K_GOOMBA), "Feyesh"),
        3 => (Some(K_GOOMBA), "Jyk"),
        4 => (Some(K_BILL), "Auroros"), // a diving bird: flies straight at the party
        7 => (Some(K_GIANT), "Borboras"),
        8 => (Some(K_GIANT), "Giant Goomba"),
        10 | 16 => (Some(K_GOOMBA), "Bucculus/Mite"),
        13 => (Some(K_BILL), "Bullet Bill"),
        15 => (Some(K_PRIMID), "Spaak"),
        17 => (Some(K_GOOMBA), "Ticken"),
        18 => (Some(K_GIANT), "Towtow"),
        19 => (Some(K_PRIMID), "Hammer Bro"),
        23 => (Some(K_PRIMID), "Primid"),
        25 => (Some(K_GOOMBA), "Koopa"),
        32 => (Some(K_PRIMID), "Boomerang Primid"),
        35 => (Some(K_PRIMID), "Sword Primid"),
        45 => (Some(K_GOOMBA), "Glunder"),
        53 => (None, "Rayquaza (boss)"),
        _ => (Some(K_GOOMBA), "unmapped"),
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Door {
    pub sec: usize,
    pub x: f32,
    pub y: f32,
    /// GDOR door type (0 normal, 1 yellow, 2 arrow, ...)
    pub kind: u8,
    /// index into `Map::doors` of the destination; -1: the exit (or unlinked)
    pub to: i32,
    /// the door's own id (GDOR +0x30: level, step, room, index)
    pub id: u32,
    /// GDOR +0x34: set on a side room's way back to its main section
    pub back: bool,
    /// GDOR +0x51 == 0x13: a main-section door into a side room
    pub into_room: bool,
    /// the level's exit (the door the GMSJ block names, else an arrow door)
    pub exit: bool,
}

/// A GIB2 crate generator (section, x, y, Brawl's size: 0 small, 1 crate, 2 big) -> a Melee container item.
#[derive(Clone, Copy, Debug)]
pub struct Crate {
    pub sec: usize,
    pub x: f32,
    pub y: f32,
    pub size: u32,
}

pub struct Map {
    verts: Vec<Vec2>,
    lines: Vec<MapLine>,
    ranges: [MapLineRange; 5],
    bounds: [f32; 4],
    /// collision joints: 0 the static map, k block k-1
    joints: Vec<MapJoint>,
    pub blocks: Vec<Block>,
    /// per section: collision extent (left, right, top, bottom), world space
    pub ext: Vec<[f32; 4]>,
    pub doors: Vec<Door>,
    /// enemy spawn points (section, x, y, facing, Brawl kind)
    pub spawns: Vec<(usize, f32, f32, i32, u16)>,
    pub crates: Vec<Crate>,
    /// per section: (rig, posed triangles, offset); presentation only
    models: Vec<(pm::Rig, Vec<pm::Vert>, [f32; 2])>,
    door_models: Vec<(pm::Rig, Vec<pm::Vert>)>,
    /// From the stage position model (arena only): camera limit and blast zones (left, right, top, bottom) and the
    /// player spawn points.
    pub cam: Option<[f32; 4]>,
    pub blast: Option<[f32; 4]>,
    pub players: Vec<[f32; 2]>,
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
    /// collision joint: 0 the static map, k the k-th breakable block
    joint: usize,
}

/// A breakable block (Brawl GBLK brick): its own collision joint (index in `Map::blocks` + 1), centre and half size.
#[derive(Clone, Copy, Debug)]
pub struct Block {
    pub sec: usize,
    pub x: f32,
    pub y: f32,
    pub hw: f32,
    pub hh: f32,
}

fn load() -> Result<Map, String> {
    let mut m = load_secs(&SECS, true, true)?;
    link_doors(&mut m, &SECS);
    Ok(m)
}

/// Door targets from Brawl's own door data (notes/sse-stage.md, "Doors"): a main-section door flagged as a room door
/// leads into a side room (the first such door into `a`, the next into `b`; two of them share one spot in `080103`
/// and the second takes over once the first room has been seen, see `room_tick`); a side room's way-back door
/// returns to the main door that led in; the exit (GMSJ, else an arrow door) has no target.
fn link_doors(m: &mut Map, secs: &[SecDef]) {
    let room_sec = |r: u8| secs.iter().position(|s| s.room == r);
    let mut next_room = 1u8;
    for i in 0..m.doors.len() {
        let d = m.doors[i];
        if secs[d.sec].room != 0 || !d.into_room || d.exit {
            continue;
        }
        let Some(rs) = room_sec(next_room) else { continue };
        next_room += 1;
        if let Some(t) = m.doors.iter().position(|o| o.sec == rs && o.back) {
            m.doors[i].to = t as i32;
            if m.doors[t].to < 0 {
                m.doors[t].to = i as i32;
            }
        }
    }
}

/// `whole`: every collision file of a section, welded (the arena's centre floor is its own file).
/// `objects`: the map's way: every collision object by its bone (world objects static, bricks as block joints).
fn load_secs(secs: &[SecDef], whole: bool, objects: bool) -> Result<Map, String> {
    let mut braw: Vec<RawLine> = vec![];
    let mut bverts: Vec<[f32; 2]> = vec![];
    let mut blocks: Vec<Block> = vec![];
    let mut verts: Vec<[f32; 2]> = vec![];
    let mut raw: Vec<RawLine> = vec![];
    let mut ext = vec![];
    let mut doors = vec![];
    let mut spawns = vec![];
    let mut crates = vec![];
    let mut exits: Vec<u32> = vec![];
    let mut models = vec![];
    let mut door_models = vec![];
    let mut cam = None;
    let mut blast = None;
    let mut players = vec![];
    for (si, s) in secs.iter().enumerate() {
        let Some(leaves) = pm::pac_leaves(s.file) else {
            if si > 0 {
                // a missing side room: the main section still plays (its doors into it stay shut)
                ext.push([0.0; 4]);
                continue;
            }
            return Err("map archive not readable".into());
        };
        let colls: Vec<&Vec<u8>> = leaves.iter().filter(|d| is_coll(d)).collect();
        if colls.is_empty() {
            return Err("no collision".into());
        }
        let mut e = [f32::MAX, f32::MIN, f32::MIN, f32::MAX];
        if objects {
            // every collision file of the section, object by object (a collision object names the bone it rides):
            // world objects (no bone) are the static map, welded below; a brick (`AdvGdtPunBrick*`, Brawl's GBLK
            // block) at its bone's rest place becomes its own collision joint, so it can break; an item box top
            // (`AdvGdtItemBox*`) is the GIB2 crate's own, and the crate is a Melee item here; the rest ride moving
            // parts (GMOV) the port does not move yet, so they stay out.
            for c in &colls {
                let (pl, oo, nobj) = (be32(c, 12) as usize, be32(c, 0x10) as usize, be16(c, 4) as usize);
                let po = be32(c, 8) as usize;
                for k in 0..nobj {
                    let o = oo + 0x6C * k;
                    let (p0, pc) = (be16(c, o) as usize, be16(c, o + 2) as usize);
                    let bone = be16(c, o + 0x2A);
                    let name_end = c.get(o + 0x4C..o + 0x6C).and_then(|n| n.iter().position(|&x| x == 0)).unwrap_or(32);
                    let name = c.get(o + 0x4C..o + 0x4C + name_end).unwrap_or(&[]);
                    let (place, joint) = if bone == 0xFFFF {
                        ([s.off[0], s.off[1]], 0usize)
                    } else if name.starts_with(b"AdvGdtPunBrick") {
                        match leaves.iter().find_map(|b| pm::bone_trans(b, name)) {
                            Some(t) => {
                                let bb = [bef(c, o + 0x14), bef(c, o + 0x18), bef(c, o + 0x1C), bef(c, o + 0x20)];
                                let (x, y) = (t[0] + s.off[0] + (bb[0] + bb[2]) / 2.0, t[1] + s.off[1] + (bb[1] + bb[3]) / 2.0);
                                blocks.push(Block { sec: si, x, y, hw: (bb[2] - bb[0]) / 2.0, hh: (bb[3] - bb[1]) / 2.0 });
                                ([t[0] + s.off[0], t[1] + s.off[1]], blocks.len())
                            }
                            None => {
                                eprintln!("explore: section {si}: a brick's bone not found; left out");
                                continue;
                            }
                        }
                    } else {
                        if !name.starts_with(b"AdvGdtItemBox") {
                            eprintln!("explore: section {si}: a collision object riding a moving part left out ({} lines)", pc);
                        }
                        continue;
                    };
                    // the object's planes, their points copied (an object's points are its own)
                    let mut vmap: Vec<(usize, usize)> = vec![];
                    let l0 = if joint == 0 { raw.len() } else { braw.len() };
                    for i in p0..p0 + pc {
                        let q = pl + 16 * i;
                        let mut vid = |pi: usize, verts: &mut Vec<[f32; 2]>, bverts: &mut Vec<[f32; 2]>, e: &mut [f32; 4]| -> usize {
                            if let Some(&(_, n)) = vmap.iter().find(|(o, _)| *o == pi) {
                                return n;
                            }
                            let p = [bef(c, po + 8 * pi) + place[0], bef(c, po + 8 * pi + 4) + place[1]];
                            let n = if joint == 0 {
                                *e = [fmin(e[0], p[0]), fmax(e[1], p[0]), fmax(e[2], p[1]), fmin(e[3], p[1])];
                                verts.push(p);
                                verts.len() - 1
                            } else {
                                bverts.push(p);
                                bverts.len() - 1
                            };
                            vmap.push((pi, n));
                            n
                        };
                        let a = vid(be16(c, q) as usize, &mut verts, &mut bverts, &mut e);
                        let b = vid(be16(c, q + 2) as usize, &mut verts, &mut bverts, &mut e);
                        let link = |k: usize| -> i32 {
                            let v = be16(c, q + k) as i16 as i32;
                            if v < p0 as i32 || v as usize >= p0 + pc { -1 } else { (l0 + v as usize - p0) as i32 }
                        };
                        let l = RawLine { a, b, kind: be16(c, q + 12), mat_flags: c[q + 14], mat: c[q + 15], prev: link(4), next: link(6), joint };
                        if joint == 0 {
                            raw.push(l);
                        } else {
                            braw.push(l);
                        }
                    }
                }
            }
        }
        let colls = if objects { vec![] } else { colls };
        for c in colls {
        let (np, npl) = (be16(c, 0) as usize, be16(c, 2) as usize);
        let (po, pl) = (be32(c, 8) as usize, be32(c, 12) as usize);
        let v0 = verts.len();
        let l0 = raw.len() as i32;
        for i in 0..np {
            let p = [bef(c, po + 8 * i) + s.off[0], bef(c, po + 8 * i + 4) + s.off[1]];
            e = [fmin(e[0], p[0]), fmax(e[1], p[0]), fmax(e[2], p[1]), fmin(e[3], p[1])];
            verts.push(p);
        }
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
                joint: 0,
            });
        }
        }
        ext.push(e);
        if whole {
            for b in leaves.iter() {
                if let (Some(a), Some(z)) = (pm::bone_trans(b, b"CamLimit0N"), pm::bone_trans(b, b"CamLimit1N")) {
                    cam = Some([fmin(a[0], z[0]) + s.off[0], fmax(a[0], z[0]) + s.off[0], fmax(a[1], z[1]) + s.off[1], fmin(a[1], z[1]) + s.off[1]]);
                    for k in 1..=4 {
                        if let Some(p) = pm::bone_trans(b, format!("Player{k}N").as_bytes()) {
                            players.push([p[0] + s.off[0], p[1] + s.off[1]]);
                        }
                    }
                }
                if blast.is_none() {
                    blast = pm::arena_blast(b);
                }
            }
        }
        // doors (GDOR) and enemy spawn points (GEG1)
        for b in leaves.iter().filter(|d| d.get(..4) == Some(b"BLOC")) {
            for e in bloc_entries(b, b"GDOR") {
                doors.push(Door {
                    sec: si,
                    x: bef(b, e + 0x38) + s.off[0],
                    y: bef(b, e + 0x3C) + s.off[1],
                    kind: b[e + 0x44],
                    to: -1,
                    id: be32(b, e + 0x30),
                    back: b.get(e + 0x34) == Some(&1),
                    into_room: b.get(e + 0x51) == Some(&0x13),
                    exit: false,
                });
            }
            for e in bloc_entries(b, b"GEG1") {
                spawns.push((si, bef(b, e + 0x28) + s.off[0], bef(b, e + 0x2C) + s.off[1], if b[e + 0x67] != 0 { 1 } else { -1 }, be16(b, e + 0x1C)));
            }
            // the level-exit block (GMSJ) carries a copy of its exit door: that door's id
            for e in bloc_entries(b, b"GMSJ") {
                if b.get(e + 0xC..e + 0x10) == Some(b"GDOR") {
                    let inner = e + 0xC;
                    let first = inner + be32(b, inner + 8) as usize;
                    exits.push(be32(b, first + 0x30));
                }
            }
            for e in bloc_entries(b, b"GIB2") {
                crates.push(Crate { sec: si, x: bef(b, e + 0x30) + s.off[0], y: bef(b, e + 0x34) + s.off[1], size: be32(b, e + 0x38) });
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
    for d in doors.iter_mut() {
        d.exit = if exits.is_empty() { d.kind == 2 && !d.back } else { exits.contains(&d.id) };
    }
    if whole {
        // weld: one vertex per position, and each line's neighbours by shared vertices (the files' own links stop at
        // their file's edge, which would leave mid-floor ledges where two files meet)
        let mut uniq: Vec<[f32; 2]> = vec![];
        let mut map_v = vec![0usize; verts.len()];
        for (i, p) in verts.iter().enumerate() {
            map_v[i] = match uniq.iter().position(|q| q[0] == p[0] && q[1] == p[1]) {
                Some(k) => k,
                None => {
                    uniq.push(*p);
                    uniq.len() - 1
                }
            };
        }
        verts = uniq;
        for l in raw.iter_mut() {
            l.a = map_v[l.a];
            l.b = map_v[l.b];
        }
        let n = raw.len();
        for i in 0..n {
            let (a, b, k) = (raw[i].a, raw[i].b, raw[i].kind & 0xF);
            raw[i].next = (0..n).find(|&j| j != i && raw[j].a == b && raw[j].kind & 0xF == k).or_else(|| (0..n).find(|&j| j != i && raw[j].a == b)).map(|j| j as i32).unwrap_or(-1);
            raw[i].prev = (0..n).find(|&j| j != i && raw[j].b == a && raw[j].kind & 0xF == k).or_else(|| (0..n).find(|&j| j != i && raw[j].b == a)).map(|j| j as i32).unwrap_or(-1);
        }
    }
    // the blocks' own lines and points after the static map's (each block a joint of its own, never welded)
    let nstatic_v = verts.len();
    let l_off = raw.len();
    verts.extend(bverts.iter().copied());
    for mut l in braw.drain(..) {
        l.a += nstatic_v;
        l.b += nstatic_v;
        l.prev = if l.prev < 0 { -1 } else { l.prev + l_off as i32 };
        l.next = if l.next < 0 { -1 } else { l.next + l_off as i32 };
        raw.push(l);
    }
    // whether another floor line than `i` has vertex `v` as an end
    let floor_at = |v: usize, i: usize| raw.iter().enumerate().any(|(j, o)| j != i && o.kind & 0xF == 1 && (o.a == v || o.b == v));
    // group the lines the way the retail format wants them (floor, ceiling, right wall, left wall), links remapped
    let group = |k: u16| match k & 0xF {
        1 => Some(0usize),
        2 => Some(1),
        4 => Some(2),
        8 => Some(3),
        _ => None,
    };
    // within a group the joints follow each other (static map first), so each joint's lines of a group are one run
    let nj = blocks.len() + 1;
    let mut order: Vec<usize> = vec![];
    let mut ranges = [MapLineRange { start: 0, count: 0 }; 5];
    let mut jranges = vec![[MapLineRange { start: 0, count: 0 }; 5]; nj];
    for g in 0..4 {
        ranges[g].start = order.len() as i16;
        for j in 0..nj {
            jranges[j][g].start = order.len() as i16;
            for (i, l) in raw.iter().enumerate() {
                if group(l.kind) == Some(g) && l.joint == j {
                    order.push(i);
                }
            }
            jranges[j][g].count = order.len() as i16 - jranges[j][g].start;
        }
        ranges[g].count = order.len() as i16 - ranges[g].start;
    }
    ranges[4].start = order.len() as i16;
    for jr in jranges.iter_mut() {
        jr[4].start = order.len() as i16;
    }
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
            // welded arena: a floor line is a ledge where one of its ends meets no other floor (the outer ends of the
            // main floor, where it turns down into a wall); a weld seam (floor on both sides) never is. Elsewhere the
            // file's own ledge flag.
            let ledge = if whole { !drop && group(l.kind) == Some(0) && (!floor_at(l.a, i) || !floor_at(l.b, i)) } else { l.mat_flags & 0x60 != 0 };
            let kind: u16 = match group(l.kind).unwrap_or(0) {
                0 => 1,
                1 => 2,
                2 => 4,
                _ => 8,
            };
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
    let bounds = verts.iter().fold([f32::MAX, f32::MAX, f32::MIN, f32::MIN], |b, p| [fmin(b[0], p[0]), fmin(b[1], p[1]), fmax(b[2], p[0]), fmax(b[3], p[1])]);
    // one MapJoint per joint: the static map (its points first), then one per block (its four points)
    let mut joints: Vec<MapJoint> = vec![];
    let mut vstart = 0usize;
    for j in 0..nj {
        // a block's points follow the previous joint's, so its last point is the highest one its lines use
        let vend = if j == 0 { nstatic_v } else { raw.iter().filter(|l| l.joint == j).map(|l| l.a.max(l.b) + 1).max().unwrap_or(vstart) };
        let mut mj: MapJoint = unsafe { core::mem::zeroed() };
        mj.ranges = jranges[j];
        let vs = &verts[vstart..vend.min(verts.len())];
        let b = if vs.is_empty() { bounds } else { vs.iter().fold([f32::MAX, f32::MAX, f32::MIN, f32::MIN], |b, p| [fmin(b[0], p[0]), fmin(b[1], p[1]), fmax(b[2], p[0]), fmax(b[3], p[1])]) };
        mj.left_bound = b[0];
        mj.bottom_bound = b[1];
        mj.right_bound = b[2];
        mj.top_bound = b[3];
        mj.vtx_start = vstart as i16;
        mj.vtx_count = (vend - vstart) as i16;
        joints.push(mj);
        vstart = vend;
    }
    Ok(Map {
        verts: verts.iter().map(|p| Vec2 { x: p[0], y: p[1] }).collect(),
        lines,
        ranges,
        bounds,
        joints,
        blocks,
        ext,
        doors,
        spawns,
        crates,
        models,
        door_models,
        cam,
        blast,
        players,
    })
}

/// The GD stage id range this loader owns (external stage ids; retail stops at 0x20s, m-ex starts at 288).
pub const GD_STAGE_FIRST: i32 = 0x7000;
pub const GD_STAGE_LAST: i32 = 0x70FF;
/// The Lake, step 3 (+ its side rooms), from the owner's own Brawl files.
pub const GD_STAGE_SSE_LAKE3: i32 = 0x7001;
/// Final Destination's external stage id: the carrier of every GD stage (its stage proc, camera and music).
pub const FD_STKIND: i32 = 0x20;

/// A GD stage slot: id, name (the stage select's label), the archives it needs.
pub struct GdStage {
    pub id: i32,
    pub name: &'static str,
    pub files: &'static [&'static str],
}
/// The GD stage table (the stage select appends one icon per available row).
pub const GD_STAGES: [GdStage; 1] = [GdStage { id: GD_STAGE_SSE_LAKE3, name: "The Lake 3 (SSE)", files: &["080103.pac", "080103a.pac"] }];

pub fn is_gd_stage(id: i32) -> bool {
    (GD_STAGE_FIRST..=GD_STAGE_LAST).contains(&id)
}

/// Whether GD stage `id` can be played: its archives are in the owner's Brawl folder (`MELEE_BRAWL_ABSENT` hides it).
pub fn available(id: i32) -> bool {
    GD_STAGES.iter().any(|g| g.id == id && g.files.iter().all(|f| pm::adventure_present(f)))
}

pub fn gd_name(id: i32) -> Option<&'static str> {
    GD_STAGES.iter().find(|g| g.id == id).map(|g| g.name)
}

/// The VS match about to start asked for the SSE stage (set by the stage select / `--match ...,sse`, read at the
/// stage load). Plain static: set before the match, constant through it.
pub static mut PICK: bool = false;

/// The retail stage id that carries stage `id`: a GD stage -> Final Destination (and the pick is noted); any other
/// id -> itself (and any earlier pick is cleared). An unavailable GD stage falls back to Final Destination itself.
pub fn carrier(id: i32) -> i32 {
    unsafe {
        PICK = is_gd_stage(id) && id == GD_STAGE_SSE_LAKE3 && available(id);
        if PICK {
            eprintln!("explore: stage {id:#x} ({}) on the Final Destination carrier", gd_name(id).unwrap_or("?"));
        }
    }
    if is_gd_stage(id) { FD_STKIND } else { id }
}

/// Envoy's opt-in for its rooms (`MELEE_ENVOY_EXPLORE=1`).
pub fn enabled() -> bool {
    static E: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *E.get_or_init(|| std::env::var("MELEE_ENVOY_EXPLORE").is_ok_and(|v| v == "1"))
}

/// The map, loaded once on a helper thread (system heap, outside the arena rollback rewrites). `None` when off or
/// without the owner's files.
pub fn map() -> Option<&'static Map> {
    static M: std::sync::OnceLock<Option<Map>> = std::sync::OnceLock::new();
    M.get_or_init(|| {
        if !available(GD_STAGE_SSE_LAKE3) {
            return None;
        }
        let t = std::time::Instant::now();
        let r = std::thread::spawn(load).join().unwrap_or_else(|_| Err("loader panicked".into()));
        match &r {
            Ok(m) => eprintln!(
                "explore: map loaded ({} sections, {} verts, {} lines, {} doors, {} spawns, {} crates, {} blocks, {} tris) in {} ms",
                m.ext.len(), m.verts.len(), m.lines.len(), m.doors.len(), m.spawns.len(), m.crates.len(), m.blocks.len(),
                m.models.iter().map(|(_, t, _)| t.len() / 3).sum::<usize>(), t.elapsed().as_millis()
            ),
            Err(e) => eprintln!("explore: no map ({e}); Final Destination as usual"),
        }
        if let Ok(m) = &r {
            for (i, d) in m.doors.iter().enumerate() {
                eprintln!("explore: door {i} section {} at ({:.0},{:.0}) type {} to {}", d.sec, d.x, d.y, d.kind, d.to);
            }
        }
        r.ok()
    })
    .as_ref()
}

/// Petey's arena, loaded once on a helper thread; `None` without the owner's files.
pub fn arena() -> Option<&'static Map> {
    static M: std::sync::OnceLock<Option<Map>> = std::sync::OnceLock::new();
    M.get_or_init(|| {
        let r = std::thread::spawn(|| load_secs(&ARENA_SECS, true, false)).join().unwrap_or_else(|_| Err("loader panicked".into()));
        match &r {
            Ok(m) => eprintln!(
                "explore: Petey arena loaded ({} verts, {} lines, {} players, camera {}, blast {})",
                m.verts.len(), m.lines.len(), m.players.len(), m.cam.is_some(), m.blast.is_some()
            ),
            Err(e) => eprintln!("explore: no Petey arena ({e}); Final Destination"),
        }
        r.ok()
    })
    .as_ref()
}

/// The map of the current stage: the arena in the Petey fight, else the exploration room.
fn cur() -> Option<&'static Map> {
    if unsafe { ARENA } { arena() } else { map() }
}

// ---------------------------------------------------------------- sim state (statics: snapshot covered)

/// Envoy's plan for the coming fight wants the room (set by `envoy::match_begin`; outside an Envoy run `PICK`).
pub static mut WANT: bool = false;
/// Whether the current stage is the exploration room (set at each stage load).
pub static mut ACTIVE: bool = false;
/// Envoy's coming fight is Petey's: play it in his arena (set by `envoy::match_begin`).
pub static mut ARENA_WANT: bool = false;
/// The current stage is Petey's arena (set at each stage load; implies `ACTIVE`).
pub static mut ARENA: bool = false;
/// The party's section.
pub static mut SEC: i32 = 0;
/// Sections the party has entered (bit per section; a shared door spot opens the room not yet seen first).
pub static mut VISITED: u32 = 0;
/// Blocks broken (bit per block; its collision joint is off).
pub static mut BROKEN: u64 = 0;
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

/// door transition: FADE_IN frames to black, the party moves, HOLD frames of black while Melee's camera travels to the
/// new section (it eases, it does not cut), then FADE_IN frames back
const FADE_IN: i32 = 12;
const HOLD: i32 = 40;
const FADE_LEN: i32 = 2 * FADE_IN + HOLD;

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
    VISITED = 1;
    BROKEN = 0;
    ARENA = false;
    if sg::stage_info.grkind as i32 != 37 {
        return core::ptr::null_mut();
    }
    let m = if ARENA_WANT && crate::tu::envoy::armed() != 0 {
        let Some(a) = arena() else { return core::ptr::null_mut() };
        ARENA = true;
        a
    } else {
        let want = if crate::tu::envoy::armed() != 0 { WANT } else { PICK };
        if !want {
            return core::ptr::null_mut();
        }
        let Some(m) = map() else { return core::ptr::null_mut() };
        m
    };
    let alloc = |n: usize| unsafe { melee_runtime::tu::memory::HSD_MemAlloc(n as _) };
    let cd = alloc(core::mem::size_of::<MapCollData>()) as *mut MapCollData;
    let vs = alloc(core::mem::size_of::<Vec2>() * m.verts.len()) as *mut Vec2;
    let ls = alloc(core::mem::size_of::<MapLine>() * m.lines.len()) as *mut MapLine;
    let js = alloc(core::mem::size_of::<MapJoint>() * m.joints.len().max(1)) as *mut MapJoint;
    if cd.is_null() || vs.is_null() || ls.is_null() || js.is_null() {
        eprintln!("explore: no heap for the collision; Final Destination as usual");
        ARENA = false;
        return core::ptr::null_mut();
    }
    core::ptr::copy_nonoverlapping(m.verts.as_ptr(), vs, m.verts.len());
    core::ptr::copy_nonoverlapping(m.lines.as_ptr(), ls, m.lines.len());
    core::ptr::copy_nonoverlapping(m.joints.as_ptr(), js, m.joints.len());
    let mut c: MapCollData = core::mem::zeroed();
    c.verts = vs;
    c.vert_count = m.verts.len() as i32;
    c.lines = ls;
    c.line_count = m.lines.len() as i32;
    c.ranges = m.ranges;
    c.joints = js;
    c.joint_count = m.joints.len() as i32;
    *cd = c;
    ACTIVE = true;
    set_bounds(0);
    eprintln!("explore: {} collision in place ({} lines)", if ARENA { "Petey arena" } else { "room" }, m.lines.len());
    cd
}

/// How far past a section's extent its blast zones lie.
const BLAST_MARGIN: f32 = 120.0;

/// Screen KOs: a section is walled in (`080103`'s corridor and both side rooms are closed boxes), so a launched
/// fighter hits a wall or the ceiling long before any blast zone and nobody is ever KO'd: an Envoy fight in it ran
/// past 24000 frames with the foe at 443% (notes/sse-stage.md, "Envoy"). Brawl's SSE KOs a fighter that is sent off
/// the screen; here, a fighter in a damage flight whose knockback would carry it past where the blast zone would be
/// without the walls (Melee's knockback decays 0.051 a frame: distance v^2 / 0.102) is put just past the real blast
/// zone on that side, and Melee's own KO (and its star / side effects) takes it from there.
unsafe fn screen_kos(m: &Map) {
    let Some(e) = m.ext.get(SEC as usize).copied() else { return };
    each_fighter(|_, fp| {
        let ms = (*fp).motion_id as i32;
        if !(ftCo_MS_DamageFlyHi as i32..=ftCo_MS_DamageFlyRoll as i32).contains(&ms) {
            return;
        }
        let v = (*fp).x8c_kb_vel;
        let p = (*fp).cur_pos;
        let reach = |s: f32| s * s / 0.102;
        let side = if v.x > 0.0 { e[1] + BLAST_MARGIN - p.x } else { p.x - (e[0] - BLAST_MARGIN) };
        let up = e[2] + BLAST_MARGIN - p.y;
        let out = if v.y > 0.0 && reach(v.y) > up {
            Some((p.x, e[2] + BLAST_MARGIN + 20.0))
        } else if reach(v.x) > side {
            Some((if v.x > 0.0 { e[1] + BLAST_MARGIN + 20.0 } else { e[0] - BLAST_MARGIN - 20.0 }, p.y))
        } else {
            None
        };
        if let Some((x, y)) = out {
            eprintln!("explore: screen KO, player {} at ({:.0},{:.0}) launched ({:.2},{:.2}) at room frame {}", (*fp).player_idx, p.x, p.y, v.x, v.y, CLOCK);
            // world space, as `set_bounds` puts the blast zone at the extent + margin (stage-offset corrected)
            (*fp).cur_pos.x = x;
            (*fp).cur_pos.y = y;
            (*fp).prev_pos = (*fp).cur_pos; // no swept collision back through the wall
        }
    });
}

/// The camera and blast bounds of section `s` (camera: the section's extent; blast: 120 units outside it).
unsafe fn set_bounds(s: usize) {
    let Some(m) = cur() else { return };
    if ARENA {
        // the arena's own camera limit and blast zones (stage position model)
        let ci = &mut sg::stage_info.cam_info;
        let (ox, oy) = (ci.cam_x_offset, ci.cam_y_offset);
        if let Some(c) = m.cam {
            ci.cam_bounds.left = c[0] - ox;
            ci.cam_bounds.right = c[1] - ox;
            ci.cam_bounds.top = c[2] - oy;
            ci.cam_bounds.bottom = c[3] - oy;
        }
        if let Some(b) = m.blast {
            let bz = &mut sg::stage_info.blast_zone;
            bz.left = b[0] - ox;
            bz.right = b[1] - ox;
            bz.top = b[2] - oy;
            bz.bottom = b[3] - oy;
        }
        return;
    }
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
    bz.left = e[0] - BLAST_MARGIN - ox;
    bz.right = e[1] + BLAST_MARGIN - ox;
    bz.top = e[2] + BLAST_MARGIN - oy;
    bz.bottom = e[3] - BLAST_MARGIN - oy;
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
    // still in the match entry (Petey's arena places the party on its first frame): Entry squashes the root joint
    // to the stand's height (x6C4 in Y) and only EntryStart grows it back; leaving the entry here would keep the
    // fighter flat for the whole match. Give the root its saved scale back first.
    let ms = (*fp).motion_id;
    if ms >= melee_types::ftCo_MS_Entry as i32 && ms <= melee_types::ftCo_MS_EntryEnd as i32 {
        let j = (*g).hsd_obj as *mut melee_types::HSD_JObj;
        if !j.is_null() {
            let s = (*fp).mv.co.entry.x8;
            crate::mn_inl::HSD_JObjSetScaleX(j as _, s.x);
            crate::mn_inl::HSD_JObjSetScaleY(j as _, s.y);
        }
    }
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
pub unsafe fn room_tick() {
    if !ACTIVE {
        return;
    }
    if ARENA {
        CLOCK += 1;
        if CLOCK == 1 {
            set_bounds(0);
            // the party at the arena's player spawn points
            if let Some(m) = arena() {
                let mut k = 0;
                each_fighter(|g, fp| {
                    if let Some(p) = m.players.get(k) {
                        place(g, fp, p[0], p[1]);
                    }
                    k += 1;
                });
            }
        }
        return;
    }
    let Some(m) = map() else { return };
    CLOCK += 1;
    if CLOCK == 1 {
        set_bounds(SEC as usize); // after the stage's own camera data has been read
    }
    // the section's enemies (the Subspace enemy lane: plain sim entities, no Melee articles, so a VS match spawns
    // them too) and every crate of the map
    if CLOCK == 2 {
        spawn_section(m, 0);
        spawn_crates(m);
    }
    // outside an Envoy run nothing else ticks the enemy pool (the mod host's post chain does inside one); its own
    // frame latch keeps it to one step per logic frame either way
    if crate::tu::envoy::armed() == 0 {
        crate::tu::subspace::sub_tick();
    }
    if FADE == 0 {
        screen_kos(m);
    }
    blocks_tick(m);
    if CLOCK % 10 == 0 && std::env::var_os("MELEE_EXPLORE_LOG").is_some() {
        each_fighter(|_, fp| {
            let p = (*fp).cur_pos;
            eprintln!("explore: frame {} player {} at ({:.1},{:.1}) ground {} section {}", CLOCK, (*fp).player_idx, p.x, p.y, ((*fp).ground_or_air as i32 == GA_Ground as i32) as i32, SEC);
        });
    }
    if FADE > 0 {
        FADE += 1;
        // the up tilt that opened the door is also a tap jump: a fighter still in jump squat on the way in stands
        if FADE < FADE_IN {
            each_fighter(|g, fp| {
                if (*fp).motion_id == ftCo_MS_KneeBend as i32 && (*fp).ground_or_air as i32 == GA_Ground as i32 {
                    melee_fighter::tu::ft_08a1::ft_8008A2BC(g as _);
                }
            });
        }
        if FADE == FADE_IN && DEST >= 0 {
            let d = m.doors[DEST as usize];
            let mut k = 0;
            each_fighter(|g, fp| {
                // the party stands side by side at the destination door
                place(g, fp, d.x + if k % 2 == 0 { -6.0 } else { 6.0 } * ((k / 2 + 1) as f32), d.y);
                k += 1;
            });
            if SEC != d.sec as i32 {
                spawn_section(m, d.sec);
            }
            SEC = d.sec as i32;
            VISITED |= 1u32.pshl(d.sec.min(31) as i64);
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
    let mut opener: *mut HSD_GObj = core::ptr::null_mut();
    each_fighter(|g, fp| {
        let up = (*fp).input.lstick[0].y > 0.7;
        let fresh = up && !UP_PREV[idx.min(5)];
        UP_PREV[idx.min(5)] = up;
        idx += 1;
        // only a human player opens doors (a CPU's up input is its AI: an Envoy foe, or a CPU partner, would drag the
        // whole party into the next room mid-fight), and in an Envoy run only the party's slots
        let slot = (*fp).player_idx as i32;
        let party = crate::tu::envoy::armed() == 0 || slot < if crate::tu::envoy::PLAN[crate::tu::envoy::P_COOP] == 1 { 2 } else { 1 };
        let human = melee_fighter::tu::player::Player_GetPlayerSlotType(slot) as i32 == 0;
        if !fresh || !party || !human || (*fp).ground_or_air as i32 != GA_Ground as i32 || want >= 0 {
            return;
        }
        let p = (*fp).cur_pos;
        for (di, d) in m.doors.iter().enumerate() {
            // overlapping doors (080103 has its two room doors at one spot): a linked one wins over an unlinked
            // one, and a door into a room not yet seen over one into a seen room
            let near = d.sec as i32 == SEC && (p.x - d.x).abs() <= 15.0 && p.y >= d.y - 55.0 && p.y <= d.y + 20.0;
            let score = |d: &Door| if d.to < 0 { if d.exit { 1 } else { 0 } } else if VISITED & 1u32.pshl(m.doors[d.to as usize].sec.min(31) as i64) == 0 { 3 } else { 2 };
            if near && (want < 0 || score(d) > score(&m.doors[want as usize])) {
                want = di as i32;
                opener = g;
            }
        }
    });
    if want >= 0 {
        let d = m.doors[want as usize];
        DEST = if d.to >= 0 {
            d.to
        } else if !d.exit {
            -1 // an unlinked door stays shut
        } else {
            // the level exit (GMSJ / arrow door): back to the first linked door of section 0 here
            m.doors.iter().position(|o| o.sec == 0 && o.to >= 0).map(|i| i as i32).unwrap_or(-1)
        };
        if DEST >= 0 {
            FADE = 1;
            // the same up tilt started a tap jump (jump squat): the door takes it, so the opener stands instead
            if !opener.is_null() {
                melee_fighter::tu::ft_08a1::ft_8008A2BC(opener as _);
            }
            eprintln!("explore: door {want} opened (type {}) at room frame {}", d.kind, CLOCK);
        }
    }
}

/// Section `s`'s enemy points, each as its own Brawl kind's nearest Subspace kind (`enemy_kind`); the previous
/// section's enemies go (Brawl rebuilds a room's generators on entry too). At most the pool's size.
unsafe fn spawn_section(m: &Map, s: usize) {
    use crate::tu::subspace as ss;
    // only the map's own: the Envoy sim's enemies (its foes for the fight) stay, and keep half the pool free for it
    ss::clear_room();
    let cap = if crate::tu::envoy::armed() != 0 { ss::MAX / 2 } else { ss::MAX };
    let mut n = 0;
    for &(si, x, y, face, id) in &m.spawns {
        if si != s || n >= cap {
            continue;
        }
        let (k, name) = enemy_kind(id);
        let Some(k) = k else {
            eprintln!("explore: enemy point kind {id} ({name}) not spawned");
            continue;
        };
        let h = ss::spawn(k, 0, x, y + 5.0, face);
        eprintln!("explore: enemy {name} (Brawl kind {id}) as {} at ({x:.0},{y:.0}) section {s} -> {h}", ss::kind_name(k));
        n += 1;
    }
}

/// Every GIB2 crate of the map as a Melee container (small: capsule, crate: box, big: barrel), once per room. Its
/// contents follow the match's item switch, as any Melee container's.
unsafe fn spawn_crates(m: &Map) {
    for c in &m.crates {
        let kind = match c.size {
            0 => It_Kind_Capsule,
            2 => It_Kind_Taru,
            _ => It_Kind_Box,
        };
        let mut sp = SpawnItem::zeroed();
        sp.kind = kind;
        sp.pos = melee_types::Vec { x: c.x, y: c.y + 2.0, z: 0.0 };
        sp.prev_pos = sp.pos;
        sp.facing_dir = -1.0;
        sp.x48_ground_or_air = GA_Air;
        let g = melee_items::tu::item::Item_80268B18(&mut sp);
        eprintln!("explore: crate (size {}) at ({:.0},{:.0}) section {} -> item kind {} {}", c.size, c.x, c.y, c.sec, kind, if g.is_null() { "refused" } else { "spawned" });
    }
}

/// The breakable blocks: a live hitbox of any fighter (or a fighter's item) touching an unbroken block breaks it,
/// and its collision joint goes off (`mpLib_80057BC0`, as Melee's stages switch a joint off). One hit breaks one,
/// as Brawl's bricks under a normal attack.
unsafe fn blocks_tick(m: &Map) {
    if m.blocks.is_empty() || FADE > 0 {
        return;
    }
    let touches = |b: &Block, x: f32, y: f32, r: f32| (x - b.x).abs() <= b.hw + r && (y - b.y).abs() <= b.hh + r;
    for (k, b) in m.blocks.iter().enumerate().take(64) {
        if BROKEN & 1u64.pshl(k as i64) != 0 || b.sec as i32 != SEC {
            continue;
        }
        let mut hit = false;
        each_fighter(|_, fp| {
            for h in (*fp).x914.iter() {
                if h.state != HitCapsule_Disabled && h.damage > 0.0 && (touches(b, h.x4C.x, h.x4C.y, h.scale) || touches(b, h.x58.x, h.x58.y, h.scale)) {
                    hit = true;
                }
            }
        });
        if hit {
            BROKEN |= 1u64.pshl(k as i64);
            melee_world::tu::mplib::mpLib_80057BC0(k as i32 + 1);
            eprintln!("explore: block {k} broken at ({:.0},{:.0}) room frame {}", b.x, b.y, CLOCK);
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
    let Some(m) = cur() else { return };
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
        let a = if FADE < FADE_IN {
            FADE * 255 / FADE_IN
        } else if FADE <= FADE_IN + HOLD {
            255
        } else {
            (FADE_LEN - FADE).max(0) * 255 / FADE_IN
        };
        pm::draw_fade(a as u8);
    }
}

/// Install the hooks (the engine start). Nothing is hooked without the owner's Brawl files (or Petey's mode).
pub fn install() {
    if !available(GD_STAGE_SSE_LAKE3) && !crate::tu::petey::selected() {
        return;
    }
    unsafe {
        melee_world::tu::ground::EXPLORE_COLL = Some(coll_override);
        melee_world::tu::grlast::EXPLORE_FRAME = Some(room_tick);
        melee_world::tu::grdisplay::HIDE_GROUND = Some(hide_ground);
        melee_fighter::tu::ftdrawcommon::MAP_DRAW = Some(draw);
    }
    // start loading now, in the background (the first stage load that needs it waits for it there)
    if available(GD_STAGE_SSE_LAKE3) {
        std::thread::spawn(|| {
            let _ = map();
        });
    }
    if crate::tu::petey::selected() {
        let _ = arena();
    }
}

fn fmin(a: f32, b: f32) -> f32 {
    if b < a { b } else { a }
}

fn fmax(a: f32, b: f32) -> f32 {
    if b > a { b } else { a }
}


#[cfg(test)]
mod arena_tests {
    /// With the owner's Brawl files (skipped otherwise): the welded arena floor has ledges at its two outer ends
    /// and none at the weld seams.
    #[test]
    fn arena_outer_ledges() {
        let Ok(m) = super::load_secs(&super::ARENA_SECS, true, false) else { return };
        let mut ledges = vec![];
        for l in &m.lines {
            if l.hi_flags & 1 != 0 {
                let (a, b) = (m.verts[l.v0_idx as usize], m.verts[l.v1_idx as usize]);
                let lg = l.lo_flags & 0x200 != 0;
                eprintln!("floor ({:.1},{:.1})-({:.1},{:.1}) ledge {}", a.x, a.y, b.x, b.y, lg);
                if lg {
                    ledges.push(a.x.min(b.x));
                    ledges.push(a.x.max(b.x));
                }
            }
        }
        let (lo, hi) = (m.verts.iter().map(|v| v.x).fold(f32::MAX, f32::min), m.verts.iter().map(|v| v.x).fold(f32::MIN, f32::max));
        assert!(ledges.iter().any(|&x| x == lo) && ledges.iter().any(|&x| x == hi), "outer ends not ledges");
        assert!(!ledges.contains(&0.0), "weld seam is a ledge");
    }
}

#[cfg(test)]
mod lake3_tests {
    /// With the owner's Brawl files (skipped otherwise): every door of the main section's room doors and the side
    /// rooms' way-back doors is linked from Brawl's own door data, the exit is the arrow door, and every enemy point
    /// has a kind.
    #[test]
    fn lake3_links() {
        let Ok(mut m) = super::load_secs(&super::SECS, true, true) else { return };
        super::link_doors(&mut m, &super::SECS);
        for (i, d) in m.doors.iter().enumerate() {
            eprintln!("door {i} sec {} type {} to {} exit {} back {} room {}", d.sec, d.kind, d.to, d.exit, d.back, d.into_room);
        }
        for s in &m.spawns {
            eprintln!("spawn sec {} kind {} -> {}", s.0, s.4, super::enemy_kind(s.4).1);
        }
        eprintln!("crates {} blocks {} joints {} lines {}", m.crates.len(), m.blocks.len(), m.joints.len(), m.lines.len());
        assert!(m.doors.iter().any(|d| d.exit), "no exit");
        for d in m.doors.iter().filter(|d| d.into_room || d.back) {
            assert!(d.to >= 0, "room door unlinked");
        }
        assert!(m.spawns.iter().all(|s| super::enemy_kind(s.4).1 != "unmapped"));
    }
}