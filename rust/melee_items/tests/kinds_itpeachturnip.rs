//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/itpeachturnip.c.
//! src/tu/itpeachturnip.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py itpeachturnip`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itpeachturnip as m;
use mtu_harness::*;
#[path = "kinds_gen/itpeachturnip_gen.rs"]
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

const KK: [i32; 6] = [99, 6, 7, 12, 5, 99];

unsafe fn spec_fill(prof: i32) {
    match prof {
        0 => {
            for j in 0..3 {
                let w = SPECB.0[j].as_mut_ptr() as *mut i32;
                *(w as *mut f32) = rw();
                *w.add(1) = (rnd() % 10) as i32 - 1;
                for m in 0..9usize {
                    *w.add(2 + 2 * m) = (rnd() % 6) as i32;
                    *w.add(3 + 2 * m) = (rnd() % 20) as i32;
                }
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
        IT[k].kind = KK[(rnd() % 6) as usize];
        let pt = &mut IT[k].xDD4_itemVar.peachturnip;
        pt.xDD8 = (rnd() % 10) as i32 - 1;
        pt.xDD4.anon_0.set_b0((rnd() & 1) as u8);
        pt.xDDC_damage = (rnd() % 30) as i32;
        pt.xDE0_scl = rw();
        pt.xDE4_owner = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        pt.xDE8_scl = rw();
        IT[k].xD44_lifeTimer = if rnd() & 1 != 0 { 1.0 } else { rw() };
    }
}

unsafe fn hash_all() {
    for k in 0..3 {
        let pt = &mut IT[k].xDD4_itemVar.peachturnip;
        hm(pi(pt.xDE4_owner)); pt.xDE4_owner = null_mut();
        hm(pt.xDD4.anon_0.b0() as u32);
        core::ptr::write_bytes(&raw mut pt.xDD4 as *mut u8, 0, core::mem::size_of_val(&pt.xDD4));
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
fn itpeachturnip_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        g::freg_all();
        const PROFTAB: [u8; 22] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_itpeachturnip.txt"));
        for op in 0..22u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => {  ret = m::decrease_lifetimer(&raw mut GO[0]) as u32; } // decrease_lifetimer
                    1 => {  ret = m::it_802BD32C(&raw mut GO[0]) as u32; } // it_802BD32C
                    2 => {  m::it_802BD45C(&raw mut GO[0]); } // it_802BD45C
                    3 => {  m::itPeachTurnip_Logic56_Destroyed(&raw mut GO[0]); } // itPeachTurnip_Logic56_Destroyed
                    4 => { let v2: Fighter_Part = fv() as i32 as Fighter_Part; let v3: ItemKind = KK[(rnd() % 6) as usize] as ItemKind; let v4: f32 = rw() as f32; ret = pi(m::it_802BD4AC(&raw mut GO[0], &raw mut VA[0], v2, v3, v4)); } // it_802BD4AC
                    5 => {  m::itPeachTurnip_Logic56_PickedUp(&raw mut GO[0]); } // itPeachTurnip_Logic56_PickedUp
                    6 => {  ret = m::itPeachturnip_UnkMotion4_Anim(&raw mut GO[0]) as u32; } // itPeachturnip_UnkMotion4_Anim
                    7 => {  m::itPeachturnip_UnkMotion4_Phys(&raw mut GO[0]); } // itPeachturnip_UnkMotion4_Phys
                    8 => {  ret = m::itPeachturnip_UnkMotion1_Anim(&raw mut GO[0]) as u32; } // itPeachturnip_UnkMotion1_Anim
                    9 => {  m::itPeachturnip_UnkMotion1_Phys(&raw mut GO[0]); } // itPeachturnip_UnkMotion1_Phys
                    10 => {  m::itPeachTurnip_Logic56_Thrown(&raw mut GO[0]); } // itPeachTurnip_Logic56_Thrown
                    11 => {  ret = m::itPeachturnip_UnkMotion3_Anim(&raw mut GO[0]) as u32; } // itPeachturnip_UnkMotion3_Anim
                    12 => {  m::itPeachturnip_UnkMotion3_Phys(&raw mut GO[0]); } // itPeachturnip_UnkMotion3_Phys
                    13 => {  ret = m::itPeachturnip_UnkMotion3_Coll(&raw mut GO[0]) as u32; } // itPeachturnip_UnkMotion3_Coll
                    14 => {  m::itPeachTurnip_Logic56_Dropped(&raw mut GO[0]); } // itPeachTurnip_Logic56_Dropped
                    15 => {  ret = m::itPeachTurnip_Logic56_DmgDealt(&raw mut GO[0]) as u32; } // itPeachTurnip_Logic56_DmgDealt
                    16 => {  ret = m::itPeachTurnip_Logic56_Clanked(&raw mut GO[0]) as u32; } // itPeachTurnip_Logic56_Clanked
                    17 => {  ret = m::itPeachTurnip_Logic56_Reflected(&raw mut GO[0]) as u32; } // itPeachTurnip_Logic56_Reflected
                    18 => {  ret = m::itPeachTurnip_Logic56_HitShield(&raw mut GO[0]) as u32; } // itPeachTurnip_Logic56_HitShield
                    19 => {  ret = m::itPeachTurnip_Logic56_ShieldBounced(&raw mut GO[0]) as u32; } // itPeachTurnip_Logic56_ShieldBounced
                    20 => {  m::itPeachTurnip_Logic56_EvtUnk(&raw mut GO[0], &raw mut GO[1]); } // itPeachTurnip_Logic56_EvtUnk
                    21 => { for j in 0..5 { let row = &m::it_803F74A8[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 8800);
    }
}
