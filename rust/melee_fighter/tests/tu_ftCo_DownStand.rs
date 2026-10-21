//! src/tu/ftCo_DownStand.rs vs the C (gen/cref/tu/ftCo_DownStand.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_DownStand`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
#[macro_use]
mod tu_ftco_dfe;
use tu_ftco_harness::*;
use tu_ftco_dfe::*;
use melee_fighter::tu::ftCo_DownStand as m;
use m::ext::hook as hk;

#[test]
fn ftCo_DownStand_matches_c() {
    init();
    df_init();
    unsafe {
        hk::ftCo_GetLStickAngle = Some(|p| { t(1); t(is_f(p)); rg() });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(2); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftAnim_IsFramesRemaining = dchk!(3);
        hk::ft_8008A2BC = dv!(4);
        hk::ft_80084F3C = dv!(5);
        hk::ft_80083F88 = dv!(6);
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_DownStand.txt"));
    for op in 0..6u32 {
        for k in 0..3000u32 {
            case(op, k);
            df_fill();
            let ms = (rnd() % 400) as i32;
            let g = go();
            let ret = match op {
                0 => m::ftCo_800980BC(g) as u32,
                1 => { m::ftCo_80098160(g, ms); 0 }
                2 => { m::ftCo_DownStand_Anim(g); 0 }
                3 => { m::ftCo_DownStand_IASA(g); 0 }
                4 => { m::ftCo_DownStand_Phys(g); 0 }
                _ => { m::ftCo_DownStand_Coll(g); 0 }
            };
            c.line(op, k, ret, df_hash().0);
        }
    }
    assert_eq!(c.done(), 18000);
}
