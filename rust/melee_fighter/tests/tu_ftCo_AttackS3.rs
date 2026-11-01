//! src/tu/ftCo_AttackS3.rs vs the C (gen/cref/tu/ftCo_AttackS3.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_AttackS3`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_AttackS3 as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

static mut ANG: [f32; 2] = [0.0; 2];
static mut NANG: usize = 0;
static mut ND: usize = 0;
static mut DR: [Fighter_WaitAnimData; 4] = unsafe { core::mem::zeroed() };

#[test]
fn ftCo_AttackS3_matches_c() {
    init();
    unsafe {
        hk::ftCo_GetLStickAngle = Some(|_| { t(1); let a = ANG[NANG & 1]; NANG += 1; a });
        hk::it_8026B30C = Some(|i| { t(2); t((i == igo() as *mut Item_GObj) as u32); (SRV[0] % 5) as i32 });
        hk::ftCo_800957F4 = Some(|g, ms| { t(4); t(is_go(g)); t(ms as u32) });
        hk::ftCo_Attack_800CCF58 = Some(|g, a| { t(5); t(is_go(g)); t(a as u32) });
        hk::ftCo_Attack_800CDD14 = Some(|g| { t(6); t(is_go(g)) });
        hk::ftpickupitem_80094790 = Some(|g| { t(14); t(is_go(g)); sr(1) });
        hk::ftData_80085FD4 = Some(|fr, ms| { t(15); t((fr as *mut Fighter == F) as u32); t(ms as u32); let p = &mut DR[ND & 3] as *mut _; ND += 1; p });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(7); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftAnim_8006EBA4 = Some(|g| { t(8); t(is_go(g)) });
        hk::ftAnim_IsFramesRemaining = Some(|_| { t(9); sr(3) });
        hk::ft_8008A2BC = Some(|g| { t(10); t(is_go(g)) });
        hk::ftCo_Wait_IASA = Some(|g| { t(11); t(is_go(g)) });
        hk::ft_80084F3C = Some(|g| { t(12); t(is_go(g)) });
        hk::ft_80084104 = Some(|g| { t(13); t(is_go(g)) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_AttackS3.txt"));
    for op in 0..6u32 {
        for k in 0..2000u32 {
            case(op, k);
            unsafe {
                NANG = 0;
                ND = 0;
                for i in 0..4 { DR[i].x8 = (SRV[4 + i] & 1) as i32; }
                ANG[0] = rg() * 2.0;
                ANG[1] = rg() * 2.0;
            }
            let d = cd();
            d.x98 = rg(); d.x20_radians = rg(); d.x9C_radians = rg(); d.xA0_radians = rg();
            d.xA4_radians = rg(); d.xA8_radians = rg();
            let fr = f();
            fr.input.pressed_buttons = rnd() & 0x1FF;
            fr.input.held_buttons[0] = rnd() & 0x8000_0000;
            fr.input.lstick[0].x = rg();
            fr.facing_dir = rg();
            fr.item_gobj = if rnd() & 1 != 0 { igo() as *mut Item_GObj } else { core::ptr::null_mut() };
            fr.set_allow_interrupt((rnd() & 1) as u8);
            let g = go();
            let ret = match op {
                0 => m::ftCo_AttackS3_CheckInput(g) as u32,
                1 => { m::decideAngle(g); 0 }
                2 => { m::ftCo_AttackS3_Anim(g); 0 }
                3 => { m::ftCo_AttackS3_IASA(g); 0 }
                4 => { m::ftCo_AttackS3_Phys(g); 0 }
                _ => { m::ftCo_AttackS3_Coll(g); 0 }
            };
            let mut h = Hs::new();
            h.m(f().allow_interrupt() as u32);
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 12000);
}
