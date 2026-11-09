//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/itoldkuri.c.
//! src/tu/itoldkuri.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py itoldkuri`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itoldkuri as m;
use mtu_harness::*;
#[path = "kinds_gen/itoldkuri_gen.rs"]
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

unsafe fn spec_fill(prof: i32) {
    match prof {
        0 => {
            let a = SPECB.0[0].as_mut_ptr() as *mut m::itOldkuriAttributes;
            let o = SPECB.0[0].as_mut_ptr().add(64) as *mut melee_types::itOldkuriAttributes_x0; (*a).x0 = o as *mut _; (*o).x0 = (rnd() % 30) as i32; (*o).x4 = rw();
            (*a).x4 = rw(); (*a).x8 = rw(); (*a).xC = (rnd() % 10) as f32;
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
        IT[k].xDD4_itemVar.oldkuri.xDF8 = (rnd() % 3) as i32;
        IT[k].xDD4_itemVar.oldkuri.xDFC = (rnd() & 1) as i32;
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
fn itoldkuri_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        g::freg_all();
        const PROFTAB: [u8; 53] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_itoldkuri.txt"));
        for op in 0..53u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => {  m::itOldKuri_Logic29_EvtUnk(&raw mut GO[0], &raw mut GO[1]); } // itOldKuri_Logic29_EvtUnk
                    1 => {  m::it_802D73F0(&raw mut GO[0]); } // it_802D73F0
                    2 => {  m::it_802D747C(&raw mut GO[0]); } // it_802D747C
                    3 => {  ret = m::itOldkuri_UnkMotion0_Anim(&raw mut GO[0]) as u32; } // itOldkuri_UnkMotion0_Anim
                    4 => {  m::itOldkuri_UnkMotion0_Phys(&raw mut GO[0]); } // itOldkuri_UnkMotion0_Phys
                    5 => {  ret = m::itOldkuri_UnkMotion0_Coll(&raw mut GO[0]) as u32; } // itOldkuri_UnkMotion0_Coll
                    6 => {  m::it_802D758C(&raw mut GO[0]); } // it_802D758C
                    7 => {  ret = m::itOldkuri_UnkMotion1_Anim(&raw mut GO[0]) as u32; } // itOldkuri_UnkMotion1_Anim
                    8 => {  m::itOldkuri_UnkMotion1_Phys(&raw mut GO[0]); } // itOldkuri_UnkMotion1_Phys
                    9 => {  ret = m::itOldkuri_UnkMotion1_Coll(&raw mut GO[0]) as u32; } // itOldkuri_UnkMotion1_Coll
                    10 => {  m::it_802D775C(&raw mut GO[0]); } // it_802D775C
                    11 => {  ret = m::itOldkuri_UnkMotion2_Anim(&raw mut GO[0]) as u32; } // itOldkuri_UnkMotion2_Anim
                    12 => {  m::itOldkuri_UnkMotion2_Phys(&raw mut GO[0]); } // itOldkuri_UnkMotion2_Phys
                    13 => {  ret = m::itOldkuri_UnkMotion2_Coll(&raw mut GO[0]) as u32; } // itOldkuri_UnkMotion2_Coll
                    14 => {  ret = m::itOldkuri_UnkMotion3_Anim(&raw mut GO[0]) as u32; } // itOldkuri_UnkMotion3_Anim
                    15 => {  m::itOldkuri_UnkMotion3_Phys(&raw mut GO[0]); } // itOldkuri_UnkMotion3_Phys
                    16 => {  ret = m::itOldkuri_UnkMotion3_Coll(&raw mut GO[0]) as u32; } // itOldkuri_UnkMotion3_Coll
                    17 => {  m::it_802D7AF0(&raw mut GO[0]); } // it_802D7AF0
                    18 => {  ret = m::itOldkuri_UnkMotion4_Anim(&raw mut GO[0]) as u32; } // itOldkuri_UnkMotion4_Anim
                    19 => {  m::itOldkuri_UnkMotion4_Phys(&raw mut GO[0]); } // itOldkuri_UnkMotion4_Phys
                    20 => {  ret = m::itOldkuri_UnkMotion4_Coll(&raw mut GO[0]) as u32; } // itOldkuri_UnkMotion4_Coll
                    21 => {  m::itOldKuri_Logic0_PickedUp(&raw mut GO[0]); } // itOldKuri_Logic0_PickedUp
                    22 => {  ret = m::itOldkuri_UnkMotion5_Anim(&raw mut GO[0]) as u32; } // itOldkuri_UnkMotion5_Anim
                    23 => {  m::itOldkuri_UnkMotion5_Phys(&raw mut GO[0]); } // itOldkuri_UnkMotion5_Phys
                    24 => {  m::it_2725_Logic0_Dropped(&raw mut GO[0]); } // it_2725_Logic0_Dropped
                    25 => {  m::it_2725_Logic0_Thrown(&raw mut GO[0]); } // it_2725_Logic0_Thrown
                    26 => {  ret = m::itOldkuri_UnkMotion6_Anim(&raw mut GO[0]) as u32; } // itOldkuri_UnkMotion6_Anim
                    27 => {  m::itOldkuri_UnkMotion6_Phys(&raw mut GO[0]); } // itOldkuri_UnkMotion6_Phys
                    28 => {  ret = m::itOldkuri_UnkMotion6_Coll(&raw mut GO[0]) as u32; } // itOldkuri_UnkMotion6_Coll
                    29 => {  ret = m::itOldkuri_UnkMotion9_Anim(&raw mut GO[0]) as u32; } // itOldkuri_UnkMotion9_Anim
                    30 => {  m::itOldkuri_UnkMotion9_Phys(&raw mut GO[0]); } // itOldkuri_UnkMotion9_Phys
                    31 => {  ret = m::itOldkuri_UnkMotion9_Coll(&raw mut GO[0]) as u32; } // itOldkuri_UnkMotion9_Coll
                    32 => {  ret = m::it_2725_Logic0_DmgReceived(&raw mut GO[0]) as u32; } // it_2725_Logic0_DmgReceived
                    33 => {  ret = m::it_802D8098(&raw mut GO[0]) as u32; } // it_802D8098
                    34 => {  m::it_802D813C(&raw mut GO[0]); } // it_802D813C
                    35 => {  ret = m::itOldkuri_UnkMotion7_Anim(&raw mut GO[0]) as u32; } // itOldkuri_UnkMotion7_Anim
                    36 => {  m::itOldkuri_UnkMotion7_Phys(&raw mut GO[0]); } // itOldkuri_UnkMotion7_Phys
                    37 => {  ret = m::itOldkuri_UnkMotion7_Coll(&raw mut GO[0]) as u32; } // itOldkuri_UnkMotion7_Coll
                    38 => {  m::it_802D81FC(&raw mut GO[0]); } // it_802D81FC
                    39 => {  ret = m::itOldkuri_UnkMotion8_Anim(&raw mut GO[0]) as u32; } // itOldkuri_UnkMotion8_Anim
                    40 => {  m::itOldkuri_UnkMotion8_Phys(&raw mut GO[0]); } // itOldkuri_UnkMotion8_Phys
                    41 => {  ret = m::itOldkuri_UnkMotion8_Coll(&raw mut GO[0]) as u32; } // itOldkuri_UnkMotion8_Coll
                    42 => {  m::it_802D82C4(&raw mut GO[0]); } // it_802D82C4
                    43 => {  ret = m::itOldkuri_UnkMotion10_Anim(&raw mut GO[0]) as u32; } // itOldkuri_UnkMotion10_Anim
                    44 => {  m::itOldkuri_UnkMotion10_Phys(&raw mut GO[0]); } // itOldkuri_UnkMotion10_Phys
                    45 => {  ret = m::itOldkuri_UnkMotion10_Coll(&raw mut GO[0]) as u32; } // itOldkuri_UnkMotion10_Coll
                    46 => {  m::it_802D839C(&raw mut GO[0]); } // it_802D839C
                    47 => {  ret = m::itOldkuri_UnkMotion11_Anim(&raw mut GO[0]) as u32; } // itOldkuri_UnkMotion11_Anim
                    48 => {  m::itOldkuri_UnkMotion11_Phys(&raw mut GO[0]); } // itOldkuri_UnkMotion11_Phys
                    49 => {  ret = m::itOldkuri_UnkMotion11_Coll(&raw mut GO[0]) as u32; } // itOldkuri_UnkMotion11_Coll
                    50 => { let v1: i32 = fv() as i32 as i32; let v2: Item_StateChangeFlags = fv() as i32 as Item_StateChangeFlags; m::it_802D848C(&raw mut GO[0], v1, v2); } // it_802D848C
                    51 => {  m::it_802D84D8(&raw mut GO[0], &raw mut GO[1]); } // it_802D84D8
                    52 => { for j in 0..12 { let row = &m::it_803F8320[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 21200);
    }
}
