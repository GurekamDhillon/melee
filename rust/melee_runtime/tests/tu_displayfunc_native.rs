//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/displayfunc_native.c.
//! src/tu/displayfunc_native.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py displayfunc_native`).
//! C functions of the TU this test covers (the private ones through their public callers): HSD_EraseRect, HSD_JObjDisp, HSD_JObjDispDObj, HSD_JObjDispSub, HSD_JObjFindSkeleton, HSD_JObjMakePositionMtx, HSD_JObjSetSPtclCallback, HSD_SetEraseColor, HSD_StateInitDirect, HSD_ZListAlloc, HSD_ZListFree, HSD_ZListInitAllocData, _HSD_DispForgetMemory, _HSD_ZListClear, _HSD_ZListDisp, _HSD_ZListSort, _HSD_mkEnvelopeModelNodeMtx, mkBillBoardMtx, mkHBillBoardMtx, mkRBillBoardMtx, mkVBillBoardMtx, zlist_sort.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_HSD_ZListInitAllocData: u32 = 0;
const OP_HSD_StateInitDirect: u32 = 1;
const OP_HSD_JObjMakePositionMtx: u32 = 2;
const OP_HSD_JObjFindSkeleton: u32 = 3;
const OP__HSD_mkEnvelopeModelNodeMtx: u32 = 4;
const OP_HSD_JObjDispSub: u32 = 5;
const OP_HSD_JObjDispDObj: u32 = 6;
const OP__HSD_ZListSort: u32 = 7;
const OP__HSD_ZListDisp: u32 = 8;
const OP__HSD_ZListClear: u32 = 9;
const OP_HSD_JObjDisp: u32 = 10;
const OP_HSD_JObjSetSPtclCallback: u32 = 11;
const OP_HSD_SetEraseColor: u32 = 12;
const OP_HSD_EraseRect: u32 = 13;
const OP__HSD_DispForgetMemory: u32 = 14;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::displayfunc_native as m;
use mtu_harness::*;
#[path = "mtu/displayfunc_native_gen.rs"]
mod g;


use m::HSD_ZList;
static mut MT: [[u8; 48]; 4] = [[0; 48]; 4];
unsafe extern "C" fn cb_pmtx(j: *mut HSD_JObj, v: *mut [f32; 4], m: *mut [f32; 4]) { t(8210); t(pi(j)); t(pi(v)); t(pi(m)); }
unsafe extern "C" fn cb_jdisp(j: *mut HSD_JObj, v: *mut [f32; 4], m: *mut [f32; 4], trsp: u32, r: u32) { t(8211); t(pi(j)); t(pi(v)); t(pi(m)); t(trsp); t(r); }
unsafe extern "C" fn cb_ddisp(d: *mut HSD_DObj, v: *mut [f32; 4], m: *mut [f32; 4], r: u32) { t(8212); t(pi(d)); t(pi(v)); t(pi(m)); t(r); }
unsafe extern "C" fn cb_sp(a: i32, b: i32, c: i32, j: *mut HSD_JObj) { t(8213); t(a as u32); t(b as u32); t(c as u32); t(pi(j)); }
static mut ZK: u32 = 0;


pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut PPA: [*mut c_void; 4] = [core::ptr::null_mut(); 4];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub static mut J: [HSD_JObj; 4] = unsafe { core::mem::zeroed() };
pub static mut JI: [HSD_JObjInfo; 3] = unsafe { core::mem::zeroed() };
pub static mut DO: [HSD_DObj; 3] = unsafe { core::mem::zeroed() };
pub static mut DI: [HSD_DObjInfo; 2] = unsafe { core::mem::zeroed() };
pub static mut CO: [HSD_CObj; 2] = unsafe { core::mem::zeroed() };
pub static mut SL: [HSD_SList; 3] = unsafe { core::mem::zeroed() };
pub static mut ZL: [HSD_ZList; 4] = unsafe { core::mem::zeroed() };
pub static mut CI: [HSD_ClassInfo; 2] = unsafe { core::mem::zeroed() };
pub fn pick_HSD_JObj() -> *mut HSD_JObj { let r = rnd(); unsafe { &raw mut J[((r >> 8) % 4) as usize] } }
pub fn picko_HSD_JObj() -> *mut HSD_JObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut J[((r >> 8) % 4) as usize] } } }
pub fn pick_HSD_JObjInfo() -> *mut HSD_JObjInfo { let r = rnd(); unsafe { &raw mut JI[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_JObjInfo() -> *mut HSD_JObjInfo { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut JI[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_DObj() -> *mut HSD_DObj { let r = rnd(); unsafe { &raw mut DO[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_DObj() -> *mut HSD_DObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut DO[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_DObjInfo() -> *mut HSD_DObjInfo { let r = rnd(); unsafe { &raw mut DI[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_DObjInfo() -> *mut HSD_DObjInfo { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut DI[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_CObj() -> *mut HSD_CObj { let r = rnd(); unsafe { &raw mut CO[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_CObj() -> *mut HSD_CObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut CO[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_SList() -> *mut HSD_SList { let r = rnd(); unsafe { &raw mut SL[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_SList() -> *mut HSD_SList { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut SL[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_ZList() -> *mut HSD_ZList { let r = rnd(); unsafe { &raw mut ZL[((r >> 8) % 4) as usize] } }
pub fn picko_HSD_ZList() -> *mut HSD_ZList { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut ZL[((r >> 8) % 4) as usize] } } }
pub fn pick_HSD_ClassInfo() -> *mut HSD_ClassInfo { let r = rnd(); unsafe { &raw mut CI[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_ClassInfo() -> *mut HSD_ClassInfo { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut CI[((r >> 8) % 2) as usize] } } }
pub unsafe extern "C" fn cbev1(g: *mut HSD_GObj) { t(7001); t(pi(g)); }
pub unsafe extern "C" fn cbev2(g: *mut HSD_GObj) { t(7002); t(pi(g)); }
pub unsafe extern "C" fn cbp1(g: *mut HSD_GObj) -> i32 { t(7003); t(pi(g)); (fv() & 1) as i32 }
pub unsafe extern "C" fn cbp2(g: *mut HSD_GObj) -> i32 { t(7004); t(pi(g)); (fv() & 1) as i32 }
pub unsafe extern "C" fn cbev3(g: *mut HSD_GObj, h: *mut HSD_GObj) { t(7005); t(pi(g)); t(pi(h)); }
pub unsafe extern "C" fn cbi1(a: i32) { t(7006); t(a as u32); }
pub unsafe extern "C" fn cbi2(a: i32) { t(7007); t(a as u32); }
pub unsafe extern "C" fn autocb_0(a0: i32, a1: i32, a2: i32, a3: *mut HSD_JObj) { t(9000); t(a0 as u32); t(a1 as u32); t(a2 as u32); t(pi(a3)); }
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
pub fn freg_world() { freg(cbev1 as usize); freg(cbev2 as usize); freg(cbp1 as usize); freg(cbp2 as usize); freg(cbev3 as usize); freg(cbi1 as usize); freg(cbi2 as usize); freg(autocb_0 as usize); }
pub fn preg_world() {
    unsafe {
        preg(&raw const J, core::mem::size_of_val(&J));
        preg(&raw const JI, core::mem::size_of_val(&JI));
        preg(&raw const DO, core::mem::size_of_val(&DO));
        preg(&raw const DI, core::mem::size_of_val(&DI));
        preg(&raw const CO, core::mem::size_of_val(&CO));
        preg(&raw const SL, core::mem::size_of_val(&SL));
        preg(&raw const ZL, core::mem::size_of_val(&ZL));
        preg(&raw const CI, core::mem::size_of_val(&CI));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS)); preg(&raw const PPA, core::mem::size_of_val(&PPA));
    }
}

pub unsafe fn setup_world() {
    J = core::mem::zeroed();
    JI = core::mem::zeroed();
    DO = core::mem::zeroed();
    DI = core::mem::zeroed();
    CO = core::mem::zeroed();
    SL = core::mem::zeroed();
    ZL = core::mem::zeroed();
    CI = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..4 { PPA[i] = core::ptr::null_mut(); }
    for i in 0..4 { g::fill_HSD_JObj(&mut J[i]); }
    for i in 0..3 { g::fill_HSD_JObjInfo(&mut JI[i]); }
    for i in 0..3 { g::fill_HSD_DObj(&mut DO[i]); }
    for i in 0..2 { g::fill_HSD_DObjInfo(&mut DI[i]); }
    for i in 0..2 { g::fill_HSD_CObj(&mut CO[i]); }
    for i in 0..3 { g::fill_HSD_SList(&mut SL[i]); }
    for i in 0..4 { g::fill_HSD_ZList(&mut ZL[i]); }
    for i in 0..2 { g::fill_HSD_ClassInfo(&mut CI[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..4 { g::hash_HSD_JObj(&J[i]); }
    for i in 0..3 { g::hash_HSD_JObjInfo(&JI[i]); }
    for i in 0..3 { g::hash_HSD_DObj(&DO[i]); }
    for i in 0..2 { g::hash_HSD_DObjInfo(&DI[i]); }
    for i in 0..2 { g::hash_HSD_CObj(&CO[i]); }
    for i in 0..3 { g::hash_HSD_SList(&SL[i]); }
    for i in 0..4 { g::hash_HSD_ZList(&ZL[i]); }
    for i in 0..2 { g::hash_HSD_ClassInfo(&CI[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    for i in 0..4 { hm(pi(PPA[i])); }
    g::hashg_all();
    g::hasho_all();
}

// harness = false (Cargo.toml): a case that ends in a panic is cut off inside the hook (see mtu_harness::run_case),
// which a libtest main thread cannot survive at exit.
fn main() { displayfunc_native_matches_c(); println!("tu_displayfunc_native: ok"); }

fn displayfunc_native_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();
preg(&raw const MT, 192); m::use_ext_data = true; freg(cb_pmtx as usize); freg(cb_jdisp as usize); freg(cb_ddisp as usize); freg(cb_sp as usize); melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());
        install_abort_hook();
        let mut c = Check::new(include_str!("data/mtu_displayfunc_native.txt"));
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
{ let mut ord = [0usize; 4];  for q in 0..3usize { JI[q].make_pmtx = Some(cb_pmtx); JI[q].disp = Some(cb_jdisp); }  for q in 0..2usize { DI[q].disp = Some(cb_ddisp); }  for q in 0..3usize { SL[q].next = if q < 2 && fv() & 1 != 0 { &raw mut SL[q + 1] } else { null_mut() }; }  for q in 0..3usize { SL[q].data = (if fv() & 1 != 0 { fv() | 0x8000_0000 } else { fv() }) as usize as *mut c_void; }  for q in 0..3usize { DO[q].next = if q < 2 && fv() & 1 != 0 { &raw mut DO[q + 1] } else { null_mut() }; DO[q].parent.class_info = (&raw mut DI[q & 1]) as *mut HSD_ClassInfo; DO[q].flags = fv() & 0xFF; }  for q in 0..4usize { J[q].object.parent.class_info = (&raw mut JI[q % 3]) as *mut HSD_ClassInfo; J[q].parent = if q > 0 && fv() & 3 != 0 { &raw mut J[(fv() % q as u32) as usize] } else { null_mut() }; J[q].flags = fv() & 0x1FFFFFF; if q < 2 { J[q].flags &= !0x4020u32; }    J[q].envelopemtx = if fv() & 1 != 0 { (&raw mut MT[q]) as *mut [f32; 4] } else { null_mut() };    if J[q].flags & 0x20 != 0 { J[q].u.ptcl = if fv() & 1 != 0 { &raw mut SL[(fv() % 3) as usize] } else { null_mut() }; } else { J[q].u.dobj = if fv() & 1 != 0 { &raw mut DO[(fv() % 3) as usize] } else { null_mut() }; } }  for q in 0..4usize { ZL[q].jobj = &raw mut J[(fv() % 4) as usize]; ZL[q].vmtx = if fv() & 1 != 0 { (&raw mut MT[q]) as *mut [f32; 4] } else { null_mut() }; ZL[q].sort.texedge = null_mut(); ZL[q].sort.xlu = null_mut(); ZL[q].next = null_mut(); }  let _n = fv() % 5; m::zlist_texedge_nb = 0; m::zlist_texedge_top = null_mut(); m::zlist_xlu_nb = 0; m::zlist_xlu_top = null_mut();  for q in 0..4usize { ord[q] = q; } for q in (1..4usize).rev() { let k = (fv() % (q as u32 + 1)) as usize; ord.swap(q, k); }  let k = (fv() % 5) as usize; { let mut pp: *mut *mut m::HSD_ZList = &raw mut m::zlist_texedge_top; for q in 0..k.min(4) { *pp = &raw mut ZL[ord[q]]; pp = &raw mut ZL[ord[q]].sort.texedge; m::zlist_texedge_nb += 1; } }  for q in (1..4usize).rev() { let k = (fv() % (q as u32 + 1)) as usize; ord.swap(q, k); }  let k = (fv() % 5) as usize; { let mut pp: *mut *mut m::HSD_ZList = &raw mut m::zlist_xlu_top; for q in 0..k.min(4) { *pp = &raw mut ZL[ord[q]]; pp = &raw mut ZL[ord[q]].sort.xlu; m::zlist_xlu_nb += 1; } }  m::zlist_top = if fv() & 1 != 0 { &raw mut ZL[0] } else { null_mut() }; for q in 0..3usize { ZL[q].next = if !m::zlist_top.is_null() && fv() & 1 != 0 { &raw mut ZL[q + 1] } else { null_mut() }; }  m::zlist_bottom = if fv() & 1 != 0 { &raw mut m::zlist_top } else { let k = (fv() % 4) as usize; &raw mut ZL[k].next }; m::zlist_texedge_bottom = if fv() & 1 != 0 { &raw mut m::zlist_texedge_top } else { let k = (fv() % 4) as usize; &raw mut ZL[k].sort.texedge }; m::zlist_xlu_bottom = if fv() & 1 != 0 { &raw mut m::zlist_xlu_top } else { let k = (fv() % 4) as usize; &raw mut ZL[k].sort.xlu };  m::sptcl_callback = if fv() & 1 != 0 { Some(cb_sp) } else { None };  for q in 0..4usize { for i in 0..48usize { MT[q][i] = rnd() as u8; } } }
                let mut ret = 0u32;
                let aborted = run_case(|| unsafe { match op {
                    0 => { 

m::HSD_ZListInitAllocData();
 }
                    1 => { 
let a0 = fv() as _;
let a1 = fv() as _;
m::HSD_StateInitDirect(a0, a1);
 }
                    2 => { 
let a0 = pick_HSD_JObj();
let a1 = (&raw mut MT[(rnd() % 4) as usize]) as *mut _;
let a2 = (&raw mut MT[(rnd() % 4) as usize]) as *mut _;
m::HSD_JObjMakePositionMtx(a0, a1, a2);
 }
                    3 => { 
let a0 = pick_HSD_JObj();
ret = pi(m::HSD_JObjFindSkeleton(a0));
 }
                    4 => { 
let a0 = pick_HSD_JObj();
let a1 = (&raw mut MT[(rnd() % 4) as usize]) as *mut _;
ret = pi(m::_HSD_mkEnvelopeModelNodeMtx(a0, a1));
 }
                    5 => { 
let a0 = &raw mut J[(fv() & 1) as usize];
let a1 = (&raw mut MT[(rnd() % 4) as usize]) as *mut _;
let a2 = (&raw mut MT[(rnd() % 4) as usize]) as *mut _;
let a3 = fv() as _;
let a4 = fv() as _;
m::HSD_JObjDispSub(a0, a1, a2, a3, a4);
 }
                    6 => { 
let a0 = pick_HSD_JObj();
let a1 = if fv() & 3 != 0 { (&raw mut MT[(rnd() % 4) as usize]) as *mut _ } else { null_mut() };
let a2 = fv() as _;
let a3 = fv() as _;
m::HSD_JObjDispDObj(a0, a1, a2, a3);
 }
                    7 => { 

m::_HSD_ZListSort();
 }
                    8 => { 

m::_HSD_ZListDisp();
 }
                    9 => { 

m::_HSD_ZListClear();
 }
                    10 => { 
let a0 = pick_HSD_JObj();
let a1 = if fv() & 3 != 0 { (&raw mut MT[(rnd() % 4) as usize]) as *mut _ } else { null_mut() };
let a2 = fv() as _;
let a3 = fv() as _;
m::HSD_JObjDisp(a0, a1, a2, a3);
 }
                    11 => { 
let a0 = Some(autocb_0 as _);
m::HSD_JObjSetSPtclCallback(a0);
 }
                    12 => { 
let a0 = fv() as _;
let a1 = fv() as _;
let a2 = fv() as _;
let a3 = fv() as _;
m::HSD_SetEraseColor(a0, a1, a2, a3);
 }
                    13 => { 
let a0 = rw();
let a1 = rw();
let a2 = rw();
let a3 = rw();
let a4 = rw();
let a5 = fv() as _;
let a6 = fv() as _;
let a7 = fv() as _;
m::HSD_EraseRect(a0, a1, a2, a3, a4, a5, a6, a7);
 }
                    14 => { 
let a0 = (&raw mut BUF) as *mut c_void;
let a1 = (&raw mut BUF) as *mut c_void;
m::_HSD_DispForgetMemory(a0, a1);
 }
                    _ => unreachable!(),
                } });
                if aborted { c.abort(op, k); total += 1; continue; }
                hash_world();
{ for i in 0..3usize { hm(SL[i].data as usize as u32); } }
                c.line(op, k, ret);
                total += 1;
            }
        }
        assert_eq!(c.done(), total);
    }
}
