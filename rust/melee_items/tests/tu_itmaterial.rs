//! Written by gen/mtu/mk_driver.py from the same description as gen/mtu/tu/itmaterial.c.
//! src/tu/itmaterial.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu/run_tu.py itmaterial`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "mtu_w/mod.rs"]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itmaterial as m;
use mtu_harness::*;
#[path = "mtu/itmaterial_gen.rs"]
mod g;

static mut TBOG: [u8; 16] = [0; 16]; static mut GCOL: [GXColor; 2] = unsafe { core::mem::zeroed() }; static mut TX: [HSD_TExp; 3] = unsafe { core::mem::zeroed() };
unsafe extern "C" fn cb_stev(m: *mut HSD_MObj, t: *mut HSD_TObj, r: u32) { t_(7010); t_pi(m); t_pi(t); t_(r); }
fn t_(v: u32) { t(v) } fn t_pi<T>(p: *mut T) { t(pi(p)) }

unsafe fn setup_stem() {

{ let q: usize; m::it_mobj_static_init(); g::setup_xpools(); TX = core::mem::zeroed(); for q in 0..3usize { g::fill_HSD_TECnst(&mut TX[q].cnst); } for q in 0..16usize { TBOG[q] = rnd() as u8; } for q in 0..2usize { GCOL[q].r = rnd() as u8; GCOL[q].g = rnd() as u8; GCOL[q].b = rnd() as u8; GCOL[q].a = rnd() as u8; } for q in 0..2usize { g::MO[q].parent.class_info = (&raw mut g::MI[0]) as *mut HSD_ClassInfo; } g::MI[0].setup_tev = Some(cb_stev); m::ext::hsdMObj.parent.release = core::mem::transmute::<usize, _>(cbev1 as usize); m::ext::hsdMObj.parent.amnesia = core::mem::transmute::<usize, _>(cbev2 as usize); m::ext::hsdMObj.load = core::mem::transmute::<usize, _>(cbp1 as usize); m::ext::hsdMObj.make_texp = core::mem::transmute::<usize, _>(cbev3 as usize); for q in 0..5usize { g::TO[q].next = if q < 2 && (rnd() & 1) != 0 { &raw mut g::TO[q + 1] } else { null_mut() }; } g::TO[4].imagedesc = if rnd() & 1 != 0 { (&raw mut TBOG) as *mut _ } else { null_mut() }; for q in 0..2usize { g::MO[q].tobj = if rnd() & 3 != 0 { &raw mut g::TO[(rnd() % 3) as usize] } else { null_mut() }; } m::ext::tobj_shadows = if rnd() & 1 != 0 { &raw mut g::TO[3] } else { null_mut() }; m::ext::tobj_toon = if rnd() & 1 != 0 { &raw mut g::TO[4] } else { null_mut() }; for q in 0..2usize { g::MO[q].mat = if rnd() % 5 != 0 { &raw mut g::MA[(rnd() & 1) as usize] } else { &raw mut g::MA[0] }; } for q in 0..2usize { g::MO[q].pe = if rnd() & 1 != 0 { &raw mut g::PE[(rnd() & 1) as usize] } else { null_mut() }; } for q in 0..3usize { g::DO[q].next = if q < 2 && (rnd() & 1) != 0 { &raw mut g::DO[q + 1] } else { null_mut() }; } for q in 0..3usize { g::DO[q].mobj = if rnd() % 4 != 0 { &raw mut g::MO[(rnd() & 1) as usize] } else { null_mut() }; } m::ext::HSD_GObj_804D7814 = &raw mut GO[(rnd() % 4) as usize]; if HDBG { println!("S {:08X} {:08X} {:08X} {:08X}", pi(g::MO[0].tobj), pi(g::MO[1].tobj), pi(m::ext::tobj_shadows), pi(m::ext::tobj_toon)); } }
}

unsafe fn hash_stem() {
    g::hashg_all();

g::hash_xpools(); g::hash_it_MObjInfo(&m::it_mobj); g::hash_GXColor(&GCOL[0]); g::hash_GXColor(&GCOL[1]); for q in 0..3usize { g::hash_HSD_TECnst(&TX[q].cnst); }
}

#[test]
fn itmaterial_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::freg_all();
        freg_world();
g::preg_xpools(); preg(&raw const TBOG, 16); preg(&raw const GCOL, 8); preg(&raw const TX, core::mem::size_of_val(&TX)); freg(cb_stev as usize);
        let mut c = Check::new(include_str!("data/mtu_itmaterial.txt"));
        let mut total = 0u32;
        for op in 0..5u32 {
            let (n, ft) = match op {
                0 => (300u32, false),
                1 => (300u32, false),
                2 => (300u32, false),
                3 => (300u32, false),
                4 => (300u32, false),
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
                setup_stem();
                let mut ret = 0u32;
                match op {
                    0 => { 
m::it_80277D08();
 }
                    1 => { let a0 = &raw mut g::MO[(rnd() & 1) as usize];
let a1 = fv();
let a2 = fv();
m::fn_80277D8C(a0, a1, a2);
 }
                    2 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
let a1 = &raw mut g::MO[(rnd() & 1) as usize];
let a2 = &raw mut TX[(rnd() % 3) as usize];
ret = pi(m::it_80277F90(a0, a1, a2));
 }
                    3 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
let a1 = &raw mut g::MO[(rnd() & 1) as usize];
let a2 = if rnd() & 1 != 0 { &raw mut TX[(rnd() % 3) as usize] } else { null_mut() };
m::it_80278108(a0, a1, a2);
 }
                    4 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut GCOL[(rnd() & 1) as usize];
m::it_80278574(a0, a1);
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
