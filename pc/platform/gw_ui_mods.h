/* gw_ui_mods.h - the MODS screen as a pure model: an accessor table in, an AtScreen and view out, events applied. No game, no
 * mods folder, no Lua. The real accessors (gw_Mods_* and the registry) are in gw_ui_mods_native.c.
 *
 * The screen never runs a mod's code. It reads what the mods API knows, writes mods/enabled.txt for the NEXT boot through
 * set_enabled and save, and says what it did. While `locked` says so (a netplay session), it changes nothing. */
#ifndef GW_UI_MODS_H
#define GW_UI_MODS_H
#include "gw_ui_input.h"
#include "gw_ui_parts.h"
#include "gw_ui_screen.h"
#ifdef __cplusplus
extern "C" {
#endif

#define AT_MODS_MAX 256                 /* GW_MODS_MAX in gw_mods.h */
#define AT_MODS_WINDOW AT_MAX_ITEMS     /* rows the record holds */
#define AT_MODS_DETAIL_VISIBLE 8        /* the detail has no tab strip: (362 - 28) / 39 */
#define AT_MODS_VISIBLE 7               /* rows the list shows under its tab strip: (362 - 42 - 28) / 39 */
/* the same meaning as GW_MOD_*; the real accessor maps them, so no number is shared */
enum { AT_MOD_ACTIVE, AT_MOD_OFF, AT_MOD_MISSING_DEP, AT_MOD_CONFLICT };

typedef struct AtModsSrc {
    void *user;
    int (*count)(void *u);
    const char *(*id)(void *u, int i);
    const char *(*name)(void *u, int i);
    const char *(*version)(void *u, int i);
    const char *(*kind)(void *u, int i);
    const char *(*pack)(void *u, int i);
    const char *(*desc)(void *u, int i);
    const char *(*requires)(void *u, int i);        /* comma-separated ids */
    const char *(*conflicts)(void *u, int i);       /* comma-separated ids: the mod's own list */
    int (*status)(void *u, int i);                  /* AT_MOD_*, for this boot */
    const char *(*status_text)(void *u, int i);
    int (*enabled)(void *u, int i);                 /* for the next boot */
    int (*active)(void *u, int i);                  /* mounted this boot */
    int (*set_enabled)(void *u, int i, int on);     /* how many mods changed, cascades included */
    int (*save)(void *u);                           /* 0 ok */
    int (*restart_needed)(void *u);
    int (*adds)(void *u, int i, char out[][AT_STR], int cap);                             /* "Solo > Envoy" lines from the registry */
    int (*settings)(void *u, int i, char *screen_id, int cap, char *label, int lcap);     /* 1 when the mod has its own settings entry */
    int (*locked)(void *u);                         /* optional: 1 while mods must not be changed (a netplay session) */
} AtModsSrc;

typedef struct {
    int tab;                         /* 0 installed, 1 conflicts */
    int sel[2], top[2];              /* per tab: the selected row and the first visible row, as indices into the tab's list */
    int list[2][AT_MODS_MAX], n[2];  /* mod indices per tab, rebuilt every build */
    int base;                        /* the first list row held in the record (the window) */
    char note[AT_STR]; int note_kind; double note_until;
} AtModsState;

enum { AT_MA_NONE, AT_MA_BACK, AT_MA_DETAIL, AT_MA_SETTINGS };
typedef struct { int kind, arg; char id[AT_ID * 2]; } AtModsAction;

void at_mods_state_init(AtModsState *st);
int at_mods_build(const AtModsSrc *s, AtModsState *st, AtScreen *sc, AtView *vw, double now_ms);
/* applies one event; fills *act; the caller rebuilds afterwards */
void at_mods_event(const AtModsSrc *s, AtModsState *st, const AtEvent *e, double now_ms, AtModsAction *act);
const char *at_mods_parent_label(const char *parent);                          /* "solo" -> "Solo"; unknown ids are returned as given */
void at_mods_add_line(const char *parent, const char *label, char *out, int cap);   /* "Solo > Envoy" */

/* the detail of one mod (Y on the list) */
typedef struct { int mod, sel; } AtModsDetail;
int at_mods_detail_build(const AtModsSrc *s, const AtModsDetail *d, AtScreen *sc, AtView *vw, double now_ms);
void at_mods_detail_event(const AtModsSrc *s, AtModsDetail *d, const AtEvent *e, AtModsAction *act);

#ifdef __cplusplus
}
#endif
#endif
