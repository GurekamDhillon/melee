//! src/tu/ftCo_ItemParasolFall.rs vs the C (gen/cref/tu/ftCo_ItemParasolFall.c: the whole TU,
//! shipping contraction; regenerate with
//! `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_ItemParasolFall`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_ItemParasolFall as m;
use m::ext::hook as hk;

#[test]
fn ftCo_ItemParasolFall_matches_c() {
    init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(1); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftCo_SpecialAir_CheckInput = Some(|_| { t(2); sr(1) });
        hk::ftCo_80095328 = Some(|_, p| { t(3); t(p.is_null() as u32); sr(2) });
        hk::ftCo_800C3B10 = Some(|_| { t(4); sr(3) });
        hk::ftCo_80099A58 = Some(|_| { t(5); sr(4) });
        hk::ftCo_AttackAir_CheckItemThrowInput = Some(|_| { t(6); sr(5) });
        hk::ft_80087988 = Some(|g| { t(7); t(is_go(g)); sr(6) });
        hk::ftCo_Fall_Enter = Some(|g| { t(8); t(is_go(g)) });
        hk::ftCo_800CB870 = Some(|_| { t(9); sr(7) });
        hk::ftCo_ItemParasol_Phys = Some(|g| { t(10); t(is_go(g)) });
        hk::ftCo_Landing_Enter_Basic = Some(|g| { t(11); t(is_go(g)) });
        hk::ft_8008370C = Some(|g, cb| {
            t(12); t(is_go(g)); t((cb as usize == m::ext::ftCo_Landing_Enter_Basic as usize) as u32); cb(g)
        });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_ItemParasolFall.txt"));
    for op in 0..5u32 {
        for k in 0..2000u32 {
            case(op, k);
            f().input.lstick[0].y = rg(); cd().close_parasol_threshold = rg();
            let g = go();
            match op {
                0 => m::ftCo_800CF280(g),
                1 => m::ftCo_ItemParasolFall_Anim(g),
                2 => m::ftCo_ItemParasolFall_IASA(g),
                3 => m::ftCo_ItemParasolFall_Phys(g),
                _ => m::ftCo_ItemParasolFall_Coll(g),
            }
            c.line(op, k, 0, f().motion_id as u32);
        }
    }
    assert_eq!(c.done(), 10000);
}
