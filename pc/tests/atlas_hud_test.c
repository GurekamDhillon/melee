#include <stdlib.h>
#include "atlas_check.h"
#include "atlas_fake.h"
#include "atlas_rec.h"
#include "../platform/gw_ui_hud.h"
#include "../platform/gw_ui_val.h"

static const float WIDTHS[3] = { 640.0f, 853.0f, 1140.0f };
static int meets(AtRect a, AtRect b) { return a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h; }
static AtHud envoy_fixture(void)                   /* what Envoy's HUD holds at its busiest legal moment */
{
    AtHud h; int i; memset(&h, 0, sizeof h);
    snprintf(h.id, sizeof h.id, "%s", "envoy.hud");
    h.z[AT_Z_TOP_LEFT][0].kind = AT_HP_STRIP; h.z[AT_Z_TOP_LEFT][0].strip.n_pips = 6; h.z[AT_Z_TOP_LEFT][0].strip.n_keys = 3; h.n[AT_Z_TOP_LEFT] = 1;
    h.z[AT_Z_TOP_RIGHT][0].kind = AT_HP_TOAST; snprintf(h.z[AT_Z_TOP_RIGHT][0].text, AT_STR, "SKYWARD ASSEMBLED");
    snprintf(h.z[AT_Z_TOP_RIGHT][0].rule, AT_TEXT, "Your aerials gain Haste for 2 s."); h.z[AT_Z_TOP_RIGHT][0].until_ms = 5000.0;
    for (i = 1; i <= 3; i++) { h.z[AT_Z_TOP_RIGHT][i].kind = AT_HP_CARD; snprintf(h.z[AT_Z_TOP_RIGHT][i].text, AT_STR, "MARTH  CPU %d", i);
        snprintf(h.z[AT_Z_TOP_RIGHT][i].lines[0], AT_STR, "Pyromancer: All your attacks become fire."); h.z[AT_Z_TOP_RIGHT][i].n_lines = 1; }
    h.n[AT_Z_TOP_RIGHT] = 4;
    h.z[AT_Z_TOP_CENTER][0].kind = AT_HP_BANNER; h.z[AT_Z_TOP_CENTER][0].btn = 'A'; h.z[AT_Z_TOP_CENTER][0].progress = -1.0f;
    snprintf(h.z[AT_Z_TOP_CENTER][0].text, AT_STR, "Collect the drives"); h.n[AT_Z_TOP_CENTER] = 1;
    h.z[AT_Z_BOTTOM_LEFT][0].kind = AT_HP_NOTE; snprintf(h.z[AT_Z_BOTTOM_LEFT][0].text, AT_STR, "Merged: Lingering got stronger");
    h.z[AT_Z_BOTTOM_LEFT][0].until_ms = 5000.0; h.n[AT_Z_BOTTOM_LEFT] = 1;
    return h;
}
static void keepout_and_safe(void)
{
    int w, z, i, k;
    for (w = 0; w < 3; w++) {
        AtHud h = envoy_fixture(); AtKeepOut ko; AtHudLayout l; AtRect safe = at_hud_safe(WIDTHS[w]);
        at_hud_retail_keepouts(WIDTHS[w], 0xFFFFFFFFu, &ko);                   /* every retail element visible */
        CHECK(ko.n == 2);
        {   float ox = (WIDTHS[w] - 640.0f) * 0.5f;                            /* the measured retail HUD (Atlas proof, 640x480 canvas): plates + stocks, then the timer */
            CHECK(ko.r[0].x == ox + 40.0f && ko.r[0].y == 356.0f && ko.r[0].w == 560.0f && ko.r[0].h == 100.0f);
            CHECK(ko.r[1].x == ox + 246.0f && ko.r[1].y == 44.0f && ko.r[1].w == 168.0f && ko.r[1].h == 40.0f);
        }
        at_hud_layout(&h, WIDTHS[w], &ko, 1000.0, &FAKE, &l);
        for (z = 0; z < AT_Z_COUNT; z++) for (i = 0; i < h.n[z]; i++) {
            AtRect r = l.rect[z][i];
            if (!l.shown[z][i]) continue;
            CHECK(r.x >= safe.x - 0.01f && r.x + r.w <= safe.x + safe.w + 0.01f && r.y >= safe.y - 0.01f && r.y + r.h <= safe.y + safe.h + 0.01f);
            for (k = 0; k < ko.n; k++) CHECK(!meets(r, ko.r[k]));            /* never over the retail percent or the timer */
        }
        CHECK(l.shown[AT_Z_TOP_CENTER][0]);                                    /* the banner moved below the timer, not dropped */
        CHECK(l.rect[AT_Z_TOP_CENTER][0].y >= ko.r[1].y + ko.r[1].h);
    }
}
static void hidden_element_frees_space(void)
{
    AtKeepOut ko;
    at_hud_retail_keepouts(640.0f, ~(1u << AT_RE_HUD_TIMER), &ko);            /* the timer is masked */
    CHECK(ko.n == 1);
}
static void caps(void)
{
    AtHud h = envoy_fixture(); char why[96];
    CHECK(at_hud_cap_ok(&h, why, sizeof why));
    h.z[AT_Z_TOP_LEFT][1].kind = AT_HP_BANNER; h.n[AT_Z_TOP_LEFT] = 2;
    CHECK(!at_hud_cap_ok(&h, why, sizeof why) && strstr(why, "banner") != NULL);
    h = envoy_fixture(); h.z[AT_Z_BOTTOM_RIGHT][0].kind = AT_HP_TOAST; h.n[AT_Z_BOTTOM_RIGHT] = 1;
    CHECK(!at_hud_cap_ok(&h, why, sizeof why));                                /* a toast lives in a top corner */
    h = envoy_fixture(); h.z[AT_Z_TOP_LEFT][1].kind = AT_HP_CARD; h.n[AT_Z_TOP_LEFT] = 2;
    CHECK(!at_hud_cap_ok(&h, why, sizeof why) && strstr(why, "three") != NULL);   /* a fourth opponent card */
}
static void expiry_and_render_style(void)
{
    int w;
    for (w = 0; w < 3; w++) {
        AtHud h = envoy_fixture(); AtKeepOut ko; AtHudLayout l; AtSink s = rec_sink(); int entries = 0;
        at_hud_retail_keepouts(WIDTHS[w], 0xFFFFFFFFu, &ko);
        at_hud_layout(&h, WIDTHS[w], &ko, 6000.0, &FAKE, &l);              /* past the toast's and the note's until_ms */
        CHECK(!l.shown[AT_Z_TOP_RIGHT][0] && !l.shown[AT_Z_BOTTOM_LEFT][0]);
        CHECK(l.rect[AT_Z_TOP_RIGHT][1].y <= at_hud_safe(WIDTHS[w]).y + 0.01f);   /* the cards move up into the free space */
        at_hud_render(&h, &l, 6000.0, 0, &FAKE, &s, &entries);
        CHECK(no_focus_cues() && texts_legible() && entries <= AT_HUD_QUAD_CAP);
    }
}
/* ---- the description -> record conversion (what gd.ui.hud copies from Lua) ---- */
static AtvArena ARENA;
static int tbl(void) { return atv_table(&ARENA); }
static int sset(int t, const char *k, const char *v) { return atv_set(&ARENA, t, k, atv_str(&ARENA, v)); }
static int nset(int t, const char *k, double v) { return atv_set(&ARENA, t, k, atv_num(&ARENA, v)); }
static int part(const char *kind) { int p = tbl(); sset(p, "kind", kind); return p; }
static void from_val(void)
{
    AtHud h; char err[160]; int root, zones, tl, tc, strip, pips, pip, keys, key, banner, tr, card, lines;
    atv_init(&ARENA);
    root = tbl(); sset(root, "id", "envoy.hud"); zones = tbl(); atv_set(&ARENA, root, "zones", zones);
    tl = tbl(); atv_set(&ARENA, zones, "top_left", tl);
    strip = part("strip"); atv_push(&ARENA, tl, strip);
    pips = tbl(); atv_set(&ARENA, strip, "pips", pips); pip = tbl(); nset(pip, "fill", (double) 0xF07474FFu); nset(pip, "ring", (double) 0xF2C14EFFu); atv_push(&ARENA, pips, pip);
    keys = tbl(); atv_set(&ARENA, strip, "keys", keys); key = tbl(); sset(key, "letter", "P"); nset(key, "rgba", (double) 0xB872F0FFu); atv_push(&ARENA, keys, key);
    sset(strip, "wait", "2 waiting");
    tc = tbl(); atv_set(&ARENA, zones, "top_center", tc);
    banner = part("banner"); sset(banner, "text", "Collect the drives"); sset(banner, "button", "A"); atv_push(&ARENA, tc, banner);
    tr = tbl(); atv_set(&ARENA, zones, "top_right", tr);
    card = part("card"); sset(card, "title", "MARTH"); lines = tbl(); atv_push(&ARENA, lines, atv_str(&ARENA, "Pyromancer")); atv_set(&ARENA, card, "lines", lines); atv_push(&ARENA, tr, card);
    CHECK(at_hud_from_val(&ARENA, root, "envoy", 1000.0, &h, err, sizeof err));
    CHECK_STR(h.id, "envoy.hud");
    CHECK(h.n[AT_Z_TOP_LEFT] == 1 && h.z[AT_Z_TOP_LEFT][0].kind == AT_HP_STRIP && h.z[AT_Z_TOP_LEFT][0].strip.n_pips == 1 && h.z[AT_Z_TOP_LEFT][0].strip.n_keys == 1);
    CHECK(h.z[AT_Z_TOP_LEFT][0].strip.pip_fill[0] == 0xF07474FFu && h.z[AT_Z_TOP_LEFT][0].strip.key_letter[0] == 'P');
    CHECK(h.z[AT_Z_TOP_CENTER][0].kind == AT_HP_BANNER && h.z[AT_Z_TOP_CENTER][0].btn == 'A' && h.z[AT_Z_TOP_CENTER][0].progress < 0.0f);
    CHECK(h.z[AT_Z_TOP_RIGHT][0].kind == AT_HP_CARD && h.z[AT_Z_TOP_RIGHT][0].n_lines == 1);
    CHECK(!at_hud_from_val(&ARENA, root, "other", 1000.0, &h, err, sizeof err) && strstr(err, "must start with") != NULL);
    { int r2 = tbl(), z2 = tbl(), l2 = tbl(); sset(r2, "id", "envoy.hud"); atv_set(&ARENA, r2, "zones", z2); atv_set(&ARENA, z2, "top_left", l2);
      atv_push(&ARENA, l2, part("wobble"));
      CHECK(!at_hud_from_val(&ARENA, r2, "envoy", 1000.0, &h, err, sizeof err) && strstr(err, "unknown kind") != NULL); }
    { int b2 = part("banner"); atv_push(&ARENA, tl, b2);                   /* a second banner, and in the wrong zone */
      CHECK(!at_hud_from_val(&ARENA, root, "envoy", 1000.0, &h, err, sizeof err) && strstr(err, "banner") != NULL && strstr(err, "gd.ui.hud") != NULL); }
    { int pc = part("port_card"); int z2 = tbl(); int r2 = tbl(); atv_set(&ARENA, r2, "id", atv_str(&ARENA, "envoy.hud")); atv_set(&ARENA, r2, "zones", z2);
      atv_set(&ARENA, z2, "top_left", tbl()); atv_push(&ARENA, atv_get(&ARENA, z2, "top_left"), pc); nset(pc, "port", 7);
      CHECK(!at_hud_from_val(&ARENA, r2, "envoy", 1000.0, &h, err, sizeof err) && strstr(err, "port 1 to 4") != NULL); }
    CHECK(at_hud_zone_by_name("top_right") == AT_Z_TOP_RIGHT && at_hud_zone_by_name("middle") == -1 && strcmp(at_hud_zone_name(AT_Z_BOTTOM_LEFT), "bottom_left") == 0);
}
/* Atlas step 7: the LAB's HUD (two readouts, a timeline, a chip strip, a notice) next to the retail HUD, at 4:3 and wide */
static AtHud lab_fixture(void)
{
    AtHud h; int i; AtHudPart *p;
    memset(&h, 0, sizeof h);
    snprintf(h.id, sizeof h.id, "%s", "geno-lab.hud");
    for (i = 0; i < 2; i++) {                                                                    /* the info panel: one readout per fighter, P1 left, P2 right, ten rows each */
        int z = i == 0 ? AT_Z_TOP_LEFT : AT_Z_TOP_RIGHT, r;
        p = &h.z[z][0]; p->kind = AT_HP_READOUT; snprintf(p->data.readout.title, AT_STR, "P%d FOX", i + 1); p->data.readout.cols = 1; p->data.readout.n = 10;
        for (r = 0; r < 10; r++) { snprintf(p->data.readout.row[r].label, 24, "Row %d", r); snprintf(p->data.readout.row[r].value, 40, "%d.5  air", r); }
        h.n[z] = 1;
    }
    p = &h.z[AT_Z_BOTTOM_CENTER][0]; p->kind = AT_HP_TRACK; snprintf(p->data.track.title, AT_STR, "P1 FOX  AttackS3S"); p->data.track.len = 26; p->data.track.now = 5;
    p->data.track.n_spans = 1; p->data.track.span[0].from = 5; p->data.track.span[0].to = 9; h.n[AT_Z_BOTTOM_CENTER] = 1;
    p = &h.z[AT_Z_BOTTOM_LEFT][0]; p->kind = AT_HP_CHIPS; p->data.chips.n = 3;
    snprintf(p->data.chips.c[0].text, 24, "FRAMES"); p->data.chips.c[0].on = -1; snprintf(p->data.chips.c[1].text, 24, "T Timeline"); p->data.chips.c[1].on = 1; snprintf(p->data.chips.c[2].text, 24, "H Hits");
    h.n[AT_Z_BOTTOM_LEFT] = 1;
    p = &h.z[AT_Z_TOP_CENTER][0]; p->kind = AT_HP_NOTE; snprintf(p->text, AT_STR, "Reloaded: ok"); p->tone = AT_NOTE_OK; p->until_ms = 5000.0; p->from_ms = 1000.0; h.n[AT_Z_TOP_CENTER] = 1;
    return h;
}
static void lab_hud_keeps_off_the_retail_hud(void)
{
    int w, z, i, k;
    for (w = 0; w < 3; w++) {
        AtHud h = lab_fixture(); AtKeepOut ko; AtHudLayout l; AtRect safe = at_hud_safe(WIDTHS[w]);
        char why[96];
        CHECK(at_hud_cap_ok(&h, why, sizeof why));
        at_hud_retail_keepouts(WIDTHS[w], 0xFFFFFFFFu, &ko);
        at_hud_layout(&h, WIDTHS[w], &ko, 1500.0, &FAKE, &l);
        CHECK(l.dropped == 0);                                                                   /* every part found room: none was dropped for the retail plates or timer */
        for (z = 0; z < AT_Z_COUNT; z++) for (i = 0; i < h.n[z]; i++) {
            AtRect r = l.rect[z][i];
            CHECK(l.shown[z][i]);
            CHECK(r.x >= safe.x - 0.01f && r.x + r.w <= safe.x + safe.w + 0.01f && r.y >= safe.y - 0.01f && r.y + r.h <= safe.y + safe.h + 0.01f);
            for (k = 0; k < ko.n; k++) CHECK(!meets(r, ko.r[k]));
        }
        {   int a, b;                                                                            /* and none on another */
            AtRect all[AT_Z_COUNT * AT_HUD_PER_ZONE]; int n = 0;
            for (z = 0; z < AT_Z_COUNT; z++) for (i = 0; i < h.n[z]; i++) if (l.shown[z][i]) all[n++] = l.rect[z][i];
            for (a = 0; a < n; a++) for (b = a + 1; b < n; b++) CHECK(!meets(all[a], all[b]));
        }
        {   AtSink s = rec_sink(); int entries = 0;                                              /* it draws, inside its budget */
            at_hud_render(&h, &l, 1500.0, 0, &FAKE, &s, &entries);
            CHECK(entries > 20 && entries < AT_HUD_QUAD_CAP && REC.nt > 20);
        }
    }
}
static void lab_parts_from_val(void)
{
    AtvArena *a = (AtvArena *) malloc(sizeof *a);
    AtHud h; char err[160]; int root, zones, list, part, rows, row, k;
    atv_init(a);
    root = atv_table(a); atv_set(a, root, "id", atv_str(a, "geno-lab.hud")); zones = atv_table(a); list = atv_table(a); part = atv_table(a);
    atv_set(a, part, "kind", atv_str(a, "readout")); atv_set(a, part, "title", atv_str(a, "P1 FOX")); rows = atv_table(a);
    row = atv_table(a); atv_set(a, row, "label", atv_str(a, "Motion")); atv_set(a, row, "value", atv_str(a, "Wait f1")); atv_set(a, row, "tone", atv_str(a, "ok")); atv_push(a, rows, row);
    atv_set(a, part, "rows", rows); atv_push(a, list, part); atv_set(a, zones, "top_left", list); atv_set(a, root, "zones", zones);
    CHECK(at_hud_from_val(a, root, "geno-lab", 1000.0, &h, err, sizeof err) == 1);
    CHECK(h.n[AT_Z_TOP_LEFT] == 1 && h.z[AT_Z_TOP_LEFT][0].kind == AT_HP_READOUT && h.z[AT_Z_TOP_LEFT][0].data.readout.n == 1 && h.z[AT_Z_TOP_LEFT][0].data.readout.row[0].tone == 1);
    /* a readout with 17 rows is refused, and so is a track with 17 spans; nothing is registered */
    atv_init(a);
    root = atv_table(a); atv_set(a, root, "id", atv_str(a, "geno-lab.hud")); zones = atv_table(a); list = atv_table(a); part = atv_table(a);
    atv_set(a, part, "kind", atv_str(a, "readout")); rows = atv_table(a);
    for (k = 0; k < 17; k++) { row = atv_table(a); atv_set(a, row, "label", atv_str(a, "r")); atv_push(a, rows, row); }
    atv_set(a, part, "rows", rows); atv_push(a, list, part); atv_set(a, zones, "top_left", list); atv_set(a, root, "zones", zones);
    CHECK(at_hud_from_val(a, root, "geno-lab", 1000.0, &h, err, sizeof err) == 0 && strstr(err, "at most 16 rows") != NULL);
    atv_init(a);
    root = atv_table(a); atv_set(a, root, "id", atv_str(a, "geno-lab.hud")); zones = atv_table(a); list = atv_table(a); part = atv_table(a);
    atv_set(a, part, "kind", atv_str(a, "track")); rows = atv_table(a);
    for (k = 0; k < 17; k++) { row = atv_table(a); atv_set(a, row, "from", atv_num(a, 1)); atv_push(a, rows, row); }
    atv_set(a, part, "spans", rows); atv_push(a, list, part); atv_set(a, zones, "bottom_center", list); atv_set(a, root, "zones", zones);
    CHECK(at_hud_from_val(a, root, "geno-lab", 1000.0, &h, err, sizeof err) == 0 && strstr(err, "at most 16 spans") != NULL);
    /* a chip strip and a note with a tone */
    atv_init(a);
    root = atv_table(a); atv_set(a, root, "id", atv_str(a, "geno-lab.hud")); zones = atv_table(a); list = atv_table(a); part = atv_table(a);
    atv_set(a, part, "kind", atv_str(a, "chips")); rows = atv_table(a);
    row = atv_table(a); atv_set(a, row, "text", atv_str(a, "FRAMES")); atv_set(a, row, "tone", atv_str(a, "ok")); atv_push(a, rows, row);
    row = atv_table(a); atv_set(a, row, "text", atv_str(a, "T Timeline")); atv_set(a, row, "on", atv_bool(a, 1)); atv_push(a, rows, row);
    atv_set(a, part, "items", rows); atv_push(a, list, part);
    part = atv_table(a); atv_set(a, part, "kind", atv_str(a, "note")); atv_set(a, part, "text", atv_str(a, "Not saved")); atv_set(a, part, "tone", atv_str(a, "err")); atv_push(a, list, part);
    atv_set(a, zones, "bottom_left", list); atv_set(a, root, "zones", zones);
    CHECK(at_hud_from_val(a, root, "geno-lab", 1000.0, &h, err, sizeof err) == 1);
    CHECK(h.z[AT_Z_BOTTOM_LEFT][0].kind == AT_HP_CHIPS && h.z[AT_Z_BOTTOM_LEFT][0].data.chips.n == 2 && h.z[AT_Z_BOTTOM_LEFT][0].data.chips.c[0].on == -1 && h.z[AT_Z_BOTTOM_LEFT][0].data.chips.c[1].on == 1);
    CHECK(h.z[AT_Z_BOTTOM_LEFT][1].kind == AT_HP_NOTE && h.z[AT_Z_BOTTOM_LEFT][1].tone == AT_NOTE_ERR);
    free(a);
}

int main(void) { lab_hud_keeps_off_the_retail_hud(); lab_parts_from_val(); keepout_and_safe(); hidden_element_frees_space(); caps(); expiry_and_render_style(); from_val(); ATLAS_DONE("atlas-hud"); }
