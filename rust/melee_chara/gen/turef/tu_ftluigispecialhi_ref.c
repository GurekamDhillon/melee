/* C reference for src/tu/ftluigispecialhi.rs: the whole game TU is #included verbatim; Fall and
 * CalcSelfAccel_DeaccelQuickAir are cut from ftcommon.c by tu_shard345.py (shipping contraction
 * via run.py). Everything else it calls is a recording stand-in. Prints, per case, the IASA's
 * momentum angle / rotation / facing update, the aerial Phys velocities and the Enter velocity. */
#include "ft/kinds/ftLuigi/ftluigispecialhi.c"
#include "tu_ftluigispecialhi_extract.inc"
#include <stdio.h>

float ScriptGame_ModValue(int slot, int field, float base) { (void) slot; (void) field; return base; }
f32 Geno_FallLimit(Fighter* fp, f32 terminal) { (void) fp; return terminal; }
ftCommonData* p_ftCommonData;
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }

static int FACING_N, ROT_N, ROT_PART;
static float ROT_VAL;
void ftPartSetRotY(Fighter* fp, int part, float v) { (void) fp; ROT_N++; ROT_PART = part; ROT_VAL = v; }
void ft_80085154(HSD_GObj* g) { (void) g; }
void ft_80084FA8(HSD_GObj* g) { (void) g; }
void ft_80083B68(HSD_GObj* g) { (void) g; }
void ft_800831CC(Fighter_GObj* g, bool (*a)(Fighter_GObj*, int), HSD_GObjEvent b) { (void) g; (void) a; (void) b; }
void ft_80084104(HSD_GObj* g) { (void) g; }
bool ftCo_80096CC8(HSD_GObj* g, int l) { (void) g; (void) l; return 0; }
void Fighter_ChangeMotionState(HSD_GObj* g, int m, u32 f, float a, float b, float c, HSD_GObj* d)
{ (void) g; (void) f; (void) a; (void) b; (void) c; (void) d; (void) m; }
void ftAnim_8006EBA4(HSD_GObj* g) { (void) g; }
int ftAnim_IsFramesRemaining(HSD_GObj* g) { (void) g; return 0; }
static float FS_MOB, FS_LAG;
void ftCo_80096900(HSD_GObj* g, int a, int b, int c, float mob, float lag) { (void) g; (void) a; (void) b; (void) c; FS_MOB = mob; FS_LAG = lag; }
void ftCo_LandingFallSpecial_Enter(HSD_GObj* g, bool a, float lag) { (void) g; (void) a; FS_LAG = lag; }

static u32 s = 0x1A9D0F3u;
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
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }

int main(void)
{
    static Fighter F;
    static HSD_GObj G;
    static ftLuigiAttributes A;
    static ftCommonData CD;
    int i;
    G.user_data = &F;
    F.dat_attrs = &A;
    p_ftCommonData = &CD;
    for (i = 0; i < 4000; i++) {
        /* IASA */
        F.input.lstick[0].x = rf();
        A.x5C_LUIGI_SUPERJUMP_MOMENTUM_STICK_RANGE = rf();
        A.x60_LUIGI_SUPERJUMP_ANGLE_DIFF = rf() * 60.0f;
        A.x58_LUIGI_SUPERJUMP_REVERSE_STICK_RANGE = rf();
        F.cmd_vars[0] = rnd() & 1;
        F.lstick_angle = rf();
        F.facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
        F.throw_flags_b3 = rnd() & 1;
        FACING_N = ROT_N = 0;
        ROT_VAL = 0.0f;
        ROT_PART = -1;
        ftLg_SpecialHi_IASA(&G);
        printf("I %d %08x %d %08x %d %08x %d\n", i,B(F.lstick_angle), (int) F.throw_flags_b3, B(F.facing_dir), ROT_N, B(ROT_VAL),
               ROT_PART);
        /* Phys */
        A.x6C_LUIGI_SUPERJUMP_VEL_Y = rf();
        A.x68_LUIGI_SUPERJUMP_GRAVITY_START = rf();
        F.co_attrs.terminal_velocity = rf();
        F.co_attrs.aerial_friction = rf();
        CD.aerial_friction_oob = rf();
        F.co_attrs.air_drift_max = rf();
        F.cmd_vars[0] = rnd() & 1;
        F.self_vel.x = rf(); F.self_vel.y = rf(); F.self_vel.z = rf();
        ftLg_SpecialAirHi_Phys(&G);
        printf("P %d %08x %08x %08x %08x\n", i, B(F.self_vel.x), B(F.self_vel.y), B(F.self_vel.z), B(F.x74_self_accel.x));
        /* Enter */
        A.x64_LUIGI_SUPERJUMP_VEL_X = rf();
        F.self_vel.x = rf(); F.self_vel.y = rf();
        ftLg_SpecialAirHi_Enter(&G);
        printf("E %d %08x %08x\n", i, B(F.self_vel.x), B(F.self_vel.y));
        /* Anim */
        A.x50_LUIGI_SUPERJUMP_FREEFALL_MOBILITY = rf();
        A.x54_LUIGI_SUPERJUMP_LANDING_LAG = rf();
        FS_MOB = FS_LAG = 123.0f;
        ftLg_SpecialHi_Anim(&G);
        printf("N %d %08x %08x\n", i, B(FS_MOB), B(FS_LAG));
    }
    return 0;
}
