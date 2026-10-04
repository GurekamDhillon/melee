#include "dolphin/gx/GXAurora.h"

#include <limits>

#include "__gx.h"
#include "gx.hpp"
#include "../../window.hpp"

#include "../../gx/fifo.hpp"
#include "../../gx/surface.hpp"
#include <aurora/motion.hpp>

static void GXWriteString(const char* label) {
  auto length = strlen(label);

  if (length > std::numeric_limits<u16>::max()) {
    Log.warn("Debug marker size over u16 max, truncating");
    length = std::numeric_limits<u16>::max();
  }

  GX_WRITE_U16(length);
  GX_WRITE_DATA(label, length);
}

void GXPushDebugGroup(const char* label) {
  GX_WRITE_AURORA(GX_AURORA_DEBUG_GROUP_PUSH);
  GXWriteString(label);
}

void GXPopDebugGroup() { GX_WRITE_AURORA(GX_AURORA_DEBUG_GROUP_POP); }

void GXSetPipelineWaitAURORA(u8 on) {
  GX_WRITE_AURORA(GX_AURORA_PIPELINE_WAIT);
  GX_WRITE_U8(on ? 1 : 0);
}

void GXInsertDebugMarker(const char* label) {
  GX_WRITE_AURORA(GX_AURORA_DEBUG_MARKER_INSERT);
  GXWriteString(label);
}

void GXAuroraCallback(void (*fn)(const void* data, u32 size), const void* data, u32 size) {
  if (fn == nullptr || size > std::numeric_limits<u16>::max()) {
    return;
  }
  GX_WRITE_AURORA(GX_AURORA_CALLBACK);
  GX_WRITE_U64(reinterpret_cast<uintptr_t>(fn));
  GX_WRITE_U16(size);
  if (size > 0) {
    GX_WRITE_DATA(data, size);
  }
}

void GXAuroraMotionCallback(void (*fn)(const void*,u32),const void* data,u32 size) {
  if (!fn || size > 65535) return;
  GX_WRITE_AURORA(GX_AURORA_MOTION_CALLBACK);
  GX_WRITE_U64(reinterpret_cast<uintptr_t>(fn));
  GX_WRITE_U16(static_cast<u16>(size));
  if(size) GX_WRITE_DATA(data,size);
}

void GXAuroraLoadPalette(u32 n, u32 key, const float* data) {
  if (n == 0 || n > GX_AURORA_PALETTE_MAX || data == nullptr) {
    return;
  }
  GX_WRITE_AURORA(GX_AURORA_LOAD_PALETTE);
  GX_WRITE_U16(static_cast<u16>(n));
  GX_WRITE_U32(key);
  GX_WRITE_DATA(data, n * 96u);
}

u32 GXAuroraSurfaceRegister(const char* source, const char* label, char* error, u32 errorSize) {
  std::string message;
  u32 id = 0;
  if (!source || !label || strlen(source) > 65536 || strstr(source, "@group") ||
      strstr(source, "@binding") || strstr(source, "@vertex") || strstr(source, "@fragment")) {
    message = "surface must be <=64 KiB, with no entry points or resource bindings";
  } else {
    id = aurora::gx::surface::registry.find(source);
    if (id) {
      if (error && errorSize) error[0] = 0;
      return id;
    }
    std::string probe = aurora::gx::surface::contract() + source;
    probe += "\n@fragment fn fs_main() -> @location(0) vec4f { return gd_surface(vec4f(1.0), "
             "GdSurfaceInput(vec4f(1.0), vec3f(0.0,0.0,1.0), vec3f(0.0,0.0,1.0), "
             "vec2f(0.0), 0.0, 0.0, 1.0, array<vec4f,4>(vec4f(0.0),vec4f(0.0),vec4f(0.0),vec4f(0.0)))); }";
    auto module = aurora::gx::surface::checked_module(probe, label, message);
    if (module) id = aurora::gx::surface::registry.add(source, label);
    if (module && !id) message = "surface registry exhausted (256 immutable sources per process)";
  }
  if (error && errorSize) snprintf(error, errorSize, "%s", message.c_str());
  return id;
}

void GXAuroraSurface(u32 program, const float* params20) {
  if (!params20) return;
  GX_WRITE_AURORA(GX_AURORA_SURFACE);
  GX_WRITE_U32(program);
  GX_WRITE_DATA(params20, 80);
}

void GXAuroraEndPalette(void) { GX_WRITE_AURORA(GX_AURORA_END_PALETTE); }

void AuroraSetContentAspect(float aspect) {
  aurora::window::set_frame_buffer_aspect(aspect);
}

void AuroraSetViewportPolicy(AuroraViewportPolicy policy) {
  aurora::gx::set_viewport_policy(policy);
}

void AuroraGetRenderSize(u32* width, u32* height) {
  const auto windowSize = aurora::window::get_window_size();
  if (width != nullptr) {
    *width = windowSize.fb_width;
  }
  if (height != nullptr) {
    *height = windowSize.fb_height;
  }
}

void AuroraGXSync() {
  GXFlush();
  aurora::gx::fifo::drain();
}

void GXSetViewportRender(f32 left, f32 top, f32 wd, f32 ht, f32 nearz, f32 farz) {
  GX_WRITE_AURORA(GX_AURORA_LOAD_VIEWPORT_RENDER);
  GX_WRITE_F32(left);
  GX_WRITE_F32(top);
  GX_WRITE_F32(wd);
  GX_WRITE_F32(ht);
  GX_WRITE_F32(nearz);
  GX_WRITE_F32(farz);
}

void GXSetScissorRender(u32 left, u32 top, u32 wd, u32 ht) {
  GX_WRITE_AURORA(GX_AURORA_LOAD_SCISSOR_RENDER);
  GX_WRITE_U32(left);
  GX_WRITE_U32(top);
  GX_WRITE_U32(wd);
  GX_WRITE_U32(ht);
}

void GXSetProjectionFull(const void* mtx) {
  const f32* values = reinterpret_cast<const f32*>(mtx);
  GX_WRITE_AURORA(GX_AURORA_LOAD_PROJECTION_FULL);
  for (int i = 0; i < 16; ++i) {
    GX_WRITE_F32(values[i]);
  }
}

void GX2SetPolygonOffset(f32 mFrontOffset, f32 mFrontScale, f32 mBackOffset, f32 mBackScale, f32 mClamp) {
  GX_WRITE_AURORA(GX2_SET_POLYGON_OFFSET);
  GX_WRITE_F32(mFrontOffset);
  GX_WRITE_F32(mFrontScale);
  GX_WRITE_F32(mBackOffset);
  GX_WRITE_F32(mBackScale);
  GX_WRITE_F32(mClamp);
}

void GXCreateFrameBuffer(u32 width, u32 height) {
  GX_WRITE_AURORA(GX_AURORA_BEGIN_OFFSCREEN);
  GX_WRITE_U32(width);
  GX_WRITE_U32(height);
  aurora::gx::fifo::publish();
}

void GXRestoreFrameBuffer() {
  GX_WRITE_AURORA(GX_AURORA_END_OFFSCREEN);
  aurora::gx::fifo::publish();
}
