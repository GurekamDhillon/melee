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
  uint32_t vertexCount;
} AuroraStats;

/// Pipeline tag bits, recorded per pipeline config in the pipeline cache (and its seed).
/// MUST_DRAW: drawn while GXSetPipelineWaitAURORA was on (item models).
#define AURORA_PIPELINE_TAG_MUST_DRAW 1u
#define AURORA_PIPELINE_TAG_CORE 2u

/// Queue known tagged configs in the background for the scene layout. Unrelated
/// item coverage must not pin a scene's hold; declared warm handles gate staging.
/// Returns how many were not built yet.
uint32_t aurora_prewarm_tagged_pipelines(uint32_t tagMask);
/// How many known pipeline configs carry any of `tagMask`. The boot warm-up builds MUST_DRAW
/// configs first, so a frontend waiting on the first N seed builds should add this to N.
uint32_t aurora_count_tagged_pipelines(uint32_t tagMask);
/// Capture existing GX material traversal, queue compiles, discard draw submission.
/// begin returns 0 on nesting/capacity; pending is -1 stale, -2 unsealed, or remaining.
uint32_t aurora_pipeline_warm_begin(void);
void aurora_pipeline_warm_end(void);
void aurora_pipeline_warm_label(const char* content);
int aurora_pipeline_warm_pending(uint32_t handle);
void aurora_pipeline_warm_release(uint32_t handle);
/// Actual current scene-layout core configurations still unbuilt (not a job count).
uint32_t aurora_pipeline_core_pending(void);
uint32_t aurora_pipeline_core_count(void);

const AuroraStats* aurora_get_stats();
float aurora_get_fps();
/// steady_clock time (ns) at which the render worker last returned from Present(), and how many
/// presents have completed. Lets an application measure input-to-present latency.
int64_t aurora_get_last_present_ns();
uint32_t aurora_get_present_count();
/// Cumulative render-worker queue work, including Present (ns); sample deltas per frame.
int64_t aurora_get_worker_busy_ns();
/// GX vertices decoded for the last completed frame, including display lists.
uint32_t aurora_get_last_vertex_count();
/// Enable the two extra performance counters used by the in-game visualizer.
/* Optional diagnostics sink. Called on the render worker, never from simulation.
 * GPU results are delayed; frame is Aurora's presentation id, not a logic frame.
 * Register before initialization; clear only after the worker has synchronized.
 * Names have callback lifetime. durationNs is a duration, not a CPU timestamp. */
typedef void (*AuroraProfilerSink)(const char* name, uint64_t frame, uint64_t durationNs, void* userdata);
void aurora_profiler_set_sink(AuroraProfilerSink sink, void* userdata);
void aurora_profiler_enable(bool enabled);
bool aurora_profiler_gpu_available(void);
uint64_t aurora_profiler_gpu_dropped_zones(void);
uint64_t aurora_profiler_gpu_dropped_frames(void);
void aurora_perf_enable(bool enabled);
bool aurora_perf_enabled();

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
/// Port patch: 0 = automatic (Mailbox, else Immediate, when vsync is off); 1 = force Immediate
/// (tearing allowed) when vsync is off. Call before aurora_initialize or any time before
/// aurora_enable_vsync.
void aurora_set_present_mode(int mode);
/// Port patch (aurora-gd-gpu-choice-v1): which GPU. "" / "auto" / "discrete" = the high-performance adapter
/// (the default), "integrated" = the low-power one, anything else = the adapter whose name or vendor
/// contains the text (case-insensitive, e.g. "nvidia", "intel", "rtx"). list != 0 also probes the other
/// power preference so the log shows both. Call before aurora_initialize.
void aurora_set_gpu_preference(const char* spec, int list);
/// One line: the adapter in use (name, vendor, ids, type, backend, driver), how it was chosen, the
/// adapter(s) not used, and the surface format and present mode. Valid after aurora_initialize.
const char* aurora_get_gpu_summary(void);

#ifdef __cplusplus
}
#endif

#endif
