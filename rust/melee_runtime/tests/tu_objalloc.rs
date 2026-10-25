//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/objalloc.c.
//! src/tu/objalloc.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py objalloc`).
//! C functions of the TU this test covers (the private ones through their public callers): HSD_ObjAlloc, HSD_ObjAllocAddFree, HSD_ObjAllocInit, HSD_ObjAllocReport, HSD_ObjAllocTopUp, HSD_ObjFree, HSD_ObjSetHeap, _HSD_ObjAllocForgetMemory, removeAll, snap_note_render.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_snap_note_render: u32 = 0;
const OP_HSD_ObjAllocTopUp: u32 = 1;
const OP_HSD_ObjAllocReport: u32 = 2;
const OP_HSD_ObjAllocAddFree: u32 = 3;
const OP_HSD_ObjAlloc: u32 = 4;
const OP_HSD_ObjFree: u32 = 5;
const OP_removeAll: u32 = 6;
const OP_HSD_ObjAllocInit: u32 = 7;
const OP__HSD_ObjAllocForgetMemory: u32 = 8;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::objalloc as m;
use mtu_harness::*;
#[path = "mtu/objalloc_gen.rs"]
mod g;

static mut POOLMEM: [u8; 16384] = [0; 16384]; static mut LST: [[u8; 64]; 4] = [[0; 64]; 4]; unsafe fn hw(b: *const u8, n: usize) { let mut i = 0; while i < n { let v = (b.add(i) as *const u64).read_unaligned(); let p = pi(v as usize as *const u8); if p != 0xFFFF_FFFF { hm(p); } else { hm(v as u32); hm((v >> 32) as u32); } i += 8; } }

pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub static mut OD: [HSD_ObjAllocData; 4] = unsafe { core::mem::zeroed() };
pub fn pick_HSD_ObjAllocData() -> *mut HSD_ObjAllocData { let r = rnd(); unsafe { &raw mut OD[((r >> 8) % 4) as usize] } }
pub fn picko_HSD_ObjAllocData() -> *mut HSD_ObjAllocData { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut OD[((r >> 8) % 4) as usize] } } }
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
        preg(&raw const OD, core::mem::size_of_val(&OD));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS));
    }
}

pub unsafe fn setup_world() {
    OD = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..4 { g::fill_HSD_ObjAllocData(&mut OD[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..4 { g::hash_HSD_ObjAllocData(&OD[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    g::hashg_all();
    g::hasho_all();
}

// harness = false (Cargo.toml): a case that ends in a panic is cut off inside the hook (see mtu_harness::run_case),
// which a libtest main thread cannot survive at exit.
fn main() { objalloc_matches_c(); println!("tu_objalloc: ok"); }

fn objalloc_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();
preg(&raw const POOLMEM, 16384); preg(&raw const LST, 256); melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());
        install_abort_hook();
        let mut c = Check::new(include_str!("data/mtu_objalloc.txt"));
        let mut total = 0u32;
        for op in 0..9u32 {
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
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
{ m::alloc_datas = if fv() & 1 != 0 { &raw mut OD[(fv() % 4) as usize] } else { null_mut() }; m::obj_heap.top = 0; m::obj_heap.curr = 0; m::obj_heap.size = u32::MAX as _; m::obj_heap.remain = fv() as _; for i in 0..16384usize { POOLMEM[i] = (i as u32).wrapping_mul(31).wrapping_add(7) as u8; } for i in 0..256usize { LST[i >> 6][i & 63] = (i as u32).wrapping_mul(13).wrapping_add(1) as u8; } for q in 0..4usize { OD[q].size = 8 + 8 * (fv() % 4); OD[q].align = 7; OD[q].next = if q < 3 && fv() % 4 != 0 { &raw mut OD[q + 1] } else { null_mut() }; let k = (fv() % 3) as usize; for i in 0..k { *((&raw mut LST[q] as *mut u8).add(32 * i) as *mut *mut c_void) = if i + 1 < k { (&raw mut LST[q] as *mut u8).add(32 * (i + 1)) as *mut c_void } else { null_mut() }; } OD[q].freehead = if k != 0 { (&raw mut LST[q]) as *mut HSD_ObjAllocLink } else { null_mut() }; OD[q].free = k as u32; OD[q].used = fv() % 5; OD[q].peak = OD[q].used + fv() % 3; OD[q].num_limit = fv() % 8; OD[q].heap_limit_size = fv() % 1000; OD[q].heap_limit_num = if fv() & 1 != 0 { u32::MAX } else { fv() % 8 }; } }
                let mut ret = 0u32;
                let aborted = run_case(|| unsafe { match op {
                    0 => { 
let a0 = pick_HSD_ObjAllocData();
let a1 = fv() as _;
m::snap_note_render(a0, a1);
 }
                    1 => { 

m::HSD_ObjAllocTopUp();
 }
                    2 => { 

m::HSD_ObjAllocReport();
 }
                    3 => { 
let a0 = pick_HSD_ObjAllocData();
let a1 = 1 + fv() % 8;
ret = m::HSD_ObjAllocAddFree(a0, a1) as u32;
 }
                    4 => { 
let a0 = pick_HSD_ObjAllocData();
ret = pi(m::HSD_ObjAlloc(a0));
 }
                    5 => { 
let a0 = pick_HSD_ObjAllocData();
let a1 = (&raw mut LST as *mut u8).add(8 * (fv() % 32) as usize) as *mut c_void;
m::HSD_ObjFree(a0, a1);
 }
                    6 => { 
let a0 = pick_HSD_ObjAllocData();
m::removeAll(a0);
 }
                    7 => { 
let a0 = pick_HSD_ObjAllocData();
let a1 = (1 + fv() % 40) as usize;
let a2 = 8u32;
m::HSD_ObjAllocInit(a0, a1, a2);
 }
                    8 => { 
let a0 = (&raw mut BUF) as *mut c_void;
let a1 = (&raw mut BUF) as *mut c_void;
m::_HSD_ObjAllocForgetMemory(a0, a1);
 }
                    _ => unreachable!(),
                } });
                if aborted { c.abort(op, k); total += 1; continue; }
                hash_world();
{ hw(&raw const POOLMEM as *const u8, 16384); hw(&raw const LST as *const u8, 256); }
                c.line(op, k, ret);
                total += 1;
            }
        }
        assert_eq!(c.done(), total);
    }
}
