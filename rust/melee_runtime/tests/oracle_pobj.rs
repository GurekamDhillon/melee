//! Bit-for-bit check of the logic of baselib pobj.c (`melee_runtime::pobj`, over the aobj/fobj
//! ports) against the C. Vectors: `tests/data/oracle_pobj.txt`, written by
//! `oracle/build_pobj.sh` (helper_pobj.c includes pobj.c and aobj.c; shipping contraction +
//! gwtool lowering; generated inputs only). Each row's last word is an FNV-1a hash of the whole
//! state (3 PObjs with shape sets and AObjs, mtx_mark, the AObj counters).
use std::sync::Arc;

use melee_runtime::aobj::{AObjGlobals, HsdAObj, HsdAObjDesc};
use melee_runtime::fobj::HsdFObjDesc;
use melee_runtime::pobj::{self, HsdPObj, HsdShapeAnim, HsdShapeSet, MtxMarks, POBJ_SHAPEANIM, SHAPESET_ADDITIVE};

const DATA: &str = include_str!("data/oracle_pobj.txt");

struct Rd<'a> {
    t: Vec<&'a str>,
    i: usize,
}
impl<'a> Rd<'a> {
    fn s(&mut self) -> &'a str {
        let s = self.t[self.i];
        self.i += 1;
        s
    }
    fn u(&mut self) -> u32 {
        u32::from_str_radix(self.s(), 16).unwrap()
    }
    fn f(&mut self) -> f32 {
        f32::from_bits(self.u())
    }
    fn sep(&mut self) {
        assert_eq!(self.s(), "|");
    }
    fn desc(&mut self) -> Option<HsdAObjDesc> {
        if self.u() == 0 {
            return None;
        }
        let flags = self.u();
        let end_frame = self.f();
        let nf = self.u();
        let fobjdesc = (0..nf)
            .map(|_| {
                let length = self.u();
                let startframe = self.f();
                let type_ = self.u() as u8;
                let frac_value = self.u() as u8;
                let frac_slope = self.u() as u8;
                let h = self.s();
                let ad: Vec<u8> = (0..h.len()).step_by(2).map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap()).collect();
                HsdFObjDesc { length, startframe, type_, frac_value, frac_slope, ad: Arc::from(ad) }
            })
            .collect();
        Some(HsdAObjDesc { flags, end_frame, fobjdesc, obj_id: 0 })
    }
}

struct H(u32);
impl H {
    fn w(&mut self, v: u32) {
        for b in v.to_le_bytes() {
            self.0 ^= b as u32;
            self.0 = self.0.wrapping_mul(16777619);
        }
    }
}
fn oidx(o: Option<u32>) -> u32 {
    o.unwrap_or(0xFFFF_FFFF)
}
fn dump_aobj(h: &mut H, a: Option<&HsdAObj<()>>) {
    let Some(a) = a else { return h.w(0) };
    h.w(1);
    for v in [a.flags, a.curr_frame.to_bits(), a.rewind_frame.to_bits(), a.end_frame.to_bits(), a.framerate.to_bits()] {
        h.w(v);
    }
    h.w(a.fobj.len() as u32);
    for f in &a.fobj {
        for v in [
            f.flags as u32,
            f.op as u32,
            f.op_intrp as u32,
            f.nb_pack as u32,
            f.fterm as u32,
            f.time.to_bits(),
            f.p0.to_bits(),
            f.p1.to_bits(),
            f.d0.to_bits(),
            f.d1.to_bits(),
            f.ad as u32,
        ] {
            h.w(v);
        }
    }
}
fn hash(po: &[HsdPObj], mm: &MtxMarks<u32>, g: &AObjGlobals) -> u32 {
    let mut h = H(2166136261);
    for p in po {
        h.w(p.flags as u32);
        if p.pobj_type() == POBJ_SHAPEANIM {
            let ss = p.shape_set.as_ref().unwrap();
            h.w(ss.flags as u32);
            if ss.flags & SHAPESET_ADDITIVE != 0 {
                ss.bp.iter().for_each(|b| h.w(b.to_bits()));
            } else {
                h.w(ss.bl.to_bits());
            }
            dump_aobj(&mut h, ss.aobj.as_ref());
        }
    }
    for i in 0..2 {
        h.w(oidx(mm.obj[i]));
        h.w(mm.mark[i]);
    }
    h.w(g.n_stopped as u32);
    h.w(g.n_running as u32);
    h.0
}
fn load(d: Option<HsdAObjDesc>) -> Option<HsdAObj<()>> {
    d.map(|d| HsdAObj::load_desc(&d, |_| None))
}

#[test]
fn pobj_matches_c() {
    let mut po: Vec<HsdPObj> = Vec::new();
    let mut mm = MtxMarks::<u32>::default();
    let mut g = AObjGlobals { n_stopped: 0, n_running: 0, endcallback_list: Vec::new() };
    let mut rows = 0;
    for (ln, line) in DATA.lines().enumerate() {
        let mut r = Rd { t: line.split_whitespace().collect(), i: 0 };
        let op = r.u();
        let mut out: Vec<u32> = Vec::new();
        if op == 0xFFFF_FFFF {
            po.clear();
            g.n_stopped = 0;
            g.n_running = 0;
            for _ in 0..3 {
                let flags = r.u() as u16;
                let mut p = HsdPObj { flags, n_display: 0, shape_set: None };
                if p.pobj_type() == POBJ_SHAPEANIM {
                    let sflags = r.u() as u16;
                    let nb_shape = r.u() as u16;
                    let bp: Vec<f32> = (0..8).map(|_| r.f()).collect();
                    let bl = r.f();
                    let mut ss = HsdShapeSet { flags: sflags, nb_shape, bp, bl, aobj: None };
                    if sflags & SHAPESET_ADDITIVE != 0 {
                        ss.bl = 0.0;
                    }
                    ss.aobj = load(r.desc());
                    p.shape_set = Some(ss);
                }
                po.push(p);
            }
        } else if op <= 9 {
            let nul = r.u() != 0;
            let i = r.u() as usize;
            match op {
                0 => {
                    r.sep();
                    out.push(pobj::get_flags(if nul { None } else { Some(&po[i]) }));
                }
                1 | 2 => {
                    let f = r.u();
                    r.sep();
                    if nul {
                        pobj::remove_anim_by_flags(None, f);
                    } else if op == 1 {
                        pobj::remove_anim_by_flags(Some(&mut po[i]), f);
                    } else {
                        pobj::remove_anim_all_by_flags(&mut po[i..], f);
                    }
                }
                3 => {
                    let nosa = r.u() != 0;
                    let sa = if !nul && !nosa { Some(HsdShapeAnim { aobjdesc: r.desc() }) } else { None };
                    r.sep();
                    if nul {
                        pobj::add_anim(None, None);
                    } else {
                        pobj::add_anim(Some(&mut po[i]), sa.as_ref());
                    }
                }
                4 => {
                    let none = r.u() != 0;
                    let sa: Vec<HsdShapeAnim> = (i..3)
                        .map(|_| HsdShapeAnim { aobjdesc: if !nul && !none { r.desc() } else { None } })
                        .collect();
                    r.sep();
                    if !nul {
                        pobj::add_anim_all(&mut po[i..], if none { &[] } else { &sa });
                    }
                }
                5 | 6 => {
                    let fr = r.f();
                    let f = r.u();
                    r.sep();
                    if nul {
                        pobj::req_anim_by_flags(None, fr, f);
                    } else if op == 5 {
                        pobj::req_anim_by_flags(Some(&mut po[i]), fr, f);
                    } else {
                        pobj::req_anim_all_by_flags(&mut po[i..], fr, f);
                    }
                }
                7 => {
                    let ty = r.u();
                    let fv = r.f();
                    r.sep();
                    pobj::pobj_update_func(if nul { None } else { Some(&mut po[i]) }, ty, fv);
                }
                _ => {
                    let reps = r.u();
                    r.sep();
                    for _ in 0..reps {
                        if nul {
                            pobj::anim(None, &mut g);
                        } else if op == 8 {
                            pobj::anim(Some(&mut po[i]), &mut g);
                        } else {
                            pobj::anim_all(&mut po[i..], &mut g);
                        }
                    }
                }
            }
        } else if op == 10 {
            let sub = r.u();
            let idx = r.u() as i32;
            let o = r.u();
            let obj = if o == 0xFFFF_FFFF { None } else { Some(o) };
            let mark = r.u();
            r.sep();
            match sub {
                0 => mm.clear(obj, mark),
                1 => mm.set(idx, obj, mark),
                _ => {
                    let (go, gm) = mm.get(idx);
                    out.push(oidx(go));
                    out.push(gm);
                }
            }
        } else {
            let kind = r.u();
            let scale = r.u() as i32;
            let src: Vec<u8> = (0..6).map(|_| r.u() as u8).collect();
            r.sep();
            let h: Vec<u16> = (0..3).map(|k| u16::from_le_bytes([src[2 * k], src[2 * k + 1]])).collect();
            let d = match kind {
                0 => pobj::decode_u8_xyz(&src, scale),
                1 => pobj::decode_s8_xyz(&src, scale),
                2 => pobj::decode_u16_xyz(&h, scale),
                _ => pobj::decode_s16_xyz(&h, scale),
            };
            out.extend(d.map(|x| if x.is_nan() { 0x7FC0_0000 } else { x.to_bits() }));
        }
        out.push(hash(&po, &mm, &g));
        let want: Vec<u32> = r.t[r.i..].iter().map(|s| u32::from_str_radix(s, 16).unwrap()).collect();
        assert_eq!(out, want, "row {ln}: {line}");
        rows += 1;
    }
    assert_eq!(rows, 20_500);
}
