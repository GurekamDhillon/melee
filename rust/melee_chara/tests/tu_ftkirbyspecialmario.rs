//! Smoke test for tu::ftkirbyspecialmario (stateful; no fused sites) through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkirbyspecialmario as t;
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
    unsafe { smoke_inner() }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn smoke_inner() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut bones: std::vec::Vec<FighterBone> = (0..64).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.parts = bones.as_mut_ptr();
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        melee_fighter::hooks::FTPARTS_GET_BONE_INDEX = Some(|_, p| p);
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, af, _, _, _| log(format!("ms {ms} {fl:x} {af}")));
        st::FTANIM_8006EBA4 = Some(|_| {});
        st::FT_80082708 = Some(|_| 0);
        st::FT_80081D0C = Some(|_| 1);
        st::FTCOMMON_8007D7FC = Some(|_| log("7FC".into()));
        t::ext::FTCO_FALL_IASA_INNER = Some(|_| {
            log("fallIASA".into());
            false
        });
        t::ext::LB_8000B1CC = Some(|_, _, out| unsafe { *out = Vec3 { x: 1.0, y: 2.0, z: 3.0 } });
        t::ext::HSD_RANDI = Some(|n| unsafe {
            log(format!("randi {n}"));
            RANDI.remove(0)
        });
        t::ext::IT_8029B6F8 = Some(|_, pos, kind, fd| log(format!("mfire {} {kind} {fd}", unsafe { (*pos).x })));
        t::ext::ITDRMARIOPILL_SPAWN = Some(|_, _, pick, kind, fd| log(format!("pill {pick} {kind} {fd}")));
        t::ext::EFSYNC_SPAWN = Some(|id, _, va| {
            log(format!("efsync {id:x} {}", va.len()));
            core::ptr::null_mut()
        });
    }
    f.facing_dir = 1.0;

    // Enter picks the Dr. Mario state for any hat but Mario's.
    f.u.kb.hat.kind = Ft_Kind_Mario;
    t::ftKb_SpecialN_800F9110(gp);
    f.u.kb.hat.kind = Ft_Kind_DrMario;
    t::ftKb_SpecialN_800F9110(gp);
    t::ftKb_SpecialNMr_800F93CC(gp);
    f.u.kb.hat.kind = Ft_Kind_Mario;
    t::ftKb_SpecialNMr_800F93CC(gp);
    assert_eq!(
        take(),
        [
            format!("ms {} 0 0", ftKb_MS_MrSpecialN),
            "ms 512 0 0".to_string(),
            "ms 513 0 0".to_string(),
            format!("ms {} 0 0", ftKb_MS_MrSpecialAirN),
        ]
    );
    assert!(f.accessory4_cb.is_some());

    // Coll: ground to air keeps the frame and uses the per-hat id.
    f.cur_anim_frame = 2.5;
    t::ftKb_MrSpecialN_Coll(gp); // ft_80082708 -> 0 == Ground
    f.u.kb.hat.kind = Ft_Kind_DrMario;
    t::ftKb_MrSpecialAirN_Coll(gp);
    assert_eq!(take(), ["ms 400 5000 2.5".to_string(), "7FC".to_string(), "ms 512 5000 2.5".to_string()]);

    // Air IASA.
    f.cmd_vars[0] = 1;
    t::ftKb_MrSpecialAirN_IASA(gp);
    assert_eq!(take(), ["fallIASA"]);

    // Accessory callback: Kirby himself does nothing; Mario shoots a fireball; Dr. Mario a pill
    // that avoids the last two colours.
    f.u.kb.hat.kind = Ft_Kind_Kirby;
    unsafe { f.x2210.x0.set_throw_flags_b0(1) };
    t::fn_800F9260(gp);
    assert!(take().is_empty());
    f.u.kb.hat.kind = Ft_Kind_Mario;
    t::fn_800F9260(gp);
    assert_eq!(take(), [format!("mfire 1 {} 1", It_Kind_Kirby_MarioFire), "efsync 49f 2".to_string()]);
    f.u.kb.hat.kind = Ft_Kind_DrMario;
    f.u.kb.x68 = 2;
    f.u.kb.x6C = 5;
    unsafe {
        f.x2210.x0.set_throw_flags_b0(1);
        RANDI = vec![4];
    }
    t::fn_800F9260(gp);
    // candidates = 0,1,3,4,6,7,8: index 4 is 6.
    assert_eq!(take(), ["randi 7".to_string(), format!("pill 6 {} 1", It_Kind_Kirby_DrMarioVitamin)]);
    assert_eq!((f.u.kb.x68, f.u.kb.x6C), (6, 2));
}
