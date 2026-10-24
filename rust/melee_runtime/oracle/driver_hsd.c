/* Oracle driver for baselib mtx.c / quatlib.c / spline.c (see build_hsd.sh). Writes hex bit
 * patterns of generated inputs and the C outputs; tests/oracle_hsd.rs replays them. */
#include <stdio.h>
#include <string.h>
#include <stdint.h>
typedef float f32; typedef unsigned u32; typedef int s32; typedef short s16; typedef unsigned char u8;
typedef struct { f32 x, y, z; } Vec3;
typedef struct { f32 x, y, z, w; } Quaternion;
typedef f32 Mtx[3][4];
typedef struct { u8 type; s16 numcv; f32 tension; Vec3* cv; f32 totalLength; f32* segLength; f32 (*segPoly)[5]; } HSD_Spline;

void HSD_MtxInverse(Mtx, Mtx); void HSD_MtxInverseConcat(Mtx, Mtx, Mtx); void HSD_MtxInverseTranspose(Mtx, Mtx);
void HSD_MtxGetRotation(Mtx, Vec3*); void HSD_MtxGetTranslate(Mtx, Vec3*); void HSD_MtxGetScale(Mtx, Vec3*);
void HSD_MkRotationMtx(Mtx, Vec3*); void HSD_MtxQuat(Mtx, Quaternion*);
void HSD_MtxSRT(Mtx, Vec3*, Vec3*, Vec3*, Vec3*); void HSD_MtxSRTQuat(Mtx, Vec3*, Quaternion*, Vec3*, Vec3*);
void HSD_MtxScaledAdd(Mtx, Mtx, Mtx, f32);
s32 MatToQuat(Mtx, Quaternion*); s32 HSD_QuatLib_8037EB28(Mtx, Vec3*);
s32 HSD_QuatLib_8037EC4C(Quaternion*, Quaternion*, Quaternion*); s32 HSD_QuatLib_8037ECE0(Vec3*, Quaternion*, f32);
s32 EulerToQuat(Vec3*, Quaternion*); s32 HSD_QuatLib_8037EF28(Quaternion*, Quaternion*, Quaternion*, f32);
f32 splGetHelmite(f32, f32, f32, f32, f32, f32); void splGetSplinePoint(Vec3*, HSD_Spline*, f32);
f32 splArcLengthGetParameter(HSD_Spline*, f32); void splArcLengthPoint(Vec3*, HSD_Spline*, f32);

static uint64_t st = 0x2545F4914F6CDD1Dull;
static uint64_t nx(void) { st ^= st << 13; st ^= st >> 7; st ^= st << 17; return st; }
static u32 fb(f32 f) { u32 u; memcpy(&u, &f, 4); return u; }
static f32 rf(f32 r) { return (f32) ((double) (nx() >> 11) / 9007199254740992.0 * 2.0 - 1.0) * r; }
static FILE* o;
static void pf(f32 f) { fprintf(o, " %08x", fb(f)); }
static void pv(Vec3* v) { pf(v->x); pf(v->y); pf(v->z); }
static void pq(Quaternion* q) { pf(q->x); pf(q->y); pf(q->z); pf(q->w); }
static void pm(Mtx m) { int i; for (i = 0; i < 12; i++) pf((&m[0][0])[i]); }
static void rv(Vec3* v, f32 r) { v->x = rf(r); v->y = rf(r); v->z = rf(r); }
static void rq(Quaternion* q) { q->x = rf(1); q->y = rf(1); q->z = rf(1); q->w = rf(1); }
/* Random matrix: plain random, or a scaled rotation (realistic joint matrices), or singular. */
static void gm(Mtx m, int i)
{
    int k;
    switch (i % 4) {
    case 0: for (k = 0; k < 12; k++) (&m[0][0])[k] = rf(10); break;
    case 3: if (i % 8 == 3) { for (k = 0; k < 12; k++) (&m[0][0])[k] = (k % 4 == 0) ? rf(1) * 1e-4f : rf(1); m[1][0] = m[0][0]; m[1][1] = m[0][1]; m[1][2] = m[0][2]; break; }
    /* fallthrough */
    default: {
        Vec3 s, r, t;
        rv(&s, 3); rv(&r, 7); rv(&t, 100);
        if (i % 16 == 1) { r.y = 1.5707964f; }
        HSD_MtxSRT(m, &s, &r, &t, NULL);
    }
    }
}
int main(int argc, char** argv)
{
    int i, k;
    o = fopen(argv[1], "w");
    for (i = 0; i < 1500; i++) {
        Mtx a, b, c; Vec3 v, s, r, t, ps; Quaternion q, p, w; f32 f = rf(2);
        gm(a, i); gm(b, i / 4 + 1);
        rv(&s, 4); rv(&r, 8); rv(&t, 100); rv(&ps, 3); rq(&q); rq(&p);
        if (i % 10 == 0) { ps.x = 0; }
        if (i % 13 == 0) { p = q; }
        if (i % 17 == 0) { p.x = -q.x; p.y = -q.y; p.z = -q.z; p.w = -q.w; }
        fprintf(o, "hm"); pm(a); pm(b); pv(&s); pv(&r); pv(&t); pv(&ps); pq(&q); pq(&p); pf(f);
        memset(c, 0, sizeof c); HSD_MtxInverse(a, c); pm(c);
        memcpy(c, a, sizeof c); HSD_MtxInverse(c, c); pm(c);
        HSD_MtxInverseConcat(a, b, c); pm(c);
        memcpy(c, b, sizeof c); HSD_MtxInverseConcat(a, c, c); pm(c);
        memset(c, 0, sizeof c); HSD_MtxInverseTranspose(a, c); pm(c);
        HSD_MtxGetRotation(a, &v); pv(&v);
        HSD_MtxGetTranslate(a, &v); pv(&v);
        HSD_MtxGetScale(a, &v); pv(&v);
        HSD_MkRotationMtx(c, &r); pm(c);
        HSD_MtxQuat(c, &q); pm(c);
        HSD_MtxSRT(c, &s, &r, &t, NULL); pm(c);
        HSD_MtxSRT(c, &s, &r, &t, &ps); pm(c);
        HSD_MtxSRTQuat(c, &s, &q, &t, NULL); pm(c);
        HSD_MtxSRTQuat(c, &s, &q, &t, &ps); pm(c);
        HSD_MtxScaledAdd(a, b, c, f); pm(c);
        /* Native Rust returns these out-values directly: success is implicit.
         * Check the C status too, so the oracle covers both parts of the API. */
        if (MatToQuat(a, &w) != 0) return 2; pq(&w);
        if (HSD_QuatLib_8037EB28(a, &v) != 0) return 3; pv(&v);
        if (HSD_QuatLib_8037EC4C(&p, &q, &w) != 0) return 4; pq(&w);
        w.x = w.y = w.z = w.w = 7; v = s; if (i % 11 == 0) { v.x = v.y = v.z = 0; }
        k = HSD_QuatLib_8037ECE0(&v, &w, r.x); fprintf(o, " %d", k); pq(&w);
        if (EulerToQuat(&r, &w) != 0) return 5; pq(&w);
        if (HSD_QuatLib_8037EF28(&p, &q, &w, f * 0.5f + 0.5f) != 0) return 6; pq(&w);
        fprintf(o, "\n");
    }
    for (i = 0; i < 1500; i++) {
        Vec3 cv[28]; f32 seg[16]; f32 poly[16][5]; HSD_Spline sp; Vec3 v; f32 u, ft, tm, p0, p1, d0, d1, acc;
        int n = 2 + (int) (nx() % 7), j;
        for (j = 0; j < 28; j++) rv(&cv[j], 50);
        acc = 0; for (j = 0; j < n; j++) { seg[j] = acc; acc += 0.05f + rf(1) * rf(1); }
        for (j = 0; j < n; j++) seg[j] /= seg[n - 1];
        for (j = 0; j < 16; j++) { int q; for (q = 0; q < 5; q++) poly[j][q] = rf(20); poly[j][4] = rf(1) + 1.0f; }
        if (i % 9 == 0) poly[0][4] = -0.0004f;
        sp.type = (u8) (i % 5); sp.numcv = (s16) n; sp.tension = rf(1.5f); sp.cv = cv; sp.totalLength = 10 + rf(5);
        sp.segLength = seg; sp.segPoly = poly;
        u = (i % 23 == 0) ? 1.0f : (i % 29 == 0 ? 0.0f : rf(1.1f) * 0.5f + 0.5f);
        ft = rf(2); tm = rf(2); p0 = rf(30); p1 = rf(30); d0 = rf(30); d1 = rf(30);
        fprintf(o, "spl %d %d", sp.type, n); pf(sp.tension); pf(sp.totalLength);
        for (j = 0; j < 28; j++) pv(&cv[j]);
        for (j = 0; j < n; j++) pf(seg[j]);
        for (j = 0; j < 16 * 5; j++) pf((&poly[0][0])[j]);
        pf(u); pf(ft); pf(tm); pf(p0); pf(p1); pf(d0); pf(d1);
        pf(splGetHelmite(ft, tm, p0, p1, d0, d1));
        v.x = v.y = v.z = 99; splGetSplinePoint(&v, &sp, u); pv(&v);
        pf(splArcLengthGetParameter(&sp, u));
        v.x = v.y = v.z = 99; splArcLengthPoint(&v, &sp, u); pv(&v);
        fprintf(o, "\n");
    }
    /* Arc-length bisection only: one segment, many polynomials (discriminates the fused
     * Simpson accumulation, which flips a bisection comparison only rarely). */
    for (i = 0; i < 30000; i++) {
        f32 seg[2] = { 0, 1 }; f32 poly[1][5]; Vec3 cv[1]; HSD_Spline sp; int q; f32 u = rf(0.5f) + 0.5f;
        for (q = 0; q < 5; q++) poly[0][q] = rf(50);
        poly[0][4] = rf(10) + 10.5f;
        sp.type = 1 + (u8) (i % 3); sp.numcv = 2; sp.tension = 0; sp.cv = cv; sp.totalLength = rf(30) + 31;
        sp.segLength = seg; sp.segPoly = poly;
        fprintf(o, "arc %d", sp.type); pf(sp.totalLength); for (q = 0; q < 5; q++) pf(poly[0][q]); pf(u);
        pf(splArcLengthGetParameter(&sp, u)); fprintf(o, "\n");
    }
    fclose(o);
    return 0;
}
