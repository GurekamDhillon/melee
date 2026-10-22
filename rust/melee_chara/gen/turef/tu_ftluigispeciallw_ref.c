/* C reference for src/tu/ftluigispeciallw.rs: the whole game TU is #included verbatim; the group 3
 * movement helpers it calls are cut from ftcommon.c and atan2f from lbtrigf.c by tu_shard345.py
 * (shipping contraction via run.py). Everything else it calls is a recording stand-in.
 * Prints, per case, the Tornado's physics, collision (rotation / tap hand-over) and Enter / Anim
 * results. */
#include "ft/kinds/ftLuigi/ftluigispeciallw.c"
void Geno_GroundEdge(Fighter* fp, int landing);
#include "tu_ftluigispeciallw_extract.inc"
#include <stdio.h>

float MSL_TrigF_80400770[1];
float MSL_TrigF_80400774[1];
float ScriptGame_ModValue(int slot, int field, float base) { (void) slot; (void) field; return base; }
f32 Geno_FallLimit(Fighter* fp, f32 terminal) { (void) fp; return terminal; }
ftCommonData* p_ftCommonData;
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }

static int CMS_N, CMS_MSID, ROT_N, FALL_N, FS_N, UNSET_N;
static u32 CMS_FLAGS;
static float CMS_FRAME, ROT_VAL, FS_LAG, FS_MOB;
static int FS_A, FS_B, FS_C;
static int HIT_G, HIT_A;
static int R_GROUND, R_AIR;
void Fighter_ChangeMotionState(HSD_GObj* g, int m, u32 f, float a, float b, float c, HSD_GObj* d)
{ (void) g; (void) b; (void) c; (void) d; CMS_N++; CMS_MSID = m; CMS_FLAGS = f; CMS_FRAME = a; }
void ftAnim_8006EBA4(HSD_GObj* g) { (void) g; }
static int FRAMES_LEFT;
int ftAnim_IsFramesRemaining(HSD_GObj* g) { (void) g; return FRAMES_LEFT; }
void ft_8008A2BC(HSD_GObj* g) { (void) g; }
void ftCo_Fall_Enter(HSD_GObj* g) { (void) g; FALL_N++; }
void ftCo_80096900(HSD_GObj* g, int a, int b, int c, float mob, float lag)
{ (void) g; FS_N++; FS_A = a; FS_B = b; FS_C = c; FS_MOB = mob; FS_LAG = lag; }
void Geno_GroundEdge(Fighter* fp, int l) { (void) fp; (void) l; }
void ftCommon_8007D7FC(Fighter* fp) { (void) fp; }
void ftPartSetRotX(Fighter* fp, int part, float v) { (void) fp; (void) part; ROT_N++; ROT_VAL = v; }
bool ft_80082888(HSD_GObj* g, ftCollisionBox* b) { (void) g; (void) b; return R_GROUND; }
bool ft_800824A0(HSD_GObj* g, ftCollisionBox* b) { (void) g; (void) b; return R_AIR; }
void efLib_PauseAll(HSD_GObj* g) { (void) g; }
void efLib_ResumeAll(HSD_GObj* g) { (void) g; }
void* efSync_Spawn(s32 id, HSD_GObj* g, ...) { (void) id; (void) g; return 0; }
float ft_GetGroundFrictionMultiplier(Fighter* fp) { (void) fp; return 1.0f; }

static u32 s = 0x2C5E19u;
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
    for (i = 0; i < 4000; i++) {
        /* ground Phys */
        A.x74_LUIGI_CYCLONE_MOMENTUM_X_GROUND = rf();
        A.x7C_LUIGI_CYCLONE_MOMENTUM_X_MUL_GROUND = rf();
        A.x84_LUIGI_CYCLONE_FRICTION_END = rf();
        A.x8C_LUIGI_CYCLONE_TAP_Y_VEL_MAX = rf();
        A.x90_LUIGI_CYCLONE_TAP_GRAVITY = rf();
        A.x78_LUIGI_CYCLONE_MOMENTUM_X_AIR = rf();
        F.mv.lg.SpecialLw.groundVelX = rf();
        F.cmd_vars[0] = rnd() & 1;
        F.cmd_vars[2] = rnd() & 1;
        F.input.lstick[0].x = rf();
        F.input.pressed_buttons = (rnd() & 1) ? HSD_PAD_B : 0;
        F.gr_vel = rf();
        F.self_vel.y = rf();
        F.coll_data.floor.normal.x = rf(); F.coll_data.floor.normal.y = rf();
        F.cur_anim_frame = rf();
        CMS_N = 0;
        ftLg_SpecialLw_Phys(&G);
        printf("G %d %08x %08x %08x %08x %08x %08x %d %d %08x %u\n", i, B(F.gr_vel), B(F.xE4_ground_accel_1), B(F.self_vel.x),
               B(F.self_vel.y), B(F.x74_self_accel.x), B(F.mv.lg.SpecialLw.groundVelX), CMS_N, CMS_N ? CMS_MSID : -1,
               B(CMS_FRAME), CMS_N ? CMS_FLAGS : 0);
        /* aerial Phys */
        A.x80_LUIGI_CYCLONE_MOMENTUM_X_MUL_AIR = rf();
        F.co_attrs.gravity = rf();
        F.co_attrs.terminal_velocity = rf();
        F.u.lg.x222C_cycloneCharge = rnd() & 1;
        F.cmd_vars[0] = rnd() & 1;
        F.cmd_vars[2] = rnd() & 1;
        F.self_vel.x = rf(); F.self_vel.y = rf();
        F.mv.lg.SpecialLw.groundVelX = rf();
        ftLg_SpecialAirLw_Phys(&G);
        printf("A %d %08x %08x %08x %08x\n", i, B(F.self_vel.x), B(F.self_vel.y), B(F.x74_self_accel.x),
               B(F.mv.lg.SpecialLw.groundVelX));
        /* Coll */
        R_GROUND = rnd() & 1;
        R_AIR = rnd() & 1;
        F.ground_or_air = (rnd() & 1) ? GA_Ground : GA_Air;
        F.cmd_vars[3] = rnd() & 1;
        F.mv.lg.SpecialLw.isUnkColl = rnd() & 1;
        F.facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
        F.coll_data.floor.normal.x = rf(); F.coll_data.floor.normal.y = rf();
        F.self_vel.y = rf();
        CMS_N = ROT_N = 0;
        ftLg_SpecialLw_Coll(&G);
        printf("C %d %d %d %08x %d %08x %d %08x\n", i, CMS_N, (int) F.mv.lg.SpecialLw.isUnkColl, B(F.self_vel.y), ROT_N,
               B(ROT_VAL), CMS_N ? CMS_MSID : -1, CMS_N ? CMS_FLAGS : 0);
        R_AIR = rnd() & 1;
        CMS_N = ROT_N = 0;
        F.u.lg.x222C_cycloneCharge = 7;
        ftLg_SpecialAirLw_Coll(&G);
        printf("D %d %d %d %08x %d %08x %d %08x %d\n", i, CMS_N, (int) F.mv.lg.SpecialLw.isUnkColl, B(F.self_vel.y), ROT_N,
               B(ROT_VAL), CMS_N ? CMS_MSID : -1, CMS_N ? CMS_FLAGS : 0, (int) F.u.lg.x222C_cycloneCharge);
        /* Enter */
        A.x70_LUIGI_CYCLONE_TAP_MOMENTUM = rf();
        A.x88_LUIGI_CYCLONE_UNK = (int) (rnd() % 20) - 5;
        F.u.lg.x222C_cycloneCharge = rnd() & 1;
        F.self_vel.x = rf(); F.self_vel.y = rf();
        ftLg_SpecialAirLw_Enter(&G);
        printf("E %d %08x %08x %d %d %u\n", i, B(F.self_vel.x), B(F.self_vel.y), (int) F.mv.lg.SpecialLw.unk,
               (int) F.cmd_vars[2], (u32) F.x2219_b0);
        F.self_vel.x = rf(); F.self_vel.y = rf();
        ftLg_SpecialLw_Enter(&G);
        printf("N %d %08x %08x\n", i, B(F.self_vel.x), B(F.self_vel.y));
        /* AirLw_Anim */
        A.x94_LUIGI_CYCLONE_LANDING_LAG = (rnd() % 3 == 0) ? 0 : (int) (rnd() % 30);
        F.cmd_vars[1] = rnd() & 1;
        FRAMES_LEFT = rnd() & 1;
        FS_N = FALL_N = 0;
        F.u.lg.x222C_cycloneCharge = 0;
        ftLg_SpecialAirLw_Anim(&G);
        printf("L %d %d %d %d %08x %08x %d\n", i, FS_N, FALL_N, (int) F.u.lg.x222C_cycloneCharge, B(FS_LAG), B(FS_MOB),
               (int) F.cmd_vars[1]);
    }
    return 0;
}
