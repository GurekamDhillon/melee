#pragma once

#include "gx.hpp"

#include <bit>
#include <cstring>
#include <span>

namespace aurora::gx::fifo {

constexpr u32 reg_get(const u32 reg, const u32 size, const u32 shift) noexcept {
  return reg >> shift & (1u << size) - 1;
}

void handle_bp(u32 value) noexcept;
void handle_cp(u8 addr, u32 value) noexcept;
void handle_xf(u16 addr, std::span<const u8> data) noexcept;

bool copy_xf_data(u32 addr, const u8* data, u32 len, std::endian e) noexcept;

// Frame interpolation (port patch; see aurora_frame_replay in fifo.cpp). Position/normal matrix and
// projection loads are blended from the previous frame's value for the same load toward the value
// in the stream. A load is identified by its XF address, the POS array bound at the time and how
// many times that pair has occurred so far in the frame.
namespace interp {
// A real (game-drawn) frame starts: the last real frame's values become the blend source, and this
// frame's values are recorded. alpha 1 = no blending.
void begin_real_frame(bool enabled, float alpha) noexcept;
// A replay of the last real frame starts: blend from the frame before it, record nothing.
void begin_replay(float alpha) noexcept;
void end() noexcept;
// Blend the matrix slots loaded since the last draw (called at the top of every draw).
void before_draw() noexcept;
void take_stats(u32* blended, u32* rejected, u32* missing) noexcept;
} // namespace interp

} // namespace aurora::gx::fifo
