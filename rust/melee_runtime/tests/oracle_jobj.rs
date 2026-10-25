//! C functions covered: HSD_JObjMakeMatrix = JObjTree::make_matrix (op 14).
//! Bit-for-bit check of the baselib jobj.c port (tree, flags, dirty propagation, matrix build)
//! against the C. Vectors: `tests/data/oracle_jobj.txt`, written by `oracle/build_jobj.sh` (jobj.c,
//! mtx.c, mtx_pc.c compiled with the shipping build's contraction + gwtool lowering; generated
//! inputs only). Each row is one operation, its result and an FNV-1a hash of the full state after
//! it (see `oracle/driver_jobj.c`); the RObj stubs here mirror `oracle/helper_jobj.c`.
use melee_runtime::aobj::HsdAObj;
use melee_runtime::jobj::{JObjHost, JObjId, JObjTree, JointSource};
use melee_runtime::mtx::Vec3;

const DATA: &str = include_str!("data/oracle_jobj.txt");
const NJ: usize = 24;
const NONE: u32 = 0xFFFF_FFFF;

fn hx(s: &str) -> u32 {
    u32::from_str_radix(s, 16).unwrap()
}
fn fl(s: &str) -> f32 {
    f32::from_bits(hx(s))
}

struct Fnv(u32);
impl Fnv {
    fn w(&mut self, w: u32) {
        for i in 0..4 {
            self.0 ^= (w >> (8 * i)) & 0xFF;
            self.0 = self.0.wrapping_mul(16777619);
        }
    }
}

struct Joints {
    v: Vec<[f32; 9]>,
    child: Vec<Option<usize>>,
    next: Vec<Option<usize>>,
}
impl JointSource for Joints {
    type Id = usize;
    fn rotation(&self, j: usize) -> Vec3 {
        Vec3 { x: self.v[j][0], y: self.v[j][1], z: self.v[j][2] }
    }
    fn scale(&self, j: usize) -> Vec3 {
        Vec3 { x: self.v[j][3], y: self.v[j][4], z: self.v[j][5] }
    }
    fn position(&self, j: usize) -> Vec3 {
        Vec3 { x: self.v[j][6], y: self.v[j][7], z: self.v[j][8] }
    }
    fn child(&self, j: usize) -> Option<usize> {
        self.child[j]
    }
    fn next(&self, j: usize) -> Option<usize> {
        self.next[j]
    }
}

struct Host {
    h: Fnv,
    robj_flags: Vec<u32>,
    robj_bone: Vec<f32>,
    updates: usize,
    effector_hits: usize,
}
impl JObjHost<u32, u32> for Host {
    fn ik_hint_bone_length(&mut self, robj: &[u32]) -> Option<f32> {
        let r = robj.iter().find(|&&r| self.robj_flags[r as usize] & 0x4000_0000 != 0)?;
        self.effector_hits += 1;
        Some(self.robj_bone[*r as usize])
    }
    fn robj_update_all(&mut self, _tree: &mut JObjTree<u32, u32>, jobj: JObjId) {
        self.updates += 1;
        self.h.w(0xABCD_0000);
        self.h.w(jobj.0);
    }
    fn resolve_ik_joint1(&mut self, _: &mut JObjTree<u32, u32>, _: JObjId) {
        panic!("driver never builds JOINT1");
    }
    fn resolve_ik_joint2(&mut self, _: &mut JObjTree<u32, u32>, _: JObjId) {
        panic!("driver never builds JOINT2");
    }
    fn unref(&mut self, _: &mut JObjTree<u32, u32>, _: JObjId) {}
}

fn idx(j: Option<JObjId>) -> u32 {
    j.map_or(NONE, |j| j.0)
}
fn opt(a: u32) -> Option<JObjId> {
    (a != NONE).then_some(JObjId(a))
}
fn optu(a: u32) -> Option<u32> {
    (a != NONE).then_some(a)
}

fn state_hash(t: &JObjTree<u32, u32>, h: &mut Fnv) {
    for o in &t.nodes {
        h.w(idx(o.next));
        h.w(idx(o.parent));
        h.w(idx(o.child));
        h.w(o.flags);
        let r = o.rotate;
        for v in [r.x, r.y, r.z, r.w, o.scale.x, o.scale.y, o.scale.z, o.translate.x, o.translate.y, o.translate.z] {
            h.w(v.to_bits());
        }
        for row in &o.mtx {
            for v in row {
                h.w(v.to_bits());
            }
        }
        h.w(o.scl.is_some() as u32);
        let s = o.scl.unwrap_or_default();
        for v in [s.x, s.y, s.z] {
            h.w(if o.scl.is_some() { v.to_bits() } else { 0 });
        }
        for r in &o.robj {
            h.w(*r);
        }
        h.w(NONE);
        for d in &o.dobj {
            h.w(*d);
        }
        h.w(NONE);
    }
}

#[test]
fn jobj_matches_c() {
    let mut joints = Joints { v: vec![[0.0; 9]; 48], child: vec![None; 48], next: vec![None; 48] };
    let mut host = Host { h: Fnv(0), robj_flags: vec![0; 16], robj_bone: vec![0.0; 16], updates: 0, effector_hits: 0 };
    let mut t: JObjTree<u32, u32> = JObjTree::new();
    for _ in 0..NJ {
        t.alloc();
    }
    let (mut rows, mut failed) = (0usize, 0usize);
    let mut counts = [0usize; 26];
    for line in DATA.lines() {
        let tk: Vec<&str> = line.split(' ').collect();
        match tk[0] {
            "J" => {
                let k = hx(tk[1]) as usize;
                joints.child[k] = optu(hx(tk[2])).map(|x| x as usize);
                joints.next[k] = optu(hx(tk[3])).map(|x| x as usize);
                for i in 0..9 {
                    joints.v[k][i] = fl(tk[4 + i]);
                }
            }
            "R" => {
                let k = hx(tk[1]) as usize;
                host.robj_flags[k] = hx(tk[2]);
                host.robj_bone[k] = fl(tk[3]);
            }
            "O" => {
                let op = hx(tk[2]);
                let (a, b, c) = (hx(tk[3]), hx(tk[4]), hx(tk[5]));
                let v: Vec<f32> = if tk.len() > 8 { (0..10).map(|i| fl(tk[6 + i])).collect() } else { vec![0.0; 10] };
                let (want_res, want_h) = (hx(tk[tk.len() - 2]), hx(tk[tk.len() - 1]));
                host.h = Fnv(2166136261);
                let ja = opt(a);
                let mut res = 0u32;
                counts[op as usize] += 1;
                match op {
                    0 => t.add_child(ja, opt(b)),
                    1 => res = idx(t.reparent(ja, opt(b))),
                    2 => t.add_next(ja, opt(b)),
                    3 => res = idx(t.get_prev(ja)),
                    4 => res = idx(t.remove(&mut host, ja)),
                    5 => t.remove_all(&mut host, ja),
                    6 => t.set_flags(ja, b),
                    7 => t.clear_flags(ja, b),
                    8 => t.set_flags_all(ja, b),
                    9 => t.clear_flags_all(ja, b),
                    10 => t.set_mtx_dirty(ja),
                    11 => t.check_depend(ja),
                    12 | 13 => t.setup_matrix(&mut host, ja),
                    14 => t.make_matrix(&mut host, ja.unwrap()),
                    15..=17 => {
                        let o = t.get_mut(ja.unwrap());
                        o.rotate.x = v[0];
                        o.rotate.y = v[1];
                        o.rotate.z = v[2];
                        o.rotate.w = v[3];
                        o.scale = Vec3 { x: v[4], y: v[5], z: v[6] };
                        o.translate = Vec3 { x: v[7], y: v[8], z: v[9] };
                    }
                    18 => t.reset_rst_one(ja, &joints, Some(b as usize)),
                    19 => t.reset_rst(ja, &joints, Some(b as usize)),
                    20 => {
                        let h = &mut host.h;
                        t.walk_tree(ja, &mut |j, ty| {
                            h.w(0xC0DE_0000);
                            h.w(j.0);
                            h.w(ty);
                        });
                    }
                    21 => t.prepend_robj(ja, optu(b)),
                    22 => t.delete_robj(ja, optu(b)),
                    23 => {
                        t.add_dobj(ja, optu(b));
                        if c == 1 {
                            res = t.get_dobj(ja).unwrap_or(NONE);
                        }
                    }
                    24 => {
                        let mut ao = HsdAObj::alloc();
                        ao.hsd_obj = opt(b);
                        t.get_mut(ja.unwrap()).aobj = Some(ao);
                    }
                    _ => {
                        res = t.get_flags(ja) ^ ja.map_or(0, |j| (t.mtx_is_dirty(j) as u32) << 31);
                        if c == 1 {
                            t.recalc_parent_trsp_bits(ja);
                        }
                    }
                }
                state_hash(&t, &mut host.h);
                rows += 1;
                if res != want_res || host.h.0 != want_h {
                    if failed < 10 {
                        eprintln!("row {line}: got res {res:x} hash {:08x}", host.h.0);
                    }
                    failed += 1;
                }
            }
            _ => panic!("bad row {line}"),
        }
    }
    eprintln!(
        "jobj oracle: {rows} rows, {failed} failed; op counts {counts:?}; robj updates {}, effector hits {}",
        host.updates, host.effector_hits
    );
    assert!(rows > 20000);
    assert_eq!(failed, 0);
}
