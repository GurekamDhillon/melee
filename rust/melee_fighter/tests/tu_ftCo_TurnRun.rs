//! src/tu/ftCo_TurnRun.rs vs the C (gen/cref/tu/ftCo_TurnRun.c: the whole TU, TARGET_PC, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_TurnRun`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_TurnRun as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

static mut MV: [f32; 2] = [0.0; 2];
static mut NMV: usize = 0;

fn isf(fr: &Fighter) -> u32 { (fr as *const Fighter == f() as *const Fighter) as u32 }

#[test]
fn ftCo_TurnRun_matches_c() {
    init();
    unsafe {
        hk::ScriptGame_CapsActionAllowed = Some(|fr, fl| { t(1); t(isf(fr)); t(fl as u32); sr(1) as i32 });
        hk::ScriptGame_ModValue = Some(|s, fl, bs| { t(2); t(s as u32); t(fl as u32); t(b(bs)); if sr(2) { bs } else { let v = MV[NMV & 1]; NMV += 1; v } });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(3); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ft_PlaySFX = Some(|fr, id, a, bb| { t(4); t(isf(fr)); t(id as u32); t(a as u32); t(bb as u32) });
        hk::ftAnim_SetAnimRate = Some(|g, r| { t(5); t(is_go(g)); t(b(r)) });
        hk::ftAnim_IsFramesRemaining = Some(|_| { t(6); sr(6) });
        hk::fn_800CA644 = Some(|_| { t(7); sr(7) });
        hk::ft_8008A2BC = Some(|g| { t(8); t(is_go(g)) });
        hk::fn_800CAF78 = Some(|_| { t(9); sr(9) });
        hk::ftCommon_CalcGroundAccel_Deaccel = Some(|fr, x| { t(10); t(isf(fr)); t(b(x)) });
        hk::ftCommon_SetSelfMovementFromGroundedMovement = Some(|g| { t(11); t(is_go(g)) });
        hk::ft_800827A0 = Some(|_| { t(12); sr(12) });
        hk::ftCo_Fall_Enter = Some(|g| { t(13); t(is_go(g)) });
        hk::ftCommon_8007E2FC = Some(|g| { t(14); t(is_go(g)) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_TurnRun.txt"));
    for op in 0..7u32 {
        for k in 0..4000u32 {
            case(op, k);
            unsafe {
                NMV = 0;
                MV[0] = if rnd() & 3 != 0 { rw() } else { 0.0 };
                MV[1] = if rnd() & 3 != 0 { rw() } else { 0.0 };
            }
            let d = cd();
            d.x38_someLStickXThreshold = rg();
            d.run_dash_turn_friction_multiplier = rg();
            let fr = f();
            fr.input.lstick[0].x = rg();
            fr.facing_dir = rg();
            fr.cur_anim_frame = rw();
            fr.player_id = (rnd() % 6) as u8;
            fr.x197C = if rnd() & 1 != 0 { go2() } else { core::ptr::null_mut() };
            fr.cmd_vars[1] = rnd() & 1;
            fr.gr_vel = rw();
            fr.co_attrs.dash_accel_mul = rg();
            fr.co_attrs.dash_accel_base = rg();
            if fr.co_attrs.dash_accel_base.is_nan() { fr.co_attrs.dash_accel_base = 0.5; }
            fr.co_attrs.dash_max_velocity = rw();
            fr.co_attrs.ground_friction = rg();
            fr.xE4_ground_accel_1 = rw();
            fr.coll_data.env_flags = (rnd() & 0x0030_0000) as _;
            unsafe {
                fr.mv.co.turnrun.accel_mul = rg();
                fr.mv.co.turnrun.x14 = (rnd() & 1) as i32;
                fr.mv.co.walk.middle_anim_frame = rg();
            }
            let g = go();
            let ret = match op {
                0 => m::fn_800C9CEC(g) as u32,
                1 => m::fn_800C9D40(g) as u32,
                2 => { m::ftCo_TurnRun_Enter(g, rw()); 0 }
                3 => { m::ftCo_TurnRun_Anim(g); 0 }
                4 => { m::ftCo_TurnRun_IASA(g); 0 }
                5 => { m::ftCo_TurnRun_Phys(g); 0 }
                _ => { m::ftCo_TurnRun_Coll(g); 0 }
            };
            let fr = f();
            let mut h = Hs::new();
            h.m(fr.cmd_vars[1]);
            unsafe { h.m(b(fr.mv.co.turnrun.accel_mul)); h.m(fr.mv.co.turnrun.x14 as u32); }
            h.m(b(fr.facing_dir)); h.m(b(fr.xE4_ground_accel_1));
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 28000);
}
