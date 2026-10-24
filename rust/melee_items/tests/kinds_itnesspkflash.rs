//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/itnesspkflash.c.
//! src/tu/itnesspkflash.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py itnesspkflash`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itnesspkflash as m;
use mtu_harness::*;
#[path = "kinds_gen/itnesspkflash_gen.rs"]
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
            for j in 0..3 {
                let a = SPECB.0[j].as_mut_ptr() as *mut itFlashAttributes;
                let f = a as *mut f32; for q in 0..11 { *f.add(q) = rw(); } (*a).x28_FLASH_EXPLOSION_DELAY = (rnd() % 8) as f32; (*a).x4_FLASH_HITBOX_SIZE_MUL = (rnd() % 6) as f32;
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
        let i = rnd() % 4;
        IT[k].xDD4_itemVar.pkflush.xDE0_PKFlash_Owner = if i == 0 { null_mut() } else if i == 1 { IT[k].owner } else { &raw mut GO[(k + i as usize) % 3] };
        IT[k].xDD4_itemVar.pkflush.xDD8_PKFlash = rw();
        IT[k].xDD4_itemVar.pkflush.xDDC_PKFlash = (rnd() & 1) as i32;
        let i = rnd() % 3;
        IT[k].kind = if i == 0 { It_Kind_Ness_PKFlush } else if i == 1 { It_Kind_Kirby_NessPKFlush } else { It_Kind_Ness_Bat };
        IT[k].msid = (rnd() % 4) as i32;
        IT[k].xDAC_itcmd_var0 = rnd() % 8;
        IT[k].x40_vel.x = rg(); IT[k].x40_vel.y = rg();
    }
}

unsafe fn hash_all() {
    for k in 0..3 { hm(pi(IT[k].xDD4_itemVar.pkflush.xDE0_PKFlash_Owner)); IT[k].xDD4_itemVar.pkflush.xDE0_PKFlash_Owner = null_mut(); }
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
fn itnesspkflash_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        g::freg_all();
        const PROFTAB: [u8; 21] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_itnesspkflash.txt"));
        for op in 0..21u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => {  ret = pi(m::it_802AA7E4(&raw mut GO[0])); } // it_802AA7E4
                    1 => {  ret = m::it_802AA7F0(&raw mut GO[0]) as u32; } // it_802AA7F0
                    2 => { ret = m::it_802AA810(&raw mut GO[0], &raw mut IT[0].x378_itemColl) as u32; } // it_802AA810
                    3 => { let v2: ItemKind = (rnd() % 150) as i32 as ItemKind; let v3: f32 = rw() as f32; ret = pi(m::it_802AA8C0(&raw mut GO[0], &raw mut VA[0], v2, v3)); } // it_802AA8C0
                    4 => {  m::it_2725_Logic102_Destroyed(&raw mut GO[0]); } // it_2725_Logic102_Destroyed
                    5 => {  m::it_802AAA50(&raw mut GO[0]); } // it_802AAA50
                    6 => {  m::it_802AAA80(&raw mut GO[0]); } // it_802AAA80
                    7 => {  ret = m::itNesspkflash_UnkMotion0_Anim(&raw mut GO[0]) as u32; } // itNesspkflash_UnkMotion0_Anim
                    8 => {  ret = m::itNesspkflash_UnkMotion1_Anim(&raw mut GO[0]) as u32; } // itNesspkflash_UnkMotion1_Anim
                    9 => {  ret = m::itNesspkflash_UnkMotion2_Anim(&raw mut GO[0]) as u32; } // itNesspkflash_UnkMotion2_Anim
                    10 => {  m::itNesspkflash_UnkMotion0_Phys(&raw mut GO[0]); } // itNesspkflash_UnkMotion0_Phys
                    11 => {  m::itNesspkflash_UnkMotion1_Phys(&raw mut GO[0]); } // itNesspkflash_UnkMotion1_Phys
                    12 => {  m::itNesspkflash_UnkMotion2_Phys(&raw mut GO[0]); } // itNesspkflash_UnkMotion2_Phys
                    13 => {  ret = m::itNesspkflash_UnkMotion0_Coll(&raw mut GO[0]) as u32; } // itNesspkflash_UnkMotion0_Coll
                    14 => {  ret = m::itNesspkflash_UnkMotion1_Coll(&raw mut GO[0]) as u32; } // itNesspkflash_UnkMotion1_Coll
                    15 => {  ret = m::itNesspkflash_UnkMotion2_Coll(&raw mut GO[0]) as u32; } // itNesspkflash_UnkMotion2_Coll
                    16 => {  ret = m::itNesspkflash_Logic102_Reflected(&raw mut GO[0]) as u32; } // itNesspkflash_Logic102_Reflected
                    17 => {  ret = m::itNesspkflash_Logic102_Clanked(&raw mut GO[0]) as u32; } // itNesspkflash_Logic102_Clanked
                    18 => {  ret = m::itNesspkflash_Logic102_Absorbed(&raw mut GO[0]) as u32; } // itNesspkflash_Logic102_Absorbed
                    19 => {  m::itNesspkflash_Logic102_EvtUnk(&raw mut GO[0], &raw mut GO[1]); } // itNesspkflash_Logic102_EvtUnk
                    20 => { for j in 0..3 { let row = &m::it_803F6B70[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 8400);
    }
}
