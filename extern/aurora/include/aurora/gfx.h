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
  /// Pipelines something is drawing with right now that are not built yet: queued at normal or
  /// blocking priority (or promoted out of the background queue) and not finished. Unlike
  /// queuedPipelines this ignores the seed's background warm-up, so it reaches 0 as soon as
  /// the current frame has everything it draws - what a loading screen should wait on.
  uint32_t urgentPipelinesPending;
} AuroraStats;

const AuroraStats* aurora_get_stats();
float aurora_get_fps();
/// steady_clock time (ns) at which the render worker last returned from Present(), and how many
/// presents have completed. Lets an application measure input-to-present latency.
int64_t aurora_get_last_present_ns();
uint32_t aurora_get_present_count();

void aurora_enable_vsync(bool enabled);

#ifdef __cplusplus
}
#endif

#endif
