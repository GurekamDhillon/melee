//! GameCube texture decoding, a port of Aurora's `lib/gfx/texture_convert.cpp` (CPU path) and
//! `gx::texture::texture_source_size` (`lib/gx/texture.cpp`).
//!
//! Every GC format is decoded to RGBA8 (row-major, mips concatenated), except the colour-index
//! formats C4/C8/C14X2, which decode to `u16` indices that [`convert_texture_palette`] then looks
//! up in a TLUT. Source bytes are the game's big-endian texture data, untouched. Verified bit for bit
//! against Aurora's own `convert_texture` / `convert_texture_palette` (tests/oracle.rs, `tex`).
use crate::consts::*;

/// Aurora `ConvertedTexture`: decoded bytes plus the arbitrary-mip heuristic result.
#[derive(Clone, Debug, Default)]
pub struct Converted {
    pub width: u32,
    pub height: u32,
    pub mips: u32,
    /// RGBA8 texels (or little-endian u16 indices for C4/C8/C14X2 via [`convert_texture`]).
    pub data: Vec<u8>,
    /// Aurora's `arb_mip_check`: mips that do not look like box-filtered copies of level 0.
    pub has_arbitrary_mips: bool,
}

const ARB_MIP_THRESHOLD: f32 = 15.0;

fn downscale(src: &[u8], sw: u32, sh: u32, dw: u32, dh: u32) -> Vec<u8> {
    let mut dst = vec![0u8; (dw * dh * 4) as usize];
    let off = |x: u32, y: u32, w: u32| ((y as usize) * (w as usize) + x as usize) * 4;
    for y in 0..dh {
        let y0 = (y * 2).min(sh - 1);
        let y1 = (y0 + 1).min(sh - 1);
        for x in 0..dw {
            let x0 = (x * 2).min(sw - 1);
            let x1 = (x0 + 1).min(sw - 1);
            let s = [off(x0, y0, sw), off(x1, y0, sw), off(x0, y1, sw), off(x1, y1, sw)];
            let o = off(x, y, dw);
            for c in 0..4 {
                let sum: u32 = s.iter().map(|&p| src[p + c] as u32).sum();
                dst[o + c] = ((sum + 2) / 4) as u8;
            }
        }
    }
    dst
}

fn avg_diff(lhs: &[u8], rhs: &[u8], w: u32, h: u32) -> f32 {
    let mut sum = 0.0f64;
    let n = (w * h * 4) as usize;
    for i in 0..n {
        let d = lhs[i] as i32 - rhs[i] as i32;
        sum += (d * d) as f64;
    }
    let count = w as f64 * h as f64 * 4.0;
    ((sum / count).sqrt() / 2.56) as f32
}

/// Aurora `arb_mip_check`.
fn arb_mip_check(mut w: u32, mut h: u32, mips: u32, data: &[u8]) -> bool {
    if mips < 2 {
        return false;
    }
    assert!(mips <= 10, "arb_mip_check: unsupported mip count {mips}");
    let mut levels = Vec::with_capacity(mips as usize);
    let mut off = 0usize;
    for _ in 0..mips {
        let sz = (w * h * 4) as usize;
        assert!(off + sz <= data.len(), "arb_mip_check: expected {} bytes, got {}", off + sz, data.len());
        levels.push((off, w, h));
        off += sz;
        w = (w >> 1).max(1);
        h = (h >> 1).max(1);
    }
    let mut down: Vec<u8> = Vec::new();
    let mut total = 0.0f32;
    for m in 0..(mips as usize - 1) {
        let (o, lw, lh) = levels[m];
        let (no, nw, nh) = levels[m + 1];
        let src: &[u8] = if m == 0 { &data[o..] } else { &down };
        let next = downscale(src, lw, lh, nw, nh);
        total += avg_diff(&data[no..], &next, nw, nh);
        down = next;
    }
    total / (mips - 1) as f32 > ARB_MIP_THRESHOLD
}

#[inline]
fn expand3(n: u8) -> u8 {
    (n << 5) | (n << 2) | (n >> 1)
}
#[inline]
fn expand(v: u32, n: u8) -> u8 {
    if v == 3 {
        return expand3(n);
    }
    // (n << (8 - v)) | (n >> ((v * 2) - 8)), in u8 arithmetic promoted to int then truncated
    let n = n as u32;
    ((n << (8 - v)) | (n >> (v * 2 - 8))) as u8
}
#[inline]
fn s3tc_blend(a: u32, b: u32) -> u8 {
    ((((a << 1) + a) + ((b << 2) + b)) >> 3) as u8
}
#[inline]
fn half_blend(a: u8, b: u8) -> u8 {
    ((a as u32 + b as u32) >> 1) as u8
}

fn mipped_texel_count(mut w: u32, mut h: u32, mips: u32) -> usize {
    let mut ret = (w * h) as usize;
    let mut i = mips;
    while i > 1 {
        if w > 1 {
            w /= 2;
        }
        if h > 1 {
            h /= 2;
        }
        ret += (w * h) as usize;
        i -= 1;
    }
    ret
}

/// One tiled decoder (Aurora's `TextureDecoder` concept). `decode` writes texel `x` of the current
/// row from `src` (the row's source bytes); `frac` source units per texel denominator.
#[derive(Clone, Copy)]
enum Tiled {
    I4,
    I8,
    IA4,
    IA8,
    C4,
    C8,
    C14X2,
    RGB565,
    RGB5A3,
}

impl Tiled {
    /// (block width, block height, bytes per block row): Aurora's BlockWidth / BlockHeight /
    /// `BlockWidth / Frac * sizeof(Source)`.
    fn geometry(self) -> (u32, u32, usize) {
        match self {
            Tiled::I4 | Tiled::C4 => (8, 8, 4),
            Tiled::I8 | Tiled::IA4 | Tiled::C8 => (8, 4, 8),
            Tiled::IA8 | Tiled::C14X2 | Tiled::RGB565 | Tiled::RGB5A3 => (4, 4, 8),
        }
    }
    /// Target size in bytes: RGBA8 or u16 index.
    fn target_size(self) -> usize {
        match self {
            Tiled::C4 | Tiled::C8 | Tiled::C14X2 => 2,
            _ => 4,
        }
    }
    fn decode(self, out: &mut [u8], src: &[u8], x: usize) {
        // Aurora reads u16 sources natively (little-endian on PC): IA8 keeps that order (intensity
        // = high byte = src[2x+1]); RGB565/RGB5A3/C14X2 bswap to the GC value.
        let be16 = |i: usize| u16::from_be_bytes([src[2 * i], src[2 * i + 1]]);
        match self {
            Tiled::I4 => {
                let v = expand(4, (src[x / 2] >> if x & 1 != 0 { 0 } else { 4 }) & 0xF);
                out.copy_from_slice(&[v, v, v, v]);
            }
            Tiled::I8 => {
                let v = src[x];
                out.copy_from_slice(&[v, v, v, v]);
            }
            Tiled::IA4 => {
                let v = expand(4, src[x] & 0xF);
                out.copy_from_slice(&[v, v, v, expand(4, src[x] >> 4)]);
            }
            Tiled::IA8 => {
                let le = u16::from_le_bytes([src[2 * x], src[2 * x + 1]]);
                let v = (le >> 8) as u8;
                out.copy_from_slice(&[v, v, v, (le & 0xFF) as u8]);
            }
            Tiled::C4 => {
                let v = ((src[x / 2] >> if x & 1 != 0 { 0 } else { 4 }) & 0xF) as u16;
                out.copy_from_slice(&v.to_le_bytes());
            }
            Tiled::C8 => out.copy_from_slice(&(src[x] as u16).to_le_bytes()),
            Tiled::C14X2 => out.copy_from_slice(&(be16(x) & 0x3FFF).to_le_bytes()),
            Tiled::RGB565 => {
                let t = be16(x) as u32;
                out.copy_from_slice(&[
                    expand(5, ((t >> 11) & 0x1F) as u8),
                    expand(6, ((t >> 5) & 0x3F) as u8),
                    expand(5, (t & 0x1F) as u8),
                    0xFF,
                ]);
            }
            Tiled::RGB5A3 => {
                let t = be16(x) as u32;
                if t & 0x8000 != 0 {
                    out.copy_from_slice(&[
                        expand(5, ((t >> 10) & 0x1F) as u8),
                        expand(5, ((t >> 5) & 0x1F) as u8),
                        expand(5, (t & 0x1F) as u8),
                        0xFF,
                    ]);
                } else {
                    out.copy_from_slice(&[
                        expand(4, ((t >> 8) & 0xF) as u8),
                        expand(4, ((t >> 4) & 0xF) as u8),
                        expand(4, (t & 0xF) as u8),
                        expand3(((t >> 12) & 0x7) as u8),
                    ]);
                }
            }
        }
    }
}

/// Aurora `DecodeTiled<T>`. Reads past the end of `data` as zero (Aurora would read garbage; the
/// oracle always passes `texture_source_size` bytes, so this never differs there).
fn decode_tiled(t: Tiled, width: u32, height: u32, mips: u32, data: &[u8]) -> Vec<u8> {
    let ts = t.target_size();
    let mut buf = vec![0u8; mipped_texel_count(width, height, mips) * ts];
    let (bw_, bh_, row_bytes) = t.geometry();
    let (mut w, mut h) = (width, height);
    let mut target_mip = 0usize;
    let mut inp = 0usize;
    let mut row = vec![0u8; row_bytes];
    for _ in 0..mips {
        let bw = w.div_ceil(bw_);
        let bh = h.div_ceil(bh_);
        for by in 0..bh {
            let base_y = by * bh_;
            let num_rows = (h - base_y).min(bh_);
            for bx in 0..bw {
                let base_x = bx * bw_;
                for y in 0..num_rows {
                    let target = target_mip + (((base_y + y) * w + base_x) as usize);
                    let n = (w - base_x).min(bw_) as usize;
                    for (i, b) in row.iter_mut().enumerate() {
                        *b = data.get(inp + i).copied().unwrap_or(0);
                    }
                    for x in 0..n {
                        let o = (target + x) * ts;
                        t.decode(&mut buf[o..o + ts], &row, x);
                    }
                    inp += row_bytes;
                }
                inp += row_bytes * (bh_ - num_rows) as usize;
            }
        }
        target_mip += (w * h) as usize;
        if w > 1 {
            w /= 2;
        }
        if h > 1 {
            h /= 2;
        }
    }
    buf
}

/// Aurora `DecodeLinear<T>` for the TLUT formats (IA8 / RGB565 / RGB5A3), one texel per u16.
fn decode_linear(t: Tiled, count: u32, data: &[u8]) -> Vec<u8> {
    let mut buf = vec![0u8; count as usize * 4];
    for x in 0..count as usize {
        let src = [data.get(2 * x).copied().unwrap_or(0), data.get(2 * x + 1).copied().unwrap_or(0)];
        t.decode(&mut buf[4 * x..4 * x + 4], &src, 0);
    }
    buf
}

/// Aurora `BuildRGBA8FromGCN` (GX_TF_RGBA8: AR and GB planes per 4x4 block).
fn decode_rgba8(width: u32, height: u32, mips: u32, data: &[u8]) -> Vec<u8> {
    let mut buf = vec![0u8; mipped_texel_count(width, height, mips) * 4];
    let (mut w, mut h) = (width, height);
    let mut target_mip = 0usize;
    let mut inp = 0usize;
    let rd = |i: usize| data.get(i).copied().unwrap_or(0);
    for _ in 0..mips {
        for by in 0..h.div_ceil(4) {
            let base_y = by * 4;
            let num_rows = (h - base_y).min(4);
            for bx in 0..w.div_ceil(4) {
                let base_x = bx * 4;
                let num_cols = (w - base_x).min(4);
                for c in 0..2 {
                    for y in 0..4 {
                        if y < num_rows {
                            let t = target_mip + ((base_y + y) * w + base_x) as usize;
                            for x in 0..num_cols as usize {
                                let o = (t + x) * 4;
                                if c != 0 {
                                    buf[o + 1] = rd(inp + x * 2);
                                    buf[o + 2] = rd(inp + x * 2 + 1);
                                } else {
                                    buf[o + 3] = rd(inp + x * 2);
                                    buf[o] = rd(inp + x * 2 + 1);
                                }
                            }
                        }
                        inp += 8;
                    }
                }
            }
        }
        target_mip += (w * h) as usize;
        if w > 1 {
            w /= 2;
        }
        if h > 1 {
            h /= 2;
        }
    }
    buf
}

/// Aurora `BuildRGBA8FromCMPR` (GC S3TC: 2x2 sub-blocks per 8x8, big-endian colours, the 3-colour
/// mode's fourth entry is the midpoint with alpha 0).
fn decode_cmpr(width: u32, height: u32, mips: u32, data: &[u8]) -> Vec<u8> {
    let mut buf = vec![0u8; mipped_texel_count(width, height, mips) * 4];
    let (mut w, mut h) = (width, height);
    let mut dst = 0usize;
    let mut src = 0usize;
    let rd = |i: usize| data.get(i).copied().unwrap_or(0);
    for _ in 0..mips {
        let mut yy = 0;
        while yy < h {
            let mut xx = 0;
            while xx < w {
                for yb in [0u32, 4] {
                    for xb in [0u32, 4] {
                        let c1 = u16::from_be_bytes([rd(src), rd(src + 1)]);
                        let c2 = u16::from_be_bytes([rd(src + 2), rd(src + 3)]);
                        src += 4;
                        let mut t = [0u8; 16];
                        t[0] = expand(5, ((c1 >> 11) & 0x1F) as u8);
                        t[1] = expand(6, ((c1 >> 5) & 0x3F) as u8);
                        t[2] = expand(5, (c1 & 0x1F) as u8);
                        t[3] = 0xFF;
                        t[4] = expand(5, ((c2 >> 11) & 0x1F) as u8);
                        t[5] = expand(6, ((c2 >> 5) & 0x3F) as u8);
                        t[6] = expand(5, (c2 & 0x1F) as u8);
                        t[7] = 0xFF;
                        if c1 > c2 {
                            for k in 0..3 {
                                t[8 + k] = s3tc_blend(t[4 + k] as u32, t[k] as u32);
                                t[12 + k] = s3tc_blend(t[k] as u32, t[4 + k] as u32);
                            }
                            t[11] = 0xFF;
                            t[15] = 0xFF;
                        } else {
                            for k in 0..3 {
                                t[8 + k] = half_blend(t[k], t[4 + k]);
                                t[12 + k] = t[8 + k];
                            }
                            t[11] = 0xFF;
                            t[15] = 0;
                        }
                        for y in 0..4u32 {
                            let mut bits = rd(src + y as usize);
                            for x in 0..4u32 {
                                if xx + xb + x >= w || yy + yb + y >= h {
                                    continue;
                                }
                                let o = dst + (((yy + yb + y) * w + (xx + xb + x)) * 4) as usize;
                                let ci = (((bits >> 6) & 3) as usize) * 4;
                                buf[o..o + 4].copy_from_slice(&t[ci..ci + 4]);
                                bits <<= 2;
                            }
                        }
                        src += 4;
                    }
                }
                xx += 8;
            }
            yy += 8;
        }
        dst += (w * h * 4) as usize;
        if w > 1 {
            w /= 2;
        }
        if h > 1 {
            h /= 2;
        }
    }
    buf
}

/// Aurora `convert_texture` for the GC formats (the `_PC` formats are Aurora upload formats the
/// game never uses; not ported).
pub fn convert_texture(format: u32, width: u32, height: u32, mips: u32, data: &[u8]) -> Converted {
    let data_out = match format {
        GX_TF_I4 => decode_tiled(Tiled::I4, width, height, mips, data),
        GX_TF_I8 => decode_tiled(Tiled::I8, width, height, mips, data),
        // Z textures (GXSetZTexture reads the last TEV stage's texel bytes, shader.rs): Z8 is I8's 8x4 tiling of 8-bit
        // texels (every channel), Z16 IA8's (A the high byte, I the low), Z24X8 RGBA8's AR/GB planes (X in A, z bits
        // 23-16 / 15-8 / 7-0 in R / G / B). HSD_EraseRect binds a 4x4 Z8 one; a copy-made Z24X8 one is an EFB copy.
        GX_TF_Z8 => decode_tiled(Tiled::I8, width, height, mips, data),
        GX_TF_Z16 => decode_tiled(Tiled::IA8, width, height, mips, data),
        GX_TF_Z24X8 => decode_rgba8(width, height, mips, data),
        GX_TF_IA4 => decode_tiled(Tiled::IA4, width, height, mips, data),
        GX_TF_IA8 => decode_tiled(Tiled::IA8, width, height, mips, data),
        GX_TF_C4 => decode_tiled(Tiled::C4, width, height, mips, data),
        GX_TF_C8 => decode_tiled(Tiled::C8, width, height, mips, data),
        GX_TF_C14X2 => decode_tiled(Tiled::C14X2, width, height, mips, data),
        GX_TF_RGB565 => decode_tiled(Tiled::RGB565, width, height, mips, data),
        GX_TF_RGB5A3 => decode_tiled(Tiled::RGB5A3, width, height, mips, data),
        GX_TF_RGBA8 => decode_rgba8(width, height, mips, data),
        GX_TF_CMPR => decode_cmpr(width, height, mips, data),
        _ => panic!("convert_texture: unknown texture format {format}"),
    };
    // to_wgpu(format) is RGBA8Unorm for every non-index GC format.
    let rgba = !matches!(format, GX_TF_C4 | GX_TF_C8 | GX_TF_C14X2);
    let has_arbitrary_mips = rgba && mips > 1 && arb_mip_check(width, height, mips, &data_out);
    Converted { width, height, mips, data: data_out, has_arbitrary_mips }
}

/// Aurora `tlut_texture_format`: GX_TL_* -> the GX_TF_* it decodes as.
pub fn tlut_texture_format(tlut_fmt: u32) -> u32 {
    match tlut_fmt {
        GX_TL_IA8 => GX_TF_IA8,
        GX_TL_RGB565 => GX_TF_RGB565,
        GX_TL_RGB5A3 => GX_TF_RGB5A3,
        _ => panic!("tlut_texture_format: unsupported tlut format {tlut_fmt}"),
    }
}

/// Aurora `convert_tlut`: `entries` u16 palette entries -> RGBA8.
pub fn convert_tlut(tex_format: u32, entries: u32, data: &[u8]) -> Vec<u8> {
    match tex_format {
        GX_TF_IA8 => decode_linear(Tiled::IA8, entries, data),
        GX_TF_RGB565 => decode_linear(Tiled::RGB565, entries, data),
        GX_TF_RGB5A3 => decode_linear(Tiled::RGB5A3, entries, data),
        _ => panic!("convert_tlut: unsupported tlut format {tex_format}"),
    }
}

/// Aurora `convert_texture_palette`: index texture + TLUT -> RGBA8. Indices past the TLUT read
/// transparent black.
pub fn convert_texture_palette(
    format: u32,
    width: u32,
    height: u32,
    mips: u32,
    tex: &[u8],
    tlut_format: u32,
    tlut_entries: u16,
    tlut: &[u8],
) -> Converted {
    let idx = convert_texture(format, width, height, mips, tex);
    let pal = convert_tlut(tlut_texture_format(tlut_format), tlut_entries as u32, tlut);
    let mut pixels = Vec::with_capacity(idx.data.len() * 2);
    let (mut mw, mut mh) = (width, height);
    let mut off = 0usize;
    for _ in 0..mips {
        let n = (mw * mh) as usize;
        for i in 0..n {
            let j = 2 * (off + i);
            let index = u16::from_le_bytes([idx.data[j], idx.data[j + 1]]) as usize;
            if index >= tlut_entries as usize {
                pixels.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                pixels.extend_from_slice(&pal[index * 4..index * 4 + 4]);
            }
        }
        off += n;
        mw = (mw >> 1).max(1);
        mh = (mh >> 1).max(1);
    }
    let has_arbitrary_mips = arb_mip_check(width, height, mips, &pixels);
    Converted { width, height, mips, data: pixels, has_arbitrary_mips }
}

/// Aurora `texture::texture_source_size`: GC bytes a `format` texture of this size occupies.
pub fn texture_source_size(format: u32, width: u32, height: u32, mips: u32) -> usize {
    if width == 0 || height == 0 || mips == 0 {
        return 0;
    }
    let mut total = 0u64;
    for mip in 0..mips {
        let mw = (width >> mip).max(1);
        let mh = (height >> mip).max(1);
        let (bx, by, bytes) = match format {
            GX_TF_I4 | GX_TF_C4 | GX_TF_CMPR => (8, 8, 32),
            GX_TF_I8 | GX_TF_Z8 | GX_TF_IA4 | GX_TF_C8 => (8, 4, 32),
            GX_TF_IA8 | GX_TF_Z16 | GX_TF_RGB565 | GX_TF_RGB5A3 | GX_TF_C14X2 => (4, 4, 32),
            GX_TF_RGBA8 | GX_TF_Z24X8 => (4, 4, 64),
            _ => return 0,
        };
        total += (mw.div_ceil(bx) * mh.div_ceil(by)) as u64 * bytes;
    }
    total as usize
}

/// Aurora `texture::tlut_source_size`.
pub fn tlut_source_size(entries: u16) -> usize {
    entries as usize * 2
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn expand_matches_aurora_template() {
        // ExpandTo8<5>(31) = 255, <6>(63) = 255, <4>(15) = 255, <3>(7) = 255, 0 -> 0
        assert_eq!(expand(5, 31), 255);
        assert_eq!(expand(6, 63), 255);
        assert_eq!(expand(4, 15), 255);
        assert_eq!(expand3(7), 255);
        assert_eq!(expand(5, 0), 0);
        assert_eq!(expand(5, 16), 0x84);
    }
}
