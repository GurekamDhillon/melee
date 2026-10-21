/* Driver for the camera accessor oracle (gen_camera_acc_ref.sh). Each case randomises the
 * game_camera stand-in, prints `IN op iarg fin[17] state`, runs acc_op and prints
 * `OUT ret fout[9] state`. tests/camera_acc_vs_c.rs replays the IN lines. */
#include <stdio.h>
#include <string.h>
#include "camera_acc.h"

typedef struct { float x, y, z; } V3;
typedef struct { V3 interest, target_interest, position, target_position; float fov, target_fov; } TS;
typedef struct { unsigned char r, g, b, a; } COL;
typedef struct { float x, y; } V2;
typedef struct Camera { CAM_ACC_MEMBERS(V3, TS, COL, V2) } Camera;
extern Camera game_camera;
extern struct { float x[16]; float x40; } cm_803BCCA0;
extern struct { unsigned last_mode; } cm_80453004;

static unsigned rng = 0x2468ace1u;
static unsigned rnd(void) { rng ^= rng << 13; rng ^= rng >> 17; rng ^= rng << 5; return rng; }
static unsigned rn(unsigned n) { return rnd() % n; }
static float fr(float lo, float hi) { return lo + (hi - lo) * ((rnd() >> 8) * (1.0f / 16777216.0f)); }
static unsigned fb(float f) { unsigned u; memcpy(&u, &f, 4); return u; }
static float bf(unsigned u) { float f; memcpy(&f, &u, 4); return f; }
static void pf(float f) { printf(" %08x", fb(f)); }
static void pv(V3 v) { pf(v.x); pf(v.y); pf(v.z); }
static float anyf(void)
{
    switch (rn(8)) {
    case 0: return bf(rnd());
    case 1: return 0.0f;
    default: return fr(-1000.0f, 1000.0f);
    }
}
static V3 rv(void) { V3 v; v.x = anyf(); v.y = anyf(); v.z = anyf(); return v; }

static void dump(void)
{
    Camera* c = &game_camera;
    printf(" %u %u %u %u %u", c->mode, c->background_color.r, c->background_color.g,
           c->background_color.b, c->background_color.a);
    pf(c->nearz); pf(c->farz);
    pv(c->transform.interest); pv(c->transform.target_interest); pv(c->transform.position);
    pv(c->transform.target_position); pf(c->transform.fov); pf(c->transform.target_fov);
    pf(c->translation.x); pf(c->translation.y); pf(c->quake_scale); pf(c->x2B0);
    printf(" %d", c->x2B8); pf(c->x2BC); pv(c->pause_up);
    printf(" %u %u %u %u %u", c->x341_b0, c->x341_b1_b2, c->x341_b3_b4, c->x341_b5_b6, c->x341_b7);
    pv(c->x350); pv(c->x368); pf(c->x374);
    printf(" %u %u %u %u %u %u %u", c->x398_b0, c->x398_b1, c->x398_b2, c->x398_b3, c->x398_b4,
           c->x398_b5, c->x398_b6_b7);
    printf(" %u %u %u %u %u %u %u", c->x399_b0_b1, c->x399_b2, c->x399_b3, c->x399_b4, c->x399_b5,
           c->x399_b6, c->x399_b7);
    printf(" %u %u %u %u %u %u %u %u", c->x39A_b0, c->x39A_b1, c->x39A_b2, c->x39A_b3, c->x39A_b4,
           c->x39A_b5, c->x39A_b6, c->x39A_b7);
    pf(cm_803BCCA0.x40); printf(" %u", cm_80453004.last_mode);
}

static void randomise(void)
{
    Camera* c = &game_camera;
    memset(c, 0, sizeof *c);
    c->mode = rn(10);
    c->background_color.r = rnd(); c->background_color.g = rnd(); c->background_color.b = rnd();
    c->background_color.a = rnd();
    c->nearz = anyf(); c->farz = anyf();
    c->transform.interest = rv(); c->transform.target_interest = rv();
    c->transform.position = rv(); c->transform.target_position = rv();
    c->transform.fov = anyf(); c->transform.target_fov = anyf();
    c->translation.x = anyf(); c->translation.y = anyf();
    c->quake_scale = anyf(); c->x2B0 = anyf();
    c->x2B8 = rn(4) == 0 ? (short) rnd() : (short) ((int) rn(5) - 2);
    c->x2BC = anyf(); c->pause_up = rv();
    c->x341_b0 = rnd(); c->x341_b1_b2 = rnd(); c->x341_b3_b4 = rnd(); c->x341_b5_b6 = rnd(); c->x341_b7 = rnd();
    c->x350 = rv(); c->x368 = rv(); c->x374 = anyf();
    c->x398_b0 = rnd(); c->x398_b1 = rnd(); c->x398_b2 = rnd(); c->x398_b3 = rnd(); c->x398_b4 = rnd();
    c->x398_b5 = rnd(); c->x398_b6_b7 = rnd();
    c->x399_b0_b1 = rnd(); c->x399_b2 = rnd(); c->x399_b3 = rnd(); c->x399_b4 = rnd(); c->x399_b5 = rnd();
    c->x399_b6 = rnd(); c->x399_b7 = rnd();
    c->x39A_b0 = rnd(); c->x39A_b1 = rnd(); c->x39A_b2 = rnd(); c->x39A_b3 = rnd(); c->x39A_b4 = rnd();
    c->x39A_b5 = rnd(); c->x39A_b6 = rnd(); c->x39A_b7 = rnd();
    cm_803BCCA0.x40 = anyf(); cm_80453004.last_mode = rn(10);
}

static void one(int op)
{
    float fin[17], fout[9];
    int i, iarg, r;
    randomise();
    for (i = 0; i < 17; i++) fin[i] = anyf();
    iarg = (int) rnd();
    if (rn(2)) iarg = (int) rn(4);
    if (op == 42) {
        /* Camera_8002BC78: forward near the +-0.999 clamps half the time */
        if (rn(2)) {
            for (i = 0; i < 6; i++) fin[i] = fr(-1.0f, 1.0f);
            fin[1] = (rn(2) ? 1.0f : -1.0f) * fr(0.998f, 1.001f);
        } else {
            for (i = 0; i < 6; i++) fin[i] = fr(-1.0f, 1.0f);
        }
        if (rn(16) == 0) fin[1] = rn(2) ? 0.999f : -0.999f;
        if (rn(32) == 0) for (i = 0; i < 6; i++) fin[i] = anyf();
    }
    if (op == 45 || op == 47) {
        int nt = op == 45 ? 2 : 16;
        fin[nt] = rn(8) == 0 ? anyf() : fr(-0.25f, 1.25f);
        for (i = 0; i < nt; i++) if (rn(4)) fin[i] = fr(-500.0f, 500.0f);
    }
    if (op == 46) {
        fin[0] = rn(8) == 0 ? anyf() : fr(-0.25f, 1.25f);
        iarg = (int) rn(5);
        if (rn(16) == 0) fin[0] = 0.5f;
    }
    if (op == 43 && rn(2)) iarg = (int) fb(anyf());
    printf("IN %d %d", op, iarg);
    for (i = 0; i < 17; i++) pf(fin[i]);
    dump();
    printf("\n");
    for (i = 0; i < 9; i++) fout[i] = 0.0f;
    r = acc_op(op, iarg, fin, fout);
    printf("OUT %d", r);
    for (i = 0; i < 9; i++) pf(fout[i]);
    dump();
    printf("\n");
}

int main(void)
{
    int op, k;
    for (op = 0; op <= 47; op++) {
        int n = (op == 42 || op >= 45) ? 1500 : 60;
        for (k = 0; k < n; k++) one(op);
    }
    return 0;
}
