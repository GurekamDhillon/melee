/* C reference for src/tu/ftkirbyspecialcaptain.rs: ftKb_CaSpecialAirN_IASA cut verbatim (with
 * lbtrigf.c's sinf/cosf) by tu_shard305.py. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include "ft/kinds/ftKirby/ftkirby.h"
#include <stdio.h>

ftCommonData* p_ftCommonData;
float MSL_TrigF_80400770[1];
float MSL_TrigF_80400774[1];
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }

#include "tu_ftkirbyspecialcaptain_extract.inc"

static Fighter F;
static HSD_GObj G;
static ftKb_DatAttrs DA;
static u32 s = 0x13579BDu;
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
    for (i = 0; i < 4000; i++) {
        F.input.lstick[0].y = rf();
        DA.specialn_ca_y_axis_range = rf();
        DA.specialn_ca_x_axis_range = rf();
        DA.specialn_ca_angle_difference = rf() * 60.0f;
        DA.specialn_ca_forward_momentum = rf();
        F.facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
        F.cmd_vars[0] = rnd() & 1;
        F.self_vel.x = rf();
        F.self_vel.y = rf();
        ftKb_CaSpecialAirN_IASA(&G);
        printf("%d %08x %08x %u\n", i, b(F.self_vel.x), b(F.self_vel.y), F.cmd_vars[0]);
    }
    return 0;
}
