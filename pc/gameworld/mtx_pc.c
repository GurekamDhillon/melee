/*
 * Plain-C stand-ins for the Dolphin SDK matrix/vector routines that ship as
 * PowerPC paired-single assembly (extern/dolphin/src/dolphin/mtx/{mtx,mtxvec,
 * psmtx,vec}.c) and therefore cannot be compiled for the PC port.
 *
 * This translation unit is game-world code: it is compiled for ppc32 and run
 * through gwtool exactly like the rest of melee, so memory is handled for us
 * and nothing here needs byte-swapping.
 *
 * The "PS" prefix only ever referred to the instructions used, so each of
 * these is the logic of the corresponding C_MTX / C_VEC reference routine,
 * transcribed with its original operand grouping preserved (float addition is
 * not associative; reordering would change results).
 */

#include <dolphin/mtx.h>

#include <math.h>

/* A paired-single ps_madd lane: a*c + b with ONE rounding to single. Both factors are floats, so
 * their product is exact in double; the sum is rounded to double and then to float - Dolphin's
 * model of the Gekko's fused multiply-add (the same as gwtool's lowering of fmuladd).
 *
 * The product and the add are separate statements on purpose. As one expression clang contracts
 * them to llvm.fmuladd.f64, gwtool lowers that to llvm.fma.f64, and without FMA3 that is a call
 * to the CRT's software fma - 2.6 ms a frame for Sora's 2,080 envelopes. Since the float*float
 * product is exact in double (48 significant bits, no overflow or underflow possible), mul then
 * add rounds once, exactly like fma(a, c, b): the same bits, no call. geno_tests checks it. */
static float mtx_madd(float a, float c, float b)
{
    double p = (double) a * (double) c;
    return (float) (p + (double) b);
}

void PSMTXIdentity(Mtx m)
{
    m[0][0] = 1.0f;
    m[0][1] = 0.0f;
    m[0][2] = 0.0f;
    m[0][3] = 0.0f;
    m[1][0] = 0.0f;
    m[1][1] = 1.0f;
    m[1][2] = 0.0f;
    m[1][3] = 0.0f;
    m[2][0] = 0.0f;
    m[2][1] = 0.0f;
    m[2][2] = 1.0f;
    m[2][3] = 0.0f;
}

void PSMTXCopy(Mtx src, Mtx dst)
{
    if (src != dst) {
        dst[0][0] = src[0][0];
        dst[0][1] = src[0][1];
        dst[0][2] = src[0][2];
        dst[0][3] = src[0][3];
        dst[1][0] = src[1][0];
        dst[1][1] = src[1][1];
        dst[1][2] = src[1][2];
        dst[1][3] = src[1][3];
        dst[2][0] = src[2][0];
        dst[2][1] = src[2][1];
        dst[2][2] = src[2][2];
        dst[2][3] = src[2][3];
    }
}

void PSMTXConcat(Mtx mA, Mtx mB, Mtx mAB)
{
    Mtx mTmp;
    MtxPtr m;

    /* The destination is routinely also one of the sources. */
    if (mAB == mA || mAB == mB) {
        m = mTmp;
    } else {
        m = mAB;
    }

    /* The retail paired-single code's dataflow (mtx.c PSMTXConcat), not the C_MTX grouping:
     * per element, ps_muls0 then two fused ps_madds, t = fma(b2j, ai2, fma(b1j, ai1, b0j ai0));
     * then the Unit01 lane: column 3 adds ai3 (1 * ai3, fused - exact product), column 2 adds
     * 0 * ai3. One statement per rounding, so clang contracts nothing. */
    {
        int i, j;
        for (i = 0; i < 3; i++) {
            for (j = 0; j < 4; j++) {
                float t = mB[0][j] * mA[i][0];
                t = mtx_madd(mB[1][j], mA[i][1], t);
                t = mtx_madd(mB[2][j], mA[i][2], t);
                if (j == 3) {
                    t = mtx_madd(1.0F, mA[i][3], t);
                } else if (j == 2) {
                    t = mtx_madd(0.0F, mA[i][3], t);
                }
                m[i][j] = t;
            }
        }
    }

    if (m == mTmp) {
        PSMTXCopy(mTmp, mAB);
    }
}

void PSMTXTranspose(Mtx src, Mtx xPose)
{
    Mtx mTmp;
    MtxPtr m;

    if (src == xPose) {
        m = mTmp;
    } else {
        m = xPose;
    }

    /* Only the 3x3 part is transposed; the translation column is cleared. */
    m[0][0] = src[0][0];
    m[0][1] = src[1][0];
    m[0][2] = src[2][0];
    m[0][3] = 0.0f;
    m[1][0] = src[0][1];
    m[1][1] = src[1][1];
    m[1][2] = src[2][1];
    m[1][3] = 0.0f;
    m[2][0] = src[0][2];
    m[2][1] = src[1][2];
    m[2][2] = src[2][2];
    m[2][3] = 0.0f;

    if (m == mTmp) {
        PSMTXCopy(mTmp, xPose);
    }
}

/* PSMTXInverse follows the retail paired-single code (libs/dolphin/src/dolphin/mtx/mtx.c), not
 * C_MTXInverse: each cofactor is ps_mul then ps_msub (a*c - b, one rounding), the determinant
 * ps_mul + two ps_madd, 1/det is the fres estimate refined once (2r - det*(r*r), ps_nmsub), and the
 * translation column is ps_mul + ps_madd + ps_nmadd. The C reference's exact divide and grouping
 * differ in the last bits; the camera's inverse view matrix (a star-KO fighter's position,
 * ftDrawCommon_80080E18) goes through here (.slp parity, notes/fma.md). One statement per rounding,
 * so clang contracts nothing. */
extern double __fres(double val);
static float mtx_msub(float a, float c, float b)
{
    double p = (double) a * (double) c;
    return (float) (p - (double) b);
}
u32 PSMTXInverse(Mtx src, Mtx inv)
{
    f32 c00, c10, c01, c11, c02, c12, c20, c21, c22, p, det, r, r2, rr, i, t;
    int k;

    p = src[2][1] * src[1][2];
    c00 = mtx_msub(src[1][1], src[2][2], p);
    p = src[2][2] * src[1][0];
    c10 = mtx_msub(src[1][2], src[2][0], p);
    p = src[0][1] * src[2][2];
    c01 = mtx_msub(src[2][1], src[0][2], p);
    p = src[0][2] * src[2][0];
    c11 = mtx_msub(src[2][2], src[0][0], p);
    p = src[1][1] * src[0][2];
    c02 = mtx_msub(src[0][1], src[1][2], p);
    p = src[1][2] * src[0][0];
    c12 = mtx_msub(src[0][2], src[1][0], p);
    p = src[1][1] * src[2][0];
    c20 = mtx_msub(src[1][0], src[2][1], p);
    p = src[0][0] * src[2][1];
    c21 = mtx_msub(src[0][1], src[2][0], p);
    p = src[0][1] * src[1][0];
    c22 = mtx_msub(src[0][0], src[1][1], p);
    det = src[0][0] * c00;
    det = mtx_madd(src[1][0], c01, det);
    det = mtx_madd(src[2][0], c02, det);

    /* Singular: leave the destination untouched and report failure, as the original does. */
    if (det == 0.0f) {
        return 0;
    }

    r = (f32) __fres((double) det);
    r2 = r + r;
    rr = r * r;
    i = mtx_msub(det, rr, r2);
    i = -i;

    /* src is read only above and for the translation column below, so inv may alias it. */
    {
        f32 s03 = src[0][3], s13 = src[1][3], s23 = src[2][3];
        f32 m[3][3];
        m[0][0] = c00 * i;
        m[1][0] = c10 * i;
        m[0][1] = c01 * i;
        m[1][1] = c11 * i;
        m[0][2] = c02 * i;
        m[1][2] = c12 * i;
        m[2][0] = c20 * i;
        m[2][1] = c21 * i;
        m[2][2] = c22 * i;
        for (k = 0; k < 3; k++) {
            t = m[k][0] * s03;
            t = mtx_madd(m[k][1], s13, t);
            t = mtx_madd(m[k][2], s23, t);
            inv[k][0] = m[k][0];
            inv[k][1] = m[k][1];
            inv[k][2] = m[k][2];
            inv[k][3] = -t;
        }
    }
    return 1;
}

void PSMTXTrans(Mtx m, f32 xT, f32 yT, f32 zT)
{
    m[0][0] = 1.0f;
    m[0][1] = 0.0f;
    m[0][2] = 0.0f;
    m[0][3] = xT;
    m[1][0] = 0.0f;
    m[1][1] = 1.0f;
    m[1][2] = 0.0f;
    m[1][3] = yT;
    m[2][0] = 0.0f;
    m[2][1] = 0.0f;
    m[2][2] = 1.0f;
    m[2][3] = zT;
}

void PSMTXScale(Mtx m, f32 xS, f32 yS, f32 zS)
{
    m[0][0] = xS;
    m[0][1] = 0.0f;
    m[0][2] = 0.0f;
    m[0][3] = 0.0f;
    m[1][0] = 0.0f;
    m[1][1] = yS;
    m[1][2] = 0.0f;
    m[1][3] = 0.0f;
    m[2][0] = 0.0f;
    m[2][1] = 0.0f;
    m[2][2] = zS;
    m[2][3] = 0.0f;
}

void PSMTXRotAxisRad(Mtx m, Vec* axis, f32 rad)
{
    Vec vN;
    f32 s;
    f32 c;
    f32 t;
    f32 x;
    f32 y;
    f32 z;

    s = sinf(rad);
    c = cosf(rad);
    t = 1.0f - c;

    PSVECNormalize(axis, &vN);
    x = vN.x;
    y = vN.y;
    z = vN.z;

    /* The retail paired-single code (80342530), not C_MTXRotAxisRad: xt, yt, zt = x*t, y*t, z*t
     * (ps_muls0), every product rounded on its own and summed by ps_sum0/ps_sum1; only m01 and
     * m10 fuse z*s with xt*y (fnmsubs, fmadds). One statement per rounding. */
    {
        f32 xt = x * t, yt = y * t, zt = z * t;
        f32 xs = x * s, ys = y * s;
        f32 xty = xt * y, xtz = xt * z, ytz = yt * z, p;
        p = xt * x;
        m[0][0] = p + c;
        p = mtx_msub(z, s, xty);
        m[0][1] = -p;
        m[0][2] = xtz + ys;
        m[0][3] = 0.0f;
        m[1][0] = mtx_madd(z, s, xty);
        p = yt * y;
        m[1][1] = c + p;
        m[1][2] = ytz - xs;
        m[1][3] = 0.0f;
        m[2][0] = xtz - ys;
        m[2][1] = xs + ytz;
        p = zt * z;
        m[2][2] = p + c;
        m[2][3] = 0.0f;
    }
}

/* Body of MTXRotTrig.  Kept static: PSMTXRotTrig is not part of the shim
 * surface, so introducing it here would only add an unresolved reference. */
static void mtx_rot_trig(Mtx m, char axis, f32 sinA, f32 cosA)
{
    switch (axis) {
    case 'x':
    case 'X':
        m[0][0] = 1.0f;
        m[0][1] = 0.0f;
        m[0][2] = 0.0f;
        m[0][3] = 0.0f;
        m[1][0] = 0.0f;
        m[1][1] = cosA;
        m[1][2] = -sinA;
        m[1][3] = 0.0f;
        m[2][0] = 0.0f;
        m[2][1] = sinA;
        m[2][2] = cosA;
        m[2][3] = 0.0f;
        break;
    case 'y':
    case 'Y':
        m[0][0] = cosA;
        m[0][1] = 0.0f;
        m[0][2] = sinA;
        m[0][3] = 0.0f;
        m[1][0] = 0.0f;
        m[1][1] = 1.0f;
        m[1][2] = 0.0f;
        m[1][3] = 0.0f;
        m[2][0] = -sinA;
        m[2][1] = 0.0f;
        m[2][2] = cosA;
        m[2][3] = 0.0f;
        break;
    case 'z':
    case 'Z':
        m[0][0] = cosA;
        m[0][1] = -sinA;
        m[0][2] = 0.0f;
        m[0][3] = 0.0f;
        m[1][0] = sinA;
        m[1][1] = cosA;
        m[1][2] = 0.0f;
        m[1][3] = 0.0f;
        m[2][0] = 0.0f;
        m[2][1] = 0.0f;
        m[2][2] = 1.0f;
        m[2][3] = 0.0f;
        break;
    default:
        /* An invalid axis only trips an assert in a debug build; the retail
         * routine leaves the matrix alone. */
        break;
    }
}

void MTXRotRad(Mtx m, char axis, f32 rad)
{
    f32 sinA;
    f32 cosA;

    sinA = sinf(rad);
    cosA = cosf(rad);
    mtx_rot_trig(m, axis, sinA, cosA);
}

/* PSMTXQuat follows the retail paired-single code (80342690), not C_MTXQuat: n = (z*z + x*x) +
 * (w*w + y*y) (ps_mul, ps_madd, ps_sum0); s = 2 * (r * (2 - n*r)) with r the fres estimate (ps_nmsub,
 * ps_mul, fmuls); each element one ps_madd / ps_msub / ps_nmsub per lane. The reference's exact 2/n
 * is a few ULP off: Fox's tail (lbDynamics lb_8001044C) rotated 1-2 ULP off the console from frame
 * -123, which reached the hurt capsules (Dolphin oracle, notes/fp-audit.md "PSMTXQuat"). One
 * statement per rounding, so clang contracts nothing. */
void PSMTXQuat(Mtx m, QuaternionPtr q)
{
    f32 x = q->x, y = q->y, z = q->z, w = q->w;
    f32 xx, yy, zz, zz_xx, ww_yy, n, r, s, t;
    f32 xx_yy, zz_yy, yw, xw, zw, xz_p, yz_p;

    xx = x * x;
    yy = y * y;
    zz_xx = mtx_madd(z, z, xx);
    ww_yy = mtx_madd(w, w, yy);
    zz = z * z;
    n = zz_xx + ww_yy;
    r = (f32) __fres((double) n);
    t = mtx_msub(n, r, 2.0f);
    t = -t;
    s = r * t;
    s = s * 2.0f;
    xx_yy = xx + yy;
    zz_yy = zz + yy;
    yw = y * w;
    xw = x * w;
    zw = z * w;
    t = mtx_madd(x, y, zw);
    m[1][0] = t * s;
    t = mtx_msub(x, y, zw);
    m[0][1] = t * s;
    t = mtx_msub(xx_yy, s, 1.0f);
    m[2][2] = -t;
    t = mtx_msub(zz_xx, s, 1.0f);
    m[1][1] = -t;
    t = mtx_msub(zz_yy, s, 1.0f);
    m[0][0] = -t;
    xz_p = mtx_madd(x, z, yw);
    yz_p = mtx_madd(y, z, xw);
    m[0][2] = xz_p * s;
    m[2][1] = yz_p * s;
    t = mtx_msub(yw, 2.0f, xz_p);
    t = -t;
    m[2][0] = t * s;
    t = mtx_msub(xw, 2.0f, yz_p);
    t = -t;
    m[1][2] = t * s;
    m[0][3] = 0.0f;
    m[1][3] = 0.0f;
    m[2][3] = 0.0f;
}

/* PSMTXMultVec and PSMTXMultVecSR follow the retail paired-single code's dataflow exactly, not
 * the C_MTX reference's grouping: the two differ in the last bit, and a stage's collision
 * vertices go through here every frame (mpLib_80055E9C), so one ULP in a wall moved a wall push by
 * one ULP (.slp parity: Fox under Fountain of Dreams' edge, frame 228). Each product and sum is
 * its own statement, so clang never contracts it into something the Gekko did not do.
 *
 * PSMTXMultVec (mtxvec.c): ps_mul (m00 x, m01 y); ps_madd (m02 z + that, m03 * 1 + that);
 * ps_sum0 adds the two lanes. So x' = fma(m02, z, m00 x) + (m03 + m01 y). */
void PSMTXMultVec(Mtx44 m, Vec* src, Vec* dst)
{
    float x = src->x, y = src->y, z = src->z; /* src and dst are frequently the same vector */
    float p0, p1, s0, s1, rx, ry, rz;
    int r;
    for (r = 0; r < 3; r++) {
        p0 = m[r][0] * x;
        p1 = m[r][1] * y;
        s0 = mtx_madd(m[r][2], z, p0);
        s1 = m[r][3] + p1; /* ps_madd with c = 1.0: the product is exact */
        if (r == 0) {
            rx = s0 + s1;
        } else if (r == 1) {
            ry = s0 + s1;
        } else {
            rz = s0 + s1;
        }
    }
    dst->x = rx;
    dst->y = ry;
    dst->z = rz;
}

/* PSMTXMultVecSR (mtxvec.c): ps_mul (m00 x, m01 y); ps_sum0 adds them; ps_madd m02 z onto that.
 * So x' = fma(m02, z, m00 x + m01 y). */
void PSMTXMultVecSR(Mtx44 m, Vec* src, Vec* dst)
{
    float x = src->x, y = src->y, z = src->z;
    float p0, p1, s, rx, ry, rz;
    int r;
    for (r = 0; r < 3; r++) {
        p0 = m[r][0] * x;
        p1 = m[r][1] * y;
        s = p0 + p1;
        s = mtx_madd(m[r][2], z, s);
        if (r == 0) {
            rx = s;
        } else if (r == 1) {
            ry = s;
        } else {
            rz = s;
        }
    }
    dst->x = rx;
    dst->y = ry;
    dst->z = rz;
}

void PSVECAdd(Vec* a, Vec* b, Vec* c)
{
    c->x = a->x + b->x;
    c->y = a->y + b->y;
    c->z = a->z + b->z;
}

void PSVECSubtract(Vec* a, Vec* b, Vec* c)
{
    c->x = a->x - b->x;
    c->y = a->y - b->y;
    c->z = a->z - b->z;
}

void PSVECScale(Vec* src, Vec* dst, f32 scale)
{
    dst->x = (src->x * scale);
    dst->y = (src->y * scale);
    dst->z = (src->z * scale);
}

/* Dolphin's Force25Bit: a single-precision multiply's frC operand keeps 25 mantissa bits,
 * rounded (Source/Core/Core/PowerPC/Interpreter/Interpreter_FPUtils.h). */
static double gekko_force25(double d)
{
    union {
        double d;
        unsigned long long u;
    } v;
    v.d = d;
    v.u = (v.u & 0xFFFFFFFFF8000000ULL) + (v.u & 0x8000000ULL);
    return v.d;
}

/* The SDK's paired-single PSVECNormalize, operation for operation:
 *     ps_mul   xx_yy = v.xy * v.xy
 *     ps_madd  xx_zz = v.z * v.z + xx_yy        (psq_l of z sets ps1 = 1.0)
 *     ps_sum0  sqsum = xx_zz.ps0 + xx_yy.ps1    = (z*z + x*x) + y*y
 *     frsqrte  rsqrt = estimate(sqsum)
 *     fmuls    n0 = rsqrt * rsqrt ; fmuls n1 = rsqrt * 0.5
 *     fnmsubs  n0 = -(n0 * sqsum - 3)
 *     fmuls    rsqrt = n0 * n1                  (ONE Newton step)
 *     ps_muls0 v * rsqrt
 * One refinement of the hardware estimate is not correctly rounded: a flat floor line normalizes
 * to y = 0.99999988, not 1. That value multiplies every grounded fighter's speed
 * (ftCommon_SetSelfMovementFromGroundedMovement), so the correctly rounded 1/sqrtf the port used
 * before moved fighters a few ULPs further per frame than the console - found by .slp playback,
 * where it was the first thing to diverge. Fused steps (ps_madd, fnmsubs) are exact in double here:
 * their single-precision operands' products fit in 53 bits.
 *
 * A zero-length input still yields a non-finite scale, as on the console (frsqrte(0) = +inf,
 * refined into a NaN); callers never feed it one, so do not "fix" it. */
void PSVECNormalize(Vec* vec1, Vec* dst)
{
    f32 x = vec1->x, y = vec1->y, z = vec1->z;
    f32 xx = x * x;
    f32 yy = y * y;
    f32 xz = (f32) ((double) z * (double) z + (double) xx);
    f32 sqsum = xz + yy;
    double est = __frsqrte((double) sqsum);
    f32 n0 = (f32) (est * gekko_force25(est));
    f32 n1 = (f32) (est * 0.5);
    f32 n2 = (f32) (-((double) n0 * (double) sqsum - 3.0));
    f32 rsqrt = n2 * n1;
    dst->x = x * rsqrt;
    dst->y = y * rsqrt;
    dst->z = z * rsqrt;
}

/* PSVECMag, PSVECDotProduct and PSVECCrossProduct follow the retail paired-single code (vec.c), one
 * statement per rounding (see PSMTXInverse). PSVECMag: ps_mul, ps_madd, ps_sum0 give
 * (z*z + x*x) + y*y, then frsqrte refined once as in PSVECNormalize, fsel keeps a zero length 0,
 * and the result is sqsum * rsqrt - not a correctly rounded sqrtf. */
f32 PSVECMag(Vec* v)
{
    f32 x = v->x, y = v->y, z = v->z;
    f32 xx = x * x;
    f32 yy = y * y;
    f32 xz = mtx_madd(z, z, xx);
    f32 sqsum = xz + yy;
    double est = __frsqrte((double) sqsum);
    f32 n0 = (f32) (est * gekko_force25(est));
    f32 n1 = (f32) (est * 0.5);
    f32 n2 = (f32) (-((double) n0 * (double) sqsum - 3.0));
    f32 rsqrt = n2 * n1;
    if (!(rsqrt >= 0.0f)) {
        rsqrt = sqsum;
    }
    return sqsum * rsqrt;
}

/* ps_mul (y1*y2, z1*z2); ps_madd x1*x2 + y1*y2; ps_sum0 adds z1*z2. */
f32 PSVECDotProduct(Vec* vec1, Vec* vec2)
{
    f32 yy = vec1->y * vec2->y;
    f32 zz = vec1->z * vec2->z;
    f32 xy = mtx_madd(vec1->x, vec2->x, yy);
    return xy + zz;
}

/* ps_msub then ps_neg for y and z: x = ay*bz - (by*az), y = -(ax*bz - (bx*az)),
 * z = -(ay*bx - (by*ax)), the rounded product being the second one in each. */
void PSVECCrossProduct(Vec* vec1, Vec* vec2, Vec* dst)
{
    f32 ax = vec1->x, ay = vec1->y, az = vec1->z;
    f32 bx = vec2->x, by = vec2->y, bz = vec2->z;
    f32 p;
    Vec vTmp;

    /* dst may alias either source. */
    p = by * az;
    vTmp.x = mtx_msub(ay, bz, p);
    p = bx * az;
    vTmp.y = -mtx_msub(ax, bz, p);
    p = by * ax;
    vTmp.z = -mtx_msub(ay, bx, p);
    dst->x = vTmp.x;
    dst->y = vTmp.y;
    dst->z = vTmp.z;
}

/* C_MTXLookAt is C in the SDK, which ships built with MWCC -fp_contract off (configure.py
 * DolphinLib): the dot products are not fused. */
#pragma STDC FP_CONTRACT OFF
void C_MTXLookAt(Mtx m, Point3dPtr camPos, VecPtr camUp, Point3dPtr target)
{
    Vec vLook;
    Vec vRight;
    Vec vUp;

    vLook.x = camPos->x - target->x;
    vLook.y = camPos->y - target->y;
    vLook.z = camPos->z - target->z;
    PSVECNormalize(&vLook, &vLook);

    PSVECCrossProduct(camUp, &vLook, &vRight);
    PSVECNormalize(&vRight, &vRight);
    PSVECCrossProduct(&vLook, &vRight, &vUp);

    m[0][0] = vRight.x;
    m[0][1] = vRight.y;
    m[0][2] = vRight.z;
    m[0][3] = -((camPos->z * vRight.z) +
                ((camPos->x * vRight.x) + (camPos->y * vRight.y)));

    m[1][0] = vUp.x;
    m[1][1] = vUp.y;
    m[1][2] = vUp.z;
    m[1][3] =
        -((camPos->z * vUp.z) + ((camPos->x * vUp.x) + (camPos->y * vUp.y)));

    m[2][0] = vLook.x;
    m[2][1] = vLook.y;
    m[2][2] = vLook.z;
    m[2][3] = -((camPos->z * vLook.z) +
                ((camPos->x * vLook.x) + (camPos->y * vLook.y)));
}

#pragma STDC FP_CONTRACT ON

/* The light-matrix routines live in extern/dolphin/src/dolphin/mtx/mtx.c, which does not compile
 * for the port; these are that file's plain-C bodies, unchanged. */

void MTXLightFrustum(Mtx m, f32 t, f32 b, f32 l, f32 r, f32 n, f32 scaleS,
                     f32 scaleT, f32 transS, f32 transT)
{
    f32 tmp;

    tmp = 1 / (r - l);
    m[0][0] = (scaleS * (2 * n * tmp));
    m[0][1] = 0;
    m[0][2] = (scaleS * (tmp * (r + l))) - transS;
    m[0][3] = 0;
    tmp = 1 / (t - b);
    m[1][0] = 0;
    m[1][1] = (scaleT * (2 * n * tmp));
    m[1][2] = (scaleT * (tmp * (t + b))) - transT;
    m[1][3] = 0;
    m[2][0] = 0;
    m[2][1] = 0;
    m[2][2] = -1;
    m[2][3] = 0;
}

void MTXLightPerspective(Mtx m, f32 fovY, f32 aspect, f32 scaleS, f32 scaleT,
                         f32 transS, f32 transT)
{
    f32 angle;
    f32 cot;

    angle = (0.5f * fovY);
    angle = angle * 0.017453293f;
    cot = 1 / tanf(angle);
    m[0][0] = (scaleS * (cot / aspect));
    m[0][1] = 0;
    m[0][2] = -transS;
    m[0][3] = 0;
    m[1][0] = 0;
    m[1][1] = (cot * scaleT);
    m[1][2] = -transT;
    m[1][3] = 0;
    m[2][0] = 0;
    m[2][1] = 0;
    m[2][2] = -1;
    m[2][3] = 0;
}

void MTXLightOrtho(Mtx m, f32 t, f32 b, f32 l, f32 r, f32 scaleS, f32 scaleT,
                   f32 transS, f32 transT)
{
    f32 tmp;

    tmp = 1 / (r - l);
    m[0][0] = (2 * tmp * scaleS);
    m[0][1] = 0;
    m[0][2] = 0;
    m[0][3] = (transS + (scaleS * (tmp * -(r + l))));
    tmp = 1 / (t - b);
    m[1][0] = 0;
    m[1][1] = (2 * tmp * scaleT);
    m[1][2] = 0;
    m[1][3] = (transT + (scaleT * (tmp * -(t + b))));
    m[2][0] = 0;
    m[2][1] = 0;
    m[2][2] = 0;
    m[2][3] = 1;
}
