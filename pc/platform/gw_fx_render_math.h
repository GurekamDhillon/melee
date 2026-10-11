/* gw_fx_render_math.h - the Geno FX renderer's pure maths (R1..R6), header-only so the native test
 * (pc/tests/fx_render_math_test.cpp) and the renderer share one definition. Nothing here touches simulation state:
 * every input is the already-simulated particle list or a render-frame number, so a rollback resimulation that
 * reaches the same state draws the same picture (cosmetic only). */
#ifndef GW_FX_RENDER_MATH_H
#define GW_FX_RENDER_MATH_H
#include <math.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* R2: width multiplier and u coordinate of strip point i of n. cum = world distance from the head to point i, total
 * = the strip's length. taper = tail width / head width (1 = uniform). uv_per_unit 0: u runs 0..1 over the strip. */
static inline float fx_rib_width_mul(int i, int n, float taper) {
    const float t = n > 1 ? (float) i / (float) (n - 1) : 0.0f;
    return 1.0f + (taper - 1.0f) * t;
}
static inline float fx_rib_u(int i, int n, float cum, float uv_per_unit) {
    if (uv_per_unit > 0.0f) return cum * uv_per_unit;
    return n > 1 ? (float) i / (float) (n - 1) : 0.0f;
}
/* the camera-facing side vector of a strip segment A->B: perpendicular to the segment and to the view axis
 * (camz = the view's Z axis in world, as the billboards use). Returns 0 when the segment points along the view. */
static inline int fx_rib_side(const float a[3], const float b[3], const float camz[3], float out[3]) {
    const float d[3] = {b[0] - a[0], b[1] - a[1], b[2] - a[2]};
    float s[3] = {d[1] * camz[2] - d[2] * camz[1], d[2] * camz[0] - d[0] * camz[2], d[0] * camz[1] - d[1] * camz[0]};
    const float l = sqrtf(s[0] * s[0] + s[1] * s[1] + s[2] * s[2]);
    if (!(l > 1e-8f)) { out[0] = out[1] = out[2] = 0.0f; return 0; }
    out[0] = s[0] / l; out[1] = s[1] / l; out[2] = s[2] / l;
    return 1;
}

/* R3: soft-particle fade from two view-space distances (positive in front of the camera): 0 where the sprite
 * touches the scene, 1 at `distance` in front of it. The shader (fs soft_fade) computes the same expression. */
static inline float fx_soft_fade(float scene_dist, float frag_dist, float distance) {
    float f;
    if (!(distance > 0.0f)) return 1.0f;
    f = (scene_dist - frag_dist) / distance;
    return f < 0.0f ? 0.0f : f > 1.0f ? 1.0f : f;
}
/* view-space z (negative in front) of a depth-buffer value, from the projection's z and w rows (the same rows the
 * shader gets): zn = (r2.z*vz + r2.w) / (r3.z*vz + r3.w)  =>  vz = (r2.w - zn*r3.w) / (zn*r3.z - r2.z) */
static inline float fx_depth_to_viewz(float zn, const float r2[4], const float r3[4]) {
    const float den = zn * r3[2] - r2[2];
    return fabsf(den) > 1e-12f ? (r2[3] - zn * r3[3]) / den : 0.0f;
}

/* R4: the dissolve: alpha kept where the mask is above the threshold, a soft edge of width `edge` above it, and the
 * edge glow weight (1 at the threshold, 0 at the edge's far side). */
static inline float fx_dissolve_alpha(float mask, float threshold, float edge) {
    float f = edge > 1e-5f ? (mask - threshold) / edge : (mask >= threshold ? 1.0f : 0.0f);
    return f < 0.0f ? 0.0f : f > 1.0f ? 1.0f : f;
}
static inline float fx_dissolve_glow(float mask, float threshold, float edge) {
    const float a = fx_dissolve_alpha(mask, threshold, edge);
    return a > 0.0f && a < 1.0f ? 1.0f - a : 0.0f;
}

/* R6: camera shake. A shake started on game frame `start` lasting `frames` decays linearly; the offset is a pure
 * function of (frame - start), so resimulated frames draw the same offset. Output in projection (NDC) units. */
static inline uint32_t fx_hash32(uint32_t x) {
    x ^= x >> 16; x *= 0x7feb352dU; x ^= x >> 15; x *= 0x846ca68bU; x ^= x >> 16;
    return x;
}
static inline void fx_shake_offset(int frame, int start, int frames, float amp, uint32_t seed, float out[2]) {
    const int age = frame - start;
    float k;
    uint32_t h;
    out[0] = out[1] = 0.0f;
    if (frames <= 0 || age < 0 || age >= frames) return;
    k = amp * (1.0f - (float) age / (float) frames);
    h = fx_hash32(seed * 2654435761U + (uint32_t) age * 40503U + 1U);
    out[0] = k * ((float) (h & 0xFFFFU) / 32767.5f - 1.0f);
    out[1] = k * ((float) (h >> 16) / 32767.5f - 1.0f);
}
/* a flash: alpha at `frame` for a flash of peak alpha `a` over `frames` (linear fade out) */
static inline float fx_flash_alpha(int frame, int start, int frames, float a) {
    const int age = frame - start;
    if (frames <= 0 || age < 0 || age >= frames) return 0.0f;
    return a * (1.0f - (float) age / (float) frames);
}

#ifdef __cplusplus
}
#endif
#endif
