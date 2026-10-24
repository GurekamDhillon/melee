//! `tu::jobj_world` vs baselib/jobj.c (shipping contraction), bit for bit over random JObj forests.
//! Regenerate the expected data with `python rust/melee_runtime/gen/pure/run_pure.py jobj_world`.
#![allow(static_mut_refs)]
mod pure_common;
use melee_runtime::tu::jobj_world::*;
use melee_types::*;
use pure_common::*;

const NN: usize = 8;

struct W {
    n: [HSD_JObj; NN],
    d: [HSD_DObj; NN],
    h: u64,
}
impl W {
    fn hw(&mut self, w: u32) {
        for b in w.to_le_bytes() {
            self.h ^= b as u64;
            self.h = self.h.wrapping_mul(0x100000001B3);
        }
    }
    fn idx(&self, p: *mut HSD_JObj) -> i32 {
        if p.is_null() {
            return -1;
        }
        for i in 0..NN {
            if &self.n[i] as *const _ as *mut HSD_JObj == p {
                return i as i32;
            }
        }
        -2
    }
    fn didx(&self, p: *mut HSD_DObj) -> i32 {
        if p.is_null() {
            return -1;
        }
        for i in 0..NN {
            if &self.d[i] as *const _ as *mut HSD_DObj == p {
                return i as i32;
            }
        }
        -2
    }
    fn hall(&mut self) {
        for i in 0..NN {
            let (f, p, c, n) = (self.n[i].flags, self.n[i].parent, self.n[i].child, self.n[i].next);
            let (pi, ci, ni) = (self.idx(p), self.idx(c), self.idx(n));
            self.hw(f);
            self.hw(pi as u32);
            self.hw(ci as u32);
            self.hw(ni as u32);
        }
    }
}

#[test]
fn jobj_world_matches_c() {
    let mut r = Rng::new();
    let mut out = String::new();
    let mut w = Box::new(W { n: unsafe { core::mem::zeroed() }, d: unsafe { core::mem::zeroed() }, h: 0 });
    let np = |w: &mut W, i: usize| -> *mut HSD_JObj { &mut w.n[i] };
    for op in 0..13 {
        for k in 0..400 {
            w.h = 0xCBF29CE484222325;
            let mut last = [-1i32; NN];
            for i in 0..NN {
                w.n[i] = HSD_JObj::zeroed();
            }
            for i in 0..NN - 1 {
                let par = if i > 0 && r.u() % 4 != 0 { (r.u() % i as u64) as i32 } else { -1 };
                w.n[i].flags = r.u() as u32 & !(1u32 << 12) & !(1u32 << 5) & !(1u32 << 14);
                if r.u() % 5 == 0 {
                    w.n[i].flags |= 1 << 12;
                }
                if op >= 8 {
                    w.n[i].flags &= !(1u32 << 12);
                }
                if par >= 0 {
                    let pp = np(&mut w, par as usize);
                    w.n[i].parent = pp;
                    let ip = np(&mut w, i);
                    if last[par as usize] < 0 {
                        w.n[par as usize].child = ip;
                    } else {
                        w.n[last[par as usize] as usize].next = ip;
                    }
                    last[par as usize] = i as i32;
                }
            }
            w.n[NN - 1].flags = r.u() as u32 & !(1u32 << 12) & !(1u32 << 5) & !(1u32 << 14);
            for i in 0..NN {
                w.d[i].next = core::ptr::null_mut();
                w.n[i].u.dobj = if r.u() % 3 == 0 {
                    core::ptr::null_mut()
                } else {
                    let j = (r.u() % NN as u64) as usize;
                    &mut w.d[j]
                };
                if r.u() % 6 == 0 {
                    w.n[i].flags |= if r.u() % 2 != 0 { 1 << 5 } else { 1 << 14 };
                }
            }
            let fl = r.u() as u32;
            let a = (r.u() % (NN as u64 - 1)) as usize;
            let mut ret = 0i32;
            let mut nz = |r: &mut Rng, w: &mut W| -> *mut HSD_JObj {
                if r.u() % 8 == 0 {
                    core::ptr::null_mut()
                } else {
                    np(w, a)
                }
            };
            unsafe {
                match op {
                    0 => {
                        let p = nz(&mut r, &mut w);
                        ret = HSD_JObjGetFlags(p) as i32
                    }
                    1 => {
                        let p = nz(&mut r, &mut w);
                        HSD_JObjSetFlags(p, fl)
                    }
                    2 => {
                        let p = nz(&mut r, &mut w);
                        HSD_JObjClearFlags(p, fl)
                    }
                    3 => {
                        let p = nz(&mut r, &mut w);
                        HSD_JObjSetFlagsAll(p, fl)
                    }
                    4 => {
                        let p = nz(&mut r, &mut w);
                        HSD_JObjClearFlagsAll(p, fl)
                    }
                    5 => {
                        let p = np(&mut w, a);
                        HSD_JObjSetMtxDirtySub(p)
                    }
                    6 => {
                        let p = nz(&mut r, &mut w);
                        let d = HSD_JObjGetDObj(p);
                        ret = w.didx(d)
                    }
                    7 => {
                        let p = nz(&mut r, &mut w);
                        let q = HSD_JObjGetPrev(p);
                        ret = w.idx(q)
                    }
                    8 => {
                        let (p, c) = (np(&mut w, a), np(&mut w, NN - 1));
                        HSD_JObjAddChild(p, c)
                    }
                    9 => {
                        let b = (r.u() % (a as u64 + 1)) as i32 - 1;
                        let (p, q) = (np(&mut w, a), if b < 0 { core::ptr::null_mut() } else { np(&mut w, b as usize) });
                        let x = HSD_JObjReparent(p, q);
                        ret = w.idx(x)
                    }
                    10 => {
                        let (p, c) = (np(&mut w, a), np(&mut w, NN - 1));
                        HSD_JObjAddNext(p, c)
                    }
                    11 => {
                        let p = np(&mut w, a);
                        RecalcParentTrspBits(p)
                    }
                    _ => {
                        let (p, c) = (np(&mut w, NN - 1), np(&mut w, 0));
                        HSD_JObjAddChild(p, c)
                    }
                }
            }
            w.hw(ret as u32);
            w.hall();
            out += &line(&format!("op{op}"), k, w.h);
        }
    }
    compare("jobj_world", &out);
}
