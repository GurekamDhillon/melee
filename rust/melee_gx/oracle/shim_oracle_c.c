/* C half of the shim oracle: the two shim_gx.c functions, cut verbatim into shim_cut.inc by
 * shim_build.sh, with the gw.h helpers they use (game memory is big-endian in the C build). */
#include <math.h>
#include <stdint.h>
#include <string.h>

typedef float f32;
typedef uint8_t u8;
typedef uint16_t u16;
typedef uint32_t u32;

static uint32_t be32(const unsigned char *p) {
  return ((uint32_t)p[0] << 24) | ((uint32_t)p[1] << 16) | ((uint32_t)p[2] << 8) | p[3];
}
static void gw_read_mtx44(f32 dst[4][4], const void *src) {
  const unsigned char *s = (const unsigned char *)src;
  for (int i = 0; i < 16; ++i) {
    uint32_t w = be32(s + 4 * i);
    memcpy(&dst[i / 4][i % 4], &w, 4);
  }
}
static void gw_w16(void *p, uint16_t v) {
  unsigned char *d = (unsigned char *)p;
  d[0] = (unsigned char)(v >> 8);
  d[1] = (unsigned char)v;
}

#include "shim_cut.inc"
