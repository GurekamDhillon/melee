//! UI text: real fonts as signed-distance glyphs on the UI atlas (gd-ui-v1.1).
//!
//! Four faces ship with the host (rust/melee_gx/assets/fonts, SIL OFL 1.1, subset by
//! tools/rust-port/fonts/subset_fonts.py): Inter Regular/Bold and JetBrains Mono Regular/Bold. Every glyph of the fixed
//! set (ASCII, Latin-1, the kit symbols) is rasterized once, when the atlas is built, at 4x supersampling and turned
//! into a single-channel SDF tile (`SDF_EM` texels per em, `SDF_RANGE` texels of spread) in the atlas alpha. The
//! shader decodes it with a linear sampler (instance flag [`crate::ui::Q_SDF`]), so one tile serves every size and DPI
//! and text stays inside the single UI batch.
//!
//! Layout is table lookups only: per-face advances, bearings and an ASCII kern table baked at build. Laying out a
//! string never allocates and never touches the font files again.

use crate::ui::{glyph_char, Atlas, AtlasRect, ATLAS, N_GLYPHS};
use ab_glyph::{Font, FontRef, ScaleFont};

pub const FACE_SANS: usize = 0;
pub const FACE_SANS_BOLD: usize = 1;
pub const FACE_MONO: usize = 2;
pub const FACE_MONO_BOLD: usize = 3;
pub const N_FACES: usize = 4;

static FONT_FILES: [&[u8]; N_FACES] = [
    include_bytes!("../assets/fonts/Inter-Regular.ttf"),
    include_bytes!("../assets/fonts/Inter-Bold.ttf"),
    include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf"),
    include_bytes!("../assets/fonts/JetBrainsMono-Bold.ttf"),
];

/// SDF tile resolution: texels per em, and the distance spread (texels each side of the edge).
pub const SDF_EM: f32 = 24.0;
pub const SDF_RANGE: f32 = 3.0;
const SS: usize = 4;

/// ASCII printable chars get pair kerning (index = char - 0x20).
const KN: usize = 95;

/// One glyph of one face, in em units relative to the pen on the baseline (y down).
#[derive(Clone, Copy, Debug, Default)]
pub struct Glyph {
    pub tile: AtlasRect,
    /// Tile top-left offset from the pen.
    pub ox: f32,
    pub oy: f32,
    pub adv: f32,
}

pub struct Face {
    pub glyphs: [Glyph; N_GLYPHS],
    /// Cap height (em), for centering text in its line box.
    pub cap: f32,
    kern: Box<[f32]>,
}

impl Face {
    #[inline]
    pub fn kern(&self, a: char, b: char) -> f32 {
        let (a, b) = (a as u32, b as u32);
        if self.kern.is_empty() || !(0x20..0x7F).contains(&a) || !(0x20..0x7F).contains(&b) {
            return 0.0;
        }
        self.kern[(a - 0x20) as usize * KN + (b - 0x20) as usize]
    }
}

/// A resolved text style: pixel size (line height), face, uppercase.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextStyle {
    pub size: f32,
    pub face: usize,
    pub upper: bool,
}

impl TextStyle {
    pub fn new(size: f32, bold: bool, mono: bool) -> TextStyle {
        TextStyle { size, face: (mono as usize) * 2 + bold as usize, upper: false }
    }
}

/// Reusable scratch for building tiles (only used while the atlas is built).
#[derive(Default)]
struct Scratch {
    cov: Vec<f32>,
    f: Vec<f32>,
    din: Vec<f32>,
    dout: Vec<f32>,
    v: Vec<usize>,
    z: Vec<f32>,
    d1: Vec<f32>,
}

/// Felzenszwalb-Huttenlocher 1D squared distance transform of `f` (n values) into `d`.
fn edt1(f: &[f32], d: &mut [f32], v: &mut [usize], z: &mut [f32]) {
    let n = f.len();
    let mut k = 0usize;
    v[0] = 0;
    z[0] = f32::NEG_INFINITY;
    z[1] = f32::INFINITY;
    for q in 1..n {
        loop {
            let p = v[k];
            let s = ((f[q] + (q * q) as f32) - (f[p] + (p * p) as f32)) / (2.0 * q as f32 - 2.0 * p as f32);
            if s <= z[k] && k > 0 {
                k -= 1;
                continue;
            }
            if s <= z[k] {
                // k == 0 and the new parabola dominates everywhere
                v[0] = q;
                z[0] = f32::NEG_INFINITY;
                z[1] = f32::INFINITY;
                break;
            }
            k += 1;
            v[k] = q;
            z[k] = s;
            z[k + 1] = f32::INFINITY;
            break;
        }
    }
    k = 0;
    for (q, dq) in d.iter_mut().enumerate().take(n) {
        while z[k + 1] < q as f32 {
            k += 1;
        }
        let p = v[k];
        let dx = q as f32 - p as f32;
        *dq = dx * dx + f[p];
    }
}

/// 2D squared EDT: distance of every cell to the nearest cell where `seed(i)` holds.
fn edt2(s: &mut Scratch, w: usize, h: usize, inside: bool, out_is_din: bool) {
    const BIG: f32 = 1.0e12;
    let n = w.max(h);
    s.f.resize(n, 0.0);
    s.d1.resize(n, 0.0);
    s.v.resize(n + 1, 0);
    s.z.resize(n + 2, 0.0);
    let mut out = std::mem::take(if out_is_din { &mut s.din } else { &mut s.dout });
    out.clear();
    out.resize(w * h, 0.0);
    for (i, o) in out.iter_mut().enumerate() {
        *o = if (s.cov[i] >= 0.5) == inside { 0.0 } else { BIG };
    }
    for x in 0..w {
        for y in 0..h {
            s.f[y] = out[y * w + x];
        }
        edt1(&s.f[..h], &mut s.d1[..h], &mut s.v, &mut s.z);
        for y in 0..h {
            out[y * w + x] = s.d1[y];
        }
    }
    for y in 0..h {
        s.f[..w].copy_from_slice(&out[y * w..y * w + w]);
        edt1(&s.f[..w], &mut s.d1[..w], &mut s.v, &mut s.z);
        out[y * w..y * w + w].copy_from_slice(&s.d1[..w]);
    }
    if out_is_din {
        s.din = out;
    } else {
        s.dout = out;
    }
}

fn build_face(atlas: &mut Atlas, idx: usize, fallback: Option<&Face>, s: &mut Scratch) -> Face {
    let font = FontRef::try_from_slice(FONT_FILES[idx]).expect("bundled UI font");
    let upem = font.units_per_em().unwrap_or(1000.0);
    let hi = SDF_EM * SS as f32; // supersampled px per em
    let sf = font.as_scaled(ab_glyph::PxScale::from(hi * font.height_unscaled() / upem));
    let pad = (SDF_RANGE as usize + 1) * SS;
    let mut glyphs = [Glyph::default(); N_GLYPHS];
    for (i, slot) in glyphs.iter_mut().enumerate() {
        let c = glyph_char(i);
        let gid = font.glyph_id(c);
        if gid.0 == 0 {
            if let Some(fb) = fallback {
                *slot = fb.glyphs[i];
            }
            continue;
        }
        let adv = sf.h_advance(gid) / hi;
        *slot = Glyph { adv, ..Default::default() };
        let Some(og) = font.outline_glyph(gid.with_scale_and_position(sf.scale(), ab_glyph::point(0.0, 0.0))) else { continue };
        let b = og.px_bounds();
        let (x0, y0) = (b.min.x.floor() as i32 - pad as i32, b.min.y.floor() as i32 - pad as i32);
        let bw = (b.max.x.ceil() as i32 - x0) as usize + pad;
        let bh = (b.max.y.ceil() as i32 - y0) as usize + pad;
        let (tw, th) = (bw.div_ceil(SS), bh.div_ceil(SS));
        let (w, h) = (tw * SS, th * SS);
        s.cov.clear();
        s.cov.resize(w * h, 0.0);
        let (offx, offy) = ((b.min.x.floor() as i32 - x0) as usize, (b.min.y.floor() as i32 - y0) as usize);
        og.draw(|x, y, v| {
            let (px, py) = (x as usize + offx, y as usize + offy);
            if px < w && py < h {
                s.cov[py * w + px] = v;
            }
        });
        edt2(s, w, h, true, true); // distance to the nearest inside cell
        edt2(s, w, h, false, false); // distance to the nearest outside cell
        let Some(r) = atlas.alloc(tw as u16, th as u16) else { continue };
        let range_hi = SDF_RANGE * SS as f32;
        for ty in 0..th {
            for tx in 0..tw {
                let mut acc = 0.0;
                for sy in 0..SS {
                    for sx in 0..SS {
                        let o = (ty * SS + sy) * w + tx * SS + sx;
                        // signed distance in hi px, positive inside
                        acc += if s.cov[o] >= 0.5 { s.dout[o].sqrt() - 0.5 } else { 0.5 - s.din[o].sqrt() };
                    }
                }
                let d = acc / (SS * SS) as f32;
                let a = (0.5 + d / (2.0 * range_hi)).clamp(0.0, 1.0);
                let o = (((r.y as usize + ty) * ATLAS as usize) + r.x as usize + tx) * 4;
                atlas.px[o..o + 4].copy_from_slice(&[255, 255, 255, (a * 255.0 + 0.5) as u8]);
            }
        }
        atlas.mark_pub(r);
        *slot = Glyph { tile: r, ox: x0 as f32 / hi, oy: y0 as f32 / hi, adv };
    }
    let cap = font.glyph_bounds(&ab_glyph::Glyph { id: font.glyph_id('H'), scale: sf.scale(), position: ab_glyph::point(0.0, 0.0) }).min.y / -hi;
    let mut kern;
    {
        let ids: Vec<_> = (0x20u8..0x7F).map(|c| font.glyph_id(c as char)).collect();
        let mut any = false;
        kern = vec![0.0f32; KN * KN];
        for (a, &ia) in ids.iter().enumerate() {
            for (b, &ib) in ids.iter().enumerate() {
                let k = sf.kern(ia, ib) / hi;
                any |= k != 0.0;
                kern[a * KN + b] = k;
            }
        }
        if !any {
            kern.clear();
        }
    }
    Face { glyphs, cap, kern: kern.into_boxed_slice() }
}

/// Build the four faces onto `atlas` (mono first: it carries the symbols the sans faces lack).
pub fn build_faces(atlas: &mut Atlas) -> [Face; N_FACES] {
    let mut s = Scratch::default();
    let mono = build_face(atlas, FACE_MONO, None, &mut s);
    let mono_b = build_face(atlas, FACE_MONO_BOLD, Some(&mono), &mut s);
    let sans = build_face(atlas, FACE_SANS, Some(&mono), &mut s);
    let sans_b = build_face(atlas, FACE_SANS_BOLD, Some(&mono_b), &mut s);
    [sans, sans_b, mono, mono_b]
}

/// Line placement: baseline offset from the line top for a line box of `size` (cap height centred).
#[inline]
pub fn baseline(face: &Face, size: f32) -> f32 {
    (size + face.cap * size) * 0.5
}

/// Lay out `s` (one line). Returns (chars that fit, width). `max_w` <= 0: no limit.
pub fn fit(faces: &[Face; N_FACES], st: TextStyle, s: &str, max_w: f32) -> (usize, f32) {
    let f = &faces[st.face];
    let mut w = 0.0f32;
    let mut prev: Option<char> = None;
    let mut n = 0usize;
    for ch in s.chars() {
        let ch = if st.upper { ch.to_ascii_uppercase() } else { ch };
        let k = prev.map_or(0.0, |p| f.kern(p, ch));
        let nw = w + (k + f.glyphs[crate::ui::glyph_index(ch)].adv) * st.size;
        if max_w > 0.0 && nw > max_w + 0.01 {
            break;
        }
        w = nw;
        n += 1;
        prev = Some(ch);
    }
    (n, w)
}

/// Emit the glyph quads of the first `n` chars of `s` with the pen at (x, line top y): `emit(rect, uv)`.
pub fn emit(faces: &[Face; N_FACES], st: TextStyle, s: &str, n: usize, x: f32, y: f32, mut out: impl FnMut([f32; 4], [f32; 4])) {
    let f = &faces[st.face];
    let by = y + baseline(f, st.size);
    let mut pen = x;
    let mut prev: Option<char> = None;
    let px = st.size / SDF_EM; // canvas units per tile texel
    for ch in s.chars().take(n) {
        let ch = if st.upper { ch.to_ascii_uppercase() } else { ch };
        if let Some(p) = prev {
            pen += f.kern(p, ch) * st.size;
        }
        let g = &f.glyphs[crate::ui::glyph_index(ch)];
        if g.tile.w > 0 {
            out([pen + g.ox * st.size, by + g.oy * st.size, g.tile.w as f32 * px, g.tile.h as f32 * px], g.tile.uv());
        }
        pen += g.adv * st.size;
        prev = Some(ch);
    }
}

/// Placeholder faces (no glyphs) while the atlas is being built.
pub fn empty_faces() -> [Face; N_FACES] {
    std::array::from_fn(|_| Face { glyphs: [Glyph::default(); N_GLYPHS], cap: 0.7, kern: Box::new([]) })
}
