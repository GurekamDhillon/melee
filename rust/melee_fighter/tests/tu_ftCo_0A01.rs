//! src/tu/ftCo_0A01.rs vs the C (gen/cref/tu/ftCo_0A01.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_0A01`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_0A01 as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

#[test]
fn ftCo_0A01_matches_c() {
    init();
    unsafe {
        hk::Player_8003248C = Some(|s, sub| { t(1); t(s as u32); t(sub as u32); if sr(0) { Gm_PKind_Cpu as _ } else { Gm_PKind_Human as _ } });
        hk::lbColl_800083C4 = Some(|h| unsafe {
            t(2);
            (*h).a_pos.x = rw(); (*h).a_pos.y = rw();
            (*h).b_pos.x = rw(); (*h).b_pos.y = rw();
        });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_0A01.txt"));
    for op in 0..9u32 {
        for k in 0..2000u32 {
            case(op, k);
            let fr = f();
            fr.cpu.lstick.x = rnd() as i8; fr.cpu.lstick.y = rnd() as i8;
            fr.cpu.cstick.x = rnd() as i8; fr.cpu.cstick.y = rnd() as i8;
            fr.cpu.ltrigger = rnd() as u8; fr.cpu.rtrigger = rnd() as u8;
            fr.cpu.buttons = rnd() as _;
            fr.cpu.kind = (rnd() % 12) as _;
            fr.player_idx = (rnd() % 6) as _;
            fr.set_is_sub_fighter((rnd() & 1) as _);
            fr.hurt_capsules_len = (rnd() % 16) as _;
            fr.cur_pos.x = rw(); fr.cur_pos.y = rw();
            fr.x34_scale.y = rw();
            fr.facing_dir = rg();
            for i in 0..15 { fr.hurt_capsules[i].capsule.scale = rw(); }
            fr.cpu.x55C = rw(); fr.cpu.x560 = rw(); fr.cpu.x564 = rw(); fr.cpu.x568 = rw();
            let mut h = Hs::new();
            let mut r = 0f32;
            let mut ri = 0u32;
            unsafe {
                match op {
                    0 => r = m::ftCo_GetCpuLStickX(fr),
                    1 => r = m::ftCo_GetCpuLStickY(fr),
                    2 => r = m::ftCo_GetCpuCStickX(fr),
                    3 => r = m::ftCo_GetCpuCStickY(fr),
                    4 => r = m::ftCo_GetCpuLTrigger(fr),
                    5 => r = m::ftCo_GetCpuRTrigger(fr),
                    6 => ri = m::ftCo_GetCpuButtons(fr),
                    7 => ri = m::ftCo_IsCpuControlled(fr) as u32,
                    _ => m::ftCo_800A0DA4(fr),
                }
            }
            let hf = |h: &mut Hs, v: f32| h.m(v.to_bits());
            hf(&mut h, r); h.m(ri);
            hf(&mut h, fr.cpu.x55C); hf(&mut h, fr.cpu.x560); hf(&mut h, fr.cpu.x564); hf(&mut h, fr.cpu.x568);
            for i in 0..15 { hf(&mut h, fr.hurt_capsules[i].capsule.a_pos.x); hf(&mut h, fr.hurt_capsules[i].capsule.b_pos.y); }
            c.line2(op, k, h.0);
        }
    }
    assert_eq!(c.done(), 18000);
}
