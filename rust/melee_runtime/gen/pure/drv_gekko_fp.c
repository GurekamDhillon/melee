/* C oracle for melee_runtime::tu::gekko_fp::{__frsqrte, __fres}: pc/gameworld/gekko_fp.c is linked (run_pure.py always adds it).
 * Inputs: random bit patterns, random floats widened, and the special values. */
#include "common.h"
extern int printf(const char*, ...);
extern double __frsqrte(double);
extern double __fres(double);

static double bits(u64_ u) { union { u64_ u; double d; } c; c.u = u; return c.d; }
static u64_ ub(double d) { union { u64_ u; double d; } c; c.d = d; return c.u; }

int main(void)
{
    int k;
    for (k = 0; k < 3000; k++) {
        double x;
        switch (k % 5) {
        case 0: x = bits(rnd()); break;
        case 1: x = (double) rndf(); break;
        case 2: x = bits((rnd() & 0x800FFFFFFFFFFFFFULL) | ((u64_) (rnd() % 2048) << 52)); break;
        case 3: x = bits(rnd() & 0x000FFFFFFFFFFFFFULL); break;
        default: x = (double) (rnd() % 100000) / 7.0; break;
        }
        printf("frsqrte %d %016llx %016llx\n", k, ub(x), ub(__frsqrte(x)));
        printf("fres %d %016llx %016llx\n", k, ub(x), ub(__fres(x)));
    }
    return 0;
}
