//! src/tu/ftCo_DamageFall.rs vs the C (gen/cref/tu/ftCo_DamageFall.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_DamageFall`).
//! Covers the functions new in that file; the re-exported leaves are checked by ftdmgfall_ref.
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_DamageFall as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

fn is_f(p: *const Fighter) -> u32 { (p == f() as *const Fighter) as u32 }
fn s8(i: usize) -> bool { unsafe { SRV[i] & 7 == 0 } }

#[test]
fn ftCo_DamageFall_matches_c() {
    init();
    unsafe {
        hk::ftCommon_8007D5D4 = Some(|fr| { t(1); t(is_f(fr)) });
        hk::ftGetParasolStatus = Some(|g| { t(2); t(is_go(g)); if s8(1) { (SRV[2] % 3) as i32 } else { -1 } });
        hk::ftCo_800CF4DC = Some(|g| { t(3); t(is_go(g)) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(4); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftCommon_ClampAirDrift = Some(|fr| { t(5); t(is_f(fr)) });
        hk::ftCommon_8007EBAC = Some(|fr, a, bb| { t(6); t(is_f(fr)); t(a); t(bb) });
        hk::Replay_UcfVersion = Some(|p| { t(7); t(p as u32); if SRV[3] & 1 != 0 { 84 } else { 83 } });
        hk::ftUcf_PadRaw = Some(|p, w, bk| { t(8); t(p as u32); t(w as u32); t(bk as u32); (rnd() % 256) as i32 - 128 });
        hk::ftCo_800C5240 = Some(|g| { t(9); t(is_go(g)); s8(4) });
        hk::ftCo_SpecialAir_CheckInput = Some(|g| { t(10); t(is_go(g)); s8(5) });
        hk::ftCo_80095328 = Some(|g, o| { t(11); t(is_go(g)); t(o.is_null() as u32); s8(6) });
        hk::ftCo_800D7100 = Some(|g| { t(12); t(is_go(g)); s8(7) });
        hk::ftCo_800C3B10 = Some(|g| { t(13); t(is_go(g)); s8(8) });
        hk::ftCo_AttackAir_CheckItemThrowInput = Some(|g| { t(14); t(is_go(g)); s8(9) });
        hk::ftCo_800D705C = Some(|g| { t(15); t(is_go(g)); s8(10) });
        hk::ftCo_800CB870 = Some(|g| { t(16); t(is_go(g)); s8(11) });
        hk::ftCo_Fall_Enter = Some(|g| { t(17); t(is_go(g)) });
        hk::ftCo_800C5DDC = Some(|g| { t(18); t(is_go(g)); s8(12) });
        hk::ftCo_800C5CD4 = Some(|g| { t(19); t(is_go(g)); s8(13) });
        hk::ft_8008370C = Some(|g, cb| { t(20); t(is_go(g)); t((cb as usize == m::ext::ftCo_80090984 as usize) as u32) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_DamageFall.txt"));
    for op in 0..4u32 {
        for k in 0..2000u32 {
            case(op, k);
            let fr = f();
            fr.ground_or_air = (rnd() & 1) as _;
            fr.x618_player_id = (rnd() % 4) as u8;
            fr.input.lstick[1].x = rg();
            fr.input.lstick[0].x = rg();
            cd().x210 = rg();
            cd().x214 = (rnd() % 10) as i32;
            fr.active_timer.lstick.x = (rnd() % 12) as _;
            let g = go();
            let ret = match op {
                0 => { m::ftCo_80090780(g); 0 }
                1 => m::ftCo_Ucf084Tumble(f()) as u32,
                2 => { m::ftCo_DamageFall_IASA(g); 0 }
                _ => { m::ftCo_DamageFall_Coll(g); 0 }
            };
            c.line(op, k, ret, Hs::new().0);
        }
    }
    assert_eq!(c.done(), 8000);
}
