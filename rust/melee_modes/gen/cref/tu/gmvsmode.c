/* TU: src/melee/gm/gmvsmode.c */
/* HS: StartMeleeData MatchExitInfo SSSData ResultsMatchInfo PlayerInitData */
/* C oracle for src/tu/gmvsmode.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub, the state-table callbacks included (calling a table entry tells which function
 * it points at). One line per case: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "hs_gmvsmode.h"

CSSData gmVsMelee_CssData;
SSSData gmVsMelee_SssData;
StartMeleeData gmVsMelee_StartData;
MatchExitInfo gmVsMelee_VsExitInfo;
MatchExitInfo gmVsMelee_SuddenDeathExitInfo;
ResultsMatchInfo gmVsMelee_ResultsEnterData;
struct VsApproachData gmVsMelee_ApproachData;
struct un_804A1F48_t if_Scene_Prize_EnterData;

static GameModeState GS;
static VsModeData VSROW;
static StartMeleeData START;
static MatchExitInfo MEI;
static SSSData SSS;

void* gm_GetGameModeStateEnterData(GameModeState* s) { T(1); T(s == &GS); return s->info.enter_data; }
void* gm_GetGameModeStateExitData(GameModeState* s) { T(2); T(s == &GS); return s->info.exit_data; }
void gm_SetNextGameModeStateId(u8 id) { T(3); T(id); }
void gm_SetupRulesDefaults(struct StartMeleeRules* r) { T(4); T(r == &START.rules); r->x6 = rnd(); r->time_limit = rnd(); }
void gm_SetupPlayerDefaults(struct PlayerInitData* p)
{
    T(5); T((u32) (p - START.players));
    p->color = rnd(); p->stocks = rnd(); p->cpu_kind = rnd(); p->ckind = rnd(); p->slot_type = rnd();
    p->rumble_enabled = rnd();
}
void gm_LoadAnnouncer(void) { T(6); }
void gmFrontend_SelectScene(struct GameModeState* s, int sss) { T(7); T(s == &GS); T(sss); }
void gmFrontend_BeginLoading(void) { T(8); }
VsModeData* gmVsMelee_GetVsData(void) { T(9); return &VSROW; }
void gmVsMelee_EnterCss(GameModeState* s, struct VsModeData* v, CSSMatchType m) { T(10); T(s == &GS); T(v == &VSROW); T(m); }
void gmVsMelee_ExitCss(GameModeState* s, struct VsModeData* v) { T(11); T(s == &GS); T(v == &VSROW); }
void gmVsMelee_EnterSss(GameModeState* s, struct VsModeData* v) { T(12); T(s == &GS); T(v == &VSROW); }
void gmVsMelee_ExitSss(GameModeState* s, VsModeData* v, u8 id) { T(13); T(s == &GS); T(v == &VSROW); T(id); }
void gmVsMelee_EnterVs(GameModeState* s, struct VsModeData* v, void (*c1)(StartMeleeData*, StartMeleeData*), void (*c2)(PlayerInitData*, PlayerInitData*))
{
    T(14); T(s == &GS); T(v == &VSROW); T(c1 != 0); T(c2 != 0);
}
void gmVsMelee_ExitVs(GameModeState* s, u8 a, u8 b) { T(15); T(s == &GS); T(a); T(b); }
void gmVsMelee_EnterSuddenDeath(GameModeState* s, struct VsModeData* v, gm_StartMeleeCallback c1, gm_PlayerInitCallback c2)
{
    T(16); T(s == &GS); T(v == &VSROW); T(c1 != 0); T(c2 != 0);
}
void gmVsMelee_ExitSuddenDeath(GameModeState* s) { T(17); T(s == &GS); }
void gmVsMelee_EnterResults(GameModeState* s) { T(18); T(s == &GS); }
void gmVsMelee_ExitResults(GameModeState* s, struct VsModeData* v, u8 id) { T(19); T(s == &GS); T(v == &VSROW); T(id); }
struct gmm_x1868_1A8_t* gm_80162A98(s32 a) { T(20); T(a); return 0; }
void gm_RecordSelfDestructs(s32 a) { T(21); T(a); }
s32 gm_80162A4C(s32 a) { T(22); T(a); return 0; }
bool gm_WasMatchCanceled(u8 o) { T(23); T(o); return SR(3); }
s32 gm_801623A4(MatchEnd* e) { T(24); T(e == &gmVsMelee_ResultsEnterData.match_end); return 0; }
void gm_ModeState_Approach_OnEnter(GameModeState* s) { T(70); T(s == &GS); }
void gm_ModeState_ApproachVs_OnEnter(GameModeState* s) { T(71); T(s == &GS); }
void gm_ModeState_ApproachVs_OnExit(GameModeState* s) { T(72); T(s == &GS); }
void gm_ModeState_Prize_OnEnter(GameModeState* s) { T(73); T(s == &GS); }
void gm_ModeState_Prize_OnExit(GameModeState* s) { T(74); T(s == &GS); }

static u32 who(void* p)
{
    if (p == 0) return 0;
    if (p == &gmVsMelee_CssData) return 1;
    if (p == &gmVsMelee_SssData) return 2;
    if (p == &gmVsMelee_StartData) return 3;
    if (p == &gmVsMelee_VsExitInfo) return 4;
    if (p == &gmVsMelee_SuddenDeathExitInfo) return 5;
    if (p == &gmVsMelee_ResultsEnterData) return 6;
    if (p == &gmVsMelee_ApproachData) return 7;
    if (p == &if_Scene_Prize_EnterData) return 8;
    return 99;
}
static void table(GameModeState* t)
{
    int i;
    for (i = 0;; i++) {
        T(t[i].id); T(t[i].preload); T(t[i].flags); T(t[i].on_enter != 0); T(t[i].on_exit != 0);
        T(t[i].info.scene_kind); T(who(t[i].info.enter_data)); T(who(t[i].info.exit_data));
        zero(&GS, sizeof GS); GS.info.enter_data = &START; GS.info.exit_data = &MEI;
        if (t[i].on_enter) { T(1000 + i); t[i].on_enter(&GS); }
        if (t[i].on_exit) { T(2000 + i); t[i].on_exit(&GS); }
        if (t[i].id == 255) break;
    }
}

static u32 state_hash(void)
{
    u32 h = 0;
    h = hs_StartMeleeData(&START, h);
    h = hs_MatchExitInfo(&MEI, h);
    h = hs_SSSData(&SSS, h);
    h = hs_ResultsMatchInfo(&gmVsMelee_ResultsEnterData, h);
    return h;
}

int main(void)
{
    int op, k;
    tu_case(0, 0);
    zero(&MEI, sizeof MEI); zero(&gmVsMelee_ResultsEnterData, sizeof gmVsMelee_ResultsEnterData);
    table(gm_Mode_Vs_States);
    T(0xDEB);
    table(gm_Mode_DebugVs_States);
    printf("T 0 %08X %08X %08X\n", 0u, 0u, TR);
    for (op = 1; op <= 13; op++) {
        for (k = 0; k < 600; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(&GS, sizeof GS); zero(&START, sizeof START); zero(&MEI, sizeof MEI); zero(&SSS, sizeof SSS);
            zero(&gmVsMelee_ResultsEnterData, sizeof gmVsMelee_ResultsEnterData);
            fl_StartMeleeData(&START);
            fl_MatchExitInfo(&MEI);
            fl_SSSData(&SSS);
            fl_ResultsMatchInfo(&gmVsMelee_ResultsEnterData);
            {
                int i;
                for (i = 0; i < 6; i++) MEI.match_end.player_standings[i].pkind = rnd() % 5;
            }
            SSS.start_game = rnd() & 1;
            GS.info.enter_data = &START;
            GS.info.exit_data = &MEI;
            switch (op) {
            case 1: onEnterDebugVs(&GS); break;
            case 2: onEnterCss(&GS); break;
            case 3: onExitCss(&GS); break;
            case 4: onEnterSss(&GS); break;
            case 5: GS.info.exit_data = &SSS; onExitSss(&GS); break;
            case 6: onEnterLoading(&GS); break;
            case 7: onExitLoading(&GS); break;
            case 8: onEnterVs(&GS); break;
            case 9: onExitVs(&GS); break;
            case 10: onEnterSuddenDeath(&GS); break;
            case 11: onExitSuddenDeath(&GS); break;
            case 12: onEnterResults(&GS); break;
            case 13: gmVsMelee_ResultsEnterData.match_end.outcome = rnd() % 9; onExitResults(&GS); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
