//! Smoke test for tu::ftluigispecialn: the IASA callbacks and the fireball spawn through
//! recording hooks (no float arithmetic, no fused sites).
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftluigispecialn as t;
use melee_types::*;

/// The hooks are process globals: the tests of this file run one at a time.
static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}

#[test]
fn iasa_and_fire_spawn() {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut parts: std::vec::Vec<FighterBone> = (0..80).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.parts = parts.as_mut_ptr();
    f.facing_dir = -1.0;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        melee_fighter::hooks::FTPARTS_GET_BONE_INDEX = Some(|_, i| i);
        st::FTCO_WAIT_IASA = Some(|_| log("wait".into()));
        t::ext::FTCO_FALL_IASA_INNER = Some(|_| log("fall".into()));
        t::ext::LB_8000B1CC = Some(|_, _, out| *out = Vec3 { x: 4.0, y: 5.0, z: 6.0 });
        t::ext::IT_802C01AC = Some(|_, pos, kind, facing| log(format!("fire {} {} {kind} {facing}", (*pos).x, (*pos).z)));
        t::ext::EFSYNC_SPAWN = Some(|id, _, args| {
            log(format!("ef {id} {}", args.len()));
            core::ptr::null_mut()
        });
    }
    f.cmd_vars[0] = 0;
    t::ftLg_SpecialN_IASA(gp);
    t::ftLg_SpecialAirN_IASA(gp);
    f.cmd_vars[0] = 1;
    t::ftLg_SpecialN_IASA(gp);
    t::ftLg_SpecialAirN_IASA(gp);
    // No spawn flag: nothing; flag set: one fireball + effect, and the flag is consumed.
    t::ftLg_SpecialN_FireSpawn(gp);
    unsafe { f.anon_2210.anon_0.set_throw_flags_b0(1) };
    t::ftLg_SpecialN_FireSpawn(gp);
    assert_eq!(unsafe { f.anon_2210.anon_0.throw_flags_b0() }, 0);
    t::ftLg_SpecialN_FireSpawn(gp);
    assert_eq!(unsafe { LOG.join(",") }, "wait,fall,fire 4 6 105 -1,ef 1287 2");
}
