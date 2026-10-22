//! src/tu/gmhanyucss.rs vs the C (gen/cref/tu/gmhanyucss.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_modes/gen/cref/run_tu.py gmhanyucss`).
#![allow(static_mut_refs, non_snake_case)]
mod mtu_harness;
use melee_modes::tu::gmhanyucss as m;
use melee_types::*;
use mtu_harness::*;
#[path = "mtu/gmhanyucss_gen.rs"]
mod g;

pub static mut VSD: VsModeData = unsafe { core::mem::zeroed() };
static mut ST: GameModeState = unsafe { core::mem::zeroed() };

#[test]
fn gmhanyucss_matches_c() {
    unsafe {
        g::install();
        preg(&raw const ST, core::mem::size_of::<GameModeState>());
        preg(&raw const m::gm_8049BEE8, core::mem::size_of::<CSSData>());
        preg(&raw const VSD, core::mem::size_of::<VsModeData>());
        preg(&raw const m::gm_Mode_HanyuCss_States, core::mem::size_of_val(&m::gm_Mode_HanyuCss_States));
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_gmhanyucss.txt"));
        for op in 0..3u32 {
            for k in 0..1000u32 {
                case(op, k);
                ST = core::mem::zeroed();
                m::gm_8049BEE8 = core::mem::zeroed();
                VSD = core::mem::zeroed();
                ST.info.enter_data = &raw mut m::gm_8049BEE8 as *mut _;
                g::fill_CSSData(&mut m::gm_8049BEE8);
                g::fill_VsModeData(&mut VSD);
                let ret = 0u32;
                match op {
                    0 => m::gm_801BED3C(&raw mut ST),
                    1 => m::gm_801BEDA8(&raw mut ST),
                    _ => {
                        for i in 0..2 {
                            g::hash_GameModeState(&m::gm_Mode_HanyuCss_States[i]);
                        }
                    }
                }
                g::hash_CSSData(&m::gm_8049BEE8);
                g::hash_VsModeData(&VSD);
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 3000);
    }
}

