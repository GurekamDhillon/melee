//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/rumble.c.
//! src/tu/rumble.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py rumble`).
//! C functions of the TU this test covers (the private ones through their public callers): HSD_PadRumbleAdd, HSD_PadRumbleFree, HSD_PadRumbleInit, HSD_PadRumbleInterpret, HSD_PadRumbleInterpret1, HSD_PadRumbleOffN, HSD_PadRumbleOn, HSD_PadRumblePause, HSD_PadRumblePauseAll, HSD_PadRumbleRemove, HSD_PadRumbleRemoveAll, HSD_PadRumbleRemoveId, HSD_PadRumbleUnpauseAll, HSD_Rumble_80378524, func_80378430_inline.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_HSD_PadRumbleOn: u32 = 0;
const OP_HSD_PadRumbleOffN: u32 = 1;
const OP_HSD_PadRumbleRemove: u32 = 2;
const OP_HSD_PadRumbleRemoveAll: u32 = 3;
const OP_HSD_PadRumbleRemoveId: u32 = 4;
const OP_HSD_PadRumblePause: u32 = 5;
const OP_HSD_PadRumblePauseAll: u32 = 6;
const OP_HSD_PadRumbleUnpauseAll: u32 = 7;
const OP_HSD_PadRumbleAdd: u32 = 8;
const OP_HSD_Rumble_80378524: u32 = 9;
const OP_HSD_PadRumbleInterpret1: u32 = 10;
const OP_HSD_PadRumbleInterpret: u32 = 11;
const OP_HSD_PadRumbleInit: u32 = 12;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::rumble as m;
use mtu_harness::*;
#[path = "mtu/rumble_gen.rs"]
mod g;



pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub static mut RL: [HSD_PadRumbleListData; 8] = unsafe { core::mem::zeroed() };
pub static mut CM: [HSD_Rumble; 40] = unsafe { core::mem::zeroed() };
pub fn pick_HSD_PadRumbleListData() -> *mut HSD_PadRumbleListData { let r = rnd(); unsafe { &raw mut RL[((r >> 8) % 8) as usize] } }
pub fn picko_HSD_PadRumbleListData() -> *mut HSD_PadRumbleListData { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut RL[((r >> 8) % 8) as usize] } } }
pub fn pick_HSD_Rumble() -> *mut HSD_Rumble { let r = rnd(); unsafe { &raw mut CM[((r >> 8) % 40) as usize] } }
pub fn picko_HSD_Rumble() -> *mut HSD_Rumble { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut CM[((r >> 8) % 40) as usize] } } }
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
        preg(&raw const RL, core::mem::size_of_val(&RL));
        preg(&raw const CM, core::mem::size_of_val(&CM));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS));
    }
}

pub unsafe fn setup_world() {
    RL = core::mem::zeroed();
    CM = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..8 { g::fill_HSD_PadRumbleListData(&mut RL[i]); }
    for i in 0..40 { g::fill_HSD_Rumble(&mut CM[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..8 { g::hash_HSD_PadRumbleListData(&RL[i]); }
    for i in 0..40 { g::hash_HSD_Rumble(&CM[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    g::hashg_all();
    g::hasho_all();
}

// harness = false (Cargo.toml): a case that ends in a panic is cut off inside the hook (see mtu_harness::run_case),
// which a libtest main thread cannot survive at exit.
fn main() { rumble_matches_c(); println!("tu_rumble: ok"); }

fn rumble_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();

        install_abort_hook();
        let mut c = Check::new(include_str!("data/mtu_rumble.txt"));
        let mut total = 0u32;
        for op in 0..13u32 {
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
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
{ let mut cnt = [0u32; 4]; let mut fr: *mut HSD_PadRumbleListData = null_mut(); let mut tail: [*mut HSD_PadRumbleListData; 4] = [null_mut(); 4]; m::ext::use_own_libdata = true; for q in 0..40usize { CM[q].command.set_op(if q >= 39 { 0 } else if q % 4 == 1 { 4 } else if q % 4 == 3 { 5 } else { 1 + fv() % 3 } as _); CM[q].command.set_frame((1 + fv() % 6) as _); } for ch in 0..4usize { cnt[ch] = 0; tail[ch] = null_mut(); m::HSD_Rumble_804C22E0[ch].listdatap = null_mut(); } for q in 0..8usize { RL[q].next = null_mut(); RL[q].listp = &raw mut CM[(fv() % 30) as usize]; RL[q].headp = RL[q].listp; RL[q].stack = &raw mut CM[(fv() % 30) as usize]; RL[q].pause = (fv() & 3 == 0) as u8; RL[q].id = fv() % 6; RL[q].pri = (fv() % 5) as u8; RL[q].loop_count = (fv() % 5) as u16; RL[q].wait = (fv() % 3) as u16; let f = fv() % 5; RL[q].frame = if f == 0 { -1 } else if f == 1 { -2 } else { (1 + fv() % 6) as i32 }; let ch = (fv() % 5) as usize; if ch == 4 { RL[q].next = fr; fr = &raw mut RL[q]; } else { if !tail[ch].is_null() { (*tail[ch]).next = &raw mut RL[q]; } else { m::HSD_Rumble_804C22E0[ch].listdatap = &raw mut RL[q]; } tail[ch] = &raw mut RL[q]; cnt[ch] += 1; } } for ch in 0..4usize { m::HSD_Rumble_804C22E0[ch].nb_list = cnt[ch] as u16; } m::ext::HSD_PadLibData.rumble_info.listdatap = fr; m::ext::HSD_PadLibData.rumble_info.max_list = (1 + fv() % 8) as u16; m::ext::HSD_PadLibData.rumble_info.unk2 = (fv() & 3 == 0) as u8; }
                let mut ret = 0u32;
                let aborted = run_case(|| unsafe { match op {
                    0 => { 
let a0 = (fv() % 4) as u8;
m::HSD_PadRumbleOn(a0);
 }
                    1 => { 
let a0 = (fv() % 4) as u8;
m::HSD_PadRumbleOffN(a0);
 }
                    2 => { 
let a0 = (fv() % 4) as u8;
m::HSD_PadRumbleRemove(a0);
 }
                    3 => { 

m::HSD_PadRumbleRemoveAll();
 }
                    4 => { 
let a0 = (fv() % 4) as u8;
let a1 = (fv() % 6) as i32;
m::HSD_PadRumbleRemoveId(a0, a1);
 }
                    5 => { 
let a0 = (fv() % 4) as u8;
let a1 = (fv() % 3) as i32;
m::HSD_PadRumblePause(a0, a1);
 }
                    6 => { 

m::HSD_PadRumblePauseAll();
 }
                    7 => { 

m::HSD_PadRumbleUnpauseAll();
 }
                    8 => { 
let a0 = (fv() % 4) as u8;
let a1 = (fv() % 6) as i32;
let a2 = fv() as _;
let a3 = (fv() % 5) as i32;
let a4 = (&raw mut CM[(fv() % 30) as usize]) as *mut c_void;
ret = m::HSD_PadRumbleAdd(a0, a1, a2, a3, a4) as u32;
 }
                    9 => { 
let a0 = fv() as _;
m::HSD_Rumble_80378524(a0);
 }
                    10 => { 
let a0 = pick_HSD_PadRumbleListData();
let a1 = (&raw mut U8A) as *mut u8;
ret = m::HSD_PadRumbleInterpret1(a0, a1) as u32;
 }
                    11 => { 

m::HSD_PadRumbleInterpret();
 }
                    12 => { 
let a0 = (fv() % 9) as u16;
let a1 = (&raw mut RL) as *mut c_void;
m::HSD_PadRumbleInit(a0, a1);
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
