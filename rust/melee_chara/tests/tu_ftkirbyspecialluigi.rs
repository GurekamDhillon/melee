//! Smoke test for tu::ftkirbyspecialluigi (stateful; no fused sites) through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkirbyspecialluigi as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
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
        melee_fighter::hooks::FTPARTS_GET_BONE_INDEX = Some(|_, p| p + 2);
        t::ext::FTCO_FALL_IASA_INNER = Some(|_| {
            log("fallIASA".into());
            false
        });
        t::ext::LB_8000B1CC = Some(|_, _, out| unsafe { *out = Vec3 { x: 1.0, y: 2.0, z: 3.0 } });
        t::ext::IT_802C01AC = Some(|_, pos, kind, fd| log(format!("fire {} {} {kind} {fd}", unsafe { (*pos).x }, unsafe { (*pos).y })));
        t::ext::EFSYNC_SPAWN = Some(|id, _, va| {
            log(format!("efsync {id:x} {}", va.len()));
            core::ptr::null_mut()
        });
    }

    // The air IASA only runs the shared fall IASA once the move has progressed.
    t::ftKb_LgSpecialAirN_IASA(gp);
    assert!(take().is_empty());
    f.cmd_vars[0] = 1;
    t::ftKb_LgSpecialAirN_IASA(gp);
    assert_eq!(take(), ["fallIASA"]);

    // The accessory callback spawns the fireball once per throw flag and clears it.
    f.facing_dir = -1.0;
    t::fn_800F98F4(gp);
    assert!(take().is_empty());
    unsafe { f.anon_2210.anon_0.set_throw_flags_b0(1) };
    t::fn_800F98F4(gp);
    assert_eq!(take(), [format!("fire 1 2 {} -1", It_Kind_Kirby_LuigiFire), "efsync 4b1 2".to_string()]);
    assert_eq!(unsafe { f.anon_2210.anon_0.throw_flags_b0() }, 0);

    // Re-exports stay wired to the ported callbacks.
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, _, _, _| log(format!("ms {ms} {fl:x}")));
        st::FTANIM_8006EBA4 = Some(|_| {});
    }
    t::ftKb_SpecialNLg_800F9614(gp);
    assert_eq!(take(), [format!("ms {} 0", ftKb_MS_LgSpecialN)]);
    assert!(f.accessory4_cb.is_some());
}
