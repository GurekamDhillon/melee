//! Bit-for-bit comparison against the C build's arithmetic. `tests/data/oracle_vectors.txt` is
//! produced by `oracle/build.sh` from the C sources (pc/gameworld/{gekko_fp,mtx_pc}.c, mtx44.c,
//! baselib/random.c, lb/lbtrigf.c, math_ppc.h), compiled with the real build's contraction + gwtool lowering.
use melee_runtime::gekko::*;
use melee_runtime::lbtrigf;
use melee_runtime::mtx::{self, Mtx, Quaternion, Vec3};
use melee_runtime::random::HsdRand;

const DATA: &str = include_str!("data/oracle_vectors.txt");

struct Rd<'a>(std::str::SplitWhitespace<'a>);
impl Rd<'_> {
    fn u(&mut self) -> u32 {
        u32::from_str_radix(self.0.next().unwrap(), 16).unwrap()
    }
    fn f(&mut self) -> f32 {
        f32::from_bits(self.u())
    }
    fn d(&mut self) -> u64 {
        u64::from_str_radix(self.0.next().unwrap(), 16).unwrap()
    }
    fn m(&mut self) -> Mtx {
        let mut m = [[0.0; 4]; 3];
        for r in m.iter_mut() {
            for c in r.iter_mut() {
                *c = self.f();
            }
        }
        m
    }
    fn v(&mut self) -> Vec3 {
        Vec3 { x: self.f(), y: self.f(), z: self.f() }
    }
}

fn bits(m: &[[f32; 4]]) -> Vec<u32> {
    m.iter().flatten().map(|f| f.to_bits()).collect()
}

#[derive(Default)]
struct Fails {
    msgs: Vec<String>,
    failed: usize,
    checks: usize,
}
impl Fails {
    fn eq<T: PartialEq + std::fmt::Debug>(&mut self, what: &str, line: usize, got: T, want: T) {
        self.checks += 1;
        if got != want {
            self.failed += 1;
            if self.msgs.len() < 40 {
                self.msgs.push(format!("line {} {what}: got {got:x?} want {want:x?}", line + 1));
            }
        }
    }
    fn mtx(&mut self, what: &str, line: usize, got: &[[f32; 4]], r: &mut Rd, rows: usize) {
        let want: Vec<u32> = (0..rows * 4).map(|_| r.u()).collect();
        self.eq(what, line, bits(got), want);
    }
    fn vec(&mut self, what: &str, line: usize, got: Vec3, r: &mut Rd) {
        let want = [r.u(), r.u(), r.u()];
        self.eq(what, line, [got.x.to_bits(), got.y.to_bits(), got.z.to_bits()], want);
    }
}

#[test]
fn matches_c_build() {
    let mut fl = Fails::default();
    let mut rng = HsdRand::default();
    rng.set_logic_seed(0x12345678);
    let (mut m3_i, mut rand_i) = (0usize, 0i32);
    for (ln, line) in DATA.lines().enumerate() {
        let mut it = line.split_whitespace();
        let tag = it.next().unwrap();
        let mut r = Rd(it);
        match tag {
            "frsqrte" => {
                let x = f64::from_bits(r.d());
                let w = r.d();
                fl.eq("frsqrte", ln, frsqrte(x).to_bits(), w);
            }
            "fres" => {
                let x = f64::from_bits(r.d());
                let w = r.d();
                fl.eq("fres", ln, fres(x).to_bits(), w);
            }
            "trig" => {
                let x = r.f();
                fl.eq("sinf", ln, sinf(x).to_bits(), r.u());
                fl.eq("cosf", ln, cosf(x).to_bits(), r.u());
                fl.eq("tanf", ln, tanf(x).to_bits(), r.u());
            }
            "atan" => {
                let (x, y, u) = (r.f(), r.f(), r.f());
                fl.eq("atanf", ln, lbtrigf::atanf(x).to_bits(), r.u());
                fl.eq("atan2f", ln, lbtrigf::atan2f(y, x).to_bits(), r.u());
                fl.eq("acosf", ln, lbtrigf::acosf(u).to_bits(), r.u());
                fl.eq("asinf", ln, lbtrigf::asinf(u).to_bits(), r.u());
            }
            "sqrt" => {
                let x = r.f();
                fl.eq("sqrtf", ln, sqrtf(x).to_bits(), r.u());
                fl.eq("sqrtf_accurate", ln, sqrtf_accurate(x).to_bits(), r.u());
            }
            "m3" => {
                let a = r.m();
                let b = r.m();
                let v = r.v();
                let w = r.v();
                let s = r.f();
                let c = mtx::concat(&a, &b);
                fl.mtx("concat", ln, &c, &mut r, 3);
                fl.mtx("concat(alias)", ln, &c, &mut r, 3);
                fl.mtx("transpose", ln, &mtx::transpose(&a), &mut r, 3);
                let ok: u32 = r.0.next().unwrap().parse().unwrap();
                let inv = mtx::inverse(&a);
                fl.eq("inverse ok", ln, inv.is_some() as u32, ok);
                fl.mtx("inverse", ln, &inv.unwrap_or([[0.0; 4]; 3]), &mut r, 3);
                fl.vec("multvec", ln, mtx::mult_vec(&a, &v), &mut r);
                fl.vec("multvecsr", ln, mtx::mult_vec_sr(&a, &v), &mut r);
                fl.vec("add", ln, mtx::vec_add(&v, &w), &mut r);
                fl.vec("sub", ln, mtx::vec_sub(&v, &w), &mut r);
                fl.vec("scale", ln, mtx::vec_scale(&v, s), &mut r);
                fl.vec("normalize", ln, mtx::vec_normalize(&w), &mut r);
                fl.eq("mag", ln, mtx::vec_mag(&w).to_bits(), r.u());
                fl.eq("dot", ln, mtx::vec_dot(&v, &w).to_bits(), r.u());
                fl.vec("cross", ln, mtx::vec_cross(&v, &w), &mut r);
                fl.mtx("rotaxis", ln, &mtx::rot_axis_rad(&w, s), &mut r, 3);
                let mut c = [[0.0; 4]; 3];
                mtx::rot_rad(&mut c, b"xYzq"[m3_i % 4], s);
                m3_i += 1;
                fl.mtx("rotrad", ln, &c, &mut r, 3);
                let qw = r.f();
                let q = Quaternion { x: w.x, y: w.y, z: w.z, w: qw };
                fl.mtx("quat", ln, &mtx::quat(&q), &mut r, 3);
                let z = r.v();
                fl.mtx("lookat", ln, &mtx::look_at(&w, &v, &z), &mut r, 3);
                fl.mtx("trans", ln, &mtx::trans(v.x, v.y, s), &mut r, 3);
                fl.mtx("scale", ln, &mtx::scale(v.x, v.y, s), &mut r, 3);
                let (t, bb, l, rr, n, f, fov, asp) =
                    (r.f(), r.f(), r.f(), r.f(), r.f(), r.f(), r.f(), r.f());
                let lf = mtx::light_frustum(t, bb, l, rr, n, s, v.x, v.y, w.x);
                fl.mtx("lightfrustum", ln, &lf, &mut r, 3);
                let lp = mtx::light_perspective(fov, asp, s, v.x, v.y, w.x);
                fl.mtx("lightpersp", ln, &lp, &mut r, 3);
                let lo = mtx::light_ortho(t, bb, l, rr, s, v.x, v.y, w.x);
                fl.mtx("lightortho", ln, &lo, &mut r, 3);
                fl.mtx("frustum", ln, &mtx::frustum(t, bb, l, rr, n, f), &mut r, 4);
                fl.mtx("perspective", ln, &mtx::perspective(fov, asp, n, f), &mut r, 4);
                fl.mtx("ortho", ln, &mtx::ortho(t, bb, l, rr, n, f), &mut r, 4);
            }
            "rand" => {
                let i = rand_i;
                rand_i += 1;
                let a = rng.rand() as u32;
                let f = rng.randf().to_bits();
                let b = rng.randi((i * 37) % 1000 - 100) as u32;
                fl.eq("rand", ln, [a, f, b, rng.logic_seed()], [r.u(), r.u(), r.u(), r.u()]);
                if i % 7 == 0 {
                    rng.render_begin();
                    rng.rand();
                    rng.render_begin();
                    rng.randf();
                    rng.render_end();
                    rng.randi(9);
                    rng.render_end();
                }
            }
            t => panic!("unknown tag {t}"),
        }
    }
    assert!(
        fl.failed == 0,
        "{} of {} checks failed, first:\n{}",
        fl.failed,
        fl.checks,
        fl.msgs.join("\n")
    );
    assert!(fl.checks > 10000);
}
