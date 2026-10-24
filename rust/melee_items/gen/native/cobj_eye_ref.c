/* Reference for tests/native_cobj_eye.rs: HSD_CObjGetEyeDistance of src/sysdolphin/baselib/cobj.c, body verbatim
 * (VECSubtract -> PSVECSubtract, VECMag -> PSVECMag from pc/gameworld/mtx_pc.c, MSL sqrtf from math_ppc.h), with the
 * wobj positions read directly (wobj flags 0: HSD_WObjGetPosition copies pos). Built by build_cobj_eye.sh with the
 * shipping contraction + gwtool lowering. One line per case: distance bits (NaN canonical). */
#include <stdio.h>
#include <string.h>
typedef struct { float x, y, z; } Vec;
extern void PSVECSubtract(Vec* a, Vec* b, Vec* c);
extern float PSVECMag(Vec* v);
#define VECSubtract PSVECSubtract
#define VECMag PSVECMag
typedef struct { Vec pos; } W;
typedef struct { W* eyepos; W* interest; } C;
static void HSD_CObjGetEyePosition(C* c, Vec* p) { *p = c->eyepos->pos; }
static void HSD_CObjGetInterest(C* c, Vec* p) { *p = c->interest->pos; }
float HSD_CObjGetEyeDistance(C* cobj)
{
    Vec position;
    Vec interest;
    Vec look_vector;

    if (cobj == NULL) {
        return 0.0f;
    }
    HSD_CObjGetEyePosition(cobj, &position);
    HSD_CObjGetInterest(cobj, &interest);
    VECSubtract(&interest, &position, &look_vector);
    return VECMag(&look_vector);
}
static unsigned st;
static unsigned rnd(void) { st = st * 214013u + 2531011u; return st >> 8; }
static float rf(void)
{
    unsigned k = rnd() % 16;
    if (k == 0) return 0.0f;
    if (k == 1) return (float) (int) (rnd() % 7) - 3.0f;
    if (k == 2) { unsigned u = rnd() * 2654435761u; float f; memcpy(&f, &u, 4); return f; } /* any bit pattern */
    if (k == 3) return (float) (int) (rnd() % 2000001 - 1000000) * 1e5f;
    return (float) (int) (rnd() % 400001 - 200000) / 64.0f;
}
int main(void)
{
    int i;
    st = 4242;
    for (i = 0; i < 20000; i++) {
        W e = { { rf(), rf(), rf() } }, t = { { rf(), rf(), rf() } };
        C c = { &e, &t };
        float d = HSD_CObjGetEyeDistance(&c);
        unsigned u;
        memcpy(&u, &d, 4);
        if (d != d) u = 0x7FC00000u;
        printf("%08x %08x %08x %08x %08x %08x %08x\n", *(unsigned*) &e.pos.x, *(unsigned*) &e.pos.y, *(unsigned*) &e.pos.z,
               *(unsigned*) &t.pos.x, *(unsigned*) &t.pos.y, *(unsigned*) &t.pos.z, u);
    }
    return 0;
}
