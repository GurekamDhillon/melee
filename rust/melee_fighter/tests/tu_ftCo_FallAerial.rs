//! src/tu/ftCo_FallAerial.rs vs the C (gen/cref/tu/ftCo_FallAerial.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_FallAerial`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
#[macro_use]
mod tu_ftco_dfe;
use tu_ftco_harness::*;
use tu_ftco_dfe::*;
use melee_fighter::tu::ftCo_FallAerial as m;
use m::ext::hook as hk;

#[test]
fn ftCo_FallAerial_matches_c() {
    init();
    df_init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(1); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftCo_Fall_Anim_Inner = Some(|g, p, a, bb, c| {
            t(2); t(is_go(g));
            t((p == core::ptr::addr_of_mut!(f().mv.co.fallaerial.x4)) as u32);
            t(a as u32); t(bb as u32); t(c as u32);
            *p = rg();
        });
        hk::ftCo_800CC988 = Some(|g, a| { t(3); t(is_go(g)); t(b(a)); });
        hk::ftCo_Fall_IASA_Inner = dchk!(4);
        hk::ft_80084DB0 = dv!(5);
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_FallAerial.txt"));
    for op in 0..4u32 {
        for k in 0..2000u32 {
            case(op, k);
            df_fill();
            let g = go();
            match op {
                0 => m::ftCo_FallAerial_Enter(g),
                1 => m::ftCo_FallAerial_Anim(g),
                2 => m::ftCo_FallAerial_IASA(g),
                _ => m::ftCo_FallAerial_Phys(g),
            }
            c.line(op, k, 0, df_hash().0);
        }
    }
    assert_eq!(c.done(), 8000);
}
