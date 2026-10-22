/* C reference for rust/melee_modes/src/tu/gmboot.rs: the game's gmboot.c verbatim, with the boot
 * scene's callees stubbed (logged; the Rust hooks log the same text) and gmMainLib defined here.
 * Run: python rust/melee_chara/gen/turef/tu_shard457.py tu_gmboot (gmboot only). */
#include "../../../../src/melee/gm/gmboot.c"
#include "tu_gm_common.h"

struct gmMainLib_8046B0F0_t gmMainLib_8046B0F0;

static int g_toy_owned, g_pikmin_ret, g_lang, g_launch, g_probe_present, g_probe_ok;
static char g_archive_sentinel;

/* gm_1A3F.c: returns the scene's enter / exit data (the real accessors). */
void* gm_GetGameModeStateEnterData(GameModeState* s) { return s->info.enter_data; }
void* gm_GetGameModeStateExitData(GameModeState* s) { return s->info.exit_data; }

void gm_801BF708(s8 a) { lg("801BF708(%d)", (int) a); }
s32 Toy_803048C0(int id) { lg("toy48C0(%d)", id); return g_toy_owned; }
void Toy_803124BC(void) { lg("toy124BC"); }
void Toy_SetUnlockState(enum_t id, bool v) { lg("unlock(%d,%d)", (int) id, (int) v); }
int lb_8001C2D8(int chan, const char* company, const char* game_name, const char* filename)
{
    lg("cardprobe(%d,%s,%s,%s)", chan, company, game_name, filename);
    return g_pikmin_ret;
}
int lbLang_GetLanguageSetting(void) { lg("lang"); return g_lang; }
void gm_ChangeGameModeAfterCurrentScene(int pending_mode) { lg("chg(%d)", pending_mode); }
void gm_SetGameModeOverride(u8 (*fn)(void)) { lg("override(%s)", fn == lbCardGame_DecideGameMode ? "dec" : "?"); }
void gm_SetPendingGameMode(u8 pending_mode) { lg("pend(%u)", (unsigned) pending_mode); }
u8 lbCardGame_DecideGameMode(void) { return 0; }
int SceneLaunch_BootGameMode(void) { lg("launch"); return g_launch; }
const char* ContentProbeName(void) { lg("probename"); return g_probe_present ? "probe.dat" : NULL; }
void ContentProbeResult(const char* name, void* archive) { lg("probeRes(%s,%d)", name, archive != NULL); }
void* lbArchive_LoadArchive(const char* filename)
{
    lg("probeload(%s)", filename);
    return g_probe_ok ? (void*) &g_archive_sentinel : NULL;
}

static const char* FUNC_NAMES[] = {"bootOnLoad", "bootOnLeave", "memcardOnLoad"};
#define NFUNCS ((int) (sizeof FUNC_NAMES / sizeof FUNC_NAMES[0]))
#define NCASES 200

static struct loadData LOAD;
static struct exitData LEAVE;
static GameModeState SCENE;

static void setup_case(void)
{
    LOAD.x0 = rnd();
    LOAD.x4 = (u8) rnd();
    LOAD.mode_id = (u8) rnd();
    LEAVE.x0 = rnd();
    LEAVE.mode_id = (u8) rnd();
    gmMainLib_8046B0F0.skip_intro = rnd() & 1;
    g_toy_owned = rnd() & 1;
    g_pikmin_ret = rnd() & 1;
    g_lang = (int) (rnd() % 3);
    g_launch = (int) (rnd() % 4) - 1;
    g_probe_present = rnd() & 1;
    g_probe_ok = rnd() & 1;
    memset(&SCENE, 0, sizeof SCENE);
    SCENE.info.enter_data = &LOAD;
    SCENE.info.exit_data = &LEAVE;
}

static void fingerprint(int f, int i)
{
    printf("%d.%d %s|log=%s|load=%u,%u,%u|leave=%u,%u|skip=%d\n", f, i, FUNC_NAMES[f], g_log, (unsigned) LOAD.x0,
           (unsigned) LOAD.x4, (unsigned) LOAD.mode_id, (unsigned) LEAVE.x0, (unsigned) LEAVE.mode_id,
           (int) gmMainLib_8046B0F0.skip_intro);
}

int main(void)
{
    int f;
    int i;
    for (f = 0; f < NFUNCS; f++) {
        for (i = 0; i < NCASES; i++) {
            setup_case();
            g_loglen = 0;
            g_log[0] = 0;
            if (f == 0) {
                bootOnLoad(&SCENE);
            } else if (f == 1) {
                bootOnLeave(&SCENE);
            } else {
                memcardOnLoad(&SCENE);
            }
            fingerprint(f, i);
        }
    }
    return 0;
}
