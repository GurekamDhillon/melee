#ifndef MELEE_GM_GMFRONTEND_H
#define MELEE_GM_GMFRONTEND_H

#if defined(TARGET_PC)

#include <melee/gm/forward.h>
#include <melee/gm/types.h>

/* The port's own frontend screens: a game mode (GM_FRONTEND) with one state whose scene
 * (GS_FRONTEND) draws with Melee's systems. Both kinds sit past the retail terminators, so every
 * table walk that stops at GM_COUNT / GS_COUNT still finds the entries placed before them. */
#define GM_FRONTEND (GM_COUNT + 1)
#define GS_FRONTEND (GS_COUNT + 1)

extern GameModeState gm_Mode_Frontend_States[];

void gm_Scene_Frontend_OnFrame(void);
void gm_Scene_Frontend_OnEnter(void* enter_data);
void gm_Scene_Frontend_OnExit(void* exit_data);

/* Called by the top-level mode loop on every mode change: returns `to`, or GM_FRONTEND when a
 * frontend screen is placed between `from` and `to`. */
u8 gmFrontend_Route(u8 from, u8 to);

/* The mode a screen stands in for (the one it continues to). The loop records this as the
 * previous mode instead of GM_FRONTEND, so a native screen that positions itself by where the
 * player came from - the main menu's cursor - acts as if the player had backed out of it. */
u8 gmFrontend_ReportedMode(void);

/* The loading screen, as a state inside a mode (VS: between the SSS and the match). The state's
 * on_enter calls this; the scene ends by itself once the renderer is warm, and the state's
 * on_exit picks the next state. */
void gmFrontend_BeginLoading(void);

/* THE MENU TREE (gmfrontend_menus.inc). When the frontend wants one of GM_MENU's native screens
 * it leaves for GM_MENU with a request: gmmenumode.c positions the menu at (kind, selection) -
 * the parent menu and the item - via gmFrontend_NativeRequest, and mnmain.c takes the request
 * and opens that item's screen as the parent's think would (gmFrontend_TakeNativeRequest). */
bool gmFrontend_NativeRequest(u8* kind, u8* sel);
bool gmFrontend_TakeNativeRequest(u8* kind, u8* sel);

/* mnmain.c's mn_80229894 - a native screen backing out to (kind, selection) - asks this first:
 * true when that menu is drawn by the frontend, which has then been told where to open, and the
 * caller leaves GM_MENU for GM_FRONTEND instead of starting the native menu's think. */
bool gmFrontend_NativeReturn(int kind, int sel);

/* True once, after the loading screen has run: the in-match hold (gmscene.c) then has nothing
 * left to warm and stands down. */
bool gmFrontend_TakeWarmed(void);

#endif

#endif
