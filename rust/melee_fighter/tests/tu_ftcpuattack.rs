//! src/tu/ftcpuattack.rs vs the C of src/melee/ft/ftcpuattack.c (gen/cref/tu_ftcpuattack_ref.c
//! #includes the whole TU; expected rows in tests/data/tu_ftcpuattack.txt; regenerate with
//! `python rust/melee_fighter/gen/cref/tu_run.py tu_ftcpuattack`). Shipping-build contraction.
//! The generator, HSD_Randf and the lbColl stubs repeat the C driver's exactly.
#![allow(static_mut_refs)]

use melee_fighter::tu::ftcpuattack::{self as m, ext, ftCo_AttackEntry};
use melee_fighter::types::*;

static mut RS: u32 = 0;
static mut RR: u32 = 0;
static mut TRC: u32 = 0;

fn rnd() -> u32 {
    unsafe {
        RS ^= RS << 13;
        RS ^= RS >> 17;
        RS ^= RS << 5;
        RS
    }
}
fn randf() -> f32 {
    unsafe {
        RR ^= RR << 13;
        RR ^= RR >> 17;
        RR ^= RR << 5;
        (RR % 999) as f32 / 1000.0
    }
}
fn rf() -> f32 {
    let k = rnd() % 16;
    if k == 0 {
        return 0.0;
    }
    if k == 1 {
        return -0.0;
    }
    ((rnd() % 2001) as i32 - 1000) as f32 / 7.0
}
fn gr() -> f32 {
    match rnd() % 6 {
        0 => 0.0,
        1 => 0.000001,
        2 => -0.000005,
        _ => rf() / 50.0,
    }
}
fn sc() -> f32 {
    if rnd() % 3 == 0 { 1.0 } else { (rnd() % 200 + 1) as f32 / 100.0 }
}
fn side() -> f32 {
    if rnd() % 2 != 0 { 1.0 } else { -1.0 }
}
fn mover(f: &mut Fighter) {
    f.cur_pos.x = rf();
    f.cur_pos.y = rf();
    f.pos_delta.x = rf();
    f.pos_delta.y = rf();
    f.co_attrs.terminal_velocity = rf();
    f.co_attrs.gravity = gr();
    f.facing_dir = side();
    f.x34_scale.y = sc();
    f.ground_or_air = rnd() % 2;
    if rnd() % 2 != 0 {
        f.cpu.x55C = (rnd() % 300) as f32;
        f.cpu.x560 = (rnd() % 300) as f32;
        f.cpu.x568 = (rnd() % 300) as f32;
    } else {
        f.cpu.x55C = rf();
        f.cpu.x560 = rf();
        f.cpu.x568 = rf();
    }
}
fn mix(v: u32) {
    unsafe { TRC = TRC.wrapping_mul(31).wrapping_add(v) };
}

#[test]
fn matches_c() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftcpuattack.txt")).unwrap();
    unsafe {
        ext::HSD_RANDF = Some(randf);
        ext::LBCOLL_8000ACFC = Some(|_, _| 0);
        ext::LBCOLL_80006094 = Some(|_, b, _, _, _, _, s0, s1| {
            let b = unsafe { *b };
            mix(b.x.to_bits());
            mix(b.y.to_bits());
            mix(b.z.to_bits());
            mix(s0.to_bits());
            mix(s1.to_bits());
            0
        });
    }
    let mut lines = want.lines();
    let mut n = 0;
    for k in 0..20000u32 {
        let mode = k % 4;
        let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
        let mut t: Box<Fighter> = Box::new(Fighter::zeroed());
        let mut it: Box<Item> = Box::new(Item::zeroed());
        let mut ia: Box<ItemAttr> = Box::new(ItemAttr::zeroed());
        let mut l = [ftCo_AttackEntry::default(); 16];
        unsafe {
            RS = 0x1234567u32.wrapping_add(k.wrapping_mul(2654435761));
            if RS == 0 {
                RS = 1;
            }
            RR = k.wrapping_mul(7919).wrapping_add(1);
        }
        mover(&mut f);
        mover(&mut t);
        f.cpu.level = (rnd() % 10) as i32;
        f.cpu.xEC = (rnd() % 3) as u8;
        for i in 0..f.cpu.xEC as usize {
            f.cpu.xCC_array[i] = (rnd() % 8 + 1) as i32;
        }
        f.cpu.xC8 = if rnd() % 8 == 0 { (rnd() % 3) as u8 } else { 0 };
        for i in 0..f.cpu.xC8 as usize {
            f.cpu.xA8_array[i] = (rnd() % 8 + 1) as i32;
        }
        f.cpu.x80 = (rnd() % 100) as i32;
        f.cpu.x570 = rf() / 100.0;
        let cnt = rnd() % 12 + 1;
        for e in l.iter_mut().take(cnt as usize) {
            e.cmd = (rnd() % 8 + 1) as i32;
            e.x04 = (rnd() % 40) as i32;
            if rnd() % 2 != 0 {
                e.x08 = -((rnd() % 600) as f32);
                e.x0C = (rnd() % 600) as f32;
                e.x10 = -((rnd() % 600) as f32);
                e.x14 = (rnd() % 600) as f32;
            } else {
                e.x08 = rf();
                e.x0C = rf();
                e.x10 = rf();
                e.x14 = rf();
            }
            e.weight = if rnd() % 4 == 0 { 0.0 } else { (rnd() % 100) as f32 / 10.0 };
            e.x1C = if rnd() % 4 == 0 { 3 } else { 1 };
            e.x20 = (rnd() % 10) as i32;
        }
        unsafe { TRC = 0 };
        let ret = match mode {
            0 => m::ftCo_800B4AB0(&mut *f, &mut *t, l.as_ptr()),
            1 => {
                let reach = rf();
                m::ftCo_800B52AC(&mut *f, &mut *t, l.as_ptr(), reach)
            }
            2 => {
                it.pos.x = rf();
                it.pos.y = rf();
                it.x40_vel.x = rf();
                it.x40_vel.y = rf();
                ia.x10_fall_speed = gr();
                ia.x14_fall_speed_max = rf();
                it.xC1C.top = (rnd() % 400) as f32;
                it.xC1C.bottom = rf();
                it.xC1C.right = (rnd() % 400) as f32;
                it.xC1C.left = rf();
                it.ground_or_air = rnd() % 2;
                it.xCC_item_attr = &mut *ia;
                m::ftCo_800B5AB0(&mut *f, &mut *it, l.as_ptr())
            }
            _ => {
                for i in 0..4 {
                    let h = &mut t.x914[i];
                    h.state = rnd() % 4;
                    h.set_x43_b2((rnd() % 4 == 0) as u8);
                    h.element = rnd() % 10;
                    h.x4C.x = rf();
                    h.x4C.y = rf();
                    h.x4C.z = rf();
                    h.x58.x = rf();
                    h.x58.y = rf();
                    h.x58.z = rf();
                    h.scale = rf();
                }
                let mut p = Vec3 { x: rf(), y: rf(), z: rf() };
                let a3 = rf();
                m::ftCo_800BB104(&mut *f, &mut *t, &mut p, a3) as i32
            }
        };
        let got = format!(
            "{} {:08X} {:08X} {:08X} {:08X} {:08X}",
            ret,
            f.cpu.x6C.x.to_bits(),
            f.cpu.x6C.y.to_bits(),
            f.cpu.x74.x.to_bits(),
            f.cpu.x74.y.to_bits(),
            unsafe { TRC }
        );
        assert_eq!(got, lines.next().unwrap(), "case {k} mode {mode}");
        n += 1;
    }
    assert_eq!(n, 20000);
    // ftCo_800B9704: reaction delay (one HSD_Randf draw, two fused sites)
    for k in 0..4000u32 {
        let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
        unsafe {
            RS = 0x7654321u32.wrapping_add(k.wrapping_mul(2654435761));
            if RS == 0 {
                RS = 1;
            }
            RR = k.wrapping_mul(104729).wrapping_add(3);
        }
        f.cpu.level = (rnd() % 12) as i32 - 1;
        f.cpu.kind = if rnd() % 3 == 0 { 7 } else { rnd() % 12 };
        f.cpu.x34 = rnd() as i32;
        m::ftCo_800B9704(&mut *f);
        let got = format!("R {} {} {}", f.cpu.level, f.cpu.kind, f.cpu.x34);
        assert_eq!(got, lines.next().unwrap(), "800B9704 case {k}");
    }
    assert!(lines.next().is_none());
}
