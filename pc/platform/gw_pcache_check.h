/* Validation of a persisted Aurora GX pipeline configuration (gx::PipelineConfig v14) read back from
 * pipeline_cache.db, on raw bytes so it needs no Aurora headers. Aurora replays every row of that file
 * at boot (lib/gfx/pipeline_cache.cpp load_pipeline_cache_entries) without checking it, and a row whose
 * shader cannot be generated ends the process with aurora FATAL (lib/gx/shader.cpp "unhandled tcg src").
 * The rules mirror build_shader_info / build_shader_source: they flag exactly the configurations that
 * generation would abort or throw on. pc/tests/pcache_guard_test.cpp pins the byte offsets to the real
 * structs with offsetof/static_assert, so an Aurora layout change fails that test instead of silently
 * validating the wrong bytes. */
#ifndef GW_PCACHE_CHECK_H
#define GW_PCACHE_CHECK_H

#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#define GW_PCACHE_GX_VERSION 14 /* aurora::gx::GXPipelineConfigVersion this check understands */
#define GW_PCACHE_GX_SIZE 2776  /* sizeof(aurora::gx::PipelineConfig) at that version */

enum {
  GW_PCACHE_OK = 0,
  GW_PCACHE_UNKNOWN_LAYOUT = 1, /* not a v14 blob of the expected size: not ours to judge, left alone */
  GW_PCACHE_BAD_COUNTS = 2,     /* tevStageCount / numIndStages out of range (Aurora indexes arrays with them) */
  GW_PCACHE_SURFACE_PROGRAM = 3, /* surfaceProgram is a process-local id: meaningless in a later process */
  GW_PCACHE_SAMPLED_COORD_RANGE = 4, /* a stage names a texcoord id outside 0..7 */
  GW_PCACHE_SAMPLED_COORD_UNSET = 5  /* a sampled texcoord has no usable texgen source (default 21, or one
                                      * the generator has no case for): the fatal at shader.cpp:1339 */
};


/* Header-only on purpose: pc/platform objects are listed by hand in the shared link response, so this adds no new
 * object. Returns GW_PCACHE_OK or a reason; `detail` (optional) receives a short description. */

/* Byte offsets into gx::PipelineConfig v14 (little endian, 4-byte enums). ShaderConfig starts at +8.
 * Verified against the real structs by pc/tests/pcache_guard_test.cpp. */
enum {
  GWPC_OFF_TEV_STAGES = 284,
  GWPC_TEV_STAGE_STRIDE = 132,
  GWPC_MAX_TEV_STAGES = 16,
  GWPC_OFF_TEV_STAGE_COUNT = 2396,
  GWPC_OFF_TCGS = 2480,
  GWPC_TCG_STRIDE = 20,
  GWPC_MAX_TEX_COORD = 8,
  GWPC_OFF_IND_STAGES = 2660,
  GWPC_IND_STAGE_STRIDE = 16,
  GWPC_MAX_IND_STAGES = 4,
  GWPC_OFF_NUM_IND_STAGES = 2724,
  GWPC_OFF_SURFACE_PROGRAM = 2728,
  /* inside a TevStage */
  GWPC_ST_COLOR_PASS = 0,  /* 4 x u32 */
  GWPC_ST_ALPHA_PASS = 16, /* 4 x u32 */
  GWPC_ST_TEX_COORD = 80,
  GWPC_ST_TEX_MAP = 84,
  GWPC_ST_CHANNEL = 88,
  GWPC_ST_IND_STAGE = 100,
  GWPC_ST_IND_MTX = 116,
  GWPC_ST_IND_WRAP_S = 120,
  GWPC_ST_IND_WRAP_T = 124,
  GWPC_ST_IND_ADD_PREV = 129, /* u8 */
  /* inside a TcgConfig */
  GWPC_TCG_TYPE = 0,
  GWPC_TCG_SRC = 4,
  GWPC_TCG_EMBOSS_SRC = 17 /* u8 */
};

/* GX enum values used below (dolphin/gx/GXEnum.h) */
enum {
  GWPC_CC_TEXC = 8,
  GWPC_CC_TEXA = 9,
  GWPC_CA_TEXA = 4,
  GWPC_TEXMAP_NULL = 0xFF,
  GWPC_TEXCOORD_NULL = 0xFF,
  GWPC_CHANNEL_ALPHA_BUMP = 7,
  GWPC_CHANNEL_ALPHA_BUMPN = 8,
  GWPC_ITM_OFF = 0,
  GWPC_ITW_OFF = 0,
  GWPC_TG_BUMP0 = 2,
  GWPC_TG_BUMP7 = 9
};

static uint32_t gw_pc_rd32(const unsigned char *b, size_t o) {
  uint32_t v;
  memcpy(&v, b + o, sizeof v);
  return v;
}

/* GXTexGenSrc values lib/gx/shader.cpp has a case for: POS 0, NRM 1, BINRM 2, TANGENT 3, TEX0..TEX7 4..11,
 * COLOR0 19, COLOR1 20. 12..18 (TEXCOORDn) and 21 (GX_MAX_TEXGENSRC, an unset slot) fall to the FATAL. */
static int gw_pc_src_handled(uint32_t src) { return src <= 11 || src == 19 || src == 20; }

static void gw_pc_note(char *detail, size_t n, const char *fmt, unsigned a, unsigned b, unsigned c) {
  if (detail && n) snprintf(detail, n, fmt, a, b, c);
}

static int gw_pcache_check_gx(const unsigned char *blob, size_t size, unsigned version, char *detail,
                       size_t detail_size) {
  if (detail && detail_size) detail[0] = 0;
  if (!blob || size != GW_PCACHE_GX_SIZE || version != GW_PCACHE_GX_VERSION ||
      gw_pc_rd32(blob, 0) != GW_PCACHE_GX_VERSION)
    return GW_PCACHE_UNKNOWN_LAYOUT;

  const uint32_t stages = gw_pc_rd32(blob, GWPC_OFF_TEV_STAGE_COUNT);
  const uint32_t ind = gw_pc_rd32(blob, GWPC_OFF_NUM_IND_STAGES);
  if (stages > GWPC_MAX_TEV_STAGES || ind > GWPC_MAX_IND_STAGES) {
    gw_pc_note(detail, detail_size, "tevStageCount=%u numIndStages=%u (limits %u)", stages, ind, 16);
    return GW_PCACHE_BAD_COUNTS;
  }
  if (gw_pc_rd32(blob, GWPC_OFF_SURFACE_PROGRAM) != 0) {
    gw_pc_note(detail, detail_size, "surfaceProgram=%u", gw_pc_rd32(blob, GWPC_OFF_SURFACE_PROGRAM), 0, 0);
    return GW_PCACHE_SURFACE_PROGRAM;
  }

  unsigned sampled = 0; /* bitmask of texcoords the shader will read */
  for (uint32_t i = 0; i < stages; ++i) {
    const size_t s = GWPC_OFF_TEV_STAGES + (size_t)GWPC_TEV_STAGE_STRIDE * i;
    const uint32_t coord = gw_pc_rd32(blob, s + GWPC_ST_TEX_COORD);
    const uint32_t map = gw_pc_rd32(blob, s + GWPC_ST_TEX_MAP);
    int tex = 0;
    for (int k = 0; k < 4; ++k) {
      const uint32_t c = gw_pc_rd32(blob, s + GWPC_ST_COLOR_PASS + 4 * k);
      const uint32_t a = gw_pc_rd32(blob, s + GWPC_ST_ALPHA_PASS + 4 * k);
      if (c == GWPC_CC_TEXC || c == GWPC_CC_TEXA || a == GWPC_CA_TEXA) tex = 1;
    }
    if (tex && map != GWPC_TEXMAP_NULL) {
      if (coord >= GWPC_MAX_TEX_COORD) { /* shader_info CHECK "tex coord not bound" / bitset range */
        gw_pc_note(detail, detail_size, "stage %u samples texcoord id %u", i, coord, 0);
        return GW_PCACHE_SAMPLED_COORD_RANGE;
      }
      sampled |= 1u << coord;
    }
    const uint32_t ind_stage = gw_pc_rd32(blob, s + GWPC_ST_IND_STAGE);
    if (ind_stage >= ind) continue;
    const uint32_t mtx = gw_pc_rd32(blob, s + GWPC_ST_IND_MTX);
    const uint32_t channel = gw_pc_rd32(blob, s + GWPC_ST_CHANNEL);
    const int uses_ind_stage = mtx != GWPC_ITM_OFF || channel == GWPC_CHANNEL_ALPHA_BUMP || channel == GWPC_CHANNEL_ALPHA_BUMPN;
    const int uses_tev_coord = mtx != GWPC_ITM_OFF || gw_pc_rd32(blob, s + GWPC_ST_IND_WRAP_S) != GWPC_ITW_OFF ||
                               gw_pc_rd32(blob, s + GWPC_ST_IND_WRAP_T) != GWPC_ITW_OFF || blob[s + GWPC_ST_IND_ADD_PREV] != 0;
    if (uses_tev_coord && coord != GWPC_TEXCOORD_NULL) {
      if (coord >= GWPC_MAX_TEX_COORD) {
        gw_pc_note(detail, detail_size, "stage %u indirect uses texcoord id %u", i, coord, 0);
        return GW_PCACHE_SAMPLED_COORD_RANGE;
      }
      sampled |= 1u << coord;
    }
    if (!uses_ind_stage) continue;
    const uint32_t ind_coord = gw_pc_rd32(blob, GWPC_OFF_IND_STAGES + (size_t)GWPC_IND_STAGE_STRIDE * ind_stage);
    if (ind_coord >= GWPC_MAX_TEX_COORD) {
      gw_pc_note(detail, detail_size, "indirect stage %u uses texcoord id %u", ind_stage, ind_coord, 0);
      return GW_PCACHE_SAMPLED_COORD_RANGE;
    }
    sampled |= 1u << ind_coord;
  }
  /* emboss texgens also read their source texcoord */
  for (unsigned t = 0; t < GWPC_MAX_TEX_COORD; ++t) {
    if (!(sampled & (1u << t))) continue;
    const uint32_t type = gw_pc_rd32(blob, GWPC_OFF_TCGS + (size_t)GWPC_TCG_STRIDE * t + GWPC_TCG_TYPE);
    if (type >= GWPC_TG_BUMP0 && type <= GWPC_TG_BUMP7) {
      const unsigned e = blob[GWPC_OFF_TCGS + (size_t)GWPC_TCG_STRIDE * t + GWPC_TCG_EMBOSS_SRC];
      if (e >= GWPC_MAX_TEX_COORD) {
        gw_pc_note(detail, detail_size, "texgen %u emboss source %u", t, e, 0);
        return GW_PCACHE_SAMPLED_COORD_RANGE;
      }
      sampled |= 1u << e;
    }
  }
  for (unsigned t = 0; t < GWPC_MAX_TEX_COORD; ++t) {
    if (!(sampled & (1u << t))) continue;
    const size_t g = GWPC_OFF_TCGS + (size_t)GWPC_TCG_STRIDE * t;
    const uint32_t type = gw_pc_rd32(blob, g + GWPC_TCG_TYPE);
    if (type >= GWPC_TG_BUMP0 && type <= GWPC_TG_BUMP7) continue; /* the emboss branch never reads src */
    const uint32_t src = gw_pc_rd32(blob, g + GWPC_TCG_SRC);
    if (!gw_pc_src_handled(src)) {
      gw_pc_note(detail, detail_size, "texcoord %u is sampled but its texgen source is %u", t, src, 0);
      return GW_PCACHE_SAMPLED_COORD_UNSET;
    }
  }
  return GW_PCACHE_OK;
}

static const char *gw_pcache_reason_name(int reason) {
  switch (reason) {
  case GW_PCACHE_OK: return "ok";
  case GW_PCACHE_UNKNOWN_LAYOUT: return "unknown layout";
  case GW_PCACHE_BAD_COUNTS: return "stage counts out of range";
  case GW_PCACHE_SURFACE_PROGRAM: return "process-local surface program";
  case GW_PCACHE_SAMPLED_COORD_RANGE: return "texcoord id out of range";
  case GW_PCACHE_SAMPLED_COORD_UNSET: return "sampled texcoord without a texgen";
  }
  return "?";
}

#endif
