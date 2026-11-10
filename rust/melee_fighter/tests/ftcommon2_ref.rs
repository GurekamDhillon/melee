//! ftcommon2.rs vs the C functions of src/melee/ft/ftcommon.c (gen/cref/ftcommon2_ref.c, built
//! with the shipping build's contraction + gwtool lowering; expected values in
//! tests/data/ftcommon2.txt; regenerate with `python rust/melee_fighter/gen/cref/run.py ftcommon2`).
//! Fill order and the stubbed callees (trace word `TR`) must match the C exactly.
#![allow(static_mut_refs)]

use melee_fighter::ftcommon2 as f;
use melee_fighter::hooks;
use melee_fighter::types::{ftCommonData, Fighter, Fighter_ShakeTable_t, HSD_GObj, Vec2, Vec3, Ft_Kind_Sandbag};

static mut TR: u32 = 0;
static mut G_FRIC: f32 = 0.0;

fn t(v: u32) {
    unsafe { TR = TR.wrapping_mul(31).wrapping_add(v) }
}
fn b(x: f32) -> u32 {
    x.to_bits()
}

struct Rng(u32);
impl Rng {
    fn rnd(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    fn rf(&mut self) -> f32 {
        match self.rnd() % 16 {
            0 => 0.0,
            1 => -0.0,
            2 => 1.0,
            3 => -1.0,
            _ => ((self.rnd() % 2001) as i32 - 1000) as f32 / 7.0,
        }
    }
    fn tri(&mut self) -> i8 {
        ((self.rnd() % 3) as i32 - 1) as i8
    }
}

fn install_hooks() {
    unsafe {
        hooks::GROUND_FRICTION_MUL = Some(|_| G_FRIC);
        hooks::GENO_GROUND_EDGE = Some(|_, l| t(0x100u32.wrapping_add(l as u32)));
        hooks::PL_8003FC44 = Some(|s, u| t(0x200 + s as u32 * 2 + u as u32));
        hooks::PL_800402D0 = Some(|s, u, r| t(0x300 + s as u32 * 4 + u as u32 * 2 + r as u32));
        hooks::PL_80040460 = Some(|s, u| t(0x400 + s as u32 * 2 + u as u32));
        hooks::PLAYER_8003544C = Some(|s, u| (s as u32).wrapping_add(u as u32) & 1 != 0);
        hooks::PLAYER_UPDATE_JOYSTICK_COUNT = Some(|s, u| t(0x500 + s as u32 * 2 + u as u32));
        hooks::LB_RUMBLE_START = Some(|p, id, a, bb| {
            t(0x600 + p as u32);
            t(id as u32);
            t(a as u32);
            t(bb as u32);
        });
        hooks::HSD_PAD_RUMBLE_REMOVE_ID = Some(|p, id| {
            t(0x700 + p as u32);
            t(id as u32);
        });
        hooks::LB_RUMBLE_STOP = Some(|p| t(0x800 + p as u32));
        hooks::UN_803224DC = Some(|s, x, kb| {
            t(0x900u32.wrapping_add(s as u32));
            t(b(x));
            t(b(kb));
            s & 1 != 0
        });
        hooks::IT_80294E78 = Some(|_, x| {
            t(0xA00);
            t(b(x));
        });
        hooks::IT_8029A89C = Some(|_, x| {
            t(0xB00);
            t(b(x));
        });
        hooks::IT_8029A8F4 = Some(|_, v| {
            t(0xC00);
            t(b(v.x));
            t(b(v.y));
            t(b(v.z));
        });
        hooks::FTCO_80090690 =
            Some(|fp| if fp.kind & 1 != 0 { None } else { Some(Vec2 { x: fp.cur_pos.x, y: fp.cur_pos.y }) });
        hooks::FTCO_800DEEE8 =
            Some(|fp| if fp.kind & 2 != 0 { None } else { Some(Vec2 { x: fp.xD4_unk_vel.x, y: fp.xD4_unk_vel.y }) });
    }
}

#[test]
fn ftcommon_field_logic_matches_c() {
    install_hooks();
    let mut cd: Box<ftCommonData> = Box::new(ftCommonData::zeroed());
    let mut shake_v = vec![Vec2 { x: 0.0, y: 0.0 }; 256];
    let mut shake = Box::new(Fighter_ShakeTable_t { x0: shake_v.as_mut_ptr(), x4: 0 });
    let mut sandbag = [0f32; 2];
    let mut item_a: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut item_b: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    unsafe {
        hooks::P_FT_COMMON_DATA = &*cd;
    }
    let expected = include_str!("data/ftcommon2.txt");
    let mut lines = expected.lines();
    let mut checked = 0;
    for op in 0..34u32 {
        for k in 0..300u32 {
            let mut fp: Box<Fighter> = Box::new(Fighter::zeroed());
            *cd = ftCommonData::zeroed();
            unsafe {
                TR = 0;
                hooks::P_FT_COMMON_DATA = &*cd;
                hooks::FIGHTER_GRAB_MASH_SHAKE = &*shake;
            }
            let mut r = Rng(0x2468Bu32.wrapping_add(op.wrapping_mul(7919)).wrapping_add(k.wrapping_mul(104729)));
            fp.kind = r.rnd() % 34;
            fp.player_idx = (r.rnd() % 6) as u8;
            fp.pad_port = (r.rnd() % 4) as u8;
            fp.costume_id = (r.rnd() % 6) as u8;
            fp.x8_spawnNum = (r.rnd() % 8) as i32;
            fp.facing_dir = r.rf();
            fp.gr_vel = r.rf();
            fp.self_vel.x = r.rf();
            fp.self_vel.y = r.rf();
            fp.cur_pos.x = r.rf();
            fp.cur_pos.y = r.rf();
            fp.cur_pos.z = r.rf();
            fp.xD4_unk_vel.x = r.rf();
            fp.xD4_unk_vel.y = r.rf();
            fp.xD4_unk_vel.z = r.rf();
            fp.x34_scale.x = r.rf();
            fp.x34_scale.y = r.rf();
            fp.x34_scale.z = r.rf();
            fp.x40 = r.rf();
            fp.xF4_ground_attacker_shield_kb_vel = r.rf();
            fp.coll_data.floor.normal.x = r.rf();
            fp.coll_data.floor.normal.y = r.rf();
            fp.coll_data.x130_flags = r.rnd();
            fp.ground_or_air = r.rnd() & 1;
            fp.ecb_lock = (r.rnd() % 20) as i32;
            fp.x1968_jumpsUsed = (r.rnd() % 4) as u8;
            fp.x1969_walljumpUsed = (r.rnd() % 2) as u8;
            fp.co_attrs.max_jumps = (r.rnd() % 7) as i32;
            fp.co_attrs.model_scaling = r.rf();
            fp.co_attrs.x12C = r.rf();
            fp.co_attrs.x130.x = r.rf();
            fp.co_attrs.x130.y = r.rf();
            fp.co_attrs.x130.z = r.rf();
            fp.co_attrs.x13C = r.rf();
            fp.dmg.x18A4_knockbackMagnitude = r.rf();
            fp.dmg.x18c4_source_ply = (r.rnd() % 7) as i32;
            fp.dmg.x18C8 = r.rnd() as i32;
            fp.input.lstick[0].x = r.rf();
            fp.input.lstick[0].y = r.rf();
            fp.input.triggers[0] = r.rf();
            fp.input.pressed_buttons = r.rnd() & r.rnd();
            fp.activity_timer.lstick.x = r.rnd() as u8;
            fp.activity_timer.lstick.y = r.rnd() as u8;
            fp.activity_timer.trigger = r.rnd() as u8;
            fp.grab_timer = r.rf();
            fp.x1A50 = r.tri();
            fp.x1A51 = r.tri();
            fp.x1A52 = r.rnd() as u8;
            fp.x1A53 = r.rnd() as u8;
            fp.x2018 = (r.rnd() % 2001) as i32 - 1000;
            fp.x2020 = r.tri();
            fp.x2021 = r.tri();
            fp.x2024 = (r.rnd() % 201) as i32 - 100;
            fp.smash_attrs.x2138_smashSinceHitbox = if r.rnd() & 1 != 0 { -1.0 } else { r.rf() };
            fp.x197C = if r.rnd() & 1 != 0 { &mut *item_a } else { core::ptr::null_mut() };
            fp.x1980 = if r.rnd() & 1 != 0 { &mut *item_b } else { core::ptr::null_mut() };
            fp.dat_attrs = sandbag.as_mut_ptr().cast();
            sandbag[0] = r.rf();
            sandbag[1] = r.rf();
            let v = r.rnd() & 1;
            let _ = v; // x594_b0 (in an anonymous union; no ported function reads it)
            let v = r.rnd() & 1;
            fp.set_x2219_b0(v as u8);
            let v = r.rnd() & 1;
            fp.set_is_sleeping(v as u8);
            let v = r.rnd() & 1;
            fp.set_is_sub_fighter(v as u8);
            let v = r.rnd() & 1;
            fp.set_stamina_dead(v as u8);
            let v = r.rnd() & 1;
            fp.set_x2224_b5(v as u8);
            let v = r.rnd() & 1;
            fp.set_x2224_b6(v as u8);
            let v = r.rnd() & 1;
            fp.set_x2225_b5(v as u8);
            let v = r.rnd() & 1;
            fp.set_x2225_b6(v as u8);
            let v = r.rnd() & 1;
            fp.set_x2227_b0(v as u8);
            cd.x138 = r.rf();
            cd.x13C = r.rf();
            cd.xEC = r.rf();
            cd.xF0 = r.rf();
            cd.x308 = r.rf();
            cd.x708 = r.rf();
            cd.x70C = r.rf();
            cd.x710 = r.rf();
            cd.x7B8 = r.rf();
            cd.x7BC = r.rf();
            cd.x7C0 = r.rf();
            shake.x4 = (r.rnd() % 300) as i32;
            for e in shake_v.iter_mut() {
                e.x = r.rf();
                e.y = r.rf();
            }
            unsafe {
                G_FRIC = r.rf();
            }
            let a = r.rf();
            let _b = r.rf();
            let ia = (r.rnd() % 64) as i32 - 16;
            let ib = r.rnd() as i32;
            let mut ret: f32 = 0.0;
            let mut v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            let bf = |x: bool| if x { 1.0f32 } else { 0.0 };
            match op {
                0 => f::unlock_ecb(&mut fp),
                1 => f::ftcommon_8007d5d4(&mut fp),
                2 => f::ftcommon_8007d60c(&mut fp),
                3 => f::use_all_jumps(&mut fp),
                4 => f::ftcommon_8007d780(&mut fp),
                5 => f::init_grab(&mut fp, ia & 3, a),
                6 => ret = bf(f::grab_mash(&mut fp, a)),
                7 => f::ftcommon_8007e2a4(&mut fp),
                8 => f::ftcommon_8007e2f4(&mut fp, ib as i16),
                9 => f::ftcommon_8007e2fc(&mut fp),
                10 => f::ftcommon_8007ebac(&mut fp, ia as u32, ib as u32),
                11 => f::ftcommon_8007ecd4(&mut fp, ia),
                12 => f::ftcommon_8007ed2c(&mut fp),
                13 => f::ftcommon_8007ed50(&mut fp, ia),
                14 => f::ftcommon_8007ee0c(&mut fp, ia),
                15 => f::ftcommon_8007efc0(&mut fp, ib as u32),
                16 => ret = f::get_model_scale(&fp),
                17 => f::ftcommon_8007f76c(&mut fp),
                18 => f::ftcommon_8007f7b4(&mut fp),
                19 => {
                    let p = f::ftcommon_8007f804(&mut fp);
                    ret = if p == fp.x44_mtx.as_mut_ptr() {
                        1.0
                    } else if p.is_null() {
                        2.0
                    } else {
                        3.0
                    };
                }
                20 => v = f::ftcommon_8007f8b4(&fp),
                21 => f::ftcommon_8007fda0(&mut fp),
                22 => f::ftcommon_8007ff74(&mut fp),
                23 => ret = bf(f::ftcommon_8007ffd8(&mut fp, a)),
                24 => ret = bf(f::ftcommon_80080144(&fp)),
                25 => f::ftcommon_80080174(&mut fp),
                26 => f::ftcommon_8008021c(&mut fp),
                27 => f::ftcommon_8008031c(&mut fp),
                28 => {
                    f::ftcommon_80080460(&mut fp);
                    f::ftcommon_80080474(&mut fp);
                    t(fp.x2225_b6() as u32);
                    f::ftcommon_80080460(&mut fp);
                }
                29 => f::ftcommon_80080484(&mut fp),
                30 => f::ftcommon_800804a0(&mut fp, a),
                31 => ret = f::ftcommon_800804ec(&fp),
                32 => f::ftcommon_800804fc(&mut fp),
                33 => {
                    fp.kind = Ft_Kind_Sandbag;
                    ret = f::sandbag_get_knockback_deaccel_x(&fp);
                    t(b(f::sandbag_get_knockback_deaccel_y(&fp)));
                }
                _ => unreachable!(),
            }
            if fp.x1980.is_null() {
                t(0xDEAD);
            }
            let bits = (fp.x2224_b5() as u32)
                | ((fp.x2224_b6() as u32) << 1)
                | ((fp.x2225_b6() as u32) << 2)
                | ((fp.x2227_b0() as u32) << 3)
                | ((fp.activity_timer.lstick.x as u32) << 8)
                | ((fp.activity_timer.lstick.y as u32) << 16)
                | ((fp.activity_timer.trigger as u32) << 24);
            let got = format!(
                "{} {} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} \
                 {:08X} {:08X} {:08X} {:08X} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {:08X} {:08X} {:08X}",
                op,
                k,
                b(ret),
                b(fp.gr_vel),
                b(fp.self_vel.x),
                b(fp.self_vel.y),
                b(fp.self_vel.z),
                b(fp.x74_self_accel.y),
                b(fp.x98_atk_shield_kb.x),
                b(fp.x98_atk_shield_kb.y),
                b(fp.x98_atk_shield_kb.z),
                b(fp.cur_pos.z),
                b(fp.grab_timer),
                b(fp.xE8_ground_accel_2),
                b(fp.dmg.x18A4_knockbackMagnitude),
                b(fp.dmg.x18B8),
                b(fp.dmg.x18BC),
                b(fp.smash_attrs.x2138_smashSinceHitbox),
                b(v.x),
                b(v.y),
                fp.ground_or_air as i32,
                fp.ecb_lock,
                fp.coll_data.x130_flags as i32,
                fp.x1968_jumpsUsed,
                fp.x1969_walljumpUsed,
                fp.x1A50,
                fp.x1A51,
                fp.x1A52,
                fp.x1A53,
                fp.x2018,
                fp.x2020,
                fp.x2021,
                fp.x1A6A,
                fp.x209A,
                fp.dmg.x18c4_source_ply.wrapping_mul(1000).wrapping_add(fp.dmg.x18C8 & 0xFF),
                b(v.z),
                bits,
                unsafe { TR }
            );
            assert_eq!(got, lines.next().expect("ftcommon2.txt too short"), "op {op} case {k}");
            checked += 1;
        }
    }
    assert!(lines.next().is_none(), "ftcommon2.txt has extra lines");
    assert_eq!(checked, 34 * 300);
}
