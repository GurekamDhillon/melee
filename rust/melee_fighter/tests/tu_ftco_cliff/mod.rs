//! Rust half of gen/cref/tu/ftco_cliff_fill.h: the same fill, state hash and recording stubs
//! for the five ftCo_Cliff*.c TU tests. Each test points its module's hooks at these.
#![allow(dead_code, static_mut_refs, non_snake_case)]
use super::tu_ftco_harness::*;
use melee_fighter::types::*;

pub type G = *mut HSD_GObj;
pub static mut ANG: [f32; 2] = [0.0; 2];

pub fn cliff_fill() {
    unsafe {
        ANG[0] = rg() * 2.0;
        ANG[1] = rg() * 2.0;
    }
    let fr = f();
    fr.input.pressed_buttons = rnd() & 0xFFFF;
    fr.input.lstick[0].x = rg();
    fr.input.lstick[0].y = rg();
    fr.input.cstick[0].x = rg();
    fr.dmg.x1830_percent = rw();
    let c = cd();
    c.x488 = (rnd() % 200) as i32 - 50;
    c.x48C = rw();
    c.x490 = rw();
    c.x494 = rg();
    c.x20_radians = rg();
    c.ledge_cooldown = (rnd() % 100) as i32;
    c.x49C = (rnd() % 100) as i32;
    fr.facing_dir = rg();
    fr.mv.co.cliff.ledge_id = (rnd() % 4) as i32;
    fr.mv.co.cliff.x4 = rw();
    fr.mv.co.cliff.x8 = (rnd() & 1) as i32;
    fr.ground_or_air = (rnd() & 1) as _;
    fr.x68C_transNPos.x = rg();
    fr.x68C_transNPos.y = rg();
    fr.x68C_transNPos.z = rg();
    fr.self_vel.x = rw();
    fr.self_vel.y = rw();
    fr.co_attrs.ledge_jump_horizontal_velocity = rg();
    fr.co_attrs.ledge_jump_vertical_velocity = rg();
    fr.motion_id = if rnd() & 1 != 0 { ftCo_MS_CliffJumpQuick1 as i32 } else { ftCo_MS_CliffJumpSlow1 as i32 };
    fr.cur_pos.x = rw();
    fr.cur_pos.y = rw();
    fr.coll_data.floor.index = (rnd() % 64) as _;
    fr.x2064_ledgeCooldown = (rnd() % 9) as i32;
}

pub fn state_hash() -> u32 {
    let fr = f();
    let mut h = Hs::new();
    h.m(b(fr.cur_pos.x));
    h.m(b(fr.cur_pos.y));
    h.m(fr.coll_data.floor.index as u32);
    h.m(fr.x2064_ledgeCooldown as u32);
    h.m(fr.x221D_b5() as u32);
    h.m(fr.x221D_b7() as u32);
    h.m(fr.x2227_b1() as u32);
    unsafe {
        h.m(fr.mv.co.cliff.ledge_id as u32);
        h.m(b(fr.mv.co.cliff.x4));
        h.m(fr.mv.co.cliff.x8 as u32);
    }
    h.m(b(fr.self_vel.x));
    h.m(b(fr.self_vel.y));
    h.0
}

fn is_f(p: *const Fighter) -> u32 { (p == f() as *const Fighter) as u32 }

pub fn s_lstick(fr: &Fighter) -> f32 { t(1); t(is_f(fr)); unsafe { ANG[0] } }
pub fn s_cstick(fr: &Fighter) -> f32 { t(2); t(is_f(fr)); unsafe { ANG[1] } }
pub fn s_change_motion(g: G, m: i32, fl: u32, a: f32, bb: f32, c: f32, o: G) {
    t(3); t(is_go(g)); t(m as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32);
}
pub fn s_6eba4(g: G) { t(4); t(is_go(g)); }
pub fn s_7e2f4(fr: &mut Fighter, v: i16) { t(5); t(is_f(fr)); t(v as i32 as u32); }
pub fn s_catch_phys(g: G) { t(6); t(is_go(g)); }
pub fn s_catch_coll(g: G) { t(7); t(is_go(g)); }
pub fn s_7d7fc(fr: &mut Fighter) { t(8); t(is_f(fr)); }
pub fn s_7d92c(g: G) { t(9); t(is_go(g)); }
pub fn s_df6f8(fr: &Fighter) -> bool { t(10); t(is_f(fr)); sr(2) }
pub fn s_df72c(fr: &Fighter) -> bool { t(11); t(is_f(fr)); sr(3) }
pub fn s_df79c(fr: &Fighter) -> bool { t(12); t(is_f(fr)); sr(4) }
pub fn s_fall_enter(g: G) { t(13); t(is_go(g)); }
pub fn s_54ed8(l: i32) -> bool { t(14); t(l as u32); sr(5) }
pub fn s_53ecc(l: i32, v: &mut Vec3) { t(15); t(l as u32); v.x = rg(); v.y = rg(); v.z = rg(); }
pub fn s_53da4(l: i32, v: &mut Vec3) { t(16); t(l as u32); v.x = rg(); v.y = rg(); v.z = rg(); }
pub fn s_84fa8(g: G) { t(17); t(is_go(g)); }
pub fn s_821dc(g: G) -> bool { t(18); t(is_go(g)); sr(6) }
pub fn s_9ef68(g: G) -> bool { t(19); t(is_go(g)); sr(7) }
pub fn s_84104(g: G) { t(20); t(is_go(g)); }
pub fn s_jump_input(g: G) -> i32 { t(21); t(is_go(g)); unsafe { (SRV[8] % 3) as i32 } }
pub fn s_frames_remaining(g: G) -> bool { t(22); t(is_go(g)); sr(9) }
pub fn s_84db0(g: G) { t(23); t(is_go(g)); }
pub fn s_7b760(g: G, v: i32) { t(27); t(is_go(g)); t(v as u32); }
pub fn s_90780(g: G) { t(28); t(is_go(g)); }
/// Sibling-TU stubs (40+).
pub fn s_climb_anim(g: G) { t(40); t(is_go(g)); }
pub fn s_climb_phys(g: G) { t(41); t(is_go(g)); }
pub fn s_climb_coll(g: G) { t(42); t(is_go(g)); }
pub fn s_9b040(g: G) { t(43); t(is_go(g)); }
pub fn s_9ae14(g: G) { t(44); t(is_go(g)); }
pub fn s_9ae38(g: G) -> bool { t(45); t(is_go(g)); sr(11) }
pub fn s_9afd4(g: G) -> bool { t(46); t(is_go(g)); sr(12) }
pub fn s_9b170(g: G) -> bool { t(47); t(is_go(g)); sr(13) }
pub fn s_9aa0c(g: G) -> bool { t(48); t(is_go(g)); sr(14) }

/// Runs `nops` x 2000 cases against tests/data/tu_<stem>.txt.
pub fn run_cliff(expected: &'static str, nops: u32, run: fn(u32) -> u32) -> u32 {
    let mut c = Check::new(expected);
    for op in 0..nops {
        for k in 0..2000u32 {
            case(op, k);
            cliff_fill();
            let ret = run(op);
            c.line(op, k, ret, state_hash());
        }
    }
    c.done()
}
