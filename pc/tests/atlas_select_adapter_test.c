/* Host-side test of the select screens' native half (gw_script_ui_sel.inc): the Ui_Sel* shims the game-side adapter calls, driven the way the adapter drives
 * them, against the real render, the real screen stack and a real Lua state for the owner checks. gw_script_ui.inc is included here against the same few
 * stand-ins atlas_binding_test.c uses (the script table, the kit's draw calls). No game, no renderer, no window.
 * The test reaches the host's statics directly (this file includes the .inc), so there are no exported test accessors. */
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
    gs_sel.h = -1; gs_sel.qn = 0; gs_ui_adopt_next = 0;
    g_mx = g_my = -1000.0f; g_mbuttons = 0; g_mwheel = 0.0f; g_art_calls = 0; g_images = 0;
    for (i = 1; i < 8; i++) gs.s[i].used = 0;
}
static void fake_script(int i, const char *mod) { snprintf(gs.s[i].id, sizeof gs.s[i].id, "%s/main", mod); gs.s[i].used = 1; gs.s[i].disabled = 0; }

static int open_css(void) { return gw_Ui_SelOpen(0, 0, "VERSUS", "MELEE", "FIGHTERS"); }
static void fill_css(int h, int n)
{
    int i;
    gw_Ui_SelTab(h, 0, "ALL", n, 1); gw_Ui_SelTab(h, 1, "RETAIL", n - 3, 0); gw_Ui_SelTab(h, 2, "ADDED", 3, 0);
    gw_Ui_SelBeginCells(h, n);
    for (i = 0; i < n; i++) { char id[8]; snprintf(id, sizeof id, "f%d", i); gw_Ui_SelCell(h, i, id, "NAME", "NA", -1, 0u, 0); }
    gw_Ui_SelCursor(h, 0, 1, 3, -1);
}
static GsUiSlot *slot_of(int h) { return &gs_ui_slot[h & 0xFF]; }
static void draw_once(void) { gs_ui_draw(); }

static void filled_and_drawn(void)
{
    int h, i, cells = 0, tabs = 0, cards = 0;
    GsUiSlot *u;
    reset_ui();
    h = open_css();
    CHECK(h >= 0 && gw_Ui_TopIsEngine("select.css") == 1);
    fill_css(h, 29);
    gw_Ui_SelCard(h, 0, 1, "FOX", "Costume 1", "FO", -1, 0, 0, 0u);
    g_quads = 0; draw_once();
    CHECK(g_quads > 100 && g_quads < AT_SCREEN_QUAD_WARN);                          /* it draws: the entries it spent */
    u = slot_of(h);
    for (i = 0; i < u->hits.n; i++) { cells += u->hits.h[i].kind == AT_HIT_CELL; tabs += u->hits.h[i].kind == AT_HIT_TAB; cards += u->hits.h[i].kind == AT_HIT_CARD; }
    CHECK(cells == 29 && tabs == 3 && cards == 4);
    CHECK(gw_Ui_SelCols(h) >= 6 && gw_Ui_SelCols(h) <= 11);
    CHECK(u->sc.blocks[0].ext == gs_sel.pool && u->owner == GS_UI_ENGINE && u->native_sel == 1);   /* in place, in adapter storage, engine-owned */
    CHECK(gs_ui_stack.n == 1 && u->scene == 7);
    gw_Ui_SelClose(h);
    CHECK(gs_ui_stack.n == 0 && gw_Ui_TopIsEngine("select.css") == 0 && gs_sel.h == -1);
    /* a draw with the tab strip dropped to one tab and with cells past the pool */
    h = open_css(); gw_Ui_SelBeginCells(h, 100000); CHECK(slot_of(h)->sc.blocks[0].ext_n == AT_MAX_EXT_CELLS);
    gw_Ui_SelBeginCells(h, -3); CHECK(slot_of(h)->sc.blocks[0].ext_n == 0);
    gw_Ui_SelCell(h, 0, "x", "x", "xx", -1, 0u, 0); CHECK(gs_sel.pool[0].id[0] == '\0');   /* a write past the count is ignored */
    gw_Ui_SelTab(h, 0, "ALL", 1, 1); gw_Ui_SelTab(h, 1, "B", 1, 0); CHECK(slot_of(h)->sc.n_tabs == 2); gw_Ui_SelTab(h, 0, "ALL", 1, 1); CHECK(slot_of(h)->sc.n_tabs == 1);   /* tab 0 starts a new list */
    gw_Ui_SelTab(h, AT_MAX_TABS, "no", 0, 0); gw_Ui_SelTab(h, -1, "no", 0, 0); CHECK(slot_of(h)->sc.n_tabs == 1);
    gw_Ui_SelClose(h);
    /* the loading and stage kinds open too, with their own place of the layout */
    h = gw_Ui_SelOpen(1, 0, "VERSUS", "MELEE", "STAGES"); CHECK(h >= 0 && slot_of(h)->sc.preset == AT_PRESET_NONE && slot_of(h)->sc.band == AT_BAND_NONE && slot_of(h)->sc.grid_cell_min == 54); gw_Ui_SelClose(h);
    h = gw_Ui_SelOpen(2, 0, "VERSUS", "MELEE", "GET READY"); CHECK(h >= 0 && slot_of(h)->sc.band == AT_BAND_MATCHUP); gw_Ui_SelClose(h);
    CHECK(gw_Ui_SelOpen(3, 0, "", "", "") == -1 && gw_Ui_SelOpen(-1, 0, "", "", "") == -1);
    h = gw_Ui_SelOpen(0, 0, "SOLO", "TRAINING", "FIGHTERS"); CHECK(slot_of(h)->sc.chapter == 1 && slot_of(h)->sc.n_parents == 2); gw_Ui_SelClose(h);
}

/* Review Focus 3: a native screen left open must be closed on EVERY way a scene can end, or it draws over the next scene */
static void closed_on_every_exit(void)
{
    int h, h2, k, a, b;
    reset_ui();
    h = open_css();
    CHECK(gw_Ui_TopIsEngine("select.css") == 1);
    gw_Ui_SceneExit(gs.scene_kind);                                       /* the scene change in gw_script.c calls this for every scene that ends */
    CHECK(gs_ui_stack.n == 0 && slot_of(h)->used == 0);                    /* closed without the adapter asking */
    CHECK(poll(h, &k, &a, &b) == 0);                                       /* a stale handle is harmless, not a crash */
    gw_Ui_SelCell(h, 0, "x", "x", "xx", -1, 0u, 0); gw_Ui_SelCursor(h, 0, 1, 1, -1); gw_Ui_SelKeys(h, "A:x"); gw_Ui_SelClose(h);   /* writes to a closed handle are ignored */
    CHECK(gs_ui_stack.n == 0);
    h2 = open_css();                                                       /* a second open after a close works */
    CHECK(h2 >= 0 && h2 != h);
    CHECK(gw_Ui_SelOpen(0, 0, "A", "B", "C") == -1);                       /* only one select at a time */
    gw_Ui_SelCell(h, 0, "x", "x", "xx", -1, 0u, 0);                       /* the old handle names nothing now, even though its slot is in use again */
    gw_Ui_SelBeginCells(h2, 3); gw_Ui_SelCell(h, 1, "old", "old", "OL", -1, 0u, 0); CHECK(gs_sel.pool[1].id[0] == '\0');
    gw_Ui_SelClose(h);                                                     /* closing with the stale handle does not close the new screen */
    CHECK(gs_ui_stack.n == 1);
    gw_Ui_SceneExit(gs.scene_kind + 1);                                    /* another scene's end leaves it */
    CHECK(gs_ui_stack.n == 1);
    gw_Ui_SelClose(h2);
    CHECK(gs_ui_stack.n == 0);
    /* the screen's own record is wiped on the next open: nothing of the last screen's cells or cards shows */
    h = open_css(); fill_css(h, 10); gw_Ui_SelCard(h, 1, 2, "KIRBY", "x", "KI", -1, 9, 0, 0u); gw_Ui_SelClose(h);
    h = open_css(); CHECK(slot_of(h)->sc.ports[1].kind == 0 && slot_of(h)->sc.ports[1].name[0] == '\0' && slot_of(h)->sc.blocks[0].ext_n == 0 && slot_of(h)->sc.n_tabs == 0 && gs_sel.pool[0].id[0] == '\0');
    gw_Ui_SceneExit(gs.scene_kind);
    /* the draw pass after a close draws nothing of it */
    g_quads = 0; draw_once(); CHECK(g_quads == 0);
}

/* Review Focus 9: ownership checked as the native owner, as a mod and as the console, never only as one of them */
static void owner_native_vs_mod(void)
{
    int h;
    reset_ui();
    h = open_css();
    fake_script(3, "envoy");
    gs.cur = 3;                                                            /* a mod caller */
    CHECK(!t_lua("gd.ui.close('select.css')") && strstr(t_lua_err(), "belongs to the engine") != NULL);
    CHECK(!t_lua("gd.ui.feed('select.css','accept')") && !t_lua("gd.ui.set_focus('select.css','fighters','f0')") && !t_lua("gd.ui.open('select.css')"));
    CHECK(!t_lua("gd.ui.screen{ id='select.css', primary={kind='list', items={{id='a', label='A'}}} }"));   /* nor take its id */
    CHECK(t_lua("gd.ui.screen{ id='envoy.over', primary={kind='list', items={{id='a', label='A'}}}, on={back=function() return {pop=true} end} }"));
    t_lua("gd.ui.open('envoy.over')");                                     /* refused or ignored: either way ... */
    CHECK(gw_Ui_TopIsEngine("select.css") == 1 && gs_ui_stack.n == 1);       /* no mod screen over a select */
    gs.cur = gs.console;                                                   /* the console */
    CHECK(!t_lua("gd.ui.close('select.css')") && !t_lua("gd.ui.feed('select.css','accept')"));
    CHECK(gw_Ui_TopIsEngine("select.css") == 1);
    /* the engine's own door (the shims) is the only way to close it, and a mod releasing its screens never reaches it */
    gs_ui_release(3); gs_ui_release(GS_UI_ENGINE); gs_ui_release(-1);
    CHECK(gw_Ui_TopIsEngine("select.css") == 1);
    /* the menu's input path never feeds a select: Ui_Intent does nothing to it */
    gs.cur = 0; gw_Ui_Intent(3, 0); gw_Ui_Intent(4, 0); CHECK(gs_ui_qn == 0 && gs_sel.qn == 0);
    gs_ui_tick();                                                           /* a tick with a mod loaded and no mouse: nothing queued, nothing closed */
    CHECK(gw_Ui_TopIsEngine("select.css") == 1 && gs_sel.qn == 0);
    gw_Ui_SelClose(h);
}

static void rect_of_cell(int h, int idx, float *cx, float *cy)
{
    int i;
    GsUiSlot *u = slot_of(h);
    *cx = *cy = -1000.0f;
    for (i = 0; i < u->hits.n; i++) if (u->hits.h[i].kind == AT_HIT_CELL && u->hits.h[i].b == idx) { *cx = u->hits.h[i].r.x + u->hits.h[i].r.w * 0.5f; *cy = u->hits.h[i].r.y + u->hits.h[i].r.h * 0.5f; }
}
static void rect_of(int h, int kind, int a, float *cx, float *cy)
{
    int i;
    GsUiSlot *u = slot_of(h);
    *cx = *cy = -1000.0f;
    for (i = 0; i < u->hits.n; i++) if (u->hits.h[i].kind == kind && u->hits.h[i].a == a) { *cx = u->hits.h[i].r.x + u->hits.h[i].r.w * 0.5f; *cy = u->hits.h[i].r.y + u->hits.h[i].r.h * 0.5f; }
}
static void drain(int h) { int k, a, b; while (poll(h, &k, &a, &b)) {} }
static void mouse_port(void)
{
    int h, k, a, b;
    float x, y;
    reset_ui();
    g_mbuttons = 1;                                                        /* a button held from the menu that opened the scene */
    h = open_css(); fill_css(h, 29); draw_once();
    rect_of_cell(h, 5, &x, &y);
    g_mx = x; g_my = y;
    gs_ui_tick();                                                          /* the first sample only primes */
    CHECK(poll(h, &k, &a, &b) == 0);
    g_mbuttons = 0; gs_ui_tick(); CHECK(poll(h, &k, &a, &b) == 0);         /* releasing is nothing */
    g_mbuttons = 1; gs_ui_tick();                                          /* left click on tile 5 */
    CHECK(poll(h, &k, &a, &b) == 1 && k == 1 && a == 5 && b == 1);
    CHECK(poll(h, &k, &a, &b) == 0);
    g_mbuttons = 0; gs_ui_tick();
    g_mx = -1000.0f; g_my = -1000.0f; gs_ui_tick(); CHECK(poll(h, &k, &a, &b) == 0);   /* off the picture: nothing */
    g_mbuttons = 1; gs_ui_tick(); CHECK(poll(h, &k, &a, &b) == 0);         /* a click off the picture is nothing, too */
    g_mbuttons = 0; gs_ui_tick();
    gs_ui_tick(); rect_of_cell(h, 7, &x, &y); g_mx = x; g_my = y;           /* the pointer re-enters (primes) and then moves onto tile 7: a hover */
    gs_ui_tick(); CHECK(poll(h, &k, &a, &b) == 0);
    g_mx += 1.0f; gs_ui_tick(); CHECK(poll(h, &k, &a, &b) == 1 && k == 1 && a == 7 && b == 0);
    gs_ui_tick(); CHECK(poll(h, &k, &a, &b) == 0);                         /* a resting pointer makes no event: it does not steal the cursor from a pad */
    g_mbuttons = 2; gs_ui_tick(); CHECK(poll(h, &k, &a, &b) == 1 && k == 1 && a == 7 && b == 2);   /* right click: B */
    g_mbuttons = 0; gs_ui_tick();
    /* a card, a tab, a key hint and the wheel */
    drain(h);
    rect_of(h, AT_HIT_CARD, 2, &x, &y); g_mx = x; g_my = y; gs_ui_tick(); drain(h); g_mx += 1.0f; gs_ui_tick(); CHECK(poll(h, &k, &a, &b) == 1 && k == 2 && a == 2 && b == 0);
    drain(h); g_mbuttons = 1; gs_ui_tick(); CHECK(poll(h, &k, &a, &b) == 1 && k == 2 && a == 2 && b == 1);
    drain(h); g_mbuttons = 2; gs_ui_tick(); CHECK(poll(h, &k, &a, &b) == 1 && k == 2 && a == 2 && b == 2);
    drain(h); g_mbuttons = 0; gs_ui_tick(); drain(h);
    g_mwheel = 1.0f; gs_ui_tick(); CHECK(poll(h, &k, &a, &b) == 1 && k == 5 && a == 1 && b == 2); g_mwheel = 0.0f;   /* over a card: the wheel names the card */
    drain(h);
    rect_of(h, AT_HIT_TAB, 1, &x, &y); g_mx = x; g_my = y; gs_ui_tick(); drain(h); g_mx += 1.0f; gs_ui_tick(); CHECK(poll(h, &k, &a, &b) == 1 && k == 6);   /* hovering a tab only says the pointer moved */
    g_mbuttons = 1; gs_ui_tick(); CHECK(poll(h, &k, &a, &b) == 1 && k == 3 && a == 1);
    drain(h); g_mbuttons = 0; gs_ui_tick(); drain(h);
    gw_Ui_SelKeys(h, "A:Pick,B:Back,X:Costume,S:Fight"); draw_once();
    rect_of(h, AT_HIT_KEY, 'A', &x, &y); g_mx = x; g_my = y; gs_ui_tick(); drain(h);
    g_mbuttons = 1; gs_ui_tick(); CHECK(poll(h, &k, &a, &b) == 1 && k == 4 && a == 'A');
    drain(h); g_mbuttons = 0; gs_ui_tick(); drain(h);
    /* the ring never grows past 32 and drops the newest; it is drained in order */
    { int i; g_mx = x; for (i = 0; i < 50; i++) { g_mwheel = 1.0f; gs_ui_tick(); } g_mwheel = 0.0f; CHECK(gs_sel.qn == 32); }
    gw_Ui_SelClose(h);
    /* keys and mouse of a closed screen: the tick reaches no select */
    gs_ui_tick(); CHECK(gs_sel.qn == 0);
}
static void poll_is_big_endian(void)
{
    int h, k, a, b;
    unsigned raw[3];
    reset_ui();
    h = open_css();
    gs_sel_q_push(GS_SELEV_TAB, 2, 0x01020304);
    CHECK(gw_Ui_SelPoll(h, (int *) &raw[0], (int *) &raw[1], (int *) &raw[2]) == 1);
    CHECK(raw[0] == bswap_u(3) && raw[1] == bswap_u(2) && raw[2] == bswap_u(0x01020304u));              /* the game's locals are big-endian: raw bytes are 00 00 00 03 ... */
    { const unsigned char *p = (const unsigned char *) &raw[0]; CHECK(p[0] == 0 && p[1] == 0 && p[2] == 0 && p[3] == 3); }
    CHECK(poll(h, &k, &a, &b) == 0);
    gw_Ui_SelClose(h);
}
static void fields_land_where_the_render_reads_them(void)
{
    int h;
    GsUiSlot *u;
    reset_ui();
    h = open_css(); u = slot_of(h);
    gw_Ui_SelBeginCells(h, 4);
    gw_Ui_SelCell(h, 2, "f2", "Peach's Castle", "PC", 9, AT_CELL_BANNED | AT_CELL_P1, '+');
    CHECK(strcmp(gs_sel.pool[2].id, "f2") == 0 && strcmp(gs_sel.pool[2].abbr, "PC") == 0 && gs_sel.pool[2].tex == 9 && gs_sel.pool[2].flags == (AT_CELL_BANNED | AT_CELL_P1) && gs_sel.pool[2].origin == '+' && gs_sel.pool[2].model == AT_NO_MODEL);
    gw_Ui_SelCell(h, 3, "f3", "n", "NN", -5, 0u, 'q'); CHECK(gs_sel.pool[3].tex == -1 && gs_sel.pool[3].origin == 0);   /* a negative texture is none; an unknown origin is none */
    gw_Ui_SelCell(h, 4, "f4", "n", "NN", 1, 0u, 0); CHECK(gs_sel.pool[4].id[0] == '\0');               /* past the count */
    gw_Ui_SelCard(h, 1, 2, "KIRBY", "Costume 2", "KI", 4, 7, 2, AT_CARD_READY);
    CHECK(u->sc.ports[1].kind == 2 && u->sc.ports[1].cpu_lv == 7 && u->sc.ports[1].team == 2 && u->sc.ports[1].ck_tex == 4 && strcmp(u->sc.ports[1].name, "KIRBY") == 0 && (u->sc.ports[1].flags & AT_CARD_READY));
    gw_Ui_SelCard(h, 4, 1, "x", "x", "xx", 0, 0, 0, 0u); gw_Ui_SelCard(h, -1, 1, "x", "x", "xx", 0, 0, 0, 0u);   /* a bad port is ignored */
    gw_Ui_SelCard(h, 0, 9, "x", "x", "xx", -1, 0, 0, 0u); CHECK(u->sc.ports[0].kind == 0);
    gw_Ui_SelCursor(h, 2, 1, 11, 3); CHECK(u->view.cursor[2].active == 1 && u->view.cursor[2].index == 11 && u->view.cursor[2].card == 3);
    gw_Ui_SelCursor(h, 2, 1, 4, 9); CHECK(u->view.cursor[2].card == -1);
    gw_Ui_SelCursor(h, 4, 1, 0, -1); gw_Ui_SelCursor(h, -1, 1, 0, -1);                                  /* bad ports: ignored */
    gw_Ui_SelExplain(h, "FIGHTER", "FOX", "Fast.", "FO", 12, "COSTUME", "2 / 4", "Retail");
    CHECK(u->view.ex.has == 1 && u->view.ex.media_tex == 12 && u->view.ex.stepper == 1 && strcmp(u->view.ex.stepper_text, "2 / 4") == 0 && u->view.ex.media_model == AT_NO_MODEL);
    gw_Ui_SelExplain(h, "", "", "", "", -1, "", "", ""); CHECK(u->view.ex.has == 0 && u->view.ex.stepper == 0 && u->view.ex.media_tex == -1);
    gw_Ui_SelKeys(h, "A:Pick,B:Back,X:Costume,S:Fight");
    CHECK(u->sc.n_keys == 4 && u->sc.keys[0].btn == 'A' && strcmp(u->sc.keys[0].label, "Pick") == 0 && u->sc.keys[3].btn == 'S' && strcmp(u->view.key_label[3], "Fight") == 0 && u->view.key_shown[3] == 1);
    gw_Ui_SelKeys(h, "A:One"); CHECK(u->sc.n_keys == 1 && u->view.key_shown[1] == 0 && u->view.key_label[1][0] == '\0');
    gw_Ui_SelKeys(h, "Q:Nope,A,:x,B:Fine,A:1,A:2,A:3,A:4,A:5,A:6"); CHECK(u->sc.n_keys == AT_MAX_KEYS && u->sc.keys[0].btn == 'B');   /* junk entries are skipped; at most six */
    gw_Ui_SelCounter(h, "3 / 29"); CHECK(strcmp(u->view.counter, "3 / 29") == 0);
    gw_Ui_SelNote(h, "Press B again to go back.", AT_NOTE_INFO); CHECK(strcmp(u->view.note.text, "Press B again to go back.") == 0 && u->view.note.until_ms > g_now);
    { double until = u->view.note.until_ms; g_now += 100.0; gw_Ui_SelNote(h, "Press B again to go back.", AT_NOTE_INFO); CHECK(u->view.note.until_ms == until); g_now -= 100.0; }   /* the same toast is not restarted */
    gw_Ui_SelNote(h, "Other", 77); CHECK(u->view.note.kind == AT_NOTE_INFO);
    gw_Ui_SelProgress(h, 1500); CHECK(u->view.progress == 1000); gw_Ui_SelProgress(h, -4); CHECK(u->view.progress == 0); gw_Ui_SelProgress(h, 640); CHECK(u->view.progress == 640);
    gw_Ui_SelClose(h);
    /* the explain, keys and counter of a closed handle are ignored too */
    gw_Ui_SelExplain(h, "a", "b", "c", "dd", 1, "e", "f", "g"); gw_Ui_SelKeys(h, "A:x"); gw_Ui_SelCounter(h, "x"); gw_Ui_SelNote(h, "x", 0); gw_Ui_SelProgress(h, 5);
    CHECK(gs_ui_stack.n == 0);
}
static void art_decode_goes_to_the_kit(void)
{
    unsigned char img[32] = { 0 }, tl[4] = { 0 };
    g_art_calls = 0; g_art_ret = 7;
    CHECK(gw_Ui_ArtDecode("i:5", 9, img, 32, 8, 4, 2, tl, 2) == 7 && g_art_calls == 1 && strcmp(g_art_key, "i:5") == 0);
    g_art_ret = -1; CHECK(gw_Ui_ArtDecode("i:6", 99, img, 32, 8, 4, 0, NULL, 0) == -1);
    g_art_ret = 5;
}
static void atlas_off_or_no_roles_keeps_the_legacy_drawing(void)
{
    reset_ui();
    gs_ui_atlas_env = 0;                                                    /* MELEE_ATLAS=0 */
    CHECK(gw_Ui_SelOpen(0, 0, "VERSUS", "MELEE", "FIGHTERS") == -1 && gs_ui_stack.n == 0);
    gs_ui_atlas_env = 1;
    CHECK(open_css() >= 0);
    gw_Ui_SceneExit(gs.scene_kind);
}
static void stack_full_refuses(void)
{
    int i;
    reset_ui();
    for (i = 0; i < GS_UI_SLOTS; i++) { gs_ui_slot[i].used = 1; gs_ui_slot[i].owner = GS_UI_ENGINE; snprintf(gs_ui_slot[i].sc.id, sizeof gs_ui_slot[i].sc.id, "x%d", i); }
    CHECK(open_css() == -1 && gs_sel.h == -1);                               /* no slot: the caller keeps the legacy drawing */
    reset_ui();
}
static void held_menu_and_hold_do_not_matter(void)
{
    int h;
    reset_ui();
    gs_ui_menu_held_by = 3;                                                  /* a script holding the native menu does not blank a select */
    h = open_css(); fill_css(h, 5); g_quads = 0; draw_once();
    CHECK(g_quads > 0);
    gw_Ui_SelClose(h);
    gs_ui_menu_held_by = -2;
}

/* ---- the models behind the scalar shims: what the game-side adapter calls, checked through the getters it reads back with ---- */
static void css_open_vs(int n, int online)
{
    int i;
    CHECK(gw_Ui_CssOpen(online ? AT_MT_LOBBY : 0, online, 0) == 1);
    gw_Ui_CssRosterBegin();
    for (i = 0; i < n; i++) { gw_Ui_CssRosterAdd(i, 1, i < 26 ? 1 : 2); gw_Ui_CssCostumes(i, 4); }
    gw_Ui_CssRosterEnd();
    gw_Ui_CssCols(8);
}
/* the game side asks BEFORE it routes a mode into the frontend scene: a refusal leaves the retail native screen */
static void can_open_before_routing(void)
{
    int i, h;
    reset_ui();
    CHECK(gw_Ui_SelCanOpen() == 1);
    h = open_css(); CHECK(h >= 0 && gw_Ui_SelCanOpen() == 0);                  /* one select at a time */
    gw_Ui_SelClose(h); CHECK(gw_Ui_SelCanOpen() == 1);
    gs_ui_atlas_env = 0; CHECK(gw_Ui_SelCanOpen() == 0); gs_ui_atlas_env = 1;  /* MELEE_ATLAS=0 */
    for (i = 0; i < GS_UI_SLOTS; i++) { gs_ui_slot[i].used = 1; gs_ui_slot[i].owner = GS_UI_ENGINE; }
    CHECK(gw_Ui_SelCanOpen() == 0);                                            /* no free slot */
    reset_ui();
    CHECK(gw_Ui_SelCanOpen() == 1);
    /* the answer is the open's answer: whenever CanOpen says yes an open succeeds */
    h = open_css(); CHECK(h >= 0); gw_Ui_SelClose(h);
}
/* Atlas fix D1: the frontend opens the select in the scene's INIT, before Script_SceneBegin runs gw_Ui_SceneExit(prev) for the scene that is ending. gs.scene_kind still
 * names that scene (the previous scene can be of the SAME kind: the frontend after the frontend), so a screen stamped with it closed in the frame it opened. */
static void a_select_opened_in_a_scene_init_survives_the_ending_scene(void)
{
    int h, n;
    reset_ui();
    gs.scene_kind = 46;                                                        /* the menus' scene is ending; the select's scene has the same kind */
    gw_Ui_SelNextScene(1); h = open_css(); gw_Ui_SelNextScene(0);
    CHECK(h >= 0 && gs_sel_get(h) != NULL);
    gw_Ui_SceneExit(gs.scene_kind);                                            /* Script_SceneBegin: the ending scene's screens go */
    CHECK(gs_sel_get(h) != NULL && gs_ui_stack.n == 1);                        /* ... not the screen the new scene opened */
    gs.scene_kind = 46; gw_Ui_SceneBegin(46);                                  /* the scene that begins adopts it */
    gw_Ui_SceneExit(46);                                                       /* and its own end closes it, as for every native screen */
    CHECK(gs_sel_get(h) == NULL && gs_ui_stack.n == 0);
    /* an open inside a running scene (the CSS to the SSS) is stamped with the scene, as before */
    reset_ui();
    h = open_css(); n = (int) (gs_sel_get(h) - gs_ui_slot);
    CHECK(gs_ui_slot[n].scene == 7);
    gw_Ui_SceneExit(7); CHECK(gs_sel_get(h) == NULL);
    /* the bracket ends: a later open is not stamped for a scene that is not coming */
    reset_ui();
    gw_Ui_SelNextScene(1); gw_Ui_SelNextScene(0);
    h = open_css(); n = (int) (gs_sel_get(h) - gs_ui_slot);
    CHECK(gs_ui_slot[n].scene == 7);
    gw_Ui_SelClose(h);
}
/* Atlas fix D3: the settings screen is native too (one at a time), and it is closed WHEN ITS SCENE EXITS so the next scene's select can open (fss_exit calls Ui_SetClose). */
static void a_closed_settings_door_lets_the_select_open(void)
{
    int sh;
    reset_ui();
    sh = gw_Ui_SetOpen(0, 0, "MAIN MENU", "VERSUS", "RULES");
    CHECK(sh >= 0 && gw_Ui_SelCanOpen() == 0 && open_css() < 0);               /* the legacy kit CSS would take over here */
    gw_Ui_SetClose(sh);
    CHECK(gw_Ui_SelCanOpen() == 1 && open_css() >= 0);
}
static void art_drop_reaches_the_kit(void)
{
    int before = g_hsd_drops;
    gw_Ui_ArtDrop();
    CHECK(g_hsd_drops == before + 1);
}
static void known_match_types(void)
{
    int mt;
    for (mt = 0; mt <= 0x17; mt++) CHECK(gw_Ui_CssKnown(mt) == 1);
    CHECK(gw_Ui_CssKnown(AT_MT_LOBBY) == 1 && gw_Ui_CssKnown(0x18) == 0 && gw_Ui_CssKnown(-1) == 0 && gw_Ui_CssKnown(0xFF) == 0);
}
static void css_through_the_shims(void)
{
    char buf[96];
    unsigned e;
    CHECK(gw_Ui_CssOpen(0x18, 0, 0) == 0 && gw_Ui_CssOpen(-1, 0, 0) == 0);          /* an unknown match type has no profile: the retail screen stays */
    css_open_vs(29, 0);
    CHECK(gw_Ui_CssGet(GS_CSSG_N_SLOTS, 0) == 30 && gw_Ui_CssGet(GS_CSSG_N_VIS, 0) == 30 && gw_Ui_CssGet(GS_CSSG_KIND, 0) == AT_CSS_HMN && gw_Ui_CssGet(GS_CSSG_KIND, 1) == AT_CSS_OFF);
    CHECK(gw_Ui_CssGet(GS_CSSG_MAX_HUMANS, 0) == 4 && gw_Ui_CssGet(GS_CSSG_CPU_CARDS, 0) == 1 && gw_Ui_CssGet(GS_CSSG_MIN_TO_START, 0) == 2 && gw_Ui_CssGet(GS_CSSG_HAS_SSS, 0) == 1);
    CHECK(gw_Ui_CssGet(GS_CSSG_TAB_OFFERED, 2) == 1 && gw_Ui_CssGet(GS_CSSG_TAB_COUNT, 2) == 3 && gw_Ui_CssGet(GS_CSSG_SLOT_CK, 29) == AT_CK_RANDOM && gw_Ui_CssGet(GS_CSSG_SLOT_TAB, 28) == 2);
    CHECK(gw_Ui_CssBlocker(buf, sizeof buf) == 1 && strcmp(buf, "P1: pick a fighter.") == 0);
    gw_Ui_CssSetCursor(0, 4);
    e = gw_Ui_CssStep(0, AT_CI_A, 0, 10); CHECK((e & AT_CE_FORWARD) && gw_Ui_CssGet(GS_CSSG_CK, 0) == 4 && gw_Ui_CssGet(GS_CSSG_CUR, 0) == 4);
    e = gw_Ui_CssStep(0, AT_CI_X, 0, 11); CHECK((e & AT_CE_MOVE) && gw_Ui_CssGet(GS_CSSG_COSTUME, 0) == 1);               /* costumes come from the pushed counts */
    gw_Ui_CssCostumes(4, 1); e = gw_Ui_CssStep(0, AT_CI_X, 0, 12); CHECK(gw_Ui_CssGet(GS_CSSG_COSTUME, 0) == 0);          /* a one-costume fighter: always 0 */
    CHECK(gw_Ui_CssBlocker(buf, sizeof buf) == 1 && strcmp(buf, "Two fighters needed: pick one, or add a CPU below.") == 0);
    gw_Ui_CssPort(1, AT_CSS_CPU, AT_CK_RANDOM, 0, 7, 1);
    CHECK(gw_Ui_CssBlocker(buf, sizeof buf) == 0 && buf[0] == 0 && gw_Ui_CssGet(GS_CSSG_COUNT_IN, 0) == 2 && gw_Ui_CssGet(GS_CSSG_CPU_LV, 1) == 7 && gw_Ui_CssGet(GS_CSSG_TEAM, 1) == 1);
    e = gw_Ui_CssStep(0, AT_CI_START, 0, 13); CHECK((e & AT_CE_FINISH_GO) && gw_Ui_CssGet(GS_CSSG_DONE, 0) == 1);
    { char tiny[6]; gw_Ui_CssOpen(0, 0, 0); gw_Ui_CssRosterBegin(); gw_Ui_CssRosterEnd(); CHECK(gw_Ui_CssBlocker(tiny, sizeof tiny) == 1 && strlen(tiny) == 5); }   /* a small buffer is cut and terminated */
    CHECK(gw_Ui_CssBlocker(NULL, 0) == 1);                                    /* no buffer: still answers, writes nothing */
    /* a bad port, a bad slot and a bad field answer safely */
    css_open_vs(5, 0);
    gw_Ui_CssPort(9, 1, 0, 0, 9, 0); gw_Ui_CssPort(-1, 1, 0, 0, 9, 0); CHECK(gw_Ui_CssGet(GS_CSSG_KIND, 9) == 0 && gw_Ui_CssGet(GS_CSSG_VIS_SLOT, 99) == -1 && gw_Ui_CssGet(GS_CSSG_SLOT_CK, 99) == AT_CK_NONE && gw_Ui_CssGet(999, 0) == 0);
    gw_Ui_CssPort(2, 77, -9, -4, 99, 8); CHECK(gw_Ui_CssGet(GS_CSSG_KIND, 2) == 0 && gw_Ui_CssGet(GS_CSSG_CK, 2) == AT_CK_NONE && gw_Ui_CssGet(GS_CSSG_COSTUME, 2) == 0 && gw_Ui_CssGet(GS_CSSG_CPU_LV, 2) == 9 && gw_Ui_CssGet(GS_CSSG_TEAM, 2) == 0);
    gw_Ui_CssSetCursor(0, 9999); CHECK(gw_Ui_CssGet(GS_CSSG_CUR, 0) == 0);
    /* more than the roster holds: cut at 129 slots, never overrun */
    { int i; gw_Ui_CssOpen(0, 0, 0); gw_Ui_CssRosterBegin(); for (i = 0; i < 300; i++) gw_Ui_CssRosterAdd(i, 1, 1); gw_Ui_CssRosterEnd(); CHECK(gw_Ui_CssGet(GS_CSSG_N_SLOTS, 0) == AT_CSS_MAX_SLOTS); }
}
static void css_zelda_sheik_and_online_through_the_shims(void)
{
    unsigned e;
    css_open_vs(20, 0);
    gw_Ui_CssPair(19, 20);                                                  /* Zelda is 19, Sheik 20 and has no tile of her own */
    gw_Ui_CssCostumes(20, 4);
    gw_Ui_CssSetCursor(0, 19);
    gw_Ui_CssStep(0, AT_CI_A, 0, 1); gw_Ui_CssStep(0, AT_CI_A, 0, 2); CHECK(gw_Ui_CssGet(GS_CSSG_CK, 0) == 20);
    CHECK(gw_Ui_CssGet(GS_CSSG_VIS_OF_CK, 20) == 19);
    /* online (the lobby): one card, the opponent's missing Sheik refuses the swap */
    css_open_vs(20, 1);
    gw_Ui_CssPair(19, 20); gw_Ui_CssCostumes(20, 4); gw_Ui_CssSheikOk(0);
    CHECK(gw_Ui_CssGet(GS_CSSG_ONLINE, 0) == 1 && gw_Ui_CssGet(GS_CSSG_MIN_TO_START, 0) == 1);
    gw_Ui_CssSetCursor(0, 19); gw_Ui_CssStep(0, AT_CI_A, 0, 1); e = gw_Ui_CssStep(0, AT_CI_A, 0, 2);
    CHECK((e & AT_CE_TOAST) && gw_Ui_CssGet(GS_CSSG_CK, 0) == 19);
    { char t[80]; CHECK(gw_Ui_CssToast(t, sizeof t) > 0 && strstr(t, "Sheik") != NULL); }
    gw_Ui_CssSheikOk(1); gw_Ui_CssStep(0, AT_CI_A, 0, 3); CHECK(gw_Ui_CssGet(GS_CSSG_CK, 0) == 20);
    gw_Ui_CssSetPortCk(0, 19, 2); CHECK(gw_Ui_CssGet(GS_CSSG_CK, 0) == 19 && gw_Ui_CssGet(GS_CSSG_COSTUME, 0) == 2);      /* the adapter swap when A is held on START */
    /* the one-player profiles and Training through the setters */
    gw_Ui_CssOpen(0xB, 0, 0); gw_Ui_CssRosterBegin(); gw_Ui_CssRosterAdd(0, 1, 1); gw_Ui_CssRosterAdd(1, 1, 1); gw_Ui_CssRosterEnd(); gw_Ui_CssEntering(3);
    CHECK(gw_Ui_CssGet(GS_CSSG_KIND, 3) == AT_CSS_HMN && gw_Ui_CssGet(GS_CSSG_KIND, 0) == AT_CSS_OFF && gw_Ui_CssGet(GS_CSSG_ENTERING, 0) == 1 && gw_Ui_CssGet(GS_CSSG_MAX_HUMANS, 0) == 1);
    gw_Ui_CssOpen(0x17, 0, 0); gw_Ui_CssRosterBegin(); gw_Ui_CssRosterAdd(0, 1, 1); gw_Ui_CssRosterEnd(); gw_Ui_CssTraining(2);
    CHECK(gw_Ui_CssGet(GS_CSSG_TRAIN_H, 0) == 2 && gw_Ui_CssGet(GS_CSSG_TRAIN_C, 0) == 0 && gw_Ui_CssGet(GS_CSSG_KIND, 0) == AT_CSS_CPU && gw_Ui_CssGet(GS_CSSG_DUMMY, 0) == 1);
}
static void css_mouse_through_the_shims(void)
{
    unsigned b;
    css_open_vs(29, 0);
    CHECK(gw_Ui_CssGet(GS_CSSG_MOUSE_PORT, 0) == 0);
    b = gw_Ui_CssMouse(0, 7, -1, 1, 0, 0, 0); CHECK(gw_Ui_CssGet(GS_CSSG_CUR, 0) == 7 && b == 0);
    b = gw_Ui_CssMouse(0, 9, -1, 0, 1, 0, 0); CHECK((b & AT_CI_A) && gw_Ui_CssGet(GS_CSSG_CUR, 0) == 9);
    b = gw_Ui_CssMouse(0, -1, 1, 1, 0, 0, 0); CHECK(gw_Ui_CssGet(GS_CSSG_CARD, 0) == 1 && b == 0);
}
static void sss_through_the_shims(void)
{
    int i;
    unsigned e;
    char t[80];
    gw_Ui_SssOpen(0); gw_Ui_SssStagesBegin();
    for (i = 0; i < 12; i++) gw_Ui_SssStageAdd(20 + i, 2, i, i != 3, i >= 10 ? 2 : 1);
    gw_Ui_SssStageAdd(-1, 3, 12, 1, 0); gw_Ui_SssStageAdd(-1, 3, 13, 1, 0); gw_Ui_SssStageAdd(0, 1, 14, 1, 1);
    gw_Ui_SssStagesEnd(); gw_Ui_SssCols(8);
    CHECK(gw_Ui_SssGet(GS_SSSG_N, 0) == 13 && gw_Ui_SssGet(GS_SSSG_TYPE, 12) == 3 && gw_Ui_SssGet(GS_SSSG_EXT, 12) == -1 && gw_Ui_SssGet(GS_SSSG_N_VIS, 0) == 13);
    CHECK(gw_Ui_SssGet(GS_SSSG_TAB_OFFERED, 2) == 1 && gw_Ui_SssGet(GS_SSSG_TAB_COUNT, 2) == 2 && gw_Ui_SssGet(GS_SSSG_TAB_FOR_EXT, 287) == 1 && gw_Ui_SssGet(GS_SSSG_TAB_FOR_EXT, 288) == 2);
    gw_Ui_SssCursorExt(23); CHECK(gw_Ui_SssGet(GS_SSSG_CUR, 0) == 3);
    e = gw_Ui_SssStep(AT_CI_A, 0, 5); CHECK((e & AT_CE_BACK) && (e & AT_CE_TOAST) && gw_Ui_SssGet(GS_SSSG_GO, 0) == 0 && gw_Ui_SssToast(t, sizeof t) > 0 && strcmp(t, "That stage is locked.") == 0);
    gw_Ui_SssStep(0, AT_CI_RIGHT, 0); e = gw_Ui_SssStep(AT_CI_A, 0, 5);
    CHECK((e & AT_CE_FINISH_GO) && gw_Ui_SssGet(GS_SSSG_GO, 0) == 1 && gw_Ui_SssGet(GS_SSSG_PICK_EXT, 0) == 24 && gw_Ui_SssGet(GS_SSSG_DONE, 0) == 1);
    gw_Ui_SssOpen(0); gw_Ui_SssStagesBegin(); gw_Ui_SssStageAdd(31, 2, 0, 1, 1); gw_Ui_SssStageAdd(32, 2, 1, 0, 1); gw_Ui_SssStagesEnd();
    CHECK(gw_Ui_SssRandomPool() == 1 && gw_Ui_SssPickRandom(0) == 31 && gw_Ui_SssPickRandom(7) == 31);
    e = gw_Ui_SssMouse(0, 1, 1, 0); CHECK(e & AT_CI_A);
    gw_Ui_SssStageAdd(1, 2, 0, 1, 1);                                        /* an add after End waits for the next End */
    CHECK(gw_Ui_SssGet(GS_SSSG_N, 0) == 3);
    for (i = 0; i < 400; i++) gw_Ui_SssStageAdd(1000 + i, 2, i, 1, 1);        /* a list longer than the model keeps: cut, Random last */
    gw_Ui_SssStagesEnd(); CHECK(gw_Ui_SssGet(GS_SSSG_N, 0) == AT_SSS_MAX && gw_Ui_SssGet(GS_SSSG_TYPE, AT_SSS_MAX - 1) == 3);
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
    filled_and_drawn(); closed_on_every_exit(); owner_native_vs_mod(); mouse_port(); poll_is_big_endian(); fields_land_where_the_render_reads_them();
    art_decode_goes_to_the_kit(); atlas_off_or_no_roles_keeps_the_legacy_drawing(); stack_full_refuses(); held_menu_and_hold_do_not_matter();
    can_open_before_routing(); a_select_opened_in_a_scene_init_survives_the_ending_scene(); a_closed_settings_door_lets_the_select_open(); art_drop_reaches_the_kit(); known_match_types(); css_through_the_shims(); css_zelda_sheik_and_online_through_the_shims(); css_mouse_through_the_shims(); sss_through_the_shims();
    lua_close(L);
    ATLAS_DONE("atlas select adapter");
}
