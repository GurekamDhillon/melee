#define AT_SINK_HAS_IMAGE
#include "atlas_check.h"
#include "atlas_fake.h"
#include "atlas_rec.h"
#include "atlas_style.h"
#include "../platform/gw_ui_tokens.h"

static void plates(void)
{
    float px[3][4], py[3][4];
    int i, k;
    float total = 0.0f, edge_h;
    AtRect r = { 10.0f, 20.0f, 100.0f, 40.0f };
    CHECK(at_plate_polys(r, 3.0f, 8.0f, px, py) == 3);
    for (i = 0; i < 3; i++) { RecPoly p; for (k = 0; k < 4; k++) { p.x[k] = px[i][k]; p.y[k] = py[i][k]; } total += poly_area(&p); }
    CHECK_NEAR(total, 100.0f * 40.0f - 8.0f * 8.0f);                      /* the silhouette: a rectangle less two chamfer triangles */
    { RecPoly p; for (k = 0; k < 4; k++) { p.x[k] = px[2][k]; p.y[k] = py[2][k]; } edge_h = poly_maxy(&p) - poly_miny(&p); }
    CHECK_NEAR(edge_h, 3.0f);                                              /* the front edge is exactly its thickness */
    CHECK(at_plate_polys(r, 9.0f, 8.0f, px, py) == 3);                     /* an edge never outgrows the chamfer */
    { AtSink s = rec_sink(); at_plate(&s, r, AT_C_PLATE, AT_C_EDGE, 3.0f, 8.0f); CHECK(REC.np == 3); CHECK(REC.p[2].rgba == AT_C_EDGE && REC.p[0].rgba == AT_C_PLATE); }
    { AtSink s = rec_sink(); at_plate(&s, r, AT_C_PLATE, AT_C_EDGE, 6.0f, 8.0f); CHECK_NEAR(poly_maxy(&REC.p[2]) - poly_miny(&REC.p[2]), 6.0f); }   /* modal */
}

static AtItem item(const char *label, int vkind)
{
    AtItem it; memset(&it, 0, sizeof it);
    snprintf(it.label, sizeof it.label, "%s", label); it.vkind = vkind;
    return it;
}

static void rows(void)
{
    AtRect r = { 32.0f, 100.0f, 300.0f, 34.0f };
    AtItem it = item("Window", AT_VAL_NONE);
    AtSink s = rec_sink();
    int i; float top;
    at_part_row(&s, &FAKE, r, &it, AT_ST_REST);
    CHECK(count_color(AT_C_EMBER) == 0 && count_color(AT_C_LIFT) == 0);    /* rest: no focus cues */
    CHECK(REC.nt == 1 && REC.t[0].rgba == AT_C_TEXT2 && REC.t[0].role == AT_R_ROW16);
    CHECK(texts_legible());
    s = rec_sink();
    at_part_row(&s, &FAKE, r, &it, AT_ST_FOCUS);
    top = 1e9f; for (i = 0; i < REC.np; i++) if (poly_miny(&REC.p[i]) < top) top = poly_miny(&REC.p[i]);
    CHECK_NEAR(top, 98.0f);                                                /* lifted 2 px */
    CHECK(count_color(AT_C_LIFT) == 2 && count_color(AT_C_EMBER) == 2);    /* face, ember edge and the ember tick: three cues */
    CHECK(REC.t[0].rgba == AT_C_IVORY);
    s = rec_sink();
    at_part_row(&s, &FAKE, r, &it, AT_ST_PRESS);
    top = 1e9f; for (i = 0; i < REC.np; i++) if (poly_miny(&REC.p[i]) < top) top = poly_miny(&REC.p[i]);
    CHECK_NEAR(top, 101.0f); CHECK(count_color(AT_C_EMBER_D) >= 2);        /* pressed: drops 1 px, darker edge */
    s = rec_sink(); it.flags = AT_CELL_DISABLED;
    at_part_row(&s, &FAKE, r, &it, AT_ST_REST);
    CHECK(REC.t[0].rgba == AT_C_DIM && count_color(AT_C_EMBER) == 0);
    s = rec_sink(); it.flags = AT_CELL_SELECTED;
    at_part_row(&s, &FAKE, r, &it, AT_ST_REST);
    CHECK(count_color(AT_C_JADE) == 1);                                    /* selected: the jade bar */
    /* D1: a focused disabled row keeps the disabled look (dim text, plate) and gains the lift and the ember edge and tick */
    s = rec_sink(); it.flags = AT_CELL_DISABLED;
    at_part_row(&s, &FAKE, r, &it, AT_ST_FOCUS);
    top = 1e9f; for (i = 0; i < REC.np; i++) if (poly_miny(&REC.p[i]) < top) top = poly_miny(&REC.p[i]);
    CHECK_NEAR(top, 98.0f);
    CHECK(REC.t[0].rgba == AT_C_DIM && count_color(AT_C_EMBER) == 2 && count_color(AT_C_LIFT) == 0);
}

static void values(void)
{
    AtRect r = { 32.0f, 100.0f, 300.0f, 34.0f };
    AtItem t = item("Sync", AT_VAL_TOGGLE), c = item("Mode", AT_VAL_CHOICE), sl = item("Vol", AT_VAL_SLIDER), tx = item("Code", AT_VAL_TEXT);
    AtSink s;
    t.on = 1;
    s = rec_sink(); at_part_row(&s, &FAKE, r, &t, AT_ST_REST);
    CHECK(find_text("OFF") != NULL && find_text("ON") != NULL && count_color(AT_C_JADE) == 1);   /* the lit half is jade and says ON */
    CHECK(find_text("ON")->rgba == AT_C_INK && find_text("OFF")->rgba == AT_C_DIM);
    t.on = 0; s = rec_sink(); at_part_row(&s, &FAKE, r, &t, AT_ST_REST);
    CHECK(count_color(AT_C_JADE) == 0 && find_text("OFF")->rgba == AT_C_IVORY);
    snprintf(c.text, sizeof c.text, "Window");
    s = rec_sink(); at_part_row(&s, &FAKE, r, &c, AT_ST_FOCUS);
    CHECK(find_text("Window") != NULL);
    CHECK(count_color(AT_C_EMBER) >= 4);                                   /* the arrows light up in focus */
    sl.vmin = 0; sl.vmax = 100; sl.vval = 50;
    s = rec_sink(); at_part_row(&s, &FAKE, r, &sl, AT_ST_REST);
    CHECK(REC.np >= 12);                                                   /* ticks */
    snprintf(tx.text, sizeof tx.text, "ABC-123");
    s = rec_sink(); at_part_row(&s, &FAKE, r, &tx, AT_ST_REST);
    CHECK(find_text("ABC-123") != NULL && find_text("ABC-123")->role == AT_R_NUM14 && find_text("ABC-123")->align == AT_ALIGN_RIGHT);
    CHECK_NEAR(find_text("ABC-123")->x, 32.0f + 300.0f - 12.0f);          /* right aligned, 12 px from the edge */
}

static void tabs_and_tags(void)
{
    AtRect r = { 32.0f, 70.0f, 300.0f, 30.0f };
    static const char *const names[3] = { "VIDEO", "AUDIO", "CONTROLS" };
    static const int counts[3] = { 5, 3, 12 };
    AtSink s = rec_sink();
    float w;
    at_part_tabs(&s, &FAKE, r, names, counts, 3, 1, -1);
    CHECK(find_text("VIDEO") && find_text("AUDIO") && find_text("CONTROLS") && find_text("12"));
    CHECK(find_text("AUDIO")->rgba == AT_C_IVORY && find_text("VIDEO")->rgba == AT_C_MUTED);     /* the active tab is the bright one */
    CHECK(texts_legible());
    s = rec_sink();
    w = at_part_tag(&s, &FAKE, 40.0f, 200.0f, "GENO", AT_TAG_JADE, 0.0f);
    CHECK_NEAR(w, fake_width(0, AT_R_CAP12, "GENO") + 16.0f);
    CHECK(count_color(AT_C_JADE_D) >= 1 && find_text("GENO")->rgba == AT_C_IVORY);
    s = rec_sink();
    w = at_part_tag(&s, &FAKE, 40.0f, 200.0f, "A VERY LONG ORIGIN TAG TEXT", AT_TAG_PLAIN, 60.0f);
    CHECK(w <= 60.0f + 0.01f);                                             /* a tag never outgrows its slot */
}

static AtCell mk_cell(const char *name, int model, unsigned flags)
{
    AtCell c; memset(&c, 0, sizeof c);
    snprintf(c.id, sizeof c.id, "c"); snprintf(c.name, sizeof c.name, "%s", name);
    c.model = model; c.ring = AT_NO_MODEL; c.flags = flags;
    return c;
}

static void cells(void)
{
    AtRect r = { 100.0f, 100.0f, 56.0f, 56.0f };
    AtCell c = mk_cell("Kindling", 7, 0);
    AtSink s = rec_sink();
    int rest;
    c.ring = 9;
    at_part_cell(&s, &FAKE, r, &c, AT_ST_REST, 0);
    CHECK(REC.nm == 1 && REC.m[0].model == 7 && REC.m[0].ring == 9 && REC.m[0].focused == 0 && REC.m[0].dim == 0);
    CHECK(REC.m[0].x >= r.x && REC.m[0].x + REC.m[0].w <= r.x + r.w && REC.m[0].y >= r.y && REC.m[0].y + REC.m[0].h <= r.y + r.h);   /* the model stays inside the cell */
    rest = count_color(AT_C_EMBER);
    CHECK(rest == 0 && find_text("Kindling") == NULL);                      /* a cell with a model shows only the model */
    s = rec_sink();
    at_part_cell(&s, &FAKE, r, &c, AT_ST_FOCUS, 0);
    CHECK(REC.m[0].focused == 1);
    CHECK(count_color(AT_C_EMBER) >= 9);                                    /* the ember edge and eight bracket strips */
    s = rec_sink();
    at_part_cell(&s, &FAKE, r, &c, AT_ST_FOCUS, AT_C_P2);
    CHECK(count_color(AT_C_P2) == 8);                                       /* brackets take the focusing port's colour */
    c = mk_cell("Locked slot", AT_NO_MODEL, AT_CELL_LOCKED);
    s = rec_sink(); at_part_cell(&s, &FAKE, r, &c, AT_ST_REST, 0);
    CHECK(REC.nm == 0 && count_color(AT_C_DIM) >= 4);                       /* the lock glyph, no model, no name */
    c = mk_cell("Empty slot", AT_NO_MODEL, AT_CELL_EMPTY);
    s = rec_sink(); at_part_cell(&s, &FAKE, r, &c, AT_ST_REST, 0);
    CHECK(REC.nm == 0 && count_color(AT_C_LINE) == 4 && count_color(AT_C_LINE2) == 2);   /* an outline and a plus, no plate */
    c = mk_cell("Cell", 7, AT_CELL_MERGE);
    s = rec_sink(); at_part_cell(&s, &FAKE, r, &c, AT_ST_REST, 0);
    CHECK(find_text("+ MERGE") != NULL && count_color(AT_C_JADE) >= 1);
    c = mk_cell("Cell", 7, 0); c.pips = 3; c.origin = 'G'; c.index = 2;
    s = rec_sink(); at_part_cell(&s, &FAKE, r, &c, AT_ST_REST, 0);
    CHECK(find_text("G") != NULL && find_text("2") != NULL && count_color(AT_C_JADE) == 3 + 1 /* pips + origin mark */);
    { /* D2: the pips sit in a strip below the name, inside the cell, and clear of it */
      const RecText *nm = NULL; float pip_top = 1e9f; int i;
      c = mk_cell("Jigglypuff", AT_NO_MODEL, 0); c.pips = 3;
      s = rec_sink(); at_part_cell(&s, &FAKE, r, &c, AT_ST_REST, 0);
      nm = find_text("Jigglypuff"); if (nm == NULL) nm = &REC.t[0];
      for (i = 0; i < REC.np; i++) if (REC.p[i].rgba == AT_C_JADE) { if (poly_miny(&REC.p[i]) < pip_top) pip_top = poly_miny(&REC.p[i]); CHECK(poly_maxx(&REC.p[i]) <= r.x + r.w && poly_minx(&REC.p[i]) >= r.x); }
      CHECK(count_color(AT_C_JADE) == 3 && nm->base + 2.0f <= pip_top);       /* the name's baseline (plus descent) ends above the first pip */
    }
    c = mk_cell("Mr. Game & Watch", AT_NO_MODEL, 0);                        /* no model: the name, fitted into the cell */
    s = rec_sink(); at_part_cell(&s, &FAKE, r, &c, AT_ST_REST, 0);
    CHECK(REC.nt == 1 && REC.t[0].role == AT_R_BODY12 && REC.t[0].align == AT_ALIGN_CENTER);
    CHECK(texts_legible());
    c = mk_cell("Stone", AT_NO_MODEL, 0); c.letter = 'P'; c.rgba = 0xB872F0FFu;
    { AtRect sr = { 100.0f, 100.0f, 34.0f, 38.0f };
      s = rec_sink(); at_part_stone(&s, &FAKE, sr, &c, AT_ST_REST, 0);
      CHECK(find_text("P") != NULL && count_color(0xB872F0FFu) == 1);
      c.flags = AT_CELL_EMPTY; s = rec_sink(); at_part_stone(&s, &FAKE, sr, &c, AT_ST_REST, 0); CHECK(find_text("P") == NULL);
      c.flags = AT_CELL_LOCKED; s = rec_sink(); at_part_stone(&s, &FAKE, sr, &c, AT_ST_REST, 0); CHECK(count_color(AT_C_DIM) >= 4); }
}

static void hints_and_chrome(void)
{
    AtSink s = rec_sink();
    float adv;
    static const char *const trail3[3] = { "SOLO", "ENVOY", "YOUR DRIVES" };
    AtRect tr = { 32.0f, 22.0f, 480.0f, 30.0f };
    adv = at_part_hint(&s, &FAKE, 40.0f, 450.0f, 'A', "Merge into slot 1");
    CHECK(REC.np == 3 && REC.p[0].rgba == AT_C_PAD_A);                      /* the octagon disc */
    CHECK(find_text("Merge into slot 1") != NULL && find_text("A") != NULL);
    CHECK(adv > fake_width(0, AT_R_BODY14, "Merge into slot 1") + 18.0f);
    s = rec_sink(); at_part_hint(&s, &FAKE, 40.0f, 450.0f, 'B', "Close"); CHECK(REC.p[0].rgba == AT_C_PAD_B);
    s = rec_sink(); at_part_hint(&s, &FAKE, 40.0f, 450.0f, 'Z', "Bag"); CHECK(REC.p[0].rgba == AT_C_PAD_Z);
    s = rec_sink(); at_part_hint(&s, &FAKE, 40.0f, 450.0f, 'S', "Fight"); CHECK(find_text("START") != NULL);
    s = rec_sink(); at_part_hint(&s, &FAKE, 40.0f, 450.0f, 'M', "Move"); CHECK(REC.np == 2);   /* the d-pad cross */

    s = rec_sink();
    at_part_trail(&s, &FAKE, tr, trail3, 3);
    CHECK(find_text("YOUR DRIVES")->role == AT_R_CAP20 && find_text("YOUR DRIVES")->rgba == AT_C_IVORY);   /* here: bright and large */
    CHECK(find_text("SOLO")->role == AT_R_CAP16 && find_text("SOLO")->rgba == AT_C_MUTED);
    CHECK(count_color(AT_C_EMBER) >= 1);                                    /* the mark */
    s = rec_sink();
    tr.w = 150.0f;                                                          /* too narrow: the earliest parents drop out */
    at_part_trail(&s, &FAKE, tr, trail3, 3);
    CHECK(find_text("YOUR DRIVES") != NULL && find_text("SOLO") == NULL);
    s = rec_sink();
    at_part_chapter(&s, &FAKE, (AtRect){ 496.0f, 26.0f, 112.0f, 20.0f }, 2);
    CHECK(find_text("I") && find_text("II") && find_text("III") && find_text("IV") && find_text("V"));
    CHECK(count_color(AT_C_EMBER) == 1 && find_text("II")->rgba == AT_C_INK);
    s = rec_sink();
    at_part_rail(&s, &FAKE, (AtRect){ 32.0f, 66.0f, 104.0f, 362.0f }, 2);
    CHECK(find_text("SOLO") && find_text("VERSUS") && find_text("SETTINGS") && find_text("VERSUS")->rgba == AT_C_IVORY);
}

static void explainer_note_dialog(void)
{
    AtRect pane = { 448.0f, 66.0f, 160.0f, 362.0f };
    AtExplainer e;
    AtSink s;
    int i;
    memset(&e, 0, sizeof e);
    e.has = 1; e.media_model = 5; e.media_ring = 6;
    snprintf(e.kicker, sizeof e.kicker, "BAG CELL 1"); snprintf(e.title, sizeof e.title, "KINDLING");
    snprintf(e.what, sizeof e.what, "Your hits set the target Burning for 3 s.");
    e.n_with = 2; e.with_model[0] = 8; e.with_model[1] = 9; snprintf(e.from_text, sizeof e.from_text, "Depth 0, Fire");
    s = rec_sink();
    at_part_explainer(&s, &FAKE, pane, &e);
    CHECK(find_text("BAG CELL 1")->role == AT_R_CAP14 && find_text("BAG CELL 1")->rgba == AT_C_JADE);   /* WHAT: kicker, title, one rule */
    CHECK(find_text("KINDLING")->role == AT_R_TITLE);
    CHECK(find_text("target Burning for") != NULL);                         /* wrapped inside the 136 px inner width */
    CHECK(find_text("WITH") != NULL && find_text("FROM") != NULL && find_text("Depth 0, Fire") != NULL);
    CHECK(REC.nm == 3);                                                     /* the big media model and two WITH cells */
    for (i = 0; i < REC.nt; i++) CHECK(REC.t[i].base <= pane.y + pane.h && REC.t[i].x >= pane.x);
    CHECK(texts_legible());
    {                                                                       /* a long unbroken rule: clamped, still inside the pane */
        char big[400];
        memset(big, 'x', 150); big[150] = '\0';
        for (i = 0; i < 40; i++) memcpy(big + 150 + 5 * i, " word", 6);
        snprintf(e.what, sizeof e.what, "%s", big);
    }
    s = rec_sink(); at_part_explainer(&s, &FAKE, pane, &e);
    for (i = 0; i < REC.nt; i++) CHECK(REC.t[i].base <= pane.y + pane.h - 12.0f + 0.01f);
    e.has = 0; s = rec_sink(); at_part_explainer(&s, &FAKE, pane, &e);
    CHECK(REC.np == 3 && REC.nt == 0);                                      /* nothing to explain: an empty plate */

    s = rec_sink();
    at_part_note(&s, &FAKE, (AtRect){ 300.0f, 22.0f, 300.0f, 30.0f }, "Merged! Kindling got stronger.", AT_NOTE_OK, 0.5f);
    CHECK(count_color(AT_C_JADE) >= 2 && find_text("Merged! Kindling got stronger.")->role == AT_R_BODY14);   /* icon square and the timer line */
    s = rec_sink();
    at_part_note(&s, &FAKE, (AtRect){ 300.0f, 22.0f, 200.0f, 30.0f }, "Careful", AT_NOTE_WARN, 1.0f);
    CHECK(count_color(AT_C_SUN) >= 1);

    { AtDialog d; AtRect b[2]; int n;
      memset(&d, 0, sizeof d);
      d.open = 1; snprintf(d.title, sizeof d.title, "DISCARD DRIVE?"); snprintf(d.body, sizeof d.body, "This drive is gone for good.");
      d.n = 2; d.btn[0] = 'A'; snprintf(d.label[0], 24, "Discard"); d.btn[1] = 'B'; snprintf(d.label[1], 24, "Cancel"); d.focus = 1;
      s = rec_sink();
      n = at_part_dialog(&s, &FAKE, 640.0f, &d, 0.0f, b);
      CHECK(n == 2);
      CHECK(REC.p[0].rgba == AT_C_SCRIM && poly_minx(&REC.p[0]) == 0.0f && poly_maxx(&REC.p[0]) == 640.0f && poly_maxy(&REC.p[0]) == 480.0f);   /* the scrim covers the canvas */
      CHECK(find_text("DISCARD DRIVE?") != NULL && find_text("Discard") != NULL && find_text("Cancel") != NULL);
      CHECK(b[0].x >= 150.0f && b[1].x + b[1].w <= 640.0f - 150.0f && b[0].x + b[0].w <= b[1].x);   /* buttons inside the 332 px plate, not overlapping */
      s = rec_sink(); n = at_part_dialog(&s, &FAKE, 1706.0f, &d, 0.0f, b);
      CHECK(b[0].x > 600.0f && b[1].x + b[1].w < 1100.0f);                   /* centred on a wide canvas */
    }
}

static void fix_round1(void)
{
    AtRect r = { 32.0f, 100.0f, 300.0f, 34.0f };
    AtItem it = item("Window", AT_VAL_NONE), lg = item("A really quite long row label that cannot fit", AT_VAL_NONE);
    AtSink s;
    int i, k;
    float c = (float) AT_PX_CH_S;
    /* the focus and press ticks stay inside the chamfered silhouette: nothing coloured ember touches the cut-away corner */
    for (k = 0; k < 2; k++) {
        s = rec_sink();
        at_part_row(&s, &FAKE, r, &it, k ? AT_ST_PRESS : AT_ST_FOCUS);
        for (i = 0; i < REC.np; i++) {
            unsigned col = REC.p[i].rgba;
            if (col == AT_C_EMBER || col == AT_C_EMBER_D) {
                float top = poly_miny(&REC.p[i]), left = poly_minx(&REC.p[i]), ry = k ? 101.0f : 98.0f;
                if (left <= 32.0f + 0.01f && poly_maxy(&REC.p[i]) - top < 30.0f) CHECK(top >= ry + c - 0.01f);   /* a tick on the left edge starts below the chamfer */
            }
        }
    }
    /* a long label in a 60 px plate: every text stays inside the plate */
    { AtRect n = { 32.0f, 100.0f, 60.0f, 34.0f };
      s = rec_sink(); at_part_row(&s, &FAKE, n, &lg, AT_ST_REST);
      for (i = 0; i < REC.nt; i++) CHECK(REC.t[i].x + fake_width(0, REC.t[i].role, REC.t[i].s) <= 32.0f + 60.0f + 0.01f);
      s = rec_sink(); lg.vkind = AT_VAL_TOGGLE; at_part_row(&s, &FAKE, n, &lg, AT_ST_REST);   /* no room at all for the label: it is not drawn unfitted */
      CHECK(find_text(lg.label) == NULL);
    }
    /* a tag with a slot narrower than its padding draws no unfitted text */
    s = rec_sink(); (void) at_part_tag(&s, &FAKE, 0.0f, 0.0f, "ABCDEFGH", AT_TAG_PLAIN, 10.0f);
    for (i = 0; i < REC.nt; i++) CHECK(fake_width(0, REC.t[i].role, REC.t[i].s) <= 0.01f);
    /* a focused tab: lift 2 px, an ember front edge and a tick */
    { AtRect tr = { 32.0f, 70.0f, 300.0f, 30.0f };
      static const char *const nm[3] = { "VIDEO", "AUDIO", "CONTROLS" };
      static const int ct[3] = { 5, 3, 12 };
      float top = 1e9f;
      s = rec_sink(); at_part_tabs(&s, &FAKE, tr, nm, ct, 3, 1, 0);
      for (i = 0; i < REC.np; i++) if (REC.p[i].rgba == AT_C_GROUND2 && poly_miny(&REC.p[i]) < top) top = poly_miny(&REC.p[i]);
      CHECK_NEAR(top, 72.0f);                                              /* an unfocused tab top is 74; focused it is lifted 2 */
      CHECK(count_color(AT_C_EMBER) == 2);                                 /* the front edge and the tick */
    }
    /* a wide tab set stays inside r.w, with and without counts */
    { AtRect tr = { 32.0f, 70.0f, 200.0f, 30.0f };
      static const char *const nm[4] = { "VIDEO SETTINGS", "AUDIO SETTINGS", "CONTROLS", "ACCESSIBILITY" };
      static const int ct[4] = { 5, 3, 12, 7 };
      for (k = 0; k < 2; k++) {
          float maxr = 0.0f;
          s = rec_sink(); at_part_tabs(&s, &FAKE, tr, nm, k ? NULL : ct, 4, 0, -1);
          for (i = 0; i < REC.np; i++) if (poly_maxx(&REC.p[i]) > maxr) maxr = poly_maxx(&REC.p[i]);
          CHECK(maxr <= 32.0f + 200.0f + 0.01f);
          for (i = 0; i < REC.nt; i++) CHECK(REC.t[i].x + fake_width(0, REC.t[i].role, REC.t[i].s) <= 32.0f + 200.0f + 0.01f);
      }
    }
}

/* ---- round 2: the chamfer rule, the three focus cues, and text that never leaves its box ---------------- */

/* the right edge of a recorded text, whatever its alignment */
static float text_right(const RecText *t)
{
    float w = fake_width(0, t->role, t->s);
    return t->align == AT_ALIGN_RIGHT ? t->x : t->align == AT_ALIGN_CENTER ? t->x + w * 0.5f : t->x + w;
}
static float text_left(const RecText *t)
{
    float w = fake_width(0, t->role, t->s);
    return t->align == AT_ALIGN_RIGHT ? t->x - w : t->align == AT_ALIGN_CENTER ? t->x - w * 0.5f : t->x;
}
/* 1 when no polygon vertex lies inside the cut-away triangle of the top-left or bottom-right chamfer of r */
static int corners_clear(AtRect r, float c)
{
    int i, k;
    for (i = 0; i < REC.np; i++) {
        for (k = 0; k < 4; k++) {
            float dx = REC.p[i].x[k] - r.x, dy = REC.p[i].y[k] - r.y, ex = r.x + r.w - REC.p[i].x[k], ey = r.y + r.h - REC.p[i].y[k];
            if (dx >= -0.01f && dy >= -0.01f && dx + dy < c - 0.01f) return 0;
            if (ex >= -0.01f && ey >= -0.01f && ex + ey < c - 0.01f) return 0;
        }
    }
    return 1;
}
static float min_y_of(unsigned rgba)
{
    float m = 1e9f; int i;
    for (i = 0; i < REC.np; i++) if (REC.p[i].rgba == rgba && poly_miny(&REC.p[i]) < m) m = poly_miny(&REC.p[i]);
    return m;
}

static void fix_round2(void)
{
    AtSink s;
    int i;
    AtRect r = { 100.0f, 100.0f, 56.0f, 56.0f };
    AtCell c;
    /* nothing is drawn over a chamfered corner: the note's icon, the merge and selected outlines */
    { AtRect nr = { 300.0f, 22.0f, 300.0f, 30.0f };
      s = rec_sink(); at_part_note(&s, &FAKE, nr, "Saved", AT_NOTE_OK, 0.5f);
      CHECK(corners_clear(nr, (float) AT_PX_CH_S)); }
    c = mk_cell("Cell", 7, AT_CELL_MERGE);
    s = rec_sink(); at_part_cell(&s, &FAKE, r, &c, AT_ST_REST, 0);
    CHECK(corners_clear(r, (float) AT_PX_CH_XS));
    c = mk_cell("Cell", 7, AT_CELL_SELECTED);
    s = rec_sink(); at_part_cell(&s, &FAKE, r, &c, AT_ST_REST, 0);
    CHECK(corners_clear(r, (float) AT_PX_CH_XS) && count_color(AT_C_JADE) >= 3);
    /* a focused stone: lift 2, an ember front edge, four brackets in the port colour */
    c = mk_cell("Stone", AT_NO_MODEL, 0); c.letter = 'P'; c.rgba = 0xB872F0FFu;
    { AtRect sr = { 100.0f, 100.0f, 34.0f, 38.0f };
      s = rec_sink(); at_part_stone(&s, &FAKE, sr, &c, AT_ST_FOCUS, AT_C_P2);
      CHECK(count_color(AT_C_P2) == 8 && count_color(AT_C_EMBER) == 1 && poly_miny(&REC.p[0]) == 98.0f);
      c.flags = AT_CELL_EMPTY;
      s = rec_sink(); at_part_stone(&s, &FAKE, sr, &c, AT_ST_FOCUS, AT_C_P2);
      CHECK(count_color(AT_C_P2) == 8 && count_color(AT_C_EMBER) == 1 && poly_miny(&REC.p[0]) == 98.0f); }
    /* a focused empty cell: lift 2, an ember outline with a front edge, brackets */
    c = mk_cell("Empty", AT_NO_MODEL, AT_CELL_EMPTY);
    s = rec_sink(); at_part_cell(&s, &FAKE, r, &c, AT_ST_FOCUS, AT_C_P2);
    CHECK(count_color(AT_C_P2) == 8 && count_color(AT_C_EMBER) >= 4 && min_y_of(AT_C_EMBER) == 98.0f && count_color(AT_C_LINE) == 0);
    /* a pressed cell sits 1 px low with the dark ember edge; a disabled cell shows it */
    c = mk_cell("Cell", 7, 0);
    s = rec_sink(); at_part_cell(&s, &FAKE, r, &c, AT_ST_PRESS, 0);
    CHECK(count_color(AT_C_EMBER_D) >= 1 && count_color(AT_C_EMBER) == 0 && REC.m[0].y == 104.0f);
    s = rec_sink(); at_part_cell(&s, &FAKE, r, &c, AT_ST_DISABLED, 0);
    CHECK(REC.m[0].dim == 1 && REC.m[0].focused == 0);
    s = rec_sink(); at_part_cell(&s, &FAKE, r, &c, AT_ST_DISABLED, AT_C_P2);
    CHECK(count_color(AT_C_P2) == 0);                                       /* a disabled cell takes no focus */
    c = mk_cell("Slot", AT_NO_MODEL, AT_CELL_LOCKED);                      /* a focused locked cell (it takes focus and explains itself) shows the cues */
    s = rec_sink(); at_part_cell(&s, &FAKE, r, &c, AT_ST_FOCUS, AT_C_P2);
    CHECK(count_color(AT_C_P2) == 8 && count_color(AT_C_EMBER) >= 1);
    /* a focused dialog button: lift 2, an ember front edge and tick, four brackets */
    { AtDialog d; AtRect b[2];
      memset(&d, 0, sizeof d);
      d.open = 1; snprintf(d.title, sizeof d.title, "DISCARD?"); snprintf(d.body, sizeof d.body, "Gone for good.");
      d.n = 2; d.btn[0] = 'A'; snprintf(d.label[0], 24, "Discard"); d.btn[1] = 'B'; snprintf(d.label[1], 24, "Cancel"); d.focus = 1;
      s = rec_sink(); at_part_dialog(&s, &FAKE, 640.0f, &d, 0.0f, b);
      CHECK(count_color(AT_C_EMBER) == 10);                                 /* edge, tick, eight bracket strips */
      CHECK(min_y_of(AT_C_LIFT) == b[1].y - 2.0f);
      /* two 23-character labels stay inside the 332 px plate */
      snprintf(d.label[0], 24, "ABCDEFGHIJKLMNOPQRSTUVW"); snprintf(d.label[1], 24, "abcdefghijklmnopqrstuvw");
      s = rec_sink(); at_part_dialog(&s, &FAKE, 640.0f, &d, 0.0f, b);
      CHECK(b[0].x >= 154.0f + 16.0f - 0.01f && b[1].x + b[1].w <= 486.0f - 16.0f + 0.01f && b[0].x + b[0].w <= b[1].x);
      for (i = 0; i < REC.nt; i++) CHECK(text_left(&REC.t[i]) >= 154.0f && text_right(&REC.t[i]) <= 486.0f);
      CHECK(texts_legible()); }
    /* a long trail in a narrow rect: no text past the right edge, however narrow */
    { static const char *const tl[3] = { "SOLO", "ENVOY", "A VERY LONG CURRENT PAGE TITLE HERE" };
      static const float widths[5] = { 300.0f, 150.0f, 100.0f, 60.0f, 20.0f };
      int k;
      for (k = 0; k < 5; k++) {
          AtRect tr = { 32.0f, 22.0f, widths[k], 30.0f };
          s = rec_sink(); at_part_trail(&s, &FAKE, tr, tl, 3);
          for (i = 0; i < REC.nt; i++) CHECK(text_right(&REC.t[i]) <= tr.x + tr.w + 0.01f);
      } }
    /* a narrow footer and a long hint label stay inside */
    { AtFooter f; AtRect fr = { 32.0f, 434.0f, 200.0f, 26.0f };
      memset(&f, 0, sizeof f);
      f.has = 1; f.model_a = f.model_b = f.model_out = AT_NO_MODEL;
      snprintf(f.label, sizeof f.label, "COMBINE"); snprintf(f.text, sizeof f.text, "A long outcome sentence that has no room");
      s = rec_sink(); at_part_footer(&s, &FAKE, fr, &f);
      for (i = 0; i < REC.nt; i++) CHECK(text_right(&REC.t[i]) <= fr.x + fr.w + 0.01f);
      for (i = 0; i < REC.nm; i++) CHECK(REC.m[i].x + REC.m[i].w <= fr.x + fr.w + 0.01f); }
    { char lab[201]; float adv;
      memset(lab, 'q', 200); lab[200] = '\0';
      s = rec_sink(); adv = at_part_hint(&s, &FAKE, 40.0f, 450.0f, 'A', lab);
      for (i = 0; i < REC.nt; i++) CHECK(text_right(&REC.t[i]) <= 40.0f + adv);
      CHECK(adv < 400.0f); }
}

static const AtTextOps O = { fake_width, NULL };

static void sink_without_image_is_safe(void)
{
    AtSink s = rec_sink(); AtCell c; AtRect r = { 40, 100, 40, 40 };
    memset(&c, 0, sizeof c); c.model = AT_NO_MODEL; c.tex = 7; snprintf(c.abbr, sizeof c.abbr, "%s", "FO");
    s.image = NULL;                                                   /* a sink with no image op */
    at_part_cell(&s, &O, r, &c, AT_ST_REST, AT_C_P1);                /* must not crash and must fall back to the abbreviation */
    CHECK(find_text("FO") != NULL && REC.ni == 0);
    at_sink_image(&s, 3, 0, 0, 10, 10, 0xFFFFFFFFu); CHECK(REC.ni == 0);   /* the helper is NULL-safe too */
    s = rec_sink(); at_sink_image(&s, -1, 0, 0, 10, 10, 0xFFFFFFFFu); CHECK(REC.ni == 0);   /* and refuses a negative texture */
}
static void image_cell(void)
{
    AtSink s = rec_sink(); AtCell c; AtRect r = { 40, 100, 40, 40 };
    memset(&c, 0, sizeof c); c.model = AT_NO_MODEL; c.tex = 7; snprintf(c.abbr, sizeof c.abbr, "%s", "FO");
    at_part_cell(&s, &O, r, &c, AT_ST_REST, AT_C_P1);
    CHECK(REC.ni == 1 && REC.im[0].tex == 7);
    CHECK(REC.im[0].x >= r.x && REC.im[0].y >= r.y && REC.im[0].x + REC.im[0].w <= r.x + r.w + 0.01f && REC.im[0].y + REC.im[0].h <= r.y + r.h + 0.01f);   /* inside the cell */
    CHECK_NEAR(REC.im[0].w / REC.im[0].h, 64.0f / 56.0f);                      /* an icon keeps its 64x56 aspect */
    c.tex = -1;                                                       /* no art: the frame and the abbreviation, no image */
    { AtSink s2 = rec_sink(); at_part_cell(&s2, &O, r, &c, AT_ST_REST, AT_C_P1); CHECK(REC.ni == 0 && find_text("FO") != NULL); }
    { AtRect wide = { 40, 100, 80, 56 }; AtSink s2 = rec_sink(); at_part_cell(&s2, &O, wide, &c, AT_ST_REST, AT_C_P1);   /* the word only where it fits the cell */
      CHECK(find_text("DISC ART") != NULL && sty_text_inside(wide, 0) == -1); }
    { AtSink s2 = rec_sink(); at_part_cell(&s2, &O, r, &c, AT_ST_REST, AT_C_P1); CHECK(find_text("DISC ART") == NULL); }   /* never squeezed into a small cell */
    /* a cell with no abbreviation never draws an image, even with a texture number: a zeroed record cannot show texture 0 */
    memset(&c, 0, sizeof c); c.model = AT_NO_MODEL; snprintf(c.name, sizeof c.name, "%s", "Drive");
    { AtSink s2 = rec_sink(); at_part_cell(&s2, &O, r, &c, AT_ST_REST, AT_C_P1); CHECK(REC.ni == 0); }
}
static void cell_style(void)
{
    AtSink s; AtCell c; AtRect r = { 40, 100, 40, 40 }; StySig a, b;
    memset(&c, 0, sizeof c); c.model = AT_NO_MODEL; snprintf(c.abbr, sizeof c.abbr, "%s", "FO"); c.tex = -1;
    s = rec_sink(); at_poly_rect(&s, 0, 0, 640, 480, AT_C_PLATE); at_part_cell(&s, &O, r, &c, AT_ST_REST, AT_C_P1);
    CHECK(sty_chamfer(r, 3.0f, AT_C_PLATE) == 0);                    /* the cell's 3 px chamfers show the pane under it */
    CHECK(sty_text_inside(r, 0) == -1);
    s = rec_sink(); at_part_cell(&s, &O, r, &c, AT_ST_REST, AT_C_P1); a = sty_sig(0);
    s = rec_sink(); at_part_cell(&s, &O, r, &c, AT_ST_FOCUS, AT_C_P1); b = sty_sig(0);
    CHECK(sty_focus_cues(a, b) == 3);
    c.flags = AT_CELL_LOCKED; s = rec_sink(); at_part_cell(&s, &O, r, &c, AT_ST_DISABLED, AT_C_P1);
    CHECK(find_text("FO") != NULL && texts_legible() && count_color(AT_C_DIM) >= 4);   /* locked: the word and the lock glyph, not only dimmed */
    CHECK(sty_text_inside(r, 0) == -1);
    c.flags = AT_CELL_DISABLED; s = rec_sink(); at_part_cell(&s, &O, r, &c, AT_ST_DISABLED, AT_C_P1);
    CHECK(find_text("FO") != NULL && texts_legible() && sty_text_inside(r, 0) == -1);
    CHECK(count_color(AT_C_ROSE) >= 1);                              /* a disabled art cell is struck through (a shape), not only dimmed */
}
static void port_cards(void)
{
    AtSink s; AtPortCard pc[4]; AtRect r = { 32, 372, 140, 56 }; int p;
    memset(pc, 0, sizeof pc);
    for (p = 0; p < 4; p++) {
        pc[p].port = p; pc[p].kind = 1; snprintf(pc[p].name, sizeof pc[p].name, "%s", "SORA"); snprintf(pc[p].sub, sizeof pc[p].sub, "Costume 1");
        snprintf(pc[p].abbr, sizeof pc[p].abbr, "%s", "SO"); pc[p].ck_tex = -1;
        s = rec_sink(); at_poly_rect(&s, 0, 0, 640, 480, AT_C_GROUND);
        at_part_port_card(&s, &O, r, &pc[p], 0);
        CHECK(sty_chamfer(r, 5.0f, AT_C_GROUND) == 0);
        CHECK(sty_text_inside(r, 0) == -1);
        CHECK(find_text("SORA") != NULL);
    }
    { int shape[4]; for (p = 0; p < 4; p++) { AtSink s2 = rec_sink(); shape[p] = at_port_mark(&s2, 50, 50, 12, p, AT_C_P1); CHECK(shape[p] == REC.np); } CHECK(sty_shapes_distinct(shape) == 1); }
    /* every port's numeral is drawn (colour is never the only signal) */
    { char want[2] = { 0, 0 }; for (p = 0; p < 4; p++) { AtSink s2 = rec_sink(); want[0] = (char) ('1' + p); at_part_port_card(&s2, &O, r, &pc[p], 0); CHECK(find_text(want) != NULL); } }
    /* each port's top edge is its own colour */
    { static const unsigned col[4] = { AT_C_P1, AT_C_P2, AT_C_P3, AT_C_P4 }; for (p = 0; p < 4; p++) { AtSink s2 = rec_sink(); at_part_port_card(&s2, &O, r, &pc[p], 0); CHECK(count_color(col[p]) >= 2); } }
    /* CPU: the word CPU, a level */
    pc[1].kind = 2; pc[1].cpu_lv = 9; { AtSink s2 = rec_sink(); at_part_port_card(&s2, &O, r, &pc[1], 0); CHECK(find_text("CPU") != NULL && find_text("LV 9") != NULL && sty_text_inside(r, 0) == -1); }
    /* an open slot and a closed one say so in words */
    pc[2].kind = 0; { AtSink s2 = rec_sink(); at_part_port_card(&s2, &O, r, &pc[2], 0); CHECK(find_text("OPEN") != NULL && find_text("SORA") == NULL); }
    pc[2].flags = AT_CARD_CLOSED; { AtSink s2 = rec_sink(); at_part_port_card(&s2, &O, r, &pc[2], 0); CHECK(find_text("CLOSED") != NULL && find_text("OPEN") == NULL); }
    /* a focused card shows the three cues; an open focused card on a wide slot says how to join */
    { StySig a, b; AtSink s2; pc[3].kind = 1;
      s2 = rec_sink(); at_part_port_card(&s2, &O, r, &pc[3], 0); a = sty_sig(0);
      s2 = rec_sink(); at_part_port_card(&s2, &O, r, &pc[3], 1); b = sty_sig(0);
      CHECK(sty_focus_cues(a, b) == 3);
      pc[2].flags = AT_CARD_OPEN; s2 = rec_sink(); at_part_port_card(&s2, &O, r, &pc[2], 1); CHECK(find_text("Press A to join") != NULL && sty_text_inside(r, 0) == -1);
      s2 = rec_sink(); at_part_port_card(&s2, &O, r, &pc[2], 0); CHECK(find_text("Press A to join") == NULL); }
    /* a narrow card never lets text outside it */
    { AtRect nr = { 32, 372, 90, 56 }; pc[0].kind = 1; snprintf(pc[0].name, sizeof pc[0].name, "%s", "CAPTAIN FALCON THE FIRST"); { AtSink s2 = rec_sink(); at_part_port_card(&s2, &O, nr, &pc[0], 1); CHECK(sty_text_inside(nr, 0) == -1 && texts_legible()); } }
}
static void matchup_strip(void)
{
    AtPortCard c[2]; AtRect r = { 32, 380, 400, 40 }; AtSink s;
    memset(c, 0, sizeof c);
    c[0].port = 0; c[0].kind = 1; snprintf(c[0].name, AT_STR, "%s", "SORA"); c[1].port = 1; c[1].kind = 2; snprintf(c[1].name, AT_STR, "%s", "KIRBY");
    s = rec_sink(); at_poly_rect(&s, 0, 0, 640, 480, AT_C_GROUND); at_part_matchup(&s, &O, r, c, 2, -1);
    CHECK(find_text("SORA") != NULL && find_text("KIRBY") != NULL && find_text("VS") != NULL && sty_text_inside(r, 0) == -1 && texts_legible());
    CHECK(sty_chamfer(r, 5.0f, AT_C_GROUND) == 0);
    s = rec_sink(); at_part_matchup(&s, &O, r, c, 2, 1); CHECK(count_color(AT_C_EMBER) >= 1);          /* the picker's card carries the ember mark */
    s = rec_sink(); at_part_matchup(&s, &O, r, c, 2, -1); CHECK(count_color(AT_C_EMBER) == 0);
    { AtRect nr = { 32, 380, 150, 40 }; s = rec_sink(); at_part_matchup(&s, &O, nr, c, 2, -1); CHECK(sty_text_inside(nr, 0) == -1); }   /* narrow: names fit or drop */
    s = rec_sink(); at_part_matchup(&s, &O, r, c, 0, -1); CHECK(REC.nt == 0);                          /* nothing to show: only the plate */
}
static void explainer_art_and_stepper(void)
{
    AtRect pane = { 412.0f, 66.0f, 196.0f, 362.0f };
    AtExplainer e; AtSink s; int i;
    memset(&e, 0, sizeof e);
    e.has = 1; e.media_model = AT_NO_MODEL; e.media_ring = AT_NO_MODEL; e.media_tex = 12; snprintf(e.media_abbr, sizeof e.media_abbr, "%s", "FX");
    snprintf(e.kicker, sizeof e.kicker, "FIGHTER"); snprintf(e.title, sizeof e.title, "FOX");
    snprintf(e.what, sizeof e.what, "Fast and light.");
    snprintf(e.stepper_label, sizeof e.stepper_label, "COSTUME"); snprintf(e.stepper_text, sizeof e.stepper_text, "2 / 4"); e.stepper = 1;
    s = rec_sink(); at_part_explainer(&s, &O, pane, &e);
    CHECK(REC.ni == 1 && REC.im[0].tex == 12 && REC.im[0].x >= pane.x && REC.im[0].x + REC.im[0].w <= pane.x + pane.w);   /* the portrait, inside the well */
    CHECK_NEAR(REC.im[0].w / REC.im[0].h, 136.0f / 188.0f);
    CHECK(find_text("COSTUME") != NULL && find_text("2 / 4") != NULL && find_text("Fast and light.") != NULL && texts_legible());
    for (i = 0; i < REC.nt; i++) CHECK(REC.t[i].base <= pane.y + pane.h - 12.0f + 0.01f && REC.t[i].x >= pane.x);
    CHECK(sty_text_inside(pane, 0) == -1);
    e.media_tex = -1; s = rec_sink(); at_part_explainer(&s, &O, pane, &e);
    CHECK(REC.ni == 0 && find_text("FX") != NULL && find_text("DISC ART") != NULL && sty_text_inside(pane, 0) == -1);   /* no art: the letters and the word */
    memset(&e, 0, sizeof e); e.has = 1; e.media_model = AT_NO_MODEL; e.media_ring = AT_NO_MODEL; e.media_tex = 0;       /* a zeroed explainer never names texture 0 */
    s = rec_sink(); at_part_explainer(&s, &O, pane, &e); CHECK(REC.ni == 0);
    s = rec_sink(); at_part_explainer(&s, &O, (AtRect){ 448.0f, 66.0f, 160.0f, 362.0f }, &e); CHECK(REC.ni == 0);
}
/* the strike and ban marks (drawing only): each says what it is in words or a shape, stays inside the cell, and never relies on colour alone */
static void strike_marks(void)
{
    AtCell c; AtRect r = { 40, 100, 56, 56 }; AtSink s; StySig a, b;
    memset(&c, 0, sizeof c); c.model = AT_NO_MODEL; c.tex = -1; snprintf(c.abbr, sizeof c.abbr, "%s", "PC"); snprintf(c.name, sizeof c.name, "%s", "Peach's Castle");
    s = rec_sink(); at_poly_rect(&s, 0, 0, 640, 480, AT_C_PLATE); at_part_cell(&s, &O, r, &c, AT_ST_REST, AT_C_P1);
    CHECK(find_text("PC") != NULL && sty_text_inside(r, 0) == -1 && sty_chamfer(r, 3.0f, AT_C_PLATE) == 0);   /* the plain stage cell: letters, a name plate, inside */
    CHECK(REC.nt >= 2 && texts_legible());
    s = rec_sink(); at_part_cell(&s, &O, r, &c, AT_ST_REST, AT_C_P1); a = sty_sig(0);
    /* banned: the word BAN, a crossing in a shape, text inside the cell */
    c.flags = AT_CELL_BANNED | AT_CELL_P1;
    s = rec_sink(); at_poly_rect(&s, 0, 0, 640, 480, AT_C_PLATE); at_part_cell(&s, &O, r, &c, AT_ST_REST, AT_C_P1);
    CHECK(find_text("BAN") != NULL && count_color(AT_C_ROSE) >= 2 && sty_text_inside(r, 0) == -1 && texts_legible() && sty_chamfer(r, 3.0f, AT_C_PLATE) == 0);
    CHECK(find_text("1") != NULL);                                                          /* the port that struck it: its numeral ... */
    { AtSink s2 = rec_sink(); at_part_cell(&s2, &O, r, &c, AT_ST_REST, AT_C_P1); CHECK(REC.np > a.polys); }   /* ... and its shape */
    c.flags = AT_CELL_BANNED;                                                                /* the other port */
    s = rec_sink(); at_part_cell(&s, &O, r, &c, AT_ST_REST, AT_C_P1); CHECK(find_text("2") != NULL && find_text("1") == NULL && find_text("BAN") != NULL);
    c.flags = AT_CELL_BANNED | AT_CELL_UNSET;                                                /* struck by nobody known: the word, no numeral */
    s = rec_sink(); at_part_cell(&s, &O, r, &c, AT_ST_REST, AT_C_P1); CHECK(find_text("BAN") != NULL && find_text("1") == NULL && find_text("2") == NULL);
    /* picked: the ember ring (a shape) and the word PICK, the picker's numeral */
    c.flags = AT_CELL_PICKED | AT_CELL_P1;
    s = rec_sink(); at_part_cell(&s, &O, r, &c, AT_ST_REST, AT_C_P1); b = sty_sig(0);
    CHECK(b.ember_polys >= 4 && find_text("PICK") != NULL && find_text("1") != NULL && sty_text_inside(r, 0) == -1 && texts_legible());
    CHECK(find_text("BAN") == NULL);
    /* a small cell: the words are dropped when they do not fit, never squeezed under 12 px or out of the cell */
    { AtRect small = { 40, 100, 36, 36 }; c.flags = AT_CELL_BANNED | AT_CELL_P1; s = rec_sink(); at_part_cell(&s, &O, small, &c, AT_ST_REST, AT_C_P1);
      CHECK(sty_text_inside(small, 0) == -1 && texts_legible() && count_color(AT_C_ROSE) >= 2); }   /* the crossing still says it */
    /* ban and focus together: the three cues remain */
    c.flags = AT_CELL_BANNED | AT_CELL_P1; s = rec_sink(); at_part_cell(&s, &O, r, &c, AT_ST_REST, AT_C_P1); a = sty_sig(0);
    s = rec_sink(); at_part_cell(&s, &O, r, &c, AT_ST_FOCUS, AT_C_P2); b = sty_sig(0); CHECK(sty_focus_cues(a, b) == 3);
}
static void two_cursors_one_cell(void)
{
    AtSink s = rec_sink(); AtRect r = { 40, 100, 40, 40 };
    at_cell_brackets(&s, r, AT_C_P1, 0, 2); at_cell_brackets(&s, r, AT_C_P2, 1, 2);
    CHECK(count_color(AT_C_P1) == 8 && count_color(AT_C_P2) == 8);          /* four brackets of two strokes each, per port */
    { float a = REC.p[0].x[0], b = REC.p[8].x[0]; CHECK(a != b); }          /* the two sets do not sit on top of each other */
    s = rec_sink(); at_cell_brackets(&s, r, AT_C_P1, 0, 1); { float a = REC.p[0].x[0]; CHECK(a < r.x); }   /* one cursor: outside the cell, as the focus brackets are */
    { int slot; for (slot = 0; slot < 4; slot++) { AtSink s2 = rec_sink(); int i; at_cell_brackets(&s2, r, AT_C_P1, slot, 4);
        for (i = 0; i < REC.np; i++) CHECK(poly_minx(&REC.p[i]) >= r.x - 0.01f && poly_maxx(&REC.p[i]) <= r.x + r.w + 0.01f && poly_miny(&REC.p[i]) >= r.y - 0.01f && poly_maxy(&REC.p[i]) <= r.y + r.h + 0.01f); } }   /* four ports: all inside */
}

int main(void)
{
    plates(); rows(); values(); tabs_and_tags();
    cells(); hints_and_chrome(); explainer_note_dialog(); fix_round1(); fix_round2();
    sink_without_image_is_safe(); image_cell(); cell_style(); port_cards(); matchup_strip(); two_cursors_one_cell(); explainer_art_and_stepper(); strike_marks();
    ATLAS_DONE("atlas parts");
}
