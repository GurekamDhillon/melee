//! Smoke test of tu::ftmewtwospeciallw (Disable): Anim / RemoveDisable through recording hooks,
//! and the spawn position of CreateDisable (one fused site) against an independent
//! double-precision fused multiply-add.
#![allow(static_mut_refs)]
use melee_chara::stubs as st;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> String {
    unsafe { std::mem::take(&mut LOG).join(",") }
}
static mut FRAMES_LEFT: i32 = 0;

use melee_chara::tu::ftmewtwospeciallw as t;

#[test]
fn smoke() {
    let mut attrs: Box<ftMewtwoAttributes> = Box::new(unsafe { core::mem::zeroed() });
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    f.dat_attrs = &mut *attrs as *mut _ as *mut _;
    let mut parts: std::vec::Vec<FighterBone> = (0..64).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.parts = parts.as_mut_ptr();
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    let mut disable: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let dp: *mut HSD_GObj = &mut *disable;
    unsafe {
        st::FTANIM_ISFRAMESREMAINING = Some(|_| FRAMES_LEFT);
        st::FT_8008A2BC = Some(|_| log("wait".into()));
        st::FTCO_FALL_ENTER = Some(|_| log("fall".into()));
        t::ext::ITMEWTWODISABLE_LOGIC67_DESTROY = Some(|_| log("destroy".into()));
        t::ext::LB_8000B1CC = Some(|_, _, out| *out = Vec3 { x: 10.0, y: 20.0, z: 1.0 });
        t::ext::ITMEWTWODISABLE_LOGIC67_SPAWNMEWTWODISABLE = Some(|_, pos, facing| {
            log(format!("spawn {:08x} {:08x} {facing}", (*pos).x.to_bits(), (*pos).y.to_bits()));
            core::ptr::null_mut()
        });
    }

    // Anim keeps the item while frames remain, destroys it and ends otherwise.
    f.u.mt.x222C_disableGObj = dp;
    unsafe { FRAMES_LEFT = 1 };
    t::ftMt_SpecialLw_Anim(gp);
    assert_eq!(take(), "");
    unsafe { FRAMES_LEFT = 0 };
    t::ftMt_SpecialLw_Anim(gp);
    assert_eq!(take(), "destroy,wait");
    assert!(unsafe { f.u.mt.x222C_disableGObj.is_null() });
    f.u.mt.x222C_disableGObj = dp;
    t::ftMt_SpecialAirLw_Anim(gp);
    assert_eq!(take(), "destroy,fall");
    t::ftMt_SpecialLw_RemoveDisable(gp);
    assert_eq!(take(), "");
    f.u.mt.x222C_disableGObj = dp;
    t::ftMt_SpecialLw_RemoveDisable(gp);
    assert_eq!(take(), "destroy");
    f.u.mt.x222C_disableGObj = dp;
    t::ftMt_SpecialLw_ClearDisableGObj(gp);
    assert!(unsafe { f.u.mt.x222C_disableGObj.is_null() });

    // CreateDisable: nothing until the script pulses cmd_vars[0]; then the position is
    // fmadds(offset_x, facing, x), y + offset_y, and the item pointer is stored.
    t::ftMt_SpecialLw_CreateDisable(gp);
    assert_eq!(take(), "");
    for (ox, facing) in [(7.3f32, 1.0f32), (-3.7, -1.0), (0.1, 1.0), (123.456, -1.0)] {
        attrs.x80_MEWTWO_DISABLE_OFFSET_X = ox;
        attrs.x84_MEWTWO_DISABLE_OFFSET_Y = 2.5;
        f.facing_dir = facing;
        f.cmd_vars[0] = 1;
        t::ftMt_SpecialLw_CreateDisable(gp);
        let x = ((ox as f64) * (facing as f64) + 10.0f64) as f32;
        assert_eq!(take(), format!("spawn {:08x} {:08x} {facing}", x.to_bits(), (20.0f32 + 2.5).to_bits()));
        assert_eq!(f.cmd_vars[0], 0);
    }
}
