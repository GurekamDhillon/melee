//! src/tu/gmgover.rs vs the C (gen/cref/tu/gmgover.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_modes/gen/cref/run_tu.py gmgover`).
#![allow(static_mut_refs, non_snake_case)]
mod mtu_harness;
use melee_modes::tu::gmgover as m;
use melee_types::*;
use mtu_harness::*;
#[path = "mtu/gmgover_gen.rs"]
mod g;

static mut ST: GameModeState = unsafe { core::mem::zeroed() };
static mut EXB: [i8; 1] = [0];

#[test]
fn gmgover_matches_c() {
    unsafe {
        g::install();
        preg(&raw const ST, core::mem::size_of::<GameModeState>());
        preg(&raw const EXB, 1);
        preg(&raw const m::gm_8049C178, 16);
        preg(&raw const m::gm_804D6920, core::mem::size_of_val(&m::gm_804D6920));
        preg(&raw const m::gm_Mode_GOver_States, core::mem::size_of_val(&m::gm_Mode_GOver_States));
        preg(&raw const m::gm_Mode_DebugGOver_States, core::mem::size_of_val(&m::gm_Mode_DebugGOver_States));
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_gmgover.txt"));
        for op in 0..16u32 {
            for k in 0..1000u32 {
                case(op, k);
                ST = core::mem::zeroed();
                for i in 0..16 {
                    m::gm_8049C178[i] = fv() as u8;
                }
                EXB[0] = fv() as i8;
                ST.info.exit_data = &raw mut EXB as *mut _;
                let mut ret = 0u32;
                match op {
                    0 => m::gm_801BEE9C(&raw mut ST),
                    1 => m::gm_801BEF84(&raw mut ST),
                    2 => m::gm_801BEFA4(fv() as i32),
                    3 => m::gm_801BEFC0(fv() as i32),
                    4 => m::gm_801BEFE0(fv() as i8),
                    5 => m::gm_801BF000(fv() as i8),
                    6 => m::gm_801BF020(fv() as i8),
                    7 => m::gm_801BF040(fv() as i8),
                    8 => ret = m::gm_801BEFB0(),
                    9 => ret = m::gm_801BEFD0() as u32,
                    10 => ret = m::gm_801BEFF0() as u32,
                    11 => ret = m::gm_801BF010() as u32,
                    12 => ret = m::gm_801BF030() as u32,
                    13 => ret = m::gm_801BF050() as u32,
                    14 => {
                        for i in 0..5 {
                            g::hash_GameModeState(&m::gm_Mode_GOver_States[i]);
                        }
                    }
                    _ => {
                        for i in 0..3 {
                            g::hash_GameModeState(&m::gm_Mode_DebugGOver_States[i]);
                        }
                    }
                }
                for i in 0..16 {
                    hm(m::gm_8049C178[i] as u32);
                }
                hm(EXB[0] as i32 as u32);
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 16000);
    }
}
