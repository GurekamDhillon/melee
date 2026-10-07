#include "atlas_check.h"
#include "atlas_fake.h"
#include "atlas_rec.h"
#include "../platform/gw_ui_hud.h"

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
int main(void) { keepout_and_safe(); hidden_element_frees_space(); caps(); expiry_and_render_style(); ATLAS_DONE("atlas-hud"); }
