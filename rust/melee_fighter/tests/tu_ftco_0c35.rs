//! tu::ftco_0c35 vs the C (gen/cref/tu_ftco_0c35_ref.c, shipping contraction; data in
//! tests/data/tu_ftco_0c35.txt; regenerate with `python rust/melee_fighter/gen/cref/tu_shard97.py`).
//! The trace word TR folds every stubbed call and its arguments, in the C's order.
#![allow(static_mut_refs)]

use melee_fighter::hooks as h;
use melee_fighter::tu::ftco_0c35::{self as t, ext};
use melee_fighter::types::*;

static mut TR: u32 = 0;
static mut ANG: f32 = 0.0;
static mut BONE: i32 = 0;
static mut GO: *mut HSD_GObj = core::ptr::null_mut();
static mut FP: *mut Fighter = core::ptr::null_mut();
static mut JP: *mut HSD_JObj = core::ptr::null_mut();

fn tt(v: u32) {
    unsafe { TR = TR.wrapping_mul(31).wrapping_add(v) }
}
fn isf(fp: &Fighter) -> u32 {
    (fp as *const Fighter == unsafe { FP } as *const Fighter) as u32
}

unsafe extern "C" fn cb21f8(g: *mut HSD_GObj) {
    tt(7);
    tt((g == GO) as u32);
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
        match self.rnd() % 10 {
            0 => 0.0,
            1 => -0.0,
            2 => 1.0,
            3 => -1.0,
            _ => ((self.rnd() % 20001) as i32 - 10000) as f32 / 1013.0,
        }
    }
}

#[test]
fn matches_c() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftco_0c35.txt")).unwrap();
    let f: &'static mut Fighter = Box::leak(Box::new(Fighter::zeroed()));
    let go: &'static mut HSD_GObj = Box::leak(Box::new(unsafe { core::mem::zeroed::<HSD_GObj>() }));
    let j: &'static mut HSD_JObj = Box::leak(Box::new(unsafe { core::mem::zeroed::<HSD_JObj>() }));
    let pb: &'static mut [FighterBone; 8] = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    let cd: &'static mut ftCommonData = Box::leak(Box::new(ftCommonData::zeroed()));
    unsafe {
        GO = go;
        FP = f;
        JP = j;
        (*GO).user_data = FP.cast();
        (*FP).parts = pb.as_mut_ptr();
        h::P_FT_COMMON_DATA = cd;
        ext::FIGHTER_UNK_RECURSIVE_FUNC_8006D044 = Some(|g| { tt(1); tt((g == GO) as u32) });
        h::FTCO_DAMAGE_CALC_ANGLE = Some(|fp, kb| { tt(2); tt(isf(fp)); tt(kb.to_bits()); ANG });
        ext::FTCO_DAMAGE_CALC_VEL = Some(|fp, x, y| { tt(3); tt(isf(fp)); tt(x.to_bits()); tt(y.to_bits()) });
        h::FTPARTS_GET_BONE_INDEX = Some(|fp, p| { tt(4); tt(isf(fp)); tt(p as u32); BONE });
        ext::FT_PART_SET_ROT_Y = Some(|fp, p, r| { tt(5); tt(isf(fp)); tt(p as u32); tt(r.to_bits()) });
        ext::HSD_JOBJ_SET_MTX_DIRTY_SUB = Some(|jj| { tt(6); tt((jj == JP) as u32) });
    }
    let (f, cd, j) = unsafe { (&mut *FP, &mut *(h::P_FT_COMMON_DATA as *mut ftCommonData), &mut *JP) };
    let mut r = Rng(0x51ED2701);
    let mut n = 0;
    for (i, line) in want.lines().enumerate() {
        let op = r.rnd() % 3;
        unsafe { TR = 0 };
        cd.x648 = (r.rnd() % 40) as i32 - 2;
        cd.x64C = r.rf();
        cd.x650 = r.rf();
        cd.x654 = r.rf();
        unsafe { ANG = r.rf() };
        let bone = (r.rnd() % 8) as i32;
        unsafe { BONE = bone };
        f.ground_or_air = r.rnd() % 2;
        f.dmg.x18F4 = (r.rnd() % 42) as i32 - 2;
        f.dmg.x1954 = r.rf();
        f.dmg.kb_applied = r.rf();
        f.dmg.facing_dir_1 = r.rf();
        f.dmg.x18c4_source_ply = 0;
        f.dmg.x18C8 = 0;
        f.specialn_facing_dir = r.rf();
        f.coll_data.floor.normal.x = r.rf();
        f.coll_data.floor.normal.y = r.rf();
        f.facing_dir = r.rf();
        f.xF0_ground_kb_vel = r.rf();
        let b4 = (r.rnd() % 2) as u8;
        f.set_x2220_b4(b4);
        f.x21F8 = if r.rnd() % 2 != 0 { Some(cb21f8) } else { None };
        j.flags = r.rnd() & (JOBJ_MTX_INDEP_SRT | JOBJ_MTX_DIRTY | JOBJ_USER_DEF_MTX) as u32;
        j.rotate.y = 7.0;
        pb[bone as usize].x4_jobj2 = j;
        let g = unsafe { GO };
        match op {
            0 => t::ftco_800c3598(g),
            1 => t::ftco_800c36dc(g),
            _ => t::ftco_800c37a0(g),
        }
        let got = format!(
            "{} {} {:08X} {} {:08X} {} {:08X} {:08X} {:08X} {} {}",
            i, op, unsafe { TR }, f.dmg.x18F4, f.dmg.x1954.to_bits(), f.x2220_b4(), f.xF0_ground_kb_vel.to_bits(),
            f.facing_dir.to_bits(), j.rotate.y.to_bits(), f.dmg.x18c4_source_ply, f.dmg.x18C8
        );
        assert_eq!(got, line, "case {i}");
        n += 1;
    }
    assert_eq!(n, 6000);
}
