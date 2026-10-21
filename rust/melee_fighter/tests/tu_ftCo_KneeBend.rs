//! src/tu/ftCo_KneeBend.rs vs the C (gen/cref/tu/ftCo_KneeBend.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_KneeBend`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_KneeBend as m;
use m::ext::hook as hk;

#[test]
fn ftCo_KneeBend_matches_c() {
    init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(1); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftAnim_IsFramesRemaining = Some(|g| { t(2); t(is_go(g)); sr(2) });
        hk::ft_800D2D0C = Some(|g| { t(3); t(is_go(g)); sr(3) });
        hk::ftCo_ItemScrew_Enter = Some(|g| { t(4); t(is_go(g)) });
        hk::ftCo_Jump_Enter = Some(|g| { t(5); t(is_go(g)) });
        hk::ftCo_Attack100_CheckInput = Some(|g| { t(6); t(is_go(g)); sr(6) & sr(22) });
        hk::ftCo_Catch_CheckInput = Some(|g| { t(7); t(is_go(g)); sr(7) & sr(23) });
        hk::ftCo_AttackHi4_CheckInputNoD0 = Some(|g| { t(8); t(is_go(g)); sr(8) & sr(24) });
        hk::ft_80084F3C = Some(|g| { t(9); t(is_go(g)) });
        hk::ft_80083F88 = Some(|g| { t(10); t(is_go(g)) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_KneeBend.txt"));
    for op in 0..6u32 {
        for k in 0..2000u32 {
            case(op, k);
            let fr = f();
            fr.cur_anim_frame = rw(); fr.co_attrs.jump_startup_time = rw();
            fr.input.held_buttons[0] = rnd() & 0xC00;
            fr.input.lstick[0].y = rg(); fr.input.cstick[0].y = rg();
            cd().tap_jump_release_threshold = rg();
            unsafe {
                fr.mv.co.kneebend.jump_input = rnd() % 5;
                fr.mv.co.kneebend.is_short_hop = (rnd() & 1) as i32;
            }
            let ji = rnd() % 5;
            let g = go();
            match op {
                0 => m::ftCo_KneeBend_Enter(g, ji),
                1 => m::ftCo_KneeBend_Anim(g),
                2 => m::ftCo_KneeBend_Check_ShortHop(g),
                3 => m::ftCo_KneeBend_IASA(g),
                4 => m::ftCo_KneeBend_Phys(g),
                _ => m::ftCo_KneeBend_Coll(g),
            }
            let fr = f();
            let mut h = Hs::new();
            unsafe { h.m(fr.mv.co.kneebend.jump_input); h.m(fr.mv.co.kneebend.is_short_hop as u32); }
            c.line(op, k, 0, h.0);
        }
    }
    assert_eq!(c.done(), 12000);
}
