//! Bit-for-bit check of the logic of baselib tobj.c against the C. Vectors:
//! `tests/data/oracle_tobj.txt`, written by `oracle/build_tobj.sh` (tobj.c through
//! `helper_tobj.c`, over the real mtx.c / mtx_pc.c / gekko_fp.c / lbtrigf.c, compiled with the
//! shipping build's contraction + gwtool lowering; generated inputs only). Row format: see
//! `oracle/helper_tobj.c`.
use melee_runtime::hsd_util::GxColor;
use melee_runtime::mtx::Vec3;
use melee_runtime::tobj::{self, HsdTObj, HsdTObjTev, HsdTexLodDesc, HsdTlut};

const DATA: &str = include_str!("data/oracle_tobj.txt");

fn b(f: f32) -> u32 {
    if f.is_nan() { 0x7FC0_0000 } else { f.to_bits() }
}
fn col(c: GxColor) -> u32 {
    u32::from_be_bytes([c.r, c.g, c.b, c.a])
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

fn state(t: &HsdTObj<u32>) -> u32 {
    let lod = t.lod.unwrap();
    let tev = t.tev.unwrap();
    hash(&[
        t.flags,
        b(t.rotate.x), b(t.rotate.y), b(t.rotate.z), b(t.rotate.w),
        b(t.scale.x), b(t.scale.y), b(t.scale.z),
        b(t.translate.x), b(t.translate.y), b(t.translate.z),
        b(t.blending), t.imagedesc.unwrap_or(0xFFFF_FFFF), t.tlut_no as u32,
        b(lod.lod_bias), col(tev.konst), col(tev.tev0), col(tev.tev1),
    ])
}

#[test]
fn tobj_matches_c() {
    let mut t: HsdTObj<u32> = HsdTObj::default();
    let mut fails = 0;
    let mut counts = [0usize; 4];
    let mut rows = 0;
    for (ln, line) in DATA.lines().enumerate() {
        let (i, o) = line.split_once('|').expect("row");
        let iw: Vec<u32> = i.split_whitespace().map(|w| u32::from_str_radix(w, 16).unwrap()).collect();
        let ow: Vec<u32> = o.split_whitespace().map(|w| u32::from_str_radix(w, 16).unwrap()).collect();
        let op = iw[0];
        counts[op as usize] += 1;
        rows += 1;
        let got: Vec<u32> = match op {
            0 => {
                let v = iw[2];
                vec![match iw[1] {
                    0 => tobj::tex_map_id_to_pt_tex_mtx(v),
                    1 => tobj::tg_tex_to_index(v),
                    2 => tobj::tex_coord_id_to_tex_gen_src(v),
                    3 => tobj::tex_coord_to_index(v),
                    4 => tobj::index_to_tex_coord(v),
                    5 => tobj::tex_mtx_to_index(v),
                    6 => tobj::index_to_tex_map(v),
                    7 => tobj::tex_map_to_index(v),
                    _ => tobj::index_to_tex_mtx(v),
                }]
            }
            1 => {
                let a = HsdTlut { n_entries: iw[1] as u16, ..Default::default() };
                let c = HsdTlut { n_entries: iw[2] as u16, ..Default::default() };
                vec![tobj::different_tluts(&a, &c) as u32]
            }
            2 => {
                let f = |k: usize| f32::from_bits(iw[1 + k]);
                let mut m: HsdTObj<u32> = HsdTObj::default();
                m.scale = Vec3 { x: f(0), y: f(1), z: f(2) };
                m.rotate.x = f(3);
                m.rotate.y = f(4);
                m.rotate.z = f(5);
                m.translate = Vec3 { x: f(6), y: f(7), z: f(8) };
                m.repeat_s = iw[10] as u8;
                m.repeat_t = iw[11] as u8;
                m.wrap_t = iw[12];
                m.make_texture_mtx();
                m.mtx.iter().flat_map(|r| r.iter().map(|&x| b(x))).collect()
            }
            _ => {
                let mut k = 1;
                if iw[1] == 0xFFFF_FFFF {
                    let tbl: Vec<Option<u32>> = (0..4).map(|j| (iw[2 + j] != 0).then_some(1 + j as u32)).collect();
                    t = HsdTObj {
                        imagetbl: Some(tbl),
                        imagedesc: (iw[6] != 0).then_some(0),
                        has_tluttbl: iw[7] != 0,
                        tlut_no: iw[8] as i32,
                        flags: iw[9],
                        lod: Some(HsdTexLodDesc::default()),
                        tev: Some(HsdTObjTev::default()),
                        ..Default::default()
                    };
                    k = 10;
                }
                t.update_func(iw[k], f32::from_bits(iw[k + 1]));
                vec![state(&t)]
            }
        };
        if got != ow {
            fails += 1;
            if fails <= 10 {
                eprintln!("row {}: {line}\n  got {:08x?}", ln + 1, got);
            }
        }
    }
    eprintln!("tobj: {rows} rows, per op {counts:?}, {fails} failed");
    assert_eq!(fails, 0);
    assert!(counts.iter().all(|&c| c > 4000));
}
