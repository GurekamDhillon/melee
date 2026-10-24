//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/video.c.
//! src/tu/video.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py video`).
//! C functions of the TU this test covers (the private ones through their public callers): HSD_VICopyEFB2XFBHiResoAA, HSD_VICopyEFB2XFBPtr, HSD_VICopyXFBAsync, HSD_VIDrawDoneXFB, HSD_VIGXDrawDoneCB, HSD_VIGXSetDrawDone, HSD_VIGetDrawDoneWaitingFlag, HSD_VIGetXFBDrawEnable, HSD_VIGetXFBLastDrawDone, HSD_VIInit, HSD_VIPostRetraceCB, HSD_VIPreRetraceCB, HSD_VISearchXFBByStatus, HSD_VISetBlack, HSD_VISetConfigure, HSD_VISetUserGXDrawDoneCallback, HSD_VISetUserPostRetraceCallback, HSD_VISetUserPreRetraceCallback, HSD_VISetXFBWaitDone, HSD_VIWaitXFBDrawEnable, HSD_VIWaitXFBFlush, HSD_VIWaitXFBFlushNoYield, HSD_VIWaitXFBFlush_sub.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_HSD_VISearchXFBByStatus: u32 = 0;
const OP_HSD_VISetUserPreRetraceCallback: u32 = 1;
const OP_HSD_VISetUserPostRetraceCallback: u32 = 2;
const OP_HSD_VISetUserGXDrawDoneCallback: u32 = 3;
const OP_HSD_VIPreRetraceCB: u32 = 4;
const OP_HSD_VIPostRetraceCB: u32 = 5;
const OP_HSD_VIGXDrawDoneCB: u32 = 6;
const OP_HSD_VIGetDrawDoneWaitingFlag: u32 = 7;
const OP_HSD_VIGetXFBDrawEnable: u32 = 8;
const OP_HSD_VIWaitXFBDrawEnable: u32 = 9;
const OP_HSD_VICopyEFB2XFBHiResoAA: u32 = 10;
const OP_HSD_VICopyEFB2XFBPtr: u32 = 11;
const OP_HSD_VIGXSetDrawDone: u32 = 12;
const OP_HSD_VISetXFBWaitDone: u32 = 13;
const OP_HSD_VICopyXFBAsync: u32 = 14;
const OP_HSD_VIDrawDoneXFB: u32 = 15;
const OP_HSD_VIWaitXFBFlush_sub: u32 = 16;
const OP_HSD_VIWaitXFBFlush: u32 = 17;
const OP_HSD_VIWaitXFBFlushNoYield: u32 = 18;
const OP_HSD_VIGetXFBLastDrawDone: u32 = 19;
const OP_HSD_VISetConfigure: u32 = 20;
const OP_HSD_VISetBlack: u32 = 21;
const OP_HSD_VIInit: u32 = 22;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::video as m;
use mtu_harness::*;
#[path = "mtu/video_gen.rs"]
mod g;

static mut XB: [u8; 1 << 20] = [0; 1 << 20];

pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut PPA: [*mut c_void; 4] = [core::ptr::null_mut(); 4];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub static mut VIS: [HSD_VIStatus; 2] = unsafe { core::mem::zeroed() };
pub static mut RMO: [GXRenderModeObj; 2] = unsafe { core::mem::zeroed() };
pub fn pick_HSD_VIStatus() -> *mut HSD_VIStatus { let r = rnd(); unsafe { &raw mut VIS[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_VIStatus() -> *mut HSD_VIStatus { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut VIS[((r >> 8) % 2) as usize] } } }
pub fn pick_GXRenderModeObj() -> *mut GXRenderModeObj { let r = rnd(); unsafe { &raw mut RMO[((r >> 8) % 2) as usize] } }
pub fn picko_GXRenderModeObj() -> *mut GXRenderModeObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut RMO[((r >> 8) % 2) as usize] } } }
pub unsafe extern "C" fn cbev1(g: *mut HSD_GObj) { t(7001); t(pi(g)); }
pub unsafe extern "C" fn cbev2(g: *mut HSD_GObj) { t(7002); t(pi(g)); }
pub unsafe extern "C" fn cbp1(g: *mut HSD_GObj) -> i32 { t(7003); t(pi(g)); (fv() & 1) as i32 }
pub unsafe extern "C" fn cbp2(g: *mut HSD_GObj) -> i32 { t(7004); t(pi(g)); (fv() & 1) as i32 }
pub unsafe extern "C" fn cbev3(g: *mut HSD_GObj, h: *mut HSD_GObj) { t(7005); t(pi(g)); t(pi(h)); }
pub unsafe extern "C" fn cbi1(a: i32) { t(7006); t(a as u32); }
pub unsafe extern "C" fn cbi2(a: i32) { t(7007); t(a as u32); }
pub unsafe extern "C" fn autocb_0(a0: u32) { t(9000); t(a0 as u32); }
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
pub fn freg_world() { freg(cbev1 as usize); freg(cbev2 as usize); freg(cbp1 as usize); freg(cbp2 as usize); freg(cbev3 as usize); freg(cbi1 as usize); freg(cbi2 as usize); freg(autocb_0 as usize); }
pub fn preg_world() {
    unsafe {
        preg(&raw const VIS, core::mem::size_of_val(&VIS));
        preg(&raw const RMO, core::mem::size_of_val(&RMO));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS)); preg(&raw const PPA, core::mem::size_of_val(&PPA));
    }
}

pub unsafe fn setup_world() {
    VIS = core::mem::zeroed();
    RMO = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..4 { PPA[i] = core::ptr::null_mut(); }
    for i in 0..2 { g::fill_HSD_VIStatus(&mut VIS[i]); }
    for i in 0..2 { g::fill_GXRenderModeObj(&mut RMO[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..2 { g::hash_HSD_VIStatus(&VIS[i]); }
    for i in 0..2 { g::hash_GXRenderModeObj(&RMO[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    for i in 0..4 { hm(pi(PPA[i])); }
    g::hashg_all();
    g::hasho_all();
}

// harness = false (Cargo.toml): a case that ends in a panic is cut off inside the hook (see mtu_harness::run_case),
// which a libtest main thread cannot survive at exit.
fn main() { video_matches_c(); println!("tu_video: ok"); }

fn video_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();
preg(&raw const XB, 1 << 20);
        install_abort_hook();
        let mut c = Check::new(include_str!("data/mtu_video.txt"));
        let mut total = 0u32;
        for op in 0..23u32 {
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
                18 => 300u32,
                19 => 300u32,
                20 => 300u32,
                21 => 300u32,
                22 => 300u32,
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
{ for q in 0..2usize { if fv() & 3 != 0 { VIS[q].rmode.fbWidth %= 200; VIS[q].rmode.efbHeight %= 100; RMO[q].fbWidth %= 200; RMO[q].efbHeight %= 100; } }
 for q in 0..3usize { m::HSD_VIData.xfb[q].status = (fv() % 9) as _; m::HSD_VIData.xfb[q].buffer = if fv() & 3 != 0 { (&raw mut BUF as *mut u8).add(8 * q) as *mut c_void } else { null_mut() }; }
 m::HSD_VIData.efb.status = (fv() % 3) as _; m::HSD_VIData.nb_xfb = if fv() & 3 != 0 { (1 + fv() % 3) as i32 } else { 0 }; m::HSD_VIData.perf.frame_period = (fv() % 5) as i32; m::HSD_VIData.drawdone.waiting = (fv() & 1) as i32; }
                let mut ret = 0u32;
                let aborted = run_case(|| unsafe { match op {
                    0 => { 
let a0 = (fv() % 9) as _;
ret = m::HSD_VISearchXFBByStatus(a0) as u32;
 }
                    1 => { 
let a0 = Some(autocb_0 as _);
ret = fi(m::HSD_VISetUserPreRetraceCallback(a0));
 }
                    2 => { 
let a0 = Some(autocb_0 as _);
ret = fi(m::HSD_VISetUserPostRetraceCallback(a0));
 }
                    3 => { 
let a0 = unsafe { core::mem::transmute::<usize, _>(pick_cb(4)) };
ret = fi(m::HSD_VISetUserGXDrawDoneCallback(a0));
 }
                    4 => { 
let a0 = fv() as _;
m::HSD_VIPreRetraceCB(a0);
 }
                    5 => { 
let a0 = fv() as _;
m::HSD_VIPostRetraceCB(a0);
 }
                    6 => { 

m::HSD_VIGXDrawDoneCB();
 }
                    7 => { 

ret = m::HSD_VIGetDrawDoneWaitingFlag() as u32;
 }
                    8 => { 

ret = m::HSD_VIGetXFBDrawEnable() as u32;
 }
                    9 => { 

ret = m::HSD_VIWaitXFBDrawEnable() as u32;
 }
                    10 => { 
let a0 = pick_GXRenderModeObj();
m::HSD_VICopyEFB2XFBHiResoAA(a0);
 }
                    11 => { 
let a0 = pick_HSD_VIStatus();
let a1 = (&raw mut XB as *mut u8).add(4096) as *mut c_void;
let a2 = (fv() % 5) as _;
m::HSD_VICopyEFB2XFBPtr(a0, a1, a2);
 }
                    12 => { 
let a0 = fv() as _;
m::HSD_VIGXSetDrawDone(a0);
 }
                    13 => { 
let a0 = (fv() % 3) as i32;
m::HSD_VISetXFBWaitDone(a0);
 }
                    14 => { 
let a0 = (fv() % 5) as _;
m::HSD_VICopyXFBAsync(a0);
 }
                    15 => { 
let a0 = (fv() % 3) as i32;
m::HSD_VIDrawDoneXFB(a0);
 }
                    16 => { 

ret = m::HSD_VIWaitXFBFlush_sub() as u32;
 }
                    17 => { 

m::HSD_VIWaitXFBFlush();
 }
                    18 => { 

m::HSD_VIWaitXFBFlushNoYield();
 }
                    19 => { 

ret = m::HSD_VIGetXFBLastDrawDone() as u32;
 }
                    20 => { 
let a0 = pick_GXRenderModeObj();
m::HSD_VISetConfigure(a0);
 }
                    21 => { 
let a0 = (fv() & 1) as i32;
m::HSD_VISetBlack(a0);
 }
                    22 => { 
let a0 = pick_HSD_VIStatus();
let a1 = (&raw mut BUF) as *mut c_void;
let a2 = if fv() & 1 != 0 { (&raw mut BUF as *mut u8).add(8) as *mut c_void } else { null_mut() };
let a3 = if fv() & 1 != 0 { (&raw mut BUF as *mut u8).add(16) as *mut c_void } else { null_mut() };
m::HSD_VIInit(a0, a1, a2, a3);
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
