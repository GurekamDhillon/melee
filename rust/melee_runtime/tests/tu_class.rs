//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/class.c.
//! src/tu/class.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py class`).
//! C functions of the TU this test covers (the private ones through their public callers): ClassInfoInit, DumpClassStat, ForgetClassLibraryChild, ForgetClassLibraryReal, GetMemoryEntry, HSD_GetClassInfo, HSD_PushClassInfo, OSReport_PrintSpaces, _hsdClassAlloc, _hsdClassAmnesia, _hsdClassDestroy, _hsdClassInfoInit, _hsdClassInit, _hsdClassRelease, class_set_flags, hsdAllocMemPiece, hsdChangeClass, hsdChangeClass_inline, hsdDumpClassStat, hsdForgetClassLibrary, hsdFreeMemPiece, hsdInitClassInfo, hsdIsDescendantOf, hsdNew, hsdObjIsDescendantOf, hsdSearchClassInfo.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_ClassInfoInit: u32 = 0;
const OP_hsdInitClassInfo: u32 = 1;
const OP_OSReport_PrintSpaces: u32 = 2;
const OP_GetMemoryEntry: u32 = 3;
const OP_hsdAllocMemPiece: u32 = 4;
const OP_hsdFreeMemPiece: u32 = 5;
const OP__hsdClassAlloc: u32 = 6;
const OP__hsdClassInit: u32 = 7;
const OP__hsdClassRelease: u32 = 8;
const OP__hsdClassDestroy: u32 = 9;
const OP__hsdClassAmnesia: u32 = 10;
const OP__hsdClassInfoInit: u32 = 11;
const OP_hsdNew: u32 = 12;
const OP_hsdChangeClass: u32 = 13;
const OP_hsdIsDescendantOf: u32 = 14;
const OP_hsdObjIsDescendantOf: u32 = 15;
const OP_class_set_flags: u32 = 16;
const OP_ForgetClassLibraryReal: u32 = 17;
const OP_ForgetClassLibraryChild: u32 = 18;
const OP_hsdForgetClassLibrary: u32 = 19;
const OP_hsdSearchClassInfo: u32 = 20;
const OP_DumpClassStat: u32 = 21;
const OP_hsdDumpClassStat: u32 = 22;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::class as m;
use mtu_harness::*;
#[path = "mtu/class_gen.rs"]
mod g;


static mut ARENA: [u8; 65536] = [0; 65536]; static mut ARENA_P: usize = 0; static mut FAIL1: bool = false; static mut PK: usize = 0;
unsafe fn pnext() -> *mut c_void { let p = (&raw mut PIECES as *mut u8).add(256 * (PK & 15)); PK += 1; p as *mut c_void }
static mut PIECES: [u8; 8192] = [0; 8192];
unsafe fn arena_alloc(n: i32) -> *mut c_void { if n <= 0 || ARENA_P + ((n as usize + 7) & !7) > 65536 { return null_mut(); } let p = (&raw mut ARENA as *mut u8).add(ARENA_P); ARENA_P += (n as usize + 7) & !7; p as *mut c_void }
unsafe extern "C" fn cb_amn(c: *mut HSD_ClassInfo) { t(8201); t(pi(c)); }
unsafe extern "C" fn cb_init(c: *mut HSD_Class) -> i32 { t(8202); t(pi(c)); (fv() % 3) as i32 - 1 }
unsafe extern "C" fn cb_rel(c: *mut HSD_Class) { t(8203); t(pi(c)); }
unsafe extern "C" fn cb_dest(c: *mut HSD_Class) { t(8204); t(pi(c)); }
unsafe extern "C" fn cb_alloc(c: *mut HSD_ClassInfo) -> *mut HSD_Class { t(8205); t(pi(c)); if fv() % 5 != 0 { arena_alloc((*c).head.obj_size as i32) as *mut HSD_Class } else { null_mut() } }
unsafe extern "C" fn ii0() { t(8210); CI[0].head.flags |= 1; }
unsafe extern "C" fn ii1() { t(8211); CI[1].head.flags |= 1; }
unsafe extern "C" fn ii2() { t(8212); CI[2].head.flags |= 1; }
unsafe extern "C" fn ii3() { t(8213); CI[3].head.flags |= 1; }


pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut PPA: [*mut c_void; 4] = [core::ptr::null_mut(); 4];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub static mut CI: [HSD_ClassInfo; 4] = unsafe { core::mem::zeroed() };
pub static mut CL: [HSD_Class; 3] = unsafe { core::mem::zeroed() };
pub fn pick_HSD_ClassInfo() -> *mut HSD_ClassInfo { let r = rnd(); unsafe { &raw mut CI[((r >> 8) % 4) as usize] } }
pub fn picko_HSD_ClassInfo() -> *mut HSD_ClassInfo { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut CI[((r >> 8) % 4) as usize] } } }
pub fn pick_HSD_Class() -> *mut HSD_Class { let r = rnd(); unsafe { &raw mut CL[((r >> 8) % 3) as usize] } }
pub fn picko_HSD_Class() -> *mut HSD_Class { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut CL[((r >> 8) % 3) as usize] } } }
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
        preg(&raw const CI, core::mem::size_of_val(&CI));
        preg(&raw const CL, core::mem::size_of_val(&CL));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS)); preg(&raw const PPA, core::mem::size_of_val(&PPA));
    }
}

pub unsafe fn setup_world() {
    CI = core::mem::zeroed();
    CL = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..4 { PPA[i] = core::ptr::null_mut(); }
    for i in 0..4 { g::fill_HSD_ClassInfo(&mut CI[i]); }
    for i in 0..3 { g::fill_HSD_Class(&mut CL[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..4 { g::hash_HSD_ClassInfo(&CI[i]); }
    for i in 0..3 { g::hash_HSD_Class(&CL[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    for i in 0..4 { hm(pi(PPA[i])); }
    g::hashg_all();
    g::hasho_all();
}

// harness = false (Cargo.toml): a case that ends in a panic is cut off inside the hook (see mtu_harness::run_case),
// which a libtest main thread cannot survive at exit.
fn main() { class_matches_c(); println!("tu_class: ok"); }

fn class_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();
preg(&raw const ARENA, 65536); preg(&raw const PIECES, 8192); freg(cb_amn as usize); freg(cb_init as usize); freg(cb_rel as usize); freg(cb_dest as usize); freg(cb_alloc as usize); freg(ii0 as usize); freg(ii1 as usize); freg(ii2 as usize); freg(ii3 as usize); melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());
        install_abort_hook();
        let mut c = Check::new(include_str!("data/mtu_class.txt"));
        let mut total = 0u32;
        for op in 0..23u32 {
            let n = match op {
                0 => 400u32,
                1 => 400u32,
                2 => 400u32,
                3 => 400u32,
                4 => 400u32,
                5 => 400u32,
                6 => 400u32,
                7 => 400u32,
                8 => 400u32,
                9 => 400u32,
                10 => 400u32,
                11 => 400u32,
                12 => 400u32,
                13 => 400u32,
                14 => 400u32,
                15 => 400u32,
                16 => 400u32,
                17 => 400u32,
                18 => 400u32,
                19 => 400u32,
                20 => 400u32,
                21 => 400u32,
                22 => 400u32,
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
{ FAIL1 = false; PK = 0; ARENA = [0; 65536]; ARENA_P = 0; PIECES = [0; 8192];
 for q in 0..4usize { let c = &raw mut CI[q];
  (*c).head.info_init = Some(match q { 0 => ii0 as unsafe extern "C" fn(), 1 => ii1, 2 => ii2, _ => ii3 }); (*c).head.flags = fv() & 1;
  (*c).head.library_name = (&raw mut STRS[(fv() % 4) as usize]) as *mut i8; (*c).head.class_name = (&raw mut STRS[(fv() % 4) as usize]) as *mut i8;
  (*c).head.obj_size = (8 + 8 * (fv() % 8)) as i16; (*c).head.info_size = (core::mem::size_of::<_HSD_ClassInfoHead>() as u32 + 8 * (fv() % 6)) as i16;
  (*c).head.parent = if q > 0 && fv() & 1 != 0 { &raw mut CI[(fv() % q as u32) as usize] } else { null_mut() };
  (*c).head.child = if q < 3 && fv() & 1 != 0 { &raw mut CI[q + 1 + (fv() % (3 - q) as u32) as usize] } else { null_mut() };
  (*c).head.next = if q < 3 && fv() & 1 != 0 { &raw mut CI[q + 1 + (fv() % (3 - q) as u32) as usize] } else { null_mut() };
  (*c).head.nb_exist = fv() % 50; (*c).head.nb_peak = (*c).head.nb_exist + fv() % 20;
  (*c).alloc = if q & 1 != 0 { Some(m::_hsdClassAlloc) } else { Some(cb_alloc) }; (*c).init = if fv() & 1 != 0 { Some(cb_init) } else { Some(m::_hsdClassInit) }; (*c).release = Some(cb_rel); (*c).destroy = if fv() & 1 != 0 { Some(cb_dest) } else { Some(m::_hsdClassDestroy) };
  (*c).amnesia = if fv() & 1 != 0 { Some(cb_amn) } else { Some(m::_hsdClassAmnesia) }; }
 for q in 0..3usize { CL[q].class_info = &raw mut CI[(fv() % 4) as usize]; }
 m::hsdClass.head.info_init = Some(m::_hsdClassInfoInit); m::hsdClass.head.flags = fv() & 1; m::hsdClass.head.library_name = (&raw mut STRS[(fv() % 4) as usize]) as *mut i8; m::hsdClass.head.class_name = (&raw mut STRS[(fv() % 4) as usize]) as *mut i8;
 m::hsdClass.head.obj_size = (8 + 8 * (fv() % 8)) as i16; m::hsdClass.head.info_size = core::mem::size_of::<_HSD_ClassInfoHead>() as i16; m::hsdClass.head.parent = null_mut();
 m::hsdClass.head.child = if fv() & 1 != 0 { &raw mut CI[(fv() % 4) as usize] } else { null_mut() }; m::hsdClass.head.next = null_mut(); m::hsdClass.head.nb_exist = fv() % 50; m::hsdClass.head.nb_peak = fv() % 70;
 m::hsdClass.alloc = Some(m::_hsdClassAlloc); m::hsdClass.init = Some(m::_hsdClassInit); m::hsdClass.release = Some(m::_hsdClassRelease); m::hsdClass.destroy = Some(m::_hsdClassDestroy); m::hsdClass.amnesia = Some(m::_hsdClassAmnesia);
 m::memory_list = null_mut(); m::nb_memory_list = 0; m::current_hash = if fv() & 1 != 0 { (&raw mut BUF) as *mut HSD_Hash } else { null_mut() };
 if fv() & 1 != 0 { let mut w = fv() % 5; while w > 0 { w -= 1; m::GetMemoryEntry((fv() % 100) as i32); } w = fv() % 3; while w > 0 { w -= 1; m::hsdFreeMemPiece(pnext(), (1 + fv() % 200) as i32); } }
 FAIL1 = (op == OP_GetMemoryEntry || op == OP_hsdAllocMemPiece || op == OP__hsdClassAlloc) && (fv() & 3) == 0;
}
                let mut ret = 0u32;
                let aborted = run_case(|| unsafe { match op {
                    0 => { 
let a0 = pick_HSD_ClassInfo();
m::ClassInfoInit(a0);
 }
                    1 => { 
let a0 = pick_HSD_ClassInfo();
let a1 = picko_HSD_ClassInfo();
let a2 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
let a3 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
let a4 = (core::mem::size_of::<_HSD_ClassInfoHead>() as u32 + 8 * (fv() % 8)) as i32;
let a5 = (8 + 8 * (fv() % 10)) as i32;
m::hsdInitClassInfo(a0, a1, a2, a3, a4, a5);
 }
                    2 => { 
let a0 = (fv() % 5) as i32 - 1;
m::OSReport_PrintSpaces(a0);
 }
                    3 => { 
let a0 = if fv() % 32 == 0 { -1 } else { (fv() % 140) as i32 };
ret = pi(m::GetMemoryEntry(a0));
 }
                    4 => { 
let a0 = (1 + fv() % 1500) as i32;
ret = pi(m::hsdAllocMemPiece(a0));
 }
                    5 => { 
let a0 = if fv() & 7 != 0 { pnext() } else { null_mut() };
let a1 = (1 + fv() % 200) as i32;
m::hsdFreeMemPiece(a0, a1);
 }
                    6 => { 
let a0 = pick_HSD_ClassInfo();
ret = pi(m::_hsdClassAlloc(a0));
 }
                    7 => { 
let a0 = pick_HSD_Class();
ret = m::_hsdClassInit(a0) as u32;
 }
                    8 => { 
let a0 = pick_HSD_Class();
m::_hsdClassRelease(a0);
 }
                    9 => { 
let a0 = pick_HSD_Class();
m::_hsdClassDestroy(a0);
 }
                    10 => { 
let a0 = pick_HSD_ClassInfo();
m::_hsdClassAmnesia(a0);
 }
                    11 => { 

m::_hsdClassInfoInit();
 }
                    12 => { 
let a0 = pick_HSD_ClassInfo();
ret = pi(m::hsdNew(a0));
 }
                    13 => { 
let a0 = pick_HSD_Class() as *mut c_void;
let a1 = pick_HSD_ClassInfo() as *mut c_void;
ret = m::hsdChangeClass(a0, a1) as u32;
 }
                    14 => { 
let a0 = picko_HSD_ClassInfo() as *mut c_void;
let a1 = picko_HSD_ClassInfo() as *mut c_void;
ret = m::hsdIsDescendantOf(a0, a1) as u32;
 }
                    15 => { 
let a0 = picko_HSD_Class() as *mut HSD_Obj;
let a1 = pick_HSD_ClassInfo();
ret = m::hsdObjIsDescendantOf(a0, a1) as u32;
 }
                    16 => { 
let a0 = pick_HSD_ClassInfo();
let a1 = (fv() & 3) as i32;
let a2 = (fv() & 3) as i32;
m::class_set_flags(a0, a1, a2);
 }
                    17 => { 
let a0 = pick_HSD_ClassInfo();
m::ForgetClassLibraryReal(a0);
 }
                    18 => { 
let a0 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
let a1 = pick_HSD_ClassInfo();
m::ForgetClassLibraryChild(a0, a1);
 }
                    19 => { 
let a0 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
m::hsdForgetClassLibrary(a0);
 }
                    20 => { 
let a0 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
ret = pi(m::hsdSearchClassInfo(a0));
 }
                    21 => { 
let a0 = pick_HSD_ClassInfo();
let a1 = (fv() % 4) as i32;
m::DumpClassStat(a0, a1);
 }
                    22 => { 
let a0 = pick_HSD_ClassInfo();
let a1 = fv() & 1 != 0;
let a2 = (fv() % 4) as i32;
m::hsdDumpClassStat(a0, a1, a2);
 }
                    _ => unreachable!(),
                } });
                if aborted { c.abort(op, k); total += 1; continue; }
                hash_world();
{ g::hash_HSD_ClassInfo(&m::hsdClass); hm(pi(m::memory_list)); hm(m::nb_memory_list as u32); hm(pi(m::current_hash));
 for i in 0..m::nb_memory_list as usize { let e = *m::memory_list.add(i); hm(pi(e)); if !e.is_null() { hm((*e).size); hm((*e).nb_alloc); hm((*e).nb_free); hm(pi((*e).next)); hm(pi((*e).free_list));
   let mut f = (*e).free_list; let mut n = 0; while !f.is_null() && n < 64 { hm(pi(f)); f = (*f).next; n += 1; } } } }
                c.line(op, k, ret);
                total += 1;
            }
        }
        assert_eq!(c.done(), total);
    }
}
