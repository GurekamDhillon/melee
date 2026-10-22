//! tu::gmtoylottery vs the C (gen/cref/tu/gmtoylottery.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_modes/gen/cref/tu/run_tu.py gmtoylottery`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_gm_harness;
use melee_modes::tu::gmtoylottery as m;
use melee_types::*;
use tu_gm_harness::*;

#[test]
fn gmtoylottery_matches_c() {
    unsafe {
        let h = &mut m::ext::hook::gm_SetPendingGameMode;
        *h = Some(|mode| {
            t(1);
            t(mode as u32)
        });
        m::ext::hook::gm_SetNewGameModePending = Some(|| t(2));
        m::ext::hook::lbCardNew_AllocWorkArea = Some(|| t(3));
        m::ext::hook::lbCardGame_LoadArchive = Some(|a| {
            t(4);
            t(a as u32)
        });
        m::ext::hook::gm_80172898 = Some(|a| {
            t(5);
            t(a as u32);
            7
        });
        m::ext::hook::gm_80173754 = Some(|a, b| {
            t(6);
            t(a as u32);
            t(b as u32);
            sr(3)
        });
    }
    let mut c = Check::new(include_str!("data/tu_gmtoylottery.txt"));
    for op in 0..2u32 {
        for k in 0..40u32 {
            case(op, k);
            let mut gs = GameModeState::zeroed();
            unsafe {
                let tb = &m::gm_Mode_ToyLottery_States;
                if op == 0 && k == 0 {
                    for e in tb.iter() {
                        t(e.id as u32);
                        t(e.preload as u32);
                        t(e.flags as u32);
                        t(e.on_enter.is_some() as u32);
                        t(e.on_exit.is_some() as u32);
                        t(e.info.scene_kind as u32);
                        t(!e.info.enter_data.is_null() as u32);
                        t(!e.info.exit_data.is_null() as u32);
                    }
                    t((tb[0].info.exit_data == core::ptr::addr_of_mut!(m::exit_data) as *mut _) as u32);
                }
                fillb(&mut gs);
                if op == 0 {
                    (tb[0].on_enter.unwrap())(&mut gs)
                } else {
                    (tb[0].on_exit.unwrap())(&mut gs)
                }
            }
            c.line(op, k, 0, 0);
        }
    }
    assert_eq!(c.done(), 80);
}
