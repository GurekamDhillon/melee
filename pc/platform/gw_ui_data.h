/* gw_ui_data.h - a long data list as an Atlas list screen: a window over the rows, formatters, a row turned into an item. Pure C.
 *
 * Step 8 (the retail screens rebuilt from data). A data screen is a list whose rows are the game's own numbers, read each frame by the game side and
 * handed over as scalars and strings (never a callback: game-side C cannot give the host a function pointer). The lists are longer than a screen
 * (51 events, up to 120 name tags), so the ADAPTER owns the cursor and a window of AT_DATA_ROWS rows is what the host holds; see gmfrontend_atlas_data.inc. */
#ifndef GW_UI_DATA_H
#define GW_UI_DATA_H
#include "gw_ui_screen.h"
#ifdef __cplusplus
extern "C" {
#endif
#define AT_DATA_ROWS 32   /* rows the host holds for a data list (the record holds AT_MAX_ITEMS); the adapter windows the rest */
enum { AT_DR_DONE = 1, AT_DR_LOCKED = 2, AT_DR_NEW = 4 };
typedef struct { char label[AT_STR], value[AT_STR], sub[AT_STR]; unsigned flags; } AtDataRow;
typedef struct {
    char id[AT_ID * 2], title[AT_STR];
    int total, first, n, focus;            /* total rows; the window's first row; rows below; the focused row (absolute) */
    AtDataRow row[AT_DATA_ROWS];
} AtDataView;

/* the window's first row that keeps `focus` inside it: never moves the window while the focus is in it, slides by one at an edge, clamps to the
 * last full page, and never scrolls a list shorter than the window. A focus outside 0..total-1 is pulled in first. */
int  at_data_first(int total, int per, int focus, int first);
void at_data_counter(char *out, int cap, int focus, int total);          /* "3 / 51"; empty for an empty list */
void at_fmt_count(char *out, int cap, unsigned v);                        /* 1,234,567 */
void at_fmt_hm(char *out, int cap, unsigned seconds);                     /* H:MM, hours not capped at 24 */
void at_fmt_frames(char *out, int cap, unsigned frames);                  /* MM:SS CC from 60 fps frames (the retail Event record rule) */
void at_fmt_date(char *out, int cap, int year, int month, int day);       /* 2026-10-06 */
/* a row as a list item: id given by the caller. Done rows get the jade edge, the tag CLEARED and the word on the sub line; a locked row is a disabled
 * item (its sub line, or "Locked.", is the reason) so a click does nothing. The item is read-only: a data row is never stepped. */
void at_data_fill_item(AtItem *it, const char *id, const AtDataRow *r);
int  at_data_screen(const AtDataView *v, AtScreen *sc);                   /* 1 filled; 0 when the window does not fit the record */
#ifdef __cplusplus
}
#endif
#endif
