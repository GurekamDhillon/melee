/* gw_script.c - the Lua scripting engine: sandbox, script loading, the `gd` API, hooks, the
 * console commands and the local console socket. See gw_script.h for where it runs and the
 * determinism rule, and docs/scripting.md for the modder-facing reference.
 *
 * ONE lua_State, one environment table per script. A script's environment holds its own copies
 * of the allowed standard libraries and of `gd`, so nothing one script does to its globals or
 * library tables reaches another. The standard libraries compiled in (gw_lua.c) are base,
 * coroutine, table, string, utf8 and math; io, os, package and debug do not exist in this exe.
 *
 * LIMITS, per call from the engine into a script: an instruction budget (count hook every 1000
 * instructions) and a wall-clock budget, both reported as that script's error. A script that
 * keeps failing is switched off (its hooks stop being called) with a console message; the game
 * never goes down for a script. Total Lua memory is capped by the allocator.
 */
#include "gw.h"
#include "gw_script.h"

#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <winsock2.h>
#include <ws2tcpip.h>
#include <windows.h>

#include "../third_party/lua-5.4.7/src/lua.h"
#include "../third_party/lua-5.4.7/src/lauxlib.h"
#include "../third_party/lua-5.4.7/src/lualib.h"

/* ---- the game side (pc/gameworld/script_game.c) ---------------------------------------------- */
extern float gw_ScriptGame_FighterF(int slot, int field);
extern int gw_ScriptGame_FighterI(int slot, int field);
extern void gw_ScriptGame_SetPercent(int slot, int percent);
extern void gw_ScriptGame_SetStocks(int slot, int stocks);
extern int gw_ScriptGame_StageKind(void);
extern int gw_ScriptGame_GameMode(void);
extern void gw_ScriptGame_LaunchScene(int game_mode);
enum { SF_X, SF_Y, SF_VX, SF_VY, SF_PERCENT, SF_FACING, SF_ANIM_FRAME, SF_HITLAG };
enum { SI_PRESENT, SI_KIND, SI_CHAR, SI_ACTION, SI_AIRBORNE, SI_STOCKS, SI_COSTUME, SI_SLOT_TYPE };

/* ---- the rest of the port --------------------------------------------------------------------- */
extern void gw_SceneLaunch_SetText(const char *text);
extern int gw_SceneLaunch_BootGameMode(void);
extern const char *gw_SceneReport_ModeName(int mode);
extern const char *gw_SceneReport_SceneName(int scene_kind);
extern int gw_Snap_Resimulating(void);
extern int gw_RB_Enabled(void);
extern int gw_Netplay_Enabled(void);
extern int gw_snap_open(int k);
extern void gw_snap_save(int frame);
extern int gw_snap_load(int frame);
extern int gw_TextEntryUntil;
/* gw_script_pad.c */
extern void gw_script_pad_state(int ch, unsigned *buttons, int *sx, int *sy, int *cx, int *cy,
                                int *l, int *r);
extern void gw_script_pad_override(int ch, unsigned buttons, int sx, int sy, int cx, int cy, int l,
                                   int r, int samples);
extern void gw_script_pad_release(int ch);
extern const char *gw_script_pad_lua_path(void); /* MELEE_PAD_SCRIPT when it names a .lua */

#define GS_MAX_SCRIPTS 64
#define GS_MAX_TASKS 16
#define GS_MAX_COMMANDS 64
#define GS_MEM_CAP (64u << 20)
#define GS_SAVE_SLOTS 4
#define GS_MAX_ERRORS 20

typedef struct {
    char id[64];
    char name[64];
    char version[32];
    char author[64];
    char entry[MAX_PATH]; /* the .lua file */
    char origin[12];      /* "scripts" | "mods" | "env" | "console" | "test" */
    int api_version;
    int gameplay;
    int rollback_safe;
    int env_ref;
    int tasks[GS_MAX_TASKS]; /* registry refs to coroutines, LUA_NOREF = free */
    int task_wait[GS_MAX_TASKS];
    int errors;
    int disabled;
    int used;
    uint64_t src_hash;
} GsScript;

typedef struct {
    char name[32];
    char help[96];
    int fn_ref;
    int script; /* index into gs.s */
} GsCommand;

typedef struct {
    int used;
    int scene_epoch;
    int frame;
    int match_frame;
} GsSaveSlot;

static struct {
    int inited;
    lua_State *L;
    GsScript s[GS_MAX_SCRIPTS];
    int n;
    int cur; /* the script the engine is running now, -1 none */
    int console; /* the console's own pseudo-script */
    GsCommand cmd[GS_MAX_COMMANDS];
    int ncmd;
    size_t mem;
    long long budget;
    long long budget_per_call;
    double deadline;
    double ms_per_call;
    /* frame state */
    int frame;
    int scene_kind;
    int scene_epoch;
    int match_active;
    int match_frame;
    int last_action[6];
    int state_frame[6];
    /* pause / step */
    int paused;
    int step;
    /* savestates */
    int pending_save, pending_load; /* slot+1, 0 = none */
    GsSaveSlot slot[GS_SAVE_SLOTS];
    int snap_ready;
    /* draw list */
    GwScriptDraw draw[512];
    int ndraw;
    /* keys */
    unsigned char key_now[256], key_prev[256];
    /* console */
    int console_open;
    char exe_dir[MAX_PATH];
    char scripts_dir[MAX_PATH];
    char data_dir[MAX_PATH];
    char describe[1024];
} gs;

static double gs_now_ms(void) {
    static double freq;
    LARGE_INTEGER t;
    if (freq == 0.0) {
        LARGE_INTEGER f;
        QueryPerformanceFrequency(&f);
        freq = (double) f.QuadPart / 1000.0;
    }
    QueryPerformanceCounter(&t);
    return (double) t.QuadPart / freq;
}

static uint64_t gs_fnv(uint64_t h, const void *p, size_t n) {
    const unsigned char *b = (const unsigned char *) p;
    size_t i;
    for (i = 0; i < n; ++i) {
        h ^= b[i];
        h *= 1099511628211ull;
    }
    return h;
}

/* ============================================================================================
 * console scrollback
 * ============================================================================================ */
#define GS_LINES 400
static char gs_line[GS_LINES][200];
static uint32_t gs_line_rgba[GS_LINES];
static int gs_line_head, gs_line_count;
/* the socket client that ran the current command also gets the output */
static char *gs_capture;
static int gs_capture_cap, gs_capture_len;

static void gs_add_line(uint32_t rgba, const char *text) {
    int i = (gs_line_head + gs_line_count) % GS_LINES;
    if (gs_line_count == GS_LINES) {
        gs_line_head = (gs_line_head + 1) % GS_LINES;
        i = (gs_line_head + gs_line_count - 1) % GS_LINES;
    } else {
        gs_line_count++;
    }
    snprintf(gs_line[i], sizeof gs_line[i], "%s", text);
    gs_line_rgba[i] = rgba;
    if (gs_capture != NULL && gs_capture_len < gs_capture_cap - 1) {
        int n = snprintf(gs_capture + gs_capture_len, (size_t) (gs_capture_cap - gs_capture_len),
                         "%s\n", text);
        if (n > 0) {
            gs_capture_len += n;
            if (gs_capture_len > gs_capture_cap - 1) {
                gs_capture_len = gs_capture_cap - 1;
            }
        }
    }
}

void gw_Console_Print(uint32_t rgba, const char *fmt, ...) {
    char buf[2048], *p, *nl;
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(buf, sizeof buf, fmt, ap);
    va_end(ap);
    for (p = buf; p != NULL && *p != '\0'; p = nl) {
        nl = strchr(p, '\n');
        if (nl != NULL) {
            *nl++ = '\0';
        }
        gs_add_line(rgba, p);
    }
}

int gw_Console_LineCount(void) { return gs_line_count; }
const char *gw_Console_Line(int i) {
    return (i >= 0 && i < gs_line_count) ? gs_line[(gs_line_head + i) % GS_LINES] : "";
}
uint32_t gw_Console_LineColor(int i) {
    return (i >= 0 && i < gs_line_count) ? gs_line_rgba[(gs_line_head + i) % GS_LINES] : 0;
}
int gw_Console_Open(void) { return gs.console_open; }
void gw_Console_SetOpen(int open) { gs.console_open = open != 0; }

#define GS_WHITE 0xE8E8E8FFu
#define GS_GREY 0x9A9AA2FFu
#define GS_RED 0xFF6B6BFFu
#define GS_YELLOW 0xFFD166FFu
#define GS_GREEN 0x7BE495FFu

/* ============================================================================================
 * Lua plumbing: allocator, budget, calls, errors
 * ============================================================================================ */
static void *gs_alloc(void *ud, void *ptr, size_t osize, size_t nsize) {
    size_t old = ptr != NULL ? osize : 0;
    void *p;
    (void) ud;
    if (nsize == 0) {
        free(ptr);
        gs.mem -= old;
        return NULL;
    }
    if (gs.mem - old + nsize > GS_MEM_CAP) {
        return NULL; /* LUA_ERRMEM for the script that asked */
    }
    p = realloc(ptr, nsize);
    if (p != NULL) {
        gs.mem = gs.mem - old + nsize;
    }
    return p;
}

static void gs_count_hook(lua_State *L, lua_Debug *ar) {
    (void) ar;
    gs.budget -= 1000;
    if (gs.budget < 0 || gs_now_ms() > gs.deadline) {
        gs.budget = 0; /* keep failing until control is back with the engine */
        luaL_error(L, "ran too long (limit: %d instructions or %d ms per call)",
                   (int) gs.budget_per_call, (int) gs.ms_per_call);
    }
}

static const char *gs_script_id(int i) {
    return (i >= 0 && i < gs.n && gs.s[i].used) ? gs.s[i].id : "?";
}

static int gs_msgh(lua_State *L) {
    const char *msg = lua_tostring(L, 1);
    if (msg == NULL) {
        msg = lua_pushfstring(L, "(error object is a %s value)", luaL_typename(L, 1));
    }
    luaL_traceback(L, L, msg, 1);
    return 1;
}

static void gs_report(int script, const char *what, const char *err) {
    GsScript *s = (script >= 0 && script < gs.n) ? &gs.s[script] : NULL;
    gw_Console_Print(GS_RED, "[%s] %s: %s", gs_script_id(script), what, err);
    gw_log("script [%s] %s: %s", gs_script_id(script), what, err);
    if (s != NULL && script != gs.console && ++s->errors >= GS_MAX_ERRORS && !s->disabled) {
        s->disabled = 1;
        gw_Console_Print(GS_RED, "[%s] switched off after %d errors (\"reload\" to try again)",
                         s->id, s->errors);
    }
}

static void gs_arm_budget(void) {
    gs.budget = gs.budget_per_call;
    gs.deadline = gs_now_ms() + gs.ms_per_call;
}

/* Call the function below `nargs` arguments on the stack as script `script`. Pops the function
 * and arguments; leaves `nres` results on success. Returns 0 or -1 (error reported). */
static int gs_pcall(int script, int nargs, int nres, const char *what) {
    lua_State *L = gs.L;
    int base = lua_gettop(L) - nargs;
    int prev = gs.cur, rc;
    lua_pushcfunction(L, gs_msgh);
    lua_insert(L, base);
    gs.cur = script;
    gs_arm_budget();
    rc = lua_pcall(L, nargs, nres, base);
    gs.cur = prev;
    lua_remove(L, base);
    if (rc != LUA_OK) {
        gs_report(script, what, lua_tostring(L, -1) != NULL ? lua_tostring(L, -1) : "?");
        lua_pop(L, 1);
        return -1;
    }
    return 0;
}

/* Push script i's hook `name` if it is a function; returns 1 when pushed. */
static int gs_get_hook(int i, const char *name) {
    lua_State *L = gs.L;
    if (!gs.s[i].used || gs.s[i].disabled || gs.s[i].env_ref == LUA_NOREF) {
        return 0;
    }
    lua_rawgeti(L, LUA_REGISTRYINDEX, gs.s[i].env_ref);
    lua_getfield(L, -1, name);
    lua_remove(L, -2);
    if (lua_isfunction(L, -1)) {
        return 1;
    }
    lua_pop(L, 1);
    return 0;
}

/* The determinism gate: may script i run a hook now? */
static int gs_may_run(int i) {
    if (gw_Snap_Resimulating()) {
        return 0; /* no hook on a resimulated frame (rule in gw_script.h) */
    }
    (void) i;
    return 1;
}

static void gs_hook_all(const char *name, int nargs_int, int a, int b) {
    int i;
    for (i = 0; i < gs.n; ++i) {
        if (!gs_may_run(i) || !gs_get_hook(i, name)) {
            continue;
        }
        if (nargs_int >= 1) {
            lua_pushinteger(gs.L, a);
        }
        if (nargs_int >= 2) {
            lua_pushinteger(gs.L, b);
        }
        gs_pcall(i, nargs_int, 0, name);
    }
}

/* ============================================================================================
 * the `gd` API
 * ============================================================================================ */
static GsScript *gs_cur_script(void) {
    return (gs.cur >= 0 && gs.cur < gs.n) ? &gs.s[gs.cur] : NULL;
}

/* Gameplay writes: allowed for gameplay scripts and the console; in a netplay/rollback session
 * only for "rollback_safe" gameplay scripts (and never from the console). */
static void gs_require_gameplay(lua_State *L, const char *fn) {
    GsScript *s = gs_cur_script();
    int session = gw_RB_Enabled() || gw_Netplay_Enabled();
    if (s == NULL) {
        luaL_error(L, "gd.%s: no current script", fn);
    }
    if (gs.cur != gs.console && !s->gameplay) {
        luaL_error(L, "gd.%s changes gameplay: the script's mod.json must say \"gameplay\": true", fn);
    }
    if (session && (gs.cur == gs.console || !s->rollback_safe)) {
        luaL_error(L, "gd.%s is not allowed during a netplay/rollback session", fn);
    }
}

static int gs_slot_arg(lua_State *L, int idx) {
    lua_Integer p = luaL_checkinteger(L, idx);
    if (p < 1 || p > 6) {
        luaL_error(L, "player/port numbers are 1-6 (got %d)", (int) p);
    }
    return (int) p - 1;
}

static void gs_setnum(lua_State *L, const char *k, double v) {
    lua_pushnumber(L, v);
    lua_setfield(L, -2, k);
}
static void gs_setint(lua_State *L, const char *k, lua_Integer v) {
    lua_pushinteger(L, v);
    lua_setfield(L, -2, k);
}
static void gs_setstr(lua_State *L, const char *k, const char *v) {
    lua_pushstring(L, v);
    lua_setfield(L, -2, k);
}
static void gs_setbool(lua_State *L, const char *k, int v) {
    lua_pushboolean(L, v);
    lua_setfield(L, -2, k);
}

static const char *const gs_char_names[] = {
    "Captain Falcon", "Donkey Kong", "Fox", "Mr. Game & Watch", "Kirby", "Bowser", "Link",
    "Luigi", "Mario", "Marth", "Mewtwo", "Ness", "Peach", "Pikachu", "Ice Climbers",
    "Jigglypuff", "Samus", "Yoshi", "Zelda", "Sheik", "Falco", "Young Link", "Dr. Mario", "Roy",
    "Pichu", "Ganondorf"};

static const char *gs_char_name(int c) {
    static char buf[32];
    if (c >= 0 && c < (int) (sizeof gs_char_names / sizeof gs_char_names[0])) {
        return gs_char_names[c];
    }
    snprintf(buf, sizeof buf, "character %d", c);
    return buf;
}

static void gs_push_player(lua_State *L, int slot) {
    lua_createtable(L, 0, 20);
    gs_setint(L, "port", slot + 1);
    gs_setint(L, "char", gw_ScriptGame_FighterI(slot, SI_CHAR));
    gs_setstr(L, "char_name", gs_char_name(gw_ScriptGame_FighterI(slot, SI_CHAR)));
    gs_setint(L, "kind", gw_ScriptGame_FighterI(slot, SI_KIND));
    gs_setint(L, "costume", gw_ScriptGame_FighterI(slot, SI_COSTUME));
    gs_setbool(L, "cpu", gw_ScriptGame_FighterI(slot, SI_SLOT_TYPE) == 1);
    gs_setnum(L, "x", gw_ScriptGame_FighterF(slot, SF_X));
    gs_setnum(L, "y", gw_ScriptGame_FighterF(slot, SF_Y));
    gs_setnum(L, "vx", gw_ScriptGame_FighterF(slot, SF_VX));
    gs_setnum(L, "vy", gw_ScriptGame_FighterF(slot, SF_VY));
    gs_setnum(L, "percent", gw_ScriptGame_FighterF(slot, SF_PERCENT));
    gs_setint(L, "stocks", gw_ScriptGame_FighterI(slot, SI_STOCKS));
    gs_setint(L, "facing", gw_ScriptGame_FighterF(slot, SF_FACING) < 0.0f ? -1 : 1);
    gs_setint(L, "action", gw_ScriptGame_FighterI(slot, SI_ACTION));
    gs_setint(L, "action_frame", gs.state_frame[slot]);
    gs_setnum(L, "anim_frame", gw_ScriptGame_FighterF(slot, SF_ANIM_FRAME));
    gs_setbool(L, "airborne", gw_ScriptGame_FighterI(slot, SI_AIRBORNE) == 1);
    gs_setnum(L, "hitlag", gw_ScriptGame_FighterF(slot, SF_HITLAG));
}

static int gs_players_present(int slot) { return gw_ScriptGame_FighterI(slot, SI_PRESENT) == 1; }

static int l_log(lua_State *L) {
    luaL_Buffer b;
    int i, n = lua_gettop(L);
    luaL_buffinit(L, &b);
    for (i = 1; i <= n; ++i) {
        if (i > 1) {
            luaL_addchar(&b, ' ');
        }
        luaL_tolstring(L, i, NULL);
        luaL_addvalue(&b);
    }
    luaL_pushresult(&b);
    gw_Console_Print(GS_WHITE, "[%s] %s", gs_script_id(gs.cur), lua_tostring(L, -1));
    gw_log("script [%s] %s", gs_script_id(gs.cur), lua_tostring(L, -1));
    return 0;
}

static int l_frame(lua_State *L) {
    lua_pushinteger(L, gs.frame);
    return 1;
}

static int l_time(lua_State *L) {
    lua_pushnumber(L, gs_now_ms() / 1000.0);
    return 1;
}

static int l_scene(lua_State *L) {
    int mode = gw_ScriptGame_GameMode();
    lua_createtable(L, 0, 5);
    gs_setint(L, "kind", gs.scene_kind);
    gs_setstr(L, "name", gw_SceneReport_SceneName(gs.scene_kind));
    gs_setint(L, "mode", mode);
    gs_setstr(L, "mode_name", gw_SceneReport_ModeName(mode));
    gs_setint(L, "epoch", gs.scene_epoch);
    return 1;
}

static int l_match(lua_State *L) {
    lua_createtable(L, 0, 4);
    gs_setbool(L, "active", gs.match_active);
    gs_setint(L, "frame", gs.match_frame);
    gs_setint(L, "stage", gs.match_active ? gw_ScriptGame_StageKind() : -1);
    gs_setbool(L, "netplay", gw_Netplay_Enabled() || gw_RB_Enabled());
    return 1;
}

static int l_players(lua_State *L) {
    int slot, n = 0;
    lua_newtable(L);
    for (slot = 0; slot < 6; ++slot) {
        if (gs_players_present(slot)) {
            gs_push_player(L, slot);
            lua_rawseti(L, -2, ++n);
        }
    }
    return 1;
}

static int l_player(lua_State *L) {
    int slot = gs_slot_arg(L, 1);
    if (!gs_players_present(slot)) {
        lua_pushnil(L);
        return 1;
    }
    gs_push_player(L, slot);
    return 1;
}

static int l_char_name(lua_State *L) {
    lua_pushstring(L, gs_char_name((int) luaL_checkinteger(L, 1)));
    return 1;
}

/* buttons: "A+B", "A B", 0x0100, or a table of names */
static const struct { const char *name; unsigned bit; } gs_buttons[] = {
    {"A", 0x0100}, {"B", 0x0200}, {"X", 0x0400}, {"Y", 0x0800}, {"START", 0x1000},
    {"L", 0x0040}, {"R", 0x0020}, {"Z", 0x0010}, {"UP", 0x0008}, {"DOWN", 0x0004},
    {"LEFT", 0x0001}, {"RIGHT", 0x0002}, {"DUP", 0x0008}, {"DDOWN", 0x0004}, {"DLEFT", 0x0001},
    {"DRIGHT", 0x0002}};

static unsigned gs_parse_buttons(lua_State *L, int idx) {
    unsigned bits = 0;
    if (lua_isnoneornil(L, idx)) {
        return 0;
    }
    if (lua_isinteger(L, idx)) {
        return (unsigned) lua_tointeger(L, idx) & 0xFFFFu;
    }
    if (lua_type(L, idx) == LUA_TSTRING) {
        const char *s = lua_tostring(L, idx);
        char tok[16];
        while (*s != '\0') {
            int n = 0;
            size_t k;
            while (*s == '+' || *s == ' ' || *s == ',' || *s == '|') {
                ++s;
            }
            while (*s != '\0' && *s != '+' && *s != ' ' && *s != ',' && *s != '|' && n < 15) {
                char c = *s++;
                tok[n++] = (char) (c >= 'a' && c <= 'z' ? c - 32 : c);
            }
            tok[n] = '\0';
            if (n == 0) {
                continue;
            }
            for (k = 0; k < sizeof gs_buttons / sizeof gs_buttons[0]; ++k) {
                if (strcmp(tok, gs_buttons[k].name) == 0) {
                    bits |= gs_buttons[k].bit;
                    break;
                }
            }
            if (k == sizeof gs_buttons / sizeof gs_buttons[0]) {
                luaL_error(L, "unknown button \"%s\" (A B X Y START L R Z UP DOWN LEFT RIGHT)", tok);
            }
        }
        return bits;
    }
    luaL_error(L, "buttons are a string like \"A+B\" or a number");
    return 0;
}

static int gs_field_int(lua_State *L, int t, const char *k, int def) {
    int v = def;
    lua_getfield(L, t, k);
    if (lua_isnumber(L, -1)) {
        v = (int) lua_tointeger(L, -1);
        if (!lua_isinteger(L, -1)) {
            v = (int) lua_tonumber(L, -1);
        }
    }
    lua_pop(L, 1);
    return v;
}

static int gs_clampi(int v, int lo, int hi) { return v < lo ? lo : v > hi ? hi : v; }

/* gd.input(port, spec, frames): spec = "A+B" | {buttons="A", x=, y=, cx=, cy=, l=, r=} */
static int l_input(lua_State *L) {
    int ch = gs_slot_arg(L, 1);
    unsigned buttons = 0;
    int sx = 0, sy = 0, cx = 0, cy = 0, tl = 0, tr = 0;
    int frames = (int) luaL_optinteger(L, 3, 1);
    gs_require_gameplay(L, "input");
    if (ch > 3) {
        luaL_error(L, "gd.input: ports are 1-4");
    }
    if (lua_istable(L, 2)) {
        lua_getfield(L, 2, "buttons");
        buttons = gs_parse_buttons(L, lua_gettop(L));
        lua_pop(L, 1);
        sx = gs_clampi(gs_field_int(L, 2, "x", 0), -127, 127);
        sy = gs_clampi(gs_field_int(L, 2, "y", 0), -127, 127);
        cx = gs_clampi(gs_field_int(L, 2, "cx", 0), -127, 127);
        cy = gs_clampi(gs_field_int(L, 2, "cy", 0), -127, 127);
        tl = gs_clampi(gs_field_int(L, 2, "l", 0), 0, 255);
        tr = gs_clampi(gs_field_int(L, 2, "r", 0), 0, 255);
    } else {
        buttons = gs_parse_buttons(L, 2);
    }
    gw_script_pad_override(ch, buttons, sx, sy, cx, cy, tl, tr, frames < 1 ? 1 : frames);
    return 0;
}

static int l_release(lua_State *L) {
    int ch = gs_slot_arg(L, 1);
    if (ch <= 3) {
        gw_script_pad_release(ch);
    }
    return 0;
}

static int l_pad(lua_State *L) {
    int ch = gs_slot_arg(L, 1);
    unsigned b = 0;
    int sx = 0, sy = 0, cx = 0, cy = 0, tl = 0, tr = 0;
    size_t k;
    if (ch > 3) {
        luaL_error(L, "gd.pad: ports are 1-4");
    }
    gw_script_pad_state(ch, &b, &sx, &sy, &cx, &cy, &tl, &tr);
    lua_createtable(L, 0, 20);
    gs_setint(L, "buttons", b);
    gs_setint(L, "x", sx);
    gs_setint(L, "y", sy);
    gs_setint(L, "cx", cx);
    gs_setint(L, "cy", cy);
    gs_setint(L, "l", tl);
    gs_setint(L, "r", tr);
    for (k = 0; k < 12; ++k) { /* the first 12 entries are the distinct names */
        gs_setbool(L, gs_buttons[k].name, (b & gs_buttons[k].bit) != 0);
    }
    return 1;
}

static int l_savestate(lua_State *L) {
    int slot = (int) luaL_optinteger(L, 1, 1);
    gs_require_gameplay(L, "savestate");
    if (slot < 1 || slot > GS_SAVE_SLOTS) {
        luaL_error(L, "savestate slots are 1-%d", GS_SAVE_SLOTS);
    }
    gs.pending_save = slot;
    return 0;
}

static int l_loadstate(lua_State *L) {
    int slot = (int) luaL_optinteger(L, 1, 1);
    gs_require_gameplay(L, "loadstate");
    if (slot < 1 || slot > GS_SAVE_SLOTS) {
        luaL_error(L, "savestate slots are 1-%d", GS_SAVE_SLOTS);
    }
    if (!gs.slot[slot - 1].used) {
        luaL_error(L, "savestate slot %d is empty", slot);
    }
    if (gs.slot[slot - 1].scene_epoch != gs.scene_epoch) {
        luaL_error(L, "savestate slot %d was saved in another scene", slot);
    }
    gs.pending_load = slot;
    return 0;
}

static int l_pause(lua_State *L) {
    gs_require_gameplay(L, "pause");
    gs.paused = 1;
    return 0;
}
static int l_resume(lua_State *L) {
    (void) L;
    gs.paused = 0;
    gs.step = 0;
    return 0;
}
static int l_step(lua_State *L) {
    int n = (int) luaL_optinteger(L, 1, 1);
    gs_require_gameplay(L, "step");
    gs.paused = 1;
    gs.step += n < 1 ? 1 : n;
    return 0;
}
static int l_paused(lua_State *L) {
    lua_pushboolean(L, gs.paused);
    return 1;
}

static int l_set_percent(lua_State *L) {
    int slot = gs_slot_arg(L, 1);
    int p = (int) luaL_checknumber(L, 2);
    gs_require_gameplay(L, "set_percent");
    gw_ScriptGame_SetPercent(slot, gs_clampi(p, 0, 999));
    return 0;
}

static int l_set_stocks(lua_State *L) {
    int slot = gs_slot_arg(L, 1);
    int n = (int) luaL_checkinteger(L, 2);
    gs_require_gameplay(L, "set_stocks");
    gw_ScriptGame_SetStocks(slot, gs_clampi(n, 0, 99));
    return 0;
}

/* gd.scene_launch("mode=training;p1=fox") or gd.scene_launch{mode="training", p1="fox"} */
static int gs_pending_launch = -1;
static int l_scene_launch(lua_State *L) {
    char text[512];
    int mode;
    gs_require_gameplay(L, "scene_launch");
    if (lua_istable(L, 1)) {
        size_t n = 0;
        text[0] = '\0';
        lua_getfield(L, 1, "mode"); /* mode first, the rest in any order */
        if (lua_type(L, -1) == LUA_TSTRING) {
            n += (size_t) snprintf(text + n, sizeof text - n, "mode=%s", lua_tostring(L, -1));
        }
        lua_pop(L, 1);
        lua_pushnil(L);
        while (lua_next(L, 1) != 0 && n < sizeof text - 1) {
            if (lua_type(L, -2) == LUA_TSTRING && strcmp(lua_tostring(L, -2), "mode") != 0) {
                const char *v = luaL_tolstring(L, -1, NULL);
                n += (size_t) snprintf(text + n, sizeof text - n, "%s%s=%s", n ? ";" : "",
                                       lua_tostring(L, -3), v);
                lua_pop(L, 1);
            }
            lua_pop(L, 1);
        }
    } else {
        snprintf(text, sizeof text, "%s", luaL_checkstring(L, 1));
    }
    gw_SceneLaunch_SetText(text);
    mode = gw_SceneLaunch_BootGameMode();
    if (mode < 0) {
        luaL_error(L, "gd.scene_launch: \"%s\" names no scene (see _research/scene-launch.md)", text);
    }
    gs_pending_launch = mode; /* acted on at the next tick, outside any frame */
    lua_pushstring(L, text);
    return 1;
}

static int l_scene_clear(lua_State *L) {
    (void) L;
    gw_SceneLaunch_SetText(NULL);
    return 0;
}

/* ---- drawing ---------------------------------------------------------------------------------- */
static uint32_t gs_color_arg(lua_State *L, int idx, uint32_t def) {
    if (lua_isnoneornil(L, idx)) {
        return def;
    }
    if (lua_isinteger(L, idx)) {
        return (uint32_t) lua_tointeger(L, idx); /* always 0xRRGGBBAA; gd.rgb builds one */
    }
    if (lua_istable(L, idx)) {
        int r = gs_field_int(L, idx, "r", 255), g = gs_field_int(L, idx, "g", 255);
        int b = gs_field_int(L, idx, "b", 255), a = gs_field_int(L, idx, "a", 255);
        return ((uint32_t) gs_clampi(r, 0, 255) << 24) | ((uint32_t) gs_clampi(g, 0, 255) << 16) |
               ((uint32_t) gs_clampi(b, 0, 255) << 8) | (uint32_t) gs_clampi(a, 0, 255);
    }
    luaL_error(L, "colours are 0xRRGGBBAA numbers (gd.rgb(r, g, b [, a])) or {r=,g=,b=,a=}");
    return def;
}

static GwScriptDraw *gs_draw_new(int kind) {
    GwScriptDraw *d;
    if (gs.ndraw >= (int) (sizeof gs.draw / sizeof gs.draw[0])) {
        return NULL;
    }
    d = &gs.draw[gs.ndraw++];
    memset(d, 0, sizeof *d);
    d->kind = kind;
    d->size = 1.0f;
    return d;
}

static int l_text(lua_State *L) {
    GwScriptDraw *d = gs_draw_new(GW_SDRAW_TEXT);
    const char *s;
    float x = (float) luaL_checknumber(L, 1), y = (float) luaL_checknumber(L, 2);
    luaL_tolstring(L, 3, NULL);
    s = lua_tostring(L, -1);
    if (d != NULL) {
        d->x = x;
        d->y = y;
        d->rgba = gs_color_arg(L, 4, 0xFFFFFFFFu);
        d->size = (float) luaL_optnumber(L, 5, 1.0);
        snprintf(d->text, sizeof d->text, "%s", s);
    }
    return 0;
}

static int gs_rect(lua_State *L, int kind) {
    GwScriptDraw *d = gs_draw_new(kind);
    float x = (float) luaL_checknumber(L, 1), y = (float) luaL_checknumber(L, 2);
    float w = (float) luaL_checknumber(L, 3), h = (float) luaL_checknumber(L, 4);
    if (d != NULL) {
        d->x = x;
        d->y = y;
        d->w = w;
        d->h = h;
        d->rgba = gs_color_arg(L, 5, kind == GW_SDRAW_FILL ? 0x000000A0u : 0xFFFFFFFFu);
    }
    return 0;
}
static int l_rgb(lua_State *L) {
    int r = gs_clampi((int) luaL_checkinteger(L, 1), 0, 255);
    int g = gs_clampi((int) luaL_checkinteger(L, 2), 0, 255);
    int b = gs_clampi((int) luaL_checkinteger(L, 3), 0, 255);
    int a = gs_clampi((int) luaL_optinteger(L, 4, 255), 0, 255);
    lua_pushinteger(L, ((lua_Integer) r << 24) | (g << 16) | (b << 8) | a);
    return 1;
}

extern void gw_Overlay_SetRunLabel(const char *text); /* gw_overlay.cpp */
static int l_label(lua_State *L) {
    gw_Overlay_SetRunLabel(luaL_optstring(L, 1, ""));
    return 0;
}

/* Screenshots: beta's gw_Screenshot(path) (final-frame readback to PNG, arriving with the Aurora
 * rebuild). Until it links, /alternatename binds the name to a stub that says so - no rebuild of
 * this file is needed when it lands. Returns 0 on success. */
int gw_Screenshot_missing(const char *path) {
    (void) path;
    return -2;
}
#pragma comment(linker, "/alternatename:_gw_Screenshot=_gw_Screenshot_missing")
extern int gw_Screenshot(const char *path);

static int gs_screenshot(const char *path) {
    int rc = gw_Screenshot(path);
    if (rc == -2) {
        gw_Console_Print(GS_YELLOW, "screenshot: not available yet (gw_Screenshot is not in this build)");
    } else if (rc != 0) {
        gw_Console_Print(GS_RED, "screenshot: failed (%d) for %s", rc, path);
    } else {
        gw_Console_Print(GS_GREEN, "screenshot: %s", path);
    }
    return rc;
}

/* gd.screenshot(name): into the script's data folder (a bare file name, .png added) */
static void gs_data_path(lua_State *L, const char *name, char *out, size_t cap);
static int l_screenshot(lua_State *L) {
    char path[MAX_PATH], name[80];
    const char *n = luaL_optstring(L, 1, "shot");
    size_t len = strlen(n);
    snprintf(name, sizeof name, "%s%s", n, (len > 4 && _stricmp(n + len - 4, ".png") == 0) ? "" : ".png");
    gs_data_path(L, name, path, sizeof path);
    lua_pushboolean(L, gs_screenshot(path) == 0);
    lua_pushstring(L, path);
    return 2;
}

/* gd.quit(): close the game window the way the user would (gameplay scripts and the console) */
static BOOL CALLBACK gs_close_cb(HWND w, LPARAM lp) {
    (void) lp;
    if (IsWindowVisible(w)) PostMessageA(w, WM_CLOSE, 0, 0);
    return TRUE;
}
static int l_quit(lua_State *L) {
    gs_require_gameplay(L, "quit");
    gw_log("script [%s]: quit", gs_script_id(gs.cur));
    EnumThreadWindows(GetCurrentThreadId(), gs_close_cb, 0);
    return 0;
}

static int l_box(lua_State *L) { return gs_rect(L, GW_SDRAW_BOX); }
static int l_fill(lua_State *L) { return gs_rect(L, GW_SDRAW_FILL); }
static int l_line(lua_State *L) { return gs_rect(L, GW_SDRAW_LINE); }

/* ---- keyboard --------------------------------------------------------------------------------- */
static int gs_vk(lua_State *L, const char *name) {
    static const struct { const char *n; int vk; } named[] = {
        {"SPACE", VK_SPACE}, {"ENTER", VK_RETURN}, {"TAB", VK_TAB}, {"ESCAPE", VK_ESCAPE},
        {"BACKSPACE", VK_BACK}, {"SHIFT", VK_SHIFT}, {"CTRL", VK_CONTROL}, {"ALT", VK_MENU},
        {"LEFT", VK_LEFT}, {"RIGHT", VK_RIGHT}, {"UP", VK_UP}, {"DOWN", VK_DOWN},
        {"HOME", VK_HOME}, {"END", VK_END}, {"PAGEUP", VK_PRIOR}, {"PAGEDOWN", VK_NEXT},
        {"INSERT", VK_INSERT}, {"DELETE", VK_DELETE}};
    char up[16];
    size_t i, n = strlen(name);
    if (n == 0 || n >= sizeof up) {
        luaL_error(L, "unknown key \"%s\"", name);
    }
    for (i = 0; i <= n; ++i) {
        up[i] = (char) (name[i] >= 'a' && name[i] <= 'z' ? name[i] - 32 : name[i]);
    }
    if (n == 1 && ((up[0] >= 'A' && up[0] <= 'Z') || (up[0] >= '0' && up[0] <= '9'))) {
        return up[0];
    }
    if (up[0] == 'F' && n >= 2 && n <= 3) {
        int f = atoi(up + 1);
        if (f >= 1 && f <= 12) {
            return VK_F1 + f - 1;
        }
    }
    if (up[0] == 'K' && up[1] == 'P' && n == 3 && up[2] >= '0' && up[2] <= '9') {
        return VK_NUMPAD0 + (up[2] - '0');
    }
    for (i = 0; i < sizeof named / sizeof named[0]; ++i) {
        if (strcmp(up, named[i].n) == 0) {
            return named[i].vk;
        }
    }
    luaL_error(L, "unknown key \"%s\" (A-Z 0-9 F1-F12 KP0-KP9 SPACE ENTER SHIFT CTRL ALT arrows ...)",
               name);
    return 0;
}

static int l_key(lua_State *L) {
    int vk = gs_vk(L, luaL_checkstring(L, 1));
    lua_pushboolean(L, gs.key_now[vk]);
    return 1;
}
static int l_key_pressed(lua_State *L) {
    int vk = gs_vk(L, luaL_checkstring(L, 1));
    lua_pushboolean(L, gs.key_now[vk] && !gs.key_prev[vk]);
    return 1;
}

static void gs_poll_keys(void) {
    HWND fg = GetForegroundWindow();
    DWORD pid = 0;
    int vk, focused;
    if (fg != NULL) {
        GetWindowThreadProcessId(fg, &pid);
    }
    focused = pid == GetCurrentProcessId() && !gs.console_open;
    memcpy(gs.key_prev, gs.key_now, sizeof gs.key_now);
    for (vk = 1; vk < 256; ++vk) {
        gs.key_now[vk] = (unsigned char) (focused && (GetAsyncKeyState(vk) & 0x8000) != 0);
    }
}

/* ---- console commands registered by scripts ---------------------------------------------------- */
static int l_command(lua_State *L) {
    const char *name = luaL_checkstring(L, 1);
    const char *help = luaL_optstring(L, 3, "");
    int i;
    luaL_checktype(L, 2, LUA_TFUNCTION);
    for (i = 0; i < gs.ncmd; ++i) {
        if (_stricmp(gs.cmd[i].name, name) == 0) {
            break;
        }
    }
    if (i == gs.ncmd) {
        if (gs.ncmd >= GS_MAX_COMMANDS) {
            luaL_error(L, "too many console commands");
        }
        gs.ncmd++;
    } else {
        luaL_unref(L, LUA_REGISTRYINDEX, gs.cmd[i].fn_ref);
    }
    snprintf(gs.cmd[i].name, sizeof gs.cmd[i].name, "%s", name);
    snprintf(gs.cmd[i].help, sizeof gs.cmd[i].help, "%s", help);
    lua_pushvalue(L, 2);
    gs.cmd[i].fn_ref = luaL_ref(L, LUA_REGISTRYINDEX);
    gs.cmd[i].script = gs.cur;
    return 0;
}

/* ---- tasks (input scripts that wait on game state) ---------------------------------------------- */
static int l_run(lua_State *L) {
    GsScript *s = gs_cur_script();
    lua_State *co;
    int k;
    luaL_checktype(L, 1, LUA_TFUNCTION);
    if (s == NULL) {
        luaL_error(L, "gd.run: no current script");
    }
    for (k = 0; k < GS_MAX_TASKS && s->tasks[k] != LUA_NOREF; ++k) {
    }
    if (k == GS_MAX_TASKS) {
        luaL_error(L, "gd.run: at most %d tasks per script", GS_MAX_TASKS);
    }
    co = lua_newthread(L);
    lua_pushvalue(L, 1);
    lua_xmove(L, co, 1);
    s->tasks[k] = luaL_ref(L, LUA_REGISTRYINDEX); /* pops the thread */
    s->task_wait[k] = 0;
    lua_pushinteger(L, k + 1);
    return 1;
}

static void gs_run_tasks(void) {
    int i, k;
    lua_State *L = gs.L;
    for (i = 0; i < gs.n; ++i) {
        GsScript *s = &gs.s[i];
        if (!s->used || s->disabled || !gs_may_run(i)) {
            continue;
        }
        for (k = 0; k < GS_MAX_TASKS; ++k) {
            lua_State *co;
            int nres = 0, rc, prev;
            if (s->tasks[k] == LUA_NOREF) {
                continue;
            }
            if (s->task_wait[k] > 0) {
                s->task_wait[k]--;
                continue;
            }
            lua_rawgeti(L, LUA_REGISTRYINDEX, s->tasks[k]);
            co = lua_tothread(L, -1);
            lua_pop(L, 1);
            prev = gs.cur;
            gs.cur = i;
            gs_arm_budget();
            rc = lua_resume(co, L, 0, &nres);
            gs.cur = prev;
            if (rc == LUA_YIELD) {
                /* gd.wait(n) yields n: skip n-1 more frames */
                int n = (nres >= 1 && lua_isinteger(co, -1)) ? (int) lua_tointeger(co, -1) : 1;
                s->task_wait[k] = n > 1 ? n - 1 : 0;
                lua_pop(co, nres);
                continue;
            }
            if (rc != LUA_OK) {
                luaL_traceback(L, co, lua_tostring(co, -1), 0);
                gs_report(i, "task", lua_tostring(L, -1));
                lua_pop(L, 1);
            }
            luaL_unref(L, LUA_REGISTRYINDEX, s->tasks[k]);
            s->tasks[k] = LUA_NOREF;
        }
    }
}

/* ---- per-script data folder ---------------------------------------------------------------------- */
static void gs_data_path(lua_State *L, const char *name, char *out, size_t cap) {
    GsScript *s = gs_cur_script();
    const char *p;
    char dir[MAX_PATH];
    if (s == NULL) {
        luaL_error(L, "no current script");
    }
    if (name[0] == '\0' || strlen(name) > 64) {
        luaL_error(L, "data file names are 1-64 characters");
    }
    for (p = name; *p != '\0'; ++p) {
        if (!((*p >= 'a' && *p <= 'z') || (*p >= 'A' && *p <= 'Z') || (*p >= '0' && *p <= '9') ||
              *p == '_' || *p == '-' || (*p == '.' && p != name))) {
            luaL_error(L, "data file names use letters, digits, _ - and . (got \"%s\")", name);
        }
    }
    snprintf(dir, sizeof dir, "%s\\%s", gs.data_dir, s->id);
    for (p = dir; *p != '\0'; ++p) {
        if (*p == '/') {
            *(char *) p = '_'; /* a mod script's id "mod/file" is one folder */
        }
    }
    CreateDirectoryA(gs.data_dir, NULL);
    CreateDirectoryA(dir, NULL);
    snprintf(out, cap, "%s\\%s", dir, name);
}

static int l_data_read(lua_State *L) {
    char path[MAX_PATH];
    FILE *f;
    long n;
    char *buf;
    gs_data_path(L, luaL_checkstring(L, 1), path, sizeof path);
    f = fopen(path, "rb");
    if (f == NULL) {
        lua_pushnil(L);
        return 1;
    }
    fseek(f, 0, SEEK_END);
    n = ftell(f);
    fseek(f, 0, SEEK_SET);
    if (n < 0 || n > (1 << 20)) {
        fclose(f);
        luaL_error(L, "data file too large");
    }
    buf = (char *) malloc((size_t) n + 1);
    if (buf == NULL) {
        fclose(f);
        luaL_error(L, "out of memory");
    }
    n = (long) fread(buf, 1, (size_t) n, f);
    fclose(f);
    lua_pushlstring(L, buf, (size_t) n);
    free(buf);
    return 1;
}

static int l_data_write(lua_State *L) {
    char path[MAX_PATH];
    size_t len;
    const char *text;
    FILE *f;
    gs_data_path(L, luaL_checkstring(L, 1), path, sizeof path);
    text = luaL_checklstring(L, 2, &len);
    if (len > (1 << 20)) {
        luaL_error(L, "data files are at most 1 MB");
    }
    f = fopen(path, "wb");
    if (f == NULL) {
        luaL_error(L, "cannot write %s", luaL_checkstring(L, 1));
    }
    fwrite(text, 1, len, f);
    fclose(f);
    return 0;
}

static int l_script_info(lua_State *L) {
    GsScript *s = gs_cur_script();
    lua_createtable(L, 0, 6);
    if (s != NULL) {
        gs_setstr(L, "id", s->id);
        gs_setstr(L, "name", s->name);
        gs_setstr(L, "version", s->version);
        gs_setstr(L, "author", s->author);
        gs_setbool(L, "gameplay", s->gameplay);
        gs_setbool(L, "rollback_safe", s->rollback_safe);
    }
    return 1;
}

/* print() and load() inside the sandbox */
static int l_load(lua_State *L) {
    size_t len;
    const char *chunk = luaL_checklstring(L, 1, &len);
    const char *name = luaL_optstring(L, 2, "=(load)");
    GsScript *s = gs_cur_script();
    if (luaL_loadbufferx(L, chunk, len, name, "t") != LUA_OK) { /* text only: no bytecode */
        lua_pushnil(L);
        lua_insert(L, -2);
        return 2;
    }
    if (!lua_isnoneornil(L, 4)) {
        lua_pushvalue(L, 4);
    } else if (s != NULL) {
        lua_rawgeti(L, LUA_REGISTRYINDEX, s->env_ref);
    } else {
        lua_pushnil(L);
    }
    if (!lua_setupvalue(L, -2, 1)) {
        lua_pop(L, 1);
    }
    return 1;
}

static int l_collectgarbage(lua_State *L) {
    const char *opt = luaL_optstring(L, 1, "count");
    if (strcmp(opt, "count") == 0) {
        lua_pushnumber(L, (double) lua_gc(L, LUA_GCCOUNT, 0) + lua_gc(L, LUA_GCCOUNTB, 0) / 1024.0);
        return 1;
    }
    luaL_error(L, "collectgarbage: only \"count\" is available to scripts");
    return 0;
}

static const luaL_Reg gs_gd_funcs[] = {
    {"log", l_log}, {"frame", l_frame}, {"time", l_time}, {"scene", l_scene}, {"match", l_match},
    {"players", l_players}, {"player", l_player}, {"char_name", l_char_name}, {"pad", l_pad},
    {"input", l_input}, {"release", l_release}, {"savestate", l_savestate},
    {"loadstate", l_loadstate}, {"pause", l_pause}, {"resume", l_resume}, {"step", l_step},
    {"paused", l_paused}, {"set_percent", l_set_percent}, {"set_stocks", l_set_stocks},
    {"scene_launch", l_scene_launch}, {"scene_clear", l_scene_clear}, {"text", l_text},
    {"box", l_box}, {"fill", l_fill}, {"line", l_line}, {"key", l_key},
    {"key_pressed", l_key_pressed}, {"command", l_command}, {"run", l_run},
    {"data_read", l_data_read}, {"data_write", l_data_write}, {"script", l_script_info},
    {"rgb", l_rgb}, {"label", l_label}, {"screenshot", l_screenshot}, {"quit", l_quit},
    {NULL, NULL}};

/* Lua-side helpers, compiled once into the shared base (they only use the public API). */
static const char gs_prelude[] =
    "local gd, coroutine = ...\n"
    "function gd.wait(n) return coroutine.yield(math.max(1, math.floor(n or 1))) end\n"
    "function gd.wait_until(fn, timeout)\n"
    "  local t = 0\n"
    "  while not fn() do\n"
    "    if timeout and t >= timeout then return false end\n"
    "    coroutine.yield(1); t = t + 1\n"
    "  end\n"
    "  return true\n"
    "end\n"
    "function gd.press(port, buttons, frames, extra)\n"
    "  frames = frames or 1\n"
    "  local spec = extra or {}\n"
    "  spec.buttons = buttons\n"
    "  gd.input(port, spec, frames)\n"
    "  gd.wait(frames)\n"
    "end\n"
    "function gd.tilt(port, x, y, frames) gd.input(port, {x = x, y = y}, frames or 1) gd.wait(frames or 1) end\n";

/* ============================================================================================
 * sandbox: the shared base and per-script environments
 * ============================================================================================ */
static int gs_base_ref = LUA_NOREF; /* the template environment */

static void gs_copy_table(lua_State *L, int src) {
    src = lua_absindex(L, src);
    lua_newtable(L);
    lua_pushnil(L);
    while (lua_next(L, src) != 0) {
        lua_pushvalue(L, -2);
        lua_insert(L, -2);
        lua_rawset(L, -4);
    }
}

static void gs_build_base(lua_State *L) {
    static const char *const keep[] = {"assert", "error", "ipairs", "next", "pairs", "pcall",
                                       "select", "tonumber", "tostring", "type", "xpcall",
                                       "rawequal", "rawget", "rawset", "rawlen", "setmetatable",
                                       "getmetatable", "_VERSION", NULL};
    static const char *const libs[] = {"string", "table", "math", "utf8", "coroutine", NULL};
    int i;
    luaL_requiref(L, "_G", luaopen_base, 1);
    lua_pop(L, 1);
    luaL_requiref(L, "coroutine", luaopen_coroutine, 1);
    lua_pop(L, 1);
    luaL_requiref(L, "table", luaopen_table, 1);
    lua_pop(L, 1);
    luaL_requiref(L, "string", luaopen_string, 1);
    lua_pop(L, 1);
    luaL_requiref(L, "utf8", luaopen_utf8, 1);
    lua_pop(L, 1);
    luaL_requiref(L, "math", luaopen_math, 1);
    lua_pop(L, 1);

    /* string.dump would hand out bytecode; bytecode is never loadable anyway ("t" mode) */
    lua_getglobal(L, "string");
    lua_pushnil(L);
    lua_setfield(L, -2, "dump");
    lua_pop(L, 1);
    /* lock the string metatable: getmetatable("").__index would reach the shared string table */
    lua_pushliteral(L, "");
    if (lua_getmetatable(L, -1)) {
        lua_pushliteral(L, "locked");
        lua_setfield(L, -2, "__metatable");
        lua_pop(L, 1);
    }
    lua_pop(L, 1);
    /* a fixed seed: gameplay scripts that use math.random stay reproducible */
    lua_getglobal(L, "math");
    lua_getfield(L, -1, "randomseed");
    lua_pushinteger(L, 0);
    lua_call(L, 1, 0);
    lua_pop(L, 1);

    lua_newtable(L); /* the base template */
    for (i = 0; keep[i] != NULL; ++i) {
        lua_getglobal(L, keep[i]);
        lua_setfield(L, -2, keep[i]);
    }
    for (i = 0; libs[i] != NULL; ++i) {
        lua_getglobal(L, libs[i]);
        lua_setfield(L, -2, libs[i]);
    }
    lua_pushcfunction(L, l_log);
    lua_setfield(L, -2, "print");
    lua_pushcfunction(L, l_load);
    lua_setfield(L, -2, "load");
    lua_pushcfunction(L, l_collectgarbage);
    lua_setfield(L, -2, "collectgarbage");
    lua_getfield(L, -1, "table");
    lua_getfield(L, -1, "unpack");
    lua_setfield(L, -3, "unpack");
    lua_pop(L, 1);

    lua_newtable(L); /* gd */
    luaL_setfuncs(L, gs_gd_funcs, 0);
    lua_pushinteger(L, GW_SCRIPT_API_VERSION);
    lua_setfield(L, -2, "api_version");
    lua_pushstring(L, "GD's Melee scripting API 1");
    lua_setfield(L, -2, "api_name");
    lua_newtable(L);
    lua_setfield(L, -2, "deprecated"); /* name -> "use X instead", filled as the API evolves */
    lua_newtable(L);
    for (i = 0; i < 12; ++i) {
        lua_pushinteger(L, gs_buttons[i].bit);
        lua_setfield(L, -2, gs_buttons[i].name);
    }
    lua_setfield(L, -2, "buttons");
    /* the prelude adds wait / wait_until / press / tilt */
    if (luaL_loadbufferx(L, gs_prelude, sizeof gs_prelude - 1, "=gd.prelude", "t") == LUA_OK) {
        lua_pushvalue(L, -2);
        lua_getglobal(L, "coroutine");
        if (lua_pcall(L, 2, 0, 0) != LUA_OK) {
            gw_log("script: prelude failed: %s", lua_tostring(L, -1));
            lua_pop(L, 1);
        }
    } else {
        gw_log("script: prelude did not compile: %s", lua_tostring(L, -1));
        lua_pop(L, 1);
    }
    lua_setfield(L, -2, "gd");
    gs_base_ref = luaL_ref(L, LUA_REGISTRYINDEX);
}

/* A fresh environment: the base's entries, with private copies of every library table and gd. */
static int gs_new_env(lua_State *L) {
    static const char *const copy[] = {"string", "table", "math", "utf8", "coroutine", "gd", NULL};
    int i;
    lua_rawgeti(L, LUA_REGISTRYINDEX, gs_base_ref);
    gs_copy_table(L, -1);
    for (i = 0; copy[i] != NULL; ++i) {
        lua_getfield(L, -2, copy[i]);
        gs_copy_table(L, -1);
        lua_setfield(L, -3, copy[i]);
        lua_pop(L, 1);
    }
    lua_pushvalue(L, -1);
    lua_setfield(L, -2, "_G");
    lua_remove(L, -2);
    return luaL_ref(L, LUA_REGISTRYINDEX);
}

/* ============================================================================================
 * loading scripts
 * ============================================================================================ */
static char *gs_read_file(const char *path, size_t *len) {
    FILE *f = fopen(path, "rb");
    long n;
    char *buf;
    if (f == NULL) {
        return NULL;
    }
    fseek(f, 0, SEEK_END);
    n = ftell(f);
    fseek(f, 0, SEEK_SET);
    if (n < 0 || n > (4 << 20)) {
        fclose(f);
        return NULL;
    }
    buf = (char *) malloc((size_t) n + 1);
    if (buf == NULL) {
        fclose(f);
        return NULL;
    }
    n = (long) fread(buf, 1, (size_t) n, f);
    fclose(f);
    buf[n] = '\0';
    *len = (size_t) n;
    return buf;
}

/* mod.json: a flat object; strings, numbers, true/false (strings "yes"/"true" count as true). */
static int gs_json_field(const char *json, const char *key, char *out, size_t cap) {
    char pat[80];
    const char *p;
    size_t n = 0;
    snprintf(pat, sizeof pat, "\"%s\"", key);
    p = strstr(json, pat);
    if (p == NULL) {
        return 0;
    }
    p += strlen(pat);
    while (*p == ' ' || *p == '\t' || *p == '\r' || *p == '\n') {
        ++p;
    }
    if (*p != ':') {
        return 0;
    }
    ++p;
    while (*p == ' ' || *p == '\t' || *p == '\r' || *p == '\n') {
        ++p;
    }
    if (*p == '"') {
        ++p;
        while (*p != '\0' && *p != '"' && n < cap - 1) {
            if (*p == '\\' && p[1] != '\0') {
                ++p;
            }
            out[n++] = *p++;
        }
    } else {
        while (*p != '\0' && *p != ',' && *p != '}' && *p != '\r' && *p != '\n' && n < cap - 1) {
            out[n++] = *p++;
        }
        while (n > 0 && out[n - 1] == ' ') {
            --n;
        }
    }
    out[n] = '\0';
    return 1;
}

static int gs_truthy(const char *v) {
    return _stricmp(v, "true") == 0 || _stricmp(v, "yes") == 0 || strcmp(v, "1") == 0;
}

static int gs_find(const char *id) {
    int i;
    for (i = 0; i < gs.n; ++i) {
        if (gs.s[i].used && _stricmp(gs.s[i].id, id) == 0) {
            return i;
        }
    }
    return -1;
}

static void gs_unload(int i) {
    GsScript *s = &gs.s[i];
    int k;
    if (!s->used) {
        return;
    }
    if (gs_get_hook(i, "on_unload")) {
        gs_pcall(i, 0, 0, "on_unload");
    }
    for (k = 0; k < GS_MAX_TASKS; ++k) {
        if (s->tasks[k] != LUA_NOREF) {
            luaL_unref(gs.L, LUA_REGISTRYINDEX, s->tasks[k]);
            s->tasks[k] = LUA_NOREF;
        }
    }
    for (k = 0; k < gs.ncmd; ++k) {
        if (gs.cmd[k].script == i) {
            luaL_unref(gs.L, LUA_REGISTRYINDEX, gs.cmd[k].fn_ref);
            gs.cmd[k] = gs.cmd[--gs.ncmd];
            --k;
        }
    }
    luaL_unref(gs.L, LUA_REGISTRYINDEX, s->env_ref);
    s->env_ref = LUA_NOREF;
    s->used = 0;
}

static int gs_alloc_script(void) {
    int i, k;
    for (i = 0; i < gs.n && gs.s[i].used; ++i) {
    }
    if (i == gs.n) {
        if (gs.n >= GS_MAX_SCRIPTS) {
            return -1;
        }
        gs.n++;
    }
    memset(&gs.s[i], 0, sizeof gs.s[i]);
    gs.s[i].env_ref = LUA_NOREF;
    for (k = 0; k < GS_MAX_TASKS; ++k) {
        gs.s[i].tasks[k] = LUA_NOREF;
    }
    return i;
}

/* A single-file script's manifest lives in its leading comments: "-- @gameplay: true". */
static int gs_header_field(const char *src, const char *key, char *out, size_t cap) {
    const char *p = src;
    size_t klen = strlen(key);
    while (*p == '-' && p[1] == '-') {
        const char *e = strchr(p, '\n'), *q = p + 2;
        while (*q == ' ' || *q == '\t') ++q;
        if (*q == '@' && _strnicmp(q + 1, key, klen) == 0 && q[1 + klen] == ':') {
            size_t n = 0;
            q += 2 + klen;
            while (*q == ' ' || *q == '\t') ++q;
            while (*q != '\0' && *q != '\r' && *q != '\n' && n < cap - 1) out[n++] = *q++;
            while (n > 0 && out[n - 1] == ' ') --n;
            out[n] = '\0';
            return 1;
        }
        if (e == NULL) break;
        p = e + 1;
        while (*p == '\r' || *p == '\n') ++p;
    }
    return 0;
}

static int gs_load_text(const char *id, const char *entry, char *src, size_t len,
                        const char *manifest, const char *origin);

/* Load (or reload) a script file. `manifest` is the mod.json text or NULL. Returns index or -1. */
static int gs_load_script(const char *id, const char *entry, const char *manifest, const char *origin) {
    size_t len = 0;
    char *src = gs_read_file(entry, &len);
    if (src == NULL) {
        gw_Console_Print(GS_RED, "[%s] cannot read %s", id, entry);
        return -1;
    }
    return gs_load_text(id, entry, src, len, manifest, origin);
}

/* The common path: `src` is malloc'd and freed here. */
static int gs_load_text(const char *id, const char *entry, char *src, size_t len,
                        const char *manifest, const char *origin) {
    char v[96], chunk[96];
    int i, old;
    GsScript *s;
    old = gs_find(id);
    if (old >= 0) {
        gs_unload(old);
    }
    i = gs_alloc_script();
    if (i < 0) {
        free(src);
        gw_Console_Print(GS_RED, "too many scripts (max %d)", GS_MAX_SCRIPTS);
        return -1;
    }
    s = &gs.s[i];
    s->used = 1;
    snprintf(s->id, sizeof s->id, "%s", id);
    snprintf(s->name, sizeof s->name, "%s", id);
    snprintf(s->entry, sizeof s->entry, "%s", entry);
    snprintf(s->origin, sizeof s->origin, "%s", origin);
    s->api_version = GW_SCRIPT_API_VERSION;
    if (manifest == NULL) { /* single file: "-- @key: value" header lines */
        if (gs_header_field(src, "name", v, sizeof v)) snprintf(s->name, sizeof s->name, "%s", v);
        if (gs_header_field(src, "version", v, sizeof v)) snprintf(s->version, sizeof s->version, "%s", v);
        if (gs_header_field(src, "author", v, sizeof v)) snprintf(s->author, sizeof s->author, "%s", v);
        if (gs_header_field(src, "api_version", v, sizeof v)) s->api_version = atoi(v);
        if (gs_header_field(src, "gameplay", v, sizeof v)) s->gameplay = gs_truthy(v);
        if (gs_header_field(src, "rollback_safe", v, sizeof v)) s->rollback_safe = gs_truthy(v);
    }
    if (manifest != NULL) {
        if (gs_json_field(manifest, "name", v, sizeof v)) snprintf(s->name, sizeof s->name, "%s", v);
        if (gs_json_field(manifest, "version", v, sizeof v)) snprintf(s->version, sizeof s->version, "%s", v);
        if (gs_json_field(manifest, "author", v, sizeof v)) snprintf(s->author, sizeof s->author, "%s", v);
        if (gs_json_field(manifest, "api_version", v, sizeof v)) s->api_version = atoi(v);
        if (gs_json_field(manifest, "gameplay", v, sizeof v)) s->gameplay = gs_truthy(v);
        if (gs_json_field(manifest, "rollback_safe", v, sizeof v)) s->rollback_safe = gs_truthy(v);
    }
    if (s->api_version > GW_SCRIPT_API_VERSION) {
        gw_Console_Print(GS_RED, "[%s] needs scripting API %d; this build has %d - not loaded", id,
                         s->api_version, GW_SCRIPT_API_VERSION);
        s->used = 0;
        free(src);
        return -1;
    }
    s->src_hash = gs_fnv(gs_fnv(14695981039346656037ull, id, strlen(id)), src, len);
    s->env_ref = gs_new_env(gs.L);
    snprintf(chunk, sizeof chunk, "@%s", id);
    if (luaL_loadbufferx(gs.L, src, len, chunk, "t") != LUA_OK) {
        gs_report(i, "load", lua_tostring(gs.L, -1));
        lua_pop(gs.L, 1);
        free(src);
        gs_unload(i);
        return -1;
    }
    free(src);
    lua_rawgeti(gs.L, LUA_REGISTRYINDEX, s->env_ref);
    lua_setupvalue(gs.L, -2, 1); /* _ENV */
    if (gs_pcall(i, 0, 0, "load") != 0) {
        gs_unload(i);
        return -1;
    }
    gw_Console_Print(GS_GREEN, "loaded %s%s%s (%s)%s", s->id, s->version[0] ? " " : "", s->version,
                     s->origin, s->gameplay ? " [gameplay]" : "");
    gw_log("script: loaded %s from %s%s", s->id, entry, s->gameplay ? " (gameplay)" : "");
    if (gs_get_hook(i, "on_scene")) { /* catch it up with the scene it arrived in */
        lua_pushinteger(gs.L, gs.scene_kind);
        lua_pushstring(gs.L, gw_SceneReport_SceneName(gs.scene_kind));
        gs_pcall(i, 2, 0, "on_scene");
    }
    return i;
}

/* A path relative to the scripts folder, or "examples/foo", or an absolute path. */
static const struct {
    const char *name, *manifest, *source;
} gs_builtins[] = {
#include "gw_script_builtins.inc"
    {NULL, NULL, NULL}};

static int gs_load_named(const char *name) {
    char path[MAX_PATH], id[64], manifest_path[MAX_PATH];
    const char *base;
    size_t n;
    DWORD attr;
    if (_strnicmp(name, "builtin:", 8) == 0) { /* compiled in: pc/scripts/examples */
        int k;
        for (k = 0; gs_builtins[k].name != NULL; ++k) {
            if (_stricmp(gs_builtins[k].name, name + 8) == 0) {
                size_t len = strlen(gs_builtins[k].source);
                char *src = (char *) malloc(len + 1);
                char entry[80];
                if (src == NULL) return -1;
                memcpy(src, gs_builtins[k].source, len + 1);
                snprintf(entry, sizeof entry, "builtin:%s", gs_builtins[k].name);
                return gs_load_text(gs_builtins[k].name, entry, src, len, gs_builtins[k].manifest,
                                    "builtin");
            }
        }
        gw_Console_Print(GS_RED, "no built-in script \"%s\"; the built-ins are:", name + 8);
        for (k = 0; gs_builtins[k].name != NULL; ++k) gw_Console_Print(GS_WHITE, "  builtin:%s", gs_builtins[k].name);
        return -1;
    }
    if (strchr(name, ':') != NULL || name[0] == '\\' || name[0] == '/') {
        snprintf(path, sizeof path, "%s", name);
    } else {
        snprintf(path, sizeof path, "%s\\%s", gs.scripts_dir, name);
    }
    attr = GetFileAttributesA(path);
    if (attr != INVALID_FILE_ATTRIBUTES && (attr & FILE_ATTRIBUTE_DIRECTORY)) {
        /* a folder with mod.json + main.lua (or the entry it names) */
        size_t mlen = 0;
        char *manifest;
        char entry[96] = "main.lua", full[MAX_PATH];
        int r;
        snprintf(manifest_path, sizeof manifest_path, "%s\\mod.json", path);
        manifest = gs_read_file(manifest_path, &mlen);
        if (manifest != NULL) {
            gs_json_field(manifest, "entry", entry, sizeof entry);
        }
        snprintf(full, sizeof full, "%s\\%s", path, entry);
        base = strrchr(path, '\\');
        snprintf(id, sizeof id, "%s", base != NULL ? base + 1 : path);
        if (manifest != NULL && gs_json_field(manifest, "id", manifest_path, sizeof manifest_path)) {
            snprintf(id, sizeof id, "%s", manifest_path);
        }
        r = gs_load_script(id, full, manifest, "scripts");
        free(manifest);
        return r;
    }
    n = strlen(path);
    if (n < 4 || _stricmp(path + n - 4, ".lua") != 0) {
        snprintf(path + n, sizeof path - n, ".lua");
    }
    base = strrchr(path, '\\');
    base = base != NULL ? base + 1 : path;
    snprintf(id, sizeof id, "%s", base);
    n = strlen(id);
    if (n > 4) {
        id[n - 4] = '\0';
    }
    return gs_load_script(id, path, NULL, "scripts");
}

static void gs_scan_scripts_dir(void) {
    char pattern[MAX_PATH];
    WIN32_FIND_DATAA fd;
    HANDLE h;
    snprintf(pattern, sizeof pattern, "%s\\*", gs.scripts_dir);
    h = FindFirstFileA(pattern, &fd);
    if (h == INVALID_HANDLE_VALUE) {
        return;
    }
    do {
        size_t n = strlen(fd.cFileName);
        if (fd.cFileName[0] == '.' || _stricmp(fd.cFileName, "examples") == 0) {
            continue; /* examples/ is loaded on request only */
        }
        if (fd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) {
            char mj[MAX_PATH];
            snprintf(mj, sizeof mj, "%s\\%s\\mod.json", gs.scripts_dir, fd.cFileName);
            if (GetFileAttributesA(mj) != INVALID_FILE_ATTRIBUTES) {
                gs_load_named(fd.cFileName);
            }
        } else if (n > 4 && _stricmp(fd.cFileName + n - 4, ".lua") == 0) {
            gs_load_named(fd.cFileName);
        }
    } while (FindNextFileA(h, &fd));
    FindClose(h);
}

/* Scripts shipped inside mods (docs/mods-packaging.md layout): <mods>/<id>/scripts/*.lua of every
 * mod gw_mods.c is mounting this boot - kind "script" mods carry nothing else, any other kind may
 * carry scripts beside its files/. The mod's mod.json is the scripts' manifest; each script's id
 * is "<mod id>/<file stem>". */
extern int gw_Mods_Count(void);
extern int gw_Mods_IsActive(int i);
extern const char *gw_Mods_Id(int i);
extern const char *gw_Mods_Dir(void);

static void gs_scan_mods(void) {
    const char *mods_dir = gw_Mods_Dir();
    int i, n = gw_Mods_Count();
    if (mods_dir == NULL || mods_dir[0] == '\0') {
        return;
    }
    for (i = 0; i < n; ++i) {
        char sp[MAX_PATH], mj[MAX_PATH];
        WIN32_FIND_DATAA sd;
        HANDLE sh;
        char *manifest;
        size_t mlen = 0;
        if (!gw_Mods_IsActive(i)) {
            continue;
        }
        snprintf(sp, sizeof sp, "%s\\%s\\scripts\\*.lua", mods_dir, gw_Mods_Id(i));
        sh = FindFirstFileA(sp, &sd);
        if (sh == INVALID_HANDLE_VALUE) {
            continue;
        }
        snprintf(mj, sizeof mj, "%s\\%s\\mod.json", mods_dir, gw_Mods_Id(i));
        manifest = gs_read_file(mj, &mlen);
        do {
            char id[64], entry[MAX_PATH];
            size_t k;
            snprintf(id, sizeof id, "%s/%s", gw_Mods_Id(i), sd.cFileName);
            k = strlen(id);
            if (k > 4) id[k - 4] = '\0';
            snprintf(entry, sizeof entry, "%s\\%s\\scripts\\%s", mods_dir, gw_Mods_Id(i), sd.cFileName);
            gs_load_script(id, entry, manifest, "mods");
        } while (FindNextFileA(sh, &sd));
        FindClose(sh);
        free(manifest);
    }
}

/* ============================================================================================
 * init
 * ============================================================================================ */
static void gs_socket_init(void);
static void gs_socket_poll(void);

static void gs_init(void) {
    const char *v;
    char *slash;
    if (gs.inited) {
        return;
    }
    gs.inited = 1;
    gs.cur = -1;
    gs.console = -1;
    gs.scene_kind = -1;
    GetModuleFileNameA(NULL, gs.exe_dir, sizeof gs.exe_dir);
    slash = strrchr(gs.exe_dir, '\\');
    if (slash != NULL) {
        *slash = '\0';
    }
    v = getenv("MELEE_SCRIPTS_DIR");
    if (v != NULL && v[0] != '\0') {
        snprintf(gs.scripts_dir, sizeof gs.scripts_dir, "%s", v);
    } else {
        snprintf(gs.scripts_dir, sizeof gs.scripts_dir, "%s\\scripts", gs.exe_dir);
    }
    snprintf(gs.data_dir, sizeof gs.data_dir, "%s\\scripts-data", gs.exe_dir);
    v = getenv("MELEE_SCRIPT_BUDGET");
    gs.budget_per_call = (v != NULL && atoi(v) > 0) ? atoi(v) : 2000000;
    v = getenv("MELEE_SCRIPT_MS");
    gs.ms_per_call = (v != NULL && atoi(v) > 0) ? atoi(v) : 50.0;

    gs.L = lua_newstate(gs_alloc, NULL);
    if (gs.L == NULL) {
        gw_log("script: could not create the Lua state");
        return;
    }
    lua_sethook(gs.L, gs_count_hook, LUA_MASKCOUNT, 1000);
    gs_build_base(gs.L);
    /* the console: a pseudo-script with its own persistent environment */
    gs.console = gs_alloc_script();
    snprintf(gs.s[gs.console].id, sizeof gs.s[gs.console].id, "console");
    snprintf(gs.s[gs.console].name, sizeof gs.s[gs.console].name, "console");
    snprintf(gs.s[gs.console].origin, sizeof gs.s[gs.console].origin, "console");
    gs.s[gs.console].used = 1;
    gs.s[gs.console].gameplay = 1;
    gs.s[gs.console].env_ref = gs_new_env(gs.L);
    gw_Console_Print(GS_GREY, "GD's Melee console - scripting API %d. \"help\" lists the commands.",
                     GW_SCRIPT_API_VERSION);
    gw_log("script: Lua 5.4 engine up (API %d), scripts from %s", GW_SCRIPT_API_VERSION, gs.scripts_dir);

    v = getenv("MELEE_SCRIPTS");
    if (v == NULL || v[0] != '0') {
        gs_scan_scripts_dir();
        gs_scan_mods();
    }
    v = getenv("MELEE_SCRIPT"); /* extra scripts, ';'-separated names or paths */
    if (v != NULL && v[0] != '\0') {
        char buf[1024], *p, *e;
        snprintf(buf, sizeof buf, "%s", v);
        for (p = buf; p != NULL && *p != '\0'; p = e) {
            e = strchr(p, ';');
            if (e != NULL) *e++ = '\0';
            if (*p != '\0') {
                int i = gs_load_named(p);
                if (i >= 0) snprintf(gs.s[i].origin, sizeof gs.s[i].origin, "env");
            }
        }
    }
    v = gw_script_pad_lua_path(); /* MELEE_PAD_SCRIPT=<file>.lua: an input script */
    if (v != NULL) {
        int i = gs_load_script("pad_script", v, "{\"gameplay\": true}", "env");
        (void) i;
    }
    gs_socket_init();
}

/* ============================================================================================
 * scene loop entry points
 * ============================================================================================ */
void gw_Script_SceneBegin(int scene_kind) {
    int prev;
    gs_init();
    if (gs.L == NULL) {
        return;
    }
    prev = gs.scene_kind;
    if (gs.match_active) {
        gs.match_active = 0;
        gs_hook_all("on_match_end", 0, 0, 0);
    }
    gs.scene_kind = scene_kind;
    gs.scene_epoch++;
    gs.paused = 0; /* a scene change always resumes */
    gs.step = 0;
    {
        int i;
        for (i = 0; i < gs.n; ++i) {
            if (!gs_get_hook(i, "on_scene")) continue;
            lua_pushinteger(gs.L, scene_kind);
            lua_pushstring(gs.L, gw_SceneReport_SceneName(scene_kind));
            lua_pushinteger(gs.L, prev);
            gs_pcall(i, 3, 0, "on_scene");
        }
    }
}

void gw_Script_Tick(void) {
    gs_init();
    if (gs.L == NULL) {
        return;
    }
    if (gs.console_open) {
        gw_TextEntryUntil = (int) GetTickCount() + 200; /* the keyboard is text, not a pad */
    }
    gs_poll_keys();
    gs_socket_poll();
    if (gs_pending_launch >= 0) {
        int mode = gs_pending_launch;
        gs_pending_launch = -1;
        gw_log("script: launching scene (game mode %d)", mode);
        gw_ScriptGame_LaunchScene(mode);
    }
    gs.ndraw = 0;
    gs_hook_all("on_tick", 0, 0, 0);
    gs_hook_all("on_draw", 0, 0, 0);
}

int gw_Script_Iterations(int count) {
    if (!gs.paused || gw_RB_Enabled() || gw_Netplay_Enabled()) {
        return count;
    }
    if (gs.step > 0) {
        gs.step--;
        return count > 0 ? 1 : 0;
    }
    return 0;
}

void gw_Script_FramePre(void) {
    if (gs.L == NULL) {
        return;
    }
    if ((gs.pending_save || gs.pending_load) && (gw_RB_Enabled() || gw_Netplay_Enabled())) {
        gw_Console_Print(GS_RED, "savestates are off during a netplay/rollback session");
        gs.pending_save = gs.pending_load = 0;
    }
    if (gs.pending_save) {
        int slot = gs.pending_save - 1;
        gs.pending_save = 0;
        if (!gs.snap_ready) {
            gs.snap_ready = gw_snap_open(GS_SAVE_SLOTS - 2) == 0;
        }
        if (gs.snap_ready) {
            gw_snap_save(slot);
            gs.slot[slot].used = 1;
            gs.slot[slot].scene_epoch = gs.scene_epoch;
            gs.slot[slot].frame = gs.frame;
            gs.slot[slot].match_frame = gs.match_frame;
            gw_Console_Print(GS_GREEN, "saved state %d (frame %d)", slot + 1, gs.frame);
            gs_hook_all("on_savestate", 1, slot + 1, 0);
        } else {
            gw_Console_Print(GS_RED, "savestates unavailable (snapshot memory could not be allocated)");
        }
    }
    if (gs.pending_load) {
        int slot = gs.pending_load - 1;
        gs.pending_load = 0;
        if (gs.slot[slot].used && gs.slot[slot].scene_epoch == gs.scene_epoch &&
            gw_snap_load(slot) == 0) {
            gs.match_frame = gs.slot[slot].match_frame;
            gw_Console_Print(GS_GREEN, "loaded state %d", slot + 1);
            gs_hook_all("on_loadstate", 1, slot + 1, 0);
        } else {
            gw_Console_Print(GS_RED, "could not load state %d", slot + 1);
        }
    }
    gs_hook_all("on_frame_pre", 0, 0, 0);
}

void gw_Script_FramePost(void) {
    int slot, any = 0;
    if (gs.L == NULL || gw_Snap_Resimulating()) {
        return;
    }
    gs.frame++;
    for (slot = 0; slot < 6; ++slot) {
        if (gs_players_present(slot)) {
            int a = gw_ScriptGame_FighterI(slot, SI_ACTION);
            any = 1;
            if (a != gs.last_action[slot]) {
                gs.last_action[slot] = a;
                gs.state_frame[slot] = 0;
            } else {
                gs.state_frame[slot]++;
            }
        } else {
            gs.last_action[slot] = -1;
            gs.state_frame[slot] = 0;
        }
    }
    if (any && !gs.match_active) {
        gs.match_active = 1;
        gs.match_frame = 0;
        gs_hook_all("on_match_start", 0, 0, 0);
    } else if (gs.match_active) {
        gs.match_frame++;
    }
    gs_hook_all("on_frame", 0, 0, 0);
    gs_run_tasks();
}

int gw_Script_DrawCount(void) { return gs.ndraw; }
const GwScriptDraw *gw_Script_DrawAt(int i) { return (i >= 0 && i < gs.ndraw) ? &gs.draw[i] : NULL; }

uint64_t gw_Script_GameplayHash(void) {
    uint64_t h = 0;
    int i;
    char *d = gs.describe;
    size_t left = sizeof gs.describe;
    gs.describe[0] = '\0';
    for (i = 0; i < gs.n; ++i) { /* order-independent: XOR of per-script digests */
        GsScript *s = &gs.s[i];
        if (s->used && s->gameplay && i != gs.console && !s->disabled) {
            uint64_t sh = gs_fnv(s->src_hash, s->version, strlen(s->version));
            int n;
            h ^= sh;
            n = snprintf(d, left, "%s%s@%s#%04x", d == gs.describe ? "" : ",", s->id, s->version,
                         (unsigned) (sh & 0xFFFF));
            if (n > 0 && (size_t) n < left) {
                d += n;
                left -= (size_t) n;
            }
        }
    }
    return h;
}

const char *gw_Script_GameplayDescribe(void) {
    gw_Script_GameplayHash();
    return gs.describe;
}

/* ============================================================================================
 * console commands
 * ============================================================================================ */
static void gs_print_value(lua_State *L, int idx, char *out, size_t cap, int depth) {
    size_t n = 0;
    idx = lua_absindex(L, idx);
    if (lua_istable(L, idx) && depth < 2) {
        int count = 0;
        n += (size_t) snprintf(out + n, cap - n, "{");
        lua_pushnil(L);
        while (lua_next(L, idx) != 0 && n < cap - 8) {
            char val[512];
            if (count++ > 0) {
                n += (size_t) snprintf(out + n, cap - n, ", ");
            }
            gs_print_value(L, -1, val, sizeof val, depth + 1);
            if (lua_type(L, -2) == LUA_TSTRING) {
                n += (size_t) snprintf(out + n, cap - n, "%s=%s", lua_tostring(L, -2), val);
            } else {
                lua_pushvalue(L, -2);
                n += (size_t) snprintf(out + n, cap - n, "[%s]=%s", luaL_tolstring(L, -1, NULL), val);
                lua_pop(L, 2);
            }
            lua_pop(L, 1);
            if (n >= cap) n = cap - 1;
        }
        if (n < cap - 2) {
            snprintf(out + n, cap - n, "}");
        }
        return;
    }
    if (lua_type(L, idx) == LUA_TNUMBER && !lua_isinteger(L, idx)) {
        snprintf(out, cap, "%.4g", lua_tonumber(L, idx));
        return;
    }
    snprintf(out, cap, "%s", luaL_tolstring(L, idx, NULL));
    lua_pop(L, 1);
}

static int gs_exec_lua(const char *code) {
    lua_State *L = gs.L;
    char buf[2048];
    int top = lua_gettop(L), rc, i;
    /* an expression first ("return <code>"), then a statement */
    snprintf(buf, sizeof buf, "return %s", code);
    rc = luaL_loadbufferx(L, buf, strlen(buf), "=console", "t");
    if (rc != LUA_OK) {
        lua_pop(L, 1);
        rc = luaL_loadbufferx(L, code, strlen(code), "=console", "t");
    }
    if (rc != LUA_OK) {
        gw_Console_Print(GS_RED, "%s", lua_tostring(L, -1));
        lua_pop(L, 1);
        return -1;
    }
    lua_rawgeti(L, LUA_REGISTRYINDEX, gs.s[gs.console].env_ref);
    lua_setupvalue(L, -2, 1);
    if (gs_pcall(gs.console, 0, LUA_MULTRET, "console") != 0) {
        lua_settop(L, top);
        return -1;
    }
    for (i = top + 1; i <= lua_gettop(L); ++i) {
        char val[1800];
        gs_print_value(L, i, val, sizeof val, 0);
        gw_Console_Print(GS_WHITE, "%s", val);
    }
    lua_settop(L, top);
    return 0;
}

static void gs_cmd_state(void) {
    int slot, any = 0;
    int mode = gw_ScriptGame_GameMode();
    gw_Console_Print(GS_WHITE, "frame %d  scene %s(%d)  mode %s(%d)  match %s%s", gs.frame,
                     gw_SceneReport_SceneName(gs.scene_kind), gs.scene_kind,
                     gw_SceneReport_ModeName(mode), mode, gs.match_active ? "live" : "none",
                     gs.paused ? "  PAUSED" : "");
    for (slot = 0; slot < 6; ++slot) {
        if (!gs_players_present(slot)) continue;
        any = 1;
        gw_Console_Print(GS_WHITE,
                         "P%d %-16s x=%8.2f y=%8.2f  %5.1f%%  stocks %d  action %3d (frame %d)%s",
                         slot + 1, gs_char_name(gw_ScriptGame_FighterI(slot, SI_CHAR)),
                         gw_ScriptGame_FighterF(slot, SF_X), gw_ScriptGame_FighterF(slot, SF_Y),
                         gw_ScriptGame_FighterF(slot, SF_PERCENT),
                         gw_ScriptGame_FighterI(slot, SI_STOCKS),
                         gw_ScriptGame_FighterI(slot, SI_ACTION), gs.state_frame[slot],
                         gw_ScriptGame_FighterI(slot, SI_AIRBORNE) ? " air" : "");
    }
    if (!any) gw_Console_Print(GS_GREY, "(no fighters)");
}

static void gs_cmd_help(void) {
    int i;
    gw_Console_Print(GS_YELLOW, "commands (anything else runs as Lua; \"= expr\" prints a value):");
    gw_Console_Print(GS_WHITE, "  help | scripts | load <name> | unload <id> | reload [id]");
    gw_Console_Print(GS_WHITE, "  state | frame | pause | resume | step [n]");
    gw_Console_Print(GS_WHITE, "  savestate [1-4] | loadstate [1-4]");
    gw_Console_Print(GS_WHITE, "  scene <MELEE_SCENE text> | scene clear");
    gw_Console_Print(GS_WHITE, "  input <port> <buttons> [frames] [x y]   e.g. input 1 A+B 10");
    gw_Console_Print(GS_WHITE, "  shot [path] | label <text> | echo <text> | clear | api | quit");
    for (i = 0; i < gs.ncmd; ++i) {
        gw_Console_Print(GS_WHITE, "  %s  %s  [%s]", gs.cmd[i].name, gs.cmd[i].help,
                         gs_script_id(gs.cmd[i].script));
    }
}

static int gs_exec(const char *line_in) {
    char line[1024], *arg;
    int i;
    lua_State *L = gs.L;
    while (*line_in == ' ' || *line_in == '\t') ++line_in;
    snprintf(line, sizeof line, "%s", line_in);
    {
        size_t n = strlen(line);
        while (n > 0 && (line[n - 1] == '\r' || line[n - 1] == '\n' || line[n - 1] == ' ')) line[--n] = '\0';
    }
    if (line[0] == '\0') {
        return 0;
    }
    if (line[0] == '=') {
        return gs_exec_lua(line + 1);
    }
    arg = strchr(line, ' ');
    if (arg != NULL) {
        *arg++ = '\0';
        while (*arg == ' ') ++arg;
    } else {
        arg = line + strlen(line);
    }
#define IS(c) (_stricmp(line, c) == 0)
    if (IS("help") || IS("?")) {
        gs_cmd_help();
        return 0;
    }
    if (IS("api")) {
        gw_Console_Print(GS_WHITE, "scripting API %d (docs/scripting.md), Lua %s", GW_SCRIPT_API_VERSION,
                         LUA_RELEASE);
        return 0;
    }
    if (IS("shot") || IS("screenshot")) { /* any path from the console/socket (it is the user) */
        char path[MAX_PATH];
        if (arg[0] != '\0') {
            snprintf(path, sizeof path, "%s", arg);
        } else {
            snprintf(path, sizeof path, "%s\\shot-%d.png", gs.exe_dir, gs.frame);
        }
        return gs_screenshot(path) == 0 ? 0 : -1;
    }
    if (IS("quit")) {
        gw_log("console: quit");
        EnumThreadWindows(GetCurrentThreadId(), gs_close_cb, 0);
        return 0;
    }
    if (IS("label")) { /* the run label (top-left caption and window title) */
        gw_Overlay_SetRunLabel(arg);
        gw_Console_Print(GS_GREEN, arg[0] != '\0' ? "label: %s" : "label cleared", arg);
        return 0;
    }
    if (IS("echo")) {
        gw_Console_Print(GS_WHITE, "%s", arg);
        return 0;
    }
    if (IS("clear")) {
        gs_line_count = 0;
        return 0;
    }
    if (IS("scripts")) {
        for (i = 0; i < gs.n; ++i) {
            GsScript *s = &gs.s[i];
            if (!s->used || i == gs.console) continue;
            gw_Console_Print(s->disabled ? GS_RED : GS_WHITE, "  %-24s %-8s %-8s %s%s%s", s->id,
                             s->version[0] ? s->version : "-", s->origin,
                             s->gameplay ? "gameplay " : "", s->disabled ? "OFF " : "", s->entry);
        }
        return 0;
    }
    if (IS("load")) {
        return gs_load_named(arg) >= 0 ? 0 : -1;
    }
    if (IS("unload")) {
        i = gs_find(arg);
        if (i < 0 || i == gs.console) {
            gw_Console_Print(GS_RED, "no script \"%s\"", arg);
            return -1;
        }
        gs_unload(i);
        gw_Console_Print(GS_GREEN, "unloaded %s", arg);
        return 0;
    }
    if (IS("reload")) {
        int rc = 0;
        for (i = 0; i < gs.n; ++i) {
            GsScript s;
            if (!gs.s[i].used || i == gs.console) continue;
            if (arg[0] != '\0' && _stricmp(gs.s[i].id, arg) != 0) continue;
            s = gs.s[i];
            if (s.origin[0] == 'm' || s.origin[0] == 'e') { /* keep the manifest flags */
                char m[128];
                snprintf(m, sizeof m, "{\"gameplay\": %s, \"rollback_safe\": %s}",
                         s.gameplay ? "true" : "false", s.rollback_safe ? "true" : "false");
                if (gs_load_script(s.id, s.entry, m, s.origin) < 0) rc = -1;
            } else if (gs_load_named(s.entry) < 0) {
                rc = -1;
            }
        }
        return rc;
    }
    if (IS("state")) {
        gs_cmd_state();
        return 0;
    }
    if (IS("frame")) {
        gw_Console_Print(GS_WHITE, "%d", gs.frame);
        return 0;
    }
    if (IS("pause") || IS("resume") || IS("step") || IS("savestate") || IS("loadstate")) {
        char code[64];
        snprintf(code, sizeof code, "gd.%s(%s)", line, arg[0] != '\0' ? arg : "");
        return gs_exec_lua(code);
    }
    if (IS("scene")) {
        if (_stricmp(arg, "clear") == 0) {
            gw_SceneLaunch_SetText(NULL);
            gw_Console_Print(GS_GREEN, "scene launch cleared");
            return 0;
        }
        lua_rawgeti(L, LUA_REGISTRYINDEX, gs.s[gs.console].env_ref);
        lua_getfield(L, -1, "gd");
        lua_getfield(L, -1, "scene_launch");
        lua_remove(L, -2);
        lua_remove(L, -2);
        lua_pushstring(L, arg);
        if (gs_pcall(gs.console, 1, 1, "scene") != 0) return -1;
        gw_Console_Print(GS_GREEN, "launching \"%s\"", lua_tostring(L, -1));
        lua_pop(L, 1);
        return 0;
    }
    if (IS("input")) {
        int port = 0, frames = 1, x = 0, y = 0;
        char buttons[64] = "";
        int n = sscanf(arg, "%d %63s %d %d %d", &port, buttons, &frames, &x, &y);
        char code[256];
        if (n < 2) {
            gw_Console_Print(GS_RED, "input <port> <buttons|0xhex|none> [frames] [x y]");
            return -1;
        }
        if (_stricmp(buttons, "none") == 0) buttons[0] = '\0';
        if (buttons[0] == '0' && (buttons[1] == 'x' || buttons[1] == 'X')) {
            snprintf(code, sizeof code, "gd.input(%d, {buttons=%s, x=%d, y=%d}, %d)", port, buttons, x, y, frames);
        } else {
            snprintf(code, sizeof code, "gd.input(%d, {buttons=\"%s\", x=%d, y=%d}, %d)", port, buttons, x, y, frames);
        }
        return gs_exec_lua(code);
    }
    for (i = 0; i < gs.ncmd; ++i) { /* commands scripts registered */
        if (_stricmp(gs.cmd[i].name, line) == 0) {
            lua_rawgeti(L, LUA_REGISTRYINDEX, gs.cmd[i].fn_ref);
            lua_pushstring(L, arg);
            return gs_pcall(gs.cmd[i].script, 1, 0, gs.cmd[i].name);
        }
    }
#undef IS
    return gs_exec_lua(line_in);
}

int gw_Script_Exec(const char *line, char *out, int cap) {
    int rc;
    gs_init();
    if (gs.L == NULL) {
        if (out != NULL && cap > 0) snprintf(out, (size_t) cap, "scripting unavailable\n");
        return -1;
    }
    gw_Console_Print(GS_GREY, "> %s", line);
    if (out != NULL && cap > 0) {
        gs_capture = out;
        gs_capture_cap = cap;
        gs_capture_len = 0;
        out[0] = '\0';
    }
    rc = gs_exec(line);
    gs_capture = NULL;
    return rc;
}

/* ============================================================================================
 * the local console socket: MELEE_CONSOLE_PORT=<port> (off by default), 127.0.0.1 only.
 * One command per line; the reply is the command's output lines, then ">>> ok" or ">>> error".
 * ============================================================================================ */
#define GS_CLIENTS 4
static SOCKET gs_listen = INVALID_SOCKET;
static struct {
    SOCKET s;
    char in[4096];
    int len;
} gs_client[GS_CLIENTS];

static void gs_socket_init(void) {
    const char *v = getenv("MELEE_CONSOLE_PORT");
    struct sockaddr_in a;
    u_long nb = 1;
    WSADATA wd;
    int port, i;
    for (i = 0; i < GS_CLIENTS; ++i) gs_client[i].s = INVALID_SOCKET;
    if (v == NULL || (port = atoi(v)) <= 0 || port > 65535) {
        return;
    }
    WSAStartup(MAKEWORD(2, 2), &wd);
    gs_listen = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    if (gs_listen == INVALID_SOCKET) return;
    memset(&a, 0, sizeof a);
    a.sin_family = AF_INET;
    a.sin_port = htons((u_short) port);
    a.sin_addr.s_addr = htonl(INADDR_LOOPBACK); /* never reachable from another machine */
    if (bind(gs_listen, (struct sockaddr *) &a, sizeof a) != 0 || listen(gs_listen, 4) != 0) {
        gw_log("script: console socket: cannot listen on 127.0.0.1:%d", port);
        closesocket(gs_listen);
        gs_listen = INVALID_SOCKET;
        return;
    }
    ioctlsocket(gs_listen, FIONBIO, &nb);
    gw_log("script: console socket listening on 127.0.0.1:%d", port);
    gw_Console_Print(GS_GREY, "console socket: 127.0.0.1:%d", port);
}

static void gs_send_all(SOCKET s, const char *p, int n) {
    while (n > 0) {
        int k = send(s, p, n, 0);
        if (k <= 0) return;
        p += k;
        n -= k;
    }
}

static void gs_socket_poll(void) {
    int i;
    if (gs_listen == INVALID_SOCKET) return;
    for (;;) {
        SOCKET c = accept(gs_listen, NULL, NULL);
        u_long nb = 1;
        if (c == INVALID_SOCKET) break;
        for (i = 0; i < GS_CLIENTS && gs_client[i].s != INVALID_SOCKET; ++i) {
        }
        if (i == GS_CLIENTS) {
            closesocket(c);
            continue;
        }
        ioctlsocket(c, FIONBIO, &nb);
        gs_client[i].s = c;
        gs_client[i].len = 0;
        {
            static const char banner[] = "GD's Melee console (scripting API 1)\n";
            gs_send_all(c, banner, (int) sizeof banner - 1);
        }
    }
    for (i = 0; i < GS_CLIENTS; ++i) {
        char *nl;
        int k;
        if (gs_client[i].s == INVALID_SOCKET) continue;
        k = recv(gs_client[i].s, gs_client[i].in + gs_client[i].len,
                 (int) sizeof gs_client[i].in - 1 - gs_client[i].len, 0);
        if (k == 0 || (k < 0 && WSAGetLastError() != WSAEWOULDBLOCK)) {
            closesocket(gs_client[i].s);
            gs_client[i].s = INVALID_SOCKET;
            continue;
        }
        if (k > 0) gs_client[i].len += k;
        gs_client[i].in[gs_client[i].len] = '\0';
        /* one command per tick per client keeps the frame bounded and lets "step" land between */
        if ((nl = strchr(gs_client[i].in, '\n')) != NULL) {
            static char out[16384];
            int rc;
            *nl = '\0';
            rc = gw_Script_Exec(gs_client[i].in, out, (int) sizeof out);
            gs_send_all(gs_client[i].s, out, (int) strlen(out));
            gs_send_all(gs_client[i].s, rc == 0 ? ">>> ok\n" : ">>> error\n", rc == 0 ? 7 : 10);
            k = (int) (nl + 1 - gs_client[i].in);
            memmove(gs_client[i].in, nl + 1, (size_t) (gs_client[i].len - k + 1));
            gs_client[i].len -= k;
        } else if (gs_client[i].len >= (int) sizeof gs_client[i].in - 1) {
            gs_client[i].len = 0; /* an over-long line is dropped */
        }
    }
}

/* ============================================================================================
 * tests (headless: no match, no ImGui)
 * ============================================================================================ */
#include "gw_test.h"

static int t_exec(const char *line, char *out, int cap) { return gw_Script_Exec(line, out, cap); }

static int test_script_lua_runs(void) {
    char out[512];
    if (t_exec("= 6 * 7", out, sizeof out) != 0 || strstr(out, "42") == NULL) {
        gw_test_fail("console \"= 6 * 7\" gave \"%s\"", out);
        return 1;
    }
    if (t_exec("x = 5", out, sizeof out) != 0 || t_exec("= x + 1", out, sizeof out) != 0 ||
        strstr(out, "6") == NULL) {
        gw_test_fail("console globals do not persist: \"%s\"", out);
        return 1;
    }
    if (t_exec("= gd.api_version", out, sizeof out) != 0 || strstr(out, "1") == NULL) {
        gw_test_fail("gd.api_version: \"%s\"", out);
        return 1;
    }
    return 0;
}

static int test_script_sandbox(void) {
    static const char *const banned[] = {"io", "os", "package", "debug", "require", "dofile",
                                         "loadfile", "string.dump"};
    char out[512], line[64];
    size_t i;
    for (i = 0; i < sizeof banned / sizeof banned[0]; ++i) {
        snprintf(line, sizeof line, "= %s == nil", banned[i]);
        if (t_exec(line, out, sizeof out) != 0 || strstr(out, "true") == NULL) {
            gw_test_fail("sandbox exposes %s (%s)", banned[i], out);
            return 1;
        }
    }
    /* bytecode is refused even when handed over directly */
    if (t_exec("= load(\"\\27Lua\")", out, sizeof out) != 0 || strstr(out, "nil") == NULL) {
        gw_test_fail("load accepted a binary chunk: %s", out);
        return 1;
    }
    if (t_exec("= getmetatable('')", out, sizeof out) != 0 || strstr(out, "locked") == NULL) {
        gw_test_fail("the string metatable is reachable: %s", out);
        return 1;
    }
    return 0;
}

static int test_script_budget(void) {
    char out[512];
    int rc = t_exec("while true do end", out, sizeof out);
    if (rc == 0 || strstr(out, "ran too long") == NULL) {
        gw_test_fail("an endless loop was not stopped: rc=%d \"%s\"", rc, out);
        return 1;
    }
    /* the engine is still usable afterwards */
    if (t_exec("= 1 + 1", out, sizeof out) != 0 || strstr(out, "2") == NULL) {
        gw_test_fail("console dead after a budget stop: \"%s\"", out);
        return 1;
    }
    return 0;
}

static int test_script_isolation_and_errors(void) {
    char dir[MAX_PATH], path[MAX_PATH], out[1024];
    FILE *f;
    int a, b, rc = 0;
    snprintf(dir, sizeof dir, "%s\\gw_script_test", gs.exe_dir);
    CreateDirectoryA(dir, NULL);
    snprintf(path, sizeof path, "%s\\iso_a.lua", dir);
    f = fopen(path, "w");
    fputs("shared = 'a'\nstring.upper = nil\nfunction on_tick() hits = (hits or 0) + 1 end\n"
          "gd.command('iso_a_hits', function() gd.log('hits', hits) end, 'test')\n", f);
    fclose(f);
    snprintf(path, sizeof path, "%s\\iso_b.lua", dir);
    f = fopen(path, "w");
    fputs("function on_tick() error('boom') end\n", f);
    fclose(f);
    snprintf(path, sizeof path, "%s\\iso_a.lua", dir);
    a = gs_load_script("iso_a", path, NULL, "test");
    snprintf(path, sizeof path, "%s\\iso_b.lua", dir);
    b = gs_load_script("iso_b", path, NULL, "test");
    if (a < 0 || b < 0) {
        gw_test_fail("test scripts did not load");
        return 1;
    }
    /* a's globals and library edits are invisible to b and the console */
    t_exec("= shared == nil and string.upper ~= nil", out, sizeof out);
    if (strstr(out, "true") == NULL) {
        gw_test_fail("a script's globals/library edits leaked: %s", out);
        rc = 1;
    }
    {
        int k;
        for (k = 0; k < GS_MAX_ERRORS + 2; ++k) gw_Script_Tick();
    }
    if (!gs.s[b].disabled) {
        gw_test_fail("a script failing every tick was not switched off");
        rc = 1;
    }
    if (gs.s[a].disabled) {
        gw_test_fail("a healthy script was switched off by another's errors");
        rc = 1;
    }
    t_exec("iso_a_hits", out, sizeof out);
    if (strstr(out, "hits") == NULL) {
        gw_test_fail("a script-registered command did not run: %s", out);
        rc = 1;
    }
    /* gameplay writes are refused for a script without "gameplay": true */
    {
        int prev = gs.cur;
        lua_rawgeti(gs.L, LUA_REGISTRYINDEX, gs.s[a].env_ref);
        lua_getfield(gs.L, -1, "gd");
        lua_getfield(gs.L, -1, "set_percent");
        lua_remove(gs.L, -2);
        lua_remove(gs.L, -2);
        lua_pushinteger(gs.L, 1);
        lua_pushinteger(gs.L, 50);
        gs.cur = a;
        if (gs_pcall(a, 2, 0, "test") == 0) {
            gw_test_fail("gd.set_percent ran for a non-gameplay script");
            rc = 1;
        }
        gs.cur = prev;
    }
    gs_unload(a);
    gs_unload(b);
    snprintf(path, sizeof path, "%s\\iso_a.lua", dir);
    DeleteFileA(path);
    snprintf(path, sizeof path, "%s\\iso_b.lua", dir);
    DeleteFileA(path);
    RemoveDirectoryA(dir);
    return rc;
}

static int test_script_manifest_and_hash(void) {
    char dir[MAX_PATH], path[MAX_PATH];
    FILE *f;
    int i, rc = 0;
    uint64_t h0 = gw_Script_GameplayHash(), h1, h2;
    snprintf(dir, sizeof dir, "%s\\gw_script_test2", gs.exe_dir);
    CreateDirectoryA(dir, NULL);
    snprintf(path, sizeof path, "%s\\gp.lua", dir);
    f = fopen(path, "w");
    fputs("function on_frame() end\n", f);
    fclose(f);
    i = gs_load_script("gp_test", path,
                       "{ \"name\": \"GP\", \"version\": \"1.2\", \"api_version\": 1,\n"
                       "  \"gameplay\": true, \"rollback_safe\": \"yes\" }", "test");
    if (i < 0 || !gs.s[i].gameplay || !gs.s[i].rollback_safe || strcmp(gs.s[i].version, "1.2") != 0) {
        gw_test_fail("mod.json fields not read");
        rc = 1;
    }
    h1 = gw_Script_GameplayHash();
    if (h1 == h0 || strstr(gw_Script_GameplayDescribe(), "gp_test@1.2") == NULL) {
        gw_test_fail("gameplay script not in the must-match hash (%s)", gw_Script_GameplayDescribe());
        rc = 1;
    }
    if (i >= 0) gs_unload(i);
    h2 = gw_Script_GameplayHash();
    if (h2 != h0) {
        gw_test_fail("hash did not return to its value after unloading");
        rc = 1;
    }
    /* the built-ins: a single-file header manifest and a mod.json one */
    i = gs_load_named("builtin:state_overlay");
    if (i < 0 || strcmp(gs.s[i].name, "Frame-data overlay") != 0 || gs.s[i].gameplay) {
        gw_test_fail("builtin:state_overlay did not load with its header manifest");
        rc = 1;
    }
    if (i >= 0) gs_unload(i);
    i = gs_load_named("builtin:tm_lite");
    if (i < 0 || !gs.s[i].gameplay || strcmp(gs.s[i].version, "1.0.0") != 0) {
        gw_test_fail("builtin:tm_lite did not load with its mod.json");
        rc = 1;
    }
    if (i >= 0) gs_unload(i);
    /* a script asking for a newer API is refused */
    i = gs_load_script("future", path, "{\"api_version\": 99}", "test");
    if (i >= 0) {
        gw_test_fail("a script needing API 99 was loaded");
        gs_unload(i);
        rc = 1;
    }
    DeleteFileA(path);
    RemoveDirectoryA(dir);
    return rc;
}

static int test_script_input_task(void) {
    /* gd.run + gd.press: the input reaches the pad override, then the task finishes */
    char out[512];
    unsigned b = 0;
    int x, y, cx, cy, l, r, k;
    if (t_exec("gd.run(function() gd.press(1, 'A+B', 3) done = true end)", out, sizeof out) != 0) {
        gw_test_fail("gd.run failed: %s", out);
        return 1;
    }
    gw_Script_FramePost(); /* first resume: gd.input + yield */
    {
        unsigned char st[4 * 32]; /* PADStatus[4] (16 bytes each under TARGET_PC) */
        memset(st, 0, sizeof st);
        gw_Script_PadApply(st);
        gw_script_pad_state(0, &b, &x, &y, &cx, &cy, &l, &r);
    }
    if ((b & 0x0300) != 0x0300) {
        gw_test_fail("gd.press(1, 'A+B') did not reach port 1 (buttons %04X)", b);
        return 1;
    }
    for (k = 0; k < 5; ++k) gw_Script_FramePost();
    if (t_exec("= done", out, sizeof out) != 0 || strstr(out, "true") == NULL) {
        gw_test_fail("the input task did not finish: %s", out);
        return 1;
    }
    return 0;
}

void gw_script_tests_register(void) {
    gw_test_register("script_lua_runs", test_script_lua_runs);
    gw_test_register("script_sandbox", test_script_sandbox);
    gw_test_register("script_budget", test_script_budget);
    gw_test_register("script_isolation_and_errors", test_script_isolation_and_errors);
    gw_test_register("script_manifest_and_hash", test_script_manifest_and_hash);
    gw_test_register("script_input_task", test_script_input_task);
}
