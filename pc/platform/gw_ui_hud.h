/* gw_ui_hud.h - the in-match HUD layer (spec 6.8 part 3). Pure C. Never takes focus, never records a hit.
 *
 * A HUD is a script's description: parts in six zones inside the title-safe box. The engine places them (zones, stacking, the retail
 * HUD's keep-out rectangles, the quiet-HUD caps) and draws them every frame under the screen stack. Nothing here calls Lua. */
#ifndef GW_UI_HUD_H
#define GW_UI_HUD_H
#include "gw_ui_hud_parts.h"
#include "gw_ui_parts.h"
#include "gw_ui_retail_ids.h"
#include "gw_ui_val.h"
#ifdef __cplusplus
extern "C" {
#endif

enum { AT_Z_TOP_LEFT, AT_Z_TOP_CENTER, AT_Z_TOP_RIGHT, AT_Z_BOTTOM_LEFT, AT_Z_BOTTOM_CENTER, AT_Z_BOTTOM_RIGHT, AT_Z_COUNT };
enum { AT_HP_PORT_CARD = 1, AT_HP_TIMER, AT_HP_NOTE, AT_HP_STRIP, AT_HP_BANNER, AT_HP_TOAST, AT_HP_CARD,
       AT_HP_READOUT, AT_HP_TRACK, AT_HP_CHIPS };   /* the last three are read-only data parts (gw_ui_hud_parts.h): label and value rows, a move's timeline, a mode and its toggles */
#define AT_HUD_PER_ZONE 4
#define AT_HUD_KEEPOUTS 12
#define AT_HUD_QUAD_CAP 768
#define AT_ID_HUD (AT_ID * 2)

typedef struct {
    int kind; AtPortCard port; AtStrip strip;
    char text[AT_STR]; char rule[AT_TEXT]; char btn; float progress; unsigned rgba;
    char lines[3][AT_STR]; int n_lines;          /* AT_HP_CARD: the opponent card: a title (text) and up to 3 short lines */
    int seconds;                                  /* AT_HP_TIMER */
    int tone;                                     /* AT_HP_NOTE: AT_NOTE_OK, WARN, ERR or INFO (the default) */
    union { AtReadout readout; AtTrack track; AtChips chips; } data;   /* AT_HP_READOUT, AT_HP_TRACK, AT_HP_CHIPS */
    double from_ms, until_ms;                     /* AT_HP_TOAST / AT_HP_NOTE: shown on the UI clock; until_ms 0 = always */
} AtHudPart;
typedef struct { char id[AT_ID_HUD]; int owner; AtHudPart z[AT_Z_COUNT][AT_HUD_PER_ZONE]; int n[AT_Z_COUNT]; } AtHud;
typedef struct { AtRect r[AT_HUD_KEEPOUTS]; int n; } AtKeepOut;
typedef struct { AtRect rect[AT_Z_COUNT][AT_HUD_PER_ZONE]; int shown[AT_Z_COUNT][AT_HUD_PER_ZONE]; int dropped; } AtHudLayout;

AtRect at_hud_safe(float canvas_w);                              /* the title-safe box: content area, 16 px inside it, 16 px top and bottom */
/* The retail HUD's rectangles that are still visible. visible_mask has bit AT_RE_* set for every retail element that is drawn
 * (the complement of the retail mask); a hidden element frees its space. With everything visible: [0] the plates and stocks, [1] the timer. */
void   at_hud_retail_keepouts(float canvas_w, unsigned visible_mask, AtKeepOut *k);
int    at_hud_cap_ok(const AtHud *h, char *why, int cap);       /* the quiet-HUD caps; 0 with the reason in why */
void   at_hud_layout(const AtHud *h, float canvas_w, const AtKeepOut *k, double now_ms, const AtTextOps *o, AtHudLayout *out);
/* draws every shown part; entries counts what was sent to the sink (at most AT_HUD_QUAD_CAP: later ones are dropped) */
void   at_hud_render(const AtHud *h, const AtHudLayout *l, double now_ms, int reduced, const AtTextOps *o, const AtSink *s, int *entries);

/* The zone names ("top_left" ... "bottom_right") and their numbers. */
const char *at_hud_zone_name(int zone);
int    at_hud_zone_by_name(const char *name);                     /* -1 when it is not a zone */
/* 1 ok; 0 with the reason in err ("gd.ui.hud: ..."). owner_mod: the mod id the HUD's id must start with (NULL or "" for none). The cap
 * check (at_hud_cap_ok) is part of it. A note's from/until are taken from now_ms. */
int    at_hud_from_val(const AtvArena *a, int root, const char *owner_mod, double now_ms, AtHud *out, char *err, int errcap);

#ifdef __cplusplus
}
#endif
#endif
