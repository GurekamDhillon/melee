/* C reference for src/tu/ftfoxspeciallw.rs: ftFx_SpecialLw_Turn and ftFox_SpecialLw_Turn_Inline
 * (the fused `-(MTXDegToRad(180 / x9C) - ftPartGetRotZ(fp, 0))`), cut verbatim by
 * tu_shard289.py. ftPartGetRotZ / ftPartSetRotY are recorders both sides share. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include "ft/kinds/ftFox/types.h"
#include <dolphin/mtx.h>
#include <stdio.h>

ftCommonData* p_ftCommonData;
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }
static float g_rotz;
static float g_roty;
float ftPartGetRotZ(Fighter* fp, int idx) { (void) fp; (void) idx; return g_rotz; }
void ftPartSetRotY(Fighter* fp, int idx, f32 v) { (void) fp; (void) idx; g_roty = v; }

#include "tu_ftfoxspeciallw_extract.inc"

static Fighter F;
static HSD_GObj G;
static ftFox_DatAttrs DA;
static u32 s = 0x70E1F2u;
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
        DA.x9C_FOX_REFLECTOR_TURN_FRAMES = (rnd() & 3) ? (float) (rnd() % 30) + rf() : rf();
        F.facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
        F.cmd_vars[0] = rnd() % 2;
        F.mv.fx.SpecialLw.turnFrames = (int) (rnd() % 40) - 5;
        g_rotz = rf() * 3.0f;
        g_roty = 0.0f;
        if (i & 1) {
            ftFx_SpecialLw_Turn(&G);
        } else {
            ftFox_SpecialLw_Turn_Inline(&G);
        }
        printf("%d %08x %08x %d %u\n", i, b(g_roty), b(F.facing_dir), F.mv.fx.SpecialLw.turnFrames,
               (unsigned) F.cmd_vars[0]);
    }
    return 0;
}
