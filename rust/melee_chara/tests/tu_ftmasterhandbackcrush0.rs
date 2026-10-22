//! tu::ftmasterhandbackcrush0 vs the C (gen/turef/tu_ftmasterhandbackcrush0_ref.c: the game TU
//! #included verbatim, shipping contraction; data in tests/data/tu_ftmasterhandbackcrush0.txt;
//! regenerate with `python rust/melee_chara/gen/turef/tu_shard361.py`). 2000 MS_370 scale setups
//! (int->float, `/= x7C`) each followed by 3 Anim ramp steps, bit for bit. MS_370 itself calls
//! the native ftBossLib_8015C208 (needs the fighter list), so the setup is checked through
//! `ms_370_rest` (everything after `cur_pos.x = pos.x`).
#![allow(static_mut_refs)]
#[path = "common_mh/mod.rs"]
mod common;
use common::*;
use melee_chara::tu::ftmasterhandbackcrush0 as t;
use melee_types::*;

fn bits3(v: Vec3) -> (u32, u32, u32) {
    (v.x.to_bits(), v.y.to_bits(), v.z.to_bits())
}

#[test]
fn matches_c_and_smoke() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftmasterhandbackcrush0.txt")).unwrap();
    let mut r = Rig::new();
    let gp = r.gp();
    let mut jobj: Box<HSD_JObj> = Box::new(unsafe { core::mem::zeroed() });
    jobj.flags = JOBJ_MTX_INDEP_SRT as u32; // SetScale skips the matrix-dirty call
    r.g.hsd_obj = &mut *jobj as *mut HSD_JObj as *mut _;
    hook_stubs(1);
    unsafe {
        t::ext::HSD_JOBJSETMTXDIRTYSUB = Some(|_| log("dirty".into()));
        melee_chara::tu::ftmasterhandbackairplane3::ext::FTMH_MS_389_80151018 = Some(|_| log("389".into()));
    }
    let (mut ninit, mut nanim) = (0, 0);
    for (i, line) in want.lines().enumerate() {
        let w: std::vec::Vec<&str> = line.split(' ').collect();
        if w[0] == "init" {
            // "init x70 x74 x78 x7C sx sy sz -> curx cury i70 x0 x64.. x58.. scl.x scl.y cmd"
            jobj.scale = Vec3 { x: fl(w[5]), y: fl(w[6]), z: fl(w[7]) };
            r.da.x70 = w[1].parse().unwrap();
            r.da.x74 = w[2].parse().unwrap();
            r.da.x78 = fl(w[3]);
            r.da.x7C = w[4].parse().unwrap();
            t::ms_370_rest(gp);
            let o = &w[9..];
            unsafe {
                assert_eq!(r.f.cur_pos.y.to_bits(), hex(o[1]), "init cur_y line {i}");
                assert_eq!(r.f.mv.mh.unk0.x70, o[2].parse::<i32>().unwrap(), "init x70 line {i}");
                assert_eq!(r.f.mv.mh.unk0.x0.to_bits(), hex(o[3]), "init x0 line {i}");
                assert_eq!(bits3(r.f.mv.mh.unk0.x64), (hex(o[4]), hex(o[5]), hex(o[6])), "init x64 line {i}");
                assert_eq!(bits3(r.f.mv.mh.unk0.x58), (hex(o[7]), hex(o[8]), hex(o[9])), "init x58 line {i}");
            }
            assert_eq!((jobj.scale.x.to_bits(), jobj.scale.y.to_bits()), (hex(o[10]), hex(o[11])), "init scale line {i}");
            assert_eq!(r.f.cmd_vars[0] as i32, o[12].parse::<i32>().unwrap());
            ninit += 1;
        } else {
            // "anim x70 cmd -> x70 cmd sx sy sz"
            unsafe { r.f.mv.mh.unk0.x70 = w[1].parse().unwrap() };
            r.f.cmd_vars[0] = w[2].parse().unwrap();
            t::ftMh_BackCrush_Anim(gp);
            assert_eq!(unsafe { r.f.mv.mh.unk0.x70 }, w[4].parse::<i32>().unwrap(), "anim x70 line {i}");
            assert_eq!(r.f.cmd_vars[0] as i32, w[5].parse::<i32>().unwrap(), "anim cmd line {i}");
            assert_eq!(bits3(jobj.scale), (hex(w[6]), hex(w[7]), hex(w[8])), "anim scale line {i}");
            nanim += 1;
        }
    }
    assert_eq!((ninit, nanim), (2000, 6000));
    // Anim: the motion ended -> x velocity cleared and Wait1_2 hand-off.
    let _ = take_log();
    r.f.cmd_vars[0] = 0;
    hook_stubs(0);
    r.f.self_vel.x = 3.0;
    t::ftMh_BackCrush_Anim(gp);
    assert_eq!(r.f.self_vel.x, 0.0);
    assert_eq!(take_log(), "389");
    t::ftMh_BackPunch_Coll(gp);
}
