//! Night-3 round-6 DevText functions (hud::textlib) and grstadium_mul_vec against the C
//! (cref/gen_textlib_r6_ref.sh -> tests/data/textlib_r6_ref.txt), shipping-build contraction.
//! The C rows dump DevText byte +0x27 in the PC (little-endian clang) bitfield layout
//! (unk = bits 0-5, current_color = bits 6-7); melee_types keeps it in GC byte order
//! (unk = top 6 bits, current_color = low 2), so the test converts.
use melee_types::{DevText, GXColor};
use melee_world::gr::leaves::grstadium_mul_vec;
use melee_world::hud::textlib::*;
use melee_world::vec::Vec3;

const DATA: &str = include_str!("data/textlib_r6_ref.txt");

fn h(s: &str) -> u32 {
    u32::from_str_radix(s, 16).unwrap()
}
fn col(u: u32) -> GXColor {
    GXColor { r: (u >> 24) as u8, g: (u >> 16) as u8, b: (u >> 8) as u8, a: u as u8 }
}
fn cbits(c: GXColor) -> u32 {
    (c.r as u32) << 24 | (c.g as u32) << 16 | (c.b as u32) << 8 | c.a as u32
}
fn pc_to_gc(b: u8) -> u8 {
    (b & 0x3F) << 2 | b >> 6
}

fn parse(s: &str) -> (DevText, Vec<u8>) {
    let v: Vec<&str> = s.split_whitespace().collect();
    let mut t = DevText::zeroed();
    t.x = h(v[0]) as u16 as i16;
    t.y = h(v[1]) as u16 as i16;
    t.w = h(v[2]) as u8;
    t.h = h(v[3]) as u8;
    t.cursor_x = h(v[4]) as u8;
    t.cursor_y = h(v[5]) as u8;
    t.scale_x = f32::from_bits(h(v[6]));
    t.scale_y = f32::from_bits(h(v[7]));
    t.bg_color = col(h(v[8]));
    for i in 0..4 {
        t.text_colors[i] = col(h(v[9 + i]));
    }
    t.id = h(v[13]) as u8 as i8;
    t.line_width = h(v[14]) as u8;
    t.flags = h(v[15]) as u8;
    t.bf_27 = [pc_to_gc(h(v[16]) as u8)];
    let b = v.get(17).copied().unwrap_or("");
    let buf = (0..b.len() / 2).map(|i| u8::from_str_radix(&b[2 * i..2 * i + 2], 16).unwrap()).collect();
    (t, buf)
}

fn same(a: &DevText, b: &DevText) -> bool {
    a.x == b.x && a.y == b.y && a.w == b.w && a.h == b.h && a.cursor_x == b.cursor_x
        && a.cursor_y == b.cursor_y && a.scale_x.to_bits() == b.scale_x.to_bits()
        && a.scale_y.to_bits() == b.scale_y.to_bits() && cbits(a.bg_color) == cbits(b.bg_color)
        && (0..4).all(|i| cbits(a.text_colors[i]) == cbits(b.text_colors[i]))
        && a.id == b.id && a.line_width == b.line_width && a.flags == b.flags && a.bf_27 == b.bf_27
}

#[test]
fn textlib_r6_match_c() {
    let mut strs: Vec<Vec<u8>> = vec![vec![]; 64];
    let mut counts = std::collections::BTreeMap::new();
    for (ln, line) in DATA.lines().enumerate() {
        let tag = &line[..1];
        let parts: Vec<&str> = line[2..].split('|').collect();
        let ctx = || format!("line {ln}: {line}");
        match tag {
            "S" => {
                let id = h(parts[0].trim()) as usize;
                strs[id] = parts[1].split_whitespace().map(|b| h(b) as u8).collect();
            }
            "T" => {
                let a: Vec<u32> = parts[0].split_whitespace().map(h).collect();
                let (op, a0, a1) = (a[0], a[1], a[2]);
                let (mut t, mut buf) = parse(parts[1]);
                let after = parts[2].trim();
                let (after_state, ret) = after.rsplit_once(' ').unwrap();
                let (want, want_buf) = parse(after_state);
                buf.resize(256, 0xEE);
                let mut r = 0u32;
                match op {
                    0 => devtext_erase_first_line(&t, &mut buf),
                    1 => devtext_set_cursor_xy(&mut t, a0 as i32, a1 as i32),
                    2 => devtext_set_cursor_x(&mut t, a0 as i32),
                    3 => devtext_hide_cursor(&mut t),
                    4 => devtext_80302ac0(&mut t),
                    5 => devtext_show_background(&mut t),
                    6 => devtext_hide_background(&mut t),
                    7 => devtext_show_text(&mut t),
                    8 => devtext_hide_text(&mut t),
                    9 => devtext_set_scale(&mut t, f32::from_bits(a0), f32::from_bits(a1)),
                    10 => devtext_set_xy(&mut t, a0 as i32, a1 as i32),
                    11 => r = devtext_store_color_index(&mut t, a0 as u8) as u32,
                    12 => r = cbits(devtext_set_text_color(&mut t, col(a0))),
                    13 => r = cbits(devtext_set_bg_color(&mut t, col(a0))),
                    14 => devtext_erase(&t, &mut buf),
                    15 => devtext_advance_line(&mut t, &mut buf),
                    16 => {
                        let s = if a0 == 0 { None } else { Some(strs[a0 as usize].as_slice()) };
                        devtext_print(&mut t, &mut buf, s)
                    }
                    17 => devtext_print_int(&mut t, &mut buf, a0 as i32),
                    _ => unreachable!(),
                }
                assert!(same(&t, &want), "state {}", ctx());
                assert_eq!(&buf[..want_buf.len()], &want_buf[..], "buf {}", ctx());
                assert_eq!(r, h(ret), "ret {}", ctx());
            }
            "A" => {
                assert_eq!(cbits(textlib1_adjust(col(h(parts[0].trim())))), h(parts[1].trim()), "{}", ctx());
            }
            "M" => {
                let i: Vec<u32> = parts[0].split_whitespace().map(h).collect();
                let o: Vec<u32> = parts[1].split_whitespace().map(h).collect();
                let mut v = Vec3 { x: f32::from_bits(i[0]), y: f32::from_bits(i[1]), z: f32::from_bits(i[2]) };
                grstadium_mul_vec(&mut v, f32::from_bits(i[3]));
                assert_eq!([v.x.to_bits(), v.y.to_bits(), v.z.to_bits()], [o[0], o[1], o[2]], "{}", ctx());
            }
            _ => panic!("tag {tag}"),
        }
        *counts.entry(tag.to_string()).or_insert(0usize) += 1;
    }
    assert_eq!(counts["T"], 60000);
    assert_eq!(counts["A"], 4096);
    assert_eq!(counts["M"], 20000);
}
