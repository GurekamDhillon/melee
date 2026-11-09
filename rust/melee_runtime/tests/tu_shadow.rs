//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/shadow.c.
//! src/tu/shadow.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py shadow`).
//! C functions of the TU this test covers (the private ones through their public callers): HSD_ShadowAddObject, HSD_ShadowAlloc, HSD_ShadowDeleteObject, HSD_ShadowEndRender, HSD_ShadowGetAllocData, HSD_ShadowInit, HSD_ShadowInitAllocData, HSD_ShadowRemove, HSD_ShadowSetActive, HSD_ShadowSetSize, HSD_ShadowSetViewingRect, HSD_ShadowStartRender, HSD_ViewingRectAddRect, HSD_ViewingRectCheck, HSD_ViewingRectInit, drawBackgroundRect, makeMatrix, makeShadowTObj.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_HSD_ShadowGetAllocData: u32 = 0;
const OP_HSD_ShadowInitAllocData: u32 = 1;
const OP_makeShadowTObj: u32 = 2;
const OP_HSD_ShadowAlloc: u32 = 3;
const OP_HSD_ShadowRemove: u32 = 4;
const OP_HSD_ShadowInit: u32 = 5;
const OP_HSD_ShadowSetSize: u32 = 6;
const OP_drawBackgroundRect: u32 = 7;
const OP_HSD_ShadowStartRender: u32 = 8;
const OP_HSD_ShadowEndRender: u32 = 9;
const OP_HSD_ShadowSetActive: u32 = 10;
const OP_HSD_ShadowAddObject: u32 = 11;
const OP_HSD_ShadowDeleteObject: u32 = 12;
const OP_makeMatrix: u32 = 13;
const OP_HSD_ShadowSetViewingRect: u32 = 14;
const OP_HSD_ViewingRectInit: u32 = 15;
const OP_HSD_ViewingRectCheck: u32 = 16;
const OP_HSD_ViewingRectAddRect: u32 = 17;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::shadow as m;
use mtu_harness::*;
#[path = "mtu/shadow_gen.rs"]
mod g;


unsafe extern "C" fn cb_rel(c: *mut HSD_Class) { t(8203); t(pi(c)); }
unsafe extern "C" fn cb_dest(c: *mut HSD_Class) { t(8204); t(pi(c)); }


pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut PPA: [*mut c_void; 4] = [core::ptr::null_mut(); 4];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub static mut P0: [HSD_Shadow; 2] = unsafe { core::mem::zeroed() };
pub static mut P1: [HSD_JObj; 2] = unsafe { core::mem::zeroed() };
pub static mut P2: [HSD_ViewingRect; 2] = unsafe { core::mem::zeroed() };
pub static mut CO: [HSD_CObj; 2] = unsafe { core::mem::zeroed() };
pub static mut TO: [HSD_TObj; 2] = unsafe { core::mem::zeroed() };
pub static mut IM: [HSD_ImageDesc; 2] = unsafe { core::mem::zeroed() };
pub static mut SL: [HSD_SList; 3] = unsafe { core::mem::zeroed() };
pub static mut CI: [HSD_ClassInfo; 1] = unsafe { core::mem::zeroed() };
pub fn pick_HSD_Shadow() -> *mut HSD_Shadow { let r = rnd(); unsafe { &raw mut P0[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_Shadow() -> *mut HSD_Shadow { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut P0[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_JObj() -> *mut HSD_JObj { let r = rnd(); unsafe { &raw mut P1[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_JObj() -> *mut HSD_JObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut P1[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_ViewingRect() -> *mut HSD_ViewingRect { let r = rnd(); unsafe { &raw mut P2[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_ViewingRect() -> *mut HSD_ViewingRect { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut P2[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_CObj() -> *mut HSD_CObj { let r = rnd(); unsafe { &raw mut CO[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_CObj() -> *mut HSD_CObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut CO[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_TObj() -> *mut HSD_TObj { let r = rnd(); unsafe { &raw mut TO[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_TObj() -> *mut HSD_TObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut TO[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_ImageDesc() -> *mut HSD_ImageDesc { let r = rnd(); unsafe { &raw mut IM[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_ImageDesc() -> *mut HSD_ImageDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut IM[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_SList() -> *mut HSD_SList { let r = rnd(); unsafe { &raw mut SL[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_SList() -> *mut HSD_SList { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut SL[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_ClassInfo() -> *mut HSD_ClassInfo { let r = rnd(); unsafe { &raw mut CI[((r >> 8) % 1) as usize] } }
pub fn picko_HSD_ClassInfo() -> *mut HSD_ClassInfo { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut CI[((r >> 8) % 1) as usize] } } }
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
        preg(&raw const P0, core::mem::size_of_val(&P0));
        preg(&raw const P1, core::mem::size_of_val(&P1));
        preg(&raw const P2, core::mem::size_of_val(&P2));
        preg(&raw const CO, core::mem::size_of_val(&CO));
        preg(&raw const TO, core::mem::size_of_val(&TO));
        preg(&raw const IM, core::mem::size_of_val(&IM));
        preg(&raw const SL, core::mem::size_of_val(&SL));
        preg(&raw const CI, core::mem::size_of_val(&CI));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS)); preg(&raw const PPA, core::mem::size_of_val(&PPA));
    }
}

pub unsafe fn setup_world() {
    P0 = core::mem::zeroed();
    P1 = core::mem::zeroed();
    P2 = core::mem::zeroed();
    CO = core::mem::zeroed();
    TO = core::mem::zeroed();
    IM = core::mem::zeroed();
    SL = core::mem::zeroed();
    CI = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..4 { PPA[i] = core::ptr::null_mut(); }
    for i in 0..2 { g::fill_HSD_Shadow(&mut P0[i]); }
    for i in 0..2 { g::fill_HSD_JObj(&mut P1[i]); }
    for i in 0..2 { g::fill_HSD_ViewingRect(&mut P2[i]); }
    for i in 0..2 { g::fill_HSD_CObj(&mut CO[i]); }
    for i in 0..2 { g::fill_HSD_TObj(&mut TO[i]); }
    for i in 0..2 { g::fill_HSD_ImageDesc(&mut IM[i]); }
    for i in 0..3 { g::fill_HSD_SList(&mut SL[i]); }
    for i in 0..1 { g::fill_HSD_ClassInfo(&mut CI[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..2 { g::hash_HSD_Shadow(&P0[i]); }
    for i in 0..2 { g::hash_HSD_JObj(&P1[i]); }
    for i in 0..2 { g::hash_HSD_ViewingRect(&P2[i]); }
    for i in 0..2 { g::hash_HSD_CObj(&CO[i]); }
    for i in 0..2 { g::hash_HSD_TObj(&TO[i]); }
    for i in 0..2 { g::hash_HSD_ImageDesc(&IM[i]); }
    for i in 0..3 { g::hash_HSD_SList(&SL[i]); }
    for i in 0..1 { g::hash_HSD_ClassInfo(&CI[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    for i in 0..4 { hm(pi(PPA[i])); }
    g::hashg_all();
    g::hasho_all();
}

// harness = false (Cargo.toml): a case that ends in a panic is cut off inside the hook (see mtu_harness::run_case),
// which a libtest main thread cannot survive at exit.
fn main() { shadow_matches_c(); println!("tu_shadow: ok"); }

fn shadow_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();
freg(cb_rel as usize); freg(cb_dest as usize); m::use_ext_data = true; melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());
        install_abort_hook();
        let mut c = Check::new(include_str!("data/mtu_shadow.txt"));
        let mut total = 0u32;
        for op in 0..18u32 {
            let n = match op {
                0 => 300u32,
                1 => 300u32,
                2 => 300u32,
                3 => 300u32,
                4 => 300u32,
                5 => 300u32,
                6 => 300u32,
                7 => 300u32,
                8 => 300u32,
                9 => 300u32,
                10 => 300u32,
                11 => 300u32,
                12 => 300u32,
                13 => 300u32,
                14 => 300u32,
                15 => 300u32,
                16 => 300u32,
                17 => 300u32,
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
{ CI[0].release = Some(cb_rel); CI[0].destroy = Some(cb_dest); for q in 0..2usize { CO[q].parent.parent.class_info = &raw mut CI[0]; } for q in 0..3usize { SL[q].next = if q < 2 && fv() & 1 != 0 { &raw mut SL[q + 1] } else { null_mut() }; SL[q].data = (&raw mut P1[(fv() & 1) as usize]) as *mut c_void; } for q in 0..2usize { P0[q].camera = &raw mut CO[q]; P0[q].texture = &raw mut TO[q]; TO[q].imagedesc = &raw mut IM[q]; P0[q].objects = if fv() & 1 != 0 { &raw mut SL[(fv() % 3) as usize] } else { null_mut() }; IM[q].image_ptr = if fv() & 1 != 0 { (&raw mut BUF) as *mut u8 } else { null_mut() }; IM[q].width = (fv() % 300) as u16; IM[q].height = (fv() % 300) as u16; } }
                let mut ret = 0u32;
                let aborted = run_case(|| unsafe { match op {
                    0 => { 

ret = pi(m::HSD_ShadowGetAllocData());
 }
                    1 => { 

m::HSD_ShadowInitAllocData();
 }
                    2 => { 

ret = pi(m::makeShadowTObj());
 }
                    3 => { 

ret = pi(m::HSD_ShadowAlloc());
 }
                    4 => { 
let a0 = pick_HSD_Shadow();
m::HSD_ShadowRemove(a0);
 }
                    5 => { 
let a0 = pick_HSD_Shadow();
m::HSD_ShadowInit(a0);
 }
                    6 => { 
let a0 = pick_HSD_Shadow();
let a1 = fv() as _;
let a2 = fv() as _;
m::HSD_ShadowSetSize(a0, a1, a2);
 }
                    7 => { 
let a0 = pick_HSD_Shadow();
m::drawBackgroundRect(a0);
 }
                    8 => { 
let a0 = pick_HSD_Shadow();
m::HSD_ShadowStartRender(a0);
 }
                    9 => { 
let a0 = pick_HSD_Shadow();
m::HSD_ShadowEndRender(a0);
 }
                    10 => { 
let a0 = pick_HSD_Shadow();
let a1 = fv() as _;
m::HSD_ShadowSetActive(a0, a1);
 }
                    11 => { 
let a0 = pick_HSD_Shadow();
let a1 = pick_HSD_JObj();
m::HSD_ShadowAddObject(a0, a1);
 }
                    12 => { 
let a0 = pick_HSD_Shadow();
let a1 = pick_HSD_JObj();
m::HSD_ShadowDeleteObject(a0, a1);
 }
                    13 => { 
let a0 = pick_HSD_Shadow();
m::makeMatrix(a0);
 }
                    14 => { 
let a0 = pick_HSD_Shadow();
let a1 = rw();
let a2 = rw();
let a3 = rw();
let a4 = rw();
m::HSD_ShadowSetViewingRect(a0, a1, a2, a3, a4);
 }
                    15 => { 
let a0 = pick_HSD_ViewingRect();
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = &raw mut VA[(rnd() % 4) as usize];
let a3 = &raw mut VA[(rnd() % 4) as usize];
let a4 = fv() as _;
m::HSD_ViewingRectInit(a0, a1, a2, a3, a4);
 }
                    16 => { 
let a0 = pick_HSD_ViewingRect();
ret = m::HSD_ViewingRectCheck(a0) as u32;
 }
                    17 => { 
let a0 = pick_HSD_ViewingRect();
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = rw();
let a3 = rw();
let a4 = rw();
let a5 = rw();
m::HSD_ViewingRectAddRect(a0, a1, a2, a3, a4, a5);
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
