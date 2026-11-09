//! Written by gen/mtu/if_drv.py from the same description as gen/mtu/tu/ifstock.c.
//! src/tu/ifstock.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_world/gen/mtu/run_if.py ifstock`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_world::tu::ifstock as m;
use mtu_harness::*;
#[path = "mtu/ifstock_gen.rs"]
mod g;

pub static mut GCOL: [_GXColor; 2] = unsafe { core::mem::zeroed() };
pub static mut V3A: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
pub static mut ARCP: *mut HSD_Archive = null_mut();
pub static mut PMDL: [*mut DynamicModelDesc; 2] = [null_mut(); 2];
pub static mut SCA: [i8; 140] = [0; 140];
pub static mut SCB: [i8; 140] = [0; 140];
pub static mut SCC: [i8; 140] = [0; 140];
pub static mut SCD: [i8; 140] = [0; 140];
#[repr(C, align(64))]
pub struct Heap(pub [u8; 4096]);
pub static mut HEAP: Heap = Heap([0; 4096]);
pub static mut HOFF: i32 = 0;
pub static mut NULLJ: i32 = 0;
pub static mut STK: [i32; 8] = [0; 8];
pub static mut COIN: [i32; 8] = [0; 8];
pub static mut PIDT: [*mut HSD_ImageDesc; 16] = [null_mut(); 16];
pub static mut FDAT: [u8; 1024] = [0; 1024];
pub static mut FBUF: [u8; 64] = [0; 64];
pub static mut FPTR: *mut u8 = null_mut();
pub static mut MAXI: i32 = 0;
pub static mut OUTB: [i8; 4] = [0; 4];
fn halloc(n: i32) -> *mut c_void { unsafe { if n < 0 { return null_mut(); } let sz = n.wrapping_add(7) & !7; if HOFF.wrapping_add(sz) > 4096 { return null_mut(); } let r = &raw mut HEAP.0[HOFF as usize]; HOFF += sz; r as *mut c_void } }
pub static NM: [&[u8]; 8] = [b"Captain Falcon\0", b"mr game & watch\0", b"Ness\0", b"  --  \0", b"7up\0", b"a\0", b"\0", b"Dr. Mario 64\0"];




unsafe fn setup_extra() {
    let mut n: u32;
    let mut i: usize;
if HDBG { let pd = m::ifStock_804A1378.pad1F0; println!("PAD {:02X} {:02X} {:02X} {:02X} rs={:08X}", pd[0] as u8, pd[1] as u8, pd[2] as u8, pd[3] as u8, RS); } for k in 0..6usize { g::P_HSD_GObj[k].user_data = &raw mut m::ifStock_804A157C[(rnd() % 6) as usize] as *mut _; }
}

#[test]
fn ifstock_matches_c() {
    unsafe {
        g::install();
        g::preg_state();
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_ifstock.txt"));
        let nc: [u32; 52] = [400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 800, 400, 400, 400, 400, 400, 300, 400, 300, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 800, 600, 1500, 400, 400, 800, 400];

        for op in 0..52u32 {
            for k in 0..nc[op as usize] {
                case(op, k);
                g::reset_state();
                setup_extra();
                let mut ret = 0u32;
                match op {
                    0 => {  let a0: i32 = (rnd() % 6) as i32; let a1: i32 = (rnd() % 6) as i32; ret = m::ifStock_802F7EFC(a0, a1) as u32; }
                    1 => {  let a0: u8 = (rnd() % 6) as u8; ret = m::ifStock_802F8298_tobj_frame(a0).to_bits(); }
                    2 => {  let a0: *mut IfStockUserData = ((&raw mut m::ifStock_804A157C[(rnd() % 6) as usize]) as *mut IfStockUserData) as *mut IfStockUserData; ret = pi(m::ifStock_802F8298_update_stocks(a0)); }
                    3 => {  let a0: *mut u8 = (&raw mut m::ifStock_804A157C[(rnd() % 6) as usize].anim[(rnd() % 7) as usize]) as *mut u8; m::ifStock_802F8298_step_anim(a0); }
                    4 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::ifStock_802F8298(a0); }
                    5 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = pi(m::ifStock_802F89F8_get_data(a0)); }
                    6 => {  let a0: i32 = (if rnd() & 3 == 0 { (rnd() % 12) as i32 } else { (rnd() % 120000) as i32 }) as i32; ret = m::ifStock_802F89F8_count(a0) as u32; }
                    7 => {  let a0: i32 = ((-2 + (rnd() % 15) as i32)) as i32; ret = m::ifStock_802F89F8_divisor(a0) as u32; }
                    8 => {  let a0: i32 = (if rnd() & 3 == 0 { (rnd() % 12) as i32 } else { (rnd() % 120000) as i32 }) as i32; let a1: i32 = ((-1 + (rnd() % 9) as i32)) as i32; ret = m::ifStock_802F89F8_digit(a0, a1) as u32; }
                    9 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::ifStock_802F89F8(a0); }
                    10 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::fn_802F8DB0(a0); }
                    11 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::fn_802F8E08(a0); }
                    12 => { for q in 0..16usize { if m::ifStock_804A1A8C[q].is_null() { m::ifStock_804A1A8C[q] = &raw mut g::P_HSD_GObj[0]; } } let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::fn_802F916C(a0); }
                    13 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; let a1: *mut IfStockUserData = ((&raw mut m::ifStock_804A157C[(rnd() % 6) as usize]) as *mut IfStockUserData) as *mut IfStockUserData; m::fn_802F9410_inline(a0, a1); }
                    14 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::fn_802F9410(a0); }
                    15 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; let a1: i32 = (fv()) as i32; m::fn_802F94E0(a0, a1 as isize); }
                    16 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; let a1: i32 = (fv()) as i32; m::fn_802F9548(a0, a1 as isize); }
                    17 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; let a1: i32 = (fv()) as i32; m::fn_802F9598(a0, a1 as isize); }
                    18 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; let a1: i32 = (fv()) as i32; m::fn_802F95E8(a0, a1 as isize); }
                    19 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; let a1: i32 = (fv()) as i32; m::fn_802F9680(a0, a1 as isize); }
                    20 => {  let a0: *mut HSD_JObj = g::pick_HSD_JObj(1) as *mut HSD_JObj; m::ifStock_MexAttach(a0); }
                    21 => {  let a0: i32 = (fv()) as i32; let a1: i32 = (fv()) as i32; ret = m::ifStock_MexFrame(a0, a1).to_bits(); }
                    22 => {  let a0: i32 = (fv()) as i32; let a1: i32 = (fv()) as i32; let a2: f32 = (rg()) as f32; let a3: f32 = (rg()) as f32; ret = pi(m::ifStock_802F96D0(a0, a1, a2, a3)); }
                    23 => {  let a0: *mut VsSceneState = (&raw mut g::P_VsSceneController[0].state) as *mut VsSceneState; let a1: i32 = (rnd() % 6) as i32; ret = pi(m::ifStock_802F98E8_get_match_info(a0, a1)); }
                    24 => { NULLJ = 1; if rnd() & 7 == 0 { m::ifStock_804A1378.x0 = null_mut(); } for q in 0..6usize { let m = rnd(); let bb = (m & 1) | (((m >> 1) & 1) << 1); *(&raw mut g::P_VsSceneController[0].state.fighters[q] as *mut u8).add(4) = (bb | ((bb & 1) << 7) | ((bb & 2) << 5)) as u8; } let a0: u8 = (rnd() % 6) as u8; let a1: u8 = ((0 + (rnd() % 4) as i32)) as u8; m::ifStock_802F98E8(a0, a1); }
                    25 => {  let a0: i32 = ((0 + (rnd() % 130) as i32)) as i32; ret = pi(m::ifStock_802F9F48_inline(a0)); }
                    26 => {  let a0: i32 = ((0 + (rnd() % 130) as i32)) as i32; ret = pi(m::ifStock_802F9F48(a0)); }
                    27 => {  let a0: i32 = ((0 + (rnd() % 4) as i32)) as i32; ret = pi(m::ifStock_802FA118(a0)); }
                    28 => {  let a0: i32 = ((-500 + (rnd() % 12001) as i32)) as i32; m::ifStock_802FA2D0(a0); }
                    29 => {  let a0: i32 = ((0 + (rnd() % 2) as i32)) as i32; m::ifStock_802FA5BC(a0); }
                    30 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::fn_802FA6C4(a0); }
                    31 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::fn_802FA8C0(a0); }
                    32 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::fn_802FAC34(a0); }
                    33 => {  let a0: i32 = ((0 + (rnd() % 2) as i32)) as i32; m::ifStock_802FA5BC_noinline(a0); }
                    34 => {  let a0: *mut *mut HSD_GObj = ((&raw mut m::ifStock_804A1A8C) as *mut *mut HSD_GObj) as *mut *mut HSD_GObj; let a1: i32 = ((0 + (rnd() % 17) as i32)) as i32; m::ifStock_ClearGObjSlots(a0, a1); }
                    35 => {  let a0: *mut *mut HSD_GObj = ((&raw mut m::ifStock_804A1A8C) as *mut *mut HSD_GObj) as *mut *mut HSD_GObj; m::ifStock_CreateStockGObjs(a0); }
                    36 => {   m::ifStock_802FAEC4(); }
                    37 => {   m::ifStock_802FB390(); }
                    38 => {   m::ifStock_802FB41C(); }
                    39 => {   m::ifStock_802FB484(); }
                    40 => {  let a0: *mut HSD_MObj = g::pick_HSD_MObj(0) as *mut HSD_MObj; let a1: *mut _GXColor = (&raw mut GCOL[0]) as *mut _GXColor; m::copyColor(a0, a1); }
                    41 => {  let a0: i32 = (rnd() % 6) as i32; let a1: *mut _GXColor = (&raw mut GCOL[0]) as *mut _GXColor; m::ifStock_802FB4EC(a0, a1); }
                    42 => {  let a0: i32 = (rnd() % 6) as i32; m::ifStock_802FB650(a0); }
                    43 => {  let a0: i32 = (rnd() % 6) as i32; m::ifStock_802FB6AC(a0); }
                    44 => {   m::ifStock_GenoReset(); }
                    45 => {  let a0: i32 = ((0 + (rnd() % 4) as i32)) as i32; let a1: i32 = ((0 + (rnd() % 4) as i32)) as i32; let a2: *mut HSD_ImageDesc = g::pick_HSD_ImageDesc(1) as *mut HSD_ImageDesc; ret = pi(m::ifStock_GenoFind(a0, a1, a2)); }
                    46 => {  let a0: *mut HSD_TObj = g::pick_HSD_TObj(0) as *mut HSD_TObj; let a1: i32 = (rnd() % 2) as i32; ret = pi(m::ifStock_GenoBinding(a0, a1)); }
                    47 => {  let a0: *mut HSD_TObj = g::pick_HSD_TObj(1) as *mut HSD_TObj; let a1: f32 = (if rnd() % 3 == 0 { 1000000.0f32 + ((rnd() % 6) * 256 + rnd() % 4) as f32 } else { rg() }) as f32; m::ifStock_ReqAnim(a0, a1); }
                    48 => { FBUF[63] = 0; FPTR = &raw mut FBUF[(rnd() % 8) as usize]; let a0: *mut *mut u8 = (&raw mut FPTR) as *mut *mut u8; let a1: u8 = (fv()) as u8; ret = m::genoArt_FObjValue(a0, a1).to_bits(); }
                    49 => { FBUF[63] = 0; FPTR = &raw mut FBUF[(rnd() % 8) as usize]; let a0: *mut *mut u8 = (&raw mut FPTR) as *mut *mut u8; ret = m::genoArt_FObjPacked(a0) as u32; }
                    50 => {  let a0: *mut HSD_TObj = g::pick_HSD_TObj(1) as *mut HSD_TObj; let a1: i32 = ((-1 + (rnd() % 8) as i32)) as i32; let a2: *mut i32 = (&raw mut MAXI) as *mut i32; ret = m::genoArt_ImageKeyAt(a0, a1, a2) as u32; }
                    51 => { let a0 = NM[(rnd() % 8) as usize].as_ptr() as *const i8; m::genoArt_Initials(a0, (&raw mut OUTB) as *mut i8); }
                    _ => unreachable!(),
                }
                g::hash_state();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 23100);
    }
}
