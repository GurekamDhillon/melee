//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/lbdvd.c.
//! src/tu/lbdvd.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py lbdvd`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_lbDvd_IsPlanningSixSlot: u32 = 0;
const OP_lbDvd_TrySetupSixSlotCache: u32 = 1;
const OP_lbDvd_SixBudgetTest: u32 = 2;
const OP_lbDvd_SetupVsPreloadCache: u32 = 3;
const OP_lbDvd_800174E8: u32 = 4;
const OP_lbDvd_80017598: u32 = 5;
const OP_lbDvd_80017644: u32 = 6;
const OP_lbDvd_80017700: u32 = 7;
const OP_lbDvd_80017740: u32 = 8;
const OP_lbDvd_800178E8: u32 = 9;
const OP_lbDvd_80017960: u32 = 10;
const OP_lbDvd_CachePreloadedFile: u32 = 11;
const OP_lbDvd_80017CC4: u32 = 12;
const OP_lbDvd_80017E64: u32 = 13;
const OP_lbDvd_GetPreloadedArchive: u32 = 14;
const OP_lbDvd_GetPreloadCacheScene: u32 = 15;
const OP_lbDvd_8001823C: u32 = 16;
const OP_lbDvd_80018254: u32 = 17;
const OP_lbDvd_800187F4: u32 = 18;
const OP_lbDvd_80018A2C: u32 = 19;
const OP_lbDvd_80018C2C: u32 = 20;
const OP_lbDvd_80018C6C: u32 = 21;
const OP_lbDvd_80018CF4: u32 = 22;
const OP_lbDvd_80018F58: u32 = 23;
const OP_lbDvd_80018F68: u32 = 24;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::lbdvd as m;
use mtu_harness::*;
#[path = "mtu/lbdvd_gen.rs"]
mod g;



pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub static mut AE: [HSD_AllocEntry; 8] = unsafe { core::mem::zeroed() };
pub fn pick_HSD_AllocEntry() -> *mut HSD_AllocEntry { let r = rnd(); unsafe { &raw mut AE[((r >> 8) % 8) as usize] } }
pub fn picko_HSD_AllocEntry() -> *mut HSD_AllocEntry { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut AE[((r >> 8) % 8) as usize] } } }
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
        preg(&raw const AE, core::mem::size_of_val(&AE));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS));
    }
}

pub unsafe fn setup_world() {
    AE = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..8 { g::fill_HSD_AllocEntry(&mut AE[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..8 { g::hash_HSD_AllocEntry(&AE[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    g::hashg_all();
    g::hasho_all();
}

// harness = false (Cargo.toml): a case that ends in a panic is cut off inside the hook (see mtu_harness::run_case),
// which a libtest main thread cannot survive at exit.
fn main() { lbdvd_matches_c(); println!("tu_lbdvd: ok"); }

fn lbdvd_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();

        install_abort_hook();
        let mut c = Check::new(include_str!("data/mtu_lbdvd.txt"));
        let mut total = 0u32;
        for op in 0..25u32 {
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
                23 => 300u32,
                24 => 300u32,
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
m::ext::hook::OSReport = Some(|_| {});
                let mut ret = 0u32;
                let aborted = run_case(|| unsafe { match op {
                    0 => { 

ret = m::lbDvd_IsPlanningSixSlot() as u32;
 }
                    1 => { 

ret = m::lbDvd_TrySetupSixSlotCache() as u32;
 }
                    2 => { 

ret = m::lbDvd_SixBudgetTest() as u32;
 }
                    3 => { 

m::lbDvd_SetupVsPreloadCache();
 }
                    4 => { 
let a0 = (fv() % 80) as i32;
m::lbDvd_800174E8(a0);
 }
                    5 => { 
let a0 = (fv() % 8) as i32;
ret = m::lbDvd_80017598(a0) as u32;
 }
                    6 => { 
let a0 = (fv() % 8) as i32;
ret = m::lbDvd_80017644(a0) as u32;
 }
                    7 => { 
let a0 = (fv() % 8) as i32;
m::lbDvd_80017700(a0);
 }
                    8 => { 
let a0 = fv() as _;
let a1 = fv() as _;
let a2 = fv() as _;
let a3 = fv() as _;
let a4 = fv() as _;
let a5 = fv() as _;
let a6 = fv() as _;
let a7 = fv() as _;
let a8 = fv() as _;
{ let p = m::lbDvd_80017740(a0, a1, a2, a3, a4, a5, a6, a7, a8) as *mut PreloadEntry; ret = if p.is_null() { 0 } else { p.offset_from(m::preloadCache.entries.as_ptr()) as u32 + 1 }; }
 }
                    9 => { 
let a0 = fv() as _;
let a1 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
let a2 = fv() as _;
let a3 = fv() as _;
let a4 = fv() as _;
let a5 = fv() as _;
let a6 = fv() as _;
let a7 = fv() as _;
let a8 = fv() as _;
m::lbDvd_800178E8(a0, a1, a2, a3, a4, a5, a6, a7, a8);
 }
                    10 => { 

m::lbDvd_80017960();
 }
                    11 => { 
let a0 = (fv() % 80) as i32;
m::lbDvd_CachePreloadedFile(a0);
 }
                    12 => { 

m::lbDvd_80017CC4();
 }
                    13 => { 
let a0 = fv() as _;
let a1 = (fv() % 80) as usize;
let a2 = (&raw mut BUF) as *mut c_void;
let a3 = fv() & 1 != 0;
m::lbDvd_80017E64(a0, a1, a2, a3);
 }
                    14 => { 
let a0 = (fv() % 90) as isize - 5;
ret = pi(m::lbDvd_GetPreloadedArchive(a0));
 }
                    15 => { 

ret = pi(m::lbDvd_GetPreloadCacheScene());
 }
                    16 => { 

m::lbDvd_8001823C();
 }
                    17 => { 

m::lbDvd_80018254();
 }
                    18 => { 
let a0 = (fv() % 90) as i32 - 5;
ret = m::lbDvd_800187F4(a0) as u32;
 }
                    19 => { 
let a0 = fv() as _;
ret = m::lbDvd_80018A2C(a0) as u32;
 }
                    20 => { 
let a0 = fv() as _;
m::lbDvd_80018C2C(a0);
 }
                    21 => { 

m::lbDvd_80018C6C();
 }
                    22 => { 
let a0 = fv() as _;
m::lbDvd_80018CF4(a0);
 }
                    23 => { 
let a0 = fv() & 1 != 0;
m::lbDvd_80018F58(a0);
 }
                    24 => { 

m::lbDvd_80018F68();
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
