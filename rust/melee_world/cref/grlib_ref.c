/* Driver for the grlib.c oracle (gen_grlib_ref.sh). Output, all hex:
 *   P <top bottom right left posx posy px py offset> <result>
 * tests/grlib_vs_c.rs replays them. */
#include <stdint.h>
#include <stdio.h>
#include <string.h>
typedef struct { float x, y; } Vec2;
typedef struct { float x, y, z; } Vec3;
typedef struct { Vec2 top, bottom, right, left; } ftECB;
typedef struct { Vec3 cur_pos; ftECB ecb; } CollData;
int ref_point_inside(CollData* cd, Vec3* p, float offset);
static uint32_t rng = 0x2545F491u;
static uint32_t nx(void) { rng ^= rng << 13; rng ^= rng >> 17; rng ^= rng << 5; return rng; }
static float rf(float lo, float hi) { return lo + (hi - lo) * (float) (nx() >> 8) / 16777216.0f; }
static uint32_t b(float f) { uint32_t u; memcpy(&u, &f, 4); return u; }
static float bf(uint32_t u) { float f; memcpy(&f, &u, 4); return f; }
int main(void)
{
    int n;
    for (n = 0; n < 40000; n++) {
        CollData cd; Vec3 p; float off; int r;
        memset(&cd, 0, sizeof cd);
        cd.ecb.top.y = rf(0, 30); cd.ecb.bottom.y = rf(-5, 5);
        cd.ecb.right.x = rf(0, 10); cd.ecb.left.x = rf(-10, 0);
        cd.cur_pos.x = rf(-200, 200); cd.cur_pos.y = rf(-200, 200);
        off = (n % 5 == 0) ? 0.0f : rf(-5, 20);
        p.z = 0;
        p.x = cd.cur_pos.x + rf(-25, 25); p.y = cd.cur_pos.y + rf(-25, 45);
        if (n % 3 == 0) {
            /* sit p exactly on the predicted edge so fused vs unfused rounding decides */
            float h = cd.ecb.top.y - cd.ecb.bottom.y;
            float c = 0.5f * h + off;
            float y0 = 0.5f * (cd.ecb.top.y + cd.ecb.bottom.y) + cd.cur_pos.y;
            p.y = y0 - c;
            p.x = cd.cur_pos.x;
            if (n % 2) p.y = bf(b(p.y) + (nx() % 5) - 2);
        }
        if (n % 3 == 1) {
            float w = cd.ecb.right.x - cd.ecb.left.x;
            float c = 0.5f * w + off;
            float x0 = 0.5f * (cd.ecb.left.x + cd.ecb.right.x) + cd.cur_pos.x;
            p.y = cd.cur_pos.y + 0.5f * (cd.ecb.top.y + cd.ecb.bottom.y);
            p.x = x0 + c;
            if (n % 2) p.x = bf(b(p.x) + (nx() % 5) - 2);
        }
        if (n == 7) cd.cur_pos.y = bf(0x7FC00000u);
        r = ref_point_inside(&cd, &p, off);
        printf("P %08X %08X %08X %08X %08X %08X %08X %08X %08X %d\n", b(cd.ecb.top.y), b(cd.ecb.bottom.y),
               b(cd.ecb.right.x), b(cd.ecb.left.x), b(cd.cur_pos.x), b(cd.cur_pos.y), b(p.x), b(p.y), b(off), r);
    }
    return 0;
}
