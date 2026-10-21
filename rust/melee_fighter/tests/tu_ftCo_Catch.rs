//! src/tu/ftCo_Catch.rs vs the C (gen/cref/tu/ftCo_Catch.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_Catch`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_Catch as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

#[test]
fn ftCo_Catch_matches_c() {
    init();
    unsafe {
        hk::ftCo_800951D0 = Some(|g| { t(1); t(is_go(g)); sr(1) });
        hk::ftCo_80095254 = Some(|g| { t(2); t(is_go(g)); sr(2) });
        hk::ftCo_800952DC = Some(|g| { t(3); t(is_go(g)); sr(3) });
        hk::ScriptGame_CapsActionAllowed = Some(|fr, fl| {
            t(4); t((fr as *const Fighter == F as *const Fighter) as u32); t(fl as u32);
            if sr(4) { (SRV[9] % 3) as i32 } else { 1 }
        });
        hk::fn_800D952C = Some(|g| { t(5); t(is_go(g)); sr(5) });
        hk::fn_800D8EC8 = Some(|g| { t(6); t(is_go(g)); sr(6) });
        hk::fn_800D9228 = Some(|g| { t(7); t(is_go(g)); sr(7) });
        hk::fn_800D949C = Some(|g| { t(8); t(is_go(g)) });
        hk::fn_800D9558 = Some(|g| { t(9); t(is_go(g)); sr(8) });
        hk::fn_800D9930 = Some(|g| { t(10); t(is_go(g)); sr(9) });
        hk::fn_800D9C64 = Some(|g| { t(11); t(is_go(g)) });
        hk::ftpickupitem_800948A8 = Some(|g, i| { t(12); t(is_go(g)); t((i == igo()) as u32) });
        hk::ftpickupitem_80094694 = Some(|g, ms, bb| { t(13); t(is_go(g)); t(ms as u32); t(bb as u32) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(14); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::fn_800D9CE8 = Some(|g| { t(15); t(is_go(g)) });
        hk::fn_800DAADC = Some(|g, o| { t(16); t(is_go(g)); t((o == go2()) as u32) });
        hk::ftCommon_8007E2D0 = Some(|fr, a, g0, g1, g2| {
            t(17); t((fr as *mut Fighter == F) as u32); t(a as i32 as u32);
            t((g0.map(|p| p as usize) == Some(m::fn_800D9CE8_cb as usize)) as u32);
            t((g1.map(|p| p as usize) == Some(m::fn_800D8BFC_cb as usize)) as u32);
            t((g2.map(|p| p as usize) == Some(m::fn_800DAADC_cb as usize)) as u32);
            unsafe { (g0.unwrap())(go()); (g1.unwrap())(go()); (g2.unwrap())(go(), go2()); }
        });
        hk::Mex_OnCatchDispatch = Some(|kind, g, v| { t(18); t(kind as u32); t(is_go(g)); t(v.is_null() as u32) });
        hk::ftAnim_IsFramesRemaining = Some(|g| { t(19); t(is_go(g)); sr(10) });
        hk::ft_8008A2BC = Some(|g| { t(20); t(is_go(g)) });
        hk::ftCommon_CalcGroundAccel_Deaccel = Some(|fr, x| { t(21); t((fr as *mut Fighter == F) as u32); t(b(x)) });
        hk::ftCommon_SetSelfMovementFromGroundedMovement = Some(|g| { t(22); t(is_go(g)) });
        hk::ft_80085030 = Some(|g, a, bb| { t(23); t(is_go(g)); t(b(a)); t(b(bb)) });
        hk::ft_800841B8 = Some(|g, cb| {
            t(24); t(is_go(g));
            t((cb.map(|p| p as usize) == Some(m::fn_800D8E30_cb as usize)) as u32);
            unsafe { (cb.unwrap())(g) }
        });
        hk::ftCo_Fall_Enter = Some(|g| { t(25); t(is_go(g)) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_Catch.txt"));
    for op in 0..15u32 {
        for k in 0..2000u32 {
            case(op, k);
            let fr = f();
            fr.kind = rnd() % 34;
            fr.u.lk.xC = if rnd() & 1 != 0 { igo() } else { core::ptr::null_mut() };
            fr.input.held_buttons[0] = rnd() & 0x8000_0000;
            fr.input.pressed_buttons = rnd() & 0x1FF;
            fr.mv.co.common.x0 = (rnd() % 3) as i32;
            fr.mv.co.guard.x24 = (rnd() % 3) as i32;
            fr.target_item_gobj = if rnd() & 1 != 0 { igo() } else { core::ptr::null_mut() };
            fr.x1A64 = unsafe { CD } as *mut core::ffi::c_void;
            fr.victim_gobj = go2();
            cd().x64 = rg();
            fr.co_attrs.ground_friction = rg();
            fr.facing_dir = rg();
            fr.x74_self_accel.x = rg();
            fr.x74_self_accel.y = rg();
            fr.x74_self_accel.z = rg();
            let g = go();
            let ret = match op {
                0 => m::ftCo_Catch_CheckInput(g) as u32,
                1 => m::ftCo_800D8A38(g) as u32,
                2 => m::ftCo_800D8AE0(g) as u32,
                3 => m::ftCo_800D8B9C(g) as u32,
                4 => { m::fn_800D8BFC(g); 0 }
                5 => { m::ftCo_800D8C54(g, (rnd() % 400) as i32); 0 }
                6 => { m::ftCo_Catch_Anim(g); 0 }
                7 => { m::ftCo_CatchDash_Anim(g); 0 }
                8 => { m::ftCo_Catch_IASA(g); m::ftCo_CatchDash_IASA(g); 0 }
                9 => { m::ftCo_Catch_Phys(g); 0 }
                10 => { m::ftCo_CatchDash_Phys(g); 0 }
                11 => { m::ftCo_Catch_Coll(g); 0 }
                12 => { m::fn_800D8E30(g); 0 }
                13 => { m::ftCo_CatchDash_Coll(g); 0 }
                _ => m::fn_800D8E94(g) as u32,
            };
            let fr = f();
            let mut h = Hs::new();
            h.m(b(fr.x74_self_accel.x)); h.m(b(fr.x74_self_accel.y)); h.m(b(fr.x74_self_accel.z));
            unsafe { h.m(fr.mv.co.common.x0 as u32); h.m(fr.mv.co.guard.x24 as u32); }
            h.m(fr.target_item_gobj.is_null() as u32); h.m(fr.x1A64.is_null() as u32);
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 30000);
}
