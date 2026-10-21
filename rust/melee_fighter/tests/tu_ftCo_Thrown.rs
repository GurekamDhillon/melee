//! src/tu/ftCo_Thrown.rs vs the C (gen/cref/tu/ftCo_Thrown.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_Thrown`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_s217;
use tu_ftco_harness::*;
use tu_s217::*;
use melee_fighter::tu::ftCo_Thrown as m;
use m::ext::hook as hk;

#[test]
fn ftCo_Thrown_matches_c() {
    s_init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(cms);
        hk::ftCo_800DB368 = Some(|a, bb| { t(10); t(pi(a)); t(pi(bb)) });
        hk::ftColl_8007B62C = Some(|g, n| { t(11); t(pi(g)); t(n as u32) });
        hk::ftKb_MexKirbyLike = Some(|k| { t(12); t(k as u32); chk(12) as i32 });
        hk::pl_80040614 = Some(|a, bb, f| { t(13); t(a as u32); t(bb as u32); t(b(f)) });
        hk::ftCommon_8007E2F4 = Some(|f, v| { t(14); t(pi(f)); t(v as i32 as u32) });
        hk::ftParts_GetBoneIndex = Some(|f, p| { t(17); t(pi(f)); t(p as u32); (SRV[5].wrapping_add(p as u32) % 8) as i32 });
        hk::lb_8000B1CC = Some(|j, p0, out| { t(18); t(pi(j)); t(pi(p0)); out.x = rw(); out.y = rw(); out.z = rw() });
        hk::ftAnim_SetAnimRate = Some(|g, r| { t(19); t(pi(g)); t(b(r)) });
        hk::ftCommon_GrabMash = Some(|f, v| { t(21); t(pi(f)); t(b(v)); chk(21) });
        hk::ftColl_8007B8CC = Some(|f, v| { t(27); t(pi(f)); t(pi(v)) });
        hk::ftCo_8008DCE0 = Some(|g, a, f| { t(28); t(pi(g)); t(a as u32); t(b(f)) });
        hk::ftCo_8008E5A4 = Some(|f| { t(29); t(pi(f)) });
        hk::ftColl_80079C70 = Some(|a, bb, h, n| { t(30); t(pi(a)); t(pi(bb)); t(pi(h)); t(n as u32); rw() });
        hk::ScriptGame_FighterDealtDamage = Some(|f, d| { t(31); t(pi(f)); t(b(d)); d + 1.0 });
        hk::ScriptGame_HitRuleContact = Some(|h, f, v, kb| { t(32); t(pi(h)); t(pi(f)); t(b(v)); t(kb as u32); v + kb as f32 });
        hk::ScriptGame_HitRuleReport = Some(|a, bb, h, d| { t(33); t(pi(a)); t(pi(bb)); t(pi(h)); t(b(d)) });
        hk::ScriptGame_HitRulePercentQueue = Some(|h, f, d| { t(34); t(pi(h)); t(pi(f)); t(b(d)) });
        hk::Fighter_UnkTakeDamage_8006CC30 = Some(|f, d| { t(35); t(pi(f)); t(b(d)) });
        hk::ftColl_8007891C = Some(|a, bb, d| { t(36); t(pi(a)); t(pi(bb)); t(b(d)) });
    }
    sv!(hk; ftCommon_8007E358 = 15, ftAnim_8006EBA4 = 16, ftCommon_8007E3EC = 20, fn_800DC070 = 23,
        ftCo_CaptureCut_Enter = 24, ftCamera_800762F4 = 25, ftCamera_UpdateCameraBox = 26);
    sc!(hk; fn_800DC044 = 22);
    let mut c = Check::new(include_str!("data/tu_ftCo_Thrown.txt"));
    for op in 0..25u32 {
        for k in 0..1000u32 {
            case(op, k);
            s_fill();
            let ms = 239 + (rnd() % 6) as i32;
            let sp = rg();
            let vg = if rnd() & 1 != 0 { go2() } else { core::ptr::null_mut() };
            let up = rnd() & 1 != 0;
            let g = go();
            match op {
                0 => { f().victim_gobj = go2(); m::ftCo_800DE3FC(g, ms, sp) }
                1 => m::ftCo_800DE508(g),
                2 => m::ftCo_800DE5A4(g),
                3 => m::ftCo_ThrownF_Anim(g),
                4 => m::ftCo_ThrownF_IASA(g),
                5 => m::ftCo_ThrownF_Phys(g),
                6 => m::ftCo_ThrownF_Coll(g),
                7 => m::ftCo_ThrownB_Anim(g),
                8 => m::ftCo_ThrownB_IASA(g),
                9 => m::ftCo_ThrownB_Phys(g),
                10 => m::ftCo_ThrownB_Coll(g),
                11 => m::ftCo_ThrownHi_Anim(g),
                12 => m::ftCo_ThrownHi_IASA(g),
                13 => m::ftCo_ThrownHi_Phys(g),
                14 => m::ftCo_ThrownHi_Coll(g),
                15 => { f().victim_gobj = go2(); m::ftCo_ThrownHi_Cam(g) }
                16 => m::ftCo_ThrownLw_Anim(g),
                17 => m::ftCo_ThrownLw_IASA(g),
                18 => m::ftCo_ThrownLw_Phys(g),
                19 => m::ftCo_ThrownLw_Coll(g),
                20 => m::fn_800DE798(g),
                21 => m::ftCo_800DE7C0(g, vg, up),
                22 => { f().victim_gobj = go2(); m::ftCo_800DE854(g) }
                23 => m::ftCo_800DE920(g, sp),
                _ => m::ftCo_800DE974(g),
            }
            c.line(op, k, 0, state_hash());
        }
    }
    assert_eq!(c.done(), 25000);
}
