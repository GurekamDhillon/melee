//! src/tu/ftCo_DownSpot.rs vs the C (gen/cref/tu/ftCo_DownSpot.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_DownSpot`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
#[macro_use]
mod tu_ftco_dfe;
use tu_ftco_harness::*;
use tu_ftco_dfe::*;
use melee_fighter::types::*;
use melee_fighter::tu::ftCo_DownSpot as m;
use m::ext::hook as hk;

static mut DR: *mut ftData_80085FD4_ret = core::ptr::null_mut();

#[test]
fn ftCo_DownSpot_matches_c() {
    init();
    df_init();
    unsafe {
        DR = Box::leak(Box::new(ftData_80085FD4_ret::zeroed()));
        hk::ftData_80085FD4 = Some(|p, ms| { t(1); t(is_f(p)); t(ms as u32); DR });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(2); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftAnim_8006EBA4 = dv!(3);
        hk::ftCommon_8007E2F4 = Some(|p, v| { t(4); t(is_f(p)); t(v as i32 as u32); });
        hk::ftAnim_IsFramesRemaining = dchk!(5);
        hk::ftCo_80097E8C = dv!(6);
        hk::ft_80084F3C = dv!(7);
        hk::ft_80083F88 = dv!(8);
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_DownSpot.txt"));
    for op in 0..5u32 {
        for k in 0..2000u32 {
            case(op, k);
            df_fill();
            unsafe { (*DR).x14 = SRV[4] & 1 };
            let g = go();
            match op {
                0 => m::ftCo_DownSpot_Enter(g),
                1 => m::ftCo_DownSpot_Anim(g),
                2 => m::ftCo_DownSpot_IASA(g),
                3 => m::ftCo_DownSpot_Phys(g),
                _ => m::ftCo_DownSpot_Coll(g),
            }
            c.line(op, k, 0, df_hash().0);
        }
    }
    assert_eq!(c.done(), 10000);
}
