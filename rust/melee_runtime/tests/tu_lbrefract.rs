//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/lbrefract.c.
//! src/tu/lbrefract.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py lbrefract`).
//! C functions of the TU this test covers (the private ones through their public callers): fn_80021F70, fn_80021FB4, fn_80021FF8, fn_8002206C, fn_80022650, fn_80022940, lbRefSetUnuse, lbRefract_80021CE8, lbRefract_8002219C, lbRefract_800222A4, lbRefract_8002247C, lbRefract_80022560, lbRefract_800225D4, lbRefract_80022998, lbRefract_80022BB8, lbRefract_DObjDispReset, lbRefract_PObjLoad, lbRefract_ReadTexCoordRGBA8, lbRefract_WriteTexCoord, lbRefract_WriteTexCoordIA4, my_fmodf.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_lbRefract_80021CE8: u32 = 0;
const OP_lbRefract_WriteTexCoordIA4: u32 = 1;
const OP_lbRefract_PObjLoad: u32 = 2;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::lbrefract as m;
use mtu_harness::*;
#[path = "mtu/lbrefract_gen.rs"]
mod g;


static mut RFB: [u8; 4096] = [0; 4096];
static mut RD: [m::RefractData; 1] = [m::RefractData { x0: 0, x4: null_mut() }];
unsafe extern "C" fn cb_load(p: *mut HSD_PObj, d: *mut HSD_PObjDesc) -> i32 { t(8208); t(pi(p)); t(pi(d)); if fv() & 3 != 0 { 0 } else { (fv() % 3) as i32 - 1 } }


pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut PPA: [*mut c_void; 4] = [core::ptr::null_mut(); 4];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub static mut CB: [lbRefract_CallbackData; 2] = unsafe { core::mem::zeroed() };
pub static mut P1: [HSD_PObj; 2] = unsafe { core::mem::zeroed() };
pub static mut P2: [HSD_PObjDesc; 2] = unsafe { core::mem::zeroed() };
pub static mut VD: [HSD_VtxDescList; 6] = unsafe { core::mem::zeroed() };
pub static mut PI_: [HSD_PObjInfo; 1] = unsafe { core::mem::zeroed() };
pub fn pick_lbRefract_CallbackData() -> *mut lbRefract_CallbackData { let r = rnd(); unsafe { &raw mut CB[((r >> 8) % 2) as usize] } }
pub fn picko_lbRefract_CallbackData() -> *mut lbRefract_CallbackData { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut CB[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_PObj() -> *mut HSD_PObj { let r = rnd(); unsafe { &raw mut P1[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_PObj() -> *mut HSD_PObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut P1[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_PObjDesc() -> *mut HSD_PObjDesc { let r = rnd(); unsafe { &raw mut P2[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_PObjDesc() -> *mut HSD_PObjDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut P2[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_VtxDescList() -> *mut HSD_VtxDescList { let r = rnd(); unsafe { &raw mut VD[((r >> 8) % 6) as usize] } }
pub fn picko_HSD_VtxDescList() -> *mut HSD_VtxDescList { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut VD[((r >> 8) % 6) as usize] } } }
pub fn pick_HSD_PObjInfo() -> *mut HSD_PObjInfo { let r = rnd(); unsafe { &raw mut PI_[((r >> 8) % 1) as usize] } }
pub fn picko_HSD_PObjInfo() -> *mut HSD_PObjInfo { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut PI_[((r >> 8) % 1) as usize] } } }
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
        preg(&raw const CB, core::mem::size_of_val(&CB));
        preg(&raw const P1, core::mem::size_of_val(&P1));
        preg(&raw const P2, core::mem::size_of_val(&P2));
        preg(&raw const VD, core::mem::size_of_val(&VD));
        preg(&raw const PI_, core::mem::size_of_val(&PI_));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS)); preg(&raw const PPA, core::mem::size_of_val(&PPA));
    }
}

pub unsafe fn setup_world() {
    CB = core::mem::zeroed();
    P1 = core::mem::zeroed();
    P2 = core::mem::zeroed();
    VD = core::mem::zeroed();
    PI_ = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..4 { PPA[i] = core::ptr::null_mut(); }
    for i in 0..2 { g::fill_lbRefract_CallbackData(&mut CB[i]); }
    for i in 0..2 { g::fill_HSD_PObj(&mut P1[i]); }
    for i in 0..2 { g::fill_HSD_PObjDesc(&mut P2[i]); }
    for i in 0..6 { g::fill_HSD_VtxDescList(&mut VD[i]); }
    for i in 0..1 { g::fill_HSD_PObjInfo(&mut PI_[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..2 { g::hash_lbRefract_CallbackData(&CB[i]); }
    for i in 0..2 { g::hash_HSD_PObj(&P1[i]); }
    for i in 0..2 { g::hash_HSD_PObjDesc(&P2[i]); }
    for i in 0..6 { g::hash_HSD_VtxDescList(&VD[i]); }
    for i in 0..1 { g::hash_HSD_PObjInfo(&PI_[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    for i in 0..4 { hm(pi(PPA[i])); }
    g::hashg_all();
    g::hasho_all();
}

// harness = false (Cargo.toml): a case that ends in a panic is cut off inside the hook (see mtu_harness::run_case),
// which a libtest main thread cannot survive at exit.
fn main() { lbrefract_matches_c(); println!("tu_lbrefract: ok"); }

fn lbrefract_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();
preg(&raw const RFB, 4096); preg(&raw const RD, 16); m::use_ext_data = true; freg(cb_load as usize); melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());
        install_abort_hook();
        let mut c = Check::new(include_str!("data/mtu_lbrefract.txt"));
        let mut total = 0u32;
        for op in 0..3u32 {
            let n = match op {
                0 => 400u32,
                1 => 400u32,
                2 => 400u32,
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
{ for q in 0..2usize { CB[q].buffer = (&raw mut RFB) as *mut c_void; CB[q].width = 1 + fv() % 32; CB[q].height = 1 + fv() % 32; CB[q].row_stride = 256; CB[q].buffer_size = 2048; CB[q].format = 3; CB[q].callback0 = (m::lbRefract_WriteTexCoordIA4 as unsafe extern "C" fn(*mut lbRefract_CallbackData, u32, u32, u32, u32, u32, u32)) as usize as *mut c_void; CB[q].callback1 = null_mut(); } for q in 0..4096usize { RFB[q] = rnd() as u8; } RD[0].x4 = (&raw mut FA) as *mut f32; m::refract_data = &raw mut RD[0]; for q in 0..6usize { VD[q].attr = if fv() & 3 != 0 { fv() % 26 } else { 255 }; VD[q].attr_type = fv() % 4; VD[q].comp_type = fv() % 6; VD[q].comp_cnt = fv() % 2; } VD[5].attr = 255; for q in 0..2usize { P1[q].verts = if fv() & 3 != 0 { &raw mut VD[(fv() % 6) as usize] } else { null_mut() }; P1[q].display = (&raw mut RFB) as *mut u8; P1[q].n_display = (fv() % 4) as u16; P1[q].flags = if fv() & 1 != 0 { 0x2000 } else { (fv() & 0xFFF) as u16 }; P1[q].parent.class_info = (&raw mut PI_[0]) as *mut HSD_ClassInfo; } PI_[0].load = Some(cb_load); m::ext::hsdPObj.load = Some(cb_load); }
                let mut ret = 0u32;
                let aborted = run_case(|| unsafe { match op {
                    0 => { 
let a0 = &raw mut CB[(fv() & 1) as usize];
let a1 = (fv() & 1) as i32;
m::lbRefract_80021CE8(a0, a1);
 }
                    1 => { 
let a0 = &raw mut CB[(fv() & 1) as usize];
let a1 = rnd() % 32;
let a2 = rnd() % 32;
let a3 = fv() as _;
let a4 = fv() as _;
let a5 = fv() as _;
let a6 = fv() as _;
m::lbRefract_WriteTexCoordIA4(a0, a1, a2, a3, a4, a5, a6);
 }
                    2 => { 
let a0 = pick_HSD_PObj();
let a1 = pick_HSD_PObjDesc();
ret = m::lbRefract_PObjLoad(a0, a1) as u32;
 }
                    _ => unreachable!(),
                } });
                if aborted { c.abort(op, k); total += 1; continue; }
                hash_world();
{ for i in 0..4096usize { hm(RFB[i] as u32); } }
                c.line(op, k, ret);
                total += 1;
            }
        }
        assert_eq!(c.done(), total);
    }
}
