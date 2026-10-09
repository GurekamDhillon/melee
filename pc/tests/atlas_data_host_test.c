/* Host-side test of the DATA screens' door (gw_script_ui_data.inc, over the settings door gw_script_ui_set.inc): the Ui_Data* shims the game-side adapter calls, driven the way the adapter drives them, against
 * the real render, the real screen stack and a real Lua state for the owner checks. gw_script_ui.inc is included here against the same stand-ins atlas_binding_test.c
 * and atlas_select_adapter_test.c use (the script table, the kit draw calls). No game, no renderer, no window. Reaches the host statics directly, so no test accessors. */
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <windows.h>
#include "lua.h"
#include "lauxlib.h"
#include "lualib.h"
#include "gw_kit.h"
#include "gw_screen_model.h"
#include "atlas_check.h"
#include "atlas_fake.h"
#include "atlas_rec.h"

typedef struct { char id[64]; int used, disabled, gameplay; } GsScript;
static struct { lua_State *L; int cur, console, n, scene_kind, match_active; GsScript s[8]; unsigned char key_now[256]; } gs;
static int g_may_run = 1, g_quads, g_art_calls, g_art_ret = 5;
static double g_now = 1000.0;
static float g_track;
static unsigned g_pad;
static char g_last[512];
static char g_art_key[64];

void gw_log(const char *fmt, ...) { (void) fmt; }
static int gs_may_run(int i) { (void) i; return g_may_run; }
int gw_RB_Enabled(void) { return 0; }
static int gs_ui_port_read(int port, char *name, int cap, int *percent, int *stocks, int *cpu)   /* the fighter readbacks gd.player uses */
{
    (void) port; (void) cap; name[0] = 0; *percent = *stocks = *cpu = 0;
    return 0;
}
int gw_Netplay_Enabled(void) { return 0; }
int gw_Mods_Count(void) { return 0; }
int gw_Mods_MenuCount(int i) { (void) i; return 0; }
const char *gw_Mods_MenuField(int i, int k, const char *f) { (void) i; (void) k; (void) f; return ""; }
const char *gw_Mods_Id(int i) { (void) i; return ""; }
static int gs_pcall(int script, int nargs, int nres, const char *what)
{
    int old = gs.cur, rc = 0;
    (void) what;
    gs.cur = script;
    if (lua_pcall(gs.L, nargs, nres, 0) != LUA_OK) { lua_pop(gs.L, 1); rc = -1; }
    gs.cur = old;
    return rc;
}
static int gs_get_hook(int i, const char *name)
{
    if (!gs.s[i].used || gs.s[i].disabled) return 0;
    lua_getglobal(gs.L, name);
    if (lua_isfunction(gs.L, -1)) return 1;
    lua_pop(gs.L, 1);
    return 0;
}
static const GsScript *gs_cur_script(void) { return &gs.s[gs.cur]; }
static double gs_now_ms(void) { return g_now; }
static double gs_ui_clock;
static struct { int token; } gs_stage_models[4];
static int gs_stage_nmodels;
int gw_ScriptGame_ModelRefOwned(int a, int b, int c, int d) { (void) a; (void) b; (void) c; (void) d; return 0; }
int gw_Script_StageResourceOwner(void) { return 0; }
static int gs_sm_emit(int asset, const GsmParams *p, int clip, const char **why) { (void) asset; (void) p; (void) clip; (void) why; return 0; }
void gw_script_pad_state(int ch, unsigned *b, int *sx, int *sy, int *cx, int *cy, int *tl, int *tr) { (void) ch; *b = 0; *sx = *sy = *cx = *cy = *tl = *tr = 0; }
unsigned gw_script_pad_raw_buttons(int ch) { (void) ch; return g_pad; }
static float g_mx = -1000.0f, g_my = -1000.0f, g_mwheel; static int g_mbuttons;
void gw_Mouse_ScriptRead(float *x, float *y, int *buttons, float *wheel) { *x = g_mx; *y = g_my; *buttons = g_mbuttons; *wheel = g_mwheel; }
float gw_Console_ScriptWidth(void) { return 640.0f; }
int gw_Settings_Int(const char *k, int d) { (void) k; return d; }
static void gs_prof_setfuncs(lua_State *L, const luaL_Reg *funcs, const char *prefix) { (void) prefix; luaL_setfuncs(L, funcs, 0); }
static int gs_kit_record(int first, int added) { (void) first; return added; }
static double gs_kit_optnum(lua_State *L, int t, const char *k, double def)
{
    double v = def;
    if (!lua_istable(L, t)) return def;
    lua_getfield(L, t, k);
    if (lua_isnumber(L, -1)) v = lua_tonumber(L, -1);
    lua_pop(L, 1);
    return v;
}
int gw_Kit_Available(void) { return 1; }
const char *gw_Kit_Why(void) { return ""; }
const char *gw_Kit_RoleMissingPage(int role) { (void) role; return NULL; }
int gw_Kit_Role(const char *name) { return name != NULL && name[0] == 'a' && name[1] == '_' ? 3 : -1; }
float gw_Kit_TextWidth(int role, const char *s) { (void) role; return 6.0f * (float) strlen(s) + g_track * (float) strlen(s); }
int gw_Kit_DrawText(float x, float y, const char *s, int role, uint32_t rgba, int align, float max_w, float shear, float *out_w)
{ (void) x; (void) y; (void) role; (void) rgba; (void) align; (void) max_w; (void) shear; if (out_w) *out_w = 0.0f; g_quads += (int) strlen(s); return (int) strlen(s); }
int gw_Kit_DrawPoly4(const float x[4], const float y[4], uint32_t rgba) { (void) x; (void) y; (void) rgba; g_quads++; return 1; }
static int g_images;
int gw_Kit_DrawImage(int tex, float x, float y, float w, float h, uint32_t rgba, int flip, float shear) { (void) tex; (void) x; (void) y; (void) w; (void) h; (void) rgba; (void) flip; (void) shear; g_quads++; g_images++; return 1; }
void gw_Kit_SetTracking(float px) { g_track = px; }
static int g_hsd_frame; void gw_Kit_TexHsdFrame(int frame) { g_hsd_frame = frame; }
static int g_hsd_drops; void gw_Kit_TexDropHsd(void) { g_hsd_drops++; }
int gw_Kit_TexAddHsd(const char *key, int gx_fmt, const uint8_t *img, size_t img_size, int w, int h, int tlut_fmt, const uint8_t *tlut, int tlut_n)
{ (void) gx_fmt; (void) img; (void) img_size; (void) w; (void) h; (void) tlut_fmt; (void) tlut; (void) tlut_n; g_art_calls++; snprintf(g_art_key, sizeof g_art_key, "%s", key); return g_art_ret; }
int gw_Kit_QuadCount(void) { return g_quads; }
int gw_Kit_QuadRoom(void) { return 16000; }

#include "atlas_nucleus_stubs.h"
#include "gw_script_ui.inc"

static const char *lua(const char *code)
{
    static char buf[512];
    lua_State *L = gs.L;
    int top = lua_gettop(L);
    buf[0] = '\0';
    if (luaL_dostring(L, code) != LUA_OK) snprintf(buf, sizeof buf, "ERR %s", lua_tostring(L, -1));
    else if (lua_gettop(L) > top) snprintf(buf, sizeof buf, "%s", luaL_tolstring(L, -1, NULL));
    lua_settop(L, top);
    snprintf(g_last, sizeof g_last, "%s", buf);
    return buf;
}
static int t_lua(const char *code) { return strncmp(lua(code), "ERR", 3) != 0; }
static const char *t_lua_err(void) { return g_last; }

static unsigned bswap_u(unsigned v) { return (v >> 24) | ((v >> 8) & 0xFF00u) | ((v << 8) & 0xFF0000u) | (v << 24); }
/* the adapter reads these out-parameters from game memory with byte-swapped loads: read them back the same way */
static int poll(int h, int *k, int *a, int *b)
{
    int r = gw_Ui_SelPoll(h, k, a, b);
    if (r) { *k = (int) bswap_u((unsigned) *k); *a = (int) bswap_u((unsigned) *a); *b = (int) bswap_u((unsigned) *b); }
    return r;
}
static void reset_ui(void)
{
    int i;
    for (i = 0; i < GS_UI_SLOTS; i++) { memset(&gs_ui_slot[i], 0, sizeof gs_ui_slot[i]); gs_ui_slot[i].dialog_fn = -1; }
    memset(&gs_ui_stack, 0, sizeof gs_ui_stack);
    gs_ui_qn = 0; g_pad = 0; gs.cur = 0; gs.scene_kind = 7;
    gs_sel.h = -1; gs_sel.qn = 0;
    g_mx = g_my = -1000.0f; g_mbuttons = 0; g_mwheel = 0.0f; g_art_calls = 0; g_images = 0;
    for (i = 1; i < 8; i++) gs.s[i].used = 0;
}
static void fake_script(int i, const char *mod) { snprintf(gs.s[i].id, sizeof gs.s[i].id, "%s/main", mod); gs.s[i].used = 1; gs.s[i].disabled = 0; }

/* ---- the data door (gw_script_ui_data.inc) ---------------------------------------------------------------------------------------- */
_Static_assert(GS_SETEV_MOVE == 7 && GS_SETEV_PAGE == 8, "the data events keep their numbers: the game side writes them as literals");

static GsUiSlot *slot_of(int h) { return &gs_ui_slot[h & 0xFF]; }
static void draw_once(void) { gs_ui_draw(); }
static void reset_data(void)
{
    reset_ui();
    gs_set.h = -1; gs_set.qn = 0; gs_set.frozen = 0; gs_set.answer = 0; gs_set.pending = 0; gs_set.fid[0] = '\0'; gs_set.fidx = -1; gs_set.data = 0;
    memset(gs.key_now, 0, sizeof gs.key_now);
}
static unsigned bswap_p(unsigned v) { return (v >> 24) | ((v >> 8) & 0xFF00u) | ((v << 8) & 0xFF0000u) | (v << 24); }
/* the adapter reads the poll's out-parameters with byte-swapped loads: so does this */
static int spoll(int h, int *slot, int *arg)
{
    int t = gw_Ui_SetPoll(h, slot, arg);
    if (t) { *slot = (int) bswap_p((unsigned) *slot); *arg = (int) bswap_p((unsigned) *arg); }
    return t;
}
static void drain_set(int h) { int s, a; while (spoll(h, &s, &a)) {} }
static void settle(void) { gs_ui_tick(); }
static const char *focused_id(int h)
{
    int s = gw_Ui_SetFocused(h);
    return (s >= 0 && s < slot_of(h)->sc.n_items) ? slot_of(h)->sc.items[s].id : "";
}
static void rect_of_row(int h, int idx, float *cx, float *cy)
{
    int i;
    GsUiSlot *u = slot_of(h);
    *cx = *cy = -1000.0f;
    for (i = 0; i < u->hits.n; i++) if (u->hits.h[i].kind == AT_HIT_CELL && u->hits.h[i].b == idx) { *cx = u->hits.h[i].r.x + u->hits.h[i].r.w * 0.5f; *cy = u->hits.h[i].r.y + u->hits.h[i].r.h * 0.5f; }
}
static int open_data(void) { return gw_Ui_DataOpen("data.events", "SOLO", "", "EVENT MATCH", 1); }
/* n event rows from display index `first`: the first four cleared, then open, the last locked (events 7 and up are locked when n is 9 and first 0 and `locked_from` is 7) */
static void events_window(int h, int first, int n, int locked_from)
{
    int i;
    gw_Ui_SetRows(h, n);
    for (i = 0; i < n; i++) {
        int ev = first + i, flags = (ev % 3 == 0 ? 1 : 0) | (ev >= locked_from ? 2 : 0) | 4;
        gw_Ui_RowEvent(h, i, ev, flags, 3615u, "");
    }
}

static void door_opens_and_refuses(void)
{
    int h, s;
    reset_data();
    CHECK(gw_Ui_DataOpen("settings.tabs", "", "", "X", 0) == -1 && gw_Ui_DataOpen("data.", "", "", "X", 0) == -1 && gw_Ui_DataOpen(NULL, "", "", "X", 0) == -1);   /* only data.<name> */
    CHECK(gw_Ui_DataOpen("data.a-name-that-is-far-too-long-for-the-screen-record", "", "", "X", 0) == -1);
    h = open_data();
    CHECK(h >= 0 && gw_Ui_TopIsEngine("data.events") == 1 && gs_set.data == 1);
    CHECK(slot_of(h)->owner == GS_UI_ENGINE && slot_of(h)->native_set == 1 && slot_of(h)->sc.primary == AT_PRIMARY_LIST && slot_of(h)->sc.chapter == 1 && slot_of(h)->sc.n_parents == 1);
    CHECK(gw_Ui_SetCanOpen() == 0 && gw_Ui_SelCanOpen() == 0);                         /* one native screen at a time, the settings' rule */
    CHECK(gw_Ui_DataOpen("data.misc", "", "", "X", 0) == -1);
    CHECK(gw_Ui_SetOpen(0, 0, "", "", "SETTINGS") == -1);
    s = gw_Ui_SelOpen(0, 0, "VERSUS", "MELEE", "FIGHTERS"); CHECK(s == -1);
    gw_Ui_SetClose(h);
    CHECK(gs_ui_stack.n == 0 && gw_Ui_TopIsEngine("data.events") == 0 && gw_Ui_SetCanOpen() == 1);
    h = gw_Ui_DataOpen("data.misc", "MAIN MENU", "RECORDS", "MISC. RECORDS", 0); CHECK(h >= 0 && slot_of(h)->sc.n_parents == 2 && slot_of(h)->sc.chapter == 0); gw_Ui_SetClose(h);
    h = gw_Ui_DataOpen("data.x", "", "", "X", 99); CHECK(h >= 0 && slot_of(h)->sc.chapter == 5); gw_Ui_SetClose(h);                 /* a chapter is pulled into 0..5 */
    h = open_data(); gw_Ui_SceneExit(gs.scene_kind); CHECK(gs_ui_stack.n == 0); CHECK(gw_Ui_SetCanOpen() == 1);                     /* a scene that ends closes it */
    h = gw_Ui_SetOpen(0, 0, "MAIN MENU", "", "SETTINGS"); CHECK(h >= 0 && gs_set.data == 0); gw_Ui_SetClose(h);                      /* the settings door is not a data door */
}

static void rows_are_values_and_ids_are_absolute(void)
{
    int h, i;
    GsUiSlot *u;
    reset_data();
    h = open_data(); u = slot_of(h);
    events_window(h, 2, 9, 7);                                           /* display indices 2..10: 7 and up locked */
    CHECK(u->sc.n_items == 9 && strcmp(u->sc.items[0].id, "r2") == 0 && strcmp(u->sc.items[8].id, "r10") == 0);   /* the id is the ABSOLUTE row: a slide keeps it */
    CHECK(strcmp(u->sc.items[0].label, "EVENT 3") == 0 && u->sc.items[0].vkind == AT_VAL_TEXT && (u->sc.items[0].iflags & AT_ITEM_RO));
    CHECK(strcmp(u->sc.items[0].text, "--:-- --") == 0 && !(u->sc.items[0].flags & AT_CELL_SELECTED));   /* display 2: timed, never cleared */
    CHECK(strcmp(u->sc.items[1].text, "01:00 25") == 0 && (u->sc.items[1].flags & AT_CELL_SELECTED));    /* display 3 (3 % 3 == 0): cleared, the jade edge */
    CHECK((u->sc.items[5].iflags & AT_ITEM_DISABLED) && (u->sc.items[5].flags & AT_CELL_DISABLED) && u->sc.items[5].reason[0] != '\0');   /* display 7: locked */
    CHECK(!(u->sc.items[4].iflags & AT_ITEM_DISABLED));
    gw_Ui_DataSub(h, 0, "Fight a foe"); CHECK(strcmp(u->sc.items[0].sub, "Fight a foe") == 0 && strcmp(u->sc.items[0].label, "EVENT 3") == 0);
    gw_Ui_DataSub(h, 1, "Fight another"); CHECK(strstr(u->sc.items[1].sub, "Cleared") != NULL && strstr(u->sc.items[1].sub, "Fight another") != NULL);   /* the word, then the sub */
    gw_Ui_DataSub(h, 40, "past the rows"); gw_Ui_DataSub(h, -1, "x");     /* a slot past the rows is ignored */
    gw_Ui_DataRow(h, 40, 40, "x", "x", 0); gw_Ui_DataRow(h, -1, 0, "x", "x", 0);
    gw_Ui_DataRow(h, 0, 7, "Plain", "v", 0xF0);                          /* a flag outside the four is dropped */
    CHECK(strcmp(u->sc.items[0].id, "r7") == 0 && u->sc.items[0].tag[0] == '\0' && !(u->sc.items[0].iflags & AT_ITEM_DISABLED));
    gw_Ui_DataCounter(h, "3 / 51"); CHECK(strcmp(u->sc.counter, "3 / 51") == 0 && strcmp(u->view.counter, "3 / 51") == 0);
    g_quads = 0; draw_once(); CHECK(g_quads > 80 && g_quads < AT_SCREEN_QUAD_WARN);
    for (i = 0; i < u->hits.n; i++) CHECK(u->hits.h[i].kind != AT_HIT_TAB);   /* no tabs unless the adapter sets them */
    /* the window holds AT_DATA_ROWS rows and the record AT_MAX_ITEMS: a full window still draws */
    events_window(h, 0, AT_DATA_ROWS, 51); g_quads = 0; draw_once(); CHECK(g_quads > 80 && g_quads < AT_SCREEN_QUAD_WARN);
    CHECK(gw_Ui_DataWindow() == AT_DATA_ROWS && gw_Ui_DataFirst(51, 9, 9, 0) == 1 && gw_Ui_DataFirst(120, AT_DATA_ROWS, 119, 0) == 120 - AT_DATA_ROWS);
    gw_Ui_SetClose(h);
    /* a row of a closed or wrong handle goes nowhere */
    gw_Ui_DataRow(h, 0, 0, "x", "x", 0); gw_Ui_DataCounter(h, "x"); gw_Ui_RowMisc(h, 0, 0, 1u, ""); gw_Ui_RowEvent(h, 0, 0, 0, 0u, "");
}

static void the_game_owns_the_cursor(void)
{
    int h, s, a;
    reset_data();
    h = open_data(); events_window(h, 0, 9, 7); gw_Ui_SetFocus(h, "r0"); settle();
    gw_Ui_SetIntent(h, AT_EV_MOVE, AT_DIR_DOWN);
    CHECK(spoll(h, &s, &a) == GS_SETEV_MOVE && a == AT_DIR_DOWN);                  /* the move is an event ... */
    CHECK(strcmp(focused_id(h), "r0") == 0);                                       /* ... and the host did not apply it: it would wrap inside the window */
    gw_Ui_SetIntent(h, AT_EV_MOVE, AT_DIR_UP); CHECK(spoll(h, &s, &a) == GS_SETEV_MOVE && a == AT_DIR_UP && strcmp(focused_id(h), "r0") == 0);
    gw_Ui_SetIntent(h, AT_EV_MOVE, AT_DIR_RIGHT); CHECK(spoll(h, &s, &a) == GS_SETEV_MOVE && a == AT_DIR_RIGHT);       /* left and right too (the Sound Test uses them) */
    gw_Ui_SetIntent(h, AT_EV_MOVE, AT_DIR_LEFT); CHECK(spoll(h, &s, &a) == GS_SETEV_MOVE && a == AT_DIR_LEFT);
    gw_Ui_SetIntent(h, AT_EV_PAGE, 1); CHECK(spoll(h, &s, &a) == GS_SETEV_PAGE && a == 1);                              /* R: a page, not a tab */
    gw_Ui_SetIntent(h, AT_EV_PAGE, -1); CHECK(spoll(h, &s, &a) == GS_SETEV_PAGE && a == -1);
    gw_Ui_SetIntent(h, AT_EV_ALT, 'Y'); CHECK(spoll(h, &s, &a) == GS_SETEV_ALT && a == 'Y');
    gw_Ui_SetIntent(h, AT_EV_BACK, 0); CHECK(spoll(h, &s, &a) == GS_SETEV_BACK && s == -1);
    gw_Ui_SetIntent(h, AT_EV_ACCEPT, 0); CHECK(spoll(h, &s, &a) == GS_SETEV_ACCEPT && s == 0);                          /* A on the focused row: its window index */
    /* the adapter moves its cursor and tells the host by id: the focus lands, the window may have slid */
    gw_Ui_SetFocus(h, "r4"); CHECK(strcmp(focused_id(h), "r4") == 0);
    events_window(h, 1, 9, 7); gw_Ui_SetFocus(h, "r4");                            /* the window slid by one: r4 is now slot 3 */
    CHECK(gw_Ui_SetFocused(h) == 3);
    gw_Ui_SetIntent(h, AT_EV_ACCEPT, 0); CHECK(spoll(h, &s, &a) == GS_SETEV_ACCEPT && s == 3);
    /* a click on a locked row does NOTHING, by pad and by mouse */
    gw_Ui_SetFocus(h, "r7"); events_window(h, 1, 9, 7); gw_Ui_SetFocus(h, "r7");
    gw_Ui_SetIntent(h, AT_EV_ACCEPT, 0); CHECK(spoll(h, &s, &a) == 0);                                                   /* A on a locked row */
    gw_Ui_SetIntent(h, AT_EV_ALT, 'X'); CHECK(spoll(h, &s, &a) == GS_SETEV_ALT);                                         /* other keys are not blocked */
    gw_Ui_SetClose(h);
}

static void keys_and_mouse_on_a_data_list(void)
{
    int h, s, a;
    float x, y;
    reset_data();
    h = open_data(); events_window(h, 0, 9, 7); gw_Ui_SetFocus(h, "r0"); gw_Ui_SetKeys(h, "A:Start,B:Back"); settle(); draw_once();
    gs.key_now[VK_DOWN] = 1; gs_ui_tick(); gs.key_now[VK_DOWN] = 0; gs_ui_tick();
    CHECK(spoll(h, &s, &a) == GS_SETEV_MOVE && a == AT_DIR_DOWN && s == -1 && strcmp(focused_id(h), "r0") == 0);   /* keys send slot -1 (they wrap) */
    gs.key_now[VK_TAB] = 1; gs_ui_tick(); gs.key_now[VK_TAB] = 0; gs_ui_tick(); CHECK(spoll(h, &s, &a) == GS_SETEV_PAGE && a == 1);
    gs.key_now[VK_RETURN] = 1; gs_ui_tick(); gs.key_now[VK_RETURN] = 0; gs_ui_tick(); CHECK(spoll(h, &s, &a) == GS_SETEV_ACCEPT && s == 0);
    gs.key_now[VK_ESCAPE] = 1; gs_ui_tick(); gs.key_now[VK_ESCAPE] = 0; gs_ui_tick(); CHECK(spoll(h, &s, &a) == GS_SETEV_BACK);
    drain_set(h);
    /* the mouse: a hover moves the HOST focus (the adapter then adds the window start), a click is an accept, a click on a locked row is nothing */
    draw_once(); rect_of_row(h, 2, &x, &y); CHECK(x > 0.0f);
    g_mx = x; g_my = y; gs_ui_tick(); g_mx = x + 1.0f; gs_ui_tick();
    CHECK(spoll(h, &s, &a) == GS_SETEV_FOCUS && s == 2 && strcmp(focused_id(h), "r2") == 0);
    g_mbuttons = 1; gs_ui_tick(); g_mbuttons = 0; gs_ui_tick();
    CHECK(spoll(h, &s, &a) == GS_SETEV_ACCEPT && s == 2);
    drain_set(h);
    rect_of_row(h, 7, &x, &y); CHECK(x > 0.0f);                                    /* display 7 is locked */
    g_mx = x; g_my = y; gs_ui_tick(); g_mx = x + 1.0f; gs_ui_tick(); drain_set(h);
    g_mbuttons = 1; gs_ui_tick(); g_mbuttons = 0; gs_ui_tick();
    CHECK(spoll(h, &s, &a) == 0);                                                  /* a click on a disabled item does nothing */
    g_mbuttons = 2; gs_ui_tick(); g_mbuttons = 0; gs_ui_tick(); CHECK(spoll(h, &s, &a) == GS_SETEV_BACK);   /* a right click is back */
    g_mwheel = 1.0f; gs_ui_tick(); g_mwheel = 0.0f; gs_ui_tick(); CHECK(spoll(h, &s, &a) == GS_SETEV_MOVE && a == AT_DIR_UP && s == 1);       /* the wheel is the adapter's move, not the host's; slot 1 says it was the wheel */
    g_mwheel = -1.0f; gs_ui_tick(); g_mwheel = 0.0f; gs_ui_tick(); CHECK(spoll(h, &s, &a) == GS_SETEV_MOVE && a == AT_DIR_DOWN && s == 1);
    g_mx = g_my = -1000.0f; gs_ui_tick();
    gw_Ui_SetClose(h);
}

static void the_ring_is_clean_at_every_edge(void)
{
    int h, s, a;
    reset_data();
    h = open_data(); events_window(h, 0, 9, 7); settle();
    gw_Ui_SetIntent(h, AT_EV_BACK, 0); gw_Ui_SetIntent(h, AT_EV_MOVE, AT_DIR_DOWN); CHECK(gs_set.qn == 2);
    gw_Ui_SetClose(h);                                                             /* end */
    CHECK(gs_set.qn == 0);
    h = open_data(); CHECK(gs_set.qn == 0 && spoll(h, &s, &a) == 0);               /* begin: nothing of the last screen */
    events_window(h, 0, 9, 7); gw_Ui_SetFocus(h, "r0");
    settle(); gw_Ui_SetIntent(h, AT_EV_BACK, 0); gw_Ui_SetIntent(h, AT_EV_ACCEPT, 0); CHECK(gs_set.qn == 2);
    gw_Ui_Freeze(h, 1); CHECK(gs_set.qn == 0 && spoll(h, &s, &a) == 0);            /* freeze */
    gw_Ui_SetIntent(h, AT_EV_MOVE, AT_DIR_DOWN); CHECK(gs_set.qn == 0);            /* a frozen screen takes no event */
    gw_Ui_Freeze(h, 0);
    /* a press of the frame the screen opened is not its own */
    gw_Ui_SetClose(h);
    h = open_data(); gw_Ui_SetIntent(h, AT_EV_ACCEPT, 0); CHECK(spoll(h, &s, &a) == 0);
    gw_Ui_SetClose(h);
    /* events left in the ring of a screen that a scene exit closed are gone with the next open (the host does not reset the ring at exit; open does) */
    h = open_data(); settle(); gw_Ui_SetIntent(h, AT_EV_BACK, 0); gw_Ui_SceneExit(gs.scene_kind);
    h = open_data(); CHECK(gs_set.qn == 0);
    gw_Ui_SetClose(h);
}

static void poll_and_decoder_write_big_endian(void)
{
    int h;
    unsigned char raw[8];
    /* the poll: the game reads its locals with byte-swapped loads, so the raw bytes must be big-endian (a native store of 4 arrived as 0x04000000) */
    reset_data();
    h = open_data(); events_window(h, 0, 9, 7); settle();
    gw_Ui_SetIntent(h, AT_EV_MOVE, AT_DIR_DOWN);
    memset(raw, 0xEE, sizeof raw);
    CHECK(gw_Ui_SetPoll(h, (int *) raw, (int *) (raw + 4)) == GS_SETEV_MOVE);
    CHECK(raw[0] == 0xFF && raw[1] == 0xFF && raw[2] == 0xFF && raw[3] == 0xFF);            /* the slot, -1 */
    CHECK(raw[4] == 0 && raw[5] == 0 && raw[6] == 0 && raw[7] == AT_DIR_DOWN);              /* the direction, most significant byte first */
    settle(); gw_Ui_SetIntent(h, AT_EV_PAGE, -1);
    CHECK(gw_Ui_SetPoll(h, (int *) raw, (int *) (raw + 4)) == GS_SETEV_PAGE);
    CHECK(raw[4] == 0xFF && raw[5] == 0xFF && raw[6] == 0xFF && raw[7] == 0xFF);            /* -1 is 0xFFFFFFFF either way round */
    gw_Ui_SetClose(h);
    /* the text decoder's five counts: the game reads them with byte-swapped loads too */
    { static unsigned char G[0x240] = { 0x20, 0x41, 0x20, 0x62, 0x20, 0x37, 0x20, 0x00 }, S[0x240] = { 0x82, 0x60, 0x82, 0x82, 0x82, 0x56, 0x81, 0x40 };   /* invented, the rest zero */
      const unsigned char sis[] = { 12, 255, 0, 0, 0x20, 0x41, 0x2F, 0xFF, 0x20, 0x62, 0 };
      unsigned char st[20]; char out[16];
      memset(st, 0xEE, sizeof st);
      gw_Ui_SisTables(G, S);   /* the host decodes with 0x120 pairs: the invented tables are padded */
      CHECK(gw_Ui_SisDecode(sis, (int) sizeof sis, out, sizeof out, (int *) st) == 3);
      CHECK(strcmp(out, "A?b") == 0);
      CHECK(st[0] == 0 && st[1] == 0 && st[2] == 0 && st[3] == 3);                         /* chars 3 */
      CHECK(st[4] == 0 && st[7] == 1);                                                      /* unknown 1 */
      CHECK(st[11] == 1);                                                                   /* controls 1 */
      CHECK(st[15] == 0 && st[19] == 0);                                                    /* jumps 0, bad 0 */
      CHECK(gw_Ui_SisDecode(sis, (int) sizeof sis, out, sizeof out, NULL) == 3);
      CHECK(gw_Ui_SisDecode(sis, 6, out, sizeof out, (int *) st) == 1 && st[19] == 1);                  /* a length that cuts the stream: bad (the last count, big-endian) */                     /* no stats wanted */ }
}

static void rows_through_the_models(void)
{
    int h, i;
    GsUiSlot *u;
    static const unsigned char tag_ok[8] = { 0x82, 0x60, 0x82, 0x81, 0, 0, 0, 0 }, tag_kana[8] = { 0x83, 0x41, 0, 0, 0, 0, 0, 0 };
    reset_data();
    h = gw_Ui_DataOpen("data.misc", "MAIN MENU", "RECORDS", "MISC. RECORDS", 0); u = slot_of(h);
    gw_Ui_SetRows(h, 3);
    gw_Ui_RowMisc(h, 0, 14, 1234567u, ""); gw_Ui_RowMisc(h, 1, 2, 3900u, ""); gw_Ui_RowMisc(h, 2, 20, 0u, "Fox");
    CHECK(strcmp(u->sc.items[0].text, "1,234,567") == 0 && strcmp(u->sc.items[1].text, "1:05") == 0 && strcmp(u->sc.items[2].text, "Fox") == 0);
    CHECK(strcmp(u->sc.items[0].id, "r14") == 0 && strcmp(gw_Ui_MiscLabel(14), u->sc.items[0].label) == 0 && gw_Ui_MiscLabel(77)[0] == 0);
    gw_Ui_SetRows(h, 4);
    gw_Ui_RowTag(h, 0, -1, NULL, 0); gw_Ui_RowTag(h, 1, 4, tag_ok, 1); gw_Ui_RowTag(h, 2, 5, tag_kana, 2); gw_Ui_RowTag(h, 3, 6, NULL, 3);
    CHECK(strcmp(u->sc.items[0].label, "NEW TAG") == 0 && strcmp(u->sc.items[0].tag, "NEW") == 0);
    CHECK(strcmp(u->sc.items[1].label, "Aa") == 0 && strcmp(u->sc.items[2].label, "TAG 6") == 0 && strcmp(u->sc.items[3].label, "TAG 7") == 0);   /* kana and none: TAG n */
    CHECK(strcmp(gw_Ui_TagText(tag_ok), "Aa") == 0 && gw_Ui_TagText(tag_kana)[0] == 0);
    gw_Ui_SetRows(h, 2);
    gw_Ui_RowStat(h, 0, 0, "Fox", 11, 3615u); gw_Ui_RowStat(h, 1, 1, "", 16 | ((AT_SF_US | AT_SF_OVER) << 8), 12u);
    CHECK(strcmp(u->sc.items[0].text, "60:15") == 0 && strcmp(u->sc.items[0].label, "1. Fox") == 0);
    CHECK(strcmp(u->sc.items[1].text, "12 mi") == 0 && strcmp(u->sc.items[1].label, "2.") == 0);
    CHECK(gw_Ui_StatKind(11) == AT_STAT_TIME && gw_Ui_StatKind(99) == AT_STAT_COUNT && strcmp(gw_Ui_StatText(3, 8743u), "87.43%") == 0);
    CHECK(gw_Ui_StatLabel(0)[0] != 0 && gw_Ui_StatLabel(40)[0] == 0);
    gw_Ui_SetRows(h, 2); gw_Ui_RowSound(h, 0, 79, "", 0); gw_Ui_RowSound(h, 1, 3, "Invented Song", 1);
    CHECK(strcmp(u->sc.items[0].label, "TRACK 80") == 0 && strcmp(u->sc.items[1].text, "PLAYING") == 0);
    gw_Ui_RowSfx(h, 0, 2, 3, 5);
    CHECK(strcmp(u->sc.items[0].id, "r2") == 0 && strcmp(u->sc.items[0].text, "4 / 5") == 0);
    CHECK(gw_Ui_SoundToggle(-1, 4) == AT_SND_PLAY && gw_Ui_SoundToggle(4, 4) == AT_SND_STOP && gw_Ui_SoundToggle(2, 4) == AT_SND_SWITCH);
    gw_Ui_SetRows(h, 2); gw_Ui_RowMsg(h, 0, 0, 20261006); gw_Ui_RowMsg(h, 1, 1, 0);
    CHECK(strcmp(u->sc.items[0].text, "2026-10-06") == 0 && strcmp(u->sc.items[1].label, "MESSAGE 2") == 0 && u->sc.items[1].text[0] == 0);
    gw_Ui_RowBonus(h, 0, 4);
    CHECK(strcmp(u->sc.items[0].label, "BONUS 5") == 0);
    /* the message order and the ranking are computed on the host from numbers the game writes one by one */
    for (i = 0; i < 6; i++) gw_Ui_MsgSet(i, i % 2, (unsigned) (60 - i * 10));       /* 1, 3, 5 unlocked: dates 50, 30, 10 */
    CHECK(gw_Ui_MsgOrder(6) == 3 && gw_Ui_MsgAt(0) == 5 && gw_Ui_MsgAt(1) == 3 && gw_Ui_MsgAt(2) == 1 && gw_Ui_MsgAt(3) == -1 && gw_Ui_MsgAt(-1) == -1);
    gw_Ui_MsgSet(99, 1, 1u); gw_Ui_MsgSet(-1, 1, 1u);                                 /* out of range: ignored */
    for (i = 0; i < 5; i++) gw_Ui_RankSet(i, (unsigned) (i == 2 ? 0 : 10 - i));
    CHECK(gw_Ui_RankBuild(5) == 4 && gw_Ui_RankAt(0) == 0 && gw_Ui_RankAt(1) == 1 && gw_Ui_RankAt(2) == 3 && gw_Ui_RankAt(3) == 4 && gw_Ui_RankAt(4) == -1);
    CHECK(gw_Ui_EventLastOpen(1, 0) == 9 && gw_Ui_EventLastOpen(1, 1) == 50);
    gw_Ui_SetClose(h);
}

static void results_card_through_the_door(void)
{
    int h, n, s, a;
    GsUiSlot *u;
    reset_data();
    gw_Ui_ResBegin(2, 0, 0);
    gw_Ui_ResPlayer(0, AT_PK_HUMAN, 1, 0, 1, 4); gw_Ui_ResPlayer2(0, 130, 0, 0, "Fox");
    gw_Ui_ResPlayer(1, AT_PK_CPU, 2, 2, 4, 1);   gw_Ui_ResPlayer2(1, 87, 0, 1, "Marth");
    gw_Ui_ResPlayer(9, AT_PK_HUMAN, 0, 0, 0, 0); gw_Ui_ResPlayer2(-1, 0, 0, 0, "x");     /* a bad port is ignored */
    CHECK(gw_Ui_ResDone() == -1);                                                          /* not built yet */
    gw_Ui_ResCommit();
    CHECK(gw_Ui_ResDone() == -1);                                                          /* a human has not confirmed */
    h = gw_Ui_DataOpen("data.results", "", "", "RESULTS", 0); u = slot_of(h);
    n = gw_Ui_ResFill(h);
    CHECK(n == 2 && u->sc.n_items == 2 && strcmp(u->sc.items[0].label, "P2 CPU  Marth") == 0 && strcmp(u->sc.items[0].text, "WINNER") == 0 && strcmp(u->sc.items[1].text, "2nd") == 0);
    gw_Ui_ResConfirm(1, AT_RC_START); CHECK(gw_Ui_ResDone() == -1);                         /* a CPU cannot confirm */
    gw_Ui_ResConfirm(0, AT_RC_START); CHECK(gw_Ui_ResDone() == 10);
    settle(); gw_Ui_SetIntent(h, AT_EV_ACCEPT, 0); CHECK(spoll(h, &s, &a) == GS_SETEV_ACCEPT);
    gw_Ui_SetClose(h);
    CHECK(gw_Ui_ResFill(h) == 0);                                                          /* a closed handle fills nothing */
    gw_Ui_ResBegin(2, 0, 0); gw_Ui_ResConfirm(0, AT_RC_START); CHECK(gw_Ui_ResDone() == -1);   /* a confirm before the card exists does nothing */
}

int main(void)
{
    lua_State *L = luaL_newstate();
    gs.L = L; gs.cur = 0; gs.console = 0;
    snprintf(gs.s[0].id, sizeof gs.s[0].id, "console");
    snprintf(gs.s[1].id, sizeof gs.s[1].id, "envoy/main");
    gs.n = 8; gs.s[0].used = gs.s[1].used = gs.s[2].used = 1;
    luaL_openlibs(L);
    lua_newtable(L); gs_push_ui(L); lua_setfield(L, -2, "ui"); lua_setglobal(L, "gd");
    door_opens_and_refuses(); rows_are_values_and_ids_are_absolute(); the_game_owns_the_cursor(); keys_and_mouse_on_a_data_list(); the_ring_is_clean_at_every_edge();
    poll_and_decoder_write_big_endian(); rows_through_the_models(); results_card_through_the_door();
    lua_close(L);
    ATLAS_DONE("atlas data host");
}
