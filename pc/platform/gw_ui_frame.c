#include "gw_ui_frame.h"

#include <string.h>

AtRect at_frame_hole(const AtLayout *L, AtFrameRect w)
{
    AtRect r;
    float band = L->canvas.w > 640.0f ? (L->canvas.w - 640.0f) * 0.5f : 0.0f;   /* retail's 4:3 band, centred */
    r.x = w.x + band; r.y = w.y; r.w = w.w; r.h = w.h;
    return r;
}

int at_frame_plates(const AtLayout *L, AtFrameRect win, AtRect out[4])
{
    AtRect h = at_frame_hole(L, win);
    float cw = L->canvas.w;
    int n = 0;
    if (h.w <= 0.5f || h.h <= 0.5f) {                                  /* no window: one plate, the whole canvas (nothing is framed) */
        out[n].x = 0.0f; out[n].y = 0.0f; out[n].w = cw; out[n].h = 480.0f;
        return 1;
    }
    {   AtRect top = { 0.0f, 0.0f, cw, h.y };                         if (top.h > 0.5f) out[n++] = top; }
    {   AtRect bot = { 0.0f, h.y + h.h, cw, 480.0f - (h.y + h.h) };   if (bot.h > 0.5f) out[n++] = bot; }
    {   AtRect lef = { 0.0f, h.y, h.x, h.h };                         if (lef.w > 0.5f) out[n++] = lef; }
    {   AtRect rig = { h.x + h.w, h.y, cw - (h.x + h.w), h.h };       if (rig.w > 0.5f) out[n++] = rig; }
    return n;
}

static int hits(AtRect a, AtRect b) { return a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h; }

void at_frame_slots(const AtLayout *L, AtFrameRect win, float ew, AtFrameSlots *s)
{
    AtRect h = at_frame_hole(L, win);
    float left_w = h.x, right_w = L->canvas.w - (h.x + h.w), bottom = L->body.y + L->body.h;
    memset(s, 0, sizeof *s);
    s->trail = L->trail;                 /* the places of step 1's layout that do not depend on the window */
    s->keys = L->keys;
    s->have_trail = !hits(s->trail, h);
    s->have_keys = !hits(s->keys, h);
    /* the explainer: the wider side plate if it holds the preset, else a card over the hole's bottom-right corner */
    if (right_w >= left_w && right_w >= ew + 16.0f) {
        s->explainer.x = h.x + h.w + AT_FRAME_GAP; s->explainer.w = ew; s->have_explainer = 1;
        if (s->explainer.w > right_w - 2.0f * AT_FRAME_GAP) s->explainer.w = right_w - 2.0f * AT_FRAME_GAP;
    } else if (left_w >= ew + 16.0f) {
        s->explainer.x = AT_FRAME_GAP; s->explainer.w = ew; s->have_explainer = 1;
    } else {
        s->explainer.w = ew; s->explainer.x = h.x + h.w - ew - AT_FRAME_GAP; s->explainer_overlaps = 1; s->have_explainer = 1;
    }
    s->explainer.h = AT_FRAME_EXPLAIN_H;
    if (s->explainer_overlaps) {
        s->explainer.y = h.y + h.h - AT_FRAME_EXPLAIN_H - AT_FRAME_GAP;
    } else {
        s->explainer.y = h.y > L->body.y ? h.y : L->body.y;
        if (s->explainer.y + s->explainer.h > bottom) s->explainer.y = bottom - s->explainer.h;
    }
    if (s->explainer.y < 0.0f) s->explainer.y = 0.0f;
    if (s->explainer.x < 0.0f) s->explainer.x = 0.0f;
}
