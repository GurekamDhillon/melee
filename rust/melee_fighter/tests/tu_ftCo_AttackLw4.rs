//! src/tu/ftCo_AttackLw4.rs vs the C (gen/cref/tu/ftCo_AttackLw4.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_AttackLw4`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_AttackLw4 as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

#[test]
fn ftCo_AttackLw4_matches_c() {
    init();
    unsafe {
        hk::ftCo_800DF3A8 = Some(|_| { t(1); sr(0) });
        hk::ftCo_80094E54 = Some(|_| { t(2); sr(1) });
        hk::ftCo_800DF3DC = Some(|_| { t(3); sr(2) });
        hk::ftCo_800957F4 = Some(|g, ms| { t(4); t(is_go(g)); t(ms as u32) });
        hk::ftNs_AttackLw4_Enter = Some(|g| { t(5); t(is_go(g)) });
        hk::Mex_OnDSmashDispatch = Some(|k, g, v| { t(6); t(k as u32); t(is_go(g)); t(v.is_null() as u32) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(7); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftAnim_8006EBA4 = Some(|g| { t(8); t(is_go(g)) });
        hk::ftAnim_IsFramesRemaining = Some(|_| { t(9); sr(3) });
        hk::ft_8008A2BC = Some(|g| { t(10); t(is_go(g)) });
        hk::ftCo_Wait_IASA = Some(|g| { t(11); t(is_go(g)) });
        hk::ft_80084F3C = Some(|g| { t(12); t(is_go(g)) });
        hk::ft_80084104 = Some(|g| { t(13); t(is_go(g)) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_AttackLw4.txt"));
    for op in 0..7u32 {
        for k in 0..1000u32 {
            case(op, k);
            cd().xD4 = rg();
            cd().xD8 = rw();
            let fr = f();
            fr.input.pressed_buttons = rnd() & 0x1FF;
            fr.input.lstick[0].y = rg();
            fr.active_timer.lstick.y = (rnd() % 8) as _;
            fr.item_gobj = if rnd() & 1 != 0 { igo() as *mut Item_GObj } else { core::ptr::null_mut() };
            fr.kind = if rnd() & 1 != 0 { Ft_Kind_Ness as _ } else { (rnd() % 33) as _ };
            fr.set_allow_interrupt((rnd() & 1) as u8);
            let g = go();
            let ret = match op {
                0 => m::checkLStick(f()) as u32,
                1 => m::ftCo_AttackLw4_CheckInput(g) as u32,
                2 => { m::doEnter(g); 0 }
                3 => { m::ftCo_AttackLw4_Anim(g); 0 }
                4 => { m::ftCo_AttackLw4_IASA(g); 0 }
                5 => { m::ftCo_AttackLw4_Phys(g); 0 }
                _ => { m::ftCo_AttackLw4_Coll(g); 0 }
            };
            let mut h = Hs::new();
            h.m(f().allow_interrupt() as u32);
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 7000);
}
