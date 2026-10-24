//! Written by gen/mtu_it2/mk_it_driver.py from the same description as gen/mtu_it2/tu/itlugia.c.
//! src/tu/itlugia.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu_it2/run_it.py itlugia`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "mtu_harness_it/mod.rs"]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itlugia as m;
use mtu_harness::*;
#[path = "mtu/itlugia_gen.rs"]
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
    if rnd() % 4 != 0 { let q = rnd() % 6; IT.msid = (match q { 0 => 0, 1 => 1, 2 => 2, 3 => 3, 4 => 4, _ => 5 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 4; IT.xDD4_itemVar.lugia.x60 = (match q { 0 => -1, 1 => 0, 2 => 1, _ => 3 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 4; IT.xDD4_itemVar.lugia.xE50.y = (match q { 0 => 0.0, 1 => 0.5, 2 => 1.0, _ => 1.5 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 4; IT.xDD4_itemVar.lugia.xE50.z = (match q { 0 => 0.0, 1 => 0.5, 2 => 1.0, _ => 1.5 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 4; IT.xDD4_itemVar.lugia.x88 = (match q { 0 => 0.0, 1 => 0.5, 2 => 1.0, _ => 1.5 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 3; IT.xDD4_itemVar.lugia.xA4 = (match q { 0 => 0, 1 => 90, _ => -45 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 2; IT.xDAC_itcmd_var0 = (match q { 0 => 0, _ => 1 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 4; IT.xD44_lifeTimer = (match q { 0 => 0.0, 1 => 1.0, 2 => -1.0, _ => 30.0 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 4; IT.kind = (match q { 0 => It_Kind_Lugia_Aeroblast, 1 => It_Kind_Lugia_Aeroblast2, 2 => It_Kind_Lugia_Aeroblast3, _ => 0 }) as _; }
    { let a = SA.as_mut_ptr() as *mut itLugiaAttributes; (*a).x34 = if rnd() & 1 != 0 { 40.0 } else { 90.0 }; (*a).x38 = if rnd() & 1 != 0 { 10.0 } else { 30.0 }; (*a).x3C = if rnd() & 1 != 0 { 0.5 } else { 1.0 }; { let q = rnd() % 3; (*a).x40 = if q == 0 { 1.0 } else if q == 1 { 2.0 } else { 1e9 }; }
      if rnd() & 1 != 0 { IT.xDD4_itemVar.lugia.x64.x = 1.0; IT.xDD4_itemVar.lugia.x64.y = 2.0; IT.xDD4_itemVar.lugia.x64.z = 0.0; } IT.xDD4_itemVar.lugia.x8C = IT.xDD4_itemVar.lugia.x64; }

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
fn itlugia_matches_c() {
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
        freg(m::my_sqrtf as usize);
        freg(m::my_sqrtf_accurate as usize);
        freg(m::it_2725_Logic17_Spawned as usize);
        freg(m::it_802D14D0 as usize);
        freg(m::itLugia_Logic17_EvtUnk as usize);
        freg(m::itLugia_UnkMotion1_Anim as usize);
        freg(m::itLugia_UnkMotion1_Phys as usize);
        freg(m::itLugia_UnkMotion1_Coll as usize);
        freg(m::it_802D1580 as usize);
        freg(m::itLugia_UnkMotion2_Anim as usize);
        freg(m::itLugia_UnkMotion2_Phys as usize);
        freg(m::itLugia_UnkMotion2_Coll as usize);
        freg(m::it_802D16D4 as usize);
        freg(m::itLugia_UnkMotion3_Anim as usize);
        freg(m::itLugia_UnkMotion3_Phys as usize);
        freg(m::itLugia_UnkMotion3_Coll as usize);
        freg(m::it_802D1830 as usize);
        freg(m::itLugia_UnkMotion4_Anim as usize);
        freg(m::itLugia_UnkMotion4_Phys as usize);
        freg(m::itLugia_UnkMotion4_Coll as usize);
        freg(m::it_802D1A44 as usize);
        freg(m::itLugia_UnkMotion5_Anim as usize);
        freg(m::itLugia_UnkMotion5_Phys as usize);
        freg(m::itLugia_UnkMotion5_Coll as usize);
        freg(m::it_802D1BBC as usize);
        freg(m::it_802D1D40 as usize);
        freg(m::it_802D1DB4 as usize);
        freg(m::it_802D1DD8 as usize);
        freg(m::it_802D1E64 as usize);
        freg(m::it_802D1E8C as usize);
        freg(m::it_802D208C_inline as usize);
        freg(m::it_802D1F64 as usize);
        freg(m::it_802D208C as usize);
        freg(m::itLugia_Logic39_Spawned as usize);
        freg(m::itLugia_Logic40_Spawned as usize);
        freg(m::itLugia_Logic41_Spawned as usize);
        freg(m::it_802D23D4 as usize);
        freg(m::it_802D23F4 as usize);
        freg(m::it_802D246C as usize);
        freg(m::it_802D24A0 as usize);
        freg(m::it_802D24FC as usize);
        let mut c = Check::new(include_str!("data/mtu_itlugia.txt"));
        for op in 0..43u32 {
            for k in 0..150u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                    0 => { ret = m::my_sqrtf(rw()).to_bits(); }
                    1 => { ret = m::my_sqrtf_accurate(rw()).to_bits(); }
                    2 => { m::it_2725_Logic17_Spawned(&raw mut G); }
                    3 => { m::it_802D14D0(); }
                    4 => { m::itLugia_Logic17_EvtUnk(&raw mut G, &raw mut G2); }
                    5 => { ret = m::itLugia_UnkMotion1_Anim(&raw mut G) as u32; }
                    6 => { m::itLugia_UnkMotion1_Phys(&raw mut G); }
                    7 => { ret = m::itLugia_UnkMotion1_Coll(&raw mut G) as u32; }
                    8 => { m::it_802D1580(&raw mut G); }
                    9 => { ret = m::itLugia_UnkMotion2_Anim(&raw mut G) as u32; }
                    10 => { m::itLugia_UnkMotion2_Phys(&raw mut G); }
                    11 => { ret = m::itLugia_UnkMotion2_Coll(&raw mut G) as u32; }
                    12 => { m::it_802D16D4(&raw mut G); }
                    13 => { ret = m::itLugia_UnkMotion3_Anim(&raw mut G) as u32; }
                    14 => { m::itLugia_UnkMotion3_Phys(&raw mut G); }
                    15 => { ret = m::itLugia_UnkMotion3_Coll(&raw mut G) as u32; }
                    16 => { m::it_802D1830(&raw mut G); }
                    17 => { ret = m::itLugia_UnkMotion4_Anim(&raw mut G) as u32; }
                    18 => { m::itLugia_UnkMotion4_Phys(&raw mut G); }
                    19 => { ret = m::itLugia_UnkMotion4_Coll(&raw mut G) as u32; }
                    20 => { m::it_802D1A44(&raw mut G); }
                    21 => { ret = m::itLugia_UnkMotion5_Anim(&raw mut G) as u32; }
                    22 => { m::itLugia_UnkMotion5_Phys(&raw mut G); }
                    23 => { ret = m::itLugia_UnkMotion5_Coll(&raw mut G) as u32; }
                    24 => { m::it_802D1BBC(&raw mut G); }
                    25 => { m::it_802D1D40(&raw mut G); }
                    26 => { ret = m::it_802D1DB4(&raw mut G) as u32; }
                    27 => { m::it_802D1DD8(&raw mut G); }
                    28 => { ret = m::it_802D1E64(&raw mut G) as u32; }
                    29 => { ret = pi(m::it_802D1E8C(&raw mut G, fv() as _, rw())); }
                    30 => { ret = m::it_802D208C_inline(rw(), rw(), rw()).to_bits(); }
                    31 => { { let v = m::it_802D1F64(&raw mut G, rw()); ret = v.x.to_bits(); hm(v.y.to_bits()); hm(v.z.to_bits()); } }
                    32 => { m::it_802D208C(&raw mut G); }
                    33 => { m::itLugia_Logic39_Spawned(&raw mut G); }
                    34 => { m::itLugia_Logic40_Spawned(&raw mut G); }
                    35 => { m::itLugia_Logic41_Spawned(&raw mut G); }
                    36 => { m::it_802D23D4(&raw mut G, &raw mut G2); }
                    37 => { m::it_802D23F4(&raw mut G); }
                    38 => { ret = m::it_802D246C(&raw mut G) as u32; }
                    39 => { m::it_802D24A0(&raw mut G); }
                    40 => { ret = m::it_802D24FC(&raw mut G) as u32; }
                    41 => { { for i in 0..6 { hm(m::it_803F7EE8[i].anim_id as u32); hm(fi(m::it_803F7EE8[i].animated)); hm(fi(m::it_803F7EE8[i].physics_updated)); hm(fi(m::it_803F7EE8[i].collided)); } } }
                    42 => { { for i in 0..1 { hm(m::it_803F7F48[i].anim_id as u32); hm(fi(m::it_803F7F48[i].animated)); hm(fi(m::it_803F7F48[i].physics_updated)); hm(fi(m::it_803F7F48[i].collided)); } } }
                    _ => unreachable!(),
                }
                pre_hash();
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 6450);
    }
}
