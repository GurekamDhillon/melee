/* C reference for rust/melee_modes/src/tu/gm_1A3F.rs: the game's gm_1A3F.c verbatim (so state_machine and the
 * static helpers are the game's own); every callee is stubbed here (logged; the Rust hooks log the same text),
 * the frame loop (gm_801A4D34) is a scripted stub, and gm_801A4510's endless loop is left with __builtin_longjmp.
 * Every case builds a random mode table (modes, states, scenes), runs one function and prints the log, the
 * return value and the whole state machine.
 * Run: python rust/melee_modes/gen/turef/tu_shard537_gm.py tu_gm_1A3F */
#include "../../../../src/melee/gm/gm_1A3F.c"
#include "tu_gm_common.h"

struct gmMainLib_8046B0F0_t gmMainLib_8046B0F0;
u16 db_gameLaunchButtonState;

#define GM_FRONTEND_K 46
#define NMODES 7

static GameMode MODES[NMODES + 1];
static GameModeState STATES[NMODES][6];
static int DAT_[4];
static GameScene SCENES[4];
static GameScene STANDIN;
static PreloadedGameModeState PRELOAD;
static int g_script[16];
static int g_si;
static int g_ovr[4];
static int g_oi;
static int g_heap2, g_heap3, g_pol1, g_pol2, g_stand, g_toyknown, g_probe, g_barrier, g_dvd, g_cnt11, g_busy, g_dtv, g_prog;
static int g_route, g_loads, g_maxloads, g_escape;
static void* g_jb[5];

void memzero(void* mem, ssize_t size) { memset(mem, 0, (size_t) size); }
static void se_a(GameModeState* s) { lg("seA(%d);", (int) s->id); }
static void se_b(GameModeState* s) { lg("seB(%d);", (int) s->id); }
static void sx_a(GameModeState* s) { lg("sxA(%d);", (int) s->id); }
static void sx_b(GameModeState* s) { lg("sxB(%d);", (int) s->id); }
static void ld_a(void) { lg("ldA;"); }
static void ld_b(void) { lg("ldB;"); }
static void ud_a(void) { lg("udA;"); }
static void ud_b(void) { lg("udB;"); }
static void in_a(void) { lg("inA;"); }
static void in_b(void) { lg("inB;"); }
static const char* dname(void* d)
{
    if (d == NULL) return "null";
    if (d == &DAT_[0]) return "d0";
    if (d == &DAT_[1]) return "d1";
    if (d == &DAT_[2]) return "d2";
    return "d3";
}
static void scn_enter(void* d) { lg("scE(%s);", dname(d)); }
static void scn_exit(void* d) { lg("scX(%s);", dname(d)); }
static void stand_enter(void* d) { lg("stE(%s);", dname(d)); }
static void fr0(void) {}
static void fr1(void) {}
static void fr2(void) {}
static void fr3(void) {}
static void frs(void) {}
static void wrapped(void) {}
static unsigned char ovr(void)
{
    int v = g_ovr[g_oi++ % 4];
    lg("ovr=%d;", v);
    return (unsigned char) v;
}
static const char* fname(void (*f)(void))
{
    if (f == fr0) return "f0";
    if (f == fr1) return "f1";
    if (f == fr2) return "f2";
    if (f == fr3) return "f3";
    if (f == frs) return "fS";
    if (f == wrapped) return "wrap";
    return "?";
}

void lbDvd_80018CF4(int v) { lg("8CF4(%d);", v); }
PreloadedGameModeState* lbDvd_GetPreloadCacheScene(void) { lg("getcache;"); return &PRELOAD; }
void lbDvd_80018254(void) { lg("8254;"); }
void lbDvd_80018F58(bool v)
{
    lg("8F58(%d);", (int) v);
    if (g_escape && g_loads++ >= g_maxloads) {
        __builtin_longjmp(g_jb, 1);
    }
}
LbHeapStatus lbHeap_80015BB8(int i)
{
    lg("heap(%d);", i);
    return (LbHeapStatus) (i == 2 ? g_heap2 : g_heap3);
}
void lbCardNew_ForgetMemory(void) { lg("forget;"); }
void lbCardGame_Reset(void) { lg("cardreset;"); }
void lbSnap_8001E27C(void) { lg("snapE27C;"); }
void Toy_803127D4(void) { lg("toy27D4;"); }
void tyDisplay_8031C8B8(void) { lg("tyC8B8;"); }
void HSD_SisLib_803A6048(size_t n) { lg("6048(%x);", (unsigned) n); }
void lb_8001CDB4(void) { lg("CDB4;"); }
int lbCardNew_CompleteAllTasks(int r) { lg("all(%d);", r); return 0; }
enum_t lbCardNew_CompleteNextTask(void)
{
    int r = g_cnt11 > 0 ? 11 : 0;
    g_cnt11--;
    lg("next=%d;", r);
    return (enum_t) r;
}
void lbMthp_8001F800(void) { lg("mthp;"); }
void lbAudioAx_80027DBC(void) { lg("audio;"); }
void HSD_PadReset(void) { lg("padreset;"); }
bool HSD_DevComIsBusy(int i)
{
    int r = g_busy > 0;
    g_busy--;
    lg("busy(%d)=%d;", i, r);
    return r;
}
void HSD_VISetBlack(bool b) { lg("black(%d);", (int) b); }
BOOL DVDCheckDisk(void) { lg("dvd;"); return g_dvd; }
void OSResetSystem(int a, u32 b, BOOL c) { lg("osreset(%d,%u,%d);", a, (unsigned) b, (int) c); }
u32 VIGetDTVStatus(void) { lg("dtv;"); return g_dtv; }
u32 OSGetProgressiveMode(void) { lg("prog;"); return g_prog; }
GameScene* gm_FindGameSceneHandler(u8 k) { lg("find(%d);", (int) k); return &SCENES[k % 4]; }
void gm_801A4BD4(void) { lg("4BD4;"); }
void gm_801A4B88(GameSceneInfo* i) { lg("4B88(%d);", (int) i->scene_kind); }
void gmMainLib_8015FBA4(void) { lg("FBA4;"); }
GameMode* gm_GetAllGameModes(void) { return MODES; }
void gm_801A3EF4(void) { lg("3EF4;"); }
void SceneReport_State(int phase, int mode, int id, int kind) { lg("rep(%d,%d,%d,%d);", phase, mode, id, kind); }
int Ui_ScenePolicy(int k)
{
    int r = (k % 2) ? g_pol1 : g_pol2;
    lg("policy(%d)=%d;", k, r);
    return r;
}
GameScene* gmFrontend_AtlasStandIn(u8 k)
{
    lg("standin(%d);", (int) k);
    return g_stand ? &STANDIN : NULL;
}
void Ui_ScenePolicyMask(int k, int known) { lg("mask(%d,%d);", k, known); }
int Fad_ToyWindowKnown(u8 k) { lg("known(%d);", (int) k); return g_toyknown; }
void FighterControls_MatchClear(void) { lg("fcclear;"); }
int Ui_ToyProbe(void) { lg("probe;"); return g_probe; }
void (*Fad_OverlayWrap(u8 kind, void (*inner)(void)))(void)
{
    lg("wrap(%d,%s);", (int) kind, fname(inner));
    return wrapped;
}
int Script_OnePBarrierPending(void) { lg("barrier;"); return g_barrier; }
void gm_OnePInterstage(void) { lg("interstage;"); }
u8 gmFrontend_Route(u8 from, u8 to)
{
    u8 r = g_route ? (u8) GM_FRONTEND_K : to;
    lg("route(%d,%d)=%d;", (int) from, (int) to, (int) r);
    return r;
}
u8 gmFrontend_ReportedMode(void) { lg("reported;"); return 17; }

static int script_next(void)
{
    if (g_si >= 16) {
        g_si++;
        return (g_si % 2 == 1) ? 1 : 7;
    }
    return g_script[g_si++];
}
void gm_801A4D34(void (*cb)(void), GameSceneInfo* info)
{
    int op;
    int arg;
    lg("4D34(%s,%d);", fname(cb), (int) info->scene_kind);
    op = script_next();
    arg = script_next();
    switch (op) {
    case 1: gm_ChangeGameModeAfterCurrentScene(MODES[arg % NMODES].kind); break;
    case 2:
        gm_SetPendingGameMode(MODES[arg % NMODES].kind);
        gm_SetNewGameModePending();
        break;
    case 3: gm_SetPendingGameMode(MODES[arg % NMODES].kind); break;
    case 4: gm_SetNextGameModeStateId((u8) (arg % 3)); break;
    case 5: gm_SetGameModeStateId((u8) (arg % 4)); break;
    case 6:
        gmMainLib_8046B0F0.resetting = 1;
        break;
    case 7: gm_SetGameModeOverride(ovr); break;
    case 8: gm_SetGameModeOverride(NULL); break;
    default: break;
    }
}

static const char* FUNC_NAMES[] = {"preloadState", "firstState",   "nextState",   "findState",
                                   "findMode",     "gm_801A4014",  "EnterData",   "ExitData",
                                   "SetGameModeStateId", "SetNextGameModeStateId", "GetPreviousSceneIndex",
                                   "GetCurrentSceneIndex", "SetNewGameModePending", "SetPendingGameMode",
                                   "ChangeGameModeAfterCurrentScene", "GetCurrentGameMode", "GetPreviousGameMode",
                                   "SetGameModeOverride", "Is1PMode", "runGameMode", "gm_801A4510"};
#define NFUNCS ((int) (sizeof FUNC_NAMES / sizeof FUNC_NAMES[0]))
static const int NCASES[] = {60, 60, 100, 100, 80, 120, 20, 20, 30, 30, 20, 20, 20, 30, 30, 20, 20, 30, 60, 250, 150};

static const int KPOOL[] = {3, 7, 12, 21, 28, 33};
static const int SKPOOL[] = {GS_STAFFROLL, GS_RESULTS, GS_CSS, 10, 20, 30};

static void build_world(void)
{
    int i;
    int k;
    u32 r;
    int a;
    int b;
    int c;
    memset(MODES, 0, sizeof MODES);
    memset(STATES, 0, sizeof STATES);
    for (i = 0; i < 4; i++) DAT_[i] = i;
    for (k = 0; k < 4; k++) {
        memset(&SCENES[k], 0, sizeof SCENES[k]);
        SCENES[k].kind = (u8) k;
        SCENES[k].on_frame = k == 0 ? fr0 : (k == 1 ? fr1 : (k == 2 ? fr2 : fr3));
        SCENES[k].on_enter = k == 3 ? NULL : scn_enter;
        SCENES[k].on_exit = k == 2 ? NULL : scn_exit;
    }
    memset(&STANDIN, 0, sizeof STANDIN);
    STANDIN.kind = 99;
    STANDIN.on_frame = frs;
    STANDIN.on_enter = stand_enter;
    STANDIN.on_exit = NULL;
    r = rnd();
    a = (int) (r % 6);
    r = rnd();
    b = (a + 1 + (int) (r % 5)) % 6;
    c = b;
    while (c == a || c == b) {
        r = rnd();
        c = (int) (r % 6);
    }
    MODES[0].kind = GM_BOOT;
    MODES[1].kind = GM_PROGRESSIVE_SCAN;
    MODES[2].kind = (u8) KPOOL[a];
    MODES[3].kind = (u8) KPOOL[b];
    MODES[4].kind = (u8) KPOOL[c];
    MODES[5].kind = GM_FRONTEND_K;
    MODES[6].kind = 0; /* a restored backup routing can leave pending_mode at 0 */
    MODES[NMODES].kind = GM_COUNT;
    for (i = 0; i < NMODES; i++) {
        int n;
        int id;
        r = rnd();
        MODES[i].preloaded = (u8) (r % 2);
        r = rnd();
        MODES[i].on_load = r % 3 == 0 ? NULL : (r % 3 == 1 ? ld_a : ld_b);
        r = rnd();
        MODES[i].on_unload = r % 3 == 0 ? NULL : (r % 3 == 1 ? ud_a : ud_b);
        r = rnd();
        MODES[i].on_init = r % 3 == 0 ? NULL : (r % 3 == 1 ? in_a : in_b);
        MODES[i].states = STATES[i];
        r = rnd();
        n = 1 + (int) (r % 4);
        id = 0;
        for (k = 0; k < n; k++) {
            GameModeState* s = &STATES[i][k];
            r = rnd();
            id = (k == 0) ? (int) (r % 2) : id + 1 + (int) (r % 2);
            if (k == n - 1 && id < 4) {
                id = 4;
            }
            s->id = (u8) id;
            r = rnd();
            s->preload = (u8) (r % 5);
            r = rnd();
            s->flags = (u16) r;
            r = rnd();
            s->on_enter = r % 3 == 0 ? NULL : (r % 3 == 1 ? se_a : se_b);
            r = rnd();
            s->on_exit = r % 3 == 0 ? NULL : (r % 3 == 1 ? sx_a : sx_b);
            r = rnd();
            s->info.scene_kind = (u8) SKPOOL[r % 6];
            r = rnd();
            s->info.enter_data = (r % 2) ? (void*) &DAT_[(r >> 4) % 4] : NULL;
            r = rnd();
            s->info.exit_data = (r % 2) ? (void*) &DAT_[(r >> 4) % 4] : NULL;
        }
        STATES[i][n].id = 0xFF;
    }
}

/* gm_801A4510 never returns: lbDvd_80018F58 leaves it with __builtin_longjmp after g_maxloads mode loads. The
 * jump target lives in this separate function so no caller register is live across it. */
static __attribute__((noinline)) void run_main_loop(void)
{
    g_escape = 1;
    if (__builtin_setjmp(g_jb) == 0) {
        gm_801A4510();
    }
    g_escape = 0;
}

static int mode_index(GameMode* m) { return m == NULL ? -1 : (int) (m - MODES); }
static int state_index(GameModeState* s)
{
    int i;
    int k;
    if (s == NULL) return -1;
    for (i = 0; i < NMODES; i++) {
        for (k = 0; k < 6; k++) {
            if (s == &STATES[i][k]) return i * 10 + k;
        }
    }
    return -2;
}

int main(void)
{
    int f;
    int i;
    for (f = 0; f < NFUNCS; f++) {
        for (i = 0; i < NCASES[f]; i++) {
            int ret = 0;
            int k;
            int mi;
            int si;
            int nst;
            u32 r;
            u32 a;
            GameMode* m;
            build_world();
            memset(&gmMainLib_8046B0F0, 0, sizeof gmMainLib_8046B0F0);
            g_si = 0;
            g_oi = 0;
            for (k = 0; k < 16; k += 2) {
                r = rnd();
                switch (r % 10) {
                case 0: case 1: case 2: g_script[k] = 0; break;
                case 3: g_script[k] = 1; break;
                case 4: g_script[k] = 2; break;
                case 5: g_script[k] = 3; break;
                case 6: g_script[k] = 4; break;
                case 7: g_script[k] = 5; break;
                case 8: g_script[k] = 6; break;
                default: g_script[k] = (r >> 8) % 2 ? 7 : 8; break;
                }
                r = rnd();
                g_script[k + 1] = (int) (r % 64);
            }
            for (k = 0; k < 4; k++) {
                r = rnd();
                g_ovr[k] = (r % 3 == 0) ? (int) MODES[2 + (r >> 4) % 3].kind : GM_COUNT;
            }
            r = rnd();
            g_heap2 = (int) (r % 2);
            r = rnd();
            g_heap3 = (int) (r % 2);
            r = rnd();
            g_pol1 = (int) (r % 3);
            r = rnd();
            g_pol2 = (int) (r % 3);
            r = rnd();
            g_stand = (int) (r % 2);
            r = rnd();
            g_toyknown = (int) (r % 2);
            r = rnd();
            g_probe = (int) (r % 3 == 0);
            r = rnd();
            g_barrier = (int) (r % 2);
            r = rnd();
            g_dvd = (int) (r % 2);
            r = rnd();
            g_cnt11 = (int) (r % 3);
            r = rnd();
            g_busy = (int) (r % 3);
            r = rnd();
            g_dtv = (int) (r % 2);
            r = rnd();
            g_prog = (int) (r % 2);
            r = rnd();
            g_route = (int) (r % 3 == 0);
            r = rnd();
            db_gameLaunchButtonState = (u16) ((r % 2) ? HSD_PAD_B : 0);
            r = rnd();
            PRELOAD.is_heap_persistent[0] = (int) (r % 2);
            r = rnd();
            PRELOAD.is_heap_persistent[1] = (int) (r % 2);
            memset(&state_machine, 0, sizeof state_machine);
            r = rnd();
            state_machine.routing.curr_mode = (u8) (r % 47);
            r = rnd();
            state_machine.routing.pending_mode = (u8) (r % 47);
            r = rnd();
            state_machine.routing.prev_mode = (u8) (r % 47);
            r = rnd();
            state_machine.routing.curr_state_id = (u8) (r % 5);
            r = rnd();
            state_machine.routing.prev_state_id = (u8) (r % 5);
            r = rnd();
            state_machine.routing.next_state_id = (u8) ((r % 4 == 0) ? (r >> 4) % 3 : 0);
            r = rnd();
            state_machine.get_override = (r % 3 == 0) ? ovr : NULL;
            g_loglen = 0;
            g_log[0] = 0;
            g_escape = 0;
            g_loads = 0;
            g_maxloads = 3;
            r = rnd();
            mi = (int) (r % NMODES);
            r = rnd();
            si = (int) (r % 6);
            a = rnd();
            for (nst = 0; STATES[mi][nst].id != 0xFF; nst++) {
            }
            si = si % nst;
            m = &MODES[mi];
            switch (f) {
            case 0: preloadState(&STATES[mi][si]); break;
            case 1: ret = firstState(STATES[mi], (u8) a); break;
            case 2: ret = nextState(STATES[mi]); break;
            case 3: ret = state_index(findState(STATES[mi])); break;
            case 4: ret = mode_index(findMode((u8) (a % 48))); break;
            case 5: gm_801A4014(m); break;
            case 6: ret = (int) (gm_GetGameModeStateEnterData(&STATES[mi][si]) == NULL ? -1 : *(int*) gm_GetGameModeStateEnterData(&STATES[mi][si])); break;
            case 7: ret = (int) (gm_GetGameModeStateExitData(&STATES[mi][si]) == NULL ? -1 : *(int*) gm_GetGameModeStateExitData(&STATES[mi][si])); break;
            case 8: gm_SetGameModeStateId((u8) a); break;
            case 9: gm_SetNextGameModeStateId((u8) a); break;
            case 10: ret = gm_GetPreviousSceneIndex(); break;
            case 11: ret = gm_GetCurrentSceneIndex(); break;
            case 12: gm_SetNewGameModePending(); break;
            case 13: gm_SetPendingGameMode((u8) a); break;
            case 14: gm_ChangeGameModeAfterCurrentScene((int) (a % 64)); break;
            case 15: ret = gm_GetCurrentGameMode(); break;
            case 16: ret = gm_GetPreviousGameMode(); break;
            case 17: gm_SetGameModeOverride((a % 2) ? ovr : NULL); break;
            case 18: ret = gm_Is1PMode((u8) (a % 50)); break;
            case 19: ret = runGameMode(m->kind); break;
            default:
                run_main_loop();
                break;
            }
            printf("%d.%d %s|log=%s|ret=%d|sm=", f, i, FUNC_NAMES[f], g_log, ret);
            printf("%u,%u,%u,%u,%u,%u,%u,%u,%u,%u,%u,%u,%u,%d", (unsigned) state_machine.routing.curr_mode,
                   (unsigned) state_machine.routing.pending_mode, (unsigned) state_machine.routing.prev_mode,
                   (unsigned) state_machine.routing.curr_state_id, (unsigned) state_machine.routing.prev_state_id,
                   (unsigned) state_machine.routing.next_state_id, (unsigned) state_machine.backup_routing.curr_mode,
                   (unsigned) state_machine.backup_routing.pending_mode, (unsigned) state_machine.backup_routing.prev_mode,
                   (unsigned) state_machine.backup_routing.curr_state_id, (unsigned) state_machine.backup_routing.prev_state_id,
                   (unsigned) state_machine.backup_routing.next_state_id, (unsigned) state_machine.pending_mode_change,
                   state_machine.get_override == NULL ? 0 : (state_machine.get_override == ovr ? 1 : 2));
            printf("|pre=%d,%d|si=%d,%d|rs=%d,%d,%d,%d\n", PRELOAD.is_heap_persistent[0], PRELOAD.is_heap_persistent[1],
                   g_si, g_oi, (int) gmMainLib_8046B0F0.resetting, (int) gmMainLib_8046B0F0.skip_intro, g_loads, g_cnt11);
        }
    }
    return 0;
}
