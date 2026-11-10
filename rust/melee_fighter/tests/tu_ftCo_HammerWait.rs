//! src/tu/ftCo_HammerWait.rs vs the C (gen/cref/tu/ftCo_HammerWait.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_HammerWait`).
#![allow(static_mut_refs, non_snake_case)]
#[path = "../../melee_core/test_support/fnsym.rs"]
mod fnsym;
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_HammerWait as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

fn itid(p: *mut Item_GObj) -> u32 {
    if p.is_null() { 0 } else if p as *mut HSD_GObj == igo() { 1 } else { 99 }
}
fn isf(p: &Fighter) -> u32 { core::ptr::eq(p, f()) as u32 }

#[test]
fn ftCo_HammerWait_matches_c() {
    init();
    unsafe {
        hk::ft_8008A1FC = Some(|g| { t(1); t(is_go(g)); sr(1) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(2); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftCommon_8007EBAC = Some(|fr, a, bb| { t(3); t(isf(fr)); t(a); t(bb) });
        hk::ftColl_8007AFF8 = Some(|g| { t(4); t(is_go(g)) });
        hk::ft_800892A0 = Some(|g| { t(5); t(is_go(g)) });
        hk::ft_80089824 = Some(|g| { t(6); t(is_go(g)) });
        hk::it_8027B070 = Some(|i, g| { t(7); t(itid(i)); t(is_go(g)) });
        hk::ftCo_800C5A50 = Some(|_| { t(8); sr(2) });
        hk::ftCo_800C60C8 = Some(|_| { t(9); sr(3) });
        hk::ftCo_800C576C = Some(|_| { t(10); sr(4) });
        hk::ftCo_800C55CC = Some(|_| { t(11); sr(5) });
        hk::ft_80084F3C = Some(|g| { t(12); t(is_go(g)) });
        hk::ftCo_800C5D34 = Some(|g| { t(13); t(is_go(g)) });
        hk::ft_8008403C = Some(|g, cb| {
            t(14); t(is_go(g)); t((fnsym::same(cb as usize, m::ext::ftCo_800C5D34 as usize)) as u32); cb(g)
        });
        hk::it_80293E34 = Some(|i| { t(15); t(itid(i)) });
        hk::ftCommon_8007ECD4 = Some(|fr, a| { t(16); t(isf(fr)); t(a as u32) });
        hk::Item_8026A8EC = Some(|i| { t(17); t(itid(i)) });
        hk::ft_80088110 = Some(|fr| { t(18); t(isf(fr)) });
        hk::ftCo_800C0694 = Some(|fr| { t(19); t(isf(fr)); if sr(6) { 106 } else { (rnd() % 200) as i32 } });
        hk::ftCommon_8007D92C = Some(|g| { t(20); t(is_go(g)) });
        hk::ftCo_800C0200 = Some(|fr, a| { t(21); t(isf(fr)); t(a as u32) });
        hk::itGetKind = Some(|i| { t(22); t(itid(i)); if sr(7) { It_Kind_Hammer as i32 } else { (rnd() % 40) as i32 } });
        hk::it_80293DCC = Some(|i| { t(23); t(itid(i)); (rnd() % 3) as i32 });
        hk::pl_800403C0 = Some(|a, bb| { t(24); t(a as u32); t(bb as u32) });
        hk::ft_800880AC = Some(|fr| { t(25); t(isf(fr)) });
        hk::ftCo_800BFFD0 = Some(|fr, a, bb| { t(26); t(isf(fr)); t(a as u32); t(bb as u32); sr(8) });
        hk::it_8026C16C = Some(|i, h| { t(27); t(itid(i)); t(h as u32) });
    }
    let ms = [306, 307, 310, 313, 314, ftCo_MS_LightGet as i32];
    let mut c = Check::new(include_str!("data/tu_ftCo_HammerWait.txt"));
    for op in 0..22u32 {
        for k in 0..1000u32 {
            case(op, k);
            let fr = f();
            fr.motion_id = ms[(rnd() % 6) as usize];
            fr.x2330.x = (rnd() % 3) as i32; fr.x2330.y = (rnd() % 3) as i32;
            fr.x2338.x = (rnd() % 2) as i32; fr.x2338.y = (rnd() % 3) as i32;
            fr.cur_anim_frame = rg(); fr.frame_speed_mul = rg();
            fr.item_gobj = if rnd() & 1 != 0 { igo() as *mut Item_GObj } else { core::ptr::null_mut() };
            fr.ground_or_air = (rnd() & 1) as _;
            fr.player_idx = (rnd() % 6) as _; fr.set_is_sub_fighter((rnd() & 1) as _);
            fr.set_x2219_b4((rnd() & 1) as u8);
            cd().x6AC = (rnd() % 1000) as i32 - 5; cd().x6B8 = (rnd() % 1000) as i32 - 5;
            let g = go();
            let ret = match op {
                0 => m::ftCo_800C4E4C(g) as u32,
                1 => { m::ftCo_800C4E94(f()); 0 }
                2 => { m::ftCo_800C4ED8(g); 0 }
                3 => { m::ftCo_800C4F64(g); 0 }
                4 => { m::ftCo_HammerWait_Anim(g); 0 }
                5 => { m::ftCo_HammerWait_IASA(g); 0 }
                6 => { m::ftCo_HammerWait_Phys(g); 0 }
                7 => { m::ftCo_HammerWait_Coll(g); 0 }
                8 => { m::ftCo_800C511C(g); 0 }
                9 => m::ftCo_800C5240(g) as u32,
                10 => { m::ftCo_800C5284(g); 0 }
                11 => { m::ftCo_800C52F4(g); 0 }
                12 => m::ftCo_800C53E4(f()) as u32,
                13 => { m::ftCo_800C5408(f()); 0 }
                14 => { m::ftCo_800C544C(f()); 0 }
                15 => b(m::ftCo_800C548C(f())),
                16 => m::ftCo_800C54C4(f()),
                17 => { m::ftCo_800C5500(g); 0 }
                18 => { m::ftCo_800C554C(f()); 0 }
                19 => { m::ftCo_800C555C(g); 0 }
                20 => { m::ftCo_800C5588(f()); 0 }
                _ => { m::ftCo_800C511C(g); m::ftCo_800C511C(g); m::ftCo_800C511C(g); 0 }
            };
            let fr = f();
            let mut h = Hs::new();
            h.m(fr.motion_id as u32); h.m(fr.x2330.x as u32); h.m(fr.x2330.y as u32);
            h.m(fr.x2338.x as u32); h.m(fr.x2338.y as u32); h.m(fr.x2219_b4() as u32);
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 22000);
}
