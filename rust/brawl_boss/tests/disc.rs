//! Tests against the player's own files. They skip (pass with a note) when the files are absent, as they are on every
//! machine without the disc. Nothing here prints a path or file contents.
//!
//! * the stage archive: from `MELEE_BRAWL_LOCAL` or `_build/brawl` above the crate (where the game looks);
//! * the reader's decode for comparison: `BRAWL_BOSS_JSON` or `_build/petey-spec/decoded/bosspackun.json`.

mod common;
use brawl_boss::archive;
use brawl_boss::file::BossFile;
use brawl_boss::fx::Fx;
use brawl_boss::rng::{FixedRng, RandomSource, TestRng};
use brawl_boss::runner::{CageEvent, Config, FrameInput, Runner};
use brawl_boss::script::{Arg, SoundKind, Target};
use brawl_boss::world::{Event, Obj, Recorder, ResolvedHitbox};
use common::*;
use std::collections::BTreeMap;

fn petey() -> Option<Vec<u8>> {
    let a = stage_archive()?;
    Some(archive::find_petey_file(&a).expect("the boss file is in the stage archive"))
}

macro_rules! need {
    ($e:expr) => {
        match $e {
            Some(v) => v,
            None => {
                eprintln!("skipped: the player's Brawl files are not present");
                return;
            }
        }
    };
}

fn fx(v: f64) -> Fx {
    Fx((v * 60000.0).round() as i64)
}

/// Motion lengths (CHR0 frame counts, as the port already records them in petey.rs `MOT_LEN`).
const MOT_LEN: [i32; 12] = [70, 160, 160, 30, 30, 721, 90, 71, 31, 20, 271, 160];

fn world() -> Recorder {
    let mut w = Recorder::default();
    w.lengths = MOT_LEN.to_vec();
    w.present = [true, true, false, false];
    w.dx = [Fx::from_int(-50), Fx::from_int(30), Fx::ZERO, Fx::ZERO];
    w.angle = [Fx::from_int(180), Fx::from_int(10), Fx::ZERO, Fx::ZERO];
    w.values.insert(4001, Fx::from_int(100));
    w.values.insert(4002, Fx::from_int(100));
    w
}

#[test]
fn header_and_layout() {
    let f = need!(petey());
    assert_eq!(f.len(), 28_826);
    let bf = BossFile::parse(&f).unwrap();
    assert_eq!(bf.header.data_size, 0x6134);
    assert_eq!(bf.header.reloc_count, 750);
    assert_eq!(bf.header.root_count, 8);
    assert_eq!(bf.header.import_count, 16);
    assert_eq!(bf.imports.len(), 16);
    for i in &bf.imports {
        assert!(brawl_boss::world::Builtin::from_name(&i.name).is_some(), "import without a built-in: {}", i.name);
    }
    assert_eq!(bf.root("data"), Some(0x60C4));
    assert_eq!(bf.motions.len(), 12);
    let names = ["Angry", "BasketAttackL", "BasketAttackR", "DamageLeft", "DamageRight", "Down", "JumpAttack", "JumpAttackLanding", "JumpLoop", "JumpStart", "Wait1", "Wait2"];
    for (m, n) in bf.motions.iter().zip(names) {
        assert_eq!(m.name, n);
    }
    assert!(bf.motions[8].looping() && bf.motions[10].looping() && bf.motions[11].looping() && !bf.motions[1].looping());
    assert_eq!(bf.rules.len(), 31);
    assert_eq!(bf.scripts.len(), 165, "every reachable script");
    let n: usize = bf.scripts.values().map(|s| s.cmds.len()).sum();
    assert_eq!(n, 581);
    // the spec's whole-file command counts (3.11): 64 listed, 63 distinct ids occur
    let mut counts: BTreeMap<u16, usize> = BTreeMap::new();
    for s in bf.scripts.values() {
        for c in &s.cmds {
            *counts.entry(c.id()).or_default() += 1;
            assert!(!matches!(c.op, brawl_boss::script::Op::Unknown), "untyped command {:08x}", c.word);
        }
    }
    let expect: [(u16, usize); 63] = [
        (0x0001, 40), (0x0002, 25), (0x0004, 8), (0x0005, 8), (0x0007, 30), (0x0008, 25), (0x000A, 40), (0x000B, 1),
        (0x000C, 2), (0x000D, 1), (0x000E, 14), (0x000F, 40), (0x0010, 4), (0x0011, 9), (0x0012, 4), (0x0013, 4),
        (0x0018, 7), (0x0202, 11), (0x0409, 13), (0x0410, 1), (0x0604, 6), (0x0605, 2), (0x0619, 9), (0x0808, 1),
        (0x0900, 1), (0x0A00, 28), (0x0A02, 1), (0x0A05, 1), (0x1100, 29), (0x1101, 12), (0x1102, 12), (0x1200, 13),
        (0x1203, 1), (0x1206, 6), (0x1208, 1), (0x120A, 12), (0x120B, 2), (0x120F, 2), (0x1A00, 13), (0x1B02, 5),
        (0x6505, 3), (0x6507, 2), (0x6509, 1), (0x650F, 2), (0x6514, 2), (0x651D, 1), (0x651F, 2), (0x6523, 2),
        (0x6600, 2), (0x6607, 6), (0x660B, 11), (0x660C, 1), (0x6900, 2), (0x6901, 1), (0x6907, 1), (0x6908, 1),
        (0x6A00, 30), (0x6A02, 14), (0x6A03, 2), (0x6B03, 3), (0x6E00, 38), (0x6E01, 2), (0x6E02, 8),
    ];
    assert_eq!(counts, expect.into_iter().collect::<BTreeMap<_, _>>());
}

#[test]
fn spec_cross_check_values() {
    let f = need!(petey());
    let bf = BossFile::parse(&f).unwrap();
    let p = &bf.params;
    // HP, cage HP, death threshold, bounce factor
    assert_eq!(bf.body_hp(), Fx::from_int(99999));
    assert_eq!((p.cage_hp(0), p.cage_hp(1)), (Some(999), Some(999)));
    assert_eq!(bf.death_threshold(), Fx::from_int(10));
    assert_eq!(bf.bounce_factor(), fx(0.75));
    // phase thresholds and the cage proc records
    let (l, r) = (p.cage_proc(0).unwrap(), p.cage_proc(1).unwrap());
    assert_eq!((l.phase1_pct, l.phase2_pct, r.phase1_pct, r.phase2_pct), (50, 25, 50, 25));
    assert_eq!((l.effect, l.effect_bone, r.effect, r.effect_bone), (65, 3, 66, 4));
    assert_eq!((l.a, l.b), (Fx::from_int(120), Fx::from_int(15)));
    // strengths, repeat factors, slot weights
    assert_eq!((p.int(24000), p.int(24001), p.int(24002)), (Some(0), Some(2), Some(3)));
    assert_eq!(p.scalar(4000), Some(fx(0.3)));
    assert_eq!(bf.slot_weights(), [25, 10, 20, 8].map(Fx::from_int));
    assert_eq!(bf.slot_repeat_factor(), fx(0.3));
    // wait ranges and hover
    assert_eq!(p.range(44003).unwrap(), [100, -9, 150, -12].map(Fx::from_int));
    assert_eq!(p.range(44004).unwrap(), [fx(2.0), fx(-0.18), fx(20.0), fx(-1.75)]);
    assert_eq!(p.floats(44005).unwrap(), vec![Fx::from_int(60), fx(-5.4)]);
    assert_eq!(p.floats(44006).unwrap()[0], Fx::from_int(-68));
    assert_eq!(p.floats(44007).unwrap()[0], Fx::from_int(68));
    // weighted lists
    let w = |id, n| p.weighted(id, n).unwrap().iter().map(|e| (e.base, e.slope, e.condition != Target::None)).collect::<Vec<_>>();
    assert_eq!(w(44008, 2), vec![(50, Fx::ZERO, true); 2]);
    assert_eq!(w(44009, 3), vec![(20, Fx::ZERO, true), (20, Fx::ZERO, true), (60, Fx::from_int(2), true)]);
    assert_eq!(w(44010, 2), vec![(20, Fx::ZERO, false), (80, Fx::ZERO, false)]);
    assert_eq!(w(44011, 2), vec![(75, Fx::ZERO, false), (25, Fx::ZERO, false)]);
    // movement kinds
    let k16 = p.kinetic(16).unwrap();
    assert_eq!([k16[1], k16[3], k16[4], k16[7], k16[8]], [fx(0.2), fx(0.015), fx(0.003), fx(1.4), fx(0.03)]);
    let k14 = p.kinetic(14).unwrap();
    assert_eq!((k14[1], k14[3]), (brawl_boss::params::KEEP, fx(-0.06)));
    assert_eq!(p.kinetic(15).unwrap()[4], fx(-0.005));
    // hurtboxes
    let b = bf.body_hurtboxes[0];
    assert_eq!((b.bone(), b.radius, b.offset[2], b.stretch[2]), (28, Fx::from_int(39), Fx::from_int(10), Fx::from_int(10)));
    for (s, y) in [(0usize, -20), (1, 20)] {
        let c = bf.cage_hurtboxes[s][0];
        assert_eq!((c.bone(), c.radius, c.stretch[1]), (9, Fx::from_int(11), Fx::from_int(y)));
    }
    // rules (format spec 4.4)
    let pr: Vec<i32> = bf.rules.iter().map(|r| r.priority).collect();
    assert_eq!(&pr[..8], &[-200, -200, -200, -200, -200, -201, -198, -197]);
    assert_eq!((pr[10], pr[13], pr[28], pr[29], pr[30]), (-100, 200, -100, -100, 0));
    let on: Vec<usize> = (0..31).filter(|&i| bf.rules[i].default_enabled).collect();
    assert_eq!(on, vec![0, 1, 2, 4, 5, 30]);
    let lk: Vec<usize> = (0..31).filter(|&i| bf.rules[i].lock).collect();
    assert_eq!(lk, vec![13, 24, 26]);
    let per: Vec<usize> = (0..31).filter(|&i| bf.rules[i].target_mask == 0x0F).collect();
    assert_eq!(per, vec![13, 24]);
    assert_eq!(bf.rules[5].target_status, 15);
    assert_eq!(bf.rules[10].target_status, 1);
    // the level table
    assert_eq!(brawl_boss::level::level(1, 5), fx(2.5));
}

fn arg_matches(bf: &BossFile, a: &Arg, j: &J) -> bool {
    match (a, j.get("t").s()) {
        (Arg::Int(i), "int") => *i as i64 == j.get("v").i(),
        (Arg::Scalar(f), "scalar") => f.raw() == j.get("raw").i(),
        (Arg::Bool(b), "bool") => *b as i64 == j.get("v").i(),
        (Arg::Var(v), "var") => v.raw() as i64 == j.get("raw").i(),
        (Arg::Req { not, id }, "req") => *not == (j.get("not") == &J::Bool(true)) && *id as i64 == j.get("id").i(),
        (Arg::Offset(o), "type4") => *o as i64 == j.get("raw").i(),
        (Arg::Ptr(t), "ptr") => target_matches(bf, t, j),
        _ => false,
    }
}

fn target_matches(bf: &BossFile, t: &Target, j: &J) -> bool {
    match t {
        Target::None => j.is_null(),
        Target::Local(o) => *o as i64 == if j.is_int() { j.i() } else { j.get("to").i() },
        Target::Import(i) => bf.import_name(*i) == Some(j.get("ext").s()),
        Target::Unresolved(_) => false,
    }
}

#[test]
fn matches_the_reader_decode() {
    let f = need!(petey());
    let js = need!(spec_json());
    let bf = BossFile::parse(&f).unwrap();
    assert_eq!(js.get("container").get("size").i(), f.len() as i64);
    assert_eq!(js.get("container").get("data_size").i(), bf.header.data_size as i64);
    for (m, jm) in bf.motions.iter().zip(js.get("motions").arr()) {
        assert_eq!(m.name, jm.get("name").s());
        assert_eq!(format!("{:08x}", m.flags), jm.get("flags").s().to_lowercase());
    }
    // general and tuning blocks: ints as ints, floats bit-exact
    for (blk, key) in [(&bf.general, "general_block"), (&bf.tuning, "tuning_block")] {
        for (k, v) in js.get(key).arr().iter().enumerate() {
            if v.is_int() {
                assert_eq!(blk[k] as i32 as i64, v.i(), "{key}[{k}]");
            } else {
                assert_eq!(blk[k], (v.f() as f32).to_bits(), "{key}[{k}]");
            }
        }
    }
    // tables
    for (c, key) in ["game", "effect", "sound"].iter().enumerate() {
        for t in 0..3 {
            let jt = js.get("timelines").get(key).at(t).arr();
            assert_eq!(bf.timelines[c][t].len(), jt.len());
            for (x, y) in bf.timelines[c][t].iter().zip(jt) {
                assert!(target_matches(&bf, x, y), "{key} {t}");
            }
        }
    }
    for (tab, key) in [(&bf.status_enter, "enter"), (&bf.status_loop, "loop"), (&bf.status_exit, "exit")] {
        for (x, y) in tab.iter().zip(js.get("status_scripts").get(key).arr()) {
            assert!(target_matches(&bf, x, y), "status {key}");
        }
    }
    assert!(target_matches(&bf, &bf.init, js.get("init_script")));
    for (r, jr) in bf.rules.iter().zip(js.get("rules").arr()) {
        assert_eq!(r.at as i64, jr.get("at").i());
        assert_eq!(r.priority as i64, jr.get("priority").i());
        assert_eq!(r.target_mask as i64, jr.get("target_mask").i());
        assert!(target_matches(&bf, &r.condition, jr.get("condition")));
        assert!(target_matches(&bf, &r.action, jr.get("action")));
        assert_eq!(r.target_status as i64, jr.get("target_status").i());
        assert_eq!(r.default_enabled as i64, jr.get("default_enabled").i());
        assert_eq!(r.lock as i64, jr.get("lock").i());
    }
    // imports: every referring field
    let mut mine: BTreeMap<i64, String> = BTreeMap::new();
    for i in &bf.imports {
        for &fl in &i.fields {
            mine.insert(fl as i64, i.name.clone());
        }
    }
    let theirs: BTreeMap<i64, String> = js.get("imports").obj().unwrap().iter().map(|(k, v)| (k.parse().unwrap(), v.s().to_string())).collect();
    // the reader lists the fields in scripts, tables and rules; every one of them must be in our chains
    for (k, v) in &theirs {
        assert_eq!(mine.get(k), Some(v), "import field {k}");
    }
    // scripts, command by command
    let js_scripts = js.get("scripts").obj().unwrap();
    let mine_at: Vec<i64> = bf.scripts.keys().map(|&k| k as i64).collect();
    let theirs_at: Vec<i64> = js_scripts.keys().map(|k| k.parse().unwrap()).collect::<std::collections::BTreeSet<_>>().into_iter().collect();
    assert_eq!(mine_at, theirs_at);
    for (at, s) in &bf.scripts {
        let jc = js_scripts[&at.to_string()].arr();
        assert_eq!(s.cmds.len(), jc.len(), "script {at}");
        for (c, j) in s.cmds.iter().zip(jc) {
            assert_eq!(c.at as i64, j.get("at").i());
            assert_eq!(format!("{:08X}", c.word), j.get("id").s().to_uppercase());
            assert_eq!(c.args.len(), j.get("args").arr().len());
            for (a, ja) in c.args.iter().zip(j.get("args").arr()) {
                assert!(arg_matches(&bf, a, ja), "script {at} cmd {:08x}: {a:?}", c.word);
            }
        }
    }
    // params, raw
    let jp = js.get("params");
    assert_eq!(bf.params.at as i64, jp.get("base_offset").i());
    for (id, key) in [(44003, "44003_waitA_range"), (44004, "44004_waitB_range"), (44005, "44005_hover"), (44006, "44006_goal_side1")] {
        let mine = bf.params.floats(id).unwrap();
        for (k, v) in jp.get(key).arr().iter().enumerate() {
            assert_eq!(mine[k], Fx::from_f32_bits((v.f() as f32).to_bits()), "{key}");
        }
    }
    for (side, key) in [(0, "44012_cage_left_spawn"), (1, "44013_cage_right_spawn")] {
        let raw = bf.params.cage_spawn(side).unwrap();
        for (k, v) in jp.get(key).arr().iter().enumerate() {
            if v.is_int() && !matches!(bf.params.cage_spawn_ptrs(side)[k], Target::Local(_)) {
                assert_eq!(raw[k] as i32 as i64, v.i(), "{key}[{k}]");
            }
        }
    }
}

// ------------------------------------------------------------------------------------------- runner on the real file

struct Play {
    log: Vec<(u32, i32, Obj, Event)>,
}

fn play(bf: &BossFile, m: i32, frames: u32, level: Fx) -> Play {
    let mut w = world();
    let mut rng = FixedRng::new(&[0]);
    let mut r = Runner::new(bf, Config::default());
    r.set_level(level);
    // the cages: spawned with the boss's truncated L, as at placement
    r.cages[0].vars.floats.insert(0, Fx::from_int(level.trunc()));
    r.cages[1].vars.floats.insert(0, Fx::from_int(level.trunc()));
    r.play_motion(&mut w, &mut rng, m);
    let inp = FrameInput::default();
    for _ in 0..frames {
        r.step(&mut w, &mut rng, &inp);
    }
    Play { log: w.log.into_iter().map(|(s, o, e)| (s.tick, s.frame, o, e)).collect() }
}

impl Play {
    fn ticks(&self, f: impl Fn(Obj, &Event) -> bool) -> Vec<u32> {
        self.log.iter().filter(|e| f(e.2, &e.3)).map(|e| e.0).collect()
    }
    fn hitboxes(&self, obj: Obj) -> Vec<(u32, ResolvedHitbox)> {
        self.log
            .iter()
            .filter_map(|e| match (&e.3, e.2 == obj) {
                (Event::Hitbox(h), true) => Some((e.0, h.clone())),
                _ => None,
            })
            .collect()
    }
    fn sound(&self, id: i32) -> Vec<u32> {
        self.ticks(|_, e| matches!(e, Event::Sound(_, x) if *x == id))
    }
}

const L25: Fx = Fx(150_000);

fn body_contact(h: &ResolvedHitbox, wkb: i32) {
    assert_eq!((h.id, h.bone, h.damage, h.angle, h.kbg, h.wkb, h.bkb), (4, 28, 0, 0, 100, wkb, 0));
    assert_eq!(h.size, fx(25.7));
    assert_eq!(h.offset, [Fx::ZERO, Fx::from_int(3), Fx::from_int(8)]);
    assert_eq!((h.hitlag, h.rehit(), h.target_mask(), h.hit_sound()), (Fx::ZERO, 4, 9, 1));
}

#[test]
fn timeline_angry() {
    let f = need!(petey());
    let bf = BossFile::parse(&f).unwrap();
    let p = play(&bf, 0, 70, L25);
    assert_eq!(p.ticks(|o, e| o == Obj::Boss && *e == Event::Shake(1)), vec![4, 12, 20, 28, 36, 44, 52, 60]);
    assert_eq!(p.sound(9592), vec![9]);
    assert!(p.log.iter().any(|e| e.0 == 0 && matches!(e.3, Event::Effect(..))), "effect cluster at frame 0");
}

#[test]
fn timeline_baskets() {
    let f = need!(petey());
    let bf = BossFile::parse(&f).unwrap();
    for (m, cage, fx_ids) in [(1, 0u8, [190, 191, 192]), (2, 1u8, [180, 181, 182])] {
        let p = play(&bf, m, 160, L25);
        let c = Obj::Cage(cage);
        // body: contact (WKB 5) from frame 0; cleared at 69
        let body = p.hitboxes(Obj::Boss);
        assert_eq!(body[0].0, 0);
        body_contact(&body[0].1, 5);
        assert_eq!(p.ticks(|o, e| o == Obj::Boss && *e == Event::ClearHitboxes), vec![69]);
        // the cage runs boxes A and B at frame 70 with its truncated L (2): 10 + 2.7 * 2 = 15.4 -> 15, 105 - 4
        let cb = p.hitboxes(c);
        assert_eq!(cb.len(), 2);
        for (t, h) in &cb {
            assert_eq!(*t, 70);
            assert_eq!((h.damage, h.angle, h.kbg, h.wkb, h.bkb), (15, 145, 44, 0, 101));
            assert_eq!((h.hitlag, h.rehit(), h.shield_damage(), h.hit_sound(), h.target_mask()), (Fx::from_int(1), 60, 20, 311, 15));
        }
        assert_eq!((cb[0].1.id, cb[0].1.bone, cb[0].1.size, cb[0].1.offset[1]), (0, 9, Fx::from_int(8), fx(5.5)));
        assert_eq!((cb[1].1.id, cb[1].1.bone, cb[1].1.size, cb[1].1.offset[1]), (1, 10, Fx::from_int(10), fx(-3.8)));
        // the cage clears its boxes at 115 (hitbox window 70-114)
        assert_eq!(p.ticks(|o, e| o == c && *e == Event::ClearHitboxes), vec![115]);
        // 20 small shakes from frame 70 (the absolute reading puts them all on frame 70)
        let sh = p.ticks(|o, e| o == Obj::Boss && *e == Event::Shake(0));
        assert_eq!(sh.len(), 20);
        assert!(sh.iter().all(|&t| t == 70));
        // effects at frame 70
        for id in fx_ids {
            // the spec's "0xB9:190" is the decimal id 12 124 190 (= 0xB9 << 16 | 30); see the questions in the notes
            let want = 12_124_000 + id;
            assert!(p.log.iter().any(|e| e.0 == 70 && matches!(&e.3, Event::Effect(_, a) if a.id == want)), "effect {id}");
        }
        // sound at frame 17; the variant voice needs 24000 == 2, which is 0 here
        assert!(p.log.iter().any(|e| e.0 == 17 && matches!(e.3, Event::Sound(..))));
        assert!(p.sound(9606).is_empty() && p.sound(9605).is_empty());
    }
}

#[test]
fn timeline_damage_down() {
    let f = need!(petey());
    let bf = BossFile::parse(&f).unwrap();
    assert_eq!(play(&bf, 3, 30, L25).sound(9594), vec![0]);
    assert_eq!(play(&bf, 4, 30, L25).sound(9595), vec![0]);
    let p = play(&bf, 5, 721, L25);
    let st: Vec<(u32, i32)> = p.log.iter().filter_map(|e| if let Event::StageEvent(x) = e.3 { Some((e.0, x)) } else { None }).collect();
    assert_eq!(st, vec![(0, 5), (1, 6), (180, 7), (420, 8)]);
    assert_eq!(p.sound(9612), vec![0]);
    let last = p.log.iter().filter(|e| matches!(e.3, Event::Effect(..))).map(|e| e.0).max().unwrap();
    assert!(last <= 152, "debris through frame 152");
    assert!(!p.sound(9596).is_empty());
}

#[test]
fn timeline_jumps() {
    let f = need!(petey());
    let bf = BossFile::parse(&f).unwrap();
    // JumpAttack
    let p = play(&bf, 6, 90, L25);
    body_contact(&p.hitboxes(Obj::Boss)[0].1, 6);
    assert_eq!(p.hitboxes(Obj::Boss)[0].0, 0);
    let s0 = p.ticks(|o, e| o == Obj::Boss && *e == Event::Shake(0));
    assert_eq!(s0, (0..13).map(|k| 4 + 5 * k).collect::<Vec<u32>>());
    // game thread 2 also shakes (1) at frame 0 (the spec puts that shake on Wait1; the file has it on JumpAttack)
    let s1 = p.ticks(|o, e| o == Obj::Boss && *e == Event::Shake(1));
    assert_eq!(s1, [0].into_iter().chain((0..5).map(|k| 69 + 5 * k)).collect::<Vec<u32>>());
    assert_eq!(p.sound(9597), vec![0]);
    // JumpAttackLanding at L 2.5
    let p = play(&bf, 7, 71, L25);
    let hb = p.hitboxes(Obj::Boss);
    assert_eq!(hb.len(), 3);
    let (t, c) = &hb[0];
    assert_eq!(*t, 2);
    assert_eq!((c.id, c.bone, c.damage, c.angle, c.kbg, c.wkb, c.bkb), (0, 0, 19, -90, 47, 0, 142)); // 12+3L, 45+L, 146-1.5L
    assert_eq!((c.size, c.offset[1], c.hitlag, c.rehit(), c.shield_damage()), (Fx::from_int(28), Fx::from_int(31), fx(0.5), 120, 30));
    for (k, (t, h)) in hb[1..].iter().enumerate() {
        assert_eq!(*t, 3);
        assert_eq!((h.id, h.part, h.damage, h.angle, h.kbg, h.bkb), (k as i32, 1, 13, 30, 48, 109)); // 8+2L, 112-1.2L
        assert_eq!(h.size, fx(31.25)); // 30 + 0.5L
        assert_eq!(h.offset[0], Fx::from_int(if k == 0 { -29 } else { 29 }));
        assert_eq!(h.shield_damage(), 10);
    }
    assert_eq!(p.ticks(|o, e| o == Obj::Boss && *e == Event::Shake(2)), vec![2]);
    assert_eq!(p.ticks(|o, e| o == Obj::Boss && *e == Event::ClearHitboxes), vec![5]);
    assert_eq!(p.ticks(|o, e| o == Obj::Boss && *e == Event::Shake(1)), vec![5]);
    assert_eq!(p.sound(9598), vec![0]);
    let fxs = p.ticks(|_, e| matches!(e, Event::Effect(_, a) if (12_124_170..=12_124_175).contains(&a.id)));
    assert!(!fxs.is_empty() && fxs.iter().all(|&t| t <= 1));
    // JumpLoop: body contact, 3 small shakes (absolute reading: 0, 10, 10), status-bound sound
    let p = play(&bf, 8, 31, L25);
    body_contact(&p.hitboxes(Obj::Boss)[0].1, 6);
    assert_eq!(p.ticks(|o, e| o == Obj::Boss && *e == Event::Shake(0)), vec![0, 10, 10]);
    assert_eq!(p.ticks(|_, e| *e == Event::Sound(SoundKind::StatusBound, 9602)), vec![0]);
    // JumpStart
    let p = play(&bf, 9, 20, L25);
    body_contact(&p.hitboxes(Obj::Boss)[0].1, 6);
    assert_eq!(p.sound(9601), vec![14]);
}

#[test]
fn timeline_waits() {
    let f = need!(petey());
    let bf = BossFile::parse(&f).unwrap();
    let p = play(&bf, 10, 271, L25);
    body_contact(&p.hitboxes(Obj::Boss)[0].1, 6);
    assert!(p.ticks(|o, e| o == Obj::Boss && matches!(e, Event::Shake(_))).is_empty(), "no shake on Wait1 (see JumpAttack)");
    let p = play(&bf, 11, 160, L25);
    body_contact(&p.hitboxes(Obj::Boss)[0].1, 6);
    assert_eq!(p.sound(9599), vec![20]);
    assert_eq!(p.sound(9600), vec![88]);
    assert!(p.sound(9603).is_empty() && p.sound(9604).is_empty());
}

// ------------------------------------------------------------------------------------------- the fight

fn fight(seed: u64, frames: u32) -> (Vec<(u32, Obj, Event)>, Vec<(i32, i32)>) {
    let f = petey().unwrap();
    let bf = BossFile::parse(&f).unwrap();
    let mut w = world();
    let mut rng = TestRng(seed);
    let mut r = Runner::new(&bf, Config::default());
    r.start(&mut w, &mut rng);
    let mut hp = [999i64, 999];
    let mut edges = Vec::new();
    let mut last = r.status;
    for k in 0..frames {
        // a player hits the body now and then (forwarded to both cages by the scripts) and a cage sometimes
        let hit = k % 97 == 13;
        w.damage = if hit { vec![brawl_boss::world::DamageCheck::Ground] } else { vec![] };
        let before = Fx::from_int(99999);
        let now = if hit { Fx::from_int(99999 - 12) } else { before };
        r.step(&mut w, &mut rng, &FrameInput { hp_now: now, hp_before: before, hit_by_player: hit });
        // apply the forwarded damage the way the cage proc would
        for e in w.log.iter().rev().take_while(|e| e.0.tick == r.tick() - 1) {
            if let Event::LinkFloat { link, event: 5105, value, .. } = &e.2 {
                hp[(*link & 1) as usize] -= value.trunc();
            }
        }
        if k % 211 == 50 && (r.status == 2 || r.status == 3) {
            hp[0] -= 30;
            r.cage_event(&mut w, &mut rng, 0, CageEvent::Damaged);
        }
        for s in 0..2 {
            let pct = Fx::from_int(hp[s].max(0) * 100).mul(Fx::ratio(1, 999));
            w.values.insert(4001 + s as u32, pct);
            if pct <= Fx::from_int(50) && r.cages[s].counter == 0 {
                r.cage_event(&mut w, &mut rng, s as u8, CageEvent::PhaseOne);
            } else if pct <= Fx::from_int(25) && r.cages[s].counter == 1 {
                r.cage_event(&mut w, &mut rng, s as u8, CageEvent::PhaseTwo);
            }
        }
        if r.status != last {
            edges.push((last, r.status));
            last = r.status;
        }
        if r.dead {
            break;
        }
    }
    (w.log.into_iter().map(|(s, o, e)| (s.tick, o, e)).collect(), edges)
}

#[test]
fn fight_flow_and_determinism() {
    need!(petey());
    let (a, edges) = fight(11, 20_000);
    let (b, _) = fight(11, 20_000);
    assert_eq!(a, b, "same seed, same events");
    let (c, _) = fight(12, 20_000);
    assert_ne!(a, c);
    // every transition is one the spec's status table allows (4.3 / 4.4 / 6.3)
    let allowed: &[(i32, &[i32])] = &[
        (2, &[4, 6, 13, 14, 1, 15]),
        (3, &[5, 6, 13, 14, 1, 15]),
        (4, &[11, 12, 1, 15]),
        (5, &[11, 12, 7, 1, 15]),
        (6, &[4, 5, 1, 15]),
        (7, &[8, 1, 15]),
        (8, &[9, 1, 15]),
        (9, &[10, 15]),
        (10, &[2, 7, 1, 15]),
        (11, &[3, 1, 15]),
        (12, &[3, 1, 15]),
        (13, &[4, 5, 1, 15]),
        (14, &[4, 5, 1, 15]),
    ];
    let mut seen = std::collections::BTreeSet::new();
    for (from, to) in &edges {
        seen.insert((*from, *to));
        if *from < 0 {
            continue;
        }
        let ok = allowed.iter().find(|x| x.0 == *from).is_some_and(|x| x.1.contains(to));
        assert!(ok, "transition {from} -> {to}");
    }
    // the whole loop is exercised: baskets, the menu, a jump chain, the phase-up roar
    assert!(seen.contains(&(2, 4)) && seen.contains(&(3, 5)) && seen.contains(&(9, 10)), "{seen:?}");
    assert!(seen.iter().any(|e| e.1 == 6), "the phase-up roar plays: {seen:?}");
    // the phase-up sends L to the cages as an int (truncated) and sets the cage counters
    assert!(a.iter().any(|e| matches!(&e.2, Event::LinkInt { event: 5112, args, .. } if args == &vec![3])), "L 3.3 -> 3");
}

#[test]
fn rng_order_in_a_frame() {
    // the menu (rule 22) draws once for its switch; a basket enter does not draw; Wait B's enter draws once for T
    let f = need!(petey());
    let bf = BossFile::parse(&f).unwrap();
    let mut w = world();
    let mut rng = FixedRng::new(&[0]);
    let mut r = Runner::new(&bf, Config::default());
    r.start(&mut w, &mut rng);
    let d0 = rng.draws;
    r.force_status(&mut w, &mut rng, 5);
    assert_eq!(rng.draws, d0);
    r.step(&mut w, &mut rng, &FrameInput::default());
    // menu weights (20, 20, 65) with no previous pick -> u = 0 picks entry 0 -> basket L in phase 0
    assert_eq!(rng.draws, d0 + 1);
    assert_eq!(r.status, 11);
    assert_eq!(r.vars.int(16), 0, "menuPick");
    let _ = rng.next_u32();
}

#[test]
fn death_on_hit_after_cage_low() {
    let f = need!(petey());
    let bf = BossFile::parse(&f).unwrap();
    let mut w = world();
    let mut rng = FixedRng::new(&[0]);
    let mut r = Runner::new(&bf, Config::default());
    r.start(&mut w, &mut rng);
    // tie: equal cage percents count as the right cage (lowerSideIsLeft clear)
    w.values.insert(4001, Fx::from_int(10));
    w.values.insert(4002, Fx::from_int(10));
    r.step(&mut w, &mut rng, &FrameInput { hp_now: Fx::from_int(99990), hp_before: Fx::from_int(99999), hit_by_player: true });
    assert!(r.dead);
    assert_eq!(r.status, 15);
    assert!(!r.vars.flag(7), "a tie counts as the right cage");
    assert!(w.log.iter().any(|e| e.2 == Event::StageEvent(1)));
    // status 15's loop: the stage is told 1001 (right cage lower)
    r.step(&mut w, &mut rng, &FrameInput::default());
    assert!(w.log.iter().any(|e| e.2 == Event::StageEvent(1001)));
    assert!(w.log.iter().any(|e| e.2 == Event::Sound(SoundKind::Voice, 8837) || matches!(e.2, Event::Sound(_, 8837))));
    assert!(w.log.iter().any(|e| e.2 == Event::CommonEffect(383)));
}

#[test]
fn real_file_fuzz() {
    let f = need!(petey());
    for n in (0..f.len()).step_by(37) {
        let _ = BossFile::parse(&f[..n]);
    }
    let mut rng = TestRng(5);
    for _ in 0..1500 {
        let mut g = f.clone();
        for _ in 0..3 {
            let at = rng.next_u32() as usize % g.len();
            g[at] ^= 1 << (rng.next_u32() % 8);
        }
        if let Ok(bf) = BossFile::parse(&g) {
            let mut w = world();
            let mut rr = TestRng(9);
            let mut r = Runner::new(&bf, Config::default());
            r.start(&mut w, &mut rr);
            for _ in 0..200 {
                r.step(&mut w, &mut rr, &FrameInput::default());
            }
        }
    }
}
