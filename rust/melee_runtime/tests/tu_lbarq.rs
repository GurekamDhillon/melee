//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/lbarq.c.
//! src/tu/lbarq.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py lbarq`).
//! C functions of the TU this test covers (the private ones through their public callers): lbArq_80014ABC, lbArq_80014AC4, lbArq_80014BD0, lbArq_80014D2C.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_lbArq_80014ABC: u32 = 0;
const OP_lbArq_80014AC4: u32 = 1;
const OP_lbArq_80014BD0: u32 = 2;
const OP_lbArq_80014D2C: u32 = 3;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::lbarq as m;
use mtu_harness::*;
#[path = "mtu/lbarq_gen.rs"]
mod g;


static mut PEND: *mut ARQRequest = null_mut();
pub unsafe extern "C" fn cbv1(a: *mut c_void) { t(7101); t(pi(a)); }
pub unsafe extern "C" fn cbv2(a: *mut c_void) { t(7102); t(pi(a)); }


pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut PPA: [*mut c_void; 4] = [core::ptr::null_mut(); 4];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub unsafe extern "C" fn cbev1(g: *mut HSD_GObj) { t(7001); t(pi(g)); }
pub unsafe extern "C" fn cbev2(g: *mut HSD_GObj) { t(7002); t(pi(g)); }
pub unsafe extern "C" fn cbp1(g: *mut HSD_GObj) -> i32 { t(7003); t(pi(g)); (fv() & 1) as i32 }
pub unsafe extern "C" fn cbp2(g: *mut HSD_GObj) -> i32 { t(7004); t(pi(g)); (fv() & 1) as i32 }
pub unsafe extern "C" fn cbev3(g: *mut HSD_GObj, h: *mut HSD_GObj) { t(7005); t(pi(g)); t(pi(h)); }
pub unsafe extern "C" fn cbi1(a: i32) { t(7006); t(a as u32); }
pub unsafe extern "C" fn cbi2(a: i32) { t(7007); t(a as u32); }
pub fn pick_fn(kind: i32) -> usize {
    let r = rnd();
    if r & 3 == 0 { return 0; }
    match kind {
        1 => if r & 4 != 0 { cbev1 as usize } else { cbev2 as usize },
        2 => if r & 4 != 0 { cbp1 as usize } else { cbp2 as usize },
        3 => cbev3 as usize,
        4 => if r & 4 != 0 { cbi1 as usize } else { cbi2 as usize },
        _ => 0,
    }
}
pub fn pick_cb(kind: i32) -> usize { let f = pick_fn(kind); if f != 0 { f } else if kind == 2 { cbp1 as usize } else if kind == 3 { cbev3 as usize } else if kind == 4 { cbi1 as usize } else { cbev1 as usize } }
pub fn freg_world() { freg(cbev1 as usize); freg(cbev2 as usize); freg(cbp1 as usize); freg(cbp2 as usize); freg(cbev3 as usize); freg(cbi1 as usize); freg(cbi2 as usize);  }
pub fn preg_world() {
    unsafe {
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS)); preg(&raw const PPA, core::mem::size_of_val(&PPA));
    }
}

pub unsafe fn setup_world() {
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..4 { PPA[i] = core::ptr::null_mut(); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    for i in 0..4 { hm(pi(PPA[i])); }
    g::hashg_all();
    g::hasho_all();
}

// harness = false (Cargo.toml): a case that ends in a panic is cut off inside the hook (see mtu_harness::run_case),
// which a libtest main thread cannot survive at exit.
fn main() { lbarq_matches_c(); println!("tu_lbarq: ok"); }

fn lbarq_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();
freg(cbv1 as *const () as usize); freg(cbv2 as *const () as usize);
        install_abort_hook();
        let mut c = Check::new(include_str!("data/mtu_lbarq.txt"));
        let mut total = 0u32;
        for op in 0..4u32 {
            let n = match op {
                0 => 300u32,
                1 => 300u32,
                2 => 300u32,
                3 => 300u32,
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
{ PEND = null_mut(); let mut last: [*mut m::lbArqNode; 3] = [null_mut(); 3];
  m::lbArq_804316C0.list = [null_mut(); 3];
  for q in 0..10usize { let n = &raw mut m::lbArq_804316C0.nodes[q]; let s = if q == 0 { 0 } else { fv() % 3 };
    (*n).state = s; (*n).next = null_mut(); if !last[s as usize].is_null() { (*last[s as usize]).next = n; } else { m::lbArq_804316C0.list[s as usize] = n; } last[s as usize] = n;
    (*n).callback = if fv() & 1 != 0 { if fv() & 1 != 0 { Some(cbv1) } else { Some(cbv2) } } else { None };
    (*n).callback_arg = if fv() & 1 != 0 { (&raw mut BUF as *mut u8).add(8 * (fv() % 8) as usize) as *mut c_void } else { null_mut() }; } }
                let mut ret = 0u32;
                let aborted = run_case(|| unsafe { match op {
                    0 => { 
let a0 = &raw mut m::lbArq_804316C0.nodes[(fv() % 10) as usize];
ret = m::lbArq_80014ABC(a0) as u32;
 }
                    1 => { 
let a0 = (&raw mut m::lbArq_804316C0.nodes[(fv() % 10) as usize].arq);
m::lbArq_80014AC4(a0);
 }
                    2 => { 
let a0 = fv() as _;
let a1 = (&raw mut BUF) as *mut c_void;
let a2 = fv() as _;
let a3 = if fv() & 1 != 0 { if fv() & 1 != 0 { Some(cbv1 as unsafe extern "C" fn(*mut c_void)) } else { Some(cbv2 as unsafe extern "C" fn(*mut c_void)) } } else { None };
let a4 = (&raw mut BUF) as *mut c_void;
m::lbArq_80014BD0(a0, a1, a2, a3, a4);
 }
                    3 => { 

m::lbArq_80014D2C();
 }
                    _ => unreachable!(),
                } });
                if aborted { c.abort(op, k); total += 1; continue; }
                hash_world();

                c.line(op, k, ret);
                total += 1;
            }
        }
        assert_eq!(c.done(), total);
    }
}
