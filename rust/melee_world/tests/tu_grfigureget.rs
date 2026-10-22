//! Written by gen/mtu/mk_stage_driver.py from the same description as gen/mtu/tu/grfigureget.c.
//! src/tu/grfigureget.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_world/gen/mtu/run_tu.py grfigureget`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables)]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_world::tu::grfigureget as m;
use mtu_harness::*;
#[path = "mtu/grfigureget_gen.rs"]
mod g;

static mut SGO: HSD_GObj = unsafe { core::mem::zeroed() };
static mut SGP: Ground = unsafe { core::mem::zeroed() };
static mut SJO: [u8; 16] = [0; 16];
static mut YPB: [u8; 64] = [0; 64];
static mut SV: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
static mut FIG: [HSD_GObj; 3] = unsafe { core::mem::zeroed() };
static mut PLINK: [*mut HSD_GObj; 16] = [null_mut(); 16];
static mut YP: m::grFigureGet_Params = m::grFigureGet_Params { x0: 0, x4: 0, x8: 0, xC: 0.0, x10: 0.0, x14: 0.0 };
static mut FCD: CollData = unsafe { core::mem::zeroed() };
static mut CD: CollData = unsafe { core::mem::zeroed() };
static mut IG: [HSD_GObj; 3] = unsafe { core::mem::zeroed() };
static mut CGO: HSD_GObj = unsafe { core::mem::zeroed() };
static mut ITD: Item = unsafe { core::mem::zeroed() };




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
    PLINK = [null_mut(); 16];
    FIG = core::mem::zeroed();
    for i in 0..3 {
        g::fill_HSD_GObj(&mut FIG[i]);
    }
    m::ext::HSD_GObjPLinkHead = PLINK.as_mut_ptr();
    n = rnd() % 4;
    for i in 0..n as usize {
        FIG[i].next = if i + 1 < n as usize { &raw mut FIG[i + 1] } else { null_mut() };
    }
    PLINK[HSD_GOBJ_PLINK_FIGHTER as usize] = if n != 0 { &raw mut FIG[0] } else { null_mut() };
    YP = core::mem::zeroed();
    FCD = core::mem::zeroed();
    CD = core::mem::zeroed();
    IG = core::mem::zeroed();
    CGO = core::mem::zeroed();
    ITD = core::mem::zeroed();
    g::fill_grFigureGet_Params(&mut YP);
    YP.x8 = (rnd() % 4) as i32;
    YP.x4 = if rnd() & 1 != 0 { 0 } else { (rnd() % 40) as i32 };
    YP.x0 = (rnd() % 30) as i32;
    YP.xC = 0.0;
    YP.x10 = (rnd() % 200) as f32;
    YP.x14 = rw();
    g::fill_CollData(&mut FCD);
    g::fill_CollData(&mut CD);
    g::fill_HSD_GObj(&mut CGO);
    for i in 0..3 {
        g::fill_HSD_GObj(&mut IG[i]);
    }
    m::yakumono_param = &raw mut YP;
    SGP.u.figureget.x4 = (rnd() % 3) as i32;
    SGP.u.figureget.x8 = (rnd() % 4) as i32;
    SGP.u.figureget.xC = (rnd() % 4) as i32;
    SGP.u.figureget.x0 = (rnd() % 4) as i32;
    SGP.u.figureget.x28[0] = if rnd() & 3 == 0 { &raw mut CGO } else { null_mut() };
    CD.x0_gobj = if rnd() & 3 == 0 { null_mut() } else { &raw mut CGO };
    CGO.classifier = if rnd() & 3 != 0 { HSD_GOBJ_CLASS_ITEM as u16 } else { fv() as u16 };
    CGO.user_data = &raw mut ITD as *mut _;
    ITD.kind = if rnd() & 3 != 0 { It_Kind_Coin } else { fv() as i32 };
    CD.x34_flags.set_b1234(if rnd() & 3 != 0 { 5 } else { fv() as u8 });

}

unsafe fn hash_all() {
    for i in 0..3 {
        hm(pib(SGP.u.figureget.x28[i])); SGP.u.figureget.x28[i] = null_mut();
        hm(pib(SGP.u.figureget.x34[i])); SGP.u.figureget.x34[i] = null_mut();
    }
    g::hash_HSD_GObj(&SGO);
    g::hash_Ground(&SGP);
    g::hash_StageInfo(&m::ext::stage_info);
    hm(SV.x.to_bits());
    hm(SV.y.to_bits());
    hm(SV.z.to_bits());
    for i in 0..3 {
        g::hash_HSD_GObj(&FIG[i]);
    }
    g::hash_grFigureGet_Params(&YP);
    g::hash_CollData(&FCD);
    g::hash_CollData(&CD);
    g::hash_HSD_GObj(&CGO);
    for i in 0..3 {
        g::hash_HSD_GObj(&IG[i]);
    }
    hm(pi(m::yakumono_param));

}

#[test]
fn grfigureget_matches_c() {
    unsafe {
        g::install();
        preg(&raw const SGO, core::mem::size_of::<HSD_GObj>());
        preg(&raw const SGP, core::mem::size_of::<Ground>());
        preg(&raw const SJO, 16);
        preg(&raw const YPB, 64);
        preg(&raw const SV, core::mem::size_of::<Vec3>());
        preg(&raw const FIG, core::mem::size_of_val(&FIG));
        preg(&raw const PLINK, core::mem::size_of_val(&PLINK));
        preg(&raw const YP, core::mem::size_of::<m::grFigureGet_Params>());
        preg(&raw const FCD, core::mem::size_of::<CollData>());
        preg(&raw const CD, core::mem::size_of::<CollData>());
        preg(&raw const IG, core::mem::size_of_val(&IG));
        preg(&raw const CGO, core::mem::size_of::<HSD_GObj>());
        preg(&raw const ITD, 16);
        preg(&raw const m::grFigureGet_StageCallbacks, core::mem::size_of_val(&m::grFigureGet_StageCallbacks));
        preg(&raw const m::grFigureGet_StageData, core::mem::size_of_val(&m::grFigureGet_StageData));
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_grfigureget.txt"));
        for op in 0..21u32 {
            for k in 0..500u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                    0 => { m::grFigureGet_OnDemoInit(fv() as i32); }
                    1 => { m::grFigureGet_OnInit(); }
                    2 => { m::grFigureGet_OnLoad(); }
                    3 => { m::grFigureGet_OnStart(); }
                    4 => { ret = m::grFigureGet_802195C4() as u32; }
                    5 => { ret = pi(m::grFigureGet_802195CC((rnd() % 2) as i32)); }
                    6 => { m::stageGObj0_OnInit(&raw mut SGO); }
                    7 => { ret = m::grFigureGet_802196E0(&raw mut SGO) as u32; }
                    8 => { m::grFigureGet_802196E8(&raw mut SGO); }
                    9 => { m::grFigureGet_802196EC(&raw mut SGO); }
                    10 => { m::grFigureGet_802196F0(&raw mut SGO); }
                    11 => { ret = m::grFigureGet_80219890(&raw mut SGO) as u32; }
                    12 => { m::grFigureGet_80219898(&raw mut SGO); }
                    13 => { m::grFigureGet_80219B0C(&raw mut SGO); }
                    14 => { { let j = fv() as i32; let x = fv() as i32; let d = rw(); m::grFigureGet_80219B10(&raw mut SGP as *mut _, j, &raw mut CD, x, fv() as _, d); } }
                    15 => { { let g = if rnd() & 3 == 0 { null_mut() } else { &raw mut SGO }; m::grFigureGet_80219C34(g); } }
                    16 => { { let g = if rnd() & 3 == 0 { null_mut() } else { &raw mut SGO }; ret = m::grFigureGet_80219C50(g) as u32; } }
                    17 => { ret = pi(m::grFigureGet_OnTouchLine(fv() as _)); }
                    18 => { ret = m::grFigureGet_OnCheckShadowRender(&raw mut SV, fv() as i32, &raw mut SJO as *mut _) as u32; }
                    19 => { { for _ in 0..12 { m::grFigureGet_80219898(&raw mut SGO); CD.cur_pos.x = rw(); if rnd() & 1 != 0 { let a = fv() as i32; m::grFigureGet_80219B10(&raw mut SGP as *mut _, 0, &raw mut CD, a, 0, 0.0); } } } }
                    20 => { { for i in 0..2 { g::hash_StageCallbacks(&m::grFigureGet_StageCallbacks[i]); } g::hash_StageData(&m::grFigureGet_StageData); hm(sh(m::grFigureGet_StageData.data1 as *const u8)); hm(m::grFigureGet_StageData.joint_count as u32); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 10500);
    }
}
