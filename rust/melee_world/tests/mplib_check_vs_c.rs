//! melee_world::mp::check against the real C (src/melee/mp/mplib.c bodies compiled natively with
//! clang, contraction on + gwtool's fmuladd lowering, like the shipping build; see
//! cref/gen_mplib_check_ref.sh). Every result, every float bit (NaN as "nan"), and the joint
//! flags / didCheckBounding after every call must match.

use std::cell::RefCell;

use melee_world::mp::check::{CollJointRt, SegHit};
use melee_world::mp::lines::GroundColl;
use melee_world::mp::types::{CollLine, CollVtx, MapJoint, MapLine, MapLineRange};
use melee_world::vec::{Vec2, Vec3};

const REF: &str = include_str!("data/mplib_check_ref.txt");

thread_local! {
    static LINE_ACTIVE: RefCell<Vec<bool>> = const { RefCell::new(Vec::new()) };
    static JOINT_ACTIVE: RefCell<Vec<bool>> = const { RefCell::new(Vec::new()) };
}

fn line_active(id: i32) -> bool {
    LINE_ACTIVE.with(|a| a.borrow()[id as usize])
}
fn joint_active(id: i32) -> bool {
    JOINT_ACTIVE.with(|a| a.borrow()[id as usize])
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

const UNSET: Vec3 = Vec3 { x: -9.0, y: -9.0, z: -9.0 };

/// Format like the C `hit()` row (without the name), from (r, pos, line, flags, normal).
fn row(r: i32, p: Vec3, lid: i32, fl: u32, n: Vec3) -> String {
    format!("{r} {} {} {} {lid} {fl} {} {} {}", b(p.x), b(p.y), b(p.z), b(n.x), b(n.y), b(n.z))
}
fn seg(h: Option<SegHit>) -> String {
    match h {
        Some(h) => row(1, h.pos, h.line_id, h.flags, h.normal),
        None => row(0, UNSET, -5, 0xdead, UNSET),
    }
}

/// The C reference row with its float fields normalised (`nan`).
fn norm_ref(f: &[&str]) -> String {
    // r px py pz lid fl nx ny nz
    format!(
        "{} {} {} {} {} {} {} {} {}",
        f[0],
        b(hx(f[1])),
        b(hx(f[2])),
        b(hx(f[3])),
        f[4],
        f[5],
        b(hx(f[6])),
        b(hx(f[7])),
        b(hx(f[8]))
    )
}

struct Query {
    q: [f32; 8],
    yoff: f32,
    lskip: i32,
    jskip: i32,
    jonly: i32,
    usecb: bool,
    dir: i32,
    checks: u32,
    pre: bool,
}

fn call(g: &mut GroundColl, name: &str, c: &Query, cb_tab: &[bool]) -> String {
    let [ax, ay, bx, by, cx, cy, dx, dy] = c.q;
    let (js, jo) = (c.jskip, c.jonly);
    let mut cb = |id: i32| cb_tab[id as usize];
    let cbo: Option<&mut dyn FnMut(i32) -> bool> = if c.usecb { Some(&mut cb) } else { None };
    match name {
        "mpCheckFloor" => seg(g.mp_check_floor(ax, ay, bx, by, c.yoff, c.lskip, js, jo, cbo)),
        "mpCheckFloorRemap" => seg(g.mp_check_floor_remap(ax, ay, bx, by, c.yoff, c.lskip, js, jo, cbo)),
        "mpCheckCeiling" => seg(g.mp_check_ceiling(ax, ay, bx, by, js, jo)),
        "mpCheckCeilingRemap" => seg(g.mp_check_ceiling_remap(ax, ay, bx, by, js, jo)),
        "mpCheckLeftWall" => seg(g.mp_check_left_wall(ax, ay, bx, by, js, jo)),
        "mpCheckLeftWallRemap" => seg(g.mp_check_left_wall_remap(ax, ay, bx, by, js, jo)),
        "mpCheckRightWall" => seg(g.mp_check_right_wall(ax, ay, bx, by, js, jo)),
        "mpCheckRightWallRemap" => seg(g.mp_check_right_wall_remap(ax, ay, bx, by, js, jo)),
        "mpLib_800511A4_RightWall" | "mpLib_800515A0_LeftWall" => {
            let r = if name == "mpLib_800511A4_RightWall" {
                g.mp_lib_800511a4_right_wall(ax, ay, bx, by, cx, cy, dx, dy, js, jo)
            } else {
                g.mp_lib_800515a0_left_wall(ax, ay, bx, by, cx, cy, dx, dy, js, jo)
            };
            match r {
                Some(id) => row(1, UNSET, id, 0xdead, UNSET),
                None => row(0, UNSET, -5, 0xdead, UNSET),
            }
        }
        "mpLib_8005199C_Floor" => {
            let r = g.mp_lib_8005199c_floor(Vec3 { x: ax, y: ay, z: 0.0 }, js, jo);
            row(r, UNSET, -5, 0xdead, UNSET)
        }
        "mpLib_80051BA8_Floor" => {
            let (r, v) = g.mp_lib_80051ba8_floor(c.lskip, js, jo, c.dir, cx, cy, dx, dy);
            row(r, v.unwrap_or(UNSET), -5, 0xdead, UNSET)
        }
        "mpCheckMultiple" => seg(g.mp_check_multiple(ax, ay, bx, by, c.checks, js, jo)),
        "mpCheckAll" => seg(g.mp_check_all(js, jo, ax, ay, bx, by)),
        "mpCheckAllRemap" => seg(g.mp_check_all_remap(js, jo, ax, ay, bx, by)),
        _ => panic!("unknown fn {name}"),
    }
}

#[test]
fn check_matches_c() {
    let mut g = GroundColl { line_active, joint_active, ..Default::default() };
    let mut cb_tab: Vec<bool> = Vec::new();
    let mut cur: Option<Query> = None;
    let (mut rows, mut hits) = (0usize, 0usize);
    for (n, line) in REF.lines().enumerate() {
        let f: Vec<&str> = line.split(' ').collect();
        let int = |i: usize| f[i].parse::<i64>().unwrap();
        match f[0] {
            "G" => {
                g.vtx.clear();
                g.lines.clear();
                g.map_lines = vec![MapLine::default(); int(2) as usize];
                g.line_count = int(2) as i32;
                g.joints.clear();
                g.map_joints.clear();
                g.joint_list_start = int(4) as i32;
                g.did_check_bounding = false;
                cb_tab.clear();
                LINE_ACTIVE.with(|a| a.borrow_mut().clear());
                JOINT_ACTIVE.with(|a| a.borrow_mut().clear());
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
                LINE_ACTIVE.with(|a| a.borrow_mut().push(int(11) != 0));
                cb_tab.push(int(12) != 0);
            }
            "J" => {
                let mut mj = MapJoint::default();
                for k in 0..5 {
                    mj.ranges[k] =
                        MapLineRange { start: int(8 + 2 * k) as i16, count: int(9 + 2 * k) as i16 };
                }
                g.joints.push(CollJointRt {
                    next: int(1) as i32,
                    inner: g.map_joints.len() as i32,
                    flags: int(2) as u32,
                    bounding_min: Vec2 { x: hx(f[3]), y: hx(f[4]) },
                    bounding_max: Vec2 { x: hx(f[5]), y: hx(f[6]) },
                });
                g.map_joints.push(mj);
                JOINT_ACTIVE.with(|a| a.borrow_mut().push(int(7) != 0));
            }
            "C" => {
                let mut q = [0f32; 8];
                for (k, v) in q.iter_mut().enumerate() {
                    *v = hx(f[1 + k]);
                }
                cur = Some(Query {
                    q,
                    yoff: hx(f[9]),
                    lskip: int(10) as i32,
                    jskip: int(11) as i32,
                    jonly: int(12) as i32,
                    usecb: int(13) != 0,
                    dir: int(14) as i32,
                    checks: int(15) as u32,
                    pre: int(16) != 0,
                });
            }
            "R" => {
                let c = cur.as_ref().unwrap();
                if c.pre {
                    let [a, b_, cc, d, e, ff, gg, h] = c.q;
                    g.mp_bounding_check3(a, b_, cc, d, e, ff, gg, h);
                }
                let got = call(&mut g, f[1], c, &cb_tab);
                let mut got = format!("{got} {}", g.did_check_bounding as i32);
                for j in &g.joints {
                    got += &format!(" {:x}", j.flags);
                }
                let mut want = norm_ref(&f[2..11]);
                for t in &f[11..] {
                    want += " ";
                    want += t;
                }
                assert_eq!(got, want, "ref line {}: {line}", n + 1);
                if c.pre {
                    g.mp_uncheck_bounding();
                }
                rows += 1;
                hits += (f[2] != "0" && f[2] != "-1") as usize;
            }
            _ => panic!("bad ref line {}", n + 1),
        }
    }
    assert!(rows > 40_000 && hits > 2_000, "rows {rows} hits {hits}");
}
