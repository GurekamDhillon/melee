//! The enemy-command VM on the player's own Brawl files (skipped when they are absent). Never prints a path or any
//! file content: only names, counts and command ids.
//!
//! * `vm_coverage`: compiles all eight bosses and the 23 enemies against the shared script file and lists every
//!   command / requirement / engine value the VM cannot place (the deliverable is an empty list, or this list).
//! * `vm_rayquaza_runs`: Rayquaza from the VM, headless: statuses change, motions run, hash replay is identical.
//! * `vm_stress_bench` (release, `--ignored`): a boss plus 8 Primids, per-frame cost, depth-7 resim, snapshot /
//!   restore, hash cost.

use brawl_boss::archive;
use brawl_boss::file::BossFile;
use brawl_boss::vm::{self, fnv, leaves, param_map, Actor, Input, Program, COMMON_FNV, FILES};
use std::path::{Path, PathBuf};

fn local_root() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("MELEE_BRAWL_LOCAL") {
        return Some(PathBuf::from(p));
    }
    let mut d = Some(std::env::current_dir().ok()?);
    while let Some(p) = d {
        let c = p.join("_build").join("brawl");
        if c.join("disc").join("files").is_dir() {
            return Some(c);
        }
        d = p.parent().map(Path::to_path_buf);
    }
    None
}
fn ir_dir(root: &Path) -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("MELEE_BOSS_IR_DIR") {
        return Some(PathBuf::from(p));
    }
    for c in [root.join("boss-ir"), root.parent()?.join("boss-ir")] {
        if c.is_dir() {
            return Some(c);
        }
    }
    None
}

struct Disc {
    root: PathBuf,
    ir: Option<PathBuf>,
    cache: std::collections::HashMap<String, Vec<Vec<u8>>>,
}
impl Disc {
    fn open() -> Option<Disc> {
        let root = local_root()?;
        let ir = ir_dir(&root);
        Some(Disc { root, ir, cache: Default::default() })
    }
    fn leaves(&mut self, arc: &str) -> &Vec<Vec<u8>> {
        let p = self.root.join("disc").join("files").join("stage").join("adventure").join(arc);
        self.cache.entry(arc.to_string()).or_insert_with(|| {
            let mut o = Vec::new();
            if let Ok(b) = std::fs::read(&p) {
                leaves(&b, &mut o, 0);
            }
            o
        })
    }
    fn leaf(&mut self, arc: &str, h: u64) -> Option<Vec<u8>> {
        self.leaves(arc)
            .iter()
            .find(|l| if h == 0 { archive::looks_like_boss_file(l, None) } else { fnv(l) == h })
            .cloned()
    }
    fn common(&mut self) -> Option<BossFile> {
        BossFile::parse_raw(&self.leaf("adventure_common.arc", COMMON_FNV)?).ok()
    }
    fn program(&mut self, name: &str, common: &BossFile) -> Result<Program, String> {
        let &(_, irn, arc, h) = FILES.iter().find(|f| f.0 == name).ok_or("unknown name")?;
        let leaf = self.leaf(arc, h).ok_or("file not found")?;
        let bf = BossFile::parse_raw(&leaf).map_err(|e| format!("{e:?}"))?;
        let ir = self.ir.as_ref().and_then(|d| std::fs::read_to_string(d.join(format!("{irn}.brawl.ir.json"))).ok());
        let pm = ir.as_deref().map(param_map).unwrap_or_default();
        let acc = self
            .ir
            .as_ref()
            .and_then(|d| std::fs::read_to_string(d.join("enemies").join("raw").join("dec").join("accmap.json")).ok())
            .map(|j| vm::acc_map(&j, vm::class_of(name)))
            .unwrap_or_default();
        let pm = vm::merged_map(&pm, &acc);
        let mut p = Program::build(&bf, Some(common), &pm).map_err(|e| e.to_string())?;
        p.inits = ir.as_deref().map(vm::var_inits).unwrap_or_default();
        p.block_values = self
            .ir
            .as_ref()
            .and_then(|d| std::fs::read_to_string(d.join("vm-values.txt")).ok())
            .map(|t| vm::block_values(&t))
            .unwrap_or_default();
        Ok(p)
    }
}

#[test]
fn vm_coverage() {
    let Some(mut d) = Disc::open() else {
        eprintln!("vm_coverage: no local Brawl files, skipped");
        return;
    };
    let common = d.common().expect("shared script file");
    let mut total = std::collections::BTreeMap::<String, u32>::new();
    let mut clean = 0;
    for &(name, ..) in FILES {
        match d.program(name, &common) {
            Ok(p) => {
                let u = p.unknown();
                println!(
                    "vm: {name}: {} instructions, {} rules, {} statuses, {} motions; {} left{}{}",
                    p.code.len(),
                    p.rules.len(),
                    p.status_enter.len(),
                    p.motion_names.len(),
                    u.len(),
                    if u.is_empty() { "" } else { ": " },
                    u.join(", ")
                );
                if u.is_empty() {
                    clean += 1;
                }
                for (k, n) in &p.diag.unknown_cmds {
                    *total.entry(format!("cmd {k:04X}")).or_default() += n;
                }
                for (k, n) in &p.diag.unknown_reqs {
                    *total.entry(format!("req R{k}")).or_default() += n;
                }
                for (k, n) in &p.diag.unknown_values {
                    *total.entry(format!("value {k}")).or_default() += n;
                }
                if !p.diag.unmapped_params.is_empty() {
                    println!("vm: {name}: params not in the IR map: {:?}", p.diag.unmapped_params.keys().collect::<Vec<_>>());
                }
            }
            Err(e) => println!("vm: {name}: LOAD ERROR {e}"),
        }
    }
    println!("vm: {clean}/{} files with nothing left; across all: {total:?}", FILES.len());
}

fn libm_abs(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

fn targets(f: u32) -> Input {
    // two scripted players walking back and forth across the arena
    let x0 = -60.0 + ((f % 240) as f32 - 120.0).abs();
    let x1 = 50.0 - ((f % 180) as f32 - 90.0).abs() * 0.8;
    let mut i = Input { ground: true, ..Default::default() };
    i.targets[0] = Some([x0, 0.0]);
    i.targets[1] = Some([x1, 10.0]);
    if f % 97 == 0 {
        i.damage = 7.0;
    }
    i
}

/// A boss from the VM, headless, 3600 frames against two scripted players and a hit every 97 frames: it must move
/// through its statuses, and a replay from the frame-1800 snapshot must give identical hashes.
fn boss_runs(name: &str, start: i32) {
    let Some(mut d) = Disc::open() else {
        eprintln!("vm_{name}_runs: no local Brawl files, skipped");
        return;
    };
    let common = d.common().expect("shared script file");
    let p = d.program(name, &common).expect(name);
    let mut a = vm::OFF;
    vm::begin(&p, &mut a, 1, 5, 0x1234, start, true);
    let mut statuses = std::collections::BTreeSet::new();
    let mut motions = std::collections::BTreeSet::new();
    let mut hashes = Vec::new();
    let mut snap = None;
    let mut trail = Vec::new();
    let mut ext = [0.0f32, f32::MIN, f32::MAX];
    for f in 0..3600u32 {
        if f == 1800 {
            snap = Some(a);
        }
        vm::step(&p, &mut a, &targets(f));
        for e in a.events() {
            if e.kind == vm::EV_STATUS && trail.len() < 80 {
                trail.push((f, e.v));
            }
        }
        statuses.insert(a.status);
        motions.insert(a.motion);
        ext[0] = ext[0].max(libm_abs(a.pos[0]));
        ext[1] = ext[1].max(a.pos[1]);
        ext[2] = ext[2].min(a.pos[1]);
        hashes.push(a.hash());
    }
    println!(
        "vm: {name} 3600 frames: statuses {:?}, motions {:?}, HP {:.1} ({}%), dead {}, pos ({:.1}, {:.1}), last hash {:08X}",
        statuses,
        motions,
        a.hp(),
        a.hp_pct(),
        a.dead(),
        a.pos[0],
        a.pos[1],
        hashes[3599]
    );
    println!("vm: {name} status trail (frame, status): {trail:?}");
    println!("vm: {name} extent: |x| <= {:.1}, y in [{:.1}, {:.1}]", ext[0], ext[2], ext[1]);
    // the boss stays inside generous stage bounds (the arenas' blast zones are within +-300 x, -150..300 y)
    assert!(ext[0] <= 300.0 && ext[1] <= 300.0 && ext[2] >= -150.0, "{name} left the stage bounds");
    let mut b = snap.unwrap();
    for f in 1800..3600u32 {
        vm::step(&p, &mut b, &targets(f));
        assert_eq!(b.hash(), hashes[f as usize], "{name}: replay diverged at frame {f}");
    }
    assert!(statuses.len() > 3, "{name} should move through statuses");
}

/// Status 2 = Hub for both (the IR's cycle starts there; status 0 is DyingNoEffect).
#[test]
fn vm_rayquaza_runs() {
    boss_runs("rayquaza", 2);
}
#[test]
fn vm_ridley_runs() {
    boss_runs("ridley", 2);
}
#[test]
#[ignore]
fn vm_galleom_runs() {
    boss_runs("galleom", 2);
}

/// Release only: `cargo test --release -p brawl_boss --test vm_disc vm_stress_bench -- --ignored --nocapture`.
#[test]
#[ignore]
fn vm_stress_bench() {
    use std::time::Instant;
    let Some(mut d) = Disc::open() else {
        eprintln!("vm_stress_bench: no local Brawl files, skipped");
        return;
    };
    let common = d.common().expect("shared script file");
    let boss = d.program("rayquaza", &common).expect("rayquaza");
    let prim = d.program("primid", &common).expect("primid");
    println!("vm: primid rules {}", prim.rules.len());
    let progs: Vec<&Program> = std::iter::once(&boss).chain(std::iter::repeat(&prim).take(8)).collect();
    let mut world = [vm::OFF; 9];
    for (k, p) in progs.iter().enumerate() {
        vm::begin(p, &mut world[k], 1, 5, 77 + k as u32, if k == 0 { 2 } else { 7 }, k == 0);
        world[k].pos[0] = -80.0 + 20.0 * k as f32;
    }
    let sim = |w: &mut [Actor; 9], f: u32| {
        // no damage: every actor stays active for the whole measurement
        let i = Input { damage: 0.0, ..targets(f) };
        for (k, p) in progs.iter().enumerate() {
            vm::step(p, &mut w[k], &i);
        }
    };
    // warm-up
    for f in 0..600 {
        sim(&mut world, f);
    }
    let live = world.iter().filter(|a| a.live != 0).count();
    let n = 10_000u32;
    let mut t = Vec::with_capacity(n as usize);
    for f in 600..600 + n {
        let s = Instant::now();
        sim(&mut world, f);
        t.push(s.elapsed().as_nanos() as u64);
    }
    t.sort_unstable();
    let pc = |q: f64| t[((t.len() as f64 - 1.0) * q) as usize] as f64 / 1000.0;
    // snapshot / restore and a depth-7 resim (restore + 7 frames), the netplay pattern
    let size = std::mem::size_of::<[Actor; 9]>();
    let reps = 2000;
    let s = Instant::now();
    let mut snap = world;
    for _ in 0..reps {
        snap = std::hint::black_box(world);
    }
    let save_us = s.elapsed().as_nanos() as f64 / reps as f64 / 1000.0;
    let s = Instant::now();
    for _ in 0..reps {
        world = std::hint::black_box(snap);
    }
    let load_us = s.elapsed().as_nanos() as f64 / reps as f64 / 1000.0;
    let mut rs = Vec::new();
    let base = 600 + n;
    for r in 0..500u32 {
        let s = Instant::now();
        world = snap;
        for f in 0..7 {
            sim(&mut world, base + f + r % 3);
        }
        rs.push(s.elapsed().as_nanos() as u64);
    }
    rs.sort_unstable();
    let s = Instant::now();
    let mut hx = 0u32;
    for _ in 0..reps {
        for a in &world {
            hx ^= std::hint::black_box(a).hash();
        }
    }
    let hash_us = s.elapsed().as_nanos() as f64 / reps as f64 / 1000.0;
    println!(
        "vm: stress (Rayquaza + 8 Primids, {live} live after warm-up): per frame p50 {:.1} us, p99 {:.1} us, max {:.1} us",
        pc(0.5),
        pc(0.99),
        *t.last().unwrap() as f64 / 1000.0
    );
    println!(
        "vm: depth-7 resim (restore + 7 frames) p50 {:.1} us, p99 {:.1} us",
        rs[rs.len() / 2] as f64 / 1000.0,
        rs[rs.len() * 99 / 100] as f64 / 1000.0
    );
    println!(
        "vm: snapshot {size} bytes ({} per actor): save {save_us:.2} us, restore {load_us:.2} us; hash of all 9 {hash_us:.2} us ({hx:08X})",
        std::mem::size_of::<Actor>()
    );
    assert!(pc(0.99) < 500.0, "VM over 0.5 ms per frame (p99)");
}

/// The movement-unit kinds each file starts (66xx starts with a constant kind): the kinetic-port plan.
#[test]
#[ignore]
fn vm_kinds() {
    use brawl_boss::vm::A;
    let Some(mut d) = Disc::open() else {
        eprintln!("vm_kinds: no local Brawl files, skipped");
        return;
    };
    let common = d.common().expect("shared script file");
    let mut by_kind = std::collections::BTreeMap::<i32, Vec<String>>::new();
    for &(name, ..) in FILES {
        let Ok(p) = d.program(name, &common) else { continue };
        let mut ks = std::collections::BTreeMap::<(u16, i32), u32>::new();
        for ins in &p.code {
            if matches!(ins.id, 0x6600 | 0x6601 | 0x6602 | 0x6609 | 0x660A | 0x660B) && ins.n >= 2 {
                let k = match p.args[(ins.a + 1) as usize] {
                    A::K(v) => v as i32,
                    _ => -1,
                };
                *ks.entry((ins.id, k)).or_default() += 1;
            }
        }
        let r44 = p.recs.iter().position(|r| r.0 == 44000);
        let ek = |k: i32| -> i32 {
            match r44 {
                Some(r) if k >= 14 => p.rec_w(r as u16, 16 * (k as u32 - 14)) as i32,
                _ => k,
            }
        };
        let l: Vec<String> = ks.iter().map(|((id, k), n)| format!("{id:04X}:{k}=e{} x{n}", ek(*k))).collect();
        println!("kinds: {name}: {}", l.join(", "));
        for ((_, k), _) in ks {
            let e = by_kind.entry(ek(k)).or_default();
            if !e.contains(&name.to_string()) {
                e.push(name.to_string());
            }
        }
    }
    for (k, v) in by_kind {
        println!("kind {k}: {}", v.join(" "));
    }
}

/// The most events any file queues in one frame (headless, 3600 frames, scripted players and hits): sizes EVQ.
/// Release only (the debug build asserts on overflow): `... vm_disc vm_events_max -- --ignored --nocapture`.
#[test]
#[ignore]
fn vm_events_max() {
    let Some(mut d) = Disc::open() else {
        eprintln!("vm_events_max: no local Brawl files, skipped");
        return;
    };
    let common = d.common().expect("shared script file");
    let mut worst = 0;
    for &(name, ..) in FILES {
        let Ok(p) = d.program(name, &common) else { continue };
        let mut m = 0;
        for st in [2, 7] {
            let mut a = vm::OFF;
            vm::begin(&p, &mut a, 1, 5, 0x1234, st, true);
            for f in 0..3600u32 {
                vm::step(&p, &mut a, &targets(f));
                m = m.max(a.ev_n);
            }
        }
        println!("events: {name}: most in one frame {m}");
        worst = worst.max(m);
    }
    println!("events: worst {worst} (queue {})", brawl_boss::vm::EVQ);
    assert!((worst as usize) < brawl_boss::vm::EVQ, "the event queue is too small");
}

/// A synthetic placement path (a circle, 120 frames) for the path movers (kinds 21 / 22).
fn circle(_id: usize, f: f32) -> [f32; 3] {
    let t = f / 120.0 * core::f32::consts::TAU;
    [40.0 * libm_cos(t), 30.0 + 40.0 * libm_sin(t), 0.0]
}
fn libm_cos(x: f32) -> f32 {
    x.cos()
}
fn libm_sin(x: f32) -> f32 {
    x.sin()
}

/// Jyk rides its placement path (6602 kind 21) when the host hands one over: it must leave the start, stay on the
/// circle (plus the offset), and a replay from the frame-600 snapshot must give identical hashes.
#[test]
#[ignore]
fn vm_jyk_path() {
    let Some(mut d) = Disc::open() else {
        eprintln!("vm_jyk_path: no local Brawl files, skipped");
        return;
    };
    let common = d.common().expect("shared script file");
    let Ok(p) = d.program("jyk", &common) else {
        eprintln!("vm_jyk_path: no jyk program, skipped");
        return;
    };
    let path = vm::Path { id: 0, frames: 120, pos: circle };
    let inp = |f: u32| Input { path: Some(path), home: Some([40.0, 30.0]), ..targets(f) };
    let mut a = vm::OFF;
    // status 1: the placed-path variant (6602 kind 21)
    vm::begin(&p, &mut a, 1, 5, 0x1234, 1, true);
    let (mut hashes, mut snap, mut moved) = (Vec::new(), None, 0.0f32);
    let mut pts = std::collections::BTreeSet::new();
    for f in 0..1200u32 {
        if f == 600 {
            snap = Some(a);
        }
        vm::step(&p, &mut a, &inp(f));
        moved += a.moved;
        pts.insert(((a.pos[0] / 10.0) as i32, (a.pos[1] / 10.0) as i32));
        hashes.push(a.hash());
    }
    println!("vm: jyk on a path 1200 frames: moved {moved:.1}, {} distinct cells, pos ({:.1}, {:.1})", pts.len(), a.pos[0], a.pos[1]);
    let mut b = snap.unwrap();
    for f in 600..1200u32 {
        vm::step(&p, &mut b, &inp(f));
        assert_eq!(b.hash(), hashes[f as usize], "replay diverged at {f}");
    }
}

/// Bullet Bill (661D bounce, 661E nose aim) headless in every start status, on the synthetic path: counts the EV_AIM
/// events and the frames a unit moves, and a replay from the frame-600 snapshot must give identical hashes.
#[test]
#[ignore]
fn vm_bullet_bill() {
    let Some(mut d) = Disc::open() else {
        eprintln!("vm_bullet_bill: no local Brawl files, skipped");
        return;
    };
    let common = d.common().expect("shared script file");
    let Ok(p) = d.program("bullet-bill", &common) else {
        eprintln!("vm_bullet_bill: no program, skipped");
        return;
    };
    let path = vm::Path { id: 0, frames: 120, pos: circle };
    // reflected (R75: the 661D bounce, then 661E) for a few frames twice, once on each side of the snapshot; frame 100
    // is before the first wrap of the synthetic path
    let inp = |f: u32| Input { path: Some(path), home: Some([40.0, 30.0]), reflected: (100..104).contains(&f) || (800..804).contains(&f), ..targets(f) };
    for st in 0..p.status_enter.len() as i32 {
        let mut a = vm::OFF;
        vm::begin(&p, &mut a, 1, 5, 0x1234, st, true);
        let (mut hashes, mut snap, mut moved, mut aims) = (Vec::new(), None, 0.0f32, 0u32);
        for f in 0..1200u32 {
            if f == 600 {
                snap = Some(a);
            }
            vm::step(&p, &mut a, &inp(f));
            moved += a.moved;
            aims += a.ev[..(a.ev_n as usize).min(vm::EVQ)].iter().filter(|e| e.kind == vm::EV_AIM).count() as u32;
            hashes.push(a.hash());
        }
        if st == 4 {
            assert!(aims > 0, "status 4: the reflection did not bounce");
        }
        println!("vm: bullet-bill status {st}: moved {moved:.1}, {aims} aims, pos ({:.1}, {:.1}), status now {}", a.pos[0], a.pos[1], a.status);
        let mut b = snap.unwrap();
        for f in 600..1200u32 {
            vm::step(&p, &mut b, &inp(f));
            assert_eq!(b.hash(), hashes[f as usize], "status {st}: replay diverged at {f}");
        }
    }
}
