//! Written by gen/mtu_it2/mk_it_driver.py from the same description as gen/mtu_it2/tu/itlinkboomerang.c.
//! src/tu/itlinkboomerang.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu_it2/run_it.py itlinkboomerang`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "mtu_harness_it/mod.rs"]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itlinkboomerang as m;
use mtu_harness::*;
#[path = "mtu/itlinkboomerang_gen.rs"]
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

static mut LT: ItemLogicTable = unsafe { core::mem::zeroed() };
static mut ISA: ItemStateArray = unsafe { core::mem::zeroed() };
unsafe extern "C" fn lt_thrown(g: *mut HSD_GObj) { t(0xBEEF); t(pi(g)); }

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
    if rnd() % 4 != 0 { let q = rnd() % 3; IT.xDAC_itcmd_var0 = (match q { 0 => 0, 1 => 1, _ => 2 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 3; IT.xDB0_itcmd_var1 = (match q { 0 => 0, 1 => 1, _ => 2 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 3; IT.xDB4_itcmd_var2 = (match q { 0 => 0, 1 => 1, _ => 2 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 2; IT.xDD4_itemVar.linkboomerang.xDE8 = (match q { 0 => 0, _ => 1 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 2; IT.xDD4_itemVar.linkboomerang.xDEC = (match q { 0 => 6, _ => 1 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 2; IT.xDD4_itemVar.linkboomerang.xDE4 = (match q { 0 => 0, _ => 3 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 3; IT.xDD4_itemVar.linkboomerang.xF70 = (match q { 0 => 0.0, 1 => 1.0, _ => 5.0 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 4; IT.xDD4_itemVar.linkboomerang.xF8C = (match q { 0 => 0.0, 1 => 1.0, 2 => -2.0, _ => 5.0 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 3; IT.xDD4_itemVar.linkboomerang.xF80 = (match q { 0 => 0.0, 1 => 1.0, _ => 3.0 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 3; IT.xDD4_itemVar.linkboomerang.xF84 = (match q { 0 => 0.1, 1 => 0.5, _ => 1.0 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 7; IT.xDD4_itemVar.linkboomerang.xF74 = (match q { 0 => 0.0, 1 => 1.0, 2 => 3.0, 3 => -3.0, 4 => 7.0, 5 => -7.0, _ => 0.5 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 4; IT.xD44_lifeTimer = (match q { 0 => 0.0, 1 => 1.0, 2 => -1.0, _ => 30.0 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 2; IT.facing_dir = (match q { 0 => 1.0, _ => -1.0 }) as _; }
    if rnd() % 4 != 0 { let q = rnd() % 5; IT.x378_itemColl.env_flags = (match q { 0 => 0, 1 => 0xFC0, 2 => 0x3F, 3 => 0x6000, _ => 0x18000 }) as _; }
    LT = core::mem::zeroed(); ISA = core::mem::zeroed(); LT.thrown = Some(lt_thrown); IT.xB8_itemLogicTable = &raw mut LT; IT2.xB8_itemLogicTable = &raw mut LT; ART.xC_itemStates = &raw mut ISA;
    IT.xDD4_itemVar.linkboomerang.xDDC[0] = (rnd() % 16) as i32; IT.xDD4_itemVar.linkboomerang.xDDC[1] = (rnd() % 16) as i32; IT.xDD4_itemVar.linkboomerang.xDD8 = (rnd() % 16) as i32;
    IT2.xDD4_itemVar.linkboomerang.xDDC[0] = (rnd() % 16) as i32; IT2.xDD4_itemVar.linkboomerang.xDDC[1] = (rnd() % 16) as i32; IT2.xDD4_itemVar.linkboomerang.xDD8 = (rnd() % 16) as i32;
    IT.xDD4_itemVar.linkboomerang.xF90[0] = if rnd() & 1 != 0 { &raw mut JO[1] } else { null_mut() }; IT.xDD4_itemVar.linkboomerang.xF90[1] = if rnd() & 1 != 0 { &raw mut JO[2] } else { null_mut() };
    IT2.xDD4_itemVar.linkboomerang.xF90[0] = if rnd() & 1 != 0 { &raw mut JO[1] } else { null_mut() }; IT2.xDD4_itemVar.linkboomerang.xF90[1] = if rnd() & 1 != 0 { &raw mut JO[2] } else { null_mut() };
    { let q = rnd() % 4; IT.xDD4_itemVar.linkboomerang.xF98 = if q == 0 { null_mut() } else if q == 1 { &raw mut OWN } else if q == 2 { &raw mut G2 } else { IT.owner }; }
    IT2.xDD4_itemVar.linkboomerang.xF98 = null_mut();

}

unsafe fn pre_hash() {
    { let id = pi(IT.xDD4_itemVar.linkboomerang.xF90[0]); let b = &raw mut IT.xDD4_itemVar.linkboomerang.xF90[0] as *mut u8; for i2 in 0..8 { *b.add(i2) = 0; } *(b as *mut u32) = id; }
    { let id = pi(IT.xDD4_itemVar.linkboomerang.xF90[1]); let b = &raw mut IT.xDD4_itemVar.linkboomerang.xF90[1] as *mut u8; for i2 in 0..8 { *b.add(i2) = 0; } *(b as *mut u32) = id; }
    { let id = pi(IT.xDD4_itemVar.linkboomerang.xF98); let b = &raw mut IT.xDD4_itemVar.linkboomerang.xF98 as *mut u8; for i2 in 0..8 { *b.add(i2) = 0; } *(b as *mut u32) = id; }
    { let id = pi(IT2.xDD4_itemVar.linkboomerang.xF90[0]); let b = &raw mut IT2.xDD4_itemVar.linkboomerang.xF90[0] as *mut u8; for i2 in 0..8 { *b.add(i2) = 0; } *(b as *mut u32) = id; }
    { let id = pi(IT2.xDD4_itemVar.linkboomerang.xF90[1]); let b = &raw mut IT2.xDD4_itemVar.linkboomerang.xF90[1] as *mut u8; for i2 in 0..8 { *b.add(i2) = 0; } *(b as *mut u32) = id; }
    { let id = pi(IT2.xDD4_itemVar.linkboomerang.xF98); let b = &raw mut IT2.xDD4_itemVar.linkboomerang.xF98 as *mut u8; for i2 in 0..8 { *b.add(i2) = 0; } *(b as *mut u32) = id; }

}

unsafe fn hash_all() {
    g::hash_HSD_GObj(&G); g::hash_HSD_GObj(&G2); g::hash_HSD_GObj(&OWN);
    g::hash_Item(&IT); g::hash_Item(&IT2); g::hash_ItemAttr(&IA);
    for i in 0..4 { hash_jo(&JO[i]); }
    for i in 0..256 { hm(SA[i]); }
    hm(V1.x.to_bits()); hm(V1.y.to_bits()); hm(V1.z.to_bits()); hm(V2.x.to_bits()); hm(V2.y.to_bits()); hm(V2.z.to_bits());

}

#[test]
fn itlinkboomerang_matches_c() {
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
        freg(m::norm_xF74 as usize);
        freg(m::norm_xF74_from_angle as usize);
        freg(m::remove_boomerang as usize);
        freg(m::it_8029FDBC as usize);
        freg(m::it_8029FDDC as usize);
        freg(m::it_8029FE64 as usize);
        freg(m::it_802A013C_inline as usize);
        freg(m::it_802A013C_LoadAnim as usize);
        freg(m::it_802A013C as usize);
        freg(m::loop_lb_8000BA0C_gobj as usize);
        freg(m::loop_lb_8000BA0C_gobj_1 as usize);
        freg(m::it_802A0534 as usize);
        freg(m::it_802A07B4 as usize);
        freg(m::it_802A0810 as usize);
        freg(m::itLinkBoomerang_Logic18_Destroyed as usize);
        freg(m::it_802A0930 as usize);
        freg(m::it_8029FE64_no_inline as usize);
        freg(m::it_802A0C34_sub_1 as usize);
        freg(m::it_802A0C34_sub_2 as usize);
        freg(m::it_802A0C34 as usize);
        freg(m::it_802A0E70 as usize);
        freg(m::itLinkboomerang_UnkMotion0_Anim as usize);
        freg(m::it_802A0F84 as usize);
        freg(m::it_802A0F88 as usize);
        freg(m::itLinkboomerang_UnkMotion1_Anim as usize);
        freg(m::itLinkboomerang_get_x18 as usize);
        freg(m::itLinkboomerang_UnkMotion1_Phys as usize);
        freg(m::itLinkboomerang_UnkMotion1_Coll as usize);
        freg(m::it_802A10E4 as usize);
        freg(m::itLinkboomerang_UnkMotion2_Anim as usize);
        freg(m::itLinkboomerang_UnkMotion2_Phys as usize);
        freg(m::clamp_tau as usize);
        freg(m::clamp_pi_tau as usize);
        freg(m::it_802A13EC_inline as usize);
        freg(m::it_802A13EC as usize);
        freg(m::it_802A15EC as usize);
        freg(m::it_802A15EC_no_inline as usize);
        freg(m::itLinkboomerang_UnkMotion2_Coll as usize);
        freg(m::it_802A19E0_no_inline as usize);
        freg(m::clamp_angle_pi as usize);
        freg(m::it_802A1948 as usize);
        freg(m::it_802A19E0 as usize);
        freg(m::itLinkboomerang_UnkMotion3_Anim as usize);
        freg(m::itLinkboomerang_UnkMotion3_Phys_sub as usize);
        freg(m::itLinkboomerang_UnkMotion3_Phys_add_clamp as usize);
        freg(m::itLinkboomerang_UnkMotion3_Phys as usize);
        freg(m::it_802A1F08 as usize);
        freg(m::it_802A1FA8 as usize);
        freg(m::itLinkBoomerang_Logic18_Absorbed as usize);
        freg(m::it_802A20E8_inline as usize);
        freg(m::it_802A20E8 as usize);
        freg(m::it_802A2288 as usize);
        freg(m::it_802A2320 as usize);
        freg(m::it_802A23CC as usize);
        let mut c = Check::new(include_str!("data/mtu_itlinkboomerang.txt"));
        for op in 0..55u32 {
            for k in 0..150u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                    0 => { m::norm_xF74(&raw mut IT); }
                    1 => { m::norm_xF74_from_angle(&raw mut IT, rw()); }
                    2 => { m::remove_boomerang(&raw mut G); }
                    3 => { ret = m::it_8029FDBC(&raw mut G) as u32; }
                    4 => { m::it_8029FDDC(&raw mut G, fv() as _); }
                    5 => { m::it_8029FE64(&raw mut G, (fv() % 2) as i32); }
                    6 => { m::it_802A013C_inline(&raw mut G); }
                    7 => { ret = pi(m::it_802A013C_LoadAnim(SA.as_mut_ptr() as *mut itLinkBoomerangAttributes, (fv() % 2) as i32)); }
                    8 => { ret = pi(m::it_802A013C(rw(), &raw mut G, &raw mut V1, fv() as _, fv() as _)); }
                    9 => { m::loop_lb_8000BA0C_gobj(&raw mut G, &raw mut JO[1], SA.as_mut_ptr() as *mut itLinkBoomerangAttributes); }
                    10 => { m::loop_lb_8000BA0C_gobj_1(&raw mut G, &raw mut JO[1]); }
                    11 => { m::it_802A0534(&raw mut G, &raw mut V1, rw()); }
                    12 => { m::it_802A07B4(&raw mut G); }
                    13 => { m::it_802A0810(&raw mut G); }
                    14 => { m::itLinkBoomerang_Logic18_Destroyed(&raw mut G); }
                    15 => { m::it_802A0930(&raw mut G); }
                    16 => { m::it_8029FE64_no_inline(&raw mut G, (fv() % 2) as i32); }
                    17 => { m::it_802A0C34_sub_1(&raw mut G); }
                    18 => { m::it_802A0C34_sub_2(&raw mut G, &raw mut V1); }
                    19 => { ret = m::it_802A0C34(&raw mut G) as u32; }
                    20 => { m::it_802A0E70(&raw mut G); }
                    21 => { ret = m::itLinkboomerang_UnkMotion0_Anim(&raw mut G) as u32; }
                    22 => { m::it_802A0F84(&raw mut G); }
                    23 => { m::it_802A0F88(&raw mut G); }
                    24 => { ret = m::itLinkboomerang_UnkMotion1_Anim(&raw mut G) as u32; }
                    25 => { ret = m::itLinkboomerang_get_x18(SA.as_mut_ptr() as *mut itLinkBoomerangAttributes).to_bits(); }
                    26 => { m::itLinkboomerang_UnkMotion1_Phys(&raw mut G); }
                    27 => { ret = m::itLinkboomerang_UnkMotion1_Coll(&raw mut G) as u32; }
                    28 => { m::it_802A10E4(&raw mut G); }
                    29 => { ret = m::itLinkboomerang_UnkMotion2_Anim(&raw mut G) as u32; }
                    30 => { m::itLinkboomerang_UnkMotion2_Phys(&raw mut G); }
                    31 => { { let mut a: f32 = ((rnd() % 2001) as i32 - 1000) as f32 / 100.0; m::clamp_tau(&mut a); ret = a.to_bits(); } }
                    32 => { { let mut a: f32 = ((rnd() % 2001) as i32 - 1000) as f32 / 100.0; m::clamp_pi_tau(&mut a); ret = a.to_bits(); } }
                    33 => { ret = m::it_802A13EC_inline(&raw mut IT).to_bits(); }
                    34 => { ret = m::it_802A13EC(&raw mut G).to_bits(); }
                    35 => { m::it_802A15EC(&raw mut G, &raw mut V1); }
                    36 => { m::it_802A15EC_no_inline(&raw mut G, &raw mut V1); }
                    37 => { ret = m::itLinkboomerang_UnkMotion2_Coll(&raw mut G) as u32; }
                    38 => { m::it_802A19E0_no_inline(&raw mut G); }
                    39 => { { let mut a: f32 = ((rnd() % 2001) as i32 - 1000) as f32 / 100.0; m::clamp_angle_pi(&mut a); ret = a.to_bits(); } }
                    40 => { m::it_802A1948(&raw mut G, fv() as _); }
                    41 => { m::it_802A19E0(&raw mut G); }
                    42 => { ret = m::itLinkboomerang_UnkMotion3_Anim(&raw mut G) as u32; }
                    43 => { m::itLinkboomerang_UnkMotion3_Phys_sub(&raw mut G, rw()); }
                    44 => { ret = m::itLinkboomerang_UnkMotion3_Phys_add_clamp(rw(), rw(), rw()).to_bits(); }
                    45 => { m::itLinkboomerang_UnkMotion3_Phys(&raw mut G); }
                    46 => { ret = m::it_802A1F08(&raw mut G) as u32; }
                    47 => { ret = m::it_802A1FA8(&raw mut G) as u32; }
                    48 => { ret = m::itLinkBoomerang_Logic18_Absorbed(&raw mut G) as u32; }
                    49 => { ret = m::it_802A20E8_inline(&raw mut IT).to_bits(); }
                    50 => { ret = m::it_802A20E8(&raw mut G) as u32; }
                    51 => { ret = m::it_802A2288(&raw mut G) as u32; }
                    52 => { ret = m::it_802A2320(&raw mut G) as u32; }
                    53 => { m::it_802A23CC(&raw mut G, &raw mut G2); }
                    54 => { { for i in 0..4 { hm(m::it_803F6920[i].anim_id as u32); hm(fi(m::it_803F6920[i].animated)); hm(fi(m::it_803F6920[i].physics_updated)); hm(fi(m::it_803F6920[i].collided)); } } }
                    _ => unreachable!(),
                }
                pre_hash();
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 8250);
    }
}
