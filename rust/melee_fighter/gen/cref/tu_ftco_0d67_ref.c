/* C reference for src/tu/ftco_0d67.rs: ftCo_800D67C4 cut verbatim (tu_shard97.py). */
#include "ft/types.h"
#include "ft/fighter.h"
#include <stdio.h>

static Fighter F;
static ftCommonData cd;
ftCommonData* p_ftCommonData;
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }

#include "tu_ftco_0d67_extract.inc"

static u32 s = 0x1234567u;
static u32 rnd(void) { s ^= s << 13; s ^= s >> 17; s ^= s << 5; return s; }
static float rf(void)
{
    switch (rnd() % 8) {
    case 0: return 0.0f;
    case 1: return -0.0f;
    case 2: return 0.5f;
    case 3: return -0.5f;
    default: return (float) ((int) (rnd() % 2001) - 1000) / 997.0f;
    }
}

int main(void)
{
    int i;
    p_ftCommonData = &cd;
    for (i = 0; i < 4000; i++) {
        F.input.pressed_buttons = rnd() & 0x300;
        F.input.lstick[0].x = rf();
        F.input.lstick[0].y = rf();
        cd.x218 = rf();
        cd.x21C = rf();
        printf("%d %d\n", i, (int) ftCo_800D67C4(&F));
    }
    return 0;
}
