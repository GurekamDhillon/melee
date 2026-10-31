//! `melee diff`: field-level C-vs-Rust state diff (usage: docs/rust-port/notes/diff.md).
//!
//! The C build's state at golden frame N (`MELEE_SNAP_DUMP`, tools/rust-port/snap/) is read through the
//! GameCube offset tables of `melee_types` into the same native `Fighter` the sim uses; the Rust sim runs
//! from frame 0 to N; the two are compared field by field with the generated tables of
//! `diff_layout.rs` (tools/rust-port/diff/gen_layout.py) and every difference is printed by name.
//! What the native Fighter does not hold (effects, particles, stage animation, the sim's private
//! per-player state) is compared as the canonical `state_dump` text of a sim loaded from the snapshot
//! against the from-0 sim (the "model state" section), when the snapshot loader can take that snapshot.

use crate::snap::Snap;
use crate::vs_match::VsMatch;
use crate::Sim;
use melee_types::Fighter;
use std::fmt::Write as _;
use std::process::ExitCode;

#[derive(Clone, Copy)]
pub enum Kind {
    /// A pointer: native and guest values are not comparable.
    Ptr,
    F32,
    /// Integer of the field's size; the flag is signedness.
    Int(bool),
    /// Plain bytes (byte arrays, bitfield bytes, opaque types).
    Blob,
    /// A C union: native code writes arms the GC decode cannot see; only shown with `--unions`.
    Union,
    Struct(&'static str, &'static [F]),
    /// (count, GC stride, native stride, element)
    Arr(usize, u32, usize, &'static Kind),
}

#[derive(Clone, Copy)]
pub struct F {
    pub name: &'static str,
    pub gc: u32,
    pub nat: usize,
    pub size: usize,
    pub kind: Kind,
}

/// One named difference.
struct Diff {
    /// Importance rank: 0 fighters, 1 items, 2 stage, 3 rng, 4 anything else.
    rank: u8,
    order: (u32, u32),
    strukt: &'static str,
    instance: String,
    field: String,
    gc: Option<u32>,
    c: String,
    rust: String,
    /// A field the golden hash reads: never hidden by the baseline.
    hashed: bool,
}

struct Opts {
    unions: bool,
    /// `--peek NAME`: also list fields whose name contains NAME when C and Rust agree.
    peek: Option<String>,
}

impl Opts {
    fn peeks(&self, name: &str) -> bool {
        self.peek.as_deref().is_some_and(|p| name.contains(p))
    }
}

fn rd(b: &[u8], size: usize) -> u64 {
    let mut v = 0u64;
    for i in (0..size.min(8)).rev() {
        v = (v << 8) | b[i] as u64;
    }
    v
}

fn fnum(b: &[u8]) -> String {
    let bits = u32::from_le_bytes(b[..4].try_into().unwrap());
    format!("{} ({bits:08x})", f32::from_bits(bits))
}

fn int_str(b: &[u8], size: usize, signed: bool) -> String {
    let u = rd(b, size);
    let sh = 64 - 8 * size.min(8) as u32;
    if signed {
        let s = ((u << sh) as i64) >> sh;
        format!("{s} ({u:#x})")
    } else {
        format!("{u} ({u:#x})")
    }
}

fn hexs(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect::<Vec<_>>().join("")
}

#[allow(clippy::too_many_arguments)]
fn walk(c: &[u8], r: &[u8], name: &str, gc: u32, nat: usize, size: usize, kind: &Kind, parent: &str, root_bits: &[(&str, u32, u32, u32)], o: &Opts, sink: &mut dyn FnMut(String, u32, String, String)) {
    match kind {
        Kind::Ptr => {}
        Kind::Union if !o.unions => {}
        Kind::F32 => {
            let (a, b) = (&c[nat..nat + 4], &r[nat..nat + 4]);
            if a != b || o.peeks(name) {
                sink(name.into(), gc, fnum(a), fnum(b));
            }
        }
        Kind::Int(sg) => {
            let (a, b) = (&c[nat..nat + size], &r[nat..nat + size]);
            if a != b || o.peeks(name) {
                sink(name.into(), gc, int_str(a, size, *sg), int_str(b, size, *sg));
            }
        }
        Kind::Blob | Kind::Union => {
            let (a, b) = (&c[nat..nat + size], &r[nat..nat + size]);
            if a == b && !o.peeks(name) {
                return;
            }
            if name.rsplit('.').next().is_some_and(|n| n.starts_with("bf_")) {
                // Bitfield bytes stay in GC order: name the bits that changed.
                let mut names = Vec::new();
                for (n, byte, bit, w) in root_bits {
                    for k in 0..*w {
                        let p = byte * 8 + bit + k;
                        let (bi, sh) = ((p / 8) as i64 - gc as i64, 7 - p % 8);
                        if bi >= 0 && (bi as usize) < size && (a[bi as usize] ^ b[bi as usize]) >> sh & 1 != 0 {
                            names.push(*n);
                            break;
                        }
                    }
                }
                let what = if names.is_empty() { String::new() } else { format!(" [bits {}]", names.join(",")) };
                sink(format!("{name}{what}"), gc, hexs(a), hexs(b));
                return;
            }
            // Byte ranges, 4 bytes at a time.
            let mut i = 0;
            while i < size {
                let e = (i + 4).min(size);
                if a[i..e] != b[i..e] {
                    sink(format!("{name}+{i:#x}"), gc + i as u32, hexs(&a[i..e]), hexs(&b[i..e]));
                }
                i = e;
            }
        }
        Kind::Struct(ty, tab) => {
            let bits = crate::diff_layout::bitfields(ty);
            for f in tab.iter() {
                let nm = if name.is_empty() { f.name.to_string() } else { format!("{name}.{}", f.name) };
                walk(c, r, &nm, gc + f.gc, nat + f.nat, f.size, &f.kind, ty, bits, o, sink);
            }
        }
        Kind::Arr(n, gs, ns, el) => {
            for i in 0..*n {
                walk(c, r, &format!("{name}[{i}]"), gc + i as u32 * gs, nat + i * ns, *ns, el, parent, root_bits, o, sink);
            }
        }
    }
}

/// Everything compared, copied out of a finished sim so a second sim can be built afterwards.
struct State {
    frame: i32,
    seed: u32,
    render_seed: u32,
    fighters: Vec<(i32, Vec<u8>)>,
    items: Vec<(i32, i32, [f32; 3], [f32; 3])>,
    respawn_busy: [i32; 6],
    stocks: Vec<(i32, i32)>,
    dump: String,
    /// Captured from the consolidated engine (`--engine tu`): fighters and the seed only so far.
    engine: bool,
    /// Engine only: per slot, the fighter's JObj tree in preorder as `[flags, rot x4, scale x3, translate x3, mtx 3x4]` bit patterns.
    joints: Vec<(i32, Vec<[u32; 26]>)>,
    /// Engine only: per slot, the dynamic bone chains (`Engine::dyn_words`).
    dyns: Vec<(i32, Vec<Vec<Vec<u32>>>)>,
}

fn capture(sim: &VsMatch, frame: i32) -> State {
    let (items, respawn_busy) = sim.diff_items();
    State {
        frame,
        seed: sim.rand.logic_seed(),
        render_seed: sim.rand.render_seed,
        fighters: sim.players.iter().map(|p| (p.slot, raw(&*p.fp).to_vec())).collect(),
        items,
        respawn_busy,
        stocks: sim.players.iter().map(|p| (p.slot, melee_fighter::stale::stocks(p.slot))).collect(),
        dump: sim.state_dump(),
        engine: false,
        joints: vec![],
        dyns: vec![],
    }
}

/// The engine run to golden frame `frame`: the native Fighter records and the seed (items / stocks / model state wait for step 17).
fn engine_state(iso: &str, scn: &str, frame: i32) -> Result<State, String> {
    if std::env::var_os("MELEE_ENGINE_PRE_DRAWS").is_none() {
        let pre = std::fs::read_to_string("tools/rust-port/engine/pre_draws.tsv").ok().and_then(|t| t.lines().find_map(|l| l.strip_prefix(&format!("{scn}	")).map(str::to_string)));
        // SAFETY: single-threaded start-up, before the engine reads it.
        unsafe { std::env::set_var("MELEE_ENGINE_PRE_DRAWS", pre.unwrap_or_else(|| "0".into())) };
    }
    let mut e = crate::boot_engine_scenario(iso, scn)?;
    for _ in 0..=frame {
        if !e.tick() {
            return Err(format!("the engine's scene ended before frame {frame}"));
        }
    }
    let r = e.neutral_row();
    let mut fighters = Vec::new();
    for slot in 0..2 {
        if let Some(b) = e.fighter_bytes(slot) {
            fighters.push((slot, b));
        }
    }
    let joints = (0..2).map(|slot| (slot, e.joint_words(slot))).collect();
    let dyns = (0..2).map(|slot| (slot, e.dyn_words(slot))).collect();
    Ok(State { frame, seed: r[1], render_seed: 0, fighters, items: e.item_rows(), respawn_busy: [0; 6], stocks: vec![], dump: String::new(), engine: true, joints, dyns })
}

fn raw<T>(v: &T) -> &[u8] {
    // SAFETY: bytes of a live plain-data value.
    unsafe { std::slice::from_raw_parts(v as *const T as *const u8, std::mem::size_of::<T>()) }
}

fn scenario(name: &str) -> Option<(&'static str, &'static [&'static str], Option<i32>)> {
    Some(match name {
        "pad_falcon_fox_fd" => ("GrNLa.dat", &["falcon", "fox"], None),
        "pad_marth_sheik_bf" => ("GrNBa.dat", &["marth", "sheik"], None),
        "cpu_fox_falco_fd" => ("GrNLa.dat", &["fox/cpu9", "falco/cpu9"], None),
        "cpu_fox_falco_dl" => ("GrGr.dat", &["fox/cpu9", "falco/cpu9"], None),
        "cpu_peach_samus_ys_items" => ("GrSt.dat", &["peach/cpu9", "samus/cpu9"], Some(4)),
        _ => return None,
    })
}

fn build(iso: &str, scn: &str) -> Result<VsMatch, String> {
    let (stage, fighters, item_freq) = scenario(scn).ok_or_else(|| format!("scenario {scn:?} not wired"))?;
    if scn.starts_with("cpu_") {
        crate::vs_match::set_cpu_player(0, 9);
        crate::vs_match::set_cpu_player(1, 9);
    }
    let mut d = crate::disc::Disc::open(iso)?;
    let mut sim = VsMatch::new_ex(&mut d, stage, fighters, 12345, 3, item_freq)?;
    if scn.starts_with("cpu_") {
        sim.pad_feed = Some([Default::default(); 4]);
    }
    Ok(sim)
}

/// The golden CSV row of a sim frame, as text columns (the order of the file).
fn row_cols(w: &crate::FrameWords) -> Vec<String> {
    let h = melee_types::neutral_hash::nh_game(w.seed, &w.slots, w.items, w.itemsum);
    let mut v = vec![w.frame.to_string(), format!("{h:08X}"), format!("{:08X}", w.seed), w.items.to_string(), format!("{:08X}", w.itemsum)];
    for s in w.slots {
        v.push(format!("{:08X}", s.unwrap_or(0)));
    }
    v
}

fn find_snap(scn: &str, frame: i32, explicit: Option<String>) -> Result<String, String> {
    if let Some(p) = explicit {
        return Ok(p);
    }
    let file = format!("{scn}.f{frame}.snap");
    let ws = std::env::var("GW_WORKSPACE").unwrap_or_else(|_| "E:/Projects/Melee Workspace".into());
    let mut dirs = vec![];
    if let Ok(r) = std::env::var("GW_BUILD_ROOT") {
        dirs.push(std::path::PathBuf::from(r).join("snaps"));
    }
    let agents = std::path::Path::new(&ws).join("_build/agents");
    if let Ok(rd) = std::fs::read_dir(&agents) {
        for e in rd.flatten() {
            dirs.push(e.path().join("snaps"));
        }
    }
    for d in &dirs {
        let p = d.join(&file);
        if p.is_file() {
            return Ok(p.to_string_lossy().into_owned());
        }
    }
    // Ask the C build for it (needs GW_MELEE and a built GW_BUILD_ROOT).
    if let (Ok(_), Ok(root)) = (std::env::var("GW_MELEE"), std::env::var("GW_BUILD_ROOT")) {
        eprintln!("diff: no snapshot of {scn} frame {frame}; dumping it from the C build (tools/rust-port/snap/dump.sh)");
        let melee = std::env::var("GW_MELEE").unwrap();
        let st = std::process::Command::new("bash").arg(format!("{melee}/tools/rust-port/snap/dump.sh")).arg(scn).arg(frame.to_string()).status();
        let p = std::path::Path::new(&root).join("snaps").join(&file);
        if st.is_ok() && p.is_file() {
            return Ok(p.to_string_lossy().into_owned());
        }
    }
    Err(format!("no C snapshot {file} found; make one: GW_MELEE=<worktree> GW_BUILD_ROOT=<build root> bash tools/rust-port/snap/dump.sh {scn} {frame} (or pass --snap FILE)"))
}

pub fn diff(args: &[String]) -> ExitCode {
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let has = |k: &str| args.iter().any(|a| a == k);
    let (Some(scn), Some(iso)) = (get("--scenario"), get("--iso").or_else(|| std::env::var("GW_ISO_VANILLA").ok())) else {
        eprintln!("usage: melee diff --scenario NAME (--frame N | --frames A,B,C | --first) [--subsystems fighter,item,..] [--baseline-check DIR] [--engine legacy] [--iso DISC] [--snap FILE] [--golden CSV] [--max N] [--unions]\n  scenarios: pad_falcon_fox_fd pad_marth_sheik_bf cpu_fox_falco_dl cpu_fox_falco_fd cpu_peach_samus_ys_items");
        return ExitCode::from(2);
    };
    crate::crash::note_disc(&iso);
    let engine = get("--engine").as_deref() == Some("tu");
    if let Some(e) = get("--engine") {
        if e != "legacy" && e != "tu" {
            eprintln!("melee diff: --engine {e}: legacy | tu");
            return ExitCode::from(2);
        }
    }
    if engine && has("--first") {
        eprintln!("melee diff: --engine tu takes --frame N / --frames LIST (use melee golden --engine tu for the first divergence)");
        return ExitCode::from(2);
    }
    if let Some(list) = get("--frames") {
        return diff_frames(args, &scn, &list);
    }
    let max: usize = get("--max").and_then(|v| v.parse().ok()).unwrap_or(40);
    let target: Option<i32> = get("--frame").and_then(|v| v.parse().ok());
    let first = has("--first");
    if target.is_none() && !first {
        eprintln!("melee diff: give --frame N or --first");
        return ExitCode::from(2);
    }
    let golden_path = get("--golden").unwrap_or_else(|| format!("docs/rust-port/golden/{scn}.csv"));
    let golden: Vec<Vec<String>> = if first {
        match std::fs::read_to_string(&golden_path) {
            Ok(t) => t.lines().skip(1).map(|l| l.split(',').map(str::to_string).collect()).collect(),
            Err(e) => {
                eprintln!("melee diff: {golden_path}: {e} (run from the repo root or pass --golden)");
                return ExitCode::from(2);
            }
        }
    } else {
        vec![]
    };
    let mut state: Option<State> = None;
    let mut why = String::new();
    let mut n_rows = 0;
    if engine {
        match engine_state(&iso, &scn, target.unwrap_or(0)) {
            Ok(s) => state = Some(s),
            Err(e) => {
                eprintln!("melee diff: {e}");
                return ExitCode::from(3);
            }
        }
    }
    let mut sim = if engine {
        None
    } else {
        match build(&iso, &scn) {
            Ok(s) => Some(s),
            Err(e) => {
                eprintln!("melee diff: {e}");
                return ExitCode::FAILURE;
            }
        }
    };
    // Run the sim from frame 0.
    while let Some(sim) = sim.as_mut() {
        if !sim.step() {
            why = sim.stopped.clone().unwrap_or_else(|| "sim ended".into());
            break;
        }
        let w = sim.frame_words();
        let f = w.frame as i32;
        n_rows += 1;
        if first {
            let cols = row_cols(&w);
            match golden.iter().find(|g| g.first().map(String::as_str) == Some(&cols[0])) {
                Some(g) if g.iter().zip(cols.iter()).any(|(a, b)| a != b) => {
                    let bad: Vec<String> = ["frame", "hash", "seed", "items", "itemsum", "s0", "s1", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11"].iter().zip(g.iter().zip(cols.iter()))
                        .filter(|(_, (a, b))| a != b).map(|(n, (a, b))| format!("{n} golden {a} rust {b}")).collect();
                    println!("first golden-diverging frame: {f} ({})", bad.join("; "));
                    state = Some(capture(&sim, f));
                    break;
                }
                Some(_) => {}
                None => {
                    why = "end of the golden".into();
                    break;
                }
            }
        } else if Some(f) == target {
            state = Some(capture(&sim, f));
            break;
        }
    }
    let Some(st) = state else {
        if first {
            println!("no golden divergence in {n_rows} frames of {scn} ({why})");
            if why != "end of the golden" && n_rows > 0 {
                // The sim cannot go on (unported code): diff the last frame it finished.
                println!("diffing the last simulated frame, {}", n_rows - 1);
                let mut a: Vec<String> = vec!["diff".into(), "--scenario".into(), scn.clone(), "--frame".into(), (n_rows - 1).to_string()];
                let mut it = args.iter().skip(1);
                while let Some(x) = it.next() {
                    match x.as_str() {
                        "--first" | "--scenario" | "--frame" => {
                            if x != "--first" {
                                it.next();
                            }
                        }
                        _ => a.push(x.clone()),
                    }
                }
                return match std::env::current_exe().and_then(|e| std::process::Command::new(e).args(&a).status()) {
                    Ok(s) => ExitCode::from(s.code().unwrap_or(1) as u8),
                    Err(e) => {
                        eprintln!("melee diff: {e}");
                        ExitCode::FAILURE
                    }
                };
            }
            return ExitCode::SUCCESS;
        }
        eprintln!("melee diff: the sim stopped before frame {}: {why}", target.unwrap());
        return ExitCode::from(3); // the --frames wrapper skips frames beyond the engine's count
    };
    let frame = st.frame;
    let snap_path = match find_snap(&scn, frame, get("--snap")) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("melee diff: {e}");
            return ExitCode::from(4);
        }
    };
    let snap = match Snap::load(&snap_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("melee diff: {e}");
            return ExitCode::FAILURE;
        }
    };
    if snap.frame != frame {
        eprintln!("melee diff: {snap_path} is a snapshot of frame {}, not {frame}", snap.frame);
        return ExitCode::FAILURE;
    }
    let opts = Opts { unions: has("--unions"), peek: get("--peek") };
    let mut out: Vec<Diff> = Vec::new();
    if let Err(e) = compare(&snap, &st, &opts, &mut out) {
        eprintln!("melee diff: {e}");
        return ExitCode::FAILURE;
    }
    // Model state: the sim loaded from the snapshot against the from-0 sim.
    let mut model_note = String::new();
    if !st.engine {
        let (stage, fighters, _) = scenario(&scn).unwrap();
        match crate::disc::Disc::open(&iso).and_then(|mut d| VsMatch::from_snapshot(&mut d, &snap, stage, fighters, 12345, 3)) {
            Ok(m) => model_diff(&st.dump, &m.state_dump(), &mut out),
            Err(e) => model_note = format!("model-state section skipped: the snapshot loader refused this state ({e})"),
        }
    }
    for d in out.iter_mut() {
        d.hashed = match d.strukt {
            "Item" | "HSD_Rand" | "StaticPlayer" => true,
            "Fighter" => HASHED.iter().any(|h| d.field == *h || d.field.starts_with(&format!("{h}.")) || d.field.starts_with(&format!("{h} ["))) || d.field.contains("x2219_b5") || (scn.starts_with("cpu_") && HASHED_CPU.iter().any(|h| d.field == *h || d.field.starts_with(&format!("{h}.")) || d.field.starts_with("cpu.bf_f8"))),
            _ => false,
        };
    }
    let check_dir = get("--baseline-check");
    let subs: Option<Vec<String>> = get("--subsystems").map(|v| v.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect());
    let by_sub = load_baselines(check_dir.as_deref());
    let baseline: std::collections::BTreeSet<String> = by_sub.values().flatten().cloned().collect();
    if let Some(path) = get("--baseline-out") {
        // A directory (or a path ending in / ) gets one file per subsystem; a file gets the union (the old layout).
        let mut set = baseline.clone();
        if let Ok(old) = std::fs::read_to_string(&path) {
            set.extend(baseline_lines(&old));
        }
        set.extend(out.iter().filter(|d| !d.hashed).map(norm));
        let head = "# Known-unmodelled baseline for `melee diff` (docs/rust-port/notes/diff.md): fields/lines the native sim does not keep
# like the C build at golden-matching frames. Built with --baseline-out; a hashed field is never listed.
";
        let res = if std::path::Path::new(&path).is_dir() || path.ends_with('/') {
            std::fs::create_dir_all(&path).and_then(|_| {
                for (name, _) in SUBSYSTEMS {
                    let body: Vec<&String> = set.iter().filter(|k| subsystem_of_key(k) == *name).collect();
                    std::fs::write(std::path::Path::new(&path).join(format!("{name}.txt")), head.to_string() + &body.iter().map(|k| k.as_str()).collect::<Vec<_>>().join("
") + if body.is_empty() { "" } else { "
" })?;
                }
                Ok(())
            })
        } else {
            std::fs::write(&path, head.to_string() + &set.into_iter().collect::<Vec<_>>().join("
") + "
")
        };
        if let Err(e) = res {
            eprintln!("melee diff: {path}: {e}");
        }
    }
    // `--subsystems a,b`: only those parts of the state.
    if let Some(list) = &subs {
        out.retain(|d| list.iter().any(|s| s == subsystem_of_key(&norm(d))));
    }
    let total_all = out.len();
    if has("--print-hits") {
        // The baseline lines this frame hit (the `--frames` wrapper and gate.sh decide which lines are STALE).
        let hit: std::collections::BTreeSet<String> = out.iter().filter(|d| !d.hashed).map(norm).filter(|k| baseline.contains(k)).collect();
        for k in hit {
            println!("HIT {k}");
        }
    }
    if !has("--all") || check_dir.is_some() {
        out.retain(|d| d.hashed || !baseline.contains(&norm(d)));
    }
    let hidden = total_all - out.len();
    out.sort_by(|a, b| (a.rank, &a.instance, a.order).cmp(&(b.rank, &b.instance, b.order)));
    println!("melee diff {scn} frame {frame}: {} differing fields, {hidden} more hidden as known-unmodelled (--all shows them) (C = {}, Rust = native sim from frame 0); '*' = read by the golden hash ({} shown fields are)", out.len(), snap_path.rsplit(['/', '\\']).next().unwrap_or(&snap_path), out.iter().filter(|d| d.hashed).count());
    for d in out.iter().take(max) {
        let gc = d.gc.map_or("-".to_string(), |g| format!("{g:#x}"));
        println!("  {:<14} {:<16} {}{:<34} gc {:<7} C {}  |  Rust {}", d.strukt, d.instance, if d.hashed { "*" } else { " " }, d.field, gc, d.c, d.rust);
    }
    if out.len() > max && max > 0 {
        println!("  ... {} more (raise --max)", out.len() - max);
    }
    if !model_note.is_empty() {
        println!("  note: {model_note}");
    }
    if out.is_empty() { ExitCode::SUCCESS } else { ExitCode::from(1) }
}

/// `--frames A,B,C`: one `diff --frame N` child per frame (each re-simulates from 0), with the baseline check on.
/// Frames beyond what the sim reaches are skipped. The run fails if any frame has a difference outside the baseline,
/// and, with `--baseline-check`, if a baseline line of the checked subsystems is hit by none of the frames (STALE:
/// remove it; baselines only shrink).
fn diff_frames(args: &[String], scn: &str, list: &str) -> ExitCode {
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let exe = match std::env::current_exe() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("melee diff: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut base: Vec<String> = vec!["diff".into()];
    let mut it = args.iter().skip(1);
    while let Some(x) = it.next() {
        if x == "--frames" {
            it.next();
        } else {
            base.push(x.clone());
        }
    }
    let check = get("--baseline-check");
    let mut hits = std::collections::BTreeSet::new();
    let (mut bad, mut ran) = (0, 0);
    for f in list.split(',').filter_map(|v| v.trim().parse::<i32>().ok()) {
        let mut a = base.clone();
        a.extend(["--frame".into(), f.to_string(), "--print-hits".into()]);
        match std::process::Command::new(&exe).args(&a).output() {
            Ok(o) => {
                let text = String::from_utf8_lossy(&o.stdout);
                for l in text.lines() {
                    if let Some(k) = l.strip_prefix("HIT ") {
                        hits.insert(k.to_string());
                        if args.iter().any(|a| a == "--print-hits") {
                            println!("{l}"); // gate.sh unions these over all scenarios
                        }
                    } else {
                        println!("{l}");
                    }
                }
                match o.status.code() {
                    Some(0) => ran += 1,
                    Some(1) => {
                        ran += 1;
                        bad += 1;
                    }
                    Some(3) => println!("{scn} frame {f}: beyond the sim's last frame, skipped"),
                    _ => {
                        eprint!("{}", String::from_utf8_lossy(&o.stderr));
                        bad += 1;
                    }
                }
            }
            Err(e) => {
                eprintln!("melee diff: {e}");
                bad += 1;
            }
        }
    }
    let mut stale = 0;
    // STALE is only meaningful over the whole frame set of every scenario; `--stale` is for a single-scenario check
    // (gate.sh unions the HIT lines instead).
    if check.is_some() && ran > 0 && args.iter().any(|a| a == "--stale") {
        let subs: Option<Vec<String>> = get("--subsystems").map(|v| v.split(',').map(str::to_string).collect());
        for (name, keys) in load_baselines(check.as_deref()) {
            if subs.as_ref().is_some_and(|s| !s.iter().any(|x| x == name)) {
                continue;
            }
            for k in keys.iter().filter(|k| !hits.contains(*k)) {
                println!("STALE {name}: {k}");
                stale += 1;
            }
        }
    }
    println!("melee diff {scn}: {ran} frames compared, {bad} with differences outside the baseline, {stale} stale baseline lines");
    if bad == 0 && stale == 0 { ExitCode::SUCCESS } else { ExitCode::from(1) }
}

fn compare(snap: &Snap, st: &State, o: &Opts, out: &mut Vec<Diff>) -> Result<(), String> {
    let mut push = |rank: u8, order: (u32, u32), strukt: &'static str, instance: String, field: String, gc: Option<u32>, c: String, rust: String| {
        out.push(Diff { rank, order, strukt, instance, field, gc, c, rust, hashed: false });
    };
    // Fighters.
    let gobjs = snap.plink(8)?;
    let pid = crate::vs_match::snapshot::gc_off::<Fighter>("player_id");
    for (slot, bytes) in &st.fighters {
        let fp_va = gobjs.iter().map(|&g| snap.u32(g + 0x2c)).find(|&u| u != 0 && snap.u8(u + pid) as i32 == *slot).ok_or(format!("no C fighter for player {}", slot + 1))?;
        let cf: Fighter = snap.decode_flat(fp_va);
        let cb = raw(&cf);
        let inst = format!("P{} fighter", slot + 1);
        let tab = crate::diff_layout::T_Fighter;
        let bits = crate::diff_layout::bitfields("Fighter");
        let root = Kind::Struct("Fighter", tab);
        let mut k = 0u32;
        walk(cb, bytes, "", 0, 0, bytes.len(), &root, "Fighter", bits, o, &mut |f, gc, c, r| {
            k += 1;
            push(0, (*slot as u32, gc * 64 + k % 64), "Fighter", inst.clone(), f, Some(gc), c, r);
        });
    }
    if st.engine {
        // The fighters' JObj trees (preorder: a joint, its children, then its siblings).
        for (slot, rj) in &st.joints {
            let Some(fg) = gobjs.iter().copied().find(|&g| {
                let u = snap.u32(g + 0x2c);
                u != 0 && snap.u8(u + pid) as i32 == *slot
            }) else { continue };
            let mut cj: Vec<[u32; 26]> = Vec::new();
            let mut stack = vec![snap.u32(fg + 0x28)];
            while let Some(j) = stack.pop() {
                if j == 0 || cj.len() > 512 {
                    continue;
                }
                let mut w = [0u32; 26];
                w[0] = snap.u32(j + 0x14);
                for k in 0..4 { w[1 + k] = snap.u32(j + 0x1C + 4 * k as u32); }
                for k in 0..3 { w[5 + k] = snap.u32(j + 0x2C + 4 * k as u32); }
                for k in 0..3 { w[8 + k] = snap.u32(j + 0x38 + 4 * k as u32); }
                for k in 0..12 { w[11 + k] = snap.u32(j + 0x44 + 4 * k as u32); }
                cj.push(w);
                stack.push(snap.u32(j + 8));
                stack.push(snap.u32(j + 0x10));
            }
            let names = ["flags", "rot.x", "rot.y", "rot.z", "rot.w", "scale.x", "scale.y", "scale.z", "trans.x", "trans.y", "trans.z", "mtx00", "mtx01", "mtx02", "mtx03", "mtx10", "mtx11", "mtx12", "mtx13", "mtx20", "mtx21", "mtx22", "mtx23"];
            if cj.len() != rj.len() {
                push(0, (*slot as u32, 1 << 30), "JObj tree", format!("P{} joints", slot + 1), "count".into(), None, cj.len().to_string(), rj.len().to_string());
            }
            for (n, (c, r)) in cj.iter().zip(rj.iter()).enumerate() {
                // the world matrices are computed lazily (a stale one is not a state difference): only flags / SRT
                for (k, nm) in names.iter().enumerate().take(if std::env::var_os("MELEE_DIFF_MTX").is_some() { 23 } else { 11 }) {
                    if c[k] != r[k] {
                        let (cs, rs) = if k == 0 { (format!("{:08x}", c[k]), format!("{:08x}", r[k])) } else { (format!("{} ({:08x})", f32::from_bits(c[k]), c[k]), format!("{} ({:08x})", f32::from_bits(r[k]), r[k])) };
                        push(0, (*slot as u32, (1 << 30) + n as u32 * 32 + k as u32), "JObj", format!("P{} j{n}", slot + 1), (*nm).into(), None, cs, rs);
                    }
                }
            }
        }
    }
    if st.engine {
        // The fighters' dynamic bone chains (tail / hair physics state): the internal state the Fighter record does not hold.
        for (slot, sets) in &st.dyns {
            let Some(fp_va) = gobjs.iter().map(|&g| snap.u32(g + 0x2c)).find(|&u| u != 0 && snap.u8(u + pid) as i32 == *slot) else { continue };
            for (i, nodes) in sets.iter().enumerate() {
                let mut n = snap.u32(fp_va + 0x2F0 + 0x18 * i as u32 + 4);
                let mut idx = 0usize;
                while n != 0 && idx < nodes.len() {
                    for k in 1..36u32 {
                        let (c, r) = (snap.u32(n + 4 * k), nodes[idx][k as usize - 1]);
                        if c != r {
                            push(0, (*slot as u32, (2 << 30) + (i as u32 * 64 + idx as u32) * 64 + k), "Dynamics", format!("P{} bone set {i} node {idx}", slot + 1), format!("word {k:#x} (gc +{:#x})", 4 * k), Some(4 * k), format!("{} ({c:08x})", f32::from_bits(c)), format!("{} ({r:08x})", f32::from_bits(r)));
                        }
                    }
                    n = snap.u32(n + 0x90);
                    idx += 1;
                }
            }
        }
    }
    // Stocks.
    let slots = snap.must_sym("_player_slots")?;
    for (slot, rs) in st.stocks.iter().filter(|_| !st.engine) {
        let off = crate::vs_match::snapshot::gc_off::<melee_types::_StaticPlayer>("stocks");
        let cs = snap.u8(slots + 0xE90 * *slot as u32 + off) as i8 as i32;
        if cs != *rs {
            push(0, (*slot as u32, u32::MAX), "StaticPlayer", format!("P{} slot", slot + 1), "stocks".into(), Some(off), cs.to_string(), rs.to_string());
        }
    }
    // Items (p_link 9, in list order).
    let mut citems = Vec::new();
    for g in snap.plink(9)? {
        let it = snap.u32(g + 0x2C);
        if it != 0 {
            citems.push((snap.s32(it + 0x10), snap.s32(it + 0x24), [snap.f32(it + 0x4C), snap.f32(it + 0x50), snap.f32(it + 0x54)], [snap.f32(it + 0x40), snap.f32(it + 0x44), snap.f32(it + 0x48)]));
        }
    }
    if citems.len() != st.items.len() {
        push(1, (0, 0), "Item list", "p_link 9".into(), "count".into(), None, citems.len().to_string(), st.items.len().to_string());
    }
    for (i, (c, r)) in citems.iter().zip(st.items.iter()).enumerate() {
        let inst = format!("item#{i} kind {}", c.0);
        let mut chk = |field: &str, gc: u32, a: u32, b: u32, f: bool, ord: u32| {
            if a != b {
                let s = |v: u32| if f { format!("{} ({v:08x})", f32::from_bits(v)) } else { format!("{}", v as i32) };
                push(1, (i as u32, ord), "Item", inst.clone(), field.into(), Some(gc), s(a), s(b));
            }
        };
        chk("kind", 0x10, c.0 as u32, r.0 as u32, false, 0);
        chk("msid", 0x24, c.1 as u32, r.1 as u32, false, 1);
        for (j, ax) in ["x", "y", "z"].iter().enumerate() {
            chk(&format!("pos.{ax}"), 0x4C + 4 * j as u32, c.2[j].to_bits(), r.2[j].to_bits(), true, 2 + j as u32);
        }
        for (j, ax) in ["x", "y", "z"].iter().enumerate() {
            chk(&format!("vel.{ax}"), 0x40 + 4 * j as u32, c.3[j].to_bits(), r.3[j].to_bits(), true, 5 + j as u32);
        }
    }
    // Stage: the respawn spots' countdowns.
    let ctl = snap.must_sym("_controller")?;
    for i in (0..6u32).filter(|_| !st.engine) {
        let cv = snap.u8(ctl + 0x38 + 14 * i + 8) as i32;
        if cv != st.respawn_busy[i as usize] {
            push(2, (i, 0), "SceneState", "stage".into(), format!("fighters[{i}].x8 (respawn_busy)"), Some(0x38 + 14 * i + 8), cv.to_string(), st.respawn_busy[i as usize].to_string());
        }
    }
    // RNG.
    let seed = snap.u32(snap.must_sym("_seed")?);
    if seed != st.seed {
        push(3, (0, 0), "HSD_Rand", "rng".into(), "logic seed".into(), None, format!("{seed:08x}"), format!("{:08x}", st.seed));
    }
    let rseed = snap.u32(snap.must_sym("_render_seed")?);
    if rseed != st.render_seed && !st.engine {
        push(3, (1, 0), "HSD_Rand", "rng".into(), "render seed".into(), None, format!("{rseed:08x}"), format!("{:08x}", st.render_seed));
    }
    Ok(())
}

/// Line-level diff of two `state_dump`s (lines already covered natively are skipped).
fn model_diff(rust: &str, c: &str, out: &mut Vec<Diff>) {
    use std::collections::BTreeMap;
    let skip = |l: &str| {
        let t: Vec<&str> = l.splitn(3, ' ').collect();
        t.get(1).is_some_and(|x| x.starts_with("fp.")) || matches!(t[0], "frame" | "rand" | "respawn_busy") || l.starts_with("pad") || t.get(1) == Some(&"stocks") || l.starts_with("item")
    };
    let key = |l: &str, seen: &mut BTreeMap<String, u32>| {
        let k: String = l.split(' ').take(3).collect::<Vec<_>>().join(" ");
        let n = seen.entry(k.clone()).or_insert(0);
        *n += 1;
        if *n > 1 { format!("{k}#{n}") } else { k }
    };
    let mut map = |t: &str| {
        let mut seen = BTreeMap::new();
        t.lines().filter(|l| !skip(l)).map(|l| (key(l, &mut seen), l.to_string())).collect::<Vec<_>>()
    };
    let (a, b) = (map(c), map(rust));
    let bm: BTreeMap<_, _> = b.iter().cloned().collect();
    let am: BTreeMap<_, _> = a.iter().cloned().collect();
    let mut i = 0u32;
    let mut emit = |k: &str, cl: Option<&String>, rl: Option<&String>, out: &mut Vec<Diff>| {
        let rank = if k.starts_with('p') && k.as_bytes().get(1).is_some_and(u8::is_ascii_digit) { 0 } else if k.starts_with("ground") || k.starts_with("cg") || k.starts_with("static_player") || k.starts_with("last_bg") { 2 } else { 4 };
        let (cs, rs) = match (cl, rl) {
            (Some(c), Some(r)) => {
                let (ct, rt): (Vec<&str>, Vec<&str>) = (c.split(' ').collect(), r.split(' ').collect());
                let d: Vec<usize> = (0..ct.len().max(rt.len())).filter(|&j| ct.get(j) != rt.get(j)).collect();
                let show = |t: &Vec<&str>| d.iter().take(4).map(|&j| format!("[{j}]{}", t.get(j).copied().unwrap_or("-"))).collect::<Vec<_>>().join(" ");
                (show(&ct), show(&rt))
            }
            (Some(_), None) => ("present".into(), "missing".into()),
            _ => ("missing".into(), "present".into()),
        };
        i += 1;
        let (inst, field) = k.split_once(' ').map_or((k.to_string(), String::new()), |(a, b)| (a.to_string(), b.to_string()));
        out.push(Diff { rank, order: (u32::MAX, i), strukt: "model state", instance: inst, field, gc: None, c: cs, rust: rs, hashed: false });
    };
    for (k, l) in &a {
        match bm.get(k) {
            Some(r) if r == l => {}
            r => emit(k, Some(l), r, out),
        }
    }
    for (k, l) in &b {
        if !am.contains_key(k) {
            emit(k, None, Some(l), out);
        }
    }
}

/// Fighter fields `nh_fighter` reads (rust/melee_types/src/neutral_hash.rs); a difference in one of
/// these is what moves the golden hash, so the baseline never hides them.
const HASHED: &[&str] = &["motion_id", "cur_pos", "self_vel.x", "self_vel.y", "dmg.x1830_percent", "facing_dir", "ground_or_air", "x8c_kb_vel.x", "x8c_kb_vel.y", "gr_vel", "dmg.x195c_hitlag_frames", "shield_health", "x3E4_fighterCmdScript.frame_count", "x1968_jumpsUsed", "x67F", "x680"];

/// `nh_cpu`'s inputs (read only for a CPU-controlled fighter).
const HASHED_CPU: &[&str] = &["cpu.buttons", "cpu.lstick", "cpu.cstick", "cpu.ltrigger", "cpu.rtrigger", "cpu.kind", "cpu.level", "cpu.x14", "cpu.x18", "cpu.x1C", "cpu.x20", "cpu.x24", "cpu.x28", "cpu.x2C", "cpu.x30", "cpu.x34", "cpu.x38", "cpu.x3C", "cpu.x40", "cpu.x7C", "cpu.x80", "cpu.x84", "cpu.xC8", "cpu.xEC", "cpu.command_duration", "cpu.csP"];

/// The diff subsystems (consolidation.md 4.1) and their baselines, `tools/rust-port/diff/baseline/<name>.txt`.
const SUBSYSTEMS: &[(&str, &str)] = &[
    ("gobj", include_str!("../../../tools/rust-port/diff/baseline/gobj.txt")),
    ("fighter", include_str!("../../../tools/rust-port/diff/baseline/fighter.txt")),
    ("item", include_str!("../../../tools/rust-port/diff/baseline/item.txt")),
    ("stage", include_str!("../../../tools/rust-port/diff/baseline/stage.txt")),
    ("mp", include_str!("../../../tools/rust-port/diff/baseline/mp.txt")),
    ("camera", include_str!("../../../tools/rust-port/diff/baseline/camera.txt")),
    ("effect", include_str!("../../../tools/rust-port/diff/baseline/effect.txt")),
    ("particle", include_str!("../../../tools/rust-port/diff/baseline/particle.txt")),
    ("hud", include_str!("../../../tools/rust-port/diff/baseline/hud.txt")),
    ("rng", include_str!("../../../tools/rust-port/diff/baseline/rng.txt")),
    ("globals", include_str!("../../../tools/rust-port/diff/baseline/globals.txt")),
];

fn baseline_lines(text: &str) -> impl Iterator<Item = String> + '_ {
    text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')).map(str::to_string)
}

/// Which subsystem a normalised key (`norm`) belongs to. `model state` rows are filed by the model part they cover.
fn subsystem_of_key(key: &str) -> &'static str {
    let mut w = key.split(' ');
    match (w.next().unwrap_or(""), w.next().unwrap_or("")) {
        ("Fighter", _) => "fighter",
        ("Item" | "Item list", _) => "item",
        ("HSD_Rand", _) => "rng",
        ("StaticPlayer" | "SceneState", _) => "globals",
        ("model", i) if i.starts_with("p*") || i == "joint_mtx" => "fighter",
        ("model", i) if i.starts_with("effect") || i == "next_effect_instance" => "effect",
        ("model", i) if i.starts_with("dptcl") || i == "ps" => "particle",
        ("model", "ground") => "stage",
        ("model", "aobj_globals") => "gobj",
        ("model", _) => "globals",
        _ => "globals",
    }
}

/// Baseline keys per subsystem: the embedded files, or `--baseline-check DIR`'s on disk.
fn load_baselines(dir: Option<&str>) -> std::collections::BTreeMap<&'static str, std::collections::BTreeSet<String>> {
    SUBSYSTEMS
        .iter()
        .map(|(name, text)| {
            let t = match dir {
                Some(d) => std::fs::read_to_string(std::path::Path::new(d).join(format!("{name}.txt"))).unwrap_or_default(),
                None => text.to_string(),
            };
            (*name, baseline_lines(&t).collect())
        })
        .collect()
}

/// A difference with indices and addresses removed: the key the baseline lists.
fn norm(d: &Diff) -> String {
    // `p1`, `effect0`: a word and its index; fighter field names like `x203C` keep their digits.
    let stars = |s: &str| {
        let w = s.trim_end_matches(|c: char| c.is_ascii_digit());
        if w.len() < s.len() && !w.is_empty() && w.chars().all(|c| c.is_ascii_lowercase() || c == '_') { format!("{w}*") } else { s.to_string() }
    };
    let brackets = |s: &str| {
        let mut o = String::new();
        let mut inb = false;
        for ch in s.chars() {
            match ch {
                '[' => { inb = true; o.push(ch); }
                ']' => { inb = false; o.push(ch); }
                c if inb && c.is_ascii_digit() => {}
                c => o.push(c),
            }
        }
        o
    };
    if d.strukt == "model state" {
        let f = d.field.split(' ').next().unwrap_or("");
        let f = if f.starts_with("0x") { "0x*".to_string() } else { stars(f) };
        return format!("model {} {}", stars(&d.instance), f);
    }
    let mut f = d.field.split(" [").next().unwrap().to_string();
    if let Some(i) = f.find("+0x") {
        f.truncate(i);
    }
    format!("{} {}", d.strukt, brackets(&f))
}
