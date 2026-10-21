//! Shared driver for tests/tu_ftCo_CapturePulled.rs and tests/tu_ftCo_CaptureKirby.rs: replays
//! gen/cref/tu_shard121_ref.c's generator against the ports and compares with
//! tests/data/tu_shard121.txt (regenerate: `python rust/melee_fighter/gen/cref/tu_shard121.py`).
#![allow(static_mut_refs, dead_code)]

use melee_fighter::tu::{ftCo_CaptureKirby as kb, ftCo_CapturePulled as cp};
use melee_fighter::types::*;

struct W {
    cd: ftCommonData,
    f: Fighter,
    v: Fighter,
    g: HSD_GObj,
    vg: HSD_GObj,
    j: HSD_JObj,
    hold: HSD_JObj,
    xrot: HSD_JObj,
    pb: [FighterBone; 4],
}
static mut WP: *mut W = core::ptr::null_mut();
static mut SLOT: i32 = 0;
static mut HC: i32 = 0;
static mut LBN: usize = 0;
static mut DIRTY: i32 = 0;
static mut LBV: [Vec3; 2] = [Vec3 { x: 0.0, y: 0.0, z: 0.0 }; 2];
static mut KBV: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

struct Rng(u32);
impl Rng {
    fn rnd(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    fn rf(&mut self) -> f32 {
        match self.rnd() % 10 {
            0 => 0.0,
            1 => -0.0,
            2 => 1.0,
            3 => -1.0,
            _ => ((self.rnd() % 20001) as i32 - 10000) as f32 / 1013.0,
        }
    }
}

/// Runs every case; `want_op(op)` picks which lines are compared. Returns the number compared.
pub fn run(want_op: fn(u32) -> bool) -> usize {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_shard121.txt")).unwrap();
    let mut lines = want.lines();
    let w: &mut W = Box::leak(Box::new(unsafe { core::mem::zeroed::<W>() }));
    unsafe {
        WP = w;
        melee_fighter::hooks::P_FT_COMMON_DATA = &w.cd;
        cp::ext::hook::Player_80033BB8 = Some(|_| SLOT);
        cp::ext::hook::Player_GetHandicap = Some(|_| HC);
        cp::ext::hook::ftParts_GetBoneIndex = Some(|_, _| 1);
        cp::ext::hook::lb_8000B1CC = Some(|_, _, out| {
            *out = LBV[LBN & 1];
            LBN += 1;
        });
        kb::ext::hook::ftKb_SpecialN_800F5B5C = Some(|_, out| *out = KBV);
        kb::ext::hook::HSD_JObjSetMtxDirtySub = Some(|_| DIRTY += 1);
    }
    w.g.user_data = (&mut w.f as *mut Fighter).cast();
    w.g.hsd_obj = (&mut w.j as *mut HSD_JObj).cast();
    w.vg.user_data = (&mut w.v as *mut Fighter).cast();
    w.f.victim_gobj = &mut w.vg;
    w.f.parts = w.pb.as_mut_ptr();
    w.pb[1].joint = &mut w.xrot;
    let g: *mut HSD_GObj = &mut w.g;
    let vg: *mut HSD_GObj = &mut w.vg;
    let mut r = Rng(0x121C0DE5);
    let mut n = 0;
    for i in 0..6000 {
        let op = r.rnd() % 7;
        let mut res: f32 = 0.0;
        unsafe {
            LBN = 0;
            DIRTY = 0;
        }
        let cd = &mut w.cd;
        cd.x354 = r.rf(); cd.x358 = r.rf(); cd.x35C = r.rf(); cd.x360 = r.rf(); cd.x364 = r.rf(); cd.x368 = r.rf();
        cd.x3C4 = r.rf();
        unsafe {
            SLOT = (r.rnd() % 6) as i32 - 1;
            HC = (r.rnd() % 10) as i32;
        }
        let f = &mut w.f;
        f.dmg.x1830_percent = r.rf();
        f.facing_dir = r.rf();
        f.x1A70.y = r.rf(); f.x1A70.z = r.rf();
        f.x34_scale.y = r.rf();
        f.cur_pos.x = r.rf(); f.cur_pos.y = r.rf(); f.cur_pos.z = r.rf();
        f.ground_or_air = if r.rnd() & 1 != 0 { GA_Ground } else { GA_Air };
        f.x2170 = r.rf();
        w.v.cur_pos.y = r.rf();
        w.v.mv.co.capturedamage.x18 = &mut w.hold;
        unsafe {
            LBV[0] = Vec3 { x: r.rf(), y: r.rf(), z: r.rf() };
            LBV[1] = Vec3 { x: r.rf(), y: r.rf(), z: r.rf() };
            KBV = Vec3 { x: r.rf(), y: r.rf(), z: r.rf() };
        }
        let k = unsafe { &mut w.f.mv.co.capturekirby };
        k.pos_offset.x = r.rf();
        k.pos_offset.y = r.rf();
        k.x8.x = r.rf();
        k.x8.y = r.rf();
        k.x10.x = r.rf();
        k.x10.y = r.rf();
        k.scale.x = r.rf();
        k.scale.y = r.rf();
        k.scale.z = r.rf();
        w.j.flags = r.rnd() & (JOBJ_MTX_DIRTY | JOBJ_USER_DEF_MTX | JOBJ_MTX_INDEP_SRT) as u32;
        w.j.scale = Vec3 { x: 7.0, y: 7.0, z: 7.0 };
        match op {
            0 => res = cp::ftCo_800DA824(&mut w.f),
            1 => cp::fn_800DAA40(g, vg),
            2 => res = cp::fn_800DAD18(g) as i32 as f32,
            3 => cp::ftCo_CapturePulledHi_Phys(g),
            4 => cp::ftCo_800DB464(g),
            5 => {
                let p = Vec3 { x: r.rf(), y: r.rf(), z: r.rf() };
                res = kb::ftCo_800BD19C(g, &p);
            }
            _ => kb::ftCo_800BD39C(g),
        }
        let line = lines.next().unwrap();
        if !want_op(op) {
            continue;
        }
        let k = unsafe { &w.f.mv.co.capturekirby };
        let got = format!(
            "{} {} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {}",
            i, op, res.to_bits(), w.f.cur_pos.x.to_bits(), w.f.cur_pos.y.to_bits(), w.f.cur_pos.z.to_bits(),
            w.v.x2170.to_bits(), k.pos_offset.x.to_bits(), k.pos_offset.y.to_bits(),
            w.j.scale.x.to_bits(), w.j.scale.y.to_bits(), w.j.scale.z.to_bits(), w.j.flags, unsafe { DIRTY }
        );
        assert_eq!(got, line, "case {i}");
        n += 1;
    }
    n
}
