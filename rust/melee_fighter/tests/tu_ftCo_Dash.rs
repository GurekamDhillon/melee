//! src/tu/ftCo_Dash.rs vs the C (gen/cref/tu/ftCo_Dash.c, shipping contraction; regenerate with
//! `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_Dash`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_Dash as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

static mut FR: f32 = 0.0;
static mut MV: [f32; 2] = [0.0; 2];
static mut NMV: usize = 0;

fn isf(fr: &Fighter) -> u32 { (fr as *const Fighter == unsafe { F } as *const Fighter) as u32 }

#[test]
fn ftCo_Dash_matches_c() {
    init();
    unsafe {
        hk::ScriptGame_CapsActionAllowed = Some(|fr, fl| { t(1); t(isf(fr)); t(fl as u32); sr(20) as i32 });
        hk::ScriptGame_ModValue = Some(|s, fl, bs| { t(2); t(s as u32); t(fl as u32); t(b(bs)); let v = MV[NMV & 1]; NMV += 1; v });
        hk::ftCo_Turn_Enter_Smash = Some(|g| { t(3); t(is_go(g)) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(4); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftAnim_8006EBA4 = Some(|g| { t(5); t(is_go(g)) });
        hk::ftCommon_800804A0 = Some(|fr, v| { t(6); t(isf(fr)); t(b(v)) });
        hk::ft_PlaySFX = Some(|fr, id, a, bb| { t(7); t(isf(fr)); t(id as u32); t(a as u32); t(bb as u32) });
        hk::ftAnim_IsFramesRemaining = Some(|_| { t(8); sr(3) });
        hk::ft_8008A2BC = Some(|g| { t(9); t(is_go(g)) });
        hk::ftCo_SpecialS_CheckInput = Some(|_| { t(10); sr(4) });
        hk::ftCo_80094E54 = Some(|fr| { t(11); t(isf(fr)); sr(5) });
        hk::ftCo_800957F4 = Some(|g, ms| { t(12); t(is_go(g)); t(ms as u32) });
        hk::ftCo_800D8A38 = Some(|_| { t(13); sr(6) });
        hk::ftCo_AttackS4_8008C114 = Some(|_| { t(14); sr(7) });
        hk::ftCo_80099264 = Some(|_| { t(15); sr(8) });
        hk::ftCo_AttackDash_CheckInput = Some(|_| { t(16); sr(9) });
        hk::ftCo_AttackDash_SetMv0 = Some(|g| { t(17); t(is_go(g)) });
        hk::ftCo_80091AD8 = Some(|_, n| { t(18); t(n as u32); sr(10) });
        hk::ftCo_80091A4C = Some(|_| { t(19); sr(11) });
        hk::ftCo_80091B9C = Some(|g| { t(20); t(is_go(g)) });
        hk::ftCo_800DE9D8 = Some(|_| { t(21); sr(12) });
        hk::fn_800CAF78 = Some(|_| { t(22); sr(13) });
        hk::fn_800CA5F0 = Some(|_| { t(23); sr(14) });
        hk::ft_GetGroundFrictionMultiplier = Some(|fr| { t(24); t(isf(fr)); FR });
        hk::ftCommon_CalcGroundAccel_DashRun = Some(|fr, a, tv, fc| { t(25); t(isf(fr)); t(b(a)); t(b(tv)); t(b(fc)) });
        hk::ftCommon_SetSelfMovementFromGroundedMovement = Some(|g| { t(26); t(is_go(g)) });
        hk::ft_800844EC = Some(|g| { t(27); t(is_go(g)) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_Dash.txt"));
    for op in 0..7u32 {
        for k in 0..3000u32 {
            case(op, k);
            unsafe {
                NMV = 0;
                FR = rg();
                MV[0] = rw();
                MV[1] = rw();
            }
            let d = cd();
            d.dash_smash_stick_threshold = rg(); d.dash_smash_window = (rnd() % 8) as i32;
            d.x44 = rg(); d.x48 = rg(); d.x4C = rw(); d.x54 = rg();
            d.run_dash_turn_friction_multiplier = rg();
            let fr = f();
            fr.input.lstick[0].x = rg();
            fr.facing_dir = rg();
            fr.active_timer.lstick.x = (rnd() % 10) as u8;
            fr.co_attrs.dash_initial_velocity = rw(); fr.co_attrs.dash_accel_mul = rg();
            fr.co_attrs.dash_accel_base = rg(); if fr.co_attrs.dash_accel_base.is_nan() { fr.co_attrs.dash_accel_base = 0.5; } /* NaN+NaN picks an operand by codegen order */ fr.co_attrs.dash_max_velocity = rw();
            fr.co_attrs.ground_friction = rg();
            fr.gr_vel = rw();
            fr.player_idx = (rnd() % 6) as _;
            fr.x197C = if rnd() & 1 != 0 { go2() } else { core::ptr::null_mut() };
            fr.item_gobj = if rnd() & 1 != 0 { igo() as *mut Item_GObj } else { core::ptr::null_mut() };
            fr.cmd_vars[0] = rnd() & 1;
            fr.cur_anim_frame = rw();
            unsafe {
                fr.mv.co.dash.x0 = if rnd() & 1 != 0 { rg() } else { 0.0 };
                fr.mv.co.dash.x4 = (rnd() & 1) as i32;
            }
            let g = go();
            let ret = match op {
                0 => m::ftCo_Dash_CheckInput(g) as u32,
                1 => { m::ftCo_Dash_Enter(g, (rnd() % 3) as i32); 0 }
                2 => { m::ftCo_Dash_Anim(g); 0 }
                3 | 6 => { m::ftCo_Dash_IASA(g); 0 }
                4 => { m::ftCo_Dash_Phys(g); 0 }
                _ => { m::ftCo_Dash_Coll(g); 0 }
            };
            let f = f();
            let mut h = Hs::new();
            h.m(f.cmd_vars[0]); h.m(f.active_timer.lstick.x as u32);
            unsafe { h.m(b(f.mv.co.dash.x0)); h.m(f.mv.co.dash.x4 as u32); }
            h.m(b(f.gr_vel));
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 21000);
}
