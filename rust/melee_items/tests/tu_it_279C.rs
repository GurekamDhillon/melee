//! Written by gen/mtu/mk_driver.py from the same description as gen/mtu/tu/it_279C.c.
//! src/tu/it_279C.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu/run_tu.py it_279C`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "mtu_w/mod.rs"]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::it_279C as m;
use mtu_harness::*;
#[path = "mtu/it_279C_gen.rs"]
mod g;



unsafe fn setup_stem() {

{ g::setup_xpools(); g::S70 = core::mem::zeroed(); g::S74 = core::mem::zeroed(); for q in 0..2usize { g::S70[q].x2070_int = fv() as _; g::S74[q].x207C.x = fv() as i32; g::S74[q].x207C.y = fv() as i32; } let n = rnd() % 28; for k in 0..32usize { g::KL[k] = if k as u32 >= n { It_PKind_Terminate as i32 } else { (It_PKind_Start as u32 + rnd() % (It_PKind_Terminate - It_PKind_Start) as u32) as i32 }; } for q in 0..3usize { core::ptr::write_bytes((&raw mut FT[q].x2070) as *mut u8, 0, core::mem::size_of_val(&FT[q].x2070)); FT[q].x2070.x2070_int = fv() as i32; core::ptr::write_bytes((&raw mut FT[q].x2074) as *mut u8, 0, core::mem::size_of_val(&FT[q].x2074)); FT[q].x2074.x2074_vec.x = rw(); FT[q].x2074.x2074_vec.y = rw(); FT[q].x2074.x207C.x = fv() as i32; FT[q].x2074.x207C.y = fv() as i32; FT[q].x2074.anon_10.x2084 = fv(); FT[q].x2074.x2088 = fv() as u16; } for q in 0..2usize { g::PDA[q].x38 = (rnd() % 7) as i32; ART[q].x4_specialAttributes = (&raw mut g::PDA[q]) as *mut _; } for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); (*ps).x0 = fv() as i32; (*ps).x4.x = rw(); (*ps).x4.y = rw(); (*ps).x4.z = rw(); (*ps).x10.x = rw(); (*ps).x10.y = rw(); (*ps).x10.z = rw(); (*ps).x1C = rw(); (*ps).x20 = rw(); (*ps).x24 = rw(); (*ps).x28 = rw(); for k in 0..6usize { (*ps).x2C[k] = rw(); } (*ps).x44 = rw(); (*ps).x48 = rw(); (*ps).x4C = rw(); (*ps).x50 = rw(); (*ps).x58 = (rnd() % 6) as i32; (*ps).x54 = (rnd() % 8) as i32; (*ps).x5C = (rnd() % 3) as i32; core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); IT[q].xD90.x2070_int = fv() as i32; } for q in 0..8usize { JO[q].flags &= !(JOBJ_USE_QUATERNION as u32); } }
}

unsafe fn hash_stem() {
    g::hashg_all();

g::hash_xpools(); hm(rh(&g::KL)); hm(rh(&g::S70)); hm(rh(&g::S74));
}

#[test]
fn it_279C_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::freg_all();
        freg_world();
g::preg_xpools(); preg(&raw const g::KL, core::mem::size_of_val(&g::KL)); preg(&raw const g::S70, core::mem::size_of_val(&g::S70)); preg(&raw const g::S74, core::mem::size_of_val(&g::S74));
        let mut c = Check::new(include_str!("data/mtu_it_279C.txt"));
        let mut total = 0u32;
        for op in 0..30u32 {
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
                14 => (300u32, false),
                15 => (300u32, false),
                16 => (300u32, false),
                17 => (300u32, false),
                18 => (300u32, false),
                19 => (300u32, false),
                20 => (300u32, false),
                21 => (300u32, false),
                22 => (300u32, false),
                23 => (300u32, false),
                24 => (300u32, false),
                25 => (300u32, false),
                26 => (300u32, false),
                27 => (300u32, false),
                28 => (300u32, false),
                29 => (300u32, false),
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
                setup_stem();
                let mut ret = 0u32;
                match op {
                    0 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80279C48(a0);
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
                    1 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = rw();
m::it_80279CDC(a0, a1);
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
                    2 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80279D38(a0);
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
                    3 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = rw();
m::it_80279D5C(a0, a1);
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
                    4 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80279E24(a0);
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
                    5 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80279FF8(a0);
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
                    6 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8027A09C(a0) as u32;
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
                    7 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = unsafe { core::mem::transmute::<usize, _>(pick_cb(1)) };
ret = m::it_8027A118(a0, a1) as u32;
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
                    8 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8027A13C(a0);
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
                    9 => { let a0 = &raw mut JO[(rnd() % 8) as usize];
let a1 = &raw mut IT[(rnd() % 4) as usize];
m::it_8027A160(a0, a1);
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
                    10 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8027A344(a0);
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
                    11 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
ret = m::it_8027A364(a0) as u32;
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
                    12 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
ret = m::it_8027A4D4(a0) as u32;
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
                    13 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
let a1 = (&raw mut g::KL) as *mut c_void;
ret = m::it_8027A780(a0, a1) as u32;
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
                    14 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
ret = m::it_8027A9B8(a0) as u32;
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
                    15 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut IT[(rnd() % 4) as usize];
let a2 = (if rnd() % 3 == 0 { It_PKind_Marumine as u32 } else { fv() }) as i32;
m::it_8027AAA0(a0, a1, a2);
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
                    16 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8027AB64(a0) as u32;
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
                    17 => { let a0 = fv() as _;
let a1 = &raw mut GO[(rnd() % 4) as usize];
let a2 = &raw mut JO[(rnd() % 8) as usize];
let a3 = rw();
m::it_8027ADEC(a0, a1, a2, a3);
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
                    18 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8027AE34(a0) as u32;
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
                    19 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8027AF50(a0) as u32;
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
                    20 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut GO[4 + (rnd() % 3) as usize];
m::it_8027B070(a0, a1);
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
                    21 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = { SP.x0_parent_gobj = &raw mut GO[(rnd() % 7) as usize]; SP.x4_parent_gobj2 = &raw mut GO[(rnd() % 7) as usize]; &raw mut SP };
m::it_8027B0C4(a0, a1);
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
                    22 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8027B1F4(a0);
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
                    23 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
m::it_8027B288(a0, a1);
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
                    24 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
m::it_8027B330(a0, a1);
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
                    25 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut GO[(rnd() % 4) as usize];
let a2 = rw();
m::it_8027B378(a0, a1, a2);
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
                    26 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut GO[(rnd() % 4) as usize];
let a2 = rw();
m::it_8027B408(a0, a1, a2);
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
                    27 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut GO[(rnd() % 4) as usize];
m::it_8027B4A4(a0, a1);
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
                    28 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut GO[(rnd() % 4) as usize];
m::it_8027B508(a0, a1);
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
                    29 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8027B564(a0);
for q in 0..4usize { let ps = &raw mut IT[q].xDD4_itemVar.pokemon_spawn; hm((*ps).x0 as u32); hm((*ps).x4.x.to_bits()); hm((*ps).x4.y.to_bits()); hm((*ps).x4.z.to_bits()); hm((*ps).x10.x.to_bits()); hm((*ps).x10.y.to_bits()); hm((*ps).x10.z.to_bits()); hm((*ps).x1C.to_bits()); hm((*ps).x20.to_bits()); hm((*ps).x24.to_bits()); hm((*ps).x28.to_bits()); for k in 0..6usize { hm((*ps).x2C[k].to_bits()); } hm((*ps).x44.to_bits()); hm((*ps).x48.to_bits()); hm((*ps).x4C.to_bits()); hm((*ps).x50.to_bits()); hm((*ps).x54 as u32); hm((*ps).x58 as u32); hm((*ps).x5C as u32); core::ptr::write_bytes((&raw mut IT[q].xDD4_itemVar) as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); hm(IT[q].xD90.x2070_int as u32); core::ptr::write_bytes((&raw mut IT[q].xD90) as *mut u8, 0, core::mem::size_of_val(&IT[q].xD90)); } }
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
