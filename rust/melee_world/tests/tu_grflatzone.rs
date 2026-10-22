//! Written by gen/mtu/mk_stage_driver.py from the same description as gen/mtu/tu/grflatzone.c.
//! src/tu/grflatzone.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_world/gen/mtu/run_tu.py grflatzone`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables)]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_world::tu::grflatzone as m;
use mtu_harness::*;
#[path = "mtu/grflatzone_gen.rs"]
mod g;

static mut SGO: HSD_GObj = unsafe { core::mem::zeroed() };
static mut SGP: Ground = unsafe { core::mem::zeroed() };
static mut SJO: [u8; 16] = [0; 16];
static mut YPB: [u8; 64] = [0; 64];
static mut SV: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
static mut SJOB: HSD_JObj = unsafe { core::mem::zeroed() };
static mut YP: m::grFlatzone_YakumonoParam = unsafe { core::mem::zeroed() };
static mut MGO: [HSD_GObj; 10] = unsafe { core::mem::zeroed() };
static mut IGO: HSD_GObj = unsafe { core::mem::zeroed() };
static mut MGP: [Ground; 10] = unsafe { core::mem::zeroed() };
static mut MJO: [HSD_JObj; 10] = unsafe { core::mem::zeroed() };
static mut ITD: Item = unsafe { core::mem::zeroed() };
static mut DYN: [grDynamicAttr_UnkStruct; 2] = unsafe { core::mem::zeroed() };




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
    IGO = core::mem::zeroed();
    MGP = core::mem::zeroed();
    MJO = core::mem::zeroed();
    ITD = core::mem::zeroed();
    DYN = core::mem::zeroed();
    g::fill_HSD_JObj(&mut SJOB);
    SGO.hsd_obj = &raw mut SJOB as *mut _;
    g::fill_grFlatzone_YakumonoParam(&mut YP);
    YP.unk0 = (rnd() % 60) as i32;
    YP.unk4 = (rnd() % 60) as i32;
    YP.unk8 = (rnd() % 60) as i32;
    YP.unkC = (rnd() % 60) as i32;
    YP.unk10 = (rnd() % 100) as i32;
    YP.unk14 = 1 + (rnd() % 6) as i32;
    YP.unk18 = (rnd() % 80) as i32 - 40;
    YP.unk1C = (rnd() % 80) as i32 - 40;
    YP.unk20 = (rnd() % 50) as i32;
    YP.unk2C = 1 + (rnd() % 5) as i32;
    YP.unk30 = (rnd() % 10) as i32;
    YP.unk34 = (rnd() % 10) as i32;
    YP.unk3C = (rnd() % 80) as i32;
    YP.unk24 = rw();
    YP.unk28 = rw();
    YP.unk38 = rw();
    m::yakumono_param = &raw mut YP;
    g::fill_HSD_GObj(&mut IGO);
    IGO.user_data = if rnd() & 3 == 0 { null_mut() } else { &raw mut ITD as *mut _ };
    g::fill_CollData(&mut ITD.x378_itemColl);
    for i in 0..2 {
        g::fill_grDynamicAttr_UnkStruct(&mut DYN[i]);
    }
    for i in 0..10 {
        g::fill_HSD_GObj(&mut MGO[i]);
        g::fill_Ground(&mut MGP[i]);
        g::fill_HSD_JObj(&mut MJO[i]);
        MGO[i].user_data = if rnd() & 7 == 0 { null_mut() } else { &raw mut MGP[i] as *mut _ };
        MGO[i].hsd_obj = if rnd() & 7 == 0 { null_mut() } else { &raw mut MJO[i] as *mut _ };
    }
    n = rnd() % 4;
    SGP.u.flatzone2.xD0 = if n == 0 { 2 } else if n == 1 { 1 } else { (rnd() % 7) as i32 - 1 };
    SGP.u.flatzone2.timer = (rnd() % 45) as i32 - 3;
    SGP.u.flatzone2.xC8 = if rnd() & 1 != 0 {
        if rnd() & 1 != 0 { 1.0 } else { -1.0 }
    } else {
        rw()
    };
    SGP.u.flatzone2.xCC = null_mut();

}

unsafe fn hash_all() {
    for i in 0..10 {
        hm(pib(MGP[i].u.flatzone2.xCC));
        MGP[i].u.flatzone2.xCC = null_mut();
    }
    hm(pib(SGP.u.flatzone2.xCC));
    SGP.u.flatzone2.xCC = null_mut();
    g::hash_HSD_GObj(&SGO);
    g::hash_Ground(&SGP);
    g::hash_StageInfo(&m::ext::stage_info);
    hm(SV.x.to_bits());
    hm(SV.y.to_bits());
    hm(SV.z.to_bits());
    g::hash_HSD_JObj(&SJOB);
    g::hash_grFlatzone_YakumonoParam(&YP);
    hm(pi(m::yakumono_param));
    for i in 0..10 {
        g::hash_HSD_GObj(&MGO[i]);
        g::hash_Ground(&MGP[i]);
        g::hash_HSD_JObj(&MJO[i]);
    }
    g::hash_HSD_GObj(&IGO);
    g::hash_CollData(&ITD.x378_itemColl);
    for i in 0..2 {
        g::hash_grDynamicAttr_UnkStruct(&DYN[i]);
    }

}

#[test]
fn grflatzone_matches_c() {
    unsafe {
        g::install();
        preg(&raw const SGO, core::mem::size_of::<HSD_GObj>());
        preg(&raw const SGP, core::mem::size_of::<Ground>());
        preg(&raw const SJO, 16);
        preg(&raw const YPB, 64);
        preg(&raw const SV, core::mem::size_of::<Vec3>());
        preg(&raw const SJOB, core::mem::size_of::<HSD_JObj>());
        preg(&raw const YP, core::mem::size_of::<m::grFlatzone_YakumonoParam>());
        preg(&raw const MGO, core::mem::size_of_val(&MGO));
        preg(&raw const IGO, core::mem::size_of::<HSD_GObj>());
        preg(&raw const MGP, core::mem::size_of_val(&MGP));
        preg(&raw const MJO, core::mem::size_of_val(&MJO));
        preg(&raw const ITD, 16);
        preg(&raw const DYN, core::mem::size_of_val(&DYN));
        preg(&raw const m::grFz_StageCallbacks, core::mem::size_of_val(&m::grFz_StageCallbacks));
        preg(&raw const m::grFz_StageData, core::mem::size_of_val(&m::grFz_StageData));
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_grflatzone.txt"));
        for op in 0..56u32 {
            for k in 0..500u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                    0 => { m::grFlatzone_80216E74(fv() as i32); }
                    1 => { m::grFlatzone_80216E78(); }
                    2 => { m::grFlatzone_UnkStage0_OnLoad(); }
                    3 => { m::grFlatzone_UnkStage0_OnStart(); }
                    4 => { ret = m::grFlatzone_80216F40() as u32; }
                    5 => { ret = pi(m::grFlatzone_80216F48((rnd() % 9) as i32)); }
                    6 => { m::stageGObj0_OnInit(&raw mut SGO); }
                    7 => { ret = m::grFlatzone_8021705C(&raw mut SGO) as u32; }
                    8 => { m::grFlatzone_80217064(&raw mut SGO); }
                    9 => { m::grFlatzone_80217068(&raw mut SGO); }
                    10 => { m::fn_8021706C(&raw mut SGO); }
                    11 => { m::grFlatzone_8021707C(&raw mut SGO); }
                    12 => { ret = m::grFlatzone_802171CC(&raw mut SGO) as u32; }
                    13 => { m::grFlatzone_802171D4(&raw mut SGO); }
                    14 => { m::grFlatzone_802173D0(&raw mut SGO); }
                    15 => { m::grFlatzone_802173D4(&raw mut SGO); }
                    16 => { ret = m::grFlatzone_80217430(&raw mut SGO) as u32; }
                    17 => { m::grFlatzone_80217438(&raw mut SGO); }
                    18 => { m::grFlatzone_8021743C(&raw mut SGO); }
                    19 => { m::fn_80217440(&raw mut SGO); }
                    20 => { m::grFlatzone_80217450(&raw mut SGO); }
                    21 => { ret = m::grFlatzone_802174E4(&raw mut SGO) as u32; }
                    22 => { SGP.u.flatzone3.xC7 = if rnd() % 5 == 4 { 8 } else { (rnd() % 4) as u8 }; SGP.u.flatzone3.xC9 = (rnd() % 5) as u8; m::grFlatzone_802174EC(&raw mut SGO); }
                    23 => { m::grFlatzone_80217644(&raw mut SGO); }
                    24 => { m::grFlatzone_80217648(&raw mut SGO); }
                    25 => { ret = m::grFlatzone_802176B4(&raw mut SGO) as u32; }
                    26 => { SGP.u.flatzone.xC4 = (rnd() & 1) as u8; SGP.u.flatzone2.xCC = if rnd() & 1 != 0 { &raw mut DYN[0] } else { null_mut() }; m::grFlatzone_802176BC(&raw mut SGO); }
                    27 => { m::grFlatzone_80217CD0(&raw mut SGO); }
                    28 => { m::grFlatzone_80217CD4(&raw mut SGO); }
                    29 => { ret = m::grFlatzone_80217D28(&raw mut SGO) as u32; }
                    30 => { m::grFlatzone_80217D30(&raw mut SGO); }
                    31 => { m::grFlatzone_80217DA4(&raw mut SGO); }
                    32 => { m::grFlatzone_80217DA8(&raw mut SGO); }
                    33 => { ret = m::grFlatzone_80217DFC(&raw mut SGO) as u32; }
                    34 => { m::grFlatzone_80217E04(&raw mut SGO); }
                    35 => { m::grFlatzone_80217E78(&raw mut SGO); }
                    36 => { m::grFlatzone_80217E7C(&raw mut SGO); }
                    37 => { ret = m::grFlatzone_80217E80(&raw mut SGO) as u32; }
                    38 => { m::grFlatzone_80217E88(&raw mut SGO); }
                    39 => { m::grFlatzone_80217E8C(&raw mut SGO); }
                    40 => { m::grFlatzone_80217E90(&raw mut SGO); }
                    41 => { ret = m::grFlatzone_80217EE8(&raw mut SGO) as u32; }
                    42 => { m::grFlatzone_80217EF0(&raw mut SGO); }
                    43 => { m::grFlatzone_8021805C(&raw mut SGO); }
                    44 => { m::grFlatzone_80218060((rnd() & 1) as i32); }
                    45 => { m::grFlatzone_802181B4(); }
                    46 => { m::grFlatzone_80218260(); }
                    47 => { ret = pi(m::grFlatzone_802182B4(fv() as _)); }
                    48 => { ret = m::grFlatzone_802182BC(&raw mut SV, fv() as i32, &raw mut SJO as *mut _) as u32; }
                    49 => { SGP.u.flatzone2.xCC = if rnd() & 1 != 0 { &raw mut DYN[0] } else { null_mut() }; SGP.u.flatzone.xC4 = 0; SGP.u.flatzone2.xD0 = 1; SGP.u.flatzone2.timer = YP.unk2C * (rnd() % 5) as i32 + 1; SGP.u.flatzone2.xC8 = rw(); m::grFlatzone_802176BC(&raw mut SGO); }
                    50 => { SGP.u.flatzone2.xCC = if rnd() & 1 != 0 { &raw mut DYN[0] } else { null_mut() }; SGP.u.flatzone.xC4 = 0; SGP.u.flatzone2.xD0 = 2; SGP.u.flatzone2.xC8 = rw(); m::grFlatzone_802176BC(&raw mut SGO); }
                    51 => { { m::grFlatzone_80217648(&raw mut SGO); SGP.u.flatzone2.xCC = if rnd() & 1 != 0 { &raw mut DYN[0] } else { null_mut() }; SGP.u.flatzone.xC4 = 1; for _ in 0..90 { m::grFlatzone_802176BC(&raw mut SGO); if rnd() & 31 == 0 { SGP.u.flatzone.xC4 = 1; } } } }
                    52 => { { m::grFlatzone_80217450(&raw mut SGO); SGP.u.flatzone3.xC4 = 0; SGP.u.flatzone3.xC7 = (rnd() % 4) as u8; SGP.u.flatzone3.xC9 = (1 + rnd() % 4) as u8; for _ in 0..60 { m::grFlatzone_802174EC(&raw mut SGO); if rnd() & 15 == 0 { SGP.u.flatzone3.xC7 = (rnd() % 4) as u8; } } } }
                    53 => { { m::grFlatzone_80217E90(&raw mut SGO); SGP.u.flatzone3.xC4 = 1; for _ in 0..80 { m::grFlatzone_80217EF0(&raw mut SGO); } } }
                    54 => { { m::grFlatzone_8021707C(&raw mut SGO); SGP.u.flatzone.xC4 = 0; for _ in 0..200 { m::grFlatzone_802171D4(&raw mut SGO); } } }
                    55 => { { for i in 0..9 { g::hash_StageCallbacks(&m::grFz_StageCallbacks[i]); } g::hash_StageData(&m::grFz_StageData); hm(sh(m::grFz_StageData.data1 as *const u8)); hm(m::grFz_StageData.joint_count as u32); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 28000);
    }
}
