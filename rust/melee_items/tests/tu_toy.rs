//! src/tu/toy.rs Toy_80304B94 / Toy_80304CC8 / _Toy_80304D30 / Toy_80311960 (and the callees that run for real in
//! both builds: un_80304470, Toy_803048C0, Toy_80304B0C, Toy_80305918, Toy_803060BC) vs the C (src/melee/ty/toy.c
//! included verbatim, shipping contraction; gen/mtu/tu/toy.c). Regenerate the expected file with
//! `python rust/melee_items/gen/mtu/run_tu.py toy`. 600 seeded states per op.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_items::tu::toy as m;
use melee_types::*;
use mtu_harness::*;
#[path = "mtu/toy_gen.rs"]
mod g;

const NT: usize = 293;
pub static mut SAVEF: [u16; NT] = [0; NT];
pub static mut CATF: u16 = 0;
pub static mut TCNT: i16 = 0;
static mut SKIP: [i16; 16] = [0; 16];
static mut TBL_JP: [TrophyData; NT + 2] = unsafe { core::mem::zeroed() };
static mut TBL_US: [TrophyData; NT + 2] = unsafe { core::mem::zeroed() };

const CATN: [i32; 9] = [0x1B, 0x7E, 0x48, 0xC, 0x17, 0x10, 0xA, 0x4, 0x2];

unsafe fn fill_td(t: *mut TrophyData, id: i32, cat: i32) {
    (*t).id = id;
    (*t).x04 = (rnd() % 9) as _;
    (*t).x08 = rw();
    (*t).x0C = rw();
    (*t).x10 = rw();
    (*t).x14 = rw();
    (*t).x18 = rw();
    (*t).x1C = rw();
    (*t).x20 = cat as _;
    (*t).x21 = (rnd() & 7) as _;
    (*t).x22 = 0;
    (*t).x23 = 0;
}

unsafe fn setup_stem() {
    let full = rnd() % 3 == 0;
    for i in 0..302 {
        m::Toy_804A284C[i] = if rnd() & 3 == 0 { 0 } else { rnd() as u16 };
    }
    for i in 0..NT {
        SAVEF[i] = if rnd() & 3 == 0 { 0 } else { rnd() as u16 };
        if full {
            if SAVEF[i] & 0xFF == 0 {
                SAVEF[i] |= 1;
            }
            if m::Toy_804A284C[5 + i] & 0xFF == 0 {
                m::Toy_804A284C[5 + i] |= 1;
            }
        }
    }
    CATF = rnd() as u16;
    TCNT = rnd() as i16;
    let n = (rnd() % 7) as usize;
    for i in 0..n {
        SKIP[i] = (rnd() % NT as u32) as i16;
    }
    SKIP[n] = -1;
    m::_Toy_sbss_804D6EB4 = (&raw mut SKIP) as *mut i16;
    TBL_US = core::mem::zeroed();
    TBL_JP = core::mem::zeroed();
    let (mut c, mut lo) = (0usize, 0i32);
    for i in 0..NT {
        while c < 8 && lo >= CATN[c] {
            lo = 0;
            c += 1;
        }
        let mut cat = c as i32;
        lo += 1;
        if rnd() & 7 == 0 {
            cat = (rnd() % 9) as i32;
        }
        fill_td(&raw mut TBL_US[i], i as i32, cat);
    }
    TBL_US[NT].id = -1;
    let n = (rnd() % 60) as usize;
    for i in 0..n {
        let id = (rnd() % NT as u32) as i32;
        let cat = (rnd() % 9) as i32;
        fill_td(&raw mut TBL_JP[i], id, cat);
    }
    TBL_JP[n].id = -1;
    m::_Toy_sbss_804D6EC0 = (&raw mut TBL_JP) as *mut TrophyData;
    m::_Toy_sbss_804D6EC4 = (&raw mut TBL_US) as *mut TrophyData;
}

unsafe fn hash_stem() {
    g::hashg_all();
    for i in 0..302 {
        hm(m::Toy_804A284C[i] as u32);
    }
    for i in 0..NT {
        hm(SAVEF[i] as u32);
    }
    hm(CATF as u32);
    hm(TCNT as u16 as u32);
    hm(pi(m::_Toy_sbss_804D6EB4));
    hm(pi(m::_Toy_sbss_804D6EC0));
    hm(pi(m::_Toy_sbss_804D6EC4));
}

#[test]
fn toy_matches_c() {
    unsafe {
        g::install();
        preg(&raw const m::Toy_804A284C, core::mem::size_of_val(&m::Toy_804A284C));
        preg(&raw const SAVEF, core::mem::size_of_val(&SAVEF));
        preg(&raw const CATF, 2);
        preg(&raw const TCNT, 2);
        preg(&raw const SKIP, core::mem::size_of_val(&SKIP));
        preg(&raw const TBL_JP, core::mem::size_of_val(&TBL_JP));
        preg(&raw const TBL_US, core::mem::size_of_val(&TBL_US));
        g::freg_all();

        let mut c = Check::new(include_str!("data/mtu_toy.txt"));
        let mut total = 0u32;
        for op in 0..6u32 {
            for k in 0..600u32 {
                case(op, k);
                setup_stem();
                let mut ret = 0u32;
                match op {
                    0 => {
                        let a0 = (rnd() % 13) as i32 - 2;
                        ret = m::Toy_80304B94(a0) as u32;
                    }
                    1 => ret = m::_Toy_80304D30() as u32,
                    2 => m::Toy_80311960(),
                    3 => {
                        let a0 = (rnd() % NT as u32) as i32;
                        ret = m::Toy_80304CC8(a0) as u32;
                    }
                    4 => ret = m::un_80304470() as u32,
                    5 => {
                        let a0 = (rnd() % (NT as u32 + 4)) as i32 - 2;
                        let a1 = (rnd() % 9) as i32;
                        let r = m::Toy_803060BC(a0, a1);
                        ret = r.to_bits();
                    }
                    _ => unreachable!(),
                }
                hash_stem();
                c.line(op, k, ret);
                total += 1;
            }
        }
        assert_eq!(c.done(), total);
    }
}
