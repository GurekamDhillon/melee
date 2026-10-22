//! tu::gmtoygallery vs the C (gen/cref/tu/gmtoygallery.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_modes/gen/cref/tu/run_tu.py gmtoygallery`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_gm_harness;
use melee_modes::tu::gmtoygallery as m;
use melee_types::*;
use tu_gm_harness::*;

#[test]
fn gmtoygallery_matches_c() {
    unsafe {
        m::ext::hook::gm_SetPendingGameMode = Some(|mode| {
            t(1);
            t(mode as u32)
        });
        m::ext::hook::gm_SetNewGameModePending = Some(|| t(2));
    }
    let mut c = Check::new(include_str!("data/tu_gmtoygallery.txt"));
    for k in 0..4u32 {
        case(0, k);
        let mut gs = GameModeState::zeroed();
        unsafe {
            let tb = &m::gm_Mode_ToyGallery_States;
            for e in tb.iter() {
                t(e.id as u32);
                t(e.preload as u32);
                t(e.flags as u32);
                t(e.on_enter.is_some() as u32);
                t(e.info.scene_kind as u32);
                t(!e.info.enter_data.is_null() as u32);
                t(!e.info.exit_data.is_null() as u32);
            }
            t((tb[0].info.exit_data == core::ptr::addr_of_mut!(m::exit_data) as *mut _) as u32);
            t((m::toy_collection_exit_data[0].is_null() && m::toy_collection_exit_data[1].is_null()) as u32);
            fillb(&mut gs);
            (tb[0].on_exit.unwrap())(&mut gs);
        }
        c.line(0, k, 0, 0);
    }
    assert_eq!(c.done(), 4);
}
