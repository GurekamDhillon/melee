//! Written by gen/kinds/mk_driver.py from the same description as gen/kinds/tu/itnessyoyo.c.
//! src/tu/itnessyoyo.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_items/gen/kinds/run_tu.py itnessyoyo`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
#[path = "kinds_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itnessyoyo as m;
use mtu_harness::*;
#[path = "kinds_gen/itnessyoyo_gen.rs"]
mod g;

#[repr(C, align(16))]
struct SpecBuf([[u8; 128]; 3]);
static mut GO: [HSD_GObj; 3] = unsafe { core::mem::zeroed() };
static mut IT: [Item; 3] = unsafe { core::mem::zeroed() };
static mut JO: [HSD_JObj; 48] = unsafe { core::mem::zeroed() };
static mut DBT: [DynamicBoneTable; 3] = unsafe { core::mem::zeroed() };
static mut ART: [Article; 3] = unsafe { core::mem::zeroed() };
static mut ATTR: [ItemAttr; 3] = unsafe { core::mem::zeroed() };
static mut ICD: ItemCommonData = unsafe { core::mem::zeroed() };
static mut SPECB: SpecBuf = SpecBuf([[0; 128]; 3]);
static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };

static mut LK: [ItemLink; 6] = unsafe { core::mem::zeroed() };
static mut LG: [HSD_GObj; 6] = unsafe { core::mem::zeroed() };
static mut LJ: [HSD_JObj; 6] = unsafe { core::mem::zeroed() };
static mut LB: [HSD_JObj; 6] = unsafe { core::mem::zeroed() };
static mut FPL: Fighter = unsafe { core::mem::zeroed() };
static mut GF: HSD_GObj = unsafe { core::mem::zeroed() };
static mut PARTS: [FighterBone; 64] = unsafe { core::mem::zeroed() };
static mut CNT: [usize; 4] = [0; 4];


unsafe fn spec_fill(prof: i32) {
    match prof {
        0 => {
            for j in 0..3 {
                let a = SPECB.0[j].as_mut_ptr() as *mut itYoyoAttributes;
                (*a).x0_CHARGE_SPAWN_POS = (rnd() % 5) as i32; (*a).x4_UPSMASH_CHARGE_STRING_LENGTH = (rnd() % 6) as i32; (*a).x8_DOWNSMASH_CHARGE_STRING_LENGTH = (rnd() % 6) as i32; (*a).xC_SIZE = rw(); (*a).x10_UNK1 = rw(); (*a).x28_YOYO_PULL_STRENGTH = rw(); (*a).x2C_UNK3_MOD = rw(); (*a).x34_GRAVITY = rw(); (*a).x38_GRAVITY_MOD = rw(); (*a).x3C_YOYO_PULL_STRENGTH_2 = rw();
            }
        }
        _ => unreachable!(),
    }
}
unsafe fn setup(prof: i32) {
    SPECB = SpecBuf([[0; 128]; 3]);
    GO = core::mem::zeroed();
    IT = core::mem::zeroed();
    JO = core::mem::zeroed();
    DBT = core::mem::zeroed();
    ART = core::mem::zeroed();
    ATTR = core::mem::zeroed();
    ICD = core::mem::zeroed();
    VA = core::mem::zeroed();
    for k in 0..3 {
        g::fill_HSD_GObj(&mut GO[k]);
        g::fill_Item(&mut IT[k]);
        g::fill_ItemAttr(&mut ATTR[k]);
    }
    for i in 0..48 {
        g::fill_HSD_JObj(&mut JO[i]);
        JO[i].flags &= !(JOBJ_USE_QUATERNION as u32);
    }
    g::fill_ItemCommonData(&mut ICD);
    for i in 0..4 {
        VA[i].x = rw();
        VA[i].y = rw();
        VA[i].z = rw();
    }
    for k in 0..3 {
        GO[k].user_data = &raw mut IT[k] as *mut _;
        GO[k].hsd_obj = &raw mut JO[k] as *mut _;
        JO[k].child = &raw mut JO[3 + k];
        IT[k].entity = &raw mut GO[k];
        IT[k].xBBC_dynamicBoneTable = &raw mut DBT[k];
        IT[k].xC4_article_data = &raw mut ART[k];
        IT[k].xCC_item_attr = &raw mut ATTR[k];
        ART[k].x0_common_attr = &raw mut ATTR[k];
        ART[k].x4_specialAttributes = SPECB.0[k].as_mut_ptr() as *mut _;
        for i in 0..14 {
            DBT[k].bones[i] = &raw mut JO[6 + 14 * k + i];
        }
    }

    IT[0].owner = if rnd() & 3 != 0 { &raw mut GO[1] } else { null_mut() };
    IT[0].xC64_reflectGObj = if rnd() & 3 != 0 { &raw mut GO[1] } else { null_mut() };
    if rnd() & 1 != 0 {
        IT[0].msid = (rnd() % 14) as i32;
    }
    if rnd() & 1 != 0 {
        IT[0].facing_dir = if rnd() & 1 != 0 { 1.0 } else { -1.0 };
    }
    IT[0].ground_or_air = if rnd() & 1 != 0 { GA_Air } else { GA_Ground };
    spec_fill(prof);
    {
        LK = core::mem::zeroed(); LG = core::mem::zeroed(); LJ = core::mem::zeroed(); LB = core::mem::zeroed();
        FPL = core::mem::zeroed(); GF = core::mem::zeroed(); PARTS = core::mem::zeroed(); CNT = [0; 4];
        for j in 0..6 {
            g::fill_ItemLink(&mut LK[j]); g::fill_HSD_GObj(&mut LG[j]); g::fill_HSD_JObj(&mut LJ[j]); g::fill_HSD_JObj(&mut LB[j]);
            LJ[j].flags &= !(JOBJ_USE_QUATERNION as u32); LB[j].flags &= !(JOBJ_USE_QUATERNION as u32);
            LJ[j].child = if rnd() & 1 != 0 { &raw mut LB[j] } else { null_mut() };
        }
        let n = 1 + (rnd() % 4) as usize;
        for j in 0..6usize {
            LK[j].prev = if j < n && j + 1 < n { &raw mut LK[j + 1] } else { null_mut() };
            LK[j].next = if j < n && j > 0 { &raw mut LK[j - 1] } else { null_mut() };
            LK[j].gobj = &raw mut LG[j];
            LK[j].jobj = &raw mut LB[j];
            LG[j].hsd_obj = &raw mut LJ[j] as *mut _;
        }
        let j = (rnd() % n as u32) as usize;
        LK[j].set_x2C_b0(1);
        FPL.u.ns.yoyo_hitbox_pos.x = rw(); FPL.u.ns.yoyo_hitbox_pos.y = rw(); FPL.u.ns.yoyo_hitbox_pos.z = rw();
        FPL.u.ns.x223C = rw();
        FPL.parts = &raw mut PARTS as *mut FighterBone;
        PARTS[FtPart_R2ndNa as usize].joint = &raw mut LB[0];
        GF.user_data = &raw mut FPL as *mut _;
        for k in 0..3 {
            let i = rnd() % 3;
            IT[k].xDD4_itemVar.nessyoyo.x0 = if i == 0 { 0x156 } else if i == 1 { 0x159 } else { (rnd() % 0x20) as i32 };
            IT[k].xDD4_itemVar.nessyoyo.x4 = rw();
            IT[k].xDD4_itemVar.nessyoyo.x8 = &raw mut LK[n - 1];
            IT[k].xDD4_itemVar.nessyoyo.xC = &raw mut LK[0];
            IT[k].xDD4_itemVar.nessyoyo.x10 = &raw mut GF;
            let i = rnd() % 4;
            IT[k].xDD4_itemVar.nessyoyo.x18 = if i == 0 { null_mut() } else { let q = (rnd() % n as u32) as usize; &raw mut LJ[q] };
        }
    }
}

unsafe fn hash_all() {
    for k in 0..3 {
        let nv = &mut IT[k].xDD4_itemVar.nessyoyo;
        hm(pi(nv.x8)); nv.x8 = null_mut(); hm(pi(nv.xC)); nv.xC = null_mut(); hm(pi(nv.x10)); nv.x10 = null_mut(); hm(pi(nv.x18)); nv.x18 = null_mut();
    }
    for i in 0..6 { g::hash_ItemLink(&LK[i]); g::hash_HSD_GObj(&LG[i]); g::hash_HSD_JObj(&LJ[i]); g::hash_HSD_JObj(&LB[i]); }
    hm(FPL.u.ns.yoyo_hitbox_pos.x.to_bits()); hm(FPL.u.ns.yoyo_hitbox_pos.y.to_bits()); hm(FPL.u.ns.yoyo_hitbox_pos.z.to_bits());
    hm(FPL.u.ns.x223C.to_bits()); hm(CNT[0] as u32); hm(CNT[1] as u32); hm(CNT[2] as u32);
    for k in 0..3 {
        g::hash_HSD_GObj(&GO[k]);
        g::hash_Item(&IT[k]);
        g::hash_ItemAttr(&ATTR[k]);
    }
    for i in 0..48 {
        g::hash_HSD_JObj(&JO[i]);
    }
    for i in 0..4 {
        hm(VA[i].x.to_bits());
        hm(VA[i].y.to_bits());
        hm(VA[i].z.to_bits());
    }
}

#[test]
fn itnessyoyo_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const IT, core::mem::size_of_val(&IT));
        preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const DBT, core::mem::size_of_val(&DBT));
        preg(&raw const ART, core::mem::size_of_val(&ART)); preg(&raw const ATTR, core::mem::size_of_val(&ATTR));
        preg(&raw const ICD, core::mem::size_of_val(&ICD)); preg(&raw const SPECB, 3 * 128); preg(&raw const VA, core::mem::size_of_val(&VA));
        preg(&raw const LK, core::mem::size_of_val(&LK)); preg(&raw const LG, core::mem::size_of_val(&LG)); preg(&raw const LJ, core::mem::size_of_val(&LJ)); preg(&raw const LB, core::mem::size_of_val(&LB)); preg(&raw const FPL, core::mem::size_of_val(&FPL)); preg(&raw const GF, core::mem::size_of_val(&GF)); preg(&raw const PARTS, core::mem::size_of_val(&PARTS));
        g::freg_all();
        const PROFTAB: [u8; 24] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut c = Check::new(include_str!("data/kinds_itnessyoyo.txt"));
        for op in 0..24u32 {
            for k in 0..400u32 {
                case(op, k);
                setup(PROFTAB[op as usize] as i32);
                let mut ret = 0u32;
                match op {
                    0 => {  m::it_802BE598(&raw mut GO[0]); } // it_802BE598
                    1 => {  m::it_802BE5B8(&raw mut GO[0]); } // it_802BE5B8
                    2 => { m::it_802BE5D8(&raw mut GO[0] as *mut core::ffi::c_void, rw()); } // it_802BE5D8
                    3 => {  ret = pi(m::it_802BE65C(&raw mut IT[0], &raw mut JO[3])); } // it_802BE65C
                    4 => {  m::it_802BE958(&raw mut GO[0]); } // it_802BE958
                    5 => { { let f = rw(); let a = if rnd() % 3 == 0 { 0x159 } else { (rnd() % 0x20) as i32 }; ret = pi(m::it_802BE9D8(&raw mut GF, f, &raw mut VA[0], a)); } } // it_802BE9D8
                    6 => {  m::itNessyoyo_UnkMotion0_Phys(&raw mut GO[0]); } // itNessyoyo_UnkMotion0_Phys
                    7 => {  m::itNessyoyo_UnkMotion1_Phys(&raw mut GO[0]); } // itNessyoyo_UnkMotion1_Phys
                    8 => {  m::itNessyoyo_UnkMotion2_Phys(&raw mut GO[0]); } // itNessyoyo_UnkMotion2_Phys
                    9 => {  m::itNessyoyo_UnkMotion3_Phys(&raw mut GO[0]); } // itNessyoyo_UnkMotion3_Phys
                    10 => {  ret = m::itNessyoyo_UnkMotion3_Anim(&raw mut GO[0]) as u32; } // itNessyoyo_UnkMotion3_Anim
                    11 => { { let ix = (rnd() % 3) as usize; let a = fv() as i32; let o = rw(); ret = m::it_802BF030(&raw mut LK[ix], a, o) as u32; } } // it_802BF030
                    12 => { { let ix = (rnd() % 3) as usize; let l = rw(); m::it_802BF180(&raw mut LK[ix], &raw mut VA[0], SPECB.0[0].as_mut_ptr() as *mut itYoyoAttributes, l); } } // it_802BF180
                    13 => { { let ix = (rnd() % 3) as usize; ret = m::it_802BF28C(&raw mut LK[ix], &raw mut VA[0], SPECB.0[0].as_mut_ptr() as *mut itYoyoAttributes, &raw mut IT[0]) as u32; } } // it_802BF28C
                    14 => { { let ix = (rnd() % 3) as usize; ret = m::it_802BF4A0(&raw mut LK[ix], &raw mut VA[0], SPECB.0[0].as_mut_ptr() as *mut itYoyoAttributes, &raw mut IT[0]) as u32; } } // it_802BF4A0
                    15 => { { let ix = (rnd() % 3) as usize; let d = rw(); ret = m::it_802BF800(&raw mut LK[ix], &raw mut VA[0], SPECB.0[0].as_mut_ptr() as *mut itYoyoAttributes, &raw mut IT[0], d) as u32; } } // it_802BF800
                    16 => {  m::it_802BF900(&raw mut IT[0]); } // it_802BF900
                    17 => {  m::it_802BFAFC(&raw mut IT[0], &raw mut VA[0]); } // it_802BFAFC
                    18 => {  m::itNessYoyo_Logic59_PickedUp(&raw mut GO[0]); } // itNessYoyo_Logic59_PickedUp
                    19 => { let v2: f32 = rw() as f32; m::it_802BFE5C(&raw mut GO[0], &raw mut VA[0], v2); } // it_802BFE5C
                    20 => {  m::it_802BFEC4(&raw mut GO[0]); } // it_802BFEC4
                    21 => {  m::it_802C0010(&raw mut GO[0], &raw mut VA[0]); } // it_802C0010
                    22 => {  m::it_2725_Logic59_EvtUnk(&raw mut GO[0], &raw mut GO[1]); } // it_2725_Logic59_EvtUnk
                    23 => { for j in 0..4 { let row = &m::it_803F7558[j]; hm(row.anim_id as u32); hm(fi(row.animated)); hm(fi(row.physics_updated)); hm(fi(row.collided)); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 9600);
    }
}
