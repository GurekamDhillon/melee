//! melee_items src/tu/itfoxblaster.rs vs the C (gen/mtu/tu/itfoxblaster.c: the whole TU, shipping contraction).
//! Written by rust/melee_world/gen/mtu/mk_itkinds.py from the same description as the C driver;
//! regenerate the expected file with `python rust/melee_world/gen/mtu/run_tu.py itfoxblaster`.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_items::tu::itfoxblaster as m;
use melee_types::*;
use mtu_harness::*;
#[path = "mtu/itfoxblaster_gen.rs"]
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
    fill_itFoxBlaster_ItemVars(&mut ITEM.xDD4_itemVar.foxblaster);
    ITEM.xC4_article_data = &raw mut ART; ITEM.xCC_item_attr = &raw mut IATTR; ITEM.xBBC_dynamicBoneTable = &raw mut DBT;
    fill_Item(&mut *(&raw mut NEWITEM));
    fill_itFoxBlaster_ItemVars(&mut NEWITEM.xDD4_itemVar.foxblaster);
    NEWITEM.xC4_article_data = &raw mut ART; NEWITEM.xCC_item_attr = &raw mut IATTR; NEWITEM.xBBC_dynamicBoneTable = &raw mut DBT;
    fill_Item(&mut *(&raw mut REFITEM));
    fill_itFoxBlaster_ItemVars(&mut REFITEM.xDD4_itemVar.foxblaster);
    REFITEM.xC4_article_data = &raw mut ART; REFITEM.xCC_item_attr = &raw mut IATTR; REFITEM.xBBC_dynamicBoneTable = &raw mut DBT;
    ITEM.entity = &raw mut G; REFITEM.entity = &raw mut REFG; NEWITEM.entity = &raw mut NEWG;
    for i in 0..64usize { *attr_ptr().add(i) = rw(); }
    fill_ItemAttr(&mut *(&raw mut IATTR)); fill_ItemCommonData(&mut *(&raw mut ICD));
    ART.x0_common_attr = &raw mut IATTR; ART.x4_specialAttributes = attr_ptr() as *mut c_void; ART.x10_modelDesc = &raw mut MD;
    MD.x0_joint = if rnd() % 3u32 == 0 { null_mut() } else { &raw mut JNT };
    MD.x4_bone_count = fv(); MD.x8_bone_attach_id = fv() as i32;
    match rnd() % 4u32 { 0 => ITEM.owner = null_mut(), 1 => ITEM.owner = &raw mut OWN, _ => ITEM.owner = &raw mut OWN }
    { const T: [i64; 10] = [74, 75, 138, 139, 74, 75, 138, 139, 0, 5]; ITEM.kind = T[(rnd() % 10u32) as usize] as _; }
    { const T: [i64; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]; ITEM.msid = T[(rnd() % 12u32) as usize] as _; }
    ITEM.xDD4_itemVar.foxblaster.xDD4 = ((0) + (rnd() % 14u32) as i64) as _;
    ITEM.xDD4_itemVar.foxblaster.xDD8 = ((0) + (rnd() % 5u32) as i64) as _;
    { const T: [i64; 5] = [-2, -1, 0, 1, 2]; ITEM.xDD4_itemVar.foxblaster.xDDC = T[(rnd() % 5u32) as usize] as _; }
    { const T: [i64; 2] = [0, 1]; ITEM.xDD4_itemVar.foxblaster.xDE0 = T[(rnd() % 2u32) as usize] as _; }
    { const T: [i64; 2] = [0, 1]; ITEM.xDD4_itemVar.foxblaster.gfx_spawn_var = T[(rnd() % 2u32) as usize] as _; }
    { const T: [i64; 4] = [0, 1, 2, 5]; ITEM.xDD4_itemVar.foxblaster.set_sfx_var2 = T[(rnd() % 4u32) as usize] as _; }
    match rnd() % 3u32 { 0 => ITEM.xDD4_itemVar.foxblaster.owner = core::ptr::null_mut(), 1 => ITEM.xDD4_itemVar.foxblaster.owner = &raw mut OWN as *mut _, 2 => ITEM.xDD4_itemVar.foxblaster.owner = &raw mut OWN as *mut _, _ => unreachable!() }
    match rnd() % 4u32 { 0 => ITEM.owner = core::ptr::null_mut(), 1 => ITEM.owner = &raw mut OWN as *mut _, 2 => ITEM.owner = &raw mut OWN as *mut _, 3 => ITEM.owner = &raw mut REFG as *mut _, _ => unreachable!() }
}

unsafe fn hash_all() {
    hash_HSD_GObj(&*(&raw const G));
    hash_HSD_GObj(&*(&raw const OWN));
    hash_HSD_GObj(&*(&raw const REFG));
    hash_HSD_GObj(&*(&raw const PARG));
    hash_HSD_GObj(&*(&raw const NEWG));
    hash_Item(&*(&raw const ITEM));
    hash_itFoxBlaster_ItemVars(&ITEM.xDD4_itemVar.foxblaster);
    hash_Item(&*(&raw const NEWITEM));
    hash_itFoxBlaster_ItemVars(&NEWITEM.xDD4_itemVar.foxblaster);
    hash_Item(&*(&raw const REFITEM));
    hash_itFoxBlaster_ItemVars(&REFITEM.xDD4_itemVar.foxblaster);
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
fn itfoxblaster_matches_c() {
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
        let mut c = Check::new(include_str!("data/mtu_itfoxblaster.txt"));
        let mut total = 0u32;
        for op in 0..31u32 {
            let ncase = if op == 30 { 1 } else { 300 };
            for k in 0..ncase {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                0 => {
                    let a1 = fv();
                    let a2 = fv();
                    m::itFoxBlaster_PlaySFX(&raw mut ITEM, a1 as _, a2 as _);
                }
                1 => {
                    match rnd() % 4u32 { 0 => G.user_data = &raw mut ITEM as *mut _, 1 => G.user_data = &raw mut ITEM as *mut _, 2 => G.user_data = &raw mut ITEM as *mut _, 3 => G.user_data = core::ptr::null_mut(), _ => unreachable!() }
                    let a0: *mut HSD_GObj = if rnd() % 8u32 == 0 { null_mut() } else { &raw mut G };
                    let a1 = ((0) + (rnd() % 4u32) as i64) as i32;
                    m::it_802ADDD0(a0, a1 as _);
                }
                2 => {
                    m::it_802ADEF0(&raw mut G);
                }
                3 => {
                    fillvec(&raw mut VEC0);
                    fillvec(&raw mut VEC1);
                    m::itFoxBlaster_SetShotOffset(&raw mut ITEM, &raw mut VEC0, &raw mut VEC1);
                }
                4 => {
                    fillvec(&raw mut VEC0);
                    fillvec(&raw mut VEC1);
                    m::itFoxBlaster_SetShotAngle(&raw mut ITEM, &raw mut VEC0, &raw mut VEC1);
                }
                5 => {
                    m::it_802ADF10(&raw mut G);
                }
                6 => {
                    m::it_802AE1D0(&raw mut G);
                }
                7 => {
                    match rnd() % 4u32 { 0 => G.user_data = &raw mut ITEM as *mut _, 1 => G.user_data = &raw mut ITEM as *mut _, 2 => G.user_data = &raw mut ITEM as *mut _, 3 => G.user_data = core::ptr::null_mut(), _ => unreachable!() }
                    m::it_802AE200(&raw mut G);
                }
                8 => {
                    m::it_802AE538(&raw mut G);
                }
                9 => {
                    m::it_802AE608(&raw mut G);
                }
                10 => {
                    match rnd() % 4u32 { 0 => G.user_data = &raw mut ITEM as *mut _, 1 => G.user_data = &raw mut ITEM as *mut _, 2 => G.user_data = &raw mut ITEM as *mut _, 3 => G.user_data = core::ptr::null_mut(), _ => unreachable!() }
                    m::it_802AE63C(&raw mut G);
                }
                11 => {
                    let a1 = fv() as i32;
                    m::itFoxBlaster_SetCommandVars(&raw mut ITEM, a1 as _);
                }
                12 => {
                    let a1 = ((0) + (rnd() % 6u32) as i64) as i32;
                    m::itFoxBlaster_ClearShot(&raw mut G, a1 as _);
                }
                13 => {
                    m::it_802AE7B8(&raw mut G);
                }
                14 => {
                    let a0 = rw();
                    let a1: *mut HSD_GObj = if rnd() % 4u32 == 0 { null_mut() } else { &raw mut REFG };
                    fillvec(&raw mut VEC0);
                    let a3 = fv() as i32;
                    let a4 = fv() as i32;
                    ret = pi(m::it_802AE8A8(a0 as _, a1, &raw mut VEC0, a3 as _, a4 as _));
                }
                15 => {
                    let a0: *mut HSD_GObj = if rnd() % 4u32 == 0 { null_mut() } else { &raw mut REFG };
                    let a1 = fv() as i32;
                    let a2 = fv() as i32;
                    ret = pi(m::it_802AE994(a0, a1 as _, a2 as _));
                }
                16 => {
                    let a0: *mut HSD_GObj = if rnd() % 8u32 == 0 { null_mut() } else { &raw mut G };
                    m::it_802AEAB4(a0);
                }
                17 => {
                    m::itFoxBlaster_Logic96_PickedUp(&raw mut G);
                }
                18 => {
                    match rnd() % 4u32 { 0 => G.user_data = &raw mut ITEM as *mut _, 1 => G.user_data = &raw mut ITEM as *mut _, 2 => G.user_data = &raw mut ITEM as *mut _, 3 => G.user_data = core::ptr::null_mut(), _ => unreachable!() }
                    m::clear_blaster_references(&raw mut G);
                }
                19 => {
                    m::clear_blaster(&raw mut G);
                }
                20 => {
                    ret = m::itFoxblaster_UnkMotion8_Anim(&raw mut G) as u32;
                }
                21 => {
                    m::itFoxblaster_UnkMotion8_Phys(&raw mut G);
                }
                22 => {
                    ret = m::itFoxblaster_UnkMotion8_Coll(&raw mut G) as u32;
                }
                23 => {
                    ret = m::itFoxblaster_UnkMotion9_Anim(&raw mut G) as u32;
                }
                24 => {
                    m::itFoxblaster_UnkMotion9_Phys(&raw mut G);
                }
                25 => {
                    ret = m::itFoxblaster_UnkMotion9_Coll(&raw mut G) as u32;
                }
                26 => {
                    match rnd() % 4u32 { 0 => G.user_data = &raw mut ITEM as *mut _, 1 => G.user_data = &raw mut ITEM as *mut _, 2 => G.user_data = &raw mut ITEM as *mut _, 3 => G.user_data = core::ptr::null_mut(), _ => unreachable!() }
                    let a0: *mut HSD_GObj = if rnd() % 8u32 == 0 { null_mut() } else { &raw mut G };
                    ret = m::itFoxblaster_UnkMotion10_Anim(a0) as u32;
                }
                27 => {
                    m::itFoxblaster_UnkMotion10_Phys(&raw mut G);
                }
                28 => {
                    ret = m::itFoxblaster_UnkMotion10_Coll(&raw mut G) as u32;
                }
                29 => {
                    m::itFoxBlaster_Logic96_EvtUnk(&raw mut G, &raw mut REFG);
                }
                30 => {
                    for i in 0..11 { hm(m::it_803F6CA8[i].anim_id as u32); hm(fi(m::it_803F6CA8[i].animated)); hm(fi(m::it_803F6CA8[i].physics_updated)); hm(fi(m::it_803F6CA8[i].collided)); }
                    for i in 0..14 { hm(m::it_803F6D58[i].to_bits()); }
                    for i in 0..14 { hm(m::it_803F6D90[i].to_bits()); }
                    for i in 0..14 { hm(m::it_803F6DC8[i].to_bits()); }
                    for i in 0..5 { hm(m::it_803F6E28[i].to_bits()); }
                    for i in 0..11 { hm(m::it_803F6E3C[i] as u32); }
                    for i in 0..11 { hm(m::it_803F6E68[i] as u32); }
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
