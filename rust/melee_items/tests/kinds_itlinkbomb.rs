//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/itlinkbomb.c.
//! src/tu/itlinkbomb.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py itlinkbomb`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itlinkbomb as m;
use mtu_harness::*;
#[path = "kinds_gen/itlinkbomb_gen.rs"]
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

static mut ISA: [ItemStateArray; 3] = unsafe { core::mem::zeroed() };

unsafe fn spec_fill(prof: i32) {
    match prof {
        0 => {
            for q in 0..3usize {
            let a = SPECB.0[q].as_mut_ptr() as *mut itLinkBombAttributes;
            (*a).lifetime = rnd() % 100; (*a).x4 = rw(); (*a).x8 = rw(); (*a).xC = rnd() % 60; (*a).x10 = (rnd() % 10) as i32;
            (*a).x14 = rw(); (*a).x18 = rw(); (*a).x1C = rw(); (*a).x20 = rw(); (*a).x24 = rw(); (*a).x28 = rw(); (*a).x2C = rw(); (*a).x30 = rw();
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
        ART[k].xC_itemStates = &raw mut ISA[k]; IT[k].xD0_itemStateDesc = null_mut();
        core::ptr::write_bytes(&raw mut IT[k].xDD4_itemVar.linkbomb.x0 as *mut u8, 0, core::mem::size_of_val(&IT[k].xDD4_itemVar.linkbomb.x0)); IT[k].xDD4_itemVar.linkbomb.x0.x0.set_b0((rnd() & 1) as u8); IT[k].xDD4_itemVar.linkbomb.x0.x0.set_b1((rnd() & 1) as u8); IT[k].xDD4_itemVar.linkbomb.x0.x0.set_b2((rnd() & 1) as u8);
        IT[k].xDD4_itemVar.linkbomb.x4 = rw(); IT[k].xDD4_itemVar.linkbomb.x8 = rw(); IT[k].xDD4_itemVar.linkbomb.xC = rw();
        IT[k].xDD4_itemVar.linkbomb.x10 = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        IT[k].xD44_lifeTimer = if rnd() % 3 == 0 { 0.0 } else { (rnd() % 80) as f32 - 5.0 };
        IT[k].xCA0 = (rnd() % 12) as i32;
        IT[k].xCCC_incDamageDirection = if rnd() & 1 != 0 { 1.0 } else { -1.0 };
        IT[k].owner = if rnd() & 1 != 0 { IT[k].xDD4_itemVar.linkbomb.x10 } else if rnd() & 1 != 0 { &raw mut GO[(k + 2) % 3] } else { null_mut() };
    }
}

unsafe fn hash_all() {
    for k in 0..3 {
        let lb = &mut IT[k].xDD4_itemVar.linkbomb;
        hm(pi(lb.x10));
        hm(lb.x0.x0.b0() as u32); hm(lb.x0.x0.b1() as u32); hm(lb.x0.x0.b2() as u32); hm(lb.x0.x0.b3() as u32);
        hm(lb.x0.x0.b4() as u32); hm(lb.x0.x0.b5() as u32); hm(lb.x0.x0.b6() as u32); hm(lb.x0.x0.b7() as u32);
        lb.x10 = null_mut(); core::ptr::write_bytes(&raw mut lb.x0 as *mut u8, 0, core::mem::size_of_val(&lb.x0));
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
fn itlinkbomb_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        preg(&raw const ISA, core::mem::size_of_val(&ISA));
        g::freg_all();
        const PROFTAB: [u8; 38] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_itlinkbomb.txt"));
        for op in 0..38u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => {  m::it_8029D968(&raw mut GO[0]); } // it_8029D968
                    1 => { let v1: i32 = (rnd() % 8) as i32 as i32; let v2: Item_StateChangeFlags = rnd() % 32 as Item_StateChangeFlags; m::it_8029D9A4(&raw mut GO[0], v1, v2); } // it_8029D9A4
                    2 => {  m::it_8029DB5C(&raw mut GO[0]); } // it_8029DB5C
                    3 => { let v2: u32 = rnd() % 32 as u32; let v3: i32 = (rnd() % 400) as i32 as i32; let v4: f32 = rw() as f32; ret = pi(m::it_8029DD58(&raw mut GO[0], &raw mut VA[0], v2, v3, v4)); } // it_8029DD58
                    4 => {  m::it_8029DEB0(&raw mut GO[0]); } // it_8029DEB0
                    5 => {  ret = m::itLinkbomb_UnkMotion0_Anim(&raw mut GO[0]) as u32; } // itLinkbomb_UnkMotion0_Anim
                    6 => {  m::itLinkbomb_UnkMotion0_Phys(&raw mut GO[0]); } // itLinkbomb_UnkMotion0_Phys
                    7 => {  m::fn_8029E21C(&raw mut GO[0]); } // fn_8029E21C
                    8 => {  ret = m::itLinkbomb_UnkMotion1_Anim(&raw mut GO[0]) as u32; } // itLinkbomb_UnkMotion1_Anim
                    9 => {  m::itLinkbomb_UnkMotion1_Phys(&raw mut GO[0]); } // itLinkbomb_UnkMotion1_Phys
                    10 => {  ret = m::itLinkbomb_UnkMotion1_Coll(&raw mut GO[0]) as u32; } // itLinkbomb_UnkMotion1_Coll
                    11 => {  m::it_8029E5D0(&raw mut GO[0]); } // it_8029E5D0
                    12 => {  ret = m::itLinkbomb_UnkMotion2_Anim(&raw mut GO[0]) as u32; } // itLinkbomb_UnkMotion2_Anim
                    13 => {  m::itLinkbomb_UnkMotion2_Phys(&raw mut GO[0]); } // itLinkbomb_UnkMotion2_Phys
                    14 => {  ret = m::itLinkbomb_UnkMotion2_Coll(&raw mut GO[0]) as u32; } // itLinkbomb_UnkMotion2_Coll
                    15 => {  m::it_8029EC34(&raw mut GO[0]); } // it_8029EC34
                    16 => {  ret = m::itLinkbomb_UnkMotion3_Anim(&raw mut GO[0]) as u32; } // itLinkbomb_UnkMotion3_Anim
                    17 => {  m::itLinkbomb_UnkMotion3_Phys(&raw mut GO[0]); } // itLinkbomb_UnkMotion3_Phys
                    18 => {  ret = m::itLinkbomb_UnkMotion3_Coll(&raw mut GO[0]) as u32; } // itLinkbomb_UnkMotion3_Coll
                    19 => {  m::it_8029F18C(&raw mut GO[0]); } // it_8029F18C
                    20 => {  ret = m::itLinkbomb_UnkMotion4_Anim(&raw mut GO[0]) as u32; } // itLinkbomb_UnkMotion4_Anim
                    21 => {  m::itLinkbomb_UnkMotion4_Phys(&raw mut GO[0]); } // itLinkbomb_UnkMotion4_Phys
                    22 => {  ret = m::itLinkbomb_UnkMotion4_Coll(&raw mut GO[0]) as u32; } // itLinkbomb_UnkMotion4_Coll
                    23 => {  m::it_8029F69C(&raw mut GO[0]); } // it_8029F69C
                    24 => {  ret = m::itLinkbomb_UnkMotion5_Anim(&raw mut GO[0]) as u32; } // itLinkbomb_UnkMotion5_Anim
                    25 => {  m::itLinkbomb_UnkMotion5_Phys(&raw mut GO[0]); } // itLinkbomb_UnkMotion5_Phys
                    26 => {  ret = m::itLinkbomb_UnkMotion5_Coll(&raw mut GO[0]) as u32; } // itLinkbomb_UnkMotion5_Coll
                    27 => {  ret = m::it_8029F960(&raw mut GO[0]) as u32; } // it_8029F960
                    28 => {  ret = m::itLinkBomb_Logic16_DmgReceived(&raw mut GO[0]) as u32; } // itLinkBomb_Logic16_DmgReceived
                    29 => {  m::itLinkBomb_Logic16_EnteredAir(&raw mut GO[0]); } // itLinkBomb_Logic16_EnteredAir
                    30 => {  ret = m::itLinkbomb_UnkMotion6_Anim(&raw mut GO[0]) as u32; } // itLinkbomb_UnkMotion6_Anim
                    31 => {  m::itLinkbomb_UnkMotion6_Phys(&raw mut GO[0]); } // itLinkbomb_UnkMotion6_Phys
                    32 => {  ret = m::itLinkbomb_UnkMotion6_Coll(&raw mut GO[0]) as u32; } // itLinkbomb_UnkMotion6_Coll
                    33 => {  ret = m::itLinkBomb_Logic16_Reflected(&raw mut GO[0]) as u32; } // itLinkBomb_Logic16_Reflected
                    34 => {  ret = m::itLinkBomb_Logic16_HitShield(&raw mut GO[0]) as u32; } // itLinkBomb_Logic16_HitShield
                    35 => {  ret = m::itLinkBomb_Logic16_ShieldBounced(&raw mut GO[0]) as u32; } // itLinkBomb_Logic16_ShieldBounced
                    36 => {  m::it_8029FD84(&raw mut GO[0], &raw mut GO[1]); } // it_8029FD84
                    37 => { for j in 0..7 { let row = &m::it_803F6888[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 15200);
    }
}
