/* gw_script.h - the Lua scripting engine and the console (docs/scripting.md is the reference).
 *
 * Scripts are sandboxed Lua 5.4 (pc/third_party/lua-5.4.7) loaded from:
 *   scripts/<name>.lua                 next to melee-pc.exe (single file)
 *   scripts/<id>/mod.json + main.lua   next to melee-pc.exe (a script with a manifest)
 *   <mods>/<id>/scripts/*.lua          inside a mod folder (gw_mods.h layout), when the mod is on
 * plus the console and the local socket. The public API a script sees is the `gd` table,
 * versioned by gd.api_version (GW_SCRIPT_API_VERSION).
 *
 * WHERE THE ENGINE RUNS (all on the game thread, from gmscene.c's scene loop):
 *   gw_Script_SceneBegin   a scene started (on_scene, and on_match_end when a match scene ends)
 *   gw_Script_Tick         once per render tick before the logic: console, socket, on_tick,
 *                          on_draw (the draw list is rebuilt here every tick)
 *   gw_Script_Iterations   pause / frame advance: how many logic frames this tick runs
 *   gw_Script_FramePre     at the logic-frame boundary: pending savestate/loadstate, on_frame_pre
 *   gw_Script_FramePost    after the frame's GObj procs: on_frame, input-script tasks, match
 *                          start detection
 *   gw_Script_PadApply     shim_pad.c, once per PADRead: MELEE_PAD_SCRIPT (.txt or .lua),
 *                          MELEE_PAD_LIVE, and inputs scripts/console inject with gd.input
 *
 * DETERMINISM RULE (netplay/rollback): during a rollback session no script hook runs on a
 * RESIMULATED frame, and only scripts whose manifest says "gameplay": true may call the gameplay
 * writes (gd.set_percent, gd.set_stocks, gd.input, savestates, pause) - and in a netplay session
 * even those are refused unless the manifest also says "rollback_safe": true, which promises the
 * script derives everything it writes from game state (no Lua-side memory across frames). The
 * gameplay scripts' sources are hashed into gw_Script_GameplayHash for the must-match set.
 */
#ifndef GW_SCRIPT_H
#define GW_SCRIPT_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define GW_SCRIPT_API_VERSION 1

/* ---- scene loop (gmscene.c; the game calls these without the gw_ prefix) --------------------- */
void gw_Script_SceneBegin(int scene_kind);
void gw_Script_Tick(void);
int gw_Script_Iterations(int count);
void gw_Script_FramePre(void);
void gw_Script_FramePost(void);

/* ---- input (shim_pad.c) -------------------------------------------------------------------- */
void gw_Script_PadApply(void *pad_status_array); /* PADStatus[4], button field big-endian */

/* ---- console --------------------------------------------------------------------------------- */
/* Run one console line (a built-in command or Lua). Output goes to the console scrollback; when
 * `out` is non-NULL it also receives it. Returns 0 on success, -1 on an error. */
int gw_Script_Exec(const char *line, char *out, int cap);
int gw_Console_LineCount(void);
const char *gw_Console_Line(int i);   /* 0 = oldest kept line */
uint32_t gw_Console_LineColor(int i); /* 0xRRGGBBAA */
void gw_Console_Print(uint32_t rgba, const char *fmt, ...);
int gw_Console_Open(void);           /* the in-game overlay is showing */
void gw_Console_SetOpen(int open);

/* ---- script drawing (rendered by gw_console.cpp in the ImGui pass) --------------------------- */
enum { GW_SDRAW_TEXT = 0, GW_SDRAW_BOX = 1, GW_SDRAW_FILL = 2, GW_SDRAW_LINE = 3 };
typedef struct {
    int kind;
    float x, y, w, h; /* 640x480 virtual screen; LINE uses (x,y)-(w,h) as the end point */
    uint32_t rgba;
    float size; /* text scale, 1 = the console font */
    char text[160];
} GwScriptDraw;
int gw_Script_DrawCount(void);
const GwScriptDraw *gw_Script_DrawAt(int i);

/* ---- netplay (delta's handshake fingerprint) ------------------------------------------------- */
/* 64-bit digest of every loaded script whose manifest says "gameplay": true AND "rollback_safe":
 * true (id, version and source) - the only scripts that may write gameplay during a netplay
 * session - 0 when there is none: two peers whose such scripts differ must not match. */
uint64_t gw_Script_GameplayHash(void);
/* "id@version#hhhh,..." of the same set, "" when none. */
const char *gw_Script_GameplayDescribe(void);

void gw_script_tests_register(void);

#ifdef __cplusplus
}
#endif
#endif /* GW_SCRIPT_H */
