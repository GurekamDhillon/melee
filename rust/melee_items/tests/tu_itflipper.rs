//! Written by gen/mtu/mk_it_driver.py from the same description as gen/mtu/tu/itflipper.c.
//! src/tu/itflipper.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu/run_it.py itflipper`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itflipper as m;
use mtu_harness::*;
#[path = "mtu/itflipper_gen.rs"]
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
    if rnd() % 4 != 0 { let q = rnd() % 2; IT.xC38 = (match q { 0 => 20, _ => 19 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 8; IT.xDD4_itemVar.flipper.xDD4_flightTimer = (match q { 0 => -1, 1 => 0, 2 => 1, 3 => 5, 4 => 6, 5 => 10, 6 => 11, _ => 12 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 2; IT.xDD4_itemVar.flipper.xDD8_isSettled = (match q { 0 => 0, _ => 1 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 2; IT.xDD4_itemVar.flipper.xDE8_isStageFixed = (match q { 0 => 0, _ => 1 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 3; IT.xDD4_itemVar.flipper.xDDC_hitboxTimer = (match q { 0 => 0, 1 => 1, _ => 2 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 8; IT.xDD4_itemVar.flipper.xDE4_spinVel = (match q { 0 => 0.0, 1 => 0.01, 2 => -0.01, 3 => 0.5, 4 => -0.5, 5 => 3.5, 6 => -3.5, _ => 7.0 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 4; IT.xD44_lifeTimer = (match q { 0 => 0.5, 1 => 1.0, 2 => 2.0, _ => 30.0 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 4; ICD.x34 = (match q { 0 => 0, 1 => 1, 2 => 2, _ => 30 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 4; SA[8] = (match q { 0 => 0.0, 1 => 0.1, 2 => 0.5, _ => 5.0 } as f32).to_bits(); }
    if rnd() % 4 != 0 { let q = rnd() % 3; SA[9] = (match q { 0 => 0.5, 1 => 3.0, _ => 10.0 } as f32).to_bits(); }
    if rnd() % 4 != 0 { let q = rnd() % 3; SA[7] = (match q { 0 => 0.0, 1 => 0.3, _ => 2.0 } as f32).to_bits(); }
    if rnd() % 4 != 0 { let q = rnd() % 3; SA[6] = (match q { 0 => 0.5, 1 => 1.0, _ => 2.0 } as f32).to_bits(); }
    IT.xDD4_itemVar.flipper.xDEC_stageAnchor = if rnd() & 1 != 0 { &raw mut JO[1] } else { null_mut() }; IT2.xDD4_itemVar.flipper.xDEC_stageAnchor = null_mut();
    IT.xCF4_fighterGObjUnk = if rnd() & 1 != 0 { &raw mut OWN } else { null_mut() }; IT.xCEC_fighterGObj = if rnd() & 1 != 0 { &raw mut OWN } else { null_mut() };

}

unsafe fn pre_hash() {
    {
        let its: [*mut Item; 2] = [&raw mut IT, &raw mut IT2];
        for k2 in 0..2 {
            let id = pi((*its[k2]).xDD4_itemVar.flipper.xDEC_stageAnchor);
            let b = &raw mut (*its[k2]).xDD4_itemVar.flipper.xDEC_stageAnchor as *mut u8;
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
fn itflipper_matches_c() {
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
        let mut c = Check::new(include_str!("data/mtu_itflipper.txt"));
        for op in 0..46u32 {
            for k in 0..150u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                    0 => { ret = pi(m::itFlipper_Spawn(&raw mut JO[1])); }
                    1 => { m::itFlipper_Spawned(&raw mut G); }
                    2 => { m::itFlipper_UpdateSpin(&raw mut G); }
                    3 => { m::itFlipper_AddSpinImpulse(&raw mut G, &raw mut V1, rw()); }
                    4 => { ret = m::itFlipper_SpinSpeedFromFighter(&raw mut G, &raw mut G2, &raw mut V1, &raw mut V1).to_bits(); }
                    5 => { m::itFlipper_SpinFromFighter_inline(&raw mut G, &raw mut V1, &raw mut V1, rw()); }
                    6 => { m::itFlipper_SpinFromFighter(&raw mut G); }
                    7 => { m::itFlipper_Repel_inline(&raw mut G, fv() as _, &raw mut V1, &raw mut V1); }
                    8 => { m::itFlipper_Repel(&raw mut G, fv() as _, &raw mut V1); }
                    9 => { m::itFlipper_EnterResting(&raw mut G); }
                    10 => { ret = m::itFlipper_Resting_Anim(&raw mut G) as u32; }
                    11 => { m::itFlipper_Resting_Phys(&raw mut G); }
                    12 => { ret = m::itFlipper_Resting_Coll(&raw mut G) as u32; }
                    13 => { m::itFlipper_EnterFalling(&raw mut G); }
                    14 => { ret = m::itFlipper_Falling_Anim(&raw mut G) as u32; }
                    15 => { m::itFlipper_Falling_Phys(&raw mut G); }
                    16 => { ret = m::itFlipper_Falling_Coll(&raw mut G) as u32; }
                    17 => { m::itFlipper_PickedUp(&raw mut G); }
                    18 => { ret = m::itFlipper_Held_Anim(&raw mut G) as u32; }
                    19 => { m::itFlipper_Held_Phys(&raw mut G); }
                    20 => { m::itFlipper_Dropped(&raw mut G); }
                    21 => { m::itFlipper_Thrown(&raw mut G); }
                    22 => { ret = m::itFlipper_Inflight_Anim(&raw mut G) as u32; }
                    23 => { m::itFlipper_Inflight_Phys(&raw mut G); }
                    24 => { ret = m::itFlipper_Inflight_Coll(&raw mut G) as u32; }
                    25 => { m::itFlipper_Settle(&raw mut G); }
                    26 => { m::itFlipper_EnterActive(&raw mut G); }
                    27 => { m::itFlipper_RefreshHitboxes(&raw mut G); }
                    28 => { ret = m::itFlipper_UpdateActive(&raw mut G) as u32; }
                    29 => { ret = m::itFlipper_Active_Anim(&raw mut G) as u32; }
                    30 => { ret = m::itFlipper_Spinning_Anim(&raw mut G) as u32; }
                    31 => { m::itFlipper_Spinning_Phys(&raw mut G); }
                    32 => { ret = m::itFlipper_Spinning_Coll(&raw mut G) as u32; }
                    33 => { ret = m::itFlipper_DmgDealt(&raw mut G) as u32; }
                    34 => { ret = m::itFlipper_Clanked(&raw mut G) as u32; }
                    35 => { ret = m::itFlipper_HitShield(&raw mut G) as u32; }
                    36 => { ret = m::itFlipper_Reflected(&raw mut G) as u32; }
                    37 => { ret = m::itFlipper_ShieldBounced(&raw mut G) as u32; }
                    38 => { m::itFlipper_SpinFromFighterRecv(&raw mut G, &raw mut V1, &raw mut V1, rw()); }
                    39 => { ret = m::itFlipper_DmgReceived(&raw mut G) as u32; }
                    40 => { m::itFlipper_EnteredAir(&raw mut G); }
                    41 => { ret = m::itFlipper_Airborne_Anim(&raw mut G) as u32; }
                    42 => { m::itFlipper_Airborne_Phys(&raw mut G); }
                    43 => { ret = m::itFlipper_Airborne_Coll(&raw mut G) as u32; }
                    44 => { m::itFlipper_EvtUnk(&raw mut G, &raw mut G2); }
                    45 => { { for i in 0..7 { hm(m::ItemStateTable_Flipper[i].anim_id as u32); hm(fi(m::ItemStateTable_Flipper[i].animated)); hm(fi(m::ItemStateTable_Flipper[i].physics_updated)); hm(fi(m::ItemStateTable_Flipper[i].collided)); } } }
                    _ => unreachable!(),
                }
                pre_hash();
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 6900);
    }
}
