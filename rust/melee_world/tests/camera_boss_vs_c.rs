//! `melee_world::cm::camera_boss` (Camera_8002E6FC .. Camera_8002F274 with Camera_8002DFE4 /
//! Camera_8002E234) against the real C (cref/camera_boss_ref.c: the functions cut verbatim out of
//! src/melee/cm/camera.c, shipping contraction + gwtool lowering; expected values in
//! tests/data/camera_boss_ref.txt; regenerate with
//! `python rust/melee_world/cref/gen_camera_boss_ref.py`). Fill order matches the C driver exactly.
#![allow(static_mut_refs)]

use melee_types::{Camera, Vec as Vec3, CAMERA_BOSS_INTRO};
use melee_world::cm::camera::CameraWorld;

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
        if k == 2 {
            return 1.0;
        }
        ((self.rnd() % 2001) as i32 - 1000) as f32 / 37.0
    }
    fn rv(&mut self) -> Vec3 {
        Vec3 { x: self.rf(), y: self.rf(), z: self.rf() }
    }
}

static mut CBV: [Vec3; 2] = [Vec3 { x: 0.0, y: 0.0, z: 0.0 }; 2];
static mut CBRET: [bool; 2] = [false; 2];
static mut CBLOG: u32 = 0;
static mut SUBJ: [Option<Vec3>; 6] = [None; 6];

unsafe extern "C" fn cb_a(v: *mut Vec3) -> bool {
    *v = CBV[0];
    CBLOG = CBLOG.wrapping_mul(31).wrapping_add(1);
    CBRET[0]
}
unsafe extern "C" fn cb_b(v: *mut Vec3) -> bool {
    *v = CBV[1];
    CBLOG = CBLOG.wrapping_mul(31).wrapping_add(2);
    CBRET[1]
}
fn subject(slot: i32) -> Option<Vec3> {
    if (0..6).contains(&slot) {
        unsafe { SUBJ[slot as usize] }
    } else {
        None
    }
}

fn state(c: &Camera, dump: bool) -> u32 {
    let mut h: u32 = 0;
    let mut hm = |v: u32| {
        h = h.wrapping_mul(31).wrapping_add(v);
        if dump {
            print!(" {:08X}", v);
        }
    };
    let b = |x: f32| x.to_bits();
    unsafe {
        hm(c.mode as u32);
        for v in [c.transform.interest, c.transform.target_interest, c.transform.position, c.transform.target_position] {
            hm(b(v.x));
            hm(b(v.y));
            hm(b(v.z));
        }
        hm(b(c.transform.fov));
        hm(b(c.transform.target_fov));
        hm(c.x341_b1_b2() as u32);
        hm(c.x341_b3_b4() as u32);
        hm(c.x341_b5_b6() as u32);
        hm(c.x341_b7() as u32);
        match c.x341_b1_b2() {
            1 => hm(c.x344.slot as u32),
            2 => {
                hm(b(c.x344.vec.x));
                hm(b(c.x344.vec.y));
                hm(b(c.x344.vec.z));
            }
            3 => hm(match c.x344.cb {
                Some(f) if f as usize == cb_a as usize => 1,
                None => 0,
                _ => 2,
            }),
            _ => {}
        }
        hm(b(c.x350.x));
        hm(b(c.x350.y));
        hm(b(c.x350.z));
        match c.x341_b3_b4() {
            2 => {
                hm(c.x35C.bits.b0() as u32);
                hm(c.x35C.bits.b1() as u32);
                hm(c.x35C.bits.b2() as u32);
                // the axis values are only meaningful while their flag is set (the rest is stale pointer bytes)
                if c.x35C.bits.b0() != 0 {
                    hm(c.x35C.bits.x2 as i32 as u32);
                }
                if c.x35C.bits.b1() != 0 {
                    hm(b(c.x35C.vec.y));
                }
                if c.x35C.bits.b2() != 0 {
                    hm(b(c.x35C.vec.z));
                }
            }
            3 => hm(match c.x35C.cb {
                Some(f) if f as usize == cb_b as usize => 1,
                None => 0,
                _ => 2,
            }),
            _ => {
                // x35C.vec.x aliases the bitfield bytes (laid out differently natively than on the GC)
                if c.x341_b3_b4() == 1 {
                    hm(b(c.x35C.vec.y));
                    hm(b(c.x35C.vec.z));
                } // mode 0: stale (maybe pointer) bytes
            }
        }
        hm(*(&c.x368 as *const Vec3 as *const i16) as i32 as u32);
        hm(b(c.x368.y));
        hm(b(c.x368.z));
        if c.x341_b3_b4() != 2 {
            hm(b(c.x368.x));
        }
        hm(b(c.x374));
        hm(c.x378.s32_v as u32);
        hm(c.x37C.s32_v as u32);
        hm(CBLOG);
    }
    h
}

#[test]
fn camera_boss_matches_c() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/camera_boss_ref.txt")).unwrap();
    let mut lines = text.lines();
    let mut w = CameraWorld::default();
    w.subject_bone_pos = Some(subject);
    let mut n = 0;
    for op in 0..11u32 {
        for k in 0..500u32 {
            w.game_camera = Camera::zeroed();
            let mut rg = Rng(0xCA3u32.wrapping_add(op.wrapping_mul(7919)).wrapping_add(k.wrapping_mul(104729)));
            unsafe { CBLOG = 0 };
            let mut ret: u32 = 0;
            {
                let c = &mut w.game_camera;
                c.mode = if rg.rnd() % 3 == 0 { CAMERA_BOSS_INTRO } else { rg.rnd() % 9 };
                c.transform.interest = rg.rv();
                c.transform.target_interest = rg.rv();
                c.transform.position = rg.rv();
                c.transform.target_position = rg.rv();
                c.transform.fov = rg.rf();
                c.transform.target_fov = rg.rf();
                c.set_x341_b1_b2((rg.rnd() % 4) as u8);
                c.set_x341_b3_b4((rg.rnd() % 4) as u8);
                c.set_x341_b5_b6((rg.rnd() % 4) as u8);
                c.set_x341_b7((rg.rnd() & 1) as u8);
                unsafe {
                    if c.x341_b1_b2() == 3 {
                        c.x344.cb = Some(core::mem::transmute::<unsafe extern "C" fn(*mut Vec3) -> bool, unsafe extern "C" fn(*mut Vec3) -> i32>(cb_a));
                    } else if c.x341_b1_b2() == 1 {
                        c.x344.slot = (rg.rnd() % 6) as i32;
                    } else {
                        c.x344.vec = rg.rv();
                    }
                    c.x350 = rg.rv();
                    if c.x341_b3_b4() == 2 {
                        c.x35C.bits.set_b0((rg.rnd() & 1) as u8);
                        c.x35C.bits.set_b1((rg.rnd() & 1) as u8);
                        c.x35C.bits.set_b2((rg.rnd() & 1) as u8);
                        c.x35C.bits.x2 = rg.rnd() as u16 as i16;
                        c.x35C.vec.y = rg.rf();
                        c.x35C.vec.z = rg.rf();
                    } else if c.x341_b3_b4() == 3 {
                        c.x35C.cb = Some(core::mem::transmute::<unsafe extern "C" fn(*mut Vec3) -> bool, unsafe extern "C" fn(*mut Vec3) -> i32>(cb_b));
                    } else {
                        c.x35C.vec = rg.rv();
                    }
                    c.x368 = rg.rv();
                    c.x374 = rg.rf();
                    if c.x341_b5_b6() == 1 {
                        c.x378.s32_v = (rg.rnd() % 20) as i32;
                        c.x37C.s32_v = if rg.rnd() % 4 == 0 { c.x378.s32_v } else { (rg.rnd() % 20) as i32 };
                    } else {
                        c.x378.f32_v = rg.rf();
                        c.x37C.f32_v = rg.rf();
                    }
                    for i in 0..6 {
                        let m = rg.rnd() % 3;
                        let pos = rg.rv();
                        SUBJ[i] = if m == 2 { Some(pos) } else { None };
                    }
                    CBV[0] = rg.rv();
                    CBV[1] = rg.rv();
                    CBRET[0] = rg.rnd() & 1 != 0;
                    CBRET[1] = rg.rnd() & 1 != 0;
                }
            }
            if std::env::var("DUMP").map_or(false, |d| d == format!("{op} {k}")) {
                let c = &w.game_camera;
                println!("FILL mode={} b1={} b3={} b5={} b7={} y={:08X} z={:08X} x={:08X}", c.mode, c.x341_b1_b2(), c.x341_b3_b4(), c.x341_b5_b6(), c.x341_b7(), unsafe { c.x35C.vec.y.to_bits() }, unsafe { c.x35C.vec.z.to_bits() }, unsafe { c.x35C.vec.x.to_bits() });
            }
            match op {
                0 => w.camera_8002e6fc((rg.rnd() % 6) as i32),
                1 => w.camera_8002e948(cb_a),
                2 => w.camera_8002eb5c(rg.rf()),
                3 => w.camera_8002ec7c(rg.rf()),
                4 => w.camera_8002ed9c(rg.rf()),
                5 => w.camera_8002eec8(rg.rf()),
                6 => w.camera_8002ef14(),
                7 => w.camera_8002f0e4((rg.rnd() % 30) as i32),
                8 => ret = w.camera_8002f260() as u32,
                9 => w.camera_8002f274(),
                10 => ret = w.camera_8002e234() as u32,
                _ => unreachable!(),
            }
            let dump = std::env::var("DUMP").map_or(false, |d| d == format!("{op} {k}"));
            if dump {
                print!("DUMP");
            }
            let got = format!("{} {} {:08X} {:08X}", op, k, ret, state(&w.game_camera, dump));
            let want = lines.next().expect("expected file too short");
            assert_eq!(got, want, "op {op} case {k}");
            n += 1;
        }
    }
    assert!(lines.next().is_none());
    assert_eq!(n, 5500);
}
