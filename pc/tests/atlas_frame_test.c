/* atlas-frame: the window, the plates around it and the chrome slots of a framed screen. No game. */
#include "atlas_check.h"
#include "atlas_fake.h"
#include "../platform/gw_ui_frame.h"

static float area(AtRect r) { return r.w > 0 && r.h > 0 ? r.w * r.h : 0.0f; }
static int overlaps(AtRect a, AtRect b) { return a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h && area(a) > 0 && area(b) > 0; }

int main(void)
{
    AtLayout L; AtFrameRect win = { 200, 90, 240, 260 }; AtRect hole, pl[4]; int n, i, j; float sum;
    static const float widths[3] = { 640.0f, 853.0f, 1140.0f };
    for (i = 0; i < 3; i++) {
        at_layout(widths[i], AT_PRESET_NORMAL, &L);
        hole = at_frame_hole(&L, win);
        CHECK_NEAR(hole.x, 200.0f + (widths[i] - 640.0f) * 0.5f);          /* retail's 4:3 band centred */
        CHECK_NEAR(hole.y, 90.0f); CHECK_NEAR(hole.w, 240.0f); CHECK_NEAR(hole.h, 260.0f);
        n = at_frame_plates(&L, win, pl);
        CHECK(n == 4);
        sum = 0.0f;
        for (j = 0; j < n; j++) { sum += area(pl[j]); CHECK(!overlaps(pl[j], hole)); }
        CHECK_NEAR(sum, widths[i] * 480.0f - hole.w * hole.h);             /* plates plus the hole are exactly the canvas */
        for (j = 1; j < n; j++) { int k; for (k = 0; k < j; k++) CHECK(!overlaps(pl[j], pl[k])); }
    }
    /* a window touching the left edge gives no left plate; a full-canvas window gives none at all; no window at all is one plate */
    at_layout(640.0f, AT_PRESET_NORMAL, &L);
    { AtFrameRect w2 = { 0, 90, 300, 260 }; CHECK(at_frame_plates(&L, w2, pl) == 3); }
    { AtFrameRect w3 = { 0, 0, 640, 480 }; CHECK(at_frame_plates(&L, w3, pl) == 0); }
    { AtFrameRect w0 = { 0, 0, 0, 0 }; n = at_frame_plates(&L, w0, pl); CHECK(n == 1 && area(pl[0]) == 640.0f * 480.0f); }

    /* chrome slots: trail and keys at the top and bottom plates, the explainer in the wider side plate; nothing silently overlaps the hole */
    { AtFrameSlots s; AtFrameRect w4 = { 40, 70, 330, 350 };               /* the window left of centre: the right plate is wide */
      at_layout(640.0f, AT_PRESET_NORMAL, &L); at_frame_slots(&L, w4, 160.0f, &s);
      hole = at_frame_hole(&L, w4);
      CHECK(!s.explainer_overlaps && s.have_explainer && s.explainer.x >= hole.x + hole.w && s.explainer.w >= 160.0f);
      CHECK(s.explainer.x + s.explainer.w <= 640.0f && s.explainer.y + s.explainer.h <= 480.0f);
      CHECK(s.have_trail && !overlaps(s.trail, hole));
      CHECK(s.have_keys && !overlaps(s.keys, hole)); }
    { AtFrameSlots s; AtFrameRect w5 = { 120, 60, 400, 380 };              /* wide window: no side plate fits the explainer; keys would hit the hole */
      at_layout(640.0f, AT_PRESET_NORMAL, &L); at_frame_slots(&L, w5, 160.0f, &s);
      hole = at_frame_hole(&L, w5);
      CHECK(s.explainer_overlaps && s.have_explainer);                       /* a card over the hole's corner, and it says so */
      CHECK(s.explainer.y >= hole.y && s.explainer.y + s.explainer.h <= hole.y + hole.h);
      CHECK(!s.have_keys);                                                   /* a slot that would sit on the model is dropped, not overlapped */ }
    { AtFrameSlots s; AtFrameRect w6 = { 0, 0, 640, 480 };
      at_layout(640.0f, AT_PRESET_NORMAL, &L); at_frame_slots(&L, w6, 160.0f, &s);
      CHECK(!s.have_trail && !s.have_keys && s.explainer_overlaps); }        /* a full-canvas window: only the overlap card can exist */
    /* at 16:9 the side plates grow: the explainer that overlapped at 640 fits */
    { AtFrameSlots s; AtFrameRect w7 = { 120, 60, 400, 380 };
      at_layout(1140.0f, AT_PRESET_NORMAL, &L); at_frame_slots(&L, w7, 160.0f, &s);
      CHECK(!s.explainer_overlaps && s.have_explainer); }
    /* the left plate takes the explainer when the window sits right of centre */
    { AtFrameSlots s; AtFrameRect w8 = { 330, 70, 300, 350 };
      at_layout(640.0f, AT_PRESET_NORMAL, &L); at_frame_slots(&L, w8, 160.0f, &s);
      hole = at_frame_hole(&L, w8);
      CHECK(!s.explainer_overlaps && s.explainer.x + s.explainer.w <= hole.x); }
    ATLAS_DONE("atlas frame");
}
