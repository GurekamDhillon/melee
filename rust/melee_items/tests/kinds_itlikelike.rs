//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/itlikelike.c.
//! src/tu/itlikelike.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py itlikelike`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itlikelike as m;
use mtu_harness::*;
#[path = "kinds_gen/itlikelike_gen.rs"]
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

static mut LLX: [Vec3; 3] = unsafe { core::mem::zeroed() }; static mut D40: it_804D6D40_t = unsafe { core::mem::zeroed() };

unsafe fn spec_fill(prof: i32) {
    match prof {
        0 => {
            for q in 0..3usize {
            let a = SPECB.0[q].as_mut_ptr() as *mut itLikelikeAttributes;
            *(&raw mut LLX[q].x as *mut i32) = (rnd() % 40) as i32; LLX[q].y = rw(); LLX[q].z = rw();
            (*a).x0 = &raw mut LLX[q] as *mut itSpecialAttrsHead; (*a).x4 = (rnd() % 20) as i32; (*a).x8 = (rnd() % 10) as i32; (*a).xC = (rnd() % 14) as i32 + 1;
            (*a).x18 = (rnd() % 4) as i32; (*a).x1C = (rnd() % 30) as f32; (*a).x20 = (rnd() % 30) as f32; (*a).x24 = (rnd() % 30) as f32;
            (*a).x28 = (rnd() % 30) as f32; (*a).x2C = (rnd() % 30) as f32; (*a).x30 = rw(); (*a).x34 = rw(); (*a).x38 = (rnd() % 45) as f32;
            (*a).x3C = (rnd() % 3) as i8; (*a).x3D = (rnd() % 4) as i8; (*a).x3E = (rnd() % 4) as i8;
            }
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
        m::ext::it_804D6D40 = &raw mut D40; D40.x8 = (rnd() % 1000) as f32 / 1000.0;
    for k in 0..3 {
        IT[k].xDD4_itemVar.likelike.x18 = (rnd() % 4) as i32;
        IT[k].xDD4_itemVar.likelike.x20.x = rw(); IT[k].xDD4_itemVar.likelike.x20.y = rw(); IT[k].xDD4_itemVar.likelike.x20.z = rw();
        IT[k].xDD4_itemVar.likelike.x38 = (rnd() % 3) as i32;
        IT[k].xDD4_itemVar.likelike.x3C = if rnd() & 1 != 0 { 1 } else if rnd() & 1 != 0 { -1 } else { 0 };
        IT[k].xDD4_itemVar.likelike.x44 = (rnd() % 0x40) as i32 - 2;
        IT[k].xDD4_itemVar.likelike.x48 = (rnd() % 7) as i32;
        IT[k].xDD4_itemVar.likelike.x4C = (rnd() % 8) as i32 - 2;
        IT[k].xDD4_itemVar.likelike.x50 = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        IT[k].grab_victim = if rnd() & 3 == 0 { IT[k].xDD4_itemVar.likelike.x50 } else if rnd() & 1 != 0 { &raw mut GO[(k + 2) % 3] } else { null_mut() };
        IT[k].atk_victim = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        IT[k].xC9C = (rnd() % 60) as i32;
        IT[k].x378_itemColl.ecb.top.y = rw(); IT[k].x378_itemColl.ecb.bottom.y = rw();
    }
}

unsafe fn hash_all() {
    for k in 0..3 { hm(pi(IT[k].xDD4_itemVar.likelike.x50)); IT[k].xDD4_itemVar.likelike.x50 = null_mut(); }
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
fn itlikelike_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        g::freg_all();
        const PROFTAB: [u8; 91] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_itlikelike.txt"));
        for op in 0..91u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => {  ret = m::it_802D9A2C(&raw mut GO[0]) as u32; } // it_802D9A2C
                    1 => {  m::it_802D9B78(&raw mut GO[0]); } // it_802D9B78
                    2 => {  m::it_802D9BA8(&raw mut GO[0]); } // it_802D9BA8
                    3 => {  ret = m::it_2725_Logic5_DmgReceived(&raw mut GO[0]) as u32; } // it_2725_Logic5_DmgReceived
                    4 => {  m::it_802D9DDC(&raw mut GO[0]); } // it_802D9DDC
                    5 => {  ret = m::itLikelike_UnkMotion0_Anim(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion0_Anim
                    6 => {  m::itLikelike_UnkMotion0_Phys(&raw mut GO[0]); } // itLikelike_UnkMotion0_Phys
                    7 => {  ret = m::itLikelike_UnkMotion0_Coll(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion0_Coll
                    8 => {  ret = m::itLikelike_UnkMotion7_Anim(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion7_Anim
                    9 => {  m::itLikelike_UnkMotion7_Phys(&raw mut GO[0]); } // itLikelike_UnkMotion7_Phys
                    10 => {  ret = m::itLikelike_UnkMotion7_Coll(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion7_Coll
                    11 => {  m::it_802DA104(&raw mut GO[0]); } // it_802DA104
                    12 => {  ret = m::itLikelike_UnkMotion1_Anim(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion1_Anim
                    13 => {  m::itLikelike_UnkMotion1_Phys(&raw mut GO[0]); } // itLikelike_UnkMotion1_Phys
                    14 => {  ret = m::itLikelike_UnkMotion1_Coll(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion1_Coll
                    15 => {  m::it_802DA4C0(&raw mut GO[0]); } // it_802DA4C0
                    16 => {  ret = m::itLikelike_UnkMotion2_Anim(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion2_Anim
                    17 => {  m::itLikelike_UnkMotion2_Phys(&raw mut GO[0]); } // itLikelike_UnkMotion2_Phys
                    18 => {  ret = m::itLikelike_UnkMotion2_Coll(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion2_Coll
                    19 => {  m::it_802DA8D8(&raw mut GO[0]); } // it_802DA8D8
                    20 => {  m::it_802DA960(&raw mut GO[0]); } // it_802DA960
                    21 => {  m::it_802DAA10(&raw mut GO[0]); } // it_802DAA10
                    22 => {  ret = m::itLikelike_UnkMotion5_Anim(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion5_Anim
                    23 => {  m::itLikelike_UnkMotion5_Phys(&raw mut GO[0]); } // itLikelike_UnkMotion5_Phys
                    24 => {  ret = m::itLikelike_UnkMotion4_Coll(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion4_Coll
                    25 => {  ret = m::itLikelike_UnkMotion5_Coll(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion5_Coll
                    26 => {  m::it_802DABC0(&raw mut GO[0]); } // it_802DABC0
                    27 => {  ret = m::itLikelike_UnkMotion17_Anim(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion17_Anim
                    28 => {  m::itLikelike_UnkMotion17_Phys(&raw mut GO[0]); } // itLikelike_UnkMotion17_Phys
                    29 => {  ret = m::itLikelike_UnkMotion17_Coll(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion17_Coll
                    30 => {  m::it_802DAD18(&raw mut GO[0]); } // it_802DAD18
                    31 => {  ret = m::itLikelike_UnkMotion3_Anim(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion3_Anim
                    32 => {  m::itLikelike_UnkMotion3_Phys(&raw mut GO[0]); } // itLikelike_UnkMotion3_Phys
                    33 => {  ret = m::itLikelike_UnkMotion3_Coll(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion3_Coll
                    34 => {  m::it_802DAE6C(&raw mut GO[0]); } // it_802DAE6C
                    35 => {  ret = m::itLikelike_UnkMotion16_Anim(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion16_Anim
                    36 => {  m::itLikelike_UnkMotion16_Phys(&raw mut GO[0]); } // itLikelike_UnkMotion16_Phys
                    37 => {  ret = m::itLikelike_UnkMotion16_Coll(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion16_Coll
                    38 => {  m::it_802DB074(&raw mut GO[0]); } // it_802DB074
                    39 => {  ret = m::itLikelike_UnkMotion8_Anim(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion8_Anim
                    40 => {  m::itLikelike_UnkMotion8_Phys(&raw mut GO[0]); } // itLikelike_UnkMotion8_Phys
                    41 => {  ret = m::itLikelike_UnkMotion8_Coll(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion8_Coll
                    42 => {  m::it_802DB358(&raw mut GO[0]); } // it_802DB358
                    43 => {  m::it_802DB398(&raw mut GO[0]); } // it_802DB398
                    44 => {  ret = m::itLikelike_UnkMotion12_Anim(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion12_Anim
                    45 => {  m::itLikelike_UnkMotion12_Phys(&raw mut GO[0]); } // itLikelike_UnkMotion12_Phys
                    46 => {  ret = m::itLikelike_UnkMotion12_Coll(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion12_Coll
                    47 => {  m::it_802DB5F0(&raw mut GO[0]); } // it_802DB5F0
                    48 => {  ret = m::itLikelike_UnkMotion9_Anim(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion9_Anim
                    49 => {  m::itLikelike_UnkMotion9_Phys(&raw mut GO[0]); } // itLikelike_UnkMotion9_Phys
                    50 => {  ret = m::itLikelike_UnkMotion9_Coll(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion9_Coll
                    51 => {  m::it_802DB74C(&raw mut GO[0]); } // it_802DB74C
                    52 => {  ret = m::itLikelike_UnkMotion10_Anim(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion10_Anim
                    53 => {  m::itLikelike_UnkMotion10_Phys(&raw mut GO[0]); } // itLikelike_UnkMotion10_Phys
                    54 => {  ret = m::itLikelike_UnkMotion10_Coll(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion10_Coll
                    55 => {  m::it_802DB8A8(&raw mut GO[0]); } // it_802DB8A8
                    56 => {  ret = m::itLikelike_UnkMotion13_Anim(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion13_Anim
                    57 => {  m::itLikelike_UnkMotion13_Phys(&raw mut GO[0]); } // itLikelike_UnkMotion13_Phys
                    58 => {  ret = m::itLikelike_UnkMotion13_Coll(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion13_Coll
                    59 => {  m::it_802DB9F4(&raw mut GO[0]); } // it_802DB9F4
                    60 => {  m::it_802DBA68(&raw mut GO[0]); } // it_802DBA68
                    61 => { let v1: i32 = (rnd() % 3) as i32 - 1 as i32; let v2: i32 = (rnd() % 3) as i32 as i32; m::it_802DBAF0(&raw mut GO[0], v1, v2); } // it_802DBAF0
                    62 => {  ret = m::itLikelike_UnkMotion14_Anim(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion14_Anim
                    63 => {  m::itLikelike_UnkMotion14_Phys(&raw mut GO[0]); } // itLikelike_UnkMotion14_Phys
                    64 => {  ret = m::itLikelike_UnkMotion14_Coll(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion14_Coll
                    65 => {  ret = m::itLikelike_UnkMotion15_Anim(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion15_Anim
                    66 => {  m::itLikelike_UnkMotion15_Phys(&raw mut GO[0]); } // itLikelike_UnkMotion15_Phys
                    67 => {  ret = m::itLikelike_UnkMotion15_Coll(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion15_Coll
                    68 => {  m::itLikeLike_Logic5_PickedUp(&raw mut GO[0]); } // itLikeLike_Logic5_PickedUp
                    69 => {  ret = m::itLikelike_UnkMotion18_Anim(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion18_Anim
                    70 => {  m::itLikelike_UnkMotion18_Phys(&raw mut GO[0]); } // itLikelike_UnkMotion18_Phys
                    71 => {  m::it_2725_Logic5_Dropped(&raw mut GO[0]); } // it_2725_Logic5_Dropped
                    72 => {  m::itLikeLike_Logic5_Thrown(&raw mut GO[0]); } // itLikeLike_Logic5_Thrown
                    73 => {  ret = m::itLikelike_UnkMotion19_Anim(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion19_Anim
                    74 => {  m::itLikelike_UnkMotion19_Phys(&raw mut GO[0]); } // itLikelike_UnkMotion19_Phys
                    75 => {  ret = m::itLikelike_UnkMotion19_Coll(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion19_Coll
                    76 => {  m::it_802DC0AC(&raw mut GO[0]); } // it_802DC0AC
                    77 => {  ret = m::itLikelike_UnkMotion6_Anim(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion6_Anim
                    78 => {  m::itLikelike_UnkMotion6_Phys(&raw mut GO[0]); } // itLikelike_UnkMotion6_Phys
                    79 => {  ret = m::itLikelike_UnkMotion6_Coll(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion6_Coll
                    80 => {  m::it_802DC310(&raw mut GO[0]); } // it_802DC310
                    81 => {  ret = m::itLikelike_UnkMotion20_Anim(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion20_Anim
                    82 => {  m::itLikelike_UnkMotion20_Phys(&raw mut GO[0]); } // itLikelike_UnkMotion20_Phys
                    83 => {  ret = m::itLikelike_UnkMotion20_Coll(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion20_Coll
                    84 => {  m::it_802DC3DC(&raw mut GO[0]); } // it_802DC3DC
                    85 => {  ret = m::itLikelike_UnkMotion21_Anim(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion21_Anim
                    86 => {  m::itLikelike_UnkMotion21_Phys(&raw mut GO[0]); } // itLikelike_UnkMotion21_Phys
                    87 => {  ret = m::itLikelike_UnkMotion21_Coll(&raw mut GO[0]) as u32; } // itLikelike_UnkMotion21_Coll
                    88 => {  m::itLikeLike_Logic5_Destroyed(&raw mut GO[0]); } // itLikeLike_Logic5_Destroyed
                    89 => { let v0: i32 = (rnd() % 3) as i32 as i32; let v2: i32 = (rnd() % 3) as i32 as i32; ret = pi(m::it_802DC4BC(v0, &raw mut VA[0], v2)); } // it_802DC4BC
                    90 => { for j in 0..22 { let row = &m::it_803F8468[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 36400);
    }
}
