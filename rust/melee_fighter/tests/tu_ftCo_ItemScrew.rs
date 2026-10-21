//! src/tu/ftCo_ItemScrew.rs vs the C (gen/cref/tu/ftCo_ItemScrew.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_ItemScrew`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_ItemScrew as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

static mut V: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

fn is_f(p: &mut Fighter) -> u32 { (p as *mut Fighter == f() as *mut Fighter) as u32 }

#[test]
fn ftCo_ItemScrew_matches_c() {
    init();
    unsafe {
        hk::ftCommon_8007D5D4 = Some(|p| { t(1); t(is_f(p)) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| { t(2); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32) });
        hk::ftCo_800CB110 = Some(|g, a, mm| { t(3); t(is_go(g)); t(a as u32); t(b(mm)) });
        hk::ftAnim_IsFramesRemaining = Some(|g| { t(4); t(is_go(g)); sr(1) });
        hk::ftCo_Fall_Enter = Some(|g| { t(5); t(is_go(g)) });
        hk::ftCo_FallAerial_Enter = Some(|g| { t(6); t(is_go(g)) });
        hk::ftCo_Jump_IASA = Some(|g| { t(7); t(is_go(g)) });
        hk::ftCo_Jump_Phys_Inner = Some(|g| { t(8); t(is_go(g)) });
        hk::ftCo_JumpAerial_IASA = Some(|g| { t(9); t(is_go(g)) });
        hk::ft_80084DB0 = Some(|g| { t(10); t(is_go(g)) });
        hk::ftCo_800CBAC4 = Some(|g, ms, v, a| { t(11); t(is_go(g)); t(ms as u32); t((v as *const Vec3 == &raw const V) as u32); t(b(v.x)); t(b(v.y)); t(b(v.z)); t(a as u32) });
        hk::ftCo_80096CC8 = Some(|_, _| { t(12); false });
        hk::ft_80082B1C = Some(|_| t(13));
        hk::ft_800831CC = Some(|g, a, bb| { t(14); t(is_go(g)); t((a as usize == m::ext::ftCo_80096CC8 as usize) as u32); t((bb as usize == m::ext::ft_80082B1C as usize) as u32) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_ItemScrew.txt"));
    for op in 0..10u32 {
        for k in 0..1000u32 {
            case(op, k);
            cd().x800 = rw();
            unsafe { V.x = rw(); V.y = rw(); V.z = rw(); }
            let g = go();
            match op {
                0 => m::ftCo_ItemScrew_Enter(g),
                1 => m::ftCo_ItemScrew_Anim(g),
                2 => m::ftCo_ItemScrew_IASA(g),
                3 => m::ftCo_ItemScrew_Phys(g),
                4 => m::ftCo_ItemScrew_Coll(g),
                5 => m::ft_800D2E7C(g, unsafe { &mut *(&raw mut V) }),
                6 => m::ftCo_ItemScrewAir_Anim(g),
                7 => m::ftCo_ItemScrewAir_IASA(g),
                8 => m::ftCo_ItemScrewAir_Phys(g),
                _ => m::ftCo_ItemScrewAir_Coll(g),
            }
            let mut h = Hs::new();
            h.m(f().x2227_b0() as u32);
            unsafe { h.m(b(V.x)); h.m(b(V.y)); h.m(b(V.z)); }
            c.line(op, k, 0, h.0);
        }
    }
    assert_eq!(c.done(), 10000);
}
