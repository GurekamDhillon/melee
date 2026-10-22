/* C reference for rust/melee_modes/src/tu/gm_1BFA.rs: the game's gm_1BFA.c verbatim, with the callees
 * stubbed (each logs its arguments; the Rust hooks log the same text) and the data of other TUs defined
 * here. The log is folded into a rolling hash after every callee so that long runs (the prize list
 * calls ~1000 callees) fit; the first bytes are kept for diagnosis.
 * Run: python rust/melee_modes/gen/turef/tu_shard457_gm.py tu_gm_1BFA */
#include "../../../../src/melee/gm/gm_1BFA.c"
#include "tu_gm_common.h"

#define NPOOL 400

struct VsApproachData gmVsMelee_ApproachData;
StartMeleeData gmVsMelee_StartData;
MatchExitInfo gmVsMelee_VsExitInfo;
struct un_804A1F48_t if_Scene_Prize_EnterData;

static struct un_804A1F48_t POOL[NPOOL];
static int g_nalloc, g_alloc_fail_after;
static ChallengerData CH;
static u32 g_t804[0x42];
static u32 g_t970[TY_TROPHY_COUNT];
static u8 g_t1[0x42];
static s32 g_t2[TY_TROPHY_COUNT], g_t3[TY_TROPHY_COUNT], g_t4[TY_TROPHY_COUNT];
static u32 g_tick;
static int g_pending_ret, g_unlock_idx, g_rand_u8, g_stkind;

/* ---- folded log */
static u32 g_h = 2166136261u;
static int g_ncalls;
static char g_head[121];
static int g_hl;
static void fold(void)
{
    size_t i;
    for (i = 0; i < g_loglen; i++) {
        g_h = (g_h ^ (u8) g_log[i]) * 16777619u;
        if (g_hl < 120) {
            g_head[g_hl++] = g_log[i];
            g_head[g_hl] = 0;
        }
    }
    g_ncalls++;
    g_loglen = 0;
    g_log[0] = 0;
}
#define LG(...) \
    do { \
        lg(__VA_ARGS__); \
        fold(); \
    } while (0)

/* gm_1A3F.c: the real accessors. */
void* gm_GetGameModeStateEnterData(GameModeState* s) { return s->info.enter_data; }
void* gm_GetGameModeStateExitData(GameModeState* s) { return s->info.exit_data; }

ChallengerData* gm_GetChallengerData(void) { LG("chal"); return &CH; }
void gm_SetupRulesDefaults(struct StartMeleeRules* r) { LG("rules"); }
void gm_SetupAllPlayerDefaults(struct PlayerInitData* p) { LG("players"); }
StKind gm_GetChallengerStKind(SelectableCharacterKind k) { LG("stkind(%u)", (unsigned) k); return (StKind) g_stkind; }
u8 gm_CKindToSelKind(u8 ck) { LG("sel(%u)", (unsigned) ck); return (u8) (ck ^ 0x5A); }
void gm_SetupHumanPlayer(PlayerInitData* p, u8 ck, u8 color, u8 stocks, u8 slot)
{
    LG("human(%d,%u,%u,%u,%u)", (int) (p - gmVsMelee_StartData.players), (unsigned) ck, (unsigned) color, (unsigned) stocks, (unsigned) slot);
}
void gm_SetupCpuPlayer(PlayerInitData* p, u8 ck, u8 color, u8 stocks, u8 slot)
{
    LG("cpu(%d,%u,%u,%u,%u)", (int) (p - gmVsMelee_StartData.players), (unsigned) ck, (unsigned) color, (unsigned) stocks, (unsigned) slot);
}
u8 gm_DecideChallengerCpuLevel(u8 ck, u8 tag) { LG("lvl(%u,%u)", (unsigned) ck, (unsigned) tag); return (u8) g_rand_u8; }
void gm_LoadRumbleEnabled(struct StartMeleeData* s) { LG("rumble(%d)", s == &gmVsMelee_StartData); }
s32 gm_80162968(u32 seconds) { LG("sec(%u)", (unsigned) seconds); return 0; }
s32 gm_8016247C(s32 a) { LG("xE(%d)", (int) a); return 0; }
void gm_UnlockCKind(CharacterKind ck) { LG("unlock(%u)", (unsigned) ck); }
void gmMainLib_8015DB2C(u8 a) { LG("relock(%u)", (unsigned) a); }
s8 gm_CKindToUnlockIndex(CharacterKind ck) { LG("uidx(%u)", (unsigned) ck); return (s8) g_unlock_idx; }
void gm_80173EEC(void) { LG("3EEC"); }
int gm_80172898(u16 a) { LG("2898(%u)", (unsigned) a); return 0; }
bool gm_801721EC(void) { LG("21EC"); return g_pending_ret != 0; }
void gm_SetPendingGameMode(u8 m) { LG("pend(%u)", (unsigned) m); }
void gm_SetNewGameModePending(void) { LG("newpend"); }
void memzero(void* mem, ssize_t size)
{
    while (size-- > 0) {
        *(u8*) mem = 0;
        mem = (u8*) mem + 1;
    }
}
void Toy_803124BC(void) { LG("toy124BC"); }
bool gm_8017219C(u32 i) { LG("have(%u)", (unsigned) i); return g_t1[i] != 0; }
u32* gmMainLib_8015D804(s32 i) { LG("d804(%d)", (int) i); return &g_t804[i]; }
s32 gmMainLib_8015DA1C(u32 i) { LG("da1c(%u)", (unsigned) i); return g_t2[i]; }
int gmMainLib_8015DA90(u32 i) { LG("da90(%u)", (unsigned) i); return g_t3[i]; }
u32* gmMainLib_8015D970(ssize_t i) { LG("d970(%d)", (int) i); return &g_t970[i]; }
void gmMainLib_8015DA40(u32 i) { LG("da40(%u)", (unsigned) i); }
u32 lbTime_GetTimeInSeconds(void) { LG("time"); return g_tick += 7; }
void Toy_SetUnlockState(enum_t id, bool v) { LG("setunl(%d,%d)", (int) id, (int) v); }
s32 Toy_803048C0(int i) { LG("toy48C0(%d)", i); return g_t4[i]; }
void gm_80172174(void) { LG("2174"); }
void gm_80174180(void) { LG("4180"); }
void* HSD_MemAlloc(ssize_t size)
{
    LG("alloc(%d)", (int) size);
    if (g_nalloc < NPOOL && g_nalloc < g_alloc_fail_after) {
        return &POOL[g_nalloc++];
    }
    return NULL;
}
void lbCardNew_AllocWorkArea(void) { LG("allocwork"); }
void lbCardGame_LoadArchive(int a) { LG("cardload(%d)", a); }
void lbCardGame_SaveChanges(void) { LG("cardsave"); }

static const char* FUNC_NAMES[] = {"ApproachOnEnter", "ApproachVsOnEnter", "onExitVs", "gm_801BFC60", "PrizeOnEnter", "onExitPrize", "table"};
#define NFUNCS ((int) (sizeof FUNC_NAMES / sizeof FUNC_NAMES[0]))
#define NCASES 200

static GameModeState SCENE;
static struct VsApproachData VSAPP;
static void* SLOT;

static void fill_bytes(void* p, size_t n)
{
    size_t i;
    for (i = 0; i < n; i++) {
        ((u8*) p)[i] = (u8) rnd();
    }
}

static const char* idp(const void* p)
{
    static char buf[8][24];
    static int k;
    char* b = buf[k++ & 7];
    int i;
    if (p == NULL) return "null";
    if (p == &gmVsMelee_ApproachData) return "APP";
    if (p == &gmVsMelee_StartData) return "START";
    if (p == &gmVsMelee_VsExitInfo) return "EXIT";
    if (p == &if_Scene_Prize_EnterData) return "PRIZE";
    if (p == &if_Scene_Prize_EnterData.x8) return "PRIZE.x8";
    if (p == &SLOT) return "SLOT";
    for (i = 0; i < NPOOL; i++) {
        if (p == &POOL[i] || p == &POOL[i].x8) {
            int digits = 0;
            int w = i;
            int v = i;
            int n;
            do {
                digits++;
                w /= 10;
            } while (w);
            b[0] = 'P';
            for (n = digits; n > 0; n--) {
                b[n] = (char) ('0' + v % 10);
                v /= 10;
            }
            n = digits + 1;
            if (p == &POOL[i].x8) {
                b[n++] = '.';
                b[n++] = 'x';
                b[n++] = '8';
            }
            b[n] = 0;
            return b;
        }
    }
    return "?";
}

static const char* fname(void (*f)(GameModeState*))
{
    if (f == NULL) return "-";
    if (f == gm_ModeState_Approach_OnEnter) return "A";
    if (f == gm_ModeState_ApproachVs_OnEnter) return "V";
    if (f == onExitVs) return "XV";
    if (f == gm_ModeState_Prize_OnEnter) return "P";
    if (f == onExitPrize) return "XP";
    return "?";
}

static void setup_case(int f)
{
    int i;
    CH.human_ckind = (u8) rnd();
    CH.human_color = (u8) rnd();
    CH.human_slot = (u8) rnd();
    CH.human_nametag = (u8) rnd();
    CH.cpu_ckind = (u8) rnd();
    CH.curr_mode = (u8) rnd();
    CH.prev_mode = (u8) rnd();
    CH.x8 = (u16) rnd();
    g_stkind = (int) (rnd() % 40);
    g_rand_u8 = (int) (rnd() & 0xFF);
    g_unlock_idx = (int) (rnd() % 30) - 5;
    g_pending_ret = (int) (rnd() & 1);
    g_nalloc = 0;
    g_alloc_fail_after = (rnd() % 4 == 0) ? (int) (rnd() % 30) : NPOOL;
    g_tick = rnd() & 0xFFFF;
    for (i = 0; i < 0x42; i++) {
        g_t1[i] = (u8) (rnd() % 3 != 0);
        g_t804[i] = rnd();
    }
    for (i = 0; i < TY_TROPHY_COUNT; i++) {
        g_t2[i] = (s32) (rnd() % 3);
        g_t3[i] = (s32) (rnd() % 2);
        g_t4[i] = (s32) (rnd() % 4);
        g_t970[i] = rnd();
    }
    fill_bytes(POOL, sizeof POOL);
    fill_bytes(&if_Scene_Prize_EnterData, sizeof if_Scene_Prize_EnterData);
    if_Scene_Prize_EnterData.x8 = NULL;
    for (i = 0; i < NPOOL; i++) {
        POOL[i].x8 = NULL;
    }
    fill_bytes(gm_8049E558, sizeof gm_8049E558);
    fill_bytes(&VSAPP, sizeof VSAPP);
    fill_bytes(&gmVsMelee_StartData, sizeof gmVsMelee_StartData);
    fill_bytes(&gmVsMelee_VsExitInfo, sizeof gmVsMelee_VsExitInfo);
    gmVsMelee_VsExitInfo.match_end.outcome = (u8) (rnd() % 10);
    gmVsMelee_VsExitInfo.match_end.player_standings[0].stocks = (s8) ((int) (rnd() % 5) - 1);
    SLOT = (void*) &CH;
    memset(&SCENE, 0, sizeof SCENE);
    if (f == 0) {
        SCENE.info.enter_data = &VSAPP;
    } else if (f == 1) {
        SCENE.info.enter_data = &gmVsMelee_StartData;
    } else if (f == 2) {
        SCENE.info.exit_data = &gmVsMelee_VsExitInfo;
    } else if (f == 4) {
        SCENE.info.enter_data = &if_Scene_Prize_EnterData;
    }
    g_loglen = 0;
    g_log[0] = 0;
    g_h = 2166136261u;
    g_ncalls = 0;
    g_hl = 0;
    g_head[0] = 0;
}

static u32 fnv_bytes(const u8* p, size_t n)
{
    u32 h = 2166136261u;
    size_t i;
    for (i = 0; i < n; i++) {
        h = (h ^ p[i]) * 16777619u;
    }
    return h;
}

static void print_log(void)
{
    printf("|log=%08x,%d,%s", (unsigned) g_h, g_ncalls, g_head);
}

static void print_pool(void)
{
    int i;
    printf("|prize=%u,%u,%d,%s|nalloc=%d", (unsigned) if_Scene_Prize_EnterData.x0, (unsigned) if_Scene_Prize_EnterData.x2,
           (int) if_Scene_Prize_EnterData.x4, idp(if_Scene_Prize_EnterData.x8), g_nalloc);
    {
        u32 h = 2166136261u;
        for (i = 0; i < g_nalloc; i++) {
            u32 w[4];
            w[0] = POOL[i].x0;
            w[1] = POOL[i].x2;
            w[2] = (u32) POOL[i].x4;
            w[3] = (u32) ((POOL[i].x8 == NULL) ? -1 : (int) ((struct un_804A1F48_t*) POOL[i].x8 - POOL));
            h = (h ^ fnv_bytes((u8*) w, sizeof w)) * 16777619u;
        }
        printf("|pool=%08x", (unsigned) h);
    }
}

int main(void)
{
    int f;
    int i;
    for (f = 0; f < NFUNCS; f++) {
        for (i = 0; i < (f == 6 ? 1 : NCASES); i++) {
            printf("%d.%d %s", f, i, FUNC_NAMES[f]);
            if (f == 6) {
                GameModeState* t = gm_Mode_ChallengerApproach_States;
                int k;
                for (k = 0; k < 4; k++) {
                    printf("|%d:%u,%u,%u,%s,%s,%u,%s,%s", k, (unsigned) t[k].id, (unsigned) t[k].preload, (unsigned) t[k].flags,
                           fname(t[k].on_enter), fname(t[k].on_exit), (unsigned) t[k].info.scene_kind, idp(t[k].info.enter_data),
                           idp(t[k].info.exit_data));
                }
                printf("\n");
                continue;
            }
            setup_case(f);
            if (f == 0) {
                gm_ModeState_Approach_OnEnter(&SCENE);
                print_log();
                printf("|vs=%u,%u|ch=%u,%u,%u,%u,%u,%u,%u,%u\n", (unsigned) VSAPP.cpu_ckind, (unsigned) VSAPP.human_slot, (unsigned) CH.human_ckind,
                       (unsigned) CH.human_color, (unsigned) CH.human_slot, (unsigned) CH.human_nametag, (unsigned) CH.cpu_ckind,
                       (unsigned) CH.curr_mode, (unsigned) CH.prev_mode, (unsigned) CH.x8);
            } else if (f == 1) {
                StartMeleeData* s = &gmVsMelee_StartData;
                int k;
                gm_ModeState_ApproachVs_OnEnter(&SCENE);
                print_log();
                printf("|rules=%u,%u,%u,%u,%u,%u,%u,%d,%d,%u,%u,%u", (unsigned) s->rules.timer_enabled, (unsigned) s->rules.match_kind,
                       (unsigned) s->rules.x0_3, (unsigned) s->rules.x6, (unsigned) s->rules.x7, (unsigned) s->rules.is_teams,
                       (unsigned) s->rules.x9, (int) s->rules.xA, (int) s->rules.item_freq, (unsigned) s->rules.xD,
                       (unsigned) s->rules.stkind, (unsigned) s->rules.time_limit);
                for (k = 0; k < 3; k++) {
                    PlayerInitData* p = &s->players[k];
                    printf("|p%d=%d,%u,%d,%u,%u,%d,%d,%u,%d,%u,%u,%u,%u,%u,%u,%u,%u", k, (int) p->ckind, (unsigned) p->slot_type, (int) p->stocks,
                           (unsigned) p->color, (unsigned) p->slot, (int) p->spawn_pos, (int) p->spawn_dir, (unsigned) p->sub_color,
                           (int) p->handicap, (unsigned) p->team, (unsigned) p->nametag, (unsigned) p->xB, (unsigned) p->cpu_kind,
                           (unsigned) p->cpu_level, (unsigned) p->damage, (unsigned) p->damage1, (unsigned) p->hp);
                }
                printf("\n");
            } else if (f == 2) {
                onExitVs(&SCENE);
                print_log();
                printf("\n");
            } else if (f == 3) {
                u32 a0 = rnd();
                s32 a1 = (rnd() % 3 == 0) ? 0 : (s32) (rnd() % 5) + 1;
                u32 a2 = rnd();
                u32 a3 = rnd();
                void** r = (void**) gm_801BFC60(a0, a1, a2, a3, &SLOT);
                printf("|arg=%u,%d,%u,%u|ret=%s", (unsigned) a0, (int) a1, (unsigned) a2, (unsigned) a3,
                       r == &SLOT ? "A" : idp(r));
                print_log();
                printf("|slot=%s", SLOT == (void*) &CH ? "CH" : idp(SLOT));
                print_pool();
                printf("\n");
            } else if (f == 4) {
                gm_ModeState_Prize_OnEnter(&SCENE);
                print_log();
                print_pool();
                printf("|flags=%08x|t970=%08x\n", (unsigned) fnv_bytes(gm_8049E558, sizeof gm_8049E558),
                       (unsigned) fnv_bytes((u8*) g_t970, sizeof g_t970));
            } else if (f == 5) {
                onExitPrize(&SCENE);
                print_log();
                printf("\n");
            }
        }
    }
    return 0;
}
