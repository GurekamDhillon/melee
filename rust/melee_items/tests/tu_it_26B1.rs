//! Written by gen/mtu/mk_driver.py from the same description as gen/mtu/tu/it_26B1.c.
//! src/tu/it_26B1.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu/run_tu.py it_26B1`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "mtu_w/mod.rs"]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::it_26B1 as m;
use mtu_harness::*;
#[path = "mtu/it_26B1_gen.rs"]
mod g;

static mut ARR: [*mut Article; 256] = [null_mut(); 256]; static mut ARR2: [*mut Article; 256] = [null_mut(); 256]; static mut SPA: [[u32; 8]; 2] = [[0; 8]; 2]; static KINDS: [i32; 28] = [It_Kind_Heart as i32, It_Kind_Tomato as i32, It_Kind_Foods as i32, It_Kind_Coin as i32, It_Kind_Lucky_Egg as i32, It_Kind_WhispyHealApple as i32, It_Kind_L_Gun as i32, It_Kind_S_Scope as i32, It_Kind_StarRod as i32, It_Kind_LipStick as i32, It_Kind_F_Flower as i32, It_Kind_Harisen as i32, It_Kind_Freeze as i32, It_Kind_MSBomb as i32, It_Kind_Flipper as i32, It_Kind_BombHei as i32, It_Kind_Link_Bomb as i32, It_Kind_CLink_Bomb as i32, It_Kind_Kyasarin_Egg as i32, It_Kind_WhispyApple as i32, It_Kind_Kuriboh as i32, It_Kind_Octarock_Stone as i32, It_Kind_Old_Kuri as i32, It_Kind_Arwing_Laser as i32, It_Kind_Box as i32, It_Kind_Kusudama as i32, It_Kind_Nokonoko as i32, It_PKind_Marumine as i32];

unsafe fn setup_stem() {
    m::ext::HSD_GObjPLinkHead = (&raw mut PLINK) as *mut *mut HSD_GObj;
    m::ext::it_804D6D28 = &raw mut ICD[0];
{ g::setup_xpools(); for q in 0..256usize { ARR[q] = &raw mut ART[q & 1]; ARR2[q] = if rnd() & 3 != 0 { null_mut() } else { &raw mut ART[q & 1] }; } m::ext::it_804D6D24 = (&raw mut ARR) as *mut *mut Article; m::ext::it_804D6D38 = (&raw mut ARR2) as *mut *mut Article; for q in 0..2usize { for k in 0..8usize { SPA[q][k] = if k & 1 != 0 { fv() } else { rw().to_bits() }; } ART[q].x4_specialAttributes = (&raw mut SPA[q]) as *mut _; ART[q].xC_itemStates = &raw mut g::STA[q]; } for q in 0..2usize { const BL: [i32; 11] = [1, 20, 8, 9, 4, 12, 21, 25, 31, 11, 6]; g::BOR[q].x14 = if rnd() % 4 == 0 { fv() as i32 } else { BL[(rnd() % 11) as usize] }; } for q in 0..4usize { if rnd() % 4 != 0 { IT[q].kind = KINDS[(rnd() % 28) as usize] as _; } if rnd() % 3 != 0 { IT[q].hold_kind = (4 + rnd() % 4) as _; } IT[q].anim_id = if rnd() % 4 == 0 { -1 } else { (rnd() % 8) as _ }; } for q in 0..8usize { JO[q].flags &= !(JOBJ_USE_QUATERNION as u32); } }
}

unsafe fn hash_stem() {
    g::hashg_all();
    hm(pi(m::ext::HSD_GObjPLinkHead));
    hm(pi(m::ext::it_804D6D28));
g::hash_xpools(); for q in 0..256usize { hm(pi(ARR2[q])); } hm(rh(&SPA));
}

#[test]
fn it_26B1_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::freg_all();
        freg_world();
g::preg_xpools(); preg(&raw const ARR, core::mem::size_of_val(&ARR)); preg(&raw const ARR2, core::mem::size_of_val(&ARR2)); preg(&raw const SPA, core::mem::size_of_val(&SPA));
        let mut c = Check::new(include_str!("data/mtu_it_26B1.txt"));
        let mut total = 0u32;
        for op in 0..82u32 {
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
                30 => (300u32, false),
                31 => (300u32, false),
                32 => (300u32, false),
                33 => (300u32, false),
                34 => (300u32, false),
                35 => (300u32, false),
                36 => (300u32, false),
                37 => (300u32, false),
                38 => (300u32, false),
                39 => (300u32, false),
                40 => (300u32, false),
                41 => (300u32, false),
                42 => (300u32, false),
                43 => (300u32, false),
                44 => (300u32, false),
                45 => (300u32, false),
                46 => (300u32, false),
                47 => (300u32, false),
                48 => (300u32, false),
                49 => (300u32, false),
                50 => (300u32, false),
                51 => (300u32, false),
                52 => (300u32, false),
                53 => (300u32, false),
                54 => (300u32, false),
                55 => (300u32, false),
                56 => (300u32, false),
                57 => (300u32, false),
                58 => (300u32, false),
                59 => (300u32, false),
                60 => (300u32, false),
                61 => (300u32, false),
                62 => (300u32, false),
                63 => (300u32, false),
                64 => (300u32, false),
                65 => (300u32, false),
                66 => (300u32, false),
                67 => (300u32, false),
                68 => (300u32, false),
                69 => (300u32, false),
                70 => (300u32, false),
                71 => (300u32, false),
                72 => (300u32, false),
                73 => (300u32, false),
                74 => (300u32, false),
                75 => (300u32, false),
                76 => (300u32, false),
                77 => (300u32, false),
                78 => (300u32, false),
                79 => (300u32, false),
                80 => (300u32, false),
                81 => (300u32, false),
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
                setup_stem();
                let mut ret = 0u32;
                match op {
                    0 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = pick_hit();
ret = m::it_8026B1D4(a0, a1).to_bits();
 }
                    1 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
m::it_8026B294(a0, a1);
 }
                    2 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itIsHeavy(a0) as u32;
 }
                    3 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026B2D8(a0) as u32;
 }
                    4 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itGetKind(a0) as u32;
 }
                    5 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026B30C(a0) as u32;
 }
                    6 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itGetHoldKind(a0) as u32;
 }
                    7 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itGetDamageMultiplier(a0).to_bits();
 }
                    8 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
m::it_8026B344(a0, a1);
 }
                    9 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itGetGrabRangeX(a0).to_bits();
 }
                    10 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itGetGrabRangeY(a0).to_bits();
 }
                    11 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8026B390(a0);
 }
                    12 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8026B3A8(a0);
 }
                    13 => { let a0 = KINDS[(rnd() % 28) as usize] as _;
ret = m::it_8026B3C0(a0) as u32;
 }
                    14 => { let a0 = &raw mut ART[(rnd() % 2) as usize];
let a1 = (It_Kind_Kuriboh as u32 + fv() % 200) as i32;
m::it_8026B3F8(a0, a1);
 }
                    15 => { let a0 = &raw mut ART[(rnd() % 2) as usize];
let a1 = (It_Kind_Kuriboh as u32 + fv() % 200) as i32;
m::it_Mex_RegisterIfEmpty(a0, a1);
 }
                    16 => { 
m::it_ResetStageArticleTable();
 }
                    17 => { 
ret = m::Item_StageArticleTableTest() as u32;
 }
                    18 => { let a0 = &raw mut ART[(rnd() % 2) as usize];
let a1 = (if rnd() % 6 == 0 { 5000 + fv() % 10 } else { It_Kind_Old_Kuri as u32 + fv() % 30 }) as i32;
m::it_8026B40C(a0, a1);
 }
                    19 => { let a0 = fv() as _;
ret = m::it_8026B424(a0).to_bits();
 }
                    20 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026B47C(a0) as u32;
 }
                    21 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026B4F0(a0) as u32;
 }
                    22 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026B54C(a0).to_bits();
 }
                    23 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026B560(a0).to_bits();
 }
                    24 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026B574(a0).to_bits();
 }
                    25 => { 
ret = m::it_8026B588() as u32;
 }
                    26 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026B594(a0) as u32;
 }
                    27 => { let a0 = &raw mut VA[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = &raw mut GO[(rnd() % 4) as usize];
ret = pi(m::it_8026B5E4(a0, a1, a2));
 }
                    28 => { let a0 = &raw mut VA[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = &raw mut GO[(rnd() % 4) as usize];
let a3 = rw();
ret = pi(m::it_8026B634(a0, a1, a2, a3));
 }
                    29 => { let a0 = &raw mut VA[(rnd() % 4) as usize];
ret = m::it_8026B684(a0).to_bits();
 }
                    30 => { let a0 = &raw mut VA[(rnd() % 4) as usize];
let a1 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026B6A8(a0, a1).to_bits();
 }
                    31 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026B6C8(a0) as u32;
 }
                    32 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = rw();
m::it_8026B718(a0, a1);
 }
                    33 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8026B724(a0);
 }
                    34 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8026B73C(a0);
 }
                    35 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
ret = m::it_8026B774(a0, a1) as u32;
 }
                    36 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itGetMotionId(a0) as u32;
 }
                    37 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itGetTeamId(a0) as u32;
 }
                    38 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026B7BC(a0) as u32;
 }
                    39 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026B7CC(a0) as u32;
 }
                    40 => { 
ret = m::it_8026B7D8() as u32;
 }
                    41 => { 
ret = m::it_8026B7E0() as u32;
 }
                    42 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026B7E8(a0) as u32;
 }
                    43 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8026B7F8(a0);
 }
                    44 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026B894(a0, a1) as u32;
 }
                    45 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026B924(a0) as u32;
 }
                    46 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026B960(a0).to_bits();
 }
                    47 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut GO[(rnd() % 4) as usize];
let a2 = fv() as _;
m::it_8026B9A8(a0, a1, a2);
 }
                    48 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = rw();
m::it_8026BAE8(a0, a1);
 }
                    49 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8026BB20(a0);
 }
                    50 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8026BB44(a0);
 }
                    51 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
m::it_8026BB68(a0, a1);
 }
                    52 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
m::it_8026BB88(a0, a1);
 }
                    53 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
m::it_8026BBCC(a0, a1);
 }
                    54 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8026BC14(a0);
 }
                    55 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026BC68(a0) as u32;
 }
                    56 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = pi(m::itGetOwner(a0));
 }
                    57 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itGetAttackId(a0) as u32;
 }
                    58 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
m::it_8026BC90(a0, a1);
 }
                    59 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8026BCF4(a0);
 }
                    60 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8026BD0C(a0);
 }
                    61 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8026BD24(a0);
 }
                    62 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8026BD3C(a0);
 }
                    63 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8026BD54(a0);
 }
                    64 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8026BD6C(a0);
 }
                    65 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8026BD84(a0);
 }
                    66 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8026BD9C(a0);
 }
                    67 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8026BDB4(a0);
 }
                    68 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8026BDCC(a0);
 }
                    69 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8026BE28(a0);
 }
                    70 => { let a0 = &raw mut g::BOR[(rnd() & 1) as usize];
ret = pi(m::it_8026BE84(a0));
 }
                    71 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = pi(m::it_8026C100(a0));
 }
                    72 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
m::it_8026C16C(a0, a1);
 }
                    73 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026C1B4(a0) as u32;
 }
                    74 => { 
ret = m::it_8026C1D4() as u32;
 }
                    75 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_8026C1E8(a0) as u32;
 }
                    76 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut GO[(rnd() % 4) as usize];
m::it_8026C220(a0, a1);
 }
                    77 => { let a0 = &raw mut VA[(rnd() % 4) as usize];
let a1 = rw();
ret = pi(m::it_8026C258(a0, a1));
 }
                    78 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
m::it_8026C334(a0, a1);
 }
                    79 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8026C368(a0);
 }
                    80 => { 
m::it_8026C3FC();
 }
                    81 => { 
m::it_8026C42C();
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
