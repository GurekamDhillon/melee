//! `tu::cobj_world` vs baselib/cobj.c (over the real wobj.c, shipping contraction), bit for bit on
//! generated inputs. Regenerate the expected data with
//! `python rust/melee_runtime/gen/pure/run_pure.py cobj_world`.
#![allow(static_mut_refs)]
mod pure_common;
use melee_runtime::tu::cobj::ext::hook as ch;
use melee_runtime::tu::cobj_world::*;
use melee_types::*;
use pure_common::*;

static mut SCALE: u32 = 0;

struct H(u64);
impl H {
    fn w(&mut self, w: u32) {
        for b in w.to_le_bytes() {
            self.0 ^= b as u64;
            self.0 = self.0.wrapping_mul(0x100000001B3);
        }
    }
    fn f(&mut self, f: f32) {
        self.w(f.to_bits())
    }
    fn wobj(&mut self, w: *mut HSD_WObj) {
        if w.is_null() {
            return self.w(0xDEAD);
        }
        unsafe {
            self.f((*w).pos.x);
            self.f((*w).pos.y);
            self.f((*w).pos.z);
            self.w((*w).flags);
        }
    }
    fn cobj(&mut self, c: &HSD_CObj) {
        self.w(c.flags);
        self.f(c.viewport.xmin);
        self.f(c.viewport.xmax);
        self.f(c.viewport.ymin);
        self.f(c.viewport.ymax);
        self.w(c.scissor.left as u32);
        self.w(c.scissor.right as u32);
        self.w(c.scissor.top as u32);
        self.w(c.scissor.bottom as u32);
        unsafe {
            self.f(c.projection_param.frustum.top);
            self.f(c.projection_param.frustum.bottom);
            self.f(c.projection_param.frustum.left);
            self.f(c.projection_param.frustum.right);
        }
        self.w(c.projection_type as u32);
        self.f(c.view_authored_aspect);
        self.w(c.view_auto as u32);
        self.wobj(c.eyepos);
        self.wobj(c.interest);
        unsafe {
            self.f(c.u.up.x);
            self.f(c.u.up.y);
            self.f(c.u.up.z);
        }
        for i in 0..3 {
            for j in 0..4 {
                self.f(c.view_mtx[i][j]);
            }
        }
        if c.proj_mtx.is_null() {
            self.w(0);
        } else {
            self.w(1);
            for i in 0..3 {
                for j in 0..4 {
                    unsafe { self.f((*c.proj_mtx)[i][j]) };
                }
            }
        }
    }
}

fn rwobj(r: &mut Rng, w: &mut HSD_WObj) {
    w.pos.x = r.f();
    w.pos.y = r.f();
    w.pos.z = r.f();
    w.flags = r.u() as u32 & 0xFFFE;
    w.aobj = core::ptr::null_mut();
}

fn nz(r: &mut Rng, c: &mut HSD_CObj) -> *mut HSD_CObj {
    if r.u() % 6 == 0 {
        core::ptr::null_mut()
    } else {
        c
    }
}

#[test]
fn cobj_world_matches_c() {
    unsafe {
        ch::View_ProjectionScaleBits = Some(|| SCALE as i32);
        melee_runtime::tu::cobj_world::ext::HSD_MtxAlloc = Some(|| Box::into_raw(Box::new([[0.0f32; 4]; 3])));
    }
    let mut r = Rng::new();
    let mut out = String::new();
    for op in 0..23 {
        for k in 0..300 {
            let mut c: HSD_CObj = HSD_CObj::zeroed();
            let (mut we, mut wi) = (HSD_WObj::zeroed(), HSD_WObj::zeroed());
            c.flags = r.u() as u32;
            c.viewport.xmin = r.f();
            c.viewport.xmax = r.f();
            c.viewport.ymin = r.f();
            c.viewport.ymax = r.f();
            c.scissor.left = r.u() as u16;
            c.scissor.right = r.u() as u16;
            c.scissor.top = r.u() as u16;
            c.scissor.bottom = r.u() as u16;
            c.projection_type = (r.u() % 4) as u8;
            unsafe {
                c.projection_param.frustum.top = r.f();
                c.projection_param.frustum.bottom = r.f();
                c.projection_param.frustum.left = r.f();
                c.projection_param.frustum.right = r.f();
            }
            c.view_authored_aspect = r.f();
            c.view_auto = (r.u() % 2) as u8;
            unsafe { SCALE = 0x3F000000 + (r.u() % 0x01000000) as u32 };
            rwobj(&mut r, &mut we);
            rwobj(&mut r, &mut wi);
            if r.u() % 5 == 0 {
                wi.pos = we.pos;
            }
            if r.u() % 7 == 0 {
                wi.pos.x = we.pos.x + 1e-39;
                wi.pos.y = we.pos.y;
                wi.pos.z = we.pos.z;
            }
            c.eyepos = if r.u() % 8 == 0 { core::ptr::null_mut() } else { &mut we };
            c.interest = if r.u() % 8 == 0 { core::ptr::null_mut() } else { &mut wi };
            let mut v = Vec { x: r.f(), y: r.f(), z: r.f() };
            let mut pm: [[f32; 4]; 3] = [[0.0; 4]; 3];
            c.u.up = Vec { x: r.f(), y: r.f(), z: r.f() };
            for i in 0..3 {
                for j in 0..4 {
                    c.view_mtx[i][j] = r.f();
                }
            }
            for i in 0..3 {
                for j in 0..4 {
                    pm[i][j] = r.f();
                }
            }
            c.proj_mtx = if r.u() % 3 == 0 { core::ptr::null_mut() } else { &mut pm };
            if op == 18 || op == 19 {
                c.eyepos = &mut we;
                c.interest = &mut wi;
            }
            let (mut t, mut b) = (-7.0f32, -9.0f32);
            let mut ret = 0i32;
            let mut h = H(0xCBF29CE484222325);
            unsafe {
                match op {
                    0 => ret = HSD_CObjGetFlags(&mut c) as i32,
                    1 => {
                        let p = nz(&mut r, &mut c);
                        h.f(HSD_CObjGetFov(p))
                    }
                    2 => {
                        let p = nz(&mut r, &mut c);
                        h.f(HSD_CObjGetAuthoredAspect(p))
                    }
                    3 => {
                        let p = nz(&mut r, &mut c);
                        let a = r.f();
                        HSD_CObjSetAspect(p, a)
                    }
                    4 => {
                        let m = (r.u() % 4) as i32;
                        let p = nz(&mut r, &mut c);
                        HSD_CObjGetPerspective(
                            p,
                            if m & 1 != 0 { &mut t } else { core::ptr::null_mut() },
                            if m & 2 != 0 { &mut b } else { core::ptr::null_mut() },
                        );
                        h.f(t);
                        h.f(b);
                    }
                    5 => {
                        let mut sc = _Scissor { left: 1, right: 2, top: 3, bottom: 4 };
                        let p = nz(&mut r, &mut c);
                        HSD_CObjGetScissor(p, &mut sc);
                        h.w(sc.left as u32);
                        h.w(sc.right as u32);
                        h.w(sc.top as u32);
                        h.w(sc.bottom as u32);
                    }
                    6 => {
                        let mut vp = _HSD_RectF32 { xmin: 1.0, xmax: 2.0, ymin: 3.0, ymax: 4.0 };
                        let p = nz(&mut r, &mut c);
                        HSD_CObjGetViewportf(p, &mut vp);
                        h.f(vp.xmin);
                        h.f(vp.xmax);
                        h.f(vp.ymin);
                        h.f(vp.ymax);
                    }
                    7 => {
                        let (a, b2, c2, d) = (r.f(), r.f(), r.f(), r.f());
                        let mut vp = _HSD_RectF32 { xmin: a, xmax: b2, ymin: c2, ymax: d };
                        let p = nz(&mut r, &mut c);
                        HSD_CObjSetViewportf(p, &mut vp);
                    }
                    8 => {
                        let p = nz(&mut r, &mut c);
                        let t = r.u() as u32;
                        HSD_CObjSetProjectionType(p, t)
                    }
                    9 => {
                        if !c.interest.is_null() {
                            HSD_CObjGetInterest(&mut c, &mut v);
                            h.f(v.x);
                            h.f(v.y);
                            h.f(v.z);
                        }
                    }
                    10 => {
                        if !c.interest.is_null() {
                            HSD_CObjSetInterest(&mut c, &mut v)
                        }
                    }
                    11 => {
                        if !c.eyepos.is_null() {
                            HSD_CObjGetEyePosition(&mut c, &mut v);
                            h.f(v.x);
                            h.f(v.y);
                            h.f(v.z);
                        }
                    }
                    12 => {
                        if !c.eyepos.is_null() {
                            HSD_CObjSetEyePosition(&mut c, &mut v)
                        }
                    }
                    13 => {
                        ret = HSD_CObjGetEyeVector(&mut c, &mut v);
                        h.f(v.x);
                        h.f(v.y);
                        h.f(v.z);
                    }
                    14 => ret = HSD_CObjGetEyeVector(&mut c, core::ptr::null_mut()),
                    15 => {
                        let cp = nz(&mut r, &mut c);
                        let vp: *mut Vec = if r.u() % 6 == 0 { core::ptr::null_mut() } else { &mut v };
                        ret = HSD_CObjGetUpVector(cp, vp);
                        h.f(v.x);
                        h.f(v.y);
                        h.f(v.z);
                    }
                    16 => {
                        let cp = nz(&mut r, &mut c);
                        let vp: *mut Vec = if r.u() % 6 == 0 { core::ptr::null_mut() } else { &mut v };
                        HSD_CObjSetUpVector(cp, vp);
                    }
                    17 => {
                        let cp = nz(&mut r, &mut c);
                        let roll = r.f();
                        HSD_CObjSetRoll(cp, roll);
                    }
                    18 => {
                        let m = HSD_CObjGetViewingMtxPtr(&mut c);
                        ret = if m as *const f32 == c.view_mtx.as_ptr() as *const f32 { 1 } else { 2 };
                    }
                    19 => {
                        let m = HSD_CObjGetInvViewingMtxPtrDirect(&mut c);
                        ret = if m as *const f32 == pm.as_ptr() as *const f32 {
                            1
                        } else if m == c.proj_mtx as *mut [f32; 4] {
                            2
                        } else {
                            3
                        };
                    }
                    20 => {
                        let cp = nz(&mut r, &mut c);
                        let (l, rr, tt, bb) = (r.u() as u16, r.u() as u16, r.u() as u16, r.u() as u16);
                        HSD_CObjSetScissorx4(cp, l, rr, tt, bb);
                    }
                    21 => {
                        let cp = nz(&mut r, &mut c);
                        let (l, rr, tt, bb) = (r.f(), r.f(), r.f(), r.f());
                        HSD_CObjSetViewportfx4(cp, l, rr, tt, bb);
                    }
                    _ => {
                        let m = (r.u() % 16) as i32;
                        let mut o = [-1.0f32, -2.0, -3.0, -4.0];
                        let cp = nz(&mut r, &mut c);
                        let op_ = o.as_mut_ptr();
                        let pick = |bit: i32, i: usize| if m & bit != 0 { op_.add(i) } else { core::ptr::null_mut() };
                        HSD_CObjGetOrtho(cp, pick(1, 0), pick(2, 1), pick(4, 2), pick(8, 3));
                        for x in o {
                            h.f(x);
                        }
                    }
                }
            }
            h.w(ret as u32);
            h.cobj(&c);
            out += &line(&format!("op{op}"), k, h.0);
        }
    }
    compare("cobj_world", &out);
}
