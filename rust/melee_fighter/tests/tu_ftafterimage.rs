//! tu/ftafterimage.rs vs the C (gen/cref/tu_ftafterimage_ref.c, data in
//! tests/data/tu_ftafterimage.txt; regenerate with
//! `python rust/melee_fighter/gen/cref/tu_run.py tu_ftafterimage`): the vertices ftCo_800C2600
//! hands to GX, bit for bit, over 3000 random sword rings.
use melee_fighter::tu::ftafterimage as m;
use melee_fighter::types::{itSword_UnkBytes, Fighter};

fn h32(s: &str) -> u32 { u32::from_str_radix(s, 16).unwrap() }
fn f(s: &str) -> f32 { f32::from_bits(h32(s)) }

#[test]
fn strip_bit_exact_vs_c() {
    let data = include_str!("data/tu_ftafterimage.txt");
    let (mut n, mut drawn) = (0, 0);
    for line in data.lines() {
        let (inp, out) = line.split_once(" |").unwrap();
        let w: Vec<&str> = inp.split_whitespace().collect();
        assert_eq!(w[0], "C");
        let mut fi: Box<Fighter> = Box::new(unsafe { core::mem::zeroed() });
        fi.x2100 = w[1].parse::<i8>().unwrap();
        fi.set_x2101_bits_0to6(w[2].parse::<u8>().unwrap());
        fi.x20F8 = f(w[3]);
        fi.x20FC = f(w[4]);
        let mut p = itSword_UnkBytes::zeroed();
        p.x0 = f(w[5]);
        p.x4 = f(w[6]);
        let b = h32(&w[7][..8]).to_be_bytes();
        let b2 = h32(&w[7][8..]).to_be_bytes();
        (p.x8, p.x9, p.xA, p.xB, p.xC, p.xE, p.xF, p.x10) = (b[0], b[1], b[2], b[3], b2[0], b2[1], b2[2], b2[3]);
        for j in 0..3 {
            let k = 8 + j * 6;
            let e = &mut fi.x20B0[j];
            (e.x0.x, e.x0.y, e.x0.z) = (f(w[k]), f(w[k + 1]), f(w[k + 2]));
            (e.xC.x, e.xC.y, e.xC.z) = (f(w[k + 3]), f(w[k + 4]), f(w[k + 5]));
        }
        let o: Vec<&str> = out.split_whitespace().collect();
        n += 1;
        if fi.x2100 <= 1 {
            assert!(o.is_empty(), "{line}");
            continue;
        }
        drawn += 1;
        let v = m::afterimage_strip(&fi, &p);
        assert_eq!(o[0], "N");
        assert_eq!(o[1].parse::<usize>().unwrap(), v.len(), "{line}");
        let mut k = 2;
        for (i, x) in v.iter().enumerate() {
            let got = (x.x.to_bits(), x.y.to_bits(), x.z.to_bits(), u32::from_be_bytes([x.r, x.g, x.b, x.a]));
            let exp = (h32(o[k]), h32(o[k + 1]), h32(o[k + 2]), h32(o[k + 3]));
            assert_eq!(got, exp, "vertex {i} of {line}");
            k += 4;
        }
        assert_eq!(k, o.len());
    }
    assert_eq!(n, 3000);
    assert!(drawn > 1000);
}

#[test]
fn c2fd8_smoke() {
    use melee_fighter::types::HSD_GObj;
    let fi: *mut Fighter = Box::into_raw(Box::new(unsafe { core::mem::zeroed::<Fighter>() }));
    let mut g: HSD_GObj = unsafe { core::mem::zeroed() };
    g.user_data = fi as *mut _;
    unsafe {
        (*fi).x2100 = -1;
        m::ftCo_800C2FD8(&mut g); // inactive ring: untouched
        assert_eq!((*fi).x2100, -1);
        (*fi).x2100 = 0;
        (*fi).set_x2101_bits_8(1); // item trail with no item: disables the ring
        m::ftCo_800C2FD8(&mut g);
        assert_eq!((*fi).x2100, -1);
        m::ftCo_800C2600(&mut g, 1); // not the trail pass: draws nothing
        drop(Box::from_raw(fi));
    }
}
