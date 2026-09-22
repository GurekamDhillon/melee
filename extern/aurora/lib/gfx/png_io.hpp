#pragma once

#include <filesystem>
#include <optional>

#include "texture_convert.hpp"
#include "../webgpu/wgpu.hpp"

namespace aurora::gfx::png {
std::optional<ConvertedTexture> parse_png_bytes(ArrayRef<uint8_t> bytes) noexcept;
std::optional<ConvertedTexture> load_png_file(const std::filesystem::path& path) noexcept;

// Screenshots (port patch): aurora_request_screenshot queues a path; the render worker copies the
// frame's final image (the present source, at the render resolution, before the ImGui overlay) into
// a readback buffer in the same command buffer (encode_screenshot), maps it after the submit
// (after_submit_screenshot), and a detached thread writes the PNG. Nothing waits on the GPU.
void encode_screenshot(const wgpu::CommandEncoder& encoder) noexcept;
void after_submit_screenshot() noexcept;
} // namespace aurora::gfx::png
