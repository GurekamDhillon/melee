//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/itwstar.c.
//! src/tu/itwstar.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py itwstar`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itwstar as m;
use mtu_harness::*;
#[path = "kinds_gen/itwstar_gen.rs"]
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

static mut BOXV: ftCollisionBox = unsafe { core::mem::zeroed() };

unsafe fn spec_fill(prof: i32) {
    match prof {
        0 => {
            let a = SPECB.0[0].as_mut_ptr() as *mut itWstarAttributes;
            let en = (&raw mut (*a).x28_entries) as *mut itWstarAttrEntry;
            (*a).x0 = rw(); (*a).x4 = rw(); (*a).x8 = rw(); (*a).xC = rw(); (*a).x10 = rw(); (*a).x14 = rw(); (*a).x18 = rw();
            (*a).x1C = rw(); (*a).x20 = rw();
            (*a).x24_count = 1 + (rnd() % 4) as i32;
            for i in 0..4 { (*en.add(i)).x0_anim_joint = &raw mut JO[3 + i] as *mut HSD_AnimJoint; (*en.add(i)).x4_sfx = (rnd() % 500) as _; }
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
    BOXV = core::mem::zeroed();
    m::ext::it_804D6D00 = ((rnd() % 6) as i32 - 1) as i8;
}

unsafe fn hash_all() {
    hm(m::ext::it_804D6D00 as i32 as u32);
    hm(BOXV.top.to_bits()); hm(BOXV.bottom.to_bits()); hm(BOXV.left.x.to_bits()); hm(BOXV.left.y.to_bits()); hm(BOXV.right.x.to_bits()); hm(BOXV.right.y.to_bits());
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
fn itwstar_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        g::freg_all();
        const PROFTAB: [u8; 26] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_itwstar.txt"));
        for op in 0..26u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => {  ret = pi(m::it_80294364(&raw mut GO[0])); } // it_80294364
                    1 => { let v1: f32 = rw() as f32; let v2: f32 = rw() as f32; m::it_80294430(&raw mut GO[0], v1, v2); } // it_80294430
                    2 => { m::it_802944AC(&raw mut GO[0], &raw mut BOXV); } // it_802944AC
                    3 => {  m::itWStar_Logic29_Spawned(&raw mut GO[0]); } // itWStar_Logic29_Spawned
                    4 => {  m::it_8029455C(&raw mut GO[0]); } // it_8029455C
                    5 => {  ret = m::itWstar_UnkMotion0_Anim(&raw mut GO[0]) as u32; } // itWstar_UnkMotion0_Anim
                    6 => {  m::itWstar_UnkMotion0_Phys(&raw mut GO[0]); } // itWstar_UnkMotion0_Phys
                    7 => {  ret = m::itWstar_UnkMotion0_Coll(&raw mut GO[0]) as u32; } // itWstar_UnkMotion0_Coll
                    8 => {  m::it_80294624(&raw mut GO[0]); } // it_80294624
                    9 => {  ret = m::itWstar_UnkMotion1_Anim(&raw mut GO[0]) as u32; } // itWstar_UnkMotion1_Anim
                    10 => {  m::itWstar_UnkMotion1_Phys(&raw mut GO[0]); } // itWstar_UnkMotion1_Phys
                    11 => {  ret = m::itWstar_UnkMotion1_Coll(&raw mut GO[0]) as u32; } // itWstar_UnkMotion1_Coll
                    12 => {  m::it_802946B0(&raw mut GO[0]); } // it_802946B0
                    13 => {  m::it_3F14_Logic29_PickedUp(&raw mut GO[0]); } // it_3F14_Logic29_PickedUp
                    14 => {  ret = m::itWstar_UnkMotion3_Anim(&raw mut GO[0]) as u32; } // itWstar_UnkMotion3_Anim
                    15 => {  m::itWStar_Logic29_Dropped(&raw mut GO[0]); } // itWStar_Logic29_Dropped
                    16 => {  m::it_802947CC(&raw mut GO[0], &raw mut VA[0]); } // it_802947CC
                    17 => {  ret = m::itWstar_UnkMotion5_Anim(&raw mut GO[0]) as u32; } // itWstar_UnkMotion5_Anim
                    18 => {  m::itWstar_UnkMotion5_Phys(&raw mut GO[0]); } // itWstar_UnkMotion5_Phys
                    19 => {  ret = m::itWstar_UnkMotion5_Coll(&raw mut GO[0]) as u32; } // itWstar_UnkMotion5_Coll
                    20 => {  m::itWStar_Logic29_EnteredAir(&raw mut GO[0]); } // itWStar_Logic29_EnteredAir
                    21 => {  ret = m::itWstar_UnkMotion4_Anim(&raw mut GO[0]) as u32; } // itWstar_UnkMotion4_Anim
                    22 => {  m::itWstar_UnkMotion4_Phys(&raw mut GO[0]); } // itWstar_UnkMotion4_Phys
                    23 => {  ret = m::itWstar_UnkMotion4_Coll(&raw mut GO[0]) as u32; } // itWstar_UnkMotion4_Coll
                    24 => {  m::itWStar_Logic30_EvtUnk(&raw mut GO[0], &raw mut GO[1]); } // itWStar_Logic30_EvtUnk
                    25 => { for j in 0..6 { let row = &m::it_803F61B0[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 10400);
    }
}
