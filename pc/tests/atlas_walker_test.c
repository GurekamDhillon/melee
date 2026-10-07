/* atlas-walker: the table walker (gmfrontend_atlas_table.h): one FrontendScreen becomes the rows of one tab, and the host's events become the legacy rules.
 * Each check is a branch of the legacy code, named for its source (gmfrontend.c fe_change, the Confirm and Back branches of gm_Scene_Frontend_OnFrame). */
#include "atlas_check.h"
#include "../../src/melee/gm/gmfrontend_items.h"
#include "atlas_settings_stub.h"
#include "../../src/melee/gm/gmfrontend_atlas_table.h"

static int g_val, g_sets, g_calls, g_en = 1, g_vis = 1, g_last;
static int get_v(void) { return g_val; }
static void set_v(int v) { g_val = v; g_sets++; g_last = v; }
static void call_it(void) { g_calls++; }
static int en(void) { return g_en; }
static int vis(void) { return g_vis; }
static void fmt_pct(int v, char *o) { o[0] = (char) ('0' + v / 10); o[1] = (char) ('0' + v % 10); o[2] = '%'; o[3] = 0; }
static const char *const OPTS[] = { "Off", "FPS", "Performance" };

static const FrontendItem T[] = {
    /* 0 */ { FE_TOGGLE, 0, "VSync", "Wait for the display.", get_v, set_v, 0, 1, 1 },
    /* 1 */ { FE_SLIDER, 0, "Master Volume", "Everything.", get_v, set_v, 0, 100, 5, NULL, fmt_pct },
    /* 2 */ { FE_CHOICE, 0, "Show FPS", "Readout.", get_v, set_v, 0, 2, 1, OPTS },
    /* 3 */ { FE_SLIDER, 0, "Port 1", "Live.", get_v, NULL, 0, 0, 0, NULL, fmt_pct },                       /* a readout: a slider with no set */
    /* 4 */ { FE_ACTION, FE_DO_CALL, "Recalibrate", "Adapter.", NULL, NULL, 0, 0, 0, NULL, fmt_pct, NULL, call_it },
    /* 5 */ { FE_ACTION, FE_DO_CALL, "Bind", "Capture.", NULL, NULL, 0, 0, 0, NULL, NULL, NULL, call_it, en, "Connect a controller first." },
    /* 6 */ { FE_SLIDER, 0, "Stick Dead Zone", "SDL only.", get_v, set_v, 0, 40, 2, NULL, NULL, vis },
    /* 7 */ { FE_ACTION, FE_DO_CONTINUE, "Continue", "On." },
    /* 8 */ { FE_ACTION, FE_DO_BACK, "Done", "Back." },
};
static const FrontendScreen S = { "SETTINGS", "VIDEO", T, 9, 0 };

static void visible_and_ids(void)
{
    FssVis v;
    g_vis = 0; fss_visible(&S, &v);
    CHECK(v.n == 8 && v.idx[6] == 7);                                  /* row 6 hidden: slots shift, table indices stay */
    stub_reset(); fss_table_submit(1, &S, &v);
    CHECK(REC_ROWS.n == 8 && strcmp(REC_ROWS.r[6].id, "i7") == 0);     /* ids are TABLE indices: stable when rows show or hide (focus refocuses by id) */
    CHECK(strcmp(REC_ROWS.r[0].id, "i0") == 0 && strcmp(REC_ROWS.r[5].id, "i5") == 0);
    g_vis = 1; fss_visible(&S, &v); CHECK(v.n == 9);
    stub_reset(); fss_table_submit(1, &S, &v); CHECK(strcmp(REC_ROWS.r[6].id, "i6") == 0 && strcmp(REC_ROWS.r[8].id, "i8") == 0);
}
static void kinds_and_text(void)
{
    FssVis v;
    g_val = 45; fss_visible(&S, &v); stub_reset(); fss_table_submit(1, &S, &v);
    CHECK(REC_ROWS.r[0].vkind == STUB_TOGGLE && REC_ROWS.r[0].value == 1);                                                /* any non-zero is on */
    CHECK(REC_ROWS.r[1].vkind == STUB_SLIDER && REC_ROWS.r[1].vmax == 100 && REC_ROWS.r[1].vstep == 5 && strcmp(REC_ROWS.r[1].text, "45%") == 0 && REC_ROWS.r[1].value == 45);
    CHECK(REC_ROWS.r[1].flags & STUB_A_STEPS);                                                                            /* A steps a slider: the legacy Confirm branch */
    CHECK(REC_ROWS.r[2].vkind == STUB_CHOICE && REC_ROWS.r[2].n_opts == 3 && strcmp(REC_ROWS.r[2].opt[1], "FPS") == 0 && strcmp(REC_ROWS.r[2].opt[2], "Performance") == 0);
    CHECK(REC_ROWS.r[3].vkind == STUB_TEXT && (REC_ROWS.r[3].flags & STUB_RO) && strcmp(REC_ROWS.r[3].text, "45%") == 0);   /* set == NULL: a readout (the kit's FKW_READOUT rule) */
    CHECK(REC_ROWS.r[4].vkind == STUB_TEXT && strcmp(REC_ROWS.r[4].text, "00%") == 0 && !(REC_ROWS.r[4].flags & STUB_RO));  /* an action with a status shows it, and A still acts */
    CHECK(REC_ROWS.r[7].vkind == STUB_NONE);                                                                               /* an action without one */
    CHECK(strcmp(REC_ROWS.r[0].help, "Wait for the display.") == 0 && strcmp(REC_ROWS.r[0].label, "VSync") == 0);
    CHECK((REC_ROWS.r[5].flags & STUB_DISABLED) == 0 && g_en == 1);
    g_en = 0; stub_reset(); fss_table_submit(1, &S, &v);
    CHECK((REC_ROWS.r[5].flags & STUB_DISABLED) && strcmp(REC_ROWS.r[5].reason, "Connect a controller first.") == 0);       /* disabled has words, not only a dim */
    CHECK((REC_ROWS.r[4].flags & STUB_DISABLED) == 0);                                                                     /* only the row with the predicate */
    g_en = 1;
}
static void legacy_change(void)                                          /* fe_change, gmfrontend.c:1888 */
{
    FssVis v; unsigned fx; fss_visible(&S, &v);
    g_val = 95; g_sets = 0; fx = fss_table_event(&S, &v, 1, FSS_EV_CHANGE, +1);
    CHECK(g_last == 100 && g_sets == 1 && (fx & FSS_FX_MOVE) && (fx & FSS_FX_REBUILD));                                 /* step 5, clamps at 100 */
    g_sets = 0; fx = fss_table_event(&S, &v, 1, FSS_EV_CHANGE, +1);
    CHECK(g_sets == 0);                                                                                                    /* held at the end: nothing set ... */
    CHECK(fx & FSS_FX_BUMP);                                                                                               /* ... the kit's "can't go further" */
    g_val = 2; g_sets = 0; fss_table_event(&S, &v, 2, FSS_EV_CHANGE, +1); CHECK(g_last == 0);                              /* a choice wraps */
    g_val = 0; fss_table_event(&S, &v, 2, FSS_EV_CHANGE, -1); CHECK(g_last == 2);                                          /* both ways */
    g_val = 0; fss_table_event(&S, &v, 0, FSS_EV_CHANGE, -1); CHECK(g_last == 1);                                          /* a toggle flips from either direction */
    g_val = 1; fss_table_event(&S, &v, 0, FSS_EV_CHANGE, +1); CHECK(g_last == 0);
    g_val = 3; fss_table_event(&S, &v, 0, FSS_EV_CHANGE, +1); CHECK(g_last == 0);                                          /* any non-zero toggles off */
    g_sets = 0; fx = fss_table_event(&S, &v, 3, FSS_EV_CHANGE, +1); CHECK(g_sets == 0 && fx == 0);                         /* a readout never changes */
    fx = fss_table_event(&S, &v, 4, FSS_EV_CHANGE, +1); CHECK(g_sets == 0 && fx == 0);                                     /* left and right on an action: nothing */
    g_val = 38; fss_table_event(&S, &v, 6, FSS_EV_CHANGE, +1); CHECK(g_last == 40);                                        /* Stick Dead Zone, step 2 */
}
static void legacy_accept(void)                                          /* gm_Scene_Frontend_OnFrame, the Confirm branch (gmfrontend.c:2144) */
{
    FssVis v; unsigned fx; fss_visible(&S, &v);
    g_calls = 0; fx = fss_table_event(&S, &v, 4, FSS_EV_ACCEPT, 0);
    CHECK(g_calls == 1 && (fx & FSS_FX_FORWARD) && (fx & FSS_FX_REBUILD));                                                 /* FE_DO_CALL: call, rebuild */
    g_en = 0; g_calls = 0; fx = fss_table_event(&S, &v, 5, FSS_EV_ACCEPT, 0);
    CHECK(g_calls == 0 && (fx & FSS_FX_BACK) && (fx & FSS_FX_BUMP));                                                       /* a disabled row: no call, back sound, bump */
    g_en = 1; g_calls = 0; fx = fss_table_event(&S, &v, 5, FSS_EV_ACCEPT, 0); CHECK(g_calls == 1 && (fx & FSS_FX_FORWARD));  /* enabled again */
    fx = fss_table_event(&S, &v, 7, FSS_EV_ACCEPT, 0); CHECK((fx & FSS_FX_CONTINUE) && (fx & FSS_FX_FORWARD));
    fx = fss_table_event(&S, &v, 8, FSS_EV_ACCEPT, 0); CHECK((fx & FSS_FX_LEAVE) && (fx & FSS_FX_BACK));
    g_val = 10; g_sets = 0; fss_table_event(&S, &v, 1, FSS_EV_ACCEPT, 0); CHECK(g_last == 15);                              /* A on a slider steps +1 step: the legacy rule */
    fss_table_event(&S, &v, 0, FSS_EV_ACCEPT, 0); CHECK(g_sets == 2);                                                      /* A on a toggle flips */
    g_sets = 0; fx = fss_table_event(&S, &v, 3, FSS_EV_ACCEPT, 0); CHECK(g_sets == 0 && fx == 0);                          /* A on a readout: nothing */
    CHECK(fss_table_event(&S, &v, 99, FSS_EV_ACCEPT, 0) == 0 && fss_table_event(&S, &v, -1, FSS_EV_CHANGE, 1) == 0);       /* a stale or bad slot does nothing */
    CHECK(fss_table_event(&S, &v, 0, 99, 0) == 0);                                                                         /* an unknown event does nothing */
    fx = fss_table_event(&S, &v, -1, FSS_EV_BACK, 0); CHECK((fx & FSS_FX_BACK) && (fx & FSS_FX_LEAVE));                    /* Back: the caller decides what it means */
}
static void big_table(void)
{
    static FrontendItem M[41]; static FrontendScreen MS; FssVis v; int i;
    memset(M, 0, sizeof M); M[0].kind = FE_SLIDER; M[0].label = "Mods"; M[0].get = get_v; M[0].format = fmt_pct;
    for (i = 1; i < 41; i++) { M[i].kind = FE_TOGGLE; M[i].label = "mod"; M[i].get = get_v; M[i].set = set_v; M[i].max = 1; M[i].step = 1; }
    MS.items = M; MS.n_items = 41; fss_visible(&MS, &v); CHECK(v.n == 41);
    stub_reset(); fss_table_submit(1, &MS, &v); CHECK(REC_ROWS.n == 41);                                                   /* the MODS page fits (FSS_MAX 64, AT_MAX_ITEMS 64) */
    MS.n_items = 70; { static FrontendItem B[70]; int k; memset(B, 0, sizeof B); for (k = 0; k < 70; k++) { B[k].kind = FE_ACTION; B[k].label = "x"; } MS.items = B; }
    fss_visible(&MS, &v); CHECK(v.n == FSS_MAX);                                                                           /* a table past the cap is cut, never overrun */
}
static void options_beyond_the_record(void)
{
    static const char *const BIG[] = { "a", "b", "c", "d", "e", "f", "g", "h", "i", "j" };
    static const FrontendItem C[] = { { FE_CHOICE, 0, "Many", "Ten options.", get_v, set_v, 0, 9, 1, BIG } };
    static const FrontendScreen CS = { "T", "S", C, 1, 0 };
    FssVis v;
    g_val = 3; fss_visible(&CS, &v); stub_reset(); fss_table_submit(1, &CS, &v);
    CHECK(REC_ROWS.r[0].vkind == STUB_CHOICE && REC_ROWS.r[0].n_opts == 0 && strcmp(REC_ROWS.r[0].text, "d") == 0);        /* more than the record holds: the walker owns the text, the host shows it */
    g_val = 12; stub_reset(); fss_table_submit(1, &CS, &v); CHECK(strcmp(REC_ROWS.r[0].text, "") == 0);                    /* a value outside the options: empty, never a read past the array */
}
static void groups_and_reasons(void)
{
    static const FrontendItem G[] = {
        { FE_TOGGLE, 0, "A", "a", get_v, set_v, 0, 1, 1, NULL, NULL, NULL, NULL, NULL, NULL, "FIRST" },
        { FE_TOGGLE, 0, "B", "b", get_v, set_v, 0, 1, 1, NULL, NULL, NULL, NULL, NULL, NULL, "FIRST" },
        { FE_TOGGLE, 0, "C", "c", get_v, set_v, 0, 1, 1, NULL, NULL, NULL, NULL, NULL, NULL, "SECOND" },
        { FE_TOGGLE, 0, "D", NULL, get_v, set_v, 0, 1, 1 },
    };
    static const FrontendScreen GS = { "T", "S", G, 4, 0 };
    FssVis v;
    fss_visible(&GS, &v); stub_reset(); fss_table_submit(1, &GS, &v);
    CHECK(strcmp(REC_ROWS.r[0].group, "FIRST") == 0 && strcmp(REC_ROWS.r[2].group, "SECOND") == 0 && REC_ROWS.r[3].group[0] == 0);
    CHECK(strcmp(REC_ROWS.r[3].help, "") == 0);                                                                            /* no help: empty, never NULL */
}
static void calls_per_frame(void)
{
    FssVis v; int rows_calls;
    fss_visible(&S, &v); stub_reset(); fss_table_submit(7, &S, &v);
    CHECK(REC_ROWS.h == 7 && REC_ROWS.rows_calls == 1 && REC_ROWS.row_calls == v.n);                                       /* one SetRows and one SetRow per visible row */
    rows_calls = REC_ROWS.rows_calls; fss_table_submit(7, &S, &v); CHECK(REC_ROWS.rows_calls == rows_calls + 1);
}
static void hint_strings(void)                       /* the labels are true to what the legacy rule does: A steps a slider or flips a toggle */
{
    char o[96];
    fss_hints_for(FSS_VK_TOGGLE, 0, 1, o, sizeof o); CHECK_STR(o, "A:Change,L:Page,B:Back");
    fss_hints_for(FSS_VK_SLIDER, 0, 1, o, sizeof o); CHECK_STR(o, "A:Change,L:Page,B:Back");
    fss_hints_for(FSS_VK_CHOICE, 0, 1, o, sizeof o); CHECK_STR(o, "A:Change,L:Page,B:Back");
    fss_hints_for(FSS_VK_NONE, 0, 1, o, sizeof o);   CHECK_STR(o, "A:Select,L:Page,B:Back");       /* an action */
    fss_hints_for(FSS_VK_TEXT, 0, 1, o, sizeof o);   CHECK_STR(o, "L:Page,B:Back");                 /* a readout: nothing to press A for */
    fss_hints_for(FSS_VK_NONE, 1, 1, o, sizeof o);   CHECK_STR(o, "L:Page,B:Back");                 /* a disabled row offers no A */
    fss_hints_for(FSS_VK_TOGGLE, 1, 1, o, sizeof o); CHECK_STR(o, "L:Page,B:Back");
    fss_hints_for(FSS_VK_NONE, 0, 0, o, sizeof o);   CHECK_STR(o, "A:Select,B:Back");               /* no tabs: the how-to page, the erase screen */
    fss_hints_for(FSS_VK_TEXT, 0, 0, o, sizeof o);   CHECK_STR(o, "B:Back");
    fss_hints_for(FSS_VK_NONE, 0, 1, o, 8);          CHECK_STR(o, "A:Selec");                        /* a short buffer: cut and terminated, never overrun */
    fss_hints_for(FSS_VK_NONE, 0, 1, o, 1);          CHECK_STR(o, "");
    o[0] = 'x'; fss_hints_for(FSS_VK_NONE, 0, 1, o, 0); CHECK(o[0] == 'x');                           /* a zero buffer: nothing written */
}
int main(void) { visible_and_ids(); kinds_and_text(); legacy_change(); legacy_accept(); big_table(); options_beyond_the_record(); groups_and_reasons(); calls_per_frame(); hint_strings(); ATLAS_DONE("atlas walker"); }
