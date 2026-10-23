//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/itnesspkthunderball.c.
//! src/tu/itnesspkthunderball.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py itnesspkthunderball`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itnesspkthunderball as m;
use mtu_harness::*;
#[path = "kinds_gen/itnesspkthunderball_gen.rs"]
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
                let a = SPECB.0[j].as_mut_ptr() as *mut itPKThunderAttributes;
                (*a).x0_PKTHUNDER_LIFETIME = rw(); (*a).x4_PKTHUNDER_SPEED = rw(); (*a).x8_PKTHUNDER_SPAWN_ANGLE = rw(); (*a).xC_PKTHUNDER_STICK_THRESHOLD = rg(); (*a).x10_PKTHUNDER_TURN_RADIUS = (rnd() % 100) as f32;
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
        IT[k].xDD4_itemVar.pkthunder.xF04 = if i == 0 { null_mut() } else if i == 1 { IT[k].owner } else { &raw mut GO[(k + i as usize) % 3] };
        let pk = &mut IT[k].xDD4_itemVar.pkthunder;
        for j in 0..6 { pk.xDD4[j] = if rnd() & 1 != 0 { &raw mut GO[(k + 1 + j) % 3] } else { null_mut() }; }
        pk.xEF4 = null_mut();
        pk.xF08 = null_mut();
        pk.xEF8 = (rnd() % 14) as i32;
        pk.xEFC = (rnd() % 8) as i32;
        pk.xF00 = (rnd() % 4 == 0) as i32;
        pk.xEEC = rw();
        for j in 0..16 { pk.angles[j] = rg() * 4.0; }
    }
}

unsafe fn hash_all() {
    for k in 0..3 {
        let pk = &mut IT[k].xDD4_itemVar.pkthunder;
        for j in 0..6 { hm(pi(pk.xDD4[j])); pk.xDD4[j] = null_mut(); }
        hm(pi(pk.xEF4)); pk.xEF4 = null_mut(); hm(pi(pk.xF04)); pk.xF04 = null_mut(); hm(pi(pk.xF08)); pk.xF08 = null_mut();
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
fn itnesspkthunderball_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        g::freg_all();
        const PROFTAB: [u8; 23] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_itnesspkthunderball.txt"));
        for op in 0..23u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => { let v2: i32 = (rnd() % 8) as i32 as i32; m::it_802AB3F0(&raw mut GO[0], &raw mut VA[0], v2); } // it_802AB3F0
                    1 => { m::it_802AB468(&raw mut GO[0], &raw mut VA[0].x, fv() as i32); } // it_802AB468
                    2 => { ret = m::it_802AB4B8(&raw mut GO[0], &raw mut IT[0].x378_itemColl) as u32; } // it_802AB4B8
                    3 => {  ret = pi(m::it_802AB568(&raw mut GO[0])); } // it_802AB568
                    4 => { let v2: f32 = rw() as f32; ret = pi(m::it_802AB58C(&raw mut GO[0], &raw mut VA[0], v2)); } // it_802AB58C
                    5 => {  m::it_802AB90C(&raw mut GO[0]); } // it_802AB90C
                    6 => {  m::it_802AB9C0(&raw mut GO[0]); } // it_802AB9C0
                    7 => {  m::it_802ABA4C(&raw mut GO[0]); } // it_802ABA4C
                    8 => {  ret = m::itNesspkthunderball_UnkMotion0_Anim(&raw mut GO[0]) as u32; } // itNesspkthunderball_UnkMotion0_Anim
                    9 => {  m::itNesspkthunderball_UnkMotion0_Phys(&raw mut GO[0]); } // itNesspkthunderball_UnkMotion0_Phys
                    10 => {  ret = m::itNesspkthunderball_UnkMotion0_Coll(&raw mut GO[0]) as u32; } // itNesspkthunderball_UnkMotion0_Coll
                    11 => {  ret = m::itNessPKThunderball_Logic26_DmgDealt(&raw mut GO[0]) as u32; } // itNessPKThunderball_Logic26_DmgDealt
                    12 => {  ret = m::it_802AC074(&raw mut GO[0]) as u32; } // it_802AC074
                    13 => {  ret = m::it_802AC098(&raw mut GO[0]) as u32; } // it_802AC098
                    14 => {  ret = m::it_802AC338(&raw mut GO[0]) as u32; } // it_802AC338
                    15 => {  ret = m::it_802AC35C(&raw mut GO[0]) as u32; } // it_802AC35C
                    16 => {  ret = m::it_802AC3F8(&raw mut GO[0]) as u32; } // it_802AC3F8
                    17 => {  m::it_802AC41C(&raw mut GO[0], &raw mut GO[1]); } // it_802AC41C
                    18 => { m::it_802AB3F0(null_mut(), &raw mut VA[0], 1); } // it_802AB3F0(NULL)
                    19 => { m::it_802AB3F0(&raw mut GO[0], null_mut(), 1); } // it_802AB3F0(out NULL)
                    20 => { m::it_802AB468(null_mut(), &raw mut VA[0].x, 1); } // it_802AB468(NULL)
                    21 => { ret = pi(m::it_802AB568(null_mut())); } // it_802AB568(NULL)
                    22 => { for j in 0..1 { let row = &m::it_803F6BC8[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 9200);
    }
}
