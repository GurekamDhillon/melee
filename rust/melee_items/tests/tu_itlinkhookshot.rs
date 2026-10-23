//! Written by gen/mtu_it2/mk_it_driver.py from the same description as gen/mtu_it2/tu/itlinkhookshot.c.
//! src/tu/itlinkhookshot.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/mtu_it2/run_it.py itlinkhookshot`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "mtu_harness_it/mod.rs"]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itlinkhookshot as m;
use mtu_harness::*;
#[path = "mtu/itlinkhookshot_gen.rs"]
mod g;

static mut G: HSD_GObj = unsafe { core::mem::zeroed() };
static mut G2: HSD_GObj = unsafe { core::mem::zeroed() };
static mut OWN: HSD_GObj = unsafe { core::mem::zeroed() };
static mut IT: Item = unsafe { core::mem::zeroed() };
static mut IT2: Item = unsafe { core::mem::zeroed() };
static mut ART: Article = unsafe { core::mem::zeroed() };
static mut IA: ItemAttr = unsafe { core::mem::zeroed() };
#[repr(C, align(16))]
struct SaBuf([u32; 256]);
impl core::ops::Deref for SaBuf { type Target = [u32; 256]; fn deref(&self) -> &[u32; 256] { &self.0 } }
impl core::ops::DerefMut for SaBuf { fn deref_mut(&mut self) -> &mut [u32; 256] { &mut self.0 } }
static mut SA: SaBuf = SaBuf([0; 256]);
static mut JO: [HSD_JObj; 4] = unsafe { core::mem::zeroed() };
static mut DBT: DynamicBoneTable = unsafe { core::mem::zeroed() };
static mut V1: Vec = unsafe { core::mem::zeroed() };
static mut V2: Vec = unsafe { core::mem::zeroed() };
static mut ICD: ItemCommonData = unsafe { core::mem::zeroed() };

static mut LK: [ItemLink; 16] = unsafe { core::mem::zeroed() };
static mut LG: [HSD_GObj; 16] = unsafe { core::mem::zeroed() };
static mut FTR: Fighter = unsafe { core::mem::zeroed() };
static mut FPB: [FighterBone; 160] = unsafe { core::mem::zeroed() };
static mut FTD: ftData = unsafe { core::mem::zeroed() };
static mut LKA: ftLk_DatAttrs = unsafe { core::mem::zeroed() };
static mut MPU: mp_UnkStruct0 = unsafe { core::mem::zeroed() };
static mut MTXB: [[f32; 4]; 3] = [[0.0; 4]; 3];
static mut MF: f32 = 0.0;
static mut LTAIL: *mut ItemLink = null_mut();
static mut LHEAD: *mut ItemLink = null_mut();
static mut LP: *mut ItemLink = null_mut();
static mut ACNT: usize = 0;
static mut GCNT: usize = 0;

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


unsafe fn fill_ftr() {
    FTR = core::mem::zeroed(); FPB = core::mem::zeroed(); FTD = core::mem::zeroed(); LKA = core::mem::zeroed();
    let q = rnd() % 6; FTR.motion_id = match q { 0 => 0x168, 1 => 0x169, 2 => 0xD4, 3 => 0xD6, 4 => 0x100, _ => 0x168 } as _;
    FTR.kind = (if rnd() & 1 != 0 { 20 } else { 6 }) as _;
    FTR.ground_or_air = (if rnd() & 1 != 0 { 1 } else { 0 }) as _;
    FTR.facing_dir = if rnd() & 1 != 0 { 1.0 } else { -1.0 };
    { let q3 = rnd() % 3; FTR.x34_scale.y = if q3 == 0 { 0.5 } else if q3 == 1 { 1.0 } else { 2.0 }; }
    FTR.cur_pos.x = rw(); FTR.cur_pos.y = rw(); FTR.cur_pos.z = rw();
    FTR.self_vel.x = rw(); FTR.self_vel.y = rw(); FTR.pos_delta.x = rw(); FTR.pos_delta.y = rw();
    FTR.input.pressed_buttons = if rnd() & 1 != 0 { 0x100 } else { rnd() & 0xFF };
    LKA.x88 = (rnd() % 4) as _; LKA.x98 = (rnd() % 4) as _; LKA.xA8 = (rnd() % 4) as _; LKA.xB4 = rw(); LKA.xB8 = (rnd() % 4) as _;
    if rnd() & 1 != 0 { let q = rnd() % 5; FTR.mv.ca.specials.grav = match q { 0 => 0.0, 1 => 1.0, 2 => 2.0, 3 => 3.0, _ => rw() }; }
    else { let q = rnd() % 4; FTR.mv.lk.specialn.x0.y = match q { 0 => 0.0, 1 => 1.0, 2 => -1.0, _ => 5.0 }; }
    FTR.u.lk.xC = if rnd() & 1 != 0 { &raw mut G } else { null_mut() };
    FTR.accessory2_cb = if rnd() & 1 != 0 { Some(m::it_802A7AAC) } else { None };
    FTR.death1_cb = if rnd() & 1 != 0 { Some(m::it_802A7AF0) } else { None };
    FTR.accessory3_cb = if rnd() & 1 != 0 { Some(m::it_802A7B34) } else { None };
    FTR.ft_data = &raw mut FTD; FTD.ext_attr = (&raw mut LKA) as *mut c_void; FTR.dat_attrs = (&raw mut LKA) as *mut _; FTR.parts = FPB.as_mut_ptr();
    for i in 0..160usize { FPB[i].joint = &raw mut JO[i & 3]; }
}
unsafe fn hash_ftr() {
    hm(FTR.motion_id as u32); hm(FTR.kind as u32); hm(FTR.ground_or_air as u32); hm(FTR.facing_dir.to_bits()); hm(FTR.x34_scale.y.to_bits());
    hm(FTR.cur_pos.x.to_bits()); hm(FTR.cur_pos.y.to_bits()); hm(FTR.cur_pos.z.to_bits());
    hm(FTR.self_vel.x.to_bits()); hm(FTR.self_vel.y.to_bits()); hm(FTR.pos_delta.x.to_bits()); hm(FTR.pos_delta.y.to_bits());
    hm(FTR.input.pressed_buttons as u32); hm(FTR.mv.ca.specials.grav.to_bits()); hm(FTR.mv.lk.specialn.x0.y.to_bits());
    hm(pi(FTR.u.lk.xC)); hm(fi(FTR.accessory2_cb)); hm(fi(FTR.death1_cb)); hm(fi(FTR.accessory3_cb));
    hm(pi(FPB[139].joint));
}


unsafe fn setup() {
    let mut i: usize;
    G = core::mem::zeroed(); G2 = core::mem::zeroed(); OWN = core::mem::zeroed(); IT = core::mem::zeroed(); IT2 = core::mem::zeroed();
    ART = core::mem::zeroed(); IA = core::mem::zeroed(); SA = SaBuf([0; 256]); JO = core::mem::zeroed(); DBT = core::mem::zeroed();
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
    if rnd() % 4 != 0 { let q = rnd() % 2; IT.xDD4_itemVar.linkhookshot.x14 = (match q { 0 => 0, _ => 1 }) as _; }
    { let A = SA.as_mut_ptr() as *mut itLinkHookshotAttributes;
      (*A).x0 = rw();
      (*A).x4 = rw();
      (*A).x8 = (rnd() % 5) as f32 / 4.0;
      (*A).xC = (2 + rnd() % 3) as i32;
      (*A).x10 = 0.5 + (rnd() % 4) as f32 * 0.5;
      (*A).x14 = (rnd() % 8) as f32 * 0.25;
      (*A).x18 = (rnd() % 8) as f32 * 0.25;
      (*A).x1C = (rnd() % 8) as f32 * 0.25;
      (*A).x20 = (rnd() % 8) as f32 * 0.25;
      (*A).x24 = (rnd() % 8) as f32 * 0.25;
      (*A).x28 = (rnd() % 8) as f32 * 0.25;
      (*A).x2C = (2 + rnd() % 4) as i32;
      (*A).x30 = 0.5 + (rnd() % 4) as f32 * 0.5;
      (*A).x34 = (rnd() % 8) as f32 * 0.25;
      (*A).x38 = (rnd() % 8) as f32 * 0.25;
      (*A).x3C = (rnd() % 4) as f32 * 0.05;
      (*A).x40 = (rnd() % 6) as f32 * 0.5;
      (*A).x44 = (rnd() % 6) as f32 * 0.5;
      (*A).x48 = (rnd() % 4) as f32 * 0.1;
      (*A).x4C = 1.0 + (rnd() % 3) as f32 * 0.5;
      (*A).x50 = 1.0 + (rnd() % 3) as f32 * 0.5;
    }
    fill_ftr();
    ACNT = 0; GCNT = 0; LP = null_mut();
    for i in 0..4usize { for r in 0..3usize { for c in 0..4usize { JO[i].mtx[r][c] = rw(); } } }
    for r in 0..3usize { for c in 0..4usize { MTXB[r][c] = rw(); } }
    MF = rw();
    MPU.x8.x = rw(); MPU.x8.y = rw(); MPU.x8.z = rw(); MPU.x14.x = rw(); MPU.x14.y = rw(); MPU.x14.z = rw();
    {
        let k = 3 + (rnd() % 3) as usize;
        for i in 0..16usize { LK[i] = core::mem::zeroed(); g::fill_ItemLink(&mut LK[i]); LG[i] = core::mem::zeroed(); g::fill_HSD_GObj(&mut LG[i]); }
        for i in 0..16usize {
            LK[i].next = if i < k && i != 0 { &raw mut LK[i - 1] } else { null_mut() };
            LK[i].prev = if i + 1 < k { &raw mut LK[i + 1] } else { null_mut() };
            LK[i].gobj = &raw mut LG[i]; LK[i].jobj = &raw mut JO[(i + 1) & 3];
            LG[i].user_data = &raw mut LK[i] as *mut _; LG[i].hsd_obj = &raw mut JO[i & 3] as *mut _;
            LK[i].set_x2C_b0(((i == 0) as u32 | (i + 1 == k) as u32 | (rnd() & 1)) as u8); LK[i].set_x2C_b1((rnd() & 1) as u8); LK[i].set_x2C_b2((rnd() & 1) as u8);
        }
        LTAIL = &raw mut LK[k - 1]; LHEAD = &raw mut LK[0];
        IT.xDD4_itemVar.linkhookshot.x0 = LTAIL; IT.xDD4_itemVar.linkhookshot.x4 = LHEAD;
        IT.xDD4_itemVar.linkhookshot.x8 = &raw mut OWN; IT.xDD4_itemVar.linkhookshot.xC = &raw mut JO[1];
        IT.xDD4_itemVar.linkhookshot.x10 = if rnd() & 1 != 0 { Some(m::it_802A2EE4) } else { None };
        IT2.xDD4_itemVar.linkhookshot.x0 = null_mut(); IT2.xDD4_itemVar.linkhookshot.x4 = null_mut(); IT2.xDD4_itemVar.linkhookshot.x8 = null_mut();
        IT2.xDD4_itemVar.linkhookshot.xC = null_mut(); IT2.xDD4_itemVar.linkhookshot.x10 = None;
        IT.owner = &raw mut OWN; IT2.owner = &raw mut OWN; OWN.user_data = &raw mut FTR as *mut _; OWN.hsd_obj = &raw mut JO[0] as *mut _;
    }
    m::it_804D6D48 = if rnd() & 1 != 0 { 0.0 } else { 6.0 };

}

unsafe fn pre_hash() {
    { let id = pi(IT.xDD4_itemVar.linkhookshot.x0); let b = &raw mut IT.xDD4_itemVar.linkhookshot.x0 as *mut u8; for i2 in 0..8 { *b.add(i2) = 0; } *(b as *mut u32) = id; }
    { let id = pi(IT.xDD4_itemVar.linkhookshot.x4); let b = &raw mut IT.xDD4_itemVar.linkhookshot.x4 as *mut u8; for i2 in 0..8 { *b.add(i2) = 0; } *(b as *mut u32) = id; }
    { let id = pi(IT.xDD4_itemVar.linkhookshot.x8); let b = &raw mut IT.xDD4_itemVar.linkhookshot.x8 as *mut u8; for i2 in 0..8 { *b.add(i2) = 0; } *(b as *mut u32) = id; }
    { let id = pi(IT.xDD4_itemVar.linkhookshot.xC); let b = &raw mut IT.xDD4_itemVar.linkhookshot.xC as *mut u8; for i2 in 0..8 { *b.add(i2) = 0; } *(b as *mut u32) = id; }
    { let id = fi(IT.xDD4_itemVar.linkhookshot.x10); let b = &raw mut IT.xDD4_itemVar.linkhookshot.x10 as *mut u8; for i2 in 0..8 { *b.add(i2) = 0; } *(b as *mut u32) = id; }
    { let id = pi(IT2.xDD4_itemVar.linkhookshot.x0); let b = &raw mut IT2.xDD4_itemVar.linkhookshot.x0 as *mut u8; for i2 in 0..8 { *b.add(i2) = 0; } *(b as *mut u32) = id; }
    { let id = pi(IT2.xDD4_itemVar.linkhookshot.x4); let b = &raw mut IT2.xDD4_itemVar.linkhookshot.x4 as *mut u8; for i2 in 0..8 { *b.add(i2) = 0; } *(b as *mut u32) = id; }
    { let id = pi(IT2.xDD4_itemVar.linkhookshot.x8); let b = &raw mut IT2.xDD4_itemVar.linkhookshot.x8 as *mut u8; for i2 in 0..8 { *b.add(i2) = 0; } *(b as *mut u32) = id; }
    { let id = pi(IT2.xDD4_itemVar.linkhookshot.xC); let b = &raw mut IT2.xDD4_itemVar.linkhookshot.xC as *mut u8; for i2 in 0..8 { *b.add(i2) = 0; } *(b as *mut u32) = id; }
    { let id = fi(IT2.xDD4_itemVar.linkhookshot.x10); let b = &raw mut IT2.xDD4_itemVar.linkhookshot.x10 as *mut u8; for i2 in 0..8 { *b.add(i2) = 0; } *(b as *mut u32) = id; }

}

unsafe fn hash_all() {
    g::hash_HSD_GObj(&G); g::hash_HSD_GObj(&G2); g::hash_HSD_GObj(&OWN);
    g::hash_Item(&IT); g::hash_Item(&IT2); g::hash_ItemAttr(&IA);
    for i in 0..4 { hash_jo(&JO[i]); }
    for i in 0..256 { hm(SA[i]); }
    hm(V1.x.to_bits()); hm(V1.y.to_bits()); hm(V1.z.to_bits()); hm(V2.x.to_bits()); hm(V2.y.to_bits()); hm(V2.z.to_bits());
    hash_ftr();
    for i in 0..16usize { g::hash_ItemLink(&LK[i]); g::hash_HSD_GObj(&LG[i]); }
    for i in 0..4usize { for r in 0..3usize { for c in 0..4usize { hm(JO[i].mtx[r][c].to_bits()); } } }
    for r in 0..3usize { for c in 0..4usize { hm(MTXB[r][c].to_bits()); } }
    hm(MF.to_bits()); hm(m::it_804D6D48.to_bits()); hm(pi(LP)); hm(ACNT as u32); hm(GCNT as u32);

}

#[test]
fn itlinkhookshot_matches_c() {
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
        preg(&raw const LK, core::mem::size_of::<[ItemLink; 16]>());
        preg(&raw const LG, core::mem::size_of::<[HSD_GObj; 16]>());
        preg(&raw const FTR, core::mem::size_of::<Fighter>());
        preg(&raw const FPB, core::mem::size_of::<[FighterBone; 160]>());
        preg(&raw const MPU, core::mem::size_of::<mp_UnkStruct0>());
        g::freg_all();
        freg(m::it_802A2418 as usize);
        freg(m::it_802A2428 as usize);
        freg(m::it_802A2474 as usize);
        freg(m::it_802A24A0 as usize);
        freg(m::it_802A24D0 as usize);
        freg(m::link_fighter_compare as usize);
        freg(m::it_802A2568_inline as usize);
        freg(m::it_link_get_joint as usize);
        freg(m::it_link_get_joint_c as usize);
        freg(m::it_link_lerp as usize);
        freg(m::it_link_attr_math as usize);
        freg(m::it_802A2568 as usize);
        freg(m::it_802A2B10 as usize);
        freg(m::it_802A2BA4 as usize);
        freg(m::itLinkhookshot_UnkMotion8_Anim as usize);
        freg(m::fn_802A2E4C as usize);
        freg(m::itLinkhookshot_UnkMotion0_Phys as usize);
        freg(m::it_802A2EE4_inline as usize);
        freg(m::it_802A2EE4_inline_alt as usize);
        freg(m::it_802A2EE4_inline_alt_mtx_first as usize);
        freg(m::it_802A2EE4_inline_alt_pad as usize);
        freg(m::it_802A2EE4 as usize);
        freg(m::itLinkhookshot_UnkMotion1_Phys as usize);
        freg(m::mtx_reset as usize);
        freg(m::vec3_eq_mtx as usize);
        freg(m::fn_802A3110_inline as usize);
        freg(m::fn_802A3110 as usize);
        freg(m::itLinkhookshot_UnkMotion2_Phys as usize);
        freg(m::it_802A3254 as usize);
        freg(m::itLinkhookshot_UnkMotion3_Phys as usize);
        freg(m::fn_802A33A0_GetFighter as usize);
        freg(m::fn_802A33A0 as usize);
        freg(m::itLinkhookshot_UnkMotion4_Phys as usize);
        freg(m::it_802A3500 as usize);
        freg(m::itLinkhookshot_UnkMotion5_Phys as usize);
        freg(m::it_802A3630 as usize);
        freg(m::itLinkhookshot_UnkMotion6_Phys as usize);
        freg(m::it_802A3828 as usize);
        freg(m::itLinkhookshot_UnkMotion7_Phys as usize);
        freg(m::it_802A39FC as usize);
        freg(m::itLinkhookshot_UnkMotion8_Phys as usize);
        freg(m::it_802A3C98 as usize);
        freg(m::it_802A3D90 as usize);
        freg(m::it_802A3E50 as usize);
        freg(m::it_802A40D0 as usize);
        freg(m::it_802A42F4 as usize);
        freg(m::it_802A43B8 as usize);
        freg(m::it_802A43EC as usize);
        freg(m::it_802A4420 as usize);
        freg(m::it_802A4454 as usize);
        freg(m::it_802A44CC as usize);
        freg(m::test_comp as usize);
        freg(m::it_802A4758_permuterslop as usize);
        freg(m::it_802A4758 as usize);
        freg(m::it_802A49B0 as usize);
        freg(m::it_802A6A78_normalize_diff as usize);
        freg(m::it_802A4BFC_sqrtf_offset as usize);
        freg(m::it_802A4BFC_normalize_diff as usize);
        freg(m::it_802A4BFC as usize);
        freg(m::it_802A5320 as usize);
        freg(m::it_802A5770_inline as usize);
        freg(m::it_802A5770 as usize);
        freg(m::it_802A5AE0 as usize);
        freg(m::it_802A5E28 as usize);
        freg(m::it_802A5FE0 as usize);
        freg(m::it_802A4758_no_inline as usize);
        freg(m::it_802A6474 as usize);
        freg(m::it_802A678C as usize);
        freg(m::it_802A6944 as usize);
        freg(m::it_802A6A78_get_next as usize);
        freg(m::it_802A6A78_get_pos as usize);
        freg(m::it_802A6A78_normalize_diff_rev as usize);
        freg(m::it_802A6A78 as usize);
        freg(m::it_802A6DC8 as usize);
        freg(m::it_802A6F80 as usize);
        freg(m::it_802A7168 as usize);
        freg(m::it_802A7384 as usize);
        freg(m::itLinkHookshot_Logic20_PickedUp_inline as usize);
        freg(m::itLinkHookshot_Logic20_PickedUp as usize);
        freg(m::it_802A76EC as usize);
        freg(m::it_802A7764 as usize);
        freg(m::it_802A77DC as usize);
        freg(m::it_802A7840 as usize);
        freg(m::it_802A78B8 as usize);
        freg(m::it_802A793C as usize);
        freg(m::it_802A79A0 as usize);
        freg(m::it_802A7A04 as usize);
        freg(m::it_802A7AAC as usize);
        freg(m::it_802A7AF0 as usize);
        freg(m::it_802A7B34_6944_inline as usize);
        freg(m::it_802A7B34 as usize);
        freg(m::it_802A7D40 as usize);
        let mut c = Check::new(include_str!("data/mtu_itlinkhookshot.txt"));
        for op in 0..93u32 {
            for k in 0..150u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                    0 => { m::it_802A2418(&raw mut G); }
                    1 => { m::it_802A2428(&raw mut G); }
                    2 => { m::it_802A2474((&raw mut IT) as *mut c_void); }
                    3 => { m::it_802A24A0(&raw mut LG[0], fv() as i32); }
                    4 => { m::it_802A24D0(LTAIL, rw()); }
                    5 => { ret = m::link_fighter_compare(&raw mut FTR) as u32; }
                    6 => { m::it_802A2568_inline(LTAIL, &raw mut JO[1], &raw mut G, &raw mut V1); }
                    7 => { ret = pi(m::it_link_get_joint(&raw mut IT, fv() as _)); }
                    8 => { ret = pi(m::it_link_get_joint_c(&raw mut IT)); }
                    9 => { ret = m::it_link_lerp(rw(), rw(), rw()).to_bits(); }
                    10 => { m::it_link_attr_math(SA.as_mut_ptr() as *mut itLinkHookshotAttributes, (fv() % 2) as i32, if rnd() & 1 != 0 { if rnd() & 1 != 0 { 0.5 } else { 1.0 } } else { 2.0 }); }
                    11 => { ret = pi(m::it_802A2568(&raw mut IT, &raw mut JO[1], (fv() % 2) as i32, if rnd() & 1 != 0 { if rnd() & 1 != 0 { 0.5 } else { 1.0 } } else { 2.0 })); }
                    12 => { m::it_802A2B10(&raw mut G); }
                    13 => { ret = pi(m::it_802A2BA4(&raw mut OWN, &raw mut V1, rw(), fv() as i32)); }
                    14 => { ret = m::itLinkhookshot_UnkMotion8_Anim(&raw mut G) as u32; }
                    15 => { m::fn_802A2E4C(&raw mut G); }
                    16 => { m::itLinkhookshot_UnkMotion0_Phys(&raw mut G); }
                    17 => { m::it_802A2EE4_inline(MTXB.as_mut_ptr(), LTAIL, &raw mut V1); }
                    18 => { m::it_802A2EE4_inline_alt(LTAIL, &raw mut V1); }
                    19 => { m::it_802A2EE4_inline_alt_mtx_first(LTAIL, &raw mut V1); }
                    20 => { m::it_802A2EE4_inline_alt_pad(LTAIL, &raw mut V1); }
                    21 => { m::it_802A2EE4(&raw mut G); }
                    22 => { m::itLinkhookshot_UnkMotion1_Phys(&raw mut G); }
                    23 => { m::mtx_reset(MTXB.as_mut_ptr()); }
                    24 => { m::vec3_eq_mtx(&raw mut V1, MTXB.as_mut_ptr()); }
                    25 => { m::fn_802A3110_inline(LTAIL, &raw mut V1); }
                    26 => { m::fn_802A3110(&raw mut G); }
                    27 => { m::itLinkhookshot_UnkMotion2_Phys(&raw mut G); }
                    28 => { m::it_802A3254(&raw mut G); }
                    29 => { m::itLinkhookshot_UnkMotion3_Phys(&raw mut G); }
                    30 => { ret = pi(m::fn_802A33A0_GetFighter(&raw mut IT)); }
                    31 => { m::fn_802A33A0(&raw mut G); }
                    32 => { m::itLinkhookshot_UnkMotion4_Phys(&raw mut G); }
                    33 => { m::it_802A3500(&raw mut G); }
                    34 => { m::itLinkhookshot_UnkMotion5_Phys(&raw mut G); }
                    35 => { m::it_802A3630(&raw mut G); }
                    36 => { m::itLinkhookshot_UnkMotion6_Phys(&raw mut G); }
                    37 => { m::it_802A3828(&raw mut G); }
                    38 => { m::itLinkhookshot_UnkMotion7_Phys(&raw mut G); }
                    39 => { m::it_802A39FC(&raw mut G); }
                    40 => { m::itLinkhookshot_UnkMotion8_Phys(&raw mut G); }
                    41 => { ret = m::it_802A3C98(&raw mut V1, &raw mut V2, &raw mut IT.pos).to_bits(); }
                    42 => { ret = m::it_802A3D90(LTAIL) as u32; }
                    43 => { ret = m::it_802A3E50(LTAIL, if rnd() & 1 != 0 { 20 } else { 6 }, rw()) as u32; }
                    44 => { ret = m::it_802A40D0(LTAIL, rw()) as u32; }
                    45 => { ret = m::it_802A42F4(LTAIL, rw()) as u32; }
                    46 => { m::it_802A43B8(LTAIL); }
                    47 => { m::it_802A43EC(LTAIL); }
                    48 => { m::it_802A4420(LTAIL); }
                    49 => { m::it_802A4454(LTAIL); }
                    50 => { m::it_802A44CC(LTAIL, &raw mut V1, SA.as_mut_ptr() as *mut itLinkHookshotAttributes, rw()); }
                    51 => { m::test_comp(&raw mut IT.pos, &raw mut V1, &raw mut V2, &raw const MF); }
                    52 => { ret = pi(m::it_802A4758_permuterslop(LTAIL)); }
                    53 => { m::it_802A4758(LTAIL, &raw mut V1, SA.as_mut_ptr() as *mut itLinkHookshotAttributes, rw()); }
                    54 => { m::it_802A49B0(LTAIL, &raw mut V1, SA.as_mut_ptr() as *mut itLinkHookshotAttributes, rw()); }
                    55 => { { let d = m::it_802A6A78_normalize_diff(&raw mut V1, &raw mut V2, &raw mut IT.pos); hm64(d); } }
                    56 => { ret = m::it_802A4BFC_sqrtf_offset(rw()).to_bits(); }
                    57 => { { let d = m::it_802A4BFC_normalize_diff(&raw mut V1, &raw mut V2, &raw mut IT.pos); hm64(d); } }
                    58 => { ret = m::it_802A4BFC(LTAIL, &raw mut V1, SA.as_mut_ptr() as *mut itLinkHookshotAttributes, &raw mut FTR) as u32; }
                    59 => { ret = m::it_802A5320(LTAIL, &raw mut V1, SA.as_mut_ptr() as *mut itLinkHookshotAttributes, &raw mut FTR) as u32; }
                    60 => { m::it_802A5770_inline(LTAIL, SA.as_mut_ptr() as *mut itLinkHookshotAttributes, &raw mut FTR, fv() as _); }
                    61 => { m::it_802A5770(LTAIL, &raw mut V1, SA.as_mut_ptr() as *mut itLinkHookshotAttributes, &raw mut FTR); }
                    62 => { ret = m::it_802A5AE0(LTAIL, &raw mut V1, SA.as_mut_ptr() as *mut itLinkHookshotAttributes) as u32; }
                    63 => { ret = m::it_802A5E28(LTAIL, &raw mut V1, SA.as_mut_ptr() as *mut itLinkHookshotAttributes, rw()) as u32; }
                    64 => { ret = m::it_802A5FE0(LTAIL, LHEAD, &raw mut V1, SA.as_mut_ptr() as *mut itLinkHookshotAttributes, rw()) as u32; }
                    65 => { m::it_802A4758_no_inline(LTAIL, &raw mut V1, SA.as_mut_ptr() as *mut itLinkHookshotAttributes, rw()); }
                    66 => { m::it_802A6474(LTAIL, LHEAD, &raw mut V1, SA.as_mut_ptr() as *mut itLinkHookshotAttributes); }
                    67 => { ret = m::it_802A678C(LTAIL, &raw mut V1, SA.as_mut_ptr() as *mut itLinkHookshotAttributes, rw()) as u32; }
                    68 => { m::it_802A6944(&raw mut IT, rw()); }
                    69 => { m::it_802A6A78_get_next(LTAIL, &raw mut LP); }
                    70 => { ret = pi(m::it_802A6A78_get_pos(LTAIL)); }
                    71 => { { let d = m::it_802A6A78_normalize_diff_rev(&raw mut V1, &raw mut V2, &raw mut IT.pos); hm64(d); } }
                    72 => { ret = m::it_802A6A78(LTAIL, &raw mut V1, SA.as_mut_ptr() as *mut itLinkHookshotAttributes, &raw mut FTR) as u32; }
                    73 => { m::it_802A6DC8(&raw mut JO[1], &raw mut V1, &raw mut V2); }
                    74 => { m::it_802A6F80(&raw mut JO[1], &raw mut V1, &raw mut V2, rw()); }
                    75 => { m::it_802A7168(&raw mut IT, &raw mut V1, rw()); }
                    76 => { m::it_802A7384(&raw mut IT, &raw mut V1, rw()); }
                    77 => { m::itLinkHookshot_Logic20_PickedUp_inline(&raw mut G); }
                    78 => { m::itLinkHookshot_Logic20_PickedUp(&raw mut G); }
                    79 => { m::it_802A76EC(&raw mut G); }
                    80 => { m::it_802A7764(&raw mut G); }
                    81 => { m::it_802A77DC(&raw mut G); }
                    82 => { m::it_802A7840(&raw mut G); }
                    83 => { m::it_802A78B8(&raw mut G, &raw mut V1); }
                    84 => { m::it_802A793C(&raw mut G); }
                    85 => { m::it_802A79A0(&raw mut G); }
                    86 => { m::it_802A7A04(&raw mut G); }
                    87 => { m::it_802A7AAC(&raw mut OWN); }
                    88 => { m::it_802A7AF0(&raw mut OWN); }
                    89 => { m::it_802A7B34_6944_inline(&raw mut IT); }
                    90 => { m::it_802A7B34(&raw mut OWN); }
                    91 => { m::it_802A7D40(&raw mut G, &raw mut G2); }
                    92 => { { for i in 0..9 { hm(m::it_803F6998[i].anim_id as u32); hm(fi(m::it_803F6998[i].animated)); hm(fi(m::it_803F6998[i].physics_updated)); hm(fi(m::it_803F6998[i].collided)); } } }
                    _ => unreachable!(),
                }
                pre_hash();
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 13950);
    }
}
