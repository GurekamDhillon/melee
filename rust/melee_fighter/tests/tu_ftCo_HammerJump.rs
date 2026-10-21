//! src/tu/ftCo_HammerJump.rs vs the C (gen/cref/tu/ftCo_HammerJump.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_HammerJump`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_HammerJump as m;
use m::ext::hook as hk;

static mut FR: f32 = 0.0;
static mut FL: u32 = 0;
static mut JI: u32 = 0;
fn isf(p: &mut melee_fighter::types::Fighter) -> u32 { (p as *mut _ == unsafe { F }) as u32 }

#[test]
fn ftCo_HammerJump_matches_c() {
    init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| { t(1); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32) });
        hk::ftCo_800C54C4 = Some(|p| { t(2); t(isf(p)); FL });
        hk::ftCo_800C548C = Some(|p| { t(3); t(isf(p)); FR });
        hk::ftCo_800C4E94 = Some(|p| { t(4); t(isf(p)) });
        hk::ftCo_800C4F64 = Some(|g| { t(5); t(is_go(g)) });
        hk::ftCo_Jump_GetInput = Some(|g| { t(6); t(is_go(g)); JI });
        hk::ftCo_800C5B88 = Some(|g, e| { t(7); t(is_go(g)); t(e as u32) });
        hk::ftCommon_8007D5D4 = Some(|p| { t(8); t(isf(p)) });
        hk::ftCo_800CB110 = Some(|g, bb, v| { t(9); t(is_go(g)); t(bb as u32); t(b(v)) });
        hk::ftCo_Jump_Phys_Inner = Some(|g| { t(10); t(is_go(g)) });
        hk::ftCo_HammerFall_Coll = Some(|g| { t(11); t(is_go(g)) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_HammerJump.txt"));
    for op in 0..6u32 {
        for k in 0..2000u32 {
            case(op, k);
            unsafe { FR = rw(); FL = rnd(); JI = rnd() % 4; }
            let g = go();
            let ret = match op {
                0 => m::ftCo_800C5A50(g) as u32,
                1 => { m::ftCo_800C5A98(g); 0 }
                2 => { m::ftCo_HammerJump_Anim(g); 0 }
                3 => { m::ftCo_HammerJump_IASA(g); 0 }
                4 => { m::ftCo_HammerJump_Phys(g); 0 }
                _ => { m::ftCo_HammerJump_Coll(g); 0 }
            };
            c.line(op, k, ret, Hs::new().0);
        }
    }
    assert_eq!(c.done(), 12000);
}
