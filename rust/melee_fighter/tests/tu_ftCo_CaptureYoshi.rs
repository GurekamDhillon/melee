//! src/tu/ftCo_CaptureYoshi.rs vs the C (gen/cref/tu/ftCo_CaptureYoshi.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_CaptureYoshi`).
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
use melee_fighter::tu::ftCo_CaptureYoshi as m;
use m::ext::hook as hk;

#[test]
fn ftCo_CaptureYoshi_matches_c() {
    cargo_init();
    v!(ftCommon_8007DB58 = 1, ftCo_8009750C = 2, ftCo_800DD168 = 3, ftCo_800DB464 = 5, ftAnim_8006EBA4 = 8,
       ftCommon_8007E2FC = 10);
    unsafe {
        hk::Fighter_ChangeMotionState = Some(CMS);
        hk::ftCo_800DB368 = Some(|a, bb| { t(4); t(pi(a)); t(pi(bb)) });
        hk::ftCommon_8007D5D4 = Some(|f| { t(6); t(pf(f)) });
        hk::ftCommon_8007E2F4 = Some(|f, x| { t(7); t(pf(f)); t(x as i32 as u32) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_CaptureYoshi.txt"));
    for op in 0..5u32 {
        for k in 0..1000u32 {
            case(op, k);
            cargo_fill();
            let g = go();
            let ret: u32 = match op {
                0 => {
                    let fr = f();
                    fr.victim_gobj = core::ptr::null_mut();
                    fr.x221B.set_x221B_b5(1);
                    fr.x221B.set_x221B_b7((rnd() & 1) as u8);
                    m::ftCo_800BBB8C(g, go2());
                    0
                }
                1 => { m::ftCo_CaptureYoshi_Anim(g); 0 }
                2 => { m::ftCo_CaptureYoshi_IASA(g); 0 }
                3 => { m::ftCo_CaptureYoshi_Phys(g); 0 }
                _ => { m::ftCo_CaptureYoshi_Coll(g); 0 }
            };
            c.line(op, k, ret, state_hash());
        }
    }
    assert_eq!(c.done(), 5000);
}
