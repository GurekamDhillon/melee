//! src/tu/ftCo_DamageScrew.rs vs the C (gen/cref/tu/ftCo_DamageScrew.c, shipping contraction;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_DamageScrew`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_DamageScrew as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

#[test]
fn ftCo_DamageScrew_matches_c() {
    init();
    unsafe {
        hk::ftCommon_8007DB58 = Some(|g| { t(1); t(is_go(g)) });
        hk::ftCo_8009750C = Some(|g| { t(3); t(is_go(g)) });
        hk::ftCo_800DD168 = Some(|g| { t(4); t(is_go(g)) });
        hk::ftCommon_8007D5D4 = Some(|fr| { t(2); t((fr as *mut Fighter == F) as u32) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(7); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftAnim_IsFramesRemaining = Some(|_| { t(9); sr(3) });
        hk::ftCo_800968C8 = Some(|g| { t(10); t(is_go(g)) });
        hk::ft_80084DB0 = Some(|g| { t(11); t(is_go(g)) });
        hk::ftCo_AirCatchHit_Coll = Some(|g| { t(12); t(is_go(g)) });
        hk::ft_80083B68 = Some(|g| { t(13); t(is_go(g)) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_DamageScrew.txt"));
    for op in 0..6u32 {
        for k in 0..1000u32 {
            case(op, k);
            let fr = f();
            fr.dmg.x1860_element = if rnd() & 1 != 0 { HitElement_Scball as u32 } else { rnd() % 16 };
            unsafe { fr.x2070.x0.set_x2071_b0_3((rnd() & 15) as u8) };
            fr.cmd_vars[0] = rnd() & 1;
            fr.ground_or_air = (rnd() & 1) as _;
            fr.co_attrs.screw_attack_launch_velocity = rw();
            fr.self_vel.x = rg(); fr.self_vel.y = rg(); fr.self_vel.z = rg();
            let g = go();
            let ret = match op {
                0 => m::ftCo_800D2FA4(g) as u32,
                1 => { m::ftCo_800D3004(g); 0 }
                2 => { m::ftCo_DamageScrew_Anim(g); 0 }
                3 => { m::ftCo_DamageScrew_IASA(g); 0 }
                4 => { m::ftCo_DamageScrew_Phys(g); 0 }
                _ => { m::ftCo_DamageScrew_Coll(g); 0 }
            };
            let f = f();
            let mut h = Hs::new();
            h.m(f.cmd_vars[0]); h.m(b(f.self_vel.x)); h.m(b(f.self_vel.y)); h.m(b(f.self_vel.z));
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 6000);
}
