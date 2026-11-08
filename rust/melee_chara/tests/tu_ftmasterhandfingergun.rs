//! tu::ftmasterhandfingergun vs the C (gen/turef/tu_ftmasterhandfingergun_ref.c: the game TU
//! #included verbatim with lbVector_Diff/Normalize cut from lbvector.c, shipping contraction; data
//! in tests/data/tu_ftmasterhandfingergun.txt; regenerate with `python
//! rust/melee_chara/gen/turef/tu_shard369.py tu_ftmasterhandfingergun`). 4000 FingerGun1_Phys
//! velocities, 400 MS_363 inits, 400 FingerGun1_Anim steps, 100 FingerGun2_Anim steps and 100
//! bullet spawns, bit for bit.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftmasterhandfingergun as t;
use melee_fighter::hooks as fh;
use melee_types::*;

static mut RANDI: i32 = 0;
static mut FRAMES: i32 = 0;
static mut PERCENT: i32 = 0;
static mut RATE_SET: i32 = 0;
static mut RATE: f32 = -1.0;
static mut TGT: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
static mut BUL: [f32; 8] = [0.0; 8];
static mut ACB: i32 = 0;
fn hex(s: &str) -> u32 {
    u32::from_str_radix(s, 16).unwrap()
}
fn fl(s: &str) -> f32 {
    f32::from_bits(hex(s))
}

#[test]
fn matches_c_and_smoke() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftmasterhandfingergun.txt")).unwrap();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut fd: Box<ftData> = Box::new(unsafe { core::mem::zeroed() });
    let mut da = Box::new(ftMasterHand_SpecialAttrs::zeroed());
    let mut parts: std::vec::Vec<FighterBone> = (0..64).map(|_| FighterBone::zeroed()).collect();
    fd.ext_attr = &mut *da as *mut _ as *mut _;
    f.ft_data = &mut *fd;
    f.parts = parts.as_mut_ptr();
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FT_80085134 = Some(|_| {});
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, m, _, _, _, _, _| {
            if m == 365 {
                ACB += 1;
            }
        });
        st::FTANIM_8006EBA4 = Some(|_| {});
        st::FTANIM_ISFRAMESREMAINING = Some(|_| FRAMES);
        st::FTANIM_SETANIMRATE = Some(|_, r| {
            RATE = r;
            RATE_SET += 1;
        });
        fh::HSD_RANDI = Some(|_| RANDI);
        melee_chara::tu::ftmasterhandpoke::ext::FTBOSSLIB_8015C208 = Some(|_, out| unsafe { *out = TGT });
        melee_chara::tu::ftmasterhanddamage0::ext::FTMH_MS_389_80151018 = Some(|_| {});
        t::ext::FTLIB_GETPERCENT = Some(|_| PERCENT);
        melee_chara::tu::ftmasterhanddamage0::ext::LB_8000B1CC = Some(|_, p0, p1| unsafe {
            assert!(p0.is_null());
            *p1 = Vec3 { x: 1.5, y: -2.25, z: 0.5 };
        });
        t::ext::IT_802F0AE0 = Some(|_, prev, pos, kind, msid, facing, vx, vy| unsafe {
            assert_eq!(kind, It_Kind_MasterHand_Bullet);
            BUL = [(*prev).x, (*prev).y, (*pos).x, (*pos).y, msid as f32, facing, vx, vy];
        });
    }
    let mut counts = [0usize; 5];
    for (i, line) in want.lines().enumerate() {
        let w: std::vec::Vec<&str> = line.split(' ').collect();
        match w[0] {
            "init" => {
                da.xE4 = fl(w[1]);
                da.xE8 = fl(w[2]);
                unsafe { RANDI = w[3].parse().unwrap() };
                f.cmd_vars = [9, 9, 9, 0];
                t::ftMh_MS_363_801530A4(gp);
                let got = unsafe { (f.mv.mh.unk0.x50.to_bits(), f.cmd_vars[0], f.cmd_vars[1], f.cmd_vars[2]) };
                assert_eq!(got, (hex(w[4]), w[5].parse().unwrap(), w[6].parse().unwrap(), w[7].parse().unwrap()), "init line {i}");
                counts[1] += 1;
            }
            "a1" => {
                f.mv.mh.unk0.x50 = fl(w[1]);
                da.xEC = w[2].parse().unwrap();
                da.xF0 = w[3].parse().unwrap();
                unsafe { PERCENT = w[4].parse().unwrap() };
                da.xF4 = fl(w[5]);
                unsafe { FRAMES = 0 };
                f.mv.mh.unk0.x54 = 77;
                f.cmd_vars[2] = 5;
                f.self_vel.x = 3.0;
                f.self_vel.y = 4.0;
                unsafe {
                    RATE_SET = 0;
                    RATE = -1.0;
                }
                t::ftMh_FingerGun1_Anim(gp);
                let got = unsafe { (f.mv.mh.unk0.x50.to_bits(), f.mv.mh.unk0.x54, f.cmd_vars[2], RATE_SET, RATE.to_bits(), f.self_vel.x.to_bits(), f.self_vel.y.to_bits()) };
                let exp = (hex(w[6]), w[7].parse().unwrap(), w[8].parse().unwrap(), w[9].parse().unwrap(), hex(w[10]), hex(w[11]), hex(w[12]));
                assert_eq!(got, exp, "a1 line {i}");
                counts[2] += 1;
            }
            "a2" => {
                f.mv.mh.unk0.x54 = w[1].parse().unwrap();
                da.xEC = w[2].parse().unwrap();
                unsafe { PERCENT = w[3].parse().unwrap() };
                da.xF4 = fl(w[4]);
                unsafe {
                    FRAMES = 0;
                    RATE_SET = 0;
                    RATE = -1.0;
                    ACB = 0;
                }
                t::ftMh_FingerGun2_Anim(gp);
                let got = unsafe { (f.mv.mh.unk0.x54, RATE_SET, RATE.to_bits()) };
                assert_eq!(got, (w[5].parse().unwrap(), w[6].parse().unwrap(), hex(w[7])), "a2 line {i}");
                counts[3] += 1;
            }
            "bl" => {
                da.xEC = w[1].parse().unwrap();
                unsafe { PERCENT = w[2].parse().unwrap() };
                da.xF8 = fl(w[3]);
                da.xFC = fl(w[4]);
                da.xD4 = fl(w[5]);
                da.xD8 = fl(w[6]);
                f.facing_dir = fl(w[7]);
                f.cmd_vars[0] = 1;
                f.cmd_vars[1] = 0;
                unsafe { t::ftMh_MS_365_801535B0(gp) };
                let got = unsafe { BUL.map(f32::to_bits) };
                // C order: prev.x prev.y pos.x pos.y msid facing (vx, vy are the attrs xD4/xD8)
                let exp = [hex(w[8]), hex(w[9]), hex(w[10]), hex(w[11]), hex(w[12]), hex(w[13])];
                assert_eq!(&got[..6], &exp, "bullet line {i}");
                assert_eq!((got[6], got[7]), (da.xD4.to_bits(), da.xD8.to_bits()), "bullet vel {i}");
                assert_eq!(f.cmd_vars[0], w[14].parse::<u32>().unwrap(), "bullet flag {i}");
                counts[4] += 1;
            }
            _ => {
                let tgt = Vec3 { x: fl(w[1]), y: fl(w[2]), z: 5.0 };
                let cur = Vec3 { x: fl(w[3]), y: fl(w[4]), z: fl(w[9]) };
                da.x2C = fl(w[5]);
                da.x28 = fl(w[6]);
                da.xDC = fl(w[7]);
                da.xE0 = fl(w[8]);
                let (vx, vy) = t::fingergun1_vel(&tgt, &cur, da.xDC, da.xE0, da.x2C, da.x28);
                assert_eq!((vx.to_bits(), vy.to_bits()), (hex(w[10]), hex(w[11])), "phys line {i}");
                // and through the real callback
                unsafe { TGT = tgt };
                f.cur_pos = cur;
                f.cmd_vars[2] = 1;
                f.self_vel.x = 12345.0;
                f.self_vel.y = 12345.0;
                t::ftMh_FingerGun1_Phys(gp);
                assert_eq!((f.self_vel.x.to_bits(), f.self_vel.y.to_bits()), (hex(w[10]), hex(w[11])), "phys cb line {i}");
                counts[0] += 1;
            }
        }
    }
    assert_eq!(counts, [4000, 400, 400, 100, 100]);
    // Phys leaves the velocity alone until the Anim callback has raised cmd_vars[2]
    f.cmd_vars[2] = 0;
    f.self_vel.x = 9.0;
    t::ftMh_FingerGun1_Phys(gp);
    assert_eq!(f.self_vel.x, 9.0);
}
