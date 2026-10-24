//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/itnokonoko.c.
//! src/tu/itnokonoko.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py itnokonoko`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itnokonoko as m;
use mtu_harness::*;
#[path = "kinds_gen/itnokonoko_gen.rs"]
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
            let a = SPECB.0[0].as_mut_ptr() as *mut itNokoNoko_DatAttrs;
            (*a).x0 = SPECB.0[0].as_mut_ptr().add(64) as *mut _; *(SPECB.0[0].as_mut_ptr().add(64) as *mut i32) = (rnd() % 30) as i32; *(SPECB.0[0].as_mut_ptr().add(68) as *mut f32) = rw();
            (*a).x4 = rw(); (*a).x8 = rw();
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
        let nk = &mut IT[k].xDD4_itemVar.nokonoko;
        nk.x20 = (rnd() % 4) as i32 - 1;
        nk.x28 = (rnd() % 4) as i32 - 1;
        nk.x34 = rw();
        nk.x38 = if rnd() & 1 != 0 { 1.0 } else { -1.0 };
        nk.x3C = (rnd() % 8) as i32;
        nk.x40 = (rnd() % 4) as i32;
        nk.x4 = (rnd() % 4) as i32;
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
fn itnokonoko_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        g::freg_all();
        const PROFTAB: [u8; 48] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_itnokonoko.txt"));
        for op in 0..48u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => {  m::it_802DC69C(&raw mut GO[0]); } // it_802DC69C
                    1 => {  ret = m::itNokonoko_Logic3_DmgReceived(&raw mut GO[0]) as u32; } // itNokonoko_Logic3_DmgReceived
                    2 => {  ret = m::it_802DC83C(&raw mut GO[0]) as u32; } // it_802DC83C
                    3 => {  m::fn_802DC8B8(&raw mut GO[0]); } // fn_802DC8B8
                    4 => {  ret = m::itNokonoko_UnkMotion1_Anim(&raw mut GO[0]) as u32; } // itNokonoko_UnkMotion1_Anim
                    5 => {  m::itNokonoko_UnkMotion1_Phys(&raw mut GO[0]); } // itNokonoko_UnkMotion1_Phys
                    6 => {  ret = m::itNokonoko_UnkMotion1_Coll(&raw mut GO[0]) as u32; } // itNokonoko_UnkMotion1_Coll
                    7 => {  m::it_802DC990(&raw mut GO[0]); } // it_802DC990
                    8 => {  ret = m::itNokonoko_UnkMotion2_Anim(&raw mut GO[0]) as u32; } // itNokonoko_UnkMotion2_Anim
                    9 => {  m::itNokonoko_UnkMotion2_Phys(&raw mut GO[0]); } // itNokonoko_UnkMotion2_Phys
                    10 => {  ret = m::itNokonoko_UnkMotion2_Coll(&raw mut GO[0]) as u32; } // itNokonoko_UnkMotion2_Coll
                    11 => {  m::it_802DCB9C(&raw mut GO[0]); } // it_802DCB9C
                    12 => {  ret = m::itNokonoko_UnkMotion4_Anim(&raw mut GO[0]) as u32; } // itNokonoko_UnkMotion4_Anim
                    13 => {  m::itNokonoko_UnkMotion4_Phys(&raw mut GO[0]); } // itNokonoko_UnkMotion4_Phys
                    14 => {  ret = m::itNokonoko_UnkMotion6_Coll(&raw mut GO[0]) as u32; } // itNokonoko_UnkMotion6_Coll
                    15 => {  m::it_802DCCCC(&raw mut GO[0]); } // it_802DCCCC
                    16 => {  ret = m::itNokonoko_UnkMotion8_Anim(&raw mut GO[0]) as u32; } // itNokonoko_UnkMotion8_Anim
                    17 => {  m::itNokonoko_UnkMotion7_Phys(&raw mut GO[0]); } // itNokonoko_UnkMotion7_Phys
                    18 => {  ret = m::itNokonoko_UnkMotion8_Coll(&raw mut GO[0]) as u32; } // itNokonoko_UnkMotion8_Coll
                    19 => {  m::it_802DCE00(&raw mut GO[0]); } // it_802DCE00
                    20 => {  m::itNokonoko_UnkMotion8_Phys(&raw mut GO[0]); } // itNokonoko_UnkMotion8_Phys
                    21 => {  m::it_802DCEC4(&raw mut GO[0]); } // it_802DCEC4
                    22 => {  m::it_802DCFBC(&raw mut GO[0]); } // it_802DCFBC
                    23 => {  ret = m::itNokonoko_UnkMotion5_Anim(&raw mut GO[0]) as u32; } // itNokonoko_UnkMotion5_Anim
                    24 => {  m::itNokonoko_UnkMotion5_Phys(&raw mut GO[0]); } // itNokonoko_UnkMotion5_Phys
                    25 => {  ret = m::itNokonoko_UnkMotion5_Coll(&raw mut GO[0]) as u32; } // itNokonoko_UnkMotion5_Coll
                    26 => {  ret = m::itNokonoko_UnkMotion6_Anim(&raw mut GO[0]) as u32; } // itNokonoko_UnkMotion6_Anim
                    27 => {  m::itNokonoko_UnkMotion6_Phys(&raw mut GO[0]); } // itNokonoko_UnkMotion6_Phys
                    28 => {  m::it_802DD290(&raw mut GO[0]); } // it_802DD290
                    29 => {  m::it_802DD2DC(&raw mut GO[0]); } // it_802DD2DC
                    30 => {  ret = m::itNokonoko_UnkMotion3_Anim(&raw mut GO[0]) as u32; } // itNokonoko_UnkMotion3_Anim
                    31 => {  m::itNokonoko_UnkMotion3_Phys(&raw mut GO[0]); } // itNokonoko_UnkMotion3_Phys
                    32 => {  ret = m::itNokonoko_UnkMotion3_Coll(&raw mut GO[0]) as u32; } // itNokonoko_UnkMotion3_Coll
                    33 => { let v1: i32 = fv() as i32 as i32; let v2: u32 = fv() as i32 as u32; m::it_802DD4A8(&raw mut GO[0], v1, v2); } // it_802DD4A8
                    34 => {  m::it_802DD4F4(&raw mut GO[0]); } // it_802DD4F4
                    35 => {  ret = m::itNokonoko_UnkMotion9_Anim(&raw mut GO[0]) as u32; } // itNokonoko_UnkMotion9_Anim
                    36 => {  m::itNokonoko_UnkMotion9_Phys(&raw mut GO[0]); } // itNokonoko_UnkMotion9_Phys
                    37 => {  m::it_802DD59C_inline(&raw mut GO[0]); } // it_802DD59C_inline
                    38 => {  m::it_802DD59C(&raw mut GO[0]); } // it_802DD59C
                    39 => {  m::it_802DD67C(&raw mut GO[0]); } // it_802DD67C
                    40 => {  ret = m::itNokonoko_UnkMotion10_Anim(&raw mut GO[0]) as u32; } // itNokonoko_UnkMotion10_Anim
                    41 => {  m::itNokonoko_UnkMotion10_Phys(&raw mut GO[0]); } // itNokonoko_UnkMotion10_Phys
                    42 => {  ret = m::itNokonoko_UnkMotion10_Coll(&raw mut GO[0]) as u32; } // itNokonoko_UnkMotion10_Coll
                    43 => {  m::it_802DD78C(&raw mut GO[0]); } // it_802DD78C
                    44 => {  m::it_802DD7D0(&raw mut GO[0], &raw mut GO[1]); } // it_802DD7D0
                    45 => { let v0: i32 = (rnd() % 4) as i32 as i32; let v3: i32 = if rnd() & 1 != 0 { 1 } else { -1 } as i32; ret = pi(m::it_802DD7F0(v0, &raw mut VA[0], &raw mut VA[1], v3)); } // it_802DD7F0
                    46 => {  ret = m::it_802DDA84(&raw mut GO[0]) as u32; } // it_802DDA84
                    47 => { for j in 0..11 { let row = &m::it_803F85F0[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 19200);
    }
}
