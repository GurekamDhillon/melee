//! src/tu/ftCo_HammerFall.rs vs the C (gen/cref/tu/ftCo_HammerFall.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_HammerFall`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_HammerFall as m;
use m::ext::hook as hk;

static mut FR: f32 = 0.0;
static mut FL: u32 = 0;
fn isf(p: &mut melee_fighter::types::Fighter) -> u32 { (p as *mut _ == unsafe { F }) as u32 }

#[test]
fn ftCo_HammerFall_matches_c() {
    init();
    unsafe {
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| { t(1); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32) });
        hk::ftCo_800C54C4 = Some(|p| { t(2); t(isf(p)); FL });
        hk::ftCo_800C548C = Some(|p| { t(3); t(isf(p)); FR });
        hk::ftCo_800C4E94 = Some(|p| { t(4); t(isf(p)) });
        hk::ftCo_800C4F64 = Some(|g| { t(5); t(is_go(g)) });
        hk::ftCo_800C5240 = Some(|g| { t(6); t(is_go(g)); sr(1) });
        hk::ftCo_800C544C = Some(|p| { t(7); t(isf(p)) });
        hk::ftCommon_8007D5D4 = Some(|p| { t(8); t(isf(p)) });
        hk::Item_8026ABD8 = Some(|g, p, v| { t(9); t((g == igo()) as u32); t(b(p.x)); t(b(p.y)); t(b(p.z)); t(b(v)) });
        hk::ftCo_Fall_Enter = Some(|g| { t(10); t(is_go(g)) });
        hk::pl_8003FDF4 = Some(|a, bb| { t(11); t(a as u32); t(bb as u32) });
        hk::ft_80084DB0 = Some(|g| { t(12); t(is_go(g)) });
        hk::ft_80082C74 = Some(|g, cb| { t(13); t(is_go(g)); t((cb as usize == m::ext::ftCo_HammerLanding_Enter as usize) as u32) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_HammerFall.txt"));
    for op in 0..7u32 {
        for k in 0..2000u32 {
            case(op, k);
            unsafe { FR = rw(); FL = rnd(); }
            let fr = f();
            fr.input.pressed_buttons = rnd(); fr.input.held_buttons[0] = rnd();
            fr.x2338.y = (rnd() & 1) as i32; fr.x683 = (rnd() & 0xFF) as u8; cd().x1C = (rnd() % 256) as i32;
            fr.ground_or_air = (rnd() & 1) as _; fr.item_gobj = igo() as _; fr.player_id = (rnd() % 6) as u8;
            fr.set_is_sub_fighter((rnd() & 1) as u8);
            let g = go();
            let ret = match op {
                0 => m::ftCo_800C5CD4(g) as u32,
                1 => { m::ftCo_800C5D34(g); 0 }
                2 => { m::ftCo_HammerFall_Anim(g); 0 }
                3 => m::ftCo_800C5DDC(g) as u32,
                4 => { m::ftCo_HammerFall_IASA(g); 0 }
                5 => { m::ftCo_HammerFall_Phys(g); 0 }
                _ => { m::ftCo_HammerFall_Coll(g); 0 }
            };
            c.line(op, k, ret, Hs::new().0);
        }
    }
    assert_eq!(c.done(), 14000);
}
