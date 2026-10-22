/* C reference for src/tu/ftfoxspecialhi.rs: ftFx_SpecialAirHi_Phys (two fused multiply-subtracts,
 * sinf/cosf) and ftFx_SpecialAirHi_Enter (sinf/cosf/atan2f, stick tests, f32 -> int conversions),
 * with ftFox_SpecialHi_RotateModel and the real ftCommon_UpdateFacing, all cut verbatim by
 * tu_shard289.py. Calls outside the TU are recorders both sides share. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include "ft/kinds/ftFox/types.h"
#include "script_fighter_caps.h"
#include <math.h>
#include <stdio.h>

ftCommonData* p_ftCommonData;
float MSL_TrigF_80400770[1];
float MSL_TrigF_80400774[1];
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }
#define HALF_PI32 (1.5707963705062866f)
void ftFx_SpecialHi_CreateLaunchGFX(HSD_GObj* g) { (void) g; }
static float g_rotx;
static int g_ms;
void ftPartSetRotX(Fighter* fp, int idx, f32 v) { (void) fp; (void) idx; g_rotx = v; }
Fighter_Part ftParts_GetBoneIndex(Fighter* fp, Fighter_Part p) { (void) fp; return p; }
void Fighter_ChangeMotionState(Fighter_GObj* g, int msid, u32 flags, f32 a, f32 b, f32 c, Fighter_GObj* d)
{
    (void) g; (void) flags; (void) a; (void) b; (void) c; (void) d;
    g_ms = msid;
}
void ftAnim_8006EBA4(Fighter_GObj* g) { (void) g; }
void ftCommon_8007F76C(Fighter_GObj* g) { (void) g; }
int ScriptGame_CapsMaxJumps(Fighter* fp) { return fp->co_attrs.max_jumps; }

#include "tu_ftfoxspecialhi_extract.inc"

static Fighter F;
static HSD_GObj G;
static ftFox_DatAttrs DA;
static u32 s = 0xF0F1F2u;
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
        DA.x64_FOX_FIREFOX_DIRECTION_STICK_RANGE_MIN = rf() * 0.5f;
        DA.x68_FOX_FIREFOX_DURATION = rf() * 20.0f;
        DA.x70_FOX_FIREFOX_DURATION_END = (float) (rnd() % 6);
        DA.x74_FOX_FIREFOX_SPEED = rf();
        DA.x78_FOX_FIREFOX_REVERSE_ACCEL = rf();
        DA.x88_FOX_FIREFOX_FACING_STICK_RANGE_MIN = rf() * 0.5f;
        F.facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
        F.self_vel.x = rf();
        F.self_vel.y = rf();
        F.input.lstick[0].x = rf();
        F.input.lstick[0].y = rf();
        F.co_attrs.max_jumps = rnd() % 6;
        F.mv.fx.SpecialHi.rotateModel = rf() * 3.0f;
        F.mv.fx.SpecialHi.unk = rnd() % 6;
        g_rotx = 0.0f;
        if (i & 1) {
            ftFx_SpecialAirHi_Phys(&G);
            printf("%d p %08x %08x %d\n", i, b(F.self_vel.x), b(F.self_vel.y), F.mv.fx.SpecialHi.unk);
        } else {
            ftFx_SpecialAirHi_Enter(&G);
            printf("%d e %08x %08x %08x %08x %d %d %d %d %d\n", i, b(F.self_vel.x), b(F.self_vel.y),
                   b(F.facing_dir), b(F.mv.fx.SpecialHi.rotateModel), F.mv.fx.SpecialHi.travelFrames,
                   F.mv.fx.SpecialHi.unk, F.mv.fx.SpecialHi.unk2, (int) F.x1968_jumpsUsed, g_ms);
            printf("%d r %08x\n", i, b(g_rotx));
        }
    }
    return 0;
}
