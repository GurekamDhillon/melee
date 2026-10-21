//! src/tu/ftCo_ItemScopeStart.rs vs the C (gen/cref/tu/ftCo_ItemScopeStart.c: the whole TU,
//! shipping contraction; regenerate with
//! `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_ItemScopeStart`). The C's GC-offset
//! overlay words (`fp + 0x2340/0x2344`) are `mv` words 0/1 here.
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_ItemScopeStart as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

fn ovf() -> &'static mut f32 { unsafe { &mut *(&mut f().mv as *mut _ as *mut f32) } }
fn ovi() -> &'static mut i32 { unsafe { &mut *((&mut f().mv as *mut _ as *mut i32).add(1)) } }

#[test]
fn ftCo_ItemScopeStart_matches_c() {
    init();
    unsafe {
        hk::it_8026B594 = Some(|g| { t(1); t((g == igo()) as u32); sr(0) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| { t(2); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32) });
        hk::ftAnim_8006EBA4 = Some(|_| t(3));
        hk::ftCommon_ClampAirDrift = Some(|_| t(4));
        hk::ftCommon_8007E79C = Some(|g, a| { t(5); t(is_go(g)); t(a as u32) });
        hk::ftCommon_8007E7E4 = Some(|g, a| { t(6); t(is_go(g)); t(a as u32) });
        hk::ftCommon_8007D7FC = Some(|_| t(7));
        hk::ftCommon_8007D5D4 = Some(|_| t(8));
        hk::it_80291DAC = Some(|_, n| { t(9); t(n as u32); (n & 3) - 1 });
        hk::it_80291F14 = Some(|_, n| { t(10); t(n as u32) });
        hk::it_80291CF4 = Some(|_, n| { t(11); t(n as u32); sr(1) as i32 });
        hk::ftAnim_IsFramesRemaining = Some(|_| { t(12); sr(2) });
        hk::fn_800D7BDC = Some(|_| t(13));
        hk::fn_800D7C60 = Some(|_| t(14));
        hk::fn_800D8140 = Some(|_, n| { t(15); t(n as u32) });
        hk::fn_800D81D0 = Some(|_, n| { t(16); t(n as u32) });
        hk::ftCo_800CDE54 = Some(|_| t(17));
        hk::ftCo_800CDE74 = Some(|_| t(18));
        hk::ft_800841B8 = Some(|g, cb| { t(19); t((cb as usize == m::fn_800D78B0 as usize) as u32); cb(g) });
        hk::ft_80082C74 = Some(|g, cb| { t(20); t((cb as usize == m::fn_800D7830 as usize) as u32); cb(g) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_ItemScopeStart.txt"));
    for op in 0..14u32 {
        for k in 0..2000u32 {
            case(op, k);
            let fr = f();
            fr.gobj = go();
            fr.item_gobj = if rnd() % 8 != 0 { igo() } else { core::ptr::null_mut() };
            fr.motion_id = (158 + rnd() % 16) as i32;
            fr.input.held_buttons[0] = rnd() & 0x100;
            fr.cur_anim_frame = rw();
            fr.frame_speed_mul = rw();
            *ovf() = rg() * 20.0;
            *ovi() = (rnd() & 1) as i32;
            cd().x5B8 = rg() * 20.0;
            let g = go();
            let ret = match op {
                0 => { let ms = (rnd() % 400) as i32; m::fn_800D769C(f(), ms) as u32 }
                1 => { m::ft_800D76B8(g); 0 }
                2 => { m::ft_800D7770(g); 0 }
                3 => { m::fn_800D7830(g); 0 }
                4 => { m::fn_800D78B0(g); 0 }
                5 => { *ovi() = ovf().to_bits() as i32; m::fn_800D7938(g); 0 }
                6 => { m::ftCo_ItemScopeStart_Anim(g); 0 }
                7 => { m::ftCo_ItemScopeAirStart_Anim(g); 0 }
                8 => { m::ftCo_ItemScopeStart_IASA(g); 0 }
                9 => { m::ftCo_ItemScopeAirStart_IASA(g); 0 }
                10 => { m::ftCo_ItemScopeStart_Phys(g); 0 }
                11 => { m::ftCo_ItemScopeAirStart_Phys(g); 0 }
                12 => { m::ftCo_ItemScopeStart_Coll(g); 0 }
                _ => { m::ftCo_ItemScopeAirStart_Coll(g); 0 }
            };
            let mut h = Hs::new();
            h.m(match f().take_dmg_cb { None => 0, Some(p) if p as usize == m::fn_800D7938_c as usize => 1, _ => 9 });
            h.m(b(*ovf())); h.m(*ovi() as u32);
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 28000);
}
