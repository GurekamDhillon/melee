#pragma once
#include <aurora/surface_state.hpp>
#include "../webgpu/gpu.hpp"
#include <chrono>
#include <condition_variable>
#include <cstring>

namespace aurora::gx::surface {
inline gw_surface::Registry registry;
// GX processing thread only; FIFO copies parameters and interpolation replays them.
inline uint32_t activeProgram = 0;
inline gw_surface::Params activeParams;
inline std::string contract() {
  return R"(
struct GdSurfaceInput {
  vertex_color: vec4f,
  normal: vec3f,
  eye_position: vec3f,
  uv0: vec2f,
  time: f32,
  object_id: f32,
  normal_valid: f32,
  params: array<vec4f, 4>,
};
)";
}
inline wgpu::ShaderModule checked_module(const std::string& source, const char* label, std::string& error) {
  if (source.empty() || !webgpu::g_device) { error = "surface shader/device unavailable"; return {}; }
  struct Result {
    std::mutex mutex;
    std::condition_variable cv;
    bool done = false;
    std::string error;
  };
  auto result = std::make_shared<Result>();
  wgpu::ShaderSourceWGSL wgsl{};
  wgsl.code = source.c_str();
  wgpu::ShaderModuleDescriptor desc{};
  desc.nextInChain = &wgsl;
  desc.label = label;
  webgpu::g_device.PushErrorScope(wgpu::ErrorFilter::Validation);
  auto module = webgpu::g_device.CreateShaderModule(&desc);
  webgpu::g_device.PopErrorScope(wgpu::CallbackMode::AllowSpontaneous,
      [result](wgpu::PopErrorScopeStatus status, wgpu::ErrorType type, wgpu::StringView message) {
        std::lock_guard lock(result->mutex);
        if (status != wgpu::PopErrorScopeStatus::Success || type != wgpu::ErrorType::NoError) {
          result->error = message.data ? std::string(message.data,
              message.length == SIZE_MAX ? strlen(message.data) : message.length) : "shader validation failed";
          if (result->error.empty()) result->error = "shader validation failed";
        }
        result->done = true;
        result->cv.notify_all();
      });
  std::unique_lock lock(result->mutex);
  if (!result->cv.wait_for(lock, std::chrono::seconds(5), [&] { return result->done; })) {
    error = "surface shader validation timed out";
    return {};
  }
  error = result->error;
  return error.empty() ? module : wgpu::ShaderModule{};
}
} // namespace aurora::gx::surface
