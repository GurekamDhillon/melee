//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/itbombhei.c.
//! src/tu/itbombhei.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py itbombhei`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itbombhei as m;
use mtu_harness::*;
#[path = "kinds_gen/itbombhei_gen.rs"]
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
            let a = SPECB.0[0].as_mut_ptr() as *mut itBombHeiAttributes;
            (*a).x0 = rw(); (*a).x4 = (rnd() % 12) as f32; (*a).x8 = (rnd() % 12) as f32; (*a).xC = rw();
            (*a).x10 = (rnd() % 12) as f32; (*a).x14 = (rnd() % 12) as f32; (*a).x18 = (rnd() % 12) as f32; (*a).x1C = rw();
            (*a).x20.x = (rnd() % 3) as f32 / 4.0; (*a).x20.y = (rnd() % 3) as f32 / 4.0; (*a).x20.z = rw();
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
        let bh = &mut IT[k].xDD4_itemVar.bombhei;
        bh.xDD4 = (rnd() % 12) as i32 - 3;
        bh.xDD8 = if rnd() & 1 != 0 { 1 } else { -1 };
        bh.xDDC = (rnd() & 1) as i32;
        bh.xDE0 = (rnd() & 1) as i32;
        bh.xDE4 = (rnd() & 1) as i32;
        bh.xDE8 = if rnd() & 1 != 0 { 1.0 } else { rw() };
        let i = (rnd() % 10) as i32 - 3;
        bh.xDEC = i as f32;
        let i = (rnd() % 6) as i32 - 2;
        bh.xDF0 = i as f32;
        bh.xDF4 = if rnd() & 1 != 0 { 1.0 } else { -1.0 };
        bh.xDF8 = rw();
        bh.xDFC = rw();
        bh.xE04 = rw();
        bh.xE0C.x = rw(); bh.xE0C.y = rw(); bh.xE0C.z = rw();
    }
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
fn itbombhei_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        g::freg_all();
        const PROFTAB: [u8; 59] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_itbombhei.txt"));
        for op in 0..59u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => {  ret = pi(m::it_8027D670(&raw mut VA[0])); } // it_8027D670
                    1 => {  m::it_8027D730(&raw mut GO[0]); } // it_8027D730
                    2 => {  m::itBombhei_Logic6_Spawned(&raw mut GO[0]); } // itBombhei_Logic6_Spawned
                    3 => {  m::it_8027D820(&raw mut GO[0]); } // it_8027D820
                    4 => {  m::fn_8027DAC8(&raw mut GO[0]); } // fn_8027DAC8
                    5 => {  ret = m::itBombhei_UnkMotion0_Anim(&raw mut GO[0]) as u32; } // itBombhei_UnkMotion0_Anim
                    6 => {  m::itBombhei_UnkMotion0_Phys(&raw mut GO[0]); } // itBombhei_UnkMotion0_Phys
                    7 => {  ret = m::itBombhei_UnkMotion0_Coll(&raw mut GO[0]) as u32; } // itBombhei_UnkMotion0_Coll
                    8 => {  m::it_8027DE18(&raw mut GO[0]); } // it_8027DE18
                    9 => {  ret = m::itBombhei_UnkMotion1_Anim(&raw mut GO[0]) as u32; } // itBombhei_UnkMotion1_Anim
                    10 => {  m::itBombhei_UnkMotion1_Phys(&raw mut GO[0]); } // itBombhei_UnkMotion1_Phys
                    11 => {  ret = m::itBombhei_UnkMotion1_Coll(&raw mut GO[0]) as u32; } // itBombhei_UnkMotion1_Coll
                    12 => {  m::itBombhei_Logic6_PickedUp(&raw mut GO[0]); } // itBombhei_Logic6_PickedUp
                    13 => {  ret = m::itBombhei_UnkMotion8_Anim(&raw mut GO[0]) as u32; } // itBombhei_UnkMotion8_Anim
                    14 => {  m::itBombhei_UnkMotion8_Phys(&raw mut GO[0]); } // itBombhei_UnkMotion8_Phys
                    15 => {  m::it_3F14_Logic6_Dropped(&raw mut GO[0]); } // it_3F14_Logic6_Dropped
                    16 => {  m::it_8027E978(&raw mut GO[0]); } // it_8027E978
                    17 => {  ret = m::itBombhei_UnkMotion3_Anim(&raw mut GO[0]) as u32; } // itBombhei_UnkMotion3_Anim
                    18 => {  m::itBombhei_UnkMotion3_Phys(&raw mut GO[0]); } // itBombhei_UnkMotion3_Phys
                    19 => {  ret = m::itBombhei_UnkMotion3_Coll(&raw mut GO[0]) as u32; } // itBombhei_UnkMotion3_Coll
                    20 => {  m::it_8027EE04(&raw mut GO[0]); } // it_8027EE04
                    21 => {  ret = m::itBombhei_UnkMotion2_Anim(&raw mut GO[0]) as u32; } // itBombhei_UnkMotion2_Anim
                    22 => {  m::itBombhei_UnkMotion2_Phys(&raw mut GO[0]); } // itBombhei_UnkMotion2_Phys
                    23 => {  ret = m::itBombhei_UnkMotion2_Coll(&raw mut GO[0]) as u32; } // itBombhei_UnkMotion2_Coll
                    24 => {  m::it_8027F42C(&raw mut GO[0]); } // it_8027F42C
                    25 => {  ret = m::itBombhei_UnkMotion4_Anim(&raw mut GO[0]) as u32; } // itBombhei_UnkMotion4_Anim
                    26 => {  m::itBombhei_UnkMotion4_Phys(&raw mut GO[0]); } // itBombhei_UnkMotion4_Phys
                    27 => {  ret = m::itBombhei_UnkMotion4_Coll(&raw mut GO[0]) as u32; } // itBombhei_UnkMotion4_Coll
                    28 => {  m::it_8027F8E0(&raw mut GO[0]); } // it_8027F8E0
                    29 => {  ret = m::itBombhei_UnkMotion5_Anim(&raw mut GO[0]) as u32; } // itBombhei_UnkMotion5_Anim
                    30 => {  m::itBombhei_UnkMotion5_Phys(&raw mut GO[0]); } // itBombhei_UnkMotion5_Phys
                    31 => {  ret = m::itBombhei_UnkMotion5_Coll(&raw mut GO[0]) as u32; } // itBombhei_UnkMotion5_Coll
                    32 => {  m::fn_8027FCA8(&raw mut GO[0]); } // fn_8027FCA8
                    33 => {  ret = m::itBombhei_UnkMotion6_Anim(&raw mut GO[0]) as u32; } // itBombhei_UnkMotion6_Anim
                    34 => {  m::itBombhei_UnkMotion6_Phys(&raw mut GO[0]); } // itBombhei_UnkMotion6_Phys
                    35 => {  m::fn_8028007C(&raw mut GO[0]); } // fn_8028007C
                    36 => {  ret = m::itBombhei_UnkMotion6_Coll(&raw mut GO[0]) as u32; } // itBombhei_UnkMotion6_Coll
                    37 => {  m::it_3F14_Logic6_Thrown(&raw mut GO[0]); } // it_3F14_Logic6_Thrown
                    38 => {  ret = m::itBombhei_UnkMotion10_Anim(&raw mut GO[0]) as u32; } // itBombhei_UnkMotion10_Anim
                    39 => {  m::itBombhei_UnkMotion10_Phys(&raw mut GO[0]); } // itBombhei_UnkMotion10_Phys
                    40 => {  m::fn_80280974(&raw mut GO[0]); } // fn_80280974
                    41 => {  ret = m::itBombhei_UnkMotion10_Coll(&raw mut GO[0]) as u32; } // itBombhei_UnkMotion10_Coll
                    42 => {  m::it_80280B60(&raw mut GO[0]); } // it_80280B60
                    43 => {  m::it_80280DC0(&raw mut GO[0]); } // it_80280DC0
                    44 => {  ret = m::it_3F14_Logic6_DmgDealt(&raw mut GO[0]) as u32; } // it_3F14_Logic6_DmgDealt
                    45 => {  ret = m::it_3F14_Logic6_DmgReceived(&raw mut GO[0]) as u32; } // it_3F14_Logic6_DmgReceived
                    46 => {  ret = m::itBombhei_UnkMotion11_Anim(&raw mut GO[0]) as u32; } // itBombhei_UnkMotion11_Anim
                    47 => {  m::itBombhei_UnkMotion11_Phys(&raw mut GO[0]); } // itBombhei_UnkMotion11_Phys
                    48 => {  ret = m::itBombhei_UnkMotion11_Coll(&raw mut GO[0]) as u32; } // itBombhei_UnkMotion11_Coll
                    49 => {  m::it_3F14_Logic6_EnteredAir(&raw mut GO[0]); } // it_3F14_Logic6_EnteredAir
                    50 => {  ret = m::itBombhei_UnkMotion12_Anim(&raw mut GO[0]) as u32; } // itBombhei_UnkMotion12_Anim
                    51 => {  m::itBombhei_UnkMotion12_Phys(&raw mut GO[0]); } // itBombhei_UnkMotion12_Phys
                    52 => {  ret = m::itBombhei_UnkMotion12_Coll(&raw mut GO[0]) as u32; } // itBombhei_UnkMotion12_Coll
                    53 => {  ret = m::itBombhei_Logic6_Clanked(&raw mut GO[0]) as u32; } // itBombhei_Logic6_Clanked
                    54 => {  ret = m::itBombhei_Logic6_Reflected(&raw mut GO[0]) as u32; } // itBombhei_Logic6_Reflected
                    55 => {  ret = m::it_3F14_Logic6_HitShield(&raw mut GO[0]) as u32; } // it_3F14_Logic6_HitShield
                    56 => {  ret = m::it_3F14_Logic6_ShieldBounced(&raw mut GO[0]) as u32; } // it_3F14_Logic6_ShieldBounced
                    57 => {  m::itBombhei_Logic6_EvtUnk(&raw mut GO[0], &raw mut GO[1]); } // itBombhei_Logic6_EvtUnk
                    58 => { for j in 0..13 { let row = &m::it_803F54D8[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 23600);
    }
}
