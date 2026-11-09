//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/itleadead.c.
//! src/tu/itleadead.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py itleadead`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itleadead as m;
use mtu_harness::*;
#[path = "kinds_gen/itleadead_gen.rs"]
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

static mut LDX: [itSpecialAttrsHead; 3] = unsafe { core::mem::zeroed() }; static mut D40: it_804D6D40_t = unsafe { core::mem::zeroed() };

unsafe fn spec_fill(prof: i32) {
    match prof {
        0 => {
            for q in 0..3usize {
            let a = SPECB.0[q].as_mut_ptr() as *mut itLeadeadAttributes;
            LDX[q].x0 = (rnd() % 60) as i32; LDX[q].x4 = rw();
            (*a).x0 = &raw mut LDX[q]; (*a).x4 = (rnd() % 30) as i32; (*a).x8 = (rnd() % 4) as f32; (*a).xC = rw(); (*a).x10 = rw(); (*a).x14 = rw();
            (*a).x18 = (rnd() % 5) as i16; (*a).x1A = (rnd() % 4) as i16; (*a).x1C = (rnd() % 4) as i16; (*a).x1E = (rnd() % 4) as i8;
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
        IT[k].xDD4_itemVar.leadead.x38 = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        IT[k].xDD4_itemVar.leadead.x3C = rw();
        IT[k].xDD4_itemVar.leadead.x44 = (rnd() % 0x120) as i32;
        IT[k].xDD4_itemVar.leadead.x48 = (rnd() % 0x20) as i32 - 3;
        IT[k].xDD4_itemVar.leadead.x4C = (rnd() % 4) as i32 - 1;
        IT[k].xDD4_itemVar.leadead.x50 = (rnd() % 4) as i32;
        IT[k].xDD4_itemVar.leadead.xE14 = (rnd() % 8) as i32;
        IT[k].xC9C = (rnd() % 80) as i32;
    }
}

unsafe fn hash_all() {
    for k in 0..3 { hm(pi(IT[k].xDD4_itemVar.leadead.x38)); IT[k].xDD4_itemVar.leadead.x38 = null_mut(); }
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
fn itleadead_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        g::freg_all();
        const PROFTAB: [u8; 79] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_itleadead.txt"));
        for op in 0..79u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => {  m::it_802E8BCC(&raw mut GO[0]); } // it_802E8BCC
                    1 => {  ret = m::it_802E8CD8(&raw mut GO[0]) as u32; } // it_802E8CD8
                    2 => {  m::it_802E8F24(&raw mut GO[0]); } // it_802E8F24
                    3 => {  ret = m::itLeadead_UnkMotion1_Anim(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion1_Anim
                    4 => {  m::itLeadead_UnkMotion1_Phys(&raw mut GO[0]); } // itLeadead_UnkMotion1_Phys
                    5 => {  ret = m::itLeadead_UnkMotion1_Coll(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion1_Coll
                    6 => {  m::it_802E9038(&raw mut GO[0]); } // it_802E9038
                    7 => {  ret = m::itLeadead_UnkMotion2_Anim(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion2_Anim
                    8 => {  m::itLeadead_UnkMotion2_Phys(&raw mut GO[0]); } // itLeadead_UnkMotion2_Phys
                    9 => {  ret = m::itLeadead_UnkMotion2_Coll(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion2_Coll
                    10 => {  m::it_802E9308(&raw mut GO[0]); } // it_802E9308
                    11 => {  ret = m::itLeadead_UnkMotion10_Anim(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion10_Anim
                    12 => {  m::itLeadead_UnkMotion10_Phys(&raw mut GO[0]); } // itLeadead_UnkMotion10_Phys
                    13 => {  ret = m::itLeadead_UnkMotion10_Coll(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion10_Coll
                    14 => {  m::it_802E93C8(&raw mut GO[0]); } // it_802E93C8
                    15 => {  m::it_802E9494(&raw mut GO[0]); } // it_802E9494
                    16 => {  ret = m::itLeadead_UnkMotion11_Anim(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion11_Anim
                    17 => {  ret = m::itLeadead_UnkMotion12_Anim(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion12_Anim
                    18 => {  m::itLeadead_UnkMotion12_Phys(&raw mut GO[0]); } // itLeadead_UnkMotion12_Phys
                    19 => {  ret = m::itLeadead_UnkMotion12_Coll(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion12_Coll
                    20 => {  m::it_802E9738(&raw mut GO[0]); } // it_802E9738
                    21 => {  ret = m::itLeadead_UnkMotion7_Anim(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion7_Anim
                    22 => {  m::itLeadead_UnkMotion7_Phys(&raw mut GO[0]); } // itLeadead_UnkMotion7_Phys
                    23 => {  ret = m::itLeadead_UnkMotion7_Coll(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion7_Coll
                    24 => {  m::it_802E98E0(&raw mut GO[0]); } // it_802E98E0
                    25 => {  ret = m::itLeadead_UnkMotion8_Anim(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion8_Anim
                    26 => {  m::itLeadead_UnkMotion8_Phys(&raw mut GO[0]); } // itLeadead_UnkMotion8_Phys
                    27 => {  ret = m::itLeadead_UnkMotion8_Coll(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion8_Coll
                    28 => {  m::it_802E9A00(&raw mut GO[0]); } // it_802E9A00
                    29 => {  ret = m::itLeadead_UnkMotion9_Anim(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion9_Anim
                    30 => {  m::itLeadead_UnkMotion9_Phys(&raw mut GO[0]); } // itLeadead_UnkMotion9_Phys
                    31 => {  ret = m::itLeadead_UnkMotion9_Coll(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion9_Coll
                    32 => {  m::it_802E9BA0(&raw mut GO[0]); } // it_802E9BA0
                    33 => {  ret = m::itLeadead_UnkMotion3_Anim(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion3_Anim
                    34 => {  m::itLeadead_UnkMotion3_Phys(&raw mut GO[0]); } // itLeadead_UnkMotion3_Phys
                    35 => {  ret = m::itLeadead_UnkMotion3_Coll(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion3_Coll
                    36 => {  m::it_802E9D50(&raw mut GO[0]); } // it_802E9D50
                    37 => {  ret = m::itLeadead_UnkMotion4_Anim(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion4_Anim
                    38 => {  m::itLeadead_UnkMotion4_Phys(&raw mut GO[0]); } // itLeadead_UnkMotion4_Phys
                    39 => {  m::itLeadead_UnkMotion4_Coll_inline(&raw mut GO[0]); } // itLeadead_UnkMotion4_Coll_inline
                    40 => {  ret = m::itLeadead_UnkMotion4_Coll(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion4_Coll
                    41 => {  ret = m::itLeadead_UnkMotion5_Anim(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion5_Anim
                    42 => {  m::itLeadead_UnkMotion5_Phys(&raw mut GO[0]); } // itLeadead_UnkMotion5_Phys
                    43 => {  ret = m::itLeadead_UnkMotion5_Coll(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion5_Coll
                    44 => {  ret = m::itLeadead_UnkMotion6_Anim(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion6_Anim
                    45 => {  m::itLeadead_UnkMotion6_Phys(&raw mut GO[0]); } // itLeadead_UnkMotion6_Phys
                    46 => {  ret = m::itLeadead_UnkMotion6_Coll(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion6_Coll
                    47 => {  m::itLeadead_Logic1_PickedUp(&raw mut GO[0]); } // itLeadead_Logic1_PickedUp
                    48 => {  ret = m::itLeadead_UnkMotion13_Anim(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion13_Anim
                    49 => {  m::itLeadead_UnkMotion13_Phys(&raw mut GO[0]); } // itLeadead_UnkMotion13_Phys
                    50 => {  m::itLeadead_Logic1_Dropped(&raw mut GO[0]); } // itLeadead_Logic1_Dropped
                    51 => {  m::itLeadead_Logic1_Thrown(&raw mut GO[0]); } // itLeadead_Logic1_Thrown
                    52 => {  ret = m::itLeadead_UnkMotion14_Anim(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion14_Anim
                    53 => {  m::itLeadead_UnkMotion14_Phys(&raw mut GO[0]); } // itLeadead_UnkMotion14_Phys
                    54 => {  ret = m::itLeadead_UnkMotion14_Coll(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion14_Coll
                    55 => {  m::it_802EA2A0(&raw mut GO[0]); } // it_802EA2A0
                    56 => {  ret = m::itLeadead_UnkMotion15_Anim(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion15_Anim
                    57 => {  m::itLeadead_UnkMotion15_Phys(&raw mut GO[0]); } // itLeadead_UnkMotion15_Phys
                    58 => {  ret = m::itLeadead_UnkMotion15_Coll(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion15_Coll
                    59 => {  m::it_802EA334(&raw mut GO[0]); } // it_802EA334
                    60 => {  ret = m::itLeadead_UnkMotion16_Anim(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion16_Anim
                    61 => {  m::itLeadead_UnkMotion16_Phys(&raw mut GO[0]); } // itLeadead_UnkMotion16_Phys
                    62 => {  ret = m::itLeadead_UnkMotion16_Coll(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion16_Coll
                    63 => {  ret = m::itLeadead_UnkMotion17_Anim(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion17_Anim
                    64 => {  m::itLeadead_UnkMotion17_Phys(&raw mut GO[0]); } // itLeadead_UnkMotion17_Phys
                    65 => {  ret = m::itLeadead_UnkMotion17_Coll(&raw mut GO[0]) as u32; } // itLeadead_UnkMotion17_Coll
                    66 => {  m::itLeadead_Logic1_Destroyed(&raw mut GO[0]); } // itLeadead_Logic1_Destroyed
                    67 => { let v1: i32 = (rnd() % 5) as i32 as i32; let v2: i32 = (rnd() % 6) as i32 as i32; m::it_802EA478(&raw mut GO[0], v1, v2); } // it_802EA478
                    68 => {  ret = m::it_802EA674(&raw mut GO[0]) as u32; } // it_802EA674
                    69 => {  ret = m::it_802EA6F4(&raw mut GO[0]) as u32; } // it_802EA6F4
                    70 => { let v1: f32 = rw() as f32; ret = m::it_802EA804(&raw mut GO[0], v1) as u32; } // it_802EA804
                    71 => { let v1: f32 = rw() as f32; let v2: f32 = rw() as f32; ret = m::it_802EA988(&raw mut GO[0], v1, v2) as u32; } // it_802EA988
                    72 => { let v1: i32 = if rnd() & 1 != 0 { 1 } else { -1 } as i32; ret = pi(m::it_802EA9FC(&raw mut VA[0], v1)); } // it_802EA9FC
                    73 => { let v2: i32 = (rnd() % 8) as i32 as i32; let v3: f32 = rw() as f32; m::it_802EAAEC(&raw mut GO[0], &raw mut GO[1], v2, v3); } // it_802EAAEC
                    74 => {  m::it_802EAC8C(&raw mut GO[0]); } // it_802EAC8C
                    75 => {  m::it_802EADD8(&raw mut GO[0]); } // it_802EADD8
                    76 => {  m::it_802EAE80(&raw mut GO[0]); } // it_802EAE80
                    77 => {  ret = (m::it_802EAF28(&raw mut GO[0]) as u32); } // it_802EAF28
                    78 => { for j in 0..18 { let row = &m::it_803F8EB0[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 31600);
    }
}
