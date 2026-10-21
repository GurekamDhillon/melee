//! src/tu/ftCo_ItemParasolOpen.rs vs the C (gen/cref/tu/ftCo_ItemParasolOpen.c: the whole TU,
//! shipping contraction; regenerate with
//! `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_ItemParasolOpen`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_ItemParasolOpen as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

fn isf(p: &Fighter) -> u32 { core::ptr::eq(p, f()) as u32 }

#[test]
fn ftCo_ItemParasolOpen_matches_c() {
    init();
    unsafe {
        hk::ftGetParasolStatus = Some(|g| { t(1); t(is_go(g)); if sr(1) { 6 } else { (rnd() % 8) as i32 } });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(2); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftAnim_8006EBA4 = Some(|g| { t(3); t(is_go(g)) });
        hk::ftAnim_IsFramesRemaining = Some(|g| { t(4); t(is_go(g)); sr(2) });
        hk::ftCo_800CF3C8 = Some(|g| { t(5); t(is_go(g)) });
        hk::ftCo_800CF280 = Some(|g| { t(6); t(is_go(g)) });
        hk::ftCo_SpecialAir_CheckInput = Some(|_| { t(7); sr(3) });
        hk::ftCo_80095328 = Some(|_, p| { t(8); t(p.is_null() as u32); sr(4) });
        hk::ftCo_800C3B10 = Some(|_| { t(9); sr(5) });
        hk::ftCo_AttackAir_CheckItemThrowInput = Some(|_| { t(10); sr(6) });
        hk::ftCo_800CB870 = Some(|_| { t(11); sr(7) });
        hk::ftCommon_Fall = Some(|fr, a, bb| { t(12); t(isf(fr)); t(b(a)); t(b(bb)) });
        hk::ftCommon_CalcSelfAccel_AccelToVelClamped = Some(|fr, a, bb, c| { t(13); t(isf(fr)); t(b(a)); t(b(bb)); t(b(c)) });
        hk::ftCo_Landing_Enter_Basic = Some(|g| { t(14); t(is_go(g)) });
        hk::ft_8008370C = Some(|g, cb| {
            t(15); t(is_go(g)); t((cb as usize == m::ext::ftCo_Landing_Enter_Basic as usize) as u32); cb(g)
        });
    }
    let ms = [29, ftCo_MS_FallSpecial as i32, 36, 360];
    let mut c = Check::new(include_str!("data/tu_ftCo_ItemParasolOpen.txt"));
    for op in 0..10u32 {
        for k in 0..2000u32 {
            case(op, k);
            let fr = f();
            fr.motion_id = ms[(rnd() % 4) as usize];
            unsafe { fr.mv.co.parasol_open.prev_msid = ms[(rnd() % 4) as usize] };
            fr.kind = if rnd() & 1 != 0 { Ft_Kind_Peach } else { (rnd() % 30) as _ };
            unsafe { fr.u.pe.parasol_gobj_0 = if rnd() & 1 != 0 { igo() as *mut Item_GObj } else { core::ptr::null_mut() } };
            fr.set_x2221_b4((rnd() & 1) as u8); fr.set_x2221_b5((rnd() & 1) as u8);
            fr.set_x2221_b6((rnd() & 1) as u8); fr.set_x2221_b7((rnd() & 1) as u8);
            fr.x2104 = (rnd() % 100) as i32;
            fr.input.lstick[0].x = rg(); fr.input.lstick[0].y = rg();
            fr.self_vel.y = rg();
            fr.co_attrs.gravity = rw(); fr.co_attrs.terminal_velocity = rw();
            fr.co_attrs.air_drift_stick_mul = rw(); fr.co_attrs.air_drift_max = rw();
            fr.co_attrs.aerial_friction = rw();
            let d = cd();
            d.x590 = rg(); d.x58C = rw(); d.x258 = rg(); d.open_parasol_threshold = rg();
            d.x59C = rw();
            let g = go();
            let ret = match op {
                0 => m::ftCo_800CEE70(g) as u32,
                1 => { m::ft_800CEF08(g); 0 }
                2 => { m::ftCo_800CEFE0(g, rnd() as i32); 0 }
                3 => { m::ftCo_ItemParasolOpen_Anim(g); 0 }
                4 => { m::ftCo_ItemParasolOpen_IASA(g); 0 }
                5 => { m::ftCo_ItemParasolOpen_Phys(g); 0 }
                6 => { m::ftCo_ItemParasolOpen_Coll(g); 0 }
                7 => m::ftCo_ItemParasolGetFallMotionId(g, 1000, 2000) as u32,
                8 => { m::ftCo_ItemParasol_Phys(g); 0 }
                _ => { let r = m::ftCo_800CEE70(g) as u32; m::ftCo_ItemParasolOpen_Anim(g); r }
            };
            let fr = f();
            let mut h = Hs::new();
            h.m(fr.motion_id as u32); h.m(unsafe { fr.mv.co.parasol_open.prev_msid } as u32); h.m(fr.x2104 as u32);
            h.m(fr.x2221_b4() as u32); h.m(fr.x2221_b5() as u32); h.m(fr.x2221_b6() as u32); h.m(fr.x2221_b7() as u32);
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 20000);
}
