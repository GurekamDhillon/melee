//! wgpu backend: draws [`DrawCmd`]s into an offscreen colour + depth target, the way Aurora's
//! `gx::render` / `build_pipeline` (`lib/gx/gx.cpp`, `pipeline.cpp`) draw a `DrawData`.
//!
//! Same resource model as Aurora: one storage buffer of raw vertex bytes (`vbuf`), one of attribute
//! array snapshots (`abuf`), per-draw uniform blocks at 256-byte dynamic offsets, a 64-byte
//! immediate block (Aurora: Dawn immediates; here: push constants), texture group 2 with eight
//! texture/sampler pairs, reversed Z, clockwise front faces.
//!
//! The WGSL is Aurora's text (see [`crate::shader`]) put through [`adapt_wgsl`], two mechanical
//! rewrites naga needs and Dawn does not: `var<immediate>` -> `var<push_constant>`, and pointer
//! parameters into storage buffers (`ptr<storage, array<u32>>`, which naga rejects) -> a buffer
//! selector (0 = vbuf, 1 = abuf). Loads read the same words either way.
use crate::consts::*;
use crate::draw::DrawCmd;
use crate::state::*;
use crate::resources::GxData;
use crate::texture;
use std::collections::HashMap;

type FxMap<K, V> = HashMap<K, V, rustc_hash::FxBuildHasher>;

pub const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
const IMMEDIATE_SIZE: u32 = 64;
const UNIFORM_ALIGN: u64 = 256;

/// Rewrite Aurora's WGSL into the dialect naga/wgpu accepts (see the module docs).
pub fn adapt_wgsl(src: &str) -> String {
    let mut s = src
        .replace("var<immediate>", "var<push_constant>")
        .replace("ptr<storage, array<u32>>", "u32")
        .replace("&vbuf", "0u")
        .replace("&abuf", "1u");
    let head = "fn load_word(p: u32, word_idx: u32) -> u32 {";
    if let Some(a) = s.find(head) {
        let b = a + s[a..].find("\n}\n").expect("load_word end") + 3;
        s.replace_range(
            a..b,
            "fn load_word(p: u32, word_idx: u32) -> u32 {\n  if (p == 0u) {\n    if (word_idx < arrayLength(&vbuf)) {\n      return vbuf[word_idx];\n    }\n    return 0u;\n  }\n  if (word_idx < arrayLength(&abuf)) {\n    return abuf[word_idx];\n  }\n  return 0u;\n}\n",
        );
    }
    // naga does not const-evaluate builtins over abstract floats, so an alpha combiner whose inputs
    // are all literals (`mix(0.0, 0.0, 0.0)`) leaves an abstract-typed runtime expression and fails
    // validation; Tint folds it. Typing the literals of alpha-combiner lines as f32 is value-exact
    // (every such literal - 0.0, 0.5, 1.0, k/8.0, 255.0... - is an exact f32).
    let mut out = String::with_capacity(s.len() + 1024);
    for line in s.split_inclusive('\n') {
        if line.trim_start().starts_with("prev.a = clamp(") || line.contains(".a = clamp((") {
            out.push_str(&type_float_literals(line));
        } else {
            out.push_str(line);
        }
    }
    out
}

/// Wrap every bare float literal (`12.5`) in `f32(...)`.
fn type_float_literals(line: &str) -> String {
    let b = line.as_bytes();
    let mut out = String::with_capacity(line.len() + 64);
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        let prev_ident = i > 0 && (b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_' || b[i - 1] == b'.');
        if c.is_ascii_digit() && !prev_ident {
            let start = i;
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
            if i + 1 < b.len() && b[i] == b'.' && b[i + 1].is_ascii_digit() {
                i += 1;
                while i < b.len() && b[i].is_ascii_digit() {
                    i += 1;
                }
                out.push_str("f32(");
                out.push_str(&line[start..i]);
                out.push(')');
            } else {
                out.push_str(&line[start..i]);
            }
            continue;
        }
        out.push(c as char);
        i += 1;
    }
    out
}

fn blend_factor(f: u32, is_dst: bool) -> wgpu::BlendFactor {
    use wgpu::BlendFactor as B;
    match f {
        GX_BL_ZERO => B::Zero,
        GX_BL_ONE => B::One,
        GX_BL_SRCCLR => {
            if is_dst {
                B::Src
            } else {
                B::Dst
            }
        }
        GX_BL_INVSRCCLR => {
            if is_dst {
                B::OneMinusSrc
            } else {
                B::OneMinusDst
            }
        }
        GX_BL_SRCALPHA => B::SrcAlpha,
        GX_BL_INVSRCALPHA => B::OneMinusSrcAlpha,
        GX_BL_DSTALPHA => B::DstAlpha,
        GX_BL_INVDSTALPHA => B::OneMinusDstAlpha,
        _ => panic!("invalid blend factor {f}"),
    }
}

/// Aurora `to_compare_function` (reversed Z).
fn compare(f: u32) -> wgpu::CompareFunction {
    use wgpu::CompareFunction as C;
    match f {
        GX_NEVER => C::Never,
        GX_LESS => C::Greater,
        GX_EQUAL => C::Equal,
        GX_LEQUAL => C::GreaterEqual,
        GX_GREATER => C::Less,
        GX_NEQUAL => C::NotEqual,
        GX_GEQUAL => C::LessEqual,
        GX_ALWAYS => C::Always,
        _ => panic!("invalid depth fn {f}"),
    }
}

/// Aurora `to_blend_state`.
fn blend_state(mode: u32, src: u32, dst: u32, op: u32, dst_alpha: u32) -> wgpu::BlendState {
    use wgpu::{BlendComponent as BC, BlendFactor as B, BlendOperation as O};
    let color = match mode {
        GX_BM_NONE => BC { operation: O::Add, src_factor: B::One, dst_factor: B::Zero },
        GX_BM_BLEND => BC { operation: O::Add, src_factor: blend_factor(src, false), dst_factor: blend_factor(dst, true) },
        GX_BM_SUBTRACT => BC { operation: O::ReverseSubtract, src_factor: B::One, dst_factor: B::One },
        GX_BM_LOGIC => match op {
            GX_LO_CLEAR => BC { operation: O::Add, src_factor: B::Zero, dst_factor: B::Zero },
            GX_LO_COPY => BC { operation: O::Add, src_factor: B::One, dst_factor: B::Zero },
            GX_LO_NOOP => BC { operation: O::Add, src_factor: B::Zero, dst_factor: B::One },
            _ => panic!("unsupported logic op {op}"),
        },
        _ => panic!("unsupported blend mode {mode}"),
    };
    let alpha =
        if dst_alpha != u32::MAX { BC { operation: O::Add, src_factor: B::Constant, dst_factor: B::Zero } } else { color };
    wgpu::BlendState { color, alpha }
}

fn address_mode(w: u32) -> wgpu::AddressMode {
    match w {
        GX_CLAMP => wgpu::AddressMode::ClampToEdge,
        GX_REPEAT => wgpu::AddressMode::Repeat,
        GX_MIRROR => wgpu::AddressMode::MirrorRepeat,
        _ => panic!("invalid wrap mode {w}"),
    }
}

/// (filter, mip filter or None) for a GX filter (Aurora `wgpu_filter_mode`).
fn filter_mode(f: u32) -> (wgpu::FilterMode, Option<wgpu::FilterMode>) {
    use wgpu::{FilterMode as F, FilterMode as M};
    match f {
        GX_NEAR => (F::Nearest, None),
        GX_LINEAR => (F::Linear, None),
        GX_NEAR_MIP_NEAR => (F::Nearest, Some(M::Nearest)),
        GX_LIN_MIP_NEAR => (F::Linear, Some(M::Nearest)),
        GX_NEAR_MIP_LIN => (F::Nearest, Some(M::Linear)),
        GX_LIN_MIP_LIN => (F::Linear, Some(M::Linear)),
        _ => panic!("invalid filter mode {f}"),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct SamplerKey {
    wrap_s: u32,
    wrap_t: u32,
    min: u32,
    mag: u32,
    min_lod: u32,
    max_lod: u32,
}

/// Aurora `TextureBind::get_descriptor` (no replacements, anisotropy 1).
fn sampler_key(o: &TexObj) -> SamplerKey {
    let (_, mip) = filter_mode(o.min_filter());
    let (min_lod, max_lod) = if mip.is_none() { (0.0f32, 0.0f32) } else { (o.min_lod(), o.max_lod()) };
    SamplerKey {
        wrap_s: o.wrap_s(),
        wrap_t: o.wrap_t(),
        min: o.min_filter(),
        mag: o.mag_filter(),
        min_lod: min_lod.to_bits(),
        max_lod: max_lod.to_bits(),
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct PipelineKey {
    wgsl_hash: u64,
    cull: u32,
    blend: (u32, u32, u32, u32, bool),
    depth: (bool, bool, u32),
    write: (bool, bool),
    bias: (u32, u32, u32),
}

/// Decoded-texture cache key: the texture's content identity (guest bytes, shape, format and, for
/// palette formats, the TLUT bytes and layout). NOT the GXTexObj id: the HSD layer re-inits a fresh
/// TexObj on every `load_texture`, so an id-keyed cache created a new GPU texture per draw per frame.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct TexKey {
    ptr: usize,
    len: usize,
    w: u32,
    h: u32,
    fmt: u32,
    mips: u32,
    tlut: (usize, usize, u32, u16),
}

/// A cached texture; `_data` / `_tlut` keep the source `Arc`s alive so their addresses (the key) cannot be reused.
struct TexEntry {
    _tex: wgpu::Texture,
    view: wgpu::TextureView,
    _data: GxData,
    _tlut: Option<GxData>,
}

/// Offscreen GX renderer.
pub struct Renderer {
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter_info: wgpu::AdapterInfo,
    pub width: u32,
    pub height: u32,
    /// Internal resolution multiplier over the 640x480 EFB (`MELEE_RENDER_SCALE`): GX viewport / scissor
    /// coordinates are scaled by it.
    pub scale: u32,
    color: wgpu::Texture,
    depth: wgpu::Texture,
    bgl_storage: wgpu::BindGroupLayout,
    bgl_uniform: wgpu::BindGroupLayout,
    bgl_textures: wgpu::BindGroupLayout,
    layout: wgpu::PipelineLayout,
    pipelines: FxMap<PipelineKey, wgpu::RenderPipeline>,
    wgsl_hashes: FxMap<crate::shader::ShaderConfig, u64>,
    tex_bgs: FxMap<[Option<(TexKey, SamplerKey)>; MAX_TEXTURES], wgpu::BindGroup>,
    samplers: FxMap<SamplerKey, wgpu::Sampler>,
    textures: FxMap<TexKey, TexEntry>,
    empty_view: wgpu::TextureView,
    empty_sampler: wgpu::Sampler,
    /// WGSL sources that failed to compile, with naga's message.
    pub shader_errors: Vec<String>,
    blit: HashMap<wgpu::TextureFormat, (wgpu::RenderPipeline, wgpu::BindGroupLayout)>,
    blit_sampler: Option<wgpu::Sampler>,
    /// The last [`Renderer::prepare`]d frame: persistent GPU buffers, per-draw pipeline / bind group / offsets.
    frame: FrameCache,
}

/// One draw of the prepared frame.
struct PreparedDraw {
    pipeline: Option<wgpu::RenderPipeline>,
    tex_bg: Option<wgpu::BindGroup>,
    imm: [u8; 64],
    uoff: u32,
    first_index: u32,
    /// This draw owns its uniform block (false: it shares the previous draw's identical block).
    owner: bool,
}

/// A GPU buffer reused across frames (grown to the next power of two when a frame needs more).
struct GpuBuf {
    buf: wgpu::Buffer,
    cap: u64,
}

#[derive(Default)]
struct FrameCache {
    draws: Vec<PreparedDraw>,
    vbuf: Vec<u8>,
    abuf: Vec<u8>,
    ibuf: Vec<u8>,
    ubuf: Vec<u8>,
    arr_max: FxMap<usize, usize>,
    arr_off: FxMap<usize, u32>,
    gv: Option<GpuBuf>,
    ga: Option<GpuBuf>,
    gi: Option<GpuBuf>,
    gu: Option<GpuBuf>,
    storage_bg: Option<wgpu::BindGroup>,
    uniform_bg: Option<(wgpu::BindGroup, u64)>,
    max_uniform: u64,
    /// 0: `ubuf` must be refilled from the draws; 1: it holds their uniforms; 2: `fill` rewrote parts of it.
    ubuf_state: u8,
    /// The previous draw's pipeline config and its pipeline (consecutive draws mostly share one).
    last_pipe: Option<(crate::shader::PipelineConfig, Option<wgpu::RenderPipeline>)>,
}

/// Make `slot` hold at least `size` bytes; true when the buffer was (re)created.
fn ensure_buf(device: &wgpu::Device, slot: &mut Option<GpuBuf>, size: u64, usage: wgpu::BufferUsages, label: &str) -> bool {
    if slot.as_ref().is_some_and(|b| b.cap >= size) {
        return false;
    }
    let cap = size.next_power_of_two().max(4096);
    let buf = device.create_buffer(&wgpu::BufferDescriptor { label: Some(label), size: cap, usage: usage | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
    *slot = Some(GpuBuf { buf, cap });
    true
}

/// `MELEE_GX_PROF`: print per-frame renderer timings.
fn prof() -> bool {
    static PROF: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *PROF.get_or_init(|| std::env::var_os("MELEE_GX_PROF").is_some())
}


fn fnv(d: &[u8]) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for &b in d {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

impl Renderer {
    /// A headless renderer with a `width` x `height` target. `backends` defaults to the platform's
    /// primary backends; WGPU_BACKEND (e.g. `dx12`, `vulkan`) overrides as in wgpu.
    pub fn new(width: u32, height: u32) -> Result<Self, String> {
        Self::with_instance(&Self::instance(), None, width, height)
    }

    /// `MELEE_RENDER_SCALE` (1..=4), when set.
    pub fn env_scale() -> Option<u32> {
        std::env::var("MELEE_RENDER_SCALE").ok().and_then(|v| v.parse::<u32>().ok()).map(|s| s.clamp(1, 4))
    }

    /// The internal scale that renders a 640x480 EFB at (at least) a `w` x `h` window's native resolution.
    pub fn scale_for_window(w: u32, h: u32) -> u32 {
        (w.div_ceil(640)).max(h.div_ceil(480)).clamp(1, 4)
    }

    /// Change the internal resolution multiplier: recreates the colour / depth targets (contents are lost).
    pub fn set_scale(&mut self, scale: u32) {
        let scale = scale.clamp(1, 4);
        if scale == self.scale {
            return;
        }
        let (w, h) = (self.width / self.scale * scale, self.height / self.scale * scale);
        let mk = |format, usage, label| {
            self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            })
        };
        self.color = mk(
            COLOR_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::TEXTURE_BINDING,
            "GX EFB color",
        );
        self.depth = mk(DEPTH_FORMAT, wgpu::TextureUsages::RENDER_ATTACHMENT, "GX EFB depth");
        self.width = w;
        self.height = h;
        self.scale = scale;
    }

    /// The wgpu instance [`Renderer::new`] uses (primary backends, WGPU_BACKEND overrides).
    pub fn instance() -> wgpu::Instance {
        let backends = wgpu::Backends::from_env().unwrap_or(wgpu::Backends::PRIMARY);
        wgpu::Instance::new(&wgpu::InstanceDescriptor { backends, ..Default::default() })
    }

    /// A renderer on `instance` whose adapter can present to `surface` (when given). The EFB
    /// stays an offscreen `width` x `height` target; [`Renderer::blit_to`] scales it out.
    pub fn with_instance(
        instance: &wgpu::Instance,
        surface: Option<&wgpu::Surface<'_>>,
        width: u32,
        height: u32,
    ) -> Result<Self, String> {
        let scale = Self::env_scale().unwrap_or(1);
        let (width, height) = (width * scale, height * scale);
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: surface,
        }))
        .map_err(|e| format!("no adapter: {e}"))?;
        let adapter_info = adapter.get_info();
        if !adapter.features().contains(wgpu::Features::PUSH_CONSTANTS) {
            return Err(format!("adapter {} lacks PUSH_CONSTANTS", adapter_info.name));
        }
        let limits = wgpu::Limits { max_push_constant_size: IMMEDIATE_SIZE, ..wgpu::Limits::default() };
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("melee_gx"),
            required_features: wgpu::Features::PUSH_CONSTANTS,
            required_limits: limits,
            ..Default::default()
        }))
        .map_err(|e| format!("request_device: {e}"))?;
        let mk_target = |format, usage, label| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            })
        };
        let color = mk_target(
            COLOR_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::TEXTURE_BINDING,
            "GX EFB color",
        );
        let depth = mk_target(DEPTH_FORMAT, wgpu::TextureUsages::RENDER_ATTACHMENT, "GX EFB depth");
        let vs_fs = wgpu::ShaderStages::VERTEX_FRAGMENT;
        let storage = |b| wgpu::BindGroupLayoutEntry {
            binding: b,
            visibility: vs_fs,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let bgl_storage = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("GX storage"),
            entries: &[storage(0), storage(1)],
        });
        let bgl_uniform = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("GX uniform"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: vs_fs,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let mut tex_entries = Vec::new();
        for i in 0..MAX_TEXTURES as u32 {
            tex_entries.push(wgpu::BindGroupLayoutEntry {
                binding: i * 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            });
            tex_entries.push(wgpu::BindGroupLayoutEntry {
                binding: i * 2 + 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });
        }
        let bgl_textures = device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("GX textures"), entries: &tex_entries });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("GX pipeline layout"),
            bind_group_layouts: &[&bgl_storage, &bgl_uniform, &bgl_textures],
            push_constant_ranges: &[wgpu::PushConstantRange { stages: vs_fs, range: 0..IMMEDIATE_SIZE }],
        });
        let empty = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Empty texture"),
            size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let empty_view = empty.create_view(&Default::default());
        let empty_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        Ok(Self {
            adapter,
            device,
            queue,
            adapter_info,
            width,
            height,
            scale,
            color,
            depth,
            bgl_storage,
            bgl_uniform,
            bgl_textures,
            layout,
            pipelines: FxMap::default(),
            wgsl_hashes: FxMap::default(),
            tex_bgs: FxMap::default(),
            samplers: FxMap::default(),
            textures: FxMap::default(),
            empty_view,
            empty_sampler,
            shader_errors: Vec::new(),
            blit: HashMap::new(),
            blit_sampler: None,
            frame: FrameCache::default(),
        })
    }

    fn pipeline(&mut self, d: &DrawCmd) -> Option<wgpu::RenderPipeline> {
        let pc = &d.config;
        // Generating the WGSL per draw per frame dominated render time: memoise its hash by config.
        let wgsl_hash = match self.wgsl_hashes.get(&pc.shader) {
            Some(&h) => h,
            None => {
                let h = fnv(crate::shader::build_shader_source(&pc.shader, u32::MAX).as_bytes());
                self.wgsl_hashes.insert(pc.shader.clone(), h);
                h
            }
        };
        let key = PipelineKey {
            wgsl_hash,
            cull: pc.cull_mode,
            blend: (pc.blend_mode, pc.blend_fac_src, pc.blend_fac_dst, pc.blend_op, pc.dst_alpha != u32::MAX),
            depth: (pc.depth_compare, pc.depth_update, pc.depth_func),
            write: (pc.color_update, pc.alpha_update),
            bias: (pc.polygon_offset.to_bits(), pc.polygon_offset_scale.to_bits(), pc.polygon_offset_clamp.to_bits()),
        };
        if let Some(p) = self.pipelines.get(&key) {
            return Some(p.clone());
        }
        let src = crate::shader::build_shader_source(&pc.shader, u32::MAX);
        let wgsl = adapt_wgsl(&src);
        self.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let module = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("GX shader"),
            source: wgpu::ShaderSource::Wgsl(wgsl.clone().into()),
        });
        let cull = match pc.cull_mode {
            GX_CULL_FRONT => Some(wgpu::Face::Front),
            GX_CULL_BACK => Some(wgpu::Face::Back),
            _ => None,
        };
        let writes_depth = pc.depth_compare && pc.depth_update;
        let mut write_mask = wgpu::ColorWrites::empty();
        if pc.color_update {
            write_mask |= wgpu::ColorWrites::COLOR;
        }
        if pc.alpha_update {
            write_mask |= wgpu::ColorWrites::ALPHA;
        }
        let bias_const = -pc.polygon_offset;
        let pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("GX pipeline"),
            layout: Some(&self.layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: cull,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: writes_depth,
                depth_compare: if pc.depth_compare { compare(pc.depth_func) } else { wgpu::CompareFunction::Always },
                stencil: Default::default(),
                bias: wgpu::DepthBiasState {
                    constant: if bias_const < 0.0 { bias_const.floor() as i32 } else { bias_const.ceil() as i32 },
                    slope_scale: -pc.polygon_offset_scale,
                    clamp: pc.polygon_offset_clamp,
                },
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: COLOR_FORMAT,
                    blend: Some(blend_state(pc.blend_mode, pc.blend_fac_src, pc.blend_fac_dst, pc.blend_op, pc.dst_alpha)),
                    write_mask,
                })],
            }),
            multiview: None,
            cache: None,
        });
        if let Some(e) = pollster::block_on(self.device.pop_error_scope()) {
            self.shader_errors.push(format!("{e}\n--- WGSL ---\n{wgsl}"));
            return None;
        }
        self.pipelines.insert(key, pipeline.clone());
        Some(pipeline)
    }

    /// The cache key `texture_view` would use for `map`, if the texture is already decoded.
    fn cached_tex_key(&self, d: &DrawCmd, map: usize) -> Option<TexKey> {
        let o = &d.textures[map];
        let data = o.data.as_ref()?;
        let fmt = o.format();
        let pal = matches!(fmt, GX_TF_C4 | GX_TF_C8 | GX_TF_C14X2);
        let tl = if pal { d.tluts.get(o.tlut as usize) } else { None };
        let tlut_key = match (tl, tl.and_then(|t| t.data.as_ref())) {
            (Some(t), Some(td)) => (td.as_ptr() as usize, td.len(), t.format, t.num_entries),
            _ => (0, 0, 0, 0),
        };
        let key = TexKey { ptr: data.as_ptr() as usize, len: data.len(), w: o.width(), h: o.height(), fmt, mips: o.mip_count(), tlut: tlut_key };
        self.textures.contains_key(&key).then_some(key)
    }

    fn texture_view(&mut self, d: &DrawCmd, map: usize) -> Option<wgpu::TextureView> {
        let o = &d.textures[map];
        let data = o.data.as_ref()?;
        let fmt = o.format();
        let (w, h, mips) = (o.width(), o.height(), o.mip_count());
        let pal = matches!(fmt, GX_TF_C4 | GX_TF_C8 | GX_TF_C14X2);
        let tl = if pal { d.tluts.get(o.tlut as usize) } else { None };
        let tlut_data = tl.and_then(|t| t.data.clone());
        let tlut_key = match (tl, &tlut_data) {
            (Some(t), Some(td)) => (td.as_ptr() as usize, td.len(), t.format, t.num_entries),
            _ => (0, 0, 0, 0),
        };
        let key = TexKey { ptr: data.as_ptr() as usize, len: data.len(), w, h, fmt, mips, tlut: tlut_key };
        if let Some(e) = self.textures.get(&key) {
            return Some(e.view.clone());
        }
        if w == 0 || h == 0 {
            return None;
        }
        let conv = if matches!(fmt, GX_TF_C4 | GX_TF_C8 | GX_TF_C14X2) {
            let t = &d.tluts[o.tlut as usize];
            texture::convert_texture_palette(fmt, w, h, mips, data, t.format, t.num_entries, t.data.as_deref()?)
        } else {
            texture::convert_texture(fmt, w, h, mips, data)
        };
        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("GX texture"),
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: mips,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let (mut mw, mut mh) = (w, h);
        let mut off = 0usize;
        for mip in 0..mips {
            let n = (mw * mh * 4) as usize;
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo { texture: &tex, mip_level: mip, origin: wgpu::Origin3d::ZERO, aspect: Default::default() },
                &conv.data[off..off + n],
                wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(mw * 4), rows_per_image: Some(mh) },
                wgpu::Extent3d { width: mw, height: mh, depth_or_array_layers: 1 },
            );
            off += n;
            mw = (mw >> 1).max(1);
            mh = (mh >> 1).max(1);
        }
        let view = tex.create_view(&Default::default());
        self.textures.insert(key, TexEntry { _tex: tex, view: view.clone(), _data: data.clone(), _tlut: tlut_data });
        Some(view)
    }

    fn sampler(&mut self, o: &TexObj) -> wgpu::Sampler {
        let k = sampler_key(o);
        if let Some(s) = self.samplers.get(&k) {
            return s.clone();
        }
        let (min, mip) = filter_mode(k.min);
        let (mag, _) = filter_mode(k.mag);
        let s = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("GX sampler"),
            address_mode_u: address_mode(k.wrap_s),
            address_mode_v: address_mode(k.wrap_t),
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: mag,
            min_filter: min,
            mipmap_filter: mip.unwrap_or(wgpu::FilterMode::Nearest),
            lod_min_clamp: f32::from_bits(k.min_lod),
            lod_max_clamp: f32::from_bits(k.max_lod),
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });
        self.samplers.insert(k, s.clone());
        s
    }

    /// Clear the target to `state`'s copy-clear colour / depth, draw `draws`, return RGBA8 pixels.
    pub fn render(&mut self, state: &GxState, draws: &[DrawCmd]) -> Vec<u8> {
        self.render_frame(state, draws);
        self.read_color()
    }

    /// [`Renderer::render`] without the readback: clear, draw, submit (the EFB stays on the GPU
    /// for [`Renderer::blit_to`] / [`Renderer::read_color`]).
    pub fn render_frame(&mut self, state: &GxState, draws: &[DrawCmd]) {
        self.prepare(state, draws);
        self.draw_prepared(state, draws, None);
    }

    /// The per-frame work that does not depend on the uniforms: pack and upload every draw's vertices, array
    /// snapshots and indices into the persistent buffers, look up its pipeline and texture bind group. A frame
    /// presented several times (interpolation) is prepared once; [`Renderer::draw_prepared`] then only rewrites
    /// the uniforms and records the pass.
    pub fn prepare(&mut self, _state: &GxState, draws: &[DrawCmd]) {
        let t0 = std::time::Instant::now();
        let mut fc = std::mem::take(&mut self.frame);
        fc.vbuf.clear();
        fc.abuf.clear();
        fc.ibuf.clear();
        fc.arr_max.clear();
        fc.arr_off.clear();
        fc.draws.clear();
        // Draws share array snapshots (a model's vertex arrays): upload each one once per frame.
        // Keyed by the snapshot's allocation; the upload covers the most any draw this frame needs.
        for a in draws.iter().flat_map(|d| d.arrays.iter().flatten()) {
            let n = (a.needed as usize).min(a.data.len());
            let e = fc.arr_max.entry(a.data.as_ptr() as usize).or_insert(0);
            *e = (*e).max(n);
        }
        let mut uoff = 0u32;
        let mut max_uniform = 16usize;
        let mut prev_d: Option<&DrawCmd> = None;
        for d in draws {
            let mut imm = [0u32; 16];
            imm[0] = fc.vbuf.len() as u32;
            imm[1] = d.current_pn_mtx;
            fc.vbuf.extend_from_slice(&d.verts);
            fc.vbuf.resize(fc.vbuf.len().next_multiple_of(4), 0);
            for (i, a) in d.arrays.iter().enumerate() {
                if let Some(a) = a {
                    let k = a.data.as_ptr() as usize;
                    if let Some(&o) = fc.arr_off.get(&k) {
                        imm[4 + i] = o;
                        continue;
                    }
                    let n = fc.arr_max[&k];
                    imm[4 + i] = fc.abuf.len() as u32;
                    fc.arr_off.insert(k, fc.abuf.len() as u32);
                    fc.abuf.extend_from_slice(&a.data[..n]);
                    fc.abuf.resize(fc.abuf.len().next_multiple_of(4) + 4, 0);
                }
            }
            let first_index = (fc.ibuf.len() / 2) as u32;
            for i in &d.indices {
                fc.ibuf.extend_from_slice(&i.to_le_bytes());
            }
            fc.ibuf.resize(fc.ibuf.len().next_multiple_of(4), 0);
            let mut immb = [0u8; 64];
            for (k, w) in imm.iter().enumerate() {
                immb[k * 4..k * 4 + 4].copy_from_slice(&w.to_le_bytes());
            }
            // Consecutive draws of one joint mostly carry identical uniform blocks: share one (same matrix slot and
            // skinning, so an interpolating `fill` of the owner covers them all).
            let share = prev_d.is_some_and(|p: &DrawCmd| {
                p.current_pn_mtx == d.current_pn_mtx && p.config.shader.attrs[0] == d.config.shader.attrs[0] && p.uniform == d.uniform
            });
            if share {
                let u = fc.draws.last().unwrap().uoff;
                fc.draws.push(PreparedDraw { pipeline: None, tex_bg: None, imm: immb, uoff: u, first_index, owner: false });
            } else {
                fc.draws.push(PreparedDraw { pipeline: None, tex_bg: None, imm: immb, uoff, first_index, owner: true });
                uoff += (d.uniform.len() as u32).next_multiple_of(UNIFORM_ALIGN as u32);
                max_uniform = max_uniform.max(d.uniform.len());
            }
            prev_d = Some(d);
        }
        fc.max_uniform = (max_uniform as u64).next_multiple_of(16);
        let ubytes = uoff as u64 + fc.max_uniform;
        fc.ubuf.resize(ubytes as usize, 0);
        fc.ubuf_state = 0;
        let t1 = std::time::Instant::now();
        let st = wgpu::BufferUsages::STORAGE;
        let mut regroup = ensure_buf(&self.device, &mut fc.gv, fc.vbuf.len() as u64 + 4, st, "GX vbuf");
        regroup |= ensure_buf(&self.device, &mut fc.ga, fc.abuf.len() as u64 + 4, st, "GX abuf");
        ensure_buf(&self.device, &mut fc.gi, fc.ibuf.len() as u64 + 4, wgpu::BufferUsages::INDEX, "GX indices");
        let ugrown = ensure_buf(&self.device, &mut fc.gu, ubytes, wgpu::BufferUsages::UNIFORM, "GX uniforms");
        for (g, b) in [(&fc.gv, &fc.vbuf), (&fc.ga, &fc.abuf), (&fc.gi, &fc.ibuf)] {
            if !b.is_empty() {
                self.queue.write_buffer(&g.as_ref().unwrap().buf, 0, b);
            }
        }
        if regroup || fc.storage_bg.is_none() {
            fc.storage_bg = Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("GX storage"),
                layout: &self.bgl_storage,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: fc.gv.as_ref().unwrap().buf.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: fc.ga.as_ref().unwrap().buf.as_entire_binding() },
                ],
            }));
        }
        if ugrown || fc.uniform_bg.as_ref().is_none_or(|(_, m)| *m != fc.max_uniform) {
            let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("GX uniform"),
                layout: &self.bgl_uniform,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &fc.gu.as_ref().unwrap().buf,
                        offset: 0,
                        size: std::num::NonZeroU64::new(fc.max_uniform),
                    }),
                }],
            });
            fc.uniform_bg = Some((bg, fc.max_uniform));
        }
        let t2 = std::time::Instant::now();
        // Pipelines and texture groups (they borrow self mutably). Consecutive draws mostly share a config:
        // compare with the previous draw's before hashing.
        let mut last_pipe = fc.last_pipe.take();
        for (i, d) in draws.iter().enumerate() {
            let p = if d.config.cull_mode == GX_CULL_ALL {
                None
            } else {
                match last_pipe.as_ref() {
                    Some((cfg, p)) if *cfg == d.config => p.clone(),
                    _ => {
                        let p = self.pipeline(d);
                        last_pipe = Some((d.config.clone(), p.clone()));
                        p
                    }
                }
            };
            fc.draws[i].pipeline = p;
            fc.draws[i].tex_bg = Some(self.tex_bind_group(d));
        }
        fc.last_pipe = last_pipe;
        self.frame = fc;
        if prof() {
            eprintln!(
                "gxprof prepare n={} pack={:?} upload={:?} lookup={:?} vb={} ab={} ub={}",
                draws.len(),
                t1 - t0,
                t2 - t1,
                t2.elapsed(),
                self.frame.vbuf.len(),
                self.frame.abuf.len(),
                self.frame.ubuf.len()
            );
        }
    }

    /// The texture bind group of `d` (cached by decoded texture and sampler per map).
    fn tex_bind_group(&mut self, d: &DrawCmd) -> wgpu::BindGroup {
        // Texture bind groups are cached by (decoded texture, sampler) per map: the decoded
        // texture cache never evicts, so its keys stay valid for the bind group's lifetime.
        let used = |i: usize| d.info.sampled_textures & (1 << i) != 0 || d.info.sampled_ind_textures & (1 << i) != 0;
        let mut bkey: [Option<(TexKey, SamplerKey)>; MAX_TEXTURES] = Default::default();
        let mut cacheable = true;
        for (i, k) in bkey.iter_mut().enumerate() {
            if used(i) {
                match self.cached_tex_key(d, i) {
                    Some(t) => *k = Some((t, sampler_key(&d.textures[i]))),
                    None => cacheable = false,
                }
            }
        }
        if cacheable {
            if let Some(bg) = self.tex_bgs.get(&bkey) {
                return bg.clone();
            }
        }
        let mut views = Vec::with_capacity(MAX_TEXTURES);
        let mut samplers = Vec::with_capacity(MAX_TEXTURES);
        for i in 0..MAX_TEXTURES {
            let v = if used(i) { self.texture_view(d, i) } else { None };
            if let Some(v) = v {
                views.push(v);
                samplers.push(self.sampler(&d.textures[i]));
            } else {
                views.push(self.empty_view.clone());
                samplers.push(self.empty_sampler.clone());
            }
        }
        let mut entries = Vec::with_capacity(MAX_TEXTURES * 2);
        for i in 0..MAX_TEXTURES {
            entries.push(wgpu::BindGroupEntry { binding: (i * 2) as u32, resource: wgpu::BindingResource::TextureView(&views[i]) });
            entries.push(wgpu::BindGroupEntry { binding: (i * 2 + 1) as u32, resource: wgpu::BindingResource::Sampler(&samplers[i]) });
        }
        let tex_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("GX textures"), layout: &self.bgl_textures, entries: &entries });
        if cacheable {
            self.tex_bgs.insert(bkey, tex_bg.clone());
        }
        tex_bg
    }

    /// Draw the last [`Renderer::prepare`]d frame: `draws` must be that frame (same draws, same order). Each
    /// draw's uniform block is copied from `draws`, then `fill(i, block)` may rewrite it (interpolated
    /// matrices); the blocks are uploaded once and the pass is recorded with redundant state changes skipped.
    pub fn draw_prepared(&mut self, state: &GxState, draws: &[DrawCmd], mut fill: Option<&mut dyn FnMut(usize, &mut [u8])>) {
        let t0 = std::time::Instant::now();
        let fc = &mut self.frame;
        assert_eq!(fc.draws.len(), draws.len(), "draw_prepared: not the prepared frame");
        // The uniform blocks are copied once per prepared frame; `fill` only rewrites what it changes
        // (it must write every byte it ever changes, from its own inputs, each time it is called).
        if fc.ubuf_state == 0 || (fill.is_none() && fc.ubuf_state == 2) {
            for (i, d) in draws.iter().enumerate() {
                if fc.draws[i].owner {
                    let o = fc.draws[i].uoff as usize;
                    fc.ubuf[o..o + d.uniform.len()].copy_from_slice(&d.uniform);
                }
            }
            fc.ubuf_state = 1;
        }
        if let Some(f) = fill.as_mut() {
            for (i, d) in draws.iter().enumerate() {
                if fc.draws[i].owner {
                    let o = fc.draws[i].uoff as usize;
                    f(i, &mut fc.ubuf[o..o + d.uniform.len()]);
                }
            }
            fc.ubuf_state = 2;
        }
        if let Some(g) = fc.gu.as_ref() {
            self.queue.write_buffer(&g.buf, 0, &fc.ubuf);
        }
        let t1 = std::time::Instant::now();
        let color_view = self.color.create_view(&Default::default());
        let depth_view = self.depth.create_view(&Default::default());
        let mut enc = self.device.create_command_encoder(&Default::default());
        {
            let cc = state.clear_color;
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("GX EFB"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &color_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: cc[0] as f64, g: cc[1] as f64, b: cc[2] as f64, a: cc[3] as f64 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(state.clear_depth_value()), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            if let (Some(sbg), Some(gi), Some((ubg, _))) = (fc.storage_bg.as_ref(), fc.gi.as_ref(), fc.uniform_bg.as_ref()) {
                pass.set_bind_group(0, sbg, &[]);
                pass.set_index_buffer(gi.buf.slice(..), wgpu::IndexFormat::Uint16);
                let (w, h) = (self.width as f32, self.height as f32);
                let k = self.scale as f32;
                let ki = self.scale as i32;
                let mut cur_vp = [f32::NAN; 6];
                let mut cur_sc = [u32::MAX; 4];
                let mut cur_pipe: Option<&wgpu::RenderPipeline> = None;
                let mut cur_tex: Option<&wgpu::BindGroup> = None;
                let mut cur_blend = f64::NAN;
                let mut cur_uoff = u32::MAX;
                for (i, d) in draws.iter().enumerate() {
                    let pd = &fc.draws[i];
                    let (Some(p), Some(tex_bg)) = (pd.pipeline.as_ref(), pd.tex_bg.as_ref()) else { continue };
                    let sc = d.render_scissor;
                    let sx = (sc.x * ki).clamp(0, self.width as i32) as u32;
                    let sy = (sc.y * ki).clamp(0, self.height as i32) as u32;
                    let sw = ((sc.width * ki).max(0) as u32).min(self.width - sx);
                    let sh = ((sc.height * ki).max(0) as u32).min(self.height - sy);
                    if sw == 0 || sh == 0 {
                        continue;
                    }
                    let vp = d.render_viewport;
                    let left = (vp.left * k).clamp(0.0, w);
                    let top = (vp.top * k).clamp(0.0, h);
                    let v = [left, top, (vp.width * k).min(w - left).max(1.0), (vp.height * k).min(h - top).max(1.0), 1.0 - vp.zfar, 1.0 - vp.znear];
                    if v.iter().zip(cur_vp.iter()).any(|(a, b)| a.to_bits() != b.to_bits()) {
                        pass.set_viewport(v[0], v[1], v[2], v[3], v[4], v[5]);
                        cur_vp = v;
                    }
                    if [sx, sy, sw, sh] != cur_sc {
                        pass.set_scissor_rect(sx, sy, sw, sh);
                        cur_sc = [sx, sy, sw, sh];
                    }
                    if cur_pipe.is_none_or(|c| c != p) {
                        pass.set_pipeline(p);
                        cur_pipe = Some(p);
                    }
                    if pd.uoff != cur_uoff {
                        pass.set_bind_group(1, ubg, &[pd.uoff]);
                        cur_uoff = pd.uoff;
                    }
                    if cur_tex.is_none_or(|c| c != tex_bg) {
                        pass.set_bind_group(2, tex_bg, &[]);
                        cur_tex = Some(tex_bg);
                    }
                    pass.set_push_constants(wgpu::ShaderStages::VERTEX_FRAGMENT, 0, &pd.imm);
                    if d.config.dst_alpha != u32::MAX {
                        let a = d.config.dst_alpha as f64 / 255.0;
                        if a.to_bits() != cur_blend.to_bits() {
                            pass.set_blend_constant(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a });
                            cur_blend = a;
                        }
                    }
                    if d.indices.is_empty() {
                        pass.draw(0..d.vtx_count, 0..d.instance_count);
                    } else {
                        let n = d.indices.len() as u32;
                        pass.draw_indexed(pd.first_index..pd.first_index + n, 0, 0..d.instance_count);
                    }
                }
            }
        }
        self.queue.submit([enc.finish()]);
        if prof() {
            eprintln!("gxprof draw n={} uniforms={:?} pass={:?}", draws.len(), t1 - t0, t1.elapsed());
        }
    }

    /// Scale the EFB colour target onto `target` (a `tw` x `th` swapchain image of `format`). This
    /// is the display half of GXCopyDisp + VI: alpha is forced to 1 (the EFB's alpha is not the
    /// display's), filtering is linear like the XFB scaler. The EFB keeps its 4:3 shape, centred
    /// with black bars, unless `MELEE_ASPECT=stretch` asks for the old fill.
    pub fn blit_to(&mut self, target: &wgpu::TextureView, format: wgpu::TextureFormat, tw: u32, th: u32) {
        const BLIT_WGSL: &str = "
@group(0) @binding(0) var efb: texture_2d<f32>;
@group(0) @binding(1) var smp: sampler;
struct VsOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> VsOut {
    let uv = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    var o: VsOut;
    o.pos = vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, 0.0, 1.0);
    o.uv = uv;
    return o;
}
@fragment fn fs_main(v: VsOut) -> @location(0) vec4<f32> {
    return vec4<f32>(textureSample(efb, smp, v.uv).rgb, 1.0);
}
";
        if !self.blit.contains_key(&format) {
            let module = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("EFB blit"),
                source: wgpu::ShaderSource::Wgsl(BLIT_WGSL.into()),
            });
            let bgl = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("EFB blit"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
            let layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("EFB blit"),
                bind_group_layouts: &[&bgl],
                push_constant_ranges: &[],
            });
            let pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("EFB blit"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState { format, blend: None, write_mask: wgpu::ColorWrites::ALL })],
                }),
                multiview: None,
                cache: None,
            });
            self.blit.insert(format, (pipeline, bgl));
        }
        if self.blit_sampler.is_none() {
            self.blit_sampler = Some(self.device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("EFB blit"),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }));
        }
        let (pipeline, bgl) = &self.blit[&format];
        let view = self.color.create_view(&Default::default());
        let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("EFB blit"),
            layout: bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(self.blit_sampler.as_ref().unwrap()),
                },
            ],
        });
        let mut enc = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("EFB blit"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            let (tw, th) = (tw.max(1) as f32, th.max(1) as f32);
            let stretch = std::env::var("MELEE_ASPECT").map_or(false, |v| v == "stretch");
            if !stretch {
                // Fit the EFB's aspect (640x480 times the render scale) inside the target.
                let aspect = self.width as f32 / self.height as f32;
                let (w, h) = if tw / th > aspect { (th * aspect, th) } else { (tw, tw / aspect) };
                pass.set_viewport(((tw - w) * 0.5).floor(), ((th - h) * 0.5).floor(), w, h, 0.0, 1.0);
            }
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &bg, &[]);
            pass.draw(0..3, 0..1);
        }
        self.queue.submit([enc.finish()]);
    }


    /// Copy the colour target back to the CPU (RGBA8, row-major, tight rows).
    pub fn read_color(&self) -> Vec<u8> {
        let row = self.width * 4;
        let padded = row.div_ceil(256) * 256;
        let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GX readback"),
            size: (padded * self.height) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = self.device.create_command_encoder(&Default::default());
        enc.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo { texture: &self.color, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: Default::default() },
            wgpu::TexelCopyBufferInfo {
                buffer: &buf,
                layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(padded), rows_per_image: Some(self.height) },
            },
            wgpu::Extent3d { width: self.width, height: self.height, depth_or_array_layers: 1 },
        );
        self.queue.submit([enc.finish()]);
        let slice = buf.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::PollType::wait_indefinitely()).expect("poll");
        let data = slice.get_mapped_range();
        let mut out = Vec::with_capacity((row * self.height) as usize);
        for y in 0..self.height {
            let o = (y * padded) as usize;
            out.extend_from_slice(&data[o..o + row as usize]);
        }
        out
    }
}
