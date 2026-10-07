/* gw_ui_frame.h - a framed screen: Atlas chrome around a window onto a retail scene (spec 13.10). Pure C: no game, no renderer.
 *
 * A bespoke retail scene (the Trophy Gallery, the Collection, the Lottery) runs unchanged and draws its own 3D. Atlas draws opaque plates around
 * a rectangular window and the chrome (trail, explainer, counter, key hints) on the plates; nothing is drawn inside the window. A framed
 * screen has no focus and no hit rectangles: retail owns the pad, and the mouse is not supported on it in version 1. */
#ifndef GW_UI_FRAME_H
#define GW_UI_FRAME_H
#include "gw_ui_layout.h"
#ifdef __cplusplus
extern "C" {
#endif

/* the window in the 640x480 retail canvas: where retail's camera frames its model. Retail pictures of these scenes are a 4:3 band centred in
 * a wider canvas (gw_view_math.h: gw_view_scene_wide is false for the toy scenes), so a wider canvas shifts the window right by half the extra. */
typedef struct { float x, y, w, h; } AtFrameRect;
typedef struct { AtRect trail, explainer, keys; int have_trail, have_explainer, have_keys, explainer_overlaps; } AtFrameSlots;

#define AT_FRAME_EXPLAIN_H 200.0f   /* what the explainer part needs without a media well: kicker, title, four rule lines, FROM */
#define AT_FRAME_GAP 8.0f

/* The window on the arranged canvas. */
AtRect at_frame_hole(const AtLayout *L, AtFrameRect win);
/* The opaque plates around the hole: top, bottom, left, right in that order, only those with room. Returns the count; together with the hole
 * they tile the canvas exactly. A window with no size gives the whole canvas as one plate (nothing is framed). */
int at_frame_plates(const AtLayout *L, AtFrameRect win, AtRect out[4]);
/* Where each chrome slot goes. A slot that would intersect the hole is dropped (trail, keys) or becomes a card over the hole's corner
 * (the explainer) and says so. explainer_w is the preset's width. */
void at_frame_slots(const AtLayout *L, AtFrameRect win, float explainer_w, AtFrameSlots *out);

#ifdef __cplusplus
}
#endif
#endif
