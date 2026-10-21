//! src/tu/ftCo_Furafura.rs vs the C (gen/cref/tu/ftCo_Furafura.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_Furafura`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_Furafura as m;
use m::ext::hook as hk;

#[test]
fn ftCo_Furafura_matches_c() {
    init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| { t(1); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32) });
        hk::ftCommon_InitGrab = Some(|fp, a, tm| { t(2); t((fp as *mut _ == F) as u32); t(a as u32); t(b(tm)) });
        hk::ftCommon_8007EBAC = Some(|fp, a, bb| { t(3); t((fp as *mut _ == F) as u32); t(a); t(bb) });
        hk::ft_800885A8 = Some(|fp, s, v, p| { t(4); t((fp as *mut _ == F) as u32); t(s as u32); t(v as u32); t(p as u32) });
        hk::ftCommon_GrabMash = Some(|fp, v| { t(5); t((fp as *mut _ == F) as u32); t(b(v)); t(b(fp.grab_timer)); sr(1) });
        hk::ft_8008A2BC = Some(|g| { t(6); t(is_go(g)) });
        hk::ft_80084F3C = Some(|g| { t(7); t(is_go(g)) });
        hk::ft_80083F88 = Some(|g| { t(8); t(is_go(g)) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_Furafura.txt"));
    for op in 0..5u32 {
        for k in 0..2000u32 {
            case(op, k);
            let d = cd();
            d.x280_unkShieldHealth = rw(); d.x2F8 = rw(); d.x2FC = rw(); d.x300 = rg(); d.x304 = rg();
            let fr = f();
            fr.dmg.x1830_percent = rw(); fr.grab_timer = rg(); fr.shield_health = rg();
            let g = go();
            match op {
                0 => m::ftCo_80099010(g),
                1 => m::ftCo_Furafura_Anim(g),
                2 => m::ftCo_Furafura_IASA(g),
                3 => m::ftCo_Furafura_Phys(g),
                _ => m::ftCo_Furafura_Coll(g),
            }
            let mut h = Hs::new();
            h.m(b(f().shield_health)); h.m(b(f().grab_timer));
            c.line(op, k, 0, h.0);
        }
    }
    assert_eq!(c.done(), 10000);
}
