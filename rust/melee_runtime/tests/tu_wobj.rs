//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/wobj.c.
//! src/tu/wobj.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py wobj`).
//! C functions of the TU this test covers (the private ones through their public callers): HSD_WObjAddAnim, HSD_WObjAlloc, HSD_WObjGetPosition, HSD_WObjInit, HSD_WObjInterpretAnim, HSD_WObjLoadDesc, HSD_WObjRemoveAnim, HSD_WObjReqAnim, HSD_WObjSetDefaultClass, HSD_WObjSetPosition, HSD_WObjSetPositionX, HSD_WObjSetPositionY, HSD_WObjSetPositionZ, WObjAmnesia, WObjInfoInit, WObjLoad, WObjRelease, WObjUpdateFunc.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_HSD_WObjRemoveAnim: u32 = 0;
const OP_HSD_WObjReqAnim: u32 = 1;
const OP_HSD_WObjAddAnim: u32 = 2;
const OP_WObjUpdateFunc: u32 = 3;
const OP_HSD_WObjInterpretAnim: u32 = 4;
const OP_WObjLoad: u32 = 5;
const OP_HSD_WObjInit: u32 = 6;
const OP_HSD_WObjSetDefaultClass: u32 = 7;
const OP_HSD_WObjLoadDesc: u32 = 8;
const OP_HSD_WObjSetPosition: u32 = 9;
const OP_HSD_WObjSetPositionX: u32 = 10;
const OP_HSD_WObjSetPositionY: u32 = 11;
const OP_HSD_WObjSetPositionZ: u32 = 12;
const OP_HSD_WObjGetPosition: u32 = 13;
const OP_HSD_WObjAlloc: u32 = 14;
const OP_WObjRelease: u32 = 15;
const OP_WObjAmnesia: u32 = 16;
const OP_WObjInfoInit: u32 = 17;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::wobj as m;
use mtu_harness::*;
#[path = "mtu/wobj_gen.rs"]
mod g;



pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub static mut WO: [HSD_WObj; 3] = unsafe { core::mem::zeroed() };
pub static mut AO: [HSD_AObj; 3] = unsafe { core::mem::zeroed() };
pub static mut RO: [HSD_RObj; 2] = unsafe { core::mem::zeroed() };
pub static mut JO: [HSD_JObj; 2] = unsafe { core::mem::zeroed() };
pub static mut WI: [HSD_WObjInfo; 2] = unsafe { core::mem::zeroed() };
pub static mut CI: [HSD_ClassInfo; 2] = unsafe { core::mem::zeroed() };
pub static mut SPL: [HSD_Spline; 1] = unsafe { core::mem::zeroed() };
pub static mut OBD: [HSD_ObjData; 2] = unsafe { core::mem::zeroed() };
pub static mut WD: [HSD_WObjDesc; 2] = unsafe { core::mem::zeroed() };
pub static mut WA: [HSD_WObjAnim; 2] = unsafe { core::mem::zeroed() };
pub static mut AD: [HSD_AObjDesc; 2] = unsafe { core::mem::zeroed() };
pub static mut RD: [HSD_RObjDesc; 2] = unsafe { core::mem::zeroed() };
pub static mut RJ: [HSD_RObjAnimJoint; 2] = unsafe { core::mem::zeroed() };
pub fn pick_HSD_WObj() -> *mut HSD_WObj { let r = rnd(); unsafe { &raw mut WO[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_WObj() -> *mut HSD_WObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut WO[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_AObj() -> *mut HSD_AObj { let r = rnd(); unsafe { &raw mut AO[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_AObj() -> *mut HSD_AObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut AO[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_RObj() -> *mut HSD_RObj { let r = rnd(); unsafe { &raw mut RO[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_RObj() -> *mut HSD_RObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut RO[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_JObj() -> *mut HSD_JObj { let r = rnd(); unsafe { &raw mut JO[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_JObj() -> *mut HSD_JObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut JO[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_WObjInfo() -> *mut HSD_WObjInfo { let r = rnd(); unsafe { &raw mut WI[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_WObjInfo() -> *mut HSD_WObjInfo { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut WI[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_ClassInfo() -> *mut HSD_ClassInfo { let r = rnd(); unsafe { &raw mut CI[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_ClassInfo() -> *mut HSD_ClassInfo { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut CI[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_Spline() -> *mut HSD_Spline { let r = rnd(); unsafe { &raw mut SPL[((r >> 8) % 1) as usize] } }
pub fn picko_HSD_Spline() -> *mut HSD_Spline { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut SPL[((r >> 8) % 1) as usize] } } }
pub fn pick_HSD_ObjData() -> *mut HSD_ObjData { let r = rnd(); unsafe { &raw mut OBD[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_ObjData() -> *mut HSD_ObjData { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut OBD[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_WObjDesc() -> *mut HSD_WObjDesc { let r = rnd(); unsafe { &raw mut WD[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_WObjDesc() -> *mut HSD_WObjDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut WD[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_WObjAnim() -> *mut HSD_WObjAnim { let r = rnd(); unsafe { &raw mut WA[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_WObjAnim() -> *mut HSD_WObjAnim { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut WA[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_AObjDesc() -> *mut HSD_AObjDesc { let r = rnd(); unsafe { &raw mut AD[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_AObjDesc() -> *mut HSD_AObjDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut AD[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_RObjDesc() -> *mut HSD_RObjDesc { let r = rnd(); unsafe { &raw mut RD[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_RObjDesc() -> *mut HSD_RObjDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut RD[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_RObjAnimJoint() -> *mut HSD_RObjAnimJoint { let r = rnd(); unsafe { &raw mut RJ[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_RObjAnimJoint() -> *mut HSD_RObjAnimJoint { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut RJ[((r >> 8) % 2) as usize] } } }
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
        preg(&raw const WO, core::mem::size_of_val(&WO));
        preg(&raw const AO, core::mem::size_of_val(&AO));
        preg(&raw const RO, core::mem::size_of_val(&RO));
        preg(&raw const JO, core::mem::size_of_val(&JO));
        preg(&raw const WI, core::mem::size_of_val(&WI));
        preg(&raw const CI, core::mem::size_of_val(&CI));
        preg(&raw const SPL, core::mem::size_of_val(&SPL));
        preg(&raw const OBD, core::mem::size_of_val(&OBD));
        preg(&raw const WD, core::mem::size_of_val(&WD));
        preg(&raw const WA, core::mem::size_of_val(&WA));
        preg(&raw const AD, core::mem::size_of_val(&AD));
        preg(&raw const RD, core::mem::size_of_val(&RD));
        preg(&raw const RJ, core::mem::size_of_val(&RJ));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS));
    }
}

pub unsafe fn setup_world() {
    melee_runtime::tu::wobj::ext::use_ext_data = true;
    WO = core::mem::zeroed();
    AO = core::mem::zeroed();
    RO = core::mem::zeroed();
    JO = core::mem::zeroed();
    WI = core::mem::zeroed();
    CI = core::mem::zeroed();
    SPL = core::mem::zeroed();
    OBD = core::mem::zeroed();
    WD = core::mem::zeroed();
    WA = core::mem::zeroed();
    AD = core::mem::zeroed();
    RD = core::mem::zeroed();
    RJ = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..3 { g::fill_HSD_WObj(&mut WO[i]); }
    for i in 0..3 { g::fill_HSD_AObj(&mut AO[i]); }
    for i in 0..2 { g::fill_HSD_RObj(&mut RO[i]); }
    for i in 0..2 { g::fill_HSD_JObj(&mut JO[i]); }
    for i in 0..2 { g::fill_HSD_WObjInfo(&mut WI[i]); }
    for i in 0..2 { g::fill_HSD_ClassInfo(&mut CI[i]); }
    for i in 0..1 { g::fill_HSD_Spline(&mut SPL[i]); }
    for i in 0..2 { g::fill_HSD_ObjData(&mut OBD[i]); }
    for i in 0..2 { g::fill_HSD_WObjDesc(&mut WD[i]); }
    for i in 0..2 { g::fill_HSD_WObjAnim(&mut WA[i]); }
    for i in 0..2 { g::fill_HSD_AObjDesc(&mut AD[i]); }
    for i in 0..2 { g::fill_HSD_RObjDesc(&mut RD[i]); }
    for i in 0..2 { g::fill_HSD_RObjAnimJoint(&mut RJ[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..3 { g::hash_HSD_WObj(&WO[i]); }
    for i in 0..3 { g::hash_HSD_AObj(&AO[i]); }
    for i in 0..2 { g::hash_HSD_RObj(&RO[i]); }
    for i in 0..2 { g::hash_HSD_JObj(&JO[i]); }
    for i in 0..2 { g::hash_HSD_WObjInfo(&WI[i]); }
    for i in 0..2 { g::hash_HSD_ClassInfo(&CI[i]); }
    for i in 0..1 { g::hash_HSD_Spline(&SPL[i]); }
    for i in 0..2 { g::hash_HSD_ObjData(&OBD[i]); }
    for i in 0..2 { g::hash_HSD_WObjDesc(&WD[i]); }
    for i in 0..2 { g::hash_HSD_WObjAnim(&WA[i]); }
    for i in 0..2 { g::hash_HSD_AObjDesc(&AD[i]); }
    for i in 0..2 { g::hash_HSD_RObjDesc(&RD[i]); }
    for i in 0..2 { g::hash_HSD_RObjAnimJoint(&RJ[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    g::hashg_all();
    g::hasho_all();
}

// harness = false (Cargo.toml): a case that ends in a panic is cut off inside the hook (see mtu_harness::run_case),
// which a libtest main thread cannot survive at exit.
fn main() { wobj_matches_c(); println!("tu_wobj: ok"); }

fn wobj_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();

        install_abort_hook();
        let mut c = Check::new(include_str!("data/mtu_wobj.txt"));
        let mut total = 0u32;
        for op in 0..18u32 {
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
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
{ for q in 0..2usize { WI[q].load = Some(m::WObjLoad); } for q in 0..3usize { WO[q].parent.parent.class_info = (&raw mut WI[q & 1]) as *mut HSD_ClassInfo; WO[q].aobj = if fv() & 3 != 0 { &raw mut AO[(fv() % 3) as usize] } else { null_mut() }; WO[q].robj = if fv() & 3 != 0 { &raw mut RO[(fv() & 1) as usize] } else { null_mut() }; } for q in 0..3usize { AO[q].jobj = (&raw mut JO[q & 1]) as *mut HSD_JObj; } for q in 0..2usize { JO[q].u.spline = if fv() & 3 != 0 { &raw mut SPL[0] } else { null_mut() }; } m::hsdWObj.parent.parent.head.parent = &raw mut CI[0]; CI[0].release = Some(core::mem::transmute::<usize, unsafe extern "C" fn(*mut HSD_Class)>(cbev1 as usize)); CI[0].amnesia = Some(core::mem::transmute::<usize, unsafe extern "C" fn(*mut HSD_ClassInfo)>(cbev1 as usize)); m::default_class = if fv() & 1 != 0 { &raw mut CI[1] } else { null_mut() }; }
                let mut ret = 0u32;
                let aborted = run_case(|| unsafe { match op {
                    0 => { 
let a0 = pick_HSD_WObj();
m::HSD_WObjRemoveAnim(a0);
 }
                    1 => { 
let a0 = pick_HSD_WObj();
let a1 = rw();
m::HSD_WObjReqAnim(a0, a1);
 }
                    2 => { 
let a0 = pick_HSD_WObj();
let a1 = pick_HSD_WObjAnim();
m::HSD_WObjAddAnim(a0, a1);
 }
                    3 => { 
let a0 = picko_HSD_WObj() as *mut c_void;
let a1 = (fv() % 9) as i32;
let a2 = pick_HSD_ObjData();
m::WObjUpdateFunc(a0, a1, a2);
 }
                    4 => { 
let a0 = pick_HSD_WObj();
m::HSD_WObjInterpretAnim(a0);
 }
                    5 => { 
let a0 = pick_HSD_WObj();
let a1 = pick_HSD_WObjDesc();
ret = m::WObjLoad(a0, a1) as u32;
 }
                    6 => { 
let a0 = pick_HSD_WObj();
let a1 = pick_HSD_WObjDesc();
m::HSD_WObjInit(a0, a1);
 }
                    7 => { 
let a0 = pick_HSD_ClassInfo();
m::HSD_WObjSetDefaultClass(a0);
 }
                    8 => { 
let a0 = pick_HSD_WObjDesc();
ret = pi(m::HSD_WObjLoadDesc(a0));
 }
                    9 => { 
let a0 = pick_HSD_WObj();
let a1 = &raw mut VA[(rnd() % 4) as usize];
m::HSD_WObjSetPosition(a0, a1);
 }
                    10 => { 
let a0 = pick_HSD_WObj();
let a1 = rw();
m::HSD_WObjSetPositionX(a0, a1);
 }
                    11 => { 
let a0 = pick_HSD_WObj();
let a1 = rw();
m::HSD_WObjSetPositionY(a0, a1);
 }
                    12 => { 
let a0 = pick_HSD_WObj();
let a1 = rw();
m::HSD_WObjSetPositionZ(a0, a1);
 }
                    13 => { 
let a0 = pick_HSD_WObj();
let a1 = &raw mut VA[(rnd() % 4) as usize];
m::HSD_WObjGetPosition(a0, a1);
 }
                    14 => { 

ret = pi(m::HSD_WObjAlloc());
 }
                    15 => { 
let a0 = pick_HSD_WObj() as *mut HSD_Class;
m::WObjRelease(a0);
 }
                    16 => { 
let a0 = pick_HSD_ClassInfo();
m::WObjAmnesia(a0);
 }
                    17 => { 

m::WObjInfoInit();
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
