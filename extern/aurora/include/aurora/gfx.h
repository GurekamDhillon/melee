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
  /// Draws that had to block on a pipeline still compiling (GXSetPipelineWaitAURORA), and the
  /// total time they blocked (microseconds). Each is a hitch: a pipeline no warm-up covered.
  uint32_t pipelineWaitHits;
  uint32_t pipelineWaitUs;
  /// Pipeline compile workers running (AURORA_PIPELINE_WORKERS).
  uint32_t pipelineWorkers;
} AuroraStats;

/// Pipeline tag bits, recorded per pipeline config in the pipeline cache (and its seed).
/// MUST_DRAW: drawn while GXSetPipelineWaitAURORA was on (item models).
#define AURORA_PIPELINE_TAG_MUST_DRAW 1u

/// Queue every known pipeline config carrying any of `tagMask` at normal priority for the current
/// scene layout, ahead of the background warm-up. They count in urgentPipelinesPending until built,
/// so a loading screen that waits on that also waits on them. Returns how many were not built yet.
uint32_t aurora_prewarm_tagged_pipelines(uint32_t tagMask);
/// How many known pipeline configs carry any of `tagMask`. The boot warm-up builds MUST_DRAW
/// configs first, so a frontend waiting on the first N seed builds should add this to N.
uint32_t aurora_count_tagged_pipelines(uint32_t tagMask);

const AuroraStats* aurora_get_stats();
float aurora_get_fps();
/// steady_clock time (ns) at which the render worker last returned from Present(), and how many
/// presents have completed. Lets an application measure input-to-present latency.
int64_t aurora_get_last_present_ns();
uint32_t aurora_get_present_count();

/// Frame interpolation (port patch). With replay enabled, every frame's GX command stream and the
/// state it started from are kept. Between two game frames, a frame opened with
/// aurora_frame_replay_mark(true) + aurora_begin_frame() can call aurora_frame_replay(alpha) to draw
/// the last game frame again with its position/normal matrices and projection blended `alpha` of
/// the way from the previous game frame's values (0 = previous frame's pose, 1 = its own); then
/// aurora_end_frame() and aurora_frame_replay_mark(false). aurora_frame_set_alpha sets the blend
/// used while the game's own next frame is processed (1 = none).
void aurora_frame_replay_enable(bool enable);
bool aurora_frame_replay_available();
void aurora_frame_set_alpha(float alpha);
void aurora_frame_replay_mark(bool replayFrame);
bool aurora_frame_replay(float alpha);
/// True when aurora_begin_frame() would not block waiting for the render worker.
bool aurora_frame_slot_available();

/// Write the next presented frame to a PNG at `path` (port patch): the final image at the render
/// resolution, without the ImGui overlay. Asynchronous - the file appears a few frames later.
void aurora_request_screenshot(const char* path);
/// Matrix loads blended / not blended because the pairing looked wrong / with no match in the
/// previous frame, since the last call.
void aurora_frame_interp_stats(uint32_t* blended, uint32_t* rejected, uint32_t* missing);

void aurora_enable_vsync(bool enabled);

#ifdef __cplusplus
}
#endif

#endif
