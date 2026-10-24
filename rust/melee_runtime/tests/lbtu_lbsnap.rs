//! Written by gen/lbtu/mk_driver.py from the same description as gen/lbtu/tu/lbsnap.c.
//! src/tu/lbsnap.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/lbtu/run_tu.py lbsnap`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, unused_parens)]
#[path = "lbtu_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::lbsnap as m;
use mtu_harness::*;
#[path = "lbtu_gen/lbsnap_gen.rs"]
mod g;

static mut SNAP: [m::Unk80433380_0; 1] = unsafe { core::mem::zeroed() };
static mut SLOT: [m::Unk80433380_48; 2] = unsafe { core::mem::zeroed() };
static mut ICON: [m::LbMcSnapMemSnapIconData; 2] = unsafe { core::mem::zeroed() };
static mut BANNER: [u8; 6208] = unsafe { core::mem::zeroed() };
static mut ICNB: [u8; 1024] = unsafe { core::mem::zeroed() };
static mut SRC: [u8; 458752] = unsafe { core::mem::zeroed() };
static mut TXT: [i8; 64] = unsafe { core::mem::zeroed() };
static mut SRCT: [u32; 16] = [0; 16];

unsafe fn setup() {
    let _ = 0;
    SNAP = core::mem::zeroed();
    SLOT = core::mem::zeroed();
    ICON = core::mem::zeroed();
    BANNER = core::mem::zeroed();
    ICNB = core::mem::zeroed();
    SRC = core::mem::zeroed();
    TXT = core::mem::zeroed();
    for i in 0..1 { g::fill_Unk80433380_0(&mut SNAP[i]); }
    for i in 0..2 { g::fill_Unk80433380_48(&mut SLOT[i]); }
    m::ext::__OSBusClock = 162000000;
    for k in 0..2usize {
        g::fill_Unk80433380_48(&mut SLOT[k]);
        SLOT[k].num = 8 + (rnd() % 100) as i32;
        SLOT[k].card_result = if rnd() % 3 == 0 { 0 } else { fv() as i32 };
        for q in 0..0x7Fusize { SLOT[k].entries[q].time = rnd() % 6; }
    }
    g::fill_Unk80433380_0(&mut SNAP[0]);
    SNAP[0].x0 = if rnd() & 1 != 0 { 4 } else { fv() as i32 };
    g::fill_Unk80433380(&mut m::lbSnap_80433380);
    m::lbSnap_80433380.snap = &raw mut SNAP[0];
    m::lbSnap_80433380.slot = &raw mut SLOT[0];
    m::lbSnap_80433380.icon_data = &raw mut ICON[0];
    ICON[0].ptr = BANNER.as_mut_ptr(); ICON[1].ptr = ICNB.as_mut_ptr();
    for k in 0..0x40usize { TXT[k] = (rnd() | 1) as i8; }
    for k in 0..0x1800 + 64usize { BANNER[k] = rnd() as u8; }
    for k in 0..0x400usize { ICNB[k] = rnd() as u8; }
}
unsafe fn hash_all() {
    g::hash_Unk80433380(&m::lbSnap_80433380);
    g::hash_Unk803BACC8(&m::lbSnap_803BACC8);
    hm(pib(ICON[0].ptr)); hm(pib(ICON[1].ptr));
    g::hash_Unk80433380_0(&SNAP[0]);
    for k in 0..2usize { g::hash_Unk80433380_48(&SLOT[k]); }
    for i in 0..1 { g::hash_Unk80433380_0(&SNAP[i]); }
    for i in 0..2 { g::hash_Unk80433380_48(&SLOT[i]); }
    for k in 0..0x40usize { hm(TXT[k] as u8 as u32); }
    for k in 0..0x1800 + 64usize { hm(BANNER[k] as u32); }
    for k in 0..0x400usize { hm(ICNB[k] as u32); }
}
#[test]
fn lbsnap_matches_c() {
    unsafe {
        g::install();
        preg(&raw const SNAP, core::mem::size_of_val(&SNAP));
        preg(&raw const SLOT, core::mem::size_of_val(&SLOT));
        preg(&raw const ICON, core::mem::size_of_val(&ICON));
        preg(&raw const BANNER, core::mem::size_of_val(&BANNER));
        preg(&raw const ICNB, core::mem::size_of_val(&ICNB));
        preg(&raw const SRC, core::mem::size_of_val(&SRC));
        preg(&raw const TXT, core::mem::size_of_val(&TXT));
        preg(&raw const m::lbSnap_80433380, core::mem::size_of_val(&m::lbSnap_80433380)); preg(&raw const m::lbSnap_803BACC8, core::mem::size_of_val(&m::lbSnap_803BACC8));
        g::freg_all();

        let mut c = Check::new(include_str!("data/lbtu_lbsnap.txt"));
        for op in 0..22u32 {
            for k in 0..400u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                    0 => {   m::lbSnap_8001D2BC(); } // lbSnap_8001D2BC
                    1 => {  let v0: i32 = ((fv() % 3) as i32) as i32; ret = m::lbSnap_8001D338(v0) as u32; } // lbSnap_8001D338
                    2 => {  let v0: i32 = ((fv() & 1) as i32) as i32; ret = m::lbSnap_8001D350(v0) as u32; } // lbSnap_8001D350
                    3 => {  let v0: i32 = ((fv() & 1) as i32) as i32; ret = m::lbSnap_8001D394(v0) as u32; } // lbSnap_8001D394
                    4 => {  let v0: i32 = ((fv() & 1) as i32) as i32; ret = m::lbSnap_8001D3B0(v0) as u32; } // lbSnap_8001D3B0
                    5 => {  let v0: i32 = ((fv() & 1) as i32) as i32; ret = m::lbSnap_8001D3CC(v0) as u32; } // lbSnap_8001D3CC
                    6 => {  let v0: i32 = ((fv() & 1) as i32) as i32; let v1: i32 = ((fv() % 8) as i32) as i32; ret = m::lbSnap_8001D3E8(v0, v1) as u32; } // lbSnap_8001D3E8
                    7 => {  let v0: i32 = ((fv() & 1) as i32) as i32; ret = m::lbSnap_8001D40C(v0) as u32; } // lbSnap_8001D40C
                    8 => {  let v0: i32 = ((fv() & 1) as i32) as i32; m::lbSnap_8001D4A4(v0, TXT.as_mut_ptr()); } // lbSnap_8001D4A4
                    9 => {  let v0: i32 = ((fv() & 1) as i32) as i32; let v1: i32 = ((fv() % 8) as i32) as i32; ret = m::lbSnap_8001D5FC(v0, v1) as u32; } // lbSnap_8001D5FC
                    10 => {  let v0: i32 = ((fv() & 1) as i32) as i32; let v1: i32 = ((fv() % 8) as i32) as i32; let v2: i32 = ((fv() % 8) as i32) as i32; ret = m::lbSnap_8001D7B0(v0, v1, v2) as u32; } // lbSnap_8001D7B0
                    11 => { for q in 0..16usize { SRCT[q] = rnd(); } for q in 0..0x70000usize / 4 { *(SRC.as_mut_ptr() as *mut u32).add(q) = SRCT[q & 15] ^ (q as u32).wrapping_mul(2654435761u32); }  m::lbSnap_8001DA5C(SRC.as_ptr()); } // lbSnap_8001DA5C
                    12 => { for q in 0..16usize { SRCT[q] = rnd(); } for q in 0..0x70000usize / 4 { *(SRC.as_mut_ptr() as *mut u32).add(q) = SRCT[q & 15] ^ (q as u32).wrapping_mul(2654435761u32); }  ret = m::lbSnap_8001DC0C(SRC.as_mut_ptr()) as u32; } // lbSnap_8001DC0C
                    13 => { for q in 0..16usize { SRCT[q] = rnd(); } for q in 0..0x70000usize / 4 { *(SRC.as_mut_ptr() as *mut u32).add(q) = SRCT[q & 15] ^ (q as u32).wrapping_mul(2654435761u32); }  ret = m::lbSnap_8001DE8C(SRC.as_mut_ptr() as *mut core::ffi::c_void) as u32; } // lbSnap_8001DE8C
                    14 => {   ret = m::lbSnap_8001DF20() as u32; } // lbSnap_8001DF20
                    15 => {  let v0: i32 = ((fv() & 1) as i32) as i32; ret = m::lbSnap_8001DF6C(v0) as u32; } // lbSnap_8001DF6C
                    16 => {  let v0: i32 = ((fv() & 1) as i32) as i32; let v1: i32 = ((fv() % 8) as i32) as i32; ret = m::lbSnap_8001E058(v0, v1) as u32; } // lbSnap_8001E058
                    17 => {   ret = m::lbSnap_8001E204() as u32; } // lbSnap_8001E204
                    18 => {   ret = m::lbSnap_8001E210() as u32; } // lbSnap_8001E210
                    19 => {   m::lbSnap_8001E218(SRC.as_mut_ptr() as *mut core::ffi::c_void, &raw mut SLOT[0]); } // lbSnap_8001E218
                    20 => {   m::lbSnap_8001E27C(); } // lbSnap_8001E27C
                    21 => {   m::lbSnap_8001E290(); } // lbSnap_8001E290
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 8800);
    }
}
