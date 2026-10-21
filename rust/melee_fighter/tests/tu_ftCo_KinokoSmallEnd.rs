//! src/tu/ftCo_KinokoSmallEnd.rs vs the C (gen/cref/tu/ftCo_KinokoSmallEnd.c: the whole TU,
//! shipping contraction; regenerate with
//! `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_KinokoSmallEnd`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_KinokoSmallEnd as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

unsafe extern "C" fn cb0(g: *mut HSD_GObj) { t(90); t(is_go(g)) }
unsafe extern "C" fn cb4(g: *mut HSD_GObj) { t(91); t(is_go(g)) }
fn cbid(e: HSD_GObjEvent) -> u32 {
    let Some(p) = e else { return 0 };
    let p = p as usize;
    if p == cb0 as usize { 1 } else if p == cb4 as usize { 2 } else if p == m::ftCo_800D23F4_cb as usize { 3 } else { 99 }
}
fn evid(cb: fn(*mut HSD_GObj)) -> u32 {
    let p = cb as usize;
    if p == m::fn_800D26A0 as usize { 4 } else if p == m::fn_800D2530 as usize { 5 } else { 99 }
}
fn isf(fr: &Fighter) -> u32 { (fr as *const Fighter == f() as *const Fighter) as u32 }

macro_rules! v { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(is_go(g)) }); )* } } }
macro_rules! fs { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|fr| { t($id); t(isf(fr)) }); )* } } }

#[test]
fn ftCo_KinokoSmallEnd_matches_c() {
    init();
    v!(fn_800D2BF8 = 4, ftCommon_8007E2FC = 6, ft_8008A324 = 7, fn_800D2B30 = 8, fn_800D290C = 13);
    fs!(fn_800D2AD8 = 11, fn_800D2CA0 = 15, fn_800D2CD4 = 16);
    unsafe {
        hk::fn_800D2818 = Some(|fr| { t(1); t(isf(fr)); sr(1) | sr(17) });
        hk::ftCo_800D2770 = Some(|g, s| { t(5); t(is_go(g)); t(b(s)) });
        hk::fn_800D2890 = Some(|g, ms| { t(9); t(is_go(g)); t(ms as u32) });
        hk::fn_800D299C = Some(|g, i, a, bb| { t(10); t(is_go(g)); t(i as u32); t(b(a)); t(b(bb)) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(14); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::fn_800D2A3C = Some(|g| { t(17); t(is_go(g)); sr(18) });
        hk::ft_8008403C = Some(|g, cb| { t(18); t(is_go(g)); t(evid(cb)) });
        hk::ft_80082C74 = Some(|g, cb| { t(19); t(is_go(g)); t(evid(cb)) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_KinokoSmallEnd.txt"));
    for op in 0..10u32 {
        for k in 0..2000u32 {
            case(op, k);
            let fr = f();
            let cdd = cd();
            fr.motion_id = (310 + rnd() % 16) as _;
            fr.set_x2220_b5((rnd() & 1) as u8); fr.set_x2220_b6((rnd() & 1) as u8);
            fr.ground_or_air = (rnd() & 1) as _;
            cdd.x688 = (rnd() % 300) as i32 - 50; cdd.x68C = (rnd() % 300) as i32 - 50; cdd.x690 = (rnd() % 300) as i32 - 50;
            cdd.x678 = rw(); cdd.x67C = rw();
            fr.dmg.x1830_percent = rw();
            fr.x2008 = (rnd() % 1000) as i32;
            fr.x34_scale.x = if rnd() & 1 != 0 { rg() + 1.0 } else { rw() };
            fr.x34_scale.y = rw();
            fr.cur_anim_frame = rw(); fr.frame_speed_mul = rw();
            unsafe {
                fr.mv.co.mushroom.x8 = (rnd() % 3) as i32;
                fr.mv.co.walk.fast_anim_frame = rw();
                fr.mv.co.mushroom.x0 = Some(cb0);
                fr.mv.co.mushroom.x4 = Some(cb4);
            }
            let g = go();
            let mut ret = 0u32;
            match op {
                0 => ret = m::Fighter_PoisonMushroomEnd(g) as u32,
                1 => m::ftCo_800D23F4(g),
                2 => m::ftCo_800D2490(g, (rnd() % 3) as i32),
                3 => m::fn_800D2530(g),
                4 => m::ftCo_KinokoSmallEnd_Anim(g),
                5 => m::ftCo_KinokoSmallEnd_Coll(g),
                6 => m::ftCo_800D2600(g, (rnd() % 3) as i32),
                7 => m::fn_800D26A0(g),
                8 => m::ftCo_KinokoSmallEndAir_Anim(g),
                _ => m::ftCo_KinokoSmallEndAir_Coll(g),
            }
            let fr = f();
            let mut h = Hs::new();
            unsafe { h.m(cbid(fr.mv.co.mushroom.x0)); h.m(cbid(fr.mv.co.mushroom.x4)); h.m(fr.mv.co.mushroom.x8 as u32); }
            h.m(cbid(fr.take_dmg_cb)); h.m(fr.x2220_b5() as u32); h.m(fr.x2220_b6() as u32); h.m(fr.x2008 as u32);
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 20000);
}
