//! tu::ftnessspecials smoke test through recording hooks (one #[test]: hooks are process globals).
#![allow(static_mut_refs)]
use melee_chara::stubs as st;
use melee_chara::tu::ftnessspecials as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let attr: &'static mut ftNessAttributes = Box::leak(Box::new(ftNessAttributes::zeroed()));
    f.dat_attrs = attr as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, _, _, _| log(format!("ms {ms} {fl:x}")));
        st::FTANIM_8006EBA4 = Some(|_| log("anim".into()));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        st::FT_8008A2BC = Some(|_| log("8A2BC".into()));
        st::FTCO_FALL_ENTER = Some(|_| log("Fall".into()));
        st::FT_800827A0 = Some(|_| 0);
        st::FT_80081D0C = Some(|_| 1);
        st::FTCO_LANDINGFALLSPECIAL_ENTER = Some(|_, _, lag| log(format!("land {lag}")));
    }
    f.cmd_vars[0] = 5;
    t::ftNs_SpecialS_Enter(gp);
    assert_eq!(f.cmd_vars[0], 0);
    assert!(f.accessory4_cb.is_some());
    t::ftNs_SpecialAirS_Enter(gp);
    t::ftNs_SpecialS_Anim(gp);
    t::ftNs_SpecialAirS_Anim(gp);
    t::ftNs_SpecialS_Coll(gp);
    attr.x38_PKFIRE_LANDING_LAG = 7.0;
    t::ftNs_SpecialAirS_Coll(gp);
    let got = unsafe { LOG.join(",") };
    let want = format!(
        "ms {} 0,anim,ms {} 0,anim,8A2BC,Fall,Fall,land 7",
        ftNs_MS_SpecialS as i32, ftNs_MS_SpecialAirS as i32
    );
    assert_eq!(got, want);
}
