/* C reference for rust/melee_runtime/src/tu/db*.rs (tests/tu_db*.rs).
 * Formatting lines cut from src/melee/db/dbcpu.c / dbcamera.c; float math from dbcamera.c.
 * Build: bash rust/melee_runtime/oracle/gen_tu_db_ref.sh  (contraction on, gwtool lowering). */
#include <stdio.h>
#include <stdint.h>
#include <string.h>


typedef float f32;
#define ABS(x) ((x) < 0 ? -(x) : (x))
float tanf(float); /* pc/gameworld/gekko_fp.c */
#define MTXDegToRad(a) ((a) * 0.01745329252f)

static uint32_t st = 0x12345678u;
static uint32_t rnd(void) { st ^= st << 13; st ^= st >> 17; st ^= st << 5; return st; }
static float rf(float lo, float hi) { return lo + (hi - lo) * ((rnd() >> 8) / 16777216.0f); }
static uint32_t fb(float f) { uint32_t u; memcpy(&u, &f, 4); return u; }
static float bf(uint32_t u) { float f; memcpy(&f, &u, 4); return f; }

/* dbcamera.c static inline cstick_threshold */
static inline float cstick_threshold(float cstick, float val)
{
    if (ABS(cstick) < val) {
        return 0;
    }
    return cstick;
}
/* dbcamera.c fn_80227188: per-component clamp then (int) */
static int eye_int(float v) { int r; if (ABS(v) > 99999.0F) { r = -1; } else { r = v; } return r; }
/* dbcamera.c fn_80227FE0 / fn_80227EB0 scale factor (MTXDegToRad form) */
static float pan_scale(float eye_dist, float fov)
{
    return 0.03F * (2.0F * (eye_dist * tanf(MTXDegToRad(fov) / 2)));
}
/* dbcamera.c fn_80228124 scale factor (literal 0.017453292f form) */
static float pan_scale_follow(float dist, float fov)
{
    return 0.03f * (2.0f * (dist * tanf(0.017453292f * fov / 2)));
}
/* dbcamera.c fn_80227D38 dolly factor */
static float dolly_follow(float dist, float arg2) { return dist * -(0.05F * arg2 - 1.0F); }
/* dbcamera.c fn_80227CAC dolly factor */
static float dolly_free(float dist, float y) { return dist * (1.0F - 0.05F * y); }

int main(void)
{
    int i;
    for (i = 0; i < 400; i++) {
        int state = (int) (rnd() % 4), lvl = (int) (rnd() % 10), type = (int) (rnd() % 30),
            hc = (int) (rnd() % 10);
        float a = rf(-5, 120), b = rf(0, 3), c = rf(-1, 10);
        if (i % 7 == 0) { a = bf(rnd()); }
        if (((fb(a) >> 23) & 0xFF) == 0xFF) { a = 1.005f; }
        printf("CPU %d %d %d %d %08x %08x %08x|", state, lvl, type, hc, fb(a), fb(b), fb(c));
        printf("%d %d %2d %d %2.2f %2.2f %2.2f\n", state, lvl, type, hc, a, b, c);
    }
    for (i = 0; i < 4000; i++) {
        float x = rf(-1.2f, 1.2f), t = rf(0, 1), d = rf(1, 2000), fov = rf(1, 120);
        float big = (i % 3 == 0) ? rf(-200000, 200000) : rf(-100000, 100000);
        if (i == 0) { x = 0.2f; } if (i == 1) { x = -0.2f; } if (i == 2) { big = 99999.0f; }
        if (i == 3) { big = -99999.01f; }
        printf("CAM %08x %08x %08x %08x %08x %08x %d %08x %08x %08x %08x\n", fb(x), fb(t),
               fb(d), fb(fov), fb(big), fb(cstick_threshold(x, t)), eye_int(big),
               fb(pan_scale(d, fov)), fb(pan_scale_follow(d, fov)), fb(dolly_follow(d, x)),
               fb(dolly_free(d, x)));
    }
    return 0;
}
