//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/itsamusbomb.c.
//! src/tu/itsamusbomb.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py itsamusbomb`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itsamusbomb as m;
use mtu_harness::*;
#[path = "kinds_gen/itsamusbomb_gen.rs"]
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
            let a = SPECB.0[0].as_mut_ptr() as *mut itSamusBombAttributes;
            (*a).x0 = rw(); (*a).x4 = (rnd() % 30) as f32; (*a).x8 = rw(); (*a).xC = (rnd() % 5) as f32 / 2.0; (*a).x10 = rw(); (*a).x14 = rw(); (*a).x18 = rw();
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
        let sb = &mut IT[k].xDD4_itemVar.samusbomb;
        IT[k].xD44_lifeTimer = ((rnd() % 30) as i32 - 2) as f32;
        sb.x0 = (rnd() & 1) as i32;
        sb.owner = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        if rnd() & 1 != 0 { IT[k].x7C.x = ((rnd() % 7) as i32 - 3) as f32 / 2.0; IT[k].x7C.y = ((rnd() % 7) as i32 - 3) as f32 / 2.0; }
        if rnd() & 1 != 0 { IT[k].x40_vel.x = 0.0; }
        if k == 0 && rnd() & 1 != 0 {
            let m = (rnd() % 4 + 1) as f32 / 2.0;
            IT[k].x7C.x = 3.0 * m; IT[k].x7C.y = 4.0 * m;
            let d = (rnd() % 3) as i32 - 1;
            (*(SPECB.0[0].as_mut_ptr() as *mut itSamusBombAttributes)).xC = f32::from_bits((5.0 * m).to_bits().wrapping_add(d as u32));
        }
    }
}

unsafe fn hash_all() {
    for k in 0..3 { let sb = &mut IT[k].xDD4_itemVar.samusbomb; hm(pi(sb.owner)); sb.owner = null_mut(); }
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
fn itsamusbomb_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        g::freg_all();
        const PROFTAB: [u8; 24] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_itsamusbomb.txt"));
        for op in 0..24u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => { let v2: f32 = rw() as f32; ret = pi(m::it_802B4AC8(&raw mut GO[0], &raw mut VA[0], v2)); } // it_802B4AC8
                    1 => {  m::it_802B4BA0(&raw mut GO[0]); } // it_802B4BA0
                    2 => {  m::it_802B4C10(&raw mut GO[0]); } // it_802B4C10
                    3 => {  ret = m::itSamusbomb_UnkMotion0_Anim(&raw mut GO[0]) as u32; } // itSamusbomb_UnkMotion0_Anim
                    4 => {  m::itSamusbomb_UnkMotion0_Phys(&raw mut GO[0]); } // itSamusbomb_UnkMotion0_Phys
                    5 => {  ret = m::itSamusbomb_UnkMotion0_Coll(&raw mut GO[0]) as u32; } // itSamusbomb_UnkMotion0_Coll
                    6 => {  m::it_802B4CF4(&raw mut GO[0]); } // it_802B4CF4
                    7 => {  ret = m::itSamusbomb_UnkMotion1_Anim(&raw mut GO[0]) as u32; } // itSamusbomb_UnkMotion1_Anim
                    8 => {  m::itSamusbomb_UnkMotion1_Phys(&raw mut GO[0]); } // itSamusbomb_UnkMotion1_Phys
                    9 => {  ret = m::itSamusbomb_UnkMotion1_Coll(&raw mut GO[0]) as u32; } // itSamusbomb_UnkMotion1_Coll
                    10 => {  m::itSamusBomb_Logic50_EnteredAir(&raw mut GO[0]); } // itSamusBomb_Logic50_EnteredAir
                    11 => {  ret = m::itSamusbomb_UnkMotion2_Anim(&raw mut GO[0]) as u32; } // itSamusbomb_UnkMotion2_Anim
                    12 => {  m::itSamusbomb_UnkMotion2_Phys(&raw mut GO[0]); } // itSamusbomb_UnkMotion2_Phys
                    13 => {  ret = m::itSamusbomb_UnkMotion2_Coll(&raw mut GO[0]) as u32; } // itSamusbomb_UnkMotion2_Coll
                    14 => {  ret = m::itSamusBomb_Logic50_DmgDealt(&raw mut GO[0]) as u32; } // itSamusBomb_Logic50_DmgDealt
                    15 => {  ret = m::itSamusBomb_Logic50_Clanked(&raw mut GO[0]) as u32; } // itSamusBomb_Logic50_Clanked
                    16 => {  ret = m::itSamusBomb_Logic50_HitShield(&raw mut GO[0]) as u32; } // itSamusBomb_Logic50_HitShield
                    17 => {  ret = m::itSamusBomb_Logic50_ShieldBounced(&raw mut GO[0]) as u32; } // itSamusBomb_Logic50_ShieldBounced
                    18 => {  ret = m::it_2725_Logic50_Reflected(&raw mut GO[0]) as u32; } // it_2725_Logic50_Reflected
                    19 => {  m::it_802B53CC(&raw mut GO[0]); } // it_802B53CC
                    20 => {  ret = m::itSamusbomb_UnkMotion3_Anim(&raw mut GO[0]) as u32; } // itSamusbomb_UnkMotion3_Anim
                    21 => {  m::it_802B5478(&raw mut GO[0]); } // it_802B5478
                    22 => {  m::itSamusBomb_Logic50_EvtUnk(&raw mut GO[0], &raw mut GO[1]); } // itSamusBomb_Logic50_EvtUnk
                    23 => { for j in 0..4 { let row = &m::it_803F7220[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 9600);
    }
}
