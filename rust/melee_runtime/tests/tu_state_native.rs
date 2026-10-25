//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/state_native.c.
//! src/tu/state_native.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py state_native`).
//! C functions of the TU this test covers (the private ones through their public callers): HSD_SetMaterialColor, HSD_SetMaterialShininess, HSD_SetupChannelMode, HSD_SetupPEMode, HSD_SetupRenderMode, HSD_SetupRenderModeWithCustomPE, HSD_StateInvalidate, HSD_StateSetAlphaCompare, HSD_StateSetAlphaUpdate, HSD_StateSetBlendMode, HSD_StateSetColorUpdate, HSD_StateSetCullMode, HSD_StateSetDither, HSD_StateSetDstAlpha, HSD_StateSetLineWidth, HSD_StateSetPointSize, HSD_StateSetZCompLoc, HSD_StateSetZMode, _HSD_StateInvalidatePrimitive, _HSD_StateInvalidateRenderMode, _HSD_StateInvalidateVtxAttr, setupTevMode_last.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_HSD_SetupChannelMode: u32 = 0;
const OP_HSD_SetupPEMode: u32 = 1;
const OP_HSD_SetupRenderModeWithCustomPE: u32 = 2;
const OP_HSD_SetupRenderMode: u32 = 3;
const OP_HSD_SetMaterialColor: u32 = 4;
const OP_HSD_SetMaterialShininess: u32 = 5;
const OP_HSD_StateSetLineWidth: u32 = 6;
const OP_HSD_StateSetCullMode: u32 = 7;
const OP_HSD_StateSetBlendMode: u32 = 8;
const OP_HSD_StateSetZMode: u32 = 9;
const OP_HSD_StateSetPointSize: u32 = 10;
const OP_HSD_StateSetAlphaCompare: u32 = 11;
const OP_HSD_StateSetColorUpdate: u32 = 12;
const OP_HSD_StateSetAlphaUpdate: u32 = 13;
const OP_HSD_StateSetDstAlpha: u32 = 14;
const OP_HSD_StateSetZCompLoc: u32 = 15;
const OP_HSD_StateSetDither: u32 = 16;
const OP__HSD_StateInvalidatePrimitive: u32 = 17;
const OP__HSD_StateInvalidateVtxAttr: u32 = 18;
const OP__HSD_StateInvalidateRenderMode: u32 = 19;
const OP_HSD_StateInvalidate: u32 = 20;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::state_native as m;
use mtu_harness::*;
#[path = "mtu/state_native_gen.rs"]
mod g;


fn pick_col() -> GXColor { let r = fv() as u8; let g = fv() as u8; let b = fv() as u8; let a = fv() as u8; GXColor { r, g, b, a } }


pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut PPA: [*mut c_void; 4] = [core::ptr::null_mut(); 4];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub static mut L: [HSD_LObj; 3] = unsafe { core::mem::zeroed() };
pub static mut PE: [HSD_PEDesc; 2] = unsafe { core::mem::zeroed() };
pub fn pick_HSD_LObj() -> *mut HSD_LObj { let r = rnd(); unsafe { &raw mut L[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_LObj() -> *mut HSD_LObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut L[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_PEDesc() -> *mut HSD_PEDesc { let r = rnd(); unsafe { &raw mut PE[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_PEDesc() -> *mut HSD_PEDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut PE[((r >> 8) % 2) as usize] } } }
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
        preg(&raw const L, core::mem::size_of_val(&L));
        preg(&raw const PE, core::mem::size_of_val(&PE));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS)); preg(&raw const PPA, core::mem::size_of_val(&PPA));
    }
}

pub unsafe fn setup_world() {
    L = core::mem::zeroed();
    PE = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..4 { PPA[i] = core::ptr::null_mut(); }
    for i in 0..3 { g::fill_HSD_LObj(&mut L[i]); }
    for i in 0..2 { g::fill_HSD_PEDesc(&mut PE[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..3 { g::hash_HSD_LObj(&L[i]); }
    for i in 0..2 { g::hash_HSD_PEDesc(&PE[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    for i in 0..4 { hm(pi(PPA[i])); }
    g::hashg_all();
    g::hasho_all();
}

// harness = false (Cargo.toml): a case that ends in a panic is cut off inside the hook (see mtu_harness::run_case),
// which a libtest main thread cannot survive at exit.
fn main() { state_native_matches_c(); println!("tu_state_native: ok"); }

fn state_native_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();
melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());
        install_abort_hook();
        let mut c = Check::new(include_str!("data/mtu_state_native.txt"));
        let mut total = 0u32;
        for op in 0..21u32 {
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
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();

                let mut ret = 0u32;
                let aborted = run_case(|| unsafe { match op {
                    0 => { 
let a0 = fv() as _;
m::HSD_SetupChannelMode(a0);
 }
                    1 => { 
let a0 = fv() as _;
let a1 = pick_HSD_PEDesc();
m::HSD_SetupPEMode(a0, a1);
 }
                    2 => { 
let a0 = fv() as _;
let a1 = pick_HSD_PEDesc();
m::HSD_SetupRenderModeWithCustomPE(a0, a1);
 }
                    3 => { 
let a0 = fv() as _;
m::HSD_SetupRenderMode(a0);
 }
                    4 => { 
let a0 = pick_col();
let a1 = pick_col();
let a2 = pick_col();
let a3 = rw();
m::HSD_SetMaterialColor(a0, a1, a2, a3);
 }
                    5 => { 
let a0 = rw();
m::HSD_SetMaterialShininess(a0);
 }
                    6 => { 
let a0 = fv() as _;
let a1 = fv() as _;
m::HSD_StateSetLineWidth(a0, a1);
 }
                    7 => { 
let a0 = fv() as _;
m::HSD_StateSetCullMode(a0);
 }
                    8 => { 
let a0 = fv() as _;
let a1 = fv() as _;
let a2 = fv() as _;
let a3 = fv() as _;
m::HSD_StateSetBlendMode(a0, a1, a2, a3);
 }
                    9 => { 
let a0 = fv() as _;
let a1 = fv() as _;
let a2 = fv() as _;
m::HSD_StateSetZMode(a0, a1, a2);
 }
                    10 => { 
let a0 = fv() as _;
let a1 = fv() as _;
m::HSD_StateSetPointSize(a0, a1);
 }
                    11 => { 
let a0 = fv() as _;
let a1 = fv() as _;
let a2 = fv() as _;
let a3 = fv() as _;
let a4 = fv() as _;
m::HSD_StateSetAlphaCompare(a0, a1, a2, a3, a4);
 }
                    12 => { 
let a0 = fv() as _;
m::HSD_StateSetColorUpdate(a0);
 }
                    13 => { 
let a0 = fv() as _;
m::HSD_StateSetAlphaUpdate(a0);
 }
                    14 => { 
let a0 = fv() as _;
let a1 = fv() as _;
m::HSD_StateSetDstAlpha(a0, a1);
 }
                    15 => { 
let a0 = fv() as _;
m::HSD_StateSetZCompLoc(a0);
 }
                    16 => { 
let a0 = fv() as _;
m::HSD_StateSetDither(a0);
 }
                    17 => { 

m::_HSD_StateInvalidatePrimitive();
 }
                    18 => { 

m::_HSD_StateInvalidateVtxAttr();
 }
                    19 => { 

m::_HSD_StateInvalidateRenderMode();
 }
                    20 => { 
let a0 = fv() as _;
m::HSD_StateInvalidate(a0);
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
