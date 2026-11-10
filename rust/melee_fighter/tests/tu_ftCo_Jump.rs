//! src/tu/ftCo_Jump.rs vs the C (gen/cref/tu/ftCo_Jump.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_Jump`).
#![allow(static_mut_refs, non_snake_case)]
#[path = "../../melee_core/test_support/fnsym.rs"]
mod fnsym;
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_Jump as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

#[test]
fn ftCo_Jump_matches_c() {
    init();
    let fd = Box::leak(Box::new(ftData::zeroed()));
    let sfx = Box::leak(Box::new(FtSFX::zeroed()));
    let x197c = Box::leak(Box::new(HSD_GObj::zeroed())) as *mut HSD_GObj;
    unsafe {
        hk::FighterControls_TapJumpOff = Some(|p| { t(1); t(p as u32); sr(0) as i32 });
        hk::ScriptGame_ModJump = Some(|s, fl, bb| { t(2); t(s as u32); t(fl as u32); t(b(bb)); if sr(1) { bb * 2.0 } else { bb } });
        hk::ftCo_800C5240 = Some(|g| { t(3); t(is_go(g)); sr(2) });
        hk::ftCo_800C5A50 = Some(|g| { t(4); t(is_go(g)); sr(3) });
        hk::ftCo_KneeBend_Enter = Some(|g, i| { t(5); t(is_go(g)); t(i as u32) });
        hk::ftCo_800DF910 = Some(|fp| { t(6); t((fp as *mut Fighter == f() as *mut Fighter) as u32); sr(4) });
        hk::ft_800881D8 = Some(|fp, id, v, p| { t(7); t((fp as *mut Fighter == f() as *mut Fighter) as u32); t(id as u32); t(v as u32); t(p as u32) });
        hk::ft_PlaySFX = Some(|fp, id, v, p| { t(8); t((fp as *mut Fighter == f() as *mut Fighter) as u32); t(id as u32); t(v as u32); t(p as u32) });
        hk::ftCommon_8007D5D4 = Some(|fp| { t(9); t((fp as *mut Fighter == f() as *mut Fighter) as u32) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| { t(10); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32) });
        hk::ftAnim_IsFramesRemaining = Some(|g| { t(11); t(is_go(g)); sr(5) });
        hk::ftCo_Fall_Enter = Some(|g| { t(12); t(is_go(g)) });
        hk::ftCo_SpecialAir_CheckInput = Some(|_| { t(13); sr(6) });
        hk::ftCo_80095328 = Some(|_, p| { t(14); t(p.is_null() as u32); sr(7) });
        hk::ftCo_800D7100 = Some(|_| { t(15); sr(8) });
        hk::ftCo_800C3B10 = Some(|_| { t(16); sr(9) });
        hk::ftCo_80099A58 = Some(|_| { t(17); sr(10) });
        hk::ftCo_AttackAir_CheckItemThrowInput = Some(|_| { t(18); sr(11) });
        hk::ftCo_800D705C = Some(|_| { t(19); sr(12) });
        hk::ftCo_800CEE70 = Some(|_| { t(20); sr(13) });
        hk::ftPe_8011BA54 = Some(|_| { t(21); sr(14) });
        hk::ftCo_800CB870 = Some(|_| { t(22); sr(15) });
        hk::ftPe_8011BAD8 = Some(|_| { t(23); sr(16) });
        hk::ft_80084DB0 = Some(|g| { t(24); t(is_go(g)) });
        hk::ftCo_80096CC8 = Some(|_, _| { t(25); false });
        hk::ft_80082B1C = Some(|_| { t(26) });
        hk::ft_800835B0 = Some(|g, a, bb| { t(27); t(is_go(g)); t((fnsym::same(a as usize, m::ext::ftCo_80096CC8 as usize)) as u32); t((fnsym::same(bb as usize, m::ext::ft_80082B1C as usize)) as u32) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_Jump.txt"));
    for op in 0..11u32 {
        for k in 0..2000u32 {
            case(op, k);
            *fd = ftData::zeroed();
            *sfx = FtSFX::zeroed();
            sfx.x10 = (rnd() % 1000) as i32;
            fd.x4C_sfx = sfx as *mut _;
            let fr = f();
            fr.ft_data = fd as *mut _;
            fr.x197C = if rnd() & 1 != 0 { x197c } else { core::ptr::null_mut() };
            fr.player_idx = (rnd() % 6) as u8;
            fr.input.lstick[0].x = rg();
            fr.input.lstick[0].y = rg();
            fr.active_timer.lstick.y = (rnd() % 12) as u8;
            fr.input.pressed_buttons = rnd() & 0xF00;
            let cdd = cd();
            cdd.tap_jump_threshold = rg();
            cdd.tap_jump_window = (rnd() % 12) as i32;
            cdd.relaxed_tap_jump_threshold = rg();
            cdd.x438 = rg();
            cdd.x78 = rg();
            fr.co_attrs.ground_to_air_jump_momentum_multiplier = rw();
            fr.co_attrs.jump_h_initial_velocity = rw();
            fr.co_attrs.hop_v_initial_velocity = rw();
            fr.co_attrs.jump_v_initial_velocity = rw();
            fr.co_attrs.jump_h_max_velocity = rw();
            unsafe {
                fr.mv.co.jump.x0 = (rnd() & 1) as i32;
                fr.mv.co.jump.x4 = (rnd() & 1) as i32;
            }
            fr.self_vel.x = rw();
            fr.self_vel.y = rw();
            fr.self_vel.z = rw();
            fr.facing_dir = rg();
            let g = go();
            let ret = match op {
                0 => m::ftCo_Jump_GetInput(g) as u32,
                1 => m::ftCo_Jump_CheckInput(g) as u32,
                2 => m::fn_800CAF78(g) as u32,
                3 => m::ftCo_800CB024(g) as u32,
                4 => { let a = rnd() & 1 != 0; let mm = rw(); m::ftCo_800CB110(g, a, mm); 0 }
                5 => { m::ftCo_Jump_Enter(g); 0 }
                6 => { m::ftCo_Jump_Anim(g); 0 }
                7 => { m::ftCo_Jump_IASA(g); 0 }
                8 => { m::ftCo_Jump_Phys_Inner(g); 0 }
                9 => { m::ftCo_Jump_Phys(g); 0 }
                _ => { m::ftCo_Jump_Coll(g); 0 }
            };
            let fr = f();
            let mut h = Hs::new();
            h.m(b(fr.self_vel.x)); h.m(b(fr.self_vel.y)); h.m(b(fr.self_vel.z));
            unsafe { h.m(fr.mv.co.jump.x0 as u32); h.m(fr.mv.co.jump.x4 as u32); h.m(b(fr.mv.co.jump.jump_mul)); }
            h.m(fr.active_timer.lstick.y as u32); h.m(fr.x2227_b0() as u32);
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 22000);
}
