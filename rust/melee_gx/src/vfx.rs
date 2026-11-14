//! Shader effect quads (docs/rust-port/notes/envoy-vfx.md): the Envoy effects as procedural fragment shaders.
//!
//! One instanced pipeline draws every effect of a present: a quad per instance, in the world (the stage plane, at the
//! instance's depth, depth-tested against the scene so fighters in front stay in front) or on the 640x480 canvas,
//! with a fixed built-in shader picked by `kind` and driven by per-instance parameters (age, seed, four numbers, a
//! two-colour ramp). Mods never ship WGSL. The quads go into the EFB after the GX frame's last pass, inside the
//! effects layer's world pass ([`crate::fx`]), so they write the emissive target the bloom reads; kinds flagged
//! [`F_DISTORT`] are drawn a second time into a half-resolution offset target the blit refracts the picture through
//! (shockwaves, heat haze: local, never the whole screen).
//!
//! Presentation only: the list is plain data the UI mod queues each present ([`begin`] / [`push`] / [`commit`]), an
//! effect's time is the age the mod gives it (no clock here), and nothing is read back.

use std::sync::Mutex;

/// Effect kinds (`VfxInst::kind`; `gd_vfx_batch`). The parameters `p[0..4]` per kind are listed in envoy-vfx.md.
pub const K_GLOW: u32 = 1;
pub const K_SHOCK: u32 = 2;
pub const K_BOLT: u32 = 3;
pub const K_CIRCLE: u32 = 4;
pub const K_FLAME: u32 = 5;
pub const K_EMBERS: u32 = 6;
pub const K_ICE: u32 = 7;
pub const K_TRAIL: u32 = 8;
pub const K_SPARKS: u32 = 9;
pub const K_STAR: u32 = 10;
pub const K_WISP: u32 = 11;
pub const K_SLASH: u32 = 12;
pub const K_CRACK: u32 = 13;
pub const K_BEAM: u32 = 14;
pub const K_ORB: u32 = 15;
pub const K_DOME: u32 = 16;
pub const K_MINE: u32 = 17;
pub const K_COIN: u32 = 18;
pub const K_HAZE: u32 = 19;
pub const K_AURA: u32 = 20;
pub const K_RING: u32 = 21;
pub const K_COUNT: u32 = 22;

/// The position is on the 640x480 canvas (x right, y down; the size in canvas units), not in the world.
pub const F_SCREEN: u32 = 1;
/// Drawn over the scene whatever its depth (world effects that must never be hidden; screen quads always are).
pub const F_NO_DEPTH: u32 = 2;
/// Also refracts the picture behind it (K_SHOCK, K_HAZE, K_DOME); skipped at intensity Off.
pub const F_DISTORT: u32 = 4;

/// A segment: `pos` (x, y) is its start, `size` its end point and `rot` its width; the host turns it into the quad
/// along it (bolts, trails, beams between two points: mods have no atan2).
pub const F_LINE: u32 = 8;

/// Effect quads a present may queue.
pub const MAX_VFX: usize = 768;

/// One effect quad: 64 bytes, uploaded as-is.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct VfxInst {
    /// Centre: world units, or canvas units with [`F_SCREEN`].
    pub pos: [f32; 3],
    /// Rotation in radians, counter-clockwise.
    pub rot: f32,
    /// Full width and height.
    pub size: [f32; 2],
    /// The effect's age in seconds (looping effects: any running time).
    pub t: f32,
    pub seed: f32,
    pub p: [f32; 4],
    /// 0xRRGGBBAA: the main colour (alpha: the whole effect's opacity) and the outer / second colour.
    pub col0: u32,
    pub col1: u32,
    pub kind: u32,
    pub flags: u32,
}

static BUILD: Mutex<Vec<VfxInst>> = Mutex::new(Vec::new());
static SHOW: Mutex<Vec<VfxInst>> = Mutex::new(Vec::new());

/// A present begins: nothing queued yet.
pub fn begin() {
    if let Ok(mut b) = BUILD.lock() {
        b.clear();
    }
}

/// Queue one effect quad. False: refused (unknown kind, a non-finite number, or the present's cap).
pub fn push(mut i: VfxInst) -> bool {
    let nums = [i.pos[0], i.pos[1], i.pos[2], i.rot, i.size[0], i.size[1], i.t, i.seed, i.p[0], i.p[1], i.p[2], i.p[3]];
    if !(1..K_COUNT).contains(&i.kind) || i.flags & !15 != 0 || !nums.iter().all(|v| v.is_finite() && v.abs() < 1.0e6) {
        return false;
    }
    let Ok(mut b) = BUILD.lock() else { return false };
    if b.len() >= MAX_VFX {
        return false;
    }
    if i.flags & F_LINE != 0 {
        let (dx, dy) = (i.size[0] - i.pos[0], i.size[1] - i.pos[1]);
        let len = (dx * dx + dy * dy).sqrt();
        i.pos = [i.pos[0] + dx * 0.5, i.pos[1] + dy * 0.5, i.pos[2]];
        i.size = [len, i.rot.abs()];
        // the canvas's y runs down
        i.rot = if i.flags & F_SCREEN != 0 { (-dy).atan2(dx) } else { dy.atan2(dx) };
        i.flags &= !F_LINE;
    }
    i.size = [i.size[0].clamp(0.0, 4000.0), i.size[1].clamp(0.0, 4000.0)];
    b.push(i);
    true
}

/// The present ended: its list is what is drawn until the next one.
pub fn commit() {
    let (Ok(mut b), Ok(mut s)) = (BUILD.lock(), SHOW.lock()) else { return };
    std::mem::swap(&mut *b, &mut *s);
    b.clear();
}

/// Effect quads in the list being shown.
pub fn shown() -> usize {
    SHOW.lock().map_or(0, |s| s.len())
}

/// Copy the shown list into `out` (cleared first).
pub fn copy_shown(out: &mut Vec<VfxInst>) {
    out.clear();
    if let Ok(s) = SHOW.lock() {
        out.extend_from_slice(&s);
    }
}

pub(crate) const INST_BYTES: u64 = 64;
/// The offset target's format (signed uv offsets, added up).
pub(crate) const DISTORT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg16Float;

/// The shader source: `MELEE_VFX_WGSL=<file>` replaces the built-in one (a development aid for tuning effects without
/// a rebuild; a file that does not compile is a GPU validation error, as any bad shader is).
pub fn wgsl() -> String {
    if let Some(p) = std::env::var_os("MELEE_VFX_WGSL") {
        if let Ok(s) = std::fs::read_to_string(&p) {
            return s;
        }
    }
    VFX_WGSL.to_string()
}

/// The GPU half: lives in the effects layer ([`crate::fx`]), which owns the globals and the passes.
pub(crate) struct VfxGpu {
    module: wgpu::ShaderModule,
    /// depth-tested, drawn over everything; for `samples`
    pipes: Vec<wgpu::RenderPipeline>,
    samples: u32,
    distort_pipe: wgpu::RenderPipeline,
    inst: wgpu::Buffer,
    scratch: Vec<u8>,
    /// Instance ranges uploaded for this present: depth-tested, over everything, distorting.
    pub n_depth: u32,
    pub n_over: u32,
    pub n_distort: u32,
    pub distort: Option<(wgpu::Texture, (u32, u32))>,
}

fn attrs() -> [wgpu::VertexAttribute; 6] {
    [
        wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 0, shader_location: 0 },
        wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 16, shader_location: 1 },
        wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 32, shader_location: 2 },
        wgpu::VertexAttribute { format: wgpu::VertexFormat::Uint32x2, offset: 48, shader_location: 3 },
        wgpu::VertexAttribute { format: wgpu::VertexFormat::Uint32x2, offset: 56, shader_location: 4 },
        wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32, offset: 0, shader_location: 5 },
    ]
}

impl VfxGpu {
    pub fn new(device: &wgpu::Device, layout: &wgpu::PipelineLayout) -> VfxGpu {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("vfx"), source: wgpu::ShaderSource::Wgsl(wgsl().into()) });
        let inst = device.create_buffer(&wgpu::BufferDescriptor { label: Some("vfx instances"), size: INST_BYTES * (MAX_VFX as u64) * 2, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let a = attrs();
        let add = wgpu::BlendState { color: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add }, alpha: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add } };
        let distort_pipe = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("vfx distort"),
            layout: Some(layout),
            vertex: wgpu::VertexState { module: &module, entry_point: Some("vs_main"), compilation_options: Default::default(), buffers: &[wgpu::VertexBufferLayout { array_stride: INST_BYTES, step_mode: wgpu::VertexStepMode::Instance, attributes: &a[..5] }] },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState { module: &module, entry_point: Some("fs_distort"), compilation_options: Default::default(), targets: &[Some(wgpu::ColorTargetState { format: DISTORT_FORMAT, blend: Some(add), write_mask: wgpu::ColorWrites::ALL })] }),
            multiview: None,
            cache: None,
        });
        VfxGpu { module, pipes: Vec::new(), samples: 0, distort_pipe, inst, scratch: Vec::with_capacity(INST_BYTES as usize * 256), n_depth: 0, n_over: 0, n_distort: 0, distort: None }
    }

    /// The colour pipelines for the world pass's sample count (created once per count: at arm, so never mid-fight).
    pub fn pipelines(&mut self, device: &wgpu::Device, layout: &wgpu::PipelineLayout, samples: u32) {
        if self.samples == samples && !self.pipes.is_empty() {
            return;
        }
        let a = attrs();
        let add = wgpu::BlendState { color: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add }, alpha: wgpu::BlendComponent::REPLACE };
        let mk = |depth: bool| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(if depth { "vfx" } else { "vfx over" }),
                layout: Some(layout),
                vertex: wgpu::VertexState { module: &self.module, entry_point: Some("vs_main"), compilation_options: Default::default(), buffers: &[wgpu::VertexBufferLayout { array_stride: INST_BYTES, step_mode: wgpu::VertexStepMode::Instance, attributes: &a[..5] }] },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: crate::render::DEPTH_FORMAT,
                    depth_write_enabled: false,
                    depth_compare: if !depth { wgpu::CompareFunction::Always } else if crate::state::USE_REVERSED_Z { wgpu::CompareFunction::GreaterEqual } else { wgpu::CompareFunction::LessEqual },
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: wgpu::MultisampleState { count: samples, ..Default::default() },
                fragment: Some(wgpu::FragmentState {
                    module: &self.module,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[
                        Some(wgpu::ColorTargetState { format: crate::render::COLOR_FORMAT, blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING), write_mask: wgpu::ColorWrites::COLOR }),
                        Some(wgpu::ColorTargetState { format: crate::render::COLOR_FORMAT, blend: Some(add), write_mask: wgpu::ColorWrites::COLOR }),
                    ],
                }),
                multiview: None,
                cache: None,
            })
        };
        self.pipes = vec![mk(true), mk(false)];
        self.samples = samples;
    }

    /// The half-resolution offset target for a `w` x `h` EFB.
    pub fn target(&mut self, device: &wgpu::Device, w: u32, h: u32) {
        let size = ((w / 2).max(1), (h / 2).max(1));
        if self.distort.as_ref().is_some_and(|d| d.1 == size) {
            return;
        }
        let t = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("vfx distort"),
            size: wgpu::Extent3d { width: size.0, height: size.1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DISTORT_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        self.distort = Some((t, size));
    }

    /// Upload this present's quads: the depth-tested ones, then the ones drawn over everything, then a copy of the
    /// distorting ones (none when `distort` is off).
    pub fn upload(&mut self, queue: &wgpu::Queue, list: &[VfxInst], distort: bool) {
        self.scratch.clear();
        let put = |b: &mut Vec<u8>, i: &VfxInst| {
            // SAFETY: VfxInst is repr(C), 64 bytes of plain f32 / u32 with no padding.
            b.extend_from_slice(unsafe { std::slice::from_raw_parts(i as *const VfxInst as *const u8, INST_BYTES as usize) });
        };
        let over = |i: &VfxInst| i.flags & (F_SCREEN | F_NO_DEPTH) != 0;
        let list = &list[..list.len().min(MAX_VFX)];
        self.n_depth = 0;
        self.n_over = 0;
        self.n_distort = 0;
        for i in list.iter().filter(|i| !over(i)) {
            put(&mut self.scratch, i);
            self.n_depth += 1;
        }
        for i in list.iter().filter(|i| over(i)) {
            put(&mut self.scratch, i);
            self.n_over += 1;
        }
        if distort {
            for i in list.iter().filter(|i| i.flags & F_DISTORT != 0) {
                put(&mut self.scratch, i);
                self.n_distort += 1;
            }
        }
        if !self.scratch.is_empty() {
            queue.write_buffer(&self.inst, 0, &self.scratch);
        }
    }

    pub fn any(&self) -> bool {
        self.n_depth + self.n_over > 0
    }

    /// Record the colour draws into the world pass (bind group 0, the effects layer's globals, is already set).
    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        if !self.any() || self.pipes.is_empty() {
            return;
        }
        pass.set_vertex_buffer(0, self.inst.slice(..));
        if self.n_depth > 0 {
            pass.set_pipeline(&self.pipes[0]);
            pass.draw(0..6, 0..self.n_depth);
        }
        if self.n_over > 0 {
            pass.set_pipeline(&self.pipes[1]);
            pass.draw(0..6, self.n_depth..self.n_depth + self.n_over);
        }
    }

    /// Record the offset draws (their own pass: the half-resolution offset target, cleared to zero).
    pub fn draw_distort(&self, pass: &mut wgpu::RenderPass<'_>) {
        if self.n_distort == 0 {
            return;
        }
        let first = self.n_depth + self.n_over;
        pass.set_pipeline(&self.distort_pipe);
        pass.set_vertex_buffer(0, self.inst.slice(..));
        pass.draw(0..6, first..first + self.n_distort);
    }
}

/// The effect shaders. Globals: the effects layer's (`fx.rs` WORLD_WGSL `G`) plus `vfx` (EFB aspect, EFB height in
/// pixels, 0, 0). Every kind returns straight intensity in its colours; `finish` turns it into a premultiplied pixel
/// with a white-hot core and a little occlusion so it stays readable on a bright stage.
pub const VFX_WGSL: &str = r#"
struct G { v0: vec4f, v1: vec4f, v2: vec4f, p0: vec4f, p1: vec4f, p2: vec4f, p3: vec4f, misc: vec4f, glass: vec4f, vfx: vec4f };
@group(0) @binding(0) var<uniform> g: G;
struct VO {
    @builtin(position) pos: vec4f,
    @location(0) uv: vec2f,
    @location(1) @interpolate(flat) a: vec4f,   // t, seed, aspect (w / h), kind
    @location(2) @interpolate(flat) p: vec4f,
    @location(3) @interpolate(flat) c0: vec4f,
    @location(4) @interpolate(flat) c1: vec4f,
};
struct FO { @location(0) c: vec4f, @location(1) e: vec4f };
const TAU: f32 = 6.2831853;
fn unpack(c: u32) -> vec4f {
    return vec4f(f32((c >> 24u) & 255u), f32((c >> 16u) & 255u), f32((c >> 8u) & 255u), f32(c & 255u)) / 255.0;
}
@vertex fn vs_main(@builtin(vertex_index) vi: u32, @location(0) pr: vec4f, @location(1) st: vec4f, @location(2) par: vec4f,
                   @location(3) col: vec2u, @location(4) kf: vec2u) -> VO {
    var cx = array<f32, 6>(-1.0, 1.0, 1.0, -1.0, 1.0, -1.0);
    var cy = array<f32, 6>(-1.0, -1.0, 1.0, -1.0, 1.0, 1.0);
    let l = vec2f(cx[vi], cy[vi]);
    let cs = cos(pr.w);
    let sn = sin(pr.w);
    let h = l * st.xy * 0.5;
    let d = vec2f(h.x * cs - h.y * sn, h.x * sn + h.y * cs);
    var o: VO;
    if ((kf.y & 1u) != 0u) {
        // the canvas: 640x480 units centred on the picture, y down (the 4:3 band inside a wider EFB)
        let c = vec2f(pr.x + d.x, pr.y - d.y);
        o.pos = vec4f((c.x / 640.0 - 0.5) * (4.0 / 3.0) / g.vfx.x * 2.0, 1.0 - c.y / 480.0 * 2.0, 0.5, 1.0);
    } else {
        let w = vec4f(pr.x + d.x, pr.y + d.y, pr.z, 1.0);
        let vp = vec4f(dot(g.v0, w), dot(g.v1, w), dot(g.v2, w), 1.0);
        o.pos = vec4f(dot(g.p0, vp), dot(g.p1, vp), dot(g.p2, vp), dot(g.p3, vp));
    }
    o.uv = l;
    o.a = vec4f(st.z, st.w, st.x / max(st.y, 1.0e-4), f32(kf.x));
    o.p = par;
    o.c0 = unpack(col.x);
    o.c1 = unpack(col.y);
    return o;
}
// ---- noise
fn h11(x: f32) -> f32 { return fract(sin(x * 127.1 + 0.317) * 43758.547); }
fn h21(p: vec2f) -> f32 { return fract(sin(dot(p, vec2f(127.1, 311.7))) * 43758.547); }
fn h22(p: vec2f) -> vec2f { return fract(sin(vec2f(dot(p, vec2f(127.1, 311.7)), dot(p, vec2f(269.5, 183.3)))) * 43758.547); }
fn n1(x: f32) -> f32 {
    let i = floor(x);
    let f = fract(x);
    return mix(h11(i), h11(i + 1.0), f * f * (3.0 - 2.0 * f));
}
// linear (sharp-cornered) 1D noise: lightning and cracks
fn n1l(x: f32) -> f32 {
    let i = floor(x);
    return mix(h11(i), h11(i + 1.0), fract(x));
}
fn vn(p: vec2f) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let s = f * f * (3.0 - 2.0 * f);
    return mix(mix(h21(i), h21(i + vec2f(1.0, 0.0)), s.x), mix(h21(i + vec2f(0.0, 1.0)), h21(i + vec2f(1.0, 1.0)), s.x), s.y);
}
fn fbm(p: vec2f) -> f32 {
    return vn(p) * 0.5 + vn(p * 2.03 + vec2f(17.1, 3.3)) * 0.3 + vn(p * 4.11 + vec2f(5.7, 29.9)) * 0.2;
}
fn rot2(p: vec2f, a: f32) -> vec2f {
    let c = cos(a);
    let s = sin(a);
    return vec2f(p.x * c - p.y * s, p.x * s + p.y * c);
}
fn seg(p: vec2f, a: vec2f, b: vec2f) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / max(dot(ba, ba), 1.0e-6), 0.0, 1.0);
    return length(pa - ba * h);
}
fn ease(x: f32) -> f32 {
    let k = 1.0 - clamp(x, 0.0, 1.0);
    return 1.0 - k * k * k;
}
// a glowing line: a crisp core of half-width w plus a soft halo
fn line(d: f32, w: f32) -> f32 {
    return smoothstep(w, w * 0.25, d) + 0.35 * exp(-d / (w * 3.5));
}
struct R { rgb: vec3f, a: f32 };   // straight light (may exceed 1) and how much it covers what is behind

// ---- 1 glow: a soft light. p0 falloff (default 4), p1 core size 0..1
fn k_glow(uv: vec2f, v: VO) -> R {
    let r = length(uv);
    let k = select(v.p.x, 4.0, v.p.x <= 0.0);
    let i = exp(-r * r * k) * smoothstep(1.0, 0.7, r);
    let core = exp(-r * r * k * 9.0) * v.p.y * 2.5;
    return R(v.c0.rgb * i + mix(v.c0.rgb, vec3f(1.0), 0.6) * core, 0.0);
}
// ---- 2 shock: a ring running out. p0 progress 0..1, p1 width (default 0.08), p2 electric crackle 0..1, p3 refraction
fn k_shock(uv: vec2f, v: VO) -> R {
    let r = length(uv);
    let a = atan2(uv.y, uv.x);
    let pr = clamp(v.p.x, 0.0, 1.0);
    let rad = 0.06 + 0.84 * ease(pr);
    let w = select(v.p.y, 0.08, v.p.y <= 0.0) * (1.0 - 0.55 * pr) + 0.012;
    let brk = fbm(vec2f(a * 2.5 + v.a.y * 9.0, r * 5.0 - v.a.x * 2.0));
    let x = (r - rad) / w;
    // a crisp leading edge, a soft body behind it, torn by the noise
    let lead = smoothstep(1.0, 0.0, abs(x * 2.6));
    let body = exp(-x * x) * (0.45 + 0.9 * brk);
    let wake = smoothstep(rad, rad - 0.5, r) * smoothstep(0.0, 0.12, r) * 0.16 * (1.0 - pr);
    var i = (lead * 1.6 + body) * smoothstep(0.0, 0.1, pr + 0.03);
    var arc = 0.0;
    if (v.p.z > 0.0) {
        // crackle: thin arcs riding the ring, re-drawn 30 times a second
        let fr = floor(v.a.x * 30.0);
        let sd = v.a.y * 13.7 + fr * 7.13;
        for (var j = 0; j < 2; j++) {
            let ra = rad + (n1l(a * (5.0 + f32(j) * 4.0) + sd + f32(j) * 31.0) - 0.5) * (0.16 + 0.08 * f32(j)) * v.p.z;
            let gate = smoothstep(0.35, 0.6, n1(a * 1.6 + sd * 0.37 + f32(j) * 11.0));
            arc += line(abs(r - ra), 0.008) * gate;
        }
    }
    let fade = pow(1.0 - pr, 0.7);
    let rgb = (v.c0.rgb * i + v.c1.rgb * (wake + body * 0.6) + vec3f(1.0) * arc * 1.4) * fade;
    return R(rgb, 0.0);
}
// ---- 3 bolt: lightning from the quad's -x end to its +x end, redrawn from the seed. p0 jaggedness (default 1),
// p1 branches 0..1, p2 redraws per second (default 24), p3 thickness (default 1)
fn bolt_off(x: f32, sd: f32) -> f32 {
    let env = smoothstep(0.0, 0.3, 1.0 - abs(x));
    return ((n1l(x * 1.7 + sd) - 0.5) * 0.9 + (n1l(x * 4.3 + sd * 1.7) - 0.5) * 0.75 + (n1l(x * 9.7 + sd * 2.3) - 0.5) * 0.42 + (n1l(x * 23.0 + sd * 3.1) - 0.5) * 0.16) * env;
}
fn k_bolt(uv: vec2f, v: VO) -> R {
    let rate = select(v.p.z, 24.0, v.p.z <= 0.0);
    let fr = floor(v.a.x * rate);
    let sd = v.a.y * 31.7 + fr * 7.13;
    let jag = select(v.p.x, 1.0, v.p.x <= 0.0);
    let th = 0.035 * select(v.p.w, 1.0, v.p.w <= 0.0);
    let x = uv.x;
    let y = uv.y;
    let amp = 0.62 * jag;
    let d = abs(y - amp * bolt_off(x, sd));
    // the flicker: every redraw has its own brightness, now and then a dark one
    let fl = 0.55 + 0.45 * h11(fr + v.a.y * 3.1);
    var core = smoothstep(th, th * 0.2, d);
    var glow = pow(th / (d + th), 1.7);
    var halo = exp(-d * 5.5) * 0.35;
    // a fainter second strand that leaves the main one and rejoins it
    let d2 = abs(y - amp * (bolt_off(x, sd) + (n1l(x * 3.1 + sd * 5.0) - 0.5) * 0.7 * smoothstep(0.0, 0.5, 1.0 - abs(x))));
    core += smoothstep(th * 0.5, th * 0.1, d2) * 0.6;
    glow += pow(th * 0.5 / (d2 + th * 0.5), 1.7) * 0.5;
    if (v.p.y > 0.0) {
        for (var j = 0; j < 4; j++) {
            let fj = f32(j);
            let x0 = h11(sd + fj * 3.7) * 1.3 - 0.75;
            let ln = (0.25 + 0.45 * h11(sd * 1.3 + fj * 9.1)) * v.p.y;
            let u = (x - x0) / ln;
            if (u > 0.0 && u < 1.0) {
                let dir = select(-1.0, 1.0, h11(sd * 0.7 + fj * 5.3) > 0.5);
                let yb = amp * bolt_off(x0, sd) + dir * u * ln * (0.7 + 0.8 * h11(sd + fj * 17.0)) + (n1l(x * 11.0 + sd + fj * 5.0) - 0.5) * 0.3 * u;
                let db = abs(y - yb);
                let tb = th * 0.7 * (1.0 - u * 0.7);
                core += smoothstep(tb, tb * 0.2, db) * (1.0 - u);
                glow += pow(tb / (db + tb), 1.7) * (1.0 - u) * 1.1;
            }
        }
    }
    // where it strikes: a hot point at the +x end
    let hv = vec2f((uv.x - 0.96) * v.a.z, uv.y);
    let hit = exp(-dot(hv, hv) * 9.0);
    let ends = smoothstep(1.0, 0.92, abs(x));
    let rgb = (vec3f(1.0) * core * 2.2 + v.c0.rgb * glow * 1.5 + v.c1.rgb * (halo + hit * 0.8)) * fl * ends * smoothstep(1.0, 0.7, abs(y));
    return R(rgb, 0.0);
}
// ---- 4 circle: a magic circle. p0 drawn-in 0..1, p1 cracks 0..1, p2 charge 0..1 (energy running round), p3 spin
fn tri_d(p: vec2f, r: f32) -> f32 {
    // distance to the outline of an equilateral triangle of circumradius r, point up
    let k = 1.7320508;
    var q = vec2f(abs(p.x), p.y);
    let a = vec2f(0.0, r);
    let b = vec2f(r * 0.8660254, -r * 0.5);
    return min(seg(q, a, b), seg(q, b, vec2f(0.0, -r * 0.5)));
}
fn rune(c: vec2f, id: f32) -> f32 {
    // a glyph in a cell (c in -1..1): three strokes between points of a 3x3 lattice, picked by the cell's hash
    var d = 9.0;
    for (var k = 0; k < 3; k++) {
        let h = h22(vec2f(id, f32(k) * 7.3 + 1.0));
        let g2 = h22(vec2f(id + 3.1, f32(k) * 5.9 + 2.0));
        let a = (floor(h * 2.999) - 1.0) * vec2f(0.55, 0.7);
        let b = (floor(g2 * 2.999) - 1.0) * vec2f(0.55, 0.7);
        d = min(d, seg(c, a, b));
    }
    return d;
}
fn k_circle(uv: vec2f, v: VO) -> R {
    let r = length(uv);
    if (r > 1.0) { return R(vec3f(0.0), 0.0); }
    let t = v.a.x;
    let spin = select(v.p.w, 0.5, v.p.w == 0.0);
    let a = atan2(uv.y, uv.x);
    let draw = clamp(v.p.x, 0.0, 1.0);
    // drawn in: each ring sweeps round from its own start
    let sweep = fract(a / TAU + 0.5);
    let on1 = smoothstep(0.0, 0.06, draw * 1.25 - sweep);
    let on2 = smoothstep(0.0, 0.06, draw * 1.25 - fract(sweep + 0.37));
    let on3 = smoothstep(0.2, 0.9, draw);
    var i = 0.0;
    // the outer rings and their tick marks
    i += (line(abs(r - 0.90), 0.011) + line(abs(r - 0.80), 0.007)) * on1;
    let tick = abs(fract((a + t * spin * 0.3) / TAU * 60.0) - 0.5);
    i += smoothstep(0.14, 0.04, tick) * smoothstep(0.035, 0.02, abs(r - 0.94)) * on1 * 0.9;
    // the rune band
    let a1 = a + t * spin;
    let n = 20.0;
    let cell = floor(fract(a1 / TAU) * n);
    let cu = (fract(fract(a1 / TAU) * n) - 0.5) * 2.0;
    let cv = (r - 0.705) / 0.06;
    if (abs(cv) < 1.4) {
        let d = rune(vec2f(cu, cv), cell + floor(v.a.y) * 13.0) * 0.06;
        i += line(d, 0.009) * on2 * 1.1;
    }
    // the inner ring, the hexagram turning the other way, the rings at its points
    i += line(abs(r - 0.61), 0.008) * on2;
    let q = rot2(uv, t * spin * 0.6);
    let star = min(tri_d(q, 0.61), tri_d(vec2f(q.x, -q.y), 0.61));
    i += line(star, 0.007) * on3;
    let qa = atan2(q.y, q.x);
    let sa = (floor(qa / TAU * 6.0 + 0.5)) / 6.0 * TAU;
    let node = length(q - vec2f(cos(sa + 0.5236), sin(sa + 0.5236)) * 0.61);
    let node2 = length(rot2(q, 0.5236) - vec2f(cos(sa), sin(sa)) * 0.61);
    i += (line(abs(min(node, node2) - 0.055), 0.006)) * on3;
    // the heart: a small ring, a second rune band, a core that breathes
    i += line(abs(r - 0.30), 0.007) * on3;
    let a2 = a - t * spin * 1.4;
    let cell2 = floor(fract(a2 / TAU) * 9.0);
    let cu2 = (fract(fract(a2 / TAU) * 9.0) - 0.5) * 2.0;
    let cv2 = (r - 0.215) / 0.055;
    if (abs(cv2) < 1.4) {
        i += line(rune(vec2f(cu2, cv2), cell2 + 71.0 + floor(v.a.y) * 5.0) * 0.055, 0.008) * on3;
    }
    let breathe = 0.6 + 0.4 * sin(t * 5.0);
    let core = exp(-r * r * 60.0) * (0.8 + 1.2 * v.p.z) * breathe * on3;
    // charge: light running round the rings
    let run = 0.7 + (0.3 + 0.9 * v.p.z) * pow(0.5 + 0.5 * sin(a * 3.0 - t * 6.0), 3.0);
    // cracks: a web of fractures opening from the centre, white-hot
    var crack = 0.0;
    if (v.p.y > 0.0) {
        let pc = uv * 4.2 + vec2f(v.a.y * 3.3, v.a.y * 1.7);
        let ip = floor(pc);
        var d1 = 9.0;
        var d2 = 9.0;
        for (var j = -1; j <= 1; j++) {
            for (var k = -1; k <= 1; k++) {
                let o = vec2f(f32(k), f32(j));
                let c = ip + o + h22(ip + o);
                let dd = length(pc - c);
                if (dd < d1) { d2 = d1; d1 = dd; } else if (dd < d2) { d2 = dd; }
            }
        }
        let e = d2 - d1;
        let reach = smoothstep(v.p.y * 1.15, v.p.y * 1.15 - 0.25, r + (vn(uv * 5.0 + v.a.y) - 0.5) * 0.3);
        crack = smoothstep(0.09, 0.015, e) * reach * (0.7 + 0.3 * sin(t * 40.0 + d1 * 20.0));
    }
    let disc = smoothstep(1.0, 0.2, r) * 0.10 * on3 + exp(-pow((r - 0.85) / 0.12, 2.0)) * 0.12 * on1;
    let rgb = v.c0.rgb * (i * run + core) + v.c1.rgb * (disc + i * 0.35) + mix(v.c1.rgb, vec3f(1.0), 0.75) * crack * 2.4;
    return R(rgb * smoothstep(1.0, 0.96, r), disc * 0.6);
}
// ---- 5 flame: fire standing on the quad's bottom edge, flowing up. p0 height 0..1 (default 1), p1 lean, p2 heat
// (0 the ramp's colours, 1 white-hot)
fn k_flame(uv: vec2f, v: VO) -> R {
    let t = v.a.x;
    let y = uv.y * 0.5 + 0.5;
    let asp = v.a.z;
    let hgt = select(v.p.x, 1.0, v.p.x <= 0.0);
    var x = uv.x * asp - v.p.y * y * y;
    let n = fbm(vec2f(x * 2.2 + v.a.y * 7.0, y * 2.4 - t * 2.8));
    let n2 = vn(vec2f(x * 5.0 - v.a.y * 3.0, y * 5.5 - t * 5.0));
    // a single tongue tapers to its tip; a wide strip is a row of them that only falls off at its ends
    var body = 1.0;
    if (asp < 1.6) {
        body = 1.0 - smoothstep(0.0, 1.0, abs(x / asp) / mix(0.95, 0.12, y));
    } else {
        body = smoothstep(1.0, 0.9, abs(uv.x)) * (0.75 + 0.5 * n1(x * 1.3 + v.a.y * 5.0 + t * 0.7));
    }
    let f = body * (1.3 - y * 1.3 / hgt) + (n - 0.5) * 1.0 * (0.25 + y) + (n2 - 0.5) * 0.35 - 0.12;
    let i = smoothstep(0.0, 0.55, f);
    let hot = smoothstep(0.5, 1.0, f + v.p.z * 0.4);
    let rgb = mix(v.c1.rgb * 0.9, v.c0.rgb, smoothstep(0.1, 0.5, f)) * i * 1.3 + vec3f(1.0, 0.92, 0.7) * hot * 1.2;
    return R(rgb * smoothstep(0.0, 0.05, y), i * 0.35);
}
// ---- 6 embers: sparks drifting up through the quad. p0 density (default 6 columns), p1 speed (default 1)
fn k_embers(uv: vec2f, v: VO) -> R {
    let t = v.a.x;
    let den = select(v.p.x, 6.0, v.p.x <= 0.0);
    let spd = select(v.p.y, 1.0, v.p.y <= 0.0);
    let y = uv.y * 0.5 + 0.5;
    var i = 0.0;
    for (var l = 0; l < 2; l++) {
        let fl = f32(l);
        let sc = den * (1.0 + fl * 0.7);
        let gp = vec2f(uv.x * v.a.z * sc * 0.5 + fl * 13.0 + v.a.y * 5.0, y * sc * 0.5 / max(v.a.z, 0.3) * v.a.z - t * spd * (0.9 + fl * 0.5));
        let c = floor(gp);
        let h = h22(c + fl * 71.0);
        if (h.x > 0.45) {
            let sway = sin(t * 3.0 + h.y * 40.0) * 0.18;
            let d = length(fract(gp) - vec2f(0.5 + (h.y - 0.5) * 0.5 + sway, 0.5));
            let tw = 0.6 + 0.4 * sin(t * 11.0 + h.x * 60.0);
            i += (smoothstep(0.11, 0.0, d) + 0.25 * exp(-d * 9.0)) * tw;
        }
    }
    let m = smoothstep(0.0, 0.12, y) * smoothstep(1.0, 0.35, y) * smoothstep(1.0, 0.7, abs(uv.x));
    return R((v.c0.rgb + vec3f(0.5, 0.35, 0.1) * i) * i * m * 1.6, 0.0);
}
// ---- 7 ice: a cluster of crystal shards growing from the quad's bottom middle. p0 growth 0..1, p1 shatter 0..1
fn k_ice(uv: vec2f, v: VO) -> R {
    let q = vec2f(uv.x * v.a.z, uv.y + 1.0);
    let grow = clamp(v.p.x, 0.0, 1.0);
    var rgb = vec3f(0.0);
    var al = 0.0;
    var glow = 0.0;
    for (var j = 0; j < 5; j++) {
        let fj = f32(j);
        let h = h22(vec2f(fj * 3.1 + 1.0, v.a.y * 7.7 + 2.0));
        // the middle shard is the tallest; the outer ones lean out and come up a little later
        let order = abs(fj - 2.0);
        let ang = (fj - 2.0) * 0.36 + (h.x - 0.5) * 0.2;
        let g2 = ease(clamp(grow * 1.5 - order * 0.14, 0.0, 1.0));
        let ln = (1.85 - order * 0.42 + h.y * 0.25) * g2;
        let wd = (0.17 - order * 0.025) * (0.5 + 0.5 * g2);
        let dir = vec2f(sin(ang), cos(ang));
        let o = vec2f((fj - 2.0) * 0.07 * v.a.z, 0.0) + dir * v.p.y * (0.3 + h.x) * 0.6;
        let along = dot(q - o, dir);
        let across = dot(q - o, vec2f(dir.y, -dir.x));
        let prof = wd * min(along / (0.22 * ln + 1.0e-4), (ln - along) / (0.78 * ln + 1.0e-4));
        let ins = prof - abs(across);
        if (ins > 0.0 && along > 0.0) {
            // two facets either side of the ridge, streaks inside, a bright rim and ridge
            let side = step(0.0, across);
            let facet = mix(0.55, 1.0, side) + (vn(vec2f(across * 40.0 + fj * 9.0, along * 3.0)) - 0.5) * 0.25;
            let rim = smoothstep(0.022, 0.0, ins);
            let ridge = smoothstep(0.012, 0.0, abs(across)) * 0.7;
            let tip = smoothstep(0.75, 1.0, along / max(ln, 1.0e-4));
            let c = mix(v.c1.rgb, v.c0.rgb, facet) * (0.55 + 0.45 * facet) + vec3f(1.0) * (rim * 0.9 + ridge + tip * 0.5);
            let cover = smoothstep(0.0, 0.01, ins) * 0.88;
            rgb = mix(rgb, c, cover);
            al = max(al, cover);
        }
        glow += exp(-max(-ins, 0.0) * 9.0) * step(0.0, along + 0.2) * 0.10 * g2;
    }
    // glints, and frost mist at the foot
    let gl = pow(vn(q * 9.0 + vec2f(v.a.x * 1.5, 0.0)), 9.0) * 3.0 * al;
    let mist = exp(-dot(q * vec2f(0.8, 2.6), q * vec2f(0.8, 2.6)) * 1.4) * 0.5 * grow;
    let k = 1.0 - clamp(v.p.y, 0.0, 1.0);
    let ef = smoothstep(1.0, 0.6, abs(uv.x)) * smoothstep(1.0, 0.7, uv.y);
    return R((rgb * al + vec3f(1.0) * gl + v.c0.rgb * (glow + mist) * ef) * k, al * 0.9 * k);
}
// ---- 8 trail: a comet streak, head at +x, tail thinning to -x. p0 wobble 0..1 (ink), p1 head size (default 1)
fn k_trail(uv: vec2f, v: VO) -> R {
    let t = v.a.x;
    let h = uv.x * 0.5 + 0.5;
    let y = uv.y + sin(uv.x * 5.0 + t * 9.0 + v.a.y * 6.0) * 0.28 * v.p.x * (1.0 - h);
    let w = 0.10 + 0.75 * pow(h, 1.6);
    let yy = y / w;
    let body = exp(-yy * yy * 3.2) * pow(h, 1.3);
    // motion streaks: noise stretched along the travel
    let st = 0.55 + 0.75 * vn(vec2f(uv.x * 2.5 * v.a.z - t * 14.0, y * 7.0 + v.a.y * 9.0));
    let hs = select(v.p.y, 1.0, v.p.y <= 0.0);
    let hd = vec2f((uv.x - 0.72) * v.a.z, uv.y) / (0.42 * hs);
    let head = exp(-dot(hd, hd) * 3.0);
    let core = exp(-yy * yy * 22.0) * pow(h, 2.2);
    let rgb = v.c1.rgb * body * st * 0.9 + v.c0.rgb * (body * st * 0.6 + head * 1.6) + vec3f(1.0) * (core * 0.9 + head * head * 1.2);
    return R(rgb * smoothstep(-1.0, -0.85, uv.x), 0.0);
}
// ---- 9 sparks: an impact burst, streaks thrown out and falling. p0 progress 0..1, p1 count (default 26),
// p2 gravity (default 1), p3 spread (1 all round; 0.25 a quarter cone about +x)
fn k_sparks(uv: vec2f, v: VO) -> R {
    let pr = clamp(v.p.x, 0.0, 1.0);
    let n = select(v.p.y, 26.0, v.p.y <= 0.0);
    let grav = select(v.p.z, 1.0, v.p.z < 0.0);
    let spread = select(v.p.w, 1.0, v.p.w <= 0.0);
    var i = 0.0;
    var hot = 0.0;
    // a streak is a couple of pixels thick whatever the burst's size
    let px = fwidth(uv.x) + fwidth(uv.y);
    let a = atan2(uv.y, uv.x) / (TAU * spread);
    let c0 = floor((a + 0.5) * n);
    for (var j = -1; j <= 1; j++) {
        let id = c0 + f32(j);
        if (spread < 0.999 && (id < 0.0 || id >= n)) { continue; }
        let h = h22(vec2f(id + 0.5, v.a.y * 9.1 + 3.0));
        let h2 = h11(id * 1.7 + v.a.y * 4.3);
        let ang = ((id + 0.5 + (h.x - 0.5) * 0.9) / n - 0.5) * TAU * spread;
        let spd = 0.35 + 0.6 * h.y;
        let life = 0.55 + 0.45 * h2;
        let u = clamp(pr / life, 0.0, 1.0);
        let dir = vec2f(cos(ang), sin(ang));
        let head = dir * spd * ease(u) + vec2f(0.0, -0.42 * grav * u * u);
        let tail = head - (dir * spd * (1.0 - u) * 0.55 + vec2f(0.0, -0.3 * grav * u) * 0.4) * (0.45 + 0.55 * (1.0 - u));
        let d = seg(uv, tail, head);
        let th = clamp(0.022 * sqrt(26.0 / n), px * 0.8, px * 2.4 * g.vfx.y / 480.0) * (1.0 - 0.6 * u) * (0.6 + 0.8 * h2);
        let k = pow(1.0 - u, 0.6) * step(u, 0.999) * smoothstep(0.0, 0.25, u);
        i += pow(th / (d + th), 2.0) * k;
        hot += smoothstep(th * 1.5, 0.0, length(uv - head)) * k;
    }
    let pop = exp(-dot(uv, uv) * 60.0) * pow(1.0 - pr, 6.0) * 0.9;
    i = min(i, 1.6) * smoothstep(0.03, 0.22, length(uv));
    return R(v.c0.rgb * i * 2.0 + v.c1.rgb * (i * 0.8 + pop) + vec3f(1.0) * (min(hot, 1.0) * 1.2 + pop * 0.5), 0.0);
}
// ---- 10 star: a four-point sparkle. p0 sharpness (default 1), p1 spin (radians / s), p2 second cross 0..1
fn k_star(uv: vec2f, v: VO) -> R {
    let q = rot2(uv, v.a.x * v.p.y + v.a.y);
    let r = length(q);
    let sh = select(v.p.x, 1.0, v.p.x <= 0.0);
    let a = abs(q.x);
    let b = abs(q.y);
    let ray = (1.0 / (1.0 + a * b * 260.0 * sh)) * smoothstep(1.0, 0.25, max(a, b));
    let q2 = rot2(q, 0.7853982) * 1.9;
    let ray2 = (1.0 / (1.0 + abs(q2.x) * abs(q2.y) * 260.0 * sh)) * smoothstep(1.0, 0.25, max(abs(q2.x), abs(q2.y))) * v.p.z;
    let core = exp(-r * r * 26.0);
    let halo = exp(-r * r * 5.0) * 0.35;
    let tw = 0.85 + 0.15 * sin(v.a.x * 23.0 + v.a.y * 9.0);
    return R((v.c0.rgb * (ray * 1.3 + ray2 + core * 1.2) + v.c1.rgb * halo + vec3f(1.0) * (core * 1.4 + ray * ray * 1.2)) * tw, 0.0);
}
// ---- 11 wisp: a spectre, head and a ragged shroud. p0 facing (-1 / 1), p1 strike flash 0..1
fn k_wisp(uv: vec2f, v: VO) -> R {
    let t = v.a.x;
    let x = uv.x * v.a.z / 0.6;
    let y = uv.y;
    let sway = sin(t * 2.3 + y * 2.5) * 0.06 * (0.6 - y);
    let px = x - sway;
    let head = length(vec2f(px, y - 0.52)) - 0.30;
    let wdt = 0.26 + 0.22 * smoothstep(0.5, -0.9, y);
    let hem = -0.72 + 0.10 * sin(px * 11.0 + t * 4.0) + 0.06 * sin(px * 23.0 - t * 6.0);
    let shroud = max(abs(px) - wdt, max(hem - y, y - 0.5));
    let d = min(head, shroud);
    let n = fbm(vec2f(px * 2.6 + v.a.y * 4.0, y * 2.6 - t * 1.1));
    let body = smoothstep(0.05, -0.08, d + (n - 0.5) * 0.12);
    let rim = smoothstep(0.10, 0.0, abs(d)) * 0.8;
    let thin = smoothstep(-0.9, 0.3, y);
    let f = select(-1.0, 1.0, v.p.x >= 0.0);
    let ex = px - f * 0.06;
    let eyes = smoothstep(0.075, 0.03, length(vec2f((abs(ex) - 0.105) * 1.2, (y - 0.55) * 0.8)));
    let i = body * (0.30 + 0.45 * n) * (0.35 + 0.65 * thin) + rim * (0.4 + 0.6 * thin);
    let rgb = v.c0.rgb * i * (1.0 + 1.5 * v.p.y) * (1.0 - eyes * 0.9) + mix(v.c1.rgb, vec3f(1.0), 0.5) * eyes * (1.3 + 2.0 * v.p.y) + v.c1.rgb * exp(-d * d * 30.0) * 0.25;
    return R(rgb, body * 0.25);
}
// ---- 12 slash: a crescent cut. p0 progress 0..1, p1 arc span in turns (default 0.42), p2 thickness (default 1)
fn k_slash(uv: vec2f, v: VO) -> R {
    let r = length(uv);
    let span = select(v.p.y, 0.42, v.p.y <= 0.0);
    let u = (atan2(uv.y, uv.x) / TAU) / span + 0.5;
    if (u < 0.0 || u > 1.0) { return R(vec3f(0.0), 0.0); }
    let pr = clamp(v.p.x, 0.0, 1.0);
    let front = pr * 1.5;
    let shown = smoothstep(front, front - 0.08, u);
    let trail = exp(-max(front - u, 0.0) * 2.6);
    let thick = pow(sin(u * 3.14159), 0.7) * 0.26 * select(v.p.z, 1.0, v.p.z <= 0.0);
    let ro = 0.9;
    let ri = ro - thick;
    let edge = smoothstep(ro + 0.012, ro - 0.012, r);
    let fill = pow(smoothstep(ri, ro, r), 2.2) * edge;
    let rimline = smoothstep(0.03, 0.0, abs(r - ro + 0.012)) * edge;
    let st = 0.7 + 0.5 * vn(vec2f(u * 26.0, r * 3.0 + v.a.y));
    let k = shown * trail * pow(1.0 - pr, 0.5);
    return R((v.c0.rgb * fill * st * 1.5 + v.c1.rgb * fill * 0.6 + vec3f(1.0) * (rimline * 1.6 + fill * fill * 0.9)) * k, 0.0);
}
// ---- 13 crack: the ground split open along the quad's x axis. p0 spread 0..1, p1 glow 0..1, p2 dust 0..1
fn k_crack(uv: vec2f, v: VO) -> R {
    let sd = v.a.y * 17.3;
    let x = uv.x;
    let y = uv.y + 0.35;
    let spread = clamp(v.p.x, 0.0, 1.0);
    let open = smoothstep(spread, spread - 0.12, abs(x));
    let off = bolt_off(x * 0.98, sd) * 0.5;
    let d = abs(y - off);
    let th = 0.075 * (1.0 - abs(x) * 0.75);
    var dark = smoothstep(th, th * 0.35, d);
    var lava = exp(-d / (th * 1.1));
    for (var j = 0; j < 5; j++) {
        let fj = f32(j);
        let x0 = h11(sd + fj * 3.7) * 1.6 - 0.8;
        let ln = 0.16 + 0.22 * h11(sd * 1.3 + fj * 9.1);
        let dirx = select(-1.0, 1.0, h11(sd * 2.1 + fj) > 0.5);
        let u = (x - x0) * dirx / ln;
        if (u > 0.0 && u < 1.0) {
            let up = select(-1.0, 1.0, h11(sd * 0.7 + fj * 5.3) > 0.35);
            let yb = bolt_off(x0 * 0.98, sd) * 0.5 + up * u * ln * 1.4 + (n1l(x * 15.0 + sd + fj * 5.0) - 0.5) * 0.14 * u;
            let db = abs(y - yb);
            let tb = th * 0.5 * (1.0 - u);
            dark = max(dark, smoothstep(tb, tb * 0.35, db));
            lava += exp(-db / (tb * 1.1 + 1.0e-4)) * (1.0 - u) * 0.7;
        }
    }
    // dust thrown up along the split
    let yu = uv.y * 0.5 + 0.5;
    let puff = fbm(vec2f(x * 3.0 * v.a.z * 0.3 + sd, yu * 3.0 - v.a.x * 1.4)) ;
    let dust = smoothstep(0.45, 0.85, puff + (0.5 - yu) * 0.7) * smoothstep(0.95, 0.35, yu) * smoothstep(0.3, 0.42, yu) * v.p.z * open * smoothstep(1.0, 0.6, abs(x)) * (1.0 - spread * 0.5);
    let glow = lava * v.p.y * open * (0.75 + 0.25 * sin(v.a.x * 17.0 + x * 9.0));
    let rgb = v.c0.rgb * glow * 1.5 * (1.0 - dark * 0.8) + mix(v.c0.rgb, vec3f(1.0, 0.95, 0.8), 0.6) * pow(glow, 3.0) * 0.6 + v.c1.rgb * dust * 0.55;
    return R(rgb, max(dark * open * 0.92, dust * 0.45));
}
// ---- 14 beam: a pillar of light standing on the quad's bottom edge. p0 core 0..1, p1 motes 0..1
fn k_beam(uv: vec2f, v: VO) -> R {
    let t = v.a.x;
    let y = uv.y * 0.5 + 0.5;
    let x = uv.x;
    let fall = pow(1.0 - y, 1.25);
    let wide = exp(-x * x * 3.6) * (0.55 + 0.6 * vn(vec2f(x * 7.0 + v.a.y * 5.0, y * 2.5 - t * 1.3)));
    let core = exp(-x * x * 70.0) * v.p.x;
    var motes = 0.0;
    if (v.p.y > 0.0) {
        let gp = vec2f(x * 4.0 + v.a.y * 3.0, y * 9.0 / max(v.a.z, 0.05) * 0.25 - t * 0.8);
        let h = h22(floor(gp));
        motes = smoothstep(0.16, 0.0, length(fract(gp) - vec2f(0.3 + h.x * 0.4, 0.5))) * step(0.5, h.y) * v.p.y;
    }
    let foot = exp(-(x * x * 2.0 + y * y * 60.0)) * 0.8;
    return R((v.c0.rgb * (wide * 0.8 + foot) + mix(v.c0.rgb, vec3f(1.0), 0.7) * (core + motes * 1.3)) * fall * smoothstep(1.0, 0.8, abs(x)) + v.c1.rgb * foot * 0.5, 0.0);
}
// ---- 15 orb: an energy ball. p0 plasma 0..1, p1 crescent 0..1 (a moon), p2 pulse rate
fn k_orb(uv: vec2f, v: VO) -> R {
    let t = v.a.x;
    let r = length(uv) / 0.52;
    let pulse = 1.0 + 0.08 * sin(t * select(v.p.z, 5.0, v.p.z <= 0.0) + v.a.y * 5.0);
    let rr = r / pulse;
    let ball = smoothstep(1.0, 0.86, rr);
    // a lit sphere: bright toward the upper left, a rim, swirling plasma inside
    let nz = sqrt(max(1.0 - rr * rr, 0.0));
    let lit = 0.45 + 0.55 * clamp(dot(vec3f(uv / 0.52, nz), normalize(vec3f(-0.45, 0.55, 0.7))), 0.0, 1.0);
    let sw = fbm(rot2(uv, t * 0.9 + v.a.y) * 3.4 / (0.4 + nz) + vec2f(0.0, t * 0.6));
    let rim = pow(rr, 5.0) * ball;
    var body = ball * (lit * (0.75 + 0.5 * (sw - 0.5) * v.p.x) + rim * 0.9);
    var cres = 1.0;
    if (v.p.y > 0.0) {
        // a moon: the ball with a bite taken out, the bitten side a dim earthshine
        let bite = smoothstep(0.98, 0.84, length(uv / 0.52 - vec2f(0.42, 0.18)) / 0.92);
        cres = 1.0 - bite * v.p.y * 0.9;
    }
    let halo = exp(-max(r - 0.9, 0.0) * 3.2) * 0.55 * smoothstep(1.92, 1.2, r);
    let spec = exp(-dot(uv / 0.52 - vec2f(-0.34, 0.38), uv / 0.52 - vec2f(-0.34, 0.38)) * 22.0) * ball;
    let rgb = v.c0.rgb * body * cres * 1.25 + mix(v.c0.rgb, vec3f(1.0), 0.8) * (spec * 1.0 + pow(body * cres, 3.0) * 0.5) + v.c1.rgb * (halo * (1.0 - ball * 0.7) + rim * 0.6);
    return R(rgb, ball * 0.75);
}
// ---- 16 dome: a hexagonal shield flaring out. p0 progress 0..1, p1 hex cells across (default 5), p3 refraction
fn hex_r(p: vec2f) -> f32 {
    let q = abs(p);
    return max(q.x * 0.8660254 + q.y * 0.5, q.y);
}
fn hex_edge(p: vec2f) -> f32 {
    // distance to the nearest edge of a hexagonal tiling (0 on an edge, 0.5 at a cell centre)
    let s = vec2f(1.0, 1.7320508);
    let a = (fract(p / s) - 0.5) * s;
    let b = (fract(p / s + 0.5) - 0.5) * s;
    let q = select(b, a, dot(a, a) < dot(b, b));
    return 0.5 - hex_r(vec2f(q.y, q.x));
}
fn k_dome(uv: vec2f, v: VO) -> R {
    let pr = clamp(v.p.x, 0.0, 1.0);
    let size = 0.35 + 0.6 * ease(pr * 1.6);
    let q = rot2(uv, 0.5236) / size;
    let rh = hex_r(q);
    if (rh > 1.08) { return R(vec3f(0.0), 0.0); }
    let cells = select(v.p.y, 5.0, v.p.y <= 0.0);
    let e = hex_edge(q * cells * 0.5);
    let grid = smoothstep(0.07, 0.015, e);
    let fres = pow(clamp(rh, 0.0, 1.0), 4.0);
    let wave = exp(-pow((rh - fract(pr * 2.2)) / 0.2, 2.0));
    let edge = smoothstep(0.05, 0.0, abs(rh - 1.0));
    let inside = smoothstep(1.02, 0.98, rh);
    let fade = pow(1.0 - pr, 0.8);
    let flash = exp(-rh * rh * 3.0) * pow(1.0 - pr, 6.0);
    let rgb = (v.c0.rgb * (grid * (0.25 + 1.3 * wave) + fres * 0.9) * inside + mix(v.c0.rgb, vec3f(1.0), 0.7) * (edge * 1.8 + flash * 1.5) + v.c1.rgb * inside * (0.14 + 0.3 * wave)) * fade;
    return R(rgb, inside * 0.12 * fade);
}
// ---- 17 mine: a spiked charge with a warning light. p0 light 0..1, p1 warning ring 0..1 (0 none), p2 armed 0..1
fn k_mine(uv: vec2f, v: VO) -> R {
    let r = length(uv);
    let a = atan2(uv.y, uv.x) + v.a.y;
    let spike = 0.40 + 0.2 * pow(abs(cos(a * 4.0)), 12.0);
    let body = smoothstep(spike + 0.02, spike - 0.02, r);
    let ball = smoothstep(0.42, 0.38, r);
    let nz = sqrt(max(1.0 - (r / 0.42) * (r / 0.42), 0.0));
    let lit = 0.25 + 0.75 * clamp(dot(vec3f(uv / 0.42, nz), normalize(vec3f(-0.5, 0.6, 0.62))), 0.0, 1.0);
    let metal = mix(vec3f(0.10, 0.09, 0.14), vec3f(0.55, 0.52, 0.62), lit * lit) * ball + vec3f(0.30, 0.28, 0.36) * (body - ball) * (0.6 + 0.4 * lit);
    let band = smoothstep(0.03, 0.0, abs(r - 0.27)) * 0.5 * ball;
    let led = smoothstep(0.15, 0.10, r);
    let on = clamp(v.p.x, 0.0, 1.0);
    let halo = exp(-r * r * 7.0) * on * (0.5 + 0.8 * v.p.z);
    var ring = 0.0;
    if (v.p.y > 0.0) {
        ring = line(abs(r - (0.5 + 0.45 * v.p.y)), 0.012) * (1.0 - v.p.y) * 0.9;
    }
    let rgb = metal + vec3f(0.8) * band * lit + v.c0.rgb * (led * (0.25 + 1.6 * on) + halo + ring) + vec3f(1.0) * led * on * on * 0.9;
    return R(rgb, body * 0.95);
}
// ---- 18 coin: a coin turning on its upright axis. p0 turns per second (default 1.5), p1 sparkle 0..1
fn k_coin(uv: vec2f, v: VO) -> R {
    let ph = v.a.x * select(v.p.x, 1.5, v.p.x <= 0.0) * TAU + v.a.y * 5.0;
    let cs = cos(ph);
    let w = max(abs(cs), 0.14);
    let q = vec2f(uv.x / w, uv.y) / 0.72;
    let r = length(q);
    let disc = smoothstep(1.0, 0.94, r);
    let rimz = smoothstep(0.80, 0.86, r) * disc;
    let face = 0.55 + 0.45 * abs(cs);
    let shade = 0.7 + 0.3 * (uv.y * 0.5 + 0.5);
    // the stamp: a four-point star
    let st = smoothstep(0.36, 0.30, abs(q.x) + abs(q.y)) - smoothstep(0.22, 0.18, abs(q.x) + abs(q.y)) * 0.6;
    let sweep = exp(-pow((q.x + q.y * 0.5 - sin(ph) * 1.6) * 2.2, 2.0)) * disc;
    let glint = exp(-dot(uv, uv) * 4.0) * pow(abs(cs), 24.0) * v.p.y;
    let q3 = rot2(uv, 0.2);
    let cross = (1.0 / (1.0 + abs(q3.x) * abs(q3.y) * 300.0)) * smoothstep(1.0, 0.3, length(uv)) * pow(abs(cs), 24.0) * v.p.y;
    let rgb = v.c0.rgb * disc * face * shade * (0.8 + rimz * 0.5 + st * 0.35) + mix(v.c0.rgb, vec3f(1.0), 0.75) * (sweep * 0.7 + glint + cross * 1.5) + v.c1.rgb * exp(-dot(uv, uv) * 3.0) * 0.18;
    return R(rgb, disc * 0.95);
}
// ---- 20 aura: flames licking round a fighter, hollow so the fighter shows. p0 strength 0..1
fn k_aura(uv: vec2f, v: VO) -> R {
    let t = v.a.x;
    let n = fbm(vec2f(uv.x * 2.6 * v.a.z + v.a.y * 5.0, uv.y * 2.2 - t * 2.3));
    let ell = length(vec2f(uv.x / 0.78, (uv.y + 0.12) / 0.86));
    let up = uv.y * 0.5 + 0.5;
    let band = smoothstep(0.42, 0.80, ell) * smoothstep(1.02, 0.80, ell + (0.5 - n) * 0.55 - up * 0.10);
    let i = band * (0.2 + 1.5 * n * n) * (0.45 + 0.75 * up) * clamp(v.p.x, 0.0, 1.0) * 1.3;
    return R(v.c1.rgb * i * 0.7 + v.c0.rgb * i * i * 1.6 + vec3f(1.0, 0.9, 0.7) * pow(i, 4.0) * 0.8, 0.0);
}
// ---- 21 ring: a thin standing ring of light. p0 radius (default 0.8), p1 line width (default 0.035), p2 dashes
// (0 a whole ring), p3 turn rate
fn k_ring(uv: vec2f, v: VO) -> R {
    let r = length(uv);
    let rad = select(v.p.x, 0.8, v.p.x <= 0.0);
    let w = select(v.p.y, 0.035, v.p.y <= 0.0);
    let a = atan2(uv.y, uv.x) + v.a.x * v.p.w;
    var gate = 1.0;
    if (v.p.z > 0.0) {
        gate = smoothstep(0.2, 0.35, abs(fract(a / TAU * v.p.z) - 0.5));
    }
    let run = 0.75 + 0.25 * sin(a * 3.0 - v.a.x * 5.0 + v.a.y);
    let i = line(abs(r - rad), w) * gate * run;
    let fill = smoothstep(rad, rad * 0.3, r) * 0.07;
    return R(v.c0.rgb * i * 1.3 + v.c1.rgb * (i * 0.5 + fill), 0.0);
}
fn shade(v: VO) -> R {
    let uv = v.uv;
    switch u32(v.a.w + 0.5) {
        case 1u: { return k_glow(uv, v); }
        case 2u: { return k_shock(uv, v); }
        case 3u: { return k_bolt(uv, v); }
        case 4u: { return k_circle(uv, v); }
        case 5u: { return k_flame(uv, v); }
        case 6u: { return k_embers(uv, v); }
        case 7u: { return k_ice(uv, v); }
        case 8u: { return k_trail(uv, v); }
        case 9u: { return k_sparks(uv, v); }
        case 10u: { return k_star(uv, v); }
        case 11u: { return k_wisp(uv, v); }
        case 12u: { return k_slash(uv, v); }
        case 13u: { return k_crack(uv, v); }
        case 14u: { return k_beam(uv, v); }
        case 15u: { return k_orb(uv, v); }
        case 16u: { return k_dome(uv, v); }
        case 17u: { return k_mine(uv, v); }
        case 18u: { return k_coin(uv, v); }
        case 20u: { return k_aura(uv, v); }
        case 21u: { return k_ring(uv, v); }
        default: { return R(vec3f(0.0), 0.0); }
    }
}
@fragment fn fs_main(v: VO) -> FO {
    let s = shade(v);
    // intensity: Subtle dims the light a little; the mod scales its own opacity with the player's setting
    let k = v.c0.a * mix(0.7, 1.0, clamp(g.misc.y * 2.0 - 1.0, 0.0, 1.0));
    var rgb = max(s.rgb, vec3f(0.0)) * k;
    // light past full brightness runs to white instead of clipping to a flat colour
    let m = max(rgb.r, max(rgb.g, rgb.b));
    if (m > 1.0) {
        rgb = mix(rgb / m, vec3f(1.0), clamp((m - 1.0) * 0.45, 0.0, 1.0));
    }
    // what it covers: solid parts by their own cover, light by a little of its brightness (so it reads on a bright
    // stage and never turns a fighter behind it into a white blot)
    let lum = dot(rgb, vec3f(0.3, 0.5, 0.2));
    let a = clamp(s.a * k + max(lum - 0.45, 0.0) * 0.5 * (1.0 - s.a), 0.0, 0.96);
    var o: FO;
    o.c = vec4f(rgb * mix(1.0, a, s.a), a);
    o.e = vec4f(rgb * (1.0 - s.a * 0.7) * 0.5 * g.misc.y, 1.0);
    return o;
}
// the offsets (EFB uv units) the blit bends the picture by: a shock's ring pushes outward, haze shimmers upward
@fragment fn fs_distort(v: VO) -> @location(0) vec4f {
    let uv = v.uv;
    let kind = u32(v.a.w + 0.5);
    var d = vec2f(0.0);
    let k = v.c0.a * g.misc.y;
    if (kind == 2u || kind == 16u) {
        let r = length(uv);
        let pr = clamp(v.p.x, 0.0, 1.0);
        var rad = 0.06 + 0.84 * ease(pr);
        var w = select(v.p.y, 0.08, v.p.y <= 0.0) * 1.6 + 0.03;
        if (kind == 16u) {
            rad = 0.35 + 0.6 * ease(pr * 1.6);
            w = 0.12;
        }
        let x = (r - rad) / w;
        let amp = select(v.p.w, 1.0, v.p.w <= 0.0);
        d = uv / max(r, 1.0e-3) * exp(-x * x) * 0.012 * amp * (1.0 - pr) * smoothstep(1.0, 0.85, r);
    } else if (kind == 19u) {
        // haze: p0 strength, rising shimmer, faded to nothing at the quad's edge
        let t = v.a.x;
        let q = vec2f(uv.x * v.a.z * 3.0 + v.a.y * 7.0, uv.y * 3.0 - t * 1.6);
        let n = vec2f(vn(q) + 0.5 * vn(q * 2.1 + vec2f(t * 0.7, 0.0)), vn(q * 1.3 + vec2f(5.0, t * 0.9)) + 0.5 * vn(q * 2.3)) - 0.75;
        let m = smoothstep(1.0, 0.3, length(uv));
        d = n * 0.006 * select(v.p.x, 1.0, v.p.x <= 0.0) * m;
    }
    return vec4f(d * k / vec2f(g.vfx.x / (4.0 / 3.0), 1.0), 0.0, 0.0);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inst_is_64_bytes() {
        assert_eq!(std::mem::size_of::<VfxInst>(), INST_BYTES as usize);
    }

    #[test]
    fn queue_refuses_bad_input_and_caps() {
        begin();
        assert!(!push(VfxInst { kind: 0, ..Default::default() }));
        assert!(!push(VfxInst { kind: K_COUNT, ..Default::default() }));
        assert!(!push(VfxInst { kind: K_BOLT, t: f32::NAN, ..Default::default() }));
        assert!(!push(VfxInst { kind: K_BOLT, flags: 16, ..Default::default() }));
        for _ in 0..MAX_VFX {
            assert!(push(VfxInst { kind: K_BOLT, ..Default::default() }));
        }
        assert!(!push(VfxInst { kind: K_BOLT, ..Default::default() }));
        commit();
        assert_eq!(shown(), MAX_VFX);
        begin();
        commit();
        assert_eq!(shown(), 0);
    }

    #[test]
    fn shader_parses() {
        let m = naga_check(VFX_WGSL);
        assert!(m.is_ok(), "{m:?}");
    }

    fn naga_check(src: &str) -> Result<(), String> {
        let module = wgpu::naga::front::wgsl::parse_str(src).map_err(|e| e.emit_to_string(src))?;
        wgpu::naga::valid::Validator::new(wgpu::naga::valid::ValidationFlags::all(), wgpu::naga::valid::Capabilities::all()).validate(&module).map(|_| ()).map_err(|e| format!("{e:?}"))
    }
}

impl crate::render::Renderer {
    /// A development aid (the effect gallery, `tests/vfx_gallery.rs`): put `backdrop` (RGBA8, the EFB's size) into
    /// the EFB and draw `list` over it through the armed effects layer, as a fight's frame would.
    pub fn vfx_preview(&mut self, backdrop: Option<&[u8]>, list: &[VfxInst], intensity: f32) {
        if let Some(px) = backdrop.filter(|p| p.len() == (self.width * self.height * 4) as usize) {
            // through the EFB blit pipeline (the EFB itself is not a copy destination)
            let fmt = crate::render::COLOR_FORMAT;
            if !self.blit.contains_key(&fmt) {
                let _ = self.read_composed(&[]);
            }
            let size = wgpu::Extent3d { width: self.width, height: self.height, depth_or_array_layers: 1 };
            let tex = self.device.create_texture(&wgpu::TextureDescriptor { label: Some("vfx backdrop"), size, mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2, format: fmt, usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST, view_formats: &[] });
            self.queue.write_texture(wgpu::TexelCopyTextureInfo { texture: &tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All }, px, wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(self.width * 4), rows_per_image: Some(self.height) }, size);
            let smp = self.device.create_sampler(&wgpu::SamplerDescriptor { mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, ..Default::default() });
            let (pipe, bgl) = &self.blit[&fmt];
            let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("vfx backdrop"),
                layout: bgl,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&tex.create_view(&Default::default())) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&smp) },
                ],
            });
            let view = self.color.create_view(&Default::default());
            let mut enc = self.device.create_command_encoder(&Default::default());
            {
                let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("vfx backdrop"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &view, resolve_target: None, depth_slice: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store } })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                pass.set_pipeline(pipe);
                pass.set_bind_group(0, &bg, &[]);
                pass.draw(0..3, 0..1);
            }
            self.queue.submit([enc.finish()]);
        }
        let mut f = crate::fx::FxFrame { intensity, ..Default::default() };
        f.vfx.extend_from_slice(list);
        self.set_fx(&mut f);
        self.fx_world();
    }
}
