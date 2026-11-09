//! src/tu/ftco_800c7590.rs vs the C (gen/cref/tu_ftco_800c7590_ref.c includes
//! src/melee/ft/ftCo_800C7590.c whole, shipping contraction + gwtool lowering; expected values in
//! tests/data/tu_ftco_800c7590.txt; regenerate with
//! `python rust/melee_fighter/gen/cref/run.py tu_ftco_800c7590`). Same recorders, same inputs.
#![allow(static_mut_refs)]

use melee_fighter::tu::ftco_800c7070::ext as e70;
use melee_fighter::tu::ftco_800c7590 as m;
use melee_fighter::tu::ftco_800c7590::ext as e;
use melee_fighter::tu::ftco_800c78b0::ext as e8b;
use melee_fighter::types::*;

static mut RS: u32 = 0;
fn rnd() -> u32 {
    unsafe {
        RS ^= RS << 13;
        RS ^= RS >> 17;
        RS ^= RS << 5;
        RS
    }
}
fn rg() -> f32 {
    match rnd() % 16 {
        0 => 0.0,
        1 => -0.0,
        2 => f32::from_bits(0x7FC0_0000),
        3 => 1.0,
        4 => -1.0,
        _ => ((rnd() % 41) as i32 - 20) as f32 / 20.0,
    }
}
fn rw() -> f32 {
    const K: [f32; 4] = [1.0, 13.0, 1000.0, 100000.0];
    let a = rg();
    a * K[(rnd() % 4) as usize]
}

static mut TR: u32 = 0;
fn t(v: u32) {
    unsafe { TR = TR.wrapping_mul(31).wrapping_add(v) }
}
fn b(f: f32) -> u32 {
    f.to_bits()
}
static mut GO: *mut HSD_GObj = core::ptr::null_mut();
static mut ITEM: *mut HSD_GObj = core::ptr::null_mut();
static mut FP: *mut Fighter = core::ptr::null_mut();
static mut RET_BB8: i32 = 0;
static mut RET_HC: i32 = 0;
static mut MASH: f32 = 0.0;
fn isg(g: *mut HSD_GObj) -> u32 {
    (g == unsafe { GO }) as u32
}
fn isf(f: &Fighter) -> u32 {
    (f as *const Fighter == unsafe { FP } as *const Fighter) as u32
}
fn isi(i: *mut HSD_GObj) -> u32 {
    (i == unsafe { ITEM }) as u32
}

fn install() {
    unsafe {
        e70::FIGHTER_CHANGE_MOTION_STATE = Some(|g, ms, fl, a, bb, c, o| {
            t(1); t(isg(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32);
        });
        e::FTCOMMON_8007DB58 = Some(|g| { t(2); t(isg(g)); });
        e::FTCO_8009750C = Some(|g| { t(3); t(isg(g)); });
        e::FTCO_800DD168 = Some(|g| { t(4); t(isg(g)); });
        e::FTCO_8008DCE0 = Some(|g, a, f| { t(5); t(isg(g)); t(a as u32); t(b(f)); });
        e::FTCOMMON_8007D7FC = Some(|f| { t(6); t(isf(f)); });
        e::FTCOMMON_8007E2FC = Some(|f| { t(7); t(isf(f)); });
        e::PLAYER_80033BB8 = Some(|s| { t(8); t(s as u32); RET_BB8 });
        e::PLAYER_GET_HANDICAP = Some(|s| { t(9); t(s as u32); RET_HC });
        e::FTCOMMON_INIT_GRAB = Some(|f, a, tm| { t(10); t(isf(f)); t(a as u32); t(b(tm)); });
        e::FTCOMMON_8007E2F4 = Some(|f, v| { t(11); t(isf(f)); t(v as i32 as u32); });
        e::IT_802EAAEC = Some(|i, g, a, f| { t(12); t(isi(i)); t(isg(g)); t(a as u32); t(b(f)); });
        e::FTCOMMON_8007EBAC = Some(|f, a, bb| { t(13); t(isf(f)); t(a); t(bb); });
        e::CAMERA_REQUEST_QUAKE = Some(|k, p| {
            t(14); t(k);
            // the C passes &fp->cur_pos; the port passes a copy: compare the value
            t((p.x.to_bits() == (*FP).cur_pos.x.to_bits() && p.y.to_bits() == (*FP).cur_pos.y.to_bits()) as u32);
        });
        e::FTCOMMON_GRAB_MASH = Some(|f, tm| { t(15); t(isf(f)); t(b(tm)); f.grab_timer -= MASH; false });
        e::IT_802EAE80 = Some(|i| { t(16); t(isi(i)); });
        e::FT_8008403C = Some(|g, cb| { t(18); t(isg(g)); t(cb.is_some() as u32); });
        e::FTCO_80090780 = Some(|g| { t(19); t(isg(g)); });
        e8b::IT_802DBA68 = Some(|i| { t(20); t(isi(i)); });
    }
}

struct Hs(u32);
impl Hs {
    fn m(&mut self, v: u32) {
        self.0 = self.0.wrapping_mul(0x0100_0193) ^ v;
    }
}
fn fhash(f: &Fighter) -> u32 {
    let mut s = Hs(0x811C_9DC5);
    s.m(b(f.grab_timer)); s.m(b(f.gr_vel)); s.m(b(f.self_vel.x)); s.m(f.capture_timer as u32);
    s.m(f.x2219_b1() as u32); s.m(f.x2219_b2() as u32); s.m(f.x221D_b5() as u32); s.m(f.x2220_b3() as u32);
    s.m(isi(unsafe { f.mv.co.captureleadead.x0 })); s.m(f.take_dmg_cb.is_some() as u32); s.m(f.death2_cb.is_some() as u32);
    s.0
}

#[test]
fn matches_c() {
    let data = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftco_800c7590.txt")).unwrap();
    let mut go = Box::new(HSD_GObj::zeroed());
    let mut item = Box::new(HSD_GObj::zeroed());
    let mut f = Box::new(Fighter::zeroed());
    let mut cd = Box::new(ftCommonData::zeroed());
    go.user_data = &mut *f as *mut Fighter as *mut _;
    unsafe {
        GO = &mut *go;
        ITEM = &mut *item;
        FP = &mut *f;
        melee_fighter::hooks::P_FT_COMMON_DATA = &*cd;
    }
    install();
    let (mut n, mut bad) = (0, 0);
    for line in data.lines() {
        let w: std::vec::Vec<&str> = line.split_whitespace().collect();
        let (op, k): (u32, u32) = (w[0].parse().unwrap(), w[1].parse().unwrap());
        let ftr = unsafe { &mut *FP };
        *ftr = Fighter::zeroed();
        *cd = ftCommonData::zeroed();
        unsafe {
            TR = 0;
            RS = 0x7590u32.wrapping_add(op.wrapping_mul(7919)).wrapping_add(k.wrapping_mul(104729));
        }
        cd.x71C = rw(); cd.x720 = rw(); cd.x724 = rw(); cd.x728 = rw(); cd.x72C = rw(); cd.x730 = rw();
        cd.leadead_grab_timer_step = rw(); cd.leadead_grab_break_threshold = rw();
        cd.x370 = rw(); cd.x73C = rnd() as i32;
        ftr.player_idx = (rnd() % 6) as u8;
        ftr.dmg.x1830_percent = rw();
        ftr.dmg.x1868_source = unsafe { ITEM };
        ftr.mv.co.captureleadead.x0 = unsafe { ITEM };
        ftr.grab_timer = rw();
        ftr.facing_dir = rg();
        ftr.ground_or_air = rnd() % 2;
        unsafe {
            RET_BB8 = (rnd() % 9) as i32 - 4;
            RET_HC = (rnd() % 12) as i32;
            MASH = rw();
        }
        let g = unsafe { GO };
        match op {
            0 => m::ftco_800c7590(g),
            1 => m::ftco_capture_leadead_anim(g),
            _ => {
                m::ftco_capture_leadead_coll(g);
                m::ftco_800c7800(g);
                unsafe { m::fn_800c7890(g) };
            }
        }
        let got = format!("{:08x} {:08x}", unsafe { TR }, fhash(unsafe { &*FP }));
        let want = format!("{} {}", w[2], w[3]);
        n += 1;
        if got != want {
            if bad < 5 {
                eprintln!("op {op} case {k}: got {got} want {want}");
            }
            bad += 1;
        }
    }
    assert_eq!(n, 9000);
    assert_eq!(bad, 0, "{bad} of {n} differ");
}
