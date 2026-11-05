#ifndef MELEE_GM_GMTITLEMODE_H
#define MELEE_GM_GMTITLEMODE_H

#include <melee/gm/types.h>

/* 1B087C */ void gmTitleMode_OnEnter(GameModeState*);
/* 3DD6A0 */ extern GameModeState gm_Mode_Title_States[];

#if defined(TARGET_PC)
/* The title's B / Y debug-menu shortcut (a PC addition): only with MELEE_DEBUG_MENU=1. The env read lives on the
   platform side (pc/platform/gw_runtime.c), as for PcTraceMotionEnabled. */
extern int PcDebugMenuShortcut(void);
#define gw_debug_menu_shortcut() PcDebugMenuShortcut()
#endif

#endif
