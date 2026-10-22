//! src/tu/gmgiant.rs vs the C (gen/cref/tu/gmgiant.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_modes/gen/cref/run_tu.py gmgiant`).
#![allow(static_mut_refs, non_snake_case)]
mod mtu_harness;
use melee_modes::tu::gmgiant as m;
use melee_types::*;
use mtu_harness::*;
#[path = "mtu/gmgiant_gen.rs"]
mod g;

static mut ST: GameModeState = unsafe { core::mem::zeroed() };
static mut GM: gmm_x0 = unsafe { core::mem::zeroed() };
static mut PID: PlayerInitData = unsafe { core::mem::zeroed() };
static mut PID2: PlayerInitData = unsafe { core::mem::zeroed() };

fn reg<T>(p: *const T) {
    preg(p, core::mem::size_of::<T>());
}

#[test]
fn gmgiant_matches_c() {
    unsafe {
        g::install();
        reg(&raw const ST);
        reg(&raw const GM);
        reg(&raw const PID);
        reg(&raw const PID2);
        reg(&raw const m::gm_Mode_GiantVs_States);
        reg(&raw const m::ext::gmVsMelee_ApproachData);
        reg(&raw const m::ext::gmVsMelee_CssData);
        reg(&raw const m::ext::gmVsMelee_ResultsEnterData);
        reg(&raw const m::ext::gmVsMelee_SssData);
        reg(&raw const m::ext::gmVsMelee_StartData);
        reg(&raw const m::ext::gmVsMelee_SuddenDeathExitInfo);
        reg(&raw const m::ext::gmVsMelee_VsExitInfo);
        reg(&raw const m::ext::if_Scene_Prize_EnterData);
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_gmgiant.txt"));
        for op in 0..14u32 {
            for k in 0..300u32 {
                case(op, k);
                ST = core::mem::zeroed();
                PID = core::mem::zeroed();
                PID2 = core::mem::zeroed();
                m::ext::gmMainLib_804D3EE0 = &raw mut GM;
                g::fill_PlayerInitData(&mut PID);
                match op {
                    0 => m::gm_801B8FB8(&raw mut ST),
                    1 => m::gm_801B8FE4(&raw mut ST),
                    2 => m::gm_801B900C(&raw mut ST),
                    3 => m::gm_801B9034(&raw mut ST),
                    4 => m::gm_801B9084(&raw mut ST),
                    5 => m::gm_801B90B8(&raw mut ST),
                    6 => m::gm_801B90E0(&raw mut ST),
                    7 => m::gm_801B9114(&raw mut ST),
                    8 => m::gm_801B9134(&raw mut ST),
                    9 => m::gm_801B9154(&raw mut ST),
                    10 => m::fn_801B9060(&raw mut PID, &raw mut PID2),
                    11 => m::gm_Mode_GiantVs_OnInit(),
                    12 => m::gm_Mode_GiantVs_OnLoad(),
                    _ => {
                        for i in 0..9 {
                            g::hash_GameModeState(&m::gm_Mode_GiantVs_States[i]);
                        }
                    }
                }
                g::hash_PlayerInitData(&PID);
                g::hash_PlayerInitData(&PID2);
                c.line(op, k, 0);
            }
        }
        assert_eq!(c.done(), 14 * 300);
    }
}
