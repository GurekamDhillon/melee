//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/itrshell.c.
//! src/tu/itrshell.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py itrshell`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itrshell as m;
use mtu_harness::*;
#[path = "kinds_gen/itrshell_gen.rs"]
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
            let a = SPECB.0[0].as_mut_ptr() as *mut m::itRShell_Attrs;
            (*a).x0 = rw(); (*a).x4 = rw(); (*a).x8 = rw(); (*a).xC = (rnd() % 4) as f32 / 2.0; (*a).x10 = (rnd() % 5) as f32 / 2.0;
            (*a).x14.x = rw(); (*a).x14.y = rw(); (*a).x14.z = rw(); (*a).x20 = rw(); (*a).x24 = rw(); (*a).x28 = rw(); (*a).x2C = rw();
            (*a).x38 = rw(); (*a).x3C = rw(); (*a).x40 = rw(); (*a).x44 = rw(); (*a).x48.x = rw(); (*a).x48.y = rw(); (*a).x48.z = rw();
            (*a).x54 = (rnd() % 4) as i32;
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
        let rs = &mut IT[k].xDD4_itemVar.rshell;
        IT[k].msid = (rnd() % 10) as i32 - 1;
        rs.xDD4 = ((rnd() % 4) as i32 - 1) as f32;
        rs.xDD8 = ((rnd() % 4) as i32 - 1) as f32;
        rs.xDDC = ((rnd() % 4) as i32 - 1) as f32;
        rs.xDE0 = ((rnd() % 5) as i32 - 2) as f32;
        rs.xDE4 = ((rnd() % 4) as i32 - 1) as f32;
        rs.xDE8 = (rnd() & 1) as i32;
        rs.xDEC = (rnd() % 4) as i32;
        rs.xDF0 = (rnd() & 1) as i32;
        rs.set_xDF4_b0((rnd() & 1) as u8);
        IT[k].xDCD_flag.anon_0.set_b5((rnd() & 1) as u8);
        if rnd() & 1 != 0 { IT[k].x40_vel.x = ((rnd() % 5) as i32 - 2) as f32 / 2.0; }
    }
}

unsafe fn hash_all() {
    for k in 0..3 { let rs = &mut IT[k].xDD4_itemVar.rshell; hm(rs.xDF4_b0() as u32); core::ptr::write_bytes(rs as *mut _ as *mut u8, 0, core::mem::size_of::<itRShell_ItemVars>()); hm(IT[k].xDCD_flag.anon_0.b5() as u32); *(&raw mut IT[k].xDCD_flag as *mut u8) = 0; }
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
fn itrshell_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        g::freg_all();
        const PROFTAB: [u8; 50] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_itrshell.txt"));
        for op in 0..50u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => {  m::it_8028CFE0(&raw mut GO[0]); } // it_8028CFE0
                    1 => {  m::it_8028D090(&raw mut GO[0]); } // it_8028D090
                    2 => {  m::it_8028D100(&raw mut GO[0]); } // it_8028D100
                    3 => {  m::it_8028D26C(&raw mut GO[0]); } // it_8028D26C
                    4 => {  m::it_8028D390(&raw mut GO[0]); } // it_8028D390
                    5 => {  m::it_8028D3B8(&raw mut GO[0]); } // it_8028D3B8
                    6 => {  m::fn_8028D4A8(&raw mut GO[0]); } // fn_8028D4A8
                    7 => {  m::it_8028D4E4(&raw mut GO[0]); } // it_8028D4E4
                    8 => { let v1: f32 = rw() as f32; let v2: f32 = rw() as f32; ret = (m::it_8028D56C(&raw mut GO[0], v1, v2)).to_bits(); } // it_8028D56C
                    9 => {  m::it_3F14_Logic15_Spawned(&raw mut GO[0]); } // it_3F14_Logic15_Spawned
                    10 => {  m::it_8028D62C(&raw mut GO[0]); } // it_8028D62C
                    11 => {  ret = m::itRshell_UnkMotion0_Anim(&raw mut GO[0]) as u32; } // itRshell_UnkMotion0_Anim
                    12 => {  m::itRshell_UnkMotion0_Phys(&raw mut GO[0]); } // itRshell_UnkMotion0_Phys
                    13 => {  ret = m::itRshell_UnkMotion0_Coll(&raw mut GO[0]) as u32; } // itRshell_UnkMotion0_Coll
                    14 => {  m::it_8028D7F0(&raw mut GO[0]); } // it_8028D7F0
                    15 => {  ret = m::itRshell_UnkMotion1_Anim(&raw mut GO[0]) as u32; } // itRshell_UnkMotion1_Anim
                    16 => {  m::itRshell_UnkMotion1_Phys(&raw mut GO[0]); } // itRshell_UnkMotion1_Phys
                    17 => {  ret = m::itRshell_UnkMotion1_Coll(&raw mut GO[0]) as u32; } // itRshell_UnkMotion1_Coll
                    18 => {  m::itRShell_Logic15_PickedUp(&raw mut GO[0]); } // itRShell_Logic15_PickedUp
                    19 => {  ret = m::itRshell_UnkMotion2_Anim(&raw mut GO[0]) as u32; } // itRshell_UnkMotion2_Anim
                    20 => {  m::itRshell_UnkMotion2_Phys(&raw mut GO[0]); } // itRshell_UnkMotion2_Phys
                    21 => {  m::it_3F14_Logic15_Thrown(&raw mut GO[0]); } // it_3F14_Logic15_Thrown
                    22 => {  ret = m::itRshell_UnkMotion3_Anim(&raw mut GO[0]) as u32; } // itRshell_UnkMotion3_Anim
                    23 => {  m::itRshell_UnkMotion3_Phys(&raw mut GO[0]); } // itRshell_UnkMotion3_Phys
                    24 => {  ret = m::itRshell_UnkMotion3_Coll(&raw mut GO[0]) as u32; } // itRshell_UnkMotion3_Coll
                    25 => {  m::it_3F14_Logic15_Dropped(&raw mut GO[0]); } // it_3F14_Logic15_Dropped
                    26 => {  ret = m::itRshell_UnkMotion4_Anim(&raw mut GO[0]) as u32; } // itRshell_UnkMotion4_Anim
                    27 => {  m::itRshell_UnkMotion4_Phys(&raw mut GO[0]); } // itRshell_UnkMotion4_Phys
                    28 => {  ret = m::itRshell_UnkMotion4_Coll(&raw mut GO[0]) as u32; } // itRshell_UnkMotion4_Coll
                    29 => {  m::it_8028DAE4(&raw mut GO[0]); } // it_8028DAE4
                    30 => {  ret = m::itRshell_UnkMotion5_Anim(&raw mut GO[0]) as u32; } // itRshell_UnkMotion5_Anim
                    31 => {  m::itRshell_UnkMotion5_Phys(&raw mut GO[0]); } // itRshell_UnkMotion5_Phys
                    32 => {  ret = m::itRshell_UnkMotion5_Coll(&raw mut GO[0]) as u32; } // itRshell_UnkMotion5_Coll
                    33 => {  m::it_8028E170(&raw mut GO[0]); } // it_8028E170
                    34 => {  ret = m::itRshell_UnkMotion6_Anim(&raw mut GO[0]) as u32; } // itRshell_UnkMotion6_Anim
                    35 => {  m::itRshell_UnkMotion6_Phys(&raw mut GO[0]); } // itRshell_UnkMotion6_Phys
                    36 => {  ret = m::itRshell_UnkMotion6_Coll(&raw mut GO[0]) as u32; } // itRshell_UnkMotion6_Coll
                    37 => {  m::it_3F14_Logic15_EnteredAir(&raw mut GO[0]); } // it_3F14_Logic15_EnteredAir
                    38 => {  ret = m::itRshell_UnkMotion7_Anim(&raw mut GO[0]) as u32; } // itRshell_UnkMotion7_Anim
                    39 => {  m::itRshell_UnkMotion7_Phys(&raw mut GO[0]); } // itRshell_UnkMotion7_Phys
                    40 => {  ret = m::itRshell_UnkMotion7_Coll(&raw mut GO[0]) as u32; } // itRshell_UnkMotion7_Coll
                    41 => {  ret = m::it_3F14_Logic15_DmgDealt(&raw mut GO[0]) as u32; } // it_3F14_Logic15_DmgDealt
                    42 => {  ret = m::it_3F14_Logic15_DmgReceived(&raw mut GO[0]) as u32; } // it_3F14_Logic15_DmgReceived
                    43 => {  ret = m::itRShell_Logic15_Reflected(&raw mut GO[0]) as u32; } // itRShell_Logic15_Reflected
                    44 => {  ret = m::itRShell_Logic15_Clanked(&raw mut GO[0]) as u32; } // itRShell_Logic15_Clanked
                    45 => {  ret = m::it_3F14_Logic15_HitShield(&raw mut GO[0]) as u32; } // it_3F14_Logic15_HitShield
                    46 => {  ret = m::itRShell_Logic15_ShieldBounced(&raw mut GO[0]) as u32; } // itRShell_Logic15_ShieldBounced
                    47 => {  ret = m::it_8028E6C0(&raw mut GO[0]) as u32; } // it_8028E6C0
                    48 => {  m::itRShell_Logic15_EvtUnk(&raw mut GO[0], &raw mut GO[1]); } // itRShell_Logic15_EvtUnk
                    49 => { for j in 0..8 { let row = &m::it_803F5C48[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 20000);
    }
}
