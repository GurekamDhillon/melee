/* Host-side test of the native settings screen (gw_script_ui_set.inc): the Ui_Set* shims the game-side adapter calls, driven the way the adapter drives them, against
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

#include "../../src/melee/gm/gmfrontend_items.h"
#include "../../src/melee/gm/gmfrontend_atlas_table.h"

/* the game side (gmfrontend_atlas_table.h) and the host (gw_script_ui_set.inc) write these numbers independently: they must agree */
_Static_assert(FSS_EV_CHANGE == GS_SETEV_CHANGE && FSS_EV_ACCEPT == GS_SETEV_ACCEPT && FSS_EV_BACK == GS_SETEV_BACK && FSS_EV_TAB == GS_SETEV_TAB &&
               FSS_EV_FOCUS == GS_SETEV_FOCUS, "the walker's event numbers are the host's");
_Static_assert(FSS_VK_NONE == AT_VAL_NONE && FSS_VK_TOGGLE == AT_VAL_TOGGLE && FSS_VK_CHOICE == AT_VAL_CHOICE && FSS_VK_SLIDER == AT_VAL_SLIDER &&
               FSS_VK_TEXT == AT_VAL_TEXT, "the walker's value kinds are the host's");
_Static_assert(FSS_K_TABS == GS_SET_TABS && FSS_K_REMAP == GS_SET_REMAP && FSS_K_HOWTO == GS_SET_HOWTO && FSS_K_ERASE == GS_SET_ERASE, "the adapter's screen kinds are the host's");
_Static_assert(FSS_IF_A_STEPS == AT_ITEM_A_STEPS && FSS_IF_RO == AT_ITEM_RO && FSS_IF_DISABLED == AT_ITEM_DISABLED, "the walker's row flags are the host's");
_Static_assert(FSS_IN_MOVE == AT_EV_MOVE && FSS_IN_ACCEPT == AT_EV_ACCEPT && FSS_IN_BACK == AT_EV_BACK && FSS_IN_ALT == AT_EV_ALT && FSS_IN_PAGE == AT_EV_PAGE &&
               FSS_DIR_LEFT == AT_DIR_LEFT && FSS_DIR_RIGHT == AT_DIR_RIGHT && FSS_DIR_UP == AT_DIR_UP && FSS_DIR_DOWN == AT_DIR_DOWN, "the adapter's intents are the host's");

static GsUiSlot *slot_of(int h) { return &gs_ui_slot[h & 0xFF]; }
static void draw_once(void) { gs_ui_draw(); }
static void reset_set(void)
{
    reset_ui();
    gs_set.h = -1; gs_set.qn = 0; gs_set.frozen = 0; gs_set.answer = 0; gs_set.pending = 0; gs_set.fid[0] = '\0'; gs_set.fidx = -1;
    memset(gs.key_now, 0, sizeof gs.key_now);
}
static int open_set(void) { return gw_Ui_SetOpen(0, 0, "MAIN MENU", "", "SETTINGS"); }
static void tabs6(int h, int active) { gw_Ui_SetTabs(h, 6, "VIDEO,AUDIO,CONTROLS,ONLINE,GAME,MODS", active); }

/* one page of n rows the way the walker submits them: kinds cycle toggle, slider (A steps), action, toggle, readout, action */
static void row(int h, int slot, int i, int kind, unsigned flags, const char *text, const char *group)
{
    char id[12], label[24], help[40];
    snprintf(id, sizeof id, "i%d", i); snprintf(label, sizeof label, "Row %d", i); snprintf(help, sizeof help, "Help for row %d.", i);
    gw_Ui_SetRow(h, slot, id, label, help, kind);
    gw_Ui_SetRowVal(h, slot, 0, kind == AT_VAL_SLIDER ? 100 : 1, 5, kind == AT_VAL_SLIDER ? 40 : 1, flags);
    gw_Ui_SetRowText(h, slot, text, "", group);
}
static void build(int h, int n)
{
    static const int kinds[6] = { AT_VAL_TOGGLE, AT_VAL_SLIDER, AT_VAL_NONE, AT_VAL_TOGGLE, AT_VAL_TEXT, AT_VAL_NONE };
    int i;
    gw_Ui_SetRows(h, n);
    for (i = 0; i < n; i++) row(h, i, i, kinds[i % 6], kinds[i % 6] == AT_VAL_SLIDER ? AT_ITEM_A_STEPS : kinds[i % 6] == AT_VAL_TEXT ? AT_ITEM_RO : 0u, kinds[i % 6] == AT_VAL_SLIDER ? "40%" : kinds[i % 6] == AT_VAL_TEXT ? "live" : "", i < 3 ? "DISPLAY" : "INPUT");
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
static void settle(void) { gs_ui_tick(); }                                      /* a tick: the screen is no longer 'just opened' */
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
static void rect_of_hit(int h, int kind, int a, float *cx, float *cy)
{
    int i;
    GsUiSlot *u = slot_of(h);
    *cx = *cy = -1000.0f;
    for (i = 0; i < u->hits.n; i++) if (u->hits.h[i].kind == kind && u->hits.h[i].a == a) { *cx = u->hits.h[i].r.x + u->hits.h[i].r.w * 0.5f; *cy = u->hits.h[i].r.y + u->hits.h[i].r.h * 0.5f; }
}

static void filled_and_drawn(void)
{
    int h, i, rows = 0, tabs = 0, keys = 0;
    GsUiSlot *u;
    reset_set();
    h = open_set();
    CHECK(h >= 0 && gw_Ui_TopIsEngine("settings.tabs") == 1);
    tabs6(h, 0); build(h, 6); gw_Ui_SetKeys(h, "A:Change,L:Page,B:Back"); gw_Ui_SetFocus(h, "i0");
    g_quads = 0; draw_once();
    CHECK(g_quads > 80 && g_quads < AT_SCREEN_QUAD_WARN);
    u = slot_of(h);
    for (i = 0; i < u->hits.n; i++) { rows += u->hits.h[i].kind == AT_HIT_CELL; tabs += u->hits.h[i].kind == AT_HIT_TAB; keys += u->hits.h[i].kind == AT_HIT_KEY; }
    CHECK(rows == 6 && tabs == 6 && keys == 3);                                                        /* the rows, the strip, three clickable hints */
    CHECK(u->owner == GS_UI_ENGINE && u->native_set == 1 && u->native_sel == 0 && u->sc.primary == AT_PRIMARY_LIST && u->scene == 7);
    CHECK(u->sc.n_tabs == 6 && strcmp(u->sc.tabs[2].name, "CONTROLS") == 0 && u->sc.tabs[2].count < 0 && u->view.tab == 0);   /* a count below zero: no number on the tab */
    CHECK(gs_ui_stack.n == 1);
    gw_Ui_SetClose(h);
    CHECK(gs_ui_stack.n == 0 && gw_Ui_TopIsEngine("settings.tabs") == 0 && gs_set.h == -1);
    h = gw_Ui_SetOpen(1, 0, "SETTINGS", "CONTROLS", "REMAP"); CHECK(h >= 0 && gw_Ui_TopIsEngine("settings.remap") == 1 && slot_of(h)->sc.n_parents == 2); gw_Ui_SetClose(h);
    h = gw_Ui_SetOpen(2, 0, "SETTINGS", "", "HOW TO PLAY ONLINE"); CHECK(h >= 0 && gw_Ui_TopIsEngine("settings.howto") == 1); gw_Ui_SetClose(h);
    h = gw_Ui_SetOpen(3, 0, "SETTINGS", "GAME", "ERASE DATA"); CHECK(h >= 0 && gw_Ui_TopIsEngine("settings.erase") == 1); gw_Ui_SetClose(h);
    CHECK(gw_Ui_SetOpen(4, 0, "", "", "") == -1 && gw_Ui_SetOpen(-1, 0, "", "", "") == -1);
    h = gw_Ui_SetOpen(0, 4, "MAIN MENU", "", "SETTINGS"); CHECK(slot_of(h)->view.tab == 4); gw_Ui_SetClose(h);   /* opens on a given tab */
}

static void one_native_screen_at_a_time(void)
{
    int h, s;
    reset_set();
    h = open_set();
    CHECK(gw_Ui_SelOpen(0, 0, "VERSUS", "MELEE", "FIGHTERS") == -1 && gw_Ui_SelCanOpen() == 0);       /* the stack has one top */
    CHECK(open_set() == -1 && gw_Ui_SetCanOpen() == 0);
    gw_Ui_SetClose(h);
    CHECK(gw_Ui_SetCanOpen() == 1);
    s = gw_Ui_SelOpen(0, 0, "VERSUS", "MELEE", "FIGHTERS"); CHECK(s >= 0);
    CHECK(open_set() == -1 && gw_Ui_SetCanOpen() == 0);                                               /* and the other way round */
    gw_Ui_SelClose(s);
    CHECK(open_set() >= 0);
    gw_Ui_SceneExit(gs.scene_kind);
}

static void rows_land_where_the_render_reads_them(void)
{
    int h;
    GsUiSlot *u;
    AtItem *it;
    reset_set();
    h = open_set(); u = slot_of(h);
    gw_Ui_SetRows(h, 5);
    gw_Ui_SetRow(h, 0, "i0", "VSync", "Wait for the display.", AT_VAL_TOGGLE); gw_Ui_SetRowVal(h, 0, 0, 1, 1, 7, 0u); gw_Ui_SetRowText(h, 0, "", "", "DISPLAY");
    it = &u->sc.items[0];
    CHECK(strcmp(it->id, "i0") == 0 && strcmp(it->label, "VSync") == 0 && it->vkind == AT_VAL_TOGGLE && it->on == 1 && strcmp(it->group, "DISPLAY") == 0);
    gw_Ui_SetRow(h, 1, "i1", "Master Volume", "x", AT_VAL_SLIDER); gw_Ui_SetRowVal(h, 1, 0, 100, 5, 250, AT_ITEM_A_STEPS); gw_Ui_SetRowText(h, 1, "100%", "", "");
    it = &u->sc.items[1];
    CHECK(it->vmin == 0 && it->vmax == 100 && it->vstep == 5 && it->vval == 100 && strcmp(it->text, "100%") == 0 && (it->iflags & AT_ITEM_A_STEPS));   /* a value past the range is pulled in */
    gw_Ui_SetRow(h, 2, "i2", "Show FPS", "x", AT_VAL_CHOICE); gw_Ui_SetRowVal(h, 2, 0, 2, 1, 1, 0u); gw_Ui_SetRowText(h, 2, "", "", "");
    gw_Ui_SetRowOpt(h, 2, 0, "Off"); gw_Ui_SetRowOpt(h, 2, 1, "FPS"); gw_Ui_SetRowOpt(h, 2, 2, "Performance");
    it = &u->sc.items[2];
    CHECK(it->n_opts == 3 && strcmp(it->opt[2], "Performance") == 0 && strcmp(it->text, "FPS") == 0);                         /* the text a choice shows is its option */
    gw_Ui_SetRowOpt(h, 2, 8, "no"); gw_Ui_SetRowOpt(h, 2, -1, "no"); CHECK(it->n_opts == 3);                                  /* past the record: ignored */
    gw_Ui_SetRow(h, 3, "i3", "Bind Input", "x", AT_VAL_NONE); gw_Ui_SetRowVal(h, 3, 0, 0, 0, 0, AT_ITEM_DISABLED); gw_Ui_SetRowText(h, 3, "", "Connect a controller first.", "");
    it = &u->sc.items[3];
    CHECK((it->flags & AT_CELL_DISABLED) && (it->iflags & AT_ITEM_DISABLED) && strcmp(it->reason, "Connect a controller first.") == 0);
    gw_Ui_SetRow(h, 4, "i4", "A very long label that the field will cut somewhere around the sixty-third char", "x", 99);
    CHECK(strlen(u->sc.items[4].label) == AT_STR - 1 && u->sc.items[4].vkind == AT_VAL_NONE);                                 /* a bad kind is none, a long label is cut */
    gw_Ui_SetRow(h, 5, "i5", "past the count", "x", AT_VAL_NONE); CHECK(u->sc.items[5].id[0] == '\0');                         /* a slot past SetRows' count is ignored */
    gw_Ui_SetRow(h, -1, "x", "x", "x", 0); gw_Ui_SetRowVal(h, 70, 0, 0, 0, 0, 0u); gw_Ui_SetRowText(h, 70, "x", "x", "x");
    gw_Ui_SetRows(h, 100); CHECK(u->sc.n_items == AT_MAX_ITEMS);                                                                  /* 64 rows */
    gw_Ui_SetRows(h, -5); CHECK(u->sc.n_items == 0);
    /* a row that is written again starts clean: nothing of the last page's row stays */
    gw_Ui_SetRows(h, 1); gw_Ui_SetRow(h, 0, "i9", "Plain", "", AT_VAL_NONE);
    CHECK(u->sc.items[0].group[0] == '\0' && u->sc.items[0].reason[0] == '\0' && u->sc.items[0].n_opts == 0 && u->sc.items[0].iflags == 0u && !(u->sc.items[0].flags & AT_CELL_DISABLED));
    gw_Ui_SetClose(h);
    CHECK(gw_Ui_ItemStep(AT_VAL_SLIDER, 0, 100, 5, 95, 1) == 100 && gw_Ui_ItemStep(AT_VAL_SLIDER, 0, 100, 5, 100, 1) == 100);        /* the new value, or cur when nothing changed */
    CHECK(gw_Ui_ItemStep(AT_VAL_CHOICE, 0, 5, 1, 5, 1) == 0 && gw_Ui_ItemStep(AT_VAL_TOGGLE, 0, 1, 1, 0, 1) == 1 && gw_Ui_ItemStep(AT_VAL_TEXT, 0, 0, 0, 3, 1) == 3);
}

static void tabs_and_focus(void)                                       /* Review Focus 5 */
{
    int h, s, a;
    reset_set();
    h = open_set(); tabs6(h, 0); build(h, 6); settle();
    gw_Ui_SetFocus(h, "i3");
    CHECK(strcmp(focused_id(h), "i3") == 0 && gw_Ui_SetFocused(h) == 3);
    gw_Ui_SetIntent(h, AT_EV_PAGE, 1);                                  /* R */
    CHECK(spoll(h, &s, &a) == GS_SETEV_TAB && a == 1);                  /* the screen asks for tab 1 ... */
    CHECK(slot_of(h)->view.tab == 0);                                   /* ... and does not switch itself: the adapter submits the rows first */
    tabs6(h, 1); build(h, 3); gw_Ui_SetFocus(h, "");
    CHECK(slot_of(h)->view.tab == 1 && strcmp(focused_id(h), "i0") == 0);                 /* a fresh tab starts on its first row */
    tabs6(h, 0); build(h, 6); gw_Ui_SetFocus(h, "i3");
    CHECK(strcmp(focused_id(h), "i3") == 0);                           /* the adapter refocuses by id: where the page was left */
    drain_set(h);
    gw_Ui_SetIntent(h, AT_EV_PAGE, -1); CHECK(spoll(h, &s, &a) == GS_SETEV_TAB && a == 5);   /* L from the first tab wraps */
    tabs6(h, 5); gw_Ui_SetIntent(h, AT_EV_PAGE, 1); CHECK(spoll(h, &s, &a) == GS_SETEV_TAB && a == 0);   /* R from the last wraps */
    drain_set(h);
    gw_Ui_SetTabs(h, 1, "ONLY", 0); gw_Ui_SetIntent(h, AT_EV_PAGE, 1); CHECK(spoll(h, &s, &a) == 0);   /* one tab: nothing to switch to */
    gw_Ui_SetTabs(h, 0, "", 0); CHECK(slot_of(h)->sc.n_tabs == 0);
    gw_Ui_SetTabs(h, 9, "A,B,C,D,E,F,G,H,I", 7); CHECK(slot_of(h)->sc.n_tabs == AT_MAX_TABS && slot_of(h)->view.tab == AT_MAX_TABS - 1);   /* the record holds six */
    gw_Ui_SetClose(h);
}

static void refocus_by_id(void)                                        /* Review Focus 5 */
{
    int h, i;
    reset_set();
    h = open_set(); tabs6(h, 0); build(h, 6); settle();
    gw_Ui_SetFocus(h, "i4");
    CHECK(strcmp(focused_id(h), "i4") == 0);
    /* a value hid row 4: the page is submitted again without it (ids are table indices, so the rest keep theirs) */
    gw_Ui_SetRows(h, 5); for (i = 0; i < 5; i++) row(h, i, i < 4 ? i : i + 1, AT_VAL_NONE, 0u, "", "");
    CHECK(strcmp(focused_id(h), "i5") == 0);                            /* the next visible row, never nothing and never a stale index */
    gw_Ui_SetFocus(h, "i5");
    gw_Ui_SetRows(h, 2); for (i = 0; i < 2; i++) row(h, i, i, AT_VAL_NONE, 0u, "", "");
    CHECK(strcmp(focused_id(h), "i1") == 0);                            /* the last row when the focused one was past the new end */
    gw_Ui_SetRows(h, 0);
    CHECK(gw_Ui_SetFocused(h) == -1 && focused_id(h)[0] == '\0');       /* no rows: no focus, no crash */
    g_quads = 0; draw_once(); CHECK(g_quads > 20);                      /* and it still draws (the pane, the tabs) */
    gw_Ui_SetIntent(h, AT_EV_ACCEPT, 0); gw_Ui_SetIntent(h, AT_EV_MOVE, AT_DIR_DOWN); gs_ui_tick();    /* input with no rows: nothing to act on, no crash */
    drain_set(h);
    build(h, 6);
    CHECK(strcmp(focused_id(h), "i0") == 0);                            /* the rows are back: the first one */
    /* the same page every frame changes nothing */
    gw_Ui_SetFocus(h, "i2");
    for (i = 0; i < 5; i++) { build(h, 6); }
    CHECK(strcmp(focused_id(h), "i2") == 0);
    /* a focus that moved by the pad survives a resubmit and a draw */
    gw_Ui_SetIntent(h, AT_EV_MOVE, AT_DIR_DOWN); settle(); gw_Ui_SetIntent(h, AT_EV_MOVE, AT_DIR_DOWN);
    CHECK(strcmp(focused_id(h), "i3") == 0 || strcmp(focused_id(h), "i4") == 0);
    { char keep[24]; snprintf(keep, sizeof keep, "%s", focused_id(h)); build(h, 6); draw_once(); build(h, 6); CHECK(strcmp(focused_id(h), keep) == 0); }
    gw_Ui_SetClose(h);
}

static void events_from_the_pad(void)
{
    int h, s, a;
    reset_set();
    h = open_set(); tabs6(h, 0); build(h, 6);
    gw_Ui_SetFocus(h, "i0");
    gw_Ui_SetIntent(h, AT_EV_ACCEPT, 0);
    CHECK(spoll(h, &s, &a) == 0);                                       /* the press of the frame the screen opened is not its own */
    settle();
    gw_Ui_SetIntent(h, AT_EV_MOVE, AT_DIR_DOWN);
    CHECK(spoll(h, &s, &a) == GS_SETEV_FOCUS && s == 1 && gw_Ui_SetFocused(h) == 1);
    gw_Ui_SetIntent(h, AT_EV_MOVE, AT_DIR_RIGHT); CHECK(spoll(h, &s, &a) == GS_SETEV_CHANGE && s == 1 && a == 1);       /* a slider: a change, the direction */
    gw_Ui_SetIntent(h, AT_EV_MOVE, AT_DIR_LEFT);  CHECK(spoll(h, &s, &a) == GS_SETEV_CHANGE && s == 1 && a == -1);
    gw_Ui_SetIntent(h, AT_EV_MOVE, AT_DIR_DOWN); drain_set(h);
    gw_Ui_SetIntent(h, AT_EV_MOVE, AT_DIR_RIGHT); CHECK(spoll(h, &s, &a) == 0);                                          /* an action: left and right do nothing */
    gw_Ui_SetIntent(h, AT_EV_ACCEPT, 0); CHECK(spoll(h, &s, &a) == GS_SETEV_ACCEPT && s == 2);
    gw_Ui_SetIntent(h, AT_EV_BACK, 0); CHECK(spoll(h, &s, &a) == GS_SETEV_BACK && s == -1);
    gw_Ui_SetIntent(h, AT_EV_ALT, 'X'); CHECK(spoll(h, &s, &a) == GS_SETEV_ALT && s == 2 && a == 'X');
    gw_Ui_SetFocus(h, "i4"); gw_Ui_SetIntent(h, AT_EV_MOVE, AT_DIR_RIGHT); CHECK(spoll(h, &s, &a) == 0);               /* a readout never changes */
    gw_Ui_SetFocus(h, "i0"); gw_Ui_SetIntent(h, AT_EV_MOVE, AT_DIR_UP);
    CHECK(spoll(h, &s, &a) == GS_SETEV_FOCUS && s == 5 && gw_Ui_SetFocused(h) == 5);                                      /* Up from the first row wraps to the last */
    gw_Ui_SetIntent(h, AT_EV_MOVE, AT_DIR_DOWN); CHECK(spoll(h, &s, &a) == GS_SETEV_FOCUS && s == 0);                    /* and Down from the last wraps to the first */
    gw_Ui_SetIntent(h, AT_EV_MOVE, AT_DIR_DOWN); gw_Ui_SetIntent(h, 99, 0); gw_Ui_SetIntent(h, AT_EV_NONE, 0); drain_set(h);   /* an unknown intent is nothing */
    CHECK(spoll(h, &s, &a) == 0);
    /* a toggle changes on left, right and A: the screen reports A as an accept, the walker decides (fe_change +1) */
    gw_Ui_SetFocus(h, "i3"); gw_Ui_SetIntent(h, AT_EV_MOVE, AT_DIR_LEFT); CHECK(spoll(h, &s, &a) == GS_SETEV_CHANGE && s == 3 && a == -1);
    /* intents are the game's alone: a mod or the generic menu path does not feed this screen */
    gw_Ui_Intent(AT_EV_ACCEPT, 0); gw_Ui_Intent(AT_EV_BACK, 0); CHECK(spoll(h, &s, &a) == 0 && gs_ui_qn == 0);
    gw_Ui_SetClose(h);
    gw_Ui_SetIntent(h, AT_EV_ACCEPT, 0); CHECK(spoll(h, &s, &a) == 0);                                                    /* a closed handle: nothing */
}

static void keys_become_events(void)
{
    int h, s, a;
    reset_set();
    h = open_set(); tabs6(h, 0); build(h, 6); gw_Ui_SetFocus(h, "i0"); settle();
    gs.key_now[VK_DOWN] = 1; gs_ui_tick(); gs.key_now[VK_DOWN] = 0; gs_ui_tick();
    CHECK(spoll(h, &s, &a) == GS_SETEV_FOCUS && s == 1);
    gs.key_now[VK_RIGHT] = 1; gs_ui_tick(); gs.key_now[VK_RIGHT] = 0; gs_ui_tick();
    CHECK(spoll(h, &s, &a) == GS_SETEV_CHANGE && s == 1 && a == 1);
    gs.key_now[VK_RETURN] = 1; gs_ui_tick(); gs.key_now[VK_RETURN] = 0; gs_ui_tick(); CHECK(spoll(h, &s, &a) == GS_SETEV_ACCEPT && s == 1);
    gs.key_now[VK_ESCAPE] = 1; gs_ui_tick(); gs.key_now[VK_ESCAPE] = 0; gs_ui_tick(); CHECK(spoll(h, &s, &a) == GS_SETEV_BACK);
    gs.key_now[VK_TAB] = 1; gs_ui_tick(); gs.key_now[VK_TAB] = 0; gs_ui_tick(); CHECK(spoll(h, &s, &a) == GS_SETEV_TAB && a == 1);
    gs.key_now[VK_SHIFT] = 1; gs.key_now[VK_TAB] = 1; gs_ui_tick(); gs.key_now[VK_TAB] = 0; gs.key_now[VK_SHIFT] = 0; gs_ui_tick(); CHECK(spoll(h, &s, &a) == GS_SETEV_TAB && a == 5);
    drain_set(h);
    gw_Ui_SetClose(h);
}

/* Review Focus 4: text entry swallowed by the screen. While a name is typed, Enter and Escape are the text's, not the screen's. */
static void freeze_swallows_everything(void)
{
    int h, s, a;
    float x, y;
    reset_set();
    h = open_set(); tabs6(h, 0); build(h, 6); gw_Ui_SetFocus(h, "i2"); settle(); draw_once();
    gw_Ui_Freeze(h, 1);
    CHECK(gs_set.frozen == 1);
    gs.key_now[VK_RETURN] = 1; gs.key_now[VK_ESCAPE] = 1; gs_ui_tick();
    gw_Ui_SetIntent(h, AT_EV_ACCEPT, 0); gw_Ui_SetIntent(h, AT_EV_BACK, 0); gw_Ui_SetIntent(h, AT_EV_MOVE, AT_DIR_DOWN); gw_Ui_SetIntent(h, AT_EV_PAGE, 1);
    rect_of_row(h, 1, &x, &y); g_mx = x; g_my = y; gs_ui_tick(); g_mbuttons = 1; gs_ui_tick(); g_mbuttons = 0; gs_ui_tick(); g_mx = x + 1.0f; gs_ui_tick();
    CHECK(spoll(h, &s, &a) == 0);                                       /* nothing reaches the adapter while a name is being typed */
    CHECK(strcmp(focused_id(h), "i2") == 0);                            /* and the focus did not move */
    draw_once(); g_quads = 0; draw_once(); CHECK(g_quads > 80);          /* a frozen screen still draws */
    g_mx = g_my = -1000.0f; gs_ui_tick();
    gw_Ui_Freeze(h, 0);                                                 /* the edit ended: Enter and Escape are STILL HELD from that frame */
    gw_Ui_SetIntent(h, AT_EV_ACCEPT, 0);                                /* the press that ended the edit may arrive as an intent in the same frame */
    CHECK(spoll(h, &s, &a) == 0);
    gs_ui_tick();
    CHECK(spoll(h, &s, &a) == 0);                                       /* a key held across a freeze primes from the held state: no stray accept or back */
    gs.key_now[VK_RETURN] = 0; gs.key_now[VK_ESCAPE] = 0; gs_ui_tick();
    gs.key_now[VK_RETURN] = 1; gs_ui_tick(); gs.key_now[VK_RETURN] = 0;
    CHECK(spoll(h, &s, &a) == GS_SETEV_ACCEPT && s == 2);               /* a fresh press does */
    gw_Ui_SetIntent(h, AT_EV_BACK, 0); CHECK(spoll(h, &s, &a) == GS_SETEV_BACK);
    gw_Ui_Freeze(h, 1); gw_Ui_Freeze(h, 1); gw_Ui_Freeze(h, 0); gw_Ui_Freeze(h, 0); CHECK(gs_set.frozen == 0);   /* idempotent */
    gw_Ui_SetClose(h);
    gw_Ui_Freeze(h, 1); CHECK(gs_set.frozen == 0);                      /* a closed handle freezes nothing */
}

static void dialog_defaults_to_cancel(void)                            /* Review Focus 6 */
{
    int h, s, a;
    float x, y;
    reset_set();
    h = open_set(); build(h, 6); gw_Ui_SetFocus(h, "i0"); settle();
    gw_Ui_SetDialog(h, "ERASE DATA", "This cannot be undone.", "Erase", "Cancel", 2);
    CHECK(slot_of(h)->view.dialog.open == 1 && slot_of(h)->view.dialog.focus == 1 && slot_of(h)->view.dialog.n == 2 && slot_of(h)->view.dialog.btn[0] == 'A' && slot_of(h)->view.dialog.btn[1] == 'B');
    CHECK(strcmp(slot_of(h)->view.dialog.label[0], "Erase") == 0 && strcmp(slot_of(h)->view.dialog.label[1], "Cancel") == 0);   /* Cancel is the focused one */
    CHECK(gw_Ui_SetDialogAnswer(h) == 0);
    gw_Ui_SetIntent(h, AT_EV_ACCEPT, 0);                                /* a stray press of the frame it opened: must not answer */
    CHECK(gw_Ui_SetDialogAnswer(h) == 0 && slot_of(h)->view.dialog.open == 1);
    settle();
    gw_Ui_SetIntent(h, AT_EV_MOVE, AT_DIR_DOWN); gw_Ui_SetIntent(h, AT_EV_PAGE, 1);
    CHECK(strcmp(focused_id(h), "i0") == 0 && spoll(h, &s, &a) == 0);   /* the rows take nothing while the dialog is up */
    gw_Ui_SetIntent(h, AT_EV_ACCEPT, 0);                                /* A answers the FOCUSED button: Cancel */
    CHECK(gw_Ui_SetDialogAnswer(h) == 2 && slot_of(h)->view.dialog.open == 0);
    CHECK(gw_Ui_SetDialogAnswer(h) == 0);                               /* taken once */
    gw_Ui_SetDialog(h, "ERASE DATA", "x", "Erase", "Cancel", 2); settle();
    gw_Ui_SetIntent(h, AT_EV_MOVE, AT_DIR_LEFT); CHECK(slot_of(h)->view.dialog.focus == 0);          /* left moves to Erase ... */
    gw_Ui_SetIntent(h, AT_EV_MOVE, AT_DIR_RIGHT); CHECK(slot_of(h)->view.dialog.focus == 1);         /* ... right back to Cancel */
    gw_Ui_SetIntent(h, AT_EV_MOVE, AT_DIR_LEFT); gw_Ui_SetIntent(h, AT_EV_ACCEPT, 0);
    CHECK(gw_Ui_SetDialogAnswer(h) == 1 && slot_of(h)->view.dialog.open == 0);                       /* a deliberate move and A: Erase */
    gw_Ui_SetDialog(h, "ERASE DATA", "x", "Erase", "Cancel", 2); settle();
    gw_Ui_SetIntent(h, AT_EV_MOVE, AT_DIR_LEFT); gw_Ui_SetIntent(h, AT_EV_BACK, 0);
    CHECK(gw_Ui_SetDialogAnswer(h) == 2);                               /* B is always Cancel, whatever is focused */
    gw_Ui_SetDialog(h, "T", "x", "OK", "", 1); CHECK(slot_of(h)->view.dialog.n == 1 && slot_of(h)->view.dialog.focus == 0); settle();   /* one button: it is the focus */
    gw_Ui_SetIntent(h, AT_EV_BACK, 0); CHECK(gw_Ui_SetDialogAnswer(h) == 2 && slot_of(h)->view.dialog.open == 0);   /* B closes it as a cancel */
    /* the mouse: only the dialog's buttons are hit, and a click answers */
    gw_Ui_SetDialog(h, "ERASE DATA", "x", "Erase", "Cancel", 2); draw_once();
    { int i, rows = 0; for (i = 0; i < slot_of(h)->hits.n; i++) rows += slot_of(h)->hits.h[i].kind == AT_HIT_CELL; CHECK(rows == 0); }       /* no row can be reached */
    rect_of_hit(h, AT_HIT_DIALOG, 'A', &x, &y); g_mx = x; g_my = y; settle(); g_mbuttons = 1; settle(); g_mbuttons = 0; settle();
    CHECK(gw_Ui_SetDialogAnswer(h) == 1);
    CHECK(strcmp(focused_id(h), "i0") == 0 && spoll(h, &s, &a) == 0);
    g_mx = g_my = -1000.0f;
    gw_Ui_SetDialog(h, "LONG", "a body that is longer than the dialog holds, repeated and repeated and repeated and repeated and repeated and repeated and repeated and repeated and repeated and repeated and repeated", "Erase everything and more", "Cancel", 2);
    draw_once(); CHECK(slot_of(h)->view.dialog.open == 1);              /* a long body and label draw without crash */
    gw_Ui_SetClose(h);
}

static void owner_native_vs_mod(void)                                  /* Review Focus 7 */
{
    int h, h2, s, a;
    reset_set();
    h = open_set();
    fake_script(3, "envoy");
    gs.cur = 3;                                                            /* a mod caller */
    CHECK(!t_lua("gd.ui.close('settings.tabs')") && strstr(t_lua_err(), "belongs to the engine") != NULL);
    CHECK(!t_lua("gd.ui.feed('settings.tabs','accept')") && !t_lua("gd.ui.set_focus('settings.tabs','list','i0')") && !t_lua("gd.ui.open('settings.tabs')"));
    CHECK(!t_lua("gd.ui.set_value('settings.tabs','i0',true)") && !t_lua("gd.ui.value('settings.tabs','i0')"));
    CHECK(!t_lua("gd.ui.screen{ id='settings.tabs', primary={kind='list', items={{id='a', label='A'}}} }"));   /* nor take its id */
    CHECK(t_lua("gd.ui.screen{ id='envoy.over', primary={kind='list', items={{id='a', label='A'}}}, on={back=function() return {pop=true} end} }"));
    t_lua("gd.ui.open('envoy.over')");
    CHECK(gw_Ui_TopIsEngine("settings.tabs") == 1 && gs_ui_stack.n == 1);  /* no mod screen over the settings */
    gs.cur = gs.console;                                                   /* the console */
    CHECK(!t_lua("gd.ui.close('settings.tabs')") && !t_lua("gd.ui.feed('settings.tabs','accept')"));
    CHECK(gw_Ui_TopIsEngine("settings.tabs") == 1);
    gs_ui_release(3); gs_ui_release(GS_UI_ENGINE); gs_ui_release(-1);      /* a mod unloading never reaches it */
    CHECK(gw_Ui_TopIsEngine("settings.tabs") == 1);
    gs.cur = 0;
    gs_ui_tick(); CHECK(gw_Ui_TopIsEngine("settings.tabs") == 1);          /* a tick with a mod loaded: nothing closed */
    /* closed on every scene exit (the step 4 rule) */
    gw_Ui_SceneExit(gs.scene_kind);
    CHECK(gs_ui_stack.n == 0 && slot_of(h)->used == 0);
    CHECK(gw_Ui_SetFocused(h) == -1 && spoll(h, &s, &a) == 0);             /* a stale handle is harmless */
    gw_Ui_SetRows(h, 3); gw_Ui_SetIntent(h, AT_EV_ACCEPT, 0); gw_Ui_SetTabs(h, 2, "A,B", 1); gw_Ui_SetDialog(h, "x", "x", "A", "B", 2); gw_Ui_SetClose(h);
    CHECK(gs_ui_stack.n == 0);
    h2 = open_set(); CHECK(h2 >= 0 && h2 != h);                            /* a second open after the exit works */
    gw_Ui_SetRows(h, 3); CHECK(slot_of(h2)->sc.n_items == 0);              /* the old handle names nothing now, even though its slot is in use again */
    gw_Ui_SetClose(h); CHECK(gs_ui_stack.n == 1);                           /* and cannot close the new one */
    gw_Ui_SetClose(h2);
    /* a screen pushed in another scene is not closed by this scene's exit */
    h = open_set(); gw_Ui_SceneExit(gs.scene_kind + 1); CHECK(gs_ui_stack.n == 1); gw_Ui_SceneExit(gs.scene_kind); CHECK(gs_ui_stack.n == 0);
    (void) h;
}

static void mouse_and_wheel(void)
{
    int h, s, a;
    float x, y;
    reset_set();
    g_mbuttons = 1;                                                        /* a button held from the menu that opened the scene */
    h = open_set(); tabs6(h, 0); build(h, 6); gw_Ui_SetKeys(h, "A:Change,L:Page,B:Back"); gw_Ui_SetFocus(h, "i0"); draw_once();
    rect_of_row(h, 2, &x, &y); g_mx = x; g_my = y;
    gs_ui_tick();                                                          /* the first sample only primes */
    CHECK(spoll(h, &s, &a) == 0);
    g_mbuttons = 0; gs_ui_tick(); CHECK(spoll(h, &s, &a) == 0);            /* releasing is nothing */
    g_mx += 1.0f; gs_ui_tick();                                            /* a hover: the pointer moved onto row 2 */
    CHECK(spoll(h, &s, &a) == GS_SETEV_FOCUS && s == 2 && gw_Ui_SetFocused(h) == 2);
    gs_ui_tick(); CHECK(spoll(h, &s, &a) == 0);                            /* a resting pointer makes no event: it does not steal the focus from a pad */
    gw_Ui_SetIntent(h, AT_EV_MOVE, AT_DIR_DOWN); drain_set(h); gs_ui_tick(); CHECK(gw_Ui_SetFocused(h) == 3);   /* the pad moved on and the resting pointer left it there */
    g_mbuttons = 1; gs_ui_tick();                                          /* a click on row 2 (the pointer is still on it): focus, then A */
    CHECK(spoll(h, &s, &a) == GS_SETEV_FOCUS && s == 2);
    CHECK(spoll(h, &s, &a) == GS_SETEV_ACCEPT && s == 2);
    g_mbuttons = 0; gs_ui_tick(); drain_set(h);
    g_mbuttons = 2; gs_ui_tick(); CHECK(spoll(h, &s, &a) == GS_SETEV_BACK);   /* right click: B */
    g_mbuttons = 0; gs_ui_tick(); drain_set(h);
    g_mx = -1000.0f; g_my = -1000.0f; gs_ui_tick(); g_mbuttons = 1; gs_ui_tick(); CHECK(spoll(h, &s, &a) == 0);   /* a click off the picture is nothing */
    g_mbuttons = 0; gs_ui_tick();
    /* the wheel steps the hovered value row, and scrolls the focus over anything else */
    rect_of_row(h, 1, &x, &y); g_mx = x; g_my = y; gs_ui_tick(); drain_set(h);
    g_mwheel = 1.0f; gs_ui_tick(); g_mwheel = 0.0f;
    CHECK(spoll(h, &s, &a) == GS_SETEV_FOCUS && s == 1);
    CHECK(spoll(h, &s, &a) == GS_SETEV_CHANGE && s == 1 && a == 1);        /* wheel up: right */
    g_mwheel = -1.0f; gs_ui_tick(); g_mwheel = 0.0f; CHECK(spoll(h, &s, &a) == GS_SETEV_CHANGE && s == 1 && a == -1);
    rect_of_row(h, 2, &x, &y); g_mx = x; g_my = y; gs_ui_tick(); drain_set(h);                     /* over an action */
    g_mwheel = -1.0f; gs_ui_tick(); g_mwheel = 0.0f;
    CHECK(spoll(h, &s, &a) == GS_SETEV_FOCUS && s == 3 && gw_Ui_SetFocused(h) == 3);                 /* the wheel scrolls the focus down */
    drain_set(h);
    /* a tab and a key hint */
    rect_of_hit(h, AT_HIT_TAB, 2, &x, &y); g_mx = x; g_my = y; gs_ui_tick(); drain_set(h);
    g_mbuttons = 1; gs_ui_tick(); CHECK(spoll(h, &s, &a) == GS_SETEV_TAB && a == 2); g_mbuttons = 0; gs_ui_tick(); drain_set(h);
    rect_of_hit(h, AT_HIT_KEY, 'B', &x, &y); g_mx = x; g_my = y; gs_ui_tick(); drain_set(h);
    g_mbuttons = 1; gs_ui_tick(); CHECK(spoll(h, &s, &a) == GS_SETEV_BACK); g_mbuttons = 0; gs_ui_tick(); drain_set(h);
    rect_of_hit(h, AT_HIT_KEY, 'L', &x, &y); g_mx = x; g_my = y; gs_ui_tick(); drain_set(h);
    g_mbuttons = 1; gs_ui_tick(); CHECK(spoll(h, &s, &a) == GS_SETEV_TAB && a == 5); g_mbuttons = 0; gs_ui_tick(); drain_set(h);   /* the L hint pages back (wraps) */
    rect_of_hit(h, AT_HIT_KEY, 'A', &x, &y); g_mx = x; g_my = y; gs_ui_tick(); drain_set(h);
    g_mbuttons = 1; gs_ui_tick(); CHECK(spoll(h, &s, &a) == GS_SETEV_ACCEPT); g_mbuttons = 0; gs_ui_tick(); drain_set(h);
    /* the ring never grows past 32 and drops the newest */
    { int i; rect_of_row(h, 1, &x, &y); g_mx = x; g_my = y; gs_ui_tick(); drain_set(h); for (i = 0; i < 50; i++) { g_mwheel = 1.0f; gs_ui_tick(); } g_mwheel = 0.0f; CHECK(gs_set.qn == 32); }   /* 50 steps of a slider */
    g_mx = g_my = -1000.0f;
    gw_Ui_SetClose(h);
    gs_ui_tick(); CHECK(gs_set.qn == 0);
}

static void poll_is_big_endian(void)
{
    int h;
    unsigned raw[2];
    const unsigned char *p;
    reset_set();
    h = open_set();
    gs_set_push(GS_SETEV_TAB, 0x01020304, 0x0A0B0C0D);
    CHECK(gw_Ui_SetPoll(h, (int *) &raw[0], (int *) &raw[1]) == GS_SETEV_TAB);
    CHECK(raw[0] == bswap_p(0x01020304u) && raw[1] == bswap_p(0x0A0B0C0Du));                              /* the game's locals are big-endian: raw bytes 01 02 03 04 ... */
    p = (const unsigned char *) &raw[0]; CHECK(p[0] == 1 && p[1] == 2 && p[2] == 3 && p[3] == 4);
    p = (const unsigned char *) &raw[1]; CHECK(p[0] == 0x0A && p[1] == 0x0B && p[2] == 0x0C && p[3] == 0x0D);
    gs_set_push(GS_SETEV_CHANGE, -1, -1);
    CHECK(gw_Ui_SetPoll(h, (int *) &raw[0], (int *) &raw[1]) == GS_SETEV_CHANGE && raw[0] == 0xFFFFFFFFu && raw[1] == 0xFFFFFFFFu);   /* negatives: every byte */
    CHECK(gw_Ui_SetPoll(h, (int *) &raw[0], (int *) &raw[1]) == 0);
    gw_Ui_SetClose(h);
}

static void explainer_text_and_clamp(void)                             /* Review Focus 9 */
{
    int h, i, seen = 0;
    char big[600];
    AtLayout L;
    AtExplainer *e;
    reset_set();
    h = open_set(); tabs6(h, 4); build(h, 6);
    gw_Ui_SetFocus(h, "i1");
    gw_Ui_SetExplain(h, NULL, NULL, NULL, NULL, "");                       /* the page's rows speak for themselves */
    e = &slot_of(h)->view.ex;
    CHECK(e->has && strcmp(e->kicker, "GAME") == 0 && strcmp(e->title, "Row 1") == 0 && strcmp(e->what, "Help for row 1.") == 0);   /* the tab, the row, its help */
    CHECK(strcmp(e->now_text, "NOW 40%") == 0 && e->no_well == 1 && e->media_model == AT_NO_MODEL);                                  /* the value the row shows; no media well */
    gw_Ui_SetFocus(h, "i0"); gw_Ui_SetExplain(h, NULL, NULL, NULL, NULL, "");
    CHECK(strcmp(e->now_text, "NOW ON") == 0);                                                                                       /* a toggle reads ON or OFF */
    gw_Ui_SetFocus(h, "i4"); gw_Ui_SetExplain(h, NULL, NULL, NULL, NULL, "");
    CHECK(e->now_text[0] == '\0');                                                                                                   /* a readout: its text is on the row already */
    gw_Ui_SetExplain(h, "MODS", "A MOD", "Hello.", "ON", "mods/x");
    CHECK(strcmp(e->kicker, "MODS") == 0 && strcmp(e->title, "A MOD") == 0 && strcmp(e->what, "Hello.") == 0 && strcmp(e->now_text, "NOW ON") == 0 && strcmp(e->from_text, "mods/x") == 0);
    memset(big, 'x', sizeof big - 1); big[sizeof big - 1] = '\0';
    gw_Ui_SetExplain(h, "MODS", "A MOD", big, "", "");
    CHECK(strlen(e->what) <= AT_TEXT - 1 && strstr(e->what, "...") != NULL && e->now_text[0] == '\0');                              /* clamped with an ellipsis, not cut silently */
    draw_once();
    { AtSink s = rec_sink(); AtRect ex; at_layout(640.0f, AT_PRESET_NORMAL, &L); ex = L.explainer;
      at_render(&slot_of(h)->sc, &slot_of(h)->view, 640.0f, 10000.0, 1, &FAKE, &s, &slot_of(h)->hits);
      CHECK(texts_legible());
      for (i = 0; i < REC.nt; i++) if (strncmp(REC.t[i].s, "xxx", 3) == 0) {                                                       /* the lines of the clamped rule text */
          CHECK(text_left(&REC.t[i]) >= ex.x - 0.01f && text_right(&REC.t[i]) <= ex.x + ex.w + 0.01f); seen++; }
      CHECK(seen >= 1 && seen <= 4); }                                                                                              /* wrapped to at most 4 lines, all inside the explainer */
    /* a help line from a row (up to 512 characters in a mod) is clamped the same way when the row is submitted */
    gw_Ui_SetRows(h, 1); gw_Ui_SetRow(h, 0, "i0", "Mod", big, AT_VAL_TOGGLE); gw_Ui_SetFocus(h, "i0"); gw_Ui_SetExplain(h, NULL, NULL, NULL, NULL, "");
    CHECK(strlen(e->what) <= AT_TEXT - 1 && strstr(e->what, "...") != NULL);
    gw_Ui_SetClose(h);
}

static void keys_hints_and_notes(void)
{
    int h;
    GsUiSlot *u;
    reset_set();
    h = open_set(); u = slot_of(h);
    gw_Ui_SetKeys(h, "A:Change,L:Page,B:Back");
    CHECK(u->sc.n_keys == 3 && u->sc.keys[0].btn == 'A' && strcmp(u->view.key_label[1], "Page") == 0 && u->view.key_shown[2] == 1);
    gw_Ui_SetKeys(h, "L:Page,B:Back"); CHECK(u->sc.n_keys == 2 && u->sc.keys[0].btn == 'L');          /* a changed set replaces the old */
    gw_Ui_SetKeys(h, "L:Page,B:Back"); CHECK(u->sc.n_keys == 2);                                         /* the same set is not parsed again (no per-frame cost) */
    gw_Ui_SetKeys(h, ""); CHECK(u->sc.n_keys == 0);
    gw_Ui_SetKeys(h, "A:One,B:Two,X:Three,Y:Four,L:Five,R:Six,Z:Seven"); CHECK(u->sc.n_keys == AT_MAX_KEYS);   /* six fit */
    gw_Ui_SetKeys(h, "Q:Nope,A"); CHECK(u->sc.n_keys == 0);                                              /* a bad spec is nothing */
    gw_Ui_SetNote(h, "Erased.", AT_NOTE_OK);
    CHECK(strcmp(u->view.note.text, "Erased.") == 0 && u->view.note.kind == AT_NOTE_OK && u->view.note.until_ms > u->view.note.from_ms);
    gw_Ui_SetNote(h, "", AT_NOTE_OK); CHECK(strcmp(u->view.note.text, "Erased.") == 0);                   /* an empty note changes nothing */
    gw_Ui_SetNote(h, "x", 77); CHECK(u->view.note.kind == AT_NOTE_INFO);
    gw_Ui_SetClose(h);
}

static void mods_41_rows_scroll(void)                                  /* Review Focus 9 */
{
    int h, i;
    char id[12], label[24];
    reset_set();
    h = open_set(); tabs6(h, 5);
    gw_Ui_SetRows(h, 41);
    for (i = 0; i < 41; i++) { snprintf(id, sizeof id, "i%d", i); snprintf(label, sizeof label, "Mod %d", i); gw_Ui_SetRow(h, i, id, label, "a mod", AT_VAL_TOGGLE); gw_Ui_SetRowVal(h, i, 0, 1, 1, i & 1, AT_ITEM_A_STEPS); gw_Ui_SetRowText(h, i, "", "", ""); }
    gw_Ui_SetFocus(h, "i40"); settle(); draw_once();
    CHECK(slot_of(h)->sc.n_items == 41 && slot_of(h)->view.scroll > 0 && slot_of(h)->view.scroll <= 40);   /* the window followed the focus */
    { int k, found = 0; for (k = 0; k < slot_of(h)->hits.n; k++) if (slot_of(h)->hits.h[k].kind == AT_HIT_CELL && slot_of(h)->hits.h[k].b == 40) found = 1; CHECK(found); }   /* the 41st is on screen: reachable */
    gw_Ui_SetFocus(h, "i0"); draw_once(); CHECK(slot_of(h)->view.scroll == 0);
    for (i = 0; i < 40; i++) { gw_Ui_SetIntent(h, AT_EV_MOVE, AT_DIR_DOWN); drain_set(h); settle(); }
    CHECK(strcmp(focused_id(h), "i40") == 0);                               /* walking all the way down with the pad */
    gw_Ui_SetClose(h);
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
    filled_and_drawn(); one_native_screen_at_a_time(); rows_land_where_the_render_reads_them(); tabs_and_focus(); refocus_by_id(); events_from_the_pad();
    keys_become_events(); freeze_swallows_everything(); dialog_defaults_to_cancel(); owner_native_vs_mod(); mouse_and_wheel(); poll_is_big_endian();
    explainer_text_and_clamp(); keys_hints_and_notes(); mods_41_rows_scroll();
    lua_close(L);
    ATLAS_DONE("atlas settings host");
}
