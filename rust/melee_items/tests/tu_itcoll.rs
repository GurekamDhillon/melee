//! Written by gen/mtu/mk_driver.py from the same description as gen/mtu/tu/itcoll.c.
//! src/tu/itcoll.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu/run_tu.py itcoll`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itcoll as m;
use mtu_harness::*;
#[path = "mtu/itcoll_gen.rs"]
mod g;

static mut HBL: [ItHurtBoneList; 2] = unsafe { core::mem::zeroed() }; static mut HBD: [[ItHurtBoneDesc; 2]; 2] = unsafe { core::mem::zeroed() }; static mut DYN: [m::ItCollDynamics; 2] = unsafe { core::mem::zeroed() }; static mut DYD: [[m::ItCollDynamicsDesc; 2]; 2] = unsafe { core::mem::zeroed() }; static mut F6524: Fighter_804D6524_t = unsafe { core::mem::zeroed() };

unsafe fn setup_stem() {
    m::ext::HSD_GObjPLinkHead = (&raw mut PLINK) as *mut *mut HSD_GObj;
    m::ext::it_804D6D28 = &raw mut ICD[0];
{ F6524 = core::mem::zeroed(); F6524.x4 = rw(); m::ext::Fighter_804D6524 = &raw mut F6524; m::ext::it_804D6D18 = rnd() % 15; m::ext::Item_804A0CCC.count = (rnd() % 12) as usize; for q in 0..15usize { let r = rnd(); m::ext::it_804A0E70[q].x0 = if r & 1 != 0 { 2 } else { 1 }; m::ext::it_804A0E70[q].x4 = if r & 1 != 0 { (&raw mut IT[((r >> 4) % 4) as usize]) as *mut c_void } else { (&raw mut FT[((r >> 4) % 3) as usize]) as *mut c_void }; m::ext::it_804A0E70[q].x8 = pick_hit(); m::ext::it_804A0E70[q].xC = pick_hurt(); } for q in 0..4usize { IT[q].xAC4_ignoreItemID = rnd() % 3; IT[q].xAC8_hurtboxNum = (rnd() % 3) as u8; IT[q].xB68 = (rnd() % 3) as u8; for k in 0..4usize { IT[q].x5D4_hitboxes[k].hit.element = rnd() % 17; } } for q in 0..3usize { FT[q].hurt_capsules_len = (rnd() % 16) as u8; for k in 0..4usize { FT[q].x914[k].element = rnd() % 17; } } HBL = core::mem::zeroed(); HBD = core::mem::zeroed(); DYN = core::mem::zeroed(); DYD = core::mem::zeroed(); for q in 0..2usize { HBL[q].count = (rnd() % 3) as i32; HBL[q].descs = (&raw mut HBD[q]) as *mut ItHurtBoneDesc; DYN[q].count = (rnd() % 3) as i32; DYN[q].descs = (&raw mut DYD[q]) as *mut m::ItCollDynamicsDesc; for k in 0..2usize { HBD[q][k].bone_id = (rnd() % 4) as i32; HBD[q][k].a_offset.x = rw(); HBD[q][k].b_offset.y = rw(); HBD[q][k].scale = rw(); DYD[q][k].bone_id = (rnd() % 4) as i32; DYD[q][k].offset.x = rw(); DYD[q][k].size = rw(); } ART[q].x8_hurtbones = if rnd() & 1 != 0 { &raw mut HBL[q] } else { null_mut() }; ART[q].x14_dynamics = if rnd() & 1 != 0 { (&raw mut DYN[q]) as *mut ItemDynamics } else { null_mut() }; } }
}

unsafe fn hash_stem() {
    g::hashg_all();
    hm(pi(m::ext::HSD_GObjPLinkHead));
    hm(pi(m::ext::it_804D6D28));
hm(pi(m::ext::Fighter_804D6524));
}

#[test]
fn itcoll_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::freg_all();
        freg_world();
preg(&raw const HBL, core::mem::size_of_val(&HBL)); preg(&raw const HBD, core::mem::size_of_val(&HBD)); preg(&raw const DYN, core::mem::size_of_val(&DYN)); preg(&raw const DYD, core::mem::size_of_val(&DYD));
        let mut c = Check::new(include_str!("data/mtu_itcoll.txt"));
        let mut total = 0u32;
        for op in 0..38u32 {
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
                12 => (300u32, true),
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
                30 => (300u32, false),
                31 => (300u32, false),
                32 => (300u32, false),
                33 => (300u32, false),
                34 => (300u32, false),
                35 => (300u32, false),
                36 => (300u32, false),
                37 => (300u32, false),
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
                setup_stem();
                let mut ret = 0u32;
                match op {
                    0 => { let a0 = rw();
let a1 = rw();
let a2 = &raw mut EC[(rnd() % 3) as usize];
let a3 = &raw mut EC[(rnd() % 3) as usize];
let a4 = &raw mut VA[(rnd() % 4) as usize];
ret = m::itColl_chkECBOverlap(a0, a1, a2, a3, a4) as u32;
 }
                    1 => { 
m::it_8026F9A0();
 }
                    2 => { let a0 = fv() as _;
let a1 = (&raw mut FT[(rnd() % 3) as usize]) as *mut c_void;
let a2 = pick_hit();
let a3 = &raw mut IT[(rnd() % 4) as usize];
let a4 = pick_hurt();
m::it_8026F9AC(a0, a1, a2, a3, a4);
 }
                    3 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
let a1 = pick_hit();
let a2 = fv() as _;
let a3 = &raw mut IT[(rnd() % 4) as usize];
let a4 = fv() as _;
m::it_8026FA2C(a0, a1, a2, a3, a4);
 }
                    4 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
let a1 = pick_hit();
let a2 = fv() as _;
let a3 = (&raw mut IT[(rnd() % 4) as usize]) as *mut c_void;
let a4 = fv() as _;
m::it_8026FAC4(a0, a1, a2, a3, a4);
 }
                    5 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
let a1 = pick_hit();
let a2 = fv() as _;
let a3 = &raw mut FT[(rnd() % 3) as usize];
m::it_8026FC00_inline(a0, a1, a2, a3);
 }
                    6 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
let a1 = pick_hit();
let a2 = fv() as _;
let a3 = &raw mut FT[(rnd() % 3) as usize];
m::it_8026FC00(a0, a1, a2, a3);
 }
                    7 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
let a1 = pick_hit();
m::it_8026FCF8(a0, a1);
 }
                    8 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
let a1 = pick_hit();
let a2 = &raw mut IT[(rnd() % 4) as usize];
let a3 = pick_hit();
m::it_8026FE68(a0, a1, a2, a3);
 }
                    9 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
let a1 = pick_hit();
let a2 = fv() as _;
let a3 = (&raw mut IT[(rnd() % 4) as usize]) as *mut c_void;
let a4 = fv() as _;
m::it_8026FAC4_noinline(a0, a1, a2, a3, a4);
 }
                    10 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802701BC(a0);
 }
                    11 => { let a0 = fv() as _;
let a1 = (&raw mut FT[(rnd() % 3) as usize]) as *mut c_void;
let a2 = pick_hit();
let a3 = &raw mut IT[(rnd() % 4) as usize];
let a4 = pick_hurt();
m::it_8026F9AC_noinline(a0, a1, a2, a3, a4);
 }
                    12 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802703E8(a0);
 }
                    13 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
let a1 = &raw mut IT[(rnd() % 4) as usize];
let a2 = pick_hit();
let a3 = pick_hurt();
m::it_802706D0_sub3(a0, a1, a2, a3);
 }
                    14 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802706D0(a0);
 }
                    15 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
let a1 = pick_hit();
ret = m::it_80270CD8(a0, a1).to_bits();
 }
                    16 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80270E30(a0);
 }
                    17 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = (fv() & 3) as i32;
m::it_8027129C(a0, a1);
 }
                    18 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8027137C(a0);
 }
                    19 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8027146C(a0);
 }
                    20 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802714C0(a0);
 }
                    21 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
m::it_80271508(a0, a1);
 }
                    22 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = (fv() & 1) as i32;
let a2 = pick_hurt();
m::it_80271534(a0, a1, a2);
 }
                    23 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = (fv() & 1) as i32;
let a2 = pick_hurt();
m::it_80271590(a0, a1, a2);
 }
                    24 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8027163C(a0);
 }
                    25 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
let a1 = ((fv() % 200) as i32 - 100) as f32 * 0.37;
m::it_80271830(a0, a1);
 }
                    26 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80271A58(a0);
 }
                    27 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80271B60(a0);
 }
                    28 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80271D2C(a0);
 }
                    29 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80271F78(a0);
 }
                    30 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802721B8(a0);
 }
                    31 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80272280(a0);
 }
                    32 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80272298(a0);
 }
                    33 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802722B0(a0);
 }
                    34 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80272304(a0);
 }
                    35 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = pi(m::it_8027236C(a0));
 }
                    36 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = pi(m::it_802723FC(a0));
 }
                    37 => { let a0 = pick_hit();
let a1 = fv() as _;
let a2 = &raw mut GO[(rnd() % 4) as usize];
m::it_80272460(a0, a1, a2);
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
