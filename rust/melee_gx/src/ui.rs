//! The UI layer's batched quad + atlas renderer (gd-ui-v1, docs/rust-port/notes/wasm-ui-lab-api.md section 0).
//!
//! Every UI element (flat, textured and sheared quads, 9-slice pieces, glyphs) is one [`UiQuad`] instance of a single
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
pub const ATLAS: u32 = 1024;

/// Instance flag: the quad (and its clip rect) is in window pixels, not band units.
pub const Q_WINDOW: u32 = 1;

/// One UI instance: 64 bytes, uploaded as-is (`#[repr(C)]`, little-endian like the GPU).
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
    pub _pad: u32,
}

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

/// Host 9-slice pieces (lab-ui-spec.md 7.3): rounded fills and borders per theme radius, and the pre-blurred glow.
/// Ids match `GD_UI_SLICE_*` in docs/specs/gd-ui-lab-v1.h.
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
const EXTRA_GLYPHS: [char; 7] = ['◀', '▶', '›', '●', '▍', '◆', '·'];
const N_GLYPHS: usize = 96 + 96 + EXTRA_GLYPHS.len();

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

/// Glyph bitmaps (8 rows, bit 0 = leftmost pixel).
fn glyph_bits(i: usize) -> [u8; 8] {
    if i < 96 {
        return font8x8::legacy::BASIC_LEGACY[i + 0x20];
    }
    if i < 192 {
        return font8x8::legacy::LATIN_LEGACY[i - 96];
    }
    match EXTRA_GLYPHS[i - 192] {
        '◀' => [0x20, 0x30, 0x38, 0x3C, 0x38, 0x30, 0x20, 0x00],
        '▶' => [0x04, 0x0C, 0x1C, 0x3C, 0x1C, 0x0C, 0x04, 0x00],
        '›' => [0x00, 0x04, 0x08, 0x10, 0x08, 0x04, 0x00, 0x00],
        '●' => [0x00, 0x1C, 0x3E, 0x3E, 0x3E, 0x1C, 0x00, 0x00],
        '▍' => [0x0E, 0x0E, 0x0E, 0x0E, 0x0E, 0x0E, 0x0E, 0x0E],
        '◆' => [0x08, 0x1C, 0x3E, 0x7F, 0x3E, 0x1C, 0x08, 0x00],
        _ => [0x00, 0x00, 0x00, 0x18, 0x18, 0x00, 0x00, 0x00],
    }
}

/// The atlas page: a CPU image plus the regions changed since the last upload.
pub struct Atlas {
    pub px: Vec<u8>,
    /// Changed regions (texels) the renderer uploads at the next present, then clears.
    pub dirty: Vec<AtlasRect>,
    /// Bumped on every change (the renderer re-uploads when it differs from what it last saw).
    pub generation: u64,
    shelf: (u16, u16, u16),
    pub white: AtlasRect,
    pub glyphs: [AtlasRect; N_GLYPHS],
    /// Per slice id: the tile and its corner size in texels.
    pub slices: [(AtlasRect, f32); SLICES.len()],
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
fn sd_round_box(px: f32, py: f32, hx: f32, hy: f32, r: f32) -> f32 {
    let qx = px.abs() - hx + r;
    let qy = py.abs() - hy + r;
    let ox = qx.max(0.0);
    let oy = qy.max(0.0);
    (ox * ox + oy * oy).sqrt() + qx.max(qy).min(0.0) - r
}

impl Atlas {
    /// The page with the host's own content: a white texel, the glyphs, the 9-slice tiles and the kit pieces.
    pub fn new() -> Atlas {
        let mut a = Atlas {
            px: vec![0u8; (ATLAS * ATLAS * 4) as usize],
            dirty: Vec::with_capacity(64),
            generation: 1,
            shelf: (0, 0, 0),
            white: AtlasRect::default(),
            glyphs: [AtlasRect::default(); N_GLYPHS],
            slices: [(AtlasRect::default(), 0.0); SLICES.len()],
            kit: [AtlasRect::default(); KIT_PIECES.len()],
        };
        // white: 4x4, sampled at its centre (nearest) so bleeding never matters
        let w = a.alloc(4, 4).unwrap();
        a.fill(w, |_, _| [255, 255, 255, 255]);
        a.white = AtlasRect { x: w.x + 1, y: w.y + 1, w: 2, h: 2 };
        for i in 0..N_GLYPHS {
            let r = a.alloc(8, 8).unwrap();
            let bits = glyph_bits(i);
            a.fill(r, |x, y| if bits[y as usize] >> x & 1 != 0 { [255, 255, 255, 255] } else { [255, 255, 255, 0] });
            a.glyphs[i] = r;
        }
        for (id, s) in SLICES.iter().enumerate().skip(1) {
            let c = (s.radius + s.glow).ceil().max(1.0);
            let n = (2.0 * c + 1.0) as u16;
            let r = a.alloc(n, n).unwrap();
            let half = c + 0.5; // the shape spans the tile minus the glow
            let (hx, rad) = (half - s.glow, s.radius);
            let s2 = *s;
            a.fill(r, move |x, y| {
                let (px, py) = (x as f32 + 0.5 - half, y as f32 + 0.5 - half);
                let d = sd_round_box(px, py, hx, hx, rad);
                let al = if s2.glow > 0.0 {
                    if d <= 0.0 { 1.0 } else { (-(d / (s2.glow * 0.5)).powi(2)).exp() * 0.9 }
                } else if s2.border > 0.0 {
                    (0.5 - d).clamp(0.0, 1.0) * (0.5 + d + s2.border).clamp(0.0, 1.0)
                } else {
                    0.5 - d
                };
                [255, 255, 255, smooth(al)]
            });
            a.slices[id] = (r, c);
        }
        for (id, &(name, w, h)) in KIT_PIECES.iter().enumerate().skip(1) {
            let r = a.alloc(w, h).unwrap();
            let (cx, cy) = (w as f32 * 0.5, h as f32 * 0.5);
            let name = name.to_string();
            a.fill(r, move |x, y| {
                let (px, py) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
                let d = (px * px + py * py).sqrt();
                let al = match name.as_str() {
                    "ui.dot" | "ui.knob" => cx - d + 0.5,
                    "ui.ring" => (0.5 - (d - (cx - 1.0)).abs() + 1.0).clamp(0.0, 1.0),
                    "ui.diamond" => cx - (px.abs() + py.abs()) + 0.5,
                    // two-tone 2 px diagonal stripes (the colour-blind stun hatch)
                    _ => if ((x + y) / 2) % 2 == 0 { 1.0 } else { 0.45 },
                };
                [255, 255, 255, smooth(al)]
            });
            a.kit[id] = r;
        }
        a
    }

    /// Shelf allocation with a 1-texel gutter. None when the page is full.
    pub fn alloc(&mut self, w: u16, h: u16) -> Option<AtlasRect> {
        let (mut x, mut y, mut sh) = self.shelf;
        if x as u32 + w as u32 + 1 > ATLAS {
            x = 0;
            y += sh + 1;
            sh = 0;
        }
        if y as u32 + h as u32 + 1 > ATLAS || w as u32 > ATLAS {
            return None;
        }
        let r = AtlasRect { x, y, w, h };
        self.shelf = (x + w + 1, y, sh.max(h));
        Some(r)
    }

    fn fill(&mut self, r: AtlasRect, mut f: impl FnMut(u16, u16) -> [u8; 4]) {
        for y in 0..r.h {
            for x in 0..r.w {
                let o = (((r.y + y) as usize) * ATLAS as usize + (r.x + x) as usize) * 4;
                self.px[o..o + 4].copy_from_slice(&f(x, y));
            }
        }
        self.mark(r);
    }

    /// Copy tight RGBA8 rows into `r` (a mod texture).
    pub fn put(&mut self, r: AtlasRect, rgba: &[u8]) {
        let w = r.w as usize;
        self.fill(r, |x, y| {
            let o = (y as usize * w + x as usize) * 4;
            [rgba[o], rgba[o + 1], rgba[o + 2], rgba[o + 3]]
        });
    }

    fn mark(&mut self, r: AtlasRect) {
        self.dirty.push(r);
        self.generation += 1;
    }
}

/// Text metrics of the glyph font at a pixel size: (glyph scale, advance, line height).
pub fn text_metrics(size: f32) -> (f32, f32, f32) {
    let k = size / 10.0;
    (k, 7.0 * k, size)
}

/// Width of `s` at `size` (the host's `gd_ui_measure`).
pub fn measure(s: &str, size: f32) -> f32 {
    s.chars().count() as f32 * text_metrics(size).1
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
    bgl: wgpu::BindGroupLayout,
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
}

const UI_WGSL: &str = "
struct U { tgt: vec4<f32>, band: vec4<f32> };
@group(0) @binding(0) var<uniform> u: U;
@group(0) @binding(1) var atlas: texture_2d<f32>;
@group(0) @binding(2) var smp: sampler;
struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) c: vec4<f32>,
    @location(2) clip: vec4<f32>,
};
fn to_px(p: vec2<f32>, win: bool) -> vec2<f32> {
    if (win) { return p; }
    return u.band.xy + p * u.band.zw / vec2<f32>(640.0, 480.0);
}
@vertex fn vs_main(@builtin(vertex_index) vi: u32,
                   @location(0) rect: vec4<f32>, @location(1) uvr: vec4<f32>, @location(2) clip: vec4<f32>,
                   @location(3) rgba: u32, @location(4) shear: f32, @location(5) flags: u32) -> VsOut {
    var cx = array<f32, 6>(0.0, 1.0, 1.0, 0.0, 1.0, 0.0);
    var cy = array<f32, 6>(0.0, 0.0, 1.0, 0.0, 1.0, 1.0);
    let l = vec2<f32>(cx[vi], cy[vi]);
    let win = (flags & 1u) != 0u;
    let p = rect.xy + vec2<f32>(l.x * rect.z + shear * (1.0 - l.y) * rect.w, l.y * rect.w);
    let px = to_px(p, win);
    var o: VsOut;
    o.pos = vec4<f32>(px.x / u.tgt.x * 2.0 - 1.0, 1.0 - px.y / u.tgt.y * 2.0, 0.0, 1.0);
    o.uv = mix(uvr.xy, uvr.zw, l);
    let c = unpack4x8unorm(rgba);
    o.c = vec4<f32>(c.w, c.z, c.y, c.x);
    o.clip = vec4<f32>(to_px(clip.xy, win), to_px(clip.zw, win));
    return o;
}
@fragment fn fs_main(v: VsOut) -> @location(0) vec4<f32> {
    if (v.pos.x < v.clip.x || v.pos.y < v.clip.y || v.pos.x >= v.clip.z || v.pos.y >= v.clip.w) { discard; }
    return textureSample(atlas, smp, v.uv) * v.c;
}
";

impl UiGpu {
    fn new(device: &wgpu::Device) -> UiGpu {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("gd_ui"), source: wgpu::ShaderSource::Wgsl(UI_WGSL.into()) });
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("gd_ui"),
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::VERTEX, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("gd_ui"), bind_group_layouts: &[&bgl], push_constant_ranges: &[] });
        let atlas = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("gd_ui atlas"),
            size: wgpu::Extent3d { width: ATLAS, height: ATLAS, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor { label: Some("gd_ui"), size: 32, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor { label: Some("gd_ui"), mag_filter: wgpu::FilterMode::Nearest, min_filter: wgpu::FilterMode::Nearest, ..Default::default() });
        let view = atlas.create_view(&Default::default());
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gd_ui"),
            layout: &bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: uniform.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
        });
        let inst_cap = 4096 * std::mem::size_of::<UiQuad>() as u64;
        let inst = device.create_buffer(&wgpu::BufferDescriptor { label: Some("gd_ui instances"), size: inst_cap, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        UiGpu { pipes: Vec::new(), bgl, layout, module, atlas, uniform, bind, inst, inst_cap, count: 0, atlas_gen: 0, hide_gx: false, draw_calls: 0, frames: 0 }
    }

    fn pipeline(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat) -> usize {
        if let Some(i) = self.pipes.iter().position(|p| p.0 == format) {
            return i;
        }
        let _ = &self.bgl;
        let attrs = [
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 0, shader_location: 0 },
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 16, shader_location: 1 },
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 32, shader_location: 2 },
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Uint32, offset: 48, shader_location: 3 },
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32, offset: 52, shader_location: 4 },
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Uint32, offset: 56, shader_location: 5 },
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
                targets: &[Some(wgpu::ColorTargetState { format, blend: Some(wgpu::BlendState::ALPHA_BLENDING), write_mask: wgpu::ColorWrites::COLOR })],
            }),
            multiview: None,
            cache: None,
        });
        self.pipes.push((format, p));
        self.pipes.len() - 1
    }
}

fn as_bytes(q: &[UiQuad]) -> &[u8] {
    // SAFETY: UiQuad is repr(C), 64 bytes of plain f32/u32 with no padding holes.
    unsafe { std::slice::from_raw_parts(q.as_ptr() as *const u8, std::mem::size_of_val(q)) }
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
        if g.atlas_gen != atlas.generation {
            let full = g.atlas_gen == 0;
            let rects: &[AtlasRect] = if full { &[AtlasRect { x: 0, y: 0, w: ATLAS as u16, h: ATLAS as u16 }] } else { &atlas.dirty };
            for r in rects {
                self.queue.write_texture(
                    wgpu::TexelCopyTextureInfo { texture: &g.atlas, mip_level: 0, origin: wgpu::Origin3d { x: r.x as u32, y: r.y as u32, z: 0 }, aspect: wgpu::TextureAspect::All },
                    &atlas.px[((r.y as usize * ATLAS as usize) + r.x as usize) * 4..],
                    wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(ATLAS * 4), rows_per_image: Some(r.h as u32) },
                    wgpu::Extent3d { width: r.w as u32, height: r.h as u32, depth_or_array_layers: 1 },
                );
            }
            atlas.dirty.clear();
            g.atlas_gen = atlas.generation;
        }
        let bytes = as_bytes(&f.quads);
        if bytes.len() as u64 > g.inst_cap {
            g.inst_cap = (bytes.len() as u64).next_power_of_two();
            g.inst = self.device.create_buffer(&wgpu::BufferDescriptor { label: Some("gd_ui instances"), size: g.inst_cap, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
            if cfg!(debug_assertions) {
                eprintln!("gd_ui: instance buffer grown to {} quads", g.inst_cap / 64);
            }
        }
        if !bytes.is_empty() {
            self.queue.write_buffer(&g.inst, 0, bytes);
        }
        g.count = f.quads.len() as u32;
        g.hide_gx = f.hide_gx;
    }

    /// Whether a custom scene hides the EFB this present.
    pub(crate) fn ui_hides_gx(&self) -> bool {
        self.ui.as_ref().is_some_and(|g| g.hide_gx)
    }

    /// Record the UI batch into the blit pass (one instanced draw). `band`: the 4:3 band rect in target pixels.
    pub(crate) fn ui_record(&mut self, pass: &mut wgpu::RenderPass<'_>, format: wgpu::TextureFormat, tw: f32, th: f32, band: [f32; 4]) {
        let device = &self.device;
        let Some(g) = self.ui.as_mut() else { return };
        if g.count == 0 {
            return;
        }
        let pi = g.pipeline(device, format);
        let u: [f32; 8] = [tw, th, 0.0, 0.0, band[0], band[1], band[2], band[3]];
        let mut ub = [0u8; 32];
        for (i, v) in u.iter().enumerate() {
            ub[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
        }
        self.queue.write_buffer(&g.uniform, 0, &ub);
        g.draw_calls += 1;
        g.frames += 1;
        let g = self.ui.as_ref().unwrap();
        pass.set_viewport(0.0, 0.0, tw, th, 0.0, 1.0);
        pass.set_pipeline(&g.pipes[pi].1);
        pass.set_bind_group(0, &g.bind, &[]);
        pass.set_vertex_buffer(0, g.inst.slice(..g.count as u64 * 64));
        pass.draw(0..6, 0..g.count);
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
    fn quad_is_64_bytes() {
        assert_eq!(std::mem::size_of::<UiQuad>(), 64);
    }
    #[test]
    fn atlas_builtins_fit() {
        let a = Atlas::new();
        assert!(a.white.w > 0);
        assert_eq!(glyph_index('A'), 0x21);
        assert_eq!(glyph_index('▶'), 193);
        assert!(a.slices[3].1 >= 14.0);
        assert!(a.kit[2].w == 56);
        // the white texel is opaque white
        let o = ((a.white.y as usize * ATLAS as usize) + a.white.x as usize) * 4;
        assert_eq!(&a.px[o..o + 4], &[255, 255, 255, 255]);
    }
}
