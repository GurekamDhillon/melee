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
//! and the two arm / cage MDL0s, their TEX0 textures (CMPR / RGB / I formats) and their CHR0 animations. Each part
//! is kept as a rig (bones, matrix sources, unlit triangles); the draw poses it per frame from a CHR0 (bone
//! matrices -> skinning -> a fixed light). Which animation and frame is chosen by `petey::draw` from the sim state.
#![allow(clippy::missing_safety_doc)]

use std::path::{Path, PathBuf};

pub struct Model {
    pub body: Rig,
    pub arm_l: Option<Rig>,
    pub arm_r: Option<Rig>,
    /// The boss's enemy param file (hurtbox sets, camera range), when found.
    pub params: Option<Params>,
}

/// One hurtbox record (BrawlBox "HurtBox" layout): offset and stretch in the bone's space, radius, flag word whose
/// top byte is the bone index.
#[derive(Clone, Copy, Debug)]
pub struct Hurt {
    pub off: [f32; 3],
    pub stretch: [f32; 3],
    pub r: f32,
    pub flags: u32,
}

/// What is read from the boss's enemy param file (a relocatable section: header, data, relocations, named roots).
#[derive(Default, Debug)]
pub struct Params {
    pub body: Vec<Hurt>,
    pub arm_l: Vec<Hurt>,
    pub arm_r: Vec<Hurt>,
    /// cameraRangeSet floats, CameraClipSphereData floats
    pub camera_range: Vec<f32>,
    pub camera_clip: Vec<f32>,
}

fn params(d: &[u8]) -> Option<Params> {
    if d.len() < 0x20 || be32(d, 0) as usize != d.len() {
        return None;
    }
    let (ds, rc, r1, r2) = (be32(d, 4) as usize, be32(d, 8) as usize, be32(d, 12) as usize, be32(d, 16) as usize);
    let t = 0x20 + ds + rc * 4;
    let names = t + (r1 + r2) * 8;
    let root = |want: &str| -> Option<usize> {
        (0..r1 + r2).find_map(|i| {
            let (o, n) = (be32(d, t + 8 * i) as usize, be32(d, t + 8 * i + 4) as usize);
            (cstr(d, names + n) == want).then_some(0x20 + o)
        })
    };
    // a set: (data offset of the records, count)
    let set = |want: &str| -> Vec<Hurt> {
        let Some(r) = root(want) else { return vec![] };
        let (p, n) = (0x20 + be32(d, r) as usize, be32(d, r + 4) as usize);
        (0..n.min(16))
            .map(|i| {
                let q = p + 0x20 * i;
                let v = |o: usize| [bef(d, q + o), bef(d, q + o + 4), bef(d, q + o + 8)];
                Hurt { off: v(0), stretch: v(12), r: bef(d, q + 24), flags: be32(d, q + 28) }
            })
            .collect()
    };
    let floats = |want: &str, n: usize| -> Vec<f32> {
        let Some(r) = root(want) else { return vec![] };
        let p = 0x20 + be32(d, r) as usize;
        (0..n).map(|i| bef(d, p + 4 * i)).collect()
    };
    let body = set("HitDataSet");
    if body.is_empty() {
        return None;
    }
    Some(Params {
        body,
        arm_l: set("wpnHitDataSet_ArmLeft"),
        arm_r: set("wpnHitDataSet_ArmRight"),
        camera_range: floats("cameraRangeSet", 8),
        camera_clip: floats("CameraClipSphereData", 5),
    })
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
            Ok(m) => eprintln!("petey: real model loaded ({} animations)", m.body.anims.len()),
            Err(e) => eprintln!("petey: no real model ({e}); placeholder look"),
        }
        LOADED.store(r.is_ok(), std::sync::atomic::Ordering::Relaxed);
        r.ok()
    })
    .as_ref()
}

/// Whether the player's own Brawl files are present (the boss archive under the local Brawl folder). Envoy requires them
/// (the full-game plan, Q3: no Brawl disc, no Envoy); checked once per process, never prints the path.
pub fn brawl_present() -> bool {
    static P: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *P.get_or_init(|| {
        if std::env::var_os("MELEE_BRAWL_ABSENT").is_some() {
            return false; // test knob: behave as if the files were missing
        }
        local_root().is_some_and(|r| r.join("disc").join("files").join("stage").join("adventure").join(STAGE).is_file())
    })
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
    let par = leaves.iter().find_map(|b| params(b));
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
    let rig = |want: &str| -> Option<Rig> {
        let (_, b, off) = mdls.iter().find(|(n, _, _)| n == want)?;
        let mut r = mdl0(b, *off, &texs)?;
        // its animations: CHR0 `<model><Name>`, kept by suffix
        for (n, cb, co) in &chrs {
            if let Some(suf) = n.strip_prefix(want) {
                if !suf.is_empty() && !suf.starts_with("Arm") {
                    r.anims.push((suf.to_string(), chr0(cb, *co)));
                }
            }
        }
        // extent from the first frame of the idle (bind pose without one): size and depth centre stay put while animating
        let mut tris = vec![];
        r.skin(r.anim("Wait1"), 0.0, &mut tris);
        (r.min, r.max) = extent(&tris);
        Some(r)
    };
    let body = rig("EnmBosspackun").ok_or("body model not found")?;
    Ok(Model { body, arm_l: rig("WpnBosspackunArmL"), arm_r: rig("WpnBosspackunArmR"), params: par })
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

// ---------------------------------------------------------------- MDL0 (v9) arrays

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

// ---------------------------------------------------------------- CHR0

/// One animated channel.
enum Track {
    Fixed(f32),
    /// Hermite keys: (frame, value, tangent).
    Keys(Vec<[f32; 3]>),
    /// One value per frame (linear between them).
    Baked(Vec<f32>),
}

impl Track {
    fn at(&self, f: f32) -> f32 {
        match self {
            Track::Fixed(v) => *v,
            Track::Baked(v) => {
                if v.is_empty() {
                    return 0.0;
                }
                let i = (f.max(0.0) as usize).min(v.len() - 1);
                let j = (i + 1).min(v.len() - 1);
                let t = (f - i as f32).clamp(0.0, 1.0);
                v[i] + (v[j] - v[i]) * t
            }
            Track::Keys(k) => {
                if k.is_empty() {
                    return 0.0;
                }
                if f <= k[0][0] {
                    return k[0][1];
                }
                for w in k.windows(2) {
                    let (a, b) = (w[0], w[1]);
                    if f <= b[0] {
                        let span = b[0] - a[0];
                        if span <= 0.0 {
                            return b[1];
                        }
                        let t = (f - a[0]) / span;
                        let (t2, t3) = (t * t, t * t * t);
                        return (2.0 * t3 - 3.0 * t2 + 1.0) * a[1]
                            + (t3 - 2.0 * t2 + t) * span * a[2]
                            + (-2.0 * t3 + 3.0 * t2) * b[1]
                            + (t3 - t2) * span * b[2];
                    }
                }
                k[k.len() - 1][1]
            }
        }
    }
}

/// A CHR0: its length and, per bone name, the nine channels (scale xyz, rotation xyz degrees, translation xyz).
pub struct Anim {
    pub frames: f32,
    tracks: Vec<(String, [Option<Track>; 9])>,
}

fn chr0(d: &[u8], c: usize) -> Anim {
    let ver = be32(d, c + 8);
    let frames = be16(d, c + if ver >= 5 { 0x20 } else { 0x1C }) as usize;
    let data = at(c, bei(d, c + 0x10));
    let mut tracks = vec![];
    for (name, e) in group(d, data) {
        let code = be32(d, e + 4);
        let mut v: [Option<Track>; 9] = Default::default();
        let mut q = e + 8;
        // one channel: a fixed value, or an offset (from the entry) to keyframes in the given format
        let track = |q: usize, fixed: bool, fmt: u32| -> Track {
            if fixed {
                return Track::Fixed(bef(d, q));
            }
            let k = at(e, bei(d, q));
            match fmt {
                1..=3 => {
                    let n = be16(d, k) as usize;
                    let (step, base) = (bef(d, k + 8), bef(d, k + 12));
                    let mut keys = Vec::with_capacity(n.min(4096));
                    for i in 0..n.min(4096) {
                        keys.push(match fmt {
                            1 => {
                                let w = be32(d, k + 16 + 4 * i);
                                let tan = (((w & 0xFFF) as i32) << 20 >> 20) as f32 / 32.0;
                                [(w >> 24) as f32, base + step * ((w >> 12) & 0xFFF) as f32, tan]
                            }
                            2 => {
                                let o = k + 16 + 6 * i;
                                [be16(d, o) as f32 / 32.0, base + step * be16(d, o + 2) as f32, be16(d, o + 4) as i16 as f32 / 256.0]
                            }
                            _ => {
                                // the float-key header is 8 bytes (count, pad, frame scale): no step / base
                                let o = k + 8 + 12 * i;
                                [bef(d, o), bef(d, o + 4), bef(d, o + 8)]
                            }
                        });
                    }
                    Track::Keys(keys)
                }
                // baked rotations: one angle per frame, a byte or a u16 over the full turn (no header)
                4 | 5 => {
                    let (step, base) = (bef(d, k), bef(d, k + 4));
                    Track::Baked(
                        (0..=frames)
                            .map(|i| {
                                let x = if fmt == 4 { d.get(k + 8 + i).copied().unwrap_or(0) as f32 } else { be16(d, k + 8 + 2 * i) as f32 };
                                base + step * x
                            })
                            .collect(),
                    )
                }
                _ => Track::Baked((0..=frames).map(|i| bef(d, k + 4 * i)).collect()),
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
                    let fixed = code & (1 << fixed_sh) != 0;
                    for a in 0..3 {
                        v[part * 3 + a] = Some(track(q, fixed, fmt));
                    }
                    q += 4;
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
                v[a].get_or_insert(Track::Fixed(1.0));
            }
            if code & 2 != 0 || code & 4 != 0 {
                v[3 + a].get_or_insert(Track::Fixed(0.0));
                v[6 + a].get_or_insert(Track::Fixed(0.0));
            }
        }
        tracks.push((name, v));
    }
    Anim { frames: frames.max(1) as f32, tracks }
}

// ---------------------------------------------------------------- MDL0 -> skinned rig

struct Bone {
    name: String,
    /// the bone's own index (MDL0 bone header +0x0C), what param files refer to
    index: u32,
    srt: [[f32; 3]; 3],
    parent: Option<usize>,
    inv: M34,
}

/// Where a vertex's matrix comes from.
enum Src {
    Ident,
    /// stored in that bone's space: the bone's world
    Bone(usize),
    /// weights x (world x inverse bind)
    Mix(Vec<(usize, f32)>),
}

/// One body part ready to pose: bones, matrix sources and a triangle list in source space.
/// Scratch for drawing a rig (see [`Rig::draw_bufs`]).
pub struct DrawBufs {
    world: Vec<M34>,
    mats: Vec<M34>,
    pub tris: Vec<([f32; 3], [u8; 4])>,
}

pub struct Rig {
    bones: Vec<Bone>,
    srcs: Vec<Src>,
    /// (source, position, normal (zero: none), unlit colour)
    verts: Vec<(u16, [f32; 3], [f32; 3], [u8; 4])>,
    /// animations by suffix (the CHR0 name after the model name: `Wait1`, `BasketAttackL`, ...)
    pub anims: Vec<(String, Anim)>,
    /// Extent in the first frame of `Wait1` (size and depth centre stay fixed while animating; the ground is y 0).
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl Rig {
    pub fn anim(&self, name: &str) -> Option<&Anim> {
        self.anims.iter().find(|(n, _)| n == name).map(|(_, a)| a)
    }

    /// Bone slot by its MDL0 index (param files name bones by index).
    pub fn bone_by_index(&self, i: u32) -> Option<usize> {
        self.bones.iter().position(|b| b.index == i)
    }
    /// Bone slot whose name ends with `suffix` (arm bones carry the model name as a prefix).
    pub fn bone_named(&self, suffix: &str) -> Option<usize> {
        self.bones.iter().position(|b| b.name.ends_with(suffix))
    }
    pub fn bone_name(&self, slot: usize) -> &str {
        self.bones.get(slot).map(|b| b.name.as_str()).unwrap_or("")
    }
    /// A point given in a bone's local space, in model space at `frame` of `anim`.
    pub fn bone_point(&self, anim: Option<&Anim>, frame: f32, slot: usize, local: [f32; 3]) -> [f32; 3] {
        let w = self.pose_bones(anim, frame);
        w.get(slot).map(|m| xf(m, local, 1.0)).unwrap_or(local)
    }

    /// Each bone's world matrix at `frame` of `anim` (bind pose where a bone has no channel).
    fn pose_bones(&self, anim: Option<&Anim>, frame: f32) -> Vec<M34> {
        let mut world: Vec<M34> = Vec::with_capacity(self.bones.len());
        self.pose_bones_into(anim, frame, &mut world);
        world
    }

    /// [`Self::pose_bones`] into a caller's buffer (no allocation once it holds `bones` matrices).
    fn pose_bones_into(&self, anim: Option<&Anim>, frame: f32, world: &mut Vec<M34>) {
        world.clear();
        for b in &self.bones {
            let mut s = b.srt;
            if let Some((_, v)) = anim.and_then(|a| a.tracks.iter().find(|(n, _)| *n == b.name)) {
                for (k, t) in v.iter().enumerate() {
                    if let Some(t) = t {
                        s[k / 3][k % 3] = t.at(frame);
                    }
                }
            }
            let local = srt(s[0], s[1], s[2]);
            world.push(match b.parent {
                Some(p) if p < world.len() => mul(&world[p], &local),
                _ => local,
            });
        }
    }

    /// Buffers sized for [`Self::skin_with`]: (bone matrices, skin matrices, vertices). Make them off the game thread
    /// (a helper thread) so that drawing never allocates inside the game scope, which rollback rewrites.
    pub fn draw_bufs(&self) -> DrawBufs {
        DrawBufs { world: Vec::with_capacity(self.bones.len()), mats: Vec::with_capacity(self.srcs.len()), tris: Vec::with_capacity(self.verts.len()) }
    }

    /// Skin the triangle list for `frame` of `anim` into `out` (model space, lit colour).
    pub fn skin(&self, anim: Option<&Anim>, frame: f32, out: &mut Vec<([f32; 3], [u8; 4])>) {
        let mut b = DrawBufs { world: Vec::new(), mats: Vec::new(), tris: core::mem::take(out) };
        self.skin_with(anim, frame, &mut b);
        *out = b.tris;
    }

    /// [`Self::skin`] into `b.tris`, with no allocation when `b` came from [`Self::draw_bufs`].
    pub fn skin_with(&self, anim: Option<&Anim>, frame: f32, b: &mut DrawBufs) {
        let DrawBufs { world, mats, tris: out } = b;
        self.pose_bones_into(anim, frame, world);
        let world = &*world;
        mats.clear();
        mats.extend(self
            .srcs
            .iter()
            .map(|s| match s {
                Src::Ident => IDENT,
                Src::Bone(i) => world.get(*i).copied().unwrap_or(IDENT),
                Src::Mix(ws) => {
                    let mut acc = [[0.0f32; 4]; 3];
                    for (bi, w) in ws {
                        if let (Some(bw), Some(b)) = (world.get(*bi), self.bones.get(*bi)) {
                            let m = mul(bw, &b.inv);
                            for i in 0..3 {
                                for j in 0..4 {
                                    acc[i][j] += w * m[i][j];
                                }
                            }
                        }
                    }
                    acc
                }
            }));
        let light = {
            let l = [0.35f32, 0.55, 0.75];
            let n = (l[0] * l[0] + l[1] * l[1] + l[2] * l[2]).sqrt();
            [l[0] / n, l[1] / n, l[2] / n]
        };
        out.clear();
        out.reserve(self.verts.len());
        for (si, p, n, c) in &self.verts {
            let m = &mats[*si as usize];
            let mut c = *c;
            if *n != [0.0; 3] {
                let n = xf(m, *n, 0.0);
                let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-6);
                let dot = ((n[0] * light[0] + n[1] * light[1] + n[2] * light[2]) / l).max(0.0);
                let s = 0.6 + 0.6 * dot;
                for ck in c.iter_mut().take(3) {
                    *ck = (*ck as f32 * s).min(255.0) as u8;
                }
            }
            out.push((xf(m, *p, 1.0), c));
        }
    }
}

fn mdl0(d: &[u8], m: usize, texs: &[(String, Tex)]) -> Option<Rig> {
    if be32(d, m + 8) != 9 {
        return None; // only the v9 layout is read
    }
    let sec = |i: usize| -> Vec<(String, usize)> {
        let s = bei(d, m + 0x10 + 4 * i);
        if s == 0 { vec![] } else { group(d, at(m, s)) }
    };
    let (defs, verts, norms, cols, uvs, mats, objs, texlinks) = (sec(0), sec(2), sec(3), sec(4), sec(5), sec(6), sec(8), sec(9));
    // bones in file order (parents come first); node id -> matrix source
    let mut bones: Vec<Bone> = vec![];
    let mut offs: Vec<usize> = vec![];
    let mut srcs: Vec<Src> = vec![Src::Ident];
    let mut node_src: Vec<(u32, u16)> = vec![];
    let mut bone_of_node: Vec<(u32, usize)> = vec![];
    for (name, b) in &sec(1) {
        let b = *b;
        let rd = |o: usize| [bef(d, b + o), bef(d, b + o + 4), bef(d, b + o + 8)];
        let par = bei(d, b + 0x5C);
        let parent = if par == 0 { None } else { offs.iter().position(|o| *o == at(b, par)) };
        let mut inv = [[0.0; 4]; 3];
        for (i, row) in inv.iter_mut().enumerate() {
            for (j, x) in row.iter_mut().enumerate() {
                *x = bef(d, b + 0xA0 + 16 * i + 4 * j);
            }
        }
        let bi = bones.len();
        bones.push(Bone { name: name.clone(), index: be32(d, b + 0x0C), srt: [rd(0x20), rd(0x2C), rd(0x38)], parent, inv });
        offs.push(b);
        let id = be32(d, b + 0x10);
        srcs.push(Src::Bone(bi));
        node_src.push((id, (srcs.len() - 1) as u16));
        bone_of_node.push((id, bi));
    }
    if let Some((_, o)) = defs.iter().find(|(n, _)| n == "NodeMix") {
        let mut p = *o;
        loop {
            match d.get(p) {
                Some(5) => p += 5,
                Some(3) => {
                    let (id, n) = (be16(d, p + 1) as u32, d[p + 3] as usize);
                    let mut ws = vec![];
                    for k in 0..n {
                        let (bn, w) = (be16(d, p + 4 + 6 * k) as u32, bef(d, p + 6 + 6 * k));
                        if let Some((_, bi)) = bone_of_node.iter().find(|(i, _)| *i == bn) {
                            ws.push((*bi, w));
                        }
                    }
                    srcs.push(Src::Mix(ws));
                    node_src.push((id, (srcs.len() - 1) as u16));
                    p += 4 + 6 * n;
                }
                _ => break,
            }
        }
    }
    let src_of = |id: u32| node_src.iter().rev().find(|(i, _)| *i == id).map(|(_, s)| *s).unwrap_or(0);
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
    let mut tris = vec![];
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
        let obj_src = if obj_node >= 0 {
            bone_of_node.iter().find(|(i, _)| *i == obj_node as u32).map(|(_, bi)| (*bi + 1) as u16)
        } else {
            None
        };
        let mut slots = [0u32; 10];
        let dl = at(o + 0x24, bei(d, o + 0x2C));
        let dl_end = dl + be32(d, o + 0x28) as usize;
        let mut p = dl;
        let vert = |p: usize, slots: &[u32; 10]| -> (u16, [f32; 3], [f32; 3], [u8; 4]) {
            let mut q = p;
            let (mut pi, mut ni, mut ci, mut ti) = (0, usize::MAX, usize::MAX, usize::MAX);
            let mtx_idx = if lo & 1 != 0 { d.get(q).copied().unwrap_or(0) as usize / 3 } else { 0 };
            let si = obj_src.unwrap_or_else(|| src_of(slots[mtx_idx.min(9)]));
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
            c[3] = 255;
            let n = match &nrm {
                Some(na) if ni != usize::MAX => comp_read(d, na, ni, 3),
                _ => [0.0; 3],
            };
            (si, comp_read(d, &pos, pi, 3), n, c)
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
    if tris.is_empty() {
        return None;
    }
    Some(Rig { bones, srcs, verts: tris, anims: vec![], min: [0.0; 3], max: [0.0; 3] })
}

/// Extent of a posed triangle list; y from the 1st..99th percentile (a few far-off vertices must not set the feet
/// or the scale).
pub fn extent(tris: &[([f32; 3], [u8; 4])]) -> ([f32; 3], [f32; 3]) {
    let mut mn = [f32::MAX; 3];
    let mut mx = [f32::MIN; 3];
    for (p, _) in tris {
        for k in 0..3 {
            mn[k] = mn[k].min(p[k]);
            mx[k] = mx[k].max(p[k]);
        }
    }
    if !tris.is_empty() {
        let mut ys: Vec<f32> = tris.iter().map(|(p, _)| p[1]).collect();
        ys.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        mn[1] = ys[ys.len() / 100];
        mx[1] = ys[ys.len() - 1 - ys.len() / 100];
    }
    (mn, mx)
}

// ---------------------------------------------------------------- draw

/// Draw a posed triangle list: model x scale, yawed, with model-space y `foot` at `at` (world). Opaque, vertex
/// colours, back faces kept.
/// `org`: the model-space ground height and depth centre that land on `at` (x stays: the model's x is its own).
pub unsafe fn draw_tris(tris: &[([f32; 3], [u8; 4])], org: [f32; 2], at: [f32; 3], scale: f32, yaw: f32, flash: bool) {
    use melee_runtime::tu::displayfunc_native::ext as gx;
    use melee_types::*;
    if tris.is_empty() {
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
    // GX vertex counts are u16: send in chunks of whole triangles
    for chunk in tris.chunks(3 * 20000) {
        gx::GXBegin(GX_TRIANGLES as _, GX_VTXFMT0 as _, chunk.len() as _);
        for (p, col) in chunk {
            let (x, y, z) = (p[0] * scale, (p[1] - org[0]) * scale, (p[2] - org[1]) * scale);
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

#[cfg(test)]
mod tests {
    /// With the owner's local data present (skipped otherwise): every clip decodes so that the arm models' attach
    /// bones follow the body's frame for frame, and the feet stay on y 0 in every grounded clip.
    #[test]
    fn arms_follow_body_and_feet_on_ground() {
        let Ok(m) = super::load() else { return };
        let b = &m.body;
        let bi = |r: &super::Rig, n: &str| r.bones.iter().position(|x| x.name.ends_with(n)).unwrap();
        let mut t = vec![];
        for (an, a) in &b.anims {
            for f in 0..a.frames as i32 {
                let w = b.pose_bones(Some(a), f as f32);
                for (r, n) in [(m.arm_l.as_ref().unwrap(), "Attach01N"), (m.arm_r.as_ref().unwrap(), "Attach02N")] {
                    let wa = r.pose_bones(r.anim(an), f as f32);
                    let (x, y) = (w[bi(b, n)], wa[bi(r, n)]);
                    for i in 0..3 {
                        for j in 0..4 {
                            assert!((x[i][j] - y[i][j]).abs() < 0.5, "{an} f{f} {n}: arm attach off the body");
                        }
                    }
                }
                if an != "JumpAttack" {
                    b.skin(Some(a), f as f32, &mut t);
                    let lo = super::extent(&t).0[1];
                    assert!((-2.0..6.0).contains(&lo), "{an} f{f}: lowest point {lo}");
                }
            }
        }
    }
}

#[cfg(test)]
mod probe {
    /// Local-only probe (needs the owner's files; prints derived numbers only).
    #[test]
    #[ignore]
    fn petey_probe() {
        let m = super::load().expect("local data");
        let p = m.params.as_ref().expect("params");
        eprintln!("params {:?}", p);
        for (nm, r) in [("body", Some(&m.body)), ("armL", m.arm_l.as_ref()), ("armR", m.arm_r.as_ref())] {
            let r = r.unwrap();
            eprintln!("{nm} extent {:?} {:?}", r.min, r.max);
            for i in 0..r.bones.len() {
                eprintln!("  {nm} slot {i} idx {} {}", r.bones[i].index, r.bones[i].name);
            }
            for (an, a) in &r.anims {
                eprintln!("  {nm} anim {an} frames {}", a.frames);
            }
        }
        let w = m.arm_l.as_ref().unwrap();
        for b in ["XRotN", "LWeakN", "LIronNa", "PeachBodyM"] {
            let s = w.bone_named(b).unwrap();
            for f in [0.0, 40.0, 80.0, 120.0] {
                eprintln!("armL {b} BasketAttackL f{f} {:?}", w.bone_point(w.anim("BasketAttackL"), f, s, [0.0; 3]));
            }
        }
    }
}
