#ifndef AURORA_GFX_H
#define AURORA_GFX_H

#ifdef __cplusplus
#include <cstdint>

extern "C" {
#else
#include "stdint.h"
#endif

#if !defined(NDEBUG) && !defined(AURORA_GFX_DEBUG_GROUPS)
#define AURORA_GFX_DEBUG_GROUPS
#endif

void push_debug_group(const char* label);
void pop_debug_group();

typedef struct {
  uint32_t queuedPipelines;
  uint32_t createdPipelines;
  uint32_t drawCallCount;
  uint32_t mergedDrawCallCount;
  uint32_t lastVertSize;
  uint32_t lastUniformSize;
  uint32_t lastIndexSize;
  uint32_t lastStorageSize;
  uint32_t lastTextureUploadSize;
  /// Pipelines built that were queued by the seed warm-up (initial pipeline cache), in the
  /// seed's own order. A frontend can wait on this to know the seed's first N are ready.
  uint32_t seedPipelinesBuilt;
} AuroraStats;

const AuroraStats* aurora_get_stats();
float aurora_get_fps();

void aurora_enable_vsync(bool enabled);

#ifdef __cplusplus
}
#endif

#endif
