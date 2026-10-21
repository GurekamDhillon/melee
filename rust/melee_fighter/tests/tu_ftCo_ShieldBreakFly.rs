//! src/tu/ftCo_ShieldBreakFly.rs vs the C (gen/cref/tu/ftCo_ShieldBreakFly.c: the whole TU,
//! shipping contraction; regenerate with
//! `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_ShieldBreakFly`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_ftco_s209;
use tu_ftco_harness::*;
use tu_ftco_s209::*;
use melee_fighter::tu::ftCo_ShieldBreakFly as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

macro_rules! vs { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(pi(g)) }); )* } } }

#[test]
fn ftCo_ShieldBreakFly_matches_c() {
    s209_init();
    unsafe {
        hk::ftCommon_8007D5D4 = Some(|f| { t(1); t(pi(f as *const Fighter)) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(2); t(pi(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(pi(o))
        });
        hk::efAsync_Spawn = Some(|g, q, kind, gfx, j| { t(4); t(pi(g)); t(pi(q)); t(kind); t(gfx); t(pi(j)) });
        hk::ft_PlaySFX = Some(|f, id, v, p| { t(5); t(pi(f as *const Fighter)); t(id as u32); t(v as u32); t(p as u32) });
        hk::ftCommon_8007EBAC = Some(|f, a, bb| { t(6); t(pi(f as *const Fighter)); t(a); t(bb) });
        hk::ftColl_8007B62C = Some(|g, n| { t(7); t(pi(g)); t(n as u32) });
        hk::ftAnim_IsFramesRemaining = Some(|g| { t(8); t(pi(g)); sr(8) });
        hk::ft_80082C74 = Some(|g, cb| {
            t(11); t(pi(g));
            t((cb as usize == melee_fighter::tu::ftCo_ShieldBreakDown::ftCo_80098E3C as usize) as u32)
        });
    }
    vs!(ftAnim_8006EBA4 = 3, ftCo_80098D90 = 9, ft_80084EEC = 10);
    let mut c = Check::new(include_str!("data/tu_ftCo_ShieldBreakFly.txt"));
    for op in 0..6u32 {
        for k in 0..1000u32 {
            case(op, k);
            s209_fill();
            let g = go();
            match op {
                0 => if k & 1 != 0 { m::ftCo_80098C9C(g) } else { m::ftCo_80098B20(g) },
                1 => m::ftCo_ShieldBreakFly_Anim(g),
                2 => m::ftCo_ShieldBreakFly_IASA(g),
                3 => m::ftCo_ShieldBreakFly_Phys(g),
                4 => m::ftCo_ShieldBreakFly_Coll(g),
                _ => {
                    if k & 1 != 0 { f().kind = Ft_Kind_Purin as _; }
                    m::ftCo_80098B20(g)
                }
            }
            c.line(op, k, 0, state_hash());
        }
    }
    assert_eq!(c.done(), 6000);
}
