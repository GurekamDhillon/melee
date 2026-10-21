//! src/tu/ftCo_CaptureWaitKirby.rs vs the C (gen/cref/tu/ftCo_CaptureWaitKirby.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_CaptureWaitKirby`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_ftco_cargo;
use tu_ftco_harness::*;
use tu_ftco_cargo::*;
use melee_fighter::types::*;

macro_rules! v { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(pi(g)) }); )* } } }
const CMS: fn(*mut HSD_GObj, i32, u32, f32, f32, f32, *mut HSD_GObj) = |g, ms, fl, a, bb, c, o| {
    t(9); t(pi(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(pi(o))
};
fn kf(i: usize) -> f32 { ((unsafe { SRV[i & 15] } % 41) as i32 - 20) as f32 / 4.0 }
fn pf(f: &mut Fighter) -> u32 { pi(f as *const Fighter) }
use melee_fighter::tu::ftCo_CaptureWaitKirby as m;
use m::ext::hook as hk;

#[test]
fn ftCo_CaptureWaitKirby_matches_c() {
    cargo_init();
    v!(Fighter_UpdateModelScale = 2, ftCo_800DB464 = 3, ftAnim_8006EBA4 = 7, ftCommon_8007E2FC = 8,
       ftKb_SpecialN_800F5954 = 12, ftCo_CaptureCut_Enter = 20);
    unsafe {
        hk::Fighter_ChangeMotionState = Some(CMS);
        hk::ftCo_800DB368 = Some(|a, bb| { t(1); t(pi(a)); t(pi(bb)) });
        hk::ftColl_8007B62C = Some(|g, x| { t(4); t(pi(g)); t(x as u32) });
        hk::ftCommon_8007D5D4 = Some(|f| { t(5); t(pf(f)) });
        hk::ftCommon_8007E2F4 = Some(|f, x| { t(6); t(pf(f)); t(x as i32 as u32) });
        hk::ftKb_SpecialN_800F598C = Some(|g, a| { t(10); t(pi(g)); t(a as u32) });
        hk::ftKb_SpecialN_800F597C = Some(|g| { t(11); t(pi(g)); sr(11) });
        hk::ftKb_SpecialN_800F5A04 = Some(|g, a| { t(13); t(pi(g)); t(b(a)) });
        hk::ftKb_SpecialN_800F5A38 = Some(|g, x| { t(14); t(pi(g)); t(pi(x)); sr(14) });
        hk::ftKb_SpecialN_800F5B00 = Some(|g| { t(15); t(pi(g)); kf(15) });
        hk::ftKb_SpecialN_800F5B10 = Some(|g| { t(16); t(pi(g)); kf(0) });
        hk::ftKb_SpecialN_800F5A60 = Some(|g| { t(17); t(pi(g)); kf(1) });
        hk::ftCommon_GrabMash = Some(|f, a| { t(18); t(pf(f)); t(b(a)); sr(18) });
        hk::HSD_JObjSetMtxDirtySub = Some(|j| { t(19); t((j == JO) as u32) });
        hk::ftCo_800DA698 = Some(|g, x| { t(21); t(pi(g)); t(x as u32) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_CaptureWaitKirby.txt"));
    for op in 0..7u32 {
        for k in 0..1000u32 {
            case(op, k);
            cargo_fill();
            let g = go();
            let ret: u32 = match op {
                0 => { m::ftCo_800BD620(g); 0 }
                1 => { m::ftCo_CaptureWaitKirby_Anim(g); 0 }
                2 => { m::ftCo_800BD6EC_noinline(g); 0 }
                3 => { m::ftCo_800BD6EC(g); 0 }
                4 => { m::ftCo_CaptureWaitKirby_IASA(g); 0 }
                5 => { m::ftCo_CaptureWaitKirby_Phys(g); 0 }
                _ => { m::ftCo_CaptureWaitKirby_Coll(g); 0 }
            };
            c.line(op, k, ret, state_hash());
        }
    }
    assert_eq!(c.done(), 7000);
}
