//! melee_world::mp::lines against the real C (src/melee/mp/mplib.c compiled natively with
//! clang; see cref/gen_mplib_lines_ref.sh). Every result and every float bit must match.

use std::cell::{Cell, RefCell};
use std::panic::{self, AssertUnwindSafe};

use melee_world::mp::lines::GroundColl;
use melee_world::mp::types::{CollLine, CollVtx, MapLine};
use melee_world::vec::{ps_vec_normalize, Vec2, Vec3};

const REF: &str = include_str!("data/mplib_lines_ref.txt");

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

/// Float bits, except every NaN prints as "nan": NaN sign/payload depends on how the host
/// compiler arranged the ops (x86 default NaN is negative, Gekko's is positive), so the native
/// clang reference cannot pin it. Only zero-length lines produce one (see PSVECNormalize).
fn bits(v: Vec3) -> String {
    let b = |f: f32| if f.is_nan() { "nan".to_string() } else { format!("{:08x}", f.to_bits()) };
    format!("{} {} {}", b(v.x), b(v.y), b(v.z))
}

fn ref_bits(f: &[&str]) -> String {
    let v: Vec<f32> = f.iter().map(|h| f32::from_bits(u32::from_str_radix(h, 16).unwrap())).collect();
    bits(Vec3 { x: v[0], y: v[1], z: v[2] })
}

fn id_fn(g: &GroundColl, name: &str, id: i32) -> i32 {
    match name {
        "mpLineGetNext" => g.mp_line_get_next(id),
        "mpLineGetPrev" => g.mp_line_get_prev(id),
        "mpLineNextNonFloor" => g.mp_line_next_non_floor(id),
        "mpLinePrevNonFloor" => g.mp_line_prev_non_floor(id),
        "mpLinePrevNonCeiling" => g.mp_line_prev_non_ceiling(id),
        "mpLineNextNonCeiling" => g.mp_line_next_non_ceiling(id),
        "mpLineNextNonLeftWall" => g.mp_line_next_non_left_wall(id),
        "mpLinePrevNonLeftWall" => g.mp_line_prev_non_left_wall(id),
        "mpLinePrevNonRightWall" => g.mp_line_prev_non_right_wall(id),
        "mpLineNextNonRightWall" => g.mp_line_next_non_right_wall(id),
        "mpLib_80053394_Floor" => g.mp_lib_80053394_floor(id),
        "mpLib_80053448_Floor" => g.mp_lib_80053448_floor(id),
        "mpLib_800534FC_Floor" => g.mp_lib_800534fc_floor(id),
        "mpLib_800536CC_Floor" => g.mp_lib_800536cc_floor(id),
        "mpLib_8005389C_Ceiling" => g.mp_lib_8005389c_ceiling(id),
        "mpLib_80053950_Ceiling" => g.mp_lib_80053950_ceiling(id),
        "mpLib_80053A04_Ceiling" => g.mp_lib_80053a04_ceiling(id),
        "mpLib_80053BD4_Ceiling" => g.mp_lib_80053bd4_ceiling(id),
        "mpLineGetKind" => g.mp_line_get_kind(id) as i32,
        "mpLineGetFlags" => g.mp_line_get_flags(id) as i32,
        "mpLib_80054ED8" => g.mp_lib_80054ed8(id) as i32,
        _ => panic!("unknown fn {name}"),
    }
}

const POS_FNS: &[&str] = &[
    "mpLib_80053DA4_Floor", "mpLib_80053ECC_Floor", "mpFloorGetRight", "mpFloorGetLeft",
    "mpCeilingGetRight", "mpCeilingGetLeft", "mpLeftWallGetTop", "mpLeftWallGetBottom",
    "mpRightWallGetTop", "mpRightWallGetBottom", "mpLineGetV1Pos", "mpLineGetV0Pos",
    "mpLineGetNormal",
];

fn pos_fn(g: &GroundColl, name: &str, id: i32) -> Vec3 {
    match name {
        "mpLib_80053DA4_Floor" => g.mp_lib_80053da4_floor(id),
        "mpLib_80053ECC_Floor" => g.mp_lib_80053ecc_floor(id),
        "mpFloorGetRight" => g.mp_floor_get_right(id),
        "mpFloorGetLeft" => g.mp_floor_get_left(id),
        "mpCeilingGetRight" => g.mp_ceiling_get_right(id),
        "mpCeilingGetLeft" => g.mp_ceiling_get_left(id),
        "mpLeftWallGetTop" => g.mp_left_wall_get_top(id),
        "mpLeftWallGetBottom" => g.mp_left_wall_get_bottom(id),
        "mpRightWallGetTop" => g.mp_right_wall_get_top(id),
        "mpRightWallGetBottom" => g.mp_right_wall_get_bottom(id),
        "mpLineGetV1Pos" => g.mp_line_get_v1_pos(id),
        "mpLineGetV0Pos" => g.mp_line_get_v0_pos(id),
        "mpLineGetNormal" => g.mp_line_get_normal(id),
        _ => panic!("unknown fn {name}"),
    }
}

#[test]
fn line_queries_match_c() {
    panic::set_hook(Box::new(|info| {
        if info.payload().downcast_ref::<Loop>().is_none() {
            eprintln!("{info}");
        }
    }));
    let mut g = GroundColl { line_active: active, ..Default::default() };
    let (mut checked, mut graphs) = (0usize, 0usize);
    for (n, row) in REF.lines().enumerate() {
        let f: Vec<&str> = row.split(' ').collect();
        let int = |i: usize| f[i].parse::<i64>().unwrap();
        let hex = |i: usize| f32::from_bits(u32::from_str_radix(f[i], 16).unwrap());
        let ctx = || format!("ref line {}: {row}", n + 1);
        match f[0] {
            "G" => {
                graphs += 1;
                g.vtx.clear();
                g.lines.clear();
                g.map_lines = vec![MapLine::default(); int(2) as usize];
                g.line_count = int(2) as i32;
                ACTIVE.with(|a| a.borrow_mut().clear());
            }
            "V" => g.vtx.push(CollVtx { pos: Vec2 { x: hex(1), y: hex(2) }, ..Default::default() }),
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
            "R" => {
                let got = run(|| id_fn(&g, f[1], int(2) as i32));
                assert_eq!(got, Some(int(3) as i32), "{}", ctx());
            }
            "P" => {
                let got = run(|| pos_fn(&g, f[1], int(2) as i32)).map(bits);
                assert_eq!(got.as_deref(), Some(ref_bits(&f[3..6]).as_str()), "{}", ctx());
            }
            "LOOP" => {
                let id = int(2) as i32;
                let got = if f[1] == "mpLinesConnected" {
                    run(|| g.mp_lines_connected(id, int(3) as i32)).map(|_| ())
                } else if POS_FNS.contains(&f[1]) {
                    run(|| pos_fn(&g, f[1], id)).map(|_| ())
                } else {
                    run(|| id_fn(&g, f[1], id)).map(|_| ())
                };
                assert!(got.is_none(), "expected an endless walk: {}", ctx());
            }
            "C" => {
                let got = run(|| g.mp_lines_connected(int(1) as i32, int(2) as i32));
                assert_eq!(got, Some(int(3) != 0), "{}", ctx());
            }
            "F" => {
                let id = int(1) as i32;
                let save = *g.map_line(id);
                g.mp_lib_80054d68(id, int(2) as u32);
                assert_eq!(g.map_line(id).lo_flags as i64, int(3), "{}", ctx());
                let idx = g.lines[id as usize].x0 as usize;
                g.map_lines[idx] = save;
            }
            "N" => {
                let v = Vec3 { x: hex(1), y: hex(2), z: hex(3) };
                assert_eq!(bits(ps_vec_normalize(v)), ref_bits(&f[4..7]), "{}", ctx());
            }
            _ => panic!("bad {}", ctx()),
        }
        checked += 1;
    }
    let _ = panic::take_hook();
    assert!(graphs > 0 && checked > 1000);
}
