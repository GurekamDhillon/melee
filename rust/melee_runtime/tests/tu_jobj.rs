//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/jobj.c.
//! src/tu/jobj.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py jobj`).
//! C functions of the TU this test covers (the private ones through their public callers): HSD_JObjAddAnim, HSD_JObjAddAnimAll, HSD_JObjAddChild, HSD_JObjAddDObj, HSD_JObjAddNext, HSD_JObjAlloc, HSD_JObjAnim, HSD_JObjAnimAll, HSD_JObjCheckDepend, HSD_JObjClearFlags, HSD_JObjClearFlagsAll, HSD_JObjDeleteRObj, HSD_JObjDispAll, HSD_JObjGetCurrent, HSD_JObjGetDObj, HSD_JObjGetFlags, HSD_JObjGetPrev, HSD_JObjLoadJoint, HSD_JObjMakeMatrix, HSD_JObjPrependRObj, HSD_JObjRemove, HSD_JObjRemoveAll, HSD_JObjRemoveAnim, HSD_JObjRemoveAnimAll, HSD_JObjRemoveAnimAllByFlags, HSD_JObjRemoveAnimByFlags, HSD_JObjReparent, HSD_JObjReqAnim, HSD_JObjReqAnimAll, HSD_JObjReqAnimAllByFlags, HSD_JObjReqAnimByFlags, HSD_JObjResetRST, HSD_JObjResolveRefs, HSD_JObjResolveRefsAll, HSD_JObjSetCurrent, HSD_JObjSetDPtclCallback, HSD_JObjSetDefaultClass, HSD_JObjSetFlags, HSD_JObjSetFlagsAll, HSD_JObjSetMtxDirtySub, HSD_JObjSetupMatrixSub, HSD_JObjUnref, HSD_JObjUnrefThis, HSD_JObjWalkTree, HSD_JObjWalkTree0, JObjAmnesia, JObjAnimAll, JObjInfoInit, JObjInit, JObjLoad, JObjLoadJointSub, JObjRelease, JObjReleaseChild, JObjResetRST, JObjSortAnim, JObjUpdateFunc, RecalcParentTrspBits, UpdateParentTrspBits, has_scl, jobj_get_effector, jobj_get_effector_checked, jobj_get_joint2, resolveIKJoint1, resolveIKJoint2, robj_set_next.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_HSD_JObjSetDefaultClass: u32 = 0;
const OP_JObjLoadJointSub: u32 = 1;
const OP_JObjLoad: u32 = 2;
const OP_HSD_JObjLoadJoint: u32 = 3;
const OP_HSD_JObjResolveRefs: u32 = 4;
const OP_HSD_JObjResolveRefsAll: u32 = 5;
const OP_HSD_JObjUnref: u32 = 6;
const OP_HSD_JObjUnrefThis: u32 = 7;
const OP_HSD_JObjAlloc: u32 = 8;
const OP_HSD_JObjSetCurrent: u32 = 9;
const OP_HSD_JObjGetCurrent: u32 = 10;
const OP_JObjInit: u32 = 11;
const OP_JObjRelease: u32 = 12;
const OP_JObjAmnesia: u32 = 13;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::jobj as m;
use mtu_harness::*;
#[path = "mtu/jobj_gen.rs"]
mod g;


unsafe extern "C" fn cb_rel(c: *mut HSD_Class) { t(8203); t(pi(c)); }
unsafe extern "C" fn cb_dest(c: *mut HSD_Class) { t(8204); t(pi(c)); }
unsafe extern "C" fn cb_amn(c: *mut HSD_ClassInfo) { t(8205); t(pi(c)); }
unsafe extern "C" fn cb_init(c: *mut HSD_Class) -> i32 { t(8206); t(pi(c)); (fv() % 3) as i32 - 1 }
unsafe extern "C" fn cb_load(j: *mut HSD_JObj, d: *mut HSD_Joint, p: *mut HSD_JObj) -> i32 { t(8207); t(pi(j)); t(pi(d)); t(pi(p)); (fv() % 3) as i32 - 1 }
unsafe extern "C" fn cb_relchild(j: *mut HSD_JObj) { t(8208); t(pi(j)); }
static mut MT: [[u8; 48]; 4] = [[0; 48]; 4];


pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut PPA: [*mut c_void; 4] = [core::ptr::null_mut(); 4];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub static mut J: [HSD_JObj; 4] = unsafe { core::mem::zeroed() };
pub static mut JT: [HSD_Joint; 3] = unsafe { core::mem::zeroed() };
pub static mut JI: [HSD_JObjInfo; 3] = unsafe { core::mem::zeroed() };
pub static mut CI: [HSD_ClassInfo; 3] = unsafe { core::mem::zeroed() };
pub static mut DO: [HSD_DObj; 2] = unsafe { core::mem::zeroed() };
pub static mut DD: [HSD_DObjDesc; 2] = unsafe { core::mem::zeroed() };
pub static mut RO: [HSD_RObj; 2] = unsafe { core::mem::zeroed() };
pub static mut RD: [HSD_RObjDesc; 2] = unsafe { core::mem::zeroed() };
pub static mut AO: [HSD_AObj; 2] = unsafe { core::mem::zeroed() };
pub static mut SL: [HSD_SList; 3] = unsafe { core::mem::zeroed() };
pub static mut SPL: [HSD_Spline; 1] = unsafe { core::mem::zeroed() };
pub static mut CL: [HSD_Class; 2] = unsafe { core::mem::zeroed() };
pub fn pick_HSD_JObj() -> *mut HSD_JObj { let r = rnd(); unsafe { &raw mut J[((r >> 8) % 4) as usize] } }
pub fn picko_HSD_JObj() -> *mut HSD_JObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut J[((r >> 8) % 4) as usize] } } }
pub fn pick_HSD_Joint() -> *mut HSD_Joint { let r = rnd(); unsafe { &raw mut JT[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_Joint() -> *mut HSD_Joint { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut JT[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_JObjInfo() -> *mut HSD_JObjInfo { let r = rnd(); unsafe { &raw mut JI[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_JObjInfo() -> *mut HSD_JObjInfo { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut JI[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_ClassInfo() -> *mut HSD_ClassInfo { let r = rnd(); unsafe { &raw mut CI[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_ClassInfo() -> *mut HSD_ClassInfo { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut CI[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_DObj() -> *mut HSD_DObj { let r = rnd(); unsafe { &raw mut DO[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_DObj() -> *mut HSD_DObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut DO[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_DObjDesc() -> *mut HSD_DObjDesc { let r = rnd(); unsafe { &raw mut DD[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_DObjDesc() -> *mut HSD_DObjDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut DD[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_RObj() -> *mut HSD_RObj { let r = rnd(); unsafe { &raw mut RO[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_RObj() -> *mut HSD_RObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut RO[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_RObjDesc() -> *mut HSD_RObjDesc { let r = rnd(); unsafe { &raw mut RD[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_RObjDesc() -> *mut HSD_RObjDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut RD[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_AObj() -> *mut HSD_AObj { let r = rnd(); unsafe { &raw mut AO[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_AObj() -> *mut HSD_AObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut AO[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_SList() -> *mut HSD_SList { let r = rnd(); unsafe { &raw mut SL[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_SList() -> *mut HSD_SList { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut SL[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_Spline() -> *mut HSD_Spline { let r = rnd(); unsafe { &raw mut SPL[((r >> 8) % 1) as usize] } }
pub fn picko_HSD_Spline() -> *mut HSD_Spline { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut SPL[((r >> 8) % 1) as usize] } } }
pub fn pick_HSD_Class() -> *mut HSD_Class { let r = rnd(); unsafe { &raw mut CL[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_Class() -> *mut HSD_Class { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut CL[((r >> 8) % 2) as usize] } } }
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
        preg(&raw const J, core::mem::size_of_val(&J));
        preg(&raw const JT, core::mem::size_of_val(&JT));
        preg(&raw const JI, core::mem::size_of_val(&JI));
        preg(&raw const CI, core::mem::size_of_val(&CI));
        preg(&raw const DO, core::mem::size_of_val(&DO));
        preg(&raw const DD, core::mem::size_of_val(&DD));
        preg(&raw const RO, core::mem::size_of_val(&RO));
        preg(&raw const RD, core::mem::size_of_val(&RD));
        preg(&raw const AO, core::mem::size_of_val(&AO));
        preg(&raw const SL, core::mem::size_of_val(&SL));
        preg(&raw const SPL, core::mem::size_of_val(&SPL));
        preg(&raw const CL, core::mem::size_of_val(&CL));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS)); preg(&raw const PPA, core::mem::size_of_val(&PPA));
    }
}

pub unsafe fn setup_world() {
    J = core::mem::zeroed();
    JT = core::mem::zeroed();
    JI = core::mem::zeroed();
    CI = core::mem::zeroed();
    DO = core::mem::zeroed();
    DD = core::mem::zeroed();
    RO = core::mem::zeroed();
    RD = core::mem::zeroed();
    AO = core::mem::zeroed();
    SL = core::mem::zeroed();
    SPL = core::mem::zeroed();
    CL = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..4 { PPA[i] = core::ptr::null_mut(); }
    for i in 0..4 { g::fill_HSD_JObj(&mut J[i]); }
    for i in 0..3 { g::fill_HSD_Joint(&mut JT[i]); }
    for i in 0..3 { g::fill_HSD_JObjInfo(&mut JI[i]); }
    for i in 0..3 { g::fill_HSD_ClassInfo(&mut CI[i]); }
    for i in 0..2 { g::fill_HSD_DObj(&mut DO[i]); }
    for i in 0..2 { g::fill_HSD_DObjDesc(&mut DD[i]); }
    for i in 0..2 { g::fill_HSD_RObj(&mut RO[i]); }
    for i in 0..2 { g::fill_HSD_RObjDesc(&mut RD[i]); }
    for i in 0..2 { g::fill_HSD_AObj(&mut AO[i]); }
    for i in 0..3 { g::fill_HSD_SList(&mut SL[i]); }
    for i in 0..1 { g::fill_HSD_Spline(&mut SPL[i]); }
    for i in 0..2 { g::fill_HSD_Class(&mut CL[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..4 { g::hash_HSD_JObj(&J[i]); }
    for i in 0..3 { g::hash_HSD_Joint(&JT[i]); }
    for i in 0..3 { g::hash_HSD_JObjInfo(&JI[i]); }
    for i in 0..3 { g::hash_HSD_ClassInfo(&CI[i]); }
    for i in 0..2 { g::hash_HSD_DObj(&DO[i]); }
    for i in 0..2 { g::hash_HSD_DObjDesc(&DD[i]); }
    for i in 0..2 { g::hash_HSD_RObj(&RO[i]); }
    for i in 0..2 { g::hash_HSD_RObjDesc(&RD[i]); }
    for i in 0..2 { g::hash_HSD_AObj(&AO[i]); }
    for i in 0..3 { g::hash_HSD_SList(&SL[i]); }
    for i in 0..1 { g::hash_HSD_Spline(&SPL[i]); }
    for i in 0..2 { g::hash_HSD_Class(&CL[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    for i in 0..4 { hm(pi(PPA[i])); }
    g::hashg_all();
    g::hasho_all();
}

// harness = false (Cargo.toml): a case that ends in a panic is cut off inside the hook (see mtu_harness::run_case),
// which a libtest main thread cannot survive at exit.
fn main() { jobj_matches_c(); println!("tu_jobj: ok"); }

fn jobj_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();
preg(&raw const MT, 192); m::use_ext_data = true; freg(cb_rel as usize); freg(cb_dest as usize); freg(cb_amn as usize); freg(cb_init as usize); freg(cb_load as usize); freg(cb_relchild as usize); melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());
        install_abort_hook();
        let mut c = Check::new(include_str!("data/mtu_jobj.txt"));
        let mut total = 0u32;
        for op in 0..14u32 {
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
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
{  for q in 0..3usize { let ci = (&raw mut JI[q]) as *mut HSD_ClassInfo; (*ci).release = Some(cb_rel); (*ci).destroy = Some(cb_dest); (*ci).amnesia = Some(cb_amn); (*ci).init = Some(cb_init); JI[q].load = Some(cb_load); JI[q].release_child = Some(cb_relchild); }  for q in 0..3usize { CI[q].release = Some(cb_rel); CI[q].destroy = Some(cb_dest); CI[q].amnesia = Some(cb_amn); CI[q].init = Some(cb_init); }  for q in 0..4usize { J[q].object.parent.class_info = (&raw mut JI[q % 3]) as *mut HSD_ClassInfo; J[q].next = if q < 3 && fv() & 1 != 0 { &raw mut J[q + 1] } else { null_mut() };    J[q].child = if q < 3 && fv() & 1 != 0 { let k = q + 1 + (fv() % (3 - q as u32)) as usize; &raw mut J[k] } else { null_mut() };    J[q].parent = null_mut(); J[q].flags = fv() & 0xFFFFF; J[q].robj = if fv() & 1 != 0 { &raw mut RO[(fv() & 1) as usize] } else { null_mut() }; J[q].aobj = null_mut(); J[q].scl = if fv() & 1 != 0 { (&raw mut VA[(fv() % 4) as usize]) as *mut Vec } else { null_mut() };    J[q].envelopemtx = if fv() & 1 != 0 { (&raw mut MT[q]) as *mut [f32; 4] } else { null_mut() };    J[q].object.ref_count = if fv() & 3 != 0 { (fv() % 3) as u16 } else { 65535 }; J[q].object.ref_count_individual = (fv() % 3) as u16;    if J[q].flags & 0x4020 != 0 { J[q].u.ptcl = if fv() & 1 != 0 { &raw mut SL[(fv() % 3) as usize] } else { null_mut() }; } else { J[q].u.dobj = if fv() & 1 != 0 { &raw mut DO[(fv() & 1) as usize] } else { null_mut() }; } }  for q in 0..3usize { SL[q].next = if q < 2 && fv() & 1 != 0 { &raw mut SL[q + 1] } else { null_mut() }; }  for q in 0..3usize { JT[q].class_name = if fv() & 3 != 0 { null_mut() } else { (&raw mut STRS[(fv() % 4) as usize]) as *mut i8 }; JT[q].flags = fv() & 0xFFFFF;    JT[q].child = if q < 2 && fv() & 1 != 0 { let k = q + 1 + (fv() % (2 - q as u32)) as usize; &raw mut JT[k] } else { null_mut() }; JT[q].next = if q < 2 && fv() & 1 != 0 { &raw mut JT[q + 1] } else { null_mut() };    JT[q].mtx = if fv() & 1 != 0 { (&raw mut MT[q]) as *mut [f32; 4] } else { null_mut() }; JT[q].robjdesc = if fv() & 1 != 0 { &raw mut RD[(fv() & 1) as usize] } else { null_mut() };    if JT[q].flags & 0x4020 != 0 { if JT[q].flags & 0x4000 != 0 { JT[q].u.spline = &raw mut SPL[0]; } else { JT[q].u.ptcl = if fv() & 1 != 0 { &raw mut SL[(fv() % 3) as usize] } else { null_mut() }; } } else { JT[q].u.dobjdesc = if fv() & 1 != 0 { &raw mut DD[(fv() & 1) as usize] } else { null_mut() }; } }  for q in 0..4usize { for i in 0..48usize { MT[q][i] = rnd() as u8; } }  m::hsdJObj.parent.parent.head.parent = &raw mut CI[0]; m::hsdJObj.parent.parent.init = Some(cb_init); m::hsdJObj.parent.parent.release = Some(cb_rel); m::hsdJObj.parent.parent.amnesia = Some(cb_amn);  m::default_class = if fv() & 1 != 0 { &raw mut CI[(fv() % 3) as usize] } else { null_mut() }; m::ufc_callbacks = if fv() & 1 != 0 { &raw mut SL[(fv() % 3) as usize] } else { null_mut() }; m::current_jobj = if fv() & 1 != 0 { &raw mut J[(fv() % 4) as usize] } else { null_mut() }; }
                let mut ret = 0u32;
                let aborted = run_case(|| unsafe { match op {
                    0 => { 
let a0 = pick_HSD_ClassInfo();
m::HSD_JObjSetDefaultClass(a0);
 }
                    1 => { 
let a0 = pick_HSD_Joint();
let a1 = pick_HSD_JObj();
ret = pi(m::JObjLoadJointSub(a0, a1));
 }
                    2 => { 
let a0 = pick_HSD_JObj();
let a1 = pick_HSD_Joint();
let a2 = pick_HSD_JObj();
ret = m::JObjLoad(a0, a1, a2) as u32;
 }
                    3 => { 
let a0 = pick_HSD_Joint();
ret = pi(m::HSD_JObjLoadJoint(a0));
 }
                    4 => { 
let a0 = pick_HSD_JObj();
let a1 = pick_HSD_Joint();
m::HSD_JObjResolveRefs(a0, a1);
 }
                    5 => { 
let a0 = pick_HSD_JObj();
let a1 = pick_HSD_Joint();
m::HSD_JObjResolveRefsAll(a0, a1);
 }
                    6 => { 
let a0 = pick_HSD_JObj();
m::HSD_JObjUnref(a0);
 }
                    7 => { 
let a0 = pick_HSD_JObj();
m::HSD_JObjUnrefThis(a0);
 }
                    8 => { 

ret = pi(m::HSD_JObjAlloc());
 }
                    9 => { 
let a0 = pick_HSD_JObj();
m::HSD_JObjSetCurrent(a0);
 }
                    10 => { 

ret = pi(m::HSD_JObjGetCurrent());
 }
                    11 => { 
let a0 = pick_HSD_JObj() as *mut HSD_Class;
ret = m::JObjInit(a0) as u32;
 }
                    12 => { 
let a0 = pick_HSD_JObj() as *mut HSD_Class;
m::JObjRelease(a0);
 }
                    13 => { 
let a0 = pick_HSD_ClassInfo();
m::JObjAmnesia(a0);
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
