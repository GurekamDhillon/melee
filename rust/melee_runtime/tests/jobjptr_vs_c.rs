//! `melee_runtime::tu::jobj_ptr` (the raw-pointer jobj.c entry points) against the real C
//! (cref/jobjptr_ref.c: the functions cut verbatim out of src/sysdolphin/baselib/jobj.c, shipping
//! contraction + gwtool lowering; expected values in tests/data/jobjptr_ref.txt; regenerate with
//! `python rust/melee_runtime/cref/gen_jobjptr_ref.py`). Fill order matches the C driver exactly.
#![allow(static_mut_refs)]

use core::ptr::{null_mut, addr_of_mut};
use melee_runtime::tu::jobj_ptr as jp;
use melee_types::*;

struct Rng(u32);
impl Rng {
    fn rnd(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    fn rf(&mut self) -> f32 {
        let k = self.rnd() % 16;
        if k == 0 {
            return 0.0;
        }
        if k == 1 {
            return -0.0;
        }
        if k == 2 {
            return 1.0;
        }
        if k == 3 {
            return 0.0005;
        }
        ((self.rnd() % 2001) as i32 - 1000) as f32 / 37.0
    }
}

static mut CBLOG: u32 = 0;
static mut JO: *mut HSD_JObj = null_mut();

fn jid(j: *mut HSD_JObj) -> u32 {
    if j.is_null() {
        return 0;
    }
    (0..6).find(|&i| j == unsafe { JO.add(i) }).map_or(99, |i| 1 + i as u32)
}
fn lg(v: u32, m: u32) {
    unsafe { CBLOG = CBLOG.wrapping_mul(m).wrapping_add(v) }
}

unsafe extern "C" fn ufc1(j: *mut HSD_JObj, t: u32, v: f32) {
    lg(jid(j), 31);
    lg(t, 31);
    lg(v.to_bits(), 31);
}
unsafe extern "C" fn ufc2(j: *mut HSD_JObj, t: u32, v: f32) {
    lg(jid(j).wrapping_add(1), 17);
    lg(t, 17);
    lg(v.to_bits(), 17);
}
unsafe extern "C" fn dpc(a: i32, lo: i32, hi: i32, j: *mut HSD_JObj) {
    lg(a as u32, 31);
    lg(lo as u32, 31);
    lg(hi as u32, 31);
    lg(jid(j), 31);
}
unsafe extern "C" fn jsc(v: i32) {
    lg((v as u32).wrapping_add(3), 31);
}
unsafe extern "C" fn ptc(j: *mut HSD_JObj, v: i32) {
    lg(jid(j).wrapping_add(5), 31);
    lg(v as u32, 31);
}
unsafe fn spl_arc(out: *mut Vec, _spl: *mut HSD_Spline, s: f32) {
    (*out).x = s * 2.0;
    (*out).y = s + 1.0;
    (*out).z = -s;
    lg(7, 31);
}
unsafe fn robj_by_type(r: *mut HSD_RObj, ty: u32, _x: u32) -> *mut HSD_RObj {
    lg(ty, 31);
    r
}

#[test]
fn jobjptr_matches_c() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/jobjptr_ref.txt")).unwrap();
    let mut lines = text.lines();
    let mut jo: Box<[HSD_JObj; 6]> = Box::new(unsafe { core::mem::zeroed() });
    let mut dobjs: Box<[HSD_DObj; 4]> = Box::new(unsafe { core::mem::zeroed() });
    let mut ao: Box<HSD_AObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut fo: Box<[HSD_FObj; 4]> = Box::new(unsafe { core::mem::zeroed() });
    let mut ro: Box<HSD_RObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut spl: Box<HSD_Spline> = Box::new(unsafe { core::mem::zeroed() });
    let mut n1: Box<_HSD_SList> = Box::new(unsafe { core::mem::zeroed() });
    let mut n2: Box<_HSD_SList> = Box::new(unsafe { core::mem::zeroed() });
    unsafe {
        JO = jo.as_mut_ptr();
        jp::ext::SPL_ARC = Some(spl_arc);
        jp::ext::ROBJ_BY_TYPE = Some(robj_by_type);
    }
    let utypes: [i32; 26] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 20, 23, 29, 30, 33, 39, 0x28, 0x29, 0x2A, 0x32, 0x33, 0x34, 0x35, 99];
    let mut ut = [0u32; 6];
    let mut n = 0;
    for op in 0..12u32 {
        for k in 0..500u32 {
            unsafe {
                core::ptr::write_bytes(jo.as_mut_ptr(), 0, 6);
                core::ptr::write_bytes(dobjs.as_mut_ptr(), 0, 4);
                core::ptr::write_bytes(&mut *ao as *mut HSD_AObj, 0, 1);
                core::ptr::write_bytes(fo.as_mut_ptr(), 0, 4);
                core::ptr::write_bytes(&mut *ro as *mut HSD_RObj, 0, 1);
                core::ptr::write_bytes(&mut *spl as *mut HSD_Spline, 0, 1);
                CBLOG = 0;
            }
            let mut rg = Rng(0x7B0u32.wrapping_add(op.wrapping_mul(7919)).wrapping_add(k.wrapping_mul(104729)));
            let jb = jo.as_mut_ptr();
            let db = dobjs.as_mut_ptr();
            let fb = fo.as_mut_ptr();
            let splp: *mut HSD_Spline = &mut *spl;
            let aop: *mut HSD_AObj = &mut *ao;
            let rop: *mut HSD_RObj = &mut *ro;
            const MASK: u32 = 0x40 | 0x80_0000 | 0x100_0000 | 0x200_0000 | 0x1000 | 0x10 | 0x8 | 0x20 | 0x4000 | 0x20_0000 | 0x40_0000 | 0x80 | 0x100;
            unsafe {
                for i in 0..6 {
                    let j = &mut *jb.add(i);
                    j.flags = rg.rnd() & MASK;
                    j.translate = Vec { x: rg.rf(), y: rg.rf(), z: rg.rf() };
                    j.rotate.x = rg.rf();
                    j.rotate.y = rg.rf();
                    j.rotate.z = rg.rf();
                    j.rotate.w = rg.rf();
                    j.scale = Vec { x: rg.rf(), y: rg.rf(), z: rg.rf() };
                    for r in 0..3 {
                        for c in 0..4 {
                            j.mtx[r][c] = rg.rf();
                        }
                    }
                    if i > 0 {
                        let pi = (rg.rnd() % i as u32) as usize;
                        j.parent = jb.add(pi);
                        if (*jb.add(pi)).child.is_null() {
                            (*jb.add(pi)).child = jb.add(i);
                        } else {
                            let mut last = (*jb.add(pi)).child;
                            while !(*last).next.is_null() {
                                last = (*last).next;
                            }
                            (*last).next = jb.add(i);
                        }
                    }
                }
                for i in 0..4 {
                    (*db.add(i)).next = if i < 3 && (rg.rnd() & 1) != 0 { db.add(i + 1) } else { null_mut() };
                }
                for i in 0..6 {
                    let m = rg.rnd() % 3;
                    ut[i] = m;
                    let j = &mut *jb.add(i);
                    if m == 0 {
                        j.u.dobj = null_mut();
                        j.flags &= !(0x20 | 0x4000);
                    } else if m == 1 {
                        j.u.dobj = db.add((rg.rnd() % 4) as usize);
                        j.flags &= !(0x20 | 0x4000);
                    } else {
                        j.u.spline = splp;
                        j.flags |= 0x4000;
                    }
                    j.robj = if rg.rnd() & 1 != 0 { rop } else { null_mut() };
                    j.aobj = if rg.rnd() & 1 != 0 { aop } else { null_mut() };
                }
                {
                    let hi = (rg.rnd() % 6) as usize;
                    (*aop).hsd_obj = jb.add(hi).cast();
                    (*jb.add(hi)).u.spline = splp;
                    ut[hi] = 2;
                    (*jb.add(hi)).flags |= 0x4000;
                }
                for i in 0..4 {
                    (*fb.add(i)).obj_type = if rg.rnd() % 3 == 0 { 12 } else { (rg.rnd() % 12) as u8 };
                    (*fb.add(i)).next = if i < 3 { fb.add(i + 1) } else { null_mut() };
                }
                (*aop).fobj = fb;
                for i in 0..6 {
                    (*jb.add(i)).flags &= !0x20000;
                }
                n1.next = &mut *n2;
                n1.data = ufc1 as usize as *mut core::ffi::c_void;
                n2.next = null_mut();
                n2.data = ufc2 as usize as *mut core::ffi::c_void;
                jp::ufc_callbacks_ptr = if rg.rnd() & 1 != 0 { &mut *n1 } else { null_mut() };
                jp::dptcl_callback = if rg.rnd() & 1 != 0 { Some(dpc) } else { None };
                jp::jsound_callback = if rg.rnd() & 1 != 0 { Some(jsc) } else { None };
                jp::ptcltgt_callback = if rg.rnd() & 1 != 0 { Some(ptc) } else { None };
            }
            let j = unsafe { jb.add((rg.rnd() % 6) as usize) };
            let mut fl = rg.rnd();
            match rg.rnd() % 4 {
                0 => fl &= 0x8,
                1 => fl &= 0x10 | 0x8,
                2 => fl &= 0x1000 | 0x40 | 0x10,
                _ => {}
            }
            fl &= !0x20000;
            let mut ret: u32 = 0;
            unsafe {
                match op {
                    0 => jp::HSD_JObjSetMtxDirtySub(j),
                    1 => jp::HSD_JObjCheckDepend(if rg.rnd() % 8 == 0 { null_mut() } else { j }),
                    2 => jp::HSD_JObjSetFlags(j, fl),
                    3 => jp::HSD_JObjClearFlags(j, fl),
                    4 => jp::HSD_JObjSetFlagsAll(j, fl),
                    5 => jp::HSD_JObjClearFlagsAll(j, fl),
                    6 => {
                        let d = jp::HSD_JObjGetDObj(j);
                        ret = if d.is_null() { 0 } else { d.offset_from(db) as u32 + 1 };
                    }
                    7 => {
                        let a = if rg.rnd() % 8 == 0 { null_mut() } else { j };
                        let b = if rg.rnd() % 8 == 0 { null_mut() } else { db.add((rg.rnd() % 4) as usize) };
                        jp::HSD_JObjAddDObj(a, b)
                    }
                    8 => {
                        let mut d: HSD_ObjData = core::mem::zeroed();
                        let t = utypes[(rg.rnd() % 26) as usize];
                        match rg.rnd() % 3 {
                            0 => d.fv = rg.rf(),
                            1 => d.iv = rg.rnd() as i32,
                            _ => d.p = Vec { x: rg.rf(), y: rg.rf(), z: rg.rf() },
                        }
                        if t == 4 && (*j).aobj.is_null() {
                            (*j).aobj = aop;
                        }
                        jp::JObjUpdateFunc(j.cast(), t, &mut d);
                        ret = d.fv.to_bits();
                    }
                    9 => jp::JObjSortAnim(if rg.rnd() % 8 == 0 { null_mut() } else { aop }),
                    10 => {
                        let v = Vec { x: rg.rf(), y: rg.rf(), z: rg.rf() };
                        jp::HSD_JObjSetTranslate(j, &v)
                    }
                    11 => jp::HSD_JObjSetRotationZ(j, rg.rf()),
                    _ => unreachable!(),
                }
            }
            let st = {
                let mut h: u32 = 0;
                let mut hm = |v: u32| h = h.wrapping_mul(31).wrapping_add(v);
                unsafe {
                    for i in 0..6 {
                        let j = &*jb.add(i);
                        hm(j.flags);
                        hm(jid(j.parent));
                        hm(jid(j.child));
                        hm(jid(j.next));
                        hm(j.translate.x.to_bits());
                        hm(j.translate.y.to_bits());
                        hm(j.translate.z.to_bits());
                        hm(j.rotate.x.to_bits());
                        hm(j.rotate.y.to_bits());
                        hm(j.rotate.z.to_bits());
                        hm(j.rotate.w.to_bits());
                        hm(j.scale.x.to_bits());
                        hm(j.scale.y.to_bits());
                        hm(j.scale.z.to_bits());
                        for r in 0..3 {
                            for c in 0..4 {
                                hm(j.mtx[r][c].to_bits());
                            }
                        }
                        hm(ut[i]);
                        if ut[i] == 1 {
                            hm(if j.u.dobj.is_null() { 0 } else { j.u.dobj.offset_from(db) as u32 + 1 });
                        } else if ut[i] == 2 {
                            hm((j.u.spline == splp) as u32);
                        }
                    }
                    for i in 0..4 {
                        let d = (*db.add(i)).next;
                        hm(if d.is_null() { 0 } else { d.offset_from(db) as u32 + 1 });
                    }
                    for i in 0..4 {
                        let f = &*fb.add(i);
                        hm(if f.next.is_null() { 0 } else { f.next.offset_from(fb) as u32 + 1 });
                        hm(f.obj_type as u32);
                    }
                    hm(if (*aop).fobj.is_null() { 0 } else { (*aop).fobj.offset_from(fb) as u32 + 1 });
                    hm((*rop).u.ik_hint.rotate_x.to_bits());
                    hm(CBLOG);
                }
                h
            };
            let got = format!("{} {} {:08X} {:08X}", op, k, ret, st);
            let want = lines.next().expect("expected file too short");
            assert_eq!(got, want, "op {op} case {k}");
            n += 1;
        }
    }
    let _ = addr_of_mut!(jp::ufc_callbacks_ptr);
    assert!(lines.next().is_none());
    assert_eq!(n, 6000);
}
