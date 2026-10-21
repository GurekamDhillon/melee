//! src/tu/ftCo_Turn.rs vs the C (gen/cref/tu/ftCo_Turn.c: the whole TU, TARGET_PC, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_Turn`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_Turn as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

macro_rules! ck { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(is_go(g)); sr($id) && sr($id + 16) && sr($id + 32) }); )* } } }
macro_rules! vs { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(is_go(g)) }); )* } } }

static mut UCF: i32 = 0;

#[test]
fn ftCo_Turn_matches_c() {
    init();
    ck!(ftCo_SpecialS_CheckInput = 1, ftCo_800D68C0 = 2, ftCo_Attack100_CheckInput = 3, ftCo_Catch_CheckInput = 4,
        ftCo_AttackS4_CheckInput = 5, ftCo_AttackHi4_CheckInput = 6, ftCo_AttackLw4_CheckInput = 7,
        ftCo_AttackS3_CheckInput = 8, ftCo_AttackHi3_CheckInput = 9, ftCo_AttackLw3_CheckInput = 10,
        ftCo_Attack1_CheckInput = 11, ftCo_80091A4C = 12, ftCo_800DE9D8 = 13, ftCo_Jump_CheckInput = 14);
    vs!(ftAnim_8006EBA4 = 21, ft_8008A2BC = 23, ft_80084F3C = 25, ft_80083F88 = 26);
    unsafe {
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(20); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftAnim_IsFramesRemaining = Some(|_| { t(22); sr(22) });
        hk::ftCo_Dash_Enter = Some(|g, a| { t(24); t(is_go(g)); t(a as u32) });
        hk::ScriptGame_CapsActionAllowed = Some(|fr, fl| { t(27); t((fr as *const Fighter == f() as *const Fighter) as u32); t(fl as u32); sr(27) as i32 });
        hk::Replay_UcfVersion = Some(|p| { t(28); t(p as u32); UCF });
        hk::Replay_RawStickBack = Some(|p, w, bk| { t(29); t(p as u32); t(w as u32); t(bk as u32); (rnd() % 256) as i32 - 128 });
        hk::ftUcf_PadRaw = Some(|p, w, bk| { t(30); t(p as u32); t(w as u32); t(bk as u32); (rnd() % 256) as i32 - 128 });
    }
    const UCFS: [i32; 4] = [0, 73, 84, 84];
    let mut c = Check::new(include_str!("data/tu_ftCo_Turn.txt"));
    for op in 0..10u32 {
        for k in 0..3000u32 {
            case(op, k);
            unsafe { UCF = UCFS[(rnd() % 4) as usize] };
            let d = cd();
            d.x34 = rg();
            d.dash_smash_stick_threshold = rg();
            d.dash_smash_window = (rnd() % 8) as i32;
            let fr = f();
            fr.input.lstick[0].x = rg();
            fr.input.pressed_buttons = rnd() & 0x3FF;
            fr.facing_dir = rg();
            fr.active_timer.lstick.x = (rnd() % 4) as u8;
            fr.player_id = (rnd() % 6) as u8;
            fr.x618_player_id = (rnd() % 6) as u8;
            fr.set_is_sub_fighter((rnd() & 1) as _);
            fr.x3E4_fighterCmdScript.frame_count = if rnd() & 1 != 0 { 2.0 } else { rg() };
            fr.co_attrs.standing_turn_frames = rw();
            unsafe {
                let tn = &mut fr.mv.co.turn;
                tn.has_turned = (rnd() & 1) as i32;
                tn.just_turned = (rnd() & 1) as i32;
                tn.facing_after = rg();
                tn.frames_to_turn = if rnd() & 1 != 0 { rg() } else { 0.0 };
                tn.x8 = if rnd() & 1 != 0 { rg() } else { 0.0 };
                tn.x1C = rnd() & 0x3FF;
            }
            let g = go();
            let ret = match op {
                0 => m::ftCo_800C97A8(g) as u32,
                1 => m::ftCo_Turn_CheckInput(g) as u32,
                2 => {
                    let (a, bb, cc) = (rw(), rw(), rw());
                    let ms = rnd() % 400;
                    let fl = rnd();
                    m::ftCo_Turn_Enter(g, ms as i32, fl, a, bb, cc);
                    0
                }
                3 => { m::ftCo_Turn_Enter_Basic(g); 0 }
                4 => { m::ftCo_Turn_Anim(g); 0 }
                5 => { m::ftCo_Turn_IASA(g); 0 }
                6 => { m::ftCo_Turn_Phys(g); m::ftCo_Turn_Coll(g); 0 }
                7 => m::fn_800C9C2C(g) as u32,
                8 => { m::ftCo_Turn_Enter_Smash(g); 0 }
                _ => { m::ftCo_Turn_Anim_Inner(g); 0 }
            };
            let fr = f();
            let mut h = Hs::new();
            h.m(b(fr.facing_dir)); h.m(fr.input.pressed_buttons);
            unsafe {
                let tn = &fr.mv.co.turn;
                h.m(tn.has_turned as u32); h.m(tn.just_turned as u32); h.m(b(tn.facing_after));
                h.m(b(tn.frames_to_turn)); h.m(b(tn.x8)); h.m(tn.x1C);
            }
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 30000);
}
