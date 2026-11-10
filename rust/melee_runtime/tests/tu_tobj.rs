//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/tobj.c.
//! src/tu/tobj.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py tobj`).
//! C functions of the TU this test covers (the private ones through their public callers): DifferentTluts, HSD_ImageDescAlloc, HSD_ImageDescCopyFromEFB, HSD_ImageDescFree, HSD_Index2TexCoord, HSD_Index2TexMap, HSD_Index2TexMtx, HSD_TGTex2Index, HSD_TObjAddAnim, HSD_TObjAddAnimAll, HSD_TObjAlloc, HSD_TObjAnim, HSD_TObjAnimAll, HSD_TObjAssignResources, HSD_TObjFree, HSD_TObjGetDefaultClass, HSD_TObjGetNext, HSD_TObjLoadDesc, HSD_TObjRemove, HSD_TObjRemoveAll, HSD_TObjRemoveAnim, HSD_TObjRemoveAnimAll, HSD_TObjReqAnim, HSD_TObjReqAnimAll, HSD_TObjReqAnimAllByFlags, HSD_TObjReqAnimByFlags, HSD_TObjSetDefaultClass, HSD_TObjSetup, HSD_TObjSetupTextureCoordGen, HSD_TObjSetupVolatileTev, HSD_TObjTevAlloc, HSD_TObjTevFree, HSD_TObjTevLoadDesc, HSD_TObjTevRemove, HSD_TexCoord2Index, HSD_TexCoordID2TexGenSrc, HSD_TexMap2Index, HSD_TexMapID2PTTexMtx, HSD_TexMtx2Index, HSD_TlutAlloc, HSD_TlutFree, HSD_TlutLoadDesc, HSD_TlutRemove, MakeColorGenTExp, MakeTextureMtx, SetupEmbossBumpTev, TObjAmnesia, TObjInfoInit, TObjLoad, TObjMakeTExp, TObjRelease, TObjSetupMtx, TObjSetupTevModulateShadow, TObjUpdateFunc, _HSD_TObjGetCurrentByType, lookupTextureAnim, setupTextureCoordGen, setupTextureCoordGenBump, setupTextureCoordGenToon.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_HSD_TObjRemoveAnim: u32 = 0;
const OP_HSD_TObjRemoveAnimAll: u32 = 1;
const OP_lookupTextureAnim: u32 = 2;
const OP_HSD_TObjAddAnim: u32 = 3;
const OP_HSD_TObjAddAnimAll: u32 = 4;
const OP_HSD_TObjReqAnimByFlags: u32 = 5;
const OP_HSD_TObjReqAnimAllByFlags: u32 = 6;
const OP_HSD_TObjReqAnim: u32 = 7;
const OP_HSD_TObjReqAnimAll: u32 = 8;
const OP_TObjUpdateFunc: u32 = 9;
const OP_HSD_TObjAnim: u32 = 10;
const OP_HSD_TObjAnimAll: u32 = 11;
const OP_TObjLoad: u32 = 12;
const OP_HSD_TObjLoadDesc: u32 = 13;
const OP_HSD_TlutLoadDesc: u32 = 14;
const OP_HSD_TObjTevLoadDesc: u32 = 15;
const OP__HSD_TObjGetCurrentByType: u32 = 16;
const OP_MakeTextureMtx: u32 = 17;
const OP_TObjSetupMtx: u32 = 18;
const OP_setupTextureCoordGen: u32 = 19;
const OP_setupTextureCoordGenBump: u32 = 20;
const OP_setupTextureCoordGenToon: u32 = 21;
const OP_HSD_TObjSetupTextureCoordGen: u32 = 22;
const OP_TObjSetupTevModulateShadow: u32 = 23;
const OP_SetupEmbossBumpTev: u32 = 24;
const OP_HSD_TObjSetupVolatileTev: u32 = 25;
const OP_MakeColorGenTExp: u32 = 26;
const OP_TObjMakeTExp: u32 = 27;
const OP_HSD_TObjAssignResources: u32 = 28;
const OP_DifferentTluts: u32 = 29;
const OP_HSD_TObjSetup: u32 = 30;
const OP_HSD_Index2TexMtx: u32 = 31;
const OP_HSD_TObjRemove: u32 = 32;
const OP_HSD_TObjRemoveAll: u32 = 33;
const OP_HSD_TObjGetNext: u32 = 34;
const OP_HSD_TObjSetDefaultClass: u32 = 35;
const OP_HSD_TObjGetDefaultClass: u32 = 36;
const OP_HSD_TObjAlloc: u32 = 37;
const OP_HSD_TObjFree: u32 = 38;
const OP_HSD_TlutAlloc: u32 = 39;
const OP_HSD_TlutFree: u32 = 40;
const OP_HSD_TlutRemove: u32 = 41;
const OP_HSD_TObjTevAlloc: u32 = 42;
const OP_HSD_TObjTevFree: u32 = 43;
const OP_HSD_TObjTevRemove: u32 = 44;
const OP_HSD_ImageDescAlloc: u32 = 45;
const OP_HSD_ImageDescFree: u32 = 46;
const OP_HSD_ImageDescCopyFromEFB: u32 = 47;
const OP_TObjRelease: u32 = 48;
const OP_TObjAmnesia: u32 = 49;
const OP_TObjInfoInit: u32 = 50;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::tobj as m;
use mtu_harness::*;
#[path = "mtu/tobj_gen.rs"]
mod g;


static mut IMT: [[*mut HSD_ImageDesc; 4]; 3] = [[null_mut(); 4]; 3];
static mut TLT: [[*mut HSD_Tlut; 4]; 3] = [[null_mut(); 4]; 3];
static mut TLDT: [[*mut HSD_TlutDesc; 4]; 3] = [[null_mut(); 4]; 3];
static mut HP: [u8; 4096] = [0; 4096];
static mut MA: [u8; 1024] = [0; 1024];
static mut TXA: [[u8; 64]; 4] = [[0; 64]; 4];
static mut HPI: usize = 0;
static mut MAI: usize = 0; static mut VF: [f32; 4] = [0.0, 1.0, 2.0, 0.5];
unsafe extern "C" fn cb_rel(c: *mut HSD_Class) { t(8203); t(pi(c)); }
unsafe extern "C" fn cb_dest(c: *mut HSD_Class) { t(8204); t(pi(c)); }
unsafe extern "C" fn cb_amn(c: *mut HSD_ClassInfo) { t(8205); t(pi(c)); }
const FMT: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 8, 9, 10, 14, 7];


pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut PPA: [*mut c_void; 4] = [core::ptr::null_mut(); 4];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub static mut TOB: [HSD_TObj; 6] = unsafe { core::mem::zeroed() };
pub static mut TA: [HSD_TexAnim; 2] = unsafe { core::mem::zeroed() };
pub static mut AOB: [HSD_AObj; 2] = unsafe { core::mem::zeroed() };
pub static mut AD: [HSD_AObjDesc; 2] = unsafe { core::mem::zeroed() };
pub static mut IMG: [HSD_ImageDesc; 3] = unsafe { core::mem::zeroed() };
pub static mut TL: [HSD_Tlut; 3] = unsafe { core::mem::zeroed() };
pub static mut TLD: [HSD_TlutDesc; 3] = unsafe { core::mem::zeroed() };
pub static mut TEV: [HSD_TObjTev; 2] = unsafe { core::mem::zeroed() };
pub static mut TEVD: [HSD_TObjTevDesc; 2] = unsafe { core::mem::zeroed() };
pub static mut TDS: [HSD_TObjDesc; 2] = unsafe { core::mem::zeroed() };
pub static mut LOD: [HSD_TexLODDesc; 1] = unsafe { core::mem::zeroed() };
pub static mut TINF: [HSD_TObjInfo; 1] = unsafe { core::mem::zeroed() };
pub static mut CIN: [HSD_ClassInfo; 2] = unsafe { core::mem::zeroed() };
pub static mut LOB: [HSD_LObj; 1] = unsafe { core::mem::zeroed() };
pub static mut COB: [HSD_CObj; 1] = unsafe { core::mem::zeroed() };
pub static mut OBD: [HSD_ObjData; 2] = unsafe { core::mem::zeroed() };
pub fn pick_HSD_TObj() -> *mut HSD_TObj { let r = rnd(); unsafe { &raw mut TOB[((r >> 8) % 6) as usize] } }
pub fn picko_HSD_TObj() -> *mut HSD_TObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut TOB[((r >> 8) % 6) as usize] } } }
pub fn pick_HSD_TexAnim() -> *mut HSD_TexAnim { let r = rnd(); unsafe { &raw mut TA[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_TexAnim() -> *mut HSD_TexAnim { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut TA[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_AObj() -> *mut HSD_AObj { let r = rnd(); unsafe { &raw mut AOB[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_AObj() -> *mut HSD_AObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut AOB[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_AObjDesc() -> *mut HSD_AObjDesc { let r = rnd(); unsafe { &raw mut AD[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_AObjDesc() -> *mut HSD_AObjDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut AD[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_ImageDesc() -> *mut HSD_ImageDesc { let r = rnd(); unsafe { &raw mut IMG[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_ImageDesc() -> *mut HSD_ImageDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut IMG[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_Tlut() -> *mut HSD_Tlut { let r = rnd(); unsafe { &raw mut TL[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_Tlut() -> *mut HSD_Tlut { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut TL[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_TlutDesc() -> *mut HSD_TlutDesc { let r = rnd(); unsafe { &raw mut TLD[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_TlutDesc() -> *mut HSD_TlutDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut TLD[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_TObjTev() -> *mut HSD_TObjTev { let r = rnd(); unsafe { &raw mut TEV[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_TObjTev() -> *mut HSD_TObjTev { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut TEV[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_TObjTevDesc() -> *mut HSD_TObjTevDesc { let r = rnd(); unsafe { &raw mut TEVD[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_TObjTevDesc() -> *mut HSD_TObjTevDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut TEVD[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_TObjDesc() -> *mut HSD_TObjDesc { let r = rnd(); unsafe { &raw mut TDS[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_TObjDesc() -> *mut HSD_TObjDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut TDS[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_TexLODDesc() -> *mut HSD_TexLODDesc { let r = rnd(); unsafe { &raw mut LOD[((r >> 8) % 1) as usize] } }
pub fn picko_HSD_TexLODDesc() -> *mut HSD_TexLODDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut LOD[((r >> 8) % 1) as usize] } } }
pub fn pick_HSD_TObjInfo() -> *mut HSD_TObjInfo { let r = rnd(); unsafe { &raw mut TINF[((r >> 8) % 1) as usize] } }
pub fn picko_HSD_TObjInfo() -> *mut HSD_TObjInfo { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut TINF[((r >> 8) % 1) as usize] } } }
pub fn pick_HSD_ClassInfo() -> *mut HSD_ClassInfo { let r = rnd(); unsafe { &raw mut CIN[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_ClassInfo() -> *mut HSD_ClassInfo { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut CIN[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_LObj() -> *mut HSD_LObj { let r = rnd(); unsafe { &raw mut LOB[((r >> 8) % 1) as usize] } }
pub fn picko_HSD_LObj() -> *mut HSD_LObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut LOB[((r >> 8) % 1) as usize] } } }
pub fn pick_HSD_CObj() -> *mut HSD_CObj { let r = rnd(); unsafe { &raw mut COB[((r >> 8) % 1) as usize] } }
pub fn picko_HSD_CObj() -> *mut HSD_CObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut COB[((r >> 8) % 1) as usize] } } }
pub fn pick_HSD_ObjData() -> *mut HSD_ObjData { let r = rnd(); unsafe { &raw mut OBD[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_ObjData() -> *mut HSD_ObjData { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut OBD[((r >> 8) % 2) as usize] } } }
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
        preg(&raw const TOB, core::mem::size_of_val(&TOB));
        preg(&raw const TA, core::mem::size_of_val(&TA));
        preg(&raw const AOB, core::mem::size_of_val(&AOB));
        preg(&raw const AD, core::mem::size_of_val(&AD));
        preg(&raw const IMG, core::mem::size_of_val(&IMG));
        preg(&raw const TL, core::mem::size_of_val(&TL));
        preg(&raw const TLD, core::mem::size_of_val(&TLD));
        preg(&raw const TEV, core::mem::size_of_val(&TEV));
        preg(&raw const TEVD, core::mem::size_of_val(&TEVD));
        preg(&raw const TDS, core::mem::size_of_val(&TDS));
        preg(&raw const LOD, core::mem::size_of_val(&LOD));
        preg(&raw const TINF, core::mem::size_of_val(&TINF));
        preg(&raw const CIN, core::mem::size_of_val(&CIN));
        preg(&raw const LOB, core::mem::size_of_val(&LOB));
        preg(&raw const COB, core::mem::size_of_val(&COB));
        preg(&raw const OBD, core::mem::size_of_val(&OBD));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS)); preg(&raw const PPA, core::mem::size_of_val(&PPA));
    }
}

pub unsafe fn setup_world() {
    melee_runtime::tu::tobj::ext::use_ext_data = true;
    TOB = core::mem::zeroed();
    TA = core::mem::zeroed();
    AOB = core::mem::zeroed();
    AD = core::mem::zeroed();
    IMG = core::mem::zeroed();
    TL = core::mem::zeroed();
    TLD = core::mem::zeroed();
    TEV = core::mem::zeroed();
    TEVD = core::mem::zeroed();
    TDS = core::mem::zeroed();
    LOD = core::mem::zeroed();
    TINF = core::mem::zeroed();
    CIN = core::mem::zeroed();
    LOB = core::mem::zeroed();
    COB = core::mem::zeroed();
    OBD = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..4 { PPA[i] = core::ptr::null_mut(); }
    for i in 0..6 { g::fill_HSD_TObj(&mut TOB[i]); }
    for i in 0..2 { g::fill_HSD_TexAnim(&mut TA[i]); }
    for i in 0..2 { g::fill_HSD_AObj(&mut AOB[i]); }
    for i in 0..2 { g::fill_HSD_AObjDesc(&mut AD[i]); }
    for i in 0..3 { g::fill_HSD_ImageDesc(&mut IMG[i]); }
    for i in 0..3 { g::fill_HSD_Tlut(&mut TL[i]); }
    for i in 0..3 { g::fill_HSD_TlutDesc(&mut TLD[i]); }
    for i in 0..2 { g::fill_HSD_TObjTev(&mut TEV[i]); }
    for i in 0..2 { g::fill_HSD_TObjTevDesc(&mut TEVD[i]); }
    for i in 0..2 { g::fill_HSD_TObjDesc(&mut TDS[i]); }
    for i in 0..1 { g::fill_HSD_TexLODDesc(&mut LOD[i]); }
    for i in 0..1 { g::fill_HSD_TObjInfo(&mut TINF[i]); }
    for i in 0..2 { g::fill_HSD_ClassInfo(&mut CIN[i]); }
    for i in 0..1 { g::fill_HSD_LObj(&mut LOB[i]); }
    for i in 0..1 { g::fill_HSD_CObj(&mut COB[i]); }
    for i in 0..2 { g::fill_HSD_ObjData(&mut OBD[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..6 { g::hash_HSD_TObj(&TOB[i]); }
    for i in 0..2 { g::hash_HSD_TexAnim(&TA[i]); }
    for i in 0..2 { g::hash_HSD_AObj(&AOB[i]); }
    for i in 0..2 { g::hash_HSD_AObjDesc(&AD[i]); }
    for i in 0..3 { g::hash_HSD_ImageDesc(&IMG[i]); }
    for i in 0..3 { g::hash_HSD_Tlut(&TL[i]); }
    for i in 0..3 { g::hash_HSD_TlutDesc(&TLD[i]); }
    for i in 0..2 { g::hash_HSD_TObjTev(&TEV[i]); }
    for i in 0..2 { g::hash_HSD_TObjTevDesc(&TEVD[i]); }
    for i in 0..2 { g::hash_HSD_TObjDesc(&TDS[i]); }
    for i in 0..1 { g::hash_HSD_TexLODDesc(&LOD[i]); }
    for i in 0..1 { g::hash_HSD_TObjInfo(&TINF[i]); }
    for i in 0..2 { g::hash_HSD_ClassInfo(&CIN[i]); }
    for i in 0..1 { g::hash_HSD_LObj(&LOB[i]); }
    for i in 0..1 { g::hash_HSD_CObj(&COB[i]); }
    for i in 0..2 { g::hash_HSD_ObjData(&OBD[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    for i in 0..4 { hm(pi(PPA[i])); }
    g::hashg_all();
    g::hasho_all();
}

// harness = false (Cargo.toml): a case that ends in a panic is cut off inside the hook (see mtu_harness::run_case),
// which a libtest main thread cannot survive at exit.
fn main() { tobj_matches_c(); println!("tu_tobj: ok"); }

fn tobj_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();
preg(&raw const IMT, 96); preg(&raw const TLT, 96); preg(&raw const TLDT, 96); preg(&raw const HP, 4096); preg(&raw const MA, 1024); preg(&raw const TXA, 256); freg(cb_rel as usize); freg(cb_dest as usize); freg(cb_amn as usize); melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());
        install_abort_hook();
        let mut c = Check::new(include_str!("data/mtu_tobj.txt"));
        let mut total = 0u32;
        for op in 0..51u32 {
            let n = match op {
                0 => 250u32,
                1 => 250u32,
                2 => 250u32,
                3 => 250u32,
                4 => 250u32,
                5 => 250u32,
                6 => 250u32,
                7 => 250u32,
                8 => 250u32,
                9 => 250u32,
                10 => 250u32,
                11 => 250u32,
                12 => 250u32,
                13 => 250u32,
                14 => 250u32,
                15 => 250u32,
                16 => 250u32,
                17 => 250u32,
                18 => 250u32,
                19 => 250u32,
                20 => 250u32,
                21 => 250u32,
                22 => 250u32,
                23 => 250u32,
                24 => 250u32,
                25 => 250u32,
                26 => 250u32,
                27 => 250u32,
                28 => 250u32,
                29 => 250u32,
                30 => 250u32,
                31 => 250u32,
                32 => 250u32,
                33 => 250u32,
                34 => 250u32,
                35 => 250u32,
                36 => 250u32,
                37 => 250u32,
                38 => 250u32,
                39 => 250u32,
                40 => 250u32,
                41 => 250u32,
                42 => 250u32,
                43 => 250u32,
                44 => 250u32,
                45 => 250u32,
                46 => 250u32,
                47 => 250u32,
                48 => 250u32,
                49 => 250u32,
                50 => 250u32,
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
{ HPI = 0; MAI = 0; for q in 0..3usize { for j in 0..4usize { IMT[q][j] = if j < 3 { &raw mut IMG[(j + q) % 3] } else { null_mut() }; TLT[q][j] = if (j as u32) < 1 + fv() % 3 { &raw mut TL[(j + q) % 3] } else { null_mut() }; TLDT[q][j] = if j < 3 { &raw mut TLD[(j + q) % 3] } else { null_mut() }; } } for q in 0..3usize { TLT[q][3] = null_mut(); } for q in 0..6usize { TOB[q].next = if q < 3 { &raw mut TOB[q + 1] } else { null_mut() }; TOB[q].parent.parent.class_info = (&raw mut TINF[0]) as *mut HSD_ClassInfo; TOB[q].id = (fv() % 9) as _; TOB[q].coord = (fv() % 9) as _; TOB[q].repeat_s = if fv() % 16 != 0 { (1 + fv() % 4) as u8 } else { 0 }; TOB[q].repeat_t = if fv() % 16 != 0 { (1 + fv() % 4) as u8 } else { 0 }; TOB[q].wrap_s = (fv() % 3) as _; TOB[q].wrap_t = (fv() % 3) as _; TOB[q].imagedesc = &raw mut IMG[(fv() % 3) as usize]; TOB[q].lod = if q < 2 || (fv() & 1 != 0) { &raw mut LOD[0] } else { null_mut() }; TOB[q].tlut = if fv() & 3 != 0 { &raw mut TL[(fv() % 3) as usize] } else { null_mut() }; TOB[q].tluttbl = (&raw mut TLT[(fv() % 3) as usize]) as *mut *mut HSD_Tlut; TOB[q].imagetbl = (&raw mut IMT[(fv() % 3) as usize]) as *mut *mut HSD_ImageDesc; TOB[q].tlut_no = if fv() & 1 != 0 { -1 } else { (fv() % 4) as _ }; TOB[q].tev = if q < 2 || (fv() & 3 != 0) { &raw mut TEV[(fv() & 1) as usize] } else { null_mut() }; TOB[q].aobj = if fv() & 3 != 0 { &raw mut AOB[(fv() & 1) as usize] } else { null_mut() }; } for q in 0..3usize { IMG[q].format = FMT[(fv() % 12) as usize] as _; IMG[q].image_ptr = if fv() & 7 != 0 { (&raw mut BUF) as *mut u8 } else { null_mut() }; IMG[q].mipmap = fv() & 1; IMG[q].width = (1 + fv() % 64) as u16; IMG[q].height = (1 + fv() % 64) as u16; TL[q].lut = (&raw mut BUF) as *mut c_void; TL[q].n_entries = (fv() % 300) as u16; TL[q].fmt = (fv() % 3) as _; } for q in 0..2usize { TA[q].next = if q < 1 { &raw mut TA[1] } else { null_mut() }; TA[q].id = (fv() % 9) as _; TA[q].aobjdesc = if fv() & 3 != 0 { &raw mut AD[(fv() & 1) as usize] } else { null_mut() }; TA[q].imagetbl = (&raw mut IMT[(fv() % 3) as usize]) as *mut *mut HSD_ImageDesc; TA[q].tluttbl = (&raw mut TLDT[(fv() % 3) as usize]) as *mut *mut HSD_TlutDesc; TA[q].n_tluttbl = (fv() % 3) as u16; } for q in 0..2usize { TDS[q].next = if q < 1 { &raw mut TDS[1] } else { null_mut() }; TDS[q].class_name = if fv() & 1 != 0 { (&raw mut STRS[0]) as *mut i8 } else { null_mut() }; TDS[q].imagedesc = &raw mut IMG[(fv() % 3) as usize]; TDS[q].tlutdesc = if fv() & 1 != 0 { &raw mut TLD[(fv() % 3) as usize] } else { null_mut() }; TDS[q].tev = if fv() & 1 != 0 { &raw mut TEVD[(fv() & 1) as usize] } else { null_mut() }; TDS[q].lod = if fv() & 1 != 0 { &raw mut LOD[0] } else { null_mut() }; } TINF[0].make_mtx = Some(m::MakeTextureMtx); TINF[0].load = Some(m::TObjLoad); TINF[0].make_texp = Some(core::mem::transmute::<usize, _>(m::TObjMakeTExp as usize)); TINF[0].parent.release = Some(cb_rel); TINF[0].parent.destroy = Some(cb_dest); m::hsdTObj.parent.head.parent = &raw mut CIN[0]; CIN[0].release = Some(cb_rel); CIN[0].amnesia = Some(cb_amn); m::default_class = if fv() & 1 != 0 { &raw mut TINF[0] } else { null_mut() }; m::tobj_head = if fv() & 1 != 0 { &raw mut TOB[(fv() % 4) as usize] } else { null_mut() }; }
                let mut ret = 0u32;
                let aborted = run_case(|| unsafe { match op {
                    0 => { 
let a0 = pick_HSD_TObj();
m::HSD_TObjRemoveAnim(a0);
 }
                    1 => { 
let a0 = pick_HSD_TObj();
m::HSD_TObjRemoveAnimAll(a0);
 }
                    2 => { 
let a0 = fv() as _;
let a1 = pick_HSD_TexAnim();
ret = pi(m::lookupTextureAnim(a0, a1));
 }
                    3 => { 
let a0 = pick_HSD_TObj();
let a1 = pick_HSD_TexAnim();
m::HSD_TObjAddAnim(a0, a1);
 }
                    4 => { 
let a0 = pick_HSD_TObj();
let a1 = pick_HSD_TexAnim();
m::HSD_TObjAddAnimAll(a0, a1);
 }
                    5 => { 
let a0 = pick_HSD_TObj();
let a1 = rw();
let a2 = fv() | 1;
m::HSD_TObjReqAnimByFlags(a0, a1, a2);
 }
                    6 => { 
let a0 = pick_HSD_TObj();
let a1 = rw();
let a2 = fv() as _;
m::HSD_TObjReqAnimAllByFlags(a0, a1, a2);
 }
                    7 => { 
let a0 = pick_HSD_TObj();
let a1 = rw();
m::HSD_TObjReqAnim(a0, a1);
 }
                    8 => { 
let a0 = pick_HSD_TObj();
let a1 = rw();
m::HSD_TObjReqAnimAll(a0, a1);
 }
                    9 => { 
let a0 = (&raw mut TOB[(fv() % 2) as usize]) as *mut c_void;
let a1 = (fv() % 24) as i32;
let a2 = (&raw mut VF[(fv() % 4) as usize]) as *mut HSD_ObjData;
m::TObjUpdateFunc(a0, a1, a2);
 }
                    10 => { 
let a0 = pick_HSD_TObj();
m::HSD_TObjAnim(a0);
 }
                    11 => { 
let a0 = pick_HSD_TObj();
m::HSD_TObjAnimAll(a0);
 }
                    12 => { 
let a0 = pick_HSD_TObj();
let a1 = pick_HSD_TObjDesc();
ret = m::TObjLoad(a0, a1) as u32;
 }
                    13 => { 
let a0 = pick_HSD_TObjDesc();
ret = pi(m::HSD_TObjLoadDesc(a0));
 }
                    14 => { 
let a0 = pick_HSD_TlutDesc();
ret = pi(m::HSD_TlutLoadDesc(a0));
 }
                    15 => { 
let a0 = pick_HSD_TObjTevDesc();
ret = pi(m::HSD_TObjTevLoadDesc(a0));
 }
                    16 => { 
let a0 = pick_HSD_TObj();
let a1 = fv() as _;
ret = pi(m::_HSD_TObjGetCurrentByType(a0, a1));
 }
                    17 => { 
let a0 = pick_HSD_TObj();
m::MakeTextureMtx(a0);
 }
                    18 => { 
let a0 = pick_HSD_TObj();
m::TObjSetupMtx(a0);
 }
                    19 => { 
let a0 = pick_HSD_TObj();
m::setupTextureCoordGen(a0);
 }
                    20 => { 
let a0 = pick_HSD_TObj();
m::setupTextureCoordGenBump(a0);
 }
                    21 => { 
let a0 = pick_HSD_TObj();
m::setupTextureCoordGenToon(a0);
 }
                    22 => { 
let a0 = pick_HSD_TObj();
m::HSD_TObjSetupTextureCoordGen(a0);
 }
                    23 => { 
let a0 = pick_HSD_TObj();
m::TObjSetupTevModulateShadow(a0);
 }
                    24 => { 
let a0 = pick_HSD_TObj();
m::SetupEmbossBumpTev(a0);
 }
                    25 => { 
let a0 = pick_HSD_TObj();
let a1 = fv() as _;
m::HSD_TObjSetupVolatileTev(a0, a1);
 }
                    26 => { 
let a0 = fv() as _;
let a1 = &raw mut TOB[(fv() % 2) as usize];
let a2 = (&raw mut PPA[(fv() % 4) as usize]) as *mut _;
let a3 = (&raw mut PPA[(fv() % 4) as usize]) as *mut _;
let a4 = (&raw mut PPA[(fv() % 4) as usize]) as *mut _;
let a5 = fv() as _;
m::MakeColorGenTExp(a0, a1, a2, a3, a4, a5);
 }
                    27 => { 
let a0 = pick_HSD_TObj();
let a1 = fv() as _;
let a2 = fv() as _;
let a3 = (&raw mut PPA[(fv() % 4) as usize]) as *mut _;
let a4 = (&raw mut PPA[(fv() % 4) as usize]) as *mut _;
let a5 = (&raw mut PPA[(fv() % 4) as usize]) as *mut _;
m::TObjMakeTExp(a0, a1, a2, a3, a4, a5);
 }
                    28 => { 
let a0 = pick_HSD_TObj();
ret = m::HSD_TObjAssignResources(a0) as u32;
 }
                    29 => { 
let a0 = pick_HSD_Tlut();
let a1 = pick_HSD_Tlut();
ret = m::DifferentTluts(a0, a1) as u32;
 }
                    30 => { 
let a0 = pick_HSD_TObj();
m::HSD_TObjSetup(a0);
 }
                    31 => { 
let a0 = fv() % 12;
ret = m::HSD_Index2TexMtx(a0) as u32;
 }
                    32 => { 
let a0 = pick_HSD_TObj();
m::HSD_TObjRemove(a0);
 }
                    33 => { 
let a0 = pick_HSD_TObj();
m::HSD_TObjRemoveAll(a0);
 }
                    34 => { 
let a0 = pick_HSD_TObj();
ret = pi(m::HSD_TObjGetNext(a0));
 }
                    35 => { 
let a0 = pick_HSD_TObjInfo();
m::HSD_TObjSetDefaultClass(a0);
 }
                    36 => { 

ret = pi(m::HSD_TObjGetDefaultClass());
 }
                    37 => { 

ret = pi(m::HSD_TObjAlloc());
 }
                    38 => { 
let a0 = pick_HSD_TObj();
m::HSD_TObjFree(a0);
 }
                    39 => { 

ret = pi(m::HSD_TlutAlloc());
 }
                    40 => { 
let a0 = pick_HSD_Tlut();
m::HSD_TlutFree(a0);
 }
                    41 => { 
let a0 = pick_HSD_Tlut();
m::HSD_TlutRemove(a0);
 }
                    42 => { 

ret = pi(m::HSD_TObjTevAlloc());
 }
                    43 => { 
let a0 = pick_HSD_TObjTev();
m::HSD_TObjTevFree(a0);
 }
                    44 => { 
let a0 = pick_HSD_TObjTev();
m::HSD_TObjTevRemove(a0);
 }
                    45 => { 

ret = pi(m::HSD_ImageDescAlloc());
 }
                    46 => { 
let a0 = pick_HSD_ImageDesc();
m::HSD_ImageDescFree(a0);
 }
                    47 => { 
let a0 = pick_HSD_ImageDesc();
let a1 = fv() as _;
let a2 = fv() as _;
let a3 = fv() as _;
let a4 = fv() as _;
m::HSD_ImageDescCopyFromEFB(a0, a1, a2, a3, a4);
 }
                    48 => { 
let a0 = pick_HSD_TObj() as *mut HSD_Class;
m::TObjRelease(a0);
 }
                    49 => { 
let a0 = pick_HSD_ClassInfo();
m::TObjAmnesia(a0);
 }
                    50 => { 

m::TObjInfoInit();
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
