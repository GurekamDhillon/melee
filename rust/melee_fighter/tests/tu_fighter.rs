//! tu/fighter.rs (the pure and self-contained parts of fighter.c) vs the C
//! (gen/cref/tu_fighter_ref.c, data in tests/data/tu_fighter.txt; regenerate with
//! `python rust/melee_fighter/gen/cref/tu_run.py tu_fighter`):
//!   SEG  cpu_assist_seg_dist2 (9 fused sites)       POL  tech_ai_policy_step sequences
//!   DEC  cpu_assist_decide / xorshift roll          RBH  RB_FighterHash mixing (widened + legacy, CPU block)
//! plus smoke tests of the self-contained stateful functions. The nh_* words are checked by
//! tests/nh_ref.rs (one shared implementation in melee_types::neutral_hash).
use melee_fighter::tu::fighter as m;
use melee_fighter::types::{Fighter, Vec3};

fn hx(s: &str) -> f32 { f32::from_bits(u32::from_str_radix(s, 16).unwrap()) }
fn v3(a: &str, b: &str, c: &str) -> Vec3 { Vec3 { x: hx(a), y: hx(b), z: hx(c) } }

#[test]
fn seg_dist2_bit_exact_vs_c() {
    let mut n = 0;
    for line in include_str!("data/tu_fighter.txt").lines().filter(|l| l.starts_with("SEG ")) {
        let w: Vec<&str> = line.split_whitespace().collect();
        let r = m::cpu_assist_seg_dist2(&v3(w[1], w[2], w[3]), &v3(w[4], w[5], w[6]), &v3(w[7], w[8], w[9]));
        assert_eq!(r.to_bits(), hx(w[10]).to_bits(), "{line}");
        n += 1;
    }
    assert_eq!(n, 3000);
}

#[test]
fn tech_ai_policy_bit_exact_vs_c() {
    let mut p: m::TechAiPolicy = unsafe { core::mem::zeroed() };
    let (mut n, mut events) = (0, 0);
    for line in include_str!("data/tu_fighter.txt").lines().filter(|l| l.starts_with("POL ")) {
        let w: Vec<&str> = line.split_whitespace().collect();
        let i = |k: usize| -> i32 { w[k].parse().unwrap() };
        if i(2) == 0 {
            m::tech_ai_policy_init(&mut p, i(3), w[4].parse().unwrap());
        }
        let o = m::TechAiObservation { type_: i(5), motion: i(6), floor: i(7), fresh_shoulder: i(8), tech_unlocked: i(9), eta: hx(w[10]) };
        let ev = m::tech_ai_policy_step(&mut p, &o);
        assert_eq!(ev, i(11), "{line}");
        assert_eq!((p.opportunities, p.lcancels, p.techs, p.misses, p.last_event, p.delay, p.cooldown),
                   (i(12), i(13), i(14), i(15), i(16), i(17), i(18)), "{line}");
        assert_eq!(p.seed, w[19].parse::<u32>().unwrap(), "{line}");
        n += 1;
        events += (ev != 0) as i32;
    }
    assert_eq!(n, 4800);
    assert!(events >= 10, "the sequences must actually fire events ({events})");
}

#[test]
fn cpu_assist_decide_bit_exact_vs_c() {
    let (mut n, mut accepted) = (0, 0);
    for line in include_str!("data/tu_fighter.txt").lines().filter(|l| l.starts_with("DEC ")) {
        let w: Vec<&str> = line.split_whitespace().collect();
        let i = |k: usize| -> i32 { w[k].parse().unwrap() };
        let mut s: m::CpuAssistSlot = unsafe { core::mem::zeroed() };
        let t = i(3) as usize;
        s.permille[t] = i(4);
        s.seed = w[9].parse().unwrap();
        let mut sub = -7;
        let r = m::cpu_assist_decide(&mut s, t, Some(&mut sub));
        assert_eq!((r, sub, s.seed, s.opportunities[t]), (i(5), i(6), w[7].parse::<u32>().unwrap(), i(8)), "{line}");
        n += 1;
        accepted += r;
    }
    assert_eq!(n, 2000);
    assert!(accepted > 200, "{accepted}");
}

#[test]
fn rb_fighter_hash_bit_exact_vs_c() {
    let lines: Vec<&str> = include_str!("data/tu_fighter.txt").lines().filter(|l| l.starts_with("RB")).collect();
    assert_eq!(lines.len(), 1500 * 4);
    let (mut cpu_cases, mut legacy_cases) = (0, 0);
    for q in lines.chunks(4) {
        let h: Vec<&str> = q[0].split_whitespace().collect();
        let f: Vec<&str> = q[1].split_whitespace().collect();
        let c: Vec<&str> = q[2].split_whitespace().collect();
        let d: Vec<&str> = q[3].split_whitespace().collect();
        assert_eq!((h[0], f[0], c[0], d[0]), ("RBH", "RBF", "RBC", "RBD"));
        let mut fp: Fighter = unsafe { core::mem::zeroed() };
        let int = |s: &str| -> i32 { s.parse().unwrap() };
        let uint = |s: &str| -> u32 { s.parse().unwrap() };
        fp.motion_id = int(f[1]) as _;
        fp.cur_pos = v3(f[2], f[3], f[4]);
        fp.self_vel.x = hx(f[5]);
        fp.self_vel.y = hx(f[6]);
        fp.dmg.x1830_percent = hx(f[7]);
        fp.facing_dir = hx(f[8]);
        fp.x8c_kb_vel.x = hx(f[9]);
        fp.x8c_kb_vel.y = hx(f[10]);
        fp.gr_vel = hx(f[11]);
        fp.dmg.x195c_hitlag_frames = hx(f[12]);
        fp.ground_or_air = int(f[13]) as _;
        fp.set_x2219_b5(int(f[14]) as _);
        fp.x1968_jumpsUsed = int(f[15]) as _;
        fp.shield_health = hx(f[16]);
        fp.x3E4_fighterCmdScript.frame_count = hx(f[17]);
        fp.x67F = int(f[18]) as _;
        fp.x680 = int(f[19]) as _;
        let cpu = &mut fp.cpu;
        cpu.buttons = uint(c[1]) as _;
        cpu.lstick.x = uint(c[2]) as u8 as i8 as _;
        cpu.lstick.y = uint(c[3]) as u8 as i8 as _;
        cpu.cstick.x = uint(c[4]) as u8 as i8 as _;
        cpu.cstick.y = uint(c[5]) as u8 as i8 as _;
        cpu.ltrigger = int(c[6]) as _;
        cpu.rtrigger = int(c[7]) as _;
        cpu.kind = int(c[8]) as _;
        cpu.level = int(c[9]) as _;
        cpu.x14 = int(c[10]);
        cpu.x18 = int(c[11]);
        cpu.x1C = int(c[12]);
        cpu.x20 = int(c[13]);
        cpu.x24 = int(c[14]);
        cpu.x28 = int(c[15]);
        cpu.x2C = int(c[16]);
        cpu.x30 = int(c[17]);
        cpu.x34 = int(c[18]);
        cpu.x38 = hx(c[19]);
        cpu.x3C = hx(c[20]);
        cpu.x40 = hx(c[21]);
        cpu.x7C = int(c[22]);
        cpu.x80 = int(c[23]);
        cpu.x84 = int(c[24]);
        cpu.xC8 = int(c[25]) as _;
        cpu.xEC = int(c[26]) as _;
        cpu.command_duration = uint(c[27]) as _;
        cpu.bf_f8 = [uint(c[28]) as u8, uint(c[29]) as u8, uint(c[30]) as u8, uint(d[1]) as u8];
        let csp = int(h[8]);
        cpu.csP = if csp < 0 { core::ptr::null_mut() } else { unsafe { cpu.buffer.as_mut_ptr().add(csp as usize) as *mut _ } };
        let legacy = int(h[7]) != 0;
        let is_cpu = int(h[3]) != 0;
        cpu_cases += is_cpu as i32;
        legacy_cases += legacy as i32;
        // the C mixes the geno word only when nonzero, already tagged (digest ^ "GDIF")
        let geno = uint(h[6]);
        let gw = if geno != 0 { geno ^ 0x4744_4946 } else { 0 };
        let r = unsafe { m::rb_fighter_hash_core(uint(h[1]), &fp, int(h[2]), uint(h[5]), gw, legacy, int(h[4]), is_cpu) };
        assert_eq!(r, uint(h[9]), "{} / {}", q[0], q[1]);
    }
    assert!(cpu_cases > 300 && legacy_cases > 300);
}

#[test]
fn self_contained_stateful_smoke() {
    unsafe {
        // the C's own native policy / ownership checks, ported 1:1: zero failed cases
        assert_eq!(m::ScriptGame_CpuTechnicalTest(), 0);
        assert_eq!(m::ScriptGame_CpuAssistTest(), 0);
        assert!(m::cpu_assist_config_ok(0, 0, 0, 0, 0, 0));
        assert!(!m::cpu_assist_config_ok(1001, 0, 0, 0, 0, 0));
        // an unowned slot's overlays return before touching netplay / gm / mp
        let mut fp: Fighter = core::mem::zeroed();
        fp.player_id = 2;
        m::technical_ai_overlay(&mut fp);
        m::cpu_assist_overlay(&mut fp);
        fp.player_id = 9;
        m::technical_ai_overlay(&mut fp);
        m::cpu_assist_overlay(&mut fp);
        assert_eq!(m::ScriptGame_CpuAssistState(2, 5, 0), 0);
        assert_eq!(m::ScriptGame_CpuTechnicalState(2, 5, 0), 0);
        m::RB_HashSetLegacy(1);
        m::RB_HashSetLegacy(-1);
        assert_eq!(m::RB_NeutralHashWord(-1), 0);
        assert_eq!(m::RB_NeutralHashWord(16), 0);
    }
}
