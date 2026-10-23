//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/itpikachutjoltair.c.
//! src/tu/itpikachutjoltair.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py itpikachutjoltair`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itpikachutjoltair as m;
use mtu_harness::*;
#[path = "kinds_gen/itpikachutjoltair_gen.rs"]
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

const KJ: [i32; 6] = [It_Kind_Kirby_PikachuTJolt_Air, It_Kind_Pikachu_TJolt_Air, It_Kind_Kirby_PichuTJolt_Air, It_Kind_Pichu_TJolt_Air, 5, 0];

unsafe fn spec_fill(prof: i32) {
    match prof {
        0 => {
            let _ = 0;
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
        IT[k].kind = KJ[(rnd() % 6) as usize];
        let ja = &mut IT[k].xDD4_itemVar.pikachujoltair;
        ja.xDD4 = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        ja.xDD8 = if rnd() & 1 != 0 { &raw mut GO[(k + 2) % 3] } else { null_mut() };
        ja.xDE8.x = rw(); ja.xDE8.y = rw(); ja.xDE8.z = rw();
        ja.xDF4 = (rnd() % 30) as i32 - 2;
    }
}

unsafe fn hash_all() {
    for k in 0..3 {
        let ja = &mut IT[k].xDD4_itemVar.pikachujoltair;
        hm(pi(ja.xDD4)); hm(pi(ja.xDD8)); ja.xDD4 = null_mut(); ja.xDD8 = null_mut();
    }
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
fn itpikachutjoltair_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        g::freg_all();
        const PROFTAB: [u8; 17] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_itpikachutjoltair.txt"));
        for op in 0..17u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => {  ret = pi(m::it_802B3EFC(&raw mut GO[0])); } // it_802B3EFC
                    1 => {  m::it_802B3F20(&raw mut GO[0], &raw mut VA[0]); } // it_802B3F20
                    2 => { let v2: *mut CollData = (&raw mut IT[1].x378_itemColl) as *mut CollData; m::it_802B3F88(&raw mut GO[0], &raw mut VA[0], v2, &raw mut VA[1]); } // it_802B3F88
                    3 => { let v3: i32 = fv() as i32 as i32; let v4: f32 = rw() as f32; ret = pi(m::it_802B4224(&raw mut GO[0], &raw mut GO[1], &raw mut VA[0], v3, v4)); } // it_802B4224
                    4 => {  m::it_802B43B0(&raw mut GO[0]); } // it_802B43B0
                    5 => {  m::it_802B43D0(&raw mut GO[0], &raw mut GO[1]); } // it_802B43D0
                    6 => {  ret = m::itPikachutjoltair_UnkMotion0_Anim(&raw mut GO[0]) as u32; } // itPikachutjoltair_UnkMotion0_Anim
                    7 => {  m::itPikachutjoltair_UnkMotion0_Phys(&raw mut GO[0]); } // itPikachutjoltair_UnkMotion0_Phys
                    8 => {  ret = m::itPikachutjoltair_UnkMotion0_Coll(&raw mut GO[0]) as u32; } // itPikachutjoltair_UnkMotion0_Coll
                    9 => {  ret = m::it_2725_Logic107_DmgDealt(&raw mut GO[0]) as u32; } // it_2725_Logic107_DmgDealt
                    10 => {  ret = m::it_2725_Logic107_Clanked(&raw mut GO[0]) as u32; } // it_2725_Logic107_Clanked
                    11 => {  ret = m::it_2725_Logic107_Absorbed(&raw mut GO[0]) as u32; } // it_2725_Logic107_Absorbed
                    12 => {  ret = m::it_2725_Logic107_Reflected(&raw mut GO[0]) as u32; } // it_2725_Logic107_Reflected
                    13 => {  ret = m::it_2725_Logic107_HitShield(&raw mut GO[0]) as u32; } // it_2725_Logic107_HitShield
                    14 => {  ret = m::it_2725_Logic107_ShieldBounced(&raw mut GO[0]) as u32; } // it_2725_Logic107_ShieldBounced
                    15 => {  m::itPikachuTJoltAir_Logic107_EvtUnk(&raw mut GO[0], &raw mut GO[1]); } // itPikachuTJoltAir_Logic107_EvtUnk
                    16 => { for j in 0..1 { let row = &m::it_803F71D8[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 6800);
    }
}
