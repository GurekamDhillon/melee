//! melee_world::mp::island and mp::stitch against the real C (src/melee/mp/mpisland.c and the
//! mplib.c stitch/bounds bodies compiled natively with clang, contraction on + gwtool's fmuladd
//! lowering, like the shipping build; see cref/gen_mpisland_ref.sh). After every op the island
//! lists (each segment's arena index = the C's allocation order, and all its fields), every
//! MapLine's `*_id1` links, every joint's flags/xE and both mpLib_80458868 boxes must match;
//! every query result must match bit for bit.

use std::cell::{Cell, RefCell};
use std::panic::{self, AssertUnwindSafe};

use melee_world::mp::check::CollJointRt;
use melee_world::mp::lines::GroundColl;
use melee_world::mp::types::{CollLine, CollVtx, MapJoint, MapLine, MapLineRange, MpCollisionBox};
use melee_world::vec::{Vec2, Vec3};

const REF: &str = include_str!("data/mpisland_ref.txt");

thread_local! {
    static ACTIVE: RefCell<Vec<bool>> = const { RefCell::new(Vec::new()) };
    static JACTIVE: RefCell<Vec<bool>> = const { RefCell::new(Vec::new()) };
    static STAGE: RefCell<Vec<bool>> = const { RefCell::new(Vec::new()) };
    static BUDGET: Cell<i64> = const { Cell::new(0) };
}

struct Loop;

fn active(id: i32) -> bool {
    let b = BUDGET.with(|b| {
        b.set(b.get() - 1);
        b.get()
    });
    if b < 0 {
        panic::panic_any(Loop);
    }
    ACTIVE.with(|a| a.borrow()[id as usize])
}
fn jactive(id: i32) -> bool {
    JACTIVE.with(|a| a.borrow()[id as usize])
}
fn stage(id: i32) -> bool {
    STAGE.with(|a| a.borrow()[id as usize])
}

fn b(f: f32) -> String {
    format!("{:08x}", f.to_bits())
}
fn hx(s: &str) -> f32 {
    f32::from_bits(u32::from_str_radix(s, 16).unwrap())
}

fn dump_list(g: &GroundColl, name: &str, mut p: i32) -> String {
    let mut s = format!(" {name}");
    let mut n = 0;
    while p != -1 && n < 5000 {
        let x = &g.islands.segs[p as usize];
        s += &format!(
            " [{} {} {} {} {} {} {} {} {} {} {} {} {}]",
            p, x.x4, x.x6, b(x.x8.x), b(x.x8.y), b(x.x8.z), b(x.x14.x), b(x.x14.y), b(x.x14.z),
            x.x20, x.x24, x.x26, x.x28
        );
        p = x.next;
        n += 1;
    }
    if p != -1 {
        s += " CYC";
    }
    s
}

fn dump(g: &GroundColl) -> String {
    let h = g.islands.heads;
    let mut s = format!("S {} {} |", h.x8, h.xc);
    for (n, p) in [("n", h.next), ("c", h.x4), ("d", h.x10), ("e", h.x14), ("h", h.x18), ("i", h.x1c), ("f", h.x20)] {
        s += &dump_list(g, n, p);
    }
    s += " |";
    for m in &g.map_lines {
        s += &format!(" {},{}", m.prev_id1, m.next_id1);
    }
    s += " |";
    for j in &g.joints {
        s += &format!(" {:x},{}", j.flags, j.x_e as i32);
    }
    s += " |";
    for bx in &g.coll_box {
        s += &format!(" {} {} {} {}", b(bx.top), b(bx.bottom), b(bx.left), b(bx.right));
    }
    s
}

fn bx(f: &[&str]) -> MpCollisionBox {
    MpCollisionBox { top: hx(f[0]), bottom: hx(f[1]), left: hx(f[2]), right: hx(f[3]) }
}

#[test]
fn mpisland_matches_c() {
    let _ = panic::take_hook();
    panic::set_hook(Box::new(|info| {
        if info.payload().downcast_ref::<Loop>().is_none() {
            eprintln!("{info}");
        }
    }));
    let mut g = GroundColl::default();
    let mut pending: Option<Box<dyn FnMut(&mut GroundColl) -> String>> = None;
    let mut looped = false;
    let mut c_loop_op = false;
    let (mut ops, mut queries, mut loops) = (0, 0, 0);
    for (ln, line) in REF.lines().enumerate() {
        let f: Vec<&str> = line.split_whitespace().collect();
        let int = |i: usize| f[i].parse::<i64>().unwrap();
        match f[0] {
            "G" => {
                g = GroundColl { line_active: active, joint_active: jactive, stage_joint: stage, ..Default::default() };
                let nl = int(2) as usize;
                g.line_count = nl as i32;
                g.map_lines = vec![MapLine::default(); nl];
                ACTIVE.with(|a| a.borrow_mut().clear());
                JACTIVE.with(|a| a.borrow_mut().clear());
                STAGE.with(|a| a.borrow_mut().clear());
            }
            "V" => g.vtx.push(CollVtx { pos: Vec2 { x: hx(f[1]), y: hx(f[2]) }, ..Default::default() }),
            "L" => {
                let x0 = int(1) as u32;
                g.map_lines[x0 as usize] = MapLine {
                    v0_idx: int(2) as u16,
                    v1_idx: int(3) as u16,
                    prev_id0: int(4) as i16,
                    next_id0: int(5) as i16,
                    prev_id1: int(6) as i16,
                    next_id1: int(7) as i16,
                    hi_flags: int(8) as u16,
                    lo_flags: int(9) as u16,
                };
                g.lines.push(CollLine { x0, flags: int(10) as u32 });
                ACTIVE.with(|a| a.borrow_mut().push(int(11) != 0));
            }
            "J" => {
                let mut mj = MapJoint { vtx_start: int(6) as i16, vtx_count: int(7) as i16, ..Default::default() };
                for k in 0..5 {
                    mj.ranges[k] = MapLineRange { start: int(10 + 2 * k) as i16, count: int(11 + 2 * k) as i16 };
                }
                JACTIVE.with(|a| a.borrow_mut().push(int(8) != 0));
                STAGE.with(|a| a.borrow_mut().push(int(9) != 0));
                g.joints.push(CollJointRt {
                    next: -1,
                    inner: g.map_joints.len() as i32,
                    flags: int(1) as u32,
                    x_e: false,
                    bounding_min: Vec2 { x: hx(f[2]), y: hx(f[3]) },
                    bounding_max: Vec2 { x: hx(f[4]), y: hx(f[5]) },
                });
                g.map_joints.push(mj);
            }
            "M" => {
                g.map_ranges[0] = MapLineRange { start: int(1) as i16, count: int(2) as i16 };
                g.map_ranges[1] = MapLineRange { start: int(3) as i16, count: int(4) as i16 };
                g.coll_box = [bx(&f[5..9]), bx(&f[9..13])];
            }
            "O" => {
                let a = || int(2) as i32;
                let op = |g: &mut GroundColl| match f[1] {
                    "a728" => g.mp_island_8005a728(),
                    "b334" => g.mp_island_8005b334(a(), int(3) as i32, int(4) as i32, int(5) != 0),
                    "batch" => {
                        let cnt = a();
                        let dirty: Vec<u8> = (0..cnt as usize).map(|k| int(3 + k) as u8).collect();
                        g.mp_island_script_batch(&dirty, cnt);
                    }
                    "free" => g.mp_island_stage_slot_free(),
                    "81dc" => g.mp_lib_800581dc(a(), int(3) as i32),
                    "bound" => g.mp_joint_update_bounding(a()),
                    "8560" => g.mp_lib_80058560(),
                    "8614" => g.mp_lib_80058614_floor(),
                    "xe" => g.joints[a() as usize].x_e = int(3) != 0,
                    "87fc" => g.mp_lib_800587fc(),
                    "8820" => g.mp_lib_80058820(),
                    "fl" => g.joints[a() as usize].flags = int(3) as u32,
                    "lf" => g.lines[a() as usize].flags = int(3) as u32,
                    "add" => {
                        g.mp_joint_list_add(a());
                        g.mp_island_drain_refresh();
                    }
                    "hide" => {
                        g.mp_joint_hide(a());
                        g.mp_island_drain_refresh();
                    }
                    o => panic!("op {o}"),
                };
                BUDGET.with(|b| b.set(100_000));
                looped = panic::catch_unwind(AssertUnwindSafe(|| op(&mut g))).map_err(|e| {
                    if !e.is::<Loop>() {
                        panic::resume_unwind(e)
                    }
                }).is_err();
                ops += 1;
            }
            "Q" => {
                let w: Vec<String> = f.iter().map(|s| s.to_string()).collect();
                pending = Some(Box::new(move |g: &mut GroundColl| {
                    let i = |k: usize| w[k].parse::<i64>().unwrap();
                    match w[1].as_str() {
                        "ab54" => format!("R {}", g.mp_island_8005ab54(i(2) as i32)),
                        "ac14" => {
                            let p = Vec3 { x: hx(&w[2]), y: hx(&w[3]), z: 0.0 };
                            format!("R {}", g.mp_island_8005ac14(p, hx(&w[4])))
                        }
                        "ac8c" => format!("R {}", g.mp_island_8005ac8c(i(2) as i32) as i32),
                        "ace8" => {
                            let un = Vec3 { x: -9.0, y: -9.0, z: -9.0 };
                            let (l, r) = g.mp_island_8005ace8(i(2) as i32, i(3) != 0, i(4) != 0);
                            let (l, r) = (l.unwrap_or(un), r.unwrap_or(un));
                            format!("R {} {} {} {} {} {}", b(l.x), b(l.y), b(l.z), b(r.x), b(r.y), b(r.z))
                        }
                        q => panic!("query {q}"),
                    }
                }));
            }
            "R" => {
                if f[1] == "LOOP" {
                    // The C walk hit the budget inside the op or query just issued.
                    if let Some(mut q) = pending.take() {
                        BUDGET.with(|b| b.set(100_000));
                        let r = panic::catch_unwind(AssertUnwindSafe(|| q(&mut g)));
                        assert!(matches!(&r, Err(e) if e.is::<Loop>()), "expected endless walk, ref line {}", ln + 1);
                    } else {
                        assert!(looped, "expected endless walk in op, ref line {}", ln + 1);
                        c_loop_op = true;
                    }
                    loops += 1;
                    continue;
                }
                let mut q = pending.take().expect("R without Q");
                BUDGET.with(|b| b.set(100_000));
                let got = panic::catch_unwind(AssertUnwindSafe(|| q(&mut g)))
                    .unwrap_or_else(|_| panic!("Rust query ran out of budget, ref line {}", ln + 1));
                assert_eq!(got, line, "query result, ref line {}", ln + 1);
                queries += 1;
            }
            "S" => {
                assert_eq!(looped, c_loop_op, "Rust op endless walk mismatch, ref line {}", ln + 1);
                c_loop_op = false;
                assert_eq!(dump(&g), line, "state, ref line {}", ln + 1);
                looped = false;
            }
            _ => panic!("bad ref line {}", ln + 1),
        }
    }
    eprintln!("ops {ops}, queries {queries}, endless walks {loops}");
    assert!(ops > 4000 && queries > 1000);
}
