//! Smoke test of tu::ftfoxappeals (no float math, so no C reference): the Corneria taunt's
//! input check, entry and three-stage motion flow, through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftfoxappeals as t;
use melee_fighter::hooks as fh;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> String {
    unsafe { std::mem::take(&mut LOG).join(",") }
}

static mut STATS: [plActionStats; 6] = unsafe { core::mem::zeroed() };
static mut USED: [bool; 6] = [false; 6];
static mut READY: bool = false;
static mut RAN_2CE8: bool = true;

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        t::ext::PLAYER_GETACTIONSTATS = Some(|slot| &raw mut STATS[slot as usize]);
        t::ext::PL_800386D8 = Some(|stats, attack| {
            let i = (stats as usize - (&raw const STATS) as usize) / core::mem::size_of::<plActionStats>();
            assert_eq!(attack, 0x72);
            USED[i] as u32
        });
        t::ext::PL_80040120 = Some(|slot, sub| log(format!("pl40120 {slot} {sub}")));
        t::ext::GRCORNERIA_801E2D14 = Some(|| READY);
        t::ext::GRCORNERIA_801E2CE8 = Some(|| RAN_2CE8);
        t::ext::GRCORNERIA_801E2B80 = Some(|| {
            log("fox_tx".into());
            true
        });
        t::ext::GRCORNERIA_801E2C34 = Some(|| {
            log("falco_tx".into());
            true
        });
        t::ext::GRCORNERIA_801E2AF4 = Some(|| log("tx_off".into()));
        t::ext::FT_8008A324 = Some(|_| log("wait".into()));
        t::ext::FT_80084280 = Some(|_| log("coll".into()));
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, _, _, _, _, _| log(format!("M{ms}")));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        fh::GENO_DEFINE_TEMPLATE_SKELETON = Some(|_| 0);
    }

    // CheckIfUsed: this player used it AND the stage transmission is ready.
    f.player_idx = 2;
    assert!(!t::ftFx_AppealS_CheckIfUsed(&mut *f));
    unsafe { USED[2] = true };
    assert!(!t::ftFx_AppealS_CheckIfUsed(&mut *f));
    unsafe { READY = true };
    assert!(t::ftFx_AppealS_CheckIfUsed(&mut *f));
    assert!(t::ftFox_CheckAppealSCount());
    unsafe { USED[2] = false };
    assert!(!t::ftFox_CheckAppealSCount());

    // Enter: facing right (1.0 == facing) takes the R column, left the L column.
    f.facing_dir = 1.0;
    f.x2210.throw_flags = 0xFF;
    t::ftFx_AppealS_Enter(gp);
    assert_eq!(take(), format!("M{}", ftFx_MS_AppealSStartR));
    assert_eq!(unsafe { (f.mv.fx.AppealS.facingDir, f.mv.fx.AppealS.animCount) }, (0, 0));
    assert_eq!(unsafe { f.x2210.throw_flags }, 0);
    f.facing_dir = -1.0;
    t::ftFx_AppealS_Enter(gp);
    assert_eq!(take(), format!("M{}", ftFx_MS_AppealSStartL));
    assert_eq!(unsafe { f.mv.fx.AppealS.facingDir }, 1);

    // CheckInput: Fox or Falco, on Corneria, D-pad down not held, x682 == 1, nobody used it.
    f.kind = Ft_Kind_Fox;
    f.x682 = 1;
    f.player_idx = 1;
    f.facing_dir = 1.0;
    assert!(t::ftFx_AppealS_CheckInput(gp));
    assert_eq!(take(), format!("M{},pl40120 1 0", ftFx_MS_AppealSStartR));
    f.kind = Ft_Kind_Falco;
    assert!(t::ftFx_AppealS_CheckInput(gp));
    take();
    f.kind = Ft_Kind_Mario;
    assert!(!t::ftFx_AppealS_CheckInput(gp));
    f.kind = Ft_Kind_Fox;
    f.input.held_buttons[0] = HSD_PAD_DPADDOWN as _;
    assert!(!t::ftFx_AppealS_CheckInput(gp));
    f.input.held_buttons[0] = 0;
    f.x682 = 0;
    assert!(!t::ftFx_AppealS_CheckInput(gp));
    f.x682 = 1;
    unsafe { RAN_2CE8 = false };
    assert!(!t::ftFx_AppealS_CheckInput(gp));
    unsafe { RAN_2CE8 = true };
    unsafe { USED[4] = true };
    assert!(!t::ftFx_AppealS_CheckInput(gp));
    unsafe { USED[4] = false };
    assert_eq!(take(), "");

    // Anim: the throw flag arms the transmission callback once, per kind; then the motion
    // advances Start -> Taunt -> End -> Wait when each animation runs out.
    f.kind = Ft_Kind_Fox;
    f.mv.fx.AppealS = ftFoxAppealS { facingDir: 0, animCount: 0 };
    unsafe { f.x2210.x0.set_throw_flags_b3(1) };
    t::ftFx_AppealS_Anim(gp);
    assert_eq!(unsafe { f.x2210.x0.throw_flags_b3() }, 0);
    assert!(f.death1_cb.is_some());
    assert_eq!(take(), format!("fox_tx,M{}", ftFx_MS_AppealSR));
    t::ftFx_AppealS_Anim(gp);
    assert_eq!(take(), format!("M{}", ftFx_MS_AppealSEndR));
    t::ftFx_AppealS_Anim(gp);
    assert_eq!(take(), "wait");
    f.kind = Ft_Kind_Falco;
    f.death1_cb = None;
    f.mv.fx.AppealS = ftFoxAppealS { facingDir: 1, animCount: 0 };
    unsafe { f.x2210.x0.set_throw_flags_b3(1) };
    t::ftFx_AppealS_Anim(gp);
    assert_eq!(take(), format!("falco_tx,M{}", ftFx_MS_AppealSL));
    assert!(f.death1_cb.is_some());

    // The callback it armed turns the transmission off and disarms itself.
    unsafe { f.death1_cb.unwrap()(gp) };
    assert_eq!(take(), "tx_off");
    assert!(f.death1_cb.is_none());

    t::ftFx_AppealS_IASA(gp);
    t::ftFx_AppealS_Coll(gp);
    assert_eq!(take(), "coll");
}
