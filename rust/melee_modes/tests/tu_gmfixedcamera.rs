//! src/tu/gmfixedcamera.rs vs the C (gen/cref/tu/gmfixedcamera.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_modes/gen/cref/run_tu.py gmfixedcamera`).
#![allow(static_mut_refs, non_snake_case)]
mod mtu_harness;
use melee_modes::tu::gmfixedcamera as m;
use melee_types::*;
use mtu_harness::*;
#[path = "mtu/gmfixedcamera_gen.rs"]
mod g;

static mut ST: GameModeState = unsafe { core::mem::zeroed() };
static mut GM: gmm_x0 = unsafe { core::mem::zeroed() };
static mut SMD: StartMeleeData = unsafe { core::mem::zeroed() };
static mut SMD2: StartMeleeData = unsafe { core::mem::zeroed() };

fn reg<T>(p: *const T) {
    preg(p, core::mem::size_of::<T>());
}

#[test]
fn gmfixedcamera_matches_c() {
    unsafe {
        g::install();
        reg(&raw const ST);
        reg(&raw const GM);
        reg(&raw const SMD);
        reg(&raw const SMD2);
        reg(&raw const m::gm_Mode_CameraVs_States);
        reg(&raw const m::ext::gmVsMelee_ApproachData);
        reg(&raw const m::ext::gmVsMelee_CssData);
        reg(&raw const m::ext::gmVsMelee_ResultsEnterData);
        reg(&raw const m::ext::gmVsMelee_SssData);
        reg(&raw const m::ext::gmVsMelee_StartData);
        reg(&raw const m::ext::gmVsMelee_SuddenDeathExitInfo);
        reg(&raw const m::ext::gmVsMelee_VsExitInfo);
        reg(&raw const m::ext::if_Scene_Prize_EnterData);
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_gmfixedcamera.txt"));
        for op in 0..14u32 {
            for k in 0..300u32 {
                case(op, k);
                ST = core::mem::zeroed();
                SMD = core::mem::zeroed();
                SMD2 = core::mem::zeroed();
                m::ext::gmMainLib_804D3EE0 = &raw mut GM;
                g::fill_StartMeleeData(&mut SMD);
                match op {
                    0 => m::gm_801B9F10(&raw mut ST),
                    1 => m::gm_801B9F3C(&raw mut ST),
                    2 => m::gm_801B9F64(&raw mut ST),
                    3 => m::gm_801B9F8C(&raw mut ST),
                    4 => m::gm_801B9FC8(&raw mut ST),
                    5 => m::gm_801B9FFC(&raw mut ST),
                    6 => m::gm_801BA024(&raw mut ST),
                    7 => m::gm_801BA058(&raw mut ST),
                    8 => m::gm_801BA078(&raw mut ST),
                    9 => m::gm_801BA098(&raw mut ST),
                    10 => m::fn_801B9FB8(&raw mut SMD, &raw mut SMD2),
                    11 => m::gm_Mode_CameraVs_OnInit(),
                    12 => m::gm_Mode_CameraVs_OnLoad(),
                    _ => {
                        for i in 0..9 {
                            g::hash_GameModeState(&m::gm_Mode_CameraVs_States[i]);
                        }
                    }
                }
                g::hash_StartMeleeData(&SMD);
                g::hash_StartMeleeData(&SMD2);
                c.line(op, k, 0);
            }
        }
        assert_eq!(c.done(), 14 * 300);
    }
}
