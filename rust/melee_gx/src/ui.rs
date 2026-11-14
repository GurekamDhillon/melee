//! The UI layer's batched quad + atlas renderer (gd-ui-v1, docs/rust-port/notes/wasm-ui-lab-api.md section 0).
//!
//! Every UI element (flat, textured and sheared quads, 9-slice pieces, SDF glyphs from [`crate::ui_font`]) is one [`UiQuad`] instance of a single
//! instanced pipeline that samples one atlas page. The quads are drawn inside the existing EFB blit pass
//! ([`crate::render::Renderer::blit_to`]): no second surface, no readback, no extra submit, and one draw call per
//! present (clipping is per instance in the fragment shader, so a `CLIP` never splits the batch).
//!
//! Two coordinate spaces (GAP-0 of lab-ui-spec.md): `BAND` (the default, 640x480 canvas units on the 4:3 band, which
//! follows the EFB blit rect, letterboxing included) and `WINDOW` (window pixels, so panels can sit in the 16:9 side
//! bars). The band rect is passed to the shader per present.
//!
//! Nothing here exists until a UI mod is loaded: the renderer's `ui` field stays `None`, so the blit pass is unchanged
//! with mods off. The CPU side allocates only at a new high-water mark (the instance vector, the GPU buffer).

use crate::render::Renderer;

/// Atlas page size (texels, square, RGBA8).
pub const ATLAS: u32 = 4096;
/// Mip levels of the atlas (level 0 included). Every allocation is aligned to `CELL` texels and followed by a `CELL`
/// gutter, so a texel of the coarsest level never mixes two entries (uiquality).
pub const ATLAS_MIPS: u32 = 4;
pub const CELL: u16 = 1 << (ATLAS_MIPS - 1);

/// Instance flag: the quad (and its clip rect) is in window pixels, not band units.
pub const Q_WINDOW: u32 = 1;
/// Instance flag: the quad samples a signed-distance glyph tile (linear filtering, alpha = coverage of d > 0.5).
pub const Q_SDF: u32 = 2;
/// Instance flags: this side (left, top, right, bottom) is shared with an abutting piece of the same shape (a 9-slice,
/// a tiled bar). Edge AA on a slanted or rotated quad skips it, so the pieces meet with no coverage ramp and no seam.
pub const Q_IN_L: u32 = 4;
pub const Q_IN_T: u32 = 8;
pub const Q_IN_R: u32 = 16;
pub const Q_IN_B: u32 = 32;
/// Instance flag: the quad shows a whole atlas entry (not a cell of a sheet), so the texels round it are the atlas's
/// clear gutter. The quad then reaches one pixel past its rect and filtering may read into the gutter: the art's edge
/// fades out through the filter instead of being cut off at the rect (a minified sprite keeps its outline).
pub const Q_FULL: u32 = 64;
/// Instance flags bits 16..19: an analytic shape (uiquality). The quad samples no texture: the fragment shader
/// evaluates the shape's signed distance in device pixels, so it is exact and antialiased at every size, resolution,
/// rotation and tilt, and one quad is the whole shape (no pieces, so no seams). `shape` = [corner radius, border
/// width, glow spread] in the quad's units; `rgba` fills, `rgba2` is the border (drawn inside the outline).
pub const Q_SHAPE_SHIFT: u32 = 16;
pub const Q_SHAPE_MASK: u32 = 15 << Q_SHAPE_SHIFT;
/// A rounded box (a circle or a pill when the radius reaches half the short side); with a border and a clear fill,
/// a ring. `shape[2]` > 0 dashes the border: that long on, 0.8 of it off (the period is fitted to the perimeter).
pub const SHAPE_BOX: u32 = 1;
/// A diamond inscribed in the rect.
pub const SHAPE_DIAMOND: u32 = 2;
/// A rounded box with a soft falloff reaching `shape[2]` outside it (the rect already includes the spread).
pub const SHAPE_GLOW: u32 = 3;

/// One UI instance: 96 bytes, uploaded as-is (`#[repr(C)]`, little-endian like the GPU).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UiQuad {
    /// x, y, w, h in the quad's space.
    pub rect: [f32; 4],
    /// u0, v0, u1, v1 on the atlas (0..1).
    pub uv: [f32; 4],
    /// x0, y0, x1, y1: pixels outside are discarded (same space as `rect`).
    pub clip: [f32; 4],
    /// 0xRRGGBBAA, straight alpha.
    pub rgba: u32,
    /// x offset of the top edge per unit of height (a parallelogram: the kit's sheared bars).
    pub shear: f32,
    pub flags: u32,
    /// Rotation in radians (clockwise on screen) about `piv` (same space as `rect`); 0 = none (envoy-v3-ui.md).
    pub rot: f32,
    pub piv: [f32; 2],
    /// `[effect id, effect colour 0xRRGGBBAA]` ([`FX_NONE`] .. [`FX_GLOW`], envoy-fx.md); 0 draws exactly as before.
    pub fx: [u32; 2],
    /// Shapes: [corner radius, border width, glow spread] in the quad's units. SDF text: `shape[1]` widens the glyph
    /// by that much of the distance range (an outline pass under the fill).
    pub shape: [f32; 3],
    /// Shapes: the border colour, 0xRRGGBBAA.
    pub rgba2: u32,
}

/// UI quad effects (`gd_ui_node_effect`, the `OP_EFFECT` record): presentation only, timed by the render clock.
pub const FX_NONE: u32 = 0;
/// A foil sheen sweeping across the quad's texture (Epic stickers), tinted by the effect colour.
pub const FX_FOIL: u32 = 1;
/// A stronger holographic treatment: rainbow bands, a sweep and glints (Unique stickers).
pub const FX_HOLO: u32 = 2;
/// A soft pulsing glow: the quad's coverage in the effect colour, breathing (the hovered sticker's halo).
pub const FX_GLOW: u32 = 3;
pub const FX_COUNT: u32 = 4;

/// No clipping.
pub const NO_CLIP: [f32; 4] = [-1.0e9, -1.0e9, 1.0e9, 1.0e9];

/// A rectangle on the atlas, in texels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AtlasRect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

impl AtlasRect {
    pub fn uv(&self) -> [f32; 4] {
        let s = 1.0 / ATLAS as f32;
        [self.x as f32 * s, self.y as f32 * s, (self.x + self.w) as f32 * s, (self.y + self.h) as f32 * s]
    }
    /// The uv of a sub-rectangle given in this rect's texels.
    pub fn sub_uv(&self, x: f32, y: f32, w: f32, h: f32) -> [f32; 4] {
        let s = 1.0 / ATLAS as f32;
        let (bx, by) = (self.x as f32 + x, self.y as f32 + y);
        [bx * s, by * s, (bx + w) * s, (by + h) * s]
    }
}

/// Host slice pieces (lab-ui-spec.md 7.3): rounded fills and borders per theme radius, and the glow. Each is drawn
/// as one analytic shape quad (uiquality); ids match `GD_UI_SLICE_*` in docs/specs/gd-ui-lab-v1.h.
#[derive(Clone, Copy, Debug)]
pub struct SliceDef {
    pub radius: f32,
    /// 0: filled; else the border width.
    pub border: f32,
    /// The glow's spread outside the shape (0: none).
    pub glow: f32,
}

pub const SLICES: [SliceDef; 9] = [
    SliceDef { radius: 0.0, border: 0.0, glow: 0.0 },   // 0 unused
    SliceDef { radius: 4.0, border: 0.0, glow: 0.0 },   // 1 FILL_R4
    SliceDef { radius: 6.0, border: 0.0, glow: 0.0 },   // 2 FILL_R6
    SliceDef { radius: 14.0, border: 0.0, glow: 0.0 },  // 3 FILL_R14
    SliceDef { radius: 4.0, border: 1.0, glow: 0.0 },   // 4 LINE_R4
    SliceDef { radius: 6.0, border: 1.0, glow: 0.0 },   // 5 LINE_R6
    SliceDef { radius: 14.0, border: 3.0, glow: 0.0 },  // 6 LINE_R14_3
    SliceDef { radius: 6.0, border: 0.0, glow: 14.0 },  // 7 GLOW_R6
    SliceDef { radius: 1.0, border: 0.0, glow: 0.0 },   // 8 FILL_R1 (frame-meter cells)
];

/// Built-in kit pieces (`KIT` records): name, size in texels.
pub const KIT_PIECES: [(&str, u16, u16); 6] = [
    ("", 0, 0),
    ("ui.dot", 10, 10),
    ("ui.ring", 56, 56),
    ("ui.diamond", 9, 9),
    ("ui.knob", 32, 32),
    ("ui.hatch", 9, 14),
];

/// Glyphs: ASCII 0x20..0x7F, Latin-1 0xA0..0xFF, then the kit's symbols.
const EXTRA_GLYPHS: [char; 15] = ['◀', '▶', '›', '●', '▍', '◆', '·', '←', '→', '↑', '↓', '↖', '↗', '↘', '↙'];
pub const N_GLYPHS: usize = 96 + 96 + EXTRA_GLYPHS.len();

/// The glyph index of `c` (unknown characters draw as '?').
pub fn glyph_index(c: char) -> usize {
    let u = c as u32;
    match u {
        0x20..=0x7F => (u - 0x20) as usize,
        0xB7 => 96 + 96 + 6,
        0xA0..=0xFF => (u - 0xA0) as usize + 96,
        _ => EXTRA_GLYPHS.iter().position(|&e| e == c).map_or(('?' as usize) - 0x20, |i| 96 + 96 + i),
    }
}

/// The character of glyph index `i` (inverse of [`glyph_index`]).
pub fn glyph_char(i: usize) -> char {
    match i {
        0..=95 => char::from_u32(0x20 + i as u32).unwrap(),
        96..=191 => char::from_u32(0xA0 + (i - 96) as u32).unwrap(),
        _ => EXTRA_GLYPHS[i - 192],
    }
}

/// The atlas page: a CPU image plus the regions changed since the last upload.
///
/// The page is premultiplied alpha and carries its own mip chain, both built when an entry is written (at load), so
/// an upload is a plain copy. The buffers are reserved whole, once, and never reallocated; rows are only touched as the shelves reach them.
pub struct Atlas {
    /// Level 0, premultiplied RGBA8, `ATLAS` texels wide, `rows()` rows.
    pub px: Vec<u8>,
    /// Levels 1.. (each half the width of the one before), covering the same rows.
    pub mips: [Vec<u8>; (ATLAS_MIPS - 1) as usize],
    /// Changed regions (texels) the renderer uploads at the next present, then clears.
    pub dirty: Vec<AtlasRect>,
    /// Bumped on every change (the renderer re-uploads when it differs from what it last saw).
    pub generation: u64,
    shelf: (u16, u16, u16),
    pub white: AtlasRect,
    /// The UI fonts (Inter, JetBrains Mono; regular and bold) as SDF glyph tiles.
    pub faces: Box<[crate::ui_font::Face; crate::ui_font::N_FACES]>,
    /// Built-in bitmap kit pieces (only the hatch is a bitmap; the round pieces are analytic shapes).
    pub kit: [AtlasRect; KIT_PIECES.len()],
}

impl Default for Atlas {
    fn default() -> Self {
        Atlas::new()
    }
}

fn smooth(a: f32) -> u8 {
    (a.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}

/// Signed distance from (px, py) to a rounded box of half size (hx, hy) and radius r centred at 0.
pub fn sd_round_box(px: f32, py: f32, hx: f32, hy: f32, r: f32) -> f32 {
    let r = r.min(hx).min(hy);
    let qx = px.abs() - hx + r;
    let qy = py.abs() - hy + r;
    let ox = qx.max(0.0);
    let oy = qy.max(0.0);
    (ox * ox + oy * oy).sqrt() + qx.max(qy).min(0.0) - r
}

impl Atlas {
    /// The page with the host's own content: a white block, the glyphs and the hatch.
    pub fn new() -> Atlas {
        let mut a = Atlas {
            // The whole page is reserved here, once (address space; the rows are only touched as the shelves reach
            // them). The buffers must never be reallocated later: entries are written while the game's own allocator
            // scopes may be active (mod init, host pictures during a match), and a block allocated there does not
            // outlive them.
            px: Vec::with_capacity((ATLAS * ATLAS * 4) as usize),
            mips: std::array::from_fn(|l| Vec::with_capacity(((ATLAS >> (l + 1)) * (ATLAS >> (l + 1)) * 4) as usize)),
            dirty: Vec::with_capacity(4096),
            generation: 1,
            shelf: (CELL, CELL, 0),
            white: AtlasRect::default(),
            faces: Box::new(crate::ui_font::empty_faces()),
            kit: [AtlasRect::default(); KIT_PIECES.len()],
        };
        // white: an opaque 16x16 block sampled at its centre, so no filter and no mip level ever reaches its edge
        let w = a.alloc(16, 16).unwrap();
        a.fill(w, |_, _| [255, 255, 255, 255]);
        a.white = AtlasRect { x: w.x + 7, y: w.y + 7, w: 2, h: 2 };
        for (id, &(name, w, h)) in KIT_PIECES.iter().enumerate().skip(1) {
            if name != "ui.hatch" {
                continue;
            }
            let r = a.alloc(w, h).unwrap();
            // two-tone 2 px diagonal stripes (the colour-blind stun hatch)
            a.fill(r, move |x, y| [255, 255, 255, smooth(if ((x + y) / 2) % 2 == 0 { 1.0 } else { 0.45 })]);
            a.kit[id] = r;
        }
        a.faces = Box::new(crate::ui_font::build_faces(&mut a));
        a
    }

    /// Rows of level 0 the CPU page holds so far.
    pub fn rows(&self) -> usize {
        self.px.len() / (ATLAS as usize * 4)
    }

    fn grow(&mut self, rows: usize) {
        let rows = rows.next_multiple_of(256).min(ATLAS as usize);
        if rows <= self.rows() {
            return;
        }
        // within the capacity reserved by `new`: no reallocation
        self.px.resize(rows * ATLAS as usize * 4, 0);
        for (l, m) in self.mips.iter_mut().enumerate() {
            let s = l + 1;
            m.resize((rows >> s) * (ATLAS as usize >> s) * 4, 0);
        }
        debug_assert!(self.px.capacity() == (ATLAS * ATLAS * 4) as usize);
    }

    /// Shelf allocation: the entry starts on a `CELL` boundary and is followed by at least a `CELL` of clear texels,
    /// so no mip level blends it with a neighbour. None when the page is full.
    pub fn alloc(&mut self, w: u16, h: u16) -> Option<AtlasRect> {
        let cell = |v: u16| v.next_multiple_of(CELL) as u32 + CELL as u32;
        let (cw, ch) = (cell(w), cell(h));
        let (mut x, mut y, mut sh) = (self.shelf.0 as u32, self.shelf.1 as u32, self.shelf.2 as u32);
        if x + cw > ATLAS {
            x = CELL as u32;
            y += sh;
            sh = 0;
        }
        if y + ch > ATLAS || cw > ATLAS {
            return None;
        }
        self.shelf = ((x + cw) as u16, y as u16, sh.max(ch) as u16);
        self.grow((y + ch) as usize);
        Some(AtlasRect { x: x as u16, y: y as u16, w, h })
    }

    /// Write `r` from straight-alpha texels (stored premultiplied).
    fn fill(&mut self, r: AtlasRect, mut f: impl FnMut(u16, u16) -> [u8; 4]) {
        for y in 0..r.h {
            for x in 0..r.w {
                let o = (((r.y + y) as usize) * ATLAS as usize + (r.x + x) as usize) * 4;
                let t = f(x, y);
                let m = |c: u8| ((c as u32 * t[3] as u32 + 127) / 255) as u8;
                self.px[o..o + 4].copy_from_slice(&[m(t[0]), m(t[1]), m(t[2]), t[3]]);
            }
        }
        self.mark(r);
    }

    /// Copy tight straight-alpha RGBA8 rows into `r` (a mod texture).
    pub fn put(&mut self, r: AtlasRect, rgba: &[u8]) {
        let w = r.w as usize;
        self.fill(r, |x, y| {
            let o = (y as usize * w + x as usize) * 4;
            [rgba[o], rgba[o + 1], rgba[o + 2], rgba[o + 3]]
        });
    }

    /// `r` was written directly into `px` (premultiplied): rebuild its mips and queue the upload.
    pub(crate) fn mark_pub(&mut self, r: AtlasRect) {
        self.mark(r);
    }

    /// Width of `s` in `st` (the host's `gd_ui_measure`).
    pub fn measure(&self, s: &str, st: crate::ui_font::TextStyle) -> f32 {
        crate::ui_font::fit(&self.faces, st, s, 0.0).1
    }

    /// The level-`l` texels that cover level-0 rect `r`: (x, y, w, h).
    pub fn mip_rect(r: AtlasRect, l: u32) -> (usize, usize, usize, usize) {
        let (x0, y0) = (r.x as usize >> l, r.y as usize >> l);
        let (x1, y1) = ((r.x as usize + r.w as usize + (1 << l) - 1) >> l, (r.y as usize + r.h as usize + (1 << l) - 1) >> l);
        (x0, y0, x1 - x0, y1 - y0)
    }

    fn mark(&mut self, r: AtlasRect) {
        // box-filter the premultiplied texels down the chain (the page around an entry is clear, so a partial block
        // averages with transparent, which is what a premultiplied edge wants)
        for l in 1..ATLAS_MIPS {
            let (x, y, w, h) = Atlas::mip_rect(r, l);
            let (sw, dw) = ((ATLAS >> (l - 1)) as usize, (ATLAS >> l) as usize);
            let (lo, hi) = self.mips.split_at_mut(l as usize - 1);
            let src: &[u8] = if l == 1 { &self.px } else { &lo[l as usize - 2] };
            let dst = &mut hi[0];
            for yy in y..y + h {
                for xx in x..x + w {
                    let s0 = (yy * 2 * sw + xx * 2) * 4;
                    let s1 = s0 + sw * 4;
                    let d = (yy * dw + xx) * 4;
                    if s1 + 8 > src.len() || d + 4 > dst.len() {
                        continue;
                    }
                    for k in 0..4 {
                        dst[d + k] = ((src[s0 + k] as u32 + src[s0 + 4 + k] as u32 + src[s1 + k] as u32 + src[s1 + 4 + k] as u32 + 2) / 4) as u8;
                    }
                }
            }
        }
        // never grow the list in place (see `new`): past its capacity, fold everything into one rect
        if self.dirty.len() == self.dirty.capacity() {
            let (x0, y0, x1, y1) = self.dirty.iter().chain([&r]).fold((u16::MAX, u16::MAX, 0u16, 0u16), |a, d| (a.0.min(d.x), a.1.min(d.y), a.2.max(d.x + d.w), a.3.max(d.y + d.h)));
            self.dirty.clear();
            self.dirty.push(AtlasRect { x: x0, y: y0, w: x1 - x0, h: y1 - y0 });
        } else {
            self.dirty.push(r);
        }
        self.generation += 1;
    }
}

/// One UI frame: the sorted instances to draw and whether a custom scene hides the GX frame.
#[derive(Default)]
pub struct UiFrame {
    pub quads: Vec<UiQuad>,
    /// A custom scene (layer SCENE) is on top: the EFB is not shown.
    pub hide_gx: bool,
}

/// The GPU half, created on the first present that carries UI quads.
pub struct UiGpu {
    pipes: Vec<(wgpu::TextureFormat, wgpu::RenderPipeline)>,
    layout: wgpu::PipelineLayout,
    module: wgpu::ShaderModule,
    atlas: wgpu::Texture,
    uniform: wgpu::Buffer,
    bind: wgpu::BindGroup,
    inst: wgpu::Buffer,
    inst_cap: u64,
    /// Instances uploaded for the next blit, the atlas generation last uploaded.
    pub count: u32,
    atlas_gen: u64,
    pub hide_gx: bool,
    /// UI draw calls issued (the budget: <= 2 per frame) and presents with UI, for the stats line.
    pub draw_calls: u64,
    pub frames: u64,
    /// The render clock the effects animate on (never the tick count).
    clock: std::time::Instant,
}

const UI_WGSL: &str = "
struct U { tgt: vec4<f32>, band: vec4<f32>, tilt: vec4<f32>, tilt2: vec4<f32> };
// tilt: the menu camera (yaw, pitch radians, focal length px, parallax depth px); tilt2.x: instances this present.
// All zero at rest: the transform is skipped and every pixel is the untilted one.
// tgt.z: render-clock seconds, tgt.w: effects intensity (0 off, 0.5 subtle, 1 full)
@group(0) @binding(0) var<uniform> u: U;
@group(0) @binding(1) var atlas: texture_2d<f32>;
@group(0) @binding(2) var lin: sampler;
struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) c: vec4<f32>,
    @location(2) clip: vec4<f32>,
    @location(3) @interpolate(flat) kind: vec2<u32>,
    @location(4) @interpolate(flat) uvc: vec4<f32>,
    @location(5) l: vec2<f32>,
    @location(6) @interpolate(flat) aa: vec4<f32>,
    @location(7) @interpolate(flat) fx: vec2<u32>,
    @location(8) pp: vec2<f32>,
    @location(9) sp: vec2<f32>,
    @location(10) @interpolate(flat) hs: vec2<f32>,
    @location(11) @interpolate(flat) shp: vec3<f32>,
    @location(12) @interpolate(flat) c2: vec4<f32>,
};
fn to_px(p: vec2<f32>, win: bool) -> vec2<f32> {
    if (win) { return p; }
    return u.band.xy + p * u.band.zw / vec2<f32>(640.0, 480.0);
}
@vertex fn vs_main(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32,
                   @location(0) rect_in: vec4<f32>, @location(1) uvr: vec4<f32>, @location(2) clip: vec4<f32>,
                   @location(3) rgba: u32, @location(4) shear: f32, @location(5) flags: u32,
                   @location(6) rp: vec3<f32>, @location(7) fx: vec2<u32>,
                   @location(8) shp_in: vec3<f32>, @location(9) rgba2: u32) -> VsOut {
    var cx = array<f32, 6>(0.0, 1.0, 1.0, 0.0, 1.0, 0.0);
    var cy = array<f32, 6>(0.0, 0.0, 1.0, 0.0, 1.0, 1.0);
    let win = (flags & 1u) != 0u;
    let kind = (flags >> 16u) & 15u;
    let tilted = u.tilt.x != 0.0 || u.tilt.y != 0.0;
    var s = 1.0;
    if (!win) { s = u.band.w / 480.0; }
    var rect = rect_in;
    var shp = shp_in * s;
    // Pixel policy (uiquality). An analytic shape with straight sides that is neither rotated nor tilted is snapped:
    // its origin and its size land on whole device pixels (the size is rounded on its own, so it never changes as
    // the shape moves) and its border is a whole number of pixels, so straight edges are crisp and every border of
    // one width is the same number of pixels everywhere. Circles, rotated and tilted shapes are not snapped: their
    // edges are analytic coverage. Flat and textured quads keep the rasterizer's rule (an edge lands on the nearest
    // pixel boundary), which is watertight between abutting quads.
    if (kind != 0u) {
        let round_shape = rect.z == rect.w && shp_in.x * 2.0 >= rect.z;
        if (rp.x == 0.0 && !tilted && !round_shape && kind != 2u) {
            // floor(v + 0.5): halves round up on every backend (round() is half-to-even)
            let a = floor(to_px(rect.xy, win) + vec2<f32>(0.5));
            let sz = max(floor(rect.zw * s + vec2<f32>(0.5)), vec2<f32>(1.0, 1.0));
            if (win) { rect = vec4<f32>(a, sz); } else { rect = vec4<f32>((a - u.band.xy) / s, sz / s); }
            if (shp.y > 0.0) { shp.y = max(floor(shp.y + 0.5), 1.0); }
        }
    }
    // edge AA (kitaa): a slanted edge (shear: the left and right sides; a rotation off the right angles: all four)
    // gets an analytic coverage ramp, so the quad is grown one target pixel outward on those sides. Axis-aligned
    // edges are left exact, so abutting pieces (bars) never show seams.
    // aa: left, top, right, bottom. A side shared with an abutting piece (Q_IN_*) gets no ramp and no growth: two
    // half-covered edges laid over each other leave a see-through seam (uiseams).
    var aa = vec4<f32>(0.0);
    if (shear != 0.0) { aa = vec4<f32>(1.0, 0.0, 1.0, 0.0); }
    let q = abs(sin(2.0 * rp.x));
    if (rp.x != 0.0 && q > 1.0e-3) { aa = vec4<f32>(1.0); }
    let inner = vec4<f32>(f32((flags >> 2u) & 1u), f32((flags >> 3u) & 1u), f32((flags >> 4u) & 1u), f32((flags >> 5u) & 1u));
    aa = aa * (vec4<f32>(1.0) - inner);
    // a shape's own distance field does its antialiasing: the quad only has to reach one pixel past the outline
    var grow = aa;
    let full = (flags & 64u) != 0u;
    if (kind != 0u || full) { grow = vec4<f32>(1.0); aa = vec4<f32>(0.0); }
    let sz = max(abs(rect.zw) * s, vec2<f32>(1.0e-3, 1.0e-3));
    let e0 = grow.xy / sz;
    let e1 = grow.zw / sz;
    let l = vec2<f32>(cx[vi], cy[vi]) * (1.0 + e0 + e1) - e0;
    var p = rect.xy + vec2<f32>(l.x * rect.z + shear * (1.0 - l.y) * rect.w, l.y * rect.w);
    if (rp.x != 0.0) {
        let d = p - rp.yz;
        let cs = cos(rp.x);
        let sn = sin(rp.x);
        p = rp.yz + vec2<f32>(d.x * cs - d.y * sn, d.x * sn + d.y * cs);
    }
    var px = to_px(p, win);
    var o: VsOut;
    o.pp = px;
    var os = 1.0;
    if (tilted) {
        // the menu camera (retail mn_8022BA1C's C-stick orbit): the batch as a plane seen from a turned eye, later
        // quads nearer (parallax: backdrop, stickers, their shadows, the cursor)
        let c = u.band.xy + u.band.zw * 0.5;
        // a backdrop (a quad covering the whole band) is overscanned so a full turn never shows the black past it
        let ext = select(rect.zw * u.band.zw / vec2<f32>(640.0, 480.0), rect.zw, win);
        if (ext.x >= u.band.z * 0.98 && ext.y >= u.band.w * 0.98) {
            px = c + (px - c) * 1.4;
            os = 1.4;
        }
        // the pieces of one shape (a sliced sprite, a tiled bar) share their first piece's depth (flags bits 8..15:
        // instances back to it), so their shared sides stay shared under the perspective (uiseams2)
        let gi = f32(ii) - f32((flags >> 8u) & 255u);
        let z = (1.0 - gi / max(u.tilt2.x, 1.0)) * u.tilt.w;
        let q = vec3<f32>(px.x - c.x, c.y - px.y, z);
        let sa = sin(u.tilt.x);
        let ca = cos(u.tilt.x);
        let sb = sin(u.tilt.y);
        let cb = cos(u.tilt.y);
        let x1 = q.x * ca + q.z * sa;
        let z1 = -q.x * sa + q.z * ca;
        let y2 = q.y * cb + z1 * sb;
        let z2 = -q.y * sb + z1 * cb;
        let k = u.tilt.z / (u.tilt.z + z2);
        px = vec2<f32>(c.x + x1 * k, c.y - y2 * k);
    }
    o.pos = vec4<f32>(px.x / u.tgt.x * 2.0 - 1.0, 1.0 - px.y / u.tgt.y * 2.0, 0.0, 1.0);
    o.uv = mix(uvr.xy, uvr.zw, l);
    // bilinear never reads past the sub-rect's own texels (sprite-sheet cells abut with no gutter)
    let h = 0.5 / ATLAS_SIZE;
    o.uvc = vec4<f32>(min(uvr.xy, uvr.zw) + h, max(uvr.xy, uvr.zw) - h);
    // a whole entry may be filtered into its gutter (half a cell of clear texels on every side)
    if (full) { o.uvc = vec4<f32>(o.uvc.xy - 8.0 * h, o.uvc.zw + 8.0 * h); }
    let c = unpack4x8unorm(rgba);
    o.c = vec4<f32>(c.w, c.z, c.y, c.x);
    let c2 = unpack4x8unorm(rgba2);
    o.c2 = vec4<f32>(c2.w, c2.z, c2.y, c2.x);
    o.clip = vec4<f32>(to_px(clip.xy, win), to_px(clip.zw, win));
    // an overscanned backdrop's clip grows with it (its untilted pp keeps the unscaled corners)
    if (os != 1.0) {
        let cc = u.band.xy + u.band.zw * 0.5;
        o.clip = vec4<f32>(cc + (o.clip.xy - cc) * os, cc + (o.clip.zw - cc) * os);
    }
    o.kind = vec2<u32>((flags >> 1u) & 1u, kind);
    o.l = l;
    o.aa = aa;
    o.fx = fx;
    // the shape's frame: device pixels from its centre (unrotated, untilted), half size, [radius, border, spread]
    o.sp = (l - vec2<f32>(0.5, 0.5)) * sz;
    o.hs = sz * 0.5;
    o.shp = shp;
    if (kind == 0u) { o.shp = shp_in; }
    return o;
}
fn edge(t: f32, lo: f32, hi: f32) -> f32 {
    let g = max(length(vec2<f32>(dpdx(t), dpdy(t))), 1.0e-6);
    let a = clamp(t / g + 0.5, 0.0, 1.0);
    let b = clamp((1.0 - t) / g + 0.5, 0.0, 1.0);
    return mix(1.0, a, lo) * mix(1.0, b, hi);
}
fn hue(h: f32) -> vec3<f32> {
    return clamp(abs(fract(vec3<f32>(h, h + 0.6667, h + 0.3333)) * 6.0 - 3.0) - 1.0, vec3<f32>(0.0), vec3<f32>(1.0));
}
fn hash2(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.547);
}
// effects 1..3 (ui.rs FX_*): in band units so a face shimmers as one piece
fn effect(v: VsOut, base: vec4<f32>) -> vec4<f32> {
    let k = u.tgt.w;
    let t = u.tgt.z;
    let tint = unpack4x8unorm(v.fx.y);
    let tc = vec3<f32>(tint.w, tint.z, tint.y);
    let b = (v.pos.xy - u.band.xy) / max(u.band.zw, vec2<f32>(1.0)) * vec2<f32>(640.0, 480.0);
    if (v.fx.x == 1u) {
        // foil: a bright diagonal band every 2.6 s plus a slow hue drift, on the sticker's own pixels
        let d = (b.x + b.y * 0.6) * 0.004 - t * 0.38;
        let band = pow(max(0.0, 1.0 - abs(fract(d) - 0.5) * 7.0), 2.0);
        // a tinted multiply (shows on the white faces) under a white sheen band
        let shim = hue(d * 0.7 + b.y * 0.002) * 0.6 + tc * 0.4;
        let rgb = base.rgb * mix(vec3<f32>(1.0), shim, 0.3 * k) + vec3<f32>(band * 0.35 * k);
        return vec4<f32>(min(rgb, vec3<f32>(1.0)), base.a);
    }
    if (v.fx.x == 2u) {
        // holo: rainbow bands across the face, a wider sweep and twinkling glints
        let d = (b.x - b.y * 0.8) * 0.006 + t * 0.22;
        let rb = hue(d);
        let sw = pow(max(0.0, 1.0 - abs(fract((b.x + b.y) * 0.003 - t * 0.5) - 0.5) * 5.0), 2.0);
        let cell = floor(b / 11.0);
        let h = hash2(cell);
        let q = abs(fract(b / 11.0) - 0.5) * 11.0;
        let star = max(1.0 - (q.x * q.y * 1.4 + max(q.x, q.y) * 0.5), 0.0);
        let tw = pow(max(0.0, sin(t * 3.0 + h * 40.0)), 8.0) * step(0.88, h) * star;
        let rgb = mix(base.rgb, base.rgb * 0.55 + rb * 0.6 + tc * 0.1, 0.42 * k) + vec3<f32>(sw * 0.3 + tw * 0.8) * k;
        return vec4<f32>(min(rgb, vec3<f32>(1.0)), base.a);
    }
    if (v.fx.x == 3u) {
        // glow: the coverage in the effect colour, breathing (1.6 s); steady at subtle
        let br = 0.78 + 0.22 * sin(t * 3.9) * step(0.75, k);
        return vec4<f32>(tc, base.a * tint.x * br * min(1.0, k * 1.6));
    }
    return base;
}
// a dashed border: coverage of the dash pattern at `p`, measured along the border's centre line (clockwise from the
// top-left corner's end). The period is fitted so a whole number of dashes goes round: dash on, 0.8 dash off.
fn dash(p: vec2<f32>, hs: vec2<f32>, r: f32, bw: f32, dl: f32, w: f32) -> f32 {
    let h2 = hs - vec2<f32>(bw * 0.5);
    let rc = max(r - bw * 0.5, 0.0);
    let cm = max(h2 - vec2<f32>(rc), vec2<f32>(0.0));
    let ax = 2.0 * cm.x;
    let ay = 2.0 * cm.y;
    let arc = 1.5707963 * rc;
    var t = 0.0;
    let q = abs(p) - cm;
    if (q.x <= 0.0) {
        // the top and bottom sides
        if (p.y < 0.0) { t = p.x + cm.x; } else { t = ax + 2.0 * arc + ay + cm.x - p.x; }
    } else if (q.y <= 0.0) {
        // the right and left sides
        if (p.x > 0.0) { t = ax + arc + p.y + cm.y; } else { t = 2.0 * ax + 3.0 * arc + ay + cm.y - p.y; }
    } else if (p.x > 0.0) {
        // the right corners: the angle from the side the arc starts on
        if (p.y < 0.0) { t = ax + atan2(q.x, q.y) * rc; } else { t = ax + arc + ay + atan2(q.y, q.x) * rc; }
    } else {
        if (p.y > 0.0) { t = 2.0 * ax + 2.0 * arc + ay + atan2(q.x, q.y) * rc; } else { t = 2.0 * ax + 3.0 * arc + 2.0 * ay + atan2(q.y, q.x) * rc; }
    }
    let per = 2.0 * (ax + ay) + 4.0 * arc;
    let n = max(round(per / (dl * 1.8)), 1.0);
    let period = per / n;
    let on = period / 1.8;
    let u = fract(t / period) * period;
    return clamp(min(u, on - u) / w + 0.5, 0.0, 1.0);
}
// an analytic shape (ui.rs SHAPE_*): premultiplied colour from the signed distance in device pixels
fn shape(v: VsOut) -> vec4<f32> {
    let kind = v.kind.y;
    var hs = v.hs;
    if (kind == 3u) { hs = max(hs - vec2<f32>(v.shp.z), vec2<f32>(0.5)); }
    var d = 0.0;
    if (kind == 2u) {
        d = (abs(v.sp.x) / hs.x + abs(v.sp.y) / hs.y - 1.0) * min(hs.x, hs.y) * 0.70710678;
    } else {
        let r = min(v.shp.x, min(hs.x, hs.y));
        let q = abs(v.sp) - hs + vec2<f32>(r);
        d = length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
    }
    // one pixel of the distance field on screen: 1 at rest, the local stretch under the tilt's perspective
    let m = 0.5 * (length(dpdx(v.sp)) + length(dpdy(v.sp)));
    let w = clamp(length(vec2<f32>(dpdx(d), dpdy(d))), 0.7 * m, 1.5 * m);
    let fill = vec4<f32>(v.c.rgb * v.c.a, v.c.a);
    if (kind == 3u) {
        var a = 1.0;
        if (d > 0.0) { let t = d / max(v.shp.z * 0.5, 1.0e-3); a = exp(-t * t) * 0.9; }
        return fill * a;
    }
    let co = clamp(0.5 - d / w, 0.0, 1.0);
    if (v.shp.y <= 0.0) { return fill * co; }
    let ci = clamp(0.5 - (d + v.shp.y) / w, 0.0, 1.0);
    var line = vec4<f32>(v.c2.rgb * v.c2.a, v.c2.a);
    if (v.shp.z > 0.0) { line = line * dash(v.sp, hs, min(v.shp.x, min(hs.x, hs.y)), v.shp.y, v.shp.z, w); }
    return fill * ci + line * (co - ci);
}
@fragment fn fs_main(v: VsOut) -> @location(0) vec4<f32> {
    // The atlas is premultiplied and mipmapped; art is baked above its drawn size and minified through the chain
    // (trilinear), so small sprites and small text neither shimmer nor smear. The level is picked half a level
    // sharp: art drawn within 1.4x of its texel size stays on level 0.
    let du = dpdx(v.uv) * 0.70710678;
    let dv = dpdy(v.uv) * 0.70710678;
    let n = textureSampleGrad(atlas, lin, clamp(v.uv, v.uvc.xy, v.uvc.zw), du, dv);
    let sh = shape(v);
    // A glyph's coverage: the distance field's spread is SDF_RANGE texels each side of the outline, so one screen
    // pixel is a known step of the field: (texels per pixel) / (2 * SDF_RANGE). The edge is one pixel wide from
    // that. (It used to be normalised by fwidth of the sampled value, which is zero on the ridge of a thin stem:
    // a stem that fell between two pixel centres vanished instead of showing as two half-covered pixels, so small
    // text lost stems depending on where it sat on the pixel grid.)
    let tpp = max(max(length(dpdx(v.uv)), length(dpdy(v.uv))) * ATLAS_SIZE, 1.0e-4);
    let pxr = max(2.0 * SDF_RANGE / tpp, 1.0);
    let ea = edge(v.l.x, v.aa.x, v.aa.z) * edge(v.l.y, v.aa.y, v.aa.w);
    // clipping in the untilted plane (at rest: the fragment position, as before)
    let cp = select(v.pos.xy, v.pp, u.tilt.x != 0.0 || u.tilt.y != 0.0);
    if (cp.x < v.clip.x || cp.y < v.clip.y || cp.x >= v.clip.z || cp.y >= v.clip.w) { discard; }
    let c = vec4<f32>(v.c.rgb * v.c.a, v.c.a) * ea;
    if (v.kind.x != 0u) {
        // a glyph: coverage of the distance field, one pixel wide; shp.y widens it (the outline pass)
        let d = n.a + v.shp.y;
        return c * clamp((d - 0.5) * pxr + 0.5, 0.0, 1.0);
    }
    var b = n * c;
    if (v.kind.y != 0u) { b = sh; }
    if (v.fx.x != 0u && u.tgt.w > 0.0) {
        // effects work on straight alpha; the atlas and the blend are premultiplied (kitaa)
        var un = vec4<f32>(0.0);
        if (b.a > 0.0) { un = vec4<f32>(b.rgb / b.a, b.a); }
        let e = effect(v, un);
        return vec4<f32>(e.rgb * e.a, e.a);
    }
    return b;
}
";

impl UiGpu {
    pub fn new(device: &wgpu::Device) -> UiGpu {
        let src = UI_WGSL.replace("ATLAS_SIZE", &format!("{ATLAS}.0")).replace("SDF_RANGE", &format!("{:.1}", crate::ui_font::SDF_RANGE));
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("gd_ui"), source: wgpu::ShaderSource::Wgsl(src.into()) });
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("gd_ui"),
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::VERTEX_FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("gd_ui"), bind_group_layouts: &[&bgl], push_constant_ranges: &[] });
        let atlas = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("gd_ui atlas"),
            size: wgpu::Extent3d { width: ATLAS, height: ATLAS, depth_or_array_layers: 1 },
            mip_level_count: ATLAS_MIPS,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor { label: Some("gd_ui"), size: 64, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let linear = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("gd_ui"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_max_clamp: (ATLAS_MIPS - 1) as f32,
            ..Default::default()
        });
        let view = atlas.create_view(&Default::default());
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gd_ui"),
            layout: &bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: uniform.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&linear) },
            ],
        });
        let inst_cap = 4096 * std::mem::size_of::<UiQuad>() as u64;
        let inst = device.create_buffer(&wgpu::BufferDescriptor { label: Some("gd_ui instances"), size: inst_cap, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        UiGpu { pipes: Vec::new(), layout, module, atlas, uniform, bind, inst, inst_cap, count: 0, atlas_gen: 0, hide_gx: false, draw_calls: 0, frames: 0, clock: std::time::Instant::now() }
    }

    fn pipeline(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat) -> usize {
        if let Some(i) = self.pipes.iter().position(|p| p.0 == format) {
            return i;
        }
        let attrs = [
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 0, shader_location: 0 },
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 16, shader_location: 1 },
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 32, shader_location: 2 },
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Uint32, offset: 48, shader_location: 3 },
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32, offset: 52, shader_location: 4 },
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Uint32, offset: 56, shader_location: 5 },
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 60, shader_location: 6 },
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Uint32x2, offset: 72, shader_location: 7 },
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 80, shader_location: 8 },
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Uint32, offset: 92, shader_location: 9 },
        ];
        let p = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("gd_ui"),
            layout: Some(&self.layout),
            vertex: wgpu::VertexState {
                module: &self.module,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout { array_stride: std::mem::size_of::<UiQuad>() as u64, step_mode: wgpu::VertexStepMode::Instance, attributes: &attrs }],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &self.module,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format, blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING), write_mask: wgpu::ColorWrites::COLOR })],
            }),
            multiview: None,
            cache: None,
        });
        self.pipes.push((format, p));
        self.pipes.len() - 1
    }

    /// Upload this present's instances and the atlas regions changed since the last call (every mip level of each:
    /// plain copies out of the CPU page, which is already premultiplied and filtered).
    pub fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, f: &UiFrame, atlas: &mut Atlas) {
        let g = self;
        if g.atlas_gen != atlas.generation {
            // many small regions (the first upload: every glyph) go up as the rows they span
            let all;
            let rects: &[AtlasRect] = if atlas.dirty.len() > 48 {
                let (y0, y1) = atlas.dirty.iter().fold((u16::MAX, 0u16), |(a, b), r| (a.min(r.y), b.max(r.y + r.h)));
                all = [AtlasRect { x: 0, y: y0, w: ATLAS as u16, h: y1 - y0 }];
                &all
            } else {
                &atlas.dirty
            };
            for r in rects {
                for l in 0..ATLAS_MIPS {
                    let (x, y, w, h) = Atlas::mip_rect(*r, l);
                    let lw = (ATLAS >> l) as usize;
                    let page: &[u8] = if l == 0 { &atlas.px } else { &atlas.mips[l as usize - 1] };
                    let rows = page.len() / (lw * 4);
                    let h = h.min(rows.saturating_sub(y));
                    if w == 0 || h == 0 {
                        continue;
                    }
                    let o = (y * lw + x) * 4;
                    queue.write_texture(
                        wgpu::TexelCopyTextureInfo { texture: &g.atlas, mip_level: l, origin: wgpu::Origin3d { x: x as u32, y: y as u32, z: 0 }, aspect: wgpu::TextureAspect::All },
                        &page[o..o + ((h - 1) * lw + w) * 4],
                        wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(lw as u32 * 4), rows_per_image: Some(h as u32) },
                        wgpu::Extent3d { width: w as u32, height: h as u32, depth_or_array_layers: 1 },
                    );
                }
            }
            atlas.dirty.clear();
            g.atlas_gen = atlas.generation;
        }
        let bytes = as_bytes(&f.quads);
        if bytes.len() as u64 > g.inst_cap {
            g.inst_cap = (bytes.len() as u64).next_power_of_two();
            g.inst = device.create_buffer(&wgpu::BufferDescriptor { label: Some("gd_ui instances"), size: g.inst_cap, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
            if cfg!(debug_assertions) {
                eprintln!("gd_ui: instance buffer grown to {} quads", g.inst_cap / std::mem::size_of::<UiQuad>() as u64);
            }
        }
        if !bytes.is_empty() {
            queue.write_buffer(&g.inst, 0, bytes);
        }
        g.count = f.quads.len() as u32;
        g.hide_gx = f.hide_gx;
    }

    /// Record the UI batch into `pass` (one instanced draw). `band`: the 4:3 band rect in target pixels; `tilt`: the
    /// menu camera (yaw, pitch radians); `fx`: effects intensity.
    pub fn record(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, pass: &mut wgpu::RenderPass<'_>, format: wgpu::TextureFormat, tw: f32, th: f32, band: [f32; 4], tilt: [f32; 2], fx: f32) {
        let g = self;
        if g.count == 0 {
            return;
        }
        let pi = g.pipeline(device, format);
        // the effects' clock wraps every hour (f32 precision); intensity 0 turns every quad effect off
        let secs = (g.clock.elapsed().as_secs_f64() % 3600.0) as f32;
        // the menu tilt (Renderer::ui_tilt): focal length ~ retail's menu camera (a 30-degree turn reads as a turn,
        // not a smear), parallax depth 5% of the band height
        let [ya, pa] = tilt;
        let tilt = if ya != 0.0 || pa != 0.0 { [ya, pa, band[3] * 1.9, band[3] * 0.05] } else { [0.0; 4] };
        let u: [f32; 16] = [tw, th, secs, fx, band[0], band[1], band[2], band[3], tilt[0], tilt[1], tilt[2], tilt[3], g.count as f32, 0.0, 0.0, 0.0];
        let mut ub = [0u8; 64];
        for (i, v) in u.iter().enumerate() {
            ub[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
        }
        queue.write_buffer(&g.uniform, 0, &ub);
        g.draw_calls += 1;
        g.frames += 1;
        pass.set_viewport(0.0, 0.0, tw, th, 0.0, 1.0);
        pass.set_pipeline(&g.pipes[pi].1);
        pass.set_bind_group(0, &g.bind, &[]);
        pass.set_vertex_buffer(0, g.inst.slice(..g.count as u64 * std::mem::size_of::<UiQuad>() as u64));
        pass.draw(0..6, 0..g.count);
    }
}

fn as_bytes(q: &[UiQuad]) -> &[u8] {
    // SAFETY: UiQuad is repr(C), 96 bytes of plain f32/u32 with no padding holes.
    unsafe { std::slice::from_raw_parts(q.as_ptr() as *const u8, std::mem::size_of_val(q)) }
}

/// The 4:3 band inside blit rect `vp` (x, y, w, h): the largest 4:3 rect, centred. Its pixels are square at every
/// window shape, a window narrower than 4:3 included (uiquality: the band used to take the full height there, which
/// squeezed every shape sideways).
pub fn band_in(vp: [f32; 4]) -> [f32; 4] {
    let bw = (vp[3] * 4.0 / 3.0).min(vp[2]);
    let bh = bw * 0.75;
    [vp[0] + (vp[2] - bw) * 0.5, vp[1] + (vp[3] - bh) * 0.5, bw, bh]
}

impl Renderer {
    /// Hand this present's UI to the renderer: uploads the used instance range and the atlas regions changed since
    /// the last call. `None` (or an empty frame) draws no UI; with no UI mod loaded this is never called and nothing
    /// is created.
    pub fn set_ui(&mut self, frame: Option<&UiFrame>, atlas: &mut Atlas) {
        let Some(f) = frame.filter(|f| !f.quads.is_empty() || f.hide_gx) else {
            if let Some(g) = self.ui.as_mut() {
                g.count = 0;
                g.hide_gx = false;
            }
            return;
        };
        let g = self.ui.get_or_insert_with(|| UiGpu::new(&self.device));
        g.upload(&self.device, &self.queue, f, atlas);
    }

    /// Whether a custom scene hides the EFB this present.
    pub(crate) fn ui_hides_gx(&self) -> bool {
        self.ui.as_ref().is_some_and(|g| g.hide_gx)
    }

    /// Record the UI batch into the blit pass (one instanced draw). `band`: the 4:3 band rect in target pixels.
    pub(crate) fn ui_record(&mut self, pass: &mut wgpu::RenderPass<'_>, format: wgpu::TextureFormat, tw: f32, th: f32, band: [f32; 4]) {
        let (tilt, fx) = (self.ui_tilt, self.fx_intensity);
        if let Some(g) = self.ui.as_mut() {
            g.record(&self.device, &self.queue, pass, format, tw, th, band, tilt, fx);
        }
    }

    /// (UI draw calls, presents with UI) so far.
    pub fn ui_stats(&self) -> (u64, u64) {
        self.ui.as_ref().map_or((0, 0), |g| (g.draw_calls, g.frames))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quad_is_96_bytes() {
        assert_eq!(std::mem::size_of::<UiQuad>(), 96);
    }
    #[test]
    fn atlas_builtins_fit() {
        let a = Atlas::new();
        assert!(a.white.w > 0);
        assert_eq!(glyph_index('A'), 0x21);
        assert_eq!(glyph_index('▶'), 193);
        assert!(a.kit[5].w == 9);
        // the white block is opaque white at every mip level under its sample point
        let o = ((a.white.y as usize * ATLAS as usize) + a.white.x as usize) * 4;
        assert_eq!(&a.px[o..o + 4], &[255, 255, 255, 255]);
        for l in 1..ATLAS_MIPS {
            let (x, y, _, _) = Atlas::mip_rect(a.white, l);
            let o = (y * (ATLAS >> l) as usize + x) * 4;
            assert_eq!(&a.mips[l as usize - 1][o..o + 4], &[255, 255, 255, 255], "white at mip {l}");
        }
    }
    #[test]
    fn entries_never_share_a_coarse_texel() {
        let mut a = Atlas::new();
        let r1 = a.alloc(13, 21).unwrap();
        let r2 = a.alloc(30, 9).unwrap();
        for r in [r1, r2] {
            assert_eq!(r.x % CELL, 0);
            assert_eq!(r.y % CELL, 0);
        }
        assert!(r2.x >= r1.x + r1.w.next_multiple_of(CELL) + CELL || r2.y >= r1.y + r1.h.next_multiple_of(CELL) + CELL);
    }
    #[test]
    fn band_is_square_pixels_at_every_window_shape() {
        for vp in [[0.0, 0.0, 1920.0, 1080.0], [0.0, 0.0, 1280.0, 1024.0], [0.0, 0.0, 1080.0, 1920.0], [10.0, 5.0, 960.0, 720.0]] {
            let b = band_in(vp);
            assert!((b[2] / 640.0 - b[3] / 480.0).abs() < 1.0e-4, "{vp:?} -> {b:?}");
            assert!(b[2] <= vp[2] + 1.0e-3 && b[3] <= vp[3] + 1.0e-3);
        }
    }
}
