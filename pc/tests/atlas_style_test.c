#include "atlas_check.h"
#include "atlas_fake.h"
#include "atlas_style.h"

static const AtTextOps OPS = { fake_width, NULL };

/* A correct chamfered plate: ground first, then at_plate (3 quads: face, the two chamfered ends, front edge). */
static void draw_plate(AtRect r, float c, int broken)
{
    AtSink s = rec_sink();
    at_poly_rect(&s, 0, 0, 640, 480, AT_C_GROUND);
    at_plate(&s, r, AT_C_PLATE2, AT_C_EDGE, 3.0f, c);
    if (broken) at_poly_rect(&s, r.x, r.y, 14, 14, AT_C_PLATE2);   /* a square drawn over the top-left chamfer: the bug step 1 had */
}

int main(void)
{
    AtRect r = { 100, 100, 120, 40 };
    StySig a, b;
    /* the check passes a correct plate ... */
    draw_plate(r, 5.0f, 0);
    CHECK(sty_chamfer(r, 5.0f, AT_C_GROUND) == 0);
    /* ... and FAILS a plate with a square drawn over the cut (the check can fail) */
    draw_plate(r, 5.0f, 1);
    CHECK(sty_chamfer(r, 5.0f, AT_C_GROUND) == 1);
    /* a plate with all four corners square fails too (the other two corners must be filled, these two cut) */
    { AtSink s = rec_sink(); at_poly_rect(&s, 0, 0, 640, 480, AT_C_GROUND); at_poly_rect(&s, r.x, r.y, r.w, r.h, AT_C_PLATE2); }
    CHECK(sty_chamfer(r, 5.0f, AT_C_GROUND) == 1);
    /* a plate whose other corners are cut as well is named too: top-right (3) and bottom-left (4) */
    { AtSink s; draw_plate(r, 5.0f, 0); s = rec_sink(); at_poly_rect(&s, 0, 0, 640, 480, AT_C_GROUND); at_plate(&s, r, AT_C_PLATE2, AT_C_EDGE, 3.0f, 5.0f);
      at_poly_rect(&s, r.x + r.w - 3, r.y, 3, 3, AT_C_GROUND);
      CHECK(sty_chamfer(r, 5.0f, AT_C_GROUND) == 3);
      s = rec_sink(); at_poly_rect(&s, 0, 0, 640, 480, AT_C_GROUND); at_plate(&s, r, AT_C_PLATE2, AT_C_EDGE, 3.0f, 5.0f);
      at_poly_rect(&s, r.x, r.y + r.h - 3, 3, 3, AT_C_GROUND);
      CHECK(sty_chamfer(r, 5.0f, AT_C_GROUND) == 4); }

    /* focus cues: a row at rest and focused; the focused one must show lift, ember edge and a tick */
    { AtItem it; AtSink s; AtRect row = { 40, 100, 300, 34 };
      memset(&it, 0, sizeof it); snprintf(it.label, sizeof it.label, "%s", "Stocks");
      s = rec_sink(); at_part_row(&s, &OPS, row, &it, AT_ST_REST);  a = sty_sig(0);
      s = rec_sink(); at_part_row(&s, &OPS, row, &it, AT_ST_FOCUS); b = sty_sig(0);
      CHECK(sty_focus_cues(a, b) == 3);
      CHECK(sty_focus_cues(a, a) == 0); }                                  /* identical draws have no cue: the check can fail */
    /* the same cues read out of a whole render by area: the ground's own polys do not hide the lift, a part elsewhere does not count */
    { AtItem it; AtSink s; AtRect row = { 40, 100, 300, 34 }, area = { 30, 90, 320, 50 }, other = { 40, 300, 300, 34 };
      memset(&it, 0, sizeof it); snprintf(it.label, sizeof it.label, "%s", "Stocks");
      s = rec_sink(); at_poly_rect(&s, 0, 0, 640, 480, AT_C_GROUND); at_part_row(&s, &OPS, row, &it, AT_ST_REST); at_part_row(&s, &OPS, other, &it, AT_ST_FOCUS); a = sty_sig_in(0, area);
      s = rec_sink(); at_poly_rect(&s, 0, 0, 640, 480, AT_C_GROUND); at_part_row(&s, &OPS, row, &it, AT_ST_FOCUS); at_part_row(&s, &OPS, other, &it, AT_ST_FOCUS); b = sty_sig_in(0, area);
      CHECK(sty_focus_cues(a, b) == 3);
      CHECK(sty_focus_cues(sty_sig(0), sty_sig(0)) == 0);
      s = rec_sink(); at_poly_rect(&s, 0, 0, 640, 480, AT_C_GROUND); at_part_row(&s, &OPS, row, &it, AT_ST_REST); at_part_row(&s, &OPS, other, &it, AT_ST_REST); a = sty_sig_in(0, area);
      s = rec_sink(); at_poly_rect(&s, 0, 0, 640, 480, AT_C_GROUND); at_part_row(&s, &OPS, row, &it, AT_ST_REST); at_part_row(&s, &OPS, other, &it, AT_ST_FOCUS); b = sty_sig_in(0, area);
      CHECK(sty_focus_cues(a, b) == 0); }                                  /* a focus outside the area is not a cue inside it */
    /* a cell: the cues are lift, ember edge and four brackets */
    { AtCell c; AtSink s; AtRect cell = { 40, 100, 40, 40 };
      memset(&c, 0, sizeof c); c.model = AT_NO_MODEL; snprintf(c.name, sizeof c.name, "%s", "FOX");
      s = rec_sink(); at_part_cell(&s, &OPS, cell, &c, AT_ST_REST, AT_C_P1);  a = sty_sig(0);
      s = rec_sink(); at_part_cell(&s, &OPS, cell, &c, AT_ST_FOCUS, AT_C_P1); b = sty_sig(0);
      CHECK(sty_focus_cues(a, b) == 3); }

    /* text inside its pane: a long label in a narrow pane is caught */
    { AtSink s = rec_sink(); AtRect pane = { 10, 10, 60, 20 };
      at_text(&s, &OPS, AT_R_ROW16, "A very long label indeed", 12, 26, AT_C_IVORY, AT_ALIGN_LEFT, 0.0f);
      CHECK(sty_text_inside(pane, 0) == 0);                                /* unclipped, it overflows: index 0 is the offender */
      REC.nt = 0;
      at_text(&s, &OPS, AT_R_ROW16, "Fox", 12, 26, AT_C_IVORY, AT_ALIGN_LEFT, 0.0f);
      CHECK(sty_text_inside(pane, 0) == -1); }

    /* ports differ by shape: four different poly counts, and the same count for two ports is reported */
    { int ok[4] = { 3, 1, 3, 2 }, bad[4] = { 3, 1, 1, 2 };
      CHECK(sty_shapes_distinct(ok) == 0);                                  /* 3 appears twice: ports 1 and 3 */
      ok[2] = 4; CHECK(sty_shapes_distinct(ok) == 1);
      CHECK(sty_shapes_distinct(bad) == 0); }
    ATLAS_DONE("atlas style");
}
