//! `melee pose-dump --fighter NAME (--row N | --symbol S | --all) --out DIR [--iso ISO] [--raw] [--mods DIR [--enable a,b]]`: headless dump of a
//! fighter's animation clips as per-frame joint poses (format: docs/rust-port/notes/pose-dump.md, schema `pose-dump/1`).
//!
//! The reference evaluator is the booted engine's own path: a VS match on Final Destination is booted with idle pads,
//! and each clip is played through the Lab preview hook (`ScriptGame_LabSetPreviewRow` + `Fighter_ChangeMotionState`
//! to Wait, blend 0, rate 1, frame 0, facing +1). The preview plays the clip's figatree on the fighter's own skeleton
//! with the WAIT row's flag word and script, so no subaction script runs and no root-motion extraction happens unless
//! Wait's own flags ask for it (the per-frame `extract` field records it). Every frame is one `Engine::tick`; joints are
//! read after `ScriptGame_JointsSetup`, i.e. exactly what `ScriptGame_JointMtx` returns.
//!
//! Nothing here runs in unmodded play: the command boots its own engine in its own process. Disc data is only read,
//! the output directory is the caller's (never inside the repo).
#![allow(static_mut_refs, clippy::missing_safety_doc)]

use melee_types::*;
use std::vec::Vec;
use std::fmt::Write as _;
use std::io::Write as _;
use std::process::ExitCode;

pub const SCHEMA: &str = "pose-dump/1";
const MAGIC: &[u8; 4] = b"MPD1";
const VERSION: u32 = 1;
/// u32 track mask + raw SRT 9 + engine SRT 9 + world 12
const WORDS_PER_JOINT: u32 = 31;
const PART_NAMES: [&str; 53] = [
    "TopN", "TransN", "XRotN", "YRotN", "HipN", "WaistN", "LLegJA", "LLegJ", "LKneeJ", "LFootJA", "LFootJ", "RLegJA",
    "RLegJ", "RKneeJ", "RFootJA", "RFootJ", "BustN", "LShoulderN", "LShoulderJA", "LShoulderJ", "LArmJ", "LHandN",
    "L1stNa", "L1stNb", "L2ndNa", "L2ndNb", "L3rdNa", "L3rdNb", "L4thNa", "L4thNb", "LThumbNa", "LThumbNb", "LHandNb",
    "NeckN", "HeadN", "RShoulderN", "RShoulderJA", "RShoulderJ", "RArmJ", "RHandN", "R1stNa", "R1stNb", "R2ndNa",
    "R2ndNb", "R3rdNa", "R3rdNb", "R4thNa", "R4thNb", "RThumbNa", "RThumbNb", "RHandNb", "ThrowN", "TransN2",
];
/// Two-letter file codes (Pl<code>AJ.dat / Pl<code>Nr.dat), by the scenario's character names; only for `--raw`.
const CODES: [(&str, &str); 26] = [
    ("falcon", "Ca"), ("dk", "Dk"), ("fox", "Fx"), ("gw", "Gw"), ("kirby", "Kb"), ("bowser", "Kp"), ("link", "Lk"),
    ("luigi", "Lg"), ("mario", "Mr"), ("marth", "Ms"), ("mewtwo", "Mt"), ("ness", "Ns"), ("peach", "Pe"),
    ("pikachu", "Pk"), ("ics", "Pp"), ("jigglypuff", "Pr"), ("samus", "Ss"), ("yoshi", "Ys"), ("zelda", "Zd"),
    ("sheik", "Sk"), ("falco", "Fc"), ("younglink", "Cl"), ("drmario", "Dr"), ("roy", "Fe"), ("pichu", "Pc"),
    ("ganon", "Gn"),
];

fn jf(v: f32) -> String {
    if v.is_finite() { format!("{v:?}") } else { "null".into() }
}
fn jv(v: &[f32]) -> String {
    let s: Vec<String> = v.iter().map(|x| jf(*x)).collect();
    format!("[{}]", s.join(","))
}
fn js(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c if (c as u32) < 0x20 => { let _ = write!(o, "\\u{:04x}", c as u32); }
            c => o.push(c),
        }
    }
    o.push('"');
    o
}
unsafe fn cstr(p: *const i8) -> Option<String> {
    if p.is_null() { return None; }
    Some(std::ffi::CStr::from_ptr(p as *const core::ffi::c_char).to_string_lossy().into_owned())
}

/// One FObj key (fobj.c opcodes: 1 CON, 2 LIN, 3 SPL0, 4 SPL, 5 SLP, 6 KEY).
#[derive(Clone, Copy)]
pub struct Key { pub op: u8, pub value: Option<f32>, pub wait: Option<u32> }

/// Decodes an FObj key stream (the same rules as fobj.c's parser): header byte op|count, packed varints, numbers in
/// the track's value/slope format. `le` picks the byte order of multi-byte numbers (the game's is little endian; the
/// census decodes both ways to show it). Returns None on a malformed stream.
pub fn decode_keys(b: &[u8], fv: u8, fs: u8, le: bool) -> Option<Vec<Key>> {
    let mut p = 0usize;
    let byte = |p: &mut usize| -> Option<u8> { let v = *b.get(*p)?; *p += 1; Some(v) };
    let varint = |p: &mut usize| -> Option<u32> {
        let mut v = 0u32;
        for sh in (0..35).step_by(7) {
            let x = byte(p)?;
            v |= ((x & 127) as u32) << sh;
            if x & 128 == 0 { return Some(v); }
        }
        None
    };
    let number = |p: &mut usize, frac: u8| -> Option<f32> {
        let (kind, sh) = (frac >> 5, frac & 31);
        let n = [4usize, 2, 2, 1, 1].get(kind as usize).copied()?;
        let s = b.get(*p..*p + n)?;
        *p += n;
        let mut a = [0u8; 4];
        a[..n].copy_from_slice(s);
        if !le { a[..n].reverse(); }
        let raw = match kind {
            0 => return Some(f32::from_le_bytes(a)),
            1 => i16::from_le_bytes([a[0], a[1]]) as f32,
            2 => u16::from_le_bytes([a[0], a[1]]) as f32,
            3 => a[0] as i8 as f32,
            _ => a[0] as f32,
        };
        Some(raw / (1u32 << sh) as f32)
    };
    let mut out = Vec::new();
    while p < b.len() {
        let h = byte(&mut p)?;
        let op = h & 15;
        let mut count = (((h >> 4) & 7) + 1) as u32;
        if !(1..=6).contains(&op) { return None; }
        if h & 128 != 0 { count += varint(&mut p)? << 3; }
        for _ in 0..count {
            let value = if op != 5 { Some(number(&mut p, fv)?) } else { None };
            if op == 4 || op == 5 { number(&mut p, fs)?; }
            let wait = if op != 5 && p < b.len() { Some(varint(&mut p)?) } else { None };
            out.push(Key { op, value, wait });
        }
    }
    Some(out)
}

/// One track of a FigaTree as the engine holds it.
struct Track { joint: usize, ty: u8, fv: u8, fs: u8, start: u16, bytes: Vec<u8> }

unsafe fn tree_tracks(t: *const FigaTree, max_joints: usize) -> Vec<Track> {
    let mut out = Vec::new();
    if t.is_null() || (*t).nodes.is_null() || (*t).tracks.is_null() { return out; }
    let (mut n, mut ti) = (0usize, 0usize);
    loop {
        let c = *(*t).nodes.add(n) as u8;
        if c == 0xFF || n > 512 { break; }
        for _ in 0..c {
            let tr = &*(*t).tracks.add(ti);
            ti += 1;
            let bytes = if tr.ad_head.is_null() { Vec::new() } else { std::slice::from_raw_parts(tr.ad_head, tr.length as usize).to_vec() };
            out.push(Track { joint: n, ty: tr.obj_type, fv: tr.frac_value, fs: tr.frac_slope, start: tr.startframe, bytes });
        }
        n += 1;
    }
    let _ = max_joints;
    out
}

#[derive(Default)]
struct Census {
    clips: u32,
    track_types: std::collections::BTreeMap<u8, u32>,
    ops: std::collections::BTreeMap<u8, u32>,
    value_fmt: std::collections::BTreeMap<String, u32>,
    slope_fmt: std::collections::BTreeMap<String, u32>,
    bad_streams: u32,
    order_le: u32,
    order_be: u32,
    order_n: u32,
    example: Option<String>,
}
fn fmt_name(f: u8) -> String {
    let k = ["f32", "s16", "u16", "s8", "u8"].get((f >> 5) as usize).copied().unwrap_or("bad");
    format!("{k}/shift{}", f & 31)
}

struct Fig {
    fp: *mut Fighter,
}

unsafe fn local_of(j: *const HSD_JObj) -> [f32; 9] {
    let j = &*j;
    [j.scale.x, j.scale.y, j.scale.z, j.rotate.x, j.rotate.y, j.rotate.z, j.translate.x, j.translate.y, j.translate.z]
}

/// The rows to dump and the clip each one plays.
struct Row { row: i32, sym: Option<String>, off: i32, size: i32, flags: u32, has_script: bool }

pub fn pose_dump(args: &[String]) -> ExitCode {
    let arg = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let has = |k: &str| args.iter().any(|a| a == k);
    let (Some(fighter), Some(out)) = (arg("--fighter"), arg("--out")) else {
        eprintln!("usage: melee pose-dump --fighter NAME (--row N | --symbol S | --all) --out DIR [--iso ISO] [--raw] [--mods DIR [--enable a,b]]");
        return ExitCode::from(2);
    };
    let Some(iso) = arg("--iso").or_else(|| std::env::var("GW_ISO_VANILLA").ok()) else {
        eprintln!("pose-dump: no --iso / GW_ISO_VANILLA");
        return ExitCode::from(2);
    };
    let Some(ckind) = melee_engine::scenario::ckind_by_name(&fighter) else {
        eprintln!("pose-dump: unknown fighter {fighter:?}");
        return ExitCode::from(2);
    };
    let want_row = arg("--row").and_then(|s| s.parse::<i32>().ok());
    let want_sym = arg("--symbol");
    if want_row.is_none() && want_sym.is_none() && !has("--all") {
        eprintln!("pose-dump: choose --row N, --symbol S or --all");
        return ExitCode::from(2);
    }
    let other = if fighter == "fox" { "falco" } else { "fox" };
    let scene = format!("mode=vs;at=match;p1={fighter}/human;p2={other}/human;p3=none;p4=none;stage=fd;match=time;minutes=0;items=off");
    // --mods DIR [--enable a,b]: mount a mods folder (its anim_overrides too, notes/anim-overrides.md) over the disc
    if arg("--mods").is_some() {
        if let Err(er) = crate::set_engine_mods(args) {
            eprintln!("pose-dump: mods: {er}");
            return ExitCode::FAILURE;
        }
    }
    let disc = match crate::engine_disc(&iso) { Ok(d) => d, Err(e) => { eprintln!("pose-dump: {e}"); return ExitCode::FAILURE; } };
    let scn = melee_engine::scenario::Scenario::parse(&scene).expect("scene");
    let mut e = match melee_engine::Engine::boot_vs(disc, 64 << 20, melee_platform::PadSystem::headless(), scn) {
        Ok(e) => e,
        Err(er) => { eprintln!("pose-dump: engine boot: {er}"); return ExitCode::FAILURE; }
    };
    let dir = std::path::Path::new(&out).join(&fighter);
    if let Err(er) = std::fs::create_dir_all(dir.join("clips")) {
        eprintln!("pose-dump: {}: {er}", dir.display());
        return ExitCode::FAILURE;
    }
    unsafe { run(&mut e, &fighter, ckind, &dir, want_row, want_sym.as_deref(), has("--raw"), &iso) }
}

fn fighter_now() -> *mut Fighter {
    unsafe { melee_modes::tu::script_game::script_fighter(0) }
}

/// Into the match proper: P1 standing in Wait.
fn settle(e: &mut melee_engine::Engine) -> bool {
    for i in 0..900 {
        e.tick();
        let fp = fighter_now();
        if i > 30 && !fp.is_null() && unsafe { (*fp).motion_id } == ftCo_MS_Wait as _ && unsafe { (*fp).ground_or_air } == GA_Ground {
            return true;
        }
    }
    false
}

#[allow(clippy::too_many_arguments)]
unsafe fn run(e: &mut melee_engine::Engine, fighter: &str, ckind: i32, dir: &std::path::Path, want_row: Option<i32>, want_sym: Option<&str>, raw: bool, iso: &str) -> ExitCode {
    use melee_modes::tu::script_game as sg;
    if !settle(e) {
        eprintln!("pose-dump: P1 never reached Wait");
        return ExitCode::FAILURE;
    }
    let fp = fighter_now();
    let f = Fig { fp };
    let fpr = &mut *f.fp;
    let nrows = sg::lab_row_count(fpr);
    let nj = sg::lab_joint_count(fpr) as usize;
    let pt = *melee_fighter::hooks::ft_parts_table().offset(fpr.kind as isize);
    // joints: parent, flags, part, name
    let mut parent = vec![-1i32; nj];
    let mut jflags = vec![0u32; nj];
    let mut part = vec![-1i32; nj];
    let mut name = vec![String::new(); nj];
    for i in 0..nj {
        let j = (*fpr.parts.add(i)).joint;
        parent[i] = sg::ScriptGame_JointParent(0, i as i32);
        jflags[i] = if j.is_null() { 0 } else { (*j).flags };
        let p = *(*pt).joint_to_part.add(i) as i32;
        let unique = (p as usize) < PART_NAMES.len() && *(*pt).part_to_joint.add(p as usize) as usize == i;
        part[i] = p;
        name[i] = if unique { PART_NAMES[p as usize].to_string() } else { format!("JOBJ_{i}") };
    }
    // rest pose: the costume HSD_Joint tree, depth first (child before sibling), the order of fp.parts
    let mut rest: Vec<*mut HSD_Joint> = Vec::new();
    let mut stack = vec![fpr.x108_costume_joint];
    while let Some(j) = stack.pop() {
        if j.is_null() || rest.len() > 1024 { continue; }
        rest.push(j);
        stack.push((*j).next);
        if (*j).flags & 0x1000 == 0 { stack.push((*j).child); }
    }
    let rest_ok = rest.len() == nj;
    let rest_srt: Vec<[f32; 9]> = if rest_ok { rest.iter().map(|h| { let h = &**h; [h.scale.x, h.scale.y, h.scale.z, h.rotation.x, h.rotation.y, h.rotation.z, h.position.x, h.position.y, h.position.z] }).collect() } else { Vec::new() };
    let rows: Vec<Row> = (0..nrows).map(|r| {
        let w = &*fpr.x24.add(r as usize);
        Row { row: r, sym: cstr(w.x0 as *const i8), off: w.x4, size: w.x8, flags: w.x10_animCurrFlags as u32, has_script: !w.xC.is_null() }
    }).collect();
    let wait_flags = (*fpr.x24.add(fpr.x1C_actionStateList.add(ftCo_MS_Wait as usize).as_ref().unwrap().anim_id as usize)).x10_animCurrFlags as u32;
    let selected: Vec<i32> = rows.iter().filter(|r| match (want_row, want_sym) {
        (Some(n), _) => r.row == n,
        (_, Some(s)) => r.sym.as_deref() == Some(s),
        _ => true,
    }).map(|r| r.row).collect();
    if selected.is_empty() {
        eprintln!("pose-dump: no row matches");
        return ExitCode::from(2);
    }
    let mut first_of: std::collections::HashMap<(i32, i32), i32> = Default::default();
    let mut census = Census::default();
    let mut clips_json = Vec::new();
    let (mut dumped, mut skipped, mut aliased) = (0, 0, 0);
    let mut seen_alloc = 0;
    for &r in &selected {
        let row = &rows[r as usize];
        let mut j = String::new();
        let _ = write!(j, "{{\"row\":{},\"symbol\":{},\"aj_offset\":{},\"aj_size\":{},\"flags\":{},\"loop\":{},\"anim_vel\":{},\"has_script\":{}",
            row.row, row.sym.as_deref().map(js).unwrap_or("null".into()), row.off, row.size, row.flags, (row.flags >> 30) & 1, row.flags >> 31, row.has_script);
        let status: Result<String, String> = (|| {
            if row.size == 0 || row.sym.is_none() { return Err("no animation data (AJ size 0 or no symbol)".into()); }
            if let Some(&a) = first_of.get(&(row.off, row.size)) { aliased += 1; return Ok(format!(",\"alias_of\":{a}")); }
            first_of.insert((row.off, row.size), row.row);
            let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| pose_clip(e, row, nj, &name, &rest_srt, &mut census, dir)));
            match res {
                Ok(Ok(s)) => { dumped += 1; Ok(s) }
                Ok(Err(m)) => Err(m),
                Err(p) => Err(format!("panic: {}", p.downcast_ref::<String>().cloned().or(p.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default())),
            }
        })();
        match status {
            Ok(s) => j.push_str(&s),
            Err(m) => { skipped += 1; let _ = write!(j, ",\"skipped\":{}", js(&m)); }
        }
        j.push('}');
        clips_json.push(j);
        seen_alloc += 1;
        if seen_alloc % 25 == 0 { eprintln!("pose-dump: {fighter}: {seen_alloc}/{} rows", selected.len()); }
        // the preview hook off again, back to plain Wait
        sg::ScriptGame_LabSetPreviewRow(0, -1, 0);
        let fp = fighter_now();
        if fp.is_null() { eprintln!("pose-dump: P1 gone"); return ExitCode::FAILURE; }
    }
    // index.json
    let mut s = String::new();
    let _ = write!(s, "{{\"schema\":{},\"fighter\":{},\"char_kind\":{},\"fighter_kind\":{},\"evaluator\":\"engine-lab-preview\",\"facing\":1,\"wait_row_flags\":{},\"rows\":{},\"joint_count\":{nj},",
        js(SCHEMA), js(fighter), ckind, fpr.kind, wait_flags, nrows);
    let _ = write!(s, "\"joints\":[");
    for i in 0..nj {
        if i > 0 { s.push(','); }
        let bf = &(*fpr.parts.add(i)).x8.x0;
        // ftAnim_8006E7B8 skips HSD_JObjAnim on bones flagged b0 / b5 (cape and other dynamics bones): the clip's
        // tracks there are never applied by the game
        let applied = bf.flags_b0() == 0 && bf.flags_b5() == 0;
        let _ = write!(s, "\n {{\"index\":{i},\"name\":{},\"parent\":{},\"flags\":{},\"part\":{},\"anim_applied\":{applied}", js(&name[i]), parent[i], jflags[i], part[i]);
        if rest_ok {
            let h = &*rest[i];
            let _ = write!(s, ",\"rest\":{{\"flags\":{},\"scale\":{},\"rotation\":{},\"translation\":{}", h.flags,
                jv(&[h.scale.x, h.scale.y, h.scale.z]), jv(&[h.rotation.x, h.rotation.y, h.rotation.z]), jv(&[h.position.x, h.position.y, h.position.z]));
            if h.mtx.is_null() { s.push_str(",\"inverse_bind\":null}"); } else {
                let m = std::slice::from_raw_parts(h.mtx, 3);
                let v: Vec<f32> = m.iter().flat_map(|r| r.iter().copied()).collect();
                let _ = write!(s, ",\"inverse_bind\":{}}}", jv(&v));
            }
        }
        s.push('}');
    }
    let _ = write!(s, "],\"rest_pose_available\":{rest_ok},\n\"clips\":[\n{}\n],", clips_json.join(",\n"));
    let _ = write!(s, "\"summary\":{{\"rows_selected\":{},\"dumped\":{dumped},\"aliases\":{aliased},\"skipped\":{skipped}}}}}\n", selected.len());
    if let Err(er) = std::fs::write(dir.join("index.json"), s) { eprintln!("pose-dump: index.json: {er}"); return ExitCode::FAILURE; }
    // census.json
    let mut c = String::new();
    let m = |b: &std::collections::BTreeMap<u8, u32>| b.iter().map(|(k, v)| format!("\"{k}\":{v}")).collect::<Vec<_>>().join(",");
    let ms = |b: &std::collections::BTreeMap<String, u32>| b.iter().map(|(k, v)| format!("{}:{v}", js(k))).collect::<Vec<_>>().join(",");
    let quat = rest.iter().filter(|h| (***h).flags & JOBJ_USE_QUATERNION as u32 != 0).count();
    let _ = write!(c, "{{\"schema\":{},\"fighter\":{},\"clips\":{},\"joint_count\":{nj},\"track_types\":{{{}}},\"ops\":{{{}}},\"value_formats\":{{{}}},\"slope_formats\":{{{}}},\"bad_streams\":{},\"rest_quaternion_joints\":{quat},\"byte_order\":{{\"tracks_compared\":{},\"match_little_endian\":{},\"match_big_endian\":{},\"example\":{}}}}}\n",
        js(SCHEMA), js(fighter), census.clips, m(&census.track_types), m(&census.ops), ms(&census.value_fmt), ms(&census.slope_fmt), census.bad_streams,
        census.order_n, census.order_le, census.order_be, census.example.as_deref().map(js).unwrap_or("null".into()));
    let _ = std::fs::write(dir.join("census.json"), &c);
    if raw {
        if let Some(code) = CODES.iter().find(|(n, _)| *n == fighter).map(|(_, c)| *c) {
            match melee_engine::gcm::Gcm::open(iso) {
                Ok(mut g) => {
                    let _ = std::fs::create_dir_all(dir.join("raw"));
                    for f in [format!("Pl{code}AJ.dat"), format!("Pl{code}Nr.dat")] {
                        match g.read(&f) { Ok(b) => { let _ = std::fs::write(dir.join("raw").join(&f), b); } Err(er) => eprintln!("pose-dump: --raw {f}: {er}") }
                    }
                }
                Err(_) => eprintln!("pose-dump: --raw: could not open the disc"),
            }
        }
    }
    eprintln!("pose-dump: {fighter}: {} rows selected, {dumped} clips dumped, {aliased} aliases, {skipped} skipped -> {}", selected.len(), dir.display());
    ExitCode::SUCCESS
}

/// Plays one clip through the preview hook and writes its .pose file. Returns the clip's index.json fields.
unsafe fn pose_clip(e: &mut melee_engine::Engine, row: &Row, nj: usize, name: &[String], rest: &[[f32; 9]], census: &mut Census, dir: &std::path::Path) -> Result<String, String> {
    use melee_modes::tu::script_game as sg;
    let fp = fighter_now();
    if fp.is_null() { return Err("no fighter".into()); }
    {
        let _scope = melee_engine::arena::GameScope::enter();
        sg::ScriptGame_LabSetPreviewRow(0, row.row, 1);
        (*fp).facing_dir = 1.0;
        melee_fighter::tu::fighter::Fighter_ChangeMotionState((*fp).gobj as _, ftCo_MS_Wait as i32, 0, 0.0, 1.0, -1.0, core::ptr::null_mut()); // blend -1: none (0 would take the row's default blend)
    }
    let fp = fighter_now();
    if (*fp).anim_id != row.row || (*fp).x590.is_null() {
        return Err(format!("clip did not bind (anim_id {}, tree null {})", (*fp).anim_id, (*fp).x590.is_null()));
    }
    let tree = (*fp).x590;
    let frames = (*tree).frames;
    let n = if frames.is_finite() && frames > 0.0 { frames.ceil() as usize } else { 1 };
    let tracks = tree_tracks(tree, nj);
    // per-joint track lists for the index, and the census
    let mut per_joint: Vec<Vec<String>> = vec![Vec::new(); nj.max(1)];
    let mut extra_joints = 0usize;
    census.clips += 1;
    for t in &tracks {
        *census.track_types.entry(t.ty).or_default() += 1;
        *census.value_fmt.entry(fmt_name(t.fv)).or_default() += 1;
        *census.slope_fmt.entry(fmt_name(t.fs)).or_default() += 1;
        match decode_keys(&t.bytes, t.fv, t.fs, true) {
            Some(k) => for key in &k { *census.ops.entry(key.op).or_default() += 1; },
            None => census.bad_streams += 1,
        }
        if t.joint < nj {
            per_joint[t.joint].push(format!("{{\"type\":{},\"value_format\":{},\"slope_format\":{},\"start\":{},\"bytes\":{}}}", t.ty, t.fv, t.fs, t.start as i16, t.bytes.len()));
        } else {
            extra_joints = extra_joints.max(t.joint + 1);
        }
    }
    let mut buf: Vec<u8> = Vec::with_capacity(32 + n * (16 + nj * 84));
    buf.extend_from_slice(MAGIC);
    for v in [VERSION, nj as u32, n as u32, WORDS_PER_JOINT] { buf.extend_from_slice(&v.to_le_bytes()); }
    let mut got = 0usize;
    let mut first_locals: Vec<[f32; 9]> = Vec::new();
    for k in 0..n {
        if k > 0 { e.tick(); }
        let fp = fighter_now();
        if fp.is_null() || (*fp).anim_id != row.row || (*fp).motion_id != ftCo_MS_Wait as _ { break; }
        sg::ScriptGame_JointsSetup(0);
        let extract = (*fp).x594.x0.x594_b0() as u32;
        buf.extend_from_slice(&(k as u32).to_le_bytes());
        buf.extend_from_slice(&(*fp).cur_anim_frame.to_le_bytes());
        buf.extend_from_slice(&extract.to_le_bytes());
        for v in [(*fp).x68C_transNPos, (*fp).x6A4_transNOffset] { for c in [v.x, v.y, v.z] { buf.extend_from_slice(&c.to_le_bytes()); } }
        for i in 0..nj {
            let j = (*(*fp).parts.add(i)).joint;
            let l = if j.is_null() { [0.0; 9] } else { local_of(j) };
            // raw: the clip's own track value per channel, else the rest pose (else the engine's value)
            let tv = if j.is_null() { [None; 9] } else { melee_runtime::pose_capture::track_values(j) };
            let mut raw = rest.get(i).copied().unwrap_or(l);
            let mut mask = 0u32;
            for (c, slot) in [3usize, 4, 5, 6, 7, 8, 0, 1, 2].into_iter().enumerate() {
                if let Some(v) = tv[c] { raw[slot] = v; mask |= 1 << slot; }
            }
            if k == 0 { first_locals.push(raw); }
            buf.extend_from_slice(&mask.to_le_bytes());
            for c in raw { buf.extend_from_slice(&c.to_le_bytes()); }
            for c in l { buf.extend_from_slice(&c.to_le_bytes()); }
            for c in 0..12 { buf.extend_from_slice(&sg::ScriptGame_JointMtx(0, i as i32, c).to_le_bytes()); }
        }
        got += 1;
    }
    if got == 0 { return Err("left the preview before frame 0 was read".into()); }
    buf[12..16].copy_from_slice(&(got as u32).to_le_bytes());
    // byte order evidence: at track time 0 (start 0) a track evaluates to its first value key, which frame 0 shows
    for t in &tracks {
        let ch = match t.ty { 1..=3 => 3 + (t.ty - 1) as usize, 5..=7 => 6 + (t.ty - 5) as usize, 8..=10 => (t.ty - 8) as usize, _ => continue };
        if t.joint == 0 || t.joint >= first_locals.len() { continue; }
        let (Some(le), Some(be)) = (decode_keys(&t.bytes, t.fv, t.fs, true), decode_keys(&t.bytes, t.fv, t.fs, false)) else { continue };
        let vals = |k: &Vec<Key>| k.iter().filter(|x| x.op != 5).map(|x| x.value.unwrap()).collect::<Vec<_>>();
        let (vl, vb) = (vals(&le), vals(&be));
        if vl.is_empty() || vb.is_empty() || t.start != 0 || le[0].op == 5 { continue; }
        let width = [4usize, 2, 2, 1, 1].get((t.fv >> 5) as usize).copied().unwrap_or(1);
        if width == 1 { continue; }
        let eng = first_locals[t.joint][ch];
        census.order_n += 1;
        if (vl[0] - eng).abs() <= 1e-5 * eng.abs().max(1.0) { census.order_le += 1; }
        if (vb[0] - eng).abs() <= 1e-5 * eng.abs().max(1.0) { census.order_be += 1; }
        if census.example.is_none() && (vl[0] - eng).abs() <= 1e-5 * eng.abs().max(1.0) && (vl[0] - vb.first().copied().unwrap_or(f32::NAN)).abs() > 1e-3 {
            let hex: Vec<String> = t.bytes.iter().map(|b| format!("{b:02x}")).collect();
            census.example = Some(format!("{} joint {} ({}) type {} format {}: stream [{}] decodes little endian to {} (engine frame 0: {}), big endian to {}",
                row.sym.clone().unwrap_or_default(), t.joint, name[t.joint], t.ty, fmt_name(t.fv), hex.join(" "), vl[0], eng, vb.first().copied().unwrap_or(f32::NAN)));
        }
    }
    let file = format!("{:03}_{}.pose", row.row, row.sym.as_deref().unwrap_or("x").replace(|c: char| !c.is_ascii_alphanumeric() && c != '_', "_"));
    let mut fh = std::fs::File::create(dir.join("clips").join(&file)).map_err(|e| e.to_string())?;
    fh.write_all(&buf).map_err(|e| e.to_string())?;
    let tj: Vec<String> = per_joint.iter().map(|v| format!("[{}]", v.join(","))).collect();
    Ok(format!(",\"file\":{},\"tree_type\":{},\"tree_flags\":{},\"frames\":{},\"frames_dumped\":{got},\"anim_joints_beyond_skeleton\":{extra_joints},\"tracks\":[{}]",
        js(&format!("clips/{file}")), (*tree).r#type, (*tree).flags, jf(frames), tj.join(",")))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A synthetic stream (no disc data): LIN x2 (header 0x12: op 2, count 2) with u16 values at shift 12,
    /// value 0x3000 = 3.0, wait 5, value 0x1000 = 1.0, last wait 0.
    #[test]
    fn decode_keys_little_endian_u16() {
        let s = [0x12u8, 0x00, 0x30, 0x05, 0x00, 0x10, 0x00];
        let fv = (2 << 5) | 12;
        let le = decode_keys(&s, fv, 0, true).unwrap();
        assert_eq!(le.len(), 2);
        assert_eq!((le[0].op, le[0].value, le[0].wait), (2, Some(3.0), Some(5)));
        assert_eq!((le[1].value, le[1].wait), (Some(1.0), Some(0)));
        let be = decode_keys(&s, fv, 0, false).unwrap();
        assert_eq!(be[0].value, Some(0x0030 as f32 / 4096.0));
    }

    /// SPL keys carry a slope, SLP a slope only, s16 and s8 formats sign-extend, f32 is raw, varint counts extend.
    #[test]
    fn decode_keys_ops_and_formats() {
        // SPL count 1: s16 value -2.0 at shift 8 (0xFE00), s8 slope -1 at shift 0, wait 1; then SLP: slope 2
        let s = [0x04u8, 0x00, 0xFE, 0xFF, 0x01, 0x05, 0x02];
        let k = decode_keys(&s, (1 << 5) | 8, 3 << 5, true).unwrap();
        assert_eq!((k[0].op, k[0].value, k[0].wait), (4, Some(-2.0), Some(1)));
        assert_eq!((k[1].op, k[1].value), (5, None));
        // CON with f32 1.5, count extended by the varint (0x80 | count-1 = 0) + 1 << 3 = 9 keys
        let mut s = vec![0x81u8, 0x01];
        for _ in 0..9 { s.extend_from_slice(&1.5f32.to_le_bytes()); s.push(1); }
        let k = decode_keys(&s, 0, 0, true).unwrap();
        assert_eq!(k.len(), 9);
        assert!(k.iter().all(|x| x.op == 1 && x.value == Some(1.5)));
        // a bad opcode is refused
        assert!(decode_keys(&[0x07, 0], 0, 0, true).is_none());
    }
}
