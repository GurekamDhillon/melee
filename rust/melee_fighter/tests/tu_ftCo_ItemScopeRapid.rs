//! src/tu/ftCo_ItemScopeRapid.rs vs the C (gen/cref/tu/ftCo_ItemScopeRapid.c: the whole TU,
//! shipping contraction; regenerate with
//! `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_ItemScopeRapid`). The C's GC-offset
//! accesses (`fp + 0x2340`, `p_ftCommonData + 0x5BC`, `gobj + 0x2C`) are the named fields here.
#![allow(static_mut_refs, non_snake_case)]
#[path = "../../melee_core/test_support/fnsym.rs"]
mod fnsym;
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_ItemScopeRapid as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

#[test]
fn ftCo_ItemScopeRapid_matches_c() {
    init();
    unsafe {
        hk::fn_800D769C = Some(|p, ms| { t(1); t((p as *const Fighter == f() as *const Fighter) as u32); t(ms as u32); if sr(0) { ms + 8 } else { ms } });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| { t(2); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32) });
        hk::ftCommon_ClampAirDrift = Some(|_| t(3));
        hk::ftCommon_8007D7FC = Some(|_| t(4));
        hk::ftCommon_8007D5D4 = Some(|_| t(5));
        hk::fn_800D8378 = Some(|_| t(6));
        hk::ft_800892A0 = Some(|_| t(7));
        hk::ft_80089824 = Some(|_| t(8));
        hk::fn_800D86E0 = Some(|_| t(9));
        hk::fn_800D874C = Some(|_| t(10));
        hk::it_8026B594 = Some(|g| { t(11); t((g == igo()) as u32); sr(1) });
        hk::ftCo_800CDE54 = Some(|_| t(12));
        hk::ftCo_800CDE74 = Some(|_| t(13));
        hk::ft_800841B8 = Some(|g, cb| { t(14); t((fnsym::same(cb as usize, m::fn_800D7D70 as usize)) as u32); cb(g) });
        hk::ft_80082C74 = Some(|g, cb| { t(15); t((fnsym::same(cb as usize, m::fn_800D7CEC as usize)) as u32); cb(g) });
        hk::it_80291DAC = Some(|_, n| { t(16); t(n as u32); if sr(2) { 7 } else { -1 } });
        hk::fn_800D84D4 = Some(|_, n| { t(17); t(n as u32) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_ItemScopeRapid.txt"));
    for op in 0..13u32 {
        for k in 0..2000u32 {
            case(op, k);
            let fr = f();
            fr.item_gobj = if rnd() % 8 != 0 { igo() } else { core::ptr::null_mut() };
            fr.motion_id = (158 + rnd() % 16) as i32;
            fr.ground_or_air = rnd() & 1;
            fr.input.pressed_buttons = rnd() & 0x100;
            fr.cur_anim_frame = rg() * 2.0;
            fr.frame_speed_mul = rg() * 2.0;
            fr.mv.co.common.x0 = (rnd() % 3) as i32;
            cd().x5BC = (rnd() % 100) as i32;
            let g = go();
            match op {
                0 => m::fn_800D7BDC(g),
                1 => m::fn_800D7C60(g),
                2 => m::fn_800D7CEC(g),
                3 => m::fn_800D7D70(g),
                4 => m::ftCo_ItemScopeRapid_Anim(g),
                5 => m::ftCo_ItemScopeAirRapid_Anim(g),
                6 => m::ftCo_ItemScopeRapid_IASA(g),
                7 => m::ftCo_ItemScopeAirRapid_IASA(g),
                8 => m::ftCo_ItemScopeRapid_Phys(g),
                9 => m::ftCo_ItemScopeAirRapid_Phys(g),
                10 => m::ftCo_ItemScopeRapid_Coll(g),
                11 => m::ftCo_ItemScopeAirRapid_Coll(g),
                _ => m::fn_800D80F4(g),
            }
            let fr = f();
            let mut h = Hs::new();
            h.m(match fr.accessory4_cb { None => 0, Some(p) if fnsym::same(p as usize, m::fn_800D80F4_c as usize) => 1, _ => 9 });
            h.m(match fr.take_dmg_cb { None => 0, Some(p) if fnsym::same(p as usize, m::fn_800D8378_c as usize) => 2, _ => 9 });
            h.m(unsafe { fr.mv.co.common.x0 } as u32);
            c.line(op, k, 0, h.0);
        }
    }
    assert_eq!(c.done(), 26000);
}
