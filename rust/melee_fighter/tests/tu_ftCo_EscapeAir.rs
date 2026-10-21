//! src/tu/ftCo_EscapeAir.rs vs the C (gen/cref/tu/ftCo_EscapeAir.c: the whole TU incl. the
//! TARGET_PC caps check, shipping contraction, the real gekko_fp.c cosf/sinf against group 1's
//! port; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_EscapeAir`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
#[macro_use]
mod tu_ftco_dfe;
use tu_ftco_harness::*;
use tu_ftco_dfe::*;
use melee_fighter::tu::ftCo_EscapeAir as m;
use m::ext::hook as hk;

#[test]
fn ftCo_EscapeAir_matches_c() {
    init();
    df_init();
    unsafe {
        hk::ScriptGame_CapsActionAllowed = Some(|p, fl| { t(1); t(is_f(p)); t(fl as u32); chk(1) as i32 });
        hk::ftCo_800D705C = dchk!(2);
        hk::ftCommon_8007D9D4 = Some(|p| { t(3); t(is_f(p)); rw() });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(6); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftAnim_8006EBA4 = dv!(7);
        hk::ftCommon_8007EBAC = Some(|p, a, bb| { t(8); t(is_f(p)); t(a); t(bb); });
        hk::ftAnim_IsFramesRemaining = dchk!(9);
        hk::ftCo_80096900 = Some(|g, a1, a2, ai, mob, ll| { t(10); t(is_go(g)); t(a1 as u32); t(a2 as u32); t(ai as u32); t(b(mob)); t(b(ll)); });
        hk::ftCo_80095328 = Some(|g, bp| {
            t(11); t(is_go(g)); t((!bp.is_null()) as u32);
            if !bp.is_null() { *bp = chk(12); }
            chk(11)
        });
        hk::ftCo_800C3B10 = dchk!(13);
        hk::ft_80084DB0 = dv!(14);
        hk::ft_80082C74 = Some(|g, cb| {
            t(15); t(is_go(g));
            t((cb.map(|x| x as usize) == Some(m::ftCo_80099D70 as usize)) as u32);
        });
        hk::ftCo_LandingFallSpecial_Enter = Some(|g, a, bb| { t(16); t(is_go(g)); t(a as u32); t(b(bb)); });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_EscapeAir.txt"));
    for op in 0..8u32 {
        for k in 0..3000u32 {
            case(op, k);
            df_fill();
            let timer = (rnd() % 3) as i32;
            let g = go();
            let ret = match op {
                0 => m::ftCo_80099A58(g) as u32,
                1 => { m::ftCo_80099A9C(g, timer); 0 }
                2 => { m::inlineA0(f()); 0 }
                3 => { m::ftCo_EscapeAir_Anim(g); 0 }
                4 => { m::ftCo_EscapeAir_IASA(g); 0 }
                5 => { m::ftCo_EscapeAir_Phys(g); 0 }
                6 => { m::ftCo_EscapeAir_Coll(g); 0 }
                _ => { m::ftCo_80099D70(g); 0 }
            };
            c.line(op, k, ret, df_hash().0);
        }
    }
    assert_eq!(c.done(), 24000);
}
