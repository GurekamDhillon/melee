//! melee_items src/tu/itgamewatchchef.rs vs the C (gen/mtu/tu/itgamewatchchef.c: the whole TU, shipping contraction).
//! Written by rust/melee_world/gen/mtu/mk_itkinds.py from the same description as the C driver;
//! regenerate the expected file with `python rust/melee_world/gen/mtu/run_tu.py itgamewatchchef`.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_items::tu::itgamewatchchef as m;
use melee_types::*;
use mtu_harness::*;
#[path = "mtu/itgamewatchchef_gen.rs"]
mod g;
use g::*;

static mut G: HSD_GObj = unsafe { core::mem::zeroed() };
static mut OWN: HSD_GObj = unsafe { core::mem::zeroed() };
static mut REFG: HSD_GObj = unsafe { core::mem::zeroed() };
static mut PARG: HSD_GObj = unsafe { core::mem::zeroed() };
static mut NEWG: HSD_GObj = unsafe { core::mem::zeroed() };
static mut ITEM: Item = unsafe { core::mem::zeroed() };
static mut NEWITEM: Item = unsafe { core::mem::zeroed() };
static mut REFITEM: Item = unsafe { core::mem::zeroed() };
static mut JO: [HSD_JObj; 8] = unsafe { core::mem::zeroed() };
static mut OJO: HSD_JObj = unsafe { core::mem::zeroed() };
static mut GJO: HSD_JObj = unsafe { core::mem::zeroed() };
static mut LJO: HSD_JObj = unsafe { core::mem::zeroed() };
static mut ART: Article = unsafe { core::mem::zeroed() };
static mut MD: ItemModelDesc = unsafe { core::mem::zeroed() };
static mut IATTR: ItemAttr = unsafe { core::mem::zeroed() };
static mut ICD: ItemCommonData = unsafe { core::mem::zeroed() };
static mut JNT: HSD_Joint = unsafe { core::mem::zeroed() };
static mut DBT: DynamicBoneTable = unsafe { core::mem::zeroed() };
static mut ATTRD: [f64; 32] = [0.0; 32];
static mut VEC0: Vec = unsafe { core::mem::zeroed() };
static mut VEC1: Vec = unsafe { core::mem::zeroed() };

unsafe fn attr_ptr() -> *mut f32 {
    &raw mut ATTRD as *mut f32
}

unsafe fn fillvec(v: *mut Vec) {
    (*v).x = rw();
    (*v).y = rw();
    (*v).z = rw();
}

unsafe fn setup() {
    G = core::mem::zeroed(); OWN = core::mem::zeroed(); REFG = core::mem::zeroed(); PARG = core::mem::zeroed();
    NEWG = core::mem::zeroed(); ITEM = core::mem::zeroed(); NEWITEM = core::mem::zeroed(); REFITEM = core::mem::zeroed();
    JO = core::mem::zeroed(); OJO = core::mem::zeroed(); GJO = core::mem::zeroed(); LJO = core::mem::zeroed();
    ART = core::mem::zeroed(); MD = core::mem::zeroed(); IATTR = core::mem::zeroed(); ICD = core::mem::zeroed();
    JNT = core::mem::zeroed(); DBT = core::mem::zeroed(); ATTRD = [0.0; 32]; VEC0 = core::mem::zeroed();
    VEC1 = core::mem::zeroed();
    fill_HSD_GObj(&mut *(&raw mut G));
    fill_HSD_GObj(&mut *(&raw mut OWN));
    fill_HSD_GObj(&mut *(&raw mut REFG));
    fill_HSD_GObj(&mut *(&raw mut PARG));
    fill_HSD_GObj(&mut *(&raw mut NEWG));
    G.user_data = &raw mut ITEM as *mut c_void; G.hsd_obj = &raw mut JO[0] as *mut c_void; OWN.hsd_obj = &raw mut OJO as *mut c_void; REFG.user_data = &raw mut REFITEM as *mut c_void; REFG.hsd_obj = &raw mut OJO as *mut c_void; PARG.hsd_obj = &raw mut OJO as *mut c_void; NEWG.user_data = &raw mut NEWITEM as *mut c_void; NEWG.hsd_obj = &raw mut OJO as *mut c_void;
    fill_HSD_JObj(&mut *(&raw mut JO[0])); JO[0].flags &= !(JOBJ_USE_QUATERNION as u32);
    fill_HSD_JObj(&mut *(&raw mut JO[1])); JO[1].flags &= !(JOBJ_USE_QUATERNION as u32);
    fill_HSD_JObj(&mut *(&raw mut JO[2])); JO[2].flags &= !(JOBJ_USE_QUATERNION as u32);
    fill_HSD_JObj(&mut *(&raw mut JO[3])); JO[3].flags &= !(JOBJ_USE_QUATERNION as u32);
    fill_HSD_JObj(&mut *(&raw mut JO[4])); JO[4].flags &= !(JOBJ_USE_QUATERNION as u32);
    fill_HSD_JObj(&mut *(&raw mut JO[5])); JO[5].flags &= !(JOBJ_USE_QUATERNION as u32);
    fill_HSD_JObj(&mut *(&raw mut JO[6])); JO[6].flags &= !(JOBJ_USE_QUATERNION as u32);
    fill_HSD_JObj(&mut *(&raw mut JO[7])); JO[7].flags &= !(JOBJ_USE_QUATERNION as u32);
    fill_HSD_JObj(&mut *(&raw mut OJO)); OJO.flags &= !(JOBJ_USE_QUATERNION as u32);
    fill_HSD_JObj(&mut *(&raw mut GJO)); GJO.flags &= !(JOBJ_USE_QUATERNION as u32);
    fill_HSD_JObj(&mut *(&raw mut LJO)); LJO.flags &= !(JOBJ_USE_QUATERNION as u32);
    if rnd() % 6u32 != 0 { JO[0].child = &raw mut JO[1]; }
    if rnd() % 6u32 != 0 { JO[1].child = &raw mut JO[2]; }
    if rnd() % 6u32 != 0 { JO[2].child = &raw mut JO[3]; }
    if rnd() % 6u32 != 0 { JO[3].child = &raw mut JO[4]; }
    JO[4].next = &raw mut JO[5]; JO[5].next = &raw mut JO[6]; JO[6].next = &raw mut JO[7];
    for i in 0..100usize { DBT.bones[i] = &raw mut JO[i % 8]; }
    fill_Item(&mut *(&raw mut ITEM));
    fill_itGamewatchchef_ItemVars(&mut ITEM.xDD4_itemVar.gamewatchchef);
    ITEM.xC4_article_data = &raw mut ART; ITEM.xCC_item_attr = &raw mut IATTR; ITEM.xBBC_dynamicBoneTable = &raw mut DBT;
    fill_Item(&mut *(&raw mut NEWITEM));
    fill_itGamewatchchef_ItemVars(&mut NEWITEM.xDD4_itemVar.gamewatchchef);
    NEWITEM.xC4_article_data = &raw mut ART; NEWITEM.xCC_item_attr = &raw mut IATTR; NEWITEM.xBBC_dynamicBoneTable = &raw mut DBT;
    fill_Item(&mut *(&raw mut REFITEM));
    fill_itGamewatchchef_ItemVars(&mut REFITEM.xDD4_itemVar.gamewatchchef);
    REFITEM.xC4_article_data = &raw mut ART; REFITEM.xCC_item_attr = &raw mut IATTR; REFITEM.xBBC_dynamicBoneTable = &raw mut DBT;
    ITEM.entity = &raw mut G; REFITEM.entity = &raw mut REFG; NEWITEM.entity = &raw mut NEWG;
    for i in 0..64usize { *attr_ptr().add(i) = rw(); }
    fill_itGamewatchchefAttributes(&mut *(attr_ptr() as *mut itGamewatchchefAttributes));
    fill_ItemAttr(&mut *(&raw mut IATTR)); fill_ItemCommonData(&mut *(&raw mut ICD));
    ART.x0_common_attr = &raw mut IATTR; ART.x4_specialAttributes = attr_ptr() as *mut c_void; ART.x10_modelDesc = &raw mut MD;
    MD.x0_joint = if rnd() % 3u32 == 0 { null_mut() } else { &raw mut JNT };
    MD.x4_bone_count = fv(); MD.x8_bone_attach_id = fv() as i32;
    match rnd() % 4u32 { 0 => ITEM.owner = null_mut(), 1 => ITEM.owner = &raw mut OWN, _ => ITEM.owner = &raw mut OWN }
    ITEM.xDD4_itemVar.gamewatchchef.x4 = ((0) + (rnd() % 5u32) as i64) as _;
    { let r = rnd(); if r % 3u32 == 0 { ITEM.x40_vel.x = 0.0; } else { ITEM.x40_vel.x = rw(); } }
    { const T: [f32; 8] = [0.0, 1.0, 1.5, 2.5, 3.5, 5.5, 10.0, -1.0]; ITEM.xD44_lifeTimer = T[(rnd() % 8u32) as usize]; }
}

unsafe fn hash_all() {
    hash_HSD_GObj(&*(&raw const G));
    hash_HSD_GObj(&*(&raw const OWN));
    hash_HSD_GObj(&*(&raw const REFG));
    hash_HSD_GObj(&*(&raw const PARG));
    hash_HSD_GObj(&*(&raw const NEWG));
    hash_Item(&*(&raw const ITEM));
    hash_itGamewatchchef_ItemVars(&ITEM.xDD4_itemVar.gamewatchchef);
    hash_Item(&*(&raw const NEWITEM));
    hash_itGamewatchchef_ItemVars(&NEWITEM.xDD4_itemVar.gamewatchchef);
    hash_Item(&*(&raw const REFITEM));
    hash_itGamewatchchef_ItemVars(&REFITEM.xDD4_itemVar.gamewatchchef);
    hash_HSD_JObj(&*(&raw const JO[0]));
    hash_HSD_JObj(&*(&raw const JO[1]));
    hash_HSD_JObj(&*(&raw const JO[2]));
    hash_HSD_JObj(&*(&raw const JO[3]));
    hash_HSD_JObj(&*(&raw const JO[4]));
    hash_HSD_JObj(&*(&raw const JO[5]));
    hash_HSD_JObj(&*(&raw const JO[6]));
    hash_HSD_JObj(&*(&raw const JO[7]));
    hash_HSD_JObj(&*(&raw const OJO));
    hash_HSD_JObj(&*(&raw const GJO));
    hash_HSD_JObj(&*(&raw const LJO));
    hm(rh(&*(&raw const ATTRD))); hash_ItemAttr(&*(&raw const IATTR));
    hm(VEC0.x.to_bits());
    hm(VEC0.y.to_bits());
    hm(VEC0.z.to_bits());
    hm(VEC1.x.to_bits());
    hm(VEC1.y.to_bits());
    hm(VEC1.z.to_bits());
}

#[test]
fn itgamewatchchef_matches_c() {
    unsafe {
        g::install();
        preg(&raw const G, core::mem::size_of::<HSD_GObj>());
        preg(&raw const OWN, core::mem::size_of::<HSD_GObj>());
        preg(&raw const REFG, core::mem::size_of::<HSD_GObj>());
        preg(&raw const PARG, core::mem::size_of::<HSD_GObj>());
        preg(&raw const NEWG, core::mem::size_of::<HSD_GObj>());
        preg(&raw const ITEM, core::mem::size_of::<Item>());
        preg(&raw const NEWITEM, core::mem::size_of::<Item>());
        preg(&raw const REFITEM, core::mem::size_of::<Item>());
        preg(&raw const JO, core::mem::size_of_val(&JO));
        preg(&raw const OJO, core::mem::size_of::<HSD_JObj>());
        preg(&raw const GJO, core::mem::size_of::<HSD_JObj>());
        preg(&raw const LJO, core::mem::size_of::<HSD_JObj>());
        preg(&raw const ART, core::mem::size_of::<Article>());
        preg(&raw const MD, core::mem::size_of::<ItemModelDesc>());
        preg(&raw const IATTR, core::mem::size_of::<ItemAttr>());
        preg(&raw const ICD, core::mem::size_of::<ItemCommonData>());
        preg(&raw const JNT, core::mem::size_of::<HSD_Joint>());
        preg(&raw const DBT, core::mem::size_of::<DynamicBoneTable>());
        preg(&raw const ATTRD, core::mem::size_of_val(&ATTRD));
        preg(&raw const VEC0, core::mem::size_of::<Vec>());
        preg(&raw const VEC1, core::mem::size_of::<Vec>());
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_itgamewatchchef.txt"));
        let mut total = 0u32;
        for op in 0..17u32 {
            let ncase = if op == 16 { 1 } else { 300 };
            for k in 0..ncase {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                0 => {
                    fillvec(&raw mut VEC0);
                    let a2 = fv() as i32;
                    let a3 = 0u32 + rnd() % 5u32;
                    let a4 = rw();
                    ret = pi(m::it_802C837C(&raw mut PARG, &raw mut VEC0, a2 as _, a3 as _, a4 as _));
                }
                1 => {
                    ret = m::itGameWatchChef_Logic112_DmgDealt(&raw mut G) as u32;
                }
                2 => {
                    let a1 = ((0) + (rnd() % 5u32) as i64) as i32;
                    m::it_802C84A0(&raw mut G, a1 as _);
                }
                3 => {
                    ret = m::itGamewatchchef_UnkMotion0_Anim(&raw mut G) as u32;
                }
                4 => {
                    m::itGamewatchchef_UnkMotion0_Phys(&raw mut G);
                }
                5 => {
                    ret = m::itGamewatchchef_UnkMotion0_Coll(&raw mut G) as u32;
                }
                6 => {
                    m::it_802C875C(&raw mut G);
                }
                7 => {
                    ret = m::itGamewatchchef_UnkMotion1_Anim(&raw mut G) as u32;
                }
                8 => {
                    m::itGamewatchchef_UnkMotion1_Phys(&raw mut G);
                }
                9 => {
                    ret = m::itGamewatchchef_UnkMotion1_Coll(&raw mut G) as u32;
                }
                10 => {
                    ret = m::it_2725_Logic112_Clanked(&raw mut G) as u32;
                }
                11 => {
                    ret = m::it_2725_Logic112_HitShield(&raw mut G) as u32;
                }
                12 => {
                    ret = m::it_2725_Logic112_Absorbed(&raw mut G) as u32;
                }
                13 => {
                    ret = m::itGameWatchChef_Logic112_ShieldBounced(&raw mut G) as u32;
                }
                14 => {
                    ret = m::itGameWatchChef_Logic112_Reflected(&raw mut G) as u32;
                }
                15 => {
                    m::itGameWatchChef_Logic112_EvtUnk(&raw mut G, &raw mut REFG);
                }
                16 => {
                    for i in 0..2 { hm(m::it_803F79E0[i].anim_id as u32); hm(fi(m::it_803F79E0[i].animated)); hm(fi(m::it_803F79E0[i].physics_updated)); hm(fi(m::it_803F79E0[i].collided)); }
                }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
                total += 1;
            }
        }
        assert_eq!(c.done(), total);
    }
}
