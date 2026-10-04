/* Actual policy/reload functions are extracted at build time, never hand-copied. */
#include <assert.h>
#include <setjmp.h>
#include <stdarg.h>
#include <stdio.h>
#include <string.h>
typedef struct { int unused; } lua_State;
typedef struct { int gameplay, rollback_safe; } GsScript;
typedef struct { int unused; } GsSaveSlot;
typedef struct { int tag, sfx_open, nsfx, nq; } GsLogEntry;
#define GS_LOG_N 8
#define GS_RED 1
#define GS_GREEN 2
#define GS_YELLOW 3
static struct {
    int cur, console, in_event, hot_phase, hot_ok, paused, step, match_active;
    int rw_began, rw_head, rw_log_start, rw_force_key;
    char hot_text[800];
} gs;
static GsScript script;
static GsLogEntry gs_log[GS_LOG_N];
static jmp_buf error_jump;
static int online, rollback, reload_result, script_result, lab_active, registry_reloads;
static int applied, script_reloads, restarts, timeline_stops, hook_ok;
static GsScript *gs_cur_script(void) { return &script; }
static int gw_RB_Enabled(void) { return rollback; }
static int gw_Netplay_Enabled(void) { return online; }
static int luaL_error(lua_State *L, const char *fmt, ...) { (void)L; (void)fmt; longjmp(error_jump, 1); }
static int gs_ring_now(void) { return 0; }
static int gw_Geno_Reload(char *msg, int cap) { ++registry_reloads; snprintf(msg, cap, "fixture reload"); return reload_result; }
static int gw_GenoLab_ModeActive(void) { return lab_active; }
static int gw_GenoGame_LabReload(void) { ++applied; return 1; }
static int gs_exec(const char *text) { (void)text; ++script_reloads; return script_result; }
static void gw_Console_Print(int color, const char *fmt, ...) { (void)color; (void)fmt; }
static void gw_log(const char *fmt, ...) { (void)fmt; }
static void gs_rw_stop(void) { ++timeline_stops; }
int gw_GenoLab_Leave(int where) { assert(where == 3); ++restarts; return 1; }
static void gs_hook_all(const char *name, int count, int ok, int unused) {
    (void)name; (void)count; (void)unused; hook_ok=ok;
}
static void gs_slot_capture(GsSaveSlot *slot) { (void)slot; }
static int gw_rw_begin(int now, GsSaveSlot *slot, int size) { (void)now; (void)slot; (void)size; return 0; }
#include "script_policy_functions.inc"

static int write_allowed(void) {
    if (setjmp(error_jump)) return 0;
    gs_require_gameplay(NULL, "set_percent");
    return 1;
}
static void reset_reload(int result) {
    memset(&gs,0,sizeof gs);
    gs.paused=1; gs.hot_ok=1; gs.hot_phase=2;
    gs.match_active=1; lab_active=1; registry_reloads=0;
    reload_result=result; script_result=0;
    applied=script_reloads=restarts=timeline_stops=0; hook_ok=-1;
}
int main(int argc, char **argv) {
    (void)argv;
    if (argc == 1) {
    gs.console=1; gs.cur=0; script.gameplay=1;
    assert(write_allowed());
    for (int safe=0; safe<2; ++safe) {
        script.rollback_safe=safe;
        online=1; assert(!write_allowed()); online=0;
        rollback=1; assert(!write_allowed()); rollback=0;
    }
    script.gameplay=0; assert(!write_allowed());
    gs.cur=gs.console; assert(write_allowed());
    online=1; assert(!write_allowed()); online=0;
    }
    reset_reload(-1); gs_hot_do();
    assert(!gs.hot_ok && hook_ok==0 && gs.paused);
    reset_reload(0); lab_active=0; gs_hot_do();
    assert(!gs.hot_ok && hook_ok==0 && !registry_reloads && !restarts);
    assert(!applied && !script_reloads && !restarts && !timeline_stops);
    reset_reload(0); gs_hot_do();
    assert(!gs.hot_ok && hook_ok==0 && restarts==1 && !applied);
    reset_reload(1); gs_hot_do();
    assert(gs.hot_ok && hook_ok==1 && applied==1 && script_reloads==1 && !gs.paused);
    reset_reload(1); script_result=-1; gs_hot_do();
    assert(!gs.hot_ok && hook_ok==0 && gs.paused);
    puts("script permissions and hot reload policy passed");
    return 0;
}
