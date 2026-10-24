//! Written by gen/mtu/mk_driver.py from the same description as gen/mtu/tu/iteffect.c.
//! src/tu/iteffect.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu/run_tu.py iteffect`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "mtu_harness_world/mod.rs"]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::iteffect as m;
use mtu_harness::*;
#[path = "mtu/iteffect_gen.rs"]
mod g;

const EFIDS: [i32; 99] = [0x24f, 0x250, 0x251, 0x3e9, 0x3ed, 0x3ee, 0x3f3, 0x3f5, 0x3f7, 0x3fa, 0x3fd, 0x3ff, 0x400, 0x401, 0x402, 0x403, 0x404, 0x405, 0x407, 0x408, 0x409, 0x40c, 0x412, 0x413, 0x414, 0x415, 0x416, 0x417, 0x418, 0x419, 0x41a, 0x41b, 0x41c, 0x41e, 0x422, 0x423, 0x424, 0x425, 0x428, 0x429, 0x42a, 0x42d, 0x432, 0x433, 0x434, 0x438, 0x439, 0x43a, 0x43d, 0x43e, 0x440, 0x443, 0x446, 0x448, 0x449, 0x44a, 0x44d, 0x44e, 0x44f, 0x450, 0x451, 0x452, 0x457, 0x458, 0x459, 0x45a, 0x45b, 0x45c, 0x45f, 0x460, 0x461, 0x462, 0x463, 0x464, 0x465, 0x466, 0x467, 0x468, 0x469, 0x46a, 0x46b, 0x46c, 0x46d, 0x46e, 0x46f, 0x470, 0x471, 0x472, 0x473, 0x4d0, 0x513, 0x514, 0x516, 0x1387, 0x1388, 0x1b57, 0x1b58, 0x2327, 0x2328];
pub static mut ARC: it_804D6D20_t = unsafe { core::mem::zeroed() };
static mut ARCA: [[*mut Article; 4]; 3] = [[null_mut(); 4]; 3];
static mut ARC40: it_804D6D40_t = unsafe { core::mem::zeroed() };
static mut ARC53C: [Fighter_804D653C_t; 2] = unsafe { core::mem::zeroed() };

unsafe fn setup_stem() {
    m::ext::it_804D6D28 = &raw mut ICD[0];
{ for j in 0..15 { m::ext::it_803F1ED8[j] = b'a' as i8 + (rnd() % 26) as i8; m::ext::it_803F1EE4[j] = b'a' as i8 + (rnd() % 26) as i8; m::ext::it_803F1EF0[j] = b'a' as i8 + (rnd() % 26) as i8; } m::ext::it_803F1ED8[15] = 0; m::ext::it_803F1EE4[15] = 0; m::ext::it_803F1EF0[15] = 0; ARC = core::mem::zeroed(); ARC40 = core::mem::zeroed(); ARC53C = core::mem::zeroed(); ARC.x0 = &raw mut ICD[0]; for i in 0..3 { for j in 0..4 { ARCA[i][j] = pick_Article(); } } ARC.x4 = (&raw mut ARCA[0]) as *mut *mut Article; ARC.x8 = (&raw mut ARCA[1]) as *mut *mut Article; ARC.xC = (&raw mut ARCA[2]) as *mut *mut Article; ARC.x10 = &raw mut ARC40; ARC.x14 = &raw mut ARC53C[(rnd() & 1) as usize]; ARC40.x0 = fv() as i32; ARC40.x4 = rw(); ARC40.x8 = rw(); ARC40.xC = rw(); ARC40.x10 = rw(); ARC40.x14 = rw(); ARC40.x18 = rw(); }
}

unsafe fn hash_stem() {
    g::hashg_all();
    hm(pi(m::ext::it_804D6D28));
g::hash_it_804D6D40_t(&ARC40);
}

#[test]
fn iteffect_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::freg_all();
        freg_world();
preg(&raw const ARC, core::mem::size_of_val(&ARC)); preg(&raw const ARCA, core::mem::size_of_val(&ARCA)); preg(&raw const ARC40, core::mem::size_of_val(&ARC40)); preg(&raw const ARC53C, core::mem::size_of_val(&ARC53C));
        let mut c = Check::new(include_str!("data/mtu_iteffect.txt"));
        let mut total = 0u32;
        for op in 0..4u32 {
            let (n, ft) = match op {
                0 => (300u32, false),
                1 => (300u32, false),
                2 => (300u32, false),
                3 => (300u32, false),
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
                setup_stem();
                let mut ret = 0u32;
                match op {
                    0 => { let a0 = fv() as _;
m::it_8027870C(a0);
 }
                    1 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
m::it_802787B4(a0, a1);
 }
                    2 => { let a0 = &raw mut VA[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
m::it_80278800_rand_vec(a0, a1);
 }
                    3 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = (if fv() & 1 != 0 { EFIDS[(rnd() % 99) as usize] } else { (rnd() % 0x540) as i32 });
let a2 = fv() as _;
let a3 = &raw mut VA[(rnd() % 4) as usize];
let a4 = &raw mut VA[(rnd() % 4) as usize];
let a5 = fv() as _;
let a6 = rw();
m::it_80278800(a0, a1, a2, a3, a4, a5, a6);
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
