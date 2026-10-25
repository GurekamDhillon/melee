//! tu::gmscenelaunch vs the C (gen/cref/tu/gmscenelaunch.c: gmscenelaunch.h included verbatim into a
//! wrapper TU, shipping contraction; regenerate with
//! `python rust/melee_modes/gen/cref/run_tu.py gmscenelaunch`).
#![allow(static_mut_refs, non_snake_case)]
mod mtu_harness;
use melee_modes::tu::gmscenelaunch as m;
use melee_types::*;
use mtu_harness::*;
#[path = "mtu/gmscenelaunch_gen.rs"]
mod g;

pub static mut VSD: VsModeData = unsafe { core::mem::zeroed() };
pub static mut PCS: PreloadedGameModeState = unsafe { core::mem::zeroed() };
pub static mut GR: GameRules = unsafe { core::mem::zeroed() };
pub static mut GP: GamePrefs = unsafe { core::mem::zeroed() };

#[test]
fn gmscenelaunch_matches_c() {
    unsafe {
        g::install();
        preg(&raw const VSD, core::mem::size_of::<VsModeData>());
        preg(&raw const PCS, core::mem::size_of::<PreloadedGameModeState>());
        preg(&raw const GR, core::mem::size_of::<GameRules>());
        preg(&raw const GP, core::mem::size_of::<GamePrefs>());
        preg(&raw const m::ext::gm_80473814, core::mem::size_of::<CssSubStruct>());
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_gmscenelaunch.txt"));
        for op in 0..3u32 {
            for k in 0..3000u32 {
                case(op, k);
                g::fill_VsModeData(&mut VSD);
                g::fill_PreloadedGameModeState(&mut PCS);
                g::fill_GameRules(&mut GR);
                g::fill_GamePrefs(&mut GP);
                g::fill_CssSubStruct(&mut m::ext::gm_80473814);
                let mut ret = 0u32;
                match op {
                    0 => ret = m::SceneLaunch_SeedPlayers(&raw mut VSD, (k & 1) as i32) as u32,
                    1 => ret = m::SceneLaunch_SeedVs(&raw mut VSD, ((k >> 1) & 1) as i32) as u32,
                    _ => m::SceneLaunch_ApplyItemFrequency(&raw mut VSD.start.rules, &raw mut GP),
                }
                g::hash_VsModeData(&VSD);
                g::hash_PreloadedGameModeState(&PCS);
                g::hash_GameRules(&GR);
                g::hash_GamePrefs(&GP);
                g::hash_CssSubStruct(&m::ext::gm_80473814);
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 9000);
    }
}
