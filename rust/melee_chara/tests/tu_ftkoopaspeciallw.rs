//! Smoke test for tu::ftkoopaspeciallw (stateful; no fused sites) through recording hooks.
//! One #[test]: the hooks are process globals.
#![allow(static_mut_refs, unused_unsafe)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkoopaspeciallw as t;
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
        let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
        let da: &'static mut ftKoopaAttributes = Box::leak(Box::new(ftKoopaAttributes::zeroed()));
        let mut parts: Box<[FighterBone; 2]> = Box::new(core::mem::zeroed());
        f.dat_attrs = da as *mut _ as *mut _;
        f.parts = parts.as_mut_ptr();
        g.user_data = &mut *f as *mut Fighter as *mut _;
        let gp: *mut HSD_GObj = &mut *g;
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, _, _, _| log(format!("ms {ms} {fl:x}")));
        st::FTANIM_8006EBA4 = Some(|_| log("6EBA4".into()));
        st::FT_CHECKGROUNDANDLEDGE = Some(|_, d| {
            log(format!("ledge {d}"));
            1
        });
        st::FTCLIFFCOMMON_80081298 = Some(|_| 1);
        t::ext::EFSYNC_SPAWN = Some(|id, _, args| {
            log(format!("fx {id:x} {}", args.len()));
            core::ptr::null_mut()
        });
        t::ext::FTCLIFFCOMMON_80081370 = Some(|_| log("cliffcatch".into()));
        st::FTCO_FALL_ENTER = Some(|_| log("Fall".into()));
        melee_fighter::hooks::MPLIB_8004D164_LINE_COUNT = Some(|| None);
        st::FTCOMMON_8007D7FC = Some(|_| log("toground".into()));

        // Air Enter: horizontal / vertical momentum scaled by the attributes, motion 0x16A.
        da.x80 = 0.5;
        da.x84 = 0.25;
        f.self_vel.x = 8.0;
        f.self_vel.y = 8.0;
        t::ftKp_SpecialAirLw_Enter(gp);
        assert_eq!((f.self_vel.x, f.self_vel.y), (4.0, 2.0));
        assert_eq!(take(), "ms 362 0,6EBA4");
        assert_eq!(f.x6A4_transNOffset.y, 0.0);
        assert_eq!(f.x2223_b4(), 1);

        // Air Phys: gravity, deaccel, the fixed fall speed once the script says so.
        da.x8C = 0.0;
        da.x90 = 100.0;
        da.x88 = 0.0;
        da.x94 = -2.0;
        f.co_attrs.terminal_velocity = 100.0;
        f.self_vel.y = 3.0;
        f.cmd_vars[1] = 0;
        t::ftKp_SpecialAirLw_Phys(gp);
        assert_eq!(f.self_vel.y, 3.0);
        f.cmd_vars[1] = 1;
        f.self_vel.x = 5.0;
        t::ftKp_SpecialAirLw_Phys(gp);
        assert_eq!((f.self_vel.x, f.self_vel.y), (0.0, -2.0));
        assert!(f.accessory4_cb.is_some());
        f.set_x2219_b0(1);
        f.accessory4_cb = None;
        t::ftKp_SpecialAirLw_Phys(gp);
        assert!(f.accessory4_cb.is_none());

        // The accessory callbacks spawn their effect once and clear themselves.
        f.set_x2219_b0(0);
        f.accessory4_cb = Some(t::cb_fn_80134590);
        t::fn_80134590(gp);
        assert_eq!(take(), "fx 4df 1");
        assert!(f.accessory4_cb.is_none() && f.pre_hitlag_cb.is_some());
        f.set_x2219_b0(0);
        t::fn_80134518(gp);
        assert_eq!(take(), "fx 4d8 1");
        assert_eq!(f.x2219_b0(), 1);
        t::fn_80134518(gp);
        assert_eq!(take(), "");

        // Coll: landing on a ledge with the script flag set and not rising -> landing state;
        // otherwise back to the air state with the ECB unlocked.
        f.facing_dir = -1.0;
        f.cmd_vars[0] = 1;
        f.self_vel.y = -1.0;
        t::ftKp_SpecialAirLw_Coll(gp);
        assert_eq!(take(), "ledge -1,toground,ms 363 0");
        f.self_vel.y = 1.0;
        f.facing_dir = 1.0;
        f.ground_or_air = GA_Ground;
        t::ftKp_SpecialLw_Coll(gp);
        assert_eq!(take(), "ledge 1");
        assert_eq!(f.ground_or_air, GA_Air);
        // No ledge: the cliff catch runs when the script flag is set.
        st::FT_CHECKGROUNDANDLEDGE = Some(|_, _| 0);
        t::ftKp_SpecialAirLw_Coll(gp);
        assert_eq!(take(), "cliffcatch");
        f.cmd_vars[0] = 0;
        t::ftKp_SpecialAirLw_Coll(gp);
        assert_eq!(take(), "");
    }
}
