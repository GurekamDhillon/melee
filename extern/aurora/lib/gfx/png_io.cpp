#include "png_io.hpp"

#include "../io.hpp"
#include "../webgpu/gpu.hpp"
#include "png.h"

#include <cstring>
#include <mutex>
#include <string>
#include <thread>
#include <vector>

static aurora::Module Log("aurora::gfx::png");

namespace aurora::gfx::png {

struct PngStructs {
  png_structp pStruct = nullptr;
  png_infop pInfo = nullptr;

  ~PngStructs() {
    png_destroy_read_struct(&pStruct, &pInfo, nullptr);
  }
};

struct MemoryCursor {
  ArrayRef<uint8_t> bytes;
  size_t pos = 0;
};

static void readPngData(png_structp png, png_bytep data, const size_t length) {
  auto* cursor = static_cast<MemoryCursor*>(png_get_io_ptr(png));
  if (length > cursor->bytes.size() - cursor->pos) {
    png_error(png, "unexpected end of data");
  }
  std::memcpy(data, cursor->bytes.data() + cursor->pos, length);
  cursor->pos += length;
}

std::optional<ConvertedTexture> parse_png_bytes(ArrayRef<uint8_t> bytes) noexcept {
  PngStructs structs{};
  MemoryCursor cursor{bytes};

  structs.pStruct = png_create_read_struct(PNG_LIBPNG_VER_STRING, nullptr, nullptr, nullptr);
  if (!structs.pStruct) {
    Log.error("png_create_read_struct failed");
    return std::nullopt;
  }

  structs.pInfo = png_create_info_struct(structs.pStruct);
  if (!structs.pInfo) {
    Log.error("png_create_info_struct failed");
    return std::nullopt;
  }

  // I'm scared of putting any locals after that setjmp.
  std::vector<png_bytep> rowPointers;
  ByteBuffer imageData{};
  png_uint_32 width, height;
  int bit_depth, color_type, interlace_type, compression_type, filter_type;
  size_t rowBytes;
  int i;

  if (setjmp(png_jmpbuf(structs.pStruct))) {
    Log.error("libpng encountered an error");
    return std::nullopt;
  }

  png_set_read_fn(structs.pStruct, &cursor, readPngData);
  png_read_info(structs.pStruct, structs.pInfo);

  if (!png_get_IHDR(structs.pStruct, structs.pInfo, &width, &height, &bit_depth, &color_type, &interlace_type, &compression_type, &filter_type)) {
    Log.error("libpng unable to read IHDR");
    return std::nullopt;
  }

  // Always read as RGBA8.
  png_set_gray_to_rgb(structs.pStruct);
  png_set_filler(structs.pStruct, 0xFF, PNG_FILLER_AFTER);
  png_set_expand(structs.pStruct);
  png_set_strip_16(structs.pStruct);

  png_read_update_info(structs.pStruct, structs.pInfo);
  rowBytes = png_get_rowbytes(structs.pStruct, structs.pInfo);
  rowPointers.resize(height);

  imageData.append_zeroes(rowBytes * height);

  for (i = 0; i < height; i++) {
    rowPointers[i] = imageData.data() + i * rowBytes;
  }

  png_read_image(structs.pStruct, rowPointers.data());
  png_read_end(structs.pStruct, nullptr);

  return ConvertedTexture{
    .format = wgpu::TextureFormat::RGBA8Unorm,
    .width = width,
    .height = height,
    .mips = 1,
    .data = std::move(imageData)
  };
}

std::optional<ConvertedTexture>
load_png_file(const std::filesystem::path& path) noexcept {
  const auto bytes = io::read_file(path);
  if (!bytes.has_value()) {
    Log.error("failed to open file: {}", io::fs_path_to_string(path));
    return std::nullopt;
  }
  return parse_png_bytes(*bytes);
}
}

// ---- screenshots (port patch) --------------------------------------------------------------------
namespace aurora::gfx::png {
namespace {
std::mutex g_shotMutex;
std::vector<std::string> g_shotQueue;
struct ShotInFlight {
  std::string path;
  wgpu::Buffer buffer;
  uint32_t width = 0;
  uint32_t height = 0;
  uint32_t bytesPerRow = 0;
  bool bgra = false;
};
std::vector<ShotInFlight> g_shotsEncoded; // render worker only

void write_png(ShotInFlight shot, std::vector<uint8_t> rows) {
  std::vector<uint8_t> rgba(static_cast<size_t>(shot.width) * shot.height * 4);
  for (uint32_t y = 0; y < shot.height; ++y) {
    const uint8_t* src = rows.data() + static_cast<size_t>(y) * shot.bytesPerRow;
    uint8_t* dst = rgba.data() + static_cast<size_t>(y) * shot.width * 4;
    for (uint32_t x = 0; x < shot.width; ++x) {
      dst[x * 4 + 0] = shot.bgra ? src[x * 4 + 2] : src[x * 4 + 0];
      dst[x * 4 + 1] = src[x * 4 + 1];
      dst[x * 4 + 2] = shot.bgra ? src[x * 4 + 0] : src[x * 4 + 2];
      dst[x * 4 + 3] = 255; // the EFB's alpha is not coverage; a screenshot is opaque
    }
  }
  png_image image{};
  image.version = PNG_IMAGE_VERSION;
  image.width = shot.width;
  image.height = shot.height;
  image.format = PNG_FORMAT_RGBA;
  if (png_image_write_to_file(&image, shot.path.c_str(), 0, rgba.data(), 0, nullptr) == 0) {
    Log.warn("screenshot: could not write {}: {}", shot.path, image.message);
  } else {
    Log.info("screenshot: {} ({}x{})", shot.path, shot.width, shot.height);
  }
  png_image_free(&image);
}
} // namespace

void encode_screenshot(const wgpu::CommandEncoder& encoder) noexcept {
  std::vector<std::string> paths;
  {
    std::lock_guard lock{g_shotMutex};
    paths.swap(g_shotQueue);
  }
  if (paths.empty()) {
    return;
  }
  const auto& source = webgpu::present_source();
  const auto format = source.format;
  const bool bgra = format == wgpu::TextureFormat::BGRA8Unorm || format == wgpu::TextureFormat::BGRA8UnormSrgb;
  const bool rgba = format == wgpu::TextureFormat::RGBA8Unorm || format == wgpu::TextureFormat::RGBA8UnormSrgb;
  if (!source.texture || (!bgra && !rgba)) {
    Log.warn("screenshot: unsupported frame buffer format {}", static_cast<uint32_t>(format));
    return;
  }
  const uint32_t width = source.size.width;
  const uint32_t height = source.size.height;
  const uint32_t bytesPerRow = (width * 4 + 255) & ~255u;
  for (auto& path : paths) {
    ShotInFlight shot;
    shot.path = std::move(path);
    shot.width = width;
    shot.height = height;
    shot.bytesPerRow = bytesPerRow;
    shot.bgra = bgra;
    const wgpu::BufferDescriptor desc{
        .label = "Screenshot readback",
        .usage = wgpu::BufferUsage::CopyDst | wgpu::BufferUsage::MapRead,
        .size = static_cast<uint64_t>(bytesPerRow) * height,
    };
    shot.buffer = webgpu::g_device.CreateBuffer(&desc);
    const wgpu::TexelCopyTextureInfo src{.texture = source.texture};
    const wgpu::TexelCopyBufferInfo dst{
        .layout = {.offset = 0, .bytesPerRow = bytesPerRow, .rowsPerImage = height},
        .buffer = shot.buffer,
    };
    const wgpu::Extent3D extent{width, height, 1};
    encoder.CopyTextureToBuffer(&src, &dst, &extent);
    g_shotsEncoded.push_back(std::move(shot));
  }
}

void after_submit_screenshot() noexcept {
  for (auto& shot : g_shotsEncoded) {
    auto* heap = new ShotInFlight(std::move(shot));
    heap->buffer.MapAsync(
        wgpu::MapMode::Read, 0, static_cast<uint64_t>(heap->bytesPerRow) * heap->height,
        wgpu::CallbackMode::AllowSpontaneous, [heap](wgpu::MapAsyncStatus status, wgpu::StringView message) {
          if (status == wgpu::MapAsyncStatus::Success) {
            const size_t size = static_cast<size_t>(heap->bytesPerRow) * heap->height;
            const auto* mapped = static_cast<const uint8_t*>(heap->buffer.GetConstMappedRange(0, size));
            std::vector<uint8_t> rows;
            if (mapped != nullptr) {
              rows.assign(mapped, mapped + size);
            }
            heap->buffer.Unmap();
            if (!rows.empty()) {
              ShotInFlight shot = *heap;
              shot.buffer = {};
              std::thread(write_png, std::move(shot), std::move(rows)).detach();
            }
          } else {
            Log.warn("screenshot: readback failed for {}: {}", heap->path, std::string_view{message});
          }
          delete heap;
        });
  }
  g_shotsEncoded.clear();
}
} // namespace aurora::gfx::png

extern "C" void aurora_request_screenshot(const char* path) {
  if (path == nullptr || path[0] == 0) {
    return;
  }
  std::lock_guard lock{aurora::gfx::png::g_shotMutex};
  aurora::gfx::png::g_shotQueue.emplace_back(path);
}
