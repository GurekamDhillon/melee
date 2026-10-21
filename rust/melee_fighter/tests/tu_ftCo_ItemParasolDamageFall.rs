//! src/tu/ftCo_ItemParasolDamageFall.rs vs the C (gen/cref/tu/ftCo_ItemParasolDamageFall.c: the
//! whole TU, shipping contraction; regenerate with
//! `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_ItemParasolDamageFall`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_ItemParasolDamageFall as m;
use m::ext::hook as hk;

#[test]
fn ftCo_ItemParasolDamageFall_matches_c() {
    init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(1); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftCommon_ClampAirDrift = Some(|fr| { t(2); t(core::ptr::eq(fr, f()) as u32) });
        hk::ftCo_DamageFall_IASA = Some(|g| { t(3); t(is_go(g)) });
        hk::ft_80084DB0 = Some(|g| { t(4); t(is_go(g)) });
        hk::ftCo_80090984 = Some(|g| { t(5); t(is_go(g)) });
        hk::ft_8008370C = Some(|g, cb| {
            t(6); t(is_go(g)); t((cb as usize == m::ext::ftCo_80090984 as usize) as u32); cb(g)
        });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_ItemParasolDamageFall.txt"));
    for op in 0..5u32 {
        for k in 0..500u32 {
            case(op, k);
            let g = go();
            match op {
                0 => m::ftCo_800CF4DC(g),
                1 => m::ftCo_ItemParasolDamageFall_Anim(g),
                2 => m::ftCo_ItemParasolDamageFall_IASA(g),
                3 => m::ftCo_ItemParasolDamageFall_Phys(g),
                _ => m::ftCo_ItemParasolDamageFall_Coll(g),
            }
            c.line(op, k, 0, f().motion_id as u32);
        }
    }
    assert_eq!(c.done(), 2500);
}
