//! melee_world::cm::camera accessors, Camera_8002FE38 / 8002EEC8 / 8002BC78 and the PC
//! cm_script helpers against the real C (bodies cut from src/melee/cm/camera.c, compiled with
//! the game flags, contraction on + lower.py; see cref/gen_camera_acc_ref.sh). Each `IN` line is
//! a random game_camera state plus one call; the next `OUT` line is the C's result and state.

use melee_types::Vec as Vec3;
use melee_world::cm::camera::{
    cm_script_bits, cm_script_blend, cm_script_ease, cm_script_float, cm_script_lerp,
    CameraWorld, CmScriptPose,
};

const REF: &str = include_str!("data/camera_acc_ref.txt");

struct Tok<'a>(std::str::SplitWhitespace<'a>);
impl Tok<'_> {
    fn i(&mut self) -> i64 {
        self.0.next().unwrap().parse().unwrap()
    }
    fn b(&mut self) -> u8 {
        self.i() as u8
    }
    fn f(&mut self) -> f32 {
        f32::from_bits(u32::from_str_radix(self.0.next().unwrap(), 16).unwrap())
    }
    fn v(&mut self) -> Vec3 {
        Vec3 { x: self.f(), y: self.f(), z: self.f() }
    }
}

fn pf(o: &mut String, f: f32) {
    o.push_str(&format!(" {:08x}", f.to_bits()));
}
fn pv(o: &mut String, v: Vec3) {
    pf(o, v.x);
    pf(o, v.y);
    pf(o, v.z);
}

fn load(w: &mut CameraWorld, t: &mut Tok) {
    let c = &mut w.game_camera;
    c.mode = t.i() as _;
    c.background_color.r = t.b();
    c.background_color.g = t.b();
    c.background_color.b = t.b();
    c.background_color.a = t.b();
    c.nearz = t.f();
    c.farz = t.f();
    c.transform.interest = t.v();
    c.transform.target_interest = t.v();
    c.transform.position = t.v();
    c.transform.target_position = t.v();
    c.transform.fov = t.f();
    c.transform.target_fov = t.f();
    c.translation.x = t.f();
    c.translation.y = t.f();
    c.quake_scale = t.f();
    c.x2B0 = t.f();
    c.x2B8 = t.i() as i16;
    c.x2BC = t.f();
    c.pause_up = t.v();
    c.set_x341_b0(t.b());
    c.set_x341_b1_b2(t.b());
    c.set_x341_b3_b4(t.b());
    c.set_x341_b5_b6(t.b());
    c.set_x341_b7(t.b());
    c.x350 = t.v();
    c.x368 = t.v();
    c.x374 = t.f();
    c.set_x398_b0(t.b());
    c.set_x398_b1(t.b());
    c.set_x398_b2(t.b());
    c.set_x398_b3(t.b());
    c.set_x398_b4(t.b());
    c.set_x398_b5(t.b());
    c.set_x398_b6_b7(t.b());
    c.set_x399_b0_b1(t.b());
    c.set_x399_b2(t.b());
    c.set_x399_b3(t.b());
    c.set_x399_b4(t.b());
    c.set_x399_b5(t.b());
    c.set_x399_b6(t.b());
    c.set_x399_b7(t.b());
    c.set_x39A_b0(t.b());
    c.set_x39A_b1(t.b());
    c.set_x39A_b2(t.b());
    c.set_x39A_b3(t.b());
    c.set_x39A_b4(t.b());
    c.set_x39A_b5(t.b());
    c.set_x39A_b6(t.b());
    c.set_x39A_b7(t.b());
    w.globals.x40 = t.f();
    w.debug_last_mode = t.i() as _;
}

fn dump(w: &CameraWorld, o: &mut String) {
    let c = &w.game_camera;
    let bg = c.background_color;
    o.push_str(&format!(" {} {} {} {} {}", c.mode as u32, bg.r, bg.g, bg.b, bg.a));
    pf(o, c.nearz);
    pf(o, c.farz);
    pv(o, c.transform.interest);
    pv(o, c.transform.target_interest);
    pv(o, c.transform.position);
    pv(o, c.transform.target_position);
    pf(o, c.transform.fov);
    pf(o, c.transform.target_fov);
    pf(o, c.translation.x);
    pf(o, c.translation.y);
    pf(o, c.quake_scale);
    pf(o, c.x2B0);
    o.push_str(&format!(" {}", c.x2B8));
    pf(o, c.x2BC);
    pv(o, c.pause_up);
    for b in [c.x341_b0(), c.x341_b1_b2(), c.x341_b3_b4(), c.x341_b5_b6(), c.x341_b7()] {
        o.push_str(&format!(" {b}"));
    }
    pv(o, c.x350);
    pv(o, c.x368);
    pf(o, c.x374);
    for b in [
        c.x398_b0(), c.x398_b1(), c.x398_b2(), c.x398_b3(), c.x398_b4(), c.x398_b5(),
        c.x398_b6_b7(), c.x399_b0_b1(), c.x399_b2(), c.x399_b3(), c.x399_b4(), c.x399_b5(),
        c.x399_b6(), c.x399_b7(), c.x39A_b0(), c.x39A_b1(), c.x39A_b2(), c.x39A_b3(),
        c.x39A_b4(), c.x39A_b5(), c.x39A_b6(), c.x39A_b7(),
    ] {
        o.push_str(&format!(" {b}"));
    }
    pf(o, w.globals.x40);
    o.push_str(&format!(" {}", w.debug_last_mode as u32));
}

fn pose(f: &[f32]) -> CmScriptPose {
    CmScriptPose {
        eye: Vec3 { x: f[0], y: f[1], z: f[2] },
        interest: Vec3 { x: f[3], y: f[4], z: f[5] },
        fov: f[6],
        roll: f[7],
    }
}

fn run(w: &mut CameraWorld, op: i64, iarg: i32, fin: &[f32; 17], fout: &mut [f32; 9]) -> i32 {
    let b = |x: bool| x as i32;
    match op {
        0 => b(w.camera_8002f260()),
        1 => { w.camera_set_mode_to_standard(); 0 }
        2 => { w.camera_set_mode_to_fixed(); 0 }
        3 => { w.camera_800300f0(); 0 }
        4 => b(w.camera_8003010c()),
        5 => b(w.camera_80030130()),
        6 => b(w.camera_80030154()),
        7 => b(w.camera_80030178()),
        8 => { w.camera_80030730(fin[0]); 0 }
        9 => {
            w.camera_set_background_color(iarg as u8, (iarg >> 8) as u8, (iarg >> 16) as u8);
            0
        }
        10 => {
            let k = w.camera_get_background_color();
            (k.r as u32 | (k.g as u32) << 8 | (k.b as u32) << 16 | (k.a as u32) << 24) as i32
        }
        11 | 12 => {
            let v = if op == 11 { w.camera_get_transform_position() } else { w.camera_get_transform_interest() };
            fout[0] = v.x;
            fout[1] = v.y;
            fout[2] = v.z;
            0
        }
        13 => { w.camera_80030de4(fin[0], fin[1]); 0 }
        14 => { w.camera_80030df8(); 0 }
        15 => { fout[0] = w.camera_80030e10(); 0 }
        16 => w.camera_80031060(),
        17 => { w.camera_80031074(iarg as u8); 0 }
        18 => w.camera_8003108c(),
        19 => { w.camera_800310a0(iarg as u8); 0 }
        20 => { w.camera_800310e8(); 0 }
        21 => { fout[0] = w.camera_80031144(); 0 }
        22 => { w.camera_800311cc(fin[0]); 0 }
        23 => { w.camera_800311dc(fin[0]); 0 }
        24 => {
            let (n, f) = w.camera_stage_slot_clip_get();
            fout[0] = n;
            fout[1] = f;
            0
        }
        25 => { w.camera_80030a60(iarg); 0 }
        26 => b(w.camera_80030a78()),
        27 => { w.camera_80030a8c(iarg); 0 }
        28 => { w.camera_set_stage_visible(iarg); 0 }
        29 => b(w.camera_80030ac4()),
        30 => { w.camera_80030ae0(iarg); 0 }
        31 => b(w.camera_80030af8()),
        32 => { w.camera_80030b0c(iarg); 0 }
        33 => b(w.camera_80030b24()),
        34 => { w.camera_80030b38(iarg); 0 }
        35 => b(w.camera_80030b50()),
        36 => { w.camera_80030b64(iarg); 0 }
        37 => b(w.camera_80030b7c()),
        38 => { w.camera_80030b90(iarg); 0 }
        39 => b(w.camera_80030ba8()),
        40 => { w.camera_8002fe38(); 0 }
        41 => { w.camera_8002eec8(fin[0]); 0 }
        42 => {
            let mut f = Vec3 { x: fin[0], y: fin[1], z: fin[2] };
            let mut u = Vec3 { x: fin[3], y: fin[4], z: fin[5] };
            let (r, right) = w.camera_8002bc78(&mut f, &mut u);
            fout.copy_from_slice(&[f.x, f.y, f.z, u.x, u.y, u.z, right.x, right.y, right.z]);
            r
        }
        43 => { fout[0] = cm_script_float(iarg); 0 }
        44 => cm_script_bits(fin[0]),
        45 => { fout[0] = cm_script_lerp(fin[0], fin[1], fin[2]); 0 }
        46 => { fout[0] = cm_script_ease(fin[0], iarg); 0 }
        47 => {
            let r = cm_script_blend(&pose(&fin[0..8]), &pose(&fin[8..16]), fin[16]);
            fout[..8].copy_from_slice(&[
                r.eye.x, r.eye.y, r.eye.z, r.interest.x, r.interest.y, r.interest.z, r.fov, r.roll,
            ]);
            0
        }
        _ => -999,
    }
}

/// Token-wise equality where two NaNs of either sign match. Only used for Camera_8002BC78:
/// with a NaN `pause_up` the sign of the propagated NaN differs (1 row; LLVM leaves NaN
/// propagation through fneg/fadd unspecified, and the difference comes from
/// melee_runtime::mtx::vec_cross / melee_core::fp::fmadds). Finite inputs match bit for bit.
fn nan_equal(a: &str, b: &str) -> bool {
    let isnan = |t: &str| {
        t.len() == 8 && u32::from_str_radix(t, 16).map_or(false, |u| f32::from_bits(u).is_nan())
    };
    let (x, y): (Vec<&str>, Vec<&str>) = (a.split(' ').collect(), b.split(' ').collect());
    x.len() == y.len() && x.iter().zip(&y).all(|(p, q)| p == q || (isnan(p) && isnan(q)))
}

#[test]
fn camera_acc_matches_c() {
    let mut lines = REF.lines();
    let mut cases = 0;
    let mut bad = 0;
    while let Some(inl) = lines.next() {
        let want = lines.next().expect("OUT line");
        let mut t = Tok(inl.split_whitespace());
        assert_eq!(t.0.next(), Some("IN"));
        let op = t.i();
        let iarg = t.i() as i32;
        let mut fin = [0f32; 17];
        for f in fin.iter_mut() {
            *f = t.f();
        }
        let mut w = CameraWorld::default();
        load(&mut w, &mut t);
        let mut fout = [0f32; 9];
        let r = run(&mut w, op, iarg, &fin, &mut fout);
        let mut o = format!("OUT {r}");
        for f in fout {
            pf(&mut o, f);
        }
        dump(&w, &mut o);
        cases += 1;
        if o != want && !(op == 42 && nan_equal(&o, want)) {
            bad += 1;
            if bad <= 5 {
                eprintln!("op {op}\n  in   {inl}\n  want {want}\n  got  {o}");
            }
        }
    }
    assert!(cases > 8_000, "only {cases} cases");
    assert_eq!(bad, 0, "{bad} of {cases} cases differ");
}
