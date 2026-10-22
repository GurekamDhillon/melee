/* TU: src/melee/gm/gmvs.c */
/* HS: VsSceneController MatchEnd PlayerInitData StartMeleeData lbl_8046B488_t Vec */
/* STUBS: gm_1601_GetUnkData:manual gm_8016A1F8 Player_GetPlayerState:ret=small:3 lb_80019880 gm_GetCurrentGameMode:manual Mex_Enabled:ret=bit gm_Is1PMode:ret=bit */
/* STUBS: Ground_801C1DE4:manual Ground_801C1D98 Player_GetPlayerSlotType:manual lbTime_8000AF24 lbTime_8000AF74 grStadium_801D4150 Ground_801C1154 */
/* STUBS: grStadium_801D4084 grStadium_801D410C un_802FD404 ftLib_800868A4 Stage_802252E4 grStadium_801D4040 ifStatus_802F6EA4:manual un_802FD428 */
/* STUBS: Ground_801C1158 grStadium_801D40C8 grStadium_801D3B4C Player_GetEntity:ret=ptrbit Player_8003219C Player_GetStocks:manual */
/* STUBS: Player_GetPlayerId:manual ifStock_802F7EFC:ret=small:2 Player_LoseStock Player_SetStocks fn_8016719C gm_GetCurrentSceneIndex:ret=small:130 */
/* STUBS: Player_GetP1Stock:ret=small:3 fn_8016A1E4:ret=small:2 fn_801693A8:ret=small:2 Player_GetFlagsBit1:ret=small:2 Player_GetTeam:manual */
/* STUBS: Ground_801C1D84:ret=small:2 fn_8017EE40:manual fn_8017E8A4:manual fn_8018A364:manual pl_80039450 fn_80170110:ret=small:3 fn_8016FAD4 fn_8016F1F0 */
/* STUBS: gm_801A4BA8:ret=small:3 gm_80166378 ftLib_80086BE0:ret=small:6 fn_80171DC4 HSD_PadRumbleRemoveAll HSD_PadRumbleOn lbAudioAx_800236DC */
/* STUBS: lbAudioAx_80023694 lbAudioAx_80024C84 ifTime_FreeCountdown gm_80167858 Player_GetNametagSlotID Camera_RequestQuake gm_801B6320:manual */
/* STUBS: gm_GetAllStarData:manual gm_CKindToSelKind:ret=small:26 gmMainLib_8015D438:ret=pool:8:32 gmMainLib_8015D450 gmMainLib_8015D48C:ret=bit fn_80182B5C */
/* STUBS: lbAudioAx_80024E84:boolint lbAudioAx_80024030 ifAll_802F3394 ifTime_ShowTimers gm_801A0FEC gm_SetDbPauseFlag gm_EnablePlayerPauseCamera */
/* STUBS: HSD_PadRumblePauseAll Ui_RetailPause Ui_TakeUnpause:ret=m1:5 gm_ClearDbPauseFlag ifAll_802F33CC gm_801A10FC HSD_PadRumbleUnpauseAll */
/* STUBS: Ground_EnableMatchCamera gm_801A4624:ret=small:2 gm_801A46B8:ret=bit ifTime_IsTimerHidden:ret=bit lbAudioAx_800237A8 lbAudioAx_80023870 */
/* STUBS: lbAudioAx_80024D50 gm_801A4B60 fn_8016A4C8 fn_8016758C gm_GetDbPauseFlag:ret=bit gm_GetButtonsTriggered:manual gm_GetButtonsPressed:manual */
/* STUBS: ifStatus_802F7034:manual fn_8017DF90:ret=small:3 fn_8017DEC8:manual pl_80040688 gm_801A4B1C un_802FD668 ifAll_802F3A64 fn_80180630 */
/* STUBS: fn_80180AC0:ret=small:2 Player_SetSlottype Player_SetPlayerCharacter Player_SetCostumeId Player_SetControllerIndex Player_SetPlayerId */
/* STUBS: Netplay_Enabled:ret=bit Player_GetPlayerCharacter:ret=small:33 Player_SetHandicap Player_SetTeam Player_SetFlagsBit0 Player_SetNametagSlotID */
/* STUBS: Player_SetFlagsBit3 Player_SetUnk4C Player_SetPlayerAndEntityCpuType Player_SetPlayerAndEntityCpuLevel Player_SetHUDDamage */
/* STUBS: Player_SetMoreFlagsBit2 Player_SetOtherStamina Player_SetModelScale Player_SetAttackRatio Player_SetDefenseRatio Player_SetMoreFlagsBit1 */
/* STUBS: Player_SetFlagsBit5 Player_SetFlagsBit6 Player_SetFlagsBit7 Player_SetFlagsAEBit0 Player_SetMoreFlagsBit3 Player_SetMoreFlagsBit4 */
/* STUBS: Player_SetMoreFlagsBit5 Player_SetMoreFlagsBit6 Player_SetFacingDirection memzero:manual Camera_SetQuakeScale Player_InitAllPlayers fn_8016A09C */
/* STUBS: Player_GetCostumeId:ret=small:7 gm_GetNumCostumesForCKind:ret=small:7 Player_GetControllerIndex:ret=small:5 Player_GetFacingDirection */
/* STUBS: Player_80032610:ret=m1:7 Player_80031DA8 Player_80036DA4 Stage_80224DC8:ret=bit Player_80032768 Player_SetUnk45 fn_80160840:ret=small:5 */
/* STUBS: gm_80160854:ret=small:9 Player_80031AD0 fn_80169C54 fn_80169F50 Replay_Active:ret=bit Replay_ApplyMatch FighterControls_MatchBegin */
/* STUBS: Replay_RecordMatch db_Setup gm_SetDbPauseInputHandlers:manual;proto=void(Predicate,Predicate) gm_SetPreGObjProcCallback:manual db_RunEveryFrame */
/* STUBS: gm_801A4B50 Camera_Init Camera_Create lbRefract_800222A4 lb_8000FCDC efLib_Init efAsync_LoadSync Player_80036DD8 ftCo_800C06C0 */
/* STUBS: mpColl_80041C78 Ground_801C0378 Stage_802251E8 Item_80266F70 Item_80266FCC it_8026D018 lbAudioAx_8002785C Stage_8022524C Camera_80030730 */
/* STUBS: Ground_801C20D0 Stage_80225298 Camera_8002F3AC fn_801A1134 sfx_setupCrowdSFX ifAll_802F390C lbBgFlash_Init gm_801A4BC8:ret=ptrbit */
/* STUBS: fn_80171AD4 HSD_GObj_SetupProc:manual Stage_80225074 ifTime_CreateTimers ifStatus_802F665C fn_8017F294 Ground_801C5ABC:ret=bit */
/* STUBS: Script_OnePScripted:ret=bit gmMainLib_8015D00C gmVsMelee_EnterResults gmVsMelee_ExitResults gm_801623A4 fn_8018A000 Player_GetRemainingHP */
/* STUBS: Player_GetOtherStamina Player_80036D24 Player_80031848 ifStatus_802F6508 un_802FD28C ifStatus_802F6788 Player_80031EBC */
/* STUBS: gm_SetupRulesDefaults gm_SetupPlayerDefaults gm_RumbleEnabledForPlayer:ret=bit gmMainLib_8015DADC:ret=bit gmMainLib_8015DAB4 pl_80039418 */
/* STUBS: Stage_80224E64:manual Player_LoadPlayerCoords:manual */
/* C oracle for src/tu/gmvs.rs: the whole TU included verbatim (TARGET_PC build); every callee out of it is a recording stub and the game state
 * the TU reads (the scene controller `controller` it owns, the pads, the match results...) is synthetic. One line per case:
 * op case ret state-hash trace. The ops are generated from the OPS block below (opsgen.py). */
#include <dolphin/os.h>
/* the bus clock is guest memory (0x800000F8) in the game; the native oracle keeps it in a variable */
u32 BUSCLK_VAR = 162000000u;
#undef __OSBusClock
#define __OSBusClock BUSCLK_VAR
#include TU_SRC
#include "tu_common.h"
#include "hs_gmvs.h"
#include "stubs_gmvs.h"

void OSReport(char* fmt, ...) { (void) fmt; }

/* ---- globals the TU reads (defined elsewhere in the game) */
DbLKind DbLevel;
HSD_PadStatus HSD_PadCopyStatus[4];
HSD_PadStatus HSD_PadMasterStatus[4];
u32* HSD_RandSeedPtr;
GameModeState gm_Mode_Vs_States[10];
struct gmm_x0* gmMainLib_804D3EE0;
ResultsMatchInfo gmVsMelee_ResultsEnterData;
static u32 RANDSEED;
static struct gmm_x0 SAVEDATA;

/* OPS:
gmVs_GetSceneController() -> ptreq:controller [CTRL]
gmVs_GetSceneState() -> ptreq:controller [CTRL]
gm_GetStartMeleeRules() -> ptr [CTRL]
fn_8016AE60() -> ptreq:LBL [LBL]
gm_8016AE80() -> s32 [CTRL]
gm_8016AE94() -> f32 [CTRL]
gm_8016AEA4() -> u64 [CTRL]
gm_8016AEB8() -> s32 [CTRL]
gm_8016AEC8() -> s32 [CTRL]
gm_GetFrameCount() -> u32 [CTRL]
gm_8016AEEC() -> u32 [CTRL]
gm_8016AEFC() -> u16 [CTRL]
gm_8016AF0C() -> u16 [CTRL]
GetMatchTimer({((rnd() & 1) ? &PINT : (int*) 0)|if rnd() & 1 != 0 { addr_of_mut!(PINT) } else { null_mut() }}) -> bool [CTRL MISC]
gm_GetStKind() -> u16 [CTRL]
gm_8016B014() -> bool [CTRL]
gm_8016B094() -> bool [CTRL]
gm_8016B0B4() -> bool [CTRL]
gm_8016B0D4() -> s32 [CTRL]
gm_8016B0E8() -> s32 [CTRL]
gm_8016B0FC() -> s32 [CTRL]
gm_8016B110() -> s32 [CTRL]
gm_8016B124() -> s32 [CTRL]
fn_8016B138() -> void [CTRL]
gm_8016B14C() -> bool [CTRL]
gm_8016B168() -> bool [CTRL]
gm_8016B184() -> bool [CTRL]
gm_8016B1A8() -> bool [CTRL]
gm_8016B1C4() -> s32 [CTRL]
gm_8016B1D8() -> s32 [CTRL]
gm_8016B1EC() -> bool [CTRL]
gm_8016B204() -> bool [CTRL]
gm_8016B238() -> s32 [CTRL]
gm_8016B248() -> f32 [CTRL]
gm_8016B258(s32%6) -> s32 [CTRL]
gm_SetGameSpeed({(float) (1 + rnd() % 8) * 0.25f|(1 + rnd() % 8) as f32 * 0.25}) -> void
gm_ResetGameSpeed() -> void [CTRL]
gm_8016B328() -> void [CTRL]
gm_8016B33C(s32) -> void [CTRL]
gm_8016B350(s32) -> void [CTRL]
gm_8016B364(s32) -> void [CTRL]
gm_8016B378(s8) -> void [CTRL]
fn_8016B388(s32%6, s32) -> void [CTRL]
gm_8016B3A0() -> bool
gm_8016B3D8() -> bool
gm_IsCurrently1PMode_inline() -> bool
get_unk_float() -> f32
gm_IsCurrently1PMode() -> bool
fn_8016B4BC() -> bool [CTRL]
fn_8016B510() -> bool [CTRL]
gm_8016B558() -> s32
fn_8016B5B0() -> f32 [CTRL]
gm_8016B6E8(s32%2, s32) -> void [CTRL]
fn_8016B728() -> u8 [CTRL]
fn_8016B738(s32) -> void [CTRL]
gm_8016B774() -> ptreq:controller.state.x24C [CTRL]
fn_8016B784() -> void [CTRL]
fn_8016B7B4(s32%5) -> void
fn_8016B7F8() -> void [CTRL]
fn_8016B88C(s32%2) -> void [CTRL]
gm_8016B8D4(s32%6, u8) -> void [CTRL]
fn_8016B918() -> void [CTRL PADS]
gm_AnyControllerPressedStart() -> s32 [PADS]
gm_AnyControllerPressedZ() -> s32 [PADS]
gm_DefaultVSGetPauser() -> s32 [PADS]
gm_CameraModeVSGetPauser() -> s32 [PADS]
gm_GetFFAOutcome() -> u32 [CTRL]
gm_GetTeamBattleOutcome() -> u32 [CTRL]
gmVs_SetEndHold(s32) -> void
gmVs_GetHeldOutcome() -> s32
gm_GetMatchOutcome() -> u32 [CTRL]
fn_8016C46C(s32) -> void [CTRL]
fn_8016C4F4(&ME) -> void [ME]
gm_8016C5C0(s32%6) -> s32 [CTRL]
gm_GetMatchEndPlayerScore(s32%6) -> s32 [CTRL]
gm_8016C6C0({(HSD_GObj*) &controller|addr_of_mut!(controller) as *mut HSD_GObj}) -> s32 [CTRL]
gm_8016C75C({(HSD_GObj*) &controller|addr_of_mut!(controller) as *mut HSD_GObj}) -> s32 [CTRL]
fn_8016C7D0({(HSD_GObj*) &controller|addr_of_mut!(controller) as *mut HSD_GObj}) -> void
fn_8016C7F0() -> void [CTRL ALLSTAR MISC]
gm_DoPauseChecksAndRoutine({&controller|addr_of_mut!(controller)}, s32%3) -> void [CTRL PADS]
gm_DoUnpauseChecksAndRoutine({&controller|addr_of_mut!(controller)}, s32%3) -> void [CTRL PADS]
fn_8016CD98({&controller|addr_of_mut!(controller)}) -> void [CTRL]
fn_8016CF4C(s32%6, u32%10) -> void [CTRL]
gmVs_EndMatch(s32%10) -> void
fn_8016CFE0() -> void [CTRL PADS]
gm_Scene_Training_OnFrame() -> void [CTRL PADS]
fn_8016D538() -> bool [CTRL]
fn_8016D634() -> void [CTRL ME]
gm_Scene_Vs_OnFrame() -> void [CTRL PADS]
fn_8016D8AC(s32%6, &PIDX) -> void [CTRL PIDX PADS]
fn_8016DCC0(&SMD) -> void [CTRL SMD]
fn_8016DEEC() -> void [CTRL]
fn_8016E124() -> void
fn_8016E2BC() -> void [CTRL]
fn_8016E5C0(&SMD) -> s32 [SMD PADS]
fn_8016E730(&SMD) -> void [CTRL SMD PADS RAND]
gm_Scene_Vs_OnEnter({(void*) &SMD|addr_of_mut!(SMD) as *mut c_void}) -> void [CTRL SMD PADS RAND]
gm_Scene_Vs_OnExit({(void*) &EMD|addr_of_mut!(EMD) as *mut c_void}) -> void [CTRL EMD]
gm_Scene_SuddenDeath_OnEnter({(void*) &SMD|addr_of_mut!(SMD) as *mut c_void}) -> void [CTRL SMD PADS RAND]
gm_Scene_Training_OnEnter({(void*) &SMD|addr_of_mut!(SMD) as *mut c_void}) -> void [CTRL SMD PADS RAND]
gm_8016ECE8() -> f32
gm_8016EDDC(s32%6, &PIDX) -> bool [CTRL PIDX]
fn_8016EF98(s32%6) -> s32 [CTRL]
gm_8016F00C(s32%6) -> void
fn_8016F030(&SMD) -> void [SMD]
getPort(&PIDX, s32%6) -> s32 [PIDX]
gm_LoadRumbleEnabled(&SMD) -> void [SMD]
gm_8016F120(s32) -> s32
fn_8016F140(s32) -> void
fn_8016F160(s32, s32) -> s32
*/

#define NCASES 600

/* ---- the synthetic game state; a case fills the objects its op names (G_*) and the hash covers the same ones */
#define G_CTRL 1u
#define G_SMD 2u
#define G_PIDX 4u
#define G_EMD 8u
#define G_PADS 16u
#define G_MISC 32u
#define G_LBL 64u
#define G_ME 128u
#define G_ALLSTAR 256u
#define G_RAND 512u

static struct lbl_8046B488_t LBL;
static MatchEnd ME;
static StartMeleeData SMD;
static PlayerInitData PIDX;
static struct EndMeleeData EMD;
static int PINT;
static u8 BYTEX;
static UnkAllstarData ALLSTAR;
static Unk1PData UNK1P;
static __typeof__(*(((StartMeleeRules*) 0)->x54)) X54OBJ;

static u32 shash(const char* s) { return s ? HBUF(s, (unsigned) __builtin_strlen(s)) + 1 : 0; }
static u32 fnid(const void* p)
{
    if (!p) return 0;
    if (p == (const void*) fn_8016B784) return 1;
    if (p == (const void*) fn_8016B7B4) return 2;
    if (p == (const void*) fn_8016B7F8) return 3;
    if (p == (const void*) fn_8016B88C) return 4;
    if (p == (const void*) fn_8016C7D0) return 5;
    if (p == (const void*) gm_AnyControllerPressedStart) return 6;
    if (p == (const void*) gm_AnyControllerPressedZ) return 7;
    if (p == (const void*) db_RunEveryFrame) return 8;
    return 99;
}
/* a MatchEnd address handed over as an int: which of the two blocks the TU passes (or neither) */
static u32 meid(s32 a)
{
    if (a == (s32) (size_t) &controller.state.x24C) return 1;
    if (a == (s32) (size_t) &EMD.xC) return 2;
    return 0;
}

/* ---- stubs the generator does not write (they hand out the synthetic state, write through a pointer, or carry a function pointer) */
void memzero(void* m, ssize_t n) { T(2000); T((u32) n); zero(m, (unsigned) n); }
struct lbl_8046B488_t* gm_1601_GetUnkData(void) { T(2001); return &LBL; }
u8 gm_GetCurrentGameMode(void) { u32 r; T(2002); r = rnd(); return (r % 3 == 0) ? 31 : (u8) (rnd() % 45); }
void Ground_801C1DE4(s32* a, s32* b) { T(2003); *a = (s32) (rnd() % 4) - 1; *b = (s32) (rnd() % 5); }
void ifStatus_802F6EA4(int a, int b, int c, int d, Event e, Event f) { T(2004); T((u32) a); T((u32) b); T((u32) c); T((u32) d); T(fnid((void*) e)); T(fnid((void*) f)); }
void fn_8017EE40(int a) { T(2005); T(meid(a)); }
void fn_8017E8A4(int a) { T(2006); T(meid(a)); }
void fn_8018A364(int a) { T(2007); T(meid(a)); }
UNK_T gm_801B6320(void) { T(2008); return &BYTEX; }
UnkAllstarData* gm_GetAllStarData(void) { T(2009); return &ALLSTAR; }
u64 gm_GetButtonsTriggered(u8 a) { u32 r; T(2010); T(a); r = rnd(); return (r % 3 == 0) ? 0 : (u64) (rnd() & 0x1170u); }
u64 gm_GetButtonsPressed(u8 a) { u32 r; T(2011); T(a); r = rnd(); return (r % 2) ? (u64) (0x1160u | (rnd() & 0x1170u)) : (u64) (rnd() & 0x1170u); }
void ifStatus_802F7034(UNK_T f) { T(2012); T(fnid(f)); }
Unk1PData* fn_8017DEC8(int a) { T(2013); T((u32) a); return &UNK1P; }
void gm_SetDbPauseInputHandlers(bool (*a)(void), bool (*b)(void)) { T(2014); T(fnid((void*) a)); T(fnid((void*) b)); }
void gm_SetPreGObjProcCallback(Event cb) { T(2015); T(fnid((void*) cb)); }
HSD_GObjProc* HSD_GObj_SetupProc(HSD_GObj* g, HSD_GObjEvent f, u8 p) { T(2016); T(g != 0); T(fnid((void*) f)); T(p); return (rnd() & 1) ? (HSD_GObjProc*) (size_t) 1 : 0; }
void Stage_80224E64(enum_t id, Vec3* v) { T(2017); T((u32) id); v->x = rg(); v->y = rg(); v->z = rg(); }
void Player_LoadPlayerCoords(s32 slot, Vec3* v) { T(2018); T((u32) slot); v->x = rg(); v->y = rg(); v->z = rg(); }

/* per-slot player state the TU reads back through several getters: consistent within a case */
static u8 SLOTT[6], TEAMS[6], STOCKS[6], PLID[6];
Gm_PKind Player_GetPlayerSlotType(s32 slot) { T(2020); T((u32) slot); return (Gm_PKind) SLOTT[(u32) slot % 6]; }
int Player_GetTeam(int slot) { T(2021); T((u32) slot); return TEAMS[(u32) slot % 6]; }
s32 Player_GetStocks(int slot) { T(2022); T((u32) slot); return STOCKS[(u32) slot % 6]; }
int Player_GetPlayerId(int slot) { T(2023); T((u32) slot); return PLID[(u32) slot % 6]; }

/* the callbacks the rules carry */
static void cb_frame_start(void) { T(3001); }
static void cb_frame_end(void) { T(3002); }
static void cb_match_start(void) { T(3003); }
static void cb_match_end(u8 o) { T(3004); T(o); }
static void cb_unpause(s32 a) { T(3005); T((u32) a); }
static void cb_pause(s32 a) { T(3006); T((u32) a); }
static s32 cb_pauser(void) { T(3007); return (s32) (rnd() % 6) - 1; }

static u32 run_custom(int op);
#include "ops_gmvs.h"

static void fix_ME(MatchEnd* me, int k)
{
    int i;
    fl_MatchEnd(me);
    me->x0 = rnd() % 3;
    for (i = 0; i < 6; i++) {
        MatchPlayerData* p = &me->player_standings[i];
        p->pkind = rnd() % 4;
        p->team = rnd() % 5;
        p->is_big_loser = rnd() % 6;
        p->score = (s32) (rnd() % 9) - 4;
        p->x20 = rnd() % 5;
    }
    (void) k;
}

static void fix_rules(StartMeleeRules* r)
{
    r->stkind = rnd() % 0x50;
    r->x9 = rnd() % 4;
    r->xD = rnd() % 4;
    r->xA = (rnd() % 3 == 0) ? (u8) rnd() : 0;
    r->time_limit = rnd() % 8;
    r->game_speed = (float) (1 + rnd() % 8) * 0.25f;
    r->x14 = rnd() % 5;
    r->is_teams = rnd() % 3;
    X54OBJ.x10_b1 = rnd() & 1;
    r->x54 = (rnd() & 1) ? &X54OBJ : 0;
    r->on_unpause_override = (rnd() & 1) ? cb_unpause : 0;
    r->on_pause_override = (rnd() & 1) ? cb_pause : 0;
    r->check_for_pauser_override = (rnd() & 1) ? cb_pauser : 0;
    r->on_match_start = (rnd() & 1) ? cb_match_start : 0;
    r->on_frame_start = (rnd() & 1) ? cb_frame_start : 0;
    r->on_frame_end = (rnd() & 1) ? cb_frame_end : 0;
    r->on_match_end = (rnd() & 1) ? cb_match_end : 0;
}

static void setup(u32 g, int k)
{
    int i;
    for (i = 0; i < 6; i++) { SLOTT[i] = rnd() % 5; TEAMS[i] = rnd() % 3; STOCKS[i] = rnd() % 4; PLID[i] = rnd() % 4; }
    if (g & G_CTRL) {
        fl_VsSceneController(&controller);
        fix_rules(&controller.start);
        controller.state.unk_1C = (u32*) lbl_803D5630;
        controller.state.unk_20 = (u32*) lbl_803D5620;
        controller.state.timer_seconds = (rnd() & 1) ? rnd() % 8 : rnd();
        controller.state.unk_2C = rnd() % 64;
        controller.state.unk_30 = rnd() % 4;
        controller.state.unk_0 = rnd() % 5;
        controller.state.is_singleplayer = rnd() % 4;
        controller.state.match_result = (rnd() % 3 == 0) ? 0 : rnd() % 10;
        controller.state.terminate_match = rnd() % 3;
        controller.state.pauser = (s8) (rnd() % 5) - 1;
        for (i = 0; i < 6; i++) {
            controller.state.fighters[i].x1 = rnd() % 0x54;
            controller.state.fighters[i].respawn_timer = rnd() % 3;
            controller.state.fighters[i].spawn_point = (s8) (rnd() % 6) - 1;
            controller.state.fighters[i].slot_type = rnd() % 5;
        }
        fix_ME(&controller.state.x24C, k);
        controller.start.time_limit = (rnd() & 1) ? controller.state.timer_seconds : rnd() % 8;
    }
    if (g & G_SMD) {
        fl_StartMeleeData(&SMD);
        fix_rules(&SMD.rules);
        for (i = 0; i < 6; i++) SMD.players[i].slot_type = (i < 4) ? rnd() % 5 : 1 + rnd() % 4;
    }
    if (g & G_PIDX) {
        fl_PlayerInitData(&PIDX);
        PIDX.slot = rnd() % 5;
    }
    if (g & G_EMD) {
        fix_ME(&EMD.xC, k);
        EMD.x0 = rnd();
        EMD.x4 = rnd();
        EMD.x8 = rnd();
    }
    if (g & G_PADS) {
        for (i = 0; i < 4; i++) {
            HSD_PadCopyStatus[i].err = rnd() % 2;
            HSD_PadCopyStatus[i].trigger = (rnd() & 1) ? (rnd() & 0x1418u) : 0;
            HSD_PadCopyStatus[i].button = (rnd() & 1) ? (rnd() & 0x1578u) : 0;
            HSD_PadMasterStatus[i].err = rnd() % 2;
            HSD_PadMasterStatus[i].trigger = (rnd() & 1) ? (rnd() & 0x1418u) : 0;
        }
        DbLevel = rnd() % 5;
    }
    if (g & G_MISC) PINT = (int) rnd();
    if (g & G_LBL) fl_lbl_8046B488_t(&LBL);
    if (g & G_ME) fix_ME(&ME, k);
    if (g & G_ALLSTAR) { ALLSTAR.x0.x0.ckind = rnd() % 26; BYTEX = rnd() % 26; UNK1P.x8 = rnd(); }
    if (g & G_RAND) { RANDSEED = rnd(); }
}

static u32 state_hash(u32 g)
{
    u32 h = 0;
    int i;
    h = h * 31u + pool_hash();
    if (g & G_CTRL) { h = hs_VsSceneController(&controller, h); }
    if (g & G_SMD) h = hs_StartMeleeData(&SMD, h);
    if (g & G_PIDX) h = hs_PlayerInitData(&PIDX, h);
    if (g & G_EMD) { h = hm_(h, EMD.x0); h = hm_(h, EMD.x4); h = hm_(h, EMD.x8); h = hs_MatchEnd(&EMD.xC, h); }
    if (g & G_MISC) h = hm_(h, (u32) PINT);
    if (g & G_LBL) h = hs_lbl_8046B488_t(&LBL, h);
    if (g & G_ME) h = hs_MatchEnd(&ME, h);
    if (g & G_RAND) h = hm_(h, RANDSEED);
    (void) i;
    return h;
}

static u32 run_custom(int op) { (void) op; return 0; }

void* __acrt_iob_func(unsigned);
int setvbuf(void*, char*, int, unsigned);
int main(void)
{
    int op, k;
    setvbuf(__acrt_iob_func(1), 0, 4, 0);
    HSD_RandSeedPtr = &RANDSEED;
    gmMainLib_804D3EE0 = &SAVEDATA;
    for (op = 1; op <= NOPS; op++) {
        u32 g = op_groups(op);
        for (k = 0; k < NCASES; k++) {
            u32 ret;
            tu_case(op, k);
            pool_reset();
            setup(g, k);
            ret = run_op(op);
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(g), TR);
        }
    }
    return 0;
}
