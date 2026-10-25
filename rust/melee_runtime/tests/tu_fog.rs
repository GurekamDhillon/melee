//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/fog.c.
//! src/tu/fog.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py fog`).
//! C functions of the TU this test covers (the private ones through their public callers): FogAdjInfoInit, FogInfoInit, FogRelease, FogUpdateFunc, HSD_FogAdjAlloc, HSD_FogAdjInit, HSD_FogAdjLoadDesc, HSD_FogAlloc, HSD_FogInit, HSD_FogInterpretAnim, HSD_FogLoadDesc, HSD_FogReqAnim, HSD_FogReqAnimByFlags, HSD_FogSet, HSD_Fog_8037DE7C.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_HSD_FogSet: u32 = 0;
const OP_HSD_FogAlloc: u32 = 1;
const OP_HSD_FogLoadDesc: u32 = 2;
const OP_HSD_FogInit: u32 = 3;
const OP_HSD_FogAdjAlloc: u32 = 4;
const OP_HSD_FogAdjLoadDesc: u32 = 5;
const OP_HSD_FogAdjInit: u32 = 6;
const OP_HSD_Fog_8037DE7C: u32 = 7;
const OP_HSD_FogReqAnim: u32 = 8;
const OP_HSD_FogReqAnimByFlags: u32 = 9;
const OP_HSD_FogInterpretAnim: u32 = 10;
const OP_FogUpdateFunc: u32 = 11;
const OP_FogRelease: u32 = 12;
const OP_FogInfoInit: u32 = 13;
const OP_FogAdjInfoInit: u32 = 14;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::fog as m;
use mtu_harness::*;
#[path = "mtu/fog_gen.rs"]
mod g;


unsafe extern "C" fn cb_rel(c: *mut HSD_Class) { t(8203); t(pi(c)); }
unsafe extern "C" fn cb_dest(c: *mut HSD_Class) { t(8204); t(pi(c)); }


pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut PPA: [*mut c_void; 4] = [core::ptr::null_mut(); 4];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub static mut FG: [HSD_Fog; 3] = unsafe { core::mem::zeroed() };
pub static mut FGA: [HSD_FogAdj; 3] = unsafe { core::mem::zeroed() };
pub static mut FD: [HSD_FogDesc; 2] = unsafe { core::mem::zeroed() };
pub static mut FAD: [HSD_FogAdjDesc; 2] = unsafe { core::mem::zeroed() };
pub static mut AO: [HSD_AObj; 2] = unsafe { core::mem::zeroed() };
pub static mut AD: [HSD_AObjDesc; 2] = unsafe { core::mem::zeroed() };
pub static mut OBD: [HSD_ObjData; 2] = unsafe { core::mem::zeroed() };
pub static mut CI: [HSD_ClassInfo; 3] = unsafe { core::mem::zeroed() };
pub static mut FIN: [HSD_FogInfo; 2] = unsafe { core::mem::zeroed() };
pub fn pick_HSD_Fog() -> *mut HSD_Fog { let r = rnd(); unsafe { &raw mut FG[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_Fog() -> *mut HSD_Fog { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut FG[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_FogAdj() -> *mut HSD_FogAdj { let r = rnd(); unsafe { &raw mut FGA[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_FogAdj() -> *mut HSD_FogAdj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut FGA[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_FogDesc() -> *mut HSD_FogDesc { let r = rnd(); unsafe { &raw mut FD[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_FogDesc() -> *mut HSD_FogDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut FD[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_FogAdjDesc() -> *mut HSD_FogAdjDesc { let r = rnd(); unsafe { &raw mut FAD[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_FogAdjDesc() -> *mut HSD_FogAdjDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut FAD[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_AObj() -> *mut HSD_AObj { let r = rnd(); unsafe { &raw mut AO[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_AObj() -> *mut HSD_AObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut AO[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_AObjDesc() -> *mut HSD_AObjDesc { let r = rnd(); unsafe { &raw mut AD[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_AObjDesc() -> *mut HSD_AObjDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut AD[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_ObjData() -> *mut HSD_ObjData { let r = rnd(); unsafe { &raw mut OBD[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_ObjData() -> *mut HSD_ObjData { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut OBD[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_ClassInfo() -> *mut HSD_ClassInfo { let r = rnd(); unsafe { &raw mut CI[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_ClassInfo() -> *mut HSD_ClassInfo { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut CI[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_FogInfo() -> *mut HSD_FogInfo { let r = rnd(); unsafe { &raw mut FIN[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_FogInfo() -> *mut HSD_FogInfo { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut FIN[((r >> 8) % 2) as usize] } } }
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
        preg(&raw const FG, core::mem::size_of_val(&FG));
        preg(&raw const FGA, core::mem::size_of_val(&FGA));
        preg(&raw const FD, core::mem::size_of_val(&FD));
        preg(&raw const FAD, core::mem::size_of_val(&FAD));
        preg(&raw const AO, core::mem::size_of_val(&AO));
        preg(&raw const AD, core::mem::size_of_val(&AD));
        preg(&raw const OBD, core::mem::size_of_val(&OBD));
        preg(&raw const CI, core::mem::size_of_val(&CI));
        preg(&raw const FIN, core::mem::size_of_val(&FIN));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS)); preg(&raw const PPA, core::mem::size_of_val(&PPA));
    }
}

pub unsafe fn setup_world() {
    FG = core::mem::zeroed();
    FGA = core::mem::zeroed();
    FD = core::mem::zeroed();
    FAD = core::mem::zeroed();
    AO = core::mem::zeroed();
    AD = core::mem::zeroed();
    OBD = core::mem::zeroed();
    CI = core::mem::zeroed();
    FIN = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..4 { PPA[i] = core::ptr::null_mut(); }
    for i in 0..3 { g::fill_HSD_Fog(&mut FG[i]); }
    for i in 0..3 { g::fill_HSD_FogAdj(&mut FGA[i]); }
    for i in 0..2 { g::fill_HSD_FogDesc(&mut FD[i]); }
    for i in 0..2 { g::fill_HSD_FogAdjDesc(&mut FAD[i]); }
    for i in 0..2 { g::fill_HSD_AObj(&mut AO[i]); }
    for i in 0..2 { g::fill_HSD_AObjDesc(&mut AD[i]); }
    for i in 0..2 { g::fill_HSD_ObjData(&mut OBD[i]); }
    for i in 0..3 { g::fill_HSD_ClassInfo(&mut CI[i]); }
    for i in 0..2 { g::fill_HSD_FogInfo(&mut FIN[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..3 { g::hash_HSD_Fog(&FG[i]); }
    for i in 0..3 { g::hash_HSD_FogAdj(&FGA[i]); }
    for i in 0..2 { g::hash_HSD_FogDesc(&FD[i]); }
    for i in 0..2 { g::hash_HSD_FogAdjDesc(&FAD[i]); }
    for i in 0..2 { g::hash_HSD_AObj(&AO[i]); }
    for i in 0..2 { g::hash_HSD_AObjDesc(&AD[i]); }
    for i in 0..2 { g::hash_HSD_ObjData(&OBD[i]); }
    for i in 0..3 { g::hash_HSD_ClassInfo(&CI[i]); }
    for i in 0..2 { g::hash_HSD_FogInfo(&FIN[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    for i in 0..4 { hm(pi(PPA[i])); }
    g::hashg_all();
    g::hasho_all();
}

// harness = false (Cargo.toml): a case that ends in a panic is cut off inside the hook (see mtu_harness::run_case),
// which a libtest main thread cannot survive at exit.
fn main() { fog_matches_c(); println!("tu_fog: ok"); }

fn fog_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();
freg(cb_rel as usize); freg(cb_dest as usize); m::use_ext_data = true; melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());
        install_abort_hook();
        let mut c = Check::new(include_str!("data/mtu_fog.txt"));
        let mut total = 0u32;
        for op in 0..15u32 {
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
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
{ for q in 0..3usize { FG[q].fog_adj = if fv() & 1 != 0 { &raw mut FGA[(fv() % 3) as usize] } else { null_mut() }; FG[q].aobj = if fv() & 1 != 0 { &raw mut AO[(fv() & 1) as usize] } else { null_mut() }; FG[q].parent.parent.class_info = &raw mut CI[0]; FGA[q].parent.parent.class_info = &raw mut CI[0]; } for q in 0..2usize { FD[q].fogadjdesc = if fv() & 1 != 0 { &raw mut FAD[(fv() & 1) as usize] } else { null_mut() }; } m::hsdFog.parent.parent.head.parent = &raw mut CI[1]; for q in 0..3usize { CI[q].release = Some(cb_rel); CI[q].destroy = Some(cb_dest); } }
                let mut ret = 0u32;
                let aborted = run_case(|| unsafe { match op {
                    0 => { 
let a0 = pick_HSD_Fog();
m::HSD_FogSet(a0);
 }
                    1 => { 

ret = pi(m::HSD_FogAlloc());
 }
                    2 => { 
let a0 = pick_HSD_FogDesc();
ret = pi(m::HSD_FogLoadDesc(a0));
 }
                    3 => { 
let a0 = pick_HSD_Fog();
let a1 = pick_HSD_FogDesc();
m::HSD_FogInit(a0, a1);
 }
                    4 => { 

ret = pi(m::HSD_FogAdjAlloc());
 }
                    5 => { 
let a0 = pick_HSD_FogAdjDesc();
ret = pi(m::HSD_FogAdjLoadDesc(a0));
 }
                    6 => { 
let a0 = pick_HSD_FogAdj();
let a1 = pick_HSD_FogAdjDesc();
m::HSD_FogAdjInit(a0, a1);
 }
                    7 => { 
let a0 = pick_HSD_Fog();
let a1 = pick_HSD_AObjDesc();
m::HSD_Fog_8037DE7C(a0, a1);
 }
                    8 => { 
let a0 = pick_HSD_Fog();
let a1 = rw();
m::HSD_FogReqAnim(a0, a1);
 }
                    9 => { 
let a0 = pick_HSD_Fog();
let a1 = fv() as _;
let a2 = rw();
m::HSD_FogReqAnimByFlags(a0, a1, a2);
 }
                    10 => { 
let a0 = pick_HSD_Fog();
m::HSD_FogInterpretAnim(a0);
 }
                    11 => { 
let a0 = picko_HSD_Fog() as *mut c_void;
let a1 = fv() as _;
let a2 = pick_HSD_ObjData();
m::FogUpdateFunc(a0, a1, a2);
 }
                    12 => { 
let a0 = pick_HSD_Fog() as *mut HSD_Class;
m::FogRelease(a0);
 }
                    13 => { 

m::FogInfoInit();
 }
                    14 => { 

m::FogAdjInfoInit();
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
