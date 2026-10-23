//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/ittarucann.c.
//! src/tu/ittarucann.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py ittarucann`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::ittarucann as m;
use mtu_harness::*;
#[path = "kinds_gen/ittarucann_gen.rs"]
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

static mut MD: ItemModelDesc = unsafe { core::mem::zeroed() };
static mut JNT: [HSD_Joint; 1] = unsafe { core::mem::zeroed() };


unsafe fn spec_fill(prof: i32) {
    match prof {
        0 => {
            let a = SPECB.0[0].as_mut_ptr() as *mut f32;
            for i in 0..21 { *a.add(i) = rw(); }
            let ai = a as *mut i32;
            *ai.add(10) = (rnd() % 5) as i32; *ai.add(11) = (rnd() % 5) as i32; *ai.add(12) = (rnd() % 20) as i32; *ai.add(13) = (rnd() % 400) as i32;
            for i in 14..18 { *ai.add(i) = (rnd() % 10) as i32; }
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
        let tc = &mut IT[k].xDD4_itemVar.tarucann;
        tc.x4 = (rnd() % 4) as i32 - 1;
        tc.x8 = rg(); tc.xC = rg(); tc.x10 = rg(); tc.x14.x = rg(); tc.x14.y = rg(); tc.x14.z = rg();
        tc.x20 = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        tc.x24 = (rnd() % 4) as i32; tc.x28 = rg();
        IT[k].grab_victim = if rnd() & 1 != 0 { &raw mut GO[(k + 2) % 3] } else { null_mut() };
        IT[k].xDAC_itcmd_var0 = (rnd() % 3) as u32;
        IT[k].xDB0_itcmd_var1 = (rnd() & 1) as u32;
        IT[k].msid = (rnd() % 8) as i32;
        IT[k].xD44_lifeTimer = (rnd() % 4) as f32;
        IT[k].facing_dir = if rnd() % 3 == 0 { rw() } else if rnd() & 1 != 0 { 1.0 } else { -1.0 };
        IT[k].ground_or_air = if rnd() & 1 != 0 { GA_Air } else { GA_Ground };
        IT[k].x378_itemColl.env_flags = if rnd() % 5 == 0 { 0 } else if rnd() & 1 != 0 { 0x18000 } else if rnd() & 1 != 0 { 0x3F } else if rnd() & 1 != 0 { 0xFC0 } else { 0x18000 | 0x3F };
        ART[k].x10_modelDesc = &raw mut MD;
        MD.x0_joint = &raw mut JNT[0];
    }
}

unsafe fn hash_all() {
    for k in 0..3 { hm(pi(IT[k].xDD4_itemVar.tarucann.x20)); IT[k].xDD4_itemVar.tarucann.x20 = null_mut(); hm(pi(IT[k].grab_victim)); IT[k].grab_victim = null_mut(); }
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
fn ittarucann_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        preg(&raw const MD, core::mem::size_of_val(&MD)); preg(&raw const JNT, core::mem::size_of_val(&JNT));
        g::freg_all();
        const PROFTAB: [u8; 44] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_ittarucann.txt"));
        for op in 0..44u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => {  m::it_80295ED4(&raw mut GO[0]); } // it_80295ED4
                    1 => {  m::it_80295F38(&raw mut GO[0]); } // it_80295F38
                    2 => {  ret = m::it_802960B8(&raw mut GO[0]) as u32; } // it_802960B8
                    3 => {  ret = m::it_802960CC(&raw mut GO[0]) as u32; } // it_802960CC
                    4 => {  m::it_3F14_Logic5_Destroyed(&raw mut GO[0]); } // it_3F14_Logic5_Destroyed
                    5 => {  m::it_3F14_Logic5_Spawned(&raw mut GO[0]); } // it_3F14_Logic5_Spawned
                    6 => {  ret = m::it_802961E8(&raw mut GO[0]) as u32; } // it_802961E8
                    7 => {  m::it_802962E0(&raw mut GO[0]); } // it_802962E0
                    8 => {  ret = m::itTarucann_UnkMotion0_Anim(&raw mut GO[0]) as u32; } // itTarucann_UnkMotion0_Anim
                    9 => {  m::itTarucann_UnkMotion0_Phys(&raw mut GO[0]); } // itTarucann_UnkMotion0_Phys
                    10 => {  ret = m::itTarucann_UnkMotion0_Coll(&raw mut GO[0]) as u32; } // itTarucann_UnkMotion0_Coll
                    11 => {  m::it_80296694(&raw mut GO[0]); } // it_80296694
                    12 => {  ret = m::itTarucann_UnkMotion1_Anim(&raw mut GO[0]) as u32; } // itTarucann_UnkMotion1_Anim
                    13 => {  m::itTarucann_UnkMotion1_Phys(&raw mut GO[0]); } // itTarucann_UnkMotion1_Phys
                    14 => {  ret = m::itTarucann_UnkMotion1_Coll(&raw mut GO[0]) as u32; } // itTarucann_UnkMotion1_Coll
                    15 => {  m::itTaruCann_Logic5_PickedUp(&raw mut GO[0]); } // itTaruCann_Logic5_PickedUp
                    16 => {  ret = m::itTarucann_UnkMotion2_Anim(&raw mut GO[0]) as u32; } // itTarucann_UnkMotion2_Anim
                    17 => {  m::itTarucann_UnkMotion2_Phys(&raw mut GO[0]); } // itTarucann_UnkMotion2_Phys
                    18 => {  m::it_3F14_Logic5_Dropped(&raw mut GO[0]); } // it_3F14_Logic5_Dropped
                    19 => {  m::it_3F14_Logic5_Thrown(&raw mut GO[0]); } // it_3F14_Logic5_Thrown
                    20 => {  m::it_802969D8(&raw mut GO[0]); } // it_802969D8
                    21 => {  ret = m::itTarucann_UnkMotion6_Anim(&raw mut GO[0]) as u32; } // itTarucann_UnkMotion6_Anim
                    22 => {  m::itTarucann_UnkMotion6_Phys(&raw mut GO[0]); } // itTarucann_UnkMotion6_Phys
                    23 => {  ret = m::itTarucann_UnkMotion6_Coll(&raw mut GO[0]) as u32; } // itTarucann_UnkMotion6_Coll
                    24 => {  m::it_80296E88(&raw mut GO[0]); } // it_80296E88
                    25 => {  m::it_80296EA8(&raw mut GO[0]); } // it_80296EA8
                    26 => {  m::it_80296EF0(&raw mut GO[0]); } // it_80296EF0
                    27 => {  ret = m::itTarucann_UnkMotion8_Anim(&raw mut GO[0]) as u32; } // itTarucann_UnkMotion8_Anim
                    28 => {  m::itTarucann_UnkMotion8_Phys(&raw mut GO[0]); } // itTarucann_UnkMotion8_Phys
                    29 => {  ret = m::itTarucann_UnkMotion8_Coll(&raw mut GO[0]) as u32; } // itTarucann_UnkMotion8_Coll
                    30 => {  ret = m::itTarucann_UnkMotion7_Anim(&raw mut GO[0]) as u32; } // itTarucann_UnkMotion7_Anim
                    31 => {  ret = (m::itTarucann_UnkMotion7_Phys(&raw mut GO[0])).to_bits(); } // itTarucann_UnkMotion7_Phys
                    32 => {  ret = m::itTarucann_UnkMotion7_Coll(&raw mut GO[0]) as u32; } // itTarucann_UnkMotion7_Coll
                    33 => {  m::it_802975F4(&raw mut GO[0]); } // it_802975F4
                    34 => {  ret = m::itTarucann_UnkMotion9_Anim(&raw mut GO[0]) as u32; } // itTarucann_UnkMotion9_Anim
                    35 => {  m::itTarucann_UnkMotion9_Phys(&raw mut GO[0]); } // itTarucann_UnkMotion9_Phys
                    36 => {  ret = m::itTarucann_UnkMotion9_Coll(&raw mut GO[0]) as u32; } // itTarucann_UnkMotion9_Coll
                    37 => {  ret = m::it_80297790(&raw mut GO[0]) as u32; } // it_80297790
                    38 => {  ret = m::itTaruCann_Logic5_DmgDealt(&raw mut GO[0]) as u32; } // itTaruCann_Logic5_DmgDealt
                    39 => {  ret = m::itTaruCann_Logic5_Clanked(&raw mut GO[0]) as u32; } // itTaruCann_Logic5_Clanked
                    40 => {  ret = m::itTaruCann_Logic5_HitShield(&raw mut GO[0]) as u32; } // itTaruCann_Logic5_HitShield
                    41 => {  ret = m::itTaruCann_Logic5_Reflected(&raw mut GO[0]) as u32; } // itTaruCann_Logic5_Reflected
                    42 => {  m::itTaruCann_Logic5_EvtUnk(&raw mut GO[0], &raw mut GO[1]); } // itTaruCann_Logic5_EvtUnk
                    43 => { for j in 0..10 { let row = &m::it_803F63C0[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 17600);
    }
}
