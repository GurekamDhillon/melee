/* Reference values for rust/melee_chara/src/pure.rs, from the C sources compiled natively.
 * Build+run: rust/melee_chara/gen/run_ref.sh (clang, -O2 -ffp-contract=off, x86-64 baseline).
 * Prints a Rust table: (input bits, my_sqrtf bits, frsqrte bits) and sumOfSquares cases. */
#include <stdio.h>
#include <string.h>
#include "../../../pc/gameworld/gekko_fp.c"

typedef unsigned int u32;

/* ftSeak/ftseakspecialhi.c, ftCrazyHand/*, ftMasterHand/* (all copies are the same math) */
static float my_sqrtf(float x)
{
    volatile float y;
    if (x > 0.0f) {
        double guess = __frsqrte(x);
        guess = 0.5 * guess * (3.0 - guess * guess * x);
        guess = 0.5 * guess * (3.0 - guess * guess * x);
        guess = 0.5 * guess * (3.0 - guess * guess * x);
        y = (float) (x * guess);
        return y;
    }
    return x;
}

/* ftSeak/ftseakspecials.c */
static float sumOfSquares(float a, float b)
{
    float c;
    a = a * a;
    c = b * b;
    return a + c;
}

static u32 fb(float f) { u32 u; memcpy(&u, &f, 4); return u; }
static float bf(u32 u) { float f; memcpy(&f, &u, 4); return f; }
static unsigned long long db(double d) { unsigned long long u; memcpy(&u, &d, 8); return u; }

int main(void)
{
    static const u32 in[] = {
        0x00000000, 0x80000000, 0x3F800000, 0x40000000, 0x40400000, 0x3E99999A, 0x42C80000,
        0x461C4000, 0x3A83126F, 0x00000001, 0x007FFFFF, 0x7F7FFFFF, 0x7F800000, 0xBF800000,
        0x7FC00000, 0x3F7FFFFF, 0x3F000001, 0x447A0000, 0x41200000, 0x3C23D70A,
    };
    unsigned int i, s = 12345;
    printf("pub const SQRT_CASES: &[(u32, u32, u64)] = &[\n");
    for (i = 0; i < sizeof in / sizeof in[0]; i++)
        printf("    (0x%08X, 0x%08X, 0x%016llX),\n", in[i], fb(my_sqrtf(bf(in[i]))),
               db(__frsqrte((double) bf(in[i]))));
    for (i = 0; i < 200; i++) { /* LCG-spread positive normals */
        u32 u;
        s = s * 1103515245u + 12345u;
        u = (s & 0x3FFFFFFF) + 0x08000000;
        printf("    (0x%08X, 0x%08X, 0x%016llX),\n", u, fb(my_sqrtf(bf(u))), db(__frsqrte((double) bf(u))));
    }
    printf("];\n");
    printf("pub const SUMSQ_CASES: &[(u32, u32, u32)] = &[\n");
    for (i = 0; i < 64; i++) {
        u32 a, b;
        s = s * 1103515245u + 12345u; a = (s & 0x0FFFFFFF) | 0x38000000 | ((s & 1) << 31);
        s = s * 1103515245u + 12345u; b = (s & 0x0FFFFFFF) | 0x38000000 | ((s & 1) << 31);
        printf("    (0x%08X, 0x%08X, 0x%08X),\n", a, b, fb(sumOfSquares(bf(a), bf(b))));
    }
    printf("];\n");
    return 0;
}
