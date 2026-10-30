//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/dobj.c.
//! src/tu/dobj.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py dobj`).
//! C functions of the TU this test covers (the private ones through their public callers): DObjAmnesia, DObjInfoInit, DObjLoad, DObjRelease, HSD_DObjAddAnim, HSD_DObjAddAnimAll, HSD_DObjAlloc, HSD_DObjAnim, HSD_DObjAnimAll, HSD_DObjClearFlags, HSD_DObjDisp, HSD_DObjGetFlags, HSD_DObjLoadDesc, HSD_DObjModifyFlags, HSD_DObjRemove, HSD_DObjRemoveAll, HSD_DObjRemoveAnimAllByFlags, HSD_DObjRemoveAnimByFlags, HSD_DObjReqAnimAll, HSD_DObjReqAnimAllByFlags, HSD_DObjReqAnimByFlags, HSD_DObjResolveRefs, HSD_DObjResolveRefsAll, HSD_DObjSetCurrent, HSD_DObjSetDefaultClass, HSD_DObjSetFlags, forceStringAllocation.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_HSD_DObjSetCurrent: u32 = 0;
const OP_HSD_DObjGetFlags: u32 = 1;
const OP_HSD_DObjSetFlags: u32 = 2;
const OP_HSD_DObjClearFlags: u32 = 3;
const OP_HSD_DObjModifyFlags: u32 = 4;
const OP_HSD_DObjRemoveAnimByFlags: u32 = 5;
const OP_HSD_DObjRemoveAnimAllByFlags: u32 = 6;
const OP_HSD_DObjAddAnim: u32 = 7;
const OP_HSD_DObjAddAnimAll: u32 = 8;
const OP_HSD_DObjReqAnimByFlags: u32 = 9;
const OP_HSD_DObjReqAnimAllByFlags: u32 = 10;
const OP_HSD_DObjReqAnimAll: u32 = 11;
const OP_HSD_DObjAnim: u32 = 12;
const OP_HSD_DObjAnimAll: u32 = 13;
const OP_DObjLoad: u32 = 14;
const OP_HSD_DObjLoadDesc: u32 = 15;
const OP_HSD_DObjRemove: u32 = 16;
const OP_HSD_DObjRemoveAll: u32 = 17;
const OP_HSD_DObjSetDefaultClass: u32 = 18;
const OP_HSD_DObjAlloc: u32 = 19;
const OP_HSD_DObjResolveRefs: u32 = 20;
const OP_HSD_DObjResolveRefsAll: u32 = 21;
const OP_forceStringAllocation: u32 = 22;
const OP_HSD_DObjDisp: u32 = 23;
const OP_DObjRelease: u32 = 24;
const OP_DObjAmnesia: u32 = 25;
const OP_DObjInfoInit: u32 = 26;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::dobj::hsd as m;
use mtu_harness::*;
#[path = "mtu/dobj_gen.rs"]
mod g;


unsafe extern "C" fn cb_setup(m: *mut HSD_MObj, r: u32) { t(8200); t(pi(m)); t(r); }
unsafe extern "C" fn cb_unset(m: *mut HSD_MObj, r: u32) { t(8201); t(pi(m)); t(r); }
unsafe extern "C" fn cb_disp(p: *mut HSD_PObj, v: *mut c_void, pm: *mut c_void, r: u32) { t(8202); t(pi(p)); t(pi(v)); t(pi(pm)); t(r); }
unsafe extern "C" fn cb_rel(c: *mut HSD_Class) { t(8203); t(pi(c)); }
unsafe extern "C" fn cb_dest(c: *mut HSD_Class) { t(8204); t(pi(c)); }
unsafe extern "C" fn cb_amn(c: *mut HSD_ClassInfo) { t(8205); t(pi(c)); }


pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub static mut DOB: [HSD_DObj; 3] = unsafe { core::mem::zeroed() };
pub static mut MOB: [HSD_MObj; 2] = unsafe { core::mem::zeroed() };
pub static mut POB: [HSD_PObj; 2] = unsafe { core::mem::zeroed() };
pub static mut AOB: [HSD_AObj; 2] = unsafe { core::mem::zeroed() };
pub static mut MA: [HSD_MatAnim; 2] = unsafe { core::mem::zeroed() };
pub static mut SAD: [HSD_ShapeAnimDObj; 2] = unsafe { core::mem::zeroed() };
pub static mut SHA: [HSD_ShapeAnim; 2] = unsafe { core::mem::zeroed() };
pub static mut DD: [HSD_DObjDesc; 2] = unsafe { core::mem::zeroed() };
pub static mut MD: [HSD_MObjDesc; 2] = unsafe { core::mem::zeroed() };
pub static mut PD: [HSD_PObjDesc; 2] = unsafe { core::mem::zeroed() };
pub static mut CIN: [HSD_ClassInfo; 2] = unsafe { core::mem::zeroed() };
pub static mut DINF: [HSD_DObjInfo; 2] = unsafe { core::mem::zeroed() };
pub static mut MINF: [HSD_MObjInfo; 1] = unsafe { core::mem::zeroed() };
pub static mut PINF: [HSD_PObjInfo; 1] = unsafe { core::mem::zeroed() };
pub fn pick_HSD_DObj() -> *mut HSD_DObj { let r = rnd(); unsafe { &raw mut DOB[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_DObj() -> *mut HSD_DObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut DOB[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_MObj() -> *mut HSD_MObj { let r = rnd(); unsafe { &raw mut MOB[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_MObj() -> *mut HSD_MObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut MOB[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_PObj() -> *mut HSD_PObj { let r = rnd(); unsafe { &raw mut POB[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_PObj() -> *mut HSD_PObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut POB[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_AObj() -> *mut HSD_AObj { let r = rnd(); unsafe { &raw mut AOB[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_AObj() -> *mut HSD_AObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut AOB[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_MatAnim() -> *mut HSD_MatAnim { let r = rnd(); unsafe { &raw mut MA[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_MatAnim() -> *mut HSD_MatAnim { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut MA[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_ShapeAnimDObj() -> *mut HSD_ShapeAnimDObj { let r = rnd(); unsafe { &raw mut SAD[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_ShapeAnimDObj() -> *mut HSD_ShapeAnimDObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut SAD[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_ShapeAnim() -> *mut HSD_ShapeAnim { let r = rnd(); unsafe { &raw mut SHA[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_ShapeAnim() -> *mut HSD_ShapeAnim { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut SHA[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_DObjDesc() -> *mut HSD_DObjDesc { let r = rnd(); unsafe { &raw mut DD[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_DObjDesc() -> *mut HSD_DObjDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut DD[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_MObjDesc() -> *mut HSD_MObjDesc { let r = rnd(); unsafe { &raw mut MD[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_MObjDesc() -> *mut HSD_MObjDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut MD[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_PObjDesc() -> *mut HSD_PObjDesc { let r = rnd(); unsafe { &raw mut PD[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_PObjDesc() -> *mut HSD_PObjDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut PD[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_ClassInfo() -> *mut HSD_ClassInfo { let r = rnd(); unsafe { &raw mut CIN[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_ClassInfo() -> *mut HSD_ClassInfo { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut CIN[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_DObjInfo() -> *mut HSD_DObjInfo { let r = rnd(); unsafe { &raw mut DINF[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_DObjInfo() -> *mut HSD_DObjInfo { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut DINF[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_MObjInfo() -> *mut HSD_MObjInfo { let r = rnd(); unsafe { &raw mut MINF[((r >> 8) % 1) as usize] } }
pub fn picko_HSD_MObjInfo() -> *mut HSD_MObjInfo { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut MINF[((r >> 8) % 1) as usize] } } }
pub fn pick_HSD_PObjInfo() -> *mut HSD_PObjInfo { let r = rnd(); unsafe { &raw mut PINF[((r >> 8) % 1) as usize] } }
pub fn picko_HSD_PObjInfo() -> *mut HSD_PObjInfo { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut PINF[((r >> 8) % 1) as usize] } } }
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
        preg(&raw const DOB, core::mem::size_of_val(&DOB));
        preg(&raw const MOB, core::mem::size_of_val(&MOB));
        preg(&raw const POB, core::mem::size_of_val(&POB));
        preg(&raw const AOB, core::mem::size_of_val(&AOB));
        preg(&raw const MA, core::mem::size_of_val(&MA));
        preg(&raw const SAD, core::mem::size_of_val(&SAD));
        preg(&raw const SHA, core::mem::size_of_val(&SHA));
        preg(&raw const DD, core::mem::size_of_val(&DD));
        preg(&raw const MD, core::mem::size_of_val(&MD));
        preg(&raw const PD, core::mem::size_of_val(&PD));
        preg(&raw const CIN, core::mem::size_of_val(&CIN));
        preg(&raw const DINF, core::mem::size_of_val(&DINF));
        preg(&raw const MINF, core::mem::size_of_val(&MINF));
        preg(&raw const PINF, core::mem::size_of_val(&PINF));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS));
    }
}

pub unsafe fn setup_world() {
    melee_runtime::tu::dobj::hsd::ext::use_ext_data = true;
    DOB = core::mem::zeroed();
    MOB = core::mem::zeroed();
    POB = core::mem::zeroed();
    AOB = core::mem::zeroed();
    MA = core::mem::zeroed();
    SAD = core::mem::zeroed();
    SHA = core::mem::zeroed();
    DD = core::mem::zeroed();
    MD = core::mem::zeroed();
    PD = core::mem::zeroed();
    CIN = core::mem::zeroed();
    DINF = core::mem::zeroed();
    MINF = core::mem::zeroed();
    PINF = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..3 { g::fill_HSD_DObj(&mut DOB[i]); }
    for i in 0..2 { g::fill_HSD_MObj(&mut MOB[i]); }
    for i in 0..2 { g::fill_HSD_PObj(&mut POB[i]); }
    for i in 0..2 { g::fill_HSD_AObj(&mut AOB[i]); }
    for i in 0..2 { g::fill_HSD_MatAnim(&mut MA[i]); }
    for i in 0..2 { g::fill_HSD_ShapeAnimDObj(&mut SAD[i]); }
    for i in 0..2 { g::fill_HSD_ShapeAnim(&mut SHA[i]); }
    for i in 0..2 { g::fill_HSD_DObjDesc(&mut DD[i]); }
    for i in 0..2 { g::fill_HSD_MObjDesc(&mut MD[i]); }
    for i in 0..2 { g::fill_HSD_PObjDesc(&mut PD[i]); }
    for i in 0..2 { g::fill_HSD_ClassInfo(&mut CIN[i]); }
    for i in 0..2 { g::fill_HSD_DObjInfo(&mut DINF[i]); }
    for i in 0..1 { g::fill_HSD_MObjInfo(&mut MINF[i]); }
    for i in 0..1 { g::fill_HSD_PObjInfo(&mut PINF[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..3 { g::hash_HSD_DObj(&DOB[i]); }
    for i in 0..2 { g::hash_HSD_MObj(&MOB[i]); }
    for i in 0..2 { g::hash_HSD_PObj(&POB[i]); }
    for i in 0..2 { g::hash_HSD_AObj(&AOB[i]); }
    for i in 0..2 { g::hash_HSD_MatAnim(&MA[i]); }
    for i in 0..2 { g::hash_HSD_ShapeAnimDObj(&SAD[i]); }
    for i in 0..2 { g::hash_HSD_ShapeAnim(&SHA[i]); }
    for i in 0..2 { g::hash_HSD_DObjDesc(&DD[i]); }
    for i in 0..2 { g::hash_HSD_MObjDesc(&MD[i]); }
    for i in 0..2 { g::hash_HSD_PObjDesc(&PD[i]); }
    for i in 0..2 { g::hash_HSD_ClassInfo(&CIN[i]); }
    for i in 0..2 { g::hash_HSD_DObjInfo(&DINF[i]); }
    for i in 0..1 { g::hash_HSD_MObjInfo(&MINF[i]); }
    for i in 0..1 { g::hash_HSD_PObjInfo(&PINF[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    g::hashg_all();
    g::hasho_all();
}

// harness = false (Cargo.toml): a case that ends in a panic is cut off inside the hook (see mtu_harness::run_case),
// which a libtest main thread cannot survive at exit.
fn main() { dobj_matches_c(); println!("tu_dobj: ok"); }

fn dobj_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();
freg(cb_setup as usize); freg(cb_unset as usize); freg(cb_disp as usize); freg(cb_rel as usize); freg(cb_dest as usize); freg(cb_amn as usize); melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());
        install_abort_hook();
        let mut c = Check::new(include_str!("data/mtu_dobj.txt"));
        let mut total = 0u32;
        for op in 0..27u32 {
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
                25 => 300u32,
                26 => 300u32,
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
{ for q in 0..3usize { DOB[q].next = if q < 2 { &raw mut DOB[q + 1] } else { null_mut() }; DOB[q].mobj = &raw mut MOB[(fv() & 1) as usize]; DOB[q].pobj = if fv() & 7 != 0 { &raw mut POB[0] } else { null_mut() }; DOB[q].aobj = if fv() & 3 != 0 { &raw mut AOB[(fv() & 1) as usize] } else { null_mut() }; DOB[q].parent.class_info = (&raw mut DINF[q & 1]) as *mut HSD_ClassInfo; } POB[0].next = &raw mut POB[1]; POB[1].next = null_mut(); for q in 0..2usize { POB[q].parent.class_info = (&raw mut PINF[0]) as *mut HSD_ClassInfo; MOB[q].parent.class_info = (&raw mut MINF[0]) as *mut HSD_ClassInfo; } for q in 0..2usize { DINF[q].load = Some(m::DObjLoad); DINF[q].parent.release = Some(cb_rel); DINF[q].parent.destroy = Some(cb_dest); } MINF[0].setup = Some(core::mem::transmute::<usize, unsafe extern "C" fn(*mut HSD_MObj, u32)>(cb_setup as usize)); MINF[0].unset = Some(core::mem::transmute::<usize, unsafe extern "C" fn(*mut HSD_MObj, u32)>(cb_unset as usize)); PINF[0].disp = Some(core::mem::transmute::<usize, unsafe extern "C" fn(*mut HSD_PObj, *mut [f32; 4], *mut [f32; 4], u32)>(cb_disp as usize)); DD[0].next = &raw mut DD[1]; DD[1].next = null_mut(); m::hsdDObj.parent.head.parent = &raw mut CIN[0]; CIN[0].release = Some(cb_rel); CIN[0].amnesia = Some(cb_amn); m::default_class = if fv() & 1 != 0 { &raw mut CIN[1] } else { null_mut() }; m::current_dobj = null_mut(); }
                let mut ret = 0u32;
                let aborted = run_case(|| unsafe { match op {
                    0 => { 
let a0 = pick_HSD_DObj();
m::HSD_DObjSetCurrent(a0);
 }
                    1 => { 
let a0 = pick_HSD_DObj();
ret = m::HSD_DObjGetFlags(a0) as u32;
 }
                    2 => { 
let a0 = pick_HSD_DObj();
let a1 = fv() as _;
m::HSD_DObjSetFlags(a0, a1);
 }
                    3 => { 
let a0 = pick_HSD_DObj();
let a1 = fv() as _;
m::HSD_DObjClearFlags(a0, a1);
 }
                    4 => { 
let a0 = pick_HSD_DObj();
let a1 = fv() as _;
let a2 = fv() as _;
m::HSD_DObjModifyFlags(a0, a1, a2);
 }
                    5 => { 
let a0 = pick_HSD_DObj();
let a1 = fv() as _;
m::HSD_DObjRemoveAnimByFlags(a0, a1);
 }
                    6 => { 
let a0 = pick_HSD_DObj();
let a1 = fv() as _;
m::HSD_DObjRemoveAnimAllByFlags(a0, a1);
 }
                    7 => { 
let a0 = pick_HSD_DObj();
let a1 = pick_HSD_MatAnim();
let a2 = pick_HSD_ShapeAnimDObj();
m::HSD_DObjAddAnim(a0, a1, a2);
 }
                    8 => { 
let a0 = pick_HSD_DObj();
let a1 = pick_HSD_MatAnim();
let a2 = pick_HSD_ShapeAnimDObj();
m::HSD_DObjAddAnimAll(a0, a1, a2);
 }
                    9 => { 
let a0 = pick_HSD_DObj();
let a1 = rw();
let a2 = fv() as _;
m::HSD_DObjReqAnimByFlags(a0, a1, a2);
 }
                    10 => { 
let a0 = pick_HSD_DObj();
let a1 = rw();
let a2 = fv() as _;
m::HSD_DObjReqAnimAllByFlags(a0, a1, a2);
 }
                    11 => { 
let a0 = pick_HSD_DObj();
let a1 = rw();
m::HSD_DObjReqAnimAll(a0, a1);
 }
                    12 => { 
let a0 = pick_HSD_DObj();
m::HSD_DObjAnim(a0);
 }
                    13 => { 
let a0 = pick_HSD_DObj();
m::HSD_DObjAnimAll(a0);
 }
                    14 => { 
let a0 = pick_HSD_DObj();
let a1 = pick_HSD_DObjDesc();
ret = m::DObjLoad(a0, a1) as u32;
 }
                    15 => { 
let a0 = pick_HSD_DObjDesc();
ret = pi(m::HSD_DObjLoadDesc(a0));
 }
                    16 => { 
let a0 = pick_HSD_DObj();
m::HSD_DObjRemove(a0);
 }
                    17 => { 
let a0 = pick_HSD_DObj();
m::HSD_DObjRemoveAll(a0);
 }
                    18 => { 
let a0 = pick_HSD_ClassInfo();
m::HSD_DObjSetDefaultClass(a0);
 }
                    19 => { 

ret = pi(m::HSD_DObjAlloc());
 }
                    20 => { 
let a0 = pick_HSD_DObj();
let a1 = pick_HSD_DObjDesc();
m::HSD_DObjResolveRefs(a0, a1);
 }
                    21 => { 
let a0 = pick_HSD_DObj();
let a1 = pick_HSD_DObjDesc();
m::HSD_DObjResolveRefsAll(a0, a1);
 }
                    22 => { 
let a0 = pick_HSD_DObj();
let a1 = pick_HSD_MObj();
m::forceStringAllocation(a0, a1);
 }
                    23 => { 
let a0 = pick_HSD_DObj();
let a1 = (&raw mut BUF) as *mut [f32; 4];
let a2 = (&raw mut BUF as *mut u8).add(64) as *mut [f32; 4];
let a3 = fv() as _;
m::HSD_DObjDisp(a0, a1, a2, a3);
 }
                    24 => { 
let a0 = pick_HSD_DObj() as *mut HSD_Class;
m::DObjRelease(a0);
 }
                    25 => { 
let a0 = pick_HSD_ClassInfo();
m::DObjAmnesia(a0);
 }
                    26 => { 

m::DObjInfoInit();
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
