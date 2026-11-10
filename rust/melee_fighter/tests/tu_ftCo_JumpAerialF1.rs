//! src/tu/ftCo_JumpAerialF1.rs vs the C (gen/cref/tu/ftCo_JumpAerialF1.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_JumpAerialF1`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_JumpAerialF1 as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

fn is_f(p: &Fighter) -> u32 { (p as *const Fighter == f() as *const Fighter) as u32 }

#[test]
fn ftCo_JumpAerialF1_matches_c() {
    init();
    let x2d0 = Box::leak(Box::new(Fighter_x2D0_t::zeroed()));
    unsafe {
        hk::FighterControls_TapJumpOff = Some(|p| { t(1); t(p as u32); sr(0) as i32 });
        hk::ScriptGame_ModJump = Some(|s, fl, bb| { t(2); t(s as u32); t(fl as u32); t(b(bb)); if sr(1) { bb * 2.0 } else { bb } });
        hk::ScriptGame_CapsMaxJumps = Some(|p| { t(3); t(is_f(p)); (SRV[3] % 7) as i32 });
        hk::Geno_MultiJump = Some(|p, first, ms, vy| { t(4); t(is_f(p)); t(first as u32); t(*ms as u32); t(b(*vy)); if sr(2) { *ms += 1; *vy *= 3.0; } });
        hk::ft_did_jump = Some(|p, a| { t(5); t(is_f(p)); t(a as u32); sr(4) });
        hk::ft_800D2D0C = Some(|g| { t(6); t(is_go(g)); sr(5) });
        hk::ft_800D2E7C = Some(|_, v| { t(7); t(b(v.x)); t(b(v.y)); t(b(v.z)) });
        hk::ftCommon_8007D5D4 = Some(|p| { t(8); p.x1968_jumpsUsed = 1 });
        hk::ftCo_800D72A0 = Some(|_| { t(9); sr(6) });
        hk::ftCo_800D7268 = Some(|_| { t(10); sr(7) as i32 });
        hk::ftCo_800CBAC4 = Some(|g, ms, v, a| { t(11); t(is_go(g)); t(ms as u32); t(b(v.x)); t(b(v.y)); t(b(v.z)); t(a as u32) });
        hk::ft_800CB6EC = Some(|p, a| { t(12); t(is_f(p)); t(a as u32) });
        hk::ftAnim_IsFramesRemaining = Some(|_| { t(13); sr(8) });
        hk::ftCo_FallAerial_Enter = Some(|_| t(14));
        hk::ftCo_Fall_Enter = Some(|_| t(15));
        hk::ftCo_JumpAerial_IASA = Some(|g| { t(16); t(is_go(g)) });
        hk::ft_80084E1C = Some(|_, a, bb, c| { t(17); t(b(a)); t(b(bb)); t(b(c)) });
        hk::ft_80082F28 = Some(|g| { t(18); t(is_go(g)) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_JumpAerialF1.txt"));
    for op in 0..6u32 {
        for k in 0..4000u32 {
            case(op, k);
            *x2d0 = Fighter_x2D0_t::zeroed();
            x2d0.x0 = (rnd() % 20) as i32;
            x2d0.x4 = rg();
            x2d0.x8 = rw();
            x2d0.xC = rw();
            x2d0.x10 = rw();
            for i in 0..5 { x2d0.x14[i] = rw(); }
            x2d0.x28 = (rnd() % 8) as i32 - 1;
            x2d0.x2C = (rnd() % 400) as i32;
            x2d0.x30 = (rnd() % 400) as i32;
            let fr = f();
            fr.x2D0 = x2d0 as *mut _;
            fr.player_idx = (rnd() % 6) as u8;
            fr.x1968_jumpsUsed = (rnd() % 7) as u8;
            fr.motion_id = if rnd() & 3 == 0 { 0x9B } else { (rnd() % 400) as i32 };
            fr.input.lstick[0].x = rg();
            fr.input.lstick[0].y = rg();
            fr.input.held_buttons[0] = rnd() & 0xF00;
            fr.cmd_vars[0] = rnd() & 1;
            fr.co_attrs.max_jumps = (rnd() % 7) as i32;
            fr.co_attrs.air_drift_stick_mul = rw();
            fr.co_attrs.air_drift_max = rw();
            fr.facing_dir = rg();
            unsafe { *(&mut fr.mv as *mut _ as *mut u32) = rnd() };
            cd().tap_jump_threshold = rg();
            cd().x258 = rw();
            let g = go();
            let ret = match op {
                0 => { let a = rnd() & 1 != 0; m::ftCo_800D730C(g, a) as u32 }
                1 => { m::ftCo_800D74A4(g); 0 }
                2 => { m::ftCo_JumpAerialF1_Anim(g); 0 }
                3 => { m::ftCo_JumpAerialF1_IASA(g); 0 }
                4 => { m::ftCo_JumpAerialF1_Phys(g); 0 }
                _ => { m::ftCo_JumpAerialF1_Coll(g); 0 }
            };
            let fr = f();
            let mut h = Hs::new();
            h.m(fr.cmd_vars[0]); h.m(fr.x1968_jumpsUsed as u32);
            h.m(unsafe { *(&fr.mv as *const _ as *const u32) });
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 24000);
}
