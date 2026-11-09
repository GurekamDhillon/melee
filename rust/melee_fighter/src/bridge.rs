//! Native-pointer adapters over `melee_runtime`'s value-based baselib math (`mtx`, `hsd_mtx`): the
//! fighter TUs call `HSD_MtxInverse(src, dst)`, `PSMTXConcat(a, b, ab)` ... on the `Mtx` fields of
//! live `HSD_JObj`s / `Fighter`s, which the runtime exposes as `&Mtx -> Mtx` functions. The math
//! (and its shipping FMA contraction) is the runtime's; these only move bytes.
use crate::types::{Mtx, Vec3};
use melee_runtime::{hsd_mtx, mtx};

#[inline]
fn v(p: *const Vec3) -> mtx::Vec3 {
    unsafe { mtx::Vec3 { x: (*p).x, y: (*p).y, z: (*p).z } }
}

/// `HSD_MtxInverse(src, dst)`.
pub unsafe fn hsd_mtx_inverse(src: *mut Mtx, dst: *mut Mtx) {
    let r = hsd_mtx::inverse(&*src);
    *dst = r;
}

/// `HSD_MtxSRT(m, scale, rot, trans, pscale)` (`pscale` may be NULL).
pub unsafe fn hsd_mtx_srt(m: *mut Mtx, s: *const Vec3, r: *const Vec3, t: *const Vec3, ps: *const Vec3) {
    let ps = if ps.is_null() { None } else { Some(v(ps)) };
    *m = hsd_mtx::srt(&v(s), &v(r), &v(t), ps.as_ref());
}

/// `PSMTXConcat(a, b, ab)`.
pub unsafe fn psmtx_concat(a: *mut Mtx, b: *mut Mtx, ab: *mut Mtx) {
    let r = mtx::concat(&*a, &*b);
    *ab = r;
}

/// `PSMTXMultVec(m, src, dst)`.
pub unsafe fn psmtx_mult_vec(m: *mut Mtx, src: *const Vec3, dst: *mut Vec3) {
    let r = mtx::mult_vec(&*m, &v(src));
    (*dst).x = r.x;
    (*dst).y = r.y;
    (*dst).z = r.z;
}

/// `PSVECAdd(a, b, ab)`.
pub unsafe fn psvec_add(a: *const Vec3, b: *const Vec3, ab: *mut Vec3) {
    let (ax, ay, az) = ((*a).x, (*a).y, (*a).z);
    let (bx, by, bz) = ((*b).x, (*b).y, (*b).z);
    (*ab).x = ax + bx;
    (*ab).y = ay + by;
    (*ab).z = az + bz;
}

// ---- C callbacks into Rust-ABI function pointers ---------------------------------------------------

use crate::types::{HSD_GObj, HSD_GObjEvent};

thread_local! {
    static EV: core::cell::Cell<HSD_GObjEvent> = const { core::cell::Cell::new(None) };
}

fn ev_tramp(g: *mut HSD_GObj) {
    let f = EV.with(|c| c.get()).expect("NULL HSD_GObjEvent callback called");
    unsafe { f(g) }
}

/// Runs `f` with a Rust-ABI `fn(G)` standing for the C callback `cb`. The ported `ft_081b` functions
/// only call their callback during the call (never store it), so a trampoline reading the callback
/// from a scoped slot is exact; the slot is restored on exit so nested uses stay correct.
pub fn with_ev<R>(cb: HSD_GObjEvent, f: impl FnOnce(fn(*mut HSD_GObj)) -> R) -> R {
    let prev = EV.with(|c| c.replace(cb));
    let r = f(ev_tramp);
    EV.with(|c| c.set(prev));
    r
}

thread_local! {
    static COLL: core::cell::Cell<Option<unsafe extern "C" fn(*mut HSD_GObj, isize) -> bool>> = const { core::cell::Cell::new(None) };
}

fn coll_tramp(g: *mut HSD_GObj, i: isize) -> bool {
    let f = COLL.with(|c| c.get()).expect("NULL collision callback called");
    unsafe { f(g, i as isize) }
}

/// Like [`with_ev`] for the `(collision callback, event callback)` pair the `ft_081b` landing / cliff
/// helpers take.
pub fn with_coll_ev<R>(
    coll: unsafe extern "C" fn(*mut HSD_GObj, isize) -> bool,
    cb: HSD_GObjEvent,
    f: impl FnOnce(fn(*mut HSD_GObj, isize) -> bool, fn(*mut HSD_GObj)) -> R,
) -> R {
    let prev = COLL.with(|c| c.replace(Some(coll)));
    let r = with_ev(cb, |ev| f(coll_tramp, ev));
    COLL.with(|c| c.set(prev));
    r
}

/// `Camera_LogicToScreen(subject, point)` (cm/camera.c, `melee_world::tu::camera`): the screen position of a camera subject.
pub fn camera_logic_to_screen(s: *mut crate::types::CmSubject, out: &mut crate::types::S32Vec2) -> bool {
    unsafe { melee_world::tu::camera::Camera_LogicToScreen(s, out) != 0 }
}
