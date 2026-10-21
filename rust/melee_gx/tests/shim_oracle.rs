//! The shim's own arithmetic vs the C it ports, cut verbatim from pc/platform/shim_gx.c and
//! Aurora and built as shipping shims are (oracle/shim_build.sh -> tests/data/shim_oracle.txt):
//! GXProject (reaches the simulation), GXInitFogAdjTable, GXGetTexBufferSize, GXInitLightSpot,
//! GXInitLightDistAttn. Every result compared bit for bit.
use melee_gx::api::GxLightObj;
use melee_gx::shim::{get_tex_buffer_size, gx_project, init_fog_adj_table, init_light_dist_attn, init_light_spot};

struct Rng(u32);
impl Rng {
    fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    fn fr(&mut self, lim: i32) -> f32 {
        let m = (2 * lim * 100 + 1) as u32;
        ((self.next() % m) as i32 - lim * 100) as f32 / 100.0
    }
}

#[test]
fn shim_math_matches_c() {
    let data = include_str!("data/shim_oracle.txt");
    let mut lines = data.lines();
    let mut next_line = |tag: &str, i: usize| -> Vec<String> {
        let l = lines.next().expect("oracle file too short");
        let v: Vec<String> = l.split(' ').map(str::to_string).collect();
        assert_eq!((v[0].as_str(), v[1].parse::<usize>().unwrap()), (tag, i), "oracle line order");
        v[2..].to_vec()
    };
    let hex = |s: &str| u32::from_str_radix(s, 16).unwrap();
    let mut r = Rng(0x2545F491);
    let mut bad = 0;

    for i in 0..4000 {
        let mut m = [[0f32; 4]; 3];
        for row in m.iter_mut() {
            for (c, v) in row.iter_mut().enumerate() {
                *v = if c == 3 { r.fr(300) } else { r.fr(2) };
            }
        }
        let mut pm = [0f32; 7];
        pm[0] = if i & 1 == 1 { 1.0 } else { 0.0 };
        for k in 1..7 {
            pm[k] = r.fr(4);
        }
        let mut vp = [r.fr(50), r.fr(50), 640.0 + r.fr(100), 480.0 + r.fr(100), 0.0, 1.0];
        vp[4] = (r.next() % 50) as f32 / 100.0;
        let (x, y, z) = (r.fr(200), r.fr(200), r.fr(400));
        let (sx, sy, sz) = gx_project(x, y, z, &m, &pm, &vp);
        let want = next_line("project", i);
        let got = [sx.to_bits(), sy.to_bits(), sz.to_bits()];
        if got.iter().zip(&want).any(|(g, w)| *g != hex(w)) {
            bad += 1;
            if bad < 10 {
                eprintln!("project {i}: got {got:08x?} want {want:?}");
            }
        }
    }
    for i in 0..1000 {
        let mut m = [[0f32; 4]; 4];
        for row in m.iter_mut() {
            for v in row.iter_mut() {
                *v = r.fr(3);
            }
        }
        m[3][3] = if i & 1 == 1 { 1.0 } else { 0.0 };
        if m[0][0] == 0.0 {
            m[0][0] = 1.0;
        }
        let width = (160 + r.next() % 1000) as u16;
        let t = init_fog_adj_table(width, &m);
        let want = next_line("fog", i);
        if t.iter().zip(&want).any(|(g, w)| *g as u32 != hex(w)) {
            bad += 1;
            if bad < 10 {
                eprintln!("fog {i}: got {t:03x?} want {want:?}");
            }
        }
    }
    const FMTS: [u32; 28] = [
        0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x8, 0x9, 0xa, 0xe, 0x11, 0x13, 0x16, 0x20, 0x22, 0x23, 0x27, 0x28, 0x29,
        0x2a, 0x2b, 0x2c, 0x30, 0x39, 0x3a, 0x3c, 0x41,
    ];
    for i in 0..2000 {
        let f = FMTS[(r.next() % FMTS.len() as u32) as usize];
        let w = (1 + r.next() % 1024) as u16;
        let h = (1 + r.next() % 1024) as u16;
        let mips = r.next() & 1 == 1;
        let lod = (r.next() % 12) as u8;
        let got = get_tex_buffer_size(w, h, f, mips, lod);
        let want = next_line("texsize", i);
        if got != want[0].parse::<u32>().unwrap() {
            bad += 1;
            if bad < 10 {
                eprintln!("texsize {i}: got {got} want {want:?}");
            }
        }
    }
    for i in 0..2000 {
        let mut o = GxLightObj { a0: 1.0, a1: 0.0, a2: 0.0, k0: 1.0, k1: 0.0, k2: 0.0, ..Default::default() };
        let cutoff = r.fr(100);
        init_light_spot(&mut o, cutoff, r.next() % 7);
        let dist = r.fr(500);
        let br = r.fr(1);
        init_light_dist_attn(&mut o, dist, br, r.next() % 4);
        let got = [o.a0, o.a1, o.a2, o.k0, o.k1, o.k2].map(f32::to_bits);
        let want = next_line("light", i);
        if got.iter().zip(&want).any(|(g, w)| *g != hex(w)) {
            bad += 1;
            if bad < 10 {
                eprintln!("light {i}: got {got:08x?} want {want:?}");
            }
        }
    }
    assert_eq!(bad, 0, "{bad} of 9000 shim oracle cases differ");
}
