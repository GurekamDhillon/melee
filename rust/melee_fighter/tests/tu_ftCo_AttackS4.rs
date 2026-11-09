//! src/tu/ftCo_AttackS4.rs vs the C (gen/cref/tu/ftCo_AttackS4.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_AttackS4`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_AttackS4 as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

static mut ANG: [f32; 2] = [0.0; 2];
static mut ND: usize = 0;
static mut DR: [Fighter_WaitAnimData; 4] = unsafe { core::mem::zeroed() };
unsafe extern "C" fn pause_all(_: *mut HSD_GObj) {}
unsafe extern "C" fn resume_all(_: *mut HSD_GObj) {}

macro_rules! bstub { ($($n:ident = $id:literal),*) => { $( hk::$n = Some(|g| { t($id); t(is_go(g)); sr($id) }); )* }; }
macro_rules! vstub { ($($n:ident = $id:literal),*) => { $( hk::$n = Some(|g| { t($id); t(is_go(g)) }); )* }; }

#[test]
fn ftCo_AttackS4_matches_c() {
    init();
    unsafe {
        hk::ftCo_GetLStickAngle = Some(|_| { t(1); ANG[0] });
        hk::ftCo_GetCStickAngle = Some(|_| { t(2); ANG[1] });
        hk::ftCo_800DF1C8 = Some(|_| { t(3); sr(3) });
        hk::ftCo_800DF21C = Some(|_| { t(4); sr(4) });
        hk::it_8026B30C = Some(|i| { t(5); t((i == igo() as *mut Item_GObj) as u32); (SRV[5] % 5) as i32 });
        hk::it_8026B594 = Some(|i| { t(6); t((i == igo() as *mut Item_GObj) as u32); sr(6) });
        hk::ftCo_800957F4 = Some(|g, ms| { t(7); t(is_go(g)); t(ms as u32) });
        hk::ftCo_Attack_800CCF58 = Some(|g, a| { t(8); t(is_go(g)); t(a as u32) });
        hk::Mex_HasFSmashHook = Some(|k| { t(10); t(k as u32); sr(10) });
        hk::Mex_OnFSmashDispatch = Some(|k, g, v| { t(11); t(k as u32); t(is_go(g)); t(v.is_null() as u32) });
        hk::efLib_PauseAll_fn = Some(|| Some(pause_all));
        hk::efLib_ResumeAll_fn = Some(|| Some(resume_all));
        hk::ftData_80085FD4 = Some(|fr, ms| { t(15); t((fr as *mut Fighter == F) as u32); t(ms as u32); let p = &mut DR[ND & 3] as *mut _; ND += 1; p });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(16); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        vstub!(ftCo_Attack_800CDD14 = 9, ftNs_AttackS4_Enter = 12, ftPe_AttackS4_Enter = 13, ftGw_AttackS4_Enter = 14,
               ftAnim_8006EBA4 = 17, ft_8008A2BC = 19, ft_80084FA8 = 20, ft_80084104 = 21);
        bstub!(ftAnim_IsFramesRemaining = 18, ftCo_SpecialS_CheckInput = 22, ftCo_Attack100_CheckInput = 23,
               ftCo_800D6824 = 24, ftCo_800D68C0 = 25, ftCo_Catch_CheckInput = 26, ftCo_800CECE8 = 27,
               ftCo_AttackHi4_CheckInput = 28, ftCo_AttackLw4_CheckInput = 29, ftCo_AttackS3_CheckInput = 30,
               ftCo_AttackHi3_CheckInput = 31, ftCo_AttackLw3_CheckInput = 32, ftCo_Attack1_CheckInput = 33,
               ftCo_80091A4C = 34, ftCo_800DE9D8 = 35, ftCo_Jump_CheckInput = 36, ftCo_Dash_CheckInput = 37,
               ftCo_800D5FB0 = 38, ftCo_Turn_CheckInput = 39, ftCo_Walk_CheckInput = 40);
    }
    const KINDS: [FighterKind; 5] = [Ft_Kind_Ness, Ft_Kind_Peach, Ft_Kind_GameWatch, Ft_Kind_Pikachu, Ft_Kind_Pichu];
    let mut c = Check::new(include_str!("data/tu_ftCo_AttackS4.txt"));
    for op in 0..12u32 {
        for k in 0..2000u32 {
            case(op, k);
            unsafe {
                ND = 0;
                for i in 0..4 { DR[i].x8 = sr(48 + i) as i32; }
                ANG[0] = rg() * 2.0;
                ANG[1] = rg() * 2.0;
            }
            let d = cd();
            d.dash_smash_stick_threshold = rg();
            d.dash_smash_window = (rnd() % 6) as i32 - 1;
            d.xB8_radians = rg(); d.xBC_radians = rg(); d.xC0_radians = rg(); d.xC4_radians = rg();
            let fr = f();
            fr.input.pressed_buttons = rnd() & 0x1FF;
            fr.input.held_buttons[0] = rnd() & 0x8000_0000;
            fr.input.lstick[0].x = rg();
            fr.input.cstick[0].x = rg();
            fr.active_timer.lstick.x = (rnd() % 6) as _;
            fr.facing_dir = rg();
            fr.item_gobj = if rnd() & 1 != 0 { igo() as *mut Item_GObj } else { core::ptr::null_mut() };
            fr.kind = if rnd() & 1 != 0 { KINDS[(rnd() % 5) as usize] } else { (rnd() % 33) as _ };
            fr.set_allow_interrupt((rnd() & 1) as u8);
            fr.cmd_vars[0] = rnd();
            fr.x2210.throw_flags = rnd();
            let a0 = rg();
            let a1 = rg() * 2.0;
            let g = go();
            let ret = match op {
                0 => m::checkLStick(f()) as u32,
                1 => m::ftCo_AttackS4_CheckInput(g) as u32,
                2 => m::checkFacingDir(f()) as u32,
                3 => m::ftCo_AttackS4_8008C114(g) as u32,
                4 => m::checkItemThrow(g, a0) as u32,
                5 => { m::decideFighter(g, a0, a1); 0 }
                6 => { m::doEnter(g, a1); 0 }
                7 => { m::ftCo_AttackS4_Anim(g); 0 }
                8 => { m::ftCo_AttackS4_IASA(g); 0 }
                9 => { m::ftCo_AttackS4_Phys(g); 0 }
                10 => { m::ftCo_AttackS4_Coll(g); 0 }
                _ => { m::Fighter_SetEffectHitlagCallbacks(f()); 0 }
            };
            let fr = f();
            let mut h = Hs::new();
            h.m(fr.allow_interrupt() as u32);
            h.m(fr.facing_dir.to_bits());
            h.m(fr.cmd_vars[0]);
            h.m(unsafe { fr.x2210.throw_flags });
            h.m((fr.pre_hitlag_cb.map(|p| p as usize) == Some(pause_all as usize)) as u32);
            h.m((fr.post_hitlag_cb.map(|p| p as usize) == Some(resume_all as usize)) as u32);
            h.m(fr.pre_hitlag_cb.is_none() as u32);
            h.m(fr.post_hitlag_cb.is_none() as u32);
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 24000);
}
