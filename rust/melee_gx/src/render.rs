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
            // AND/OR/XOR/INV/SET and the rest need the destination in the shader (wgpu blending
            // has no logic ops). Melee's draws observed so far never rely on
            // them; draw as COPY and say so once per op instead of aborting the frame.
            _ => {
                static SEEN: std::sync::atomic::AtomicU16 = std::sync::atomic::AtomicU16::new(0);
                if SEEN.fetch_or(1 << (op & 15), std::sync::atomic::Ordering::Relaxed) & (1 << (op & 15)) == 0 {
                    eprintln!("gx: logic op {op} is drawn as GX_LO_COPY (not expressible as wgpu blending)");
                }
                BC { operation: O::Add, src_factor: B::One, dst_factor: B::Zero }
            }
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
    /// The last [`Renderer::prepare`] count it was used at (see `TEX_KEEP`).
    used: std::cell::Cell<u64>,
}

/// A decoded texture not used for this many prepared frames is dropped (with every cached bind group): the cache
/// is keyed by guest address, so each match's files added entries that were never used again.
const TEX_KEEP: u64 = 600;

/// Offscreen GX renderer.
pub struct Renderer {
    pub adapter: wgpu::Adapter,
    /// Microseconds the last `draw_prepared` spent in its `fill` callbacks (frame interpolation / widescreen patching).
    pub last_fill_us: u32,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter_info: wgpu::AdapterInfo,
    pub width: u32,
    pub height: u32,
    /// Internal resolution multiplier over the 640x480 EFB (`MELEE_RENDER_SCALE`): GX viewport / scissor
    /// coordinates are scaled by it (vertically; horizontally by `width / 640`, see `aspect`).
    pub scale: u32,
    /// The content aspect the target is shaped to ([`Renderer::set_target`]): 4:3 by default. Widescreen (the C
    /// build's AURORA_VIEWPORT_ASPECT policy) widens the target and maps the logical 640-wide GX viewport across it.
    pub aspect: f32,
    /// MSAA sample count of the EFB pass (`MELEE_MSAA`, 1 = off). `color` is always the single-sample (resolved)
    /// target that the EFB copies, the blit and the readback read.
    pub samples: u32,
    /// The logical EFB size GX coordinates are in (the constructor's `width` x `height`: 640x480 for the game).
    base: (u32, u32),
    color: wgpu::Texture,
    depth: wgpu::Texture,
    /// The multisampled colour target the EFB pass draws into when `samples > 1` (resolved into `color` at the end
    /// of every pass, so an EFB copy between passes reads the resolved image).
    msaa: Option<wgpu::Texture>,
    bgl_storage: wgpu::BindGroupLayout,
    bgl_uniform: wgpu::BindGroupLayout,
    bgl_textures: wgpu::BindGroupLayout,
    layout: wgpu::PipelineLayout,
    /// Pipelines by key, shared with the decode thread's warm-up ([`Renderer::pipe_factory`]).
    pipes: std::sync::Arc<PipeFactory>,
    tex_bgs: FxMap<[Option<(TexKey, SamplerKey)>; MAX_TEXTURES], wgpu::BindGroup>,
    /// Device lost / unexpected wgpu errors ([`Renderer::gpu_fault`]).
    gpu: std::sync::Arc<GpuHealth>,
    samplers: FxMap<SamplerKey, wgpu::Sampler>,
    textures: FxMap<TexKey, TexEntry>,
    /// `tex_override::generation()` the decoded textures were made under.
    ovr_gen: u64,
    /// [`Renderer::prepare`] calls so far (texture eviction clock).
    prepared: u64,
    /// A packing handed over by the decode thread for the next prepared frame ([`Renderer::set_packed`]).
    packed: Option<Packed>,
    empty_view: wgpu::TextureView,
    empty_sampler: wgpu::Sampler,
    /// WGSL sources that failed to compile, with naga's message.
    pub shader_errors: Vec<String>,
    blit: HashMap<wgpu::TextureFormat, (wgpu::RenderPipeline, wgpu::BindGroupLayout)>,
    blit_sampler: Option<wgpu::Sampler>,
    /// The HUD overlay's flat-colour quad pipeline per target format (`overlay_to`).
    overlay_pipes: HashMap<wgpu::TextureFormat, wgpu::RenderPipeline>,
    /// The last [`Renderer::prepare`]d frame: persistent GPU buffers, per-draw pipeline / bind group / offsets.
    frame: FrameCache,
    /// EFB copies (GXCopyTex) by guest destination address: textures later draws sample.
    efb_copies: FxMap<u64, CopyTarget>,
    copy_pipe: Option<(wgpu::RenderPipeline, wgpu::BindGroupLayout, wgpu::Sampler)>,
    /// The depth-format EFB copy (reads the depth target).
    zcopy_pipe: Option<(wgpu::RenderPipeline, wgpu::BindGroupLayout)>,
    /// The gd-ui-v1 batch (crate::ui): None until a UI mod hands over its first frame.
    pub(crate) ui: Option<crate::ui::UiGpu>,
}

/// The GPU texture an EFB copy writes (at the internal resolution: `dst_w * scale` x `dst_h * scale`).
struct CopyTarget {
    _tex: wgpu::Texture,
    view: wgpu::TextureView,
    dst_w: u32,
    dst_h: u32,
    scale: u32,
    /// The [`Renderer::prepare`] count of the last frame that copied into it.
    copied: u64,
}

/// A copy target nobody copied into for this many prepared frames is dropped (its guest address may hold an ordinary
/// texture by then).
const COPY_KEEP: u64 = 120;

/// A depth copy's mode for a GXSetTexCopyDst Z format (see `ZCOPY_WGSL`): the texel bytes of the 24-bit EFB depth the
/// destination format keeps, laid out as the Z texture formats are read back (Z24X8: R, G, B = bits 23-16, 15-8, 7-0;
/// Z16: A high, R low; the 8-bit ones in every channel).
fn zcopy_mode(fmt: u32) -> Option<u32> {
    Some(match fmt {
        0x11 => 0,        // GX_TF_Z8: bits 23-16
        0x13 => 1,        // GX_TF_Z16: bits 23-8
        0x16 => 2,        // GX_TF_Z24X8
        0x30 => 3,        // GX_CTF_Z4: bits 23-20
        0x39 => 4,        // GX_CTF_Z8M: bits 15-8
        0x3A => 5,        // GX_CTF_Z8L: bits 7-0
        0x3C => 6,        // GX_CTF_Z16L: bits 15-0
        _ => return None,
    })
}

const ZCOPY_WGSL: &str = "
struct P { rect: vec4<f32>, mode: u32, reversed: u32, p0: u32, p1: u32 };
var<push_constant> pc: P;
@group(0) @binding(0) var efbz: texture_depth_2d;
struct VsOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> VsOut {
    let uv = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    var o: VsOut;
    o.pos = vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, 0.0, 1.0);
    o.uv = uv;
    return o;
}
@fragment fn fs_main(v: VsOut) -> @location(0) vec4<f32> {
    let dims = vec2<f32>(textureDimensions(efbz));
    let p = clamp(vec2<i32>((pc.rect.xy + v.uv * pc.rect.zw) * dims), vec2<i32>(0), vec2<i32>(dims) - vec2<i32>(1));
    var d = textureLoad(efbz, p, 0);
    if (pc.reversed != 0u) { d = 1.0 - d; }
    let z = u32(clamp(round(d * 16777215.0), 0.0, 16777215.0));
    let hi = f32((z >> 16u) & 255u) / 255.0;
    let mid = f32((z >> 8u) & 255u) / 255.0;
    let lo = f32(z & 255u) / 255.0;
    switch (pc.mode) {
        case 0u: { return vec4<f32>(hi, hi, hi, hi); }
        case 1u: { return vec4<f32>(mid, mid, mid, hi); }
        case 2u: { return vec4<f32>(hi, mid, lo, 1.0); }
        case 3u: { let n = f32((z >> 20u) & 15u) / 15.0; return vec4<f32>(n, n, n, n); }
        case 4u: { return vec4<f32>(mid, mid, mid, mid); }
        case 5u: { return vec4<f32>(lo, lo, lo, lo); }
        default: { return vec4<f32>(lo, lo, lo, mid); }
    }
}
";

/// The copy shader's mode for a GXSetTexCopyDst colour format (see `COPY_WGSL`); None: not a colour format.
fn copy_mode(fmt: u32) -> Option<u32> {
    Some(match fmt {
        0x0 | 0x1 => 1,          // I4, I8: luma
        0x2 | 0x3 => 2,          // IA4, IA8: luma + alpha
        0x4 => 10,               // RGB565
        0x5 | 0x6 => 0,          // RGB5A3, RGBA8
        0x20 | 0x28 => 3,        // R4, R8
        0x22 | 0x23 => 4,        // RA4, RA8
        0x26 => 0,               // YUVA8
        0x27 => 5,               // A8
        0x29 => 6,               // G8
        0x2A => 7,               // B8
        0x2B => 8,               // RG8
        0x2C => 9,               // GB8
        _ => return None,
    })
}

const COPY_WGSL: &str = "
struct P { rect: vec4<f32>, mode: u32, efb_alpha: u32, p0: u32, p1: u32 };
var<push_constant> pc: P;
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
    var c = textureSampleLevel(efb, smp, pc.rect.xy + v.uv * pc.rect.zw, 0.0);
    if (pc.efb_alpha == 0u) { c.a = 1.0; }
    let y = clamp(0.257 * c.r + 0.504 * c.g + 0.098 * c.b + 16.0 / 255.0, 0.0, 1.0);
    switch (pc.mode) {
        case 1u: { return vec4<f32>(y, y, y, y); }
        case 2u: { return vec4<f32>(y, y, y, c.a); }
        case 3u: { return vec4<f32>(c.r, c.r, c.r, c.r); }
        case 4u: { return vec4<f32>(c.r, c.r, c.r, c.a); }
        case 5u: { return vec4<f32>(c.a, c.a, c.a, c.a); }
        case 6u: { return vec4<f32>(c.g, c.g, c.g, c.g); }
        case 7u: { return vec4<f32>(c.b, c.b, c.b, c.b); }
        case 8u: { return vec4<f32>(c.r, c.r, c.r, c.g); }
        case 9u: { return vec4<f32>(c.g, c.g, c.g, c.b); }
        case 10u: { return vec4<f32>(c.rgb, 1.0); }
        default: { return c; }
    }
}
";

/// One draw of the prepared frame.
struct PreparedDraw {
    pipeline: Option<wgpu::RenderPipeline>,
    tex_bg: Option<wgpu::BindGroup>,
    imm: [u8; 64],
    uoff: u32,
    first_index: u32,
    /// This draw owns its uniform block (false: it shares the previous draw's identical block).
    owner: bool,
    /// Indices (indexed) or vertices drawn: this draw's own plus those of the later draws merged into it.
    count: u32,
    /// Merged into an earlier draw of the same state (see [`pack`]): not recorded on its own.
    merged: bool,
}

/// One draw's packing (a [`PreparedDraw`] before its pipeline and texture lookups).
#[derive(Clone, Copy)]
pub struct PackedDraw {
    imm: [u8; 64],
    uoff: u32,
    first_index: u32,
    owner: bool,
    count: u32,
    merged: bool,
}

/// The CPU half of [`Renderer::prepare`] for one frame ([`pack`]): built on the GX decode thread so the main thread only
/// uploads, looks up pipelines and draws. `key` is the draw list's allocation and length.
pub struct Packed {
    key: (usize, usize),
    draws: Vec<PackedDraw>,
    vbuf: Vec<u8>,
    abuf: Vec<u8>,
    ibuf: Vec<u8>,
    uoff: u32,
    max_uniform: usize,
}

/// Pack every draw's vertices, array snapshots, indices and uniform layout (no GPU access: any thread).
pub fn pack(draws: &[DrawCmd]) -> Packed {
    let mut pk = Packed { key: (draws.as_ptr() as usize, draws.len()), draws: Vec::with_capacity(draws.len()), vbuf: Vec::new(), abuf: Vec::new(), ibuf: Vec::new(), uoff: 0, max_uniform: 16 };
    let mut arr_max: FxMap<usize, usize> = FxMap::default();
    let mut arr_off: FxMap<usize, u32> = FxMap::default();
    // Draws share array snapshots (a model's vertex arrays): upload each one once per frame.
    // Keyed by the snapshot's allocation; the upload covers the most any draw this frame needs.
    for a in draws.iter().flat_map(|d| d.arrays.iter().flatten()) {
        let n = (a.needed as usize).min(a.data.len());
        let e = arr_max.entry(a.data.as_ptr() as usize).or_insert(0);
        *e = (*e).max(n);
    }
    let mut prev_d: Option<&DrawCmd> = None;
    // The draw later draws may be merged into (index into pk.draws), its first vertex byte and the vertices it covers.
    let mut head: Option<(usize, u32, u32)> = None;
    let mut ibuf_end = 0usize; // ibuf length before the last draw's padding
    let batching = batching();
    for d in draws {
        // Batching: a draw that changes no state (same pipeline config, uniforms, matrix slot, arrays, textures,
        // viewport and scissor) and whose vertices follow the previous one's directly is drawn as part of it. Within
        // one draw call the primitives rasterise in order, so the output is the same as two back-to-back calls.
        if let (true, Some(p), Some((h, h_vtx, h_n))) = (batching, prev_d, head) {
            if can_merge(p, d) && pk.vbuf.len() as u32 == h_vtx + h_n * d.config.shader.vtx_stride as u32 {
                let indexed = !d.indices.is_empty();
                let fits = !indexed || d.indices.iter().all(|&i| (i as u32 + h_n) <= u16::MAX as u32);
                if fits {
                    pk.vbuf.extend_from_slice(&d.verts);
                    if indexed {
                        pk.ibuf.truncate(ibuf_end);
                        for &i in &d.indices {
                            pk.ibuf.extend_from_slice(&((i as u32 + h_n) as u16).to_le_bytes());
                        }
                        ibuf_end = pk.ibuf.len();
                        pk.ibuf.resize(pk.ibuf.len().next_multiple_of(4), 0);
                    }
                    let hd = &mut pk.draws[h];
                    hd.count += if indexed { d.indices.len() as u32 } else { d.vtx_count };
                    let last = pk.draws[h];
                    pk.draws.push(PackedDraw { imm: last.imm, uoff: last.uoff, first_index: last.first_index, owner: false, count: 0, merged: true });
                    head = Some((h, h_vtx, h_n + d.vtx_count));
                    prev_d = Some(d);
                    continue;
                }
            }
        }
        let mut imm = [0u32; 16];
        imm[0] = pk.vbuf.len() as u32;
        imm[1] = d.current_pn_mtx;
        pk.vbuf.extend_from_slice(&d.verts);
        pk.vbuf.resize(pk.vbuf.len().next_multiple_of(4), 0);
        for (i, a) in d.arrays.iter().enumerate() {
            if let Some(a) = a {
                let k = a.data.as_ptr() as usize;
                if let Some(&o) = arr_off.get(&k) {
                    imm[4 + i] = o;
                    continue;
                }
                let n = arr_max[&k];
                imm[4 + i] = pk.abuf.len() as u32;
                arr_off.insert(k, pk.abuf.len() as u32);
                pk.abuf.extend_from_slice(&a.data[..n]);
                pk.abuf.resize(pk.abuf.len().next_multiple_of(4) + 4, 0);
            }
        }
        // the PC matrix palette (Aurora: push_storage, `immediates._pad` = its abuf word offset)
        if let Some(p) = &d.palette {
            let k = std::sync::Arc::as_ptr(p) as usize;
            let o = match arr_off.get(&k) {
                Some(&o) => o,
                None => {
                    pk.abuf.resize(pk.abuf.len().next_multiple_of(4), 0);
                    let o = pk.abuf.len() as u32;
                    for x in p.iter() {
                        pk.abuf.extend_from_slice(&x.to_bits().to_le_bytes());
                    }
                    arr_off.insert(k, o);
                    o
                }
            };
            imm[3] = o / 4;
        }
        let first_index = (pk.ibuf.len() / 2) as u32;
        for i in &d.indices {
            pk.ibuf.extend_from_slice(&i.to_le_bytes());
        }
        ibuf_end = pk.ibuf.len();
        pk.ibuf.resize(pk.ibuf.len().next_multiple_of(4), 0);
        let mut immb = [0u8; 64];
        for (k, w) in imm.iter().enumerate() {
            immb[k * 4..k * 4 + 4].copy_from_slice(&w.to_le_bytes());
        }
        // Consecutive draws of one joint mostly carry identical uniform blocks: share one (same matrix slot and
        // skinning, so an interpolating `fill` of the owner covers them all).
        let share = prev_d.is_some_and(|p: &DrawCmd| {
            p.current_pn_mtx == d.current_pn_mtx && p.config.shader.attrs[0] == d.config.shader.attrs[0] && p.uniform == d.uniform
        });
        let count = if d.indices.is_empty() { d.vtx_count } else { d.indices.len() as u32 };
        // a head for later merges: plain-stride vertices (nothing padded after them)
        head = (d.copy.is_none() && d.config.shader.vtx_stride != 0 && d.verts.len() as u32 == d.vtx_count * d.config.shader.vtx_stride as u32)
            .then_some((pk.draws.len(), imm[0], d.vtx_count));
        if share {
            let u = pk.draws.last().unwrap().uoff;
            pk.draws.push(PackedDraw { imm: immb, uoff: u, first_index, owner: false, count, merged: false });
        } else {
            pk.draws.push(PackedDraw { imm: immb, uoff: pk.uoff, first_index, owner: true, count, merged: false });
            pk.uoff += (d.uniform.len() as u32).next_multiple_of(UNIFORM_ALIGN as u32);
            pk.max_uniform = pk.max_uniform.max(d.uniform.len());
        }
        prev_d = Some(d);
    }
    pk
}

/// `MELEE_GX_NOBATCH=1` turns the draw merging of [`pack`] off (A/B screenshots).
fn batching() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("MELEE_GX_NOBATCH").is_none())
}

/// `d` draws with exactly `p`'s state, so it can be appended to `p`'s draw call (see [`pack`]).
fn can_merge(p: &DrawCmd, d: &DrawCmd) -> bool {
    if p.copy.is_some() || d.copy.is_some() || p.instance_count != 1 || d.instance_count != 1 || p.info.line_mode != 0 || d.info.line_mode != 0 {
        return false;
    }
    if p.indices.is_empty() != d.indices.is_empty() || p.fmt != d.fmt || p.current_pn_mtx != d.current_pn_mtx {
        return false;
    }
    // no merge across palettes (Aurora: a palette load dirties the immediates)
    match (&p.palette, &d.palette) {
        (None, None) => {}
        (Some(a), Some(b)) if std::sync::Arc::ptr_eq(a, b) => {}
        _ => return false,
    }
    if !(std::sync::Arc::ptr_eq(&p.config, &d.config) || *p.config == *d.config) || p.config.shader.vtx_stride == 0 {
        return false;
    }
    if d.verts.len() as u32 != d.vtx_count * d.config.shader.vtx_stride as u32 {
        return false;
    }
    if p.render_viewport != d.render_viewport || p.render_scissor != d.render_scissor || p.uniform != d.uniform {
        return false;
    }
    if p.info.sampled_textures != d.info.sampled_textures || p.info.sampled_ind_textures != d.info.sampled_ind_textures {
        return false;
    }
    let same_arr = p.arrays.iter().zip(d.arrays.iter()).all(|(a, b)| match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => a.data.as_ptr() == b.data.as_ptr(),
        _ => false,
    });
    if !same_arr {
        return false;
    }
    let ptr = |x: &Option<crate::resources::GxData>| x.as_ref().map(|v| (v.as_ptr() as usize, v.len()));
    let used = p.info.sampled_textures | p.info.sampled_ind_textures;
    for m in 0..MAX_TEXTURES {
        if used & (1 << m) == 0 {
            continue;
        }
        let (a, b) = (&p.textures[m], &d.textures[m]);
        if a.mode0 != b.mode0 || a.mode1 != b.mode1 || a.image0 != b.image0 || a.image3 != b.image3 || a.width != b.width || a.height != b.height
            || a.format != b.format || a.tlut != b.tlut || a.guest != b.guest || a.flags != b.flags || ptr(&a.data) != ptr(&b.data)
        {
            return false;
        }
    }
    p.tluts.len() == d.tluts.len()
        && p.tluts.iter().zip(d.tluts.iter()).all(|(a, b)| a.format == b.format && a.num_entries == b.num_entries && ptr(&a.data) == ptr(&b.data))
}

/// GPU error routing. wgpu's error scopes are one stack per DEVICE, not per thread: the GX decode thread's
/// pipeline warm-up used to push a Validation scope, so an error raised meanwhile on the main thread (a failed
/// `create_texture`) was swallowed into that scope, and the next use of the invalid texture panicked in wgpu's
/// default handler ("Texture with 'GX texture' label is invalid", issue #42). The wgpu-core backend calls the
/// uncaptured-error handler synchronously on the thread that made the failing call, so [`guarded`] collects the
/// errors of one thread's calls exactly; every other error is recorded in [`GpuHealth`] (a clean exit, not a panic).
pub struct GpuHealth {
    fault: std::sync::Mutex<Option<String>>,
    lost: std::sync::atomic::AtomicBool,
}

impl GpuHealth {
    fn set_fault(&self, msg: String) {
        let mut f = self.fault.lock().unwrap();
        if f.is_none() {
            eprintln!("gx: GPU error: {msg}");
            *f = Some(msg);
        }
    }
}

thread_local! {
    static GUARD: std::cell::RefCell<Option<Vec<wgpu::Error>>> = const { std::cell::RefCell::new(None) };
}

/// Run `f` collecting the wgpu errors its calls raise on this thread (instead of reporting them as faults).
fn guarded<T>(f: impl FnOnce() -> T) -> (T, Vec<wgpu::Error>) {
    let outer = GUARD.with(|g| g.borrow_mut().replace(Vec::new()));
    let r = f();
    let errs = GUARD.with(|g| std::mem::replace(&mut *g.borrow_mut(), outer)).unwrap_or_default();
    (r, errs)
}

fn install_error_handling(device: &wgpu::Device) -> std::sync::Arc<GpuHealth> {
    let h = std::sync::Arc::new(GpuHealth { fault: Default::default(), lost: Default::default() });
    let hl = h.clone();
    device.set_device_lost_callback(move |reason, msg| {
        hl.lost.store(true, std::sync::atomic::Ordering::SeqCst);
        if reason != wgpu::DeviceLostReason::Destroyed {
            hl.set_fault(format!("GPU device lost ({reason:?}): {msg} (driver reset or out of video memory)"));
        }
    });
    let he = h.clone();
    device.on_uncaptured_error(std::sync::Arc::new(move |e: wgpu::Error| {
        let Some(e) = GUARD.with(|g| match g.borrow_mut().as_mut() {
            Some(v) => {
                v.push(e);
                None
            }
            None => Some(e),
        }) else {
            return;
        };
        if he.lost.load(std::sync::atomic::Ordering::SeqCst) {
            return; // consequences of the loss, already reported
        }
        let msg = match &e {
            wgpu::Error::OutOfMemory { .. } => format!("out of video memory: {e}"),
            _ => format!("{e}"),
        };
        he.set_fault(msg);
    }));
    h
}

/// The render pipelines by key (every config's WGSL is built once). Shared by the renderer and the GX decode thread:
/// a pipeline is a function of its key alone, so which thread creates it does not change what is drawn.
pub struct PipeFactory {
    device: wgpu::Device,
    layout: wgpu::PipelineLayout,
    samples: u32,
    pipelines: std::sync::Mutex<FxMap<PipelineKey, wgpu::RenderPipeline>>,
    wgsl_hashes: std::sync::Mutex<FxMap<crate::shader::ShaderConfig, u64>>,
}

impl PipeFactory {
    /// Create the pipelines `draws` need that do not exist yet (the decode thread, before handing the frame over).
    pub fn warm(&self, draws: &[DrawCmd]) {
        let mut last: Option<&std::sync::Arc<crate::shader::PipelineConfig>> = None;
        for d in draws {
            if d.copy.is_some() || d.config.cull_mode == GX_CULL_ALL || last.is_some_and(|l| std::sync::Arc::ptr_eq(l, &d.config)) {
                continue;
            }
            last = Some(&d.config);
            let _ = self.get(&d.config);
        }
    }

    pub fn len(&self) -> usize {
        self.pipelines.lock().unwrap().len()
    }

    /// The pipeline of `pc` (created and cached on a miss; Err: the WGSL failed validation, with the source).
    pub fn get(&self, pc: &crate::shader::PipelineConfig) -> Result<wgpu::RenderPipeline, String> {
        // Generating the WGSL per draw per frame dominated render time: memoise its hash by config.
        let known = self.wgsl_hashes.lock().unwrap().get(&pc.shader).copied();
        let wgsl_hash = match known {
            Some(h) => h,
            None => {
                let h = fnv(crate::shader::build_shader_source(&pc.shader, u32::MAX).as_bytes());
                self.wgsl_hashes.lock().unwrap().insert(pc.shader.clone(), h);
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
        if let Some(p) = self.pipelines.lock().unwrap().get(&key) {
            return Ok(p.clone());
        }
        let src = crate::shader::build_shader_source(&pc.shader, u32::MAX);
        let wgsl = adapt_wgsl(&src);
        let (pipeline, errs) = guarded(|| {
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
        self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
            multisample: wgpu::MultisampleState { count: self.samples, ..Default::default() },
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
        })
        });
        if let Some(e) = errs.first() {
            return Err(format!("{e}\n--- WGSL ---\n{wgsl}"));
        }
        self.pipelines.lock().unwrap().insert(key, pipeline.clone());
        Ok(pipeline)
    }
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
    last_pipe: Option<(std::sync::Arc<crate::shader::PipelineConfig>, Option<wgpu::RenderPipeline>)>,
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

    /// Largest internal scale (`MELEE_RENDER_SCALE`); the target is further capped by the device's texture limit.
    pub const MAX_SCALE: u32 = 8;

    /// `MELEE_RENDER_SCALE` (1..=8), when set to a number (`auto` or unset: follow the window).
    pub fn env_scale() -> Option<u32> {
        std::env::var("MELEE_RENDER_SCALE").ok().and_then(|v| v.trim().parse::<u32>().ok()).filter(|&s| s > 0).map(|s| s.clamp(1, Self::MAX_SCALE))
    }

    /// `MELEE_MSAA` (1, 2, 4 or 8, an optional trailing `x`; other counts round down to one of them), default 1 (off).
    pub fn env_msaa() -> u32 {
        let n = std::env::var("MELEE_MSAA").ok().and_then(|v| v.trim().trim_end_matches(['x', 'X']).parse::<u32>().ok()).unwrap_or(1);
        match n {
            0 | 1 => 1,
            2 | 3 => 2,
            4..=7 => 4,
            _ => 8,
        }
    }

    /// The internal scale that renders a 640x480 EFB at (at least) a `w` x `h` window's native resolution.
    pub fn scale_for_window(w: u32, h: u32) -> u32 {
        Self::scale_for_window_aspect(w, h, 4.0 / 3.0)
    }

    /// [`Renderer::scale_for_window`] for a target of content aspect `aspect` (480 logical lines, `480 * aspect` wide).
    pub fn scale_for_window_aspect(w: u32, h: u32, aspect: f32) -> u32 {
        let lw = (480.0 * aspect.max(0.1)).round().max(1.0) as u32;
        (w.div_ceil(lw)).max(h.div_ceil(480)).clamp(1, 4)
    }

    /// Target size in pixels for a logical `base` EFB at `scale` and content aspect `aspect`: `base.1 * scale` lines and
    /// `round(lines * aspect)` columns (exactly `base.0 * scale` at the base's own aspect), shrunk to fit `max` (the
    /// device's 2D texture limit).
    pub fn target_size(base: (u32, u32), scale: u32, aspect: f32, max: u32) -> (u32, u32) {
        let h = base.1 * scale;
        let native = base.0 as f32 / base.1.max(1) as f32;
        let w = if (aspect - native).abs() < 1e-6 { base.0 * scale } else { (h as f64 * aspect as f64).round() as u32 };
        if w <= max && h <= max {
            return (w.max(1), h);
        }
        let k = (max as f64 / w as f64).min(max as f64 / h as f64);
        (((w as f64 * k).floor() as u32).max(1), ((h as f64 * k).floor() as u32).max(1))
    }

    /// Change the internal resolution multiplier: recreates the colour / depth targets (contents are lost).
    pub fn set_scale(&mut self, scale: u32) {
        self.set_target(scale, self.aspect);
    }

    /// Change the scale and the content aspect (widescreen): recreates the targets when either changed. Returns
    /// true when they were recreated (their contents are lost).
    pub fn set_target(&mut self, scale: u32, aspect: f32) -> bool {
        let scale = scale.clamp(1, Self::MAX_SCALE);
        let aspect = if aspect.is_finite() && aspect > 0.0 { aspect } else { 4.0 / 3.0 };
        if scale == self.scale && aspect.to_bits() == self.aspect.to_bits() {
            return false;
        }
        let (w, h) = Self::target_size(self.base, scale, aspect, self.device.limits().max_texture_dimension_2d);
        let (color, depth, msaa) = make_targets(&self.device, w, h, self.samples);
        self.color = color;
        self.depth = depth;
        self.msaa = msaa;
        self.width = w;
        self.height = h;
        self.scale = scale;
        self.aspect = aspect;
        true
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
        let base = (width, height);
        let aspect = width as f32 / height.max(1) as f32;
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
            // the adapter's own format features (MSAA counts past WebGPU's guaranteed 1 / 4) when it has them
            required_features: wgpu::Features::PUSH_CONSTANTS | (adapter.features() & wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES),
            required_limits: limits,
            ..Default::default()
        }))
        .map_err(|e| format!("request_device: {e}"))?;
        let gpu = install_error_handling(&device);
        // MSAA: the largest count not above the request that every EFB attachment format supports on this device
        // (without TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES the device validates against WebGPU's guaranteed counts,
        // not the adapter's: an adapter-reported 8x then fails texture creation, "Sample count 8 is not supported").
        let want = Self::env_msaa();
        let mask = supported_msaa_mask(&adapter, device.features());
        SUPPORTED_MSAA.store(mask, std::sync::atomic::Ordering::Relaxed);
        let samples = clamp_msaa(want, mask);
        if samples != want {
            eprintln!("gx: MSAA {want}x not supported by {} for the EFB formats ({COLOR_FORMAT:?} / {DEPTH_FORMAT:?}): using {samples}x", adapter_info.name);
        }
        let (width, height) = Self::target_size(base, scale, aspect, device.limits().max_texture_dimension_2d);
        let (color, depth, msaa) = make_targets(&device, width, height, samples);
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
        let pipes = std::sync::Arc::new(PipeFactory { device: device.clone(), layout: layout.clone(), samples, pipelines: Default::default(), wgsl_hashes: Default::default() });
        Ok(Self {
            adapter,
            last_fill_us: 0,
            device,
            queue,
            gpu,
            adapter_info,
            width,
            height,
            scale,
            aspect,
            samples,
            base,
            color,
            depth,
            msaa,
            bgl_storage,
            bgl_uniform,
            bgl_textures,
            layout,
            pipes,
            tex_bgs: FxMap::default(),
            samplers: FxMap::default(),
            textures: FxMap::default(),
            ovr_gen: crate::tex_override::generation(),
            prepared: 0,
            packed: None,
            empty_view,
            empty_sampler,
            shader_errors: Vec::new(),
            blit: HashMap::new(),
            blit_sampler: None,
            overlay_pipes: HashMap::new(),
            frame: FrameCache::default(),
            efb_copies: FxMap::default(),
            copy_pipe: None,
            zcopy_pipe: None,
            ui: None,
        })
    }

    /// A fatal GPU error (device lost, out of video memory, an unexpected wgpu error): the caller should stop
    /// with this message. Once set, frames are no longer prepared or drawn.
    pub fn gpu_fault(&self) -> Option<String> {
        self.gpu.fault.lock().unwrap().clone()
    }

    fn pipeline(&mut self, d: &DrawCmd) -> Option<wgpu::RenderPipeline> {
        match self.pipes.get(&d.config) {
            Ok(p) => Some(p),
            Err(e) => {
                self.shader_errors.push(e);
                None
            }
        }
    }

    /// The pipeline cache, for [`crate::api::Gx::set_pipe_factory`]: the decode thread creates a frame's new pipelines
    /// before it hands the frame over, so the main thread's prepare finds them made.
    pub fn pipe_factory(&self) -> std::sync::Arc<PipeFactory> {
        self.pipes.clone()
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
        let e = self.textures.get(&key)?;
        e.used.set(self.prepared);
        Some(key)
    }

    /// The EFB copy `o` samples: its image address is a copy destination of the same size.
    fn copy_view(&self, o: &TexObj) -> Option<wgpu::TextureView> {
        if o.guest == 0 {
            return None;
        }
        let t = self.efb_copies.get(&o.guest)?;
        (t.dst_w == o.width() && t.dst_h == o.height()).then(|| t.view.clone())
    }

    fn ensure_copy_target(&mut self, c: &crate::draw::EfbCopy) {
        let now = self.prepared;
        if let Some(t) = self.efb_copies.get_mut(&c.dest) {
            if t.dst_w == c.dst_w && t.dst_h == c.dst_h && t.scale == self.scale {
                t.copied = now;
                return;
            }
        }
        if c.dst_w == 0 || c.dst_h == 0 {
            return;
        }
        let (w, h) = ((c.dst_w * self.scale).min(8192), (c.dst_h * self.scale).min(8192));
        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("GX EFB copy"),
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = tex.create_view(&Default::default());
        // cached bind groups may hold the replaced target's view
        self.tex_bgs.clear();
        self.efb_copies.insert(c.dest, CopyTarget { _tex: tex, view, dst_w: c.dst_w, dst_h: c.dst_h, scale: self.scale, copied: now });
    }

    fn copy_pipeline(&mut self) {
        if self.copy_pipe.is_some() {
            return;
        }
        let d = &self.device;
        let module = d.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("GX EFB copy"), source: wgpu::ShaderSource::Wgsl(COPY_WGSL.into()) });
        let bgl = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("GX EFB copy"),
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
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
            ],
        });
        let layout = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("GX EFB copy"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[wgpu::PushConstantRange { stages: wgpu::ShaderStages::FRAGMENT, range: 0..32 }],
        });
        let pipe = d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("GX EFB copy"),
            layout: Some(&layout),
            vertex: wgpu::VertexState { module: &module, entry_point: Some("vs_main"), compilation_options: Default::default(), buffers: &[] },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format: wgpu::TextureFormat::Rgba8Unorm, blend: None, write_mask: wgpu::ColorWrites::ALL })],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });
        let smp = d.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("GX EFB copy"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        self.copy_pipe = Some((pipe, bgl, smp));
        // with MSAA the depth target is multisampled: the copy reads sample 0 (textureLoad's last argument)
        let ms = self.samples > 1;
        let src = if ms { ZCOPY_WGSL.replace("texture_depth_2d", "texture_depth_multisampled_2d") } else { ZCOPY_WGSL.to_string() };
        let module = d.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("GX EFB depth copy"), source: wgpu::ShaderSource::Wgsl(src.into()) });
        let bgl = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("GX EFB depth copy"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Depth, view_dimension: wgpu::TextureViewDimension::D2, multisampled: ms },
                count: None,
            }],
        });
        let layout = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("GX EFB depth copy"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[wgpu::PushConstantRange { stages: wgpu::ShaderStages::FRAGMENT, range: 0..32 }],
        });
        let pipe = d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("GX EFB depth copy"),
            layout: Some(&layout),
            vertex: wgpu::VertexState { module: &module, entry_point: Some("vs_main"), compilation_options: Default::default(), buffers: &[] },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format: wgpu::TextureFormat::Rgba8Unorm, blend: None, write_mask: wgpu::ColorWrites::ALL })],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });
        self.zcopy_pipe = Some((pipe, bgl));
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
            e.used.set(self.prepared);
            return Some(e.view.clone());
        }
        if w == 0 || h == 0 {
            return None;
        }
        // a registered override (cosmetics: a skin's portrait / stock icon) is uploaded instead, at its own size
        let ovr = if crate::tex_override::active() {
            let tl = if pal { tlut_data.as_deref().map(|t| &t[..(d.tluts[o.tlut as usize].num_entries as usize * 2).min(t.len())]) } else { None };
            crate::tex_override::lookup(&crate::tex_override::key(fmt, w, h, data, tl))
        } else {
            None
        };
        let (w, h, mips) = ovr.as_ref().map_or((w, h, mips), |p| (p.w, p.h, 1));
        let conv_data;
        let conv: &[u8] = if let Some(p) = ovr.as_ref() {
            &p.rgba
        } else {
            conv_data = if matches!(fmt, GX_TF_C4 | GX_TF_C8 | GX_TF_C14X2) {
                let t = &d.tluts[o.tlut as usize];
                texture::convert_texture_palette(fmt, w, h, mips, data, t.format, t.num_entries, t.data.as_deref()?)
            } else {
                texture::convert_texture(fmt, w, h, mips, data)
            };
            &conv_data.data
        };
        let desc = wgpu::TextureDescriptor {
            label: Some("GX texture"),
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: mips,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        };
        // Out of video memory: drop every cached texture (re-decoded on use) and try once more; a texture that
        // still fails is drawn as the empty one, and an error that is not memory is a fault.
        let (mut tex, mut errs) = guarded(|| self.device.create_texture(&desc));
        if !errs.is_empty() && errs.iter().all(|e| matches!(e, wgpu::Error::OutOfMemory { .. })) {
            eprintln!("gx: out of video memory creating a {w}x{h} texture ({} cached): evicting the texture cache", self.textures.len());
            self.tex_bgs.clear();
            self.textures.clear();
            (tex, errs) = guarded(|| self.device.create_texture(&desc));
            if errs.iter().all(|e| matches!(e, wgpu::Error::OutOfMemory { .. })) && !errs.is_empty() {
                eprintln!("gx: out of video memory again: drawing it untextured");
                return None;
            }
        }
        if let Some(e) = errs.first() {
            self.gpu.set_fault(format!("create_texture {w}x{h} mips {mips}: {e}"));
            return None;
        }
        let (mut mw, mut mh) = (w, h);
        let mut off = 0usize;
        for mip in 0..mips {
            let n = (mw * mh * 4) as usize;
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo { texture: &tex, mip_level: mip, origin: wgpu::Origin3d::ZERO, aspect: Default::default() },
                &conv[off..off + n],
                wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(mw * 4), rows_per_image: Some(mh) },
                wgpu::Extent3d { width: mw, height: mh, depth_or_array_layers: 1 },
            );
            off += n;
            mw = (mw >> 1).max(1);
            mh = (mh >> 1).max(1);
        }
        let view = tex.create_view(&Default::default());
        self.textures.insert(key, TexEntry { _tex: tex, view: view.clone(), _data: data.clone(), _tlut: tlut_data, used: std::cell::Cell::new(self.prepared) });
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
    /// Use `p` when the next [`Renderer::prepare`] gets the draws it was packed from.
    pub fn set_packed(&mut self, p: Packed) {
        self.packed = Some(p);
    }

    pub fn prepare(&mut self, _state: &GxState, draws: &[DrawCmd]) {
        if self.gpu_fault().is_some() {
            return;
        }
        let t0 = std::time::Instant::now();
        self.prepared += 1;
        frame_stats(self.prepared, draws);
        let g = crate::tex_override::generation();
        if g != self.ovr_gen {
            // the override set changed: decode every texture again (a replaced one back to the game's, or anew)
            self.ovr_gen = g;
            self.tex_bgs.clear();
            self.textures.clear();
        }
        if self.prepared % TEX_KEEP == 0 {
            let now = self.prepared;
            let before = self.textures.len();
            self.textures.retain(|_, e| now - e.used.get() < TEX_KEEP);
            if self.textures.len() != before {
                self.tex_bgs.clear();
            }
        }
        if self.prepared % TEX_KEEP == 0 && std::env::var_os("MELEE_GX_CACHE_LOG").is_some() {
            let f = &self.frame;
            eprintln!("gx cache: prepared {} textures {} tex_bgs {} samplers {} pipelines {} efb_copies {} wgsl {} frame vecs {} {} {} {} draws {}", self.prepared, self.textures.len(), self.tex_bgs.len(), self.samplers.len(), self.pipes.len(), self.efb_copies.len(), self.pipes.wgsl_hashes.lock().unwrap().len(), f.vbuf.capacity(), f.abuf.capacity(), f.ibuf.capacity(), f.ubuf.capacity(), f.draws.capacity());
        }
        let mut fc = std::mem::take(&mut self.frame);
        // the CPU packing, done on the GX decode thread when it handed one over for exactly these draws
        let key = (draws.as_ptr() as usize, draws.len());
        let pk = match self.packed.take() {
            Some(p) if p.key == key => p,
            _ => pack(draws),
        };
        fc.vbuf = pk.vbuf;
        fc.abuf = pk.abuf;
        fc.ibuf = pk.ibuf;
        fc.draws.clear();
        fc.draws.extend(pk.draws.iter().map(|c| PreparedDraw { pipeline: None, tex_bg: None, imm: c.imm, uoff: c.uoff, first_index: c.first_index, owner: c.owner, count: c.count, merged: c.merged }));
        let uoff = pk.uoff;
        let max_uniform = pk.max_uniform;
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
        // EFB copy targets first: a draw may sample a copy made earlier in this frame (or in an earlier one).
        let now = self.prepared;
        // A copy's bytes stay in guest memory until overwritten (the results screen copies the winner's portrait once and
        // samples it every frame after): a target this frame's draws sample is kept as if copied now.
        if !self.efb_copies.is_empty() {
            for d in draws.iter().filter(|d| d.copy.is_none()) {
                for (i, o) in d.textures.iter().enumerate() {
                    if (d.info.sampled_textures | d.info.sampled_ind_textures) & (1 << i) == 0 || o.guest == 0 {
                        continue;
                    }
                    if let Some(t) = self.efb_copies.get_mut(&o.guest) {
                        if t.dst_w == o.width() && t.dst_h == o.height() {
                            t.copied = now;
                        }
                    }
                }
            }
        }
        self.efb_copies.retain(|_, t| now - t.copied < COPY_KEEP);
        for c in draws.iter().filter_map(|d| d.copy.as_deref()) {
            self.ensure_copy_target(c);
        }
        let mut last_pipe = fc.last_pipe.take();
        let (mut miss_pipe, mut miss_tex, mut n_pipe, mut n_tex) = (std::time::Duration::ZERO, std::time::Duration::ZERO, 0usize, 0usize);
        for (i, d) in draws.iter().enumerate() {
            if d.copy.is_some() || fc.draws[i].merged {
                fc.draws[i].pipeline = None;
                fc.draws[i].tex_bg = None;
                continue;
            }
            let p = if d.config.cull_mode == GX_CULL_ALL {
                None
            } else {
                match last_pipe.as_ref() {
                    Some((cfg, p)) if std::sync::Arc::ptr_eq(cfg, &d.config) || **cfg == *d.config => p.clone(),
                    _ => {
                        let (n0, tp) = (self.pipes.len(), std::time::Instant::now());
                        let p = self.pipeline(d);
                        if self.pipes.len() != n0 {
                            miss_pipe += tp.elapsed();
                            n_pipe += 1;
                        }
                        last_pipe = Some((d.config.clone(), p.clone()));
                        p
                    }
                }
            };
            fc.draws[i].pipeline = p;
            let (n0, tt) = (self.textures.len(), std::time::Instant::now());
            fc.draws[i].tex_bg = Some(self.tex_bind_group(d));
            if self.textures.len() != n0 {
                miss_tex += tt.elapsed();
                n_tex += self.textures.len() - n0;
            }
        }
        fc.last_pipe = last_pipe;
        self.frame = fc;
        if prof() {
            eprintln!(
                "gxprof prepare n={} calls={} pack={:?} upload={:?} lookup={:?} vb={} ab={} ub={} newpipe={} pipe={:?} newtex={} tex={:?}",
                draws.len(),
                self.frame.draws.iter().filter(|d| !d.merged && d.pipeline.is_some()).count(),
                t1 - t0,
                t2 - t1,
                t2.elapsed(),
                self.frame.vbuf.len(),
                self.frame.abuf.len(),
                self.frame.ubuf.len(),
                n_pipe,
                miss_pipe,
                n_tex,
                miss_tex
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
            if used(i) && self.copy_view(&d.textures[i]).is_some() {
                cacheable = false;
            } else if used(i) {
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
            let v = if !used(i) {
                None
            } else if let Some(v) = self.copy_view(&d.textures[i]) {
                Some(v)
            } else {
                self.texture_view(d, i)
            };
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
        if self.gpu_fault().is_some() {
            return;
        }
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
            let tf = std::time::Instant::now();
            for (i, d) in draws.iter().enumerate() {
                if fc.draws[i].owner {
                    let o = fc.draws[i].uoff as usize;
                    f(i, &mut fc.ubuf[o..o + d.uniform.len()]);
                }
            }
            fc.ubuf_state = 2;
            self.last_fill_us = tf.elapsed().as_micros() as u32;
        }
        if let Some(g) = fc.gu.as_ref() {
            self.queue.write_buffer(&g.buf, 0, &fc.ubuf);
        }
        let t1 = std::time::Instant::now();
        if draws.iter().any(|d| d.copy.is_some()) {
            self.copy_pipeline();
        }
        let color_view = self.color.create_view(&Default::default());
        let depth_view = self.depth.create_view(&Default::default());
        let msaa_view = self.msaa.as_ref().map(|t| t.create_view(&Default::default()));
        let mut enc = self.device.create_command_encoder(&Default::default());
        let fc = &self.frame;
        let dims = (self.width, self.height, self.width as f32 / self.base.0 as f32, self.height as f32 / self.base.1 as f32);
        // One EFB pass per run of draws between EFB copies; a copy with `clear` starts the next pass cleared.
        let cc = state.clear_color;
        let mut clear = Some((wgpu::Color { r: cc[0] as f64, g: cc[1] as f64, b: cc[2] as f64, a: cc[3] as f64 }, state.clear_depth_value()));
        let mut start = 0;
        loop {
            let end = draws[start..].iter().position(|d| d.copy.is_some()).map_or(draws.len(), |k| start + k);
            {
                let (cload, dload) = match clear.take() {
                    Some((c, z)) => (wgpu::LoadOp::Clear(c), wgpu::LoadOp::Clear(z)),
                    None => (wgpu::LoadOp::Load, wgpu::LoadOp::Load),
                };
                let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("GX EFB"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: msaa_view.as_ref().unwrap_or(&color_view),
                        resolve_target: msaa_view.as_ref().map(|_| &color_view),
                        depth_slice: None,
                        ops: wgpu::Operations { load: cload, store: wgpu::StoreOp::Store },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &depth_view,
                        depth_ops: Some(wgpu::Operations { load: dload, store: wgpu::StoreOp::Store }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                record_draws(&mut pass, fc, draws, start..end, dims);
            }
            if end == draws.len() {
                break;
            }
            let c = draws[end].copy.as_deref().unwrap();
            // a Z-format copy (GXSetTexCopyDst GX_TF_Z*/GX_CTF_Z*) reads the depth target, as the hardware copies the
            // EFB's z buffer for those formats (gm_1832's costume snapshots: GX_TF_Z24X8 next to the colour copy)
            if let (Some(t), Some(mode), Some((pipe, bgl))) = (self.efb_copies.get(&c.dest), zcopy_mode(c.fmt), self.zcopy_pipe.as_ref()) {
                let zview = self.depth.create_view(&wgpu::TextureViewDescriptor { aspect: wgpu::TextureAspect::DepthOnly, ..Default::default() });
                let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("GX EFB depth copy"),
                    layout: bgl,
                    entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&zview) }],
                });
                let (ew, eh) = (self.base.0 as f32, self.base.1 as f32); // copy rectangles are in logical EFB pixels
                let r = [c.src[0] as f32 / ew, c.src[1] as f32 / eh, c.src[2] as f32 / ew, c.src[3] as f32 / eh];
                let mut pc = [0u8; 32];
                for (k, v) in r.iter().enumerate() {
                    pc[k * 4..k * 4 + 4].copy_from_slice(&v.to_le_bytes());
                }
                pc[16..20].copy_from_slice(&mode.to_le_bytes());
                pc[20..24].copy_from_slice(&(crate::state::USE_REVERSED_Z as u32).to_le_bytes());
                let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("GX EFB depth copy"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &t.view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                pass.set_pipeline(pipe);
                pass.set_bind_group(0, &bg, &[]);
                pass.set_push_constants(wgpu::ShaderStages::FRAGMENT, 0, &pc);
                pass.draw(0..3, 0..1);
            } else if let (Some(t), Some(mode), Some((pipe, bgl, smp))) = (self.efb_copies.get(&c.dest), copy_mode(c.fmt), self.copy_pipe.as_ref()) {
                let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("GX EFB copy"),
                    layout: bgl,
                    entries: &[
                        wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&color_view) },
                        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(smp) },
                    ],
                });
                let (ew, eh) = (self.base.0 as f32, self.base.1 as f32); // copy rectangles are in logical EFB pixels
                let r = [c.src[0] as f32 / ew, c.src[1] as f32 / eh, c.src[2] as f32 / ew, c.src[3] as f32 / eh];
                let mut pc = [0u8; 32];
                for (k, v) in r.iter().enumerate() {
                    pc[k * 4..k * 4 + 4].copy_from_slice(&v.to_le_bytes());
                }
                pc[16..20].copy_from_slice(&mode.to_le_bytes());
                pc[20..24].copy_from_slice(&(c.efb_alpha as u32).to_le_bytes());
                let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("GX EFB copy"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &t.view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                pass.set_pipeline(pipe);
                pass.set_bind_group(0, &bg, &[]);
                pass.set_push_constants(wgpu::ShaderStages::FRAGMENT, 0, &pc);
                pass.draw(0..3, 0..1);
            }
            if c.clear {
                let cc = c.clear_color;
                clear = Some((wgpu::Color { r: cc[0] as f64, g: cc[1] as f64, b: cc[2] as f64, a: cc[3] as f64 }, c.clear_depth));
            }
            start = end + 1;
        }
        self.queue.submit([enc.finish()]);
        self.dump_copies();
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
            let mut vp = [0.0, 0.0, tw, th];
            if !stretch {
                // Fit the EFB's aspect (640x480 times the render scale) inside the target.
                let aspect = self.width as f32 / self.height as f32;
                let (w, h) = if tw / th > aspect { (th * aspect, th) } else { (tw, tw / aspect) };
                vp = [((tw - w) * 0.5).floor(), ((th - h) * 0.5).floor(), w, h];
                pass.set_viewport(vp[0], vp[1], w, h, 0.0, 1.0);
            }
            if !self.ui_hides_gx() {
                pass.set_pipeline(pipeline);
                pass.set_bind_group(0, &bg, &[]);
                pass.draw(0..3, 0..1);
            }
            if self.ui.is_some() {
                // gd-ui-v1: the UI batch inside this same pass (one instanced draw). The band is the 4:3 area centred in the blit rect.
                let bw = (vp[3] * 4.0 / 3.0).min(vp[2]);
                let band = [vp[0] + (vp[2] - bw) * 0.5, vp[1], bw, vp[3]];
                self.ui_record(&mut pass, format, tw, th, band);
            }
        }
        self.queue.submit([enc.finish()]);
    }


    /// Draw `quads` (target pixels: x, y, w, h, RGBA8 straight alpha) alpha-blended over `target`, after
    /// [`Renderer::blit_to`]. The HUD overlay (fps, ping, rollbacks): it never touches the EFB, the GX state or any
    /// draw list, so the game's frames, their hashes and the EFB copies the game reads are unchanged.
    pub fn overlay_to(&mut self, target: &wgpu::TextureView, format: wgpu::TextureFormat, tw: u32, th: u32, quads: &[OverlayQuad]) {
        const OVERLAY_WGSL: &str = "
struct VsOut { @builtin(position) pos: vec4<f32>, @location(0) c: vec4<f32> };
@vertex fn vs_main(@location(0) p: vec2<f32>, @location(1) c: vec4<f32>) -> VsOut {
    var o: VsOut;
    o.pos = vec4<f32>(p, 0.0, 1.0);
    o.c = c;
    return o;
}
@fragment fn fs_main(v: VsOut) -> @location(0) vec4<f32> {
    return v.c;
}
";
        if quads.is_empty() {
            return;
        }
        if !self.overlay_pipes.contains_key(&format) {
            let module = self.device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("HUD overlay"), source: wgpu::ShaderSource::Wgsl(OVERLAY_WGSL.into()) });
            let layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("HUD overlay"), bind_group_layouts: &[], push_constant_ranges: &[] });
            let attrs = [
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 0, shader_location: 0 },
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Unorm8x4, offset: 8, shader_location: 1 },
            ];
            let pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("HUD overlay"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[wgpu::VertexBufferLayout { array_stride: 12, step_mode: wgpu::VertexStepMode::Vertex, attributes: &attrs }],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState { format, blend: Some(wgpu::BlendState::ALPHA_BLENDING), write_mask: wgpu::ColorWrites::COLOR })],
                }),
                multiview: None,
                cache: None,
            });
            self.overlay_pipes.insert(format, pipeline);
        }
        // two triangles per quad, in clip space
        let (fw, fh) = (tw.max(1) as f32, th.max(1) as f32);
        let mut v: Vec<u8> = Vec::with_capacity(quads.len() * 6 * 12);
        for q in quads {
            let (x0, y0) = (q.x / fw * 2.0 - 1.0, 1.0 - q.y / fh * 2.0);
            let (x1, y1) = ((q.x + q.w) / fw * 2.0 - 1.0, 1.0 - (q.y + q.h) / fh * 2.0);
            for (px, py) in [(x0, y0), (x1, y0), (x1, y1), (x0, y0), (x1, y1), (x0, y1)] {
                v.extend_from_slice(&px.to_le_bytes());
                v.extend_from_slice(&py.to_le_bytes());
                v.extend_from_slice(&q.c);
            }
        }
        let buf = self.device.create_buffer(&wgpu::BufferDescriptor { label: Some("HUD overlay"), size: v.len() as u64, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        self.queue.write_buffer(&buf, 0, &v);
        let mut enc = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("HUD overlay"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.overlay_pipes[&format]);
            pass.set_vertex_buffer(0, buf.slice(..));
            pass.draw(0..(quads.len() * 6) as u32, 0..1);
        }
        self.queue.submit([enc.finish()]);
    }

    /// What the window shows at the EFB's size: the EFB blitted (alpha opaque) with `quads` drawn over it, read back
    /// as RGBA8 (screenshots with the HUD overlay).
    pub fn read_composed(&mut self, quads: &[OverlayQuad]) -> Vec<u8> {
        self.read_composed_size(self.width, self.height, quads)
    }

    /// [`Renderer::read_composed`] at a given target size (the window's: the gd_ui batch is laid out for it).
    pub fn read_composed_size(&mut self, w: u32, h: u32, quads: &[OverlayQuad]) -> Vec<u8> {
        let fmt = wgpu::TextureFormat::Rgba8Unorm;
        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("HUD composed"),
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: fmt,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = tex.create_view(&Default::default());
        self.blit_to(&view, fmt, w, h);
        self.overlay_to(&view, fmt, w, h, quads);
        read_texture(&self.device, &self.queue, &tex, w, h)
    }

    /// Copy the colour target back to the CPU (RGBA8, row-major, tight rows).
    pub fn read_color(&self) -> Vec<u8> {
        read_texture(&self.device, &self.queue, &self.color, self.width, self.height)
    }

    /// `MELEE_GX_COPYDUMP=<prepared frame>`: write every EFB copy target as `gxcopy_<dest>.ppm` in the working directory.
    fn dump_copies(&self) {
        let Some(n) = std::env::var("MELEE_GX_COPYDUMP").ok().and_then(|v| v.parse::<u64>().ok()) else { return };
        if self.prepared != n {
            return;
        }
        for (dest, t) in &self.efb_copies {
            let (w, h) = (t._tex.width(), t._tex.height());
            let px = read_texture(&self.device, &self.queue, &t._tex, w, h);
            let mut f = format!("P6
{w} {h}
255
").into_bytes();
            for p in px.chunks(4) {
                f.extend_from_slice(&p[..3]);
            }
            let _ = std::fs::write(format!("gxcopy_{dest:x}.ppm"), f);
        }
    }
}

/// One HUD overlay rectangle in target pixels (see [`Renderer::overlay_to`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverlayQuad {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub c: [u8; 4],
}

/// Read back an RGBA8 texture.
fn read_texture(device: &wgpu::Device, queue: &wgpu::Queue, tex: &wgpu::Texture, width: u32, height: u32) -> Vec<u8> {
    {
        let row = width * 4;
        let padded = row.div_ceil(256) * 256;
        let buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GX readback"),
            size: (padded * height) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = device.create_command_encoder(&Default::default());
        enc.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo { texture: tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: Default::default() },
            wgpu::TexelCopyBufferInfo {
                buffer: &buf,
                layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(padded), rows_per_image: Some(height) },
            },
            wgpu::Extent3d { width: width, height: height, depth_or_array_layers: 1 },
        );
        queue.submit([enc.finish()]);
        let slice = buf.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::wait_indefinitely()).expect("poll");
        let data = slice.get_mapped_range();
        let mut out = Vec::with_capacity((row * height) as usize);
        for y in 0..height {
            let o = (y * padded) as usize;
            out.extend_from_slice(&data[o..o + row as usize]);
        }
        out
    }
}

impl Renderer {
}

/// `MELEE_GX_STATS=<csv path>` (or `1` for stderr): one row per prepared frame with the draw count, the
/// distinct sampled textures (by data address), the distinct pipeline configs (TEV + fixed-function
/// state, compared by value) and the config changes between consecutive draws. Comparable to the Source
/// Port's frames.csv `draws` column. Off by default; render-only.
fn frame_stats(frame: u64, draws: &[DrawCmd]) {
    use std::io::Write;
    use std::sync::OnceLock;
    static OUT: OnceLock<Option<std::sync::Mutex<Box<dyn Write + Send>>>> = OnceLock::new();
    let out = OUT.get_or_init(|| {
        let v = std::env::var("MELEE_GX_STATS").ok()?;
        let mut w: Box<dyn Write + Send> = if v == "1" {
            Box::new(std::io::stderr())
        } else {
            Box::new(std::io::BufWriter::new(std::fs::File::create(&v).ok()?))
        };
        let _ = writeln!(w, "frame,draws,textures,configs,config_changes,verts,batches,copies,missing_tex,direct_pos");
        Some(std::sync::Mutex::new(w))
    });
    let Some(out) = out else { return };
    let mut tex: std::collections::HashSet<(usize, u32, u32)> = Default::default();
    let mut cfgs: Vec<&std::sync::Arc<crate::shader::PipelineConfig>> = Vec::new();
    let mut changes = 0u32;
    let mut verts = 0u64;
    let mut prev: Option<&std::sync::Arc<crate::shader::PipelineConfig>> = None;
    // `batches`: draws left after merging each run of back-to-back draws that change no state (same pipeline config,
    // uniforms, textures, matrix slot, viewport and scissor), the Source Port's per-display-list primitive merge.
    let mut batches = 0u32;
    let mut copies = 0u32;
    let mut missing = 0u32;
    let mut direct = 0u32;
    let mut prev_d: Option<&DrawCmd> = None;
    let mut last_copy = 0usize;
    let dump = std::env::var("MELEE_GX_DRAWDUMP").ok().and_then(|v| v.parse::<u64>().ok()) == Some(frame);
    for (di, d) in draws.iter().enumerate() {
        if d.copy.is_none() && d.config.shader.attrs[GX_VA_POS as usize].attr_type == GX_DIRECT as u8 {
            direct += 1;
        }
        if dump {
            let f = |o: usize| if d.verts.len() >= o + 4 { f32::from_be_bytes([d.verts[o], d.verts[o + 1], d.verts[o + 2], d.verts[o + 3]]) } else { f32::NAN };
            eprintln!("DD {di} prim {:#x} n {} fmt {} vlen {} p0 {:.1} {:.1} {:.1} tex {:#x} t0 {}x{}f{} blend {} {} {} z {} {} acmp {:?} stages {} vp {:?} pnmtx {} attrs {:?}", d.prim, d.vtx_count, d.fmt, d.verts.len(), f(0), f(4), f(8), d.info.sampled_textures, d.textures[0].width(), d.textures[0].height(), d.textures[0].format(), d.config.blend_mode, d.config.blend_fac_src, d.config.blend_fac_dst, d.config.depth_compare, d.config.depth_func, d.config.shader.alpha_compare, d.config.shader.tev_stage_count, (d.render_viewport.left, d.render_viewport.width), d.current_pn_mtx, d.config.shader.attrs.iter().map(|a| a.attr_type).collect::<Vec<_>>());
        }
        if (d.prim == 0xA8 || d.prim == 0xB0 || d.prim == 0xB8) && frame % 120 == 0 && std::env::var_os("MELEE_GX_LINELOG").is_some() {
            eprintln!("gx line frame {frame} draw {di} prim {:#x} verts {} fmt {} line_mode {} tex {:#x} vtx {:02x?}", d.prim, d.vtx_count, d.fmt, d.info.line_mode, d.info.sampled_textures, &d.verts[..d.verts.len().min(64)]);
        }
        if let Some(c) = d.copy.as_deref() {
            copies += 1;
            prev_d = None;
            if std::env::var_os("MELEE_GX_COPYLOG").is_some() && frame % 120 == 0 {
                eprintln!("gx copy frame {frame} at draw {di} (+{}) dest {:#x} src {:?} dst {}x{} fmt {:#x} half {} clear {} alpha {}", di - last_copy, c.dest, c.src, c.dst_w, c.dst_h, c.fmt, c.half, c.clear, c.efb_alpha);
                for k in last_copy..di {
                    let e = &draws[k];
                    if e.copy.is_none() {
                        eprintln!("   draw {k} prim {:#x} verts {} tex {:#x} stages {} vp {:?} cull {} zcmp {} blend {}", e.prim, e.vtx_count, e.info.sampled_textures, e.config.shader.tev_stage_count, e.render_viewport, e.config.cull_mode, e.config.depth_compare, e.config.blend_mode);
                        if k == last_copy + 1 && std::env::var_os("MELEE_GX_COPYLOG_TEV").is_some() {
                            for t in 0..e.config.shader.tev_stage_count as usize { eprintln!("     TEV {t}: {:?}", e.config.shader.tev_stages[t]); }
                            eprintln!("     ALPHACMP {:?} blend {} {} {} chans {:?}", e.config.shader.alpha_compare, e.config.blend_fac_src, e.config.blend_fac_dst, e.config.blend_op, &e.config.shader.color_channels[..2]);
                            let t0 = &e.textures[0];
                            eprintln!("     TEX0 {}x{} fmt {} data {:?} guest {:#x} verts {:02x?} uniform {}", t0.width(), t0.height(), t0.format(), t0.data.as_ref().map(|d| d.len()), t0.guest, &e.verts[..e.verts.len().min(40)], e.uniform.len());
                        }
                    }
                }
            }
            last_copy = di + 1;
            continue;
        }
        let same = prev_d.is_some_and(|p| {
            (std::sync::Arc::ptr_eq(&p.config, &d.config) || *p.config == *d.config)
                && p.uniform == d.uniform
                && p.current_pn_mtx == d.current_pn_mtx
                && p.render_viewport == d.render_viewport
                && p.render_scissor == d.render_scissor
                && (0..MAX_TEXTURES).all(|m| p.textures[m].tex_obj_id == d.textures[m].tex_obj_id && p.textures[m].guest == d.textures[m].guest)
        });
        if !same {
            batches += 1;
        }
        prev_d = Some(d);
        verts += d.vtx_count as u64;
        for m in 0..MAX_TEXTURES {
            if d.info.sampled_textures & (1 << m) != 0 {
                let t = &d.textures[m];
                if t.data.is_none() {
                    missing += 1;
                    if std::env::var_os("MELEE_GX_MISSLOG").is_some() && frame % 60 == 0 {
                        eprintln!("gx missing tex frame {frame} map {m} {}x{} fmt {} guest {:#x}", t.width(), t.height(), t.format(), t.guest);
                    }
                }
                tex.insert((t.data.as_ref().map_or(0, |x| x.as_ptr() as usize), t.image0, t.tlut));
            }
        }
        if prev.map_or(true, |p| !std::sync::Arc::ptr_eq(p, &d.config) && **p != *d.config) {
            changes += 1;
        }
        prev = Some(&d.config);
        if !cfgs.iter().any(|c| std::sync::Arc::ptr_eq(c, &d.config) || ***c == *d.config) {
            cfgs.push(&d.config);
        }
    }
    if let Ok(mut w) = out.lock() {
        let _ = writeln!(w, "{frame},{},{},{},{changes},{verts},{batches},{copies},{missing},{direct}", draws.len() as u32 - copies, tex.len(), cfgs.len());
        let _ = w.flush();
    }
}

/// The MSAA counts (bit n = n samples) every EFB attachment format supports on the device the renderer created; 0
/// until a renderer exists. The settings overlay offers only these.
pub static SUPPORTED_MSAA: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Bit n set when n samples work for both EFB attachment formats on a device with `features` from `adapter`.
fn supported_msaa_mask(adapter: &wgpu::Adapter, features: wgpu::Features) -> u32 {
    let flags = |f: wgpu::TextureFormat| {
        if features.contains(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES) {
            adapter.get_texture_format_features(f).flags
        } else {
            f.guaranteed_format_features(features).flags
        }
    };
    let mut m = 1 << 1;
    for n in [2u32, 4, 8] {
        if [COLOR_FORMAT, DEPTH_FORMAT].iter().all(|&f| flags(f).sample_count_supported(n)) {
            m |= 1 << n;
        }
    }
    m
}

/// The largest supported count (`mask` as [`SUPPORTED_MSAA`]) not above `want`; 1 when none is.
pub fn clamp_msaa(want: u32, mask: u32) -> u32 {
    [8u32, 4, 2, 1].into_iter().find(|&n| n <= want && mask >> n & 1 != 0).unwrap_or(1)
}

/// The EFB colour (single-sample: the resolve target with MSAA), the depth (`samples`) and, with MSAA, the
/// multisampled colour target, all `w` x `h`.
fn make_targets(device: &wgpu::Device, w: u32, h: u32, samples: u32) -> (wgpu::Texture, wgpu::Texture, Option<wgpu::Texture>) {
    let mk = |format, usage, label, n| {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: n,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        })
    };
    let color = mk(
        COLOR_FORMAT,
        wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::TEXTURE_BINDING,
        "GX EFB color",
        1,
    );
    let depth = mk(DEPTH_FORMAT, wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING, "GX EFB depth", samples);
    let msaa = (samples > 1).then(|| mk(COLOR_FORMAT, wgpu::TextureUsages::RENDER_ATTACHMENT, "GX EFB color (MSAA)", samples));
    (color, depth, msaa)
}

/// Record `draws[range]` of the prepared frame `fc` into `pass` (an EFB pass; redundant state changes skipped).
fn record_draws(pass: &mut wgpu::RenderPass<'_>, fc: &FrameCache, draws: &[DrawCmd], range: std::ops::Range<usize>, (width, height, kx, ky): (u32, u32, f32, f32)) {
    let (Some(sbg), Some(gi), Some((ubg, _))) = (fc.storage_bg.as_ref(), fc.gi.as_ref(), fc.uniform_bg.as_ref()) else { return };
    if range.is_empty() {
        return;
    }
    pass.set_bind_group(0, sbg, &[]);
    pass.set_index_buffer(gi.buf.slice(..), wgpu::IndexFormat::Uint16);
    let (w, h) = (width as f32, height as f32);
    // Logical GX pixels -> target pixels: `kx == ky == scale` at 4:3; a widescreen target stretches x, as Aurora's
    // map_logical_viewport / map_logical_scissor do. Scissor edges snap outward to whole pixels.
    let edge = |v: i32, k: f32, up: bool, max: u32| -> u32 {
        let f = v as f32 * k;
        (if up { f.ceil() } else { f.floor() }).clamp(0.0, max as f32) as u32
    };
    let mut cur_vp = [f32::NAN; 6];
    let mut cur_sc = [u32::MAX; 4];
    let mut cur_pipe: Option<&wgpu::RenderPipeline> = None;
    let mut cur_tex: Option<&wgpu::BindGroup> = None;
    let mut cur_blend = f64::NAN;
    let mut cur_uoff = u32::MAX;
    for i in range {
        let d = &draws[i];
        let pd = &fc.draws[i];
        let (Some(p), Some(tex_bg)) = (pd.pipeline.as_ref(), pd.tex_bg.as_ref()) else { continue };
        let sc = d.render_scissor;
        let sx = edge(sc.x, kx, false, width);
        let sy = edge(sc.y, ky, false, height);
        let sw = edge(sc.x + sc.width.max(0), kx, true, width).saturating_sub(sx);
        let sh = edge(sc.y + sc.height.max(0), ky, true, height).saturating_sub(sy);
        if sw == 0 || sh == 0 {
            continue;
        }
        let vp = d.render_viewport;
        let left = (vp.left * kx).clamp(0.0, w);
        let top = (vp.top * ky).clamp(0.0, h);
        let v = [left, top, (vp.width * kx).min(w - left).max(1.0), (vp.height * ky).min(h - top).max(1.0), 1.0 - vp.zfar, 1.0 - vp.znear];
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
            pass.draw(0..pd.count, 0..d.instance_count);
        } else {
            pass.draw_indexed(pd.first_index..pd.first_index + pd.count, 0, 0..d.instance_count);
        }
    }
}
