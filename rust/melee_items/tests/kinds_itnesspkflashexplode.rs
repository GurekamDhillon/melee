//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/itnesspkflashexplode.c.
//! src/tu/itnesspkflashexplode.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py itnesspkflashexplode`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itnesspkflashexplode as m;
use mtu_harness::*;
#[path = "kinds_gen/itnesspkflashexplode_gen.rs"]
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
                let a = SPECB.0[j].as_mut_ptr() as *mut itFlashExplAttributes;
                (*a).x0_FLASH_EXPL_HITBOX_SIZE_MUL = (1 + rnd() % 5) as f32; (*a).x4_FLASH_EXPL_GRAPHIC_SIZE_INIT = rw(); (*a).x8_FLASH_EXPL_GRAPHIC_SIZE_GROWTH = rw(); (*a).xC_FLASH_EXPL_BASE_DAMAGE = (rnd() % 30) as f32; (*a).x10_FLASH_EXPL_DAMAGE_MUL = rw();
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
        let fe = &mut IT[k].xDD4_itemVar.pkflushexplode;
        fe.xDD4 = (rnd() % 6) as f32;
        fe.xDD8 = if rnd() & 1 != 0 { 0.0 } else { rw() };
        fe.xDDC = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        IT[k].xDB4_itcmd_var2 = rnd() & 1;
    }
}

unsafe fn hash_all() {
    for k in 0..3 { hm(pi(IT[k].xDD4_itemVar.pkflushexplode.xDDC)); IT[k].xDD4_itemVar.pkflushexplode.xDDC = null_mut(); }
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
fn itnesspkflashexplode_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        g::freg_all();
        const PROFTAB: [u8; 12] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_itnesspkflashexplode.txt"));
        for op in 0..12u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => { let v3: ItemKind = (rnd() % 150) as i32 as ItemKind; let v4: f32 = rw() as f32; let v5: f32 = rw() as f32; ret = pi(m::it_802AF940(&raw mut GO[0], &raw mut GO[1], &raw mut VA[0], v3, v4, v5)); } // it_802AF940
                    1 => {  m::it_2725_Logic103_Destroyed(&raw mut GO[0]); } // it_2725_Logic103_Destroyed
                    2 => {  m::it_802AFA70(&raw mut GO[0]); } // it_802AFA70
                    3 => {  ret = m::itNessPKFlashExplode_UnkMotion0_Anim(&raw mut GO[0]) as u32; } // itNessPKFlashExplode_UnkMotion0_Anim
                    4 => {  m::itNessPKFlashExplode_UnkMotion0_Phys(&raw mut GO[0]); } // itNessPKFlashExplode_UnkMotion0_Phys
                    5 => {  ret = m::itNessPKFlashExplode_UnkMotion0_Coll(&raw mut GO[0]) as u32; } // itNessPKFlashExplode_UnkMotion0_Coll
                    6 => {  ret = m::itNessPKFlashExplode_Logic103_Clanked(&raw mut GO[0]) as u32; } // itNessPKFlashExplode_Logic103_Clanked
                    7 => {  ret = m::itNessPKFlashExplode_Logic103_Absorbed(&raw mut GO[0]) as u32; } // itNessPKFlashExplode_Logic103_Absorbed
                    8 => {  ret = m::itNessPKFlashExplode_Logic103_ShieldBounced(&raw mut GO[0]) as u32; } // itNessPKFlashExplode_Logic103_ShieldBounced
                    9 => {  ret = m::itNessPKFlashExplode_Logic103_HitShield(&raw mut GO[0]) as u32; } // itNessPKFlashExplode_Logic103_HitShield
                    10 => {  m::itNessPKFlashExplode_Logic103_EvtUnk(&raw mut GO[0], &raw mut GO[1]); } // itNessPKFlashExplode_Logic103_EvtUnk
                    11 => { for j in 0..1 { let row = &m::it_803F6F40[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 4800);
    }
}
