//! src/tu/ftCo_CargoWait.rs vs the C (gen/cref/tu/ftCo_CargoWait.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_CargoWait`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_CargoWait as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

#[test]
fn ftCo_CargoWait_matches_c() {
    init();
    let dk = Box::leak(Box::new(ftDonkeyAttributes::zeroed()));
    unsafe {
        hk::ft_8008A1FC = Some(|g| { t(1); t(is_go(g)); sr(1) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(2); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftCo_8009C5A4 = Some(|g, ms| { t(3); t((g == go2()) as u32); t(ms as u32) });
        hk::ftCo_8009C640 = Some(|g, ms| { t(4); t((g == go2()) as u32); t(ms as u32) });
        hk::ftCo_8009BF3C = Some(|_| { t(5); sr(2) });
        hk::ftCo_8009BB1C = Some(|_| { t(6); sr(3) });
        hk::ftCo_8009C4F8 = Some(|_| { t(7); sr(4) });
        hk::ftCo_8009B818 = Some(|_| { t(8); sr(5) });
        hk::ftCo_8009B67C = Some(|_| { t(9); sr(6) });
        hk::ft_80084F3C = Some(|g| { t(10); t(is_go(g)) });
        hk::ftCo_8009BC58 = Some(|g| { t(11); t(is_go(g)) });
        hk::ft_8008403C = Some(|g, cb| {
            t(12); t(is_go(g));
            t((cb.map(|p| p as usize) == Some(m::ftCo_8009BC58_cb as usize)) as u32);
            unsafe { (cb.unwrap())(g) }
        });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_CargoWait.txt"));
    for op in 0..8u32 {
        for k in 0..2000u32 {
            case(op, k);
            *dk = ftDonkeyAttributes::zeroed();
            dk.x4_motion_state = (rnd() % 400) as i32;
            f().x2CC = dk as *mut _;
            f().victim_gobj = go2();
            let g = go();
            let ret = match op {
                0 => m::ftCo_8009B4D0(g) as u32,
                1 => { m::ftCo_8009B518(g); 0 }
                2 => { m::ftCo_8009B56C(g); 0 }
                3 => { m::ftCo_CargoWait_Anim(g); 0 }
                4 => { m::ftCo_CargoWait_IASA(g); 0 }
                5 => { m::ftCo_CargoWait_Phys(g); 0 }
                6 => { m::ftCo_CargoWait_Coll(g); 0 }
                _ => { m::ftCo_8009B518(g); m::ftCo_8009B56C(g); 0 }
            };
            let mut h = Hs::new();
            h.m(f().motion_id as u32);
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 16000);
}
