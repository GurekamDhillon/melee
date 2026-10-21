/* Driver for the ground.c oracle (gen_ground_ref.sh). Output, all hex:
 *   B <tx ty tz> | <ax ay az>        Ground_801C4B50 angles for tangent t
 *   E <a b c (9)> | <x y z>          Ground_801C5AEC
 *   V <val> <vol> <pan>              Ground_801C5630 / 5694 arguments
 * tests/ground_vs_c.rs replays them. */
#include <stdint.h>
#include <stdio.h>
#include <string.h>
typedef struct { float x, y, z; } Vec3;
typedef struct { uint8_t pad[0x20]; int32_t x20[8]; } Ground;
extern Vec3 ref_spline_point, ref_tangent;
extern int ref_last_id, ref_last_val;
void Ground_801C4B50(void* spline, Vec3* arg1, Vec3* result, float t);
void Ground_801C5630(Ground* gp, int i, float val);
void Ground_801C5694(Ground* gp, int i, float val);
void Ground_801C5AEC(Vec3* v, Vec3* a, Vec3* b, Vec3* c);

static uint32_t rng = 0x9E3779B9u;
static uint32_t nx(void) { rng ^= rng << 13; rng ^= rng >> 17; rng ^= rng << 5; return rng; }
static float rf(float lo, float hi) { return lo + (hi - lo) * (float) (nx() >> 8) / 16777216.0f; }
static uint32_t b(float f) { uint32_t u; memcpy(&u, &f, 4); return u; }
static float pick(float lo, float hi)
{
    uint32_t k = nx() % 16;
    if (k == 0) return 0.0f;
    if (k == 1) return -0.0f;
    if (k == 2) return 1.0f;
    if (k == 3) return -1.0f;
    return rf(lo, hi);
}
int main(void)
{
    int n;
    for (n = 0; n < 20000; n++) {
        Vec3 p, r;
        ref_tangent.x = pick(-3, 3);
        ref_tangent.y = pick(-3, 3);
        ref_tangent.z = pick(-3, 3);
        if (n % 7 == 0) ref_tangent.x = ref_tangent.z = rf(-1e-3f, 1e-3f);
        if (n % 11 == 0) ref_tangent.y = 0.0f;
        Ground_801C4B50(0, &p, &r, 0.5f);
        printf("B %08x %08x %08x | %08x %08x %08x\n", b(ref_tangent.x), b(ref_tangent.y),
               b(ref_tangent.z), b(r.x), b(r.y), b(r.z));
    }
    for (n = 0; n < 20000; n++) {
        Vec3 v[3], o;
        int k;
        float* f = (float*) v;
        for (k = 0; k < 9; k++) f[k] = pick(-2, 2);
        if (n % 5 == 0) v[1].z = (n & 8) ? 1.0f : -1.0f;
        if (n % 13 == 0) for (k = 0; k < 9; k++) f[k] = rf(-1e-30f, 1e-30f);
        Ground_801C5AEC(&o, &v[0], &v[1], &v[2]);
        printf("E");
        for (k = 0; k < 9; k++) printf(" %08x", b(f[k]));
        printf(" | %08x %08x %08x\n", b(o.x), b(o.y), b(o.z));
    }
    {
        Ground g;
        memset(&g, 0, sizeof g);
        g.x20[3] = 77;
        for (n = 0; n < 20000; n++) {
            float val = rf(-1.5f, 1.5f);
            int vol, pan;
            if (n < 64) val = -1.0f + n / 32.0f;
            Ground_801C5630(&g, 3, val);
            vol = ref_last_val;
            Ground_801C5694(&g, 3, val);
            pan = ref_last_val;
            printf("V %08x %08x %08x\n", b(val), (uint32_t) vol, (uint32_t) pan);
        }
    }
    return 0;
}
