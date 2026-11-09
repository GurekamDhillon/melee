//! src/tu/ft_0d4d.rs vs the C (gen/cref/tu_ft_0d4d_ref.c, shipping contraction), 10000 cases:
//! fn_800D54A4, fn_800D55B4, ftCo_Rebirth_Phys, ftCo_RebirthWait_Phys, ftCo_800D4E50.
#![allow(static_mut_refs)]
use melee_fighter::hooks;
use melee_fighter::tu::ft_0d4d as t;
use melee_fighter::types::*;
use std::fmt::Write;

static mut S: u32 = 0;
fn rnd() -> u32 {
    unsafe {
        S ^= S << 13;
        S ^= S >> 17;
        S ^= S << 5;
        S
    }
}
fn rf() -> f32 {
    match rnd() % 16 {
        0 => 0.0,
        1 => -0.0,
        2 => (rnd() % 7) as f32 * 1e-4f32,
        _ => ((rnd() % 200001) as i32 - 100000) as f32 / 37.0,
    }
}
fn b(f: f32) -> u32 { f.to_bits() }

static mut STAGE: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
static mut GM: bool = false;
static mut OUT: String = String::new();

fn stage(v: &mut Vec3, _i: i32) { *v = unsafe { STAGE } }
fn gm() -> bool { unsafe { GM } }
fn it(_g: *mut HSD_GObj, v: &Vec3, a: i32, bb: i32, c: f32, d: f32) -> i32 {
    unsafe {
        write!(OUT, " IT {:08x} {:08x} {:08x} {} {} {:08x} {:08x}", b(v.x), b(v.y), b(v.z), a, bb, b(c), b(d)).unwrap();
    }
    0
}
fn set_tr(j: *mut HSD_JObj, v: &Vec3) { unsafe { (*j).translate = *v } }

fn zeroed<T>() -> Box<T> { unsafe { Box::new(core::mem::zeroed()) } }

#[test]
fn ft_0d4d_vs_c() {
    let p = format!("{}/tests/data/tu_ft_0d4d.txt", env!("CARGO_MANIFEST_DIR"));
    let exp: std::vec::Vec<String> = std::fs::read_to_string(&p).unwrap().lines().map(String::from).collect();
    let mut f: Box<Fighter> = zeroed();
    let mut o: Box<Fighter> = zeroed();
    let mut g: Box<HSD_GObj> = zeroed();
    let mut go: Box<HSD_GObj> = zeroed();
    let mut j: Box<HSD_JObj> = zeroed();
    let mut cd: Box<ftCommonData> = zeroed();
    g.user_data = &mut *f as *mut Fighter as *mut _;
    go.user_data = &mut *o as *mut Fighter as *mut _;
    unsafe {
        hooks::P_FT_COMMON_DATA = &*cd;
        t::ext::H_Stage_80224E38 = Some(stage);
        hooks::GM_8016B0B4 = Some(gm);
        hooks::IT_802E5F8C = Some(it);
        hooks::HSD_JOBJ_SET_TRANSLATE = Some(set_tr);
        melee_fighter::stale::PLAYER_SLOTS[0].transformed = [0, 1];
    }
    let gp: *mut HSD_GObj = &mut *g;
    let mut n = 0;
    for op in 0..5u32 {
        for k in 0..2000u32 {
            unsafe { S = 0xD4D0u32.wrapping_add(op.wrapping_mul(7919)).wrapping_add(k.wrapping_mul(104729)) };
            unsafe {
                core::ptr::write_bytes(&mut *f as *mut Fighter, 0, 1);
                core::ptr::write_bytes(&mut *o as *mut Fighter, 0, 1);
                core::ptr::write_bytes(&mut *j as *mut HSD_JObj, 0, 1);
                core::ptr::write_bytes(&mut *cd as *mut ftCommonData, 0, 1);
            }
            f.facing_dir = rf();
            f.x40 = rf();
            f.x34_scale.x = rf();
            f.cur_pos.x = rf();
            f.cur_pos.y = rf();
            f.cur_pos.z = rf();
            f.xD4_unk_vel.x = rf();
            f.xD4_unk_vel.y = rf();
            unsafe {
                f.mv.co.common.x4.x = rf();
                f.mv.co.common.x4.y = rf();
                f.mv.co.common.x0 = (rnd() % 21) as i32 - 10;
            }
            f.smash_attrs.x2135 = if rnd() & 1 != 0 { -1 } else { (rnd() % 4) as i8 };
            f.set_is_sub_fighter((rnd() & 1) as _);
            o.cur_pos.y = rf();
            o.self_vel.x = rf();
            o.self_vel.y = rf();
            o.set_is_sleeping((rnd() & 1) as _);
            unsafe {
                STAGE = Vec3 { x: rf(), y: rf(), z: rf() };
                let sp = Vec3 { x: rf(), y: rf(), z: rf() };
                melee_fighter::player::set_some_pos(0, &sp);
                melee_fighter::stale::PLAYER_SLOTS[0].player_entity[0] = &mut *go;
                melee_fighter::stale::PLAYER_SLOTS[0].player_entity[1] =
                    if rnd() & 1 != 0 { &mut *go } else { core::ptr::null_mut() };
            }
            j.flags = 1 << 25;
            f.x20A0_accessory = &mut *j;
            melee_fighter::player::set_coins(0, (rnd() % 400) as i32 - 50);
            melee_fighter::player::set_unk9c(0, (rnd() % 100) as i32);
            unsafe { GM = rnd() & 1 != 0 };
            cd.x5C8 = (rnd() % 200) as i32;
            cd.x5CC = rf();
            unsafe {
                OUT.clear();
                write!(OUT, "{} {}", op, k).unwrap();
            }
            match op {
                0 => t::fn_800D54A4(gp),
                1 => t::fn_800D55B4(gp),
                2 | 3 => {
                    unsafe {
                        if f.mv.co.common.x0 == 0 {
                            f.mv.co.common.x0 = 3;
                        }
                    }
                    if op == 2 { t::ftCo_Rebirth_Phys(gp) } else { t::ftCo_RebirthWait_Phys(gp) }
                }
                _ => {
                    let a = rf();
                    let bb = (rnd() & 1) as i32;
                    let v = Vec3 { x: rf(), y: rf(), z: rf() };
                    t::ftCo_800D4E50(&mut f, &v, bb, a);
                }
            }
            let line = unsafe {
                let c = &f.mv.co.common;
                format!(
                    "{} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {} {}",
                    OUT,
                    b(f.cur_pos.x), b(f.cur_pos.y), b(f.self_vel.x), b(f.self_vel.y),
                    b(c.x4.x), b(c.x4.y), b(c.x4.z), b(j.translate.x), b(j.translate.y),
                    melee_fighter::player::get_coins(0), melee_fighter::player::get_unk9c(0)
                )
            };
            assert_eq!(line, exp[n], "case {n}");
            n += 1;
        }
    }
    assert_eq!(n, exp.len());
}
