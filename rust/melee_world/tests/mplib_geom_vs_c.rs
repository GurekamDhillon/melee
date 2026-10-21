//! melee_world::mp::geom against the real C (src/melee/mp/mplib.c compiled natively with clang,
//! contraction on + gwtool's fmuladd lowering, like the shipping build; see
//! cref/gen_mplib_geom_ref.sh). Every result and every float bit must match (NaN as "nan").

use std::cell::{Cell, RefCell};
use std::panic::{self, AssertUnwindSafe};

use melee_world::mp::geom::{self, LineHit};
use melee_world::mp::lines::GroundColl;
use melee_world::mp::types::{CollLine, CollVtx, MapLine};
use melee_world::vec::{Vec2, Vec3};

const REF: &str = include_str!("data/mplib_geom_ref.txt");

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

/// Normalise a reference float field the same way (`nan` for any NaN).
fn rb(s: &str) -> String {
    b(hx(s))
}

fn query(g: &GroundColl, name: &str, id: i32, q: Vec3) -> Option<LineHit> {
    match name {
        "mpLib_8004DD90_Floor" => g.mp_lib_8004dd90_floor(id, q),
        "mpLib_8004E090_Ceiling" => g.mp_lib_8004e090_ceiling(id, q),
        "mpLib_8004E398_LeftWall" => g.mp_lib_8004e398_left_wall(id, q),
        "mpLib_8004E684_RightWall" => g.mp_lib_8004e684_right_wall(id, q),
        _ => panic!("unknown fn {name}"),
    }
}

/// C out-param sentinels when the function returns -1 / false.
const UNSET_F: f32 = -7.0;
const UNSET_V: f32 = -9.0;

#[test]
fn geom_matches_c() {
    panic::set_hook(Box::new(|info| {
        if info.payload().downcast_ref::<Loop>().is_none() {
            eprintln!("{info}");
        }
    }));
    let mut g = GroundColl { line_active: active, ..Default::default() };
    let mut last_m: Vec<f32> = Vec::new();
    let mut counts = [0usize; 8];
    for (n, row) in REF.lines().enumerate() {
        let f: Vec<&str> = row.split(' ').collect();
        let int = |i: usize| f[i].parse::<i64>().unwrap();
        let ctx = || format!("ref line {}: {row}", n + 1);
        match f[0] {
            "G" => {
                g.vtx.clear();
                g.lines.clear();
                g.map_lines = vec![MapLine::default(); int(2) as usize];
                g.line_count = int(2) as i32;
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
            "Q" => {
                let q = Vec3 { x: hx(f[3]), y: hx(f[4]), z: 0.0 };
                let got = run(|| query(&g, f[1], int(2) as i32, q)).expect("unexpected loop");
                let (r, d, fl, nv) = match got {
                    Some(h) => (h.line_id, h.dist, h.flags, h.normal),
                    None => (-1, UNSET_F, 0xdead, Vec3 { x: UNSET_V, y: UNSET_V, z: UNSET_V }),
                };
                let got = format!("{r} {} {fl} {} {} {}", b(d), b(nv.x), b(nv.y), b(nv.z));
                let want = format!("{} {} {} {} {} {}", f[5], rb(f[6]), f[7], rb(f[8]), rb(f[9]), rb(f[10]));
                assert_eq!(got, want, "{}", ctx());
                counts[0] += 1;
            }
            "S" => {
                let q = Vec3 { x: hx(f[2]), y: hx(f[3]), z: 0.0 };
                let got = run(|| g.mp_get_speed(int(1) as i32, q)).expect("unexpected loop");
                let (r, v) = match got {
                    Some(v) => (1, v),
                    None => (0, Vec3 { x: UNSET_V, y: UNSET_V, z: UNSET_V }),
                };
                assert_eq!(
                    format!("{r} {} {} {}", b(v.x), b(v.y), b(v.z)),
                    format!("{} {} {} {}", f[4], rb(f[5]), rb(f[6]), rb(f[7])),
                    "{}",
                    ctx()
                );
                counts[1] += 1;
            }
            "E" => {
                let (a, bb, c, d) = run(|| g.mp_lib_8004ed5c(int(1) as i32)).expect("unexpected loop");
                assert_eq!(
                    format!("{} {} {} {}", b(a), b(bb), b(c), b(d)),
                    format!("{} {} {} {}", rb(f[2]), rb(f[3]), rb(f[4]), rb(f[5])),
                    "{}",
                    ctx()
                );
                counts[2] += 1;
            }
            "LOOP" => {
                let id = int(2) as i32;
                let got = if f[1] == "mpLib_8004ED5C" {
                    run(|| g.mp_lib_8004ed5c(id)).map(|_| ())
                } else {
                    let q = Vec3 { x: hx(f[3]), y: hx(f[4]), z: 0.0 };
                    run(|| query(&g, f[1], id, q)).map(|_| ())
                };
                assert!(got.is_none(), "expected an endless walk: {}", ctx());
            }
            "W" => {
                let l = int(2) as i32;
                let (a, bb, c, d) = (hx(f[3]), hx(f[4]), hx(f[5]), hx(f[6]));
                match int(1) {
                    0 => g.mp_line_set_pos(l, a, bb, c, d),
                    1 => g.mp_lib_80056758(l, a, bb, c, d),
                    _ => {
                        let nv = g.vtx.len() as i32;
                        g.mp_vtx_set_pos(l % nv, a, bb)
                    }
                }
            }
            "X" => {
                let (x, y) = g.mp_vtx_get_pos(int(1) as i32);
                assert_eq!((x.to_bits(), y.to_bits()), (hx(f[2]).to_bits(), hx(f[3]).to_bits()), "{}", ctx());
                counts[3] += 1;
            }
            "M" => {
                last_m = f[1..11].iter().map(|s| hx(s)).collect();
                let v = &last_m;
                let (x, y) = geom::mp_remap2d(v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], v[8], v[9]);
                assert_eq!(format!("{} {}", b(x), b(y)), format!("{} {}", rb(f[11]), rb(f[12])), "{}", ctx());
                counts[4] += 1;
            }
            "I" | "H" | "Y" => {
                let v = &last_m;
                let got = match f[0] {
                    "I" => geom::mp_line_intersection(v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]),
                    "H" => geom::mp_line_intersection_h(v[0], v[1], v[2], v[4], v[5], v[6], v[7]),
                    _ => geom::mp_line_intersection_v(v[0], v[1], v[3], v[4], v[5], v[6], v[7]),
                };
                let (r, (x, y)) = match got {
                    Some(p) => (1, p),
                    None => (0, (UNSET_F, UNSET_F)),
                };
                assert_eq!(format!("{r} {} {}", b(x), b(y)), format!("{} {} {}", f[1], rb(f[2]), rb(f[3])), "{}", ctx());
                counts[5] += r as usize;
            }
            "Z" => {
                let s = hx(f[1]);
                assert_eq!(geom::sqrtf(s).to_bits(), hx(f[2]).to_bits(), "{}", ctx());
                counts[6] += 1;
            }
            _ => panic!("bad {}", ctx()),
        }
    }
    let _ = panic::take_hook();
    assert!(counts.iter().take(7).all(|&c| c > 100), "{counts:?}");
}
