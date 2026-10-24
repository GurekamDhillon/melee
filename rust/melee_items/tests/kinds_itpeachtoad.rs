//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/itpeachtoad.c.
//! src/tu/itpeachtoad.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py itpeachtoad`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itpeachtoad as m;
use mtu_harness::*;
#[path = "kinds_gen/itpeachtoad_gen.rs"]
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

const FR: [f32; 5] = [7.0, 53.0, 60.0, 59.0, 0.0];

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
    IT[0].kind = if rnd() & 1 != 0 { It_Kind_Peach_Toad } else { It_Kind_Peach_Toad + 1 + (rnd() % 40) as i32 };
    IT[0].x5CC_currentAnimFrame = if rnd() % 6 == 5 { rw() } else { FR[(rnd() % 5) as usize] };
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
fn itpeachtoad_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        g::freg_all();
        const PROFTAB: [u8; 11] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_itpeachtoad.txt"));
        for op in 0..11u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => { let v2: Fighter_Part = fv() as i32 as Fighter_Part; let v3: ItemKind = fv() as i32 as ItemKind; let v4: f32 = rw() as f32; ret = pi(m::it_802BDE18(&raw mut GO[0], &raw mut VA[0], v2, v3, v4)); } // it_802BDE18
                    1 => {  m::itPeachToad_Logic91_Destroyed(&raw mut GO[0]); } // itPeachToad_Logic91_Destroyed
                    2 => {  m::it_802BDF40(&raw mut GO[0]); } // it_802BDF40
                    3 => {  m::it_802BDFA0(&raw mut GO[0]); } // it_802BDFA0
                    4 => {  m::it_802BDFC0(&raw mut GO[0]); } // it_802BDFC0
                    5 => {  m::itPeachToad_Logic91_PickedUp(&raw mut GO[0]); } // itPeachToad_Logic91_PickedUp
                    6 => {  ret = m::itPeachtoad_UnkMotion0_Anim(&raw mut GO[0]) as u32; } // itPeachtoad_UnkMotion0_Anim
                    7 => {  m::it_802BE100(&raw mut GO[0]); } // it_802BE100
                    8 => {  ret = m::itPeachtoad_UnkMotion1_Anim(&raw mut GO[0]) as u32; } // itPeachtoad_UnkMotion1_Anim
                    9 => {  m::itPeachToad_Logic91_EvtUnk(&raw mut GO[0], &raw mut GO[1]); } // itPeachToad_Logic91_EvtUnk
                    10 => { for j in 0..2 { let row = &m::it_803F7528[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 4400);
    }
}
