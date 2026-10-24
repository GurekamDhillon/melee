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
use crate::texture;
use std::collections::HashMap;

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
    pipelines: HashMap<PipelineKey, wgpu::RenderPipeline>,
    samplers: HashMap<SamplerKey, wgpu::Sampler>,
    textures: HashMap<(u32, u32, usize, u32), (wgpu::Texture, wgpu::TextureView)>,
    empty_view: wgpu::TextureView,
    empty_sampler: wgpu::Sampler,
    /// WGSL sources that failed to compile, with naga's message.
    pub shader_errors: Vec<String>,
    blit: HashMap<wgpu::TextureFormat, (wgpu::RenderPipeline, wgpu::BindGroupLayout)>,
    blit_sampler: Option<wgpu::Sampler>,
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
        let scale = std::env::var("MELEE_RENDER_SCALE").ok().and_then(|v| v.parse::<u32>().ok()).unwrap_or(1).clamp(1, 4);
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
            pipelines: HashMap::new(),
            samplers: HashMap::new(),
            textures: HashMap::new(),
            empty_view,
            empty_sampler,
            shader_errors: Vec::new(),
            blit: HashMap::new(),
            blit_sampler: None,
        })
    }

    fn pipeline(&mut self, d: &DrawCmd) -> Option<wgpu::RenderPipeline> {
        let pc = &d.config;
        let src = crate::shader::build_shader_source(&pc.shader, u32::MAX);
        let key = PipelineKey {
            wgsl_hash: fnv(src.as_bytes()),
            cull: pc.cull_mode,
            blend: (pc.blend_mode, pc.blend_fac_src, pc.blend_fac_dst, pc.blend_op, pc.dst_alpha != u32::MAX),
            depth: (pc.depth_compare, pc.depth_update, pc.depth_func),
            write: (pc.color_update, pc.alpha_update),
            bias: (pc.polygon_offset.to_bits(), pc.polygon_offset_scale.to_bits(), pc.polygon_offset_clamp.to_bits()),
        };
        if let Some(p) = self.pipelines.get(&key) {
            return Some(p.clone());
        }
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

    fn texture_view(&mut self, d: &DrawCmd, map: usize) -> Option<wgpu::TextureView> {
        let o = &d.textures[map];
        let data = o.data.as_ref()?;
        let key = (o.tex_obj_id, o.tex_data_version, data.as_ptr() as usize, o.tlut);
        if let Some((_, v)) = self.textures.get(&key) {
            return Some(v.clone());
        }
        let fmt = o.format();
        let (w, h, mips) = (o.width(), o.height(), o.mip_count());
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
        self.textures.insert(key, (tex, view.clone()));
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
        // Pack every draw's vertices, array snapshots and uniforms into three buffers.
        let mut vbuf: Vec<u8> = Vec::new();
        let mut abuf: Vec<u8> = Vec::new();
        let mut ubuf: Vec<u8> = Vec::new();
        let mut ibuf: Vec<u8> = Vec::new();
        struct Prepared {
            imm: [u8; 64],
            uoff: u32,
            ioff: u64,
        }
        let mut prep = Vec::with_capacity(draws.len());
        for d in draws {
            let mut imm = [0u32; 16];
            imm[0] = vbuf.len() as u32;
            imm[1] = d.current_pn_mtx;
            vbuf.extend_from_slice(&d.verts);
            while vbuf.len() % 4 != 0 {
                vbuf.push(0);
            }
            for (i, a) in d.arrays.iter().enumerate() {
                if let Some(a) = a {
                    imm[4 + i] = abuf.len() as u32;
                    let n = (a.needed as usize).min(a.data.len());
                    abuf.extend_from_slice(&a.data[..n]);
                    while abuf.len() % 4 != 0 {
                        abuf.push(0);
                    }
                    abuf.extend_from_slice(&[0; 4]);
                }
            }
            let uoff = ubuf.len() as u32;
            ubuf.extend_from_slice(&d.uniform);
            ubuf.resize(ubuf.len().div_ceil(UNIFORM_ALIGN as usize) * UNIFORM_ALIGN as usize, 0);
            let ioff = ibuf.len() as u64;
            for i in &d.indices {
                ibuf.extend_from_slice(&i.to_le_bytes());
            }
            while ibuf.len() % 4 != 0 {
                ibuf.push(0);
            }
            let mut immb = [0u8; 64];
            for (k, w) in imm.iter().enumerate() {
                immb[k * 4..k * 4 + 4].copy_from_slice(&w.to_le_bytes());
            }
            prep.push(Prepared { imm: immb, uoff, ioff });
        }
        let max_uniform = draws.iter().map(|d| d.uniform.len()).max().unwrap_or(16).max(16) as u64;
        let max_uniform = max_uniform.div_ceil(16) * 16;
        ubuf.resize(ubuf.len() + max_uniform as usize, 0);
        for b in [&mut vbuf, &mut abuf, &mut ubuf, &mut ibuf] {
            if b.len() < 256 {
                b.resize(256, 0);
            }
        }
        use wgpu::util::DeviceExt;
        let mk = |data: &[u8], usage, label| {
            self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some(label), contents: data, usage })
        };
        let vb = mk(&vbuf, wgpu::BufferUsages::STORAGE, "GX vbuf");
        let ab = mk(&abuf, wgpu::BufferUsages::STORAGE, "GX abuf");
        let ub = mk(&ubuf, wgpu::BufferUsages::UNIFORM, "GX uniforms");
        let ib = mk(&ibuf, wgpu::BufferUsages::INDEX, "GX indices");
        let storage_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("GX storage"),
            layout: &self.bgl_storage,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: vb.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: ab.as_entire_binding() },
            ],
        });
        let uniform_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("GX uniform"),
            layout: &self.bgl_uniform,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &ub,
                    offset: 0,
                    size: std::num::NonZeroU64::new(max_uniform),
                }),
            }],
        });
        // Pipelines and texture groups first (they borrow self mutably).
        let mut per_draw = Vec::with_capacity(draws.len());
        for d in draws {
            let p = if d.config.cull_mode == GX_CULL_ALL { None } else { self.pipeline(d) };
            let mut views = Vec::new();
            let mut samplers = Vec::new();
            for i in 0..MAX_TEXTURES {
                let used = d.info.sampled_textures & (1 << i) != 0 || d.info.sampled_ind_textures & (1 << i) != 0;
                let v = if used { self.texture_view(d, i) } else { None };
                if let Some(v) = v {
                    views.push(v);
                    samplers.push(self.sampler(&d.textures[i]));
                } else {
                    views.push(self.empty_view.clone());
                    samplers.push(self.empty_sampler.clone());
                }
            }
            let mut entries = Vec::new();
            for i in 0..MAX_TEXTURES {
                entries.push(wgpu::BindGroupEntry {
                    binding: (i * 2) as u32,
                    resource: wgpu::BindingResource::TextureView(&views[i]),
                });
                entries.push(wgpu::BindGroupEntry {
                    binding: (i * 2 + 1) as u32,
                    resource: wgpu::BindingResource::Sampler(&samplers[i]),
                });
            }
            let tex_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("GX textures"),
                layout: &self.bgl_textures,
                entries: &entries,
            });
            per_draw.push((p, tex_bg));
        }
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
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: cc[0] as f64,
                            g: cc[1] as f64,
                            b: cc[2] as f64,
                            a: cc[3] as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(state.clear_depth_value()),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_bind_group(0, &storage_bg, &[]);
            pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint16);
            for (i, d) in draws.iter().enumerate() {
                let (Some(p), tex_bg) = &per_draw[i] else { continue };
                let vp = d.render_viewport;
                let k = self.scale as f32;
                let vp = crate::state::Viewport { left: vp.left * k, top: vp.top * k, width: vp.width * k, height: vp.height * k, ..vp };
                let (w, h) = (self.width as f32, self.height as f32);
                let left = vp.left.clamp(0.0, w);
                let top = vp.top.clamp(0.0, h);
                pass.set_viewport(
                    left,
                    top,
                    vp.width.min(w - left).max(1.0),
                    vp.height.min(h - top).max(1.0),
                    1.0 - vp.zfar,
                    1.0 - vp.znear,
                );
                let sc = d.render_scissor;
                let ki = self.scale as i32;
                let sc = crate::state::ClipRect { x: sc.x * ki, y: sc.y * ki, width: sc.width * ki, height: sc.height * ki, ..sc };
                let sx = sc.x.clamp(0, self.width as i32) as u32;
                let sy = sc.y.clamp(0, self.height as i32) as u32;
                let sw = (sc.width.max(0) as u32).min(self.width - sx);
                let sh = (sc.height.max(0) as u32).min(self.height - sy);
                if sw == 0 || sh == 0 {
                    continue;
                }
                pass.set_scissor_rect(sx, sy, sw, sh);
                pass.set_pipeline(p);
                pass.set_bind_group(1, &uniform_bg, &[prep[i].uoff]);
                pass.set_bind_group(2, tex_bg, &[]);
                pass.set_push_constants(wgpu::ShaderStages::VERTEX_FRAGMENT, 0, &prep[i].imm);
                if d.config.dst_alpha != u32::MAX {
                    pass.set_blend_constant(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: d.config.dst_alpha as f64 / 255.0 });
                }
                if d.indices.is_empty() {
                    pass.draw(0..d.vtx_count, 0..d.instance_count);
                } else {
                    let n = d.indices.len() as u32;
                    let first = (prep[i].ioff / 2) as u32;
                    pass.draw_indexed(first..first + n, 0, 0..d.instance_count);
                }
            }
        }
        self.queue.submit([enc.finish()]);
    }

    /// Scale the EFB colour target onto `target` (a swapchain image of `format`), filling it. This
    /// is the display half of GXCopyDisp + VI: alpha is forced to 1 (the EFB's alpha is not the
    /// display's), filtering is linear like the XFB scaler.
    pub fn blit_to(&mut self, target: &wgpu::TextureView, format: wgpu::TextureFormat) {
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
