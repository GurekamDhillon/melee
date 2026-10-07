#include "atlas_lint.h"
#include "../platform/gw_ui_hud_parts.h"

static AtReadout ro(int n, int cols)
{
    AtReadout d; int i;
    memset(&d, 0, sizeof d);
    snprintf(d.title, sizeof d.title, "%s", "P1 FOX");
    d.cols = cols; d.n = n;
    for (i = 0; i < n; i++) { snprintf(d.row[i].label, sizeof d.row[i].label, "Row %d", i); snprintf(d.row[i].value, sizeof d.row[i].value, "%d.%d", i, i); d.row[i].tone = i % 4; }
    return d;
}

static void readout(void)
{
    AtSink s;
    AtReadout d = ro(12, 2);
    AtRect r;
    r.x = 8.0f; r.y = 8.0f; r.w = 400.0f; r.h = at_readout_height(&d);
    s = rec_sink(); at_part_readout(&s, &FAKE, r, &d);
    CHECK(lint_plate_corners(r, 8.0f) == 0);                                        /* the plate: 8 px chamfer, top-left and bottom-right */
    CHECK(lint_text_inside(r, 0) == 0 && lint_text_overlaps() == 0 && texts_legible());
    CHECK(find_text("P1 FOX") != NULL && find_text("Row 0") != NULL && find_text("Row 11") != NULL);
    CHECK(at_readout_height(&d) < 150.0f);                                          /* 12 rows in two columns: six lines */
    {   int i, ok = 0, warn = 0, bad = 0;                                            /* tones: ok, warn, bad are told apart (the value text takes the tone) */
        for (i = 0; i < REC.nt; i++) { ok += REC.t[i].rgba == AT_C_JADE; warn += REC.t[i].rgba == AT_C_SUN; bad += REC.t[i].rgba == AT_C_ROSE; }
        CHECK(ok >= 1 && warn >= 1 && bad >= 1);
    }
    /* a narrow plate falls back to one column; nothing leaves it */
    {
        AtReadout one = d; one.cols = 1;
        r.w = 200.0f; r.h = at_readout_height(&one);
        s = rec_sink(); at_part_readout(&s, &FAKE, r, &one);
        CHECK(lint_text_inside(r, 0) == 0 && lint_text_overlaps() == 0);
    }
    /* the worst strings: a 39-character value and a 23-character label are cut by the fit rule inside their column */
    memset(d.row[3].value, 'v', 39); d.row[3].value[39] = '\0'; memset(d.row[3].label, 'l', 23); d.row[3].label[23] = '\0';
    r.w = 400.0f; r.h = at_readout_height(&d);
    s = rec_sink(); at_part_readout(&s, &FAKE, r, &d);
    CHECK(lint_text_inside(r, 0) == 0 && lint_text_overlaps() == 0);
    /* 16 rows is the cap, 0 rows is a title only */
    d = ro(16, 1); r.h = at_readout_height(&d); s = rec_sink(); at_part_readout(&s, &FAKE, r, &d); CHECK(lint_text_inside(r, 0) == 0);
    d = ro(0, 1); r.h = at_readout_height(&d); s = rec_sink(); at_part_readout(&s, &FAKE, r, &d); CHECK(find_text("P1 FOX") != NULL && lint_text_inside(r, 0) == 0);
    /* a tiny plate: no text with no room for it, nothing at 12 px or under */
    d = ro(3, 1); r.w = 30.0f; r.h = at_readout_height(&d); s = rec_sink(); at_part_readout(&s, &FAKE, r, &d); CHECK(lint_text_inside(r, 0) == 0);
}

static AtTrack tk(int len, int now)
{
    AtTrack t;
    memset(&t, 0, sizeof t);
    snprintf(t.title, sizeof t.title, "%s", "P1 FOX  AttackS3S"); snprintf(t.right, sizeof t.right, "%s", "f 5 / 26   IASA 20"); snprintf(t.note, sizeof t.note, "%s", "f5-9 #0 9% a45   f12-14 #1 7% a361");
    t.len = len; t.now = now;
    t.n_spans = 3; t.span[0].from = 5; t.span[0].to = 9; t.span[0].id = 0; t.span[1].from = 12; t.span[1].to = 14; t.span[1].id = 1; t.span[2].from = 20; t.span[2].to = 22; t.span[2].id = 7;
    t.n_marks = 5; t.mark[0].frame = 20; t.mark[0].kind = AT_MK_IASA; t.mark[1].frame = 3; t.mark[1].kind = AT_MK_INVINC; t.mark[2].frame = 6; t.mark[2].kind = AT_MK_GFX;
    t.mark[3].frame = 8; t.mark[3].kind = AT_MK_SFX; t.mark[4].frame = 10; t.mark[4].kind = AT_MK_VIS;
    return t;
}

static void track(void)
{
    AtSink s;
    AtTrack t = tk(26, 5);
    AtRect r;
    int i;
    r.x = 8.0f; r.y = 350.0f; r.w = 624.0f; r.h = at_track_height();
    s = rec_sink(); at_part_track(&s, &FAKE, r, &t);
    CHECK(lint_plate_corners(r, 8.0f) == 0);
    CHECK(lint_text_inside(r, 0) == 0 && lint_text_overlaps() == 0 && texts_legible());
    CHECK(find_text("P1 FOX  AttackS3S") != NULL && find_text("f 5 / 26   IASA 20") != NULL);
    /* a hit window is never colour alone: its id is a number over it when it fits (a 5 frame window at 624 px is wide) */
    CHECK(find_text("#0") != NULL && find_text("#1") != NULL);
    CHECK(count_color(AT_C_ROSE) >= 1 && count_color(AT_C_SUN) >= 1 && count_color(AT_C_JADE) >= 1);   /* ids 0, 1, and the IASA mark */
    /* each mark kind has its own shape: five kinds draw distinct polygon footprints below the bar */
    {
        float seen[5]; int ns = 0, k;
        for (i = 0; i < REC.np; i++) {
            float w = poly_maxx(&REC.p[i]) - poly_minx(&REC.p[i]), h = poly_maxy(&REC.p[i]) - poly_miny(&REC.p[i]);
            if (poly_miny(&REC.p[i]) < r.y + 46.0f || poly_maxy(&REC.p[i]) > r.y + 64.0f || w > 12.0f) continue;
            for (k = 0; k < ns && !(fabsf(seen[k] - (w * 100.0f + h)) < 0.5f); k++) ;
            if (k == ns && ns < 5) seen[ns++] = w * 100.0f + h;
        }
        CHECK(ns >= 4);                                                              /* bar, square, two triangles and a disc: at least four footprints */
    }
    /* the playhead is inside the bar for any 'now', also when 'now' is out of range or the move is one frame long */
    { int nows[5] = { 0, 1, 26, 99, -4 }, k;
      for (k = 0; k < 5; k++) { AtTrack u = tk(26, nows[k]); s = rec_sink(); at_part_track(&s, &FAKE, r, &u); CHECK(lint_text_inside(r, 0) == 0); } }
    { AtTrack u = tk(1, 1); s = rec_sink(); at_part_track(&s, &FAKE, r, &u); CHECK(lint_text_inside(r, 0) == 0); }          /* length 1: no division by zero */
    { AtTrack u = tk(0, 0); s = rec_sink(); at_part_track(&s, &FAKE, r, &u); CHECK(REC.np > 0); }                           /* length 0: drawn as 1 */
    /* a 600-frame move: windows are at least 2 px, and ids are dropped when they do not fit rather than overlapped */
    { AtTrack u = tk(600, 100); s = rec_sink(); at_part_track(&s, &FAKE, r, &u); CHECK(lint_text_inside(r, 0) == 0 && lint_text_overlaps() == 0); }
    /* spans and marks outside the move are clamped, never drawn outside the bar */
    { AtTrack u = tk(26, 5); u.span[0].from = -5; u.span[0].to = 900; u.mark[0].frame = 5000; s = rec_sink(); at_part_track(&s, &FAKE, r, &u);
      for (i = 0; i < REC.np; i++) CHECK(poly_minx(&REC.p[i]) >= r.x - 0.5f && poly_maxx(&REC.p[i]) <= r.x + r.w + 0.5f); }
    /* a narrow plate (a 4:3 corner): still inside */
    r.w = 300.0f; s = rec_sink(); at_part_track(&s, &FAKE, r, &t); CHECK(lint_text_inside(r, 0) == 0 && lint_text_overlaps() == 0);
    r.w = 520.0f; s = rec_sink(); at_part_track(&s, &FAKE, r, &t); CHECK(lint_text_inside(r, 0) == 0 && lint_text_overlaps() == 0);
}

static void chips(void)
{
    AtSink s;
    AtChips d;
    AtRect r;
    int i;
    memset(&d, 0, sizeof d);
    snprintf(d.c[0].text, sizeof d.c[0].text, "%s", "FRAMES"); d.c[0].tone = 1; d.c[0].on = -1;
    snprintf(d.c[1].text, sizeof d.c[1].text, "%s", "T Timeline"); d.c[1].on = 1;
    snprintf(d.c[2].text, sizeof d.c[2].text, "%s", "H Hits"); d.c[2].on = 0;
    d.n = 3; snprintf(d.right, sizeof d.right, "%s", "f 120  rewind 4.2 s");
    r.x = 8.0f; r.y = 300.0f; r.w = 380.0f; r.h = at_chips_height();
    s = rec_sink(); at_part_chips(&s, &FAKE, r, &d);
    CHECK(find_text("FRAMES") != NULL && find_text("T Timeline") != NULL && find_text("H Hits") != NULL && find_text("f 120  rewind 4.2 s") != NULL);   /* the mode is a word */
    CHECK(lint_text_inside(r, 0) == 0 && lint_text_overlaps() == 0 && texts_legible());
    CHECK(find_text("T Timeline")->rgba == AT_C_IVORY && find_text("H Hits")->rgba == AT_C_DIM);                 /* on and off differ in the text as well as in the square */
    CHECK(count_color(AT_C_JADE) >= 2);                                                                         /* the mode bar and the lit square */
    /* a narrow strip drops the chips that do not fit (the last first); nothing is cut or overlapped */
    r.w = 150.0f; s = rec_sink(); at_part_chips(&s, &FAKE, r, &d);
    CHECK(lint_text_inside(r, 0) == 0 && lint_text_overlaps() == 0 && find_text("FRAMES") != NULL && find_text("H Hits") == NULL);
    /* twelve chips with long text, at several widths */
    memset(&d, 0, sizeof d);
    for (i = 0; i < AT_CHIPS_MAX; i++) { snprintf(d.c[i].text, sizeof d.c[i].text, "%s", "W Wide label here"); d.c[i].on = i & 1; }
    d.n = AT_CHIPS_MAX;
    { static const float ws[3] = { 220.0f, 380.0f, 560.0f }; int k; for (k = 0; k < 3; k++) { r.w = ws[k]; s = rec_sink(); at_part_chips(&s, &FAKE, r, &d); CHECK(lint_text_inside(r, 0) == 0 && lint_text_overlaps() == 0); } }
    r.w = 0.0f; s = rec_sink(); at_part_chips(&s, &FAKE, r, &d); CHECK(REC.nt == 0);                         /* no room: nothing */
}

int main(void)
{
    readout();
    track();
    chips();
    ATLAS_DONE("atlas hud parts");
}
