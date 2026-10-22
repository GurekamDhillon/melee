/* C reference for src/tu/ftseakspecialhi.rs: the state functions of Sheik's Up B that touch only
 * the fighter, the motion-state calls and the anim queries: ftSk_SpecialHi_80113EAC and
 * ftSk_SpecialHi_80113F68 (with their static inlines), ftSk_SpecialHi_801137C8, and the two
 * travel timers ftSk_SpecialHiStart_1_Anim / ftSk_SpecialAirHiStart_1_Anim. Those function bodies
 * are cut verbatim from ftseakspecialhi.c by tu_shard441.py. Calls outside the TU are recorders
 * both sides share (Fighter_ChangeMotionState, ftAnim_8006EBA4, ftAnim_IsFramesRemaining,
 * ftCommon_8007D7FC, ftColl_8007B62C). fn_80113038 is an identity stand-in: only its address is
 * compared (the accessory callback slot), its body is not part of this oracle. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include "ft/kinds/ftSeak/types.h"
#include "ft/kinds/ftSeak/forward.h"
#include "script_fighter_caps.h"
#include <math.h>
#include <stdio.h>

static int g_nchg;
static int g_msid;
static u32 g_flags;
static int g_anim_cnt;
static int g_d7fc_cnt;
static int g_frames;
static int g_coll_s;

void Fighter_ChangeMotionState(Fighter_GObj* g, int msid, u32 flags, f32 a, f32 b, f32 c, Fighter_GObj* d)
{
    (void) g; (void) a; (void) b; (void) c; (void) d;
    g_nchg++;
    g_msid = msid;
    g_flags = flags;
}
void ftAnim_8006EBA4(Fighter_GObj* g) { (void) g; g_anim_cnt++; }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { (void) g; return g_frames != 0; }
void ftCommon_8007D7FC(Fighter* fp) { (void) fp; g_d7fc_cnt++; }
void ftColl_8007B62C(Fighter_GObj* g, s32 s) { (void) g; g_coll_s = s; }
void fn_80113038(HSD_GObj* gobj) { (void) gobj; }

#include "tu_ftseakspecialhi_extract.inc"

static Fighter F;
static HSD_GObj G;
static ftSeakAttributes DA;
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
        int mode = (int) (rnd() % 5);
        F.self_vel.x = rf();
        F.self_vel.y = rf();
        F.gr_vel = rf();
        F.mv.co.walk.fast_anim_frame = rf();
        F.mv.co.common.x14 = rf();
        F.mv.co.common.x18 = rf();
        F.mv.sk.specialn.x0 = (int) (rnd() % 7) - 2;
        F.mv.sk.specialhi.x0 = (int) (rnd() % 7) - 2;
        F.mv.sk.specialhi.xC = (int) (rnd() % 9);
        F.mv.sk.specialhi.vel.x = rf();
        F.mv.sk.specialhi.vel.y = rf();
        F.invisible = (int) (rnd() & 1);
        F.x2223_b4 = (int) (rnd() & 1);
        F.x1968_jumpsUsed = (u8) (rnd() % 4);
        F.cur_anim_frame = rf() * 40.0f;
        DA.x54 = rf();
        DA.x38 = (int) (rnd() % 5);
        g_frames = (int) (rnd() & 1);
        F.accessory4_cb = (rnd() & 1) ? fn_80113038 : NULL;
        g_nchg = 0;
        g_anim_cnt = 0;
        g_d7fc_cnt = 0;
        g_msid = -1;
        g_flags = 0;
        switch (mode) {
        case 0: ftSk_SpecialHi_80113EAC(&G); break;
        case 1: ftSk_SpecialHi_80113F68(&G); break;
        case 2: ftSk_SpecialHi_801137C8(&G); break;
        case 3: ftSk_SpecialHiStart_1_Anim(&G); break;
        default: ftSk_SpecialAirHiStart_1_Anim(&G); break;
        }
        printf("%d %d %d %d %08x %d %d %d %d %d %d %d %d %08x %08x %08x %08x %08x %08x %08x %08x\n",
               i, mode, g_nchg, g_msid, g_flags, g_anim_cnt, g_d7fc_cnt,
               F.accessory4_cb == fn_80113038 ? 1 : 0,
               F.invisible, F.x2223_b4, F.mv.sk.specialhi.x0, F.mv.sk.specialhi.xC,
               F.mv.sk.specialn.x0,
               b(F.self_vel.x), b(F.self_vel.y), b(F.gr_vel), b(F.mv.co.walk.fast_anim_frame),
               b(F.mv.co.common.x14), b(F.mv.co.common.x18), b(F.mv.sk.specialhi.vel.x),
               b(F.mv.sk.specialhi.vel.y));
    }
    return 0;
}
