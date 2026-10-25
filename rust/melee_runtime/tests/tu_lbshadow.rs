//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/lbshadow.c.
//! src/tu/lbshadow.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py lbshadow`).
//! C functions of the TU this test covers (the private ones through their public callers): lbShadow_8000E9F0, lbShadow_8000ED54, lbShadow_8000EE8C, lbShadow_8000EEE0, lbShadow_8000EFEC, lbShadow_8000F214, lbShadow_8000F38C, splGetBSplineTangent, splGetBezierTangent, splGetCardinalTangent.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_splGetCardinalTangent: u32 = 0;
const OP_splGetBSplineTangent: u32 = 1;
const OP_splGetBezierTangent: u32 = 2;
const OP_lbShadow_8000E9F0: u32 = 3;
const OP_lbShadow_8000ED54: u32 = 4;
const OP_lbShadow_8000EE8C: u32 = 5;
const OP_lbShadow_8000EEE0: u32 = 6;
const OP_lbShadow_8000EFEC: u32 = 7;
const OP_lbShadow_8000F214: u32 = 8;
const OP_lbShadow_8000F38C: u32 = 9;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::lbshadow as m;
use mtu_harness::*;
#[path = "mtu/lbshadow_gen.rs"]
mod g;

static mut PLH: [*mut HSD_GObj; 16] = [null_mut(); 16]; static mut GXH: [*mut HSD_GObj; 8] = [null_mut(); 8];

pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub static mut GO: [HSD_GObj; 4] = unsafe { core::mem::zeroed() };
pub static mut LS: [LbShadow; 2] = unsafe { core::mem::zeroed() };
pub static mut JO: [HSD_JObj; 3] = unsafe { core::mem::zeroed() };
pub static mut SHD: [HSD_Shadow; 3] = unsafe { core::mem::zeroed() };
pub static mut CO: [HSD_CObj; 2] = unsafe { core::mem::zeroed() };
pub static mut LO: [HSD_LObj; 3] = unsafe { core::mem::zeroed() };
pub static mut FTR: [Fighter; 3] = unsafe { core::mem::zeroed() };
pub static mut CM: [CmSubject; 2] = unsafe { core::mem::zeroed() };
pub static mut TO: [HSD_TObj; 2] = unsafe { core::mem::zeroed() };
pub static mut ID: [HSD_ImageDesc; 2] = unsafe { core::mem::zeroed() };
pub static mut SP: [HSD_Spline; 1] = unsafe { core::mem::zeroed() };
pub fn pick_HSD_GObj() -> *mut HSD_GObj { let r = rnd(); unsafe { &raw mut GO[((r >> 8) % 4) as usize] } }
pub fn picko_HSD_GObj() -> *mut HSD_GObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut GO[((r >> 8) % 4) as usize] } } }
pub fn pick_LbShadow() -> *mut LbShadow { let r = rnd(); unsafe { &raw mut LS[((r >> 8) % 2) as usize] } }
pub fn picko_LbShadow() -> *mut LbShadow { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut LS[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_JObj() -> *mut HSD_JObj { let r = rnd(); unsafe { &raw mut JO[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_JObj() -> *mut HSD_JObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut JO[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_Shadow() -> *mut HSD_Shadow { let r = rnd(); unsafe { &raw mut SHD[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_Shadow() -> *mut HSD_Shadow { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut SHD[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_CObj() -> *mut HSD_CObj { let r = rnd(); unsafe { &raw mut CO[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_CObj() -> *mut HSD_CObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut CO[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_LObj() -> *mut HSD_LObj { let r = rnd(); unsafe { &raw mut LO[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_LObj() -> *mut HSD_LObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut LO[((r >> 8) % 3) as usize] } } }
pub fn pick_Fighter() -> *mut Fighter { let r = rnd(); unsafe { &raw mut FTR[((r >> 8) % 3) as usize] } }
pub fn picko_Fighter() -> *mut Fighter { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut FTR[((r >> 8) % 3) as usize] } } }
pub fn pick_CmSubject() -> *mut CmSubject { let r = rnd(); unsafe { &raw mut CM[((r >> 8) % 2) as usize] } }
pub fn picko_CmSubject() -> *mut CmSubject { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut CM[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_TObj() -> *mut HSD_TObj { let r = rnd(); unsafe { &raw mut TO[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_TObj() -> *mut HSD_TObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut TO[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_ImageDesc() -> *mut HSD_ImageDesc { let r = rnd(); unsafe { &raw mut ID[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_ImageDesc() -> *mut HSD_ImageDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut ID[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_Spline() -> *mut HSD_Spline { let r = rnd(); unsafe { &raw mut SP[((r >> 8) % 1) as usize] } }
pub fn picko_HSD_Spline() -> *mut HSD_Spline { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut SP[((r >> 8) % 1) as usize] } } }
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
        preg(&raw const LS, core::mem::size_of_val(&LS));
        preg(&raw const JO, core::mem::size_of_val(&JO));
        preg(&raw const SHD, core::mem::size_of_val(&SHD));
        preg(&raw const CO, core::mem::size_of_val(&CO));
        preg(&raw const LO, core::mem::size_of_val(&LO));
        preg(&raw const FTR, core::mem::size_of_val(&FTR));
        preg(&raw const CM, core::mem::size_of_val(&CM));
        preg(&raw const TO, core::mem::size_of_val(&TO));
        preg(&raw const ID, core::mem::size_of_val(&ID));
        preg(&raw const SP, core::mem::size_of_val(&SP));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS));
    }
}

pub unsafe fn setup_world() {
    GO = core::mem::zeroed();
    LS = core::mem::zeroed();
    JO = core::mem::zeroed();
    SHD = core::mem::zeroed();
    CO = core::mem::zeroed();
    LO = core::mem::zeroed();
    FTR = core::mem::zeroed();
    CM = core::mem::zeroed();
    TO = core::mem::zeroed();
    ID = core::mem::zeroed();
    SP = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..4 { g::fill_HSD_GObj(&mut GO[i]); }
    for i in 0..2 { g::fill_LbShadow(&mut LS[i]); }
    for i in 0..3 { g::fill_HSD_JObj(&mut JO[i]); }
    for i in 0..3 { g::fill_HSD_Shadow(&mut SHD[i]); }
    for i in 0..2 { g::fill_HSD_CObj(&mut CO[i]); }
    for i in 0..3 { g::fill_HSD_LObj(&mut LO[i]); }
    for i in 0..3 { g::fill_Fighter(&mut FTR[i]); }
    for i in 0..2 { g::fill_CmSubject(&mut CM[i]); }
    for i in 0..2 { g::fill_HSD_TObj(&mut TO[i]); }
    for i in 0..2 { g::fill_HSD_ImageDesc(&mut ID[i]); }
    for i in 0..1 { g::fill_HSD_Spline(&mut SP[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..4 { g::hash_HSD_GObj(&GO[i]); }
    for i in 0..2 { g::hash_LbShadow(&LS[i]); }
    for i in 0..3 { g::hash_HSD_JObj(&JO[i]); }
    for i in 0..3 { g::hash_HSD_Shadow(&SHD[i]); }
    for i in 0..2 { g::hash_HSD_CObj(&CO[i]); }
    for i in 0..3 { g::hash_HSD_LObj(&LO[i]); }
    for i in 0..3 { g::hash_Fighter(&FTR[i]); }
    for i in 0..2 { g::hash_CmSubject(&CM[i]); }
    for i in 0..2 { g::hash_HSD_TObj(&TO[i]); }
    for i in 0..2 { g::hash_HSD_ImageDesc(&ID[i]); }
    for i in 0..1 { g::hash_HSD_Spline(&SP[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    g::hashg_all();
    g::hasho_all();
}

// harness = false (Cargo.toml): a case that ends in a panic is cut off inside the hook (see mtu_harness::run_case),
// which a libtest main thread cannot survive at exit.
fn main() { lbshadow_matches_c(); println!("tu_lbshadow: ok"); }

fn lbshadow_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();
preg(&raw const PLH, core::mem::size_of_val(&PLH)); preg(&raw const GXH, core::mem::size_of_val(&GXH));
        install_abort_hook();
        let mut c = Check::new(include_str!("data/mtu_lbshadow.txt"));
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
{ for q in 0..16usize { PLH[q] = null_mut(); } for q in 0..8usize { GXH[q] = null_mut(); } PLH[8] = if fv() & 7 != 0 { &raw mut GO[0] } else { null_mut() }; GXH[4] = if fv() & 7 != 0 { &raw mut GO[(fv() & 3) as usize] } else { null_mut() }; m::ext::HSD_GObjPLinkHead = (&raw mut PLH) as *mut *mut HSD_GObj; m::ext::HSD_GObjGXLinkHead = (&raw mut GXH) as *mut *mut HSD_GObj; for q in 0..4usize { GO[q].next = if q < 3 { &raw mut GO[q + 1] } else { null_mut() }; GO[q].next_gx = if q < 3 { &raw mut GO[q + 1] } else { null_mut() }; GO[q].user_data = (&raw mut FTR[q % 3]) as *mut c_void; GO[q].hsd_obj = (&raw mut LO[q % 3]) as *mut c_void; } for q in 0..3usize { LO[q].next = if q < 2 { &raw mut LO[q + 1] } else { null_mut() }; } for q in 0..3usize { FTR[q].x890_cameraBox = if fv() & 3 != 0 { &raw mut CM[(fv() & 1) as usize] } else { null_mut() }; FTR[q].x20A4.shadow = if fv() & 3 != 0 { &raw mut SHD[(fv() % 3) as usize] } else { null_mut() }; SHD[q].camera = &raw mut CO[q & 1]; SHD[q].texture = &raw mut TO[q & 1]; } for q in 0..2usize { TO[q].imagedesc = &raw mut ID[q]; } SP[0].r#type = (fv() % 5) as u8; SP[0].numcv = (1 + fv() % 6) as i16; SP[0].tension = rw(); SP[0].cv = ((&raw mut BUF) as *mut u8).add(12) as *mut Vec; for q in 0..63usize { *((&raw mut BUF) as *mut f32).add(q) = rw(); } }
                let mut ret = 0u32;
                let aborted = run_case(|| unsafe { match op {
                    0 => { 
let a0 = (&raw mut FA) as *mut Vec;
let a1 = (&raw mut VA) as *mut Vec;
let a2 = rw();
let a3 = if fv() & 7 == 0 { 1.0f32 } else { (fv() % 1001) as f32 / 1000.0f32 };
m::splGetCardinalTangent(&mut *a0, core::slice::from_raw_parts(a1, 4), a2, a3);
 }
                    1 => { 
let a0 = (&raw mut FA) as *mut Vec;
let a1 = (&raw mut VA) as *mut Vec;
let a2 = if fv() & 7 == 0 { 1.0f32 } else { (fv() % 1001) as f32 / 1000.0f32 };
m::splGetBSplineTangent(&mut *a0, core::slice::from_raw_parts(a1, 4), a2);
 }
                    2 => { 
let a0 = (&raw mut FA) as *mut Vec;
let a1 = (&raw mut VA) as *mut Vec;
let a2 = if fv() & 7 == 0 { 1.0f32 } else { (fv() % 1001) as f32 / 1000.0f32 };
m::splGetBezierTangent(&mut *a0, core::slice::from_raw_parts(a1, 4), a2);
 }
                    3 => { 
let a0 = (&raw mut FA) as *mut Vec;
let a1 = pick_HSD_Spline();
let a2 = if fv() & 7 == 0 { 1.0f32 } else { (fv() % 1001) as f32 / 1000.0f32 };
m::lbShadow_8000E9F0(&mut *a0, &*a1, a2);
 }
                    4 => { 
let a0 = pick_LbShadow();
let a1 = pick_HSD_JObj();
m::lbShadow_8000ED54(a0, a1);
 }
                    5 => { 
let a0 = pick_LbShadow();
m::lbShadow_8000EE8C(a0);
 }
                    6 => { 
let a0 = pick_HSD_GObj();
m::lbShadow_8000EEE0(a0);
 }
                    7 => { 

m::lbShadow_8000EFEC();
 }
                    8 => { 
let a0 = pick_HSD_Shadow();
m::lbShadow_8000F214(a0);
 }
                    9 => { 
let a0 = fv() as _;
m::lbShadow_8000F38C(a0);
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
