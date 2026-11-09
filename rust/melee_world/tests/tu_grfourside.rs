//! Written by gen/mtu/mk_stage_driver.py from the same description as gen/mtu/tu/grfourside.c.
//! src/tu/grfourside.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_world/gen/mtu/run_tu.py grfourside`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables)]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_world::tu::grfourside as m;
use mtu_harness::*;
#[path = "mtu/grfourside_gen.rs"]
mod g;

static mut SGO: HSD_GObj = unsafe { core::mem::zeroed() };
static mut SGP: Ground = unsafe { core::mem::zeroed() };
static mut SJO: [u8; 16] = [0; 16];
static mut YPB: [u8; 64] = [0; 64];
static mut SV: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
static mut SJOB: HSD_JObj = unsafe { core::mem::zeroed() };
static mut YP: m::grFourside_YakumonoParam = unsafe { core::mem::zeroed() };
static mut MGO: [HSD_GObj; 8] = unsafe { core::mem::zeroed() };
static mut MGP: [Ground; 8] = unsafe { core::mem::zeroed() };
static mut MJO: [HSD_JObj; 8] = unsafe { core::mem::zeroed() };
static mut FJ: [HSD_JObj; 10] = unsafe { core::mem::zeroed() };
static mut CMS: CmSubject = unsafe { core::mem::zeroed() };
static mut COBJ: HSD_CObj = unsafe { core::mem::zeroed() };
static mut CD: CollData = unsafe { core::mem::zeroed() };
static mut PTRMODE: i32 = 0;




unsafe fn setup() {
    let mut n: u32;
    SGO = core::mem::zeroed();
    SGP = core::mem::zeroed();
    g::fill_HSD_GObj(&mut SGO);
    g::fill_Ground(&mut SGP);
    g::fill_StageInfo(&mut m::ext::stage_info);
    SGO.user_data = &raw mut SGP as *mut _;
    SGO.hsd_obj = &raw mut SJO as *mut _;
    SV.x = rw();
    SV.y = rw();
    SV.z = rw();
    SJOB = core::mem::zeroed();
    YP = core::mem::zeroed();
    MGO = core::mem::zeroed();
    MGP = core::mem::zeroed();
    MJO = core::mem::zeroed();
    FJ = core::mem::zeroed();
    CMS = core::mem::zeroed();
    COBJ = core::mem::zeroed();
    CD = core::mem::zeroed();
    PTRMODE = 0;
    g::fill_HSD_JObj(&mut SJOB);
    SJOB.flags &= !(JOBJ_USE_QUATERNION as u32);
    SGO.hsd_obj = &raw mut SJOB as *mut _;
    g::fill_grFourside_YakumonoParam(&mut YP);
    YP.heli_wait = (rnd() % 60) as i32;
    YP.heli_wait_add = (rnd() % 40) as i32;
    YP.heli_stay_time = (rnd() % 60) as i32;
    YP.crane_wait = (rnd() % 50) as i32;
    YP.crane_wait_add = (rnd() % 30) as i32;
    YP.crane_iron_wait = (rnd() % 40) as i32;
    YP.crane_iron_wait_add = (rnd() % 20) as i32;
    YP.crane_iron_up_min = (rnd() % 20) as f32 * 0.5;
    YP.crane_iron_up_max = (rnd() % 60) as f32 * 0.5;
    YP.crane_iron_down_min = (rnd() % 20) as f32 * 0.5;
    YP.crane_iron_down_max = (rnd() % 60) as f32 * 0.5;
    YP.crane_iron_spd = (rnd() % 30) as f32 / 10.0;
    YP.crane_iron_stop_acl = rw();
    YP.ufo_wait = (rnd() % 60) as i32;
    YP.ufo_cs_offs = rw();
    YP.ufo_stay_time = (rnd() % 60) as i32;
    YP.ufo_stay_time_add = (rnd() % 30) as i32;
    YP.ufo_challenge = (rnd() % 6) as u16;
    YP.x46 = (rnd() % 10) as u16;
    YP.x48 = (rnd() % 10) as u16;
    m::yakumono_param = &raw mut YP;
    g::fill_CmSubject(&mut CMS);
    g::fill_CollData(&mut CD);
    CD.x34_flags.set_b1234(if rnd() & 1 != 0 { 1 } else { fv() as u8 });
    for i in 0..8 {
        g::fill_HSD_GObj(&mut MGO[i]);
        g::fill_Ground(&mut MGP[i]);
        g::fill_HSD_JObj(&mut MJO[i]);
        MJO[i].flags &= !(JOBJ_USE_QUATERNION as u32);
        MGO[i].user_data = &raw mut MGP[i] as *mut _;
        MGO[i].hsd_obj = &raw mut MJO[i] as *mut _;
    }
    for i in 0..10 {
        g::fill_HSD_JObj(&mut FJ[i]);
        FJ[i].flags &= !(JOBJ_USE_QUATERNION as u32);
    }

}

unsafe fn hash_all() {
    if PTRMODE & 1 != 0 {
        hm(pib(SGP.u.foursideUfo.xC));
        SGP.u.foursideUfo.xC = null_mut();
    }
    if PTRMODE & 2 != 0 {
        hm(pib(SGP.u.fourside.x0));
        hm(pib(SGP.u.fourside.x4));
        hm(pib(SGP.u.fourside.x8));
        SGP.u.fourside.x0 = null_mut();
        SGP.u.fourside.x4 = null_mut();
        SGP.u.fourside.x8 = null_mut();
    }
    if PTRMODE & 4 != 0 {
        let x = &SGP.u.foursideCrane.x1;
        hm(x.b0() as u32);
        hm(x.b1() as u32);
        hm(x.b2() as u32);
        hm(x.b3() as u32);
        hm(x.b4() as u32);
        hm(x.b5() as u32);
        hm(x.b6() as u32);
        hm(x.b7() as u32);
        *(&raw mut SGP.u as *mut u8).add(1) = 0;
    }
    g::hash_HSD_GObj(&SGO);
    g::hash_Ground(&SGP);
    g::hash_StageInfo(&m::ext::stage_info);
    hm(SV.x.to_bits());
    hm(SV.y.to_bits());
    hm(SV.z.to_bits());
    g::hash_HSD_JObj(&SJOB);
    g::hash_grFourside_YakumonoParam(&YP);
    hm(pi(m::yakumono_param));
    g::hash_CmSubject(&CMS);
    g::hash_CollData(&CD);
    for i in 0..8 {
        g::hash_HSD_GObj(&MGO[i]);
        g::hash_Ground(&MGP[i]);
        g::hash_HSD_JObj(&MJO[i]);
    }
    for i in 0..10 {
        g::hash_HSD_JObj(&FJ[i]);
    }

}

#[test]
fn grfourside_matches_c() {
    unsafe {
        g::install();
        preg(&raw const SGO, core::mem::size_of::<HSD_GObj>());
        preg(&raw const SGP, core::mem::size_of::<Ground>());
        preg(&raw const SJO, 16);
        preg(&raw const YPB, 64);
        preg(&raw const SV, core::mem::size_of::<Vec3>());
        preg(&raw const SJOB, core::mem::size_of::<HSD_JObj>());
        preg(&raw const YP, core::mem::size_of::<m::grFourside_YakumonoParam>());
        preg(&raw const MGO, core::mem::size_of_val(&MGO));
        preg(&raw const MGP, core::mem::size_of_val(&MGP));
        preg(&raw const MJO, core::mem::size_of_val(&MJO));
        preg(&raw const FJ, core::mem::size_of_val(&FJ));
        preg(&raw const CMS, core::mem::size_of::<CmSubject>());
        preg(&raw const COBJ, 16);
        preg(&raw const CD, core::mem::size_of::<CollData>());
        preg(&raw const m::grFs_StageCallbacks, core::mem::size_of_val(&m::grFs_StageCallbacks));
        preg(&raw const m::grFs_StageData, core::mem::size_of_val(&m::grFs_StageData));
        preg(&raw const m::grFs_803E3CE8, core::mem::size_of_val(&m::grFs_803E3CE8));
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_grfourside.txt"));
        for op in 0..44u32 {
            for k in 0..500u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                    0 => { m::grFourside_801F2D0C(fv() as i32); }
                    1 => { PTRMODE |= 7; m::grFourside_801F2D10(); }
                    2 => { m::grFourside_801F2DA0(); }
                    3 => { m::grFourside_801F2DA4(); }
                    4 => { ret = m::grFourside_801F2DC8() as u32; }
                    5 => { PTRMODE |= 7; ret = pi(m::grFourside_801F2DD0((rnd() % 7) as i32)); }
                    6 => { m::stageGObj0_OnInit(&raw mut SGO); }
                    7 => { ret = m::grFourside_801F2EE8(&raw mut SGO) as u32; }
                    8 => { m::grFourside_801F2EF0(&raw mut SGO); }
                    9 => { m::grFourside_801F2EF4(&raw mut SGO); }
                    10 => { m::stageGObj2_OnInit(&raw mut SGO); }
                    11 => { ret = m::grFourside_801F2F24(&raw mut SGO) as u32; }
                    12 => { m::grFourside_801F2F2C(&raw mut SGO); }
                    13 => { m::grFourside_801F2F30(&raw mut SGO); }
                    14 => { PTRMODE |= 2; m::grFourside_801F2F34(&raw mut SGO); }
                    15 => { ret = m::grFourside_801F3070(&raw mut SGO) as u32; }
                    16 => { m::grFourside_801F3078(&raw mut SGO); }
                    17 => { m::grFourside_801F309C(&raw mut SGO); }
                    18 => { { let j = fv() as i32; let x = fv() as i32; let d = rw(); m::grFourside_801F30A0(&raw mut SGP as *mut _, j, &raw mut CD, x, fv() as _, d); } }
                    19 => { m::grFourside_801F30F0(&raw mut SGO); }
                    20 => { ret = m::grFourside_801F3144(&raw mut SGO) as u32; }
                    21 => { m::grFourside_801F314C(&raw mut SGO); }
                    22 => { m::grFourside_801F3150(&raw mut SGO); }
                    23 => { PTRMODE |= 4; SGP.u.foursideCrane.x1.set_b0((rnd() & 1) as u8); SGP.u.foursideCrane.x1.set_b1((rnd() & 1) as u8); SGP.u.foursideCrane.x1.set_b2((rnd() & 1) as u8); SGP.u.foursideCrane.x1.set_b3((rnd() & 1) as u8); SGP.u.foursideCrane.x1.set_b4((rnd() & 1) as u8); SGP.u.foursideCrane.x1.set_b5((rnd() & 1) as u8); SGP.u.foursideCrane.x1.set_b6((rnd() & 1) as u8); SGP.u.foursideCrane.x1.set_b7((rnd() & 1) as u8); m::grFourside_801F3154(&raw mut SGO); }
                    24 => { ret = m::grFourside_801F326C(&raw mut SGO) as u32; }
                    25 => { PTRMODE |= 4; SGP.u.foursideCrane.x0 = (rnd() % 8) as u8; SGP.u.foursideCrane.x1.set_b0((rnd() & 1) as u8); SGP.u.foursideCrane.x1.set_b1((rnd() & 1) as u8); SGP.u.foursideCrane.x1.set_b2((rnd() & 1) as u8); SGP.u.foursideCrane.x1.set_b3((rnd() & 1) as u8); SGP.u.foursideCrane.x1.set_b4((rnd() & 1) as u8); SGP.u.foursideCrane.x1.set_b5((rnd() & 1) as u8); SGP.u.foursideCrane.x1.set_b6((rnd() & 1) as u8); SGP.u.foursideCrane.x1.set_b7((rnd() & 1) as u8); SGP.u.foursideCrane.x4 = (rnd() % 320) as i32 - 5; SGP.u.foursideCrane.x8 = (rnd() % 200) as f32 / 10.0 - 10.0; SGP.u.foursideCrane.xC = (rnd() % 200) as f32 / 10.0 - 10.0; SGP.u.foursideCrane.x10 = (rnd() % 200) as f32 / 10.0 - 10.0; SGP.u.foursideCrane.x14 = (rnd() % 30) as f32 / 10.0; SGP.u.foursideCrane.x18 = rw(); SGP.u.foursideCrane.x1C = rw(); m::grFourside_801F3274(&raw mut SGO); }
                    26 => { m::grFourside_801F37F8(&raw mut SGO); }
                    27 => { PTRMODE |= 1; m::grFourside_801F37FC(&raw mut SGO); }
                    28 => { ret = m::grFourside_801F388C(&raw mut SGO) as u32; }
                    29 => { PTRMODE |= 1; SGP.u.foursideUfo.x0 = (rnd() % 5) as u8; SGP.u.foursideUfo.x1 = if rnd() & 3 != 0 { (rnd() % 3) as u8 } else { 0xFF }; SGP.u.foursideUfo.x2 = (rnd() % 8) as u8; SGP.u.foursideUfo.x3 = (rnd() % 3) as u8; SGP.u.foursideUfo.x4 = (rnd() % 80) as i32 - 5; SGP.u.foursideUfo.x8 = (rnd() & 1) as i32; SGP.u.foursideUfo.xC = if rnd() & 1 != 0 { &raw mut CMS } else { null_mut() }; m::grFourside_801F3894(&raw mut SGO); }
                    30 => { m::grFourside_801F3B6C(&raw mut SGO); }
                    31 => { PTRMODE |= 1; SGP.u.foursideUfo.x0 = (rnd() % 5) as u8; SGP.u.foursideUfo.x1 = if rnd() & 3 != 0 { (rnd() % 3) as u8 } else { 0xFF }; SGP.u.foursideUfo.x2 = (rnd() % 8) as u8; SGP.u.foursideUfo.x3 = (rnd() % 3) as u8; SGP.u.foursideUfo.x4 = (rnd() % 80) as i32 - 5; SGP.u.foursideUfo.x8 = (rnd() & 1) as i32; SGP.u.foursideUfo.xC = if rnd() & 1 != 0 { &raw mut CMS } else { null_mut() }; m::grFourside_801F3B70(&raw mut SGO); }
                    32 => { m::grFourside_801F3C40(&raw mut SGO); }
                    33 => { ret = m::grFourside_801F3CC0(&raw mut SGO) as u32; }
                    34 => { SGP.u.fourside2.x0 = (rnd() % 5) as u8; SGP.u.fourside2.x1 = (rnd() % 4) as u8; SGP.u.fourside2.x4 = (rnd() % 900) as i32; SGP.u.fourside2.x8 = (rnd() % 30) as i32; m::grFourside_801F3CC8(&raw mut SGO); }
                    35 => { m::grFourside_801F3F0C(&raw mut SGO); }
                    36 => { MGP[3].u.fourside2.x0 = (rnd() % 5) as u8; MGP[3].u.fourside2.x1 = (rnd() % 3) as u8; MGP[3].u.fourside2.x4 = 695 + (rnd() % 12) as i32; ret = m::grFourside_801F3F10() as u32; }
                    37 => { PTRMODE |= 2; SGP.u.fourside.x0 = if rnd() & 1 != 0 { &raw mut FJ[0] } else { null_mut() }; SGP.u.fourside.x4 = if rnd() & 1 != 0 { &raw mut FJ[1] } else { null_mut() }; SGP.u.fourside.x8 = if rnd() & 1 != 0 { &raw mut FJ[2] } else { null_mut() }; m::fn_801F3F74(&raw mut SGO, (rnd() % 3) as isize); }
                    38 => { ret = pi(m::grFourside_801F41E0(fv() as _)); }
                    39 => { ret = m::grFourside_801F41E8(&raw mut SV, fv() as i32, &raw mut SJO as *mut _) as u32; }
                    40 => { { PTRMODE |= 4; SGP.u.foursideCrane.x1.set_b0((rnd() & 1) as u8); SGP.u.foursideCrane.x1.set_b1((rnd() & 1) as u8); SGP.u.foursideCrane.x1.set_b2((rnd() & 1) as u8); SGP.u.foursideCrane.x1.set_b3((rnd() & 1) as u8); SGP.u.foursideCrane.x1.set_b4((rnd() & 1) as u8); SGP.u.foursideCrane.x1.set_b5((rnd() & 1) as u8); SGP.u.foursideCrane.x1.set_b6((rnd() & 1) as u8); SGP.u.foursideCrane.x1.set_b7((rnd() & 1) as u8); m::grFourside_801F3154(&raw mut SGO); for _ in 0..900 { m::grFourside_801F3274(&raw mut SGO); } } }
                    41 => { { PTRMODE |= 1; m::grFourside_801F37FC(&raw mut SGO); for _ in 0..500 { m::grFourside_801F3894(&raw mut SGO); if rnd() & 7 == 0 { MGP[5].u.fourside2.x8 += 1; } } } }
                    42 => { { m::grFourside_801F3C40(&raw mut SGO); for _ in 0..700 { m::grFourside_801F3CC8(&raw mut SGO); if rnd() & 63 == 0 { MGP[5].u.fourside2.x0 = (rnd() % 3) as u8; MGP[5].u.fourside2.x1 = (rnd() % 3) as u8; } } } }
                    43 => { { for i in 0..7 { g::hash_StageCallbacks(&m::grFs_StageCallbacks[i]); } g::hash_StageData(&m::grFs_StageData); hm(sh(m::grFs_StageData.data1 as *const u8)); hm(m::grFs_StageData.joint_count as u32); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 22000);
    }
}
