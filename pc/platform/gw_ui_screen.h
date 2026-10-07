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
#define AT_MAX_CELLS 12               /* the Lua door's limit (documented in scripting.md): an inline block holds this many */
#define AT_MAX_EXT_CELLS 256          /* a native grid block's capacity (stages: 256), in adapter-owned storage */
#define AT_MAX_TABS 6
#define AT_MAX_CURSORS 4
#define AT_MAX_ITEMS 32
#define AT_MAX_KEYS 6
#define AT_MAX_WITH 4
#define AT_MAX_MORE 4
#define AT_ID 24
#define AT_STR 64
#define AT_TEXT 160
#define AT_NO_MODEL (-1)
#define AT_MAX_CARDS 4
#define AT_MAX_LINKS 16

enum { AT_PRIMARY_LIST = 1, AT_PRIMARY_GRID = 2, AT_PRIMARY_TILES = 3, AT_PRIMARY_DISPLAY = 4, AT_PRIMARY_CARDS = 5 };   /* the plan says CARDS = 3; 3 is TILES */
enum { AT_CELL_LOCKED = 1, AT_CELL_EMPTY = 2, AT_CELL_MERGE = 4, AT_CELL_NEW = 8, AT_CELL_SELECTED = 16, AT_CELL_DISABLED = 32,
       AT_CELL_BANNED = 64, AT_CELL_PICKED = 128, AT_CELL_UNSET = 256, AT_CELL_P1 = 512 };   /* 64 and up: strike marks, drawing only (step 6 sets them) */
enum { AT_CARD_OPEN = 1, AT_CARD_CLOSED = 2, AT_CARD_READY = 4, AT_CARD_FOCUS = 8 };   /* AtPortCard.flags */
enum { AT_VAL_NONE, AT_VAL_TOGGLE, AT_VAL_CHOICE, AT_VAL_SLIDER, AT_VAL_TEXT, AT_VAL_COUNTER };

/* tex: a disc-art texture slot, or -1 for none (a record built from Lua sets -1: a zeroed cell must never name texture 0); abbr: the two-letter
 * frame text drawn when there is no art. */
typedef struct { char id[AT_ID]; char name[AT_STR]; int model, ring; unsigned flags; int index, pips; char origin; unsigned rgba; char letter; int tex; char abbr[3]; } AtCell;
/* ext non-NULL: the cells live in adapter-owned storage (a native screen: a roster, the stages) and cells[] is unused. Read cells only
 * through at_block_cell and at_block_count. A screen with ext set is never copied (at_screen_copy refuses): the pointer would outlive its pool. */
typedef struct { char id[AT_ID]; char title[AT_STR]; char count[24]; char note[AT_STR]; int cols, n, stones; AtCell cells[AT_MAX_CELLS]; AtCell *ext; int ext_n; } AtBlock;
typedef struct { char name[24]; int count; } AtTab;
/* one port's card (the band): kind 0 off, 1 human, 2 CPU (the same numbers as AT_CSS_*); ck_tex: the fighter's art slot or -1; cur: its cursor cell */
typedef struct { int port, kind, ck_tex, cur; char name[AT_STR], sub[AT_STR]; unsigned flags; char abbr[3]; int cpu_lv, team; } AtSelCard;
/* an offer card: a model well (or, when model < 0 and letter != 0, a keystone arch stone with its letter), the name, ONE rule, a tag */
typedef struct { int model, ring; char name[AT_STR]; char rule[AT_TEXT]; char tag[24]; int tag_tone; unsigned rgba; char letter; } AtOffer;
typedef struct { char id[AT_ID]; int disabled; AtOffer offer; } AtCardRec;
typedef struct { char a[AT_ID], b[AT_ID]; unsigned rgba; } AtLink;      /* a grid link between two cells, by cell id */
typedef struct { char id[AT_ID]; char label[AT_STR]; char sub[AT_STR]; unsigned flags; int vkind, on; char text[AT_STR]; int vmin, vmax, vval;
               char icon[AT_ID]; char tag[16]; char badge[8]; char numeral[6]; } AtItem;   /* icon, tag, badge, numeral: tiles (hubs, the main menu) */
typedef struct { char btn; char label[AT_STR]; int fn_label, fn_when; } AtKey;
typedef struct { int has; char label[24]; int model_a, model_b, model_out; char text[AT_STR]; } AtFooter;
typedef struct { int has; int media_model, media_ring; char kicker[AT_STR], title[AT_STR], what[AT_TEXT]; int n_with, with_model[AT_MAX_WITH]; char from_text[AT_STR]; int warn;
               char with_text[AT_MAX_WITH][24]; int n_with_text;   /* with_text: tags such as "Melee", "Rules" */
               int media_tex; char media_abbr[3]; char stepper_label[16], stepper_text[24]; int stepper; } AtExplainer;   /* media_tex: a disc-art slot, -1 none; stepper: a "< 1 / 4 >" line (costume) */

typedef struct {
    char id[AT_ID * 2];
    char title[AT_STR];
    char parent[3][AT_STR]; int n_parents;
    int primary, preset, chapter;
    AtBlock blocks[AT_MAX_BLOCKS]; int n_blocks;
    AtItem items[AT_MAX_ITEMS]; int n_items;
    char hero[AT_STR], prompt[AT_STR], foot_left[24], foot_right[AT_STR];   /* a display screen (the title): wordmark, prompt, footers */
    int tile_cols;                                   /* tiles: 1 (main menu: big rows) or 2 (hubs); 0 = by count: <= 3 items -> 1, else 2 */
    AtItem more[AT_MAX_MORE]; int n_more;            /* tiles: the small More row under the tiles */
    AtFooter footer;
    AtTab tabs[AT_MAX_TABS]; int n_tabs;             /* a strip of tabs over the grid pane; 0 = none (the active one is AtView.tab) */
    int band;                                        /* AT_BAND_*: port cards or the matchup strip under the pane */
    AtSelCard ports[4];                               /* the character select's port cards (step 3's offer cards are `cards`) */
    int grid_cols_auto;                              /* 1: the renderer picks the grid's columns from the pane's width (at_grid_cols) */
    int grid_cell_min, grid_cell_max;                /* auto columns: the tile size range in px (0: 36 and 56, the character select's; the stage select uses 54 and 72) */
    AtKey keys[AT_MAX_KEYS]; int n_keys;
    char counter[AT_STR]; int fn_counter;
    int input_feed, port;
    AtCardRec cards[AT_MAX_CARDS]; int n_cards;      /* primary kind "cards": one row of offer cards */
    AtLink links[AT_MAX_LINKS]; int n_links, links_skipped;   /* a grid: lines drawn between cells under them; one naming a missing cell is skipped and counted */
    int has_countdown, countdown;                    /* seconds shown at the trail's right end (0:45, rose under 10 s). The script re-registers it once a second */
    int pause;                                       /* kind = "pause": a screen the retail pause takeover may push (a list primary; offline only) */
    int persist;                                     /* persist = true: the screen is not closed when the scene it was opened in ends (only its owner closes it) */
    int fn_provide, fn_accept, fn_back, fn_alt[3], fn_focus, fn_change, fn_open, fn_close, fn_page, fn_start;
    int warnings;
} AtScreen;

typedef struct { char text[AT_STR]; int kind; double from_ms, until_ms; } AtNote;
typedef struct { int open; char title[AT_STR]; char body[AT_TEXT]; int n; char btn[2]; char label[2][24]; int focus; double from_ms; } AtDialog;
typedef struct {
    AtFocusPos focus; int scroll; AtExplainer ex;
    char key_label[AT_MAX_KEYS][AT_STR]; unsigned char key_shown[AT_MAX_KEYS];
    char counter[AT_STR]; AtNote note; AtDialog dialog; double opened_ms; unsigned port_rgba;
    struct { int active, block, index, card; } cursor[AT_MAX_CURSORS];   /* one per port on a shared screen: card -1 = on the grid, else the band's card */
    int tab;                                          /* the active tab */
    int progress;                                     /* 0..1000, the loading bar */
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
int at_screen_tile_cols(const AtScreen *s);                                                     /* the columns a tiles screen uses: 1 or 2 */
int at_screen_wants_pad(const AtScreen *s);                                                     /* 0 when the script feeds its own pad input */
const AtCell *at_block_cell(const AtBlock *b, int i);                                           /* the ONE accessor: ext when set, else cells[]; NULL out of range */
int at_block_count(const AtBlock *b);                                                           /* ext_n (clamped to AT_MAX_EXT_CELLS) when ext is set, else n */
int at_screen_copy(AtScreen *dst, const AtScreen *src);                                         /* 1 copied; 0 (dst untouched) when any block has ext set */
void at_screen_clear_ext(AtScreen *s);                                                          /* every block back to inline cells, none */

#ifdef __cplusplus
}
#endif
#endif
