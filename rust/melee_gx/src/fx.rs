//! The Envoy effects layer (docs/rust-port/notes/envoy-fx.md): presentation-only GPU effects on top of the GX frame.
//!
//! Three parts, all on the presenting side and all off until a run arms ([`Renderer::fx_arm`]):
//! - **World materials** ([`Renderer::fx_world`]): the drive models and their fire shapes, drawn after the GX frame's
//!   last EFB pass into the same colour and depth targets (depth-tested), with fixed material shaders (glass with a
//!   fresnel rim, a pulsing inner glow, foil, dissolve, unlit additive) and an emissive target the bloom reads.
//! - **Bloom**: the emissive target blurred at half resolution (the drives' output only: the stage never writes it).
//! - **Post** (inside [`Renderer::blit_to`], before the UI batch): the Heat screen treatment (vignette, ember tint,
//!   edge heat-haze with drifting embers) and one-shots (cash-out ring, flash, colour split, boss-break shockwave),
//!   fused into the blit's one full-screen pass. The centre of the picture is never touched and every tint is capped.
//!
//! The simulation never sees any of it: the inputs are plain data the presenter hands over each present
//! ([`FxFrame`]), time is a render clock, and nothing here touches the GX stream or the game arena. With the layer
//! disarmed (or armed with nothing to draw) every pass is skipped and the blit is the plain one, byte for byte.

use crate::render::{Renderer, COLOR_FORMAT, DEPTH_FORMAT};

/// World materials (`gd_ui_world_model_ex`).
pub const MAT_BODY: u8 = 0;
pub const MAT_GLASS: u8 = 1;
pub const MAT_GLOW: u8 = 2;
pub const MAT_FOIL: u8 = 3;
pub const MAT_DISSOLVE: u8 = 4;
pub const MAT_UNLIT_ADD: u8 = 5;
pub const MAT_COUNT: u8 = 6;
/// Material flag: the host leaves a light trail behind this draw (orbiting drives).
pub const MAT_TRAIL: u8 = 0x80;
/// Material flag: a drive body with the Party-pack ink edge (~1.2 px at 640x480, a screen-space hull drawn under it).
pub const MAT_OUTLINE: u8 = 0x40;
/// The material id bits.
pub const MAT_ID: u8 = 0x3F;
/// The ink edge: colour (Party pack INK) and width in 640x480 pixels.
pub const INK_RGB: [f32; 3] = [0x2A as f32 / 255.0, 0x1A as f32 / 255.0, 0x6E as f32 / 255.0];
pub const INK_PX: f32 = 1.2;

/// Post effects (`gd_fx_post`). Held ones (re-posted every present while they last): vignette, ember, heat-haze.
pub const POST_VIGNETTE: u32 = 1;
pub const POST_EMBER: u32 = 2;
pub const POST_HEAT_HAZE: u32 = 3;
/// One-shots (posted once; the host runs their timeline): a flash, a radial ring, a colour split, a shockwave.
pub const POST_FLASH: u32 = 4;
pub const POST_RADIAL_WAVE: u32 = 5;
pub const POST_CHROMA: u32 = 6;
pub const POST_SHOCKWAVE: u32 = 7;
pub const POST_COUNT: u32 = 8;
/// Flag on the id: (x, y) is a world position (projected with the fight camera), else 640x480 canvas units.
pub const POST_WORLD: u32 = 0x100;
/// Active post effects at once (held + one-shots).
pub const MAX_POST: usize = 4;
/// The cap on any tint / flash weight at full intensity (the study's 15%).
pub const CAP: f32 = 0.15;
/// Photosensitivity: at most this many flashes per second.
pub const FLASH_PER_SEC: f32 = 3.0;

/// One post request.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Post {
    pub id: u32,
    pub strength: f32,
    pub x: f32,
    pub y: f32,
    pub rgba: u32,
}

/// One world draw (a drive model with a material).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WorldDraw {
    pub model: u8,
    /// MAT_* (| MAT_TRAIL).
    pub mat: u8,
    pub at: [f32; 3],
    pub yaw: f32,
    /// Diameter in world units.
    pub scale: f32,
    pub alpha: f32,
    /// Material parameters: glow, charge 0..1, wearer's Heat state 0..3, material extra (foil rarity / dissolve).
    pub p: [f32; 4],
    pub rgba: u32,
}

/// The fight camera as GX had it when the world list was drawn: the view matrix rows, the GX projection
/// (`GXGetProjectionv`: type, p00, p02, p11, p12, p22, p23) and viewport (`GXGetViewportv`, logical pixels).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Camera {
    pub view: [[f32; 4]; 3],
    pub proj: [f32; 7],
    pub vp: [f32; 6],
}

/// A mesh handed over once at arm: triangle list, unit-diameter space.
pub struct FxMesh<'a> {
    pub pos: &'a [[f32; 3]],
    pub nrm: &'a [[f32; 3]],
    pub base: &'a [[f32; 3]],
    pub glow: &'a [[f32; 3]],
}

/// A coloured world triangle vertex (fire shapes; drawn additive).
pub type TriVert = ([f32; 3], [u8; 4]);

/// One present's effect inputs (the presenter fills it; [`Renderer::set_fx`] swaps the vectors, so steady state
/// allocates nothing).
#[derive(Default)]
pub struct FxFrame {
    /// 0 off, 0.5 subtle, 1 full.
    pub intensity: f32,
    pub cam: Option<Camera>,
    pub draws: Vec<WorldDraw>,
    pub tris: Vec<TriVert>,
    pub posts: Vec<Post>,
}

/// One tick's world list, kept with the GX frame it arrived with (the presenter adopts both together, so the
/// drives are posed at the same tick, and the same in-between phase, as the scene under them).
pub struct FxSnap {
    draws: Vec<WorldDraw>,
    tris: Vec<TriVert>,
    cam: Option<Camera>,
}

/// An in-between present: phase `t` (interpolate: 0 = the previous tick, 1 = the adopted one; extrapolate: past
/// the adopted one) and the scene camera's view at that phase (the interpolator's; None: the tick's own).
#[derive(Clone, Copy, Debug)]
pub struct FxPhase {
    pub t: f32,
    pub extrapolate: bool,
    pub view: Option<[[f32; 4]; 3]>,
}

/// A drive moving further than this between ticks teleported (a respawn, a tag): it snaps, never lerps.
const FX_MAX_STEP: f32 = 30.0;

/// The effects-intensity setting (`MELEE_FX` = off | subtle | full; default full offline, subtle in netplay).
pub fn intensity_from_env(netplay: bool) -> f32 {
    static V: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    match V.get_or_init(|| std::env::var("MELEE_FX").ok()).as_deref().map(str::trim).map(str::to_ascii_lowercase).as_deref() {
        Some("off") | Some("0") => 0.0,
        Some("subtle") | Some("1") => 0.5,
        Some("full") | Some("2") => 1.0,
        _ => if netplay { 0.5 } else { 1.0 },
    }
}

// ---- the post queue (filled by `gd_fx_post` during a mod's present, drained by the presenter)

static POSTS: std::sync::Mutex<Vec<Post>> = std::sync::Mutex::new(Vec::new());

/// Queue a post request for this present. False: refused (bad id, non-finite, or this present already has
/// [`MAX_POST`]). Parameters are clamped here.
pub fn post(id: u32, strength: f32, x: f32, y: f32, rgba: u32) -> bool {
    let kind = id & 0xFF;
    if !(1..POST_COUNT).contains(&kind) || id & !(0xFF | POST_WORLD) != 0 || ![strength, x, y].iter().all(|v| v.is_finite() && v.abs() < 1.0e6) {
        return false;
    }
    let Ok(mut q) = POSTS.lock() else { return false };
    if q.len() >= MAX_POST {
        return false;
    }
    q.push(Post { id, strength: strength.clamp(0.0, 1.0), x, y, rgba });
    true
}

/// Move the queued posts into `out` (cleared first).
pub fn take_posts(out: &mut Vec<Post>) {
    out.clear();
    if let Ok(mut q) = POSTS.lock() {
        out.extend(q.drain(..));
    }
}

/// Whether any post is queued (the presenter arms on it).
pub fn posts_pending() -> bool {
    POSTS.lock().is_ok_and(|q| !q.is_empty())
}

// ---- GPU side

const INST_BYTES: u64 = 96;
const VERT_BYTES: u64 = 48;
const TRI_BYTES: u64 = 16;
const MAX_INST: usize = 384;
const MAX_TRI_VERTS: usize = 3 * 24_000;
const TRAIL_LEN: usize = 10;
const TRAILS: usize = 32;
const POST_UBYTES: usize = 16 * 16;

#[derive(Clone, Copy, Default)]
struct Held {
    id: u32,
    cur: f32,
    target: f32,
    seen: f32,
    rgba: u32,
}

#[derive(Clone, Copy, Default)]
struct Shot {
    id: u32,
    t0: f32,
    uv: [f32; 2],
    strength: f32,
    rgba: u32,
}

fn life(id: u32) -> f32 {
    match id {
        POST_FLASH => 0.16,
        POST_RADIAL_WAVE => 0.5,
        POST_CHROMA => 0.3,
        POST_SHOCKWAVE => 0.8,
        _ => 0.0,
    }
}

/// Per-pass GPU times (ms, measured by `MELEE_FX_PROF=1` with a queue wait: diagnostics only).
#[derive(Clone, Copy, Debug, Default)]
pub struct FxTimes {
    pub world_ms: f32,
    pub bloom_ms: f32,
}

pub(crate) struct FxGpu {
    pub armed: bool,
    clock: std::time::Instant,
    last_t: f32,
    // world
    vbuf: wgpu::Buffer,
    ranges: Vec<(u32, u32)>,
    inst: wgpu::Buffer,
    tri: wgpu::Buffer,
    globals: wgpu::Buffer,
    world_bg: wgpu::BindGroup,
    world_module: wgpu::ShaderModule,
    world_layout: wgpu::PipelineLayout,
    /// opaque, blend, add, tris; for `world_samples`
    world_pipes: Vec<wgpu::RenderPipeline>,
    world_samples: u32,
    // emissive + bloom (internal resolution / half)
    emis: Option<Targets>,
    blur_bgl: wgpu::BindGroupLayout,
    blur_pipes: [wgpu::RenderPipeline; 2],
    lin: wgpu::Sampler,
    /// The last world pass wrote emissive pixels (the bloom composite is on).
    bloom_live: bool,
    // post
    post_bgl: wgpu::BindGroupLayout,
    post_layout: wgpu::PipelineLayout,
    post_module: wgpu::ShaderModule,
    post_pipes: Vec<(wgpu::TextureFormat, wgpu::RenderPipeline)>,
    post_ubuf: wgpu::Buffer,
    black: wgpu::TextureView,
    // state
    held: [Held; MAX_POST],
    shots: [Shot; MAX_POST],
    last_flash: f32,
    intensity: f32,
    frame: FxFrame,
    inst_scratch: Vec<u8>,
    tri_scratch: Vec<u8>,
    trails: Vec<([[f32; 3]; TRAIL_LEN], usize, f32)>,
    /// The previous and the adopted tick's world lists (see `FxSnap`), and the posed list drawn this present.
    snap: [Option<Box<FxSnap>>; 2],
    eff: Vec<WorldDraw>,
    pub times: FxTimes,
}

struct Targets {
    size: (u32, u32, u32),
    emis: wgpu::Texture,
    emis_msaa: Option<wgpu::Texture>,
    half: [wgpu::Texture; 2],
    bgs: [wgpu::BindGroup; 2],
}

const WORLD_WGSL: &str = "
struct G { v0: vec4f, v1: vec4f, v2: vec4f, p0: vec4f, p1: vec4f, p2: vec4f, p3: vec4f, misc: vec4f, glass: vec4f };
@group(0) @binding(0) var<uniform> g: G;
struct VO {
    @builtin(position) pos: vec4f,
    @location(0) base: vec3f,
    @location(1) glow: vec3f,
    @location(2) nw: vec3f,
    @location(3) vp: vec3f,
    @location(4) lp: vec3f,
    @location(5) nv: vec3f,
    @location(6) @interpolate(flat) par: vec4f,
    @location(7) @interpolate(flat) col: vec4f,
    @location(8) @interpolate(flat) misc: vec4f,
};
struct FO { @location(0) c: vec4f, @location(1) e: vec4f };
fn proj(vp: vec3f) -> vec4f {
    let h = vec4f(vp, 1.0);
    return vec4f(dot(g.p0, h), dot(g.p1, h), dot(g.p2, h), dot(g.p3, h));
}
fn view(w: vec3f) -> vec3f {
    let h = vec4f(w, 1.0);
    return vec3f(dot(g.v0, h), dot(g.v1, h), dot(g.v2, h));
}
@vertex fn vs_mesh(@location(0) pos: vec3f, @location(1) nrm: vec3f, @location(2) base: vec3f, @location(3) glow: vec3f,
                   @location(4) m0: vec4f, @location(5) m1: vec4f, @location(6) m2: vec4f,
                   @location(7) par: vec4f, @location(8) col: vec4f, @location(9) misc: vec4f) -> VO {
    let h = vec4f(pos, 1.0);
    let w = vec3f(dot(m0, h), dot(m1, h), dot(m2, h));
    let nw = normalize(vec3f(dot(m0.xyz, nrm), dot(m1.xyz, nrm), dot(m2.xyz, nrm)));
    var o: VO;
    o.vp = view(w);
    o.pos = proj(o.vp);
    o.nw = nw;
    o.nv = vec3f(dot(g.v0.xyz, nw), dot(g.v1.xyz, nw), dot(g.v2.xyz, nw));
    o.base = base;
    o.glow = glow;
    o.lp = pos;
    o.par = par;
    o.col = col;
    o.misc = misc;
    return o;
}
fn hue(h: f32) -> vec3f {
    return clamp(abs(fract(vec3f(h, h + 0.6667, h + 0.3333)) * 6.0 - 3.0) - 1.0, vec3f(0.0), vec3f(1.0));
}
fn h3(p: vec3f) -> f32 {
    return fract(sin(dot(p, vec3f(127.1, 311.7, 74.7))) * 43758.547);
}
fn vnoise(p: vec3f) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = mix(mix(h3(i), h3(i + vec3f(1.0, 0.0, 0.0)), u.x), mix(h3(i + vec3f(0.0, 1.0, 0.0)), h3(i + vec3f(1.0, 1.0, 0.0)), u.x), u.y);
    let b = mix(mix(h3(i + vec3f(0.0, 0.0, 1.0)), h3(i + vec3f(1.0, 0.0, 1.0)), u.x), mix(h3(i + vec3f(0.0, 1.0, 1.0)), h3(i + vec3f(1.0, 1.0, 1.0)), u.x), u.y);
    return mix(a, b, u.z);
}
@fragment fn fs_mesh(v: VO) -> FO {
    let t = g.misc.x;
    let k = g.misc.y;
    let mat = u32(v.misc.x + 0.5);
    let alpha = v.misc.y;
    let heat = clamp(v.par.z, 0.0, 3.0);
    // the CPU Lambert drive_draw used (the same light), per pixel now
    let lit = 0.42 + 0.7 * max(dot(v.nw, vec3f(0.35, 0.6, 0.72)), 0.0);
    let nv = normalize(v.nv);
    let vd = normalize(-v.vp);
    let ndv = abs(dot(nv, vd));
    var gk = v.par.x;
    if (mat == 2u || mat == 3u || mat == 4u) {
        // the inner glow breathes faster as the drive charges and burns brighter with the wearer's Heat
        let ch = clamp(v.par.y, 0.0, 1.0);
        let pulse = 0.5 + 0.5 * sin(t * (2.2 + 6.0 * ch) + v.misc.z);
        gk = v.par.x * (1.0 + (pulse - 0.5) * (0.25 + 0.5 * ch) * k) * (1.0 + 0.2 * heat * k);
    }
    var rgb = min(v.base * lit + v.glow * gk, vec3f(1.0));
    var em = v.glow * gk * 0.6;
    // a faint rim on solid bodies
    let rim = pow(1.0 - ndv, 3.0);
    rgb = min(rgb + vec3f(rim * 0.18 * k), vec3f(1.0));
    var a = alpha;
    switch mat {
        case 1u: {
            // glass (drive_glass.material.json: opacity, roughness -> rim sharpness, tint)
            let f = pow(1.0 - ndv, g.glass.y);
            let tint = g.glass.z * mix(vec3f(0.78, 0.9, 1.0), v.col.rgb, 0.35);
            rgb = mix(tint * 0.5, vec3f(1.0), f);
            a = clamp(alpha * (0.45 + 2.2 * f), 0.0, 0.9);
            em = tint * f * 0.5 * k;
        }
        case 3u: {
            // iridescent foil: hue by view angle, drifting; Unique (par.w >= 2) stronger, with glints
            let uq = v.par.w >= 1.5;
            let ir = hue(ndv * 1.6 + t * 0.12 + v.lp.y * 1.8);
            let s = select(0.35, 0.6, uq) * k;
            rgb = mix(rgb, min(rgb * 0.45 + ir * 0.75, vec3f(1.0)), s);
            em = em + ir * s * 0.3;
            if (uq) {
                let c = floor(v.lp * 22.0);
                let sp = step(0.93, h3(c)) * pow(max(0.0, sin(t * 4.0 + h3(c + vec3f(5.0)) * 40.0)), 10.0);
                rgb = min(rgb + vec3f(sp * k), vec3f(1.0));
                em = em + vec3f(sp * 0.8 * k);
            }
        }
        case 4u: {
            // dissolve: par.w 0 whole .. 1 gone, a hot edge where it eats in
            let pr = clamp(v.par.w, 0.0, 1.0);
            let n = vnoise(v.lp * 9.0 + vec3f(3.1, 1.7, 0.4));
            let th = pr * 1.1 - 0.05;
            if (pr > 0.001 && n < th) { discard; }
            let edge = (1.0 - smoothstep(0.0, 0.1, n - th)) * step(0.001, pr);
            let ec = mix(vec3f(1.0, 0.86, 0.5), v.col.rgb, 0.45);
            rgb = mix(rgb, ec, edge);
            em = em + ec * edge * 1.4;
        }
        case 5u: {
            let c = v.col.rgb * v.col.a * alpha;
            rgb = c;
            a = 1.0;
            em = c * 0.8;
        }
        default: {}
    }
    var o: FO;
    o.c = vec4f(rgb, a);
    o.e = vec4f(em * k * select(1.0, a, mat == 1u), 1.0);
    return o;
}
// the ink edge: the body pushed out along its screen-space normal by misc.zw (NDC), flat ink, no emissive
@vertex fn vs_ink(@location(0) pos: vec3f, @location(1) nrm: vec3f, @location(2) base: vec3f, @location(3) glow: vec3f,
                  @location(4) m0: vec4f, @location(5) m1: vec4f, @location(6) m2: vec4f,
                  @location(7) par: vec4f, @location(8) col: vec4f, @location(9) misc: vec4f) -> @builtin(position) vec4f {
    let h = vec4f(pos, 1.0);
    let w = vec3f(dot(m0, h), dot(m1, h), dot(m2, h));
    let nw = normalize(vec3f(dot(m0.xyz, nrm), dot(m1.xyz, nrm), dot(m2.xyz, nrm)));
    let vp = view(w);
    var c = proj(vp);
    let nv = vec2f(dot(g.v0.xyz, nw), dot(g.v1.xyz, nw));
    let d = select(vec2f(0.0), normalize(nv), dot(nv, nv) > 1e-8);
    c = vec4f(c.xy + d * g.misc.zw * c.w, c.zw);
    return c;
}
@fragment fn fs_ink() -> FO {
    var o: FO;
    o.c = vec4f(0.165, 0.102, 0.431, 1.0);
    o.e = vec4f(0.0, 0.0, 0.0, 1.0);
    return o;
}
struct TO { @builtin(position) pos: vec4f, @location(0) c: vec4f };
@vertex fn vs_tri(@location(0) p: vec3f, @location(1) c: vec4f) -> TO {
    var o: TO;
    o.pos = proj(view(p));
    o.c = c;
    return o;
}
@fragment fn fs_tri(v: TO) -> FO {
    var o: FO;
    let c = v.c.rgb * v.c.a;
    o.c = vec4f(c, 1.0);
    o.e = vec4f(c * 0.45 * g.misc.y, 1.0);
    return o;
}
";

const BLUR_WGSL: &str = "
@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var smp: sampler;
struct VsOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> VsOut {
    let uv = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    var o: VsOut;
    o.pos = vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, 0.0, 1.0);
    o.uv = uv;
    return o;
}
fn blur(uv: vec2<f32>, step: vec2<f32>) -> vec4<f32> {
    var w = array<f32, 5>(0.2270, 0.1945, 0.1216, 0.0540, 0.0162);
    var c = textureSample(src, smp, uv).rgb * w[0];
    for (var i = 1; i < 5; i = i + 1) {
        let o = step * f32(i) * 1.6;
        c = c + (textureSample(src, smp, uv + o).rgb + textureSample(src, smp, uv - o).rgb) * w[i];
    }
    return vec4<f32>(c, 1.0);
}
// full-resolution emissive -> half: horizontal (in half-resolution texels)
@fragment fn fs_h(v: VsOut) -> @location(0) vec4<f32> {
    let d = vec2<f32>(textureDimensions(src));
    return blur(v.uv, vec2<f32>(2.0 / d.x, 0.0));
}
@fragment fn fs_v(v: VsOut) -> @location(0) vec4<f32> {
    let d = vec2<f32>(textureDimensions(src));
    return blur(v.uv, vec2<f32>(0.0, 1.0 / d.y));
}
";

/// The blit with the post effects fused in (used only while one is active; else the plain blit runs).
const POST_WGSL: &str = "
struct W { a: vec4<f32>, b: vec4<f32> };
struct U {
    m: vec4<f32>,      // time, intensity, EFB aspect, bloom
    heat: vec4<f32>,   // vignette, ember tint, haze, ember specks
    tint: vec4<f32>,   // heat colour
    flash: vec4<f32>,  // rgb, amount
    w: array<W, 4>,    // a: centre uv, radius, width; b: distortion, colour split, glow, 0
    wc: array<vec4<f32>, 4>,
};
@group(0) @binding(0) var efb: texture_2d<f32>;
@group(0) @binding(1) var smp: sampler;
@group(0) @binding(2) var bloom: texture_2d<f32>;
@group(0) @binding(3) var<uniform> u: U;
struct VsOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> VsOut {
    let uv = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    var o: VsOut;
    o.pos = vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, 0.0, 1.0);
    o.uv = uv;
    return o;
}
fn h2(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.547);
}
fn vn(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let s = f * f * (3.0 - 2.0 * f);
    return mix(mix(h2(i), h2(i + vec2<f32>(1.0, 0.0)), s.x), mix(h2(i + vec2<f32>(0.0, 1.0)), h2(i + vec2<f32>(1.0, 1.0)), s.x), s.y);
}
@fragment fn fs_main(v: VsOut) -> @location(0) vec4<f32> {
    let t = u.m.x;
    let asp = u.m.z;
    // 0 in the middle, 1 on the edge midpoints: the centre ellipse (60% of each axis) is never touched
    let r = length((v.uv - 0.5) * 2.0);
    let edge = smoothstep(0.6, 1.1, r);
    var uv = v.uv;
    // heat-haze, edges only
    if (u.heat.z > 0.0) {
        let q = v.uv * vec2<f32>(14.0 * asp, 14.0);
        let n = vec2<f32>(vn(q + vec2<f32>(0.0, t * 1.4)) + 0.5 * vn(q * 2.1 + vec2<f32>(t * 0.7, 0.0)), vn(q * 1.3 + vec2<f32>(t * 0.5, t * 1.9)) + 0.5 * vn(q * 2.3 - vec2<f32>(0.0, t))) - 0.75;
        uv = uv + n * 0.008 * u.heat.z * edge;
    }
    // rings: a displacement band travelling out from each centre
    var split = 0.0;
    var glow = vec3<f32>(0.0);
    let wm = smoothstep(0.45, 0.9, r);
    for (var i = 0; i < 4; i = i + 1) {
        let w = u.w[i];
        if (w.a.w <= 0.0) { continue; }
        let d = (v.uv - w.a.xy) * vec2<f32>(asp, 1.0);
        let dist = length(d);
        let x = (dist - w.a.z) / w.a.w;
        let band = exp(-x * x);
        let dir = d / max(dist, 1.0e-4);
        uv = uv - dir / vec2<f32>(asp, 1.0) * band * w.b.x * wm;
        split = split + band * w.b.y * wm;
        glow = glow + u.wc[i].rgb * band * w.b.z * wm;
    }
    // the colour split (uniform control flow: always three samples; o is 0 without a ring)
    let o = (v.uv - 0.5) * split;
    var col = vec3<f32>(textureSample(efb, smp, uv + o).r, textureSample(efb, smp, uv).g, textureSample(efb, smp, uv - o).b);
    let bl = textureSample(bloom, smp, v.uv).rgb;
    col = col + bl * u.m.w;
    // the tints: each weight edge-masked, their sum capped
    let fm = smoothstep(0.5, 1.2, r);
    let wv = u.heat.x * edge;
    let we = u.heat.y * edge * edge;
    // embers: specks rising in the edge band
    var sp = 0.0;
    if (u.heat.w > 0.0) {
        // embers: glowing specks rising (and swaying) through the edge band; not a tint, so outside the tint cap
        let g = v.uv * vec2<f32>(16.0 * asp, 16.0) + vec2<f32>(0.0, t * 1.6);
        let c = floor(g);
        let h = h2(c);
        let sway = sin(t * 2.0 + h * 30.0) * 0.18;
        let jit = vec2<f32>(h2(c + 3.7) * 0.6 + 0.2 + sway, h2(c + 9.1) * 0.6 + 0.2);
        let dv = (fract(g) - jit) * vec2<f32>(1.0, 0.55);
        let dd = length(dv);
        let fl = 0.65 + 0.35 * sin(t * 9.0 + h * 50.0);
        sp = step(0.78, h) * (1.0 - smoothstep(0.03, 0.13, dd)) * fl * u.heat.w * edge;
    }
    let wf = u.flash.w * fm;
    let gl = length(glow);
    let sum = wv + we + wf + gl;
    let cap = 0.15 * u.m.y;
    let s = select(1.0, cap / sum, sum > cap);
    col = mix(col, u.tint.rgb, wv * s);
    col = mix(col, u.tint.rgb, we * s);
    // embers paint over (not add to) the picture: orange on a bright stage too, a hot yellow core
    col = mix(col, mix(vec3<f32>(1.0, 0.36, 0.05), vec3<f32>(1.0, 0.78, 0.3), sp * sp), min(sp * 1.4, 0.95));
    col = col + glow * s;
    col = mix(col, u.flash.rgb, wf * s);
    return vec4<f32>(col, 1.0);
}
";

fn rgba_f(c: u32) -> [f32; 4] {
    [(c >> 24) as f32 / 255.0, ((c >> 16) & 0xFF) as f32 / 255.0, ((c >> 8) & 0xFF) as f32 / 255.0, (c & 0xFF) as f32 / 255.0]
}

fn put4(b: &mut Vec<u8>, v: [f32; 4]) {
    for x in v {
        b.extend_from_slice(&x.to_le_bytes());
    }
}

fn ease(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    1.0 - (1.0 - x) * (1.0 - x) * (1.0 - x)
}

impl FxGpu {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, meshes: &[FxMesh<'_>], samples: u32) -> FxGpu {
        // meshes: one vertex buffer, a range per model
        let mut v: Vec<u8> = Vec::new();
        let mut ranges = Vec::with_capacity(meshes.len());
        for m in meshes {
            let start = (v.len() as u64 / VERT_BYTES) as u32;
            for i in 0..m.pos.len() {
                for a in [m.pos[i], m.nrm[i], m.base[i], m.glow[i]] {
                    for x in a {
                        v.extend_from_slice(&x.to_le_bytes());
                    }
                }
            }
            ranges.push((start, m.pos.len() as u32));
        }
        if v.is_empty() {
            v.resize(VERT_BYTES as usize, 0);
        }
        let mk = |label, size: u64, usage| device.create_buffer(&wgpu::BufferDescriptor { label: Some(label), size, usage, mapped_at_creation: false });
        let vbuf = mk("fx meshes", v.len() as u64, wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST);
        queue.write_buffer(&vbuf, 0, &v);
        let inst = mk("fx instances", INST_BYTES * MAX_INST as u64, wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST);
        let tri = mk("fx tris", TRI_BYTES * MAX_TRI_VERTS as u64, wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST);
        let globals = mk("fx globals", 16 * 9, wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST);
        let post_ubuf = mk("fx post", POST_UBYTES as u64, wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST);
        let world_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fx world"),
            entries: &[wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::VERTEX_FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None }],
        });
        let world_bg = device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("fx world"), layout: &world_bgl, entries: &[wgpu::BindGroupEntry { binding: 0, resource: globals.as_entire_binding() }] });
        let world_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("fx world"), bind_group_layouts: &[&world_bgl], push_constant_ranges: &[] });
        let world_module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("fx world"), source: wgpu::ShaderSource::Wgsl(WORLD_WGSL.into()) });
        let tex_entry = |b| wgpu::BindGroupLayoutEntry { binding: b, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None };
        let smp_entry = |b| wgpu::BindGroupLayoutEntry { binding: b, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None };
        let blur_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("fx blur"), entries: &[tex_entry(0), smp_entry(1)] });
        let blur_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("fx blur"), bind_group_layouts: &[&blur_bgl], push_constant_ranges: &[] });
        let blur_module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("fx blur"), source: wgpu::ShaderSource::Wgsl(BLUR_WGSL.into()) });
        let blur = |fs: &str| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("fx blur"),
                layout: Some(&blur_layout),
                vertex: wgpu::VertexState { module: &blur_module, entry_point: Some("vs_main"), compilation_options: Default::default(), buffers: &[] },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState { module: &blur_module, entry_point: Some(fs), compilation_options: Default::default(), targets: &[Some(wgpu::ColorTargetState { format: COLOR_FORMAT, blend: None, write_mask: wgpu::ColorWrites::ALL })] }),
                multiview: None,
                cache: None,
            })
        };
        let blur_pipes = [blur("fs_h"), blur("fs_v")];
        let lin = device.create_sampler(&wgpu::SamplerDescriptor { label: Some("fx linear"), mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, address_mode_u: wgpu::AddressMode::ClampToEdge, address_mode_v: wgpu::AddressMode::ClampToEdge, ..Default::default() });
        let u_entry = wgpu::BindGroupLayoutEntry { binding: 3, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None };
        let post_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("fx post"), entries: &[tex_entry(0), smp_entry(1), tex_entry(2), u_entry] });
        let post_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("fx post"), bind_group_layouts: &[&post_bgl], push_constant_ranges: &[] });
        let post_module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("fx post"), source: wgpu::ShaderSource::Wgsl(POST_WGSL.into()) });
        let black_t = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("fx black"),
            size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: COLOR_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo { texture: &black_t, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            &[0, 0, 0, 255],
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4), rows_per_image: Some(1) },
            wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
        );
        let mut g = FxGpu {
            armed: true,
            clock: std::time::Instant::now(),
            last_t: 0.0,
            vbuf,
            ranges,
            inst,
            tri,
            globals,
            world_bg,
            world_module,
            world_layout,
            world_pipes: Vec::new(),
            world_samples: 0,
            emis: None,
            blur_bgl,
            blur_pipes,
            lin,
            bloom_live: false,
            post_bgl,
            post_layout,
            post_module,
            post_pipes: Vec::new(),
            post_ubuf,
            black: black_t.create_view(&Default::default()),
            held: [Held::default(); MAX_POST],
            shots: [Shot::default(); MAX_POST],
            last_flash: -10.0,
            intensity: 1.0,
            frame: FxFrame { intensity: 1.0, cam: None, draws: Vec::with_capacity(128), tris: Vec::with_capacity(4096), posts: Vec::with_capacity(MAX_POST) },
            inst_scratch: Vec::with_capacity(INST_BYTES as usize * MAX_INST),
            tri_scratch: Vec::with_capacity(TRI_BYTES as usize * 4096),
            trails: Vec::with_capacity(TRAILS),
            snap: [None, None],
            eff: Vec::with_capacity(128),
            times: FxTimes::default(),
        };
        g.world_pipelines(device, samples);
        g
    }

    fn world_pipelines(&mut self, device: &wgpu::Device, samples: u32) {
        if self.world_samples == samples && !self.world_pipes.is_empty() {
            return;
        }
        let mesh_attrs = [
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 0, shader_location: 0 },
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 12, shader_location: 1 },
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 24, shader_location: 2 },
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 36, shader_location: 3 },
        ];
        let inst_attrs: [wgpu::VertexAttribute; 6] = std::array::from_fn(|i| wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: i as u64 * 16, shader_location: 4 + i as u32 });
        let tri_attrs = [
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 0, shader_location: 0 },
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Unorm8x4, offset: 12, shader_location: 1 },
        ];
        let add = wgpu::BlendState { color: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add }, alpha: wgpu::BlendComponent::REPLACE };
        let mk = |label: &str, tri: bool, blend: Option<wgpu::BlendState>, emis_blend: Option<wgpu::BlendState>, zwrite: bool| {
            let bufs_mesh = [
                wgpu::VertexBufferLayout { array_stride: VERT_BYTES, step_mode: wgpu::VertexStepMode::Vertex, attributes: &mesh_attrs },
                wgpu::VertexBufferLayout { array_stride: INST_BYTES, step_mode: wgpu::VertexStepMode::Instance, attributes: &inst_attrs },
            ];
            let bufs_tri = [wgpu::VertexBufferLayout { array_stride: TRI_BYTES, step_mode: wgpu::VertexStepMode::Vertex, attributes: &tri_attrs }];
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&self.world_layout),
                vertex: wgpu::VertexState { module: &self.world_module, entry_point: Some(if tri { "vs_tri" } else if label == "fx ink" { "vs_ink" } else { "vs_mesh" }), compilation_options: Default::default(), buffers: if tri { &bufs_tri } else { &bufs_mesh } },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: zwrite,
                    // reversed z (state::USE_REVERSED_Z): GX LEQUAL
                    depth_compare: if crate::state::USE_REVERSED_Z { wgpu::CompareFunction::GreaterEqual } else { wgpu::CompareFunction::LessEqual },
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: wgpu::MultisampleState { count: samples, ..Default::default() },
                fragment: Some(wgpu::FragmentState {
                    module: &self.world_module,
                    entry_point: Some(if tri { "fs_tri" } else if label == "fx ink" { "fs_ink" } else { "fs_mesh" }),
                    compilation_options: Default::default(),
                    targets: &[
                        Some(wgpu::ColorTargetState { format: COLOR_FORMAT, blend, write_mask: wgpu::ColorWrites::COLOR }),
                        Some(wgpu::ColorTargetState { format: COLOR_FORMAT, blend: emis_blend, write_mask: wgpu::ColorWrites::COLOR }),
                    ],
                }),
                multiview: None,
                cache: None,
            })
        };
        self.world_pipes = vec![
            mk("fx opaque", false, None, None, true),
            mk("fx blend", false, Some(wgpu::BlendState::ALPHA_BLENDING), Some(add), false),
            mk("fx add", false, Some(add), Some(add), false),
            mk("fx tris", true, Some(add), Some(add), false),
            mk("fx ink", false, None, None, true),
        ];
        self.world_samples = samples;
    }

    fn post_pipeline(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat) -> usize {
        if let Some(i) = self.post_pipes.iter().position(|p| p.0 == format) {
            return i;
        }
        let p = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fx post"),
            layout: Some(&self.post_layout),
            vertex: wgpu::VertexState { module: &self.post_module, entry_point: Some("vs_main"), compilation_options: Default::default(), buffers: &[] },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState { module: &self.post_module, entry_point: Some("fs_main"), compilation_options: Default::default(), targets: &[Some(wgpu::ColorTargetState { format, blend: None, write_mask: wgpu::ColorWrites::ALL })] }),
            multiview: None,
            cache: None,
        });
        self.post_pipes.push((format, p));
        self.post_pipes.len() - 1
    }

    fn targets(&mut self, device: &wgpu::Device, w: u32, h: u32, samples: u32) {
        if self.emis.as_ref().is_some_and(|t| t.size == (w, h, samples)) {
            return;
        }
        let mk = |label, w: u32, h: u32, n: u32, usage| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d { width: w.max(1), height: h.max(1), depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: n,
                dimension: wgpu::TextureDimension::D2,
                format: COLOR_FORMAT,
                usage,
                view_formats: &[],
            })
        };
        let rt = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        let emis = mk("fx emissive", w, h, 1, rt);
        let emis_msaa = (samples > 1).then(|| mk("fx emissive (MSAA)", w, h, samples, wgpu::TextureUsages::RENDER_ATTACHMENT));
        let half = [mk("fx bloom a", w / 2, h / 2, 1, rt), mk("fx bloom b", w / 2, h / 2, 1, rt)];
        let bg = |t: &wgpu::Texture| {
            let v = t.create_view(&Default::default());
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("fx blur"),
                layout: &self.blur_bgl,
                entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&v) }, wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.lin) }],
            })
        };
        let bgs = [bg(&emis), bg(&half[0])];
        self.emis = Some(Targets { size: (w, h, samples), emis, emis_msaa, half, bgs });
    }

    fn now(&self) -> f32 {
        (self.clock.elapsed().as_secs_f64() % 3600.0) as f32
    }

    /// Whether the post pass has anything to show (else the plain blit runs).
    fn post_active(&self) -> bool {
        self.armed && self.intensity > 0.0 && (self.bloom_live || self.held.iter().any(|h| h.id != 0 && h.cur > 0.002) || self.shots.iter().any(|s| s.id != 0))
    }

    /// The post uniform for now (`aspect`: the EFB's width / height).
    fn post_uniform(&self, aspect: f32) -> Vec<u8> {
        let t = self.now();
        let k = self.intensity;
        let mut heat = [0.0f32; 4];
        let mut tint = [1.0, 0.55, 0.2, 1.0];
        let mut best = 0.0;
        for h in self.held.iter().filter(|h| h.id != 0) {
            let s = h.cur * k;
            match h.id {
                POST_VIGNETTE => heat[0] += s * 0.55,
                POST_EMBER => heat[1] += s * 0.5,
                POST_HEAT_HAZE => {
                    heat[2] += s;
                    heat[3] += s * 0.9;
                }
                _ => {}
            }
            if h.cur > best && h.rgba & 0xFF != 0 {
                best = h.cur;
                tint = rgba_f(h.rgba);
            }
        }
        let mut b = Vec::with_capacity(POST_UBYTES);
        put4(&mut b, [t, k, aspect, if self.bloom_live { 1.15 * k } else { 0.0 }]);
        put4(&mut b, heat);
        put4(&mut b, tint);
        let mut flash = [1.0f32, 1.0, 1.0, 0.0];
        let mut waves = [[0.0f32; 8]; 4];
        let mut wcol = [[0.0f32; 4]; 4];
        for (i, s) in self.shots.iter().enumerate().filter(|(_, s)| s.id != 0) {
            let a = ((t - s.t0) / life(s.id)).clamp(0.0, 1.0);
            let fade = 1.0 - a;
            let c = rgba_f(s.rgba);
            match s.id {
                POST_FLASH => {
                    flash = [c[0], c[1], c[2], CAP * s.strength * fade * fade * k];
                }
                POST_RADIAL_WAVE | POST_SHOCKWAVE | POST_CHROMA => {
                    let big = s.id == POST_SHOCKWAVE;
                    let (rad, wid, amp) = if big { (0.9, 0.07, 0.022) } else if s.id == POST_CHROMA { (0.5, 0.25, 0.0) } else { (0.55, 0.045, 0.014) };
                    waves[i] = [s.uv[0], s.uv[1], ease(a) * rad, wid, amp * s.strength * fade * k, 0.012 * s.strength * fade * k, 0.25 * s.strength * fade * k, 0.0];
                    wcol[i] = [c[0], c[1], c[2], 1.0];
                }
                _ => {}
            }
        }
        put4(&mut b, flash);
        for w in waves {
            put4(&mut b, [w[0], w[1], w[2], w[3]]);
            put4(&mut b, [w[4], w[5], w[6], w[7]]);
        }
        for c in wcol {
            put4(&mut b, c);
        }
        b.resize(POST_UBYTES, 0);
        b
    }
}

/// The GX projection rows for `cam` (reversed z as the GX shaders have it; x widened for true widescreen).
fn proj_rows(cam: &Camera, widen: f32) -> [[f32; 4]; 4] {
    let p = cam.proj;
    let mut m = if p[0] as u32 == 1 {
        // GX_ORTHOGRAPHIC
        [[p[1], 0.0, 0.0, p[2]], [0.0, p[3], 0.0, p[4]], [0.0, 0.0, p[5], p[6]], [0.0, 0.0, 0.0, 1.0]]
    } else {
        [[p[1], 0.0, p[2], 0.0], [0.0, p[3], p[4], 0.0], [0.0, 0.0, p[5], p[6]], [0.0, 0.0, -1.0, 0.0]]
    };
    if crate::state::USE_REVERSED_Z {
        for x in m[2].iter_mut() {
            *x = -*x;
        }
    }
    for x in m[0].iter_mut() {
        *x *= widen;
    }
    m
}

/// A world point's EFB uv through `cam` (None: behind the camera).
pub fn project(cam: &Camera, widen: f32, w: [f32; 3]) -> Option<[f32; 2]> {
    let h = [w[0], w[1], w[2], 1.0];
    let dot = |r: &[f32; 4], v: &[f32; 4]| r[0] * v[0] + r[1] * v[1] + r[2] * v[2] + r[3] * v[3];
    let vp = [dot(&cam.view[0], &h), dot(&cam.view[1], &h), dot(&cam.view[2], &h), 1.0];
    let m = proj_rows(cam, widen);
    let c = [dot(&m[0], &vp), dot(&m[1], &vp), dot(&m[3], &vp)];
    if c[2] <= 1e-5 {
        return None;
    }
    let (nx, ny) = (c[0] / c[2], c[1] / c[2]);
    // the viewport (logical 640x480) -> uv
    let v = cam.vp;
    let (vw, vh) = if v[2] > 0.0 && v[3] > 0.0 { (v[2], v[3]) } else { (640.0, 480.0) };
    Some([(v[0] + (nx * 0.5 + 0.5) * vw) / 640.0, (v[1] + (0.5 - ny * 0.5) * vh) / 480.0])
}

/// The model matrix rows of a drive (drive_draw's `rotator`: yaw, then a 0.35 rad pitch; uniform scale).
fn model_rows(d: &WorldDraw) -> [[f32; 4]; 3] {
    let (sy, cy) = d.yaw.sin_cos();
    let (sp, cp) = 0.35f32.sin_cos();
    let r = [[cy, 0.0, sy], [sy * sp, cp, -cy * sp], [-sy * cp, sp, cy * cp]];
    let s = d.scale;
    std::array::from_fn(|i| [r[i][0] * s, r[i][1] * s, r[i][2] * s, d.at[i]])
}

impl Renderer {
    /// Whether the effects layer is armed.
    pub fn fx_armed(&self) -> bool {
        self.fx.as_ref().is_some_and(|f| f.armed)
    }

    /// Arm the layer (a run starts): on the first call creates every buffer and pipeline (the post pipeline for
    /// the formats seen so far and RGBA8, the world ones for the current MSAA) so nothing compiles mid-fight; later
    /// calls only re-enable it. `meshes`: the drive models (uploaded once).
    pub fn fx_arm(&mut self, meshes: &[FxMesh<'_>]) {
        if let Some(f) = self.fx.as_mut() {
            f.armed = true;
            return;
        }
        let mut f = FxGpu::new(&self.device, &self.queue, meshes, self.samples);
        let mut fmts: Vec<wgpu::TextureFormat> = self.blit.keys().copied().collect();
        fmts.push(wgpu::TextureFormat::Rgba8Unorm);
        for fmt in fmts {
            f.post_pipeline(&self.device, fmt);
        }
        f.targets(&self.device, self.width, self.height, self.samples);
        self.fx = Some(Box::new(f));
    }

    /// Disarm (the run ended): every pass is skipped from the next present; the GPU objects are kept.
    pub fn fx_disarm(&mut self) {
        if let Some(f) = self.fx.as_mut() {
            f.armed = false;
            f.bloom_live = false;
            f.held = [Held::default(); MAX_POST];
            f.shots = [Shot::default(); MAX_POST];
            f.trails.clear();
            f.snap = [None, None];
        }
    }

    /// The world list just handed over by `set_fx`, for the presenter to keep with the GX frame of the same tick.
    pub fn fx_snapshot(&self) -> Option<Box<FxSnap>> {
        let f = self.fx.as_ref().filter(|f| f.armed)?;
        Some(Box::new(FxSnap { draws: f.frame.draws.clone(), tris: f.frame.tris.clone(), cam: f.frame.cam }))
    }

    /// The presenter adopted a GX frame: its world list (None: none was kept, draw the latest) becomes current.
    pub fn fx_adopt(&mut self, s: Option<Box<FxSnap>>) {
        if let Some(f) = self.fx.as_mut().filter(|f| f.armed) {
            f.snap[0] = f.snap[1].take();
            f.snap[1] = s;
        }
    }

    /// The effect times measured so far (MELEE_FX_PROF=1).
    pub fn fx_times(&self) -> FxTimes {
        self.fx.as_ref().map_or_else(FxTimes::default, |f| f.times)
    }

    /// Hand over this present's inputs (the vectors are swapped with the layer's own, so `frame` comes back
    /// holding last present's buffers, cleared). Nothing happens while disarmed.
    pub fn set_fx(&mut self, frame: &mut FxFrame) {
        let (aspect, widen) = (self.width as f32 / self.height.max(1) as f32, self.fx_widen);
        let Some(f) = self.fx.as_mut().filter(|f| f.armed) else {
            frame.draws.clear();
            frame.tris.clear();
            frame.posts.clear();
            return;
        };
        let now = f.now();
        let dt = (now - f.last_t).clamp(0.0, 0.25);
        f.last_t = now;
        f.intensity = frame.intensity.clamp(0.0, 1.0);
        std::mem::swap(&mut f.frame.draws, &mut frame.draws);
        std::mem::swap(&mut f.frame.tris, &mut frame.tris);
        std::mem::swap(&mut f.frame.posts, &mut frame.posts);
        f.frame.cam = frame.cam;
        frame.draws.clear();
        frame.tris.clear();
        frame.posts.clear();
        // posts: held ones refresh their slot, one-shots start (flashes rate-capped, none at intensity 0)
        let cam = f.frame.cam;
        for i in 0..f.frame.posts.len() {
            let p = f.frame.posts[i];
            let id = p.id & 0xFF;
            let uv = if p.id & POST_WORLD != 0 {
                match cam.and_then(|c| project(&c, widen, [p.x, p.y, 0.0])) {
                    Some(uv) => uv,
                    None => continue,
                }
            } else {
                [0.5 + (p.x / 640.0 - 0.5) * (4.0 / 3.0) / aspect, p.y / 480.0]
            };
            let live = f.held.iter().filter(|h| h.id != 0).count() + f.shots.iter().filter(|s| s.id != 0).count();
            if (POST_VIGNETTE..=POST_HEAT_HAZE).contains(&id) {
                if let Some(h) = f.held.iter_mut().find(|h| h.id == id) {
                    h.target = p.strength;
                    h.seen = now;
                    h.rgba = p.rgba;
                } else if live < MAX_POST {
                    if let Some(h) = f.held.iter_mut().find(|h| h.id == 0) {
                        *h = Held { id, cur: 0.0, target: p.strength, seen: now, rgba: p.rgba };
                    }
                }
            } else {
                if f.intensity <= 0.0 {
                    continue;
                }
                if id == POST_FLASH {
                    if now - f.last_flash < 1.0 / FLASH_PER_SEC {
                        continue;
                    }
                    f.last_flash = now;
                }
                if live < MAX_POST {
                    if let Some(s) = f.shots.iter_mut().find(|s| s.id == 0) {
                        *s = Shot { id, t0: now, uv, strength: p.strength, rgba: p.rgba };
                    }
                }
            }
        }
        // held: ease toward the target (0 once not re-posted for a quarter second); one-shots expire
        for h in f.held.iter_mut().filter(|h| h.id != 0) {
            if now - h.seen > 0.25 {
                h.target = 0.0;
            }
            h.cur += (h.target - h.cur) * (dt / 0.35).min(1.0);
            if h.target == 0.0 && h.cur < 0.002 {
                *h = Held::default();
            }
        }
        for s in f.shots.iter_mut().filter(|s| s.id != 0) {
            if now - s.t0 > life(s.id) {
                *s = Shot::default();
            }
        }
        // trails: the last positions of each trailed draw (in list order), sampled at present time
        let mut k = 0;
        let n = f.frame.draws.len();
        for i in 0..n {
            let d = f.frame.draws[i];
            if d.mat & MAT_TRAIL == 0 || k >= TRAILS {
                continue;
            }
            if f.trails.len() <= k {
                f.trails.push(([d.at; TRAIL_LEN], 0, now));
            }
            let tr = &mut f.trails[k];
            let last = tr.0[(tr.1 + TRAIL_LEN - 1) % TRAIL_LEN];
            let jump = (0..3).map(|j| (d.at[j] - last[j]).powi(2)).sum::<f32>().sqrt();
            if jump > d.scale * 6.0 {
                *tr = ([d.at; TRAIL_LEN], 0, now);
            } else if now - tr.2 >= 1.0 / 60.0 {
                tr.0[tr.1] = d.at;
                tr.1 = (tr.1 + 1) % TRAIL_LEN;
                tr.2 = now;
            }
            k += 1;
        }
        f.trails.truncate(k);
    }

    /// The world layer: drives and fire shapes into the EFB (after its last pass), then the bloom. Called at the
    /// end of `draw_prepared`; does nothing unless armed with something to draw.
    pub(crate) fn fx_world(&mut self) {
        let (w, h, samples) = (self.width, self.height, self.samples);
        let widen = self.fx_widen;
        let phase = self.fx_phase;
        let Some(f) = self.fx.as_mut().filter(|f| f.armed) else { return };
        // the list drawn: the adopted tick's, posed at this present's phase (the scene under it is); none kept (no
        // interpolating presenter): the latest
        let mut eff = std::mem::take(&mut f.eff);
        eff.clear();
        let cam = match f.snap[1].as_deref() {
            Some(s) => {
                eff.extend_from_slice(&s.draws);
                if let (Some(ph), Some(p)) = (phase, f.snap[0].as_deref()) {
                    // pair by list position and model; a teleport (a respawn, a tag) snaps
                    if p.draws.len() == s.draws.len() {
                        let k = if ph.extrapolate { ph.t } else { ph.t - 1.0 };
                        for (d, q) in eff.iter_mut().zip(&p.draws) {
                            let dl = [d.at[0] - q.at[0], d.at[1] - q.at[1], d.at[2] - q.at[2]];
                            if d.model != q.model || dl.iter().any(|x| x.abs() > FX_MAX_STEP) {
                                continue;
                            }
                            for j in 0..3 {
                                d.at[j] += dl[j] * k;
                            }
                            let mut dy = d.yaw - q.yaw;
                            dy = (dy + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
                            d.yaw += dy * k;
                        }
                    }
                }
                s.cam
            }
            None => {
                eff.extend_from_slice(&f.frame.draws);
                f.frame.cam
            }
        };
        let cam = cam.map(|mut c| {
            if let Some(v) = phase.and_then(|p| p.view) {
                c.view = v;
            }
            c
        });
        let tris = std::mem::take(match f.snap[1].as_mut() {
            Some(s) => &mut s.tris,
            None => &mut f.frame.tris,
        });
        let Some(cam) = cam.filter(|_| !eff.is_empty() || !tris.is_empty()) else {
            f.bloom_live = false;
            f.eff = eff;
            match f.snap[1].as_mut() {
                Some(s) => s.tris = tris,
                None => f.frame.tris = tris,
            }
            return;
        };
        let t0 = std::time::Instant::now();
        f.world_pipelines(&self.device, samples);
        f.targets(&self.device, w, h, samples);
        // globals: view rows, projection rows, time / intensity, the glass material
        let mut gb = Vec::with_capacity(16 * 9);
        for r in cam.view {
            put4(&mut gb, r);
        }
        for r in proj_rows(&cam, widen) {
            put4(&mut gb, r);
        }
        // misc: time, intensity, the ink width in NDC units (x, y) at this target size
        let (vw, vh) = if cam.vp[2] > 0.0 { (cam.vp[2] * w as f32 / 640.0, cam.vp[3] * h as f32 / 480.0) } else { (w as f32, h as f32) };
        let ink_px = INK_PX * h as f32 / 480.0;
        put4(&mut gb, [f.now(), f.intensity, 2.0 * ink_px / vw.max(1.0), 2.0 * ink_px / vh.max(1.0)]);
        let gl = glass_params();
        put4(&mut gb, [gl.0, gl.1, gl.2, 0.0]);
        self.queue.write_buffer(&f.globals, 0, &gb);
        // instances: per pipeline class (opaque, blend, add), then the trails as additive ghosts
        let mut lists: [Vec<(u32, u32, u32)>; 3] = [Vec::new(), Vec::new(), Vec::new()];
        let mut ink: Vec<(u32, u32, u32)> = Vec::new();
        f.inst_scratch.clear();
        let mut ni = 0u32;
        let push = |b: &mut Vec<u8>, d: &WorldDraw, seed: f32| {
            for r in model_rows(d) {
                put4(b, r);
            }
            put4(b, d.p);
            put4(b, rgba_f(d.rgba));
            put4(b, [(d.mat & MAT_ID) as f32, d.alpha, seed, 0.0]);
        };
        for (i, d) in eff.iter().enumerate() {
            let Some(&(start, count)) = f.ranges.get(d.model as usize) else { continue };
            if count == 0 || ni as usize >= MAX_INST {
                continue;
            }
            let mat = d.mat & MAT_ID;
            let class = if mat == MAT_UNLIT_ADD { 2 } else if mat == MAT_GLASS || d.alpha < 0.99 { 1 } else { 0 };
            push(&mut f.inst_scratch, d, i as f32 * 1.7);
            lists[class].push((start, count, ni));
            if d.mat & MAT_OUTLINE != 0 && class != 2 {
                ink.push((start, count, ni));
            }
            ni += 1;
        }
        let mut k = 0;
        for d in eff.iter() {
            if d.mat & MAT_TRAIL == 0 || k >= f.trails.len() {
                continue;
            }
            let Some(&(start, count)) = f.ranges.get(d.model as usize) else { continue };
            let tr = &f.trails[k];
            k += 1;
            for j in 1..TRAIL_LEN {
                if ni as usize >= MAX_INST {
                    break;
                }
                // oldest first, shrinking and fading toward the tail
                let p = tr.0[(tr.1 + j) % TRAIL_LEN];
                let age = 1.0 - j as f32 / TRAIL_LEN as f32;
                let ghost = WorldDraw { model: d.model, mat: MAT_UNLIT_ADD, at: p, yaw: d.yaw, scale: d.scale * (0.85 - 0.5 * age), alpha: (1.0 - age) * 0.22, p: [0.0; 4], rgba: d.rgba | 0xFF };
                push(&mut f.inst_scratch, &ghost, 0.0);
                lists[2].push((start, count, ni));
                ni += 1;
            }
        }
        if !f.inst_scratch.is_empty() {
            self.queue.write_buffer(&f.inst, 0, &f.inst_scratch);
        }
        f.tri_scratch.clear();
        let nt = tris.len().min(MAX_TRI_VERTS) / 3 * 3;
        for (p, c) in &tris[..nt] {
            for x in p {
                f.tri_scratch.extend_from_slice(&x.to_le_bytes());
            }
            f.tri_scratch.extend_from_slice(c);
        }
        if !f.tri_scratch.is_empty() {
            self.queue.write_buffer(&f.tri, 0, &f.tri_scratch);
        }
        f.eff = eff;
        match f.snap[1].as_mut() {
            Some(s) => s.tris = tris,
            None => f.frame.tris = tris,
        }
        let tg = f.emis.as_ref().unwrap();
        let color_view = self.color.create_view(&Default::default());
        let depth_view = self.depth.create_view(&Default::default());
        let msaa_view = self.msaa.as_ref().map(|t| t.create_view(&Default::default()));
        let emis_view = tg.emis.create_view(&Default::default());
        let emis_msaa_view = tg.emis_msaa.as_ref().map(|t| t.create_view(&Default::default()));
        let mut enc = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fx world"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: msaa_view.as_ref().unwrap_or(&color_view),
                        resolve_target: msaa_view.as_ref().map(|_| &color_view),
                        depth_slice: None,
                        ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: emis_msaa_view.as_ref().unwrap_or(&emis_view),
                        resolve_target: emis_msaa_view.as_ref().map(|_| &emis_view),
                        depth_slice: None,
                        ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                    }),
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            // the fight camera's viewport (logical pixels -> target, as record_draws maps GX viewports)
            let (kx, ky) = (w as f32 / 640.0, h as f32 / 480.0);
            let v = cam.vp;
            if v[2] > 0.0 && v[3] > 0.0 {
                let left = (v[0] * kx).clamp(0.0, w as f32);
                let top = (v[1] * ky).clamp(0.0, h as f32);
                pass.set_viewport(left, top, (v[2] * kx).min(w as f32 - left).max(1.0), (v[3] * ky).min(h as f32 - top).max(1.0), (1.0 - v[5]).clamp(0.0, 1.0), (1.0 - v[4]).clamp(0.0, 1.0));
            }
            pass.set_bind_group(0, &f.world_bg, &[]);
            pass.set_vertex_buffer(0, f.vbuf.slice(..));
            pass.set_vertex_buffer(1, f.inst.slice(..));
            // ink edges first: each tagged body's screen-space hull, then the body over it at the same depth
            if !ink.is_empty() {
                pass.set_pipeline(&f.world_pipes[4]);
                for &(start, count, i) in &ink {
                    pass.draw(start..start + count, i..i + 1);
                }
            }
            for (class, list) in lists.iter().enumerate() {
                if list.is_empty() {
                    continue;
                }
                pass.set_pipeline(&f.world_pipes[class]);
                for &(start, count, i) in list {
                    pass.draw(start..start + count, i..i + 1);
                }
            }
            if nt > 0 {
                pass.set_pipeline(&f.world_pipes[3]);
                pass.set_vertex_buffer(0, f.tri.slice(..));
                pass.draw(0..nt as u32, 0..1);
            }
        }
        // bloom: emissive (full) -> half A (horizontal) -> half B (vertical); the blit adds B
        let half_views = [tg.half[0].create_view(&Default::default()), tg.half[1].create_view(&Default::default())];
        for (i, pipe) in f.blur_pipes.iter().enumerate() {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fx bloom"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &half_views[i], resolve_target: None, depth_slice: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store } })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(pipe);
            pass.set_bind_group(0, &tg.bgs[i], &[]);
            pass.draw(0..3, 0..1);
        }
        self.queue.submit([enc.finish()]);
        f.bloom_live = f.intensity > 0.0;
        if prof() {
            let _ = self.device.poll(wgpu::PollType::wait_indefinitely());
            f.times.world_ms = t0.elapsed().as_secs_f32() * 1e3;
        }
    }

    /// The post pass for this blit, when one is active: (pipeline index, bind group).
    pub(crate) fn fx_post(&mut self, format: wgpu::TextureFormat, efb: &wgpu::TextureView, sampler: &wgpu::Sampler) -> Option<(usize, wgpu::BindGroup)> {
        let aspect = self.width as f32 / self.height.max(1) as f32;
        let f = self.fx.as_mut().filter(|f| f.post_active())?;
        let pi = f.post_pipeline(&self.device, format);
        let ub = f.post_uniform(aspect);
        self.queue.write_buffer(&f.post_ubuf, 0, &ub);
        let bloom = f.emis.as_ref().filter(|_| f.bloom_live).map(|t| t.half[1].create_view(&Default::default()));
        let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fx post"),
            layout: &f.post_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(efb) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(sampler) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(bloom.as_ref().unwrap_or(&f.black)) },
                wgpu::BindGroupEntry { binding: 3, resource: f.post_ubuf.as_entire_binding() },
            ],
        });
        Some((pi, bg))
    }

    pub(crate) fn fx_post_pipe(&self, i: usize) -> Option<&wgpu::RenderPipeline> {
        self.fx.as_ref().and_then(|f| f.post_pipes.get(i)).map(|p| &p.1)
    }
}

fn prof() -> bool {
    static P: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *P.get_or_init(|| std::env::var("MELEE_FX_PROF").is_ok_and(|v| v == "1"))
}

/// The glass material (pc/scripts/examples/envoy_drives/models/drive_glass.material.json): (opacity, fresnel
/// exponent from roughness, tint brightness).
pub fn glass_params() -> (f32, f32, f32) {
    static G: std::sync::OnceLock<(f32, f32, f32)> = std::sync::OnceLock::new();
    *G.get_or_init(|| {
        let j = include_str!("../../../pc/scripts/examples/envoy_drives/models/drive_glass.material.json");
        let num = |key: &str| -> Option<f32> {
            let at = j.find(&format!("\"{key}\""))? + key.len() + 2;
            let rest = j[at..].trim_start_matches([' ', ':']);
            let end = rest.find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-')).unwrap_or(rest.len());
            rest[..end].parse().ok()
        };
        let tint = j.find("\"tint\"").and_then(|i| {
            let r = &j[i..];
            let a = r.find('[')? + 1;
            let b = r.find(']')?;
            let v: Vec<f32> = r[a..b].split(',').filter_map(|x| x.trim().parse().ok()).collect();
            (v.len() >= 3).then(|| (v[0] + v[1] + v[2]) / 3.0)
        });
        let opacity = num("opacity").unwrap_or(0.28).clamp(0.05, 1.0);
        let rough = num("roughness").unwrap_or(0.4).clamp(0.0, 1.0);
        (opacity, 1.5 + (1.0 - rough) * 3.0, tint.unwrap_or(1.0).clamp(0.0, 2.0))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn post_queue_caps_and_clamps() {
        let mut out = Vec::new();
        take_posts(&mut out);
        assert!(!post(0, 1.0, 0.0, 0.0, 0));
        assert!(!post(POST_COUNT, 1.0, 0.0, 0.0, 0));
        assert!(!post(POST_FLASH, f32::NAN, 0.0, 0.0, 0));
        for _ in 0..MAX_POST {
            assert!(post(POST_RADIAL_WAVE | POST_WORLD, 3.0, 1.0, 2.0, 0xFFFF_FFFF));
        }
        assert!(!post(POST_FLASH, 1.0, 0.0, 0.0, 0));
        take_posts(&mut out);
        assert_eq!(out.len(), MAX_POST);
        assert_eq!(out[0].strength, 1.0);
        assert!(!posts_pending());
    }

    #[test]
    fn glass_json_is_read() {
        let (o, e, t) = glass_params();
        assert!((o - 0.28).abs() < 1e-6, "{o}");
        assert!((e - (1.5 + 0.6 * 3.0)).abs() < 1e-5, "{e}");
        assert!((t - 1.0).abs() < 1e-6);
    }

    #[test]
    fn projection_centres_the_look_point() {
        // a camera at z = 100 looking down -z: the origin lands mid-screen
        let cam = Camera { view: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, -100.0]], proj: [0.0, 1.0, 0.0, 1.0, 0.0, -0.01, -1.0], vp: [0.0, 0.0, 640.0, 480.0, 0.0, 1.0] };
        let uv = project(&cam, 1.0, [0.0, 0.0, 0.0]).unwrap();
        assert!((uv[0] - 0.5).abs() < 1e-5 && (uv[1] - 0.5).abs() < 1e-5, "{uv:?}");
        let up = project(&cam, 1.0, [0.0, 10.0, 0.0]).unwrap();
        assert!(up[1] < 0.5);
    }
}
