//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/itwhitebea.c.
//! src/tu/itwhitebea.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py itwhitebea`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itwhitebea as m;
use mtu_harness::*;
#[path = "kinds_gen/itwhitebea_gen.rs"]
mod g;

#[repr(C, align(16))]
struct SpecBuf([[u8; 128]; 3]);
static mut GO: [HSD_GObj; 3] = unsafe { core::mem::zeroed() };
static mut IT: [Item; 3] = unsafe { core::mem::zeroed() };
static mut JO: [HSD_JObj; 48] = unsafe { core::mem::zeroed() };
static mut DBT: [DynamicBoneTable; 3] = unsafe { core::mem::zeroed() };
static mut ART: [Article; 3] = unsafe { core::mem::zeroed() };
static mut ATTR: [ItemAttr; 3] = unsafe { core::mem::zeroed() };
static mut ICD: ItemCommonData = unsafe { core::mem::zeroed() };
static mut SPECB: SpecBuf = SpecBuf([[0; 128]; 3]);
static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };

static mut D40: it_804D6D40_t = unsafe { core::mem::zeroed() };
static mut SFXA: [i32; 4] = [0; 4];

unsafe fn spec_fill(prof: i32) {
    match prof {
        0 => {
            let a = SPECB.0[0].as_mut_ptr() as *mut itWhiteBeaAttributes;
            (*a).x0 = SPECB.0[0].as_mut_ptr().add(64) as *mut _;
            (*(*a).x0).x0 = (rnd() % 20) as i32; (*(*a).x0).x4 = rw();
            (*a).x4 = rw(); (*a).x8 = (rnd() % 30) as i16; (*a).xA = (rnd() % 30) as i16; (*a).xC = (rnd() % 20) as i16;
            (*a).xE = ((*a).xC as i32 + (rnd() % 25) as i32 - 3) as i16; (*a).x10 = rw(); (*a).x14 = (rnd() % 50) as i16;
        }
        1 => {
            let a = SPECB.0[0].as_mut_ptr() as *mut itOldottoseaAttributes;
            (*a).x14 = (rnd() % 30) as f32; (*a).x18 = (rnd() % 40) as f32; (*a).x1C = rw(); (*a).x20 = rw();
        }
        _ => unreachable!(),
    }
}
unsafe fn setup(prof: i32) {
    SPECB = SpecBuf([[0; 128]; 3]);
    GO = core::mem::zeroed();
    IT = core::mem::zeroed();
    JO = core::mem::zeroed();
    DBT = core::mem::zeroed();
    ART = core::mem::zeroed();
    ATTR = core::mem::zeroed();
    ICD = core::mem::zeroed();
    VA = core::mem::zeroed();
    for k in 0..3 {
        g::fill_HSD_GObj(&mut GO[k]);
        g::fill_Item(&mut IT[k]);
        g::fill_ItemAttr(&mut ATTR[k]);
    }
    for i in 0..48 {
        g::fill_HSD_JObj(&mut JO[i]);
        JO[i].flags &= !(JOBJ_USE_QUATERNION as u32);
    }
    g::fill_ItemCommonData(&mut ICD);
    for i in 0..4 {
        VA[i].x = rw();
        VA[i].y = rw();
        VA[i].z = rw();
    }
    for k in 0..3 {
        GO[k].user_data = &raw mut IT[k] as *mut _;
        GO[k].hsd_obj = &raw mut JO[k] as *mut _;
        JO[k].child = &raw mut JO[3 + k];
        IT[k].entity = &raw mut GO[k];
        IT[k].xBBC_dynamicBoneTable = &raw mut DBT[k];
        IT[k].xC4_article_data = &raw mut ART[k];
        IT[k].xCC_item_attr = &raw mut ATTR[k];
        ART[k].x0_common_attr = &raw mut ATTR[k];
        ART[k].x4_specialAttributes = SPECB.0[k].as_mut_ptr() as *mut _;
        for i in 0..14 {
            DBT[k].bones[i] = &raw mut JO[6 + 14 * k + i];
        }
    }

    IT[0].owner = if rnd() & 3 != 0 { &raw mut GO[1] } else { null_mut() };
    IT[0].xC64_reflectGObj = if rnd() & 3 != 0 { &raw mut GO[1] } else { null_mut() };
    if rnd() & 1 != 0 {
        IT[0].msid = (rnd() % 14) as i32;
    }
    if rnd() & 1 != 0 {
        IT[0].facing_dir = if rnd() & 1 != 0 { 1.0 } else { -1.0 };
    }
    IT[0].ground_or_air = if rnd() & 1 != 0 { GA_Air } else { GA_Ground };
    spec_fill(prof);
    D40 = core::mem::zeroed(); D40.x8 = (rnd() % 11) as f32 / 10.0; m::ext::it_804D6D40 = &raw mut D40;
    SFXA[0] = -1; for i in 1..4 { SFXA[i] = (rnd() % 500) as i32 - 1; }
    for k in 0..3 { IT[k].xDD4_itemVar.whitebea.x20 = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() }; }
}

unsafe fn hash_all() {
    for k in 0..3 { hm(pi(IT[k].xDD4_itemVar.whitebea.x20)); IT[k].xDD4_itemVar.whitebea.x20 = null_mut(); }
    for k in 0..3 {
        g::hash_HSD_GObj(&GO[k]);
        g::hash_Item(&IT[k]);
        g::hash_ItemAttr(&ATTR[k]);
    }
    for i in 0..48 {
        g::hash_HSD_JObj(&JO[i]);
    }
    for i in 0..4 {
        hm(VA[i].x.to_bits());
        hm(VA[i].y.to_bits());
        hm(VA[i].z.to_bits());
    }
}

#[test]
fn itwhitebea_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        g::freg_all();
        const PROFTAB: [u8; 73] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_itwhitebea.txt"));
        for op in 0..73u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => {  m::it_802E31F8(&raw mut GO[0]); } // it_802E31F8
                    1 => {  ret = m::itOldottosea_UnkMotion8_Anim(&raw mut GO[0]) as u32; } // itOldottosea_UnkMotion8_Anim
                    2 => {  m::itOldottosea_UnkMotion8_Phys(&raw mut GO[0]); } // itOldottosea_UnkMotion8_Phys
                    3 => {  m::it_802E32B4(&raw mut GO[0]); } // it_802E32B4
                    4 => {  m::it_802E3314(&raw mut GO[0]); } // it_802E3314
                    5 => {  ret = m::itOldottosea_UnkMotion9_Anim(&raw mut GO[0]) as u32; } // itOldottosea_UnkMotion9_Anim
                    6 => {  m::itOldottosea_UnkMotion9_Phys(&raw mut GO[0]); } // itOldottosea_UnkMotion9_Phys
                    7 => {  ret = m::itOldottosea_UnkMotion9_Coll(&raw mut GO[0]) as u32; } // itOldottosea_UnkMotion9_Coll
                    8 => {  m::it_802E3400(&raw mut GO[0]); } // it_802E3400
                    9 => {  ret = m::itOldottosea_UnkMotion10_Anim(&raw mut GO[0]) as u32; } // itOldottosea_UnkMotion10_Anim
                    10 => {  m::itOldottosea_UnkMotion10_Phys(&raw mut GO[0]); } // itOldottosea_UnkMotion10_Phys
                    11 => {  ret = m::itOldottosea_UnkMotion10_Coll(&raw mut GO[0]) as u32; } // itOldottosea_UnkMotion10_Coll
                    12 => {  m::it_2725_Logic3_Destroyed(&raw mut GO[0]); } // it_2725_Logic3_Destroyed
                    13 => {  m::it_802E3528(&raw mut GO[0]); } // it_802E3528
                    14 => {  ret = m::itOldottosea_UnkMotion11_Anim(&raw mut GO[0]) as u32; } // itOldottosea_UnkMotion11_Anim
                    15 => {  m::itOldottosea_UnkMotion11_Phys(&raw mut GO[0]); } // itOldottosea_UnkMotion11_Phys
                    16 => {  ret = m::itOldottosea_UnkMotion11_Coll(&raw mut GO[0]) as u32; } // itOldottosea_UnkMotion11_Coll
                    17 => {  ret = m::it_802E35CC(&raw mut GO[0]) as u32; } // it_802E35CC
                    18 => {  m::it_802E3784(&raw mut GO[0], &raw mut GO[1]); } // it_802E3784
                    19 => {  m::it_802E37A4(&raw mut GO[0]); } // it_802E37A4
                    20 => {  m::it_802E37BC(&raw mut GO[0]); } // it_802E37BC
                    21 => {  ret = m::it_802E3884(&raw mut GO[0]) as u32; } // it_802E3884
                    22 => {  m::fn_802E398C(&raw mut GO[0]); } // fn_802E398C
                    23 => {  ret = m::itWhitebea_UnkMotion0_Anim(&raw mut GO[0]) as u32; } // itWhitebea_UnkMotion0_Anim
                    24 => {  m::itWhitebea_UnkMotion0_Phys(&raw mut GO[0]); } // itWhitebea_UnkMotion0_Phys
                    25 => {  ret = m::itWhitebea_UnkMotion0_Coll(&raw mut GO[0]) as u32; } // itWhitebea_UnkMotion0_Coll
                    26 => {  m::it_802E3AC8(&raw mut GO[0]); } // it_802E3AC8
                    27 => {  ret = m::itWhitebea_UnkMotion1_Anim(&raw mut GO[0]) as u32; } // itWhitebea_UnkMotion1_Anim
                    28 => {  m::itWhitebea_UnkMotion1_Phys(&raw mut GO[0]); } // itWhitebea_UnkMotion1_Phys
                    29 => {  ret = m::itWhitebea_UnkMotion1_Coll(&raw mut GO[0]) as u32; } // itWhitebea_UnkMotion1_Coll
                    30 => {  m::it_802E3DA0(&raw mut GO[0]); } // it_802E3DA0
                    31 => {  ret = m::itWhitebea_UnkMotion3_Anim(&raw mut GO[0]) as u32; } // itWhitebea_UnkMotion3_Anim
                    32 => {  m::itWhitebea_UnkMotion3_Phys(&raw mut GO[0]); } // itWhitebea_UnkMotion3_Phys
                    33 => {  ret = m::itWhitebea_UnkMotion3_Coll(&raw mut GO[0]) as u32; } // itWhitebea_UnkMotion3_Coll
                    34 => {  m::it_802E3ED0(&raw mut GO[0]); } // it_802E3ED0
                    35 => {  ret = m::itWhitebea_UnkMotion4_Anim(&raw mut GO[0]) as u32; } // itWhitebea_UnkMotion4_Anim
                    36 => {  m::itWhitebea_UnkMotion4_Phys(&raw mut GO[0]); } // itWhitebea_UnkMotion4_Phys
                    37 => {  ret = m::itWhitebea_UnkMotion4_Coll(&raw mut GO[0]) as u32; } // itWhitebea_UnkMotion4_Coll
                    38 => {  m::it_802E40A4(&raw mut GO[0]); } // it_802E40A4
                    39 => {  m::it_802E4190(&raw mut GO[0]); } // it_802E4190
                    40 => {  ret = m::itWhitebea_UnkMotion2_Anim(&raw mut GO[0]) as u32; } // itWhitebea_UnkMotion2_Anim
                    41 => {  m::itWhitebea_UnkMotion2_Phys(&raw mut GO[0]); } // itWhitebea_UnkMotion2_Phys
                    42 => {  ret = m::itWhitebea_UnkMotion2_Coll(&raw mut GO[0]) as u32; } // itWhitebea_UnkMotion2_Coll
                    43 => {  m::it_802E436C(&raw mut GO[0]); } // it_802E436C
                    44 => {  ret = m::itWhitebea_UnkMotion5_Anim(&raw mut GO[0]) as u32; } // itWhitebea_UnkMotion5_Anim
                    45 => {  m::itWhitebea_UnkMotion5_Phys(&raw mut GO[0]); } // itWhitebea_UnkMotion5_Phys
                    46 => {  ret = m::itWhitebea_UnkMotion5_Coll(&raw mut GO[0]) as u32; } // itWhitebea_UnkMotion5_Coll
                    47 => {  m::it_802E4464(&raw mut GO[0]); } // it_802E4464
                    48 => {  ret = m::itWhitebea_UnkMotion6_Anim(&raw mut GO[0]) as u32; } // itWhitebea_UnkMotion6_Anim
                    49 => {  m::itWhitebea_UnkMotion6_Phys(&raw mut GO[0]); } // itWhitebea_UnkMotion6_Phys
                    50 => {  ret = m::itWhitebea_UnkMotion6_Coll(&raw mut GO[0]) as u32; } // itWhitebea_UnkMotion6_Coll
                    51 => {  m::it_802E4558(&raw mut GO[0]); } // it_802E4558
                    52 => {  ret = m::itWhitebea_UnkMotion7_Anim(&raw mut GO[0]) as u32; } // itWhitebea_UnkMotion7_Anim
                    53 => {  m::itWhitebea_UnkMotion7_Phys(&raw mut GO[0]); } // itWhitebea_UnkMotion7_Phys
                    54 => {  ret = m::itWhitebea_UnkMotion7_Coll(&raw mut GO[0]) as u32; } // itWhitebea_UnkMotion7_Coll
                    55 => {  m::itWhiteBea_Logic9_PickedUp(&raw mut GO[0]); } // itWhiteBea_Logic9_PickedUp
                    56 => {  ret = m::itWhitebea_UnkMotion8_Anim(&raw mut GO[0]) as u32; } // itWhitebea_UnkMotion8_Anim
                    57 => {  m::itWhitebea_UnkMotion8_Phys(&raw mut GO[0]); } // itWhitebea_UnkMotion8_Phys
                    58 => {  m::it_2725_Logic9_Dropped(&raw mut GO[0]); } // it_2725_Logic9_Dropped
                    59 => {  m::it_2725_Logic9_Thrown(&raw mut GO[0]); } // it_2725_Logic9_Thrown
                    60 => {  ret = m::itWhitebea_UnkMotion9_Anim(&raw mut GO[0]) as u32; } // itWhitebea_UnkMotion9_Anim
                    61 => {  m::itWhitebea_UnkMotion9_Phys(&raw mut GO[0]); } // itWhitebea_UnkMotion9_Phys
                    62 => {  ret = m::itWhitebea_UnkMotion9_Coll(&raw mut GO[0]) as u32; } // itWhitebea_UnkMotion9_Coll
                    63 => {  m::it_802E48B4(&raw mut GO[0]); } // it_802E48B4
                    64 => {  ret = m::itWhitebea_UnkMotion10_Anim(&raw mut GO[0]) as u32; } // itWhitebea_UnkMotion10_Anim
                    65 => {  m::itWhitebea_UnkMotion10_Phys(&raw mut GO[0]); } // itWhitebea_UnkMotion10_Phys
                    66 => {  ret = m::itWhitebea_UnkMotion10_Coll(&raw mut GO[0]) as u32; } // itWhitebea_UnkMotion10_Coll
                    67 => {  m::it_802E4980(&raw mut GO[0]); } // it_802E4980
                    68 => {  ret = m::itWhitebea_UnkMotion11_Anim(&raw mut GO[0]) as u32; } // itWhitebea_UnkMotion11_Anim
                    69 => {  m::itWhitebea_UnkMotion11_Phys(&raw mut GO[0]); } // itWhitebea_UnkMotion11_Phys
                    70 => {  ret = m::itWhitebea_UnkMotion11_Coll(&raw mut GO[0]) as u32; } // itWhitebea_UnkMotion11_Coll
                    71 => {  m::it_802E4A24(&raw mut GO[0], &raw mut GO[1]); } // it_802E4A24
                    72 => { for j in 0..12 { let row = &m::it_803F8A88[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 29200);
    }
}
