//! src/tu/ftCo_DamageSong.rs vs the C (gen/cref/tu/ftCo_DamageSong.c, shipping contraction;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_DamageSong`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_DamageSong as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

static mut HC: i32 = 0;
static mut BB: i32 = 0;
fn isf(fr: &Fighter) -> u32 { (fr as *const Fighter == unsafe { F } as *const Fighter) as u32 }

#[test]
fn ftCo_DamageSong_matches_c() {
    init();
    unsafe {
        hk::ftCommon_8007DB58 = Some(|g| { t(1); t(is_go(g)) });
        hk::ftCo_8009750C = Some(|g| { t(2); t(is_go(g)) });
        hk::ftCo_800DD168 = Some(|g| { t(3); t(is_go(g)) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(4); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::Player_GetHandicap = Some(|s| { t(5); t(s as u32); HC });
        hk::Player_80033BB8 = Some(|s| { t(6); t(s as u32); BB });
        hk::ftCommon_InitGrab = Some(|fr, a, tm| { t(7); t(isf(fr)); t(a as u32); t(b(tm)) });
        hk::ftCommon_GrabMash = Some(|fr, v| { t(8); t(isf(fr)); t(b(v)); sr(9) });
        hk::ftAnim_IsFramesRemaining = Some(|_| { t(9); sr(3) });
        hk::ft_8008A2BC = Some(|g| { t(10); t(is_go(g)) });
        hk::ft_80084F3C = Some(|g| { t(11); t(is_go(g)) });
        hk::ft_80084280 = Some(|g| { t(12); t(is_go(g)) });
        hk::ftCo_800C3598 = Some(|g| { t(13); t(is_go(g)) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_DamageSong.txt"));
    for op in 0..16u32 {
        for k in 0..3000u32 {
            case(op, k);
            unsafe {
                HC = (rnd() % 9) as i32 - 1;
                BB = (rnd() % 6) as i32 - 1;
            }
            let d = cd();
            d.x624 = rw(); d.x628 = rw(); d.x62C = rw(); d.x630 = rw(); d.x634 = rw();
            d.x638 = rw(); d.x63C = rw(); d.x640 = rg(); d.x644 = rw();
            let fr = f();
            fr.dmg.x1830_percent = rw();
            fr.player_id = (rnd() % 6) as _;
            fr.grab_timer = rw();
            fr.set_x2222_b2((rnd() & 1) as u8);
            unsafe { fr.x2070.anon_0.set_x2071_b0_3((rnd() & 15) as u8) };
            let g = go();
            let ret = match op {
                0 => { m::ftCo_800C318C(g, rnd() & 1 != 0); 0 }
                1 => { m::ftCo_DamageSong_Anim(g); 0 }
                2 => { m::ftCo_DamageSong_IASA(g); 0 }
                3 => { m::ftCo_DamageSong_Phys(g); 0 }
                4 => { m::ftCo_DamageSong_Coll(g); 0 }
                5 => { m::ftCo_800C3390(g); 0 }
                6 => { m::ftCo_DamageSongWait_Anim(g); 0 }
                7 => { m::ftCo_DamageSongWait_IASA(g); 0 }
                8 => { m::ftCo_DamageSongWait_Phys(g); 0 }
                9 => { m::ftCo_DamageSongWait_Coll(g); 0 }
                10 => { m::ftCo_800C3480(g); 0 }
                11 => { m::ftCo_DamageSongRv_Anim(g); 0 }
                12 => { m::ftCo_DamageSongRv_IASA(g); 0 }
                13 => { m::ftCo_DamageSongRv_Phys(g); 0 }
                14 => { m::ftCo_DamageSongRv_Coll(g); 0 }
                _ => m::ftCo_800C3538(g) as u32,
            };
            let mut h = Hs::new();
            h.m(b(f().grab_timer));
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 48000);
}
