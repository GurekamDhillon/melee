//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/cobj.c.
//! src/tu/cobj.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py cobj`).
//! C functions of the TU this test covers (the private ones through their public callers): CObjAmnesia, CObjInfoInit, CObjInit, CObjLoad, CObjRelease, CObjResetFlags, CObjUpdateFunc, HSD_CObjAddAnim, HSD_CObjAlloc, HSD_CObjAnim, HSD_CObjClearFlags, HSD_CObjEndCurrent, HSD_CObjEraseScreen, HSD_CObjGetAspect, HSD_CObjGetAuthoredAspect, HSD_CObjGetBottom, HSD_CObjGetCurrent, HSD_CObjGetEyeDistance, HSD_CObjGetEyePosition, HSD_CObjGetEyePositionWObj, HSD_CObjGetEyeVector, HSD_CObjGetFar, HSD_CObjGetFlags, HSD_CObjGetFov, HSD_CObjGetInterest, HSD_CObjGetInterestWObj, HSD_CObjGetInvViewingMtxPtr, HSD_CObjGetInvViewingMtxPtrDirect, HSD_CObjGetLeft, HSD_CObjGetLeftVector, HSD_CObjGetNear, HSD_CObjGetOrtho, HSD_CObjGetPerspective, HSD_CObjGetProjectionType, HSD_CObjGetRight, HSD_CObjGetScissor, HSD_CObjGetTop, HSD_CObjGetUpVector, HSD_CObjGetViewingMtx, HSD_CObjGetViewingMtxPtr, HSD_CObjGetViewportf, HSD_CObjInit, HSD_CObjLoadDesc, HSD_CObjMtxIsDirty, HSD_CObjRemoveAnim, HSD_CObjRemoveAnimByFlags, HSD_CObjReqAnim, HSD_CObjSetAspect, HSD_CObjSetBottom, HSD_CObjSetCurrent, HSD_CObjSetDefaultClass, HSD_CObjSetEyePosition, HSD_CObjSetEyePositionWObj, HSD_CObjSetFar, HSD_CObjSetFlags, HSD_CObjSetFov, HSD_CObjSetFrustum, HSD_CObjSetInterest, HSD_CObjSetInterestWObj, HSD_CObjSetLeft, HSD_CObjSetMtxDirty, HSD_CObjSetNear, HSD_CObjSetOrtho, HSD_CObjSetPerspective, HSD_CObjSetProjectionType, HSD_CObjSetRight, HSD_CObjSetRoll, HSD_CObjSetScissor, HSD_CObjSetScissorx4, HSD_CObjSetTop, HSD_CObjSetUpVector, HSD_CObjSetViewport, HSD_CObjSetViewportf, HSD_CObjSetViewportfx4, HSD_CObjSetupViewingMtx, HSD_CObjUpdateView, cobj_get_up_x, get_up_vector_for_viewing_mtx, get_up_vector_for_viewing_mtx_inner, makeProjectionMtx, roll2upvec, setupBottomHalfCamera, setupNormalCamera, setupOffscreenCamera, setupTopHalfCamera, upvec2roll, vec_get_abs_y, vec_get_x.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_HSD_CObjEraseScreen: u32 = 0;
const OP_HSD_CObjRemoveAnimByFlags: u32 = 1;
const OP_HSD_CObjRemoveAnim: u32 = 2;
const OP_HSD_CObjAddAnim: u32 = 3;
const OP_CObjUpdateFunc: u32 = 4;
const OP_HSD_CObjAnim: u32 = 5;
const OP_HSD_CObjReqAnim: u32 = 6;
const OP_makeProjectionMtx: u32 = 7;
const OP_HSD_CObjSetupViewingMtx: u32 = 8;
const OP_HSD_CObjUpdateView: u32 = 9;
const OP_HSD_CObjSetCurrent: u32 = 10;
const OP_HSD_CObjEndCurrent: u32 = 11;
const OP_HSD_CObjGetInterestWObj: u32 = 12;
const OP_HSD_CObjGetEyePositionWObj: u32 = 13;
const OP_HSD_CObjSetMtxDirty: u32 = 14;
const OP_HSD_CObjMtxIsDirty: u32 = 15;
const OP_HSD_CObjSetFov: u32 = 16;
const OP_HSD_CObjGetNear: u32 = 17;
const OP_HSD_CObjSetNear: u32 = 18;
const OP_HSD_CObjGetFar: u32 = 19;
const OP_HSD_CObjSetFar: u32 = 20;
const OP_HSD_CObjSetScissor: u32 = 21;
const OP_HSD_CObjSetViewport: u32 = 22;
const OP_HSD_CObjGetProjectionType: u32 = 23;
const OP_HSD_CObjSetPerspective: u32 = 24;
const OP_HSD_CObjSetFrustum: u32 = 25;
const OP_HSD_CObjSetOrtho: u32 = 26;
const OP_HSD_CObjSetFlags: u32 = 27;
const OP_HSD_CObjClearFlags: u32 = 28;
const OP_HSD_CObjGetCurrent: u32 = 29;
const OP_HSD_CObjSetDefaultClass: u32 = 30;
const OP_HSD_CObjAlloc: u32 = 31;
const OP_CObjLoad: u32 = 32;
const OP_HSD_CObjInit: u32 = 33;
const OP_HSD_CObjLoadDesc: u32 = 34;
const OP_CObjInit: u32 = 35;
const OP_CObjRelease: u32 = 36;
const OP_CObjAmnesia: u32 = 37;
const OP_CObjInfoInit: u32 = 38;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::cobj as m;
use mtu_harness::*;
#[path = "mtu/cobj_gen.rs"]
mod g;


unsafe extern "C" fn cb_rel(c: *mut HSD_Class) { t(8203); t(pi(c)); }
unsafe extern "C" fn cb_dest(c: *mut HSD_Class) { t(8204); t(pi(c)); }
unsafe extern "C" fn cb_amn(c: *mut HSD_ClassInfo) { t(8205); t(pi(c)); }
unsafe extern "C" fn cb_init(c: *mut HSD_Class) -> i32 { t(8206); t(pi(c)); (fv() % 3) as i32 - 1 }
static mut MT: [[u8; 64]; 2] = [[0; 64]; 2];


pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut PPA: [*mut c_void; 4] = [core::ptr::null_mut(); 4];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub static mut COB: [HSD_CObj; 3] = unsafe { core::mem::zeroed() };
pub static mut WOB: [HSD_WObj; 4] = unsafe { core::mem::zeroed() };
pub static mut AOB: [HSD_AObj; 2] = unsafe { core::mem::zeroed() };
pub static mut CDS: [HSD_CObjDesc; 2] = unsafe { core::mem::zeroed() };
pub static mut CINF: [HSD_CObjInfo; 2] = unsafe { core::mem::zeroed() };
pub static mut CIN: [HSD_ClassInfo; 2] = unsafe { core::mem::zeroed() };
pub static mut CAN: [HSD_CameraAnim; 2] = unsafe { core::mem::zeroed() };
pub static mut WDS: [HSD_WObjDesc; 2] = unsafe { core::mem::zeroed() };
pub static mut WAN: [HSD_WObjAnim; 2] = unsafe { core::mem::zeroed() };
pub static mut ADS: [HSD_AObjDesc; 2] = unsafe { core::mem::zeroed() };
pub static mut RMO: [GXRenderModeObj; 1] = unsafe { core::mem::zeroed() };
pub static mut OBD: [HSD_ObjData; 2] = unsafe { core::mem::zeroed() };
pub static mut SCS: [Scissor; 1] = unsafe { core::mem::zeroed() };
pub static mut RS16: [HSD_RectS16; 1] = unsafe { core::mem::zeroed() };
pub fn pick_HSD_CObj() -> *mut HSD_CObj { let r = rnd(); unsafe { &raw mut COB[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_CObj() -> *mut HSD_CObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut COB[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_WObj() -> *mut HSD_WObj { let r = rnd(); unsafe { &raw mut WOB[((r >> 8) % 4) as usize] } }
pub fn picko_HSD_WObj() -> *mut HSD_WObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut WOB[((r >> 8) % 4) as usize] } } }
pub fn pick_HSD_AObj() -> *mut HSD_AObj { let r = rnd(); unsafe { &raw mut AOB[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_AObj() -> *mut HSD_AObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut AOB[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_CObjDesc() -> *mut HSD_CObjDesc { let r = rnd(); unsafe { &raw mut CDS[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_CObjDesc() -> *mut HSD_CObjDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut CDS[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_CObjInfo() -> *mut HSD_CObjInfo { let r = rnd(); unsafe { &raw mut CINF[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_CObjInfo() -> *mut HSD_CObjInfo { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut CINF[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_ClassInfo() -> *mut HSD_ClassInfo { let r = rnd(); unsafe { &raw mut CIN[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_ClassInfo() -> *mut HSD_ClassInfo { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut CIN[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_CameraAnim() -> *mut HSD_CameraAnim { let r = rnd(); unsafe { &raw mut CAN[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_CameraAnim() -> *mut HSD_CameraAnim { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut CAN[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_WObjDesc() -> *mut HSD_WObjDesc { let r = rnd(); unsafe { &raw mut WDS[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_WObjDesc() -> *mut HSD_WObjDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut WDS[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_WObjAnim() -> *mut HSD_WObjAnim { let r = rnd(); unsafe { &raw mut WAN[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_WObjAnim() -> *mut HSD_WObjAnim { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut WAN[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_AObjDesc() -> *mut HSD_AObjDesc { let r = rnd(); unsafe { &raw mut ADS[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_AObjDesc() -> *mut HSD_AObjDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut ADS[((r >> 8) % 2) as usize] } } }
pub fn pick_GXRenderModeObj() -> *mut GXRenderModeObj { let r = rnd(); unsafe { &raw mut RMO[((r >> 8) % 1) as usize] } }
pub fn picko_GXRenderModeObj() -> *mut GXRenderModeObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut RMO[((r >> 8) % 1) as usize] } } }
pub fn pick_HSD_ObjData() -> *mut HSD_ObjData { let r = rnd(); unsafe { &raw mut OBD[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_ObjData() -> *mut HSD_ObjData { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut OBD[((r >> 8) % 2) as usize] } } }
pub fn pick_Scissor() -> *mut Scissor { let r = rnd(); unsafe { &raw mut SCS[((r >> 8) % 1) as usize] } }
pub fn picko_Scissor() -> *mut Scissor { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut SCS[((r >> 8) % 1) as usize] } } }
pub fn pick_HSD_RectS16() -> *mut HSD_RectS16 { let r = rnd(); unsafe { &raw mut RS16[((r >> 8) % 1) as usize] } }
pub fn picko_HSD_RectS16() -> *mut HSD_RectS16 { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut RS16[((r >> 8) % 1) as usize] } } }
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
        preg(&raw const COB, core::mem::size_of_val(&COB));
        preg(&raw const WOB, core::mem::size_of_val(&WOB));
        preg(&raw const AOB, core::mem::size_of_val(&AOB));
        preg(&raw const CDS, core::mem::size_of_val(&CDS));
        preg(&raw const CINF, core::mem::size_of_val(&CINF));
        preg(&raw const CIN, core::mem::size_of_val(&CIN));
        preg(&raw const CAN, core::mem::size_of_val(&CAN));
        preg(&raw const WDS, core::mem::size_of_val(&WDS));
        preg(&raw const WAN, core::mem::size_of_val(&WAN));
        preg(&raw const ADS, core::mem::size_of_val(&ADS));
        preg(&raw const RMO, core::mem::size_of_val(&RMO));
        preg(&raw const OBD, core::mem::size_of_val(&OBD));
        preg(&raw const SCS, core::mem::size_of_val(&SCS));
        preg(&raw const RS16, core::mem::size_of_val(&RS16));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS)); preg(&raw const PPA, core::mem::size_of_val(&PPA));
    }
}

pub unsafe fn setup_world() {
    COB = core::mem::zeroed();
    WOB = core::mem::zeroed();
    AOB = core::mem::zeroed();
    CDS = core::mem::zeroed();
    CINF = core::mem::zeroed();
    CIN = core::mem::zeroed();
    CAN = core::mem::zeroed();
    WDS = core::mem::zeroed();
    WAN = core::mem::zeroed();
    ADS = core::mem::zeroed();
    RMO = core::mem::zeroed();
    OBD = core::mem::zeroed();
    SCS = core::mem::zeroed();
    RS16 = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..4 { PPA[i] = core::ptr::null_mut(); }
    for i in 0..3 { g::fill_HSD_CObj(&mut COB[i]); }
    for i in 0..4 { g::fill_HSD_WObj(&mut WOB[i]); }
    for i in 0..2 { g::fill_HSD_AObj(&mut AOB[i]); }
    for i in 0..2 { g::fill_HSD_CObjDesc(&mut CDS[i]); }
    for i in 0..2 { g::fill_HSD_CObjInfo(&mut CINF[i]); }
    for i in 0..2 { g::fill_HSD_ClassInfo(&mut CIN[i]); }
    for i in 0..2 { g::fill_HSD_CameraAnim(&mut CAN[i]); }
    for i in 0..2 { g::fill_HSD_WObjDesc(&mut WDS[i]); }
    for i in 0..2 { g::fill_HSD_WObjAnim(&mut WAN[i]); }
    for i in 0..2 { g::fill_HSD_AObjDesc(&mut ADS[i]); }
    for i in 0..1 { g::fill_GXRenderModeObj(&mut RMO[i]); }
    for i in 0..2 { g::fill_HSD_ObjData(&mut OBD[i]); }
    for i in 0..1 { g::fill_Scissor(&mut SCS[i]); }
    for i in 0..1 { g::fill_HSD_RectS16(&mut RS16[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..3 { g::hash_HSD_CObj(&COB[i]); }
    for i in 0..4 { g::hash_HSD_WObj(&WOB[i]); }
    for i in 0..2 { g::hash_HSD_AObj(&AOB[i]); }
    for i in 0..2 { g::hash_HSD_CObjDesc(&CDS[i]); }
    for i in 0..2 { g::hash_HSD_CObjInfo(&CINF[i]); }
    for i in 0..2 { g::hash_HSD_ClassInfo(&CIN[i]); }
    for i in 0..2 { g::hash_HSD_CameraAnim(&CAN[i]); }
    for i in 0..2 { g::hash_HSD_WObjDesc(&WDS[i]); }
    for i in 0..2 { g::hash_HSD_WObjAnim(&WAN[i]); }
    for i in 0..2 { g::hash_HSD_AObjDesc(&ADS[i]); }
    for i in 0..1 { g::hash_GXRenderModeObj(&RMO[i]); }
    for i in 0..2 { g::hash_HSD_ObjData(&OBD[i]); }
    for i in 0..1 { g::hash_Scissor(&SCS[i]); }
    for i in 0..1 { g::hash_HSD_RectS16(&RS16[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    for i in 0..4 { hm(pi(PPA[i])); }
    g::hashg_all();
    g::hasho_all();
}

// harness = false (Cargo.toml): a case that ends in a panic is cut off inside the hook (see mtu_harness::run_case),
// which a libtest main thread cannot survive at exit.
fn main() { cobj_matches_c(); println!("tu_cobj: ok"); }

fn cobj_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();
preg(&raw const MT, 128); m::use_ext_data = true; freg(cb_rel as usize); freg(cb_dest as usize); freg(cb_amn as usize); freg(cb_init as usize); melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());
        install_abort_hook();
        let mut c = Check::new(include_str!("data/mtu_cobj.txt"));
        let mut total = 0u32;
        for op in 0..39u32 {
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
                27 => 300u32,
                28 => 300u32,
                29 => 300u32,
                30 => 300u32,
                31 => 300u32,
                32 => 300u32,
                33 => 300u32,
                34 => 300u32,
                35 => 300u32,
                36 => 300u32,
                37 => 300u32,
                38 => 300u32,
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
{ for q in 0..3usize { COB[q].parent.parent.class_info = (&raw mut CINF[q & 1]) as *mut HSD_ClassInfo; COB[q].eyepos = &raw mut WOB[(fv() % 4) as usize]; COB[q].interest = &raw mut WOB[(fv() % 4) as usize]; COB[q].aobj = if fv() & 3 != 0 { &raw mut AOB[(fv() & 1) as usize] } else { null_mut() }; COB[q].proj_mtx = if fv() & 3 != 0 { (&raw mut MT[q & 1]) as *mut Mtx } else { null_mut() }; COB[q].projection_type = (1 + fv() % 3) as u8; } for q in 0..2usize { CINF[q].load = Some(m::CObjLoad); CINF[q].parent.parent.init = Some(cb_init); CINF[q].parent.parent.release = Some(cb_rel); CINF[q].parent.parent.destroy = Some(cb_dest); } for q in 0..2usize { CDS[q].class_name = if fv() & 1 != 0 { (&raw mut STRS[0]) as *mut i8 } else { null_mut() }; CDS[q].common.eyepos = &raw mut WDS[0]; CDS[q].common.interest = &raw mut WDS[1]; CDS[q].common.up_vector = if fv() & 1 != 0 { &raw mut VA[(fv() % 4) as usize] } else { null_mut() }; } m::hsdCObj.parent.parent.head.parent = &raw mut CIN[0]; CIN[0].init = Some(cb_init); CIN[0].release = Some(cb_rel); CIN[0].destroy = Some(cb_dest); CIN[0].amnesia = Some(cb_amn); for q in 0..4usize { WOB[q].parent.parent.class_info = &raw mut CIN[0]; } m::default_class = if fv() & 1 != 0 { &raw mut CIN[1] } else { null_mut() }; m::current = if fv() & 1 != 0 { &raw mut COB[(fv() % 3) as usize] } else { null_mut() }; }
                let mut ret = 0u32;
                let aborted = run_case(|| unsafe { match op {
                    0 => { 
let a0 = pick_HSD_CObj();
let a1 = fv() as _;
let a2 = fv() as _;
let a3 = fv() as _;
m::HSD_CObjEraseScreen(a0, a1, a2, a3);
 }
                    1 => { 
let a0 = pick_HSD_CObj();
let a1 = fv() as _;
m::HSD_CObjRemoveAnimByFlags(a0, a1);
 }
                    2 => { 
let a0 = pick_HSD_CObj();
m::HSD_CObjRemoveAnim(a0);
 }
                    3 => { 
let a0 = pick_HSD_CObj();
let a1 = pick_HSD_CameraAnim();
m::HSD_CObjAddAnim(a0, a1);
 }
                    4 => { 
let a0 = (&raw mut BUF) as *mut c_void;
let a1 = fv() as _;
let a2 = pick_HSD_ObjData();
m::CObjUpdateFunc(a0, a1, a2);
 }
                    5 => { 
let a0 = pick_HSD_CObj();
m::HSD_CObjAnim(a0);
 }
                    6 => { 
let a0 = pick_HSD_CObj();
let a1 = rw();
m::HSD_CObjReqAnim(a0, a1);
 }
                    7 => { 
let a0 = pick_HSD_CObj();
let a1 = (&raw mut BUF) as *mut [[f32; 4]; 4];
ret = m::makeProjectionMtx(a0, &mut *a1) as u32;
 }
                    8 => { 
let a0 = pick_HSD_CObj();
m::HSD_CObjSetupViewingMtx(a0);
 }
                    9 => { 
let a0 = pick_HSD_CObj();
m::HSD_CObjUpdateView(a0);
 }
                    10 => { 
let a0 = pick_HSD_CObj();
ret = m::HSD_CObjSetCurrent(a0) as u32;
 }
                    11 => { 

m::HSD_CObjEndCurrent();
 }
                    12 => { 
let a0 = pick_HSD_CObj();
ret = pi(m::HSD_CObjGetInterestWObj(a0));
 }
                    13 => { 
let a0 = pick_HSD_CObj();
ret = pi(m::HSD_CObjGetEyePositionWObj(a0));
 }
                    14 => { 
let a0 = pick_HSD_CObj();
m::HSD_CObjSetMtxDirty(a0);
 }
                    15 => { 
let a0 = pick_HSD_CObj();
ret = m::HSD_CObjMtxIsDirty(a0) as u32;
 }
                    16 => { 
let a0 = pick_HSD_CObj();
let a1 = rw();
m::HSD_CObjSetFov(a0, a1);
 }
                    17 => { 
let a0 = pick_HSD_CObj();
ret = (m::HSD_CObjGetNear(a0)).to_bits();
 }
                    18 => { 
let a0 = pick_HSD_CObj();
let a1 = rw();
m::HSD_CObjSetNear(a0, a1);
 }
                    19 => { 
let a0 = pick_HSD_CObj();
ret = (m::HSD_CObjGetFar(a0)).to_bits();
 }
                    20 => { 
let a0 = pick_HSD_CObj();
let a1 = rw();
m::HSD_CObjSetFar(a0, a1);
 }
                    21 => { 
let a0 = pick_HSD_CObj();
let a1 = pick_Scissor();
m::HSD_CObjSetScissor(a0, a1);
 }
                    22 => { 
let a0 = pick_HSD_CObj();
let a1 = pick_HSD_RectS16();
m::HSD_CObjSetViewport(a0, a1);
 }
                    23 => { 
let a0 = pick_HSD_CObj();
ret = m::HSD_CObjGetProjectionType(a0) as u32;
 }
                    24 => { 
let a0 = pick_HSD_CObj();
let a1 = rw();
let a2 = rw();
m::HSD_CObjSetPerspective(a0, a1, a2);
 }
                    25 => { 
let a0 = pick_HSD_CObj();
let a1 = rw();
let a2 = rw();
let a3 = rw();
let a4 = rw();
m::HSD_CObjSetFrustum(a0, a1, a2, a3, a4);
 }
                    26 => { 
let a0 = pick_HSD_CObj();
let a1 = rw();
let a2 = rw();
let a3 = rw();
let a4 = rw();
m::HSD_CObjSetOrtho(a0, a1, a2, a3, a4);
 }
                    27 => { 
let a0 = pick_HSD_CObj();
let a1 = fv() as _;
m::HSD_CObjSetFlags(a0, a1);
 }
                    28 => { 
let a0 = pick_HSD_CObj();
let a1 = fv() as _;
m::HSD_CObjClearFlags(a0, a1);
 }
                    29 => { 

ret = pi(m::HSD_CObjGetCurrent());
 }
                    30 => { 
let a0 = pick_HSD_ClassInfo();
m::HSD_CObjSetDefaultClass(a0);
 }
                    31 => { 

ret = pi(m::HSD_CObjAlloc());
 }
                    32 => { 
let a0 = pick_HSD_CObj();
let a1 = pick_HSD_CObjDesc();
ret = m::CObjLoad(a0, a1) as u32;
 }
                    33 => { 
let a0 = pick_HSD_CObj();
let a1 = pick_HSD_CObjDesc();
m::HSD_CObjInit(a0, a1);
 }
                    34 => { 
let a0 = pick_HSD_CObjDesc();
ret = pi(m::HSD_CObjLoadDesc(a0));
 }
                    35 => { 
let a0 = pick_HSD_CObj() as *mut HSD_Class;
ret = m::CObjInit(a0) as u32;
 }
                    36 => { 
let a0 = pick_HSD_CObj() as *mut HSD_Class;
m::CObjRelease(a0);
 }
                    37 => { 
let a0 = pick_HSD_ClassInfo();
m::CObjAmnesia(a0);
 }
                    38 => { 

m::CObjInfoInit();
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
