// Standalone fixture for gw_fx_render_math.h (R2..R6 maths); no renderer/GPU dependencies.
//   clang++ -std=c++17 -o fx_render_math_test pc/tests/fx_render_math_test.cpp && ./fx_render_math_test
#include "../platform/gw_fx_render_math.h"
#include <cassert>
#include <cstdio>
int main() {
    // ribbon: taper endpoints, u, side vector
    assert(fx_rib_width_mul(0, 9, 0.0f) == 1.0f && fx_rib_width_mul(8, 9, 0.0f) == 0.0f);
    assert(fabsf(fx_rib_u(4, 9, 3.0f, 0.0f) - 0.5f) < 1e-6f && fabsf(fx_rib_u(4, 9, 3.0f, 2.0f) - 6.0f) < 1e-6f);
    const float a[3] = {0, 0, 0}, b[3] = {0, 2, 0}, cz[3] = {0, 0, 1};
    float s[3];
    assert(fx_rib_side(a, b, cz, s) == 1 && fabsf(fabsf(s[0]) - 1.0f) < 1e-6f && fabsf(s[1]) < 1e-6f);
    assert(fx_rib_side(a, cz, cz, s) == 0);
    // soft fade and depth inversion (both reversed-style and standard rows)
    assert(fx_soft_fade(3, 3, 1) == 0.0f && fx_soft_fade(4, 3, 1) == 1.0f && fabsf(fx_soft_fade(3.25f, 3, 1) - 0.25f) < 1e-6f);
    assert(fx_soft_fade(1, 3, 1) == 0.0f && fx_soft_fade(1, 3, 0) == 1.0f);
    const float r2[4] = {0, 0, 1.001f, 0.1001f}, r3[4] = {0, 0, -1, 0};
    for (float vz = -1.0f; vz > -2000.0f; vz *= 1.7f) {
        const float zn = (r2[2] * vz + r2[3]) / (r3[2] * vz + r3[3]);
        assert(fabsf(fx_depth_to_viewz(zn, r2, r3) - vz) < 1e-3f * fabsf(vz));
    }
    // dissolve
    assert(fx_dissolve_alpha(0.0f, 0.5f, 0.1f) == 0.0f && fx_dissolve_alpha(1.0f, 0.5f, 0.1f) == 1.0f);
    assert(fx_dissolve_alpha(0.5f, 0.5f, 0.0f) == 1.0f && fx_dissolve_alpha(0.49f, 0.5f, 0.0f) == 0.0f);
    assert(fx_dissolve_glow(0.55f, 0.5f, 0.1f) > 0.0f && fx_dissolve_glow(0.7f, 0.5f, 0.1f) == 0.0f);
    // shake: deterministic, bounded, decays to zero; flash fades
    for (int f = 50; f < 70; ++f) {
        float v[2], w[2];
        fx_shake_offset(f, 50, 12, 0.04f, 99, v);
        fx_shake_offset(f, 50, 12, 0.04f, 99, w);
        assert(v[0] == w[0] && v[1] == w[1]);
        const float cap = 0.04f * (1.0f - (float) (f - 50) / 12.0f) + 1e-6f;
        assert(f - 50 < 12 ? (fabsf(v[0]) <= cap && fabsf(v[1]) <= cap) : (v[0] == 0.0f && v[1] == 0.0f));
    }
    assert(fx_flash_alpha(50, 50, 4, 1.0f) == 1.0f && fx_flash_alpha(52, 50, 4, 1.0f) == 0.5f && fx_flash_alpha(54, 50, 4, 1.0f) == 0.0f);
    std::puts("fx_render_math_test: ok");
    return 0;
}
