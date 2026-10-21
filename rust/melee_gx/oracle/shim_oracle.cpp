// Shim oracle driver (see shim_build.sh). Prints one line per case: the inputs' seed index and the
// result bits; tests/shim_oracle.rs regenerates the same inputs and compares.
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstring>

typedef float f32;
typedef uint8_t u8;
typedef uint16_t u16;
typedef uint32_t u32;
typedef int32_t s32;
typedef u8 GXBool;
#define M_PIF 3.14159265358979323846f

enum GXSpotFn { GX_SP_OFF, GX_SP_FLAT, GX_SP_COS, GX_SP_COS2, GX_SP_SHARP, GX_SP_RING1, GX_SP_RING2 };
enum GXDistAttnFn { GX_DA_OFF, GX_DA_GENTLE, GX_DA_MEDIUM, GX_DA_STEEP };
enum {
  GX_TF_I4 = 0x0, GX_TF_I8 = 0x1, GX_TF_IA4 = 0x2, GX_TF_IA8 = 0x3, GX_TF_RGB565 = 0x4, GX_TF_RGB5A3 = 0x5,
  GX_TF_RGBA8 = 0x6, GX_TF_C4 = 0x8, GX_TF_C8 = 0x9, GX_TF_C14X2 = 0xa, GX_TF_CMPR = 0xe, GX_TF_Z8 = 0x11,
  GX_TF_Z16 = 0x13, GX_TF_Z24X8 = 0x16, GX_CTF_R4 = 0x20, GX_CTF_RA4 = 0x22, GX_CTF_RA8 = 0x23, GX_CTF_A8 = 0x27,
  GX_CTF_R8 = 0x28, GX_CTF_G8 = 0x29, GX_CTF_B8 = 0x2a, GX_CTF_RG8 = 0x2b, GX_CTF_GB8 = 0x2c, GX_CTF_Z4 = 0x30,
  GX_CTF_Z8M = 0x39, GX_CTF_Z8L = 0x3a, GX_CTF_Z16L = 0x3c,
};
struct GXLightObj_ {
  float a0, a1, a2, k0, k1, k2;
};
typedef GXLightObj_ GXLightObj;

#include "aurora_cut.inc"

extern "C" {
void gw_xplat_GXProject(f32 x, f32 y, f32 z, const f32 mtx[3][4], const f32 *pm, const f32 *vp, f32 *sx, f32 *sy,
                        f32 *sz);
void gw_GXInitFogAdjTable(void *table, u16 width, const void *projmtx);
}

static uint32_t rng = 0x2545F491u;
static uint32_t next() {
  rng ^= rng << 13;
  rng ^= rng >> 17;
  rng ^= rng << 5;
  return rng;
}
// A float in [-lim, lim] with 1/100 steps: one int->float conversion and one division, so the
// Rust side builds the identical value.
static f32 fr(int lim) { return (f32)((int)(next() % (uint32_t)(2 * lim * 100 + 1)) - lim * 100) / 100.0f; }
static uint32_t bits(f32 f) {
  uint32_t u;
  memcpy(&u, &f, 4);
  return u;
}

int main() {
  // GXProject: 4000 cases, half perspective, half orthographic.
  for (int i = 0; i < 4000; ++i) {
    f32 m[3][4];
    for (int r = 0; r < 3; ++r)
      for (int c = 0; c < 4; ++c) m[r][c] = c == 3 ? fr(300) : fr(2);
    f32 pm[7];
    pm[0] = (i & 1) ? 1.0f : 0.0f;
    for (int k = 1; k < 7; ++k) pm[k] = fr(4);
    f32 vp[6] = {fr(50), fr(50), 640.0f + fr(100), 480.0f + fr(100), 0.0f, 1.0f};
    vp[4] = (f32)(next() % 50) / 100.0f;
    f32 x = fr(200), y = fr(200), z = fr(400);
    f32 sx, sy, sz;
    gw_xplat_GXProject(x, y, z, m, pm, vp, &sx, &sy, &sz);
    printf("project %d %08x %08x %08x\n", i, bits(sx), bits(sy), bits(sz));
  }
  // GXInitFogAdjTable: 1000 cases (perspective when m[3][3] == 0).
  for (int i = 0; i < 1000; ++i) {
    f32 m[4][4];
    for (int r = 0; r < 4; ++r)
      for (int c = 0; c < 4; ++c) m[r][c] = fr(3);
    m[3][3] = (i & 1) ? 1.0f : 0.0f;
    if (m[0][0] == 0.0f) m[0][0] = 1.0f;
    unsigned char be[64];
    for (int k = 0; k < 16; ++k) {
      uint32_t w = bits(m[k / 4][k % 4]);
      be[4 * k] = (unsigned char)(w >> 24);
      be[4 * k + 1] = (unsigned char)(w >> 16);
      be[4 * k + 2] = (unsigned char)(w >> 8);
      be[4 * k + 3] = (unsigned char)w;
    }
    u16 width = (u16)(160 + next() % 1000);
    unsigned char table[20];
    gw_GXInitFogAdjTable(table, width, be);
    printf("fog %d", i);
    for (int k = 0; k < 10; ++k) printf(" %03x", (table[2 * k] << 8) | table[2 * k + 1]);
    printf("\n");
  }
  // GXGetTexBufferSize: every format x sizes x mips.
  static const u32 fmts[] = {0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x8, 0x9, 0xa, 0xe, 0x11, 0x13, 0x16, 0x20, 0x22,
                             0x23, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x30, 0x39, 0x3a, 0x3c, 0x41};
  for (int i = 0; i < 2000; ++i) {
    u32 f = fmts[next() % (sizeof fmts / sizeof fmts[0])];
    u16 w = (u16)(1 + next() % 1024), h = (u16)(1 + next() % 1024);
    GXBool mips = (GXBool)(next() & 1);
    u8 lod = (u8)(next() % 12);
    printf("texsize %d %u\n", i, GXGetTexBufferSize(w, h, f, mips, lod));
  }
  // GXInitLightSpot / GXInitLightDistAttn.
  for (int i = 0; i < 2000; ++i) {
    GXLightObj o = {1, 0, 0, 1, 0, 0};
    f32 cutoff = fr(100);
    GXInitLightSpot(&o, cutoff, (GXSpotFn)(next() % 7));
    f32 dist = fr(500), br = fr(1);
    GXInitLightDistAttn(&o, dist, br, (GXDistAttnFn)(next() % 4));
    printf("light %d %08x %08x %08x %08x %08x %08x\n", i, bits(o.a0), bits(o.a1), bits(o.a2), bits(o.k0), bits(o.k1),
           bits(o.k2));
  }
  return 0;
}
