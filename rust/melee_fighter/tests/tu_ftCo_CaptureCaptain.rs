//! src/tu/ftCo_CaptureCaptain.rs vs the C (gen/cref/tu/ftCo_CaptureCaptain.c: the whole TU,
//! shipping contraction; regenerate with
//! `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_CaptureCaptain`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_CaptureCaptain as m;
use melee_fighter::tu::ftCo_Barrel::ext::hook as bk;
use m::ext::hook as hk;
use melee_fighter::types::*;

unsafe extern "C" fn db464(_: *mut HSD_GObj) {}
fn isf(p: *const Fighter) -> u32 { (p == f() as *const Fighter) as u32 }

#[test]
fn ftCo_CaptureCaptain_matches_c() {
    init();
    unsafe {
        bk::ftCommon_8007DB58 = Some(|g| { t(1); t(is_go(g)) });
        bk::ftCo_8009750C = Some(|g| { t(2); t(is_go(g)) });
        bk::ftCo_800DD168 = Some(|g| { t(3); t(is_go(g)) });
        hk::ftCo_800DB368 = Some(|v, fr| { t(4); t((v == f2() as *mut Fighter) as u32); t(isf(fr)) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(5); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t((o == go2()) as u32)
        });
        hk::ftCo_800DB464_fn = Some(|| Some(db464));
        hk::ftCommon_8007E2F4 = Some(|fr, v| { t(6); t(isf(fr)); t(v as i32 as u32) });
        hk::ftAnim_8006EBA4 = Some(|g| { t(7); t(is_go(g)) });
        hk::ftCommon_8007E2FC = Some(|g| { t(8); t(is_go(g)) });
        hk::ft_80083B68 = Some(|g| { t(9); t(is_go(g)) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_CaptureCaptain.txt"));
    for op in 0..5u32 {
        for k in 0..1000u32 {
            case(op, k);
            let fr = f();
            fr.anon_221b.set_x221B_b5((rnd() & 1) as u8);
            fr.anon_221b.set_x221B_b7((rnd() & 1) as u8);
            fr.set_x2226_b2((rnd() & 1) as u8);
            fr.ground_or_air = (rnd() & 1) as _;
            fr.facing_dir = rg();
            f2().facing_dir = rg();
            let g = go();
            match op {
                0 => m::ftCo_8009CA0C(g, go2()),
                1 => m::ftCo_CaptureCaptain_Anim(g),
                2 => m::ftCo_CaptureCaptain_IASA(g),
                3 => m::ftCo_CaptureCaptain_Phys(g),
                _ => m::ftCo_CaptureCaptain_Coll(g),
            }
            let fr = f();
            let mut h = Hs::new();
            h.m((fr.victim_gobj == go2()) as u32);
            h.m((fr.x1A5C == go2()) as u32);
            h.m(fr.anon_221b.x221B_b5() as u32);
            h.m(fr.anon_221b.x221B_b7() as u32);
            h.m(b(fr.facing_dir));
            h.m((fr.accessory1_cb.map(|p| p as usize) == Some(db464 as usize)) as u32);
            h.m(fr.accessory1_cb.is_none() as u32);
            c.line(op, k, 0, h.0);
        }
    }
    assert_eq!(c.done(), 5000);
}
