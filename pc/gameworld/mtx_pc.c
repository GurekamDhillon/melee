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
 * model of the Gekko's fused multiply-add (the same as gwtool's lowering of fmuladd). */
static float mtx_madd(float a, float c, float b)
{
    return (float) ((double) a * (double) c + (double) b);
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

u32 PSMTXInverse(Mtx src, Mtx inv)
{
    Mtx mTmp;
    MtxPtr m;
    f32 det;

    if (src == inv) {
        m = mTmp;
    } else {
        m = inv;
    }

    det = ((((src[2][1] * (src[0][2] * src[1][0])) +
             ((src[2][2] * (src[0][0] * src[1][1])) +
              (src[2][0] * (src[0][1] * src[1][2])))) -
            (src[0][2] * (src[2][0] * src[1][1]))) -
           (src[2][2] * (src[1][0] * src[0][1]))) -
          (src[1][2] * (src[0][0] * src[2][1]));

    /* Singular: leave the destination untouched and report failure, exactly
     * as both the C and the paired-single original do. */
    if (det == 0.0f) {
        return 0;
    }

    det = 1.0f / det;

    m[0][0] = (det * +((src[1][1] * src[2][2]) - (src[2][1] * src[1][2])));
    m[0][1] = (det * -((src[0][1] * src[2][2]) - (src[2][1] * src[0][2])));
    m[0][2] = (det * +((src[0][1] * src[1][2]) - (src[1][1] * src[0][2])));

    m[1][0] = (det * -((src[1][0] * src[2][2]) - (src[2][0] * src[1][2])));
    m[1][1] = (det * +((src[0][0] * src[2][2]) - (src[2][0] * src[0][2])));
    m[1][2] = (det * -((src[0][0] * src[1][2]) - (src[1][0] * src[0][2])));

    m[2][0] = (det * +((src[1][0] * src[2][1]) - (src[2][0] * src[1][1])));
    m[2][1] = (det * -((src[0][0] * src[2][1]) - (src[2][0] * src[0][1])));
    m[2][2] = (det * +((src[0][0] * src[1][1]) - (src[1][0] * src[0][1])));

    m[0][3] =
        ((-m[0][0] * src[0][3]) - (m[0][1] * src[1][3])) - (m[0][2] * src[2][3]);
    m[1][3] =
        ((-m[1][0] * src[0][3]) - (m[1][1] * src[1][3])) - (m[1][2] * src[2][3]);
    m[2][3] =
        ((-m[2][0] * src[0][3]) - (m[2][1] * src[1][3])) - (m[2][2] * src[2][3]);

    if (m == mTmp) {
        PSMTXCopy(mTmp, inv);
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

    m[0][0] = (c + (t * (x * x)));
    m[0][1] = (y * (t * x)) - (s * z);
    m[0][2] = (z * (t * x)) + (s * y);
    m[0][3] = 0.0f;
    m[1][0] = ((y * (t * x)) + (s * z));
    m[1][1] = (c + (t * (y * y)));
    m[1][2] = ((z * (t * y)) - (s * x));
    m[1][3] = 0.0f;
    m[2][0] = ((z * (t * x)) - (s * y));
    m[2][1] = ((z * (t * y)) + (s * x));
    m[2][2] = (c + (t * (z * z)));
    m[2][3] = 0.0f;
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

void PSMTXQuat(Mtx m, QuaternionPtr q)
{
    f32 s;
    f32 xs, ys, zs;
    f32 wx, wy, wz;
    f32 xx, xy, xz;
    f32 yy, yz;
    f32 zz;

    s = 2.0f /
        ((q->w * q->w) + ((q->z * q->z) + ((q->x * q->x) + (q->y * q->y))));
    xs = q->x * s;
    ys = q->y * s;
    zs = q->z * s;
    wx = q->w * xs;
    wy = q->w * ys;
    wz = q->w * zs;
    xx = q->x * xs;
    xy = q->x * ys;
    xz = q->x * zs;
    yy = q->y * ys;
    yz = q->y * zs;
    zz = q->z * zs;

    m[0][0] = (1.0f - (yy + zz));
    m[0][1] = (xy - wz);
    m[0][2] = (xz + wy);
    m[0][3] = 0.0f;
    m[1][0] = (xy + wz);
    m[1][1] = (1.0f - (xx + zz));
    m[1][2] = (yz - wx);
    m[1][3] = 0.0f;
    m[2][0] = (xz - wy);
    m[2][1] = (yz + wx);
    m[2][2] = (1.0f - (xx + yy));
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

f32 PSVECMag(Vec* v)
{
    return sqrtf((v->z * v->z) + ((v->x * v->x) + (v->y * v->y)));
}

f32 PSVECDotProduct(Vec* vec1, Vec* vec2)
{
    return (vec1->z * vec2->z) + ((vec1->x * vec2->x) + (vec1->y * vec2->y));
}

void PSVECCrossProduct(Vec* vec1, Vec* vec2, Vec* dst)
{
    Vec vTmp;

    /* dst may alias either source. */
    vTmp.x = (vec1->y * vec2->z) - (vec1->z * vec2->y);
    vTmp.y = (vec1->z * vec2->x) - (vec1->x * vec2->z);
    vTmp.z = (vec1->x * vec2->y) - (vec1->y * vec2->x);
    dst->x = vTmp.x;
    dst->y = vTmp.y;
    dst->z = vTmp.z;
}

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
