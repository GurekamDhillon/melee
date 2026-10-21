//! Bit-for-bit check of the logic half of baselib cobj.c against the C. Vectors:
//! `tests/data/oracle_cobj.txt`, written by `oracle/build_cobj.sh` (cobj.c through
//! `helper_cobj.c`, over the real wobj.c / jobj.c / mtx.c / mtx_pc.c / mtx44.c, compiled with the
//! shipping build's contraction + gwtool lowering; generated inputs only). Row format: see
//! `oracle/helper_cobj.c`.
use melee_runtime::cobj::{self, HsdCObj, RectF32, RectS16, Scissor};
use melee_runtime::jobj::{JObjHost, JObjId, JObjTree};
use melee_runtime::mtx::{Mtx44, Vec3};
use melee_runtime::wobj::HsdWObj;

const DATA: &str = include_str!("data/oracle_cobj.txt");

fn hx(s: &str) -> u32 {
    u32::from_str_radix(s, 16).unwrap()
}
fn fl(s: &str) -> f32 {
    f32::from_bits(hx(s))
}
fn fnv(mut h: u32, w: u32) -> u32 {
    for i in 0..4 {
        h ^= (w >> (8 * i)) & 0xFF;
        h = h.wrapping_mul(16777619);
    }
    h
}

struct Host;
impl JObjHost<u32, u32> for Host {
    fn ik_hint_bone_length(&mut self, _: &[u32]) -> Option<f32> {
        None
    }
    fn robj_update_all(&mut self, _: &mut JObjTree<u32, u32>, _: JObjId) {}
    fn resolve_ik_joint1(&mut self, _: &mut JObjTree<u32, u32>, _: JObjId) {
        unreachable!()
    }
    fn resolve_ik_joint2(&mut self, _: &mut JObjTree<u32, u32>, _: JObjId) {
        unreachable!()
    }
    fn unref(&mut self, _: &mut JObjTree<u32, u32>, _: JObjId) {}
}

type C = HsdCObj<u32>;

fn state(c: &C) -> Vec<u32> {
    let mut v = vec![c.flags];
    v.extend([c.viewport.xmin, c.viewport.xmax, c.viewport.ymin, c.viewport.ymax].map(b));
    v.extend([c.scissor.left, c.scissor.right, c.scissor.top, c.scissor.bottom].map(u32::from));
    for w in [&c.eyepos, &c.interest] {
        match w {
            Some(w) => v.extend([1, w.flags, b(w.pos.x), b(w.pos.y), b(w.pos.z)]),
            None => v.extend([0; 5]),
        }
    }
    v.extend([c.u.x, c.u.y, c.u.z, c.near, c.far].map(b));
    v.extend(c.projection_param.map(b));
    v.push(c.projection_type as u32);
    v.extend(c.view_mtx.iter().flatten().map(|f| b(*f)));
    v.push(c.proj_mtx.is_some() as u32);
    match &c.proj_mtx {
        Some(m) => v.extend(m.iter().flatten().map(|f| b(*f))),
        None => v.extend([0; 12]),
    }
    v.push(c.view_authored_aspect.to_bits());
    v.push(c.view_auto as u32);
    v
}

fn from_state(t: &[&str]) -> C {
    let w = |i: usize| hx(t[i]);
    let f = |i: usize| fl(t[i]);
    let wobj = |i: usize| {
        (w(i) != 0).then(|| {
            let mut o = HsdWObj::<u32>::alloc();
            o.flags = w(i + 1);
            o.pos = Vec3 { x: f(i + 2), y: f(i + 3), z: f(i + 4) };
            o
        })
    };
    let mut c = C::default();
    c.flags = w(0);
    c.viewport = RectF32 { xmin: f(1), xmax: f(2), ymin: f(3), ymax: f(4) };
    c.scissor = Scissor { left: w(5) as u16, right: w(6) as u16, top: w(7) as u16, bottom: w(8) as u16 };
    c.eyepos = wobj(9);
    c.interest = wobj(14);
    c.u = Vec3 { x: f(19), y: f(20), z: f(21) };
    c.near = f(22);
    c.far = f(23);
    c.projection_param = [f(24), f(25), f(26), f(27)];
    c.projection_type = w(28) as u8;
    for i in 0..12 {
        c.view_mtx[i / 4][i % 4] = f(29 + i);
    }
    if w(41) != 0 {
        let mut m = [[0.0; 4]; 3];
        for i in 0..12 {
            m[i / 4][i % 4] = f(42 + i);
        }
        c.proj_mtx = Some(m);
    }
    c.view_authored_aspect = f(54);
    c.view_auto = w(55) as u8;
    c
}

fn vbits(v: &Vec3) -> [u32; 3] {
    [b(v.x), b(v.y), b(v.z)]
}
/// A float's bits with NaN canonical (the C writes every NaN as 7fc00000).
fn b(f: f32) -> u32 {
    if f.is_nan() {
        0x7FC0_0000
    } else {
        f.to_bits()
    }
}

#[test]
fn oracle_cobj() {
    let mut tree: JObjTree<u32, u32> = JObjTree::new();
    let mut host = Host;
    let mut c = C::default();
    let (mut rows, mut failed) = (0usize, 0usize);
    let mut per_op = [0usize; 53];
    for (ln, line) in DATA.lines().enumerate() {
        let t: Vec<&str> = line.split_whitespace().collect();
        if t[0] == "S" {
            c = from_state(&t[1..]);
            assert_eq!(state(&c), t[1..].iter().map(|s| hx(s)).collect::<Vec<_>>(), "line {}", ln + 1);
            continue;
        }
        let bar1 = t.iter().position(|s| *s == "|").unwrap();
        let bar2 = bar1 + 1 + t[bar1 + 1..].iter().position(|s| *s == "|").unwrap();
        let (op, isnull, scale) = (hx(t[1]) as usize, hx(t[2]) != 0, fl(t[3]));
        let (a0, a1, a2, a3, w0) = (fl(t[4]), fl(t[5]), fl(t[6]), fl(t[7]), hx(t[8]));
        let v = Vec3 { x: fl(t[9]), y: fl(t[10]), z: fl(t[11]) };
        let want: Vec<u32> = t[bar1 + 1..bar2].iter().map(|s| hx(s)).collect();
        let want_hash = hx(t[bar2 + 1]);
        per_op[op] += 1;
        let mut got: Vec<u32> = Vec::new();
        {
            let cn: Option<&mut C> = if isnull { None } else { Some(&mut c) };
            let (tr, h) = (&mut tree, &mut host);
            match op {
                0 => cobj::set_mtx_dirty(cn.unwrap()),
                1 => got.push(cobj::mtx_is_dirty(cn.unwrap()) as u32),
                2 => got.push(cobj::get_flags(cn.unwrap())),
                3 => cobj::set_flags(cn, w0),
                4 => cobj::clear_flags(cn, w0),
                5 => got.extend(vbits(&cobj::get_interest(cn.unwrap(), tr, h))),
                6 => cobj::set_interest(cn.unwrap(), &v),
                7 => got.extend(vbits(&cobj::get_eye_position(cn.unwrap(), tr, h))),
                8 => cobj::set_eye_position(cn.unwrap(), &v),
                9 => {
                    let (r, e) = cobj::get_eye_vector(cn, tr, h);
                    got.push(r as u32);
                    got.extend(vbits(&e));
                }
                10 => got.push(b(cobj::get_eye_distance(cn, tr, h))),
                11 => {
                    let (r, e) = cobj::get_up_vector(cn, tr, h);
                    got.push(r as u32);
                    got.extend(vbits(&e));
                }
                12 => cobj::set_up_vector(cn, Some(&v), tr, h),
                13 => {
                    let (r, e) = cobj::get_left_vector(cn, tr, h);
                    got.push(r as u32);
                    got.extend(vbits(&e));
                }
                14 => {
                    let roll = if w0 & 3 != 0 { a0 * 0.05 } else { cn.as_ref().map_or(0.0, |c| c.u.x) };
                    cobj::set_roll(cn, roll, tr, h)
                }
                15 => cobj::setup_viewing_mtx(cn.unwrap(), tr, h),
                16 => got.extend(cobj::get_viewing_mtx(cn.unwrap(), tr, h).iter().flatten().map(|f| b(*f))),
                17 => got.push(cobj::get_inv_viewing_mtx_ptr(cn.unwrap(), tr, h).is_some() as u32),
                18 => got.push(cobj::get_inv_viewing_mtx_ptr_direct(cn.unwrap()).is_some() as u32),
                19 => got.push(b(cobj::get_fov(cn.as_deref()))),
                20 => cobj::set_fov(cn, a0),
                21 => got.push(b(cobj::get_aspect(cn, scale))),
                22 => cobj::set_aspect(cn, a0 * 0.02, scale),
                23 => got.push(b(cobj::get_authored_aspect(cn, scale))),
                24 => got.push(b(cobj::get_top(cn.as_deref()))),
                25 => cobj::set_top(cn, a0),
                26 => got.push(b(cobj::get_bottom(cn.as_deref()))),
                27 => cobj::set_bottom(cn, a0),
                28 => got.push(b(cobj::get_left(cn, scale))),
                29 => cobj::set_left(cn, a0),
                30 => got.push(b(cobj::get_right(cn, scale))),
                31 => cobj::set_right(cn, a0),
                32 => got.push(b(cobj::get_near(cn.as_deref()))),
                33 => cobj::set_near(cn, a0),
                34 => got.push(b(cobj::get_far(cn.as_deref()))),
                35 => cobj::set_far(cn, a0),
                36 => {
                    let mut s = Scissor { left: 7, right: 7, top: 7, bottom: 7 };
                    cobj::get_scissor(cn.as_deref(), &mut s);
                    got.extend([s.left, s.right, s.top, s.bottom].map(u32::from));
                }
                37 => cobj::set_scissor(
                    cn,
                    &Scissor { left: w0 as u16, right: (w0 >> 16) as u16, top: (w0 >> 3) as u16, bottom: (w0 >> 9) as u16 },
                ),
                38 => cobj::set_scissor_x4(cn, w0 as u16, (w0 >> 16) as u16, (w0 >> 3) as u16, (w0 >> 9) as u16),
                39 => {
                    let mut r = RectF32 { xmin: 3.0, xmax: 3.0, ymin: 3.0, ymax: 3.0 };
                    cobj::get_viewportf(cn.as_deref(), &mut r);
                    got.extend([r.xmin, r.xmax, r.ymin, r.ymax].map(b));
                }
                40 => cobj::set_viewport(
                    cn,
                    &RectS16 { xmin: w0 as i16, xmax: (w0 >> 16) as i16, ymin: (w0 >> 3) as i16, ymax: (w0 >> 9) as i16 },
                ),
                41 => cobj::set_viewportf(cn, &RectF32 { xmin: a0, xmax: a1, ymin: a2, ymax: a3 }),
                42 => cobj::set_viewportf_x4(cn, a0, a1, a2, a3),
                43 => got.push(cobj::get_projection_type(cn, scale) as u32),
                44 => cobj::set_projection_type(cn, w0 % 5 + (w0 & 0x300)),
                45 => cobj::set_perspective(cn, a0, a1 * 0.02, scale),
                46 => cobj::set_frustum(cn, a0, a1, a2, a3),
                47 => cobj::set_ortho(cn, a0, a1, a2, a3),
                48 => {
                    let (tp, bt) = cobj::get_perspective(cn, scale).unwrap_or((5.0, 6.0));
                    got.extend([b(tp), b(bt)]);
                }
                49 => got.extend(cobj::get_ortho(cn.as_deref()).unwrap_or([5.0, 6.0, 7.0, 8.0]).map(b)),
                50 => {
                    let mut m: Mtx44 = [[9.0; 4]; 4];
                    got.push(cobj::make_projection_mtx(cn.unwrap(), &mut m, scale));
                    got.extend(m.iter().flatten().map(|f| b(*f)));
                }
                51 => {
                    let c = cn.unwrap();
                    if let (Some(e), true) = (c.eyepos.as_mut(), w0 & 1 != 0) {
                        e.flags |= 2;
                        e.pos = Vec3 { x: v.y, y: v.z, z: v.x };
                    }
                    if let (Some(i), true) = (c.interest.as_mut(), w0 & 2 != 0) {
                        i.flags |= 2;
                        i.pos = v;
                    }
                }
                52 => {
                    let c = cn.unwrap();
                    c.eyepos.as_mut().unwrap().pos = Vec3::default();
                    c.interest.as_mut().unwrap().pos = v;
                    let (r, up) = cobj::roll2upvec(c, a0 * 0.05, tr, h);
                    got.push(r as u32);
                    got.extend(vbits(&up.unwrap_or(Vec3 { x: 5.0, y: 5.0, z: 5.0 })));
                }
                _ => unreachable!(),
            }
        }
        rows += 1;
        let h = state(&c).into_iter().fold(2166136261u32, fnv);
        if got != want || h != want_hash {
            failed += 1;
            if failed <= 10 {
                eprintln!("line {}: op {op}: got {:08x?} want {:08x?}, state {:08x} want {:08x}", ln + 1, got, want, h, want_hash);
            }
            // Resync to keep later rows meaningful is not possible (state is hashed); stop here.
            if failed >= 50 {
                break;
            }
        }
    }
    eprintln!("oracle_cobj: {rows} op rows, {failed} failed; per op {per_op:?}");
    assert_eq!(failed, 0);
    assert!(rows >= 30000);
}
