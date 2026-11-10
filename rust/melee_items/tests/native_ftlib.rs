//! src/tu/ftlib.rs leaf accessors: each reads/writes the field the C body (src/melee/ft/ftlib.c) names.
#![allow(non_snake_case)]
use melee_items::tu::ftlib::*;
use melee_types::*;

#[test]
fn ftlib_leaf_accessors() {
    unsafe {
        let mut f: Box<Fighter> = Box::new(core::mem::zeroed());
        let mut parts: [FighterBone; 4] = core::mem::zeroed();
        let mut jo: [HSD_JObj; 4] = core::mem::zeroed();
        let mut other: HSD_GObj = core::mem::zeroed();
        let mut item: HSD_GObj = core::mem::zeroed();
        for i in 0..4 {
            parts[i].joint = &mut jo[i];
        }
        f.parts = parts.as_mut_ptr();
        f.cur_pos = Vec { x: 1.0, y: 2.0, z: 3.0 };
        f.pos_delta = Vec { x: 4.0, y: 5.0, z: 6.0 };
        f.facing_dir = -1.0;
        f.ground_or_air = 1 as _;
        f.player_idx = 3 as _;
        f.team = 2 as _;
        f.x34_scale.y = 1.25;
        f.input.lstick[0].x = 0.5;
        f.input.lstick[0].y = -0.75;
        f.item_gobj = &mut item;
        f.x1984_heldItemSpec = &mut other;
        let mut g: HSD_GObj = core::mem::zeroed();
        g.user_data = &mut *f as *mut Fighter as *mut _;
        g.classifier = HSD_GOBJ_CLASS_FIGHTER as _;
        let gp = &mut g as *mut HSD_GObj;
        assert!(ftLib_IsFighter(gp));
        assert!(!ftLib_IsFighter(core::ptr::null_mut()));
        g.classifier = 0;
        assert!(!ftLib_IsFighter(gp));
        let mut v: Vec = core::mem::zeroed();
        ftLib_GetPos(gp, &mut v);
        assert_eq!((v.x, v.y, v.z), (1.0, 2.0, 3.0));
        ftLib_GetPosDelta(gp, &mut v);
        assert_eq!((v.x, v.y, v.z), (4.0, 5.0, 6.0));
        assert_eq!(ftLib_GetFacingDir(gp), -1.0);
        assert_eq!(ftLib_GetPartJObj(gp, 2), &mut jo[2] as *mut HSD_JObj);
        assert_eq!(ftLib_GetGroundAir(gp), 1);
        assert_eq!(ftLib_GetPlayerIndex(gp), 3);
        assert_eq!(ftLib_GetTeam(gp), 2);
        assert_eq!(ftLib_GetScale(gp), 1.25);
        let (mut x, mut y) = (0.0, 0.0);
        ftLib_GetLStick(gp, &mut x, &mut y);
        assert_eq!((x, y), (0.5, -0.75));
        assert!(ftLib_IsHoldingItem(gp, &mut item));
        assert!(!ftLib_IsHoldingItem(gp, &mut other));
        assert_eq!(ftLib_GetHeldSpecialItem(gp), &mut other as *mut HSD_GObj);
    }
}
