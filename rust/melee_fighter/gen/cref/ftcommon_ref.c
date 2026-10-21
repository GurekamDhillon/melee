/* C reference for src/ftcommon.rs: the movement helpers of src/melee/ft/ftcommon.c, cut
 * verbatim into ftcommon_extract.inc by run.py, run on random fighters. tests/ftcommon_ref.rs
 * repeats the fill order and compares every output field bit for bit. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/ftcommon.h"
#include "ft/fighter.h"
#include <sysdolphin/baselib/gobj.h>

#define FT_CAPS_MAX_JUMPS(fp) ((fp)->co_attrs.max_jumps)

/* the callees outside the cut, as the vanilla game behaves (script/Geno add-ons pass through) */
float ScriptGame_ModValue(int slot, int field, float base) { (void) slot; (void) field; return base; }
f32 Geno_FallLimit(Fighter* fp, f32 terminal) { (void) fp; return terminal; }
static int g_sfx;
static float g_fric;
void ft_PlaySFX(Fighter* fp, enum_t id, int vol, int pan) { (void) fp; g_sfx += id + vol + pan; }
float ft_GetGroundFrictionMultiplier(Fighter* fp) { (void) fp; return g_fric; }
void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) line; (void) cond; for (;;) {} }
ftCommonData* p_ftCommonData;

#include "ftcommon_extract.inc"

int printf(const char*, ...);
static void zero(void* p, unsigned n) { unsigned char* b = p; while (n--) *b++ = 0; }

static u32 rs;
static u32 rnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }
static float rf(void)
{
    u32 k = rnd() % 16;
    if (k == 0) return 0.0f;
    if (k == 1) return -0.0f;
    return (float) ((int) (rnd() % 2001) - 1000) / 7.0f;
}
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }

static Fighter fp;
static ftCommonData cd;

int main(void)
{
    int op, k;
    HSD_GObj g;
    zero(&g, sizeof g);
    g.user_data = &fp;
    p_ftCommonData = &cd;
    for (op = 0; op < 33; op++) {
        for (k = 0; k < 200; k++) {
            float a, b, c, d, ret = 0;
            zero(&fp, sizeof fp);
            zero(&cd, sizeof cd);
            g_sfx = 0;
            rs = 0x12345u + (u32) op * 7919u + (u32) k * 104729u;
            fp.gr_vel = rf(); fp.xE4_ground_accel_1 = rf();
            fp.self_vel.x = rf(); fp.self_vel.y = rf(); fp.self_vel.z = rf();
            fp.x74_self_accel.x = rf(); fp.x74_self_accel.y = rf(); fp.x74_self_accel.z = rf();
            fp.xF0_ground_kb_vel = rf(); fp.xF4_ground_attacker_shield_kb_vel = rf();
            fp.x8c_kb_vel.x = rf(); fp.x8c_kb_vel.y = rf();
            fp.input.lstick[0].x = rf(); fp.input.lstick[0].y = rf();
            fp.coll_data.floor.normal.x = rf(); fp.coll_data.floor.normal.y = rf();
            fp.ground_or_air = (GroundOrAir) (rnd() & 1);
            fp.co_attrs.ground_max_horizontal_velocity = rf();
            fp.co_attrs.air_drift_max = rf();
            fp.co_attrs.aerial_friction = rf();
            fp.co_attrs.air_max_horizontal_velocity = rf();
            fp.co_attrs.air_drift_stick_mul = rf();
            fp.co_attrs.aerial_drift_base = rf();
            fp.co_attrs.gravity = rf();
            fp.co_attrs.terminal_velocity = rf();
            fp.co_attrs.fast_fall_velocity = rf();
            fp.fall_fast = rnd() & 1;
            fp.active_timer.lstick.y = (u8) rnd();
            fp.facing_dir = rf();
            cd.max_grounded_kb_on_landing = rf();
            cd.aerial_friction_oob = rf();
            cd.x88 = rf();
            cd.x8C = (int) (rnd() % 8);
            cd.horizontal_stick_deadzone = rf();
            cd.x198 = rf(); cd.x19C = rf(); cd.x1A0 = rf();
            g_fric = rf();
            a = rf(); b = rf(); c = rf(); d = rf();
            switch (op) {
            case 0: ftCommon_CalcGroundAccel_Deaccel(&fp, a); break;
            case 1: ftCommon_CalcGroundAccel_DashRun(&fp, a, b, c); break;
            case 2: ftCommon_CalcGroundAccel_AccelToVel(&fp, a, b, c); break;
            case 3: ftCommon_CalcGroundAccel_AccelToLStickX(&fp, a, b, c); break;
            case 4: ftCommon_SetSelfMovementFromGroundedMovement(&g); break;
            case 5: ftCommon_SetSelfMovementFromGroundedMovement_NoFriction(&g); break;
            case 6: ftCommon_ClampGroundVel(&fp, a); break;
            case 7: ftCommon_ApplyGroundedKnockbackFriction(&fp, a); break;
            case 8: ftCommon_SetGroundedKnockbackIfLanded(&fp); break;
            case 9: ret = ftCommon_SandbagKnockbackDeaccel(a, b); break;
            case 10: ftCommon_ApplyShieldKnockbackFriction(&fp, a); break;
            case 11: ftCommon_CalcSelfAccel_Deaccel(&fp, a); break;
            case 12: ftCommon_CalcSelfAccel_DeaccelAir(&fp); break;
            case 13: ret = ftCommon_CalcSelfAccel_DeaccelQuickAir(&fp); break;
            case 14: ret = ftCommon_CalcSelfAccel_DeaccelQuick(&fp, a); break;
            case 15: ftCommon_CalcSelfAccel_AccelToVelClamped(&fp, a, b, c); break;
            case 16: ftCommon_CalcSelfAccel_AccelToVelClampedFrom(&fp, a, b, c, d); break;
            case 17: ftCommon_CalcSelfAccel_Drift(&fp); break;
            case 18: ftCommon_CalcSelfAccel_DriftFrom(&fp, a); break;
            case 19: ftCommon_CalcSelfAccel_AccelToVel(&fp, a, b, c); break;
            case 20: ftCommon_CalcSelfAccel_DriftSimple(&fp, a, b, c); break;
            case 21: ftCommon_CalcSelfAccel_DriftSimple_NoFriction(&fp, a, b, c); break;
            case 22: ftCommon_ClampSelfVelX(&fp, a); break;
            case 23: ftCommon_ClampAirDrift(&fp); break;
            case 24: ftCommon_Fall(&fp, a, b); break;
            case 25: ftCommon_FallBasic(&fp); break;
            case 26: ftCommon_FallFast(&fp); break;
            case 27: ftCommon_ClampFallSpeed(&fp, a); break;
            case 28: ftCommon_Ascend(&fp, a, b); break;
            case 29: ret = ftCommon_CheckFallFast(&fp); break;
            case 30: ftCommon_UpdateFacing(&fp); break;
            case 31: ftCommon_8007DA24(&fp); break;
            case 32: { int dmg = (int) (rnd() % 40); FtMotionId ms = (FtMotionId) (37 + rnd() % 5); ret = ftCommon_CalcHitlag(dmg, ms, a); } break;
            }
            printf("%d %d %08X %08X %08X %08X %08X %08X %08X %08X %08X %08X %08X %08X %08X %08X %d %d %d\n",
                   op, k, B(ret), B(fp.gr_vel), B(fp.xE4_ground_accel_1), B(fp.self_vel.x),
                   B(fp.self_vel.y), B(fp.self_vel.z), B(fp.x74_self_accel.x), B(fp.x74_self_accel.y),
                   B(fp.x74_self_accel.z), B(fp.xF0_ground_kb_vel),
                   B(fp.xF4_ground_attacker_shield_kb_vel), B(fp.x8c_kb_vel.x), B(fp.x8c_kb_vel.y),
                   B(fp.facing_dir), (int) fp.fall_fast, (int) fp.active_timer.lstick.y, g_sfx);
        }
    }
    return 0;
}
