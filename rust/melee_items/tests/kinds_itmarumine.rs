//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/itmarumine.c.
//! src/tu/itmarumine.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py itmarumine`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itmarumine as m;
use mtu_harness::*;
#[path = "kinds_gen/itmarumine_gen.rs"]
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
            let a = SPECB.0[0].as_mut_ptr() as *mut itPokemonAttributes;
            (*a).x0 = rw(); (*a).timer = (rnd() % 3) as i32; (*a).max = (rnd() % 200) as i32; (*a).xC = (rnd() % 4) as i32;
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
        IT[k].xDD4_itemVar.pokemon.timer = (rnd() % 5) as i32 - 2;
        IT[k].xDD4_itemVar.pokemon.x64 = (rnd() % 3) as i32;
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
fn itmarumine_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        g::freg_all();
        const PROFTAB: [u8; 38] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_itmarumine.txt"));
        for op in 0..38u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => {  m::it_802D09D0(&raw mut GO[0]); } // it_802D09D0
                    1 => {  m::itMarumine_Logic16_Spawned(&raw mut GO[0]); } // itMarumine_Logic16_Spawned
                    2 => {  m::it_802D0AAC(&raw mut GO[0]); } // it_802D0AAC
                    3 => {  m::it_802D0AB0(&raw mut GO[0], &raw mut GO[1]); } // it_802D0AB0
                    4 => {  ret = m::it_802D0AD0(&raw mut GO[0]) as u32; } // it_802D0AD0
                    5 => {  m::it_802D0B04(&raw mut GO[0]); } // it_802D0B04
                    6 => {  ret = m::itMarumine_UnkMotion1_Anim(&raw mut GO[0]) as u32; } // itMarumine_UnkMotion1_Anim
                    7 => {  m::itMarumine_UnkMotion1_Phys(&raw mut GO[0]); } // itMarumine_UnkMotion1_Phys
                    8 => {  ret = m::itMarumine_UnkMotion1_Coll(&raw mut GO[0]) as u32; } // itMarumine_UnkMotion1_Coll
                    9 => {  m::it_802D0C44(&raw mut GO[0]); } // it_802D0C44
                    10 => {  ret = m::itMarumine_UnkMotion2_Anim(&raw mut GO[0]) as u32; } // itMarumine_UnkMotion2_Anim
                    11 => {  m::itMarumine_UnkMotion2_Phys(&raw mut GO[0]); } // itMarumine_UnkMotion2_Phys
                    12 => {  ret = m::itMarumine_UnkMotion2_Coll(&raw mut GO[0]) as u32; } // itMarumine_UnkMotion2_Coll
                    13 => {  m::it_802D0D18(&raw mut GO[0]); } // it_802D0D18
                    14 => {  ret = m::itMarumine_UnkMotion3_Anim(&raw mut GO[0]) as u32; } // itMarumine_UnkMotion3_Anim
                    15 => {  m::itMarumine_UnkMotion3_Phys(&raw mut GO[0]); } // itMarumine_UnkMotion3_Phys
                    16 => {  ret = m::itMarumine_UnkMotion3_Coll(&raw mut GO[0]) as u32; } // itMarumine_UnkMotion3_Coll
                    17 => {  m::it_802D0DBC(&raw mut GO[0]); } // it_802D0DBC
                    18 => {  m::it_802D0E30(&raw mut GO[0]); } // it_802D0E30
                    19 => {  m::it_802D0E90(&raw mut GO[0]); } // it_802D0E90
                    20 => {  ret = m::itMarumine_UnkMotion4_Anim(&raw mut GO[0]) as u32; } // itMarumine_UnkMotion4_Anim
                    21 => {  m::itMarumine_UnkMotion4_Phys(&raw mut GO[0]); } // itMarumine_UnkMotion4_Phys
                    22 => {  ret = m::itMarumine_UnkMotion4_Coll(&raw mut GO[0]) as u32; } // itMarumine_UnkMotion4_Coll
                    23 => {  m::fn_802D0F98(&raw mut GO[0]); } // fn_802D0F98
                    24 => {  m::it_802D100C(&raw mut GO[0]); } // it_802D100C
                    25 => {  ret = m::itMarumine_UnkMotion5_Anim(&raw mut GO[0]) as u32; } // itMarumine_UnkMotion5_Anim
                    26 => {  m::itMarumine_UnkMotion5_Phys(&raw mut GO[0]); } // itMarumine_UnkMotion5_Phys
                    27 => {  ret = m::itMarumine_UnkMotion5_Coll(&raw mut GO[0]) as u32; } // itMarumine_UnkMotion5_Coll
                    28 => {  m::it_802D1140(&raw mut GO[0]); } // it_802D1140
                    29 => {  m::it_802D1204(&raw mut GO[0]); } // it_802D1204
                    30 => {  ret = m::itMarumine_UnkMotion6_Anim(&raw mut GO[0]) as u32; } // itMarumine_UnkMotion6_Anim
                    31 => {  m::itMarumine_UnkMotion6_Phys(&raw mut GO[0]); } // itMarumine_UnkMotion6_Phys
                    32 => {  ret = m::itMarumine_UnkMotion6_Coll(&raw mut GO[0]) as u32; } // itMarumine_UnkMotion6_Coll
                    33 => {  m::it_802D1320(&raw mut GO[0]); } // it_802D1320
                    34 => {  ret = m::itMarumine_UnkMotion0_Anim(&raw mut GO[0]) as u32; } // itMarumine_UnkMotion0_Anim
                    35 => {  m::itMarumine_UnkMotion0_Phys(&raw mut GO[0]); } // itMarumine_UnkMotion0_Phys
                    36 => {  ret = m::itMarumine_UnkMotion0_Coll(&raw mut GO[0]) as u32; } // itMarumine_UnkMotion0_Coll
                    37 => { for j in 0..7 { let row = &m::it_803F7E78[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 15200);
    }
}
