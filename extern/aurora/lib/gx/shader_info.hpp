#pragma once

#include "gx.hpp"

namespace aurora::gx {
ShaderInfo build_shader_info(const ShaderConfig& config) noexcept;
gfx::Range build_uniform(const ShaderInfo& info) noexcept;
// The same bytes build_uniform would push, written into `out` (cleared first by the caller) instead of the frame arena.
void build_uniform_bytes(const ShaderInfo& info, ByteBuffer& out) noexcept;
u8 color_channel(GXChannelID id) noexcept;
}; // namespace aurora::gx
