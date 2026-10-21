/* C reference for src/tu/ftwalkcommon.rs: all of src/melee/ft/ftwalkcommon.c plus the
 * ftcommon.c ground-accel helpers it calls, cut verbatim into tu_ftwalkcommon_extract.inc by
 * gen/cref/tu_run.py; callees outside the TU are logging stubs. tests/tu_ftwalkcommon.rs
 * repeats the fill order and compares the log and an FNV hash of the whole native Fighter. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/ftcommon.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include "ft/ftanim.h"
#include "ft/ftwalkcommon.h"
#include <sysdolphin/baselib/gobj.h>

int printf(const char*, ...);
float ScriptGame_ModValue(int slot, int field, float base) { (void) slot; (void) field; return base; }
f32 Geno_FallLimit(Fighter* fp, f32 terminal) { (void) fp; return terminal; }
void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) line; (void) cond; for (;;) {} }
void OSReport(char* s, ...) { (void) s; }
ftCommonData* p_ftCommonData;
void ft_PlaySFX(Fighter* fp, enum_t id, int vol, int pan) { (void) fp; (void) id; (void) vol; (void) pan; }
static float g_fric, g_f484;
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
float ft_GetGroundFrictionMultiplier(Fighter* fp) { (void) fp; return g_fric; }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId msid, MotionFlags fl, float a, float r, float b, HSD_GObj* o)
{ (void) g; printf("CMS %d %u %08X %08X %08X %d\n", (int) msid, (u32) fl, B(a), B(r), B(b), o != NULL); }
void ftAnim_8006EBA4(Fighter_GObj* g) { (void) g; printf("EBA4\n"); }
void ftAnim_SetAnimRate(Fighter_GObj* g, float r) { (void) g; printf("RATE %08X\n", B(r)); }
float ftAnim_8006F484(Fighter_GObj* g) { (void) g; return g_f484; }
static void cb(Fighter_GObj* g, float f) { (void) g; printf("CB %08X\n", B(f)); }

#include "tu_ftwalkcommon_extract.inc"

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
static u32 fnv(const void* p, unsigned n)
{ const unsigned char* b = p; u32 h = 2166136261u; while (n--) { h ^= *b++; h *= 16777619u; } return h; }

static Fighter fp;
static ftCommonData cd;

int main(void)
{
    int op, k;
    HSD_GObj g;
    zero(&g, sizeof g);
    g.user_data = &fp;
    p_ftCommonData = &cd;
    for (op = 0; op < 6; op++) {
        for (k = 0; k < 500; k++) {
            int ret = 0;
            float a[11];
            int i;
            zero(&fp, sizeof fp);
            zero(&cd, sizeof cd);
            rs = 0x0DFBF8u + (u32) op * 7919u + (u32) k * 104729u;
            fp.gr_vel = rf(); fp.xE4_ground_accel_1 = rf();
            fp.input.lstick[0].x = rf();
            fp.facing_dir = rf();
            fp.coll_data.floor.normal.x = rf(); fp.coll_data.floor.normal.y = rf();
            fp.co_attrs.walk_max_vel = rf(); fp.co_attrs.walk_accel_mul = rf();
            fp.co_attrs.walk_accel_base = rf(); fp.co_attrs.ground_friction = rf();
            fp.cur_anim_frame = rf();
            fp.mv.co.walk.x0 = rf();
            fp.mv.co.walk.msid = (int) (rnd() % 64);
            fp.motion_id = fp.mv.co.walk.msid + (int) (rnd() % 5) - 1;
            fp.mv.co.walk.slow_anim_frame = rf(); fp.mv.co.walk.middle_anim_frame = rf();
            fp.mv.co.walk.fast_anim_frame = rf(); fp.mv.co.walk.slow_anim_rate = rf();
            fp.mv.co.walk.middle_anim_rate = rf(); fp.mv.co.walk.fast_anim_rate = rf();
            fp.mv.co.walk.accel_mul = rf();
            cd.walk_stick_threshold = rf(); cd.walk_middle_animation_stick_threshold = rf();
            cd.walk_fast_stick_threshold = rf(); cd.walk_accel_taper_gain = rf(); cd.x440 = rf();
            g_fric = (rnd() & 1) ? rf() : 1.0f;
            g_f484 = rf();
            for (i = 0; i < 11; i++) a[i] = rf();
            printf("OP %d %d\n", op, k);
            switch (op) {
            case 0: ret = ftWalkCommon_GetWalkType(&g); break;
            case 1: ret = ftWalkCommon_800DFC70(&g); break;
            case 2: { int m = (int) (rnd() % 64); u32 fl = rnd(); ftWalkCommon_800DFCA4(&g, m, fl, a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7]); } break;
            case 3: ftWalkCommon_800DFDDC(&g); break;
            case 4:
                /* the C asserts (spins) on an unknown walk type; keep to the reachable cases */
                if (fp.motion_id == fp.mv.co.walk.msid + ftWalkCommon_GetWalkType(&g)) fp.motion_id += 1;
                ftWalkCommon_800DFEC8(&g, cb); break;
            case 5: ftWalkCommon_800E0060(&g); break;
            }
            printf("R %d %08X\n", ret, fnv(&fp, sizeof fp));
        }
    }
    return 0;
}
