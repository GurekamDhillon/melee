//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/initialize.c.
//! src/tu/initialize.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py initialize`).
//! C functions of the TU this test covers (the private ones through their public callers): HSD_AllocateFifo, HSD_AllocateXFB, HSD_CreateMainHeap, HSD_DVDInit, HSD_GXInit, HSD_GXSetFifoObj, HSD_GetCurrentRenderPass, HSD_GetHeap, HSD_GetNextArena, HSD_InitComponent, HSD_Init_803755A8, HSD_OSInit, HSD_ObjDumpStat, HSD_ObjInit, HSD_SetHeap, HSD_SetInitParameterPtr, HSD_SetInitParameterU32, HSD_StartRender, hsd_init_parameter_allowed.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_HSD_InitComponent: u32 = 0;
const OP_HSD_GXSetFifoObj: u32 = 1;
const OP_HSD_DVDInit: u32 = 2;
const OP_HSD_AllocateXFB: u32 = 3;
const OP_HSD_AllocateFifo: u32 = 4;
const OP_HSD_GXInit: u32 = 5;
const OP_HSD_OSInit: u32 = 6;
const OP_HSD_GetHeap: u32 = 7;
const OP_HSD_SetHeap: u32 = 8;
const OP_HSD_GetNextArena: u32 = 9;
const OP_HSD_CreateMainHeap: u32 = 10;
const OP_HSD_GetCurrentRenderPass: u32 = 11;
const OP_HSD_StartRender: u32 = 12;
const OP_HSD_Init_803755A8: u32 = 13;
const OP_HSD_ObjInit: u32 = 14;
const OP_HSD_ObjDumpStat: u32 = 15;
const OP_HSD_SetInitParameterU32: u32 = 16;
const OP_HSD_SetInitParameterPtr: u32 = 17;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::initialize as m;
use mtu_harness::*;
#[path = "mtu/initialize_gen.rs"]
mod g;


#[repr(align(64))] struct Al([u8; 4096]);
static mut ARN: Al = Al([0; 4096]);
static mut AZ: bool = false; static mut LOK: usize = 0; static mut HIK: usize = 0;


pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut PPA: [*mut c_void; 4] = [core::ptr::null_mut(); 4];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub static mut RMO: [GXRenderModeObj; 2] = unsafe { core::mem::zeroed() };
pub static mut OAD: [HSD_ObjAllocData; 2] = unsafe { core::mem::zeroed() };
pub fn pick_GXRenderModeObj() -> *mut GXRenderModeObj { let r = rnd(); unsafe { &raw mut RMO[((r >> 8) % 2) as usize] } }
pub fn picko_GXRenderModeObj() -> *mut GXRenderModeObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut RMO[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_ObjAllocData() -> *mut HSD_ObjAllocData { let r = rnd(); unsafe { &raw mut OAD[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_ObjAllocData() -> *mut HSD_ObjAllocData { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut OAD[((r >> 8) % 2) as usize] } } }
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
        preg(&raw const RMO, core::mem::size_of_val(&RMO));
        preg(&raw const OAD, core::mem::size_of_val(&OAD));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS)); preg(&raw const PPA, core::mem::size_of_val(&PPA));
    }
}

pub unsafe fn setup_world() {
    RMO = core::mem::zeroed();
    OAD = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..4 { PPA[i] = core::ptr::null_mut(); }
    for i in 0..2 { g::fill_GXRenderModeObj(&mut RMO[i]); }
    for i in 0..2 { g::fill_HSD_ObjAllocData(&mut OAD[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..2 { g::hash_GXRenderModeObj(&RMO[i]); }
    for i in 0..2 { g::hash_HSD_ObjAllocData(&OAD[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    for i in 0..4 { hm(pi(PPA[i])); }
    g::hashg_all();
    g::hasho_all();
}

// harness = false (Cargo.toml): a case that ends in a panic is cut off inside the hook (see mtu_harness::run_case),
// which a libtest main thread cannot survive at exit.
fn main() { initialize_matches_c(); println!("tu_initialize: ok"); }

fn initialize_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();
m::use_ext_data = true; preg(&raw const ARN, 4096); preg(&raw const m::ext::GXNtsc480IntDf, core::mem::size_of::<GXRenderModeObj>()); melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());
        install_abort_hook();
        let mut c = Check::new(include_str!("data/mtu_initialize.txt"));
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
{ AZ = fv() & 3 == 0; LOK = (fv() % 16) as usize; HIK = if fv() & 3 == 0 { LOK } else { 32 + (fv() % 16) as usize }; m::memReport = core::mem::zeroed(); for q in 0..3usize { m::FrameBuffer[q] = null_mut(); }
 for q in 0..2usize { RMO[q].fbWidth = (fv() % 700) as u16; RMO[q].xfbHeight = (fv() % 500) as u16; RMO[q].viHeight = (fv() % 500) as u16; RMO[q].aa = (fv() & 1) as u8; RMO[q].field_rendering = (fv() & 1) as u8; }
 m::hsd_heap_next_arena_lo = if !AZ { (&raw mut ARN as *mut u8).add(32 * (fv() % 16) as usize) as *mut c_void } else { null_mut() }; m::hsd_heap_next_arena_hi = if !AZ { (&raw mut ARN as *mut u8).add(1024 + 32 * (fv() % 16) as usize) as *mut c_void } else { null_mut() };
 m::current_render_pass = fv() % 4; m::DefaultFifoObj = if fv() & 1 != 0 { (&raw mut BUF) as *mut GXFifoObj } else { null_mut() }; m::current_pix_fmt = (fv() % 8) as i32; m::init_done = (fv() & 1) as i32; m::shown = (fv() & 1) as i32; m::current_heap = fv() as i32 - 1;
 m::rmode = if fv() & 1 != 0 { &raw mut RMO[(fv() % 2) as usize] } else { &raw mut m::ext::GXNtsc480IntDf }; m::current_z_fmt = (fv() % 3) as i32; m::iparam_fifo_size = fv(); m::iparam_xfb_max_num = fv() as i32; m::iparam_heap_max_num = fv() as i32 % 8; m::iparam_audio_heap_size = 32 * (fv() % 8);
}
                let mut ret = 0u32;
                let aborted = run_case(|| unsafe { match op {
                    0 => { 

m::HSD_InitComponent();
 }
                    1 => { 
let a0 = (&raw mut BUF) as *mut _;
m::HSD_GXSetFifoObj(a0);
 }
                    2 => { 

m::HSD_DVDInit();
 }
                    3 => { 
let a0 = (fv() % 4) as i32;
let a1 = pick_GXRenderModeObj();
ret = pi(m::HSD_AllocateXFB(a0, a1));
 }
                    4 => { 
let a0 = fv() % 3000;
ret = pi(m::HSD_AllocateFifo(a0));
 }
                    5 => { 

m::HSD_GXInit();
 }
                    6 => { 

m::HSD_OSInit();
 }
                    7 => { 

ret = m::HSD_GetHeap() as u32;
 }
                    8 => { 
let a0 = fv() as _;
m::HSD_SetHeap(a0);
 }
                    9 => { 
let a0 = (&raw mut PPA[(fv() % 4) as usize]) as *mut _;
let a1 = (&raw mut PPA[(fv() % 4) as usize]) as *mut _;
m::HSD_GetNextArena(a0, a1);
 }
                    10 => { 
let a0 = if !AZ && fv() & 1 != 0 { (&raw mut ARN as *mut u8).add(32 * (fv() % 16) as usize) as *mut c_void } else { null_mut() };
let a1 = if !AZ && fv() & 1 != 0 { (&raw mut ARN as *mut u8).add(1024 + 32 * (fv() % 16) as usize) as *mut c_void } else { null_mut() };
ret = m::HSD_CreateMainHeap(a0, a1) as u32;
 }
                    11 => { 

ret = m::HSD_GetCurrentRenderPass() as u32;
 }
                    12 => { 
let a0 = fv() % 4;
m::HSD_StartRender(a0);
 }
                    13 => { 

m::HSD_Init_803755A8();
 }
                    14 => { 

m::HSD_ObjInit();
 }
                    15 => { 

m::HSD_ObjDumpStat();
 }
                    16 => { 
let a0 = (fv() % 6) as _;
let a1 = fv() as _;
ret = m::HSD_SetInitParameterU32(a0, a1) as u32;
 }
                    17 => { 
let a0 = fv() as _;
let a1 = (&raw mut BUF) as *mut c_void;
ret = m::HSD_SetInitParameterPtr(a0, a1) as u32;
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
