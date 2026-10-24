//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/itwhispyapple.c.
//! src/tu/itwhispyapple.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py itwhispyapple`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itwhispyapple as m;
use mtu_harness::*;
#[path = "kinds_gen/itwhispyapple_gen.rs"]
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
            let a = SPECB.0[0].as_mut_ptr() as *mut itWhispyAppleAttributes;
            (*a).x4 = (rnd() % 30) as i32; (*a).x8 = (rnd() % 30) as i32; (*a).x14 = rw(); (*a).x18 = rw();
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
    m::ext::it_804D6D28 = &raw mut ICD;
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
        IT[k].xDD4_itemVar.whispyapple.xDD8 = (rnd() % 3) as i32;
        IT[k].xD44_lifeTimer = if rnd() & 1 != 0 { (rnd() % 4) as i32 as f32 } else { rg() };
        IT[k].pos.z = if rnd() & 1 != 0 { (rnd() % 3) as i32 as f32 - 1.0 } else { rg() };
        (*m::ext::it_804D6D28).x34 = rnd() % 4;
        if rnd() & 1 != 0 { IT[k].msid = 7; }
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
fn itwhispyapple_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        g::freg_all();
        const PROFTAB: [u8; 34] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_itwhispyapple.txt"));
        for op in 0..34u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => {  ret = m::itWhispyapple_UnkMotion0_Anim_inline(&raw mut GO[0]) as u32; } // itWhispyapple_UnkMotion0_Anim_inline
                    1 => { let v2: f32 = (rnd() % 10) as f32 / 10.0 as f32; let v3: f32 = (rnd() % 10) as f32 / 10.0 as f32; ret = pi(m::it_802EE200(&raw mut GO[0], &raw mut VA[0], v2, v3)); } // it_802EE200
                    2 => {  m::it_802EE374(&raw mut GO[0]); } // it_802EE374
                    3 => {  ret = m::itWhispyapple_UnkMotion0_Anim(&raw mut GO[0]) as u32; } // itWhispyapple_UnkMotion0_Anim
                    4 => {  m::itWhispyapple_UnkMotion0_Phys(&raw mut GO[0]); } // itWhispyapple_UnkMotion0_Phys
                    5 => {  ret = m::itWhispyapple_UnkMotion0_Coll(&raw mut GO[0]) as u32; } // itWhispyapple_UnkMotion0_Coll
                    6 => {  m::it_802EE6A0(&raw mut GO[0]); } // it_802EE6A0
                    7 => {  ret = m::itWhispyapple_UnkMotion1_Anim(&raw mut GO[0]) as u32; } // itWhispyapple_UnkMotion1_Anim
                    8 => {  m::itWhispyapple_UnkMotion1_Phys(&raw mut GO[0]); } // itWhispyapple_UnkMotion1_Phys
                    9 => {  ret = m::itWhispyapple_UnkMotion1_Coll(&raw mut GO[0]) as u32; } // itWhispyapple_UnkMotion1_Coll
                    10 => {  m::fn_802EE7FC(&raw mut GO[0]); } // fn_802EE7FC
                    11 => {  ret = m::itWhispyapple_UnkMotion5_Anim(&raw mut GO[0]) as u32; } // itWhispyapple_UnkMotion5_Anim
                    12 => {  m::itWhispyapple_UnkMotion5_Phys(&raw mut GO[0]); } // itWhispyapple_UnkMotion5_Phys
                    13 => {  ret = m::itWhispyapple_UnkMotion5_Coll(&raw mut GO[0]) as u32; } // itWhispyapple_UnkMotion5_Coll
                    14 => {  m::it_802EEA08(&raw mut GO[0]); } // it_802EEA08
                    15 => {  ret = m::itWhispyapple_UnkMotion3_Anim(&raw mut GO[0]) as u32; } // itWhispyapple_UnkMotion3_Anim
                    16 => {  m::itWhispyapple_UnkMotion3_Phys(&raw mut GO[0]); } // itWhispyapple_UnkMotion3_Phys
                    17 => {  m::it_802EEA70(&raw mut GO[0]); } // it_802EEA70
                    18 => {  m::it_802EEB28(&raw mut GO[0]); } // it_802EEB28
                    19 => {  m::itWhispyApple_Logic18_EnteredAir(&raw mut GO[0]); } // itWhispyApple_Logic18_EnteredAir
                    20 => {  ret = m::itWhispyapple_UnkMotion6_Anim(&raw mut GO[0]) as u32; } // itWhispyapple_UnkMotion6_Anim
                    21 => {  m::itWhispyapple_UnkMotion6_Phys(&raw mut GO[0]); } // itWhispyapple_UnkMotion6_Phys
                    22 => {  ret = m::itWhispyapple_UnkMotion6_Coll(&raw mut GO[0]) as u32; } // itWhispyapple_UnkMotion6_Coll
                    23 => {  m::it_802EED00(&raw mut GO[0]); } // it_802EED00
                    24 => {  ret = m::itWhispyapple_UnkMotion7_Anim(&raw mut GO[0]) as u32; } // itWhispyapple_UnkMotion7_Anim
                    25 => {  m::itWhispyapple_UnkMotion7_Phys(&raw mut GO[0]); } // itWhispyapple_UnkMotion7_Phys
                    26 => {  ret = m::itWhispyapple_UnkMotion7_Coll(&raw mut GO[0]) as u32; } // itWhispyapple_UnkMotion7_Coll
                    27 => {  ret = m::it_802EEED0(&raw mut GO[0]) as u32; } // it_802EEED0
                    28 => {  ret = m::it_802EEED8(&raw mut GO[0]) as u32; } // it_802EEED8
                    29 => {  ret = m::it_802EEF10(&raw mut GO[0]) as u32; } // it_802EEF10
                    30 => {  ret = m::it_802EEF30(&raw mut GO[0]) as u32; } // it_802EEF30
                    31 => {  ret = m::it_802EEF68(&raw mut GO[0]) as u32; } // it_802EEF68
                    32 => {  m::it_802EEF88(&raw mut GO[0], &raw mut GO[1]); } // it_802EEF88
                    33 => { for j in 0..8 { let row = &m::it_803F91D0[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 13600);
    }
}
