//! melee_world::mp::joint against the real C (src/melee/mp/mplib.c bodies compiled natively with
//! clang, contraction on + gwtool's fmuladd lowering, like the shipping build; see
//! cref/gen_mplib_joint_ref.sh). After every joint/line edit the whole state (list links, joint
//! flags/xE/bounds, line flags, MapLine lo_flags, vertex x10/x14, and the island-refresh calls in
//! order) must match; mpJointFromLine per line and every mpLib_80056C54 output bit must match.

use std::cell::{Cell, RefCell};
use std::panic::{self, AssertUnwindSafe};

use melee_world::mp::check::CollJointRt;
use melee_world::mp::lines::GroundColl;
use melee_world::mp::types::{CollLine, CollVtx, MapJoint, MapLine, MapLineRange};
use melee_world::vec::{Vec2, Vec3};

const REF: &str = include_str!("data/mplib_joint_ref.txt");

thread_local! {
    static ACTIVE: RefCell<Vec<bool>> = const { RefCell::new(Vec::new()) };
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

fn run<T>(f: impl FnOnce() -> T) -> Option<T> {
    BUDGET.with(|b| b.set(100_000));
    match panic::catch_unwind(AssertUnwindSafe(f)) {
        Ok(v) => Some(v),
        Err(e) if e.is::<Loop>() => None,
        Err(e) => panic::resume_unwind(e),
    }
}

fn b(f: f32) -> String {
    if f.is_nan() {
        "nan".into()
    } else {
        format!("{:08x}", f.to_bits())
    }
}
fn hx(s: &str) -> f32 {
    f32::from_bits(u32::from_str_radix(s, 16).unwrap())
}

fn dump(g: &GroundColl) -> String {
    let mut s = format!("S {} {} |", g.joint_list_start, g.joint_list_end);
    for j in &g.joints {
        s += &format!(
            " {} {:x} {} {} {} {} {}",
            j.next,
            j.flags,
            j.x_e as i32,
            b(j.bounding_min.x),
            b(j.bounding_min.y),
            b(j.bounding_max.x),
            b(j.bounding_max.y)
        );
    }
    s += " |";
    for l in &g.lines {
        s += &format!(" {:x}", l.flags);
    }
    s += " |";
    for m in &g.map_lines {
        s += &format!(" {:x}", m.lo_flags);
    }
    s += " |";
    for v in &g.vtx {
        s += &format!(" {} {}", b(v.x10), b(v.x14));
    }
    s += " |";
    for r in &g.island_refresh {
        s += &format!(" {},{},{},{}", r.joint, r.vtx_start, r.vtx_count, r.enabled as i32);
    }
    s
}

fn norm_s(line: &str) -> String {
    // Normalise hex float tokens the same way (only NaN differs); S rows have no NaN source,
    // so compare verbatim.
    line.to_string()
}

#[test]
fn mplib_joint_matches_c() {
    let _ = panic::take_hook();
    panic::set_hook(Box::new(|info| {
        if info.payload().downcast_ref::<Loop>().is_none() {
            eprintln!("{info}");
        }
    }));
    let mut g = GroundColl::default();
    let mut nl = 0usize;
    let mut map_tmp: Vec<MapLine> = Vec::new();
    let mut pending: Option<(i32, Vec3, f32, f32)> = None;
    let (mut ops, mut walks, mut froms, mut loops) = (0, 0, 0, 0);
    for (ln, line) in REF.lines().enumerate() {
        let f: Vec<&str> = line.split_whitespace().collect();
        let int = |i: usize| f[i].parse::<i64>().unwrap();
        match f[0] {
            "G" => {
                g = GroundColl { line_active: active, ..Default::default() };
                nl = int(2) as usize;
                g.line_count = nl as i32;
                g.map_lines = vec![MapLine::default(); nl];
                map_tmp.clear();
                ACTIVE.with(|a| a.borrow_mut().clear());
            }
            "V" => g.vtx.push(CollVtx {
                x0: hx(f[1]),
                x4: hx(f[2]),
                pos: Vec2 { x: hx(f[3]), y: hx(f[4]) },
                x10: hx(f[5]),
                x14: hx(f[6]),
            }),
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
                    mj.ranges[k] =
                        MapLineRange { start: int(8 + 2 * k) as i16, count: int(9 + 2 * k) as i16 };
                }
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
            "O" => {
                let a = int(2) as i32;
                BUDGET.with(|b| b.set(i64::MAX));
                match f[1] {
                    "hide" => g.mp_joint_hide(a),
                    "unhide" => g.mp_joint_unhide(a),
                    "dyn" => g.mp_joint_update_dynamics(a),
                    "e24" => g.mp_lib_80055e24(a),
                    "bound" => g.mp_joint_update_bounding(a),
                    "67c" => g.mp_lib_8005667c(a),
                    "424" => g.mp_lib_80057424(a),
                    "528" => g.mp_lib_80057528(a),
                    "5b0" => g.mp_lib_800575b0(a),
                    "add" => g.mp_joint_list_add(a),
                    "bc0" => g.mp_lib_80057bc0(a),
                    "fdc" => g.mp_lib_80057fdc(a),
                    "044" => g.mp_lib_80058044(a),
                    "b10" => g.mp_joint_set_b10(a),
                    o => panic!("op {o}"),
                }
                ops += 1;
            }
            "S" => {
                assert_eq!(dump(&g), norm_s(line), "state after op, ref line {}", ln + 1);
                g.island_refresh.clear();
            }
            "F" => {
                assert_eq!(g.mp_joint_from_line(int(1) as i32), int(2) as i32, "mpJointFromLine line {}", ln + 1);
                froms += 1;
            }
            "W" => {
                pending = Some((
                    int(1) as i32,
                    Vec3 { x: hx(f[2]), y: hx(f[3]), z: hx(f[4]) },
                    hx(f[5]),
                    hx(f[6]),
                ));
            }
            "R" => {
                let (lid, pos, dist, wmax) = pending.take().unwrap();
                let got = run(|| g.mp_lib_80056c54(lid, pos, dist, wmax));
                walks += 1;
                if f[1] == "LOOP" {
                    assert!(got.is_none(), "expected endless walk, ref line {}", ln + 1);
                    loops += 1;
                    continue;
                }
                let got = got.expect("Rust walk ran out of budget");
                let un = Vec3 { x: -9.0, y: -9.0, z: -9.0 };
                let s = match got {
                    None => format!("0 -5 {} {} {} {} {} {} {}", 0xdead, b(un.x), b(un.y), b(un.z), b(un.x), b(un.y), b(un.z)),
                    Some(w) => {
                        let n = w.normal.unwrap_or(un);
                        format!(
                            "{} {} {} {} {} {} {} {} {}",
                            w.result as i32,
                            w.line_id,
                            w.flags.unwrap_or(0xdead),
                            b(n.x),
                            b(n.y),
                            b(n.z),
                            b(w.vec.x),
                            b(w.vec.y),
                            b(w.vec.z)
                        )
                    }
                };
                let want = format!(
                    "{} {} {} {} {} {} {} {} {}",
                    f[1],
                    f[2],
                    f[3],
                    b(hx(f[4])),
                    b(hx(f[5])),
                    b(hx(f[6])),
                    b(hx(f[7])),
                    b(hx(f[8])),
                    b(hx(f[9]))
                );
                assert_eq!(s, want, "mpLib_80056C54 ref line {}", ln + 1);
            }
            _ => panic!("bad ref line {}", ln + 1),
        }
    }
    let _ = (nl, &map_tmp);
    eprintln!("ops {ops}, mpJointFromLine {froms}, mpLib_80056C54 {walks} ({loops} endless)");
    assert!(ops > 4000 && walks > 9000);
}
