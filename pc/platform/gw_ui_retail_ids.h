/* gw_ui_retail_ids.h - retail element ids (spec 6.8). An enum only: included by host C and by game-side TUs under TARGET_PC. */
#ifndef GW_UI_RETAIL_IDS_H
#define GW_UI_RETAIL_IDS_H
enum {
    AT_RE_HUD_DAMAGE = 0,   /* the percent plates: ifStatus_802F5DE0 / ifStatus_802F5E50 */
    AT_RE_HUD_STOCK = 1,    /* stock icons: fn_802F9680, fn_802F94E0, fn_802F95E8, fn_802F9548, fn_802F9598 */
    AT_RE_HUD_TIMER = 2,    /* match timer and countdown: iftime.c GXLink sites */
    AT_RE_HUD_NAMETAG = 3, AT_RE_HUD_MAGNIFY = 4, AT_RE_HUD_COIN = 5, AT_RE_HUD_PRIZE = 6, AT_RE_HUD_HAZARD = 7,
    AT_RE_PAUSE_PANEL = 8,  /* the GmPause panel (gm_801A0FEC) */
    AT_RE_COUNT = 9
};
#endif
