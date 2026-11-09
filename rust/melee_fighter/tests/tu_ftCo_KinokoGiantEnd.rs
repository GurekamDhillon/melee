//! src/tu/ftCo_KinokoGiantEnd.rs vs the C (gen/cref/tu/ftCo_KinokoGiantEnd.c: the whole TU,
//! shipping contraction; regenerate with
//! `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_KinokoGiantEnd`).
#![allow(static_mut_refs, non_snake_case)]
#[path = "../../melee_core/test_support/fnsym.rs"]
mod fnsym;
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_KinokoGiantEnd as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

unsafe extern "C" fn mush(g: *mut HSD_GObj) { t(17); t(is_go(g)) }

fn cbid(cb: HSD_GObjEvent) -> u32 {
    match cb { None => 0, Some(p) if fnsym::same(p as usize, m::ftCo_800D1B04_c as usize) => 1, _ => 9 }
}

#[test]
fn ftCo_KinokoGiantEnd_matches_c() {
    init();
    unsafe {
        hk::fn_800D2818 = Some(|p| { t(1); t((p as *const Fighter == f() as *const Fighter) as u32); sr(0) });
        hk::fn_800D2BF8 = Some(|_| t(2));
        hk::ftCo_800D2770 = Some(|_, s| { t(3); t(b(s)) });
        hk::ftCommon_8007E2FC = Some(|g| { t(4); t(is_go(g)) });
        hk::ft_8008A324 = Some(|_| t(5));
        hk::fn_800D2B30 = Some(|_| t(6));
        hk::fn_800D2890 = Some(|_, id| { t(7); t(id as u32) });
        hk::fn_800D299C = Some(|_, a, bb, c| { t(8); t(a as u32); t(b(bb)); t(b(c)) });
        hk::fn_800D2B04 = Some(|_| t(9));
        hk::fn_800D290C = Some(|_| t(10));
        hk::fn_800D2CA0 = Some(|_| t(11));
        hk::fn_800D2CD4 = Some(|_| t(12));
        hk::fn_800D2A3C = Some(|_| { t(13); sr(1) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| { t(14); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32) });
        hk::ft_8008403C = Some(|g, cb| { t(15); t((fnsym::same(cb as usize, m::fn_800D1DB0 as usize)) as u32); cb(g) });
        hk::ft_80082C74 = Some(|g, cb| { t(16); t((fnsym::same(cb as usize, m::fn_800D1C40 as usize)) as u32); cb(g) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_KinokoGiantEnd.txt"));
    for op in 0..10u32 {
        for k in 0..1000u32 {
            case(op, k);
            let fr = f();
            fr.ground_or_air = rnd() & 1;
            fr.motion_id = (rnd() % 400) as i32;
            fr.x34_scale.x = rw();
            fr.x34_scale.y = rw();
            fr.cur_anim_frame = rw();
            fr.frame_speed_mul = rw();
            fr.mv.co.mushroom.x8 = (rnd() & 1) as i32;
            fr.mv.co.mushroom.x0 = Some(mush);
            fr.set_x2220_b5(1);
            let g = go();
            let ret = match op {
                0 => m::Fighter_SuperMushroomEnd(g) as u32,
                1 => { m::ftCo_800D1B04(g); 0 }
                2 => { let a = (rnd() % 3) as i32; m::ftCo_800D1BA0(g, a); 0 }
                3 => { m::fn_800D1C40(g); 0 }
                4 => { m::ftCo_KinokoGiantEnd_Anim(g); 0 }
                5 => { m::ftCo_KinokoGiantEnd_Coll(g); 0 }
                6 => { let a = (rnd() % 3) as i32; m::ftCo_800D1D10(g, a); 0 }
                7 => { m::fn_800D1DB0(g); 0 }
                8 => { m::ftCo_KinokoGiantEndAir_Anim(g); 0 }
                _ => { m::ftCo_KinokoGiantEndAir_Coll(g); 0 }
            };
            let fr = f();
            let mut h = Hs::new();
            h.m(fr.x2220_b5() as u32);
            unsafe {
                h.m(fr.mv.co.mushroom.x8 as u32);
                h.m(cbid(fr.mv.co.mushroom.x4));
            }
            h.m(cbid(fr.take_dmg_cb));
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 10000);
}
