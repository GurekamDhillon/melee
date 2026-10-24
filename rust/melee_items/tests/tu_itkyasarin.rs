//! Written by gen/mtu_it/mk_it_driver.py from the same description as gen/mtu_it/tu/itkyasarin.c.
//! src/tu/itkyasarin.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu_it/run_it.py itkyasarin`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itkyasarin as m;
use mtu_harness::*;
#[path = "mtu/itkyasarin_gen.rs"]
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

static mut D40: it_804D6D40_t = unsafe { core::mem::zeroed() };

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
    if rnd() % 4 != 0 { let q = rnd() % 11; IT.msid = (match q { 0 => 0, 1 => 1, 2 => 2, 3 => 3, 4 => 4, 5 => 5, 6 => 6, 7 => 7, 8 => 8, 9 => 9, _ => 10 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 2; IT.facing_dir = (match q { 0 => 1.0, _ => -1.0 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 4; IT.xDD4_itemVar.kyasarin.x24 = (match q { 0 => 0.0, 1 => 0.5, 2 => 1.0, _ => -0.5 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 4; IT.xDD4_itemVar.kyasarin.x28 = (match q { 0 => 0.0, 1 => 0.5, 2 => 1.0, _ => -0.5 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 4; IT.xDD4_itemVar.kyasarin.x2C = (match q { 0 => 0.0, 1 => 0.5, 2 => 1.0, _ => -0.5 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 3; IT.xDD4_itemVar.kyasarin.x30 = (match q { 0 => 1.0, 1 => -1.0, _ => 0.5 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 5; IT.xDD4_itemVar.kyasarin.x34 = (match q { 0 => -1, 1 => 0, 2 => 1, 3 => 2, _ => 5 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 5; IT.xDD4_itemVar.kyasarin.x38 = (match q { 0 => 0, 1 => 10, 2 => 100, 3 => 1000, _ => -5 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 10; IT.xDD4_itemVar.kyasarin.x3C = (match q { 0 => 0, 1 => 1, 2 => 2, 3 => 3, 4 => 4, 5 => 5, 6 => 6, 7 => 7, 8 => 8, _ => 9 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 5; IT.xCA0 = (match q { 0 => 0, 1 => 1, 2 => 5, 3 => 50, _ => 500 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 4; IT.x5D0_animFrameSpeed = (match q { 0 => 0.0, 1 => 0.5, 2 => 1.0, _ => 2.0 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 5; IT.pos.x = (match q { 0 => -50.0, 1 => -10.0, 2 => 0.0, 3 => 10.0, _ => 50.0 }) as _; }
    D40 = core::mem::zeroed(); m::ext::it_804D6D40 = &raw mut D40;
    if rnd() % 4 != 0 { let q = rnd() % 4; SA[5] = (match q { 0 => 20.0, 1 => 3.0, 2 => 0.5, _ => -1.0 } as f32).to_bits(); }
    if rnd() % 4 != 0 { let q = rnd() % 4; SA[7] = (match q { 0 => 20.0, 1 => 3.0, 2 => 0.5, _ => -1.0 } as f32).to_bits(); }
    if rnd() % 4 != 0 { let q = rnd() % 3; SA[11] = (match q { 0 => -20.0, 1 => 0.0, _ => 20.0 } as f32).to_bits(); }
    if rnd() % 4 != 0 { let q = rnd() % 3; SA[10] = (match q { 0 => -20.0, 1 => 0.0, _ => 20.0 } as f32).to_bits(); }
    if rnd() % 4 != 0 { let q = rnd() % 3; SA[13] = (match q { 0 => -20.0, 1 => 0.0, _ => 20.0 } as f32).to_bits(); }
    if rnd() % 4 != 0 { let q = rnd() % 3; SA[12] = (match q { 0 => -20.0, 1 => 0.0, _ => 20.0 } as f32).to_bits(); }
    if rnd() % 4 != 0 { let q = rnd() % 4; SA[16] = (match q { 0 => 0, 1 => 10, 2 => 100, _ => 1000 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 4; SA[17] = (match q { 0 => 0, 1 => 10, 2 => 100, _ => 1000 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 3; D40.x8 = (match q { 0 => 0.0, 1 => 0.5, _ => 1.0 }) as _; }

}

unsafe fn pre_hash() {
    {
        let its: [*mut Item; 2] = [&raw mut IT, &raw mut IT2];
        for k2 in 0..2 {
            let id = pi((*its[k2]).xDD4_itemVar.kyasarin.x20);
            let b = &raw mut (*its[k2]).xDD4_itemVar.kyasarin.x20 as *mut u8;
            for i2 in 0..8 { *b.add(i2) = 0; }
            *(b as *mut u32) = id;
        }
    }

}

unsafe fn hash_all() {
    g::hash_HSD_GObj(&G); g::hash_HSD_GObj(&G2); g::hash_HSD_GObj(&OWN);
    g::hash_Item(&IT); g::hash_Item(&IT2); g::hash_ItemAttr(&IA);
    for i in 0..4 { hash_jo(&JO[i]); }
    for i in 0..256 { hm(SA[i]); }
    hm(V1.x.to_bits()); hm(V1.y.to_bits()); hm(V1.z.to_bits()); hm(V2.x.to_bits()); hm(V2.y.to_bits()); hm(V2.z.to_bits());

}

#[test]
fn itkyasarin_matches_c() {
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
        let mut c = Check::new(include_str!("data/mtu_itkyasarin.txt"));
        for op in 0..32u32 {
            for k in 0..150u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                    0 => { m::itKyasarin_Logic25_Destroyed(&raw mut G); }
                    1 => { m::it_802ECD1C(&raw mut G, &raw mut V1); }
                    2 => { ret = pi(m::it_802ECD3C(&raw mut G, &raw mut V1, rw())); }
                    3 => { m::it_802ECE90(&raw mut G); }
                    4 => { m::it_802ECEB0(&raw mut G); }
                    5 => { ret = m::itKyasarin_UnkMotion0_Anim(&raw mut G) as u32; }
                    6 => { ret = m::itKyasarin_UnkMotion0_Coll(&raw mut G) as u32; }
                    7 => { m::it_802ECFE0(&raw mut G); }
                    8 => { ret = m::itKyasarin_UnkMotion1_Anim(&raw mut G) as u32; }
                    9 => { ret = m::itKyasarin_UnkMotion3_Anim(&raw mut G) as u32; }
                    10 => { m::it_802ED0D0(&raw mut G); }
                    11 => { ret = m::itKyasarin_UnkMotion2_Anim(&raw mut G) as u32; }
                    12 => { ret = m::itKyasarin_UnkMotion2_Coll(&raw mut G) as u32; }
                    13 => { m::it_802ED25C(&raw mut G); }
                    14 => { ret = m::itKyasarin_UnkMotion4_Anim(&raw mut G) as u32; }
                    15 => { ret = m::itKyasarin_UnkMotion4_Coll(&raw mut G) as u32; }
                    16 => { m::it_802ED4F8(&raw mut G); }
                    17 => { ret = m::itKyasarin_UnkMotion6_Anim(&raw mut G) as u32; }
                    18 => { m::it_802ED774(&raw mut G); }
                    19 => { ret = m::itKyasarin_UnkMotion7_Anim(&raw mut G) as u32; }
                    20 => { m::it_802ED8BC(&raw mut G); }
                    21 => { ret = m::itKyasarin_UnkMotion5_Anim(&raw mut G) as u32; }
                    22 => { ret = m::itKyasarin_UnkMotion8_Anim(&raw mut G) as u32; }
                    23 => { ret = m::itKyasarin_UnkMotion9_Anim(&raw mut G) as u32; }
                    24 => { m::itKyasarin_UnkMotion9_Phys(&raw mut G); }
                    25 => { ret = m::itKyasarin_UnkMotion9_Coll(&raw mut G) as u32; }
                    26 => { ret = m::itKyasarin_UnkMotion10_Anim(&raw mut G) as u32; }
                    27 => { m::itKyasarin_UnkMotion10_Phys(&raw mut G); }
                    28 => { ret = m::itKyasarin_UnkMotion10_Coll(&raw mut G) as u32; }
                    29 => { ret = m::it_802EDDC0(&raw mut G) as u32; }
                    30 => { m::it_802EE1E0(&raw mut G, &raw mut G2); }
                    31 => { { for i in 0..11 { hm(m::it_803F90D0[i].anim_id as u32); hm(fi(m::it_803F90D0[i].animated)); hm(fi(m::it_803F90D0[i].physics_updated)); hm(fi(m::it_803F90D0[i].collided)); } } }
                    _ => unreachable!(),
                }
                pre_hash();
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 4800);
    }
}
