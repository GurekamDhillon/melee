// melee_gx oracle: Aurora's own code, linked from the shipping Aurora build (aurora_gx.lib etc.), run
// over deterministic inputs. Modes (build.py runs all four and writes tests/data/):
//   oracle tex           convert_texture / convert_texture_palette hashes (no window)
//   oracle shader [n]    build_shader_source (WGSL) length + hash for n random ShaderConfigs
//   oracle state [n]     n random GX API call sequences after GXInit -> config / pipeline / uniform / WGSL
//   oracle render <png>  draws scene.inc through Aurora and writes Aurora's screenshot
// shader/state/render open Aurora's 640x480 window. Backend: D3D11 (ORACLE_BACKEND=<AuroraBackend>
// overrides): this Dawn build has no Vulkan, and its D3D12 swapchain faults in GetCurrentTexture
// (WaitForQueueSerial) in this harness on the dev machine. Debug aids: SCENE_MASK, SHOT_FRAME, NO_SCENE,
// and an unhandled-exception reporter that prints symbolized frames (no debugger installed here).
// Note: Aurora itself writes its pipeline-coverage db under %LOCALAPPDATA%\GD Melee while running.
// Built by build.py (MSVC x86, the flags CMake used for aurora_gx). Inputs come from xorshift32 seeds
// that tests/oracle.rs regenerates bit for bit; only parameters + FNV-1a 64 hashes are written.
#include <aurora/aurora.h>
#include <aurora/event.h>
#include <aurora/gfx.h>
#include <aurora/main.h>
#include <dolphin/gx.h>
#include <dolphin/os.h>

#include "gx/gx.hpp"
#include "gx/shader_info.hpp"
#include "gx/texture.hpp"
#include "gfx/texture_convert.hpp"
#include "gx/fifo.hpp"
#include "gx/pipeline.hpp"

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>
#include <filesystem>
#include <thread>
#include <chrono>

using namespace aurora;

namespace {
struct Rng {
  uint32_t s;
  uint32_t next() {
    s ^= s << 13;
    s ^= s >> 17;
    s ^= s << 5;
    return s;
  }
  uint32_t below(uint32_t n) { return next() % n; }
  bool chance(uint32_t pct) { return below(100) < pct; }
};

uint64_t fnv(const void* p, size_t n, uint64_t h = 0xcbf29ce484222325ull) {
  const auto* b = static_cast<const uint8_t*>(p);
  for (size_t i = 0; i < n; ++i) {
    h ^= b[i];
    h *= 0x100000001b3ull;
  }
  return h;
}

std::vector<uint8_t> bytes(Rng& r, size_t n) {
  std::vector<uint8_t> v(n);
  for (auto& b : v) b = static_cast<uint8_t>(r.next() >> 24);
  return v;
}

// ---------------------------------------------------------------- textures
int run_tex() {
  const uint32_t fmts[] = {GX_TF_I4, GX_TF_I8, GX_TF_IA4, GX_TF_IA8, GX_TF_RGB565, GX_TF_RGB5A3, GX_TF_RGBA8, GX_TF_CMPR};
  const uint32_t sizes[][3] = {{8, 8, 1}, {16, 16, 3}, {13, 7, 1}, {32, 8, 2}, {4, 4, 1}, {1, 1, 1}, {64, 32, 4}, {24, 40, 2}};
  uint32_t seed = 1;
  printf("# fmt w h mips seed in_len out_len arbmips fnv\n");
  for (uint32_t fmt : fmts) {
    for (auto& sz : sizes) {
      Rng r{seed};
      const size_t n = gx::texture::texture_source_size(fmt, sz[0], sz[1], sz[2]);
      auto in = bytes(r, n);
      auto out = gfx::convert_texture(fmt, sz[0], sz[1], sz[2], {in.data(), in.size()});
      printf("T %u %u %u %u %u %zu %zu %d %016llx\n", fmt, sz[0], sz[1], sz[2], seed, n, out.data.size(),
             out.hasArbitraryMips ? 1 : 0, (unsigned long long)fnv(out.data.data(), out.data.size()));
      ++seed;
    }
  }
  // Smooth RGBA8 mips (hasArbitraryMips false) vs noise (true) is covered above; add a box-filtered chain.
  const uint32_t ci[] = {GX_TF_C4, GX_TF_C8, GX_TF_C14X2};
  const uint32_t tl[] = {GX_TL_IA8, GX_TL_RGB565, GX_TL_RGB5A3};
  const uint16_t entries[] = {16, 256, 200};
  for (uint32_t fmt : ci) {
    for (uint32_t t : tl) {
      for (uint16_t e : entries) {
        for (int s = 0; s < 3; ++s) {
          const auto& sz = sizes[s];
          Rng r{seed};
          const size_t n = gx::texture::texture_source_size(fmt, sz[0], sz[1], sz[2]);
          auto in = bytes(r, n);
          auto tlut = bytes(r, e * 2u);
          auto out = gfx::convert_texture_palette(fmt, sz[0], sz[1], sz[2], {in.data(), in.size()},
                                                  static_cast<GXTlutFmt>(t), e, {tlut.data(), tlut.size()});
          printf("P %u %u %u %u %u %u %u %zu %zu %d %016llx\n", fmt, t, e, sz[0], sz[1], sz[2], seed, n,
                 out.data.size(), out.hasArbitraryMips ? 1 : 0,
                 (unsigned long long)fnv(out.data.data(), out.data.size()));
          ++seed;
        }
      }
    }
  }
  return 0;
}

// ---------------------------------------------------------------- shaders
template <typename T>
T pick(Rng& r, std::initializer_list<T> l) {
  return *(l.begin() + r.below(static_cast<uint32_t>(l.size())));
}

gx::ShaderConfig random_config(Rng& r, uint32_t& normalAttachment) {
  gx::ShaderConfig c{};
  c.fogType = r.chance(25) ? pick<u8>(r, {2, 4, 5, 6, 7, 10, 12, 13, 14, 15}) : 0;
  c.fogRangeEnabled = c.fogType != 0 && r.chance(30);
  c.lineMode = r.chance(15) ? r.below(4) : 0;
  c.pnPalette = c.lineMode == 0 && r.chance(15);
  u8 offset = 0;
  for (u32 a = GX_VA_PNMTXIDX; a <= GX_VA_TEX7; ++a) {
    auto& m = c.attrs[a];
    bool present;
    if (a == GX_VA_POS) {
      present = true;
    } else if (a == GX_VA_NRM || a == GX_VA_CLR0) {
      present = r.chance(60);
    } else if (a == GX_VA_PNMTXIDX) {
      present = r.chance(30);
    } else if (a <= GX_VA_TEX7MTXIDX) {
      present = r.chance(8);
    } else {
      present = r.chance(25);
    }
    if (!present) continue;
    const bool mtxIdx = a <= GX_VA_TEX7MTXIDX;
    m.attrType = mtxIdx ? GX_DIRECT : pick<u8>(r, {GX_DIRECT, GX_INDEX8, GX_INDEX16});
    if (a == GX_VA_CLR0 || a == GX_VA_CLR1) {
      m.compType = r.below(6);
      m.cnt = 1;
    } else if (mtxIdx) {
      m.compType = 0;
      m.cnt = 1;
    } else {
      m.compType = r.below(5);
      if (a == GX_VA_POS) m.cnt = pick<u8>(r, {2, 3});
      else if (a == GX_VA_NRM) m.cnt = pick<u8>(r, {3, 3, 9});
      else m.cnt = pick<u8>(r, {1, 2});
    }
    m.frac = static_cast<u8>(r.below(16));
    m.offset = offset;
    m.nbt3 = a == GX_VA_NRM && m.cnt == 9 && m.attrType != GX_DIRECT && r.chance(50);
    if (m.attrType == GX_DIRECT) {
      offset += static_cast<u8>(gx::comp_type_size(static_cast<GXAttr>(a), static_cast<GXCompType>(m.compType)) * m.cnt);
      m.le = false;
    } else {
      m.stride = static_cast<u8>(4 + r.below(28));
      m.le = r.chance(50);
      offset += (m.attrType == GX_INDEX8 ? 1 : 2) * (m.nbt3 ? 3 : 1);
    }
  }
  c.vtxStride = offset;
  for (auto& s : c.tevSwapTable) {
    s.red = static_cast<GXTevColorChan>(r.below(4));
    s.green = static_cast<GXTevColorChan>(r.below(4));
    s.blue = static_cast<GXTevColorChan>(r.below(4));
    s.alpha = static_cast<GXTevColorChan>(r.below(4));
  }
  if (r.chance(50)) c.tevSwapTable[0] = gx::TevSwap{};
  c.numIndStages = r.chance(20) ? 1 + r.below(4) : 0;
  for (u32 i = 0; i < c.numIndStages; ++i) {
    auto& s = c.indStages[i];
    s.texCoordId = static_cast<GXTexCoordID>(r.below(8));
    s.texMapId = static_cast<GXTexMapID>(r.below(8));
    s.scaleS = static_cast<GXIndTexScale>(r.below(9));
    s.scaleT = static_cast<GXIndTexScale>(r.below(9));
  }
  c.tevStageCount = 1 + r.below(r.chance(50) ? 3 : 16);
  for (u32 i = 0; i < c.tevStageCount; ++i) {
    auto& s = c.tevStages[i];
    s.colorPass = {static_cast<GXTevColorArg>(r.below(16)), static_cast<GXTevColorArg>(r.below(16)),
                   static_cast<GXTevColorArg>(r.below(16)), static_cast<GXTevColorArg>(r.below(16))};
    s.alphaPass = {static_cast<GXTevAlphaArg>(r.below(8)), static_cast<GXTevAlphaArg>(r.below(8)),
                   static_cast<GXTevAlphaArg>(r.below(8)), static_cast<GXTevAlphaArg>(r.below(8))};
    const auto op = [&](gx::TevOp& o) {
      o.op = r.chance(70) ? pick<GXTevOp>(r, {GX_TEV_ADD, GX_TEV_SUB})
                          : static_cast<GXTevOp>(8 + r.below(8));
      o.bias = static_cast<GXTevBias>(r.below(3));
      o.scale = static_cast<GXTevScale>(r.below(4));
      o.outReg = static_cast<GXTevRegID>(r.below(4));
      o.clamp = r.chance(70);
    };
    op(s.colorOp);
    op(s.alphaOp);
    s.kcSel = static_cast<GXTevKColorSel>(pick<u32>(r, {0, 1, 2, 3, 4, 5, 6, 7, 12, 13, 14, 15}) +
                                          (r.chance(40) ? 0 : 0));
    if (r.chance(40)) s.kcSel = static_cast<GXTevKColorSel>(0x10 + r.below(16));
    s.kaSel = static_cast<GXTevKAlphaSel>(r.chance(50) ? r.below(8) : 0x10 + r.below(16));
    const bool tex = r.chance(70);
    s.texMapId = tex ? static_cast<GXTexMapID>(r.below(8)) : GX_TEXMAP_NULL;
    s.texCoordId = tex || r.chance(30) ? static_cast<GXTexCoordID>(r.below(8)) : GX_TEXCOORD_NULL;
    s.channelId = pick<GXChannelID>(r, {GX_COLOR0, GX_COLOR1, GX_ALPHA0, GX_ALPHA1, GX_COLOR0A0, GX_COLOR1A1,
                                        GX_COLOR_ZERO, GX_ALPHA_BUMP, GX_ALPHA_BUMPN, GX_COLOR_NULL});
    s.tevSwapRas = static_cast<GXTevSwapSel>(r.below(4));
    s.tevSwapTex = static_cast<GXTevSwapSel>(r.below(4));
    if (c.numIndStages != 0 && r.chance(60)) {
      s.indTexStage = static_cast<GXIndTexStageID>(r.below(4));
      s.indTexFormat = static_cast<GXIndTexFormat>(r.below(4));
      s.indTexBiasSel = static_cast<GXIndTexBiasSel>(r.below(8));
      s.indTexAlphaSel = static_cast<GXIndTexAlphaSel>(r.below(4));
      s.indTexMtxId = pick<GXIndTexMtxID>(r, {GX_ITM_OFF, GX_ITM_0, GX_ITM_1, GX_ITM_2, GX_ITM_S0, GX_ITM_S1,
                                              GX_ITM_S2, GX_ITM_T0, GX_ITM_T1, GX_ITM_T2});
      s.indTexWrapS = static_cast<GXIndTexWrap>(r.below(7));
      s.indTexWrapT = static_cast<GXIndTexWrap>(r.below(7));
      s.indTexUseOrigLOD = r.chance(50);
      s.indTexAddPrev = r.chance(30);
    }
  }
  bool lighting = false;
  for (auto& cc : c.colorChannels) {
    cc.matSrc = static_cast<GXColorSrc>(r.below(2));
    if (r.chance(35)) {
      cc.lightingEnabled = true;
      lighting = true;
      cc.ambSrc = static_cast<GXColorSrc>(r.below(2));
      cc.diffFn = static_cast<GXDiffuseFn>(r.below(3));
      cc.attnFn = static_cast<GXAttnFn>(r.below(3));
    }
  }
  for (u32 i = 0; i < gx::MaxTexCoord; ++i) {
    if (!r.chance(60)) continue;
    auto& t = c.tcgs[i];
    t.type = r.chance(85) ? pick<GXTexGenType>(r, {GX_TG_MTX2x4, GX_TG_MTX2x4, GX_TG_MTX3x4, GX_TG_SRTG})
                          : static_cast<GXTexGenType>(GX_TG_BUMP0 + r.below(8));
    t.src = static_cast<GXTexGenSrc>(r.below(22));
    t.mtx = r.chance(50) ? GX_IDENTITY
                         : static_cast<GXTexMtx>(r.chance(50) ? 30 + 3 * r.below(10) : 3 * r.below(10));
    t.postMtx = r.chance(70) || lighting ? GX_PTIDENTITY : static_cast<GXPTTexMtx>(64 + 3 * r.below(20));
    t.normalize = r.chance(20);
    t.embossSrc = static_cast<u8>(r.below(8));
  }
  if (r.chance(30)) {
    c.alphaCompare.comp0 = static_cast<GXCompare>(r.below(8));
    c.alphaCompare.ref0 = r.chance(20) ? pick<u32>(r, {0, 255}) : r.below(256);
    c.alphaCompare.op = static_cast<GXAlphaOp>(r.below(4));
    c.alphaCompare.comp1 = static_cast<GXCompare>(r.below(8));
    c.alphaCompare.ref1 = r.chance(20) ? pick<u32>(r, {0, 255}) : r.below(256);
  }
  normalAttachment = r.chance(20) ? 1 : UINT32_MAX;
  return c;
}

void print_config(const gx::ShaderConfig& c, uint32_t na) {
  printf("C %u %u %u %u %u %u", c.fogType, c.vtxStride, c.lineMode, c.fogRangeEnabled, c.pnPalette, na);
  for (const auto& a : c.attrs) {
    printf(" %u %u %u %u %u %u %u %u", a.attrType, a.cnt, a.compType, a.offset, a.stride, a.frac, a.le, a.nbt3);
  }
  for (const auto& s : c.tevSwapTable) printf(" %u %u %u %u", s.red, s.green, s.blue, s.alpha);
  printf(" %u", c.tevStageCount);
  for (u32 i = 0; i < c.tevStageCount; ++i) {
    const auto& s = c.tevStages[i];
    printf(" %u %u %u %u %u %u %u %u", s.colorPass.a, s.colorPass.b, s.colorPass.c, s.colorPass.d, s.alphaPass.a,
           s.alphaPass.b, s.alphaPass.c, s.alphaPass.d);
    for (const auto* o : {&s.colorOp, &s.alphaOp}) printf(" %u %u %u %u %u", o->op, o->bias, o->scale, o->outReg, o->clamp);
    printf(" %u %u %u %u %u %u %u", s.kcSel, s.kaSel, s.texCoordId, s.texMapId, s.channelId, s.tevSwapRas, s.tevSwapTex);
    printf(" %u %u %u %u %u %u %u %u %u", s.indTexStage, s.indTexFormat, s.indTexBiasSel, s.indTexAlphaSel,
           s.indTexMtxId, s.indTexWrapS, s.indTexWrapT, s.indTexUseOrigLOD, s.indTexAddPrev);
  }
  for (const auto& cc : c.colorChannels) printf(" %u %u %u %u %u", cc.matSrc, cc.ambSrc, cc.diffFn, cc.attnFn, cc.lightingEnabled);
  for (const auto& t : c.tcgs) printf(" %u %u %u %u %u %u", t.type, t.src, t.mtx, t.postMtx, t.normalize, t.embossSrc);
  printf(" %u %u %u %u %u", c.alphaCompare.comp0, c.alphaCompare.ref0, c.alphaCompare.op, c.alphaCompare.comp1,
         c.alphaCompare.ref1);
  printf(" %u", c.numIndStages);
  for (u32 i = 0; i < c.numIndStages; ++i) {
    const auto& s = c.indStages[i];
    printf(" %u %u %u %u", s.texCoordId, s.texMapId, s.scaleS, s.scaleT);
  }
  printf("\n");
}

int run_shader(int argc, char** argv) {
  const uint32_t count = argc > 2 ? static_cast<uint32_t>(atoi(argv[2])) : 400;
  const char* dumpDir = argc > 3 ? argv[3] : nullptr;
  printf("# C <config...>  then  W <len> <fnv> <uniformSize>\n");
  for (uint32_t i = 0; i < count; ++i) {
    Rng r{0x9E3779B9u ^ (i * 0x85EBCA6Bu + 1)};
    uint32_t na = UINT32_MAX;
    const auto cfg = random_config(r, na);
    const auto info = gx::build_shader_info(cfg);
    const auto src = gx::build_shader_source(cfg, na);
    print_config(cfg, na);
    printf("W %zu %016llx %u\n", src.size(), (unsigned long long)fnv(src.data(), src.size()), info.uniformSize);
    if (dumpDir) {
      char path[1024];
      snprintf(path, sizeof(path), "%s/%04u.wgsl", dumpDir, i);
      if (FILE* f = fopen(path, "wb")) {
        fwrite(src.data(), 1, src.size(), f);
        fclose(f);
      }
    }
  }
  return 0;
}

// ---------------------------------------------------------------- GX state after API sequences
// Random GX API calls (logged as "O <op> <args>", floats as hex bits), then drain the FIFO and print
// what a GX_TRIANGLES draw with VTXFMT0 would see: the ShaderConfig (C line), the fixed-function
// PipelineConfig fields (P), the uniform bytes' and the WGSL's hashes (U). tests/oracle.rs replays the
// ops through melee_gx::Gx from its own GXInit state and compares.
static uint32_t fbits(float f) {
  uint32_t u;
  memcpy(&u, &f, 4);
  return u;
}
alignas(32) static uint8_t sDummyArray[4096];

struct StateGen {
  Rng r;
  bool pt; // this case may use post-transform matrices (and keeps lighting off)
  uint32_t u(uint32_t n) { return r.below(n); }
  float f() {
    static const float vals[] = {0.f, 1.f, -1.f, 0.5f, 2.f, 0.25f, -0.75f, 3.5f, 100.f, 0.1f, 10.f, -2.5f};
    return vals[u(12)];
  }
  GXColor col() {
    GXColor c;
    c.r = (u8)u(256);
    c.g = (u8)u(256);
    c.b = (u8)u(256);
    c.a = (u8)u(256);
    return c;
  }
};

static u32 rand_chan(StateGen& g) {
  static const u32 c[] = {GX_COLOR0, GX_COLOR1, GX_ALPHA0, GX_ALPHA1, GX_COLOR0A0, GX_COLOR1A1};
  return c[g.u(6)];
}

static void state_op(StateGen& g) {
  const uint32_t op = g.u(36);
  switch (op) {
  case 0: {
    u32 st = g.u(16), tc = g.u(9), tm = g.u(9);
    static const u32 ch[] = {GX_COLOR0, GX_COLOR1, GX_ALPHA0, GX_ALPHA1, GX_COLOR0A0, GX_COLOR1A1, GX_COLOR_ZERO,
                             GX_ALPHA_BUMP, GX_ALPHA_BUMPN, GX_COLOR_NULL};
    u32 c = ch[g.u(10)];
    tc = tc == 8 ? GX_TEXCOORD_NULL : tc;
    tm = tm == 8 ? GX_TEXMAP_NULL : tm;
    printf("O tevorder %u %u %u %u\n", st, tc, tm, c);
    GXSetTevOrder((GXTevStageID)st, (GXTexCoordID)tc, (GXTexMapID)tm, (GXChannelID)c);
    break;
  }
  case 1: {
    u32 st = g.u(16), a = g.u(16), b = g.u(16), c = g.u(16), d = g.u(16);
    printf("O colorin %u %u %u %u %u\n", st, a, b, c, d);
    GXSetTevColorIn((GXTevStageID)st, (GXTevColorArg)a, (GXTevColorArg)b, (GXTevColorArg)c, (GXTevColorArg)d);
    break;
  }
  case 2: {
    u32 st = g.u(16), a = g.u(8), b = g.u(8), c = g.u(8), d = g.u(8);
    printf("O alphain %u %u %u %u %u\n", st, a, b, c, d);
    GXSetTevAlphaIn((GXTevStageID)st, (GXTevAlphaArg)a, (GXTevAlphaArg)b, (GXTevAlphaArg)c, (GXTevAlphaArg)d);
    break;
  }
  case 3:
  case 4: {
    static const u32 ops[] = {0, 1, 8, 9, 10, 11, 12, 13, 14, 15};
    u32 st = g.u(16), o = ops[g.u(10)], bias = g.u(3), scale = g.u(4), clamp = g.u(2), out = g.u(4);
    printf("O %s %u %u %u %u %u %u\n", op == 3 ? "colorop" : "alphaop", st, o, bias, scale, clamp, out);
    if (op == 3)
      GXSetTevColorOp((GXTevStageID)st, (GXTevOp)o, (GXTevBias)bias, (GXTevScale)scale, clamp != 0, (GXTevRegID)out);
    else
      GXSetTevAlphaOp((GXTevStageID)st, (GXTevOp)o, (GXTevBias)bias, (GXTevScale)scale, clamp != 0, (GXTevRegID)out);
    break;
  }
  case 5: {
    u32 st = g.u(16), sel = g.u(28);
    sel = sel < 8 ? sel : sel + 4;
    printf("O kcsel %u %u\n", st, sel);
    GXSetTevKColorSel((GXTevStageID)st, (GXTevKColorSel)sel);
    break;
  }
  case 6: {
    u32 st = g.u(16), sel = g.u(24);
    sel = sel < 8 ? sel : sel + 8;
    printf("O kasel %u %u\n", st, sel);
    GXSetTevKAlphaSel((GXTevStageID)st, (GXTevKAlphaSel)sel);
    break;
  }
  case 7: {
    u32 st = g.u(16), a = g.u(4), b = g.u(4);
    printf("O swapmode %u %u %u\n", st, a, b);
    GXSetTevSwapMode((GXTevStageID)st, (GXTevSwapSel)a, (GXTevSwapSel)b);
    break;
  }
  case 8: {
    u32 id = g.u(4), a = g.u(4), b = g.u(4), c = g.u(4), d = g.u(4);
    printf("O swaptable %u %u %u %u %u\n", id, a, b, c, d);
    GXSetTevSwapModeTable((GXTevSwapSel)id, (GXTevColorChan)a, (GXTevColorChan)b, (GXTevColorChan)c,
                          (GXTevColorChan)d);
    break;
  }
  case 9: {
    u32 n = 1 + g.u(16);
    printf("O numtev %u\n", n);
    GXSetNumTevStages((u8)n);
    break;
  }
  case 10: {
    u32 n = g.u(3);
    printf("O numchans %u\n", n);
    GXSetNumChans((u8)n);
    break;
  }
  case 11: {
    u32 ch = rand_chan(g), en = g.pt ? 0 : g.u(2), amb = g.u(2), mat = g.u(2), mask = g.u(256), diff = g.u(3),
        attn = g.u(3);
    printf("O chanctrl %u %u %u %u %u %u %u\n", ch, en, amb, mat, mask, diff, attn);
    GXSetChanCtrl((GXChannelID)ch, en != 0, (GXColorSrc)amb, (GXColorSrc)mat, mask, (GXDiffuseFn)diff,
                  (GXAttnFn)attn);
    break;
  }
  case 12:
  case 13: {
    u32 ch = rand_chan(g);
    GXColor c = g.col();
    printf("O %s %u %u %u %u %u\n", op == 12 ? "ambcolor" : "matcolor", ch, c.r, c.g, c.b, c.a);
    if (op == 12)
      GXSetChanAmbColor((GXChannelID)ch, c);
    else
      GXSetChanMatColor((GXChannelID)ch, c);
    break;
  }
  case 14:
  case 15: {
    u32 id = g.u(4);
    GXColor c = g.col();
    printf("O %s %u %u %u %u %u\n", op == 14 ? "tevcolor" : "kcolor", id, c.r, c.g, c.b, c.a);
    if (op == 14)
      GXSetTevColor((GXTevRegID)id, c);
    else
      GXSetTevKColor((GXTevKColorID)id, c);
    break;
  }
  case 16: {
    u32 id = g.u(4);
    s16 v[4];
    for (auto& x : v) x = static_cast<s16>(static_cast<int>(g.u(2048)) - 1024);
    printf("O tevcolors10 %u %d %d %d %d\n", id, v[0], v[1], v[2], v[3]);
    GXColorS10 c;
    c.r = v[0];
    c.g = v[1];
    c.b = v[2];
    c.a = v[3];
    GXSetTevColorS10((GXTevRegID)id, c);
    break;
  }
  case 17: {
    u32 n = g.u(9);
    printf("O numtexgens %u\n", n);
    GXSetNumTexGens((u8)n);
    break;
  }
  case 18: {
    u32 dst = g.u(8);
    static const u32 types[] = {GX_TG_MTX2x4, GX_TG_MTX2x4, GX_TG_MTX3x4, GX_TG_SRTG, GX_TG_BUMP0, GX_TG_BUMP3};
    u32 ty = types[g.u(6)];
    u32 src = g.u(21);
    if (ty >= GX_TG_BUMP0 && ty <= GX_TG_BUMP7) src = GX_TG_TEXCOORD0 + g.u(7);
    u32 mtx = g.u(2) ? GX_IDENTITY : GX_TEXMTX0 + 3 * g.u(10);
    u32 norm = g.u(2);
    u32 post = (g.pt && g.u(2)) ? GX_PTTEXMTX0 + 3 * g.u(20) : GX_PTIDENTITY;
    printf("O texgen %u %u %u %u %u %u\n", dst, ty, src, mtx, norm, post);
    GXSetTexCoordGen2((GXTexCoordID)dst, (GXTexGenType)ty, (GXTexGenSrc)src, mtx, norm != 0, post);
    break;
  }
  case 19: {
    static const u32 attrs[] = {GX_VA_PNMTXIDX, GX_VA_TEX0MTXIDX, GX_VA_TEX3MTXIDX, GX_VA_NRM, GX_VA_NRM,
                                GX_VA_CLR0, GX_VA_CLR1, GX_VA_TEX0, GX_VA_TEX1, GX_VA_TEX5, GX_VA_POS};
    u32 a = attrs[g.u(11)];
    u32 t = a <= GX_VA_TEX7MTXIDX ? g.u(2) : g.u(4);
    if (a == GX_VA_POS && t == GX_NONE) t = GX_DIRECT;
    printf("O vtxdesc %u %u\n", a, t);
    GXSetVtxDesc((GXAttr)a, (GXAttrType)t);
    break;
  }
  case 20: {
    static const u32 attrs[] = {GX_VA_POS, GX_VA_NRM, GX_VA_CLR0, GX_VA_CLR1, GX_VA_TEX0, GX_VA_TEX1, GX_VA_TEX4,
                                GX_VA_TEX5, GX_VA_TEX7};
    u32 a = attrs[g.u(9)];
    u32 cnt, ty;
    if (a == GX_VA_CLR0 || a == GX_VA_CLR1) {
      cnt = g.u(2);
      ty = g.u(6);
    } else if (a == GX_VA_NRM) {
      cnt = g.u(3);
      ty = g.u(5);
    } else {
      cnt = g.u(2);
      ty = g.u(5);
    }
    u32 frac = g.u(16);
    printf("O vtxfmt %u %u %u %u\n", a, cnt, ty, frac);
    GXSetVtxAttrFmt(GX_VTXFMT0, (GXAttr)a, (GXCompCnt)cnt, (GXCompType)ty, (u8)frac);
    break;
  }
  case 21: {
    u32 c0 = g.u(8), r0 = g.u(256), o = g.u(4), c1 = g.u(8), r1 = g.u(256);
    printf("O alphacomp %u %u %u %u %u\n", c0, r0, o, c1, r1);
    GXSetAlphaCompare((GXCompare)c0, (u8)r0, (GXAlphaOp)o, (GXCompare)c1, (u8)r1);
    break;
  }
  case 22: {
    u32 m = g.u(4), src = g.u(8), dst = g.u(8), lo = g.u(16);
    printf("O blend %u %u %u %u\n", m, src, dst, lo);
    GXSetBlendMode((GXBlendMode)m, (GXBlendFactor)src, (GXBlendFactor)dst, (GXLogicOp)lo);
    break;
  }
  case 23: {
    u32 c = g.u(2), f = g.u(8), u = g.u(2);
    printf("O zmode %u %u %u\n", c, f, u);
    GXSetZMode(c != 0, (GXCompare)f, u != 0);
    break;
  }
  case 24: {
    static const u32 types[] = {GX_FOG_NONE, GX_FOG_PERSP_LIN, GX_FOG_PERSP_EXP, GX_FOG_PERSP_EXP2,
                                GX_FOG_PERSP_REVEXP, GX_FOG_PERSP_REVEXP2, GX_FOG_ORTHO_LIN, GX_FOG_ORTHO_EXP,
                                GX_FOG_ORTHO_EXP2, GX_FOG_ORTHO_REVEXP, GX_FOG_ORTHO_REVEXP2};
    u32 t = types[g.u(11)];
    float st = 10.f + (float)g.u(100), en = 200.f + (float)g.u(800), nz = 0.1f * (float)(1 + g.u(10)),
          fz = 1000.f + (float)g.u(10000);
    GXColor c = g.col();
    printf("O fog %u %08x %08x %08x %08x %u %u %u %u\n", t, fbits(st), fbits(en), fbits(nz), fbits(fz), c.r, c.g, c.b,
           c.a);
    GXSetFog((GXFogType)t, st, en, nz, fz, c);
    break;
  }
  case 25: {
    u32 m = g.u(3);
    printf("O cull %u\n", m);
    GXSetCullMode((GXCullMode)m);
    break;
  }
  case 26: {
    u32 kind = g.u(4);
    float m[3][4];
    for (auto& row : m)
      for (auto& x : row) x = g.f();
    u32 id;
    if (kind <= 1)
      id = 3 * g.u(10);
    else if (kind == 2)
      id = GX_TEXMTX0 + 3 * g.u(10);
    else
      id = GX_PTTEXMTX0 + 3 * g.u(20);
    u32 ty = kind == 2 ? g.u(2) : 0;
    printf("O mtx %u %u %u", kind, id, ty);
    for (auto& row : m)
      for (auto& x : row) printf(" %08x", fbits(x));
    printf("\n");
    if (kind == 0)
      GXLoadPosMtxImm(m, id);
    else if (kind == 1)
      GXLoadNrmMtxImm(m, id);
    else
      GXLoadTexMtxImm(m, id, (GXTexMtxType)ty);
    break;
  }
  case 27: {
    float m[4][4];
    for (auto& row : m)
      for (auto& x : row) x = g.f();
    u32 ty = g.u(2);
    printf("O proj %u", ty);
    for (auto& row : m)
      for (auto& x : row) printf(" %08x", fbits(x));
    printf("\n");
    GXSetProjection(m, (GXProjectionType)ty);
    break;
  }
  case 28: {
    float l = (float)g.u(100), t = (float)g.u(100), w = 100.f + (float)g.u(540), h = 100.f + (float)g.u(380);
    float n = g.u(2) ? 0.f : 0.25f, fz = g.u(2) ? 1.f : 0.75f;
    printf("O viewport %08x %08x %08x %08x %08x %08x\n", fbits(l), fbits(t), fbits(w), fbits(h), fbits(n), fbits(fz));
    GXSetViewport(l, t, w, h, n, fz);
    break;
  }
  case 29: {
    static GXLightObj lo;
    float v[12];
    for (auto& x : v) x = g.f();
    GXColor c = g.col();
    u32 id = 1u << g.u(8);
    GXInitLightAttn(&lo, v[0], v[1], v[2], v[3], v[4], v[5]);
    GXInitLightPos(&lo, v[6], v[7], v[8]);
    GXInitLightDir(&lo, v[9], v[10], v[11]);
    GXInitLightColor(&lo, c);
    printf("O light %u %u %u %u %u", id, c.r, c.g, c.b, c.a);
    for (float x : v) printf(" %08x", fbits(x));
    printf("\n");
    GXLoadLightObjImm(&lo, (GXLightID)id);
    break;
  }
  case 30: {
    u32 k = g.u(5);
    if (k == 0) {
      u32 n = g.u(5);
      printf("O numind %u\n", n);
      GXSetNumIndStages((u8)n);
    } else if (k == 1) {
      u32 st = g.u(4), tc = g.u(8), tm = g.u(8);
      printf("O indorder %u %u %u\n", st, tc, tm);
      GXSetIndTexOrder((GXIndTexStageID)st, (GXTexCoordID)tc, (GXTexMapID)tm);
    } else if (k == 2) {
      u32 st = g.u(4), a = g.u(9), b = g.u(9);
      printf("O indscale %u %u %u\n", st, a, b);
      GXSetIndTexCoordScale((GXIndTexStageID)st, (GXIndTexScale)a, (GXIndTexScale)b);
    } else if (k == 3) {
      static const u32 mtxs[] = {0, 1, 2, 3, 5, 6, 7, 9, 10, 11};
      u32 st = g.u(16), is = g.u(4), fmt = g.u(4), bias = g.u(8), mtx = mtxs[g.u(10)], ws = g.u(7), wt = g.u(7),
          ap = g.u(2), lod = g.u(2), al = g.u(4);
      printf("O tevind %u %u %u %u %u %u %u %u %u %u\n", st, is, fmt, bias, mtx, ws, wt, ap, lod, al);
      GXSetTevIndirect((GXTevStageID)st, (GXIndTexStageID)is, (GXIndTexFormat)fmt, (GXIndTexBiasSel)bias,
                       (GXIndTexMtxID)mtx, (GXIndTexWrap)ws, (GXIndTexWrap)wt, ap != 0, lod != 0,
                       (GXIndTexAlphaSel)al);
    } else {
      u32 id = 1 + g.u(3);
      float m[2][3];
      for (auto& row : m)
        for (auto& x : row) x = g.f() * 0.25f;
      s8 e = static_cast<s8>(static_cast<int>(g.u(30)) - 17);
      printf("O indmtx %u %d", id, e);
      for (auto& row : m)
        for (auto& x : row) printf(" %08x", fbits(x));
      printf("\n");
      GXSetIndTexMtx((GXIndTexMtxID)id, m, e);
    }
    break;
  }
  case 31: {
    u32 k = g.u(3), v = g.u(2), a = g.u(256);
    printf("O update %u %u %u\n", k, v, a);
    if (k == 0)
      GXSetColorUpdate(v != 0);
    else if (k == 1)
      GXSetAlphaUpdate(v != 0);
    else
      GXSetDstAlpha(v != 0, (u8)a);
    break;
  }
  case 32: {
    u32 a = GX_VA_POS + g.u(12), stride = 1 + g.u(40), le = g.u(2), size = 64 + g.u(4000);
    printf("O array %u %u %u %u\n", a, size, stride, le);
    GXSetArray((GXAttr)a, sDummyArray, size, (u8)stride, le != 0);
    break;
  }
  case 33: {
    u32 id = 3 * g.u(10);
    printf("O curmtx %u\n", id);
    GXSetCurrentMtx(id);
    break;
  }
  case 34: {
    u32 k = g.u(3), w = g.u(256), o = g.u(6), c = g.u(8), en = g.u(2), ss = 1 + g.u(1024), ts = 1 + g.u(1024);
    printf("O lp %u %u %u %u %u %u %u\n", k, w, o, c, en, ss, ts);
    if (k == 0)
      GXSetLineWidth((u8)w, (GXTexOffset)o);
    else if (k == 1)
      GXSetPointSize((u8)w, (GXTexOffset)o);
    else
      GXSetTexCoordScaleManually((GXTexCoordID)c, en != 0, (u16)ss, (u16)ts);
    break;
  }
  default: {
    u32 c = g.u(8), l = g.u(2), p = g.u(2);
    printf("O texoffsets %u %u %u\n", c, l, p);
    GXEnableTexOffsets((GXTexCoordID)c, l != 0, p != 0);
    break;
  }
  }
}

static void print_state_result() {
  gx::PipelineConfig pc{};
  gx::populate_pipeline_config(pc, GX_TRIANGLES, GX_VTXFMT0);
  print_config(pc.shaderConfig, UINT32_MAX);
  const auto info = gx::build_shader_info(pc.shaderConfig);
  ByteBuffer ub;
  gx::build_uniform_bytes(info, ub);
  const auto src = gx::build_shader_source(pc.shaderConfig, UINT32_MAX);
  printf("P %u %u %u %u %u %u %u %08x %08x %08x %u %u %u %u\n", pc.depthFunc, pc.cullMode, pc.blendMode,
         pc.blendFacSrc, pc.blendFacDst, pc.blendOp, pc.dstAlpha, pc.polygonOffsetBits, pc.polygonOffsetScaleBits,
         pc.polygonOffsetClampBits, pc.depthCompare, pc.depthUpdate, pc.alphaUpdate, pc.colorUpdate);
  printf("U %u %zu %016llx W %zu %016llx\n", info.uniformSize, ub.size(), (unsigned long long)fnv(ub.data(), ub.size()),
         src.size(), (unsigned long long)fnv(src.data(), src.size()));
}

static void run_state_cases(uint32_t count) {
  printf("# per case: O lines (GX calls), then C (ShaderConfig), P (pipeline), U (uniform + WGSL hashes)\n");
  printf("S init 0\n");
  GXFlush();
  aurora::gx::fifo::drain();
  print_state_result();
  for (uint32_t i = 0; i < count; ++i) {
    StateGen g{Rng{0xC0FFEEu + i * 7919u}, (i & 1) != 0};
    printf("S %u %u\n", i, g.pt ? 1 : 0);
    if (g.pt) {
      for (u32 c : {GX_COLOR0, GX_COLOR1, GX_ALPHA0, GX_ALPHA1}) {
        printf("O chanctrl %u 0 0 0 0 0 2\n", c);
        GXSetChanCtrl((GXChannelID)c, false, GX_SRC_REG, GX_SRC_REG, 0, GX_DF_NONE, GX_AF_NONE);
      }
    } else {
      for (u32 t = 0; t < 8; ++t) {
        printf("O texgen %u 1 %u 60 0 125\n", t, GX_TG_TEX0 + t);
        GXSetTexCoordGen2((GXTexCoordID)t, GX_TG_MTX2x4, (GXTexGenSrc)(GX_TG_TEX0 + t), GX_IDENTITY, false,
                          GX_PTIDENTITY);
      }
    }
    const uint32_t n = 5 + g.u(40);
    for (uint32_t k = 0; k < n; ++k) state_op(g);
    GXFlush();
    aurora::gx::fifo::drain();
    print_state_result();
  }
}

// ---------------------------------------------------------------- render
#include "scene.inc"

template <typename F>
int with_aurora(int argc, char** argv, F&& body) {
  AuroraConfig config{};
  config.appName = "melee_gx oracle";
  config.windowWidth = 640;
  config.windowHeight = 480;
  config.desiredBackend = getenv("ORACLE_BACKEND") ? (AuroraBackend)atoi(getenv("ORACLE_BACKEND")) : BACKEND_D3D11;
  AuroraInfo info = aurora_initialize(argc, argv, &config);
  (void)info;
  int rc = 1;
  bool done = false;
  for (int frame = 0; frame < 900 && !done; ++frame) {
    const AuroraEvent* ev = aurora_update();
    while (ev != nullptr && ev->type != AURORA_NONE) {
      if (ev->type == AURORA_EXIT) done = true;
      ++ev;
    }
    if (done || !aurora_begin_frame()) continue;
    done = body(frame, rc);
    aurora_end_frame();
  }
  // aurora_shutdown() faults in this standalone harness after Dawn's device is destroyed; the
  // results are already written, so leave without it.
  fflush(stdout);
  fflush(stderr);
  std::_Exit(rc);
}

int run_render(int argc, char** argv) {
  if (argc < 3) {
    fprintf(stderr, "usage: oracle render <out.png>\n");
    return 2;
  }
  const std::string out = argv[2];
  std::filesystem::remove(out);
  if (const char* m = getenv("SCENE_MASK")) scene_mask = static_cast<unsigned>(atoi(m));
  int shotFrame = 60;
  if (const char* m = getenv("SHOT_FRAME")) shotFrame = atoi(m);
  const bool noScene = getenv("NO_SCENE") != nullptr;
  int requestedAt = -1;
  return with_aurora(argc, argv, [&](int frame, int& rc) {
    if (frame == 0) GXInit(nullptr, 0); // the game calls GXInit itself; aurora_initialize does not
    if (!noScene) scene_draw();
    // Aurora skips a draw while its pipeline compiles (asynchronously); by frame 120 they are built.
    // (GXSetPipelineWaitAURORA would wait instead, but faults in this standalone harness.)
    if (frame == shotFrame) {
      aurora_request_screenshot(out.c_str());
      requestedAt = frame;
    }
    if (requestedAt >= 0 && std::filesystem::exists(out) && std::filesystem::file_size(out) > 0) {
      std::this_thread::sleep_for(std::chrono::milliseconds(300));
      rc = 0;
      return true;
    }
    return false;
  });
}

int run_state(int argc, char** argv) {
  const uint32_t count = argc > 2 ? static_cast<uint32_t>(atoi(argv[2])) : 300;
  return with_aurora(argc, argv, [&](int frame, int& rc) {
    if (frame < 2) return false;
    GXInit(nullptr, 0);
    run_state_cases(count);
    fflush(stdout);
    rc = 0;
    return true;
  });
}

int run_shader_init(int argc, char** argv) {
  return with_aurora(argc, argv, [&](int frame, int& rc) {
    (void)frame;
    rc = run_shader(argc, argv);
    fflush(stdout);
    return true;
  });
}
} // namespace


// Crash reporter (no debugger on this machine): print the faulting frames with symbols.
#include <windows.h>
#include <dbghelp.h>
#pragma comment(lib, "dbghelp.lib")
static LONG WINAPI oracle_crash(EXCEPTION_POINTERS* ep) {
  HANDLE proc = GetCurrentProcess();
  SymInitialize(proc, nullptr, TRUE);
  CONTEXT ctx = *ep->ContextRecord;
  STACKFRAME frame{};
  frame.AddrPC.Offset = ctx.Eip;
  frame.AddrPC.Mode = AddrModeFlat;
  frame.AddrFrame.Offset = ctx.Ebp;
  frame.AddrFrame.Mode = AddrModeFlat;
  frame.AddrStack.Offset = ctx.Esp;
  frame.AddrStack.Mode = AddrModeFlat;
  fprintf(stderr, "CRASH code %08lx at %p\n", ep->ExceptionRecord->ExceptionCode, ep->ExceptionRecord->ExceptionAddress);
  for (int i = 0; i < 40; ++i) {
    if (!StackWalk(IMAGE_FILE_MACHINE_I386, proc, GetCurrentThread(), &frame, &ctx, nullptr, SymFunctionTableAccess,
                   SymGetModuleBase, nullptr))
      break;
    char buf[sizeof(SYMBOL_INFO) + 256];
    auto* sym = reinterpret_cast<SYMBOL_INFO*>(buf);
    sym->SizeOfStruct = sizeof(SYMBOL_INFO);
    sym->MaxNameLen = 255;
    DWORD64 disp = 0;
    IMAGEHLP_LINE64 line{};
    line.SizeOfStruct = sizeof(line);
    DWORD ldisp = 0;
    const bool hasSym = SymFromAddr(proc, frame.AddrPC.Offset, &disp, sym);
    const bool hasLine = SymGetLineFromAddr64(proc, frame.AddrPC.Offset, &ldisp, &line);
    fprintf(stderr, "  #%d %08lx %s %s:%lu\n", i, (unsigned long)frame.AddrPC.Offset, hasSym ? sym->Name : "?",
            hasLine ? line.FileName : "", hasLine ? line.LineNumber : 0);
  }
  fflush(stderr);
  return EXCEPTION_EXECUTE_HANDLER;
}

int main(int argc, char** argv) {
  SetUnhandledExceptionFilter(oracle_crash);
  if (argc < 2) {
    fprintf(stderr, "usage: oracle tex|shader [n] [dumpdir]|render <png>\n");
    return 2;
  }
  const std::string mode = argv[1];
  if (mode == "tex") return run_tex();
  if (mode == "shader") return run_shader_init(argc, argv);
  if (mode == "render") return run_render(argc, argv);
  if (mode == "state") return run_state(argc, argv);
  return 2;
}
