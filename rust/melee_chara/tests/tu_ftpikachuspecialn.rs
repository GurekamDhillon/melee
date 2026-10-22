//! Smoke test of tu::ftpikachuspecialn through recording hooks: Enter clears the command vars, the
//! jolt spawns once at the fused offset, the x21EC stick check, the Pikachu / Pichu SFX switch.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftpikachuspecialn as t;
use melee_fighter::hooks as fh;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> std::vec::Vec<String> {
    unsafe { core::mem::take(&mut LOG) }
}

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut ftPikachuAttributes = Box::leak(Box::new(ftPikachuAttributes::zeroed()));
    f.dat_attrs = da as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, _, _, _| log(format!("ms {ms} {fl:x}")));
        st::FTANIM_8006EBA4 = Some(|_| {});
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        st::FT_8008A2BC = Some(|_| log("Wait".into()));
        st::FTCO_FALL_ENTER = Some(|_| log("Fall".into()));
        st::FTCO_80096900 = Some(|_, a, b, c, m, l| log(format!("land {a} {b} {c} {m} {l}")));
        t::ext::ITPIKACHUTHUNDERJOLT_SPAWN = Some(|_, pos, fd, kind| {
            log(format!("jolt {} {} {} {fd} {kind}", (*pos).x, (*pos).y, (*pos).z))
        });
        fh::PLAY_SFX = Some(|_, id, v, p| log(format!("sfx {id} {v} {p}")));
        fh::FTLIB_GET_KIND = Some(|_| Ft_Kind_Pikachu);
    }

    f.cmd_vars = [5, 6, 7, 8];
    t::ftPk_SpecialN_Enter(gp);
    assert_eq!(take(), ["ms 341 0"]);
    assert_eq!(f.cmd_vars, [0; 4]);
    t::ftPk_SpecialAirN_Enter(gp);
    assert_eq!(take(), ["ms 342 0"]);

    // Anim: pulse 1 spawns the jolt once at fmadds(scale.y, off.x * facing, cur.x).
    da.specialn_spawn_offset = Vec2 { x: 3.0, y: 4.0 };
    da.specialn_itkind = 9 as _;
    f.x34_scale.y = 1.5;
    f.facing_dir = -1.0;
    f.cur_pos = Vec3 { x: 10.0, y: 20.0, z: 7.0 };
    f.cmd_vars = [1, 0, 0, 0];
    t::ftPk_SpecialN_Anim(gp);
    assert_eq!(take(), ["jolt 5.5 26 0 -1 9", "sfx 240076 127 64", "Wait"]);
    t::ftPk_SpecialN_Anim(gp);
    assert_eq!(take(), ["Wait"]);

    // Pichu plays its own SFX.
    unsafe { fh::FTLIB_GET_KIND = Some(|_| Ft_Kind_Pichu) };
    f.cmd_vars = [1, 0, 0, 0];
    t::ftPk_SpecialN_Anim(gp);
    assert_eq!(take()[1], "sfx 230067 127 64");

    // Air Anim: no landing lag falls, otherwise the landing state with the lag.
    da.specialairn_landing_lag = 0.0;
    f.cmd_vars = [0; 4];
    t::ftPk_SpecialAirN_Anim(gp);
    assert_eq!(take(), ["Fall"]);
    da.specialairn_landing_lag = 12.0;
    t::ftPk_SpecialAirN_Anim(gp);
    assert_eq!(take(), ["land 1 0 1 1 12"]);

    // x21EC: a hard stick sets the thrown-item count and the attribute; a soft one clears it.
    da.x1C = 50.0;
    da.x20 = 7.0;
    f.active_sticky.lstick.x = 100;
    t::ftPk_SpecialN_80124DC8(gp);
    assert_eq!(unsafe { f.mv.pk.unk2.x0 }, 0);
    f.active_sticky.lstick.x = 10;
    t::ftPk_SpecialN_80124DC8(gp);
    assert_eq!(unsafe { f.mv.pk.unk2.x0 }, 7);
}
