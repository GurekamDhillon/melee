//! Petey's real look, loaded at runtime from the owner's own extracted Brawl files (notes/petey-extract.md).
//!
//! Presentation only: nothing here feeds the sim (the boss state lives in `petey.rs`); the draw reads `petey::pose()`
//! and the model is drawn on top of it. When the local data is absent (the usual case) `get()` is `None` and the
//! placeholder capsules are drawn instead.
//!
//! Where: `MELEE_BRAWL_LOCAL` (a folder holding `disc/files/...`), else the first `_build/brawl` found walking up
//! from the working folder or the exe. Nothing from it is ever written anywhere.
//!
//! What is read: the adventure stage archive holding the boss (ARC, LZ77 inside), the BRRES with Petey's body MDL0
//! and the two arm / cage MDL0s, their TEX0 textures (CMPR / RGB / I formats). Drawn in bind pose with each
//! vertex coloured from its texture texel and vertex colour, plus a fixed light. CHR0 animation is not played yet.
#![allow(clippy::missing_safety_doc)]

use std::path::{Path, PathBuf};

/// One drawable mesh: triangles as (model-space position, rgba).
pub struct Mesh {
    pub tris: Vec<([f32; 3], [u8; 4])>,
    pub min: [f32; 3],
    pub max: [f32; 3],
}

pub struct Model {
    pub body: Mesh,
    pub arm_l: Option<Mesh>,
    pub arm_r: Option<Mesh>,
}

/// The model was loaded (without loading it: the HUD asks every frame).
pub fn loaded() -> bool {
    LOADED.load(std::sync::atomic::Ordering::Relaxed)
}
static LOADED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Petey's model, loaded once on first use; `None` without local data (or if it failed to parse).
pub fn get() -> Option<&'static Model> {
    static M: std::sync::OnceLock<Option<Model>> = std::sync::OnceLock::new();
    M.get_or_init(|| {
        if std::env::var_os("MELEE_PETEY_NO_MODEL").is_some() {
            return None;
        }
        // load on a helper thread: its allocations stay on the system heap, outside the game arena that rollback
        // snapshots rewrite (the model must survive every load / resimulation)
        let r = std::thread::spawn(load).join().unwrap_or_else(|_| Err("loader panicked".into()));
        match &r {
            Ok(m) => eprintln!("petey: real model loaded ({} body triangles)", m.body.tris.len() / 3),
            Err(e) => eprintln!("petey: no real model ({e}); placeholder look"),
        }
        LOADED.store(r.is_ok(), std::sync::atomic::Ordering::Relaxed);
        r.ok()
    })
    .as_ref()
}

fn local_root() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("MELEE_BRAWL_LOCAL") {
        return Some(PathBuf::from(p));
    }
    let mut starts = vec![];
    if let Ok(c) = std::env::current_dir() {
        starts.push(c);
    }
    if let Ok(e) = std::env::current_exe() {
        if let Some(d) = e.parent() {
            starts.push(d.to_path_buf());
        }
    }
    for s in starts {
        let mut d: Option<&Path> = Some(&s);
        while let Some(p) = d {
            let c = p.join("_build").join("brawl");
            if c.join("disc").join("files").is_dir() {
                return Some(c);
            }
            d = p.parent();
        }
    }
    None
}

/// The adventure stage archive that carries the boss (found by scanning; see notes/petey-extract.md).
const STAGE: &str = "030101.pac";

fn load() -> Result<Model, String> {
    let root = local_root().ok_or("no local Brawl folder")?;
    let f = root.join("disc").join("files").join("stage").join("adventure").join(STAGE);
    let raw = std::fs::read(&f).map_err(|e| format!("boss archive not readable ({})", e.kind()))?;
    let mut leaves = vec![];
    walk(&unwrap(raw), 0, &mut leaves);
    let mut texs: Vec<(String, Tex)> = vec![];
    let mut mdls: Vec<(String, Vec<u8>, usize)> = vec![];
    let mut chrs: Vec<(String, Vec<u8>, usize)> = vec![];
    for b in &leaves {
        if b.len() < 0x10 || &b[..4] != b"bres" {
            continue;
        }
        let Some(entries) = brres_entries(b) else { continue };
        for (name, off) in entries {
            match b.get(off..off + 4) {
                Some(b"TEX0") => {
                    if !texs.iter().any(|(n, _)| *n == name) {
                        if let Some(t) = tex0(b, off) {
                            texs.push((name, t));
                        }
                    }
                }
                Some(b"CHR0") => {
                    if !chrs.iter().any(|(n, _, _)| *n == name) {
                        chrs.push((name, b.clone(), off));
                    }
                }
                Some(b"MDL0") => {
                    if !mdls.iter().any(|(n, _, _)| *n == name) {
                        mdls.push((name, b.clone(), off));
                    }
                }
                _ => {}
            }
        }
    }
    let mesh = |want: &str| -> Option<Mesh> {
        let (_, b, off) = mdls.iter().find(|(n, _, _)| n == want)?;
        // posed by the first frame of its idle animation (CHR0 `<model>Wait1`); bind pose without one
        let wait = format!("{want}Wait1");
        let pose = chrs.iter().find(|(n, _, _)| *n == wait).map(|(_, cb, co)| chr0_frame0(cb, *co)).unwrap_or_default();
        mdl0(b, *off, &texs, &pose)
    };
    let body = mesh("EnmBosspackun").ok_or("body model not found")?;
    Ok(Model { body, arm_l: mesh("WpnBosspackunArmL"), arm_r: mesh("WpnBosspackunArmR") })
}

// ---------------------------------------------------------------- containers

fn be16(d: &[u8], o: usize) -> u16 {
    d.get(o..o + 2).map(|s| u16::from_be_bytes([s[0], s[1]])).unwrap_or(0)
}
fn be32(d: &[u8], o: usize) -> u32 {
    d.get(o..o + 4).map(|s| u32::from_be_bytes([s[0], s[1], s[2], s[3]])).unwrap_or(0)
}
fn bei(d: &[u8], o: usize) -> i64 {
    be32(d, o) as i32 as i64
}
fn bef(d: &[u8], o: usize) -> f32 {
    f32::from_bits(be32(d, o))
}
fn at(base: usize, rel: i64) -> usize {
    (base as i64 + rel).max(0) as usize
}

/// Nintendo LZ77 (types 0x10 and 0x11). Returns the input unchanged if it does not look compressed.
fn unwrap(d: Vec<u8>) -> Vec<u8> {
    if d.len() < 8 || (d[0] != 0x10 && d[0] != 0x11) {
        return d;
    }
    lz(&d).unwrap_or(d)
}

fn lz(d: &[u8]) -> Option<Vec<u8>> {
    let t = d[0];
    let mut n = d[1] as usize | (d[2] as usize) << 8 | (d[3] as usize) << 16;
    let mut p = 4;
    if n == 0 {
        n = u32::from_le_bytes(d.get(4..8)?.try_into().ok()?) as usize;
        p = 8;
    }
    if n > 64 << 20 {
        return None;
    }
    let mut o = Vec::with_capacity(n);
    while o.len() < n {
        let f = *d.get(p)?;
        p += 1;
        for b in 0..8 {
            if o.len() >= n {
                break;
            }
            if f & (0x80 >> b) != 0 {
                let (l, dist);
                if t == 0x10 {
                    let x = (*d.get(p)? as usize) << 8 | *d.get(p + 1)? as usize;
                    p += 2;
                    l = (x >> 12) + 3;
                    dist = (x & 0xfff) + 1;
                } else {
                    let b0 = *d.get(p)? as usize;
                    match b0 >> 4 {
                        0 => {
                            let (b1, b2) = (*d.get(p + 1)? as usize, *d.get(p + 2)? as usize);
                            l = ((b0 & 0xf) << 4 | b1 >> 4) + 0x11;
                            dist = ((b1 & 0xf) << 8 | b2) + 1;
                            p += 3;
                        }
                        1 => {
                            let (b1, b2, b3) = (*d.get(p + 1)? as usize, *d.get(p + 2)? as usize, *d.get(p + 3)? as usize);
                            l = ((b0 & 0xf) << 12 | b1 << 4 | b2 >> 4) + 0x111;
                            dist = ((b2 & 0xf) << 8 | b3) + 1;
                            p += 4;
                        }
                        i => {
                            l = i + 1;
                            dist = ((b0 & 0xf) << 8 | *d.get(p + 1)? as usize) + 1;
                            p += 2;
                        }
                    }
                }
                if dist > o.len() {
                    return None;
                }
                for _ in 0..l {
                    let v = o[o.len() - dist];
                    o.push(v);
                }
            } else {
                o.push(*d.get(p)?);
                p += 1;
            }
        }
    }
    o.truncate(n);
    Some(o)
}

/// Every leaf (non-ARC) file in a (nested, possibly compressed) ARC.
fn walk(d: &[u8], depth: u32, out: &mut Vec<Vec<u8>>) {
    if depth > 6 || d.len() < 0x40 || &d[..4] != b"ARC\0" {
        out.push(d.to_vec());
        return;
    }
    let n = be16(d, 6) as usize;
    let mut p = 0x40;
    for _ in 0..n {
        let sz = be32(d, p + 4) as usize;
        let Some(body) = d.get(p + 0x20..p + 0x20 + sz) else { return };
        walk(&unwrap(body.to_vec()), depth + 1, out);
        p = (p + 0x20 + sz + 0x1f) & !0x1f;
    }
}

fn cstr(d: &[u8], o: usize) -> String {
    let e = d.get(o..).and_then(|s| s.iter().position(|&c| c == 0)).map(|e| o + e).unwrap_or(o);
    String::from_utf8_lossy(d.get(o..e).unwrap_or(&[])).into_owned()
}

/// A BRRES index group: (name, absolute data offset).
fn group(d: &[u8], g: usize) -> Vec<(String, usize)> {
    let n = be32(d, g + 4) as usize;
    (1..=n.min(4096))
        .map(|i| {
            let e = g + 8 + 16 * i;
            (cstr(d, at(g, bei(d, e + 8))), at(g, bei(d, e + 12)))
        })
        .collect()
}

/// All second-level entries of a BRRES (every folder's items).
fn brres_entries(d: &[u8]) -> Option<Vec<(String, usize)>> {
    let root = be16(d, 0xC) as usize;
    if d.get(root..root + 4)? != b"root" {
        return None;
    }
    let mut v = vec![];
    for (_, f) in group(d, root + 8) {
        v.extend(group(d, f));
    }
    Some(v)
}

// ---------------------------------------------------------------- textures

pub struct Tex {
    w: usize,
    h: usize,
    px: Vec<[u8; 4]>,
}

impl Tex {
    fn sample(&self, u: f32, v: f32) -> [u8; 4] {
        let fx = u - u.floor();
        let fy = v - v.floor();
        let x = ((fx * self.w as f32) as usize).min(self.w - 1);
        let y = ((fy * self.h as f32) as usize).min(self.h - 1);
        self.px[y * self.w + x]
    }
}

fn rgb565(c: u16) -> [u8; 4] {
    let r = ((c >> 11) & 31) as u8;
    let g = ((c >> 5) & 63) as u8;
    let b = (c & 31) as u8;
    [r << 3 | r >> 2, g << 2 | g >> 4, b << 3 | b >> 2, 255]
}

fn tex0(d: &[u8], o: usize) -> Option<Tex> {
    let (w, h, fmt) = (be16(d, o + 0x1C) as usize, be16(d, o + 0x1E) as usize, be32(d, o + 0x20));
    if w == 0 || h == 0 || w > 2048 || h > 2048 {
        return None;
    }
    let data = at(o, bei(d, o + 0x10));
    let mut px = vec![[128u8, 128, 128, 255]; w * h];
    let (bw, bh, bits) = match fmt {
        0 => (8, 8, 4),  // I4
        1 => (8, 4, 8),  // I8
        2 => (8, 4, 8),  // IA4
        3 => (4, 4, 16), // IA8
        4 => (4, 4, 16), // RGB565
        5 => (4, 4, 16), // RGB5A3
        6 => (4, 4, 32), // RGBA8
        14 => (8, 8, 4), // CMPR
        _ => return None,
    };
    let bx = w.div_ceil(bw);
    let by = h.div_ceil(bh);
    let block_bytes = bw * bh * bits / 8;
    let mut put = |x: usize, y: usize, c: [u8; 4]| {
        if x < w && y < h {
            px[y * w + x] = c;
        }
    };
    for ty in 0..by {
        for tx in 0..bx {
            let b = data + (ty * bx + tx) * block_bytes;
            d.get(b..b + block_bytes)?;
            match fmt {
                14 => {
                    for s in 0..4 {
                        let sb = b + s * 8;
                        let (c0, c1) = (be16(d, sb), be16(d, sb + 2));
                        let (a, bb) = (rgb565(c0), rgb565(c1));
                        let mix = |p: u16, q: u16, x: &[u8; 4], y: &[u8; 4]| -> [u8; 4] {
                            let m = |i: usize| ((x[i] as u16 * p + y[i] as u16 * q) / (p + q)) as u8;
                            [m(0), m(1), m(2), 255]
                        };
                        let pal = if c0 > c1 {
                            [a, bb, mix(2, 1, &a, &bb), mix(1, 2, &a, &bb)]
                        } else {
                            [a, bb, mix(1, 1, &a, &bb), [0, 0, 0, 0]]
                        };
                        let idx = be32(d, sb + 4);
                        for k in 0..16 {
                            let c = pal[((idx >> (30 - 2 * k)) & 3) as usize];
                            put(tx * 8 + (s & 1) * 4 + k % 4, ty * 8 + (s >> 1) * 4 + k / 4, c);
                        }
                    }
                }
                6 => {
                    for k in 0..16 {
                        let ar = b + k * 2;
                        let gb = b + 32 + k * 2;
                        put(tx * 4 + k % 4, ty * 4 + k / 4, [d[ar + 1], d[gb], d[gb + 1], d[ar]]);
                    }
                }
                4 | 5 => {
                    for k in 0..16 {
                        let c = be16(d, b + k * 2);
                        let col = if fmt == 4 {
                            rgb565(c)
                        } else if c & 0x8000 != 0 {
                            let f = |s: u16| ((s & 31) as u8) << 3 | ((s & 31) as u8) >> 2;
                            [f(c >> 10), f(c >> 5), f(c), 255]
                        } else {
                            let f = |s: u16| ((s & 15) as u8) * 17;
                            [f(c >> 8), f(c >> 4), f(c), ((c >> 12) & 7) as u8 * 36]
                        };
                        put(tx * 4 + k % 4, ty * 4 + k / 4, col);
                    }
                }
                _ => {
                    for k in 0..bw * bh {
                        let (i, a) = match fmt {
                            0 => {
                                let v = (d[b + k / 2] >> if k % 2 == 0 { 4 } else { 0 }) & 15;
                                (v * 17, 255)
                            }
                            1 => (d[b + k], 255),
                            2 => ((d[b + k] & 15) * 17, (d[b + k] >> 4) * 17),
                            _ => (d[b + k * 2 + 1], d[b + k * 2]),
                        };
                        put(tx * bw + k % bw, ty * bh + k / bw, [i, i, i, a]);
                    }
                }
            }
        }
    }
    Some(Tex { w, h, px })
}

// ---------------------------------------------------------------- MDL0 (v9, bind pose)

struct Arr {
    data: usize,
    comp: u32,
    div: u32,
    stride: usize,
    count: usize,
}

fn arr(d: &[u8], o: usize) -> Arr {
    Arr { data: at(o, bei(d, o + 8)), comp: be32(d, o + 0x18), div: d[o + 0x1C] as u32, stride: d[o + 0x1D] as usize, count: be16(d, o + 0x1E) as usize }
}

fn comp_read(d: &[u8], a: &Arr, i: usize, n: usize) -> [f32; 3] {
    let mut r = [0.0; 3];
    if i >= a.count {
        return r;
    }
    let base = a.data + i * a.stride;
    let sc = 1.0 / (1u32 << a.div.min(31)) as f32;
    for (k, rk) in r.iter_mut().enumerate().take(n) {
        *rk = match a.comp {
            0 => d.get(base + k).copied().unwrap_or(0) as f32 * sc,
            1 => d.get(base + k).copied().unwrap_or(0) as i8 as f32 * sc,
            2 => be16(d, base + 2 * k) as f32 * sc,
            3 => be16(d, base + 2 * k) as i16 as f32 * sc,
            _ => bef(d, base + 4 * k),
        };
    }
    r
}

fn color_read(d: &[u8], o: usize, i: usize) -> [u8; 4] {
    let data = at(o, bei(d, o + 8));
    let fmt = be32(d, o + 0x18);
    let stride = d[o + 0x1C] as usize;
    if i >= be16(d, o + 0x1E) as usize {
        return [255; 4];
    }
    let b = data + i * stride;
    match fmt {
        0 => rgb565(be16(d, b)),
        1 | 2 => [d[b], d[b + 1], d[b + 2], 255],
        3 => {
            let c = be16(d, b);
            let f = |s: u16| ((s & 15) as u8) * 17;
            [f(c >> 12), f(c >> 8), f(c >> 4), f(c)]
        }
        4 => {
            let c = (d[b] as u32) << 16 | (d[b + 1] as u32) << 8 | d[b + 2] as u32;
            let f = |s: u32| ((s & 63) as u8) << 2 | ((s & 63) as u8) >> 4;
            [f(c >> 18), f(c >> 12), f(c >> 6), f(c)]
        }
        _ => [d[b], d[b + 1], d[b + 2], d[b + 3]],
    }
}

type M34 = [[f32; 4]; 3];
const IDENT: M34 = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]];

fn mul(a: &M34, b: &M34) -> M34 {
    let mut r = [[0.0; 4]; 3];
    for i in 0..3 {
        for j in 0..4 {
            r[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j] + if j == 3 { a[i][3] } else { 0.0 };
        }
    }
    r
}

/// T * Rz * Ry * Rx * S (rotation in degrees), the NW4R bone order.
fn srt(sc: [f32; 3], rot: [f32; 3], tr: [f32; 3]) -> M34 {
    let (sx, cx) = rot[0].to_radians().sin_cos();
    let (sy, cy) = rot[1].to_radians().sin_cos();
    let (sz, cz) = rot[2].to_radians().sin_cos();
    let r = [
        [cy * cz, sx * sy * cz - cx * sz, cx * sy * cz + sx * sz],
        [cy * sz, sx * sy * sz + cx * cz, cx * sy * sz - sx * cz],
        [-sy, sx * cy, cx * cy],
    ];
    let mut m = [[0.0; 4]; 3];
    for i in 0..3 {
        for j in 0..3 {
            m[i][j] = r[i][j] * sc[j];
        }
        m[i][3] = tr[i];
    }
    m
}

fn xf(m: &M34, p: [f32; 3], w: f32) -> [f32; 3] {
    let f = |i: usize| m[i][0] * p[0] + m[i][1] * p[1] + m[i][2] * p[2] + m[i][3] * w;
    [f(0), f(1), f(2)]
}

/// Bone name -> scale / rotation (degrees) / translation tracks at frame 0 of a CHR0 (None: not animated).
fn chr0_frame0(d: &[u8], c: usize) -> Vec<(String, [Option<f32>; 9])> {
    let data = at(c, bei(d, c + 0x10));
    let mut out = vec![];
    for (name, e) in group(d, data) {
        let code = be32(d, e + 4);
        let mut v: [Option<f32>; 9] = [None; 9];
        let mut q = e + 8;
        // one track: a fixed value, or an offset (from the entry) to keyframes in the given format; frame 0 = key 0
        let track = |q: usize, fixed: bool, fmt: u32| -> f32 {
            if fixed {
                return bef(d, q);
            }
            let k = at(e, bei(d, q));
            match fmt {
                1 => bef(d, k + 12) + bef(d, k + 8) * ((be32(d, k + 16) >> 12) & 0xFFF) as f32,
                2 => bef(d, k + 12) + bef(d, k + 8) * be16(d, k + 18) as f32,
                3 => bef(d, k + 12),
                4 => bef(d, k + 4) + bef(d, k) * d.get(k + 8).copied().unwrap_or(0) as f32,
                5 => bef(d, k + 4) + bef(d, k) * be16(d, k + 8) as f32,
                _ => bef(d, k),
            }
        };
        if code & 2 == 0 {
            for (part, exists_bit, iso_bit, fixed_sh, fmt_sh, fmt_mask) in
                [(0usize, 22, 4, 13, 25, 3u32), (1, 23, 5, 16, 27, 7), (2, 24, 6, 19, 30, 3)]
            {
                if code & (1 << exists_bit) == 0 {
                    continue;
                }
                let fmt = (code >> fmt_sh) & fmt_mask;
                if code & (1 << iso_bit) != 0 {
                    let x = track(q, code & (1 << fixed_sh) != 0, fmt);
                    q += 4;
                    for a in 0..3 {
                        v[part * 3 + a] = Some(x);
                    }
                } else {
                    for a in 0..3 {
                        v[part * 3 + a] = Some(track(q, code & (1 << (fixed_sh + a)) != 0, fmt));
                        q += 4;
                    }
                }
            }
        }
        // flags that fix a part without data: identity, scale one, rotation + translation zero
        for a in 0..3 {
            if code & 2 != 0 || code & 8 != 0 {
                v[a].get_or_insert(1.0);
            }
            if code & 2 != 0 || code & 4 != 0 {
                v[3 + a].get_or_insert(0.0);
                v[6 + a].get_or_insert(0.0);
            }
        }
        out.push((name, v));
    }
    out
}

fn mdl0(d: &[u8], m: usize, texs: &[(String, Tex)], pose: &[(String, [Option<f32>; 9])]) -> Option<Mesh> {
    if be32(d, m + 8) != 9 {
        return None; // only the v9 layout is read
    }
    let sec = |i: usize| -> Vec<(String, usize)> {
        let s = bei(d, m + 0x10 + 4 * i);
        if s == 0 { vec![] } else { group(d, at(m, s)) }
    };
    let (defs, verts, norms, cols, uvs, mats, objs, texlinks) = (sec(0), sec(2), sec(3), sec(4), sec(5), sec(6), sec(8), sec(9));
    // skinning: each bone's posed world matrix (bind SRT where the pose has no track) x its inverse bind
    let bones = sec(1);
    let mut world: Vec<(usize, M34)> = vec![]; // (bone header offset, world)
    let mut node_mtx: Vec<(u32, M34)> = vec![]; // node id -> skin matrix
    let mut single: Vec<(u32, M34)> = vec![]; // node id -> world (single-bound objects)
    let mut skin: Vec<(u32, M34)> = vec![]; // bone node id -> world x inverse bind (weights of a mix)
    for (name, b) in &bones {
        let b = *b;
        let rd = |o: usize| [bef(d, b + o), bef(d, b + o + 4), bef(d, b + o + 8)];
        let (mut sc, mut ro, mut tr) = (rd(0x20), rd(0x2C), rd(0x38));
        if let Some((_, v)) = pose.iter().find(|(n, _)| n == name) {
            for a in 0..3 {
                sc[a] = v[a].unwrap_or(sc[a]);
                ro[a] = v[3 + a].unwrap_or(ro[a]);
                tr[a] = v[6 + a].unwrap_or(tr[a]);
            }
        }
        let local = srt(sc, ro, tr);
        let par = bei(d, b + 0x5C);
        let w = if par == 0 {
            local
        } else {
            world.iter().find(|(o, _)| *o == at(b, par)).map(|(_, pw)| mul(pw, &local)).unwrap_or(local)
        };
        world.push((b, w));
        let mut inv = [[0.0; 4]; 3];
        for (i, row) in inv.iter_mut().enumerate() {
            for (j, x) in row.iter_mut().enumerate() {
                *x = bef(d, b + 0xA0 + 16 * i + 4 * j);
            }
        }
        skin.push((be32(d, b + 0x10), mul(&w, &inv)));
        node_mtx.push((be32(d, b + 0x10), w)); // a vertex bound to one bone is stored in that bone's space
        single.push((be32(d, b + 0x10), w));
    }
    if let Some((_, o)) = defs.iter().find(|(n, _)| n == "NodeMix") {
        let mut p = *o;
        loop {
            match d.get(p) {
                Some(5) => p += 5,
                Some(3) => {
                    let (id, n) = (be16(d, p + 1) as u32, d[p + 3] as usize);
                    let mut acc = [[0.0f32; 4]; 3];
                    for k in 0..n {
                        let (bn, w) = (be16(d, p + 4 + 6 * k) as u32, bef(d, p + 6 + 6 * k));
                        if let Some((_, bm)) = skin.iter().find(|(i, _)| *i == bn) {
                            for i in 0..3 {
                                for j in 0..4 {
                                    acc[i][j] += w * bm[i][j];
                                }
                            }
                        }
                    }
                    node_mtx.push((id, acc));
                    p += 4 + 6 * n;
                }
                _ => break,
            }
        }
    }
    let node_of = |id: u32| node_mtx.iter().rev().find(|(i, _)| *i == id).map(|(_, m)| *m).unwrap_or(IDENT);
    // material header offset -> texture
    let mut mat_tex: Vec<(usize, usize)> = vec![];
    for (name, list) in &texlinks {
        let Some(ti) = texs.iter().position(|(n, _)| n == name) else { continue };
        for k in 0..be32(d, *list).min(256) as usize {
            mat_tex.push((at(*list, bei(d, list + 4 + 8 * k)), ti)); // (material, ref) pairs, list-relative
        }
    }
    // object index -> material header offset, from DrawOpa / DrawXlu
    let mut obj_mat: Vec<(usize, usize)> = vec![];
    for (n, o) in &defs {
        if !n.starts_with("Draw") {
            continue;
        }
        let mut p = *o;
        while d.get(p) == Some(&4) {
            let (mi, oi) = (be16(d, p + 1) as usize, be16(d, p + 3) as usize);
            if let Some((_, mo)) = mats.iter().find(|(_, mo)| be32(d, mo + 0xC) as usize == mi) {
                obj_mat.push((oi, *mo));
            }
            p += 8;
        }
    }
    let light = {
        let l = [0.35f32, 0.55, 0.75];
        let n = (l[0] * l[0] + l[1] * l[1] + l[2] * l[2]).sqrt();
        [l[0] / n, l[1] / n, l[2] / n]
    };
    let mut tris = vec![];
    let mut mn = [f32::MAX; 3];
    let mut mx = [f32::MIN; 3];
    for (oi, (_, o)) in objs.iter().enumerate() {
        let o = *o;
        let idx_of = |id: i16, set: &Vec<(String, usize)>| -> Option<usize> {
            if id < 0 { None } else { set.iter().map(|x| x.1).find(|a| be32(d, a + 0x10) as usize == id as usize) }
        };
        let id = |k: usize| be16(d, o + 0x48 + 2 * k) as i16;
        let pos = idx_of(id(0), &verts).map(|a| arr(d, a));
        let nrm = idx_of(id(1), &norms).map(|a| arr(d, a));
        let clr = idx_of(id(2), &cols);
        let uv = idx_of(id(4), &uvs).map(|a| arr(d, a));
        let Some(pos) = pos else { continue };
        let tex = obj_mat
            .iter()
            .find(|(i, _)| *i == oi)
            .and_then(|(_, mo)| mat_tex.iter().find(|(l, _)| *l == *mo))
            .map(|(_, ti)| &texs[*ti].1);
        let (lo, hi) = (be32(d, o + 0xC), be32(d, o + 0x10));
        // per-vertex attribute byte sizes in display-list order
        let mut lay: Vec<(u8, usize)> = vec![]; // (attr: 0 skip, 1 pos, 2 nrm, 3 clr0, 4 tex0), size
        if lo & 1 != 0 {
            lay.push((0, 1));
        }
        for k in 0..8 {
            if lo & (2 << k) != 0 {
                lay.push((0, 1));
            }
        }
        let sz = |v: u32| -> Option<usize> {
            match v & 3 {
                0 => Some(0),
                2 => Some(1),
                3 => Some(2),
                _ => None, // direct data: not handled
            }
        };
        for (attr, sh) in [(1u8, 9), (2, 11), (3, 13), (0, 15)] {
            let s = sz(lo >> sh)?;
            if s > 0 {
                lay.push((attr, s));
            }
        }
        for k in 0..8 {
            let s = sz(hi >> (2 * k))?;
            if s > 0 {
                lay.push((if k == 0 { 4 } else { 0 }, s));
            }
        }
        let vsize: usize = lay.iter().map(|l| l.1).sum();
        let obj_node = bei(d, o + 8);
        let obj_world = single.iter().find(|(i, _)| obj_node >= 0 && *i == obj_node as u32).map(|(_, m)| *m).unwrap_or(IDENT);
        let mut slots = [0u32; 10];
        let dl = at(o + 0x24, bei(d, o + 0x2C));
        let dl_end = dl + be32(d, o + 0x28) as usize;
        let mut p = dl;
        let vert = |p: usize, slots: &[u32; 10]| -> ([f32; 3], [u8; 4]) {
            let mut q = p;
            let (mut pi, mut ni, mut ci, mut ti) = (0, usize::MAX, usize::MAX, usize::MAX);
            let mtx_idx = if lo & 1 != 0 { d.get(q).copied().unwrap_or(0) as usize / 3 } else { 0 };
            let sm = if obj_node >= 0 { obj_world } else { node_of(slots[mtx_idx.min(9)]) };
            for (attr, s) in &lay {
                let v = if *s == 1 { d.get(q).copied().unwrap_or(0) as usize } else { be16(d, q) as usize };
                match attr {
                    1 => pi = v,
                    2 => ni = v,
                    3 => ci = v,
                    4 => ti = v,
                    _ => {}
                }
                q += s;
            }
            let pp = xf(&sm, comp_read(d, &pos, pi, 3), 1.0);
            let mut c = match (tex, &uv) {
                (Some(t), Some(u)) if ti != usize::MAX => {
                    let st = comp_read(d, u, ti, 2);
                    t.sample(st[0], st[1])
                }
                _ => [200, 200, 200, 255],
            };
            if let Some(co) = clr {
                if ci != usize::MAX {
                    let vc = color_read(d, co, ci);
                    for k in 0..3 {
                        c[k] = ((c[k] as u16 * (vc[k] as u16 + 64)) / 255).min(255) as u8;
                    }
                }
            }
            if let Some(na) = &nrm {
                if ni != usize::MAX {
                    let n = xf(&sm, comp_read(d, na, ni, 3), 0.0);
                    let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-6);
                    let n = [n[0] / l, n[1] / l, n[2] / l];
                    let dot = (n[0] * light[0] + n[1] * light[1] + n[2] * light[2]).max(0.0);
                    let s = 0.6 + 0.6 * dot;
                    for ck in c.iter_mut().take(3) {
                        *ck = (*ck as f32 * s).min(255.0) as u8;
                    }
                }
            }
            c[3] = 255;
            (pp, c)
        };
        while p < dl_end {
            let op = d.get(p).copied().unwrap_or(0);
            match op {
                0x00 => p += 1,
                0x08 => p += 6,
                0x10 => {
                    let n = ((be32(d, p + 1) >> 16) & 0xF) as usize + 1;
                    p += 5 + 4 * n;
                }
                0x20 => {
                    let v = be32(d, p + 1);
                    let slot = ((v & 0xFFF) / 12) as usize; // low 12 bits: XF address (top 4: size - 1)
                    if slot < 10 {
                        slots[slot] = v >> 16;
                    }
                    p += 5;
                }
                0x28 | 0x30 | 0x38 => p += 5,
                0x80..=0xBF => {
                    let n = be16(d, p + 1) as usize;
                    let vs: Vec<_> = (0..n).map(|i| vert(p + 3 + i * vsize, &slots)).collect();
                    p += 3 + n * vsize;
                    let mut emit = |a: usize, b: usize, c: usize| {
                        for i in [a, b, c] {
                            tris.push(vs[i]);
                        }
                    };
                    match op & 0xF8 {
                        0x80 => {
                            for q in (0..n / 4).map(|k| k * 4) {
                                emit(q, q + 1, q + 2);
                                emit(q, q + 2, q + 3);
                            }
                        }
                        0x90 => (0..n / 3).for_each(|k| emit(3 * k, 3 * k + 1, 3 * k + 2)),
                        0x98 => (2..n).for_each(|k| if k % 2 == 0 { emit(k - 2, k - 1, k) } else { emit(k - 1, k - 2, k) }),
                        0xA0 => (2..n).for_each(|k| emit(0, k - 1, k)),
                        _ => {}
                    }
                }
                _ => break,
            }
        }
    }
    for (p, _) in &tris {
        for k in 0..3 {
            mn[k] = mn[k].min(p[k]);
            mx[k] = mx[k].max(p[k]);
        }
    }
    if tris.is_empty() {
        return None;
    }
    // height from the 1st..99th percentile of y: a few far-off vertices (weights the pose leaves behind) must not
    // set the feet or the scale
    let mut ys: Vec<f32> = tris.iter().map(|(p, _)| p[1]).collect();
    ys.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
    mn[1] = ys[ys.len() / 100];
    mx[1] = ys[ys.len() - 1 - ys.len() / 100];
    Some(Mesh { tris, min: mn, max: mx })
}

// ---------------------------------------------------------------- draw

/// Draw a mesh: model x scale, yawed, its feet at `at` (world). Opaque, vertex colours, back faces kept.
pub unsafe fn draw_mesh(m: &Mesh, at: [f32; 3], scale: f32, yaw: f32, flash: bool) {
    draw_mesh_rel(m, m, at, scale, yaw, flash)
}

/// As `draw_mesh`, with the feet taken from `base` (a part authored in the same space as the body).
pub unsafe fn draw_mesh_rel(m: &Mesh, base: &Mesh, at: [f32; 3], scale: f32, yaw: f32, flash: bool) {
    use melee_runtime::tu::displayfunc_native::ext as gx;
    use melee_types::*;
    if m.tris.is_empty() {
        return;
    }
    melee_runtime::tu::state_native::HSD_StateInvalidate(-1);
    gx::GXSetColorUpdate(1);
    gx::GXSetAlphaUpdate(0);
    gx::GXSetBlendMode(GX_BM_NONE as _, GX_BL_ONE as _, GX_BL_ZERO as _, GX_LO_NOOP as _);
    gx::GXSetAlphaCompare(GX_ALWAYS as _, 0, GX_AOP_AND as _, GX_ALWAYS as _, 0);
    gx::GXSetZMode(1, GX_LEQUAL as _, 1);
    gx::GXSetZCompLoc(1);
    gx::GXSetNumTexGens(0);
    gx::GXSetNumTevStages(1);
    gx::GXSetTevOrder(GX_TEVSTAGE0 as _, GX_TEXCOORD_NULL as _, GX_TEXMAP_NULL as _, GX_COLOR0A0 as _);
    gx::GXSetTevOp(GX_TEVSTAGE0 as _, GX_PASSCLR as _);
    gx::GXSetNumChans(1);
    gx::GXSetChanCtrl(GX_COLOR0A0 as _, 0, GX_SRC_REG as _, GX_SRC_VTX as _, 0, GX_DF_NONE as _, GX_AF_NONE as _);
    gx::GXSetCullMode(GX_CULL_NONE as _);
    gx::GXClearVtxDesc();
    gx::GXSetVtxAttrFmt(GX_VTXFMT0 as _, GX_VA_POS as _, GX_POS_XYZ as _, GX_F32 as _, 0);
    gx::GXSetVtxAttrFmt(GX_VTXFMT0 as _, GX_VA_CLR0 as _, GX_CLR_RGBA as _, GX_RGBA8 as _, 0);
    gx::GXSetVtxDesc(GX_VA_POS as _, GX_DIRECT as _);
    gx::GXSetVtxDesc(GX_VA_CLR0 as _, GX_DIRECT as _);
    let cobj = melee_runtime::tu::cobj::HSD_CObjGetCurrent();
    if cobj.is_null() {
        return;
    }
    melee_runtime::tu::pobj::native::mx::GXLoadPosMtxImm((&mut (*cobj).view_mtx) as *mut Mtx as _, 0);
    gx::GXSetCurrentMtx(0);
    let (s, c) = (yaw.sin(), yaw.cos());
    let foot = base.min[1];
    // GX vertex counts are u16: send in chunks of whole triangles
    for chunk in m.tris.chunks(3 * 20000) {
        gx::GXBegin(GX_TRIANGLES as _, GX_VTXFMT0 as _, chunk.len() as _);
        for (p, col) in chunk {
            let (x, y, z) = (p[0] * scale, (p[1] - foot) * scale, p[2] * scale);
            gx::GXPosition3f32(at[0] + x * c + z * s, at[1] + y, at[2] - x * s + z * c);
            if flash {
                gx::GXColor4u8(255, 255, 255, 255);
            } else {
                gx::GXColor4u8(col[0], col[1], col[2], col[3]);
            }
        }
    }
    melee_runtime::tu::state_native::HSD_StateInvalidate(-1);
}
