//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/pobj.c.
//! src/tu/pobj.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py pobj`).
//! C functions of the TU this test covers (the private ones through their public callers): GenoPalDisp, GenoPalInfoInit, GenoPalLoad, GenoPalSetupMtx, GetSetupFlags, HSD_ClearVtxDesc, HSD_EnvelopeAlloc, HSD_EnvelopeFree, HSD_EnvelopeListFree, HSD_PObjAddAnim, HSD_PObjAddAnimAll, HSD_PObjAlloc, HSD_PObjAnim, HSD_PObjAnimAll, HSD_PObjClearMtxMark, HSD_PObjDisp, HSD_PObjFree, HSD_PObjGetDefaultClass, HSD_PObjGetFlags, HSD_PObjGetMtxMark, HSD_PObjLoadDesc, HSD_PObjRemove, HSD_PObjRemoveAll, HSD_PObjRemoveAnimAllByFlags, HSD_PObjRemoveAnimByFlags, HSD_PObjReqAnimAllByFlags, HSD_PObjReqAnimByFlags, HSD_PObjResolveRefs, HSD_PObjResolveRefsAll, HSD_PObjSetDefaultClass, HSD_PObjSetMtxMark, HSD_ShapeSetFree, HSD_ShapeSetRemove, PObjAmnesia, PObjDispShapeAnim, PObjDispSimplePrimitive, PObjInfoInit, PObjLoad, PObjRelease, PObjSetupMtx, PObjUpdateFunc, SetupEnvelopeModelMtx, SetupRigidModelMtx, SetupSharedVtxModelMtx, ShapeSetSetAnimResult, decode_s16_xyz, decode_s8_xyz, decode_u16_xyz, decode_u8_xyz, drawShapeAnim, env_hash_f32s, geno_pal_compute, geno_pal_desc, geno_pal_force_convert, geno_pal_selftest_check, get_shape_nbt_xyz, get_shape_normal_xyz, get_shape_vertex_xyz, interpretShapeAnimDisplayList, loadEnvelopeDesc, loadShapeSetDesc, resolveEnvelope, setupArrayDesc, setupShapeAnimArrayDesc, setupShapeAnimVtxDesc, setupVtxDesc.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_HSD_PObjGetFlags: u32 = 0;
const OP_HSD_PObjRemoveAnimByFlags: u32 = 1;
const OP_HSD_PObjRemoveAnimAllByFlags: u32 = 2;
const OP_HSD_PObjAddAnim: u32 = 3;
const OP_HSD_PObjAddAnimAll: u32 = 4;
const OP_HSD_PObjReqAnimByFlags: u32 = 5;
const OP_HSD_PObjReqAnimAllByFlags: u32 = 6;
const OP_ShapeSetSetAnimResult: u32 = 7;
const OP_PObjUpdateFunc: u32 = 8;
const OP_HSD_PObjAnim: u32 = 9;
const OP_HSD_PObjAnimAll: u32 = 10;
const OP_HSD_EnvelopeAlloc: u32 = 11;
const OP_HSD_EnvelopeFree: u32 = 12;
const OP_HSD_EnvelopeListFree: u32 = 13;
const OP_loadEnvelopeDesc: u32 = 14;
const OP_HSD_ShapeSetFree: u32 = 15;
const OP_HSD_ShapeSetRemove: u32 = 16;
const OP_loadShapeSetDesc: u32 = 17;
const OP_PObjLoad: u32 = 18;
const OP_HSD_PObjLoadDesc: u32 = 19;
const OP_HSD_PObjRemove: u32 = 20;
const OP_HSD_PObjRemoveAll: u32 = 21;
const OP_HSD_PObjGetDefaultClass: u32 = 22;
const OP_HSD_PObjSetDefaultClass: u32 = 23;
const OP_HSD_PObjAlloc: u32 = 24;
const OP_HSD_PObjFree: u32 = 25;
const OP_resolveEnvelope: u32 = 26;
const OP_HSD_PObjResolveRefs: u32 = 27;
const OP_HSD_PObjResolveRefsAll: u32 = 28;
const OP_HSD_ClearVtxDesc: u32 = 29;
const OP_setupArrayDesc: u32 = 30;
const OP_setupVtxDesc: u32 = 31;
const OP_HSD_PObjClearMtxMark: u32 = 32;
const OP_HSD_PObjSetMtxMark: u32 = 33;
const OP_HSD_PObjGetMtxMark: u32 = 34;
const OP_GetSetupFlags: u32 = 35;
const OP_SetupRigidModelMtx: u32 = 36;
const OP_SetupSharedVtxModelMtx: u32 = 37;
const OP_SetupEnvelopeModelMtx: u32 = 38;
const OP_PObjSetupMtx: u32 = 39;
const OP_PObjDispSimplePrimitive: u32 = 40;
const OP_HSD_PObjDisp: u32 = 41;
const OP_PObjRelease: u32 = 42;
const OP_PObjAmnesia: u32 = 43;
const OP_PObjInfoInit: u32 = 44;
const OP_GenoPalInfoInit: u32 = 45;
const OP_geno_pal_desc: u32 = 46;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::pobj as m;
use mtu_harness::*;
#[path = "mtu/pobj_gen.rs"]
mod g;


static mut EPL: [[*mut HSD_EnvelopeDesc; 4]; 3] = [[null_mut(); 4]; 3]; static mut RK: usize = 0;
unsafe extern "C" fn cb_rel(c: *mut HSD_Class) { t(8203); t(pi(c)); }
unsafe extern "C" fn cb_dest(c: *mut HSD_Class) { t(8204); t(pi(c)); }
unsafe extern "C" fn cb_amn(c: *mut HSD_ClassInfo) { t(8205); t(pi(c)); }
unsafe extern "C" fn cb_disp(p: *mut HSD_PObj, v: *mut [f32; 4], m: *mut [f32; 4], r: u32) { t(8206); t(pi(p)); t(pi(v)); t(pi(m)); t(r); }
unsafe extern "C" fn cb_setup(p: *mut HSD_PObj, v: *mut [f32; 4], m: *mut [f32; 4], r: u32) { t(8207); t(pi(p)); t(pi(v)); t(pi(m)); t(r); }
unsafe extern "C" fn cb_load(p: *mut HSD_PObj, d: *mut HSD_PObjDesc) -> i32 { t(8208); t(pi(p)); t(pi(d)); (fv() % 3) as i32 - 1 }
static mut MT: [[u8; 48]; 4] = [[0; 48]; 4];


pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut PPA: [*mut c_void; 4] = [core::ptr::null_mut(); 4];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub static mut P0: [HSD_PObj; 3] = unsafe { core::mem::zeroed() };
pub static mut P1: [HSD_ShapeAnim; 3] = unsafe { core::mem::zeroed() };
pub static mut P2: [HSD_ShapeSet; 3] = unsafe { core::mem::zeroed() };
pub static mut P3: [HSD_ObjData; 3] = unsafe { core::mem::zeroed() };
pub static mut P4: [HSD_Envelope; 3] = unsafe { core::mem::zeroed() };
pub static mut P5: [HSD_SList; 4] = unsafe { core::mem::zeroed() };
pub static mut P6: [HSD_ShapeSetDesc; 2] = unsafe { core::mem::zeroed() };
pub static mut P7: [HSD_PObjDesc; 3] = unsafe { core::mem::zeroed() };
pub static mut P8: [HSD_PObjInfo; 3] = unsafe { core::mem::zeroed() };
pub static mut P9: [HSD_VtxDescList; 9] = unsafe { core::mem::zeroed() };
pub static mut P10: [HSD_JObj; 3] = unsafe { core::mem::zeroed() };
pub static mut P12: [HSD_ClassInfo; 3] = unsafe { core::mem::zeroed() };
pub static mut P13: [HSD_EnvelopeDesc; 3] = unsafe { core::mem::zeroed() };
pub static mut P14: [HSD_AObj; 2] = unsafe { core::mem::zeroed() };
pub static mut P15: [HSD_AObjDesc; 2] = unsafe { core::mem::zeroed() };
pub static mut P16: [HSD_Joint; 2] = unsafe { core::mem::zeroed() };
pub static mut P11: [HSD_Class; 2] = unsafe { core::mem::zeroed() };
pub fn pick_HSD_PObj() -> *mut HSD_PObj { let r = rnd(); unsafe { &raw mut P0[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_PObj() -> *mut HSD_PObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut P0[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_ShapeAnim() -> *mut HSD_ShapeAnim { let r = rnd(); unsafe { &raw mut P1[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_ShapeAnim() -> *mut HSD_ShapeAnim { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut P1[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_ShapeSet() -> *mut HSD_ShapeSet { let r = rnd(); unsafe { &raw mut P2[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_ShapeSet() -> *mut HSD_ShapeSet { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut P2[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_ObjData() -> *mut HSD_ObjData { let r = rnd(); unsafe { &raw mut P3[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_ObjData() -> *mut HSD_ObjData { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut P3[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_Envelope() -> *mut HSD_Envelope { let r = rnd(); unsafe { &raw mut P4[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_Envelope() -> *mut HSD_Envelope { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut P4[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_SList() -> *mut HSD_SList { let r = rnd(); unsafe { &raw mut P5[((r >> 8) % 4) as usize] } }
pub fn picko_HSD_SList() -> *mut HSD_SList { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut P5[((r >> 8) % 4) as usize] } } }
pub fn pick_HSD_ShapeSetDesc() -> *mut HSD_ShapeSetDesc { let r = rnd(); unsafe { &raw mut P6[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_ShapeSetDesc() -> *mut HSD_ShapeSetDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut P6[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_PObjDesc() -> *mut HSD_PObjDesc { let r = rnd(); unsafe { &raw mut P7[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_PObjDesc() -> *mut HSD_PObjDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut P7[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_PObjInfo() -> *mut HSD_PObjInfo { let r = rnd(); unsafe { &raw mut P8[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_PObjInfo() -> *mut HSD_PObjInfo { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut P8[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_VtxDescList() -> *mut HSD_VtxDescList { let r = rnd(); unsafe { &raw mut P9[((r >> 8) % 9) as usize] } }
pub fn picko_HSD_VtxDescList() -> *mut HSD_VtxDescList { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut P9[((r >> 8) % 9) as usize] } } }
pub fn pick_HSD_JObj() -> *mut HSD_JObj { let r = rnd(); unsafe { &raw mut P10[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_JObj() -> *mut HSD_JObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut P10[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_ClassInfo() -> *mut HSD_ClassInfo { let r = rnd(); unsafe { &raw mut P12[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_ClassInfo() -> *mut HSD_ClassInfo { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut P12[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_EnvelopeDesc() -> *mut HSD_EnvelopeDesc { let r = rnd(); unsafe { &raw mut P13[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_EnvelopeDesc() -> *mut HSD_EnvelopeDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut P13[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_AObj() -> *mut HSD_AObj { let r = rnd(); unsafe { &raw mut P14[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_AObj() -> *mut HSD_AObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut P14[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_AObjDesc() -> *mut HSD_AObjDesc { let r = rnd(); unsafe { &raw mut P15[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_AObjDesc() -> *mut HSD_AObjDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut P15[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_Joint() -> *mut HSD_Joint { let r = rnd(); unsafe { &raw mut P16[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_Joint() -> *mut HSD_Joint { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut P16[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_Class() -> *mut HSD_Class { let r = rnd(); unsafe { &raw mut P11[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_Class() -> *mut HSD_Class { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut P11[((r >> 8) % 2) as usize] } } }
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
        preg(&raw const P1, core::mem::size_of_val(&P1));
        preg(&raw const P2, core::mem::size_of_val(&P2));
        preg(&raw const P3, core::mem::size_of_val(&P3));
        preg(&raw const P4, core::mem::size_of_val(&P4));
        preg(&raw const P5, core::mem::size_of_val(&P5));
        preg(&raw const P6, core::mem::size_of_val(&P6));
        preg(&raw const P7, core::mem::size_of_val(&P7));
        preg(&raw const P8, core::mem::size_of_val(&P8));
        preg(&raw const P9, core::mem::size_of_val(&P9));
        preg(&raw const P10, core::mem::size_of_val(&P10));
        preg(&raw const P12, core::mem::size_of_val(&P12));
        preg(&raw const P13, core::mem::size_of_val(&P13));
        preg(&raw const P14, core::mem::size_of_val(&P14));
        preg(&raw const P15, core::mem::size_of_val(&P15));
        preg(&raw const P16, core::mem::size_of_val(&P16));
        preg(&raw const P11, core::mem::size_of_val(&P11));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS)); preg(&raw const PPA, core::mem::size_of_val(&PPA));
    }
}

pub unsafe fn setup_world() {
    P0 = core::mem::zeroed();
    P1 = core::mem::zeroed();
    P2 = core::mem::zeroed();
    P3 = core::mem::zeroed();
    P4 = core::mem::zeroed();
    P5 = core::mem::zeroed();
    P6 = core::mem::zeroed();
    P7 = core::mem::zeroed();
    P8 = core::mem::zeroed();
    P9 = core::mem::zeroed();
    P10 = core::mem::zeroed();
    P12 = core::mem::zeroed();
    P13 = core::mem::zeroed();
    P14 = core::mem::zeroed();
    P15 = core::mem::zeroed();
    P16 = core::mem::zeroed();
    P11 = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..4 { PPA[i] = core::ptr::null_mut(); }
    for i in 0..3 { g::fill_HSD_PObj(&mut P0[i]); }
    for i in 0..3 { g::fill_HSD_ShapeAnim(&mut P1[i]); }
    for i in 0..3 { g::fill_HSD_ShapeSet(&mut P2[i]); }
    for i in 0..3 { g::fill_HSD_ObjData(&mut P3[i]); }
    for i in 0..3 { g::fill_HSD_Envelope(&mut P4[i]); }
    for i in 0..4 { g::fill_HSD_SList(&mut P5[i]); }
    for i in 0..2 { g::fill_HSD_ShapeSetDesc(&mut P6[i]); }
    for i in 0..3 { g::fill_HSD_PObjDesc(&mut P7[i]); }
    for i in 0..3 { g::fill_HSD_PObjInfo(&mut P8[i]); }
    for i in 0..9 { g::fill_HSD_VtxDescList(&mut P9[i]); }
    for i in 0..3 { g::fill_HSD_JObj(&mut P10[i]); }
    for i in 0..3 { g::fill_HSD_ClassInfo(&mut P12[i]); }
    for i in 0..3 { g::fill_HSD_EnvelopeDesc(&mut P13[i]); }
    for i in 0..2 { g::fill_HSD_AObj(&mut P14[i]); }
    for i in 0..2 { g::fill_HSD_AObjDesc(&mut P15[i]); }
    for i in 0..2 { g::fill_HSD_Joint(&mut P16[i]); }
    for i in 0..2 { g::fill_HSD_Class(&mut P11[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..3 { g::hash_HSD_PObj(&P0[i]); }
    for i in 0..3 { g::hash_HSD_ShapeAnim(&P1[i]); }
    for i in 0..3 { g::hash_HSD_ShapeSet(&P2[i]); }
    for i in 0..3 { g::hash_HSD_ObjData(&P3[i]); }
    for i in 0..3 { g::hash_HSD_Envelope(&P4[i]); }
    for i in 0..4 { g::hash_HSD_SList(&P5[i]); }
    for i in 0..2 { g::hash_HSD_ShapeSetDesc(&P6[i]); }
    for i in 0..3 { g::hash_HSD_PObjDesc(&P7[i]); }
    for i in 0..3 { g::hash_HSD_PObjInfo(&P8[i]); }
    for i in 0..9 { g::hash_HSD_VtxDescList(&P9[i]); }
    for i in 0..3 { g::hash_HSD_JObj(&P10[i]); }
    for i in 0..3 { g::hash_HSD_ClassInfo(&P12[i]); }
    for i in 0..3 { g::hash_HSD_EnvelopeDesc(&P13[i]); }
    for i in 0..2 { g::hash_HSD_AObj(&P14[i]); }
    for i in 0..2 { g::hash_HSD_AObjDesc(&P15[i]); }
    for i in 0..2 { g::hash_HSD_Joint(&P16[i]); }
    for i in 0..2 { g::hash_HSD_Class(&P11[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    for i in 0..4 { hm(pi(PPA[i])); }
    g::hashg_all();
    g::hasho_all();
}

// harness = false (Cargo.toml): a case that ends in a panic is cut off inside the hook (see mtu_harness::run_case),
// which a libtest main thread cannot survive at exit.
fn main() { pobj_matches_c(); println!("tu_pobj: ok"); }

fn pobj_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();
preg(&raw const MT, 192); preg(&raw const EPL, 96); m::use_ext_data = true; freg(cb_rel as usize); freg(cb_dest as usize); freg(cb_amn as usize); freg(cb_disp as usize); freg(cb_setup as usize); freg(cb_load as usize); melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());
        install_abort_hook();
        let mut c = Check::new(include_str!("data/mtu_pobj.txt"));
        let mut total = 0u32;
        for op in 0..47u32 {
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
                39 => 300u32,
                40 => 300u32,
                41 => 300u32,
                42 => 300u32,
                43 => 300u32,
                44 => 300u32,
                45 => 300u32,
                46 => 300u32,
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
{  for q in 0..3usize { let ci = (&raw mut P8[q]) as *mut HSD_ClassInfo; (*ci).release = Some(cb_rel); (*ci).destroy = Some(cb_dest); (*ci).amnesia = Some(cb_amn); P8[q].disp = Some(cb_disp); P8[q].setup_mtx = if q == 0 { Some(m::PObjSetupMtx) } else { Some(cb_setup) }; P8[q].load = Some(cb_load); }  for l in 0..3usize { P9[3 * l].attr = fv() % 26; P9[3 * l + 1].attr = fv() % 26; P9[3 * l + 2].attr = 255; P9[3 * l].vertex = if fv() & 1 != 0 { (&raw mut BUF) as *mut c_void } else { null_mut() }; P9[3 * l + 1].vertex = if fv() & 1 != 0 { ((&raw mut BUF) as *mut u8).add(64) as *mut c_void } else { null_mut() }; P9[3 * l].attr_type = fv() % 4; P9[3 * l + 1].attr_type = fv() % 4; }  for q in 0..4usize { P5[q].next = if q < 3 && fv() & 1 != 0 { &raw mut P5[q + 1] } else { null_mut() }; P5[q].data = (&raw mut P4[(fv() % 3) as usize]) as *mut c_void; }  for q in 0..3usize { P4[q].next = if q < 2 && fv() & 1 != 0 { &raw mut P4[q + 1] } else { null_mut() }; P4[q].jobj = &raw mut P10[(fv() % 3) as usize]; }  for q in 0..3usize { let n = (fv() % 4) as usize; let mut k = 0usize; while k < n { EPL[q][k] = &raw mut P13[(fv() % 3) as usize]; k += 1; } EPL[q][k] = null_mut(); }  for q in 0..3usize { P13[q].joint = if fv() & 3 != 0 { &raw mut P16[(fv() & 1) as usize] } else { null_mut() }; } P13[2].joint = null_mut();  for q in 0..3usize { P2[q].aobj = if fv() & 1 != 0 { &raw mut P14[(fv() & 1) as usize] } else { null_mut() }; P2[q].flags &= !2u16; }  for q in 0..3usize { let kind: u32 = if q == 0 { 0 } else if q == 1 { 0x2000 } else { (fv() % 3) * 0x1000 };    P0[q].parent.class_info = (&raw mut P8[q]) as *mut HSD_ClassInfo; P0[q].next = if q < 2 && fv() & 1 != 0 { &raw mut P0[q + 1] } else { null_mut() }; P0[q].verts = &raw mut P9[(3 * (fv() % 3)) as usize];    P0[q].flags = ((fv() & 0xCFFF) | kind) as u16; P0[q].n_display = (fv() % 4) as u16; P0[q].display = (&raw mut BUF) as *mut u8;    if kind == 0 { P0[q].u.jobj = if q == 0 || fv() & 1 != 0 { &raw mut P10[(fv() % 3) as usize] } else { null_mut() }; } else if kind == 0x1000 { P0[q].u.shape_set = &raw mut P2[(fv() % 3) as usize]; } else { P0[q].u.envelope_list = if fv() & 3 != 0 { &raw mut P5[(fv() % 4) as usize] } else { null_mut() }; } }  for q in 0..3usize { let kind: u32 = (P0[q].flags & 0x3000) as u32; P7[q].n_display = (fv() % 4) as u16; P7[q].class_name = if fv() & 3 != 0 { null_mut() } else { (&raw mut STRS[(fv() % 4) as usize]) as *mut i8 }; P7[q].next = if !P0[q].next.is_null() { &raw mut P7[q + 1] } else { null_mut() }; P7[q].verts = &raw mut P9[(3 * (fv() % 3)) as usize];    P7[q].flags = ((fv() & 0xCFFF) | kind) as u16; P7[q].display = (&raw mut BUF) as *mut u8; if kind == 0 { P7[q].u.joint = if fv() & 3 != 0 { &raw mut P16[(fv() & 1) as usize] } else { null_mut() }; } else if kind == 0x1000 { P7[q].u.shape_set = &raw mut P6[(fv() & 1) as usize]; } else { P7[q].u.envelope_p = if fv() & 3 != 0 { (&raw mut EPL[(fv() % 3) as usize]) as *mut *mut HSD_EnvelopeDesc } else { null_mut() }; } }  for q in 0..2usize { P6[q].vertex_desc = &raw mut P9[0]; P6[q].normal_desc = &raw mut P9[3]; P6[q].flags &= !2u16; }  for q in 0..4usize { for i in 0..48usize { MT[q][i] = rnd() as u8; } }  m::hsdPObj.parent.release = Some(cb_rel); m::hsdPObj.parent.destroy = Some(cb_dest); m::hsdPObj.parent.amnesia = Some(cb_amn); m::hsdPObj.parent.head.parent = &raw mut P12[0]; P12[0].release = Some(cb_rel); P12[0].amnesia = Some(cb_amn); P12[0].destroy = Some(cb_dest);  m::default_class = if fv() & 1 != 0 { &raw mut P8[(fv() % 3) as usize] } else { null_mut() }; m::mtx_mark[0].obj = if fv() & 1 != 0 { (&raw mut P10[(fv() % 3) as usize]) as *mut c_void } else { null_mut() }; m::mtx_mark[1].obj = if fv() & 1 != 0 { (&raw mut P10[(fv() % 3) as usize]) as *mut c_void } else { null_mut() }; m::mtx_mark[0].mark = fv() % 4; m::mtx_mark[1].mark = fv() % 4;  m::prev_vtxdesc = if fv() & 1 != 0 { &raw mut P9[(3 * (fv() % 3)) as usize] } else { null_mut() }; m::prev_vtxdesclist_array = if fv() & 1 != 0 { &raw mut P9[(3 * (fv() % 3)) as usize] } else { null_mut() }; }
                let mut ret = 0u32;
                let aborted = run_case(|| unsafe { match op {
                    0 => { 
let a0 = pick_HSD_PObj();
ret = m::HSD_PObjGetFlags(a0) as u32;
 }
                    1 => { 
let a0 = pick_HSD_PObj();
let a1 = fv() as _;
m::HSD_PObjRemoveAnimByFlags(a0, a1);
 }
                    2 => { 
let a0 = pick_HSD_PObj();
let a1 = fv() as _;
m::HSD_PObjRemoveAnimAllByFlags(a0, a1);
 }
                    3 => { 
let a0 = pick_HSD_PObj();
let a1 = pick_HSD_ShapeAnim();
m::HSD_PObjAddAnim(a0, a1);
 }
                    4 => { 
let a0 = pick_HSD_PObj();
let a1 = pick_HSD_ShapeAnim();
m::HSD_PObjAddAnimAll(a0, a1);
 }
                    5 => { 
let a0 = pick_HSD_PObj();
let a1 = rw();
let a2 = fv() as _;
m::HSD_PObjReqAnimByFlags(a0, a1, a2);
 }
                    6 => { 
let a0 = pick_HSD_PObj();
let a1 = rw();
let a2 = fv() as _;
m::HSD_PObjReqAnimAllByFlags(a0, a1, a2);
 }
                    7 => { 
let a0 = pick_HSD_ShapeSet();
let a1 = fv() as _;
let a2 = pick_HSD_ObjData();
m::ShapeSetSetAnimResult(a0, a1, a2);
 }
                    8 => { 
let a0 = picko_HSD_PObj() as *mut c_void;
let a1 = fv() as _;
let a2 = pick_HSD_ObjData();
m::PObjUpdateFunc(a0, a1, a2);
 }
                    9 => { 
let a0 = pick_HSD_PObj();
m::HSD_PObjAnim(a0);
 }
                    10 => { 
let a0 = pick_HSD_PObj();
m::HSD_PObjAnimAll(a0);
 }
                    11 => { 

ret = pi(m::HSD_EnvelopeAlloc());
 }
                    12 => { 
let a0 = pick_HSD_Envelope();
m::HSD_EnvelopeFree(a0);
 }
                    13 => { 
let a0 = pick_HSD_SList();
m::HSD_EnvelopeListFree(a0);
 }
                    14 => { 
let a0 = (&raw mut PPA[(fv() % 4) as usize]) as *mut _;
ret = pi(m::loadEnvelopeDesc(a0));
 }
                    15 => { 
let a0 = pick_HSD_ShapeSet();
m::HSD_ShapeSetFree(a0);
 }
                    16 => { 
let a0 = pick_HSD_ShapeSet();
m::HSD_ShapeSetRemove(a0);
 }
                    17 => { 
let a0 = pick_HSD_ShapeSetDesc();
ret = pi(m::loadShapeSetDesc(a0));
 }
                    18 => { 
let a0 = pick_HSD_PObj();
let a1 = pick_HSD_PObjDesc();
ret = m::PObjLoad(a0, a1) as u32;
 }
                    19 => { 
let a0 = pick_HSD_PObjDesc();
ret = pi(m::HSD_PObjLoadDesc(a0));
 }
                    20 => { 
let a0 = pick_HSD_PObj();
m::HSD_PObjRemove(a0);
 }
                    21 => { 
let a0 = pick_HSD_PObj();
m::HSD_PObjRemoveAll(a0);
 }
                    22 => { 

ret = pi(m::HSD_PObjGetDefaultClass());
 }
                    23 => { 
let a0 = pick_HSD_PObjInfo();
m::HSD_PObjSetDefaultClass(a0);
 }
                    24 => { 

ret = pi(m::HSD_PObjAlloc());
 }
                    25 => { 
let a0 = pick_HSD_PObj();
m::HSD_PObjFree(a0);
 }
                    26 => { 
let a0 = pick_HSD_SList();
let a1 = (&raw mut PPA[(fv() % 4) as usize]) as *mut _;
m::resolveEnvelope(a0, a1);
 }
                    27 => { 
let a0 = { RK = (fv() % 3) as usize; &raw mut P0[RK] };
let a1 = &raw mut P7[RK];
m::HSD_PObjResolveRefs(a0, a1);
 }
                    28 => { 
let a0 = &raw mut P0[0];
let a1 = &raw mut P7[0];
m::HSD_PObjResolveRefsAll(a0, a1);
 }
                    29 => { 

m::HSD_ClearVtxDesc();
 }
                    30 => { 
let a0 = pick_HSD_VtxDescList();
m::setupArrayDesc(a0);
 }
                    31 => { 
let a0 = pick_HSD_PObj();
m::setupVtxDesc(a0);
 }
                    32 => { 
let a0 = (&raw mut BUF) as *mut c_void;
let a1 = fv() as _;
m::HSD_PObjClearMtxMark(a0, a1);
 }
                    33 => { 
let a0 = (fv() % 4) as i32;
let a1 = (&raw mut BUF) as *mut c_void;
let a2 = fv() as _;
m::HSD_PObjSetMtxMark(a0, a1, a2);
 }
                    34 => { 
let a0 = fv() as _;
let a1 = (&raw mut PPA[(fv() % 4) as usize]) as *mut _;
let a2 = (&raw mut UA) as *mut u32;
m::HSD_PObjGetMtxMark(a0, a1, a2);
 }
                    35 => { 
let a0 = pick_HSD_JObj();
let a1 = fv() as _;
ret = m::GetSetupFlags(a0, a1) as u32;
 }
                    36 => { 
let a0 = pick_HSD_PObj();
let a1 = (&raw mut BUF) as *mut _;
let a2 = (&raw mut BUF) as *mut _;
let a3 = fv() as _;
m::SetupRigidModelMtx(a0, a1, a2, a3);
 }
                    37 => { 
let a0 = &raw mut P0[0];
let a1 = (&raw mut BUF) as *mut _;
let a2 = (&raw mut BUF) as *mut _;
let a3 = fv() as _;
m::SetupSharedVtxModelMtx(a0, a1, a2, a3);
 }
                    38 => { 
let a0 = &raw mut P0[1];
let a1 = (&raw mut BUF) as *mut _;
let a2 = (&raw mut BUF) as *mut _;
let a3 = fv() as _;
m::SetupEnvelopeModelMtx(a0, a1, a2, a3);
 }
                    39 => { 
let a0 = pick_HSD_PObj();
let a1 = (&raw mut BUF) as *mut _;
let a2 = (&raw mut BUF) as *mut _;
let a3 = fv() as _;
m::PObjSetupMtx(a0, a1, a2, a3);
 }
                    40 => { 
let a0 = pick_HSD_PObj();
let a1 = fv() as _;
m::PObjDispSimplePrimitive(a0, a1);
 }
                    41 => { 
let a0 = if fv() & 1 != 0 { &raw mut P0[1] } else { &raw mut P0[0] };
let a1 = (&raw mut MT[(rnd() % 4) as usize]) as *mut _;
let a2 = (&raw mut MT[(rnd() % 4) as usize]) as *mut _;
let a3 = fv() as _;
m::HSD_PObjDisp(a0, a1, a2, a3);
 }
                    42 => { 
let a0 = pick_HSD_PObj() as *mut HSD_Class;
m::PObjRelease(a0);
 }
                    43 => { 
let a0 = pick_HSD_ClassInfo();
m::PObjAmnesia(a0);
 }
                    44 => { 

m::PObjInfoInit();
 }
                    45 => { 

m::GenoPalInfoInit();
 }
                    46 => { 
let a0 = pick_HSD_PObjDesc();
ret = m::geno_pal_desc(a0) as u32;
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
