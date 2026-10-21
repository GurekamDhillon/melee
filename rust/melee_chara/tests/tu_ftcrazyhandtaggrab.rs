//! Smoke test of tu::ftcrazyhandtaggrab (hooks record the calls). One #[test]: hooks are globals.
#![allow(static_mut_refs)]
mod ch_common;
use ch_common::*;
use melee_chara::tu::ftcrazyhandtaggrab as t;
use melee_types::*;

#[test]
fn smoke() {
    let mut r = Rig::new();
    let gp = r.gp();
    unsafe {
        t::ext::IT_802F046C = Some(|_| log("it"));
        t::ext::LBAUDIOAX_800236B8 = Some(|id| log(format!("stop{id}")));
        t::ext::LBAUDIOAX_8002438C = Some(|id| log(format!("snd{id:x}")));
        t::ext::FTCH_GRABUNK1_8015B800 = Some(|_| log("victim"));
        melee_fighter::hooks::LBAUDIOAX_800233EC = Some(|_| 0);
        melee_fighter::hooks::LBAUDIOAX_80023130 = Some(|_| 0);
        melee_fighter::hooks::LBAUDIOAX_80023220 = Some(|_| 0);
        melee_fighter::hooks::LBAUDIOAX_800237A8 = Some(|_, _, _| 0);
        melee_fighter::hooks::LBAUDIOAX_80024DC4 = Some(|_| ());
        melee_fighter::hooks::LBAUDIOAX_80023870 = Some(|_, _, _, _| 0);
        melee_fighter::hooks::LBAUDIOAX_80024B94 = Some(|_, _| 0);
        melee_fighter::hooks::PLAY_SFX = Some(|_, id, v, p| log(format!("sfx{id:x} {v} {p}")));
    }
    t::ftCh_TagGrab_Coll(gp);
    // Phys.
    r.da.x12C_pos = Vec2 { x: 1.0, y: 0.5 };
    r.da.x134_pos = Vec2 { x: 2.0, y: 0.25 };
    r.f.self_vel.y = 1.0;
    r.f.self_vel.z = 1.0;
    t::ftCh_TagGrab_Phys(gp);
    assert_eq!((r.f.self_vel.y, r.f.self_vel.z), (1.5, 1.25));
    // Anim counters: x124 = 1 first sound, x128 = 1 second sound.
    r.da.x124 = 1;
    r.da.x128 = 1;
    r.f.cmd_vars[0] = 1;
    unsafe { r.f.mv.ch.unk0.x8 = 10 };
    unsafe { FRAMES = 1 };
    t::ftCh_TagGrab_Anim(gp); // x60 -> 1 <= 1: snd 81652
    t::ftCh_TagSqueeze_Anim(gp); // cmd_vars cleared: no sound
    r.f.cmd_vars[0] = 1;
    t::ftCh_TagGrab_Anim(gp); // x60 -> 2 > 1: x64 -> 1 <= 1: snd 81653
    r.f.cmd_vars[0] = 1;
    t::ftCh_TagGrab_Anim(gp); // both past: silent
    assert_eq!(take(), "snd81652,snd81653");
    unsafe {
        assert_eq!((r.f.mv.ch.ch_dmg.x60, r.f.mv.ch.ch_dmg.x64), (3, 2));
        assert_eq!(r.f.mv.ch.unk0.x8, 6);
    }
    // x8 > 0 and animation done -> 0x182.
    unsafe { FRAMES = 0 };
    t::ftCh_TagGrab_Anim(gp);
    assert_eq!(take(), "cms 0x182 0 0 1 0,8006EBA4");
    // Init: runs the inline and enters 0x181.
    r.da.x13C_pos = Vec2 { x: 7.9, y: 0.0 };
    unsafe { r.f.mv.ch.unk0.x20 = 1 };
    t::ftCh_GrabUnk1_8015ADD0(gp);
    let s = take();
    assert!(s.starts_with("it,it,it,it,stop0,stop0,stop0,victim,cms 0x181 0 0 1 0,8006EBA4"), "{s}");
    assert!(s.ends_with("sfx4e219 127 64"), "{s}");
    unsafe {
        assert_eq!(r.f.mv.ch.unk0.x8, 7);
        assert_eq!((r.f.mv.ch.ch_dmg.x60, r.f.mv.ch.ch_dmg.x64), (0, 0));
    }
    assert_eq!((r.f.self_vel.y, r.f.self_vel.z), (1.0, 2.0));
}
