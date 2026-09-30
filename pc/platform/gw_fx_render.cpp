// gw_fx_render.cpp - Geno effects runtime, part 2: drawing (design: workspace _research/geno-effects-runtime.md 4).
//
// Through Aurora's public extension API only (aurora/gfx.hpp): no Aurora or shim_gx change.
//   - push_custom_draw: our own WebGPU pipelines, recorded into the EFB pass at the point the game calls us (the
//     Geno effects GObj, gx link 8 after Melee's own particles: geno_game_articles.inc) and replayed on the render
//     worker with the pass's layout (MSAA, the normal attachment...).
//   - resolve_pass: the frame copy (distortion) and the depth snapshot (the bloom's occlusion), only on frames that
//     draw a material that needs them.
//   - create_pass / resolve_pass: the effect-only bloom's buffers (half resolution, then a quarter-resolution blur
//     in two passes), composited additively over the EFB. Melee's own world never enters the bloom buffer.
//
// A frame with no live Geno particle does nothing here (the GObj does not even exist in a vanilla match), so the
// vanilla picture and frame time are untouched.
//
// The shader library is ONE WGSL module (sprite / warp / distortion as a uniform switch, the colour / alpha
// texture masks from the effect IR's material.shader); one pipeline per (pass kind, blend, depth test, target
// layout). Textures are the package's PNGs, decoded with WIC (the swizzle applied on the CPU) at the first draw
// and uploaded on the render worker.
#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#ifdef _WIN32
#include <windows.h>
#include <wincodec.h>
#else
#include "gw_compat_linux.h"
#include <png.h>
#endif

#include <aurora/gfx.hpp>
#include <webgpu/webgpu_cpp.h>

#include <algorithm>
#include <cmath>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <mutex>
#include <string>
#include <unordered_map>
#include <vector>

extern "C" {
#include "gw.h"
#include "gw_fx_internal.h"
#include <dolphin/gx.h>
}


namespace {

using aurora::gfx::Range;

// ---- textures (decoded on the game thread, uploaded on the render worker) -------------------------------------
struct FxTex {
  int state = 0; // 0 not loaded, 1 decoded, -1 failed
  uint32_t w = 0, h = 0;
  std::vector<uint8_t> rgba;
  wgpu::Texture tex;
  wgpu::TextureView view;
};
std::vector<std::vector<FxTex>> g_tex; // [pkg][tex]

#ifdef _WIN32
const GUID kClsidWic = {0xcacaf262, 0x9370, 0x4615, {0xa1, 0x3b, 0x9f, 0x55, 0x39, 0xda, 0x4c, 0x0a}};
const GUID kFmtRGBA = {0xf5c7ad2d, 0x6a8d, 0x43dd, {0xa7, 0xa8, 0xa2, 0x99, 0x35, 0x26, 0x1a, 0xe9}};
#endif

bool decode_png(const char* path, const char* swz, FxTex& t) {
#ifdef _WIN32
  IWICImagingFactory* f = nullptr;
  IWICBitmapDecoder* dec = nullptr;
  IWICBitmapFrameDecode* fr = nullptr;
  IWICFormatConverter* cv = nullptr;
  bool ok = false;
  wchar_t wpath[520];
  CoInitializeEx(nullptr, COINIT_MULTITHREADED); // S_FALSE / RPC_E_CHANGED_MODE: already initialised - fine
  if (MultiByteToWideChar(CP_UTF8, 0, path, -1, wpath, 520) <= 0) return false;
  if (FAILED(CoCreateInstance(kClsidWic, nullptr, CLSCTX_INPROC_SERVER, __uuidof(IWICImagingFactory), (void**)&f)))
    return false;
  if (SUCCEEDED(f->CreateDecoderFromFilename(wpath, nullptr, GENERIC_READ, WICDecodeMetadataCacheOnDemand, &dec)) &&
      SUCCEEDED(dec->GetFrame(0, &fr)) && SUCCEEDED(f->CreateFormatConverter(&cv)) &&
      SUCCEEDED(cv->Initialize(fr, kFmtRGBA, WICBitmapDitherTypeNone, nullptr, 0.0, WICBitmapPaletteTypeCustom))) {
    UINT w = 0, h = 0;
    cv->GetSize(&w, &h);
    std::vector<uint8_t> src(size_t(w) * h * 4);
    if (w && h && SUCCEEDED(cv->CopyPixels(nullptr, w * 4, UINT(src.size()), src.data()))) {
      // the effect IR's swizzle: which source channel feeds r, g, b, a ('0' / '1' constants)
      int sel[4];
      for (int c = 0; c < 4; ++c) {
        const char s = swz[c];
        sel[c] = s == 'r' ? 0 : s == 'g' ? 1 : s == 'b' ? 2 : s == 'a' ? 3 : s == '0' ? 4 : 5;
      }
      t.rgba.resize(src.size());
      for (size_t i = 0; i < size_t(w) * h; ++i)
        for (int c = 0; c < 4; ++c)
          t.rgba[i * 4 + c] = sel[c] < 4 ? src[i * 4 + sel[c]] : sel[c] == 4 ? 0 : 255;
      t.w = w;
      t.h = h;
      ok = true;
    }
  }
  if (cv) cv->Release();
  if (fr) fr->Release();
  if (dec) dec->Release();
  f->Release();
  return ok;
#else
  png_image image{};
  std::string native_path(path);
  for (char& c : native_path) if (c == '\\') c = '/';
  image.version = PNG_IMAGE_VERSION;
  if (!png_image_begin_read_from_file(&image, native_path.c_str())) return false;
  // The host is 32-bit: bound dimensions before libpng's size macro or allocation.
  if (!image.width || !image.height || image.width > 16384 || image.height > 16384 ||
      uint64_t(image.width) * image.height * 4 > 256u * 1024u * 1024u) {
    png_image_free(&image);
    return false;
  }
  image.format = PNG_FORMAT_RGBA;
  std::vector<uint8_t> src(PNG_IMAGE_SIZE(image));
  const bool ok = png_image_finish_read(&image, nullptr, src.data(), 0, nullptr) != 0;
  if (ok && image.width && image.height) {
    int sel[4];
    for (int c = 0; c < 4; ++c) {
      const char s = swz[c];
      sel[c] = s == 'r' ? 0 : s == 'g' ? 1 : s == 'b' ? 2 : s == 'a' ? 3 : s == '0' ? 4 : 5;
    }
    t.rgba.resize(src.size());
    for (size_t i = 0; i < size_t(image.width) * image.height; ++i)
      for (int c = 0; c < 4; ++c)
        t.rgba[i * 4 + c] = sel[c] < 4 ? src[i * 4 + sel[c]] : sel[c] == 4 ? 0 : 255;
    t.w = image.width;
    t.h = image.height;
  } else if (ok) {
    png_image_free(&image);
    return false;
  }
  png_image_free(&image);
  return ok;
#endif
}

void ensure_textures(int pk) {
  const fx_pkg* p = gw_fx_pkg(pk);
  if (p == nullptr) return;
  if (g_tex.empty()) g_tex.resize(FX_MAX_PKGS); // never resized again: the render worker reads it
  if (pk >= FX_MAX_PKGS) return;
  auto& v = g_tex[pk];
  if (!v.empty()) return;
  v.resize(p->ntex);
  int ok = 0;
  for (int i = 0; i < p->ntex; ++i) {
    std::string path = std::string(p->dir) + "\\" + p->tex_file[i];
    for (auto& ch : path)
      if (ch == '/') ch = '\\';
    v[i].state = decode_png(path.c_str(), p->tex_swizzle[i], v[i]) ? 1 : -1;
    ok += v[i].state == 1;
    if (v[i].state < 0) gw_log("fx: %s: texture %s did not decode", p->name, path.c_str());
  }
  gw_log("fx: %s: %d/%d textures decoded", p->name, ok, p->ntex);
}

// ---- per-frame targets (resolve_pass views; written on the game thread, read on the worker) ---------------------
struct Slot {
  uint32_t frame = ~0u;
  wgpu::TextureView color, depth, bloom[3];
  uint32_t w = 0, h = 0;
};
constexpr int kSlots = 8;
Slot g_slots[kSlots];
std::mutex g_slotMutex;

// ---- GPU objects (render worker only) ---------------------------------------------------------------------------
wgpu::ShaderModule g_module;
wgpu::BindGroupLayout g_bgl;
wgpu::PipelineLayout g_pl;
wgpu::Sampler g_samp[3], g_linearClamp;
wgpu::TextureView g_white, g_black, g_depth0;
std::unordered_map<uint64_t, wgpu::RenderPipeline> g_pipes;

enum Kind : uint32_t { KParticles = 0, KBloomParticles = 1, KBlurH = 2, KBlurV = 3, KComposite = 4, KMesh = 5,
                       KBloomMesh = 6 };
inline bool is_mesh_kind(uint32_t k) { return k == KMesh || k == KBloomMesh; }

struct Payload {
  uint32_t kind, slot, count, blend, depthTest, src; // src: the bloom buffer a blur / composite reads
  int32_t pkg, em;
  Range uni, sto;
  Range mv;    // mesh kinds: the mesh's expanded vertices (12 floats each)
  uint32_t nv; // mesh kinds: vertex count
};
static_assert(sizeof(Payload) <= aurora::gfx::InlineDrawPayloadSize, "payload");

const char* kWGSL = R"(
struct U {
  view: array<vec4f, 3>,
  proj: array<vec4f, 4>,
  screen: vec4f,      // target w, h, 1/w, 1/h
  mat0: vec4u,        // shader, color mode, offset slot (+1; 0 none), flags (1 alpha test)
  mat1: vec4u,        // offset mask, color mask, alpha mask, kind
  mat2: vec4f,        // strength.xy, alpha threshold, depth-snapshot scale (full / this target)
  mat3: vec4f,        // bloom threshold, bloom intensity, blur dir.xy
  inv_div: array<vec4f, 3>, // per sampler: 1/cols, 1/rows, atlas (1/0), 0
};
struct P {
  p0: vec4f,  // world pos, size x
  p1: vec4f,  // size y, rotation, param, 0
  c0: vec4f,  // colour0 (x scale), alpha0
  c1: vec4f,  // colour1, alpha1
  a: array<vec4f, 3>,  // per sampler: scroll.xy, scale.xy
  b: array<vec4f, 3>,  // per sampler: rotation, cell origin uv, 0
  ax: vec4f,  // mode 1 / 2: the particle's world X axis x its size (p1.w: 0 billboard, 1 oriented quad, 2 mesh)
  ay: vec4f,
  az: vec4f,
};
@group(0) @binding(0) var<uniform> u: U;
@group(0) @binding(1) var<storage, read> parts: array<P>;
@group(0) @binding(2) var t0: texture_2d<f32>;
@group(0) @binding(3) var t1: texture_2d<f32>;
@group(0) @binding(4) var t2: texture_2d<f32>;
@group(0) @binding(5) var tsrc: texture_2d<f32>;
@group(0) @binding(6) var tdepth: texture_2d<f32>;
@group(0) @binding(7) var s0: sampler;
@group(0) @binding(8) var s1: sampler;
@group(0) @binding(9) var s2: sampler;
@group(0) @binding(10) var sl: sampler;
@group(0) @binding(11) var<storage, read> mverts: array<vec4f>; // mesh kinds: 3 vec4f a vertex

struct V {
  @builtin(position) pos: vec4f,
  @location(0) uv: vec2f,
  @location(1) @interpolate(flat) ii: u32,
  @location(2) vc: vec4f,
  @location(3) fres: f32, // 1 - |the surface normal . the view axis|: 0 facing the camera, 1 at the rim
};

const corners = array<vec2f, 6>(vec2f(-0.5, -0.5), vec2f(0.5, -0.5), vec2f(0.5, 0.5),
                                vec2f(-0.5, -0.5), vec2f(0.5, 0.5), vec2f(-0.5, 0.5));

@vertex fn vs_particle(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> V {
  let p = parts[ii];
  let c = corners[vi];
  var vp: vec4f;
  if (p.p1.w > 0.5) { // oriented quad: the axes carry the size and facing
    let w = vec4f(p.p0.xyz + p.ax.xyz * c.x + p.ay.xyz * c.y, 1.0);
    vp = vec4f(dot(u.view[0], w), dot(u.view[1], w), dot(u.view[2], w), 1.0);
  } else {
    let s = c * vec2f(p.p0.w, p.p1.x);
    let cr = cos(p.p1.y);
    let sr = sin(p.p1.y);
    let w = vec4f(p.p0.xyz, 1.0);
    vp = vec4f(dot(u.view[0], w) + s.x * cr - s.y * sr, dot(u.view[1], w) + s.x * sr + s.y * cr,
               dot(u.view[2], w), 1.0);
  }
  var o: V;
  o.pos = vec4f(dot(u.proj[0], vp), dot(u.proj[1], vp), dot(u.proj[2], vp), dot(u.proj[3], vp));
  o.uv = vec2f(c.x + 0.5, 0.5 - c.y);
  o.ii = ii;
  o.vc = vec4f(1.0);
  o.fres = 1.0;
  return o;
}

@vertex fn vs_mesh(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> V {
  let p = parts[ii];
  let a = mverts[vi * 3u];
  let b = mverts[vi * 3u + 1u];
  let w = vec4f(p.p0.xyz + p.ax.xyz * a.x + p.ay.xyz * a.y + p.az.xyz * a.z, 1.0);
  let vp = vec4f(dot(u.view[0], w), dot(u.view[1], w), dot(u.view[2], w), 1.0);
  var o: V;
  o.pos = vec4f(dot(u.proj[0], vp), dot(u.proj[1], vp), dot(u.proj[2], vp), dot(u.proj[3], vp));
  o.uv = vec2f(a.w, b.x);
  o.ii = ii;
  o.vc = mverts[vi * 3u + 2u];
  let n = b.yzw;
  let nw = normalize(normalize(p.ax.xyz) * n.x + normalize(p.ay.xyz) * n.y + normalize(p.az.xyz) * n.z + vec3f(1e-6));
  o.fres = 1.0 - abs(dot(u.view[2].xyz, nw));
  return o;
}

fn smp(k: u32, uv: vec2f) -> vec4f {
  // uniform control flow: sample all three, pick one
  let a = textureSample(t0, s0, uv);
  let b = textureSample(t1, s1, uv);
  let c = textureSample(t2, s2, uv);
  return select(select(c, b, k == 1u), a, k == 0u);
}

fn uv_of(p: P, k: u32, uv: vec2f) -> vec2f {
  let a = p.a[k];
  let b = p.b[k];
  let cr = cos(b.x);
  let sr = sin(b.x);
  var q = (uv - 0.5) * a.zw;
  q = vec2f(q.x * cr - q.y * sr, q.x * sr + q.y * cr) + 0.5 + a.xy;
  let d = u.inv_div[k];
  if (d.z > 0.5) {
    q = clamp(q, vec2f(0.0), vec2f(1.0)) * d.xy + b.yz;
  }
  return q;
}

fn shade(in: V) -> vec4f {
  let p = parts[in.ii];
  var uv0 = uv_of(p, 0u, in.uv);
  var uv1 = uv_of(p, 1u, in.uv);
  var uv2 = uv_of(p, 2u, in.uv);
  var off = vec2f(0.0);
  let o = u.mat0.z;
  let ouv = select(select(uv2, uv1, o == 2u), uv0, o == 1u);
  let ot = smp(max(o, 1u) - 1u, ouv);
  if (o > 0u) {
    off = (vec2f(ot.r, ot.a) * 2.0 - 1.0) * u.mat2.xy * p.p1.z;
  }
  let om = u.mat1.x;
  if ((om & 1u) != 0u) { uv0 += off; }
  if ((om & 2u) != 0u) { uv1 += off; }
  if ((om & 4u) != 0u) { uv2 += off; }
  let s0v = smp(0u, uv0);
  let s1v = smp(1u, uv1);
  let s2v = smp(2u, uv2);
  let cm = u.mat1.y;
  let am = u.mat1.z;
  var alpha = p.c0.a * p.c1.a * in.vc.a;
  if ((am & 1u) != 0u) { alpha *= s0v.a; }
  if ((am & 2u) != 0u) { alpha *= s1v.a; }
  if ((am & 4u) != 0u) { alpha *= s2v.a; }
  var rgb = p.c0.rgb * in.vc.rgb;
  let first = select(select(s2v, s1v, (cm & 2u) != 0u), s0v, (cm & 1u) != 0u);
  if (u.mat0.y == 1u) {
    if ((cm & 1u) != 0u) { rgb *= s0v.rgb; }
    if ((cm & 2u) != 0u) { rgb *= s1v.rgb; }
    if ((cm & 4u) != 0u) { rgb *= s2v.rgb; }
  } else if (u.mat0.y == 2u) {
    rgb = mix(p.c1.rgb, p.c0.rgb, first.r);
  }
  let suv = in.pos.xy * u.screen.zw + off;
  let frame = textureSampleLevel(tsrc, sl, suv, 0.0);
  if (u.mat0.x == 2u) {
    rgb = frame.rgb;
  }
  if (u.inv_div[0].w > 0.5) { // fresnel alpha: the rim shows, the face toward the camera fades
    let lo = u.inv_div[1].w;
    let hi = u.inv_div[2].w;
    alpha *= clamp((in.fres - lo) / max(hi - lo, 1e-4), 0.0, 1.0);
  }
  alpha = clamp(alpha, 0.0, 1.0);
  return vec4f(rgb, alpha);
}

@fragment fn fs_particle(in: V) -> @location(0) vec4f {
  if (u.mat3.z > 0.5) { return vec4f(0.1, 1.0, 0.2, 0.7); } // MELEE_FX_DEBUGFLAT: the geometry only
  let c = shade(in);
  if ((u.mat0.w & 1u) != 0u && c.a <= u.mat2.z) { discard; }
  if (c.a <= 0.0) { discard; }
  return c;
}

// the bloom buffer: the part of the effect's colour over the threshold, occluded by the scene's depth snapshot
@fragment fn fs_bloom(in: V) -> @location(0) vec4f {
  let c = shade(in);
  if ((u.mat0.w & 1u) != 0u && c.a <= u.mat2.z) { discard; }
  let dp = vec2i(in.pos.xy * u.mat2.w);
  let scene = textureLoad(tdepth, dp, 0).r;
  if (scene > in.pos.z + 0.00001) { discard; } // reversed Z: a larger depth is nearer
  let b = max(c.rgb - vec3f(u.mat3.x), vec3f(0.0)) * u.mat3.y * c.a;
  return vec4f(b, 1.0);
}

struct F { @builtin(position) pos: vec4f, @location(0) uv: vec2f };
@vertex fn vs_full(@builtin(vertex_index) vi: u32) -> F {
  let xy = array<vec2f, 3>(vec2f(-1.0, 1.0), vec2f(-1.0, -3.0), vec2f(3.0, 1.0))[vi];
  var o: F;
  o.pos = vec4f(xy, 0.0, 1.0);
  o.uv = vec2f(xy.x * 0.5 + 0.5, 0.5 - xy.y * 0.5);
  return o;
}
@fragment fn fs_blur(in: F) -> @location(0) vec4f {
  let d = u.mat3.zw;
  let w = array<f32, 5>(0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216);
  var c = textureSampleLevel(tsrc, sl, in.uv, 0.0).rgb * w[0];
  for (var i = 1; i < 5; i++) {
    let o = d * f32(i);
    c += textureSampleLevel(tsrc, sl, in.uv + o, 0.0).rgb * w[i];
    c += textureSampleLevel(tsrc, sl, in.uv - o, 0.0).rgb * w[i];
  }
  return vec4f(c, 1.0);
}
@fragment fn fs_composite(in: F) -> @location(0) vec4f {
  return vec4f(textureSampleLevel(tsrc, sl, in.uv, 0.0).rgb, 0.0);
}
)";

wgpu::TextureView solid_view(const wgpu::Device& dev, const wgpu::Queue& q, wgpu::TextureFormat fmt, const void* px,
                             uint32_t bpp) {
  wgpu::TextureDescriptor td{};
  td.size = {1, 1, 1};
  td.format = fmt;
  td.usage = wgpu::TextureUsage::TextureBinding | wgpu::TextureUsage::CopyDst;
  auto t = dev.CreateTexture(&td);
  wgpu::TexelCopyTextureInfo dst{};
  dst.texture = t;
  wgpu::TexelCopyBufferLayout lay{};
  lay.bytesPerRow = 256;
  lay.rowsPerImage = 1;
  wgpu::Extent3D ext{1, 1, 1};
  q.WriteTexture(&dst, px, bpp, &lay, &ext);
  return t.CreateView();
}

wgpu::Sampler make_sampler(const wgpu::Device& dev, wgpu::AddressMode m) {
  wgpu::SamplerDescriptor sd{};
  sd.addressModeU = sd.addressModeV = sd.addressModeW = m;
  sd.magFilter = sd.minFilter = wgpu::FilterMode::Linear;
  sd.mipmapFilter = wgpu::MipmapFilterMode::Linear;
  return dev.CreateSampler(&sd);
}

void init_gpu(const aurora::gfx::DrawContext& ctx) {
  if (g_module) return;
  const auto& dev = ctx.device;
  wgpu::ShaderSourceWGSL src{};
  src.code = kWGSL;
  wgpu::ShaderModuleDescriptor md{};
  md.nextInChain = &src;
  md.label = "Geno fx module";
  g_module = dev.CreateShaderModule(&md);

  wgpu::BindGroupLayoutEntry e[12]{};
  const auto vis = wgpu::ShaderStage::Vertex | wgpu::ShaderStage::Fragment;
  e[0].binding = 0;
  e[0].visibility = vis;
  e[0].buffer.type = wgpu::BufferBindingType::Uniform;
  e[1].binding = 1;
  e[1].visibility = vis;
  e[1].buffer.type = wgpu::BufferBindingType::ReadOnlyStorage;
  for (int i = 2; i <= 5; ++i) {
    e[i].binding = i;
    e[i].visibility = wgpu::ShaderStage::Fragment;
    e[i].texture.sampleType = wgpu::TextureSampleType::Float;
    e[i].texture.viewDimension = wgpu::TextureViewDimension::e2D;
  }
  e[6].binding = 6;
  e[6].visibility = wgpu::ShaderStage::Fragment;
  e[6].texture.sampleType = wgpu::TextureSampleType::UnfilterableFloat;
  e[6].texture.viewDimension = wgpu::TextureViewDimension::e2D;
  for (int i = 7; i <= 10; ++i) {
    e[i].binding = i;
    e[i].visibility = wgpu::ShaderStage::Fragment;
    e[i].sampler.type = wgpu::SamplerBindingType::Filtering;
  }
  e[11].binding = 11;
  e[11].visibility = wgpu::ShaderStage::Vertex;
  e[11].buffer.type = wgpu::BufferBindingType::ReadOnlyStorage;
  wgpu::BindGroupLayoutDescriptor bd{};
  bd.entryCount = 12;
  bd.entries = e;
  g_bgl = dev.CreateBindGroupLayout(&bd);
  wgpu::PipelineLayoutDescriptor pd{};
  pd.bindGroupLayoutCount = 1;
  pd.bindGroupLayouts = &g_bgl;
  g_pl = dev.CreatePipelineLayout(&pd);

  g_samp[FX_WRAP_MIRROR] = make_sampler(dev, wgpu::AddressMode::MirrorRepeat);
  g_samp[FX_WRAP_REPEAT] = make_sampler(dev, wgpu::AddressMode::Repeat);
  g_samp[FX_WRAP_CLAMP] = make_sampler(dev, wgpu::AddressMode::ClampToEdge);
  g_linearClamp = g_samp[FX_WRAP_CLAMP];
  const uint8_t white[4] = {255, 255, 255, 255}, black[4] = {0, 0, 0, 0};
  const float zero = 0.0f;
  g_white = solid_view(dev, ctx.queue, wgpu::TextureFormat::RGBA8Unorm, white, 4);
  g_black = solid_view(dev, ctx.queue, wgpu::TextureFormat::RGBA8Unorm, black, 4);
  g_depth0 = solid_view(dev, ctx.queue, wgpu::TextureFormat::R32Float, &zero, 4);
  gw_log("fx: renderer initialised (layout: %u colour attachment(s), %u sample(s))",
         ctx.layout.colorAttachmentCount, ctx.layout.sampleCount);
}

wgpu::RenderPipeline pipeline(const aurora::gfx::DrawContext& ctx, uint32_t kind, uint32_t blend, uint32_t depthTest) {
  const uint64_t key = ctx.layout.key * 1000003ull ^ (uint64_t(kind) << 8 | uint64_t(blend) << 4 | depthTest);
  auto it = g_pipes.find(key);
  if (it != g_pipes.end()) return it->second;
  const auto& L = ctx.layout;
  wgpu::BlendState bs{};
  auto comp = [](wgpu::BlendOperation op, wgpu::BlendFactor s, wgpu::BlendFactor d) {
    wgpu::BlendComponent c{};
    c.operation = op;
    c.srcFactor = s;
    c.dstFactor = d;
    return c;
  };
  using BF = wgpu::BlendFactor;
  using BO = wgpu::BlendOperation;
  if (kind == KParticles || kind == KMesh) {
    switch (blend) {
    case FX_BLEND_ADD: bs.color = comp(BO::Add, BF::SrcAlpha, BF::One); break;
    case FX_BLEND_SUB: bs.color = comp(BO::ReverseSubtract, BF::SrcAlpha, BF::One); break;
    case FX_BLEND_MUL: bs.color = comp(BO::Add, BF::Dst, BF::Zero); break;
    case FX_BLEND_SCREEN: bs.color = comp(BO::Add, BF::OneMinusDst, BF::One); break;
    default: bs.color = comp(BO::Add, BF::SrcAlpha, BF::OneMinusSrcAlpha); break;
    }
    bs.alpha = comp(BO::Add, BF::Zero, BF::One); // the EFB's alpha is Melee's: leave it
  } else if (kind == KBlurH || kind == KBlurV) {
    bs.color = comp(BO::Add, BF::One, BF::Zero);
    bs.alpha = comp(BO::Add, BF::One, BF::Zero);
  } else { // bloom particles accumulate; the composite adds
    bs.color = comp(BO::Add, BF::One, BF::One);
    bs.alpha = comp(BO::Add, BF::Zero, BF::One);
  }
  wgpu::ColorTargetState ct[aurora::gfx::MaxColorAttachments]{};
  for (uint32_t i = 0; i < L.colorAttachmentCount; ++i) {
    ct[i].format = L.colorAttachments[i].format;
    ct[i].writeMask = wgpu::ColorWriteMask::None;
  }
  ct[0].blend = &bs;
  ct[0].writeMask = wgpu::ColorWriteMask::All;
  const bool particles = kind == KParticles || kind == KBloomParticles || is_mesh_kind(kind);
  wgpu::FragmentState fs{};
  fs.module = g_module;
  fs.entryPoint = kind == KParticles || kind == KMesh ? "fs_particle"
                  : kind == KBloomParticles || kind == KBloomMesh ? "fs_bloom"
                  : kind == KComposite ? "fs_composite" : "fs_blur";
  fs.targetCount = L.colorAttachmentCount;
  fs.targets = ct;
  wgpu::DepthStencilState ds{};
  ds.format = L.depthStencilFormat;
  ds.depthWriteEnabled = false;
  ds.depthCompare = (kind == KParticles || kind == KMesh) && depthTest
                        ? (aurora::gfx::uses_reversed_z() ? wgpu::CompareFunction::GreaterEqual
                                                          : wgpu::CompareFunction::LessEqual)
                        : wgpu::CompareFunction::Always;
  wgpu::RenderPipelineDescriptor rd{};
  rd.label = "Geno fx pipeline";
  rd.layout = g_pl;
  rd.vertex.module = g_module;
  rd.vertex.entryPoint = is_mesh_kind(kind) ? "vs_mesh" : particles ? "vs_particle" : "vs_full";
  rd.primitive.topology = wgpu::PrimitiveTopology::TriangleList;
  rd.depthStencil = L.depthStencilFormat != wgpu::TextureFormat::Undefined ? &ds : nullptr;
  rd.multisample.count = L.sampleCount;
  rd.fragment = &fs;
  auto p = ctx.device.CreateRenderPipeline(&rd);
  g_pipes.emplace(key, p);
  gw_log("fx: pipeline kind %u blend %u depth %u created (%zu total)", kind, blend, depthTest, g_pipes.size());
  return p;
}

wgpu::TextureView tex_view(const aurora::gfx::DrawContext& ctx, int pkg, int t) {
  if (pkg < 0 || pkg >= int(g_tex.size()) || t < 0 || t >= int(g_tex[pkg].size())) return g_white;
  FxTex& x = g_tex[pkg][t];
  if (x.state != 1) return g_white;
  if (!x.view) {
    wgpu::TextureDescriptor td{};
    td.size = {x.w, x.h, 1};
    td.format = wgpu::TextureFormat::RGBA8Unorm;
    td.usage = wgpu::TextureUsage::TextureBinding | wgpu::TextureUsage::CopyDst;
    x.tex = ctx.device.CreateTexture(&td);
    wgpu::TexelCopyTextureInfo dst{};
    dst.texture = x.tex;
    wgpu::TexelCopyBufferLayout lay{};
    lay.bytesPerRow = x.w * 4;
    lay.rowsPerImage = x.h;
    wgpu::Extent3D ext{x.w, x.h, 1};
    ctx.queue.WriteTexture(&dst, x.rgba.data(), x.rgba.size(), &lay, &ext);
    x.view = x.tex.CreateView();
  }
  return x.view;
}

void draw_cb(const aurora::gfx::DrawContext& ctx, const wgpu::RenderPassEncoder& pass, const void* payload, size_t,
             void*) {
  Payload d;
  std::memcpy(&d, payload, sizeof d);
  init_gpu(ctx);
  Slot slot;
  {
    std::lock_guard<std::mutex> lk(g_slotMutex);
    slot = g_slots[d.slot % kSlots];
  }
  const fx_pkg* p = d.pkg >= 0 ? gw_fx_pkg(d.pkg) : nullptr;
  const fx_emitter* e = p && d.em >= 0 && d.em < p->nem ? &p->em[d.em] : nullptr;
  wgpu::TextureView tv[3] = {g_white, g_white, g_white};
  int wrap[3] = {FX_WRAP_REPEAT, FX_WRAP_REPEAT, FX_WRAP_REPEAT};
  if (e)
    for (int k = 0; k < FX_SAMPLERS; ++k) {
      tv[k] = tex_view(ctx, d.pkg, e->smp[k].tex);
      wrap[k] = e->smp[k].wrap[0];
    }
  wgpu::TextureView src = g_black;
  if ((d.kind == KParticles || d.kind == KMesh) && slot.color) src = slot.color;
  if ((d.kind == KBlurH || d.kind == KBlurV || d.kind == KComposite) && d.src < 3 && slot.bloom[d.src])
    src = slot.bloom[d.src];
  wgpu::TextureView depth = slot.depth ? slot.depth : g_depth0;

  wgpu::BindGroupEntry be[12]{};
  be[0].binding = 0;
  be[0].buffer = ctx.uniformBuffer;
  be[0].offset = d.uni.offset;
  be[0].size = d.uni.size;
  be[1].binding = 1;
  be[1].buffer = ctx.storageBuffer;
  be[1].offset = d.sto.offset;
  be[1].size = d.sto.size ? d.sto.size : 256;
  if (d.sto.size == 0) be[1].offset = 0;
  for (int k = 0; k < 3; ++k) {
    be[2 + k].binding = 2 + k;
    be[2 + k].textureView = tv[k];
  }
  be[5].binding = 5;
  be[5].textureView = src;
  be[6].binding = 6;
  be[6].textureView = depth;
  for (int k = 0; k < 3; ++k) {
    be[7 + k].binding = 7 + k;
    be[7 + k].sampler = g_samp[wrap[k] >= 0 && wrap[k] < 3 ? wrap[k] : FX_WRAP_REPEAT];
  }
  be[10].binding = 10;
  be[10].sampler = g_linearClamp;
  wgpu::BindGroupDescriptor bgd{};
  bgd.layout = g_bgl;
  be[11].binding = 11;
  be[11].buffer = ctx.storageBuffer;
  be[11].offset = is_mesh_kind(d.kind) && d.mv.size ? d.mv.offset : be[1].offset;
  be[11].size = is_mesh_kind(d.kind) && d.mv.size ? d.mv.size : be[1].size;
  bgd.entryCount = 12;
  bgd.entries = be;
  auto bg = ctx.device.CreateBindGroup(&bgd);
  pass.SetPipeline(pipeline(ctx, d.kind, d.blend, d.depthTest));
  pass.SetBindGroup(0, bg);
  if (is_mesh_kind(d.kind))
    pass.Draw(d.nv, d.count);
  else if (d.kind == KParticles || d.kind == KBloomParticles)
    pass.Draw(6, d.count);
  else
    pass.Draw(3);
}

aurora::gfx::DrawTypeId g_drawType = aurora::gfx::InvalidDrawType;

// ---- building the frame's data (game thread) -------------------------------------------------------------------
struct alignas(16) GpuU {
  float view[3][4];
  float proj[4][4];
  float screen[4];
  uint32_t mat0[4], mat1[4];
  float mat2[4], mat3[4];
  float inv_div[3][4];
};
struct alignas(16) GpuP {
  float p0[4], p1[4], c0[4], c1[4], a[3][4], b[3][4], ax[4], ay[4], az[4];
};

float curve(const fx_curve& c, float t, int ch) { return gw_fx_curve_at(&c, t, ch); }

void v3norm(float v[3]) {
  const float l = std::sqrt(v[0] * v[0] + v[1] * v[1] + v[2] * v[2]);
  if (l > 1e-6f)
    for (int k = 0; k < 3; ++k) v[k] /= l;
}
void v3cross(const float a[3], const float b[3], float o[3]) {
  o[0] = a[1] * b[2] - a[2] * b[1];
  o[1] = a[2] * b[0] - a[0] * b[2];
  o[2] = a[0] * b[1] - a[1] * b[0];
}
// the emitter's own rotation (X, then Y, then Z, radians; as gw_fx.c fx_erot) after the particle's roll about Z
void local_axis(const fx_emitter& e, float roll, int axis, float v[3]) {
  float x = axis == 0 ? 1.0f : 0.0f, y = axis == 1 ? 1.0f : 0.0f, z = axis == 2 ? 1.0f : 0.0f, c, s;
  c = std::cos(roll); s = std::sin(roll); { const float x2 = x * c - y * s, y2 = x * s + y * c; x = x2; y = y2; }
  c = std::cos(e.rot[0]); s = std::sin(e.rot[0]); { const float y2 = y * c - z * s, z2 = y * s + z * c; y = y2; z = z2; }
  c = std::cos(e.rot[1]); s = std::sin(e.rot[1]); { const float x2 = x * c + z * s, z2 = -x * s + z * c; x = x2; z = z2; }
  c = std::cos(e.rot[2]); s = std::sin(e.rot[2]); { const float x2 = x * c - y * s, y2 = x * s + y * c; x = x2; y = y2; }
  v[0] = x; v[1] = y; v[2] = z;
}
// the emitter frame (the owner's matrix x basis; the joint's scale included) applied to a direction
void frame_dir(const fx_inst& in, const float v[3], float o[3]) {
  for (int r = 0; r < 3; ++r) o[r] = in.m[r][0] * v[0] + in.m[r][1] * v[1] + in.m[r][2] * v[2];
}

void particle_data(const fx_emitter& e, const fx_part& q, const fx_inst& in, const float (*view)[4], GpuP& o) {
  const float t = q.life > 0 ? float(q.age) / float(q.life) : 0.0f;
  // the simulation's per-particle visual state (gw_fx.c fx_visual_update: wave offset / scale, curve loops,
  // fade in / on stop, colour x HDR scale, the flipbook cell)
  o.p0[0] = q.visual_pos[0];
  o.p0[1] = q.visual_pos[1];
  o.p0[2] = q.visual_pos[2];
  o.p0[3] = q.visual_scale[0];
  o.p1[0] = q.visual_scale[1];
  o.p1[1] = q.rot;
  o.p1[2] = q.visual_param;
  o.p1[3] = 0.0f;
  {
    // facing: 0 camera billboard (above), 1 an oriented quad, 2 a mesh instance; the axes carry the size
    // the emitter frame's own scale (a fighter bone: the model's scale, 1.15 for Sora): offsets and velocities
    // already pass through it, so quad sizes take it too (meshes get it from the frame itself, below)
    float fs = 0.0f;
    for (int c = 0; c < 3; ++c)
      fs += std::sqrt(in.m[0][c] * in.m[0][c] + in.m[1][c] * in.m[1][c] + in.m[2][c] * in.m[2][c]) / 3.0f;
    if (!(fs > 1e-4f)) fs = 1.0f;
    const float sx = o.p0[3] * e.escale[0] * fs, sy = o.p1[0] * e.escale[1] * fs;
    float cz = curve(e.scale_keys, t, 2);
    if (!e.scale_keys.keyed && cz == 0.0f) cz = curve(e.scale_keys, t, 0);
    const float sz = e.scale_z * cz * q.scale * e.escale[2];
    float X[3], Y[3], Z[3], lx[3], ly[3], lz[3];
    local_axis(e, q.rot, 0, lx);
    local_axis(e, q.rot, 1, ly);
    local_axis(e, q.rot, 2, lz);
    frame_dir(in, lx, X);
    frame_dir(in, ly, Y);
    frame_dir(in, lz, Z);
    const float camz[3] = {view[2][0], view[2][1], view[2][2]}; // the view's Z axis, in world
    int mode = 0;
    if (e.mesh && e.mesh_idx >= 0) {
      mode = 2; // mesh units are the emitter frame's (joint-local): keep the frame's own scale
      for (int k = 0; k < 3; ++k) { X[k] *= o.p0[3] * e.escale[0]; Y[k] *= o.p1[0] * e.escale[1]; Z[k] *= sz; }
    } else if (e.pshape == FX_PS_DIRECTIONAL || e.pshape == FX_PS_Y_BILLBOARD) {
      mode = 1;
      float d[3];
      if (e.pshape == FX_PS_Y_BILLBOARD) {
        d[0] = 0.0f; d[1] = 1.0f; d[2] = 0.0f;
      } else {
        // along the velocity (world): follow-srt particles keep theirs in the emitter frame
        if (e.follow == 0) frame_dir(in, q.lvel, d);
        else for (int k = 0; k < 3; ++k) d[k] = q.vel[k];
        if (d[0] * d[0] + d[1] * d[1] + d[2] * d[2] < 1e-8f)
          for (int k = 0; k < 3; ++k) d[k] = Y[k];
      }
      v3norm(d);
      float side[3];
      v3cross(d, camz, side);
      v3norm(side);
      for (int k = 0; k < 3; ++k) { X[k] = side[k] * sx; Y[k] = d[k] * sy; Z[k] = 0.0f; }
    } else if (e.pshape == FX_PS_PLATE_XY || e.pshape == FX_PS_PLATE_XZ) {
      mode = 1;
      v3norm(X);
      if (e.pshape == FX_PS_PLATE_XZ) std::memcpy(Y, Z, sizeof Y);
      v3norm(Y);
      for (int k = 0; k < 3; ++k) { X[k] *= sx; Y[k] *= sy; Z[k] = 0.0f; }
    } else {
      o.p0[3] = sx;
      o.p1[0] = sy;
    }
    o.p1[3] = float(mode);
    for (int k = 0; k < 3; ++k) { o.ax[k] = X[k]; o.ay[k] = Y[k]; o.az[k] = Z[k]; }
    o.ax[3] = o.ay[3] = o.az[3] = 0.0f;
  }
  for (int c = 0; c < 4; ++c) {
    o.c0[c] = q.visual_color0[c];
    o.c1[c] = q.visual_color1[c];
  }
  const float age = float(q.age);
  for (int k = 0; k < FX_SAMPLERS; ++k) {
    const fx_sampler& s = e.smp[k];
    // the particle's own initial UV (the sampler's value + its seeded random), then the per-frame adds
    o.a[k][0] = q.uv_scroll[k][0] + (s.en_scroll ? s.scroll_add[0] * age : 0.0f);
    o.a[k][1] = q.uv_scroll[k][1] + (s.en_scroll ? s.scroll_add[1] * age : 0.0f);
    o.a[k][2] = q.uv_scale[k][0] + (s.en_scale ? s.scale_add[0] * age : 0.0f);
    o.a[k][3] = q.uv_scale[k][1] + (s.en_scale ? s.scale_add[1] * age : 0.0f);
    o.b[k][0] = q.uv_rotate[k] + (s.en_rotate ? s.rotate_add * age : 0.0f);
    o.b[k][1] = q.pattern_uv[k][0];
    o.b[k][2] = q.pattern_uv[k][1];
    o.b[k][3] = 0.0f;
  }
}

void fill_material(const fx_emitter& e, GpuU& u, uint32_t kind) {
  static int flat = -1;
  if (flat < 0) { const char* v = std::getenv("MELEE_FX_DEBUGFLAT"); flat = v != nullptr && v[0] == '1'; }
  u.mat3[2] = flat ? 1.0f : 0.0f;
  u.mat0[0] = uint32_t(e.shader);
  u.mat0[1] = uint32_t(e.color_mode);
  u.mat0[2] = e.offset >= 0 && e.offset < FX_SAMPLERS ? uint32_t(e.offset + 1) : 0u;
  u.mat0[3] = e.alpha_test ? 1u : 0u;
  u.mat1[0] = uint32_t(e.offset_mask);
  u.mat1[1] = uint32_t(e.color_mask);
  u.mat1[2] = uint32_t(e.alpha_mask);
  u.mat1[3] = kind;
  u.mat2[0] = e.strength[0];
  u.mat2[1] = e.strength[1];
  u.mat2[2] = e.alpha_threshold;
  u.mat3[0] = e.bloom_threshold;
  u.mat3[1] = e.bloom_intensity;
  for (int k = 0; k < FX_SAMPLERS; ++k) {
    const int cols = e.smp[k].div[0] > 0 ? e.smp[k].div[0] : 1, rows = e.smp[k].div[1] > 0 ? e.smp[k].div[1] : 1;
    u.inv_div[k][0] = 1.0f / float(cols);
    u.inv_div[k][1] = 1.0f / float(rows);
    u.inv_div[k][2] = cols > 1 || rows > 1 ? 1.0f : 0.0f;
    u.inv_div[k][3] = 0.0f;
    if (k == 0) u.inv_div[0][3] = e.fresnel && e.mesh ? 1.0f : 0.0f;
    if (k == 1) u.inv_div[1][3] = e.fresnel_lo;
    if (k == 2) u.inv_div[2][3] = e.fresnel_hi;
  }
}

uint32_t g_lastFrame = ~0u;
uint32_t g_drawnFrames, g_logEvery = 60;
double g_cpuMs, g_cpuMax; // game-thread cost of gw_Fx_Draw since the last log line

double now_ms() {
  static LARGE_INTEGER f;
  LARGE_INTEGER c;
  if (f.QuadPart == 0) QueryPerformanceFrequency(&f);
  QueryPerformanceCounter(&c);
  return double(c.QuadPart) * 1000.0 / double(f.QuadPart);
}

struct Group { int pkg, em, first, count, mesh; }; // mesh: the package mesh index, -1 quads
struct FrameData {
  GpuU base;
  std::vector<GpuP> buf;
  std::vector<Group> groups;
  bool needFrame = false, needBloom = false;
  uint32_t slotIdx = 0, frame = 0;
};
constexpr uint32_t kFrames = 4; // the GX thread consumes a frame's data before the frame ends (aurora_end_frame drains)
FrameData g_frames[kFrames];
uint32_t g_frameSeq;
double g_gxMs;
uint32_t g_gxCount;
int g_lastBloomDraws;
uint32_t g_lastW, g_lastH;
void record_frame(const void* data, u32 size);
} // namespace

extern "C" void gw_diag_fx_cam(int cobj, int code, int main_cobj, float ex, float ey, float ez, float mx, float my,
                               float mz) {
  static int on = -1, lines = 0;
  if (on < 0) { const char* v = std::getenv("MELEE_FX_CAMLOG"); on = v != nullptr && v[0] == '1'; }
  if (!on || lines >= 60) return;
  ++lines;
  gw_log("fx: camcall code %d current cobj 0x%08X eye (%.1f %.1f %.1f) | main camera 0x%08X eye (%.1f %.1f %.1f)", code,
         uint32_t(cobj), ex, ey, ez, uint32_t(main_cobj), mx, my, mz);
}

// Called from the Geno effects GObj's render callback (game thread, inside the camera's EFB pass), with the
// guest address of the camera's viewing matrix (world -> view, 3x4, big-endian). Draws every live Geno particle.
extern "C" void gw_Fx_Draw(int view_guest) {
  const fx_state* st = gw_fx_state();
  if (st == nullptr || st->nlive <= 0 || view_guest == 0) return;
  const uint32_t frame = aurora::gfx::current_frame();
  {
    // MELEE_FX_CAMLOG=1: every call (each camera that draws link 8), every 60th frame: the view's translation
    // and the projection, to see which camera the effects take
    static int camlog = -1;
    if (camlog < 0) { const char* v = std::getenv("MELEE_FX_CAMLOG"); camlog = v != nullptr && v[0] == '1'; }
    static int camlines = 0;
    if (camlog && camlines < 400 && (frame % 4) == 0) {
      ++camlines;
      f32 pv[7];
      GXGetProjectionv(pv);
      float vt[3];
      for (int r = 0; r < 3; ++r) vt[r] = gw_rf32((const void*)(uintptr_t)(uint32_t(view_guest) + (r * 4 + 3) * 4));
      gw_log("fx: camlog frame %u view@0x%08X t=(%.1f %.1f %.1f) proj %s %.4f %.4f %.4f %.4f %.4f %.4f%s", frame,
             uint32_t(view_guest), vt[0], vt[1], vt[2], pv[0] != 0.0f ? "ortho" : "persp", pv[1], pv[2], pv[3], pv[4],
             pv[5], pv[6], frame == g_lastFrame ? " (skipped: not the first this frame)" : " (TAKEN)");
    }
  }
  if (frame == g_lastFrame) return; // once per frame: the first camera that draws link 8 (the main one)
  g_lastFrame = frame;
  const double t0 = now_ms();
  if (g_drawType == aurora::gfx::InvalidDrawType) {
    aurora::gfx::DrawTypeDescriptor dd{};
    dd.label = "Geno fx";
    dd.draw = draw_cb;
    g_drawType = aurora::gfx::register_draw_type(dd);
  }

  GpuU base{};
  for (int r = 0; r < 3; ++r)
    for (int c = 0; c < 4; ++c)
      base.view[r][c] = gw_rf32((const void*)(uintptr_t)(uint32_t(view_guest) + (r * 4 + c) * 4));
  {
    f32 pv[7];
    GXGetProjectionv(pv);
    float(*P)[4] = base.proj;
    std::memset(P, 0, sizeof base.proj);
    P[0][0] = pv[1];
    P[1][1] = pv[3];
    P[2][2] = pv[5];
    P[2][3] = pv[6];
    if (pv[0] != 0.0f) { // orthographic
      P[0][3] = pv[2];
      P[1][3] = pv[4];
      P[3][3] = 1.0f;
    } else {
      P[0][2] = pv[2];
      P[1][2] = pv[4];
      P[3][2] = -1.0f;
    }
    if (aurora::gfx::uses_reversed_z()) // as Aurora's GX shaders do (shader_info.cpp)
      for (int c = 0; c < 4; ++c) P[2][c] = -P[2][c];
    else
      for (int c = 0; c < 4; ++c) P[2][c] += P[3][c];
  }

  // live particles grouped by emitter instance
  std::vector<GpuP> buf;
  std::vector<Group> groups;
  buf.reserve(size_t(st->nlive));
  bool needFrame = false, needBloom = false;
  for (int i = 0; i < FX_MAX_INST; ++i) {
    const fx_inst& in = st->inst[i];
    if (!in.used) continue;
    const fx_pkg* p = gw_fx_pkg(in.pkg);
    if (p == nullptr || in.em >= p->nem) continue;
    const fx_emitter& e = p->em[in.em];
    const int first = int(buf.size());
    for (int k = 0; k < FX_MAX_PARTICLES; ++k)
      if (st->part[k].inst == i) {
        buf.emplace_back();
        particle_data(e, st->part[k], in, base.view, buf.back());
      }
    if (int(buf.size()) == first) continue;
    ensure_textures(in.pkg);
    groups.push_back({in.pkg, in.em, first, int(buf.size()) - first,
                      e.mesh && e.mesh_idx >= 0 && e.mesh_idx < p->nmesh_loaded && p->mesh_nv[e.mesh_idx] > 0 ? e.mesh_idx : -1});
    needFrame |= e.shader == FX_SH_DISTORTION;
    needBloom |= e.bloom_intensity > 0.0f;
  }
  if (groups.empty()) return;
  {
    // MELEE_FX_DRAWLOG=1: per emitter group, what is drawn and where it lands on screen (NDC bbox of the particle
    // centres, or of the mesh vertices), capped
    static int on = -1, lines = 0;
    if (on < 0) { const char* v = std::getenv("MELEE_FX_DRAWLOG"); on = v != nullptr && v[0] == '1'; }
    if (on && lines < 600) {
      for (const Group& g : groups) {
        const fx_pkg* p = gw_fx_pkg(g.pkg);
        const fx_emitter& e = p->em[g.em];
        float mn[2] = {1e9f, 1e9f}, mx[2] = {-1e9f, -1e9f};
        auto proj = [&](const float w[3]) {
          float vv[4];
          for (int r = 0; r < 3; ++r) vv[r] = base.view[r][0] * w[0] + base.view[r][1] * w[1] + base.view[r][2] * w[2] + base.view[r][3];
          vv[3] = 1.0f;
          float c[4];
          for (int r = 0; r < 4; ++r) c[r] = base.proj[r][0] * vv[0] + base.proj[r][1] * vv[1] + base.proj[r][2] * vv[2] + base.proj[r][3] * vv[3];
          if (c[3] > 1e-6f) for (int k = 0; k < 2; ++k) { mn[k] = std::min(mn[k], c[k] / c[3]); mx[k] = std::max(mx[k], c[k] / c[3]); }
        };
        for (int q = g.first; q < g.first + g.count; ++q) {
          const GpuP& gp = buf[size_t(q)];
          if (g.mesh >= 0) {
            const float* mv = p->mesh_v[g.mesh];
            for (int v = 0; v < p->mesh_nv[g.mesh]; v += 7) {
              const float* a = mv + size_t(v) * 12;
              float w[3];
              for (int r = 0; r < 3; ++r) w[r] = gp.p0[r] + gp.ax[r] * a[0] + gp.ay[r] * a[1] + gp.az[r] * a[2];
              proj(w);
            }
          } else {
            proj(gp.p0);
          }
        }
        const GpuP& f = buf[size_t(g.first)];
        int texok = 0, texn = 0;
        for (int k = 0; k < FX_SAMPLERS; ++k)
          if (e.smp[k].tex >= 0) { ++texn; texok += g.pkg < int(g_tex.size()) && e.smp[k].tex < int(g_tex[g.pkg].size()) && g_tex[g.pkg][e.smp[k].tex].state == 1; }
        ++lines;
        gw_log("fx: drawlog f%u %s/%s %s n=%d nv=%d blend %d depth %d shader %d cmode %d cmask %u amask %u tex %d/%d "
               "c0 (%.2f %.2f %.2f %.2f) c1 (%.2f %.2f %.2f %.2f) size (%.2f %.2f) mode %.0f ndc x[%.2f %.2f] y[%.2f %.2f]",
               frame, p->name, e.name, g.mesh >= 0 ? "mesh" : "quad", g.count, g.mesh >= 0 ? p->mesh_nv[g.mesh] : 6,
               e.blend, e.depth_test, e.shader, e.color_mode, unsigned(e.color_mask), unsigned(e.alpha_mask), texok, texn,
               f.c0[0], f.c0[1], f.c0[2], f.c0[3], f.c1[0], f.c1[1], f.c1[2], f.c1[3], f.p0[3], f.p1[0], f.p1[3],
               mn[0], mx[0], mn[1], mx[1]);
      }
    }
  }

  // hand the frame to the GX thread: it records the draws at this point of the command stream (after everything
  // Melee queued before us), so the game thread never waits for it (Aurora port patch GXAuroraCallback)
  FrameData& fd = g_frames[g_frameSeq++ % kFrames];
  fd.base = base;
  fd.buf.swap(buf);
  fd.groups.swap(groups);
  fd.needFrame = needFrame;
  fd.needBloom = needBloom;
  fd.slotIdx = frame % kSlots;
  fd.frame = frame;
  const uint32_t idx = uint32_t(&fd - g_frames);
  GXAuroraCallback(record_frame, &idx, sizeof idx);
  const double dt = now_ms() - t0;
  g_cpuMs += dt;
  g_cpuMax = std::max(g_cpuMax, dt);
  if ((++g_drawnFrames % g_logEvery) == 1) {
    gw_log("fx: draw frame %u: %zu particles in %zu emitter draw(s), frame copy %d, bloom %d; game-thread cost avg "
           "%.3f ms max %.3f ms over %u frame(s); GX-thread recording avg %.3f ms (%d bloom draws, %ux%u)",
           frame, fd.buf.size(), fd.groups.size(), needFrame ? 1 : 0, needBloom ? 1 : 0,
           g_cpuMs / double(g_drawnFrames == 1 ? 1 : g_logEvery), g_cpuMax, g_drawnFrames == 1 ? 1u : g_logEvery,
           g_gxMs / double(g_gxCount ? g_gxCount : 1), g_lastBloomDraws, g_lastW, g_lastH);
    g_cpuMs = g_cpuMax = g_gxMs = 0.0;
    g_gxCount = 0;
  }
}

namespace {
// Runs on the GX thread (GX_AURORA_CALLBACK): the aurora::gfx recording calls do not drain the FIFO here.
void record_frame(const void* data, u32 size) {
  if (size != sizeof(uint32_t)) return;
  uint32_t idx;
  std::memcpy(&idx, data, sizeof idx);
  FrameData& fd = g_frames[idx % kFrames];
  const double g0 = now_ms();
  const GpuU& base = fd.base;
  const std::vector<GpuP>& buf = fd.buf;
  const std::vector<Group>& groups = fd.groups;
  bool needFrame = fd.needFrame, needBloom = fd.needBloom;

  const uint32_t slotIdx = fd.slotIdx;
  Slot slot;
  slot.frame = fd.frame;
  if (needFrame || needBloom) {
    aurora::gfx::ResolveDesc rd{};
    rd.color = needFrame;
    rd.depth = needBloom;
    aurora::gfx::ResolvedTargets rt{};
    if (aurora::gfx::resolve_pass(rd, rt)) {
      slot.color = rt.color;
      slot.depth = rt.depth;
      slot.w = rt.width;
      slot.h = rt.height;
    }
    if (!slot.depth) needBloom = false;
  }
  {
    std::lock_guard<std::mutex> lk(g_slotMutex);
    g_slots[slotIdx] = slot;
  }

  auto push = [&](uint32_t kind, const Group* g, float tw, float th, float depthScale, float dx, float dy,
                  uint32_t src) {
    GpuU u = base;
    Payload d{};
    d.kind = kind;
    d.slot = slotIdx;
    d.src = src;
    d.pkg = d.em = -1;
    if (g) {
      const fx_emitter& e = gw_fx_pkg(g->pkg)->em[g->em];
      fill_material(e, u, kind);
      d.pkg = g->pkg;
      d.em = g->em;
      d.count = uint32_t(g->count);
      d.blend = uint32_t(e.blend);
      d.depthTest = uint32_t(e.depth_test);
    }
    u.screen[0] = tw;
    u.screen[1] = th;
    u.screen[2] = tw > 0 ? 1.0f / tw : 0.0f;
    u.screen[3] = th > 0 ? 1.0f / th : 0.0f;
    u.mat2[3] = depthScale;
    if (!g) {
      u.mat3[2] = dx;
      u.mat3[3] = dy;
    }
    d.uni = aurora::gfx::push_uniform(reinterpret_cast<const uint8_t*>(&u), sizeof u);
    aurora::gfx::push_custom_draw(g_drawType, &d, sizeof d);
  };
  // one storage push per emitter group: push_storage aligns each to the device's storage offset alignment
  const float W = float(slot.w ? slot.w : 640), H = float(slot.h ? slot.h : 480);
  std::vector<Range> own(groups.size());
  for (size_t gi = 0; gi < groups.size(); ++gi) {
    own[gi] = aurora::gfx::push_storage(reinterpret_cast<const uint8_t*>(buf.data() + groups[gi].first),
                                        size_t(groups[gi].count) * sizeof(GpuP));
  }
  std::vector<Range> meshRange(groups.size());
  for (size_t gi = 0; gi < groups.size(); ++gi) {
    const Group& g = groups[gi];
    if (g.mesh < 0) continue;
    for (size_t gj = 0; gj < gi; ++gj) // the same mesh earlier this frame: share its upload
      if (groups[gj].pkg == g.pkg && groups[gj].mesh == g.mesh) { meshRange[gi] = meshRange[gj]; break; }
    if (meshRange[gi].size == 0) {
      const fx_pkg* mp = gw_fx_pkg(g.pkg);
      meshRange[gi] = aurora::gfx::push_storage(reinterpret_cast<const uint8_t*>(mp->mesh_v[g.mesh]),
                                                size_t(mp->mesh_nv[g.mesh]) * 12 * sizeof(float));
    }
  }
  auto push_group = [&](uint32_t kind, size_t gi, float tw, float th, float depthScale) {
    // as push(), with the group's own storage range
    Group g = groups[gi];
    if (g.mesh >= 0) {
      kind = kind == KParticles ? KMesh : KBloomMesh;
    }
    GpuU u = base;
    const fx_emitter& e = gw_fx_pkg(g.pkg)->em[g.em];
    fill_material(e, u, kind);
    Payload d{};
    d.kind = kind;
    d.slot = slotIdx;
    d.pkg = g.pkg;
    d.em = g.em;
    d.count = uint32_t(g.count);
    d.blend = uint32_t(e.blend);
    d.depthTest = uint32_t(e.depth_test);
    d.sto = own[gi];
    if (g.mesh >= 0) {
      d.mv = meshRange[gi];
      d.nv = uint32_t(gw_fx_pkg(g.pkg)->mesh_nv[g.mesh]);
    }
    u.screen[0] = tw;
    u.screen[1] = th;
    u.screen[2] = tw > 0 ? 1.0f / tw : 0.0f;
    u.screen[3] = th > 0 ? 1.0f / th : 0.0f;
    u.mat2[3] = depthScale;
    d.uni = aurora::gfx::push_uniform(reinterpret_cast<const uint8_t*>(&u), sizeof u);
    aurora::gfx::push_custom_draw(g_drawType, &d, sizeof d);
  };
  for (size_t gi = 0; gi < groups.size(); ++gi) push_group(KParticles, gi, W, H, 1.0f);

  int bloomDraws = 0;
  if (needBloom) {
    const uint32_t bw = std::max(1u, slot.w / 2), bh = std::max(1u, slot.h / 2);
    const uint32_t qw = std::max(1u, slot.w / 4), qh = std::max(1u, slot.h / 4);
    aurora::gfx::ResolvedTargets rt{};
    aurora::gfx::ResolveDesc colorOnly{};
    if (aurora::gfx::create_pass(bw, bh)) {
      for (size_t gi = 0; gi < groups.size(); ++gi) {
        if (gw_fx_pkg(groups[gi].pkg)->em[groups[gi].em].bloom_intensity <= 0.0f) continue;
        push_group(KBloomParticles, gi, float(bw), float(bh), 2.0f);
        ++bloomDraws;
      }
      if (aurora::gfx::resolve_pass(colorOnly, rt)) slot.bloom[0] = rt.color;
    }
    if (slot.bloom[0] && aurora::gfx::create_pass(qw, qh)) {
      {
        std::lock_guard<std::mutex> lk(g_slotMutex);
        g_slots[slotIdx] = slot;
      }
      push(KBlurH, nullptr, float(qw), float(qh), 1.0f, 1.0f / float(bw), 0.0f, 0);
      if (aurora::gfx::resolve_pass(colorOnly, rt)) slot.bloom[1] = rt.color;
    }
    if (slot.bloom[1] && aurora::gfx::create_pass(qw, qh)) {
      {
        std::lock_guard<std::mutex> lk(g_slotMutex);
        g_slots[slotIdx] = slot;
      }
      push(KBlurV, nullptr, float(qw), float(qh), 1.0f, 0.0f, 1.0f / float(qh), 1);
      if (aurora::gfx::resolve_pass(colorOnly, rt)) slot.bloom[2] = rt.color;
    }
    {
      std::lock_guard<std::mutex> lk(g_slotMutex);
      g_slots[slotIdx] = slot;
    }
    if (slot.bloom[2]) push(KComposite, nullptr, W, H, 1.0f, 0.0f, 0.0f, 2);
  }
  g_lastBloomDraws = bloomDraws;
  g_lastW = slot.w;
  g_lastH = slot.h;
  g_gxMs += now_ms() - g0;
  ++g_gxCount;
}
} // namespace
