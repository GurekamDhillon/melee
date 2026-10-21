/* C reference for src/tu/ftco_0d72.rs: ftCo_800D72A0 cut verbatim (tu_shard97.py). */
#include "ft/types.h"
#include <stdio.h>

static Fighter F;
static struct Fighter_x2D0_t X;
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }

#include "tu_ftco_0d72_extract.inc"

static u32 s = 0x2468ACEu;
static u32 rnd(void) { s ^= s << 13; s ^= s >> 17; s ^= s << 5; return s; }
static s32 ri(void)
{
    switch (rnd() % 8) {
    case 0: return -1;
    case 1: return 0x7FFFFFF0;
    case 2: return (s32) 0x80000000u;
    default: return (s32) (rnd() % 400);
    }
}

int main(void)
{
    int i;
    F.x2D0 = &X;
    for (i = 0; i < 4000; i++) {
        X.x2C = ri();
        X.x30 = ri();
        X.x28 = (rnd() % 4 == 0) ? ri() : (s32) (rnd() % 40);
        F.motion_id = (s32) (rnd() % 400);
        printf("%d %d\n", i, (int) ftCo_800D72A0(&F));
    }
    return 0;
}
