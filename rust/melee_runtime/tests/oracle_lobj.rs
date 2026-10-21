//! Bit-for-bit check of the logic half of baselib lobj.c against the C. Vectors:
//! `tests/data/oracle_lobj.txt`, written by `oracle/build_lobj.sh` (lobj.c through
//! `helper_lobj.c`, over the real wobj.c / list.c / jobj.c, compiled with the shipping build's
//! contraction + gwtool lowering; generated inputs only). Row format: see `oracle/helper_lobj.c`.
use melee_runtime::jobj::{JObjHost, JObjId, JObjTree};
use melee_runtime::lobj::{self, HsdLObj, LObjSystem};
use melee_runtime::mtx::Vec3;
use melee_runtime::wobj::HsdWObj;

const DATA: &str = include_str!("data/oracle_lobj.txt");
const NL: usize = 8;
const NONE: u32 = 0xFFFF_FFFF;

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
fn b(f: f32) -> u32 {
    if f.is_nan() {
        0x7FC0_0000
    } else {
        f.to_bits()
    }
}
fn vb(v: &Vec3) -> [u32; 3] {
    [b(v.x), b(v.y), b(v.z)]
}
fn ix(o: Option<usize>) -> u32 {
    o.map_or(NONE, |i| i as u32)
}
fn col(c: [u8; 4]) -> u32 {
    u32::from_be_bytes(c)
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

type S = LObjSystem<u32>;

fn state(s: &S, nrelease: u32) -> Vec<u32> {
    let mut v = vec![s.nb_active as u32];
    v.extend(s.active.iter().map(|a| ix(*a)));
    v.push(s.current.len() as u32);
    for i in 0..NL {
        v.push(s.current.get(i).map_or(NONE, |c| *c as u32));
    }
    v.extend([s.lightmask_diffuse, s.lightmask_specular, s.lightmask_attnfunc, s.lightmask_alpha].map(|m| m as u32));
    v.push(nrelease);
    for i in 0..NL {
        match &s.objs[i] {
            None => v.extend([0u32; 29]),
            Some(l) => {
                v.extend([1, l.ref_count as u32, l.flags as u32, l.priority as u32, ix(l.next)]);
                v.extend([col(l.color), col(l.hw_color)]);
                for w in [&l.position, &l.interest] {
                    match w {
                        Some(w) => {
                            v.extend([1, w.flags]);
                            v.extend(vb(&w.pos));
                        }
                        None => v.extend([0; 5]),
                    }
                }
                v.extend(l.u);
                v.push(b(l.shininess));
                v.extend(vb(&l.lvec));
                v.extend([l.id, l.spec_id]);
            }
        }
    }
    v
}

fn from_state(t: &[&str]) -> (S, u32) {
    let w = |i: usize| hx(t[i]);
    let f = |i: usize| fl(t[i]);
    let mut s = S::new();
    s.nb_active = w(0) as i32;
    for i in 0..9 {
        s.active[i] = (w(1 + i) != NONE).then(|| w(1 + i) as usize);
    }
    for i in 0..w(10) as usize {
        s.current.push(w(11 + i) as usize);
    }
    s.lightmask_diffuse = w(19) as i32;
    s.lightmask_specular = w(20) as i32;
    s.lightmask_attnfunc = w(21) as i32;
    s.lightmask_alpha = w(22) as i32;
    let nrelease = w(23);
    for i in 0..NL {
        let o = 24 + 29 * i;
        assert_eq!(w(o), 1);
        let wobj = |k: usize| {
            (w(k) != 0).then(|| {
                let mut x = HsdWObj::<u32>::alloc();
                x.flags = w(k + 1);
                x.pos = Vec3 { x: f(k + 2), y: f(k + 3), z: f(k + 4) };
                x
            })
        };
        let mut l = HsdLObj::<u32>::default();
        l.ref_count = w(o + 1) as u16;
        l.flags = w(o + 2) as u16;
        l.priority = w(o + 3) as u16;
        l.next = (w(o + 4) != NONE).then(|| w(o + 4) as usize);
        l.color = w(o + 5).to_be_bytes();
        l.hw_color = w(o + 6).to_be_bytes();
        l.position = wobj(o + 7);
        l.interest = wobj(o + 12);
        for k in 0..6 {
            l.u[k] = w(o + 17 + k);
        }
        l.shininess = f(o + 23);
        l.lvec = Vec3 { x: f(o + 24), y: f(o + 25), z: f(o + 26) };
        l.id = w(o + 27);
        l.spec_id = w(o + 28);
        assert_eq!(s.insert(l), i);
    }
    (s, nrelease)
}

const IDS: [u32; 9] = [1, 2, 4, 8, 0x10, 0x20, 0x40, 0x80, 0x100];

#[test]
fn oracle_lobj() {
    let mut tree: JObjTree<u32, u32> = JObjTree::new();
    let mut host = Host;
    let mut sys = S::new();
    let mut nrelease = 0u32;
    let (mut rows, mut failed) = (0usize, 0usize);
    let mut ops = [0usize; 32];
    for (ln, line) in DATA.lines().enumerate() {
        let t: Vec<&str> = line.split_whitespace().collect();
        if t[0] == "S" {
            let st: Vec<&str> = t[1..].to_vec();
            (sys, nrelease) = from_state(&st);
            assert_eq!(state(&sys, nrelease).len(), st.len());
            continue;
        }
        rows += 1;
        let bar = t.iter().position(|x| *x == "|").unwrap();
        let bar2 = t.iter().rposition(|x| *x == "|").unwrap();
        let (op, isnull, idx, w0) = (hx(t[1]), hx(t[2]) != 0, hx(t[3]) as usize, hx(t[4]));
        let a: Vec<f32> = (0..6).map(|i| fl(t[5 + i])).collect();
        let v = Vec3 { x: fl(t[11]), y: fl(t[12]), z: fl(t[13]) };
        assert_eq!(bar, 14);
        let want_res: Vec<u32> = t[bar + 1..bar2].iter().map(|x| hx(x)).collect();
        let want_hash = hx(t[bar2 + 1]);
        ops[op as usize] += 1;
        let c = idx;
        let cn = (!isnull).then_some(idx);
        let mut res: Vec<u32> = Vec::new();
        // A destroyed light's release count: count slots that go from Some to None.
        let before: Vec<bool> = sys.objs.iter().map(|o| o.is_some()).collect();
        match op {
            0 => res.push(sys.get_flags(cn)),
            1 => sys.set_flags(cn, w0),
            2 => sys.clear_flags(cn, w0),
            3 => res.extend(
                [
                    sys.get_light_mask_diffuse(),
                    sys.get_light_mask_attn_func(),
                    sys.get_light_mask_alpha(),
                    sys.get_light_mask_specular(),
                ]
                .map(|m| m as u32),
            ),
            4 => res.push(sys.get(c).get_type()),
            5 => sys.set_active(c),
            6 => res.push(sys.get_nb_active() as u32),
            7 => res.push(ix(sys.get_active_by_id(IDS[(w0 % 9) as usize]))),
            8 => res.push(ix(sys.get_active_by_index((w0 % 13) as i32 - 2))),
            9 => sys.update_func(cn, w0 % 26, a[0]),
            10 => {
                let r = match cn {
                    Some(l) => sys.get_mut(l).get_light_vector(&mut tree, &mut host),
                    None => Vec3 { x: 7.0, y: 7.0, z: 7.0 },
                };
                res.extend(vb(&r));
            }
            11 => res.push(lobj::light_id_to_index(IDS[(w0 % 9) as usize])),
            12 => res.push(lobj::index_to_light_id(w0 % 11)),
            13 => sys.get_mut(c).set_color(w0.to_be_bytes()),
            14 => res.push(col(sys.get(c).get_color())),
            15 => {
                if let Some(l) = cn {
                    sys.get_mut(l).set_spot(a[0], w0 as i32)
                }
            }
            16 => {
                if let Some(l) = cn {
                    sys.get_mut(l).set_dist_attn(a[0], a[1], w0 as i32)
                }
            }
            17 => {
                if let Some(l) = cn {
                    sys.get_mut(l).set_attn_a(a[0], a[1], a[2])
                }
            }
            18 => {
                if let Some(l) = cn {
                    sys.get_mut(l).set_attn_k(a[0], a[1], a[2])
                }
            }
            19 => sys.get_mut(c).set_position(&v),
            20 | 22 => {
                let r = cn.and_then(|l| {
                    let o = sys.get_mut(l);
                    if op == 20 {
                        o.get_position(&mut tree, &mut host)
                    } else {
                        o.get_interest(&mut tree, &mut host)
                    }
                });
                res.push(r.is_some() as u32);
                res.extend(vb(&r.unwrap_or(Vec3 { x: 7.0, y: 7.0, z: 7.0 })));
            }
            21 => sys.get_mut(c).set_interest(&v),
            23 => {
                res.push(sys.get_position_wobj(cn).is_some() as u32);
                res.push(sys.get_interest_wobj(cn).is_some() as u32);
            }
            24 => sys.add_current(cn),
            25 => sys.delete_current(cn),
            26 => {
                if let Some(l) = cn {
                    sys.get_mut(l).set_attn(a[0], a[1], a[2], a[3], a[4], a[5])
                }
            }
            27 => res.push(ix(sys.get_current_by_type(w0 as u16))),
            28 => sys.delete_current_all(cn),
            29 => sys.set_current_all(cn),
            30 => sys.replace_all(cn),
            31 => sys.remove_all(cn),
            _ => unreachable!(),
        }
        for (i, was) in before.iter().enumerate() {
            if *was && sys.objs[i].is_none() {
                nrelease += 1;
            }
        }
        let st = state(&sys, nrelease);
        let h = st.iter().fold(2166136261u32, |h, w| fnv(h, *w));
        if res != want_res || h != want_hash {
            failed += 1;
            if failed <= 10 {
                eprintln!("line {}: op {op}: res {res:x?} want {want_res:x?}, hash {h:08x} want {want_hash:08x}", ln + 1);
            }
        }
    }
    eprintln!("oracle_lobj: {rows} rows, {failed} failed, ops {ops:?}");
    assert!(rows > 20000);
    assert!(ops.iter().all(|n| *n > 500));
    assert_eq!(failed, 0);
}
