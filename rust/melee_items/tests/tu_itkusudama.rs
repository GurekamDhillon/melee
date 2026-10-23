//! Written by gen/mtu_it/mk_it_driver.py from the same description as gen/mtu_it/tu/itkusudama.c.
//! src/tu/itkusudama.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu_it/run_it.py itkusudama`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itkusudama as m;
use mtu_harness::*;
#[path = "mtu/itkusudama_gen.rs"]
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
    m::ext::it_804D6D28 = &raw mut ICD;
    if rnd() % 4 != 0 { let q = rnd() % 2; IT.xDD4_itemVar.kusudama.x0 = (match q { 0 => 0, _ => 1 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 4; IT.xDD4_itemVar.kusudama.x4 = (match q { 0 => -1, 1 => 0, 2 => 1, _ => 5 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 3; IT.xDB4_itcmd_var2 = (match q { 0 => 0, 1 => 1, _ => 2 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 2; IT.xDB0_itcmd_var1 = (match q { 0 => 0, _ => 1 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 2; IT.xDAC_itcmd_var0 = (match q { 0 => 0, _ => 1 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 2; IT.facing_dir = (match q { 0 => 1.0, _ => -1.0 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 3; IT.xC9C = (match q { 0 => 0, 1 => 5, _ => 50 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 5; IT.x378_itemColl.env_flags = (match q { 0 => 0, 1 => 0x2000, 2 => 0x4000, 3 => 0x6000, _ => 1 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 4; IT.x40_vel.y = (match q { 0 => 0.0, 1 => -3.0, 2 => 3.0, _ => -0.5 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 6; JO[0].rotate.x = (match q { 0 => 0.0, 1 => 1.5, 2 => -1.5, 3 => 3.0, 4 => -3.0, _ => 0.1 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 6; JO[0].rotate.y = (match q { 0 => 0.0, 1 => 1.6, 2 => -1.6, 3 => 4.7, 4 => -4.7, _ => 2.5 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 2; IA.x10_fall_speed = (match q { 0 => 0.1, _ => 0.5 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 2; IA.x14_fall_speed_max = (match q { 0 => 0.3, _ => 2.0 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 3; SA[0] = (match q { 0 => 0, 1 => 1, _ => 5 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 3; SA[1] = (match q { 0 => 0, 1 => 1, _ => 5 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 3; SA[2] = (match q { 0 => 0, 1 => 1, _ => 5 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 3; SA[3] = (match q { 0 => 0, 1 => 1, _ => 5 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 4; SA[4] = (match q { 0 => 34, 1 => 18, 2 => 5, _ => 0 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 4; SA[5] = (match q { 0 => 0, 1 => 1, 2 => 3, _ => 15 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 1; SA[16] = (match q {  _ => 0 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 3; SA[6] = (match q { 0 => 0.0, 1 => 3.0, _ => -3.0 } as f32).to_bits(); }
    if rnd() % 4 != 0 { let q = rnd() % 3; SA[7] = (match q { 0 => 0.0, 1 => 30.0, _ => -30.0 } as f32).to_bits(); }
    if rnd() % 4 != 0 { let q = rnd() % 4; SA[10] = (match q { 0 => 0.0, 1 => 0.1, 2 => 0.5, _ => 3.0 } as f32).to_bits(); }
    if rnd() % 4 != 0 { let q = rnd() % 3; SA[11] = (match q { 0 => 0.0, 1 => 0.5, _ => 3.0 } as f32).to_bits(); }
    if rnd() % 4 != 0 { let q = rnd() % 3; SA[8] = (match q { 0 => 0.0, 1 => 5.0, _ => 50.0 } as f32).to_bits(); }
    if (SA[5] as i32) > 15 { SA[5] &= 15; }

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
fn itkusudama_matches_c() {
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
        let mut c = Check::new(include_str!("data/mtu_itkusudama.txt"));
        for op in 0..47u32 {
            for k in 0..150u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                    0 => { ret = pi(m::it_802896CC(&raw mut V1)); }
                    1 => { m::itKusudama_Logic4_Spawned(&raw mut G); }
                    2 => { m::it_802897C8(&raw mut G, rw()); }
                    3 => { m::it_80289910(&raw mut G, rw()); }
                    4 => { m::it_80289A00(&raw mut G); }
                    5 => { m::it_80289B50(&raw mut G, fv() as _); }
                    6 => { m::it_80289BE8(&raw mut G, (rnd() % 4) as i32, (rnd() % 4) as i32, (rnd() % 4) as i32); }
                    7 => { ret = m::it_8028A114(&raw mut G, fv() as _, fv() as _, fv() as _, fv() as _) as u32; }
                    8 => { m::it_8028A190(&raw mut G); }
                    9 => { ret = m::itKusudama_UnkMotion0_Anim(&raw mut G) as u32; }
                    10 => { m::itKusudama_UnkMotion0_Phys(&raw mut G); }
                    11 => { ret = m::itKusudama_UnkMotion0_Coll(&raw mut G) as u32; }
                    12 => { ret = m::itKusudama_UnkMotion1_Anim(&raw mut G) as u32; }
                    13 => { m::itKusudama_UnkMotion1_Phys(&raw mut G); }
                    14 => { ret = m::itKusudama_UnkMotion1_Coll(&raw mut G) as u32; }
                    15 => { m::it_8028A3CC(&raw mut G); }
                    16 => { ret = m::itKusudama_UnkMotion2_Anim(&raw mut G) as u32; }
                    17 => { m::itKusudama_UnkMotion2_Phys(&raw mut G); }
                    18 => { ret = m::itKusudama_UnkMotion2_Coll(&raw mut G) as u32; }
                    19 => { m::it_8028A544(&raw mut G); }
                    20 => { ret = m::itKusudama_UnkMotion3_Anim(&raw mut G) as u32; }
                    21 => { m::itKusudama_UnkMotion3_Phys(&raw mut G); }
                    22 => { ret = m::itKusudama_UnkMotion3_Coll(&raw mut G) as u32; }
                    23 => { m::itKusudama_Logic4_PickedUp(&raw mut G); }
                    24 => { ret = m::itKusudama_UnkMotion4_Anim(&raw mut G) as u32; }
                    25 => { m::itKusudama_UnkMotion4_Phys(&raw mut G); }
                    26 => { m::itKusudama_Logic4_Thrown(&raw mut G); }
                    27 => { ret = m::itKusudama_UnkMotion6_Anim(&raw mut G) as u32; }
                    28 => { m::itKusudama_UnkMotion6_Phys(&raw mut G); }
                    29 => { ret = m::itKusudama_UnkMotion5_Coll(&raw mut G) as u32; }
                    30 => { m::itKusudama_Logic4_Dropped(&raw mut G); }
                    31 => { ret = m::itKusudama_UnkMotion6_Coll(&raw mut G) as u32; }
                    32 => { m::it_8028AC74(&raw mut G); }
                    33 => { ret = m::itKusudama_UnkMotion7_Anim(&raw mut G) as u32; }
                    34 => { m::itKusudama_UnkMotion7_Phys(&raw mut G); }
                    35 => { ret = m::itKusudama_UnkMotion7_Coll(&raw mut G) as u32; }
                    36 => { m::it_8028AD44(&raw mut G); }
                    37 => { ret = m::itKusudama_UnkMotion8_Anim(&raw mut G) as u32; }
                    38 => { m::itKusudama_UnkMotion8_Phys(&raw mut G); }
                    39 => { ret = m::itKusudama_UnkMotion8_Coll(&raw mut G) as u32; }
                    40 => { ret = m::itKusudama_Logic4_DmgDealt(&raw mut G) as u32; }
                    41 => { ret = m::itKusudama_Logic4_Clanked(&raw mut G) as u32; }
                    42 => { ret = m::itKusudama_Logic4_HitShield(&raw mut G) as u32; }
                    43 => { ret = m::itKusudama_Logic4_Reflected(&raw mut G) as u32; }
                    44 => { ret = m::itKusudama_Logic4_DmgReceived(&raw mut G) as u32; }
                    45 => { m::itKusudama_Logic4_EvtUnk(&raw mut G, &raw mut G2); }
                    46 => { { for i in 0..9 { hm(m::it_803F59F8[i].anim_id as u32); hm(fi(m::it_803F59F8[i].animated)); hm(fi(m::it_803F59F8[i].physics_updated)); hm(fi(m::it_803F59F8[i].collided)); } } }
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
