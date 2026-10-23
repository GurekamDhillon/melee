//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/itpikachuthunder.c.
//! src/tu/itpikachuthunder.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py itpikachuthunder`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itpikachuthunder as m;
use mtu_harness::*;
#[path = "kinds_gen/itpikachuthunder_gen.rs"]
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
            for j in 0..3 {
                let a = SPECB.0[j].as_mut_ptr() as *mut f32;
                *a = rw(); *a.add(1) = if rnd() & 1 != 0 { (1 + rnd() % 40) as f32 } else { rw() }; *a.add(2) = rw();
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
    for k in 0..3 {
        let th = &mut IT[k].xDD4_itemVar.pikachuthunder;
        th.x0 = (rnd() % 3) as i32;
        th.x4 = (rnd() & 1) as i32;
        th.x8 = (rnd() % 4) as i32 - 1;
        th.xC = if rnd() & 1 != 0 { (rnd() % 40) as f32 } else { rw() };
        th.x10 = rw();
        th.x14 = if rnd() & 1 != 0 { 3.0 } else { rw() };
        th.x18 = rw();
        th.x1C.x = rw();
        th.x1C.y = if rnd() & 1 != 0 { -((rnd() % 8) as f32) } else { rw() };
        th.x1C.z = rw();
        th.x28.x = rw(); th.x28.y = rw(); th.x28.z = rw();
        th.x34 = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        th.x38 = if rnd() & 1 != 0 { &raw mut GO[(k + 2) % 3] } else { null_mut() };
        IT[k].xD44_lifeTimer = if rnd() & 1 != 0 { 1.0 } else { rw() };
    }
}

unsafe fn hash_all() {
    for k in 0..3 {
        let th = &mut IT[k].xDD4_itemVar.pikachuthunder;
        hm(pi(th.x34)); hm(pi(th.x38)); th.x34 = null_mut(); th.x38 = null_mut();
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
fn itpikachuthunder_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        g::freg_all();
        const PROFTAB: [u8; 19] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_itpikachuthunder.txt"));
        for op in 0..19u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => {  ret = m::it_802B1DEC(&raw mut GO[0]) as u32; } // it_802B1DEC
                    1 => { let v3: i32 = (rnd() % 4) as i32 as i32; let v4: i32 = (rnd() % 4) as i32 as i32; let v5: i32 = fv() as i32 as i32; ret = pi(m::it_802B1DF8(&raw mut GO[0], &raw mut VA[0], &raw mut VA[1], v3, v4, v5)); } // it_802B1DF8
                    2 => {  m::it_802B1FC8(&raw mut GO[0]); } // it_802B1FC8
                    3 => {  m::it_802B1FE8(&raw mut GO[0], &raw mut VA[0]); } // it_802B1FE8
                    4 => {  m::it_2725_Logic39_Destroyed(&raw mut GO[0]); } // it_2725_Logic39_Destroyed
                    5 => {  m::it_802B2080(&raw mut GO[0]); } // it_802B2080
                    6 => {  ret = m::itPikachuthunder_UnkMotion0_Anim(&raw mut GO[0]) as u32; } // itPikachuthunder_UnkMotion0_Anim
                    7 => {  m::it_802B211C(&raw mut GO[0]); } // it_802B211C
                    8 => {  ret = m::itPikachuthunder_UnkMotion1_Anim(&raw mut GO[0]) as u32; } // itPikachuthunder_UnkMotion1_Anim
                    9 => {  ret = m::itPikachuthunder_UnkMotion1_Coll(&raw mut GO[0]) as u32; } // itPikachuthunder_UnkMotion1_Coll
                    10 => {  m::it_802B22B8(&raw mut GO[0]); } // it_802B22B8
                    11 => { let v0: f32 = rw() as f32; let v1: f32 = rw() as f32; ret = (m::pika_scale(v0, v1)).to_bits(); } // pika_scale
                    12 => {  ret = m::itPikachuthunder_UnkMotion2_Anim(&raw mut GO[0]) as u32; } // itPikachuthunder_UnkMotion2_Anim
                    13 => {  ret = m::itPikachuThunder_Logic39_DmgDealt(&raw mut GO[0]) as u32; } // itPikachuThunder_Logic39_DmgDealt
                    14 => {  ret = m::itPikachuThunder_Logic39_HitShield(&raw mut GO[0]) as u32; } // itPikachuThunder_Logic39_HitShield
                    15 => {  ret = m::itPikachuThunder_Logic39_Clanked(&raw mut GO[0]) as u32; } // itPikachuThunder_Logic39_Clanked
                    16 => {  ret = m::itPikachuThunder_Logic39_Absorbed(&raw mut GO[0]) as u32; } // itPikachuThunder_Logic39_Absorbed
                    17 => {  m::itPikachuThunder_Logic39_EvtUnk(&raw mut GO[0], &raw mut GO[1]); } // itPikachuThunder_Logic39_EvtUnk
                    18 => { for j in 0..3 { let row = &m::it_803F70C8[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 7600);
    }
}
