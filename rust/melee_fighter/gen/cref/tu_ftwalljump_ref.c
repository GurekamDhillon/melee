/* C reference for src/tu/ftwalljump.rs: src/melee/ft/ftwalljump.c cut verbatim into
 * tu_ftwalljump_extract.inc by gen/cref/tu_run_s57.py; mpGetSpeed / ftCo_800C1E64 are logging
 * stubs. tests/tu_ftwalljump.rs repeats the fill order and compares the log + Fighter hash. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include "ft/ftwalljump.h"
#include <melee/mp/mplib.h>
#include <sysdolphin/baselib/gobj.h>

int printf(const char*, ...);
ftCommonData* p_ftCommonData;
static float g_speed;
static int g_ok;
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
bool mpGetSpeed(int line, Vec3* pos, Vec3* speed)
{
    printf("SPD %d %08X %08X %08X\n", line, B(pos->x), B(pos->y), B(pos->z));
    speed->x = g_speed;
    return g_ok;
}
void ftCo_800C1E64(Fighter_GObj* g, int msid, int timer, int e, float d)
{ (void) g; printf("PW %d %d %d %08X\n", msid, timer, e, B(d)); }

#include "tu_ftwalljump_extract.inc"

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
static float side(void) { u32 k = rnd() % 4; return k == 0 ? -1.0f : k == 1 ? 1.0f : rf(); }
static u32 fnv(const void* p, unsigned n)
{ const unsigned char* b = p; u32 h = 2166136261u; while (n--) { h ^= *b++; h *= 16777619u; } return h; }

static Fighter fp;
static ftCommonData cd;

int main(void)
{
    int k;
    HSD_GObj g;
    zero(&g, sizeof g);
    g.user_data = &fp;
    p_ftCommonData = &cd;
    for (k = 0; k < 6000; k++) {
        int ret;
        zero(&fp, sizeof fp);
        zero(&cd, sizeof cd);
        rs = 0x8169Cu + (u32) k * 104729u;
        fp.can_walljump = (rnd() % 8) != 0;
        fp.coll_data.env_flags = (int) (rnd() & (Collide_RightWallHug | Collide_LeftWallHug | 0x10001));
        fp.wall_jump_input_timer = (u8) ((rnd() & 1) ? rnd() % 8 : 250 + rnd() % 6);
        fp.x2110_walljumpWallSide = side();
        fp.coll_data.ecb.left.x = rf(); fp.coll_data.ecb.left.y = rf();
        fp.coll_data.ecb.right.x = rf(); fp.coll_data.ecb.right.y = rf();
        fp.cur_pos.x = rf(); fp.cur_pos.y = rf(); fp.cur_pos.z = rf();
        fp.coll_data.right_facing_wall.index = (int) (rnd() % 100) - 1;
        fp.coll_data.left_facing_wall.index = (int) (rnd() % 100) - 1;
        fp.pos_delta.x = rf();
        fp.co_attrs.wall_jump_min_approach_speed = rf() / 50.0f;
        fp.input.lstick[0].x = rf() / 100.0f;
        fp.active_timer.lstick.x = (u8) (rnd() % 12);
        fp.x1969_walljumpUsed = (u8) ((rnd() & 1) ? 255 : rnd() % 4);
        cd.x768 = (float) (rnd() % 20); cd.x76C = rf() / 200.0f; cd.x770 = (float) (rnd() % 12);
        cd.x774 = (int) (rnd() % 30);
        g_speed = rf(); g_ok = rnd() & 1;
        printf("K %d\n", k);
        ret = ftWallJump_8008169C(&g);
        { /* bitfields are stored in GC bit order on the Rust side: print them, hash the rest */
            int cw = fp.can_walljump;
            fp.can_walljump = 0;
            printf("R %d %d %08X\n", ret, cw, fnv(&fp, sizeof fp));
        }
    }
    return 0;
}
