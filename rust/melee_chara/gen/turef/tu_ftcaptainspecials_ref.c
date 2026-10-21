/* C reference for src/tu/ftcaptainspecials.rs: ftCa_SpecialAirSStart_Phys / ftCa_SpecialAirS_Phys
 * cut verbatim by tu_shard89.py (their ft_80085134 call is a no-op here; the Rust test calls the
 * gravity half, specials_gravity, which is everything else they do). */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include "ft/kinds/ftCaptain/types.h"
#include <stdio.h>

ftCommonData* p_ftCommonData;
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }
void ft_80085134(HSD_GObj* g) { (void) g; }

#include "tu_ftcaptainspecials_extract.inc"

static Fighter F;
static HSD_GObj G;
static ftCaptain_DatAttrs DA;
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
        DA.specials_grav = rf();
        DA.specials_terminal_vel = rf();
        F.mv.ca.specials.grav = rf();
        F.self_vel.y = rf();
        F.cmd_vars[1] = rnd() % 3;
        if (i & 1) {
            ftCa_SpecialAirS_Phys(&G);
        } else {
            ftCa_SpecialAirSStart_Phys(&G);
        }
        printf("%d %08x %08x\n", i, b(F.mv.ca.specials.grav), b(F.self_vel.y));
    }
    return 0;
}
