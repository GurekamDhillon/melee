//! Written by gen/mtu_it2/mk_it_driver.py from the same description as gen/mtu_it2/tu/itlucky.c.
//! src/tu/itlucky.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu_it2/run_it.py itlucky`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "mtu_harness_it/mod.rs"]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itlucky as m;
use mtu_harness::*;
#[path = "mtu/itlucky_gen.rs"]
mod g;

static mut G: HSD_GObj = unsafe { core::mem::zeroed() };
static mut G2: HSD_GObj = unsafe { core::mem::zeroed() };
static mut OWN: HSD_GObj = unsafe { core::mem::zeroed() };
static mut IT: Item = unsafe { core::mem::zeroed() };
static mut IT2: Item = unsafe { core::mem::zeroed() };
static mut ART: Article = unsafe { core::mem::zeroed() };
static mut IA: ItemAttr = unsafe { core::mem::zeroed() };
static mut SA: [u32; 256] = [0; 256];
static mut JO: [HSD_JObj; 4] = unsafe { core::mem::zeroed() };
static mut DBT: DynamicBoneTable = unsafe { core::mem::zeroed() };
static mut V1: Vec = unsafe { core::mem::zeroed() };
static mut V2: Vec = unsafe { core::mem::zeroed() };
static mut ICD: ItemCommonData = unsafe { core::mem::zeroed() };


unsafe fn fill_jo(j: &mut HSD_JObj) {
    j.flags = rnd() & ((JOBJ_MTX_INDEP_SRT | JOBJ_MTX_DIRTY | JOBJ_USER_DEF_MTX | JOBJ_HIDDEN) as u32);
    j.translate.x = rw(); j.translate.y = rw(); j.translate.z = rw();
    j.rotate.x = rw(); j.rotate.y = rw(); j.rotate.z = rw(); j.rotate.w = rw();
    j.scale.x = rw(); j.scale.y = rw(); j.scale.z = rw();
}
unsafe fn hash_jo(j: &HSD_JObj) {
    hm(j.flags);
    hm(j.translate.x.to_bits()); hm(j.translate.y.to_bits()); hm(j.translate.z.to_bits());
    hm(j.rotate.x.to_bits()); hm(j.rotate.y.to_bits()); hm(j.rotate.z.to_bits()); hm(j.rotate.w.to_bits());
    hm(j.scale.x.to_bits()); hm(j.scale.y.to_bits()); hm(j.scale.z.to_bits());
    hm(pi(j.child)); hm(pi(j.next));
}



unsafe fn setup() {
    let mut i: usize;
    G = core::mem::zeroed(); G2 = core::mem::zeroed(); OWN = core::mem::zeroed(); IT = core::mem::zeroed(); IT2 = core::mem::zeroed();
    ART = core::mem::zeroed(); IA = core::mem::zeroed(); SA = [0; 256]; JO = core::mem::zeroed(); DBT = core::mem::zeroed();
    ICD = core::mem::zeroed();
    g::fill_HSD_GObj(&mut G); g::fill_HSD_GObj(&mut G2); g::fill_HSD_GObj(&mut OWN);
    g::fill_Item(&mut IT); g::fill_Item(&mut IT2); g::fill_ItemAttr(&mut IA); g::fill_ItemCommonData(&mut ICD);
    for i in 0..4 { fill_jo(&mut JO[i]); }
    JO[0].child = &raw mut JO[1]; JO[1].child = &raw mut JO[2]; JO[1].next = &raw mut JO[3]; JO[0].next = null_mut();
    for i in 0..256 { SA[i] = if rnd() & 1 != 0 { fv() } else { rw().to_bits() }; }
    V1.x = rw(); V1.y = rw(); V1.z = rw(); V2.x = rw(); V2.y = rw(); V2.z = rw();
    G.user_data = &raw mut IT as *mut _; G.hsd_obj = &raw mut JO[0] as *mut _; G2.user_data = &raw mut IT2 as *mut _; G2.hsd_obj = &raw mut JO[3] as *mut _;
    OWN.user_data = null_mut(); OWN.hsd_obj = null_mut();
    IT.entity = &raw mut G; IT2.entity = &raw mut G2;
    IT.xC4_article_data = &raw mut ART; IT2.xC4_article_data = &raw mut ART; ART.x4_specialAttributes = (&raw mut SA) as *mut _; ART.x0_common_attr = &raw mut IA;
    IT.xCC_item_attr = &raw mut IA; IT2.xCC_item_attr = &raw mut IA;
    IT.xBBC_dynamicBoneTable = &raw mut DBT; IT2.xBBC_dynamicBoneTable = &raw mut DBT;
    for i in 0..100 { DBT.bones[i] = &raw mut JO[i & 3]; }
    IT.owner = if rnd() & 1 != 0 { &raw mut OWN } else { null_mut() }; IT2.owner = null_mut();
    if rnd() % 4 != 0 { let q = rnd() % 5; IT.xDD4_itemVar.lucky.x60 = (match q { 0 => 0, 1 => 1, 2 => 2, 3 => 3, _ => 4 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 4; IT.xDD4_itemVar.lucky.x64 = (match q { 0 => 0, 1 => 1, 2 => 2, _ => 5 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 2; IT.xDD4_itemVar.lucky.x68 = (match q { 0 => 0, _ => 1 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 2; IT.xDAC_itcmd_var0 = (match q { 0 => 0, _ => 1 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 5; IT.xD44_lifeTimer = (match q { 0 => 0.0, 1 => 1.0, 2 => -1.0, 3 => 30.0, _ => 1.5 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 3; IT.xC9C = (match q { 0 => 0, 1 => 1, _ => 100 }) as _; }

}

unsafe fn pre_hash() {

}

unsafe fn hash_all() {
    g::hash_HSD_GObj(&G); g::hash_HSD_GObj(&G2); g::hash_HSD_GObj(&OWN);
    g::hash_Item(&IT); g::hash_Item(&IT2); g::hash_ItemAttr(&IA);
    for i in 0..4 { hash_jo(&JO[i]); }
    for i in 0..256 { hm(SA[i]); }
    hm(V1.x.to_bits()); hm(V1.y.to_bits()); hm(V1.z.to_bits()); hm(V2.x.to_bits()); hm(V2.y.to_bits()); hm(V2.z.to_bits());

}

#[test]
fn itlucky_matches_c() {
    unsafe {
        g::install();
        preg(&raw const G, core::mem::size_of::<HSD_GObj>());
        preg(&raw const G2, core::mem::size_of::<HSD_GObj>());
        preg(&raw const OWN, core::mem::size_of::<HSD_GObj>());
        preg(&raw const IT, core::mem::size_of::<Item>());
        preg(&raw const IT2, core::mem::size_of::<Item>());
        preg(&raw const ART, core::mem::size_of::<Article>());
        preg(&raw const IA, core::mem::size_of::<ItemAttr>());
        preg(&raw const SA, 1024);
        preg(&raw const JO, core::mem::size_of::<[HSD_JObj; 4]>());
        preg(&raw const DBT, core::mem::size_of::<DynamicBoneTable>());
        preg(&raw const V1, core::mem::size_of::<Vec>());
        preg(&raw const V2, core::mem::size_of::<Vec>());
        preg(&raw const ICD, core::mem::size_of::<ItemCommonData>());
        g::freg_all();
        freg(m::it_802D5050 as usize);
        freg(m::it_802D50F0 as usize);
        freg(m::it_802D5124 as usize);
        freg(m::it_802D51C8 as usize);
        freg(m::it_802D52E4 as usize);
        freg(m::it_802D533C_inline as usize);
        freg(m::it_802D533C as usize);
        freg(m::it_802D53AC as usize);
        freg(m::it_802D53F0 as usize);
        freg(m::it_802D53F4 as usize);
        freg(m::it_802D5420 as usize);
        freg(m::it_802D546C_inline as usize);
        freg(m::it_802D546C as usize);
        freg(m::itLucky_UnkMotion5_Anim as usize);
        freg(m::itLucky_UnkMotion5_Phys as usize);
        freg(m::itLucky_UnkMotion5_Coll as usize);
        freg(m::itLucky_UnkMotion6_Anim as usize);
        freg(m::itLucky_UnkMotion6_Phys as usize);
        freg(m::it_802D5560 as usize);
        freg(m::it_802D55DC as usize);
        freg(m::it_802D5600 as usize);
        freg(m::it_802D5620 as usize);
        freg(m::it_802D5648_inline as usize);
        freg(m::it_802D5648 as usize);
        freg(m::it_802D56F0 as usize);
        freg(m::it_802D5710_inline as usize);
        freg(m::it_802D5710 as usize);
        freg(m::itLucky_Logic44_Spawned as usize);
        freg(m::it_802D582C as usize);
        freg(m::it_802D5884 as usize);
        freg(m::it_802D58BC as usize);
        freg(m::it_802D58C0 as usize);
        freg(m::it_802D58EC as usize);
        freg(m::itLucky_UnkMotion3_Anim as usize);
        freg(m::itLucky_UnkMotion3_Phys as usize);
        freg(m::itLucky_UnkMotion3_Coll as usize);
        freg(m::itLucky_Logic44_PickedUp as usize);
        freg(m::itLucky_UnkMotion2_Anim as usize);
        freg(m::itLucky_UnkMotion2_Phys as usize);
        freg(m::itLucky_Logic44_Dropped as usize);
        freg(m::itLucky_Logic44_EnteredAir as usize);
        freg(m::it_802D5A2C as usize);
        freg(m::it_802D5A64 as usize);
        freg(m::it_802D5A68 as usize);
        freg(m::itLucky_Logic44_EvtUnk as usize);
        let mut c = Check::new(include_str!("data/mtu_itlucky.txt"));
        for op in 0..47u32 {
            for k in 0..150u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                    0 => { m::it_802D5050(&raw mut G); }
                    1 => { m::it_802D50F0(&raw mut G); }
                    2 => { ret = m::it_802D5124(&raw mut G) as u32; }
                    3 => { m::it_802D51C8(&raw mut G); }
                    4 => { m::it_802D52E4(&raw mut G); }
                    5 => { m::it_802D533C_inline(&raw mut G); }
                    6 => { m::it_802D533C(&raw mut G); }
                    7 => { ret = m::it_802D53AC(&raw mut G) as u32; }
                    8 => { m::it_802D53F0(&raw mut G); }
                    9 => { ret = m::it_802D53F4(&raw mut G) as u32; }
                    10 => { m::it_802D5420(&raw mut G); }
                    11 => { m::it_802D546C_inline(&raw mut G); }
                    12 => { m::it_802D546C(&raw mut G); }
                    13 => { ret = m::itLucky_UnkMotion5_Anim(&raw mut G) as u32; }
                    14 => { m::itLucky_UnkMotion5_Phys(&raw mut G); }
                    15 => { ret = m::itLucky_UnkMotion5_Coll(&raw mut G) as u32; }
                    16 => { ret = m::itLucky_UnkMotion6_Anim(&raw mut G) as u32; }
                    17 => { m::itLucky_UnkMotion6_Phys(&raw mut G); }
                    18 => { m::it_802D5560(&raw mut G); }
                    19 => { ret = m::it_802D55DC(&raw mut G) as u32; }
                    20 => { m::it_802D5600(&raw mut G); }
                    21 => { ret = m::it_802D5620(&raw mut G) as u32; }
                    22 => { m::it_802D5648_inline(&raw mut G); }
                    23 => { ret = m::it_802D5648(&raw mut G) as u32; }
                    24 => { m::it_802D56F0(&raw mut G, &raw mut G2); }
                    25 => { ret = pi(m::it_802D5710_inline(&raw mut G, &raw mut V1, &raw mut V1, rw())); }
                    26 => { ret = pi(m::it_802D5710(&raw mut G, &raw mut V1, &raw mut V1, rw())); }
                    27 => { m::itLucky_Logic44_Spawned(&raw mut G); }
                    28 => { m::it_802D582C(&raw mut G); }
                    29 => { ret = m::it_802D5884(&raw mut G) as u32; }
                    30 => { m::it_802D58BC(&raw mut G); }
                    31 => { ret = m::it_802D58C0(&raw mut G) as u32; }
                    32 => { m::it_802D58EC(&raw mut G); }
                    33 => { ret = m::itLucky_UnkMotion3_Anim(&raw mut G) as u32; }
                    34 => { m::itLucky_UnkMotion3_Phys(&raw mut G); }
                    35 => { ret = m::itLucky_UnkMotion3_Coll(&raw mut G) as u32; }
                    36 => { m::itLucky_Logic44_PickedUp(&raw mut G); }
                    37 => { ret = m::itLucky_UnkMotion2_Anim(&raw mut G) as u32; }
                    38 => { m::itLucky_UnkMotion2_Phys(&raw mut G); }
                    39 => { m::itLucky_Logic44_Dropped(&raw mut G); }
                    40 => { m::itLucky_Logic44_EnteredAir(&raw mut G); }
                    41 => { ret = m::it_802D5A2C(&raw mut G) as u32; }
                    42 => { m::it_802D5A64(&raw mut G); }
                    43 => { ret = m::it_802D5A68(&raw mut G) as u32; }
                    44 => { m::itLucky_Logic44_EvtUnk(&raw mut G, &raw mut G2); }
                    45 => { { for i in 0..7 { hm(m::it_803F8190[i].anim_id as u32); hm(fi(m::it_803F8190[i].animated)); hm(fi(m::it_803F8190[i].physics_updated)); hm(fi(m::it_803F8190[i].collided)); } } }
                    46 => { { for i in 0..5 { hm(m::it_803F8200[i].anim_id as u32); hm(fi(m::it_803F8200[i].animated)); hm(fi(m::it_803F8200[i].physics_updated)); hm(fi(m::it_803F8200[i].collided)); } } }
                    _ => unreachable!(),
                }
                pre_hash();
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 7050);
    }
}
