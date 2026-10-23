//! Written by gen/mtu/mk_it_driver.py from the same description as gen/mtu/tu/itfflower.c.
//! src/tu/itfflower.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu/run_it.py itfflower`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itfflower as m;
use mtu_harness::*;
#[path = "mtu/itfflower_gen.rs"]
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
    if rnd() % 4 != 0 { let q = rnd() % 8; IT.msid = (match q { 0 => 0, 1 => 1, 2 => 2, 3 => 3, 4 => 4, 5 => 5, 6 => 6, _ => 7 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 4; IT.xD4C = (match q { 0 => -1, 1 => 0, 2 => 1, _ => 5 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 4; IT.xDD4_itemVar.fflower.x0 = (match q { 0 => 0, 1 => 1, 2 => 2, _ => 5 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 2; IT.xDD4_itemVar.fflower.x4 = (match q { 0 => 0, _ => 1 }) as _; }

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
fn itfflower_matches_c() {
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
        let mut c = Check::new(include_str!("data/mtu_itfflower.txt"));
        for op in 0..33u32 {
            for k in 0..150u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                    0 => { m::msid_check(&raw mut G, &raw mut IT); }
                    1 => { ret = pi(m::it_80292D48(&raw mut V1)); }
                    2 => { m::itFFlower_Logic25_Spawned(&raw mut G); }
                    3 => { m::it_80292E64(&raw mut G); }
                    4 => { m::it_80292EAC(&raw mut G); }
                    5 => { m::it_80292EF8(&raw mut G, &raw mut V1); }
                    6 => { m::it_80292F14(&raw mut G, &raw mut V1, rw()); }
                    7 => { m::it_80292FF0(&raw mut G); }
                    8 => { ret = m::itFflower_UnkMotion0_Anim(&raw mut G) as u32; }
                    9 => { m::itFflower_UnkMotion0_Phys(&raw mut G); }
                    10 => { ret = m::itFflower_UnkMotion0_Coll(&raw mut G) as u32; }
                    11 => { m::it_8029313C(&raw mut G); }
                    12 => { ret = m::itFflower_UnkMotion6_Anim(&raw mut G) as u32; }
                    13 => { m::itFflower_UnkMotion1_Phys(&raw mut G); }
                    14 => { ret = m::itFflower_UnkMotion1_Coll(&raw mut G) as u32; }
                    15 => { m::itFFlower_Logic25_PickedUp(&raw mut G); }
                    16 => { ret = m::itFflower_UnkMotion5_Anim(&raw mut G) as u32; }
                    17 => { m::itFflower_UnkMotion5_Phys(&raw mut G); }
                    18 => { m::itFFlower_Logic25_Dropped(&raw mut G); }
                    19 => { m::itFFlower_Logic25_Thrown(&raw mut G); }
                    20 => { m::itFflower_UnkMotion6_Phys(&raw mut G); }
                    21 => { ret = m::itFflower_UnkMotion6_Coll(&raw mut G) as u32; }
                    22 => { ret = m::itFFlower_Logic25_DmgDealt(&raw mut G) as u32; }
                    23 => { ret = m::itFFlower_Logic25_Clanked(&raw mut G) as u32; }
                    24 => { ret = m::itFFlower_Logic25_HitShield(&raw mut G) as u32; }
                    25 => { ret = m::itFFlower_Logic25_Reflected(&raw mut G) as u32; }
                    26 => { ret = m::itFFlower_Logic25_ShieldBounced(&raw mut G) as u32; }
                    27 => { m::itFFlower_Logic25_EnteredAir(&raw mut G); }
                    28 => { ret = m::itFflower_UnkMotion7_Anim(&raw mut G) as u32; }
                    29 => { m::itFflower_UnkMotion7_Phys(&raw mut G); }
                    30 => { ret = m::itFflower_UnkMotion7_Coll(&raw mut G) as u32; }
                    31 => { m::itFFlower_Logic25_EvtUnk(&raw mut G, &raw mut G2); }
                    32 => { { for i in 0..8 { hm(m::it_803F6090[i].anim_id as u32); hm(fi(m::it_803F6090[i].animated)); hm(fi(m::it_803F6090[i].physics_updated)); hm(fi(m::it_803F6090[i].collided)); } } }
                    _ => unreachable!(),
                }
                pre_hash();
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 4950);
    }
}
