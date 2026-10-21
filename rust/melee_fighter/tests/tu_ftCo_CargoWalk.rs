//! src/tu/ftCo_CargoWalk.rs vs the C (gen/cref/tu/ftCo_CargoWalk.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_CargoWalk`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_CargoWalk as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

static mut CBV: f32 = 0.0;

#[test]
fn ftCo_CargoWalk_matches_c() {
    init();
    let dk = Box::leak(Box::new(ftDonkeyAttributes::zeroed()));
    unsafe {
        hk::ftWalkCommon_800DFC70 = Some(|g| { t(1); t(is_go(g)); sr(1) });
        hk::ftWalkCommon_800DFCA4 = Some(|g, ms, fl, s, a, bb, c, d, e, ff, am| {
            t(2); t(is_go(g)); t(ms as u32); t(fl); t(b(s)); t(b(a)); t(b(bb)); t(b(c)); t(b(d)); t(b(e)); t(b(ff)); t(b(am))
        });
        hk::ftWalkCommon_GetWalkType = Some(|g| { t(3); t(is_go(g)); SRV[7] % 3 });
        hk::ftWalkCommon_800DFDDC = Some(|g| { t(4); t(is_go(g)) });
        hk::ftWalkCommon_800DFEC8 = Some(|g, cb| { t(5); t(is_go(g)); cb(g, CBV) });
        hk::ftWalkCommon_800E0060 = Some(|g| { t(6); t(is_go(g)) });
        hk::ftCo_8009C5A4 = Some(|g, ms| { t(7); t((g == go2()) as u32); t(ms as u32) });
        hk::ftCo_8009BF3C = Some(|_| { t(8); sr(2) });
        hk::ftCo_8009BB1C = Some(|_| { t(9); sr(3) });
        hk::ftCo_8009C4F8 = Some(|_| { t(10); sr(4) });
        hk::ftCo_8009B4D0 = Some(|_| { t(11); sr(5) });
        hk::ftCo_CargoWait_Coll = Some(|g| { t(12); t(is_go(g)) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_CargoWalk.txt"));
    for op in 0..7u32 {
        for k in 0..2000u32 {
            case(op, k);
            *dk = ftDonkeyAttributes::zeroed();
            dk.x4_motion_state = (rnd() % 400) as i32;
            dk.x8 = rw(); dk.xC = rw(); dk.x10 = rw(); dk.x14 = rw(); dk.x18 = rw(); dk.x1C = rw();
            unsafe { CBV = rw(); }
            f().x2CC = dk as *mut _;
            f().victim_gobj = go2();
            let g = go();
            let ret = match op {
                0 => m::ftCo_8009B67C(g) as u32,
                1 => { m::ftCo_8009B6C8(g, unsafe { CBV }); 0 }
                2 => { m::ftCo_CargoWalk_Anim(g); 0 }
                3 => { m::ftCo_CargoWalk_IASA(g); 0 }
                4 => { m::ftCo_CargoWalk_Phys(g); 0 }
                5 => { m::ftCo_CargoWalk_Coll(g); 0 }
                _ => { m::ftCo_8009B6C8(g, 0.0); 0 }
            };
            let mut h = Hs::new();
            h.m(f().motion_id as u32);
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 14000);
}
