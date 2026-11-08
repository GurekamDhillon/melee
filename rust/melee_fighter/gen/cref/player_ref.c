/* C reference for src/player.rs (port round 2): the player-table half of src/melee/pl/player.c,
 * cut verbatim into player_extract.inc by run.py (ftMapping_list + player_slots, the
 * Player_80031790..Player_80031900 block, Player_80031DA8..Player_80036978, and the TARGET_PC
 * m-ex setters + Player_ForgetEntities), run on random player tables. Calls out of player.c are
 * stubs that fold their arguments into a trace word `g_tr` and return values derived from it;
 * tests/player_ref.rs installs the same stubs as hooks and compares every printed line.
 * player.c has no fused (a*b+c) site (tools/rust-port/fma_sites.py); built with the shipping
 * contraction anyway (run.py). */
#include <melee/pl/player.h>
#include <melee/pl/types.h>
#include <melee/pl/forward.h>
#include <melee/ft/forward.h>
#include <melee/ft/types.h>
#include <melee/ft/ftlib.h>
#include <melee/ft/ftdemo.h>
#include <melee/ft/ft_0877.h>
#include <melee/ft/ft_0D4D.h>
#include <melee/ft/kinds/ftKirby/ftkirby.h>
#include <melee/gm/gm_unsplit.h>
#include <melee/if/ifstatus.h>
#include <sysdolphin/baselib/debug.h>
#include <sysdolphin/baselib/gobj.h>
#include <sysdolphin/baselib/objalloc.h>

static u32 g_tr;
static void T(u32 v) { g_tr = g_tr * 31u + v; }
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
static HSD_GObj ga, gb;
static u32 E(HSD_GObj* g) { return g == NULL ? 0u : g == &ga ? 1u : g == &gb ? 2u : 3u; }

void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) line; (void) cond; for (;;) {} }
void OSReport(char* fmt, ...) { (void) fmt; }
float ScriptGame_ModValue(int slot, int field, float base) { T(0x220u + (u32) slot * 2u + (u32) field); T(B(base)); return base; }

void ftLib_DisableInput(HSD_GObj* g) { T(0x10u + E(g)); }
void ftLib_EnableInput(HSD_GObj* g) { T(0x20u + E(g)); }
void ftKb_SpecialN_800EED50(s32 a, s32 b) { T(0x30u); T((u32) a); T((u32) b); }
bool ftLib_IsSleeping(HSD_GObj* g) { T(0x40u + E(g)); return (g_tr >> 3) & 1; }
void ftCo_800D4F24(Fighter_GObj* g, int i) { T(0x50u + E(g)); T((u32) i); }
void HSD_GObjFree(HSD_GObj* g) { T(0x60u + E(g)); }
void ftCo_800D4FF4(Fighter_GObj* g) { T(0x70u + E(g)); }
void ifStatus_802F6E1C(int s) { T(0x80u + (u32) s); }
bool ftLib_IsRebirth(HSD_GObj* g) { T(0x90u + E(g)); return (g_tr >> 5) & 1; }
void ftLib_80087140(HSD_GObj* g) { T(0xA0u + E(g)); }
void ftLib_SetPos(HSD_GObj* g, Vec3* v) { T(0xB0u + E(g)); T(B(v->x)); T(B(v->y)); T(B(v->z)); }
void ftDemo_SetFacingDirection(Fighter_GObj* g, float f) { T(0xC0u + E(g)); T(B(f)); }
float ftLib_800869F8(HSD_GObj* g) { T(0xD0u + E(g)); return (float) (int) (g_tr % 1000u) / 7.0f; }
void ftLib_SetScale(HSD_GObj* g, float f) { T(0xE0u + E(g)); T(B(f)); }
void ft_80087BEC(Fighter_GObj* g, int v) { T(0xF0u + E(g)); T((u32) v); }
void ft_80087BC0(Fighter_GObj* g, int v) { T(0x100u + E(g)); T((u32) v); }
int gm_8016C5C0(int s) { T(0x110u + (u32) s); return (int) (g_tr % 97u); }
void ftLib_SetPercent(HSD_GObj* g, s32 v) { T(0x120u + E(g)); T((u32) v); }
void ifStatus_802F6D10(s32 s) { T(0x130u + (u32) s); }
bool gm_8016B168(void) { T(0x140u); return (g_tr >> 4) & 1; }
void ifStatus_802F69C0(s32 a, s32 b) { T(0x150u); T((u32) a); T((u32) b); }
void ifStatus_802F6AF8(s32 s) { T(0x160u + (u32) s); }
u32 gm_GetFrameCount(void) { T(0x170u); return g_tr; }
void ifStatus_802F6C04(s32 s) { T(0x180u + (u32) s); }
bool ftLib_80086F4C(HSD_GObj* g) { T(0x190u + E(g)); return (g_tr >> 2) & 1; }
float ftLib_GetNameTagHeight(HSD_GObj* g) { T(0x1A0u + E(g)); return (float) (int) (g_tr % 777u) / 3.0f; }
FighterKind ftLib_GetKind(HSD_GObj* g) { T(0x1B0u + E(g)); return (FighterKind) (g_tr % 40u); }
s32 ftLib_GetLastAttackerSlot(HSD_GObj* g) { T(0x1C0u + E(g)); return (s32) (g_tr % 11u); }
void ft_80087AC0(Fighter_GObj* g, s32 v) { T(0x1D0u + E(g)); T((u32) v); }
void ftLib_80086A4C(HSD_GObj* g, float f) { T(0x1E0u + E(g)); T(B(f)); }
void ft_80087BAC(Fighter_GObj* g, s32 v) { T(0x1F0u + E(g)); T((u32) v); }
bool ftLib_IsCameraSubjectInBounds(HSD_GObj* g) { T(0x200u + E(g)); return (g_tr >> 6) & 1; }
void ftLib_GetCameraSubjectBonePos(HSD_GObj* g, Vec3* v) { T(0x210u + E(g)); v->x = (float) (int) (g_tr % 100u); v->y = -1.0f; }

#include "player_extract.inc"

int printf(const char*, ...);
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
static void cb(s32 a, s32 b) { T(0x230u); T((u32) a); T((u32) b); }
static void sf(s32 a) { (void) a; }
static HSD_GObj* rent(void) { u32 k = rnd() % 3; return k == 0 ? NULL : k == 1 ? &ga : &gb; }

static void init_slot(StaticPlayer* p)
{
    int i;
    p->player_state = (s32) (rnd() % 4);
    p->ckind = (CharacterKind) (rnd() % 40);
    p->pkind = (Gm_PKind) (rnd() % 4);
    p->transformed[0] = (u8) (rnd() % 2);
    p->transformed[1] = (u8) (rnd() % 2);
    for (i = 0; i < 4; i++) {
        p->player_poses.byIndex[i].x = rf();
        p->player_poses.byIndex[i].y = rf();
        p->player_poses.byIndex[i].z = rf();
    }
    p->facing_direction = rf();
    p->costume_id = (u8) rnd(); p->unk45 = (u8) rnd(); p->controller_index = (s8) rnd();
    p->team = (u8) (rnd() % 3); p->player_id = (u8) rnd(); p->cpu_level = (u8) rnd();
    p->cpu_type = (u8) rnd(); p->handicap = (u8) rnd(); p->unk4C = (s8) rnd();
    p->unk4D = (s8) rnd(); p->unk4E = (s8) rnd();
    p->unk50 = rf(); p->attack_ratio = rf(); p->defense_ratio = rf(); p->model_scale = rf();
    for (i = 0; i < 4; i++) p->staminas.byIndex[i] = (s16) rnd();
    for (i = 0; i < 2; i++) p->falls[i] = (s32) rnd();
    for (i = 0; i < 6; i++) p->kos_by_player[i] = rnd() % 4 == 0 ? 0xFFFFFFFFu : rnd();
    p->match_frame_count = rnd() % 3 == 0 ? 0xFFFFFFFFu : rnd();
    p->self_destructs = rnd() % 4 == 0 ? 0xFFFF : (u16) rnd();
    p->stocks = (s8) (rnd() % 7 - 2);
    p->current_coins = (s32) rnd(); p->total_coins = (s32) rnd();
    p->unk98 = (s32) rnd(); p->unk9C = (s32) rnd();
    p->joystick_direction_input_count[0] = (s32) rnd();
    p->joystick_direction_input_count[1] = (s32) rnd();
    p->nametag_slot_id = (s32) rnd();
    p->flags.b0 = rnd() & 1; p->flags.b1 = rnd() & 1; p->flags.b2 = rnd() & 1; p->flags.b3 = rnd() & 1;
    p->flags.b4 = rnd() & 1; p->flags.is_metal = rnd() & 1; p->flags.b6 = rnd() & 1; p->flags.b7 = rnd() & 1;
    p->more_flags.b0 = rnd() & 1; p->more_flags.b1 = rnd() & 1; p->more_flags.b2 = rnd() & 1;
    p->more_flags.b3 = rnd() & 1; p->more_flags.b4 = rnd() & 1; p->more_flags.b5 = rnd() & 1;
    p->more_flags.b6 = rnd() & 3;
    p->flagsAE.b0 = rnd() & 1; p->flagsAE.b1 = rnd() & 1;
    p->player_entity[0] = rent();
    p->player_entity[1] = rent();
    p->struct_func = rnd() & 1 ? sf : NULL;
}

static void dump(int slot)
{
    StaticPlayer* p = &player_slots[slot];
    int i;
    printf("S%d %x %x %x %x %x", slot, (u32) p->player_state, (u32) p->ckind, (u32) p->pkind,
           p->transformed[0], p->transformed[1]);
    for (i = 0; i < 4; i++)
        printf(" %x %x %x", B(p->player_poses.byIndex[i].x), B(p->player_poses.byIndex[i].y), B(p->player_poses.byIndex[i].z));
    printf(" %x %x %x %x %x %x %x %x %x %x %x %x", B(p->facing_direction), p->costume_id, p->unk45,
           (u32) p->controller_index, p->team, p->player_id, p->cpu_level, p->cpu_type, p->handicap,
           (u32) p->unk4C, (u32) p->unk4D, (u32) p->unk4E);
    printf(" %x %x %x %x", B(p->unk50), B(p->attack_ratio), B(p->defense_ratio), B(p->model_scale));
    for (i = 0; i < 4; i++) printf(" %x", (u32) p->staminas.byIndex[i]);
    printf(" %x %x", (u32) p->falls[0], (u32) p->falls[1]);
    for (i = 0; i < 6; i++) printf(" %x", p->kos_by_player[i]);
    printf(" %x %x %x %x %x %x %x %x %x %x", p->match_frame_count, p->self_destructs, (u32) p->stocks,
           (u32) p->current_coins, (u32) p->total_coins, (u32) p->unk98, (u32) p->unk9C,
           (u32) p->joystick_direction_input_count[0], (u32) p->joystick_direction_input_count[1],
           (u32) p->nametag_slot_id);
    printf(" f%d%d%d%d%d%d%d%d m%d%d%d%d%d%d%d a%d%d", p->flags.b0, p->flags.b1, p->flags.b2, p->flags.b3,
           p->flags.b4, p->flags.is_metal, p->flags.b6, p->flags.b7, p->more_flags.b0, p->more_flags.b1,
           p->more_flags.b2, p->more_flags.b3, p->more_flags.b4, p->more_flags.b5, p->more_flags.b6,
           p->flagsAE.b0, p->flagsAE.b1);
    printf(" e%x%x %d\n", E(p->player_entity[0]), E(p->player_entity[1]), p->struct_func != NULL);
}

#define NOPS 144

int main(void)
{
    int k, step, i;
    static ftMapping init_map[ChKind_Cap];
    for (i = 0; i < ChKind_Cap; i++) init_map[i] = ftMapping_list[i];
    for (k = 0; k < 200; k++) {
        rs = 0x5EED1u + (u32) k * 2654435761u;
        g_tr = 0;
        for (i = 0; i < ChKind_Cap; i++) ftMapping_list[i] = init_map[i];
        for (i = 0; i < 6; i++) init_slot(&player_slots[i]);
        for (step = 0; step < 60; step++) {
            int slot = (int) (rnd() % 6);
            u32 op = rnd() % NOPS;
            u32 ret = 0;
            Vec3 v;
            v.x = rf(); v.y = rf(); v.z = rf();
            switch (op) {
            case 0: Player_80031790(slot); break;
            case 1: Player_80031848(slot); break;
            case 2: Player_80031900(); break;
            case 3: ret = Player_800319C4(slot, (bool) (rnd() % 3)); break;
            case 4: { s32 a = (s32) rnd(); Player_80031DA8(a, (s32) rnd()); break; }
            case 5: Player_80031DC8(cb); break;
            case 6: Player_80031EBC(slot); break;
            case 7: Player_80031FB0(slot, (s32) (rnd() % 2)); break;
            case 8: Player_80032070(slot, (bool) (rnd() % 2)); break;
            case 9: ret = Player_8003219C(slot); break;
            case 10: ret = Player_8003221C(slot); break;
            case 11: ret = (u32) Player_GetPlayerState(slot); break;
            case 12: ret = (u32) Player_GetPlayerCharacter(slot); break;
            case 13: Player_SetPlayerCharacter(slot, (CharacterKind) (rnd() % 40)); break;
            case 14: ret = (u32) Player_8003248C(slot, (bool) (rnd() % 3)); break;
            case 15: { CharacterKind c = (CharacterKind) (rnd() % 40); ret = (u32) Player_800325C8(c, (bool) (rnd() % 3)); break; }
            case 16: ret = (u32) Player_80032610(slot, (bool) (rnd() % 3)); break;
            case 17: Player_LoadPlayerCoords(slot, &v); break;
            case 18: Player_80032768(slot, &v); break;
            case 19: Player_80032828(slot, (s32) (rnd() % 2), &v); break;
            case 20: Player_800328D4(slot, &v); break;
            case 21: Player_80032A04(slot, &v); break;
            case 22: Player_SetPlayerAndEntityFacingDirection(slot, rf()); break;
            case 23: ret = B(Player_80032BB0(slot)); break;
            case 24: Player_SetScale(slot, rf()); break;
            case 25: Player_GetSpawnPlatformPos(slot, &v); break;
            case 26: Player_SetSpawnPlatformPos(slot, &v); break;
            case 27: Player_GetSomePos(slot, &v); break;
            case 28: Player_SetSomePos(slot, &v); break;
            case 29: ret = (u32) Player_80032F30(slot); break;
            case 30: Player_80032FA4(slot, (s32) rnd()); break;
            case 31: ret = B(Player_GetFacingDirection(slot)); break;
            case 32: Player_SetFacingDirection(slot, rf()); break;
            case 33: { bool b = (bool) (rnd() % 3); Player_SetFacingDirectionConditional(slot, b, rf()); break; }
            case 34: ret = Player_GetCostumeId(slot); break;
            case 35: Player_SetCostumeId(slot, (int) rnd()); break;
            case 36: ret = Player_GetControllerIndex(slot); break;
            case 37: Player_SetControllerIndex(slot, (s8) rnd()); break;
            case 38: ret = (u32) Player_GetTeam(slot); break;
            case 39: Player_SetTeam(slot, (s8) rnd()); break;
            case 40: ret = (u32) Player_GetPlayerId(slot); break;
            case 41: Player_SetPlayerId(slot, (int) rnd()); break;
            case 42: ret = (u32) Player_GetCpuLevel(slot); break;
            case 43: Player_SetPlayerAndEntityCpuLevel(slot, (int) rnd()); break;
            case 44: ret = (u32) Player_GetCpuType(slot); break;
            case 45: Player_SetPlayerAndEntityCpuType(slot, (int) rnd()); break;
            case 46: ret = (u32) Player_GetHandicap(slot); break;
            case 47: Player_SetHandicap(slot, (s8) rnd()); break;
            case 48: ret = B(Player_GetUnk50(slot)); break;
            case 49: ret = B(Player_GetAttackRatio(slot)); break;
            case 50: Player_SetAttackRatio(slot, rf()); break;
            case 51: ret = B(Player_GetDefenseRatio(slot)); break;
            case 52: Player_SetDefenseRatio(slot, rf()); break;
            case 53: ret = B(Player_GetModelScale(slot)); break;
            case 54: Player_SetModelScale(slot, rf()); break;
            case 55: ret = (u32) Player_80033BB8(slot); break;
            case 56: ret = (u32) Player_GetP1Stock(); break;
            case 57: Player_LoseStock(slot); break;
            case 58: ret = (u32) Player_GetCoins(slot); break;
            case 59: Player_SetCoins(slot, (int) rnd()); break;
            case 60: ret = (u32) Player_GetTotalCoins(slot); break;
            case 61: Player_SetTotalCoins(slot, (int) rnd()); break;
            case 62: ret = (u32) Player_GetUnk98(slot); break;
            case 63: Player_SetUnk98(slot, (s32) rnd()); break;
            case 64: ret = (u32) Player_GetUnk9C(slot); break;
            case 65: Player_SetUnk9C(slot, (s32) rnd()); break;
            case 66: ret = E(Player_GetEntity(slot)); break;
            case 67: ret = E(Player_GetEntityAtIndex(slot, (int) (rnd() % 2))); break;
            case 68: { s32 a = (s32) (rnd() % 2); Player_SwapTransformedStates(slot, a, (s32) (rnd() % 2)); break; }
            case 69: ret = (u32) Player_GetDamage(slot); break;
            case 70: Player_SetHUDDamage(slot, (s32) rnd()); break;
            case 71: { s32 a = (s32) (rnd() % 2); Player_SetHPByIndex(slot, a, (s32) rnd()); break; }
            case 72: ret = (u32) Player_GetOtherStamina(slot); break;
            case 73: Player_SetOtherStamina(slot, (s32) rnd()); break;
            case 74: ret = (u32) Player_GetRemainingHP(slot); break;
            case 75: ret = (u32) Player_GetRemainingHPByIndex(slot, (s32) (rnd() % 2)); break;
            case 76: ret = (u32) Player_GetMoreFlagsBit2(slot); break;
            case 77: Player_SetMoreFlagsBit2(slot, (u8) rnd()); break;
            case 78: ret = (u32) Player_GetMoreFlagsBit3(slot); break;
            case 79: Player_SetMoreFlagsBit3(slot, (u8) rnd()); break;
            case 80: ret = (u32) Player_GetMoreFlagsBit4(slot); break;
            case 81: Player_SetMoreFlagsBit4(slot, (u8) rnd()); break;
            case 82: ret = (u32) Player_GetMoreFlagsBit5(slot); break;
            case 83: Player_SetMoreFlagsBit5(slot, (u8) rnd()); break;
            case 84: ret = (u32) Player_GetMoreFlagsBit6(slot); break;
            case 85: Player_SetMoreFlagsBit6(slot, (u8) rnd()); break;
            case 86: ret = Player_GetFlagsAEBit0(slot); break;
            case 87: Player_SetFlagsAEBit0(slot, (u8) rnd()); break;
            case 88: ret = (u32) Player_GetFalls(slot); break;
            case 89: ret = (u32) Player_GetFallsByIndex(slot, (s32) (rnd() % 2)); break;
            case 90: Player_SetFalls(slot, (s32) rnd()); break;
            case 91: { s32 a = (s32) (rnd() % 2); Player_SetFallsByIndex(slot, a, (s32) rnd()); break; }
            case 92: ret = (u32) Player_GetKOsByPlayerIndex(slot, (int) (rnd() % 6)); break;
            case 93: { bool b = rnd() % 3 == 0; Player_UpdateKOsBySlot(slot, b, (int) (rnd() % 6)); break; }
            case 94: ret = Player_GetMatchFrameCount(slot); break;
            case 95: Player_UpdateMatchFrameCount(slot, rnd() % 3 == 0); break;
            case 96: ret = Player_GetSelfDestructs(slot); break;
            case 97: { u32 x = rnd() % 3 == 0 ? rnd() : rnd() % 0x10002u; Player_SetSelfDestructs(slot, x); break; }
            case 98: Player_IncSelfDestructs(slot, rnd() % 3 == 0); break;
            case 99: ret = (u32) Player_800353BC(slot); break;
            case 100: ret = (u32) Player_8003544C(slot, rnd() % 3 == 0); break;
            case 101: Player_SetFlagsBit0(slot, (bool) (rnd() % 3)); break;
            case 102: ret = Player_GetNametagSlotID(slot); break;
            case 103: Player_SetNametagSlotID(slot, (int) rnd()); break;
            case 104: ret = (u32) Player_GetFlagsBit1(slot); break;
            case 105: Player_SetFlagsBit1(slot); break;
            case 106: Player_UnsetFlagsBit1(slot); break;
            case 107: ret = (u32) Player_GetFlagsBit3(slot); break;
            case 108: Player_SetFlagsBit3(slot, (u8) rnd()); break;
            case 109: ret = (u32) Player_GetFlagsBit4(slot); break;
            case 110: ret = Player_GetFlagsBit5(slot); break;
            case 111: Player_SetFlagsBit5(slot, (u8) rnd()); break;
            case 112: ret = Player_GetFlagsBit6(slot); break;
            case 113: Player_SetFlagsBit6(slot, (u8) rnd()); break;
            case 114: ret = Player_GetFlagsBit7(slot); break;
            case 115: Player_SetFlagsBit7(slot, (u8) rnd()); break;
            case 116: ret = (u32) Player_GetMoreFlagsBit0(slot); break;
            case 117: ret = (u32) Player_GetMoreFlagsBit1(slot); break;
            case 118: Player_SetMoreFlagsBit1(slot, (u8) rnd()); break;
            case 119: ret = (u32) Player_GetUnk4D(slot); break;
            case 120: Player_SetUnk4D(slot, (s8) rnd()); break;
            case 121: ret = Player_GetFlagsAEBit1(slot); break;
            case 122: Player_SetFlagsAEBit1(slot, (u8) rnd()); break;
            case 123: ret = (u32) Player_GetUnk4C(slot); break;
            case 124: Player_SetUnk4C(slot, (u8) rnd()); break;
            case 125: ret = (u32) Player_80036058(slot); break;
            case 126: ret = B(Player_800360D8(slot)); break;
            case 127: Player_SetStructFunc(slot, rnd() % 2 ? (void*) sf : NULL); break;
            case 128: ret = (u32) ((char*) Player_GetUnk6A8Ptr(slot) - (char*) &player_slots[slot]); break;
            case 129: ret = (u32) ((char*) Player_GetStaleMoveTableIndexPtr2(slot) - (char*) &player_slots[slot]); break;
            case 130: ret = (u32) Player_80036394(slot); break;
            case 131: ret = (u32) Player_80036428(slot); break;
            case 132: Player_SetUnk45(slot, (int) rnd()); break;
            case 133: ret = Player_GetUnk45(slot); break;
            case 134: Player_UpdateJoystickCountByIndex(slot, (s32) (rnd() % 2)); break;
            case 135: ret = (u32) Player_GetJoystickCountByIndex(slot, (s32) (rnd() % 2)); break;
            case 136: Player_800366DC(slot, (s32) rnd()); break;
            case 137: Player_80036790(slot, rf()); break;
            case 138: Player_80036844(slot, (s32) rnd()); break;
            case 139: ret = (u32) Player_800368F8(slot); break;
            case 140: Player_80036978(slot, &v); break;
            case 141: { int c = (int) (rnd() % 40); Player_MexSetMapping(c, (int) (rnd() % 300)); break; }
            case 142: { int c = (int) (rnd() % 40); Player_MexSetPartner(c, (int) (rnd() % 300)); break; }
            case 143: Player_ForgetEntities(); break;
            }
            printf("%d %d %u %d %x %x %x %x %x\n", k, step, op, slot, ret, B(v.x), B(v.y), B(v.z), g_tr);
            dump(slot);
        }
        for (i = 0; i < 6; i++) dump(i);
        for (i = 0; i < 40; i++) printf("M%d %x %x %x\n", i, (u32) ftMapping_list[i].internal_id,
                                        (u32) ftMapping_list[i].extra_internal_id, (u32) ftMapping_list[i].has_transformation);
    }
    return 0;
}
