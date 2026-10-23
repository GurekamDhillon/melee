//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/itoldottosea.c.
//! src/tu/itoldottosea.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py itoldottosea`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itoldottosea as m;
use mtu_harness::*;
#[path = "kinds_gen/itoldottosea_gen.rs"]
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
            let a = SPECB.0[0].as_mut_ptr() as *mut itOldottoseaAttributes;
            (*a).x0 = SPECB.0[0].as_mut_ptr().add(64) as *mut _; *(SPECB.0[0].as_mut_ptr().add(64) as *mut i32) = (rnd() % 30) as i32; *(SPECB.0[0].as_mut_ptr().add(68) as *mut f32) = rw();
            (*a).x4 = rw(); (*a).x8 = rw(); (*a).xC = rw(); (*a).x10 = (rnd() % 4) as i8;
            (*a).x14 = rw(); (*a).x18 = rw(); (*a).x1C = rw(); (*a).x20 = rw(); (*a).x24 = rw(); (*a).x28 = (rnd() % 3) as i8;
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
        IT[k].xDD4_itemVar.oldottosea.x20 = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        IT[k].xDD4_itemVar.oldottosea.x24 = [0, 10, 25, 42, 80, 81][(rnd() % 6) as usize];
        IT[k].xDD4_itemVar.oldottosea.x28 = (rnd() % 3) as i32;
        IT[k].xDD4_itemVar.oldottosea.x2C = (rnd() & 1) as i16;
    }
}

unsafe fn hash_all() {
    for k in 0..3 { hm(pi(IT[k].xDD4_itemVar.oldottosea.x20)); IT[k].xDD4_itemVar.oldottosea.x20 = null_mut(); }
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
fn itoldottosea_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        g::freg_all();
        const PROFTAB: [u8; 28] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_itoldottosea.txt"));
        for op in 0..28u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => {  m::it_802E2470(&raw mut GO[0]); } // it_802E2470
                    1 => {  ret = m::it_2725_Logic8_DmgReceived(&raw mut GO[0]) as u32; } // it_2725_Logic8_DmgReceived
                    2 => {  m::it_802E269C(&raw mut GO[0]); } // it_802E269C
                    3 => {  ret = m::itOldottosea_UnkMotion0_Anim(&raw mut GO[0]) as u32; } // itOldottosea_UnkMotion0_Anim
                    4 => {  m::itOldottosea_UnkMotion0_Phys(&raw mut GO[0]); } // itOldottosea_UnkMotion0_Phys
                    5 => {  ret = m::itOldottosea_UnkMotion0_Coll(&raw mut GO[0]) as u32; } // itOldottosea_UnkMotion0_Coll
                    6 => {  m::it_802E27B4(&raw mut GO[0]); } // it_802E27B4
                    7 => {  ret = m::itOldottosea_UnkMotion2_Anim(&raw mut GO[0]) as u32; } // itOldottosea_UnkMotion2_Anim
                    8 => {  m::itOldottosea_UnkMotion2_Phys(&raw mut GO[0]); } // itOldottosea_UnkMotion2_Phys
                    9 => {  ret = m::itOldottosea_UnkMotion2_Coll(&raw mut GO[0]) as u32; } // itOldottosea_UnkMotion2_Coll
                    10 => {  m::it_802E2BC0(&raw mut GO[0]); } // it_802E2BC0
                    11 => {  ret = m::itOldottosea_UnkMotion4_Anim(&raw mut GO[0]) as u32; } // itOldottosea_UnkMotion4_Anim
                    12 => {  m::itOldottosea_UnkMotion4_Phys(&raw mut GO[0]); } // itOldottosea_UnkMotion4_Phys
                    13 => {  ret = m::itOldottosea_UnkMotion4_Coll(&raw mut GO[0]) as u32; } // itOldottosea_UnkMotion4_Coll
                    14 => {  m::it_802E2C80(&raw mut GO[0]); } // it_802E2C80
                    15 => {  ret = m::itOldottosea_UnkMotion5_Anim(&raw mut GO[0]) as u32; } // itOldottosea_UnkMotion5_Anim
                    16 => {  m::itOldottosea_UnkMotion5_Phys(&raw mut GO[0]); } // itOldottosea_UnkMotion5_Phys
                    17 => {  ret = m::itOldottosea_UnkMotion5_Coll(&raw mut GO[0]) as u32; } // itOldottosea_UnkMotion5_Coll
                    18 => {  m::it_802E2DF4(&raw mut GO[0]); } // it_802E2DF4
                    19 => {  m::it_802E2E30(&raw mut GO[0]); } // it_802E2E30
                    20 => {  ret = m::itOldottosea_UnkMotion7_Anim(&raw mut GO[0]) as u32; } // itOldottosea_UnkMotion7_Anim
                    21 => {  m::itOldottosea_UnkMotion7_Phys(&raw mut GO[0]); } // itOldottosea_UnkMotion7_Phys
                    22 => {  ret = m::itOldottosea_UnkMotion7_Coll(&raw mut GO[0]) as u32; } // itOldottosea_UnkMotion7_Coll
                    23 => {  m::it_802E3098(&raw mut GO[0]); } // it_802E3098
                    24 => {  ret = m::itOldottosea_UnkMotion3_Anim(&raw mut GO[0]) as u32; } // itOldottosea_UnkMotion3_Anim
                    25 => {  m::itOldottosea_UnkMotion3_Phys(&raw mut GO[0]); } // itOldottosea_UnkMotion3_Phys
                    26 => {  ret = m::itOldottosea_UnkMotion3_Coll(&raw mut GO[0]) as u32; } // itOldottosea_UnkMotion3_Coll
                    27 => { for j in 0..12 { let row = &m::it_803F89C8[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 11200);
    }
}
