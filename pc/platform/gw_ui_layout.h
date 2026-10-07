/* gw_ui_layout.h - Atlas: rectangles, the text-role table, text fit and wrap, and the four-place layout.
 * Pure C: no game, no Lua, no renderer. Text is measured through AtTextOps, so the same code runs against
 * the real font (gw_script_ui.inc) and a fixed-width fake (pc/tests/atlas_fake.h). */
#ifndef GW_UI_LAYOUT_H
#define GW_UI_LAYOUT_H
#ifdef __cplusplus
extern "C" {
#endif

typedef struct { float x, y, w, h; } AtRect;

/* The Atlas text roles. The names are the keys of the font manifest (menu/pipeline/kit.py ATLAS_TYPE_SCALE). */
typedef enum {
    AT_R_CAP12, AT_R_CAP14, AT_R_CAP16, AT_R_CAP20, AT_R_TITLE, AT_R_HERO, AT_R_DISPLAY,
    AT_R_BODY12, AT_R_BODY14, AT_R_ROW16, AT_R_NUM12, AT_R_NUM14, AT_R_NUM16, AT_R_COUNT
} AtRole;

typedef struct {
    const char *name;
    int size;      /* px at 640x480; never under 12 */
    int face;      /* 0 condensed, 1 sans, 2 numerals: the fit rule steps down inside one face */
    int caps_only;
    int tracked;   /* the adapter adds +0.10 em letter-spacing */
    int smaller;   /* the next role down in the same face, or -1 */
} AtRoleInfo;

const AtRoleInfo *at_role(int role);
int at_role_size(int role);
int at_role_by_name(const char *name); /* -1 when it is not an Atlas role */

typedef struct AtTextOps {
    float (*width)(void *user, int role, const char *s); /* px at 1x, tracking included */
    void *user;
} AtTextOps;

/* The fit rule: too wide -> the next smaller role of the same face -> truncate with an ellipsis (UTF-8
 * U+2026). `out` receives the text; returns the role it fits in. max_w <= 0 means no limit. */
int at_fit(const AtTextOps *ops, int role, const char *s, float max_w, char *out, int cap);

/* Word wrap to max_w: breaks at spaces and '\n'. At most max_lines lines of 95 bytes; a clamped block ends its
 * last line with an ellipsis and sets *clamped. Returns the line count. */
int at_wrap(const AtTextOps *ops, int role, const char *s, float max_w, int max_lines, char lines[][96], int *clamped);

enum { AT_PRESET_NARROW, AT_PRESET_NORMAL, AT_PRESET_WIDE, AT_PRESET_NONE };

typedef struct {
    int wide;
    AtRect canvas, header, trail, chapter, rule, body, primary, explainer, keys, rail;
    float content_x, content_w;
} AtLayout;

/* The four places for a canvas width (>= 640) and an explainer preset. Compact below 760, wide from 760;
 * content at most 1140 wide and centred; 32 px margins inside it. */
void at_layout(float canvas_w, int preset, AtLayout *out);

/* The primary pane split for a screen with tabs and a band (the character select): the tab strip (30) hangs on the pane's top edge, the band
 * (56 for port cards, 40 for the matchup strip) sits at the bottom of the primary place with a 12 px gap above it, and the grid pane is
 * what is left. A strip that is not wanted has h = 0. Everything keeps the primary place's x and width. */
enum { AT_BAND_NONE = 0, AT_BAND_CARDS = 1, AT_BAND_MATCHUP = 2 };   /* the strip between the primary pane and the keys */
typedef struct { AtRect tabs, grid, band; } AtSplit;
void at_layout_split(const AtLayout *L, int has_tabs, int band, AtSplit *out);
/* How many columns a grid of `width` px takes: as many cells of at least min_cell as fit, and more when a cell would be wider than max_cell
 * (a wider window gives more columns, never bigger cells), but never so many that a cell drops under min_cell. At least 1. */
int at_grid_cols(float width, float min_cell, float max_cell, float gap);

/* The first visible row of a scrolling list that keeps `focus` inside a window of `visible` rows. */
int at_list_scroll(int focus, int scroll, int visible, int n);

#ifdef __cplusplus
}
#endif
#endif
