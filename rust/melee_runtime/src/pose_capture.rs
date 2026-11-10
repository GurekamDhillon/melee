//! Render-only capture of what the render pass drew from (melee_core `pose`), for frame interpolation.
//!
//! The pobj / cobj setup code calls in here right after it loads a matrix or makes a camera current. Everything is
//! read from the game's objects and copied: no game memory is written, no game function is called, and the hooks are
//! one relaxed atomic load when capture is off (always, in headless runs: the windowed presenter turns it on). The
//! authored channels of a joint are sampled by running a copy of each `HSD_FObj` forward with this module's own
//! interpreter (the same keyframe rules as fobj.c's `HSD_FObjInterpretAnim`), never the game's FObj.
#![allow(static_mut_refs, clippy::missing_safety_doc)]
use core::sync::atomic::{AtomicBool, Ordering::Relaxed};
use melee_core::pose::*;
use melee_types::*;
use std::collections::HashMap;

/// Capture on (set by the windowed presenter while interpolation is on).
pub static ON: AtomicBool = AtomicBool::new(false);
/// Emits the GX stream's pose tag for matrix slot `slot` (0..9): 1-based slot record index.
pub static mut TAG: Option<fn(u32, u32)> = None;
/// Emits the GX stream's camera tag (1-based camera index).
pub static mut CAMERA: Option<fn(u32)> = None;

/// Addresses hash well with one multiply (SipHash was a third of the capture's time).
#[derive(Default)]
struct AddrHasher(u64);
impl core::hash::Hasher for AddrHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, b: &[u8]) {
        for &x in b {
            self.0 = (self.0 ^ x as u64).wrapping_mul(0x100000001b3);
        }
    }
    fn write_usize(&mut self, v: usize) {
        self.0 = (v as u64 ^ (v as u64 >> 29)).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    }
}
type AddrMap = HashMap<usize, u32, core::hash::BuildHasherDefault<AddrHasher>>;

#[derive(Default)]
struct Builder {
    f: PoseFrame,
    joints: AddrMap,
    cams: AddrMap,
    cur_cam: u32,
    last_right: Option<(usize, Mtx34, RightRec)>,
}

static mut B: Option<Builder> = None;
/// MELEE_SHOW_FPS: capture time and records per frame, logged every 600 frames.
static mut PROF: (u128, u64, u64, u64) = (0, 0, 0, 0);

struct Timed(Option<std::time::Instant>);
impl Timed {
    fn start() -> Self {
        static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        Timed(ON.get_or_init(|| std::env::var_os("MELEE_SHOW_FPS").is_some()).then(std::time::Instant::now))
    }
}
impl Drop for Timed {
    fn drop(&mut self) {
        if let Some(t) = self.0 {
            unsafe { PROF.0 += t.elapsed().as_nanos() };
        }
    }
}

#[inline]
pub fn on() -> bool {
    ON.load(Relaxed)
}

unsafe fn b() -> &'static mut Builder {
    B.get_or_insert_with(Builder::default)
}

/// The frame recorded since the last call (at the end of a rendered frame).
pub fn take() -> Option<PoseFrame> {
    unsafe {
        let b = B.as_mut()?;
        b.joints.clear();
        b.cams.clear();
        b.cur_cam = 0;
        b.last_right = None;
        // the next frame starts with this one's capacity
        let next = PoseFrame {
            cams: std::vec::Vec::with_capacity(b.f.cams.len()),
            joints: std::vec::Vec::with_capacity(b.f.joints.len()),
            slots: std::vec::Vec::with_capacity(b.f.slots.len()),
            bones: std::vec::Vec::with_capacity(b.f.bones.len()),
            declined: 0,
        };
        let f = std::mem::replace(&mut b.f, next);
        PROF.1 += 1;
        PROF.2 += f.joints.len() as u64;
        PROF.3 += f.slots.len() as u64;
        if PROF.1 % 600 == 0 && std::env::var_os("MELEE_SHOW_FPS").is_some() {
            eprintln!("gw: perf: pose capture {:.3} ms/frame, {:.0} joints, {:.0} slots per frame (last 600)", PROF.0 as f64 / 6e8, PROF.2 as f64 / 600.0, PROF.3 as f64 / 600.0);
            PROF = (0, PROF.1, 0, 0);
        }
        (!f.is_empty()).then_some(f)
    }
}

/// Drop whatever was recorded (a frame that is not drawn).
pub fn discard() {
    let _ = take();
}

fn m(x: &Mtx) -> Mtx34 {
    *x
}

/// `HSD_CObjSetCurrent` succeeded for `cobj` (after `HSD_CObjSetupViewingMtx`).
pub unsafe fn camera(cobj: *mut HSD_CObj) {
    if !on() || cobj.is_null() {
        return;
    }
    let b = b();
    let key = cobj as usize;
    let id = match b.cams.get(&key) {
        Some(&i) if b.f.cams[i as usize - 1].view == (*cobj).view_mtx => i,
        _ => {
            b.f.cams.push(CamRec { key, view: m(&(*cobj).view_mtx), perspective: (*cobj).projection_type == 1 });
            let i = b.f.cams.len() as u32;
            b.cams.insert(key, i);
            i
        }
    };
    b.cur_cam = id;
    if let Some(f) = CAMERA {
        f(id);
    }
}

/// The camera index for a draw whose view matrix is `v` (the current camera's, or a new record for another view).
unsafe fn cam_for(b: &mut Builder, v: &Mtx34) -> u32 {
    if b.cur_cam != 0 && b.f.cams[b.cur_cam as usize - 1].view == *v {
        return b.cur_cam;
    }
    // a view the game made itself (HSD_JObjDispAll's instance path): its own record, keyed by value
    let key = 1usize << 63 | (v[0][3].to_bits() as usize) << 32 | v[2][3].to_bits() as usize;
    if let Some(&i) = b.cams.get(&key) {
        if b.f.cams[i as usize - 1].view == *v {
            return i;
        }
    }
    b.f.cams.push(CamRec { key, view: *v, perspective: true });
    let i = b.f.cams.len() as u32;
    b.cams.insert(key, i);
    i
}

const MTX_FLAGS_OPAQUE: u32 = crate::jobj::JOBJ_JOINT
    | crate::jobj::JOBJ_USER_DEF_MTX
    | crate::jobj::JOBJ_MTX_INDEP_PARENT
    | crate::jobj::JOBJ_MEX_SCALE_COMPENSATE;

fn v3(v: &Vec) -> [f32; 3] {
    [v.x, v.y, v.z]
}

/// The joint record of `j` (and its ancestors), 0-based.
unsafe fn joint(b: &mut Builder, j: *mut HSD_JObj) -> u32 {
    if let Some(&i) = b.joints.get(&(j as usize)) {
        return i;
    }
    let par = (*j).parent;
    let parent = if par.is_null() { u32::MAX } else { joint(b, par) };
    let fl = (*j).flags;
    let srt = [(*j).rotate.x, (*j).rotate.y, (*j).rotate.z, (*j).translate.x, (*j).translate.y, (*j).translate.z, (*j).scale.x, (*j).scale.y, (*j).scale.z];
    let quat = [(*j).rotate.x, (*j).rotate.y, (*j).rotate.z, (*j).rotate.w];
    let pscl = if par.is_null() || (*par).scl.is_null() { None } else { Some(v3(&*(*par).scl)) };
    let world = m(&(*j).mtx);
    let targeted = !(*j).aobj.is_null() && !(*(*j).aobj).jobj.is_null();
    let mut kind = if fl & MTX_FLAGS_OPAQUE != 0 || !(*j).robj.is_null() || targeted {
        JointKind::Opaque
    } else if fl & crate::jobj::JOBJ_USE_QUATERNION != 0 {
        JointKind::SrtQuat
    } else {
        JointKind::Srt
    };
    if kind != JointKind::Opaque {
        // the build must be exactly the game's (HSD_JObjMakeMatrix): otherwise something else set this matrix
        use crate::mtx::{Quaternion, Vec3};
        let vv = |a: [f32; 3]| Vec3 { x: a[0], y: a[1], z: a[2] };
        let ps = pscl.map(vv);
        let local = if kind == JointKind::SrtQuat {
            crate::hsd_mtx::srt_quat(&vv([srt[6], srt[7], srt[8]]), &Quaternion { x: quat[0], y: quat[1], z: quat[2], w: quat[3] }, &vv([srt[3], srt[4], srt[5]]), ps.as_ref())
        } else {
            crate::hsd_mtx::srt(&vv([srt[6], srt[7], srt[8]]), &vv([srt[0], srt[1], srt[2]]), &vv([srt[3], srt[4], srt[5]]), ps.as_ref())
        };
        let w = if par.is_null() { local } else { crate::mtx::concat(&(*par).mtx, &local) };
        if max_diff(&w, &world) > 1e-4 * (1.0 + world[0][3].abs().max(world[1][3].abs()).max(world[2][3].abs())) {
            kind = JointKind::Opaque;
        }
    }
    let (authored, samples) = if kind == JointKind::Opaque { (0, None) } else { sample_joint(j, &srt) };
    if kind != JointKind::Opaque && authored == 0 && !(*j).aobj.is_null() {
        b.f.declined += 1;
    }
    b.f.joints.push(JointRec { key: j as usize, parent, kind, srt, quat, pscl, world, authored, samples });
    let i = b.f.joints.len() as u32 - 1;
    b.joints.insert(j as usize, i);
    i
}

/// A tagged slot: emit the tag (1-based) for GX matrix id `id`.
unsafe fn emit(b: &mut Builder, id: u32, rec: SlotRec) {
    b.f.slots.push(rec);
    if let Some(f) = TAG {
        f(id / 3, b.f.slots.len() as u32);
    }
}

fn close(a: &Mtx34, b: &Mtx34) -> bool {
    let s = 1.0 + a.iter().flatten().fold(0.0f32, |x, v| x.max(v.abs()));
    max_diff(a, b) <= 1e-4 * s
}

/// `SetupRigidModelMtx` / `SetupSharedVtxModelMtx` loaded `loaded` (= `view * j->mtx` normally) into GX matrix `id`.
pub unsafe fn rigid(id: u32, view: *const [f32; 4], loaded: *const [f32; 4], j: *mut HSD_JObj) {
    let (view, loaded) = (view as *const Mtx, loaded as *const Mtx);
    if !on() || j.is_null() || view.is_null() || loaded.is_null() {
        return;
    }
    let _t = Timed::start();
    let b = b();
    let v = *view;
    // a billboard / anything that is not view * world stays untagged (the presenter only moves it with the camera)
    if !close(&concat(&v, &(*j).mtx), &*loaded) {
        return;
    }
    let cam = cam_for(b, &v);
    let joint = joint(b, j);
    emit(b, id, SlotRec::Rigid { cam, joint });
}

/// `SetupEnvelopeModelMtx` loaded `loaded` into GX matrix `id` from envelope list `e` (`single`: a one-bone slot with
/// weight 1), with the right-hand node matrix `right` (null: none) that `_HSD_mkEnvelopeModelNodeMtx(cur)` made.
pub unsafe fn envelope(id: u32, view: *const [f32; 4], loaded: *const [f32; 4], mut e: *mut HSD_Envelope, cur: *mut HSD_JObj, right: *const [f32; 4]) {
    let (view, loaded, right) = (view as *const Mtx, loaded as *const Mtx, right as *const Mtx);
    if !on() || e.is_null() || view.is_null() || loaded.is_null() {
        return;
    }
    let _t = Timed::start();
    let b = b();
    let v = *view;
    let single = (*e).weight >= 1.0 - f32::EPSILON;
    let start = b.f.bones.len() as u32;
    let mut sum = [[0.0f32; 4]; 3];
    while !e.is_null() {
        let j = (*e).jobj;
        if j.is_null() {
            b.f.bones.truncate(start as usize);
            return;
        }
        let use_env = !(single && right.is_null());
        if use_env && (*j).envelopemtx.is_null() {
            b.f.bones.truncate(start as usize);
            return;
        }
        let env = use_env.then(|| *((*j).envelopemtx as *const Mtx34));
        let w = if single { 1.0 } else { (*e).weight };
        let jw = &(*j).mtx;
        if right.is_null() {
            // the check below needs only the translation column of world * env
            for r in 0..3 {
                let t = match &env {
                    Some(x) => jw[r][0] * x[0][3] + jw[r][1] * x[1][3] + jw[r][2] * x[2][3] + jw[r][3],
                    None => jw[r][3],
                };
                sum[r][3] += t * w;
            }
        } else {
            let jm = env.map_or(*jw, |x| concat(jw, &x));
            for r in 0..3 {
                for c in 0..4 {
                    sum[r][c] += jm[r][c] * w;
                }
            }
        }
        let ji = joint(b, j);
        b.f.bones.push(BoneRec { joint: ji, weight: w, envelope: env });
        if single {
            break;
        }
        e = (*e).next;
    }
    let mut rr = RightRec::None;
    if !right.is_null() {
        let r = *right;
        sum = concat(&sum, &r);
        // which of _HSD_mkEnvelopeModelNodeMtx's three forms made it (displayfunc.c): rebuilt from the joints later
        // (the same for every slot of one joint's pobjs: cached per joint)
        rr = match &b.last_right {
            Some((k, m, x)) if *k == cur as usize && *m == r => x.clone(),
            _ => {
                let x = crate::tu::displayfunc_native::HSD_JObjFindSkeleton(cur);
                let mut rr = RightRec::Fixed(r);
                if !x.is_null() && x != cur {
                    let (xi, mi) = (joint(b, x), joint(b, cur));
                    if (*x).flags & crate::jobj::JOBJ_SKELETON_ROOT != 0 {
                        rr = RightRec::InvWorld { x: xi, m: mi };
                    } else if !(*x).envelopemtx.is_null() {
                        rr = RightRec::InvWorldEnv { x: xi, env: *((*x).envelopemtx as *const Mtx34), m: mi };
                    }
                }
                b.last_right = Some((cur as usize, r, rr.clone()));
                rr
            }
        };
    }
    let ok = if right.is_null() {
        let l = &*loaded;
        let s3 = 1.0 + l[0][3].abs().max(l[1][3].abs()).max(l[2][3].abs());
        (0..3).all(|r| (v[r][0] * sum[0][3] + v[r][1] * sum[1][3] + v[r][2] * sum[2][3] + v[r][3] - l[r][3]).abs() <= 1e-4 * s3)
    } else {
        close(&concat(&v, &sum), &*loaded)
    };
    if !ok {
        b.f.bones.truncate(start as usize);
        return;
    }
    let cam = cam_for(b, &v);
    let end = b.f.bones.len() as u32;
    emit(b, id, SlotRec::Skinned { cam, bones: start..end, right: rr });
}

// ---------------------------------------------------------------- authored channels

/// The joint's channels that follow its own animation over the next tick, sampled at SAMPLES + 1 phases.
unsafe fn sample_joint(j: *mut HSD_JObj, cur: &[f32; 9]) -> (u16, Option<Box<[[f32; 9]; SAMPLES + 1]>>) {
    let a = (*j).aobj;
    if a.is_null() {
        return (0, None);
    }
    use crate::aobj::{AOBJ_FIRST_PLAY, AOBJ_NO_ANIM, AOBJ_NO_UPDATE};
    let fl = (*a).flags;
    if fl & (AOBJ_NO_ANIM | AOBJ_NO_UPDATE) != 0 {
        return (0, None);
    }
    // HSD_AObjInterpretAnim's next call: curr_frame += framerate (0 on the first play), then a loop rewinds or the
    // end stops it. Only a tick that stays inside the animation is sampled.
    let rate = if fl & AOBJ_FIRST_PLAY != 0 { 0.0 } else { (*a).framerate };
    if !((*a).curr_frame + rate < (*a).end_frame) {
        return (0, None);
    }
    let mut mask = 0u16;
    let mut bad = 0u16;
    let mut out = Box::new([*cur; SAMPLES + 1]);
    let mut f = (*a).fobj;
    let mut n = 0;
    while !f.is_null() && n < 64 {
        n += 1;
        let ty = (*f).obj_type as u32;
        let c = match ty {
            1..=3 => Some(ty as usize - 1),
            5..=10 => Some(ty as usize - 2),
            4 => {
                bad |= 0x38; // a path drives the translation: not sampled
                None
            }
            _ => None,
        };
        if let Some(c) = c {
            let clamp = |v: f32| if c >= 6 && v.abs() < 1e-3 { 1e-3 } else { v };
            // phase 0 must be exactly where the channel's own track put it this tick (else something else wrote it)
            let mut ok = true;
            for k in 0..=SAMPLES {
                let mut z = core::ptr::read(f);
                match eval(&mut z, rate * k as f32 / SAMPLES as f32) {
                    Some(v) => out[k][c] = clamp(v),
                    None if k == 0 => ok = false,
                    None => out[k][c] = out[k - 1][c],
                }
                if k == 0 && !(ok && out[0][c].to_bits() == cur[c].to_bits()) {
                    ok = false;
                    out[0][c] = cur[c];
                    break;
                }
            }
            if ok {
                mask |= 1 << c;
            } else {
                bad |= 1 << c;
                for k in 0..=SAMPLES {
                    out[k][c] = cur[c];
                }
            }
        }
        f = (*f).next;
    }
    let m = mask & !bad & 0x1FF;
    (m, (m != 0).then_some(out))
}

// fobj.c's keyframe interpreter over a private copy (the game's FObj is never touched)
const OP_CON: u8 = 1;
const OP_LIN: u8 = 2;
const OP_SPL0: u8 = 3;
const OP_SPL: u8 = 4;
const OP_SLP: u8 = 5;
const OP_KEY: u8 = 6;

unsafe fn rd(p: &mut *mut u8) -> u8 {
    let b = **p;
    *p = p.add(1);
    b
}

unsafe fn parse_float(p: &mut *mut u8, frac: u8) -> f32 {
    if frac & 0xE0 == 0 {
        let d = rd(p) as u32 | (rd(p) as u32) << 8 | (rd(p) as u32) << 16 | (rd(p) as u32) << 24;
        return f32::from_bits(d);
    }
    let denom = (1i32 << (frac & 0x1F)) as f32;
    let n = match frac & 0xE0 {
        0x60 => rd(p) as i8 as f32,
        0x80 => rd(p) as f32,
        0x20 => {
            let lo = rd(p) as i32;
            let hi = rd(p) as i8 as i32;
            (hi << 8 | lo) as f32
        }
        0x40 => {
            let lo = rd(p) as i32;
            let hi = rd(p) as i32;
            (hi << 8 | lo) as f32
        }
        _ => return 0.0,
    };
    n / denom
}

unsafe fn pack_info(p: &mut *mut u8) -> u32 {
    let mut d = rd(p);
    let mut n = ((d >> 4) & 7) as u32 + 1;
    let mut shift = 3u32;
    while d & 0x80 != 0 {
        d = rd(p);
        n = n.wrapping_add(((d & 0x7F) as u32).wrapping_shl(shift));
        shift += 7;
    }
    n
}

unsafe fn wait(p: &mut *mut u8) -> u32 {
    let (mut w, mut shift) = (0u32, 0u32);
    loop {
        let d = rd(p);
        w |= ((d & 0x7F) as u32).wrapping_shl(shift);
        shift += 7;
        if d & 0x80 == 0 {
            return w;
        }
    }
}

fn state(f: &HSD_FObj) -> u32 {
    (f.flags & 0xF) as u32
}
fn set_state(f: &mut HSD_FObj, s: u32) -> u32 {
    f.flags = (s & 0xF) as u8 | (f.flags & 0xF0);
    s
}
fn launch_key(f: &mut HSD_FObj) {
    if f.flags & 0x40 != 0 {
        f.op_intrp = f.op;
        f.flags &= !0x40;
        f.flags |= 0x80;
        f.p0 = f.p1;
    }
}
unsafe fn consumed(f: &HSD_FObj) -> bool {
    f.ad.offset_from(f.ad_head) as u32 >= f.length
}

unsafe fn load_data(f: &mut HSD_FObj) -> u32 {
    if consumed(f) {
        return 6;
    }
    f.op_intrp = f.op;
    if f.nb_pack == 0 {
        f.op = *f.ad & 0xF;
        f.nb_pack = pack_info(&mut f.ad) as u16;
    }
    f.nb_pack = f.nb_pack.wrapping_sub(1);
    let st = state(f);
    let next = if st == 1 { 3 } else { 4 };
    match f.op {
        OP_CON | OP_LIN => {
            f.p0 = f.p1;
            f.p1 = parse_float(&mut f.ad, f.frac_value);
            if f.op_intrp != OP_SLP {
                f.d0 = f.d1;
                f.d1 = 0.0;
            }
            set_state(f, next)
        }
        OP_SPL0 => {
            f.p0 = f.p1;
            f.d0 = f.d1;
            f.p1 = parse_float(&mut f.ad, f.frac_value);
            f.d1 = 0.0;
            set_state(f, next)
        }
        OP_SPL => {
            f.p0 = f.p1;
            f.p1 = parse_float(&mut f.ad, f.frac_value);
            f.d0 = f.d1;
            f.d1 = parse_float(&mut f.ad, f.frac_slope);
            set_state(f, next)
        }
        OP_SLP => {
            f.d0 = f.d1;
            f.d1 = parse_float(&mut f.ad, f.frac_slope);
            st
        }
        OP_KEY => {
            launch_key(f);
            f.p1 = parse_float(&mut f.ad, f.frac_value);
            f.flags |= 0x40;
            set_state(f, next)
        }
        _ => 0,
    }
}

unsafe fn load_wait(f: &mut HSD_FObj) -> u32 {
    if consumed(f) {
        6
    } else {
        f.fterm = wait(&mut f.ad) as u16;
        f.flags |= 0x20;
        set_state(f, 2)
    }
}

/// `FObjUpdateAnim`'s value (None: no update).
fn value(f: &mut HSD_FObj) -> Option<f32> {
    use melee_core::fp::fmadds;
    match f.op_intrp {
        OP_KEY => {
            if f.flags & 0x80 != 0 {
                f.flags &= 0x7F;
                Some(f.p0)
            } else {
                None
            }
        }
        OP_CON => Some(if f.time >= f.fterm as f32 { f.p1 } else { f.p0 }),
        OP_LIN => {
            if f.flags & 0x20 != 0 {
                f.flags &= 0xDF;
                if f.fterm != 0 {
                    f.d0 = (f.p1 - f.p0) / f.fterm as f32;
                } else {
                    f.d0 = 0.0;
                    f.p0 = f.p1;
                }
            }
            Some(fmadds(f.d0, f.time, f.p0))
        }
        OP_SPL0 | OP_SPL | OP_SLP => Some(if f.fterm != 0 {
            crate::spline::get_helmite((1.0f64 / f.fterm as f64) as f32, f.time, f.p0, f.p1, f.d0, f.d1)
        } else {
            f.p1
        }),
        _ => None,
    }
}

/// `HSD_FObjInterpretAnim(f, rate)` on a copy: the last value it would hand the joint (None: none).
unsafe fn eval(f: &mut HSD_FObj, rate: f32) -> Option<f32> {
    let mut out = None;
    let mut st = state(f);
    if st == 0 {
        return None;
    }
    f.time += rate;
    if (f.time as f64) < 0.0 {
        return None;
    }
    let mut fterm = 0.0f32;
    for _ in 0..4096 {
        match st {
            6 => {
                f.time += fterm;
                launch_key(f);
                if let Some(v) = value(f) {
                    out = Some(v);
                }
                return out;
            }
            1 | 2 => st = load_data(f),
            3 => {
                if f.flags & 0x80 != 0 {
                    if let Some(v) = value(f) {
                        out = Some(v);
                    }
                }
                st = load_wait(f);
            }
            4 => {
                if f.fterm as f32 <= f.time {
                    st = 3;
                    fterm = f.fterm as f32;
                    f.time -= f.fterm as f32;
                    set_state(f, st);
                    continue;
                }
                if let Some(v) = value(f) {
                    out = Some(v);
                }
                set_state(f, 5);
                return out;
            }
            5 => {
                st = 4;
                set_state(f, st);
            }
            _ => return out,
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A linear track 0 -> 10 over 10 frames: the copy's interpreter follows it and the original is untouched.
    #[test]
    fn linear_track_samples() {
        // op LIN, 2 keys packed: nb_pack byte = (2-1)<<4 | op; values as f32 (frac 0), waits 10
        let mut data: std::vec::Vec<u8> = vec![0x12];
        data.extend_from_slice(&0.0f32.to_bits().to_le_bytes());
        data.push(10);
        data.extend_from_slice(&10.0f32.to_bits().to_le_bytes());
        data.push(10);
        let mut f: HSD_FObj = unsafe { core::mem::zeroed() };
        f.ad = data.as_mut_ptr();
        f.ad_head = data.as_mut_ptr();
        f.length = data.len() as u32;
        f.obj_type = 5;
        set_state(&mut f, 1);
        unsafe {
            // the game's first interpret at rate 0 lands at frame 0
            let v = eval(&mut f, 0.0);
            assert_eq!(v, Some(0.0));
            let mut c = core::ptr::read(&f);
            assert_eq!(eval(&mut c, 2.5), Some(2.5));
            let mut c = core::ptr::read(&f);
            assert_eq!(eval(&mut c, 0.0), Some(0.0));
        }
    }
}
