//! src/tu/ftCo_Shouldered.rs vs the C (gen/cref/tu/ftCo_Shouldered.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_Shouldered`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_ftco_s209;
use tu_ftco_harness::*;
use tu_ftco_s209::*;
use melee_fighter::tu::ftCo_Shouldered as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

fn kf(i: usize) -> f32 { ((unsafe { SRV[i & 15] } % 41) as i32 - 20) as f32 / 4.0 }
fn pf(f: *const Fighter) -> u32 { pi(f) }

#[test]
fn ftCo_Shouldered_matches_c() {
    s209_init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(1); t(pi(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(pi(o))
        });
        hk::ftCommon_8007E2FC = Some(|g| { t(2); t(pi(g)) });
        hk::ftCommon_8007E2F4 = Some(|f, v| { t(3); t(pf(f)); t(v as i32 as u32) });
        hk::ftCo_800DB464 = Some(|g| { t(4); t(pi(g)) });
        hk::ftCommon_InitGrab = Some(|f, a, v| { t(5); t(pf(f)); t(a as u32); t(b(v)) });
        hk::ftCommon_GrabMash = Some(|f, v| { t(6); t(pf(f)); t(b(v)); sr(6) });
        hk::ftCo_800DC920 = Some(|a, bb| { t(7); t(pi(a)); t(pi(bb)) });
        hk::ftParts_GetBoneIndex = Some(|f, p| { t(8); t(pf(f)); t(p as u32); (SRV[3] % 4) as i32 });
        hk::lb_8000B1CC = Some(|j, p0, out| {
            t(9); t(pi(j)); t(pi(p0));
            out.x = kf(1); out.y = kf(2); out.z = kf(5);
        });
        hk::ftColl_80079C70 = Some(|f, v, h, n| { t(10); t(pf(f)); t(pf(v)); t(pi(h)); t(n as u32); kf(6) });
        hk::ftColl_80078710 = Some(|a, bb, c| { t(11); t(pi(a)); t(pi(bb)); t(pi(c)) });
        hk::ScriptGame_FighterDealtDamage = Some(|f, d| { t(12); t(pf(f)); t(b(d)); d * 2.0 });
        hk::ScriptGame_HitRuleContact = Some(|h, f, v, kb| { t(13); t(pi(h)); t(pf(f)); t(b(v)); t(kb as u32); v + kf(7) });
        hk::ScriptGame_HitRuleReport = Some(|a, v, h, d| { t(14); t(pf(a)); t(pf(v)); t(pi(h)); t(b(d)) });
        hk::ScriptGame_HitRulePercentQueue = Some(|h, v, d| { t(15); t(pi(h)); t(pf(v)); t(b(d)) });
        hk::Fighter_UnkTakeDamage_8006CC30 = Some(|f, d| { t(16); t(pf(f)); t(b(d)) });
        hk::ftCo_Damage_CalcKnockback = Some(|f| { t(17); t(pf(f)) });
        hk::ftCo_8008E908 = Some(|g, v| { t(18); t(pi(g)); t(b(v)) });
        hk::ftAnim_SetAnimRate = Some(|g, r| { t(19); t(pi(g)); t(b(r)) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_Shouldered.txt"));
    for op in 0..7u32 {
        for k in 0..1000u32 {
            case(op, k);
            s209_fill();
            let msid = if rnd() & 1 != 0 { ftCo_MS_ShoulderedWait as i32 } else { (rnd() % 400) as i32 };
            let g = go();
            match op {
                0 => m::ftCo_8009C5A4(g, msid),
                1 => m::ftCo_8009C640(g, msid),
                2 => m::ftCo_8009C744(g),
                3 => m::ftCo_Shouldered_Anim(g),
                4 => m::ftCo_Shouldered_IASA(g),
                5 => m::ftCo_Shouldered_Phys(g),
                _ => m::ftCo_Shouldered_Coll(g),
            }
            c.line(op, k, 0, state_hash());
        }
    }
    assert_eq!(c.done(), 7000);
}
