/* C reference for src/tu/ftclink.rs: ftCl_Init_80149114 cut verbatim by tu_shard89.py.
 * ftCo_80092ED8 (not in this TU) is a stand-in both sides share: b - (float) a. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include "ft/kinds/ftLink/types.h"
#include <stdio.h>

ftCommonData* p_ftCommonData;
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }
float ftCo_80092ED8(int a, float b) { return b - (float) a; }
static u32 sfx;
void ft_PlaySFX(Fighter* fp, enum_t id, int vol, int pan) { (void) fp; sfx = id + vol + pan; }

#include "tu_ftclink_extract.inc"

static Fighter F;
static HSD_GObj G;
static ftData FD;
static ftLk_DatAttrs EA;
static ftCommonData CD;
static u32 s = 0xC11C11u;
static u32 rnd(void) { s ^= s << 13; s ^= s >> 17; s ^= s << 5; return s; }
static float rf(void)
{
    switch (rnd() % 8) {
    case 0: return 0.0f;
    case 1: return -0.0f;
    case 2: return 1.0f;
    case 3: return -1.0f;
    default: return (float) ((int) (rnd() % 4001) - 2000) / 997.0f;
    }
}
static u32 b(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }

int main(void)
{
    int i;
    G.user_data = &F;
    F.ft_data = &FD;
    FD.ext_attr = &EA;
    p_ftCommonData = &CD;
    for (i = 0; i < 4000; i++) {
        F.x19A4 = (int) (rnd() % 61) - 30;
        EA.xD8 = rf();
        CD.x294 = rf();
        F.specialn_facing_dir = rf();
        sfx = 0;
        ftCl_Init_80149114(&G);
        printf("%d %08x %u\n", i, b(F.gr_vel), sfx);
    }
    return 0;
}
