//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/itlinkarrow.c.
//! src/tu/itlinkarrow.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py itlinkarrow`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itlinkarrow as m;
use mtu_harness::*;
#[path = "kinds_gen/itlinkarrow_gen.rs"]
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
            for q in 0..3usize {
            let a = SPECB.0[q].as_mut_ptr() as *mut itLinkArrowAttributes;
            (*a).x0 = (rnd() % 100) as f32; (*a).x4 = rw(); (*a).x8 = rw(); (*a).xC = rw(); (*a).x10 = rw(); (*a).x14 = rw();
            (*a).x18 = (rnd() % 100) as f32; (*a).x1C = rw(); (*a).x20 = rw();
            (*a).x24 = if rnd() & 1 != 0 { &raw mut JO[3 + q] as *mut HSD_Joint } else { null_mut() };
            (*a).x28 = if rnd() & 1 != 0 { &raw mut JO[3 + q] as *mut HSD_Joint } else { null_mut() };
            (*a).x2C = rw();
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
        let la = &mut IT[k].xDD4_itemVar.linkarrow;
        la.x18.x = rw(); la.x18.y = rw(); la.x18.z = rw();
        la.x24.x = rw(); la.x24.y = rw(); la.x24.z = rw();
        la.x30.x = rw(); la.x30.y = rw(); la.x30.z = rw();
        la.x3C.x = rw(); la.x3C.y = rw(); la.x3C.z = rw();
        la.x48.x = rw(); la.x48.y = rw(); la.x48.z = rw();
        la.x54.x = rw(); la.x54.y = rw(); la.x54.z = rw(); la.x54.w = rw();
        la.x64.x = rw(); la.x64.y = rw(); la.x64.z = rw(); la.x64.w = rw();
        la.x74.x = rw(); la.x74.y = rw(); la.x74.z = rw(); la.x74.w = rw();
        la.x84.x = rw(); la.x84.y = rw(); la.x84.z = rw(); la.x84.w = rw();
        la.x94 = rw(); la.x98 = rw();
        la.x9C = if rnd() & 7 == 0 { rnd() % 12 } else { rnd() % 9 };
        la.xA0 = (rnd() % 2) as i32; la.xA4 = rnd() % 10; la.xA8 = rw(); la.xAC = rw();
        la.xB0 = (rnd() % 4) as i32;
        la.xB4[0] = if rnd() & 1 != 0 { &raw mut JO[6 + 14 * k + 0] } else { null_mut() };
        la.xB4[1] = if rnd() & 1 != 0 { &raw mut JO[6 + 14 * k + 1] } else { null_mut() };
        la.xB4[2] = null_mut();
        la.xC0 = rw();
        la.xC4 = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        la.xC8 = rw(); la.xCC = rw(); la.xD0 = null_mut(); la.xD4 = rw(); la.xD8 = rw(); la.xDC = rw();
        la.xE0 = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        IT[k].owner = if rnd() & 3 != 0 { la.xE0 } else if rnd() & 1 != 0 { &raw mut GO[(k + 2) % 3] } else { null_mut() };
        la.xE4 = if rnd() & 3 != 0 { (rnd() % 20) as i32 } else { -1 };
        la.xE8 = rw(); la.xEC = rw(); la.xF0 = if rnd() & 3 == 0 { 0x12B + (rnd() % 3) as i32 } else { (rnd() % 0x130) as i32 - 1 };
        IT[k].kind = if rnd() & 1 != 0 { if rnd() % 4 == 0 { It_Kind_Kirby_LinkArrow } else { It_Kind_Link_Arrow } } else { if rnd() % 4 == 0 { It_Kind_Kirby_CLinkArrow } else { It_Kind_CLink_Arrow } };
        if rnd() & 7 == 0 { IT[k].kind = (rnd() % 400) as i32; }
        IT[k].xCF4_fighterGObjUnk = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        IT[k].xDAC_itcmd_var0 = if rnd() & 3 != 0 { rnd() % 3 } else { 1 }; IT[k].xDB0_itcmd_var1 = rnd() % 3; IT[k].xDB4_itcmd_var2 = rnd() % 2;
    }
}

unsafe fn hash_all() {
    for k in 0..3 {
        let la = &mut IT[k].xDD4_itemVar.linkarrow;
        hm(pi(la.xB4[0])); hm(pi(la.xB4[1])); hm(pi(la.xB4[2])); hm(pi(la.xC4)); hm(pi(la.xD0)); hm(pi(la.xE0));
        la.xB4[0] = null_mut(); la.xB4[1] = null_mut(); la.xB4[2] = null_mut(); la.xC4 = null_mut(); la.xD0 = null_mut(); la.xE0 = null_mut();
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
fn itlinkarrow_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        g::freg_all();
        const PROFTAB: [u8; 33] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_itlinkarrow.txt"));
        for op in 0..33u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => { let v1: i32 = (rnd() % 3) as i32 as i32; m::it_802A7D8C(&raw mut GO[0], v1); } // it_802A7D8C
                    1 => {  m::it_802A7E40(&raw mut GO[0]); } // it_802A7E40
                    2 => {  m::itLinkArrow_802A81C4(&raw mut GO[0]); } // itLinkArrow_802A81C4
                    3 => {  m::it_802A8330(&raw mut GO[0]); } // it_802A8330
                    4 => {  m::it_802A8398(&raw mut GO[0], &raw mut VA[0], &raw mut VA[1]); } // it_802A8398
                    5 => { let v0: f32 = rw() as f32; let v3: Fighter_Part = fv() as i32 as Fighter_Part; let v4: i32 = (rnd() % 3) as i32 as i32; ret = pi(m::it_802A83E0(v0, &raw mut GO[0], &raw mut VA[0], v3, v4)); } // it_802A83E0
                    6 => { let v2 = rw(); let v3 = rw(); let v4 = rw(); let cond = !IT[0].xDD4_itemVar.linkarrow.xE0.is_null() && IT[0].owner == IT[0].xDD4_itemVar.linkarrow.xE0; let r = m::itLinkArrow_802A850C(&raw mut GO[0], &raw mut VA[0], &raw mut VA[1], v2, v3, v4) as u32; if cond { ret = r; } } // itLinkArrow_802A850C
                    7 => {  m::itLinkArrow_Logic98_Destroyed(&raw mut GO[0]); } // itLinkArrow_Logic98_Destroyed
                    8 => {  m::it_802A8A7C(&raw mut GO[0]); } // it_802A8A7C
                    9 => {  m::itLinkArrow_Logic98_PickedUp(&raw mut GO[0]); } // itLinkArrow_Logic98_PickedUp
                    10 => {  ret = m::itLinkarrow_UnkMotion0_Anim(&raw mut GO[0]) as u32; } // itLinkarrow_UnkMotion0_Anim
                    11 => {  m::itLinkarrow_UnkMotion0_Phys(&raw mut GO[0]); } // itLinkarrow_UnkMotion0_Phys
                    12 => {  ret = m::itLinkarrow_UnkMotion0_Coll(&raw mut GO[0]) as u32; } // itLinkarrow_UnkMotion0_Coll
                    13 => {  m::it_802A8C7C(&raw mut GO[0]); } // it_802A8C7C
                    14 => {  ret = m::itLinkarrow_UnkMotion1_Anim(&raw mut GO[0]) as u32; } // itLinkarrow_UnkMotion1_Anim
                    15 => {  m::itLinkarrow_UnkMotion1_Phys(&raw mut GO[0]); } // itLinkarrow_UnkMotion1_Phys
                    16 => {  ret = m::itLinkarrow_UnkMotion1_Coll(&raw mut GO[0]) as u32; } // itLinkarrow_UnkMotion1_Coll
                    17 => {  ret = m::itLinkarrow_UnkMotion2_Anim(&raw mut GO[0]) as u32; } // itLinkarrow_UnkMotion2_Anim
                    18 => {  m::itLinkarrow_UnkMotion2_Phys(&raw mut GO[0]); } // itLinkarrow_UnkMotion2_Phys
                    19 => {  ret = m::itLinkarrow_UnkMotion2_Coll(&raw mut GO[0]) as u32; } // itLinkarrow_UnkMotion2_Coll
                    20 => {  ret = m::itLinkarrow_UnkMotion3_Anim(&raw mut GO[0]) as u32; } // itLinkarrow_UnkMotion3_Anim
                    21 => {  m::itLinkarrow_UnkMotion3_Phys(&raw mut GO[0]); } // itLinkarrow_UnkMotion3_Phys
                    22 => {  ret = m::itLinkarrow_UnkMotion3_Coll(&raw mut GO[0]) as u32; } // itLinkarrow_UnkMotion3_Coll
                    23 => {  m::it_802A9458(&raw mut GO[0]); } // it_802A9458
                    24 => {  ret = m::itLinkarrow_UnkMotion4_Anim(&raw mut GO[0]) as u32; } // itLinkarrow_UnkMotion4_Anim
                    25 => {  m::itLinkarrow_UnkMotion4_Phys(&raw mut GO[0]); } // itLinkarrow_UnkMotion4_Phys
                    26 => {  ret = m::itLinkarrow_UnkMotion4_Coll(&raw mut GO[0]) as u32; } // itLinkarrow_UnkMotion4_Coll
                    27 => {  ret = m::itLinkArrow_Logic98_DmgDealt(&raw mut GO[0]) as u32; } // itLinkArrow_Logic98_DmgDealt
                    28 => {  ret = m::itLinkArrow_Logic98_HitShield(&raw mut GO[0]) as u32; } // itLinkArrow_Logic98_HitShield
                    29 => {  ret = m::itLinkArrow_Logic98_Clanked(&raw mut GO[0]) as u32; } // itLinkArrow_Logic98_Clanked
                    30 => {  ret = m::itLinkArrow_Logic98_Reflected(&raw mut GO[0]) as u32; } // itLinkArrow_Logic98_Reflected
                    31 => {  m::itLinkArrow_Logic98_EvtUnk(&raw mut GO[0], &raw mut GO[1]); } // itLinkArrow_Logic98_EvtUnk
                    32 => { for j in 0..5 { let row = &m::it_803F6A28[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 13200);
    }
}
