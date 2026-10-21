//! Bit-for-bit check of jobj.c's IK solvers (resolveIKJoint1 / resolveIKJoint2 through
//! HSD_JObjSetupMatrix), the effector branch, HSD_RObjUpdateAll with the real JObjUpdateFunc,
//! and JObjUpdateFunc itself, against the C. Vectors: `tests/data/oracle_ik.txt`, written by
//! `oracle/build_ik.sh` (jobj.c, robj.c, mtx.c, mtx_pc.c with the shipping build's contraction +
//! gwtool lowering; generated inputs only). Format: `oracle/driver_ik.c`.
use melee_runtime::jobj::{JObjId, JObjTree};
use melee_runtime::jobj_rt::*;
use melee_runtime::mtx::{Quaternion, Vec3};
use melee_runtime::robj::{self, *};
use std::cell::RefCell;
use std::rc::Rc;

const DATA: &str = include_str!("data/oracle_ik.txt");
const NONE: u32 = 0xFFFF_FFFF;

fn hx(s: &str) -> u32 {
    u32::from_str_radix(s, 16).unwrap()
}
fn fl(s: &str) -> f32 {
    f32::from_bits(hx(s))
}
fn cn(u: u32) -> u32 {
    if u & 0x7F80_0000 == 0x7F80_0000 && u & 0x007F_FFFF != 0 { 0x7FC0_0000 } else { u }
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

struct Ext;
impl RObjHost for Ext {
    fn jobj_ref_this(&mut self, _: JObjId) {}
    fn jobj_unref_this(&mut self, _: Option<JObjId>) {}
    fn byte_code_eval(&mut self, _: Option<&[u8]>, _: &[f32], _: u32) -> f32 {
        panic!("no bytecode in this oracle")
    }
}
impl JObjExt for Ext {
    type D = u32;
    type MatAnim = ();
    type ShapeAnim = ();
    fn dobj_remove_anim_all_by_flags(&mut self, _: &[u32], _: u32) {}
    fn dobj_req_anim_all_by_flags(&mut self, _: &[u32], _: f32, _: u32) {}
    fn dobj_add_anim_all(&mut self, _: &[u32], _: Option<&()>, _: Option<&()>) {}
    fn dobj_anim_all(&mut self, _: &[u32]) {}
    fn spline_point(&mut self, _: JObjId, _: f32) -> Vec3 {
        panic!("no PATH in this oracle")
    }
    fn aobj_resolve_obj(&mut self, _: u32) -> Option<JObjId> {
        None
    }
    fn jobj_ref(&mut self, _: Option<JObjId>) {}
    fn jobj_unref(&mut self, _: Option<JObjId>) {}
}

fn state_hash(t: &JObjTree<u32, u32>, rt: &JObjRt<Ext>, h: &mut Fnv) {
    for (i, o) in t.nodes.iter().enumerate() {
        h.w(o.flags);
        let r = o.rotate;
        let mut w: Vec<u32> = [r.x, r.y, r.z, r.w, o.scale.x, o.scale.y, o.scale.z, o.translate.x, o.translate.y, o.translate.z]
            .iter()
            .map(|v| v.to_bits())
            .collect();
        for row in &o.mtx {
            w.extend(row.iter().map(|v| v.to_bits()));
        }
        w.push(o.scl.is_some() as u32);
        let s = o.scl.unwrap_or(Vec3 { x: 0.0, y: 0.0, z: 0.0 });
        w.extend([s.x.to_bits(), s.y.to_bits(), s.z.to_bits()]);
        for (k, v) in w.iter().enumerate() {
            h.w(if k == 22 { *v } else { cn(*v) });
        }
        for r in rt.chain(t, JObjId(i as u32)) {
            h.w(r.flags);
            match r.flags & ROBJ_TYPE_MASK {
                REFTYPE_JOBJ => h.w(r.u.jobj.map_or(NONE, |j| j.0)),
                REFTYPE_LIMIT => h.w(cn(r.u.limit.to_bits())),
                REFTYPE_IKHINT => {
                    h.w(cn(r.u.ik_hint.bone_length.to_bits()));
                    h.w(cn(r.u.ik_hint.rotate_x.to_bits()));
                }
                _ => {}
            }
        }
    }
}

fn flush(tree: &mut JObjTree<u32, u32>, rt: &mut JObjRt<Ext>, pend: &mut Option<(u32, Vec<HsdRObjDesc>)>) {
    if let Some((j, descs)) = pend.take() {
        let mut chain = robj::load_desc_list(&descs);
        let mut ext = Ext;
        robj::resolve_refs_all(&mut chain, &descs, &mut |k| Some(JObjId(k - 1)), &mut ext);
        rt.set_robj_chain(tree, JObjId(j), chain);
    }
}

#[test]
fn ik_matches_c() {
    let mut tree: JObjTree<u32, u32> = JObjTree::new();
    let mut rt = JObjRt::new(Ext);
    let log: Rc<RefCell<Vec<u32>>> = Rc::new(RefCell::new(Vec::new()));
    {
        let l = log.clone();
        rt.set_dptcl_callback(Some(Box::new(move |z, lo, hi, j: JObjId| {
            l.borrow_mut().extend([0xD000_0000 | z as u32, lo as u32, hi as u32, j.0]);
        })));
    }
    let mut pend: Option<(u32, Vec<HsdRObjDesc>)> = None;
    let (mut checks, mut failed) = (0usize, 0usize);
    let mut ops = [0usize; 4];
    for line in DATA.lines() {
        let tk: Vec<&str> = line.split(' ').collect();
        if tk[0] != "R" {
            flush(&mut tree, &mut rt, &mut pend);
        }
        match tk[0] {
            "S" => {
                tree = JObjTree::new();
                rt.robjs.clear();
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
            "R" => {
                let i = hx(tk[1]);
                let flags = hx(tk[2]);
                let (a, b) = (hx(tk[4]), hx(tk[5]));
                let u = match hx(tk[3]) {
                    0 => RObjDescU::Joint(a + 1),
                    1 => RObjDescU::Limit(f32::from_bits(a)),
                    _ => RObjDescU::IkHint(HsdIKHint { bone_length: f32::from_bits(a), rotate_x: f32::from_bits(b) }),
                };
                pend.get_or_insert_with(|| (i, Vec::new())).1.push(HsdRObjDesc { flags, u });
            }
            "L" => {
                let mut h = Fnv(2166136261);
                state_hash(&tree, &rt, &mut h);
                checks += 1;
                if h.0 != hx(tk[1]) {
                    failed += 1;
                    eprintln!("build mismatch: {line}");
                }
            }
            "O" => {
                let op = hx(tk[2]) as usize;
                let a = JObjId(hx(tk[3]));
                let b = hx(tk[4]);
                let v: Vec<f32> = (0..10).map(|k| fl(tk[5 + k])).collect();
                let res_c = hx(tk[15]);
                let mut res = 0u32;
                ops[op] += 1;
                match op {
                    0 => tree.setup_matrix(&mut rt, Some(a)),
                    1 => {
                        let o = tree.get_mut(a);
                        o.rotate = Quaternion { x: v[0], y: v[1], z: v[2], w: v[3] };
                        o.scale = Vec3 { x: v[4], y: v[5], z: v[6] };
                        o.translate = Vec3 { x: v[7], y: v[8], z: v[9] };
                        tree.set_mtx_dirty(Some(a));
                    }
                    2 => {
                        let mut d = ObjData { w: [v[0].to_bits(), v[1].to_bits(), v[2].to_bits()] };
                        rt.update_func(&mut tree, Some(a), b, Some(&mut d));
                        res = d.w[0];
                    }
                    _ => tree.set_mtx_dirty(Some(a)),
                }
                let mut h = Fnv(2166136261);
                for w in log.borrow_mut().drain(..) {
                    h.w(w);
                }
                state_hash(&tree, &rt, &mut h);
                checks += 1;
                if h.0 != hx(tk[16]) || res != res_c {
                    failed += 1;
                    if failed <= 10 {
                        eprintln!("mismatch: {line} (rust res {res:08x} hash {:08x})", h.0);
                    }
                }
            }
            _ => panic!("bad line {line}"),
        }
    }
    eprintln!("oracle_ik: {checks} checks, {failed} failed, ops {ops:?}");
    assert_eq!(failed, 0);
}
