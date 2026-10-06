#include "atlas_check.h"
#include "atlas_fake.h"
#include "atlas_rec.h"
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

int main(void)
{
    plates(); rows(); values(); tabs_and_tags();
    ATLAS_DONE("atlas parts");
}
