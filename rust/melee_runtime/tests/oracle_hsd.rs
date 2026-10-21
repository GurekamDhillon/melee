//! Bit-for-bit check of baselib mtx.c / quatlib.c / spline.c against the C. Vectors:
//! `tests/data/oracle_hsd.txt`, written by `oracle/build_hsd.sh` (the C TUs compiled with the
//! shipping build's contraction + gwtool lowering; generated inputs, no disc data).
use melee_runtime::hsd_mtx as hm;
use melee_runtime::mtx::{Mtx, Quaternion, Vec3};
use melee_runtime::{quatlib, spline};

const DATA: &str = include_str!("data/oracle_hsd.txt");

struct Rd<'a>(std::str::SplitWhitespace<'a>);
impl Rd<'_> {
    fn s(&mut self) -> &str {
        self.0.next().unwrap()
    }
    fn u(&mut self) -> u32 {
        u32::from_str_radix(self.s(), 16).unwrap()
    }
    fn i(&mut self) -> i32 {
        self.s().parse().unwrap()
    }
    fn f(&mut self) -> f32 {
        f32::from_bits(self.u())
    }
    fn m(&mut self) -> Mtx {
        core::array::from_fn(|_| core::array::from_fn(|_| self.f()))
    }
    fn v(&mut self) -> Vec3 {
        Vec3 { x: self.f(), y: self.f(), z: self.f() }
    }
    fn q(&mut self) -> Quaternion {
        Quaternion { x: self.f(), y: self.f(), z: self.f(), w: self.f() }
    }
}

#[derive(Default)]
struct Fl {
    msgs: Vec<String>,
    failed: usize,
    checks: usize,
    per: std::collections::BTreeMap<String, usize>,
}
impl Fl {
    fn eq(&mut self, what: &str, ln: usize, got: Vec<u32>, want: Vec<u32>) {
        self.checks += 1;
        if got != want {
            self.failed += 1;
            *self.per.entry(what.to_string()).or_default() += 1;
            if self.msgs.len() < 40 {
                self.msgs.push(format!("line {} {what}: got {got:08x?} want {want:08x?}", ln + 1));
            }
        }
    }
    fn m(&mut self, what: &str, ln: usize, got: &Mtx, r: &mut Rd) {
        let want = (0..12).map(|_| r.u()).collect();
        self.eq(what, ln, got.iter().flatten().map(|f| f.to_bits()).collect(), want);
    }
    fn v(&mut self, what: &str, ln: usize, g: Vec3, r: &mut Rd) {
        let want = (0..3).map(|_| r.u()).collect();
        self.eq(what, ln, vec![g.x.to_bits(), g.y.to_bits(), g.z.to_bits()], want);
    }
    fn q(&mut self, what: &str, ln: usize, g: Quaternion, r: &mut Rd) {
        let want = (0..4).map(|_| r.u()).collect();
        self.eq(what, ln, vec![g.x.to_bits(), g.y.to_bits(), g.z.to_bits(), g.w.to_bits()], want);
    }
    fn f(&mut self, what: &str, ln: usize, g: f32, r: &mut Rd) {
        let want = vec![r.u()];
        self.eq(what, ln, vec![g.to_bits()], want);
    }
}

#[test]
fn hsd_math_matches_c_build() {
    let mut fl = Fl::default();
    let (mut n_hm, mut n_spl, mut n_arc) = (0, 0, 0);
    for (ln, line) in DATA.lines().enumerate() {
        let mut it = line.split_whitespace();
        let tag = it.next().unwrap();
        let mut r = Rd(it);
        match tag {
            "hm" => {
                n_hm += 1;
                let a = r.m();
                let b = r.m();
                let (s, rot, t, ps) = (r.v(), r.v(), r.v(), r.v());
                let (q, p) = (r.q(), r.q());
                let f = r.f();
                fl.m("MtxInverse", ln, &hm::inverse(&a), &mut r);
                let mut c = a;
                hm::inverse_in_place(&mut c);
                fl.m("MtxInverse aliased", ln, &c, &mut r);
                let ic = hm::inverse_concat(&a, &b);
                fl.m("MtxInverseConcat", ln, &ic, &mut r);
                fl.m("MtxInverseConcat aliased", ln, &ic, &mut r);
                fl.m("MtxInverseTranspose", ln, &hm::inverse_transpose(&a), &mut r);
                fl.v("MtxGetRotation", ln, hm::get_rotation(&a), &mut r);
                fl.v("MtxGetTranslate", ln, hm::get_translate(&a), &mut r);
                fl.v("MtxGetScale", ln, hm::get_scale(&a), &mut r);
                fl.m("MkRotationMtx", ln, &hm::mk_rotation_mtx(&rot), &mut r);
                fl.m("MtxQuat", ln, &hm::mtx_quat(&q), &mut r);
                fl.m("MtxSRT", ln, &hm::srt(&s, &rot, &t, None), &mut r);
                fl.m("MtxSRT ps", ln, &hm::srt(&s, &rot, &t, Some(&ps)), &mut r);
                fl.m("MtxSRTQuat", ln, &hm::srt_quat(&s, &q, &t, None), &mut r);
                fl.m("MtxSRTQuat ps", ln, &hm::srt_quat(&s, &q, &t, Some(&ps)), &mut r);
                fl.m("MtxScaledAdd", ln, &hm::scaled_add(&a, &b, f), &mut r);
                fl.q("MatToQuat", ln, quatlib::mat_to_quat(&a), &mut r);
                fl.v("QuatLib_8037EB28", ln, quatlib::mtx_to_euler(&a), &mut r);
                fl.q("QuatLib_8037EC4C", ln, quatlib::mul(&p, &q), &mut r);
                let mut axis = s;
                if (n_hm - 1) % 11 == 0 {
                    axis = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                }
                let rc = r.i();
                let res = quatlib::from_axis_angle(&axis, rot.x);
                fl.eq("QuatLib_8037ECE0 rc", ln, vec![res.is_none() as u32], vec![(rc == -1) as u32]);
                let seven = Quaternion { x: 7.0, y: 7.0, z: 7.0, w: 7.0 };
                fl.q("QuatLib_8037ECE0", ln, res.unwrap_or(seven), &mut r);
                fl.q("EulerToQuat", ln, quatlib::euler_to_quat(&rot), &mut r);
                fl.q("QuatLib_8037EF28", ln, quatlib::slerp(&p, &q, f * 0.5 + 0.5), &mut r);
            }
            "spl" => {
                n_spl += 1;
                let ty = r.i() as u8;
                let n = r.i() as usize;
                let tension = r.f();
                let total_length = r.f();
                let cv: Vec<Vec3> = (0..28).map(|_| r.v()).collect();
                let seg: Vec<f32> = (0..n).map(|_| r.f()).collect();
                let poly: Vec<[f32; 5]> =
                    (0..16).map(|_| core::array::from_fn(|_| r.f())).collect();
                let sp = spline::Spline {
                    ty,
                    numcv: n as i16,
                    tension,
                    cv: &cv,
                    total_length,
                    seg_length: &seg,
                    seg_poly: &poly,
                };
                let (u, ft, tm, p0, p1, d0, d1) = (r.f(), r.f(), r.f(), r.f(), r.f(), r.f(), r.f());
                fl.f("splGetHelmite", ln, spline::get_helmite(ft, tm, p0, p1, d0, d1), &mut r);
                let ninety9 = Vec3 { x: 99.0, y: 99.0, z: 99.0 };
                fl.v("splGetSplinePoint", ln, spline::get_spline_point(&sp, u).unwrap_or(ninety9), &mut r);
                fl.f("splArcLengthGetParameter", ln, spline::arc_length_get_parameter(&sp, u), &mut r);
                fl.v("splArcLengthPoint", ln, spline::arc_length_point(&sp, u).unwrap_or(ninety9), &mut r);
            }
            "arc" => {
                n_arc += 1;
                let ty = r.i() as u8;
                let total_length = r.f();
                let poly = [core::array::from_fn(|_| r.f())];
                let u = r.f();
                let sp = spline::Spline {
                    ty,
                    numcv: 2,
                    tension: 0.0,
                    cv: &[],
                    total_length,
                    seg_length: &[0.0, 1.0],
                    seg_poly: &poly,
                };
                fl.f("splArcLengthGetParameter (1 seg)", ln, spline::arc_length_get_parameter(&sp, u), &mut r);
            }
            t => panic!("unknown tag {t}"),
        }
        assert!(r.0.next().is_none(), "line {}: trailing fields", ln + 1);
    }
    assert!(n_hm > 1000 && n_spl > 1000 && n_arc > 10000);
    assert!(fl.failed == 0, "{} of {} checks differ {:?}:\n{}", fl.failed, fl.checks, fl.per, fl.msgs.join("\n"));
    eprintln!("{} checks over {} rows, all bit-identical", fl.checks, n_hm + n_spl + n_arc);
}
