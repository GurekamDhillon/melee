//! src/tu/ftCo_ItemParasolFallSpecial.rs vs the C (gen/cref/tu/ftCo_ItemParasolFallSpecial.c: the
//! whole TU, shipping contraction; regenerate with
//! `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_ItemParasolFallSpecial`).
#![allow(static_mut_refs, non_snake_case)]
#[path = "../../melee_core/test_support/fnsym.rs"]
mod fnsym;
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_ItemParasolFallSpecial as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

#[test]
fn ftCo_ItemParasolFallSpecial_matches_c() {
    init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(1); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ft_80087988 = Some(|g| { t(2); t(is_go(g)); sr(1) });
        hk::ftCo_800968C8 = Some(|g| { t(3); t(is_go(g)) });
        hk::ftCo_800CB870 = Some(|_| { t(4); sr(2) });
        hk::ftCo_ItemParasol_Phys = Some(|g| { t(5); t(is_go(g)) });
        hk::ftCo_LandingFallSpecial_Enter_Basic = Some(|g| { t(6); t(is_go(g)) });
        hk::ft_8008370C = Some(|g, cb| {
            t(7); t(is_go(g)); t((fnsym::same(cb as usize, m::ext::ftCo_LandingFallSpecial_Enter_Basic as usize)) as u32); cb(g)
        });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_ItemParasolFallSpecial.txt"));
    for op in 0..5u32 {
        for k in 0..2000u32 {
            case(op, k);
            let fr = f();
            fr.input.lstick[0].y = rg(); cd().close_parasol_threshold = rg();
            fr.kind = if rnd() & 1 != 0 { Ft_Kind_Peach } else { (rnd() % 30) as _ };
            unsafe { fr.u.pe.parasol_gobj_0 = if rnd() & 1 != 0 { igo() as *mut Item_GObj } else { core::ptr::null_mut() } };
            let g = go();
            match op {
                0 => m::ftCo_800CF3C8(g),
                1 => m::ftCo_ItemParasolFallSpecial_Anim(g),
                2 => m::ftCo_ItemParasolFallSpecial_IASA(g),
                3 => m::ftCo_ItemParasolFallSpecial_Phys(g),
                _ => m::ftCo_ItemParasolFallSpecial_Coll(g),
            }
            c.line(op, k, 0, f().motion_id as u32);
        }
    }
    assert_eq!(c.done(), 10000);
}
