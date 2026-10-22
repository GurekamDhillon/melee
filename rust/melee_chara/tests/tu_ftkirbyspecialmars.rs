//! Smoke test for tu::ftkirbyspecialmars (stateful; no fused sites) through recording hooks.
//! One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkirbyspecialmars as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> std::vec::Vec<String> {
    unsafe { core::mem::take(&mut LOG) }
}
static mut MARS_HAT: KirbyHatStruct = unsafe { core::mem::zeroed() };
static mut FE_HAT: KirbyHatStruct = unsafe { core::mem::zeroed() };

#[test]
fn smoke() {
    unsafe { smoke_inner() }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn smoke_inner() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(core::mem::zeroed());
    let da: &'static mut ftKb_DatAttrs = Box::leak(Box::new(ftKb_DatAttrs::zeroed()));
    let mut bones: std::vec::Vec<FighterBone> = (0..64).map(|_| core::mem::zeroed()).collect();
    f.parts = bones.as_mut_ptr();
    f.dat_attrs = da as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    melee_fighter::hooks::FTPARTS_GET_BONE_INDEX = Some(|_, p| p);
    st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, af, _, _, _| log(format!("ms {ms} {fl:x} {af}")));
    st::FTANIM_8006EBA4 = Some(|_| {});
    st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
    st::FT_8008A2BC = Some(|_| log("Wait".into()));
    st::FTCO_FALL_ENTER = Some(|_| log("Fall".into()));
    st::FT_80082708 = Some(|_| 0);
    st::FT_80081D0C = Some(|_| 1);
    st::FTCOMMON_8007D7FC = Some(|_| log("7FC".into()));
    t::ext::EFSYNC_SPAWN = Some(|id, _, va| {
        log(format!("efsync {id:x} {}", va.len()));
        core::ptr::null_mut()
    });
    t::ext::FT_80459B88_HAT = Some(|i| {
        log(format!("hat {i}"));
        if i == Ft_Kind_Mars as i32 - 1 {
            unsafe { &raw mut MARS_HAT }
        } else {
            unsafe { &raw mut FE_HAT }
        }
    });
    t::ext::FTCOMMON_SETACCESSORY = Some(|_, j| log(format!("setacc {}", j as usize)));
    t::ext::FTCOMMON_GETMODELSCALE = Some(|_| 1.5);
    t::ext::HSD_JOBJSETSCALE = Some(|_, s| log(format!("scale {} {} {}", unsafe { (*s).x }, unsafe { (*s).y }, unsafe { (*s).z })));
    t::ext::LB_8000C2F8 = Some(|_, _| log("attach".into()));
    t::ext::FTCO_800BFFD0 = Some(|_, a, b| {
        log(format!("colanim {a} {b}"));
        true
    });
    t::ext::FTCOLL_8007ABD0 = Some(|_, dmg, _| log(format!("hit {dmg}")));

    da.ms.charge_iterations = 2;
    da.fe.charge_iterations = 3;
    da.ms.base_damage = 10;
    da.ms.additional_damage_per_iteration = 4;
    da.ms.air_horizontal_momentum_preservation = 2.0;
    da.ms.air_horizontal_deceleration_rate = 0.5;
    MARS_HAT.hat_dynamics[0] = 0x10 as *mut _;
    FE_HAT.hat_dynamics[0] = 0x20 as *mut _;

    // Start (ground): gr_vel is divided by the preservation, accessory scaled to the model scale.
    f.u.kb.hat.kind = Ft_Kind_Mars;
    f.gr_vel = 6.0;
    t::ftKb_SpecialNMs_8010B2FC(gp);
    assert_eq!(f.gr_vel, 3.0);
    assert_eq!(
        take(),
        [
            format!("ms {} 0 0", ftKb_MS_MsSpecialNStart),
            format!("hat {}", Ft_Kind_Mars - 1),
            format!("hat {}", Ft_Kind_Emblem - 1),
            "setacc 16".to_string(),
            "scale 1.5 1.5 1.5".to_string(),
            "attach".to_string(),
        ]
    );
    // Air start: horizontal speed divided, rising speed kept, falling zeroed.
    f.self_vel = Vec3 { x: 8.0, y: -2.0, z: 0.0 };
    f.u.kb.hat.kind = Ft_Kind_Emblem;
    da.fe.air_horizontal_momentum_preservation = 4.0;
    t::ftKb_SpecialNMs_8010B4A0(gp);
    assert_eq!((f.self_vel.x, f.self_vel.y), (2.0, 0.0));
    assert!(take().contains(&"setacc 32".to_string()));
    f.cmd_vars[0] = 5;
    f.mv.ms.specialn.cur_frame = 77;
    t::fn_8010B2E8(gp);
    assert_eq!((f.cmd_vars[0], f.mv.ms.specialn.cur_frame), (0, 0));

    // Start Anim end -> loop state, then the color-animation hook per hat.
    f.u.gw.x2238_panicCharge = 0x12;
    f.u.kb.hat.kind = Ft_Kind_Mars;
    t::ftKb_MsSpecialNStart_Anim(gp);
    assert_eq!(take(), [format!("ms {} 3200 0", ftKb_MS_MsSpecialNLoop), "colanim 99 0".to_string()]);
    f.u.gw.x2238_panicCharge = 0;
    f.u.kb.hat.kind = Ft_Kind_Emblem;
    t::ftKb_MsSpecialAirNStart_Anim(gp);
    assert_eq!(take(), [format!("ms {} 3200 0", ftKb_MS_FeSpecialAirNLoop), "colanim 100 0".to_string()]);

    // Loop Anim: charge frames past charge_iterations * 30 release the sword.
    f.u.kb.hat.kind = Ft_Kind_Mars;
    f.mv.ms.specialn.cur_frame = 59;
    t::ftKb_MsSpecialNLoop_Anim(gp);
    assert!(take().is_empty());
    t::ftKb_MsSpecialAirNLoop_Anim(gp);
    assert_eq!(f.cmd_vars[0], 1);
    assert_eq!(take(), [format!("ms {} 2000 1", ftKb_MS_MsSpecialAirNEnd0 + 1)]);
    // Released B: End0 (no full charge) vs End1 (cmd_vars[0] set by the timeout).
    f.cmd_vars[0] = 5;
    f.input.held_buttons[0] = HSD_PAD_B as u32;
    t::ftKb_MsSpecialNLoop_IASA(gp);
    assert_eq!(f.cmd_vars[0], 5);
    f.input.held_buttons[0] = 0;
    t::ftKb_MsSpecialNLoop_IASA(gp);
    assert_eq!(f.cmd_vars[0], 0);
    assert_eq!(take(), [format!("ms {} 2000 1", ftKb_MS_MsSpecialNEnd0)]);
    assert!(f.accessory4_cb.is_some());

    // End Anim: every enabled hit capsule gets base + (frame/30) * extra, through f32.
    f.mv.ms.specialn.cur_frame = 95; // 95/30 = 3
    f.cmd_vars[0] = 0;
    f.x914[1].state = HitCapsule_Enabled;
    f.x914[3].state = HitCapsule_Enabled;
    t::ftKb_MsSpecialNEnd_Anim(gp);
    let l = take();
    assert_eq!(l.len(), 3);
    assert!(l[0] == "hit 22" && l[1] == "hit 22" && l[2] == "Wait");
    f.cmd_vars[0] = 1; // timed out: no damage scaling
    t::ftKb_MsSpecialAirNEnd_Anim(gp);
    assert_eq!(take(), ["Fall"]);

    // Phys: ground slide-out uses the deceleration rate; Coll swaps ground/air ids.
    f.u.kb.hat.kind = Ft_Kind_Mars;
    f.cur_anim_frame = 3.0;
    t::ftKb_MsSpecialNStart_Coll(gp);
    t::ftKb_MsSpecialAirNLoop_Coll(gp);
    t::ftKb_MsSpecialAirNEnd_Coll(gp);
    f.cmd_vars[0] = 0;
    t::ftKb_MsSpecialAirNEnd_Coll(gp);
    assert_eq!(
        take(),
        [
            format!("ms {} c4c7084 3", ftKb_MS_MsSpecialAirNStart),
            "7FC".to_string(),
            format!("ms {} c4c7a86 3", ftKb_MS_MsSpecialNLoop),
            "7FC".to_string(),
            format!("ms {} c4c708e 3", ftKb_MS_MsSpecialNEnd0 + 1),
            "7FC".to_string(),
            format!("ms {} c4c708e 3", ftKb_MS_MsSpecialNEnd0),
        ]
    );
}
