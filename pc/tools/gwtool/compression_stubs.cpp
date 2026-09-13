// The LLVM release libraries reference zlib and zstd, but the release does not ship those
// libraries. gwtool never compresses anything, so compression entry points fail cleanly;
// crc32 is real because LLVM uses it for more than compression.
#include <cstddef>

extern "C" {

unsigned long crc32(unsigned long crc, const unsigned char *buf, unsigned int len) {
  static unsigned long table[256];
  static bool init = false;
  if (!init) {
    for (unsigned long n = 0; n < 256; n++) {
      unsigned long c = n;
      for (int k = 0; k < 8; k++)
        c = (c & 1) ? 0xEDB88320UL ^ (c >> 1) : c >> 1;
      table[n] = c;
    }
    init = true;
  }
  if (!buf)
    return 0;
  crc = crc ^ 0xFFFFFFFFUL;
  while (len--)
    crc = table[(crc ^ *buf++) & 0xFF] ^ (crc >> 8);
  return crc ^ 0xFFFFFFFFUL;
}

unsigned long compressBound(unsigned long sourceLen) { return sourceLen + 64; }
int compress2(unsigned char *, unsigned long *, const unsigned char *, unsigned long, int) {
  return -2; // Z_STREAM_ERROR
}
int uncompress(unsigned char *, unsigned long *, const unsigned char *, unsigned long) {
  return -3; // Z_DATA_ERROR
}

static const size_t kZstdError = static_cast<size_t>(-1);
void *ZSTD_createCCtx(void) { return nullptr; }
size_t ZSTD_freeCCtx(void *) { return 0; }
size_t ZSTD_CCtx_setParameter(void *, int, int) { return kZstdError; }
unsigned ZSTD_isError(size_t code) { return code > static_cast<size_t>(-120); }
size_t ZSTD_compressBound(size_t srcSize) { return srcSize + 64; }
size_t ZSTD_compress2(void *, void *, size_t, const void *, size_t) { return kZstdError; }
size_t ZSTD_decompress(void *, size_t, const void *, size_t) { return kZstdError; }
const char *ZSTD_getErrorName(size_t) { return "zstd not available in gwtool"; }
}
