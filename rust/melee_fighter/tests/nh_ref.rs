//! NHV1 fighter word vs the C build's own nh_Fighter (gen/cref/nh_ref.c; expected values in
//! tests/data/nh.txt, regenerate with `python rust/melee_fighter/gen/cref/run.py nh`).
//! The fill order below must stay identical to nh_ref.c's.

use melee_fighter::neutral_hash::{cpu_flags, fighter_word};
use melee_fighter::types::Fighter;

struct Rng(u32);
impl Rng {
    fn rnd(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    fn rf(&mut self) -> f32 {
        let mut u = self.rnd();
        if (u >> 23) & 0xFF == 0xFF {
            u &= !(1 << 23);
        }
        f32::from_bits(u)
    }
}

#[test]
fn fighter_word_matches_c() {
    let expected = include_str!("data/nh.txt");
    let mut n = 0;
    for line in expected.lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        let k: u32 = f[0].parse().unwrap();
        let want_word = u32::from_str_radix(f[1], 16).unwrap();
        let want_flags = u32::from_str_radix(f[2], 16).unwrap();

        let mut fp: Box<Fighter> = Box::new(Fighter::zeroed());
        let mut r = Rng(0x9E37_79B9 ^ k.wrapping_mul(0x85EB_CA6B));
        if r.0 == 0 {
            r.0 = 1;
        }
        fp.motion_id = (r.rnd() % 400) as _;
        fp.cur_pos.x = r.rf();
        fp.cur_pos.y = r.rf();
        fp.cur_pos.z = r.rf();
        fp.self_vel.x = r.rf();
        fp.self_vel.y = r.rf();
        fp.dmg.x1830_percent = r.rf();
        fp.facing_dir = r.rf();
        fp.ground_or_air = (r.rnd() & 1) as _;
        fp.x8c_kb_vel.x = r.rf();
        fp.x8c_kb_vel.y = r.rf();
        fp.gr_vel = r.rf();
        fp.dmg.x195c_hitlag_frames = r.rf();
        fp.set_x2219_b5((r.rnd() & 1) as u8);
        fp.shield_health = r.rf();
        fp.x3E4_fighterCmdScript.frame_count = r.rf();
        fp.x1968_jumpsUsed = r.rnd() as u8;
        fp.x67F = r.rnd() as u8;
        fp.x680 = r.rnd() as u8;
        let stocks = (r.rnd() % 7) as i32 - 1;
        let cpu = r.rnd() & 1 != 0;
        let c = &mut fp.cpu;
        c.buttons = r.rnd();
        c.lstick.x = r.rnd() as i8;
        c.lstick.y = r.rnd() as i8;
        c.cstick.x = r.rnd() as i8;
        c.cstick.y = r.rnd() as i8;
        c.ltrigger = r.rnd() as u8;
        c.rtrigger = r.rnd() as u8;
        c.kind = (r.rnd() % 32) as _;
        c.level = (r.rnd() % 10) as i32;
        c.x14 = r.rnd() as i32;
        c.x18 = r.rnd() as i32;
        c.x1C = r.rnd() as i32;
        c.x20 = r.rnd() as i32;
        c.x24 = r.rnd() as i32;
        c.x28 = r.rnd() as i32;
        c.x2C = r.rnd() as i32;
        c.x30 = r.rnd() as i32;
        c.x34 = r.rnd() as i32;
        c.x38 = r.rf();
        c.x3C = r.rf();
        c.x40 = r.rf();
        c.x7C = r.rnd() as i32;
        c.x80 = r.rnd() as i32;
        c.x84 = r.rnd() as i32;
        c.xC8 = r.rnd() as u8;
        c.xEC = r.rnd() as u8;
        c.command_duration = r.rnd();
        let off = r.rnd() % 300;
        c.csP = if off <= 256 { unsafe { c.buffer.as_mut_ptr().add(off as usize) } } else { core::ptr::null_mut() };
        macro_rules! b {
            ($($set:ident $w:literal),*) => {$( c.$set((r.rnd() & ((1u32 << $w) - 1)) as u8); )*};
        }
        b!(set_xF8_b0 1, set_xF8_b12 2, set_xF8_b34 2, set_xF8_b5 1, set_xF8_b6 1, set_xF8_b7 1,
           set_xF9_b0 1, set_xF9_b1 1, set_xF9_b2 1, set_xF9_b3 1, set_xF9_b4 1, set_xF9_b5 1, set_xF9_b6 1, set_xF9_b7 1,
           set_xFA_b0 1, set_xFA_b1 1, set_xFA_b2 1, set_xFA_b34 2, set_xFA_b5 1, set_xFA_b6 1, set_xFA_b7 1,
           set_xFB_b0 1, set_xFB_b1 1, set_xFB_b2 1, set_xFB_b3 1, set_xFB_b4 1, set_xFB_b5 1, set_xFB_b6 1, set_xFB_b7 1);

        assert_eq!(cpu_flags(&fp.cpu), want_flags, "case {k}: cpu flag word");
        assert_eq!(fighter_word(&fp, k % 12, stocks, cpu), want_word, "case {k}: fighter word");
        n += 1;
    }
    assert_eq!(n, 64);
}
