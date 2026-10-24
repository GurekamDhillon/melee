//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/itseakchain.c.
//! src/tu/itseakchain.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py itseakchain`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itseakchain as m;
use mtu_harness::*;
#[path = "kinds_gen/itseakchain_gen.rs"]
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

static mut LK: [ItemLink; 16] = unsafe { core::mem::zeroed() };
static mut LGO: [HSD_GObj; 16] = unsafe { core::mem::zeroed() };
static mut LJO: [HSD_JObj; 32] = unsafe { core::mem::zeroed() };
static mut FGT: Fighter = unsafe { core::mem::zeroed() };
static mut FBN: [FighterBone; 32] = unsafe { core::mem::zeroed() };
static mut LKC: usize = 0;
static mut LGC: usize = 0;
static mut NL: u32 = 0;


unsafe fn spec_fill(prof: i32) {
    match prof {
        0 => {
            let a = SPECB.0[0].as_mut_ptr() as *mut itSeakChain_Attrs;
            (*a).x0 = 3 + (rnd() % 12) as i32; (*a).x4 = (rnd() % 8) as f32 + if rnd() & 1 != 0 { 0.5 } else { 0.0 };
            (*a).x10 = rw(); (*a).x14 = rw(); (*a).x18 = rw(); (*a).x1C = rw(); (*a).x20 = rw(); (*a).x24 = rw(); (*a).x28 = rw(); (*a).x2C = rw();
            (*a).x30 = rw(); (*a).x34 = rw(); (*a).x38 = rw(); (*a).x3C = rw(); (*a).x40 = rw(); (*a).x44 = rw(); (*a).x48 = rw();
            (*a).x54 = rw(); (*a).x58 = rw(); (*a).x5C = rw(); (*a).x60 = rw();
            (*a).x64_joint = if rnd() & 1 != 0 { &raw mut JO[0] as *mut HSD_Joint } else { null_mut() };
            (*a).x68_joint = if rnd() & 1 != 0 { &raw mut JO[1] as *mut HSD_Joint } else { null_mut() };
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
    NL = 3 + rnd() % 5;
    LKC = 0; LGC = 0;
    for i in 0..16 { g::fill_ItemLink(&mut LK[i]); g::fill_HSD_GObj(&mut LGO[i]); }
    for i in 0..32 { g::fill_HSD_JObj(&mut LJO[i]); LJO[i].flags &= !(JOBJ_USE_QUATERNION as u32); }
    FGT = core::mem::zeroed(); FBN = core::mem::zeroed();
    FGT.u.sk.lstick_delta.x = rw(); FGT.u.sk.lstick_delta.y = rw(); FGT.u.sk.lstick_delta.z = rw();
    FGT.mv.co.common.x1C = if rnd() & 1 != 0 { 0.0 } else { rw() };
    FGT.parts = &raw mut FBN as *mut FighterBone; FBN[26].joint = &raw mut LJO[16];
    GO[2].user_data = &raw mut FGT as *mut _;
    for i in 0..16usize {
        LK[i].prev = if i > 0 && (i as u32) < NL { &raw mut LK[i - 1] } else { null_mut() };
        LK[i].next = if (i as u32) + 1 < NL { &raw mut LK[i + 1] } else { null_mut() };
        LK[i].gobj = &raw mut LGO[i]; LGO[i].hsd_obj = &raw mut LJO[i] as *mut _; LK[i].jobj = &raw mut LJO[16 + i];
        LK[i].set_x2C_b0(if i == 0 { 1 } else { (rnd() & 1) as u8 });
        LK[i].set_x2C_b1((rnd() & 1) as u8); LK[i].set_x2C_b2((rnd() & 1) as u8);
    }
    for k in 0..3 {
        let sc = &mut IT[k].xDD4_itemVar.seakchain;
        sc.x0 = &raw mut LK[0]; sc.x4 = &raw mut LK[(NL - 1) as usize]; sc.parent_gobj = &raw mut GO[2];
        sc.x10 = (rnd() & 1) as i32; sc.x14 = (rnd() & 1) as i32; sc.x18 = (rnd() & 1) as i32;
        for j in 0..15 { sc.history[j].x = rw(); sc.history[j].y = rw(); sc.history[j].z = rw(); }
    }
}

unsafe fn hash_all() {
    for k in 0..3 {
        let sc = &mut IT[k].xDD4_itemVar.seakchain;
        hm(pi(sc.x0)); hm(pi(sc.x4)); hm(pi(sc.parent_gobj));
        sc.x0 = null_mut(); sc.x4 = null_mut(); sc.parent_gobj = null_mut();
    }
    for i in 0..16 { g::hash_ItemLink(&LK[i]); g::hash_HSD_GObj(&LGO[i]); }
    for i in 0..32 { g::hash_HSD_JObj(&LJO[i]); }
    hm(FGT.u.sk.lstick_delta.x.to_bits()); hm(FGT.u.sk.lstick_delta.y.to_bits()); hm(FGT.u.sk.lstick_delta.z.to_bits()); hm(FGT.mv.co.common.x1C.to_bits());
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
fn itseakchain_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        preg(&raw const LK, core::mem::size_of_val(&LK)); preg(&raw const LGO, core::mem::size_of_val(&LGO)); preg(&raw const LJO, core::mem::size_of_val(&LJO)); preg(&raw const FGT, core::mem::size_of_val(&FGT)); preg(&raw const FBN, core::mem::size_of_val(&FBN));
        g::freg_all();
        const PROFTAB: [u8; 30] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_itseakchain.txt"));
        for op in 0..30u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => {  m::it_802BAEEC(&raw mut GO[0]); } // it_802BAEEC
                    1 => {  m::it_802BAF0C(&raw mut GO[0]); } // it_802BAF0C
                    2 => {  m::it_802BAF2C(&raw mut IT[0], &raw mut JO[3]); } // it_802BAF2C
                    3 => {  m::inlineA0(&raw mut GO[0]); } // inlineA0
                    4 => {  m::it_802BB20C(&raw mut GO[0]); } // it_802BB20C
                    5 => { { let f = rw(); ret = pi(m::itSeakChain_Spawn(&raw mut GO[2], &raw mut VA[0], f)); } } // itSeakChain_Spawn
                    6 => {  m::fn_802BB428(&raw mut GO[0]); } // fn_802BB428
                    7 => {  m::fn_802BB44C(&raw mut GO[0]); } // fn_802BB44C
                    8 => {  m::fn_802BB574(&raw mut GO[0]); } // fn_802BB574
                    9 => {  m::fn_802BB694(&raw mut GO[0]); } // fn_802BB694
                    10 => {  m::fn_802BB784(&raw mut GO[0]); } // fn_802BB784
                    11 => {  ret = m::notInSpecialS(&raw mut GO[0]) as u32; } // notInSpecialS
                    12 => {  ret = m::itSeakchain_UnkMotion4_Anim(&raw mut GO[0]) as u32; } // itSeakchain_UnkMotion4_Anim
                    13 => { let v1: i32 = fv() as i32 as i32; let v2: f32 = rw() as f32; ret = m::it_802BB938(&raw mut LK[(rnd() % NL) as usize], v1, v2) as u32; } // it_802BB938
                    14 => { let v1: i32 = fv() as i32 as i32; let v2: f32 = rw() as f32; ret = m::it_802BBAEC(&raw mut LK[(rnd() % NL) as usize], v1, v2) as u32; } // it_802BBAEC
                    15 => { let v3: f32 = rw() as f32; m::it_802BBB0C(&raw mut LK[(rnd() % NL) as usize], &raw mut VA[0], SPECB.0[0].as_mut_ptr() as *mut itSeakChain_Attrs, v3); } // it_802BBB0C
                    16 => { let v3: f32 = rw() as f32; m::it_802BBC38(&raw mut LK[(rnd() % NL) as usize], &raw mut VA[0], SPECB.0[0].as_mut_ptr() as *mut itSeakChain_Attrs, v3); } // it_802BBC38
                    17 => {  ret = m::it_802BBD64(&raw mut LK[(rnd() % NL) as usize], &raw mut VA[0], SPECB.0[0].as_mut_ptr() as *mut itSeakChain_Attrs) as u32; } // it_802BBD64
                    18 => {  ret = m::it_802BBED0(&raw mut LK[(rnd() % NL) as usize], &raw mut VA[0], SPECB.0[0].as_mut_ptr() as *mut itSeakChain_Attrs) as u32; } // it_802BBED0
                    19 => {  m::it_802BC080(&raw mut LK[(rnd() % NL) as usize], &raw mut VA[0], &raw mut IT[0]); } // it_802BC080
                    20 => { let v3: f32 = rw() as f32; ret = m::it_802BC94C(&raw mut LK[(rnd() % NL) as usize], &raw mut VA[0], SPECB.0[0].as_mut_ptr() as *mut itSeakChain_Attrs, v3) as u32; } // it_802BC94C
                    21 => {  m::it_802BCA30(&raw mut IT[0]); } // it_802BCA30
                    22 => {  m::it_802BCB88(&raw mut IT[0], &raw mut VA[0]); } // it_802BCB88
                    23 => {  m::it_2725_Logic54_PickedUp(&raw mut GO[0]); } // it_2725_Logic54_PickedUp
                    24 => {  m::it_802BCED4(&raw mut GO[0]); } // it_802BCED4
                    25 => {  m::it_802BCF2C(&raw mut GO[0]); } // it_802BCF2C
                    26 => {  m::it_802BCF84(&raw mut GO[0]); } // it_802BCF84
                    27 => {  m::it_802BCFC4(&raw mut GO[0], &raw mut VA[0]); } // it_802BCFC4
                    28 => { m::itSeakChain_Logic54_EvtUnk(&raw mut GO[0], if rnd() & 1 != 0 { &raw mut GO[2] } else { &raw mut GO[1] }); } // itSeakChain_Logic54_EvtUnk
                    29 => { for j in 0..5 { let row = &m::it_803F7438[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 12000);
    }
}
