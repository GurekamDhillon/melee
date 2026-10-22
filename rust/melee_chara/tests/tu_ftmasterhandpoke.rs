//! tu::ftmasterhandpoke vs the C (gen/turef/tu_ftmasterhandpoke_ref.c: the game TU #included
//! verbatim with lbVector_Diff/Normalize cut from lbvector.c, shipping contraction; data in
//! tests/data/tu_ftmasterhandpoke.txt; regenerate with `python
//! rust/melee_chara/gen/turef/tu_shard369.py tu_ftmasterhandpoke`). 4000 Poke1_Phys velocities
//! (fused length, Newton sqrt, normalize) and 400 Init / Poke1_Anim countdown steps, bit for bit.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftmasterhandpoke as t;
use melee_fighter::hooks as fh;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
static mut RANDI: i32 = 0;
static mut FRAMES: i32 = 0;
static mut TGT: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
fn hex(s: &str) -> u32 {
    u32::from_str_radix(s, 16).unwrap()
}
fn fl(s: &str) -> f32 {
    f32::from_bits(hex(s))
}

#[test]
fn matches_c_and_smoke() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftmasterhandpoke.txt")).unwrap();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut fd: Box<ftData> = Box::new(unsafe { core::mem::zeroed() });
    let mut da = Box::new(ftMasterHand_SpecialAttrs::zeroed());
    fd.ext_attr = &mut *da as *mut _ as *mut _;
    f.ft_data = &mut *fd;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FT_80085134 = Some(|_| {});
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, m, _, _, _, _, _| LOG.push(format!("ms {m}")));
        st::FTANIM_8006EBA4 = Some(|_| {});
        st::FTANIM_ISFRAMESREMAINING = Some(|_| FRAMES);
        fh::HSD_RANDI = Some(|_| RANDI);
        fh::PLAY_SFX = Some(|_, id, v, p| LOG.push(format!("sfx {id} {v} {p}")));
        melee_chara::tu::ftmasterhandpoke::ext::FTBOSSLIB_8015C208 = Some(|_, out| unsafe { *out = TGT });
        melee_chara::tu::ftmasterhanddamage0::ext::FTMH_MS_389_80151018 = Some(|_| LOG.push("wait".into()));
    }
    let (mut nphys, mut ninit) = (0, 0);
    for (i, line) in want.lines().enumerate() {
        let w: std::vec::Vec<&str> = line.split(' ').collect();
        if w[0] == "init" {
            da.x90 = w[1].parse().unwrap();
            da.x94 = w[2].parse().unwrap();
            unsafe { RANDI = w[3].parse().unwrap() };
            t::ftMh_MS_358_80152880(gp);
            let (x0, x4) = unsafe { (f.mv.mh.unk13.x0, f.mv.mh.unk13.x4) };
            assert_eq!((x0.to_bits(), x4.to_bits()), (hex(w[4]), hex(w[5])), "init line {i}");
            // w[6] = "in", w[7] = countdown fed to Anim, w[8] = "anim"
            unsafe { FRAMES = 0 };
            f.self_vel.x = 7.0;
            f.self_vel.y = 8.0;
            f.mv.mh.unk13.x0 = fl(w[7]);
            unsafe { LOG.clear() };
            t::ftMh_Poke1_Anim(gp);
            let got = unsafe { (f.mv.mh.unk13.x0.to_bits(), f.mv.mh.unk13.x4.to_bits(), f.self_vel.x.to_bits(), f.self_vel.y.to_bits()) };
            assert_eq!(got, (hex(w[9]), hex(w[10]), hex(w[11]), hex(w[12])), "anim line {i}");
            // the countdown ran out exactly when the motion changed to Poke2 and its sound played
            assert_eq!(unsafe { LOG.join(",") }, if fl(w[9]) < 0.0 { "ms 360,sfx 320007 127 64" } else { "" }, "anim log {i}");
            ninit += 1;
        } else {
            let tgt = Vec3 { x: fl(w[1]), y: fl(w[2]), z: 5.0 };
            let cur = Vec3 { x: fl(w[3]), y: fl(w[4]), z: fl(w[9]) };
            da.x2C = fl(w[5]);
            da.x28 = fl(w[6]);
            da.x98 = fl(w[7]);
            da.x9C = fl(w[8]);
            let (vx, vy) = t::poke1_vel(&tgt, &cur, da.x98, da.x9C, da.x2C, da.x28);
            assert_eq!((vx.to_bits(), vy.to_bits()), (hex(w[10]), hex(w[11])), "phys line {i}");
            // and through the real callback
            unsafe { TGT = tgt };
            f.cur_pos = cur;
            f.mv.mh.unk13.x4 = 1.0;
            f.self_vel.x = 12345.0;
            f.self_vel.y = 12345.0;
            t::ftMh_Poke1_Phys(gp);
            assert_eq!((f.self_vel.x.to_bits(), f.self_vel.y.to_bits()), (hex(w[10]), hex(w[11])), "phys cb line {i}");
            nphys += 1;
        }
    }
    assert_eq!((nphys, ninit), (4000, 400));
    // Phys leaves the velocity alone until the Poke1 countdown flagged it
    f.mv.mh.unk13.x4 = 0.0;
    f.self_vel.x = 9.0;
    t::ftMh_Poke1_Phys(gp);
    assert_eq!(f.self_vel.x, 9.0);
    // Poke2 waits for the end of the motion
    unsafe { FRAMES = 1 };
    unsafe { LOG.clear() };
    t::ftMh_Poke2_Anim(gp);
    assert!(unsafe { LOG.is_empty() });
    unsafe { FRAMES = 0 };
    t::ftMh_Poke2_Anim(gp);
    assert_eq!(unsafe { LOG.join(",") }, "wait");
}
