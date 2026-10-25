//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/lbanim.c.
//! src/tu/lbanim.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py lbanim`).
//! C functions of the TU this test covers (the private ones through their public callers): fn_8001E60C, lbAnim_8001E6D8, lbAnim_8001E7E8, lbAnim_8001E8F8, lbAnim_InitFrames, lbAnim_JObjSortAnim.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_lbAnim_InitFrames: u32 = 0;
const OP_fn_8001E60C: u32 = 1;
const OP_lbAnim_8001E6D8: u32 = 2;
const OP_lbAnim_8001E7E8: u32 = 3;
const OP_lbAnim_8001E8F8: u32 = 4;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::lbanim as m;
use mtu_harness::*;
#[path = "mtu/lbanim_gen.rs"]
mod g;

static mut FOI: usize = 0; static mut AOI: usize = 0;

pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub static mut TRK: [FigaTrack; 4] = unsafe { core::mem::zeroed() };
pub static mut TRE: [FigaTree; 2] = unsafe { core::mem::zeroed() };
pub static mut JO: [HSD_JObj; 2] = unsafe { core::mem::zeroed() };
pub static mut AO: [HSD_AObj; 4] = unsafe { core::mem::zeroed() };
pub static mut FO: [HSD_FObj; 4] = unsafe { core::mem::zeroed() };
pub fn pick_FigaTrack() -> *mut FigaTrack { let r = rnd(); unsafe { &raw mut TRK[((r >> 8) % 4) as usize] } }
pub fn picko_FigaTrack() -> *mut FigaTrack { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut TRK[((r >> 8) % 4) as usize] } } }
pub fn pick_FigaTree() -> *mut FigaTree { let r = rnd(); unsafe { &raw mut TRE[((r >> 8) % 2) as usize] } }
pub fn picko_FigaTree() -> *mut FigaTree { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut TRE[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_JObj() -> *mut HSD_JObj { let r = rnd(); unsafe { &raw mut JO[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_JObj() -> *mut HSD_JObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut JO[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_AObj() -> *mut HSD_AObj { let r = rnd(); unsafe { &raw mut AO[((r >> 8) % 4) as usize] } }
pub fn picko_HSD_AObj() -> *mut HSD_AObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut AO[((r >> 8) % 4) as usize] } } }
pub fn pick_HSD_FObj() -> *mut HSD_FObj { let r = rnd(); unsafe { &raw mut FO[((r >> 8) % 4) as usize] } }
pub fn picko_HSD_FObj() -> *mut HSD_FObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut FO[((r >> 8) % 4) as usize] } } }
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
        preg(&raw const TRK, core::mem::size_of_val(&TRK));
        preg(&raw const TRE, core::mem::size_of_val(&TRE));
        preg(&raw const JO, core::mem::size_of_val(&JO));
        preg(&raw const AO, core::mem::size_of_val(&AO));
        preg(&raw const FO, core::mem::size_of_val(&FO));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS));
    }
}

pub unsafe fn setup_world() {
    TRK = core::mem::zeroed();
    TRE = core::mem::zeroed();
    JO = core::mem::zeroed();
    AO = core::mem::zeroed();
    FO = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..4 { g::fill_FigaTrack(&mut TRK[i]); }
    for i in 0..2 { g::fill_FigaTree(&mut TRE[i]); }
    for i in 0..2 { g::fill_HSD_JObj(&mut JO[i]); }
    for i in 0..4 { g::fill_HSD_AObj(&mut AO[i]); }
    for i in 0..4 { g::fill_HSD_FObj(&mut FO[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..4 { g::hash_FigaTrack(&TRK[i]); }
    for i in 0..2 { g::hash_FigaTree(&TRE[i]); }
    for i in 0..2 { g::hash_HSD_JObj(&JO[i]); }
    for i in 0..4 { g::hash_HSD_AObj(&AO[i]); }
    for i in 0..4 { g::hash_HSD_FObj(&FO[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    g::hashg_all();
    g::hasho_all();
}

// harness = false (Cargo.toml): a case that ends in a panic is cut off inside the hook (see mtu_harness::run_case),
// which a libtest main thread cannot survive at exit.
fn main() { lbanim_matches_c(); println!("tu_lbanim: ok"); }

fn lbanim_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();

        install_abort_hook();
        let mut c = Check::new(include_str!("data/mtu_lbanim.txt"));
        let mut total = 0u32;
        for op in 0..5u32 {
            let n = match op {
                0 => 400u32,
                1 => 400u32,
                2 => 400u32,
                3 => 400u32,
                4 => 400u32,
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
{ FOI = 0; AOI = 0; for q in 0..4usize { TRK[q].obj_type = if fv() % 4 != 0 { 1 + (fv() % 4) as u8 } else { 8 + (fv() % 20) as u8 }; } for q in 0..2usize { JO[q].aobj = if fv() & 1 != 0 { &raw mut AO[3] } else { null_mut() }; } }
                let mut ret = 0u32;
                let aborted = run_case(|| unsafe { match op {
                    0 => { 
let a0 = &raw mut TRK[0];
let a1 = (1 + fv() % 3) as i8;
ret = pi(m::lbAnim_InitFrames(a0, a1));
 }
                    1 => { 
let a0 = &raw mut TRK[0];
let a1 = (1 + fv() % 3) as i8;
ret = pi(m::fn_8001E60C(a0, a1));
 }
                    2 => { 
let a0 = pick_HSD_JObj();
let a1 = pick_FigaTree();
let a2 = &raw mut TRK[0];
let a3 = (fv() % 4) as i8;
m::lbAnim_8001E6D8(a0, a1, a2, a3);
 }
                    3 => { 
let a0 = pick_HSD_JObj();
let a1 = pick_FigaTree();
let a2 = &raw mut TRK[0];
let a3 = (fv() % 4) as i8;
m::lbAnim_8001E7E8(a0, a1, a2, a3);
 }
                    4 => { 
let a0 = pick_FigaTree();
ret = (m::lbAnim_8001E8F8(if a0.is_null() { None } else { Some(&*a0) })).to_bits();
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
