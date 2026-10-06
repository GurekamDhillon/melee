/* gw_ui_screen.h - the Atlas screen record, its dynamic view, conversion from a value tree, validation, focus helpers. Pure C. */
#ifndef GW_UI_SCREEN_H
#define GW_UI_SCREEN_H
#include "gw_ui_focus.h"
#include "gw_ui_layout.h"
#include "gw_ui_val.h"
#ifdef __cplusplus
extern "C" {
#endif

#define AT_MAX_BLOCKS 6
#define AT_MAX_CELLS 12
#define AT_MAX_ITEMS 32
#define AT_MAX_KEYS 6
#define AT_MAX_WITH 4
#define AT_ID 24
#define AT_STR 64
#define AT_TEXT 160
#define AT_NO_MODEL (-1)

enum { AT_PRIMARY_LIST = 1, AT_PRIMARY_GRID = 2 };
enum { AT_CELL_LOCKED = 1, AT_CELL_EMPTY = 2, AT_CELL_MERGE = 4, AT_CELL_NEW = 8, AT_CELL_SELECTED = 16, AT_CELL_DISABLED = 32 };
enum { AT_VAL_NONE, AT_VAL_TOGGLE, AT_VAL_CHOICE, AT_VAL_SLIDER, AT_VAL_TEXT, AT_VAL_COUNTER };

typedef struct { char id[AT_ID]; char name[AT_STR]; int model, ring; unsigned flags; int index, pips; char origin; unsigned rgba; char letter; } AtCell;
typedef struct { char id[AT_ID]; char title[AT_STR]; char count[24]; char note[AT_STR]; int cols, n, stones; AtCell cells[AT_MAX_CELLS]; } AtBlock;
typedef struct { char id[AT_ID]; char label[AT_STR]; char sub[AT_STR]; unsigned flags; int vkind, on; char text[AT_STR]; int vmin, vmax, vval; } AtItem;
typedef struct { char btn; char label[AT_STR]; int fn_label, fn_when; } AtKey;
typedef struct { int has; char label[24]; int model_a, model_b, model_out; char text[AT_STR]; } AtFooter;
typedef struct { int has; int media_model, media_ring; char kicker[AT_STR], title[AT_STR], what[AT_TEXT]; int n_with, with_model[AT_MAX_WITH]; char from_text[AT_STR]; int warn; } AtExplainer;

typedef struct {
    char id[AT_ID * 2];
    char title[AT_STR];
    char parent[3][AT_STR]; int n_parents;
    int primary, preset, chapter;
    AtBlock blocks[AT_MAX_BLOCKS]; int n_blocks;
    AtItem items[AT_MAX_ITEMS]; int n_items;
    AtFooter footer;
    AtKey keys[AT_MAX_KEYS]; int n_keys;
    char counter[AT_STR]; int fn_counter;
    int input_feed, port;
    int fn_provide, fn_accept, fn_back, fn_alt[3], fn_focus, fn_change, fn_open, fn_close, fn_page, fn_start;
    int warnings;
} AtScreen;

typedef struct { char text[AT_STR]; int kind; double from_ms, until_ms; } AtNote;
typedef struct { int open; char title[AT_STR]; char body[AT_TEXT]; int n; char btn[2]; char label[2][24]; int focus; double from_ms; } AtDialog;
typedef struct {
    AtFocusPos focus; int scroll; AtExplainer ex;
    char key_label[AT_MAX_KEYS][AT_STR]; unsigned char key_shown[AT_MAX_KEYS];
    char counter[AT_STR]; AtNote note; AtDialog dialog; double opened_ms; unsigned port_rgba;
} AtView;

/* 1 ok; 0 with the reason in err. owner_mod: the mod id every screen id must start with (NULL or "" for none). */
int at_screen_from_val(const AtvArena *a, int root, const char *owner_mod, AtScreen *out, char *err, int errcap);
int at_explainer_from_val(const AtvArena *a, int t, AtExplainer *out, char *err, int errcap);   /* t = -1: an empty explainer */
int at_screen_fn_refs(const AtScreen *s, int *out, int cap);                                    /* every Lua reference the screen holds */
void at_view_init(AtView *v);
int at_screen_focus_blocks(const AtScreen *s, AtFocusBlock *fb);                                /* returns the block count */
const char *at_screen_block_id(const AtScreen *s, int block);
const char *at_screen_cell_id(const AtScreen *s, AtFocusPos p);                                 /* NULL when invalid */
AtFocusPos at_screen_refocus(const AtScreen *s, const char *block_id, const char *cell_id, AtFocusPos old);
int at_cell_accepts(const AtScreen *s, AtFocusPos p);                                           /* 0 for a disabled or missing cell */
int at_screen_wants_pad(const AtScreen *s);                                                     /* 0 when the script feeds its own pad input */

#ifdef __cplusplus
}
#endif
#endif
