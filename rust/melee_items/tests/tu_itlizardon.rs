//! Written by gen/mtu_it2/mk_it_driver.py from the same description as gen/mtu_it2/tu/itlizardon.c.
//! src/tu/itlizardon.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu_it2/run_it.py itlizardon`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "mtu_harness_it/mod.rs"]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itlizardon as m;
use mtu_harness::*;
#[path = "mtu/itlizardon_gen.rs"]
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
    if rnd() % 4 != 0 { let q = rnd() % 4; IT.msid = (match q { 0 => 0, 1 => 1, 2 => 2, _ => 3 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 4; IT.xDD4_itemVar.lizardon.x60 = (match q { 0 => -1, 1 => 0, 2 => 1, _ => 3 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 2; IT.xDD4_itemVar.lizardon.x64 = (match q { 0 => 0, _ => 1 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 2; IT.xDD4_itemVar.lizardon.x6C = (match q { 0 => 0, _ => 1 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 4; IT.xDD4_itemVar.lizardon.x68 = (match q { 0 => 0.0, 1 => 1.0, 2 => 2.0, _ => 5.0 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 4; IT.xDD4_itemVar.lizardon.x70 = (match q { 0 => 0, 1 => 1, 2 => -1, _ => 7 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 3; IT.xDD4_itemVar.lizardon.x74[0] = (match q { 0 => 0, 1 => 1, _ => 5 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 3; IT.xDD4_itemVar.lizardon.x74[1] = (match q { 0 => 0, 1 => 1, _ => 5 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 3; IT.xDD4_itemVar.lizardon.x74[2] = (match q { 0 => 0, 1 => 1, _ => 5 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 3; IT.xDD4_itemVar.lizardon.x74[3] = (match q { 0 => 0, 1 => 1, _ => 5 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 2; IT.xDAC_itcmd_var0 = (match q { 0 => 0, _ => 1 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 2; IT.xDB0_itcmd_var1 = (match q { 0 => 0, _ => 1 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 4; IT.xD44_lifeTimer = (match q { 0 => 0.0, 1 => 1.0, 2 => -1.0, _ => 30.0 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 2; IT.ground_or_air = (match q { 0 => GA_Ground, _ => GA_Air }) as _; }

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
fn itlizardon_matches_c() {
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
        freg(m::it_802CB8AC as usize);
        freg(m::it_802CB93C as usize);
        freg(m::it_802CB940 as usize);
        freg(m::it_802CB960 as usize);
        freg(m::it_802CB994 as usize);
        freg(m::itLizardon_UnkMotion1_Anim as usize);
        freg(m::itLizardon_UnkMotion1_Phys as usize);
        freg(m::itLizardon_UnkMotion1_Coll as usize);
        freg(m::it_802CBAA8_inline as usize);
        freg(m::it_802CBAA8 as usize);
        freg(m::itLizardon_UnkMotion2_Anim as usize);
        freg(m::itLizardon_UnkMotion2_Phys as usize);
        freg(m::itLizardon_UnkMotion2_Coll as usize);
        freg(m::it_802CBD24_inline as usize);
        freg(m::it_802CBD24 as usize);
        freg(m::it_802CBFE4 as usize);
        freg(m::itLizardon_UnkMotion3_Anim as usize);
        freg(m::itLizardon_UnkMotion3_Phys as usize);
        freg(m::itLizardon_UnkMotion3_Coll as usize);
        freg(m::it_802CC0EC as usize);
        freg(m::it_802CC160 as usize);
        freg(m::it_802CC184 as usize);
        freg(m::it_802CC1A4 as usize);
        freg(m::it_802CC1CC as usize);
        freg(m::itLizardon_Logic34_Spawned as usize);
        freg(m::itLizardon_Logic35_Spawned as usize);
        freg(m::itLizardon_Logic36_Spawned as usize);
        freg(m::itLizardon_Logic37_Spawned as usize);
        freg(m::itLizardon_Logic37_EvtUnk as usize);
        freg(m::itLizardon_Logic37_Reflected as usize);
        freg(m::itLizardon_Logic37_HitShield as usize);
        freg(m::itLizardon_Logic37_Absorbed as usize);
        freg(m::it_802CC5D4 as usize);
        freg(m::it_802CC650 as usize);
        freg(m::it_802CC684 as usize);
        freg(m::it_802CC6C4 as usize);
        let mut c = Check::new(include_str!("data/mtu_itlizardon.txt"));
        for op in 0..38u32 {
            for k in 0..150u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                    0 => { m::it_802CB8AC(&raw mut G); }
                    1 => { m::it_802CB93C(&raw mut G); }
                    2 => { m::it_802CB940(&raw mut G, &raw mut G2); }
                    3 => { m::it_802CB960(&raw mut G); }
                    4 => { m::it_802CB994(&raw mut G); }
                    5 => { ret = m::itLizardon_UnkMotion1_Anim(&raw mut G) as u32; }
                    6 => { m::itLizardon_UnkMotion1_Phys(&raw mut G); }
                    7 => { ret = m::itLizardon_UnkMotion1_Coll(&raw mut G) as u32; }
                    8 => { m::it_802CBAA8_inline(&raw mut G); }
                    9 => { m::it_802CBAA8(&raw mut G); }
                    10 => { ret = m::itLizardon_UnkMotion2_Anim(&raw mut G) as u32; }
                    11 => { m::itLizardon_UnkMotion2_Phys(&raw mut G); }
                    12 => { ret = m::itLizardon_UnkMotion2_Coll(&raw mut G) as u32; }
                    13 => { m::it_802CBD24_inline(&raw mut G); }
                    14 => { m::it_802CBD24(&raw mut G); }
                    15 => { m::it_802CBFE4(&raw mut G); }
                    16 => { ret = m::itLizardon_UnkMotion3_Anim(&raw mut G) as u32; }
                    17 => { m::itLizardon_UnkMotion3_Phys(&raw mut G); }
                    18 => { ret = m::itLizardon_UnkMotion3_Coll(&raw mut G) as u32; }
                    19 => { m::it_802CC0EC(&raw mut G); }
                    20 => { ret = m::it_802CC160(&raw mut G) as u32; }
                    21 => { m::it_802CC184(&raw mut G); }
                    22 => { ret = m::it_802CC1A4(&raw mut G) as u32; }
                    23 => { m::it_802CC1CC(&raw mut G, fv() as _); }
                    24 => { m::itLizardon_Logic34_Spawned(&raw mut G); }
                    25 => { m::itLizardon_Logic35_Spawned(&raw mut G); }
                    26 => { m::itLizardon_Logic36_Spawned(&raw mut G); }
                    27 => { m::itLizardon_Logic37_Spawned(&raw mut G); }
                    28 => { m::itLizardon_Logic37_EvtUnk(&raw mut G, &raw mut G2); }
                    29 => { ret = m::itLizardon_Logic37_Reflected(&raw mut G) as u32; }
                    30 => { ret = m::itLizardon_Logic37_HitShield(&raw mut G) as u32; }
                    31 => { ret = m::itLizardon_Logic37_Absorbed(&raw mut G) as u32; }
                    32 => { m::it_802CC5D4(&raw mut G); }
                    33 => { ret = m::it_802CC650(&raw mut G) as u32; }
                    34 => { m::it_802CC684(&raw mut G); }
                    35 => { ret = m::it_802CC6C4(&raw mut G) as u32; }
                    36 => { { for i in 0..4 { hm(m::it_803F7BC0[i].anim_id as u32); hm(fi(m::it_803F7BC0[i].animated)); hm(fi(m::it_803F7BC0[i].physics_updated)); hm(fi(m::it_803F7BC0[i].collided)); } } }
                    37 => { { for i in 0..1 { hm(m::it_803F7C00[i].anim_id as u32); hm(fi(m::it_803F7C00[i].animated)); hm(fi(m::it_803F7C00[i].physics_updated)); hm(fi(m::it_803F7C00[i].collided)); } } }
                    _ => unreachable!(),
                }
                pre_hash();
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 5700);
    }
}
