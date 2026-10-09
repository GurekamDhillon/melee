/* gw_ui_render.h - compose one Atlas screen from its parts. Pure C: draws only through AtSink. */
#ifndef GW_UI_RENDER_H
#define GW_UI_RENDER_H
#include "gw_ui_input.h"
#include "gw_ui_parts.h"
#include "gw_ui_screen.h"
#ifdef __cplusplus
extern "C" {
#endif

#define AT_SCREEN_QUAD_CAP 4096   /* one screen's entries in the kit quad list: the cap */
#define AT_SCREEN_QUAD_WARN 3000  /* and the line the host logs a warning at */

/* Draws the screen for a canvas of width canvas_w at time now_ms (the UI clock). `reduced` turns the open fade
 * into a cut. Fills `hits` with the rectangles the mouse can reach (cells or rows, key hints, dialog buttons). */
/* What one render did, for the host to log once: the entries the screen spent (a quad is 1, a text its glyphs, a model
 * AT_MODEL_COST), whether the warn line or the cap was reached, how many entries were not forwarded past the cap, and
 * how many hit rectangles did not fit in AtHits. */
#define AT_MODEL_COST 150
typedef struct { int entries, dropped, hits_dropped, warned, capped; } AtRenderInfo;

void at_render(const AtScreen *sc, const AtView *v, float canvas_w, double now_ms, int reduced,
               const AtTextOps *o, const AtSink *s, AtHits *hits);
/* The same, and fills `info` (may be NULL). Draws through a counting wrapper: nothing is forwarded past AT_SCREEN_QUAD_CAP. */
void at_render_ex(const AtScreen *sc, const AtView *v, float canvas_w, double now_ms, int reduced,
                  const AtTextOps *o, const AtSink *s, AtHits *hits, AtRenderInfo *info);

/* The one place the primary pane's list and tiles geometry lives (the renderer, hit testing and the host's scroll all call it).
 * at_list_visible: the rows of a list that fit the primary pane. at_tiles_geometry: the rectangle of every tile (the count is
 * returned; a tile scrolled out of the window has h = 0) and of the More labels (m[0..n_more)). The _ex form takes the first
 * visible row, and reports the visible and total rows and the More strip's rectangle. */
int at_list_visible(const AtLayout *L);
/* A list pane with group headings (22 px each) and 39 px rows: how many rows fit from `first` in a pane of height pane_h, and the first row that keeps `focus`
 * in view. With no groups the window is exactly at_list_visible's. A list with tabs has the shorter pane (at_layout_split). */
#define AT_CREDIT_H 22.0f                                  /* the strip a list's credit line takes from the bottom of its pane */
float at_item_pitch(const AtItem *it);                      /* a list row's step: its height plus the 5 px gap (39 for a plain row) */
int at_list_window(const AtScreen *sc, float pane_h, int first);
int at_list_scroll_to(const AtScreen *sc, float pane_h, int focus, int scroll);
/* the rectangle of every offer card of a cards screen (the count is returned), for the renderer, hit testing and tests */
int at_cards_geometry(const AtScreen *sc, const AtLayout *L, AtRect *out, int cap);
int at_tiles_geometry(const AtScreen *sc, const AtLayout *L, AtRect *tiles, int cap, AtRect *more, int more_cap);
int at_tiles_geometry_ex(const AtScreen *sc, const AtLayout *L, int scroll, AtRect *tiles, int cap, AtRect *more, int more_cap,
                         int *rows_visible, int *rows_total, AtRect *strip);
/* the first visible tile row that keeps the focused tile in view */
int at_tiles_scroll(const AtScreen *sc, const AtLayout *L, int focus_index, int scroll);

#ifdef __cplusplus
}
#endif
#endif
