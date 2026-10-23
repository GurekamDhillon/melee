//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/itmewtwoshadowball.c.
//! src/tu/itmewtwoshadowball.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py itmewtwoshadowball`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itmewtwoshadowball as m;
use mtu_harness::*;
#[path = "kinds_gen/itmewtwoshadowball_gen.rs"]
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
            let a = SPECB.0[0].as_mut_ptr() as *mut itMewtwoShadowball_DatAttrs;
            (*a).x0 = rw(); (*a).x4 = rw(); (*a).x8 = rw(); (*a).xC = rw(); (*a).x10 = rw(); (*a).x14 = rw(); (*a).x18 = rw(); (*a).x1C = rw();
            (*a).x20 = (rnd() % 6) as i32; (*a).x24 = rw(); (*a).x28 = rw(); (*a).x2C = rw(); (*a).x30 = rw(); (*a).x34 = rw(); (*a).x38 = rw(); (*a).x3C = rw();
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
        JO[3 + k].child = &raw mut JO[9 + k];
        let sb = &mut IT[k].xDD4_itemVar.mewtwoshadowball;
        sb.x0 = rw();
        let i = (rnd() % 2000) as i32 - 1000;
        sb.x4.x = i as f32 / 37.0; sb.x4.y = rw(); sb.x4.z = rw();
        sb.x10 = rw();
        sb.x14 = (rnd() % 2) as i32;
        sb.x18 = (rnd() % 6) as i32 - 1;
        sb.x1C = (rnd() % 5) as i32 + 1;
        sb.x20 = (rnd() % 5) as i32;
        sb.x24 = rnd() % 9;
        sb.x28 = (rnd() % 2) as i32;
        sb.x2C = if rnd() & 3 != 0 { &raw mut GO[1] } else { null_mut() };
        sb.x30.x = rw(); sb.x30.y = rw(); sb.x30.z = rw();
        sb.x3C = rw(); sb.x40 = rw(); sb.x44 = rw();
        sb.x48 = (rnd() % 6) as i32;
        sb.x4C = (rnd() % 4) as i32 - 1;
        sb.x50 = rw(); sb.x54 = rw();
        if rnd() % 3 == 0 { sb.x58.x = 0.0; sb.x58.y = 0.0; sb.x58.z = 0.0; }
        else { sb.x58.x = rw(); sb.x58.y = rw(); sb.x58.z = rw(); }
        sb.x64 = rw();
        if k == 0 {
            let i = rnd() % 3;
            if i == 0 { IT[0].kind = It_Kind_Mewtwo_ShadowBall; }
            else if i == 1 { IT[0].kind = It_Kind_Kirby_MewtwoShadowBall; }
        }
    }
}

unsafe fn hash_all() {
    for k in 0..3 { hm(pi(IT[k].xDD4_itemVar.mewtwoshadowball.x2C)); IT[k].xDD4_itemVar.mewtwoshadowball.x2C = null_mut(); }
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
fn itmewtwoshadowball_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        g::freg_all();
        const PROFTAB: [u8; 31] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_itmewtwoshadowball.txt"));
        for op in 0..31u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => {  m::it_802C4D10(&raw mut GO[0]); } // it_802C4D10
                    1 => { ret = m::it_802C4F50(&raw mut GO[0], &raw mut IT[0].x378_itemColl) as u32; } // it_802C4F50
                    2 => { let v2: i32 = fv() as i32 as i32; let v3: i32 = fv() as i32 as i32; let v4: f32 = rw() as f32; ret = pi(m::it_802C5000(&raw mut GO[0], &raw mut VA[0], v2, v3, v4)); } // it_802C5000
                    3 => { let v2: i32 = fv() as i32 as i32; let v3: i32 = ((rnd() % 5) as i32 + 1) as i32; let v4: f32 = rw() as f32; let v5: f32 = rw() as f32; ret = pi(m::it_802C519C(&raw mut GO[0], &raw mut VA[0], v2, v3, v4, v5)); } // it_802C519C
                    4 => { let v2: f32 = rw() as f32; let v3: f32 = ((rnd() % 9) as f32 * 0.5 - 1.0) as f32; let v4: f32 = ((rnd() % 5) as i32 + 1) as f32; m::it_802C53F0(&raw mut GO[0], &raw mut VA[0], v2, v3, v4); } // it_802C53F0
                    5 => {  m::it_2725_Logic101_Destroyed(&raw mut GO[0]); } // it_2725_Logic101_Destroyed
                    6 => {  m::it_802C573C(&raw mut GO[0]); } // it_802C573C
                    7 => {  m::it_2725_Logic101_PickedUp(&raw mut GO[0]); } // it_2725_Logic101_PickedUp
                    8 => {  ret = m::itMewtwoshadowball_UnkMotion0_Anim(&raw mut GO[0]) as u32; } // itMewtwoshadowball_UnkMotion0_Anim
                    9 => {  m::itMewtwoshadowball_UnkMotion0_Phys(&raw mut GO[0]); } // itMewtwoshadowball_UnkMotion0_Phys
                    10 => {  ret = m::itMewtwoshadowball_UnkMotion0_Coll(&raw mut GO[0]) as u32; } // itMewtwoshadowball_UnkMotion0_Coll
                    11 => {  m::it_802C5B18(&raw mut GO[0], &raw mut GO[1]); } // it_802C5B18
                    12 => {  ret = m::itMewtwoshadowball_UnkMotion8_Anim(&raw mut GO[0]) as u32; } // itMewtwoshadowball_UnkMotion8_Anim
                    13 => {  m::itMewtwoshadowball_UnkMotion8_Phys(&raw mut GO[0]); } // itMewtwoshadowball_UnkMotion8_Phys
                    14 => {  ret = m::itMewtwoshadowball_UnkMotion8_Coll(&raw mut GO[0]) as u32; } // itMewtwoshadowball_UnkMotion8_Coll
                    15 => {  ret = m::itMewtwoshadowball_UnkMotion9_Anim(&raw mut GO[0]) as u32; } // itMewtwoshadowball_UnkMotion9_Anim
                    16 => {  m::itMewtwoshadowball_UnkMotion9_Phys(&raw mut GO[0]); } // itMewtwoshadowball_UnkMotion9_Phys
                    17 => {  ret = m::itMewtwoshadowball_UnkMotion9_Coll(&raw mut GO[0]) as u32; } // itMewtwoshadowball_UnkMotion9_Coll
                    18 => {  m::fn_802C5E18(&raw mut GO[0]); } // fn_802C5E18
                    19 => {  m::it_802C5E5C(&raw mut GO[0]); } // it_802C5E5C
                    20 => {  ret = m::itMewtwoshadowball_UnkMotion17_Anim(&raw mut GO[0]) as u32; } // itMewtwoshadowball_UnkMotion17_Anim
                    21 => {  m::itMewtwoshadowball_UnkMotion17_Phys(&raw mut GO[0]); } // itMewtwoshadowball_UnkMotion17_Phys
                    22 => {  ret = m::itMewtwoshadowball_UnkMotion17_Coll(&raw mut GO[0]) as u32; } // itMewtwoshadowball_UnkMotion17_Coll
                    23 => {  ret = m::itMewtwoShadowball_Logic101_DmgDealt(&raw mut GO[0]) as u32; } // itMewtwoShadowball_Logic101_DmgDealt
                    24 => {  ret = m::itMewtwoShadowball_Logic101_Clanked(&raw mut GO[0]) as u32; } // itMewtwoShadowball_Logic101_Clanked
                    25 => {  ret = m::itMewtwoShadowball_Logic101_Absorbed(&raw mut GO[0]) as u32; } // itMewtwoShadowball_Logic101_Absorbed
                    26 => {  ret = m::it_2725_Logic101_Reflected(&raw mut GO[0]) as u32; } // it_2725_Logic101_Reflected
                    27 => {  ret = m::itMewtwoShadowball_Logic101_HitShield(&raw mut GO[0]) as u32; } // itMewtwoShadowball_Logic101_HitShield
                    28 => {  ret = m::it_2725_Logic101_ShieldBounced(&raw mut GO[0]) as u32; } // it_2725_Logic101_ShieldBounced
                    29 => {  m::itMewtwoShadowball_Logic101_EvtUnk(&raw mut GO[0], &raw mut GO[1]); } // itMewtwoShadowball_Logic101_EvtUnk
                    30 => { for j in 0..18 { let row = &m::it_803F7760[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 12400);
    }
}
