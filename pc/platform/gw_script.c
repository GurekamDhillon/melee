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
#include "gw_kit.h"
#include "../geno/geno.h" /* GENO_VERSION, for the state library header */

#include <math.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
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

/* ---- the Geno Lab's inspection half (script_game.c; field numbers in script_lab.h) ------------ */
#include "../gameworld/script_lab.h"
extern float gw_ScriptGame_LabF(int slot, int field);
extern int gw_ScriptGame_LabI(int slot, int field);
extern const char *gw_ScriptGame_LabAnimSymbol(int slot);
extern int gw_ScriptGame_HitI(int slot, int i, int field);
extern float gw_ScriptGame_HitF(int slot, int i, int field);
extern int gw_ScriptGame_HurtI(int slot, int i, int field);
extern float gw_ScriptGame_HurtF(int slot, int i, int field);
extern float gw_ScriptGame_JointF(int slot, int i, int comp);
extern int gw_ScriptGame_JointParent(int slot, int i);
extern int gw_ScriptGame_LabDObjI(int slot, int d, int field);
extern float gw_ScriptGame_LabTObjF(int slot, int d, int t, int field);
extern float gw_ScriptGame_CameraF(int field);
extern int gw_ScriptGame_LabDebugDraw(int slot, int set, int value);
extern int gw_ScriptGame_LabStageDraw(int mask, int value);
extern int gw_ScriptGame_LabAttrCount(void);
extern const char *gw_ScriptGame_LabAttrName(int i);
extern float gw_ScriptGame_LabAttrF(int slot, int i);
#include "gw_motion_names.inc"

/* ---- the rest of the port --------------------------------------------------------------------- */
extern void gw_SceneLaunch_SetText(const char *text);
extern int gw_SceneLaunch_BootGameMode(void);
extern const char *gw_SceneReport_ModeName(int mode);
extern const char *gw_SceneReport_SceneName(int scene_kind);
extern int gw_Snap_Resimulating(void);
extern int gw_RB_Enabled(void);
extern int gw_Netplay_Enabled(void);
extern int gw_snap_reserve(int n);
extern void gw_snap_save_index(int idx, int tag);
extern int gw_snap_load_index(int idx);
extern uint32_t gw_snap_slot_bytes(void);
/* gw_snap.c: the Lab's long rewind (delta keyframes) and state files */
extern int gw_rw_begin(int tag, const void *user, int ulen);
extern int gw_rw_save(int tag, const void *user, int ulen);
extern int gw_rw_load(int tag, void *user_out, int ulen);
extern int gw_rw_drop_from(int tag);
extern void gw_rw_trim(int limit);
extern void gw_rw_end(void);
extern int gw_rw_find_le(int tag);
extern int gw_rw_base_tag(void);
extern int gw_rw_latest(void);
extern int gw_rw_count(void);
extern double gw_rw_bytes(void);
extern double gw_rw_delta_bytes(void);
extern double gw_rw_ms_last_load(void);
extern double gw_rw_ms_save_avg(void);
extern uint32_t gw_rw_last_load_pages(void);
extern int gw_rw_capture(void);
extern int gw_rw_compare(double out[2], uint64_t out_h[2]);
extern void gw_rw_measure(int on);
extern void gw_Snap_LabResim(int on);
extern int gw_snap_file_save(const char *path, const void *hdr, uint32_t hdr_len);
extern int gw_snap_file_header(const char *path, void *hdr, uint32_t cap);
extern int gw_snap_file_rewrite_header(const char *path, const void *hdr, uint32_t hdr_len);
extern int gw_snap_file_load(const char *path);
/* Geno hot reload (geno_registry.c, and the game half in pc/geno/geno_game.c) */
extern int gw_Geno_Reload(char *msg, int cap);
extern int gw_GenoGame_LabReload(void);
extern int gw_Geno_ProfileCount(void);
extern const char *gw_Geno_ProfileHex(int p);
extern uint64_t gw_net_hash_file(const char *path, size_t max_bytes);
extern const char *gw_Mods_Describe(void);
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
    char ui_dir[MAX_PATH]; /* the script's (or its mod's) ui/ folder for gd.kit art, "" = none */
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
    int last_action[6], state_frame[6]; /* gd.player().action_frame bookkeeping */
} GsSaveSlot;

/* The Geno Lab's long rewind (gd.history / gd.step_back / gd.rewind_to): delta keyframes in
 * gw_snap.c (gw_rw_*) plus the per-frame input log below. A frame's `tag` is the match frame about
 * to run (gs_ring_now). docs/geno.md 14.10. */
#define GS_RW_MAX_FRAMES (60 * 60) /* the longest window: a minute */
#define GS_RW_INTERVAL 30          /* a keyframe every half second by default */
#define GS_LOG_N 4096              /* the input log ring (> the longest window + an interval) */
#define GS_LOG_SFX 16
#define GS_LOG_Q 48
typedef struct {
    int tag;
    unsigned char have;     /* the frame renewed the pads (else it kept the previous statuses) */
    unsigned char sfx_open; /* the sound handles are still being recorded (the frame's first run) */
    unsigned char nsfx;
    unsigned char nq;       /* the voice pool's answers to the game (axdriver.c LAB_AUDIO) */
    unsigned char pad[48];  /* PADStatus[4] as the game read them */
    int sfx_id[GS_LOG_SFX], sfx_h[GS_LOG_SFX];
    unsigned char q_kind[GS_LOG_Q];
    int q_arg[GS_LOG_Q], q_val[GS_LOG_Q];
} GsLogEntry;
static GsLogEntry gs_log[GS_LOG_N];
static unsigned gs_sfx_claimed; /* the replayed frame's handles already given out */
static int gs_q_pos;            /* the replayed frame's next logged audio answer */
static int gs_in_frame;         /* between FramePre and FramePost: a logic frame is running */
/* sounds a replay at speed really started: the logged handle the game keeps -> the real voice */
#define GS_SFX_MAP 128
static int gs_sfx_map_from[GS_SFX_MAP], gs_sfx_map_to[GS_SFX_MAP], gs_sfx_map_next;

/* events the engine reports mid-frame (Script_GameEvent), dispatched after the frame */
#define GS_MAX_EVENTS 256
typedef struct {
    int what, a, b, c, d;
    int hit[LAB_HI_COUNT]; /* LAB_EV_HIT: the attacker's hitbox as it was when it connected */
    float hitf[LAB_HF_COUNT];
    int has_hit;
} GsEvent;

#define GS_MAX_DRAW 2048

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
    /* the long rewind (Geno Lab) */
    int rw_frames;     /* the window, 0 = off */
    int rw_interval;   /* a keyframe every N frames */
    int rw_began;      /* the keyframe store holds this match's timeline */
    int rw_head;       /* the first frame not in the input log: the live edge */
    int rw_log_start;  /* the first frame in the log */
    int rw_force_key;  /* a write changed the game: keyframe at the next boundary */
    int rw_pending_on, rw_pending_to; /* a rewind to that frame at the next boundary */
    int rw_resim;      /* frames the next tick re-simulates */
    int rw_resim_run;  /* re-simulated iterations left in this tick */
    int rw_notify;     /* on_loadstate(0) once the re-simulation is done */
    double rw_t0, rw_last_ms, rw_last_load_ms;
    int rw_last_frames;
    /* the exactness self-test (gd.rewind_test) */
    int rwt_phase, rwt_tag, rwt_back, rwt_pass, rwt_keep, rwt_arm;
    double rwt_diff[2];
    char rwt_text[320];
    /* hot reload (gd.hot_reload) */
    int hot_phase, hot_start, hot_ok;
    char hot_text[512];
    /* the persistent state library: a load / save waiting for a frame boundary */
    char st_pending_load[MAX_PATH];
    char st_pending_save[MAX_PATH];
    char st_save_name[64], st_save_desc[96];
    /* engine events (Geno Lab) */
    GsEvent ev[GS_MAX_EVENTS];
    int nev, ev_dropped;
    int want_events; /* some script defines one of the event hooks */
    int in_event;    /* dispatching an event hook now */
    /* draw lists: scripts build `draw[build]`, the overlay shows `draw[!build]` (swapped when a
       frame's list is complete, so a render never shows a half-built or cleared list) */
    GwScriptDraw draw[2][GS_MAX_DRAW];
    int ndraw[2];
    int build;
    /* the match camera, fetched once per draw pass (gd.project) */
    int cam_stamp, cam_fetched, cam_have;
    float cam[LAB_CAM_COUNT];
    int stage_zones; /* LAB_STAGE_ZONES has no getter: remember what we wrote */
    int set_motion[6]; /* gd.set_motion: motion + 1 to enter at the next frame boundary */
    float set_rate[6];
    float set_lift[6];
    int lab_request;   /* the frontend's LAB entry / MELEE_LAB=1 asked for the Lab */
    /* keys */
    unsigned char key_now[256], key_prev[256];
    /* console */
    int console_open;
    char exe_dir[MAX_PATH];
    char scripts_dir[MAX_PATH];
    char data_dir[MAX_PATH];
    char describe[1024];
} gs;

static void gs_rw_branch(void); /* the Lab's rewind: a write forks the timeline here */

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
    if (session && gs.in_event) {
        /* an event hook reports a frame that rollback may still undo: nothing may follow from it */
        luaL_error(L, "gd.%s is not allowed from an event hook during a netplay/rollback session", fn);
    }
}

/* Offline-only writes (the Geno Lab's cosmetic switches and history): gameplay scripts and the
 * console, never during a netplay/rollback session - not even for rollback_safe scripts. */
static void gs_require_offline(lua_State *L, const char *fn) {
    gs_require_gameplay(L, fn);
    if (gw_RB_Enabled() || gw_Netplay_Enabled()) {
        luaL_error(L, "gd.%s is offline-only (refused during a netplay/rollback session)", fn);
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

extern const char *gw_Mex_FighterName(int ext);
extern int gw_Mex_PortCKindToExt(int ckind);

static const char *gs_char_name(int c) {
    static char buf[32];
    const char *m;
    if (c >= 0 && c < (int) (sizeof gs_char_names / sizeof gs_char_names[0])) {
        return gs_char_names[c];
    }
    /* an m-ex fighter (ACE's Wolf is 34): its own name from the disc's m-ex data, lower case like
       the vanilla table */
    m = c >= 0 ? gw_Mex_FighterName(gw_Mex_PortCKindToExt(c)) : NULL;
    if (m != NULL && m[0] != '\0') {
        size_t i;
        for (i = 0; i + 1 < sizeof buf && m[i] != '\0'; ++i) {
            buf[i] = (char) ((m[i] >= 'A' && m[i] <= 'Z') ? m[i] - 'A' + 'a' : m[i]);
        }
        buf[i] = '\0';
        return buf;
    }
    snprintf(buf, sizeof buf, "character %d", c);
    return buf;
}

/* ---- Geno Lab: names and extra player fields ------------------------------------------------ */
static void gs_push_hit_fields(lua_State *L, const int *hi, const float *hf);
static const char *gs_element_name(int e);
/* "PlyKirby5K_Share_ACTION_AttackAirF_figatree" -> "AttackAirF"; the whole symbol when it has
   another shape; "" for none. */
static void gs_anim_name(const char *sym, char *out, size_t cap) {
    const char *p, *e;
    size_t n;
    out[0] = '\0';
    if (sym == NULL) {
        return;
    }
    p = strstr(sym, "_ACTION_");
    p = p != NULL ? p + 8 : sym;
    e = strstr(p, "_figatree");
    n = e != NULL ? (size_t) (e - p) : strlen(p);
    if (n >= cap) {
        n = cap - 1;
    }
    memcpy(out, p, n);
    out[n] = '\0';
}

/* A motion (action-state) id's name: the common states and each vanilla fighter's special states
   from the decomp's enums (gw_motion_names.inc); for a fighter with no table (m-ex), `fallback`
   (the animation name) or "Special<n>". `name_kind` is LAB_I_NAME_KIND (Kirby clones -> Kirby). */
static const char *gs_motion_name(int name_kind, int motion, const char *fallback, char *buf,
                                  size_t cap) {
    if (motion < 0) {
        return "none";
    }
    if (motion < GW_MOTION_COMMON_COUNT && gw_motion_common[motion][0] != '\0') {
        return gw_motion_common[motion];
    }
    if (motion >= GW_MOTION_COMMON_COUNT && name_kind >= 0 &&
        name_kind < (int) (sizeof gw_motion_special / sizeof gw_motion_special[0])) {
        int k = motion - GW_MOTION_COMMON_COUNT;
        if (gw_motion_special[name_kind].names != NULL && k < gw_motion_special[name_kind].count &&
            gw_motion_special[name_kind].names[k][0] != '\0') {
            return gw_motion_special[name_kind].names[k];
        }
    }
    if (fallback != NULL && fallback[0] != '\0') {
        return fallback;
    }
    snprintf(buf, cap, "Special%d", motion - GW_MOTION_COMMON_COUNT);
    return buf;
}

static const char *gs_motion_name_at(int slot, int name_kind, int motion, const char *fallback, char *buf,
                                     size_t cap);
static const char *const gs_body_state_names[] = {"normal", "invincible", "intangible"};
static const char *gs_body_state(int v) { return v >= 0 && v <= 2 ? gs_body_state_names[v] : "?"; }

static void gs_push_hitbox(lua_State *L, int slot, int i) {
    int hi[LAB_HI_COUNT], k;
    float hf[LAB_HF_COUNT];
    for (k = 0; k < LAB_HI_COUNT; ++k) hi[k] = gw_ScriptGame_HitI(slot, i, k);
    for (k = 0; k < LAB_HF_COUNT; ++k) hf[k] = gw_ScriptGame_HitF(slot, i, k);
    lua_createtable(L, 0, 24);
    gs_setint(L, "id", i);
    gs_setint(L, "state", hi[LAB_HI_STATE]);
    gs_setbool(L, "active", hi[LAB_HI_STATE] != 0);
    gs_setbool(L, "thrown", i == 4);
    gs_push_hit_fields(L, hi, hf);
    gs_setnum(L, "ox", hf[LAB_HF_OX]);
    gs_setnum(L, "oy", hf[LAB_HF_OY]);
    gs_setnum(L, "oz", hf[LAB_HF_OZ]);
    gs_setint(L, "sfx_severity", hi[LAB_HI_SFX_SEVERITY]);
    gs_setint(L, "sfx_kind", hi[LAB_HI_SFX_KIND]);
    gs_setbool(L, "clank", hi[LAB_HI_CLANK]);
    gs_setbool(L, "rebound", hi[LAB_HI_REBOUND]);
}

/* the fighter's hitboxes that are on (0-3, and 4 = the thrown hitbox), as a list */
static void gs_push_hitbox_list(lua_State *L, int slot) {
    int i, n = 0;
    lua_newtable(L);
    for (i = 0; i < 5; ++i) {
        if (gw_ScriptGame_HitI(slot, i, LAB_HI_STATE) > 0) {
            gs_push_hitbox(L, slot, i);
            lua_rawseti(L, -2, ++n);
        }
    }
}

static void gs_push_xy(lua_State *L, const char *k, float x, float y) {
    lua_createtable(L, 0, 2);
    gs_setnum(L, "x", x);
    gs_setnum(L, "y", y);
    lua_setfield(L, -2, k);
}

static void gs_push_lab_fields(lua_State *L, int slot) {
    char anim[128], buf[32];
    const char *sym = gw_ScriptGame_LabAnimSymbol(slot);
    int name_kind = gw_ScriptGame_LabI(slot, LAB_I_NAME_KIND);
    int action = gw_ScriptGame_FighterI(slot, SI_ACTION);
    int jumps_used = gw_ScriptGame_LabI(slot, LAB_I_JUMPS_USED);
    int jumps_max = gw_ScriptGame_LabI(slot, LAB_I_MAX_JUMPS);
    gs_anim_name(sym, anim, sizeof anim);
    gs_setstr(L, "motion_name", gs_motion_name_at(slot, name_kind, action, anim, buf, sizeof buf));
    gs_setint(L, "anim_id", gw_ScriptGame_LabI(slot, LAB_I_ANIM_ID));
    gs_setstr(L, "anim_name", anim);
    gs_setstr(L, "anim_symbol", sym != NULL ? sym : "");
    gs_setnum(L, "anim_frame_f", gw_ScriptGame_FighterF(slot, SF_ANIM_FRAME));
    gs_setnum(L, "anim_rate", gw_ScriptGame_LabF(slot, LAB_F_ANIM_RATE));
    gs_setnum(L, "hitstun", gw_ScriptGame_LabF(slot, LAB_F_HITSTUN));
    gs_setbool(L, "in_hitlag", gw_ScriptGame_LabI(slot, LAB_I_IN_HITLAG) == 1);
    gs_setbool(L, "in_hitstun", gw_ScriptGame_LabI(slot, LAB_I_IN_HITSTUN) == 1);
    gs_setint(L, "intangible", gw_ScriptGame_LabI(slot, LAB_I_INTANG_TIMER));
    gs_setint(L, "invincible", gw_ScriptGame_LabI(slot, LAB_I_INVINC_TIMER));
    gs_setstr(L, "body_state", gs_body_state(gw_ScriptGame_LabI(slot, LAB_I_BODY_STATE)));
    gs_setstr(L, "timed_state", gs_body_state(gw_ScriptGame_LabI(slot, LAB_I_TIMED_STATE)));
    gs_setnum(L, "kb_vx", gw_ScriptGame_LabF(slot, LAB_F_KB_VX));
    gs_setnum(L, "kb_vy", gw_ScriptGame_LabF(slot, LAB_F_KB_VY));
    gs_setnum(L, "ground_vel", gw_ScriptGame_LabF(slot, LAB_F_GR_VEL));
    gs_setnum(L, "kb_applied", gw_ScriptGame_LabF(slot, LAB_F_KB_APPLIED));
    gs_setnum(L, "z", gw_ScriptGame_LabF(slot, LAB_F_Z));
    gs_setnum(L, "scale", gw_ScriptGame_LabF(slot, LAB_F_SCALE));
    gs_setnum(L, "cmd_timer", gw_ScriptGame_LabF(slot, LAB_F_CMD_TIMER));
    gs_setnum(L, "kb_last", gw_ScriptGame_LabF(slot, LAB_F_KB_LAST));
    lua_createtable(L, 0, 4);
    gs_push_xy(L, "top", gw_ScriptGame_LabF(slot, LAB_F_ECB_TOP_X), gw_ScriptGame_LabF(slot, LAB_F_ECB_TOP_Y));
    gs_push_xy(L, "bottom", gw_ScriptGame_LabF(slot, LAB_F_ECB_BOTTOM_X),
               gw_ScriptGame_LabF(slot, LAB_F_ECB_BOTTOM_Y));
    gs_push_xy(L, "left", gw_ScriptGame_LabF(slot, LAB_F_ECB_LEFT_X),
               gw_ScriptGame_LabF(slot, LAB_F_ECB_LEFT_Y));
    gs_push_xy(L, "right", gw_ScriptGame_LabF(slot, LAB_F_ECB_RIGHT_X),
               gw_ScriptGame_LabF(slot, LAB_F_ECB_RIGHT_Y));
    lua_setfield(L, -2, "ecb");
    gs_setint(L, "ecb_lock", gw_ScriptGame_LabI(slot, LAB_I_ECB_LOCK));
    gs_setint(L, "jumps_used", jumps_used);
    gs_setint(L, "jumps_max", jumps_max);
    gs_setint(L, "jumps_left", jumps_max > jumps_used ? jumps_max - jumps_used : 0);
    gs_setint(L, "walljumps_used", gw_ScriptGame_LabI(slot, LAB_I_WALLJUMPS_USED));
    gs_setnum(L, "shield", gw_ScriptGame_LabF(slot, LAB_F_SHIELD));
    gs_setbool(L, "iasa", gw_ScriptGame_LabI(slot, LAB_I_IASA) == 1);
    gs_setint(L, "ledge_cooldown", gw_ScriptGame_LabI(slot, LAB_I_LEDGE_COOLDOWN));
    gs_setint(L, "lr_age", gw_ScriptGame_LabI(slot, LAB_I_LR_AGE));
    gs_setint(L, "jump_age", gw_ScriptGame_LabI(slot, LAB_I_JUMP_AGE));
    gs_setint(L, "draw_flags", gw_ScriptGame_LabI(slot, LAB_I_DRAW_FLAGS));
    gs_setint(L, "joint_count", gw_ScriptGame_LabI(slot, LAB_I_JOINTS));
    gs_setbool(L, "hidden", gw_ScriptGame_LabI(slot, LAB_I_HIDDEN) == 1);
    gs_setint(L, "hurtbox_count", gw_ScriptGame_LabI(slot, LAB_I_HURTBOXES));
    gs_push_hitbox_list(L, slot);
    lua_setfield(L, -2, "hitboxes");
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
    gs_push_lab_fields(L, slot);
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

/* ---- menus and online (B4: state-waiting test drivers) ---------------------------------------- */
extern int gw_SceneReport_MenuKind, gw_SceneReport_MenuHovered;
extern const char *gw_Frontend_ScreenTitle(void);
extern const char *gw_Frontend_ScreenSubtitle(void);
extern int gw_Frontend_Cursor(void);
extern const char *gw_Frontend_CursorLabel(void);
extern int gw_Netplay_Phase(void);
extern const char *gw_Netplay_Status(void);
extern const char *gw_Netplay_Code(void);
extern int gw_Netplay_IsHost(void);
extern int gw_Netplay_LobbyPhase(void);
extern int gw_Netplay_LobbyMe(void);
extern int gw_Netplay_LobbyInfo(int what);
extern int gw_Netplay_LobbyPlayer(int who, int what);
extern int gw_Netplay_LobbyStage(int i);
extern int gw_Netplay_LobbyStageGroup(int i);
extern int gw_Frontend_LobbyCursor(void);
extern int gw_Netplay_RematchPending(void);
extern int gw_Netplay_RandomStatus(void);
extern int gw_Netplay_LocalCk(void);
extern int gw_Netplay_LocalColor(void);
extern void gw_Netplay_LobbyChar(int ck, int color);
extern void gw_Netplay_LobbyStageAct(int i);
extern void gw_Netplay_LobbyReady(int on);
extern int gw_Netplay_SetCode(const char *code);

/* gd.menu() -> {frontend = {title, screen, cursor, item} (the port's own menus: gmfrontend.c),
 * native = {menu, hovered} (Melee's menu tree)}. Which one is live follows gd.scene(). */
static int l_menu(lua_State *L) {
    lua_createtable(L, 0, 6);
    gs_setstr(L, "title", gw_Frontend_ScreenTitle());
    gs_setstr(L, "screen", gw_Frontend_ScreenSubtitle());
    gs_setint(L, "cursor", gw_Frontend_Cursor());
    gs_setstr(L, "item", gw_Frontend_CursorLabel());
    gs_setint(L, "native_menu", gw_SceneReport_MenuKind);
    gs_setint(L, "native_hovered", gw_SceneReport_MenuHovered);
    return 1;
}

static const char *const gs_np_phase[] = {"idle", "working", "connected", "failed", "running", "lobby"};
static const char *const gs_lb_phase[] = {"off", "char_blind", "strike", "ban", "pick",
                                          "char_winner", "char_loser", "ready", "go"};
static const char *const gs_rnd_state[] = {"off", "looking", "matched", "timeout", "failed"};

/* gd.netplay() -> the connection and the lobby, read-only (stages, groups, the screen's cursor). */
static int l_netplay(lua_State *L) {
    int ph = gw_Netplay_Phase(), lp = gw_Netplay_LobbyPhase(), rs = gw_Netplay_RandomStatus(), i;
    int n = gw_Netplay_LobbyInfo(9);
    lua_createtable(L, 0, 16);
    gs_setstr(L, "phase", ph >= 0 && ph < 6 ? gs_np_phase[ph] : "?");
    gs_setstr(L, "status", gw_Netplay_Status());
    gs_setstr(L, "code", gw_Netplay_Code());
    gs_setbool(L, "host", gw_Netplay_IsHost());
    gs_setbool(L, "rematch", gw_Netplay_RematchPending());
    gs_setstr(L, "random", rs >= 0 && rs < 5 ? gs_rnd_state[rs] : "?");
    gs_setstr(L, "lobby", lp >= 0 && lp < 9 ? gs_lb_phase[lp] : "?");
    gs_setint(L, "me", gw_Netplay_LobbyMe());
    gs_setint(L, "game", gw_Netplay_LobbyInfo(0));
    gs_setint(L, "turn", gw_Netplay_LobbyInfo(4));
    gs_setint(L, "left", gw_Netplay_LobbyInfo(5));
    gs_setint(L, "countdown", gw_Netplay_LobbyInfo(8));
    gs_setint(L, "ck", gw_Netplay_LocalCk());
    gs_setint(L, "color", gw_Netplay_LocalColor());
    lua_createtable(L, 2, 0); /* players[1] = host, [2] = guest: {ck, color, locked, ready} */
    for (i = 0; i < 2; ++i) {
        lua_createtable(L, 0, 4);
        gs_setint(L, "ck", gw_Netplay_LobbyPlayer(i, 0));
        gs_setint(L, "color", gw_Netplay_LobbyPlayer(i, 1));
        gs_setbool(L, "locked", gw_Netplay_LobbyPlayer(i, 2));
        gs_setbool(L, "ready", gw_Netplay_LobbyPlayer(i, 3));
        lua_rawseti(L, -2, i + 1);
    }
    lua_setfield(L, -2, "players");
    lua_createtable(L, n, 0); /* stages[i] = 0 free, 1/2 struck by P1/P2, 3 banned, 4 picked */
    for (i = 0; i < n; ++i) {
        lua_pushinteger(L, gw_Netplay_LobbyStage(i));
        lua_rawseti(L, -2, i + 1);
    }
    lua_setfield(L, -2, "stages");
    lua_createtable(L, n, 0); /* groups[i] = 0 starter, 1 counterpick (the list has starters first) */
    for (i = 0; i < n; ++i) {
        lua_pushinteger(L, gw_Netplay_LobbyStageGroup(i));
        lua_rawseti(L, -2, i + 1);
    }
    lua_setfield(L, -2, "groups");
    gs_setint(L, "cursor", gw_Frontend_LobbyCursor() + 1); /* the lobby screen's stage cursor, 1-based */
    return 1;
}

/* gd.netplay_act("char", ck, color) | ("stage", i) (1-based: strike, ban or pick, whichever the
 * lobby is in) | ("ready", on) | ("code", "ABCD") (the join code). Lobby actions go through the
 * same rules as a player's (the host validates them). Returns true when accepted locally. */
static int l_netplay_act(lua_State *L) {
    const char *what = luaL_checkstring(L, 1);
    int ok = 1;
    if (_stricmp(what, "char") == 0) {
        gw_Netplay_LobbyChar((int) luaL_optinteger(L, 2, gw_Netplay_LocalCk()),
                             (int) luaL_optinteger(L, 3, gw_Netplay_LocalColor()));
    } else if (_stricmp(what, "stage") == 0) {
        gw_Netplay_LobbyStageAct((int) luaL_checkinteger(L, 2) - 1);
    } else if (_stricmp(what, "ready") == 0) {
        gw_Netplay_LobbyReady(lua_isnone(L, 2) ? 1 : lua_toboolean(L, 2));
    } else if (_stricmp(what, "code") == 0) {
        ok = gw_Netplay_SetCode(luaL_checkstring(L, 2));
    } else {
        return luaL_error(L, "gd.netplay_act: unknown action \"%s\" (char, stage, ready, code)", what);
    }
    lua_pushboolean(L, ok);
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
    gs_rw_branch();
    gw_ScriptGame_SetPercent(slot, gs_clampi(p, 0, 999));
    return 0;
}

static int l_set_stocks(lua_State *L) {
    int slot = gs_slot_arg(L, 1);
    int n = (int) luaL_checkinteger(L, 2);
    gs_require_gameplay(L, "set_stocks");
    gs_rw_branch();
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

/* gd.training_select(["kit" | "native"]) -> the current choice. Training's character and
 * stage select on the port's kit screens or the native ones; stays until changed. Menu routing
 * only - the match and its rules are Training's either way - so any script may call it. */
extern int gw_Frontend_TrainingSelect(void);
extern void gw_Frontend_SetTrainingSelect(int kit);
static int l_training_select(lua_State *L) {
    if (!lua_isnoneornil(L, 1)) {
        const char *v = luaL_checkstring(L, 1);
        if (strcmp(v, "kit") == 0) {
            gw_Frontend_SetTrainingSelect(1);
        } else if (strcmp(v, "native") == 0) {
            gw_Frontend_SetTrainingSelect(0);
        } else {
            luaL_error(L, "gd.training_select: \"kit\" or \"native\" (got \"%s\")", v);
        }
    }
    lua_pushstring(L, gw_Frontend_TrainingSelect() ? "kit" : "native");
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
    if (gs.ndraw[gs.build] >= GS_MAX_DRAW) {
        return NULL;
    }
    d = &gs.draw[gs.build][gs.ndraw[gs.build]++];
    memset(d, 0, sizeof *d);
    d->kind = kind;
    d->size = 1.0f;
    return d;
}

static int l_text(lua_State *L) {
    GwScriptDraw *d;
    const char *s;
    float x = (float) luaL_checknumber(L, 1), y = (float) luaL_checknumber(L, 2);
    /* the colour and size before luaL_tolstring, which pushes: index 4 would then be the text
       (every gd.text without a colour used to fail "colours are 0xRRGGBBAA numbers") */
    uint32_t rgba = gs_color_arg(L, 4, 0xFFFFFFFFu);
    float size = (float) luaL_optnumber(L, 5, 1.0);
    luaL_tolstring(L, 3, NULL);
    s = lua_tostring(L, -1);
    d = gs_draw_new(GW_SDRAW_TEXT);
    if (d != NULL) {
        d->x = x;
        d->y = y;
        d->rgba = rgba;
        d->size = size;
        snprintf(d->text, sizeof d->text, "%s", s);
    }
    return 0;
}

static int gs_rect(lua_State *L, int kind) {
    GwScriptDraw *d;
    float x = (float) luaL_checknumber(L, 1), y = (float) luaL_checknumber(L, 2);
    float w = (float) luaL_checknumber(L, 3), h = (float) luaL_checknumber(L, 4);
    uint32_t rgba = gs_color_arg(L, 5, kind == GW_SDRAW_FILL ? 0x000000A0u : 0xFFFFFFFFu);
    d = gs_draw_new(kind); /* after the checks: a bad call leaves no blank item behind */
    if (d != NULL) {
        d->x = x;
        d->y = y;
        d->w = w;
        d->h = h;
        d->rgba = rgba;
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

/* Screenshots: beta's gw_Screenshot (shim_vi.h): the NEXT presented frame is written as a PNG at
 * render resolution, overlays excluded, without blocking - the file appears a few frames later. */
#include "shim_vi.h"

static int gs_screenshot(const char *path) {
    gw_Screenshot(path);
    gw_Console_Print(GS_GREEN, "screenshot queued: %s (written in a few frames)", path);
    return 0;
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

/* gd.mouse() -> x, y, buttons, wheel: the pointer in the 640x480 kit/overlay space (-1000, -1000
 * when it is outside the picture), buttons 1 left + 2 right + 4 middle, the wheel's notches this
 * tick (+ up). Local UI input only: it never reaches the pads or a netplay peer. Polling it is what
 * shows the cursor during a match (a script's menu is open), so poll it only while one is. */
extern void gw_Mouse_ScriptRead(float *x, float *y, int *buttons, float *wheel);
extern void gw_Mouse_ScriptTick(void);
static int l_mouse(lua_State *L) {
    float x, y, wheel;
    int buttons;
    gw_Mouse_ScriptRead(&x, &y, &buttons, &wheel);
    lua_pushnumber(L, x);
    lua_pushnumber(L, y);
    lua_pushinteger(L, buttons);
    lua_pushnumber(L, wheel);
    return 4;
}

static void gs_poll_keys(void) {
    HWND fg = GetForegroundWindow();
    DWORD pid = 0;
    int vk, focused;
    if (fg != NULL) {
        GetWindowThreadProcessId(fg, &pid);
    }
    /* typing (the console, a name, a room code): the keys are text, not hotkeys */
    focused = pid == GetCurrentProcessId() && !gs.console_open &&
              (int) (GetTickCount() - (DWORD) gw_TextEntryUntil) >= 0;
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
    if (name[0] == '\0' || strlen(name) > 96) {
        luaL_error(L, "data file names are 1-96 characters");
    }
    for (p = name; *p != '\0'; ++p) {
        /* folders too: "dir/file" (a '/' never first, last or doubled; a '.' never starts a part) */
        int slash_ok = *p == '/' && p != name && p[1] != '\0' && p[1] != '/';
        if (!((*p >= 'a' && *p <= 'z') || (*p >= 'A' && *p <= 'Z') || (*p >= '0' && *p <= '9') ||
              *p == '_' || *p == '-' || slash_ok || (*p == '.' && p != name && p[-1] != '/'))) {
            luaL_error(L, "data file names use letters, digits, _ - . and / (got \"%s\")", name);
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
    {
        /* the folders of a "dir/sub/file" name (stage E: versioned frame-data exports) */
        char *q = out + strlen(dir) + 1;
        for (; *q != '\0'; ++q) {
            if (*q == '/') {
                *q = '\0';
                CreateDirectoryA(out, NULL);
                *q = '\\';
            }
        }
    }
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

/* ===================================================================================== * the Geno Lab API (docs/geno.md "Geno Lab"): inspection, debug drawing, projection, history
 * ============================================================================================ */
static int gs_present_arg(lua_State *L, int idx) {
    int slot = gs_slot_arg(L, idx);
    return gs_players_present(slot) ? slot : -1;
}

/* gd.debug_draw(port) -> flags; gd.debug_draw(port, flags) -> previous flags (offline, gameplay) */
static int l_debug_draw(lua_State *L) {
    int slot = gs_present_arg(L, 1), v;
    if (slot < 0) {
        lua_pushnil(L);
        return 1;
    }
    if (lua_isnoneornil(L, 2)) {
        v = gw_ScriptGame_LabDebugDraw(slot, 0, 0);
    } else {
        int flags = (int) luaL_checkinteger(L, 2);
        gs_require_offline(L, "debug_draw");
        v = gw_ScriptGame_LabDebugDraw(slot, 1, flags & 0xFF);
    }
    lua_pushinteger(L, v);
    return 1;
}

/* gd.debug_stage() -> flags; gd.debug_stage(flags) -> the flags now set (offline, gameplay) */
static int l_debug_stage(lua_State *L) {
    int v;
    if (lua_isnoneornil(L, 1)) {
        v = gw_ScriptGame_LabStageDraw(0, 0);
    } else {
        int flags = (int) luaL_checkinteger(L, 1) & 31;
        gs_require_offline(L, "debug_stage");
        v = gw_ScriptGame_LabStageDraw(31, flags);
        if (v >= 0) {
            gs.stage_zones = (flags & LAB_STAGE_ZONES) != 0;
        }
    }
    if (v < 0) {
        lua_pushnil(L); /* no match camera */
        return 1;
    }
    lua_pushinteger(L, v | (gs.stage_zones ? LAB_STAGE_ZONES : 0));
    return 1;
}

/* gd.hitboxes(port [, all]) -> the hitboxes that are on (all = every slot 0-4, on or off) */
static int l_hitboxes(lua_State *L) {
    int slot = gs_present_arg(L, 1), i, n = 0;
    int all = lua_toboolean(L, 2);
    if (slot < 0) {
        lua_pushnil(L);
        return 1;
    }
    if (!all) {
        gs_push_hitbox_list(L, slot);
        return 1;
    }
    lua_newtable(L);
    for (i = 0; i < 5; ++i) {
        gs_push_hitbox(L, slot, i);
        lua_rawseti(L, -2, ++n);
    }
    return 1;
}

/* gd.hurtboxes(port) -> {{id, bone, state, height, grabbable, ax, ay, az, bx, by, bz, radius}} */
static int l_hurtboxes(lua_State *L) {
    static const char *const heights[] = {"low", "mid", "high"};
    int slot = gs_present_arg(L, 1), i, n;
    if (slot < 0) {
        lua_pushnil(L);
        return 1;
    }
    n = gw_ScriptGame_LabI(slot, LAB_I_HURTBOXES);
    lua_createtable(L, n > 0 ? n : 0, 0);
    for (i = 0; i < n && i < 15; ++i) {
        int h = gw_ScriptGame_HurtI(slot, i, LAB_UI_HEIGHT);
        lua_createtable(L, 0, 12);
        gs_setint(L, "id", i);
        gs_setint(L, "bone", gw_ScriptGame_HurtI(slot, i, LAB_UI_BONE));
        gs_setstr(L, "state", gs_body_state(gw_ScriptGame_HurtI(slot, i, LAB_UI_STATE)));
        gs_setstr(L, "height", h >= 0 && h <= 2 ? heights[h] : "?");
        gs_setbool(L, "grabbable", gw_ScriptGame_HurtI(slot, i, LAB_UI_GRABBABLE) == 1);
        gs_setnum(L, "ax", gw_ScriptGame_HurtF(slot, i, LAB_UF_AX));
        gs_setnum(L, "ay", gw_ScriptGame_HurtF(slot, i, LAB_UF_AY));
        gs_setnum(L, "az", gw_ScriptGame_HurtF(slot, i, LAB_UF_AZ));
        gs_setnum(L, "bx", gw_ScriptGame_HurtF(slot, i, LAB_UF_BX));
        gs_setnum(L, "by", gw_ScriptGame_HurtF(slot, i, LAB_UF_BY));
        gs_setnum(L, "bz", gw_ScriptGame_HurtF(slot, i, LAB_UF_BZ));
        gs_setnum(L, "radius", gw_ScriptGame_HurtF(slot, i, LAB_UF_SIZE));
        lua_rawseti(L, -2, i + 1);
    }
    return 1;
}

/* the match camera, fetched once per draw pass */
static int gs_cam_fetch(void) {
    int k;
    if (gs.cam_fetched == gs.cam_stamp) {
        return gs.cam_have;
    }
    gs.cam_fetched = gs.cam_stamp;
    gs.cam_have = gw_ScriptGame_CameraF(LAB_CAM_OK) != 0.0f;
    if (gs.cam_have) {
        for (k = 0; k < LAB_CAM_COUNT; ++k) {
            gs.cam[k] = gw_ScriptGame_CameraF(k);
        }
    }
    return gs.cam_have;
}

/* World -> the 640x480 script screen through the match camera: the same maths as the game's
 * lbVector_WorldToScreen (the viewing matrix, then MTXPerspective / MTXOrtho and GXProject with the
 * camera's viewport). Returns 1 with the point in front of the camera, 0 behind it or no camera. */
static int gs_project(float x, float y, float z, float *sx, float *sy, float *depth) {
    const float *m = &gs.cam[LAB_CAM_VIEW];
    float ex, ey, ez, xc, yc, wc;
    float vx, vy, vw, vh;
    int proj;
    if (!gs_cam_fetch()) {
        return 0;
    }
    ex = m[0] * x + m[1] * y + m[2] * z + m[3];
    ey = m[4] * x + m[5] * y + m[6] * z + m[7];
    ez = m[8] * x + m[9] * y + m[10] * z + m[11];
    proj = (int) gs.cam[LAB_CAM_PROJ];
    if (proj == 2) { /* ortho: top, bottom, left, right */
        float t = gs.cam[LAB_CAM_P0], b = gs.cam[LAB_CAM_P1], l = gs.cam[LAB_CAM_P2],
              r = gs.cam[LAB_CAM_P3];
        if (r == l || t == b) return 0;
        xc = ex * (2.0f / (r - l)) - (r + l) / (r - l);
        yc = ey * (2.0f / (t - b)) - (t + b) / (t - b);
        wc = 1.0f;
    } else if (proj == 0) { /* perspective: fov (degrees), aspect */
        float fov = gs.cam[LAB_CAM_P0], aspect = gs.cam[LAB_CAM_P1];
        float cot;
        if (ez > -0.01f || aspect == 0.0f) {
            *depth = -ez;
            return 0; /* at or behind the camera */
        }
        cot = 1.0f / tanf(fov * 0.5f * 3.14159265f / 180.0f);
        xc = ex * (cot / aspect);
        yc = ey * cot;
        wc = 1.0f / -ez;
    } else {
        return 0; /* frustum cameras: not used by a match */
    }
    vx = gs.cam[LAB_CAM_VP_XMIN];
    vy = gs.cam[LAB_CAM_VP_YMIN];
    vw = gs.cam[LAB_CAM_VP_XMAX] - vx;
    vh = gs.cam[LAB_CAM_VP_YMAX] - vy;
    *sx = vw * 0.5f + vx + wc * xc * vw * 0.5f;
    *sy = vh * 0.5f + vy - wc * yc * vh * 0.5f;
    *depth = -ez;
    return 1;
}

/* gd.project(x, y [, z]) -> sx, sy, visible, depth  (nil when there is no match camera) */
static int l_project(lua_State *L) {
    float sx = 0.0f, sy = 0.0f, depth = 0.0f;
    float x = (float) luaL_checknumber(L, 1), y = (float) luaL_checknumber(L, 2);
    float z = (float) luaL_optnumber(L, 3, 0.0);
    int ok = gs_project(x, y, z, &sx, &sy, &depth);
    if (!ok && !gs.cam_have) {
        lua_pushnil(L);
        return 1;
    }
    lua_pushnumber(L, sx);
    lua_pushnumber(L, sy);
    lua_pushboolean(L, ok && sx >= 0.0f && sx < 640.0f && sy >= 0.0f && sy < 480.0f);
    lua_pushnumber(L, depth);
    return 4;
}

/* gd.joints(port) -> {{index, parent, x, y, z, sx, sy, on}, ...}; list position = index + 1 */
static int l_joints(lua_State *L) {
    int slot = gs_present_arg(L, 1), n, i;
    if (slot < 0) {
        lua_pushnil(L);
        return 1;
    }
    n = gw_ScriptGame_LabI(slot, LAB_I_JOINTS);
    lua_createtable(L, n > 0 ? n : 0, 0);
    for (i = 0; i < n; ++i) {
        float x = gw_ScriptGame_JointF(slot, i, 0), y = gw_ScriptGame_JointF(slot, i, 1);
        float z = gw_ScriptGame_JointF(slot, i, 2), sx = 0.0f, sy = 0.0f, depth = 0.0f;
        int parent = gw_ScriptGame_JointParent(slot, i);
        lua_createtable(L, 0, 8);
        gs_setint(L, "index", i);
        gs_setint(L, "parent", parent);
        gs_setbool(L, "valid", parent != -2);
        gs_setnum(L, "x", x);
        gs_setnum(L, "y", y);
        gs_setnum(L, "z", z);
        if (parent != -2 && gs_project(x, y, z, &sx, &sy, &depth)) {
            gs_setnum(L, "sx", sx);
            gs_setnum(L, "sy", sy);
            gs_setbool(L, "on", 1);
        } else {
            gs_setbool(L, "on", 0);
        }
        lua_rawseti(L, -2, i + 1);
    }
    return 1;
}

/* gd.dobjs(port) -> the fighter's draw list, read-only: { models = {state per model-part model},
 * costume = {{frame, tu, tv, su, sv} per costume texture-anim TObj}, {index, hidden, render, tobjs =
 * {{id, src, flags, tu, tv, su, sv, frame, fmt, w, h}}} per DObj }. */
static void gs_tobj_table(lua_State *L, int slot, int d, int t) {
    static const char *const names[LAB_TF_COUNT] = {"id", "src", "flags", "tu", "tv", "su", "sv", "frame", "fmt", "w", "h"};
    int f;
    lua_createtable(L, 0, LAB_TF_COUNT);
    for (f = 0; f < LAB_TF_COUNT; ++f) {
        gs_setnum(L, names[f], gw_ScriptGame_LabTObjF(slot, d, t, f));
    }
}

static int l_dobjs(lua_State *L) {
    int slot = gs_present_arg(L, 1), n, i, t, nt;
    if (slot < 0) {
        lua_pushnil(L);
        return 1;
    }
    n = gw_ScriptGame_LabDObjI(slot, 0, LAB_DI_COUNT);
    lua_createtable(L, n > 0 ? n : 0, 2);
    nt = gw_ScriptGame_LabDObjI(slot, 0, LAB_DI_MODELS);
    lua_createtable(L, nt > 0 ? nt : 0, 0);
    for (i = 0; i < nt && i < 12; ++i) {
        lua_pushinteger(L, gw_ScriptGame_LabDObjI(slot, i, LAB_DI_MODEL_STATE));
        lua_rawseti(L, -2, i + 1);
    }
    lua_setfield(L, -2, "models");
    nt = gw_ScriptGame_LabDObjI(slot, 0, LAB_DI_COSTUME_TOBJS);
    lua_createtable(L, nt > 0 ? nt : 0, 0);
    for (i = 0; i < nt && i < 5; ++i) {
        gs_tobj_table(L, slot, -1, i);
        lua_rawseti(L, -2, i + 1);
    }
    lua_setfield(L, -2, "costume");
    for (i = 0; i < n; ++i) {
        int flags = gw_ScriptGame_LabDObjI(slot, i, LAB_DI_FLAGS);
        lua_createtable(L, 0, 4);
        gs_setint(L, "index", i);
        gs_setbool(L, "hidden", (flags & 1) != 0);
        gs_setint(L, "render", gw_ScriptGame_LabDObjI(slot, i, LAB_DI_RENDER));
        nt = gw_ScriptGame_LabDObjI(slot, i, LAB_DI_TOBJS);
        lua_createtable(L, nt > 0 ? nt : 0, 0);
        for (t = 0; t < nt; ++t) {
            gs_tobj_table(L, slot, i, t);
            lua_rawseti(L, -2, t + 1);
        }
        lua_setfield(L, -2, "tobjs");
        lua_rawseti(L, -2, i + 1);
    }
    return 1;
}

/* gd.attrs(port) -> {name = value} for the 40 named ftCo_DatAttrs fields, as the fighter has them */
static int l_attrs(lua_State *L) {
    int slot = gs_present_arg(L, 1), i, n = gw_ScriptGame_LabAttrCount();
    if (slot < 0) {
        lua_pushnil(L);
        return 1;
    }
    lua_createtable(L, 0, n);
    for (i = 0; i < n; ++i) {
        const char *name = gw_ScriptGame_LabAttrName(i);
        if (name != NULL) {
            gs_setnum(L, name, gw_ScriptGame_LabAttrF(slot, i));
        }
    }
    return 1;
}

/* gd.motion_name(id [, port]) -> the action state's name (the port picks the fighter's table) */
static int l_motion_name(lua_State *L) {
    char buf[32];
    int id = (int) luaL_checkinteger(L, 1);
    int kind = -1, slot = -1;
    if (!lua_isnoneornil(L, 2)) {
        slot = gs_present_arg(L, 2);
        if (slot >= 0) {
            kind = gw_ScriptGame_LabI(slot, LAB_I_NAME_KIND);
        }
    }
    lua_pushstring(L, gs_motion_name_at(slot, kind, id, NULL, buf, sizeof buf));
    return 1;
}

/* ---- Stage 2: subaction-script timeline (read-only walk of the guest script words) ------------ */
static int gs_motion_ok(int slot, int motion);
extern int gw_ScriptGame_LabMotionAnim(int slot, int msid);
extern int gw_ScriptGame_LabCommonCount(int slot);
extern const void *gw_ScriptGame_LabScript(int slot, int anim);
extern const char *gw_ScriptGame_LabAnimSymbolFor(int slot, int anim);
extern float gw_ScriptGame_LabAnimEnd(int slot);
extern int gw_ScriptGame_LabSetMotion(int slot, int msid, int rate_bits, int lift_bits);

/* Command lengths in words for opcodes 10-58 (ftaction.c ftAction_803C0870) and the names the
   community decoders use (HSDRawViewer command_fighter.yml; checked against the decomp's handlers). */
static const unsigned char gs_cmd_len[49] = {5, 5, 1, 1, 1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                                             1, 1, 1, 1, 1, 1, 1, 3, 1, 1, 1, 7, 4, 1, 1, 1, 1,
                                             1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 3, 2, 1, 4};
static const char *const gs_cmd_names[64] = {
    "end", "wait", "wait_until", "loop", "loop_end", "call", "return", "goto", "wait_anim",
    "bg_flash", "gfx", "hitbox", "hitbox_damage", "hitbox_size", "hitbox_flags", "hitbox_remove",
    "hitboxes_clear", "sfx", "smash_sfx", "cmd_var", "throw_flag", "throw_flag_b1",
    "throw_flag_b2", "iasa", "throw_flag_b0", "air_state", "body_state", "hurtboxes_state",
    "hurtbox_state", "jab_combo", "jab_rapid", "model_state", "models_revert", "models_remove",
    "throw", "item_visibility", "article_visibility", "visibility", "random_sfx", "pitch_sfx",
    "tex_anim", "part_anim", "parasol", "rumble", "rumble_stop", "color_anim", "color_overlay",
    "color_overlay_off", "flag_221E", "sword_trail", "anim_part", "self_damage", "continuation",
    "flag_2225", "footstep_fx", "landing_fx", "smash_charge", "unk_57", "wind", "geno",
    "op_60", "op_61", "op_62", "op_63"};

static int gs_guest_ok(uint32_t addr, uint32_t len) {
    uint32_t base = (uint32_t) (uintptr_t) gw_mem1;
    return gw_mem1 != NULL && addr >= base && addr + len <= base + gw_mem1_size && addr + len > addr;
}
static uint32_t gs_guest_word(uint32_t addr) {
    const unsigned char *p = (const unsigned char *) (uintptr_t) addr;
    return ((uint32_t) p[0] << 24) | ((uint32_t) p[1] << 16) | ((uint32_t) p[2] << 8) | p[3];
}
static int gs_sbits(uint32_t v, int hi, int n) { /* signed field: n bits ending `hi` bits down */
    int32_t x = (int32_t) ((v << hi) & 0xFFFFFFFFu);
    return (int) (x >> (32 - n));
}
#define GS_UBITS(v, shift, n) ((int) (((v) >> (shift)) & ((1u << (n)) - 1u)))

/* one command as a Lua table on the stack top's list; `t` = the frame it runs (1-based) */
static void gs_push_cmd(lua_State *L, uint32_t addr, int op, float t, int len) {
    uint32_t w0 = gs_guest_word(addr);
    int k;
    lua_createtable(L, 0, 16);
    gs_setnum(L, "frame", t + 1.0f);
    gs_setint(L, "op", op);
    gs_setstr(L, "name", gs_cmd_names[op & 63]);
    gs_setint(L, "addr", (lua_Integer) addr);
    lua_createtable(L, len, 0);
    for (k = 0; k < len; ++k) {
        lua_pushinteger(L, (lua_Integer) gs_guest_word(addr + 4u * (uint32_t) k));
        lua_rawseti(L, -2, k + 1);
    }
    lua_setfield(L, -2, "words");
    switch (op) {
    case 10: /* gfx */
        gs_setint(L, "bone", GS_UBITS(w0, 18, 8));
        gs_setint(L, "gfx", GS_UBITS(gs_guest_word(addr + 4), 16, 16));
        break;
    case 11: { /* hitbox */
        uint32_t w1 = gs_guest_word(addr + 4), w2 = gs_guest_word(addr + 8);
        uint32_t w3 = gs_guest_word(addr + 12), w4 = gs_guest_word(addr + 16);
        gs_setint(L, "id", GS_UBITS(w0, 23, 3));
        gs_setint(L, "group", GS_UBITS(w0, 20, 3));
        gs_setint(L, "bone", GS_UBITS(w0, 11, 8));
        gs_setint(L, "damage", GS_UBITS(w0, 0, 10));
        gs_setnum(L, "size", GS_UBITS(w1, 16, 16) / 256.0);
        gs_setnum(L, "oz", gs_sbits(w1, 16, 16) / 256.0);
        gs_setnum(L, "oy", gs_sbits(w2, 0, 16) / 256.0);
        gs_setnum(L, "ox", gs_sbits(w2, 16, 16) / 256.0);
        gs_setint(L, "angle", GS_UBITS(w3, 23, 9));
        gs_setint(L, "kbg", GS_UBITS(w3, 14, 9));
        gs_setint(L, "wbk", GS_UBITS(w3, 5, 9));
        gs_setint(L, "bkb", GS_UBITS(w4, 23, 9));
        gs_setint(L, "element", GS_UBITS(w4, 18, 5));
        gs_setstr(L, "element_name", gs_element_name(GS_UBITS(w4, 18, 5)));
        gs_setint(L, "shield_damage", gs_sbits(w4, 14, 8));
        gs_setbool(L, "hit_ground", GS_UBITS(w4, 1, 1));
        gs_setbool(L, "hit_air", GS_UBITS(w4, 0, 1));
        break;
    }
    case 12: case 13: /* hitbox_damage / hitbox_size */
        gs_setint(L, "id", GS_UBITS(w0, 23, 3));
        gs_setint(L, "value", GS_UBITS(w0, 0, 23));
        break;
    case 15: /* hitbox_remove */
        gs_setint(L, "id", GS_UBITS(w0, 0, 26));
        break;
    case 17: /* sfx */
        gs_setint(L, "sfx", (lua_Integer) gs_guest_word(addr + 4));
        break;
    case 19: /* cmd_var */
        gs_setint(L, "index", GS_UBITS(w0, 24, 2));
        gs_setint(L, "value", GS_UBITS(w0, 0, 24));
        break;
    case 26: case 27: case 37: case 36: case 35: /* states / visibility: one value */
    case 25: case 29: case 30:
        gs_setint(L, "value", GS_UBITS(w0, 0, 26));
        break;
    case 28: /* hurtbox_state */
        gs_setint(L, "bone", GS_UBITS(w0, 18, 8));
        gs_setint(L, "value", GS_UBITS(w0, 0, 18));
        break;
    default:
        break;
    }
}

/* Walk the script from `start` the way ftAction's interpreter times it: wait (1) adds frames,
   wait_until (2) jumps to an animation frame, loops (3/4) and calls (5/6/7) are followed, and
   the walk stops at end (0), wait_anim (8), 2000 commands or frame 1000. Pushes the event list;
   returns the last frame reached. */
static float gs_walk_script(lua_State *L, uint32_t start, int *truncated) {
    uint32_t pc = start, ret[8], loop_body[8];
    int loop_left[8], nret = 0, nloop = 0, steps = 0, n = 0;
    float t = 0.0f;
    *truncated = 0;
    lua_newtable(L);
    while (gs_guest_ok(pc, 4)) {
        uint32_t w0 = gs_guest_word(pc);
        int op = (int) (w0 >> 26), len;
        if (++steps > 2000 || t > 1000.0f) {
            *truncated = 1;
            break;
        }
        switch (op) {
        case 0:
            return t;
        case 1:
            t += (float) (w0 & 0x3FFFFFFu);
            pc += 4;
            continue;
        case 2: {
            float at = (float) (w0 & 0x3FFFFFFu);
            if (at > t) t = at;
            pc += 4;
            continue;
        }
        case 3:
            if (nloop < 8) {
                loop_body[nloop] = pc + 4;
                loop_left[nloop] = (int) (w0 & 0x3FFFFFFu);
                nloop++;
            }
            pc += 4;
            continue;
        case 4:
            if (nloop > 0 && --loop_left[nloop - 1] > 0) {
                pc = loop_body[nloop - 1];
            } else {
                if (nloop > 0) nloop--;
                pc += 4;
            }
            continue;
        case 5:
            if (nret < 8 && gs_guest_ok(pc + 4, 4)) {
                ret[nret++] = pc + 8;
                pc = gs_guest_word(pc + 4);
            } else {
                return t;
            }
            continue;
        case 6:
            if (nret == 0) return t;
            pc = ret[--nret];
            continue;
        case 7:
            if (!gs_guest_ok(pc + 4, 4)) return t;
            pc = gs_guest_word(pc + 4);
            continue;
        case 8:
            *truncated = 2; /* waits for the animation to end */
            return t;
        case 9:
            pc += 4;
            continue;
        default:
            break;
        }
        if (op == 59) {
            len = (int) ((w0 >> 16) & 0xF);
            if (len == 0) len = 1;
        } else if (op >= 10 && op <= 58) {
            len = gs_cmd_len[op - 10];
        } else {
            len = 1;
        }
        if (!gs_guest_ok(pc, 4u * (uint32_t) len)) break;
        gs_push_cmd(L, pc, op, t, len);
        lua_rawseti(L, -2, ++n);
        pc += 4u * (uint32_t) len;
    }
    return t;
}

/* gd.timeline(port [, motion]) -> {motion, motion_name, anim_id, anim_name, end_frame, length,
   events = {{frame, op, name, words, ...decoded fields}}, truncated}
   The script of the fighter's current action, or of `motion` (its row in the fighter's tables). */
static int l_timeline(lua_State *L) {
    int slot = gs_present_arg(L, 1), motion, anim, name_kind, truncated = 0;
    const void *script;
    char anim_name[128], buf[32];
    float t;
    if (slot < 0) {
        lua_pushnil(L);
        return 1;
    }
    name_kind = gw_ScriptGame_LabI(slot, LAB_I_NAME_KIND);
    if (lua_isnoneornil(L, 2)) {
        motion = gw_ScriptGame_FighterI(slot, SI_ACTION);
        anim = gw_ScriptGame_LabI(slot, LAB_I_ANIM_ID);
    } else {
        motion = (int) luaL_checkinteger(L, 2);
        if (!gs_motion_ok(slot, motion)) {
            lua_pushnil(L);
            lua_pushstring(L, "no such motion for this fighter");
            return 2;
        }
        anim = gw_ScriptGame_LabMotionAnim(slot, motion);
    }
    lua_createtable(L, 0, 10);
    gs_setint(L, "motion", motion);
    gs_setint(L, "anim_id", anim);
    gs_anim_name(anim >= 0 ? gw_ScriptGame_LabAnimSymbolFor(slot, anim) : NULL, anim_name,
                 sizeof anim_name);
    gs_setstr(L, "anim_name", anim_name);
    gs_setstr(L, "motion_name", gs_motion_name_at(slot, name_kind, motion, anim_name, buf, sizeof buf));
    if (lua_isnoneornil(L, 2)) {
        gs_setnum(L, "end_frame", gw_ScriptGame_LabAnimEnd(slot));
    }
    script = anim >= 0 ? gw_ScriptGame_LabScript(slot, anim) : NULL;
    if (script == NULL || !gs_guest_ok((uint32_t) (uintptr_t) script, 4)) {
        lua_newtable(L);
        lua_setfield(L, -2, "events");
        gs_setnum(L, "length", 0);
        return 1;
    }
    gs_setint(L, "script", (lua_Integer) (uintptr_t) script);
    t = gs_walk_script(L, (uint32_t) (uintptr_t) script, &truncated);
    lua_setfield(L, -2, "events");
    gs_setnum(L, "length", t + 1.0f);
    gs_setstr(L, "stop", truncated == 1 ? "limit" : truncated == 2 ? "anim_end" : "end");
    return 1;
}

/* ---- Stage E: every action state a fighter has (the state browser) ---------------------------- */
extern int gw_Mex_MoveLogicEntriesForKind(int fk);
extern int gw_Geno_ProfileForKind(int kind);
extern int gw_Geno_StateCount(int p);
extern const char *gw_Geno_StateName(int p, int s);
#define GS_GENO_MOTION_BASE 0x400

static int gs_geno_profile(int slot) {
    return gw_Geno_ProfileForKind(gw_ScriptGame_LabI(slot, LAB_I_KIND));
}

/* how many special states the fighter has: its kind's table in the decomp (Kirby clones: Kirby's),
   or the m-ex MoveLogic table when that is longer */
static int gs_special_count(int slot) {
    int kind = gw_ScriptGame_LabI(slot, LAB_I_NAME_KIND), n = 0, m;
    if (kind >= 0 && kind < (int) (sizeof gw_motion_special / sizeof gw_motion_special[0])) {
        n = gw_motion_special[kind].count;
    }
    m = gw_Mex_MoveLogicEntriesForKind(gw_ScriptGame_LabI(slot, LAB_I_KIND));
    return m > n ? m : n;
}

/* gs_motion_name, plus Geno action states (motion 0x400 + n) by their geno.json name */
static const char *gs_motion_name_at(int slot, int name_kind, int motion, const char *fallback, char *buf,
                                     size_t cap) {
    if (motion >= GS_GENO_MOTION_BASE && slot >= 0) {
        const char *n = gw_Geno_StateName(gs_geno_profile(slot), motion - GS_GENO_MOTION_BASE);
        if (n != NULL && n[0] != '\0') {
            return n;
        }
        snprintf(buf, cap, "Geno%d", motion - GS_GENO_MOTION_BASE);
        return buf;
    }
    return gs_motion_name(name_kind, motion, fallback, buf, cap);
}

/* a motion id the fighter really has a row for: common states, a special in the decomp's table for
   its kind (Kirby clones use Kirby's) or in its m-ex MoveLogic table, or one of its Geno states -
   so a script never enters a garbage row */
static int gs_motion_ok(int slot, int motion) {
    int common = gw_ScriptGame_LabCommonCount(slot);
    if (motion < 0 || common <= 0) return 0;
    if (motion >= GS_GENO_MOTION_BASE) {
        int p = gs_geno_profile(slot);
        return p >= 0 && motion - GS_GENO_MOTION_BASE < gw_Geno_StateCount(p) &&
               gw_ScriptGame_LabMotionAnim(slot, motion) >= -1;
    }
    if (motion < common) return gw_ScriptGame_LabMotionAnim(slot, motion) >= -1;
    if (motion - common < gs_special_count(slot)) {
        return gw_ScriptGame_LabMotionAnim(slot, motion) >= -1;
    }
    return 0;
}

/* gd.motion_list(port) -> {{id, name, group ("common" | "special" | "mex" | "geno"), anim_id,
   anim_name}, ...}: every action state the fighter has a row for, in id order. "mex" = a special
   past the decomp's table (the m-ex MoveLogic table's own). Read-only. */
static int l_motion_list(lua_State *L) {
    int slot = gs_present_arg(L, 1), common, nspec, kind, dec = 0, m, n = 0, p;
    char anim_name[128], buf[32];
    if (slot < 0) {
        lua_pushnil(L);
        return 1;
    }
    common = gw_ScriptGame_LabCommonCount(slot);
    nspec = gs_special_count(slot);
    kind = gw_ScriptGame_LabI(slot, LAB_I_NAME_KIND);
    if (kind >= 0 && kind < (int) (sizeof gw_motion_special / sizeof gw_motion_special[0])) {
        dec = gw_motion_special[kind].count;
    }
    p = gs_geno_profile(slot);
    lua_newtable(L);
    for (m = 0; m < common + nspec + (p >= 0 ? GS_GENO_MOTION_BASE + gw_Geno_StateCount(p) : 0); ++m) {
        int anim;
        const char *group;
        if (m >= common + nspec && m < GS_GENO_MOTION_BASE) {
            if (p < 0) break;
            m = GS_GENO_MOTION_BASE - 1;
            continue;
        }
        if (!gs_motion_ok(slot, m)) continue;
        anim = gw_ScriptGame_LabMotionAnim(slot, m);
        gs_anim_name(anim >= 0 ? gw_ScriptGame_LabAnimSymbolFor(slot, anim) : NULL, anim_name, sizeof anim_name);
        group = m >= GS_GENO_MOTION_BASE ? "geno" : m < common ? "common" : (m - common < dec ? "special" : "mex");
        lua_createtable(L, 0, 5);
        gs_setint(L, "id", m);
        gs_setstr(L, "name", gs_motion_name_at(slot, kind, m, anim_name, buf, sizeof buf));
        gs_setstr(L, "group", group);
        gs_setint(L, "anim_id", anim);
        gs_setstr(L, "anim_name", anim_name);
        lua_rawseti(L, -2, ++n);
    }
    return 1;
}

/* gd.set_motion(port | {ports}, motion [, frame [, rate [, lift]]]) -> true | false, why. Offline,
   gameplay.
   `lift` (units, e.g. 40) first puts a grounded fighter in the air that much higher - for aerials,
   which the next collision check would otherwise land at once.
   At the next frame boundary the fighter enters `motion` (the plain Fighter_ChangeMotionState its
   entry function would make; states whose entry does more may not behave), then the game runs
   the frames up to `frame` (default 1; entering is frame 1, as frame-data sites count) and
   pauses: the fighter shows that frame of the move with everything its script did on the way
   (hitboxes included). Ports set before the same boundary start together: lock-step. */
static int l_set_motion(lua_State *L) {
    int slots[6], n = 0, k;
    int motion = (int) luaL_checkinteger(L, 2);
    int frame = (int) luaL_optinteger(L, 3, 1);
    float rate = (float) luaL_optnumber(L, 4, 1.0);
    float lift = (float) luaL_optnumber(L, 5, 0.0);
    gs_require_offline(L, "set_motion");
    if (lua_istable(L, 1)) { /* {1, 2}: several ports at once - lock-step */
        lua_Integer i, len = (lua_Integer) lua_rawlen(L, 1);
        for (i = 1; i <= len && n < 6; ++i) {
            lua_rawgeti(L, 1, i);
            slots[n++] = gs_present_arg(L, lua_gettop(L));
            lua_pop(L, 1);
        }
    } else {
        slots[n++] = gs_present_arg(L, 1);
    }
    for (k = 0; k < n; ++k) {
        if (slots[k] < 0) {
            lua_pushboolean(L, 0);
            lua_pushstring(L, "no fighter on that port");
            return 2;
        }
        if (!gs_motion_ok(slots[k], motion)) {
            lua_pushboolean(L, 0);
            lua_pushstring(L, "no such motion for this fighter");
            return 2;
        }
    }
    if (frame < 1) frame = 1;
    if (frame > 600) frame = 600;
    gs.step = 0; /* a new request replaces one still stepping */
    for (k = 0; k < n; ++k) {
        gs.set_motion[slots[k]] = motion + 1;
        gs.set_rate[slots[k]] = rate;
        gs.set_lift[slots[k]] = lift;
    }
    gs.paused = 1;
    gs.step = frame - 1; /* entering is frame 1 */
    lua_pushboolean(L, 1);
    return 1;
}

extern void gw_script_pad_mirror(int from, int to);
/* gd.mirror_pad(from, to) / gd.mirror_pad(): port `to` gets exactly what port `from` sends (for
   comparing two fighters under the same inputs; `to` must be a human slot). Offline, gameplay. */
static int l_mirror_pad(lua_State *L) {
    gs_require_offline(L, "mirror_pad");
    if (lua_isnoneornil(L, 1)) {
        gw_script_pad_mirror(-1, -1);
        return 0;
    }
    {
        int from = gs_slot_arg(L, 1), to = gs_slot_arg(L, 2);
        if (from > 3 || to > 3 || from == to) {
            luaL_error(L, "gd.mirror_pad: two different ports 1-4");
        }
        gw_script_pad_mirror(from, to);
    }
    return 0;
}

/* gd.lab_request([clear]) -> true when the frontend's LAB entry (or MELEE_LAB=1) asked for the
   Lab this session; `clear` resets it */
static int l_lab_request(lua_State *L) {
    int clear = lua_toboolean(L, 1); /* before the push: with no argument index 1 IS the push */
    lua_pushboolean(L, gs.lab_request);
    if (clear) gs.lab_request = 0;
    return 1;
}

/* LAB, the Lab's own game mode (pc/geno/geno_lab_mode.c) */
extern int gw_GenoLab_ModeActive(void);
extern int gw_GenoLab_Leave(int where);

/* gd.lab_mode() -> true while LAB is the running game mode (its match, its select screens) */
static int l_lab_mode(lua_State *L) {
    lua_pushboolean(L, gw_GenoLab_ModeActive() != 0);
    return 1;
}

/* gd.lab_leave("css" | "sss" | "menu" | "restart") -> true when a LAB match was ended (a no contest): back to
   LAB's character select, its stage select, or the menus. Offline only (the match's own end). */
static int l_lab_leave(lua_State *L) {
    const char *to = luaL_optstring(L, 1, "css");
    int where;
    gs_require_gameplay(L, "lab_leave");
    if (strcmp(to, "css") == 0) {
        where = 0;
    } else if (strcmp(to, "sss") == 0) {
        where = 1;
    } else if (strcmp(to, "menu") == 0) {
        where = 2;
    } else if (strcmp(to, "restart") == 0) {
        where = 3; /* the same match again */
    } else {
        return luaL_error(L, "gd.lab_leave: \"css\", \"sss\", \"menu\" or \"restart\" (got \"%s\")", to);
    }
    gs.paused = 0; /* the match has to run a frame to end */
    lua_pushboolean(L, gw_GenoLab_Leave(where) != 0);
    return 1;
}

static void gs_slot_capture(GsSaveSlot *s);
static void gs_slot_restore(const GsSaveSlot *s);
static int gs_ring_now(void);
static int gs_snap_ensure(int slots);
static void gs_rw_stop(void);
static int gs_rw_request(int target, const char **why);
static void gs_rw_branch(void);
static int gs_states_dir(char *out, size_t cap);
static int gs_state_check(const void *hdr, int len, char *why, size_t cap);
static int gs_exec(const char *line_in);

/* gd.history([frames [, interval]]) -> {depth, seconds, back, fwd, now, head, oldest, keys, mb,
   delta_mb, key_ms, last_ms, last_load_ms, last_frames, replaying, busy, ...}: with frames, keep that
   many frames of rewind (0 = off, at most a minute); a keyframe every `interval` frames (default
   30). Offline, gameplay. The cost is one MEM1 image plus the pages the game writes. */
static int l_history(lua_State *L) {
    int now;
    if (!lua_isnoneornil(L, 1)) {
        int want = (int) luaL_checkinteger(L, 1);
        int iv = (int) luaL_optinteger(L, 2, gs.rw_interval > 0 ? gs.rw_interval : GS_RW_INTERVAL);
        gs_require_offline(L, "history");
        if (want < 0) want = 0;
        if (want > GS_RW_MAX_FRAMES) want = GS_RW_MAX_FRAMES;
        if (iv < 1) iv = 1;
        if (iv > 600) iv = 600;
        gs.rw_interval = iv;
        if (want == 0) {
            gs_rw_stop();
        }
        gs.rw_frames = want;
    }
    now = gs_ring_now();
    lua_createtable(L, 0, 24);
    gs_setint(L, "depth", gs.rw_frames);
    gs_setnum(L, "seconds", gs.rw_frames / 60.0);
    gs_setint(L, "interval", gs.rw_interval > 0 ? gs.rw_interval : GS_RW_INTERVAL);
    gs_setint(L, "now", now);
    gs_setint(L, "back", gs.rw_began && now > gw_rw_base_tag() ? now - gw_rw_base_tag() : 0);
    gs_setint(L, "fwd", gs.rw_began && gs.rw_head > now ? gs.rw_head - now : 0);
    gs_setint(L, "head", gs.rw_began ? gs.rw_head : now);
    gs_setint(L, "oldest", gs.rw_began ? gw_rw_base_tag() : now);
    gs_setint(L, "keys", gw_rw_count());
    gs_setnum(L, "mb", gw_rw_bytes() / 1048576.0);
    gs_setnum(L, "delta_mb", gw_rw_delta_bytes() / 1048576.0);
    gs_setnum(L, "slot_mb", (double) gw_snap_slot_bytes() / 1048576.0);
    gs_setnum(L, "key_ms", gw_rw_ms_save_avg());
    gs_setnum(L, "last_ms", gs.rw_last_ms);
    gs_setnum(L, "last_load_ms", gs.rw_last_load_ms);
    gs_setint(L, "last_frames", gs.rw_last_frames);
    gs_setint(L, "last_pages", (lua_Integer) gw_rw_last_load_pages());
    gs_setbool(L, "replaying", gs.rw_began && now < gs.rw_head);
    gs_setbool(L, "busy", gs.rw_resim > 0 || gs.rw_resim_run > 0 || gs.rw_pending_on);
    return 1;
}

static int gs_push_fail(lua_State *L, const char *why) {
    lua_pushboolean(L, 0);
    lua_pushstring(L, why);
    return 2;
}

/* gd.step_back([n]) -> true | false, why: go back n frames and stay paused. Offline, gameplay.
   The newest keyframe at or before the target is loaded and the frames after it re-simulated on the
   logged inputs (silently, in one tick). */
static int l_step_back(lua_State *L) {
    int n = (int) luaL_optinteger(L, 1, 1);
    const char *why = NULL;
    gs_require_offline(L, "step_back");
    if (n < 1) n = 1;
    if (gs_rw_request(gs_ring_now() - n, &why) != 0) {
        return gs_push_fail(L, why);
    }
    lua_pushboolean(L, 1);
    return 1;
}

/* gd.rewind_to(frame) -> true | false, why: any frame from gd.history().oldest to .head (forward
   too, while the log still has the frames after "now"). Offline, gameplay; stays paused. */
static int l_rewind_to(lua_State *L) {
    int f = (int) luaL_checkinteger(L, 1);
    const char *why = NULL;
    gs_require_offline(L, "rewind_to");
    if (gs_rw_request(f, &why) != 0) {
        return gs_push_fail(L, why);
    }
    lua_pushboolean(L, 1);
    return 1;
}

/* gd.rewind_live() -> true: stop replaying the log here; the pads play from this frame on (the
   logged frames after it are dropped). Offline, gameplay. */
static int l_rewind_live(lua_State *L) {
    gs_require_offline(L, "rewind_live");
    gs_rw_branch();
    lua_pushboolean(L, 1);
    return 1;
}

/* gd.rewind_test([frames [, keep_running]]) -> true | false, why: the exactness self-test. Copies
   the whole state at the next frame, runs `frames` more (default 90), rewinds to the copied frame
   through the keyframes and the input log, and compares every byte. The result:
   gd.rewind_test_result(). Offline, gameplay; unpauses to run, pauses again after unless
   keep_running. */
static int l_rewind_test(lua_State *L) {
    int n = (int) luaL_optinteger(L, 1, 90);
    gs_require_offline(L, "rewind_test");
    if (!gs.rw_began || gs.rw_frames <= 0) {
        return gs_push_fail(L, "history is off (gd.history(n) turns it on)");
    }
    if (n < 1 || n > gs.rw_frames - gs.rw_interval) {
        return gs_push_fail(L, "frames: 1 .. the history window minus one keyframe interval");
    }
    gs.rwt_phase = 1;
    gs.rwt_back = n;
    gs.rwt_keep = lua_toboolean(L, 2);
    gs.rwt_arm = 0;
    gs.rwt_pass = -1;
    snprintf(gs.rwt_text, sizeof gs.rwt_text, "running");
    gs.paused = 0;
    gs.step = 0;
    lua_pushboolean(L, 1);
    return 1;
}

/* gd.rewind_test_result() -> {phase (0 = done), pass (true/false/nil), diff, diff_compared, text} */
static int l_rewind_test_result(lua_State *L) {
    lua_createtable(L, 0, 5);
    gs_setint(L, "phase", gs.rwt_phase);
    if (gs.rwt_pass >= 0) {
        gs_setbool(L, "pass", gs.rwt_pass);
    }
    gs_setnum(L, "diff", gs.rwt_diff[0]);
    gs_setnum(L, "diff_compared", gs.rwt_diff[1]);
    gs_setstr(L, "text", gs.rwt_text);
    return 1;
}

/* gd.hot_reload([seconds]) -> true, start | false, why: save where you are, rewind `seconds`
   (default 2) through the history, reload the fighters' geno.json (attributes, parameters, states,
   subaction overlays and their words files) and the Lab script, re-apply them to the fighters, and
   replay the logged input from there, so an edit is checked on the same inputs at once. When the
   reload changed the layout (a fighter's state or overlay count, the profile set) the state cannot
   take it: the match restarts instead. The outcome: gd.hot_reload_status(), and on_hot_reload(ok).
   Offline, gameplay. */
static int l_hot_reload(lua_State *L) {
    double sec = luaL_optnumber(L, 1, 2.0);
    const char *why = NULL;
    int now = gs_ring_now(), start;
    gs_require_offline(L, "hot_reload");
    if (gs.hot_phase != 0) {
        return gs_push_fail(L, "a reload is already running");
    }
    if (!gs.match_active) {
        return gs_push_fail(L, "not in a match");
    }
    start = now - (int) (sec * 60.0 + 0.5);
    if (!gs.rw_began || sec <= 0.0) {
        start = now; /* no history: reload in place */
        gs.hot_start = now;
        gs.hot_phase = 2;
        gs.paused = 1;
        gs.step = 0;
    } else {
        if (start < gw_rw_base_tag()) start = gw_rw_base_tag();
        if (start < gs.rw_log_start) start = gs.rw_log_start;
        if (gs_rw_request(start, &why) != 0) {
            return gs_push_fail(L, why);
        }
        gs.hot_start = start;
        gs.hot_phase = 1;
    }
    snprintf(gs.hot_text, sizeof gs.hot_text, "reloading");
    lua_pushboolean(L, 1);
    lua_pushinteger(L, start);
    return 2;
}

/* gd.lab_peek(addr [, n]) -> hex string: n (<= 64) raw bytes of MEM1 at addr, as memory holds them
   (game memory is big-endian). Read-only, for the Lab's debugging. */
static int l_lab_peek(lua_State *L) {
    lua_Integer a = luaL_checkinteger(L, 1);
    int n = (int) luaL_optinteger(L, 2, 16), i;
    char out[140];
    if (n < 1) n = 1;
    if (n > 64) n = 64;
    if (a < 0x80000000LL || a + n > 0x80000000LL + (lua_Integer) gw_mem1_size) {
        lua_pushnil(L);
        return 1;
    }
    for (i = 0; i < n; ++i) {
        snprintf(out + i * 2, 3, "%02X", ((const unsigned char *) (uintptr_t) a)[i]);
    }
    lua_pushstring(L, out);
    return 1;
}

/* gd.hot_reload_status() -> {phase (0 = idle), ok, text} */
static int l_hot_reload_status(lua_State *L) {
    lua_createtable(L, 0, 3);
    gs_setint(L, "phase", gs.hot_phase);
    gs_setbool(L, "ok", gs.hot_ok);
    gs_setstr(L, "text", gs.hot_text);
    return 1;
}

/* ---- the persistent state library (gd.state_*) -------------------------------------------------
 * One file per state in the Lab's saved data (scripts-data/<script>/states/*.gdst) plus index.txt
 * (a readable list, rewritten on every change; the files are the truth). A state is refused -
 * before a byte of it is loaded - when its header does not match this exe, disc, mods and Geno
 * data, or when the match running is not the one it was saved in (fighters, costumes, stage). */
#define GS_ST_MAGIC "GDLABHD1"
typedef struct {
    char magic[8];
    uint32_t size, geno_version;
    uint64_t exe_hash, disc_hash, mods_hash, geno_hash;
    int32_t match_frame, stage;
    int32_t present[6], chr[6], costume[6], cpu[6];
    int64_t unix_time;
    char name[64];
    char desc[96];
    GsSaveSlot slot;
} GsStateHdr;
static int gs_st_gen = 1; /* bumped on every change to the library, so a menu knows to re-list */

static void gs_identity(uint64_t out[4]) {
    static uint64_t exe, disc;
    static int done;
    uint64_t h;
    int i, n;
    if (!done) {
        char path[MAX_PATH];
        const char *iso = gw_iso_path();
        DWORD len = GetModuleFileNameA(NULL, path, sizeof path);
        exe = len > 0 && len < sizeof path ? gw_net_hash_file(path, 0) : 0;
        disc = iso != NULL && iso[0] != '\0' ? gw_net_hash_file(iso, 4u << 20) : 0;
        if (iso != NULL && iso[0] != '\0') {
            WIN32_FILE_ATTRIBUTE_DATA fa;
            if (GetFileAttributesExA(iso, GetFileExInfoStandard, &fa)) {
                disc ^= ((uint64_t) fa.nFileSizeHigh << 32 | fa.nFileSizeLow) * 1099511628211ull;
            }
        }
        done = 1;
    }
    out[0] = exe;
    out[1] = disc;
    {
        const char *m = gw_Mods_Describe();
        out[2] = gs_fnv(14695981039346656037ull, m != NULL ? m : "", m != NULL ? strlen(m) : 0);
    }
    h = gs_fnv(14695981039346656037ull, "geno", 4);
    n = gw_Geno_ProfileCount();
    for (i = 0; i < n; ++i) {
        const char *x = gw_Geno_ProfileHex(i);
        h = gs_fnv(h, x, strlen(x));
    }
    out[3] = h;
}

static void gs_state_fill(GsStateHdr *h, const char *name, const char *desc) {
    uint64_t id[4];
    int s;
    memset(h, 0, sizeof *h);
    memcpy(h->magic, GS_ST_MAGIC, 8);
    h->size = sizeof *h;
    h->geno_version = GENO_VERSION;
    gs_identity(id);
    h->exe_hash = id[0];
    h->disc_hash = id[1];
    h->mods_hash = id[2];
    h->geno_hash = id[3];
    h->match_frame = gs.match_frame;
    h->stage = gw_ScriptGame_StageKind();
    for (s = 0; s < 6; ++s) {
        h->present[s] = gs_players_present(s);
        h->chr[s] = h->present[s] ? gw_ScriptGame_FighterI(s, SI_CHAR) : -1;
        h->costume[s] = h->present[s] ? gw_ScriptGame_FighterI(s, SI_COSTUME) : -1;
        h->cpu[s] = h->present[s] ? gw_ScriptGame_FighterI(s, SI_SLOT_TYPE) : -1;
    }
    h->unix_time = (int64_t) time(NULL);
    snprintf(h->name, sizeof h->name, "%s", name != NULL ? name : "");
    snprintf(h->desc, sizeof h->desc, "%s", desc != NULL ? desc : "");
    gs_slot_capture(&h->slot);
}

static int gs_state_check(const void *hdr, int len, char *why, size_t cap) {
    const GsStateHdr *h = (const GsStateHdr *) hdr;
    uint64_t id[4];
    int s;
    if (len != (int) sizeof *h || memcmp(h->magic, GS_ST_MAGIC, 8) != 0 || h->size != sizeof *h) {
        snprintf(why, cap, "not a state this Lab can read");
        return -1;
    }
    gs_identity(id);
    if (h->exe_hash != id[0]) {
        snprintf(why, cap, "saved by another build of the game");
        return -1;
    }
    if (h->disc_hash != id[1]) {
        snprintf(why, cap, "saved with another disc");
        return -1;
    }
    if (h->mods_hash != id[2]) {
        snprintf(why, cap, "saved with other mods");
        return -1;
    }
    if (h->geno_hash != id[3] || h->geno_version != GENO_VERSION) {
        snprintf(why, cap, "saved with other Geno fighter data");
        return -1;
    }
    if (!gs.match_active) {
        snprintf(why, cap, "load it during a match");
        return -1;
    }
    for (s = 0; s < 6; ++s) {
        int present = gs_players_present(s);
        if (present != h->present[s] ||
            (present && (gw_ScriptGame_FighterI(s, SI_CHAR) != h->chr[s] ||
                         gw_ScriptGame_FighterI(s, SI_COSTUME) != h->costume[s]))) {
            snprintf(why, cap, "this state is %s: start that match first", h->desc[0] ? h->desc : "another match");
            return -1;
        }
    }
    if (gw_ScriptGame_StageKind() != h->stage) {
        snprintf(why, cap, "this state is %s: start that match first", h->desc[0] ? h->desc : "another stage");
        return -1;
    }
    why[0] = '\0';
    return 0;
}

static int gs_states_dir(char *out, size_t cap) {
    GsScript *s = gs_cur_script();
    char *p;
    const char *id = (s != NULL && gs.cur != gs.console) ? s->id : "geno-lab/lab";
    snprintf(out, cap, "%s\\%s", gs.data_dir, id);
    for (p = out + strlen(gs.data_dir); *p != '\0'; ++p) {
        if (*p == '/') *p = '_';
    }
    CreateDirectoryA(gs.data_dir, NULL);
    CreateDirectoryA(out, NULL);
    strncat(out, "\\states", cap - strlen(out) - 1);
    CreateDirectoryA(out, NULL);
    return 0;
}

static int gs_state_file_ok(const char *f) {
    size_t n = strlen(f), i;
    if (n < 6 || n > 64 || _stricmp(f + n - 5, ".gdst") != 0) {
        return 0;
    }
    for (i = 0; i < n; ++i) {
        char c = f[i];
        if (!((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9') || c == '_' ||
              c == '-' || c == '.')) {
            return 0;
        }
    }
    return 1;
}

/* index.txt beside the states: file | frame | saved | name | what (tab separated) */
static void gs_state_index(const char *dir) {
    char pat[MAX_PATH], path[MAX_PATH];
    WIN32_FIND_DATAA fd;
    HANDLE hf;
    FILE *out;
    snprintf(path, sizeof path, "%s\\index.txt", dir);
    out = fopen(path, "w");
    if (out == NULL) {
        return;
    }
    fprintf(out, "# the Geno Lab's saved states (the .gdst files are the truth; this list is rewritten)\n");
    snprintf(pat, sizeof pat, "%s\\*.gdst", dir);
    hf = FindFirstFileA(pat, &fd);
    if (hf != INVALID_HANDLE_VALUE) {
        do {
            GsStateHdr h;
            char full[MAX_PATH];
            snprintf(full, sizeof full, "%s\\%s", dir, fd.cFileName);
            if (gw_snap_file_header(full, &h, sizeof h) == (int) sizeof h) {
                char when[32];
                time_t t = (time_t) h.unix_time;
                struct tm *tm = localtime(&t);
                if (tm != NULL) strftime(when, sizeof when, "%Y-%m-%d %H:%M", tm);
                else snprintf(when, sizeof when, "?");
                fprintf(out, "%s\tf%d\t%s\t%s\t%s\n", fd.cFileName, h.match_frame, when, h.name, h.desc);
            }
        } while (FindNextFileA(hf, &fd));
        FindClose(hf);
    }
    fclose(out);
}

/* gd.state_save([name [, what]]) -> file | false, why: the whole state to a new file in the Lab's
   library, at the next frame boundary (at once when paused). Offline, gameplay. */
static int l_state_save(lua_State *L) {
    const char *name = luaL_optstring(L, 1, "");
    const char *desc = luaL_optstring(L, 2, "");
    char dir[MAX_PATH], file[64];
    static int counter;
    gs_require_offline(L, "state_save");
    if (!gs.match_active) {
        return gs_push_fail(L, "not in a match");
    }
    if (gs.st_pending_save[0] != '\0') {
        return gs_push_fail(L, "a save is already waiting");
    }
    gs_states_dir(dir, sizeof dir);
    snprintf(file, sizeof file, "st_%lld_%03d.gdst", (long long) time(NULL), ++counter % 1000);
    snprintf(gs.st_pending_save, sizeof gs.st_pending_save, "%s\\%s", dir, file);
    snprintf(gs.st_save_name, sizeof gs.st_save_name, "%s", name);
    snprintf(gs.st_save_desc, sizeof gs.st_save_desc, "%s", desc);
    lua_pushstring(L, file);
    return 1;
}

typedef struct {
    char file[64];
    GsStateHdr h;
    int ok;
    char why[128];
} GsStateRow;

static int gs_state_row_cmp(const void *a, const void *b) {
    const GsStateRow *x = (const GsStateRow *) a, *y = (const GsStateRow *) b;
    if (x->h.unix_time != y->h.unix_time) return x->h.unix_time < y->h.unix_time ? 1 : -1;
    return strcmp(y->file, x->file);
}

/* gd.state_list() -> {{file, name, what, frame, time, saved, stage, fighters = {{port, char,
   costume, cpu}}, ok, why}, ...}, newest first. `ok` false = it would be refused now, `why` says
   why. gd.state_gen() changes whenever the library does. */
static int l_state_list(lua_State *L) {
    char dir[MAX_PATH], pat[MAX_PATH];
    WIN32_FIND_DATAA fd;
    HANDLE hf;
    GsStateRow *rows = NULL;
    int n = 0, cap = 0, i, s;
    gs_states_dir(dir, sizeof dir);
    snprintf(pat, sizeof pat, "%s\\*.gdst", dir);
    hf = FindFirstFileA(pat, &fd);
    if (hf != INVALID_HANDLE_VALUE) {
        do {
            char full[MAX_PATH];
            GsStateRow r;
            int len;
            if (!gs_state_file_ok(fd.cFileName)) continue;
            memset(&r, 0, sizeof r);
            snprintf(r.file, sizeof r.file, "%s", fd.cFileName);
            snprintf(full, sizeof full, "%s\\%s", dir, fd.cFileName);
            len = gw_snap_file_header(full, &r.h, sizeof r.h);
            r.ok = gs_state_check(&r.h, len, r.why, sizeof r.why) == 0;
            if (len != (int) sizeof r.h) memset(&r.h, 0, sizeof r.h);
            if (n == cap) {
                GsStateRow *nr = (GsStateRow *) realloc(rows, (size_t) (cap = cap ? cap * 2 : 16) * sizeof *rows);
                if (nr == NULL) break;
                rows = nr;
            }
            rows[n++] = r;
        } while (FindNextFileA(hf, &fd) && n < 512);
        FindClose(hf);
    }
    if (n > 1) qsort(rows, (size_t) n, sizeof *rows, gs_state_row_cmp);
    lua_createtable(L, n, 0);
    for (i = 0; i < n; ++i) {
        GsStateRow *r = &rows[i];
        char when[32];
        time_t t = (time_t) r->h.unix_time;
        struct tm *tm = localtime(&t);
        int k = 0;
        if (tm != NULL) strftime(when, sizeof when, "%Y-%m-%d %H:%M", tm);
        else snprintf(when, sizeof when, "?");
        lua_createtable(L, 0, 12);
        gs_setstr(L, "file", r->file);
        gs_setstr(L, "name", r->h.name);
        gs_setstr(L, "what", r->h.desc);
        gs_setint(L, "frame", r->h.match_frame);
        gs_setint(L, "time", (lua_Integer) r->h.unix_time);
        gs_setstr(L, "saved", when);
        gs_setint(L, "stage", r->h.stage);
        gs_setbool(L, "ok", r->ok);
        gs_setstr(L, "why", r->why);
        lua_createtable(L, 6, 0);
        for (s = 0; s < 6; ++s) {
            if (!r->h.present[s]) continue;
            lua_createtable(L, 0, 4);
            gs_setint(L, "port", s + 1);
            gs_setint(L, "char", r->h.chr[s]);
            gs_setstr(L, "char_name", gs_char_name(r->h.chr[s]));
            gs_setint(L, "costume", r->h.costume[s]);
            gs_setbool(L, "cpu", r->h.cpu[s] == 1);
            lua_rawseti(L, -2, ++k);
        }
        lua_setfield(L, -2, "fighters");
        lua_rawseti(L, -2, i + 1);
    }
    free(rows);
    return 1;
}

static int l_state_gen(lua_State *L) {
    lua_pushinteger(L, gs_st_gen);
    return 1;
}

/* gd.state_load(file) -> true | false, why: checked now (build, disc, mods, Geno data, and that this
   match is the state's match); loaded whole at the next frame boundary. Offline, gameplay. */
static int l_state_load(lua_State *L) {
    const char *file = luaL_checkstring(L, 1);
    char dir[MAX_PATH], path[MAX_PATH], why[160];
    GsStateHdr h;
    int len;
    gs_require_offline(L, "state_load");
    if (!gs_state_file_ok(file)) {
        return gs_push_fail(L, "not a state file name");
    }
    gs_states_dir(dir, sizeof dir);
    snprintf(path, sizeof path, "%s\\%s", dir, file);
    len = gw_snap_file_header(path, &h, sizeof h);
    if (len < 0) {
        return gs_push_fail(L, "no such state");
    }
    if (gs_state_check(&h, len, why, sizeof why) != 0) {
        return gs_push_fail(L, why);
    }
    snprintf(gs.st_pending_load, sizeof gs.st_pending_load, "%s", path);
    lua_pushboolean(L, 1);
    return 1;
}

/* gd.state_delete(file) -> true | false, why */
static int l_state_delete(lua_State *L) {
    const char *file = luaL_checkstring(L, 1);
    char dir[MAX_PATH], path[MAX_PATH];
    gs_require_offline(L, "state_delete");
    if (!gs_state_file_ok(file)) {
        return gs_push_fail(L, "not a state file name");
    }
    gs_states_dir(dir, sizeof dir);
    snprintf(path, sizeof path, "%s\\%s", dir, file);
    if (!DeleteFileA(path)) {
        return gs_push_fail(L, "could not delete it");
    }
    gs_st_gen++;
    gs_state_index(dir);
    lua_pushboolean(L, 1);
    return 1;
}

/* gd.state_rename(file, name) -> true | false, why */
static int l_state_rename(lua_State *L) {
    const char *file = luaL_checkstring(L, 1);
    const char *name = luaL_checkstring(L, 2);
    char dir[MAX_PATH], path[MAX_PATH];
    GsStateHdr h;
    gs_require_offline(L, "state_rename");
    if (!gs_state_file_ok(file)) {
        return gs_push_fail(L, "not a state file name");
    }
    gs_states_dir(dir, sizeof dir);
    snprintf(path, sizeof path, "%s\\%s", dir, file);
    if (gw_snap_file_header(path, &h, sizeof h) != (int) sizeof h) {
        return gs_push_fail(L, "not a state this Lab can read");
    }
    snprintf(h.name, sizeof h.name, "%s", name);
    if (gw_snap_file_rewrite_header(path, &h, sizeof h) != 0) {
        return gs_push_fail(L, "could not rename it");
    }
    gs_st_gen++;
    gs_state_index(dir);
    lua_pushboolean(L, 1);
    return 1;
}


/* ---- gd.kit: the frontend kit's fonts, palette, 9-slice panels, icons and rows (gw_kit.h) ------
 * Local drawing only: it builds quads for the overlay pass, reads no game state and writes none,
 * so it is allowed in every script, online included. */
static const char *gs_kit_mod_ui(void) {
    GsScript *s = gs_cur_script();
    return (s != NULL && s->ui_dir[0] != '\0') ? s->ui_dir : NULL;
}

/* A colour argument: 0xRRGGBBAA, a kit/mod token ("bone", "@face", "p1", "#rrggbb", "accent"),
 * {r=, g=, b=, a=}, or nil for `def`. */
static uint32_t gs_kit_colour(lua_State *L, int idx, uint32_t def) {
    uint32_t c;
    if (lua_type(L, idx) == LUA_TSTRING) {
        const char *tok = lua_tostring(L, idx);
        if (!gw_Kit_Colour(tok, gs_kit_mod_ui(), &c)) {
            luaL_error(L, "gd.kit: unknown colour \"%s\" (a kit palette name, @face, p1..p4, #rrggbb)", tok);
        }
        return c;
    }
    return gs_color_arg(L, idx, def);
}

static int gs_kit_record(int first, int added) {
    GwScriptDraw *d, *last;
    int nb = gs.ndraw[gs.build];
    if (added <= 0) return 0;
    last = nb > 0 ? &gs.draw[gs.build][nb - 1] : NULL;
    if (last != NULL && last->kind == GW_SDRAW_KIT && last->kq0 + last->kqn == first) {
        last->kqn += added; /* consecutive kit calls share one draw entry */
        return added;
    }
    d = gs_draw_new(GW_SDRAW_KIT);
    if (d != NULL) {
        d->kq0 = first;
        d->kqn = added;
    }
    return added;
}

static double gs_kit_optnum(lua_State *L, int t, const char *k, double def) {
    double v = def;
    if (!lua_istable(L, t)) return def;
    lua_getfield(L, t, k);
    if (lua_isnumber(L, -1)) v = lua_tonumber(L, -1);
    lua_pop(L, 1);
    return v;
}

static int gs_kit_optbool(lua_State *L, int t, const char *k, int def) {
    int v = def;
    if (!lua_istable(L, t)) return def;
    lua_getfield(L, t, k);
    if (!lua_isnil(L, -1)) v = lua_toboolean(L, -1);
    lua_pop(L, 1);
    return v;
}

/* opts[k] as a colour (pushes nothing); `def` when absent */
static uint32_t gs_kit_optcolour(lua_State *L, int t, const char *k, uint32_t def) {
    uint32_t c = def;
    if (!lua_istable(L, t)) return def;
    lua_getfield(L, t, k);
    if (!lua_isnil(L, -1)) c = gs_kit_colour(L, lua_gettop(L), def);
    lua_pop(L, 1);
    return c;
}

static int gs_kit_role_arg(lua_State *L, int idx, const char *def) {
    const char *name = luaL_optstring(L, idx, def);
    int r = gw_Kit_Role(name);
    if (r < 0) {
        luaL_error(L, "gd.kit: unknown text role \"%s\" (gd.kit.roles lists them)", name);
    }
    return r;
}

static int gs_kit_align_arg(lua_State *L, int idx) {
    const char *a = luaL_optstring(L, idx, "left");
    if (strcmp(a, "center") == 0 || strcmp(a, "centre") == 0) return GW_KIT_ALIGN_CENTER;
    if (strcmp(a, "right") == 0) return GW_KIT_ALIGN_RIGHT;
    return GW_KIT_ALIGN_LEFT;
}

static void gs_kit_need(lua_State *L) {
    if (!gw_Kit_Available()) {
        luaL_error(L, "gd.kit: the kit is not available (%s)", gw_Kit_Why());
    }
}

static int l_kit_available(lua_State *L) {
    lua_pushboolean(L, gw_Kit_Available());
    lua_pushstring(L, gw_Kit_Why());
    return 2;
}

/* gd.kit.text(x, baseline_y, text [, role [, colour [, align [, opts]]]]) -> width */
static int l_kit_text(lua_State *L) {
    float x = (float) luaL_checknumber(L, 1), y = (float) luaL_checknumber(L, 2), w = 0;
    const char *s;
    int role, align, first = gw_Kit_QuadCount();
    uint32_t c;
    gs_kit_need(L);
    lua_settop(L, 7);
    s = luaL_tolstring(L, 3, NULL);
    role = gs_kit_role_arg(L, 4, "body");
    c = gs_kit_colour(L, 5, 0xF2EFE4FFu);
    align = gs_kit_align_arg(L, 6);
    gs_kit_record(first, gw_Kit_DrawText(x, y, s, role, c, align, (float) gs_kit_optnum(L, 7, "max_w", 0),
                                         (float) gs_kit_optnum(L, 7, "shear", gw_Kit_Shear()), &w));
    lua_pushnumber(L, w);
    return 1;
}

/* gd.kit.measure(text [, role [, max_w]]) -> width, line height, the text as fitted */
static int l_kit_measure(lua_State *L) {
    char fit[512];
    const char *s;
    int role, k;
    float line = 0, max_w;
    gs_kit_need(L);
    lua_settop(L, 3);
    s = luaL_tolstring(L, 1, NULL);
    role = gs_kit_role_arg(L, 2, "body");
    max_w = (float) luaL_optnumber(L, 3, 0);
    role = gw_Kit_Fit(role, s, max_w, fit, sizeof fit);
    gw_Kit_RoleMetrics(role, NULL, NULL, NULL, NULL, &line);
    lua_pushnumber(L, gw_Kit_TextWidth(role, fit));
    lua_pushnumber(L, line);
    for (k = 0; fit[k] != '\0'; k++) {
        if (fit[k] == 0x01) fit[k] = '~'; /* the ellipsis byte, for display only */
    }
    lua_pushstring(L, fit);
    return 3;
}

/* gd.kit.metrics(role) -> {size, ascent, descent, cap, line} */
static int l_kit_metrics(lua_State *L) {
    float size, asc, desc, cap, line;
    int role;
    gs_kit_need(L);
    role = gs_kit_role_arg(L, 1, "body");
    gw_Kit_RoleMetrics(role, &size, &asc, &desc, &cap, &line);
    lua_createtable(L, 0, 5);
    gs_setnum(L, "size", size);
    gs_setnum(L, "ascent", asc);
    gs_setnum(L, "descent", desc);
    gs_setnum(L, "cap", cap);
    gs_setnum(L, "line", line);
    return 1;
}

/* gd.kit.texture(name) -> {w, h, w1x, h1x, mask, tint} or nil */
static int l_kit_texture(lua_State *L) {
    int t = gw_Kit_Tex(luaL_checkstring(L, 1), gs_kit_mod_ui()), w, h, mask;
    float w1, h1;
    const char *tint;
    if (!gw_Kit_TexInfo(t, &w, &h, &w1, &h1, &mask, &tint)) {
        lua_pushnil(L);
        return 1;
    }
    lua_createtable(L, 0, 6);
    gs_setint(L, "w", w);
    gs_setint(L, "h", h);
    gs_setnum(L, "w1x", w1);
    gs_setnum(L, "h1x", h1);
    gs_setbool(L, "mask", mask);
    gs_setstr(L, "tint", tint);
    return 1;
}

/* Draws texture `name` at (x, y); w/h default to its 1x size times opts.scale. */
static int gs_kit_image(lua_State *L, const char *name, float x, float y, int wi, int hi, int oi,
                        uint32_t def_tint) {
    int t = gw_Kit_Tex(name, gs_kit_mod_ui()), mask, flip = 0, first = gw_Kit_QuadCount();
    float w1, h1, w, h, scale;
    const char *tint_tok;
    uint32_t tint = def_tint;
    if (!gw_Kit_TexInfo(t, NULL, NULL, &w1, &h1, &mask, &tint_tok)) {
        lua_pushnil(L);
        return 1;
    }
    scale = (float) gs_kit_optnum(L, oi, "scale", 1.0);
    w = lua_isnumber(L, wi) ? (float) lua_tonumber(L, wi) : w1 * scale;
    h = lua_isnumber(L, hi) ? (float) lua_tonumber(L, hi) : h1 * scale;
    if (tint_tok != NULL && tint_tok[0] != '\0') {
        gw_Kit_Colour(tint_tok, gs_kit_mod_ui(), &tint); /* the manifest's tint */
    }
    tint = gs_kit_optcolour(L, oi, "tint", tint);
    if (gs_kit_optbool(L, oi, "flip_x", 0)) flip |= GW_KIT_FLIP_X;
    if (gs_kit_optbool(L, oi, "flip_y", 0)) flip |= GW_KIT_FLIP_Y;
    gs_kit_record(first, gw_Kit_DrawImage(t, x, y, w, h, tint, flip, (float) gs_kit_optnum(L, oi, "shear", 0)));
    lua_pushnumber(L, w);
    lua_pushnumber(L, h);
    return 2;
}

/* gd.kit.image(name, x, y [, w [, h [, opts]]]) -> w, h drawn (nil when there is no such texture) */
static int l_kit_image(lua_State *L) {
    const char *name = luaL_checkstring(L, 1);
    float x = (float) luaL_checknumber(L, 2), y = (float) luaL_checknumber(L, 3);
    lua_settop(L, 6);
    return gs_kit_image(L, name, x, y, 4, 5, 6, 0xFFFFFFFFu);
}

/* gd.kit.icon(name, x, y [, scale [, tint]]) -> w, h: the texture ico_<name>, bone by default */
static int l_kit_icon(lua_State *L) {
    char nm[80];
    float x = (float) luaL_checknumber(L, 2), y = (float) luaL_checknumber(L, 3);
    uint32_t bone = 0xF2EFE4FFu;
    snprintf(nm, sizeof nm, "ico_%s", luaL_checkstring(L, 1));
    lua_settop(L, 5);
    gw_Kit_Colour("bone", NULL, &bone);
    lua_createtable(L, 0, 2); /* index 6: the options gs_kit_image reads */
    lua_pushvalue(L, 4);
    lua_setfield(L, 6, "scale");
    lua_pushvalue(L, 5);
    lua_setfield(L, 6, "tint");
    lua_settop(L, 8); /* 7, 8: no explicit size */
    return gs_kit_image(L, nm, x, y, 7, 8, 6, bone);
}

/* gd.kit.panel(x, y, w, h [, style]) - style {prefix="frame", piece, tint, fill, shear} */
static int l_kit_panel(lua_State *L) {
    float x = (float) luaL_checknumber(L, 1), y = (float) luaL_checknumber(L, 2);
    float w = (float) luaL_checknumber(L, 3), h = (float) luaL_checknumber(L, 4);
    const char *prefix = "frame";
    uint32_t tint, fill = 0x032568E0u; /* the Versus section's bg, a little translucent */
    int first = gw_Kit_QuadCount();
    lua_settop(L, 5);
    gw_Kit_Colour("@bg", NULL, &fill);
    fill = (fill & 0xFFFFFF00u) | 0xE0u;
    if (lua_istable(L, 5)) {
        lua_getfield(L, 5, "prefix");
        if (lua_isstring(L, -1)) prefix = lua_tostring(L, -1); /* stays on the stack until return */
        lua_getfield(L, 5, "fill");
        if (lua_isboolean(L, -1) && !lua_toboolean(L, -1)) fill = 0;
        else if (!lua_isnil(L, -1)) fill = gs_kit_colour(L, lua_gettop(L), fill);
        lua_pop(L, 1);
    }
    tint = gs_kit_optcolour(L, 5, "tint", 0xFFFFFFFFu);
    gs_kit_record(first, gw_Kit_DrawPanel(x, y, w, h, prefix, gs_kit_mod_ui(), (float) gs_kit_optnum(L, 5, "piece", 0),
                                          tint, fill, (float) gs_kit_optnum(L, 5, "shear", 0)));
    return 0;
}

static int gs_kit_state_arg(lua_State *L, int idx) {
    if (lua_isboolean(L, idx)) return lua_toboolean(L, idx) ? GW_KIT_ROW_SEL : GW_KIT_ROW_NG;
    if (lua_type(L, idx) == LUA_TSTRING) {
        const char *s = lua_tostring(L, idx);
        if (strcmp(s, "sel") == 0 || strcmp(s, "selected") == 0) return GW_KIT_ROW_SEL;
        if (strcmp(s, "disabled") == 0) return GW_KIT_ROW_DISABLED;
    }
    return GW_KIT_ROW_NG;
}

static int gs_kit_section_arg(lua_State *L, int t) {
    int i;
    const char *s = NULL;
    if (lua_istable(L, t)) {
        lua_getfield(L, t, "section");
        s = lua_tostring(L, -1);
        lua_pop(L, 1); /* the string stays alive in the table */
    }
    for (i = 0; s != NULL && i < gw_Kit_SectionCount(); i++) {
        if (strcmp(gw_Kit_SectionName(i), s) == 0) return i;
    }
    return -1;
}

/* gd.kit.button(x, y, w, label [, state [, opts]]) -> h. state: true/"sel", false/"ng",
 * "disabled"; opts {value, h, shear, section} */
static int l_kit_button(lua_State *L) {
    float x = (float) luaL_checknumber(L, 1), y = (float) luaL_checknumber(L, 2);
    float w = (float) luaL_checknumber(L, 3), h, rh;
    const char *label, *value = NULL;
    int first = gw_Kit_QuadCount();
    gs_kit_need(L);
    lua_settop(L, 6);
    label = luaL_tolstring(L, 4, NULL); /* index 7 */
    if (lua_istable(L, 6)) {
        lua_getfield(L, 6, "value"); /* index 8 */
        if (!lua_isnil(L, -1)) value = luaL_tolstring(L, -1, NULL);
    }
    gw_Kit_RowMetrics(&rh, NULL, NULL, NULL, NULL);
    h = (float) gs_kit_optnum(L, 6, "h", rh);
    gs_kit_record(first, gw_Kit_DrawRow(x, y, w, h, label, value, gs_kit_state_arg(L, 5), gs_kit_section_arg(L, 6),
                                        (float) gs_kit_optnum(L, 6, "shear", gw_Kit_Shear())));
    lua_pushnumber(L, h);
    return 1;
}

/* gd.kit.list(x, y, w, items, selected [, opts]) -> height. items: strings or {label, value,
 * disabled}; selected: 1-based (0/nil = none); opts {pitch, h, shear, section, first, visible} */
static int l_kit_list(lua_State *L) {
    float x = (float) luaL_checknumber(L, 1), y = (float) luaL_checknumber(L, 2);
    float w = (float) luaL_checknumber(L, 3), rh, pitch, h;
    int sel, n, i, firsti, visible, shown = 0, section;
    float shear;
    gs_kit_need(L);
    luaL_checktype(L, 4, LUA_TTABLE);
    lua_settop(L, 6);
    sel = (int) luaL_optinteger(L, 5, 0);
    gw_Kit_RowMetrics(&rh, &pitch, NULL, NULL, NULL);
    h = (float) gs_kit_optnum(L, 6, "h", rh);
    pitch = (float) gs_kit_optnum(L, 6, "pitch", pitch + (h - rh));
    shear = (float) gs_kit_optnum(L, 6, "shear", gw_Kit_Shear());
    firsti = (int) gs_kit_optnum(L, 6, "first", 1);
    visible = (int) gs_kit_optnum(L, 6, "visible", 64);
    section = gs_kit_section_arg(L, 6);
    n = (int) lua_rawlen(L, 4);
    for (i = firsti < 1 ? 1 : firsti; i <= n && shown < visible; i++, shown++) {
        const char *label = NULL, *value = NULL;
        int state = i == sel ? GW_KIT_ROW_SEL : GW_KIT_ROW_NG, first = gw_Kit_QuadCount(), top = lua_gettop(L);
        lua_rawgeti(L, 4, i);
        if (lua_istable(L, -1)) {
            int it = lua_gettop(L);
            lua_getfield(L, it, "label");
            label = lua_isnil(L, -1) ? "" : luaL_tolstring(L, -1, NULL);
            lua_getfield(L, it, "value");
            if (!lua_isnil(L, -1)) value = luaL_tolstring(L, -1, NULL);
            lua_getfield(L, it, "disabled");
            if (lua_toboolean(L, -1) && state != GW_KIT_ROW_SEL) state = GW_KIT_ROW_DISABLED;
        } else {
            label = luaL_tolstring(L, -1, NULL);
        }
        gs_kit_record(first, gw_Kit_DrawRow(x, y + pitch * shown, w, h, label, value, state, section, shear));
        lua_settop(L, top);
    }
    lua_pushnumber(L, shown > 0 ? pitch * (shown - 1) + h : 0);
    return 1;
}

/* gd.kit.color(token) -> 0xRRGGBBAA or nil (the calling mod's palette names included) */
static int l_kit_color(lua_State *L) {
    uint32_t c;
    if (!gw_Kit_Colour(luaL_checkstring(L, 1), gs_kit_mod_ui(), &c)) {
        lua_pushnil(L);
        return 1;
    }
    lua_pushinteger(L, (lua_Integer) c);
    return 1;
}

/* ---- Stage E: knockback and launch preview ----------------------------------------------------
 * The knockback value comes from the game's own functions (script_game.c ScriptGame_LabKnockback:
 * ftColl_80079AB0 then ftCo_Damage_CalcKnockback); the level from ftCo_8008D8E8. The angle
 * (ftCo_Damage_CalcAngle's Sakurai-angle rule), the trajectory DI (ftCo_8008E5A4), the launch speed
 * (ftCo_8008DCE0) and the flight (the DamageFly physics: gravity / terminal velocity / aerial
 * friction on the fighter's own velocity, fighter.c's knockback decay on the knockback velocity)
 * are the same expressions with the loaded PlCo constants, read through ScriptGame_LabCommonF. */
extern float gw_ScriptGame_LabKnockback(int slot, int attacker_slot, int dmg_bits, int kbg, int wbk, int bkb,
                                        int pct_bits, int post);
extern int gw_ScriptGame_LabKbLevel(int kb_bits);
extern float gw_ScriptGame_LabCommonF(int which);
extern float gw_ScriptGame_LabFlightF(int slot, int which);

static int gs_fbits(float f) {
    union {
        float f;
        int i;
    } u;
    u.f = f;
    return u.i;
}
static double gs_optnum_field(lua_State *L, int t, const char *k, double def) {
    double v = def;
    lua_getfield(L, t, k);
    if (lua_isnumber(L, -1)) v = lua_tonumber(L, -1);
    lua_pop(L, 1);
    return v;
}

/* the trajectory DI of ftCo_8008E5A4 for a stick (sx, sy) on a knockback velocity */
static void gs_apply_di(float *vx, float *vy, float sx, float sy, float di_deg) {
    float kx = *vx, ky = *vy, mag2 = kx * kx + ky * ky, f3, f30, a, mag;
    if ((sx == 0.0f && sy == 0.0f) || mag2 < 0.00001f) return;
    f3 = ky * sx + (-kx) * sy;
    f30 = f3 * f3 / mag2;
    if (kx * sy - ky * sx < 0.0f) f30 = -f30;
    a = atan2f(ky, kx) + di_deg * 3.14159265f / 180.0f * f30;
    mag = sqrtf(mag2);
    *vx = mag * cosf(a);
    *vy = mag * sinf(a);
}

/* gd.kb_preview(victim_port, {damage, angle, kbg, bkb, wbk, attacker, dir, di, percent, x, y,
   extra}) -> {kb, level, tumble, hitstun, angle, angle_di, vx, vy, percent, weight, points =
   {{x, y}, ...} (one per hitstun frame), blast = {x, y, frame, side, after_hitstun} | nil,
   zones = {left, right, top, bottom}}.
   `dir` +1 launches right, -1 left (default: away from the attacker, else behind the victim's
   facing); `di` "none" | "in" | "out" | "survival" | {x, y} (a stick); `percent` overrides the
   victim's (before the hit; the hit's damage is added as the game does); `extra` frames past the
   hitstun are still checked against the blast zones. Offline (the game half sets the percent for
   the call and restores it bit for bit). */
static int l_kb_preview(lua_State *L) {
    int slot = gs_present_arg(L, 1), att = -1, angle, kbg, bkb, wbk, level, hitstun, f, extra, npts = 0;
    float dmg, pct, kb, a, vx, vy, sx = 0, sy = 0, px, py, svx = 0, svy = 0, g, term, fric, decay;
    float left, right, top, bottom, dir, a_di;
    const char *di = "none";
    luaL_checktype(L, 2, LUA_TTABLE);
    gs_require_offline(L, "kb_preview");
    if (slot < 0) {
        lua_pushnil(L);
        lua_pushstring(L, "no fighter on that port");
        return 2;
    }
    lua_getfield(L, 2, "attacker");
    if (lua_isinteger(L, -1)) att = gs_present_arg(L, lua_gettop(L));
    lua_pop(L, 1);
    dmg = (float) gs_optnum_field(L, 2, "damage", 10);
    angle = (int) gs_optnum_field(L, 2, "angle", 45);
    kbg = (int) gs_optnum_field(L, 2, "kbg", 100);
    bkb = (int) gs_optnum_field(L, 2, "bkb", 0);
    wbk = (int) gs_optnum_field(L, 2, "wbk", 0);
    pct = (float) gs_optnum_field(L, 2, "percent", -1);
    extra = (int) gs_optnum_field(L, 2, "extra", 90);
    px = (float) gs_optnum_field(L, 2, "x", gw_ScriptGame_FighterF(slot, SF_X));
    py = (float) gs_optnum_field(L, 2, "y", gw_ScriptGame_FighterF(slot, SF_Y));
    {
        float def = -gw_ScriptGame_LabFlightF(slot, 3);
        if (att >= 0) def = gw_ScriptGame_FighterF(slot, SF_X) >= gw_ScriptGame_FighterF(att, SF_X) ? 1.0f : -1.0f;
        dir = (float) gs_optnum_field(L, 2, "dir", def) >= 0 ? 1.0f : -1.0f;
    }
    kb = gw_ScriptGame_LabKnockback(slot, att, gs_fbits(dmg), kbg, wbk, bkb, gs_fbits(pct), 1);
    level = gw_ScriptGame_LabKbLevel(gs_fbits(kb));
    hitstun = (int) (kb * gw_ScriptGame_LabCommonF(LAB_C_HITSTUN_MUL));
    if (hitstun < 1) hitstun = 1;
    /* ftCo_Damage_CalcAngle */
    if (angle != 361) {
        a = (float) angle * 3.14159265f / 180.0f;
    } else if (gw_ScriptGame_LabFlightF(slot, 4) == 0.0f) {
        a = gw_ScriptGame_LabCommonF(LAB_C_ANGLE_AIR_361);
    } else if (kb < gw_ScriptGame_LabCommonF(LAB_C_ANGLE_GROUND_KB0)) {
        a = 0.0f;
    } else {
        float k0 = gw_ScriptGame_LabCommonF(LAB_C_ANGLE_GROUND_KB0);
        float k1 = gw_ScriptGame_LabCommonF(LAB_C_ANGLE_GROUND_KB1);
        float mx = gw_ScriptGame_LabCommonF(LAB_C_ANGLE_GROUND_MAX);
        a = (mx * ((kb - k0) / (k1 - k0)) + 1.0f) * 3.14159265f / 180.0f;
        if (a > mx * 3.14159265f / 180.0f) a = mx * 3.14159265f / 180.0f;
    }
    /* ftCo_8008DCE0: speed = kb * x100; x goes against the victim's facing (it faces the hit) */
    vx = kb * gw_ScriptGame_LabCommonF(LAB_C_KB_SPEED) * cosf(a) * dir;
    vy = kb * gw_ScriptGame_LabCommonF(LAB_C_KB_SPEED) * sinf(a);
    /* the stick */
    lua_getfield(L, 2, "di");
    if (lua_istable(L, -1)) {
        di = "stick";
        sx = (float) gs_optnum_field(L, lua_gettop(L), "x", 0);
        sy = (float) gs_optnum_field(L, lua_gettop(L), "y", 0);
    } else if (lua_type(L, -1) == LUA_TSTRING) {
        di = lua_tostring(L, -1);
    }
    {
        float t = atan2f(vy, vx);
        float ccx = -sinf(t), ccy = cosf(t); /* the perpendicular that turns the launch CCW */
        float di_deg = gw_ScriptGame_LabCommonF(LAB_C_DI_DEGREES);
        if (strcmp(di, "in") == 0 || strcmp(di, "out") == 0) {
            /* "in": the perpendicular pointing back toward where the launch came from */
            int toward = (ccx * dir < 0.0f) == (strcmp(di, "in") == 0);
            sx = toward ? ccx : -ccx;
            sy = toward ? ccy : -ccy;
        } else if (strcmp(di, "survival") == 0) {
            /* the perpendicular whose result lands nearer the diagonal (45 / 135 degrees) */
            float ax = vx, ay = vy, bx = vx, by = vy, goal = dir > 0 ? 0.785398f : 2.356194f;
            gs_apply_di(&ax, &ay, ccx, ccy, di_deg);
            gs_apply_di(&bx, &by, -ccx, -ccy, di_deg);
            if (fabsf(atan2f(ay, ax) - goal) <= fabsf(atan2f(by, bx) - goal)) {
                sx = ccx, sy = ccy;
            } else {
                sx = -ccx, sy = -ccy;
            }
        }
        gs_apply_di(&vx, &vy, sx, sy, di_deg);
    }
    lua_pop(L, 1);
    a_di = atan2f(vy, vx);
    g = gw_ScriptGame_LabFlightF(slot, 0);
    term = gw_ScriptGame_LabFlightF(slot, 1);
    fric = gw_ScriptGame_LabFlightF(slot, 2);
    decay = gw_ScriptGame_LabCommonF(LAB_C_KB_DECAY);
    left = gw_ScriptGame_LabCommonF(LAB_C_BLAST_LEFT);
    right = gw_ScriptGame_LabCommonF(LAB_C_BLAST_RIGHT);
    top = gw_ScriptGame_LabCommonF(LAB_C_BLAST_TOP);
    bottom = gw_ScriptGame_LabCommonF(LAB_C_BLAST_BOTTOM);
    lua_createtable(L, 0, 16);
    gs_setnum(L, "kb", kb);
    gs_setint(L, "level", level);
    gs_setbool(L, "tumble", level >= 3);
    gs_setint(L, "hitstun", hitstun);
    gs_setnum(L, "angle", a * 180.0 / 3.14159265);
    gs_setnum(L, "angle_di", a_di * 180.0 / 3.14159265);
    gs_setnum(L, "vx", vx);
    gs_setnum(L, "vy", vy);
    gs_setstr(L, "di", di);
    gs_setnum(L, "percent", pct >= 0 ? pct : gw_ScriptGame_LabFlightF(slot, 6));
    gs_setnum(L, "weight", gw_ScriptGame_LabFlightF(slot, 5));
    lua_createtable(L, 0, 4);
    gs_setnum(L, "left", left);
    gs_setnum(L, "right", right);
    gs_setnum(L, "top", top);
    gs_setnum(L, "bottom", bottom);
    lua_setfield(L, -2, "zones");
    lua_createtable(L, hitstun, 0);
    for (f = 1; f <= hitstun + extra && f <= 1200; ++f) {
        float m;
        /* the DamageFly physics on the fighter's own velocity (ft_80084EEC) */
        svy -= g;
        if (svy < -term) svy = -term;
        if (svx > 0.0f) {
            svx -= fric;
            if (svx < 0.0f) svx = 0.0f;
        } else if (svx < 0.0f) {
            svx += fric;
            if (svx > 0.0f) svx = 0.0f;
        }
        /* fighter.c: the knockback velocity decays along itself */
        m = sqrtf(vx * vx + vy * vy);
        if (m < decay) {
            vx = vy = 0.0f;
        } else {
            float t = atan2f(vy, vx);
            vx -= decay * cosf(t);
            vy -= decay * sinf(t);
        }
        px += svx + vx;
        py += svy + vy;
        if (f <= hitstun) {
            lua_createtable(L, 2, 0);
            lua_pushnumber(L, px);
            lua_rawseti(L, -2, 1);
            lua_pushnumber(L, py);
            lua_rawseti(L, -2, 2);
            lua_rawseti(L, -2, ++npts);
        }
        if (px < left || px > right || py > top || py < bottom) {
            lua_setfield(L, -2, "points");
            lua_createtable(L, 0, 5);
            gs_setnum(L, "x", px);
            gs_setnum(L, "y", py);
            gs_setint(L, "frame", f);
            gs_setstr(L, "side", px < left ? "left" : px > right ? "right" : py > top ? "top" : "bottom");
            gs_setbool(L, "after_hitstun", f > hitstun);
            lua_setfield(L, -2, "blast");
            return 1;
        }
        if (f >= hitstun && vx == 0.0f && vy == 0.0f) {
            break; /* hitstun over and the launch spent: from here the fighter flies itself home */
        }
    }
    lua_setfield(L, -2, "points");
    return 1;
}

/* ---- Stage E: the rollback visualiser (gw_snap.c's ring) --------------------------------------- */
extern int gw_RbViz_Total(void);
extern int gw_RbViz_Get(int back, int *frame, int *first, int *depth, int *cause, int *kind, int *mismatch,
                        float *ms);
extern const char *gw_RbViz_FirstMismatch(int *frame, int *count, uint32_t *va, int *was, int *now);
extern void gw_RbViz_Reset(void);

/* gd.rollbacks([n]) -> {total, list = {{frame, first, depth, cause (port; 0 = SyncTest's own), kind
   ("synctest" | "fake" | "netplay"), mismatch, ms}, ...} newest first (n, default 120),
   mismatch = {frame, count, where, va, was, now} | nil}. Read-only; works in any session. */
static int l_rollbacks(lua_State *L) {
    static const char *const kinds[] = {"?", "synctest", "fake", "netplay"};
    int n = (int) luaL_optinteger(L, 1, 120), i, fr, first, depth, cause, kind, mm, mf, mc, was, now;
    float ms;
    uint32_t va;
    const char *where;
    if (n > 600) n = 600;
    if (n < 0) n = 0;
    lua_createtable(L, 0, 3);
    gs_setint(L, "total", gw_RbViz_Total());
    lua_createtable(L, n, 0);
    for (i = 0; i < n && gw_RbViz_Get(i, &fr, &first, &depth, &cause, &kind, &mm, &ms); ++i) {
        lua_createtable(L, 0, 7);
        gs_setint(L, "frame", fr);
        gs_setint(L, "first", first);
        gs_setint(L, "depth", depth);
        gs_setint(L, "cause", cause + 1);
        gs_setstr(L, "kind", kind >= 0 && kind <= 3 ? kinds[kind] : "?");
        gs_setbool(L, "mismatch", mm);
        gs_setnum(L, "ms", ms);
        lua_rawseti(L, -2, i + 1);
    }
    lua_setfield(L, -2, "list");
    where = gw_RbViz_FirstMismatch(&mf, &mc, &va, &was, &now);
    if (mf >= 0) {
        lua_createtable(L, 0, 6);
        gs_setint(L, "frame", mf);
        gs_setint(L, "count", mc);
        gs_setstr(L, "where", where);
        gs_setint(L, "va", (lua_Integer) va);
        gs_setint(L, "was", was);
        gs_setint(L, "now", now);
        lua_setfield(L, -2, "mismatch");
    }
    return 1;
}
static int l_rollbacks_clear(lua_State *L) {
    (void) L;
    gw_RbViz_Reset();
    return 0;
}

/* gd.lab_now([long]) -> the local time, "20260924-153000" (a version tag) or, with long,
   "2026-09-24 15:30:00" (the sandbox has no os.date) */
static int l_lab_now(lua_State *L) {
    SYSTEMTIME t;
    char b[32];
    GetLocalTime(&t);
    if (lua_toboolean(L, 1)) {
        snprintf(b, sizeof b, "%04u-%02u-%02u %02u:%02u:%02u", t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute, t.wSecond);
    } else {
        snprintf(b, sizeof b, "%04u%02u%02u-%02u%02u%02u", t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute, t.wSecond);
    }
    lua_pushstring(L, b);
    return 1;
}

/* gd.lab_common() -> the PlCo constants the Lab's training readouts use (stage D), as loaded:
   lcancel_window (an L-cancel needs player.lr_age below it at landing), lcancel_div (the landing
   lag is divided by it), hitstun_mul, kb_speed, kb_decay. Read-only. */
static int l_lab_common(lua_State *L) {
    lua_createtable(L, 0, 5);
    gs_setnum(L, "lcancel_window", gw_ScriptGame_LabCommonF(LAB_C_LCANCEL_WINDOW));
    gs_setnum(L, "lcancel_div", gw_ScriptGame_LabCommonF(LAB_C_LCANCEL_DIV));
    gs_setnum(L, "hitstun_mul", gw_ScriptGame_LabCommonF(LAB_C_HITSTUN_MUL));
    gs_setnum(L, "kb_speed", gw_ScriptGame_LabCommonF(LAB_C_KB_SPEED));
    gs_setnum(L, "kb_decay", gw_ScriptGame_LabCommonF(LAB_C_KB_DECAY));
    return 1;
}

/* gd.lab_env(name) -> the value of the environment variable MELEE_LAB_<name>, or nil (only that
   family: the Lab's launch switches, e.g. MELEE_LAB_BATCH for the headless frame-data export) */
static int l_lab_env(lua_State *L) {
    char k[64];
    const char *v;
    snprintf(k, sizeof k, "MELEE_LAB_%s", luaL_checkstring(L, 1));
    v = getenv(k);
    if (v == NULL || v[0] == '\0') {
        lua_pushnil(L);
    } else {
        lua_pushstring(L, v);
    }
    return 1;
}

static const luaL_Reg gs_kit_funcs[] = {
    {"available", l_kit_available}, {"text", l_kit_text}, {"measure", l_kit_measure},
    {"metrics", l_kit_metrics}, {"texture", l_kit_texture}, {"image", l_kit_image},
    {"icon", l_kit_icon}, {"panel", l_kit_panel}, {"button", l_kit_button}, {"list", l_kit_list},
    {"color", l_kit_color}, {NULL, NULL}};

/* gd.kit: the functions, and the kit's data as tables (colors, roles, row, shear). */
static void gs_push_kit(lua_State *L) {
    int i, k;
    static const char *const which[4] = {"face", "bg", "band", "face_hi"};
    static const char *const ports[5] = {"p1", "p2", "p3", "p4", "cpu"};
    float rh = 0, pitch = 0, lx = 0, lb = 0, lift = 0;
    lua_newtable(L);
    luaL_setfuncs(L, gs_kit_funcs, 0);
    lua_newtable(L); /* colors */
    for (i = 0; i < gw_Kit_PaletteCount(); i++) {
        lua_pushinteger(L, (lua_Integer) gw_Kit_PaletteRGBA(i));
        lua_setfield(L, -2, gw_Kit_PaletteName(i));
    }
    for (i = 0; i < gw_Kit_SectionCount(); i++) {
        lua_newtable(L);
        for (k = 0; k < 4; k++) {
            lua_pushinteger(L, (lua_Integer) gw_Kit_SectionRGBA(i, k));
            lua_setfield(L, -2, which[k]);
        }
        lua_setfield(L, -2, gw_Kit_SectionName(i));
    }
    for (k = 0; k < 5; k++) {
        uint32_t c;
        if (gw_Kit_Colour(ports[k], NULL, &c)) {
            lua_pushinteger(L, (lua_Integer) c);
            lua_setfield(L, -2, ports[k]);
        }
    }
    lua_setfield(L, -2, "colors");
    lua_newtable(L); /* roles, smallest first as the manifest lists them */
    for (i = 0; i < gw_Kit_RoleCount(); i++) {
        lua_pushstring(L, gw_Kit_RoleName(i));
        lua_rawseti(L, -2, i + 1);
    }
    lua_setfield(L, -2, "roles");
    gw_Kit_RowMetrics(&rh, &pitch, &lx, &lb, &lift);
    lua_createtable(L, 0, 5);
    gs_setnum(L, "h", rh);
    gs_setnum(L, "pitch", pitch);
    gs_setnum(L, "label_x", lx);
    gs_setnum(L, "label_base", lb);
    gs_setnum(L, "lift", lift);
    lua_setfield(L, -2, "row");
    lua_pushnumber(L, gw_Kit_Shear());
    lua_setfield(L, -2, "shear");
}

static const luaL_Reg gs_gd_funcs[] = {
    {"log", l_log}, {"frame", l_frame}, {"time", l_time}, {"scene", l_scene}, {"match", l_match},
    {"players", l_players}, {"player", l_player}, {"char_name", l_char_name}, {"pad", l_pad},
    {"input", l_input}, {"release", l_release}, {"savestate", l_savestate},
    {"loadstate", l_loadstate}, {"pause", l_pause}, {"resume", l_resume}, {"step", l_step},
    {"paused", l_paused}, {"set_percent", l_set_percent}, {"set_stocks", l_set_stocks},
    {"scene_launch", l_scene_launch}, {"scene_clear", l_scene_clear}, {"text", l_text},
    {"box", l_box}, {"fill", l_fill}, {"line", l_line}, {"key", l_key},
    {"key_pressed", l_key_pressed}, {"mouse", l_mouse}, {"command", l_command}, {"run", l_run},
    {"data_read", l_data_read}, {"data_write", l_data_write}, {"script", l_script_info},
    {"rgb", l_rgb}, {"label", l_label}, {"screenshot", l_screenshot}, {"quit", l_quit},
    {"menu", l_menu}, {"netplay", l_netplay}, {"netplay_act", l_netplay_act},
    /* the Geno Lab (docs/geno.md) */
    {"debug_draw", l_debug_draw}, {"debug_stage", l_debug_stage}, {"hitboxes", l_hitboxes},
    {"hurtboxes", l_hurtboxes}, {"joints", l_joints}, {"dobjs", l_dobjs}, {"project", l_project}, {"attrs", l_attrs},
    {"motion_name", l_motion_name}, {"history", l_history}, {"step_back", l_step_back},
    {"rewind_to", l_rewind_to}, {"rewind_live", l_rewind_live}, {"rewind_test", l_rewind_test},
    {"rewind_test_result", l_rewind_test_result}, {"hot_reload", l_hot_reload},
    {"hot_reload_status", l_hot_reload_status}, {"state_save", l_state_save},
    {"state_list", l_state_list}, {"state_load", l_state_load}, {"state_delete", l_state_delete},
    {"state_rename", l_state_rename}, {"state_gen", l_state_gen}, {"lab_peek", l_lab_peek},
    {"timeline", l_timeline}, {"set_motion", l_set_motion}, {"mirror_pad", l_mirror_pad},
    {"lab_request", l_lab_request}, {"lab_mode", l_lab_mode}, {"lab_leave", l_lab_leave},
    {"training_select", l_training_select},
    /* stage E */
    {"motion_list", l_motion_list}, {"kb_preview", l_kb_preview}, {"rollbacks", l_rollbacks},
    {"rollbacks_clear", l_rollbacks_clear}, {"lab_env", l_lab_env}, {"lab_now", l_lab_now},
    /* stage D */
    {"lab_common", l_lab_common},
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
    lua_pushinteger(L, 1);
    lua_setfield(L, -2, "lab_api"); /* the Geno Lab API (private build); nil elsewhere */
    lua_newtable(L);
    lua_setfield(L, -2, "deprecated"); /* name -> "use X instead", filled as the API evolves */
    lua_newtable(L);
    for (i = 0; i < 12; ++i) {
        lua_pushinteger(L, gs_buttons[i].bit);
        lua_setfield(L, -2, gs_buttons[i].name);
    }
    lua_setfield(L, -2, "buttons");
    {
        /* Geno Lab: Fighter.x21FC_flag bits (gd.debug_draw) - the byte ftDrawCommon_800805C8 reads
           (PPC bitfields: b7 is 0x01). HIT and HURT are one switch in the game. */
        static const struct {
            const char *name;
            int bit;
        } draw[] = {{"MODEL", 0x01}, {"HIT", 0x02}, {"HURT", 0x02}, {"COLL", 0x02},
                    {"DYNAMICS", 0x04}, {"STOMP", 0x08}, {"CPU", 0x10}, {"ITEM_PICKUP", 0x20},
                    {"THROWN", 0x40}, {"COIN", 0x80}, {"DEFAULT", 0x01}},
          stage[] = {{"COLL", LAB_STAGE_COLL}, {"ECB", LAB_STAGE_COLL},
                     {"TERRAIN", LAB_STAGE_TERRAIN}, {"LEDGES", LAB_STAGE_LEDGES},
                     {"POINTS", LAB_STAGE_POINTS}, {"ZONES", LAB_STAGE_ZONES}};
        size_t k;
        lua_newtable(L);
        for (k = 0; k < sizeof draw / sizeof draw[0]; ++k) {
            lua_pushinteger(L, draw[k].bit);
            lua_setfield(L, -2, draw[k].name);
        }
        lua_setfield(L, -2, "draw");
        lua_newtable(L);
        for (k = 0; k < sizeof stage / sizeof stage[0]; ++k) {
            lua_pushinteger(L, stage[k].bit);
            lua_setfield(L, -2, stage[k].name);
        }
        lua_setfield(L, -2, "stage_draw");
    }
    gs_push_kit(L);
    lua_setfield(L, -2, "kit");
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

/* A copy of the table at `src` whose table values are copied too, `depth` levels down. */
static void gs_copy_deep(lua_State *L, int src, int depth) {
    src = lua_absindex(L, src);
    lua_newtable(L);
    lua_pushnil(L);
    while (lua_next(L, src) != 0) {
        if (depth > 1 && lua_istable(L, -1)) {
            gs_copy_deep(L, -1, depth - 1);
            lua_remove(L, -2);
        }
        lua_pushvalue(L, -2);
        lua_insert(L, -2);
        lua_rawset(L, -4);
    }
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
        if (strcmp(copy[i], "gd") == 0) {
            /* gd.kit and its tables (colors and its sections, roles, row): private copies too */
            lua_getfield(L, -1, "kit");
            gs_copy_deep(L, -1, 3);
            lua_setfield(L, -3, "kit");
            lua_pop(L, 1);
        }
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

/* A script's kit art (gd.kit): ui/ beside the script, else ui/ one level up - a mod's
 * <mod>/scripts/x.lua finds <mod>/ui, a scripts/<id>/main.lua finds scripts/<id>/ui. */
static void gs_find_ui_dir(const char *entry, char *out, size_t cap) {
    char dir[MAX_PATH], cand[MAX_PATH + 8];
    char *slash;
    int up;
    out[0] = '\0';
    snprintf(dir, sizeof dir, "%s", entry);
    for (up = 0; up < 2; up++) {
        slash = strrchr(dir, '\\');
        if (strrchr(dir, '/') > slash) slash = strrchr(dir, '/');
        if (slash == NULL) return;
        *slash = '\0';
        snprintf(cand, sizeof cand, "%s\\ui", dir);
        {
            DWORD a = GetFileAttributesA(cand);
            if (a != INVALID_FILE_ATTRIBUTES && (a & FILE_ATTRIBUTE_DIRECTORY)) {
                snprintf(out, cap, "%s", cand);
                return;
            }
        }
    }
}

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
    gs_find_ui_dir(entry, s->ui_dir, sizeof s->ui_dir);
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
    gs.cam_fetched = -1;
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
    v = getenv("MELEE_LAB"); /* the Geno Lab on from the start (as the frontend's LAB entry) */
    gs.lab_request = v != NULL && v[0] != '\0' && v[0] != '0';
    if (gs.lab_request) {
        gw_log("script: MELEE_LAB: the Geno Lab is requested for this session");
    }
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
    v = getenv("MELEE_LOBBY_AUTOPLAY"); /* the old native test switch is this built-in now */
    if (v != NULL && v[0] != '\0' && v[0] != '0') {
        int i = gs_load_named("builtin:lobby_autoplay");
        if (i >= 0) snprintf(gs.s[i].origin, sizeof gs.s[i].origin, "env");
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
    gs.nev = 0; /* events from the scene that ended are dropped */
    gs_rw_stop();
    memset(gs_sfx_map_from, 0xFF, sizeof gs_sfx_map_from);
    gw_rw_measure(0);
    gs.rwt_arm = 0;
    gs.rwt_phase = 0;
    gs.hot_phase = 0;
    gs.st_pending_save[0] = gs.st_pending_load[0] = '\0';
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

/* Does any script define one of the event hooks? (checked every tick: cheap, and it follows
   scripts that define a hook late, e.g. from the console) */
static void gs_update_want_events(void) {
    static const char *const hooks[] = {"on_action_change", "on_hit", "on_hitlag", "on_land"};
    int i, k, want = 0;
    for (i = 0; i < gs.n && !want; ++i) {
        for (k = 0; k < 4 && !want; ++k) {
            if (gs_get_hook(i, hooks[k])) {
                lua_pop(gs.L, 1);
                want = 1;
            }
        }
    }
    gs.want_events = want;
}

/* The draw pass: on_tick opens a list (gw_Script_Tick), on_draw completes it after the render
   (gw_Script_PostRender) and the overlay is handed the finished list. A scene loop that never
   reaches PostRender still gets on_draw, at the next tick (the old timing). */
static int gs_draw_open;
static void gs_finish_draw(void) {
    if (!gs_draw_open) {
        return;
    }
    gs.cam_stamp++;
    gs_hook_all("on_draw", 0, 0, 0);
    gs.build = !gs.build;
    gw_Kit_SwapBanks(); /* the kit's quads flip with the list that indexes them */
    gs_draw_open = 0;
}

static void gs_apply_pending(int at_tick);
static void gs_apply_set_motion(void);
static void gs_rw_tick_top(void);

void gw_Script_Tick(void) {
    gs_init();
    if (gs.L == NULL) {
        return;
    }
    if (gs.console_open) {
        gw_TextEntryUntil = (int) GetTickCount() + 200; /* the keyboard is text, not hotkeys */
    }
    gs_poll_keys();
    gw_Mouse_ScriptTick();
    gs_socket_poll();
    if (gs_pending_launch >= 0) {
        int mode = gs_pending_launch;
        gs_pending_launch = -1;
        gw_log("script: launching scene (game mode %d)", mode);
        gw_ScriptGame_LaunchScene(mode);
    }
    gs_finish_draw();
    gs.ndraw[gs.build] = 0;
    gs_draw_open = 1;
    gs.cam_stamp++;
    gs_update_want_events();
    gw_Kit_BeginFrame();
    gs_rw_tick_top();
    if (gs.paused && !gw_RB_Enabled() && !gw_Netplay_Enabled()) {
        /* Paused: no logic frame reads the pads, so gd.pad would stay stale and a script's menu
           (the Geno Lab's LAB pause menu) could not be driven by a controller. Sample them here,
           as PADRead would (scripted overrides included); the game never sees this read. */
        extern int gw_PADRead(void *status);
        unsigned char st[4 * 32]; /* PADStatus[4], 16 bytes each */
        memset(st, 0, sizeof st);
        (void) gw_PADRead(st);
    }
    gs_hook_all("on_tick", 0, 0, 0);
    /* Paused: no logic frame will run this tick, so there is no frame boundary for a pending
       savestate / loadstate / step-back to wait for. This point is between frames too (the loop
       top, before any logic), so apply them now - the render below shows the loaded state. */
    if (gs.paused && gs.step == 0 && !gw_RB_Enabled() && !gw_Netplay_Enabled()) {
        gs_apply_pending(1);
    }
    if (gs.paused && !gw_RB_Enabled() && !gw_Netplay_Enabled()) {
        gs_apply_set_motion(); /* before this tick's frames: then exactly `frame - 1` run */
    }
}

/* the frontend's LAB entry (gmfrontend_menus.inc): shown when a Lab script is loaded */
int gw_Script_LabAvailable(void) {
    int i;
    gs_init();
    for (i = 0; i < gs.n; ++i) {
        if (gs.s[i].used && !gs.s[i].disabled && _strnicmp(gs.s[i].id, "geno-lab/", 9) == 0) {
            return 1;
        }
    }
    return 0;
}
void gw_Script_LabRequest(void) { gs.lab_request = 1; }

void gw_Script_PostRender(void) {
    if (gs.L == NULL) {
        return;
    }
    gs_finish_draw();
}

int gw_Script_Iterations(int count) {
    if (gs.rw_resim > 0 && !gw_RB_Enabled() && !gw_Netplay_Enabled()) {
        /* a rewind: re-simulate every frame from the keyframe to the target in this one tick */
        int n = gs.rw_resim;
        gs.rw_resim = 0;
        gs.rw_resim_run = n;
        return n;
    }
    if (!gs.paused || gw_RB_Enabled() || gw_Netplay_Enabled()) {
        return count;
    }
    if (gs.step > 0 && count > 0) {
        /* up to 30 frames per rendered tick, so gd.set_motion / long steps land quickly */
        int n = gs.step < 30 ? gs.step : 30;
        gs.step -= n;
        return n;
    }
    return 0;
}

static void gs_slot_capture(GsSaveSlot *s) {
    s->used = 1;
    s->scene_epoch = gs.scene_epoch;
    s->frame = gs.frame;
    s->match_frame = gs.match_frame;
    memcpy(s->last_action, gs.last_action, sizeof s->last_action);
    memcpy(s->state_frame, gs.state_frame, sizeof s->state_frame);
}

static void gs_slot_restore(const GsSaveSlot *s) {
    gs.match_frame = s->match_frame;
    memcpy(gs.last_action, s->last_action, sizeof gs.last_action);
    memcpy(gs.state_frame, s->state_frame, sizeof gs.state_frame);
}

static int gs_snap_ensure(int slots) {
    int got = gw_snap_reserve(slots);
    gs.snap_ready = got >= GS_SAVE_SLOTS;
    return got;
}

/* the rewind's "now": the match frame about to run */
static int gs_ring_now(void) { return gs.match_active ? gs.match_frame + 1 : 0; }

/* ---- the long rewind: the input log, keyframes as frames pass, rewinds -------------------------- */
static int gs_lab_frame_kind; /* this logic frame: 0 live, 1 re-simulated (silent), 2 a burst's last */

static int gs_rw_session(void) { return gw_RB_Enabled() || gw_Netplay_Enabled(); }

static int gs_rw_replaying(int tag) {
    return gs.rw_began && tag >= gs.rw_log_start && tag < gs.rw_head;
}

static void gs_rw_stop(void) {
    gw_rw_end();
    gs.rw_began = 0;
    gs.rw_resim = gs.rw_resim_run = 0;
    gs.rw_pending_on = 0;
    gs.rw_notify = 0;
    gs.rw_head = gs.rw_log_start = 0;
    gw_Snap_LabResim(0);
}

/* A write changed the game at this boundary (set_percent, set_motion, ...): the logged frames from
 * here on belong to the old timeline, and the next boundary keeps a keyframe of the new one. */
static void gs_rw_branch(void) {
    int now = gs_ring_now();
    if (!gs.rw_began) {
        return;
    }
    if (gs.rw_head > now) {
        gs.rw_head = now;
    }
    if (gw_rw_drop_from(now) != 0) {
        gs.rw_began = 0; /* the base itself is this frame or later: start over here */
    }
    gs.rw_force_key = 1;
}

static int gs_rw_request(int target, const char **why) {
    if (gs.rw_frames <= 0 || !gs.rw_began) {
        *why = "history is off (gd.history(n) turns it on)";
        return -1;
    }
    if (gs_rw_session()) {
        *why = "not during a netplay/rollback session";
        return -1;
    }
    if (gs.rw_resim > 0 || gs.rw_resim_run > 0 || gs.rw_pending_on) {
        *why = "busy rewinding";
        return -1;
    }
    if (target < gw_rw_base_tag() || target < gs.rw_log_start) {
        *why = "not that far back in the history";
        return -1;
    }
    if (target > gs.rw_head) {
        *why = "not that far forward";
        return -1;
    }
    gs.rw_pending_on = 1;
    gs.rw_pending_to = target;
    gs.paused = 1;
    gs.step = 0;
    return 0;
}

/* HSD_PadRenewMasterStatus (controller.c): the pads for a frame being replayed. 0 = not replaying,
 * 1 = `out` holds the logged statuses, 2 = the frame renewed nothing when it first ran. */
int gw_LabPad_Replay(void *out, int size) {
    int tag;
    GsLogEntry *e;
    if (gs.L == NULL || !gs.rw_began || !gs.match_active) {
        return 0;
    }
    tag = gs_ring_now();
    if (!gs_rw_replaying(tag)) {
        return 0;
    }
    e = &gs_log[(unsigned) tag % GS_LOG_N];
    if (e->tag != tag) {
        return 0;
    }
    if (!e->have) {
        return 2;
    }
    memcpy(out, e->pad, size < (int) sizeof e->pad ? (size_t) size : sizeof e->pad);
    return 1;
}

/* ...and a live frame's statuses (NULL: it renewed nothing), logged at the live edge. */
void gw_LabPad_Record(const void *st, int size) {
    int tag;
    GsLogEntry *e;
    if (gs.L == NULL || !gs.rw_began || !gs.match_active || gs_rw_session()) {
        return;
    }
    tag = gs_ring_now();
    if (gs_rw_replaying(tag)) {
        return;
    }
    if (tag != gs.rw_head) {
        gs.rw_began = 0; /* a gap in the log: the next boundary starts the history over */
        return;
    }
    e = &gs_log[(unsigned) tag % GS_LOG_N];
    e->tag = tag;
    e->have = st != NULL;
    if (st != NULL) {
        memcpy(e->pad, st, size < (int) sizeof e->pad ? (size_t) size : sizeof e->pad);
    }
    e->nsfx = 0;
    e->nq = 0;
    e->sfx_open = 1;
    gs.rw_head = tag + 1;
}

/* HSD_AudioSFXStartParam (axdriver.c): a replayed frame gets the voice handle it got the first time
 * (the handle is game state, the voice pool is not; gw_LabSfx_Handle). 0 = not replaying (play,
 * then Record);
 * 1 = the logged handle, silent; 2 = the logged handle, play it too; 3 = not logged: play it. */
static int gs_lab_handle = -1, gs_lab_value, gs_lab_real;
int gw_LabSfx_Handle(void) { return gs_lab_handle; }
int gw_LabAudio_Value(void) { return gs_lab_value; }
int gw_LabAudio_Real(void) { return gs_lab_real; }

int gw_LabSfx_Replay(int sound_id) {
    int tag, i;
    GsLogEntry *e;
    gs_lab_handle = -1;
    if (gs.L == NULL || !gs.rw_began || !gs.match_active || !gs_in_frame) {
        return 0;
    }
    tag = gs_ring_now();
    if (!gs_rw_replaying(tag)) {
        return 0;
    }
    e = &gs_log[(unsigned) tag % GS_LOG_N];
    if (e->tag != tag) {
        return 3;
    }
    if (e->sfx_open) {
        return 0; /* this pass is the frame's first on this timeline (after a hot reload): record */
    }
    for (i = 0; i < e->nsfx; ++i) {
        if (!(gs_sfx_claimed & (1u << i)) && e->sfx_id[i] == sound_id) {
            gs_sfx_claimed |= 1u << i;
            gs_lab_handle = e->sfx_h[i];
            return gs_lab_frame_kind != 0 ? 1 : 2;
        }
    }
    return 3;
}

void gw_LabSfx_Record(int sound_id, int handle) {
    int tag;
    GsLogEntry *e;
    if (gs.L == NULL || !gs.rw_began || !gs.match_active || !gs_in_frame) {
        return;
    }
    tag = gs_ring_now();
    e = &gs_log[(unsigned) tag % GS_LOG_N];
    if (e->tag == tag && e->sfx_open && e->nsfx < GS_LOG_SFX) {
        e->sfx_id[e->nsfx] = sound_id;
        e->sfx_h[e->nsfx] = handle;
        e->nsfx++;
    }
}

void gw_LabSfx_Map(int logged, int real) {
    if (logged == real || logged < 0) {
        return;
    }
    gs_sfx_map_from[gs_sfx_map_next] = logged;
    gs_sfx_map_to[gs_sfx_map_next] = real;
    gs_sfx_map_next = (gs_sfx_map_next + 1) % GS_SFX_MAP;
}

static int gs_sfx_real(int h) {
    int i;
    for (i = 0; i < GS_SFX_MAP; ++i) {
        if (gs_sfx_map_from[i] == h && h >= 0) {
            return gs_sfx_map_to[i];
        }
    }
    return h;
}

/* axdriver.c LAB_AUDIO: the voice pool's answer to one call (a key-off, "still playing?", ...).
 * 0 = live (call it, then Record); 1 = re-simulated (the logged answer, no side effect); 2 =
 * replayed at speed (the logged answer; the call goes to the voice the replay started). */
int gw_LabAudio_Replay(int kind, int arg) {
    int tag;
    GsLogEntry *e;
    gs_lab_value = 0;
    gs_lab_real = gs_sfx_real(arg);
    if (gs.L == NULL || !gs.rw_began || !gs.match_active || !gs_in_frame) {
        return 0;
    }
    tag = gs_ring_now();
    if (!gs_rw_replaying(tag)) {
        return 0;
    }
    e = &gs_log[(unsigned) tag % GS_LOG_N];
    if (e->tag != tag || e->sfx_open) {
        return 0;
    }
    if (gs_q_pos < e->nq && e->q_kind[gs_q_pos] == kind && e->q_arg[gs_q_pos] == arg) {
        gs_lab_value = e->q_val[gs_q_pos++];
        return gs_lab_frame_kind != 0 ? 1 : 2;
    }
    return 0; /* the replay asks something the first run did not: answer live */
}

void gw_LabAudio_Record(int kind, int arg, int value) {
    int tag;
    GsLogEntry *e;
    if (gs.L == NULL || !gs.rw_began || !gs.match_active || !gs_in_frame) {
        return;
    }
    tag = gs_ring_now();
    e = &gs_log[(unsigned) tag % GS_LOG_N];
    if (e->tag == tag && e->sfx_open && e->nq < GS_LOG_Q) {
        e->q_kind[e->nq] = (unsigned char) kind;
        e->q_arg[e->nq] = arg;
        e->q_val[e->nq] = value;
        e->nq++;
    }
}

/* At every frame boundary: start the history, keep a keyframe every interval, slide the window. */
static void gs_rw_frame(void) {
    int now = gs_ring_now(), latest, edge;
    GsSaveSlot u;
    if (gs.rw_frames <= 0 || !gs.match_active || gs_rw_session()) {
        return;
    }
    if (gs.rw_began && now > gs.rw_head) {
        gs.rw_began = 0; /* frames ran that the log did not see */
    }
    if (!gs.rw_began) {
        gs_slot_capture(&u);
        if (gw_rw_begin(now, &u, sizeof u) != 0) {
            gw_Console_Print(GS_RED, "history unavailable (no memory for the base image)");
            gs.rw_frames = 0;
            return;
        }
        gs.rw_began = 1;
        gs.rw_head = now;
        gs.rw_log_start = now;
        gs.rw_force_key = 0;
        gw_log("rewind: on at frame %d: %d frames, a keyframe every %d", now, gs.rw_frames,
               gs.rw_interval);
        return;
    }
    latest = gw_rw_latest();
    if (now > latest && (gs.rw_force_key || now - latest >= gs.rw_interval)) {
        gs_slot_capture(&u);
        if (gw_rw_save(now, &u, sizeof u) != 0) {
            gw_log("rewind: keyframe at frame %d failed - starting over", now);
            gs.rw_began = 0;
            return;
        }
        gs.rw_force_key = 0;
    }
    edge = gs.rw_head > now ? gs.rw_head : now;
    gw_rw_trim(edge - gs.rw_frames);
    if (gs.rw_log_start < gw_rw_base_tag()) {
        gs.rw_log_start = gw_rw_base_tag();
    }
}

/* the exactness self-test (gd.rewind_test), at the very start of a frame */
static void gs_rwt_fail(const char *text) {
    double d[2];
    uint64_t h[2];
    (void) gw_rw_compare(d, h); /* frees the copy */
    gw_rw_measure(0);
    gs.rwt_arm = 0;
    gs.rwt_phase = 0;
    gs.rwt_pass = 0;
    snprintf(gs.rwt_text, sizeof gs.rwt_text, "FAIL: %s", text);
    gw_Console_Print(GS_RED, "rewind test: %s", gs.rwt_text);
    gw_log("rewind test: %s", gs.rwt_text);
}

static void gs_rwt_frame(void) {
    int now = gs_ring_now();
    if (gs.rwt_phase == 1) {
        if (!gs.rw_began || !gs.match_active) {
            gs_rwt_fail("history is off");
            return;
        }
        /* measure render-owned bytes from here on, and capture once a keyframe has been kept
           since: the keyframe the rewind starts from then lies inside the measured window */
        if (gs.rwt_arm == 0) {
            gw_rw_measure(1);
            gs.rwt_arm = now;
            return;
        }
        if (gw_rw_latest() < gs.rwt_arm) {
            return;
        }
        gs.rwt_arm = 0;
        if (gw_rw_capture() != 0) {
            gs_rwt_fail("no memory for the copy");
            return;
        }
        gs.rwt_tag = now;
        gs.rwt_phase = 2;
    } else if (gs.rwt_phase == 2 && now >= gs.rwt_tag + gs.rwt_back) {
        const char *why = NULL;
        if (gs_rw_request(gs.rwt_tag, &why) != 0) {
            gs_rwt_fail(why);
        } else {
            gs.rwt_phase = 3;
        }
    } else if (gs.rwt_phase == 4) {
        double d[2] = {0, 0};
        uint64_t h[2] = {0, 0};
        if (!gs.rwt_keep) {
            gs.paused = 1;
            gs.step = 0;
        }
        if (now != gs.rwt_tag) {
            char t[96];
            snprintf(t, sizeof t, "landed on frame %d, not %d", now, gs.rwt_tag);
            gs_rwt_fail(t);
            return;
        }
        gs.rwt_phase = 0;
        if (gw_rw_compare(d, h) != 0) {
            gs_rwt_fail("the copy was lost");
            return;
        }
        gw_rw_measure(0);
        gs.rwt_diff[0] = d[0];
        gs.rwt_diff[1] = d[1];
        gs.rwt_pass = d[1] == 0 && h[0] == h[1];
        snprintf(gs.rwt_text, sizeof gs.rwt_text,
                 "%s: frame %d, %d frames later rewound to it (keyframe load %.2f ms + %d frames "
                 "re-simulated, %.1f ms): %.0f simulation bytes differ, hash %016llx vs %016llx "
                 "(%.0f counting render-owned bytes, the disc's async blocks and pad-side rumble)",
                 gs.rwt_pass ? "PASS" : "FAIL", now, gs.rwt_back, gs.rw_last_load_ms,
                 gs.rw_last_frames, gs.rw_last_ms, d[1], (unsigned long long) h[0],
                 (unsigned long long) h[1], d[0]);
        gw_Console_Print(gs.rwt_pass ? GS_GREEN : GS_RED, "rewind test: %s", gs.rwt_text);
        gw_log("rewind test: %s", gs.rwt_text);
    }
}

extern int gw_GenoLab_Leave(int where);

/* hot reload, once the rewind to its start frame is done (the loop top, between frames) */
static void gs_hot_do(void) {
    char msg[400];
    int safe, n = 0, now = gs_ring_now(), t, script_rc;
    gs.hot_phase = 0;
    msg[0] = '\0';
    safe = gw_Geno_Reload(msg, sizeof msg);
    if (safe == 0) {
        snprintf(gs.hot_text, sizeof gs.hot_text,
                 "%s - the state no longer fits the fighters' layout: restarting the match", msg);
        gs.hot_ok = 0;
        gw_Console_Print(GS_YELLOW, "hot reload: %s", gs.hot_text);
        gw_log("hot reload: %s", gs.hot_text);
        gs.paused = 0;
        gs.step = 0;
        gs_rw_stop();
        if (!gw_GenoLab_Leave(3)) {
            gw_Console_Print(GS_RED, "hot reload: not a LAB match - restart it yourself");
        }
        gs_hook_all("on_hot_reload", 1, 0, 0);
        return;
    }
    if (safe > 0) {
        n = gw_GenoGame_LabReload();
    }
    script_rc = gs_exec("reload geno-lab/lab");
    /* the timeline from here on is the new data's: re-record its sounds, restart the history here,
       keep the logged input to replay */
    if (gs.rw_began) {
        GsSaveSlot u;
        for (t = now; t < gs.rw_head; ++t) {
            GsLogEntry *e = &gs_log[(unsigned) t % GS_LOG_N];
            if (e->tag == t) {
                e->sfx_open = 1;
                e->nsfx = 0;
                e->nq = 0;
            }
        }
        gs_slot_capture(&u);
        if (gw_rw_begin(now, &u, sizeof u) != 0) {
            gs_rw_stop();
        } else {
            gs.rw_log_start = now;
            gs.rw_force_key = 0;
        }
    }
    gs.hot_ok = 1;
    snprintf(gs.hot_text, sizeof gs.hot_text, "%s; %d fighter(s) re-applied; Lab script %s; replaying %.1f s",
             msg[0] ? msg : "geno.json unchanged", n, script_rc == 0 ? "reloaded" : "NOT reloaded",
             gs.rw_began && gs.rw_head > now ? (gs.rw_head - now) / 60.0 : 0.0);
    gw_Console_Print(GS_GREEN, "hot reload: %s", gs.hot_text);
    gw_log("hot reload: %s", gs.hot_text);
    gs.paused = 0;
    gs.step = 0;
    gs_hook_all("on_hot_reload", 1, 1, 0);
}

/* the loop top: the work that waits for a finished re-simulation */
static void gs_rw_tick_top(void) {
    if (gs.rw_resim_run > 0) {
        gs.rw_resim_run = 0; /* the loop ended the burst early (a scene change) */
        gw_Snap_LabResim(0);
    }
    if (gs.rw_resim > 0) {
        return;
    }
    if (gs.rw_notify) {
        gs.rw_notify = 0;
        gs.nev = 0;
        gs_hook_all("on_loadstate", 1, 0, 0); /* slot 0 = the history */
        if (gs.rwt_phase == 3) {
            gs.rwt_phase = 4;
            gs.paused = 0; /* the self-test compares at the next frame's start */
            gs.step = 0;
        }
    }
    if (gs.hot_phase == 2 && !gs.rw_pending_on) {
        gs_hot_do();
    }
}

static void gs_state_dir_of(const char *path, char *out, size_t cap) {
    char *s;
    snprintf(out, cap, "%s", path);
    s = strrchr(out, '\\');
    if (s != NULL) *s = '\0';
}

/* pending savestate / loadstate / rewind / library save and load, at a frame boundary: FramePre,
   or the loop top when paused (at_tick). A rewind starts a re-simulation burst, so it only
   happens at the loop top. */
static void gs_apply_pending(int at_tick) {
    if ((gs.pending_save || gs.pending_load || gs.rw_pending_on || gs.st_pending_save[0] ||
         gs.st_pending_load[0]) &&
        gs_rw_session()) {
        gw_Console_Print(GS_RED, "savestates are off during a netplay/rollback session");
        gs.pending_save = gs.pending_load = gs.rw_pending_on = 0;
        gs.st_pending_save[0] = gs.st_pending_load[0] = '\0';
    }
    if (gs.pending_save) {
        int slot = gs.pending_save - 1;
        gs.pending_save = 0;
        if (!gs.snap_ready) {
            gs_snap_ensure(GS_SAVE_SLOTS);
        }
        if (gs.snap_ready) {
            gw_snap_save_index(slot, -1 - slot);
            gs_slot_capture(&gs.slot[slot]);
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
            gw_snap_load_index(slot) == 0) {
            gs_slot_restore(&gs.slot[slot]);
            gs.rw_began = 0; /* the history belongs to the timeline we just left: start over */
            gw_Console_Print(GS_GREEN, "loaded state %d", slot + 1);
            gs_hook_all("on_loadstate", 1, slot + 1, 0);
        } else {
            gw_Console_Print(GS_RED, "could not load state %d", slot + 1);
        }
    }
    if (gs.st_pending_save[0] != '\0') {
        char path[MAX_PATH], dir[MAX_PATH];
        GsStateHdr h;
        double t0 = gs_now_ms();
        snprintf(path, sizeof path, "%s", gs.st_pending_save);
        gs.st_pending_save[0] = '\0';
        gs_state_fill(&h, gs.st_save_name, gs.st_save_desc);
        if (gw_snap_file_save(path, &h, sizeof h) == 0) {
            gs_state_dir_of(path, dir, sizeof dir);
            gs_state_index(dir);
            gs_st_gen++;
            gw_Console_Print(GS_GREEN, "state saved: %s (frame %d, %.0f ms)", h.name, h.match_frame,
                             gs_now_ms() - t0);
            gw_log("lab: state saved to %s in %.0f ms", path, gs_now_ms() - t0);
        } else {
            gw_Console_Print(GS_RED, "state could not be saved (%s)", path);
        }
    }
    if (gs.st_pending_load[0] != '\0') {
        char path[MAX_PATH], why[160];
        GsStateHdr h;
        int len, rc;
        snprintf(path, sizeof path, "%s", gs.st_pending_load);
        gs.st_pending_load[0] = '\0';
        len = gw_snap_file_header(path, &h, sizeof h);
        if (gs_state_check(&h, len, why, sizeof why) != 0) {
            gw_Console_Print(GS_RED, "state refused: %s", why);
        } else if ((rc = gw_snap_file_load(path)) != 0) {
            gw_Console_Print(GS_RED, "state not loaded: %s", rc == -2 ? "made for another memory layout"
                                                            : rc == -3 ? "the file is damaged"
                                                                       : "unreadable");
        } else {
            gs_slot_restore(&h.slot);
            gs.nev = 0;
            gs.rw_began = 0; /* a new timeline */
            gw_Console_Print(GS_GREEN, "state loaded: %s (frame %d)", h.name, h.match_frame);
            gs_hook_all("on_loadstate", 1, 9, 0); /* slot 9 = the library */
        }
    }
    if (gs.rw_pending_on && at_tick) {
        int target = gs.rw_pending_to, k;
        GsSaveSlot u;
        gs.rw_pending_on = 0;
        k = gw_rw_find_le(target);
        if (!gs.rw_began || k == -0x7FFFFFFF - 1 || target > gs.rw_head ||
            gw_rw_load(k, &u, sizeof u) != 0) {
            gw_Console_Print(GS_RED, "rewind: frame %d is no longer in the history", target);
            if (gs.hot_phase == 1) gs.hot_phase = 0;
            if (gs.rwt_phase == 3) gs_rwt_fail("the target left the history");
        } else {
            gs_slot_restore(&u);
            gs.nev = 0;
            gs.rw_last_load_ms = gw_rw_ms_last_load();
            gs.rw_t0 = gs_now_ms() - gs.rw_last_load_ms;
            gs.rw_last_frames = target - k;
            gs.rw_resim = target - k;
            gs.rw_notify = 1;
            if (gs.rw_resim == 0) {
                gs.rw_last_ms = gs.rw_last_load_ms;
            }
            if (gs.hot_phase == 1) {
                gs.hot_phase = 2;
            }
        }
    }
}

/* pending gd.set_motion calls, at a frame boundary (the loop top when paused, else FramePre) */
static void gs_apply_set_motion(void) {
    int slot, any = 0;
    for (slot = 0; slot < 6; ++slot) {
        if (gs.set_motion[slot] > 0 && !gs_rw_session()) {
            union {
                float f;
                int i;
            } r, lift;
            r.f = gs.set_rate[slot] > 0.0f ? gs.set_rate[slot] : 1.0f;
            lift.f = gs.set_lift[slot];
            if (!any) {
                gs_rw_branch(); /* a write: the timeline forks here */
                any = 1;
            }
            gw_ScriptGame_LabSetMotion(slot, gs.set_motion[slot] - 1, r.i, lift.i);
        }
        gs.set_motion[slot] = 0;
    }
}

void gw_Script_FramePre(void) {
    int resim = 0;
    if (gs.L == NULL) {
        return;
    }
    gs_lab_frame_kind = 0;
    gs_sfx_claimed = 0;
    gs_q_pos = 0;
    gs_in_frame = 1;
    if (gs.rw_resim_run > 0) {
        resim = gs.rw_resim_run > 1;
        gs_lab_frame_kind = resim ? 1 : 2;
        gw_Snap_LabResim(resim);
        gs.rw_resim_run--;
    }
    if (!resim) {
        gs_rwt_frame();
        gs_apply_pending(0);
        gs_apply_set_motion();
    }
    gs_rw_frame();
    if (!resim) {
        gs_hook_all("on_frame_pre", 0, 0, 0);
    }
}

/* ---- engine events (Script_GameEvent from ft/fighter.c, ft/ftcoll.c, ft/ftcommon.c) ------------ */
/* Called from inside a logic frame. Only queues: the hooks run after the frame (FramePost), never
   mid-frame and never for a resimulated frame, so a script cannot change the game while the
   engine is in the middle of it. Reading the attacker's hitbox here is read-only. */
void gw_Script_GameEvent(int what, int a, int b, int c, int d) {
    GsEvent *e;
    if (gs.L == NULL || !gs.want_events || gw_Snap_Resimulating()) {
        return;
    }
    if (gs.nev >= GS_MAX_EVENTS) {
        gs.ev_dropped++;
        return;
    }
    e = &gs.ev[gs.nev++];
    e->what = what;
    e->a = a;
    e->b = b;
    e->c = c;
    e->d = d;
    e->has_hit = 0;
    if (what == LAB_EV_HIT && a >= 0 && a < 6 && (c & LAB_HIT_INDEX_MASK) < 5 &&
        !(c & (LAB_HIT_ATTACKER_SUB | LAB_HIT_BY_ITEM))) {
        int k, idx = c & LAB_HIT_INDEX_MASK;
        for (k = 0; k < LAB_HI_COUNT; ++k) e->hit[k] = gw_ScriptGame_HitI(a, idx, k);
        for (k = 0; k < LAB_HF_COUNT; ++k) e->hitf[k] = gw_ScriptGame_HitF(a, idx, k);
        e->has_hit = 1;
    }
}

static const char *const gs_element_names[] = {
    "normal", "fire", "electric", "slash", "coin", "ice", "nap", "sleep", "catch",
    "ground", "cape", "inert", "disable", "dark", "scball", "lipstick", "leadead"};

static const char *gs_element_name(int e) {
    return e >= 0 && e < (int) (sizeof gs_element_names / sizeof gs_element_names[0])
               ? gs_element_names[e]
               : "?";
}

static void gs_push_hit_fields(lua_State *L, const int *hi, const float *hf) {
    gs_setint(L, "group", hi[LAB_HI_GROUP]);
    gs_setint(L, "bone", hi[LAB_HI_BONE]);
    gs_setnum(L, "damage", hf[LAB_HF_DAMAGE]);
    gs_setint(L, "angle", hi[LAB_HI_ANGLE]);
    gs_setint(L, "kbg", hi[LAB_HI_KBG]);
    gs_setint(L, "bkb", hi[LAB_HI_BKB]);
    gs_setint(L, "wbk", hi[LAB_HI_WBK]);
    gs_setint(L, "element", hi[LAB_HI_ELEMENT]);
    gs_setstr(L, "element_name", gs_element_name(hi[LAB_HI_ELEMENT]));
    gs_setint(L, "shield_damage", hi[LAB_HI_SHIELD_DMG]);
    gs_setnum(L, "radius", hf[LAB_HF_SIZE]);
    gs_setnum(L, "x", hf[LAB_HF_X]);
    gs_setnum(L, "y", hf[LAB_HF_Y]);
    gs_setnum(L, "z", hf[LAB_HF_Z]);
    gs_setnum(L, "px", hf[LAB_HF_PX]);
    gs_setnum(L, "py", hf[LAB_HF_PY]);
    gs_setnum(L, "pz", hf[LAB_HF_PZ]);
    gs_setbool(L, "hit_air", hi[LAB_HI_HIT_AIR]);
    gs_setbool(L, "hit_ground", hi[LAB_HI_HIT_GROUND]);
}

static void gs_dispatch_events(void) {
    int n = gs.nev, k, i;
    if (n == 0) {
        return;
    }
    gs.nev = 0;
    if (gs.ev_dropped > 0) {
        gw_log("script: %d engine events dropped (queue full)", gs.ev_dropped);
        gs.ev_dropped = 0;
    }
    gs.in_event = 1;
    for (k = 0; k < n; ++k) {
        const GsEvent *e = &gs.ev[k];
        static const char *const names[] = {"", "on_action_change", "on_hit", "on_hitlag", "on_land"};
        if (e->what < 1 || e->what > 4) {
            continue;
        }
        for (i = 0; i < gs.n; ++i) {
            lua_State *L = gs.L;
            int nargs;
            if (!gs_may_run(i) || !gs_get_hook(i, names[e->what])) {
                continue;
            }
            switch (e->what) {
            case LAB_EV_ACTION: /* (port, old, new, sub) */
                lua_pushinteger(L, e->a + 1);
                lua_pushinteger(L, e->b);
                lua_pushinteger(L, e->c);
                lua_pushboolean(L, e->d);
                nargs = 4;
                break;
            case LAB_EV_HIT: /* (attacker port or nil, victim port, info) */
                if (e->a >= 0) {
                    lua_pushinteger(L, e->a + 1);
                } else {
                    lua_pushnil(L);
                }
                lua_pushinteger(L, e->b + 1);
                lua_createtable(L, 0, 24);
                {
                    union {
                        int i;
                        float f;
                    } u;
                    u.i = e->d;
                    gs_setnum(L, "dealt", u.f);
                }
                if ((e->c & LAB_HIT_INDEX_MASK) != LAB_HIT_INDEX_MASK) {
                    gs_setint(L, "hitbox", e->c & LAB_HIT_INDEX_MASK);
                }
                gs_setbool(L, "item", (e->c & LAB_HIT_BY_ITEM) != 0);
                gs_setbool(L, "attacker_sub", (e->c & LAB_HIT_ATTACKER_SUB) != 0);
                gs_setbool(L, "victim_sub", (e->c & LAB_HIT_VICTIM_SUB) != 0);
                if (e->has_hit) {
                    gs_push_hit_fields(L, e->hit, e->hitf);
                }
                nargs = 3;
                break;
            case LAB_EV_HITLAG: /* (port, entering, sub) */
                lua_pushinteger(L, e->a + 1);
                lua_pushboolean(L, e->b);
                lua_pushboolean(L, e->c);
                nargs = 3;
                break;
            default: /* LAB_EV_LAND: (port, motion, sub) */
                lua_pushinteger(L, e->a + 1);
                lua_pushinteger(L, e->b);
                lua_pushboolean(L, e->c);
                nargs = 3;
                break;
            }
            gs_pcall(i, nargs, 0, names[e->what]);
        }
    }
    gs.in_event = 0;
}

void gw_Script_FramePost(void) {
    int slot, any = 0, kind = gs_lab_frame_kind;
    gs_in_frame = 0;
    if (gs.L == NULL) {
        return;
    }
    if (gs.rw_began) {
        /* the frame's sounds are all in the log now */
        GsLogEntry *e = &gs_log[(unsigned) gs_ring_now() % GS_LOG_N];
        if (e->tag == gs_ring_now()) {
            e->sfx_open = 0;
        }
    }
    if (gw_Snap_Resimulating() && kind != 1) {
        return; /* a rollback's resimulated frame */
    }
    gs_lab_frame_kind = 0;
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
    if (kind == 1) {
        /* a frame the Lab re-simulates: its bookkeeping, none of its hooks */
        if (gs.match_active) {
            gs.match_frame++;
        }
        return;
    }
    if (any && !gs.match_active) {
        gs.match_active = 1;
        gs.match_frame = 0;
        gs_hook_all("on_match_start", 0, 0, 0);
    } else if (gs.match_active) {
        gs.match_frame++;
    }
    if (kind == 2) {
        gs.rw_last_ms = gs_now_ms() - gs.rw_t0;
        gw_log("rewind: to frame %d: keyframe load %.2f ms (%u pages), %d frame(s) re-simulated, "
               "%.1f ms in all", gs_ring_now(), gs.rw_last_load_ms, gw_rw_last_load_pages(),
               gs.rw_last_frames, gs.rw_last_ms);
    }
    gs_dispatch_events();
    gs_hook_all("on_frame", 0, 0, 0);
    gs_run_tasks();
}

int gw_Script_DrawCount(void) { return gs.ndraw[!gs.build]; }
const GwScriptDraw *gw_Script_DrawAt(int i) {
    return (i >= 0 && i < gs.ndraw[!gs.build]) ? &gs.draw[!gs.build][i] : NULL;
}

uint64_t gw_Script_GameplayHash(void) {
    uint64_t h = 0;
    int i;
    char *d = gs.describe;
    size_t left = sizeof gs.describe;
    gs.describe[0] = '\0';
    for (i = 0; i < gs.n; ++i) { /* order-independent: XOR of per-script digests */
        GsScript *s = &gs.s[i];
        /* Only scripts that can change a netplay match count: gameplay AND rollback_safe. A
           gameplay script that is not rollback_safe is refused every gameplay write during a
           session (gs_require_gameplay), so it cannot make two peers' matches differ - e.g. the
           menu-driving input scripts of the netplay tests (np_host / np_guest). */
        if (s->used && s->gameplay && s->rollback_safe && i != gs.console && !s->disabled) {
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

/* ---- Geno Lab ---------------------------------------------------------------------------------- */
static int test_script_lab_api(void) {
    static const struct {
        const char *expr, *want;
    } checks[] = {
        {"= gd.lab_api", "1"},
        {"= gd.draw.MODEL, gd.draw.HIT, gd.draw.HURT, gd.draw.THROWN", "1\n2\n2\n64"},
        {"= gd.stage_draw.COLL, gd.stage_draw.ECB, gd.stage_draw.ZONES", "1\n1\n16"},
        {"= gd.motion_name(14)", "Wait"},
        {"= gd.motion_name(341)", "Special0"},
        {"= gd.player(1) == nil and gd.debug_draw(1) == nil and gd.joints(1) == nil", "true"},
        {"= gd.hitboxes(1) == nil and gd.hurtboxes(1) == nil and gd.attrs(1) == nil", "true"},
        {"= gd.history().depth", "0"},
        {"= select(2, gd.step_back(1))", "history is off"},
    };
    char out[512];
    size_t i;
    for (i = 0; i < sizeof checks / sizeof checks[0]; ++i) {
        if (t_exec(checks[i].expr, out, sizeof out) != 0 || strstr(out, checks[i].want) == NULL) {
            gw_test_fail("%s gave \"%s\" (want \"%s\")", checks[i].expr, out, checks[i].want);
            return 1;
        }
    }
    /* the Kirby table: special state 341 is JumpAerialF1 */
    if (strcmp(gs_motion_name(4, 341, NULL, out, sizeof out), "JumpAerialF1") != 0 ||
        strcmp(gs_motion_name(0x21, 400, "AttackAirF", out, sizeof out), "AttackAirF") != 0) {
        gw_test_fail("motion names: kirby 341 / m-ex fallback wrong");
        return 1;
    }
    {
        /* reading the LAB request must not clear it (it once did: the push became argument 1) */
        int prev = gs.lab_request;
        gs.lab_request = 1;
        t_exec("= gd.lab_request(), gd.lab_request(), gd.lab_request(true), gd.lab_request()", out,
               sizeof out);
        gs.lab_request = prev;
        if (strstr(out, "true\ntrue\ntrue\nfalse") == NULL) {
            gw_test_fail("gd.lab_request read/clear: \"%s\"", out);
            return 1;
        }
    }
    {
        char a[64];
        gs_anim_name("PlyKirby5K_Share_ACTION_AttackAirF_figatree", a, sizeof a);
        if (strcmp(a, "AttackAirF") != 0) {
            gw_test_fail("anim name from symbol: \"%s\"", a);
            return 1;
        }
    }
    return 0;
}

/* gd.project against a hand-built camera: the fighter-origin maths of lbVector_WorldToScreen */
static int test_script_lab_project(void) {
    float sx = 0.0f, sy = 0.0f, depth = 0.0f;
    int k, rc = 0;
    memset(gs.cam, 0, sizeof gs.cam);
    gs.cam[LAB_CAM_OK] = 1.0f;
    /* identity view with the camera 100 units back: eye space z = world z - 100 */
    gs.cam[LAB_CAM_VIEW + 0] = 1.0f;
    gs.cam[LAB_CAM_VIEW + 5] = 1.0f;
    gs.cam[LAB_CAM_VIEW + 10] = 1.0f;
    gs.cam[LAB_CAM_VIEW + 11] = -100.0f;
    gs.cam[LAB_CAM_PROJ] = 0.0f;
    gs.cam[LAB_CAM_P0] = 90.0f; /* fov: cot(45) = 1 */
    gs.cam[LAB_CAM_P1] = 640.0f / 480.0f;
    gs.cam[LAB_CAM_VP_XMIN] = 0.0f;
    gs.cam[LAB_CAM_VP_XMAX] = 640.0f;
    gs.cam[LAB_CAM_VP_YMIN] = 0.0f;
    gs.cam[LAB_CAM_VP_YMAX] = 480.0f;
    gs.cam_have = 1;
    gs.cam_fetched = gs.cam_stamp;
    if (!gs_project(0.0f, 0.0f, 0.0f, &sx, &sy, &depth) || fabsf(sx - 320.0f) > 0.01f ||
        fabsf(sy - 240.0f) > 0.01f || fabsf(depth - 100.0f) > 0.01f) {
        gw_test_fail("origin projected to %.2f %.2f (depth %.2f), want 320 240 (100)", sx, sy, depth);
        rc = 1;
    }
    /* y = 50 at depth 100 with fov 90: half the way up from the centre */
    if (!gs_project(0.0f, 50.0f, 0.0f, &sx, &sy, &depth) || fabsf(sy - 120.0f) > 0.01f) {
        gw_test_fail("(0, 50) projected to y %.2f, want 120", sy);
        rc = 1;
    }
    if (gs_project(0.0f, 0.0f, 150.0f, &sx, &sy, &depth)) {
        gw_test_fail("a point behind the camera projected as visible");
        rc = 1;
    }
    for (k = 0; k < LAB_CAM_COUNT; ++k) gs.cam[k] = 0.0f;
    gs.cam_have = 0;
    gs.cam_fetched = -1;
    return rc;
}

/* engine events: queued mid-frame, dispatched after it with the documented arguments */
static int test_script_lab_events(void) {
    char out[512];
    union {
        float f;
        int i;
    } bits;
    if (t_exec("ev = {}; function on_action_change(p, o, n, sub) ev[#ev+1] = 'a'..p..':'..o..'>'..n end "
               "function on_hit(a, v, i) ev[#ev+1] = 'h'..tostring(a)..'>'..v..':'..i.dealt..(i.item and 'i' or '') end "
               "function on_hitlag(p, on) ev[#ev+1] = 'l'..p..(on and '+' or '-') end "
               "function on_land(p, m) ev[#ev+1] = 'g'..p..':'..m end",
               out, sizeof out) != 0) {
        gw_test_fail("defining event hooks failed: %s", out);
        return 1;
    }
    gw_Script_Tick(); /* notices the hooks */
    gw_Script_GameEvent(LAB_EV_ACTION, 0, 14, 20, 0);
    bits.f = 12.5f;
    gw_Script_GameEvent(LAB_EV_HIT, -1, 1, 0xFF | LAB_HIT_BY_ITEM, bits.i);
    gw_Script_GameEvent(LAB_EV_HITLAG, 1, 1, 0, 0);
    gw_Script_GameEvent(LAB_EV_LAND, 0, 42, 0, 0);
    if (t_exec("= #ev", out, sizeof out) != 0 || strstr(out, "0") == NULL) {
        gw_test_fail("events ran before the frame ended: %s", out);
        return 1;
    }
    gw_Script_FramePost();
    if (t_exec("= table.concat(ev, ' ')", out, sizeof out) != 0 ||
        strstr(out, "a1:14>20 hnil>2:12.5i l2+ g1:42") == NULL) {
        gw_test_fail("event hooks got \"%s\"", out);
        return 1;
    }
    t_exec("on_action_change, on_hit, on_hitlag, on_land, ev = nil", out, sizeof out);
    gw_Script_Tick();
    if (gs.want_events) {
        gw_test_fail("event queue still armed with no hook defined");
        return 1;
    }
    return 0;
}

/* the subaction-script walk (gd.timeline) on a hand-made script at the top of MEM1 */
static int test_script_lab_timeline(void) {
    static const uint32_t words[] = {
        0x04000005u,                                     /* wait 5 */
        0x2C80100Cu, 0x04000000u, 0u, 0xB4990000u, 0x0A000000u, /* hitbox #1 bone 2 12% ... */
        0x0800000Au,                                     /* wait_until 10 */
        0x40000000u,                                     /* hitboxes_clear */
        0x5C000000u,                                     /* iasa */
        0x0C000002u,                                     /* loop 2 */
        0x04000003u,                                     /* wait 3 */
        0x10000000u,                                     /* loop_end */
        0x00000000u};                                    /* end */
    unsigned char saved[sizeof words * 4];
    unsigned char *at;
    uint32_t addr;
    size_t i;
    int truncated = 0, rc = 0, top;
    float t;
    lua_State *L = gs.L;
    if (gw_mem1 == NULL || gw_mem1_size < 0x1000) {
        return 0; /* no guest memory in this run */
    }
    at = gw_mem1 + gw_mem1_size - 0x400;
    addr = (uint32_t) (uintptr_t) at;
    memcpy(saved, at, sizeof saved);
    for (i = 0; i < sizeof words / sizeof words[0]; ++i) {
        at[4 * i] = (unsigned char) (words[i] >> 24);
        at[4 * i + 1] = (unsigned char) (words[i] >> 16);
        at[4 * i + 2] = (unsigned char) (words[i] >> 8);
        at[4 * i + 3] = (unsigned char) words[i];
    }
    top = lua_gettop(L);
    t = gs_walk_script(L, addr, &truncated);
    /* hitbox at frame 6, clear + iasa at 11, then 2 x wait 3: the script ends at t = 16 */
    if (t != 16.0f || truncated != 0 || lua_rawlen(L, -1) != 3) {
        gw_test_fail("walk: t=%.1f truncated=%d events=%d", t, truncated, (int) lua_rawlen(L, -1));
        rc = 1;
    } else {
        lua_rawgeti(L, -1, 1);
        lua_getfield(L, -1, "frame");
        lua_getfield(L, -2, "damage");
        lua_getfield(L, -3, "angle");
        lua_getfield(L, -4, "kbg");
        lua_getfield(L, -5, "bkb");
        lua_getfield(L, -6, "size");
        lua_getfield(L, -7, "id");
        if (lua_tonumber(L, -7) != 6.0 || lua_tointeger(L, -6) != 12 || lua_tointeger(L, -5) != 361 ||
            lua_tointeger(L, -4) != 100 || lua_tointeger(L, -3) != 20 || lua_tonumber(L, -2) != 4.0 ||
            lua_tointeger(L, -1) != 1) {
            gw_test_fail("hitbox decoded wrong: frame %.1f dmg %d ang %d kbg %d bkb %d size %.2f id %d",
                         lua_tonumber(L, -7), (int) lua_tointeger(L, -6), (int) lua_tointeger(L, -5),
                         (int) lua_tointeger(L, -4), (int) lua_tointeger(L, -3), lua_tonumber(L, -2),
                         (int) lua_tointeger(L, -1));
            rc = 1;
        }
        lua_settop(L, top + 1);
        lua_rawgeti(L, -1, 3);
        lua_getfield(L, -1, "name");
        lua_getfield(L, -2, "frame");
        if (strcmp(lua_tostring(L, -2), "iasa") != 0 || lua_tonumber(L, -1) != 11.0) {
            gw_test_fail("iasa event: %s at %.1f", lua_tostring(L, -2), lua_tonumber(L, -1));
            rc = 1;
        }
    }
    lua_settop(L, top);
    memcpy(at, saved, sizeof saved);
    return rc;
}

/* on_draw runs after the render and its list is the one the overlay gets */
static int test_script_lab_draw_pass(void) {
    char out[256];
    int n0;
    t_exec("function on_draw() gd.text(1, 2, 'lab-draw-test') end", out, sizeof out);
    gw_Script_Tick();
    gw_Script_PostRender();
    n0 = gw_Script_DrawCount();
    if (n0 < 1 || strcmp(gw_Script_DrawAt(n0 - 1)->text, "lab-draw-test") != 0) {
        gw_test_fail("on_draw's text is not in the finished list (%d items)", n0);
        t_exec("on_draw = nil", out, sizeof out);
        return 1;
    }
    gw_Script_Tick(); /* a new list opens; the overlay still shows the finished one */
    if (gw_Script_DrawCount() != n0) {
        gw_test_fail("opening a new list changed the shown one");
        t_exec("on_draw = nil", out, sizeof out);
        return 1;
    }
    t_exec("on_draw = nil", out, sizeof out);
    gw_Script_PostRender();
    return 0;
}

static int test_script_text_optional_args(void) {
    /* gd.text's colour and size are optional (luaL_tolstring's push once landed in their slots) */
    char out[512];
    gs.ndraw[gs.build] = 0;
    if (t_exec("gd.text(10, 20, 'plain')", out, sizeof out) != 0 ||
        t_exec("gd.text(10, 40, 'red', 0xFF0000FF)", out, sizeof out) != 0 ||
        t_exec("gd.text(10, 60, 42, 0x00FF00FF, 2)", out, sizeof out) != 0) {
        gw_test_fail("gd.text with optional arguments left out raised: %s", out);
        return 1;
    }
    if (gs.ndraw[gs.build] != 3 || strcmp(gs.draw[gs.build][0].text, "plain") != 0 || gs.draw[gs.build][0].rgba != 0xFFFFFFFFu ||
        gs.draw[gs.build][1].rgba != 0xFF0000FFu || gs.draw[gs.build][1].size != 1.0f || strcmp(gs.draw[gs.build][2].text, "42") != 0 ||
        gs.draw[gs.build][2].size != 2.0f) {
        gw_test_fail("gd.text draw list wrong (n=%d)", gs.ndraw[gs.build]);
        return 1;
    }
    gs.ndraw[gs.build] = 0;
    return 0;
}

/* A tiny .gxtex: 8x8 I4, every texel 0xF (an opaque mask). */
static int t_write_gxtex(const char *path) {
    unsigned char blob[64 + 32];
    static const unsigned hdr[11] = {0x47585458u, 1, 0 /* I4 */, 8, 8, 0xFFFFFFFFu, 0, 32, 0, 64, 0};
    FILE *f;
    int i;
    memset(blob, 0, sizeof blob);
    for (i = 0; i < 11; ++i) {
        blob[i * 4] = (unsigned char) (hdr[i] >> 24);
        blob[i * 4 + 1] = (unsigned char) (hdr[i] >> 16);
        blob[i * 4 + 2] = (unsigned char) (hdr[i] >> 8);
        blob[i * 4 + 3] = (unsigned char) hdr[i];
    }
    memset(blob + 64, 0xFF, 32);
    f = fopen(path, "wb");
    if (f == NULL) return -1;
    fwrite(blob, 1, sizeof blob, f);
    fclose(f);
    return 0;
}

/* gd.kit from a script mod: its own ui/ art (icon, 9-slice by prefix, *_ui.json size / tint /
 * palette) and the kit's fonts, all landing in the draw list as kit quads, in call order. */
static int test_script_kit_mod_art(void) {
    char dir[MAX_PATH], ui[MAX_PATH], path[MAX_PATH];
    static const char *const pieces[] = {"ico_testmark", "tpanel_corner_tl", "tpanel_corner_tr",
                                         "tpanel_corner_bl", "tpanel_corner_br", "tpanel_edge_h",
                                         "tpanel_edge_v", "tpanel_fill"};
    FILE *f;
    int i, k, rc = 0, n_kit = 0;
    gs_init(); /* the engine starts lazily; this may be the first test to touch it */
    if (gs.L == NULL) {
        gw_test_fail("the Lua state did not start");
        return 1;
    }
    snprintf(dir, sizeof dir, "%s\\gw_kit_test", gs.exe_dir);
    snprintf(ui, sizeof ui, "%s\\ui", dir);
    CreateDirectoryA(dir, NULL);
    CreateDirectoryA(ui, NULL);
    snprintf(path, sizeof path, "%s\\scripts", dir);
    CreateDirectoryA(path, NULL);
    for (k = 0; k < 8; ++k) {
        snprintf(path, sizeof path, "%s\\%s.gxtex", ui, pieces[k]);
        t_write_gxtex(path);
    }
    snprintf(path, sizeof path, "%s\\test_ui.json", ui);
    f = fopen(path, "w");
    fputs("{\"palette\": {\"mine\": {\"accentx\": {\"hex\": \"#38c9d9\", \"u32\": \"0x38C9D9FF\"}}},\n"
          " \"textures\": [{\"name\": \"ico_testmark\", \"size_1x\": [12, 10], \"tint\": \"accentx\"}]}\n", f);
    fclose(f);
    snprintf(path, sizeof path, "%s\\scripts\\kit.lua", dir);
    f = fopen(path, "w");
    fputs("function on_draw()\n"
          "  gd.text(1, 1, 'plain first')\n"
          "  local w, h = gd.kit.icon('testmark', 10, 10)\n"
          "  iw, ih = w, h\n"
          "  gd.kit.panel(20, 20, 100, 60, {prefix = 'tpanel', fill = 'accentx'})\n"
          "  if gd.kit.available() then\n"
          "    tw = gd.kit.text(30, 50, 'Kit', 'row', 'bone')\n"
          "    gd.kit.list(30, 90, 200, {'One', {label = 'Two', value = 'On'}}, 2)\n"
          "  end\n"
          "  accent = gd.kit.color('accentx')\n"
          "  tex = gd.kit.texture('ico_testmark')\n"
          "end\n", f);
    fclose(f);
    i = gs_load_script("kit_test", path, NULL, "test");
    if (i < 0 || strcmp(gs.s[i].ui_dir, ui) != 0) {
        gw_test_fail("a script mod did not find its ui/ folder (%s)", i >= 0 ? gs.s[i].ui_dir : "not loaded");
        rc = 1;
        goto done;
    }
    gw_Script_Tick();       /* a new list opens */
    gw_Script_PostRender(); /* on_draw completes it; it is the shown list now */
    if (gw_Script_DrawCount() < 2 || gw_Script_DrawAt(0)->kind != GW_SDRAW_TEXT || gw_Script_DrawAt(1)->kind != GW_SDRAW_KIT) {
        gw_test_fail("kit draws are not in the draw list after the plain text (n=%d)", gw_Script_DrawCount());
        rc = 1;
        goto done;
    }
    for (k = 0; k < gw_Script_DrawCount(); ++k) {
        if (gw_Script_DrawAt(k)->kind == GW_SDRAW_KIT) n_kit += gw_Script_DrawAt(k)->kqn;
    }
    /* icon 1 + panel 9 (fill, 4 edges, 4 corners), plus the text and rows when the kit is here */
    if (n_kit < 10 || gw_Kit_ShownQuadAt(n_kit - 1) == NULL || gw_Kit_ShownQuadAt(n_kit) != NULL) {
        gw_test_fail("kit quads: %d in the draw list, not the shown bank's count", n_kit);
        rc = 1;
    }
    {
        const GwKitQuad *icon = gw_Kit_ShownQuadAt(0), *fill = gw_Kit_ShownQuadAt(1);
        if (icon == NULL || icon->x[1] - icon->x[0] != 12.0f || icon->y[2] - icon->y[0] != 10.0f ||
            icon->rgba != 0x38C9D9FFu) {
            gw_test_fail("the icon ignored its *_ui.json size / tint");
            rc = 1;
        }
        if (fill == NULL || fill->tex < 0 || fill->rgba != 0x38C9D9FFu) {
            gw_test_fail("the panel's <prefix>_fill texture was not used with the fill colour");
            rc = 1;
        }
    }
    lua_rawgeti(gs.L, LUA_REGISTRYINDEX, gs.s[i].env_ref);
    lua_getfield(gs.L, -1, "accent");
    if (lua_tointeger(gs.L, -1) != 0x38C9D9FF) {
        gw_test_fail("gd.kit.color did not read the mod's palette");
        rc = 1;
    }
    lua_pop(gs.L, 1);
    lua_getfield(gs.L, -1, "tex");
    if (!lua_istable(gs.L, -1)) {
        gw_test_fail("gd.kit.texture returned no table for the mod's texture");
        rc = 1;
    } else {
        lua_getfield(gs.L, -1, "mask");
        if (!lua_toboolean(gs.L, -1)) {
            gw_test_fail("an I4 texture is not reported as a mask");
            rc = 1;
        }
        lua_pop(gs.L, 1);
    }
    lua_pop(gs.L, 2);
done:
    if (i >= 0) gs_unload(i);
    gs.ndraw[0] = gs.ndraw[1] = 0;
    gw_Kit_BeginFrame();
    gw_Kit_SwapBanks();
    gw_Kit_BeginFrame();
    gw_Kit_SwapBanks();
    for (k = 0; k < 8; ++k) {
        snprintf(path, sizeof path, "%s\\%s.gxtex", ui, pieces[k]);
        DeleteFileA(path);
    }
    snprintf(path, sizeof path, "%s\\test_ui.json", ui);
    DeleteFileA(path);
    snprintf(path, sizeof path, "%s\\scripts\\kit.lua", dir);
    DeleteFileA(path);
    snprintf(path, sizeof path, "%s\\scripts", dir);
    RemoveDirectoryA(path);
    RemoveDirectoryA(ui);
    RemoveDirectoryA(dir);
    return rc;
}

/* gd.kit is per-script like the rest of gd; kit calls from a script without art still work */
static int test_script_kit_isolated(void) {
    char out[512];
    t_exec("gd.kit.shear = 99; gd.kit.colors.gold = 1", out, sizeof out);
    t_exec("= gd.kit.panel ~= nil and type(gd.kit.roles) == 'table'", out, sizeof out);
    if (strstr(out, "true") == NULL) {
        gw_test_fail("gd.kit is missing from the console's gd: %s", out);
        return 1;
    }
    if (gs.console >= 0) {
        /* another environment still sees the kit's own values */
        int e = gs_new_env(gs.L);
        lua_rawgeti(gs.L, LUA_REGISTRYINDEX, e);
        lua_getfield(gs.L, -1, "gd");
        lua_getfield(gs.L, -1, "kit");
        lua_getfield(gs.L, -1, "shear");
        if (lua_tonumber(gs.L, -1) == 99) {
            gw_test_fail("one script's gd.kit edit leaked into another environment");
            lua_pop(gs.L, 4);
            luaL_unref(gs.L, LUA_REGISTRYINDEX, e);
            return 1;
        }
        lua_pop(gs.L, 4);
        luaL_unref(gs.L, LUA_REGISTRYINDEX, e);
    }
    t_exec("= gd.training_select()", out, sizeof out);
    if (strstr(out, "native") == NULL && strstr(out, "kit") == NULL) {
        gw_test_fail("gd.training_select() gave %s", out);
        return 1;
    }
    return 0;
}

/* ---- the Lab's long rewind (docs/geno.md 14.10) ---------------------------------------------------
 * The keyframe store on its own: a scratch region at the top of MEM1 (restored afterwards) is
 * written in three steps, two of them kept as keyframes; every load must give back exactly that
 * step's bytes, across trims (the base moving forward) and drops (a timeline forking). */
static int test_lab_rewind_store(void) {
    const uint32_t len = 5 * 4096 + 100; /* crosses pages; the last is partial */
    uint8_t *region = (uint8_t *) (uintptr_t) (0x80000000u + gw_mem1_size - 8 * 4096);
    uint8_t *orig = (uint8_t *) malloc(len), *want1 = (uint8_t *) malloc(len), *want2 = (uint8_t *) malloc(len);
    uint32_t i;
    int rc = 1;
    GsSaveSlot u, back;
    if (orig == NULL || want1 == NULL || want2 == NULL) {
        gw_test_fail("no memory");
        goto out;
    }
    memcpy(orig, region, len);
    memset(&u, 0, sizeof u);
    u.match_frame = 999;
    if (gw_rw_begin(1000, &u, sizeof u) != 0) {
        rc = 0; /* no map beside the exe in this run: nothing to test */
        goto out;
    }
    for (i = 0; i < len; ++i) region[i] = (uint8_t) (i * 7 + 1);          /* step 1: every page */
    memcpy(want1, region, len);
    u.match_frame = 1000;
    if (gw_rw_save(1001, &u, sizeof u) != 0) { gw_test_fail("save 1001"); goto end; }
    for (i = 4096; i < 2 * 4096; ++i) region[i] = (uint8_t) (i * 13 + 5);  /* step 2: page 1 only */
    memcpy(want2, region, len);
    u.match_frame = 1001;
    if (gw_rw_save(1002, &u, sizeof u) != 0) { gw_test_fail("save 1002"); goto end; }
    for (i = 0; i < len; i += 3) region[i] ^= 0x5A;                        /* step 3: not kept */
    if (gw_rw_save(1002, &u, sizeof u) == 0) { gw_test_fail("a second keyframe at 1002 was taken"); goto end; }
    if (gw_rw_load(1001, &back, sizeof back) != 0 || memcmp(region, want1, len) != 0 || back.match_frame != 1000) {
        gw_test_fail("load 1001 did not give step 1 back");
        goto end;
    }
    if (gw_rw_load(1002, &back, sizeof back) != 0 || memcmp(region, want2, len) != 0 || back.match_frame != 1001) {
        gw_test_fail("load 1002 did not give step 2 back");
        goto end;
    }
    if (gw_rw_load(1000, &back, sizeof back) != 0 || memcmp(region, orig, len) != 0 || back.match_frame != 999) {
        gw_test_fail("load 1000 (the base) did not give the original back");
        goto end;
    }
    if (gw_rw_find_le(1001) != 1001 || gw_rw_find_le(5000) != 1002 || gw_rw_find_le(999) != -0x7FFFFFFF - 1) {
        gw_test_fail("find_le: %d %d", gw_rw_find_le(1001), gw_rw_find_le(5000));
        goto end;
    }
    gw_rw_trim(1001); /* the base moves to 1001 */
    if (gw_rw_base_tag() != 1001 || gw_rw_load(1000, NULL, 0) == 0) {
        gw_test_fail("trim: base %d, 1000 still loadable", gw_rw_base_tag());
        goto end;
    }
    if (gw_rw_load(1002, NULL, 0) != 0 || memcmp(region, want2, len) != 0 ||
        gw_rw_load(1001, NULL, 0) != 0 || memcmp(region, want1, len) != 0) {
        gw_test_fail("after the trim, 1001 / 1002 are wrong");
        goto end;
    }
    for (i = 0; i < len; i += 5) region[i] = 0xEE; /* a fork after 1001 */
    if (gw_rw_drop_from(1002) != 0 || gw_rw_latest() != 1001 || gw_rw_load(1002, NULL, 0) == 0) {
        gw_test_fail("drop_from 1002");
        goto end;
    }
    if (gw_rw_save(1003, NULL, 0) != 0 || gw_rw_load(1001, NULL, 0) != 0 || memcmp(region, want1, len) != 0) {
        gw_test_fail("the fork's keyframe or the way back to 1001");
        goto end;
    }
    rc = 0;
end:
    gw_rw_end();
    memcpy(region, orig, len);
out:
    free(orig);
    free(want1);
    free(want2);
    return rc;
}

/* The input log: a live frame's pad statuses and sound handles come back, byte for byte, when the
 * frame is replayed; a frame that renewed nothing replays as nothing. */
static int test_lab_rewind_log(void) {
    unsigned char st[48], back[48];
    int saved_began = gs.rw_began, saved_active = gs.match_active, saved_mf = gs.match_frame;
    int saved_head = gs.rw_head, saved_start = gs.rw_log_start, m, rc = 1, i;
    for (i = 0; i < 48; ++i) st[i] = (unsigned char) (i * 11 + 3);
    gs.rw_began = 1;
    gs.match_active = 1;
    gs.match_frame = 499; /* now = 500 */
    gs.rw_head = 500;
    gs.rw_log_start = 500;
    gs_in_frame = 1;
    gw_LabPad_Record(st, 48);
    gw_LabSfx_Record(1234, 0x2F5C);
    gw_LabAudio_Record(6, 0x2F5C, 1);
    gs.match_frame = 500;
    gw_LabPad_Record(NULL, 48); /* frame 501 renewed nothing */
    if (gs.rw_head != 502) {
        gw_test_fail("head %d, want 502", gs.rw_head);
        goto out;
    }
    gs_log[500 % GS_LOG_N].sfx_open = 0; /* FramePost closes it */
    gs.match_frame = 499;
    gs_sfx_claimed = 0;
    gs_q_pos = 0;
    gs_lab_frame_kind = 1;
    memset(back, 0, sizeof back);
    if (gw_LabPad_Replay(back, 48) != 1 || memcmp(back, st, 48) != 0) {
        gw_test_fail("frame 500's pads did not come back");
        goto out;
    }
    m = gw_LabSfx_Replay(1234);
    if (m != 1 || gw_LabSfx_Handle() != 0x2F5C) {
        gw_test_fail("frame 500's sound handle: mode %d handle 0x%X", m, gw_LabSfx_Handle());
        goto out;
    }
    if (gw_LabSfx_Replay(1234) != 3) {
        gw_test_fail("a second start of the same sound should be a new one");
        goto out;
    }
    if (gw_LabAudio_Replay(6, 0x2F5C) != 1 || gw_LabAudio_Value() != 1) {
        gw_test_fail("frame 500's still-playing answer");
        goto out;
    }
    gs.match_frame = 500;
    if (gw_LabPad_Replay(back, 48) != 2) {
        gw_test_fail("frame 501 should replay as no renewal");
        goto out;
    }
    gs.match_frame = 501; /* now = 502: the live edge, not replayed */
    if (gw_LabPad_Replay(back, 48) != 0) {
        gw_test_fail("the live edge was replayed");
        goto out;
    }
    rc = 0;
out:
    gs_in_frame = 0;
    gs_lab_frame_kind = 0;
    gs.rw_began = saved_began;
    gs.match_active = saved_active;
    gs.match_frame = saved_mf;
    gs.rw_head = saved_head;
    gs.rw_log_start = saved_start;
    return rc;
}

/* A saved state is refused, with the reason, when anything in its header does not match. */
static int test_lab_state_header(void) {
    GsStateHdr h;
    char why[160];
    int saved_active = gs.match_active;
    gs_state_fill(&h, "Fox v Falco", "Fox v Falco on FD");
    gs.match_active = 1;
    if (gs_state_check(&h, sizeof h, why, sizeof why) != 0) {
        gw_test_fail("a fresh header was refused: %s", why);
        goto fail;
    }
    h.exe_hash ^= 1;
    if (gs_state_check(&h, sizeof h, why, sizeof why) == 0 || strstr(why, "build") == NULL) {
        gw_test_fail("another build: \"%s\"", why);
        goto fail;
    }
    h.exe_hash ^= 1;
    h.geno_hash ^= 1;
    if (gs_state_check(&h, sizeof h, why, sizeof why) == 0 || strstr(why, "Geno") == NULL) {
        gw_test_fail("other Geno data: \"%s\"", why);
        goto fail;
    }
    h.geno_hash ^= 1;
    h.stage ^= 1;
    if (gs_state_check(&h, sizeof h, why, sizeof why) == 0 || strstr(why, "Fox v Falco on FD") == NULL) {
        gw_test_fail("another stage: \"%s\"", why);
        goto fail;
    }
    h.stage ^= 1;
    if (gs_state_check(&h, sizeof h - 4, why, sizeof why) == 0) {
        gw_test_fail("a short header was taken");
        goto fail;
    }
    gs.match_active = 0;
    if (gs_state_check(&h, sizeof h, why, sizeof why) == 0 || strstr(why, "match") == NULL) {
        gw_test_fail("outside a match: \"%s\"", why);
        goto fail;
    }
    gs.match_active = saved_active;
    return 0;
fail:
    gs.match_active = saved_active;
    return 1;
}

void gw_script_tests_register(void) {
    gw_test_register("script_kit_mod_art", test_script_kit_mod_art);
    gw_test_register("script_kit_isolated", test_script_kit_isolated);
    gw_test_register("script_text_optional_args", test_script_text_optional_args);
    gw_test_register("script_lua_runs", test_script_lua_runs);
    gw_test_register("script_sandbox", test_script_sandbox);
    gw_test_register("script_budget", test_script_budget);
    gw_test_register("script_isolation_and_errors", test_script_isolation_and_errors);
    gw_test_register("script_manifest_and_hash", test_script_manifest_and_hash);
    gw_test_register("script_input_task", test_script_input_task);
    gw_test_register("script_lab_api", test_script_lab_api);
    gw_test_register("lab_rewind_store", test_lab_rewind_store);
    gw_test_register("lab_rewind_log", test_lab_rewind_log);
    gw_test_register("lab_state_header", test_lab_state_header);
    gw_test_register("script_lab_project", test_script_lab_project);
    gw_test_register("script_lab_events", test_script_lab_events);
    gw_test_register("script_lab_draw_pass", test_script_lab_draw_pass);
    gw_test_register("script_lab_timeline", test_script_lab_timeline);
}
