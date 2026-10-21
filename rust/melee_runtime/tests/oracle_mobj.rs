//! Bit-for-bit check of the logic of baselib mobj.c against the C. Vectors:
//! `tests/data/oracle_mobj.txt`, written by `oracle/build_mobj.sh` (mobj.c through
//! `helper_mobj.c`, shipping-build contraction + gwtool lowering; generated inputs only).
//! Row format: see `oracle/helper_mobj.c`.
use melee_runtime::hsd_util::GxColor;
use melee_runtime::mobj::{self, HsdMObj, HsdMaterial, HsdPeDesc, MObjGlobals};

const DATA: &str = include_str!("data/oracle_mobj.txt");

fn b(f: f32) -> u32 {
    if f.is_nan() { 0x7FC0_0000 } else { f.to_bits() }
}
fn col(c: GxColor) -> u32 {
    u32::from_be_bytes([c.r, c.g, c.b, c.a])
}
fn uncol(w: u32) -> GxColor {
    let [r, g, b, a] = w.to_be_bytes();
    GxColor { r, g, b, a }
}
fn hash(words: &[u32]) -> u32 {
    let mut h: u32 = 2166136261;
    for w in words {
        for byte in w.to_le_bytes() {
            h ^= byte as u32;
            h = h.wrapping_mul(16777619);
        }
    }
    h
}

/// The PE bytes live on even with `m.pe == None` (the C keeps the static record).
fn state(m: &HsdMObj<u32>, pe_store: &HsdPeDesc, cur: &MObjGlobals<()>) -> u32 {
    let mat = m.mat.unwrap();
    let p = m.pe.unwrap_or(*pe_store);
    hash(&[
        m.rendermode, m.tobj.unwrap_or(0xFFFF_FFFF), cur.current_mobj.is_some() as u32,
        col(mat.ambient), col(mat.diffuse), col(mat.specular), b(mat.alpha), b(mat.shininess),
        m.pe.is_some() as u32,
        p.flags as u32, p.ref0 as u32, p.ref1 as u32, p.dst_alpha as u32, p.ty as u32,
        p.src_factor as u32, p.dst_factor as u32, p.logic_op as u32, p.z_comp as u32,
        p.alpha_comp0 as u32, p.alpha_op as u32, p.alpha_comp1 as u32,
    ])
}

#[test]
fn mobj_matches_c() {
    let mut m: HsdMObj<u32> = HsdMObj::default();
    let mut pe_store = HsdPeDesc::default();
    let mut cur: MObjGlobals<()> = MObjGlobals::default();
    let (mut fails, mut rows) = (0, 0);
    let mut counts = [0usize; 6];
    for (ln, line) in DATA.lines().enumerate() {
        let (i, o) = line.split_once('|').expect("row");
        let iw: Vec<u32> = i.split_whitespace().map(|w| u32::from_str_radix(w, 16).unwrap()).collect();
        let ow: Vec<u32> = o.split_whitespace().map(|w| u32::from_str_radix(w, 16).unwrap()).collect();
        let mut k = 0;
        if iw[0] == 0xFFFF_FFFF {
            let pb: Vec<u8> = iw[9..21].iter().map(|&x| x as u8).collect();
            let pe = HsdPeDesc {
                flags: pb[0], ref0: pb[1], ref1: pb[2], dst_alpha: pb[3], ty: pb[4], src_factor: pb[5],
                dst_factor: pb[6], logic_op: pb[7], z_comp: pb[8], alpha_comp0: pb[9], alpha_op: pb[10],
                alpha_comp1: pb[11],
            };
            m = HsdMObj {
                rendermode: iw[1],
                pe: (iw[2] != 0).then_some(pe),
                tobj: (iw[3] != 0xFFFF_FFFF).then_some(iw[3]),
                mat: Some(HsdMaterial {
                    ambient: uncol(iw[4]), diffuse: uncol(iw[5]), specular: uncol(iw[6]),
                    alpha: f32::from_bits(iw[7]), shininess: f32::from_bits(iw[8]),
                }),
            };
            pe_store = pe;
            cur = MObjGlobals::default();
            k = 21;
        }
        let op = iw[k];
        counts[op as usize] += 1;
        rows += 1;
        let a = &iw[k + 1..];
        let mut got = Vec::new();
        match op {
            0 => mobj::set_flags((a[0] == 0).then_some(&mut m), a[1]),
            1 => mobj::clear_flags((a[0] == 0).then_some(&mut m), a[1]),
            2 => {
                if a[0] == 0 {
                    m.update_func(a[1], f32::from_bits(a[2]));
                }
            }
            3 => m.set_diffuse_color(a[0] as u8, a[1] as u8, a[2] as u8),
            4 => m.set_alpha(f32::from_bits(a[0])),
            _ => {
                got.push(mobj::get_tobj((a[0] == 0).then_some(&m)).unwrap_or(0xFFFF_FFFF));
                cur.set_current((a[0] == 0).then_some(()));
            }
        }
        got.push(state(&m, &pe_store, &cur));
        if got != ow {
            fails += 1;
            if fails <= 10 {
                eprintln!("row {}: {line}\n  got {:08x?}", ln + 1, got);
            }
        }
    }
    eprintln!("mobj: {rows} rows, per op {counts:?}, {fails} failed");
    assert_eq!(fails, 0);
    assert!(counts.iter().all(|&c| c > 2500));
}
