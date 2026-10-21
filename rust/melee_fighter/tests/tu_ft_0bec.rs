//! Smoke test for src/tu/ft_0bec.rs, src/tu/ft_0bef.rs and src/tu/ft_0bf0.rs (the dead / sleep
//! poses): callees are recording stubs; checks motion ids, flags and the items handed out.
#![allow(static_mut_refs)]

use melee_fighter::hooks;
use melee_fighter::tu::{ft_0bec, ft_0bef, ft_0bf0};
use melee_fighter::types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
static mut ITEM: G = core::ptr::null_mut();
type G = *mut HSD_GObj;

fn take() -> std::vec::Vec<String> {
    unsafe { core::mem::take(&mut LOG) }
}

#[test]
fn dead_poses_smoke() {
    unsafe {
        hooks::FTPARTS_GET_BONE_INDEX = Some(|_, p| p + 100);
        hooks::GENO_DEFINE_TEMPLATE_SKELETON = Some(|_| 0);
        ft_0bec::ext::FIGHTER_CHANGE_MOTION_STATE = Some(|_, ms, _, _, _, _, _| log(format!("CMS {ms}")));
        ft_0bef::ext::FIGHTER_CHANGE_MOTION_STATE = Some(|_, ms, _, _, _, _, _| log(format!("CMS {ms}")));
        ft_0bf0::ext::FIGHTER_CHANGE_MOTION_STATE = Some(|_, ms, _, _, _, _, _| log(format!("CMS {ms}")));
        ft_0bec::ext::IT_802AE994 = Some(|_, b, k| { log(format!("BLASTER {b} {k}")); ITEM });
        ft_0bf0::ext::IT_802AE994 = Some(|_, b, k| { log(format!("BLASTER {b} {k}")); ITEM });
        ft_0bec::ext::IT_802AD590 = Some(|_, b| { log(format!("BAT {b}")); ITEM });
        ft_0bf0::ext::IT_802AD590 = Some(|_, b| { log(format!("BAT {b}")); ITEM });
        ft_0bec::ext::IT_8026BAE8 = Some(|_, s| log(format!("SCALE {:08X}", s.to_bits())));
        ft_0bec::ext::FTMR_SPECIALN_VITAMINRANDOM = Some(|_| 3);
        ft_0bf0::ext::FTMR_SPECIALN_VITAMINRANDOM = Some(|_| 3);
        ft_0bec::ext::ITDRMARIOPILL_802C09C4 = Some(|_, _, c, k, a, p, f| { log(format!("PILL {c} {k} {a} {p} {f}")); ITEM });
        ft_0bf0::ext::ITDRMARIOPILL_802C09C4 = Some(|_, _, c, k, a, p, f| { log(format!("PILL {c} {k} {a} {p} {f}")); ITEM });
        ft_0bf0::ext::MEX_ON_INTRO_L_DISPATCH = Some(|k, _, _| log(format!("MEX {k}")));
    }
    let mut item: Box<HSD_GObj> = Box::new(HSD_GObj::zeroed());
    unsafe { ITEM = &mut *item };
    let mut fa: Box<ftFox_DatAttrs> = Box::new(ftFox_DatAttrs::zeroed());
    fa.x20_FOX_BLASTER_GUN_ITKIND = 55;
    let mut fp: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(HSD_GObj::zeroed());
    g.user_data = &mut *fp as *mut Fighter as *mut core::ffi::c_void;
    let gp: G = &mut *g;
    fp.dat_attrs = &mut *fa as *mut ftFox_DatAttrs as *mut core::ffi::c_void;
    fp.x34_scale.y = 2.0;
    fp.co_attrs.model_scaling = 1.5;
    fp.facing_dir = -1.0;

    fp.kind = Ft_Kind_Fox;
    ft_0bec::ftCo_800BECB0(gp);
    assert_eq!(take(), ["CMS 0", "BLASTER 149 55", format!("SCALE {:08X}", 3.0f32.to_bits()).as_str()]);
    assert_eq!((fp.x2219_b2(), fp.x2219_b1(), fp.x2228_b1()), (1, 1, 1));
    assert_eq!(fp.item_gobj, unsafe { ITEM });

    fp.kind = Ft_Kind_DrMario;
    ft_0bec::ftCo_800BED88(gp);
    let s = format!("SCALE {:08X}", (0.71428f32 * 3.0).to_bits());
    assert_eq!(take(), [format!("CMS {}", ftCo_MS_DeadRight).as_str(), "PILL 3 49 1 0 -1", s.as_str(), "PILL 3 49 0 0 -1", s.as_str()]);
    assert_eq!(fp.x1984_heldItemSpec, unsafe { ITEM });

    fp.kind = Ft_Kind_Ness;
    ft_0bec::ftCo_800BEF04(gp);
    assert_eq!(take(), [format!("CMS {}", ftCo_MS_DeadUpStarIce).as_str(), "BAT 149", format!("SCALE {:08X}", (0.8f32 * 3.0).to_bits()).as_str()]);

    fp.kind = Ft_Kind_Mario;
    ft_0bec::ftCo_800BECB0(gp);
    assert_eq!(take(), ["CMS 0"]);
    assert!(fp.item_gobj.is_null());

    let mut fp2: Box<Fighter> = Box::new(Fighter::zeroed());
    g.user_data = &mut *fp2 as *mut Fighter as *mut core::ffi::c_void;
    ft_0bef::ftCo_800BEFD4(gp);
    assert_eq!(take(), [format!("CMS {}", ftCo_MS_DeadUpFallIce)]);
    assert_eq!((fp2.x2219_b2(), fp2.x2219_b1(), fp2.x2228_b1()), (1, 1, 0));

    g.user_data = &mut *fp as *mut Fighter as *mut core::ffi::c_void;
    fp.kind = Ft_Kind_Mario;
    ft_0bf0::ftCo_800BF034(gp);
    assert_eq!(take(), [format!("CMS {}", ftCo_MS_DeadUpFallHitCameraIce), format!("MEX {}", Ft_Kind_Mario)]);
    fp.kind = Ft_Kind_Ness;
    ft_0bf0::ftCo_800BF034(gp);
    assert_eq!(take(), [format!("CMS {}", ftCo_MS_DeadUpFallHitCameraIce), "BAT 149".to_string()]);
    fp.kind = Ft_Kind_DrMario;
    ft_0bf0::ftCo_800BF108(gp);
    assert_eq!(take(), [format!("CMS {}", ftCo_MS_Sleep), "PILL 3 49 2 149 -1".to_string()]);

    fp.motion_id = ftCo_MS_Sleep as i32;
    assert!(ft_0bf0::ftCo_800BF228(gp));
    fp.motion_id = ftCo_MS_Wait as i32;
    assert!(!ft_0bf0::ftCo_800BF228(gp));
    assert!(!ft_0bf0::ftCo_800BF228(core::ptr::null_mut()));
}
