#include <stdio.h>
#include <string.h>
#include <stdint.h>
typedef float f32; typedef unsigned u32; typedef int s32;
typedef struct { f32 x, y, z; } Vec;
typedef struct { f32 x, y, z, w; } Quaternion;
typedef f32 Mtx[3][4]; typedef f32 Mtx44[4][4];
double __frsqrte(double); double __fres(double);
float sinf(float); float cosf(float); float tanf(float);
float w_sqrtf(float); float w_sqrtf_accurate(float);
typedef Vec Vec3;
float lbVector_Normalize(Vec3*); float lbVector_NormalizeXY(Vec3*); Vec3* lbVector_CrossprodNormalized(Vec3*, Vec3*, Vec3*);
float lbVector_Angle(Vec3*, Vec3*); float lbVector_AngleXY(Vec3*, Vec3*); void lbVector_RotateAboutUnitAxis(Vec3*, Vec3*, float);
void lbVector_Rotate(Vec3*, int, float); void lbVector_Mirror(Vec3*, Vec3*); float lbVector_CosAngle(Vec3*, Vec3*);
Vec3* lbVector_Lerp(Vec3*, Vec3*, Vec3*, float); Vec3* lbVector_8000DE38(Mtx, Vec3*, float);
Vec3* lbVector_EulerAnglesFromONB(Vec3*, Vec3*, Vec3*, Vec3*); Vec3* lbVector_EulerAnglesFromPartialONB(Vec3*, Vec3*, Vec3*);
Vec3* lbVector_ApplyEulerRotation(Vec3*, Vec3*); void lbVector_CreateEulerMatrix(Mtx, Quaternion*);
float lbVector_8000E838(Vec3*, Vec3*, Vec3*, Vec3*);
float atanf(float); float atan2f(float, float); float acosf(float); float asinf(float);
void PSMTXIdentity(Mtx); void PSMTXConcat(Mtx, Mtx, Mtx); void PSMTXTranspose(Mtx, Mtx);
u32 PSMTXInverse(Mtx, Mtx); void PSMTXTrans(Mtx, f32, f32, f32); void PSMTXScale(Mtx, f32, f32, f32);
void PSMTXRotAxisRad(Mtx, Vec*, f32); void MTXRotRad(Mtx, char, f32); void PSMTXQuat(Mtx, Quaternion*);
void PSMTXMultVec(Mtx, Vec*, Vec*); void PSMTXMultVecSR(Mtx, Vec*, Vec*);
void PSVECAdd(Vec*, Vec*, Vec*); void PSVECSubtract(Vec*, Vec*, Vec*); void PSVECScale(Vec*, Vec*, f32);
void PSVECNormalize(Vec*, Vec*); f32 PSVECMag(Vec*); f32 PSVECDotProduct(Vec*, Vec*);
void PSVECCrossProduct(Vec*, Vec*, Vec*); void C_MTXLookAt(Mtx, Vec*, Vec*, Vec*);
void MTXLightFrustum(Mtx, f32, f32, f32, f32, f32, f32, f32, f32, f32);
void MTXLightPerspective(Mtx, f32, f32, f32, f32, f32, f32);
void MTXLightOrtho(Mtx, f32, f32, f32, f32, f32, f32, f32, f32);
void MTXFrustum(Mtx44, f32, f32, f32, f32, f32, f32); void MTXPerspective(Mtx44, f32, f32, f32, f32);
void MTXOrtho(Mtx44, f32, f32, f32, f32, f32, f32);
s32 HSD_Rand(void); f32 HSD_Randf(void); s32 HSD_Randi(s32); extern u32* HSD_RandSeedPtr;
void HSD_RandRenderBegin(void); void HSD_RandRenderEnd(void);

static uint64_t st = 0x9E3779B97F4A7C15ull;
static uint64_t nx(void) { st ^= st << 13; st ^= st >> 7; st ^= st << 17; return st; }
static u32 fb(f32 f) { u32 u; memcpy(&u, &f, 4); return u; }
static f32 bf(u32 u) { f32 f; memcpy(&f, &u, 4); return f; }
static uint64_t db(double d) { uint64_t u; memcpy(&u, &d, 8); return u; }
static f32 rf(f32 r) { return (f32) ((double) (nx() >> 11) / 9007199254740992.0 * 2.0 - 1.0) * r; }
static f32 anyf(void)
{
    switch (nx() % 4) {
    case 0: return rf(1);
    case 1: return rf(100);
    case 2: return rf(10000);
    default: { u32 u = (u32) nx(); u = (u & 0x807FFFFFu) | ((u32) (100 + nx() % 56) << 23); return bf(u); }
    }
}
static FILE* o;
static void pf(f32 f) { fprintf(o, " %08x", fb(f)); }
static void pm(f32* m, int n) { int i; for (i = 0; i < n; i++) pf(m[i]); }
static void rm(f32* m, int n) { int i; for (i = 0; i < n; i++) m[i] = anyf(); }
int main(int argc, char** argv)
{
    int i, k;
    o = fopen(argv[1], "w");
    for (i = 0; i < 1500; i++) {
        double d; uint64_t u = nx();
        if (i % 3 == 0) { f32 f = anyf(); if (f < 0) f = -f; d = f; } else memcpy(&d, &u, 8);
        fprintf(o, "frsqrte %016llx %016llx\n", (unsigned long long) db(d), (unsigned long long) db(__frsqrte(d)));
        fprintf(o, "fres %016llx %016llx\n", (unsigned long long) db(d), (unsigned long long) db(__fres(d)));
    }
    for (i = 0; i < 2000; i++) {
        f32 x = (i % 2) ? rf(10) : anyf();
        if (i < 40) x = bf((u32) i * 0x1000000u);
        fprintf(o, "trig"); pf(x); pf(sinf(x)); pf(cosf(x)); pf(tanf(x)); fprintf(o, "\n");
        fprintf(o, "sqrt"); pf(x); pf(w_sqrtf(x)); pf(w_sqrtf_accurate(x)); fprintf(o, "\n");
    }
    for (i = 0; i < 2000; i++) {
        f32 x = (i % 3) ? rf(3) : anyf(), y = (i % 3 == 1) ? rf(3) : anyf(), u = rf(1.2f);
        if (i < 16) { x = (i & 1) ? -0.0f : 0.0f; y = (i & 2) ? -1.0f : ((i & 4) ? 0.0f : 2.0f); if (i & 8) u = (i & 1) ? 1.0f : -1.0f; }
        fprintf(o, "atan"); pf(x); pf(y); pf(u); pf(atanf(x)); pf(atan2f(y, x)); pf(acosf(u)); pf(asinf(u)); fprintf(o, "\n");
    }
    for (i = 0; i < 1500; i++) {
        Vec a, b, c, r, ax; Mtx m; Quaternion q; f32 ang = (i % 2) ? rf(4) : rf(8), f = rf(1.5f); int axis = ""[i % 4];
        a.x = rf(50); a.y = rf(50); a.z = rf(50); b.x = rf(50); b.y = rf(50); b.z = rf(50); c.x = rf(50); c.y = rf(50); c.z = rf(50);
        if (i % 5 == 0) b.z = (i % 10) ? 1.0f : -1.0f;
        rm(&m[0][0], 12); ax = c; lbVector_Normalize(&ax); if (i % 9 == 0) { ax.x = 1; ax.y = 0; ax.z = 0; }
        fprintf(o, "lbv"); pf(a.x); pf(a.y); pf(a.z); pf(b.x); pf(b.y); pf(b.z); pf(c.x); pf(c.y); pf(c.z); pf(ang); pf(f); fprintf(o, " %d", axis); pm(&m[0][0], 12);
        r = a; pf(lbVector_Normalize(&r)); pf(r.x); pf(r.y); pf(r.z);
        r = a; pf(lbVector_NormalizeXY(&r)); pf(r.x); pf(r.y); pf(r.z);
        lbVector_CrossprodNormalized(&a, &b, &r); pf(r.x); pf(r.y); pf(r.z);
        pf(lbVector_Angle(&a, &b)); pf(lbVector_AngleXY(&a, &b)); pf(lbVector_CosAngle(&a, &b));
        r = a; lbVector_RotateAboutUnitAxis(&r, &ax, ang); pf(ax.x); pf(ax.y); pf(ax.z); pf(r.x); pf(r.y); pf(r.z);
        r = a; lbVector_Rotate(&r, axis, ang); pf(r.x); pf(r.y); pf(r.z);
        r = a; ax = b; lbVector_Normalize(&ax); lbVector_Mirror(&r, &ax); pf(r.x); pf(r.y); pf(r.z);
        lbVector_Lerp(&a, &b, &r, f); pf(r.x); pf(r.y); pf(r.z);
        lbVector_8000DE38(m, &r, f); pf(r.x); pf(r.y); pf(r.z);
        lbVector_EulerAnglesFromONB(&r, &a, &b, &c); pf(r.x); pf(r.y); pf(r.z);
        ax = a; lbVector_Normalize(&ax); r = c; lbVector_Normalize(&r); { Vec cc = r; lbVector_EulerAnglesFromPartialONB(&r, &ax, &cc); } pf(r.x); pf(r.y); pf(r.z);
        r = a; c.x = ang; c.y = f; c.z = -ang; lbVector_ApplyEulerRotation(&r, &c); pf(r.x); pf(r.y); pf(r.z);
        q.x = ang; q.y = f; q.z = -ang * 0.5f; q.w = 0; lbVector_CreateEulerMatrix(m, &q); pm(&m[0][0], 12);
        pf(lbVector_8000E838(&a, (i % 7 == 0) ? &a : &b, &c, &r)); pf(r.x); pf(r.y); pf(r.z);
        fprintf(o, "%c", 10);
    }
    for (i = 0; i < 400; i++) {
        Mtx a, b, c; Mtx44 p; Vec v, w, z, r; Quaternion q; f32 s;
        rm(&a[0][0], 12); rm(&b[0][0], 12);
        v.x = anyf(); v.y = anyf(); v.z = anyf(); w.x = rf(50); w.y = rf(50); w.z = rf(50); s = rf(7);
        fprintf(o, "m3"); pm(&a[0][0], 12); pm(&b[0][0], 12);
        pf(v.x); pf(v.y); pf(v.z); pf(w.x); pf(w.y); pf(w.z); pf(s);
        PSMTXConcat(a, b, c); pm(&c[0][0], 12);
        memcpy(c, a, sizeof c); PSMTXConcat(c, b, c); pm(&c[0][0], 12);
        PSMTXTranspose(a, c); pm(&c[0][0], 12);
        memset(c, 0, sizeof c); k = (int) PSMTXInverse(a, c); fprintf(o, " %d", k); pm(&c[0][0], 12);
        PSMTXMultVec(a, &v, &r); pf(r.x); pf(r.y); pf(r.z);
        PSMTXMultVecSR(a, &v, &r); pf(r.x); pf(r.y); pf(r.z);
        PSVECAdd(&v, &w, &r); pf(r.x); pf(r.y); pf(r.z);
        PSVECSubtract(&v, &w, &r); pf(r.x); pf(r.y); pf(r.z);
        PSVECScale(&v, &r, s); pf(r.x); pf(r.y); pf(r.z);
        PSVECNormalize(&w, &r); pf(r.x); pf(r.y); pf(r.z);
        pf(PSVECMag(&w)); pf(PSVECDotProduct(&v, &w));
        PSVECCrossProduct(&v, &w, &r); pf(r.x); pf(r.y); pf(r.z);
        PSMTXRotAxisRad(c, &w, s); pm(&c[0][0], 12);
        memset(c, 0, sizeof c); MTXRotRad(c, "xYzq"[i % 4], s); pm(&c[0][0], 12);
        q.x = w.x; q.y = w.y; q.z = w.z; q.w = rf(50); pf(q.w); PSMTXQuat(c, &q); pm(&c[0][0], 12);
        z.x = rf(50); z.y = rf(50); z.z = rf(50); pf(z.x); pf(z.y); pf(z.z);
        C_MTXLookAt(c, &w, &v, &z); pm(&c[0][0], 12);
        PSMTXTrans(c, v.x, v.y, s); pm(&c[0][0], 12);
        PSMTXScale(c, v.x, v.y, s); pm(&c[0][0], 12);
        {
            f32 t = rf(100), bb = rf(100), l = rf(100), rr = rf(100), n = rf(10) + 11, f = rf(1000) + 2000;
            f32 fov = rf(80) + 90, asp = rf(1) + 1.5f;
            pf(t); pf(bb); pf(l); pf(rr); pf(n); pf(f); pf(fov); pf(asp);
            MTXLightFrustum(c, t, bb, l, rr, n, s, v.x, v.y, w.x); pm(&c[0][0], 12);
            MTXLightPerspective(c, fov, asp, s, v.x, v.y, w.x); pm(&c[0][0], 12);
            MTXLightOrtho(c, t, bb, l, rr, s, v.x, v.y, w.x); pm(&c[0][0], 12);
            MTXFrustum(p, t, bb, l, rr, n, f); pm(&p[0][0], 16);
            MTXPerspective(p, fov, asp, n, f); pm(&p[0][0], 16);
            MTXOrtho(p, t, bb, l, rr, n, f); pm(&p[0][0], 16);
        }
        fprintf(o, "\n");
    }
    *HSD_RandSeedPtr = 0x12345678u;
    for (i = 0; i < 500; i++) {
        s32 a = HSD_Rand(); f32 f = HSD_Randf(); s32 b = HSD_Randi((i * 37) % 1000 - 100);
        fprintf(o, "rand %08x %08x %08x %08x\n", (u32) a, fb(f), (u32) b, *HSD_RandSeedPtr);
        if (i % 7 == 0) {
            HSD_RandRenderBegin(); HSD_Rand(); HSD_RandRenderBegin(); HSD_Randf(); HSD_RandRenderEnd();
            HSD_Randi(9); HSD_RandRenderEnd();
        }
    }
    fclose(o);
    return 0;
}
