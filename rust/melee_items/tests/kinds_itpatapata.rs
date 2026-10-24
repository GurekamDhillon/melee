//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/itpatapata.c.
//! src/tu/itpatapata.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py itpatapata`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itpatapata as m;
use mtu_harness::*;
#[path = "kinds_gen/itpatapata_gen.rs"]
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



unsafe fn spec_fill(prof: i32) {
    match prof {
        0 => {
            let a = SPECB.0[0].as_mut_ptr() as *mut itPatapataAttributes;
            (*a).x0 = SPECB.0[0].as_mut_ptr().add(64) as *mut _; *(SPECB.0[0].as_mut_ptr().add(68) as *mut f32) = rw();
            (*a).x4 = rw(); (*a).x8 = rw(); (*a).xC = rw(); (*a).x10 = (rnd() % 6) as i32; (*a).x14 = rw(); (*a).x18 = rw();
            (*a).x1C = (rnd() % 20) as i32; (*a).x20 = rw(); (*a).x24 = (rnd() % 20) as i32; (*a).x28 = (rnd() % 30) as f32;
            (*a).x2C = (rnd() % 6) as i32; (*a).x30 = (rnd() % 5) as f32; (*a).x34 = (rnd() % 5) as f32; (*a).x38 = rw(); (*a).x3C = rw();
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
    for k in 0..3 {
        for i in 14..19 { DBT[k].bones[i] = &raw mut JO[6 + 14 * k + (i - 14)]; }
        let pp = &mut IT[k].xDD4_itemVar.patapata;
        pp.x40 = (rnd() % 3) as i32;
        pp.x20 = (rnd() % 5) as i32 - 1;
        pp.x24 = (rnd() % 5) as i32 - 1;
        pp.x28 = (rnd() % 5) as i32 - 1;
        pp.x2C = rw();
        pp.x30 = rw();
        pp.x44 = (rnd() & 1) as i32;
        pp.x4 = (rnd() % 4) as i32;
    }
}

unsafe fn hash_all() {

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
fn itpatapata_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        g::freg_all();
        const PROFTAB: [u8; 34] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_itpatapata.txt"));
        for op in 0..34u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => {  m::it_802E05A0(&raw mut GO[0]); } // it_802E05A0
                    1 => {  m::it_802E0678(&raw mut GO[0]); } // it_802E0678
                    2 => {  ret = m::itPatapata_UnkMotion1_Anim(&raw mut GO[0]) as u32; } // itPatapata_UnkMotion1_Anim
                    3 => {  m::itPatapata_UnkMotion1_Phys(&raw mut GO[0]); } // itPatapata_UnkMotion1_Phys
                    4 => {  ret = m::itPatapata_UnkMotion1_Coll(&raw mut GO[0]) as u32; } // itPatapata_UnkMotion1_Coll
                    5 => {  m::it_802E0734(&raw mut GO[0]); } // it_802E0734
                    6 => {  ret = m::itPatapata_UnkMotion2_Anim(&raw mut GO[0]) as u32; } // itPatapata_UnkMotion2_Anim
                    7 => {  m::itPatapata_UnkMotion3_Phys(&raw mut GO[0]); } // itPatapata_UnkMotion3_Phys
                    8 => {  ret = m::itPatapata_UnkMotion3_Coll(&raw mut GO[0]) as u32; } // itPatapata_UnkMotion3_Coll
                    9 => {  m::it_802E0D9C(&raw mut GO[0]); } // it_802E0D9C
                    10 => {  ret = m::itPatapata_UnkMotion3_Anim(&raw mut GO[0]) as u32; } // itPatapata_UnkMotion3_Anim
                    11 => {  ret = m::it_2725_Logic4_DmgReceived(&raw mut GO[0]) as u32; } // it_2725_Logic4_DmgReceived
                    12 => {  ret = m::it_802E0F1C(&raw mut GO[0]) as u32; } // it_802E0F1C
                    13 => {  m::itPatapata_Logic4_PickedUp(&raw mut GO[0]); } // itPatapata_Logic4_PickedUp
                    14 => {  ret = m::itPatapata_UnkMotion5_Anim(&raw mut GO[0]) as u32; } // itPatapata_UnkMotion5_Anim
                    15 => {  m::itPatapata_UnkMotion5_Phys(&raw mut GO[0]); } // itPatapata_UnkMotion5_Phys
                    16 => {  m::it_2725_Logic4_Dropped(&raw mut GO[0]); } // it_2725_Logic4_Dropped
                    17 => {  ret = m::itPatapata_UnkMotion7_Anim(&raw mut GO[0]) as u32; } // itPatapata_UnkMotion7_Anim
                    18 => {  m::itPatapata_UnkMotion7_Phys(&raw mut GO[0]); } // itPatapata_UnkMotion7_Phys
                    19 => {  ret = m::itPatapata_UnkMotion7_Coll(&raw mut GO[0]) as u32; } // itPatapata_UnkMotion7_Coll
                    20 => {  m::it_2725_Logic4_Thrown(&raw mut GO[0]); } // it_2725_Logic4_Thrown
                    21 => {  ret = m::itPatapata_UnkMotion6_Anim(&raw mut GO[0]) as u32; } // itPatapata_UnkMotion6_Anim
                    22 => {  m::itPatapata_UnkMotion6_Phys(&raw mut GO[0]); } // itPatapata_UnkMotion6_Phys
                    23 => {  ret = m::itPatapata_UnkMotion6_Coll(&raw mut GO[0]) as u32; } // itPatapata_UnkMotion6_Coll
                    24 => {  m::it_802E11E0(&raw mut GO[0]); } // it_802E11E0
                    25 => {  ret = m::itPatapata_UnkMotion4_Anim(&raw mut GO[0]) as u32; } // itPatapata_UnkMotion4_Anim
                    26 => {  m::itPatapata_UnkMotion4_Phys(&raw mut GO[0]); } // itPatapata_UnkMotion4_Phys
                    27 => {  ret = m::itPatapata_UnkMotion4_Coll(&raw mut GO[0]) as u32; } // itPatapata_UnkMotion4_Coll
                    28 => {  m::it_802E15B0(&raw mut GO[0]); } // it_802E15B0
                    29 => { let v1: i32 = fv() as i32 as i32; let v2: i32 = fv() as i32 as i32; m::it_802E1648(&raw mut GO[0], v1, v2); } // it_802E1648
                    30 => {  m::it_802E1694(&raw mut GO[0]); } // it_802E1694
                    31 => {  m::it_802E16D8(&raw mut GO[0], &raw mut GO[1]); } // it_802E16D8
                    32 => { let v0: i32 = (rnd() % 3) as i32 as i32; let v2: i32 = if rnd() & 1 != 0 { 1 } else { -1 } as i32; ret = pi(m::it_802E16F8(v0, &raw mut VA[0], v2)); } // it_802E16F8
                    33 => { for j in 0..8 { let row = &m::it_803F88C0[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 13600);
    }
}
