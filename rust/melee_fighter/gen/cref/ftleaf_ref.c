/* C reference for port round 9 (src/ftaction.rs, src/fighter.rs, src/ftanim.rs, src/ftcoll.rs,
 * the ftcommon3.rs atan2f angles): leaf functions cut verbatim into ftleaf_extract.inc by
 * run.py (ftaction.c skip/flag handlers, fighter.c spawn counter / hitlag flags / pos getters /
 * UCF pad ring + flick counter, ftanim.c, ftcoll.c, ftcommon.c stick angles, and lb/lbtrigf.c's
 * atan2f + TARGET_PC atanf, which the Rust takes from melee_runtime::lbtrigf). Built with the
 * shipping contraction (run.py). One fighter, random fields, one op per case; each line is
 * op, case, return bits, a hash of every field the ops write, and the command-pointer delta. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include <melee/lb/types.h>
#include <melee/lb/inlines.h>
#include <sysdolphin/baselib/gobj.h>
#include <sysdolphin/baselib/jobj.h>

#include <Runtime/platform.h>

ftCommonData* p_ftCommonData;
float MSL_TrigF_80400770[1];
float MSL_TrigF_80400774[1];
float lbRefract3_804D7DD4;
u32 Fighter_804D64F8;
#define g_spawnNumCounter Fighter_804D64F8
static int dmg_log0_idx;
static int dmg_log1_idx;
void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) line; (void) cond; for (;;) {} }

#include "ftleaf_extract.inc"

int printf(const char*, ...);
static void zero(void* p, unsigned n) { unsigned char* b = p; while (n--) *b++ = 0; }
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }

static u32 rs;
static u32 rnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }
static float rf(void)
{
    union { float f; u32 u; } c;
    switch (rnd() % 16) {
    case 0: return 0.0f;
    case 1: return -0.0f;
    case 2: return 1.0f;
    case 3: return -1.0f;
    case 4: c.u = rnd(); return c.f;
    case 5: return -0.609375f;
    case 6: c.u = 0x7FC00000u; return c.f;
    default: return (float) ((int) (rnd() % 2001) - 1000) / 777.0f;
    }
}

static Fighter F;
static HSD_GObj GO;
static ftCommonData CD;
static FighterBone BONES[4];
static HSD_JObj JO[8];
static CmdUnion CMDS[16];
static HitCapsule HB;

#define NOPS 50
typedef void (*Cmd)(Fighter_GObj*, CommandInfo*);
static Cmd CMDF[29] = {
    ftAction_800711DC, ftAction_800715EC, ftAction_8007168C, ftAction_800716F8, ftAction_80071774,
    ftAction_800717C8, ftAction_80071810, ftAction_80071CA4, ftAction_80071D30, ftAction_80071F0C,
    ftAction_800722C8, ftAction_800726C0, ftAction_80072E24, ftAction_80072FE0, ftAction_8007309C,
    ftAction_8007320C, ftAction_8007296C, ftAction_800729C4, ftAction_80072A4C, ftAction_80072AAC,
    ftAction_80072B04, ftAction_80072B84, ftAction_80072BE4, ftAction_80072C5C, ftAction_80073108,
    ftAction_80071908, ftAction_8007192C, ftAction_80071950, ftAction_80071974,
};

static u32 jid(HSD_JObj* j) { int i; for (i = 0; i < 8; i++) if (j == &JO[i]) return 1u + (u32) i; return j == NULL ? 0 : 99; }

static u32 state(void)
{
    u32 h = 0;
    int i, p;
#define H(v) (h = h * 31u + (u32) (v))
    H(F.throw_flags_b0); H(F.throw_flags_b1); H(F.throw_flags_b2); H(F.throw_flags_b3);
    H(F.allow_interrupt); H(F.x2219_b7); H(F.x221A_b1);
    for (i = 0; i < 5; i++) { H(F.x8B0[i].x10); H(F.x8B0[i].x11); }
    H(Fighter_804D64F8); H(dmg_log0_idx); H(dmg_log1_idx);
    for (p = 0; p < 4; p++) {
        for (i = 0; i < 4; i++) { H(ftUcf_Pad[p].x[i]); H(ftUcf_Pad[p].y[i]); }
        H(ftUcf_Pad[p].idx); H(ftUcf_Pad[p].counter);
    }
#undef H
    return h;
}

int main(void)
{
    int op, k, i, p;
    p_ftCommonData = &CD;
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < 200; k++) {
            u32 ret = 0;
            int delta = 0;
            CommandInfo cmd;
            Vec3 v;
            zero(&F, sizeof F); zero(&GO, sizeof GO); zero(&cmd, sizeof cmd);
            rs = 0x2468Au + (u32) op * 7919u + (u32) k * 104729u;
            GO.user_data = &F;
            F.gobj = &GO;
            F.throw_flags_b0 = rnd() & 1; F.throw_flags_b1 = rnd() & 1; F.throw_flags_b2 = rnd() & 1;
            F.throw_flags_b3 = rnd() & 1;
            F.allow_interrupt = rnd() & 1; F.x2219_b7 = rnd() & 1; F.x221A_b1 = rnd() & 1;
            for (i = 0; i < 5; i++) { F.x8B0[i].x10 = (s8) rnd(); F.x8B0[i].x11 = (s8) rnd(); }
            F.player_id = (u8) (rnd() % 6);
            F.is_sub_fighter = rnd() & 1;
            F.cur_pos.x = rf(); F.cur_pos.y = rf();
            F.input.lstick[0].x = rf(); F.input.lstick[0].y = rf();
            F.input.cstick[0].x = rf(); F.input.cstick[0].y = rf();
            if (rnd() % 3 == 0) { F.input.lstick[0].y = -(float) (rnd() % 1000) / 999.0f; }
            if (rnd() % 3 == 0) { F.input.lstick[0].x = (float) ((int) (rnd() % 161) - 80) / 80.0f; }
            F.active_timer.lstick.y = (u8) (rnd() % 4);
            for (i = 0; i < 4; i++) { BONES[i].joint = &JO[i]; BONES[i].x4_jobj2 = &JO[4 + i]; }
            F.parts = BONES;
            Fighter_804D64F8 = (rnd() % 4 == 0) ? 0xFFFFFFFFu - (rnd() % 2) : rnd();
            dmg_log0_idx = (int) rnd(); dmg_log1_idx = (int) rnd();
            for (p = 0; p < 4; p++) {
                for (i = 0; i < 4; i++) { ftUcf_Pad[p].x[i] = (s8) rnd(); ftUcf_Pad[p].y[i] = (s8) rnd(); }
                ftUcf_Pad[p].idx = (u8) ((rnd() % 3 == 0) ? rnd() : rnd() % 4);
                ftUcf_Pad[p].counter = (u8) ((rnd() % 2) ? 0 : rnd());
            }
            CD.x7A8 = rf();
            HB.coll_distance = rf();
            cmd.u = &CMDS[rnd() % 4];
            if (op < 29) {
                CmdUnion* before = cmd.u;
                CMDF[op](&GO, &cmd);
                delta = (int) (cmd.u - before);
            } else switch (op) {
            case 29: ret = Fighter_NewSpawn_80068E40(); ret = ret * 31u + Fighter_NewSpawn_80068E40(); break;
            case 30: Fighter_UnkSetFlag_8006CFBC(&GO); break;
            case 31: setBit(&GO); break;
            case 32: ret = B(Fighter_GetPosX(&F)); break;
            case 33: ret = B(Fighter_GetPosY(&F)); break;
            case 34: { int a = (int) (rnd() % 4), b = (int) (rnd() % 600) - 300, c = (int) (rnd() % 600) - 300; ftUcf_PadBufferPush(a, b, c); } break;
            case 35: p = (int) (rnd() % 4); i = (int) (rnd() % 2); ret = (u32) ftUcf_PadRaw(p, i, (int) (rnd() % 9) - 2); break;
            case 36: ret = (u32) ftUcf_FlickCounter((int) (rnd() % 4)); break;
            case 37: ret = (u32) ftUcf_RimUnits(rf()); break;
            case 38:
                /* half the cases: raw y moved by 43..45 against two entries earlier (the > 44 edge) */
                p = (int) (rnd() % 4);
                if (rnd() & 1) {
                    int i0 = ftUcf_Pad[p].idx & 3;
                    ftUcf_Pad[p].y[(i0 - 2) & 3] = (s8) (ftUcf_Pad[p].y[i0] - 43 - (int) (rnd() % 3));
                }
                ftUcf_FlickCount(&F, p);
                break;
            case 39: v.x = rf(); v.y = rf(); v.z = rf(); scale_inline(rf(), &v); ret = (B(v.x) * 31u + B(v.y)) * 31u + B(v.z); break;
            case 40: p = (int) (rnd() % 4); ret = jid(get_part_joint(&F, p, rnd() & 1)); break;
            case 41: ftAnim_8007077C(&GO); break;
            case 42: p = (int) (rnd() % 5); ftAnim_80070FB4(&GO, p, (s32) rnd()); break;
            case 43: ftColl_800765E0(); break;
            case 44: { float d = rf(); if (rnd() & 1) d = d * 0.01f; ret = (u32) getEnvDmg(d); } break;
            case 45: ret = inlineB1(&HB); break;
            case 46: ret = (u32) getUnkVal(&F, (int) rnd()); break;
            case 47: ret = B(ftCo_GetLStickAngle(&F)); break;
            case 48: ret = B(ftCo_GetCStickAngle(&F)); break;
            case 49: ret = B(ftCommon_8007D9D4(&F)); break;
            }
            printf("%d %d %08X %08X %d\n", op, k, ret, state(), delta);
        }
    }
    return 0;
}
