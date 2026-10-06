#include "atlas_check.h"
#include "../platform/gw_ui_focus.h"

static AtFocusPos P(int b, int i) { AtFocusPos p; p.block = b; p.index = i; return p; }
static int is(AtFocusPos p, int b, int i) { return p.block == b && p.index == i; }

static void legacy_scenario(void)
{
    static const unsigned char a_exists[6] = { 1, 1, 1, 1, 0, 1 };      /* a5 is a hole */
    AtFocusBlock bl[3] = {
        { 0, 0, 3, 6, a_exists },   /* A: 3 columns, rows 0..1 */
        { 3, 0, 2, 4, NULL },       /* B: right of A, 2 columns; b3 is an EMPTY cell and focusable */
        { 0, 2, 3, 2, NULL },       /* K: row 2, two cells of three columns */
    };
    AtFocusPos f = at_focus_first(bl, 3);
    CHECK(is(f, 0, 0));
    f = at_focus_move(bl, 3, f, AT_DIR_RIGHT, 1); CHECK(is(f, 0, 1));
    f = at_focus_move(bl, 3, f, AT_DIR_RIGHT, 1); f = at_focus_move(bl, 3, f, AT_DIR_RIGHT, 1);
    CHECK(is(f, 1, 0));                                                 /* right crosses into B, same row */
    f = at_focus_move(bl, 3, f, AT_DIR_RIGHT, 1); CHECK(is(f, 1, 1));
    f = at_focus_move(bl, 3, f, AT_DIR_RIGHT, 1); CHECK(is(f, 0, 0));   /* the end of the row wraps to the first cell, same row */
    f = at_focus_move(bl, 3, f, AT_DIR_RIGHT, 1); f = at_focus_move(bl, 3, f, AT_DIR_RIGHT, 1);
    CHECK(is(f, 0, 2));
    f = at_focus_move(bl, 3, f, AT_DIR_DOWN, 1); CHECK(is(f, 0, 5));    /* down inside a block */
    f = at_focus_move(bl, 3, f, AT_DIR_DOWN, 1); CHECK(is(f, 2, 1));    /* down leaves the block, nearest column (k2 is under a3) */
    f = at_focus_move(bl, 3, f, AT_DIR_LEFT, 1); CHECK(is(f, 2, 0));
    f = at_focus_move(bl, 3, f, AT_DIR_LEFT, 1); CHECK(is(f, 2, 1));    /* wraps to the last existing cell of the row */
    f = at_focus_move(bl, 3, f, AT_DIR_DOWN, 1); CHECK(is(f, 0, 1));    /* bottom wraps to the top, same column */
    f = at_focus_move(bl, 3, f, AT_DIR_UP, 1);   CHECK(is(f, 2, 1));    /* top wraps to the bottom, same column */
    f = at_focus_move(bl, 3, P(0, 3), AT_DIR_RIGHT, 1); CHECK(is(f, 0, 5));   /* right skips the hole at a5 */
    f = at_focus_move(bl, 3, f, AT_DIR_LEFT, 1);        CHECK(is(f, 0, 3));   /* and left skips it back */
    f = at_focus_move(bl, 3, P(0, 5), AT_DIR_UP, 1);    CHECK(is(f, 0, 2));
    f = at_focus_move(bl, 3, P(0, 3), AT_DIR_DOWN, 1);  CHECK(is(f, 2, 0));   /* down from the last row crosses to K */
    f = at_focus_move(bl, 3, P(1, 2), AT_DIR_LEFT, 1);  CHECK(f.block >= 0);  /* the empty cell b3 holds focus and moves */
}

static void edges(void)
{
    AtFocusBlock one[1] = { { 0, 0, 3, 3, NULL } };
    AtFocusBlock solo[1] = { { 0, 0, 1, 1, NULL } };
    AtFocusBlock none[1] = { { 0, 0, 2, 0, NULL } };
    AtFocusPos f = P(0, 0);
    f = at_focus_move(one, 1, f, AT_DIR_LEFT, 0); CHECK(is(f, 0, 0));           /* wrap = 0 stops at the edge */
    f = at_focus_move(one, 1, P(0, 2), AT_DIR_RIGHT, 0); CHECK(is(f, 0, 2));
    f = at_focus_move(solo, 1, P(0, 0), AT_DIR_RIGHT, 1); CHECK(is(f, 0, 0));   /* a single cell never loops onto itself */
    f = at_focus_move(solo, 1, P(0, 0), AT_DIR_DOWN, 1);  CHECK(is(f, 0, 0));
    f = at_focus_first(none, 1); CHECK(f.block == -1 && f.index == -1);         /* an empty block has no focus */
    f = at_focus_move(one, 1, P(-1, -1), AT_DIR_DOWN, 1); CHECK(is(f, 0, 0));   /* an invalid position falls to the first cell */
    f = at_focus_move(one, 1, P(0, 9), AT_DIR_DOWN, 1);   CHECK(is(f, 0, 0));
}

static void bag_shape(void)
{
    /* the Envoy bag: equipped 6 / bag 4 / keystones 3, one block per row, all from column 0 */
    AtFocusBlock bl[3] = { { 0, 0, 6, 6, NULL }, { 0, 1, 4, 4, NULL }, { 0, 2, 6, 3, NULL } };
    AtFocusPos f;
    f = at_focus_move(bl, 3, P(0, 5), AT_DIR_DOWN, 1);  CHECK(is(f, 1, 3));     /* slot 6 is over bag cell 4, the nearest */
    f = at_focus_move(bl, 3, P(1, 3), AT_DIR_RIGHT, 1);
    /* inherited legacy behaviour, pinned on purpose: with nothing further right in its own row the focus goes to the
       nearest cell to the right in ANOTHER row (equipped slot 5) rather than wrapping within the row */
    CHECK(is(f, 0, 4));
    f = at_focus_move(bl, 3, P(2, 2), AT_DIR_DOWN, 1);  CHECK(is(f, 0, 2));     /* bottom wraps to the top, same column */
}

static void repeat(void)
{
    AtRepeat r = { 0, 0, 0 };
    CHECK(at_repeat_step(&r, AT_DIR_LEFT, 0.0) == AT_DIR_LEFT);                 /* the first press fires at once */
    CHECK(at_repeat_step(&r, AT_DIR_LEFT, 100.0) == 0);
    CHECK(at_repeat_step(&r, AT_DIR_LEFT, 299.0) == 0);
    CHECK(at_repeat_step(&r, AT_DIR_LEFT, 300.0) == AT_DIR_LEFT);               /* 300 ms delay, then every 80 ms */
    CHECK(at_repeat_step(&r, AT_DIR_LEFT, 379.0) == 0);
    CHECK(at_repeat_step(&r, AT_DIR_LEFT, 380.0) == AT_DIR_LEFT);
    CHECK(at_repeat_step(&r, 0, 400.0) == 0);                                   /* release resets */
    CHECK(at_repeat_step(&r, AT_DIR_RIGHT, 500.0) == AT_DIR_RIGHT);
    CHECK(at_repeat_step(&r, AT_DIR_UP, 510.0) == AT_DIR_UP);                   /* a new direction fires at once */
}

int main(void)
{
    legacy_scenario(); edges(); bag_shape(); repeat();
    ATLAS_DONE("atlas focus");
}
