//! Written by gen/mtu/mk_driver.py from the same description as gen/mtu/tu/it_2725.c.
//! src/tu/it_2725.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu/run_tu.py it_2725`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "mtu_w/mod.rs"]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::it_2725 as m;
use mtu_harness::*;
#[path = "mtu/it_2725_gen.rs"]
mod g;

static mut DAT: [[*mut c_void; 2]; 2] = [[null_mut(); 2]; 2];

unsafe fn setup_stem() {
    m::ext::HSD_GObjPLinkHead = (&raw mut PLINK) as *mut *mut HSD_GObj;
    m::ext::it_804D6D28 = &raw mut ICD[0];
{ g::setup_xpools(); for q in 0..2usize { DAT[q][0] = &raw mut FT[(rnd() % 3) as usize] as *mut c_void; DAT[q][1] = null_mut(); ART[q].x4_specialAttributes = (&raw mut DAT[q]) as *mut _; ART[q].xC_itemStates = &raw mut g::STA[q]; ART[q].x10_modelDesc = &raw mut g::MD[q]; g::MD[q].x4_bone_count = if rnd() % 3 == 0 { 1 } else { 0 }; g::MD[q].x8_bone_attach_id = if g::MD[q].x4_bone_count != 0 { (rnd() % 4) as i32 } else { (rnd() % 2) as i32 }; g::MD[q].xC_bit_field = ((if rnd() % 3 == 0 { 1u32 } else { 0 }) << 6 | (rnd() & 0x3F)) as u8; } (*m::ext::it_804D6D28).x44_float = (1 + fv() % 7) as _; for q in 0..8usize { JO[q].flags &= !(JOBJ_USE_QUATERNION as u32); } }
}

unsafe fn hash_stem() {
    g::hashg_all();
    hm(pi(m::ext::HSD_GObjPLinkHead));
    hm(pi(m::ext::it_804D6D28));
g::hash_xpools();
}

#[test]
fn it_2725_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::freg_all();
        freg_world();
g::preg_xpools(); preg(&raw const DAT, core::mem::size_of_val(&DAT));
        let mut c = Check::new(include_str!("data/mtu_it_2725.txt"));
        let mut total = 0u32;
        for op in 0..96u32 {
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
                82 => (300u32, false),
                83 => (300u32, false),
                84 => (300u32, false),
                85 => (300u32, false),
                86 => (300u32, false),
                87 => (300u32, false),
                88 => (300u32, false),
                89 => (300u32, false),
                90 => (300u32, false),
                91 => (300u32, false),
                92 => (300u32, false),
                93 => (300u32, false),
                94 => (300u32, false),
                95 => (300u32, false),
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
                setup_stem();
                let mut ret = 0u32;
                match op {
                    0 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = (fv() % 4) as i32;
m::it_80272560(a0, a1);
 }
                    1 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802725D4(a0);
 }
                    2 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = (fv() % 4) as i32;
m::it_80272674(a0, a1);
 }
                    3 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80272784(a0);
 }
                    4 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
ret = pi(m::it_80272818(a0));
 }
                    5 => { let a0 = fv() as _;
ret = pi(m::it_80272828(a0));
 }
                    6 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = rw();
let a2 = rw();
m::it_80272860(a0, a1, a2);
 }
                    7 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802728C8(a0);
 }
                    8 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80272940(a0);
 }
                    9 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80272980(a0);
 }
                    10 => { let a0 = &raw mut JO[(rnd() % 8) as usize];
m::it_80272A18(a0);
 }
                    11 => { let a0 = &raw mut JO[(rnd() % 8) as usize];
m::it_80272A3C(a0);
 }
                    12 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80272A60(a0);
 }
                    13 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
m::it_80272AC4(a0, a1);
 }
                    14 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80272B40(a0);
 }
                    15 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80272BA4(a0);
 }
                    16 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80272C08(a0);
 }
                    17 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_80272C6C(a0) as u32;
 }
                    18 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = pi(m::it_80272C90(a0));
 }
                    19 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = (fv() % 3) as _;
ret = pi(m::it_80272CC0(a0, a1));
 }
                    20 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_80272D1C(a0) as u32;
 }
                    21 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_80272D40(a0) as u32;
 }
                    22 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::itColl_BounceOffVictim(a0);
 }
                    23 => { let a0 = &raw mut JO[(rnd() % 8) as usize];
let a1 = rw();
m::it_80272DE4(a0, a1);
 }
                    24 => { let a0 = &raw mut JO[(rnd() % 8) as usize];
let a1 = rw();
m::it_80272F7C(a0, a1);
 }
                    25 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_80273030(a0) as u32;
 }
                    26 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::itColl_BounceOffShield(a0) as u32;
 }
                    27 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_80273130(a0) as u32;
 }
                    28 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80273168(a0);
 }
                    29 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802731A4(a0);
 }
                    30 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802731E0(a0);
 }
                    31 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8027321C(a0);
 }
                    32 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
let a2 = fv() as _;
m::it_8027327C(a0, a1, a2);
 }
                    33 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
let a1 = fv() as _;
m::it_802732E4(a0, a1);
 }
                    34 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut g::JT[(rnd() % 2) as usize];
m::it_80273318(a0, a1);
 }
                    35 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80273408(a0);
 }
                    36 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80273454(a0);
 }
                    37 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8027346C(a0);
 }
                    38 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80273484(a0);
 }
                    39 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8027349C(a0);
 }
                    40 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802734B4(a0);
 }
                    41 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
m::it_80273500(a0, a1);
 }
                    42 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
let a2 = fv() as _;
m::it_80273598(a0, a1, a2);
 }
                    43 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80273600(a0);
 }
                    44 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
let a2 = fv() as _;
m::it_80273648(a0, a1, a2);
 }
                    45 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = (fv() % 8) as i32;
let a2 = rw();
m::it_80273670(a0, a1, a2);
 }
                    46 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
let a2 = &raw mut VA[(rnd() % 4) as usize];
m::it_80273748(a0, a1, a2);
 }
                    47 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
m::it_80273B50(a0, a1);
 }
                    48 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut GO[(rnd() % 4) as usize];
m::it_80273F34(a0, a1);
 }
                    49 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
m::it_80274198(a0, a1);
 }
                    50 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
m::it_802741F4(a0, a1);
 }
                    51 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
m::it_80274250(a0, a1);
 }
                    52 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut VA[(rnd() % 4) as usize];
m::it_8027429C(a0, a1);
 }
                    53 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut GO[(rnd() % 4) as usize];
let a2 = fv() as _;
m::it_802742F4(a0, a1, a2);
 }
                    54 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut JO[(rnd() % 8) as usize];
let a2 = rw();
m::it_80274484(a0, a1, a2);
 }
                    55 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80274574(a0);
 }
                    56 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80274594(a0);
 }
                    57 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = rw();
ret = m::it_80274658(a0, a1) as u32;
 }
                    58 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = pi(m::it_802746F8(a0));
 }
                    59 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80274740(a0);
 }
                    60 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_80274990(a0).to_bits();
 }
                    61 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80274A64(a0);
 }
                    62 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80274C60(a0);
 }
                    63 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_80274C78(a0) as u32;
 }
                    64 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80274C88(a0);
 }
                    65 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80274CAC(a0);
 }
                    66 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut g::LBA[0];
m::it_80274D04(a0, a1);
 }
                    67 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80274D6C(a0);
 }
                    68 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80274DAC(a0);
 }
                    69 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80274DFC(a0);
 }
                    70 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80274E44(a0);
 }
                    71 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
m::it_80274ECC(a0, a1);
 }
                    72 => { 
m::it_80274ED8();
 }
                    73 => { 
m::it_80274EE8();
 }
                    74 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80274EF8(a0);
 }
                    75 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80274F10(a0);
 }
                    76 => { let a0 = &raw mut IT[(rnd() % 4) as usize];
let a1 = fv() as i8;
let a2 = unsafe { core::mem::transmute::<usize, _>(pick_cb(1)) };
let a3 = unsafe { core::mem::transmute::<usize, _>(pick_cb(3)) };
m::it_80274F28(a0, a1, a2, a3);
 }
                    77 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = (fv() % 4) as i32;
let a2 = &raw mut GO[(rnd() % 4) as usize];
let a3 = fv() as _;
m::it_80274F48(a0, a1, a2, a3);
 }
                    78 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = (fv() % 4) as i32;
let a2 = &raw mut GO[(rnd() % 4) as usize];
let a3 = fv() as _;
m::it_80274FDC(a0, a1, a2, a3);
 }
                    79 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = (fv() % 4) as i32;
m::it_80275070(a0, a1);
 }
                    80 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
ret = m::it_802750E8(a0, a1) as u32;
 }
                    81 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_802750F8(a0);
 }
                    82 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = rw();
m::it_80275158(a0, a1);
 }
                    83 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = rw();
m::it_80275174(a0, a1);
 }
                    84 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_8027518C(a0);
 }
                    85 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
ret = m::it_802751D8(a0) as u32;
 }
                    86 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80275210(a0);
 }
                    87 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80275228(a0);
 }
                    88 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80275240(a0);
 }
                    89 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80275258(a0);
 }
                    90 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80275270(a0);
 }
                    91 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
let a2 = fv() as _;
m::it_80275288(a0, a1, a2);
 }
                    92 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as _;
let a2 = fv() as _;
m::it_802752D8(a0, a1, a2);
 }
                    93 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = &raw mut g::LK[0];
m::it_80275328(a0, a1);
 }
                    94 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
m::it_80275390(a0);
 }
                    95 => { let a0 = &raw mut GO[(rnd() % 4) as usize];
let a1 = fv() as i16;
m::it_802753BC(a0, a1);
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
