//! Smoke test for tu::ftkirbyspeciallink (stateful; no fused sites) through recording hooks.
//! One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkirbyspeciallink as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> std::vec::Vec<String> {
    unsafe { core::mem::take(&mut LOG) }
}
static mut ARROW: HSD_GObj = unsafe { core::mem::zeroed() };
static mut BOW: HSD_GObj = unsafe { core::mem::zeroed() };
static mut GROUP: ftLk_SpecialN_Vec3Group = unsafe { core::mem::zeroed() };

#[test]
fn smoke() {
    unsafe { smoke_inner() }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn smoke_inner() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut ftKb_DatAttrs = Box::leak(Box::new(ftKb_DatAttrs::zeroed()));
    let mut bones: std::vec::Vec<FighterBone> = (0..64).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.parts = bones.as_mut_ptr();
    f.dat_attrs = da as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        GROUP.a = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        GROUP.b = Vec3 { x: 1.0, y: 1.0, z: 0.0 };
        GROUP.c = Vec3 { x: 0.5, y: 0.25, z: 0.0 };
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, af, _, _, _| log(format!("ms {ms} {fl:x} {af}")));
        st::FTANIM_8006EBA4 = Some(|_| {});
        st::FTANIM_SETANIMRATE = Some(|_, r| log(format!("rate {r}")));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        st::FT_8008A2BC = Some(|_| log("Wait".into()));
        st::FTCO_FALL_ENTER = Some(|_| log("Fall".into()));
        st::FTCO_80096900 = Some(|_, _, _, _, m, lag| log(format!("fallspecial {m} {lag}")));
        st::FT_80082708 = Some(|_| 0);
        st::FT_80081D0C = Some(|_| 1);
        st::FTCOMMON_8007D7FC = Some(|_| log("7FC".into()));
        t::ext::LB_8000B1CC = Some(|_, _, out| unsafe { *out = Vec3 { x: 3.0, y: 4.0, z: 9.0 } });
        t::ext::IT_802A83E0 = Some(|fd, _, _, part, id| {
            log(format!("arrow {fd} {part} {id}"));
            unsafe { &raw mut ARROW }
        });
        t::ext::IT_802AF1A4 = Some(|fd, _, _, part, id| {
            log(format!("bow {fd} {part} {id}"));
            unsafe { &raw mut BOW }
        });
        t::ext::ITLINKARROW_802A850C = Some(|_, p1, p2, ang, charge, max| {
            log(format!("shoot {} {} {} {ang} {charge} {max}", unsafe { (*p1).z }, unsafe { (*p2).z }, unsafe { (*p1).x }));
            true
        });
        t::ext::IT_802A8398 = Some(|_, b, a| log(format!("aim {} {}", unsafe { (*b).x }, unsafe { (*a).x })));
        t::ext::IT_802A8A7C = Some(|_| log("freearrow".into()));
        t::ext::IT_802AF304 = Some(|_| log("freebow".into()));
        t::ext::FTPICKUPITEM_80094818 = Some(|_, b| log(format!("pickup {b}")));
        t::ext::FTKB_INIT_803B7548 = Some(|| unsafe { &raw const GROUP });
    }
    da.specialn_lk_bow_item_loader_id = 11;
    da.specialn_cl_bow_item_loader_id = 22;
    da.specialn_lk_bow_item_id = 33;
    da.specialn_cl_bow_item_id = 44;
    da.specialn_lk_arrow_charge_speed = 0.5;
    da.specialn_cl_arrow_charge_speed = 0.75;
    da.specialn_lk_max_charge = 40.0;
    da.specialn_cl_max_charge = 20.0;
    da.specialn_lk_freefall_toggle = 0.0;
    da.specialn_cl_freefall_toggle = 9.0;
    f.facing_dir = 1.0;

    // Index lookup: only while a bow exists and the motion belongs to the hat.
    f.u.kb.hat.kind = Ft_Kind_Link;
    f.motion_id = ftKb_MS_LkSpecialAirNLoop as i32;
    assert_eq!(t::ftKb_SpecialNLk800FB394(gp), ftLk_SpecialNIndex_None);
    f.u.kb.x78 = unsafe { &raw mut BOW };
    assert_eq!(t::ftKb_SpecialNLk800FB394(gp), 4);
    f.u.kb.hat.kind = Ft_Kind_CLink;
    f.motion_id = ftKb_MS_ClSpecialNEnd as i32;
    assert_eq!(t::ftKb_SpecialNLk800FB394(gp), 2);
    f.motion_id = ftKb_MS_LkSpecialNEnd as i32;
    assert_eq!(t::ftKb_SpecialNLk800FB394(gp), ftLk_SpecialNIndex_None);

    // Allow-input check: matching state with the x2071_b6 flag clear returns false.
    f.u.kb.hat.kind = Ft_Kind_Link;
    f.motion_id = ftKb_MS_LkSpecialNLoop as i32;
    assert!(!t::ftKb_SpecialNLk800FB470(gp));
    unsafe { f.x2070.anon_0.set_x2071_b6(1) };
    assert!(t::ftKb_SpecialNLk800FB470(gp));
    assert!(t::ftKb_SpecialNLk800FB470(core::ptr::null_mut()));

    // Arrow / bow creation.
    f.u.kb.x74 = core::ptr::null_mut();
    f.u.kb.x78 = core::ptr::null_mut();
    f.cmd_vars[0] = 1;
    assert!(!t::ftKb_SpecialNLk800FB500(gp));
    assert_eq!(take(), [format!("arrow 1 {FtPart_RHandN} 11")]);
    assert!(!f.u.kb.x74.is_null() && f.death2_cb.is_some());
    assert!(!t::ftKb_SpecialNLk800FB5F4(gp));
    assert_eq!(take(), [format!("bow 1 {FtPart_R3rdNa} 33")]);
    assert!(t::ftKb_SpecialNLk800FB5F4(gp)); // bow already exists

    // Fire: needs cmd_vars[1] == 1 and an arrow; frees the arrow slot and restores the held item.
    f.cmd_vars[1] = 1;
    f.mv.lk.specialn.x0.y = 7.0;
    t::ftKb_SpecialNLk800FB6DC(gp);
    assert_eq!(take(), ["shoot 0 0 3 0.08726646 7 40".to_string(), "pickup false".to_string()]);
    assert!(f.u.kb.x74.is_null());

    // Start Anim: charging flag, aim vector, then loop state when the animation ends.
    f.cmd_vars[2] = 1;
    f.mv.lk.specialn.x0.x = 0.0;
    f.u.kb.x74 = core::ptr::null_mut();
    f.cmd_vars[0] = 0;
    t::ftKb_LkSpecialNStart_Anim(gp);
    assert_eq!(f.mv.lk.specialn.x0.x, 1.0);
    let l = take();
    assert_eq!(l[0], "rate 0.5");
    assert_eq!(l[1], format!("ms {} 40010 0", ftKb_MS_LkSpecialNLoop));
    let x8 = f.mv.lk.specialn.x8;
    assert_eq!((x8.y, x8.z), (0.5, 0.75));
    assert_eq!(x8.x, melee_runtime::lbtrigf::atan2f(1.0, 1.0));

    // IASA: charge counts up to the hat's cap, release enters End.
    f.mv.lk.specialn.x0.y = 39.5;
    f.input.held_buttons[0] = HSD_PAD_B as u32;
    t::ftKb_LkSpecialNStart_IASA(gp);
    assert_eq!(f.mv.lk.specialn.x0.y, 40.0);
    assert!(take().is_empty());
    f.input.held_buttons[0] = 0;
    f.u.kb.hat.kind = Ft_Kind_CLink;
    t::ftKb_LkSpecialAirNLoop_IASA(gp);
    assert_eq!(f.mv.lk.specialn.x0.y, 20.0);
    assert_eq!(take(), [format!("ms {} 40010 0", ftKb_MS_ClSpecialAirNEnd)]);

    // Air end of the Start / End: freefall toggle picks Fall or the special fall.
    f.u.kb.hat.kind = Ft_Kind_Link;
    t::ftKb_LkSpecialAirNEnd_Anim(gp);
    assert_eq!(take(), ["Fall"]);
    f.u.kb.hat.kind = Ft_Kind_CLink;
    t::ftKb_LkSpecialAirNEnd_Anim(gp);
    assert_eq!(take(), ["fallspecial 1 9"]);

    // Coll: ground to air keeps the animation frame; End first fires a pending arrow.
    f.u.kb.hat.kind = Ft_Kind_CLink;
    f.cur_anim_frame = 3.0;
    f.cmd_vars[1] = 0;
    t::ftKb_LkSpecialNStart_Coll(gp);
    assert_eq!(take(), [format!("ms {} c4c5090 3", ftKb_MS_ClSpecialAirNStart)]);
    t::ftKb_LkSpecialAirNEnd_Coll(gp);
    assert_eq!(take(), ["7FC".to_string(), format!("ms {} c4c5090 3", ftKb_MS_ClSpecialNEnd)]);

    // Cleanup callbacks free the item slots.
    f.u.kb.x74 = unsafe { &raw mut ARROW };
    f.u.kb.x78 = unsafe { &raw mut BOW };
    t::ftKb_SpecialNLk800FB800(gp);
    t::ftKb_SpecialNLk800FB840(gp);
    assert_eq!(take(), ["freearrow", "freebow"]);
    assert!(f.u.kb.x74.is_null() && f.u.kb.x78.is_null());
}
