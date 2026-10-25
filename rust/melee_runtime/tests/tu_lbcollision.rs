//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/lbcollision.c.
//! src/tu/lbcollision.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py lbcollision`).
//! C functions of the TU this test covers (the private ones through their public callers): end, isSmall, lbColl_80005BB0, lbColl_80005C44, lbColl_80005EBC, lbColl_80005FC0, lbColl_80006094, lbColl_800067F8, lbColl_80006E58, lbColl_800077A0, lbColl_80007AFC, lbColl_80007B78, lbColl_80007BCC, lbColl_80007DD8, lbColl_80007ECC, lbColl_8000805C, lbColl_80008248, lbColl_800083C4, lbColl_80008428, lbColl_80008434, lbColl_80008440, lbColl_80008688, lbColl_80008820, lbColl_800089B8, lbColl_80008A5C, lbColl_80008D30, lbColl_80008DA4, lbColl_80008FC8, lbColl_800096B4, lbColl_80009DD4, lbColl_80009F54, lbColl_8000A044, lbColl_8000A10C, lbColl_8000A1A8, lbColl_8000A244, lbColl_8000A460, lbColl_8000A584, lbColl_8000A78C, lbColl_8000A95C, lbColl_8000AB2C, lbColl_8000ACFC, lbColl_CopyHitCapsule, lbColl_DifferenceY, lbColl_DrawHit, lbColl_DrawHitResult, lbColl_GetY, sqrDistance, sqrtf_store.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_lbColl_80006E58: u32 = 0;
const OP_lbColl_80007BCC: u32 = 1;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::lbcollision as m;
use mtu_harness::*;
#[path = "mtu/lbcollision_gen.rs"]
mod g;



pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut PPA: [*mut c_void; 4] = [core::ptr::null_mut(); 4];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub static mut P0: [HitCapsule; 2] = unsafe { core::mem::zeroed() };
pub static mut P2: [HitResult; 2] = unsafe { core::mem::zeroed() };
pub static mut P8: [HSD_JObj; 2] = unsafe { core::mem::zeroed() };
pub fn pick_HitCapsule() -> *mut HitCapsule { let r = rnd(); unsafe { &raw mut P0[((r >> 8) % 2) as usize] } }
pub fn picko_HitCapsule() -> *mut HitCapsule { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut P0[((r >> 8) % 2) as usize] } } }
pub fn pick_HitResult() -> *mut HitResult { let r = rnd(); unsafe { &raw mut P2[((r >> 8) % 2) as usize] } }
pub fn picko_HitResult() -> *mut HitResult { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut P2[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_JObj() -> *mut HSD_JObj { let r = rnd(); unsafe { &raw mut P8[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_JObj() -> *mut HSD_JObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut P8[((r >> 8) % 2) as usize] } } }
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
        preg(&raw const P2, core::mem::size_of_val(&P2));
        preg(&raw const P8, core::mem::size_of_val(&P8));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS)); preg(&raw const PPA, core::mem::size_of_val(&PPA));
    }
}

pub unsafe fn setup_world() {
    P0 = core::mem::zeroed();
    P2 = core::mem::zeroed();
    P8 = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..4 { PPA[i] = core::ptr::null_mut(); }
    for i in 0..2 { g::fill_HitCapsule(&mut P0[i]); }
    for i in 0..2 { g::fill_HitResult(&mut P2[i]); }
    for i in 0..2 { g::fill_HSD_JObj(&mut P8[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..2 { g::hash_HitCapsule(&P0[i]); }
    for i in 0..2 { g::hash_HitResult(&P2[i]); }
    for i in 0..2 { g::hash_HSD_JObj(&P8[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    for i in 0..4 { hm(pi(PPA[i])); }
    g::hashg_all();
    g::hasho_all();
}

// harness = false (Cargo.toml): a case that ends in a panic is cut off inside the hook (see mtu_harness::run_case),
// which a libtest main thread cannot survive at exit.
fn main() { lbcollision_matches_c(); println!("tu_lbcollision: ok"); }

fn lbcollision_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();

        install_abort_hook();
        let mut c = Check::new(include_str!("data/mtu_lbcollision.txt"));
        let mut total = 0u32;
        for op in 0..2u32 {
            let n = match op {
                0 => 1500u32,
                1 => 1500u32,
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
{ for q in 0..2usize { P2[q].bone = &raw mut P8[q]; P2[q].set_skip_update_pos((fv() & 1) as _); } }
                let mut ret = 0u32;
                let aborted = run_case(|| unsafe { match op {
                    0 => { 
let a0 = &raw mut VA[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = &raw mut VA[(rnd() % 4) as usize];
let a3 = &raw mut VA[(rnd() % 4) as usize];
let a4 = &raw mut VA[(rnd() % 4) as usize];
let a5 = &raw mut VA[(rnd() % 4) as usize];
let a6 = (&raw mut BUF) as *mut _;
let a7 = &raw mut VA[(rnd() % 4) as usize];
let a8 = &raw mut FA[(rnd() % 4) as usize];
let a9 = rw();
let a10 = rw();
let a11 = rw();
ret = m::lbColl_80006E58(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11) as u32;
 }
                    1 => { 
let a0 = pick_HitCapsule();
let a1 = pick_HitResult();
let a2 = if fv() & 1 != 0 { (&raw mut BUF) as *const Mtx } else { core::ptr::null() };
let a3 = fv() as _;
let a4 = rw();
let a5 = rw();
let a6 = rw();
ret = m::lbColl_80007BCC(a0, a1, a2, a3, a4, a5, a6) as u32;
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
