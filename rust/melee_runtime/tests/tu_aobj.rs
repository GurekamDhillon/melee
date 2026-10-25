//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/aobj.c.
//! src/tu/aobj.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py aobj`).
//! C functions of the TU this test covers (the private ones through their public callers): CObjForeachAnim, DObjForeachAnim, FogForeachAnim, HSD_AObjAlloc, HSD_AObjClearFlags, HSD_AObjFree, HSD_AObjGetAllocData, HSD_AObjGetFlags, HSD_AObjInitAllocData, HSD_AObjInitEndCallBack, HSD_AObjInterpretAnim, HSD_AObjInvokeCallBacks, HSD_AObjLoadDesc, HSD_AObjRemove, HSD_AObjReqAnim, HSD_AObjSetCurrentFrame, HSD_AObjSetEndFrame, HSD_AObjSetFObj, HSD_AObjSetFlags, HSD_AObjSetRate, HSD_AObjSetRewindFrame, HSD_AObjStopAnim, HSD_ForeachAnim, JObjForeachAnim, LObjForeachAnim, MObjForeachAnim, PObjForeachAnim, RObjForeachAnim, TObjForeachAnim, WObjForeachAnim, _HSD_AObjForgetMemory, callbackForeachFunc.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_HSD_AObjInitAllocData: u32 = 0;
const OP_HSD_AObjGetAllocData: u32 = 1;
const OP_HSD_AObjGetFlags: u32 = 2;
const OP_HSD_AObjSetFlags: u32 = 3;
const OP_HSD_AObjClearFlags: u32 = 4;
const OP_HSD_AObjSetFObj: u32 = 5;
const OP_HSD_AObjInitEndCallBack: u32 = 6;
const OP_HSD_AObjInvokeCallBacks: u32 = 7;
const OP_HSD_AObjReqAnim: u32 = 8;
const OP_HSD_AObjStopAnim: u32 = 9;
const OP_HSD_AObjInterpretAnim: u32 = 10;
const OP_HSD_AObjLoadDesc: u32 = 11;
const OP_HSD_AObjRemove: u32 = 12;
const OP_HSD_AObjAlloc: u32 = 13;
const OP_HSD_AObjFree: u32 = 14;
const OP_HSD_ForeachAnim: u32 = 15;
const OP_HSD_AObjSetRate: u32 = 16;
const OP_HSD_AObjSetRewindFrame: u32 = 17;
const OP_HSD_AObjSetEndFrame: u32 = 18;
const OP_HSD_AObjSetCurrentFrame: u32 = 19;
const OP__HSD_AObjForgetMemory: u32 = 20;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::aobj as m;
use mtu_harness::*;
#[path = "mtu/aobj_gen.rs"]
mod g;


static mut AOI: usize = 0;
unsafe extern "C" fn cbv0() { t(8100); }
unsafe extern "C" fn cbv1() { t(8101); }
unsafe extern "C" fn cb_A(a: *mut HSD_AObj) { t(8000); t(pi(a)); }
unsafe extern "C" fn cb_AF(a: *mut HSD_AObj, f: f32) { t(8001); t(pi(a)); t(f.to_bits()); }
unsafe extern "C" fn cb_AV(a: *mut HSD_AObj, v: *mut c_void) { t(8002); t(pi(a)); t(pi(v)); }
unsafe extern "C" fn cb_AU(a: *mut HSD_AObj, d: u32) { t(8003); t(pi(a)); t(d); }
unsafe extern "C" fn cb_AO(a: *mut HSD_AObj, o: *mut c_void) { t(8004); t(pi(a)); t(pi(o)); }
unsafe extern "C" fn cb_AOF(a: *mut HSD_AObj, o: *mut c_void, f: f32) { t(8005); t(pi(a)); t(pi(o)); t(f.to_bits()); }
unsafe extern "C" fn cb_AOV(a: *mut HSD_AObj, o: *mut c_void, v: *mut c_void) { t(8006); t(pi(a)); t(pi(o)); t(pi(v)); }
unsafe extern "C" fn cb_AOU(a: *mut HSD_AObj, o: *mut c_void, d: u32) { t(8007); t(pi(a)); t(pi(o)); t(d); }
unsafe extern "C" fn cb_AOT(a: *mut HSD_AObj, o: *mut c_void, ty: i32) { t(8008); t(pi(a)); t(pi(o)); t(ty as u32); }
unsafe extern "C" fn cb_AOTF(a: *mut HSD_AObj, o: *mut c_void, ty: i32, f: f32) { t(8009); t(pi(a)); t(pi(o)); t(ty as u32); t(f.to_bits()); }
unsafe extern "C" fn cb_AOTV(a: *mut HSD_AObj, o: *mut c_void, ty: i32, v: *mut c_void) { t(8010); t(pi(a)); t(pi(o)); t(ty as u32); t(pi(v)); }
unsafe extern "C" fn cb_AOTU(a: *mut HSD_AObj, o: *mut c_void, ty: i32, d: u32) { t(8011); t(pi(a)); t(pi(o)); t(ty as u32); t(d); }
fn fnat(i: usize) -> usize { [cb_A as usize, cb_AF as usize, cb_AV as usize, cb_AU as usize, cb_AO as usize, cb_AOF as usize, cb_AOV as usize, cb_AOU as usize, cb_AOT as usize, cb_AOTF as usize, cb_AOTV as usize, cb_AOTU as usize][i] }
const TY: [i32; 10] = [6, 3, 8, 9, 11, 7, 2, 10, 12, 5];
unsafe fn objfor(ty: i32) -> *mut c_void {
    if fv() & 7 == 0 { return null_mut(); }
    match ty { 6 => &raw mut JO[0] as *mut c_void, 3 => &raw mut DO[0] as *mut c_void, 8 => &raw mut MO[0] as *mut c_void, 9 => &raw mut PO[0] as *mut c_void,
        11 => &raw mut TO[0] as *mut c_void, 7 => &raw mut LO[0] as *mut c_void, 2 => &raw mut CO[0] as *mut c_void, 10 => &raw mut RO[0] as *mut c_void,
        12 => &raw mut WO[0] as *mut c_void, _ => &raw mut FG[0] as *mut c_void }
}


pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub static mut AO: [HSD_AObj; 4] = unsafe { core::mem::zeroed() };
pub static mut FO: [HSD_FObj; 2] = unsafe { core::mem::zeroed() };
pub static mut AD: [HSD_AObjDesc; 2] = unsafe { core::mem::zeroed() };
pub static mut JO: [HSD_JObj; 3] = unsafe { core::mem::zeroed() };
pub static mut DO: [HSD_DObj; 3] = unsafe { core::mem::zeroed() };
pub static mut MO: [HSD_MObj; 2] = unsafe { core::mem::zeroed() };
pub static mut TO: [HSD_TObj; 3] = unsafe { core::mem::zeroed() };
pub static mut PO: [HSD_PObj; 2] = unsafe { core::mem::zeroed() };
pub static mut RO: [HSD_RObj; 2] = unsafe { core::mem::zeroed() };
pub static mut WO: [HSD_WObj; 2] = unsafe { core::mem::zeroed() };
pub static mut CO: [HSD_CObj; 2] = unsafe { core::mem::zeroed() };
pub static mut LO: [HSD_LObj; 3] = unsafe { core::mem::zeroed() };
pub static mut FG: [HSD_Fog; 2] = unsafe { core::mem::zeroed() };
pub static mut SS: [HSD_ShapeSet; 2] = unsafe { core::mem::zeroed() };
pub static mut SL: [HSD_SList; 3] = unsafe { core::mem::zeroed() };
pub static mut FD: [HSD_FObjDesc; 2] = unsafe { core::mem::zeroed() };
pub fn pick_HSD_AObj() -> *mut HSD_AObj { let r = rnd(); unsafe { &raw mut AO[((r >> 8) % 4) as usize] } }
pub fn picko_HSD_AObj() -> *mut HSD_AObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut AO[((r >> 8) % 4) as usize] } } }
pub fn pick_HSD_FObj() -> *mut HSD_FObj { let r = rnd(); unsafe { &raw mut FO[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_FObj() -> *mut HSD_FObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut FO[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_AObjDesc() -> *mut HSD_AObjDesc { let r = rnd(); unsafe { &raw mut AD[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_AObjDesc() -> *mut HSD_AObjDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut AD[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_JObj() -> *mut HSD_JObj { let r = rnd(); unsafe { &raw mut JO[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_JObj() -> *mut HSD_JObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut JO[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_DObj() -> *mut HSD_DObj { let r = rnd(); unsafe { &raw mut DO[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_DObj() -> *mut HSD_DObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut DO[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_MObj() -> *mut HSD_MObj { let r = rnd(); unsafe { &raw mut MO[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_MObj() -> *mut HSD_MObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut MO[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_TObj() -> *mut HSD_TObj { let r = rnd(); unsafe { &raw mut TO[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_TObj() -> *mut HSD_TObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut TO[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_PObj() -> *mut HSD_PObj { let r = rnd(); unsafe { &raw mut PO[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_PObj() -> *mut HSD_PObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut PO[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_RObj() -> *mut HSD_RObj { let r = rnd(); unsafe { &raw mut RO[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_RObj() -> *mut HSD_RObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut RO[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_WObj() -> *mut HSD_WObj { let r = rnd(); unsafe { &raw mut WO[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_WObj() -> *mut HSD_WObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut WO[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_CObj() -> *mut HSD_CObj { let r = rnd(); unsafe { &raw mut CO[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_CObj() -> *mut HSD_CObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut CO[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_LObj() -> *mut HSD_LObj { let r = rnd(); unsafe { &raw mut LO[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_LObj() -> *mut HSD_LObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut LO[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_Fog() -> *mut HSD_Fog { let r = rnd(); unsafe { &raw mut FG[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_Fog() -> *mut HSD_Fog { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut FG[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_ShapeSet() -> *mut HSD_ShapeSet { let r = rnd(); unsafe { &raw mut SS[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_ShapeSet() -> *mut HSD_ShapeSet { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut SS[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_SList() -> *mut HSD_SList { let r = rnd(); unsafe { &raw mut SL[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_SList() -> *mut HSD_SList { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut SL[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_FObjDesc() -> *mut HSD_FObjDesc { let r = rnd(); unsafe { &raw mut FD[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_FObjDesc() -> *mut HSD_FObjDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut FD[((r >> 8) % 2) as usize] } } }
pub unsafe extern "C" fn cbev1(g: *mut HSD_GObj) { t(7001); t(pi(g)); }
pub unsafe extern "C" fn cbev2(g: *mut HSD_GObj) { t(7002); t(pi(g)); }
pub unsafe extern "C" fn cbp1(g: *mut HSD_GObj) -> i32 { t(7003); t(pi(g)); (fv() & 1) as i32 }
pub unsafe extern "C" fn cbp2(g: *mut HSD_GObj) -> i32 { t(7004); t(pi(g)); (fv() & 1) as i32 }
pub unsafe extern "C" fn cbev3(g: *mut HSD_GObj, h: *mut HSD_GObj) { t(7005); t(pi(g)); t(pi(h)); }
pub unsafe extern "C" fn cbi1(a: i32) { t(7006); t(a as u32); }
pub unsafe extern "C" fn cbi2(a: i32) { t(7007); t(a as u32); }
pub unsafe extern "C" fn autocb_0(a0: *mut c_void, a1: i32, a2: *mut HSD_ObjData) { t(9000); t(pi(a0)); t(a1 as u32); t(pi(a2)); }
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
        preg(&raw const AO, core::mem::size_of_val(&AO));
        preg(&raw const FO, core::mem::size_of_val(&FO));
        preg(&raw const AD, core::mem::size_of_val(&AD));
        preg(&raw const JO, core::mem::size_of_val(&JO));
        preg(&raw const DO, core::mem::size_of_val(&DO));
        preg(&raw const MO, core::mem::size_of_val(&MO));
        preg(&raw const TO, core::mem::size_of_val(&TO));
        preg(&raw const PO, core::mem::size_of_val(&PO));
        preg(&raw const RO, core::mem::size_of_val(&RO));
        preg(&raw const WO, core::mem::size_of_val(&WO));
        preg(&raw const CO, core::mem::size_of_val(&CO));
        preg(&raw const LO, core::mem::size_of_val(&LO));
        preg(&raw const FG, core::mem::size_of_val(&FG));
        preg(&raw const SS, core::mem::size_of_val(&SS));
        preg(&raw const SL, core::mem::size_of_val(&SL));
        preg(&raw const FD, core::mem::size_of_val(&FD));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS));
    }
}

pub unsafe fn setup_world() {
    AO = core::mem::zeroed();
    FO = core::mem::zeroed();
    AD = core::mem::zeroed();
    JO = core::mem::zeroed();
    DO = core::mem::zeroed();
    MO = core::mem::zeroed();
    TO = core::mem::zeroed();
    PO = core::mem::zeroed();
    RO = core::mem::zeroed();
    WO = core::mem::zeroed();
    CO = core::mem::zeroed();
    LO = core::mem::zeroed();
    FG = core::mem::zeroed();
    SS = core::mem::zeroed();
    SL = core::mem::zeroed();
    FD = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..4 { g::fill_HSD_AObj(&mut AO[i]); }
    for i in 0..2 { g::fill_HSD_FObj(&mut FO[i]); }
    for i in 0..2 { g::fill_HSD_AObjDesc(&mut AD[i]); }
    for i in 0..3 { g::fill_HSD_JObj(&mut JO[i]); }
    for i in 0..3 { g::fill_HSD_DObj(&mut DO[i]); }
    for i in 0..2 { g::fill_HSD_MObj(&mut MO[i]); }
    for i in 0..3 { g::fill_HSD_TObj(&mut TO[i]); }
    for i in 0..2 { g::fill_HSD_PObj(&mut PO[i]); }
    for i in 0..2 { g::fill_HSD_RObj(&mut RO[i]); }
    for i in 0..2 { g::fill_HSD_WObj(&mut WO[i]); }
    for i in 0..2 { g::fill_HSD_CObj(&mut CO[i]); }
    for i in 0..3 { g::fill_HSD_LObj(&mut LO[i]); }
    for i in 0..2 { g::fill_HSD_Fog(&mut FG[i]); }
    for i in 0..2 { g::fill_HSD_ShapeSet(&mut SS[i]); }
    for i in 0..3 { g::fill_HSD_SList(&mut SL[i]); }
    for i in 0..2 { g::fill_HSD_FObjDesc(&mut FD[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..4 { g::hash_HSD_AObj(&AO[i]); }
    for i in 0..2 { g::hash_HSD_FObj(&FO[i]); }
    for i in 0..2 { g::hash_HSD_AObjDesc(&AD[i]); }
    for i in 0..3 { g::hash_HSD_JObj(&JO[i]); }
    for i in 0..3 { g::hash_HSD_DObj(&DO[i]); }
    for i in 0..2 { g::hash_HSD_MObj(&MO[i]); }
    for i in 0..3 { g::hash_HSD_TObj(&TO[i]); }
    for i in 0..2 { g::hash_HSD_PObj(&PO[i]); }
    for i in 0..2 { g::hash_HSD_RObj(&RO[i]); }
    for i in 0..2 { g::hash_HSD_WObj(&WO[i]); }
    for i in 0..2 { g::hash_HSD_CObj(&CO[i]); }
    for i in 0..3 { g::hash_HSD_LObj(&LO[i]); }
    for i in 0..2 { g::hash_HSD_Fog(&FG[i]); }
    for i in 0..2 { g::hash_HSD_ShapeSet(&SS[i]); }
    for i in 0..3 { g::hash_HSD_SList(&SL[i]); }
    for i in 0..2 { g::hash_HSD_FObjDesc(&FD[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    g::hashg_all();
    g::hasho_all();
}

// harness = false (Cargo.toml): a case that ends in a panic is cut off inside the hook (see mtu_harness::run_case),
// which a libtest main thread cannot survive at exit.
fn main() { aobj_matches_c(); println!("tu_aobj: ok"); }

fn aobj_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();
freg(cbv0 as usize); freg(cbv1 as usize); for i in 0..12 { freg(fnat(i)); } melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());
        install_abort_hook();
        let mut c = Check::new(include_str!("data/mtu_aobj.txt"));
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
{ AOI = 0; for q in 0..3usize { JO[q].aobj = if fv() & 3 != 0 { &raw mut AO[(fv() % 4) as usize] } else { null_mut() }; } for q in 0..3usize { DO[q].aobj = if fv() & 3 != 0 { &raw mut AO[(fv() % 4) as usize] } else { null_mut() }; } for q in 0..2usize { MO[q].aobj = if fv() & 3 != 0 { &raw mut AO[(fv() % 4) as usize] } else { null_mut() }; } for q in 0..3usize { TO[q].aobj = if fv() & 3 != 0 { &raw mut AO[(fv() % 4) as usize] } else { null_mut() }; } for q in 0..2usize { RO[q].aobj = if fv() & 3 != 0 { &raw mut AO[(fv() % 4) as usize] } else { null_mut() }; } for q in 0..2usize { WO[q].aobj = if fv() & 3 != 0 { &raw mut AO[(fv() % 4) as usize] } else { null_mut() }; } for q in 0..2usize { CO[q].aobj = if fv() & 3 != 0 { &raw mut AO[(fv() % 4) as usize] } else { null_mut() }; } for q in 0..3usize { LO[q].aobj = if fv() & 3 != 0 { &raw mut AO[(fv() % 4) as usize] } else { null_mut() }; } for q in 0..2usize { FG[q].aobj = if fv() & 3 != 0 { &raw mut AO[(fv() % 4) as usize] } else { null_mut() }; } JO[0].child = &raw mut JO[1]; JO[0].next = null_mut(); JO[1].child = null_mut(); JO[1].next = &raw mut JO[2]; JO[2].child = null_mut(); JO[2].next = null_mut(); for q in 0..3usize { JO[q].u.dobj = if fv() & 3 != 0 { &raw mut DO[(fv() % 3) as usize] } else { null_mut() }; JO[q].robj = if fv() & 3 != 0 { &raw mut RO[(fv() & 1) as usize] } else { null_mut() }; } for q in 0..3usize { DO[q].next = if q < 2 { &raw mut DO[q + 1] } else { null_mut() }; DO[q].mobj = if fv() & 3 != 0 { &raw mut MO[(fv() & 1) as usize] } else { null_mut() }; DO[q].pobj = if fv() & 3 != 0 { &raw mut PO[0] } else { null_mut() }; } for q in 0..2usize { MO[q].tobj = if fv() & 3 != 0 { &raw mut TO[0] } else { null_mut() }; } for q in 0..3usize { TO[q].next = if q < 2 { &raw mut TO[q + 1] } else { null_mut() }; } PO[0].next = &raw mut PO[1]; PO[1].next = null_mut(); for q in 0..2usize { PO[q].u.shape_set = if fv() & 3 != 0 { &raw mut SS[(fv() & 1) as usize] } else { null_mut() }; } RO[0].next = &raw mut RO[1]; RO[1].next = null_mut(); for q in 0..2usize { WO[q].robj = if fv() & 3 != 0 { &raw mut RO[0] } else { null_mut() }; } for q in 0..3usize { LO[q].next = if q < 2 { &raw mut LO[q + 1] } else { null_mut() }; } SL[0].next = if fv() & 1 != 0 { &raw mut SL[1] } else { null_mut() }; SL[1].next = null_mut(); SL[2].next = null_mut(); SL[0].data = cbv0 as usize as *mut c_void; SL[1].data = cbv1 as usize as *mut c_void; m::endcallback_list = if fv() & 7 != 0 { &raw mut SL[0] } else { null_mut() }; m::HSD_AObj_804D762C = (fv() & 1) as i32; m::HSD_AObj_804D7630 = (fv() % 3 == 0) as i32; }
                let mut ret = 0u32;
                let aborted = run_case(|| unsafe { match op {
                    0 => { 

m::HSD_AObjInitAllocData();
 }
                    1 => { 

ret = pi(m::HSD_AObjGetAllocData());
 }
                    2 => { 
let a0 = pick_HSD_AObj();
ret = m::HSD_AObjGetFlags(a0) as u32;
 }
                    3 => { 
let a0 = pick_HSD_AObj();
let a1 = fv() as _;
m::HSD_AObjSetFlags(a0, a1);
 }
                    4 => { 
let a0 = pick_HSD_AObj();
let a1 = fv() as _;
m::HSD_AObjClearFlags(a0, a1);
 }
                    5 => { 
let a0 = pick_HSD_AObj();
let a1 = pick_HSD_FObj();
m::HSD_AObjSetFObj(a0, a1);
 }
                    6 => { 

m::HSD_AObjInitEndCallBack();
 }
                    7 => { 

m::HSD_AObjInvokeCallBacks();
 }
                    8 => { 
let a0 = pick_HSD_AObj();
let a1 = rw();
m::HSD_AObjReqAnim(a0, a1);
 }
                    9 => { 
let a0 = pick_HSD_AObj();
let a1 = (&raw mut BUF) as *mut c_void;
let a2 = Some(autocb_0 as _);
m::HSD_AObjStopAnim(a0, a1, a2);
 }
                    10 => { 
let a0 = pick_HSD_AObj();
let a1 = (&raw mut BUF) as *mut c_void;
let a2 = Some(autocb_0 as _);
m::HSD_AObjInterpretAnim(a0, a1, a2);
 }
                    11 => { 
let a0 = pick_HSD_AObjDesc();
ret = pi(m::HSD_AObjLoadDesc(a0));
 }
                    12 => { 
let a0 = pick_HSD_AObj();
m::HSD_AObjRemove(a0);
 }
                    13 => { 

ret = pi(m::HSD_AObjAlloc());
 }
                    14 => { 
let a0 = pick_HSD_AObj();
m::HSD_AObjFree(a0);
 }
                    15 => { 
let a0 = null_mut::<c_void>();
let a1 = 0i32;
let a2 = 0u32;
let a3 = null_mut::<c_void>();
let a4 = 0i32;
{ let ty = TY[(fv() % 10) as usize]; let at = (fv() % 14) as i32; let ob = objfor(ty); let mk = if fv() & 1 != 0 { 0x7FFFu32 } else { fv() & 0x7FFF }; let f = fnat(if at > 11 { 0 } else { at as usize }) as *mut c_void; let arg = match at { 1 | 5 | 9 => m::CallbackArg { f: (rw() as f64) as f32 }, 2 | 6 | 10 => m::CallbackArg { v: (&raw mut BUF) as *mut c_void }, 3 | 7 | 11 => m::CallbackArg { d: fv() }, _ => m::CallbackArg { d: 0 } }; m::HSD_ForeachAnim(ob, ty as _, mk as _, f, at as _, arg); }
 }
                    16 => { 
let a0 = pick_HSD_AObj();
let a1 = rw();
m::HSD_AObjSetRate(a0, a1);
 }
                    17 => { 
let a0 = pick_HSD_AObj();
let a1 = rw();
m::HSD_AObjSetRewindFrame(a0, a1);
 }
                    18 => { 
let a0 = pick_HSD_AObj();
let a1 = rw();
m::HSD_AObjSetEndFrame(a0, a1);
 }
                    19 => { 
let a0 = pick_HSD_AObj();
let a1 = rw();
m::HSD_AObjSetCurrentFrame(a0, a1);
 }
                    20 => { 
let a0 = (&raw mut BUF) as *mut c_void;
let a1 = (&raw mut BUF) as *mut c_void;
m::_HSD_AObjForgetMemory(a0, a1);
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
