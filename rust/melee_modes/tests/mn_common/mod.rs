//! Builders shared by the mn/ TU state-oracle tests (`#[path = "mn_common/mod.rs"] mod mn;`).
#![allow(dead_code)]
use core::ffi::c_void;
use core::mem::offset_of;
use core::ptr::null_mut;
use melee_modes::oracle::*;
use melee_types::*;

pub fn pick<T: Copy>(c: &mut Case, xs: &[T]) -> T {
    xs[c.rng.range(0, xs.len() as i64 - 1) as usize]
}

/// A GObj with the given user data and JObj (links null, other scalars random).
pub fn gobj(c: &mut Case, ud: *mut c_void, obj: *mut c_void) -> *mut HSD_GObj {
    let g = c.obj::<HSD_GObj>();
    unsafe {
        (*g).next = null_mut();
        (*g).prev = null_mut();
        (*g).next_gx = null_mut();
        (*g).prev_gx = null_mut();
        (*g).proc = null_mut();
        (*g).user_data = ud;
        (*g).x34_unk = null_mut();
        (*g).hsd_obj = obj;
    }
    g
}

pub fn jobj(c: &mut Case) -> *mut HSD_JObj {
    let j = c.obj::<HSD_JObj>();
    unsafe {
        (*j).next = null_mut();
        (*j).parent = null_mut();
        (*j).child = null_mut();
        (*j).aobj = null_mut();
        (*j).robj = null_mut();
        (*j).scl = null_mut();
        (*j).flags &= !(JOBJ_USE_QUATERNION as u32);
    }
    j
}

/// A text object with sane scalars.
pub fn text(c: &mut Case) -> *mut HSD_Text {
    let t = c.zobj::<HSD_Text>();
    unsafe {
        (*t).pos_x = c.rng.f32_in(-10.0, 10.0);
        (*t).pos_y = c.rng.f32_in(-10.0, 10.0);
        (*t).font_size.x = c.rng.f32_in(0.0, 1.0);
        (*t).font_size.y = c.rng.f32_in(0.0, 1.0);
    }
    t
}

/// `n` procs for `HSD_GObj_SetupProc` (bitfield byte registered for the x86 comparison).
pub fn procs(c: &mut Case, n: usize) -> std::vec::Vec<*mut HSD_GObjProc> {
    let mut v = std::vec::Vec::new();
    for _ in 0..n {
        let p = c.obj::<HSD_GObjProc>();
        c.bitfields(p, offset_of!(HSD_GObjProc, bf_d), 0xD, 1);
        v.push(p);
    }
    c.ret_seq("HSD_GObj_SetupProc", &v);
    v
}

/// `n` fresh jobjs handed out one by one through the out-parameter of lb_80011E24.
pub fn lb_children(c: &mut Case, n: usize) -> std::vec::Vec<*mut HSD_JObj> {
    let mut v = std::vec::Vec::new();
    for _ in 0..n {
        v.push(jobj(c));
    }
    c.out_ptrs("lb_80011E24", 1, &v);
    v
}
