#include "atlas_lint.h"
#include "../platform/gw_ui_tokens.h"

/* a plate with the WRONG chamfer (top-right and bottom-left cut), built from three convex bands */
static void bad_plate(const AtSink *s, AtRect r, float c, unsigned rgba)
{
    float xa[4] = { r.x, r.x + r.w - c, r.x + r.w, r.x },               ya[4] = { r.y, r.y, r.y + c, r.y + c };
    float xb[4] = { r.x, r.x + r.w, r.x + r.w, r.x },                   yb[4] = { r.y + c, r.y + c, r.y + r.h - c, r.y + r.h - c };
    float xc[4] = { r.x, r.x + r.w, r.x + r.w, r.x + c },               yc[4] = { r.y + r.h - c, r.y + r.h - c, r.y + r.h, r.y + r.h };
    s->poly(s->user, xa, ya, rgba);
    s->poly(s->user, xb, yb, rgba);
    s->poly(s->user, xc, yc, rgba);
}

int main(void)
{
    AtRect r = { 40.0f, 60.0f, 120.0f, 34.0f };
    AtSink s;

    /* the real plate passes: TL and BR cut, TR and BL square */
    s = rec_sink(); at_plate(&s, r, AT_C_PLATE2, AT_C_EDGE2, 3.0f, 5.0f);
    CHECK(lint_plate_corners(r, 5.0f) == 0);
    /* a hand-made TR/BL chamfer fails on all four corners */
    s = rec_sink(); bad_plate(&s, r, 8.0f, AT_C_PLATE2);
    CHECK(lint_plate_corners(r, 8.0f) == 4);
    /* a chamfer of 2 px or less is not probed (a 1 px inset cannot tell it from square) */
    s = rec_sink(); at_poly_rect(&s, r.x, r.y, r.w, r.h, AT_C_PLATE2);
    CHECK(lint_plate_corners(r, 2.0f) == 0);

    /* text inside / overlapping, with the fake width (half the role size per character) */
    s = rec_sink();
    at_text(&s, &FAKE, AT_R_ROW16, "Short", 50.0f, 80.0f, AT_C_IVORY, AT_ALIGN_LEFT, 0.0f);
    CHECK(lint_text_inside(r, 0) == 0);
    at_text(&s, &FAKE, AT_R_ROW16, "This label is far too long for its plate", 50.0f, 80.0f, AT_C_IVORY, AT_ALIGN_LEFT, 0.0f);
    CHECK(lint_text_inside(r, 0) == 1);                     /* the second text runs out */
    s = rec_sink();
    at_text(&s, &FAKE, AT_R_ROW16, "AAAAAAAA", 50.0f, 80.0f, AT_C_IVORY, AT_ALIGN_LEFT, 0.0f);
    at_text(&s, &FAKE, AT_R_ROW16, "BBBBBBBB", 80.0f, 82.0f, AT_C_IVORY, AT_ALIGN_LEFT, 0.0f);
    CHECK(lint_text_overlaps() == 1);
    s = rec_sink();
    at_text(&s, &FAKE, AT_R_ROW16, "AAAA", 50.0f, 80.0f, AT_C_IVORY, AT_ALIGN_LEFT, 0.0f);
    at_text(&s, &FAKE, AT_R_ROW16, "BBBB", 120.0f, 80.0f, AT_C_IVORY, AT_ALIGN_LEFT, 0.0f);
    CHECK(lint_text_overlaps() == 0);

    /* focus cues: a row at rest has none, a focused row has all three, a half-focused one fails */
    {
        AtItem it; memset(&it, 0, sizeof it); snprintf(it.label, sizeof it.label, "Row");
        s = rec_sink(); at_part_row(&s, &FAKE, r, &it, AT_ST_FOCUS);
        CHECK(lint_focus_row(r) == 0);
        s = rec_sink(); at_part_row(&s, &FAKE, r, &it, AT_ST_REST);
        CHECK(lint_focus_row(r) == 3);                      /* no lift, no ember edge, no tick */
        s = rec_sink(); at_plate(&s, r, AT_C_LIFT, AT_C_EMBER, 3.0f, 5.0f);   /* edge and face but no lift and no tick */
        CHECK(lint_focus_row(r) == 2);
    }
    {
        AtCell c; memset(&c, 0, sizeof c); c.model = AT_NO_MODEL; c.tex = -1; snprintf(c.name, sizeof c.name, "Cell");
        AtRect cr = { 100.0f, 200.0f, 48.0f, 48.0f };
        s = rec_sink(); at_part_cell(&s, &FAKE, cr, &c, AT_ST_FOCUS, AT_C_EMBER);
        CHECK(lint_focus_cell(cr, AT_C_EMBER) == 0);
        s = rec_sink(); at_part_cell(&s, &FAKE, cr, &c, AT_ST_REST, AT_C_EMBER);
        CHECK(lint_focus_cell(cr, AT_C_EMBER) == 3);
    }
    ATLAS_DONE("atlas lint");
}
