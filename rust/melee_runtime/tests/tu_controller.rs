//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/controller.c.
//! src/tu/controller.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py controller`).
//! C functions of the TU this test covers (the private ones through their public callers): HSD_PadADConvert, HSD_PadADConvertCheck1, HSD_PadClamp, HSD_PadClampCheck1, HSD_PadClampCheck3, HSD_PadClearStatusFields, HSD_PadCopyStatusFields, HSD_PadCrossDir, HSD_PadFlushQueue, HSD_PadGetRawQueueCount, HSD_PadGetResetSwitch, HSD_PadInit, HSD_PadRawMerge, HSD_PadRawQueueShift, HSD_PadRenewCopyStatus, HSD_PadRenewGameStatus, HSD_PadRenewMasterStatus, HSD_PadRenewRawStatus, HSD_PadRenewStatus, HSD_PadReset, HSD_PadScale, sq, vec2DSqDist, vec2Dlen.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_HSD_PadGetRawQueueCount: u32 = 0;
const OP_HSD_PadGetResetSwitch: u32 = 1;
const OP_HSD_PadRenewRawStatus: u32 = 2;
const OP_HSD_PadFlushQueue: u32 = 3;
const OP_HSD_PadRenewMasterStatus: u32 = 4;
const OP_HSD_PadRenewCopyStatus: u32 = 5;
const OP_HSD_PadRenewGameStatus: u32 = 6;
const OP_HSD_PadRenewStatus: u32 = 7;
const OP_HSD_PadReset: u32 = 8;
const OP_HSD_PadInit: u32 = 9;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::controller as m;
use mtu_harness::*;
#[path = "mtu/controller_gen.rs"]
mod g;



pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub static mut PQ: [HSD_PadData; 16] = unsafe { core::mem::zeroed() };
pub static mut RL: [HSD_PadRumbleListData; 2] = unsafe { core::mem::zeroed() };
pub fn pick_HSD_PadData() -> *mut HSD_PadData { let r = rnd(); unsafe { &raw mut PQ[((r >> 8) % 16) as usize] } }
pub fn picko_HSD_PadData() -> *mut HSD_PadData { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut PQ[((r >> 8) % 16) as usize] } } }
pub fn pick_HSD_PadRumbleListData() -> *mut HSD_PadRumbleListData { let r = rnd(); unsafe { &raw mut RL[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_PadRumbleListData() -> *mut HSD_PadRumbleListData { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut RL[((r >> 8) % 2) as usize] } } }
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
pub fn freg_world() { freg(cbev1 as usize); freg(cbev2 as usize); freg(cbp1 as usize); freg(cbp2 as usize); freg(cbev3 as usize); freg(cbi1 as usize); freg(cbi2 as usize); }
pub fn preg_world() {
    unsafe {
        preg(&raw const PQ, core::mem::size_of_val(&PQ));
        preg(&raw const RL, core::mem::size_of_val(&RL));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS));
    }
}

pub unsafe fn setup_world() {
    PQ = core::mem::zeroed();
    RL = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..16 { g::fill_HSD_PadData(&mut PQ[i]); }
    for i in 0..2 { g::fill_HSD_PadRumbleListData(&mut RL[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..16 { g::hash_HSD_PadData(&PQ[i]); }
    for i in 0..2 { g::hash_HSD_PadRumbleListData(&RL[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    g::hashg_all();
    g::hasho_all();
}

// harness = false (Cargo.toml): a case that ends in a panic is cut off inside the hook (see mtu_harness::run_case),
// which a libtest main thread cannot survive at exit.
fn main() { controller_matches_c(); println!("tu_controller: ok"); }

fn controller_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();

        install_abort_hook();
        let mut c = Check::new(include_str!("data/mtu_controller.txt"));
        let mut total = 0u32;
        for op in 0..10u32 {
            let n = match op {
                0 => 400u32,
                1 => 400u32,
                2 => 400u32,
                3 => 400u32,
                4 => 400u32,
                5 => 400u32,
                6 => 400u32,
                7 => 400u32,
                8 => 400u32,
                9 => 400u32,
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
{ m::HSD_PadLibData.qnum = (1 + fv() % 4) as u8; m::HSD_PadLibData.qread = (fv() % m::HSD_PadLibData.qnum as u32) as u8; m::HSD_PadLibData.qwrite = (fv() % m::HSD_PadLibData.qnum as u32) as u8; m::HSD_PadLibData.qcount = (fv() % (m::HSD_PadLibData.qnum as u32 + 1)) as u8; m::HSD_PadLibData.qtype = (fv() % 4) as u8; m::HSD_PadLibData.queue = (&raw mut PQ) as *mut HSD_PadData; m::HSD_PadLibData.clamp_stickType = (if fv() & 3 == 0 { 1 } else { 0 }) as u8; m::HSD_PadLibData.adc_type = (if fv() & 3 == 0 { 1 } else { 0 }) as i8; m::HSD_PadLibData.adc_th = (fv() % 50) as i8; m::HSD_PadLibData.adc_angle = rw(); m::HSD_PadLibData.cross_dir = (fv() % 5) as u8; m::HSD_PadLibData.clamp_stickMax = (30 + fv() % 98) as i8; m::HSD_PadLibData.clamp_stickMin = (fv() % 30) as i8; m::HSD_PadLibData.clamp_stickShift = (fv() % 3) as u8; m::HSD_PadLibData.repeat_start = (1 + fv() % 40) as i32; m::HSD_PadLibData.repeat_interval = (1 + fv() % 20) as i32; m::HSD_PadLibData.scale_stick = (fv() % 128) as i8; m::HSD_PadLibData.reset_switch_status = (fv() & 1) as u8; m::HSD_PadLibData.reset_switch = (fv() & 1) as u8; }
                let mut ret = 0u32;
                let aborted = run_case(|| unsafe { match op {
                    0 => { 

ret = m::HSD_PadGetRawQueueCount() as u32;
 }
                    1 => { 

ret = m::HSD_PadGetResetSwitch() as u32;
 }
                    2 => { 
let a0 = fv() & 1 != 0;
m::HSD_PadRenewRawStatus(a0);
 }
                    3 => { 
let a0 = fv() as _;
m::HSD_PadFlushQueue(a0);
 }
                    4 => { 

m::HSD_PadRenewMasterStatus();
 }
                    5 => { 

m::HSD_PadRenewCopyStatus();
 }
                    6 => { 

m::HSD_PadRenewGameStatus();
 }
                    7 => { 

m::HSD_PadRenewStatus();
 }
                    8 => { 

m::HSD_PadReset();
 }
                    9 => { 
let a0 = fv() as _;
let a1 = pick_HSD_PadData();
let a2 = fv() as _;
let a3 = pick_HSD_PadRumbleListData();
m::HSD_PadInit(a0, a1, a2, a3);
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
