/* C reference for src/ft084e.rs (group 3 round 12): all of src/melee/ft/ft_084E.c plus
 * ft_80084CB0 / ft_80084DB0 of ft_081B.c, ft_8008A1FC / ft_8008A244(+inline) of ft_08A1.c, and the ftcommon.c helpers they call, cut verbatim
 * into ft084e_extract.inc by run.py, run on random fighters. tests/ft084e_ref.rs repeats the
 * fill order and compares every written field bit for bit. MSL sinf/cosf come from
 * pc/gameworld/gekko_fp.c, which run.py links into every reference. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/ftcommon.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include "ft/ft_084E.h"
#include <math.h>
#include <sysdolphin/baselib/gobj.h>

#define FT_CAPS_MAX_JUMPS(fp) ((fp)->co_attrs.max_jumps)

float ScriptGame_ModValue(int slot, int field, float base) { (void) slot; (void) field; return base; }
f32 Geno_FallLimit(Fighter* fp, f32 terminal) { (void) fp; return terminal; }
static int g_sfx;
static float g_fric;
void ft_PlaySFX(Fighter* fp, enum_t id, int vol, int pan) { (void) fp; g_sfx += id + vol + pan; }
float ft_GetGroundFrictionMultiplier(Fighter* fp) { (void) fp; return g_fric; }
void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) line; (void) cond; for (;;) {} }
ftCommonData* p_ftCommonData;
struct Fighter_804D6520_t* Fighter_804D6520;
static int g_wait;
void ft_8008A2BC(Fighter_GObj* gobj) { (void) gobj; g_wait += 1; }

#include "ft084e_extract.inc"

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
static struct Fighter_804D6520_t bh;
static HSD_GObj other;

int main(void)
{
    int op, k;
    HSD_GObj g;
    zero(&g, sizeof g);
    g.user_data = &fp;
    p_ftCommonData = &cd;
    Fighter_804D6520 = &bh;
    for (op = 0; op < 19; op++) {
        for (k = 0; k < 400; k++) {
            float a, b, c;
            int ret = 0;
            ftCollisionBox box;
            zero(&fp, sizeof fp);
            zero(&cd, sizeof cd);
            zero(&bh, sizeof bh);
            zero(&box, sizeof box);
            g_sfx = 0;
            g_wait = 0;
            rs = 0x84E1Cu + (u32) op * 7919u + (u32) k * 104729u;
            fp.gr_vel = rf(); fp.xE4_ground_accel_1 = rf();
            fp.self_vel.x = rf(); fp.self_vel.y = rf(); fp.self_vel.z = rf();
            fp.x74_self_accel.x = rf(); fp.x74_self_accel.y = rf(); fp.x74_self_accel.z = rf();
            fp.input.lstick[0].x = rf(); fp.input.lstick[0].y = rf();
            fp.coll_data.floor.normal.x = rf(); fp.coll_data.floor.normal.y = rf();
            fp.co_attrs.gravity = rf();
            fp.co_attrs.terminal_velocity = rf();
            fp.co_attrs.fast_fall_velocity = rf();
            fp.co_attrs.aerial_friction = rf();
            fp.co_attrs.ground_friction = rf();
            fp.co_attrs.walk_max_vel = rf();
            fp.co_attrs.air_drift_max = rf();
            fp.co_attrs.air_drift_stick_mul = rf();
            fp.co_attrs.aerial_drift_base = rf();
            fp.co_attrs.air_max_horizontal_velocity = rf();
            fp.fall_fast = rnd() & 1;
            fp.active_timer.lstick.y = (u8) rnd();
            fp.facing_dir = rf();
            fp.facing_dir1 = rf();
            fp.x594_b0 = rnd() & 1;
            fp.x6A4_transNOffset.y = rf();
            fp.x6A4_transNOffset.z = rf();
            fp.lstick_angle = rf();
            fp.x197C = (rnd() & 1) ? &other : NULL;
            fp.coll_data.ecb.top.y = rf(); fp.coll_data.ecb.bottom.y = rf();
            fp.coll_data.ecb.right.x = rf(); fp.coll_data.ecb.right.y = rf();
            fp.coll_data.ecb.left.x = rf(); fp.coll_data.ecb.left.y = rf();
            cd.x88 = rf();
            cd.x8C = (int) (rnd() % 8);
            cd.friction_when_above_walk_speed = rf();
            bh.x14 = rf();
            cd.walk_stick_threshold = rf();
            g_fric = rf();
            a = rf(); b = rf(); c = rf();
            switch (op) {
            case 0: ft_80084E1C(&g, a, b, c); break;
            case 1: ft_80084EEC(&g); break;
            case 2: ft_80084F3C(&g); break;
            case 3: ft_80084FA8(&g); break;
            case 4: ft_80085004(&g); break;
            case 5: ft_80085030(&g, a, b); break;
            case 6: ft_80085088(&g); break;
            case 7: ft_800850B4(&g); break;
            case 8: ft_800850E0(&g, a, b); break;
            case 9: ft_80085134(&g); break;
            case 10: ft_80085154(&g); break;
            case 11: ft_800851C0(&g); break;
            case 12: ft_800851D0(&g); break;
            case 13: ft_80085204(&g); break;
            case 14: ft_80084CB0(&fp, &box); break;
            case 15: ft_80084DB0(&g); break;
            case 16: ret = ft_8008A1FC(&g); break;
            case 17: ret = ft_8008A244(&g); break;
            case 18: ret = ft_8008A244_inline(&g); break;
            }
            printf("%d %d %08X %08X %08X %08X %08X %08X %08X %08X %d %d %d %08X %08X %08X %08X %08X %08X %d %d\n",
                   op, k, B(fp.gr_vel), B(fp.xE4_ground_accel_1), B(fp.self_vel.x),
                   B(fp.self_vel.y), B(fp.self_vel.z), B(fp.x74_self_accel.x), B(fp.x74_self_accel.y),
                   B(fp.x74_self_accel.z), (int) fp.fall_fast, (int) fp.active_timer.lstick.y, g_sfx,
                   B(box.top), B(box.bottom), B(box.left.x), B(box.left.y), B(box.right.x), B(box.right.y), ret, g_wait);
        }
    }
    return 0;
}
