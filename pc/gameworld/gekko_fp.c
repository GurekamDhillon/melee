// SPDX-License-Identifier: GPL-2.0-or-later
//
// Portions of this file are derived from Dolphin Emulator's Source/Core/Common/FloatUtils.cpp
// (Copyright 2018 Dolphin Emulator Project, also GPL-2.0-or-later): the frsqrte/fres estimate
// tables and the surrounding normalisation logic. See pc/LICENSE and pc/README.md.
//
/* Bit-exact Gekko floating-point estimate instructions.
 *
 * This is game-world code: it is compiled for PowerPC and byte-swapped by gwtool like the rest of
 * melee, so it needs no endian handling of its own.
 *
 * The GameCube's frsqrte is a hardware *estimate* with a documented 32-entry lookup table, not a
 * correctly rounded reciprocal square root. Melee feeds its result straight into Newton-Raphson
 * refinement (see src/MSL/math_ppc.h: sqrtf and sqrtf_accurate), and those results drive fighter
 * physics -- distances, normalization, knockback. src/placeholder.h substitutes sqrt() for
 * __frsqrte on non-CodeWarrior compilers, which silently changes every one of those results, so
 * the port uses this instead.
 *
 * The table and the algorithm match the hardware behaviour as implemented by Dolphin
 * (Source/Core/Common/FloatUtils.cpp, ApproximateReciprocalSquareRoot).
 */

typedef unsigned long long u64_t;
typedef long long s64_t;

typedef struct {
    int base;
    int dec;
} GekkoEstimateEntry;

static const GekkoEstimateEntry frsqrte_expected[32] = {
    { 0x1a7e800, -0x568 }, { 0x17cb800, -0x4f3 }, { 0x1552800, -0x48d },
    { 0x130c000, -0x435 }, { 0x10f2000, -0x3e7 }, { 0x0eff000, -0x3a2 },
    { 0x0d2e000, -0x365 }, { 0x0b7c000, -0x32e }, { 0x09e5000, -0x2fc },
    { 0x0867000, -0x2d0 }, { 0x06ff000, -0x2a8 }, { 0x05ab800, -0x283 },
    { 0x046a000, -0x261 }, { 0x0339800, -0x243 }, { 0x0218800, -0x226 },
    { 0x0105800, -0x20b }, { 0x3ffa000, -0x7a4 }, { 0x3c29000, -0x700 },
    { 0x38aa000, -0x670 }, { 0x3572000, -0x5f2 }, { 0x3279000, -0x584 },
    { 0x2fb7000, -0x524 }, { 0x2d26000, -0x4cc }, { 0x2ac0000, -0x47e },
    { 0x2881000, -0x43a }, { 0x2665000, -0x3fa }, { 0x2468000, -0x3c2 },
    { 0x2287000, -0x38e }, { 0x20c1000, -0x35e }, { 0x1f12000, -0x332 },
    { 0x1d79000, -0x30a }, { 0x1bf4000, -0x2e6 },
};

static const GekkoEstimateEntry fres_expected[32] = {
    { 0x7ff800, 0x3e1 }, { 0x783800, 0x3a7 }, { 0x70ea00, 0x371 },
    { 0x6a0800, 0x340 }, { 0x638800, 0x313 }, { 0x5d6200, 0x2ea },
    { 0x579000, 0x2c4 }, { 0x520800, 0x2a0 }, { 0x4cc800, 0x27f },
    { 0x47ca00, 0x261 }, { 0x430800, 0x245 }, { 0x3e8000, 0x22a },
    { 0x3a2c00, 0x212 }, { 0x360800, 0x1fb }, { 0x321400, 0x1e5 },
    { 0x2e4a00, 0x1d1 }, { 0x2aa800, 0x1be }, { 0x272c00, 0x1ac },
    { 0x23d600, 0x19b }, { 0x209e00, 0x18b }, { 0x1d8800, 0x17c },
    { 0x1a9000, 0x16e }, { 0x17ae00, 0x15b }, { 0x14f800, 0x15b },
    { 0x124400, 0x143 }, { 0x0fbe00, 0x143 }, { 0x0d3800, 0x12d },
    { 0x0ade00, 0x12d }, { 0x088400, 0x11a }, { 0x065000, 0x11a },
    { 0x041c00, 0x108 }, { 0x020c00, 0x106 },
};

union GekkoFloatBits {
    double d;
    u64_t u;
};

static double gekko_bits_to_double(u64_t bits)
{
    union GekkoFloatBits v;
    v.u = bits;
    return v.d;
}

static u64_t gekko_double_to_bits(double d)
{
    union GekkoFloatBits v;
    v.d = d;
    return v.u;
}

#define GEKKO_EXP_MASK (0x7FFLL << 52)
#define GEKKO_MANTISSA_MASK ((1LL << 52) - 1)
#define GEKKO_SIGN_BIT (1ULL << 63)
#define GEKKO_QUIET_BIT (1ULL << 51)

static double gekko_make_quiet(double val)
{
    return gekko_bits_to_double(gekko_double_to_bits(val) | GEKKO_QUIET_BIT);
}

static double gekko_infinity(int negative)
{
    return gekko_bits_to_double(negative ? (GEKKO_SIGN_BIT | GEKKO_EXP_MASK)
                                         : (u64_t) GEKKO_EXP_MASK);
}

static double gekko_nan(void)
{
    return gekko_bits_to_double(GEKKO_EXP_MASK | GEKKO_QUIET_BIT);
}

double __frsqrte(double val)
{
    s64_t integral = (s64_t) gekko_double_to_bits(val);
    s64_t mantissa = integral & GEKKO_MANTISSA_MASK;
    const s64_t sign = integral & (s64_t) GEKKO_SIGN_BIT;
    s64_t exponent = integral & GEKKO_EXP_MASK;
    s64_t exponent_lsb;
    int i;
    int index;

    if (mantissa == 0 && exponent == 0) {
        return gekko_infinity(sign != 0);
    }

    if (exponent == GEKKO_EXP_MASK) {
        if (mantissa == 0) {
            return sign ? gekko_nan() : 0.0;
        }
        return gekko_make_quiet(val);
    }

    if (sign) {
        return gekko_nan();
    }

    if (exponent == 0) {
        /* Normalize a denormal input the way the hardware does. */
        do {
            exponent -= 1LL << 52;
            mantissa <<= 1;
        } while (!(mantissa & (1LL << 52)));
        mantissa &= GEKKO_MANTISSA_MASK;
        exponent += 1LL << 52;
    }

    exponent_lsb = exponent & (1LL << 52);
    exponent = ((0x3FFLL << 52) - ((exponent - (0x3FELL << 52)) / 2)) & GEKKO_EXP_MASK;
    integral = sign | exponent;

    i = (int) ((exponent_lsb | mantissa) >> 37);
    index = i / 2048;
    integral |= (s64_t) (frsqrte_expected[index].base +
                         frsqrte_expected[index].dec * (i % 2048))
                << 26;

    return gekko_bits_to_double((u64_t) integral);
}

double __fres(double val)
{
    s64_t integral = (s64_t) gekko_double_to_bits(val);
    const s64_t mantissa = integral & GEKKO_MANTISSA_MASK;
    const s64_t sign = integral & (s64_t) GEKKO_SIGN_BIT;
    s64_t exponent = integral & GEKKO_EXP_MASK;
    int i;
    int index;

    if (mantissa == 0 && exponent == 0) {
        return gekko_infinity(sign != 0);
    }

    if (exponent == GEKKO_EXP_MASK) {
        if (mantissa == 0) {
            return gekko_bits_to_double((u64_t) sign);
        }
        return gekko_make_quiet(val);
    }

    /* Inputs too small overflow to the largest float; too large flush to zero. */
    if (exponent < (895LL << 52)) {
        return gekko_bits_to_double((u64_t) sign | 0x47EFFFFFE0000000ULL);
    }
    if (exponent >= (1149LL << 52)) {
        return gekko_bits_to_double((u64_t) sign);
    }

    exponent = (0x7FDLL << 52) - exponent;

    i = (int) (mantissa >> 37);
    index = i / 1024;
    integral = sign | exponent;
    integral |= (s64_t) (fres_expected[index].base -
                         (fres_expected[index].dec * (i % 1024) + 1) / 2)
                << 29;

    return gekko_bits_to_double((u64_t) integral);
}

/* ---- MSL sinf / cosf / tanf ----------------------------------------------------------------
 *
 * The game's trig is MSL's (src/MSL/trigf.c + math_data.c, both Matching), not a correctly rounded
 * libm: a four-term range reduction into eighth-turns and two short polynomials, every a*b+c step
 * of which the retail binary computes with a FUSED fmadds / fnmsubs / fnmadds (checked against
 * the DOL: sinf @ 0x803263D4, cosf @ 0x80326240). The port used to forward these to the Windows
 * CRT, whose results differ in the last bits - a knockback angle's sin/cos then rotated the
 * launch vector by an ULP (found by .slp playback: Falcon's frame-52 knockback, x smaller and y
 * larger than the console's by one ULP each).
 *
 * gk_fmadds reproduces a fused single-precision multiply-add without FMA hardware: both factors
 * are floats, so their product is exact in double (48 bits), and the sum is rounded to double and
 * then once more to float - the same model Dolphin's interpreter uses. Tables and constants are
 * math_data.c's and trigf.c's, whose literals the Matching build proves produce the retail bits;
 * __four_over_pi_m1 is what trigf.c's static constructor copies out of tmp_float. */
static float gk_fmadds(float a, float c, float b)
{
    return (float) ((double) a * (double) c + (double) b);
}

static const float gk_sincos_on_quadrant[] = { 0, 1, 1, 0, 0, -1, -1, 0 };
static const float gk_sincos_poly[] = {
    0.0000035287617, 0.0000003089747, -0.0003259365, -0.00003657235, 0.015854323,
    0.0024903931,    -0.30842513,     -0.08074551,   1,              0.7853982,
};
static const float gk_four_over_pi_m1[] = { 0.25f, 0.0232393741608f, 1.70555722434e-7f,
                                            1.86736494323e-11f };
#define GK_SINCOS_EPSILON 3.45266983e-4f

/* x in eighth-turns: y = (4/pi)x - 2n, as x - 2n + x*(4/pi - 1) in four fused steps. */
static float gk_sincos_reduce(float x, int* quadrant)
{
    union {
        float f;
        unsigned int u;
    } bits;
    float z = (2.0f / 3.14159265358979323846f) * x;
    int n;
    float y;
    bits.f = x;
    n = (bits.u & 0x80000000u) ? (int) (z - 0.5f) : (int) (z + 0.5f);
    y = x - (float) (n * 2);
    y = gk_fmadds(gk_four_over_pi_m1[0], x, y);
    y = gk_fmadds(gk_four_over_pi_m1[1], x, y);
    y = gk_fmadds(gk_four_over_pi_m1[2], x, y);
    y = gk_fmadds(gk_four_over_pi_m1[3], x, y);
    *quadrant = n & 3;
    return y;
}

/* ((((p0 ysq + p2) ysq + p4) ysq + p6) ysq + p8) */
static float gk_sincos_even(float ysq)
{
    const float* p = gk_sincos_poly;
    float z = gk_fmadds(p[0], ysq, p[2]);
    z = gk_fmadds(z, ysq, p[4]);
    z = gk_fmadds(z, ysq, p[6]);
    return gk_fmadds(z, ysq, p[8]);
}

/* (((p1 ysq + p3) ysq + p5) ysq + p7), the odd polynomial before its last step */
static float gk_sincos_odd_head(float ysq)
{
    const float* p = gk_sincos_poly;
    float z = gk_fmadds(p[1], ysq, p[3]);
    z = gk_fmadds(z, ysq, p[5]);
    return gk_fmadds(z, ysq, p[7]);
}

float sinf(float x)
{
    int n;
    float y = gk_sincos_reduce(x, &n);
    float ysq;
    if ((y < 0.0f ? -y : y) < GK_SINCOS_EPSILON) {
        n <<= 1;
        return gk_fmadds(gk_sincos_poly[9], gk_sincos_on_quadrant[n + 1] * y,
                         gk_sincos_on_quadrant[n]);
    }
    ysq = y * y;
    if (n & 1) {
        n <<= 1;
        return gk_sincos_even(ysq) * gk_sincos_on_quadrant[n];
    }
    n <<= 1;
    return (gk_fmadds(gk_sincos_odd_head(ysq), ysq, gk_sincos_poly[9]) * y) *
           gk_sincos_on_quadrant[n + 1];
}

float cosf(float x)
{
    int n;
    float y = gk_sincos_reduce(x, &n);
    float ysq;
    if ((y < 0.0f ? -y : y) < GK_SINCOS_EPSILON) {
        n <<= 1;
        /* fnmsubs: q[n+1] - y*q[n], fused */
        return (float) ((double) gk_sincos_on_quadrant[n + 1] -
                        (double) y * (double) gk_sincos_on_quadrant[n]);
    }
    ysq = y * y;
    if (n & 1) {
        n <<= 1;
        /* fnmadds: -(t*ysq + p9), fused, then * y, then * the quadrant */
        return (y * -gk_fmadds(gk_sincos_odd_head(ysq), ysq, gk_sincos_poly[9])) *
               gk_sincos_on_quadrant[n];
    }
    n <<= 1;
    return gk_sincos_even(ysq) * gk_sincos_on_quadrant[n + 1];
}

float tanf(float x)
{
    return sinf(x) / cosf(x);
}
