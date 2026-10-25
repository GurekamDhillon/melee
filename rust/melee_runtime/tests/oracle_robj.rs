//! C functions covered: HSD_RObjGetGlobalPosition = robj::get_global_position.
//! Bit-for-bit check of the baselib robj.c port (LoadDesc, ResolveRefs, SetConstraintObj,
//! GetByType, SetFlags, GetGlobalPosition and the constraint solver HSD_RObjUpdateAll with its
//! static helpers set_dirup_matrix / resolveCnsDirUp / resolveCnsOrientation / resolveLimits /
//! expEvaluate) against the C. Vectors: `tests/data/oracle_robj.txt`, written by
//! `oracle/build_robj.sh` (robj.c, jobj.c, mtx.c, mtx_pc.c compiled with the shipping build's
//! contraction + gwtool lowering; generated inputs only). See `oracle/driver_robj.c` for the
//! format; the callback, expression functions and HSD_ByteCodeEval stub here mirror it.
use melee_runtime::jobj::{JObjHost, JObjId, JObjTree};
use melee_runtime::mtx::{Quaternion, Vec3};
use melee_runtime::robj::{self, *};

const DATA: &str = include_str!("data/oracle_robj.txt");
const NONE: u32 = 0xFFFF_FFFF;

fn hx(s: &str) -> u32 {
    u32::from_str_radix(s, 16).unwrap()
}
fn fl(s: &str) -> f32 {
    f32::from_bits(hx(s))
}

/// NaNs hash as one canonical NaN (see driver_robj.c `cn`).
fn cn(u: u32) -> u32 {
    if u & 0x7F80_0000 == 0x7F80_0000 && u & 0x007F_FFFF != 0 { 0x7FC0_0000 } else { u }
}

struct Fnv(u32);
impl Fnv {
    /// A float's bits, NaN canonicalised.
    fn f(&mut self, v: f32) {
        self.w(cn(v.to_bits()));
    }
    fn w(&mut self, w: u32) {
        for i in 0..4 {
            self.0 ^= (w >> (8 * i)) & 0xFF;
            self.0 = self.0.wrapping_mul(16777619);
        }
    }
}

struct Host;
impl JObjHost<u32, u32> for Host {
    fn ik_hint_bone_length(&mut self, _: &[u32]) -> Option<f32> {
        None
    }
    fn robj_update_all(&mut self, _: &mut JObjTree<u32, u32>, _: JObjId) {
        panic!("the driver's JObjs carry no RObj");
    }
    fn resolve_ik_joint1(&mut self, _: &mut JObjTree<u32, u32>, _: JObjId) {
        panic!("no JOINT1");
    }
    fn resolve_ik_joint2(&mut self, _: &mut JObjTree<u32, u32>, _: JObjId) {
        panic!("no JOINT2");
    }
    fn unref(&mut self, _: &mut JObjTree<u32, u32>, _: JObjId) {}
}

struct RHost {
    refs: i64,
}
impl RObjHost for RHost {
    fn jobj_ref_this(&mut self, _: JObjId) {
        self.refs += 1;
    }
    fn jobj_unref_this(&mut self, j: Option<JObjId>) {
        if j.is_some() {
            self.refs -= 1;
        }
    }
    fn byte_code_eval(&mut self, code: Option<&[u8]>, a: &[f32], n: u32) -> f32 {
        (if code.is_some() { a[0] } else { -a[1] }) + n as f32
    }
}

fn fn0(a: &[f32]) -> f32 {
    let mut s = 0.0f32;
    for v in &a[..8] {
        s = s * 0.5 + v;
    }
    s
}
fn fn1(a: &[f32]) -> f32 {
    a[0] - a[1] * 0.25
}

fn idx(j: Option<JObjId>) -> u32 {
    j.map_or(NONE, |j| j.0)
}

fn state_hash(t: &JObjTree<u32, u32>, chain: &[HsdRObj], h: &mut Fnv) {
    for o in &t.nodes {
        h.w(o.flags);
        let r = o.rotate;
        for v in [r.x, r.y, r.z, r.w, o.scale.x, o.scale.y, o.scale.z, o.translate.x, o.translate.y, o.translate.z] {
            h.f(v);
        }
        for row in &o.mtx {
            for v in row {
                h.f(*v);
            }
        }
        h.w(o.scl.is_some() as u32);
        for v in o.scl.map_or([0; 3], |s| [cn(s.x.to_bits()), cn(s.y.to_bits()), cn(s.z.to_bits())]) {
            h.w(v);
        }
    }
    for r in chain {
        h.w(r.flags);
        match r.flags & ROBJ_TYPE_MASK {
            REFTYPE_JOBJ => h.w(idx(r.u.jobj)),
            REFTYPE_LIMIT => h.f(r.u.limit),
            REFTYPE_IKHINT => {
                h.f(r.u.ik_hint.bone_length);
                h.f(r.u.ik_hint.rotate_x);
            }
            0 => {
                h.w(r.u.exp.nb_args);
                h.w(r.u.exp.is_bytecode as u32);
                for v in &r.u.exp.rvalue {
                    h.w(v.flags);
                    h.w(idx(v.jobj));
                }
                h.w(NONE);
            }
            _ => {}
        }
    }
}

#[test]
fn robj_matches_c() {
    let mut tree: JObjTree<u32, u32> = JObjTree::new();
    let mut host = Host;
    let mut rh = RHost { refs: 0 };
    let mut g = RObjGlobals::default();
    let mut descs: Vec<HsdRObjDesc> = Vec::new();
    let mut chain: Vec<HsdRObj> = Vec::new();
    let mut nj = 0u32;
    let (mut checks, mut failed) = (0usize, 0usize);
    let mut ops = [0usize; 6];
    let mut cb_calls = 0usize;
    let mut check = |what: &str, line: &str, ok: bool| {
        checks += 1;
        if !ok {
            failed += 1;
            if failed <= 10 {
                eprintln!("mismatch ({what}): {line}");
            }
        }
    };
    for line in DATA.lines() {
        let tk: Vec<&str> = line.split(' ').collect();
        match tk[0] {
            "S" => {
                tree = JObjTree::new();
                descs.clear();
                chain.clear();
                nj = hx(tk[2]);
            }
            "J" => {
                let i = hx(tk[1]);
                let par = hx(tk[2]);
                let f = hx(tk[3]);
                let v: Vec<f32> = (0..10).map(|k| fl(tk[4 + k])).collect();
                let j = tree.alloc();
                assert_eq!(j.0, i);
                let o = tree.get_mut(j);
                o.rotate = Quaternion { x: v[0], y: v[1], z: v[2], w: v[3] };
                o.scale = Vec3 { x: v[4], y: v[5], z: v[6] };
                o.translate = Vec3 { x: v[7], y: v[8], z: v[9] };
                if hx(tk[14]) != 0 {
                    o.scl = Some(Vec3 { x: fl(tk[15]), y: fl(tk[16]), z: fl(tk[17]) });
                }
                if f != 0 {
                    tree.set_flags(Some(j), f);
                }
                if par != NONE {
                    tree.add_child(Some(JObjId(par)), Some(j));
                }
            }
            "D" => {
                let flags = hx(tk[2]);
                let (a, b) = (hx(tk[4]), hx(tk[5]));
                let rv = (b != NONE).then(Vec::new);
                let u = match hx(tk[3]) {
                    0 => RObjDescU::Joint(a),
                    1 => RObjDescU::Limit(f32::from_bits(a)),
                    2 => RObjDescU::Exp(Some(HsdExpDesc {
                        func: match a {
                            0 => Some(fn0 as ExpFunc),
                            1 => Some(fn1 as ExpFunc),
                            _ => None,
                        },
                        rvalue: rv,
                    })),
                    3 => RObjDescU::ByteCode(Some(HsdByteCodeExpDesc {
                        bytecode: (a != 0).then(|| vec![1, 2, 3, 4]),
                        rvalue: rv,
                    })),
                    _ => RObjDescU::IkHint(HsdIKHint {
                        bone_length: f32::from_bits(a),
                        rotate_x: f32::from_bits(b),
                    }),
                };
                descs.push(HsdRObjDesc { flags, u });
            }
            "V" => {
                let e = HsdRvalueList { flags: hx(tk[3]), joint: hx(tk[4]) };
                match &mut descs[hx(tk[1]) as usize].u {
                    RObjDescU::Exp(Some(d)) => d.rvalue.as_mut().unwrap().push(e),
                    RObjDescU::ByteCode(Some(d)) => d.rvalue.as_mut().unwrap().push(e),
                    _ => panic!("V row for a non-expression desc"),
                }
            }
            "L" => {
                chain = robj::load_desc_list(&descs);
                let mut lookup = |k: u32| (k >= 1 && k <= nj).then(|| JObjId(k - 1));
                robj::resolve_refs_all(&mut chain, &descs, &mut lookup, &mut rh);
                let mut h = Fnv(2166136261);
                state_hash(&tree, &chain, &mut h);
                check("load", line, h.0 == hx(tk[1]));
            }
            "C" => {
                robj::set_constraint_obj(chain.get_mut(hx(tk[1]) as usize), JObjId(hx(tk[2])), &mut rh);
                let mut h = Fnv(2166136261);
                state_hash(&tree, &chain, &mut h);
                check("constraint", line, h.0 == hx(tk[3]));
            }
            "O" => {
                let op = hx(tk[2]);
                let (a, b) = (hx(tk[3]), hx(tk[4]));
                let v: Vec<f32> = (0..10).map(|k| fl(tk[6 + k])).collect();
                let want_res = hx(tk[16]);
                let want_h = hx(tk[17]);
                let mut h = Fnv(2166136261);
                let mut res = 0u32;
                ops[op as usize] += 1;
                match op {
                    0 => {
                        let mut cb = |t: &mut JObjTree<u32, u32>, obj: JObjId, ty: u32, val: Option<RObjVal>| {
                            cb_calls += 1;
                            h.w(0xCB00_0000 | ty);
                            h.w(obj.0);
                            match val {
                                None => h.w(NONE),
                                Some(RObjVal::Vec(p)) => {
                                    assert!((0x32..=0x35).contains(&ty));
                                    let col = if ty == 0x35 { 3 } else { (ty - 0x32) as usize };
                                    h.f(p.x);
                                    h.f(p.y);
                                    h.f(p.z);
                                    let m = &mut t.get_mut(obj).mtx;
                                    m[0][col] = p.x;
                                    m[1][col] = p.y;
                                    m[2][col] = p.z;
                                }
                                Some(RObjVal::F(f)) => h.f(f),
                            }
                        };
                        robj::update_all(&mut chain, &mut g, &mut tree, &mut host, &mut rh, JObjId(a), &mut cb);
                    }
                    1 => {
                        let (n, p) = robj::get_global_position(&chain, a as i32, &mut tree, &mut host);
                        res = n as u32;
                        let p = p.unwrap_or_default();
                        h.f(p.x);
                        h.f(p.y);
                        h.f(p.z);
                    }
                    2 => res = robj::get_by_type(&chain, a, b).map_or(NONE, |i| i as u32),
                    3 => {
                        let o = tree.get_mut(JObjId(a));
                        o.rotate = Quaternion { x: v[0], y: v[1], z: v[2], w: v[3] };
                        o.scale = Vec3 { x: v[4], y: v[5], z: v[6] };
                        o.translate = Vec3 { x: v[7], y: v[8], z: v[9] };
                        tree.set_mtx_dirty(Some(JObjId(a)));
                    }
                    4 => robj::set_flags(chain.get_mut(a as usize), b),
                    5 => tree.setup_matrix(&mut host, Some(JObjId(a))),
                    _ => unreachable!(),
                }
                state_hash(&tree, &chain, &mut h);
                check("op", line, res == want_res && h.0 == want_h);
            }
            _ => panic!("bad row {line}"),
        }
    }
    eprintln!("robj oracle: {checks} checks, {failed} failed, ops {ops:?}, {cb_calls} callback calls");
    assert_eq!(failed, 0);
    assert!(checks > 15000);
}
