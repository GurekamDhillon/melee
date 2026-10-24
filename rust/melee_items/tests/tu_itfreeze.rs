//! melee_items src/tu/itfreeze.rs vs the C (gen/mtu/tu/itfreeze.c: the whole TU, shipping contraction).
//! Written by rust/melee_world/gen/mtu/mk_itkinds.py from the same description as the C driver;
//! regenerate the expected file with `python rust/melee_world/gen/mtu/run_tu.py itfreeze`.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_items::tu::itfreeze as m;
use melee_types::*;
use mtu_harness::*;
#[path = "mtu/itfreeze_gen.rs"]
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
    fill_itFreeze_ItemVars(&mut ITEM.xDD4_itemVar.freeze);
    ITEM.xC4_article_data = &raw mut ART; ITEM.xCC_item_attr = &raw mut IATTR; ITEM.xBBC_dynamicBoneTable = &raw mut DBT;
    fill_Item(&mut *(&raw mut NEWITEM));
    fill_itFreeze_ItemVars(&mut NEWITEM.xDD4_itemVar.freeze);
    NEWITEM.xC4_article_data = &raw mut ART; NEWITEM.xCC_item_attr = &raw mut IATTR; NEWITEM.xBBC_dynamicBoneTable = &raw mut DBT;
    fill_Item(&mut *(&raw mut REFITEM));
    fill_itFreeze_ItemVars(&mut REFITEM.xDD4_itemVar.freeze);
    REFITEM.xC4_article_data = &raw mut ART; REFITEM.xCC_item_attr = &raw mut IATTR; REFITEM.xBBC_dynamicBoneTable = &raw mut DBT;
    ITEM.entity = &raw mut G; REFITEM.entity = &raw mut REFG; NEWITEM.entity = &raw mut NEWG;
    for i in 0..64usize { *attr_ptr().add(i) = rw(); }
    fill_itUnkAttributes(&mut *(attr_ptr() as *mut itUnkAttributes));
    fill_ItemAttr(&mut *(&raw mut IATTR)); fill_ItemCommonData(&mut *(&raw mut ICD));
    ART.x0_common_attr = &raw mut IATTR; ART.x4_specialAttributes = attr_ptr() as *mut c_void; ART.x10_modelDesc = &raw mut MD;
    MD.x0_joint = if rnd() % 3u32 == 0 { null_mut() } else { &raw mut JNT };
    MD.x4_bone_count = fv(); MD.x8_bone_attach_id = fv() as i32;
    match rnd() % 4u32 { 0 => ITEM.owner = null_mut(), 1 => ITEM.owner = &raw mut OWN, _ => ITEM.owner = &raw mut OWN }
    m::ext::it_804D6D28 = &raw mut ICD;
    { const T: [i64; 3] = [0, 1, 1]; ITEM.ground_or_air = T[(rnd() % 3u32) as usize] as _; }
    ITEM.xDD4_itemVar.freeze.x0 = rg();
    ITEM.xDD4_itemVar.freeze.x10 = rg();
    ITEM.xDD4_itemVar.freeze.x14 = rg();
    { const T: [i64; 5] = [-1, 0, 1, 2, 90]; ITEM.xDD4_itemVar.freeze.x18 = T[(rnd() % 5u32) as usize] as _; }
    match rnd() % 2u32 { 0 => ITEM.xDD4_itemVar.freeze.unk_1C = core::ptr::null_mut(), 1 => ITEM.xDD4_itemVar.freeze.unk_1C = &raw mut REFG as *mut _, _ => unreachable!() }
    { const T: [f32; 3] = [-1.0, 1.0, 0.5]; ITEM.facing_dir = T[(rnd() % 3u32) as usize]; }
}

unsafe fn hash_all() {
    hash_HSD_GObj(&*(&raw const G));
    hash_HSD_GObj(&*(&raw const OWN));
    hash_HSD_GObj(&*(&raw const REFG));
    hash_HSD_GObj(&*(&raw const PARG));
    hash_HSD_GObj(&*(&raw const NEWG));
    hash_Item(&*(&raw const ITEM));
    hash_itFreeze_ItemVars(&ITEM.xDD4_itemVar.freeze);
    hash_Item(&*(&raw const NEWITEM));
    hash_itFreeze_ItemVars(&NEWITEM.xDD4_itemVar.freeze);
    hash_Item(&*(&raw const REFITEM));
    hash_itFreeze_ItemVars(&REFITEM.xDD4_itemVar.freeze);
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
fn itfreeze_matches_c() {
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
        let mut c = Check::new(include_str!("data/mtu_itfreeze.txt"));
        let mut total = 0u32;
        for op in 0..43u32 {
            let ncase = if op == 42 { 1 } else { 300 };
            for k in 0..ncase {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                0 => {
                    let a0: *mut HSD_GObj = if rnd() % 4u32 == 0 { null_mut() } else { &raw mut REFG };
                    fillvec(&raw mut VEC0);
                    let a2 = rw();
                    let a3 = rw();
                    ret = pi(m::it_8028EB88(a0, &raw mut VEC0, a2 as _, a3 as _));
                }
                1 => {
                    let a1 = rw();
                    m::it_8028EC98(&raw mut G, a1 as _);
                }
                2 => {
                    m::it_8028ECE0(&raw mut G);
                }
                3 => {
                    fillvec(&raw mut VEC0);
                    ret = pi(m::it_8028ECF0(&raw mut G, &raw mut VEC0));
                }
                4 => {
                    m::itFreeze_Logic17_Destroyed(&raw mut G);
                }
                5 => {
                    m::it_3F14_Logic17_Spawned(&raw mut G);
                }
                6 => {
                    m::it_8028EDBC(&raw mut G);
                }
                7 => {
                    m::it_8028EF34(&raw mut G);
                }
                8 => {
                    ret = m::itFreeze_UnkMotion0_Anim(&raw mut G) as u32;
                }
                9 => {
                    m::itFreeze_UnkMotion0_Phys(&raw mut G);
                }
                10 => {
                    ret = m::itFreeze_UnkMotion0_Coll(&raw mut G) as u32;
                }
                11 => {
                    m::it_8028F1D8(&raw mut G);
                }
                12 => {
                    ret = m::itFreeze_UnkMotion3_Anim(&raw mut G) as u32;
                }
                13 => {
                    m::itFreeze_UnkMotion1_Phys(&raw mut G);
                }
                14 => {
                    ret = m::itFreeze_UnkMotion1_Coll(&raw mut G) as u32;
                }
                15 => {
                    m::itFreeze_Logic17_PickedUp(&raw mut G);
                }
                16 => {
                    ret = m::itFreeze_UnkMotion2_Anim(&raw mut G) as u32;
                }
                17 => {
                    m::itFreeze_Logic17_Dropped(&raw mut G);
                }
                18 => {
                    m::itFreeze_Logic17_Thrown(&raw mut G);
                }
                19 => {
                    m::itFreeze_UnkMotion3_Phys(&raw mut G);
                }
                20 => {
                    ret = m::itFreeze_UnkMotion3_Coll(&raw mut G) as u32;
                }
                21 => {
                    ret = m::itFreeze_Logic17_DmgDealt(&raw mut G) as u32;
                }
                22 => {
                    ret = m::itFreeze_Logic17_Clanked(&raw mut G) as u32;
                }
                23 => {
                    ret = m::itFreeze_Logic17_HitShield(&raw mut G) as u32;
                }
                24 => {
                    ret = m::itFreeze_Logic17_Absorbed(&raw mut G) as u32;
                }
                25 => {
                    ret = m::itFreeze_Logic17_Reflected(&raw mut G) as u32;
                }
                26 => {
                    ret = m::itFreeze_Logic17_ShieldBounced(&raw mut G) as u32;
                }
                27 => {
                    ret = m::itFreeze_Logic17_DmgReceived(&raw mut G) as u32;
                }
                28 => {
                    let a1 = rw();
                    let a2: *mut HSD_GObj = if rnd() % 4u32 == 0 { null_mut() } else { &raw mut REFG };
                    m::it_8028F434(&raw mut G, a1 as _, a2);
                }
                29 => {
                    ret = m::itFreeze_UnkMotion4_Anim(&raw mut G) as u32;
                }
                30 => {
                    m::itFreeze_UnkMotion4_Phys(&raw mut G);
                }
                31 => {
                    m::itFreeze_ClearLinkedItem(&raw mut G);
                }
                32 => {
                    m::itFreeze_ResetToMotion0(&raw mut G);
                }
                33 => {
                    ret = m::itFreeze_UnkMotion4_Coll(&raw mut G) as u32;
                }
                34 => {
                    m::it_8028F7C8(&raw mut G);
                }
                35 => {
                    ret = m::itFreeze_UnkMotion5_Anim(&raw mut G) as u32;
                }
                36 => {
                    m::itFreeze_UnkMotion5_Phys(&raw mut G);
                }
                37 => {
                    ret = m::itFreeze_UnkMotion5_Coll(&raw mut G) as u32;
                }
                38 => {
                    m::itFreeze_Logic17_EvtUnk(&raw mut G, &raw mut REFG);
                }
                39 => {
                    m::it_8028F8E4(&raw mut G);
                }
                40 => {
                    m::it_8028F968(&raw mut G);
                }
                41 => {
                    m::it_8028F9B8(&raw mut G);
                }
                42 => {
                    for i in 0..6 { hm(m::it_803F5D28[i].anim_id as u32); hm(fi(m::it_803F5D28[i].animated)); hm(fi(m::it_803F5D28[i].physics_updated)); hm(fi(m::it_803F5D28[i].collided)); }
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
