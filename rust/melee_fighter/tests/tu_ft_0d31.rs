//! Smoke test for src/tu/ft_0d31.rs (stateful KO code): blast-zone check inside bounds and the
//! early-outs leave the fighter untouched; the Dead*_Cam callbacks are no-ops.
#![allow(static_mut_refs)]
use melee_fighter::hooks;
use melee_fighter::tu::ft_0d31 as t;
use melee_fighter::types::*;

fn zeroed<T>() -> Box<T> { unsafe { Box::new(core::mem::zeroed()) } }
fn right() -> f32 { 200.0 }
fn left() -> f32 { -200.0 }
fn top() -> f32 { 180.0 }
fn bottom() -> f32 { -140.0 }
fn geno(_f: &mut Fighter) -> i32 { 0 }

#[test]
fn blast_zone_inside_is_false() {
    let mut f: Box<Fighter> = zeroed();
    let mut g: Box<HSD_GObj> = zeroed();
    let cd: Box<ftCommonData> = zeroed();
    g.user_data = &mut *f as *mut Fighter as *mut _;
    unsafe {
        hooks::P_FT_COMMON_DATA = &*cd;
        t::ext::H_Stage_GetBlastZoneRightOffset = Some(right);
        t::ext::H_Stage_GetBlastZoneLeftOffset = Some(left);
        t::ext::H_Stage_GetBlastZoneTopOffset = Some(top);
        t::ext::H_Stage_GetBlastZoneBottomOffset = Some(bottom);
        t::ext::H_GenoFly_Fighter = Some(geno);
    }
    let gp = &mut *g as *mut HSD_GObj;
    for (x, y) in [(0.0f32, 0.0f32), (199.0, 179.0), (-199.0, -139.0), (200.0, 180.0)] {
        f.cur_pos.x = x;
        f.cur_pos.y = y;
        assert!(!t::ftCo_800D3158(gp), "({x},{y})");
        assert_eq!(f.motion_id, 0);
    }
    t::ftCo_DeadLeft_Cam(gp);
    t::ftCo_DeadRight_Cam(gp);
    t::ftCo_DeadDown_Cam(gp);
    t::ftCo_DeadUp_Cam(gp);
}
