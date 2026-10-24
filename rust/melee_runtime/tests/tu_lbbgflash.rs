//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/lbbgflash.c.
//! src/tu/lbbgflash.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py lbbgflash`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_bgflash_position: u32 = 0;
const OP_fn_8001FC08: u32 = 1;
const OP_fn_8001FEC4: u32 = 2;
const OP_fn_800204C8: u32 = 3;
const OP_lbBgFlash_800205F0: u32 = 4;
const OP_lbBgFlash_8002063C: u32 = 5;
const OP_lbBgFlash_80020688: u32 = 6;
const OP_lbBgFlash_800206D4: u32 = 7;
const OP_lbBgFlash_InitState: u32 = 8;
const OP_fn_800208B0: u32 = 9;
const OP_lbBgFlash_800208EC: u32 = 10;
const OP_lbBgFlash_800209F4: u32 = 11;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::lbbgflash as m;
use mtu_harness::*;
#[path = "mtu/lbbgflash_gen.rs"]
mod g;



pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub static mut GO: [HSD_GObj; 4] = unsafe { core::mem::zeroed() };
pub static mut COL: [GXColor; 4] = unsafe { core::mem::zeroed() };
pub fn pick_HSD_GObj() -> *mut HSD_GObj { let r = rnd(); unsafe { &raw mut GO[((r >> 8) % 4) as usize] } }
pub fn picko_HSD_GObj() -> *mut HSD_GObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut GO[((r >> 8) % 4) as usize] } } }
pub fn pick_GXColor() -> *mut GXColor { let r = rnd(); unsafe { &raw mut COL[((r >> 8) % 4) as usize] } }
pub fn picko_GXColor() -> *mut GXColor { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut COL[((r >> 8) % 4) as usize] } } }
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
        preg(&raw const GO, core::mem::size_of_val(&GO));
        preg(&raw const COL, core::mem::size_of_val(&COL));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS));
    }
}

pub unsafe fn setup_world() {
    GO = core::mem::zeroed();
    COL = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..4 { g::fill_HSD_GObj(&mut GO[i]); }
    for i in 0..4 { g::fill_GXColor(&mut COL[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..4 { g::hash_HSD_GObj(&GO[i]); }
    for i in 0..4 { g::hash_GXColor(&COL[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    g::hashg_all();
    g::hasho_all();
}

#[test]
fn lbbgflash_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();

        let mut c = Check::new(include_str!("data/mtu_lbbgflash.txt"));
        let mut total = 0u32;
        for op in 0..12u32 {
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
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
{ m::lbl_80433658.x32 = (1 + fv() % 60) as u8; m::lbl_80433658.x38 = (fv() % 700) as i32 - 100; m::lbl_80433658.x3C = (fv() % 40) as i32; m::lbl_80433658.x34 = (fv() % 900) as i32 - 50; m::lbl_80433658.x30 = if fv() & 3 != 0 { 0 } else { fv() as u8 }; m::lbl_80433658.x33 = if fv() & 3 != 0 { 0 } else { 1 }; }
                let mut ret = 0u32;
                match op {
                    0 => { 
let a0 = rw();
let a1 = rw();
m::bgflash_position(a0, a1);
 }
                    1 => { 

m::fn_8001FC08();
 }
                    2 => { 
let a0 = pick_HSD_GObj();
let a1 = fv() as _;
m::fn_8001FEC4(a0, a1);
 }
                    3 => { 

m::fn_800204C8();
 }
                    4 => { 
let a0 = fv() as _;
m::lbBgFlash_800205F0(a0);
 }
                    5 => { 
let a0 = fv() as _;
m::lbBgFlash_8002063C(a0);
 }
                    6 => { 
let a0 = fv() as _;
m::lbBgFlash_80020688(a0);
 }
                    7 => { 
let a0 = pick_GXColor();
let a1 = pick_GXColor();
let a2 = fv() as _;
m::lbBgFlash_800206D4(a0, a1, a2);
 }
                    8 => { 
let a0 = pick_GXColor();
m::lbBgFlash_InitState(a0);
 }
                    9 => { 
let a0 = fv() as _;
m::fn_800208B0(a0);
 }
                    10 => { 
let a0 = fv() as _;
m::lbBgFlash_800208EC(a0);
 }
                    11 => { 

m::lbBgFlash_800209F4();
 }
                    _ => unreachable!(),
                }
                hash_world();

                c.line(op, k, ret);
                total += 1;
            }
        }
        assert_eq!(c.done(), total);
    }
}
