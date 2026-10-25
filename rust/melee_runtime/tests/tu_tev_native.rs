//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/tev_native.c.
//! src/tu/tev_native.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py tev_native`).
//! C functions of the TU this test covers (the private ones through their public callers): ChanUpdateFunc, CompareRGB, CompareRGBA, CopyRGB, HSD_ChanGetAllocData, HSD_Channel2Num, HSD_Index2TevStage, HSD_RenderGetAllocData, HSD_RenderInitAllocData, HSD_SetTevRegAll, HSD_SetupChannel, HSD_SetupChannelAll, HSD_SetupTevStage, HSD_SetupTevStageAll, HSD_StateAssignTev, HSD_StateGetCommittedTevStages, HSD_StateGetNumTevStages, HSD_StateInitTev, HSD_StateRegisterTexGen, HSD_StateSetNumChans, HSD_StateSetNumTevStages, HSD_StateSetNumTexGens, HSD_TevRegGetAllocData, HSD_TevStage2Index, HSD_TevStage2Num, HSD_TexCoordID2Num, _HSD_StateInvalidateColorChannel, _HSD_StateInvalidateTevRegister, _HSD_StateInvalidateTevStage, _HSD_StateInvalidateTexCoordGen.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_HSD_RenderInitAllocData: u32 = 0;
const OP_HSD_RenderGetAllocData: u32 = 1;
const OP_HSD_TevRegGetAllocData: u32 = 2;
const OP_HSD_ChanGetAllocData: u32 = 3;
const OP_HSD_SetupChannel: u32 = 4;
const OP_HSD_StateSetNumChans: u32 = 5;
const OP_HSD_SetupChannelAll: u32 = 6;
const OP_HSD_StateRegisterTexGen: u32 = 7;
const OP_HSD_StateSetNumTexGens: u32 = 8;
const OP_HSD_StateInitTev: u32 = 9;
const OP_HSD_StateGetCommittedTevStages: u32 = 10;
const OP_HSD_StateGetNumTevStages: u32 = 11;
const OP_HSD_StateAssignTev: u32 = 12;
const OP_HSD_StateSetNumTevStages: u32 = 13;
const OP_HSD_SetupTevStage: u32 = 14;
const OP_HSD_SetupTevStageAll: u32 = 15;
const OP_HSD_Channel2Num: u32 = 16;
const OP_HSD_Index2TevStage: u32 = 17;
const OP_HSD_TevStage2Index: u32 = 18;
const OP_HSD_TevStage2Num: u32 = 19;
const OP_HSD_SetTevRegAll: u32 = 20;
const OP_HSD_TexCoordID2Num: u32 = 21;
const OP_ChanUpdateFunc: u32 = 22;
const OP__HSD_StateInvalidateColorChannel: u32 = 23;
const OP__HSD_StateInvalidateTevStage: u32 = 24;
const OP__HSD_StateInvalidateTevRegister: u32 = 25;
const OP__HSD_StateInvalidateTexCoordGen: u32 = 26;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::tev_native as m;
use mtu_harness::*;
#[path = "mtu/tev_native_gen.rs"]
mod g;



pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut PPA: [*mut c_void; 4] = [core::ptr::null_mut(); 4];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub static mut CH: [HSD_Chan; 3] = unsafe { core::mem::zeroed() };
pub static mut TD: [HSD_TevDesc; 3] = unsafe { core::mem::zeroed() };
pub fn pick_HSD_Chan() -> *mut HSD_Chan { let r = rnd(); unsafe { &raw mut CH[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_Chan() -> *mut HSD_Chan { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut CH[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_TevDesc() -> *mut HSD_TevDesc { let r = rnd(); unsafe { &raw mut TD[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_TevDesc() -> *mut HSD_TevDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut TD[((r >> 8) % 3) as usize] } } }
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
        preg(&raw const CH, core::mem::size_of_val(&CH));
        preg(&raw const TD, core::mem::size_of_val(&TD));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS)); preg(&raw const PPA, core::mem::size_of_val(&PPA));
    }
}

pub unsafe fn setup_world() {
    CH = core::mem::zeroed();
    TD = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..4 { PPA[i] = core::ptr::null_mut(); }
    for i in 0..3 { g::fill_HSD_Chan(&mut CH[i]); }
    for i in 0..3 { g::fill_HSD_TevDesc(&mut TD[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..3 { g::hash_HSD_Chan(&CH[i]); }
    for i in 0..3 { g::hash_HSD_TevDesc(&TD[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    for i in 0..4 { hm(pi(PPA[i])); }
    g::hashg_all();
    g::hasho_all();
}

// harness = false (Cargo.toml): a case that ends in a panic is cut off inside the hook (see mtu_harness::run_case),
// which a libtest main thread cannot survive at exit.
fn main() { tev_native_matches_c(); println!("tu_tev_native: ok"); }

fn tev_native_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();
melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());
        install_abort_hook();
        let mut c = Check::new(include_str!("data/mtu_tev_native.txt"));
        let mut total = 0u32;
        for op in 0..27u32 {
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
                10 => 400u32,
                11 => 400u32,
                12 => 400u32,
                13 => 400u32,
                14 => 400u32,
                15 => 400u32,
                16 => 400u32,
                17 => 400u32,
                18 => 400u32,
                19 => 400u32,
                20 => 400u32,
                21 => 400u32,
                22 => 400u32,
                23 => 400u32,
                24 => 400u32,
                25 => 400u32,
                26 => 400u32,
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
{ for q in 0..3usize { CH[q].chan = if fv() % 8 == 7 { 0xFF } else { fv() % 6 }; CH[q].next = if q < 2 && fv() & 1 != 0 { &raw mut CH[q + 1] } else { null_mut() }; CH[q].enable = (fv() & 1) as _; CH[q].amb_src = (fv() & 1) as _; CH[q].mat_src = (fv() & 1) as _; TD[q].stage = fv() % 16; TD[q].next = if q < 2 && fv() & 1 != 0 { &raw mut TD[q + 1] } else { null_mut() }; TD[q].flags = fv() & 1; TD[q].coord = fv() % 8; TD[q].map = fv() % 8; TD[q].color = fv() % 8; } m::current_tev = (fv() % 16) as i32; for q in 0..2usize { m::prev_mat_invalid[q] = (fv() & 1) as i32; m::prev_amb_invalid[q] = (fv() & 1) as i32; } for q in 0..4usize { m::TevReg[q].c = (fv() & 1) as i32; } }
                let mut ret = 0u32;
                let aborted = run_case(|| unsafe { match op {
                    0 => { 

m::HSD_RenderInitAllocData();
 }
                    1 => { 

ret = pi(m::HSD_RenderGetAllocData());
 }
                    2 => { 

ret = pi(m::HSD_TevRegGetAllocData());
 }
                    3 => { 

ret = pi(m::HSD_ChanGetAllocData());
 }
                    4 => { 
let a0 = pick_HSD_Chan();
m::HSD_SetupChannel(a0);
 }
                    5 => { 
let a0 = fv() as _;
m::HSD_StateSetNumChans(a0);
 }
                    6 => { 
let a0 = pick_HSD_Chan();
m::HSD_SetupChannelAll(a0);
 }
                    7 => { 
let a0 = fv() as _;
m::HSD_StateRegisterTexGen(a0);
 }
                    8 => { 

m::HSD_StateSetNumTexGens();
 }
                    9 => { 

m::HSD_StateInitTev();
 }
                    10 => { 

ret = m::HSD_StateGetCommittedTevStages() as u32;
 }
                    11 => { 

ret = m::HSD_StateGetNumTevStages() as u32;
 }
                    12 => { 

ret = m::HSD_StateAssignTev() as u32;
 }
                    13 => { 

m::HSD_StateSetNumTevStages();
 }
                    14 => { 
let a0 = pick_HSD_TevDesc();
m::HSD_SetupTevStage(a0);
 }
                    15 => { 
let a0 = pick_HSD_TevDesc();
m::HSD_SetupTevStageAll(a0);
 }
                    16 => { 
let a0 = fv() as _;
ret = m::HSD_Channel2Num(a0) as u32;
 }
                    17 => { 
let a0 = fv() as _;
ret = m::HSD_Index2TevStage(a0) as u32;
 }
                    18 => { 
let a0 = fv() as _;
ret = m::HSD_TevStage2Index(a0) as u32;
 }
                    19 => { 
let a0 = fv() as _;
ret = m::HSD_TevStage2Num(a0) as u32;
 }
                    20 => { 

m::HSD_SetTevRegAll();
 }
                    21 => { 
let a0 = fv() as _;
ret = m::HSD_TexCoordID2Num(a0) as u32;
 }
                    22 => { 
let a0 = pick_HSD_Chan();
let a1 = fv() as _;
let a2 = &raw mut FA[(rnd() % 4) as usize];
m::ChanUpdateFunc(a0, a1, a2);
 }
                    23 => { 

m::_HSD_StateInvalidateColorChannel();
 }
                    24 => { 

m::_HSD_StateInvalidateTevStage();
 }
                    25 => { 

m::_HSD_StateInvalidateTevRegister();
 }
                    26 => { 

m::_HSD_StateInvalidateTexCoordGen();
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
