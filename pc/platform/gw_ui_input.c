#include "gw_ui_input.h"

#include <stddef.h>

#define EMIT(T, A, B) do { if (n < cap) { out[n].type = (T); out[n].a = (A); out[n].b = (B); n++; } } while (0)

static int dir_from(unsigned held_up, unsigned held_down, unsigned held_left, unsigned held_right)
{
    if (held_up) return AT_DIR_UP;
    if (held_down) return AT_DIR_DOWN;
    if (held_left) return AT_DIR_LEFT;
    if (held_right) return AT_DIR_RIGHT;
    return 0;
}

int at_pad_events(AtPad *p, unsigned b, int sx, int sy, double now, AtEvent *out, int cap)
{
    unsigned edge = b & ~p->prev;
    int n = 0, d;
    if (edge & AT_PAD_A) EMIT(AT_EV_ACCEPT, 0, 0);
    if (edge & AT_PAD_B) EMIT(AT_EV_BACK, 0, 0);
    if (edge & AT_PAD_X) EMIT(AT_EV_ALT, 'X', 0);
    if (edge & AT_PAD_Y) EMIT(AT_EV_ALT, 'Y', 0);
    if (edge & AT_PAD_Z) EMIT(AT_EV_ALT, 'Z', 0);
    if (edge & AT_PAD_L) EMIT(AT_EV_PAGE, -1, 0);
    else if (edge & AT_PAD_R) EMIT(AT_EV_PAGE, 1, 0);
    if (edge & AT_PAD_START) EMIT(AT_EV_START, 0, 0);
    d = dir_from((b & AT_PAD_UP) || sy > 60, (b & AT_PAD_DOWN) || sy < -60, (b & AT_PAD_LEFT) || sx < -60, (b & AT_PAD_RIGHT) || sx > 60);
    d = at_repeat_step(&p->rep, d, now);
    if (d) EMIT(AT_EV_MOVE, d, 0);
    p->prev = b;
    return n;
}

int at_key_events(AtKeys *k, unsigned m, double now, AtEvent *out, int cap)
{
    unsigned edge = m & ~k->prev;
    int n = 0, d;
    if (edge & AT_KEY_ENTER) EMIT(AT_EV_ACCEPT, 0, 0);
    if (edge & AT_KEY_ESC) EMIT(AT_EV_BACK, 0, 0);
    if (edge & AT_KEY_TAB) EMIT(AT_EV_PAGE, (m & AT_KEY_SHIFT) ? -1 : 1, 0);
    d = dir_from(m & AT_KEY_UP, m & AT_KEY_DOWN, m & AT_KEY_LEFT, m & AT_KEY_RIGHT);
    d = at_repeat_step(&k->rep, d, now);
    if (d) EMIT(AT_EV_MOVE, d, 0);
    k->prev = m;
    return n;
}

int at_hit_test(const AtHits *h, float x, float y)
{
    int i;
    for (i = h->n - 1; i >= 0; i--) {                       /* the latest drawn is the topmost */
        const AtRect *r = &h->h[i].r;
        if (x >= r->x && x < r->x + r->w && y >= r->y && y < r->y + r->h) return i;
    }
    return -1;
}

static void press_button(int ch, AtEvent *out, int cap, int *np)
{
    int n = *np;
    if (ch == 'A') EMIT(AT_EV_ACCEPT, 0, 0);
    else if (ch == 'B') EMIT(AT_EV_BACK, 0, 0);
    else if (ch == 'X' || ch == 'Y' || ch == 'Z') EMIT(AT_EV_ALT, ch, 0);
    else if (ch == 'L') EMIT(AT_EV_PAGE, -1, 0);
    else if (ch == 'R') EMIT(AT_EV_PAGE, 1, 0);
    else if (ch == 'S') EMIT(AT_EV_START, 0, 0);
    *np = n;
}

int at_mouse_events(AtMouse *m, float x, float y, int buttons, int wheel, const AtHits *hits, AtEvent *out, int cap)
{
    int n = 0, hit, moved, left, right;
    if (x < 0.0f || y < 0.0f) {                              /* off the picture (-1000): never an event */
        m->valid = 0;
        m->buttons = buttons;
        return 0;
    }
    moved = m->valid && (x != m->x || y != m->y);
    left = (buttons & 1) && !(m->buttons & 1) && m->valid;
    right = (buttons & 2) && !(m->buttons & 2) && m->valid;
    hit = at_hit_test(hits, x, y);
    if (moved && hit >= 0 && hits->h[hit].kind == AT_HIT_CELL) EMIT(AT_EV_FOCUS, hits->h[hit].a, hits->h[hit].b);
    if (left && hit >= 0) {
        const AtHit *h = &hits->h[hit];
        if (h->kind == AT_HIT_CELL) {
            if (!moved) EMIT(AT_EV_FOCUS, h->a, h->b);
            EMIT(AT_EV_ACCEPT, 0, 0);
        } else {
            press_button(h->a, out, cap, &n);
        }
    }
    if (right) EMIT(AT_EV_BACK, 0, 0);
    if (wheel != 0) EMIT(AT_EV_SCROLL, wheel > 0 ? -1 : 1, 0);
    m->valid = 1;
    m->x = x;
    m->y = y;
    m->buttons = buttons;
    return n;
}
