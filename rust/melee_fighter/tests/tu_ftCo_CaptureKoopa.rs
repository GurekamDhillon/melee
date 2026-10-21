//! Smoke test for tu::ftCo_CaptureKoopa: the mash, the YRotN wobble and the entry.
#![allow(static_mut_refs)]
use melee_fighter::tu::ftCo_CaptureKoopa::{self as t, ext::hook as h};
use melee_fighter::types::*;
use std::vec::Vec;

static mut LOG: Vec<String> = Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}

#[test]
fn smoke() {
    let f: &mut Fighter = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    let v: &mut Fighter = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    let g: &mut HSD_GObj = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    let vg: &mut HSD_GObj = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    let j: &mut HSD_JObj = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    let pb: &mut [FighterBone; 2] = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    j.flags = JOBJ_MTX_DIRTY as u32;
    pb[1].joint = j;
    g.user_data = (f as *mut Fighter).cast();
    vg.user_data = (v as *mut Fighter).cast();
    let (gp, vp): (*mut HSD_GObj, *mut HSD_GObj) = (g, vg);
    let jp = pb[1].joint;
    f.parts = pb.as_mut_ptr();
    f.victim_gobj = vp;
    f.grab_timer = 10.0;
    unsafe {
        h::ftKp_SpecialS_80132DC0 = Some(|_| 50.0);
        h::ftKp_SpecialS_80132DD0 = Some(|_| 1.5);
        h::ftKp_SpecialS_80132DE0 = Some(|_| 0.25);
        h::ftKp_SpecialS_80132DF0 = Some(|_| 6.0);
        h::ftKp_SpecialS_80132E00 = Some(|_| 2.0);
        h::ftKp_SpecialS_80132E10 = Some(|_| 0.5);
        h::ftKp_SpecialS_80132E20 = Some(|_| 3.0);
        h::ftCommon_GrabMash = Some(|_, v| {
            log(format!("mash {v}"));
            true
        });
        h::ftParts_GetBoneIndex = Some(|_, _| 1);
        h::ftAnim_SetAnimRate = Some(|_, r| log(format!("rate {r}")));
        h::HSD_JObjSetMtxDirtySub = Some(|_| log("dirty".into()));
        h::ftCo_ReleaseItemAndVictim = Some(|_| log("release".into()));
        h::ftCommon_InitGrab = Some(|_, a, t| log(format!("grab {a} {t}")));
        h::ftCo_800DB368 = Some(|_, _| log("db368".into()));
        h::ftCommon_8007D5D4 = Some(|_| log("d5d4".into()));
        h::ftCommon_8007E2FC = Some(|_| log("e2fc".into()));
        h::ftCo_800BC9C8 = Some(|_| log("dmg".into()));
    }
    t::ftCo_800BC458(gp);
    let f = unsafe { &mut *((*gp).user_data as *mut Fighter) };
    assert_eq!(f.grab_timer, 8.5);
    assert_eq!(unsafe { f.mv.co.capturekoopa.x0 }, 1);
    // wobble: vel.y != 0, x1A50 = 2 -> z += 2*xC; y would leave the bound, so it stays
    f.mv.ca.specialhi.vel.y = 1.0;
    f.mv.co.capturekoopa.xC = 0.5;
    f.mv.co.capturekoopa.x10 = 3.0;
    f.mv.co.capturekoopa.x8 = 1.0;
    f.x1A50 = 2;
    f.x1A51 = -9;
    t::ftCo_800BC4A8(gp);
    let j = unsafe { &*jp };
    assert_eq!(j.translate.z, 1.0);
    assert_eq!(j.translate.y, 0.0);
    assert_eq!(unsafe { f.mv.co.capturekoopa.x8 }, 6.0);
    unsafe { LOG.clear() };
    t::ftCo_800BC7E0(gp, vp);
    assert_eq!(f.victim_gobj, vp);
    assert_eq!(unsafe { (f.mv.co.capturekoopa.xC, f.mv.co.capturekoopa.x10) }, (0.5, 3.0));
    assert_eq!(unsafe { LOG.clone() }, ["release", "grab 0 50", "db368", "dmg", "d5d4", "e2fc"]);
}
