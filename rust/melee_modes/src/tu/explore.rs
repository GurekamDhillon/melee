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
use melee_core::ppc::{fctiwz_f32, Ppc};
use melee_types::*;
use std::vec::Vec; // melee_types has its own `Vec` (the 3D vector): `melee_types::Vec` below

/// One map section: an adventure archive placed at `off` in the shared world (sections never overlap).
struct SecDef {
    file: String,
    off: [f32; 2],
    /// the section's map model; empty: found in the archive by its name (`section_model`)
    model: String,
    /// 0: a main section; 1, 2, ..: its side rooms `a`, `b`, .. (they follow their main section)
    room: u8,
    /// the start-point bones jumps arrive at in this section (`Jump::start`)
    starts: Vec<String>,
}

fn sec(file: &str, x: f32, y: f32, model: &str, room: u8) -> SecDef {
    SecDef { file: file.into(), off: [x, y], model: model.into(), room, starts: vec![] }
}

/// One record of Brawl's step-jump table (`ADSJ` in `adventure_common.arc`, one table per level; local reference
/// `_build/boss-ir/sse-chain.md`): the jump a door (GDOR id) or a step-jump block (GMSJ) performs, the step it leads
/// to and the start-point bone it arrives at. Read from the owner's files at load time, never embedded.
#[derive(Clone, Debug)]
pub struct Jump {
    pub id: u32,
    /// 0 a move inside the segment, `JUMP_START` the segment's start, `JUMP_CLEAR` the segment's end
    pub kind: u8,
    pub dest: u32,
    pub start: String,
}
pub const JUMP_START: u8 = 1;
pub const JUMP_CLEAR: u8 = 2;

/// Level `level`'s step-jump table (empty without the owner's files).
pub fn jumps(level: u16) -> Vec<Jump> {
    let Some((leaves, tags)) = pm::pac_leaves_tagged("adventure_common.arc") else { return vec![] };
    let mut v = vec![];
    for (l, t) in leaves.iter().zip(&tags) {
        if l.get(..4) != Some(b"ADSJ") || t.last().map(|x| x.1) != Some(level) {
            continue;
        }
        for k in 0..be32(l, 4).min(4096) as usize {
            let o = be32(l, 16 + 4 * k) as usize;
            let name = l.get(o + 12..(o + 44).min(l.len())).unwrap_or(&[]);
            let end = name.iter().position(|&c| c == 0).unwrap_or(name.len());
            v.push(Jump { id: be32(l, o), kind: l.get(o + 4).copied().unwrap_or(0), dest: be32(l, o + 8), start: String::from_utf8_lossy(&name[..end]).into_owned() });
        }
    }
    v
}

/// The archive stem a jump starts from: the jump id's level, segment and area digits, its high nibble the room.
fn jump_file(id: u32) -> String {
    let b = id.to_be_bytes();
    let r = b[3] >> 4;
    format!("{:02x}{:02x}{:02x}{}", b[0], b[1], b[2], if r >= 0xA { ((b'a' + r - 0xA) as char).to_string() } else { String::new() })
}

/// The archive stem of a step: level, segment, area (numbers) and its room (0 the main section, 1 `a`, ..).
fn step_file(step: u32) -> String {
    let b = step.to_be_bytes();
    format!("{:02}{:02}{:02}{}", b[0], b[1], b[2], if b[3] > 0 { ((b'a' + b[3] - 1) as char).to_string() } else { String::new() })
}

/// Our placement of a section in the shared world (the port's layout, not Brawl data): the two first maps keep the
/// places they always had; any other main section goes 8000 further on x, its side rooms 4000 below, 1500 apart.
fn sec_place(stem: &str, main_k: usize, room: u8) -> [f32; 2] {
    const PLACE: [(&str, f32, f32); 14] = [
        ("080103", 0.0, 0.0), ("080103a", 1500.0, 0.0), ("080103b", 3000.0, 0.0), ("080104", 8000.0, 0.0),
        ("080104a", 8000.0, -4000.0), ("080104b", 9500.0, -4000.0), ("080105", 16000.0, 0.0), ("080105a", 16000.0, -4000.0),
        ("060001", 0.0, 0.0), ("060002", 8000.0, 0.0), ("060002a", 11000.0, 0.0), ("060003", 16000.0, 0.0),
        ("060004", 24000.0, 0.0), ("060004a", 24000.0, -4000.0),
    ];
    match PLACE.iter().find(|p| p.0 == stem) {
        Some(p) => [p.1, p.2],
        None if room == 0 => [8000.0 * main_k as f32, 0.0],
        None => [8000.0 * main_k as f32 + 1500.0 * (room - 1) as f32, -4000.0],
    }
}

/// A GD stage's sections from Brawl's own chain: every archive reachable from its first section through the
/// step-jump table (a segment's start and clear are not walked; an archive not in the owner's folder is left out,
/// its doors stay shut), main sections in area order, each followed by its side rooms.
fn chain_secs(level: u16, first: &str, js: &[Jump]) -> Vec<SecDef> {
    let mut seen = vec![first.to_string()];
    let mut k = 0;
    while k < seen.len() {
        let from = seen[k].clone();
        k += 1;
        for j in js.iter().filter(|j| j.kind != JUMP_START && j.kind != JUMP_CLEAR && jump_file(j.id) == from) {
            let to = step_file(j.dest);
            if !seen.contains(&to) && to.get(..2) == first.get(..2) && pm::adventure_present(&format!("{to}.pac")) {
                seen.push(to);
            }
        }
    }
    let _ = level;
    seen.sort_by(|a, b| a.get(..6).cmp(&b.get(..6)).then(a.len().cmp(&b.len())).then(a.cmp(b)));
    let mut main_k = 0usize;
    let mut out: Vec<SecDef> = vec![];
    for (i, s) in seen.iter().enumerate() {
        let room = s.as_bytes().get(6).map(|c| c - b'a' + 1).unwrap_or(0);
        if room == 0 && i > 0 {
            main_k += 1;
        }
        let at = sec_place(s, main_k, room);
        let mut d = sec(&format!("{s}.pac"), at[0], at[1], "", room);
        for j in js.iter().filter(|j| j.kind != JUMP_CLEAR && step_file(j.dest) == *s) {
            if !d.starts.contains(&j.start) {
                d.starts.push(j.start.clone());
            }
        }
        out.push(d);
    }
    out
}

/// A GD stage's sections and step-jump table, built once from the owner's files.
struct Chain {
    secs: Vec<SecDef>,
    jumps: Vec<Jump>,
}

fn chain_of(id: i32) -> &'static Chain {
    static C: std::sync::OnceLock<Vec<Chain>> = std::sync::OnceLock::new();
    let all = C.get_or_init(|| {
        GD_STAGES
            .iter()
            .map(|g| {
                let js = jumps(g.level);
                Chain { secs: chain_secs(g.level, g.first, &js), jumps: js }
            })
            .collect()
    });
    &all[GD_STAGES.iter().position(|g| g.id == id).unwrap_or(0)]
}

/// The sections of GD stage `id`.
fn secs_of(id: i32) -> &'static [SecDef] {
    &chain_of(id).secs
}

/// A section's map model: the archive's MDL0 named `<level><segment digit>[m|v]Adv...<area><room>` (Brawl's naming
/// of section models), else the only `<level><segment digit>.Adv` model there.
fn section_model(leaves: &[Vec<u8>], stem: &str) -> String {
    let (pre, tail) = (format!("{}{}", stem.get(..2).unwrap_or(""), stem.get(3..4).unwrap_or("")), stem.get(4..).unwrap_or(""));
    let mut cands = vec![];
    for b in leaves.iter().filter(|b| b.get(..4) == Some(b"bres")) {
        for (n, off) in pm::brres_entries(b).unwrap_or_default() {
            if b.get(off..off + 4) == Some(b"MDL0") && n.starts_with(&pre) && n.get(pre.len() + 1..pre.len() + 4) == Some("Adv") && !cands.contains(&n) {
                cands.push(n);
            }
        }
    }
    cands.iter().find(|n| n.ends_with(tail) && !n.ends_with(&format!("{tail}a")) && !n.ends_with(&format!("{tail}b"))).or(if cands.len() == 1 { cands.first() } else { None }).cloned().unwrap_or_default()
}

fn arena_secs() -> [SecDef; 1] {
    [sec("030101.pac", 0.0, 0.0, "030vAdvStadium", 0)]
}

/// Brawl's enemy kind (GEG1 [off]) -> the nearest of our Subspace kinds (`subspace::K_*`), or None for a boss or a
/// boss part (not spawned). Kind ids follow sora_enemy's kind -> `createInstance<em*>` table (read by 0x81A0F6B0
/// for emManager::createEnemy 0x81A0C9D4); names: the classes there, English names as BrawlCrate's `EnemyList`.
fn enemy_kind(id: u16) -> (Option<usize>, &'static str) {
    use crate::tu::subspace::{K_BILL, K_GIANT, K_GOOMBA, K_HAMMER, K_JYK, K_KOOPA, K_PRIMID};
    match id {
        0 => (Some(K_GOOMBA), "Goomba"),
        1 => (Some(K_GOOMBA), "Poppant"),
        2 => (Some(K_GOOMBA), "Feyesh"),
        3 => (Some(K_JYK), "Jyk"),
        4 => (Some(K_BILL), "Auroros"), // a diving bird: flies straight at the party
        5 => (Some(K_JYK), "Cymul"),
        6 => (Some(K_PRIMID), "Roturret"),
        7 => (Some(K_GIANT), "Borboras"),
        8 => (Some(K_GIANT), "Giant Goomba"),
        9 => (Some(K_JYK), "Buckot"),
        10 | 16 => (Some(K_GOOMBA), "Bucculus/Mite"),
        11 => (Some(K_GIANT), "Greap"),
        12 => (Some(K_PRIMID), "Shaydas"),
        13 => (Some(K_BILL), "Bullet Bill"),
        14 => (Some(K_BILL), "Roader"),
        15 => (Some(K_PRIMID), "Spaak"),
        17 => (Some(K_GOOMBA), "Ticken"),
        18 => (Some(K_GIANT), "Towtow"),
        19 => (Some(K_HAMMER), "Hammer Bro"),
        20 => (Some(K_GOOMBA), "Bytan"),
        21 => (Some(K_JYK), "Floow"),
        22 => (Some(K_JYK), "Puppit"),
        23 | 28 | 31 | 33 | 34 => (Some(K_PRIMID), "Primid"),
        24 => (Some(K_KOOPA), "Shellpod"),
        25 => (Some(K_KOOPA), "Koopa"),
        26 => (Some(K_GIANT), "Armight"),
        27 => (Some(K_GOOMBA), "Bombed"),
        29 => (Some(K_GOOMBA), "Nagagog"),
        30 => (Some(K_JYK), "Trowlon"),
        32 => (Some(K_PRIMID), "Boomerang Primid"),
        35 => (Some(K_PRIMID), "Sword Primid"),
        36 => (Some(K_GIANT), "Gamyga"),
        37 => (Some(K_PRIMID), "R.O.B. Blaster"),
        38 => (Some(K_JYK), "R.O.B. Distance"),
        39 => (Some(K_PRIMID), "R.O.B. Launcher"),
        40 => (Some(K_GIANT), "R.O.B. Sentry"),
        41 => (Some(K_PRIMID), "Autolance"),
        42 => (Some(K_GIANT), "Armank"),
        43..=45 => (Some(K_GOOMBA), "Glunder"),
        46 => (None, "Petey Piranha (boss)"),
        47..=50 => (None, "Gamyga base (part)"),
        51 => (None, "Galleom (boss)"),
        52 => (None, "Ridley (boss)"),
        53 => (None, "Rayquaza (boss)"),
        54 => (None, "Duon (boss)"),
        55 => (None, "Porky (boss)"),
        56 => (None, "Meta Ridley (boss)"),
        57 => (None, "Falcon Flyer (boss)"),
        58 => (None, "Tabuu (boss)"),
        59 | 60 => (None, "Master/Crazy Hand (boss)"),
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
    /// the door's own id (GDOR [off]: level, step, room, index)
    pub id: u32,
    /// GDOR [off]: set on a side room's way back to its main section
    pub back: bool,
    /// GDOR [off] == [val]: a main-section door into a side room
    pub into_room: bool,
    /// the level's exit (the door the GMSJ block names, else an arrow door)
    pub exit: bool,
    /// the step jump the door performs (`Jump::id`): a GMSJ block's jump when the door is that block's, else its id
    pub jump: u32,
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
    /// per model: its display lists (`pm::build_lists`, at its offset), built by the loader
    lists: Vec<Vec<Vec<u8>>>,
    door_models: Vec<(pm::Rig, Vec<pm::Vert>)>,
    /// the bricks' looks (`Block::model`): name, rig, bind-pose triangles
    brick_models: Vec<(String, pm::Rig, Vec<pm::Vert>)>,
    /// moving platforms' bone positions per frame (section space; `Block::path`)
    paths: Vec<Vec<[f32; 2]>>,
    /// per spawn (`spawns`): index into `enemy_paths`, usize::MAX none
    pub spawn_path: Vec<usize>,
    /// the placements' motion paths (Brawl's grGimmickMotionPath; `enemy_path`)
    enemy_paths: Vec<EnemyPath>,
    /// what the loader left out or moved (printed with the load report)
    notes: Vec<String>,
    /// From the stage position model (arena only): camera limit and blast zones (left, right, top, bottom) and the
    /// player spawn points.
    pub cam: Option<[f32; 4]>,
    pub blast: Option<[f32; 4]>,
    pub players: Vec<[f32; 2]>,
    /// per section: where a party arriving from the section before stands (its first player spawn, else its first
    /// door, else its left floor end)
    pub entries: Vec<[f32; 2]>,
    /// the start-point bones found (section, name, world position): where a jump arrives
    pub starts: Vec<(usize, String, [f32; 2])>,
    /// per GD stage map: a digest of its collision (the content mask's per-section check, envoy-sse-design slice 0)
    pub digest: u64,
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
    /// its look: index into `Map::brick_models` (drawn at `at` until broken); usize::MAX none
    pub model: usize,
    pub at: [f32; 2],
    /// a moving platform (never breaks): index into `Map::paths`; usize::MAX a brick
    pub path: usize,
}

fn load(id: i32) -> Result<Map, String> {
    let secs = secs_of(id);
    let mut m = load_secs(secs, true, true)?;
    link_doors(&mut m, secs, &chain_of(id).jumps);
    Ok(m)
}

/// Door targets from Brawl's own chain (`jumps`, the level's step-jump table): a door performs its jump (its GMSJ
/// block's when it has one, else its own id); a segment clear is the level's way on (`STORY_NEXT`); any other jump
/// leads to its step's section, arriving at the record's start-point bone (an `ENTRY` pseudo door there, one per
/// spot; the section's entry when the bone is missing). A door with no record, or into a section not loaded, stays
/// shut.
fn link_doors(m: &mut Map, secs: &[SecDef], js: &[Jump]) {
    let mut arrive: Vec<(usize, String, usize)> = vec![];
    for i in 0..m.doors.len() {
        let d = m.doors[i];
        if d.kind == ENTRY {
            continue;
        }
        m.doors[i].exit = false;
        let Some(j) = js.iter().find(|j| j.id == d.jump) else { continue };
        if j.kind == JUMP_CLEAR {
            m.doors[i].to = STORY_NEXT;
            m.doors[i].exit = true;
            continue;
        }
        let to = step_file(j.dest);
        let Some(ts) = secs.iter().position(|s| s.file.strip_suffix(".pac") == Some(to.as_str())) else { continue };
        if m.ext.get(ts).is_none_or(|e| *e == [0.0; 4]) {
            continue; // the section's archive was not readable
        }
        let e = match arrive.iter().find(|a| a.0 == ts && a.1 == j.start) {
            Some(a) => a.2,
            None => {
                let at = m.starts.iter().find(|s| s.0 == ts && s.1 == j.start).map(|s| s.2).unwrap_or_else(|| m.entries.get(ts).copied().unwrap_or([0.0, 0.0]));
                m.doors.push(Door { sec: ts, x: at[0], y: at[1], kind: ENTRY, to: -1, id: 0, back: false, into_room: false, exit: false, jump: 0 });
                arrive.push((ts, j.start.clone(), m.doors.len() - 1));
                m.doors.len() - 1
            }
        };
        m.doors[i].to = e as i32;
    }
}

/// A door kind of our own: a section's entry point (where the way on from the section before arrives); never drawn,
/// never opened.
const ENTRY: u8 = 0xFF;
/// `Door::to` of the last main section's way on: the level's next story step (`story_next`).
pub const STORY_NEXT: i32 = -2;

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
    let mut exits: Vec<(u32, u32)> = vec![];
    let mut models = vec![];
    let mut door_models = vec![];
    let mut brick_models: Vec<(String, pm::Rig, Vec<pm::Vert>)> = vec![];
    let mut paths: Vec<Vec<[f32; 2]>> = vec![];
    let mut spawn_path: Vec<usize> = vec![];
    let mut enemy_paths: Vec<EnemyPath> = vec![];
    // load notes, printed by the game thread with the map's report (never printed from a loader thread)
    let mut notes: Vec<String> = vec![];
    let mut cam = None;
    let mut blast = None;
    let mut players = vec![];
    let mut entries: Vec<[f32; 2]> = vec![];
    let mut starts: Vec<(usize, String, [f32; 2])> = vec![];
    for (si, s) in secs.iter().enumerate() {
        let p_first = players.len();
        let mut prop_cache: Option<Vec<pm::Rig>> = None;
        let Some((leaves, tags)) = pm::pac_leaves_tagged(&s.file) else {
            if si > 0 {
                // a missing side room: the main section still plays (its doors into it stay shut)
                ext.push([0.0; 4]);
                entries.push(s.off);
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
                            // a brick bone at the origin: its place comes from Brawl's block data (GBLK), whose bone
                            // reference is not decoded (notes/sse-stage.md); left out rather than put at the origin
                            Some(t) if t[0] == 0.0 && t[1] == 0.0 => {
                                notes.push(format!("explore: section {si}: brick {} has no place (GBLK); left out", String::from_utf8_lossy(name)));
                                continue;
                            }
                            Some(t) => {
                                let bb = [bef(c, o + 0x14), bef(c, o + 0x18), bef(c, o + 0x1C), bef(c, o + 0x20)];
                                let (x, y) = (t[0] + s.off[0] + (bb[0] + bb[2]) / 2.0, t[1] + s.off[1] + (bb[1] + bb[3]) / 2.0);
                                let nm = String::from_utf8_lossy(name).into_owned();
                                // the brick's own model (an MDL0 of the brick's name in the section archive), posed at the
                                // same bone
                                let model = match brick_models.iter().position(|(n, _, _)| *n == nm) {
                                    Some(i) => i,
                                    None => match pm::static_models(&leaves, &[nm.as_str()]).pop() {
                                        Some((n, r, t)) => {
                                            brick_models.push((n, r, t));
                                            brick_models.len() - 1
                                        }
                                        None => usize::MAX,
                                    },
                                };
                                let at = [t[0] + s.off[0], t[1] + s.off[1]];
                                blocks.push(Block { sec: si, x, y, hw: (bb[2] - bb[0]) / 2.0, hh: (bb[3] - bb[1]) / 2.0, model, at, path: usize::MAX });
                                ([t[0] + s.off[0], t[1] + s.off[1]], blocks.len())
                            }
                            None => {
                                notes.push(format!("explore: section {si}: a brick's bone not found; left out"));
                                continue;
                            }
                        }
                    } else if let Some(path) = mover_path(&leaves, &mut prop_cache, name) {
                        // a platform riding an animated bone (060002a's sfyuka00-03 on the water wheel `st02asuisha`):
                        // its own joint, its points moved along the bone's path every frame (`movers_tick`)
                        let at = [path[0][0] + s.off[0], path[0][1] + s.off[1]];
                        let bb = [bef(c, o + 0x14), bef(c, o + 0x18), bef(c, o + 0x1C), bef(c, o + 0x20)];
                        let (x, y) = (at[0] + (bb[0] + bb[2]) / 2.0, at[1] + (bb[1] + bb[3]) / 2.0);
                        paths.push(path);
                        blocks.push(Block { sec: si, x, y, hw: (bb[2] - bb[0]) / 2.0, hh: (bb[3] - bb[1]) / 2.0, model: usize::MAX, at, path: paths.len() - 1 });
                        notes.push(format!("explore: section {si}: {} moves ({} frames{})", String::from_utf8_lossy(name), paths.last().map(|p| p.len()).unwrap_or(0), if paths.last().map_or(0, |p| p.len()) == 1 { ", held at rest" } else { "" }));
                        (at, blocks.len())
                    } else {
                        if !name.starts_with(b"AdvGdtItemBox") {
                            notes.push(format!("explore: section {si}: a collision object riding a moving part left out ({} lines, bone {})", pc, String::from_utf8_lossy(name)));
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
        for (bi, b) in leaves.iter().enumerate().filter(|(_, d)| d.get(..4) == Some(b"BLOC")) {
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
                    jump: be32(b, e + 0x30),
                });
            }
            for e in bloc_entries(b, b"GEG1") {
                spawn_path.push(placement_path(&leaves, &tags, bi, b.get(e + 6).copied().unwrap_or(0xFF), s.off, &mut enemy_paths, &mut notes));
                spawns.push((si, bef(b, e + 0x28) + s.off[0], bef(b, e + 0x2C) + s.off[1], if b[e + 0x67] != 0 { 1 } else { -1 }, be16(b, e + 0x1C)));
            }
            // a step-jump block (GMSJ) is fired by the door of the GDOR block right after it: that door performs
            // the block's jump (its first word, a step-jump id)
            for e in bloc_entries(b, b"GMSJ") {
                if b.get(e + 0xC..e + 0x10) == Some(b"GDOR") {
                    let inner = e + 0xC;
                    let first = inner + be32(b, inner + 8) as usize;
                    exits.push((be32(b, first + 0x30), be32(b, e)));
                }
            }
            for e in bloc_entries(b, b"GIB2") {
                crates.push(Crate { sec: si, x: bef(b, e + 0x30) + s.off[0], y: bef(b, e + 0x34) + s.off[1], size: be32(b, e + 0x38) });
            }
        }
        entries.push(
            players
                .get(p_first)
                .copied()
                .or_else(|| doors.iter().find(|d: &&Door| d.sec == si).map(|d| [d.x, d.y]))
                .unwrap_or([e[0] + 40.0, e[3]]),
        );
        for n in &s.starts {
            if let Some(p) = leaves.iter().find_map(|b| pm::bone_trans(b, n.as_bytes())) {
                starts.push((si, n.clone(), [p[0] + s.off[0], p[1] + s.off[1]]));
            }
        }
        let model = if s.model.is_empty() { section_model(&leaves, s.file.trim_end_matches(".pac")) } else { s.model.clone() };
        if model.is_empty() {
            notes.push(format!("explore: section {si}: no map model found"));
        }
        for (_, r, t) in pm::static_models(&leaves, &[model.as_str()]) {
            models.push((r, t, s.off));
        }
        if si == 0 {
            for (_, r, t) in pm::static_models(&leaves, &["AdvGdtDoor01"]) {
                door_models.push((r, t));
            }
        }
    }
    for d in doors.iter_mut() {
        if let Some(x) = exits.iter().find(|x| x.0 == d.id) {
            d.jump = x.1;
        }
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
    // a section's extent takes in its moving parts too (The Lake's 080105 floor is the raft: its static collision is a
    // ledge, and the camera and blast zones built from that alone left the arrival spot outside them)
    for b in blocks.iter() {
        if let Some(e) = ext.get_mut(b.sec) {
            *e = [fmin(e[0], b.x - b.hw), fmax(e[1], b.x + b.hw), fmax(e[2], b.y + b.hh), fmin(e[3], b.y - b.hh)];
        }
    }
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
    // FNV-1a over the collision as the game gets it (points, then each line's ends, links and flags): two players'
    // maps are the same map only when this matches (their own Brawl files make it)
    let mut digest: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |x: u32| {
        for b in x.to_le_bytes() {
            digest = (digest ^ b as u64).wrapping_mul(0x0100_0000_01b3);
        }
    };
    for p in &verts {
        eat(p[0].to_bits());
        eat(p[1].to_bits());
    }
    for l in &lines {
        eat(l.v0_idx as u32 | (l.v1_idx as u32) << 16);
        eat(l.prev_id0 as u16 as u32 | (l.next_id0 as u16 as u32) << 16);
        eat(l.hi_flags as u32 | (l.lo_flags as u32) << 16);
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
        lists: models.iter().map(|(r, t, off)| pm::build_lists(r, t, *off)).collect(),
        models,
        door_models,
        brick_models,
        paths,
        spawn_path,
        enemy_paths,
        notes,
        cam,
        blast,
        players,
        entries,
        starts,
        digest,
    })
}

/// The GD stage id range this loader owns (external stage ids; retail stops at 0x20s, m-ex starts at 288).
pub const GD_STAGE_FIRST: i32 = 0x7000;
pub const GD_STAGE_LAST: i32 = 0x70FF;
/// The Lake, step 3 (+ its side rooms), from the owner's own Brawl files.
pub const GD_STAGE_SSE_LAKE3: i32 = 0x7001;
/// The Jungle, step 2 (+ its water-wheel side room).
pub const GD_STAGE_SSE_JUNGLE: i32 = 0x7002;
/// Levels 16 (from 160201), 22 (from 220001) and 27 (from 270001), chained from Brawl's step-jump table.
pub const GD_STAGE_SSE_16: i32 = 0x7003;
pub const GD_STAGE_SSE_22: i32 = 0x7004;
pub const GD_STAGE_SSE_27: i32 = 0x7005;
/// Final Destination's external stage id: the carrier of every GD stage (its stage proc, camera and music).
pub const FD_STKIND: i32 = 0x20;

/// A GD stage slot: id, name (the stage select's label), the archives it needs.
pub struct GdStage {
    pub id: i32,
    pub name: &'static str,
    pub files: &'static [&'static str],
    /// Brawl's level number (its step-jump table, `jumps`) and the archive the party starts in
    pub level: u16,
    pub first: &'static str,
    /// the `--match` word that picks it
    pub word: &'static str,
}
/// The GD stage table (the stage select appends one icon per available row).
pub const GD_STAGES: [GdStage; 5] = [
    GdStage { id: GD_STAGE_SSE_LAKE3, name: "The Lake 3 (SSE)", files: &["080103.pac", "080103a.pac", "adventure_common.arc"], level: 8, first: "080103", word: "lake" },
    GdStage { id: GD_STAGE_SSE_JUNGLE, name: "The Jungle (SSE)", files: &["060002.pac", "060002a.pac", "adventure_common.arc"], level: 6, first: "060001", word: "jungle" },
    GdStage { id: GD_STAGE_SSE_16, name: "SSE 16", files: &["160201.pac", "160202.pac", "adventure_common.arc"], level: 16, first: "160201", word: "sse16" },
    GdStage { id: GD_STAGE_SSE_22, name: "SSE 22", files: &["220001.pac", "220002.pac", "adventure_common.arc"], level: 22, first: "220001", word: "sse22" },
    GdStage { id: GD_STAGE_SSE_27, name: "SSE 27", files: &["270001.pac", "270002.pac", "adventure_common.arc"], level: 27, first: "270001", word: "sse27" },
];

/// The GD stage a `--match` word picks (`lake` / `sse`, `jungle`, `sse16`, ..).
pub fn gd_by_word(w: &str) -> Option<i32> {
    if w.eq_ignore_ascii_case("sse") {
        return Some(GD_STAGE_SSE_LAKE3);
    }
    GD_STAGES.iter().find(|g| g.word.eq_ignore_ascii_case(w)).map(|g| g.id)
}

/// Whether any GD stage's files are here (the hooks go in at engine start on this, before any netplay agreement).
fn any_available() -> bool {
    GD_STAGES.iter().any(|g| files_present(g.id))
}

pub fn is_gd_stage(id: i32) -> bool {
    (GD_STAGE_FIRST..=GD_STAGE_LAST).contains(&id)
}

/// Whether GD stage `id`'s archives are in the owner's Brawl folder (`MELEE_BRAWL_ABSENT` hides it).
pub fn files_present(id: i32) -> bool {
    GD_STAGES.iter().any(|g| g.id == id && g.files.iter().all(|f| pm::adventure_present(f)))
}

/// The content this session may play (slice 0): `u32::MAX` = whatever this machine's files allow; a netplay session
/// sets the AND of both peers' masks (a stage whose collision digests differ is dropped), a replay its header's. Set
/// before the match (at connect / replay load), constant through it.
pub static mut ALLOW: u32 = u32::MAX;

/// Whether GD stage `id` can be played: its files are here and the session allows it (`ALLOW`). A peer without the
/// stage (or with other files) makes it unavailable on both sides: the stage select drops the row, a pick falls back
/// to Final Destination itself.
pub fn available(id: i32) -> bool {
    let k = GD_STAGES.iter().position(|g| g.id == id);
    files_present(id) && k.is_some_and(|k| unsafe { ALLOW } & 1u32.pshl(k as i64) != 0)
}

pub fn gd_name(id: i32) -> Option<&'static str> {
    GD_STAGES.iter().find(|g| g.id == id).map(|g| g.name)
}

/// The VS match about to start asked for the SSE stage (set by the stage select / `--match ...,sse`, read at the
/// stage load). Plain static: set before the match, constant through it.
/// The GD stage id picked (0 none).
pub static mut PICK: i32 = 0;
/// The GD stage id of the map in play (set at each stage load; 0 none).
pub static mut CUR_ID: i32 = 0;

/// The retail stage id that carries stage `id`: a GD stage -> Final Destination (and the pick is noted); any other
/// id -> itself (and any earlier pick is cleared). An unavailable GD stage falls back to Final Destination itself.
pub fn carrier(id: i32) -> i32 {
    unsafe {
        PICK = if is_gd_stage(id) && available(id) { id } else { 0 };
        if PICK != 0 {
            preload_textures(PICK);
            eprintln!("explore: stage {id:#x} ({}) on the Final Destination carrier", gd_name(id).unwrap_or("?"));
        }
    }
    if is_gd_stage(id) { FD_STKIND } else { id }
}

/// The renderer's upload-ahead entry (`melee_engine::gxbridge::preload_tex`, set at engine start): texels, width,
/// height, GX format, mip levels.
pub static mut TEX_PRELOAD: Option<unsafe fn(*const u8, u16, u16, u32, u32)> = None;

/// Queue every texture GD stage `id`'s map draws (all sections, doors, bricks) for upload before the match: the
/// renderer puts up about a megabyte of texels a frame and keeps them until their first draw, so the match's first
/// frame and each section's first draw find them made. Called at the pick (presentation only; nothing sim-visible).
pub fn preload_textures(id: i32) {
    let Some(f) = (unsafe { TEX_PRELOAD }) else { return };
    let Some(k) = GD_STAGES.iter().position(|g| g.id == id) else { return };
    let Some(m) = map_of(id) else { return };
    unsafe { PRELOADED |= 1u32.pshl(k as i64) };
    let mut n = 0;
    let mut seen = std::collections::HashSet::new();
    let mut each = |p: *const u8, w: u16, h: u16, fmt: u32, lv: u32| {
        if seen.insert(p as usize) {
            unsafe { f(p, w, h, fmt, lv) };
            n += 1;
        }
    };
    // the party's first section first: it is the match's first frame
    for (r, _, _) in &m.models {
        pm::rig_textures(r, &mut each);
    }
    for (r, _) in &m.door_models {
        pm::rig_textures(r, &mut each);
    }
    for (_, r, _) in &m.brick_models {
        pm::rig_textures(r, &mut each);
    }
    eprintln!("explore: {n} textures of {id:#x} queued for upload ahead of the match");
}

/// The GD stages whose textures were queued since the last match began (bit per `GD_STAGES` row; presentation).
static mut PRELOADED: u32 = 0;
/// The renderer's "a match began" (set at engine start with `TEX_PRELOAD`): pinned uploads may age out again.
pub static mut TEX_UNPIN: Option<fn()> = None;

/// Every tick of the stage select: each available GD stage's textures are queued once (the pick comes ~14 frames
/// before the match's first frame, too late to upload a map's 15-18 MB of texels at about 1 MB a frame).
pub fn sss_preload() {
    for (k, g) in GD_STAGES.iter().enumerate() {
        let bit = 1u32.pshl(k as i64);
        if unsafe { PRELOADED } & bit == 0 && available(g.id) && map_of(g.id).is_some() {
            unsafe { PRELOADED |= bit };
            preload_textures(g.id);
        }
    }
}

/// Envoy's opt-in for its rooms (`MELEE_ENVOY_EXPLORE=1`).
pub fn enabled() -> bool {
    static E: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *E.get_or_init(|| std::env::var("MELEE_ENVOY_EXPLORE").is_ok_and(|v| v == "1"))
}

/// The map, loaded once on a helper thread (system heap, outside the arena rollback rewrites). `None` when off or
/// without the owner's files.
pub fn map() -> Option<&'static Map> {
    let id = unsafe { CUR_ID };
    map_of(if id != 0 { id } else { GD_STAGE_SSE_LAKE3 })
}

/// The loaded maps and their load reports. Like `subspace_model`'s state, this must survive every rollback load (a
/// snapshot rewrites the executable's statics, and a static `OnceLock` filled by a loader mid-synctest was rewritten
/// under it): one block leaked on the system heap, found through a thread-local of the game thread and handed to the
/// loader threads. The report is printed by the game thread (`map_of`): printing from a loader raced the rewrite of
/// std's own statics and hung a synctest one run in six.
struct Maps {
    m: [std::sync::OnceLock<Option<Map>>; GD_STAGES.len()],
    msg: [std::sync::Mutex<String>; GD_STAGES.len()],
    told: [std::sync::atomic::AtomicBool; GD_STAGES.len()],
}

std::thread_local! {
    static MAPS: std::cell::Cell<Option<&'static Maps>> = const { std::cell::Cell::new(None) };
}

fn maps() -> &'static Maps {
    MAPS.with(|c| {
        if let Some(s) = c.get() {
            return s;
        }
        let s: &'static Maps = std::thread::spawn(|| {
            &*Box::leak(Box::new(Maps {
                m: [const { std::sync::OnceLock::new() }; GD_STAGES.len()],
                msg: [const { std::sync::Mutex::new(String::new()) }; GD_STAGES.len()],
                told: [const { std::sync::atomic::AtomicBool::new(false) }; GD_STAGES.len()],
            }))
        })
        .join()
        .expect("explore map state");
        c.set(Some(s));
        s
    })
}

fn slot_of(id: i32) -> usize {
    GD_STAGES.iter().position(|g| g.id == id).unwrap_or(0)
}

/// Load GD stage `id`'s map into `st` (any thread; once).
fn load_into(st: &'static Maps, id: i32) -> Option<&'static Map> {
    let k = slot_of(id);
    st.m[k]
        .get_or_init(|| {
            if !available(id) {
                return None;
            }
            let t = std::time::Instant::now();
            let r = load(id);
            let mut rep = String::new();
            use std::fmt::Write as _;
            match &r {
                Ok(m) => {
                    let _ = writeln!(
                        rep,
                        "explore: map loaded ({} sections, {} verts, {} lines, {} doors, {} spawns, {} crates, {} blocks, {} tris) in {} ms",
                        m.ext.len(), m.verts.len(), m.lines.len(), m.doors.len(), m.spawns.len(), m.crates.len(), m.blocks.len(),
                        m.models.iter().map(|(_, t, _)| t.len() / 3).sum::<usize>(), t.elapsed().as_millis()
                    );
                    for n in &m.notes {
                        let _ = writeln!(rep, "{n}");
                    }
                    for (i, d) in m.doors.iter().enumerate() {
                        let _ = writeln!(rep, "explore: door {i} section {} at ({:.0},{:.0}) type {} to {}", d.sec, d.x, d.y, d.kind, d.to);
                        // a spot the party arrives at must be inside its section's blast zones (else: KO on arrival,
                        // and again at each respawn there)
                        if let Some(e) = m.ext.get(d.sec) {
                            if d.x < e[0] - BLAST_MARGIN || d.x > e[1] + BLAST_MARGIN || d.y > e[2] + BLAST_MARGIN || d.y < e[3] - BLAST_MARGIN {
                                let _ = writeln!(rep, "explore: door {i} lies outside section {}'s blast zones (extent {:.0},{:.0},{:.0},{:.0})", d.sec, e[0], e[1], e[2], e[3]);
                            }
                        }
                    }
                    for (i, b) in m.blocks.iter().enumerate() {
                        let look = m.brick_models.get(b.model).map(|x| x.0.as_str()).unwrap_or("none");
                        let _ = writeln!(rep, "explore: block {i} section {} at ({:.0},{:.0}) size {:.0}x{:.0} model {look}", b.sec, b.x, b.y, 2.0 * b.hw, 2.0 * b.hh);
                    }
                }
                Err(e) => {
                    let _ = writeln!(rep, "explore: no map ({e}); Final Destination as usual");
                }
            }
            *st.msg[k].lock().unwrap_or_else(|e| e.into_inner()) = rep;
            r.ok()
        })
        .as_ref()
}

/// GD stage `id`'s map (game thread). Waits for a loader still running (only at the stage load that needs it).
pub fn map_of(id: i32) -> Option<&'static Map> {
    let st = maps();
    let r = load_into(st, id);
    let k = slot_of(id);
    if !st.told[k].swap(true, std::sync::atomic::Ordering::Relaxed) {
        eprint!("{}", st.msg[k].lock().unwrap_or_else(|e| e.into_inner()));
    }
    r
}

/// Petey's arena, loaded once on a helper thread; `None` without the owner's files.
pub fn arena() -> Option<&'static Map> {
    static M: std::sync::OnceLock<Option<Map>> = std::sync::OnceLock::new();
    M.get_or_init(|| {
        let r = std::thread::spawn(|| load_secs(&arena_secs(), true, false)).join().unwrap_or_else(|_| Err("loader panicked".into()));
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
    melee_core::hash::CONTENT_WORD.store(0, core::sync::atomic::Ordering::Relaxed);
    // a match begins: the uploads made ahead may age out once unused (presentation only)
    PRELOADED = 0;
    if let Some(u) = TEX_UNPIN {
        u();
    }
    ACTIVE = false;
    SEC = 0;
    FADE = 0;
    DEST = -1;
    UP_PREV = [false; 6];
    CLOCK = 0;
    DOORS_USED = 0;
    SPAWN_SEC = -1;
    SPAWN_NEXT = 0;
    SPAWN_N = 0;
    CRATE_NEXT = i32::MAX;
    VISITED = 1;
    BROKEN = 0;
    STOCKS_SEEN = [-1; 6];
    REBIRTH = [0.0, 0.0];
    CARD = 0;
    ARENA = false;
    if sg::stage_info.grkind as i32 != 37 {
        return core::ptr::null_mut();
    }
    let m = if ARENA_WANT && (crate::tu::envoy::armed() != 0 || crate::tu::petey::vs_selected()) {
        let Some(a) = arena() else { return core::ptr::null_mut() };
        ARENA = true;
        a
    } else {
        let want = if crate::tu::envoy::armed() != 0 { if WANT { GD_STAGE_SSE_LAKE3 } else { 0 } } else { PICK };
        if want == 0 {
            return core::ptr::null_mut();
        }
        CUR_ID = want;
        let Some(m) = map() else { return core::ptr::null_mut() };
        // the content word the session hash carries (slice 0): pick, mask, digest
        let mut h = melee_core::hash::StateHasher::new();
        h.write(&want.to_be_bytes());
        h.write(&effective_mask().to_be_bytes());
        h.write(&m.digest.to_be_bytes());
        let w = (h.finish() as u32 ^ (h.finish() >> 32) as u32) | 1;
        melee_core::hash::CONTENT_WORD.store(w, core::sync::atomic::Ordering::Relaxed);
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

/// The floor under a start or arrival spot: the first floor the segment from 80 above it to 300 below it meets (a
/// spot's bone or door can sit a little off its floor, above or below it; a fighter set down under the floor fell through
/// the room: sse16 and sse22's starts). The spot's own height when there is none.
unsafe fn floor_near(x: f32, y: f32) -> f32 {
    floor_at(x, y).unwrap_or(y)
}

/// The floor nearest a spot: below it first (10 above to 300 below), else above it (up to 80); `None` when neither.
unsafe fn floor_at(x: f32, y: f32) -> Option<f32> {
    let mut at: melee_types::Vec = core::mem::zeroed();
    let mut seg = |y0: f32, y1: f32| melee_world::tu::mplib::mpCheckMultiple(x, y0, x, y1, &mut at, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), 1, -1, -1) != 0;
    if seg(y + 10.0, y - 300.0) || seg(y + 80.0, y + 10.0) { Some(at.y) } else { None }
}

/// The camera frames the party only (Brawl's adventure camera, `cmAdventureController`, follows the players; a foe or
/// a CPU off in the section pulled Melee's VS framing wide and shallow): a fighter that is not a human player (in an
/// Envoy run: not one of the party's slots) has its camera subject held inactive (`CmSubject.force_inactive`, as the
/// Home-Run Contest's stage does). With no human in the room the VS framing stays.
unsafe fn camera_party() {
    let human = |slot: i32| melee_fighter::tu::player::Player_GetPlayerSlotType(slot) as i32 == 0;
    let party = |slot: i32| if crate::tu::envoy::armed() != 0 { slot < if crate::tu::envoy::PLAN[crate::tu::envoy::P_COOP] == 1 { 2 } else { 1 } } else { human(slot) };
    if !(0..6).any(|s| party(s)) {
        return;
    }
    let mut g = *sg::HSD_GObjPLinkHead.offset(HSD_GOBJ_PLINK_FIGHTER as isize);
    while !g.is_null() {
        let fp = sg::HSD_GObjGetUserData(g) as *mut Fighter;
        if !fp.is_null() && !(*fp).x890_cameraBox.is_null() {
            (*(*fp).x890_cameraBox).set_force_inactive(!party((*fp).player_idx as i32) as u8);
        }
        g = (*g).next as *mut HSD_GObj;
    }
}

/// Log the party's positions with the camera bounds and blast zones in force (world space).
unsafe fn log_bounds(what: &str) {
    let ci = &sg::stage_info.cam_info;
    let (ox, oy) = (ci.cam_x_offset, ci.cam_y_offset);
    let c = &ci.cam_bounds;
    let b = &sg::stage_info.blast_zone;
    eprintln!(
        "explore: {what} section {SEC} room frame {CLOCK}: camera ({:.0},{:.0},{:.0},{:.0}) blast ({:.0},{:.0},{:.0},{:.0})",
        c.left + ox, c.right + ox, c.top + oy, c.bottom + oy, b.left + ox, b.right + ox, b.top + oy, b.bottom + oy
    );
    each_fighter(|_, fp| {
        let p = (*fp).cur_pos;
        eprintln!("explore: {what} player {} at ({:.0},{:.0})", (*fp).player_idx, p.x, p.y);
    });
}

/// Stocks last seen per slot (logs only: a stock lost is printed with where the fighter respawns).
static mut STOCKS_SEEN: [i32; 6] = [-1; 6];

/// Stock losses in the room, logged; and, checks only (`MELEE_EXPLORE_KO=player@frame,..`), the fighter in slot
/// `player` put just under its section's blast zone at room frame `frame` (a KO without a fight).
unsafe fn ko_watch() {
    if let Some(&(_, who)) = goto_list("MELEE_EXPLORE_KO").iter().find(|&&(f, _)| f == CLOCK) {
        if let Some(e) = cur().and_then(|m| m.ext.get(SEC as usize).copied()) {
            each_fighter(|_, fp| {
                if (*fp).player_idx as i32 == who {
                    eprintln!("explore: test KO player {who} at room frame {CLOCK}");
                    (*fp).cur_pos.y = e[3] - BLAST_MARGIN - 40.0;
                    (*fp).prev_pos = (*fp).cur_pos;
                }
            });
        }
    }
    for slot in 0..6 {
        if melee_fighter::tu::player::Player_GetPlayerSlotType(slot) as i32 == 3 {
            continue;
        }
        let s = melee_fighter::tu::player::Player_GetStocks(slot) as i32;
        let was = STOCKS_SEEN[slot as usize];
        STOCKS_SEEN[slot as usize] = s;
        if was > s && CLOCK > 1 {
            eprintln!("explore: player {slot} lost a stock ({s} left) in section {SEC} at room frame {CLOCK}");
        }
    }
}

/// `stage::EXPLORE_REBIRTH`: the respawn point (Final Destination's rebirth joint `i`'s point, in `p`) moved into the
/// party's section. See `rebirth_point`.
unsafe fn rebirth_hook(p: *mut Vec3, i: i32) {
    if !ACTIVE || ARENA {
        return;
    }
    let Some(m) = map() else { return };
    let q = rebirth_point(m, i, (*p).x, (*p).y);
    eprintln!("explore: respawn point {i} in section {SEC}: ({:.0},{:.0}) -> ({:.0},{:.0}) at room frame {CLOCK}", (*p).x, (*p).y, q[0], q[1]);
    (*p).x = q[0];
    (*p).y = q[1];
}

/// The respawn point in the party's section for Final Destination's rebirth point (`x`, `y`).
/// Brawl (`stAdventure2::getFighterReStartPos`, 0x81A06050, rebirth mode 0): player 1 comes back at one stored
/// point, which the door warp (0x81A05814) and the section start (0x81A06C6C) set to the arrival position; player 2
/// comes back on player 1, or at the stored point when player 1 is gone. Final Destination's point keeps only its
/// height above Final Destination's floor (the platform's drop); the place is the section's.
unsafe fn rebirth_point(_m: &Map, i: i32, _x: f32, y: f32) -> [f32; 2] {
    let mut at = REBIRTH;
    if i != 0 {
        each_fighter(|_, fp| {
            if (*fp).player_idx == 0 {
                at = [(*fp).cur_pos.x, (*fp).cur_pos.y];
            }
        });
    }
    [at[0], at[1] + y]
}

/// The stored respawn point (world space; Brawl's `stAdventure2` rebirth position): the section's entry at the
/// room's start, the arrival door after each door. Snapshot covered (static).
pub static mut REBIRTH: [f32; 2] = [0.0, 0.0];

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
        // Brawl's section start sets the respawn point to the start position (0x81A06C6C)
        REBIRTH = m.entries.get(SEC as usize).copied().unwrap_or([0.0, 0.0]);
        // and the party stands at that start position, as the door warp places it (Final Destination's own spawn
        // points sit at its origin, where a section's floor need not be: sse22 dropped the party through nothing)
        let e = REBIRTH;
        REBIRTH[1] = floor_near(e[0], e[1]);
        let mut k = 0;
        each_fighter(|g, fp| {
            // side by side; a place with no floor (past a ledge) takes the spot itself
            let x = e[0] + if k % 2 == 0 { -6.0 } else { 6.0 } * ((k / 2 + 1) as f32);
            match floor_at(x, e[1]) {
                Some(y) => place(g, fp, x, y),
                None => place(g, fp, e[0], floor_near(e[0], e[1])),
            }
            k += 1;
        });
        log_bounds("start");
    }
    // the section's enemies (the Subspace enemy lane: plain sim entities, no Melee articles, so a VS match spawns
    // them too) and every crate of the map
    if CLOCK == 2 {
        spawn_section(m, 0);
        CRATE_NEXT = 0;
    }
    // one enemy and one crate a frame (all at once was 2-4 ms in the match's setup tick)
    spawn_step(m);
    if CLOCK >= 2 && (CRATE_NEXT as usize) < m.crates.len() {
        spawn_crate(&m.crates[CRATE_NEXT as usize]);
        CRATE_NEXT += 1;
    }
    // outside an Envoy run nothing else ticks the enemy pool (the mod host's post chain does inside one); its own
    // frame latch keeps it to one step per logic frame either way
    if crate::tu::envoy::armed() == 0 {
        crate::tu::subspace::sub_tick();
    }
    if FADE == 0 {
        screen_kos(m);
    }
    camera_party();
    blocks_tick(m);
    movers_tick(m);
    card_tick();
    ko_watch();
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
            // Brawl's door warp sets the respawn point to the arrival (0x81A05814)
            REBIRTH = [d.x, floor_near(d.x, d.y)];
            DOORS_USED += 1;
            eprintln!("explore: through a door to section {} ({:.0},{:.0}) at room frame {}", d.sec, d.x, d.y, CLOCK);
            log_bounds("arrival");
        }
        if FADE >= FADE_LEN {
            FADE = 0;
            DEST = -1;
        }
        return;
    }
    // checks only (`MELEE_EXPLORE_GOTO=door@frame[,door@frame..]`): the party goes through to `door` at room frame
    // `frame`, as if it had opened the door leading there (screenshots inside the side rooms without a walk)
    if let Some(&(_, to)) = goto_list("MELEE_EXPLORE_GOTO").iter().find(|&&(f, to)| f == CLOCK && (to as usize) < m.doors.len()) {
        DEST = to;
        FADE = 1;
        eprintln!("explore: test goto door {to} at room frame {CLOCK}");
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
            let near = d.kind != ENTRY && d.sec as i32 == SEC && (p.x - d.x).abs() <= 15.0 && p.y >= d.y - 55.0 && p.y <= d.y + 20.0;
            let score = |d: &Door| if d.to < 0 { if d.exit { 1 } else { 0 } } else if VISITED & 1u32.pshl(m.doors[d.to as usize].sec.min(31) as i64) == 0 { 3 } else { 2 };
            if near && (want < 0 || score(d) > score(&m.doors[want as usize])) {
                want = di as i32;
                opener = g;
            }
        }
    });
    // checks only (`MELEE_EXPLORE_OPEN=door@frame,..`): door `door` of the party's section is opened at room frame
    // `frame`, as a player's up tilt at it would (a story walk takes the way on with it)
    if let Some(&(_, d)) = goto_list("MELEE_EXPLORE_OPEN").iter().find(|&&(f, d)| f == CLOCK && (d as usize) < m.doors.len()) {
        if m.doors[d as usize].sec as i32 == SEC {
            want = d;
            eprintln!("explore: test open door {d} at room frame {CLOCK}");
        }
    }
    if want >= 0 {
        let d = m.doors[want as usize];
        DEST = if d.to >= 0 {
            d.to
        } else if d.to == STORY_NEXT {
            story_next();
            -1
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

/// `MELEE_EXPLORE_GOTO` / `MELEE_EXPLORE_OPEN` / `MELEE_EXPLORE_KO` as (room frame, door or player) pairs; empty when unset.
fn goto_list(var: &'static str) -> &'static [(i32, i32)] {
    static G: std::sync::OnceLock<Vec<(i32, i32)>> = std::sync::OnceLock::new();
    static O: std::sync::OnceLock<Vec<(i32, i32)>> = std::sync::OnceLock::new();
    static K: std::sync::OnceLock<Vec<(i32, i32)>> = std::sync::OnceLock::new();
    let cell = match var {
        "MELEE_EXPLORE_OPEN" => &O,
        "MELEE_EXPLORE_KO" => &K,
        _ => &G,
    };
    cell.get_or_init(|| {
        std::env::var(var)
            .unwrap_or_default()
            .split(',')
            .filter_map(|p| {
                let (d, f) = p.split_once('@')?;
                Some((f.trim().parse().ok()?, d.trim().parse().ok()?))
            })
            .collect()
    })
}

/// Section `s`'s enemy points, each as its own Brawl kind's nearest Subspace kind (`enemy_kind`); the previous
/// section's enemies go (Brawl rebuilds a room's generators on entry too). At most the pool's size.
/// The enemy spawn hook (the Geno enemy actors): (Brawl kind, x, y, facing, section) -> whether it spawned the enemy.
/// Called at each map enemy point when its section is entered, within the same pool cap; the room's own clear on a
/// door (`subspace::clear_room`) stays the caller's to mirror.
pub static mut ENEMY_HOOK: Option<unsafe fn(u16, f32, f32, i32, usize) -> bool> = None;

/// The section whose enemy points are being spawned (-1 none), the next point to look at and how many went in.
/// One a room frame (`spawn_step`). Snapshot covered (statics).
static mut SPAWN_SEC: i32 = -1;
static mut SPAWN_NEXT: i32 = 0;
static mut SPAWN_N: i32 = 0;
/// The next crate to place (one a room frame from room frame 2; past the end: done).
static mut CRATE_NEXT: i32 = i32::MAX;

/// Section `s` was entered: the room's own enemies go now, its points come in one a frame (`spawn_step`).
unsafe fn spawn_section(_m: &Map, s: usize) {
    // only the map's own: the Envoy sim's enemies (its foes for the fight) stay, and keep half the pool free for it
    crate::tu::subspace::clear_room();
    SPAWN_SEC = s as i32;
    SPAWN_NEXT = 0;
    SPAWN_N = 0;
}

/// Every room frame: the next enemy point of the section being filled, if any.
unsafe fn spawn_step(m: &Map) {
    use crate::tu::subspace as ss;
    if SPAWN_SEC < 0 {
        return;
    }
    let s = SPAWN_SEC as usize;
    let cap = if crate::tu::envoy::armed() != 0 { ss::MAX / 2 } else { ss::MAX };
    while (SPAWN_NEXT as usize) < m.spawns.len() {
        let (si, x, y, face, id) = m.spawns[SPAWN_NEXT as usize];
        SPAWN_NEXT += 1;
        if si != s || SPAWN_N >= cap as i32 {
            continue;
        }
        let n = &mut *core::ptr::addr_of_mut!(SPAWN_N);
        // the Geno enemy lane's actors take a point first (Brawl kind, place, facing, section); the hand-made
        // Subspace kinds are the stand-in until they do
        if let Some(hook) = ENEMY_HOOK {
            if hook(id, x, y, face, s) {
                *n += 1;
                return;
            }
        }
        let (k, name) = enemy_kind(id);
        let Some(k) = k else {
            eprintln!("explore: enemy point kind {id} ({name}) not spawned");
            continue;
        };
        let h = ss::spawn(k, 0, x, y + 5.0, face);
        eprintln!("explore: enemy {name} (Brawl kind {id}) as {} at ({x:.0},{y:.0}) section {s} -> {h}", ss::kind_name(k));
        *n += 1;
        return;
    }
    SPAWN_SEC = -1;
}

/// Every GIB2 crate of the map as a Melee container (small: capsule, crate: box, big: barrel), once per room. Its
/// contents follow the match's item switch, as any Melee container's.
unsafe fn spawn_crate(c: &Crate) {
    {
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

/// The path of the bone named `name` in the section's models, from the first CHR0 that moves it: its model-space
/// (x, y) at every frame. None when no model has such a bone or nothing animates it.
fn mover_path(leaves: &[Vec<u8>], cache: &mut Option<Vec<pm::Rig>>, name: &[u8]) -> Option<Vec<[f32; 2]>> {
    let rigs = cache.get_or_insert_with(|| {
        let mut names: Vec<String> = vec![];
        for b in leaves.iter().filter(|b| b.len() >= 0x10 && &b[..4] == b"bres") {
            for (n, off) in pm::brres_entries(b).unwrap_or_default() {
                if b.get(off..off + 4) == Some(b"MDL0") && !names.contains(&n) {
                    names.push(n);
                }
            }
        }
        let want: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        pm::prop_rigs(leaves, &want).into_iter().flatten().collect()
    });
    let name = std::str::from_utf8(name).ok()?;
    for r in rigs.iter() {
        let Some(k) = (0..r.bones.len()).find(|&k| r.bone_name(k) == name) else { continue };
        let Some((_, a)) = r.anims.iter().find(|(_, a)| a.tracks.iter().any(|t| t.0 == name)) else { continue };
        let n = fctiwz_f32(a.frames).clamp(1, 3600) as usize;
        let path: Vec<[f32; 2]> = (0..n).map(|f| {
            let p = r.bone_point(Some(a), f as f32, k, [0.0; 3]);
            [p[0], p[1]]
        }).collect();
        if path.iter().any(|p| (p[0] - path[0][0]).abs() > 0.01 || (p[1] - path[0][1]).abs() > 0.01) {
            return Some(path);
        }
    }
    // a part whose motion is Brawl code, not a bone animation (The Lake's raft `ikada` in 080105, sse27's lifts): its
    // collision held at the bone's rest place (a one-point path), so the floor it carries is there; left out, the
    // section had no floor and the party fell to the blast zone on arrival and at each respawn
    // (not item boxes, gates, triggers or blasts: a box is an item, a shutter held shut would close the way on)
    let keep = !["ItemBox", "Shutter", "Trigger", "Explosion", "Spike"].iter().any(|w| name.contains(w));
    for r in rigs.iter().filter(|_| keep) {
        if let Some(k) = (0..r.bones.len()).find(|&k| r.bone_name(k) == name) {
            let p = r.bone_point(None, 0.0, k, [0.0; 3]);
            return Some(vec![[p[0], p[1]]]);
        }
    }
    None
}

/// A placement's motion path (Brawl's grGimmickMotionPath, read from the code: the enemy creator 0x81A12300
/// region of sora_adv_stage.rel builds one when the placement's model index byte [off] is not 0xFF, with
/// `grGimmickMotionPath::create(index, "path_locator", ...)` (0x81C6E13C); `startup` loads model data `index` from the
/// section's archive; the creator's path object goes into the create parameter (0x81A130A0) and from there to the enemy
/// (value 41000). The position at a frame is the `path_locator` node after the model's CHR0 is set to that frame
/// (getTRS 0x81C6EB6C: setFrameModelAnim, then the node's translation).
struct EnemyPath {
    rig: pm::Rig,
    anim: usize,
    bone: usize,
    off: [f32; 2],
    frames: u32,
}

impl EnemyPath {
    fn pos(&self, f: f32) -> [f32; 3] {
        let p = self.rig.bone_point(self.rig.anims.get(self.anim).map(|a| &a.1), f, self.bone, [0.0; 3]);
        [p[0] + self.off[0], p[1] + self.off[1], p[2]]
    }
}

/// The motion path of the GEG1 placement in BLOC leaf `bi` whose model index is `idx`: model data (archive file type
/// 2) `idx` next to the BLOC in the same archive. usize::MAX when there is none (0xFF) or it has no animated
/// `path_locator`.
fn placement_path(leaves: &[Vec<u8>], tags: &[Vec<(u16, u16)>], bi: usize, idx: u8, off: [f32; 2], out: &mut Vec<EnemyPath>, notes: &mut Vec<String>) -> usize {
    if idx == 0xFF {
        return usize::MAX;
    }
    let Some(t) = tags.get(bi) else { return usize::MAX };
    let mut want = t[..t.len().saturating_sub(1)].to_vec();
    want.push((2, idx as u16));
    let Some(li) = tags.iter().position(|x| *x == want) else {
        notes.push(format!("explore: placement path {idx}: no model data"));
        return usize::MAX;
    };
    if let Some(r) = pm::locator_rig(&leaves[li]) {
        let Some(bone) = (0..r.bones.len()).find(|&k| r.bone_name(k) == "path_locator") else { return usize::MAX };
        let Some(anim) = r.anims.iter().position(|(_, a)| a.tracks.iter().any(|t| t.0 == "path_locator")) else { return usize::MAX };
        let frames = fctiwz_f32(r.anims[anim].1.frames).max(1) as u32;
        out.push(EnemyPath { rig: r, anim, bone, off, frames });
        return out.len() - 1;
    }
    notes.push(format!("explore: placement path {idx}: no animated path_locator"));
    usize::MAX
}

/// `vm::Path::pos` for the explore maps' placement paths: `id` = map slot << 16 | path index.
fn enemy_path_pos(id: usize, f: f32) -> [f32; 3] {
    maps().m.get(id >> 16)
        .and_then(|m| m.get())
        .and_then(|m| m.as_ref())
        .and_then(|m| m.enemy_paths.get(id & 0xFFFF))
        .map(|p| p.pos(f))
        .unwrap_or([0.0; 3])
}

/// The enemy VM's inputs for spawn `k` of GD stage `id`'s map: its placement path (value 41000) and its home (the
/// GEG1 spawn position, world space).
pub fn enemy_vm_place(id: i32, k: usize) -> Option<(Option<brawl_boss::vm::Path>, [f32; 2])> {
    let m = maps().m[slot_of(id)].get()?.as_ref()?;
    let s = m.spawns.get(k)?;
    let pi = m.spawn_path.get(k).copied().unwrap_or(usize::MAX);
    let path = m.enemy_paths.get(pi).map(|p| brawl_boss::vm::Path { id: slot_of(id) << 16 | pi, frames: p.frames, pos: enemy_path_pos });
    Some((path, [s.1, s.2]))
}

/// The moving platforms of the party's section: each joint's points follow its bone's path (the frame is the room
/// clock), the previous place kept for the riders (`mpLib_80057424`), the joint's bounds refreshed.
unsafe fn movers_tick(m: &Map) {
    use melee_world::tu::mplib as mp;
    for (k, b) in m.blocks.iter().enumerate() {
        if b.path == usize::MAX || b.sec as i32 != SEC {
            continue;
        }
        let Some(path) = m.paths.get(b.path) else { continue };
        let f = CLOCK.max(0).prem(path.len() as i32) as usize;
        let (dx, dy) = (path[f][0] - path[0][0], path[f][1] - path[0][1]);
        let j = k + 1;
        let Some(mj) = m.joints.get(j) else { continue };
        mp::mpLib_80057424(j as i32);
        for v in mj.vtx_start as usize..(mj.vtx_start as usize + mj.vtx_count as usize) {
            let (Some(base), vx) = (m.verts.get(v), mp::groundCollVtx.add(v)) else { continue };
            (*vx).pos.x = base.x + dx;
            (*vx).pos.y = base.y + dy;
        }
        mp::mpJointUpdateBounding(j as i32);
        ride_check(k, mj, path.len() as i32);
        if CLOCK % 100 == 0 && std::env::var_os("MELEE_EXPLORE_LOG").is_some() {
            let v = mp::groundCollVtx.add(mj.vtx_start as usize);
            eprintln!("explore: frame {} platform {k} at ({:.1},{:.1})", CLOCK, (*v).pos.x, (*v).pos.y);
        }
    }
}

/// Checks only (`MELEE_EXPLORE_RIDE=block@frame`): at that room frame player 1 is set down just above moving block
/// `block`'s top, then for one full loop of its path the fighter's place relative to the platform is tracked: it must
/// stay grounded, never fall below the top, and move with the platform (no jitter). One summary line at the end.
/// The rider is the first fighter in play (player 1 of a `--match`).
unsafe fn ride_check(k: usize, mj: &MapJoint, len: i32) {
    use melee_world::tu::mplib as mp;
    static mut AT: i32 = i32::MIN;
    static mut WHO: usize = 0;
    // (frames tracked, frames airborne after landing, min/max rel x, min/max rel y, max per-frame step of rel y, last)
    static mut ST: (i32, i32, f32, f32, f32, f32, f32, f32, bool) = (0, 0, f32::MAX, f32::MIN, f32::MAX, f32::MIN, 0.0, 0.0, false);
    static SPEC: std::sync::OnceLock<Option<(usize, i32)>> = std::sync::OnceLock::new();
    let Some((want, at)) = *SPEC.get_or_init(|| {
        let v = std::env::var("MELEE_EXPLORE_RIDE").ok()?;
        let (b, f) = v.split_once('@')?;
        Some((b.trim().parse().ok()?, f.trim().parse().ok()?))
    }) else { return };
    if k != want {
        return;
    }
    let top = || {
        let mut t = f32::MIN;
        let (mut l, mut r) = (f32::MAX, f32::MIN);
        for v in mj.vtx_start as usize..(mj.vtx_start as usize + mj.vtx_count as usize) {
            let p = (*mp::groundCollVtx.add(v)).pos;
            t = fmax(t, p.y);
            l = fmin(l, p.x);
            r = fmax(r, p.x);
        }
        ((l + r) * 0.5, t)
    };
    if CLOCK == at {
        let (cx, ty) = top();
        let mut done = false;
        each_fighter(|g, fp| {
            if !done {
                WHO = g as usize;
                place(g, fp, cx, ty + 4.0);
                done = true;
            }
        });
        AT = at;
        ST = (0, 0, f32::MAX, f32::MIN, f32::MAX, f32::MIN, 0.0, 0.0, false);
        eprintln!("explore: ride: the first fighter set on platform {k} at ({cx:.1},{:.1}), room frame {CLOCK}", ty + 4.0);
        return;
    }
    // 30 frames to land, then one loop
    if AT == i32::MIN || CLOCK < AT + 30 || CLOCK > AT + 30 + len {
        return;
    }
    each_fighter(|g, fp| {
        if g as usize != WHO {
            return;
        }
        // relative to the platform's first point (it moves rigidly): constant while the fighter rides it
        let v0 = (*mp::groundCollVtx.add(mj.vtx_start as usize)).pos;
        let (rx, ry) = ((*fp).cur_pos.x - v0.x, (*fp).cur_pos.y - v0.y);
        let (_, ty) = top();
        let ground = (*fp).ground_or_air as i32 == GA_Ground as i32;
        let st = &mut *core::ptr::addr_of_mut!(ST);
        if st.0 > 0 {
            st.6 = fmax(st.6, (ry - st.7).abs());
        }
        st.7 = ry;
        st.0 += 1;
        if !ground {
            st.1 += 1;
        }
        st.2 = fmin(st.2, rx);
        st.3 = fmax(st.3, rx);
        let _ = ty;
        st.4 = fmin(st.4, ry);
        st.5 = fmax(st.5, ry);
        if CLOCK == AT + 30 + len {
            // the fighter is placed from the platform's previous frame (mpLib_80057424's prev_pos), so its place on the
            // platform swings by about one frame's travel round the loop (~1.3 here); a jitter or a slip is a jump
            let ok = st.1 == 0 && st.5 - st.4 < 2.0 && st.6 < 0.1 && st.3 - st.2 < 2.0;
            eprintln!(
                "explore: ride {}: platform {k}, {} frames (one loop), airborne {}, height over the platform's first point {:.2}..{:.2}, x on the platform {:.2}..{:.2}, worst step {:.3}",
                if ok { "OK" } else { "FAILED" }, st.0, st.1, st.4, st.5, st.2, st.3, st.6
            );
        }
    });
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
        if b.path != usize::MAX || BROKEN & 1u64.pshl(k as i64) != 0 || b.sec as i32 != SEC {
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

/// Room frames between two more section models joining the draw (`draw`).
const WARM_STEP: usize = 2;

/// `ftdrawcommon::MAP_DRAW`: the section models, the doors and the transition fade, once per frame (player 1's
/// opaque pass). Presentation only.
pub unsafe fn draw(g: *mut HSD_GObj, pass: i32) {
    if !ACTIVE || pass != 0 {
        return;
    }
    // once per frame: in the first in-play fighter's pass
    let mut first: *mut HSD_GObj = core::ptr::null_mut();
    let (mut fx, mut fy) = (0.0f32, 0.0f32);
    each_fighter(|fg, fp| {
        if first.is_null() {
            first = fg;
            fx = (*fp).cur_pos.x;
            fy = (*fp).cur_pos.y;
        }
    });
    if first != g {
        return;
    }
    let Some(m) = cur() else { return };
    let _prof = melee_core::frameprof::Span::new(melee_core::frameprof::Kind::Fighter, || melee_core::frameprof::Text::new().s("explore map draw"));
    // The section models come in one at a time over the room's first frames, nearest the party first: every section is
    // drawn from then on (its textures stay in the renderer's cache, so a door never uploads anything), but no one
    // frame converts and uploads the whole map's textures (Lake 50 / 14.8 MB, Jungle 67 / 18.5 MB: the setup frame's
    // biggest part). Counted off the room clock (sim state), so nothing presentation-side is kept.
    let (px, py) = (fx, fy);
    let mut order: [(f32, u16); 32] = [(f32::MAX, 0); 32];
    let n = m.models.len().min(32);
    for (i, (_, _, off)) in m.models.iter().take(n).enumerate() {
        order[i] = ((off[0] - px).abs() + (off[1] - py).abs(), i as u16);
    }
    order[..n].sort_by(|a, b| a.0.total_cmp(&b.0));
    // checks only: MELEE_EXPLORE_WARM=all draws every section from the first frame (the old way, for an A/B profile)
    static ALL: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    let all = *ALL.get_or_init(|| std::env::var("MELEE_EXPLORE_WARM").is_ok_and(|v| v == "all"));
    let warm = if all { n } else { ((CLOCK.max(0) as usize) / WARM_STEP + 1).min(n) };
    // Only the party's section is drawn whole; every other one is drawn as one triangle per material, out of the
    // camera's view, so its textures stay in the renderer's cache (a texture unused for 120 frames is dropped) and its
    // first full draw after a door uploads nothing. Full draws of every section cost 3-4 ms a frame (52-62 k
    // immediate-mode triangles).
    let e = m.ext.get(SEC.max(0) as usize).copied();
    let here = |off: &[f32; 2]| all || e.map_or(true, |e| off[0] >= e[0] - 500.0 && off[0] <= e[1] + 500.0 && off[1] <= e[2] + 500.0 && off[1] >= e[3] - 500.0);
    for &(_, i) in &order[..warm] {
        let (r, t, off) = &m.models[i as usize];
        pm::SAMPLE = !here(off);
        match m.lists.get(i as usize) {
            Some(l) if !pm::SAMPLE => pm::draw_tris_lists(r, t, [0.0, 0.0, 0.0], l),
            _ => pm::draw_tris(r, t, [0.0, 0.0], [off[0], off[1], 0.0], 1.0, 0.0, false),
        }
        pm::SAMPLE = false;
    }
    for (r, t, off) in m.models.iter().skip(32) {
        pm::draw_tris(r, t, [0.0, 0.0], [off[0], off[1], 0.0], 1.0, 0.0, false);
    }
    // the bricks still standing in the party's section (a broken one's model goes with its collision)
    for (k, b) in m.blocks.iter().enumerate() {
        let broken = k < 64 && BROKEN & 1u64.pshl(k as i64) != 0;
        if broken || b.sec as i32 != SEC {
            continue;
        }
        if let Some((_, r, t)) = m.brick_models.get(b.model) {
            pm::draw_tris(r, t, [0.0, 0.0], [b.at[0], b.at[1], 0.0], 1.0, 0.0, false);
        }
    }
    if let Some((r, t)) = m.door_models.first() {
        for d in m.doors.iter().filter(|d| d.kind != ENTRY) {
            pm::draw_tris(r, t, [0.0, 0.0], [d.x, d.y, 0.0], 1.0, 0.0, false);
        }
    }
    if CARD > 0 {
        pm::draw_fade(255);
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

// ---------------------------------------------------------------- the story table (data) and its entry points

/// What Brawl does at a story point. Brawl plays a cutscene at each `Scene`; the port never plays one: a scene slot
/// names the game scene that runs there (`SceneKind`), so the owner (or Envoy) can choose per slot later.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// a GD stage map (its sections chained in story order, `secs_of`)
    Map(i32),
    /// a boss fight (the Geno boss lane plugs in through `boss::BOSS_HOOK`)
    Boss(BossKind),
    /// a cutscene spot: our slot id and what runs there
    Scene(u8, SceneKind),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BossKind {
    Petey,
    Rayquaza,
    Porky,
    Galleom,
    Ridley,
    Duon,
    MetaRidley,
    Tabuu,
}

/// What runs in a scene slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneKind {
    /// a plain placeholder card (black, 2 s)
    Card,
    /// an Envoy fight
    EnvoyFight,
    /// Envoy's camp (rest, shop)
    Camp,
    /// a results card
    Results,
}

/// How the party's state carries into a step: stocks and damage as they are, or damage cleared (a rest).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Carry {
    Keep,
    Rest,
}

/// The story table: (step, carry) in story order, for the levels the port has. Scene slots sit where Brawl's level
/// data starts or ends a level or a boss (the level-exit block GMSJ, a level's last door, a boss stage's EPBM boss
/// mark); Brawl's cutscenes themselves (`movie/`) are never read. Slot ids are ours.
pub const STORY: &[(Step, Carry)] = &[
    (Step::Scene(1, SceneKind::Card), Carry::Keep),    // Jungle opening (before 060001)
    (Step::Map(GD_STAGE_SSE_JUNGLE), Carry::Keep),     // 060001 -> 060002 (+a) -> 060003 -> 060004 (+a)
    (Step::Scene(2, SceneKind::Camp), Carry::Rest),    // Jungle end (060004's last door, 06000402)
    (Step::Scene(3, SceneKind::Card), Carry::Keep),    // Lake opening (before 080101)
    (Step::Map(GD_STAGE_SSE_LAKE3), Carry::Keep),      // 080103 (+a, b) -> 080104 (+a, b) -> 080105 (+a)
    (Step::Scene(4, SceneKind::Camp), Carry::Rest),    // before Rayquaza (080105's GMSJ exit)
    (Step::Boss(BossKind::Rayquaza), Carry::Keep),     // 080001
    (Step::Scene(5, SceneKind::Results), Carry::Keep), // after Rayquaza
    (Step::Map(GD_STAGE_SSE_16), Carry::Keep),         // 160201 -> 160202 (clear 16020201)
    (Step::Map(GD_STAGE_SSE_22), Carry::Keep),         // 220001 -> 220002 (+a) (clear 22000203)
    (Step::Map(GD_STAGE_SSE_27), Carry::Keep),         // 270001 -> 270002 (+a) (clear 27000202)
];

/// The story step the party is on (index into `STORY`; -1 outside the story). Snapshot covered.
pub static mut STORY_AT: i32 = -1;
/// The running placeholder card: frames left (0 none) and what it is (a slot id, or 100 + boss kind).
pub static mut CARD: i32 = 0;
pub static mut CARD_WHAT: i32 = 0;
/// The end event of the last thing started through an entry point; read and cleared by the caller (Envoy, Story).
pub static mut END: i32 = 0;
pub const END_NONE: i32 = 0;
/// the party left by the level's way on (its last main section's exit door)
pub const END_EXIT: i32 = 1;
pub const END_SCENE: i32 = 2;
pub const END_BOSS: i32 = 3;
/// every player out of stocks (the caller's own rule raises it; the map never ends a match)
pub const END_WIPED: i32 = 4;
const CARD_FRAMES: i32 = 120;

/// Start story step `k` when it is a scene slot or a boss (a map step needs a new stage: the caller's).
unsafe fn story_run(k: usize) {
    let Some((step, _)) = STORY.get(k) else { return };
    STORY_AT = k as i32;
    match *step {
        Step::Scene(slot, _) => scene::start(slot),
        Step::Boss(b) => boss::start(b, 0, 1.0),
        Step::Map(id) => eprintln!("explore: story step {k}: map {id:#x} (a new stage; the caller's)"),
    }
}

/// The level's way on was taken: the end event, then the story step after the current map.
unsafe fn story_next() {
    END = END_EXIT;
    eprintln!("explore: way on taken (end event exit) at room frame {CLOCK}");
    let here = if STORY_AT >= 0 { Some(STORY_AT as usize) } else { STORY.iter().position(|(s, _)| *s == Step::Map(CUR_ID)) };
    if let Some(h) = here {
        story_run(h + 1);
    }
}

/// Every room frame: a running card counts down, ends with its event and runs on into a following scene / boss step.
unsafe fn card_tick() {
    if CARD <= 0 {
        return;
    }
    CARD -= 1;
    if CARD > 0 {
        return;
    }
    END = if CARD_WHAT >= 100 { END_BOSS } else { END_SCENE };
    eprintln!("explore: {} {} ended (placeholder card) at room frame {CLOCK}", if CARD_WHAT >= 100 { "boss" } else { "scene slot" }, CARD_WHAT % 100);
    if STORY_AT >= 0 {
        if let Some((Step::Scene(..) | Step::Boss(_), _)) = STORY.get(STORY_AT as usize + 1) {
            story_run(STORY_AT as usize + 1);
        }
    }
}

/// Envoy / Story entry: play GD stage `id`'s map (every section loaded before the first frame; the party starts in its
/// first section). Returns the retail stage that carries it (Final Destination), or `id` itself when it is not an
/// available GD stage. The end event is `END_EXIT` when the party leaves by the level's way on.
pub fn enter_section(id: i32) -> i32 {
    unsafe {
        END = END_NONE;
        STORY_AT = STORY.iter().position(|(s, _)| *s == Step::Map(id)).map(|i| i as i32).unwrap_or(-1);
    }
    carrier(id)
}

/// The content this player's files allow (envoy-sse-design slice 0): bit k for `GD_STAGES[k]`, bit 30 when the enemy
/// looks are there. 0 with `MELEE_BRAWL_ABSENT`. Netplay plays the AND of both peers' masks.
pub fn content_mask() -> u32 {
    let mut m = 0u32;
    for (k, g) in GD_STAGES.iter().enumerate() {
        if files_present(g.id) {
            m |= 1u32.pshl(k as i64);
        }
    }
    if m != 0 && pm::adventure_present("060001.pac") {
        m |= 1 << 30;
    }
    // checks only: MELEE_SSE_TEST_MASK=<hex> ANDs this machine's mask (a peer "without" a stage, files kept)
    if let Some(t) = std::env::var("MELEE_SSE_TEST_MASK").ok().and_then(|v| u32::from_str_radix(&v, 16).ok()) {
        m &= t;
    }
    m
}

/// GD stage `id`'s collision digest (0 when not available). A section set counts in netplay only when both peers'
/// digests match. Waits for the map's loader (call it before a match, never inside one).
pub fn digest(id: i32) -> u64 {
    if !files_present(id) {
        return 0;
    }
    map_of(id).map(|m| m.digest).unwrap_or(0)
}

/// The mask this session plays: this machine's content AND the session's allowance.
pub fn effective_mask() -> u32 {
    content_mask() & unsafe { ALLOW }
}

/// This machine's slice-0 token for a netplay handshake / a record: `m<mask>` then `.k:<digest>` per GD stage it has
/// (hex). `melee_net::app::room::content_and` combines two of these.
pub fn content_token() -> String {
    let m = content_mask();
    let mut t = format!("m{m:x}");
    for (k, g) in GD_STAGES.iter().enumerate() {
        if m & 1u32.pshl(k as i64) != 0 {
            // folded to 32 bits: the HELLO's guest info is 64 bytes on the wire
            let d = digest(g.id);
            t += &format!(".{k}:{:x}", (d ^ (d >> 32)) as u32);
        }
    }
    t
}

/// The replay header's / run record's SSE line: the pick, the mask played and the picked map's digest
/// (`p<pick>.m<mask>.d<digest>`, hex).
pub fn header_token() -> String {
    let p = unsafe { PICK };
    format!("p{p:x}.m{:x}.d{:x}", effective_mask(), if p != 0 { digest(p) } else { 0 })
}

/// Apply a replay header's SSE line: its mask becomes the allowance and its pick the pick. A pick whose map is not
/// here, or whose digest differs from this machine's, is dropped (the replay then plays Final Destination, and the
/// hash, which carries the content word, says so at once).
pub fn apply_header_token(t: &str) {
    let mut p = 0i32;
    let mut m = 0u32;
    let mut d = 0u64;
    for f in t.split('.') {
        let (k, v) = f.split_at(1.min(f.len()));
        match k {
            "p" => p = i32::from_str_radix(v, 16).unwrap_or(0),
            "m" => m = u32::from_str_radix(v, 16).unwrap_or(0),
            "d" => d = u64::from_str_radix(v, 16).unwrap_or(0),
            _ => {}
        }
    }
    unsafe {
        ALLOW = m;
        PICK = if p != 0 && available(p) && digest(p) == d { p } else { 0 };
        if PICK != 0 {
            preload_textures(PICK);
        }
        eprintln!("explore: replay header sse {t}: pick {:#x}, mask {:x}", PICK, effective_mask());
    }
}

/// Apply a netplay session's agreed mask (`;sse=` of the agreed scene: the host's pick and the AND of both peers'
/// masks). `None` (an older host, or a host with no Brawl content): no SSE content on either side.
pub fn apply_netplay(agreed: Option<(i32, u32)>) {
    unsafe {
        let (p, m) = agreed.unwrap_or((0, 0));
        ALLOW = m;
        PICK = if p != 0 && available(p) { p } else { 0 };
        if PICK != 0 {
            preload_textures(PICK);
        }
        eprintln!("explore: netplay content: mask {:x} (this machine {:x}), pick {:#x}", effective_mask(), content_mask(), PICK);
    }
}

/// Boss fights. No boss runs here: the Geno boss lane plugs its actors in through `BOSS_HOOK`; with no hook a boss
/// step shows a placeholder card, is logged as blocked and ends with `END_BOSS` (never a faked fight).
pub mod boss {
    use super::*;
    /// (kind, tier, hp scale) -> whether the hook took the fight (it then raises `END = END_BOSS` itself).
    pub static mut BOSS_HOOK: Option<unsafe fn(BossKind, i32, f32) -> bool> = None;
    pub fn start(kind: BossKind, tier: i32, hp_scale: f32) {
        unsafe {
            END = END_NONE;
            if let Some(h) = BOSS_HOOK {
                if h(kind, tier, hp_scale) {
                    eprintln!("explore: boss {kind:?} started (tier {tier}, hp x{hp_scale})");
                    return;
                }
            }
            eprintln!("explore: boss {kind:?}: no boss runner yet (Geno boss lane); placeholder card");
            CARD = CARD_FRAMES;
            CARD_WHAT = 100 + kind as i32;
        }
    }
}

/// Scene slots: a placeholder card for every kind for now (a caller may run its own scene instead and raise
/// `END = END_SCENE` itself).
pub mod scene {
    use super::*;
    pub fn start(slot: u8) {
        unsafe {
            END = END_NONE;
            let kind = STORY.iter().find_map(|(s, _)| if let Step::Scene(k, kind) = s { (*k == slot).then_some(*kind) } else { None });
            eprintln!("explore: scene slot {slot} ({kind:?}) started");
            // a camp is a rest (`Carry::Rest`): every fighter's damage goes to 0 as the slot starts (stocks kept).
            // The camp itself is still a card until the Envoy camp is wired in; the rest is applied either way.
            let rest = STORY.iter().any(|(s, c)| matches!(s, Step::Scene(k, _) if *k == slot) && *c == Carry::Rest);
            if rest {
                rest_party();
            }
            CARD = CARD_FRAMES;
            CARD_WHAT = slot as i32;
        }
    }
}

/// `Carry::Rest`: every fighter's percent (and its HUD number) back to 0.
unsafe fn rest_party() {
    let mut n = 0;
    each_fighter(|_, fp| {
        (*fp).dmg.x1830_percent = 0.0;
        melee_fighter::tu::player::Player_SetHUDDamage((*fp).player_idx as i32, 0);
        n += 1;
    });
    eprintln!("explore: rest: {n} fighters at 0% (room frame {CLOCK})");
}

/// Install the hooks (the engine start). Nothing is hooked without the owner's Brawl files (or Petey's mode).
pub fn install() {
    // the loaders below must never getenv (petey_model::brawl_env): settle the Brawl folder settings here, first
    pm::prime_brawl_env();
    if !any_available() && !(crate::tu::petey::selected() || crate::tu::petey::vs_selected()) {
        return;
    }
    unsafe {
        melee_world::tu::ground::EXPLORE_COLL = Some(coll_override);
        melee_world::tu::grlast::EXPLORE_FRAME = Some(room_tick);
        melee_world::tu::stage::EXPLORE_REBIRTH = Some(rebirth_hook);
        melee_world::tu::grdisplay::HIDE_GROUND = Some(hide_ground);
        melee_fighter::tu::ftdrawcommon::MAP_DRAW = Some(draw);
    }
    // start loading now, in the background (the first stage load that needs it waits for it there)
    // the loaders are started from a short-lived helper thread, so their handles and result packets are system-heap
    // allocations: a loader spawned straight from the game thread puts them in the arena, and a rollback load during
    // the (now longer: moving parts) load rewrote them under it (a synctest hung one run in three)
    let st = maps();
    let _ = std::thread::spawn(move || {
        for g in GD_STAGES.iter() {
            if available(g.id) {
                let id = g.id;
                let _ = std::thread::Builder::new().name("explore-map".into()).spawn(move || {
                    let _ = load_into(st, id);
                });
            }
        }
    })
    .join();
    if crate::tu::petey::selected() || crate::tu::petey::vs_selected() {
        let _ = arena();
    }
    // the VS boss's enemy scripts compile on a helper thread at the engine start, not in the match's first frame
    if crate::tu::petey::vs_selected() && !crate::tu::petey::is_petey() {
        let name = crate::tu::petey::boss_name();
        std::thread::spawn(move || {
            crate::tu::geno_vm::prefetch(name);
            crate::tu::vm_boss_model::prefetch(name);
            let _ = crate::tu::petey::vm_start_status();
            let _ = crate::tu::petey::vm_brawl_arena();
        });
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
        let Ok(m) = super::load_secs(&super::arena_secs(), true, false) else { return };
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
    /// With the owner's Brawl files (skipped otherwise): the chain from Brawl's step-jump table: every GD stage has
    /// its sections, a door that clears the segment (the way on), its room doors linked both ways, and every enemy
    /// point a kind.
    #[test]
    fn chains_link() {
        for g in super::GD_STAGES.iter() {
            let secs = super::secs_of(g.id);
            if secs.is_empty() || !super::files_present(g.id) {
                continue;
            }
            let Ok(mut m) = super::load_secs(secs, true, true) else { continue };
            super::link_doors(&mut m, secs, &super::chain_of(g.id).jumps);
            eprintln!("{}: {:?}", g.name, secs.iter().map(|s| s.file.as_str()).collect::<Vec<_>>());
            for (i, d) in m.doors.iter().enumerate() {
                eprintln!("  door {i} sec {} type {} jump {:08x} to {} exit {}", d.sec, d.kind, d.jump, d.to, d.exit);
            }
            assert!(m.doors.iter().any(|d| d.to == super::STORY_NEXT), "{}: no way on", g.name);
            assert!(m.models.len() >= secs.len(), "{}: a section without its model", g.name);
            assert!(m.spawns.iter().all(|s| super::enemy_kind(s.4).1 != "unmapped"), "{}: unmapped kind", g.name);
            // every section is reached from the first by doors
            let mut seen = vec![0usize];
            let mut k = 0;
            while k < seen.len() {
                let s = seen[k];
                k += 1;
                for d in m.doors.iter().filter(|d| d.sec == s && d.to >= 0) {
                    let t = m.doors[d.to as usize].sec;
                    if !seen.contains(&t) {
                        seen.push(t);
                    }
                }
            }
            // every main section is reached (a side room can hang only off a door whose GMSJ jump wins: 080105a)
            for (k, s) in secs.iter().enumerate() {
                if !seen.contains(&k) {
                    eprintln!("  {} not reached by a door", s.file);
                    assert!(s.room != 0, "{}: main section {} no door reaches", g.name, s.file);
                }
            }
        }
    }

    #[test]
    fn lake_jungle_sections() {
        let names = |id| super::secs_of(id).iter().map(|s| s.file.trim_end_matches(".pac").to_string()).collect::<Vec<_>>();
        if super::files_present(super::GD_STAGE_SSE_LAKE3) {
            assert_eq!(names(super::GD_STAGE_SSE_LAKE3), ["080103", "080103a", "080103b", "080104", "080104a", "080104b", "080105", "080105a"]);
        }
        if super::files_present(super::GD_STAGE_SSE_JUNGLE) {
            assert_eq!(names(super::GD_STAGE_SSE_JUNGLE), ["060001", "060002", "060002a", "060003", "060004", "060004a"]);
        }
    }
}

#[cfg(test)]
mod jungle_tests {
    /// The Jungle's Bullet Bills on their own placement paths (GEG1 model index -> grGimmickMotionPath), through the
    /// enemy VM headless: every start status, 1200 frames from the GEG1 home; a path rider must keep to the path's
    /// points (plus the record's offset), and a replay from the frame-600 snapshot must give identical hashes.
    #[test]
    #[ignore]
    fn jungle_bullet_bill_paths() {
        use brawl_boss::vm;
        let id = super::GD_STAGE_SSE_JUNGLE;
        let Some(m) = super::load_into(super::maps(), id) else {
            eprintln!("no local files, skipped");
            return;
        };
        let Some(p) = crate::tu::geno_vm::program("bullet-bill") else {
            eprintln!("no bullet-bill program, skipped");
            return;
        };
        let riders: Vec<usize> = (0..m.spawns.len()).filter(|&k| m.spawns[k].4 == 13 && m.spawn_path[k] != usize::MAX).collect();
        for n in m.notes.iter().filter(|n| n.contains("placement")) { eprintln!("{n}"); }
        eprintln!("jungle: {} spawns, {} placement paths, {} Bullet Bills on a path", m.spawns.len(), m.enemy_paths.len(), riders.len());
        assert!(!riders.is_empty());
        let mut on_path = 0;
        for &k in riders.iter().take(3) {
            let (path, home) = super::enemy_vm_place(id, k).unwrap();
            let pa = path.unwrap();
            // no arena box (the program's default is a boss arena's; a Subspace path crosses its section's collision
            // extent, and the map host hands walls and floors through the input)
            let mut pp = p.clone();
            pp.bounds = [-1.0e6, 1.0e6, -1.0e6, 1.0e6];
            let p = &pp;
            let inp = |_f: u32| vm::Input { path: Some(pa), home: Some(home), ground: false, ..Default::default() };
            for st in 0..p.status_enter.len() as i32 {
                let mut a = vm::OFF;
                vm::begin(p, &mut a, 1, 5, 0x1234, st, true);
                a.pos = [home[0], home[1], 0.0];
                let (mut hashes, mut snap, mut near, mut moved, mut dev) = (vec![], None, 0u32, 0.0f32, None::<[f32; 2]>);
                for f in 0..1200u32 {
                    if f == 600 {
                        snap = Some(a);
                    }
                    vm::step(p, &mut a, &inp(f));
                    moved += a.moved;
                    // on the path: the actor keeps one fixed offset (the record's offset values plus the frame's step) from
                    // the path point at the rider unit's frame
                    if let Some(u) = a.kin.iter().find(|u| matches!(u.ek, 21 | 22) && u.kind >= 0) {
                        let q = (pa.pos)(pa.id, u.pf);
                        let d = [a.pos[0] - q[0], a.pos[1] - q[1]];
                        let d0 = *dev.get_or_insert(d);
                        if (d[0] - d0[0]).abs() + (d[1] - d0[1]).abs() < 1.0 {
                            near += 1;
                        }
                    }
                    hashes.push(a.vm_hash());
                }
                let q0 = (pa.pos)(pa.id, 0.0); if st == 0 { eprintln!("path pts {:?} ", [0.0f32, 60.0, 120.0, 180.0, 240.0].map(|g| { let q = (pa.pos)(pa.id, g); (q[0] as i32, q[1] as i32) })); }
                eprintln!("spawn {k} status {st}: path {} frames from ({:.0},{:.0}), home ({:.0},{:.0}), moved {moved:.0}, on path {near}/1200, pos ({:.0},{:.0})", pa.frames, q0[0], q0[1], home[0], home[1], a.pos[0], a.pos[1]);
                if moved > 400.0 && near > 1000 {
                    on_path += 1;
                }
                let mut b = snap.unwrap();
                for f in 600..1200u32 {
                    vm::step(p, &mut b, &inp(f));
                    assert_eq!(b.vm_hash(), hashes[f as usize], "spawn {k} status {st}: replay diverged at {f}");
                }
            }
        }
        assert!(on_path > 0, "no Bullet Bill rode its placement path");
    }
}
