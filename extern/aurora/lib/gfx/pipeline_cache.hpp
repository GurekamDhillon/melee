#pragma once

#include "types.hpp"

namespace aurora::gfx::clear {
struct PipelineConfig;
} // namespace aurora::gfx::clear

namespace aurora::gx {
struct PipelineConfig;
} // namespace aurora::gx

namespace aurora::rmlui {
struct PipelineConfig;
} // namespace aurora::rmlui

namespace aurora::gfx {

enum class ShaderType : uint8_t {
  Clear = 0,
  GX = 1,
  Rml = 2,
};

void initialize_pipeline_cache();
void shutdown_pipeline_cache();
void begin_pipeline_frame();
void end_pipeline_frame();
void rebuild_pipeline_cache();

PipelineRef find_pipeline(const gx::PipelineConfig& config, const RenderTargetLayout& layout);
PipelineRef find_pipeline(const clear::PipelineConfig& config, const RenderTargetLayout& layout);
PipelineRef find_pipeline(const rmlui::PipelineConfig& config);

bool get_pipeline(PipelineRef ref, wgpu::RenderPipeline& pipeline);
// Blocks until `ref` (already requested through find_pipeline) is compiled: a request still in a
// queue is taken and built on the calling thread; one a worker already has is waited for. Background
// warm-up is held while it blocks. For draws that must not be skipped (GXSetPipelineWaitAURORA).
// Returns at once when the pipeline is ready or there is no worker. Every real wait is counted
// (AuroraStats.pipelineWaitHits/Us) and logged: it is a pipeline no warm-up covered.
void wait_pipeline(PipelineRef ref);
// Records AURORA_PIPELINE_TAG_MUST_DRAW on the config behind `ref` (persisted with the cache), so
// aurora_prewarm_tagged_pipelines can build it ahead of the next scene that may draw it.
void tag_pipeline_must_draw(PipelineRef ref);

} // namespace aurora::gfx
