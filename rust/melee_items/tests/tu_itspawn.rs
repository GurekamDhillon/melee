//! Written by gen/mtu/mk_driver.py from the same description as gen/mtu/tu/itspawn.c.
//! src/tu/itspawn.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu/run_tu.py itspawn`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "mtu_w/mod.rs"]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itspawn as m;
use mtu_harness::*;
#[path = "mtu/itspawn_gen.rs"]
mod g;

pub static mut CNT: [i32; 64] = [0; 64]; pub static mut GMV: i32 = 0; pub static mut FM64: u64 = 0; pub static mut AB: [u8; 2048] = [0; 2048]; pub static mut AO: u32 = 0; static mut TB: [ItemPickTable; 2] = unsafe { core::mem::zeroed() }; static mut TBK: [[u8; 8]; 2] = [[0; 8]; 2]; static mut TBW: [[u16; 8]; 2] = [[0; 8]; 2]; static mut B47C: it_8026C47C_arg0_t = unsafe { core::mem::zeroed() };

unsafe fn setup_stem() {
    m::ext::it_804D6D28 = &raw mut ICD[0];
{ for q in 0..64usize { CNT[q] = if rnd() & 3 == 0 { 0 } else { (fv() % 40) as i32 }; } GMV = if rnd() % 8 == 0 { -1 } else { (rnd() % 5) as i32 }; FM64 = if rnd() % 6 == 0 { 0 } else { fv64() }; AO = 0; AB = [0; 2048]; B47C = core::mem::zeroed(); m::it_804A0E30 = core::mem::zeroed(); m::it_804A0E50 = core::mem::zeroed(); m::it_804A0E60 = core::mem::zeroed(); m::it_804A0E30.x0 = (fv() % 4) as i32; m::it_804A0E30.x18 = FM64; for k in 0..2usize { let r = rnd(); TB[k] = core::mem::zeroed(); TB[k].size = (1 + (r % 7)) as u8; TB[k].x4 = (&raw mut TBK[k]) as *mut u8; TB[k].xC = (&raw mut TBW[k]) as *mut u16; TB[k].x8 = if (r >> 8) % 5 == 0 { 0 } else { (1 + (r >> 12) % 60) as u16 }; let mut cum = 0u32; for q in 0..8usize { TBK[k][q] = if rnd() % 4 == 0 { 0x22 } else { (rnd() % 40) as u8 }; TBW[k][q] = cum as u16; cum += rnd() % 12; } } (*m::ext::it_804D6D28).x128[0] = (fv() % 20) as i32; }
}

unsafe fn hash_stem() {
    g::hashg_all();
    hm(pi(m::ext::it_804D6D28));
{ g::hash_RandomItemSpawner(&m::it_804A0E30); g::hash_ItemPickTable(&m::it_804A0E50); g::hash_ItemPickTable(&m::it_804A0E60); g::hash_ItemPickTable(&TB[0]); g::hash_ItemPickTable(&TB[1]); for q in 0..64usize { hm(CNT[q] as u32); } hm(GMV as u32); hm64(0.0); hm(FM64 as u32); hm((FM64 >> 32) as u32); hm(AO); hm(rh(&AB)); hm(rh(&TBK)); hm(rh(&TBW)); hm(rh(&B47C)); }
}

#[test]
fn itspawn_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::freg_all();
        freg_world();
preg(&raw const CNT, core::mem::size_of_val(&CNT)); preg(&raw const AB, 2048); preg(&raw const TBK, 16); preg(&raw const TBW, 32);
        let mut c = Check::new(include_str!("data/mtu_itspawn.txt"));
        let mut total = 0u32;
        for op in 0..14u32 {
            let (n, ft) = match op {
                0 => (300u32, false),
                1 => (300u32, false),
                2 => (300u32, false),
                3 => (300u32, false),
                4 => (300u32, false),
                5 => (300u32, false),
                6 => (300u32, false),
                7 => (300u32, false),
                8 => (300u32, false),
                9 => (300u32, false),
                10 => (300u32, false),
                11 => (300u32, false),
                12 => (300u32, false),
                13 => (300u32, false),
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
                setup_stem();
                let mut ret = 0u32;
                match op {
                    0 => { let a0 = &raw mut B47C;
m::it_8026C47C(a0);
 }
                    1 => { let a0 = &raw mut TB[0];
ret = m::it_8026C65C(a0) as u32;
 }
                    2 => { 
ret = m::it_8026C704() as u32;
 }
                    3 => { let a0 = &raw mut TB[(rnd() & 1) as usize];
ret = m::it_8026C75C(a0) as u32;
 }
                    4 => { let a0 = { GMV = (rnd() % 5) as i32; &raw mut GO[(rnd() % 4) as usize] };
m::fn_8026C88C(a0);
 }
                    5 => { let a0 = &raw mut TB[(rnd() & 1) as usize];
let a1 = (&raw mut CNT) as *mut i32;
let a2 = fv64();
let a3 = (fv() % 12) as i32;
let a4 = rw();
m::it_8026CA4C(a0, a1, a2, a3, a4);
 }
                    6 => { let a0 = &raw mut VA[(rnd() % 4) as usize];
ret = m::it_8026CB3C(a0) as u32;
 }
                    7 => { let a0 = (&raw mut CNT) as *mut i32;
let a1 = fv64();
let a2 = rw();
m::it_8026CB9C(a0, a1, a2);
 }
                    8 => { let a0 = (&raw mut CNT) as *mut i32;
let a1 = fv64();
let a2 = rw();
m::it_8026CD50(a0, a1, a2);
 }
                    9 => { 
m::it_8026CF04();
 }
                    10 => { 
m::it_8026D018();
 }
                    11 => { let a0 = &raw mut VA[(rnd() % 4) as usize];
let a1 = fv() as _;
ret = m::it_8026D258(a0, a1) as u32;
 }
                    12 => { let a0 = fv() as _;
ret = m::it_8026D324(a0) as u32;
 }
                    13 => { 
ret = m::it_8026D3CC() as u32;
 }
                    _ => unreachable!(),
                }
                hash_world(ft);
                hash_stem();
                c.line(op, k, ret);
                total += 1;
            }
        }
        assert_eq!(c.done(), total);
    }
}
