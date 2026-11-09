//! src/tu/ftCo_ItemScope.rs vs the C (gen/cref/tu/ftCo_ItemScope.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_ItemScope`).
#![allow(static_mut_refs, non_snake_case)]
#[path = "../../melee_core/test_support/fnsym.rs"]
mod fnsym;
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_ItemScope as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

#[test]
fn ftCo_ItemScope_matches_c() {
    init();
    unsafe {
        hk::fn_800D769C = Some(|p, ms| { t(1); t((p as *const Fighter == f() as *const Fighter) as u32); t(ms as u32); if sr(0) { ms + 8 } else { ms } });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| { t(2); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32) });
        hk::ftCommon_ClampAirDrift = Some(|_| t(3));
        hk::ftCommon_8007D7FC = Some(|_| t(4));
        hk::ftCommon_8007D5D4 = Some(|_| t(5));
        hk::fn_800D8378 = Some(|_| t(6));
        hk::ftCo_ItemScopeFire_Anim = Some(|g| { t(7); t(is_go(g)) });
        hk::ftCo_800CDE54 = Some(|_| t(8));
        hk::ftCo_800CDE74 = Some(|_| t(9));
        hk::ft_800841B8 = Some(|g, cb| { t(10); t((fnsym::same(cb as usize, m::fn_800D8838 as usize)) as u32); cb(g) });
        hk::ft_80082C74 = Some(|g, cb| { t(11); t((fnsym::same(cb as usize, m::fn_800D87C0 as usize)) as u32); cb(g) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_ItemScope.txt"));
    for op in 0..12u32 {
        for k in 0..2000u32 {
            case(op, k);
            let fr = f();
            fr.motion_id = (158 + rnd() % 16) as i32;
            fr.cur_anim_frame = rw();
            fr.frame_speed_mul = rw();
            let g = go();
            match op {
                0 => m::fn_800D86E0(g),
                1 => m::fn_800D874C(g),
                2 => m::fn_800D87C0(g),
                3 => m::fn_800D8838(g),
                4 => m::ftCo_ItemScopeEnd_Anim(g),
                5 => m::ftCo_ItemScopeAirEnd_Anim(g),
                6 => m::ftCo_ItemScopeEnd_IASA(g),
                7 => m::ftCo_ItemScopeAirEnd_IASA(g),
                8 => m::ftCo_ItemScopeEnd_Phys(g),
                9 => m::ftCo_ItemScopeAirEnd_Phys(g),
                10 => m::ftCo_ItemScopeEnd_Coll(g),
                _ => m::ftCo_ItemScopeAirEnd_Coll(g),
            }
            let fr = f();
            let mut h = Hs::new();
            h.m(match fr.take_dmg_cb { None => 0, Some(p) if fnsym::same(p as usize, m::fn_800D8378_c as usize) => 2, _ => 9 });
            c.line(op, k, 0, h.0);
        }
    }
    assert_eq!(c.done(), 24000);
}
