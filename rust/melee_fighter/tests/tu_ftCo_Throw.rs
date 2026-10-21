//! src/tu/ftCo_Throw.rs vs the C (gen/cref/tu/ftCo_Throw.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_Throw`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_s217;
use tu_ftco_harness::*;
use tu_s217::*;
use melee_fighter::tu::ftCo_Throw as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

static mut NEVER: bool = false;

unsafe extern "C" fn cb_c(g: *mut HSD_GObj) {
    t(98);
    t(pi(g));
}
fn fp_of(g: *mut HSD_GObj) -> &'static mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

#[test]
fn ftCo_Throw_matches_c() {
    s_init();
    unsafe {
        hk::ftCo_800DC920 = Some(|a, bb| { t(10); t(pi(a)); t(pi(bb)) });
        hk::ftCo_800DF7F4 = Some(|f| { t(13); t(pi(f)); chk(13) });
        hk::ftCo_800DF844 = Some(|f| { t(14); t(pi(f)); chk(14) });
        hk::ftCo_800DF878 = Some(|f| { t(15); t(pi(f)); chk(15) });
        hk::Fighter_ChangeMotionState = Some(cms);
        hk::ftAnim_8006EBA4 = Some(|g| {
            t(16); t(pi(g));
            if !NEVER && (rnd() & 7) == 0 { fp_of(g).cmd_vars[0] = 1; }
        });
        hk::ftCommon_8007E2F4 = Some(|f, v| { t(17); t(pi(f)); t(v as i32 as u32) });
        hk::ftCo_800DE3FC = Some(|g, ms, s| { t(18); t(pi(g)); t(ms as u32); t(b(s)) });
        hk::ftKb_MexKirbyLike = Some(|k| { t(19); t(k as u32); chk(19) as i32 });
        hk::ftSs_Init_CreateThrowGrappleBeam = Some(|g, ms, s| { t(20); t(pi(g)); t(ms as u32); t(b(s)) });
        hk::ftColl_8007B7A4 = Some(|g, n| { t(21); t(pi(g)); t(n as u32) });
        hk::Geno_DefineTemplateSkeleton = Some(|k| { t(22); t(k as u32); chk(22) as i32 });
        hk::Geno_DefineBaseKind = Some(|k| { t(23); t(k as u32); if SRV[6] & 1 != 0 { 19 } else { 9 } });
        hk::ftAnim_SetAnimRate = Some(|g, r| { t(24); t(pi(g)); t(b(r)) });
        hk::ftCommon_8007D7FC = Some(|f| { t(26); t(pi(f)) });
        hk::mpLineGetFlags = Some(|l| { t(28); t(l as u32); SRV[7] });
        hk::pl_80040614 = Some(|a, bb, f| { t(29); t(a as u32); t(bb as u32); t(b(f)) });
        hk::ftCo_800DE7C0 = Some(|g, v, up| { t(30); t(pi(g)); t(pi(v)); t(up as u32) });
        hk::ftCo_800DE920 = Some(|g, f| { t(31); t(pi(g)); t(b(f)) });
        hk::ft_80083C00 = Some(|g, cb| { t(37); t(pi(g)); t(cb.is_some() as u32) });
        hk::ft_80083CE4 = Some(|g, _a, cb| { t(38); t(pi(g)); t(1); t(cb.is_some() as u32) });
        hk::ft_800841B8 = Some(|g, cb| { t(40); t(pi(g)); t(cb.is_some() as u32) });
        hk::ftColl_8007891C = Some(|a, bb, d| { t(44); t(pi(a)); t(pi(bb)); t(b(d)) });
        hk::ftColl_8007B62C = Some(|g, n| { t(45); t(pi(g)); t(n as u32) });
        hk::ftParts_GetBoneIndex = Some(|f, p| { t(46); t(pi(f)); t(p as u32); (SRV[5].wrapping_add(p as u32) % 8) as i32 });
        hk::lb_8000B1CC = Some(|j, p0, out| { t(47); t(pi(j)); t(pi(p0)); out.x = rw(); out.y = rw(); out.z = rw() });
        hk::ftColl_8007B868 = Some(|g| { t(48); t(pi(g)); chk(48) as i32 });
        hk::ScriptGame_FighterDealtDamage = Some(|f, d| { t(49); t(pi(f)); t(b(d)); d + 1.0 });
        hk::ScriptGame_HitRuleContact = Some(|h, f, v, kb| { t(50); t(pi(h)); t(pi(f)); t(b(v)); t(kb as u32); v + kb as f32 });
        hk::ftColl_80076640 = Some(|f, d| { t(51); t(pi(f)); t(b(*d)); if chk(51) { *d = *d + 2.0; true } else { false } });
        hk::ScriptGame_HitRulePercentQueue = Some(|h, f, d| { t(52); t(pi(h)); t(pi(f)); t(b(d)) });
        hk::gm_8016B248 = Some(|| { t(53); rg() });
        hk::Player_GetAttackRatio = Some(|s| { t(54); t(s as u32); rg() });
        hk::Player_GetDefenseRatio = Some(|s| { t(55); t(s as u32); rg() });
        hk::ftColl_80079AB0 = Some(|f, h, n, a, bb, c, d| { t(56); t(pi(f)); t(pi(h)); t(n); t(b(a)); t(b(bb)); t(b(c)); t(b(d)); rw() });
        hk::ScriptGame_HitRuleWon = Some(|h, f| { t(57); t(pi(h)); t(pi(f)) });
        hk::ScriptGame_HitRuleReport = Some(|a, bb, h, d| { t(58); t(pi(a)); t(pi(bb)); t(pi(h)); t(b(d)) });
        hk::ftColl_80078710 = Some(|a, bb, p| { t(59); t(pi(a)); t(pi(bb)); t(pi(p)) });
        hk::ftCommon_8007EE0C = Some(|f, v| { t(60); t(pi(f)); t(v as u32) });
        hk::ftCo_Damage_CalcKnockback = Some(|f| { t(61); t(pi(f)) });
        hk::ftCommon_8007D5D4 = Some(|f| { t(62); t(pi(f)) });
        hk::lb_8000C390 = Some(|j| { t(63); t(pi(j)) });
        hk::HSD_JObjSetMtxDirtySub = Some(|j| { t(64); t(pi(j)) });
        hk::mpColl_80043670 = Some(|c| { t(65); t(pi(c)) });
        hk::ftCommon_UnlockECB = Some(|f| { t(66); t(pi(f)) });
        hk::mpColl_800471F8 = Some(|c| { t(67); t(pi(c)); c.cur_pos.x = c.cur_pos.x + 1.0; chk(67) });
        hk::lbColl_80008D30 = Some(|h, a| {
            t(68); t((pi(h) == 99) as u32); t(pi(a));
            h.damage = rw(); h.unk_count = rnd() % 5; h.kb_angle = (rnd() % 361) as i32; h.element = rnd() % 5;
        });
        hk::ScriptGame_ModValue = Some(|s, fl, base| { t(70); t(s as u32); t(fl as u32); t(b(base)); base + fl as f32 });
        hk::Fighter_UnkTakeDamage_8006CC30 = Some(|f, d| { t(71); t(pi(f)); t(b(d)) });
        hk::ftCo_8008E908 = Some(|g, v| { t(72); t(pi(g)); t(b(v)) });
    }
    sv!(hk; ftCo_800DCFD4 = 11, ftCommon_8007D92C = 12, ftCo_800DE974 = 25, ftCo_Fall_Enter = 27,
        ftMt_SpecialN_Shoot = 32, ftCo_8009B56C = 34, ft_80085134 = 35, ft_80085004 = 36,
        ft_80083B68 = 39, ftFx_Throw_Anim = 41, ftCamera_800762F4 = 42, ftCamera_UpdateCameraBox = 43,
        ftColl_800788D4 = 69);
    sc!(hk; ftAnim_IsFramesRemaining = 33);
    let mut c = Check::new(include_str!("data/tu_ftCo_Throw.txt"));
    for op in 0..31u32 {
        for k in 0..1000u32 {
            case(op, k);
            s_fill();
            unsafe { NEVER = SRV[3] & 1 != 0; }
            f().anim_cb = if rnd() & 1 != 0 { Some(cb_c) } else { None };
            let ms = 219 + (rnd() % 4) as i32;
            let vms = 239 + (rnd() % 5) as i32;
            let sp = rg();
            let li = if rnd() & 3 == 0 { -1 } else { (rnd() % 50) as i32 };
            let ar = rnd() & 1 != 0;
            let g = go();
            let mut ret = 0u32;
            match op {
                0 => m::ftCo_800DD100(g),
                1 => m::ftCo_800DD168(g),
                2 => { f().victim_gobj = go2(); ret = m::ftCo_800DD1E4(g) as u32 }
                3 => { f().victim_gobj = go2(); m::ftCo_800DD398(g, ms, vms, sp) }
                4 => { f().victim_gobj = go2(); m::ftCo_800DD4B0(g, ms) }
                5 => m::fn_800DD568(g),
                6 => m::fn_800DD5EC(g),
                7 => m::fn_800DD684(g),
                8 => ret = m::fn_800DD6E4(g, li) as u32,
                9 => m::ftCo_800DD724(g),
                10 => m::ftCo_ThrowF_Anim(g),
                11 => m::ftCo_ThrowF_IASA(g),
                12 => m::ftCo_ThrowF_Phys(g),
                13 => m::ftCo_ThrowF_Coll(g),
                14 => m::ftCo_ThrowB_Anim(g),
                15 => m::ftCo_ThrowB_IASA(g),
                16 => m::ftCo_ThrowB_Phys(g),
                17 => m::ftCo_ThrowB_Coll(g),
                18 => m::ftCo_ThrowHi_Anim(g),
                19 => m::ftCo_ThrowHi_IASA(g),
                20 => m::ftCo_ThrowHi_Phys(g),
                21 => m::ftCo_ThrowHi_Coll(g),
                22 => m::ftCo_ThrowHi_Cam(g),
                23 => m::ftCo_ThrowLw_Anim(g),
                24 => m::ftCo_ThrowLw_IASA(g),
                25 => m::ftCo_ThrowLw_Phys(g),
                26 => m::ftCo_ThrowLw_Coll(g),
                27 => m::ftCo_800DDDE4(g, go2(), ar),
                28 => m::ftCo_800DE2A8(g, go2()),
                29 => m::ftCo_800DE2CC(g, go2()),
                _ => m::ftCo_800DE2F0(g),
            }
            c.line(op, k, ret, state_hash());
        }
    }
    assert_eq!(c.done(), 31000);
}
