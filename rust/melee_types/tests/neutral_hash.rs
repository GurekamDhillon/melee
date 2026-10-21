//! The neutral-hash words against the C oracle (tests/data/nh_expected.txt, written by
//! gen/nh_oracle.py from fighter.c's own nh_* functions compiled natively), and the frame combine
//! against the C build's golden CSVs (docs/rust-port/golden/).
use melee_types::neutral_hash::*;
use melee_types::*;

fn zeroed<T>() -> Box<T> {
    unsafe {
        let l = std::alloc::Layout::new::<T>();
        Box::from_raw(std::alloc::alloc_zeroed(l) as *mut T)
    }
}

/// Mirrors `fill()` in gen/nh_oracle.py exactly.
fn fill(f: &mut Fighter, it: &mut Item, k: i32) {
    let kf = k as f32;
    let mut pat: u32 = 0xA5C3_0F96 ^ (k as u32).wrapping_mul(0x0101_0101);
    f.player_id = (k % 4) as u8;
    f.motion_id = (0x10 + k * 7) as _;
    f.cur_pos.x = kf * 1.5;
    f.cur_pos.y = -kf * 0.25;
    f.cur_pos.z = kf * 0.1;
    f.self_vel.x = kf * 0.5;
    f.self_vel.y = -1.0 - kf;
    f.dmg.x1830_percent = kf * 12.5;
    f.facing_dir = if k & 1 != 0 { -1.0 } else { 1.0 };
    f.ground_or_air = (k & 1) as _;
    f.x8c_kb_vel.x = kf * 3.0;
    f.x8c_kb_vel.y = kf * -2.0;
    f.gr_vel = kf * 0.75;
    f.dmg.x195c_hitlag_frames = kf;
    f.set_x2219_b5((k & 1) as u8);
    f.shield_health = 60.0 - kf;
    f.x3E4_fighterCmdScript.frame_count = kf * 2.0;
    f.x1968_jumpsUsed = k as u8;
    f.x67F = (k + 1) as u8;
    f.x680 = (k + 2) as u8;
    let c = &mut f.cpu;
    c.buttons = 0x1000 + k as u32;
    c.lstick.x = (-k) as i8;
    c.lstick.y = k as i8;
    c.cstick.x = (k * 9) as i8;
    c.cstick.y = (-k * 13) as i8;
    c.ltrigger = (k * 10) as u8;
    c.rtrigger = (255 - k) as u8;
    c.kind = k as _;
    c.level = 9;
    let xs = [&mut c.x14, &mut c.x18, &mut c.x1C, &mut c.x20, &mut c.x24, &mut c.x28, &mut c.x2C, &mut c.x30, &mut c.x34];
    for (i, x) in xs.into_iter().enumerate() {
        *x = k * 100 + i as i32;
    }
    c.x38 = kf * 0.5 + 0.0;
    c.x3C = kf * 0.5 + 1.0;
    c.x40 = kf * 0.5 + 2.0;
    c.x7C = -k;
    c.x80 = -k - 1;
    c.x84 = -k - 2;
    c.xC8 = k as u8;
    c.xEC = (k * 2) as u8;
    c.command_duration = 30 + k as u32;
    c.csP = unsafe { c.buffer.as_mut_ptr().add((k * 3) as usize) };
    macro_rules! b { ($($set:ident $w:literal)*) => {$(
        c.$set((pat & ((1u32 << $w) - 1)) as u8); pat = pat.rotate_right($w);
    )*} }
    b!(set_xF8_b0 1 set_xF8_b12 2 set_xF8_b34 2 set_xF8_b5 1 set_xF8_b6 1 set_xF8_b7 1
       set_xF9_b0 1 set_xF9_b1 1 set_xF9_b2 1 set_xF9_b3 1 set_xF9_b4 1 set_xF9_b5 1 set_xF9_b6 1 set_xF9_b7 1
       set_xFA_b0 1 set_xFA_b1 1 set_xFA_b2 1 set_xFA_b34 2 set_xFA_b5 1 set_xFA_b6 1 set_xFA_b7 1
       set_xFB_b0 1 set_xFB_b1 1 set_xFB_b2 1 set_xFB_b3 1 set_xFB_b4 1 set_xFB_b5 1 set_xFB_b6 1 set_xFB_b7 1);
    it.kind = (k * 5) as _;
    it.msid = (k + 3) as _;
    it.pos.x = kf * 4.0;
    it.pos.y = kf * -0.5;
    it.pos.z = 0.0;
    it.x40_vel.x = kf * 0.125;
    it.x40_vel.y = -kf;
    it.x40_vel.z = kf * 1e-3;
}

#[test]
fn words_match_c_oracle() {
    let want = include_str!("data/nh_expected.txt");
    let mut f: Box<Fighter> = zeroed();
    let mut it: Box<Item> = zeroed();
    let mut n = 0;
    for line in want.lines().filter(|l| !l.starts_with('#')) {
        let v: std::vec::Vec<&str> = line.split_whitespace().collect();
        let k: i32 = v[0].parse().unwrap();
        let cpu = v[1] == "1";
        let hx = |s: &str| u32::from_str_radix(s, 16).unwrap();
        fill(&mut f, &mut it, k);
        assert_eq!(nh_cpu_flags(&f.cpu), hx(v[2]), "nh_CpuFlags k={k}");
        assert_eq!(nh_fighter(&f, k, 3 + f.player_id as i32, cpu), hx(v[3]), "nh_Fighter k={k}");
        assert_eq!(nh_item(&it), hx(v[4]), "nh_Item k={k}");
        n += 1;
    }
    assert_eq!(n, 8);
}

/// Recompute every `hash` cell of the C goldens from that row's seed/items/itemsum/s0..s11.
#[test]
fn game_combine_matches_goldens() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/rust-port/golden");
    let mut rows = 0;
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.extension().map_or(true, |x| x != "csv") {
            continue;
        }
        for line in std::fs::read_to_string(&p).unwrap().lines().skip(1) {
            let c: std::vec::Vec<&str> = line.trim().split(',').collect();
            let hx = |s: &str| u32::from_str_radix(s, 16).unwrap();
            let mut w = [None; 12];
            for i in 0..12 {
                let v = hx(c[5 + i]);
                if v != 0 {
                    w[i] = Some(v);
                }
            }
            let got = nh_game(hx(c[2]), &w, c[3].parse().unwrap(), hx(c[4]));
            assert_eq!(got, hx(c[1]), "{} frame {}", p.display(), c[0]);
            rows += 1;
        }
    }
    assert!(rows > 10_000, "only {rows} golden rows");
}
