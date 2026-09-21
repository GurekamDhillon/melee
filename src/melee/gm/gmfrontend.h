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

#endif

#endif
