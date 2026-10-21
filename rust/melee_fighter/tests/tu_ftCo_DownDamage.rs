//! src/tu/ftCo_DownDamage.rs vs the C (gen/cref/tu/ftCo_DownDamage.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_DownDamage`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
#[macro_use]
mod tu_ftco_dfe;
use tu_ftco_harness::*;
use tu_ftco_dfe::*;
use melee_fighter::tu::ftCo_DownDamage as m;
use m::ext::hook as hk;

#[test]
fn ftCo_DownDamage_matches_c() {
    init();
    df_init();
    unsafe {
        hk::ftCo_8008DCE0 = Some(|g, ms, fd| { t(1); t(is_go(g)); t(ms as u32); t(b(fd)); });
        hk::ftCommon_8007E2F4 = Some(|p, v| { t(2); t(is_f(p)); t(v as i32 as u32); });
        hk::ftAnim_IsFramesRemaining = dchk!(3);
        hk::ftCo_80090780 = dv!(4);
        hk::ftCo_Fall_Enter = dv!(5);
        hk::ftCo_80098160 = Some(|g, ms| { t(6); t(is_go(g)); t(ms as u32); });
        hk::ftCo_80097F38 = dv!(7);
        hk::ftCo_Damage_Phys = dv!(8);
        hk::ft_80082708 = Some(|g| { t(9); t(is_go(g)); chk(9) as u32 });
        hk::ftCo_8008FC94 = dv!(10);
        hk::ftCo_800C7CA0 = dchk!(11);
        hk::ft_80081DD4 = dchk!(12);
        hk::ftCommon_8007D7FC = dfv!(13);
        hk::ftCo_800C1D38 = dchk!(14);
        hk::ftCo_800C17CC = dchk!(15);
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_DownDamage.txt"));
    for op in 0..6u32 {
        for k in 0..3000u32 {
            case(op, k);
            df_fill();
            let g = go();
            let ret = match op {
                0 => m::ftCo_8009F0F0(g) as u32,
                1 => { m::ftCo_8009F184(g); 0 }
                2 => { m::ftCo_DownDamage_Anim(g); 0 }
                3 => { m::ftCo_DownDamage_IASA(g); 0 }
                4 => { m::ftCo_DownDamage_Phys(g); 0 }
                _ => { m::ftCo_DownDamage_Coll(g); 0 }
            };
            c.line(op, k, ret, df_hash().0);
        }
    }
    assert_eq!(c.done(), 18000);
}
