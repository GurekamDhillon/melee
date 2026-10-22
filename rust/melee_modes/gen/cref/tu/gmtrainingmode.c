/* TU: src/melee/gm/gmtrainingmode.c */
/* HS: VsModeData CSSData SSSData StartMeleeData MatchExitInfo PreloadedGameModeState CssSubStruct PlayerInitData */
/* C oracle for src/tu/gmtrainingmode.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub (SceneLaunch_SeedVs, a TARGET_PC header inline, is one too). One line per case:
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
#define MELEE_GM_GMSCENELAUNCH_H /* keep the header inline out; stubbed below */
struct VsModeData;
static int SeedVs_stub(struct VsModeData* vs, int dummy);
#define SceneLaunch_SeedVs(vs, d) SeedVs_stub((vs), (d))
#include TU_SRC
#include "tu_common.h"
#include "hs_gmtrainingmode.h"

struct gmm_x0* gmMainLib_804D3EE0;
CssSubStruct gm_80473814;

static GameModeState GS;
static CSSData CSS;
static SSSData SSS;
static StartMeleeData SMD;
static MatchExitInfo MEI;
static PreloadedGameModeState PRE;
static struct gmm_x0 SAVE;

#define VSROW (&gmMainLib_804D3EE0->modes.table[GmVsMode_Training])

void* gm_GetGameModeStateEnterData(GameModeState* s) { T(1); T(s == &GS); return s->info.enter_data; }
void* gm_GetGameModeStateExitData(GameModeState* s) { T(2); T(s == &GS); return s->info.exit_data; }
void lbCardNew_AllocWorkArea(void) { T(3); }
void lbCardGame_LoadArchive(int a) { T(4); T(a); }
void lbCardGame_SaveChanges(void) { T(5); }
void gm_801B06B0(CSSData* c, u8 mt, s8 ck, s8 st, s8 col, u8 nt, u8 lvl, u8 slot)
{
    T(6); T(c == &CSS); T(hs_CSSData(c, 0)); T(mt); T(ck); T(st); T(col); T(nt); T(lvl); T(slot);
}
void gm_801B07B4(CSSData* c, s8 ck, s8 st, s8 col, u8 nt, u8 lvl, u8 slot)
{
    T(7); T(c == &CSS); T(hs_CSSData(c, 0)); T(ck); T(st); T(col); T(nt); T(lvl); T(slot);
}
void lbDvd_SetupVsPreloadCache(void) { T(8); }
u32 lbTime_8000AF74(u32 a, int b) { T(39); T(a); T(b); return rnd(); }
void gmFrontend_TrainingSelect(struct GameModeState* s, int sss) { T(9); T(s == &GS); T(sss); }
void gm_801B0730(CSSData* c, s8* ck, u8* st, u8* col, u8* nt, u8* lvl)
{
    T(10); T(c == &CSS); T(hs_CSSData(c, 0));
    T(ck != 0); T(st != 0); T(col != 0); T(nt != 0); T(lvl != 0);
    if (ck) *ck = rnd();
    if (st) *st = rnd();
    if (col) *col = rnd();
    if (nt) *nt = rnd();
    if (lvl) *lvl = rnd();
}
void gm_801B07E8(CSSData* c, s8* ck, s8* st, s8* col, s8* a4, u8* lvl)
{
    T(11); T(c == &CSS); T(hs_CSSData(c, 0));
    T(ck != 0); T(st != 0); T(col != 0); T(a4 != 0); T(lvl != 0);
    if (ck) *ck = rnd();
    if (st) *st = rnd();
    if (col) *col = rnd();
    if (a4) *a4 = rnd();
    if (lvl) *lvl = rnd();
}
void gm_ChangeGameModeAfterCurrentScene(int m) { T(12); T(m); }
void gm_SetupAllPlayerDefaults(struct PlayerInitData* p)
{
    T(13); T(p == VSROW->start.players ? 1 : p == SMD.players ? 2 : 0);
    p[0].ckind = rnd();
    p[1].color = rnd();
}
u8 gm_GetNumCostumesForCKind(u8 ck) { T(14); T(ck); return 1 + rnd() % 7; }
PreloadedGameModeState* lbDvd_GetPreloadCacheScene(void) { T(15); return &PRE; }
void lbDvd_80018254(void) { T(16); }
u64 lbAudioAx_80026E84(CharacterKind ck) { u32 lo, hi; T(17); T(ck); lo = rnd(); hi = rnd(); return ((u64) hi << 32) | lo; }
void lbAudioAx_80026F2C(u32 a) { T(18); T(a); }
void lbAudioAx_8002702C(u32 f, u64 m) { T(19); T(f); T((u32) m); T((u32) (m >> 32)); }
void lbAudioAx_80027168(void) { T(20); }
void gm_SetNextGameModeStateId(u8 id) { T(21); T(id); }
u64 lbAudioAx_80026EBC(StKind sk) { u32 lo, hi; T(22); T(sk); lo = rnd(); hi = rnd(); return ((u64) hi << 32) | lo; }
void gm_SetupRulesDefaults(struct StartMeleeRules* r)
{
    T(23); T(r == &SMD.rules);
    r->stkind = rnd();
    r->time_limit = rnd();
}
void gm_SetupHumanPlayer(PlayerInitData* p, u8 ck, u8 col, u8 st, u8 slot)
{
    T(24); T((u32) (p - SMD.players)); T(ck); T(col); T(st); T(slot);
    p->ckind = rnd(); p->color = rnd(); p->nametag = rnd();
}
void gm_SetupCpuPlayer(PlayerInitData* p, u8 ck, u8 col, u8 st, u8 slot)
{
    T(25); T((u32) (p - SMD.players)); T(ck); T(col); T(st); T(slot);
    p->slot = rnd(); p->team = rnd();
}
void gm_LoadRumbleEnabled(struct StartMeleeData* d) { T(26); T(d == &SMD); T(hs_StartMeleeData(d, 0)); }
void gm_80189CDC(StartMeleeData* d) { T(27); T(d == &SMD); }
s32 gm_80162968(u32 s) { T(28); T(s); return 0; }
s32 gm_8016247C(s32 a) { T(29); T(a); return 0; }
bool gm_80163298(s8 ck, u16 a) { T(30); T(ck); T(a); return 0; }
int gm_80188454(int idx) { T(31); T(idx); return (int) rnd(); }
void gm_80173BC4(s8 a) { T(32); T(a); }
void gm_80173EEC(void) { T(33); }
bool gm_80173754(u8 a, u8 b) { T(34); T(a); T(b); return SR(7); }
void lbAudioAx_80024030(int a) { T(35); T(a); }
void gm_InitVsMode(struct VsModeData* v) { T(36); T(v == VSROW); T(hs_VsModeData(v, 0)); v->loser = rnd(); }
u8 gm_801677F0(void) { T(37); return rnd() % 6; }
static int SeedVs_stub(struct VsModeData* vs, int dummy) { T(38); T(vs == VSROW); T(dummy); return SR(9); }

static void setup(void)
{
    zero(&GS, sizeof GS); zero(&CSS, sizeof CSS); zero(&SSS, sizeof SSS); zero(&SMD, sizeof SMD);
    zero(&MEI, sizeof MEI); zero(&PRE, sizeof PRE); zero(&SAVE, sizeof SAVE); zero(&gm_80473814, sizeof gm_80473814);
    gmMainLib_804D3EE0 = &SAVE;
}

static u32 state_hash(void)
{
    u32 h = 0;
    h = hs_VsModeData(VSROW, h);
    h = hs_CSSData(&CSS, h);
    h = hs_SSSData(&SSS, h);
    h = hs_StartMeleeData(&SMD, h);
    h = hs_PreloadedGameModeState(&PRE, h);
    h = hs_CssSubStruct(&gm_80473814, h);
    h = hm_(h, gm_804D68C0);
    h = hm_(h, gm_804D68C1);
    return h;
}

int main(void)
{
    int op, k;
    /* the table, once */
    {
        int i;
        tu_case(0, 0);
        for (i = 0; i < 4; i++) {
            GameModeState* t = &gm_Mode_Training_States[i];
            T(t->id); T(t->preload); T(t->flags); T(t->on_enter != 0); T(t->on_exit != 0);
            T(t->info.scene_kind); T(t->info.enter_data != 0); T(t->info.exit_data != 0);
        }
        T(gm_Mode_Training_States[0].info.enter_data == (void*) &css_data);
        T(gm_Mode_Training_States[0].info.exit_data == (void*) &css_data);
        T(gm_Mode_Training_States[1].info.enter_data == (void*) &sss_data);
        T(gm_Mode_Training_States[1].info.exit_data == (void*) &sss_data);
        T(gm_Mode_Training_States[2].info.enter_data == (void*) &training_enter_data);
        T(gm_Mode_Training_States[2].info.exit_data == (void*) &training_exit_data);
        T(sizeof training_exit_data / sizeof(void*));
        printf("T 0 %08X %08X %08X\n", 0u, 0u, TR);
    }
    for (op = 1; op <= 9; op++) {
        for (k = 0; k < 1500; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            fl_VsModeData(VSROW);
            fl_CSSData(&CSS);
            fl_SSSData(&SSS);
            fl_StartMeleeData(&SMD);
            fl_PreloadedGameModeState(&PRE);
            fl_CssSubStruct(&gm_80473814);
            fl_MatchExitInfo(&MEI);
            gm_804D68C0 = rnd() % 6;
            gm_804D68C1 = (rnd() & 1) ? 0 : rnd();
            CSS.pending_scene_change = (rnd() % 4 == 0) ? 2 : rnd();
            SSS.start_game = (rnd() & 1);
            GS.info.enter_data = &CSS;
            GS.info.exit_data = &CSS;
            switch (op) {
            case 1: gm_801B1B74(&GS); break;
            case 2: gm_801B1C24(&GS); break;
            case 3: GS.info.enter_data = &SSS; gm_801B1EB8(&GS); break;
            case 4: GS.info.exit_data = &SSS; gm_801B1EEC(&GS); break;
            case 5: GS.info.enter_data = &SMD; gm_801B1F70(&GS); break;
            case 6: GS.info.exit_data = &MEI; gm_801B2204(&GS); break;
            case 7: gm_Mode_Training_OnInit(); break;
            case 8: gm_Mode_Training_OnLoad(); break;
            case 9: fn_801B1F6C(1); gm_801B07E8_layer(&CSS, 0, 0, 0, 0, 0); break;
            }
            T(hs_MatchExitInfo(&MEI, 0));
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
