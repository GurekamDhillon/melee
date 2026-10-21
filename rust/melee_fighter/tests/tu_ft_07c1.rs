//! src/tu/ft_07c1.rs vs the C of src/melee/ft/ft_07C1.c (gen/cref/tu_ft_07C1_ref.c, expected log
//! in tests/data/tu_ft_07C1.txt; regenerate with
//! `python rust/melee_fighter/gen/cref/tu_run_s57.py tu_ft_07C1`). Fill order matches the C.
#![allow(static_mut_refs)]

use melee_fighter::hooks;
use melee_fighter::tu::ft_07c1::{self as m, ext};
use melee_fighter::types::*;

const NF: usize = 4;
static mut LOG: String = String::new();
static mut FS: *mut Fighter = core::ptr::null_mut();
static mut GO: *mut HSD_GObj = core::ptr::null_mut();
static mut JOINTS: *const u8 = core::ptr::null();
static mut ITEM_GO: *mut HSD_GObj = core::ptr::null_mut();
static mut G_KIND: i32 = 0;
static mut G_B868: [i32; NF] = [0; NF];
static mut G_ACFC: bool = false;
static mut G_7AFC: bool = false;
static mut G_POS: [f32; 3] = [0.0; 3];

fn log(s: String) {
    unsafe {
        LOG.push_str(&s);
        LOG.push('\n');
    }
}
fn b(f: f32) -> String {
    format!("{:08X}", f.to_bits())
}
fn fi<T>(p: *const T) -> i32 {
    if p.is_null() {
        return -1;
    }
    let (a, s) = (p as usize, unsafe { FS } as usize);
    let n = core::mem::size_of::<Fighter>();
    if a >= s && a < s + NF * n { ((a - s) / n) as i32 } else { 99 }
}
fn gi<T>(p: *const T) -> i32 {
    if p.is_null() {
        return -1;
    }
    let (a, s) = (p as usize, unsafe { GO } as usize);
    let n = core::mem::size_of::<HSD_GObj>();
    if a >= s && a < s + NF * n { ((a - s) / n) as i32 } else { 99 }
}
fn ji<T>(p: *const T) -> i32 {
    if p.is_null() { -1 } else { (p as usize - unsafe { JOINTS } as usize) as i32 }
}
fn fnv<T>(v: &T) -> u32 {
    let b = unsafe { core::slice::from_raw_parts(v as *const T as *const u8, core::mem::size_of::<T>()) };
    b.iter().fold(2166136261u32, |h, &x| (h ^ x as u32).wrapping_mul(16777619))
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
fn ft_07c1_matches_c() {
    let mut fs: std::vec::Vec<Fighter> = (0..NF).map(|_| Fighter::zeroed()).collect();
    let mut go: std::vec::Vec<HSD_GObj> = (0..NF).map(|_| HSD_GObj::zeroed()).collect();
    let joints = [0u8; 8];
    let mut bones: [FighterBone; 8] = core::array::from_fn(|_| FighterBone::zeroed());
    for (i, bn) in bones.iter_mut().enumerate() {
        bn.joint = &joints[i] as *const u8 as *mut HSD_JObj;
    }
    let mut x34 = ftData_x34::zeroed();
    let mut fd = ftData::zeroed();
    fd.x34 = &mut x34;
    let mut cd = ftCommonData::zeroed();
    let mut item_go = HSD_GObj::zeroed();
    let jp = |i: u32| &joints[i as usize] as *const u8 as *mut HSD_JObj;
    unsafe {
        FS = fs.as_mut_ptr();
        GO = go.as_mut_ptr();
        JOINTS = joints.as_ptr();
        ITEM_GO = &mut item_go;
        hooks::P_FT_COMMON_DATA = &cd;
        hooks::PLINK_HEAD = Some(|l| if l == HSD_GOBJ_PLINK_FIGHTER { GO } else { core::ptr::null_mut() });
        hooks::IT_GET_KIND = Some(|g| {
            log(format!("KIND {}", (g == ITEM_GO) as i32));
            G_KIND
        });
        ext::FTCO_800C555C = Some(|g| log(format!("C555C {}", gi(g))));
        ext::LBCOLL_80008440 = Some(|h| {
            log(format!("8440 {}", fi(h)));
            (*h).x4C.x = 1.5;
        });
        ext::LB_8000B1CC = Some(|j, a, o| {
            log(format!("B1CC {} {} {} {}", ji(j), b((*a).x), b((*a).y), b((*a).z)));
            (*o).x = G_POS[0];
            (*o).y = G_POS[1];
            (*o).z = G_POS[2];
        });
        ext::LBCOLL_8000ACFC = Some(|v, h| {
            log(format!("ACFC {} {}", fi(v), fi(h)));
            G_ACFC
        });
        ext::LBCOLL_80008688 = Some(|h, t, v| {
            log(format!("8688 {} {} {}", fi(h), t, fi(v)));
            false
        });
        ext::FTCOLL_8007B868 = Some(|g| {
            let i = gi(g);
            log(format!("B868 {}", i));
            G_B868[i as usize]
        });
        ext::LBCOLL_80007AFC = Some(|a, bb, x, y| {
            log(format!("7AFC {} {} {} {}", fi(a), fi(bb), b(x), b(y)));
            G_7AFC
        });
        ext::LBCOLL_800089B8 = Some(|h, v| log(format!("89B8 {} {}", fi(h), fi(v))));
    }
    let gp = go.as_mut_ptr();
    for op in 0..5u32 {
        for k in 0..1500u32 {
            for i in 0..NF {
                fs[i] = Fighter::zeroed();
                go[i] = HSD_GObj::zeroed();
            }
            cd = ftCommonData::zeroed();
            let mut r = Rng(0x7C114u32.wrapping_add(op.wrapping_mul(7919)).wrapping_add(k.wrapping_mul(104729)));
            let n = 1 + (r.rnd() % NF as u32) as usize;
            for i in 0..NF {
                go[i].user_data = &mut fs[i] as *mut Fighter as *mut _;
                go[i].next = if i + 1 < n { unsafe { gp.add(i + 1) } } else { core::ptr::null_mut() };
                let f = &mut fs[i];
                f.ft_data = &mut fd;
                f.parts = bones.as_mut_ptr();
                f.item_gobj = if r.rnd() & 1 != 0 { &mut item_go } else { core::ptr::null_mut() };
                f.set_x2219_b4((r.rnd() & 1) as u8);
                f.set_x2219_b1((r.rnd() % 4 == 0) as u8);
                f.set_x2227_b2((r.rnd() % 4 != 0) as u8);
                f.set_is_sub_fighter((r.rnd() % 6 == 0) as _);
                f.set_x221C_b6((r.rnd() % 4 == 0) as _);
                f.victim_gobj = if r.rnd() % 4 == 0 { unsafe { gp.add((r.rnd() % NF as u32) as usize) } } else { core::ptr::null_mut() };
                f.dmg.x1868_source = if r.rnd() % 3 == 0 { unsafe { gp.add((r.rnd() % NF as u32) as usize) } } else { core::ptr::null_mut() };
                f.x1064_thrownHitbox.state = (r.rnd() % 5) as _;
                f.x1064_thrownHitbox.x4 = r.rnd() as _;
                f.x1064_thrownHitbox.jobj = jp(r.rnd() % 8);
                f.x1064_thrownHitbox.b_offset.x = r.rf();
                f.x1064_thrownHitbox.b_offset.y = r.rf();
                f.x1064_thrownHitbox.x4C.x = r.rf();
                f.x1064_thrownHitbox.x4C.y = r.rf();
                f.x1064_thrownHitbox.set_x43_b1((r.rnd() & 1) as u8);
                f.x1064_thrownHitbox.set_x43_b2((r.rnd() & 1) as u8);
                f.ground_or_air = (r.rnd() % 2) as _;
                f.co_attrs.weight = r.rf();
                f.pos_delta.x = r.rf();
                f.pos_delta.y = r.rf();
                f.x98_atk_shield_kb.x = r.rf();
                f.x98_atk_shield_kb.y = r.rf();
                f.x34_scale.y = r.rf();
                unsafe { G_B868[i] = if r.rnd() % 5 == 0 { 2 } else { 0 } };
            }
            x34.x0 = (r.rnd() % 8) as _;
            x34.scale = r.rf();
            cd.hit_weight_mul = r.rf();
            unsafe {
                G_KIND = if r.rnd() & 1 != 0 { It_Kind_Hammer as i32 } else { (r.rnd() % 40) as i32 };
                G_ACFC = r.rnd() % 4 == 0;
                G_7AFC = r.rnd() & 1 != 0;
                G_POS = [r.rf(), r.rf(), r.rf()];
            }
            let sel = (r.rnd() % n as u32) as usize;
            log(format!("OP {} {} {} {}", op, k, n, sel));
            let g = unsafe { gp.add(sel) };
            match op {
                0 => m::ft_8007c114(g),
                1 => m::ft_8007c17c(g),
                2 => m::ft_8007c224(g),
                3 => {
                    if n > 1 {
                        let fp = fs.as_mut_ptr();
                        unsafe {
                            m::ft_8007c2e0(fp, &mut (*fp).x1064_thrownHitbox, fp.add(1), &mut (*fp.add(1)).x1064_thrownHitbox)
                        };
                    }
                }
                _ => m::ft_8007c4bc(g),
            }
            for i in 0..NF {
                let f = &mut fs[i];
                log(format!(
                    "F {} {}{}{}{}{} {}{} {} {} {} {}",
                    i, f.x2219_b4(), f.x2219_b1(), f.x2227_b2(), f.is_sub_fighter(), f.x221C_b6(),
                    f.x1064_thrownHitbox.x43_b1(), f.x1064_thrownHitbox.x43_b2(),
                    (!f.item_gobj.is_null()) as i32, gi(f.victim_gobj), gi(f.dmg.x1868_source), ji(f.x1064_thrownHitbox.jobj)
                ));
                f.set_x2219_b4(0);
                f.set_x2219_b1(0);
                f.set_x2227_b2(0);
                f.set_is_sub_fighter(0);
                f.set_x221C_b6(0);
                f.x1064_thrownHitbox.set_x43_b1(0);
                f.x1064_thrownHitbox.set_x43_b2(0);
                f.item_gobj = core::ptr::null_mut();
                f.victim_gobj = core::ptr::null_mut();
                f.dmg.x1868_source = core::ptr::null_mut();
                f.x1064_thrownHitbox.jobj = core::ptr::null_mut();
                f.ft_data = core::ptr::null_mut();
                f.parts = core::ptr::null_mut();
                log(format!("H {:08X}", fnv(&*f)));
            }
        }
    }
    let got = unsafe { LOG.clone() };
    let mut el = include_str!("data/tu_ft_07C1.txt").lines();
    let mut n = 0;
    for gl in got.lines() {
        assert_eq!(gl, el.next().expect("C log shorter"), "line {n}");
        n += 1;
    }
    assert!(el.next().is_none(), "C log longer");
}
