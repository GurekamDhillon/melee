/* gw_ui_parts.h - the Atlas parts, drawn only through AtSink. Pure C. */
#ifndef GW_UI_PARTS_H
#define GW_UI_PARTS_H
#include "gw_ui_layout.h"
#include "gw_ui_screen.h"
#ifdef __cplusplus
extern "C" {
#endif

typedef struct AtSink {
    void *user;
    void (*poly)(void *u, const float x[4], const float y[4], unsigned rgba);                 /* a flat convex quad */
    void (*text)(void *u, float x, float base, const char *s, int role, unsigned rgba, int align, float max_w);
    void (*model)(void *u, int model, int ring, float x, float y, float w, float h, int focused, int dim);
} AtSink;

enum { AT_ALIGN_LEFT = 0, AT_ALIGN_CENTER = 1, AT_ALIGN_RIGHT = 2 };   /* the same numbers as GW_KIT_ALIGN_* */
enum { AT_ST_REST, AT_ST_FOCUS, AT_ST_PRESS, AT_ST_DISABLED, AT_ST_SELECTED };
enum { AT_TAG_PLAIN, AT_TAG_JADE, AT_TAG_EMBER, AT_TAG_SUN, AT_TAG_ROSE };
enum { AT_NOTE_OK, AT_NOTE_WARN, AT_NOTE_ERR, AT_NOTE_INFO };

void  at_poly_rect(const AtSink *s, float x, float y, float w, float h, unsigned rgba);
void  at_disc(const AtSink *s, float cx, float cy, float r, unsigned rgba);                    /* an octagon, three quads */
int   at_plate_polys(AtRect r, float edge, float chamfer, float px[3][4], float py[3][4]);     /* the geometry, for tests: 3 quads */
void  at_plate(const AtSink *s, AtRect r, unsigned face, unsigned edge_rgba, float edge, float chamfer);
void  at_text(const AtSink *s, const AtTextOps *o, int role, const char *str, float x, float base, unsigned rgba, int align, float max_w);

void  at_part_row(const AtSink *s, const AtTextOps *o, AtRect r, const AtItem *it, int state);
void  at_part_tabs(const AtSink *s, const AtTextOps *o, AtRect r, const char *const *names, const int *counts, int n, int active, int focus_tab);
float at_part_tag(const AtSink *s, const AtTextOps *o, float x, float y, const char *text, int tone, float max_w);   /* returns its width */
void  at_part_cell(const AtSink *s, const AtTextOps *o, AtRect r, const AtCell *c, int state, unsigned focus_rgba);
void  at_part_stone(const AtSink *s, const AtTextOps *o, AtRect r, const AtCell *c, int state, unsigned focus_rgba);
float at_part_hint(const AtSink *s, const AtTextOps *o, float x, float base, char btn, const char *label);        /* returns its advance */
float at_part_trail(const AtSink *s, const AtTextOps *o, AtRect r, const char *const *items, int n);              /* returns the end x */
void  at_part_chapter(const AtSink *s, const AtTextOps *o, AtRect r, int active);
void  at_part_rail(const AtSink *s, const AtTextOps *o, AtRect r, int active);
void  at_part_explainer(const AtSink *s, const AtTextOps *o, AtRect r, const AtExplainer *e);
void  at_part_footer(const AtSink *s, const AtTextOps *o, AtRect r, const AtFooter *f);
void  at_part_note(const AtSink *s, const AtTextOps *o, AtRect r, const char *text, int kind, float remaining);   /* remaining 0..1 */
int   at_part_dialog(const AtSink *s, const AtTextOps *o, float canvas_w, const AtDialog *d, float rise, AtRect btn[2]);

#ifdef __cplusplus
}
#endif
#endif
