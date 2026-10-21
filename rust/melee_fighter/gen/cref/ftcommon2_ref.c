/* C reference for src/ftcommon.rs, round 2: the field-logic functions of
 * src/melee/ft/ftcommon.c (jumps/ECB, grab mash, nudge setters, rumble triggers, item scale,
 * stick activity, ...), cut verbatim into ftcommon2_extract.inc by run.py and run on random
 * fighters. Callees outside group 3's ported set are stubs that fold their arguments into a
 * trace word `g_tr` and return values derived from their arguments; tests/ftcommon2_ref.rs
 * installs the same stubs as hooks and compares every output field bit for bit.
 * Built with the shipping contraction (run.py): the fused sites in ftCommon_8007ED50,
 * 8007EE0C, 8007FDA0 and 80080174 are fmadds here exactly as in the game. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/ftcommon.h"
#include "ft/fighter.h"
#include <sysdolphin/baselib/gobj.h>

#define FT_CAPS_MAX_JUMPS(fp) ((fp)->co_attrs.max_jumps)

static u32 g_tr;
static void T(u32 v) { g_tr = g_tr * 31u + v; }
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }

float ScriptGame_ModValue(int slot, int field, float base) { (void) slot; (void) field; return base; }
static float g_fric;
float ft_GetGroundFrictionMultiplier(Fighter* fp) { (void) fp; return g_fric; }
void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) line; (void) cond; for (;;) {} }
ftCommonData* p_ftCommonData;
struct Fighter_ShakeTable_t* Fighter_GrabMashShake;
HSD_GObj** HSD_GObjPLinkHead;

void Geno_GroundEdge(Fighter* fp, int landing) { (void) fp; T(0x100u + (u32) landing); }
void pl_8003FC44(int slot, int sub) { T(0x200u + (u32) slot * 2u + (u32) sub); }
void pl_800402D0(int slot, int sub, int r) { T(0x300u + (u32) slot * 4u + (u32) sub * 2u + (u32) r); }
void pl_80040460(int slot, int sub) { T(0x400u + (u32) slot * 2u + (u32) sub); }
bool Player_8003544C(s32 slot, bool sub) { return ((u32) slot + (u32) sub) & 1; }
void Player_UpdateJoystickCountByIndex(s32 slot, s32 sub) { T(0x500u + (u32) slot * 2u + (u32) sub); }
void lb_80014574(u8 port, int id, int a, int b) { T(0x600u + port); T((u32) id); T((u32) a); T((u32) b); }
void HSD_PadRumbleRemoveId(u8 port, int id) { T(0x700u + port); T((u32) id); }
void lb_800145C0(u8 slot) { T(0x800u + slot); }
bool un_803224DC(s32 spawn, f32 x, f32 kb) { T(0x900u + (u32) spawn); T(B(x)); T(B(kb)); return spawn & 1; }
void it_80294E78(Item_GObj* g, float f) { (void) g; T(0xA00u); T(B(f)); }
void it_8029A89C(Item_GObj* g, f32 f) { (void) g; T(0xB00u); T(B(f)); }
void it_8029A8F4(Item_GObj* g, Vec3* v) { (void) g; T(0xC00u); T(B(v->x)); T(B(v->y)); T(B(v->z)); }
Vec2* ftCo_80090690(Fighter* fp, Vec2* s)
{
    if (fp->kind & 1) { return NULL; }
    s->x = fp->cur_pos.x; s->y = fp->cur_pos.y; return s;
}
Vec2* ftCo_800DEEE8(Fighter* fp, Vec2* s)
{
    if (fp->kind & 2) { return NULL; }
    s->x = fp->xD4_unk_vel.x; s->y = fp->xD4_unk_vel.y; return s;
}

#include "ftcommon2_extract.inc"

int printf(const char*, ...);
static void zero(void* p, unsigned n) { unsigned char* b = p; while (n--) *b++ = 0; }

static u32 rs;
static u32 rnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }
static float rf(void)
{
    u32 k = rnd() % 16;
    if (k == 0) return 0.0f;
    if (k == 1) return -0.0f;
    if (k == 2) return 1.0f;
    if (k == 3) return -1.0f;
    return (float) ((int) (rnd() % 2001) - 1000) / 7.0f;
}

static Fighter fp;
static ftCommonData cd;
static struct Fighter_ShakeTable_t shake;
static Vec2 shake_v[256];
static float sandbag[2];
static HSD_GObj item_a, item_b;

#define NOPS 34

int main(void)
{
    int op, k, i;
    HSD_GObj g;
    zero(&g, sizeof g);
    g.user_data = &fp;
    p_ftCommonData = &cd;
    Fighter_GrabMashShake = &shake;
    shake.x0 = shake_v;
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < 300; k++) {
            float a, b, ret = 0;
            int ia, ib;
            Vec3 v = { 0, 0, 0 };
            zero(&fp, sizeof fp);
            zero(&cd, sizeof cd);
            g_tr = 0;
            rs = 0x2468Bu + (u32) op * 7919u + (u32) k * 104729u;
            fp.gobj = &g;
            fp.kind = rnd() % 34;
            fp.player_id = (u8) (rnd() % 6);
            fp.x618_player_id = (u8) (rnd() % 4);
            fp.x619_costume_id = (u8) (rnd() % 6);
            fp.x8_spawnNum = (int) (rnd() % 8);
            fp.facing_dir = rf();
            fp.gr_vel = rf(); fp.self_vel.x = rf(); fp.self_vel.y = rf();
            fp.cur_pos.x = rf(); fp.cur_pos.y = rf(); fp.cur_pos.z = rf();
            fp.xD4_unk_vel.x = rf(); fp.xD4_unk_vel.y = rf(); fp.xD4_unk_vel.z = rf();
            fp.x34_scale.x = rf(); fp.x34_scale.y = rf(); fp.x34_scale.z = rf();
            fp.x40 = rf();
            fp.xF4_ground_attacker_shield_kb_vel = rf();
            fp.coll_data.floor.normal.x = rf(); fp.coll_data.floor.normal.y = rf();
            fp.coll_data.x130_flags = rnd();
            fp.ground_or_air = (GroundOrAir) (rnd() & 1);
            fp.ecb_lock = (int) (rnd() % 20);
            fp.x1968_jumpsUsed = (u8) (rnd() % 4);
            fp.x1969_walljumpUsed = (u8) (rnd() % 2);
            fp.co_attrs.max_jumps = (int) (rnd() % 7);
            fp.co_attrs.model_scaling = rf();
            fp.co_attrs.x12C = rf();
            fp.co_attrs.x130.x = rf(); fp.co_attrs.x130.y = rf(); fp.co_attrs.x130.z = rf();
            fp.co_attrs.x13C = rf();
            fp.dmg.x18A4_knockbackMagnitude = rf();
            fp.dmg.x18c4_source_ply = (int) (rnd() % 7);
            fp.dmg.x18C8 = (int) rnd();
            fp.input.lstick[0].x = rf(); fp.input.lstick[0].y = rf();
            fp.input.triggers[0] = rf();
            fp.input.pressed_buttons = rnd() & rnd();
            fp.activity_timer.lstick.x = (u8) rnd(); fp.activity_timer.lstick.y = (u8) rnd();
            fp.activity_timer.trigger = (u8) rnd();
            fp.grab_timer = rf();
            fp.x1A50 = (s8) ((int) (rnd() % 3) - 1); fp.x1A51 = (s8) ((int) (rnd() % 3) - 1);
            fp.x1A52 = (u8) rnd(); fp.x1A53 = (u8) rnd();
            fp.x2018 = (int) ((int) (rnd() % 2001) - 1000);
            fp.x2020 = (s8) ((int) (rnd() % 3) - 1); fp.x2021 = (s8) ((int) (rnd() % 3) - 1);
            fp.x2024 = (int) ((int) (rnd() % 201) - 100);
            fp.smash_attrs.x2138_smashSinceHitbox = (rnd() & 1) ? -1.0f : rf();
            fp.x197C = (rnd() & 1) ? &item_a : NULL;
            fp.x1980 = (rnd() & 1) ? &item_b : NULL;
            fp.dat_attrs = sandbag;
            sandbag[0] = rf(); sandbag[1] = rf();
            fp.x594_b0 = rnd() & 1;
            fp.x2219_b0 = rnd() & 1;
            fp.x221F_b3 = rnd() & 1;
            fp.is_sub_fighter = rnd() & 1;
            fp.x2224_b2 = rnd() & 1; fp.x2224_b5 = rnd() & 1; fp.x2224_b6 = rnd() & 1;
            fp.x2225_b5 = rnd() & 1; fp.x2225_b6 = rnd() & 1;
            fp.x2227_b0 = rnd() & 1;
            cd.x138 = rf(); cd.x13C = rf(); cd.xEC = rf(); cd.xF0 = rf();
            cd.x308 = rf();
            cd.x708 = rf(); cd.x70C = rf(); cd.x710 = rf();
            cd.x7B8 = rf(); cd.x7BC = rf(); cd.x7C0 = rf();
            shake.x4 = (int) (rnd() % 300);
            for (i = 0; i < 256; i++) { shake_v[i].x = rf(); shake_v[i].y = rf(); }
            g_fric = rf();
            a = rf(); b = rf();
            ia = (int) (rnd() % 64) - 16;
            ib = (int) rnd();
            switch (op) {
            case 0: ftCommon_UnlockECB(&fp); break;
            case 1: ftCommon_8007D5D4(&fp); break;
            case 2: ftCommon_8007D60C(&fp); break;
            case 3: ftCommon_UseAllJumps(&fp); break;
            case 4: ftCommon_8007D780(&fp); break;
            case 5: ftCommon_InitGrab(&fp, ia & 3, a); break;
            case 6: ret = ftCommon_GrabMash(&fp, a); break;
            case 7: ftCommon_8007E2A4(&g); break;
            case 8: ftCommon_8007E2F4(&fp, (s16) ib); break;
            case 9: ftCommon_8007E2FC(&g); break;
            case 10: ftCommon_8007EBAC(&fp, (u32) ia, (u32) ib); break;
            case 11: ftCommon_8007ECD4(&fp, ia); break;
            case 12: ftCommon_8007ED2C(&fp); break;
            case 13: ftCommon_8007ED50(&fp, ia); break;
            case 14: ftCommon_8007EE0C(&fp, ia); break;
            case 15: ftCommon_8007EFC0(&fp, (u32) ib); break;
            case 16: ret = ftCommon_GetModelScale(&fp); break;
            case 17: ftCommon_8007F76C(&g); break;
            case 18: ftCommon_8007F7B4(&g); break;
            case 19: ret = ftCommon_8007F804(&fp) == fp.x44_mtx ? 1.0f : ftCommon_8007F804(&fp) == NULL ? 2.0f : 3.0f; break;
            case 20: ftCommon_8007F8B4(&fp, &v); break;
            case 21: ftCommon_8007FDA0(&g); break;
            case 22: ftCommon_8007FF74(&g); break;
            case 23: ret = ftCommon_8007FFD8(&fp, a); break;
            case 24: ret = ftCommon_80080144(&fp); break;
            case 25: ftCommon_80080174(&fp); break;
            case 26: ftCommon_8008021C(&g); break;
            case 27: ftCommon_8008031C(&g); break;
            case 28: ftCommon_80080460(&fp); ftCommon_80080474(&fp); T(fp.x2225_b6); ftCommon_80080460(&fp); break;
            case 29: ftCommon_80080484(&fp); break;
            case 30: ftCommon_800804A0(&fp, a); break;
            case 31: ret = ftCommon_800804EC(&fp); break;
            case 32: ftCommon_800804FC(&fp); break;
            case 33: fp.kind = Ft_Kind_Sandbag; ret = ftCommon_SandbagGetKnockbackDeaccelX(&fp);
                     T(B(ftCommon_SandbagGetKnockbackDeaccelY(&fp))); break;
            }
            if (fp.x1980 == NULL) { T(0xDEADu); }
            printf("%d %d %08X %08X %08X %08X %08X %08X %08X %08X %08X %08X %08X %08X %08X %08X "
                   "%08X %08X %08X %08X %d %d %d %d %d %d %d %d %d %d %d %d %d %d %d %08X %08X %08X\n",
                   op, k, B(ret), B(fp.gr_vel), B(fp.self_vel.x), B(fp.self_vel.y), B(fp.self_vel.z),
                   B(fp.x74_self_accel.y), B(fp.x98_atk_shield_kb.x), B(fp.x98_atk_shield_kb.y),
                   B(fp.x98_atk_shield_kb.z), B(fp.cur_pos.z), B(fp.grab_timer), B(fp.xE8_ground_accel_2),
                   B(fp.dmg.x18A4_knockbackMagnitude), B(fp.dmg.x18B8), B(fp.dmg.x18BC),
                   B(fp.smash_attrs.x2138_smashSinceHitbox), B(v.x), B(v.y), (int) fp.ground_or_air,
                   fp.ecb_lock, (int) fp.coll_data.x130_flags, (int) fp.x1968_jumpsUsed,
                   (int) fp.x1969_walljumpUsed, (int) fp.x1A50, (int) fp.x1A51, (int) fp.x1A52,
                   (int) fp.x1A53, fp.x2018, (int) fp.x2020, (int) fp.x2021, (int) fp.x1A6A,
                   (int) fp.x209A, fp.dmg.x18c4_source_ply * 1000 + (fp.dmg.x18C8 & 0xFF),
                   B(v.z), (u32) ((fp.x2224_b5 << 0) | (fp.x2224_b6 << 1) | (fp.x2225_b6 << 2) |
                   (fp.x2227_b0 << 3) | (fp.activity_timer.lstick.x << 8) |
                   (fp.activity_timer.lstick.y << 16) | ((u32) fp.activity_timer.trigger << 24)),
                   g_tr);
        }
    }
    return 0;
}
