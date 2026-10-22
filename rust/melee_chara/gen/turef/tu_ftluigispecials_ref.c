/* C reference for src/tu/ftluigispecials.rs: the whole game TU is #included verbatim; the group 3
 * movement helpers it calls (Fall, CalcSelfAccel_Deaccel) are cut from ftcommon.c by
 * tu_shard345.py (shipping contraction via run.py). Everything else it calls is a recording
 * stand-in. Prints one line per case:
 *   A  charge damage handed to ftColl_8007ABD0 by the four hitting Anim callbacks
 *   B  SetVars (charge frames, misfire, Randi argument)
 *   C  SFly_Enter launch velocities
 *   D  AirS2 / AirSEnd physics */
#include "ft/kinds/ftLuigi/ftluigispecials.c"
#include "tu_ftluigispecials_extract.inc"
#include <stdio.h>

float ScriptGame_ModValue(int slot, int field, float base) { (void) slot; (void) field; return base; }
f32 Geno_FallLimit(Fighter* fp, f32 terminal) { (void) fp; return terminal; }
ftCommonData* p_ftCommonData;
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }

static u32 DMG_N, DMG_SUM, DMG_LAST;
static int RANDI_ARG, RANDI_RET, FRAMES_LEFT, CMS_MSID;
static u32 CMS_FLAGS;
static float CMS_FRAME;
void ftColl_8007ABD0(HitCapsule* h, u32 dmg, HSD_GObj* g) { (void) h; (void) g; DMG_N++; DMG_SUM += dmg; DMG_LAST = dmg; }
int HSD_Randi(int n) { RANDI_ARG = n; return RANDI_RET; }
void Fighter_ChangeMotionState(HSD_GObj* g, int m, u32 f, float a, float b, float c, HSD_GObj* d)
{ (void) g; (void) b; (void) c; (void) d; CMS_MSID = m; CMS_FLAGS = f; CMS_FRAME = a; }
void ftAnim_8006EBA4(HSD_GObj* g) { (void) g; }
int ftAnim_IsFramesRemaining(HSD_GObj* g) { (void) g; return FRAMES_LEFT; }
void ftCommon_8007D5D4(Fighter* fp) { (void) fp; }
void ftCommon_8007D7FC(Fighter* fp) { (void) fp; }
void ftCommon_8007DB24(HSD_GObj* g) { (void) g; }
void ftCommon_8007F76C(HSD_GObj* g) { (void) g; }
void ft_8008A2BC(HSD_GObj* g) { (void) g; }
float ft_GetGroundFrictionMultiplier(Fighter* fp) { (void) fp; return 1.0f; }
void ft_80084F3C(HSD_GObj* g) { (void) g; }
void ft_80084EEC(HSD_GObj* g) { (void) g; }
int ft_80082708(HSD_GObj* g) { (void) g; return 1; }
int ft_80081D0C(HSD_GObj* g) { (void) g; return 0; }
void ftCo_Fall_Enter(HSD_GObj* g) { (void) g; }
void efLib_PauseAll(HSD_GObj* g) { (void) g; }
void efLib_ResumeAll(HSD_GObj* g) { (void) g; }
void* efSync_Spawn(s32 id, HSD_GObj* g, ...) { (void) id; (void) g; return 0; }

static u32 s = 0x3459A1u;
static u32 rnd(void) { s ^= s << 13; s ^= s >> 17; s ^= s << 5; return s; }
static float rf(void)
{
    switch (rnd() % 8) {
    case 0: return 0.0f;
    case 1: return -0.0f;
    case 2: return 1.0f;
    case 3: return -1.0f;
    default: return (float) ((int) (rnd() % 4001) - 2000) / 97.0f;
    }
}
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }

int main(void)
{
    static Fighter F;
    static HSD_GObj G;
    static ftLuigiAttributes A;
    int i;
    G.user_data = &F;
    F.dat_attrs = &A;
    for (i = 0; i < 3000; i++) {
        /* A */
        A.x14_LUIGI_GREENMISSILE_DAMAGE_SLOPE = rf();
        A.x10_LUIGI_GREENMISSILE_DAMAGE_TILT = rf();
        F.mv.lg.SpecialS.chargeFrames = (int) (rnd() % 400) - 50;
        if (i & 1) {
            /* land the sum on an integer N when the product is rounded first (unfused): the fused
             * sum then sits an ulp-fraction off N and truncates to N - 1 half of the time */
            float r;
            A.x14_LUIGI_GREENMISSILE_DAMAGE_SLOPE = (float) (rnd() % 100000) / (float) (1 + rnd() % 9973);
            F.mv.lg.SpecialS.chargeFrames = (int) (rnd() % 400) + 1;
            r = (float) ((double) F.mv.lg.SpecialS.chargeFrames * (double) A.x14_LUIGI_GREENMISSILE_DAMAGE_SLOPE);
            A.x10_LUIGI_GREENMISSILE_DAMAGE_TILT = (float) (50 + rnd() % 150) - r;
        }
        F.mv.lg.SpecialS.isMisfire = (rnd() % 3) == 0;
        F.x914[0].state = (rnd() % 4) == 0 ? 0 : HitCapsule_Enabled;
        F.cmd_vars[0] = 0;
        DMG_N = DMG_SUM = DMG_LAST = 0;
        ftLg_SpecialS_Anim(&G);
        ftLg_SpecialAirS_Anim(&G);
        ftLg_SpecialSMisfire_Anim(&G);
        ftLg_SpecialAirSMisfire_Anim(&G);
        printf("A %d %u %u %u\n", i, DMG_N, DMG_SUM, DMG_LAST);
        /* B */
        F.active_sticky.lstick.x = (u8) (rnd() % 256);
        A.x4_LUIGI_GREENMISSILE_SMASH = (float) (rnd() % 300) - 10.0f;
        A.x8_LUIGI_GREENMISSILE_CHARGE_RATE = rf();
        A.x44_LUIGI_GREENMISSILE_MISFIRE_CHANCE = rf();
        RANDI_RET = (rnd() % 3) == 0 ? 0 : 1;
        F.cmd_vars[0] = 7;
        F.x2070.x2070_int = 0;
        ftLg_SpecialS_SetVars(&G);
        printf("B %d %d %d %d %d %d\n", i, F.mv.lg.SpecialS.chargeFrames, F.mv.lg.SpecialS.isMisfire,
               RANDI_ARG, (int) F.x2070.count_thrown_items, (int) F.cmd_vars[0]);
        /* C */
        A.x28_LUIGI_GREENMISSILE_MUL_X = rf();
        A.x24_LUIGI_GREENMISSILE_VEL_X = rf();
        A.x2C_LUIGI_GREENMISSILE_VEL_Y = rf();
        A.xC_LUIGI_GREENMISSILE_MAX_CHARGE_FRAMES = rf();
        A.x48_LUIGI_GREENMISSILE_MISFIRE_VEL_X = rf();
        A.x4C_LUIGI_GREENMISSILE_MISFIRE_VEL_Y = rf();
        F.mv.lg.SpecialS.chargeFrames = (int) (rnd() % 400) - 20;
        F.mv.lg.SpecialS.isMisfire = (rnd() % 4) == 0;
        F.facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
        F.cur_anim_frame = rf();
        F.self_vel.x = 99.0f; F.self_vel.y = 99.0f;
        CMS_MSID = -1;
        ftLg_SpecialSFly_Enter(&G);
        printf("C %d %08x %08x %d %08x %08x\n", i, B(F.self_vel.x), B(F.self_vel.y), CMS_MSID, CMS_FLAGS,
               B(CMS_FRAME));
        /* D */
        A.x40_LUIGI_GREENMISSILE_GRAVITY_MUL = rf();
        A.x34_LUIGI_GREENMISSILE_GRAVITY_START = rf();
        A.x30_LUIGI_GREENMISSILE_MUL_Y = rf();
        A.x3C_LUIGI_GREENMISSILE_X_DECEL = rf();
        F.co_attrs.terminal_velocity = rf();
        F.cmd_vars[0] = rnd() & 1;
        F.self_vel.x = rf(); F.self_vel.y = rf();
        F.x74_self_accel.x = 5.0f;
        ftLg_SpecialAirS2_Phys(&G);
        printf("D %d %08x %08x %08x", i, B(F.self_vel.x), B(F.self_vel.y), B(F.x74_self_accel.x));
        F.self_vel.x = rf(); F.self_vel.y = rf();
        ftLg_SpecialAirSEnd_Phys(&G);
        printf(" %08x %08x %08x", B(F.self_vel.x), B(F.self_vel.y), B(F.x74_self_accel.x));
        /* E */
        A.x1C_LUIGI_GREENMISSILE_UNK2 = rf();
        F.coll_data.floor.normal.x = rf(); F.coll_data.floor.normal.y = rf();
        F.gr_vel = rf();
        ftLg_SpecialSStart_Phys(&G);
        printf(" %08x %08x %08x %08x", B(F.gr_vel), B(F.xE4_ground_accel_1), B(F.self_vel.x), B(F.self_vel.y));
        F.gr_vel = rf();
        ftLg_SpecialSEnd_Phys(&G);
        printf(" %08x %08x %08x %08x %08x\n", B(F.xE4_ground_accel_1), B(F.self_vel.x), B(F.self_vel.y),
               B(F.x74_self_accel.x), B(F.x74_self_accel.y));
    }
    return 0;
}
