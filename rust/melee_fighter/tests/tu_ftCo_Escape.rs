//! src/tu/ftCo_Escape.rs vs the C (gen/cref/tu/ftCo_Escape.c: the whole TU incl. the TARGET_PC
//! UCF checks, shipping contraction; regenerate with
//! `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_Escape`).
#![allow(static_mut_refs, non_snake_case)]
#[path = "../../melee_core/test_support/fnsym.rs"]
mod fnsym;
mod tu_ftco_harness;
#[macro_use]
mod tu_ftco_dfe;
use tu_ftco_harness::*;
use tu_ftco_dfe::*;
use melee_fighter::types::*;
use melee_fighter::tu::ftCo_Escape as m;
use m::ext::hook as hk;

fn cbid(p: HSD_GObjEvent) -> u32 {
    match p {
        None => 0,
        Some(x) if fnsym::same(x as usize, m::ftCo_80099564 as usize) => 1,
        Some(x) if fnsym::same(x as usize, m::ftCo_80099754 as usize) => 2,
        Some(x) if fnsym::same(x as usize, m::ftCo_80099644 as usize) => 3,
        _ => 99,
    }
}
fn fine() -> f32 { ((rnd() % 20001) as i32 - 10000) as f32 / 10000.0 }

#[test]
fn ftCo_Escape_matches_c() {
    init();
    df_init();
    unsafe {
        hk::ftCo_800DF8B0 = Some(|p| { t(1); t(is_f(p)); chk(1) });
        hk::ftCo_800DF8E8 = Some(|p| { t(2); t(is_f(p)); chk(2) });
        hk::ftCommon_8007EBAC = Some(|p, a, bb| { t(3); t(is_f(p)); t(a); t(bb); });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(4); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftAnim_8006EBA4 = dv!(5);
        hk::ftParts_80074B0C = Some(|g, a, bb| { t(6); t(is_go(g)); t(a as u32); t(bb as u32); });
        hk::ftYs_Init_8012BDA0 = dv!(7);
        hk::ftAnim_IsFramesRemaining = dchk!(8);
        hk::ft_8008A2BC = dv!(9);
        hk::ftSs_SpecialLw_8012AEBC = dv!(10);
        hk::ftSs_SpecialLw_8012AF38 = dv!(11);
        hk::ftYs_Shield_8012CC1C = dchk!(12);
        hk::ftYs_Init_8012BE3C = dv!(13);
        hk::ftYs_Init_8012B8A4 = dv!(14);
        hk::ftCo_8009563C = dchk!(15);
        hk::ft_80085004 = dv!(16);
        hk::ft_80084104 = dv!(17);
        hk::ft_800847D0 = Some(|g, bx| {
            t(18); t(is_go(g));
            t((bx == core::ptr::addr_of_mut!((*(dat() as *mut ftSs_DatAttrs)).height_attributes)) as u32);
        });
        hk::Replay_UcfVersion = Some(|port| { t(19); t(port as u32); [73, 84, 0][(SRV[7] % 3) as usize] });
        hk::ft_80084F3C = dv!(20);
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_Escape.txt"));
    for op in 0..26u32 {
        for k in 0..2000u32 {
            case(op, k);
            df_fill();
            let ms = (rnd() % 400) as i32;
            let a2 = (rnd() % 3) as i32;
            let v = if rnd() & 1 != 0 { rg() } else { fine() };
            if rnd() & 1 != 0 {
                f().input.lstick[0].x = fine();
                f().input.lstick[0].y = fine();
            }
            let g = go();
            let ret = match op {
                0 => m::ftCo_8009917C(g) as u32,
                1 => m::ftCo_80099264(g) as u32,
                2 => { m::ftCo_800992A8(g, ms, a2); 0 }
                3 => { m::ftCo_80099314(g, ms, a2); 0 }
                4 => { m::ftCo_80099390(g, ms, a2); 0 }
                5 => { m::ftCo_80099438(g, ms, a2); 0 }
                6 => { m::ftCo_Escape_Anim(g); 0 }
                7 => { m::ftCo_80099564(g); 0 }
                8 => { m::ftCo_80099644(g); 0 }
                9 => { m::ftCo_Escape_IASA(g); 0 }
                10 => { m::ftCo_Escape_Phys(g); 0 }
                11 => { m::ftCo_Escape_Coll(g); 0 }
                12 => { m::ftCo_80099754(g); 0 }
                13 => b(m::ftCo_Ucf073RimAxis(v)),
                14 => m::ftCo_Ucf073BlocksSpotdodge(f()) as u32,
                15 => m::ftCo_Ucf084RimUnits(v) as u32,
                16 => m::ftCo_Ucf084BlocksSpotdodge(f()) as u32,
                17 => m::ftCo_80099794(g) as u32,
                18 => m::ftCo_8009980C(g) as u32,
                19 => { m::ftCo_80099894(g); 0 }
                20 => { m::ftCo_800998EC(g); 0 }
                21 => { m::ftCo_80099954(g); 0 }
                22 => { m::ftCo_EscapeN_Anim(g); 0 }
                23 => { m::ftCo_EscapeN_IASA(g); 0 }
                24 => { m::ftCo_EscapeN_Phys(g); 0 }
                _ => { m::ftCo_EscapeN_Coll(g); 0 }
            };
            let mut h = df_hash();
            h.m(cbid(f().anim_cb));
            h.m(cbid(f().coll_cb));
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 52000);
}
