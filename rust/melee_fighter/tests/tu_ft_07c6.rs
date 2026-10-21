//! Smoke test for src/tu/ft_07c6.rs (ft_07C6.c): callees stubbed, checks the probe setup and one
//! coin pickup.
#![allow(static_mut_refs)]

use melee_fighter::hooks;
use melee_fighter::tu::ft_07c6::{self as m, ext};
use melee_fighter::types::*;

static mut HEAD: *mut HSD_GObj = core::ptr::null_mut();
static mut KILLED: i32 = 0;

#[test]
fn ft_07c6_smoke() {
    let mut f = Fighter::zeroed();
    let mut x38 = [ftData_x38::zeroed(), ftData_x38::zeroed()];
    x38[0].x10 = 3.0;
    x38[1].x0 = 1;
    x38[1].x10 = 5.0;
    let mut ftd = ftData::zeroed();
    ftd.x38 = x38.as_mut_ptr();
    let mut parts = [FighterBone::zeroed(), FighterBone::zeroed()];
    parts[1].joint = 0x10 as *mut HSD_JObj;
    f.ft_data = &mut ftd;
    f.parts = parts.as_mut_ptr();
    f.player_id = 1;
    f.x34_scale.y = 1.0;
    let mut g = HSD_GObj::zeroed();
    g.user_data = &mut f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut g;

    let mut it = Item::zeroed();
    it.kind = It_Kind_Unk4 as _;
    unsafe { it.xDD4_itemVar.unk4.set_xDEC_b0(1) };
    let mut ig = HSD_GObj::zeroed();
    ig.user_data = &mut it as *mut Item as *mut _;
    it.entity = &mut ig;
    unsafe {
        HEAD = &mut ig;
        hooks::PLINK_HEAD = Some(|l| if l == HSD_GOBJ_PLINK_ITEM { HEAD } else { core::ptr::null_mut() });
        ext::LB_8000B1CC = Some(|j, p, out| {
            (*out).x = j as usize as f32 + (*p).x;
            (*out).y = 7.0;
            (*out).z = 9.0;
        });
        ext::GM_8016B0B4 = Some(|| true);
        ext::GM_801A4BB8 = Some(|| 1);
        ext::FTCOLL_8007B868 = Some(|_| 0);
        ext::LBCOLL_80007B78 = Some(|_, b, _, _| (*b).x0 == 5.0);
        ext::IT_802E5EF4 = Some(|_| 10);
        ext::LBAUDIOAX_80023870 = Some(|_, _, _, _| {});
        ext::EFSYNC_SPAWN = Some(|id, _, _, s| assert!(id == 0x432 && *s == 1.0));
        ext::ITEM_8026A8EC = Some(|_| KILLED += 1);
    }
    m::ft_8007c630(gp);
    assert_eq!(f.x1614[0].x0, 3.0);
    assert_eq!(f.x1614[1].x4, 0x10 as *mut HSD_JObj);
    assert_eq!((f.x1614[1].x8.x, f.x1614[1].x8.y, f.x1614[1].x8.z), (16.0, 7.0, 0.0));
    assert_eq!(f.x1614[1].x14.x, 16.0);
    x38[1].x4.x = 1.0;
    m::ft_8007c6dc(gp);
    assert_eq!((f.x1614[1].x14.x, f.x1614[1].x8.x), (16.0, 17.0));

    let c0 = melee_fighter::player::get_coins(1);
    m::ft_8007c77c(gp);
    assert_eq!(f.x2229_b4(), 0);
    assert_eq!(melee_fighter::player::get_coins(1), c0 + 10);
    assert_eq!(unsafe { KILLED }, 1);
    // parity mismatch: flag set, nothing collected
    f.player_id = 0;
    m::ft_8007c77c(gp);
    assert_eq!(f.x2229_b4(), 1);
    assert_eq!(unsafe { KILLED }, 1);
}
