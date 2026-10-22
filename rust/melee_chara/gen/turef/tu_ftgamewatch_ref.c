/* C reference for src/tu/ftgamewatch*.rs: the state-machine logic of the Mr. Game & Watch TUs
 * (game_tus.txt lines 297-304), cut verbatim from the game's .c files by tu_shard297.py and
 * compiled with the shipping build's float contraction. Everything the cut functions call that
 * lives in another TU is a stand-in here that folds its arguments into LOG; tests/tu_ftgamewatch.rs
 * installs the same stand-ins as hooks on the Rust side and compares one line per case. */
#include <stddef.h>
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include "ft/ftcommon.h"
#include "ft/ftcoll.h"
#include "ft/ftparts.h"
#include "ft/ftanim.h"
#include "lb/lb_00B0.h"
#include "ft/kinds/ftGameWatch/types.h"
#include "ft/kinds/ftGameWatch/ftgamewatch.h"
#include "ft/kinds/ftGameWatch/ftgamewatchspeciallw.h"
#include "ft/kinds/ftGameWatch/ftgamewatchspecialhi.h"
#include "ft/kinds/ftGameWatch/ftgamewatchspecialn.h"
#include "ft/kinds/ftGameWatch/ftgamewatchspecials.h"
#include <sysdolphin/baselib/random.h>
#include <Runtime/platform.h>
#include <math.h>
#include <melee/it/kinds/itgamewatchchef.h>
#include <melee/it/kinds/itgamewatchjudge.h>
#include <melee/it/kinds/itgamewatchrescue.h>
#include <melee/it/kinds/itfoods.h>
#include <stdio.h>

ftCommonData* p_ftCommonData;
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }

/* ---- stand-ins: each folds what it was called with into LOG ---- */
static unsigned long long LOG;
static void lg(unsigned long long v) { LOG = (LOG ^ v) * 1099511628211ULL; }
static u32 bits(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
static u32 hs = 0x2545F491u; /* HSD_Randi's stand-in stream */
static u32 hnext(void) { hs ^= hs << 13; hs ^= hs >> 17; hs ^= hs << 5; return hs; }

void Fighter_ChangeMotionState(HSD_GObj* g, int msid, u32 flags, float a, float b, float c, HSD_GObj* d)
{
    (void) g; (void) d;
    lg(1); lg((u32) msid); lg(flags); lg(bits(a)); lg(bits(b)); lg(bits(c));
}
void ftAnim_8006EBA4(HSD_GObj* g) { (void) g; lg(2); }
void ftParts_80074B0C(HSD_GObj* g, int idx, int val) { (void) g; lg(3); lg((u32) idx); lg((u32) val); }
void ftCommon_UpdateFacing(Fighter* fp) { lg(4); fp->facing_dir = -fp->facing_dir; }
void ftPartSetRotY(Fighter* fp, int part, float v) { (void) fp; lg(5); lg((u32) part); lg(bits(v)); }
float ftCommon_GetModelScale(Fighter* fp) { return fp->x34_scale.y * 0.75f; }
Fighter_Part ftParts_GetBoneIndex(Fighter* fp, Fighter_Part part) { (void) fp; return part + 3; }
void lb_8000B1CC(HSD_JObj* j, Vec3* a, Vec3* out)
{
    (void) j;
    out->x = (a ? a->x : 0.5f) + 1.25f;
    out->y = (a ? a->y : 0.75f) + 2.5f;
    out->z = (a ? a->z : 0.25f) + 3.75f;
}
s32 HSD_Randi(s32 max_val) { return max_val > 0 ? (s32) (hnext() % (u32) max_val) : 0; }
void ftColl_CreateAbsorbHit(HSD_GObj* g, AbsorbDesc* d) { (void) g; (void) d; lg(6); }
Item_GObj* it_802C8038(Item_GObj* g, Vec3* pos, s32 part, s32 kind, float facing, float scale)
{
    (void) g; lg(7); lg(bits(pos->x)); lg(bits(pos->y)); lg(bits(pos->z)); lg((u32) part); lg((u32) kind);
    lg(bits(facing)); lg(bits(scale));
    return (HSD_GObj*) 0x1000;
}
HSD_GObj* it_802C7774(float facing, HSD_GObj* g, Vec3* pos, s32 part, u32 judge)
{
    (void) g; lg(8); lg(bits(facing)); lg(bits(pos->x)); lg(bits(pos->y)); lg(bits(pos->z)); lg((u32) part);
    lg((u32) judge);
    return (HSD_GObj*) 0x2000;
}
HSD_GObj* it_8028FAF4(Item_GObj* g, Vec3* pos) { (void) g; lg(9); lg(bits(pos->x)); lg(bits(pos->y)); lg(bits(pos->z)); return NULL; }
HSD_GObj* it_802C837C(Item_GObj* g, Vec3* pos, enum_t kind, u32 var, float facing)
{
    (void) g; lg(10); lg(bits(pos->x)); lg(bits(pos->y)); lg(bits(pos->z)); lg((u32) kind); lg((u32) var);
    lg(bits(facing));
    return NULL;
}
/* the file-scope constant of ftgamewatchspecialn.c the cut Loop functions use */
static u32 const transition_flags =
    Ft_MF_KeepColAnimHitStatus | Ft_MF_SkipHit | Ft_MF_SkipModel |
    Ft_MF_SkipMatAnim | Ft_MF_SkipColAnim | Ft_MF_UpdateCmd |
    Ft_MF_SkipItemVis | Ft_MF_Unk19 | Ft_MF_SkipModelPartVis |
    Ft_MF_SkipModelFlags | Ft_MF_Unk27;
/* callbacks the cut functions store: their addresses are all that matters */
void ftGw_Init_OnDamage(HSD_GObj* g) { (void) g; }
void ftGw_SpecialLw_ItemPanicSetup(HSD_GObj* g) { (void) g; }
void ftGw_SpecialLw_ItemPanicEnterHitlag(HSD_GObj* g) { (void) g; }
void ftGw_SpecialLw_ItemPanicExitHitlag(HSD_GObj* g) { (void) g; }
void ftGw_SpecialHi_ItemRescueEnterHitlag(HSD_GObj* g) { (void) g; }
void ftGw_SpecialHi_ItemRescueExitHitlag(HSD_GObj* g) { (void) g; }
void ftGw_SpecialS_ItemJudgementEnterHitlag(HSD_GObj* g) { (void) g; }
void ftGw_SpecialS_ItemJudgementExitHitlag(HSD_GObj* g) { (void) g; }

#include "tu_ftgamewatch_extract.inc"

static Fighter F;
static FighterBone PARTS[80];
static HSD_GObj G;
static ftGameWatchAttributes DA;
static ftCommonData CD;
static u32 s = 0x297304u;
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
/* non-negative, bounded, for damage multipliers */
static float rpos(void) { return (float) (rnd() % 3001) / 211.0f; }

static void state_line(int i, int kind)
{
    printf("%d %d %016llx %08x %08x %08x %08x %08x %08x %d %d %d %d\n", i, kind, LOG, F.cmd_vars[0], F.cmd_vars[1],
           bits(F.facing_dir), bits(F.lstick_angle), F.mv.gw.SpecialLw.isRelease | (F.mv.gw.SpecialLw.turnFrames << 8),
           F.mv.gw.SpecialN.isChefLoopDisable | (F.mv.gw.SpecialN.maxSausage << 8), F.u.gw.x2238_panicCharge,
           F.u.gw.x223C_panicDamage, F.u.gw.x2240_chefVar1, F.u.gw.x2244_chefVar2);
}

int main(void)
{
    int i;
    G.user_data = &F;
    F.parts = PARTS;
    F.dat_attrs = &DA;
    p_ftCommonData = &CD;
    for (i = 0; i < 3000; i++) {
        int k = i % 8;
        LOG = 1469598103934665603ULL;
        F.cmd_vars[0] = rnd() % 4;
        F.cmd_vars[1] = rnd() % 300;
        F.facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
        F.lstick_angle = 0.0f;
        F.x34_scale.y = rf();
        F.input.lstick[0].x = rf();
        F.input.held_buttons[0] = rnd() & HSD_PAD_B;
        F.input.pressed_buttons = rnd() & HSD_PAD_B;
        F.mv.gw.SpecialLw.isRelease = rnd() & 1;
        F.mv.gw.SpecialLw.turnFrames = (int) (rnd() % 5) - 1;
        F.mv.gw.SpecialN.isChefLoopDisable = rnd() & 1;
        F.mv.gw.SpecialN.maxSausage = rnd() % 6;
        F.u.gw.x2238_panicCharge = rnd() % 5;
        F.u.gw.x223C_panicDamage = (int) (rnd() % 700) - 20;
        F.u.gw.x2240_chefVar1 = rnd() % 5;
        F.u.gw.x2244_chefVar2 = rnd() % 5;
        F.u.gw.x222C_judgeVar1 = rnd() % 9;
        F.u.gw.x2230_judgeVar2 = rnd() % 9;
        F.accessory4_cb = NULL;
        DA.x74_GAMEWATCH_PANIC_DAMAGE_ADD = rpos();
        DA.x78_GAMEWATCH_PANIC_DAMAGE_MUL = rpos();
        DA.x7C_GAMEWATCH_PANIC_TURN_FRAMES = (float) (rnd() % 20);
        DA.x58_GAMEWATCH_RESCUE_STICK_RANGE = rf();
        DA.x5C_GAMEWATCH_RESCUE_ANGLE_UNK = rf();
        DA.x1C_GAMEWATCH_CHEF_MAX = (float) (rnd() % 7);
        DA.x18_GAMEWATCH_CHEF_LOOPFRAME = rf();
        {
            int j;
            for (j = 0; j < 9; j++) {
                DA.x34_GAMEWATCH_JUDGE_ROLL[j] = (int) (rnd() % 40) + 1;
            }
        }
        CD.horizontal_stick_deadzone = rpos() / 8.0f;
        F.u.gw.x226C_rescueGObj = NULL;
        F.motion_id = ftGw_MS_SpecialHi + (int) (rnd() % 2);
        switch (k) {
        case 0: ftGw_SpecialLwShoot_ReleaseOil(&G); break;
        case 1: ftGw_SpecialAirLwShoot_ReleaseOil(&G); break;
        case 2: ftGw_SpecialLw_IASA(&G); break;
        case 3: ftGw_SpecialAirLw_IASA(&G); break;
        case 4: ftGw_SpecialHi_ItemRescueSetup(&G); break;
        case 5: ftGw_SpecialAirHi_IASA(&G); break;
        case 6: ftGw_SpecialN_CreateSausage(&G); ftGw_SpecialN_IASA(&G); ftGw_SpecialAirN_IASA(&G); break;
        case 7: ftGw_SpecialS_GetRandomInt(&G); F.cmd_vars[1] = 1; ftGw_SpecialS_ItemJudgementSetup(&G); break;
        }
        lg(F.u.gw.x2264_judgementGObj != NULL);
        lg(F.u.gw.x226C_rescueGObj != NULL);
        lg(F.accessory4_cb != NULL);
        lg(F.u.gw.x222C_judgeVar1);
        lg(F.u.gw.x2230_judgeVar2);
        state_line(i, k);
    }
    return 0;
}
