// Pins gw_pcache_check.h (the pre-boot validator of Aurora's persisted pipeline cache) to the real
// aurora::gx::PipelineConfig layout and to the rules of lib/gx/shader_info.cpp + shader.cpp, using real
// structs rather than hand-built bytes. Header-only inputs: no GPU, no game, no sqlite.
// Build (no run needed to catch a layout drift: the static_asserts fire at compile time):
//   cl /std:c++20 /EHsc /DWEBGPU_DAWN /I extern/aurora/include /I <dawn include dirs, see tools/port/shader_check.py>
//      pc/tests/pcache_guard_test.cpp
#include "../../extern/aurora/lib/gx/pipeline.hpp"
#include "../platform/gw_pcache_check.h"

#include <cassert>
#include <cstddef>
#include <cstdio>
#include <cstring>

using namespace aurora::gx;

// The offsets gw_pcache_check.h reads, against the structs Aurora serializes with memcpy.
static_assert(GW_PCACHE_GX_VERSION == GXPipelineConfigVersion, "Aurora bumped the GX pipeline config version");
static_assert(sizeof(PipelineConfig) == GW_PCACHE_GX_SIZE, "PipelineConfig size changed");
constexpr size_t SC = offsetof(PipelineConfig, shaderConfig);
static_assert(SC + offsetof(ShaderConfig, tevStages) == GWPC_OFF_TEV_STAGES);
static_assert(sizeof(TevStage) == GWPC_TEV_STAGE_STRIDE);
static_assert(MaxTevStages == GWPC_MAX_TEV_STAGES);
static_assert(SC + offsetof(ShaderConfig, tevStageCount) == GWPC_OFF_TEV_STAGE_COUNT);
static_assert(SC + offsetof(ShaderConfig, tcgs) == GWPC_OFF_TCGS);
static_assert(sizeof(TcgConfig) == GWPC_TCG_STRIDE);
static_assert(MaxTexCoord == GWPC_MAX_TEX_COORD);
static_assert(SC + offsetof(ShaderConfig, indStages) == GWPC_OFF_IND_STAGES);
static_assert(sizeof(IndStage) == GWPC_IND_STAGE_STRIDE);
static_assert(MaxIndStages == GWPC_MAX_IND_STAGES);
static_assert(SC + offsetof(ShaderConfig, numIndStages) == GWPC_OFF_NUM_IND_STAGES);
static_assert(SC + offsetof(ShaderConfig, surfaceProgram) == GWPC_OFF_SURFACE_PROGRAM);
static_assert(offsetof(TevStage, colorPass) == GWPC_ST_COLOR_PASS);
static_assert(offsetof(TevStage, alphaPass) == GWPC_ST_ALPHA_PASS);
static_assert(offsetof(TevStage, texCoordId) == GWPC_ST_TEX_COORD);
static_assert(offsetof(TevStage, texMapId) == GWPC_ST_TEX_MAP);
static_assert(offsetof(TevStage, channelId) == GWPC_ST_CHANNEL);
static_assert(offsetof(TevStage, indTexStage) == GWPC_ST_IND_STAGE);
static_assert(offsetof(TevStage, indTexMtxId) == GWPC_ST_IND_MTX);
static_assert(offsetof(TevStage, indTexWrapS) == GWPC_ST_IND_WRAP_S);
static_assert(offsetof(TevStage, indTexWrapT) == GWPC_ST_IND_WRAP_T);
static_assert(offsetof(TevStage, indTexAddPrev) == GWPC_ST_IND_ADD_PREV);
static_assert(offsetof(TcgConfig, type) == GWPC_TCG_TYPE);
static_assert(offsetof(TcgConfig, src) == GWPC_TCG_SRC);
static_assert(offsetof(TcgConfig, embossSrc) == GWPC_TCG_EMBOSS_SRC);
static_assert(sizeof(GXTexGenSrc) == 4 && sizeof(GXTexCoordID) == 4, "enums are 4 bytes in the stored layout");
static_assert(GX_TEXCOORD_NULL == GWPC_TEXCOORD_NULL && GX_TEXMAP_NULL == GWPC_TEXMAP_NULL);
static_assert(GX_CC_TEXC == GWPC_CC_TEXC && GX_CC_TEXA == GWPC_CC_TEXA && GX_CA_TEXA == GWPC_CA_TEXA);
static_assert(GX_ALPHA_BUMP == GWPC_CHANNEL_ALPHA_BUMP && GX_ALPHA_BUMPN == GWPC_CHANNEL_ALPHA_BUMPN);
static_assert(GX_ITM_OFF == GWPC_ITM_OFF && GX_ITW_OFF == GWPC_ITW_OFF);
static_assert(GX_TG_BUMP0 == GWPC_TG_BUMP0 && GX_TG_BUMP7 == GWPC_TG_BUMP7);
static_assert(GX_MAX_TEXGENSRC == 21, "an unset texgen slot is the value the cache check treats as unusable");

static int check(const PipelineConfig& c, char* why = nullptr, size_t n = 0) {
  unsigned char blob[sizeof(PipelineConfig)];
  std::memcpy(blob, &c, sizeof blob);
  return gw_pcache_check_gx(blob, sizeof blob, c.version, why, n);
}

// What lbRefract_80022998 leaves armed: one indirect stage reading texcoord 0, TEV stage 0 using it.
static PipelineConfig leaked_indirect() {
  PipelineConfig c{};
  auto& sc = c.shaderConfig;
  sc.tevStageCount = 1;
  sc.tevStages[0].texMapId = GX_TEXMAP_NULL;
  sc.tevStages[0].texCoordId = GX_TEXCOORD0; // hardware decodes a NULL order as coordinate 0
  sc.tevStages[0].indTexMtxId = GX_ITM_0;
  sc.tevStages[0].indTexBiasSel = GX_ITB_ST;
  sc.numIndStages = 1;
  sc.indStages[0] = {GX_TEXCOORD0, GX_TEXMAP0, GX_ITS_1, GX_ITS_1};
  return c;
}

int main() {
  char why[160];
  PipelineConfig plain{};
  plain.shaderConfig.tevStageCount = 1;
  assert(check(plain) == GW_PCACHE_OK); // untextured, direct: nothing sampled

  // The persisted poison: numTexGens 0, so every tcg slot still holds the default source (21).
  auto bad = leaked_indirect();
  assert(bad.shaderConfig.tcgs[0].src == GX_MAX_TEXGENSRC);
  assert(check(bad, why, sizeof why) == GW_PCACHE_SAMPLED_COORD_UNSET);
  std::printf("rejected as expected: %s\n", why);

  // The same draw with a texgen for coordinate 0 is a legitimate refraction-style config.
  auto good = leaked_indirect();
  good.shaderConfig.tcgs[0] = {GX_TG_MTX3x4, GX_TG_NRM, GX_TEXMTX0, GX_PTTEXMTX0, true, 0, 0, 0};
  assert(check(good) == GW_PCACHE_OK);

  // wrap/add-previous without a matrix reads the TEV stage's own coordinate, not the indirect stage's
  auto wrap = leaked_indirect();
  wrap.shaderConfig.tevStages[0].indTexMtxId = GX_ITM_OFF;
  wrap.shaderConfig.tevStages[0].indTexWrapS = GX_ITW_256;
  assert(check(wrap) == GW_PCACHE_SAMPLED_COORD_UNSET);
  wrap.shaderConfig.tcgs[0].src = GX_TG_TEX0;
  assert(check(wrap) == GW_PCACHE_OK);

  // an indirect stage past numIndStages is skipped by shader_info, so it samples nothing
  auto off = leaked_indirect();
  off.shaderConfig.numIndStages = 0;
  assert(check(off) == GW_PCACHE_OK);

  // an ordinary textured stage
  PipelineConfig tex{};
  tex.shaderConfig.tevStageCount = 1;
  tex.shaderConfig.tevStages[0].colorPass.d = GX_CC_TEXC;
  tex.shaderConfig.tevStages[0].texMapId = GX_TEXMAP0;
  tex.shaderConfig.tevStages[0].texCoordId = GX_TEXCOORD1;
  assert(check(tex) == GW_PCACHE_SAMPLED_COORD_UNSET);
  tex.shaderConfig.tcgs[1].src = GX_TG_TEX0;
  assert(check(tex) == GW_PCACHE_OK);
  tex.shaderConfig.tcgs[1].src = GX_TG_TEXCOORD2; // 12..18: no case in the generator either
  assert(check(tex) == GW_PCACHE_SAMPLED_COORD_UNSET);
  tex.shaderConfig.tevStages[0].texMapId = GX_TEXMAP_NULL; // unsampled again
  assert(check(tex) == GW_PCACHE_OK);

  // emboss: the source texcoord is sampled too
  PipelineConfig bump{};
  bump.shaderConfig.tevStageCount = 1;
  bump.shaderConfig.tevStages[0].colorPass.d = GX_CC_TEXC;
  bump.shaderConfig.tevStages[0].texMapId = GX_TEXMAP0;
  bump.shaderConfig.tevStages[0].texCoordId = GX_TEXCOORD0;
  bump.shaderConfig.tcgs[0] = {GX_TG_BUMP0, GX_TG_TEX0, GX_IDENTITY, GX_PTIDENTITY, false, 3, 0, 0};
  assert(check(bump) == GW_PCACHE_SAMPLED_COORD_UNSET); // coordinate 3 has no texgen
  bump.shaderConfig.tcgs[3].src = GX_TG_TEX0;
  assert(check(bump) == GW_PCACHE_OK);

  // process-local surface program and out-of-range counts
  PipelineConfig sp{};
  sp.shaderConfig.tevStageCount = 1;
  sp.shaderConfig.surfaceProgram = 1;
  assert(check(sp) == GW_PCACHE_SURFACE_PROGRAM);
  PipelineConfig counts{};
  counts.shaderConfig.tevStageCount = 17;
  assert(check(counts) == GW_PCACHE_BAD_COUNTS);
  counts.shaderConfig.tevStageCount = 1;
  counts.shaderConfig.numIndStages = 5;
  assert(check(counts) == GW_PCACHE_BAD_COUNTS);

  // a blob of another layout is not ours to judge
  unsigned char other[2772] = {};
  assert(gw_pcache_check_gx(other, sizeof other, 13, nullptr, 0) == GW_PCACHE_UNKNOWN_LAYOUT);
  PipelineConfig versioned{};
  versioned.version = 15;
  assert(check(versioned) == GW_PCACHE_UNKNOWN_LAYOUT);
  std::puts("pcache_guard_test: ok");
  return 0;
}
