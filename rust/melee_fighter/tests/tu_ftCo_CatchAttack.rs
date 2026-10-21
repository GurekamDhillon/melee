//! src/tu/ftCo_CatchAttack.rs vs the C (gen/cref/tu/ftCo_CatchAttack.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_CatchAttack`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_CatchAttack as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

unsafe extern "C" fn dummy(_g: *mut HSD_GObj) {}

fn is_cb(cb: HSD_GObjEvent, want: unsafe extern "C" fn(*mut HSD_GObj)) -> u32 {
    (cb.map(|p| p as usize) == Some(want as usize)) as u32
}

#[test]
fn ftCo_CatchAttack_matches_c() {
    init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(1); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftCommon_8007E2F4 = Some(|fr, v| { t(2); t((fr as *mut Fighter == F) as u32); t(v as i32 as u32) });
        hk::ftAnim_IsFramesRemaining = Some(|g| { t(3); t(is_go(g)); sr(1) });
        hk::fn_800DA2B0 = Some(|g| { t(4); t(is_go(g)) });
        hk::fn_800DA054 = Some(|g| { t(5); t(is_go(g)) });
        hk::ftCommon_CalcGroundAccel_Deaccel = Some(|fr, x| { t(6); t((fr as *mut Fighter == F) as u32); t(b(x)) });
        hk::ftCommon_SetSelfMovementFromGroundedMovement = Some(|g| { t(7); t(is_go(g)) });
        hk::ft_800841B8 = Some(|g, cb| { t(8); t(is_go(g)); t(is_cb(cb, m::fn_800DA618_cb)); unsafe { (cb.unwrap())(g) } });
        hk::ftCo_800DC920 = Some(|g, v| { t(9); t(is_go(g)); t((v == go2()) as u32) });
        hk::ftCo_Fall_Enter = Some(|g| { t(10); t(is_go(g)); t((g == go2()) as u32) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_CatchAttack.txt"));
    for op in 0..8u32 {
        for k in 0..2000u32 {
            case(op, k);
            let fr = f();
            fr.victim_gobj = go2();
            fr.gr_vel = rg();
            cd().x64 = rg();
            fr.co_attrs.ground_friction = rg();
            fr.take_dmg_cb = if rnd() & 1 != 0 { Some(dummy) } else { None };
            let g = go();
            match op {
                0 => m::fn_800DA4FC(g),
                1 => m::ftCo_CatchAttack_Anim(g),
                2 => m::ftCo_CatchAttack_IASA(g),
                3 => m::ftCo_CatchAttack_Phys(g),
                4 => m::ftCo_CatchAttack_Coll(g),
                5 => m::fn_800DA618(g),
                6 => m::fn_800DA668(g),
                _ => m::fn_800DA678(g),
            }
            let fr = f();
            let mut h = Hs::new();
            h.m(b(fr.gr_vel));
            h.m(is_cb(fr.accessory1_cb, m::fn_800DA678_cb));
            h.m(is_cb(fr.take_dmg_cb, m::fn_800DA668_cb));
            h.m(fr.take_dmg_cb.is_none() as u32);
            c.line(op, k, 0, h.0);
        }
    }
    assert_eq!(c.done(), 16000);
}
