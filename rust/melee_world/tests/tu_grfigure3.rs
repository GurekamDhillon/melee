//! Written by gen/mtu/mk_stage_driver.py from the same description as gen/mtu/tu/grfigure3.c.
//! src/tu/grfigure3.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_world/gen/mtu/run_tu.py grfigure3`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables)]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_world::tu::grfigure3 as m;
use mtu_harness::*;
#[path = "mtu/grfigure3_gen.rs"]
mod g;

static mut SGO: HSD_GObj = unsafe { core::mem::zeroed() };
static mut SGP: Ground = unsafe { core::mem::zeroed() };
static mut SJO: [u8; 16] = [0; 16];
static mut YPB: [u8; 64] = [0; 64];
static mut SV: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };




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
    m::yakumono_param = if rnd() & 1 != 0 { &raw mut YPB as *mut _ } else { null_mut() };

}

unsafe fn hash_all() {
    g::hash_HSD_GObj(&SGO);
    g::hash_Ground(&SGP);
    g::hash_StageInfo(&m::ext::stage_info);
    hm(SV.x.to_bits());
    hm(SV.y.to_bits());
    hm(SV.z.to_bits());
    hm(pi(m::yakumono_param));

}

#[test]
fn grfigure3_matches_c() {
    unsafe {
        g::install();
        preg(&raw const SGO, core::mem::size_of::<HSD_GObj>());
        preg(&raw const SGP, core::mem::size_of::<Ground>());
        preg(&raw const SJO, 16);
        preg(&raw const YPB, 64);
        preg(&raw const SV, core::mem::size_of::<Vec3>());
        preg(&raw const m::grEF2_StageCallbacks, core::mem::size_of_val(&m::grEF2_StageCallbacks));
        preg(&raw const m::grEF3_StageData, core::mem::size_of_val(&m::grEF3_StageData));
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_grfigure3.txt"));
        for op in 0..21u32 {
            for k in 0..500u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                    0 => { m::grFigure3_8020E504(fv() as i32); }
                    1 => { m::grFigure3_8020E508(); }
                    2 => { m::grFigure3_OnLoad(); }
                    3 => { m::grFigure3_OnStart(); }
                    4 => { ret = m::grFigure3_8020E5A0() as u32; }
                    5 => { ret = pi(m::grFigure3_8020E5A8((rnd() % 3) as i32)); }
                    6 => { m::stageGObj0_OnInit(&raw mut SGO); }
                    7 => { ret = m::grFigure3_8020E6BC(&raw mut SGO) as u32; }
                    8 => { m::grFigure3_8020E6C4(&raw mut SGO); }
                    9 => { m::grFigure3_8020E6C8(&raw mut SGO); }
                    10 => { m::grFigure3_8020E6CC(&raw mut SGO); }
                    11 => { ret = m::grFigure3_8020E6F8(&raw mut SGO) as u32; }
                    12 => { m::grFigure3_8020E700(&raw mut SGO); }
                    13 => { m::grFigure3_8020E724(&raw mut SGO); }
                    14 => { m::stageGObj2_OnInit(&raw mut SGO); }
                    15 => { ret = m::grFigure3_8020E778(&raw mut SGO) as u32; }
                    16 => { m::grFigure3_8020E780(&raw mut SGO); }
                    17 => { m::grFigure3_8020E784(&raw mut SGO); }
                    18 => { ret = pi(m::grFigure3_8020E788(fv() as _)); }
                    19 => { ret = m::grFigure3_8020E790(&raw mut SV, fv() as i32, &raw mut SJO as *mut _) as u32; }
                    20 => { { for i in 0..3 { g::hash_StageCallbacks(&m::grEF2_StageCallbacks[i]); } g::hash_StageData(&m::grEF3_StageData); hm(sh(m::grEF3_StageData.data1 as *const u8)); hm(m::grEF3_StageData.joint_count as u32); } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 10500);
    }
}
