//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/mobj.c.
//! src/tu/mobj.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py mobj`).
//! C functions of the TU this test covers (the private ones through their public callers): HSD_MObjAddAnim, HSD_MObjAddShadowTexture, HSD_MObjAlloc, HSD_MObjAnim, HSD_MObjClearFlags, HSD_MObjCompileTev, HSD_MObjDeleteShadowTexture, HSD_MObjGetTObj, HSD_MObjLoadDesc, HSD_MObjRemove, HSD_MObjRemoveAnimByFlags, HSD_MObjReqAnim, HSD_MObjReqAnimByFlags, HSD_MObjSetAlpha, HSD_MObjSetCurrent, HSD_MObjSetDiffuseColor, HSD_MObjSetFlags, HSD_MObjSetToonTextureImage, HSD_MObjSetup, HSD_MObjUnset, HSD_MaterialAlloc, MObjAmnesia, MObjInfoInit, MObjLoad, MObjMakeTExp, MObjRelease, MObjSetupTev, MObjUpdateFunc.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_HSD_MObjSetCurrent: u32 = 0;
const OP_HSD_MObjSetFlags: u32 = 1;
const OP_HSD_MObjClearFlags: u32 = 2;
const OP_HSD_MObjRemoveAnimByFlags: u32 = 3;
const OP_HSD_MObjAddAnim: u32 = 4;
const OP_HSD_MObjReqAnimByFlags: u32 = 5;
const OP_HSD_MObjReqAnim: u32 = 6;
const OP_MObjUpdateFunc: u32 = 7;
const OP_HSD_MObjAnim: u32 = 8;
const OP_MObjLoad: u32 = 9;
const OP_HSD_MObjLoadDesc: u32 = 10;
const OP_MObjMakeTExp: u32 = 11;
const OP_HSD_MObjCompileTev: u32 = 12;
const OP_MObjSetupTev: u32 = 13;
const OP_HSD_MObjSetup: u32 = 14;
const OP_HSD_MObjUnset: u32 = 15;
const OP_HSD_MObjSetToonTextureImage: u32 = 16;
const OP_HSD_MObjSetDiffuseColor: u32 = 17;
const OP_HSD_MObjSetAlpha: u32 = 18;
const OP_HSD_MObjGetTObj: u32 = 19;
const OP_HSD_MObjRemove: u32 = 20;
const OP_HSD_MObjAlloc: u32 = 21;
const OP_HSD_MaterialAlloc: u32 = 22;
const OP_HSD_MObjAddShadowTexture: u32 = 23;
const OP_HSD_MObjDeleteShadowTexture: u32 = 24;
const OP_MObjRelease: u32 = 25;
const OP_MObjAmnesia: u32 = 26;
const OP_MObjInfoInit: u32 = 27;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::mobj::native as m;
use mtu_harness::*;
#[path = "mtu/mobj_gen.rs"]
mod g;


static mut TXA: [[u8; 64]; 4] = [[0; 64]; 4];
static mut TXP: [*mut HSD_TExp; 3] = [null_mut(); 3];
static mut HP: [u8; 2048] = [0; 2048];
static mut HPI: usize = 0;
unsafe extern "C" fn cb_setup(m: *mut HSD_MObj, r: u32) { t(8200); t(pi(m)); t(r); }
unsafe extern "C" fn cb_unset(m: *mut HSD_MObj, r: u32) { t(8201); t(pi(m)); t(r); }
unsafe extern "C" fn cb_rel(c: *mut HSD_Class) { t(8203); t(pi(c)); }
unsafe extern "C" fn cb_dest(c: *mut HSD_Class) { t(8204); t(pi(c)); }
unsafe extern "C" fn cb_amn(c: *mut HSD_ClassInfo) { t(8205); t(pi(c)); }
unsafe extern "C" fn cb_tmake(tb: *mut HSD_TObj, a: u32, b: u32, x: *mut *mut HSD_TExp, y: *mut *mut HSD_TExp, l: *mut *mut HSD_TExp) { t(8300); t(pi(tb)); t(a); t(b); t(pi(x)); t(pi(y)); t(pi(l)); }


pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub static mut MOB: [HSD_MObj; 3] = unsafe { core::mem::zeroed() };
pub static mut TOB: [HSD_TObj; 6] = unsafe { core::mem::zeroed() };
pub static mut MAT: [HSD_Material; 2] = unsafe { core::mem::zeroed() };
pub static mut PED: [HSD_PEDesc; 2] = unsafe { core::mem::zeroed() };
pub static mut AOB: [HSD_AObj; 2] = unsafe { core::mem::zeroed() };
pub static mut MAN: [HSD_MatAnim; 2] = unsafe { core::mem::zeroed() };
pub static mut MDS: [HSD_MObjDesc; 2] = unsafe { core::mem::zeroed() };
pub static mut CIN: [HSD_ClassInfo; 2] = unsafe { core::mem::zeroed() };
pub static mut MINF: [HSD_MObjInfo; 2] = unsafe { core::mem::zeroed() };
pub static mut TINF: [HSD_TObjInfo; 1] = unsafe { core::mem::zeroed() };
pub static mut IMD: [HSD_ImageDesc; 2] = unsafe { core::mem::zeroed() };
pub static mut TDS: [HSD_TObjDesc; 2] = unsafe { core::mem::zeroed() };
pub fn pick_HSD_MObj() -> *mut HSD_MObj { let r = rnd(); unsafe { &raw mut MOB[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_MObj() -> *mut HSD_MObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut MOB[((r >> 8) % 3) as usize] } } }
pub fn pick_HSD_TObj() -> *mut HSD_TObj { let r = rnd(); unsafe { &raw mut TOB[((r >> 8) % 6) as usize] } }
pub fn picko_HSD_TObj() -> *mut HSD_TObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut TOB[((r >> 8) % 6) as usize] } } }
pub fn pick_HSD_Material() -> *mut HSD_Material { let r = rnd(); unsafe { &raw mut MAT[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_Material() -> *mut HSD_Material { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut MAT[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_PEDesc() -> *mut HSD_PEDesc { let r = rnd(); unsafe { &raw mut PED[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_PEDesc() -> *mut HSD_PEDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut PED[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_AObj() -> *mut HSD_AObj { let r = rnd(); unsafe { &raw mut AOB[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_AObj() -> *mut HSD_AObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut AOB[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_MatAnim() -> *mut HSD_MatAnim { let r = rnd(); unsafe { &raw mut MAN[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_MatAnim() -> *mut HSD_MatAnim { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut MAN[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_MObjDesc() -> *mut HSD_MObjDesc { let r = rnd(); unsafe { &raw mut MDS[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_MObjDesc() -> *mut HSD_MObjDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut MDS[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_ClassInfo() -> *mut HSD_ClassInfo { let r = rnd(); unsafe { &raw mut CIN[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_ClassInfo() -> *mut HSD_ClassInfo { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut CIN[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_MObjInfo() -> *mut HSD_MObjInfo { let r = rnd(); unsafe { &raw mut MINF[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_MObjInfo() -> *mut HSD_MObjInfo { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut MINF[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_TObjInfo() -> *mut HSD_TObjInfo { let r = rnd(); unsafe { &raw mut TINF[((r >> 8) % 1) as usize] } }
pub fn picko_HSD_TObjInfo() -> *mut HSD_TObjInfo { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut TINF[((r >> 8) % 1) as usize] } } }
pub fn pick_HSD_ImageDesc() -> *mut HSD_ImageDesc { let r = rnd(); unsafe { &raw mut IMD[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_ImageDesc() -> *mut HSD_ImageDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut IMD[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_TObjDesc() -> *mut HSD_TObjDesc { let r = rnd(); unsafe { &raw mut TDS[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_TObjDesc() -> *mut HSD_TObjDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut TDS[((r >> 8) % 2) as usize] } } }
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
        preg(&raw const MOB, core::mem::size_of_val(&MOB));
        preg(&raw const TOB, core::mem::size_of_val(&TOB));
        preg(&raw const MAT, core::mem::size_of_val(&MAT));
        preg(&raw const PED, core::mem::size_of_val(&PED));
        preg(&raw const AOB, core::mem::size_of_val(&AOB));
        preg(&raw const MAN, core::mem::size_of_val(&MAN));
        preg(&raw const MDS, core::mem::size_of_val(&MDS));
        preg(&raw const CIN, core::mem::size_of_val(&CIN));
        preg(&raw const MINF, core::mem::size_of_val(&MINF));
        preg(&raw const TINF, core::mem::size_of_val(&TINF));
        preg(&raw const IMD, core::mem::size_of_val(&IMD));
        preg(&raw const TDS, core::mem::size_of_val(&TDS));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS));
    }
}

pub unsafe fn setup_world() {
    melee_runtime::tu::mobj::native::ext::use_ext_data = true;
    MOB = core::mem::zeroed();
    TOB = core::mem::zeroed();
    MAT = core::mem::zeroed();
    PED = core::mem::zeroed();
    AOB = core::mem::zeroed();
    MAN = core::mem::zeroed();
    MDS = core::mem::zeroed();
    CIN = core::mem::zeroed();
    MINF = core::mem::zeroed();
    TINF = core::mem::zeroed();
    IMD = core::mem::zeroed();
    TDS = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..3 { g::fill_HSD_MObj(&mut MOB[i]); }
    for i in 0..6 { g::fill_HSD_TObj(&mut TOB[i]); }
    for i in 0..2 { g::fill_HSD_Material(&mut MAT[i]); }
    for i in 0..2 { g::fill_HSD_PEDesc(&mut PED[i]); }
    for i in 0..2 { g::fill_HSD_AObj(&mut AOB[i]); }
    for i in 0..2 { g::fill_HSD_MatAnim(&mut MAN[i]); }
    for i in 0..2 { g::fill_HSD_MObjDesc(&mut MDS[i]); }
    for i in 0..2 { g::fill_HSD_ClassInfo(&mut CIN[i]); }
    for i in 0..2 { g::fill_HSD_MObjInfo(&mut MINF[i]); }
    for i in 0..1 { g::fill_HSD_TObjInfo(&mut TINF[i]); }
    for i in 0..2 { g::fill_HSD_ImageDesc(&mut IMD[i]); }
    for i in 0..2 { g::fill_HSD_TObjDesc(&mut TDS[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..3 { g::hash_HSD_MObj(&MOB[i]); }
    for i in 0..6 { g::hash_HSD_TObj(&TOB[i]); }
    for i in 0..2 { g::hash_HSD_Material(&MAT[i]); }
    for i in 0..2 { g::hash_HSD_PEDesc(&PED[i]); }
    for i in 0..2 { g::hash_HSD_AObj(&AOB[i]); }
    for i in 0..2 { g::hash_HSD_MatAnim(&MAN[i]); }
    for i in 0..2 { g::hash_HSD_MObjDesc(&MDS[i]); }
    for i in 0..2 { g::hash_HSD_ClassInfo(&CIN[i]); }
    for i in 0..2 { g::hash_HSD_MObjInfo(&MINF[i]); }
    for i in 0..1 { g::hash_HSD_TObjInfo(&TINF[i]); }
    for i in 0..2 { g::hash_HSD_ImageDesc(&IMD[i]); }
    for i in 0..2 { g::hash_HSD_TObjDesc(&TDS[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    g::hashg_all();
    g::hasho_all();
}

// harness = false (Cargo.toml): a case that ends in a panic is cut off inside the hook (see mtu_harness::run_case),
// which a libtest main thread cannot survive at exit.
fn main() { mobj_matches_c(); println!("tu_mobj: ok"); }

fn mobj_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();
preg(&raw const TXA, 256); preg(&raw const TXP, core::mem::size_of_val(&TXP)); preg(&raw const HP, 2048); freg(cb_setup as usize); freg(cb_unset as usize); freg(cb_rel as usize); freg(cb_dest as usize); freg(cb_amn as usize); freg(cb_tmake as usize); melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());
        install_abort_hook();
        let mut c = Check::new(include_str!("data/mtu_mobj.txt"));
        let mut total = 0u32;
        for op in 0..28u32 {
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
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
{ HPI = 0; for q in 0..3usize { MOB[q].parent.class_info = (&raw mut MINF[q & 1]) as *mut HSD_ClassInfo; MOB[q].tobj = if fv() & 3 != 0 { &raw mut TOB[(fv() & 1) as usize] } else { null_mut() }; MOB[q].mat = &raw mut MAT[(fv() & 1) as usize]; MOB[q].pe = if fv() & 1 != 0 { &raw mut PED[(fv() & 1) as usize] } else { null_mut() }; MOB[q].aobj = if fv() & 3 != 0 { &raw mut AOB[(fv() & 1) as usize] } else { null_mut() }; MOB[q].tevdesc = if fv() & 1 != 0 { (&raw mut TXA[3]) as *mut _ } else { null_mut() }; MOB[q].texp = if fv() & 1 != 0 { (&raw mut TXA[2]) as *mut HSD_TExp } else { null_mut() }; } for q in 0..6usize { TOB[q].next = if q == 0 || q == 1 || q == 3 { &raw mut TOB[q + 1] } else { null_mut() }; TOB[q].parent.parent.class_info = (&raw mut TINF[0]) as *mut HSD_ClassInfo; } for q in 0..2usize { MINF[q].load = Some(m::MObjLoad); MINF[q].make_texp = Some(m::MObjMakeTExp); MINF[q].setup_tev = Some(m::MObjSetupTev); MINF[q].setup = Some(core::mem::transmute::<usize, unsafe extern "C" fn(*mut HSD_MObj, u32)>(cb_setup as usize)); MINF[q].unset = Some(core::mem::transmute::<usize, unsafe extern "C" fn(*mut HSD_MObj, u32)>(cb_unset as usize)); MINF[q].parent.release = Some(cb_rel); MINF[q].parent.destroy = Some(cb_dest); } TINF[0].make_texp = Some(core::mem::transmute::<usize, _>(cb_tmake as usize)); for q in 0..2usize { MDS[q].class_name = if fv() & 1 != 0 { (&raw mut STRS[0]) as *mut i8 } else { null_mut() }; MDS[q].mat = &raw mut MAT[0]; MDS[q].pedesc = if fv() & 1 != 0 { &raw mut PED[0] } else { null_mut() }; } m::hsdMObj.parent.head.parent = &raw mut CIN[0]; CIN[0].release = Some(cb_rel); CIN[0].amnesia = Some(cb_amn); m::default_class = if fv() & 1 != 0 { &raw mut CIN[1] } else { null_mut() }; m::current_mobj = null_mut(); m::tobj_toon = if fv() & 1 != 0 { &raw mut TOB[5] } else { null_mut() }; m::tobj_shadows = if fv() & 1 != 0 { &raw mut TOB[3] } else { null_mut() }; for q in 0..3usize { TXP[q] = null_mut(); } }
                let mut ret = 0u32;
                let aborted = run_case(|| unsafe { match op {
                    0 => { 
let a0 = pick_HSD_MObj();
m::HSD_MObjSetCurrent(a0);
 }
                    1 => { 
let a0 = pick_HSD_MObj();
let a1 = fv() as _;
m::HSD_MObjSetFlags(a0, a1);
 }
                    2 => { 
let a0 = pick_HSD_MObj();
let a1 = fv() as _;
m::HSD_MObjClearFlags(a0, a1);
 }
                    3 => { 
let a0 = pick_HSD_MObj();
let a1 = fv() as _;
m::HSD_MObjRemoveAnimByFlags(a0, a1);
 }
                    4 => { 
let a0 = pick_HSD_MObj();
let a1 = pick_HSD_MatAnim();
m::HSD_MObjAddAnim(a0, a1);
 }
                    5 => { 
let a0 = pick_HSD_MObj();
let a1 = rw();
let a2 = fv() as _;
m::HSD_MObjReqAnimByFlags(a0, a1, a2);
 }
                    6 => { 
let a0 = pick_HSD_MObj();
let a1 = rw();
m::HSD_MObjReqAnim(a0, a1);
 }
                    7 => { 
let a0 = picko_HSD_MObj() as *mut c_void;
let a1 = (fv() % 20) as i32;
let a2 = (&raw mut BUF) as *mut _;
m::MObjUpdateFunc(a0, a1, a2);
 }
                    8 => { 
let a0 = pick_HSD_MObj();
m::HSD_MObjAnim(a0);
 }
                    9 => { 
let a0 = pick_HSD_MObj();
let a1 = pick_HSD_MObjDesc();
ret = m::MObjLoad(a0, a1) as u32;
 }
                    10 => { 
let a0 = pick_HSD_MObjDesc();
ret = pi(m::HSD_MObjLoadDesc(a0));
 }
                    11 => { 
let a0 = pick_HSD_MObj();
let a1 = pick_HSD_TObj();
let a2 = &raw mut TXP[(fv() % 3) as usize];
ret = pi(m::MObjMakeTExp(a0, a1, a2));
 }
                    12 => { 
let a0 = pick_HSD_MObj();
m::HSD_MObjCompileTev(a0);
 }
                    13 => { 
let a0 = pick_HSD_MObj();
let a1 = pick_HSD_TObj();
let a2 = fv() as _;
m::MObjSetupTev(a0, a1, a2);
 }
                    14 => { 
let a0 = pick_HSD_MObj();
let a1 = fv() as _;
m::HSD_MObjSetup(a0, a1);
 }
                    15 => { 
let a0 = pick_HSD_MObj();
let a1 = fv() as _;
m::HSD_MObjUnset(a0, a1);
 }
                    16 => { 
let a0 = &raw mut IMD[(fv() & 1) as usize];
m::HSD_MObjSetToonTextureImage(a0);
 }
                    17 => { 
let a0 = pick_HSD_MObj();
let a1 = fv() as _;
let a2 = fv() as _;
let a3 = fv() as _;
m::HSD_MObjSetDiffuseColor(a0, a1, a2, a3);
 }
                    18 => { 
let a0 = pick_HSD_MObj();
let a1 = rw();
m::HSD_MObjSetAlpha(a0, a1);
 }
                    19 => { 
let a0 = pick_HSD_MObj();
ret = pi(m::HSD_MObjGetTObj(a0));
 }
                    20 => { 
let a0 = pick_HSD_MObj();
m::HSD_MObjRemove(a0);
 }
                    21 => { 

ret = pi(m::HSD_MObjAlloc());
 }
                    22 => { 

ret = pi(m::HSD_MaterialAlloc());
 }
                    23 => { 
let a0 = pick_HSD_TObj();
m::HSD_MObjAddShadowTexture(a0);
 }
                    24 => { 
let a0 = pick_HSD_TObj();
m::HSD_MObjDeleteShadowTexture(a0);
 }
                    25 => { 
let a0 = (&raw mut BUF) as *mut _;
m::MObjRelease(a0);
 }
                    26 => { 
let a0 = pick_HSD_ClassInfo();
m::MObjAmnesia(a0);
 }
                    27 => { 

m::MObjInfoInit();
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
