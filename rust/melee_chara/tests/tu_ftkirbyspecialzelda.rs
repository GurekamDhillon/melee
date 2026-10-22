//! Smoke test for tu::ftkirbyspecialzelda (stateful; no fused sites) through recording hooks.
//! One #[test]: the hooks are process globals.
#![allow(static_mut_refs, unused_unsafe)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkirbyspecialzelda as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> String {
    unsafe { core::mem::take(&mut LOG).join(",") }
}

#[test]
fn smoke() {
    unsafe {
        let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
        let mut g: Box<HSD_GObj> = Box::new(core::mem::zeroed());
        let da: &'static mut ftKb_DatAttrs = Box::leak(Box::new(ftKb_DatAttrs::zeroed()));
        let mut parts: Box<[FighterBone; 2]> = Box::new(core::mem::zeroed());
        let cd: &'static ftCommonData = Box::leak(Box::new(ftCommonData::zeroed()));
        melee_fighter::hooks::P_FT_COMMON_DATA = cd;
        f.dat_attrs = da as *mut _ as *mut _;
        f.parts = parts.as_mut_ptr();
        g.user_data = &mut *f as *mut Fighter as *mut _;
        let gp: *mut HSD_GObj = &mut *g;
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, _, _, _| log(format!("ms {ms} {fl:x}")));
        st::FTANIM_8006EBA4 = Some(|_| log("6EBA4".into()));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        st::FT_8008A2BC = Some(|_| log("Wait".into()));
        st::FTCO_FALL_ENTER = Some(|_| log("Fall".into()));
        st::FT_80084F3C = Some(|_| log("groundphys".into()));
        st::FTCOLL_8007AEF8 = Some(|_| log("AEF8".into()));
        st::FT_80082708 = Some(|_| 0);
        st::FT_80081D0C = Some(|_| 0);
        melee_fighter::hooks::MPLIB_8004D164_LINE_COUNT = Some(|| None);
        st::FTCOMMON_8007D7FC = Some(|_| log("toground".into()));
        t::ext::EFSYNC_SPAWN = Some(|id, _, args| {
            log(format!("fx {id:x} {}", args.len()));
            core::ptr::null_mut()
        });
        t::ext::FTCOLL_CREATEREFLECTHIT = Some(|_, _, cb| log(format!("reflect {}", cb.is_some())));

        // Enter: motion change, gravity delay loaded, reflect var cleared, effect callback set.
        da.specialn_zd_frames_before_gravity = 7;
        da.specialn_zd_horizontal_momentum_preservation = 4.0;
        f.cmd_vars[0] = 9;
        t::ftKb_SpecialNZd_80105B2C(gp);
        assert_eq!(take(), format!("ms {} 0,6EBA4", ftKb_MS_ZdSpecialN));
        assert_eq!((f.cmd_vars[0], f.mv.zd.specialn.x0), (0, 7));
        assert!(f.accessory4_cb.is_some());
        f.self_vel.x = 8.0;
        f.self_vel.y = 3.0;
        t::ftKb_SpecialNZd_80105BA8(gp);
        assert_eq!(take(), format!("ms {} 0,6EBA4", ftKb_MS_ZdSpecialAirN));
        assert_eq!((f.self_vel.x, f.self_vel.y), (2.0, 0.0));

        // The effect callbacks spawn once on the root joint and clear themselves.
        f.set_x2219_b0(0);
        t::fn_80105A34(gp);
        assert_eq!(take(), "fx 4b6 1");
        assert!(f.accessory4_cb.is_none() && f.pre_hitlag_cb.is_some());
        f.set_x2219_b0(0);
        t::fn_80105AB0(gp);
        assert_eq!(take(), "fx 4b7 1");
        t::fn_80105AB0(gp);
        assert_eq!(take(), "");

        // Anim: the reflect bubble is created when the script sets cmd_vars[0] == 1 and the
        // reflecting flag clears once the script drops it; the end goes to Wait / Fall.
        f.cmd_vars[0] = 1;
        t::ftKb_ZdSpecialN_Anim(gp);
        assert_eq!((take(), f.cmd_vars[0]), ("reflect true,Wait".to_string(), 2));
        f.set_reflecting(1);
        f.cmd_vars[0] = 0;
        t::ftKb_ZdSpecialAirN_Anim(gp);
        assert_eq!(take(), "Fall");
        assert_eq!(f.reflecting(), 0);

        // Phys: gravity waits `x0` frames, then falls with the Zelda-copy acceleration.
        da.specialn_zd_fall_acceleration = 0.125;
        f.co_attrs.terminal_velocity = 2.0;
        f.mv.zd.specialn.x0 = 2;
        f.self_vel.y = 0.0;
        t::ftKb_ZdSpecialAirN_Phys(gp);
        assert_eq!((f.mv.zd.specialn.x0, f.self_vel.y), (1, 0.0));
        f.mv.zd.specialn.x0 = 0;
        t::ftKb_ZdSpecialAirN_Phys(gp);
        assert_eq!(f.self_vel.y, -0.125);
        assert_eq!(take(), "AEF8,AEF8");
        t::ftKb_ZdSpecialN_Phys(gp);
        assert_eq!(take(), "groundphys,AEF8");

        // Coll: leaving the ground swaps to the air state, restoring the reflect bubble.
        f.set_x2219_b0(1);
        f.cmd_vars[0] = 2;
        t::ftKb_SpecialNSk_80105E8C(gp);
        assert_eq!(take(), format!("ms {} {:x},reflect true", ftKb_MS_ZdSpecialAirN, t::ftZd_MF_SpecialN_Coll));
        assert!(f.pre_hitlag_cb.is_some());
        st::FT_80082708 = Some(|_| 0);
        t::ftKb_ZdSpecialN_Coll(gp);
        assert!(take().starts_with(&format!("ms {}", ftKb_MS_ZdSpecialAirN)));
        st::FT_80081D0C = Some(|_| 1);
        f.cmd_vars[0] = 0;
        t::ftKb_ZdSpecialAirN_Coll(gp);
        assert_eq!(take(), format!("toground,ms {} {:x}", ftKb_MS_ZdSpecialN, t::ftZd_MF_SpecialN_Coll));
        assert_eq!(t::ftZd_MF_SpecialN_Coll, 0x0C4C508E);
    }
}
