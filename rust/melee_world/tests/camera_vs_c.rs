//! melee_world::cm::camera against the real C (src/melee/cm/camera.c bodies compiled natively
//! with clang, contraction on + gwtool's fmuladd lowering, like the shipping build; see
//! cref/gen_camera_ref.sh). Replays every `IN` case and compares every output line exactly.

use std::panic::{self, AssertUnwindSafe};

use melee_types::{CameraBounds, CameraTransformState, CmSubject, Vec as Vec3};
use melee_world::cm::camera::{CamStage, CameraWorld};

const REF: &str = include_str!("data/camera_ref.txt");

fn hx(s: &str) -> f32 {
    f32::from_bits(u32::from_str_radix(s, 16).unwrap())
}
fn pf(o: &mut String, f: f32) {
    o.push_str(&format!(" {:08x}", f.to_bits()));
}
fn pv(o: &mut String, v: Vec3) {
    pf(o, v.x);
    pf(o, v.y);
    pf(o, v.z);
}
fn pt(o: &mut String, t: &CameraTransformState) {
    pv(o, t.interest);
    pv(o, t.target_interest);
    pv(o, t.position);
    pv(o, t.target_position);
    pf(o, t.fov);
    pf(o, t.target_fov);
}
fn pb(o: &mut String, b: &CameraBounds) {
    pf(o, b.x_min);
    pf(o, b.y_min);
    pf(o, b.x_max);
    pf(o, b.y_max);
    o.push_str(&format!(" {}", b.total_subjects));
    pf(o, b.z_pos);
}

struct Tok<'a>(std::str::SplitWhitespace<'a>);
impl Tok<'_> {
    fn i(&mut self) -> i64 {
        self.0.next().unwrap().parse().unwrap()
    }
    fn f(&mut self) -> f32 {
        hx(self.0.next().unwrap())
    }
    fn v(&mut self) -> Vec3 {
        Vec3 { x: self.f(), y: self.f(), z: self.f() }
    }
}

fn psubj(w: &CameraWorld) -> String {
    let mut o = "S".to_string();
    for s in &w.subjects {
        o.push_str(&format!(" {} {}", s.state_timer, s.was_framed()));
        pf(&mut o, s.ext.h.x);
        pf(&mut o, s.ext.h.y);
        pf(&mut o, s.ext.v.x);
        pf(&mut o, s.ext.v.y);
        pf(&mut o, s.ext.v.z);
    }
    o
}

fn case_frame(t: &mut Tok, bpts: &[(Vec3, i32, f32)]) -> (Vec<String>, World) {
    let _k = t.i();
    let n = t.i() as usize;
    let mut w = CameraWorld::default();
    w.geno_fly_any = t.i() != 0;
    w.script_lift_bounds = t.i() != 0;
    let one_p = t.i() != 0;
    t.i();
    let s: Vec<f32> = (0..14).map(|_| t.f()).collect();
    w.stage = CamStage {
        left: s[0],
        right: s[1],
        top: s[2],
        bottom: s[3],
        track_ratio: s[4],
        zoom_rate: s[5],
        max_depth: s[6],
        info_x20: s[7],
        info_x24: s[8],
        pan_angle_rad: s[9],
        tcam_offset: [s[10], s[11], s[12]],
        ground_801c4368: s[13],
    };
    for _ in 0..n {
        let mut c = CmSubject::zeroed();
        c.state = t.i() as u32;
        c.set_force_inactive(t.i() as u8);
        c.set_was_framed(t.i() as u8);
        c.state_timer = t.i() as i16;
        w.geno_fly_subject.push(t.i() != 0);
        c.pos.x = t.f();
        c.pos.y = t.f();
        c.ext.h.x = t.f();
        c.ext.h.y = t.f();
        c.ext.v.x = t.f();
        c.ext.v.y = t.f();
        c.ext.v.z = t.f();
        c.target_ext.h.x = t.f();
        c.target_ext.h.y = t.f();
        c.target_ext.v.x = t.f();
        c.target_ext.v.y = t.f();
        c.target_ext.v.z = t.f();
        w.subjects.push(c);
    }
    let mut tr = CameraTransformState::zeroed();
    tr.interest = t.v();
    tr.target_interest = t.v();
    tr.position = t.v();
    tr.target_position = t.v();
    tr.fov = t.f();
    tr.target_fov = t.f();
    w.game_camera.x2BC = t.f();
    w.game_camera.quake_offset.x = t.f();
    w.game_camera.quake_offset.y = t.f();
    w.game_camera.quake_scale = t.f();
    let speed = t.f();
    let scale = t.f();

    let mut out = vec![];
    for &(p, d, df) in bpts {
        out.push(format!(
            "B {} {} {}",
            w.camera_80029124(&p, d),
            w.camera_80031154(&p) as i32,
            w.camera_8003118c(&p, df) as i32
        ));
    }
    w.camera_800293e0();
    out.push(psubj(&w));
    let mut b = CameraBounds::zeroed();
    w.camera_8002958c(&mut b, &tr);
    out.push(psubj(&w));
    let mut o = "R".to_string();
    pb(&mut o, &b);
    out.push(o);
    w.camera_80029aac(&b, &mut tr, speed);
    let mut b2 = b;
    w.camera_80029bc4(&mut b2, &tr);
    let mut o = "T".to_string();
    pt(&mut o, &tr);
    pb(&mut o, &b2);
    out.push(o);
    w.camera_80029c88(&mut tr, scale);
    let mut o = "T".to_string();
    pt(&mut o, &tr);
    out.push(o);
    let r = panic::catch_unwind(AssertUnwindSafe(|| {
        let mut t2 = tr;
        w.camera_80029cf8(&b2, &mut t2);
        t2
    }));
    match r {
        Ok(t2) => {
            tr = t2;
            let mut o = "T".to_string();
            pt(&mut o, &tr);
            out.push(o);
        }
        Err(e) => {
            let m = e.downcast_ref::<&str>().map(|s| s.to_string()).unwrap_or_default();
            out.push(format!("ASSERT {}", &m[9..13]));
        }
    }
    w.camera_apply_quake(&b2, &tr, one_p);
    let mut o = "Q".to_string();
    pf(&mut o, w.game_camera.translation.x);
    pf(&mut o, w.game_camera.translation.y);
    pf(&mut o, w.game_camera.quake_offset.x);
    pf(&mut o, w.game_camera.quake_offset.y);
    out.push(o);
    (out, w)
}
type World = CameraWorld;

#[test]
fn camera_matches_c() {
    let lines: Vec<&str> = REF.lines().collect();
    let mut i = 0;
    let (mut nf, mut ne, mut nq, mut rows) = (0, 0, 0, 0);
    let mut world_q = CameraWorld::default();
    while i < lines.len() {
        let line = lines[i];
        assert!(line.starts_with("IN "), "line {}: {line}", i + 1);
        let mut t = Tok(line[3..].split_whitespace());
        match t.0.next().unwrap() {
            "F" => {
                nf += 1;
                // Each B line carries its own random distance (int and float) after the
                // three results; the point is the subject's pos from the IN line.
                let save = t.0.clone();
                let n = {
                    let mut t2 = Tok(save.clone());
                    t2.i();
                    t2.i() as usize
                };
                let mut bpts = vec![];
                let mut t2 = Tok(save.clone());
                for _ in 0..6 {
                    t2.i();
                }
                for _ in 0..14 {
                    t2.f();
                }
                for _ in 0..n {
                    for _ in 0..5 {
                        t2.i();
                    }
                    let x = t2.f();
                    let y = t2.f();
                    for _ in 0..10 {
                        t2.f();
                    }
                    bpts.push((x, y));
                }
                let mut bp = vec![];
                for (k, &(x, y)) in bpts.iter().enumerate() {
                    let bl = lines[i + 1 + k];
                    let mut bt = Tok(bl.split_whitespace());
                    bt.0.next();
                    for _ in 0..3 {
                        bt.i();
                    }
                    let d = bt.i() as i32;
                    let df = bt.f();
                    bp.push((Vec3 { x, y, z: 0.0 }, d, df));
                }
                let (out, _) = case_frame(&mut t, &bp);
                for (k, o) in out.iter().enumerate() {
                    let r = lines[i + 1 + k];
                    let r = if r.starts_with("B ") {
                        r.split_whitespace().take(4).collect::<Vec<_>>().join(" ")
                    } else {
                        r.to_string()
                    };
                    assert_eq!(o, &r, "frame case {nf}, out line {k}");
                    rows += 1;
                }
                i += 1 + out.len();
            }
            "E" => {
                ne += 1;
                let mode = t.i() as u8;
                let a = u32::from_str_radix(t.0.next().unwrap(), 16).unwrap() as i32;
                let b = u32::from_str_radix(t.0.next().unwrap(), 16).unwrap() as i32;
                let f0 = t.f();
                let f1 = t.f();
                let mut w = CameraWorld::default();
                w.game_camera.set_x341_b5_b6(mode);
                w.game_camera.x378.s32_v = a;
                w.game_camera.x37C.s32_v = b;
                let (r, v) = w.camera_8002e158(f0, f1);
                let mut o = format!("E {}", r as i32);
                pf(&mut o, v);
                assert_eq!(o, lines[i + 1], "lerp case {ne}");
                rows += 1;
                i += 2;
            }
            "U" => {
                nq += 1;
                let ops = t.i() as usize;
                world_q.game_camera.quake_frames_left = [0; 5];
                for q in world_q.game_camera.quakes.iter_mut().flatten() {
                    *q = melee_types::CameraQuake::zeroed();
                }
                world_q.game_camera.quake_gobj = core::ptr::null_mut();
                world_q.stubs = Default::default();
                i += 1;
                for _ in 0..ops {
                    let mut ot = Tok(lines[i].split_whitespace());
                    ot.0.next();
                    match ot.0.next().unwrap() {
                        "U" => world_q.camera_update_quakes(),
                        "S" => world_q.camera_stop_quake(ot.i() as u32),
                        "R" => {
                            let k = ot.i() as u32;
                            let p = if ot.i() != 0 { Some(ot.v()) } else { None };
                            world_q.camera_request_quake(k, p);
                        }
                        x => panic!("op {x}"),
                    }
                    let gc = &world_q.game_camera;
                    let mut o = format!(
                        "Z {} {} {}",
                        world_q.stubs.quake_gobj_frees,
                        world_q.stubs.grlib_quake.len(),
                        !gc.quake_gobj.is_null() as i32
                    );
                    for j in 0..5 {
                        o.push_str(&format!(" {}", gc.quake_frames_left[j]));
                    }
                    for j in 0..32 {
                        let q = &gc.quakes[j / 16][j % 16];
                        o.push_str(&format!(" {}", q.kind));
                        pv(&mut o, q.epicenter);
                    }
                    assert_eq!(o, lines[i + 1], "quake case {nq}");
                    rows += 1;
                    i += 2;
                }
            }
            x => panic!("unknown case {x}"),
        }
    }
    assert_eq!((nf, ne, nq), (3000, 2000, 300));
    eprintln!("camera: {rows} rows matched");
}
