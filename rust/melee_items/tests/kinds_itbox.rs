//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/itbox.c.
//! src/tu/itbox.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py itbox`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itbox as m;
use mtu_harness::*;
#[path = "kinds_gen/itbox_gen.rs"]
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
            let a = SPECB.0[0].as_mut_ptr() as *mut itBoxAttributes;
            (*a).spawn_weight_0 = (rnd() % 5) as i32; (*a).spawn_weight_1 = (rnd() % 5) as i32; (*a).spawn_weight_2 = (rnd() % 5) as i32;
            (*a).empty_weight = (rnd() % 5) as i32; (*a).special_spawn_weight = (rnd() % 5) as i32;
            (*a).damage_threshold = (rnd() % 100) as f32; (*a).bounce_angle_threshold = rw(); (*a).break_vel_threshold = rw();
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
        let bx = &mut IT[k].xDD4_itemVar.r#box;
        bx.opened = (rnd() & 1) as i32;
        bx.despawn_timer = (rnd() % 5) as i32 - 1;
        bx.spawned_gobj = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        bx.rot_vel_x = rg();
        bx.rot_vel_y = rg();
        IT[k].xC9C = (rnd() % 120) as i32;
        if rnd() & 1 != 0 {
            IT[k].x378_itemColl.env_flags = Collide_FloorPush | Collide_FloorHug;
        }
    }
}

unsafe fn hash_all() {
    for k in 0..3 { hm(pi(IT[k].xDD4_itemVar.r#box.spawned_gobj)); IT[k].xDD4_itemVar.r#box.spawned_gobj = null_mut(); }
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
fn itbox_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        g::freg_all();
        const PROFTAB: [u8; 48] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_itbox.txt"));
        for op in 0..48u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => {  ret = pi(m::it_80286088(&raw mut GO[0])); } // it_80286088
                    1 => {  m::itBox_Logic1_Spawned(&raw mut GO[0]); } // itBox_Logic1_Spawned
                    2 => {  m::itBox_Logic1_Destroyed(&raw mut GO[0]); } // itBox_Logic1_Destroyed
                    3 => { let v1: i32 = (rnd() % 6) as i32 as i32; let v2: i32 = (rnd() % 6) as i32 as i32; let v3: i32 = (rnd() % 6) as i32 as i32; let v4: i32 = (rnd() % 6) as i32 as i32; m::it_80286248(&raw mut GO[0], v1, v2, v3, v4); } // it_80286248
                    4 => { let v1: i32 = (rnd() % 6) as i32 as i32; let v2: i32 = (rnd() % 6) as i32 as i32; let v3: i32 = (rnd() % 6) as i32 as i32; let v4: i32 = (rnd() % 6) as i32 as i32; ret = m::it_80286340(&raw mut GO[0], v1, v2, v3, v4) as u32; } // it_80286340
                    5 => {  ret = m::it_802863BC(&raw mut GO[0]) as u32; } // it_802863BC
                    6 => {  m::fn_80286480(&raw mut GO[0]); } // fn_80286480
                    7 => {  ret = m::itBox_UnkMotion0_Anim(&raw mut GO[0]) as u32; } // itBox_UnkMotion0_Anim
                    8 => {  m::itBox_UnkMotion0_Phys(&raw mut GO[0]); } // itBox_UnkMotion0_Phys
                    9 => {  ret = m::itBox_UnkMotion0_Coll(&raw mut GO[0]) as u32; } // itBox_UnkMotion0_Coll
                    10 => {  m::it_8028655C(&raw mut GO[0]); } // it_8028655C
                    11 => {  ret = m::itBox_UnkMotion4_Anim(&raw mut GO[0]) as u32; } // itBox_UnkMotion4_Anim
                    12 => {  m::itBox_UnkMotion1_Phys(&raw mut GO[0]); } // itBox_UnkMotion1_Phys
                    13 => {  ret = m::itBox_UnkMotion1_Coll(&raw mut GO[0]) as u32; } // itBox_UnkMotion1_Coll
                    14 => {  m::itBox_Logic1_PickedUp(&raw mut GO[0]); } // itBox_Logic1_PickedUp
                    15 => {  ret = m::itBox_UnkMotion2_Anim(&raw mut GO[0]) as u32; } // itBox_UnkMotion2_Anim
                    16 => {  m::itBox_UnkMotion2_Phys(&raw mut GO[0]); } // itBox_UnkMotion2_Phys
                    17 => {  m::itBox_Logic1_Thrown(&raw mut GO[0]); } // itBox_Logic1_Thrown
                    18 => {  m::itBox_UnkMotion4_Phys(&raw mut GO[0]); } // itBox_UnkMotion4_Phys
                    19 => {  ret = m::itBox_UnkMotion3_Coll(&raw mut GO[0]) as u32; } // itBox_UnkMotion3_Coll
                    20 => {  m::itBox_Logic1_Dropped(&raw mut GO[0]); } // itBox_Logic1_Dropped
                    21 => {  ret = m::itBox_UnkMotion4_Coll(&raw mut GO[0]) as u32; } // itBox_UnkMotion4_Coll
                    22 => {  m::it_80286AA4(&raw mut GO[0]); } // it_80286AA4
                    23 => {  ret = m::itBox_UnkMotion6_Anim(&raw mut GO[0]) as u32; } // itBox_UnkMotion6_Anim
                    24 => {  m::itBox_UnkMotion6_Phys(&raw mut GO[0]); } // itBox_UnkMotion6_Phys
                    25 => {  ret = m::itBox_UnkMotion6_Coll(&raw mut GO[0]) as u32; } // itBox_UnkMotion6_Coll
                    26 => {  m::it_80286BA0(&raw mut GO[0]); } // it_80286BA0
                    27 => {  ret = m::itBox_UnkMotion7_Anim(&raw mut GO[0]) as u32; } // itBox_UnkMotion7_Anim
                    28 => {  m::itBox_UnkMotion7_Phys(&raw mut GO[0]); } // itBox_UnkMotion7_Phys
                    29 => {  ret = m::itBox_UnkMotion7_Coll(&raw mut GO[0]) as u32; } // itBox_UnkMotion7_Coll
                    30 => {  ret = m::itBox_Logic1_DmgDealt(&raw mut GO[0]) as u32; } // itBox_Logic1_DmgDealt
                    31 => {  ret = m::itBox_Logic1_Clanked(&raw mut GO[0]) as u32; } // itBox_Logic1_Clanked
                    32 => {  ret = m::itBox_Logic1_HitShield(&raw mut GO[0]) as u32; } // itBox_Logic1_HitShield
                    33 => {  ret = m::itBox_Logic1_Reflected(&raw mut GO[0]) as u32; } // itBox_Logic1_Reflected
                    34 => {  ret = m::itBox_Logic1_DmgReceived(&raw mut GO[0]) as u32; } // itBox_Logic1_DmgReceived
                    35 => {  m::itBox_Logic1_EnteredAir(&raw mut GO[0]); } // itBox_Logic1_EnteredAir
                    36 => {  ret = m::itBox_UnkMotion5_Anim(&raw mut GO[0]) as u32; } // itBox_UnkMotion5_Anim
                    37 => {  m::itBox_UnkMotion5_Phys(&raw mut GO[0]); } // itBox_UnkMotion5_Phys
                    38 => {  ret = m::itBox_UnkMotion5_Coll(&raw mut GO[0]) as u32; } // itBox_UnkMotion5_Coll
                    39 => {  m::itBox_Logic1_EvtUnk(&raw mut GO[0], &raw mut GO[1]); } // itBox_Logic1_EvtUnk
                    40 => {  m::it_802870A4(&raw mut GO[0]); } // it_802870A4
                    41 => {  ret = m::itBox_UnkMotion8_Anim(&raw mut GO[0]) as u32; } // itBox_UnkMotion8_Anim
                    42 => {  m::itBox_UnkMotion8_Phys(&raw mut GO[0]); } // itBox_UnkMotion8_Phys
                    43 => {  ret = m::itBox_UnkMotion8_Coll(&raw mut GO[0]) as u32; } // itBox_UnkMotion8_Coll
                    44 => {  m::it_8028733C(&raw mut GO[0]); } // it_8028733C
                    45 => { ret = pi(m::it_80286088(null_mut())); } // it_80286088(NULL)
                    46 => { GO[0].hsd_obj = null_mut(); ret = pi(m::it_80286088(&raw mut GO[0])); } // it_80286088(no jobj)
                    47 => { for j in 0..9 { let row = &m::it_803F5850[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 19200);
    }
}
