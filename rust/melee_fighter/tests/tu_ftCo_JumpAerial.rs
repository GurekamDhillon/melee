//! src/tu/ftCo_JumpAerial.rs vs the C (gen/cref/tu/ftCo_JumpAerial.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_JumpAerial`).
#![allow(static_mut_refs, non_snake_case)]
#[path = "../../melee_core/test_support/fnsym.rs"]
mod fnsym;
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_JumpAerial as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

static mut JO: *mut HSD_JObj = core::ptr::null_mut();

fn is_f(p: &Fighter) -> u32 { (p as *const Fighter == f() as *const Fighter) as u32 }

fn cbid(cb: HSD_GObjEvent) -> u32 {
    match cb {
        None => 0,
        Some(p) if fnsym::same(p as usize, m::ftNs_JumpAerial_Phys_Cb_c as usize) => 1,
        Some(p) if fnsym::same(p as usize, m::ftCo_JumpAerial_Phys_Cb_c as usize) => 2,
        Some(p) if fnsym::same(p as usize, m::ftYs_JumpAerial_Anim_Cb_c as usize) => 3,
        _ => 9,
    }
}

#[test]
fn ftCo_JumpAerial_matches_c() {
    init();
    let fd = Box::leak(Box::new(ftData::zeroed()));
    let sfx = Box::leak(Box::new(FtSFX::zeroed()));
    let x197c = Box::leak(Box::new(HSD_GObj::zeroed())) as *mut HSD_GObj;
    let bone = Box::leak(Box::new(FighterBone::zeroed()));
    let jo = Box::leak(Box::new(HSD_JObj::zeroed()));
    let ys = Box::leak(Box::new(ftYoshiAttributes::zeroed()));
    let v = Box::leak(Box::new(Vec3 { x: 0.0, y: 0.0, z: 0.0 }));
    unsafe {
        JO = jo as *mut _;
        hk::FighterControls_TapJumpOff = Some(|p| { t(1); t(p as u32); sr(0) as i32 });
        hk::ScriptGame_ModJump = Some(|s, fl, bb| { t(2); t(s as u32); t(fl as u32); t(b(bb)); if sr(1) { bb * 2.0 } else { bb } });
        hk::ScriptGame_ModValue = Some(|s, fl, bb| { t(3); t(s as u32); t(fl as u32); t(b(bb)); if sr(2) { bb * 0.5 } else { bb } });
        hk::ScriptGame_CapsMaxJumps = Some(|p| { t(4); t(is_f(p)); (SRV[3] % 6) as i32 });
        hk::Mex_OnDoubleJumpDispatch = Some(|kd, g, vv| { t(5); t(kd as u32); t(is_go(g)); t(vv.is_null() as u32) });
        hk::HSD_JObjSetMtxDirtySub = Some(|j| { t(6); t((j == JO) as u32) });
        hk::ftCo_800C5240 = Some(|_| { t(7); sr(4) });
        hk::ftCo_800D730C = Some(|_, a| { t(8); t(a as u32); sr(5) });
        hk::ft_800D2D0C = Some(|_| { t(9); sr(6) });
        hk::ft_800D2E7C = Some(|_, vv| { t(10); t(b(vv.x)); t(b(vv.y)); t(b(vv.z)) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| { t(11); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32) });
        hk::ft_80088770 = Some(|p| { t(12); t(is_f(p)) });
        hk::ft_80088328 = Some(|_, id, vv, p| { t(13); t(id as u32); t(vv as u32); t(p as u32) });
        hk::ft_PlaySFX = Some(|_, id, vv, p| { t(14); t(id as u32); t(vv as u32); t(p as u32) });
        hk::ftCommon_8007D5D4 = Some(|p| { t(15); p.x1968_jumpsUsed = 1 });
        hk::ftAnim_IsFramesRemaining = Some(|_| { t(16); sr(7) });
        hk::ftCo_FallAerial_Enter = Some(|_| t(17));
        hk::ftCo_SpecialAir_CheckInput = Some(|_| { t(18); sr(8) });
        hk::ftCo_80095328 = Some(|_, p| { t(19); t(p.is_null() as u32); sr(9) });
        hk::ftCo_800D7100 = Some(|_| { t(20); sr(10) });
        hk::ftCo_800C3B10 = Some(|_| { t(21); sr(11) });
        hk::ftCo_80099A58 = Some(|_| { t(22); sr(12) });
        hk::ftCo_AttackAir_CheckItemThrowInput = Some(|_| { t(23); sr(13) });
        hk::ftCo_800D705C = Some(|_| { t(24); sr(14) });
        hk::ftPe_8011BA54 = Some(|_| { t(25); sr(15) });
        hk::ftPe_8011BAD8 = Some(|_| { t(26); sr(16) });
        hk::ftCo_800CEE70 = Some(|_| { t(27); sr(17) });
        hk::ft_80084DB0 = Some(|_| t(28));
        // The C stub's `v * 0.5f - 0.25f` is contracted to one fmadds.
        hk::ftCommon_CalcSelfAccel_DriftFrom = Some(|p, vv| { t(29); t(b(vv)); p.x74_self_accel.x = melee_core::fp::fmadds(vv, 0.5, -0.25) });
        hk::ftCommon_CalcSelfAccel_Drift = Some(|p| { t(30); p.x74_self_accel.x = 0.75 });
        hk::ft_800851D0 = Some(|_| t(31));
        hk::ftCo_80096CC8 = Some(|_, _| { t(32); false });
        hk::ft_80082B1C = Some(|_| t(33));
        hk::ft_800835B0 = Some(|g, a, bb| { t(34); t(is_go(g)); t((fnsym::same(a as usize, m::ext::ftCo_80096CC8 as usize)) as u32); t((fnsym::same(bb as usize, m::ext::ft_80082B1C as usize)) as u32) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_JumpAerial.txt"));
    for op in 0..18u32 {
        for k in 0..2000u32 {
            case(op, k);
            *fd = ftData::zeroed();
            *sfx = FtSFX::zeroed();
            *bone = FighterBone::zeroed();
            *jo = HSD_JObj::zeroed();
            *ys = ftYoshiAttributes::zeroed();
            sfx.x14 = (rnd() % 1000) as i32;
            fd.x4C_sfx = sfx as *mut _;
            let fr = f();
            fr.ft_data = fd as *mut _;
            bone.joint = jo as *mut _;
            fr.parts = bone as *mut _;
            jo.flags = rnd() & (JOBJ_MTX_INDEP_SRT | JOBJ_USER_DEF_MTX | JOBJ_MTX_DIRTY) as u32;
            jo.rotate.y = rg();
            ys.x0 = (1 + rnd() % 20) as i32;
            ys.x4 = rg();
            ys.x8 = rw();
            fr.dat_attrs = ys as *mut _ as *mut core::ffi::c_void;
            fr.x197C = if rnd() & 1 != 0 { x197c } else { core::ptr::null_mut() };
            fr.player_id = (rnd() % 6) as u8;
            let kk = rnd() % 6;
            fr.kind = match kk {
                0 => Ft_Kind_Ness,
                1 => Ft_Kind_Yoshi,
                2 => Ft_Kind_Peach,
                3 => Ft_Kind_Mewtwo,
                _ => rnd() % 33,
            };
            fr.input.lstick[0].x = rg();
            fr.input.lstick[0].y = rg();
            fr.active_timer.lstick.y = (rnd() % 12) as u8;
            fr.input.pressed_buttons = rnd() & 0xF00;
            fr.x68A = (rnd() % 12) as u8;
            fr.x1968_jumpsUsed = (rnd() % 6) as u8;
            fr.set_can_multijump((rnd() & 1) as u8);
            let cdd = cd();
            cdd.tap_jump_threshold = rg();
            cdd.tap_jump_window = (rnd() % 12) as i32;
            cdd.x1C = (rnd() % 12) as i32;
            cdd.x78 = rg();
            fr.co_attrs.air_jump_h_multiplier = rw();
            fr.co_attrs.jump_v_initial_velocity = rw();
            fr.co_attrs.air_jump_v_multiplier = rw();
            unsafe {
                fr.mv.co.jumpaerial.x0 = (rnd() % 25) as i32;
                fr.mv.co.jumpaerial.init_h_vel = rw();
            }
            fr.x6A4_transNOffset.z = rw();
            fr.x74_self_accel.x = rw();
            fr.self_vel.x = rw();
            fr.self_vel.y = rw();
            fr.self_vel.z = rw();
            fr.facing_dir = rg();
            fr.cmd_vars[0] = rnd() & 1;
            v.x = rw();
            v.y = rw();
            v.z = rw();
            let g = go();
            let ret = match op {
                0 => { let a = (1 + rnd() % 20) as i32; m::ft_800CB6EC(f(), a); 0 }
                1 => { let a = rnd() & 1 != 0; m::ft_did_jump(f(), a) as u32 }
                2 => m::ftCo_800CB870(g) as u32,
                3 => m::ftCo_800CB8E0(g) as u32,
                4 => { let a = rnd() & 1 != 0; m::ftCo_JumpAerial_CheckInput(g, a) as u32 }
                5 => { let ms = (rnd() % 400) as i32; let a = rnd() & 1 != 0; m::ftCo_800CBAC4(g, ms, v, a); 0 }
                6 => { m::ftCo_JumpAerial_Enter_Basic(g); 0 }
                7 => { m::ftNs_JumpAerial_Enter(g); 0 }
                8 => { m::ftYs_JumpAerial_Enter(g); 0 }
                9 => { m::ftPe_JumpAerial_Enter(g); 0 }
                10 => { m::ftMt_JumpAerial_Enter(g); 0 }
                11 => { m::ftCo_JumpAerial_Anim(g); 0 }
                12 => { m::ftYs_JumpAerial_Anim_Cb(g); 0 }
                13 => { m::ftCo_JumpAerial_IASA(g); 0 }
                14 => { m::ftCo_JumpAerial_Phys(g); 0 }
                15 => { m::ftNs_JumpAerial_Phys_Cb(g); 0 }
                16 => { m::ftCo_JumpAerial_Phys_Cb(g); 0 }
                _ => { m::ftCo_JumpAerial_Coll(g); 0 }
            };
            let fr = f();
            let mut h = Hs::new();
            h.m(cbid(fr.phys_cb)); h.m(cbid(fr.anim_cb));
            h.m(b(fr.self_vel.x)); h.m(b(fr.self_vel.y)); h.m(b(fr.self_vel.z));
            unsafe { h.m(fr.mv.co.jumpaerial.x0 as u32); h.m(b(fr.mv.co.jumpaerial.init_h_vel)); }
            h.m(b(fr.facing_dir)); h.m(fr.x1968_jumpsUsed as u32); h.m(fr.active_timer.lstick.y as u32); h.m(fr.x2221_b7() as u32);
            h.m(fr.cmd_vars[0]); h.m(b(fr.dmg.armor1)); h.m(b(jo.rotate.y)); h.m(b(fr.x74_self_accel.x));
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 36000);
}
