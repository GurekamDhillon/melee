//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/itparasol.c.
//! src/tu/itparasol.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py itparasol`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itparasol as m;
use mtu_harness::*;
#[path = "kinds_gen/itparasol_gen.rs"]
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
            let a = SPECB.0[0].as_mut_ptr() as *mut f32;
            *a = rw(); *a.add(1) = rw(); *a.add(2) = rw(); *a.add(3) = rw();
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
fn itparasol_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        g::freg_all();
        const PROFTAB: [u8; 34] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_itparasol.txt"));
        for op in 0..34u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => { let v1: i32 = [1, 2, 4, 5, 6, 7, 8, 9, 10][(rnd() % 9) as usize] as i32; ret = m::it_8028B08C(&raw mut GO[0], v1) as u32; } // it_8028B08C
                    1 => {  m::itParasol_Logic13_Spawned(&raw mut GO[0]); } // itParasol_Logic13_Spawned
                    2 => {  m::it_8028B0EC(&raw mut GO[0]); } // it_8028B0EC
                    3 => {  ret = m::itParasol_UnkMotion0_Anim(&raw mut GO[0]) as u32; } // itParasol_UnkMotion0_Anim
                    4 => {  m::itParasol_UnkMotion0_Phys(&raw mut GO[0]); } // itParasol_UnkMotion0_Phys
                    5 => {  ret = m::itParasol_UnkMotion0_Coll(&raw mut GO[0]) as u32; } // itParasol_UnkMotion0_Coll
                    6 => {  m::it_8028B17C(&raw mut GO[0]); } // it_8028B17C
                    7 => {  ret = m::itParasol_UnkMotion2_Anim(&raw mut GO[0]) as u32; } // itParasol_UnkMotion2_Anim
                    8 => {  m::itParasol_UnkMotion1_Phys(&raw mut GO[0]); } // itParasol_UnkMotion1_Phys
                    9 => {  ret = m::itParasol_UnkMotion2_Coll(&raw mut GO[0]) as u32; } // itParasol_UnkMotion2_Coll
                    10 => {  m::itParasol_Logic13_Dropped(&raw mut GO[0]); } // itParasol_Logic13_Dropped
                    11 => {  m::itParasol_Logic13_Thrown(&raw mut GO[0]); } // itParasol_Logic13_Thrown
                    12 => {  m::itParasol_UnkMotion2_Phys(&raw mut GO[0]); } // itParasol_UnkMotion2_Phys
                    13 => {  ret = m::itParasol_Logic13_DmgDealt(&raw mut GO[0]) as u32; } // itParasol_Logic13_DmgDealt
                    14 => {  ret = m::itParasol_Logic13_Clanked(&raw mut GO[0]) as u32; } // itParasol_Logic13_Clanked
                    15 => {  ret = m::itParasol_Logic13_HitShield(&raw mut GO[0]) as u32; } // itParasol_Logic13_HitShield
                    16 => {  ret = m::itParasol_Logic13_Reflected(&raw mut GO[0]) as u32; } // itParasol_Logic13_Reflected
                    17 => {  ret = m::itParasol_Logic13_ShieldBounced(&raw mut GO[0]) as u32; } // itParasol_Logic13_ShieldBounced
                    18 => {  m::itParasol_Logic13_EnteredAir(&raw mut GO[0]); } // itParasol_Logic13_EnteredAir
                    19 => {  ret = m::itParasol_UnkMotion3_Anim(&raw mut GO[0]) as u32; } // itParasol_UnkMotion3_Anim
                    20 => {  m::itParasol_UnkMotion3_Phys(&raw mut GO[0]); } // itParasol_UnkMotion3_Phys
                    21 => {  ret = m::itParasol_UnkMotion3_Coll(&raw mut GO[0]) as u32; } // itParasol_UnkMotion3_Coll
                    22 => {  m::itParasol_Logic13_PickedUp(&raw mut GO[0]); } // itParasol_Logic13_PickedUp
                    23 => {  ret = m::itParasol_UnkMotion10_Anim(&raw mut GO[0]) as u32; } // itParasol_UnkMotion10_Anim
                    24 => {  m::itParasol_UnkMotion10_Phys(&raw mut GO[0]); } // itParasol_UnkMotion10_Phys
                    25 => { let v1: f32 = rw() as f32; m::it_8028B618(&raw mut GO[0], v1); } // it_8028B618
                    26 => { let v1: f32 = rw() as f32; m::it_8028B648(&raw mut GO[0], v1); } // it_8028B648
                    27 => { let v1: f32 = rw() as f32; m::it_8028B6B0(&raw mut GO[0], v1); } // it_8028B6B0
                    28 => { let v1: f32 = rw() as f32; m::it_8028B718(&raw mut GO[0], v1); } // it_8028B718
                    29 => { let v1: f32 = rw() as f32; m::it_8028B780(&raw mut GO[0], v1); } // it_8028B780
                    30 => { let v1: f32 = rw() as f32; m::it_8028B7E8(&raw mut GO[0], v1); } // it_8028B7E8
                    31 => { let v1: f32 = rw() as f32; m::it_8028B850(&raw mut GO[0], v1); } // it_8028B850
                    32 => {  m::itParasol_Logic13_EvtUnk(&raw mut GO[0], &raw mut GO[1]); } // itParasol_Logic13_EvtUnk
                    33 => { for j in 0..11 { let row = &m::it_803F5AB0[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 13600);
    }
}
