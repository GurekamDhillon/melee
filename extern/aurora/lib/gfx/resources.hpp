#pragma once

#include "types.hpp"

namespace aurora::gfx {
inline constexpr bool UseTextureBuffer = true;
inline constexpr uint64_t UniformBufferSize = 25165824; // 24 MiB
// 12 MiB: four 804-piece Ultimate Soras (about 1.3 MB of vertices each) plus the stage overflowed 5 MiB at
// match start and aborted in push_verts (a silent fastfail). An overflow now drops the rest of the frame's
// draws instead (recording.cpp push), but the arena should not overflow in a normal match.
inline constexpr uint64_t VertexBufferSize = 12582912;  // 12 MiB
inline constexpr uint64_t IndexBufferSize = 2097152;    // 2 MiB
// Four palette Soras (LAB, FD, uncapped) peaked at 10,573 KB here: about 1,370 palette loads a
// frame plus indexed-array snapshots, stage and effects (8 MiB overflowed and dropped the frame's
// tail draws). 24 MiB is about 2x that peak.
inline constexpr uint64_t StorageBufferSize = 25165824; // 24 MiB
inline constexpr uint64_t TextureUploadSize = 25165824; // 24 MiB

namespace detail {
struct Resources {
  wgpu::Buffer vertexBuffer;
  wgpu::Buffer uniformBuffer;
  wgpu::Buffer indexBuffer;
  wgpu::Buffer storageBuffer;
  wgpu::BindGroupLayout staticBindGroupLayout;
  wgpu::BindGroup staticBindGroup;
  wgpu::BindGroupLayout uniformBindGroupLayout;
  wgpu::BindGroup uniformBindGroup;
  wgpu::Limits limits;
  AuroraStats stats{};
};

Resources& resources() noexcept;
} // namespace detail
} // namespace aurora::gfx
