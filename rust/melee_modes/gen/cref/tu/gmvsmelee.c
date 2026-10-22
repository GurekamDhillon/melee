/* TU: src/melee/gm/gmvsmelee.c */
/* HS: VsModeData CSSData SSSData StartMeleeData MatchExitInfo ResultsMatchInfo PreloadedGameModeState MatchEnd gmm_retval_ED98 gmm_retval_EDB0 */
/* C oracle for src/tu/gmvsmelee.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub (the gmscenelaunch.h inlines are stubs too). One line per case:
 * op case ret state-hash trace. */
/* what gmscenelaunch.h pulls in for the TU */
#include <dolphin/os.h>
#include <melee/gm/forward.h>
#include <melee/lb/lbdvd.h>
#include <melee/lb/types.h>
#include <melee/mn/types.h>
#include <melee/pl/forward.h>
#include <melee/gr/forward.h>
#include "gm_1601.h"
#include "gm_1884.h"
#include "gm_1A3F.h"
#include "gmmain_lib.h"
#define MELEE_GM_GMSCENELAUNCH_H /* keep the header inlines out; stubbed below */
struct VsModeData;
struct StartMeleeData;
int SceneLaunch_Active(void);
int SceneLaunch_EntryStateId(void);
int SceneLaunch_PlayerStocks(int slot);
static int SeedVs_stub(struct VsModeData* vs, int dummy);
static void HudCount_stub(struct StartMeleeData* s);
#define SceneLaunch_SeedVs(vs, d) SeedVs_stub((vs), (d))
#define SceneLaunch_HudCount(s) HudCount_stub((s))
#include TU_SRC
#include "tu_common.h"
#include "hs_gmvsmelee.h"

struct gmm_x0* gmMainLib_804D3EE0;
static struct gmm_x0 SAVE;
static VsModeData VSARG;
static VsModeData* VS;
static CSSData CSS;
static SSSData SSS;
static StartMeleeData START;
static MatchExitInfo MEI;
static PreloadedGameModeState PRE;
static struct gmm_retval_ED98 R98;
static struct gmm_retval_EDB0 RB0;
static GameModeState GS[2];
#define ROW (&gmMainLib_804D3EE0->modes.table[GmVsMode_Melee])

static u32 strh(const char* s) { return HBUF(s, (unsigned) __builtin_strlen(s)); }
static u32 whoend(MatchEnd* e)
{
    if (e == &gmVsMelee_VsExitInfo.match_end) return 1;
    if (e == &gmVsMelee_SuddenDeathExitInfo.match_end) return 2;
    if (e == &MEI.match_end) return 3;
    if (e == &gmVsMelee_ResultsEnterData.match_end) return 4;
    return 0;
}
static u32 whovs(VsModeData* v) { return v == ROW ? 1 : v == &VSARG ? 2 : 0; }

int Mex_Enabled(const char* s) { T(100); T(strh(s)); return SR(0); }
void OSReport(char* f, ...) { T(101); T(strh(f)); }
void memzero(void* m, ssize_t n) { T(102); T(m == (void*) ko_counts); T((u32) n); zero(m, (unsigned) n); }
u32 lbTime_8000AF74(u32 a, int b) { T(103); T(a); T(b); return rnd(); }
int SceneLaunch_Active(void) { T(104); return SR(1); }
int SceneLaunch_EntryStateId(void) { T(105); return SR(2) ? 2 : (int) (rnd() % 3); }
int SceneLaunch_PlayerStocks(int slot) { T(106); T(slot); return (int) (rnd() % 5) - 1; }
static void HudCount_stub(StartMeleeData* s) { T(107); T(s == &START); T(hs_StartMeleeData(s, 0)); }
static int SeedVs_stub(VsModeData* v, int d) { T(108); T(v == ROW); T(d); return SR(3); }
void* gm_GetGameModeStateEnterData(GameModeState* s) { T(1); T(s == &GS[0]); return s->info.enter_data; }
void* gm_GetGameModeStateExitData(GameModeState* s) { T(2); T(s == &GS[0]); return s->info.exit_data; }
void gm_ChangeGameModeAfterCurrentScene(int m) { T(3); T(m); }
void gm_SetNextGameModeStateId(u8 id) { T(4); T(id); }
void gm_InitVsMode(VsModeData* v) { T(5); T(v == ROW); T(hs_VsModeData(v, 0)); }
void gmMainLib_8015CDEC(void) { T(6); }
PreloadedGameModeState* lbDvd_GetPreloadCacheScene(void) { T(7); return &PRE; }
void lbDvd_SetupVsPreloadCache(void) { T(8); }
int Frontend_OnlinePick(void) { T(9); return (int) (rnd() % 3); }
void Frontend_OnlinePicked(int w, int a, int b) { T(10); T(w); T(a); T(b); }
void Frontend_BackToOnline(void) { T(11); }
u64 lbAudioAx_80026E84(CharacterKind ck) { u32 lo, hi; T(12); T(ck); lo = rnd(); hi = rnd(); return ((u64) hi << 32) | lo; }
void lbAudioAx_80026F2C(u32 a) { T(13); T(a); }
void lbAudioAx_8002702C(u32 f, u64 m) { T(14); T(f); T((u32) m); T((u32) (m >> 32)); }
void lbAudioAx_80027168(void) { T(15); }
u64 lbAudioAx_80026EBC(StKind sk) { u32 lo, hi; T(16); T(sk); lo = rnd(); hi = rnd(); return ((u64) hi << 32) | lo; }
void gm_80167FC4(SSSData* s) { T(17); T(s == &SSS); T(hs_SSSData(s, 0)); }
void gm_80167BC8(VsModeData* v) { T(18); T(whovs(v)); T(hs_VsModeData(v, 0)); v->start.rules.stkind = rnd(); }
void gm_SetupSubColors(StartMeleeData* s) { T(19); T(s == &START); T(hs_StartMeleeData(s, 0)); }
void gm_LoadRumbleEnabled(StartMeleeData* s) { T(20); T(s == &START); }
void gm_LoadAnnouncer(void) { T(21); }
void Netplay_StageEnd(int w, int s0, int s1) { T(22); T(w); T(s0); T(s1); }
int Netplay_RunActive(void) { T(23); return SR(4) && SR(5); }
void Netplay_GameResult(int w) { T(24); T(w); }
int Netplay_Enabled(void) { T(25); return SR(6); }
void Netplay_NoteTie(void) { T(26); }
int Netplay_RematchPending(void) { T(27); return SR(7) && SR(8); }
bool gm_WasMatchCanceled(u8 o) { T(28); T(o); return SR(9); }
bool gm_MatchHasMultipleWinners(MatchEnd* e) { T(29); T(whoend(e)); T(hs_MatchEnd(e, 0)); return SR(10); }
void gm_80162574(u8 ck, u8 o) { T(30); T(ck); T(o); }
void gm_SetupHumanResultsScreen(u8 mk, u8 o) { T(31); T(mk); T(o); }
void gm_SetupResultsScreenPlayTime(u32 s, u32 a) { T(32); T(s); T(a); }
u32 gm_80162800(MatchEnd* e) { T(33); T(whoend(e)); return rnd(); }
void gm_SetupSuddenDeath(StartMeleeData* s, MatchEnd* e) { T(34); T(s == &START); T(hs_StartMeleeData(s, 0)); T(whoend(e)); T(hs_MatchEnd(e, 0)); }
void gm_80166CCC(MatchEnd* a, MatchEnd* b) { T(35); T(whoend(a)); T(whoend(b)); T(hs_MatchEnd(a, 0)); T(hs_MatchEnd(b, 0)); }
void gm_80177724(struct ResultsMatchInfo* r) { T(36); T(r == &gmVsMelee_ResultsEnterData); T(hs_ResultsMatchInfo(r, 0)); }
void gm_80168638(MatchEnd* e) { T(37); T(whoend(e)); }
void gm_80168710(MatchEnd* e, VsModeData* v) { T(38); T(whoend(e)); T(whovs(v)); }
s32 gm_8016247C(s32 a) { T(39); T(a); return 0; }
s32 gm_801688AC(MatchEnd* e) { T(40); T(whoend(e)); return (s32) rnd(); }
u32 gm_GetVsPlayMatchTotal(void) { T(41); return rnd(); }
u8 gm_80172DD4(u32 a) { T(42); T(a); return SR(11) ? ChKind_None : rnd(); }
struct gmm_retval_ED98* gmMainLib_8015ED98(void) { T(43); return &R98; }
void gm_InitChallengerData(u8 a, u8 b, u8 c, u8 d, u8 e, u8 f) { T(44); T(a); T(b); T(c); T(d); T(e); T(f); }
u8 gm_80172D78(void) { T(45); return SR(12) ? ChKind_None : rnd(); }
u8 gm_80172E74(void) { T(46); return SR(13) ? ChKind_None : rnd(); }
u16 gm_80172F00(u32 a) { T(47); T(a); return SR(14) ? 328 : rnd(); }
struct gmm_retval_EDB0* gmMainLib_8015EDB0(void) { T(48); return &RB0; }
void gm_80164504(u16 a) { T(49); T(a); }
void gm_80173DE4(MatchEnd* e) { T(50); T(whoend(e)); }
int gm_80172898(u16 a) { T(51); T(a); return 0; }
void gm_80173EEC(void) { T(52); }
bool gm_801721EC(void) { T(53); return SR(15); }
void lbCardNew_AllocWorkArea(void) { T(54); }
void lbCardGame_LoadArchive(int a) { T(55); T(a); }
void lbCardGame_SaveChanges(void) { T(56); }

static void cbS(StartMeleeData* s, StartMeleeData* v) { T(60); T(s == &START); T(v == &VS->start); T(hs_StartMeleeData(v, 0)); s->rules.x6 = rnd(); }
static void cbP(PlayerInitData* a, PlayerInitData* b)
{
    T(61); T((u32) (a - START.players)); T(b == VS->start.players + (a - START.players));
    a->nametag = rnd(); b->team = rnd();
}

static void fix_end(MatchEnd* e)
{
    int i;
    for (i = 0; i < 6; i++) {
        e->player_standings[i].pkind = rnd() % 5;
        e->player_standings[i].is_small_loser = rnd() % 200;
        e->player_standings[i].team = rnd() % 5;
        e->player_standings[i].x3_b0 = rnd() & 63;
    }
    if (rnd() & 1) { e->player_standings[4].pkind = 3; e->player_standings[5].pkind = 3; }
    for (i = 0; i < 5; i++) e->team_standings[i].is_small_loser = rnd() % 200;
    e->is_teams = rnd() % 3;
    e->outcome = rnd() % 9;
    e->n_winners = rnd() % 3;
    e->winners[0] = rnd() % 4;
    e->frame_count = rnd() % 100000;
}
static void fix_vs(VsModeData* v)
{
    int i;
    for (i = 0; i < 6; i++) {
        v->start.players[i].slot_type = rnd() % 5;
        v->start.players[i].ckind = (rnd() % 5 == 0) ? (s8) rnd() : (s8) (rnd() % 0x28);
    }
    if (rnd() & 1) { v->start.players[4].slot_type = 3; v->start.players[5].slot_type = 3; }
    v->start.rules.stkind = rnd() % 40;
    v->start.rules.match_kind = rnd() % 4;
}

static u32 state_hash(void)
{
    u32 h = 0;
    h = hs_VsModeData(ROW, h);
    h = hs_VsModeData(&VSARG, h);
    h = hs_CSSData(&CSS, h);
    h = hs_SSSData(&SSS, h);
    h = hs_StartMeleeData(&START, h);
    h = hs_MatchExitInfo(&MEI, h);
    h = hs_MatchExitInfo(&gmVsMelee_VsExitInfo, h);
    h = hs_MatchExitInfo(&gmVsMelee_SuddenDeathExitInfo, h);
    h = hs_ResultsMatchInfo(&gmVsMelee_ResultsEnterData, h);
    h = hs_PreloadedGameModeState(&PRE, h);
    h = HBUF(ko_counts, 6) ^ (h * 31);
    return h;
}

int main(void)
{
    int op, k;
    for (op = 1; op <= 19; op++) {
        for (k = 0; k < 1200; k++) {
            u32 ret = 0;
            int i;
            u8 kc[6];
            tu_case(op, k);
            zero(&SAVE, sizeof SAVE); zero(&VSARG, sizeof VSARG); zero(&CSS, sizeof CSS); zero(&SSS, sizeof SSS);
            zero(&START, sizeof START); zero(&MEI, sizeof MEI); zero(&PRE, sizeof PRE); zero(&R98, sizeof R98);
            zero(&RB0, sizeof RB0); zero(GS, sizeof GS);
            zero(&gmVsMelee_CssData, sizeof gmVsMelee_CssData); zero(&gmVsMelee_SssData, sizeof gmVsMelee_SssData);
            zero(&gmVsMelee_StartData, sizeof gmVsMelee_StartData); zero(&gmVsMelee_VsExitInfo, sizeof gmVsMelee_VsExitInfo);
            zero(&gmVsMelee_SuddenDeathExitInfo, sizeof gmVsMelee_SuddenDeathExitInfo);
            zero(&gmVsMelee_ResultsEnterData, sizeof gmVsMelee_ResultsEnterData);
            gmMainLib_804D3EE0 = &SAVE;
            fl_VsModeData(ROW); fl_VsModeData(&VSARG); fl_CSSData(&CSS); fl_SSSData(&SSS); fl_StartMeleeData(&START);
            fl_MatchExitInfo(&MEI); fl_MatchExitInfo(&gmVsMelee_VsExitInfo); fl_MatchExitInfo(&gmVsMelee_SuddenDeathExitInfo);
            fl_ResultsMatchInfo(&gmVsMelee_ResultsEnterData); fl_PreloadedGameModeState(&PRE);
            fl_gmm_retval_ED98(&R98); fl_gmm_retval_EDB0(&RB0);
            for (i = 0; i < 6; i++) ko_counts[i] = rnd();
            for (i = 0; i < 6; i++) kc[i] = rnd();
            fix_vs(ROW); fix_vs(&VSARG);
            fix_end(&MEI.match_end); fix_end(&gmVsMelee_VsExitInfo.match_end); fix_end(&gmVsMelee_SuddenDeathExitInfo.match_end);
            fix_end(&gmVsMelee_ResultsEnterData.match_end);
            VS = (rnd() & 1) ? ROW : &VSARG;
            CSS.pending_scene_change = (rnd() % 3 == 0) ? 2 : rnd();
            SSS.start_game = rnd() & 1;
            GS[0].info.enter_data = &CSS;
            GS[0].info.exit_data = &CSS;
            GS[1].id = (rnd() % 3 == 0) ? 255 : rnd() % 200;
            switch (op) {
            case 1: T(gmVsMelee_GetVsData() == ROW); break;
            case 2: T(gmVsMelee_GetKOCounts() == ko_counts); break;
            case 3: gmVsMelee_UpdateKOCounts(kc, &MEI.match_end); T(HBUF(kc, 6)); break;
            case 4: ret = gmVsMelee_WasAnyPlayerHuman(&MEI.match_end); break;
            case 5: MEI.match_end.player_standings[rnd() % 6].pkind = 0; ret = findSmallestLoser(&MEI.match_end); break;
            case 6: gmVsMelee_Mode_OnInit(); break;
            case 7: gmVsMelee_ResetKOCounts(); break;
            case 8: gmVsMelee_Mode_OnLoad(); break;
            case 9: gm_Mode_Vs_OnUnload(); break;
            case 10: gmVsMelee_EnterCss(&GS[0], VS, rnd() % 3); break;
            case 11: gmVsMelee_ExitCss(&GS[0], VS); break;
            case 12: GS[0].info.enter_data = &SSS; gmVsMelee_EnterSss(&GS[0], VS); break;
            case 13: GS[0].info.exit_data = &SSS; gmVsMelee_ExitSss(&GS[0], VS, rnd()); break;
            case 14: GS[0].info.enter_data = &START;
                { gm_StartMeleeCallback c1 = (rnd() % 3) ? cbS : 0; gm_PlayerInitCallback c2 = (rnd() % 3) ? cbP : 0;
                  gmVsMelee_EnterVs(&GS[0], VS, c1, c2); } break;
            case 15: GS[0].info.exit_data = &MEI; { u8 a = rnd(), b = rnd(); gmVsMelee_ExitVs(&GS[0], a, b); } break;
            case 16: GS[0].info.enter_data = &START;
                { gm_StartMeleeCallback c1 = (rnd() % 3) ? cbS : 0; gm_PlayerInitCallback c2 = (rnd() % 3) ? cbP : 0;
                  gmVsMelee_EnterSuddenDeath(&GS[0], VS, c1, c2); } break;
            case 17: GS[0].info.exit_data = &MEI; gmVsMelee_ExitSuddenDeath(&GS[0]); break;
            case 18: GS[0].info.enter_data = &gmVsMelee_ResultsEnterData; gmVsMelee_EnterResults(&GS[0]); break;
            case 19: gmVsMelee_ExitResults(GS, VS, rnd()); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
