#pragma once

#include <string_view>

#include <webgpu/webgpu_cpp.h>

namespace aurora::webgpu::gpu_prof {

// Runtime opt-in; feature must have been requested when the device was created.
bool requested();
bool active();
void emit_cpu(const char* name, uint64_t durationNs);
void initialize();
void shutdown();

void frame_begin(const wgpu::CommandEncoder& encoder);
void frame_end(const wgpu::CommandEncoder& encoder);
void after_submit();
const wgpu::PassTimestampWrites* pass_writes(std::string_view name);

class Zone {
public:
  Zone(const wgpu::CommandEncoder& encoder, std::string_view name);
  ~Zone();
  Zone(const Zone&) = delete;
  Zone& operator=(const Zone&) = delete;

private:
  const wgpu::CommandEncoder* m_encoder = nullptr;
  uint32_t m_endQuery = 0;
};

} // namespace aurora::webgpu::gpu_prof
