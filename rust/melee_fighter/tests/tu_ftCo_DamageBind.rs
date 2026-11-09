//! src/tu/ftCo_DamageBind.rs vs the C (gen/cref/tu/ftCo_DamageBind.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_DamageBind`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_DamageBind as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

fn is_f(p: *const Fighter) -> u32 { (p == f() as *const Fighter) as u32 }

#[test]
fn ftCo_DamageBind_matches_c() {
    init();
    unsafe {
        hk::ftCommon_8007DB58 = Some(|g| { t(1); t(is_go(g)) });
        hk::ftCo_8009750C = Some(|g| { t(2); t(is_go(g)) });
        hk::ftCo_800DD168 = Some(|g| { t(3); t(is_go(g)) });
        hk::ftCo_80090780 = Some(|g| { t(4); t(is_go(g)) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(5); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::Player_GetHandicap = Some(|s| { t(6); t(s as u32); (SRV[1] % 8) as i32 });
        hk::Player_80033BB8 = Some(|s| { t(7); t(s as u32); (SRV[2] % 5) as i32 });
        hk::ftCommon_InitGrab = Some(|fr, a, x| { t(8); t(is_f(fr)); t(a as u32); t(b(x)) });
        hk::ftCommon_GrabMash = Some(|fr, v| { t(9); t(is_f(fr)); t(b(v)); sr(3) });
        hk::ft_8008A2BC = Some(|g| { t(10); t(is_go(g)) });
        hk::ft_80084F3C = Some(|g| { t(11); t(is_go(g)) });
        hk::ft_80084280 = Some(|g| { t(12); t(is_go(g)) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_DamageBind.txt"));
    for op in 0..7u32 {
        for k in 0..2000u32 {
            case(op, k);
            let fr = f();
            fr.dmg.x1860_element = rnd() % 12;
            fr.set_is_sandbag((rnd() & 1) as _);
            unsafe { fr.x2070.x0.set_x2071_b0_3((rnd() % 16) as u8) };
            fr.ground_or_air = (rnd() & 1) as _;
            fr.dmg.x1830_percent = rw();
            fr.player_idx = (rnd() % 4) as u8;
            fr.grab_timer = rw();
            let d = cd();
            d.x658 = rw();
            d.x65C = rg();
            d.x660 = rw();
            d.x664 = rg();
            d.pressed_inputs = rw();
            d.released_inputs = rg();
            d.x670 = rg();
            d.x674 = rg();
            let g = go();
            let ret = match op {
                0 => m::ftCo_800C44CC(g) as u32,
                1 => { m::commonCall(f()); 0 }
                2 => { m::ftCo_800C4550(g); 0 }
                3 => { m::ftCo_DamageBind_Anim(g); 0 }
                4 => { m::ftCo_DamageBind_IASA(g); 0 }
                5 => { m::ftCo_DamageBind_Phys(g); 0 }
                _ => { m::ftCo_DamageBind_Coll(g); 0 }
            };
            let mut h = Hs::new();
            h.m(b(f().grab_timer));
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 14000);
}
