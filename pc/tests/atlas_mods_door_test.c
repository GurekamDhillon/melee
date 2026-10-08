/* Host-side test of the MODS screen (gw_script_ui_mods.inc, a kind of the native settings screen) with the real accessors (gw_ui_mods_native.c) over a fake mods folder, against
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
/* No fighter registry in this mods-door fixture (Geno slice 6). */
int gw_Geno_DefineArtTex(int ck, int what, int costume) { (void) ck; (void) what; (void) costume; return -1; }
static int gs_may_run(int i) { (void) i; return g_may_run; }
static int g_online, g_set_calls, g_save_calls, g_save_result;
int gw_RB_Enabled(void) { return g_online; }
static int gs_ui_port_read(int port, char *name, int cap, int *percent, int *stocks, int *cpu)   /* the fighter readbacks gd.player uses */
{
    (void) port; (void) cap; name[0] = 0; *percent = *stocks = *cpu = 0;
    return 0;
}
int gw_Netplay_Enabled(void) { return 0; }
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


/* ---- a fake mods folder behind gw_Mods_*: the REAL accessors (gw_ui_mods_native.c) run over it, with the real resolver's cascade rules in miniature ---- */
#include "gw_mods.h"
typedef struct { const char *id, *name, *kind, *req, *con, *status_text; int status, enabled, active, nmenus; } FMod;
static FMod FM[5];
static void fixture(void)
{
    static const FMod base[5] = {
        { "ace-base", "ACE Base", "base", "", "", "", GW_MOD_ACTIVE, 1, 1, 0 },
        { "ace-wolf", "ACE Wolf", "fighter", "ace-base", "", "", GW_MOD_ACTIVE, 1, 1, 0 },
        { "envoy", "Supertime Envoy", "script", "", "", "", GW_MOD_ACTIVE, 1, 1, 2 },
        { "envoy-drives-sa2", "Envoy Drives SA2", "misc", "", "envoy-drives", "conflicts with envoy-drives", GW_MOD_CONFLICT, 1, 0, 0 },
        { "orphan", "Orphan Fighter", "fighter", "ghost-base", "", "needs ghost-base", GW_MOD_MISSING_DEP, 1, 0, 0 },
    };
    memcpy(FM, base, sizeof FM);
}
static int fm_has(const char *list, const char *id)
{
    size_t n = strlen(id);
    const char *p = list;
    while (p != NULL && *p) {
        const char *e = strchr(p, ',');
        size_t len = e ? (size_t) (e - p) : strlen(p);
        if (len == n && strncmp(p, id, n) == 0) return 1;
        p = e ? e + 1 : NULL;
    }
    return 0;
}
int gw_Mods_Count(void) { return 5; }
const char *gw_Mods_Id(int i) { return i >= 0 && i < 5 ? FM[i].id : ""; }
const char *gw_Mods_Name(int i) { return i >= 0 && i < 5 ? FM[i].name : ""; }
const char *gw_Mods_Version(int i) { (void) i; return "1.0"; }
const char *gw_Mods_Kind(int i) { return i >= 0 && i < 5 ? FM[i].kind : ""; }
const char *gw_Mods_Pack(int i) { (void) i; return ""; }
const char *gw_Mods_Description(int i) { (void) i; return "A mod."; }
const char *gw_Mods_Requires(int i) { return i >= 0 && i < 5 ? FM[i].req : ""; }
const char *gw_Mods_Conflicts(int i) { return i >= 0 && i < 5 ? FM[i].con : ""; }
const char *gw_Mods_StatusText(int i) { return i >= 0 && i < 5 ? FM[i].status_text : ""; }
int gw_Mods_Status(int i) { return i >= 0 && i < 5 ? FM[i].status : GW_MOD_OFF; }
int gw_Mods_IsEnabled(int i) { return i >= 0 && i < 5 && FM[i].enabled; }
int gw_Mods_IsActive(int i) { return i >= 0 && i < 5 && FM[i].active; }
int gw_Mods_SetEnabled(int i, int on)
{
    int j, changed = 0;
    g_set_calls++;
    if (i < 0 || i >= 5 || FM[i].enabled == on) return 0;
    FM[i].enabled = on; changed++;
    for (j = 0; j < 5; j++) {
        if (j == i) continue;
        if (on && fm_has(FM[i].req, FM[j].id) && !FM[j].enabled) { FM[j].enabled = 1; changed++; }
        if (on && FM[j].enabled && (fm_has(FM[i].con, FM[j].id) || fm_has(FM[j].con, FM[i].id))) { FM[j].enabled = 0; changed++; }
        if (!on && FM[j].enabled && fm_has(FM[j].req, FM[i].id)) { FM[j].enabled = 0; changed++; }
    }
    return changed;
}
int gw_Mods_Save(void) { g_save_calls++; return g_save_result; }
int gw_Mods_RestartNeeded(void) { int i; for (i = 0; i < 5; i++) if (FM[i].enabled != FM[i].active) return 1; return 0; }
/* envoy (index 2) adds a Solo entry and has its own settings entry under mods.self; only an active mod has entries */
int gw_Mods_MenuCount(int i) { return i >= 0 && i < 5 && FM[i].active ? FM[i].nmenus : 0; }
const char *gw_Mods_MenuField(int i, int k, const char *f)
{
    static const char *const e0[][2] = { { "id", "envoy" }, { "parent", "solo" }, { "label", "Envoy" }, { "action", "script" }, { "online", "false" }, { "blurb", "" }, { "icon", "" }, { "after", "" }, { "opens", "" } };
    static const char *const e1[][2] = { { "id", "envoy.settings" }, { "parent", "mods.self" }, { "label", "Settings" }, { "action", "opens" }, { "opens", "envoy.settings" }, { "online", "false" }, { "blurb", "" }, { "icon", "" }, { "after", "" } };
    const char *const (*e)[2] = k == 0 ? e0 : e1;
    int j;
    if (!gw_Mods_MenuCount(i) || k < 0 || k > 1) return "";
    for (j = 0; j < 9; j++) if (strcmp(e[j][0], f) == 0) return e[j][1];
    return "";
}

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
    return buf;
}
static int t_lua(const char *code) { return strncmp(lua(code), "ERR", 3) != 0; }
static void reset_ui(void)
{
    int i;
    for (i = 0; i < GS_UI_SLOTS; i++) { memset(&gs_ui_slot[i], 0, sizeof gs_ui_slot[i]); gs_ui_slot[i].dialog_fn = -1; }
    memset(&gs_ui_stack, 0, sizeof gs_ui_stack);
    gs_ui_qn = 0; g_pad = 0; gs.cur = 0; gs.scene_kind = 7;
    gs_sel.h = -1; gs_sel.qn = 0;
    g_mx = g_my = -1000.0f; g_mbuttons = 0; g_mwheel = 0.0f;
}
static GsUiSlot *slot_of(int h) { return &gs_ui_slot[h & 0xFF]; }

/* ---- the fake mods folder behind gw_Mods_*: the real accessors (gw_ui_mods_native.c) and the real resolver's cascade rules, in miniature ---- */
static int unbswap(const void *p) { const unsigned char *b = (const unsigned char *) p; return (int) (((unsigned) b[0] << 24) | ((unsigned) b[1] << 16) | ((unsigned) b[2] << 8) | b[3]); }

static void reset_door(void)
{
    reset_ui();
    gs_set.h = -1; gs_set.qn = 0; gs_set.frozen = 0; gs_set.answer = 0; gs_set.pending = 0; gs_set.fid[0] = '\0'; gs_set.fidx = -1;
    memset(gs.key_now, 0, sizeof gs.key_now);
    memset(&gs_mods, 0, sizeof gs_mods);
    memset(&gs_ui_reg, 0, sizeof gs_ui_reg); gs_ui_reg_booted = 0;
    g_online = 0; g_set_calls = 0; g_save_calls = 0; g_save_result = 0; g_now = 1000.0;
    fixture();
}
static int open_mods(void) { return gw_Ui_SetOpen(GS_SET_MODS, 0, "MAIN MENU", "", "MODS"); }
static void tick(void) { g_now += 16.0; gs_ui_tick(); }
static void intent(int h, int type, int a) { tick(); gw_Ui_SetIntent(h, type, a); }
static int poll_one(int h, int *slot, int *arg)
{
    int t = gw_Ui_SetPoll(h, slot, arg);
    if (t) { *slot = unbswap(slot); *arg = unbswap(arg); }
    return t;
}
static const char *focus_label(int h) { GsUiSlot *u = slot_of(h); return u->view.focus.index >= 0 && u->view.focus.index < u->sc.n_items ? u->sc.items[u->view.focus.index].label : "(none)"; }

static void opens_and_draws(void)
{
    int h, i;
    GsUiSlot *u;
    reset_door();
    g_pad = 0;
    h = open_mods();
    CHECK(h >= 0);
    u = slot_of(h);
    CHECK(u->used && u->owner == GS_UI_ENGINE && u->native_set && u->native_mods && at_stack_top(&gs_ui_stack) == (int) (u - gs_ui_slot));
    CHECK(strcmp(u->sc.id, "mods.list") == 0 && u->sc.n_items == 5 && u->sc.n_tabs == 2 && u->sc.tabs[0].count == 5 && u->sc.tabs[1].count == 2);
    /* the real accessors, row by row: names, the mounted bar, the next-boot toggle, the sub line from the mod's menus entry */
    CHECK(strcmp(u->sc.items[0].label, "ACE Base") == 0 && u->sc.items[0].on == 1 && (u->sc.items[0].flags & AT_CELL_SELECTED));
    CHECK(strstr(u->sc.items[2].sub, "adds Solo > Envoy") != NULL);                  /* from mod.json "menus"; its mods.self entry is not an addition */
    CHECK(strstr(u->sc.items[3].sub, "CONFLICT") != NULL && strstr(u->sc.items[4].sub, "needs ghost-base") != NULL);
    for (i = 0; i < 2; i++) { gs_ui_draw(); }
    CHECK(g_quads > 0 && u->hits.n > 5);
    gw_Ui_SetClose(h);
    CHECK(at_stack_top(&gs_ui_stack) < 0 && !gs_ui_slot[h & 0xFF].used);
    gw_Ui_SetClose(h);                                                               /* closing twice is harmless */
    gs_ui_tick(); gs_ui_draw();
}

static void pad_intents(void)
{
    int h, slot = 7, arg = 7;
    GsUiSlot *u;
    reset_door();
    h = open_mods(); u = slot_of(h);
    /* the A that opened the screen is the game's menu input of this very frame: it must not toggle the first mod */
    gw_Ui_SetIntent(h, AT_EV_ACCEPT, 0);
    CHECK(FM[0].enabled == 1 && g_save_calls == 0);
    intent(h, AT_EV_ACCEPT, 0);                                                      /* a press a frame later does: ACE Base off, ACE Wolf with it */
    CHECK(FM[0].enabled == 0 && FM[1].enabled == 0 && g_save_calls == 1);
    CHECK(strstr(u->view.note.text, "ACE Wolf") != NULL && u->view.note.kind == AT_NOTE_OK);   /* it says what else moved */
    CHECK(u->sc.items[0].on == 0 && (u->sc.items[0].flags & AT_CELL_SELECTED) && strstr(u->sc.items[0].sub, "RESTART") != NULL);
    intent(h, AT_EV_MOVE, AT_DIR_DOWN); CHECK(strcmp(focus_label(h), "ACE Wolf") == 0 && strcmp(u->view.counter, "2 / 5") == 0);
    intent(h, AT_EV_MOVE, AT_DIR_RIGHT);                                             /* right flips a toggle too: ACE Wolf on, ACE Base with it */
    CHECK(FM[1].enabled == 1 && FM[0].enabled == 1);
    intent(h, AT_EV_PAGE, 1); CHECK(u->view.tab == 1 && u->sc.n_items == 2 && strcmp(u->sc.items[0].label, "Envoy Drives SA2") == 0);
    intent(h, AT_EV_ALT, 'X');                                                       /* X resolves the conflict row */
    CHECK(FM[3].enabled == 0);
    intent(h, AT_EV_PAGE, -1); CHECK(u->view.tab == 0);
    /* nothing the model does reaches the game's event ring but B */
    CHECK(poll_one(h, &slot, &arg) == 0);
    intent(h, AT_EV_BACK, 0);
    CHECK(poll_one(h, &slot, &arg) == GS_SETEV_BACK && slot == -1 && arg == 0 && poll_one(h, &slot, &arg) == 0);
    gw_Ui_SetClose(h);
}

/* the poll writes the game's locals: big-endian, so the raw bytes are what the game's byte-swapped load reads */
static void poll_raw_bytes(void)
{
    int h, s = 0x11223344, a = 0x55667788;
    unsigned char *bs = (unsigned char *) &s, *ba = (unsigned char *) &a;
    reset_door();
    h = open_mods();
    intent(h, AT_EV_BACK, 0);
    CHECK(gw_Ui_SetPoll(h, &s, &a) == GS_SETEV_BACK);
    CHECK(bs[0] == 0xFF && bs[1] == 0xFF && bs[2] == 0xFF && bs[3] == 0xFF);        /* -1: no row */
    CHECK(ba[0] == 0 && ba[1] == 0 && ba[2] == 0 && ba[3] == 0);
    gw_Ui_SetClose(h);
}

static void keys_and_mouse(void)
{
    int h, i, k;
    GsUiSlot *u;
    reset_door();
    h = open_mods(); u = slot_of(h);
    tick(); gs_ui_draw();
    gs.key_now[VK_DOWN] = 1; tick(); gs.key_now[VK_DOWN] = 0; tick();
    CHECK(strcmp(focus_label(h), "ACE Wolf") == 0);
    gs.key_now[VK_RETURN] = 1; tick(); gs.key_now[VK_RETURN] = 0; tick();
    CHECK(FM[1].enabled == 0);
    gs.key_now[VK_TAB] = 1; tick(); gs.key_now[VK_TAB] = 0; tick();                    /* Tab: the other tab */
    CHECK(u->view.tab == 1);
    /* a click on the INSTALLED tab goes back; a hover focuses, a click toggles */
    gs_ui_draw();
    for (i = 0; i < u->hits.n; i++) if (u->hits.h[i].kind == AT_HIT_TAB && u->hits.h[i].a == 0) { g_mx = u->hits.h[i].r.x + 4.0f; g_my = u->hits.h[i].r.y + 4.0f; }
    g_mbuttons = 0; tick(); g_mbuttons = 1; tick(); g_mbuttons = 0; tick(); gs_ui_draw();
    CHECK(u->view.tab == 0);
    for (i = 0; i < u->hits.n; i++) if (u->hits.h[i].kind == AT_HIT_CELL && u->hits.h[i].b == 2) { g_mx = u->hits.h[i].r.x + 20.0f; g_my = u->hits.h[i].r.y + 8.0f; }
    tick(); tick();
    CHECK(strcmp(focus_label(h), "Supertime Envoy") == 0);                           /* a hover on a row focuses it */
    g_mbuttons = 1; tick(); g_mbuttons = 0; tick();
    CHECK(FM[2].enabled == 0);                                                       /* a click is A */
    g_mwheel = 1.0f; tick(); g_mwheel = 0.0f; tick();
    CHECK(u->view.focus.index >= 0);
    k = u->view.focus.index;
    g_mx = g_my = -1000.0f; tick(); tick();
    CHECK(u->view.focus.index == k);                                                 /* a pointer off the picture changes nothing */
    gw_Ui_SetClose(h);
}

static void detail_and_the_mods_own_screen(void)
{
    int h, i, slot = 0, arg = 0;
    GsUiSlot *u;
    reset_door();
    gs.cur = 1;                                                                      /* the mod "envoy" registers its settings screen as itself */
    CHECK(t_lua("gd.ui.screen{id='envoy.settings', title='ENVOY', primary={kind='list', items={{id='a', label='A'}}}, on={back=function() gd.ui.close() end}}; return true"));
    gs.cur = 0;
    h = open_mods(); u = slot_of(h);
    intent(h, AT_EV_MOVE, AT_DIR_DOWN); intent(h, AT_EV_MOVE, AT_DIR_DOWN);          /* the envoy row */
    intent(h, AT_EV_ALT, 'Y');
    CHECK(gs_mods.mode == 1 && strcmp(u->sc.id, "mods.detail") == 0 && strcmp(u->sc.title, "SUPERTIME ENVOY") == 0);
    for (i = 0; i < u->sc.n_items; i++) if (strcmp(u->sc.items[i].id, "settings") == 0) break;
    CHECK(i < u->sc.n_items);                                                        /* the mod's own entry is a row (registered, visible, its script running) */
    while (u->view.focus.index != i) intent(h, AT_EV_MOVE, AT_DIR_DOWN);
    intent(h, AT_EV_ACCEPT, 0);
    CHECK(at_stack_top(&gs_ui_stack) != (int) (u - gs_ui_slot) && gs_ui_slot[at_stack_top(&gs_ui_stack)].owner == 1);   /* the mod's screen is over this one */
    CHECK(strcmp(gs_ui_slot[at_stack_top(&gs_ui_stack)].sc.id, "envoy.settings") == 0);
    gw_Ui_SetIntent(h, AT_EV_BACK, 0);                                               /* covered: this screen takes no input */
    CHECK(gs_mods.mode == 1 && poll_one(h, &slot, &arg) == 0);
    gs.cur = 1; t_lua("gd.ui.close()"); gs.cur = 0;                                  /* the mod's screen closes: the detail is the top again, as it was */
    CHECK(at_stack_top(&gs_ui_stack) == (int) (u - gs_ui_slot) && gs_mods.mode == 1);
    intent(h, AT_EV_BACK, 0); CHECK(gs_mods.mode == 0 && poll_one(h, &slot, &arg) == 0);   /* B in the detail goes to the list, not out of the screen */
    intent(h, AT_EV_BACK, 0); CHECK(poll_one(h, &slot, &arg) == GS_SETEV_BACK);
    /* a mod with no settings entry has no such row */
    intent(h, AT_EV_MOVE, AT_DIR_UP); intent(h, AT_EV_MOVE, AT_DIR_UP);
    intent(h, AT_EV_ALT, 'Y');
    CHECK(gs_mods.mode == 1);
    for (i = 0; i < u->sc.n_items; i++) CHECK(strcmp(u->sc.items[i].id, "settings") != 0);
    gw_Ui_SetClose(h);
}

static void another_native_screen_still_refuses_to_be_covered(void)
{
    int h;
    reset_door();
    gs.cur = 1;
    CHECK(t_lua("gd.ui.screen{id='envoy.x', primary={kind='list', items={{id='a', label='A'}}}}; return true"));
    gs.cur = 0;
    h = gw_Ui_SetOpen(0, 0, "MAIN MENU", "", "SETTINGS");                            /* a plain settings screen: no mod screen over it, as before */
    CHECK(h >= 0 && slot_of(h)->native_mods == 0);
    gs.cur = 1; CHECK(strncmp(lua("return gd.ui.open('envoy.x')"), "ERR", 3) != 0); gs.cur = 0;
    CHECK(at_stack_top(&gs_ui_stack) == (int) (slot_of(h) - gs_ui_slot));
    gw_Ui_SetClose(h);
    /* and only one native screen at a time: the MODS screen cannot open over it either */
    h = gw_Ui_SetOpen(0, 0, "MAIN MENU", "", "SETTINGS");
    CHECK(open_mods() == -1);
    gw_Ui_SetClose(h);
}

static void online_changes_nothing(void)
{
    int h;
    GsUiSlot *u;
    reset_door();
    g_online = 1;
    h = open_mods(); u = slot_of(h);
    CHECK((u->sc.items[0].iflags & AT_ITEM_RO) && strcmp(u->view.note.text, "Locked while online") == 0);
    intent(h, AT_EV_ACCEPT, 0); intent(h, AT_EV_MOVE, AT_DIR_RIGHT); intent(h, AT_EV_PAGE, 1); intent(h, AT_EV_ALT, 'X'); intent(h, AT_EV_PAGE, -1);
    gs.key_now[VK_RETURN] = 1; tick(); gs.key_now[VK_RETURN] = 0; tick();
    CHECK(g_set_calls == 0 && g_save_calls == 0 && FM[0].enabled == 1 && FM[1].enabled == 1);
    intent(h, AT_EV_ALT, 'Y'); CHECK(gs_mods.mode == 1);                             /* it can still be read */
    g_online = 0;                                                                    /* the session ends: the next frame allows changes again */
    tick(); intent(h, AT_EV_BACK, 0); intent(h, AT_EV_ACCEPT, 0);
    CHECK(g_set_calls == 1 && FM[0].enabled == 0);
    gw_Ui_SetClose(h);
}

static void freeze_drops_events_and_the_ring(void)
{
    int h, slot = 0, arg = 0;
    reset_door();
    h = open_mods();
    intent(h, AT_EV_BACK, 0);                                                        /* a Back is in the ring ... */
    gw_Ui_Freeze(h, 1);                                                              /* ... and a freeze drops it, so it cannot leak out after the freeze */
    CHECK(poll_one(h, &slot, &arg) == 0);
    intent(h, AT_EV_ACCEPT, 0); CHECK(g_save_calls == 0 && FM[0].enabled == 1);      /* nothing is applied while frozen */
    gs.key_now[VK_RETURN] = 1; tick(); gs.key_now[VK_RETURN] = 0; tick();
    CHECK(g_save_calls == 0);
    gw_Ui_Freeze(h, 0);
    gw_Ui_SetClose(h);
}

static void scripts_cannot_touch_it(void)
{
    int h;
    reset_door();
    h = open_mods();
    gs.cur = 1;
    CHECK(strstr(lua("return gd.ui.close('mods.list')"), "belongs to the engine") != NULL);
    CHECK(strstr(lua("return gd.ui.screen{id='mods.list', primary={kind='list', items={{id='a', label='A'}}}}"), "ERR") != NULL);
    gs.cur = 0;
    CHECK(at_stack_top(&gs_ui_stack) == (int) (slot_of(h) - gs_ui_slot));
    gw_Ui_SetClose(h);
}

int main(void)
{
    lua_State *L = luaL_newstate();
    gs.L = L; gs.cur = 0; gs.console = 0;
    snprintf(gs.s[0].id, sizeof gs.s[0].id, "console");
    snprintf(gs.s[1].id, sizeof gs.s[1].id, "envoy/main");
    gs.n = 8; gs.s[0].used = gs.s[1].used = 1;
    luaL_openlibs(L);
    lua_newtable(L); gs_push_ui(L); lua_setfield(L, -2, "ui"); lua_setglobal(L, "gd");
    opens_and_draws(); pad_intents(); poll_raw_bytes(); keys_and_mouse(); detail_and_the_mods_own_screen(); another_native_screen_still_refuses_to_be_covered();
    online_changes_nothing(); freeze_drops_events_and_the_ring(); scripts_cannot_touch_it();
    lua_close(L);
    ATLAS_DONE("atlas mods door");
}
