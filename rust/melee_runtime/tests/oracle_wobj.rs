//! Bit-for-bit check of the baselib wobj.c port and the dobj.c flag functions against the C.
//! Vectors: `tests/data/oracle_wobj.txt`, written by `oracle/build_wobj.sh` (wobj.c, dobj.c,
//! jobj.c, spline.c, mtx.c, mtx_pc.c compiled with the shipping build's contraction + gwtool
//! lowering; generated inputs only). See `oracle/driver_wobj.c` for the row format.
use melee_runtime::aobj::HsdAObj;
use melee_runtime::dobj;
use melee_runtime::jobj::{JObjHost, JObjId, JObjTree};
use melee_runtime::mtx::Vec3;
use melee_runtime::spline::Spline;
use melee_runtime::wobj::{HsdWObj, WObjHost};

const DATA: &str = include_str!("data/oracle_wobj.txt");

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

#[derive(Default, Clone)]
struct Spl {
    ty: u8,
    numcv: i16,
    tension: f32,
    total: f32,
    cv: Vec<Vec3>,
    seg: Vec<f32>,
    poly: Vec<[f32; 5]>,
}

#[derive(Default)]
struct Host {
    spl: Vec<Spl>,
}
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
impl WObjHost<u32, u32> for Host {
    type RObjDesc = ();
    type RObjAnim = ();
    fn jobj_spline(&self, j: JObjId) -> Option<Spline<'_>> {
        let s = self.spl.get(j.0 as usize)?;
        Some(Spline {
            ty: s.ty,
            numcv: s.numcv,
            tension: s.tension,
            cv: &s.cv,
            total_length: s.total,
            seg_length: &s.seg,
            seg_poly: &s.poly,
        })
    }
    fn robj_remove_anim_all(&mut self, _: &mut [u32]) {}
    fn robj_req_anim_all(&mut self, _: &mut [u32], _: f32) {}
    fn robj_add_anim_all(&mut self, _: &mut [u32], _: Option<&()>) {}
    fn robj_anim_all(&mut self, _: &mut [u32]) {}
    fn robj_load_desc(&mut self, _: Option<&()>) -> Vec<u32> {
        Vec::new()
    }
    fn resolve_aobj_obj(&mut self, _: u32) -> Option<JObjId> {
        None
    }
}

fn set_srt(tree: &mut JObjTree<u32, u32>, j: JObjId, t: &[&str]) {
    let v: Vec<f32> = t.iter().map(|s| fl(s)).collect();
    let n = tree.get_mut(j);
    n.rotate.x = v[0];
    n.rotate.y = v[1];
    n.rotate.z = v[2];
    n.rotate.w = v[3];
    n.scale = Vec3 { x: v[4], y: v[5], z: v[6] };
    n.translate = Vec3 { x: v[7], y: v[8], z: v[9] };
}

#[test]
fn wobj_matches_c() {
    let mut tree: JObjTree<u32, u32> = JObjTree::new();
    let mut host = Host::default();
    let mut w: HsdWObj<u32> = HsdWObj::alloc();
    let mut dflags = 0u32;
    let mut dobj_seeded = false;
    let mut cur = dobj::DObjGlobals::<u32>::default();
    let (mut rows, mut failed, mut path_hits) = (0usize, 0usize, 0usize);
    for line in DATA.lines() {
        let t: Vec<&str> = line.split_ascii_whitespace().collect();
        match t[0] {
            "S" => {
                tree = JObjTree::new();
                let j: Vec<JObjId> = (0..3).map(|_| tree.alloc()).collect();
                tree.add_child(Some(j[0]), Some(j[1]));
                tree.add_child(Some(j[1]), Some(j[2]));
                host.spl = vec![Spl::default(); 3];
                let mode = hx(t[2]);
                let tgt = hx(t[3]);
                w = HsdWObj::alloc();
                if mode != 0 {
                    let mut a = HsdAObj::alloc();
                    // The driver's AObj is a zeroed record: only hsd_obj is set.
                    a.flags = 0;
                    a.framerate = 0.0;
                    a.hsd_obj = if mode == 2 { Some(JObjId(tgt)) } else { None };
                    w.aobj = Some(a);
                }
                dobj_seeded = false;
            }
            "D" => {
                dflags = hx(t[1]);
                dobj_seeded = true;
            }
            "J" | "R" => {
                let k = hx(t[1]);
                set_srt(&mut tree, JObjId(k), &t[2..12]);
                if t[0] == "R" {
                    tree.set_mtx_dirty(Some(JObjId(k)));
                }
            }
            "P" => {
                let k = hx(t[1]) as usize;
                let mut i = 6;
                let ncv = hx(t[i]) as usize;
                i += 1;
                let cvf: Vec<f32> = t[i..i + ncv].iter().map(|s| fl(s)).collect();
                i += ncv;
                let nseg = hx(t[i]) as usize;
                i += 1;
                let seg: Vec<f32> = t[i..i + nseg].iter().map(|s| fl(s)).collect();
                i += nseg;
                let np = hx(t[i]) as usize;
                i += 1;
                let pf: Vec<f32> = t[i..i + np].iter().map(|s| fl(s)).collect();
                host.spl[k] = Spl {
                    ty: hx(t[2]) as u8,
                    numcv: hx(t[3]) as i16,
                    tension: fl(t[4]),
                    total: fl(t[5]),
                    cv: cvf.chunks(3).map(|c| Vec3 { x: c[0], y: c[1], z: c[2] }).collect(),
                    seg,
                    poly: pf.chunks(5).map(|c| [c[0], c[1], c[2], c[3], c[4]]).collect(),
                };
            }
            "O" => {
                rows += 1;
                let op = hx(t[3]);
                let (a, b, c) = (hx(t[4]), hx(t[5]), hx(t[6]));
                assert!(dobj_seeded);
                let mut r = [0u32; 3];
                match op {
                    0 => w.set_position(&Vec3 { x: f32::from_bits(a), y: f32::from_bits(b), z: f32::from_bits(c) }),
                    1 => w.set_position_x(&mut tree, &mut host, f32::from_bits(a)),
                    2 => w.set_position_y(&mut tree, &mut host, f32::from_bits(a)),
                    3 => w.set_position_z(&mut tree, &mut host, f32::from_bits(a)),
                    4 => {
                        let p = w.get_position(&mut tree, &mut host);
                        r = [p.x.to_bits(), p.y.to_bits(), p.z.to_bits()];
                    }
                    5 => w.clear_flags(a),
                    6 => {
                        if a == 4 {
                            path_hits += 1;
                        }
                        w.update_func(&mut tree, &mut host, a as u8, f32::from_bits(b));
                    }
                    8 => {}
                    _ => match a {
                        0 => {
                            r[0] = dobj::get_flags(Some(&dflags));
                            r[1] = dobj::get_flags(None);
                            cur.set_current(Some(0));
                        }
                        1 => {
                            dobj::set_flags(Some(&mut dflags), b);
                            dobj::set_flags(None, b);
                        }
                        2 => {
                            dobj::clear_flags(Some(&mut dflags), b);
                            dobj::clear_flags(None, b);
                        }
                        _ => {
                            dobj::modify_flags(Some(&mut dflags), b, c);
                            dobj::modify_flags(None, b, c);
                        }
                    },
                }
                let mut h = 2166136261u32;
                for k in 0..3 {
                    let n = tree.get(JObjId(k));
                    h = fnv(h, n.flags);
                    for row in &n.mtx {
                        for v in row {
                            h = fnv(h, v.to_bits());
                        }
                    }
                }
                let got = [
                    r[0], r[1], r[2], w.flags, w.pos.x.to_bits(), w.pos.y.to_bits(), w.pos.z.to_bits(), dflags, h,
                ];
                let exp: Vec<u32> = t[8..17].iter().map(|s| hx(s)).collect();
                if got[..] != exp[..] {
                    failed += 1;
                    if failed <= 10 {
                        eprintln!("row {line}\n got {got:x?}");
                    }
                }
            }
            other => panic!("bad row {other}"),
        }
    }
    eprintln!("wobj oracle: {rows} rows, {failed} failed, {path_hits} path updates");
    assert!(rows > 10_000 && path_hits > 500);
    assert_eq!(failed, 0);
}
