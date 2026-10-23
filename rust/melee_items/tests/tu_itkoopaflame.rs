//! Written by gen/mtu_it/mk_it_driver.py from the same description as gen/mtu_it/tu/itkoopaflame.c.
//! src/tu/itkoopaflame.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu_it/run_it.py itkoopaflame`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itkoopaflame as m;
use mtu_harness::*;
#[path = "mtu/itkoopaflame_gen.rs"]
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
    if rnd() % 4 != 0 { let q = rnd() % 3; IT.kind = (match q { 0 => It_Kind_Koopa_Flame, 1 => It_Kind_Kirby_KoopaFlame, _ => 5 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 3; IT2.kind = (match q { 0 => It_Kind_Koopa_Flame, 1 => It_Kind_Kirby_KoopaFlame, _ => 5 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 9; IT.x378_itemColl.env_flags = (match q { 0 => 0, 1 => 1, 2 => 0x40, 3 => 0x2000, 4 => 0x8000, 5 => 0x8001, 6 => 0x2041, 7 => 0xA041, _ => 0x10000 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 2; IT.x5D4_hitboxes[0].hit.state = (match q { 0 => 0, _ => 1 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 3; IT.xDD4_itemVar.koopaflame.x34_base_scale = (match q { 0 => 0.0, 1 => 1.0, _ => 2.0 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 2; IT.xDD4_itemVar.koopaflame.x44_spawned = (match q { 0 => 0, _ => 1 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 5; IT.xDD4_itemVar.koopaflame.x40_frame_counter = (match q { 0 => 0, 1 => 10, 2 => 19, 3 => 20, _ => 21 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 7; IT.xDD4_itemVar.koopaflame.x24_angle = (match q { 0 => 0.0, 1 => 1.0, 2 => -1.0, 3 => 3.0, 4 => -3.0, 5 => 4.0, _ => 7.0 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 2; IT.facing_dir = (match q { 0 => 1.0, _ => -1.0 }) as _; }
    for i in 0..6 { SA[i] = rg().to_bits(); }
    if rnd() % 4 != 0 { let q = rnd() % 3; SA[1] = (match q { 0 => 20.0, 1 => 0.5, _ => 10.0 } as f32).to_bits(); }
    if !(IT.xDD4_itemVar.koopaflame.x24_angle > -50.0 && IT.xDD4_itemVar.koopaflame.x24_angle < 50.0) { IT.xDD4_itemVar.koopaflame.x24_angle = 0.5; }

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
fn itkoopaflame_matches_c() {
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
        let mut c = Check::new(include_str!("data/mtu_itkoopaflame.txt"));
        for op in 0..15u32 {
            for k in 0..150u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                    0 => { m::itKoopaFlame_Update_Direction(&raw mut G, (rnd() % 16) as i32); }
                    1 => { m::itKoopaFlame_Update_Angle(&raw mut G, (rnd() % 16) as i32); }
                    2 => { ret = pi(m::itKoopaFlame_Spawn(&raw mut OWN, &raw mut V1, rw(), fv(), fv() as i32, (fv() % 9) as i32, (fv() % 9) as i32, if fv() % 3 != 0 { 5 } else { 6 })); }
                    3 => { m::itKoopaFlame_Setup(&raw mut G, &raw mut G2, fv() as _); }
                    4 => { ret = m::itKoopaFlame_UnkMotion0_Anim(&raw mut G) as u32; }
                    5 => { m::itKoopaFlame_UnkMotion0_Phys(&raw mut G); }
                    6 => { ret = m::itKoopaFlame_UnkMotion0_Coll(&raw mut G) as u32; }
                    7 => { ret = m::itKoopaFlame_Logic111_DmgDealt(&raw mut G) as u32; }
                    8 => { ret = m::itKoopaFlame_Logic111_Reflected(&raw mut G) as u32; }
                    9 => { ret = m::itKoopaFlame_Logic111_Clanked(&raw mut G) as u32; }
                    10 => { ret = m::itKoopaFlame_Logic111_Absorbed(&raw mut G) as u32; }
                    11 => { ret = m::itKoopaFlame_Logic111_ShieldBounced(&raw mut G) as u32; }
                    12 => { ret = m::itKoopaFlame_Logic111_HitShield(&raw mut G) as u32; }
                    13 => { m::itKoopaFlame_Logic111_EvtUnk(&raw mut G, &raw mut G2); }
                    14 => { { for i in 0..1 { hm(m::ItemStateTable_KoopaFlame[i].anim_id as u32); hm(fi(m::ItemStateTable_KoopaFlame[i].animated)); hm(fi(m::ItemStateTable_KoopaFlame[i].physics_updated)); hm(fi(m::ItemStateTable_KoopaFlame[i].collided)); } } }
                    _ => unreachable!(),
                }
                pre_hash();
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 2250);
    }
}
