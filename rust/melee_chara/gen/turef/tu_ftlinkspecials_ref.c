/* C reference for src/tu/ftlinkspecials.rs: calcAnglePos cut verbatim (with lbtrigf.c's
 * sinf/cosf/atan2f) by tu_shard337.py. Prints "i angle pos.x pos.y pos.z". */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include "ft/kinds/ftLink/types.h"
#include <stdio.h>

ftCommonData* p_ftCommonData;
float MSL_TrigF_80400770[1];
float MSL_TrigF_80400774[1];
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }

#include "tu_ftlinkspecials_extract.inc"

static Fighter F;
static HSD_GObj G;
static ftLk_DatAttrs DA;
static u32 s = 0x7A11E5u;
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
    F.dat_attrs = &DA;
    for (i = 0; i < 6000; i++) {
        Vec3 pos;
        float a, dist;
        F.input.lstick[0].x = rf();
        F.input.lstick[0].y = rf();
        F.facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
        DA.x14 = (rnd() % 4) ? (float) (rnd() % 100) / 150.0f : rf();
        DA.x18 = rf();
        DA.x1C = (rnd() % 3) ? 1.0f : rf();
        dist = rf() * 10.0f;
        a = calcAnglePos(&G, &pos, dist);
        printf("%d %08x %08x %08x %08x\n", i, b(a), b(pos.x), b(pos.y), b(pos.z));
    }
    return 0;
}
