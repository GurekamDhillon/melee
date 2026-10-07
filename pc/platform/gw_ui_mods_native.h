/* gw_ui_mods_native.h - the MODS screen's real accessors: an AtModsSrc over gw_Mods_* (gw_mods.h). The door that draws the screen and feeds it
 * events is gw_script_ui_mods.inc; this file is only the mapping, so the model (gw_ui_mods.c) stays free of the game. */
#ifndef GW_UI_MODS_NATIVE_H
#define GW_UI_MODS_NATIVE_H
#include "gw_ui_mods.h"
#ifdef __cplusplus
extern "C" {
#endif

/* What the accessors cannot know themselves. Both may be NULL. The struct must outlive the source (the source keeps a pointer to it). */
typedef struct {
    int (*locked)(void);                        /* 1 while mods must not be changed: a netplay session or rollback is on */
    int (*entry_ok)(const char *entry_id);      /* 1 when the registry holds that entry and can open it (a mod's settings entry) */
} AtModsHooks;

void at_mods_native_src(AtModsSrc *out, const AtModsHooks *hooks);

#ifdef __cplusplus
}
#endif
#endif
