/* atlas-items: the value rule of a list row (the legacy fe_change rule as a pure function) and the text a row shows. */
#include "atlas_check.h"
#include "../platform/gw_ui_screen.h"
#include "../platform/gw_ui_item.h"

static int app(int kind, int mn, int mx, int st, int cur, int dir, int *out) { return at_item_apply(kind, mn, mx, st, cur, dir, out); }

static void apply_choice_wraps(void)
{
    int v;
    CHECK(app(AT_VAL_CHOICE, 0, 5, 1, 5, +1, &v) == 1 && v == 0);          /* Render Scale: 4x wraps to Auto */
    CHECK(app(AT_VAL_CHOICE, 0, 5, 1, 0, -1, &v) == 1 && v == 5);
    CHECK(app(AT_VAL_CHOICE, -1, 4, 1, 4, +1, &v) == 1 && v == -1);       /* Items: min is -1 (the legacy "Off" slot): the wrap uses min..max, not 0..n */
    CHECK(app(AT_VAL_CHOICE, -1, 4, 1, -1, -1, &v) == 1 && v == 4);
    CHECK(app(AT_VAL_CHOICE, 0, 1, 1, 0, +1, &v) == 1 && v == 1);
    CHECK(app(AT_VAL_CHOICE, 0, 3, 1, 1, +1, &v) == 1 && v == 2);          /* no wrap in the middle */
}
static void apply_slider_clamps_and_steps(void)
{
    int v;
    CHECK(app(AT_VAL_SLIDER, 0, 100, 5, 95, +1, &v) == 1 && v == 100);     /* Master Volume, step 5 */
    CHECK(app(AT_VAL_SLIDER, 0, 100, 5, 100, +1, &v) == 0 && v == 100);    /* held at an end: unchanged, the caller bumps */
    CHECK(app(AT_VAL_SLIDER, -100, 100, 5, -98, -1, &v) == 1 && v == -100);/* Music / Effects: -98 - 5 clamps to -100 */
    CHECK(app(AT_VAL_SLIDER, -1, 50, 1, -1, -1, &v) == 0);                 /* Main Dead Zone: step 1 across -1..50 (the Lua rule would be 2) */
    CHECK(app(AT_VAL_SLIDER, -1, 50, 1, -1, +1, &v) == 1 && v == 0);
    CHECK(app(AT_VAL_SLIDER, 0, 40, 2, 38, +1, &v) == 1 && v == 40);       /* Stick Dead Zone, step 2 */
    CHECK(app(AT_VAL_SLIDER, 1, 99, 0, 1, +1, &v) == 1 && v == 2);         /* step 0 or less is 1 (legacy: it->step > 0 ? it->step : 1) */
    CHECK(app(AT_VAL_SLIDER, 1, 99, -3, 50, +1, &v) == 1 && v == 51);
    CHECK(app(AT_VAL_SLIDER, 5, 20, 1, 20, +1, &v) == 0);                  /* Damage Ratio */
    CHECK(app(AT_VAL_SLIDER, 0, 100, 5, 120, -1, &v) == 1 && v == 100);    /* a value already past the range is pulled in */
}
static void apply_toggle(void)
{
    int v;
    CHECK(app(AT_VAL_TOGGLE, 0, 1, 1, 0, +1, &v) == 1 && v == 1);
    CHECK(app(AT_VAL_TOGGLE, 0, 1, 1, 1, -1, &v) == 1 && v == 0);          /* left and right both flip */
    CHECK(app(AT_VAL_TOGGLE, 0, 1, 1, 7, +1, &v) == 1 && v == 0);          /* any non-zero is on (the legacy !v) */
}
static void apply_rejects_readouts(void)
{
    int v = 7;
    CHECK(app(AT_VAL_TEXT, 0, 0, 0, 0, +1, &v) == 0 && v == 7);
    CHECK(app(AT_VAL_COUNTER, 0, 0, 0, 0, +1, &v) == 0 && v == 7);
    CHECK(app(AT_VAL_NONE, 0, 0, 0, 0, +1, &v) == 0);
    CHECK(app(AT_VAL_SLIDER, 5, 5, 1, 5, +1, &v) == 0);                    /* max == min: no range, no change, no divide by zero */
    CHECK(app(AT_VAL_CHOICE, 3, 2, 1, 3, +1, &v) == 0);                    /* an inverted range is refused, not wrapped through a negative n */
    CHECK(app(AT_VAL_CHOICE, 4, 4, 1, 4, +1, &v) == 0);                    /* one option: nothing to change to */
    CHECK(app(99, 0, 1, 1, 0, +1, &v) == 0 && v == 7);                     /* an unknown kind */
    CHECK(app(AT_VAL_SLIDER, 0, 10, 1, 5, +1, NULL) == 0);                 /* no out pointer: refused, not written */
}
static void text_of_values(void)
{
    AtItem it;
    char o[40];
    memset(&it, 0, sizeof it);
    it.vkind = AT_VAL_CHOICE; it.vmin = 0; it.vmax = 2; it.n_opts = 3;
    snprintf(it.opt[0], 24, "%s", "Off"); snprintf(it.opt[1], 24, "%s", "FPS"); snprintf(it.opt[2], 24, "%s", "Performance");
    at_item_text(it.vkind, 1, it.vmin, (const char (*)[24]) it.opt, it.n_opts, it.text, o, sizeof o); CHECK_STR(o, "FPS");
    at_item_text(it.vkind, 7, it.vmin, (const char (*)[24]) it.opt, it.n_opts, it.text, o, sizeof o); CHECK_STR(o, "");     /* out of range: empty, never a read past the options */
    at_item_text(it.vkind, -1, it.vmin, (const char (*)[24]) it.opt, it.n_opts, it.text, o, sizeof o); CHECK_STR(o, "");
    it.vmin = -1;
    at_item_text(it.vkind, 0, it.vmin, (const char (*)[24]) it.opt, it.n_opts, it.text, o, sizeof o); CHECK_STR(o, "FPS");   /* the index is value - min (Items: -1 is the first option) */
    snprintf(it.text, AT_STR, "%s", "Shoulder Jump");
    at_item_text(it.vkind, 0, 0, NULL, 0, it.text, o, sizeof o); CHECK_STR(o, "Shoulder Jump");                              /* a choice with no options shows its text (Profile) */
    it.vkind = AT_VAL_TOGGLE; at_item_text(it.vkind, 1, 0, NULL, 0, it.text, o, sizeof o); CHECK_STR(o, "ON");
    at_item_text(it.vkind, 0, 0, NULL, 0, it.text, o, sizeof o); CHECK_STR(o, "OFF");
    it.vkind = AT_VAL_SLIDER; it.text[0] = 0; at_item_text(it.vkind, 45, 0, NULL, 0, it.text, o, sizeof o); CHECK_STR(o, "45");
    snprintf(it.text, AT_STR, "%s", "45%");
    at_item_text(it.vkind, 45, 0, NULL, 0, it.text, o, sizeof o); CHECK_STR(o, "45%");                                      /* a formatted slider shows its text */
    it.vkind = AT_VAL_TEXT; snprintf(it.text, AT_STR, "%s", "Port 1: Nothing connected");
    at_item_text(it.vkind, 0, 0, NULL, 0, it.text, o, sizeof o); CHECK_STR(o, "Port 1: Nothing connected");
    at_item_text(it.vkind, 0, 0, NULL, 0, it.text, o, 8); CHECK_STR(o, "Port 1:");                                            /* cut to the buffer, always terminated */
    at_item_text(it.vkind, 0, 0, NULL, 0, it.text, o, 0);                                                                     /* a zero buffer: nothing written, no crash */
    it.vkind = AT_VAL_NONE; at_item_text(it.vkind, 0, 0, NULL, 0, "", o, sizeof o); CHECK_STR(o, "");
}
static void capacity(void)
{
    CHECK(AT_MAX_ITEMS >= 64 && AT_MAX_OPTS >= 6);                          /* Render Scale has 6 options, the MODS page 41 rows */
    CHECK(AT_MAX_ITEMS_LUA == 32);                                          /* the Lua door keeps its documented limit */
    CHECK(AT_VAL_TOGGLE == 1 && AT_VAL_CHOICE == 2 && AT_VAL_SLIDER == 3 && AT_VAL_TEXT == 4);   /* the game side writes these numbers (gmfrontend_atlas_table.h) */
    CHECK(AT_ITEM_A_STEPS == 1 && AT_ITEM_RO == 2 && AT_ITEM_DANGER == 4 && AT_ITEM_DISABLED == 8);
}
int main(void) { apply_choice_wraps(); apply_slider_clamps_and_steps(); apply_toggle(); apply_rejects_readouts(); text_of_values(); capacity(); ATLAS_DONE("atlas items"); }
