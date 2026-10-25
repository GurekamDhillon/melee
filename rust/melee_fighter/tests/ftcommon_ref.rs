//! ftcommon.rs vs the C functions of src/melee/ft/ftcommon.c (gen/cref/ftcommon_ref.c,
//! expected values in tests/data/ftcommon.txt; regenerate with
//! `python rust/melee_fighter/gen/cref/run.py ftcommon`). Fill order must match the C.
#![allow(static_mut_refs)]

use melee_fighter::ftcommon_c as fc;
use melee_fighter::hooks;
use melee_fighter::types::{ftCommonData, Fighter, HSD_GObj};

static mut G_FRIC: f32 = 0.0;
static mut G_SFX: i32 = 0;

struct Rng(u32);
impl Rng {
    fn rnd(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    fn rf(&mut self) -> f32 {
        let k = self.rnd() % 16;
        if k == 0 {
            return 0.0;
        }
        if k == 1 {
            return -0.0;
        }
        ((self.rnd() % 2001) as i32 - 1000) as f32 / 7.0
    }
}

#[test]
fn movement_helpers_match_c() {
    unsafe {
        hooks::GROUND_FRICTION_MUL = Some(|_| G_FRIC);
        hooks::PLAY_SFX = Some(|_, id, vol, pan| G_SFX += id + vol + pan);
    }
    let mut cd: Box<ftCommonData> = Box::new(ftCommonData::zeroed());
    let expected = include_str!("data/ftcommon.txt");
    let mut lines = expected.lines();
    let mut checked = 0;
    for op in 0..33u32 {
        for k in 0..200u32 {
            let mut fp: Box<Fighter> = Box::new(Fighter::zeroed());
            *cd = ftCommonData::zeroed();
            unsafe {
                G_SFX = 0;
            }
            let mut r = Rng(0x12345u32.wrapping_add(op.wrapping_mul(7919)).wrapping_add(k.wrapping_mul(104729)));
            fp.gr_vel = r.rf();
            fp.xE4_ground_accel_1 = r.rf();
            fp.self_vel.x = r.rf();
            fp.self_vel.y = r.rf();
            fp.self_vel.z = r.rf();
            fp.x74_self_accel.x = r.rf();
            fp.x74_self_accel.y = r.rf();
            fp.x74_self_accel.z = r.rf();
            fp.xF0_ground_kb_vel = r.rf();
            fp.xF4_ground_attacker_shield_kb_vel = r.rf();
            fp.x8c_kb_vel.x = r.rf();
            fp.x8c_kb_vel.y = r.rf();
            fp.input.lstick[0].x = r.rf();
            fp.input.lstick[0].y = r.rf();
            fp.coll_data.floor.normal.x = r.rf();
            fp.coll_data.floor.normal.y = r.rf();
            fp.ground_or_air = r.rnd() & 1;
            fp.co_attrs.ground_max_horizontal_velocity = r.rf();
            fp.co_attrs.air_drift_max = r.rf();
            fp.co_attrs.aerial_friction = r.rf();
            fp.co_attrs.air_max_horizontal_velocity = r.rf();
            fp.co_attrs.air_drift_stick_mul = r.rf();
            fp.co_attrs.aerial_drift_base = r.rf();
            fp.co_attrs.gravity = r.rf();
            fp.co_attrs.terminal_velocity = r.rf();
            fp.co_attrs.fast_fall_velocity = r.rf();
            fp.set_fall_fast((r.rnd() & 1) as u8);
            fp.active_timer.lstick.y = r.rnd() as u8;
            fp.facing_dir = r.rf();
            cd.max_grounded_kb_on_landing = r.rf();
            cd.aerial_friction_oob = r.rf();
            cd.x88 = r.rf();
            cd.x8C = (r.rnd() % 8) as i32;
            cd.horizontal_stick_deadzone = r.rf();
            cd.x198 = r.rf();
            cd.x19C = r.rf();
            cd.x1A0 = r.rf();
            unsafe {
                G_FRIC = r.rf();
                hooks::P_FT_COMMON_DATA = &*cd;
            }
            let (a, b, c, d) = (r.rf(), r.rf(), r.rf(), r.rf());
            let mut gobj: HSD_GObj = unsafe { core::mem::zeroed() };
            gobj.user_data = &mut *fp as *mut Fighter as *mut core::ffi::c_void;
            let fp = &mut *fp;
            let ret: f32 = match op {
                0 => { fc::ftCommon_CalcGroundAccel_Deaccel(fp, a); 0.0 }
                1 => { fc::ftCommon_CalcGroundAccel_DashRun(fp, a, b, c); 0.0 }
                2 => { fc::ftCommon_CalcGroundAccel_AccelToVel(fp, a, b, c); 0.0 }
                3 => { fc::ftCommon_CalcGroundAccel_AccelToLStickX(fp, a, b, c); 0.0 }
                4 => { fc::ftCommon_SetSelfMovementFromGroundedMovement(&mut gobj); 0.0 }
                5 => { fc::ftCommon_SetSelfMovementFromGroundedMovement_NoFriction(&mut gobj); 0.0 }
                6 => { fc::ftCommon_ClampGroundVel(fp, a); 0.0 }
                7 => { fc::ftCommon_ApplyGroundedKnockbackFriction(fp, a); 0.0 }
                8 => { fc::ftCommon_SetGroundedKnockbackIfLanded(fp); 0.0 }
                9 => fc::ftCommon_SandbagKnockbackDeaccel(a, b),
                10 => { fc::ftCommon_ApplyShieldKnockbackFriction(fp, a); 0.0 }
                11 => { fc::ftCommon_CalcSelfAccel_Deaccel(fp, a); 0.0 }
                12 => { fc::ftCommon_CalcSelfAccel_DeaccelAir(fp); 0.0 }
                13 => fc::ftCommon_CalcSelfAccel_DeaccelQuickAir(fp) as i32 as f32,
                14 => fc::ftCommon_CalcSelfAccel_DeaccelQuick(fp, a) as i32 as f32,
                15 => { fc::ftCommon_CalcSelfAccel_AccelToVelClamped(fp, a, b, c); 0.0 }
                16 => { fc::ftCommon_CalcSelfAccel_AccelToVelClampedFrom(fp, a, b, c, d); 0.0 }
                17 => { fc::ftCommon_CalcSelfAccel_Drift(fp); 0.0 }
                18 => { fc::ftCommon_CalcSelfAccel_DriftFrom(fp, a); 0.0 }
                19 => { fc::ftCommon_CalcSelfAccel_AccelToVel(fp, a, b, c); 0.0 }
                20 => { fc::ftCommon_CalcSelfAccel_DriftSimple(fp, a, b, c); 0.0 }
                21 => { fc::ftCommon_CalcSelfAccel_DriftSimple_NoFriction(fp, a, b, c); 0.0 }
                22 => { fc::ftCommon_ClampSelfVelX(fp, a); 0.0 }
                23 => { fc::ftCommon_ClampAirDrift(fp); 0.0 }
                24 => { fc::ftCommon_Fall(fp, a, b); 0.0 }
                25 => { fc::ftCommon_FallBasic(fp); 0.0 }
                26 => { fc::ftCommon_FallFast(fp); 0.0 }
                27 => { fc::ftCommon_ClampFallSpeed(fp, a); 0.0 }
                28 => { fc::ftCommon_Ascend(fp, a, b); 0.0 }
                29 => fc::ftCommon_CheckFallFast(fp) as i32 as f32,
                30 => { fc::ftCommon_UpdateFacing(fp); 0.0 }
                31 => { fc::ftCommon_8007DA24(fp); 0.0 }
                32 => {
                    let dmg = (r.rnd() % 40) as i32;
                    let ms = (37 + r.rnd() % 5) as i32;
                    fc::ftCommon_CalcHitlag(dmg, ms, a)
                }
                _ => unreachable!(),
            };
            let got = format!(
                "{} {} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {} {} {}",
                op, k, ret.to_bits(), fp.gr_vel.to_bits(), fp.xE4_ground_accel_1.to_bits(),
                fp.self_vel.x.to_bits(), fp.self_vel.y.to_bits(), fp.self_vel.z.to_bits(),
                fp.x74_self_accel.x.to_bits(), fp.x74_self_accel.y.to_bits(), fp.x74_self_accel.z.to_bits(),
                fp.xF0_ground_kb_vel.to_bits(), fp.xF4_ground_attacker_shield_kb_vel.to_bits(),
                fp.x8c_kb_vel.x.to_bits(), fp.x8c_kb_vel.y.to_bits(), fp.facing_dir.to_bits(),
                fp.fall_fast(), fp.active_timer.lstick.y, unsafe { G_SFX }
            );
            assert_eq!(got, lines.next().expect("ran out of C reference lines"), "op {op} case {k}");
            checked += 1;
        }
    }
    assert_eq!(checked, 6600);
}
