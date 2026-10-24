//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/itmball.c.
//! src/tu/itmball.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py itmball`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itmball as m;
use mtu_harness::*;
#[path = "kinds_gen/itmball_gen.rs"]
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
            let a = SPECB.0[0].as_mut_ptr() as *mut itMBallAttributes;
            (*a).x0 = (rnd() % 20) as f32; (*a).x4 = (rnd() % 20) as f32;
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
        for k in 0..3 { IT[k].xDD4_itemVar.mball.set_b0((rnd() & 1) as u8); IT[k].xDD4_itemVar.mball.set_b1((rnd() & 1) as u8); }
        m::ext::Item_804A0C64.x1C = (rnd() % 20) as i32;
        if rnd() & 1 != 0 {
            IT[0].xD44_lifeTimer = (*(SPECB.0[0].as_mut_ptr() as *mut itMBallAttributes)).x4;
        } else {
            IT[0].xD44_lifeTimer = (rnd() % 4) as f32 - 1.0;
        }
}

unsafe fn hash_all() {
    hm(m::ext::Item_804A0C64.x1C as u32);
    for k in 0..3 { let mb = &mut IT[k].xDD4_itemVar.mball; hm(mb.b0() as u32); hm(mb.b1() as u32); core::ptr::write_bytes(mb as *mut _ as *mut u8, 0, core::mem::size_of::<itMBall_ItemVars>()); }
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
fn itmball_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        g::freg_all();
        const PROFTAB: [u8; 37] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_itmball.txt"));
        for op in 0..37u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => {  m::itMball_Destroyed(&raw mut GO[0]); } // itMball_Destroyed
                    1 => {  m::itMball_Spawned(&raw mut GO[0]); } // itMball_Spawned
                    2 => {  m::itMball_80297944(&raw mut GO[0]); } // itMball_80297944
                    3 => {  ret = m::itMball_Motion0_Anim(&raw mut GO[0]) as u32; } // itMball_Motion0_Anim
                    4 => {  m::itMball_Motion0_Phys(&raw mut GO[0]); } // itMball_Motion0_Phys
                    5 => {  ret = m::itMball_Motion0_Coll(&raw mut GO[0]) as u32; } // itMball_Motion0_Coll
                    6 => {  m::itMball_802979D4(&raw mut GO[0]); } // itMball_802979D4
                    7 => {  ret = m::itMball_Motion3_Anim(&raw mut GO[0]) as u32; } // itMball_Motion3_Anim
                    8 => {  m::itMball_Motion1_Phys(&raw mut GO[0]); } // itMball_Motion1_Phys
                    9 => {  ret = m::itMball_Motion1_Coll(&raw mut GO[0]) as u32; } // itMball_Motion1_Coll
                    10 => {  m::itMball_PickedUp(&raw mut GO[0]); } // itMball_PickedUp
                    11 => {  ret = m::itMball_Motion2_Anim(&raw mut GO[0]) as u32; } // itMball_Motion2_Anim
                    12 => {  m::itMball_Motion2_Phys(&raw mut GO[0]); } // itMball_Motion2_Phys
                    13 => {  m::itMball_Dropped(&raw mut GO[0]); } // itMball_Dropped
                    14 => {  m::itMball_Thrown(&raw mut GO[0]); } // itMball_Thrown
                    15 => {  m::itMball_Motion3_Phys(&raw mut GO[0]); } // itMball_Motion3_Phys
                    16 => {  ret = m::itMball_Motion3_Coll(&raw mut GO[0]) as u32; } // itMball_Motion3_Coll
                    17 => {  ret = m::itMball_DmgDealt(&raw mut GO[0]) as u32; } // itMball_DmgDealt
                    18 => {  m::itMball_EnteredAir(&raw mut GO[0]); } // itMball_EnteredAir
                    19 => {  ret = m::itMball_Motion4_Anim(&raw mut GO[0]) as u32; } // itMball_Motion4_Anim
                    20 => {  m::itMball_Motion4_Phys(&raw mut GO[0]); } // itMball_Motion4_Phys
                    21 => {  ret = m::itMball_Motion4_Coll(&raw mut GO[0]) as u32; } // itMball_Motion4_Coll
                    22 => {  m::itMball_80297CC4(&raw mut GO[0]); } // itMball_80297CC4
                    23 => {  m::itMball_OnAccessory(&raw mut GO[0]); } // itMball_OnAccessory
                    24 => {  ret = m::itMball_Motion5_Anim(&raw mut GO[0]) as u32; } // itMball_Motion5_Anim
                    25 => {  m::itMball_Motion5_Phys(&raw mut GO[0]); } // itMball_Motion5_Phys
                    26 => {  ret = m::itMball_Motion5_Coll(&raw mut GO[0]) as u32; } // itMball_Motion5_Coll
                    27 => {  m::itMball_80297E8C(&raw mut GO[0]); } // itMball_80297E8C
                    28 => {  ret = m::itMball_Motion6_Anim(&raw mut GO[0]) as u32; } // itMball_Motion6_Anim
                    29 => {  m::itMball_Motion6_Phys(&raw mut GO[0]); } // itMball_Motion6_Phys
                    30 => {  ret = m::itMball_Motion6_Coll(&raw mut GO[0]) as u32; } // itMball_Motion6_Coll
                    31 => {  ret = m::itMball_Clanked(&raw mut GO[0]) as u32; } // itMball_Clanked
                    32 => {  ret = m::itMball_Reflected(&raw mut GO[0]) as u32; } // itMball_Reflected
                    33 => {  ret = m::itMball_HitShield(&raw mut GO[0]) as u32; } // itMball_HitShield
                    34 => {  ret = m::itMball_ShieldBounced(&raw mut GO[0]) as u32; } // itMball_ShieldBounced
                    35 => {  m::itMball_EvtUnk(&raw mut GO[0], &raw mut GO[1]); } // itMball_EvtUnk
                    36 => { for j in 0..7 { let row = &m::it_803F6488[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 14800);
    }
}
