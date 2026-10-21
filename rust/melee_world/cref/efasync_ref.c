/* C reference for rust/melee_world/src/tu/efasync.rs (tests/tu_efasync.rs).
 * Expressions cut from src/melee/ef/efasync.c (efAsync_Dispatch 0x3E8 / 0x427,
 * efAsync_SetEffectRandomRotationZ, efAsync_SetEffectFacingDir, the 0x417 tint,
 * efAsync_MexMiss's key). Build: bash rust/melee_world/cref/gen_efasync_ref.sh
 * (contraction on, gwtool fmuladd lowering via melee_runtime/oracle/lower.py). */
#include <stdio.h>
#include <stdint.h>
#include <string.h>
typedef float f32;
typedef int32_t s32;
typedef uint32_t u32;
#define M_PI 3.14159265358979323846
#define M_PI_2 (M_PI / 2)
#define M_TAU 6.283185307179586

static uint32_t st = 0x9E3779B9u;
static uint32_t rnd(void) { st ^= st << 13; st ^= st >> 17; st ^= st << 5; return st; }
static float rf(float lo, float hi) { return lo + (hi - lo) * ((rnd() >> 8) / 16777216.0f); }
static uint32_t fb(float f) { uint32_t u; memcpy(&u, &f, 4); return u; }

__attribute__((noinline)) static f32 scale_3e8(f32 x)
{
    f32 f32_1 = (0.04f * x) + 0.3f;
    if (f32_1 < 0.3f) {
        f32_1 = 0.3f;
    }
    if (f32_1 > 1.5f) {
        f32_1 = 1.5f;
    }
    return f32_1;
}
__attribute__((noinline)) static f32 fall_427(f32 r) { return (0.4f * r) + 2.8f; }
__attribute__((noinline)) static f32 tau(f32 r) { f32 f32_1 = (M_TAU * r); return f32_1; }
__attribute__((noinline)) static f32 facing(f32 d)
{
    double rotation;
    if (d < 0.0f) {
        rotation = -M_PI_2;
    } else {
        rotation = M_PI_2;
    }
    return rotation;
}
__attribute__((noinline)) static u32 tint(u32 u32_1)
{
    return ((u32) (u32_1 + 0xFFA00000) == 0x6060U) ? 0x808080 : 0xFFFFFF;
}
__attribute__((noinline)) static s32 key(int kind, s32 gfx_id)
{
    return (kind * 10000 + gfx_id % 100000) * 2 + (gfx_id >= 100000);
}

int main(void)
{
    int i;
    for (i = 0; i < 3000; i++) {
        f32 x = (i % 3 == 0) ? rf(-50.0f, 80.0f) : (i % 3 == 1) ? rf(0.0f, 1.0f) : rf(-1e6f, 1e6f);
        if (i < 4) {
            float sp[4] = {0.0f, -0.0f, 30.0f, -7.5f};
            x = sp[i];
        }
        printf("F %08x %08x %08x %08x %08x\n", fb(x), fb(scale_3e8(x)), fb(fall_427(x)), fb(tau(x)), fb(facing(x)));
    }
    for (i = 0; i < 1000; i++) {
        u32 c = (i & 1) ? (0x00606060u + (rnd() % 3) * 0x100000u) : rnd();
        int kind = (int) (rnd() % 140);
        s32 id = (i & 2) ? (s32) (5000 + rnd() % 4000) : (s32) (105000 + rnd() % 4000);
        printf("I %u %u %d %d %d\n", c, tint(c), kind, id, key(kind, id));
    }
    return 0;
}
