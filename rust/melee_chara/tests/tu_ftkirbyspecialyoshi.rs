//! Smoke test for tu::ftkirbyspecialyoshi and the victim-side entry of tu::ftkirbyyoshiegg
//! (stateful; no fused sites) through recording hooks. One #[test]: hooks are process globals.
#![allow(static_mut_refs, unused_unsafe)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkirbyspecialyoshi as t;
use melee_chara::tu::ftkirbyyoshiegg as egg;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
static mut HAT: Option<Box<KirbyHatStruct>> = None;
static mut FD: *mut ftData = core::ptr::null_mut();
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
        let mut v: Box<Fighter> = Box::new(Fighter::zeroed());
        let mut g: Box<HSD_GObj> = Box::new(core::mem::zeroed());
        let mut vg: Box<HSD_GObj> = Box::new(core::mem::zeroed());
        let mut item: Box<HSD_GObj> = Box::new(core::mem::zeroed());
        let mut hat_jobj: Box<HSD_JObj> = Box::new(core::mem::zeroed());
        let mut acc_jobj: Box<HSD_JObj> = Box::new(core::mem::zeroed());
        let da: &'static mut ftKb_DatAttrs = Box::leak(Box::new(ftKb_DatAttrs::zeroed()));
        let ea: &'static mut ftKb_DatAttrs = Box::leak(Box::new(ftKb_DatAttrs::zeroed()));
        let fd: &'static mut ftData = Box::leak(Box::new(core::mem::zeroed()));
        let mut parts: Box<[FighterBone; 4]> = Box::new(core::mem::zeroed());
        let mut vparts: Box<[FighterBone; 4]> = Box::new(core::mem::zeroed());
        let cd: &'static ftCommonData = Box::leak(Box::new(ftCommonData::zeroed()));
        melee_fighter::hooks::P_FT_COMMON_DATA = cd;
        let map: &'static mut [u8; 128] = Box::leak(Box::new([0u8; 128]));
        let pt: &'static mut FighterPartsTable = Box::leak(Box::new(FighterPartsTable { joint_to_part: map.as_mut_ptr(), part_to_joint: map.as_mut_ptr(), parts_num: 128 }));
        let tbls: &'static mut std::vec::Vec<*mut FighterPartsTable> = Box::leak(Box::new(vec![pt as *mut _; Ft_Kind_Max as usize]));
        melee_fighter::hooks::FT_PARTS_TABLE = tbls.as_mut_ptr();
        let mut hat: Box<KirbyHatStruct> = Box::new(KirbyHatStruct::zeroed());
        for i in 0..5 {
            hat.hat_dynamics[i] = (0x100 * (i + 1)) as *mut ftDynamics;
        }
        HAT = Some(hat);
        fd.ext_attr = ea as *mut _ as *mut _;
        FD = fd;
        f.dat_attrs = da as *mut _ as *mut _;
        f.parts = parts.as_mut_ptr();
        v.parts = vparts.as_mut_ptr();
        v.x20A0_accessory = &mut *acc_jobj;
        f.u.kb.hat.jobj = &mut *hat_jobj;
        g.user_data = &mut *f as *mut Fighter as *mut _;
        vg.user_data = &mut *v as *mut Fighter as *mut _;
        vg.hsd_obj = &mut *hat_jobj as *mut HSD_JObj as *mut _;
        let gp: *mut HSD_GObj = &mut *g;
        let vp: *mut HSD_GObj = &mut *vg;
        let itp: *mut HSD_GObj = &mut *item;

        melee_fighter::hooks::MPLIB_8004D164_LINE_COUNT = Some(|| None);
        st::FTCOMMON_8007D7FC = Some(|_| log("toground".into()));
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, _, _, a3| log(format!("ms {ms} {fl:x}{}", if a3.is_null() { "" } else { " +arg" })));
        st::FTANIM_8006EBA4 = Some(|_| log("6EBA4".into()));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        st::FT_8008A2BC = Some(|_| log("Wait".into()));
        st::FTCO_FALL_ENTER = Some(|_| log("Fall".into()));
        t::ext::FT_80459B88_HAT = Some(|k| {
            log(format!("hat {k}"));
            &mut **HAT.as_mut().unwrap() as *mut KirbyHatStruct
        });
        t::ext::FTKB_KIRBYFTDATA = Some(|| FD);
        t::ext::HSD_JOBJADDANIMALL = Some(|_, a, _, _| log(format!("addanim {:x}", a as usize)));
        t::ext::HSD_JOBJREQANIMALL = Some(|_, fr| log(format!("reqanim {fr}")));
        t::ext::HSD_JOBJANIMALL = Some(|_| log("animall".into()));
        t::ext::LB_8000BA0C = Some(|_, fr| log(format!("BA0C {fr}")));
        t::ext::LB_8000B1CC = Some(|_, _, out| *out = Vec3 { x: 1.0, y: 2.0, z: 3.0 });
        t::ext::IT_802F2D98 = Some(|_, _| log("2D98".into()));
        t::ext::IT_802F2E7C = Some(|_, d| log(format!("2E7C {d}")));
        t::ext::IT_802F2F34 = Some(|_, a| {
            let a = &*a;
            log(format!(
                "lay pos {} {} {} vel {} {} lifetime {} f2 {} f3 {} x24 {} kind {}",
                a.pos.x, a.pos.y, a.pos.z, a.vel.x, a.vel.y, a.lifetime, a.float2, a.float3, a.x24, a.kind
            ))
        });
        t::ext::FTKB_SPECIALNYS_8010AA2C = Some(|_| log("AA2C".into()));
        t::ext::FTCO_800DE2CC = Some(|_, _| log("DE2CC".into()));
        st::FT_8008403C = Some(|_, cb| log(format!("coll ground {}", cb.is_some())));
        st::FT_80082C74 = Some(|_, cb| log(format!("coll air {}", cb.is_some())));
        egg::ext::FTCOLL_8007B0C0 = Some(|_, s| log(format!("hurtall {s}")));
        egg::ext::FTCOLL_HURTBOXINIT = Some(|_, _, h| log(format!("hurtbox {}", (*h).scale)));
        egg::ext::FTCOMMON_SETACCESSORY = Some(|_, j| log(format!("setacc {:x}", j as usize)));
        egg::ext::HSD_JOBJSETSCALE = Some(|_, s| log(format!("setscale {} {} {}", (*s).x, (*s).y, (*s).z)));
        egg::ext::HSD_JOBJGETCHILD = Some(|_| core::ptr::null_mut());
        egg::ext::HSD_JOBJGETSCALE = Some(|_, out| *out = Vec3 { x: 9.0, y: 8.0, z: 7.0 });
        egg::ext::LB_8000C2F8 = Some(|_, _| log("C2F8".into()));
        egg::ext::FTCOMMON_INITGRAB = Some(|_, a, b| log(format!("initgrab {a} {b}")));

        // Attribute getters (via the pure helpers and the Kirby ftData).
        da.specialn_ys_initial_horizontal_momentum = 2.0;
        da.specialn_ys_initial_vertical_momentum = 3.0;
        da.specialn_ys_damage_multiplier = 4.0;
        da.specialn_ys_unk = 5.0;
        da.specialn_ys_growth_time = 6.0;
        da.specialn_ys_base_duration = 7.0;
        ea.specialn_ys_unk3 = 10.0;
        ea.specialn_ys_damage_multiplier = 4.0;
        ea.specialn_ys_egg_breakout_resistance = 1.5;
        ea.specialn_ys_horizontal_velocity_on_breakout = 0.5;
        ea.specialn_ys_vertical_velocity_on_breakout = 0.75;
        f.facing_dir = -1.0;
        let mut vec = Vec3 { x: 9.0, y: 9.0, z: 9.0 };
        t::ftKb_SpecialNYs_80109260(gp, &mut vec);
        assert_eq!((vec.x, vec.y, vec.z), (2.0, 3.0, 0.0));
        assert_eq!(t::ftKb_SpecialNYs_80109290(gp), -1.0);
        assert_eq!(t::ftKb_SpecialNYs_8010929C(gp), 4.0);
        assert_eq!(t::ftKb_SpecialNYs_801092BC(gp), 6.0);
        assert_eq!(t::ftKb_SpecialNYs_801092CC(gp), 7.0);
        assert_eq!(t::ftKb_SpecialNYs_801092DC(), 1.5);
        assert_eq!(t::ftKb_SpecialNYs_80109380(), 2.5);
        t::ftKb_SpecialNYs_80109354(&mut vec);
        assert_eq!((vec.x, vec.y, vec.z), (0.5, 0.75, 0.0));
        assert_eq!(t::ftKb_SpecialNYs_801093A0(gp), 0x100 as *mut ftDynamics);
        take();

        // Enter: hat animation attached, grab callbacks installed, damage callbacks installed.
        t::ftKb_SpecialNYs_8010941C(gp);
        assert_eq!(take(), format!("hat {},ms {} 0,6EBA4,addanim 200,reqanim 0,animall", Ft_Kind_Samus, ftKb_MS_YsSpecialN1));
        assert_eq!(f.x1A68, 4);
        assert!(f.grab_cb.is_some() && f.grabbed_cb.is_some() && f.death2_cb.is_some());
        t::ftKb_SpecialNYs_801094FC(gp);
        assert_eq!(take(), format!("hat {},ms {} 0,6EBA4,addanim 400,reqanim 0,animall", Ft_Kind_Samus, ftKb_MS_YsSpecialAirNCapture2));
        t::ftKb_SpecialNYs_801093B4(gp);
        assert_eq!(take(), format!("hat {},addanim 200,reqanim 0,animall,BA0C 0", Ft_Kind_Samus));

        // Anim: ends go to Wait / Fall.
        t::ftKb_YsSpecialN1_Anim(gp);
        t::ftKb_YsSpecialAirNCapture2_Anim(gp);
        assert_eq!(take(), "animall,Wait,animall,Fall");

        // Capture latch: the swallow state advances once the victim is held and the script sets var 0.
        f.cmd_vars[0] = 1;
        f.victim_gobj = vp;
        t::ftKb_YsSpecialNCapture1_Anim(gp);
        assert_eq!(take(), format!("animall,hat 13,ms {} 80012,addanim 300,reqanim 0", ftKb_MS_YsSpecialNCapture2_0));
        assert_eq!((f.cmd_vars[0], f.cmd_vars[1]), (0, 0));
        // The item-latched variants key on target_item_gobj instead.
        f.cmd_vars[0] = 1;
        f.target_item_gobj = itp;
        t::ftKb_YsSpecialAirNCapture1_Anim(gp);
        assert!(take().starts_with(&format!("hat 13,ms {} 80012,addanim 300", ftKb_MS_YsSpecialNCapture2_1)));
        f.cmd_vars[0] = 1;
        t::ftKb_YsSpecialAirCapture2_Anim(gp);
        assert!(take().starts_with(&format!("hat 13,ms {} 80012,addanim 500", ftKb_MS_YsSpecialAirN2_1)));
        f.cmd_vars[0] = 1;
        t::ftKb_YsSpecialAirCapture1_Anim(gp);
        assert!(take().starts_with(&format!("animall,hat 13,ms {} 80012,addanim 500", ftKb_MS_YsSpecialAirN2_0)));
        f.cmd_vars[0] = 0;
        t::ftKb_YsSpecialAirCapture1_Anim(gp);
        assert_eq!(take(), "animall");

        // Laying the egg: item attributes built from the fighter and the Kirby ftData.
        f.cmd_vars[0] = 1;
        f.cmd_vars[1] = 1;
        f.facing_dir = 1.0;
        t::ftKb_YsSpecialNCapture2_0_Anim(gp);
        assert_eq!(
            take(),
            format!(
                "animall,2E7C 0,lay pos 1 2 3 vel -2 3 lifetime 7 f2 1 f3 2.5 x24 4 kind {},Wait",
                It_Kind_Kirby_YoshiEggLay
            )
        );
        assert!(f.target_item_gobj.is_null());
        assert_eq!((f.cmd_vars[0], f.cmd_vars[1]), (0, 0));

        // Spitting the egg out: victim handed over, then the egg entry on the victim.
        f.cmd_vars[0] = 1;
        f.cmd_vars[1] = 1;
        f.victim_gobj = vp;
        v.co_attrs.xBC.size = 1.5;
        t::ftKb_YsSpecialNCapture2_1_Anim(gp);
        let got = take();
        assert!(got.starts_with("animall,AA2C,DE2CC,"), "{got}");
        assert!(got.contains(&format!("ms {} 40", ftCo_MS_KirbyYoshiEgg)), "{got}");
        assert!(got.contains("setscale 1.5 1.5 1.5"), "{got}");
        assert!(got.contains("hurtall 2"), "{got}");
        assert!(got.ends_with("Wait"), "{got}");
        assert_eq!(v.invisible(), 1);
        assert_eq!(v.mv.co.yoshiegg.x0, gp);
        assert_eq!(v.mv.co.walk.fast_anim_frame, 6.0);
        assert_eq!(v.facing_dir, 1.0);
        assert_eq!(v.dmg.x182c_behavior, 4.0);
        assert!(v.take_dmg_cb.is_some() && v.take_dmg_2_cb.is_some() && v.accessory4_cb.is_some());
        assert_eq!(v.mv.co.yoshiegg.x18.x, 9.0);

        // Coll hands the static transition callbacks to the common collision code.
        t::ftKb_YsSpecialN1_Coll(gp);
        t::ftKb_YsSpecialAirNCapture2_Coll(gp);
        assert_eq!(take(), "coll ground true,coll air true");

        // The transition callbacks themselves.
        f.cur_anim_frame = 3.0;
        t::fn_80109A84(gp);
        assert_eq!(take(), format!("toground,ms {} c4c5092", ftKb_MS_YsSpecialNCapture1_1));
        t::fn_80109B7C(gp);
        assert!(take().ends_with(&format!("ms {} c4c5092", ftKb_MS_YsSpecialAirN2_1)));
        t::fn_801095DC(gp);
        let got = take();
        assert!(got.starts_with(&format!("2D98,ms {} 81092", ftKb_MS_YsSpecialNCapture1_1)), "{got}");
        assert_eq!(f.x2222_b2(), 1);
        assert_eq!(f.x1A6A, 0x1FF);
        t::ftKb_SpecialNYs_8010A8BC(gp);
        assert!(f.target_item_gobj.is_null());
        assert_eq!(t::ftYs_MF_SpecialN_CollHit, 0x0C4C5080 | 2 | 16 | 8);
    }
}
