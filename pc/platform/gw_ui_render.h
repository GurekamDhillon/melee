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
void at_render(const AtScreen *sc, const AtView *v, float canvas_w, double now_ms, int reduced,
               const AtTextOps *o, const AtSink *s, AtHits *hits);

#ifdef __cplusplus
}
#endif
#endif
