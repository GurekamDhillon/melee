/* C reference for rust/melee_modes/src/tu/gm_1B03.rs: the game's gm_1B03.c verbatim (with the static
 * SceneLaunch_* helpers of gmscenelaunch.h it compiles in); gm_InitVsMode, the scene-launch queries and the
 * costume tables are stubbed here. Every function runs on a seeded StartMeleeData / CSSData / MatchEnd and
 * the touched fields are printed after the call.
 * Run: python rust/melee_modes/gen/turef/tu_shard537_gm.py tu_gm_1B03 */
#include "../../../../src/melee/gm/gm_1B03.c"
#include "tu_gm_common.h"

static int g_active, g_etc, g_variant, g_calls;

int SceneLaunch_Active(void) { return g_active; }
int SceneLaunch_EnemyTeamColors(void) { return g_etc; }
u8 gm_801692BC(u8 c)
{
    int n = g_calls++;
    return (u8) (c * 7 + 3 + ((g_variant & 1) ? (n % 2) : 0));
}
u8 gm_80169290(u8 c) { return (u8) (c * 5 + 1); }
u8 gm_80169264(u8 c) { return (u8) (c * 3 + 2); }
void gm_InitVsMode(VsModeData* vs)
{
    lg("init;");
    memset(vs, 0x11, sizeof *vs);
    /* bitfield storage is compiler-specific: put them in a known state through the members */
    vs->start.rules.x0_3 = 5;
    vs->start.rules.match_kind = 3;
    vs->start.rules.timer_enabled = 1;
    vs->start.rules.x2_5 = 0;
}

static const char* FUNC_NAMES[] = {"gm_SetupSubColors", "SceneLaunch_SixPresentationTest", "gm_SetupSuddenDeath",
                                   "gm_801B05F4",       "gm_SetupHumanPlayer",             "gm_SetupCpuPlayer",
                                   "gm_801B06B0",       "gm_801B0730",                     "gm_801B07B4",
                                   "gm_801B07E8"};
#define NFUNCS ((int) (sizeof FUNC_NAMES / sizeof FUNC_NAMES[0]))
#define NCASES 400

static StartMeleeData ST;
static CSSData CSSD;
static MatchEnd END;

static const s8 CK[] = {2, 0x12, 0x13, 5, 0, 0x12};

static void fill_start(StartMeleeData* s)
{
    int i;
    u32 r;
    memset(s, 0, sizeof *s);
    r = rnd();
    s->rules.is_teams = (u8) (r % 3);
    r = rnd();
    s->rules.x0_3 = r % 8;
    r = rnd();
    s->rules.match_kind = r % 8;
    r = rnd();
    s->rules.timer_enabled = r % 2;
    r = rnd();
    s->rules.x2_5 = r % 2;
    for (i = 0; i < 6; i++) {
        PlayerInitData* p = &s->players[i];
        r = rnd();
        p->slot_type = (u8) (r % 5);
        r = rnd();
        p->ckind = CK[r % 6];
        r = rnd();
        p->team = (u8) (r % 3);
        r = rnd();
        p->sub_color = (u8) (r % 3);
        r = rnd();
        p->stocks = (s8) (r % 6) - 1;
        r = rnd();
        p->color = (u8) (r % 5);
        r = rnd();
        p->cpu_level = (u8) (r % 9);
        r = rnd();
        p->nametag = (u8) (r % 200);
        r = rnd();
        p->damage1 = (u16) (r % 400);
        r = rnd();
        p->slot = (u8) (r % 7);
    }
}

static void dump_start(const char* tag, StartMeleeData* s)
{
    int i;
    printf("|%s=%u,%u,%u,%u,%u", tag, (unsigned) s->rules.is_teams, (unsigned) s->rules.x0_3,
           (unsigned) s->rules.match_kind, (unsigned) s->rules.timer_enabled, (unsigned) s->rules.x2_5);
    for (i = 0; i < 6; i++) {
        PlayerInitData* p = &s->players[i];
        printf(";%u,%d,%d,%u,%u,%u,%u,%u,%u,%u,%u", (unsigned) p->slot_type, (int) p->ckind, (int) p->stocks,
               (unsigned) p->color, (unsigned) p->slot, (unsigned) p->sub_color, (unsigned) p->team,
               (unsigned) p->nametag, (unsigned) p->cpu_level, (unsigned) p->damage1, (unsigned) p->xB);
    }
}

static void setup_case(void)
{
    int i;
    u32 r;
    fill_start(&ST);
    memset(&CSSD, 0, sizeof CSSD);
    fill_start(&CSSD.vs.start);
    r = rnd();
    CSSD.unk_0x0 = (u16) (r % 6 + 1);
    r = rnd();
    CSSD.match_type = (u8) (r % 5);
    memset(&END, 0, sizeof END);
    r = rnd();
    END.outcome = (u8) (r % 3);
    r = rnd();
    END.match_kind = (u8) (r % 3);
    r = rnd();
    END.is_teams = (u8) (r % 3);
    r = rnd();
    END.frame_count = r % 50;
    for (i = 0; i < 6; i++) {
        r = rnd();
        END.player_standings[i].ckind = CK[r % 6];
        r = rnd();
        END.player_standings[i].ftkind = (s8) (r % 9);
        r = rnd();
        END.player_standings[i].stocks = (s8) (r % 5) - 1;
        r = rnd();
        END.player_standings[i].x28 = r % 60;
        r = rnd();
        END.player_standings[i].is_big_loser = (u8) (r % 2);
        r = rnd();
        END.player_standings[i].team = (u8) (r % 5);
    }
    for (i = 0; i < 5; i++) {
        r = rnd();
        END.team_standings[i].is_big_loser = (u8) (r % 2);
    }
    r = rnd();
    g_active = (int) (r % 4 != 0);
    r = rnd();
    g_etc = (int) (r % 4 != 0);
    r = rnd();
    g_variant = (int) r;
    g_calls = 0;
}

int main(void)
{
    int f;
    int i;
    for (f = 0; f < NFUNCS; f++) {
        for (i = 0; i < NCASES; i++) {
            int ret = 0;
            s8 oc = 0x55;
            u8 os = 0x55;
            u8 ocol = 0x55;
            u8 on = 0x55;
            u8 ol = 0x55;
            s8 os8 = 0x55;
            s8 ocol8 = 0x55;
            s8 on8 = 0x55;
            u32 mask;
            int slot;
            u32 a;
            u32 b;
            u32 c;
            u32 d;
            setup_case();
            mask = rnd();
            slot = (int) (rnd() % 6);
            a = rnd();
            b = rnd();
            c = rnd();
            d = rnd();
            g_loglen = 0;
            g_log[0] = 0;
            switch (f) {
            case 0: gm_SetupSubColors(&ST); break;
            case 1: ret = SceneLaunch_SixPresentationTest((int) (a & 1)); break;
            case 2: gm_SetupSuddenDeath(&ST, &END); break;
            case 3: gm_801B05F4(&ST.players[slot], slot); break;
            case 4: gm_SetupHumanPlayer(&ST.players[slot], (u8) a, (u8) b, (u8) c, (u8) (d % 6)); break;
            case 5: gm_SetupCpuPlayer(&ST.players[slot], (u8) a, (u8) b, (u8) c, (u8) (d % 6)); break;
            case 6: gm_801B06B0(&CSSD, (u8) a, (s8) b, (s8) c, (s8) d, (u8) (a >> 8), (u8) (b >> 8), (u8) slot); break;
            case 7:
                gm_801B0730(&CSSD, (mask & 1) ? &oc : NULL, (mask & 2) ? &os : NULL, (mask & 4) ? &ocol : NULL,
                            (mask & 8) ? &on : NULL, (mask & 16) ? &ol : NULL);
                break;
            case 8: gm_801B07B4(&CSSD, (s8) a, (s8) b, (s8) c, (u8) d, (u8) (a >> 8), (u8) (b >> 8)); break;
            default:
                gm_801B07E8(&CSSD, (mask & 1) ? &oc : NULL, (mask & 2) ? &os8 : NULL, (mask & 4) ? &ocol8 : NULL,
                            (mask & 8) ? &on8 : NULL, (mask & 16) ? &ol : NULL);
                break;
            }
            printf("%d.%d %s|log=%s|ret=%d|out=%d,%u,%u,%u,%u,%d,%d,%d|css=%u,%u", f, i, FUNC_NAMES[f], g_log, ret, (int) oc,
                   (unsigned) os, (unsigned) ocol, (unsigned) on, (unsigned) ol, (int) os8, (int) ocol8, (int) on8,
                   (unsigned) CSSD.unk_0x0, (unsigned) CSSD.match_type);
            dump_start("st", &ST);
            dump_start("cs", &CSSD.vs.start);
            printf("\n");
        }
    }
    return 0;
}
