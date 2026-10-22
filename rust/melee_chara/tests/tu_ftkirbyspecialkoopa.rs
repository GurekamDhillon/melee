//! Smoke test for tu::ftkirbyspecialkoopa (stateful) through recording hooks. The flame spawn has
//! two fused sites (`pos.x += scale.y * (x_off * facing)`, `pos.y += y_off * scale.y`); the
//! inputs below make the fused (f64 multiply-add, one f32 rounding) and unfused results differ.
//! One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkirbyspecialkoopa as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
static mut RANDI: std::vec::Vec<i32> = std::vec::Vec::new();
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
    let da: &'static mut ftKb_DatAttrs = Box::leak(Box::new(ftKb_DatAttrs::zeroed()));
    let mut bones: std::vec::Vec<FighterBone> = (0..16).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.parts = bones.as_mut_ptr();
    f.dat_attrs = da as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, af, _, _, _| log(format!("ms {ms} {fl:x} {af}")));
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        st::FT_8008A2BC = Some(|_| log("Wait".into()));
        st::FT_80082708 = Some(|_| 0);
        st::FT_80081D0C = Some(|_| 1);
        st::FTCOMMON_8007D7FC = Some(|_| log("7FC".into()));
        t::ext::LB_8000B1CC = Some(|_, off, out| unsafe {
            assert_eq!((*off).z, 3.0);
            *out = Vec3 { x: f32::from_bits(0xc01bf53a), y: f32::from_bits(0xc01bf53a), z: 0.0 };
        });
        t::ext::HSD_RANDI = Some(|n| {
            assert_eq!(n, 0x20);
            unsafe { RANDI.remove(0) }
        });
        t::ext::ITEM_8026AE60 = Some(|| 7);
        t::ext::ITKOOPAFLAME_SPAWN = Some(|_, pos, fd, unk, gfx, speed, scale, kind| {
            let (x, y) = unsafe { ((*pos).x, (*pos).y) };
            log(format!("flame {:08x} {:08x} {fd} {unk} {gfx} {speed} {scale} {kind}", x.to_bits(), y.to_bits()));
            core::ptr::null_mut()
        });
        t::ext::FT_PLAYSFX = Some(|_, id, _, _| log(format!("sfx {id:x}")));
        t::ext::FT_80089824 = Some(|_| log("89824".into()));
        t::ext::FT_800892A0 = Some(|_| log("892A0".into()));
        t::ext::CAMERA_REQUESTQUAKE = Some(|k, _| log(format!("quake {k}")));
    }

    f.facing_dir = 1.0;
    f.x34_scale.y = f32::from_bits(0x3fbd1b19);
    da.specialn_kp_breath_x_offset = f32::from_bits(0x3fd76f55);
    da.specialn_kp_breath_y_offset = f32::from_bits(0x3fd76f55);
    da.specialn_kp_lowest_charge_graphic_size = 1.0;
    da.specialn_kp_flame_scale = 3.0;
    f.u.kb.x84 = 5.0;
    f.u.kb.x88 = 1.5; // (1.5-1)/(3-1) = 0.25: the lowest sfx
    unsafe {
        f.mv.kb.specialn_kp.x0[2] = -1;
        f.mv.kb.specialn_kp.x14 = 0;
        RANDI = vec![0, 4, 4]; // table[0]==0 -> table[4+0x20]
    }
    t::ftKb_SpecialNKp_800FA588(gp);
    let fused = 0x3d4bb162u32; // fmadds(1.477.., 1.683.., -2.436..); unfused would be 0x3d4bb180
    assert_eq!(
        take(),
        [
            format!("flame {fused:08x} {fused:08x} 1 0 0 5 1 {}", It_Kind_Kirby_KoopaFlame),
            "89824".into(),
            "892A0".into(),
            "sfx 24a2b".into(),
        ]
    );
    // dirs[0]==0 -> new_facing = dirs[randi+0x20] = dirs[0x24] = 0; stored in x0[2]; x14 steps.
    unsafe {
        assert_eq!(f.mv.kb.specialn_kp.x0[2], 0);
        assert_eq!(f.mv.kb.specialn_kp.x0[1], 7);
        assert_eq!(f.mv.kb.specialn_kp.x14, 1);
    }
    // x14 wraps modulo 12 and the sfx fires every third step.
    unsafe {
        f.mv.kb.specialn_kp.x14 = 11;
        f.mv.kb.specialn_kp.x0[2] = 1; // case 1: dirs[randi + 0x20]
        RANDI = vec![8];
    }
    f.u.kb.x88 = 2.9;
    t::ftKb_SpecialNKp_800FA588(gp);
    assert_eq!(unsafe { f.mv.kb.specialn_kp.x14 }, 0);
    assert_eq!(unsafe { f.mv.kb.specialn_kp.x0[2] }, 3); // dirs[8 + 0x20]
    assert_eq!(take().len(), 1); // flame only (11 % 3 != 0)

    // Fuel recharge clamps to the maxima, and only outside the flame states 443..448.
    da.specialn_kp_fuel_recharge_rate = 2.0;
    da.specialn_kp_max_fuel = 6.0;
    da.specialn_kp_flame_size_recharge_rate = 1.0;
    f.u.kb.x84 = 5.0;
    f.u.kb.x88 = 2.5;
    f.motion_id = 100;
    t::ftKb_SpecialNKp_800FA7D4(gp);
    assert_eq!(unsafe { (f.u.kb.x84, f.u.kb.x88) }, (6.0, 3.0));
    f.motion_id = 445;
    f.u.kb.x84 = 1.0;
    t::ftKb_SpecialNKp_800FA7D4(gp);
    assert_eq!(unsafe { f.u.kb.x84 }, 1.0);
    assert_eq!(t::ftKb_SpecialNKp_800FA83C(gp), 6);
    assert_eq!(t::ftKb_SpecialNKp_800FA878(gp), 3);
    assert_eq!(t::ftKb_SpecialNKp_800FA83C(core::ptr::null_mut()), 1);
    assert_eq!(t::ftKb_SpecialNKp_800FA878(core::ptr::null_mut()), 2);

    // Start: Bowser vs Giga Bowser ids.
    unsafe { RANDI.clear() };
    f.u.kb.hat.kind = Ft_Kind_Koopa;
    t::ftKb_SpecialNKp_800FA8B4(gp);
    f.u.kb.hat.kind = Ft_Kind_GKoops;
    t::ftKb_SpecialNKp_800FA958(gp);
    assert_eq!(
        take(),
        [format!("ms {} 0 0", ftKb_MS_KpSpecialNStart), format!("ms {} 0 0", ftKb_MS_GkSpecialAirNStart)]
    );
    assert_eq!(unsafe { f.mv.kb.specialn_kp.x0 }, [0, 7, -1, 0]);

    // Loop Anim: shake counter with the quake at 0, modulo the shake frequency.
    da.specialn_kp_screen_shake_frequency = 3;
    unsafe {
        f.mv.kb.specialn_kp.x18 = 0;
        f.mv.kb.specialn_kp.x10 = 2;
    }
    f.cur_anim_frame = 0.0;
    t::ftKb_KpSpecialN_Anim(gp);
    assert_eq!(take(), ["quake 2"]);
    unsafe {
        assert_eq!((f.mv.kb.specialn_kp.x10, f.mv.kb.specialn_kp.x18), (1, 1));
    }
    t::ftKb_KpSpecialAirN_Anim(gp);
    t::ftKb_KpSpecialN_Anim(gp);
    assert_eq!(unsafe { f.mv.kb.specialn_kp.x18 }, 0);
    take();
    // Start Anim end -> loop state per hat.
    f.u.kb.hat.kind = Ft_Kind_Koopa;
    t::ftKb_KpSpecialNStart_Anim(gp);
    t::ftKb_KpSpecialAirNStart_Anim(gp);
    f.u.kb.hat.kind = Ft_Kind_GKoops;
    t::ftKb_KpSpecialNStart_Anim(gp);
    assert_eq!(
        take(),
        [
            format!("ms {} 0 0", ftKb_MS_KpSpecialN),
            format!("ms {} 0 0", ftKb_MS_KpSpecialAirN),
            format!("ms {} 0 0", ftKb_MS_GkSpecialN),
        ]
    );

    // IASA: before the check frequency the flame fires at phase 0.
    da.specialn_kp_b_button_check_frequency = 2;
    da.specialn_kp_spew_flame_velocity = 0.5;
    da.specialn_kp_lowest_charge_graphic_size = 0.25;
    f.u.kb.hat.kind = Ft_Kind_Koopa;
    f.u.kb.x84 = 4.0;
    f.u.kb.x88 = 1.0;
    unsafe {
        f.mv.kb.specialn_kp.x0 = [0, 7, 1, 0];
        f.mv.kb.specialn_kp.x14 = 1;
        RANDI = vec![0];
    }
    t::ftKb_KpSpecialN_IASA(gp);
    let l = take();
    assert_eq!(l.len(), 1);
    assert!(l[0].starts_with("flame"));
    unsafe {
        assert_eq!(f.mv.kb.specialn_kp.x0[0], 1);
        assert_eq!(f.mv.kb.specialn_kp.x0[3], 1);
    }
    assert_eq!(unsafe { (f.u.kb.x84, f.u.kb.x88) }, (3.0, 0.25));
    // B released with the counter at the frequency and no x10 left: enter End.
    unsafe {
        f.mv.kb.specialn_kp.x0 = [1, 7, 1, 2];
        f.mv.kb.specialn_kp.x10 = 0;
    }
    f.input.held_buttons[0] = 0;
    t::ftKb_KpSpecialAirN_IASA(gp);
    assert_eq!(take(), [format!("ms {} 0 0", ftKb_MS_KpSpecialAirNEnd)]);

    // Coll: ground to air carries the animation frame and the hat-specific id.
    f.cur_anim_frame = 5.5;
    f.u.kb.hat.kind = Ft_Kind_GKoops;
    t::ftKb_KpSpecialN_Coll(gp);
    t::ftKb_KpSpecialAirNEnd_Coll(gp);
    assert_eq!(
        take(),
        [
            format!("ms {} c4c5880 5.5", ftKb_MS_GkSpecialAirN),
            "7FC".into(),
            format!("ms {} c4c5080 5.5", ftKb_MS_GkSpecialNEnd),
        ]
    );
}
